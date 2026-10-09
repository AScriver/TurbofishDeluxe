//! Rufus, the ordinary Tank 2 OtherTypePet. Board owns alien health and sound.
//! Motion/contact derive from pinned W1 OtherTypePet.cpp; PB34 corrects the
//! nearest-candidate metric to the installed pet+40 versus alien-widget+36.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RufusAlienView {
    pub id: u64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub healing: bool,
    /// A registered group is centered and contacted through its own 80px
    /// widget, rather than the ordinary Alien's 160px contact rectangle.
    pub bilaterus: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RufusUpdate {
    pub damaged_alien: Option<u64>,
    pub punch_requested: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RufusState {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub vx: f64,
    pub vy: f64,
    pub target_vx: f64,
    pub previous_vx: f64,
    pub chase_ticks: u32,
    pub movement_ticks: u8,
    pub movement_state: u8,
    pub animation_ticks: u8,
    pub frame: u8,
    pub ancillary_ticks: u16,
}

impl RufusState {
    /// Caller consumes the Board X/Y draws first, even though ordinary Rufus
    /// fixes Y=365; then the movement and ancillary draws follow.
    pub fn spawn_tank2(id: u64, rand_range: &mut impl FnMut(u64) -> u64) -> Self {
        let x = (rand_range(265) + 105) as i32;
        let _y = rand_range(520) + 20;
        let movement_state = rand_range(10) as u8;
        let ancillary_ticks = (rand_range(250) + 250) as u16;
        Self {
            id,
            x: f64::from(x),
            y: 365.0,
            widget_x: x,
            widget_y: 365,
            vx: 0.0,
            vy: 0.0,
            target_vx: 0.0,
            previous_vx: 1.0,
            chase_ticks: 40,
            movement_ticks: 0,
            movement_state,
            animation_ticks: 0,
            frame: 0,
            ancillary_ticks,
        }
    }

    fn nearest<'a>(&self, aliens: &'a [RufusAlienView]) -> Option<&'a RufusAlienView> {
        let mut best = 100_000_000_i64;
        let mut chosen = None;
        for alien in aliens.iter().filter(|alien| !alien.healing) {
            let center_offset = if alien.bilaterus { 40 } else { 36 };
            let dx = ((self.x + 40.0) - f64::from(alien.widget_x + center_offset)) as i64;
            let dy = ((self.y + 40.0) - f64::from(alien.widget_y + center_offset)) as i64;
            let d2 = dx * dx + dy * dy;
            if d2 < best {
                best = d2;
                chosen = Some(alien);
            }
        }
        chosen
    }

    pub fn tick(
        &mut self,
        aliens: &[RufusAlienView],
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> RufusUpdate {
        let mut update = RufusUpdate::default();
        let target = self.nearest(aliens);
        if let Some(target) = target {
            if self.chase_ticks > 4 {
                self.chase_ticks = 0;
                // Group steering uses its 80px center; ordinary Alien
                // steering retains the source's +80 target coordinate.
                let center = self.x + 40.0;
                let tx = f64::from(target.widget_x + if target.bilaterus { 40 } else { 80 });
                if center > tx && self.vx > -5.0 {
                    self.vx -= 1.8;
                } else if center < tx && self.vx < 5.0 {
                    self.vx += 1.8;
                }
            }
            if let Some(contact) = aliens.iter().find(|alien| {
                let overlaps = if alien.bilaterus {
                    (self.x - f64::from(alien.widget_x)).abs() < 10.0
                        && (self.y - f64::from(alien.widget_y)).abs() < 10.0
                } else {
                    self.x + 40.0 > f64::from(alien.widget_x + 30)
                        && self.x + 40.0 < f64::from(alien.widget_x + 140)
                        && self.y + 40.0 > f64::from(alien.widget_y + 10)
                        && self.y + 40.0 < f64::from(alien.widget_y + 150)
                };
                !alien.healing && overlaps
            }) {
                self.vx = 0.0;
                self.target_vx = 0.0;
                update.damaged_alien = Some(contact.id);
                update.punch_requested = true;
            }
        } else {
            self.target_vx = match self.movement_state {
                0 | 5 | 6 => 0.0,
                1 => -0.5,
                2 => 0.5,
                3 => -1.0,
                4 => 1.0,
                7 => -2.5,
                8 => 2.5,
                _ => self.target_vx,
            };
        }
        // W1 uses two comparisons, retaining small overshoot.
        if self.target_vx < self.vx {
            self.vx -= 0.1;
        }
        if self.target_vx > self.vx {
            self.vx += 0.1;
        }
        self.movement_ticks += 1;
        self.chase_ticks = self.chase_ticks.saturating_add(1);
        if self.movement_ticks > 20
            || (self.x <= 10.0 && self.target_vx <= 0.0)
            || (self.x >= 540.0 && self.y >= 0.0)
        {
            self.movement_ticks = 0;
            if rand_range(10) == 0 {
                self.movement_state = rand_range(9) as u8;
            }
        }
        self.x = self.x.clamp(10.0, 560.0);
        self.y = self.y.clamp(95.0, 365.0);
        if self.x > 535.0 && self.vx > 0.1 {
            self.movement_state = 1;
        }
        if self.x < 15.0 && self.vx < -0.1 {
            self.movement_state = 2;
        }
        self.animate(!aliens.is_empty(), target.is_some());
        self.x += self.vx / 2.0;
        self.y += self.vy / 2.0;
        self.widget_x = self.x as i32;
        self.widget_y = self.y as i32;
        update
    }

    fn animate(&mut self, aliens_present: bool, has_target: bool) {
        if self.vx.abs() <= 0.01 && self.target_vx == 0.0 && !aliens_present {
            self.animation_ticks = (self.animation_ticks + 1) % 60;
            self.frame = match self.animation_ticks {
                0..=2 => self.animation_ticks / 2,
                3..=23 => 2,
                24..=32 => self.animation_ticks / 2 - 10,
                33..=53 => 7,
                _ => self.animation_ticks / 2 - 20,
            };
            return;
        }
        self.animation_ticks %= 40;
        if self.vx.abs() <= 0.01 && self.target_vx == 0.0 && has_target {
            self.animation_ticks = (self.animation_ticks + 4) % 40;
        } else {
            let step: i16 = if self.vx >= 1.0 {
                2
            } else if self.vx > 0.0 {
                1
            } else if self.vx > -1.0 {
                -1
            } else {
                -2
            };
            self.animation_ticks = (i16::from(self.animation_ticks) + step).rem_euclid(40) as u8;
        }
        self.frame = self.animation_ticks / 4;
    }

    pub fn sprite_row(&self) -> u8 {
        u8::from(self.vx.abs() <= 0.01 && self.target_vx == 0.0)
    }

    pub fn sprite_frame(&self) -> u8 {
        self.frame
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0
            || ![
                self.x,
                self.y,
                self.vx,
                self.vy,
                self.target_vx,
                self.previous_vx,
            ]
            .into_iter()
            .all(f64::is_finite)
            || !(0.0..=565.0).contains(&self.x)
            || !(90.0..=370.0).contains(&self.y)
            || self.widget_x != self.x as i32
            || self.widget_y != self.y as i32
            || self.vx.abs() > 7.0
            || self.vy != 0.0
            || self.target_vx.abs() > 2.5
            || self.movement_ticks > 20
            || self.movement_state > 9
            || self.animation_ticks >= 60
            || self.frame > 9
            || !(250..=499).contains(&self.ancillary_ticks)
        {
            return Err("invalid Rufus state".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corrected_nearest_metric_and_separate_steering_center() {
        let mut pet = RufusState::spawn_tank2(1, &mut |_| 0);
        pet.x = 100.5;
        pet.widget_x = 100;
        let candidates = [
            RufusAlienView {
                id: 2,
                widget_x: 100,
                widget_y: 100,
                healing: false,
                bilaterus: false,
            },
            RufusAlienView {
                id: 3,
                widget_x: 105,
                widget_y: 100,
                healing: false,
                bilaterus: false,
            },
        ];
        assert_eq!(pet.nearest(&candidates).unwrap().id, 3);
        pet.chase_ticks = 5;
        pet.tick(&candidates, &mut |_| 1);
        assert!(pet.vx > 0.0, "steering uses target widget+80");
    }

    #[test]
    fn strict_contact_can_damage_pending_health_without_preemptive_removal() {
        let mut pet = RufusState::spawn_tank2(1, &mut |_| 0);
        pet.x = 100.0;
        pet.y = 150.0;
        pet.widget_x = 100;
        pet.widget_y = 150;
        pet.chase_ticks = 0;
        let alien = RufusAlienView {
            id: 2,
            widget_x: 100,
            widget_y: 100,
            healing: false,
            bilaterus: false,
        };
        assert_eq!(pet.tick(&[alien], &mut |_| 1).damaged_alien, Some(2));
    }

    #[test]
    fn group_contact_is_strict_ten_from_group_widget_not_ordinary_wide_box() {
        let group = RufusAlienView {
            id: 88,
            widget_x: 100,
            widget_y: 100,
            healing: false,
            bilaterus: true,
        };
        let mut inside = RufusState::spawn_tank2(1, &mut |_| 0);
        inside.x = 109.9;
        inside.y = 109.9;
        inside.widget_x = 109;
        inside.widget_y = 109;
        inside.chase_ticks = 0;
        assert_eq!(inside.tick(&[group], &mut |_| 1).damaged_alien, Some(88));
        assert_eq!(inside.vx, 0.0);
        assert_eq!(inside.target_vx, 0.0);

        for (x, y) in [(110.0, 100.0), (100.0, 110.0)] {
            let mut edge = RufusState::spawn_tank2(2, &mut |_| 0);
            edge.x = x;
            edge.y = y;
            edge.widget_x = x as i32;
            edge.widget_y = y as i32;
            edge.chase_ticks = 0;
            assert_eq!(edge.tick(&[group], &mut |_| 1).damaged_alien, None);
            let ordinary = RufusAlienView {
                bilaterus: false,
                ..group
            };
            assert_eq!(edge.tick(&[ordinary], &mut |_| 1).damaged_alien, Some(88));
        }
    }
}
