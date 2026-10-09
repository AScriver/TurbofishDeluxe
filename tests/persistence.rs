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
    assert_eq!(loaded.format_version, cli::SAVE_FORMAT_VERSION);
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
fn format_six_migrates_fifth_board_without_losing_earned_state() {
    let roster = vec![PetKind::Stinky, PetKind::Itchy, PetKind::Prego];
    let mut session = AdventureSession::new(42);
    session.progress.level = 5;
    session.progress.unlocked_pets = vec![
        PetKind::Stinky,
        PetKind::Niko,
        PetKind::Itchy,
        PetKind::Prego,
    ];
    session.progress.selected_pets = roster.clone();
    session.board = Some(AdventureState::new_fifth_stage(42, &roster).unwrap());
    session.apply_actions(&[Action::Click { x: 300.0, y: 200.0 }]);
    let original = session.board.as_ref().unwrap().clone();
    let mut value = serde_json::to_value(cli::ProjectSave {
        format_version: 6,
        session,
    })
    .unwrap();
    value["session"]["progress"]
        .as_object_mut()
        .unwrap()
        .remove("shell_balance");
    let board = value["session"]["board"].as_object_mut().unwrap();
    for field in ["potion_unlocked", "potion_armed"] {
        board.remove(field);
    }
    for food in board["food"].as_array_mut().unwrap() {
        for field in [
            "direction",
            "vx",
            "vy",
            "animation_period",
            "free_from_zorf",
        ] {
            food.as_object_mut().unwrap().remove(field);
        }
    }
    let migrated = cli::decode_save(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert_eq!(migrated.progress.shell_balance, 0);
    assert_eq!(migrated.progress.selected_pets, roster);
    let board = migrated.board.as_ref().unwrap();
    assert_eq!(
        (board.tick, board.balance, board.eggs),
        (original.tick, original.balance, original.eggs)
    );
    assert_eq!(
        serde_json::to_value(board).unwrap()["rng_state"],
        serde_json::to_value(&original).unwrap()["rng_state"]
    );
    assert_eq!(board.food[0].x, original.food[0].x);
    assert_eq!(board.food[0].y, original.food[0].y);
    assert!(!board.potion_armed);
    assert_eq!(board.food[0].direction, 0);
    let mut unsupported = value;
    unsupported["session"]["progress"]["tank"] = 2.into();
    unsupported["session"]["progress"]["level"] = 1.into();
    assert!(cli::decode_save(&serde_json::to_vec(&unsupported).unwrap()).is_err());
}

#[test]
fn format_seven_requires_new_board_food_and_profile_fields() {
    let mut session = AdventureSession::new(42);
    session.apply_actions(&[Action::Click { x: 300.0, y: 200.0 }]);
    let modern = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session,
    })
    .unwrap();
    for field in ["potion_unlocked", "potion_armed"] {
        let mut incomplete = modern.clone();
        incomplete["session"]["board"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(cli::decode_save(&serde_json::to_vec(&incomplete).unwrap()).is_err());
    }
    for field in [
        "direction",
        "vx",
        "vy",
        "animation_period",
        "free_from_zorf",
    ] {
        let mut incomplete = modern.clone();
        incomplete["session"]["board"]["food"][0]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(cli::decode_save(&serde_json::to_vec(&incomplete).unwrap()).is_err());
    }
    let mut incomplete = modern.clone();
    incomplete["session"]["progress"]
        .as_object_mut()
        .unwrap()
        .remove("shell_balance");
    assert!(cli::decode_save(&serde_json::to_vec(&incomplete).unwrap()).is_err());
    assert!(cli::decode_save(&serde_json::to_vec(&modern).unwrap()).is_ok());
}

#[test]
fn bonus_results_reload_does_not_repeat_profile_credit() {
    use turbofish_deluxe::bonus::BonusResult;
    let mut session = AdventureSession::new(42);
    session.progress.tank = 2;
    session.progress.level = 1;
    session.progress.unlocked_pets = vec![
        PetKind::Stinky,
        PetKind::Niko,
        PetKind::Itchy,
        PetKind::Prego,
        PetKind::Zorf,
    ];
    session.progress.shell_balance = 317;
    session.board = None;
    session.phase = AdventurePhase::BonusResults {
        result: BonusResult {
            earned: 217,
            previous_balance: 100,
            updates: 8,
        },
    };
    let modern = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session,
    })
    .unwrap();
    let mut loaded = cli::decode_save(&serde_json::to_vec(&modern).unwrap()).unwrap();
    for _ in 0..40 {
        loaded.step(&[]);
    }
    assert_eq!(loaded.progress.shell_balance, 317);
    loaded.apply_actions(&[Action::Continue]);
    assert_eq!(
        loaded.phase,
        AdventurePhase::PetSelection {
            selected: Vec::new()
        }
    );
    loaded.validate().unwrap();
    let mut legacy = modern;
    legacy["format_version"] = 6.into();
    assert!(cli::decode_save(&serde_json::to_vec(&legacy).unwrap()).is_err());
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

