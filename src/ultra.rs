//! Ordinary Tank4 Ultravore, source-informed from pinned W1 Ultra/Fish/DeadFish.
//! Typed primary association and numeric findings are in PB59–61 and its follow-up.
//! Board owns live Oscar eligibility/removal, treasure IDs, RNG and corpse membership.
//! Existing original Rust Oscar movement/death code supplies the checked shared Fish
//! behavior; Ultra-specific constructor, bounds, diet, geometry and output differ.
//! Ordinary non-Voracious actors have no scream or special virtual-tank diet.
//! Bubble particles and exact retail RNG/widget scheduling remain unverified.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UltraPrey {
    pub id: u64,
    pub widget_x: i32,
    pub widget_y: i32,
    /// Caller restricts this to a live Oscar with no eat delay.
    pub eligible: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UltraUpdate {
    pub eaten_prey: Option<u64>,
    pub treasure: Option<(i32, i32)>,
    pub died: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UltraPose {
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
pub struct UltraState {
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

/// The inherited ordinary DeadFish body specialized to Ultra's atlas row.
/// The board stores this separately from live Ultras until `tick` expires it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeadUltra {
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

impl DeadUltra {
    /// Only call after `UltraState::tick` reports `died`. Fish::Die runs in
    /// the hunger hook, so this uses the saved pose before that tick's motion.
    pub fn from_live(actor: &UltraState) -> Self {
        let pose = actor
            .death_pose
            .expect("Ultra corpse requires a death-tick pose");
        Self::from_pose(actor.id, pose)
    }

    /// Missile impact calls ordinary Fish::Die at the current live pose,
    /// outside Ultra's starvation hook and its cached pre-motion snapshot.
    pub fn from_impact(actor: &UltraState) -> Self {
        Self::from_pose(
            actor.id,
            DeathPose {
                x: actor.widget_x,
                y: actor.widget_y,
                vx: actor.vx,
                vy: actor.vy,
                speed_mod: actor.speed_mod,
                facing_right: actor.vx >= 0.0,
            },
        )
    }

    fn from_pose(id: u64, pose: DeathPose) -> Self {
        let vy = pose.vy
            - if pose.x < 115 || pose.vy < -3.0 {
                1.0
            } else {
                2.0
            };
        Self {
            id,
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
            // A death snapshot precedes the next clamp and can retain the
            // live actor's post-integration overshoot above its floor.
            || !(0.0..=480.0).contains(&self.y)
            || self.widget_x != self.x as i32
            || self.widget_y != self.y as i32
            || ![1.6, 1.8, 2.0].contains(&self.speed_mod)
            || self.frame > 9
            || self.remaining_ticks > 125
            || !self.opacity.is_finite()
            || !(0.0..=1.0).contains(&self.opacity)
        {
            return Err("invalid ordinary Ultra corpse save state".into());
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
        if remaining > 105 || self.y > 300.0 {
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
        self.y = (self.y + self.vy / self.speed_mod).clamp(85.0, 310.0);
        self.widget_x = self.x as i32;
        self.widget_y = self.y as i32;
        false
    }
}

impl UltraState {
    /// The two board position draws precede the common Fish constructor's
    /// six draws, Ultra's hunger and coin overrides, then the entrance draws.
    /// The constructor Y draw is consumed even though Board immediately
    /// places a bought Ultra at Y=40. This is W1 order, not retail RNG parity.
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
        let _common_coin_threshold = rand_range(200);
        let hunger = rand_range(200) as i32 + 600;
        let coin_threshold = rand_range(250) as u16 + 200;
        let vy = rand_range(5) as f64 + 25.0;
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

    pub fn sprite_pose(&self) -> UltraPose {
        if self.turn_ticks != 0 {
            UltraPose::Turn
        } else if self.eating_ticks != 0 {
            UltraPose::Eat
        } else {
            UltraPose::Swim
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

    /// Ultra.cpp overlays the hungry pose during its five-update transition.
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
            || !(200..=449).contains(&self.coin_threshold)
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
            return Err("invalid ordinary Ultra save state".into());
        }
        Ok(())
    }

    pub fn tick(
        &mut self,
        prey: &[UltraPrey],
        alien_present: bool,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> UltraUpdate {
        if !self.alive {
            return UltraUpdate::default();
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
        let mut result = UltraUpdate::default();
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
                result.treasure = Some((self.widget_x + 40, self.widget_y + 90));
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
        self.x = self.x.clamp(0.0, 480.0);
        self.y = self.y.min(310.0);
        if self.bought_timer == 0 || self.vy <= 0.0 {
            self.y = self.y.max(75.0);
        }
        if self.x > 475.0 && self.vx > 0.1 {
            self.vx -= 0.1;
        }
        if self.x < 5.0 && self.vx < -0.1 {
            self.vx += 0.1;
        }
        self.animate();
        // Ordinary Ultra has mSpeedy=false; the nearby-prey counter is
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

    fn hunt(&mut self, prey: &[UltraPrey], result: &mut UltraUpdate) -> bool {
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
        let center_x = self.x + 80.0;
        let center_y = self.y + 90.0;
        for candidate in prey.iter().filter(|candidate| candidate.eligible) {
            let px = f64::from(candidate.widget_x);
            let py = f64::from(candidate.widget_y);
            if center_x > px - 40.0
                && center_x < px + 120.0
                && center_y > py
                && center_y < py + 80.0
            {
                let was_hungry = self.hunger_visible();
                self.speedy_speed_ticks = 0;
                // PB05 floors only virtual/relax actors in FUN004d6a30.
                // Ordinary Adventure has identity -1 and the App flag clear.
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
                && center_x > px - 70.0
                && center_x < px + 150.0
                && center_y > py - 20.0
                && center_y < py + 100.0
            {
                self.eating_ticks = 20;
            }
        }
        true
    }

    fn nearest_prey<'a>(&self, prey: &'a [UltraPrey]) -> Option<(&'a UltraPrey, i64)> {
        let mut nearest = None;
        let mut best_distance = i64::MAX;
        for candidate in prey.iter().filter(|candidate| candidate.eligible) {
            // C++ assigns each double subtraction to an int before squaring.
            let dx = (self.x + 80.0 - f64::from(candidate.widget_x + 40)) as i32;
            let dy = (self.y + 80.0 - f64::from(candidate.widget_y + 40)) as i32;
            let distance = i64::from(dx) * i64::from(dx) + i64::from(dy) * i64::from(dy);
            if distance < best_distance {
                nearest = Some(candidate);
                best_distance = distance;
            }
        }
        nearest.map(|target| (target, best_distance))
    }

    fn steer(&mut self, target: &UltraPrey) {
        let center_x = (self.x + 80.0) as i32;
        let center_y = (self.y + 100.0) as i32;
        let tx = target.widget_x;
        let ty = target.widget_y;
        let urgent = self.hunger <= 300;
        let limit = if urgent { 5.0 } else { 4.0 };
        let step = if urgent { 0.05 } else { 0.1 };
        let horizontal = if tx + 44 < center_x {
            -1.5
        } else if tx + 36 > center_x {
            1.5
        } else if tx + 42 < center_x {
            -0.2
        } else if tx + 38 > center_x {
            0.2
        } else if tx + 40 < center_x {
            -step
        } else if tx + 40 > center_x {
            step
        } else {
            0.0
        };
        if (horizontal < 0.0 && self.vx > -limit) || (horizontal > 0.0 && self.vx < limit) {
            self.vx += horizontal;
        }
        let vertical = if urgent {
            if ty + 40 < center_y {
                -1.3
            } else if ty + 40 > center_y {
                1.3
            } else {
                0.0
            }
        } else if ty + 43 < center_y {
            -0.8
        } else if ty + 37 > center_y {
            1.3
        } else if ty + 40 < center_y {
            -0.3
        } else if ty + 40 > center_y {
            0.5
        } else {
            0.0
        };
        if (vertical < 0.0 && self.vy > -3.0) || (vertical > 0.0 && self.vy < 4.0) {
            self.vy += vertical;
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
        if self.turn_ticks != 0 {
            // W1 Ultra::FishUpdateAnimation clears its meal pose while turning.
            self.eating_ticks = 0;
        }
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

    fn settled() -> UltraState {
        let mut actor = UltraState::spawn_bought(1, &mut |_| 0);
        actor.x = 100.0;
        actor.y = 150.0;
        actor.widget_x = 100;
        actor.widget_y = 150;
        actor.bought_timer = 0;
        actor.vx = 0.0;
        actor.vy = 0.0;
        actor
    }

    #[test]
    fn constructor_consumes_checked_shared_and_ultra_draws_in_order() {
        let mut bounds = Vec::new();
        let actor = UltraState::spawn_bought(7, &mut |upper| {
            bounds.push(upper);
            upper - 1
        });
        assert_eq!(bounds, [520, 265, 2, 3, 200, 3, 10, 200, 200, 250, 5, 10]);
        assert_eq!(
            (
                actor.widget_x,
                actor.widget_y,
                actor.hunger,
                actor.coin_threshold
            ),
            (539, 40, 799, 449)
        );
        assert_eq!((actor.vy, actor.bought_timer), (29.0, 54));
        actor.validate().unwrap();
    }

    #[test]
    fn group_freezes_hunger_and_income_but_allows_ordered_oscar_meal() {
        let mut actor = settled();
        actor.hunger = 100;
        actor.coin_timer = actor.coin_threshold - 1;
        actor.eating_ticks = 8;
        let prey = [
            UltraPrey {
                id: 10,
                widget_x: 140,
                widget_y: 200,
                eligible: true,
            },
            UltraPrey {
                id: 11,
                widget_x: 140,
                widget_y: 190,
                eligible: true,
            },
        ];
        let update = actor.tick(&prey, true, &mut |_| 1);
        assert_eq!(update.eaten_prey, Some(10));
        assert_eq!(actor.hunger, 1000);
        assert_eq!(actor.coin_timer, actor.coin_threshold - 1);
        assert_eq!(update.treasure, None);
    }

    #[test]
    fn strict_bite_edges_and_eat_delay_are_independent_of_nearest_target() {
        let mut actor = settled();
        actor.hunger = 499;
        let prey = [
            UltraPrey {
                id: 10,
                widget_x: 60,
                widget_y: 200,
                eligible: true,
            },
            UltraPrey {
                id: 11,
                widget_x: 140,
                widget_y: 200,
                eligible: false,
            },
            UltraPrey {
                id: 12,
                widget_x: 140,
                widget_y: 200,
                eligible: true,
            },
        ];
        let update = actor.tick(&prey, true, &mut |_| 1);
        assert_eq!(update.eaten_prey, Some(12));
        assert_eq!(actor.hunger, 1300);
    }

    #[test]
    fn due_treasure_uses_prior_widget_and_still_runs_on_starvation_tail() {
        let mut actor = settled();
        actor.hunger = 1;
        actor.coin_timer = actor.coin_threshold - 1;
        let update = actor.tick(&[], false, &mut |_| 1);
        assert!(update.died);
        assert_eq!(update.treasure, Some((140, 240)));
        let corpse = DeadUltra::from_live(&actor);
        assert_eq!(
            (corpse.widget_x, corpse.widget_y, corpse.remaining_ticks),
            (100, 150, 125)
        );
        corpse.validate().unwrap();
    }

    #[test]
    fn turn_clears_meal_pose_and_uses_movement_direction_projection() {
        let mut actor = settled();
        actor.previous_vx = -1.0;
        actor.vx = 1.0;
        actor.eating_ticks = 8;
        actor.animate();
        assert_eq!(
            (actor.turn_ticks, actor.eating_ticks, actor.frame),
            (-19, 0, 0)
        );
        assert!(!actor.facing_right());
        actor.turn_ticks = 0;
        assert!(actor.facing_right());
    }

    #[test]
    fn post_clamp_floor_overshoot_is_a_valid_immediate_death_snapshot() {
        let mut actor = settled();
        actor.y = 310.0;
        actor.widget_y = 310;
        actor.hunger = 2;
        actor.vy = 3.0;
        actor.bought_timer = 1;
        assert!(!actor.tick(&[], false, &mut |_| 1).died);
        assert!(actor.y > 310.0);
        actor.validate().unwrap();
        assert!(actor.tick(&[], false, &mut |_| 1).died);
        let mut corpse = DeadUltra::from_live(&actor);
        assert_eq!(corpse.widget_y, 311);
        corpse.validate().unwrap();
        corpse.tick();
        assert_eq!(corpse.y, 310.0);
        corpse.validate().unwrap();
    }
}
