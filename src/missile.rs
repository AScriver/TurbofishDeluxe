//! Ordinary Destructor missile, with Board-owned target membership.
//!
//! W1 `Missle.cpp` at f919b3c supplies the classic homing/contact rules;
//! PB34 identifies the installed constructor and shot dispatch. Trail
//! particles and their random draws are not represented yet.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MissilePreyView {
    pub id: u64,
    pub widget_x: i32,
    pub widget_y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StanleyTargetView {
    pub id: u64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub group: bool,
    pub kind_six: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MissileUpdate {
    pub impact_target: Option<u64>,
    pub remove: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissileKind {
    Classic,
    EnergyBall,
    Stanley,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MissileShot {
    Miss,
    Destroyed,
    AcceptedImmune,
    Redirected,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClassicMissile {
    pub id: u64,
    pub kind: MissileKind,
    pub reflected: bool,
    // The key is required even though post-impact Stanley stores explicit null.
    #[serde(deserialize_with = "Option::<u64>::deserialize")]
    pub target_id: Option<u64>,
    pub x: f64,
    pub y: f64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub vx: f64,
    pub vy: f64,
    pub immunity_ticks: u8,
    pub frame: u8,
    pub visual_variant: u8,
    /// Raw2 impact clears its reservation but keeps this Board-owned child.
    pub stanley_hit: bool,
    /// Raw0/1 diversion clock and source-derived terminal limit.
    pub diversion_clock: u16,
    pub diversion_limit: u16,
    pub diversion_x: i32,
    pub diversion_y: i32,
    pub movement_divisor: f64,
}

impl ClassicMissile {
    pub fn launch(id: u64, target_id: u64, x: i32, y: i32, visual_draw: u32) -> Self {
        Self {
            id,
            kind: MissileKind::Classic,
            reflected: false,
            target_id: Some(target_id),
            x: f64::from(x),
            y: f64::from(y),
            widget_x: x,
            widget_y: y,
            vx: 0.0,
            vy: 0.0,
            immunity_ticks: 15,
            frame: 1,
            visual_variant: (visual_draw % 4) as u8,
            stanley_hit: false,
            diversion_clock: 0,
            diversion_limit: 0,
            diversion_x: 0,
            diversion_y: 0,
            movement_divisor: 0.8,
        }
    }

    pub fn launch_energy(id: u64, target_id: u64, x: i32, y: i32, visual_draw: u32) -> Self {
        let mut missile = Self::launch(id, target_id, x, y, visual_draw);
        missile.kind = MissileKind::EnergyBall;
        missile.movement_divisor = 0.4;
        missile
    }

    pub fn launch_stanley(id: u64, target_id: u64, x: i32, y: i32, visual_draw: u32) -> Self {
        let mut missile = Self::launch(id, target_id, x, y, visual_draw);
        missile.kind = MissileKind::Stanley;
        missile.movement_divisor = 0.4;
        missile
    }

    pub fn shot(&self, x: i32, y: i32) -> bool {
        if self.kind == MissileKind::EnergyBall {
            return f64::from(x) > self.x + 10.0
                && f64::from(x) < self.x + 70.0
                && f64::from(y) > self.y + 10.0
                && f64::from(y) < self.y + 70.0;
        }
        let cx = self.x + 40.0;
        let cy = self.y + 40.0;
        self.immunity_ticks == 0
            && f64::from(x) > cx - 30.0
            && f64::from(x) < cx + 30.0
            && f64::from(y) > cy - 30.0
            && f64::from(y) < cy + 30.0
    }

    /// PB48: accepted energy-ball shots do not dissolve its target relation.
    pub fn try_shot(&mut self, x: i32, y: i32) -> MissileShot {
        if !self.shot(x, y) {
            return MissileShot::Miss;
        }
        if self.kind != MissileKind::EnergyBall {
            return MissileShot::Destroyed;
        }
        if self.immunity_ticks != 0 {
            return MissileShot::AcceptedImmune;
        }
        self.vx = (f64::from(x) - self.x - 40.0) / -5.0;
        self.vy = (f64::from(y) - self.y - 40.0) / -5.0;
        for speed in [&mut self.vx, &mut self.vy] {
            if *speed >= 0.0 && *speed < 1.0 {
                *speed = 1.0;
            } else if *speed <= 0.0 && *speed > -1.0 {
                *speed = -1.0;
            }
        }
        self.reflected = true;
        MissileShot::Redirected
    }

    /// Checks contact at the old double position, then integrates and syncs.
    /// A missing target is a removal request, never an inferred replacement.
    // W1's guarded outer/inner bands are kept in their original first-match
    // order, even where neighboring outcomes adjust velocity equally.
    #[allow(clippy::if_same_then_else)]
    pub fn tick(&mut self, target: Option<MissilePreyView>) -> MissileUpdate {
        debug_assert_ne!(self.kind, MissileKind::Stanley);
        if self.advance_diversion_clock() {
            return MissileUpdate {
                remove: true,
                ..MissileUpdate::default()
            };
        }
        let Some(target) = target.filter(|view| Some(view.id) == self.target_id) else {
            return MissileUpdate {
                remove: true,
                ..MissileUpdate::default()
            };
        };
        if self.kind == MissileKind::EnergyBall && self.reflected {
            if self.x > 580.0 || self.x < -20.0 || self.y > 380.0 || self.y < 45.0 {
                return MissileUpdate {
                    remove: true,
                    ..MissileUpdate::default()
                };
            }
            self.x += self.vx / 0.4;
            self.y += self.vy / 0.4;
            self.immunity_ticks = self.immunity_ticks.saturating_sub(1);
            self.widget_x = self.x as i32;
            self.widget_y = self.y as i32;
            self.frame = (self.frame + 1) % 5;
            return MissileUpdate::default();
        }
        let cx = self.x + 40.0;
        let cy = self.y + 40.0;
        let tx = f64::from(target.widget_x);
        let ty = f64::from(target.widget_y);
        if cx > tx + 10.0 && cx < tx + 70.0 && cy > ty + 10.0 && cy < ty + 70.0 {
            return MissileUpdate {
                impact_target: Some(target.id),
                remove: true,
            };
        }
        if tx + 80.0 < cx && self.vx > -1.8 {
            self.vx -= 0.2;
        } else if tx > cx && self.vx < 1.8 {
            self.vx += 0.2;
        } else if tx + 60.0 < cx && self.vx > -1.4 {
            self.vx -= 0.2;
        } else if tx + 20.0 > cx && self.vx < 1.4 {
            self.vx += 0.2;
        } else if tx + 40.0 > cx && self.vx < 0.6 {
            self.vx += 0.2;
        } else if tx + 40.0 < cx && self.vx > -0.6 {
            self.vx -= 0.2;
        }
        if ty + 80.0 < cy && self.vy > -1.8 {
            self.vy -= 0.2;
        } else if ty > cy && self.vy < 1.8 {
            self.vy += 0.2;
        } else if ty + 60.0 < cy && self.vy > -1.4 {
            self.vy -= 0.2;
        } else if ty + 20.0 > cy && self.vy < 1.4 {
            self.vy += 0.2;
        } else if ty + 40.0 > cy && self.vy < 0.2 {
            self.vy += 0.1;
        } else if ty + 40.0 < cy && self.vy > -0.2 {
            self.vy -= 0.1;
        }
        let divisor = self.movement_divisor;
        self.x = self.x.clamp(10.0, 550.0) + self.vx / divisor;
        self.y = self.y.clamp(95.0, 370.0) + self.vy / divisor;
        self.immunity_ticks = self.immunity_ticks.saturating_sub(1);
        self.widget_x = self.x as i32;
        self.widget_y = self.y as i32;
        if self.kind == MissileKind::EnergyBall {
            self.frame = (self.frame + 1) % 5;
            return MissileUpdate::default();
        }
        self.frame = if self.vx > -0.8 && self.vx < 0.8 {
            if self.vy <= 0.0 { 4 } else { 12 }
        } else if self.vx <= 0.0 {
            if self.vy <= -2.5 {
                4
            } else if self.vy > 2.5 {
                12
            } else if self.vy > 2.0 {
                11
            } else if self.vy < -2.0 {
                5
            } else if self.vy > 1.5 {
                10
            } else if self.vy < -1.5 {
                6
            } else if self.vy > 1.0 {
                9
            } else if self.vy < -1.0 {
                7
            } else {
                8
            }
        } else if self.vy <= -2.5 {
            4
        } else if self.vy > 2.5 {
            12
        } else if self.vy > 2.0 {
            13
        } else if self.vy < -2.0 {
            3
        } else if self.vy > 1.5 {
            14
        } else if self.vy < -1.5 {
            2
        } else if self.vy > 1.0 {
            15
        } else if self.vy < -1.0 {
            1
        } else {
            0
        };
        MissileUpdate::default()
    }

    /// The selected raw0/1 child stays reserved to its original prey until
    /// this positive clock reaches its endpoint on a later active update.
    pub fn divert_from_stanley(&mut self, endpoint_x: i32, endpoint_y: i32) {
        debug_assert!(matches!(
            self.kind,
            MissileKind::Classic | MissileKind::EnergyBall
        ));
        let dx = i64::from(endpoint_x) - i64::from(self.widget_x + 40);
        let dy = i64::from(endpoint_y) - i64::from(self.widget_y + 40);
        let squared = dx * dx + dy * dy;
        self.diversion_x = endpoint_x;
        self.diversion_y = endpoint_y;
        self.diversion_clock = 1;
        self.diversion_limit = (((squared as f64).sqrt() as f32 as f64) / 16.0).floor() as u16;
    }

    fn advance_diversion_clock(&mut self) -> bool {
        if self.diversion_clock == 0 {
            return false;
        }
        self.diversion_clock = self.diversion_clock.saturating_add(1);
        self.diversion_clock >= self.diversion_limit
    }

    /// Contact and backlink clearing precede Board-owned damage callbacks.
    /// Board calls `finish_stanley_tick` afterward, so a lethal hit still has
    /// its source post-hit motion in the same active update.
    pub fn begin_stanley_tick(
        &mut self,
        target: Option<StanleyTargetView>,
        bilaterus_present: bool,
        threats_present: bool,
    ) -> MissileUpdate {
        debug_assert_eq!(self.kind, MissileKind::Stanley);
        if self.stanley_hit {
            return MissileUpdate::default();
        }
        let Some(target) = target.filter(|view| Some(view.id) == self.target_id) else {
            return MissileUpdate {
                remove: true,
                ..MissileUpdate::default()
            };
        };
        if !bilaterus_present && !threats_present {
            return MissileUpdate::default();
        }
        if target.kind_six || !bilaterus_present {
            let lead = if !bilaterus_present { 25.0 } else { 40.0 };
            Self::steer_axis(
                &mut self.vx,
                self.x + lead - f64::from(target.widget_x),
                [-40.0, 50.0, 80.0, 110.0, 200.0],
                0.2,
            );
            Self::steer_axis(
                &mut self.vy,
                self.y + lead - f64::from(target.widget_y),
                [-40.0, 50.0, 80.0, 110.0, 200.0],
                0.1,
            );
        } else {
            Self::steer_axis(
                &mut self.vx,
                self.x + 40.0 - f64::from(target.widget_x),
                [0.0, 20.0, 40.0, 60.0, 80.0],
                0.2,
            );
            Self::steer_axis(
                &mut self.vy,
                self.y + 40.0 - f64::from(target.widget_y),
                [0.0, 20.0, 40.0, 60.0, 80.0],
                0.1,
            );
        }
        let center_x = self.x + 25.0;
        let center_y = self.y + 25.0;
        let (near, far) = if target.group { (10, 70) } else { (20, 140) };
        if center_x > f64::from(target.widget_x + near)
            && center_x < f64::from(target.widget_x + far)
            && center_y > f64::from(target.widget_y + near)
            && center_y < f64::from(target.widget_y + far)
        {
            self.stanley_hit = true;
            self.target_id = None;
            self.vx = -self.vx;
            self.vy = -self.vy;
            self.movement_divisor = 1.5;
            return MissileUpdate {
                impact_target: Some(target.id),
                remove: false,
            };
        }
        MissileUpdate::default()
    }

    fn steer_axis(velocity: &mut f64, offset: f64, bounds: [f64; 5], middle_step: f64) {
        let [outer_low, inner_low, middle, inner_high, outer_high] = bounds;
        let middle_positive_bound = if middle_step == 0.1 { 0.2 } else { 0.6 };
        let middle_negative_bound = -middle_positive_bound;
        if offset < outer_low {
            if *velocity < 1.8 {
                *velocity += 0.2;
            }
        } else if offset < inner_low {
            if *velocity < 1.4 {
                *velocity += 0.2;
            }
        } else if offset < middle {
            if *velocity < middle_positive_bound {
                *velocity += middle_step;
            }
        } else if offset > outer_high {
            if *velocity > -1.8 {
                *velocity -= 0.2;
            }
        } else if offset > inner_high {
            if *velocity > -1.4 {
                *velocity -= 0.2;
            }
        } else if offset > middle && *velocity > middle_negative_bound {
            *velocity -= middle_step;
        }
    }

    /// Bounds follow contact/damage callbacks, before gravity and integration.
    pub fn finish_stanley_tick(&mut self) -> bool {
        debug_assert_eq!(self.kind, MissileKind::Stanley);
        if self.x < -20.0 || self.x > 580.0 || self.y < 45.0 || self.y > 380.0 {
            return true;
        }
        if self.stanley_hit {
            self.vy += 0.2;
        }
        self.x += self.vx / self.movement_divisor;
        self.y += self.vy / self.movement_divisor;
        self.immunity_ticks = self.immunity_ticks.saturating_sub(1);
        self.widget_x = self.x as i32;
        self.widget_y = self.y as i32;
        self.frame = (self.frame + 1) % 16;
        false
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0
            || self.target_id == Some(0)
            || self.target_id == Some(self.id)
            || (self.kind == MissileKind::Stanley
                && (self.stanley_hit != self.target_id.is_none() || self.diversion_clock != 0))
            || (self.kind != MissileKind::Stanley && (self.target_id.is_none() || self.stanley_hit))
            || (self.diversion_clock == 0
                && (self.diversion_limit != 0 || self.diversion_x != 0 || self.diversion_y != 0))
            || (self.diversion_clock != 0
                && (self.kind == MissileKind::Stanley
                    || !(-100..=700).contains(&self.diversion_x)
                    || !(-100..=600).contains(&self.diversion_y)))
            || ![self.x, self.y, self.vx, self.vy, self.movement_divisor]
                .into_iter()
                .all(f64::is_finite)
            || !(-100.0..=620.0).contains(&self.x)
            || !(0.0..=450.0).contains(&self.y)
            || self.widget_x != self.x as i32
            || self.widget_y != self.y as i32
            || self.vx.abs()
                > (if self.kind == MissileKind::Stanley {
                    100.0
                } else if self.kind == MissileKind::EnergyBall {
                    10.0
                } else {
                    3.0
                })
            || self.vy.abs()
                > (if self.kind == MissileKind::Stanley {
                    100.0
                } else if self.kind == MissileKind::EnergyBall {
                    10.0
                } else {
                    3.0
                })
            || self.movement_divisor
                != (match self.kind {
                    MissileKind::Classic => 0.8,
                    MissileKind::EnergyBall => 0.4,
                    MissileKind::Stanley if self.stanley_hit => 1.5,
                    MissileKind::Stanley => 0.4,
                })
            || self.immunity_ticks > 15
            || self.frame
                > (if self.kind == MissileKind::EnergyBall {
                    4
                } else {
                    15
                })
            || self.visual_variant > 3
            || (self.kind != MissileKind::EnergyBall && self.reflected)
            || (self.reflected && self.immunity_ticks > 0)
        {
            return Err("invalid classic missile state".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_shot_edges_wait_for_fifteen_active_updates() {
        let mut missile = ClassicMissile::launch(1, 2, 100, 100, 3);
        assert!(!missile.shot(140, 140));
        for _ in 0..15 {
            missile.tick(Some(MissilePreyView {
                id: 2,
                widget_x: 500,
                widget_y: 300,
            }));
        }
        let cx = missile.x + 40.0;
        let cy = missile.y + 40.0;
        assert!(missile.shot(cx as i32, cy as i32));
        assert!(!missile.shot((cx - 30.0).ceil() as i32 - 1, cy as i32));
    }

    #[test]
    fn contact_precedes_motion_and_missing_target_never_retargets() {
        let mut missile = ClassicMissile::launch(1, 2, 100, 100, 0);
        assert_eq!(
            missile.tick(None),
            MissileUpdate {
                impact_target: None,
                remove: true
            }
        );
        assert_eq!((missile.x, missile.y), (100.0, 100.0));
        let strict_edge = MissilePreyView {
            id: 2,
            widget_x: 130,
            widget_y: 130,
        };
        assert_eq!(missile.tick(Some(strict_edge)), MissileUpdate::default());
        let mut missile = ClassicMissile::launch(1, 2, 100, 100, 0);
        let target = MissilePreyView {
            id: 2,
            widget_x: 125,
            widget_y: 125,
        };
        assert_eq!(
            missile.tick(Some(target)),
            MissileUpdate {
                impact_target: Some(2),
                remove: true
            }
        );
        assert_eq!((missile.x, missile.y), (100.0, 100.0));
    }

    #[test]
    fn energy_ball_accepts_immune_shot_then_redirects_without_losing_reservation() {
        let mut ball = ClassicMissile::launch_energy(9, 4, 100, 100, 2);
        assert_eq!(ball.try_shot(110, 140), MissileShot::Miss); // strict edge
        ball.immunity_ticks = 1;
        assert_eq!(ball.try_shot(140, 140), MissileShot::AcceptedImmune);
        assert_eq!(
            (ball.target_id, ball.reflected, ball.vx),
            (Some(4), false, 0.0)
        );
        ball.immunity_ticks = 0;
        assert_eq!(ball.try_shot(140, 140), MissileShot::Redirected);
        assert_eq!(
            (ball.target_id, ball.reflected, ball.vx, ball.vy),
            (Some(4), true, 1.0, 1.0)
        );
        ball.validate().unwrap();
    }

    #[test]
    fn reflected_energy_ball_moves_independently_but_still_requires_its_target() {
        let mut ball = ClassicMissile::launch_energy(9, 4, 100, 100, 0);
        ball.immunity_ticks = 0;
        ball.try_shot(140, 140);
        assert_eq!(
            ball.tick(Some(MissilePreyView {
                id: 4,
                widget_x: 500,
                widget_y: 250
            })),
            MissileUpdate::default()
        );
        assert_eq!((ball.x, ball.y, ball.frame), (102.5, 102.5, 2));
        assert!(ball.tick(None).remove);
    }

    #[test]
    fn stanley_contact_clears_reservation_before_rebound_and_keeps_child_registered() {
        let mut missile = ClassicMissile::launch_stanley(9, 4, 110, 100, 2);
        let target = StanleyTargetView {
            id: 4,
            widget_x: 120,
            widget_y: 100,
            group: true,
            kind_six: false,
        };
        assert_eq!(
            missile
                .begin_stanley_tick(Some(target), true, true)
                .impact_target,
            Some(4)
        );
        // The +0.2 steering at offset30 precedes contact and is reversed.
        assert_eq!(
            (missile.target_id, missile.stanley_hit, missile.vx),
            (None, true, -0.2)
        );
        missile.finish_stanley_tick();
        assert!((missile.x - (110.0 - 0.2 / 1.5)).abs() < 0.0001);
        assert!((missile.y - (100.0 + 0.2 / 1.5)).abs() < 0.0001);
        missile.validate().unwrap();
    }

    #[test]
    fn stanley_strict_premotion_bounds_and_contact_edges() {
        let target = StanleyTargetView {
            id: 4,
            widget_x: 100,
            widget_y: 100,
            group: false,
            kind_six: false,
        };
        let mut missile = ClassicMissile::launch_stanley(9, 4, 95, 100, 0);
        // Center X=120 equals the strict near edge.
        assert_eq!(
            missile
                .begin_stanley_tick(Some(target), true, true)
                .impact_target,
            None
        );
        missile.x = 96.0;
        assert_eq!(
            missile
                .begin_stanley_tick(Some(target), true, true)
                .impact_target,
            Some(4)
        );
        let mut edge = ClassicMissile::launch_stanley(10, 4, 580, 380, 0);
        assert!(!edge.begin_stanley_tick(Some(target), true, true).remove);
        assert!(!edge.finish_stanley_tick());
        edge.x = 580.01;
        assert!(!edge.begin_stanley_tick(Some(target), true, true).remove);
        assert!(edge.finish_stanley_tick());
    }

    #[test]
    fn stanley_diversion_clock_reaches_float_rounded_distance_limit_before_motion() {
        let mut missile = ClassicMissile::launch(9, 4, 100, 100, 0);
        missile.divert_from_stanley(300, 140);
        assert_eq!((missile.diversion_clock, missile.diversion_limit), (1, 10));
        let origin = (missile.x, missile.y);
        for _ in 0..8 {
            assert!(!missile.advance_diversion_clock());
        }
        missile.validate().unwrap();
        assert!(missile.advance_diversion_clock());
        assert_eq!((missile.x, missile.y), origin);
    }

    #[test]
    fn stanley_kind_six_and_empty_group_use_special_steering_at_exact_forty() {
        let ordinary = StanleyTargetView {
            id: 4,
            widget_x: 100,
            widget_y: 300,
            group: false,
            kind_six: false,
        };
        let mut regular = ClassicMissile::launch_stanley(9, 4, 100, 100, 0);
        regular.begin_stanley_tick(Some(ordinary), true, true);
        assert_eq!(regular.vx, 0.0); // ordinary offset exactly 40
        let mut kind_six = ClassicMissile::launch_stanley(10, 4, 100, 100, 0);
        kind_six.begin_stanley_tick(
            Some(StanleyTargetView {
                kind_six: true,
                ..ordinary
            }),
            true,
            true,
        );
        assert_eq!(kind_six.vx, 0.2); // special bands, lead 40
        let mut empty_group = ClassicMissile::launch_stanley(11, 4, 100, 100, 0);
        empty_group.begin_stanley_tick(Some(ordinary), false, true);
        assert_eq!(empty_group.vx, 0.2); // helper true, lead 25
        let mut denied = ClassicMissile::launch_stanley(12, 4, 100, 100, 0);
        denied.begin_stanley_tick(Some(ordinary), false, false);
        assert_eq!(denied.vx, 0.0);
    }
}
