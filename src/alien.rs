//! Ordinary weak Sylvester actor for Adventure 1-2.
//!
//! The board owns spawn timing, its random stream, ordered prey membership,
//! coin creation and removal. This actor owns only its live motion, contact,
//! shooting and animation state. Constructor and update rules are secondary
//! source-derived from WinFish f919b3c (`Alien.cpp`). The installed PB05
//! payload supports shot bounds, damage and hit lockout through a strongly
//! mapped hit function; its constructor and update order remain unconfirmed.
//! No original-game run or retail parity measurement has been made.

use serde::{Deserialize, Serialize};

pub const WEAK_SYLVESTER_SIZE: i32 = 160;
const SPEED_DIVISOR: f64 = 2.0;
const STARTING_HEALTH: i16 = 50;
const SHOT_DAMAGE: i16 = 6; // Weapon strength 2 × 3.

/// A board-ordered snapshot of a possible alien target. The board decides
/// whether a fish is eligible (including its protection and newborn delay).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreyView {
    pub id: u64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub width: i32,
    pub height: i32,
    pub eligible: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AlienUpdate {
    /// At most one prey, selected in the board-provided order.
    pub prey_eaten: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShotResult {
    Miss,
    Hit {
        health: i16,
    },
    /// The board creates the single diamond and removes the live alien.
    Defeated {
        diamond_x: i32,
        diamond_y: i32,
    },
}

/// Integer widget coordinates are distinct from double motion coordinates:
/// animation can move the latter after the widget is positioned for a tick.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeakSylvester {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub vx: f64,
    pub vy: f64,
    pub target_vx: f64,
    pub target_vy: f64,
    pub previous_vx: f64,
    pub health: i16,
    pub spawn_ticks: u8,
    pub chase_ticks: u8,
    pub hit_ticks: u8,
    pub movement_state: u8,
    pub movement_change_ticks: u8,
    pub swim_ticks: u8,
    pub turn_ticks: i8,
    pub frame: u8,
    pub alive: bool,
}

impl WeakSylvester {
    /// `direction_draw` and `movement_draw` are two successive values from
    /// the board's existing RNG, supplied in that order. The actor owns no RNG.
    pub fn spawn(
        id: u64,
        widget_x: i32,
        widget_y: i32,
        direction_draw: u32,
        movement_draw: u32,
    ) -> Self {
        let left = direction_draw.is_multiple_of(2);
        Self {
            id,
            x: f64::from(widget_x),
            y: f64::from(widget_y),
            widget_x,
            widget_y,
            vx: if left { -3.0 } else { 3.0 },
            vy: 0.0,
            target_vx: 0.0,
            target_vy: 0.0,
            previous_vx: if left { -1.0 } else { 1.0 },
            health: STARTING_HEALTH,
            spawn_ticks: 15,
            chase_ticks: 100,
            hit_ticks: 0,
            movement_state: (movement_draw % 10) as u8,
            movement_change_ticks: 20,
            swim_ticks: 0,
            turn_ticks: 0,
            frame: 1,
            alive: true,
        }
    }

