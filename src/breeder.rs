//! Ordinary Tank 4 breeders. The installed payload establishes the species and
//! starter; lifecycle details here follow pinned WinFish `Breeder.cpp` and
//! `DeadFish.cpp` at f919b3c. Board owns IDs, food removal and newborn guppies.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreederSize {
    Small,
    Medium,
    Large,
}

#[derive(Clone, Copy, Debug)]
pub struct BreederFoodView {
    pub id: u64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub quality: u8,
    pub eligible: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BreederMeal {
    pub food_id: u64,
    pub hunger: i32,
    pub size: BreederSize,
    pub grew: bool,
}

#[derive(Clone, Debug, Default)]
pub struct BreederUpdate {
    pub meals: Vec<BreederMeal>,
    pub born_at: Option<(i32, i32)>,
    pub died: bool,
    /// Captured at Die, before the ordinary update's steering and birth tail.
    pub death_pose: Option<DeadBreeder>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BreederState {
    pub id: u64,
    pub alive: bool,
    pub x: f64,
    pub y: f64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub vx: f64,
    pub vy: f64,
    pub previous_vx: f64,
    pub speed_mod: f64,
    pub size: BreederSize,
    pub hunger: i32,
    pub food_points: u8,
    pub food_needed_to_grow: u8,
    pub birth_clock: u16,
    pub birth_threshold: u16,
    pub bought_timer: u8,
    pub cannot_be_eaten_ticks: u8,
    pub movement_state: u8,
    pub movement_timer: u8,
    /// Source `int` special counter; Gumbo pursuit can leave it unreset.
    pub steering_timer: i32,
    pub x_direction: i8,
    pub speed_band: u8,
    pub swim_counter: u8,
    pub turn_ticks: i8,
    pub eating_ticks: u8,
    pub growth_ticks: u8,
    pub frame: u8,
    pub hunger_visible: bool,
    pub hunger_animation_ticks: u8,
    pub hunger_fading_in: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeadBreeder {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub size: BreederSize,
    pub frame: u8,
    pub opacity: f32,
    pub facing_right: bool,
    pub remaining_ticks: u16,
    vx: f64,
    vy: f64,
    speed_mod: f64,
}

impl BreederState {
    /// Called after Board consumes the starter's x/y placement draws.
    pub fn spawn_starter(id: u64, x: i32, y: i32, rand_range: &mut impl FnMut(u64) -> u64) -> Self {
        let vx = if rand_range(2) == 0 { -0.1 } else { 0.0 };
        let speed_mod = match rand_range(3) {
            0 => 2.0,
            1 => 1.8,
            _ => 1.6,
        };
        let hunger = rand_range(200) as i32 + 400;
        let food_needed_to_grow = rand_range(2) as u8 + 4;
        let movement_state = rand_range(10) as u8;
        let birth_threshold = rand_range(400) as u16 + 1000;
        Self {
            id,
            alive: true,
            x: f64::from(x),
            y: f64::from(y),
            widget_x: x,
            widget_y: y,
            vx,
            vy: -0.5,
            previous_vx: if vx < 0.0 { -1.0 } else { 1.0 },
            speed_mod,
            size: BreederSize::Small,
            hunger,
            food_points: 2,
            food_needed_to_grow,
            birth_clock: 0,
            birth_threshold,
            bought_timer: 0,
            cannot_be_eaten_ticks: 0,
            movement_state,
            movement_timer: 0,
            steering_timer: 40,
            x_direction: 1,
            speed_band: 0,
            swim_counter: 0,
            turn_ticks: 0,
            eating_ticks: 0,
            growth_ticks: 0,
            frame: 0,
            hunger_visible: false,
            hunger_animation_ticks: 0,
            hunger_fading_in: false,
        }
    }

    /// Bought entrance consumes its own position, constructor and entrance
    /// draws in Board.cpp:5264-5273 order, retaining the discarded target Y.
    pub fn spawn_bought(id: u64, rand_range: &mut impl FnMut(u64) -> u64) -> Self {
        let x = rand_range(520) as i32 + 20;
        let y = rand_range(265) as i32 + 105;
        let mut actor = Self::spawn_starter(id, x, y, rand_range);
        actor.food_points = 0;
        actor.vy = rand_range(5) as f64 + 18.0;
        actor.y = 30.0;
        actor.widget_y = 30;
        actor.bought_timer = rand_range(10) as u8 + 45;
        actor
    }

    pub fn sprite_row(&self) -> u8 {
        self.row_for(self.hunger_visible)
    }

    fn row_for(&self, hungry: bool) -> u8 {
        let base = match self.size {
            BreederSize::Small => 0,
            BreederSize::Medium => 3,
            BreederSize::Large => 6,
        };
        base + if self.turn_ticks != 0 {
            1
        } else if self.eating_ticks != 0 {
            if hungry {
                return match self.size {
                    BreederSize::Small => 9,
                    BreederSize::Medium => 10,
                    BreederSize::Large => 11,
                };
            }
            2
        } else {
            0
        }
    }

    pub fn hungry_sprite_row(&self) -> u8 {
        self.row_for(true)
    }

    pub fn sprite_frame(&self) -> u8 {
        self.frame
    }

    pub fn facing_right(&self) -> bool {
        if self.turn_ticks > 0 {
            false
        } else if self.turn_ticks < 0 {
            true
        } else {
            self.vx >= 0.0 || self.previous_vx >= 0.0
        }
    }

    pub fn growth_scale(&self) -> f32 {
        match self.growth_ticks {
            0 => 1.0,
            4..=10 => ((10 - self.growth_ticks) as f32 * 0.5) / 7.0 + 0.7,
            ticks => (f32::from(ticks) * 0.2) / 3.0 + 1.0,
        }
    }

    pub fn hunger_overlay_alpha(&self) -> u8 {
        self.hunger_animation_ticks.saturating_mul(51)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0
            || !self.alive
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
            || !(0.0..=570.0).contains(&self.x)
            || !(0.0..=395.0).contains(&self.y)
            || self.widget_x != self.x as i32
            || self.widget_y != self.y as i32
            || ![1.6, 1.8, 2.0].contains(&self.speed_mod)
            || !(-1000..=1400).contains(&self.hunger)
            || self.food_needed_to_grow < 4
            || self.food_needed_to_grow > 9
            || self.birth_threshold < 500
            || self.birth_threshold > 1399
            || match self.size {
                BreederSize::Small => {
                    self.birth_clock != 0 || !(1000..=1399).contains(&self.birth_threshold)
                }
                BreederSize::Medium => {
                    self.birth_clock >= self.birth_threshold
                        || !(1000..=1399).contains(&self.birth_threshold)
                }
                BreederSize::Large => {
                    self.birth_clock > 1398 || !(500..=699).contains(&self.birth_threshold)
                }
            }
            || self.movement_state > 9
            || self.movement_timer > 20
            || ![-1, 1].contains(&self.x_direction)
            || self.speed_band > 5
            || self.swim_counter >= 20
            || !(-20..=20).contains(&self.turn_ticks)
            || self.eating_ticks > 20
            || self.growth_ticks > 10
            || self.frame >= 10
            || self.hunger_animation_ticks > 5
        {
            return Err("invalid ordinary Breeder state".into());
        }
        Ok(())
    }

    fn eat(
        &mut self,
        food: BreederFoodView,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> BreederMeal {
        let (nutrition, cap, points) = match food.quality {
            0 => (500, 800, 1),
            1 => (700, 1000, 2),
            _ => (1100, 1400, 3),
        };
        self.hunger = (self.hunger + nutrition).min(cap);
        self.food_points = self.food_points.saturating_add(points);
        let mut grew = false;
        if self.size != BreederSize::Large && self.food_points >= self.food_needed_to_grow {
            self.size = match self.size {
                BreederSize::Small => {
                    self.birth_clock = 900;
                    self.food_needed_to_grow = rand_range(5) as u8 + 5;
                    BreederSize::Medium
                }
                BreederSize::Medium => {
                    self.birth_threshold = rand_range(200) as u16 + 500;
                    BreederSize::Large
                }
                BreederSize::Large => unreachable!(),
            };
            self.food_points = 0;
            self.growth_ticks = 10;
            grew = true;
        }
        self.update_hunger_visibility();
        if self.eating_ticks == 0 {
            self.eating_ticks = 8;
        }
        BreederMeal {
            food_id: food.id,
            hunger: self.hunger,
            size: self.size,
            grew,
        }
    }

    /// Stops immediately before source DropCoin's birth call. Board commits
    /// the newborn guppy's constructor/ID before this actor's remaining tail.
    pub fn begin_tick(
        &mut self,
        food: &[BreederFoodView],
        alien_registered: bool,
        seek_during_invasion: bool,
        special_position: Option<(i32, i32)>,
        gumbo_widget: Option<(i32, i32)>,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> BreederUpdate {
        let mut result = BreederUpdate::default();
        self.cannot_be_eaten_ticks = self.cannot_be_eaten_ticks.saturating_sub(1);
        let special_active = special_position.is_some() && self.size != BreederSize::Large;
        let mut pursuing_food = false;
        if !special_active {
            if !alien_registered {
                self.hunger = self.hunger.saturating_sub(1);
            }
            self.update_hunger_visibility();
            if self.hunger < 1 {
                self.alive = false;
                result.died = true;
                result.death_pose = Some(DeadBreeder::from_starvation(self));
            } else if self.hunger < 500 && (!alien_registered || seek_during_invasion) {
                pursuing_food = self.seek_and_eat(food, rand_range, &mut result);
            }
        }

        if special_active {
            let (x, y) = special_position.expect("active specialty implies position");
            self.vx = if self.vx < 0.0 { -0.5 } else { 0.5 };
            if self.bought_timer == 0 {
                self.vy = 0.0;
            }
            let dx = self.x - f64::from(x);
            let dy = self.y - f64::from(y);
            self.x -= if dx > 50.0 {
                5.0
            } else if dx > 5.0 {
                3.0
            } else {
                0.0
            };
            self.x += if dx < -50.0 {
                5.0
            } else if dx < -5.0 {
                3.0
            } else {
                0.0
            };
            self.y -= if dy > 50.0 {
                4.0
            } else if dy > 5.0 {
                3.0
            } else {
                0.0
            };
            self.y += if dy < -50.0 {
                4.0
            } else if dy < -5.0 {
                3.0
            } else {
                0.0
            };
        } else if !pursuing_food {
            if alien_registered {
                if let Some((gx, gy)) = gumbo_widget {
                    self.steer_to_gumbo(gx, gy);
                } else {
                    self.wander();
                }
            } else {
                self.wander();
            }
        }
        self.steering_timer = self.steering_timer.wrapping_add(1);
        self.movement_timer += 1;
        if self.movement_timer > 20 {
            self.movement_timer = 0;
            if rand_range(10) == 0 {
                self.movement_state = rand_range(9) as u8 + 1;
            }
        }
        if !alien_registered && self.size != BreederSize::Small {
            self.birth_clock += 1;
            if self.birth_clock >= self.birth_threshold {
                self.birth_clock = 0;
                result.born_at = Some((self.widget_x + 5, self.widget_y + 10));
            }
        }
        result
    }

    fn update_hunger_visibility(&mut self) {
        let now_visible = self.hunger < 301;
        if now_visible != self.hunger_visible {
            self.hunger_visible = now_visible;
            self.hunger_fading_in = now_visible;
            self.hunger_animation_ticks = if now_visible { 1 } else { 5 };
        }
    }

    fn seek_and_eat(
        &mut self,
        food: &[BreederFoodView],
        rand_range: &mut impl FnMut(u64) -> u64,
        result: &mut BreederUpdate,
    ) -> bool {
        let cx = (self.x + 40.0) as i32;
        let cy = (self.y + 40.0) as i32;
        let target = food.iter().filter(|item| item.eligible).min_by_key(|item| {
            let dx = cx - item.widget_x - 20;
            let dy = cy - item.widget_y - 20;
            dx * dx + dy * dy
        });
        if let Some(target) = target {
            if self.steering_timer > 2 {
                self.steering_timer = 0;
                let urgent = self.hunger < 301;
                self.steer_axis(target.widget_x, target.widget_y, urgent);
                self.speed_band = self.speed_band.saturating_add(1).min(5);
            }
            let contact_y = (self.y + 35.0) as i32;
            for item in food.iter().filter(|item| item.eligible) {
                if cx > item.widget_x
                    && cx < item.widget_x + 40
                    && contact_y > item.widget_y
                    && contact_y < item.widget_y + 35
                {
                    result.meals.push(self.eat(*item, rand_range));
                }
            }
            true
        } else {
            false
        }
    }

    fn steer_axis(&mut self, fx: i32, fy: i32, urgent: bool) {
        let cx = (self.x + 40.0) as i32;
        let cy = (self.y + 45.0) as i32;
        let fx = f64::from(fx);
        let fy = f64::from(fy);
        let horizontal = if urgent {
            (4.0, 1.3, 0.2)
        } else {
            (3.0, 1.0, 0.1)
        };
        let cx = f64::from(cx);
        if cx > fx + 28.0 && self.vx > -horizontal.0 {
            self.vx -= horizontal.1;
        } else if cx < fx + 12.0 && self.vx < horizontal.0 {
            self.vx += horizontal.1;
        } else if cx > fx + 24.0 && self.vx > -horizontal.0 {
            self.vx -= horizontal.2;
        } else if cx < fx + 16.0 && self.vx < horizontal.0 {
            self.vx += horizontal.2;
        } else if cx > fx + 20.0 && self.vx > -horizontal.0 {
            self.vx -= 0.05;
        } else if cx < fx + 20.0 && self.vx < horizontal.0 {
            self.vx += 0.05;
        }
        let cy = f64::from(cy);
        let (up_guard, down_guard, up, down, fine_up, fine_down) = if urgent {
            (-3.0, 4.0, 1.0, 1.3, 0.5, 0.7)
        } else {
            (-2.0, 3.0, 0.6, 1.0, 0.3, 0.5)
        };
        if cy > fy + 26.0 && self.vy > up_guard {
            self.vy -= up;
        } else if cy < fy + 14.0 && self.vy < down_guard {
            self.vy += down;
        } else if cy > fy + 20.0 && self.vy > up_guard {
            self.vy -= fine_up;
        } else if cy < fy + 20.0 && self.vy < down_guard {
            self.vy += fine_down;
        }
    }

    fn steer_to_gumbo(&mut self, gx: i32, gy: i32) {
        let cx = self.x + 40.0;
        let gx = f64::from(gx);
        if cx > gx + 50.0 && self.vx > -4.0 {
            self.vx -= 1.3;
        } else if cx < gx + 30.0 && self.vx < 4.0 {
            self.vx += 1.3;
        } else if cx > gx + 45.0 && self.vx > -4.0 {
            self.vx -= 0.2;
        } else if cx < gx + 35.0 && self.vx < 4.0 {
            self.vx += 0.2;
        } else if cx > gx + 40.0 && self.vx > -4.0 {
            self.vx -= 0.05;
        } else if cx < gx + 40.0 && self.vx < 4.0 {
            self.vx += 0.05;
        }
        let cy = self.y + 40.0;
        let gy = f64::from(gy);
        if cy > gy + 25.0 && self.vy > -3.0 {
            self.vy -= 1.0;
        } else if cy < gy + 15.0 && self.vy < 4.0 {
            self.vy += 1.3;
        }
        if cy > gy + 20.0 && self.vy > -3.0 {
            self.vy -= 0.5;
        } else if cy < gy + 20.0 && self.vy < 4.0 {
            self.vy += 0.7;
        }
        if self.widget_y <= 95 && self.vy < 0.0 {
            self.vy = 0.0;
        }
        self.speed_band = self.speed_band.saturating_add(1).min(5);
    }

    fn wander(&mut self) {
        if self.steering_timer <= 39 {
            return;
        }
        self.steering_timer = 0;
        let target_x = match self.movement_state {
            0 => 0.0,
            1 | 4 => 1.0,
            2 | 3 => -1.0,
            _ => {
                if self.x_direction > 0 {
                    2.0
                } else {
                    -2.0
                }
            }
        };
        if self.vx < target_x {
            self.vx += if self.movement_state > 4 { 1.0 } else { 0.5 };
        } else if self.vx > target_x {
            self.vx -= if self.movement_state > 4 { 1.0 } else { 0.5 };
        }
        if self.movement_state > 4 {
            if self.x > 250.0 {
                self.x_direction = -1;
            } else if self.x < 175.0 {
                self.x_direction = 1;
            }
        }
        self.speed_band = self.vx.abs() as u8;
        if self.bought_timer == 0 {
            self.vy = match self.movement_state {
                0 => 0.5,
                3 | 4 => 3.0,
                _ => -0.5,
            };
        }
    }

    /// Completes the source tail after Board has instantiated a due guppy.
    pub fn finish_tick(&mut self, rand_range: &mut impl FnMut(u64) -> u64) -> Vec<(i32, i32)> {
        let mut bubbles = Vec::new();
        if self.hunger_animation_ticks != 0 {
            if self.hunger_fading_in {
                self.hunger_animation_ticks += 1;
                if self.hunger_animation_ticks > 5 {
                    self.hunger_animation_ticks = 0;
                }
            } else {
                self.hunger_animation_ticks -= 1;
            }
        }
        if self.bought_timer != 0 {
            self.bought_timer -= 1;
            self.vy *= 0.9;
        }
        if self.vx == 0.0 {
            self.y += 1.0 / self.speed_mod;
        } else if (1.0..=3.0).contains(&self.vx) && self.vx.fract() == 0.0 {
            self.y += (4.0 - self.vx) * 0.25 / self.speed_mod;
        }
        self.x = self.x.clamp(10.0, 540.0);
        self.y = self.y.min(370.0);
        if self.bought_timer == 0 || self.vy <= 0.0 {
            self.y = self.y.max(95.0);
        } else if self.bought_timer > 30 {
            let chance = if self.bought_timer > 40 { 1 } else { 2 };
            if rand_range(chance) == 0 {
                bubbles.push((
                    self.widget_x + 40 - rand_range(30) as i32,
                    self.widget_y + 40 - rand_range(30) as i32,
                ));
            }
        }
        if self.x > 535.0 && self.vx > 0.1 {
            self.vx -= 0.1;
        }
        if self.x < 15.0 && self.vx < -0.1 {
            self.vx += 0.1;
        }
        if self.previous_vx < 0.0 && self.vx > 0.0 {
            self.turn_ticks = -20;
        } else if self.previous_vx > 0.0 && self.vx < 0.0 {
            self.turn_ticks = 20;
        }
        self.turn_ticks -= self.turn_ticks.signum();
        self.eating_ticks = self.eating_ticks.saturating_sub(1);
        if self.turn_ticks != 0 {
            self.frame = (9_i16 - i16::from(self.turn_ticks.abs() / 2)) as u8;
        } else if self.eating_ticks != 0 {
            self.frame = 9 - self.eating_ticks / 2;
        } else {
            self.swim_counter += if self.speed_band < 2 { 1 } else { 2 };
            if self.swim_counter >= 20 {
                self.swim_counter = 0;
            }
            self.frame = self.swim_counter / 2;
        }
        if self.vx != 0.0 && self.previous_vx != 0.0 {
            self.previous_vx = self.vx;
        }
        self.growth_ticks = self.growth_ticks.saturating_sub(1);
        self.x += self.vx / self.speed_mod;
        self.y += self.vy / self.speed_mod;
        self.widget_x = self.x as i32;
        self.widget_y = self.y as i32;
        bubbles
    }
}

impl DeadBreeder {
    pub fn from_starvation(actor: &BreederState) -> Self {
        assert!(!actor.alive, "starvation snapshot precedes movement tail");
        Self::from_pose(actor)
    }

    pub fn from_impact(actor: &BreederState) -> Self {
        Self::from_pose(actor)
    }

    fn from_pose(actor: &BreederState) -> Self {
        Self {
            id: actor.id,
            x: actor.x,
            y: actor.y,
            widget_x: actor.widget_x,
            widget_y: actor.widget_y,
            size: actor.size,
            frame: 0,
            opacity: 1.0,
            facing_right: actor.vx >= 0.0,
            remaining_ticks: 125,
            vx: actor.vx,
            vy: actor.vy
                - if actor.widget_x < 115 || actor.vy < -3.0 {
                    1.0
                } else {
                    2.0
                },
            speed_mod: actor.speed_mod,
        }
    }

    pub fn sprite_row(&self) -> u8 {
        match self.size {
            BreederSize::Small => 2,
            BreederSize::Medium => 5,
            BreederSize::Large => 8,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0
            || ![self.x, self.y, self.vx, self.vy, self.speed_mod]
                .into_iter()
                .all(f64::is_finite)
            || self.widget_x != self.x as i32
            || self.widget_y != self.y as i32
            || self.remaining_ticks > 125
            || self.frame >= 10
            || !self.opacity.is_finite()
            || !(0.0..=1.0).contains(&self.opacity)
            || ![1.6, 1.8, 2.0].contains(&self.speed_mod)
        {
            return Err("invalid Breeder corpse state".into());
        }
        Ok(())
    }

    pub fn tick(&mut self) -> bool {
        let remaining = self.remaining_ticks;
        self.frame = if remaining >= 106 {
            9 - ((remaining - 106) / 2) as u8
        } else if remaining == 104 || remaining == 103 {
            8
        } else if remaining == 102 || remaining == 101 {
            7
        } else {
            6
        };
        if remaining < 105 {
            self.opacity = (self.opacity - 0.02).max(0.0);
        }
        if remaining == 0 {
            return true;
        }
        if remaining > 105 || self.y > 365.0 {
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
        self.y = (self.y + self.vy / self.speed_mod).clamp(85.0, 370.0);
        self.widget_x = self.x as i32;
        self.widget_y = self.y as i32;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn medium_birth_clock_and_large_growth_can_birth_on_same_update() {
        let mut actor = BreederState::spawn_starter(1, 200, 200, &mut |_| 0);
        actor.food_points = 3;
        actor.hunger = 450;
        let food = [BreederFoodView {
            id: 2,
            widget_x: 220,
            widget_y: 205,
            quality: 0,
            eligible: true,
        }];
        let first = actor.begin_tick(&food, false, false, None, None, &mut |_| 0);
        assert_eq!(first.meals[0].size, BreederSize::Medium);
        assert_eq!(actor.birth_clock, 901);
        actor.food_points = actor.food_needed_to_grow - 1;
        actor.birth_clock = 901;
        actor.hunger = 450;
        let second = actor.begin_tick(&food, false, false, None, None, &mut |_| 0);
        assert_eq!(second.meals[0].size, BreederSize::Large);
        assert_eq!(second.born_at, Some((205, 210)));
    }

    #[test]
    fn starvation_snapshots_before_due_birth_and_motion_tail() {
        let mut actor = BreederState::spawn_starter(1, 200, 200, &mut |_| 0);
        actor.size = BreederSize::Medium;
        actor.hunger = 1;
        actor.birth_clock = actor.birth_threshold - 1;
        let result = actor.begin_tick(&[], false, false, None, None, &mut |_| 0);
        let corpse = result
            .death_pose
            .as_ref()
            .expect("Die snapshots before steering");
        assert!(result.died);
        assert_eq!(result.born_at, Some((205, 210)));
        actor.finish_tick(&mut |_| 0);
        assert_eq!((corpse.widget_x, corpse.widget_y), (200, 200));
        assert_ne!(actor.y, corpse.y);
    }
}