fn second_stage_session() -> AdventureSession {
    let mut session = AdventureSession::new(42);
    let board = session.board.as_mut().unwrap();
    board.egg_unlocked = true;
    board.balance = 450;
    session.apply_actions(&[Action::BuyEgg, Action::BuyEgg, Action::BuyEgg]);
    for _ in 0..171 {
        session.step(&[]);
    }
    session.apply_actions(&[Action::Continue]);
    session
}

#[test]
fn format_three_migration_preserves_old_state_and_rewrites_before_play() {
    let root = temporary_root("v3-migration");
    let path = root.join("adventure.json");
    let mut session = second_stage_session();
    session.ticks = 4000;
    let board = session.board.as_mut().unwrap();
    board.tick = 1500;
    board.balance = 319;
    // The old implementation opened the egg gate at Large growth. Migration
    // keeps that evidence as the food-quality gate, without free upgrades.
    board.egg_unlocked = true;
    let expected_stinky = serde_json::to_value(&board.stinky).unwrap();
    let expected_rng = serde_json::to_value(&*board).unwrap()["rng_state"].clone();
    let first_best = session.progress.first_stage_best_seconds;
    let mut legacy = serde_json::to_value(cli::ProjectSave {
        format_version: 3,
        session,
    })
    .unwrap();
    legacy["session"]["progress"]
        .as_object_mut()
        .unwrap()
        .remove("later_stage_best_seconds");
    let old_board = legacy["session"]["board"].as_object_mut().unwrap();
    for field in ["upgrades", "invasion", "niko", "pearls"] {
        old_board.remove(field);
    }
    for fish in old_board["fish"].as_array_mut().unwrap() {
        fish.as_object_mut()
            .unwrap()
            .remove("cannot_be_eaten_ticks");
    }
    cli::write_json(&path, &legacy).unwrap();
    let options = cli::Options {
        game_dir: None,
        save_dir: root.clone(),
        evidence_dir: None,
        seed: 99,
        new_game: false,
        inspect_assets: false,
        quit_after: None,
        muted: true,
    };
    let migrated = cli::load_session(&options).unwrap();
    let board = migrated.board.as_ref().unwrap();
    assert_eq!(
        (migrated.ticks, board.tick, board.balance),
        (4000, 1500, 319)
    );
    assert_eq!(migrated.progress.first_stage_best_seconds, first_best);
    assert_eq!(
        serde_json::to_value(&board.stinky).unwrap(),
        expected_stinky
    );
    assert_eq!(
        serde_json::to_value(board).unwrap()["rng_state"],
        expected_rng
    );
    assert!(board.upgrades.quality_unlocked);
    assert_eq!((board.upgrades.quality, board.upgrades.quantity), (0, 1));
    assert!(!board.egg_unlocked);
    let wave = board.invasion.as_ref().unwrap();
    assert_eq!(
        wave.origin,
        turbofish_deluxe::invasion::InvasionOrigin::LegacyV3Resume
    );
    assert_eq!(wave.countdown, 1750);
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(saved["format_version"], cli::SAVE_FORMAT_VERSION);
    assert_eq!(
        serde_json::to_value(cli::decode_save(&fs::read(&path).unwrap()).unwrap()).unwrap(),
        serde_json::to_value(migrated).unwrap()
    );
    assert!(!path.with_extension("pending").exists());
    fs::remove_file(path).unwrap();
    fs::remove_dir(root).unwrap();
}

