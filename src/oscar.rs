//! Ordinary Adventure 1-3 carnivore, derived from pinned WinFish W1
//! `Oscar.cpp`, `Fish.cpp`, and `GameObject.cpp` (revision f919b3c).
//! Installed-binary Oscar behavior remains unconfirmed. The board owns
//! ordered guppy membership, IDs, diamonds, and deferred actor removal.
//! W1 bubble membership/constructors and optional meal particles are not
//! modeled, so the RNG schedule is only bounded through constructor draws.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OscarPrey {
    pub id: u64,
    pub widget_x: i32,
    pub widget_y: i32,
    /// Caller restricts this to a live small guppy with no eat delay.
    pub eligible: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OscarUpdate {
    pub eaten_prey: Option<u64>,
    pub diamond: Option<(i32, i32)>,
    pub died: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OscarPose {
    Swim,
    Eat,
    Turn,
}

#[derive(Clone, Copy, Debug)]
struct DeathPose {
    x: i32,
    y: i32,
    vx: f64,
    vy: f64,
    speed_mod: f64,
    facing_right: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OscarState {
    pub id: u64,
    pub alive: bool,
    pub x: f64,
    pub y: f64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub vx: f64,
    pub vy: f64,
    pub hunger: i32,
    pub cannot_be_eaten_ticks: u8,
    pub frame: u8,
    pub turn_ticks: i8,
    pub eating_ticks: u8,
    pub coin_timer: u16,
    pub coin_threshold: u16,
    pub bought_timer: u8,
    speed_mod: f64,
    previous_vx: f64,
    movement_state: u8,
    movement_timer: u8,
    special_timer: u8,
    x_direction: i8,
    vx_abs: u8,
    swim_counter: u8,
    speedy_speed_ticks: u8,
    hunger_shown: bool,
    hunger_animation_ticks: u8,
    #[serde(skip)]
    death_pose: Option<DeathPose>,
}

/// The inherited ordinary DeadFish body specialized to Oscar's atlas row.
/// The board stores this separately from live Oscars until `tick` expires it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeadOscar {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub frame: u8,
    pub opacity: f32,
    pub facing_right: bool,
    pub remaining_ticks: u16,
    vx: f64,
    vy: f64,
    speed_mod: f64,
}

impl DeadOscar {
    /// Only call after `OscarState::tick` reports `died`. Fish::Die runs in
    /// the hunger hook, so this uses the saved pose before that tick's motion.
    pub fn from_live(actor: &OscarState) -> Self {
        let pose = actor
            .death_pose
            .expect("Oscar corpse requires a death-tick pose");
        let vy = pose.vy
            - if pose.x < 115 || pose.vy < -3.0 {
                1.0
            } else {
                2.0
            };
        Self {
            id: actor.id,
            x: f64::from(pose.x),
            y: f64::from(pose.y),
            widget_x: pose.x,
            widget_y: pose.y,
            frame: 0,
            opacity: 1.0,
            facing_right: pose.facing_right,
            remaining_ticks: 125,
            vx: pose.vx,
            vy,
            speed_mod: pose.speed_mod,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0
            || ![self.x, self.y, self.vx, self.vy, self.speed_mod]
                .into_iter()
                .all(f64::is_finite)
            || !(-64.0..=640.0).contains(&self.x)
            || !(0.0..=380.0).contains(&self.y)
            || self.widget_x != self.x as i32
            || self.widget_y != self.y as i32
            || ![1.6, 1.8, 2.0].contains(&self.speed_mod)
            || self.frame > 9
            || self.remaining_ticks > 125
            || !self.opacity.is_finite()
            || !(0.0..=1.0).contains(&self.opacity)
        {
            return Err("invalid ordinary Oscar corpse save state".into());
        }
        Ok(())
    }

    /// True on the update after the countdown reaches zero. The ordinary
    /// non-Angel body pauses at 105 while it sinks, then fades near the floor.
    pub fn tick(&mut self) -> bool {
        let remaining = self.remaining_ticks;
        self.frame = if remaining >= 106 {
            9 - ((remaining - 106) / 2) as u8
        } else {
            match remaining {
                104 | 103 => 8,
                102 | 101 => 7,
                0..=100 => 6,
                _ => 9,
            }
        };
        if remaining < 105 {
            self.opacity = (self.opacity - 0.02).max(0.0);
        }
        if remaining == 0 {
            return true;
        }
        if remaining > 105 || self.y > 370.0 {
            self.remaining_ticks -= 1;
        }
        if self.vx < 0.0 {
            self.vx = (self.vx + 0.03).min(0.0);
        } else if self.vx > 0.0 {
            self.vx = (self.vx - 0.03).max(0.0);
        }
        if self.vy < 2.0 {
            self.vy += 0.05;
        }
        self.x = (self.x + self.vx / self.speed_mod).clamp(10.0, 540.0);
        self.y = (self.y + self.vy / self.speed_mod).clamp(85.0, 380.0);
        self.widget_x = self.x as i32;
        self.widget_y = self.y as i32;
        false
    }
}

impl OscarState {
    /// The two board position draws precede the common Fish constructor's
    /// six draws, Oscar's hunger override, then the board's entrance draws.
    /// The constructor Y draw is consumed even though Board immediately
    /// places a bought Oscar at Y=40. This is W1 order, not retail RNG parity.
    pub fn spawn_bought(id: u64, rand_range: &mut impl FnMut(u64) -> u64) -> Self {
        let x = rand_range(520) as i32 + 20;
        let _constructor_y = rand_range(265) as i32 + 105;
        let left = rand_range(2) != 0;
        let speed_mod = match rand_range(3) {
            0 => 2.0,
            1 => 1.8,
            _ => 1.6,
        };
        let _common_hunger = rand_range(200);
        let _growth_threshold = rand_range(3);
        let movement_state = rand_range(10) as u8;
        let coin_threshold = rand_range(200) as u16 + 150;
        let hunger = rand_range(200) as i32 + 600;
        let vy = rand_range(5) as f64 + 23.0;
        let bought_timer = rand_range(10) as u8 + 45;
        Self {
            id,
            alive: true,
            x: x as f64,
            y: 40.0,
            widget_x: x,
            widget_y: 40,
            vx: if left { -0.1 } else { 0.0 },
            vy,
            hunger,
            cannot_be_eaten_ticks: 0,
            frame: 0,
            turn_ticks: 0,
            eating_ticks: 0,
            coin_timer: 0,
            coin_threshold,
            bought_timer,
            speed_mod,
            previous_vx: if left { -1.0 } else { 1.0 },
            movement_state,
            movement_timer: 0,
            special_timer: 40,
            x_direction: 1,
            vx_abs: 0,
            swim_counter: 0,
            speedy_speed_ticks: 0,
            hunger_shown: false,
            hunger_animation_ticks: 0,
            death_pose: None,
        }
    }

    pub fn sprite_pose(&self) -> OscarPose {
        if self.turn_ticks != 0 {
            OscarPose::Turn
        } else if self.eating_ticks != 0 {
            OscarPose::Eat
        } else {
            OscarPose::Swim
        }
    }

    pub fn facing_right(&self) -> bool {
        if self.turn_ticks != 0 {
            return self.turn_ticks > 0;
        }
        if self.vx < 0.0 {
            false
        } else if self.vx as i32 == 0 {
            self.previous_vx >= 0.0
        } else {
            true
        }
    }

    pub fn hunger_visible(&self) -> bool {
        self.hunger <= 300 && self.hunger_animation_ticks == 0
    }

    /// Oscar.cpp overlays the hungry pose during its five-update transition.
    pub fn hunger_overlay_alpha(&self) -> f32 {
        f32::from(self.hunger_animation_ticks) / 5.0
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0
            || ![
                self.x,
                self.y,
                self.vx,
                self.vy,
                self.speed_mod,
                self.previous_vx,
            ]
            .into_iter()
            .all(f64::is_finite)
            || !(-64.0..=640.0).contains(&self.x)
            || !(0.0..=480.0).contains(&self.y)
            || self.widget_x < -64
            || self.widget_x > 640
            || self.widget_y < 0
            || self.widget_y > 480
            || self.widget_x != self.x as i32
            || self.widget_y != self.y as i32
            || ![1.6, 1.8, 2.0].contains(&self.speed_mod)
            || !(0..=1300).contains(&self.hunger)
            || !(150..=349).contains(&self.coin_threshold)
            || self.coin_timer >= self.coin_threshold
            || self.movement_state > 9
            || self.movement_timer > 20
            || self.special_timer > 50
            || !matches!(self.x_direction, -1 | 1)
            || self.vx_abs > 64
            || self.swim_counter >= 20
            || self.frame > 9
            || !(-19..=19).contains(&self.turn_ticks)
            || self.eating_ticks > 20
            || self.bought_timer > 54
            || self.hunger_animation_ticks > 5
        {
            return Err("invalid ordinary Oscar save state".into());
        }
        Ok(())
    }

    pub fn tick(
        &mut self,
        prey: &[OscarPrey],
        alien_present: bool,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> OscarUpdate {
        if !self.alive {
            return OscarUpdate::default();
        }
        self.cannot_be_eaten_ticks = self.cannot_be_eaten_ticks.saturating_sub(1);
        self.speedy_speed_ticks = self.speedy_speed_ticks.saturating_sub(1);
        self.update_hunger_animation();
        if !alien_present {
            self.hunger -= 1;
            if self.hunger == 304 {
                self.hunger_shown = true;
                self.hunger_animation_ticks = 1;
            }
        }
        let mut result = OscarUpdate::default();
        if self.hunger < 1 {
            self.death_pose = Some(DeathPose {
                x: self.x as i32,
                y: self.y as i32,
                vx: self.vx,
                vy: self.vy,
                speed_mod: self.speed_mod,
                facing_right: self.vx >= 0.0,
            });
            self.alive = false;
            result.died = true;
        }
        let hunting = if self.alive && self.hunger < 500 {
            self.hunt(prey, &mut result)
        } else {
            false
        };
        if !hunting {
            self.wander();
        }
        self.special_timer = self.special_timer.saturating_add(1);
        self.movement_timer += 1;
        if self.movement_timer > 20 {
            self.movement_timer = 0;
            if rand_range(10) == 0 {
                self.movement_state = rand_range(9) as u8 + 1;
            }
        }
        if !alien_present {
            self.coin_timer += 1;
            if self.coin_timer >= self.coin_threshold {
                self.coin_timer = 0;
                result.diamond = Some((self.widget_x + 5, self.widget_y + 10));
            }
        }
        if self.bought_timer > 0 {
            self.bought_timer -= 1;
            self.vy *= 0.9;
            if self.bought_timer >= 31 {
                let chance = if self.bought_timer > 40 { 1 } else { 2 };
                if rand_range(chance) == 0 {
                    // W1's entrance requests two bubbles. These are only
                    // its four coordinate draws; board bubble limits and
                    // bubble constructors consume additional conditional
                    // RNG, so this does not establish source RNG parity.
                    let _first_x = self.widget_x + 55 - rand_range(60) as i32;
                    let _first_y = self.widget_y + 55 - rand_range(60) as i32;
                    let _second_x = self.widget_x + 45 - rand_range(40) as i32;
                    let _second_y = self.widget_y + 45 - rand_range(40) as i32;
                }
            }
        }
        if self.vx == 0.0 {
            self.y += 1.0 / self.speed_mod;
        }
        if self.vx == 1.0 {
            self.y += 0.75 / self.speed_mod;
        }
        if self.vx == 2.0 {
            self.y += 0.5 / self.speed_mod;
        }
        if self.vx == 3.0 {
            self.y += 0.25 / self.speed_mod;
        }
        self.x = self.x.clamp(10.0, 540.0);
        self.y = self.y.min(360.0);
        if self.bought_timer == 0 || self.vy <= 0.0 {
            self.y = self.y.max(95.0);
        }
        if self.x > 535.0 && self.vx > 0.1 {
            self.vx -= 0.1;
        }
        if self.x < 15.0 && self.vx < -0.1 {
            self.vx += 0.1;
        }
        self.animate();
        // Ordinary Oscar has mSpeedy=false; the nearby-prey counter is
        // maintained but does not select Fish's special speed divisor.
        self.x += self.vx / self.speed_mod;
        self.y += self.vy / self.speed_mod;
        self.widget_x = self.x as i32;
        self.widget_y = self.y as i32;
        result
    }

    fn update_hunger_animation(&mut self) {
        if self.hunger_animation_ticks > 0 {
            if self.hunger_shown {
                self.hunger_animation_ticks += 1;
                if self.hunger_animation_ticks > 5 {
                    self.hunger_animation_ticks = 0;
                }
            } else {
                self.hunger_animation_ticks -= 1;
            }
        }
    }

    fn hunt(&mut self, prey: &[OscarPrey], result: &mut OscarUpdate) -> bool {
        let Some((target, best_distance)) = self.nearest_prey(prey) else {
            return false;
        };
        if best_distance < 10000 {
            self.speedy_speed_ticks = 100;
        }
        if self.special_timer > 2 {
            self.special_timer = 0;
            self.steer(target);
            if self.vx_abs < 5 {
                self.vx_abs += 1;
            }
        }
        // Consumption walks board order independently of nearest selection.
        let center_x = self.x + 40.0;
        let center_y = self.y + 45.0;
        for candidate in prey.iter().filter(|candidate| candidate.eligible) {
            let px = f64::from(candidate.widget_x);
            let py = f64::from(candidate.widget_y);
            if center_x > px + 10.0
                && center_x < px + 70.0
                && center_y > py + 22.0
                && center_y < py + 58.0
            {
                let was_hungry = self.hunger_visible();
                self.speedy_speed_ticks = 0;
                if self.hunger < 300 {
                    self.hunger = 300;
                }
                self.hunger = (self.hunger + 900).min(1300);
                if was_hungry && !self.hunger_visible() {
                    self.hunger_shown = false;
                    self.hunger_animation_ticks = 5;
                }
                if self.eating_ticks == 0 {
                    self.eating_ticks = 8;
                }
                result.eaten_prey = Some(candidate.id);
                return true;
            }
            if self.eating_ticks == 0
                && center_x > px - 7.0
                && center_x < px + 87.0
                && center_y > py + 10.0
                && center_y < py + 70.0
            {
                self.eating_ticks = 20;
            }
        }
        true
    }

    fn nearest_prey<'a>(&self, prey: &'a [OscarPrey]) -> Option<(&'a OscarPrey, i64)> {
        let mut nearest = None;
        let mut best_distance = i64::MAX;
        for candidate in prey.iter().filter(|candidate| candidate.eligible) {
            // C++ assigns each double subtraction to an int before squaring.
            let dx = (self.x + 40.0 - f64::from(candidate.widget_x + 40)) as i32;
            let dy = (self.y + 40.0 - f64::from(candidate.widget_y + 40)) as i32;
            let distance = i64::from(dx) * i64::from(dx) + i64::from(dy) * i64::from(dy);
            if distance < best_distance {
                nearest = Some(candidate);
                best_distance = distance;
            }
        }
        nearest.map(|target| (target, best_distance))
    }

    fn steer(&mut self, target: &OscarPrey) {
        let center_x = (self.x + 40.0) as i32;
        let center_y = (self.y + 50.0) as i32;
        let tx = target.widget_x;
        let ty = target.widget_y;
        let urgent = self.hunger <= 300;
        let horizontal_limit = if urgent { 5.0 } else { 4.0 };
        let step = if urgent { 0.05 } else { 0.1 };
        if tx + 48 < center_x {
            if self.vx > -horizontal_limit {
                self.vx -= 1.5;
            }
        } else if tx + 32 > center_x {
            if self.vx < horizontal_limit {
                self.vx += 1.5;
            }
        } else if tx + 44 < center_x {
            if self.vx > -horizontal_limit {
                self.vx -= 0.2;
            }
        } else if tx + 36 > center_x {
            if self.vx < horizontal_limit {
                self.vx += 0.2;
            }
        } else if tx + 40 < center_x {
            if self.vx > -horizontal_limit {
                self.vx -= step;
            }
        } else if tx + 40 > center_x && self.vx < horizontal_limit {
            self.vx += step;
        }
        if ty + 46 < center_y {
            if self.vy > -3.0 {
                self.vy -= if urgent { 1.3 } else { 0.8 };
            }
        } else if ty + 34 > center_y {
            if self.vy < 4.0 {
                self.vy += if urgent { 1.5 } else { 1.3 };
            }
        } else if ty + 40 < center_y {
            if self.vy > -3.0 {
                self.vy -= if urgent { 0.5 } else { 0.3 };
            }
        } else if ty + 40 > center_y && self.vy < 4.0 {
            self.vy += if urgent { 0.7 } else { 0.5 };
        }
    }

    fn wander(&mut self) {
        match self.movement_state {
            0..=2 => {
                if self.bought_timer == 0 {
                    self.vy = if self.movement_state == 0 { 0.5 } else { -0.5 };
                }
                if self.special_timer > 39 {
                    self.special_timer = 0;
                    match self.movement_state {
                        0 if self.vx < -0.5 => self.vx += 0.5,
                        0 if self.vx > 0.5 => self.vx -= 0.5,
                        1 if self.vx < 1.0 => self.vx += 1.0,
                        1 if self.vx > 1.0 => self.vx -= 1.0,
                        2 if self.vx < -1.0 => self.vx += 1.0,
                        2 if self.vx > -1.0 => self.vx -= 1.0,
                        _ => {}
                    }
                    self.vx_abs = self.vx.abs() as u8;
                }
                self.y -= if self.movement_state == 0 { 0.25 } else { 0.5 } / self.speed_mod;
            }
            3..=4 => {
                if self.special_timer > 39 {
                    self.special_timer = 0;
                    let goal = if self.movement_state == 3 { -1.0 } else { 1.0 };
                    if self.vx < goal {
                        self.vx += 1.0;
                    } else if self.vx > goal {
                        self.vx -= 1.0;
                    }
                    if self.vy < 3.0 {
                        self.vy += 1.0;
                    } else if self.vy > 3.0 {
                        self.vy -= 1.0;
                    }
                    if self.vx_abs < 5 {
                        if self.vy >= 4.0 {
                            if self.y > 240.0 {
                                self.movement_state = 0;
                            }
                        } else {
                            self.vx_abs += 1;
                        }
                    } else {
                        self.vx_abs -= 1;
                    }
                }
                if self.y > 240.0 {
                    self.movement_state = 0;
                }
            }
            _ => {
                if self.bought_timer == 0 {
                    self.vy = if self.y >= 115.0 { -0.5 } else { -0.1 };
                }
                if self.special_timer > 39 {
                    self.special_timer = 0;
                    if self.x_direction > 0 {
                        self.vx += if self.vx < 0.0 { 2.0 } else { 1.0 };
                        self.vx_abs = self.vx.abs() as u8;
                        if self.x > 250.0 {
                            self.x_direction = -1;
                            self.vx -= 2.0;
                        }
                    } else {
                        self.vx -= if self.vx > 0.0 { 2.0 } else { 1.0 };
                        self.vx_abs = self.vx.abs() as u8;
                        if self.x < 175.0 {
                            self.x_direction = 1;
                            self.vx += 2.0;
                        }
                    }
                }
            }
        }
    }

    fn animate(&mut self) {
        if self.previous_vx < 0.0 && self.vx > 0.0 {
            self.turn_ticks = -20;
        } else if self.previous_vx > 0.0 && self.vx < 0.0 {
            self.turn_ticks = 20;
        }
        self.turn_ticks -= self.turn_ticks.signum();
        self.eating_ticks = self.eating_ticks.saturating_sub(1);
        self.frame = if self.turn_ticks > 0 {
            (9 - self.turn_ticks / 2) as u8
        } else if self.turn_ticks < 0 {
            (9 + self.turn_ticks / 2) as u8
        } else if self.eating_ticks > 0 {
            9 - self.eating_ticks / 2
        } else {
            self.swim_counter += if self.vx_abs < 2 { 1 } else { 2 };
            if self.swim_counter >= 20 {
                self.swim_counter = 0;
            }
            self.swim_counter / 2
        };
        if self.previous_vx != self.vx && self.previous_vx != 0.0 && self.vx != 0.0 {
            self.previous_vx = self.vx;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn actor() -> OscarState {
        OscarState::spawn_bought(7, &mut |_| 0)
    }

    #[test]
    fn bought_constructor_draws_inherited_rng_and_starts_entrance() {
        let mut draws = Vec::new();
        let actor = OscarState::spawn_bought(7, &mut |upper| {
            draws.push(upper);
            0
        });
        assert_eq!(draws, [520, 265, 2, 3, 200, 3, 10, 200, 200, 5, 10]);
        assert_eq!(
            (actor.widget_x, actor.widget_y, actor.vy, actor.bought_timer),
            (20, 40, 23.0, 45)
        );
        assert_eq!((actor.hunger, actor.coin_threshold), (600, 150));
        assert!(actor.validate().is_ok());
    }

    #[test]
    fn nearest_uses_truncation_but_contact_scans_order_and_strict_edges() {
        let mut actor = actor();
        actor.x = 100.9;
        actor.y = 100.9;
        actor.widget_x = 100;
        actor.widget_y = 100;
        actor.hunger = 400;
        let prey = [
            OscarPrey {
                id: 1,
                widget_x: 110,
                widget_y: 110,
                eligible: false,
            },
            OscarPrey {
                id: 2,
                widget_x: 90,
                widget_y: 100,
                eligible: true,
            },
            OscarPrey {
                id: 3,
                widget_x: 100,
                widget_y: 100,
                eligible: true,
            },
        ];
        assert_eq!(actor.nearest_prey(&prey).unwrap().0.id, 3);
        let mut update = OscarUpdate::default();
        assert!(actor.hunt(&prey, &mut update));
        assert_eq!(update.eaten_prey, Some(2)); // first contact, despite nearest id3
        assert_eq!(actor.hunger, 1300);
        actor.x = 100.9;
        actor.y = 100.9;
        let tie = [
            OscarPrey {
                id: 5,
                widget_x: 100,
                widget_y: 100,
                eligible: true,
            },
            OscarPrey {
                id: 6,
                widget_x: 101,
                widget_y: 101,
                eligible: true,
            },
        ];
        assert_eq!(actor.nearest_prey(&tie).unwrap().0.id, 5); // subpixel differences truncate to zero
        actor.x = 130.0;
        actor.y = 107.0;
        actor.hunger = 400;
        let mut edge = OscarUpdate::default();
        actor.hunt(
            &[OscarPrey {
                id: 4,
                widget_x: 100,
                widget_y: 130,
                eligible: true,
            }],
            &mut edge,
        );
        assert_eq!(edge.eaten_prey, None); // center X == prey right edge
    }

    #[test]
    fn alien_freezes_hunger_and_diamond_but_not_hunt() {
        let mut actor = actor();
        actor.x = 100.0;
        actor.y = 100.0;
        actor.widget_x = 100;
        actor.widget_y = 100;
        actor.hunger = 400;
        actor.coin_timer = actor.coin_threshold - 1;
        let prey = [OscarPrey {
            id: 9,
            widget_x: 100,
            widget_y: 100,
            eligible: true,
        }];
        let update = actor.tick(&prey, true, &mut |_| 1);
        assert_eq!(update.eaten_prey, Some(9));
        assert_eq!(actor.hunger, 1300);
        assert_eq!(update.diamond, None);
        assert_eq!(actor.coin_timer, actor.coin_threshold - 1);
    }

    #[test]
    fn due_diamond_survives_starvation_and_uses_old_widget_origin() {
        let mut actor = actor();
        actor.hunger = 1;
        actor.coin_timer = actor.coin_threshold - 1;
        let old = (actor.widget_x, actor.widget_y);
        let update = actor.tick(&[], false, &mut |_| 1);
        assert!(update.died);
        assert_eq!(update.diamond, Some((old.0 + 5, old.1 + 10)));
        assert!(!actor.alive);
        assert_eq!(actor.tick(&[], false, &mut |_| 1), OscarUpdate::default());
    }

    #[test]
    fn inherited_wander_keeps_near_zero_and_overshoots_targets() {
        let mut actor = actor();
        actor.vx = -0.1;
        actor.movement_state = 0;
        actor.special_timer = 40;
        actor.wander();
        assert_eq!(actor.vx, -0.1);

        actor.vx = 0.5;
        actor.movement_state = 1;
        actor.special_timer = 40;
        actor.wander();
        assert_eq!(actor.vx, 1.5);

        actor.vx = -0.5;
        actor.movement_state = 2;
        actor.special_timer = 40;
        actor.wander();
        assert_eq!(actor.vx, -1.5);

        actor.movement_state = 3;
        actor.vy = 5.0;
        actor.vx_abs = 4;
        actor.special_timer = 40;
        actor.y = 200.0;
        actor.wander();
        assert_eq!((actor.vy, actor.vx_abs), (4.0, 4));
        actor.vy = 4.0;
        actor.special_timer = 40;
        actor.wander();
        assert_eq!((actor.vy, actor.vx_abs), (3.0, 5));

        actor.movement_state = 5;
        actor.x = 260.0;
        actor.vx = -0.1;
        actor.x_direction = 1;
        actor.special_timer = 40;
        actor.wander();
        assert_eq!(actor.vx_abs, 1); // captured before boundary reversal
        assert_eq!(actor.x_direction, -1);
        assert!((actor.vx + 0.1).abs() < 1e-9);

        actor.x = 170.0;
        actor.vx = 0.1;
        actor.x_direction = -1;
        actor.special_timer = 40;
        actor.wander();
        assert_eq!(actor.vx_abs, 1);
        assert_eq!(actor.x_direction, 1);
        assert!((actor.vx - 0.1).abs() < 1e-9);
    }

    #[test]
    fn swim_counter_resets_at_twenty_instead_of_wrapping_modulo() {
        let mut actor = actor();
        actor.vx = 1.0;
        actor.previous_vx = 1.0;
        actor.vx_abs = 2;
        actor.swim_counter = 19;
        actor.animate();
        assert_eq!((actor.swim_counter, actor.frame), (0, 0));
    }

    #[test]
    fn hungry_sheet_fades_in_and_fades_out_after_a_meal() {
        let mut actor = actor();
        actor.bought_timer = 0;
        actor.hunger = 305;
        actor.tick(&[], false, &mut |_| 1);
        assert_eq!(actor.hunger, 304);
        assert_eq!(actor.hunger_overlay_alpha(), 0.2);
        assert!(!actor.hunger_visible());
        for expected in [0.4, 0.6, 0.8, 1.0] {
            actor.tick(&[], false, &mut |_| 1);
            assert!((actor.hunger_overlay_alpha() - expected).abs() < 1e-6);
        }
        actor.tick(&[], false, &mut |_| 1);
        assert_eq!(actor.hunger_overlay_alpha(), 0.0);
        assert!(actor.hunger_visible());

        actor.x = 100.0;
        actor.y = 100.0;
        let prey = [OscarPrey {
            id: 9,
            widget_x: 100,
            widget_y: 100,
            eligible: true,
        }];
        let mut update = OscarUpdate::default();
        actor.hunt(&prey, &mut update);
        assert_eq!(update.eaten_prey, Some(9));
        assert_eq!(actor.hunger_overlay_alpha(), 1.0);
        assert!(!actor.hunger_visible());
    }

    #[test]
    fn corpse_uses_pre_motion_death_pose_and_expires_after_floor_sink() {
        let mut actor = actor();
        actor.hunger = 1;
        actor.vx = -0.1;
        actor.movement_state = 1;
        actor.special_timer = 40;
        let update = actor.tick(&[], false, &mut |_| 1);
        assert!(update.died);
        assert!(actor.vx > 0.0); // inherited wander happened after Fish::Die
        let mut corpse = DeadOscar::from_live(&actor);
        assert_eq!((corpse.widget_x, corpse.widget_y), (20, 40));
        assert!(!corpse.facing_right); // old VX, before wander reversed it
        assert_eq!(corpse.remaining_ticks, 125);
        assert!(corpse.validate().is_ok());
        let mut updates = 0;
        while !corpse.tick() {
            updates += 1;
            assert!(updates < 200);
        }
        assert_eq!(corpse.remaining_ticks, 0);
        assert_eq!(corpse.opacity, 0.0);
    }
}
