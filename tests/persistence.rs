use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use turbofish_deluxe::{
    adventure::{AdventurePhase, AdventureSession, PetKind},
    bonus::PurchaseReceipt,
    cli,
    sim::{Action, AdventureState, StinkyOrigin},
};

fn historical_envelope(save: cli::ProjectSave) -> serde_json::Value {
    assert!(save.format_version < 26);
    let mut value = serde_json::to_value(save).unwrap();
    let session = value["session"].as_object_mut().unwrap();
    session.remove("mode");
    session.remove("time_trial");
    session.remove("time_trial_scores");
    if let Some(board) = session
        .get_mut("board")
        .and_then(serde_json::Value::as_object_mut)
    {
        board.remove("time_trial");
    }
    value
}

#[test]
fn current_time_trial_envelope_and_actor_form_fields_are_explicit() {
    let session = vert_session();
    let encoded = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session,
    })
    .unwrap();
    for field in ["mode", "time_trial", "time_trial_scores"] {
        let mut missing = encoded.clone();
        missing["session"].as_object_mut().unwrap().remove(field);
        assert!(
            cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err(),
            "{field}"
        );
    }
    let mut missing_board_mode = encoded.clone();
    missing_board_mode["session"]["board"]
        .as_object_mut()
        .unwrap()
        .remove("time_trial");
    assert!(cli::decode_save(&serde_json::to_vec(&missing_board_mode).unwrap()).is_err());
    for field in ["clyde", "niko", "rufus", "rhubarb"] {
        let session = match field {
            "clyde" => tank_three_session(&[PetKind::Clyde]),
            "niko" => third_stage_session(),
            "rufus" => rufus_session(),
            _ => tank_four_first_session(&[PetKind::Rhubarb]),
        };
        let mut value = serde_json::to_value(cli::ProjectSave {
            format_version: cli::SAVE_FORMAT_VERSION,
            session,
        })
        .unwrap();
        let members = value["session"]["board"][field].as_array_mut().unwrap();
        assert!(!members.is_empty(), "{field}");
        members[0].as_object_mut().unwrap().remove("presto_form");
        assert!(
            cli::decode_save(&serde_json::to_vec(&value).unwrap()).is_err(),
            "{field}"
        );
    }
}

#[test]
fn current_time_trial_purchase_and_board_continuation_survive_decode() {
    let mut session = AdventureSession::new(81);
    session.progress.tank = 2;
    session.progress.level = 1;
    session.progress.unlocked_pets = vec![
        PetKind::Stinky,
        PetKind::Niko,
        PetKind::Itchy,
        PetKind::Prego,
        PetKind::Zorf,
    ];
    session.board = None;
    session.phase = AdventurePhase::GameSelector;
    session.validate().unwrap();
    session.apply_actions(&[
        Action::PlayTimeTrial,
        Action::SelectTimeTrialTank { tank: 3 },
        Action::TogglePet {
            pet: PetKind::Stinky,
        },
        Action::Continue,
    ]);
    assert_eq!(session.phase, AdventurePhase::TimeTrialPlaying);
    session.apply_actions(&[Action::BuyEgg]);
    session.validate().unwrap();
    let mut decoded = cli::decode_save(
        &serde_json::to_vec(&cli::ProjectSave {
            format_version: cli::SAVE_FORMAT_VERSION,
            session: session.clone(),
        })
        .unwrap(),
    )
    .unwrap();
    for _ in 0..32 {
        assert_eq!(
            serde_json::to_value(session.step(&[])).unwrap(),
            serde_json::to_value(decoded.step(&[])).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&session).unwrap(),
            serde_json::to_value(&decoded).unwrap()
        );
    }
}

#[test]
fn current_time_trial_rejects_acquired_presto_claim_even_with_a_valid_physical_roster() {
    let mut session = prepared_tank_five_session(0);
    session.progress.tank = 5;
    session.progress.level = 2;
    session.progress.adventure_completed = true;
    session.progress.adventure_completions = 1;
    session.progress.unlocked_pets.push(PetKind::Presto);
    session.board = None;
    session.phase = AdventurePhase::GameSelector;
    session.apply_actions(&[
        Action::PlayTimeTrial,
        Action::SelectTimeTrialTank { tank: 1 },
        Action::TogglePet {
            pet: PetKind::Stinky,
        },
        Action::Continue,
    ]);
    session.validate().unwrap();

    session.board =
        Some(AdventureState::new_time_trial(42, 1, &[PetKind::Stinky, PetKind::Presto]).unwrap());
    let board = session.board.as_mut().unwrap();
    board.egg_price = 200;
    board.validate().unwrap();
    let run = session.time_trial.as_mut().unwrap();
    run.acquired_pets.push(PetKind::Presto);
    run.egg_purchases = 1;
    run.last_purchase_candidates = Some(1);
    run.egg_maxed = true;

    assert!(session.validate().is_err());
    assert!(cli::decode_save(&current_bytes(session)).is_err());
}

#[test]
fn current_tank_five_other_pets_have_canonical_arrays_and_reject_duplicate_roster() {
    let session = prepared_tank_five_session(0);
    let encoded = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: session.clone(),
    })
    .unwrap();
    for field in ["stinky", "niko", "clyde", "rufus", "rhubarb"] {
        let members = encoded["session"]["board"][field].as_array().unwrap();
        assert_eq!(members.len(), 1, "{field}");
        let mut duplicate = encoded.clone();
        duplicate["session"]["board"][field]
            .as_array_mut()
            .unwrap()
            .push(members[0].clone());
        assert!(
            cli::decode_save(&serde_json::to_vec(&duplicate).unwrap()).is_err(),
            "{field}"
        );

        let mut old_shape = encoded.clone();
        old_shape["session"]["board"][field] = members[0].clone();
        assert!(
            cli::decode_save(&serde_json::to_vec(&old_shape).unwrap()).is_err(),
            "{field}"
        );
        old_shape["format_version"] = 22.into();
        assert!(
            cli::decode_save(&serde_json::to_vec(&old_shape).unwrap())
                .unwrap_err()
                .to_string()
                .contains("Option-shaped OtherPet saves"),
            "{field}"
        );
    }

    let mut uninterrupted = session;
    let mut reopened = cli::decode_save(&serde_json::to_vec(&encoded).unwrap()).unwrap();
    for _ in 0..16 {
        assert_eq!(
            serde_json::to_value(uninterrupted.step(&[])).unwrap(),
            serde_json::to_value(reopened.step(&[])).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&uninterrupted).unwrap(),
            serde_json::to_value(&reopened).unwrap()
        );
    }
}

#[test]
fn current_twenty_three_requires_explicit_plain_presto_form_state() {
    let session = vert_session();
    let encoded = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: session.clone(),
    })
    .unwrap();
    let pets = encoded
        .pointer("/session/board/fish_pets")
        .unwrap()
        .as_array()
        .unwrap();
    assert!(!pets.is_empty());
    assert!(pets.iter().all(|pet| pet["presto_form"].is_null()));
    for index in 0..pets.len() {
        let mut missing = encoded.clone();
        missing["session"]["board"]["fish_pets"][index]
            .as_object_mut()
            .unwrap()
            .remove("presto_form");
        assert!(cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err());
    }
    assert_eq!(
        serde_json::to_vec(&cli::decode_save(&serde_json::to_vec(&encoded).unwrap()).unwrap())
            .unwrap(),
        serde_json::to_vec(&session).unwrap()
    );
}

#[test]
fn current_plain_roster_rejects_unowned_presto_form() {
    let mut encoded = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: vert_session(),
    })
    .unwrap();
    encoded["session"]["board"]["fish_pets"][0]["presto_form"] =
        serde_json::json!({"remaining_ticks":0});
    assert!(cli::decode_save(&serde_json::to_vec(&encoded).unwrap()).is_err());
}

#[test]
fn current_twenty_five_requires_stinky_form_field_without_rewriting_old_saves() {
    let encoded = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: second_stage_session(),
    })
    .unwrap();
    assert!(encoded["session"]["board"]["stinky"][0]["presto_form"].is_null());
    let mut missing = encoded.clone();
    missing["session"]["board"]["stinky"][0]
        .as_object_mut()
        .unwrap()
        .remove("presto_form");
    assert!(cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err());
    assert!(cli::decode_save(&serde_json::to_vec(&encoded).unwrap()).is_ok());
}

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
    board.invasion.as_mut().unwrap().actors = vec![actor];
    board.invasion.as_mut().unwrap().battle_active = true;
    board.missiles.push(ClassicMissile::launch(
        missile_id,
        board.fish[0].id,
        10,
        95,
        3,
    ));
    let rufus = board.rufus.first_mut().unwrap();
    rufus.chase_ticks = 4;
    rufus.animation_ticks = 12;
    rufus.frame = 3;
    session.validate().unwrap();
    (session, alien_id, missile_id)
}

fn meryl_session() -> AdventureSession {
    let mut session = rufus_session();
    session.progress.level = 5;
    session.progress.unlocked_pets.push(PetKind::Meryl);
    session.progress.selected_pets = vec![PetKind::Niko, PetKind::Itchy, PetKind::Meryl];
    session.board =
        Some(AdventureState::new_tank2_fifth_stage(42, &session.progress.selected_pets).unwrap());
    session
}

fn tank_three_session(pets: &[PetKind]) -> AdventureSession {
    let mut session = meryl_session();
    session.progress.tank = 3;
    session.progress.level = 1;
    session.progress.unlocked_pets.push(PetKind::Wadsworth);
    session.progress.selected_pets = pets.to_vec();
    session.board = Some(AdventureState::new_tank3_first_stage(42, pets).unwrap());
    session
}

fn tank_three_second_session(pets: &[PetKind]) -> AdventureSession {
    // The previous-stage constructor correctly rejects Seymour; construct
    // only this stage with the requested roster.
    let mut session = tank_three_session(&[]);
    session.progress.level = 2;
    session.progress.unlocked_pets.push(PetKind::Seymour);
    session.progress.selected_pets = pets.to_vec();
    session.board = Some(AdventureState::new_tank3_second_stage(42, pets).unwrap());
    session
}

fn bought_gekko_session() -> AdventureSession {
    // Synthetic growth/cash boundary; purchases and actor allocation run the
    // production actions. This does not claim native earning or progression.
    let mut session = tank_three_second_session(&[PetKind::Seymour]);
    let board = session.board.as_mut().unwrap();
    board.balance = 2750;
    board.upgrades.quality_unlocked = true;
    board.upgrades.quantity_unlocked = true;
    board.grubber_unlocked = true;
    session.apply_actions(&[Action::BuyGrubber, Action::BuyGekko]);
    session.validate().unwrap();
    session
}

fn tank_three_third_session(pets: &[PetKind]) -> AdventureSession {
    let mut session = tank_three_second_session(&[]);
    session.progress.level = 3;
    session.progress.unlocked_pets.push(PetKind::Shrapnel);
    session.progress.selected_pets = pets.to_vec();
    session.board = Some(AdventureState::new_tank3_third_stage(42, pets).unwrap());
    session
}

fn tank_three_fourth_session(pets: &[PetKind]) -> AdventureSession {
    let mut session = tank_three_third_session(&[]);
    session.progress.level = 4;
    session.progress.unlocked_pets.push(PetKind::Gumbo);
    session.progress.selected_pets = pets.to_vec();
    session.board = Some(AdventureState::new_tank3_fourth_stage(42, pets).unwrap());
    session
}

fn ulysses_energyball_session(immunity_ticks: u8) -> AdventureSession {
    use turbofish_deluxe::{
        alien::{SylvesterKind, WeakSylvester},
        missile::ClassicMissile,
    };
    // Controlled current-format fixture, separate from earned progression.
    // Both IDs come from the existing serialized Board allocator boundary.
    let mut fixture = serde_json::to_value(tank_three_fourth_session(&[PetKind::Gumbo])).unwrap();
    let alien_id = fixture["board"]["next_id"].as_u64().unwrap();
    let missile_id = alien_id + 1;
    fixture["board"]["next_id"] = (missile_id + 1).into();
    let mut session: AdventureSession = serde_json::from_value(fixture).unwrap();
    let board = session.board.as_mut().unwrap();
    let mut actor = WeakSylvester::spawn_kind(SylvesterKind::Ulysses, alien_id, 460, 210, 1, 1);
    actor.spawn_ticks = 0;
    actor.launch_ticks = 59;
    let wave = board.invasion.as_mut().unwrap();
    wave.actors = vec![actor];
    wave.battle_active = true;
    wave.countdown = 3000;
    let mut missile = ClassicMissile::launch_energy(missile_id, board.fish[0].id, 100, 110, 3);
    missile.immunity_ticks = immunity_ticks;
    board.missiles.push(missile);
    session.validate().unwrap();
    session
}

fn tank_three_fifth_session(pets: &[PetKind]) -> AdventureSession {
    let mut session = tank_three_fourth_session(&[]);
    session.progress.level = 5;
    session.progress.unlocked_pets.push(PetKind::Blip);
    session.progress.selected_pets = pets.to_vec();
    session.board = Some(AdventureState::new_tank3_fifth_stage(42, pets).unwrap());
    session
}

fn tank_four_first_session(pets: &[PetKind]) -> AdventureSession {
    let mut session = tank_three_fifth_session(&[]);
    session.progress.tank = 4;
    session.progress.level = 1;
    session.progress.unlocked_pets.push(PetKind::Rhubarb);
    session.progress.selected_pets = pets.to_vec();
    session.board = Some(AdventureState::new_tank4_first_stage(42, pets).unwrap());
    session
}

fn tank_four_second_session(pets: &[PetKind]) -> AdventureSession {
    let mut session = tank_four_first_session(&[]);
    session.progress.level = 2;
    session.progress.unlocked_pets.push(PetKind::Nimbus);
    session.progress.selected_pets = pets.to_vec();
    session.board = Some(AdventureState::new_tank4_second_stage(42, pets).unwrap());
    session
}

fn tank_four_third_session(pets: &[PetKind]) -> AdventureSession {
    let mut session = tank_four_second_session(&[]);
    session.progress.level = 3;
    session.progress.unlocked_pets.push(PetKind::Amp);
    session.progress.selected_pets = pets.to_vec();
    session.board = Some(AdventureState::new_tank4_third_stage(42, pets).unwrap());
    session
}

fn tank_four_fourth_session(pets: &[PetKind]) -> AdventureSession {
    let mut session = tank_four_third_session(&[]);
    session.progress.level = 4;
    session.progress.unlocked_pets.push(PetKind::Gash);
    session.progress.selected_pets = pets.to_vec();
    session.board = Some(AdventureState::new_tank4_fourth_stage(42, pets).unwrap());
    session
}

fn tank_four_finale_session(pets: &[PetKind]) -> AdventureSession {
    let mut session = tank_four_fourth_session(&[]);
    session.progress.level = 5;
    session.progress.unlocked_pets.push(PetKind::Angie);
    session.progress.selected_pets = pets.to_vec();
    session.board = Some(AdventureState::new_tank4_finale(42, pets).unwrap());
    session
}

fn tank_four_finale_dead_guppy_session(pets: &[PetKind]) -> AdventureSession {
    use turbofish_deluxe::breeder::BreederSize;

    // Ordinary Board birth and starvation create a real guppy corpse;
    // this controlled fixture is separate from native earned play.
    let mut session = tank_four_finale_session(pets);
    let board = session.board.as_mut().unwrap();
    let breeder = &mut board.breeders[0];
    breeder.size = BreederSize::Medium;
    breeder.food_points = 0;
    breeder.food_needed_to_grow = 7;
    breeder.birth_clock = 999;
    breeder.birth_threshold = 1000;
    breeder.hunger = 300;
    board.breeder_unlocked = true;
    session.step(&[]);
    assert_eq!(session.board.as_ref().unwrap().fish.len(), 1);
    let fish = &mut session.board.as_mut().unwrap().fish[0];
    fish.beginner = false;
    fish.hunger = 1;
    session.step(&[]);
    assert_eq!(session.board.as_ref().unwrap().dead_fish.len(), 1);
    session.validate().unwrap();
    session
}

fn tank_four_finale_supported_corpse_session(pets: &[PetKind]) -> AdventureSession {
    use turbofish_deluxe::{
        breeder::{BreederState, DeadBreeder},
        oscar::{DeadOscar, OscarState},
        ultra::{DeadUltra, UltraState},
    };

    // Tank 4 can own Oscar, Ultra, Breeder and ordinary guppy corpses.
    let mut session = tank_four_finale_dead_guppy_session(pets);

    let mut value = serde_json::to_value(&session).unwrap();
    let next_id = value["board"]["next_id"].as_u64().unwrap();
    value["board"]["next_id"] = (next_id + 3).into();
    session = serde_json::from_value(value).unwrap();
    let board = session.board.as_mut().unwrap();
    board.upgrades.quality_unlocked = true;
    board.upgrades.quantity_unlocked = true;
    board.oscar_unlocked = true;
    board.ultra_unlocked = true;
    board.weapon_unlocked = true;
    board.egg_unlocked = true;
    board
        .dead_oscars
        .push(DeadOscar::from_impact(&OscarState::spawn_bought(
            next_id,
            &mut |_| 1,
        )));
    board
        .dead_ultras
        .push(DeadUltra::from_impact(&UltraState::spawn_bought(
            next_id + 1,
            &mut |_| 1,
        )));
    board
        .dead_breeders
        .push(DeadBreeder::from_impact(&BreederState::spawn_bought(
            next_id + 2,
            &mut |_| 1,
        )));
    session.validate().unwrap();
    session
}

fn tank_two_starcatcher_corpse_session() -> AdventureSession {
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
    assert_eq!(session.board.as_ref().unwrap().dead_starcatchers.len(), 1);
    session.validate().unwrap();
    session
}

fn tank_three_grubber_gekko_corpse_session() -> AdventureSession {
    use turbofish_deluxe::{gekko::DeadGekko, grubber::DeadGrubber};

    let mut session = bought_gekko_session();
    let board = session.board.as_mut().unwrap();
    let grubber = board.grubbers.remove(0);
    let gekko = board.gekkos.remove(0);
    board.dead_grubbers.push(DeadGrubber::from_impact(&grubber));
    board.dead_gekkos.push(DeadGekko::from_impact(&gekko));
    session.validate().unwrap();
    session
}

fn current_bytes(session: AdventureSession) -> Vec<u8> {
    serde_json::to_vec(&cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session,
    })
    .unwrap()
}

fn prepared_tank_five_session(attempts: u32) -> AdventureSession {
    // Prepared unit fixture at the post-4-5 boundary; it is not earned
    // progression or native-game evidence.
    let mut session = tank_four_finale_session(&[PetKind::Angie]);
    session.progress.tank = 5;
    session.progress.level = 1;
    session.progress.cyrax_attempts = attempts;
    session.board = Some(AdventureState::new_tank5_1(42, attempts));
    session.validate().unwrap();
    session
}

