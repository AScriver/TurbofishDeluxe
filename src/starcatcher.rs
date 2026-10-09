//! Ordinary Adventure 2-2 Starcatcher (Penta). Rules are derived from pinned
//! WinFish W1 `Penta.cpp`, `GameObject.cpp`, and `DeadFish.cpp` at f919b3c;
//! PB29 confirms the typed hunger/death and special-coin path in the installed
//! payload. The Board owns coin membership, diamond identity, and transactions.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StarcatcherCoinView {
    pub id: u64,
    pub widget_x: i32,
    pub widget_y: i32,
    /// An unclaimed ordinary star in the Board's birth order.
    pub eligible: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StarcatcherUpdate {
    pub eaten_coin: Option<u64>,
    /// Origin of the distinct rising diamond, from the old integer widget.
    pub diamond_at: Option<(i32, i32)>,
    pub died: bool,
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
pub struct StarcatcherState {
    pub id: u64,
    pub alive: bool,
    pub x: f64,
    pub y: f64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub vx: f64,
    pub vy: f64,
    pub hunger: i32,
    /// Zero on bought construction; entrance animation is not alien immunity.
    pub cannot_be_eaten_ticks: u8,
    pub bought_timer: u8,
    pub frame: u8,
    pub hunger_animation_timer: i8,
    pub hunger_shown: bool,
    /// The ordinary song sentinel is -1. Other values freeze hunger loss.
    pub song_id: i32,
    speed_mod: f64,
    target_vx: f64,
    speed_state: u8,
    eat_delay: u32,
    speed_change_timer: u8,
    movement_animation_timer: u8,
    coin_drop_timer: u8,
    speedy_speed_ticks: u8,
    #[serde(skip)]
    death_pose: Option<DeathPose>,
}

impl StarcatcherState {
    /// The Board's X draw precedes Penta's overwritten constructor-Y draw.
    /// A normal bought Penta starts at Y=65, with no entrance immunity.
    pub fn spawn_bought(id: u64, rand_range: &mut impl FnMut(u64) -> u64) -> Self {
        let x = rand_range(520) as i32 + 20;
        let mut actor = Self::spawn_at(id, x, rand_range);
        actor.y = 65.0;
        actor.widget_y = 65;
        actor
    }

    /// Penta's one-coordinate resurrection constructor samples a new floor Y.
    pub fn spawn_revived(id: u64, x: i32, rand_range: &mut impl FnMut(u64) -> u64) -> Self {
        Self::spawn_at(id, x, rand_range)
    }

    fn spawn_at(id: u64, x: i32, rand_range: &mut impl FnMut(u64) -> u64) -> Self {
        let y = rand_range(5) as i32 + 360;
        let speed_mod = match rand_range(3) {
            0 => 2.7,
            1 => 2.5,
            _ => 2.6,
        };
        let hunger = rand_range(200) as i32 + 900;
        let speed_state = rand_range(10) as u8;
        Self {
            id,
            alive: true,
            x: f64::from(x),
            y: f64::from(y),
            widget_x: x,
            widget_y: y,
            vx: 0.0,
            vy: 0.0,
            hunger,
            cannot_be_eaten_ticks: 0,
            bought_timer: 45,
            frame: 0,
            hunger_animation_timer: 0,
            hunger_shown: false,
            song_id: -1,
            speed_mod,
            target_vx: 0.0,
            speed_state,
            eat_delay: 40,
            speed_change_timer: 0,
            movement_animation_timer: 0,
            coin_drop_timer: 0,
            speedy_speed_ticks: 0,
            death_pose: None,
        }
    }

    pub fn sprite_frame(&self) -> u8 {
        self.frame
    }

    pub fn sprite_row(&self) -> u8 {
        u8::from(self.hunger_visible())
    }

    pub fn hunger_visible(&self) -> bool {
        self.hunger < 301 && self.hunger_animation_timer == 0
    }

    pub fn hunger_overlay_alpha(&self) -> f32 {
        f32::from(self.hunger_animation_timer.max(0)) / 5.0
    }

    /// The live sprite itself is drawn unmirrored; VX still drives animation.
    pub fn facing_right(&self) -> bool {
        self.vx >= 0.0
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0
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
            || !(-64.0..=640.0).contains(&self.x)
            || !(0.0..=480.0).contains(&self.y)
            || self.widget_x != self.x as i32
            || self.widget_y != self.y as i32
            || ![2.5, 2.6, 2.7].contains(&self.speed_mod)
            || !(-1000..=1300).contains(&self.hunger)
            || self.cannot_be_eaten_ticks != 0
            || self.bought_timer > 45
            || self.frame > 9
            || !(-1..=5).contains(&self.hunger_animation_timer)
            || self.song_id < -1
            || self.speed_state > 9
            || self.speed_change_timer > 20
            || self.movement_animation_timer >= 40
            || self.coin_drop_timer > 2
            || self.vx.abs() > 12.0
            || self.vy.abs() > 30.0
            || self.target_vx.abs() > 3.0
        {
            return Err("invalid ordinary Starcatcher save state".into());
        }
        Ok(())
    }

    pub fn tick(
        &mut self,
        coins: &[StarcatcherCoinView],
        alien_present: bool,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> StarcatcherUpdate {
        if !self.alive {
            return StarcatcherUpdate::default();
        }
        self.cannot_be_eaten_ticks = self.cannot_be_eaten_ticks.saturating_sub(1);
        self.speedy_speed_ticks = self.speedy_speed_ticks.saturating_sub(1);
        self.update_hunger_animation();
        if !alien_present && self.song_id == -1 {
            self.hunger = (self.hunger - 1).max(-1000);
            if self.hunger == 304 {
                self.hunger_shown = true;
                self.hunger_animation_timer = 1;
            }
        }
        let mut result = StarcatcherUpdate::default();
        if self.hunger < 1 {
            // Die runs inside HungryLogic; the rest of Penta::Update still
            // executes, while the corpse keeps this pre-tail integer pose.
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
        let hunting = if self.alive && self.hunger < 900 {
            self.hunt(coins, &mut result)
        } else {
            false
        };
        if !hunting {
            self.target_vx = match self.speed_state {
                1 => -0.5,
                2 => 0.5,
                3 => -1.0,
                4 => 1.0,
                7 => -2.5,
                8 => 2.5,
                9 => self.target_vx,
                _ => 0.0,
            };
            // The source has two sequential comparisons, allowing a small
            // overshoot correction during one update.
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
                self.speed_state = rand_range(9) as u8;
            }
        }
        self.x = self.x.clamp(10.0, 550.0);
        if self.y > 359.0 {
            self.y = 359.0;
            self.vy = 0.0;
        }
        if self.bought_timer == 0 {
            self.y = self.y.max(95.0);
        } else {
            let chance = if self.bought_timer > 35 { 8 } else { 5 };
            if rand_range(chance) == 0 {
                // Bubble construction/board limits can consume more RNG. The
                // two coordinate draws are the directly recovered schedule.
                let _bubble_x = self.widget_x + 55 - rand_range(60) as i32;
                let _bubble_y = self.widget_y + 55 - rand_range(60) as i32;
            }
            self.bought_timer -= 1;
        }
        if self.x > 535.0 && self.vx > 0.1 {
            self.vx -= 0.1;
        }
        if self.x < 15.0 && self.vx < -0.1 {
            self.vx += 0.1;
        }
        if self.y < 359.0 {
            self.vy += 0.4;
        }
        self.animate();
        self.x += self.vx / self.speed_mod;
        self.y += self.vy / self.speed_mod;
        self.widget_x = self.x as i32;
        self.widget_y = self.y as i32;
        result
    }

    fn update_hunger_animation(&mut self) {
        if self.hunger_animation_timer > 0 {
            if self.hunger_shown {
                self.hunger_animation_timer += 1;
                if self.hunger_animation_timer > 5 {
                    self.hunger_animation_timer = 0;
                }
            } else {
                self.hunger_animation_timer -= 1;
            }
        }
    }

    fn hunt(&mut self, coins: &[StarcatcherCoinView], result: &mut StarcatcherUpdate) -> bool {
        let nearest = coins
            .iter()
            .filter(|coin| coin.eligible)
            .min_by_key(|coin| {
                let dx = (self.x + 40.0 - f64::from(coin.widget_x + 36)) as i32;
                let dy = (self.y + 40.0 - f64::from(coin.widget_y + 36)) as i32;
                i64::from(dx) * i64::from(dx) + i64::from(dy) * i64::from(dy)
            });
        let Some(target) = nearest else {
            return false;
        };
        let dx = (self.x + 40.0 - f64::from(target.widget_x + 36)) as i32;
        let dy = (self.y + 40.0 - f64::from(target.widget_y + 36)) as i32;
        if i64::from(dx) * i64::from(dx) + i64::from(dy) * i64::from(dy) < 10000 {
            self.speedy_speed_ticks = 100;
        }
        if self.eat_delay >= 5 {
            self.eat_delay = 0;
            // Steering truncates the center to an integer; the later contact
            // test uses the full double center.
            let center = (self.x + 40.0) as i32;
            let target_x = target.widget_x;
            let urgent = self.hunger <= 300;
            let (wide_step, narrow_step, wide_limit, narrow_limit) = if urgent {
                (2.5, 1.2, 6.0, 2.5)
            } else {
                (1.8, 1.0, 4.5, 1.5)
            };
            if center > target_x + 46 && self.vx > -wide_limit {
                self.vx -= wide_step;
            } else if center < target_x + 26 && self.vx < wide_limit {
                self.vx += wide_step;
            } else if center > target_x + 36 && self.vx > -narrow_limit {
                self.vx -= narrow_step;
            } else if center < target_x + 36 && self.vx < narrow_limit {
                self.vx += narrow_step;
            }
        }
        let center_x = self.x + 40.0;
        let center_y = self.y + 40.0;
        if let Some(coin) = coins.iter().find(|coin| {
            coin.eligible
                && center_x > f64::from(coin.widget_x + 16)
                && center_x < f64::from(coin.widget_x + 56)
                && center_y > f64::from(coin.widget_y + 16)
                && center_y < f64::from(coin.widget_y + 66)
        }) {
            let was_hungry = self.hunger_visible();
            self.speedy_speed_ticks = 0;
            if self.hunger_animation_timer >= 0 {
                self.hunger = self.hunger.max(300);
            }
            self.hunger = (self.hunger + 900).min(1300);
            // Ordinary Adventure emits a special diamond on each meal; the
            // periodic virtual-tank production branch is not used here.
            self.coin_drop_timer = 0;
            if was_hungry && !self.hunger_visible() {
                self.hunger_shown = false;
                self.hunger_animation_timer = 5;
            }
            result.eaten_coin = Some(coin.id);
            result.diamond_at = Some((self.widget_x + 5, self.widget_y - 12));
        }
        true
    }

    fn animate(&mut self) {
        let (period, forward, fast) = if self.vx > 1.0 {
            (20, true, true)
        } else if self.vx < -1.0 {
            (20, false, true)
        } else if self.vx > 0.0 {
            (40, true, false)
        } else {
            (40, false, false)
        };
        self.movement_animation_timer = if forward {
            (self.movement_animation_timer + 1) % period
        } else {
            (self.movement_animation_timer + period - 1) % period
        };
        self.frame = self.movement_animation_timer / if fast { 2 } else { 4 };
    }
}

/// Ordinary non-Angel Starcatcher body, drawn from Starcatcher's corpse row.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeadStarcatcher {
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

impl DeadStarcatcher {
    pub fn from_live(actor: &StarcatcherState) -> Self {
        let pose = actor
            .death_pose
            .expect("Starcatcher corpse requires death-tick pose");
        Self::from_pose(actor.id, pose)
    }

    /// Missile impact calls Penta::Die at the current live widget, without
    /// entering the starvation hook that creates `death_pose`.
    pub fn from_impact(actor: &StarcatcherState) -> Self {
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
            revival_ticks: 100,
            vx: pose.vx,
            vy: pose.vy
                - if pose.x < 115 || pose.vy < -3.0 {
                    1.0
                } else {
                    2.0
                },
            speed_mod: pose.speed_mod,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0
            || ![self.x, self.y, self.vx, self.vy, self.speed_mod].into_iter().all(f64::is_finite)
            || !(-64.0..=640.0).contains(&self.x)
            // Starvation snapshots the pose before the normal update's
            // pre-motion bottom clamp, so the prior movement may overshoot.
            || !(0.0..=480.0).contains(&self.y)
            || self.widget_x != self.x as i32
            || self.widget_y != self.y as i32
            || ![2.5, 2.6, 2.7].contains(&self.speed_mod)
            || self.frame > 9
            || self.remaining_ticks > 125
            || !(self.revival_ticks == 100 || (0..=10).contains(&self.revival_ticks))
            || !self.opacity.is_finite()
            || !(0.0..=1.0).contains(&self.opacity)
        {
            return Err("invalid ordinary Starcatcher corpse save state".into());
        }
        Ok(())
    }

    /// True when the lifetime expires or Angie's revival countdown finishes.
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
        if remaining > 105 || self.y > 365.0 {
            self.remaining_ticks -= 1;
        }
        if self.vx < 0.0 {
            self.vx = (self.vx + 0.03).min(0.0);
        }
        if self.vx > 0.0 {
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
    fn revival_uses_penta_floor_constructor_not_bought_entrance() {
        let mut draws = Vec::new();
        let actor = StarcatcherState::spawn_revived(8, 190, &mut |upper| {
            draws.push(upper);
            0
        });
        assert_eq!(draws, [5, 3, 200, 10]);
        assert_eq!((actor.widget_x, actor.widget_y, actor.vy), (190, 360, 0.0));
        assert_eq!((actor.hunger, actor.bought_timer), (900, 45));
        actor.validate().unwrap();
    }

    #[test]
    fn constructor_draw_order_and_divisor_mapping_preserve_overwritten_y() {
        for (roll, divisor) in [(0, 2.7), (1, 2.5), (2, 2.6)] {
            let mut draws = vec![10, 4, roll, 199, 9].into_iter();
            let actor = StarcatcherState::spawn_bought(1, &mut |_| draws.next().unwrap());
            assert_eq!(
                (actor.widget_x, actor.widget_y, actor.hunger),
                (30, 65, 1099)
            );
            assert_eq!(actor.speed_mod, divisor);
            assert_eq!(actor.cannot_be_eaten_ticks, 0);
            assert_eq!(draws.next(), None);
        }
    }

    #[test]
    fn old_bought_timer_one_uses_entrance_y_before_next_tick_clamp() {
        let mut actor = StarcatcherState::spawn_bought(1, &mut |_| 1);
        actor.bought_timer = 1;
        actor.y = 65.0;
        actor.widget_y = 65;
        actor.tick(&[], false, &mut |_| 1);
        assert_eq!(actor.bought_timer, 0);
        assert!(actor.y < 95.0);
        actor.tick(&[], false, &mut |_| 1);
        assert!(actor.y >= 95.0);
    }

    #[test]
    fn starvation_runs_tail_but_corpse_keeps_pre_tail_pose_and_star() {
        let mut actor = StarcatcherState::spawn_bought(2, &mut |_| 1);
        actor.bought_timer = 0;
        actor.x = 101.8;
        actor.y = 200.5;
        actor.widget_x = 101;
        actor.widget_y = 200;
        actor.vx = 2.0;
        actor.hunger = 1;
        let star = StarcatcherCoinView {
            id: 9,
            widget_x: 105,
            widget_y: 200,
            eligible: true,
        };
        let update = actor.tick(&[star], false, &mut |_| 1);
        assert!(update.died);
        assert_eq!(update.eaten_coin, None);
        assert_ne!(actor.x, 101.8);
        let corpse = DeadStarcatcher::from_live(&actor);
        assert_eq!((corpse.widget_x, corpse.widget_y), (101, 200));
        corpse.validate().unwrap();
    }

    #[test]
    fn hungry_threshold_and_first_overlap_are_independent_of_nearest() {
        let mut actor = StarcatcherState::spawn_bought(3, &mut |_| 1);
        actor.bought_timer = 0;
        actor.x = 100.0;
        actor.y = 200.0;
        actor.widget_x = 100;
        actor.widget_y = 200;
        actor.hunger = 900;
        let first = StarcatcherCoinView {
            id: 5,
            widget_x: 90,
            widget_y: 185,
            eligible: true,
        };
        let second = StarcatcherCoinView {
            id: 6,
            widget_x: 104,
            widget_y: 201,
            eligible: true,
        };
        assert_eq!(
            actor.tick(&[first, second], true, &mut |_| 1).eaten_coin,
            None
        );
        assert_eq!(actor.widget_x, 99);
        actor.hunger = 899;
        let update = actor.tick(&[first, second], true, &mut |_| 1);
        assert_eq!(update.eaten_coin, Some(5));
        // The first update moved the widget from X=100 to X=99; the next
        // meal emits at that prior integer widget, before its own movement.
        assert_eq!(update.diamond_at, Some((104, 188)));
    }

    #[test]
    fn steering_truncates_fractional_center_but_contact_keeps_fraction() {
        let mut actor = StarcatcherState::spawn_bought(6, &mut |_| 1);
        actor.x = 102.9;
        actor.y = 200.0;
        actor.widget_x = 102;
        actor.widget_y = 200;
        actor.hunger = 899;
        actor.eat_delay = 5;
        let star = StarcatcherCoinView {
            id: 7,
            widget_x: 96,
            widget_y: 200,
            eligible: true,
        };
        let mut update = StarcatcherUpdate::default();
        assert!(actor.hunt(&[star], &mut update));
        // int(102.9 + 40) equals the outer steering boundary 96 + 46;
        // the double center remains strictly inside the contact rectangle.
        assert_eq!(actor.vx, -1.0);
        assert_eq!(update.eaten_coin, Some(7));
    }

    #[test]
    fn meal_floor_cap_respects_negative_imported_animation_sentinel() {
        let mut actor = StarcatcherState::spawn_bought(4, &mut |_| 1);
        actor.bought_timer = 0;
        actor.x = 100.0;
        actor.y = 200.0;
        actor.widget_x = 100;
        actor.widget_y = 200;
        let star = StarcatcherCoinView {
            id: 8,
            widget_x: 100,
            widget_y: 200,
            eligible: true,
        };
        actor.hunger = 20;
        actor.tick(&[star], true, &mut |_| 1);
        assert_eq!(actor.hunger, 1200);
        actor.hunger = 20;
        actor.hunger_animation_timer = -1;
        actor.tick(&[star], true, &mut |_| 1);
        assert_eq!(actor.hunger, 920);
    }

    #[test]
    fn zero_velocity_uses_backward_slow_animation_and_corpse_waits_to_sink() {
        let mut actor = StarcatcherState::spawn_bought(5, &mut |_| 1);
        actor.movement_animation_timer = 0;
        actor.vx = 0.0;
        actor.animate();
        assert_eq!((actor.movement_animation_timer, actor.frame), (39, 9));
        actor.death_pose = Some(DeathPose {
            x: 100,
            y: 200,
            vx: 0.0,
            vy: 0.0,
            speed_mod: 2.5,
            facing_right: true,
        });
        let mut corpse = DeadStarcatcher::from_live(&actor);
        corpse.remaining_ticks = 105;
        corpse.y = 360.0;
        corpse.widget_y = 360;
        corpse.tick();
        assert_eq!(corpse.remaining_ticks, 105);
    }

    #[test]
    fn corpse_vertical_acceleration_tests_old_velocity_without_clamping() {
        let actor = StarcatcherState::spawn_bought(8, &mut |_| 1);
        let mut near_limit = DeadStarcatcher::from_live(&StarcatcherState {
            death_pose: Some(DeathPose {
                x: 100,
                y: 200,
                vx: 0.0,
                vy: 0.0,
                speed_mod: 2.5,
                facing_right: true,
            }),
            ..actor
        });
        near_limit.vy = 1.99;
        let mut above_limit = near_limit.clone();
        above_limit.vy = 2.4;
        near_limit.tick();
        above_limit.tick();
        assert!((near_limit.vy - 2.04).abs() < 1e-9);
        assert_eq!(above_limit.vy, 2.4);
    }

    #[test]
    fn missile_impact_corpse_uses_current_penta_widget_without_starvation_pose() {
        let mut actor = StarcatcherState::spawn_bought(9, &mut |_| 1);
        actor.widget_x = 150;
        actor.widget_y = 220;
        actor.x = 150.75;
        actor.y = 220.25;
        actor.vx = -1.0;
        actor.vy = -4.0;
        assert!(actor.death_pose.is_none());
        let corpse = DeadStarcatcher::from_impact(&actor);
        assert_eq!(
            (corpse.widget_x, corpse.widget_y, corpse.vy),
            (150, 220, -5.0)
        );
        assert!(!corpse.facing_right);
        corpse.validate().unwrap();
    }
}
