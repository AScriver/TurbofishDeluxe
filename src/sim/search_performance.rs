//! Deterministic offline measurements for Actionable #289. These ignored checks
//! measure optimized simulation code, not rendering, FPS, or retail behavior.

use super::*;
use sha2::{Digest, Sha256};
use std::{hint::black_box, time::Instant};

const SEED: u64 = 0xa289;
const SAMPLES: usize = 9;

fn fixture(fish_count: usize, coin_count: usize, note_count: usize) -> AdventureState {
    let mut board =
        AdventureState::new_tank3_first_stage(SEED, &[PetKind::Stinky, PetKind::Meryl]).unwrap();
    let template = board.fish[0].clone();
    board.fish.clear();
    for index in 0..fish_count {
        let mut fish = template.clone();
        fish.id = board.id();
        fish.x = 20.0 + ((index * 37) % 520) as f32;
        fish.y = 100.0 + ((index * 29) % 250) as f32;
        fish.hunger = 250;
        board.fish.push(fish);
    }
    board.upgrades.quantity = 9;
    board.upgrades.quality_unlocked = true;
    board.upgrades.quantity_unlocked = true;
    board.grubber_unlocked = true;
    for index in 0..9 {
        let id = board.id();
        board.food.push(Food {
            id,
            x: 40.0 + (index * 55) as f32,
            y: 200.0 + (index % 3 * 50) as f32,
            frame: 0,
            food_type: FoodType::Ordinary,
            picked_up: false,
            ineligible_ticks: 0,
            removal_ticks: 0,
            quality: 0,
            direction: 0,
            vx: 0.0,
            vy: 0.0,
            animation_period: 3,
            free_from_zorf: false,
            nimbus_rising: false,
        });
    }
    // Interleave creation IDs as coins and Meryl notes do in the real Board.
    // Their separate vectors retain registration order, while the old query
    // sorts a newly combined vector on each call.
    for index in 0..coin_count.max(note_count) {
        if index < coin_count {
            let id = board.id();
            board.coins.push(Coin {
                id,
                x: 15.25 + ((index * 43) % 525) as f64,
                y: 100.75 + ((index * 31) % 260) as f64,
                kind: CoinKind::Silver,
                frame: 0,
                animation_ticks: 0,
                hazard_age_ticks: 0,
                collecting: false,
                bottom_ticks: 0,
                fade_ticks: 0,
                penta_rising: false,
            });
        }
        if index < note_count {
            let id = board.id();
            board.notes.push(NoteState {
                id,
                x: 20 + ((index * 47) % 520) as i32,
                y: 100 + ((index * 37) % 250) as i32,
                age_ticks: 0,
            });
        }
    }
    board
        .validate()
        .expect("benchmark fixture must be admitted");
    board
}

fn digest(value: &impl serde::Serialize) -> String {
    Sha256::digest(serde_json::to_vec(value).unwrap())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn report(label: &str, mut nanos: Vec<f64>) {
    nanos.sort_by(f64::total_cmp);
    println!(
        "A289 {label} samples={} min_ns={:.1} median_ns={:.1} max_ns={:.1}",
        nanos.len(),
        nanos[0],
        nanos[nanos.len() / 2],
        nanos[nanos.len() - 1]
    );
}

#[test]
fn a289_fixture_admission() {
    fixture(20, 16, 2);
    fixture(200, 256, 32);
}

#[test]
#[ignore = "offline timing: run --release, --ignored, --nocapture, --test-threads=1"]
fn a289_query_performance() {
    for (label, coins, notes) in [("query_normal", 16, 2), ("query_crowded", 256, 32)] {
        let board = fixture(20, coins, notes);
        println!(
            "A289 {label} seed={SEED} coins={coins} notes={notes} input={}",
            digest(&board)
        );
        let mut samples = Vec::new();
        for _ in 0..SAMPLES {
            let start = Instant::now();
            for _ in 0..100_000 {
                black_box(AdventureState::stinky_target_x(
                    black_box(&board.stinky[0]),
                    black_box(&board.coins),
                    black_box(&board.notes),
                    false,
                ));
            }
            samples.push(start.elapsed().as_nanos() as f64 / 100_000.0);
        }
        report(label, samples);
    }
}

#[test]
#[ignore = "offline timing: run --release, --ignored, --nocapture, --test-threads=1"]
fn a289_board_performance() {
    for (label, fish, coins, notes) in
        [("board_normal", 20, 16, 2), ("board_crowded", 200, 256, 32)]
    {
        let input = fixture(fish, coins, notes);
        let mut output = input.clone();
        let events = output.tick();
        let mut trace = Vec::new();
        let mut continued = input.clone();
        for _ in 0..32 {
            trace.extend(continued.tick());
        }
        println!(
            "A289 {label} seed={SEED} fish={fish} food=9 coins={coins} notes={notes} input={} output={} events={} continued={} trace={}",
            digest(&input),
            digest(&output),
            digest(&events),
            digest(&continued),
            digest(&trace)
        );
        // Clone and teardown stay outside the timed region. Every measured tick
        // starts with the same populated Board rather than a drained/dead tank.
        let mut samples = Vec::new();
        for _ in 0..SAMPLES {
            let mut elapsed = std::time::Duration::ZERO;
            for _ in 0..64 {
                let mut boards: Vec<_> = (0..64).map(|_| input.clone()).collect();
                let start = Instant::now();
                for board in &mut boards {
                    black_box(black_box(board).tick());
                }
                elapsed += start.elapsed();
            }
            samples.push(elapsed.as_nanos() as f64 / 4096.0);
        }
        report(label, samples);
    }
}
