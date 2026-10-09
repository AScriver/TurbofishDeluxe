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

fn vert_session() -> AdventureSession {
    let mut session = AdventureSession::new(42);
    session.progress.tank = 2;
    session.progress.level = 3;
    session.progress.unlocked_pets = vec![
        PetKind::Stinky,
        PetKind::Niko,
        PetKind::Itchy,
        PetKind::Prego,
        PetKind::Zorf,
        PetKind::Clyde,
        PetKind::Vert,
    ];
    session.progress.selected_pets = vec![PetKind::Niko, PetKind::Itchy, PetKind::Vert];
    session.board =
        Some(AdventureState::new_tank2_third_stage(42, &session.progress.selected_pets).unwrap());
    session
}

fn rufus_session() -> AdventureSession {
    let mut session = vert_session();
    session.progress.level = 4;
    session.progress.unlocked_pets.push(PetKind::Rufus);
    session.progress.selected_pets = vec![PetKind::Niko, PetKind::Itchy, PetKind::Rufus];
    session.board =
        Some(AdventureState::new_tank2_fourth_stage(42, &session.progress.selected_pets).unwrap());
    session
}

fn pending_destructor_session() -> (AdventureSession, u64, u64) {
    use turbofish_deluxe::{
        alien::{SylvesterKind, WeakSylvester},
        missile::ClassicMissile,
    };
    // Synthetic current-format fixture: no progression or live earning claim.
    // Allocate fixture entities through serialized next_id, keeping the live
    // Board allocator private and retaining the target's actual identity.
    let mut fixture = serde_json::to_value(rufus_session()).unwrap();
    let alien_id = fixture["board"]["next_id"].as_u64().unwrap();
    let missile_id = alien_id + 1;
    fixture["board"]["next_id"] = (missile_id + 1).into();
    let mut session: AdventureSession = serde_json::from_value(fixture).unwrap();
    let board = session.board.as_mut().unwrap();
    let mut actor = WeakSylvester::spawn_kind(SylvesterKind::Destructor, alien_id, 100, 280, 1, 1);
    actor.spawn_ticks = 0;
    actor.health = 0.25;
    assert_eq!(actor.itchy_hit(), Some(0.0));
    assert_eq!(actor.rufus_hit(), Some(-0.25));
    board.invasion.as_mut().unwrap().alien = Some(actor);
    board.missiles.push(ClassicMissile::launch(
        missile_id,
        board.fish[0].id,
        10,
        95,
        3,
    ));
    let rufus = board.rufus.as_mut().unwrap();
    rufus.chase_ticks = 4;
    rufus.animation_ticks = 12;
    rufus.frame = 3;
    session.validate().unwrap();
    (session, alien_id, missile_id)
}

#[test]
fn current_destructor_pending_death_live_missile_and_rufus_reload_exactly() {
    use turbofish_deluxe::{invasion::InvasionEvent, sim::Event};
    let (mut uninterrupted, alien_id, missile_id) = pending_destructor_session();
    let bytes = serde_json::to_vec(&cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: uninterrupted.clone(),
    })
    .unwrap();
    let mut resumed = cli::decode_save(&bytes).unwrap();
    assert_eq!(
        serde_json::to_value(&resumed).unwrap(),
        serde_json::to_value(&uninterrupted).unwrap()
    );
    for tick in 0..32 {
        let expected_events = uninterrupted.step(&[]);
        let actual_events = resumed.step(&[]);
        assert_eq!(
            serde_json::to_value(&actual_events).unwrap(),
            serde_json::to_value(&expected_events).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&resumed).unwrap(),
            serde_json::to_value(&uninterrupted).unwrap()
        );
        if tick == 0 {
            assert_eq!(actual_events.iter().filter(|event| matches!(event,
                Event::Invasion { event: InvasionEvent::AlienDefeated { id, .. }, .. } if *id == alien_id
            )).count(), 1);
            assert_eq!(
                actual_events
                    .iter()
                    .filter(|event| matches!(event,
                        Event::AlienDiamondDropped { alien_id: id, .. } if *id == alien_id
                    ))
                    .count(),
                1
            );
            let board = resumed.board.as_ref().unwrap();
            assert!(board.invasion.as_ref().unwrap().dead_alien.is_some());
            assert!(
                board
                    .missiles
                    .iter()
                    .any(|missile| missile.id == missile_id)
            );
            assert!(
                !actual_events.iter().any(|event| matches!(
                    event,
                    Event::Invasion {
                        event: InvasionEvent::BattleEnded,
                        ..
                    }
                )),
                "the registered missile keeps this battle open after alien death"
            );
        }
        resumed.validate().unwrap();
    }
}