#[test]
fn modern_save_does_not_repair_missing_or_null_state() {
    let modern = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: second_stage_session(),
    })
    .unwrap();
    for field in [
        "stinky",
        "upgrades",
        "invasion",
        "niko",
        "pearls",
        "oscars",
        "dead_oscars",
        "oscar_unlocked",
        "weapon_strength",
        "weapon_unlocked",
    ] {
        let mut incomplete = modern.clone();
        incomplete["session"]["board"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            cli::decode_save(&serde_json::to_vec(&incomplete).unwrap()).is_err(),
            "{field}"
        );
    }
    let mut null_wave = modern.clone();
    null_wave["session"]["board"]["invasion"] = serde_json::Value::Null;
    assert!(cli::decode_save(&serde_json::to_vec(&null_wave).unwrap()).is_err());
    let mut missing_fish_field = modern.clone();
    missing_fish_field["session"]["board"]["fish"][0]
        .as_object_mut()
        .unwrap()
        .remove("cannot_be_eaten_ticks");
    assert!(cli::decode_save(&serde_json::to_vec(&missing_fish_field).unwrap()).is_err());
    let mut missing_later_score = modern;
    missing_later_score["session"]["progress"]
        .as_object_mut()
        .unwrap()
        .remove("later_stage_best_seconds");
    assert!(cli::decode_save(&serde_json::to_vec(&missing_later_score).unwrap()).is_err());
}

fn third_stage_session() -> AdventureSession {
    let mut session = second_stage_session();
    let board = session.board.as_mut().unwrap();
    board.upgrades.quality_unlocked = true;
    board.balance = 1700;
    session.apply_actions(&[
        Action::BuyFoodQuality,
        Action::BuyEgg,
        Action::BuyEgg,
        Action::BuyEgg,
    ]);
    for _ in 0..171 {
        session.step(&[]);
    }
    session.apply_actions(&[Action::Continue]);
    session
}

#[test]
fn format_four_stage_three_migration_retains_state_and_remembered_growth_gate() {
    let mut session = third_stage_session();
    session.ticks = 900;
    let board = session.board.as_mut().unwrap();
    board.tick = 50;
    board.balance = 400;
    // Old format4 remembered Large growth after its fish/corpse disappeared.
    board.upgrades.quality_unlocked = true;
    board.upgrades.quantity_unlocked = true;
    let expected_rng = serde_json::to_value(&*board).unwrap()["rng_state"].clone();
    let expected_niko = serde_json::to_value(&board.niko).unwrap();
    let progress_before = session.progress.clone();
    let mut old = serde_json::to_value(cli::ProjectSave {
        format_version: 4,
        session,
    })
    .unwrap();
    let old_board = old["session"]["board"].as_object_mut().unwrap();
    old_board.insert("invasion".into(), serde_json::Value::Null);
    for field in [
        "oscars",
        "dead_oscars",
        "oscar_unlocked",
        "weapon_strength",
        "weapon_unlocked",
    ] {
        old_board.remove(field);
    }
    let migrated = cli::decode_save(&serde_json::to_vec(&old).unwrap()).unwrap();
    let board = migrated.board.as_ref().unwrap();
    assert_eq!(migrated.progress, progress_before);
    assert_eq!((migrated.ticks, board.tick, board.balance), (900, 50, 400));
    assert_eq!(
        serde_json::to_value(board).unwrap()["rng_state"],
        expected_rng
    );
    assert_eq!(serde_json::to_value(&board.niko).unwrap(), expected_niko);
    assert!(board.oscar_unlocked);
    assert!(!board.weapon_unlocked);
    assert_eq!(board.weapon_strength, 2);
    let wave = board.invasion.as_ref().unwrap();
    assert_eq!(wave.countdown, 3000);
    assert_eq!(wave.kind, turbofish_deluxe::alien::SylvesterKind::Strong);
    assert_eq!(
        wave.origin,
        turbofish_deluxe::invasion::InvasionOrigin::LegacyV4Resume
    );
    let roundtrip = cli::decode_save(
        &serde_json::to_vec(&cli::ProjectSave {
            format_version: cli::SAVE_FORMAT_VERSION,
            session: migrated.clone(),
        })
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(roundtrip).unwrap(),
        serde_json::to_value(migrated).unwrap()
    );
}

