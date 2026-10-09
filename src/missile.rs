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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MissileUpdate {
    pub impact_target: Option<u64>,
    pub remove: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClassicMissile {
    pub id: u64,
    pub target_id: u64,
    pub x: f64,
    pub y: f64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub vx: f64,
    pub vy: f64,
    pub immunity_ticks: u8,
    pub frame: u8,
    pub visual_variant: u8,
}

impl ClassicMissile {
    pub fn launch(id: u64, target_id: u64, x: i32, y: i32, visual_draw: u32) -> Self {
        Self {
            id,
            target_id,
            x: f64::from(x),
            y: f64::from(y),
            widget_x: x,
            widget_y: y,
            vx: 0.0,
            vy: 0.0,
            immunity_ticks: 15,
            frame: 1,
            visual_variant: (visual_draw % 4) as u8,
        }
    }

    pub fn shot(&self, x: i32, y: i32) -> bool {
        let cx = self.x + 40.0;
        let cy = self.y + 40.0;
        self.immunity_ticks == 0
            && f64::from(x) > cx - 30.0
            && f64::from(x) < cx + 30.0
            && f64::from(y) > cy - 30.0
            && f64::from(y) < cy + 30.0
    }

    /// Checks contact at the old double position, then integrates and syncs.
    /// A missing target is a removal request, never an inferred replacement.
    // W1's guarded outer/inner bands are kept in their original first-match
    // order, even where neighboring outcomes adjust velocity equally.
    #[allow(clippy::if_same_then_else)]
    pub fn tick(&mut self, target: Option<MissilePreyView>) -> MissileUpdate {
        let Some(target) = target.filter(|view| view.id == self.target_id) else {
            return MissileUpdate {
                remove: true,
                ..MissileUpdate::default()
            };
        };
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
        self.x = self.x.clamp(10.0, 550.0) + self.vx / 0.8;
        self.y = self.y.clamp(95.0, 370.0) + self.vy / 0.8;
        self.immunity_ticks = self.immunity_ticks.saturating_sub(1);
        self.widget_x = self.x as i32;
        self.widget_y = self.y as i32;
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

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0
            || self.target_id == 0
            || self.id == self.target_id
            || ![self.x, self.y, self.vx, self.vy]
                .into_iter()
                .all(f64::is_finite)
            || !(-100.0..=620.0).contains(&self.x)
            || !(0.0..=450.0).contains(&self.y)
            || self.widget_x != self.x as i32
            || self.widget_y != self.y as i32
            || self.vx.abs() > 3.0
            || self.vy.abs() > 3.0
            || self.immunity_ticks > 15
            || self.frame > 15
            || self.visual_variant > 3
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
}
