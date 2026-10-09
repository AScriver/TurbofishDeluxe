//! Adventure progression around the per-stage board. The first-stage transition
//! and rescue rules follow WinFish f919b3c (secondary source evidence); their
//! timing and presentation have not been checked against the installed build.

use crate::{
    bonus::{BonusResult, BonusState, MAX_SHELL_BALANCE},
    invasion::InvasionTip,
    niko::PearlPhase,
};
use serde::{Deserialize, Serialize};

pub use crate::sim::PetKind;
use crate::sim::{Action, AdventureState, EGG_PRICE, Event, Rejection, TICK_MS};

const HATCH_OPEN_CHECK: u32 = 141;
const HATCH_READY_CHECK: u32 = 170;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdventureProgress {
    pub tank: u8,
    pub level: u8,
    pub unlocked_pets: Vec<PetKind>,
    #[serde(default = "default_pet_capacity")]
    pub pet_capacity: u8,
    /// Committed profile flags. Candidates live in the selection phase until
    /// Continue copies them here, before any fewer-pet confirmation.
    #[serde(default)]
    pub selected_pets: Vec<PetKind>,
    #[serde(default)]
    pub shell_balance: u32,
    /// Missing on old format-two saves: their completed board was discarded,
    /// so its active time cannot be reconstructed from the session clock.
    #[serde(default)]
    pub first_stage_best_seconds: Option<u64>,
    #[serde(default)]
    pub later_stage_best_seconds: Vec<StageBestTime>,
}