fn prepared_tank_five_boss_session(attempts: u32) -> AdventureSession {
    use turbofish_deluxe::invasion::WarningCoords;

    let mut session = prepared_tank_five_session(attempts);
    let wave = session.board.as_mut().unwrap().invasion.as_mut().unwrap();
    wave.countdown = 1;
    wave.warning = Some(WarningCoords {
        first_x: 100,
        first_y: 120,
        second_x: 140,
        second_y: 150,
    });
    session.step(&[]);
    assert!(
        session
            .board
            .as_ref()
            .unwrap()
            .invasion
            .as_ref()
            .unwrap()
            .finale
            .as_ref()
            .unwrap()
            .boss
            .is_some()
    );
    session.validate().unwrap();
    session
}

fn pending_tank_four_second_session() -> AdventureSession {
    use turbofish_deluxe::{
        bilaterus::BilaterusState,
        sim::{Coin, CoinKind, Food},
    };

    // Controlled current-format state, never evidence of earned purchases.
    // Use the serialized allocator boundary so all new identities remain valid.
    let mut value = serde_json::to_value(tank_four_second_session(&[PetKind::Nimbus])).unwrap();
    let next_id = value["board"]["next_id"].as_u64().unwrap();
    value["board"]["next_id"] = (next_id + 3).into();
    let mut session: AdventureSession = serde_json::from_value(value).unwrap();
    let board = session.board.as_mut().unwrap();
    board.balance = 10_000;
    board.upgrades.quality_unlocked = true;
    board.upgrades.quantity_unlocked = true;
    board.oscar_unlocked = true;
    board.egg_unlocked = true;
    board.ultra_unlocked = true;
    board.weapon_unlocked = true;
    let mut group = BilaterusState::spawn(next_id, 400, 110, &mut || 1);
    group.emergence_ticks = 1;
    assert!(group.pet_damage(101.0) < 0.0);
    let wave = board.invasion.as_mut().unwrap();
    wave.bilaterus.push(group);
    wave.battle_active = true;
    board.coins.push(Coin {
        id: next_id + 1,
        x: 100.25,
        y: 160.5,
        kind: CoinKind::ShellGold,
        frame: 0,
        animation_ticks: 6,
        hazard_age_ticks: 23,
        collecting: false,
        bottom_ticks: 0,
        fade_ticks: 0,
        penta_rising: true,
    });
    board.food.push(Food {
        id: next_id + 2,
        x: 110.0,
        y: 170.0,
        frame: 0,
        food_type: turbofish_deluxe::sim::FoodType::Ordinary,
        picked_up: false,
        ineligible_ticks: 0,
        removal_ticks: 0,
        quality: 1,
        direction: 0,
        vx: 0.0,
        vy: -2.0,
        animation_period: 3,
        free_from_zorf: false,
        nimbus_rising: true,
    });
    session.apply_actions(&[Action::BuyUltra]);
    assert_eq!(session.board.as_ref().unwrap().ultras.len(), 1);
    session.validate().unwrap();
    session
}

#[test]
fn current_tank_four_second_entry_retains_sixteen_rosters_and_stage_constants() {
    use turbofish_deluxe::invasion::WavePlan;

    let canonical = tank_four_second_session(&[]).progress.unlocked_pets;
    assert_eq!(canonical.len(), 16);
    for pet in canonical {
        let session = tank_four_second_session(&[pet]);
        session.validate().unwrap();
        let resumed = cli::decode_save(&current_bytes(session.clone())).unwrap();
        assert_eq!(
            serde_json::to_value(resumed).unwrap(),
            serde_json::to_value(session).unwrap(),
            "single-pet roster {pet:?} changed on reload"
        );
    }
    let session = tank_four_second_session(&[PetKind::Niko, PetKind::Itchy, PetKind::Nimbus]);
    let board = session.board.as_ref().unwrap();
    assert_eq!(
        (board.tank, board.level, board.balance, board.egg_price),
        (4, 2, 200, 25_000)
    );
    assert_eq!(board.breeders.len(), 1);
    assert_eq!(board.breeders[0].food_points, 2);
    assert!(board.fish.is_empty());
    assert_eq!(
        board.pets,
        vec![PetKind::Niko, PetKind::Itchy, PetKind::Nimbus]
    );
    assert!(board.ultras.is_empty() && board.dead_ultras.is_empty());
    let wave = board.invasion.as_ref().unwrap();
    assert_eq!(wave.plan, WavePlan::FixedBilaterus);
    assert_eq!(wave.countdown, 3000);
    assert!(wave.bilaterus.is_empty() && wave.fragments.is_empty());
    assert!(AdventureState::new_tank4_second_stage(42, &[PetKind::Amp]).is_err());
}

#[test]
fn current_tank_four_second_pending_group_ultra_and_rising_items_resume_exactly() {
    let mut uninterrupted = pending_tank_four_second_session();
    let mut resumed = cli::decode_save(&current_bytes(uninterrupted.clone())).unwrap();
    assert_eq!(
        serde_json::to_value(&resumed).unwrap(),
        serde_json::to_value(&uninterrupted).unwrap()
    );
    // The active head has pet-lethal health but must survive the saved
    // emergence boundary. Its first death, fragments, Ultra, food and coin
    // must use identical IDs, draws and phases after reopening.
    assert!(
        resumed
            .board
            .as_ref()
            .unwrap()
            .invasion
            .as_ref()
            .unwrap()
            .bilaterus[0]
            .active()
            .health
            < 0.0
    );
    for _ in 0..40 {
        let expected = uninterrupted.step(&[]);
        let actual = resumed.step(&[]);
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::to_value(expected).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&resumed).unwrap(),
            serde_json::to_value(&uninterrupted).unwrap()
        );
        resumed.validate().unwrap();
    }
    let board = resumed.board.as_ref().unwrap();
    assert!(board.invasion.as_ref().unwrap().bilaterus[0].first_head_lost);
    assert_eq!(board.ultras.len(), 1);
    assert_eq!(
        board.coins[0].kind,
        turbofish_deluxe::sim::CoinKind::ShellGold
    );
    assert_eq!(board.coins[0].hazard_age_ticks, 63);
}

#[test]
fn current_tank_four_second_rejects_missing_and_impossible_live_state() {
    let current = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: pending_tank_four_second_session(),
    })
    .unwrap();
    for field in ["ultra_unlocked", "ultras", "dead_ultras"] {
        let mut missing = current.clone();
        missing["session"]["board"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err(),
            "missing board {field}"
        );
    }
    for field in ["bilaterus", "fragments"] {
        let mut missing = current.clone();
        missing["session"]["board"]["invasion"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err(),
            "missing wave {field}"
        );
    }
    for (path, field) in [
        ("ultras", "coin_timer"),
        ("bilaterus", "active_head"),
        ("coins", "hazard_age_ticks"),
        ("food", "nimbus_rising"),
    ] {
        let mut missing = current.clone();
        let items = if path == "bilaterus" {
            &mut missing["session"]["board"]["invasion"][path]
        } else {
            &mut missing["session"]["board"][path]
        };
        items[0].as_object_mut().unwrap().remove(field);
        assert!(
            cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err(),
            "missing {path}[0].{field}"
        );
    }
    for (path, field) in [("heads", "health"), ("bones", "follow_vx")] {
        let mut missing = current.clone();
        let group = &mut missing["session"]["board"]["invasion"]["bilaterus"][0];
        if path == "heads" {
            let active = group["active_head"].as_u64().unwrap() as usize;
            group[path][active].as_object_mut().unwrap().remove(field);
        } else {
            group[path][0].as_object_mut().unwrap().remove(field);
        }
        assert!(
            cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err(),
            "missing Bilaterus {path} field {field}"
        );
    }
    let mut corpse_session = pending_tank_four_second_session();
    let board = corpse_session.board.as_mut().unwrap();
    let ultra = board.ultras.remove(0);
    board
        .dead_ultras
        .push(turbofish_deluxe::ultra::DeadUltra::from_impact(&ultra));
    corpse_session.validate().unwrap();
    let mut missing_corpse = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: corpse_session,
    })
    .unwrap();
    missing_corpse["session"]["board"]["dead_ultras"][0]
        .as_object_mut()
        .unwrap()
        .remove("remaining_ticks");
    assert!(
        cli::decode_save(&serde_json::to_vec(&missing_corpse).unwrap()).is_err(),
        "missing Ultra corpse lifetime"
    );
    let mut invalid = current;
    invalid["session"]["board"]["invasion"]["bilaterus"][0]["first_head_lost"] = true.into();
    assert!(
        cli::decode_save(&serde_json::to_vec(&invalid).unwrap()).is_err(),
        "two-head group cannot claim its first head is lost"
    );
}

#[test]
fn current_replay_fields_are_explicit_and_missing_values_reject() {
    let current = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: AdventureSession::new(0x3758),
    })
    .unwrap();
    for (path, field) in [
        ("progress", "adventure_completions"),
        ("board", "profile_population"),
        ("board", "bonus_active"),
        ("board", "bonus_tally"),
    ] {
        let mut missing = current.clone();
        missing["session"][path]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err(),
            "missing {path}.{field}"
        );
    }
    assert!(cli::decode_save(&serde_json::to_vec(&current).unwrap()).is_ok());
}

#[test]
fn completed_replay_bonus_board_and_tally_continue_through_current_save() {
    use turbofish_deluxe::time_trial::SUPPORTED_PETS;
    let mut session = AdventureSession::new(0x3761);
    session.progress.tank = 5;
    session.progress.level = 2;
    session.progress.adventure_completed = true;
    session.progress.adventure_completions = 1;
    session.progress.unlocked_pets = SUPPORTED_PETS
        .into_iter()
        .chain([PetKind::Presto])
        .collect();
    session.board = None;
    session.phase = AdventurePhase::GameSelector;
    session.apply_actions(&[Action::PlayAdventure, Action::Continue]);
    session.apply_actions(&[
        Action::TogglePet {
            pet: PetKind::Stinky,
        },
        Action::TogglePet { pet: PetKind::Niko },
        Action::TogglePet {
            pet: PetKind::Presto,
        },
        Action::Continue,
    ]);
    let board = session.board.as_mut().unwrap();
    board.egg_unlocked = true;
    board.balance = 1000;
    session.apply_actions(&[Action::BuyEgg, Action::BuyEgg, Action::BuyEgg]);
    assert!(matches!(session.phase, AdventurePhase::Bonus { .. }));
    assert!(session.board.as_ref().unwrap().bonus_active);
    session.validate().unwrap();
    let encoded = serde_json::to_vec(&cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: session.clone(),
    })
    .unwrap();
    let mut reopened = cli::decode_save(&encoded).unwrap();
    for _ in 0..12 {
        assert_eq!(
            serde_json::to_value(session.step(&[])).unwrap(),
            serde_json::to_value(reopened.step(&[])).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&session).unwrap(),
            serde_json::to_value(&reopened).unwrap()
        );
    }
    let mut forged: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    forged["session"]["board"]["bonus_tally"] = 1.into();
    assert!(cli::decode_save(&serde_json::to_vec(&forged).unwrap()).is_err());
}

#[test]
fn current_tank_four_second_pause_keeps_live_board_while_session_time_advances() {
    let session = pending_tank_four_second_session();
    let mut resumed = cli::decode_save(&current_bytes(session)).unwrap();
    let board_before = serde_json::to_value(resumed.board.as_ref().unwrap()).unwrap();
    let session_ticks_before = resumed.ticks;
    for _ in 0..12 {
        resumed.paused_step();
        assert_eq!(
            serde_json::to_value(resumed.board.as_ref().unwrap()).unwrap(),
            board_before
        );
    }
    assert_eq!(resumed.ticks, session_ticks_before + 12);
    resumed.validate().unwrap();
    let mut reopened = cli::decode_save(&current_bytes(resumed.clone())).unwrap();
    assert_eq!(
        serde_json::to_value(&reopened).unwrap(),
        serde_json::to_value(&resumed).unwrap()
    );
    assert_eq!(
        serde_json::to_value(reopened.step(&[])).unwrap(),
        serde_json::to_value(resumed.step(&[])).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&reopened).unwrap(),
        serde_json::to_value(&resumed).unwrap()
    );
}

#[test]
fn current_tank_four_third_accepts_seventeen_rosters_and_persists_live_amp_setup() {
    use turbofish_deluxe::{alien::SylvesterKind, fish_pet::FishPetKind, invasion::EncounterKind};

    assert_eq!(cli::SAVE_FORMAT_VERSION, 28);
    let canonical = tank_four_third_session(&[]).progress.unlocked_pets;
    assert_eq!(canonical.len(), 17);
    for pet in canonical {
        let session = tank_four_third_session(&[pet]);
        session.validate().unwrap();
        let resumed = cli::decode_save(&current_bytes(session.clone())).unwrap();
        assert_eq!(
            serde_json::to_value(resumed).unwrap(),
            serde_json::to_value(session).unwrap(),
            "single-pet roster {pet:?} changed on reload"
        );
    }
    let session = tank_four_third_session(&[PetKind::Niko, PetKind::Nimbus, PetKind::Amp]);
    let board = session.board.as_ref().unwrap();
    assert_eq!(
        (board.tank, board.level, board.balance, board.egg_price),
        (4, 3, 200, 50_000)
    );
    assert_eq!(board.breeders.len(), 1);
    assert_eq!(board.breeders[0].food_points, 2);
    assert!(board.fish.is_empty());
    assert_eq!(board.fish_pets.len(), 2);
    let amp = board
        .fish_pets
        .iter()
        .find(|pet| pet.kind == FishPetKind::Amp)
        .unwrap();
    assert_eq!(
        (amp.amp_timer, amp.amp_threshold, amp.amp_charge),
        (300, 3000, 0)
    );
    let wave = board.invasion.as_ref().unwrap();
    assert_eq!(
        wave.plan.expected(),
        EncounterKind::Single(SylvesterKind::Gus)
    );
    assert_eq!(wave.countdown, 3000);
    assert!(wave.actors.is_empty());
    assert!(AdventureState::new_tank4_second_stage(42, &[PetKind::Amp]).is_err());
    assert!(AdventureState::new_tank4_third_stage(42, &[PetKind::Gash]).is_err());
}

#[test]
fn current_tank_four_third_requires_each_amp_field_without_backfill() {
    let mut session = tank_four_third_session(&[PetKind::Nimbus, PetKind::Amp]);
    let amp = session
        .board
        .as_mut()
        .unwrap()
        .fish_pets
        .last_mut()
        .unwrap();
    amp.amp_timer = 3000;
    amp.amp_charge = 2;
    session.validate().unwrap();
    let current = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session,
    })
    .unwrap();
    for pet_index in 0..2 {
        for field in ["amp_timer", "amp_threshold", "amp_charge"] {
            let mut missing = current.clone();
            missing["session"]["board"]["fish_pets"][pet_index]
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(
                cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err(),
                "current format accepted missing fish_pets[{pet_index}].{field}"
            );
        }
    }
    let mut invalid = current;
    invalid["session"]["board"]["fish_pets"][1]["amp_charge"] = 3.into();
    assert!(cli::decode_save(&serde_json::to_vec(&invalid).unwrap()).is_err());
}

#[test]
fn current_tank_four_third_live_amp_fish_coin_and_effect_resume_through_pause() {
    use turbofish_deluxe::{
        breeder::BreederSize,
        fish_pet::{AmpTap, FishPetKind},
        sim::{BombShot, Coin, CoinKind, Event},
    };

    // Controlled current-format fixture. The Breeder creates a real class-1
    // guppy; one finite electric-style Shot and one 200-value collectible
    // probe the durable types. This does not claim an earned Amp discharge.
    let mut uninterrupted = tank_four_third_session(&[PetKind::Amp]);
    let board = uninterrupted.board.as_mut().unwrap();
    let breeder = &mut board.breeders[0];
    breeder.size = BreederSize::Medium;
    breeder.food_points = 0;
    breeder.food_needed_to_grow = 7;
    breeder.birth_clock = 999;
    breeder.birth_threshold = 1000;
    breeder.hunger = 300;
    board.breeder_unlocked = true;
    let events = uninterrupted.step(&[]);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::BreederBornGuppy { .. }))
    );
    assert_eq!(uninterrupted.board.as_ref().unwrap().fish.len(), 1);

    let mut value = serde_json::to_value(&uninterrupted).unwrap();
    let coin_id = value["board"]["next_id"].as_u64().unwrap();
    value["board"]["next_id"] = (coin_id + 1).into();
    uninterrupted = serde_json::from_value(value).unwrap();
    let board = uninterrupted.board.as_mut().unwrap();
    board.coins.push(Coin {
        id: coin_id,
        x: 100.25,
        y: 160.5,
        kind: CoinKind::Diamond,
        frame: 0,
        animation_ticks: 6,
        hazard_age_ticks: 0,
        collecting: false,
        bottom_ticks: 0,
        fade_ticks: 0,
        penta_rising: false,
    });
    board.bomb_shots.push(BombShot {
        shot_type: 3,
        x: 220,
        y: 180,
        age_ticks: 12,
        frame: 5,
        delay_ticks: 0,
        alpha: 150,
    });
    let amp = board
        .fish_pets
        .iter_mut()
        .find(|pet| pet.kind == FishPetKind::Amp)
        .unwrap();
    amp.amp_timer = 3000;
    amp.amp_charge = 2;
    uninterrupted.validate().unwrap();

    let mut resumed = cli::decode_save(&current_bytes(uninterrupted.clone())).unwrap();
    assert_eq!(
        serde_json::to_value(&resumed).unwrap(),
        serde_json::to_value(&uninterrupted).unwrap()
    );
    let board_before = serde_json::to_value(resumed.board.as_ref().unwrap()).unwrap();
    let session_ticks_before = resumed.ticks;
    for _ in 0..12 {
        resumed.paused_step();
        assert_eq!(
            serde_json::to_value(resumed.board.as_ref().unwrap()).unwrap(),
            board_before
        );
    }
    assert_eq!(resumed.ticks, session_ticks_before + 12);
    for _ in 0..12 {
        uninterrupted.paused_step();
    }
    let mut reopened = cli::decode_save(&current_bytes(resumed)).unwrap();
    assert_eq!(
        serde_json::to_value(&reopened).unwrap(),
        serde_json::to_value(&uninterrupted).unwrap()
    );
    for session in [&mut reopened, &mut uninterrupted] {
        let amp = session
            .board
            .as_mut()
            .unwrap()
            .fish_pets
            .iter_mut()
            .find(|pet| pet.kind == FishPetKind::Amp)
            .unwrap();
        assert_eq!(amp.tap_amp(false), AmpTap::Discharged);
        assert_eq!(
            (amp.amp_timer, amp.amp_threshold, amp.amp_charge),
            (-20, 3200, 0)
        );
    }
    for _ in 0..35 {
        assert_eq!(
            serde_json::to_value(reopened.step(&[])).unwrap(),
            serde_json::to_value(uninterrupted.step(&[])).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&reopened).unwrap(),
            serde_json::to_value(&uninterrupted).unwrap()
        );
        reopened.validate().unwrap();
    }
    let board = reopened.board.as_ref().unwrap();
    assert!(board.bomb_shots.is_empty());
    assert_eq!(board.coins[0].kind, CoinKind::Diamond);
    assert_eq!(board.coins[0].kind.value(), 200);
    assert!(!board.fish.is_empty());
}

