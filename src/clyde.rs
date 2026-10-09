//! Ordinary Clyde actor. W1 f919b3c `OtherTypePet.cpp` supplies movement and
//! animation; installed PB28 corrects the nearest-coin metric to both centers
//! at +40. Board retains ordered coins, eligibility, identity and payout.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClydeCoinView {
    pub id: u64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub eligible: bool,
    /// Special notes remain chase targets but cannot be collected on contact.
    pub collectible: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClydeUpdate {
    pub collected_coin: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClydeState {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub vx: f64,
    pub vy: f64,
    pub target_vx: f64,
    pub frame: u8,
    movement_state: u8,
    movement_timer: u8,
    chase_timer: u32,
    animation_timer: u8,
}

impl ClydeState {
    /// Board spawn coordinates, followed by OtherTypePet's two constructor
    /// draws. The final draw seeds a timer unused by ordinary Clyde.
    pub fn spawn_tank2(id: u64, rand_range: &mut impl FnMut(u64) -> u64) -> Self {
        let widget_x = rand_range(265) as i32 + 105;
        let widget_y = rand_range(520) as i32 + 20;
        let movement_state = rand_range(10) as u8;
        let _unused_specialty_seed = rand_range(250) + 250;
        Self {
            id,
            x: f64::from(widget_x),
            y: f64::from(widget_y),
            widget_x,
            widget_y,
            vx: 0.0,
            vy: 0.0,
            target_vx: 0.0,
            frame: 0,
            movement_state,
            movement_timer: 0,
            chase_timer: 40,
            animation_timer: 0,
        }
    }

    /// `coin_list_nonempty` describes the whole Board list, including coins
    /// whose views are ineligible. Such a list suppresses fallback wandering.
    /// Views retain birth order so first-tie targeting and contact stay stable.
    pub fn tick(
        &mut self,
        coins: &[ClydeCoinView],
        coin_list_nonempty: bool,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> ClydeUpdate {
        let mut update = ClydeUpdate::default();
        if coin_list_nonempty {
            let nearest = coins
                .iter()
                .filter(|coin| coin.eligible)
                .min_by_key(|coin| {
                    // PB28 raw instructions use +40 on both operands. Cast
                    // each subtraction before squaring, as the source helper
                    // truncates each double axis toward zero.
                    let dx = (self.x + 40.0 - (f64::from(coin.widget_x) + 40.0)) as i32;
                    let dy = (self.y + 40.0 - (f64::from(coin.widget_y) + 40.0)) as i32;
                    i64::from(dx) * i64::from(dx) + i64::from(dy) * i64::from(dy)
                });
            if let Some(target) = nearest {
                if self.chase_timer > 4 {
                    self.chase_timer = 0;
                    let center_x = self.x + 40.0;
                    // Installed type-5 ChaseEntity uses +36 for steering,
                    // distinct from PB28's +40 nearest-distance metric.
                    let target_x = f64::from(target.widget_x) + 36.0;
                    if center_x > target_x && self.vx > -2.0 {
                        self.vx -= 1.0;
                    } else if center_x < target_x && self.vx < 2.0 {
                        self.vx += 1.0;
                    }
                    let center_y = self.y + 40.0;
                    let target_y = f64::from(target.widget_y) + 36.0;
                    if center_y > target_y && self.vy > -2.0 {
                        self.vy -= 1.0;
                    } else if center_y < target_y && self.vy < 2.0 {
                        self.vy += 1.0;
                    }
                }
                update.collected_coin = coins
                    .iter()
                    .filter(|coin| coin.eligible && coin.collectible)
                    .find(|coin| {
                        let center_x = self.x + 40.0;
                        let center_y = self.y + 40.0;
                        center_x > f64::from(coin.widget_x) + 16.0
                            && center_x < f64::from(coin.widget_x) + 56.0
                            && center_y > f64::from(coin.widget_y) + 16.0
                            && center_y < f64::from(coin.widget_y) + 56.0
                    })
                    .map(|coin| coin.id);
            }
        } else {
            match self.movement_state {
                0 => self.target_vx = 0.0,
                1 => self.target_vx = -0.5,
                2 => self.target_vx = 0.5,
                _ => {} // Constructor can seed 3..9; retain the last target.
            }
        }

        // The two source comparisons are sequential. Near zero, the first
        // adjustment can make the second undo it on the same update.
        if self.target_vx < self.vx {
            self.vx -= 0.1;
        }
        if self.target_vx > self.vx {
            self.vx += 0.1;
        }
        if matches!(self.animation_timer, 2 | 3) {
            self.vy -= if self.y >= 240.0 { 0.75 } else { 0.3 };
        } else {
            self.vy += 0.03;
        }

        // OtherTypePet runs Clyde's chase/easing/pulse first. The counter
        // increments and possible movement-state roll affect the next update.
        self.movement_timer += 1;
        self.chase_timer = self.chase_timer.saturating_add(1);
        if self.movement_timer > 20
            || (self.x <= 10.0 && self.target_vx <= 0.0)
            || (self.x >= 540.0 && self.y >= 0.0)
        {
            self.movement_timer = 0;
            if rand_range(10) == 0 {
                self.movement_state = rand_range(3) as u8;
            }
        }

        // OtherTypePet clamps before animation and movement. Spawn Y may be
        // below the tank until this first update.
        self.x = self.x.clamp(10.0, 550.0);
        if self.y > 370.0 {
            self.y = 370.0;
            self.vy = 0.0;
        }
        if self.y < 95.0 {
            self.y = 95.0;
        }
        if self.x > 535.0 && self.vx > 0.1 {
            self.movement_state = 1;
        }
        if self.x < 15.0 && self.vx < -0.1 {
            self.movement_state = 2;
        }
        self.animation_timer = (self.animation_timer + 1) % 40;
        self.frame = match self.animation_timer {
            0..=6 => self.animation_timer / 2,
            7..=15 => 4,
            _ => self.animation_timer / 4,
        };
        self.x += self.vx / 1.5;
        self.y += self.vy / 1.5;
        self.widget_x = self.x as i32;
        self.widget_y = self.y as i32;
        update
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0
            || !self.x.is_finite()
            || !self.y.is_finite()
            || !self.vx.is_finite()
            || !self.vy.is_finite()
            || !self.target_vx.is_finite()
            || !(0.0..=560.0).contains(&self.x)
            || !(0.0..=550.0).contains(&self.y)
            || self.vx.abs() > 100.0
            || self.vy.abs() > 100.0
            || ![-0.5, 0.0, 0.5].contains(&self.target_vx)
            || self.widget_x != self.x as i32
            || self.widget_y != self.y as i32
            || self.movement_state > 9
            || self.movement_timer > 20
            || self.animation_timer >= 40
            || self.frame > 9
        {
            return Err("invalid Clyde save state".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clyde() -> ClydeState {
        ClydeState::spawn_tank2(1, &mut |_| 0)
    }

    #[test]
    fn spawn_consumes_board_and_constructor_draws_and_clamps_only_on_tick() {
        let mut ranges = Vec::new();
        let mut actor = ClydeState::spawn_tank2(1, &mut |upper| {
            ranges.push(upper);
            upper - 1
        });
        assert_eq!(ranges, [265, 520, 10, 250]);
        assert_eq!((actor.widget_x, actor.widget_y), (369, 539));
        assert_eq!(actor.chase_timer, 40);
        actor.tick(&[], false, &mut |_| 1);
        assert_eq!(actor.widget_y, 370);
        assert!(actor.validate().is_ok());
    }

    #[test]
    fn ineligible_nonempty_list_holds_wander_target_during_invasion() {
        let mut actor = clyde();
        actor.movement_state = 2;
        actor.target_vx = -0.5;
        let blocked = ClydeCoinView {
            id: 2,
            widget_x: 150,
            widget_y: 140,
            eligible: false,
            collectible: true,
        };
        actor.tick(&[blocked], true, &mut |_| 1);
        assert_eq!(actor.target_vx, -0.5);
        assert_eq!(actor.chase_timer, 41);
        actor.tick(&[], true, &mut |_| 1);
        assert_eq!(actor.target_vx, -0.5); // Null target still suppresses fallback.
        actor.tick(&[], false, &mut |_| 1);
        assert_eq!(actor.target_vx, 0.5);
        let target = ClydeCoinView {
            eligible: true,
            ..blocked
        };
        actor.tick(&[target], true, &mut |_| 1);
        assert_eq!(actor.chase_timer, 1);
    }

    #[test]
    fn nearest_truncates_each_axis_first_ties_and_contact_uses_strict_box() {
        let mut actor = clyde();
        actor.x = 100.8;
        actor.y = 100.8;
        actor.widget_x = 100;
        actor.widget_y = 100;
        let first = ClydeCoinView {
            id: 2,
            widget_x: 99,
            widget_y: 101,
            eligible: true,
            collectible: true,
        };
        let second = ClydeCoinView {
            id: 3,
            widget_x: 102,
            ..first
        };
        let hit = actor.tick(&[first, second], true, &mut |_| 1);
        assert_eq!(hit.collected_coin, Some(2));
        assert!((actor.vx + 0.9).abs() < 1e-12); // Truncated distance ties; first steers left.
        let mut edge = clyde();
        edge.x = 115.0;
        edge.y = 117.0;
        edge.widget_x = 115;
        edge.widget_y = 117;
        // Center X=155 lies exactly on coin x+56, outside the strict box.
        assert_eq!(edge.tick(&[first], true, &mut |_| 1).collected_coin, None);
    }

    #[test]
    fn chase_guards_allow_overshoot_and_sequential_easing_can_cancel() {
        let mut actor = clyde();
        actor.x = 200.0;
        actor.widget_x = 200;
        actor.vx = 1.5;
        actor.vy = 1.5;
        let target = ClydeCoinView {
            id: 2,
            widget_x: 250,
            widget_y: 250,
            eligible: true,
            collectible: true,
        };
        actor.tick(&[target], true, &mut |_| 1);
        assert!((actor.vx - 2.4).abs() < 1e-12); // +1 under the <2 guard, then -0.1 easing.
        assert!((actor.vy - 2.53).abs() < 1e-12); // +1 under the <2 guard, then +0.03 pulse.
        actor.vx = 0.05;
        actor.target_vx = 0.0;
        actor.chase_timer = 0;
        actor.tick(&[target], true, &mut |_| 1);
        assert!((actor.vx - 0.05).abs() < 1e-12);
    }

    #[test]
    fn vertical_pulse_and_frame_boundaries_use_old_animation_counter() {
        let mut actor = clyde();
        actor.y = 239.0;
        actor.widget_y = 239;
        actor.animation_timer = 2;
        actor.tick(&[], false, &mut |_| 1);
        assert!((actor.vy + 0.3).abs() < 1e-12);
        assert_eq!(actor.frame, 1); // counter increments 2 -> 3
        actor.y = 240.0;
        actor.widget_y = 240;
        actor.animation_timer = 3;
        actor.tick(&[], false, &mut |_| 1);
        assert!((actor.vy + 1.05).abs() < 1e-12);
        assert_eq!(actor.frame, 2); // counter increments 3 -> 4
        actor.animation_timer = 6;
        actor.tick(&[], false, &mut |_| 1);
        assert_eq!(actor.frame, 4); // counter 7 enters held frame
        actor.animation_timer = 15;
        actor.tick(&[], false, &mut |_| 1);
        assert_eq!(actor.frame, 4); // counter 16 / 4
        actor.animation_timer = 39;
        actor.tick(&[], false, &mut |_| 1);
        assert_eq!(actor.frame, 0);
    }

    #[test]
    fn movement_change_roll_uses_ten_then_three_and_changes_next_tick_target() {
        let mut actor = clyde();
        actor.movement_timer = 20;
        let mut ranges = Vec::new();
        actor.tick(&[], false, &mut |upper| {
            ranges.push(upper);
            if upper == 10 { 0 } else { 2 }
        });
        assert_eq!(ranges, [10, 3]);
        assert_eq!(actor.movement_state, 2);
        assert_eq!(actor.target_vx, 0.0); // Old movement state 0 drove this tick.
        assert_eq!(actor.movement_timer, 0);
        actor.tick(&[], false, &mut |_| 1);
        assert_eq!(actor.target_vx, 0.5);
    }

    #[test]
    fn chase_counter_old_four_waits_and_steering_reset_is_incremented_in_tail() {
        let mut actor = clyde();
        actor.chase_timer = 4;
        let coin = ClydeCoinView {
            id: 2,
            widget_x: 200,
            widget_y: 150,
            eligible: true,
            collectible: true,
        };
        actor.tick(&[coin], true, &mut |_| 1);
        assert_eq!(actor.vx, 0.0);
        assert_eq!(actor.chase_timer, 5);
        actor.tick(&[coin], true, &mut |_| 1);
        assert!((actor.vx - 0.9).abs() < 1e-12);
        assert_eq!(actor.chase_timer, 1);
    }

    #[test]
    fn steering_uses_coin_plus_thirty_six_even_when_nearest_uses_plus_forty() {
        let mut actor = clyde();
        actor.x = 100.0;
        actor.y = 100.0;
        actor.widget_x = 100;
        actor.widget_y = 100;
        actor.chase_timer = 5;
        let coin = ClydeCoinView {
            id: 2,
            widget_x: 102,
            widget_y: 102,
            eligible: true,
            collectible: true,
        };
        actor.tick(&[coin], true, &mut |_| 1);
        assert!((actor.vx + 0.9).abs() < 1e-12);
        assert!((actor.vy + 0.97).abs() < 1e-12);
    }
}
