use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use turbofish_deluxe::{
    adventure::{AdventurePhase, AdventureSession, PetKind},
    cli,
    sim::{Action, AdventureState, StinkyOrigin},
};

fn temporary_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "turbofish-{name}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn atomic_save_replaces_complete_snapshot_and_retains_seeded_state() {
    let root = temporary_root("save");
    let path = root.join("adventure.json");
    let mut session = AdventureSession::new(42);
    cli::write_json(
        &path,
        &cli::ProjectSave {
            format_version: cli::SAVE_FORMAT_VERSION,
            session: session.clone(),
        },
    )
    .unwrap();
    for _ in 0..20 {
        session.step(&[]);
    }
    cli::write_json(
        &path,
        &cli::ProjectSave {
            format_version: cli::SAVE_FORMAT_VERSION,
            session: session.clone(),
        },
    )
    .unwrap();
    let loaded: cli::ProjectSave = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(loaded.format_version, 3);
    let board = loaded.session.board.as_ref().unwrap();
    assert_eq!(board.tick, 20);
    assert_eq!(board.balance, 200);
    assert_eq!(board.fish.len(), 2);
    let mut resumed = loaded.session;
    assert_eq!(resumed.step(&[]), session.step(&[]));
    assert_eq!(
        serde_json::to_value(resumed).unwrap(),
        serde_json::to_value(session).unwrap()
    );
    assert!(!path.with_extension("pending").exists());
    fs::remove_file(path).unwrap();
    fs::remove_dir(root).unwrap();
}

#[test]
fn legacy_board_save_migrates_without_losing_next_tick() {
    let mut board = AdventureState::new_adventure(42);
    for _ in 0..20 {
        board.tick();
    }
    let bytes = serde_json::to_vec(&cli::LegacyProjectSave {
        format_version: 1,
        state: board.clone(),
    })
    .unwrap();
    let mut session = cli::decode_save(&bytes).unwrap();
    assert_eq!(session.ticks, 20);
    assert_eq!(session.phase, AdventurePhase::Playing);
    assert_eq!(session.step(&[]), board.tick());
    assert_eq!(
        serde_json::to_value(session.board.unwrap()).unwrap(),
        serde_json::to_value(board).unwrap()
    );
}

#[test]
fn completed_legacy_board_becomes_one_pending_reward() {
    let mut board = AdventureState::new_adventure(42);
    board.victory = true;
    board.eggs = 3;
    board.tick = 250;
    let bytes = serde_json::to_vec(&cli::LegacyProjectSave {
        format_version: 1,
        state: board,
    })
    .unwrap();
    let session = cli::decode_save(&bytes).unwrap();
    assert_eq!((session.progress.tank, session.progress.level), (1, 2));
    assert_eq!(session.progress.unlocked_pets, vec![PetKind::Stinky]);
    assert!(session.board.is_none());
    assert_eq!(session.progress.first_stage_best_seconds, Some(7));
    assert_eq!(
        session.phase,
        AdventurePhase::Hatch {
            pet: PetKind::Stinky,
            updates: 0
        }
    );
    let resumed = cli::decode_save(
        &serde_json::to_vec(&cli::ProjectSave {
            format_version: cli::SAVE_FORMAT_VERSION,
            session,
        })
        .unwrap(),
    )
    .unwrap();
    assert_eq!(resumed.progress.unlocked_pets, vec![PetKind::Stinky]);
    assert!(resumed.board.is_none());
    assert_eq!(resumed.progress.first_stage_best_seconds, Some(7));
}