#[test]
fn current_tank_four_third_pending_mixed_spawn_keeps_order_and_next_draw_on_reload() {
    use turbofish_deluxe::{
        alien::SylvesterKind,
        invasion::{EncounterKind, WarningCoords, WavePlan},
    };

    // PB68 chooses the following encounter only after both pending actors
    // have been constructed. A reload at countdown one must retain the
    // source-ordered pair, allocator, RNG and next-plan transition.
    let mut uninterrupted = tank_four_third_session(&[PetKind::Amp]);
    let wave = uninterrupted
        .board
        .as_mut()
        .unwrap()
        .invasion
        .as_mut()
        .unwrap();
    wave.plan = WavePlan::CyclingTank4Third {
        next: EncounterKind::PsychosquidBalrogPair,
    };
    wave.countdown = 1;
    wave.warning = Some(WarningCoords {
        first_x: 120,
        first_y: 180,
        second_x: 410,
        second_y: 240,
    });
    uninterrupted.validate().unwrap();
    let mut resumed = cli::decode_save(&current_bytes(uninterrupted.clone())).unwrap();
    assert_eq!(
        serde_json::to_value(&resumed).unwrap(),
        serde_json::to_value(&uninterrupted).unwrap()
    );
    for _ in 0..8 {
        assert_eq!(
            serde_json::to_value(resumed.step(&[])).unwrap(),
            serde_json::to_value(uninterrupted.step(&[])).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&resumed).unwrap(),
            serde_json::to_value(&uninterrupted).unwrap()
        );
        resumed.validate().unwrap();
    }
    let wave = resumed.board.as_ref().unwrap().invasion.as_ref().unwrap();
    assert_eq!(wave.actors.len(), 2);
    assert_eq!(wave.actors[0].kind, SylvesterKind::Psychosquid);
    assert_eq!(wave.actors[1].kind, SylvesterKind::Balrog);
    assert_eq!(wave.countdown, 3000);
    assert!(wave.battle_active);
    assert!(matches!(wave.plan, WavePlan::CyclingTank4Third { .. }));
}

#[test]
fn current_tank_four_fourth_accepts_eighteen_rosters_and_persists_gash_setup() {
    use turbofish_deluxe::{
        fish_pet::FishPetKind,
        invasion::{EncounterKind, WavePlan},
    };

    assert_eq!(cli::SAVE_FORMAT_VERSION, 28);
    let canonical = tank_four_fourth_session(&[]).progress.unlocked_pets;
    assert_eq!(canonical.len(), 18);
    for pet in canonical {
        let session = tank_four_fourth_session(&[pet]);
        session.validate().unwrap();
        let reopened = cli::decode_save(&current_bytes(session.clone())).unwrap();
        assert_eq!(
            serde_json::to_value(reopened).unwrap(),
            serde_json::to_value(session).unwrap(),
            "single-pet roster {pet:?} changed on reload"
        );
    }
    let session = tank_four_fourth_session(&[PetKind::Niko, PetKind::Amp, PetKind::Gash]);
    let board = session.board.as_ref().unwrap();
    assert_eq!(
        (board.tank, board.level, board.balance, board.egg_price),
        (4, 4, 200, 75_000)
    );
    assert_eq!(board.breeders.len(), 1);
    assert_eq!(board.breeders[0].food_points, 2);
    assert!(board.fish.is_empty());
    let gash = board
        .fish_pets
        .iter()
        .find(|pet| pet.kind == FishPetKind::Gash)
        .unwrap();
    assert_eq!((gash.gash_timer, gash.gash_eating_ticks), (-1550, 0));
    for pet in board
        .fish_pets
        .iter()
        .filter(|pet| pet.kind != FishPetKind::Gash)
    {
        assert_eq!((pet.gash_timer, pet.gash_eating_ticks), (0, 0));
    }
    let wave = board.invasion.as_ref().unwrap();
    assert!(matches!(wave.plan, WavePlan::CyclingTank4Fourth { .. }));
    assert_eq!(wave.plan.expected(), EncounterKind::Bilaterus);
    assert_eq!(wave.countdown, 3000);
    assert!(wave.actors.is_empty() && wave.bilaterus.is_empty());
    assert!(AdventureState::new_tank4_third_stage(42, &[PetKind::Gash]).is_err());
    assert!(AdventureState::new_tank4_fourth_stage(42, &[PetKind::Angie]).is_err());
}

#[test]
fn current_tank_four_fourth_requires_each_gash_field_on_every_fish_pet() {
    let mut session = tank_four_fourth_session(&[PetKind::Nimbus, PetKind::Amp, PetKind::Gash]);
    let gash = session
        .board
        .as_mut()
        .unwrap()
        .fish_pets
        .last_mut()
        .unwrap();
    gash.gash_timer = 1569;
    gash.gash_eating_ticks = 4;
    session.validate().unwrap();
    let current = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session,
    })
    .unwrap();
    for pet_index in 0..3 {
        for field in ["gash_timer", "gash_eating_ticks"] {
            let mut missing = current.clone();
            missing["session"]["board"]["fish_pets"][pet_index]
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(
                cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err(),
                "current format accepted missing fish_pets[{pet_index}].{field}"
            );
        }
    }
    // Format nineteen retains the format-eighteen Amp fields too, even on a
    // selected Gash instance that never uses an Amp charge clock.
    for field in ["amp_timer", "amp_threshold", "amp_charge"] {
        let mut missing = current.clone();
        missing["session"]["board"]["fish_pets"][2]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err(),
            "current format accepted missing Gash.{field}"
        );
    }
}

#[test]
fn current_tank_four_fourth_live_gash_fish_coin_and_effect_resume_through_pause() {
    use turbofish_deluxe::{
        breeder::BreederSize,
        fish_pet::FishPetKind,
        sim::{BombShot, Coin, CoinKind, Event},
    };

    // Controlled current-format state, not an earned Gash board. A real
    // Breeder birth supplies an ordinary class-1 fish; a coin and finite
    // effect probe durable field preservation independent of a Gash hit.
    let mut uninterrupted = tank_four_fourth_session(&[PetKind::Amp, PetKind::Gash]);
    let board = uninterrupted.board.as_mut().unwrap();
    let breeder = &mut board.breeders[0];
    breeder.size = BreederSize::Medium;
    breeder.food_points = 0;
    breeder.food_needed_to_grow = 7;
    breeder.birth_clock = 999;
    breeder.birth_threshold = 1000;
    breeder.hunger = 300;
    board.breeder_unlocked = true;
    assert!(
        uninterrupted
            .step(&[])
            .iter()
            .any(|event| matches!(event, Event::BreederBornGuppy { .. }))
    );
    assert_eq!(uninterrupted.board.as_ref().unwrap().fish.len(), 1);

    let mut value = serde_json::to_value(&uninterrupted).unwrap();
    let coin_id = value["board"]["next_id"].as_u64().unwrap();
    value["board"]["next_id"] = (coin_id + 1).into();
    uninterrupted = serde_json::from_value(value).unwrap();
    let board = uninterrupted.board.as_mut().unwrap();
    board.coins.push(Coin {
        id: coin_id,
        x: 110.25,
        y: 165.5,
        kind: CoinKind::Diamond,
        frame: 0,
        animation_ticks: 6,
        hazard_age_ticks: 0,
        collecting: false,
        bottom_ticks: 0,
        fade_ticks: 0,
        penta_rising: false,
    });
    board.bomb_shots.push(BombShot {
        shot_type: 3,
        x: 210,
        y: 175,
        age_ticks: 12,
        frame: 5,
        delay_ticks: 0,
        alpha: 150,
    });
    let gash = board
        .fish_pets
        .iter_mut()
        .find(|pet| pet.kind == FishPetKind::Gash)
        .unwrap();
    gash.gash_timer = 1569;
    gash.gash_eating_ticks = 4;
    uninterrupted.validate().unwrap();

    let mut reopened = cli::decode_save(&current_bytes(uninterrupted.clone())).unwrap();
    assert_eq!(
        serde_json::to_value(&reopened).unwrap(),
        serde_json::to_value(&uninterrupted).unwrap()
    );
    let board_before = serde_json::to_value(reopened.board.as_ref().unwrap()).unwrap();
    let before_ticks = reopened.ticks;
    for _ in 0..12 {
        reopened.paused_step();
        assert_eq!(
            serde_json::to_value(reopened.board.as_ref().unwrap()).unwrap(),
            board_before
        );
        uninterrupted.paused_step();
    }
    assert_eq!(reopened.ticks, before_ticks + 12);
    reopened = cli::decode_save(&current_bytes(reopened)).unwrap();
    assert_eq!(
        serde_json::to_value(&reopened).unwrap(),
        serde_json::to_value(&uninterrupted).unwrap()
    );
    for _ in 0..35 {
        assert_eq!(
            serde_json::to_value(reopened.step(&[])).unwrap(),
            serde_json::to_value(uninterrupted.step(&[])).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&reopened).unwrap(),
            serde_json::to_value(&uninterrupted).unwrap()
        );
        reopened.validate().unwrap();
    }
    let board = reopened.board.as_ref().unwrap();
    assert!(board.bomb_shots.is_empty());
    assert_eq!(board.coins[0].kind, CoinKind::Diamond);
    assert_eq!(board.coins[0].kind.value(), 200);
}

#[test]
fn current_tank_four_fourth_pending_pair_preserves_constructor_order_and_next_plan() {
    use turbofish_deluxe::{
        alien::SylvesterKind,
        invasion::{EncounterKind, WarningCoords, WavePlan},
    };

    // PB69 selects the next wave after this source-ordered pair constructs.
    // Reopening at countdown one must preserve both actor IDs and RNG use.
    let mut uninterrupted = tank_four_fourth_session(&[PetKind::Gash]);
    let wave = uninterrupted
        .board
        .as_mut()
        .unwrap()
        .invasion
        .as_mut()
        .unwrap();
    wave.plan = WavePlan::CyclingTank4Fourth {
        next: EncounterKind::DestructorUlyssesPair,
    };
    wave.countdown = 1;
    wave.warning = Some(WarningCoords {
        first_x: 130,
        first_y: 180,
        second_x: 400,
        second_y: 230,
    });
    uninterrupted.validate().unwrap();
    let mut reopened = cli::decode_save(&current_bytes(uninterrupted.clone())).unwrap();
    assert_eq!(
        serde_json::to_value(&reopened).unwrap(),
        serde_json::to_value(&uninterrupted).unwrap()
    );
    for _ in 0..8 {
        assert_eq!(
            serde_json::to_value(reopened.step(&[])).unwrap(),
            serde_json::to_value(uninterrupted.step(&[])).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&reopened).unwrap(),
            serde_json::to_value(&uninterrupted).unwrap()
        );
        reopened.validate().unwrap();
    }
    let wave = reopened.board.as_ref().unwrap().invasion.as_ref().unwrap();
    assert_eq!(wave.actors.len(), 2);
    assert_eq!(wave.actors[0].kind, SylvesterKind::Destructor);
    assert_eq!(wave.actors[1].kind, SylvesterKind::Ulysses);
    assert_eq!(wave.countdown, 3000);
    assert!(wave.battle_active);
    assert!(matches!(wave.plan, WavePlan::CyclingTank4Fourth { .. }));
}

#[test]
fn current_tank_four_finale_accepts_nineteen_rosters_and_starts_with_bilaterus() {
    use turbofish_deluxe::{
        fish_pet::FishPetKind,
        invasion::{EncounterKind, WavePlan},
    };

    assert_eq!(cli::SAVE_FORMAT_VERSION, 28);
    let canonical = tank_four_finale_session(&[]).progress.unlocked_pets;
    assert_eq!(canonical.len(), 19);
    for pet in canonical {
        let session = tank_four_finale_session(&[pet]);
        session.validate().unwrap();
        let reopened = cli::decode_save(&current_bytes(session.clone())).unwrap();
        assert_eq!(
            serde_json::to_value(reopened).unwrap(),
            serde_json::to_value(session).unwrap(),
            "single-pet roster {pet:?} changed on reload"
        );
    }
    let session = tank_four_finale_session(&[PetKind::Niko, PetKind::Gash, PetKind::Angie]);
    let board = session.board.as_ref().unwrap();
    assert_eq!(
        (board.tank, board.level, board.balance, board.egg_price),
        (4, 5, 200, 99_999)
    );
    assert_eq!(board.breeders.len(), 1);
    assert_eq!(board.breeders[0].food_points, 2);
    assert!(board.fish.is_empty());
    assert!(
        board
            .fish_pets
            .iter()
            .any(|pet| pet.kind == FishPetKind::Angie)
    );
    let wave = board.invasion.as_ref().unwrap();
    assert_eq!(wave.plan.expected(), EncounterKind::Bilaterus);
    assert!(matches!(
        wave.plan,
        WavePlan::CyclingTank4Finale { wave_count: 0, .. }
    ));
    assert_eq!(wave.countdown, 3000);
    assert!(wave.actors.is_empty() && wave.bilaterus.is_empty());
    assert!(AdventureState::new_tank4_fourth_stage(42, &[PetKind::Angie]).is_err());
}

#[test]
fn current_twenty_one_requires_revival_clock_on_all_seven_valid_corpse_kinds() {
    // Each complete source state first decodes on its own valid stage. The
    // seven negative probes then remove only one required clock at a time.
    for (session, lists) in [
        (
            tank_four_finale_supported_corpse_session(&[]),
            &["dead_fish", "dead_oscars", "dead_ultras", "dead_breeders"][..],
        ),
        (
            tank_two_starcatcher_corpse_session(),
            &["dead_starcatchers"][..],
        ),
        (
            tank_three_grubber_gekko_corpse_session(),
            &["dead_grubbers", "dead_gekkos"][..],
        ),
    ] {
        let complete = serde_json::to_value(cli::ProjectSave {
            format_version: cli::SAVE_FORMAT_VERSION,
            session,
        })
        .unwrap();
        cli::decode_save(&serde_json::to_vec(&complete).unwrap()).unwrap();
        for list in lists {
            assert_eq!(complete["session"]["board"][*list][0]["revival_ticks"], 100);
            let mut missing = complete.clone();
            missing["session"]["board"][*list][0]
                .as_object_mut()
                .unwrap()
                .remove("revival_ticks");
            let error = cli::decode_save(&serde_json::to_vec(&missing).unwrap()).unwrap_err();
            assert!(
                error.to_string().contains("Incomplete format-twenty-eight"),
                "missing {list}[0].revival_ticks: {error}"
            );
        }
    }
}

#[test]
fn current_tank_four_finale_live_angie_pending_revival_survives_pause_and_reload() {
    use turbofish_deluxe::sim::{Coin, CoinKind, Event};

    // The corpse came from an ordinary Breeder-born guppy. Marking its
    // 100-clock as 10 models a successful Angie contact without claiming
    // that this fixture exercised the native/widget contact route.
    let mut uninterrupted =
        tank_four_finale_dead_guppy_session(&[PetKind::Amp, PetKind::Gash, PetKind::Angie]);
    let mut value = serde_json::to_value(&uninterrupted).unwrap();
    let coin_id = value["board"]["next_id"].as_u64().unwrap();
    value["board"]["next_id"] = (coin_id + 1).into();
    uninterrupted = serde_json::from_value(value).unwrap();
    let board = uninterrupted.board.as_mut().unwrap();
    let corpse_id = board.dead_fish[0].id;
    let corpse_size = board.dead_fish[0].size;
    board.dead_fish[0].revival_ticks = 10;
    board.coins.push(Coin {
        id: coin_id,
        x: 100.25,
        y: 160.5,
        kind: CoinKind::Diamond,
        frame: 0,
        animation_ticks: 6,
        hazard_age_ticks: 0,
        collecting: false,
        bottom_ticks: 0,
        fade_ticks: 0,
        penta_rising: false,
    });
    uninterrupted.validate().unwrap();

    let mut reopened = cli::decode_save(&current_bytes(uninterrupted.clone())).unwrap();
    assert_eq!(
        serde_json::to_value(&reopened).unwrap(),
        serde_json::to_value(&uninterrupted).unwrap()
    );
    let board_before = serde_json::to_value(reopened.board.as_ref().unwrap()).unwrap();
    let before_ticks = reopened.ticks;
    for _ in 0..12 {
        reopened.paused_step();
        uninterrupted.paused_step();
        assert_eq!(
            serde_json::to_value(reopened.board.as_ref().unwrap()).unwrap(),
            board_before
        );
    }
    assert_eq!(reopened.ticks, before_ticks + 12);
    reopened = cli::decode_save(&current_bytes(reopened)).unwrap();
    assert_eq!(
        serde_json::to_value(&reopened).unwrap(),
        serde_json::to_value(&uninterrupted).unwrap()
    );
    let mut revived = Vec::new();
    for _ in 0..10 {
        let actual = reopened.step(&[]);
        let expected = uninterrupted.step(&[]);
        assert_eq!(
            serde_json::to_value(&actual).unwrap(),
            serde_json::to_value(&expected).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&reopened).unwrap(),
            serde_json::to_value(&uninterrupted).unwrap()
        );
        revived.extend(actual.into_iter().filter_map(|event| match event {
            Event::FishRevived {
                corpse_id: old_id,
                fish_id,
                ..
            } if old_id == corpse_id => Some(fish_id),
            _ => None,
        }));
        reopened.validate().unwrap();
    }
    assert_eq!(revived.len(), 1);
    let new_id = revived[0];
    assert!(new_id > coin_id && new_id != corpse_id);
    let board = reopened.board.as_ref().unwrap();
    assert!(board.dead_fish.iter().all(|corpse| corpse.id != corpse_id));
    let fresh = board.fish.iter().find(|fish| fish.id == new_id).unwrap();
    assert_eq!(fresh.size, corpse_size);
    assert!(
        fresh.hunger > 300,
        "revival reused the starving fish's hunger"
    );
    assert_eq!(board.coins[0].kind, CoinKind::Diamond);
}