#[test]
fn modern_requires_variant_and_rejects_weak_actor_on_strong_stage() {
    let mut session = third_stage_session();
    let wave = session.board.as_mut().unwrap().invasion.as_mut().unwrap();
    wave.alien = Some(turbofish_deluxe::alien::WeakSylvester::spawn_kind(
        turbofish_deluxe::alien::SylvesterKind::Strong,
        99,
        100,
        120,
        1,
        1,
    ));
    let modern = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session,
    })
    .unwrap();
    assert!(cli::decode_save(&serde_json::to_vec(&modern).unwrap()).is_ok());
    let mut missing_wave_kind = modern.clone();
    missing_wave_kind["session"]["board"]["invasion"]
        .as_object_mut()
        .unwrap()
        .remove("kind");
    assert!(cli::decode_save(&serde_json::to_vec(&missing_wave_kind).unwrap()).is_err());
    let mut missing_actor_kind = modern.clone();
    missing_actor_kind["session"]["board"]["invasion"]["alien"]
        .as_object_mut()
        .unwrap()
        .remove("kind");
    assert!(cli::decode_save(&serde_json::to_vec(&missing_actor_kind).unwrap()).is_err());
    let mut wrong_actor_kind = modern;
    wrong_actor_kind["session"]["board"]["invasion"]["alien"]["kind"] = serde_json::json!("Weak");
    assert!(cli::decode_save(&serde_json::to_vec(&wrong_actor_kind).unwrap()).is_err());
}

