//! Ordinary Adventure fish-shaped pets, including Tank 3 Wadsworth.
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
    Meryl,
    Wadsworth,
    Seymour,
    Shrapnel,
    Gumbo,
    Blip,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WardFishView {
    pub widget_x: i32,
    pub widget_y: i32,
    pub small_or_medium: bool,
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
    /// Meryl's zero-value note is emitted at the prior integer widget.
    pub note_at: Option<(i32, i32)>,
    pub bomb_at: Option<(i32, i32)>,
    /// The board applies its shared eleven-update punch sound delay.
    pub punch_sound: bool,
    /// Wadsworth's active-state transition after the clock has advanced.
    pub ward_transition: Option<bool>,
    pub ward_bubbles: Option<[(i32, i32); 2]>,
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
    pub bomb_threshold: u16,
    pub glint_phase: f64,
    pub meryl_blink: bool,
    /// Independent of the protection clock: active with clock zero is legal.
    pub ward_active: bool,
    pub ward_timer: u16,
    /// Prior widget coordinates published before this pet's own movement.
    pub published_x: i32,
    pub published_y: i32,
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
            bomb_threshold: if kind == FishPetKind::Shrapnel {
                rand_range(20) as u16 + 633
            } else {
                0
            },
            glint_phase: 0.0,
            meryl_blink: true,
            ward_active: false,
            ward_timer: 0,
            published_x: widget_x,
            published_y: widget_y,
            speed_mod: if kind == FishPetKind::Wadsworth {
                4.0
            } else if kind == FishPetKind::Zorf {
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
                self.glint_phase,
            ]
            .into_iter()
            .all(f64::is_finite)
            || !(0.0..=550.0).contains(&self.x)
            || !(0.0..=550.0).contains(&self.y)
            || self.widget_x != self.x as i32
            || self.widget_y != self.y as i32
            || !(if self.kind == FishPetKind::Wadsworth {
                self.speed_mod == 4.0
            } else if self.kind == FishPetKind::Zorf {
                self.speed_mod == 3.0
            } else {
                [1.6, 1.8, 2.0].contains(&self.speed_mod)
            })
            || self.movement_state > 9
            || self.movement_timer > 20
            || !matches!(self.x_direction, -1 | 1)
            || self.vx_abs > 64
            || self.swim_counter
                > (if self.kind == FishPetKind::Seymour {
                    79
                } else if self.kind == FishPetKind::Shrapnel {
                    59
                } else {
                    19
                })
            || self.frame > 9
            || !(-19..=19).contains(&self.turn_ticks)
            || (self.kind == FishPetKind::Prego
                && (!matches!(self.birth_threshold, 930 | 1230 | 2000)
                    || self.birth_timer >= self.birth_threshold))
            || (self.kind != FishPetKind::Prego && self.birth_timer != 0)
            || (self.kind == FishPetKind::Zorf && self.food_timer < -10)
            || (self.kind != FishPetKind::Zorf && self.food_timer != 0)
            || (self.kind == FishPetKind::Vert && self.coin_timer >= 216)
            || (self.kind == FishPetKind::Meryl && self.coin_timer >= 1400)
            || (self.kind == FishPetKind::Shrapnel
                && (!(633..=652).contains(&self.bomb_threshold)
                    || self.coin_timer >= self.bomb_threshold
                    || !(-1.0..1.0).contains(&self.glint_phase)))
            || (self.kind == FishPetKind::Gumbo
                && (self.bomb_threshold != 0 || !(-1.0..1.0).contains(&self.glint_phase)))
            || (!matches!(self.kind, FishPetKind::Shrapnel | FishPetKind::Gumbo)
                && (self.bomb_threshold != 0 || self.glint_phase != 0.0))
            || (!matches!(
                self.kind,
                FishPetKind::Vert | FishPetKind::Meryl | FishPetKind::Shrapnel
            ) && self.coin_timer != 0)
            || (self.kind != FishPetKind::Meryl && !self.meryl_blink)
            || (self.kind == FishPetKind::Wadsworth
                && (self.ward_timer > 120
                    || !(0..=550).contains(&self.published_x)
                    || !(0..=550).contains(&self.published_y)))
            || (self.kind != FishPetKind::Wadsworth
                && (self.ward_active
                    || self.ward_timer != 0
                    || self.published_x != self.widget_x
                    || self.published_y != self.widget_y))
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
            FishPetKind::Meryl => {
                if self.turn_ticks != 0 {
                    2
                } else if self.coin_timer > 1300 {
                    1
                } else if self.frame < 5 && self.meryl_blink {
                    3
                } else {
                    0
                }
            }
            FishPetKind::Wadsworth => {
                if self.ward_timer > 0 {
                    2
                } else {
                    u8::from(self.turn_ticks != 0)
                }
            }
            FishPetKind::Seymour => u8::from(self.turn_ticks != 0),
            FishPetKind::Shrapnel => u8::from(self.turn_ticks != 0),
            FishPetKind::Gumbo => u8::from(self.turn_ticks != 0),
            FishPetKind::Blip => u8::from(self.turn_ticks != 0),
        }
    }

    /// Fish updates precede fish-pet updates, so this reflects the song for
    /// the next ordinary fish production call.
    pub fn song_active(&self) -> bool {
        self.kind == FishPetKind::Meryl && (1300..1400).contains(&self.coin_timer)
    }

    pub fn sprite_frame(&self) -> u8 {
        if self.kind == FishPetKind::Wadsworth && self.ward_timer > 0 {
            let timer = i32::from(self.ward_timer);
            let frame = if !self.ward_active && timer > 20 {
                9 - (29 - timer) / 2
            } else if self.ward_active && timer > 110 {
                9 - (120 - timer - 1) / 2
            } else if timer < 11 {
                9 - (timer - 1) / 2
            } else {
                5
            };
            frame.clamp(0, 9) as u8
        } else if self.kind == FishPetKind::Zorf && self.sprite_row(false) == 2 {
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

    pub fn shrapnel_flash_alpha(&self) -> u8 {
        if self.kind == FishPetKind::Shrapnel && self.coin_timer + 50 > self.bomb_threshold {
            (self.glint_phase.abs() * 255.0) as u8
        } else {
            0
        }
    }

    pub fn gumbo_light_alpha(&self) -> u8 {
        if self.kind == FishPetKind::Gumbo {
            (self.glint_phase.abs() * 255.0) as u8
        } else {
            0
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

    /// This subtype owns its protection clock. The Board supplies registered
    /// enemy/classic-missile membership and the ordered fish snapshot after
    /// all earlier object phases, then uses the published prior widget pose
    /// for the next fish/prey phase.
    pub fn tick_wadsworth(
        &mut self,
        threat_present: bool,
        fish: &[WardFishView],
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> FishPetUpdate {
        assert_eq!(self.kind, FishPetKind::Wadsworth);
        let mut ward = FishPetUpdate::default();
        if self.ward_timer > 0 {
            self.ward_timer -= 1;
        } else if self.ward_active
            && fish.iter().any(|view| {
                view.small_or_medium
                    && (self.x + 40.0 - f64::from(view.widget_x + 40)).abs() > 10.0
                    && (self.y + 40.0 - f64::from(view.widget_y + 40)).abs() > 10.0
            })
        {
            self.ward_timer = 100;
        }
        if threat_present {
            if !self.ward_active {
                self.ward_active = true;
                self.ward_timer = 120;
                ward.ward_transition = Some(true);
            }
            self.published_x = self.widget_x;
            self.published_y = self.widget_y;
        } else if self.ward_active {
            self.ward_active = false;
            self.ward_timer = 30;
            ward.ward_transition = Some(false);
            ward.ward_bubbles = Some([
                (self.widget_x + 11, self.widget_y + 5),
                (self.widget_x + 4, self.widget_y + 2),
            ]);
        }
        let _motion = self.tick_inner(&[], 0, &[], rand_range);
        ward
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
        if self.kind == FishPetKind::Gumbo && !aliens.is_empty() {
            self.hunt_gumbo(aliens);
        } else if hunting {
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
        if self.kind == FishPetKind::Meryl && aliens.is_empty() {
            self.coin_timer += 1;
            if self.coin_timer == 1300 {
                update.note_at = Some((self.widget_x + 15, self.widget_y - 5));
            } else if self.coin_timer >= 1400 {
                self.coin_timer = 0;
            }
        }
        if self.kind == FishPetKind::Shrapnel && aliens.is_empty() {
            self.coin_timer += 1;
            if self.coin_timer >= self.bomb_threshold {
                self.coin_timer = 0;
                update.bomb_at = Some((self.widget_x + 15, self.widget_y + 10));
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
        if self.kind == FishPetKind::Gumbo && !aliens.is_empty() && self.turn_ticks == 0 {
            self.glint_phase += 0.1;
            if self.glint_phase >= 1.0 {
                self.glint_phase = -1.0;
            }
        }
        // FishTypePet::Animate performs this only in its ordinary swim
        // branch, after decrementing the turn counter. A turn ending 1→0
        // enters that branch; turn ±19→±18 can display frame zero but must
        // not consume the blink RNG draw.
        if self.kind == FishPetKind::Meryl && self.turn_ticks == 0 && self.frame == 0 {
            if self.meryl_blink {
                self.meryl_blink = false;
            } else {
                self.meryl_blink = rand_range(10) == 0;
            }
        }
        self.x += self.vx / self.speed_mod;
        self.y += self.vy / self.speed_mod;
        self.widget_x = self.x as i32;
        self.widget_y = self.y as i32;
        if self.kind != FishPetKind::Wadsworth {
            self.published_x = self.widget_x;
            self.published_y = self.widget_y;
        }
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

    /// W1 FishTypePet::HungryBehavior selects the nearest non-healing alien,
    /// then flees its widget pose; this stage has no Bilaterus target.
    fn hunt_gumbo(&mut self, aliens: &[PetAlienView]) {
        let mut nearest = None;
        let mut best = i32::MAX;
        for alien in aliens.iter().filter(|alien| !alien.healing) {
            let dx = (self.x + 40.0 - f64::from(alien.widget_x + 80)) as i32;
            let dy = (self.y + 40.0 - f64::from(alien.widget_y + 80)) as i32;
            let distance = ((f64::from(dx * dx + dy * dy)).sqrt()) as i32;
            if distance < best {
                best = distance;
                nearest = Some(alien);
            }
        }
        if let Some(alien) = nearest.filter(|_| self.special_timer >= 5) {
            self.special_timer = 0;
            // Raw target type 0x17 takes +40; ordinary raw6 takes +80.
            let target_x = alien.widget_x + 80;
            let target_y = alien.widget_y + 80;
            if target_y > 260 && self.vy > -8.0 {
                self.vy -= 2.0;
            } else if target_y < 300 && self.vy < 8.0 {
                self.vy += 2.0;
            }
            if target_x > 290 && self.vx > -8.0 {
                self.vx -= 2.0;
            } else if target_x < 330 && self.vx < 8.0 {
                self.vx += 2.0;
            }
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
            self.swim_counter += if self.kind == FishPetKind::Zorf
                || self.kind == FishPetKind::Shrapnel && self.vx_abs < 3
                || self.vx_abs <= 1
            {
                1
            } else {
                2
            };
            if self.swim_counter
                > (if self.kind == FishPetKind::Seymour {
                    79
                } else if self.kind == FishPetKind::Shrapnel {
                    59
                } else {
                    19
                })
            {
                self.swim_counter = 0;
            }
            self.frame = if self.kind == FishPetKind::Vert {
                if self.swim_counter < 10 {
                    self.swim_counter
                } else {
                    19 - self.swim_counter
                }
            } else if self.kind == FishPetKind::Seymour {
                self.swim_counter / 8
            } else if self.kind == FishPetKind::Shrapnel {
                self.glint_phase += 0.1;
                if self.glint_phase >= 1.0 {
                    self.glint_phase = -1.0;
                }
                self.swim_counter / 6
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

    #[test]
    fn meryl_note_clock_freezes_on_registration_and_song_ends_at_1400() {
        let mut pet = actor(FishPetKind::Meryl);
        pet.coin_timer = 1299;
        let alien = [PetAlienView {
            id: 7,
            widget_x: 100,
            widget_y: 100,
            healing: false,
        }];
        assert_eq!(pet.tick(&alien, 0, &mut |_| 1).note_at, None);
        assert_eq!(pet.coin_timer, 1299);
        let old_widget = (pet.widget_x, pet.widget_y);
        assert_eq!(
            pet.tick(&[], 0, &mut |_| 1).note_at,
            Some((old_widget.0 + 15, old_widget.1 - 5))
        );
        assert!(pet.song_active());
        assert_ne!(pet.sprite_row(false), 1); // Draw row is strictly after 1300.
        pet.coin_timer = 1399;
        assert_eq!(pet.tick(&[], 0, &mut |_| 1).note_at, None);
        assert_eq!(pet.coin_timer, 0);
        assert!(!pet.song_active());
        pet.validate().unwrap();
    }

    #[test]
    fn meryl_blink_rng_is_only_in_post_decrement_swim_branch() {
        for turn in [19, -19] {
            let mut pet = actor(FishPetKind::Meryl);
            pet.turn_ticks = turn;
            pet.special_timer = 0;
            pet.vx = 0.0;
            pet.meryl_blink = false;
            pet.tick(&[], 0, &mut |_| {
                panic!("a frame-zero turn cannot roll blink")
            });
            assert_eq!(pet.frame, 0);
            assert!(!pet.meryl_blink);
        }
        let mut ending = actor(FishPetKind::Meryl);
        ending.turn_ticks = 1;
        ending.special_timer = 0;
        ending.swim_counter = 19;
        ending.vx = 0.0;
        ending.meryl_blink = false;
        let mut draws = 0;
        ending.tick(&[], 0, &mut |_| {
            draws += 1;
            0
        });
        assert_eq!((ending.turn_ticks, ending.frame, draws), (0, 0, 1));
        assert!(ending.meryl_blink);
        ending.swim_counter = 19;
        ending.tick(&[], 0, &mut |_| panic!("true blink clears without a draw"));
        assert!(!ending.meryl_blink);
    }

    fn actor(kind: FishPetKind) -> FishPetState {
        let mut rng = |_: u64| 0;
        FishPetState::spawn_tank1(1, kind, &mut rng)
    }

    #[test]
    fn shrapnel_sampled_interval_emits_prior_widget_bomb_and_freezes_under_invasion() {
        let mut requests = Vec::new();
        let mut pet = FishPetState::spawn_tank1(77, FishPetKind::Shrapnel, &mut |upper| {
            requests.push(upper);
            upper - 1
        });
        assert_eq!(pet.bomb_threshold, 652);
        assert_eq!(requests.last(), Some(&20));
        pet.coin_timer = 651;
        let alien = [PetAlienView {
            id: 9,
            widget_x: 400,
            widget_y: 280,
            healing: false,
        }];
        assert_eq!(pet.tick(&alien, 2, &mut |_| 1).bomb_at, None);
        assert_eq!(pet.coin_timer, 651);
        let origin = (pet.widget_x, pet.widget_y);
        assert_eq!(
            pet.tick(&[], 2, &mut |_| 1).bomb_at,
            Some((origin.0 + 15, origin.1 + 10))
        );
        assert_eq!(pet.coin_timer, 0);
        pet.validate().unwrap();
    }

    #[test]
    fn shrapnel_two_step_velocity_uses_slow_six_tick_swim_frame() {
        let mut pet = actor(FishPetKind::Shrapnel);
        pet.vx = 1.0;
        pet.previous_vx = 1.0;
        pet.vx_abs = 2;
        pet.swim_counter = 4;
        pet.animate();
        assert_eq!((pet.swim_counter, pet.frame), (5, 0));
        pet.swim_counter = 4;
        pet.vx_abs = 3;
        pet.animate();
        assert_eq!((pet.swim_counter, pet.frame), (6, 1));
    }

    #[test]
    fn wadsworth_clock_is_independent_of_active_and_publishes_pre_move_widget() {
        let mut pet = actor(FishPetKind::Wadsworth);
        pet.x = 100.0;
        pet.y = 200.0;
        pet.widget_x = 100;
        pet.widget_y = 200;
        let old = (pet.widget_x, pet.widget_y);
        let fish = [WardFishView {
            widget_x: 160,
            widget_y: 260,
            small_or_medium: true,
        }];
        let activated = pet.tick_wadsworth(true, &fish, &mut |_| 1);
        assert_eq!(activated.ward_transition, Some(true));
        assert_eq!((pet.ward_active, pet.ward_timer), (true, 120));
        assert_eq!((pet.published_x, pet.published_y), old);
        assert_eq!((pet.sprite_row(false), pet.sprite_frame()), (2, 9));
        pet.ward_timer = 1;
        pet.tick_wadsworth(true, &fish, &mut |_| 1);
        assert_eq!((pet.ward_active, pet.ward_timer), (true, 0));
        pet.validate().unwrap();
        pet.tick_wadsworth(true, &fish, &mut |_| 1);
        assert_eq!(pet.ward_timer, 100);
        let before_release = (pet.widget_x, pet.widget_y);
        let released = pet.tick_wadsworth(false, &fish, &mut |_| 1);
        assert_eq!(released.ward_transition, Some(false));
        assert_eq!((pet.ward_active, pet.ward_timer), (false, 30));
        assert_eq!(
            released.ward_bubbles,
            Some([
                (before_release.0 + 11, before_release.1 + 5),
                (before_release.0 + 4, before_release.1 + 2),
            ])
        );
        assert_eq!((pet.sprite_row(false), pet.sprite_frame()), (2, 9));
        pet.validate().unwrap();
    }

    #[test]
    fn wadsworth_reset_needs_both_axis_separations_and_small_or_medium() {
        let mut pet = actor(FishPetKind::Wadsworth);
        pet.ward_active = true;
        pet.ward_timer = 0;
        pet.x = 100.0;
        pet.y = 100.0;
        pet.widget_x = 100;
        pet.widget_y = 100;
        let only_x_far = [WardFishView {
            widget_x: 140,
            widget_y: 100,
            small_or_medium: true,
        }];
        pet.tick_wadsworth(true, &only_x_far, &mut |_| 1);
        assert_eq!(pet.ward_timer, 0);
        let large_far = [WardFishView {
            widget_x: 140,
            widget_y: 140,
            small_or_medium: false,
        }];
        pet.tick_wadsworth(true, &large_far, &mut |_| 1);
        assert_eq!(pet.ward_timer, 0);
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
    fn seymour_ordinary_swim_uses_eighty_tick_cycle_and_eight_tick_frames() {
        let mut pet = actor(FishPetKind::Seymour);
        pet.turn_ticks = 0;
        pet.swim_counter = 7;
        pet.vx_abs = 1;
        pet.vx = 1.0;
        pet.previous_vx = 1.0;
        pet.animate();
        assert_eq!((pet.swim_counter, pet.frame), (8, 1));
        pet.swim_counter = 79;
        pet.animate();
        assert_eq!((pet.swim_counter, pet.frame), (0, 0));
        pet.validate().unwrap();
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

    #[test]
    fn gumbo_flees_nearest_nonhealing_alien_with_source_overshoot() {
        let mut pet = actor(FishPetKind::Gumbo);
        pet.x = 200.0;
        pet.y = 200.0;
        pet.widget_x = 200;
        pet.widget_y = 200;
        pet.vx = -7.5;
        pet.vy = -7.5;
        pet.previous_vx = -1.0;
        pet.special_timer = 5;
        let aliens = [
            PetAlienView {
                id: 1,
                widget_x: 200,
                widget_y: 200,
                healing: true,
            },
            PetAlienView {
                id: 2,
                widget_x: 300,
                widget_y: 300,
                healing: false,
            },
        ];
        pet.tick(&aliens, 0, &mut |_| 1);
        assert_eq!((pet.vx, pet.vy), (-9.5, -9.5)); // guard precedes ±2 step
        assert!(pet.gumbo_light_alpha() > 0);
        pet.validate().unwrap();
    }
}