#[test]
fn current_tank_four_finale_pending_balrog_bilaterus_keeps_order_and_counter() {
    use turbofish_deluxe::{
        alien::SylvesterKind,
        invasion::{EncounterKind, WarningCoords, WavePlan},
    };

    // PB73's raw12 first creates Balrog and then Bilaterus. This explicit
    // pre-spawn state exercises source order and the post-construction draw.
    let mut uninterrupted = tank_four_finale_session(&[]);
    let wave = uninterrupted
        .board
        .as_mut()
        .unwrap()
        .invasion
        .as_mut()
        .unwrap();
    wave.plan = WavePlan::CyclingTank4Finale {
        next: EncounterKind::BalrogBilaterusPair,
        wave_count: 6,
    };
    wave.countdown = 1;
    wave.warning = Some(WarningCoords {
        first_x: 125,
        first_y: 175,
        second_x: 410,
        second_y: 235,
    });
    uninterrupted.validate().unwrap();
    let mut reopened = cli::decode_save(&current_bytes(uninterrupted.clone())).unwrap();
    assert_eq!(
        serde_json::to_value(&reopened).unwrap(),
        serde_json::to_value(&uninterrupted).unwrap()
    );
    for _ in 0..8 {
        assert_eq!(
            serde_json::to_value(reopened.step(&[])).unwrap(),
            serde_json::to_value(uninterrupted.step(&[])).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&reopened).unwrap(),
            serde_json::to_value(&uninterrupted).unwrap()
        );
        reopened.validate().unwrap();
    }
    let wave = reopened.board.as_ref().unwrap().invasion.as_ref().unwrap();
    assert_eq!(wave.actors.len(), 1);
    assert_eq!(wave.actors[0].kind, SylvesterKind::Balrog);
    assert_eq!(wave.bilaterus.len(), 1);
    assert!(wave.actors[0].id < wave.bilaterus[0].id);
    assert_eq!(wave.countdown, 3000);
    assert!(wave.battle_active);
    assert!(matches!(
        wave.plan,
        WavePlan::CyclingTank4Finale { wave_count: 7, .. }
    ));
}

#[test]
fn current_tank_four_finale_special_hatch_reload_enters_selected_then_fixed_tank_five_roster() {
    use turbofish_deluxe::{fish_pet::FishPetKind, sim::Event};

    // Controlled shop fixture: three actual BuyEgg actions exercise the
    // special phase transaction, without claiming earned native progress.
    let mut uninterrupted = tank_four_finale_session(&[PetKind::Angie]);
    let pets_before = uninterrupted.progress.unlocked_pets.clone();
    let shells_before = uninterrupted.progress.shell_balance;
    let board = uninterrupted.board.as_mut().unwrap();
    board.balance = 299_997;
    board.breeder_unlocked = true;
    board.upgrades.quality_unlocked = true;
    board.upgrades.quantity_unlocked = true;
    board.oscar_unlocked = true;
    board.ultra_unlocked = true;
    board.weapon_unlocked = true;
    board.egg_unlocked = true;
    uninterrupted.validate().unwrap();
    let events = uninterrupted.apply_actions(&[
        Action::BuyEgg,
        Action::BuyEgg,
        Action::BuyEgg,
        Action::BuyEgg,
    ]);
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, Event::EggBought { .. }))
            .count(),
        3
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, Event::TankFourFinaleHatchStarted { .. }))
            .count(),
        1
    );
    assert!(!events.iter().any(|event| matches!(
        event,
        Event::PetUnlocked { .. } | Event::HatchStarted { .. }
    )));
    assert_eq!(
        (uninterrupted.progress.tank, uninterrupted.progress.level),
        (5, 1)
    );
    assert_eq!(uninterrupted.progress.unlocked_pets, pets_before);
    assert_eq!(uninterrupted.progress.unlocked_pets.len(), 19);
    assert_eq!(uninterrupted.progress.shell_balance, shells_before);
    assert!(uninterrupted.board.is_none());
    assert!(matches!(
        uninterrupted.phase,
        AdventurePhase::TankFourFinaleHatch { updates: 0 }
    ));
    let mut reopened = cli::decode_save(&current_bytes(uninterrupted.clone())).unwrap();
    assert_eq!(
        serde_json::to_value(&reopened).unwrap(),
        serde_json::to_value(&uninterrupted).unwrap()
    );
    for _ in 0..171 {
        assert_eq!(
            serde_json::to_value(reopened.step(&[])).unwrap(),
            serde_json::to_value(uninterrupted.step(&[])).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&reopened).unwrap(),
            serde_json::to_value(&uninterrupted).unwrap()
        );
    }
    assert!(matches!(
        reopened.phase,
        AdventurePhase::TankFourFinaleHatch { updates: 171 }
    ));
    let mut reopened = cli::decode_save(&current_bytes(reopened)).unwrap();
    let entered = reopened.apply_actions(&[Action::Continue]);
    assert!(entered.iter().any(|event| matches!(
        event,
        Event::StageStarted {
            tank: 5,
            level: 1,
            ..
        }
    )));
    assert!(
        !entered
            .iter()
            .any(|event| matches!(event, Event::PetSelectionOpened { .. }))
    );
    assert_eq!(reopened.phase, AdventurePhase::Playing);
    let board = reopened.board.as_ref().unwrap();
    assert_eq!(
        (
            board.tank,
            board.level,
            board.balance,
            board.eggs,
            board.egg_price
        ),
        (5, 1, 200, 2, 0)
    );
    assert_eq!(board.pets[0], PetKind::Angie);
    assert_eq!(
        &board.pets[1..],
        &[
            PetKind::Stinky,
            PetKind::Niko,
            PetKind::Itchy,
            PetKind::Prego,
            PetKind::Zorf,
            PetKind::Clyde,
            PetKind::Vert,
            PetKind::Rufus,
            PetKind::Meryl,
            PetKind::Wadsworth,
            PetKind::Seymour,
            PetKind::Shrapnel,
            PetKind::Gumbo,
            PetKind::Blip,
            PetKind::Rhubarb,
            PetKind::Nimbus,
            PetKind::Amp,
            PetKind::Gash,
        ]
    );
    let angies: Vec<_> = board
        .fish_pets
        .iter()
        .filter(|pet| pet.kind == FishPetKind::Angie)
        .collect();
    assert_eq!(angies.len(), 1);
    assert!(angies[0].id < board.stinky[0].combat_id.unwrap());
    assert!(board.fish.is_empty());
    assert_eq!(reopened.progress.selected_pets, vec![PetKind::Angie]);
    let final_reload = cli::decode_save(&current_bytes(reopened.clone())).unwrap();
    assert_eq!(
        serde_json::to_value(final_reload).unwrap(),
        serde_json::to_value(reopened).unwrap()
    );
}

#[test]
fn current_tank_five_live_boss_child_and_pause_reopen_continues_identically() {
    use turbofish_deluxe::alien::WeakSylvester;

    let mut uninterrupted = prepared_tank_five_boss_session(2);
    let mut encoded = serde_json::to_value(&uninterrupted).unwrap();
    let child_id = encoded["board"]["next_id"].as_u64().unwrap();
    encoded["board"]["next_id"] = (child_id + 1).into();
    uninterrupted = serde_json::from_value(encoded).unwrap();
    uninterrupted
        .board
        .as_mut()
        .unwrap()
        .invasion
        .as_mut()
        .unwrap()
        .finale
        .as_mut()
        .unwrap()
        .children
        .push(WeakSylvester::spawn_mini(child_id, 300, 230, 1, 3, 7));
    uninterrupted.validate().unwrap();
    let mut reopened = cli::decode_save(&current_bytes(uninterrupted.clone())).unwrap();
    assert_eq!(
        serde_json::to_value(&reopened).unwrap(),
        serde_json::to_value(&uninterrupted).unwrap()
    );
    let board_before = serde_json::to_value(reopened.board.as_ref().unwrap()).unwrap();
    let time_before = reopened.ticks;
    for _ in 0..6 {
        reopened.paused_step();
        uninterrupted.paused_step();
    }
    assert_eq!(reopened.ticks, time_before + 6);
    assert_eq!(
        serde_json::to_value(reopened.board.as_ref().unwrap()).unwrap(),
        board_before
    );
    reopened = cli::decode_save(&current_bytes(reopened)).unwrap();
    for _ in 0..8 {
        assert_eq!(
            serde_json::to_value(reopened.step(&[])).unwrap(),
            serde_json::to_value(uninterrupted.step(&[])).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&reopened).unwrap(),
            serde_json::to_value(&uninterrupted).unwrap()
        );
        reopened.validate().unwrap();
    }
}

#[test]
fn current_tank_five_live_ulysses_launches_energyball_then_reopens_mid_flight() {
    use turbofish_deluxe::{
        alien::{SylvesterKind, WeakSylvester},
        missile::MissileKind,
        sim::Event,
    };

    let mut uninterrupted = prepared_tank_five_boss_session(0);
    let mut encoded = serde_json::to_value(&uninterrupted).unwrap();
    let alien_id = encoded["board"]["next_id"].as_u64().unwrap();
    encoded["board"]["next_id"] = (alien_id + 1).into();
    uninterrupted = serde_json::from_value(encoded).unwrap();
    let wave = uninterrupted
        .board
        .as_mut()
        .unwrap()
        .invasion
        .as_mut()
        .unwrap();
    let mut ulysses = WeakSylvester::spawn_kind(SylvesterKind::Ulysses, alien_id, 100, 120, 1, 1);
    ulysses.spawn_ticks = 0;
    ulysses.launch_ticks = ulysses.reload_ticks;
    wave.actors.push(ulysses);
    uninterrupted.validate().unwrap();

    // The production actor update chooses a live pet and allocates the ball.
    let events = uninterrupted.step(&[]);
    let launched = events
        .iter()
        .find_map(|event| match event {
            Event::EnergyBallLaunched {
                missile_id,
                target_id,
                ..
            } => Some((*missile_id, *target_id)),
            _ => None,
        })
        .expect("live Ulysses should launch toward a Tank 5 pet");
    let board = uninterrupted.board.as_ref().unwrap();
    assert!(board.fish.is_empty());
    let ball = board
        .missiles
        .iter()
        .find(|ball| ball.id == launched.0)
        .unwrap();
    assert_eq!(ball.kind, MissileKind::EnergyBall);
    assert_eq!(ball.target_id, launched.1);
    assert!(
        board.fish_pets.iter().any(|pet| pet.id == launched.1)
            || board.stinky.first().and_then(|pet| pet.combat_id) == Some(launched.1)
            || board
                .niko
                .first()
                .is_some_and(|pet| pet.owner_id == launched.1)
            || board.clyde.first().is_some_and(|pet| pet.id == launched.1)
            || board.rufus.first().is_some_and(|pet| pet.id == launched.1)
            || board
                .rhubarb
                .first()
                .is_some_and(|pet| pet.id == launched.1)
    );
    uninterrupted.validate().unwrap();

    let mut reopened = cli::decode_save(&current_bytes(uninterrupted.clone())).unwrap();
    let board_before = serde_json::to_value(reopened.board.as_ref().unwrap()).unwrap();
    for _ in 0..4 {
        reopened.paused_step();
        uninterrupted.paused_step();
    }
    assert_eq!(
        serde_json::to_value(reopened.board.as_ref().unwrap()).unwrap(),
        board_before
    );
    reopened = cli::decode_save(&current_bytes(reopened)).unwrap();
    for _ in 0..12 {
        assert_eq!(
            serde_json::to_value(reopened.step(&[])).unwrap(),
            serde_json::to_value(uninterrupted.step(&[])).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&reopened).unwrap(),
            serde_json::to_value(&uninterrupted).unwrap()
        );
        reopened.validate().unwrap();
    }
}

#[test]
fn current_tank_five_rejects_missing_or_dangling_finale_state() {
    use turbofish_deluxe::alien::WeakSylvester;

    let session = prepared_tank_five_boss_session(2);
    let complete = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session,
    })
    .unwrap();
    cli::decode_save(&serde_json::to_vec(&complete).unwrap()).unwrap();
    for path in [
        "/session/progress/cyrax_attempts",
        "/session/progress/adventure_completed",
        "/session/board/invasion/finale",
        "/session/board/invasion/finale/boss_defeated",
        "/session/board/invasion/finale/children",
        "/session/board/invasion/finale/profile_attempts",
        "/session/board/invasion/finale/ordinary_ticks",
        "/session/board/invasion/finale/child_ticks",
        "/session/board/stinky/0/combat_id",
    ] {
        let mut missing = complete.clone();
        let (parent, key) = path.rsplit_once('/').unwrap();
        missing
            .pointer_mut(parent)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(key);
        assert!(
            cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err(),
            "accepted absent {path}"
        );
    }
    let mut disagreement = complete.clone();
    disagreement["session"]["progress"]["cyrax_attempts"] = 1.into();
    assert!(cli::decode_save(&serde_json::to_vec(&disagreement).unwrap()).is_err());
    let mut duplicate = complete.clone();
    let boss_id = duplicate["session"]["board"]["invasion"]["finale"]["boss"]["id"]
        .as_u64()
        .unwrap();
    duplicate["session"]["board"]["invasion"]["finale"]["children"] =
        serde_json::json!([WeakSylvester::spawn_mini(boss_id, 300, 230, 1, 3, 7)]);
    assert!(cli::decode_save(&serde_json::to_vec(&duplicate).unwrap()).is_err());
    let mut wrong_shape = complete;
    wrong_shape["session"]["board"]["invasion"]["finale"]["children"] = serde_json::json!({});
    assert!(cli::decode_save(&serde_json::to_vec(&wrong_shape).unwrap()).is_err());
}

#[test]
fn current_tank_five_boss_death_latch_reopens_before_next_shop_refresh() {
    use turbofish_deluxe::sim::{Event, Rejection};

    // Prepared state immediately after boss removal, before Board refresh.
    let mut session = prepared_tank_five_boss_session(0);
    let board = session.board.as_mut().unwrap();
    let finale = board.invasion.as_mut().unwrap().finale.as_mut().unwrap();
    finale.boss = None;
    finale.boss_defeated = true;
    board.invasion.as_mut().unwrap().battle_active = false;
    assert!(!board.egg_unlocked);
    session.validate().unwrap();
    let mut reopened = cli::decode_save(&current_bytes(session)).unwrap();
    let rejected = reopened.apply_actions(&[Action::BuyEgg]);
    assert!(rejected.iter().any(|event| matches!(
        event,
        Event::Rejected {
            reason: Rejection::Locked,
            ..
        }
    )));
    assert_eq!(reopened.board.as_ref().unwrap().eggs, 2);
    reopened.step(&[]);
    assert!(reopened.board.as_ref().unwrap().egg_unlocked);
    let mut reopened = cli::decode_save(&current_bytes(reopened)).unwrap();
    let bought = reopened.apply_actions(&[Action::BuyEgg]);
    assert_eq!(
        bought
            .iter()
            .filter(|event| matches!(event, Event::EggBought { .. }))
            .count(),
        1
    );
    assert_eq!((reopened.progress.tank, reopened.progress.level), (5, 2));
    assert!(reopened.progress.adventure_completed);
    assert_eq!(
        reopened.progress.unlocked_pets.last(),
        Some(&PetKind::Presto)
    );
    let awarded = reopened.progress.shell_balance;
    let mut reopened = cli::decode_save(&current_bytes(reopened)).unwrap();
    assert_eq!(reopened.progress.shell_balance, awarded);
    assert!(
        reopened
            .apply_actions(&[Action::BuyEgg])
            .iter()
            .any(|event| matches!(
                event,
                Event::Rejected {
                    reason: Rejection::Locked,
                    ..
                }
            ))
    );
    for _ in 0..171 {
        reopened.step(&[]);
    }
    reopened.apply_actions(&[Action::Continue]);
    assert_eq!(reopened.phase, AdventurePhase::AdventureFinaleInterlude);
    assert!(reopened.board.is_none());
    cli::decode_save(&current_bytes(reopened)).unwrap();
}

#[test]
fn current_tank_five_loss_reopen_preserves_strict_attempt_and_fresh_retry() {
    use turbofish_deluxe::sim::Event;

    for damage in [1000.0, 1001.0] {
        let mut session = prepared_tank_five_boss_session(0);
        let board = session.board.as_mut().unwrap();
        let boss = board
            .invasion
            .as_mut()
            .unwrap()
            .finale
            .as_mut()
            .unwrap()
            .boss
            .as_mut()
            .unwrap();
        boss.health -= damage;
        board.stinky.clear();
        board.niko.clear();
        board.clyde.clear();
        board.rufus.clear();
        board.rhubarb.clear();
        board.fish_pets.clear();
        let failed_board_tick = board.tick;
        let events = session.step(&[]);
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Event::GameOverStarted { .. }))
        );
        assert!(matches!(session.phase, AdventurePhase::GameOver { .. }));
        assert_eq!(session.board.as_ref().unwrap().tick, failed_board_tick + 1);
        assert_eq!(session.progress.cyrax_attempts, u32::from(damage > 1000.0));
        let mut reopened = cli::decode_save(&current_bytes(session)).unwrap();
        assert!(matches!(reopened.phase, AdventurePhase::GameOver { .. }));
        let failed = reopened.board.as_ref().unwrap();
        assert!(
            failed.stinky.is_empty()
                && failed.niko.is_empty()
                && failed.clyde.is_empty()
                && failed.rufus.is_empty()
                && failed.rhubarb.is_empty()
                && failed.fish_pets.is_empty()
        );
        for _ in 0..31 {
            reopened.step(&[]);
        }
        reopened.apply_actions(&[Action::Continue]);
        assert_eq!(reopened.phase, AdventurePhase::GameSelector);
        reopened.apply_actions(&[Action::PlayAdventure]);
        reopened.apply_actions(&[Action::Continue]);
        let retry = reopened.board.as_ref().unwrap();
        assert_eq!((retry.tank, retry.level, retry.tick), (5, 1, 0));
        assert!(
            !retry.stinky.is_empty()
                && !retry.niko.is_empty()
                && !retry.clyde.is_empty()
                && !retry.rufus.is_empty()
                && !retry.rhubarb.is_empty()
        );
        let angies: Vec<_> = retry
            .fish_pets
            .iter()
            .filter(|pet| pet.kind == turbofish_deluxe::fish_pet::FishPetKind::Angie)
            .collect();
        assert_eq!(retry.fish_pets.len(), 14);
        assert_eq!(angies.len(), 1);
        assert!(angies[0].id < retry.stinky[0].combat_id.unwrap());
        assert_eq!(reopened.progress.selected_pets, vec![PetKind::Angie]);
        assert_eq!(
            retry
                .invasion
                .as_ref()
                .unwrap()
                .finale
                .as_ref()
                .unwrap()
                .profile_attempts,
            u32::from(damage > 1000.0)
        );
        cli::decode_save(&current_bytes(reopened)).unwrap();
    }
}

