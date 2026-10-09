//! Ordinary Rhubarb pet, with source-derived chase and specialty timing from
//! pinned WinFish `OtherTypePet.cpp` f919b3c. Board owns prey writes and RNG.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
pub struct RhubarbPrey {
    pub id: u64,
    pub widget_x: i32,
    pub widget_y: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RhubarbContact {
    /// In Board category and list order, not merely the chase target.
    pub pushed_ids: Vec<u64>,
    pub specialty_started: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RhubarbState {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub vx: f64,
    pub vy: f64,
    pub target_vx: f64,
    pub previous_vx: f64,
    pub movement_state: u8,
    pub movement_timer: u8,
    pub chase_timer: u16,
    pub movement_animation_timer: u8,
    pub turn_ticks: i8,
    pub specialty_ticks: u8,
    pub frame: u8,
}

impl RhubarbState {
    /// Board::SpawnPet consumes x%265+105,y%520+20 before this constructor.
    /// Ordinary Rhubarb overwrites the consumed Y with 355. The unused second
    /// seed draw is retained for source RNG order but not persisted.
    pub fn spawn_tank4(
        id: u64,
        x: i32,
        _discarded_y: i32,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> Self {
        let movement_state = rand_range(10) as u8;
        let _unused_specialty_seed = rand_range(250) + 250;
        Self {
            id,
            x: f64::from(x),
            y: 355.0,
            widget_x: x,
            widget_y: 355,
            vx: 0.0,
            vy: 0.0,
            target_vx: 0.0,
            previous_vx: 1.0,
            movement_state,
            movement_timer: 0,
            chase_timer: 40,
            movement_animation_timer: 0,
            turn_ticks: 0,
            specialty_ticks: 0,
            frame: 0,
        }
    }

    pub fn sprite_row(&self) -> u8 {
        u8::from(self.specialty_ticks != 0)
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
            || !(0.0..=570.0).contains(&self.x)
            || !(90.0..=380.0).contains(&self.y)
            || self.widget_x != self.x as i32
            || self.widget_y != self.y as i32
            || self.movement_state > 9
            || self.movement_timer > 20
            || self.movement_animation_timer >= 40
            || !(-20..=20).contains(&self.turn_ticks)
            || self.specialty_ticks > 19
            || self.frame >= 10
        {
            return Err("invalid ordinary Rhubarb state".into());
        }
        Ok(())
    }

    /// Runs before specialty animation. Nearest target only steers horizontal
    /// chase; contact always scans every supported category/list member.
    pub fn begin_tick(&mut self, prey: &[RhubarbPrey], aliens_registered: bool) -> RhubarbContact {
        let mut result = RhubarbContact::default();
        let selection_y = if aliens_registered { 250 } else { 260 };
        let effect_y = if aliens_registered { 260 } else { 270 };
        let cx = (self.x + 40.0) as i32;
        let cy = (self.y + 40.0) as i32;
        let nearest = prey
            .iter()
            .filter(|candidate| candidate.widget_y > selection_y)
            .min_by_key(|candidate| {
                let dx = cx - candidate.widget_x - 40;
                let dy = cy - candidate.widget_y - 40;
                dx * dx + dy * dy
            });
        if let Some(target) = nearest {
            if self.chase_timer > 4 {
                self.chase_timer = 0;
                let tx = f64::from(target.widget_x + 40);
                if self.x + 40.0 > tx && self.vx > -5.0 {
                    self.vx -= 1.8;
                } else if self.x + 40.0 < tx && self.vx < 5.0 {
                    self.vx += 1.8;
                }
            }
            for candidate in prey {
                if candidate.widget_y <= effect_y {
                    continue;
                }
                if self.specialty_ticks == 5
                    && self.x + 40.0 > f64::from(candidate.widget_x)
                    && self.x + 40.0 < f64::from(candidate.widget_x + 80)
                {
                    result.pushed_ids.push(candidate.id);
                }
                if self.specialty_ticks == 0
                    && self.x + 40.0 > f64::from(candidate.widget_x - 10)
                    && self.x + 40.0 < f64::from(candidate.widget_x + 90)
                {
                    self.specialty_ticks = 20;
                    result.specialty_started = true;
                }
            }
        }
        result
    }

    /// Board applies each reported push with its own `%2` draw before this
    /// phase consumes movement-state RNG or decrements the specialty clock.
    pub fn finish_tick(&mut self, rand_range: &mut impl FnMut(u64) -> u64) {
        self.target_vx = match self.movement_state {
            0 => 0.0,
            1 => -0.5,
            2 => 0.5,
            3 => -1.0,
            4 => 1.0,
            5 => 1.5,
            6 => -1.5,
            7 => -2.5,
            8 => 2.5,
            _ => self.target_vx,
        };
        if self.target_vx < self.vx {
            self.vx -= 0.1;
        }
        if self.target_vx > self.vx {
            self.vx += 0.1;
        }
        self.movement_timer += 1;
        self.chase_timer = self.chase_timer.saturating_add(1);
        if self.movement_timer > 20
            || (self.x <= 10.0 && self.target_vx <= 0.0)
            || (self.x >= 540.0 && self.y >= 0.0)
        {
            self.movement_timer = 0;
            if rand_range(10) == 0 {
                self.movement_state = rand_range(9) as u8;
            }
        }
        if self.specialty_ticks != 0 {
            self.specialty_ticks -= 1;
            self.frame = self.specialty_ticks / 2;
        } else {
            if self.vx >= 1.0 {
                self.movement_animation_timer = (self.movement_animation_timer + 38) % 40;
            } else if self.vx <= -1.0 {
                self.movement_animation_timer = (self.movement_animation_timer + 2) % 40;
            } else if self.vx > 0.0 {
                self.movement_animation_timer = (self.movement_animation_timer + 39) % 40;
            } else {
                self.movement_animation_timer = (self.movement_animation_timer + 1) % 40;
            }
            self.frame = self.movement_animation_timer / 4;
        }
        self.x = self.x.clamp(10.0, 540.0);
        if self.y > 355.0 {
            self.y = 355.0;
            self.vy = 0.0;
        }
        self.y = self.y.max(95.0);
        if self.x > 535.0 && self.vx > 0.1 {
            self.movement_state = 1;
        }
        if self.x < 15.0 && self.vx < -0.1 {
            self.movement_state = 2;
        }
        self.x += self.vx / 2.0;
        self.y += self.vy / 2.0;
        self.widget_x = self.x as i32;
        self.widget_y = self.y as i32;
        self.previous_vx = self.vx;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_selection_does_not_limit_multi_target_push() {
        let mut pet = RhubarbState::spawn_tank4(1, 200, 20, &mut |_| 0);
        let prey = [
            RhubarbPrey {
                id: 2,
                widget_x: 190,
                widget_y: 300,
            },
            RhubarbPrey {
                id: 3,
                widget_x: 201,
                widget_y: 301,
            },
        ];
        let arm = pet.begin_tick(&prey, false);
        assert!(arm.specialty_started);
        assert!(arm.pushed_ids.is_empty());
        pet.finish_tick(&mut |_| 1);
        assert_eq!(pet.specialty_ticks, 19);
        pet.specialty_ticks = 5;
        let hit = pet.begin_tick(&prey, false);
        assert_eq!(hit.pushed_ids, [2, 3]);
        pet.finish_tick(&mut |_| 1);
        assert_eq!(pet.specialty_ticks, 4);
    }

    #[test]
    fn selection_and_effect_y_caps_are_distinct_and_strict() {
        let mut pet = RhubarbState::spawn_tank4(1, 200, 20, &mut |_| 0);
        pet.specialty_ticks = 5;
        assert!(
            pet.begin_tick(
                &[RhubarbPrey {
                    id: 2,
                    widget_x: 200,
                    widget_y: 260
                }],
                false
            )
            .pushed_ids
            .is_empty()
        );
        assert!(
            pet.begin_tick(
                &[RhubarbPrey {
                    id: 2,
                    widget_x: 200,
                    widget_y: 270
                }],
                false
            )
            .pushed_ids
            .is_empty()
        );
        assert_eq!(
            pet.begin_tick(
                &[RhubarbPrey {
                    id: 2,
                    widget_x: 200,
                    widget_y: 271
                }],
                false
            )
            .pushed_ids,
            [2]
        );
    }
}