    /// Advances one unpaused actor update. The callback must yield the next
    /// board RNG value. It is called only when the idle movement timer rolls
    /// over, and a second time only if the first roll selects a new state.
    /// Caller-supplied prey order is the collision order; no target is saved.
    pub fn update(
        &mut self,
        prey: &[PreyView],
        mut next_random: impl FnMut() -> u32,
    ) -> AlienUpdate {
        if !self.alive {
            return AlienUpdate::default();
        }

        let mut emergence_dx = 0.0;
        if self.spawn_ticks != 0 {
            if self.spawn_ticks <= 10 {
                emergence_dx = if self.vx >= 0.0 { 3.0 } else { -3.0 };
            }
            self.spawn_ticks -= 1;
            // The count starts at 15: updates 1–6 return. On update 7,
            // old count 9 selects ±3, then count 8 permits motion.
            if self.spawn_ticks > 8 {
                return AlienUpdate::default();
            }
        }

        let mut result = AlienUpdate::default();
        if let Some(target) = self.nearest_eligible(prey) {
            self.chase(target);
            // Contact precedes motion and uses the old chase delay and old
            // integer widget location, regardless of the nearest target.
            if self.chase_ticks == 0 {
                result.prey_eaten = self.first_contact(prey);
            }
        } else {
            self.wander(&mut next_random);
        }

        self.x = self.x.clamp(-10.0, 490.0) + self.vx / SPEED_DIVISOR + emergence_dx;
        self.y = self.y.clamp(85.0, 290.0) + self.vy / SPEED_DIVISOR;
        self.hit_ticks = self.hit_ticks.saturating_sub(1);
        self.chase_ticks = self.chase_ticks.saturating_sub(1);

        // C++ Widget::Move(int,int) truncates the double arguments. Keep
        // this snapshot before animation's further double-position shift.
        self.widget_x = self.x as i32;
        self.widget_y = self.y as i32;
        self.animate();
        result
    }