#[test]
fn current_tank_five_retired_niko_pearl_reopens_and_credits_once() {
    use turbofish_deluxe::{niko::NikoPearl, sim::Event};

    let mut session = prepared_tank_five_session(0);
    let owner_id = session
        .board
        .as_ref()
        .unwrap()
        .niko
        .first()
        .unwrap()
        .owner_id;
    let mut encoded = serde_json::to_value(&session).unwrap();
    let pearl_id = encoded["board"]["next_id"].as_u64().unwrap();
    encoded["board"]["next_id"] = (pearl_id + 1).into();
    session = serde_json::from_value(encoded).unwrap();
    let board = session.board.as_mut().unwrap();
    board.niko.clear();
    board
        .pearls
        .push(NikoPearl::spawn(pearl_id, owner_id, 96, 251));
    session.validate().unwrap();
    let mut reopened = cli::decode_save(&current_bytes(session)).unwrap();
    let balance_before = reopened.board.as_ref().unwrap().balance;
    let clicked = reopened.apply_actions(&[Action::Click { x: 100.0, y: 255.0 }]);
    assert!(clicked.iter().any(|event| matches!(event,
        Event::PearlCollectionStarted { pearl_id: id, .. } if *id == pearl_id)));
    reopened = cli::decode_save(&current_bytes(reopened)).unwrap();
    let mut credits = 0;
    for _ in 0..50 {
        credits += reopened
            .step(&[])
            .iter()
            .filter(|event| {
                matches!(event,
            Event::PearlCredited { pearl_id: id, owner_id: owner, amount: 250, .. }
                if *id == pearl_id && *owner == owner_id)
            })
            .count();
    }
    assert_eq!(credits, 1);
    assert_eq!(
        reopened.board.as_ref().unwrap().balance,
        balance_before + 250
    );
    assert!(reopened.board.as_ref().unwrap().pearls.is_empty());
    cli::decode_save(&current_bytes(reopened)).unwrap();
}

#[test]
fn current_tank_four_accepts_fifteen_pet_rosters_and_rejects_unearned_nimbus() {
    let canonical = tank_four_first_session(&[]).progress.unlocked_pets;
    assert_eq!(canonical.len(), 15);
    for pet in canonical {
        let session = tank_four_first_session(&[pet]);
        session.validate().unwrap();
        let bytes = serde_json::to_vec(&cli::ProjectSave {
            format_version: cli::SAVE_FORMAT_VERSION,
            session: session.clone(),
        })
        .unwrap();
        assert_eq!(
            serde_json::to_value(cli::decode_save(&bytes).unwrap()).unwrap(),
            serde_json::to_value(session).unwrap()
        );
    }
    assert!(AdventureState::new_tank4_first_stage(42, &[PetKind::Nimbus]).is_err());
}

#[test]
fn current_tank_four_requires_all_breeder_and_rhubarb_state_without_backfill() {
    let current = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: tank_four_first_session(&[PetKind::Rhubarb]),
    })
    .unwrap();
    for field in ["breeder_unlocked", "breeders", "dead_breeders", "rhubarb"] {
        let mut missing = current.clone();
        missing["session"]["board"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err(),
            "missing {field}"
        );
    }
    for field in [
        "birth_clock",
        "birth_threshold",
        "food_points",
        "bought_timer",
    ] {
        let mut missing = current.clone();
        missing["session"]["board"]["breeders"][0]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err(),
            "missing Breeder {field}"
        );
    }
    for field in ["specialty_ticks", "chase_timer", "movement_animation_timer"] {
        let mut missing = current.clone();
        missing["session"]["board"]["rhubarb"][0]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err(),
            "missing Rhubarb {field}"
        );
    }
}

#[test]
fn current_tank_four_pending_birth_and_rhubarb_push_continue_identically_after_reload() {
    use turbofish_deluxe::{breeder::BreederSize, sim::Event};
    // Controlled current-format boundary, not native earning evidence.
    let mut uninterrupted = tank_four_first_session(&[PetKind::Rhubarb]);
    let board = uninterrupted.board.as_mut().unwrap();
    let rhubarb = board.rhubarb.first_mut().unwrap();
    rhubarb.specialty_ticks = 5;
    let breeder = &mut board.breeders[0];
    breeder.size = BreederSize::Medium;
    breeder.food_points = 0;
    breeder.food_needed_to_grow = 7;
    breeder.birth_clock = 999;
    breeder.birth_threshold = 1000;
    breeder.x = rhubarb.x + 5.0;
    breeder.y = 300.0;
    breeder.widget_x = breeder.x as i32;
    breeder.widget_y = 300;
    breeder.hunger = 300;
    board.breeder_unlocked = true;
    uninterrupted.validate().unwrap();
    let bytes = serde_json::to_vec(&cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: uninterrupted.clone(),
    })
    .unwrap();
    let mut resumed = cli::decode_save(&bytes).unwrap();
    let mut birth_count = 0;
    let mut push_count = 0;
    for _ in 0..120 {
        let expected = uninterrupted.step(&[]);
        let actual = resumed.step(&[]);
        birth_count += actual
            .iter()
            .filter(|event| matches!(event, Event::BreederBornGuppy { .. }))
            .count();
        push_count += actual
            .iter()
            .filter(|event| matches!(event, Event::RhubarbPushed { .. }))
            .count();
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::to_value(expected).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&resumed).unwrap(),
            serde_json::to_value(&uninterrupted).unwrap()
        );
        resumed.validate().unwrap();
    }
    assert_eq!(birth_count, 1);
    assert!(push_count >= 1);
    assert!(!resumed.board.as_ref().unwrap().fish.is_empty());
}

#[test]
fn format_sixteen_nimbus_hatch_enters_current_four_two_and_old_amp_selector_still_decodes() {
    use turbofish_deluxe::sim::Event;
    let mut session = tank_four_first_session(&[PetKind::Rhubarb]);
    session.progress.level = 2;
    session.progress.unlocked_pets.push(PetKind::Nimbus);
    session.board = None;
    session.phase = AdventurePhase::Hatch {
        pet: PetKind::Nimbus,
        updates: 171,
    };
    session.validate().unwrap();
    // The earned A31 endpoint has no Board. It loads byte-for-byte as a
    // format-sixteen session; opening 4-2 constructs the current live state.
    let bytes = serde_json::to_vec(&historical_envelope(cli::ProjectSave {
        format_version: 16,
        session: session.clone(),
    }))
    .unwrap();
    let mut loaded = cli::decode_save(&bytes).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(session).unwrap()
    );
    loaded.apply_actions(&[
        Action::Continue,
        Action::TogglePet {
            pet: PetKind::Nimbus,
        },
    ]);
    assert_eq!(loaded.progress.unlocked_pets.len(), 16);
    assert_eq!(loaded.progress.selected_pets, vec![PetKind::Rhubarb]);
    assert!(matches!(loaded.phase, AdventurePhase::PetSelection { .. }));
    assert!(
        matches!(&loaded.phase, AdventurePhase::PetSelection { selected } if selected.as_slice() == [PetKind::Nimbus])
    );
    assert!(
        loaded
            .apply_actions(&[Action::Continue])
            .iter()
            .any(|event| matches!(event, Event::PetSelectionConfirmation { .. }))
    );
    let bytes = current_bytes(loaded.clone());
    let mut resumed = cli::decode_save(&bytes).unwrap();
    let expected = loaded.apply_actions(&[Action::ConfirmPetSelection { accept: true }]);
    let actual = resumed.apply_actions(&[Action::ConfirmPetSelection { accept: true }]);
    assert_eq!(
        serde_json::to_value(actual).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&resumed).unwrap(),
        serde_json::to_value(&loaded).unwrap()
    );
    assert!(matches!(resumed.phase, AdventurePhase::Playing));
    assert_eq!(
        (
            resumed.board.as_ref().unwrap().tank,
            resumed.board.as_ref().unwrap().level
        ),
        (4, 2)
    );
    for _ in 0..30 {
        assert_eq!(
            serde_json::to_value(resumed.step(&[])).unwrap(),
            serde_json::to_value(loaded.step(&[])).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&resumed).unwrap(),
            serde_json::to_value(&loaded).unwrap()
        );
    }

    // A historical format-seventeen 4-3 endpoint has no live Board, so it
    // does not require format-eighteen Amp actor fields to enter the new stage.
    let mut old_selector = tank_four_second_session(&[]);
    old_selector.progress.level = 3;
    old_selector.progress.unlocked_pets.push(PetKind::Amp);
    old_selector.board = None;
    old_selector.phase = AdventurePhase::Hatch {
        pet: PetKind::Amp,
        updates: 171,
    };
    old_selector.validate().unwrap();
    let old_bytes = serde_json::to_vec(&historical_envelope(cli::ProjectSave {
        format_version: 17,
        session: old_selector.clone(),
    }))
    .unwrap();
    let mut old_selector = cli::decode_save(&old_bytes).unwrap();
    old_selector.apply_actions(&[Action::Continue, Action::TogglePet { pet: PetKind::Amp }]);
    assert!(matches!(
        old_selector.phase,
        AdventurePhase::PetSelection { .. }
    ));
    assert!(old_selector.board.is_none());
    let selector_bytes = serde_json::to_vec(&historical_envelope(cli::ProjectSave {
        format_version: 17,
        session: old_selector.clone(),
    }))
    .unwrap();
    let mut resumed = cli::decode_save(&selector_bytes).unwrap();
    assert_eq!(
        serde_json::to_value(&resumed).unwrap(),
        serde_json::to_value(&old_selector).unwrap()
    );
    resumed.apply_actions(&[Action::Continue]);
    resumed.apply_actions(&[Action::ConfirmPetSelection { accept: true }]);
    assert_eq!((resumed.progress.tank, resumed.progress.level), (4, 3));
    assert!(matches!(resumed.phase, AdventurePhase::Playing));
    assert_eq!(resumed.board.as_ref().unwrap().pets, vec![PetKind::Amp]);
    resumed.validate().unwrap();
}

#[test]
fn format_eighteen_board_null_gash_selector_enters_current_four_four() {
    // A synthetic historical reward endpoint tests the old loader boundary;
    // it is not evidence that three 4-3 eggs were earned in native play.
    let mut old_selector = tank_four_third_session(&[]);
    old_selector.progress.level = 4;
    old_selector.progress.unlocked_pets.push(PetKind::Gash);
    old_selector.board = None;
    old_selector.phase = AdventurePhase::Hatch {
        pet: PetKind::Gash,
        updates: 171,
    };
    old_selector.validate().unwrap();
    let old_bytes = serde_json::to_vec(&historical_envelope(cli::ProjectSave {
        format_version: 18,
        session: old_selector.clone(),
    }))
    .unwrap();
    let mut old_selector = cli::decode_save(&old_bytes).unwrap();
    old_selector.apply_actions(&[Action::Continue, Action::TogglePet { pet: PetKind::Gash }]);
    assert!(matches!(
        old_selector.phase,
        AdventurePhase::PetSelection { .. }
    ));
    assert!(old_selector.board.is_none());
    let selector_bytes = serde_json::to_vec(&historical_envelope(cli::ProjectSave {
        format_version: 18,
        session: old_selector.clone(),
    }))
    .unwrap();
    let mut resumed = cli::decode_save(&selector_bytes).unwrap();
    assert_eq!(
        serde_json::to_value(&resumed).unwrap(),
        serde_json::to_value(&old_selector).unwrap()
    );
    resumed.apply_actions(&[Action::Continue]);
    resumed.apply_actions(&[Action::ConfirmPetSelection { accept: true }]);
    assert_eq!((resumed.progress.tank, resumed.progress.level), (4, 4));
    assert!(matches!(resumed.phase, AdventurePhase::Playing));
    assert_eq!(resumed.board.as_ref().unwrap().pets, vec![PetKind::Gash]);
    resumed.validate().unwrap();
}

#[test]
fn current_tank_four_reloads_long_gumbo_counter_and_invasion_pending_large_birth() {
    use turbofish_deluxe::alien::{SylvesterKind, WeakSylvester};
    // Explicit current-format boundary fixture, independently specified by
    // PB05's dword timer, ordinary growth and registered-alien pause rules.
    let mut fixture = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: tank_four_first_session(&[PetKind::Gumbo]),
    })
    .unwrap();
    let board = &mut fixture["session"]["board"];
    let next_id = board["next_id"].as_u64().unwrap();
    board["next_id"] = (next_id + 1).into();
    board["invasion"]["battle_active"] = true.into();
    board["invasion"]["actors"] = serde_json::to_value(vec![WeakSylvester::spawn_kind(
        SylvesterKind::Balrog,
        next_id,
        450,
        105,
        0,
        0,
    )])
    .unwrap();
    board["breeder_unlocked"] = true.into();
    let breeder = &mut board["breeders"][0];
    breeder["size"] = "Large".into();
    breeder["hunger"] = 550.into();
    breeder["food_points"] = 0.into();
    breeder["food_needed_to_grow"] = 7.into();
    breeder["birth_clock"] = 1398.into();
    breeder["birth_threshold"] = 500.into();
    breeder["steering_timer"] = 342.into();
    breeder["x"] = 200.0.into();
    breeder["y"] = 200.0.into();
    breeder["widget_x"] = 200.into();
    breeder["widget_y"] = 200.into();
    let mut uninterrupted = cli::decode_save(&serde_json::to_vec(&fixture).unwrap()).unwrap();
    let encoded = serde_json::to_vec(&cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: uninterrupted.clone(),
    })
    .unwrap();
    let mut resumed = cli::decode_save(&encoded).unwrap();
    for _ in 0..30 {
        assert_eq!(
            serde_json::to_value(resumed.step(&[])).unwrap(),
            serde_json::to_value(uninterrupted.step(&[])).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&resumed).unwrap(),
            serde_json::to_value(&uninterrupted).unwrap()
        );
        resumed.validate().unwrap();
    }
    let breeder = &resumed.board.as_ref().unwrap().breeders[0];
    assert_eq!(
        (breeder.hunger, breeder.birth_clock, breeder.birth_threshold),
        (550, 1398, 500)
    );
    assert_eq!(breeder.steering_timer, 372);
}

#[test]
fn current_finale_accepts_fourteen_rosters_and_rejects_unearned_rhubarb() {
    // PB52 canonical unlocks: Rhubarb is the reward, not an initial live pet.
    for pet in [
        PetKind::Stinky,
        PetKind::Niko,
        PetKind::Itchy,
        PetKind::Prego,
        PetKind::Zorf,
        PetKind::Clyde,
        PetKind::Vert,
        PetKind::Rufus,
        PetKind::Meryl,
        PetKind::Wadsworth,
        PetKind::Seymour,
        PetKind::Shrapnel,
        PetKind::Gumbo,
        PetKind::Blip,
    ] {
        let session = tank_three_fifth_session(&[pet]);
        session.validate().unwrap();
        let bytes = serde_json::to_vec(&cli::ProjectSave {
            format_version: cli::SAVE_FORMAT_VERSION,
            session: session.clone(),
        })
        .unwrap();
        assert_eq!(
            serde_json::to_value(cli::decode_save(&bytes).unwrap()).unwrap(),
            serde_json::to_value(session).unwrap()
        );
    }
    assert!(AdventureState::new_tank3_fifth_stage(42, &[PetKind::Rhubarb]).is_err());
}

#[test]
fn current_blip_reveal_boundary_and_typed_continuation_survive_reload() {
    use turbofish_deluxe::sim::Event;
    // PB52 ordinary flag0 reveals only after Board update200. Existing flags
    // are the durable authority; there is no duplicate saved reveal boolean.
    let mut uninterrupted = tank_three_fifth_session(&[PetKind::Blip]);
    uninterrupted.ticks = 200;
    uninterrupted.board.as_mut().unwrap().tick = 200;
    uninterrupted.validate().unwrap();
    let bytes = serde_json::to_vec(&cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: uninterrupted.clone(),
    })
    .unwrap();
    let mut resumed = cli::decode_save(&bytes).unwrap();
    let mut reveal_count = 0;
    for _ in 0..120 {
        let expected = uninterrupted.step(&[]);
        let actual = resumed.step(&[]);
        reveal_count += actual
            .iter()
            .filter(|event| matches!(event, Event::BlipShopRevealed { .. }))
            .count();
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::to_value(expected).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&resumed).unwrap(),
            serde_json::to_value(&uninterrupted).unwrap()
        );
        resumed.validate().unwrap();
    }
    assert_eq!(reveal_count, 1);
    assert_eq!(resumed.board.as_ref().unwrap().balance, 200);
    let mut missing_actor = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: resumed,
    })
    .unwrap();
    missing_actor["session"]["board"]["fish_pets"] = serde_json::json!([]);
    assert!(cli::decode_save(&serde_json::to_vec(&missing_actor).unwrap()).is_err());
}

#[test]
fn current_tank_three_bonus_flight_and_immutable_results_survive_reload() {
    use turbofish_deluxe::bonus::{BonusResult, BonusState, ShellPhase};
    let mut session = tank_three_fifth_session(&[PetKind::Blip]);
    session.progress.level = 6;
    session.progress.unlocked_pets.push(PetKind::Rhubarb);
    session.board = None;
    let mut bonus = BonusState::new_for_tank(42, 3).unwrap();
    bonus.click(0.0, 0.0);
    bonus.update();
    let shell = &bonus.shells[0];
    bonus.click(shell.x as f32 + 1.0, shell.y as f32 + 1.0);
    bonus.update();
    assert!(
        bonus
            .shells
            .iter()
            .any(|shell| matches!(&shell.phase, ShellPhase::Collecting { .. }))
    );
    session.ticks = bonus.tick;
    session.phase = AdventurePhase::Bonus { state: bonus };
    session.validate().unwrap();
    let current = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: session.clone(),
    })
    .unwrap();
    let mut resumed = cli::decode_save(&serde_json::to_vec(&current).unwrap()).unwrap();
    for _ in 0..32 {
        assert_eq!(
            serde_json::to_value(resumed.step(&[])).unwrap(),
            serde_json::to_value(session.step(&[])).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&resumed).unwrap(),
            serde_json::to_value(&session).unwrap()
        );
        resumed.validate().unwrap();
    }
    for field in ["origin_tank", "origin_level"] {
        let mut missing = current.clone();
        missing["session"]["phase"]["Bonus"]["state"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err());
    }
    // Result origin stays3 when the profile advances to4-1; balance is already
    // committed and neither presentation nor repeated Continue can credit it.
    session.progress.tank = 4;
    session.progress.level = 1;
    session.progress.shell_balance = 1564;
    session.phase = AdventurePhase::BonusResults {
        result: BonusResult {
            origin_tank: 3,
            origin_level: 6,
            earned: 217,
            previous_balance: 1347,
            updates: 8,
            purchase: PurchaseReceipt::new(0),
        },
    };
    let results = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session,
    })
    .unwrap();
    let mut loaded = cli::decode_save(&serde_json::to_vec(&results).unwrap()).unwrap();
    for _ in 0..40 {
        loaded.step(&[]);
    }
    assert_eq!(loaded.progress.shell_balance, 1564);
    loaded.apply_actions(&[Action::Continue]);
    assert!(matches!(loaded.phase, AdventurePhase::PetSelection { .. }));
    assert!(loaded.board.is_none());
    assert_eq!(loaded.progress.unlocked_pets.len(), 15);
    assert_eq!(loaded.progress.shell_balance, 1564);
    loaded.validate().unwrap();
    for invalid in [0, 1, 2, 4, 255] {
        let mut wrong = results.clone();
        wrong["session"]["phase"]["BonusResults"]["result"]["origin_tank"] = invalid.into();
        assert!(cli::decode_save(&serde_json::to_vec(&wrong).unwrap()).is_err());
    }
    let mut missing = results;
    missing["session"]["phase"]["BonusResults"]["result"]
        .as_object_mut()
        .unwrap()
        .remove("origin_tank");
    assert!(cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err());
}