#[test]
fn current_missile_and_rufus_fields_are_required_without_repair() {
    let value = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: rufus_session(),
    })
    .unwrap();
    assert!(cli::decode_save(&serde_json::to_vec(&value).unwrap()).is_ok());
    for field in ["missiles", "rufus"] {
        let mut incomplete = value.clone();
        incomplete["session"]["board"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            cli::decode_save(&serde_json::to_vec(&incomplete).unwrap()).is_err(),
            "missing {field}"
        );
    }
}

#[test]
fn current_save_rejects_malformed_missile_clock_and_destructor_health() {
    let (session, _, _) = pending_destructor_session();
    let value = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session,
    })
    .unwrap();
    for (pointer, invalid) in [
        (
            "/session/board/missiles/0/immunity_ticks",
            serde_json::json!(16),
        ),
        (
            "/session/board/missiles/0/target_id",
            serde_json::json!(u64::MAX),
        ),
        ("/session/board/rufus/vy", serde_json::json!(1.0)),
        (
            "/session/board/invasion/alien/health",
            serde_json::json!(-0.1),
        ),
    ] {
        let mut malformed = value.clone();
        *malformed.pointer_mut(pointer).unwrap() = invalid;
        assert!(
            cli::decode_save(&serde_json::to_vec(&malformed).unwrap()).is_err(),
            "invalid {pointer}"
        );
    }
}

#[test]
fn current_gus_quarter_pending_death_and_vert_clock_reload_exactly() {
    use turbofish_deluxe::{
        alien::{SylvesterKind, WeakSylvester},
        fish_pet::FishPetKind,
        invasion::InvasionEvent,
        sim::Event,
    };
    let mut uninterrupted = vert_session();
    // Reserve a synthetic current entity through its serialized fixture;
    // the Board's live identity allocator remains private.
    let mut fixture = serde_json::to_value(&uninterrupted).unwrap();
    let alien_id = fixture["board"]["next_id"].as_u64().unwrap();
    fixture["board"]["next_id"] = (alien_id + 1).into();
    uninterrupted = serde_json::from_value(fixture).unwrap();
    let board = uninterrupted.board.as_mut().unwrap();
    let mut actor = WeakSylvester::spawn_kind(SylvesterKind::Gus, alien_id, 100, 120, 1, 1);
    actor.spawn_ticks = 0;
    actor.health = 0.25;
    assert_eq!(actor.itchy_hit(), Some(0.0));
    assert_eq!(actor.itchy_hit(), Some(-0.25));
    board.invasion.as_mut().unwrap().alien = Some(actor);
    board
        .fish_pets
        .iter_mut()
        .find(|pet| pet.kind == FishPetKind::Vert)
        .unwrap()
        .coin_timer = 215;
    uninterrupted.validate().unwrap();
    let saved = serde_json::to_vec(&cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: uninterrupted.clone(),
    })
    .unwrap();
    let mut resumed = cli::decode_save(&saved).unwrap();
    assert_eq!(
        resumed
            .board
            .as_ref()
            .unwrap()
            .invasion
            .as_ref()
            .unwrap()
            .alien
            .as_ref()
            .unwrap()
            .health,
        -0.25
    );
    for tick in 0..32 {
        let expected_events = uninterrupted.step(&[]);
        let actual_events = resumed.step(&[]);
        assert_eq!(
            serde_json::to_value(&actual_events).unwrap(),
            serde_json::to_value(&expected_events).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&resumed).unwrap(),
            serde_json::to_value(&uninterrupted).unwrap()
        );
        if tick == 0 {
            assert_eq!(actual_events.iter().filter(|event| matches!(event,
                Event::Invasion { event: InvasionEvent::AlienDefeated { id, .. }, .. } if *id == alien_id
            )).count(), 1);
            assert_eq!(
                actual_events
                    .iter()
                    .filter(|event| matches!(event, Event::VertGoldDropped { .. }))
                    .count(),
                1
            );
            assert!(
                resumed
                    .board
                    .as_ref()
                    .unwrap()
                    .invasion
                    .as_ref()
                    .unwrap()
                    .dead_alien
                    .is_none()
            );
        }
        resumed.validate().unwrap();
    }
}