    /// A single player laser attempt. The board owns shot effects and the
    /// exactly-once removal/diamond transaction returned on defeat.
    pub fn shot(&mut self, shot_x: i32, shot_y: i32) -> ShotResult {
        let sx = f64::from(shot_x);
        let sy = f64::from(shot_y);
        if !self.alive
            || sx <= self.x
            || sx >= self.x + f64::from(WEAK_SYLVESTER_SIZE)
            || sy <= self.y
            || sy >= self.y + f64::from(WEAK_SYLVESTER_SIZE)
            || self.hit_ticks != 0
        {
            return ShotResult::Miss;
        }

        self.health -= SHOT_DAMAGE;
        self.apply_shot_push(sx, sy);
        if self.health <= 0 {
            self.alive = false;
            ShotResult::Defeated {
                diamond_x: self.widget_x + 25,
                diamond_y: self.widget_y + 25,
            }
        } else {
            self.hit_ticks = 10;
            ShotResult::Hit {
                health: self.health,
            }
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if ![
            self.x,
            self.y,
            self.vx,
            self.vy,
            self.target_vx,
            self.target_vy,
            self.previous_vx,
        ]
        .into_iter()
        .all(f64::is_finite)
            || !(-32.0..=512.0).contains(&self.x)
            || !(64.0..=312.0).contains(&self.y)
            || self.vx.abs() > 7.0
            || self.vy.abs() > 7.0
            || self.target_vx.abs() > 2.0
            || self.target_vy.abs() > 2.0
        {
            return Err("invalid weak Sylvester motion".into());
        }
        if self.widget_x < -32
            || self.widget_x > 512
            || self.widget_y < 64
            || self.widget_y > 312
            || self.health > STARTING_HEALTH
            || self.alive != (self.health > 0)
            || self.spawn_ticks > 15
            || self.chase_ticks > 100
            || self.hit_ticks > 10
            || self.movement_state > 9
            || self.movement_change_ticks > 20
            || self.swim_ticks > 50
            || !(-9..=9).contains(&self.turn_ticks)
            || self.frame > 9
        {
            return Err("invalid weak Sylvester counters".into());
        }
        Ok(())
    }

    pub fn sprite_row(&self) -> u8 {
        u8::from(self.turn_ticks != 0)
    }

    pub fn facing_right(&self) -> bool {
        if self.turn_ticks != 0 {
            self.turn_ticks > 0
        } else {
            self.vx >= 0.0
        }
    }

    pub fn hit_flash(&self) -> bool {
        self.hit_ticks > 0
    }

    /// Source target selection uses old integer widget centers; equal
    /// distances retain the first item in the board's ordered view.
    fn nearest_eligible<'a>(&self, prey: &'a [PreyView]) -> Option<&'a PreyView> {
        let center_x = i64::from(self.widget_x) + 80;
        let center_y = i64::from(self.widget_y) + 80;
        let mut nearest = None;
        let mut nearest_distance = 100_000_000_i64;
        for candidate in prey.iter().filter(|view| view.eligible) {
            let dx = center_x - i64::from(candidate.widget_x + candidate.width / 2);
            let dy = center_y - i64::from(candidate.widget_y + candidate.height / 2);
            let distance = dx * dx + dy * dy;
            if distance < nearest_distance {
                nearest = Some(candidate);
                nearest_distance = distance;
            }
        }
        nearest
    }

    /// Contact scans all eligible prey in board order, independently of the
    /// nearest target selected for steering. It uses the previous widget
    /// position because the alien moves later in the update.
    fn first_contact(&self, prey: &[PreyView]) -> Option<u64> {
        let center_x = i64::from(self.widget_x) + 80;
        let center_y = i64::from(self.widget_y) + 80;
        prey.iter()
            .filter(|view| view.eligible)
            .find_map(|candidate| {
                let dx = center_x - i64::from(candidate.widget_x + candidate.width / 2);
                let dy = center_y - i64::from(candidate.widget_y + candidate.height / 2);
                (dx.abs() < 45 && dy.abs() < 65).then_some(candidate.id)
            })
    }

    fn chase(&mut self, target: &PreyView) {
        let center_x = self.x + 80.0;
        let center_y = self.y + 80.0;
        let target_x = f64::from(target.widget_x + 40);
        let target_y = f64::from(target.widget_y + 40);
        if center_x < target_x && self.vx < 1.8 {
            self.vx += 0.1;
        } else if center_x > target_x && self.vx > -1.8 {
            self.vx -= 0.1;
        }
        if center_y < target_y && self.vy < 1.8 {
            self.vy += 0.1;
        } else if center_y > target_y && self.vy > -1.8 {
            self.vy -= 0.1;
        }
    }

    fn wander(&mut self, next_random: &mut impl FnMut() -> u32) {
        match self.movement_state {
            0 => self.target_vx = -1.5,
            1 => self.target_vx = 1.5,
            2 => self.target_vy = -1.5,
            3 => self.target_vy = 1.5,
            _ => {}
        }
        // W1 uses independent comparisons, so a small overshoot can be
        // countered within the same update.
        if self.vy > self.target_vy {
            self.vy -= 0.1;
        }
        if self.vy < self.target_vy {
            self.vy += 0.1;
        }
        if self.vx > self.target_vx {
            self.vx -= 0.1;
        }
        if self.vx < self.target_vx {
            self.vx += 0.1;
        }
        self.movement_change_ticks += 1;
        if self.movement_change_ticks > 20 {
            self.movement_change_ticks = 0;
            if next_random().is_multiple_of(10) {
                self.movement_state = (next_random() % 4) as u8;
            }
        }
    }

    fn animate(&mut self) {
        if self.previous_vx < 0.0 && self.vx > 0.0 {
            self.turn_ticks = -10;
        } else if self.previous_vx > 0.0 && self.vx < 0.0 {
            self.turn_ticks = 10;
        }
        self.turn_ticks -= self.turn_ticks.signum();

        if self.turn_ticks > 0 {
            self.frame = (9 - self.turn_ticks) as u8;
        } else if self.turn_ticks < 0 {
            self.frame = (self.turn_ticks + 10) as u8;
        } else if self.vx.abs() <= 1.6 {
            self.swim_ticks += 1;
            if self.swim_ticks > 19 {
                self.swim_ticks = 0;
            }
            self.frame = self.swim_ticks / 2;
        } else {
            self.swim_ticks += 1;
            match self.swim_ticks {
                1..=6 => {
                    self.x -= self.vx / SPEED_DIVISOR * 0.25;
                    self.y -= self.vy / SPEED_DIVISOR * 0.25;
                    self.frame = self.swim_ticks / 2;
                }
                7..=10 => {
                    self.x += self.vx / SPEED_DIVISOR * 0.75;
                    self.y += self.vy / SPEED_DIVISOR * 0.75;
                    self.frame = self.swim_ticks / 2;
                }
                11..=39 => {
                    self.x += self.vx / SPEED_DIVISOR * 0.5;
                    self.y += self.vy / SPEED_DIVISOR * 0.5;
                    self.frame = 6;
                }
                40..=50 => {
                    self.x += self.vx / SPEED_DIVISOR * 0.5;
                    self.y += self.vy / SPEED_DIVISOR * 0.5;
                    self.frame = (50 - self.swim_ticks) / 2;
                }
                _ => {
                    self.swim_ticks = 0;
                    self.frame = 0;
                }
            }
        }

        if self.vx != self.previous_vx && self.vx != 0.0 && self.previous_vx != 0.0 {
            self.previous_vx = self.vx;
        }
    }

    fn apply_shot_push(&mut self, sx: f64, sy: f64) {
        let left = sx < self.x + 60.0;
        let upper = sy < self.y + 60.0;
        let right = sx > self.x + 100.0;
        let lower = sy > self.y + 100.0;
        if left && upper {
            (self.vx, self.vy) = (5.0, 5.0);
        } else if left && sy < self.y + 100.0 {
            self.vx = 6.0;
        } else if left {
            (self.vx, self.vy) = (5.0, -5.0);
        } else if sx < self.x + 100.0 && upper {
            self.vy = 6.0;
        } else if right && lower {
            (self.vx, self.vy) = (-5.0, -5.0);
        } else if right && upper {
            (self.vx, self.vy) = (-5.0, 5.0);
        } else if right {
            self.vx = -6.0;
        } else if lower {
            self.vy = -6.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alien() -> WeakSylvester {
        WeakSylvester::spawn(7, 100, 120, 1, 1)
    }

    #[test]
    fn spawn_early_returns_then_widget_precedes_animation_and_emergence() {
        let mut actor = alien();
        let mut draws = 0;
        for _ in 0..6 {
            assert_eq!(
                actor.update(&[], || {
                    draws += 1;
                    1
                }),
                AlienUpdate::default()
            );
            assert_eq!((actor.widget_x, actor.widget_y), (100, 120));
            assert_eq!(actor.chase_ticks, 100);
        }
        assert_eq!(draws, 0);
        actor.update(&[], || {
            draws += 1;
            1
        });
        assert_eq!(actor.spawn_ticks, 8);
        assert_eq!(actor.widget_x, 104); // First applied ±3 offset.
        assert!(actor.x > f64::from(actor.widget_x));
        assert_eq!(draws, 1); // First active idle rollover.
        actor.update(&[], || {
            draws += 1;
            1
        });
        assert_eq!(actor.spawn_ticks, 7);
        assert_eq!(actor.widget_x, 108);
    }

    #[test]
    fn first_overlap_is_eaten_even_when_another_fish_is_nearest() {
        let mut actor = alien();
        actor.spawn_ticks = 0;
        actor.chase_ticks = 0;
        actor.x = 200.0;
        actor.y = 200.0;
        actor.widget_x = 200;
        actor.widget_y = 200;
        let prey = [
            PreyView {
                id: 1,
                widget_x: 285,
                widget_y: 240,
                width: 80,
                height: 80,
                eligible: true,
            },
            PreyView {
                id: 2,
                widget_x: 284,
                widget_y: 240,
                width: 80,
                height: 80,
                eligible: true,
            },
            PreyView {
                id: 3,
                widget_x: 240,
                widget_y: 240,
                width: 80,
                height: 80,
                eligible: true,
            },
        ];
        assert_eq!(
            actor
                .update(&prey, || panic!("chase should consume no RNG"))
                .prey_eaten,
            Some(2)
        );
    }

    #[test]
    fn shooting_uses_strict_double_bounds_and_one_death_reward() {
        let mut actor = alien();
        actor.spawn_ticks = 0;
        assert_eq!(actor.shot(100, 200), ShotResult::Miss);
        assert_eq!(actor.shot(260, 200), ShotResult::Miss);
        assert_eq!(actor.shot(101, 121), ShotResult::Hit { health: 44 });
        assert_eq!(actor.shot(101, 121), ShotResult::Miss);
        for accepted_shot in 2..=9 {
            for _ in 0..10 {
                actor.update(&[], || 1);
            }
            let shot_x = (actor.x + 80.0) as i32;
            let shot_y = (actor.y + 80.0) as i32;
            let expected_origin = (actor.widget_x + 25, actor.widget_y + 25);
            let result = actor.shot(shot_x, shot_y);
            if accepted_shot == 9 {
                assert_eq!(
                    result,
                    ShotResult::Defeated {
                        diamond_x: expected_origin.0,
                        diamond_y: expected_origin.1,
                    }
                );
            } else {
                assert_eq!(
                    result,
                    ShotResult::Hit {
                        health: 50 - 6 * accepted_shot
                    }
                );
            }
        }
        assert_eq!(
            actor.shot(actor.widget_x + 80, actor.widget_y + 80),
            ShotResult::Miss
        );
    }

    #[test]
    fn hidden_spawn_does_not_advance_nonlethal_hit_immunity() {
        let mut actor = alien();
        assert_eq!(actor.shot(180, 200), ShotResult::Hit { health: 44 });
        for _ in 0..6 {
            actor.update(&[], || panic!("hidden return should consume no RNG"));
        }
        assert_eq!(actor.hit_ticks, 10);
        actor.update(&[], || 1);
        assert_eq!(actor.hit_ticks, 9);
    }

    #[test]
    fn shot_push_bands_use_ordered_strict_edges() {
        let mut middle = alien();
        assert_eq!(middle.shot(160, 180), ShotResult::Hit { health: 44 });
        assert_eq!((middle.vx, middle.vy), (3.0, 0.0));

        let mut left_band = alien();
        left_band.shot(159, 180);
        assert_eq!((left_band.vx, left_band.vy), (6.0, 0.0));

        let mut right_upper = alien();
        right_upper.shot(201, 179);
        assert_eq!((right_upper.vx, right_upper.vy), (-5.0, 5.0));
    }

    #[test]
    fn idle_rng_requests_only_the_conditional_state_draw() {
        let mut actor = alien();
        actor.spawn_ticks = 0;
        let mut rolls = [0_u32, 3].into_iter();
        actor.update(&[], || rolls.next().unwrap());
        assert_eq!(actor.movement_state, 3);
        assert_eq!(rolls.next(), None);
        actor.movement_change_ticks = 20;
        let mut calls = 0;
        actor.update(&[], || {
            calls += 1;
            1
        });
        assert_eq!(calls, 1);
        assert_eq!(actor.movement_state, 3);
    }

    #[test]
    fn save_round_trip_keeps_double_and_widget_positions_distinct() {
        let mut actor = alien();
        actor.spawn_ticks = 0;
        actor.update(&[], || 1);
        assert_ne!(actor.x, f64::from(actor.widget_x));
        let encoded = serde_json::to_string(&actor).unwrap();
        let resumed: WeakSylvester = serde_json::from_str(&encoded).unwrap();
        assert_eq!(resumed.widget_x, actor.widget_x);
        assert_eq!(resumed.x, actor.x);
        resumed.validate().unwrap();
    }

    #[test]
    fn slowing_from_fast_swim_resets_the_shared_counter_above_nineteen() {
        let mut actor = alien();
        actor.vx = 1.6;
        actor.previous_vx = 1.6;
        actor.swim_ticks = 30;
        actor.animate();
        assert_eq!(actor.swim_ticks, 0);
        assert_eq!(actor.frame, 0);
    }
}
