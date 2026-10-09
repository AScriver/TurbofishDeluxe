//! Ordinary Adventure 3-2 beetle-eater. Installed PB40/PB43 establish the
//! typed diet, purchase rolls, meal, and Pearl output. Local movement and
//! corpse animation follow pinned W1 `Gekko.cpp`, `Fish.cpp`, `DeadFish.cpp`
//! (f919b3c); the actor has not been runtime verified.
//! The Board owns IDs, shared RNG, prey removal, Pearl construction and death.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GekkoPreyKind {
    Larva,
    Peanut,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GekkoPrey {
    pub id: u64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub kind: GekkoPreyKind,
    /// Larva: not picked; Peanut: raw type 18 and not claimed. Larva mouse
    /// visibility is deliberately irrelevant to this diet.
    pub eligible: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GekkoUpdate {
    pub eaten_prey: Option<(u64, GekkoPreyKind)>,
    /// Previous integer widget origin. Board creates the ordinary 500 Pearl.
    pub pearl_at: Option<(i32, i32)>,
    pub died: bool,
    pub bubbles: Vec<(i32, i32)>,
}

#[derive(Clone, Copy, Debug)]
struct DeathPose {
    x: i32,
    y: i32,
    vx: f64,
    vy: f64,
    speed_mod: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GekkoState {
    pub id: u64,
    pub alive: bool,
    pub x: f64,
    pub y: f64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub vx: f64,
    pub vy: f64,
    pub hunger: i32,
    pub frame: u8,
    pub turn_ticks: i8,
    pub eating_ticks: u8,
    pub coin_timer: u16,
    pub coin_threshold: u16,
    pub bought_timer: u8,
    pub speed_mod: f64,
    pub movement_state: u8,
    pub movement_timer: u8,
    pub special_timer: u8,
    pub x_direction: i8,
    pub vx_abs: u8,
    pub swim_counter: u8,
    pub speedy_ticks: u8,
    pub hunger_shown: bool,
    pub hunger_animation_ticks: u8,
    pub scream_ticks: u16,
    previous_vx: f64,
    #[serde(skip)]
    death_pose: Option<DeathPose>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeadGekko {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub frame: u8,
    pub opacity: f32,
    pub facing_right: bool,
    pub remaining_ticks: u16,
    /// Angie changes the fresh corpse's 100 to a ten-update revival.
    pub revival_ticks: i32,
    vx: f64,
    vy: f64,
    speed_mod: f64,
}

impl GekkoState {
    /// Position rolls precede the inherited Fish rolls; the latter's ordinary
    /// coin threshold is consumed before Gekko replaces it with 200..449.
    pub fn spawn_bought(id: u64, rand_range: &mut impl FnMut(u64) -> u64) -> Self {
        let x = rand_range(520) as i32 + 20;
        let y = rand_range(265) as i32 + 105;
        let mut actor = Self::spawn_at(id, x, y, None, rand_range);
        actor.vy = rand_range(5) as f64 + 23.0;
        actor.y = 40.0;
        actor.widget_y = 40;
        actor.bought_timer = rand_range(10) as u8 + 45;
        actor
    }

    /// Fresh Gekko construction at the corpse pose; the old actor's hunger,
    /// clock and identity are intentionally absent.
    pub fn spawn_revived(
        id: u64,
        x: i32,
        y: i32,
        facing_right: bool,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> Self {
        Self::spawn_at(id, x, y, Some(facing_right), rand_range)
    }

    fn spawn_at(
        id: u64,
        x: i32,
        y: i32,
        facing_right: Option<bool>,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> Self {
        let left = rand_range(2) != 0;
        let speed_mod = match rand_range(3) {
            0 => 2.0,
            1 => 1.8,
            _ => 1.6,
        };
        let hunger = rand_range(200) as i32 + 400;
        let _unused_growth_requirement = rand_range(3);
        let movement_state = rand_range(10) as u8;
        let _inherited_coin_threshold = rand_range(200) + 150;
        let coin_threshold = rand_range(250) as u16 + 200;
        let vx = facing_right.map_or(if left { -0.1 } else { 0.0 }, |right| {
            if right { 1.0 } else { -1.0 }
        });
        Self {
            id,
            alive: true,
            x: f64::from(x),
            y: f64::from(y),
            widget_x: x,
            widget_y: y,
            vx,
            vy: -0.5,
            hunger,
            frame: 0,
            turn_ticks: 0,
            eating_ticks: 0,
            coin_timer: 0,
            coin_threshold,
            bought_timer: 0,
            speed_mod,
            movement_state,
            movement_timer: 0,
            special_timer: 40,
            x_direction: 1,
            vx_abs: 0,
            swim_counter: 0,
            speedy_ticks: 0,
            hunger_shown: false,
            hunger_animation_ticks: 0,
            scream_ticks: 0,
            previous_vx: if vx < 0.0 { -1.0 } else { 1.0 },
            death_pose: None,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0
            || !self.alive
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
            || self.widget_x != self.x as i32
            || self.widget_y != self.y as i32
            || ![1.6, 1.8, 2.0].contains(&self.speed_mod)
            || !(1..=1000).contains(&self.hunger)
            || !(200..=449).contains(&self.coin_threshold)
            || self.coin_timer >= self.coin_threshold
            || self.bought_timer > 54
            || self.movement_state > 9
            || self.movement_timer > 20
            || self.special_timer > 50
            || !matches!(self.x_direction, -1 | 1)
            || self.swim_counter >= 20
            || self.frame > 9
            || !(-19..=19).contains(&self.turn_ticks)
            || self.eating_ticks > 20
            || self.hunger_animation_ticks > 5
        {
            return Err("invalid ordinary Gekko state".into());
        }
        Ok(())
    }

    pub fn sprite_row(&self) -> u8 {
        self.row_for_hunger(self.hunger_visible())
    }

    pub fn hungry_sprite_row(&self) -> u8 {
        self.row_for_hunger(true)
    }

    fn row_for_hunger(&self, hungry: bool) -> u8 {
        if self.turn_ticks != 0 {
            if hungry { 4 } else { 1 }
        } else if self.eating_ticks != 0 || self.scream_ticks > 100 {
            if hungry { 6 } else { 2 }
        } else if hungry {
            3
        } else {
            0
        }
    }

    pub fn sprite_frame(&self) -> u8 {
        self.frame
    }

    pub fn facing_right(&self) -> bool {
        if self.turn_ticks != 0 {
            self.turn_ticks > 0
        } else if self.vx.abs() >= 1.0 {
            self.vx >= 0.0
        } else {
            self.previous_vx >= 0.0
        }
    }

    pub fn hunger_visible(&self) -> bool {
        self.hunger <= 300 && self.hunger_animation_ticks == 0
    }

    pub fn hunger_overlay_alpha(&self) -> u8 {
        (u16::from(self.hunger_animation_ticks) * 255 / 5) as u8
    }

    /// Consume a birth-ordered prey snapshot. The actor applies the two typed
    /// list scans itself, so mixed caller order cannot reverse Larva priority.
    /// Board commits any reported consumption before updating the next actor.
    pub fn tick(
        &mut self,
        prey: &[GekkoPrey],
        alien_present: bool,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> GekkoUpdate {
        let mut out = GekkoUpdate::default();
        self.death_pose = None;
        self.speedy_ticks = self.speedy_ticks.saturating_sub(1);
        self.scream_ticks = self.scream_ticks.saturating_sub(1);
        self.advance_hunger_animation();
        if !alien_present {
            self.hunger = (self.hunger - 1).max(-1000);
            if self.hunger == 304 {
                self.hunger_shown = true;
                self.hunger_animation_ticks = 1;
            }
        }
        let hunting = if self.hunger < 1 {
            self.alive = false;
            self.death_pose = Some(self.current_pose());
            out.died = true;
            false
        } else if self.hunger < 500 {
            self.hunt(prey, &mut out)
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
                out.pearl_at = Some((self.widget_x + 5, self.widget_y + 10));
            }
        }
        if self.bought_timer > 0 {
            self.bought_timer -= 1;
            self.vy *= 0.9;
            // Fish's entrance bubbles are nested under the positive-VY
            // branch; an upward-steered bought fish consumes no bubble rolls.
            if self.vy > 0.0 && self.bought_timer >= 31 {
                let chance = if self.bought_timer > 40 { 1 } else { 2 };
                if rand_range(chance) == 0 {
                    out.bubbles.push((
                        self.widget_x + 55 - rand_range(60) as i32,
                        self.widget_y + 55 - rand_range(60) as i32,
                    ));
                    out.bubbles.push((
                        self.widget_x + 45 - rand_range(40) as i32,
                        self.widget_y + 45 - rand_range(40) as i32,
                    ));
                }
            }
        }
        // Common Fish tail clamps before final Move, so a small overshoot is
        // legal in the serialized doubles and integer widget coordinates.
        match self.vx {
            0.0 => self.y += 1.0 / self.speed_mod,
            1.0 => self.y += 0.75 / self.speed_mod,
            2.0 => self.y += 0.5 / self.speed_mod,
            3.0 => self.y += 0.25 / self.speed_mod,
            _ => {}
        }
        self.x = self.x.clamp(10.0, 540.0);
        self.y = self.y.min(360.0);
        if self.bought_timer == 0 || self.vy <= 0.0 {
            self.y = self.y.max(105.0);
        }
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
        out
    }

    fn ordered(prey: &[GekkoPrey]) -> impl Iterator<Item = &GekkoPrey> {
        [GekkoPreyKind::Larva, GekkoPreyKind::Peanut]
            .into_iter()
            .flat_map(move |kind| {
                prey.iter()
                    .filter(move |candidate| candidate.eligible && candidate.kind == kind)
            })
    }

    fn nearest<'a>(&self, prey: &'a [GekkoPrey]) -> Option<&'a GekkoPrey> {
        let center_x = self.widget_x + 40;
        let center_y = self.widget_y + 40;
        Self::ordered(prey).min_by_key(|candidate| {
            let dx = i64::from(candidate.widget_x + 20 - center_x);
            let dy = i64::from(candidate.widget_y + 20 - center_y);
            dx * dx + dy * dy
        })
    }

    fn hunt(&mut self, prey: &[GekkoPrey], out: &mut GekkoUpdate) -> bool {
        let Some(target) = self.nearest(prey) else {
            return false;
        };
        let dx = i64::from(target.widget_x + 20 - self.widget_x - 40);
        let dy = i64::from(target.widget_y + 20 - self.widget_y - 40);
        if dx * dx + dy * dy < 10_000 {
            self.speedy_ticks = 100;
        }
        if self.special_timer > 2 {
            self.special_timer = 0;
            self.steer(target);
            self.vx_abs = self.vx_abs.saturating_add(1).min(5);
        }
        let center_x = self.x + 40.0;
        let center_y = self.y + 40.0;
        for candidate in Self::ordered(prey) {
            let left = f64::from(candidate.widget_x);
            let top = f64::from(candidate.widget_y);
            if center_x > left + 18.0
                && center_x < left + 54.0
                && center_y > top + 18.0
                && center_y < top + 54.0
            {
                let was_hungry = self.hunger_visible();
                self.speedy_ticks = 0;
                self.hunger = (self.hunger.max(300) + 700).min(1000);
                if was_hungry && !self.hunger_visible() {
                    self.hunger_shown = false;
                    self.hunger_animation_ticks = 5;
                }
                if self.eating_ticks == 0 {
                    self.eating_ticks = 8;
                }
                out.eaten_prey = Some((candidate.id, candidate.kind));
                break;
            }
            if self.eating_ticks == 0
                && center_x > left - 4.0
                && center_x < left + 76.0
                && center_y > top - 6.0
                && center_y < top + 66.0
            {
                self.eating_ticks = 20;
            }
        }
        true
    }

    fn steer(&mut self, target: &GekkoPrey) {
        let center_x = (self.x + 40.0) as i32;
        let center_y = (self.y + 45.0) as i32;
        let tx = target.widget_x;
        let ty = target.widget_y;
        let urgent = self.hunger <= 300;
        let limit = if urgent { 4.0 } else { 3.0 };
        let (far, near, fine) = if urgent {
            (1.3, 0.2, 0.05)
        } else {
            (1.0, 0.1, 0.05)
        };
        for (condition, signed_step) in [
            (tx + 40 < center_x, -far),
            (tx + 32 > center_x, far),
            (tx + 38 < center_x, -near),
            (tx + 34 > center_x, near),
            (tx + 36 < center_x, -fine),
            (tx + 36 > center_x, fine),
        ] {
            if condition {
                if (signed_step < 0.0 && self.vx > -limit) || (signed_step > 0.0 && self.vx < limit)
                {
                    self.vx += signed_step;
                }
                break;
            }
        }
        let vertical = if urgent {
            [
                (ty + 36 < center_y, -1.3),
                (ty + 36 > center_y, 1.3),
                (false, 0.0),
                (false, 0.0),
            ]
        } else {
            [
                (ty + 39 < center_y, -1.0),
                (ty + 33 > center_y, 1.0),
                (ty + 36 < center_y, -0.5),
                (ty + 36 > center_y, 0.5),
            ]
        };
        let (lower_vy, upper_vy) = if urgent { (-4.0, 4.0) } else { (-3.0, 3.0) };
        for (condition, step) in vertical {
            if condition {
                if (step < 0.0 && self.vy > lower_vy) || (step > 0.0 && self.vy < upper_vy) {
                    self.vy += step;
                }
                break;
            }
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
                    let desired = if self.movement_state == 3 { -1.0 } else { 1.0 };
                    if self.vx < desired {
                        self.vx += 1.0;
                    } else if self.vx > desired {
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

    fn advance_hunger_animation(&mut self) {
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
        if self.scream_ticks > 100 && self.turn_ticks == 0 {
            self.frame = 4;
        }
    }

    fn current_pose(&self) -> DeathPose {
        DeathPose {
            x: self.x as i32,
            y: self.y as i32,
            vx: self.vx,
            vy: self.vy,
            speed_mod: self.speed_mod,
        }
    }
}

impl DeadGekko {
    pub fn from_live(actor: &GekkoState) -> Self {
        let pose = actor
            .death_pose
            .expect("Gekko starvation corpse requires pre-tail pose");
        Self::from_pose(actor.id, pose)
    }

    pub fn from_impact(actor: &GekkoState) -> Self {
        Self::from_pose(actor.id, actor.current_pose())
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
            facing_right: pose.vx >= 0.0,
            remaining_ticks: 125,
            revival_ticks: 100,
            vx: pose.vx,
            vy,
            speed_mod: pose.speed_mod,
        }
    }

    pub fn sprite_row(&self) -> u8 {
        5
    }
    pub fn sprite_frame(&self) -> u8 {
        self.frame
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0
            || ![self.x, self.y, self.vx, self.vy, self.speed_mod]
                .into_iter()
                .all(f64::is_finite)
            || !(0.0..=570.0).contains(&self.x)
            || !(0.0..=380.0).contains(&self.y)
            || self.widget_x != self.x as i32
            || self.widget_y != self.y as i32
            || ![1.6, 1.8, 2.0].contains(&self.speed_mod)
            || self.frame >= 10
            || self.remaining_ticks > 125
            || !(self.revival_ticks == 100 || (0..=10).contains(&self.revival_ticks))
            || !self.opacity.is_finite()
            || !(0.0..=1.0).contains(&self.opacity)
        {
            return Err("invalid ordinary Gekko corpse state".into());
        }
        Ok(())
    }

    /// The ordinary body waits at counter105 until it reaches the bottom,
    /// then fades. Angie can separately trigger fresh reconstruction.
    pub fn tick(&mut self) -> bool {
        let remaining = self.remaining_ticks;
        self.frame = if remaining > 105 {
            9 - ((remaining - 106) / 2) as u8
        } else {
            match remaining {
                104 | 103 => 8,
                102 | 101 => 7,
                0..=100 => 6,
                _ => 9,
            }
        };
        if (1..=10).contains(&self.revival_ticks) {
            self.revival_ticks -= 1;
            self.frame = self.revival_ticks as u8;
        }
        if remaining < 105 {
            self.opacity = (self.opacity - 0.02).max(0.0);
        }
        if remaining == 0 {
            return true;
        }
        if self.revival_ticks == 0 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revival_constructs_fresh_gekko_with_corpse_facing() {
        let mut draws = Vec::new();
        let actor = GekkoState::spawn_revived(8, 190, 180, false, &mut |upper| {
            draws.push(upper);
            0
        });
        assert_eq!(draws, [2, 3, 200, 3, 10, 200, 250]);
        assert_eq!(
            (actor.widget_x, actor.widget_y, actor.vx, actor.vy),
            (190, 180, -1.0, -0.5)
        );
        assert_eq!(
            (actor.hunger, actor.coin_threshold, actor.bought_timer),
            (400, 200, 0)
        );
        actor.validate().unwrap();
    }

    fn actor() -> GekkoState {
        GekkoState::spawn_bought(7, &mut |_| 0)
    }
    fn prey(id: u64, kind: GekkoPreyKind, x: i32, y: i32, eligible: bool) -> GekkoPrey {
        GekkoPrey {
            id,
            widget_x: x,
            widget_y: y,
            kind,
            eligible,
        }
    }

    #[test]
    fn bought_constructor_consumes_inherited_and_overridden_rolls_in_order() {
        let mut ranges = Vec::new();
        let g = GekkoState::spawn_bought(7, &mut |upper| {
            ranges.push(upper);
            0
        });
        assert_eq!(ranges, [520, 265, 2, 3, 200, 3, 10, 200, 250, 5, 10]);
        assert_eq!(
            (g.widget_x, g.widget_y, g.vy, g.bought_timer),
            (20, 40, 23.0, 45)
        );
        assert_eq!((g.hunger, g.coin_threshold), (400, 200));
        g.validate().unwrap();
    }

    #[test]
    fn invisible_unpicked_larva_is_edible_but_claimed_one_is_not() {
        let mut g = actor();
        g.x = 100.0;
        g.y = 100.0;
        g.widget_x = 100;
        g.widget_y = 100;
        g.hunger = 200;
        g.bought_timer = 0;
        // No mouse-visibility field is read. Board sets eligible from pickup.
        let views = [
            prey(1, GekkoPreyKind::Larva, 100, 100, false),
            prey(2, GekkoPreyKind::Larva, 100, 100, true),
        ];
        assert_eq!(
            g.tick(&views, false, &mut |_| 1).eaten_prey,
            Some((2, GekkoPreyKind::Larva))
        );
        assert_eq!(g.hunger, 1000); // floor300 then +700
    }

    #[test]
    fn larva_list_wins_contact_ties_even_when_peanut_view_arrives_first() {
        let mut g = actor();
        g.x = 100.0;
        g.y = 100.0;
        g.widget_x = 100;
        g.widget_y = 100;
        g.hunger = 400;
        g.bought_timer = 0;
        let views = [
            prey(1, GekkoPreyKind::Peanut, 100, 100, true),
            prey(2, GekkoPreyKind::Larva, 100, 100, true),
        ];
        assert_eq!(g.nearest(&views).unwrap().id, 2);
        assert_eq!(
            g.tick(&views, false, &mut |_| 1).eaten_prey,
            Some((2, GekkoPreyKind::Larva))
        );
        assert_eq!(g.hunger, 1000);
    }

    #[test]
    fn strict_contact_edges_do_not_eat_and_nearer_peanut_can_win_targeting() {
        let mut g = actor();
        g.x = 100.0;
        g.y = 100.0;
        g.widget_x = 100;
        g.widget_y = 100;
        g.hunger = 400;
        g.bought_timer = 0;
        let edge = prey(1, GekkoPreyKind::Larva, 122, 100, true); // center140 = left+18
        let far_larva = prey(2, GekkoPreyKind::Larva, 180, 100, true);
        let near_peanut = prey(3, GekkoPreyKind::Peanut, 120, 120, true);
        assert_eq!(g.nearest(&[far_larva, near_peanut]).unwrap().id, 3);
        assert_eq!(g.tick(&[edge], false, &mut |_| 1).eaten_prey, None);
    }

    #[test]
    fn death_tail_still_produces_previous_widget_pearl_and_pre_tail_body() {
        let mut g = actor();
        g.x = 100.0;
        g.y = 100.0;
        g.widget_x = 100;
        g.widget_y = 100;
        g.hunger = 1;
        g.bought_timer = 0;
        g.coin_timer = g.coin_threshold - 1;
        let out = g.tick(&[], false, &mut |_| 1);
        assert!(out.died);
        assert_eq!(out.pearl_at, Some((105, 110)));
        assert_eq!(
            (
                DeadGekko::from_live(&g).widget_x,
                DeadGekko::from_live(&g).widget_y
            ),
            (100, 100)
        );
        assert!(!g.alive);
        assert_ne!(g.y, 100.0);
    }

    #[test]
    fn registered_alien_freezes_hunger_and_production_but_motion_continues() {
        let mut g = actor();
        g.coin_timer = g.coin_threshold - 1;
        let old = (g.hunger, g.y);
        let out = g.tick(&[], true, &mut |_| 1);
        assert_eq!((g.hunger, g.coin_timer), (old.0, g.coin_threshold - 1));
        assert_eq!(out.pearl_at, None);
        assert_ne!(g.y, old.1);
    }

    #[test]
    fn corpse_uses_ordinary_row_and_sinks_before_fading() {
        let g = actor();
        let mut body = DeadGekko::from_impact(&g);
        assert_eq!((body.sprite_row(), body.remaining_ticks), (5, 125));
        body.remaining_ticks = 105;
        body.y = 300.0;
        body.widget_y = 300;
        body.tick();
        assert_eq!(
            (body.remaining_ticks, body.frame, body.opacity),
            (105, 9, 1.0)
        );
        body.y = 371.0;
        body.widget_y = 371;
        body.tick();
        assert_eq!(body.remaining_ticks, 104);
    }

    #[test]
    fn vertical_steering_uses_distinct_urgent_and_ordinary_limits() {
        // W1 Gekko::HungryBehavior 219-227 versus 263-281.
        let mut urgent = actor();
        urgent.x = 100.0;
        urgent.y = 100.0;
        urgent.hunger = 300;
        urgent.vy = -3.2;
        urgent.steer(&prey(2, GekkoPreyKind::Larva, 100, 60, true));
        assert!((urgent.vy + 4.5).abs() < 1e-9);

        let mut ordinary = actor();
        ordinary.x = 100.0;
        ordinary.y = 100.0;
        ordinary.hunger = 400;
        ordinary.vy = 3.2;
        ordinary.steer(&prey(3, GekkoPreyKind::Larva, 100, 140, true));
        assert_eq!(ordinary.vy, 3.2);
    }

    #[test]
    fn bought_upward_motion_skips_entrance_bubble_draws() {
        // Fish::Update 466-481 enters this branch only for positive VY.
        let mut upward = actor();
        upward.vy = -1.0;
        let mut upward_draws = Vec::new();
        let update = upward.tick(&[], true, &mut |upper| {
            upward_draws.push(upper);
            0
        });
        assert!(update.bubbles.is_empty());
        assert!(upward_draws.is_empty());

        let mut descending = actor();
        let mut descending_draws = Vec::new();
        let update = descending.tick(&[], true, &mut |upper| {
            descending_draws.push(upper);
            0
        });
        assert_eq!(update.bubbles.len(), 2);
        assert_eq!(descending_draws, [1, 60, 60, 40, 40]);
    }
}