#[test]
fn current_gus_warning_and_vert_clock_fields_are_required_without_repair() {
    let session = vert_session();
    let value = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session,
    })
    .unwrap();
    for pointer in ["/session/board/fish_pets/0", "/session/board/invasion"] {
        let field = if pointer.ends_with("/0") {
            "coin_timer"
        } else {
            "gus_warning_shown"
        };
        let mut incomplete = value.clone();
        incomplete
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(cli::decode_save(&serde_json::to_vec(&incomplete).unwrap()).is_err());
    }
    let mut invalid_clock = value;
    invalid_clock["session"]["board"]["fish_pets"][1]["coin_timer"] = 216.into();
    assert!(cli::decode_save(&serde_json::to_vec(&invalid_clock).unwrap()).is_err());
}

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
fn current_format_requires_board_food_and_profile_fields() {
    let mut session = AdventureSession::new(42);
    session.apply_actions(&[Action::Click { x: 300.0, y: 200.0 }]);
    let modern = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session,
    })
    .unwrap();
    for field in [
        "potion_unlocked",
        "potion_armed",
        "starcatcher_unlocked",
        "starcatchers",
        "dead_starcatchers",
        "clyde",
    ] {
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
fn current_actor_corpse_and_diamond_phase_reload_preserves_every_next_update() {
    use turbofish_deluxe::sim::{Coin, CoinKind};
    let mut session = AdventureSession::new(42);
    session.progress.tank = 2;
    session.progress.level = 2;
    session.progress.unlocked_pets = vec![
        PetKind::Stinky,
        PetKind::Niko,
        PetKind::Itchy,
        PetKind::Prego,
        PetKind::Zorf,
        PetKind::Clyde,
    ];
    session.progress.selected_pets = vec![PetKind::Niko, PetKind::Clyde];
    session.board =
        Some(AdventureState::new_tank2_second_stage(42, &session.progress.selected_pets).unwrap());
    let board = session.board.as_mut().unwrap();
    board.balance = 2000;
    board.upgrades.quality_unlocked = true;
    board.upgrades.quantity_unlocked = true;
    board.potion_unlocked = true;
    board.starcatcher_unlocked = true;
    session.apply_actions(&[Action::BuyStarcatcher, Action::BuyStarcatcher]);
    session.board.as_mut().unwrap().starcatchers[0].hunger = 1;
    session.step(&[]);
    let board = session.board.as_ref().unwrap();
    assert_eq!(
        (board.starcatchers.len(), board.dead_starcatchers.len()),
        (1, 1)
    );
    session.validate().unwrap();
    let mut value = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session,
    })
    .unwrap();
    let coin_id = value["session"]["board"]["next_id"].as_u64().unwrap();
    value["session"]["board"]["next_id"] = (coin_id + 1).into();
    value["session"]["board"]["coins"] = serde_json::to_value(vec![Coin {
        id: coin_id,
        x: 300.0,
        y: 119.5,
        kind: CoinKind::DiamondPenta,
        frame: 0,
        collecting: false,
        bottom_ticks: 0,
        fade_ticks: 0,
        penta_rising: true,
    }])
    .unwrap();
    let bytes = serde_json::to_vec(&value).unwrap();
    let mut uninterrupted = cli::decode_save(&bytes).unwrap();
    let snapshot = serde_json::to_vec(&cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: uninterrupted.clone(),
    })
    .unwrap();
    let mut resumed = cli::decode_save(&snapshot).unwrap();
    for _ in 0..32 {
        assert_eq!(uninterrupted.step(&[]), resumed.step(&[]));
        assert_eq!(
            serde_json::to_value(&uninterrupted).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
    }
    assert!(
        !resumed
            .board
            .as_ref()
            .unwrap()
            .coins
            .iter()
            .find(|coin| coin.id == coin_id)
            .unwrap()
            .penta_rising
    );
    let mut missing_phase = value;
    missing_phase["session"]["board"]["coins"][0]
        .as_object_mut()
        .unwrap()
        .remove("penta_rising");
    assert!(cli::decode_save(&serde_json::to_vec(&missing_phase).unwrap()).is_err());
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
    let mut modern = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session,
    })
    .unwrap();
    // Reserve the synthetic actor's ID in this current-format fixture.
    modern["session"]["board"]["next_id"] = 100.into();
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
