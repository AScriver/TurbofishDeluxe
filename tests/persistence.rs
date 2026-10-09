use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use turbofish_deluxe::{
    adventure::{AdventurePhase, AdventureSession, PetKind},
    cli,
    sim::AdventureState,
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
            format_version: 2,
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
            format_version: 2,
            session: session.clone(),
        },
    )
    .unwrap();
    let loaded: cli::ProjectSave = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(loaded.format_version, 2);
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
    let bytes = serde_json::to_vec(&cli::LegacyProjectSave {
        format_version: 1,
        state: board,
    })
    .unwrap();
    let session = cli::decode_save(&bytes).unwrap();
    assert_eq!((session.progress.tank, session.progress.level), (1, 2));
    assert_eq!(session.progress.unlocked_pets, vec![PetKind::Stinky]);
    assert!(session.board.is_none());
    assert_eq!(
        session.phase,
        AdventurePhase::Hatch {
            pet: PetKind::Stinky,
            updates: 0
        }
    );
    let resumed = cli::decode_save(
        &serde_json::to_vec(&cli::ProjectSave {
            format_version: 2,
            session,
        })
        .unwrap(),
    )
    .unwrap();
    assert_eq!(resumed.progress.unlocked_pets, vec![PetKind::Stinky]);
    assert!(resumed.board.is_none());
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
