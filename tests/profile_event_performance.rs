//! Contract regressions and an opt-in, offline #292 hot-path measurement.

use sha2::{Digest, Sha256};
use std::{
    fs,
    hint::black_box,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};
use turbofish_deluxe::{
    adventure::GameMode,
    highscores::ScoreKind,
    profiles::Profiles,
    sim::{Action, Event},
};

const SEED: u64 = 0xa292;

struct FixtureDirectory(PathBuf);

impl FixtureDirectory {
    fn new() -> Self {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(".scratch/performance/292/fixtures")
            .join(format!(
                "{}-{}",
                std::process::id(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> PathBuf {
        self.0.join("profiles.json")
    }
}

impl Drop for FixtureDirectory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn stage(tank: u8, level: u8, seconds: u64, survivors: Option<u32>) -> Event {
    Event::StageResultRecorded {
        tick: 1,
        tank,
        level,
        seconds,
        survivors,
        settled_coin_ids: vec![],
        settled_amount: 0,
        final_balance: 0,
        personal_best_seconds: seconds,
    }
}

fn irrelevant_events() -> [Event; 3] {
    [
        Event::FoodDropped {
            tick: 1,
            food_id: 10,
            balance: 100,
            potion: false,
        },
        Event::VirtualBubbulatorPulsed { tick: 1 },
        Event::ChallengeShellsCredited {
            tick: 1,
            amount: 100,
            shell_balance: 100,
        },
    ]
}

fn bytes(profiles: &Profiles) -> Vec<u8> {
    serde_json::to_vec(profiles).unwrap()
}

fn digest(profiles: &Profiles) -> String {
    Sha256::digest(bytes(profiles))
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn record_events_empty_and_irrelevant_batches_preserve_memory_and_skip_disk() {
    let directory = FixtureDirectory::new();
    let blocker = directory.0.join("file-as-parent");
    fs::write(&blocker, b"untouched").unwrap();
    let path = blocker.join("profiles.json");
    let mut profiles = populated_profiles();
    let before = bytes(&profiles);
    assert!(!profiles.record_events(&path, &[]).unwrap());
    assert!(!profiles.record_events(&path, &irrelevant_events()).unwrap());
    assert_eq!(bytes(&profiles), before);
    assert_eq!(fs::read(blocker).unwrap(), b"untouched");
    assert!(!directory.path().exists());
}

#[test]
fn record_events_missing_current_still_ignores_score_errors() {
    let directory = FixtureDirectory::new();
    let mut profiles = Profiles::new("Alice", SEED).unwrap();
    profiles.entries.clear();
    profiles.current_id = None;
    profiles.validate().unwrap();
    let before = bytes(&profiles);
    assert!(
        !profiles
            .record_events(
                &directory.path(),
                &[Event::TimeTrialExpired {
                    tick: 1,
                    tank: 0,
                    score: -1,
                }],
            )
            .unwrap()
    );
    assert_eq!(bytes(&profiles), before);
    assert!(!directory.path().exists());
}

#[test]
fn record_events_score_mappings_names_and_table_duplicates_remain_durable() {
    let directory = FixtureDirectory::new();
    let path = directory.path();
    let mut profiles = Profiles::new("Alice", SEED).unwrap();
    let events = [
        irrelevant_events()[0].clone(),
        stage(1, 1, 42, None),
        Event::TimeTrialExpired {
            tick: 1,
            tank: 2,
            score: 700,
        },
        Event::ChallengeCompleted {
            tick: 1,
            tank: 3,
            seconds: 80,
        },
        stage(5, 1, 999, Some(17)),
    ];
    assert!(profiles.record_events(&path, &events).unwrap());
    for (kind, tank, level, expected) in [
        (ScoreKind::Adventure, 1, Some(1), 42),
        (ScoreKind::TimeTrial, 2, None, 700),
        (ScoreKind::Challenge, 3, None, 80),
        (ScoreKind::Adventure, 5, Some(1), 17),
    ] {
        let rows = profiles.global_scores.rows(kind, tank, level).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "Alice");
        assert_eq!(rows[0].score, expected);
    }
    let saved = fs::read(&path).unwrap();
    assert_eq!(bytes(&Profiles::decode(&saved).unwrap()), bytes(&profiles));
    assert!(
        !profiles
            .record_events(&path, &[stage(1, 1, 42, None)])
            .unwrap()
    );
    assert_eq!(fs::read(&path).unwrap(), saved);

    // Mode tables accept repeated tied rows while capacity remains; Adventure
    // stages have one row. Preserve these existing, different contracts.
    let trial = Event::TimeTrialExpired {
        tick: 1,
        tank: 2,
        score: 700,
    };
    assert!(profiles.record_events(&path, &[trial]).unwrap());
    assert_eq!(
        profiles
            .global_scores
            .rows(ScoreKind::TimeTrial, 2, None)
            .unwrap()
            .len(),
        2
    );
    profiles.current_mut().unwrap().name = "Renamed".into();
    assert!(
        profiles
            .record_events(&path, &[stage(1, 1, 41, None)])
            .unwrap()
    );
    assert_eq!(
        profiles
            .global_scores
            .rows(ScoreKind::Adventure, 1, Some(1))
            .unwrap()[0]
            .name,
        "Renamed"
    );
    assert_eq!(
        profiles
            .global_scores
            .rows(ScoreKind::TimeTrial, 2, None)
            .unwrap()[0]
            .name,
        "Alice"
    );
    assert_eq!(
        bytes(&Profiles::decode(&fs::read(path).unwrap()).unwrap()),
        bytes(&profiles)
    );
}

#[test]
fn record_events_overflow_missing_survivors_and_nonqualifying_ties_skip_publication() {
    let directory = FixtureDirectory::new();
    let mut profiles = populated_profiles();
    let before = bytes(&profiles);
    let events = [
        stage(1, 1, i32::MAX as u64 + 1, None),
        stage(5, 1, 1, None),
        stage(5, 1, 1, Some(u32::MAX)),
        stage(1, 1, 100, None),
        stage(5, 1, 1, Some(18)),
        Event::TimeTrialExpired {
            tick: 1,
            tank: 1,
            score: 100,
        },
        Event::ChallengeCompleted {
            tick: 1,
            tank: 1,
            seconds: 90,
        },
    ];
    assert!(!profiles.record_events(&directory.path(), &events).unwrap());
    assert_eq!(bytes(&profiles), before);
    assert!(!directory.path().exists());
}

#[test]
fn record_events_invalid_score_after_valid_score_rolls_back_the_entire_batch() {
    let directory = FixtureDirectory::new();
    let mut profiles = Profiles::new("Alice", SEED).unwrap();
    let before = bytes(&profiles);
    for invalid in [
        Event::TimeTrialExpired {
            tick: 1,
            tank: 1,
            score: -1,
        },
        Event::ChallengeCompleted {
            tick: 1,
            tank: 0,
            seconds: 30,
        },
        stage(1, 6, 30, None),
    ] {
        assert!(
            profiles
                .record_events(&directory.path(), &[stage(1, 1, 42, None), invalid])
                .is_err()
        );
        assert_eq!(bytes(&profiles), before);
        assert!(!directory.path().exists());
    }
}

#[test]
fn record_events_failed_save_keeps_the_previous_in_memory_manager() {
    let directory = FixtureDirectory::new();
    let blocker = directory.0.join("file-as-parent");
    fs::write(&blocker, b"untouched").unwrap();
    let mut profiles = populated_profiles();
    let before = bytes(&profiles);
    assert!(
        profiles
            .record_events(&blocker.join("profiles.json"), &[stage(1, 1, 99, None)])
            .is_err()
    );
    assert_eq!(bytes(&profiles), before);
    assert_eq!(fs::read(blocker).unwrap(), b"untouched");
}

fn populated_profiles() -> Profiles {
    let mut profiles = Profiles::new("Player00", SEED).unwrap();
    for index in 1..8 {
        let mut entry = Profiles::new(&format!("Player{index:02}"), SEED + index)
            .unwrap()
            .entries
            .remove(0);
        entry.id = index + 1;
        profiles.entries.push(entry);
    }
    profiles.next_id = 9;
    for entry in &mut profiles.entries {
        entry.screensaver_generation = format!("{:032x}", entry.id);
        let board = entry.session.board.as_mut().unwrap();
        board.balance = 10_000;
        board.guppy_unlocked = true;
        for _ in board.fish.len()..32 {
            assert!(
                board
                    .apply(Action::BuyGuppy)
                    .iter()
                    .any(|event| matches!(event, Event::GuppyBought { .. }))
            );
        }
        assert_eq!(board.fish.len(), 32);
        entry
            .session
            .select_mode(GameMode::VirtualTank, 86_400, 86_400)
            .unwrap();
        assert!(entry.session.retained_adventure.is_some());
    }
    for tank in 1..=4 {
        for level in 1..=5 {
            profiles
                .global_scores
                .record(ScoreKind::Adventure, tank, Some(level), "Previous", 100)
                .unwrap();
        }
        for (trial, challenge) in [(500, 50), (400, 60), (300, 70), (200, 80), (100, 90)] {
            profiles
                .global_scores
                .record(ScoreKind::TimeTrial, tank, None, "Previous", trial)
                .unwrap();
            profiles
                .global_scores
                .record(ScoreKind::Challenge, tank, None, "Previous", challenge)
                .unwrap();
        }
    }
    for survivors in [20, 19, 18] {
        profiles
            .global_scores
            .record(ScoreKind::Adventure, 5, Some(1), "Previous", survivors)
            .unwrap();
    }
    profiles.validate().unwrap();
    let encoded = bytes(&profiles);
    assert_eq!(bytes(&Profiles::decode(&encoded).unwrap()), encoded);
    profiles
}

#[test]
#[ignore = "offline timing only; requires a serial measurement lease"]
fn profile_event_hot_path() {
    let directory = FixtureDirectory::new();
    let path = directory.path();
    let fixture = populated_profiles();
    let input_digest = digest(&fixture);
    for (case, events) in [
        ("empty", Vec::new()),
        ("irrelevant", irrelevant_events().to_vec()),
        ("duplicate", vec![stage(1, 1, 100, None)]),
    ] {
        let mut profiles = fixture.clone();
        for _ in 0..256 {
            assert!(!profiles.record_events(&path, &events).unwrap());
        }
        let mut samples = Vec::new();
        for _ in 0..9 {
            let started = Instant::now();
            for _ in 0..10_000 {
                assert!(
                    !black_box(&mut profiles)
                        .record_events(black_box(&path), black_box(&events))
                        .unwrap()
                );
            }
            samples.push(started.elapsed().as_nanos() as f64 / 10_000.0);
        }
        let output_digest = digest(&profiles);
        assert_eq!(output_digest, input_digest);
        assert!(!path.exists());
        samples.sort_by(f64::total_cmp);
        println!(
            "A292 case={case} input={input_digest} output={output_digest} min_ns={:.3} median_ns={:.3} max_ns={:.3}",
            samples[0], samples[4], samples[8]
        );
    }
}
