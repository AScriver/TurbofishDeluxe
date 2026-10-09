//! Ordinary Adventure fish-shaped pets, limited to Itchy, Prego, and Zorf.
//! Behavioral rules are derived from pinned WinFish W1 `FishTypePet.cpp`,
//! `Fish.cpp`, and `Board.cpp` (revision f919b3c). Installed-binary coverage
//! for their full movement and animation remains partial. Board membership,
//! alien health, guppy construction, sounds, and effects belong to the caller.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FishPetKind {
    Itchy,
    Prego,
    Zorf,
    Vert,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PetAlienView {
    pub id: u64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub healing: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FishPetUpdate {
    pub damaged_alien: Option<u64>,
    pub born_at: Option<(i32, i32)>,
    pub free_food: Option<ZorfFoodRequest>,
    /// Gold appears at the prior integer widget, before this pet moves.
    pub gold_at: Option<(i32, i32)>,
    /// The board applies its shared eleven-update punch sound delay.
    pub punch_sound: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZorfHungryView {
    pub id: u64,
    pub hunger: i32,
    pub ordinary_diet: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZorfFoodRequest {
    pub x: i32,
    pub y: i32,
    /// Source direction: 1 travels left, 2 right.
    pub direction: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FishPetPose {
    Swim,
    Turn,
    Birth,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FishPetState {
    pub id: u64,
    pub kind: FishPetKind,
    pub x: f64,
    pub y: f64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub vx: f64,
    pub vy: f64,
    pub frame: u8,
    pub turn_ticks: i8,
    pub birth_timer: u16,
    pub birth_threshold: u16,
    #[serde(default)]
    pub food_timer: i32,
    pub coin_timer: u16,
    speed_mod: f64,
    previous_vx: f64,
    movement_state: u8,
    movement_timer: u8,
    special_timer: u32,
    x_direction: i8,
    vx_abs: u8,
    swim_counter: u8,
}

impl FishPetState {
    /// Board's two spawn coordinates precede six unused/common Fish draws.
    /// In particular, a spawn Y up to 539 survives until the first update's
    /// pre-integration clamp. Zorf overwrites the drawn speed divisor with 3.
    pub fn spawn_tank1(
        id: u64,
        kind: FishPetKind,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> Self {
        let widget_x = rand_range(265) as i32 + 105;
        let widget_y = rand_range(520) as i32 + 20;
        let left = rand_range(2) != 0;
        let speed_mod = match rand_range(3) {
            0 => 2.0,
            1 => 1.8,
            _ => 1.6,
        };
        let _unused_hunger = rand_range(200);
        let _unused_growth = rand_range(3);
        let movement_state = rand_range(10) as u8;
        let _unused_coin_threshold = rand_range(200);
        Self {
            id,
            kind,
            x: f64::from(widget_x),
            y: f64::from(widget_y),
            widget_x,
            widget_y,
            vx: if left { -0.1 } else { 0.0 },
            vy: -0.5,
            frame: 0,
            turn_ticks: 0,
            birth_timer: 0,
            birth_threshold: 930,
            food_timer: 0,
            coin_timer: 0,
            speed_mod: if kind == FishPetKind::Zorf {
                3.0
            } else {
                speed_mod
            },
            previous_vx: if left { -1.0 } else { 1.0 },
            movement_state,
            movement_timer: 0,
            special_timer: 40,
            x_direction: 1,
            vx_abs: 0,
            swim_counter: 0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0
            || ![
                self.x,
                self.y,
                self.vx,
                self.vy,
                self.previous_vx,
                self.speed_mod,
            ]
            .into_iter()
            .all(f64::is_finite)
            || !(0.0..=550.0).contains(&self.x)
            || !(0.0..=550.0).contains(&self.y)
            || self.widget_x != self.x as i32
            || self.widget_y != self.y as i32
            || !(if self.kind == FishPetKind::Zorf {
                self.speed_mod == 3.0
            } else {
                [1.6, 1.8, 2.0].contains(&self.speed_mod)
            })
            || self.movement_state > 9
            || self.movement_timer > 20
            || !matches!(self.x_direction, -1 | 1)
            || self.vx_abs > 64
            || self.swim_counter > 19
            || self.frame > 9
            || !(-19..=19).contains(&self.turn_ticks)
            || (self.kind == FishPetKind::Prego
                && (!matches!(self.birth_threshold, 930 | 1230 | 2000)
                    || self.birth_timer >= self.birth_threshold))
            || (self.kind != FishPetKind::Prego && self.birth_timer != 0)
            || (self.kind == FishPetKind::Zorf && self.food_timer < -10)
            || (self.kind != FishPetKind::Zorf && self.food_timer != 0)
            || (self.kind == FishPetKind::Vert && self.coin_timer >= 216)
            || (self.kind != FishPetKind::Vert && self.coin_timer != 0)
        {
            return Err("invalid ordinary fish pet save state".into());
        }
        Ok(())
    }

    pub fn facing_right(&self) -> bool {
        if self.turn_ticks != 0 {
            return self.turn_ticks > 0;
        }
        self.vx >= 0.0 && (self.vx as i32 != 0 || self.previous_vx >= 0.0)
    }

    pub fn sprite_pose(&self) -> FishPetPose {
        if self.turn_ticks != 0 {
            FishPetPose::Turn
        } else if self.kind == FishPetKind::Prego && self.birth_timer > self.birth_threshold - 200 {
            FishPetPose::Birth
        } else {
            FishPetPose::Swim
        }
    }

    pub fn sprite_row(&self, aliens_present: bool) -> u8 {
        match self.kind {
            FishPetKind::Itchy => {
                (if aliens_present { 2 } else { 0 }) + u8::from(self.turn_ticks != 0)
            }
            FishPetKind::Prego => match self.sprite_pose() {
                FishPetPose::Swim => 0,
                FishPetPose::Turn => 2,
                FishPetPose::Birth
                    if self.birth_timer < self.birth_threshold - 190
                        || self.birth_timer > self.birth_threshold - 5 =>
                {
                    3
                }
                FishPetPose::Birth => 1,
            },
            FishPetKind::Zorf => {
                if self.turn_ticks != 0 {
                    1
                } else if self.food_timer < 0 {
                    2
                } else {
                    0
                }
            }
            FishPetKind::Vert => u8::from(self.turn_ticks != 0),
        }
    }

    pub fn sprite_frame(&self) -> u8 {
        if self.kind == FishPetKind::Zorf && self.sprite_row(false) == 2 {
            (self.food_timer + 10) as u8
        } else if self.kind == FishPetKind::Prego && self.sprite_row(false) == 3 {
            if self.birth_timer < self.birth_threshold - 190 {
                ((self.birth_timer as i32 - self.birth_threshold as i32 + 200) / 2) as u8
            } else {
                (self.birth_timer as i32 - self.birth_threshold as i32 + 10) as u8
            }
        } else {
            self.frame
        }
    }

    pub fn tick(
        &mut self,
        aliens: &[PetAlienView],
        guppy_count: usize,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> FishPetUpdate {
        assert_ne!(
            self.kind,
            FishPetKind::Zorf,
            "Zorf needs the ordered hungry-fish view"
        );
        self.tick_inner(aliens, guppy_count, &[], rand_range)
    }

    pub fn tick_zorf(
        &mut self,
        aliens: &[PetAlienView],
        hungry: &[ZorfHungryView],
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> FishPetUpdate {
        assert_eq!(self.kind, FishPetKind::Zorf, "tick_zorf requires Zorf");
        self.tick_inner(aliens, 0, hungry, rand_range)
    }

    fn tick_inner(
        &mut self,
        aliens: &[PetAlienView],
        guppy_count: usize,
        hungry: &[ZorfHungryView],
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> FishPetUpdate {
        let mut update = FishPetUpdate::default();
        let hunting = self.kind == FishPetKind::Itchy && !aliens.is_empty();
        if hunting {
            self.hunt(aliens, &mut update);
        } else {
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
        if self.kind == FishPetKind::Prego && aliens.is_empty() {
            self.birth_timer += 1;
            if self.birth_timer >= self.birth_threshold {
                self.birth_timer = 0;
                update.born_at = Some((self.widget_x + 7, self.widget_y + 25));
                self.birth_threshold = if guppy_count + 1 > 10 {
                    2000
                } else if guppy_count + 1 > 5 {
                    1230
                } else {
                    930
                };
            }
        }
        if self.kind == FishPetKind::Zorf && aliens.is_empty() {
            self.food_timer = self.food_timer.saturating_add(1);
            if self.food_timer >= 65
                && hungry
                    .iter()
                    .any(|fish| fish.hunger < 300 && fish.ordinary_diet)
            {
                self.food_timer = -10;
                update.free_food = Some(ZorfFoodRequest {
                    x: self.widget_x + 15,
                    y: self.widget_y + 10,
                    direction: if self.vx < 0.0 { 1 } else { 2 },
                });
            }
        }
        // FishTypePet::Update skips DropCoin while any alien is registered.
        // The DropCoin helper owns both the counter and its threshold reset.
        if self.kind == FishPetKind::Vert && aliens.is_empty() {
            self.coin_timer += 1;
            if self.coin_timer >= 216 {
                self.coin_timer = 0;
                update.gold_at = Some((self.widget_x + 15, self.widget_y + 10));
            }
        }
        match self.vx {
            0.0 => self.y += 1.0 / self.speed_mod,
            1.0 => self.y += 0.75 / self.speed_mod,
            2.0 => self.y += 0.5 / self.speed_mod,
            3.0 => self.y += 0.25 / self.speed_mod,
            _ => {}
        }
        self.x = self.x.clamp(10.0, 540.0);
        self.y = self.y.clamp(
            95.0,
            if self.kind == FishPetKind::Prego {
                360.0
            } else if self.kind == FishPetKind::Zorf {
                270.0
            } else {
                370.0
            },
        );
        if self.x > 535.0 && self.vx > 0.1 {
            self.vx -= 0.1;
        }
        if self.x < 15.0 && self.vx < -0.1 {
            self.vx += 0.1;
        }
        self.animate();
        self.x += self.vx / self.speed_mod;
        self.y += self.vy / self.speed_mod;
        self.widget_x = self.x as i32;
        self.widget_y = self.y as i32;
        update
    }

    fn hunt(&mut self, aliens: &[PetAlienView], update: &mut FishPetUpdate) {
        let mut nearest = None;
        let mut best_distance = 10_000;
        for alien in aliens.iter().filter(|alien| !alien.healing) {
            let dx = (self.x + 40.0 - f64::from(alien.widget_x + 80)) as i32;
            let dy = (self.y + 40.0 - f64::from(alien.widget_y + 80)) as i32;
            let distance = ((f64::from(dx * dx + dy * dy)).sqrt()) as i32;
            if distance < best_distance {
                best_distance = distance;
                nearest = Some(alien);
            }
        }
        if let Some(target) = nearest.filter(|_| self.special_timer >= 5) {
            self.special_timer = 0;
            let cx = self.x + 40.0;
            let cy = self.y + 40.0;
            let tx = f64::from(target.widget_x + 80);
            let ty = f64::from(target.widget_y + 80);
            if cx < tx && self.vx < 10.0 {
                self.vx += 2.5;
            } else if cx > tx && self.vx > -10.0 {
                self.vx -= 2.5;
            }
            if cy < ty && self.vy < 4.0 {
                self.vy += 1.5;
            } else if cy > ty && self.vy > -4.0 {
                self.vy -= 1.5;
            }
            if self.vx_abs < 5 {
                self.vx_abs += 1;
            }
        }
        // Contact walks registration order, independently of the nearest
        // steering target. A pending-death alien remains eligible here.
        if let Some(alien) = aliens.iter().find(|alien| {
            !alien.healing
                && (self.x + 40.0 - f64::from(alien.widget_x + 80)).abs() < 20.0
                && (self.y + 40.0 - f64::from(alien.widget_y + 80)).abs() < 20.0
        }) {
            update.damaged_alien = Some(alien.id);
            update.punch_sound = true;
        }
    }

    fn wander(&mut self) {
        match self.movement_state {
            0 => {
                self.vy = 0.5;
                if self.special_timer >= 40 {
                    self.special_timer = 0;
                    if self.vx < -0.5 {
                        self.vx += 0.5;
                    } else if self.vx > 0.5 {
                        self.vx -= 0.5;
                    }
                    self.vx_abs = self.vx.abs() as u8;
                }
                self.y -= 0.25 / self.speed_mod;
            }
            1 | 2 => {
                self.vy = -0.5;
                if self.special_timer >= 40 {
                    self.special_timer = 0;
                    let goal = if self.movement_state == 1 { 1.0 } else { -1.0 };
                    if self.vx < goal {
                        self.vx += 1.0;
                    } else if self.vx > goal {
                        self.vx -= 1.0;
                    }
                    self.vx_abs = self.vx.abs() as u8;
                }
                self.y -= 0.5 / self.speed_mod;
            }
            3 | 4 => {
                if self.special_timer >= 40 {
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
                    if self.vx_abs >= 5 {
                        self.vx_abs -= 1;
                    } else if self.vy < 4.0 {
                        self.vx_abs += 1;
                    }
                }
                if self.y > 240.0 {
                    self.movement_state = 0;
                }
            }
            _ => {
                self.vy = if self.y < 115.0 { -0.1 } else { -0.5 };
                if self.special_timer >= 40 {
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
        let reversing =
            (self.previous_vx < 0.0 && self.vx > 0.0) || (self.previous_vx > 0.0 && self.vx < 0.0);
        if reversing {
            if self.kind == FishPetKind::Prego && self.birth_timer > self.birth_threshold - 220 {
                self.vx = 0.0;
            } else {
                self.turn_ticks = if self.vx > 0.0 { -20 } else { 20 };
            }
        }
        if self.turn_ticks > 0 {
            self.turn_ticks -= 1;
        } else if self.turn_ticks < 0 {
            self.turn_ticks += 1;
        }
        if self.turn_ticks != 0 {
            self.frame = if self.turn_ticks > 0 {
                9 - (self.turn_ticks / 2) as u8
            } else {
                (9 + self.turn_ticks / 2) as u8
            };
        } else {
            self.swim_counter += if self.kind == FishPetKind::Zorf || self.vx_abs <= 1 {
                1
            } else {
                2
            };
            if self.swim_counter > 19 {
                self.swim_counter = 0;
            }
            self.frame = if self.kind == FishPetKind::Vert {
                if self.swim_counter < 10 {
                    self.swim_counter
                } else {
                    19 - self.swim_counter
                }
            } else {
                self.swim_counter / 2
            };
            if self.kind == FishPetKind::Zorf && self.food_timer == -1 {
                self.swim_counter = 10;
            }
        }
        if self.previous_vx != self.vx && self.previous_vx != 0.0 && self.vx != 0.0 {
            self.previous_vx = self.vx;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn actor(kind: FishPetKind) -> FishPetState {
        let mut rng = |_: u64| 0;
        FishPetState::spawn_tank1(1, kind, &mut rng)
    }

    #[test]
    fn spawn_draws_and_first_clamp_preserve_original_out_of_bounds_y() {
        let mut ranges = Vec::new();
        let mut rng = |range| {
            ranges.push(range);
            range - 1
        };
        let mut pet = FishPetState::spawn_tank1(1, FishPetKind::Prego, &mut rng);
        assert_eq!(ranges, [265, 520, 2, 3, 200, 3, 10, 200]);
        assert_eq!((pet.widget_x, pet.widget_y), (369, 539));
        assert!(pet.validate().is_ok());
        pet.tick(&[], 0, &mut |_| 1);
        assert!(pet.y < 361.0);
    }

    #[test]
    fn itchy_steers_on_fifth_timer_and_contacts_strictly_in_registration_order() {
        let mut pet = actor(FishPetKind::Itchy);
        pet.x = 100.0;
        pet.y = 100.0;
        pet.widget_x = 100;
        pet.widget_y = 100;
        pet.special_timer = 4;
        let aliens = [
            PetAlienView {
                id: 2,
                widget_x: 60,
                widget_y: 60,
                healing: true,
            },
            PetAlienView {
                id: 3,
                widget_x: 40,
                widget_y: 60,
                healing: false,
            },
            PetAlienView {
                id: 4,
                widget_x: 60,
                widget_y: 60,
                healing: false,
            },
        ];
        let result = pet.tick(&aliens, 0, &mut |_| 1);
        assert_eq!(result.damaged_alien, Some(4));
        assert!(result.punch_sound);
        assert_eq!(pet.vx, 0.0);
        assert_eq!(pet.special_timer, 5);
        pet.tick(&aliens, 0, &mut |_| 1);
        assert_eq!(pet.special_timer, 1);
    }

    #[test]
    fn prego_birth_uses_old_widget_origin_and_post_birth_count() {
        let mut pet = actor(FishPetKind::Prego);
        pet.birth_timer = 929;
        pet.y = 400.0;
        pet.widget_y = 400;
        let result = pet.tick(&[], 5, &mut |_| 1);
        assert_eq!(result.born_at, Some((pet.widget_x + 7, 425)));
        assert_eq!(pet.birth_threshold, 1230);
        assert_eq!(pet.birth_timer, 0);
        pet.birth_timer = 1229;
        let result = pet.tick(&[], 10, &mut |_| 1);
        assert!(result.born_at.is_some());
        assert_eq!(pet.birth_threshold, 2000);
    }

    #[test]
    fn registered_alien_freezes_prego_birth_and_swim_counter_resets_on_overshoot() {
        let mut pet = actor(FishPetKind::Prego);
        pet.birth_timer = 929;
        pet.swim_counter = 19;
        pet.vx_abs = 2;
        pet.special_timer = 0;
        pet.movement_state = 0;
        let alien = PetAlienView {
            id: 2,
            widget_x: 0,
            widget_y: 0,
            healing: false,
        };
        pet.tick(&[alien], 0, &mut |_| 1);
        assert_eq!(pet.birth_timer, 929);
        assert_eq!(pet.swim_counter, 0);
    }

    #[test]
    fn itchy_nearest_ties_keep_registration_order_and_steering_can_overshoot_limit() {
        let mut pet = actor(FishPetKind::Itchy);
        pet.x = 100.0;
        pet.y = 100.0;
        pet.widget_x = 100;
        pet.widget_y = 100;
        pet.special_timer = 5;
        let left = PetAlienView {
            id: 2,
            widget_x: 20,
            widget_y: 60,
            healing: false,
        };
        let right = PetAlienView {
            id: 3,
            widget_x: 100,
            widget_y: 60,
            healing: false,
        };
        pet.tick(&[left, right], 0, &mut |_| 1);
        assert_eq!(pet.vx, -2.5);
        pet.vx = 9.5;
        pet.previous_vx = 9.5;
        pet.special_timer = 5;
        pet.tick(&[right], 0, &mut |_| 1);
        assert_eq!(pet.vx, 12.0);
        assert!(pet.validate().is_ok());
    }

    #[test]
    fn pursuit_keeps_twenty_one_tick_state_rng_cadence_and_combat_pose() {
        let mut pet = actor(FishPetKind::Itchy);
        pet.movement_timer = 20;
        let alien = PetAlienView {
            id: 2,
            widget_x: 400,
            widget_y: 300,
            healing: false,
        };
        let mut ranges = Vec::new();
        pet.tick(&[alien], 0, &mut |range| {
            ranges.push(range);
            0
        });
        assert_eq!(ranges, [10, 9]);
        assert_eq!(pet.movement_state, 1);
        assert_eq!(pet.sprite_row(true), 2);
        assert_eq!(pet.sprite_row(false), 0);
    }

    #[test]
    fn prego_birth_sheet_uses_two_short_row_three_sequences() {
        let mut pet = actor(FishPetKind::Prego);
        pet.birth_timer = 731;
        assert_eq!((pet.sprite_row(false), pet.sprite_frame()), (3, 0));
        pet.birth_timer = 740;
        assert_eq!(pet.sprite_row(false), 1);
        pet.birth_timer = 926;
        assert_eq!((pet.sprite_row(false), pet.sprite_frame()), (3, 6));
        pet.turn_ticks = 19;
        assert_eq!(pet.sprite_row(false), 2);
    }

    #[test]
    fn descending_wander_exits_above_240_even_before_velocity_timer() {
        for state in [3, 4] {
            let mut pet = actor(FishPetKind::Prego);
            pet.y = 241.0;
            pet.widget_y = 241;
            pet.movement_state = state;
            pet.special_timer = 1;
            pet.tick(&[], 0, &mut |_| 1);
            assert_eq!(pet.movement_state, 0);
            assert_eq!(pet.special_timer, 2);
            assert_eq!(pet.vx, 0.0);
        }
    }

    #[test]
    fn itchy_pursuit_preserves_an_existing_high_swim_speed_counter() {
        let mut pet = actor(FishPetKind::Itchy);
        pet.vx_abs = 7;
        pet.special_timer = 5;
        let alien = PetAlienView {
            id: 2,
            widget_x: 400,
            widget_y: 300,
            healing: false,
        };
        pet.tick(&[alien], 0, &mut |_| 1);
        assert_eq!(pet.vx_abs, 7);
    }

    #[test]
    fn zorf_keeps_common_draws_and_uses_its_own_speed_and_ceiling() {
        let mut ranges = Vec::new();
        let mut pet = FishPetState::spawn_tank1(1, FishPetKind::Zorf, &mut |range| {
            ranges.push(range);
            range - 1
        });
        assert_eq!(ranges, [265, 520, 2, 3, 200, 3, 10, 200]);
        assert_eq!((pet.widget_x, pet.widget_y), (369, 539));
        assert_eq!(pet.speed_mod, 3.0);
        pet.tick_zorf(&[], &[], &mut |_| 1);
        assert!(pet.y < 271.0);
        assert!(pet.validate().is_ok());
    }

    #[test]
    fn zorf_waits_for_strict_hunger_then_emits_from_old_widget_without_cap_input() {
        let mut pet = actor(FishPetKind::Zorf);
        pet.food_timer = 64;
        let not_hungry = ZorfHungryView {
            id: 2,
            hunger: 300,
            ordinary_diet: true,
        };
        assert!(
            pet.tick_zorf(&[], &[not_hungry], &mut |_| 1)
                .free_food
                .is_none()
        );
        assert_eq!(pet.food_timer, 65);
        let wrong_diet = ZorfHungryView {
            id: 3,
            hunger: 299,
            ordinary_diet: false,
        };
        assert!(
            pet.tick_zorf(&[], &[wrong_diet], &mut |_| 1)
                .free_food
                .is_none()
        );
        assert_eq!(pet.food_timer, 66);
        let old_widget = (pet.widget_x, pet.widget_y);
        pet.vx = -0.1;
        // A stable leftward heading avoids the source turn pose, so this
        // assertion isolates the newly emitted food pose.
        pet.previous_vx = -0.1;
        let hungry = ZorfHungryView {
            id: 4,
            hunger: 299,
            ordinary_diet: true,
        };
        let result = pet.tick_zorf(&[], &[hungry], &mut |_| 1);
        assert_eq!(
            result.free_food,
            Some(ZorfFoodRequest {
                x: old_widget.0 + 15,
                y: old_widget.1 + 10,
                direction: 1,
            })
        );
        assert_eq!(pet.food_timer, -10);
        assert_eq!((pet.sprite_row(false), pet.sprite_frame()), (2, 0));
    }

    #[test]
    fn zorf_invasion_freezes_negative_timer_and_turn_overrides_drop_pose() {
        let mut pet = actor(FishPetKind::Zorf);
        pet.food_timer = -5;
        pet.turn_ticks = 5;
        assert_eq!(pet.sprite_row(false), 1);
        let alien = PetAlienView {
            id: 2,
            widget_x: 100,
            widget_y: 100,
            healing: false,
        };
        pet.tick_zorf(&[alien], &[], &mut |_| 1);
        assert_eq!(pet.food_timer, -5);
        pet.turn_ticks = 0;
        pet.food_timer = -2;
        pet.tick_zorf(&[], &[], &mut |_| 1);
        assert_eq!(pet.food_timer, -1);
        assert_eq!(pet.swim_counter, 10);
        assert_eq!((pet.sprite_row(false), pet.sprite_frame()), (2, 9));
    }

    #[test]
    fn vert_emits_one_gold_at_prior_widget_when_interval_reaches_216() {
        let mut pet = actor(FishPetKind::Vert);
        pet.coin_timer = 214;
        let old_widget = (pet.widget_x, pet.widget_y);
        assert_eq!(pet.tick(&[], 0, &mut |_| 1).gold_at, None);
        assert_eq!(pet.coin_timer, 215);
        let next_origin = (pet.widget_x, pet.widget_y);
        let result = pet.tick(&[], 0, &mut |_| 1);
        assert_eq!(
            result.gold_at,
            Some((next_origin.0 + 15, next_origin.1 + 10))
        );
        assert_eq!(pet.coin_timer, 0);
        assert_ne!(next_origin, old_widget);
        assert_eq!(pet.sprite_row(false), u8::from(pet.turn_ticks != 0));
    }

    #[test]
    fn vert_fast_swim_counter_resets_after_overshoot() {
        let mut pet = actor(FishPetKind::Vert);
        pet.swim_counter = 8;
        pet.vx_abs = 2;
        pet.vx = 1.0;
        pet.previous_vx = 1.0;
        pet.animate();
        assert_eq!((pet.swim_counter, pet.frame), (10, 9));
        pet.swim_counter = 9;
        pet.vx_abs = 1;
        pet.animate();
        assert_eq!((pet.swim_counter, pet.frame), (10, 9));
        pet.swim_counter = 18;
        pet.vx_abs = 2;
        pet.vx = 1.0;
        pet.previous_vx = 1.0;
        pet.animate();
        assert_eq!((pet.swim_counter, pet.frame), (0, 0));
    }

    #[test]
    fn vert_coin_interval_freezes_for_registered_alien_including_pending_death() {
        let mut pet = actor(FishPetKind::Vert);
        pet.coin_timer = 215;
        let alien = PetAlienView {
            id: 7,
            widget_x: 500,
            widget_y: 300,
            healing: false,
        };
        for _ in 0..3 {
            assert_eq!(pet.tick(&[alien], 0, &mut |_| 1).gold_at, None);
            assert_eq!(pet.coin_timer, 215);
        }
        assert!(pet.tick(&[], 0, &mut |_| 1).gold_at.is_some());
        assert_eq!(pet.coin_timer, 0);
    }
}