#[test]
fn format_six_requires_pet_fields_and_preserves_resumable_selection() {
    let mut session = AdventureSession::new(42);
    session.progress.level = 5;
    session.progress.unlocked_pets = vec![
        PetKind::Stinky,
        PetKind::Niko,
        PetKind::Itchy,
        PetKind::Prego,
    ];
    session.board = None;
    session.phase = AdventurePhase::PetSelection {
        selected: vec![PetKind::Niko, PetKind::Prego],
    };
    session.apply_actions(&[Action::Continue]);
    let modern = serde_json::to_value(cli::ProjectSave {
        format_version: 6,
        session,
    })
    .unwrap();
    let loaded = cli::decode_save(&serde_json::to_vec(&modern).unwrap()).unwrap();
    assert_eq!(
        loaded.phase,
        AdventurePhase::PetSelectionConfirmation {
            selected: vec![PetKind::Niko, PetKind::Prego]
        }
    );
    assert_eq!(
        loaded.progress.selected_pets,
        vec![PetKind::Niko, PetKind::Prego]
    );
    assert_eq!(loaded.ticks, 0);
    for field in ["pet_capacity", "selected_pets"] {
        let mut incomplete = modern.clone();
        incomplete["session"]["progress"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(cli::decode_save(&serde_json::to_vec(&incomplete).unwrap()).is_err());
    }
    let mut rejected = modern;
    rejected["session"]["phase"]["PetSelectionConfirmation"]["selected"] = serde_json::json!([]);
    assert!(cli::decode_save(&serde_json::to_vec(&rejected).unwrap()).is_err());
    let board_session = second_stage_session();
    let modern_board = serde_json::to_value(cli::ProjectSave {
        format_version: 6,
        session: board_session,
    })
    .unwrap();
    for field in ["fish_pets", "punch_sound_cooldown"] {
        let mut incomplete = modern_board.clone();
        incomplete["session"]["board"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(cli::decode_save(&serde_json::to_vec(&incomplete).unwrap()).is_err());
    }
}

#[test]
fn format_five_fourth_board_gets_explicit_new_support_without_rewriting_earned_state() {
    let mut session = AdventureSession::new(42);
    session.progress.level = 4;
    session.progress.unlocked_pets = vec![PetKind::Stinky, PetKind::Niko, PetKind::Itchy];
    session.board = Some(AdventureState::new_fourth_stage(42));
    session.ticks = 801;
    let board = session.board.as_mut().unwrap();
    board.tick = 700;
    board.balance = 1234;
    board.invasion = None;
    board.fish_pets.clear();
    let old_niko = serde_json::to_value(&board.niko).unwrap();
    let old_fish_ids = board.fish.iter().map(|fish| fish.id).collect::<Vec<_>>();
    let mut legacy = serde_json::to_value(cli::ProjectSave {
        format_version: 5,
        session,
    })
    .unwrap();
    let old_progress = legacy["session"]["progress"].as_object_mut().unwrap();
    old_progress.remove("pet_capacity");
    old_progress.remove("selected_pets");
    let old_board = legacy["session"]["board"].as_object_mut().unwrap();
    old_board.remove("fish_pets");
    old_board.remove("punch_sound_cooldown");
    let root = temporary_root("v5-fourth");
    fs::write(
        root.join("adventure.json"),
        serde_json::to_vec_pretty(&legacy).unwrap(),
    )
    .unwrap();
    let options = cli::Options {
        game_dir: None,
        save_dir: root.clone(),
        evidence_dir: None,
        seed: 42,
        new_game: false,
        inspect_assets: false,
        quit_after: None,
        muted: true,
    };
    let migrated = cli::load_session(&options).unwrap();
    let board = migrated.board.as_ref().unwrap();
    assert_eq!(
        (migrated.ticks, board.tick, board.balance),
        (801, 700, 1234)
    );
    assert_eq!(serde_json::to_value(&board.niko).unwrap(), old_niko);
    assert_eq!(
        board.fish.iter().map(|fish| fish.id).collect::<Vec<_>>(),
        old_fish_ids
    );
    assert_eq!(board.fish_pets.len(), 1);
    assert_eq!(
        board.fish_pets[0].kind,
        turbofish_deluxe::fish_pet::FishPetKind::Itchy
    );
    let wave = board.invasion.as_ref().unwrap();
    assert_eq!(wave.kind, turbofish_deluxe::alien::SylvesterKind::Balrog);
    assert_eq!(
        wave.origin,
        turbofish_deluxe::invasion::InvasionOrigin::LegacyV5Resume
    );
    assert_eq!(wave.countdown, 3000);
    let saved: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("adventure.json")).unwrap()).unwrap();
    assert_eq!(saved["format_version"], cli::SAVE_FORMAT_VERSION);
    let loaded = cli::load_session(&options).unwrap();
    assert_eq!(
        serde_json::to_value(loaded).unwrap(),
        serde_json::to_value(migrated).unwrap()
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn format_five_strong_corpse_kind_is_recovered_but_modern_missing_kind_is_rejected() {
    use turbofish_deluxe::{alien::SylvesterKind, invasion::DeadAlienEffect};
    let mut session = third_stage_session();
    session
        .board
        .as_mut()
        .unwrap()
        .invasion
        .as_mut()
        .unwrap()
        .dead_alien = Some(DeadAlienEffect {
        kind: SylvesterKind::Strong,
        x: 100.5,
        y: 200.5,
        widget_x: 100,
        widget_y: 200,
        vx: 1.0,
        vy: 0.0,
        frame: 0,
        facing_right: true,
        opacity: 1.0,
        remaining_ticks: 125,
    });
    let mut legacy = serde_json::to_value(cli::ProjectSave {
        format_version: 5,
        session,
    })
    .unwrap();
    legacy["session"]["board"]["invasion"]["dead_alien"]
        .as_object_mut()
        .unwrap()
        .remove("kind");
    let migrated = cli::decode_save(&serde_json::to_vec(&legacy).unwrap()).unwrap();
    assert_eq!(
        migrated
            .board
            .as_ref()
            .unwrap()
            .invasion
            .as_ref()
            .unwrap()
            .dead_alien
            .as_ref()
            .unwrap()
            .kind,
        SylvesterKind::Strong
    );
    let mut modern = serde_json::to_value(cli::ProjectSave {
        format_version: 6,
        session: migrated,
    })
    .unwrap();
    modern["session"]["board"]["invasion"]["dead_alien"]
        .as_object_mut()
        .unwrap()
        .remove("kind");
    assert!(cli::decode_save(&serde_json::to_vec(&modern).unwrap()).is_err());
}

#[test]
fn saved_invasion_modal_preserves_pause_and_does_not_repeat_warning() {
    let mut session = second_stage_session();
    session
        .board
        .as_mut()
        .unwrap()
        .invasion
        .as_mut()
        .unwrap()
        .countdown = 277;
    session.step(&[]);
    let mut restored = cli::decode_save(
        &serde_json::to_vec(&cli::ProjectSave {
            format_version: cli::SAVE_FORMAT_VERSION,
            session: session.clone(),
        })
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        restored.phase,
        AdventurePhase::InvasionTutorial {
            tip: turbofish_deluxe::invasion::InvasionTip::Danger,
        }
    );
    for actions in [&[][..], &[Action::Continue][..], &[][..]] {
        assert_eq!(restored.step(actions), session.step(actions));
    }
    assert_eq!(
        serde_json::to_value(restored).unwrap(),
        serde_json::to_value(session).unwrap()
    );
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
