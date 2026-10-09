//! Ordinary Tank 3 guppycruncher. PB37 establishes the installed-binary type,
//! diet, purchased placement and larva production. Movement, counters and
//! corpse projection follow pinned W1 `Grubber.cpp`/`DeadFish.cpp` f919b3c.
//! The board owns ordered prey, IDs, larva construction and removal commits.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GrubberPrey {
    pub id: u64,
    pub widget_x: i32,
    pub widget_y: i32,
    /// Live ordinary Small guppy, virtual ID negative, eat-delay zero.
    pub eligible: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GrubberUpdate {
    pub eaten_prey: Option<u64>,
    /// Origin from the previous integer widget pose, even on a death tick.
    pub larva_at: Option<(i32, i32)>,
    pub died: bool,
    /// Board may instantiate transient bubbles; the actor only requests them.
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
pub struct GrubberState {
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
    pub eating_ticks: u8,
    pub coin_timer: u16,
    pub coin_threshold: u16,
    pub bought_timer: u8,
    pub speed_mod: f64,
    pub movement_state: u8,
    pub movement_timer: u8,
    pub eat_delay: u16,
    pub speed_change_timer: u8,
    pub speedy_ticks: u8,
    pub hunger_shown: bool,
    pub hunger_animation_ticks: u8,
    pub scream_ticks: u16,
    target_vx: f64,
    #[serde(skip)]
    death_pose: Option<DeathPose>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeadGrubber {
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

impl GrubberState {
    /// Board's X draw, then the discarded constructor Y draw, speed, hunger,
    /// movement state and production threshold. Bought placement overwrites
    /// both double and widget Y to 65 without refunding the Y draw.
    pub fn spawn_bought(id: u64, rand_range: &mut impl FnMut(u64) -> u64) -> Self {
        let x = rand_range(520) as i32 + 20;
        let _constructor_y = rand_range(5) + 360;
        let speed_mod = match rand_range(3) {
            0 => 2.7,
            1 => 2.5,
            _ => 2.6,
        };
        let hunger = rand_range(200) as i32 + 900;
        let movement_state = rand_range(10) as u8;
        let coin_threshold = rand_range(200) as u16 + 300;
        Self {
            id,
            alive: true,
            x: f64::from(x),
            y: 65.0,
            widget_x: x,
            widget_y: 65,
            vx: 0.0,
            vy: 0.0,
            hunger,
            frame: 0,
            eating_ticks: 0,
            coin_timer: 0,
            coin_threshold,
            bought_timer: 45,
            speed_mod,
            movement_state,
            movement_timer: 0,
            eat_delay: 40,
            speed_change_timer: 0,
            speedy_ticks: 0,
            hunger_shown: false,
            hunger_animation_ticks: 0,
            scream_ticks: 0,
            target_vx: 0.0,
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
                self.target_vx,
            ]
            .into_iter()
            .all(f64::is_finite)
            || !(0.0..=570.0).contains(&self.x)
            || !(0.0..=390.0).contains(&self.y)
            || self.widget_x != self.x as i32
            || self.widget_y != self.y as i32
            || ![2.5, 2.6, 2.7].contains(&self.speed_mod)
            || !(1..=1400).contains(&self.hunger)
            || self.frame >= 10
            || self.eating_ticks > 20
            || !(300..=499).contains(&self.coin_threshold)
            || self.coin_timer >= self.coin_threshold
            || self.bought_timer > 45
            || self.movement_state >= 10
            || self.movement_timer >= 40
            || self.speed_change_timer > 20
            || self.hunger_animation_ticks > 5
        {
            return Err("invalid ordinary Grubber state".into());
        }
        Ok(())
    }

    pub fn sprite_frame(&self) -> u8 {
        self.frame
    }

    pub fn sprite_row(&self) -> u8 {
        if self.eating_ticks > 0 || self.scream_ticks > 100 {
            if self.hunger_visible() { 4 } else { 1 }
        } else if self.hunger_visible() {
            2
        } else {
            0
        }
    }

    pub fn hunger_visible(&self) -> bool {
        self.hunger < 301 && self.hunger_animation_ticks == 0
    }

    pub fn hunger_overlay_alpha(&self) -> u8 {
        (u16::from(self.hunger_animation_ticks) * 255 / 5) as u8
    }

    pub fn facing_right(&self) -> bool {
        self.vx >= 0.0
    }

    /// Caller supplies the current birth-ordered prey snapshot and the shared
    /// board RNG. One tick may report both starvation and a due larva; Board
    /// must form the corpse from `from_live` before discarding this actor.
    pub fn tick(
        &mut self,
        prey: &[GrubberPrey],
        alien_present: bool,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> GrubberUpdate {
        let mut out = GrubberUpdate::default();
        self.death_pose = None;
        self.speedy_ticks = self.speedy_ticks.saturating_sub(1);
        self.scream_ticks = self.scream_ticks.saturating_sub(1);
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
        if !alien_present {
            self.hunger -= 1;
            if self.hunger == 304 {
                self.hunger_shown = true;
                self.hunger_animation_ticks = 1;
            }
        }

        let mut seeking = false;
        if self.hunger < 1 {
            out.died = true;
            self.alive = false;
            self.death_pose = Some(self.current_death_pose());
        } else if self.hunger < 900 {
            let nearest = prey.iter().filter(|fish| fish.eligible).min_by_key(|fish| {
                let dx = ((self.x + 40.0) - f64::from(fish.widget_x + 40)) as i32;
                let dy = ((self.y + 40.0) - f64::from(fish.widget_y + 40)) as i32;
                i64::from(dx).pow(2) + i64::from(dy).pow(2)
            });
            if let Some(target) = nearest {
                seeking = true;
                let dx = ((self.x + 40.0) - f64::from(target.widget_x + 40)) as i32;
                let dy = ((self.y + 40.0) - f64::from(target.widget_y + 40)) as i32;
                if i64::from(dx).pow(2) + i64::from(dy).pow(2) < 10_000 {
                    self.speedy_ticks = 100;
                }
                if self.eat_delay >= 5 {
                    self.eat_delay = 0;
                    self.steer_toward(target.widget_x);
                }
                let cx = self.x + 40.0;
                let cy = self.y + 40.0;
                if self.y >= 355.0
                    && cx > f64::from(target.widget_x)
                    && cx < f64::from(target.widget_x + 80)
                    && cy > f64::from(target.widget_y - 20)
                    && cy < f64::from(target.widget_y + 160)
                {
                    self.speedy_ticks = 100;
                    self.vy = -14.0;
                }
                // Targeting and collision are distinct. The first overlapping
                // eligible prey wins, even if the nearest target is elsewhere.
                for fish in prey.iter().filter(|fish| fish.eligible) {
                    let fx = f64::from(fish.widget_x);
                    let fy = f64::from(fish.widget_y);
                    if cx > fx && cx < fx + 80.0 && cy > fy + 10.0 && cy < fy + 70.0 {
                        out.eaten_prey = Some(fish.id);
                        // Ordinary GameObject::Unk02 floors a meal's starting
                        // hunger at 300 before this species adds 1000.
                        self.hunger = (self.hunger.max(300) + 1000).min(1400);
                        self.speedy_ticks = 100;
                        if self.eating_ticks == 0 {
                            self.eating_ticks = 12;
                        }
                        break;
                    }
                    if self.eating_ticks == 0
                        && cx > fx
                        && cx < fx + 80.0
                        && cy > fy - 20.0
                        && cy < fy + 160.0
                    {
                        self.eating_ticks = 20;
                    }
                }
            }
        }

        if !seeking {
            self.target_vx = match self.movement_state {
                0 => -0.5,
                1 => 0.5,
                2 => -1.0,
                3 => 1.0,
                4 | 5 => 0.0,
                6 => -2.5,
                7 => 2.5,
                _ => self.target_vx,
            };
            if self.vx > self.target_vx {
                self.vx -= 0.1;
            }
            if self.vx < self.target_vx {
                self.vx += 0.1;
            }
        }
        self.eat_delay = self.eat_delay.saturating_add(1);
        self.speed_change_timer += 1;
        if self.speed_change_timer > 20 || self.x <= 10.0 || self.x >= 540.0 {
            self.speed_change_timer = 0;
            if rand_range(10) == 0 {
                self.movement_state = rand_range(8) as u8;
            }
        }
        if !alien_present {
            self.coin_timer += 1;
            if self.coin_timer >= self.coin_threshold {
                self.coin_timer = 0;
                out.larva_at = Some((self.widget_x + 4, self.widget_y - 10));
            }
        }
        self.x = self.x.clamp(10.0, 560.0);
        if self.y > 355.0 {
            self.y = 355.0;
            self.vy = 0.0;
        }
        if self.bought_timer > 0 {
            let divisor = if self.bought_timer > 35 { 8 } else { 5 };
            if rand_range(divisor) == 0 {
                let bx = self.widget_x + 55 - rand_range(60) as i32;
                let by = self.widget_y + 55 - rand_range(60) as i32;
                out.bubbles.push((bx, by));
            }
            self.bought_timer -= 1;
        } else if self.y < 95.0 {
            self.y = 95.0;
        }
        if self.x > 535.0 && self.vx > 0.1 {
            self.vx -= 0.1;
        }
        if self.x < 15.0 && self.vx < -0.1 {
            self.vx += 0.1;
        }
        if self.y < 355.0 {
            self.vy += 0.4;
        }
        self.animate();
        self.x += self.vx / self.speed_mod;
        self.y += self.vy / self.speed_mod;
        self.widget_x = self.x as i32;
        self.widget_y = self.y as i32;
        out
    }

    fn current_death_pose(&self) -> DeathPose {
        DeathPose {
            x: self.x as i32,
            y: self.y as i32,
            vx: self.vx,
            vy: self.vy,
            speed_mod: self.speed_mod,
        }
    }

    fn steer_toward(&mut self, prey_x: i32) {
        let center = (self.x + 40.0) as i32;
        let hungry = self.hunger <= 300;
        let (far_limit, far_delta, near_limit, near_delta) = if hungry {
            (4.0, 1.5, 2.5, 0.8)
        } else {
            (2.5, 0.8, 1.5, 0.6)
        };
        if center > prey_x + 48 {
            if self.vx > -far_limit {
                self.vx -= far_delta;
            }
        } else if center < prey_x + 24 {
            if self.vx < far_limit {
                self.vx += far_delta;
            }
        } else if center > prey_x + 36 {
            if self.vx > -near_limit {
                self.vx -= near_delta;
            }
        } else if center < prey_x + 36 && self.vx < near_limit {
            self.vx += near_delta;
        }
    }

    fn animate(&mut self) {
        if self.eating_ticks > 0 {
            self.eating_ticks -= 1;
            self.frame = 9 - self.eating_ticks / 2;
        } else {
            let period = if self.vx.abs() > 1.0 { 20 } else { 40 };
            if self.vx > 0.0 {
                self.movement_timer = (self.movement_timer + 1) % period;
            } else {
                self.movement_timer = (self.movement_timer + period - 1) % period;
            }
            self.frame = self.movement_timer / (period / 10);
        }
        if self.scream_ticks > 100 {
            self.frame = 4;
        }
    }
}

impl DeadGrubber {
    pub fn sprite_row(&self) -> u8 {
        3
    }

    pub fn sprite_frame(&self) -> u8 {
        self.frame
    }

    /// Starvation invokes Die inside hunger handling before the rest of its
    /// update, so preserve that snapshot instead of the post-tail actor pose.
    pub fn from_live(actor: &GrubberState) -> Self {
        let pose = actor
            .death_pose
            .expect("Grubber starvation corpse requires death-tick pose");
        Self::from_pose(actor.id, pose)
    }

    /// Missile impact occurs outside Grubber's hunger hook at its current pose.
    pub fn from_impact(actor: &GrubberState) -> Self {
        Self::from_pose(actor.id, actor.current_death_pose())
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
            || !(0.0..=570.0).contains(&self.x)
            || !(0.0..=380.0).contains(&self.y)
            || self.widget_x != self.x as i32
            || self.widget_y != self.y as i32
            || ![2.5, 2.6, 2.7].contains(&self.speed_mod)
            || self.frame >= 10
            || self.remaining_ticks > 125
            || !self.opacity.is_finite()
            || !(0.0..=1.0).contains(&self.opacity)
        {
            return Err("invalid ordinary Grubber corpse state".into());
        }
        Ok(())
    }

    /// True on the update after the remaining counter reaches zero.
    pub fn tick(&mut self) -> bool {
        let remaining = self.remaining_ticks;
        // W1 DeadFish.cpp:67-73 selects the type-8/9 body animation. Grubber
        // is type 9, so its final frame stays at 9 through the fade.
        self.frame = if remaining >= 106 {
            9 - ((remaining - 106) / 2) as u8
        } else {
            9
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

    fn grubber() -> GrubberState {
        let mut ranges = vec![];
        let g = GrubberState::spawn_bought(1, &mut |range| {
            ranges.push(range);
            0
        });
        assert_eq!(ranges, [520, 5, 3, 200, 10, 200]);
        assert_eq!((g.widget_x, g.widget_y, g.y), (20, 65, 65.0));
        g
    }

    #[test]
    fn nearest_uses_truncated_squared_distance_with_first_tie() {
        let mut g = grubber();
        g.x = 100.9;
        g.y = 100.9;
        g.widget_x = 100;
        g.widget_y = 100;
        g.hunger = 899;
        g.bought_timer = 0;
        let views = [
            GrubberPrey {
                id: 2,
                widget_x: 150,
                widget_y: 100,
                eligible: true,
            },
            GrubberPrey {
                id: 3,
                widget_x: 100,
                widget_y: 150,
                eligible: true,
            },
        ];
        g.tick(&views, false, &mut |_| 1);
        assert!(g.vx > 0.0, "first equal-distance prey is to the right");
    }

    #[test]
    fn strict_contact_excludes_edges_and_eats_first_overlapping_eligible() {
        let mut g = grubber();
        g.x = 100.0;
        g.y = 100.0;
        g.widget_x = 100;
        g.widget_y = 100;
        g.hunger = 800;
        g.bought_timer = 0;
        let edge = GrubberPrey {
            id: 2,
            widget_x: 140,
            widget_y: 100,
            eligible: true,
        };
        assert_eq!(g.tick(&[edge], false, &mut |_| 1).eaten_prey, None);
        g.x = 100.0;
        g.widget_x = 100;
        g.vx = 0.0;
        let inner = GrubberPrey {
            id: 3,
            widget_x: 139,
            widget_y: 100,
            eligible: true,
        };
        assert_eq!(
            g.tick(&[edge, inner], false, &mut |_| 1).eaten_prey,
            Some(3)
        );
        assert!(g.hunger > 800);
    }

    #[test]
    fn meal_floor_applies_before_thousand_hunger_restore() {
        let mut g = grubber();
        g.x = 100.0;
        g.y = 100.0;
        g.widget_x = 100;
        g.widget_y = 100;
        g.hunger = 300;
        g.bought_timer = 0;
        let prey = GrubberPrey {
            id: 2,
            widget_x: 100,
            widget_y: 100,
            eligible: true,
        };
        let update = g.tick(&[prey], false, &mut |_| 1);
        assert_eq!(update.eaten_prey, Some(2));
        assert_eq!(g.hunger, 1300);
    }

    #[test]
    fn starvation_retains_pre_tail_corpse_and_due_production() {
        let mut g = grubber();
        g.hunger = 1;
        g.coin_threshold = 300;
        g.coin_timer = 299;
        let old = (g.widget_x, g.widget_y);
        let result = g.tick(&[], false, &mut |_| 1);
        assert!(result.died);
        assert_eq!(result.larva_at, Some((old.0 + 4, old.1 - 10)));
        let body = DeadGrubber::from_live(&g);
        assert_eq!((body.widget_x, body.widget_y), old);
        assert_ne!(g.y, f64::from(old.1));
        let impact = DeadGrubber::from_impact(&g);
        assert_eq!(impact.widget_y, g.widget_y);
    }

    #[test]
    fn alien_registration_freezes_hunger_and_production_but_not_motion() {
        let mut g = grubber();
        g.hunger = 301;
        g.coin_timer = g.coin_threshold - 1;
        let old_y = g.y;
        let result = g.tick(&[], true, &mut |_| 1);
        assert_eq!(g.hunger, 301);
        assert_eq!(g.coin_timer, g.coin_threshold - 1);
        assert_eq!(result.larva_at, None);
        assert!(g.y > old_y);
    }

    #[test]
    fn serialized_state_preserves_fractional_next_step() {
        let mut g = grubber();
        g.tick(&[], false, &mut |_| 1);
        let saved = serde_json::to_string(&g).unwrap();
        let mut restored: GrubberState = serde_json::from_str(&saved).unwrap();
        restored.validate().unwrap();
        g.tick(&[], false, &mut |_| 1);
        restored.tick(&[], false, &mut |_| 1);
        assert_eq!(
            (g.x, g.y, g.coin_timer),
            (restored.x, restored.y, restored.coin_timer)
        );
    }

    #[test]
    fn a_live_starved_actor_is_not_a_stable_save_state() {
        let mut g = grubber();
        g.hunger = 0;
        assert!(g.validate().is_err());
        g.hunger = 1;
        g.validate().unwrap();
    }

    #[test]
    fn grubber_corpse_holds_frame_nine_after_the_early_sequence() {
        let g = grubber();
        let mut corpse = DeadGrubber::from_impact(&g);
        corpse.remaining_ticks = 106;
        corpse.tick();
        assert_eq!(corpse.frame, 9);
        corpse.remaining_ticks = 105;
        corpse.tick();
        assert_eq!(corpse.frame, 9);
        corpse.remaining_ticks = 104;
        corpse.tick();
        assert_eq!(corpse.frame, 9);
        corpse.remaining_ticks = 100;
        corpse.tick();
        assert_eq!(corpse.frame, 9);
    }
}