#[test]
fn old_v2_hatch_retains_unknown_score_and_level_two_gets_one_explicit_pet_migration() {
    let mut session = AdventureSession::new(42);
    let board = session.board.as_mut().unwrap();
    board.egg_unlocked = true;
    board.balance = 450;
    session.apply_actions(&[Action::BuyEgg, Action::BuyEgg, Action::BuyEgg]);
    let mut old_hatch = serde_json::to_value(cli::ProjectSave {
        format_version: 2,
        session: session.clone(),
    })
    .unwrap();
    old_hatch["session"]["progress"]
        .as_object_mut()
        .unwrap()
        .remove("first_stage_best_seconds");
    let mut resumed = cli::decode_save(&serde_json::to_vec(&old_hatch).unwrap()).unwrap();
    assert_eq!(resumed.progress.first_stage_best_seconds, None);
    for _ in 0..171 {
        resumed.step(&[]);
    }
    resumed.apply_actions(&[Action::Continue]);
    let mut old_board = serde_json::to_value(cli::ProjectSave {
        format_version: 2,
        session: resumed,
    })
    .unwrap();
    old_board["session"]["board"]
        .as_object_mut()
        .unwrap()
        .remove("stinky");
    let migrated = cli::decode_save(&serde_json::to_vec(&old_board).unwrap()).unwrap();
    assert_eq!(
        migrated
            .board
            .as_ref()
            .unwrap()
            .stinky
            .as_ref()
            .unwrap()
            .origin,
        StinkyOrigin::LegacyV2Resume
    );
    let mut round_trip = cli::decode_save(
        &serde_json::to_vec(&cli::ProjectSave {
            format_version: cli::SAVE_FORMAT_VERSION,
            session: migrated.clone(),
        })
        .unwrap(),
    )
    .unwrap();
    let mut uninterrupted = migrated;
    assert_eq!(round_trip.step(&[]), uninterrupted.step(&[]));
    assert_eq!(
        serde_json::to_value(round_trip).unwrap(),
        serde_json::to_value(uninterrupted).unwrap()
    );
    old_board["session"]["board"]["stinky"] = serde_json::Value::Null;
    assert!(cli::decode_save(&serde_json::to_vec(&old_board).unwrap()).is_err());
}

#[test]
fn modern_missing_state_and_contradictory_completed_legacy_board_are_rejected() {
    let mut completed = AdventureState::new_adventure(42);
    completed.victory = true;
    completed.eggs = 3;
    for invalid in [
        {
            let mut board = completed.clone();
            board.egg_price = 500;
            board
        },
        {
            let mut board = completed.clone();
            board.pets.push(PetKind::Stinky);
            board
        },
    ] {
        let bytes = serde_json::to_vec(&cli::LegacyProjectSave {
            format_version: 1,
            state: invalid,
        })
        .unwrap();
        assert!(cli::decode_save(&bytes).is_err());
    }
    let mut session = AdventureSession::from_legacy_board(completed).unwrap();
    for _ in 0..171 {
        session.step(&[]);
    }
    session.apply_actions(&[Action::Continue]);
    let modern = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session,
    })
    .unwrap();
    let mut missing_pet = modern.clone();
    missing_pet["session"]["board"]
        .as_object_mut()
        .unwrap()
        .remove("stinky");
    assert!(cli::decode_save(&serde_json::to_vec(&missing_pet).unwrap()).is_err());
    let mut missing_score = modern;
    missing_score["session"]["progress"]
        .as_object_mut()
        .unwrap()
        .remove("first_stage_best_seconds");
    assert!(cli::decode_save(&serde_json::to_vec(&missing_score).unwrap()).is_err());
}

#[cfg(windows)]
#[test]
fn denied_atomic_replace_preserves_previous_complete_snapshot() {
    use std::os::windows::fs::OpenOptionsExt;
    let root = temporary_root("read-lock");
    let path = root.join("state.local.json");
    cli::write_json(&path, &serde_json::json!({"tick":1})).unwrap();
    let reader = fs::OpenOptions::new()
        .read(true)
        .share_mode(1 | 2)
        .open(&path)
        .unwrap();
    assert!(cli::write_json(&path, &serde_json::json!({"tick":2})).is_err());
    let previous: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(previous["tick"], 1);
    drop(reader);
    cli::write_json(&path, &serde_json::json!({"tick":3})).unwrap();
    let recovered: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(recovered["tick"], 3);
    fs::remove_file(path).unwrap();
    fs::remove_dir(root).unwrap();
}