#[test]
fn current_bonus_save_rejects_empty_wait_with_live_shells() {
    use turbofish_deluxe::bonus::BonusState;

    let mut session = tank_three_fifth_session(&[PetKind::Blip]);
    session.progress.level = 6;
    session.progress.unlocked_pets.push(PetKind::Rhubarb);
    session.board = None;
    let mut bonus = BonusState::new_for_tank(42, 3).unwrap();
    bonus.click(0.0, 0.0);
    while !bonus.timed_out {
        bonus.update();
    }
    assert!(!bonus.shells.is_empty());
    assert_eq!(bonus.empty_updates, 0);
    session.ticks = bonus.tick;
    session.phase = AdventurePhase::Bonus { state: bonus };
    session.validate().unwrap();

    let current = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session,
    })
    .unwrap();
    assert!(cli::decode_save(&serde_json::to_vec(&current).unwrap()).is_ok());
    let mut forged = current;
    forged["session"]["phase"]["Bonus"]["state"]["empty_updates"] = 1.into();
    assert!(cli::decode_save(&serde_json::to_vec(&forged).unwrap()).is_err());
}

#[test]
fn completed_profile_save_requires_replay_board_for_ordinary_bonus_phases() {
    use turbofish_deluxe::bonus::{BonusResult, BonusState};
    use turbofish_deluxe::time_trial::SUPPORTED_PETS;

    let mut first = AdventureSession::new(42);
    first.progress.tank = 1;
    first.progress.level = 6;
    first.progress.unlocked_pets = vec![
        PetKind::Stinky,
        PetKind::Niko,
        PetKind::Itchy,
        PetKind::Prego,
        PetKind::Zorf,
    ];
    first.board = None;
    first.phase = AdventurePhase::Bonus {
        state: BonusState::new_for_tank(42, 1).unwrap(),
    };
    assert!(cli::decode_save(&current_bytes(first.clone())).is_ok());
    first.progress.tank = 2;
    first.progress.level = 1;
    first.phase = AdventurePhase::BonusResults {
        result: BonusResult {
            origin_tank: 1,
            origin_level: 6,
            earned: 0,
            previous_balance: 0,
            updates: 0,
            purchase: PurchaseReceipt::new(0),
        },
    };
    assert!(cli::decode_save(&current_bytes(first)).is_ok());

    let mut completed = AdventureSession::new(43);
    completed.progress.tank = 1;
    completed.progress.level = 6;
    completed.progress.unlocked_pets = SUPPORTED_PETS
        .into_iter()
        .chain([PetKind::Presto])
        .collect();
    completed.progress.adventure_completed = true;
    completed.progress.adventure_completions = 2;
    completed.board = None;
    completed.phase = AdventurePhase::Bonus {
        state: BonusState::new_for_tank(42, 1).unwrap(),
    };
    assert!(cli::decode_save(&current_bytes(completed.clone())).is_err());

    completed.progress.tank = 2;
    completed.progress.level = 1;
    completed.phase = AdventurePhase::BonusResults {
        result: BonusResult {
            origin_tank: 1,
            origin_level: 6,
            earned: 0,
            previous_balance: 0,
            updates: 0,
            purchase: PurchaseReceipt::new(0),
        },
    };
    assert!(cli::decode_save(&current_bytes(completed.clone())).is_err());

    completed.progress.tank = 5;
    completed.progress.level = 2;
    completed.progress.shell_balance = 5000;
    completed.phase = AdventurePhase::BonusResults {
        result: BonusResult {
            origin_tank: 5,
            origin_level: 1,
            earned: 5000,
            previous_balance: 0,
            updates: 0,
            purchase: PurchaseReceipt::new(0),
        },
    };
    assert!(cli::decode_save(&current_bytes(completed)).is_ok());
}

#[test]
fn current_finale_retains_live_ulysses_and_balls_independently_of_next_psychosquid() {
    use turbofish_deluxe::{alien::SylvesterKind, invasion::WavePlan};
    // Controlled mixed-stage continuation; the existing raw1 state came from
    // the same identified actor constructor, not from the future wave kind.
    let mut session = ulysses_energyball_session(0);
    session.progress.level = 5;
    session.progress.unlocked_pets.push(PetKind::Blip);
    let board = session.board.as_mut().unwrap();
    board.level = 5;
    board.egg_price = 15000;
    board.invasion.as_mut().unwrap().plan = WavePlan::CyclingTank3Finale {
        next: SylvesterKind::Psychosquid,
    };
    session.validate().unwrap();
    let current = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: session.clone(),
    })
    .unwrap();
    let mut resumed = cli::decode_save(&serde_json::to_vec(&current).unwrap()).unwrap();
    for _ in 0..60 {
        assert_eq!(
            serde_json::to_value(resumed.step(&[])).unwrap(),
            serde_json::to_value(session.step(&[])).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&resumed).unwrap(),
            serde_json::to_value(&session).unwrap()
        );
        resumed.validate().unwrap();
    }
    let mut missing = current;
    missing["session"]["board"]["missiles"][0]
        .as_object_mut()
        .unwrap()
        .remove("kind");
    assert!(cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err());
}

#[test]
fn current_blip_hunger_icon_is_independent_of_growth_food_progress() {
    // PB53 and its writer adjudication: ordinary daily-fed count stays0.
    // Growth's distinct mFoodAte can be2/3/large and must not suppress an icon.
    let mut board = AdventureState::new_tank3_fifth_stage(42, &[PetKind::Blip]).unwrap();
    board.fish[0].hunger = 499;
    for growth_count in [2, 3, 25] {
        board.fish[0].food_ate = growth_count;
        assert!(
            board.blip_hunger_icon_visible(&board.fish[0]),
            "growth_count={growth_count}"
        );
    }
    board.fish[0].hunger = 500;
    assert!(!board.blip_hunger_icon_visible(&board.fish[0]));
}

#[test]
fn current_finale_rejects_classic_projectile_with_psychosquid_or_alien_free_tail() {
    use turbofish_deluxe::{
        alien::{SylvesterKind, WeakSylvester},
        invasion::WavePlan,
        missile::ClassicMissile,
    };
    let mut rejections = Vec::new();
    for with_psychosquid in [false, true] {
        let mut session = ulysses_energyball_session(0);
        session.progress.level = 5;
        session.progress.unlocked_pets.push(PetKind::Blip);
        let board = session.board.as_mut().unwrap();
        board.level = 5;
        board.egg_price = 15000;
        let original = board.missiles[0].clone();
        let wave = board.invasion.as_mut().unwrap();
        wave.plan = WavePlan::CyclingTank3Finale {
            next: SylvesterKind::Psychosquid,
        };
        let alien_id = wave.actors[0].id;
        wave.actors.clear();
        if with_psychosquid {
            let mut actor =
                WeakSylvester::spawn_kind(SylvesterKind::Psychosquid, alien_id, 460, 210, 1, 1);
            actor.spawn_ticks = 0;
            wave.actors.push(actor);
        } else {
            // A raw1-only tail is a valid starting control; neither branch
            // justifies Classic, which neither finale species can launch.
            session.validate().unwrap();
        }
        let board = session.board.as_mut().unwrap();
        board.missiles[0] = ClassicMissile::launch(original.id, original.target_id, 100, 110, 3);
        let invalid = cli::ProjectSave {
            format_version: cli::SAVE_FORMAT_VERSION,
            session,
        };
        rejections.push((
            with_psychosquid,
            cli::decode_save(&serde_json::to_vec(&invalid).unwrap()).is_err(),
        ));
    }
    assert!(
        rejections.iter().all(|(_, rejected)| *rejected),
        "(with_psychosquid, rejected)={rejections:?}"
    );
}

#[test]
fn current_finale_rejects_energy_projectile_with_live_psychosquid() {
    use turbofish_deluxe::{
        alien::{SylvesterKind, WeakSylvester},
        invasion::WavePlan,
    };
    let mut session = ulysses_energyball_session(0);
    session.progress.level = 5;
    session.progress.unlocked_pets.push(PetKind::Blip);
    let board = session.board.as_mut().unwrap();
    board.level = 5;
    board.egg_price = 15000;
    let wave = board.invasion.as_mut().unwrap();
    wave.plan = WavePlan::CyclingTank3Finale {
        next: SylvesterKind::Psychosquid,
    };
    let alien_id = wave.actors[0].id;
    wave.actors.clear();
    session.validate().unwrap(); // The orphan energy ball blocks the next wave.
    let mut actor = WeakSylvester::spawn_kind(SylvesterKind::Psychosquid, alien_id, 460, 210, 1, 1);
    actor.spawn_ticks = 0;
    session
        .board
        .as_mut()
        .unwrap()
        .invasion
        .as_mut()
        .unwrap()
        .actors
        .push(actor);
    let invalid = cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session,
    };
    assert!(cli::decode_save(&serde_json::to_vec(&invalid).unwrap()).is_err());
}

#[test]
fn current_fourth_tank_three_accepts_thirteen_rosters_but_rejects_unearned_blip() {
    for pet in [
        PetKind::Stinky,
        PetKind::Niko,
        PetKind::Itchy,
        PetKind::Prego,
        PetKind::Zorf,
        PetKind::Clyde,
        PetKind::Vert,
        PetKind::Rufus,
        PetKind::Meryl,
        PetKind::Wadsworth,
        PetKind::Seymour,
        PetKind::Shrapnel,
        PetKind::Gumbo,
    ] {
        let session = tank_three_fourth_session(&[pet]);
        session.validate().unwrap();
        let bytes = serde_json::to_vec(&cli::ProjectSave {
            format_version: cli::SAVE_FORMAT_VERSION,
            session: session.clone(),
        })
        .unwrap();
        assert_eq!(
            serde_json::to_value(cli::decode_save(&bytes).unwrap()).unwrap(),
            serde_json::to_value(session).unwrap()
        );
    }
    assert!(AdventureState::new_tank3_fourth_stage(42, &[PetKind::Blip]).is_err());
}

#[test]
fn current_energyball_requires_kind_reflection_and_rejects_invalid_reservations() {
    let current = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: ulysses_energyball_session(0),
    })
    .unwrap();
    for field in [
        "kind",
        "reflected",
        "target_id",
        "immunity_ticks",
        "vx",
        "vy",
    ] {
        let mut missing = current.clone();
        missing
            .pointer_mut("/session/board/missiles/0")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err(),
            "accepted missing energyball {field}"
        );
    }
    let mut dangling = current.clone();
    dangling["session"]["board"]["missiles"][0]["target_id"] = 99999_u64.into();
    assert!(cli::decode_save(&serde_json::to_vec(&dangling).unwrap()).is_err());
    let mut wrong_target_class = current.clone();
    wrong_target_class["session"]["board"]["missiles"][0]["target_id"] =
        wrong_target_class["session"]["board"]["invasion"]["actors"][0]["id"].clone();
    assert!(cli::decode_save(&serde_json::to_vec(&wrong_target_class).unwrap()).is_err());
    let mut classic_reflected = current.clone();
    classic_reflected["session"]["board"]["missiles"][0]["kind"] = "Classic".into();
    classic_reflected["session"]["board"]["missiles"][0]["reflected"] = true.into();
    assert!(cli::decode_save(&serde_json::to_vec(&classic_reflected).unwrap()).is_err());
    let mut duplicate = current;
    let next_id = duplicate["session"]["board"]["next_id"].as_u64().unwrap();
    duplicate["session"]["board"]["next_id"] = (next_id + 1).into();
    let mut second = duplicate["session"]["board"]["missiles"][0].clone();
    second["id"] = next_id.into();
    duplicate["session"]["board"]["missiles"]
        .as_array_mut()
        .unwrap()
        .push(second);
    assert!(cli::decode_save(&serde_json::to_vec(&duplicate).unwrap()).is_err());
}

#[test]
fn current_immune_and_redirected_energyballs_resume_effects_and_actor_clocks() {
    // PB48: an interior shot is accepted even with immunity1; immunity0
    // reflects while retaining identity/target. The accepted Shot2 persists.
    for immunity_ticks in [1, 0] {
        let mut uninterrupted = ulysses_energyball_session(immunity_ticks);
        let original = uninterrupted.board.as_ref().unwrap().missiles[0].clone();
        uninterrupted.apply_actions(&[Action::Click { x: 140.0, y: 150.0 }]);
        let board = uninterrupted.board.as_ref().unwrap();
        assert_eq!(board.missiles[0].id, original.id);
        assert_eq!(board.missiles[0].target_id, original.target_id);
        assert_eq!(board.missiles[0].reflected, immunity_ticks == 0);
        if immunity_ticks == 0 {
            assert_eq!((board.missiles[0].vx, board.missiles[0].vy), (1.0, 1.0));
        }
        assert_eq!(board.bomb_shots.len(), 1);
        assert_eq!(board.bomb_shots[0].shot_type, 2);
        let bytes = serde_json::to_vec(&cli::ProjectSave {
            format_version: cli::SAVE_FORMAT_VERSION,
            session: uninterrupted.clone(),
        })
        .unwrap();
        let mut resumed = cli::decode_save(&bytes).unwrap();
        for _ in 0..90 {
            assert_eq!(
                serde_json::to_value(resumed.step(&[])).unwrap(),
                serde_json::to_value(uninterrupted.step(&[])).unwrap()
            );
            assert_eq!(
                serde_json::to_value(&resumed).unwrap(),
                serde_json::to_value(&uninterrupted).unwrap()
            );
            resumed.validate().unwrap();
        }
    }
}

#[test]
fn current_energyball_rejects_unreachable_reflection_phase() {
    // PB48: reflection can be set only after immunity reaches zero, and
    // the clock never re-arms. This contradictory state must not resume.
    let mut invalid = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: ulysses_energyball_session(1),
    })
    .unwrap();
    invalid["session"]["board"]["missiles"][0]["reflected"] = true.into();
    assert!(cli::decode_save(&serde_json::to_vec(&invalid).unwrap()).is_err());
}

#[test]
fn current_ulysses_rejects_unreachable_reload_clock() {
    // PB47: initial75 or subsequent150..199, never the gap between them.
    let mut invalid = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: ulysses_energyball_session(0),
    })
    .unwrap();
    invalid["session"]["board"]["invasion"]["actors"][0]["reload_ticks"] = 100.into();
    assert!(cli::decode_save(&serde_json::to_vec(&invalid).unwrap()).is_err());
}

fn psychosquid_and_bomb_session(divisor: f64) -> AdventureSession {
    use turbofish_deluxe::{
        alien::{SylvesterKind, WeakSylvester},
        fish_pet::FishPetKind,
    };
    // Controlled current-format fixture. Actual update creates the bomb;
    // private next_id allocation gives the alien a distinct legal Board ID.
    let mut session = tank_three_third_session(&[PetKind::Seymour, PetKind::Shrapnel]);
    let shrapnel = session
        .board
        .as_mut()
        .unwrap()
        .fish_pets
        .iter_mut()
        .find(|pet| pet.kind == FishPetKind::Shrapnel)
        .unwrap();
    shrapnel.coin_timer = shrapnel.bomb_threshold - 1;
    session.step(&[]);
    let mut encoded = serde_json::to_value(&session).unwrap();
    let alien_id = encoded["board"]["next_id"].as_u64().unwrap();
    encoded["board"]["next_id"] = (alien_id + 1).into();
    session = serde_json::from_value(encoded).unwrap();
    let mut actor = WeakSylvester::spawn_kind(SylvesterKind::Psychosquid, alien_id, 480, 180, 1, 1);
    actor.spawn_ticks = 0;
    actor.health = 300.25;
    actor.healing = true;
    actor.ever_healed = true;
    actor.phase_ticks = 399;
    actor.movement_divisor = divisor;
    let wave = session.board.as_mut().unwrap().invasion.as_mut().unwrap();
    wave.actors = vec![actor];
    wave.battle_active = true;
    // Actual spawning resets the next countdown to3000 before registering
    // the actor. The earlier bomb-producing tick had reduced it to2999.
    wave.countdown = 3000;
    session.validate().unwrap();
    session
}

#[test]
fn current_tank_three_third_save_accepts_each_of_twelve_available_pets() {
    for pet in [
        PetKind::Stinky,
        PetKind::Niko,
        PetKind::Itchy,
        PetKind::Prego,
        PetKind::Zorf,
        PetKind::Clyde,
        PetKind::Vert,
        PetKind::Rufus,
        PetKind::Meryl,
        PetKind::Wadsworth,
        PetKind::Seymour,
        PetKind::Shrapnel,
    ] {
        let session = tank_three_third_session(&[pet]);
        session.validate().unwrap();
        let bytes = serde_json::to_vec(&cli::ProjectSave {
            format_version: cli::SAVE_FORMAT_VERSION,
            session: session.clone(),
        })
        .unwrap();
        assert_eq!(
            serde_json::to_value(cli::decode_save(&bytes).unwrap()).unwrap(),
            serde_json::to_value(session).unwrap()
        );
    }
    assert!(AdventureState::new_tank3_third_stage(42, &[PetKind::Gumbo]).is_err());
}

#[test]
fn current_psychosquid_and_shrapnel_require_all_new_authoritative_fields() {
    let current = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: psychosquid_and_bomb_session(0.5),
    })
    .unwrap();
    for field in [
        "phase_ticks",
        "phase_threshold",
        "healing",
        "ever_healed",
        "movement_divisor",
    ] {
        let mut missing = current.clone();
        missing
            .pointer_mut("/session/board/invasion/actors/0")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err(),
            "accepted missing {field}"
        );
    }
    for field in ["bomb_threshold", "glint_phase"] {
        let mut missing = current.clone();
        // All FishTypePet fields are required in the current format, even
        // when the neutral values belong to Seymour rather than Shrapnel.
        missing
            .pointer_mut("/session/board/fish_pets/0")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err(),
            "accepted missing {field}"
        );
    }
    let mut missing = current;
    missing
        .pointer_mut("/session/board/coins/0")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("hazard_age_ticks");
    assert!(cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err());
}

#[test]
fn current_psychosquid_over_starting_health_and_distinct_speed_resume_exact_updates() {
    for divisor in [0.5_f64, 2.0] {
        let mut uninterrupted = psychosquid_and_bomb_session(divisor);
        let bytes = serde_json::to_vec(&cli::ProjectSave {
            format_version: cli::SAVE_FORMAT_VERSION,
            session: uninterrupted.clone(),
        })
        .unwrap();
        let mut resumed = cli::decode_save(&bytes).unwrap();
        let actor = &resumed
            .board
            .as_ref()
            .unwrap()
            .invasion
            .as_ref()
            .unwrap()
            .actors[0];
        assert_eq!(actor.health.to_bits(), 300.25_f64.to_bits());
        assert_eq!(actor.movement_divisor.to_bits(), divisor.to_bits());
        for _ in 0..85 {
            assert_eq!(
                serde_json::to_value(resumed.step(&[])).unwrap(),
                serde_json::to_value(uninterrupted.step(&[])).unwrap()
            );
            assert_eq!(
                serde_json::to_value(&resumed).unwrap(),
                serde_json::to_value(&uninterrupted).unwrap()
            );
            resumed.validate().unwrap();
        }
    }
}

