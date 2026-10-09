//! Adventure progression around the per-stage board. The first-stage transition
//! and rescue rules follow WinFish f919b3c (secondary source evidence); their
//! timing and presentation have not been checked against the installed build.

use serde::{Deserialize, Serialize};

pub use crate::sim::PetKind;
use crate::sim::{
    Action, AdventureState, EGG_PRICE, Event, Rejection, SECOND_STAGE_EGG_PRICE, TICK_MS,
};

const HATCH_OPEN_CHECK: u32 = 141;
const HATCH_READY_CHECK: u32 = 170;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdventureProgress {
    pub tank: u8,
    pub level: u8,
    pub unlocked_pets: Vec<PetKind>,
    /// Missing on old format-two saves: their completed board was discarded,
    /// so its active time cannot be reconstructed from the session clock.
    #[serde(default)]
    pub first_stage_best_seconds: Option<u64>,
}

impl AdventureProgress {
    pub fn has_pet(&self, pet: PetKind) -> bool {
        self.unlocked_pets.contains(&pet)
    }

    fn record_first_stage_time(&mut self, seconds: u64) -> u64 {
        let best = self
            .first_stage_best_seconds
            .map_or(seconds, |previous| previous.min(seconds));
        self.first_stage_best_seconds = Some(best);
        best
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdventurePhase {
    Playing,
    FirstTankRescue,
    Hatch { pet: PetKind, updates: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MigrationError {
    UnsupportedBoard,
    InconsistentEggs,
    InvalidBoard,
}

/// `board` is absent during hatch because the completed stage must not be
/// resumed. `ticks` counts session updates; each board's `tick` starts at zero.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdventureSession {
    pub progress: AdventureProgress,
    pub board: Option<AdventureState>,
    pub phase: AdventurePhase,
    pub ticks: u64,
    next_seed: u64,
    #[serde(skip)]
    hatch_held: bool,
}

impl AdventureSession {
    pub fn new(seed: u64) -> Self {
        let board = AdventureState::new_adventure(seed);
        let next_seed = board.transition_seed();
        Self {
            progress: AdventureProgress {
                tank: 1,
                level: 1,
                unlocked_pets: Vec::new(),
                first_stage_best_seconds: None,
            },
            board: Some(board),
            phase: AdventurePhase::Playing,
            ticks: 0,
            next_seed,
            hatch_held: false,
        }
    }

    /// Upgrade a version-one save containing only the first board. A completed
    /// board is converted to durable progress and a pending Stinky hatch.
    pub fn from_legacy_board(mut board: AdventureState) -> Result<Self, MigrationError> {
        if (board.tank, board.level) != (1, 1) {
            return Err(MigrationError::UnsupportedBoard);
        }
        if board.eggs > 3 || board.victory != (board.eggs == 3) {
            return Err(MigrationError::InconsistentEggs);
        }
        if board.egg_price != EGG_PRICE || !board.pets.is_empty() || board.stinky.is_some() {
            return Err(MigrationError::InvalidBoard);
        }
        let next_seed = board.transition_seed();
        let ticks = board.tick;
        if board.victory {
            let seconds = elapsed_seconds(&board);
            // Terminal settlement is not carried into the next tank. Retire
            // this board once, just as for an ordinary first-stage completion.
            settle_collecting_coins(&mut board);
            let session = Self {
                progress: AdventureProgress {
                    tank: 1,
                    level: 2,
                    unlocked_pets: vec![PetKind::Stinky],
                    first_stage_best_seconds: Some(seconds),
                },
                board: None,
                phase: AdventurePhase::Hatch {
                    pet: PetKind::Stinky,
                    updates: 0,
                },
                ticks,
                next_seed,
                hatch_held: false,
            };
            session
                .validate()
                .map_err(|_| MigrationError::InvalidBoard)?;
            return Ok(session);
        }
        let session = Self {
            progress: AdventureProgress {
                tank: 1,
                level: 1,
                unlocked_pets: Vec::new(),
                first_stage_best_seconds: None,
            },
            board: Some(board),
            phase: AdventurePhase::Playing,
            ticks,
            next_seed,
            hatch_held: false,
        };
        session
            .validate()
            .map_err(|_| MigrationError::InvalidBoard)?;
        Ok(session)
    }

    /// Reject a decoded session whose board, profile, and screen disagree.
    pub fn validate(&self) -> Result<(), String> {
        if self.progress.tank != 1 || !(1..=2).contains(&self.progress.level) {
            return Err("unsupported Adventure progress".into());
        }
        if self.progress.unlocked_pets.len() > 1 {
            return Err("duplicate or unsupported pet unlock".into());
        }
        if self.progress.level == 1 && self.progress.has_pet(PetKind::Stinky) {
            return Err("Stinky unlocked before first-stage completion".into());
        }
        if self.progress.level == 2 && !self.progress.has_pet(PetKind::Stinky) {
            return Err("missing first-stage pet reward".into());
        }
        match (&self.phase, &self.board) {
            (AdventurePhase::Playing, Some(board)) => {
                if (board.tank, board.level) != (self.progress.tank, self.progress.level)
                    || board.victory
                    || board.eggs >= 3
                    || board.tick > self.ticks
                {
                    return Err("playing board disagrees with Adventure progress".into());
                }
                if board.level == 1 && (!board.pets.is_empty() || board.stinky.is_some()) {
                    return Err("invalid first-stage board".into());
                }
                let expected_egg_price = if board.level == 1 {
                    EGG_PRICE
                } else {
                    SECOND_STAGE_EGG_PRICE
                };
                if board.egg_price != expected_egg_price {
                    return Err("wrong egg price for Adventure stage".into());
                }
                if board.level == 2
                    && (board.pets.as_slice() != [PetKind::Stinky] || board.stinky.is_none())
                {
                    return Err("second-stage roster and live Stinky disagree".into());
                }
                if let Some(stinky) = &board.stinky {
                    stinky.validate()?;
                }
                Ok(())
            }
            (AdventurePhase::FirstTankRescue, Some(board)) => {
                if (board.tank, board.level) != (1, 1)
                    || self.progress.level != 1
                    || board.victory
                    || board.has_live_fish()
                    || board.tick > self.ticks
                    || board.egg_price != EGG_PRICE
                    || board.eggs >= 3
                    || !board.pets.is_empty()
                    || board.stinky.is_some()
                {
                    return Err("rescue board disagrees with Adventure progress".into());
                }
                Ok(())
            }
            (AdventurePhase::Hatch { pet, .. }, None)
                if self.progress.level == 2 && self.progress.has_pet(*pet) =>
            {
                Ok(())
            }
            _ => Err("Adventure phase and board disagree".into()),
        }
    }

    /// Apply ordered inputs without advancing either clock. This is also the
    /// save boundary: already accepted actions can be persisted immediately.
    pub fn apply_actions(&mut self, actions: &[Action]) -> Vec<Event> {
        self.apply_actions_inner(actions).0
    }

    fn apply_actions_inner(&mut self, actions: &[Action]) -> (Vec<Event>, bool, bool) {
        let mut events = Vec::new();
        let mut entered_playing = false;
        let mut entered_hatch = false;
        for action in actions {
            match self.phase.clone() {
                AdventurePhase::Playing => {
                    let Some(board) = self.board.as_mut() else {
                        continue;
                    };
                    events.extend(board.apply(action.clone()));
                    if board.victory {
                        self.finish_first_stage(&mut events);
                        entered_hatch = true;
                    }
                }
                AdventurePhase::FirstTankRescue => {
                    let board = self.board.as_mut().expect("rescue retains its board");
                    events.push(Event::Action {
                        tick: board.tick,
                        action: action.clone(),
                    });
                    if *action == Action::Continue {
                        let fish_id = board.spawn_bought_guppy();
                        self.phase = AdventurePhase::Playing;
                        events.push(Event::RescueGuppyGranted {
                            tick: self.ticks,
                            fish_id,
                        });
                    } else {
                        events.push(Event::Rejected {
                            tick: board.tick,
                            reason: Rejection::Locked,
                        });
                    }
                }
                AdventurePhase::Hatch { updates, .. } => {
                    events.push(Event::Action {
                        tick: self.ticks,
                        action: action.clone(),
                    });
                    if let Action::HatchHold { down } = action {
                        self.hatch_held = *down;
                    } else if *action == Action::Continue && updates > HATCH_READY_CHECK {
                        let board = AdventureState::new_second_stage(self.next_seed);
                        self.next_seed = board.transition_seed();
                        self.board = Some(board);
                        self.phase = AdventurePhase::Playing;
                        self.hatch_held = false;
                        entered_playing = true;
                        events.push(Event::StageStarted {
                            tick: self.ticks,
                            tank: 1,
                            level: 2,
                        });
                    } else {
                        events.push(Event::Rejected {
                            tick: self.ticks,
                            reason: Rejection::Locked,
                        });
                    }
                }
            }
        }
        (events, entered_playing, entered_hatch)
    }

    /// One ordered input batch followed by one 28 ms update. Hatch updates and
    /// rescue pauses are session time; they do not advance an absent/paused board.
    pub fn step(&mut self, actions: &[Action]) -> Vec<Event> {
        self.ticks += 1;
        let (mut events, entered_playing, entered_hatch) = self.apply_actions_inner(actions);
        match &mut self.phase {
            AdventurePhase::Playing if !entered_playing => {
                if let Some(board) = &mut self.board {
                    if board.level == 1 && !board.has_live_fish() {
                        // Board::Update increments the active clock before it
                        // discovers the empty live list and pauses the widgets.
                        board.advance_board_clock();
                        self.phase = AdventurePhase::FirstTankRescue;
                        events.push(Event::FirstTankRescueStarted { tick: self.ticks });
                    } else {
                        events.extend(board.tick());
                    }
                }
            }
            AdventurePhase::Hatch { pet, updates } if !entered_hatch => {
                // HatchScreen::Update tests the old counter, then increments it.
                // The sound is tested at 141; Continue becomes visible at 170.
                if self.hatch_held && *updates <= 139 {
                    *updates = 140;
                }
                if *updates == HATCH_OPEN_CHECK {
                    events.push(Event::HatchOpened {
                        tick: self.ticks,
                        pet: *pet,
                    });
                }
                if *updates == HATCH_READY_CHECK {
                    events.push(Event::HatchReady {
                        tick: self.ticks,
                        pet: *pet,
                    });
                }
                *updates += 1;
            }
            AdventurePhase::Playing
            | AdventurePhase::FirstTankRescue
            | AdventurePhase::Hatch { .. } => {}
        }
        events
    }

    fn finish_first_stage(&mut self, events: &mut Vec<Event>) {
        let mut board = self.board.take().expect("completion retains its board");
        let seconds = elapsed_seconds(&board);
        let (settled_coin_ids, settled_amount) = settle_collecting_coins(&mut board);
        let personal_best_seconds = self.progress.record_first_stage_time(seconds);
        events.push(Event::StageResultRecorded {
            tick: self.ticks,
            tank: board.tank,
            level: board.level,
            seconds,
            settled_coin_ids,
            settled_amount,
            final_balance: board.balance,
            personal_best_seconds,
        });
        self.next_seed = board.transition_seed();
        self.progress.level = 2;
        self.progress.unlocked_pets.push(PetKind::Stinky);
        self.phase = AdventurePhase::Hatch {
            pet: PetKind::Stinky,
            updates: 0,
        };
        self.hatch_held = false;
        events.push(Event::PetUnlocked {
            tick: self.ticks,
            pet: PetKind::Stinky,
        });
        events.push(Event::HatchStarted {
            tick: self.ticks,
            pet: PetKind::Stinky,
        });
    }
}

fn elapsed_seconds(board: &AdventureState) -> u64 {
    board.tick.saturating_mul(u64::from(TICK_MS)) / 1000
}

/// W1 Unk04/Unk10 settle only player-claimed objects before retiring the board.
/// Remove those coins in Rust as well, so the helper cannot credit them twice.
/// Ordinary coin values are positive; cap money exactly as the source does.
fn settle_collecting_coins(board: &mut AdventureState) -> (Vec<u64>, i32) {
    let mut ids = Vec::new();
    let mut amount = 0;
    board.coins.retain(|coin| {
        if coin.collecting {
            ids.push(coin.id);
            amount += coin.kind.value();
            false
        } else {
            true
        }
    });
    board.balance = (board.balance + amount).clamp(0, 9_999_999);
    (ids, amount)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{Coin, CoinKind, SECOND_STAGE_EGG_PRICE};

    #[test]
    fn first_stage_score_uses_active_board_ticks_and_retains_strict_best() {
        for (ticks, expected) in [(35, 0), (36, 1), (250, 7)] {
            let mut board = AdventureState::new_adventure(42);
            board.tick = ticks;
            assert_eq!(elapsed_seconds(&board), expected);
        }
        let mut progress = AdventureSession::new(42).progress;
        assert_eq!(progress.record_first_stage_time(12), 12);
        assert_eq!(progress.record_first_stage_time(10), 10);
        assert_eq!(progress.record_first_stage_time(10), 10);
        assert_eq!(progress.record_first_stage_time(11), 10);
        assert_eq!(progress.record_first_stage_time(0), 0);
        assert_eq!(progress.first_stage_best_seconds, Some(0));
    }

    #[test]
    fn completion_silently_settles_only_claimed_coins_and_records_once() {
        let mut session = AdventureSession::new(42);
        for _ in 0..250 {
            session.step(&[]);
        }
        let board = session.board.as_mut().unwrap();
        board.egg_unlocked = true;
        board.eggs = 2;
        board.coins = [
            (41, CoinKind::Silver, true),
            (42, CoinKind::Gold, true),
            (43, CoinKind::Gold, false),
        ]
        .into_iter()
        .map(|(id, kind, collecting)| Coin {
            id,
            kind,
            collecting,
            x: 200.0,
            y: 200.0,
            frame: 0,
            bottom_ticks: 0,
            fade_ticks: 0,
        })
        .collect();
        let events = session.apply_actions(&[Action::BuyEgg, Action::BuyEgg]);
        let results: Vec<_> = events
            .iter()
            .filter(|event| matches!(event, Event::StageResultRecorded { .. }))
            .collect();
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0], Event::StageResultRecorded {
            tank: 1, level: 1, seconds: 7, settled_coin_ids, settled_amount: 50,
            final_balance: 100, personal_best_seconds: 7, ..
        } if settled_coin_ids == &[41, 42]));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, Event::CoinCredited { .. }))
        );
        assert_eq!(session.progress.first_stage_best_seconds, Some(7));
        assert!(session.board.is_none());
        for _ in 0..200 {
            assert!(
                !session
                    .step(&[])
                    .iter()
                    .any(|event| matches!(event, Event::StageResultRecorded { .. }))
            );
        }
        assert_eq!(session.progress.first_stage_best_seconds, Some(7));
    }

    #[test]
    fn third_egg_can_use_claimed_in_flight_funds_before_silent_settlement() {
        let mut session = AdventureSession::new(42);
        let board = session.board.as_mut().unwrap();
        board.balance = 135;
        board.eggs = 2;
        board.egg_unlocked = true;
        board.coins.push(Coin {
            id: 41,
            x: 200.0,
            y: 200.0,
            kind: CoinKind::Silver,
            frame: 0,
            collecting: true,
            bottom_ticks: 0,
            fade_ticks: 0,
        });
        let events = session.apply_actions(&[Action::BuyEgg]);
        assert!(events.iter().any(|event| matches!(
            event,
            Event::EggBought {
                pieces: 3,
                balance: -15,
                ..
            }
        )));
        assert!(events.iter().any(|event| matches!(event,
            Event::StageResultRecorded {
                settled_coin_ids, settled_amount: 15, final_balance: 0, ..
            } if settled_coin_ids == &[41])));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, Event::CoinCredited { .. }))
        );
        assert!(session.board.is_none());
        assert_eq!(
            session.phase,
            AdventurePhase::Hatch {
                pet: PetKind::Stinky,
                updates: 0
            }
        );
    }

    #[test]
    fn rescue_detection_counts_then_pause_and_no_time_input_preserve_score_clock() {
        let mut session = AdventureSession::new(42);
        session.ticks = 35;
        let board = session.board.as_mut().unwrap();
        board.tick = 35;
        board.fish.clear();
        session.step(&[]);
        assert_eq!(session.phase, AdventurePhase::FirstTankRescue);
        assert_eq!(session.board.as_ref().unwrap().tick, 36);
        assert_eq!(elapsed_seconds(session.board.as_ref().unwrap()), 1);
        for _ in 0..100 {
            session.step(&[]);
        }
        assert_eq!(session.board.as_ref().unwrap().tick, 36);
        session.apply_actions(&[Action::Continue]);
        assert_eq!(session.board.as_ref().unwrap().tick, 36);
        assert_eq!(session.phase, AdventurePhase::Playing);
        session.step(&[]);
        assert_eq!(session.board.as_ref().unwrap().tick, 37);
        assert!(session.validate().is_ok());
    }

    fn buy_three_eggs() -> AdventureSession {
        let mut session = AdventureSession::new(0x45a1);
        let board = session.board.as_mut().unwrap();
        board.egg_unlocked = true;
        board.balance = 450;
        for _ in 0..3 {
            session.step(&[Action::BuyEgg]);
        }
        session
    }

    #[test]
    fn third_egg_commits_progress_and_discards_completed_board() {
        let mut session = AdventureSession::new(0x45a1);
        session.board.as_mut().unwrap().egg_unlocked = true;
        session.board.as_mut().unwrap().balance = 450;
        for pieces in 1..=2 {
            let events = session.step(&[Action::BuyEgg]);
            assert!(events.iter().any(
                |event| matches!(event, Event::EggBought { pieces: count, .. } if *count == pieces)
            ));
            assert_eq!((session.progress.tank, session.progress.level), (1, 1));
            assert!(session.board.is_some());
        }
        let events = session.step(&[Action::BuyEgg]);
        assert!(events.iter().any(|event| matches!(
            event,
            Event::LevelCompleted {
                next_tank: 1,
                next_level: 2,
                ..
            }
        )));
        assert!(events.iter().any(|event| matches!(
            event,
            Event::PetUnlocked {
                pet: PetKind::Stinky,
                ..
            }
        )));
        assert!(events.iter().any(|event| matches!(
            event,
            Event::HatchStarted {
                pet: PetKind::Stinky,
                ..
            }
        )));
        assert!(session.board.is_none());
        assert_eq!((session.progress.tank, session.progress.level), (1, 2));
        assert!(session.progress.has_pet(PetKind::Stinky));
        assert_eq!(
            session.phase,
            AdventurePhase::Hatch {
                pet: PetKind::Stinky,
                updates: 0
            }
        );
        assert!(session.validate().is_ok());
    }

    #[test]
    fn hatch_counter_checks_before_increment_then_starts_fresh_stage() {
        let mut session = buy_three_eggs();
        let early = session.step(&[Action::Continue]);
        assert!(early.iter().any(|event| matches!(
            event,
            Event::Rejected {
                reason: Rejection::Locked,
                ..
            }
        )));
        assert!(session.board.is_none());
        for _ in 1..=140 {
            session.step(&[]);
        }
        let opened = session.step(&[]);
        assert!(opened.iter().any(|event| matches!(
            event,
            Event::HatchOpened {
                pet: PetKind::Stinky,
                ..
            }
        )));
        for _ in 143..=170 {
            session.step(&[]);
        }
        let ready = session.step(&[Action::Continue]);
        assert!(
            !ready
                .iter()
                .any(|event| matches!(event, Event::StageStarted { .. }))
        );
        assert!(ready.iter().any(|event| matches!(
            event,
            Event::HatchReady {
                pet: PetKind::Stinky,
                ..
            }
        )));
        // The update whose old counter is 170 exposes Continue, then increments.
        assert!(matches!(
            session.phase,
            AdventurePhase::Hatch { updates: 171, .. }
        ));
        let started = session.step(&[Action::Continue]);
        assert!(started.iter().any(|event| matches!(
            event,
            Event::StageStarted {
                tank: 1,
                level: 2,
                ..
            }
        )));
        let board = session.board.as_ref().unwrap();
        assert_eq!(
            (board.tank, board.level, board.tick, board.balance),
            (1, 2, 0, 200)
        );
        assert_eq!(board.egg_price, SECOND_STAGE_EGG_PRICE);
        assert_eq!(board.pets, vec![PetKind::Stinky]);
        assert_eq!(board.fish.len(), 2);
        assert!(
            board
                .fish
                .iter()
                .all(|fish| !fish.beginner && fish.food_ate == 2)
        );
        assert!(session.validate().is_ok());
    }

    #[test]
    fn last_live_fish_triggers_one_free_rescue_even_with_dead_visuals() {
        let mut session = AdventureSession::new(0x73);
        let board = session.board.as_mut().unwrap();
        board.eggs = 1;
        board.balance = 35;
        for fish in &mut board.fish {
            fish.beginner = false;
            fish.hunger = 1;
        }
        let events = session.step(&[]);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, Event::FishDied { .. }))
                .count(),
            2
        );
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, Event::FirstTankRescueStarted { .. }))
        );
        assert_eq!(session.phase, AdventurePhase::Playing);
        assert!(session.validate().is_ok());
        let death_tick = session.board.as_ref().unwrap().tick;
        assert_eq!(session.board.as_ref().unwrap().dead_fish.len(), 2);
        let rescue = session.step(&[]);
        assert!(
            rescue
                .iter()
                .any(|event| matches!(event, Event::FirstTankRescueStarted { .. }))
        );
        assert_eq!(session.phase, AdventurePhase::FirstTankRescue);
        let paused_tick = death_tick + 1;
        assert_eq!(session.board.as_ref().unwrap().tick, paused_tick);
        assert!(
            session
                .step(&[Action::BuyGuppy])
                .iter()
                .any(|event| matches!(
                    event,
                    Event::Rejected {
                        reason: Rejection::Locked,
                        ..
                    }
                ))
        );
        assert_eq!(session.board.as_ref().unwrap().tick, paused_tick);
        let continued = session.step(&[Action::Continue]);
        assert_eq!(
            continued
                .iter()
                .filter(|event| matches!(event, Event::RescueGuppyGranted { .. }))
                .count(),
            1
        );
        let board = session.board.as_ref().unwrap();
        assert_eq!(
            (board.balance, board.eggs, board.tick),
            (35, 1, paused_tick + 1)
        );
        assert_eq!(board.fish.iter().filter(|fish| fish.alive).count(), 1);
        assert!(board.fish.iter().find(|fish| fish.alive).unwrap().beginner);
        assert!(session.validate().is_ok());

        let board = session.board.as_mut().unwrap();
        let rescued = board.fish.iter_mut().find(|fish| fish.alive).unwrap();
        rescued.beginner = false;
        rescued.hunger = 1;
        let second_loss = session.step(&[]);
        assert!(
            !second_loss
                .iter()
                .any(|event| matches!(event, Event::FirstTankRescueStarted { .. }))
        );
        let second_rescue = session.step(&[]);
        assert_eq!(
            second_rescue
                .iter()
                .filter(|event| matches!(event, Event::FirstTankRescueStarted { .. }))
                .count(),
            1
        );
        assert_eq!(session.phase, AdventurePhase::FirstTankRescue);
    }

    #[test]
    fn legacy_completion_migrates_to_hatch_and_invalid_legacy_board_is_rejected() {
        let mut completed = AdventureState::new_adventure(0x1122);
        completed.eggs = 3;
        completed.victory = true;
        let migrated = AdventureSession::from_legacy_board(completed).unwrap();
        assert!(migrated.board.is_none());
        assert!(migrated.progress.has_pet(PetKind::Stinky));
        assert!(migrated.validate().is_ok());

        let mut inconsistent = AdventureState::new_adventure(0x1122);
        inconsistent.victory = true;
        assert_eq!(
            AdventureSession::from_legacy_board(inconsistent).unwrap_err(),
            MigrationError::InconsistentEggs
        );
        let mut wrong_stage = AdventureState::new_second_stage(0x1122);
        wrong_stage.victory = false;
        assert_eq!(
            AdventureSession::from_legacy_board(wrong_stage).unwrap_err(),
            MigrationError::UnsupportedBoard
        );
    }

    #[test]
    fn accepted_inputs_can_be_saved_without_advancing_either_clock() {
        let mut session = AdventureSession::new(0x8ad);
        let board = session.board.as_mut().unwrap();
        board.egg_unlocked = true;
        board.balance = 450;
        let events = session.apply_actions(&[Action::BuyEgg, Action::BuyEgg]);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, Event::EggBought { .. }))
                .count(),
            2
        );
        assert_eq!(session.ticks, 0);
        assert_eq!(session.board.as_ref().unwrap().tick, 0);
        assert_eq!(session.board.as_ref().unwrap().eggs, 2);
        let saved = serde_json::to_vec(&session).unwrap();
        let mut restored: AdventureSession = serde_json::from_slice(&saved).unwrap();
        assert!(restored.validate().is_ok());
        let completion = restored.apply_actions(&[Action::BuyEgg]);
        assert!(
            completion
                .iter()
                .any(|event| matches!(event, Event::HatchStarted { .. }))
        );
        assert_eq!(restored.ticks, 0);
        assert_eq!(
            restored.phase,
            AdventurePhase::Hatch {
                pet: PetKind::Stinky,
                updates: 0
            }
        );
        assert!(restored.board.is_none());
    }

    #[test]
    fn rescue_save_rejects_completed_egg_count_victory_and_pet_roster() {
        let mut session = AdventureSession::new(0x2244);
        for fish in &mut session.board.as_mut().unwrap().fish {
            fish.alive = false;
        }
        assert!(session.validate().is_ok()); // Board death precedes the next Board::Update.
        session.phase = AdventurePhase::FirstTankRescue;
        assert!(session.validate().is_ok());
        session.board.as_mut().unwrap().eggs = 3;
        assert!(session.validate().is_err());
        session.board.as_mut().unwrap().eggs = 0;
        session.board.as_mut().unwrap().victory = true;
        assert!(session.validate().is_err());
        session.board.as_mut().unwrap().victory = false;
        session.board.as_mut().unwrap().pets.push(PetKind::Stinky);
        assert!(session.validate().is_err());
    }

    #[test]
    fn hatch_background_hold_skips_intro_but_never_unlocks_continue_early() {
        let mut session = buy_three_eggs();
        let hold = session.apply_actions(&[Action::HatchHold { down: true }]);
        assert_eq!(hold.len(), 1);
        assert_eq!(session.ticks, 3);
        assert_eq!(
            session.phase,
            AdventurePhase::Hatch {
                pet: PetKind::Stinky,
                updates: 0
            }
        );
        let saved = serde_json::to_string(&session).unwrap();
        assert!(!saved.contains("hatch_held"));
        let mut restored: AdventureSession = serde_json::from_str(&saved).unwrap();
        restored.step(&[]);
        assert_eq!(
            restored.phase,
            AdventurePhase::Hatch {
                pet: PetKind::Stinky,
                updates: 1
            }
        );

        session.step(&[]);
        assert_eq!(
            session.phase,
            AdventurePhase::Hatch {
                pet: PetKind::Stinky,
                updates: 141
            }
        );
        let opened = session.step(&[Action::Continue]);
        assert!(
            opened
                .iter()
                .any(|event| matches!(event, Event::HatchOpened { .. }))
        );
        assert!(opened.iter().any(|event| matches!(
            event,
            Event::Rejected {
                reason: Rejection::Locked,
                ..
            }
        )));
        assert!(session.board.is_none());
        session.apply_actions(&[Action::HatchHold { down: false }]);
        assert!(!session.hatch_held);
    }
}