fn default_pet_capacity() -> u8 {
    3
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageBestTime {
    pub tank: u8,
    pub level: u8,
    pub seconds: u64,
}

impl AdventureProgress {
    pub fn has_pet(&self, pet: PetKind) -> bool {
        self.unlocked_pets.contains(&pet)
    }

    pub fn selection_capacity(&self) -> usize {
        usize::from(self.pet_capacity).min(self.unlocked_pets.len())
    }

    fn valid_selection(&self, selected: &[PetKind]) -> bool {
        selected.len() <= self.selection_capacity()
            && self
                .unlocked_pets
                .iter()
                .copied()
                .filter(|pet| selected.contains(pet))
                .collect::<Vec<_>>()
                == selected
    }

    fn record_first_stage_time(&mut self, seconds: u64) -> u64 {
        let best = self
            .first_stage_best_seconds
            .map_or(seconds, |previous| previous.min(seconds));
        self.first_stage_best_seconds = Some(best);
        best
    }

    fn record_stage_time(&mut self, tank: u8, level: u8, seconds: u64) -> u64 {
        if (tank, level) == (1, 1) {
            return self.record_first_stage_time(seconds);
        }
        if let Some(previous) = self
            .later_stage_best_seconds
            .iter_mut()
            .find(|entry| (entry.tank, entry.level) == (tank, level))
        {
            previous.seconds = previous.seconds.min(seconds);
            previous.seconds
        } else {
            self.later_stage_best_seconds.push(StageBestTime {
                tank,
                level,
                seconds,
            });
            seconds
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AdventurePhase {
    Playing,
    FirstTankRescue,
    InvasionTutorial { tip: InvasionTip },
    GameOver { updates: u32 },
    GameSelector,
    HelpScreen,
    Hatch { pet: PetKind, updates: u32 },
    PetSelection { selected: Vec<PetKind> },
    PetSelectionConfirmation { selected: Vec<PetKind> },
    Bonus { state: BonusState },
    BonusResults { result: BonusResult },
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
                pet_capacity: default_pet_capacity(),
                selected_pets: Vec::new(),
                shell_balance: 0,
                first_stage_best_seconds: None,
                later_stage_best_seconds: Vec::new(),
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
        board.validate().map_err(|_| MigrationError::InvalidBoard)?;
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
                    pet_capacity: default_pet_capacity(),
                    selected_pets: Vec::new(),
                    shell_balance: 0,
                    first_stage_best_seconds: Some(seconds),
                    later_stage_best_seconds: Vec::new(),
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
                pet_capacity: default_pet_capacity(),
                selected_pets: Vec::new(),
                shell_balance: 0,
                first_stage_best_seconds: None,
                later_stage_best_seconds: Vec::new(),
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
        if !matches!(
            (self.progress.tank, self.progress.level),
            (1, 1..=6) | (2, 1..=6) | (3, 1..=5)
        ) || self.progress.shell_balance > MAX_SHELL_BALANCE
        {
            return Err("unsupported Adventure progress".into());
        }
        let expected_pets: &[PetKind] = match (self.progress.tank, self.progress.level) {
            (1, 1) => &[],
            (1, 2) => &[PetKind::Stinky],
            (1, 3) => &[PetKind::Stinky, PetKind::Niko],
            (1, 4) => &[PetKind::Stinky, PetKind::Niko, PetKind::Itchy],
            (1, 5) => &[
                PetKind::Stinky,
                PetKind::Niko,
                PetKind::Itchy,
                PetKind::Prego,
            ],
            (1, 6) | (2, 1) => &[
                PetKind::Stinky,
                PetKind::Niko,
                PetKind::Itchy,
                PetKind::Prego,
                PetKind::Zorf,
            ],
            (2, 2) => &[
                PetKind::Stinky,
                PetKind::Niko,
                PetKind::Itchy,
                PetKind::Prego,
                PetKind::Zorf,
                PetKind::Clyde,
            ],
            (2, 3) => &[
                PetKind::Stinky,
                PetKind::Niko,
                PetKind::Itchy,
                PetKind::Prego,
                PetKind::Zorf,
                PetKind::Clyde,
                PetKind::Vert,
            ],
            (2, 4) => &[
                PetKind::Stinky,
                PetKind::Niko,
                PetKind::Itchy,
                PetKind::Prego,
                PetKind::Zorf,
                PetKind::Clyde,
                PetKind::Vert,
                PetKind::Rufus,
            ],
            (2, 5) => &[
                PetKind::Stinky,
                PetKind::Niko,
                PetKind::Itchy,
                PetKind::Prego,
                PetKind::Zorf,
                PetKind::Clyde,
                PetKind::Vert,
                PetKind::Rufus,
                PetKind::Meryl,
            ],
            (2, 6) | (3, 1) => &[
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
            ],
            (3, 2) => &[
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
            ],
            (3, 3) => &[
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
            ],
            (3, 4) => &[
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
            ],
            (3, 5) => &[
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
            ],
            _ => unreachable!("progress checked above"),
        };
        if self.progress.unlocked_pets != expected_pets {
            return Err("Adventure pet unlocks disagree with completed stages".into());
        }
        if self.progress.pet_capacity != 3
            || !self.progress.valid_selection(&self.progress.selected_pets)
            || (self.progress.unlocked_pets.len() < 4 && !self.progress.selected_pets.is_empty())
        {
            return Err("invalid Adventure selected pet roster or capacity".into());
        }
        if let Some(board) = &self.board
            && self.progress.unlocked_pets.len() >= 4
            && board.pets != self.progress.selected_pets
        {
            return Err("active pet roster disagrees with committed selection".into());
        }
        let mut recorded_stages = Vec::new();
        for result in &self.progress.later_stage_best_seconds {
            if !matches!(
                (result.tank, result.level),
                (1, 2..=5) | (2, 1..=5) | (3, 1..=4)
            ) || (self.progress.tank, self.progress.level) <= (result.tank, result.level)
                || recorded_stages.contains(&(result.tank, result.level))
            {
                return Err("invalid or duplicate completed-stage time".into());
            }
            recorded_stages.push((result.tank, result.level));
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
                if board
                    .invasion
                    .as_ref()
                    .is_some_and(|wave| wave.pending_modal.is_some())
                {
                    return Err("playing board has an unacknowledged invasion modal".into());
                }
                board.validate()
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
                board.validate()
            }
            (AdventurePhase::InvasionTutorial { tip }, Some(board)) => {
                if (board.tank, board.level) != (self.progress.tank, self.progress.level)
                    || board.victory
                    || board.eggs >= 3
                    || board.tick > self.ticks
                    || board.invasion.as_ref().and_then(|wave| wave.pending_modal) != Some(*tip)
                {
                    return Err("invasion tutorial and board disagree".into());
                }
                board.validate()
            }
            (AdventurePhase::GameOver { .. }, Some(board)) => {
                if (board.tank, board.level) == (1, 1)
                    || (board.tank, board.level) != (self.progress.tank, self.progress.level)
                    || board.has_live_fish()
                    || board.victory
                    || board.eggs >= 3
                    || board.tick > self.ticks
                {
                    return Err("Game Over and failed board disagree".into());
                }
                board.validate()
            }
            (AdventurePhase::Hatch { pet, .. }, None)
                if matches!(
                    (self.progress.tank, self.progress.level, *pet),
                    (1, 2, PetKind::Stinky)
                        | (1, 3, PetKind::Niko)
                        | (1, 4, PetKind::Itchy)
                        | (2, 2, PetKind::Clyde)
                        | (2, 3, PetKind::Vert)
                        | (2, 4, PetKind::Rufus)
                        | (2, 5, PetKind::Meryl)
                        | (2, 6, PetKind::Wadsworth)
                        | (3, 2, PetKind::Seymour)
                        | (3, 3, PetKind::Shrapnel)
                        | (3, 4, PetKind::Gumbo)
                        | (3, 5, PetKind::Blip)
                ) =>
            {
                Ok(())
            }
            (
                AdventurePhase::Hatch {
                    pet: PetKind::Prego,
                    ..
                },
                None,
            ) if (self.progress.tank, self.progress.level) == (1, 5) => Ok(()),
            (
                AdventurePhase::Hatch {
                    pet: PetKind::Zorf, ..
                },
                None,
            ) if (self.progress.tank, self.progress.level) == (1, 6) => Ok(()),
            (AdventurePhase::Bonus { state }, None)
                if self.progress.level == 6
                    && self.progress.tank == state.origin_tank
                    && state.tick <= self.ticks =>
            {
                state.validate()
            }
            (AdventurePhase::BonusResults { result }, None)
                if matches!(result.origin_tank, 1 | 2)
                    && (self.progress.tank, self.progress.level) == (result.origin_tank + 1, 1)
                    && self.progress.shell_balance
                        == result
                            .previous_balance
                            .saturating_add(result.earned)
                            .min(MAX_SHELL_BALANCE) =>
            {
                result.validate()
            }
            (AdventurePhase::PetSelection { selected }, None)
                if self.progress.unlocked_pets.len() >= 4
                    && self.progress.level != 6
                    && self.progress.valid_selection(selected) =>
            {
                Ok(())
            }
            (AdventurePhase::PetSelectionConfirmation { selected }, None)
                if self.progress.unlocked_pets.len() >= 4
                    && self.progress.level != 6
                    && self.progress.valid_selection(selected)
                    && selected.len() < self.progress.selection_capacity()
                    && *selected == self.progress.selected_pets =>
            {
                Ok(())
            }
            (AdventurePhase::GameSelector | AdventurePhase::HelpScreen, None) => Ok(()),
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
                        self.finish_stage(&mut events);
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
                AdventurePhase::InvasionTutorial { .. } => {
                    events.push(Event::Action {
                        tick: self.ticks,
                        action: action.clone(),
                    });
                    if *action == Action::Continue {
                        self.board
                            .as_mut()
                            .expect("tutorial retains board")
                            .invasion
                            .as_mut()
                            .expect("tutorial retains wave")
                            .acknowledge_modal();
                        self.phase = AdventurePhase::Playing;
                    } else {
                        events.push(Event::Rejected {
                            tick: self.ticks,
                            reason: Rejection::Locked,
                        });
                    }
                }
                AdventurePhase::GameOver { updates } => {
                    events.push(Event::Action {
                        tick: self.ticks,
                        action: action.clone(),
                    });
                    if *action == Action::Continue && updates > 30 {
                        let board = self
                            .board
                            .take()
                            .expect("Game Over retains failed board until dismissal");
                        self.next_seed = board.transition_seed();
                        self.phase = AdventurePhase::GameSelector;
                        events.push(Event::GameSelectorOpened { tick: self.ticks });
                    } else {
                        events.push(Event::Rejected {
                            tick: self.ticks,
                            reason: Rejection::Locked,
                        });
                    }
                }
                AdventurePhase::GameSelector => {
                    events.push(Event::Action {
                        tick: self.ticks,
                        action: action.clone(),
                    });
                    if *action == Action::PlayAdventure {
                        self.phase = AdventurePhase::HelpScreen;
                    } else {
                        events.push(Event::Rejected {
                            tick: self.ticks,
                            reason: Rejection::Locked,
                        });
                    }
                }
                AdventurePhase::HelpScreen => {
                    events.push(Event::Action {
                        tick: self.ticks,
                        action: action.clone(),
                    });
                    if *action == Action::Continue {
                        entered_playing = self.start_current_stage(&mut events);
                    } else {
                        events.push(Event::Rejected {
                            tick: self.ticks,
                            reason: Rejection::Locked,
                        });
                    }
                }
                AdventurePhase::Hatch { updates, .. } => {
                    events.push(Event::Action {
                        tick: self.ticks,
                        action: action.clone(),
                    });
                    if *action == Action::OpenMenu {
                        self.phase = AdventurePhase::GameSelector;
                        self.hatch_held = false;
                        events.push(Event::GameSelectorOpened { tick: self.ticks });
                    } else if let Action::HatchHold { down } = action {
                        self.hatch_held = *down;
                    } else if *action == Action::Continue && updates > HATCH_READY_CHECK {
                        entered_playing = self.start_current_stage(&mut events);
                        self.hatch_held = false;
                    } else {
                        events.push(Event::Rejected {
                            tick: self.ticks,
                            reason: Rejection::Locked,
                        });
                    }
                }
                AdventurePhase::Bonus { .. } => {
                    let AdventurePhase::Bonus { state } = &mut self.phase else {
                        unreachable!()
                    };
                    events.push(Event::Action {
                        tick: state.tick,
                        action: action.clone(),
                    });
                    if let Action::Click { x, y } = action {
                        let tick = state.tick;
                        events.extend(
                            state
                                .click(*x, *y)
                                .into_iter()
                                .map(|event| Event::Bonus { tick, event }),
                        );
                    } else {
                        events.push(Event::Rejected {
                            tick: state.tick,
                            reason: Rejection::Locked,
                        });
                    }
                }
                AdventurePhase::BonusResults { result } => {
                    events.push(Event::Action {
                        tick: self.ticks,
                        action: action.clone(),
                    });
                    if *action == Action::Continue && result.updates >= 30 {
                        entered_playing = self.start_current_stage(&mut events);
                    } else {
                        events.push(Event::Rejected {
                            tick: self.ticks,
                            reason: Rejection::Locked,
                        });
                    }
                }
                AdventurePhase::PetSelection { mut selected } => {
                    events.push(Event::Action {
                        tick: self.ticks,
                        action: action.clone(),
                    });
                    match action {
                        Action::TogglePet { pet } if self.progress.has_pet(*pet) => {
                            if let Some(index) = selected.iter().position(|choice| choice == pet) {
                                selected.remove(index);
                            } else if selected.len() < self.progress.selection_capacity() {
                                selected.push(*pet);
                                selected = self
                                    .progress
                                    .unlocked_pets
                                    .iter()
                                    .copied()
                                    .filter(|choice| selected.contains(choice))
                                    .collect();
                            } else {
                                events.push(Event::Rejected {
                                    tick: self.ticks,
                                    reason: Rejection::Locked,
                                });
                                continue;
                            }
                            events.push(Event::PetSelectionChanged {
                                tick: self.ticks,
                                selected: selected.clone(),
                            });
                            self.phase = AdventurePhase::PetSelection { selected };
                        }
                        Action::Continue if (self.progress.tank, self.progress.level) != (3, 5) => {
                            self.progress.selected_pets = selected.clone();
                            if selected.len() < self.progress.selection_capacity() {
                                events.push(Event::PetSelectionConfirmation {
                                    tick: self.ticks,
                                    selected: selected.clone(),
                                });
                                self.phase = AdventurePhase::PetSelectionConfirmation { selected };
                            } else {
                                events.push(Event::PetSelectionAccepted {
                                    tick: self.ticks,
                                    selected,
                                });
                                self.start_board(&mut events);
                                entered_playing = true;
                            }
                        }
                        Action::OpenMenu => {
                            self.phase = AdventurePhase::GameSelector;
                            events.push(Event::GameSelectorOpened { tick: self.ticks });
                        }
                        _ => events.push(Event::Rejected {
                            tick: self.ticks,
                            reason: Rejection::Locked,
                        }),
                    }
                }
                AdventurePhase::PetSelectionConfirmation { selected } => {
                    events.push(Event::Action {
                        tick: self.ticks,
                        action: action.clone(),
                    });
                    match action {
                        Action::ConfirmPetSelection { accept: true }
                            if (self.progress.tank, self.progress.level) != (3, 5) =>
                        {
                            events.push(Event::PetSelectionAccepted {
                                tick: self.ticks,
                                selected,
                            });
                            self.start_board(&mut events);
                            entered_playing = true;
                        }
                        Action::ConfirmPetSelection { accept: false } => {
                            self.phase = AdventurePhase::PetSelection { selected };
                        }
                        _ => events.push(Event::Rejected {
                            tick: self.ticks,
                            reason: Rejection::Locked,
                        }),
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
                    events.extend(board.begin_tick());
                    if !board.has_live_fish() {
                        // Board::Update increments the active clock before it
                        // discovers the empty live list and pauses the widgets.
                        if (board.tank, board.level) == (1, 1) {
                            self.phase = AdventurePhase::FirstTankRescue;
                            events.push(Event::FirstTankRescueStarted { tick: self.ticks });
                        } else {
                            settle_collecting_coins(board);
                            self.phase = AdventurePhase::GameOver { updates: 0 };
                            events.push(Event::GameOverStarted { tick: self.ticks });
                        }
                    } else if let Some(tip) =
                        board.invasion.as_ref().and_then(|wave| wave.pending_modal)
                    {
                        self.phase = AdventurePhase::InvasionTutorial { tip };
                    } else {
                        events.extend(board.update_objects());
                    }
                }
            }
            AdventurePhase::Bonus { state } if !entered_playing => {
                let tick = state.tick;
                let update = state.update();
                events.extend(
                    update
                        .events
                        .into_iter()
                        .map(|event| Event::Bonus { tick, event }),
                );
                if update.completed {
                    let origin_tank = state.origin_tank;
                    let earned = state.shells_earned;
                    self.next_seed = state.transition_seed();
                    let previous_balance = self.progress.shell_balance;
                    self.progress.shell_balance = previous_balance
                        .saturating_add(earned)
                        .min(MAX_SHELL_BALANCE);
                    self.progress.tank = origin_tank + 1;
                    self.progress.level = 1;
                    self.phase = AdventurePhase::BonusResults {
                        result: BonusResult {
                            origin_tank,
                            earned,
                            previous_balance,
                            updates: 0,
                        },
                    };
                    events.push(Event::BonusResultsCommitted {
                        tick: self.ticks,
                        earned,
                        previous_balance,
                        shell_balance: self.progress.shell_balance,
                    });
                }
            }
            AdventurePhase::BonusResults { result } => {
                result.updates = result.updates.saturating_add(1);
            }
            AdventurePhase::GameOver { updates } => {
                if let Some(board) = &mut self.board {
                    board.paused_board_update();
                }
                *updates = updates.saturating_add(1);
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
            | AdventurePhase::InvasionTutorial { .. }
            | AdventurePhase::GameSelector
            | AdventurePhase::HelpScreen
            | AdventurePhase::Hatch { .. }
            | AdventurePhase::Bonus { .. }
            | AdventurePhase::PetSelection { .. }
            | AdventurePhase::PetSelectionConfirmation { .. } => {}
        }
        if matches!(
            self.phase,
            AdventurePhase::FirstTankRescue | AdventurePhase::InvasionTutorial { .. }
        ) && let Some(board) = &mut self.board
        {
            // A newly opened modal already received this update's pre-pause
            // work. Subsequent modal updates keep only the pre-pause delay.
            if !events.iter().any(|event| {
                matches!(
                    event,
                    Event::FirstTankRescueStarted { .. }
                        | Event::Invasion {
                            event: crate::invasion::InvasionEvent::ModalOpened(_),
                            ..
                        }
                )
            }) {
                board.paused_board_update();
            }
        }
        events
    }

    pub fn paused_step(&mut self) {
        self.ticks += 1;
        if let Some(board) = &mut self.board {
            board.paused_board_update();
        }
    }

    fn start_current_stage(&mut self, events: &mut Vec<Event>) -> bool {
        if self.progress.level == 6 {
            self.board = None;
            self.phase = AdventurePhase::Bonus {
                state: BonusState::new_for_tank(self.next_seed, self.progress.tank)
                    .expect("supported Adventure bonus origin"),
            };
            events.push(Event::StageStarted {
                tick: self.ticks,
                tank: self.progress.tank,
                level: 6,
            });
            return true;
        }
        if self.progress.unlocked_pets.len() >= 4 {
            self.progress.selected_pets.clear();
            self.board = None;
            self.phase = AdventurePhase::PetSelection {
                selected: Vec::new(),
            };
            events.push(Event::PetSelectionOpened {
                tick: self.ticks,
                capacity: self.progress.selection_capacity() as u8,
            });
            return false;
        }
        self.start_board(events);
        true
    }

    fn start_board(&mut self, events: &mut Vec<Event>) {
        let board = match (self.progress.tank, self.progress.level) {
            (1, 1) => AdventureState::new_adventure(self.next_seed),
            (1, 2) => AdventureState::new_second_stage(self.next_seed),
            (1, 3) => AdventureState::new_third_stage(self.next_seed),
            (1, 4) => AdventureState::new_fourth_stage(self.next_seed),
            (1, 5) => AdventureState::new_fifth_stage(self.next_seed, &self.progress.selected_pets)
                .expect("session selection is validated before starting a board"),
            (2, 1) => {
                AdventureState::new_tank2_first_stage(self.next_seed, &self.progress.selected_pets)
                    .expect("session selection is validated before starting a board")
            }
            (2, 2) => {
                AdventureState::new_tank2_second_stage(self.next_seed, &self.progress.selected_pets)
                    .expect("session selection is validated before starting a board")
            }
            (2, 3) => {
                AdventureState::new_tank2_third_stage(self.next_seed, &self.progress.selected_pets)
                    .expect("session selection is validated before starting a board")
            }
            (2, 4) => {
                AdventureState::new_tank2_fourth_stage(self.next_seed, &self.progress.selected_pets)
                    .expect("session selection is validated before starting a board")
            }
            (2, 5) => {
                AdventureState::new_tank2_fifth_stage(self.next_seed, &self.progress.selected_pets)
                    .expect("session selection is validated before starting a board")
            }
            (3, 1) => {
                AdventureState::new_tank3_first_stage(self.next_seed, &self.progress.selected_pets)
                    .expect("session selection is validated before starting a board")
            }
            (3, 2) => {
                AdventureState::new_tank3_second_stage(self.next_seed, &self.progress.selected_pets)
                    .expect("session selection is validated before starting a board")
            }
            (3, 3) => {
                AdventureState::new_tank3_third_stage(self.next_seed, &self.progress.selected_pets)
                    .expect("session selection is validated before starting a board")
            }
            (3, 4) => {
                AdventureState::new_tank3_fourth_stage(self.next_seed, &self.progress.selected_pets)
                    .expect("session selection is validated before starting a board")
            }
            _ => unreachable!("supported progress validated at load"),
        };
        self.next_seed = board.transition_seed();
        self.board = Some(board);
        self.phase = AdventurePhase::Playing;
        events.push(Event::StageStarted {
            tick: self.ticks,
            tank: self.progress.tank,
            level: self.progress.level,
        });
    }

    fn finish_stage(&mut self, events: &mut Vec<Event>) {
        let mut board = self.board.take().expect("completion retains its board");
        let seconds = elapsed_seconds(&board);
        let (settled_coin_ids, settled_amount) = settle_collecting_coins(&mut board);
        let personal_best_seconds =
            self.progress
                .record_stage_time(board.tank, board.level, seconds);
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
        let pet = match (board.tank, board.level) {
            (1, 1) => PetKind::Stinky,
            (1, 2) => PetKind::Niko,
            (1, 3) => PetKind::Itchy,
            (1, 4) => PetKind::Prego,
            (1, 5) => PetKind::Zorf,
            (2, 1) => PetKind::Clyde,
            (2, 2) => PetKind::Vert,
            (2, 3) => PetKind::Rufus,
            (2, 4) => PetKind::Meryl,
            (2, 5) => PetKind::Wadsworth,
            (3, 1) => PetKind::Seymour,
            (3, 2) => PetKind::Shrapnel,
            (3, 3) => PetKind::Gumbo,
            (3, 4) => PetKind::Blip,
            _ => unreachable!("stage not yet completable"),
        };
        self.progress.level = board.level + 1;
        if !self.progress.has_pet(pet) {
            self.progress.unlocked_pets.push(pet);
        }
        self.phase = AdventurePhase::Hatch { pet, updates: 0 };
        self.hatch_held = false;
        events.push(Event::PetUnlocked {
            tick: self.ticks,
            pet,
        });
        events.push(Event::HatchStarted {
            tick: self.ticks,
            pet,
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
    board.pearls.retain(|pearl| {
        if pearl.phase == PearlPhase::Collecting {
            ids.push(pearl.id);
            amount += crate::niko::PEARL_VALUE;
            false
        } else {
            true
        }
    });
    board.larvae.retain(|larva| {
        if larva.picked_up {
            ids.push(larva.id);
            amount += crate::larva::LARVA_VALUE;
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

    fn selection_session() -> AdventureSession {
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
            selected: Vec::new(),
        };
        session
    }

    fn bonus_session() -> AdventureSession {
        let mut session = selection_session();
        session.progress.level = 6;
        session.progress.unlocked_pets.push(PetKind::Zorf);
        session.phase = AdventurePhase::Bonus {
            state: BonusState::new(42),
        };
        session.ticks = 1000;
        session
    }

    fn tank_three_session() -> AdventureSession {
        let mut session = bonus_session();
        session.progress.tank = 3;
        session.progress.level = 1;
        session.progress.unlocked_pets.extend([
            PetKind::Clyde,
            PetKind::Vert,
            PetKind::Rufus,
            PetKind::Meryl,
            PetKind::Wadsworth,
        ]);
        session.progress.selected_pets = vec![PetKind::Wadsworth];
        session.board = Some(
            AdventureState::new_tank3_first_stage(42, &session.progress.selected_pets).unwrap(),
        );
        session.phase = AdventurePhase::Playing;
        session
    }

    fn tank_three_second_session() -> AdventureSession {
        let mut session = tank_three_session();
        session.progress.level = 2;
        session.progress.unlocked_pets.push(PetKind::Seymour);
        session.progress.selected_pets = vec![PetKind::Seymour];
        session.board = Some(
            AdventureState::new_tank3_second_stage(42, &session.progress.selected_pets).unwrap(),
        );
        session
    }

    fn tank_three_third_session() -> AdventureSession {
        let mut session = tank_three_second_session();
        session.progress.level = 3;
        session.progress.unlocked_pets.push(PetKind::Shrapnel);
        session.progress.selected_pets = vec![PetKind::Shrapnel];
        session.board = Some(
            AdventureState::new_tank3_third_stage(42, &session.progress.selected_pets).unwrap(),
        );
        session
    }

    fn tank_three_fourth_session() -> AdventureSession {
        let mut session = tank_three_third_session();
        session.progress.level = 4;
        session.progress.unlocked_pets.push(PetKind::Gumbo);
        session.progress.selected_pets = vec![PetKind::Gumbo];
        session.board = Some(
            AdventureState::new_tank3_fourth_stage(42, &session.progress.selected_pets).unwrap(),
        );
        session
    }

    fn add_fixture_pearl(session: &mut AdventureSession, collecting: bool) -> u64 {
        let mut encoded = serde_json::to_value(&*session).unwrap();
        let coin_id = encoded["board"]["next_id"].as_u64().unwrap();
        encoded["board"]["next_id"] = (coin_id + 1).into();
        *session = serde_json::from_value(encoded).unwrap();
        session
            .board
            .as_mut()
            .unwrap()
            .coins
            .push(crate::sim::Coin {
                id: coin_id,
                x: 200.0,
                y: 200.0,
                kind: crate::sim::CoinKind::Pearl,
                animation_ticks: 0,
                hazard_age_ticks: 0,
                frame: 0,
                collecting,
                bottom_ticks: 0,
                fade_ticks: 0,
                penta_rising: false,
            });
        coin_id
    }

    fn add_fixture_larva(session: &mut AdventureSession, picked: bool) -> u64 {
        // Synthetic transaction fixture: allocate a genuine unique Board ID,
        // without exposing or changing the production allocator API.
        let mut encoded = serde_json::to_value(&*session).unwrap();
        let larva_id = encoded["board"]["next_id"].as_u64().unwrap();
        encoded["board"]["next_id"] = (larva_id + 1).into();
        *session = serde_json::from_value(encoded).unwrap();
        let mut larva = crate::larva::LarvaState::spawn(larva_id, 100, 200, &mut || 0);
        larva.mouse_visible = true;
        if picked {
            assert!(larva.try_pick_up());
        }
        session.board.as_mut().unwrap().larvae.push(larva);
        larva_id
    }

    #[test]
    fn claimed_larva_funds_final_egg_then_settles_once_before_seymour_hatch() {
        let mut session = tank_three_session();
        let board = session.board.as_mut().unwrap();
        board.balance = 850;
        board.eggs = 2;
        board.grubber_unlocked = true;
        board.upgrades.quality_unlocked = true;
        board.upgrades.quantity_unlocked = true;
        board.egg_unlocked = true;
        let claimed_id = add_fixture_larva(&mut session, true);
        add_fixture_larva(&mut session, false);
        session.validate().unwrap();
        assert_eq!(session.board.as_ref().unwrap().available_funds(), 1000);
        let events = session.apply_actions(&[Action::BuyEgg, Action::BuyEgg]);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    Event::PetUnlocked {
                        pet: PetKind::Seymour,
                        ..
                    }
                ))
                .count(),
            1
        );
        assert!(events.iter().any(|event| matches!(event, Event::StageResultRecorded { settled_coin_ids, settled_amount: 150, final_balance: 0, .. } if *settled_coin_ids == vec![claimed_id])));
        assert_eq!((session.progress.tank, session.progress.level), (3, 2));
        assert!(session.board.is_none());
        assert!(matches!(
            session.phase,
            AdventurePhase::Hatch {
                pet: PetKind::Seymour,
                ..
            }
        ));
        session.validate().unwrap();
        session.phase = AdventurePhase::Hatch {
            pet: PetKind::Seymour,
            updates: 171,
        };
        session.apply_actions(&[Action::Continue]);
        session.apply_actions(&[Action::TogglePet {
            pet: PetKind::Seymour,
        }]);
        session.apply_actions(&[Action::Continue]);
        assert!(matches!(
            session.phase,
            AdventurePhase::PetSelectionConfirmation { .. }
        ));
        session.apply_actions(&[Action::ConfirmPetSelection { accept: true }]);
        let board = session.board.as_ref().unwrap();
        assert_eq!((board.tank, board.level, board.balance), (3, 2, 200));
        assert!(board.larvae.is_empty());
        session.validate().unwrap();
    }

    #[test]
    fn claimed_pearl_and_larva_fund_final_egg_before_one_shrapnel_hatch() {
        let mut session = tank_three_second_session();
        let board = session.board.as_mut().unwrap();
        board.balance = 4350;
        board.eggs = 2;
        board.upgrades.quality_unlocked = true;
        board.upgrades.quantity_unlocked = true;
        board.grubber_unlocked = true;
        board.gekko_unlocked = true;
        board.weapon_unlocked = true;
        board.egg_unlocked = true;
        let pearl_id = add_fixture_pearl(&mut session, true);
        let larva_id = add_fixture_larva(&mut session, true);
        add_fixture_pearl(&mut session, false);
        add_fixture_larva(&mut session, false);
        session.validate().unwrap();
        assert_eq!(session.board.as_ref().unwrap().available_funds(), 5000);
        let events = session.apply_actions(&[Action::BuyEgg, Action::BuyEgg]);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    Event::PetUnlocked {
                        pet: PetKind::Shrapnel,
                        ..
                    }
                ))
                .count(),
            1
        );
        assert!(events.iter().any(|event| matches!(event,
            Event::StageResultRecorded { settled_coin_ids, settled_amount: 650,
                final_balance: 0, .. } if *settled_coin_ids == vec![pearl_id, larva_id])));
        assert_eq!((session.progress.tank, session.progress.level), (3, 3));
        assert_eq!(session.progress.unlocked_pets.len(), 12);
        assert!(session.board.is_none());
        assert!(matches!(
            session.phase,
            AdventurePhase::Hatch {
                pet: PetKind::Shrapnel,
                ..
            }
        ));
        session.validate().unwrap();
        session.phase = AdventurePhase::Hatch {
            pet: PetKind::Shrapnel,
            updates: 171,
        };
        session.apply_actions(&[Action::Continue]);
        session.apply_actions(&[Action::TogglePet {
            pet: PetKind::Shrapnel,
        }]);
        session.apply_actions(&[Action::Continue]);
        assert!(matches!(
            session.phase,
            AdventurePhase::PetSelectionConfirmation { .. }
        ));
        session.apply_actions(&[Action::ConfirmPetSelection { accept: true }]);
        let board = session.board.as_ref().unwrap();
        assert_eq!((board.tank, board.level, board.balance), (3, 3, 200));
        assert_eq!(board.pets, vec![PetKind::Shrapnel]);
        session.validate().unwrap();
    }

    #[test]
    fn claimed_shrapnel_bomb_funds_final_egg_and_rewards_gumbo_once() {
        // Controlled transaction boundary, not earned-play evidence. Use the
        // existing unique-ID fixture allocator and actual final-egg action.
        let mut session = tank_three_third_session();
        let board = session.board.as_mut().unwrap();
        board.balance = 7350;
        board.eggs = 2;
        board.upgrades.quality_unlocked = true;
        board.upgrades.quantity_unlocked = true;
        board.grubber_unlocked = true;
        board.gekko_unlocked = true;
        board.weapon_unlocked = true;
        board.egg_unlocked = true;
        let claimed_id = add_fixture_pearl(&mut session, true);
        let unclaimed_id = add_fixture_pearl(&mut session, false);
        for coin in &mut session.board.as_mut().unwrap().coins {
            coin.kind = crate::sim::CoinKind::ShrapnelBomb;
        }
        session.validate().unwrap();
        assert_eq!(session.board.as_ref().unwrap().available_funds(), 7500);
        let events = session.apply_actions(&[Action::BuyEgg, Action::BuyEgg]);
        assert!(events.iter().any(|event| matches!(event,
            Event::StageResultRecorded { tank: 3, level: 3, settled_coin_ids,
                settled_amount: 150, final_balance: 0, .. }
                if *settled_coin_ids == vec![claimed_id] && !settled_coin_ids.contains(&unclaimed_id))));
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    Event::PetUnlocked {
                        pet: PetKind::Gumbo,
                        ..
                    }
                ))
                .count(),
            1
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    Event::HatchStarted {
                        pet: PetKind::Gumbo,
                        ..
                    }
                ))
                .count(),
            1
        );
        assert_eq!((session.progress.tank, session.progress.level), (3, 4));
        assert_eq!(session.progress.unlocked_pets.len(), 13);
        assert!(session.board.is_none());
        session.validate().unwrap();
        session.phase = AdventurePhase::Hatch {
            pet: PetKind::Gumbo,
            updates: 171,
        };
        session.apply_actions(&[
            Action::Continue,
            Action::TogglePet {
                pet: PetKind::Gumbo,
            },
        ]);
        session.apply_actions(&[Action::Continue]);
        assert!(matches!(
            session.phase,
            AdventurePhase::PetSelectionConfirmation { .. }
        ));
        session.apply_actions(&[Action::ConfirmPetSelection { accept: true }]);
        assert!(matches!(session.phase, AdventurePhase::Playing));
        let board = session.board.as_ref().unwrap();
        assert_eq!(
            (board.tank, board.level, board.balance, board.egg_price),
            (3, 4, 200, 10000)
        );
        // The Hatch-to-selection transition deliberately clears the old
        // roster; the only subsequent toggle above selected Gumbo.
        assert_eq!(board.pets, vec![PetKind::Gumbo]);
        session.validate().unwrap();
    }

    #[test]
    fn fourth_tank_three_final_egg_settles_claimed_value_and_rewards_blip_once() {
        // A29-07 controlled affordability/retirement boundary, not live earning.
        let mut session = tank_three_fourth_session();
        let board = session.board.as_mut().unwrap();
        board.balance = 9500;
        board.eggs = 2;
        board.upgrades.quality_unlocked = true;
        board.upgrades.quantity_unlocked = true;
        board.grubber_unlocked = true;
        board.gekko_unlocked = true;
        board.weapon_unlocked = true;
        board.egg_unlocked = true;
        let claimed_id = add_fixture_pearl(&mut session, true);
        let unclaimed_id = add_fixture_pearl(&mut session, false);
        session.validate().unwrap();
        assert_eq!(session.board.as_ref().unwrap().available_funds(), 10000);
        let events = session.apply_actions(&[Action::BuyEgg, Action::BuyEgg]);
        assert!(events.iter().any(|event| matches!(event,
            Event::StageResultRecorded { tank: 3, level: 4, settled_coin_ids,
                settled_amount: 500, final_balance: 0, .. }
                if *settled_coin_ids == vec![claimed_id] && !settled_coin_ids.contains(&unclaimed_id))));
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    Event::PetUnlocked {
                        pet: PetKind::Blip,
                        ..
                    }
                ))
                .count(),
            1
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    Event::HatchStarted {
                        pet: PetKind::Blip,
                        ..
                    }
                ))
                .count(),
            1
        );
        assert_eq!((session.progress.tank, session.progress.level), (3, 5));
        assert_eq!(session.progress.unlocked_pets.len(), 14);
        assert_eq!(session.progress.unlocked_pets.last(), Some(&PetKind::Blip));
        assert!(session.board.is_none());
        session.validate().unwrap();
        session.phase = AdventurePhase::Hatch {
            pet: PetKind::Blip,
            updates: 171,
        };
        session.apply_actions(&[Action::Continue, Action::TogglePet { pet: PetKind::Blip }]);
        let before = serde_json::to_value(&session).unwrap();
        assert!(
            session
                .apply_actions(&[Action::Continue])
                .iter()
                .any(|event| matches!(
                    event,
                    Event::Rejected {
                        reason: Rejection::Locked,
                        ..
                    }
                ))
        );
        assert_eq!(serde_json::to_value(&session).unwrap(), before);
        session.validate().unwrap();
    }

    #[test]
    fn terminal_larva_settlement_retires_only_claimed_membership() {
        let mut session = tank_three_session();
        let claimed_id = add_fixture_larva(&mut session, true);
        let unclaimed_id = add_fixture_larva(&mut session, false);
        let board = session.board.as_mut().unwrap();
        assert_eq!(settle_collecting_coins(board), (vec![claimed_id], 150));
        assert_eq!(board.balance, 350);
        assert_eq!(
            board
                .larvae
                .iter()
                .map(|larva| larva.id)
                .collect::<Vec<_>>(),
            vec![unclaimed_id]
        );
        assert_eq!(settle_collecting_coins(board), (vec![], 0));
        board.validate().unwrap();
    }

    #[test]
    fn game_over_settles_pending_larva_once_after_last_guppy_starves() {
        let mut session = tank_three_session();
        let claimed_id = add_fixture_larva(&mut session, true);
        let unclaimed_id = add_fixture_larva(&mut session, false);
        for fish in &mut session.board.as_mut().unwrap().fish {
            fish.hunger = 1;
        }
        session.step(&[]);
        assert!(matches!(session.phase, AdventurePhase::Playing));
        let failure = session.step(&[]);
        assert!(matches!(session.phase, AdventurePhase::GameOver { .. }));
        assert_eq!(
            failure
                .iter()
                .filter(|event| matches!(event, Event::GameOverStarted { .. }))
                .count(),
            1
        );
        assert_eq!(session.board.as_ref().unwrap().balance, 350);
        assert!(
            !session
                .board
                .as_ref()
                .unwrap()
                .larvae
                .iter()
                .any(|larva| larva.id == claimed_id)
        );
        assert_eq!(
            session
                .board
                .as_ref()
                .unwrap()
                .larvae
                .iter()
                .map(|larva| larva.id)
                .collect::<Vec<_>>(),
            vec![unclaimed_id]
        );
        assert!(
            !session
                .step(&[])
                .iter()
                .any(|event| matches!(event, Event::GameOverStarted { .. }))
        );
        session.validate().unwrap();
    }

    #[test]
    fn tank_two_rewards_once_and_starts_clyde_and_vert_before_gating_next_stage() {
        let mut session = bonus_session();
        session.progress.tank = 2;
        session.progress.level = 1;
        session.progress.selected_pets = vec![PetKind::Niko, PetKind::Itchy, PetKind::Zorf];
        session.board = Some(
            AdventureState::new_tank2_first_stage(42, &session.progress.selected_pets).unwrap(),
        );
        session.phase = AdventurePhase::Playing;
        let board = session.board.as_mut().unwrap();
        board.eggs = 2;
        board.balance = 750;
        board.upgrades.quality_unlocked = true;
        board.upgrades.quantity_unlocked = true;
        board.potion_unlocked = true;
        board.egg_unlocked = true;
        session.validate().unwrap();
        let events = session.apply_actions(&[Action::BuyEgg, Action::BuyEgg]);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    Event::PetUnlocked {
                        pet: PetKind::Clyde,
                        ..
                    }
                ))
                .count(),
            1
        );
        assert_eq!((session.progress.tank, session.progress.level), (2, 2));
        assert!(session.board.is_none());
        session.validate().unwrap();
        let mut wrong_reward = session.clone();
        wrong_reward.phase = AdventurePhase::Hatch {
            pet: PetKind::Niko,
            updates: 0,
        };
        assert!(wrong_reward.validate().is_err());
        session.phase = AdventurePhase::Hatch {
            pet: PetKind::Clyde,
            updates: 171,
        };
        session.apply_actions(&[Action::Continue]);
        session.apply_actions(&[
            Action::TogglePet { pet: PetKind::Niko },
            Action::TogglePet {
                pet: PetKind::Itchy,
            },
            Action::TogglePet {
                pet: PetKind::Clyde,
            },
            Action::Continue,
        ]);
        let board = session.board.as_ref().unwrap();
        assert_eq!(
            (
                board.tank,
                board.level,
                board.tick,
                board.balance,
                board.egg_price
            ),
            (2, 2, 0, 200, 3000)
        );
        assert_eq!(
            board.pets,
            vec![PetKind::Niko, PetKind::Itchy, PetKind::Clyde]
        );
        assert!(board.clyde.is_some());
        assert_eq!(board.fish.len(), 2);
        assert!(board.starcatchers.is_empty());
        session.validate().unwrap();

        // Isolate the earned third-egg transaction; live earning is checked
        // separately by identified window playtests, not by this fixture.
        let board = session.board.as_mut().unwrap();
        board.eggs = 2;
        board.balance = 3000;
        board.upgrades.quality_unlocked = true;
        board.upgrades.quantity_unlocked = true;
        board.potion_unlocked = true;
        board.starcatcher_unlocked = true;
        board.weapon_unlocked = true;
        board.egg_unlocked = true;
        let events = session.apply_actions(&[Action::BuyEgg, Action::BuyEgg]);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    Event::PetUnlocked {
                        pet: PetKind::Vert,
                        ..
                    }
                ))
                .count(),
            1
        );
        assert_eq!((session.progress.tank, session.progress.level), (2, 3));
        session.validate().unwrap();
        session.phase = AdventurePhase::Hatch {
            pet: PetKind::Vert,
            updates: 171,
        };
        session.apply_actions(&[Action::Continue, Action::TogglePet { pet: PetKind::Vert }]);
        session.apply_actions(&[Action::Continue]);
        assert!(matches!(
            session.phase,
            AdventurePhase::PetSelectionConfirmation { .. }
        ));
        session.apply_actions(&[Action::ConfirmPetSelection { accept: true }]);
        let board = session.board.as_ref().unwrap();
        assert_eq!(
            (
                board.tank,
                board.level,
                board.tick,
                board.balance,
                board.egg_price
            ),
            (2, 3, 0, 200, 5000)
        );
        assert_eq!(board.pets, vec![PetKind::Vert]);
        assert_eq!(board.fish_pets.len(), 1);
        assert_eq!(board.fish_pets[0].kind, crate::fish_pet::FishPetKind::Vert);
        assert_eq!(board.fish_pets[0].coin_timer, 0);
        assert_eq!(board.fish.len(), 2);
        assert_eq!(
            board.invasion.as_ref().unwrap().plan.expected(),
            crate::invasion::EncounterKind::Single(crate::alien::SylvesterKind::Gus)
        );
        session.validate().unwrap();

        // This fixture isolates the independently recovered reward transaction;
        // it does not claim that money was earned through runtime input.
        let board = session.board.as_mut().unwrap();
        board.eggs = 2;
        board.balance = 5000;
        board.upgrades.quality_unlocked = true;
        board.upgrades.quantity_unlocked = true;
        board.potion_unlocked = true;
        board.starcatcher_unlocked = true;
        board.weapon_unlocked = true;
        board.egg_unlocked = true;
        let events = session.apply_actions(&[Action::BuyEgg, Action::BuyEgg]);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    Event::PetUnlocked {
                        pet: PetKind::Rufus,
                        ..
                    }
                ))
                .count(),
            1
        );
        assert_eq!((session.progress.tank, session.progress.level), (2, 4));
        session.validate().unwrap();
        let mut wrong_reward = session.clone();
        wrong_reward.phase = AdventurePhase::Hatch {
            pet: PetKind::Vert,
            updates: 0,
        };
        assert!(wrong_reward.validate().is_err());
        session.phase = AdventurePhase::Hatch {
            pet: PetKind::Rufus,
            updates: 171,
        };
        session.apply_actions(&[
            Action::Continue,
            Action::TogglePet {
                pet: PetKind::Rufus,
            },
        ]);
        session.apply_actions(&[Action::Continue]);
        assert!(matches!(
            session.phase,
            AdventurePhase::PetSelectionConfirmation { .. }
        ));
        session.apply_actions(&[Action::ConfirmPetSelection { accept: true }]);
        let board = session.board.as_mut().unwrap();
        assert_eq!(
            (
                board.tank,
                board.level,
                board.tick,
                board.balance,
                board.egg_price
            ),
            (2, 4, 0, 200, 7500)
        );
        assert_eq!(board.pets, vec![PetKind::Rufus]);
        assert!(board.rufus.is_some());
        assert!(board.missiles.is_empty());
        assert_eq!(
            board.invasion.as_ref().unwrap().plan.expected(),
            crate::invasion::EncounterKind::Single(crate::alien::SylvesterKind::Destructor)
        );
        assert!(board.fish.iter().all(|fish| fish.food_ate == 2));
        board.eggs = 2;
        board.balance = 7500;
        board.upgrades.quality_unlocked = true;
        board.upgrades.quantity_unlocked = true;
        board.potion_unlocked = true;
        board.starcatcher_unlocked = true;
        board.weapon_unlocked = true;
        board.egg_unlocked = true;
        let events = session.apply_actions(&[Action::BuyEgg, Action::BuyEgg]);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    Event::PetUnlocked {
                        pet: PetKind::Meryl,
                        ..
                    }
                ))
                .count(),
            1
        );
        assert_eq!((session.progress.tank, session.progress.level), (2, 5));
        session.validate().unwrap();
        session.phase = AdventurePhase::Hatch {
            pet: PetKind::Meryl,
            updates: 171,
        };
        session.apply_actions(&[
            Action::Continue,
            Action::TogglePet {
                pet: PetKind::Meryl,
            },
        ]);
        session.apply_actions(&[Action::Continue]);
        assert!(matches!(
            session.phase,
            AdventurePhase::PetSelectionConfirmation { .. }
        ));
        session.apply_actions(&[Action::ConfirmPetSelection { accept: true }]);
        let board = session.board.as_ref().unwrap();
        assert_eq!(
            (
                board.tank,
                board.level,
                board.tick,
                board.balance,
                board.egg_price
            ),
            (2, 5, 0, 200, 10000)
        );
        assert_eq!(board.pets, vec![PetKind::Meryl]);
        assert_eq!(board.fish_pets[0].kind, crate::fish_pet::FishPetKind::Meryl);
        assert!(matches!(session.phase, AdventurePhase::Playing));
        session.validate().unwrap();
    }

    #[test]
    fn fifth_victory_rewards_zorf_then_enters_empty_bonus_without_selection() {
        let mut session = selection_session();
        session.apply_actions(&[
            Action::Continue,
            Action::ConfirmPetSelection { accept: true },
        ]);
        let board = session.board.as_mut().unwrap();
        board.eggs = 2;
        board.egg_unlocked = true;
        board.balance = 5000;
        board.tick = 1000;
        session.ticks = 1000;
        session.apply_actions(&[Action::BuyEgg]);
        assert_eq!((session.progress.tank, session.progress.level), (1, 6));
        assert_eq!(
            session.phase,
            AdventurePhase::Hatch {
                pet: PetKind::Zorf,
                updates: 0
            }
        );
        assert_eq!(
            session
                .progress
                .later_stage_best_seconds
                .last()
                .unwrap()
                .seconds,
            28
        );
        session.validate().unwrap();
        session.phase = AdventurePhase::Hatch {
            pet: PetKind::Zorf,
            updates: 171,
        };
        let events = session.apply_actions(&[Action::Continue]);
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, Event::PetSelectionOpened { .. }))
        );
        assert!(session.board.is_none());
        let AdventurePhase::Bonus { state } = &session.phase else {
            panic!("bonus expected")
        };
        assert!(state.shells.is_empty());
        assert_eq!(state.tick, 0);
        assert!(state.started_at.is_none());
        session.validate().unwrap();
    }

    #[test]
    fn tank_two_finale_rewards_wadsworth_then_bonus_advances_once_to_tank_three() {
        let mut session = AdventureSession::new(42);
        session.progress.tank = 2;
        session.progress.level = 5;
        session.progress.unlocked_pets = vec![
            PetKind::Stinky,
            PetKind::Niko,
            PetKind::Itchy,
            PetKind::Prego,
            PetKind::Zorf,
            PetKind::Clyde,
            PetKind::Vert,
            PetKind::Rufus,
            PetKind::Meryl,
        ];
        session.progress.selected_pets = vec![PetKind::Niko, PetKind::Itchy, PetKind::Meryl];
        session.board = Some(
            AdventureState::new_tank2_fifth_stage(42, &session.progress.selected_pets).unwrap(),
        );
        let board = session.board.as_mut().unwrap();
        board.eggs = 2;
        board.balance = 10000;
        board.upgrades.quality_unlocked = true;
        board.upgrades.quantity_unlocked = true;
        board.potion_unlocked = true;
        board.starcatcher_unlocked = true;
        board.weapon_unlocked = true;
        board.egg_unlocked = true;
        let paid = session.apply_actions(&[Action::BuyEgg, Action::BuyEgg]);
        assert_eq!(
            paid.iter()
                .filter(|event| matches!(
                    event,
                    Event::PetUnlocked {
                        pet: PetKind::Wadsworth,
                        ..
                    }
                ))
                .count(),
            1
        );
        assert_eq!((session.progress.tank, session.progress.level), (2, 6));
        assert!(session.board.is_none());
        session.validate().unwrap();
        session.phase = AdventurePhase::Hatch {
            pet: PetKind::Wadsworth,
            updates: 171,
        };
        session.apply_actions(&[Action::Continue]);
        let AdventurePhase::Bonus { state } = &mut session.phase else {
            panic!("bonus expected")
        };
        assert_eq!((state.origin_tank, state.duration_seconds()), (2, 20));
        assert!(state.shells.is_empty());
        // Controlled completion boundary, not an earned runtime claim. Keep
        // session time and the bonus's old-counter completion state coherent.
        state.started_at = Some(0);
        state.tick = 751;
        state.timed_out = true;
        state.empty_updates = 101;
        state.shells_earned = 217;
        session.ticks = 751;
        session.progress.shell_balance = 539;
        session.validate().unwrap();
        let completed = session.step(&[]);
        assert_eq!(
            completed
                .iter()
                .filter(|event| matches!(event, Event::BonusResultsCommitted { .. }))
                .count(),
            1
        );
        assert_eq!(
            (
                session.progress.tank,
                session.progress.level,
                session.progress.shell_balance
            ),
            (3, 1, 756)
        );
        let AdventurePhase::BonusResults { result } = &session.phase else {
            panic!("results expected")
        };
        assert_eq!(
            (result.origin_tank, result.earned, result.previous_balance),
            (2, 217, 539)
        );
        for _ in 0..40 {
            assert!(
                !session
                    .step(&[])
                    .iter()
                    .any(|event| matches!(event, Event::BonusResultsCommitted { .. }))
            );
        }
        assert_eq!(session.progress.shell_balance, 756);
        session.apply_actions(&[Action::Continue]);
        assert!(matches!(session.phase, AdventurePhase::PetSelection { .. }));
        session.apply_actions(&[Action::TogglePet {
            pet: PetKind::Wadsworth,
        }]);
        session.apply_actions(&[Action::Continue]);
        assert!(matches!(
            session.phase,
            AdventurePhase::PetSelectionConfirmation { .. }
        ));
        session.apply_actions(&[Action::ConfirmPetSelection { accept: true }]);
        assert_eq!(session.progress.shell_balance, 756);
        assert!(matches!(session.phase, AdventurePhase::Playing));
        let board = session.board.as_ref().unwrap();
        assert_eq!((board.tank, board.level), (3, 1));
        assert_eq!(board.pets, vec![PetKind::Wadsworth]);
        session.validate().unwrap();
        // Malformed source identity must reject rather than overflow in a guard.
        session.phase = AdventurePhase::BonusResults {
            result: BonusResult {
                origin_tank: 255,
                earned: 217,
                previous_balance: 539,
                updates: 40,
            },
        };
        assert!(session.validate().is_err());
    }

    #[test]
    fn bonus_results_award_once_before_presentation_and_next_tank_selection() {
        let mut session = bonus_session();
        session.progress.shell_balance = MAX_SHELL_BALANCE - 10;
        let AdventurePhase::Bonus { state } = &mut session.phase else {
            unreachable!()
        };
        // 101 empty timeout checks at old counts572..672 have completed.
        state.tick = 673;
        state.started_at = Some(0);
        state.timed_out = true;
        state.empty_updates = 101;
        state.shells_earned = 17;
        session.validate().unwrap();
        let events = session.step(&[]);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, Event::BonusResultsCommitted { .. }))
                .count(),
            1
        );
        assert_eq!(session.progress.shell_balance, MAX_SHELL_BALANCE);
        assert_eq!((session.progress.tank, session.progress.level), (2, 1));
        assert!(session.board.is_none());
        session.validate().unwrap();
        assert!(
            session
                .apply_actions(&[Action::Continue])
                .iter()
                .any(|event| matches!(event, Event::Rejected { .. }))
        );
        let saved = serde_json::to_vec(&session).unwrap();
        let mut restored: AdventureSession = serde_json::from_slice(&saved).unwrap();
        for _ in 0..30 {
            restored.step(&[]);
        }
        assert_eq!(restored.progress.shell_balance, MAX_SHELL_BALANCE);
        restored.apply_actions(&[Action::Continue]);
        assert_eq!(
            restored.phase,
            AdventurePhase::PetSelection {
                selected: Vec::new()
            }
        );
        restored.apply_actions(&[
            Action::TogglePet { pet: PetKind::Zorf },
            Action::Continue,
            Action::ConfirmPetSelection { accept: true },
        ]);
        let board = restored.board.as_ref().unwrap();
        assert_eq!(
            (board.tank, board.level, board.balance, board.egg_price),
            (2, 1, 200, 750)
        );
        assert_eq!(board.pets, vec![PetKind::Zorf]);
        restored.validate().unwrap();
        restored.board.as_mut().unwrap().fish.clear();
        restored.step(&[]);
        assert_eq!(restored.phase, AdventurePhase::GameOver { updates: 0 });
        restored.validate().unwrap();
    }

    #[test]
    fn fourth_stage_rewards_prego_then_requires_selection_before_fifth_board() {
        let mut session = AdventureSession::new(42);
        session.progress.level = 4;
        session.progress.unlocked_pets = vec![PetKind::Stinky, PetKind::Niko, PetKind::Itchy];
        session.board = Some(AdventureState::new_fourth_stage(42));
        let board = session.board.as_mut().unwrap();
        board.eggs = 2;
        board.egg_unlocked = true;
        board.balance = 3000;
        board.tick = 1000;
        session.ticks = 1000;
        let events = session.apply_actions(&[Action::BuyEgg]);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    Event::PetUnlocked {
                        pet: PetKind::Prego,
                        ..
                    }
                ))
                .count(),
            1
        );
        assert_eq!(session.progress.level, 5);
        assert_eq!(
            session.phase,
            AdventurePhase::Hatch {
                pet: PetKind::Prego,
                updates: 0
            }
        );
        assert!(session.board.is_none());
        session.validate().unwrap();
        session.phase = AdventurePhase::Hatch {
            pet: PetKind::Prego,
            updates: 171,
        };
        let events = session.apply_actions(&[Action::Continue]);
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Event::PetSelectionOpened { capacity: 3, .. }))
        );
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, Event::StageStarted { .. }))
        );
        assert_eq!(
            session.phase,
            AdventurePhase::PetSelection {
                selected: Vec::new()
            }
        );
        assert!(session.progress.selected_pets.is_empty());
        assert_eq!(
            session.progress.later_stage_best_seconds[0],
            StageBestTime {
                tank: 1,
                level: 4,
                seconds: 28
            }
        );
        session.validate().unwrap();
    }

    #[test]
    fn pet_selection_caps_three_and_starts_only_the_canonical_selected_roster() {
        let mut session = selection_session();
        session.validate().unwrap();
        session.apply_actions(&[
            Action::TogglePet {
                pet: PetKind::Prego,
            },
            Action::TogglePet {
                pet: PetKind::Itchy,
            },
            Action::TogglePet {
                pet: PetKind::Stinky,
            },
        ]);
        let selected = vec![PetKind::Stinky, PetKind::Itchy, PetKind::Prego];
        assert_eq!(
            session.phase,
            AdventurePhase::PetSelection {
                selected: selected.clone()
            }
        );
        let rejected = session.apply_actions(&[Action::TogglePet { pet: PetKind::Niko }]);
        assert!(rejected.iter().any(|event| matches!(
            event,
            Event::Rejected {
                reason: Rejection::Locked,
                ..
            }
        )));
        session.validate().unwrap();
        let events = session.step(&[Action::Continue]);
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Event::PetSelectionAccepted { .. }))
        );
        assert_eq!(session.phase, AdventurePhase::Playing);
        assert_eq!(session.progress.selected_pets, selected);
        let board = session.board.as_ref().unwrap();
        assert_eq!(board.pets, selected);
        assert_eq!(
            (board.level, board.tick, board.balance, board.egg_price),
            (5, 0, 200, 5000)
        );
        assert!(board.stinky.is_some());
        assert!(board.niko.is_none());
        assert_eq!(board.fish_pets.len(), 2);
        session.validate().unwrap();
    }

    #[test]
    fn fewer_pet_confirmation_preserves_choices_and_explicitly_allows_zero() {
        let mut session = selection_session();
        session.apply_actions(&[Action::TogglePet { pet: PetKind::Niko }, Action::Continue]);
        assert_eq!(session.progress.selected_pets, vec![PetKind::Niko]);
        assert_eq!(
            session.phase,
            AdventurePhase::PetSelectionConfirmation {
                selected: vec![PetKind::Niko]
            }
        );
        session.validate().unwrap();
        let rejected = session.apply_actions(&[Action::TogglePet {
            pet: PetKind::Itchy,
        }]);
        assert!(
            rejected
                .iter()
                .any(|event| matches!(event, Event::Rejected { .. }))
        );
        session.apply_actions(&[Action::ConfirmPetSelection { accept: false }]);
        assert_eq!(
            session.phase,
            AdventurePhase::PetSelection {
                selected: vec![PetKind::Niko]
            }
        );
        session.apply_actions(&[Action::TogglePet { pet: PetKind::Niko }, Action::Continue]);
        assert_eq!(
            session.phase,
            AdventurePhase::PetSelectionConfirmation {
                selected: Vec::new()
            }
        );
        assert!(session.progress.selected_pets.is_empty());
        session.validate().unwrap();
        session.apply_actions(&[Action::ConfirmPetSelection { accept: true }]);
        let board = session.board.as_ref().unwrap();
        assert!(board.pets.is_empty() && board.fish_pets.is_empty());
        assert!(board.stinky.is_none() && board.niko.is_none());
        session.validate().unwrap();
    }

    #[test]
    fn selection_rejects_duplicate_order_and_active_roster_disagreement() {
        let mut session = selection_session();
        session.phase = AdventurePhase::PetSelection {
            selected: vec![PetKind::Niko, PetKind::Stinky],
        };
        assert!(session.validate().is_err());
        session.phase = AdventurePhase::PetSelection {
            selected: vec![PetKind::Niko, PetKind::Niko],
        };
        assert!(session.validate().is_err());
        session.phase = AdventurePhase::PetSelectionConfirmation {
            selected: vec![PetKind::Niko],
        };
        assert!(session.validate().is_err());
        session.phase = AdventurePhase::PetSelection {
            selected: vec![PetKind::Niko],
        };
        session.apply_actions(&[
            Action::Continue,
            Action::ConfirmPetSelection { accept: true },
        ]);
        session.board.as_mut().unwrap().pets.clear();
        assert!(session.validate().is_err());
    }

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
            animation_ticks: 0,
            hazard_age_ticks: 0,
            x: 200.0,
            y: 200.0,
            frame: 0,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: false,
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
            animation_ticks: 0,
            hazard_age_ticks: 0,
            frame: 0,
            collecting: true,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: false,
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

    fn second_stage_session() -> AdventureSession {
        let mut session = buy_three_eggs();
        for _ in 0..171 {
            session.step(&[]);
        }
        session.apply_actions(&[Action::Continue]);
        session
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
    fn third_stage_completion_settles_pearl_and_diamond_once_before_itchy() {
        // Primary PB12 supports aggregate/raw-cash purchasing; W1 NK5/A13-12
        // supply pearl value and third-egg reward. This fixture checks their
        // transaction boundary, not an earned-play or retail-fidelity claim.
        let mut session = third_stage_session();
        let first_best = session.progress.first_stage_best_seconds;
        let previous_results = session.progress.later_stage_best_seconds.clone();
        session.ticks = 2000;
        let board = session.board.as_mut().unwrap();
        board.tick = 1000;
        board.oscar_unlocked = true;
        board.balance = 7550;
        board.coins.push(Coin {
            id: 90,
            kind: CoinKind::Diamond,
            collecting: true,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            x: 200.0,
            y: 200.0,
            frame: 0,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: false,
        });
        let niko = board.niko.as_mut().unwrap();
        niko.cycle = 1234;
        let mut pearl = crate::niko::NikoPearl::spawn(91, niko.owner_id, 96, 251);
        assert!(pearl.pick_up(niko.owner_id));
        assert!(niko.mark_pearl_taken(niko.owner_id));
        board.pearls.push(pearl);
        let events = session.apply_actions(&[
            Action::BuyOscar,
            Action::BuyWeapon,
            Action::BuyEgg,
            Action::BuyEgg,
            Action::BuyEgg,
        ]);
        let results = events
            .iter()
            .filter(|event| matches!(event, Event::StageResultRecorded { .. }))
            .collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0], Event::StageResultRecorded {
            level: 3, seconds: 28, settled_coin_ids, settled_amount: 450,
            final_balance: 0, personal_best_seconds: 28, ..
        } if settled_coin_ids == &[90, 91]));
        assert!(!events.iter().any(|event| matches!(
            event,
            Event::CoinCredited { .. } | Event::PearlCredited { .. }
        )));
        assert_eq!(session.progress.first_stage_best_seconds, first_best);
        assert_eq!(
            &session.progress.later_stage_best_seconds[..previous_results.len()],
            &previous_results
        );
        assert_eq!(session.progress.level, 4);
        assert_eq!(
            session.phase,
            AdventurePhase::Hatch {
                pet: PetKind::Itchy,
                updates: 0
            }
        );
        session.validate().unwrap();
        for _ in 0..171 {
            session.step(&[]);
        }
        session.step(&[Action::Continue]);
        let board = session.board.as_ref().unwrap();
        assert_eq!(
            (board.level, board.tick, board.balance, board.egg_price),
            (4, 0, 200, 3000)
        );
        assert_eq!(
            board.pets,
            vec![PetKind::Stinky, PetKind::Niko, PetKind::Itchy]
        );
        assert!(board.oscars.is_empty());
        assert_eq!(board.weapon_strength, 2);
        session.validate().unwrap();
    }

    #[test]
    fn oscar_only_survival_defers_game_over_until_update_after_its_death() {
        let mut session = third_stage_session();
        let board = session.board.as_mut().unwrap();
        board.fish.clear();
        board.balance = 1000;
        board.upgrades.quality_unlocked = true;
        board.upgrades.quantity_unlocked = true;
        board.oscar_unlocked = true;
        board.apply(Action::BuyOscar);
        let oscar = board.oscars.first_mut().unwrap();
        let oscar_id = oscar.id;
        oscar.hunger = 1;
        let death = session.step(&[]);
        assert!(death.iter().any(
            |event| matches!(event, Event::OscarDied { oscar_id: id, .. } if *id == oscar_id)
        ));
        assert_eq!(session.phase, AdventurePhase::Playing);
        assert_eq!(session.board.as_ref().unwrap().tick, 1);
        let next = session.step(&[]);
        assert!(
            next.iter()
                .any(|event| matches!(event, Event::GameOverStarted { .. }))
        );
        assert_eq!(session.phase, AdventurePhase::GameOver { updates: 0 });
        assert_eq!(session.board.as_ref().unwrap().tick, 2);
        session.validate().unwrap();
    }

    #[test]
    fn invasion_modal_freezes_objects_until_acknowledged_without_time() {
        // W1 Board::Update opens the warning tutorial at 276 before widgets
        // update; its pause return still runs the food-delay decrement.
        let mut session = second_stage_session();
        let board = session.board.as_mut().unwrap();
        let wave = board.invasion.as_mut().unwrap();
        wave.countdown = 277;
        wave.food_delay = 3;
        let fish_before = serde_json::to_value(&board.fish).unwrap();
        let opened = session.step(&[]);
        assert!(opened.iter().any(|event| matches!(
            event,
            Event::Invasion {
                event: crate::invasion::InvasionEvent::ModalOpened(InvasionTip::Danger),
                ..
            }
        )));
        assert_eq!(
            session.phase,
            AdventurePhase::InvasionTutorial {
                tip: InvasionTip::Danger
            }
        );
        let board = session.board.as_ref().unwrap();
        assert_eq!(board.tick, 1);
        assert_eq!(board.invasion.as_ref().unwrap().food_delay, 2);
        assert_eq!(serde_json::to_value(&board.fish).unwrap(), fish_before);
        let random_before = board.transition_seed();
        for _ in 0..20 {
            session.step(&[]);
        }
        let board = session.board.as_ref().unwrap();
        assert_eq!(board.tick, 1);
        assert_eq!(board.invasion.as_ref().unwrap().countdown, 276);
        assert_eq!(board.transition_seed(), random_before);
        assert_eq!(serde_json::to_value(&board.fish).unwrap(), fish_before);
        session.validate().unwrap();
        session.apply_actions(&[Action::Continue]);
        assert_eq!(session.phase, AdventurePhase::Playing);
        assert_eq!(session.board.as_ref().unwrap().tick, 1);
        let warning = session.step(&[]);
        assert!(warning.iter().any(|event| matches!(
            event,
            Event::Invasion {
                event: crate::invasion::InvasionEvent::WarningStarted(_),
                ..
            }
        )));
        assert_eq!(session.board.as_ref().unwrap().tick, 2);
        session.validate().unwrap();
    }

    #[test]
    fn later_tank_game_over_waits_thirty_updates_then_reenters_fresh_stage() {
        // W1 MoneyDialog accepts the footer only when its old count >30;
        // dismissal removes the failed board but retains Adventure progress.
        let mut session = second_stage_session();
        let original_progress = session.progress.clone();
        let board = session.board.as_mut().unwrap();
        board.fish.clear();
        board.balance = 987;
        session.step(&[]);
        assert_eq!(session.phase, AdventurePhase::GameOver { updates: 0 });
        assert_eq!(session.board.as_ref().unwrap().tick, 1);
        for _ in 0..30 {
            session.step(&[]);
        }
        assert_eq!(session.phase, AdventurePhase::GameOver { updates: 30 });
        session.apply_actions(&[Action::Continue]);
        assert_eq!(session.phase, AdventurePhase::GameOver { updates: 30 });
        assert_eq!(session.board.as_ref().unwrap().balance, 987);
        session.step(&[]);
        session.apply_actions(&[Action::Continue]);
        assert_eq!(session.phase, AdventurePhase::GameSelector);
        assert!(session.board.is_none());
        assert_eq!(session.progress, original_progress);
        session.validate().unwrap();
        session.apply_actions(&[Action::PlayAdventure]);
        assert_eq!(session.phase, AdventurePhase::HelpScreen);
        session.step(&[Action::Continue]);
        let board = session.board.as_ref().unwrap();
        assert_eq!(
            (board.level, board.tick, board.balance, board.eggs),
            (2, 0, 200, 0)
        );
        assert_eq!((board.upgrades.quality, board.upgrades.quantity), (0, 1));
        assert_eq!(board.invasion.as_ref().unwrap().countdown, 1750);
        assert_eq!(board.pets, vec![PetKind::Stinky]);
        assert_eq!(session.progress, original_progress);
        session.validate().unwrap();
    }

    #[test]
    fn manual_pause_decreases_only_pre_pause_delay_and_keeps_board_clock() {
        let mut session = second_stage_session();
        let wave = session.board.as_mut().unwrap().invasion.as_mut().unwrap();
        wave.food_delay = 2;
        wave.post_spawn_flash_ticks = 3;
        let before = serde_json::to_value(session.board.as_ref().unwrap()).unwrap();
        let ticks_before = session.ticks;
        session.paused_step();
        session.paused_step();
        let mut expected = before;
        expected["invasion"]["food_delay"] = serde_json::json!(0);
        assert_eq!(
            serde_json::to_value(session.board.as_ref().unwrap()).unwrap(),
            expected
        );
        assert_eq!(session.ticks, ticks_before + 2);
    }

    #[test]
    fn second_stage_reward_records_separate_time_and_starts_niko_once() {
        // W1 Board third-egg transaction uses 500 per piece and advances to
        // 1-3 with Niko; its result must not overwrite the first-stage best.
        let mut session = second_stage_session();
        let first_best = session.progress.first_stage_best_seconds;
        session.ticks = 2000;
        let board = session.board.as_mut().unwrap();
        board.tick = 1000;
        board.upgrades.quality_unlocked = true;
        board.balance = 1700;
        let events = session.apply_actions(&[
            Action::BuyFoodQuality,
            Action::BuyEgg,
            Action::BuyEgg,
            Action::BuyEgg,
        ]);
        assert_eq!(session.progress.first_stage_best_seconds, first_best);
        assert_eq!(
            session.progress.later_stage_best_seconds,
            vec![StageBestTime {
                tank: 1,
                level: 2,
                seconds: 28,
            }]
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    Event::PetUnlocked {
                        pet: PetKind::Niko,
                        ..
                    }
                ))
                .count(),
            1
        );
        assert_eq!(
            session.phase,
            AdventurePhase::Hatch {
                pet: PetKind::Niko,
                updates: 0
            }
        );
        session.validate().unwrap();
        for _ in 0..171 {
            session.step(&[]);
        }
        session.step(&[Action::Continue]);
        let board = session.board.as_ref().unwrap();
        assert_eq!(
            (board.level, board.tick, board.balance, board.egg_price),
            (3, 0, 200, 2000)
        );
        assert_eq!(board.pets, vec![PetKind::Stinky, PetKind::Niko]);
        assert!(board.niko.is_some());
        assert!(!board.egg_unlocked);
        session.validate().unwrap();
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