#[test]
fn current_bomb_burst_requires_each_field_and_continues_after_reload() {
    use turbofish_deluxe::sim::CoinKind;
    let mut session = psychosquid_and_bomb_session(2.0);
    let board = session.board.as_mut().unwrap();
    let target_x = f64::from(board.fish[0].x as i32) + 4.0;
    let target_y = f64::from(board.fish[0].y as i32) + 4.0;
    let bomb = board
        .coins
        .iter_mut()
        .find(|coin| coin.kind == CoinKind::ShrapnelBomb)
        .unwrap();
    // Controlled contact at the primary signed age31 boundary. No debug
    // earning claim: ordinary Board transactions create the finite bursts.
    bomb.x = target_x;
    bomb.y = target_y;
    bomb.hazard_age_ticks = 30;
    session.step(&[]);
    assert!(!session.board.as_ref().unwrap().bomb_shots.is_empty());
    session.validate().unwrap();
    let saved = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: session.clone(),
    })
    .unwrap();
    let mut missing_list = saved.clone();
    missing_list["session"]["board"]
        .as_object_mut()
        .unwrap()
        .remove("bomb_shots");
    assert!(cli::decode_save(&serde_json::to_vec(&missing_list).unwrap()).is_err());
    for field in [
        "shot_type",
        "x",
        "y",
        "age_ticks",
        "frame",
        "delay_ticks",
        "alpha",
    ] {
        let mut missing = saved.clone();
        missing
            .pointer_mut("/session/board/bomb_shots/0")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err(),
            "accepted missing burst {field}"
        );
    }
    let mut resumed = cli::decode_save(&serde_json::to_vec(&saved).unwrap()).unwrap();
    for _ in 0..24 {
        assert_eq!(
            serde_json::to_value(resumed.step(&[])).unwrap(),
            serde_json::to_value(session.step(&[])).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&resumed).unwrap(),
            serde_json::to_value(&session).unwrap()
        );
    }
    assert!(resumed.board.as_ref().unwrap().bomb_shots.is_empty());
}

#[test]
fn current_tank_three_second_save_accepts_all_eleven_single_pet_rosters() {
    for pet in [
        PetKind::Stinky,
        PetKind::Niko,
        PetKind::Itchy,
        PetKind::Prego,
        PetKind::Zorf,
        PetKind::Clyde,
        PetKind::Vert,
        PetKind::Rufus,
        PetKind::Meryl,
        PetKind::Wadsworth,
        PetKind::Seymour,
    ] {
        let session = tank_three_second_session(&[pet]);
        session.validate().unwrap();
        let saved = serde_json::to_vec(&cli::ProjectSave {
            format_version: cli::SAVE_FORMAT_VERSION,
            session: session.clone(),
        })
        .unwrap();
        assert_eq!(
            serde_json::to_value(cli::decode_save(&saved).unwrap()).unwrap(),
            serde_json::to_value(session).unwrap()
        );
    }
}

#[test]
fn current_gekko_lists_live_corpse_and_coin_counter_require_every_field() {
    use turbofish_deluxe::gekko::DeadGekko;
    let mut session = bought_gekko_session();
    let board = session.board.as_mut().unwrap();
    let gekko = board.gekkos.pop().unwrap();
    board.dead_gekkos.push(DeadGekko::from_impact(&gekko));
    // Keep both live and corpse schemas in this valid fixture using the
    // next production purchase, so no duplicated ID is manufactured.
    board.balance = 2000;
    session.apply_actions(&[Action::BuyGekko]);
    for _ in 0..55 {
        session.step(&[]);
    }
    let board = session.board.as_mut().unwrap();
    board.gekkos[0].coin_timer = board.gekkos[0].coin_threshold - 1;
    session.step(&[]);
    session.validate().unwrap();
    let current = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session,
    })
    .unwrap();
    for field in ["gekko_unlocked", "gekkos", "dead_gekkos"] {
        let mut missing = current.clone();
        missing["session"]["board"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err(),
            "{field}"
        );
    }
    for pointer in ["/session/board/gekkos/0", "/session/board/dead_gekkos/0"] {
        for field in current
            .pointer(pointer)
            .unwrap()
            .as_object()
            .unwrap()
            .keys()
        {
            let mut missing = current.clone();
            missing
                .pointer_mut(pointer)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(
                cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err(),
                "{pointer}/{field} must be required"
            );
        }
    }
    let mut missing = current;
    missing["session"]["board"]["coins"][0]
        .as_object_mut()
        .unwrap()
        .remove("animation_ticks");
    assert!(cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err());
}

#[test]
fn current_gekko_and_seymour_coin_continue_with_exact_double_positions() {
    use turbofish_deluxe::sim::{CoinKind, Event};
    let mut uninterrupted = bought_gekko_session();
    for _ in 0..55 {
        uninterrupted.step(&[]);
    }
    let board = uninterrupted.board.as_mut().unwrap();
    board.gekkos[0].coin_timer = board.gekkos[0].coin_threshold - 1;
    assert!(
        uninterrupted
            .step(&[])
            .iter()
            .any(|event| matches!(event, Event::GekkoPearlDropped { .. }))
    );
    let pearl_id = uninterrupted
        .board
        .as_ref()
        .unwrap()
        .coins
        .iter()
        .find(|coin| coin.kind == CoinKind::Pearl)
        .unwrap()
        .id;
    // A source-derived .8 accumulation boundary, with actual actor membership.
    uninterrupted
        .board
        .as_mut()
        .unwrap()
        .coins
        .iter_mut()
        .find(|coin| coin.id == pearl_id)
        .unwrap()
        .y = 100.0;
    for _ in 0..5 {
        uninterrupted.step(&[]);
    }
    let pearl = uninterrupted
        .board
        .as_ref()
        .unwrap()
        .coins
        .iter()
        .find(|coin| coin.id == pearl_id)
        .unwrap();
    assert_eq!(pearl.y.to_bits(), 103.99999999999999_f64.to_bits());
    let saved = serde_json::to_vec(&cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: uninterrupted.clone(),
    })
    .unwrap();
    let mut resumed = cli::decode_save(&saved).unwrap();
    for _ in 0..80 {
        let expected = uninterrupted.step(&[]);
        let actual = resumed.step(&[]);
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::to_value(expected).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&resumed).unwrap(),
            serde_json::to_value(&uninterrupted).unwrap()
        );
        let expected_coin = uninterrupted
            .board
            .as_ref()
            .unwrap()
            .coins
            .iter()
            .find(|coin| coin.id == pearl_id)
            .unwrap();
        let actual_coin = resumed
            .board
            .as_ref()
            .unwrap()
            .coins
            .iter()
            .find(|coin| coin.id == pearl_id)
            .unwrap();
        assert_eq!(actual_coin.x.to_bits(), expected_coin.x.to_bits());
        assert_eq!(actual_coin.y.to_bits(), expected_coin.y.to_bits());
        resumed.validate().unwrap();
    }
}

#[test]
fn current_gekko_purchase_evidence_requires_gates_but_not_surviving_producer() {
    use turbofish_deluxe::{gekko::DeadGekko, sim::CoinKind};
    let live = bought_gekko_session();
    let mut corpse = live.clone();
    let actor = corpse.board.as_mut().unwrap().gekkos.pop().unwrap();
    corpse
        .board
        .as_mut()
        .unwrap()
        .dead_gekkos
        .push(DeadGekko::from_impact(&actor));
    let mut pearl = live.clone();
    for _ in 0..55 {
        pearl.step(&[]);
    }
    let board = pearl.board.as_mut().unwrap();
    board.gekkos[0].coin_timer = board.gekkos[0].coin_threshold - 1;
    pearl.step(&[]);
    let board = pearl.board.as_mut().unwrap();
    assert!(board.coins.iter().any(|coin| coin.kind == CoinKind::Pearl));
    board.gekkos.clear();
    // Output is independent membership and legitimately outlives its producer.
    for session in [live, corpse, pearl] {
        session.validate().unwrap();
        let current = serde_json::to_value(cli::ProjectSave {
            format_version: cli::SAVE_FORMAT_VERSION,
            session,
        })
        .unwrap();
        assert!(cli::decode_save(&serde_json::to_vec(&current).unwrap()).is_ok());
        let mut hidden_final_gates = current.clone();
        hidden_final_gates["session"]["board"]["weapon_unlocked"] = false.into();
        hidden_final_gates["session"]["board"]["egg_unlocked"] = false.into();
        assert!(cli::decode_save(&serde_json::to_vec(&hidden_final_gates).unwrap()).is_err());
        let mut hidden_gekko_gate = current;
        hidden_gekko_gate["session"]["board"]["gekko_unlocked"] = false.into();
        assert!(cli::decode_save(&serde_json::to_vec(&hidden_gekko_gate).unwrap()).is_err());
    }
}

#[test]
fn current_tank_three_save_accepts_every_unlocked_single_pet_roster() {
    for pet in [
        PetKind::Stinky,
        PetKind::Niko,
        PetKind::Itchy,
        PetKind::Prego,
        PetKind::Zorf,
        PetKind::Clyde,
        PetKind::Vert,
        PetKind::Rufus,
        PetKind::Meryl,
        PetKind::Wadsworth,
    ] {
        let session = tank_three_session(&[pet]);
        session.validate().unwrap();
        let saved = serde_json::to_vec(&cli::ProjectSave {
            format_version: cli::SAVE_FORMAT_VERSION,
            session: session.clone(),
        })
        .unwrap();
        let restored = cli::decode_save(&saved).unwrap();
        assert_eq!(
            serde_json::to_value(restored).unwrap(),
            serde_json::to_value(session).unwrap()
        );
    }
}

#[test]
fn current_tank_three_lists_and_ward_fields_are_required_without_backfill() {
    let current = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: tank_three_session(&[PetKind::Wadsworth]),
    })
    .unwrap();
    for (pointer, fields) in [
        (
            "/session/board",
            &["grubber_unlocked", "grubbers", "dead_grubbers", "larvae"][..],
        ),
        (
            "/session/board/fish_pets/0",
            &["ward_active", "ward_timer", "published_x", "published_y"][..],
        ),
    ] {
        for field in fields {
            let mut missing = current.clone();
            missing
                .pointer_mut(pointer)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(*field);
            assert!(
                cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err(),
                "{pointer}/{field} must be required"
            );
        }
    }
}

#[test]
fn current_bought_grubber_and_claimed_larva_reload_every_next_update() {
    use turbofish_deluxe::sim::Event;
    let mut uninterrupted =
        tank_three_session(&[PetKind::Niko, PetKind::Itchy, PetKind::Wadsworth]);
    let board = uninterrupted.board.as_mut().unwrap();
    // Synthetic shop/production-boundary fixture; no earning claim.
    board.balance = 750;
    board.grubber_unlocked = true;
    board.upgrades.quality_unlocked = true;
    board.upgrades.quantity_unlocked = true;
    uninterrupted.apply_actions(&[Action::BuyGrubber]);
    for _ in 0..45 {
        uninterrupted.step(&[]);
    }
    let grubber = &mut uninterrupted.board.as_mut().unwrap().grubbers[0];
    grubber.coin_timer = grubber.coin_threshold - 1;
    let produced = uninterrupted.step(&[]);
    assert_eq!(
        produced
            .iter()
            .filter(|event| matches!(event, Event::LarvaDropped { .. }))
            .count(),
        1
    );
    let larva_id = uninterrupted.board.as_ref().unwrap().larvae[0].id;
    // Floor-born larvae must rise past the strict old-Y320 click gate.
    // Advance normal Board updates rather than editing their visibility.
    for _ in 0..64 {
        if uninterrupted.board.as_ref().unwrap().larvae[0].mouse_visible {
            break;
        }
        uninterrupted.step(&[]);
    }
    let larva = &uninterrupted.board.as_ref().unwrap().larvae[0];
    assert_eq!(larva.id, larva_id);
    assert!(larva.mouse_visible);
    let click = Action::Click {
        x: (larva.widget_x + 36) as f32,
        y: (larva.widget_y + 36) as f32,
    };
    let claimed = uninterrupted.apply_actions(&[click]);
    assert!(claimed.iter().any(|event| matches!(event, Event::LarvaCollectionStarted { larva_id: id, .. } if *id == larva_id)));
    uninterrupted.validate().unwrap();
    let saved = serde_json::to_vec(&cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: uninterrupted.clone(),
    })
    .unwrap();
    let mut resumed = cli::decode_save(&saved).unwrap();
    let mut credited = 0;
    for _ in 0..64 {
        let expected = uninterrupted.step(&[]);
        let actual = resumed.step(&[]);
        assert_eq!(
            serde_json::to_value(&actual).unwrap(),
            serde_json::to_value(&expected).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&resumed).unwrap(),
            serde_json::to_value(&uninterrupted).unwrap()
        );
        credited += actual.iter().filter(|event| matches!(event, Event::LarvaCredited { larva_id: id, amount: 150, .. } if *id == larva_id)).count();
        resumed.validate().unwrap();
    }
    assert_eq!(credited, 1);
}

#[test]
fn current_live_ward_and_published_position_reload_every_next_update() {
    use turbofish_deluxe::fish_pet::FishPetKind;
    let mut uninterrupted = tank_three_session(&[PetKind::Wadsworth]);
    // Synthetic valid warning boundary, then ordinary spawn/activation ticks.
    uninterrupted
        .board
        .as_mut()
        .unwrap()
        .invasion
        .as_mut()
        .unwrap()
        .countdown = 276;
    for _ in 0..276 {
        uninterrupted.step(&[]);
    }
    let board = uninterrupted.board.as_ref().unwrap();
    assert!(!board.invasion.as_ref().unwrap().actors.is_empty());
    let ward = board
        .fish_pets
        .iter()
        .find(|pet| pet.kind == FishPetKind::Wadsworth)
        .unwrap();
    assert!(ward.ward_active);
    assert_eq!(ward.ward_timer, 120);
    uninterrupted.validate().unwrap();
    let saved = serde_json::to_vec(&cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: uninterrupted.clone(),
    })
    .unwrap();
    let mut resumed = cli::decode_save(&saved).unwrap();
    for _ in 0..64 {
        let expected = uninterrupted.step(&[]);
        let actual = resumed.step(&[]);
        assert_eq!(
            serde_json::to_value(&actual).unwrap(),
            serde_json::to_value(&expected).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&resumed).unwrap(),
            serde_json::to_value(&uninterrupted).unwrap()
        );
        resumed.validate().unwrap();
    }
}

#[test]
fn current_pair_successor_song_note_and_pending_death_reload_every_update() {
    use turbofish_deluxe::{
        alien::{SylvesterKind, WeakSylvester},
        fish_pet::FishPetKind,
        invasion::{EncounterKind, InvasionEvent, WavePlan},
        sim::Event,
    };
    let mut uninterrupted = meryl_session();
    uninterrupted
        .board
        .as_mut()
        .unwrap()
        .fish_pets
        .iter_mut()
        .find(|pet| pet.kind == FishPetKind::Meryl)
        .unwrap()
        .coin_timer = 1299;
    let note_events = uninterrupted.step(&[]);
    assert_eq!(
        note_events
            .iter()
            .filter(|event| matches!(event, Event::MerylNoteDropped { .. }))
            .count(),
        1
    );
    let mut fixture = serde_json::to_value(&uninterrupted).unwrap();
    let first_id = fixture["board"]["next_id"].as_u64().unwrap();
    fixture["board"]["next_id"] = (first_id + 2).into();
    uninterrupted = serde_json::from_value(fixture).unwrap();
    let board = uninterrupted.board.as_mut().unwrap();
    let wave = board.invasion.as_mut().unwrap();
    wave.plan = WavePlan::CyclingTank2Finale {
        next: EncounterKind::Single(SylvesterKind::Destructor),
    };
    // A spawned encounter resets its countdown before registering actors.
    wave.countdown = 3000;
    let mut weak = WeakSylvester::spawn_kind(SylvesterKind::Weak, first_id, 40, 100, 1, 1);
    weak.spawn_ticks = 0;
    weak.health = 0.0;
    wave.actors = vec![
        weak,
        WeakSylvester::spawn_kind(SylvesterKind::Balrog, first_id + 1, 500, 100, 1, 1),
    ];
    wave.battle_active = true;
    uninterrupted.validate().unwrap();
    let saved = serde_json::to_vec(&cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: uninterrupted.clone(),
    })
    .unwrap();
    let mut resumed = cli::decode_save(&saved).unwrap();
    for index in 0..64 {
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
        if index == 0 {
            assert_eq!(actual_events.iter().filter(|event| matches!(event, Event::Invasion { event: InvasionEvent::AlienDefeated { id }, .. } if *id == first_id)).count(), 1);
            assert!(!actual_events.iter().any(|event| matches!(
                event,
                Event::Invasion {
                    event: InvasionEvent::BattleEnded,
                    ..
                }
            )));
        }
        resumed.validate().unwrap();
    }
}

#[test]
fn current_finale_fields_are_required_without_reconstruction() {
    let mut session = meryl_session();
    session
        .board
        .as_mut()
        .unwrap()
        .fish_pets
        .iter_mut()
        .find(|pet| pet.kind == turbofish_deluxe::fish_pet::FishPetKind::Meryl)
        .unwrap()
        .coin_timer = 1300;
    let current = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session,
    })
    .unwrap();
    for pointer in [
        "/session/board",
        "/session/board/invasion",
        "/session/board/fish_pets/0",
        "/session/board/fish_pets/1",
    ] {
        let fields: &[&str] = match pointer {
            "/session/board" => &["notes"],
            "/session/board/invasion" => {
                &["plan", "actors", "warps", "dead_aliens", "battle_active"]
            }
            _ => &["meryl_blink"],
        };
        for field in fields {
            let mut missing = current.clone();
            missing
                .pointer_mut(pointer)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(*field);
            assert!(
                cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err(),
                "{pointer}/{field} must be present"
            );
        }
    }
}

#[test]
fn current_save_rejects_battle_latch_without_matching_registered_threats() {
    let (mut missile_only, _, _) = pending_destructor_session();
    missile_only.step(&[]);
    let board = missile_only.board.as_ref().unwrap();
    assert!(board.invasion.as_ref().unwrap().actors.is_empty());
    assert!(!board.missiles.is_empty());
    assert!(board.invasion.as_ref().unwrap().battle_active);
    missile_only.validate().unwrap();
    let mut missing_battle = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: missile_only,
    })
    .unwrap();
    missing_battle["session"]["board"]["invasion"]["battle_active"] = false.into();
    assert!(cli::decode_save(&serde_json::to_vec(&missing_battle).unwrap()).is_err());
    let peaceful = meryl_session();
    peaceful.validate().unwrap();
    let mut phantom_battle = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: peaceful,
    })
    .unwrap();
    phantom_battle["session"]["board"]["invasion"]["battle_active"] = true.into();
    assert!(cli::decode_save(&serde_json::to_vec(&phantom_battle).unwrap()).is_err());
}

