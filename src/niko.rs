//! Ordinary tank-one Niko and his separately owned pearl.
//! Rules here are secondary-source-derived from pinned WinFish W1
//! `OtherTypePet.cpp` and `Coin.cpp`; installed-binary parity is unverified.

use serde::{Deserialize, Serialize};

pub const NIKO_X: i32 = 95;
pub const NIKO_Y: i32 = 253;
pub const PEARL_VALUE: i32 = 250;
pub const PEARL_SIZE: i32 = 72;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NikoEvent {
    OpenSound,
    PearlSpawn { owner_id: u64, x: i32, y: i32 },
    Bubble { x: i32, y: i32 },
    CloseSound,
}

/// Stable owner identity, cycle, and the counters needed to resume the source
/// RNG-call schedule after a project save. Niko is fixed at `(95,253)` in tank 1.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NikoState {
    pub owner_id: u64,
    pub cycle: u16,
    pub pearl_taken: bool,
    pub movement_animation_timer: u8,
    pub movement_change_timer: u8,
}

impl NikoState {
    /// Board::SpawnPet(-1,-1) draws `%265`, `%520`; the common pet constructor
    /// then draws `%10`, `%250`. The chosen spawn position and constructor fields
    /// do not move ordinary Niko in tank 1, but the four draws affect later RNG.
    pub fn spawn_tank1(owner_id: u64, rand_range: &mut impl FnMut(u64) -> u64) -> Self {
        let _spawn_x = rand_range(265);
        let _spawn_y = rand_range(520);
        let _movement_state = rand_range(10);
        let _random_timer = rand_range(250);
        Self {
            owner_id,
            cycle: 0,
            pearl_taken: false,
            movement_animation_timer: 0,
            movement_change_timer: 0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.owner_id == 0
            || self.cycle >= 1450
            || self.movement_animation_timer >= 19
            || self.movement_change_timer > 20
        {
            return Err("invalid ordinary Niko save state".into());
        }
        Ok(())
    }

    /// One unpaused object update. The caller owns board RNG and processes the
    /// returned one-shot events in its object-update order.
    pub fn tick(&mut self, rand_range: &mut impl FnMut(u64) -> u64) -> Vec<NikoEvent> {
        self.movement_change_timer += 1;
        if self.movement_change_timer > 20 {
            self.movement_change_timer = 0;
            if rand_range(10) == 0 {
                let _movement_state = rand_range(3);
            }
        }

        self.cycle += 1;
        let events = match self.cycle {
            1224 => vec![NikoEvent::OpenSound],
            1233 => {
                self.pearl_taken = false;
                vec![
                    NikoEvent::PearlSpawn {
                        owner_id: self.owner_id,
                        x: NIKO_X + 1,
                        y: NIKO_Y - 2,
                    },
                    NikoEvent::Bubble {
                        x: NIKO_X + 11,
                        y: NIKO_Y + 5,
                    },
                    NikoEvent::Bubble {
                        x: NIKO_X + 7,
                        y: NIKO_Y + 3,
                    },
                ]
            }
            1440 => vec![NikoEvent::CloseSound],
            1450 => {
                self.cycle = rand_range(50) as u16;
                Vec::new()
            }
            _ => Vec::new(),
        };
        self.movement_animation_timer = (self.movement_animation_timer + 1) % 19;
        events
    }

    /// Sheet cell, column then row. The open-shell sheet carries the waiting
    /// pearl; the separate coin image is drawn only after collection starts.
    pub fn frame(&self) -> (u8, u8) {
        if self.cycle < 1224 {
            (self.movement_animation_timer.abs_diff(9), 0)
        } else if self.cycle < 1234 {
            ((self.cycle - 1224) as u8, 1)
        } else {
            let column = if self.cycle < 1440 {
                9
            } else {
                9 - (self.cycle - 1440) as u8
            };
            (column, 1 + u8::from(self.pearl_taken))
        }
    }

    /// Called by the board only after the matching pearl begins collection.
    pub fn mark_pearl_taken(&mut self, owner_id: u64) -> bool {
        if self.owner_id != owner_id {
            return false;
        }
        self.pearl_taken = true;
        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PearlPhase {
    Waiting,
    Collecting,
    Finished,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PearlRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PearlUpdate {
    Alive,
    Expired,
    Credited { owner_id: u64, amount: i32 },
    Finished,
}

/// Unlike ordinary coins, Niko's pearl is stationary until a player click and
/// lives on the board's separate pearl list. `widget_y` is the prior integer
/// widget position used by the source's flight completion check.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NikoPearl {
    pub id: u64,
    pub owner_id: u64,
    pub x: f64,
    pub y: f64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub age: u16,
    pub phase: PearlPhase,
}

impl NikoPearl {
    pub fn spawn(id: u64, owner_id: u64, x: i32, y: i32) -> Self {
        Self {
            id,
            owner_id,
            x: f64::from(x),
            y: f64::from(y),
            widget_x: x,
            widget_y: y,
            age: 0,
            phase: PearlPhase::Waiting,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0
            || self.owner_id == 0
            || !self.x.is_finite()
            || !self.y.is_finite()
            || !(0.0..=640.0).contains(&self.x)
            || !(0.0..=480.0).contains(&self.y)
            || !(0..=640).contains(&self.widget_x)
            || !(0..=480).contains(&self.widget_y)
            || (self.phase != PearlPhase::Finished
                && (self.widget_x != self.x as i32 || self.widget_y != self.y as i32))
            || (self.phase != PearlPhase::Finished && self.age >= 217)
        {
            return Err("invalid ordinary Niko pearl save state".into());
        }
        Ok(())
    }

    pub fn widget_rect(&self) -> PearlRect {
        PearlRect {
            x: self.widget_x,
            y: self.widget_y,
            width: PEARL_SIZE,
            height: PEARL_SIZE,
        }
    }

    pub fn should_draw_separately(&self) -> bool {
        self.phase == PearlPhase::Collecting
    }

    /// A geometric helper only. The board owns input routing and click priority.
    pub fn contains_world_point(&self, x: i32, y: i32) -> bool {
        let rect = self.widget_rect();
        x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
    }

    pub fn can_claim(&self, owner_id: u64) -> bool {
        self.phase == PearlPhase::Waiting && self.owner_id == owner_id
    }

    /// Starts a single collection flight. A successful call asks the board to
    /// mark the matching Niko as taken and play SOUND_PEARL once.
    pub fn pick_up(&mut self, owner_id: u64) -> bool {
        if !self.can_claim(owner_id) {
            return false;
        }
        self.phase = PearlPhase::Collecting;
        true
    }

    pub fn tick(&mut self) -> PearlUpdate {
        match self.phase {
            PearlPhase::Waiting => {
                self.age += 1;
                if self.age >= 217 {
                    self.phase = PearlPhase::Finished;
                    PearlUpdate::Expired
                } else {
                    PearlUpdate::Alive
                }
            }
            PearlPhase::Collecting => {
                self.x += (550.0 - self.x) / 7.0;
                self.y += (30.0 - self.y) / 7.0;
                // Source checks the old integer widget position before Move.
                if self.widget_y < 40 {
                    self.phase = PearlPhase::Finished;
                    return PearlUpdate::Credited {
                        owner_id: self.owner_id,
                        amount: PEARL_VALUE,
                    };
                }
                self.widget_x = self.x as i32;
                self.widget_y = self.y as i32;
                PearlUpdate::Alive
            }
            PearlPhase::Finished => PearlUpdate::Finished,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn spawn_draws_four_common_values_and_cycle_events_are_exact() {
        let draws = RefCell::new(Vec::new());
        let mut rand = |upper| {
            draws.borrow_mut().push(upper);
            upper - 1
        };
        let mut niko = NikoState::spawn_tank1(7, &mut rand);
        assert_eq!(*draws.borrow(), [265, 520, 10, 250]);
        assert_eq!(niko.frame(), (9, 0));
        for _ in 1..1224 {
            assert!(niko.tick(&mut rand).is_empty());
        }
        assert_eq!(niko.tick(&mut rand), vec![NikoEvent::OpenSound]);
        for _ in 1225..1233 {
            assert!(niko.tick(&mut rand).is_empty());
        }
        assert_eq!(
            niko.tick(&mut rand),
            vec![
                NikoEvent::PearlSpawn {
                    owner_id: 7,
                    x: 96,
                    y: 251
                },
                NikoEvent::Bubble { x: 106, y: 258 },
                NikoEvent::Bubble { x: 102, y: 256 },
            ]
        );
        assert_eq!(niko.frame(), (9, 1));
        assert!(!niko.mark_pearl_taken(8));
        assert!(niko.mark_pearl_taken(7));
        niko.tick(&mut rand);
        assert_eq!(niko.frame(), (9, 2));
        for _ in 1235..1440 {
            niko.tick(&mut rand);
        }
        assert_eq!(niko.tick(&mut rand), vec![NikoEvent::CloseSound]);
        assert_eq!(niko.frame(), (9, 2));
        for _ in 1441..1450 {
            niko.tick(&mut rand);
        }
        assert!(niko.tick(&mut rand).is_empty());
        assert_eq!(niko.cycle, 49);
        assert_eq!(draws.borrow().last(), Some(&50));
    }

    #[test]
    fn common_random_branch_and_pause_contract() {
        let draws = RefCell::new(Vec::new());
        let mut rand = |upper| {
            draws.borrow_mut().push(upper);
            0
        };
        let mut niko = NikoState::spawn_tank1(1, &mut rand);
        let frozen = niko.clone();
        // Paused/modal callers skip tick, so no state or RNG is consumed.
        assert_eq!(niko, frozen);
        for _ in 0..20 {
            niko.tick(&mut rand);
        }
        assert_eq!(*draws.borrow(), [265, 520, 10, 250]);
        niko.tick(&mut rand);
        assert_eq!(&draws.borrow()[4..], &[10, 3]);
        assert_eq!(niko.movement_change_timer, 0);
    }

    #[test]
    fn pearl_pickup_flight_credit_once_and_expiry() {
        let mut pearl = NikoPearl::spawn(9, 7, 96, 251);
        assert_eq!(
            pearl.widget_rect(),
            PearlRect {
                x: 96,
                y: 251,
                width: 72,
                height: 72
            }
        );
        assert!(pearl.contains_world_point(96, 251));
        assert!(!pearl.contains_world_point(168, 251));
        assert!(!pearl.pick_up(8));
        assert!(!pearl.should_draw_separately());
        assert!(pearl.pick_up(7));
        assert!(pearl.should_draw_separately());
        assert!(!pearl.pick_up(7));
        let terminal = (0..100)
            .map(|_| pearl.tick())
            .find(|outcome| *outcome != PearlUpdate::Alive)
            .expect("collection flight should terminate");
        assert_eq!(
            terminal,
            PearlUpdate::Credited {
                owner_id: 7,
                amount: 250
            }
        );
        pearl.validate().unwrap();
        assert_eq!(pearl.tick(), PearlUpdate::Finished);
        assert!(!pearl.should_draw_separately());
        let mut stale = NikoPearl::spawn(10, 7, 96, 251);
        for _ in 0..216 {
            assert_eq!(stale.tick(), PearlUpdate::Alive);
        }
        assert_eq!(stale.age, 216);
        assert_eq!(stale.tick(), PearlUpdate::Expired);
        assert_eq!(stale.tick(), PearlUpdate::Finished);
        assert!(!stale.pick_up(7));
    }

    #[test]
    fn flight_credits_on_tick_after_prior_widget_y_crosses_threshold() {
        let mut pearl = NikoPearl::spawn(11, 7, 96, 41);
        assert!(pearl.pick_up(7));
        assert_eq!(pearl.tick(), PearlUpdate::Alive);
        assert_eq!(pearl.widget_y, 39);
        assert_eq!(
            pearl.tick(),
            PearlUpdate::Credited {
                owner_id: 7,
                amount: 250
            }
        );
        assert_eq!(pearl.tick(), PearlUpdate::Finished);
    }

    #[test]
    fn save_roundtrip_and_invalid_owner_or_position_rejected() {
        let mut rand = |_upper| 0;
        let mut niko = NikoState::spawn_tank1(7, &mut rand);
        niko.cycle = 1234;
        let mut pearl = NikoPearl::spawn(9, 7, 96, 251);
        assert!(pearl.pick_up(7));
        pearl.tick();
        let saved = serde_json::to_vec(&(niko.clone(), pearl.clone())).unwrap();
        let (loaded_niko, loaded_pearl): (NikoState, NikoPearl) =
            serde_json::from_slice(&saved).unwrap();
        assert_eq!(loaded_niko, niko);
        assert_eq!(loaded_pearl, pearl);
        loaded_niko.validate().unwrap();
        loaded_pearl.validate().unwrap();
        niko.cycle = 1450;
        assert!(niko.validate().is_err());
        pearl.owner_id = 0;
        assert!(pearl.validate().is_err());
    }
}