#[test]
fn current_tank_two_bonus_origin_and_results_credit_survive_reload() {
    use turbofish_deluxe::bonus::{BonusResult, BonusState};
    let mut session = meryl_session();
    session.progress.level = 6;
    session.progress.unlocked_pets.push(PetKind::Wadsworth);
    session.board = None;
    let mut bonus = BonusState::new_for_tank(42, 2).unwrap();
    bonus.click(0.0, 0.0);
    bonus.update();
    let shell = &bonus.shells[0];
    bonus.click(shell.x as f32 + 1.0, shell.y as f32 + 1.0);
    bonus.update();
    session.ticks = bonus.tick;
    session.phase = AdventurePhase::Bonus { state: bonus };
    session.validate().unwrap();
    let current = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: session.clone(),
    })
    .unwrap();
    let mut missing = current.clone();
    missing["session"]["phase"]["Bonus"]["state"]
        .as_object_mut()
        .unwrap()
        .remove("origin_tank");
    assert!(cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err());
    let mut resumed = cli::decode_save(&serde_json::to_vec(&current).unwrap()).unwrap();
    for _ in 0..32 {
        assert_eq!(
            serde_json::to_value(resumed.step(&[])).unwrap(),
            serde_json::to_value(session.step(&[])).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&resumed).unwrap(),
            serde_json::to_value(&session).unwrap()
        );
        resumed.validate().unwrap();
    }
    session.progress.tank = 3;
    session.progress.level = 1;
    session.progress.shell_balance = 317;
    session.phase = AdventurePhase::BonusResults {
        result: BonusResult {
            origin_tank: 2,
            origin_level: 6,
            earned: 217,
            previous_balance: 100,
            updates: 8,
            purchase: PurchaseReceipt::new(0),
        },
    };
    let results = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session,
    })
    .unwrap();
    let mut loaded = cli::decode_save(&serde_json::to_vec(&results).unwrap()).unwrap();
    for _ in 0..40 {
        loaded.step(&[]);
    }
    assert_eq!(loaded.progress.shell_balance, 317);
    loaded.apply_actions(&[Action::Continue]);
    assert!(matches!(loaded.phase, AdventurePhase::PetSelection { .. }));
    loaded.validate().unwrap();
    for invalid in [0, 1, 3, 255] {
        let mut wrong = results.clone();
        wrong["session"]["phase"]["BonusResults"]["result"]["origin_tank"] = invalid.into();
        assert!(cli::decode_save(&serde_json::to_vec(&wrong).unwrap()).is_err());
    }
    let mut missing = results;
    missing["session"]["phase"]["BonusResults"]["result"]
        .as_object_mut()
        .unwrap()
        .remove("origin_tank");
    assert!(cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err());
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
            assert!(!board.invasion.as_ref().unwrap().dead_aliens.is_empty());
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
        ("/session/board/rufus/0/vy", serde_json::json!(1.0)),
        (
            "/session/board/invasion/actors/0/health",
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
    board.invasion.as_mut().unwrap().actors = vec![actor];
    board.invasion.as_mut().unwrap().battle_active = true;
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
            .actors
            .first()
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
                    .dead_aliens
                    .is_empty()
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
    let mut value = historical_envelope(cli::ProjectSave {
        format_version: 6,
        session,
    });
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
        animation_ticks: 0,
        hazard_age_ticks: 0,
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
            origin_tank: 1,
            origin_level: 6,
            earned: 217,
            previous_balance: 100,
            updates: 8,
            purchase: PurchaseReceipt::new(0),
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
fn current_purchase_receipt_requires_exact_credited_then_debited_wallet() {
    let mut session = AdventureSession::new(0x3922);
    session.progress.tank = 2;
    session.progress.level = 1;
    session.progress.unlocked_pets = vec![
        PetKind::Stinky,
        PetKind::Niko,
        PetKind::Itchy,
        PetKind::Prego,
        PetKind::Zorf,
    ];
    session.progress.shell_balance = 20_000;
    session.board = None;
    session.phase = AdventurePhase::BonusResults {
        result: turbofish_deluxe::bonus::BonusResult {
            origin_tank: 1,
            origin_level: 6,
            earned: 0,
            previous_balance: 20_000,
            updates: 30,
            purchase: PurchaseReceipt::new(0),
        },
    };
    session.validate().unwrap();
    let base = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session: session.clone(),
    })
    .unwrap();
    for pointer in [
        "/session/progress/purchase_cursor",
        "/session/phase/BonusResults/result/purchase/offered_cursor",
        "/session/phase/BonusResults/result/purchase/confirming",
        "/session/phase/BonusResults/result/purchase/purchased",
    ] {
        let mut missing = base.clone();
        let (parent, field) = pointer.rsplit_once('/').unwrap();
        missing
            .pointer_mut(parent)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err(),
            "{pointer}"
        );
    }
    session.apply_actions(&[Action::OfferBonusPurchase]);
    let mut reopened = cli::decode_save(&current_bytes(session)).unwrap();
    reopened.apply_actions(&[Action::ConfirmBonusPurchase { accept: true }]);
    assert_eq!(reopened.progress.shell_balance, 0);
    assert_eq!(reopened.progress.purchase_cursor, 1);
    assert!(reopened.board.is_none());
    let committed = current_bytes(reopened.clone());
    assert_eq!(
        cli::decode_save(&committed).unwrap().progress,
        reopened.progress
    );
    let mut forged: serde_json::Value = serde_json::from_slice(&committed).unwrap();
    forged["session"]["progress"]["shell_balance"] = 1.into();
    assert!(cli::decode_save(&serde_json::to_vec(&forged).unwrap()).is_err());
    forged["session"]["progress"]["shell_balance"] = 0.into();
    forged["session"]["phase"]["BonusResults"]["result"]["earned"] = 1.into();
    assert!(cli::decode_save(&serde_json::to_vec(&forged).unwrap()).is_err());
    forged = serde_json::from_slice(&committed).unwrap();
    forged["session"]["phase"]["BonusResults"]["result"]["updates"] = 29.into();
    assert!(cli::decode_save(&serde_json::to_vec(&forged).unwrap()).is_err());
    forged = serde_json::from_slice(&committed).unwrap();
    forged["session"]["phase"]["BonusResults"]["result"]["purchase"]["offered_cursor"] = 255.into();
    assert!(cli::decode_save(&serde_json::to_vec(&forged).unwrap()).is_err());
}

#[test]
fn current_purchase_cursor_one_nostradamus_reopens_with_single_exact_debit() {
    let mut session = AdventureSession::new(0x3925);
    session.progress.tank = 2;
    session.progress.level = 1;
    session.progress.unlocked_pets = vec![
        PetKind::Stinky,
        PetKind::Niko,
        PetKind::Itchy,
        PetKind::Prego,
        PetKind::Zorf,
        PetKind::Brinkley,
    ];
    session.progress.purchase_cursor = 1;
    session.progress.shell_balance = 25_000;
    session.board = None;
    session.phase = AdventurePhase::BonusResults {
        result: turbofish_deluxe::bonus::BonusResult {
            origin_tank: 1,
            origin_level: 6,
            earned: 0,
            previous_balance: 25_000,
            updates: 30,
            purchase: PurchaseReceipt::new(1),
        },
    };
    session.validate().unwrap();
    session.apply_actions(&[Action::OfferBonusPurchase]);
    let mut reopened = cli::decode_save(&current_bytes(session)).unwrap();
    reopened.apply_actions(&[Action::ConfirmBonusPurchase { accept: true }]);
    assert_eq!(
        (
            reopened.progress.purchase_cursor,
            reopened.progress.shell_balance
        ),
        (2, 0)
    );
    assert_eq!(
        reopened.progress.unlocked_pets.last(),
        Some(&PetKind::Nostradamus)
    );
    assert!(reopened.board.is_none());
    let committed = current_bytes(reopened.clone());
    let mut twice = cli::decode_save(&committed).unwrap();
    let repeat = twice.apply_actions(&[Action::OfferBonusPurchase]);
    assert_eq!(
        repeat
            .iter()
            .filter(|event| matches!(
                event,
                turbofish_deluxe::sim::Event::Rejected {
                    reason: turbofish_deluxe::sim::Rejection::Locked,
                    ..
                }
            ))
            .count(),
        1
    );
    assert!(!repeat.iter().any(|event| matches!(
        event,
        turbofish_deluxe::sim::Event::BonusPurchaseOffered { .. }
            | turbofish_deluxe::sim::Event::BonusPurchaseCommitted { .. }
    )));
    assert_eq!(twice.progress, reopened.progress);
    let mut forged: serde_json::Value = serde_json::from_slice(&committed).unwrap();
    forged["session"]["progress"]["unlocked_pets"]
        .as_array_mut()
        .unwrap()
        .remove(5);
    assert!(cli::decode_save(&serde_json::to_vec(&forged).unwrap()).is_err());
}

#[test]
fn current_brinkley_initial_time_trial_actor_roundtrips_without_egg_candidate() {
    let mut session = AdventureSession::new(0x3923);
    session.progress.tank = 2;
    session.progress.level = 1;
    session.progress.unlocked_pets = vec![
        PetKind::Stinky,
        PetKind::Niko,
        PetKind::Itchy,
        PetKind::Prego,
        PetKind::Zorf,
        PetKind::Brinkley,
    ];
    session.progress.purchase_cursor = 1;
    session.board = None;
    session.phase = AdventurePhase::GameSelector;
    session.apply_actions(&[
        Action::PlayTimeTrial,
        Action::SelectTimeTrialTank { tank: 1 },
        Action::TogglePet {
            pet: PetKind::Brinkley,
        },
        Action::Continue,
    ]);
    session.validate().unwrap();
    let board = session.board.as_ref().unwrap();
    assert_eq!(board.pets, [PetKind::Brinkley]);
    assert_eq!(board.fish_pets.len(), 1);
    let serialized = current_bytes(session.clone());
    let mut reopened = cli::decode_save(&serialized).unwrap();
    let mut egg_check = reopened.clone();
    let purchased = egg_check.apply_actions(&[Action::BuyEgg]);
    assert!(purchased.iter().any(|event| matches!(
        event,
        turbofish_deluxe::sim::Event::TimeTrialPetAcquired { .. }
    )));
    assert!(!purchased.iter().any(|event| matches!(
        event,
        turbofish_deluxe::sim::Event::TimeTrialPetAcquired {
            pet: PetKind::Brinkley,
            ..
        }
    )));
    for _ in 0..12 {
        assert_eq!(
            serde_json::to_value(session.step(&[])).unwrap(),
            serde_json::to_value(reopened.step(&[])).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&session).unwrap(),
            serde_json::to_value(&reopened).unwrap()
        );
    }
    let mut missing: serde_json::Value = serde_json::from_slice(&serialized).unwrap();
    missing["session"]["board"]["fish_pets"][0]
        .as_object_mut()
        .unwrap()
        .remove("brinkley_meals");
    assert!(cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err());
    missing = serde_json::from_slice(&serialized).unwrap();
    missing["session"]["board"]["fish_pets"][0]
        .as_object_mut()
        .unwrap()
        .remove("brinkley_cooldown");
    assert!(cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err());
    missing = serde_json::from_slice(&serialized).unwrap();
    missing["session"]["board"]["fish_pets"][0]["brinkley_cooldown"] = 109.into();
    assert!(cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err());
}

#[test]
fn current_nostradamus_actor_food_and_inflight_pickup_require_complete_state() {
    let mut session = AdventureSession::new(0x3934);
    session.progress.tank = 2;
    session.progress.level = 1;
    session.progress.unlocked_pets = vec![
        PetKind::Stinky,
        PetKind::Niko,
        PetKind::Itchy,
        PetKind::Prego,
        PetKind::Zorf,
        PetKind::Brinkley,
        PetKind::Nostradamus,
    ];
    session.progress.purchase_cursor = 2;
    session.board = None;
    session.phase = AdventurePhase::GameSelector;
    session.apply_actions(&[
        Action::PlayTimeTrial,
        Action::SelectTimeTrialTank { tank: 1 },
        Action::TogglePet {
            pet: PetKind::Nostradamus,
        },
        Action::Continue,
    ]);
    session.validate().unwrap();
    let board = session.board.as_mut().unwrap();
    board.fish_pets[0].nostra_threshold = 300;
    board.fish_pets[0].nostra_elapsed = 299;
    session.step(&[]);
    let food = &session.board.as_ref().unwrap().food[0];
    assert_eq!(food.food_type, turbofish_deluxe::sim::FoodType::Nostradamus);
    assert_eq!(food.ineligible_ticks, 20);
    let serialized = current_bytes(session.clone());
    let mut reopened = cli::decode_save(&serialized).unwrap();
    for pointer in [
        "/session/board/fish_pets/0/nostra_elapsed",
        "/session/board/fish_pets/0/nostra_threshold",
        "/session/board/fish_pets/0/nostra_converted",
        "/session/board/food/0/food_type",
        "/session/board/food/0/picked_up",
        "/session/board/invasion/sneeze_shake_ticks",
    ] {
        let mut missing: serde_json::Value = serde_json::from_slice(&serialized).unwrap();
        let (parent, field) = pointer.rsplit_once('/').unwrap();
        missing
            .pointer_mut(parent)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err(),
            "{pointer}"
        );
    }
    let food = &reopened.board.as_ref().unwrap().food[0];
    let point = (food.x + 10.0, food.y + 10.0);
    reopened.apply_actions(&[Action::Click {
        x: point.0,
        y: point.1,
    }]);
    assert!(reopened.board.as_ref().unwrap().food[0].picked_up);
    let mut continued = cli::decode_save(&current_bytes(reopened.clone())).unwrap();
    for _ in 0..100 {
        assert_eq!(
            serde_json::to_value(reopened.step(&[])).unwrap(),
            serde_json::to_value(continued.step(&[])).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&reopened).unwrap(),
            serde_json::to_value(&continued).unwrap()
        );
    }
    assert_eq!(reopened.board.as_ref().unwrap().food.len(), 0);
    reopened.validate().unwrap();
}

#[test]
fn current_purchase_time_trial_result_requires_clock_and_receipt() {
    let mut session = AdventureSession::new(0x3924);
    session.progress.tank = 2;
    session.progress.level = 1;
    session.progress.unlocked_pets = vec![
        PetKind::Stinky,
        PetKind::Niko,
        PetKind::Itchy,
        PetKind::Prego,
        PetKind::Zorf,
    ];
    session.board = None;
    session.phase = AdventurePhase::GameSelector;
    session.apply_actions(&[
        Action::PlayTimeTrial,
        Action::SelectTimeTrialTank { tank: 1 },
        Action::TogglePet {
            pet: PetKind::Stinky,
        },
        Action::Continue,
    ]);
    session.board.as_mut().unwrap().tick = 10_749;
    session.ticks = 10_749;
    session.step(&[]);
    assert_eq!(session.phase, AdventurePhase::TimeTrialTimesUp);
    session.apply_actions(&[Action::Continue]);
    assert_eq!(session.phase, AdventurePhase::TimeTrialResults);
    session.validate().unwrap();
    let current = serde_json::to_value(cli::ProjectSave {
        format_version: cli::SAVE_FORMAT_VERSION,
        session,
    })
    .unwrap();
    for pointer in [
        "/session/time_trial/result/updates",
        "/session/time_trial/result/purchase",
        "/session/time_trial/result/purchase/confirming",
    ] {
        let mut missing = current.clone();
        let (parent, field) = pointer.rsplit_once('/').unwrap();
        missing
            .pointer_mut(parent)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            cli::decode_save(&serde_json::to_vec(&missing).unwrap()).is_err(),
            "{pointer}"
        );
    }
    let mut reopened = cli::decode_save(&serde_json::to_vec(&current).unwrap()).unwrap();
    for _ in 0..30 {
        reopened.step(&[]);
    }
    assert_eq!(
        reopened
            .time_trial
            .as_ref()
            .unwrap()
            .result
            .as_ref()
            .unwrap()
            .updates,
        30
    );
    reopened.validate().unwrap();
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
    let mut old_hatch = historical_envelope(cli::ProjectSave {
        format_version: 2,
        session: session.clone(),
    });
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
    let mut old_board = historical_envelope(cli::ProjectSave {
        format_version: 2,
        session: resumed,
    });
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
            .first()
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
    let mut legacy = historical_envelope(cli::ProjectSave {
        format_version: 3,
        session,
    });
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
        test_speed: 1,
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
    let mut old = historical_envelope(cli::ProjectSave {
        format_version: 4,
        session,
    });
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
    assert_eq!(
        wave.plan.expected(),
        turbofish_deluxe::invasion::EncounterKind::Single(
            turbofish_deluxe::alien::SylvesterKind::Strong
        )
    );
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
    wave.actors = vec![turbofish_deluxe::alien::WeakSylvester::spawn_kind(
        turbofish_deluxe::alien::SylvesterKind::Strong,
        99,
        100,
        120,
        1,
        1,
    )];
    wave.battle_active = true;
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
        .remove("plan");
    assert!(cli::decode_save(&serde_json::to_vec(&missing_wave_kind).unwrap()).is_err());
    let mut missing_actor_kind = modern.clone();
    missing_actor_kind["session"]["board"]["invasion"]["actors"][0]
        .as_object_mut()
        .unwrap()
        .remove("kind");
    assert!(cli::decode_save(&serde_json::to_vec(&missing_actor_kind).unwrap()).is_err());
    let mut wrong_actor_kind = modern;
    wrong_actor_kind["session"]["board"]["invasion"]["actors"][0]["kind"] =
        serde_json::json!("Weak");
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
    let modern = historical_envelope(cli::ProjectSave {
        format_version: 6,
        session,
    });
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
    let modern_board = historical_envelope(cli::ProjectSave {
        format_version: 6,
        session: board_session,
    });
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
    let mut legacy = historical_envelope(cli::ProjectSave {
        format_version: 5,
        session,
    });
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
        test_speed: 1,
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
    assert_eq!(
        wave.plan.expected(),
        turbofish_deluxe::invasion::EncounterKind::Single(
            turbofish_deluxe::alien::SylvesterKind::Balrog
        )
    );
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
        .dead_aliens = vec![DeadAlienEffect {
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
    }];
    let mut legacy = historical_envelope(cli::ProjectSave {
        format_version: 5,
        session,
    });
    legacy["session"]["board"]["invasion"]["dead_aliens"][0]
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
            .dead_aliens
            .first()
            .unwrap()
            .kind,
        SylvesterKind::Strong
    );
    let mut modern = historical_envelope(cli::ProjectSave {
        format_version: 6,
        session: migrated,
    });
    modern["session"]["board"]["invasion"]["dead_aliens"][0]
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
