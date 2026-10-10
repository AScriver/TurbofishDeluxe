//! Ordinary Adventure fish-shaped pets, including Tank 3 Wadsworth.
//! Behavioral rules are derived from pinned WinFish W1 `FishTypePet.cpp`,
//! `Fish.cpp`, and `Board.cpp` (revision f919b3c). Installed-binary coverage
//! for their full movement and animation remains partial. Board membership,
//! alien health, guppy construction, sounds, and effects belong to the caller.

use serde::{Deserialize, Serialize};

const GASH_MEAL_THRESHOLD: i32 = 1570;

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
    Nimbus,
    Amp,
    Gash,
    Angie,
    Presto,
}

/// Constructor-owned flag and recharge clock for a Presto-origin pet:
/// FishTypePet +238, OtherTypePet +1ac in PB05.
/// Plain pets have no form state; a transformed pet retains it at every form.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrestoForm {
    pub remaining_ticks: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrestoChangeEligibility {
    NotPrestoForm,
    SameForm,
    CoolingDown,
    Ready,
}

/// A direct Amp handler result. The Board owns whether normal input reaches it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AmpTap {
    NotReady,
    Charged(u8),
    Discharged,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WardFishView {
    pub widget_x: i32,
    pub widget_y: i32,
    pub small_or_medium: bool,
}

/// Board supplies only live ordinary Fish. Their installed class identity is
/// -1 (PB70/PB71); no other Fish variant is represented by Rust's Fish list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GashFishView {
    pub id: u64,
    pub widget_x: i32,
    pub widget_y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AngieCorpseView {
    pub id: u64,
    pub widget_x: i32,
    pub widget_y: i32,
    /// DeadFish lifetime +1a0, initially 125; eligibility is strictly >95.
    pub remaining_ticks: i32,
    /// Raw corpse subtype 6 uses the larger contact center and radius.
    pub ultra: bool,
    /// Board-owned revival timer +190; contact changes only 100 to 10.
    pub revival_ticks: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PetAlienView {
    pub id: u64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub healing: bool,
    /// A Bilaterus view names its active physical head, but retains the
    /// group's distinct +40 target/contact geometry.
    pub bilaterus: bool,
}

impl PetAlienView {
    fn center_offset(self) -> i32 {
        if self.bilaterus { 40 } else { 80 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NimbusCoinView {
    pub id: u64,
    pub widget_x: i32,
    pub widget_y: i32,
    /// Board owns raw-kind, Penta-membership, and collection filtering.
    pub eligible: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NimbusFoodView {
    pub id: u64,
    pub widget_x: i32,
    pub widget_y: i32,
    /// Board owns food-kind and collection filtering.
    pub eligible: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NimbusCoinRequest {
    pub coin_id: u64,
    pub x: i32,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NimbusFoodRequest {
    pub food_id: u64,
    pub x: i32,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FishPetUpdate {
    pub damaged_alien: Option<u64>,
    pub born_at: Option<(i32, i32)>,
    pub free_food: Option<ZorfFoodRequest>,
    /// Gold appears at the prior integer widget, before this pet moves.
    pub gold_at: Option<(i32, i32)>,
    /// Meryl's zero-value note is emitted at the prior integer widget.
    pub note_at: Option<(i32, i32)>,
    pub bomb_at: Option<(i32, i32)>,
    /// The Board applies eleven-update Itchy or nine-update Gash punch delay.
    pub punch_sound: bool,
    /// Wadsworth's active-state transition after the clock has advanced.
    pub ward_transition: Option<bool>,
    pub ward_bubbles: Option<[(i32, i32); 2]>,
    pub nimbus_coin: Option<NimbusCoinRequest>,
    pub nimbus_food: Option<NimbusFoodRequest>,
    /// The Board may play the ready sound after this source clock transition.
    pub amp_became_ready: bool,
    /// Board must commit removal, missile detachment and chomp before reset.
    pub gash_guppy_remove: Option<u64>,
    /// A touching eligible corpse whose Board-owned timer is exactly 100.
    pub angie_revive: Option<u64>,
    /// Gash contacts the Bilaterus list before the ordinary Alien list.
    pub gash_group_hit: Option<u64>,
}

#[derive(Clone, Copy)]
struct NimbusViews<'a> {
    coins: &'a [NimbusCoinView],
    foods: &'a [NimbusFoodView],
    enemies_registered: bool,
}

/// Only one pet subtype supplies an extra target list per update.
enum PetTargetViews<'a> {
    None,
    Tank5,
    Nimbus(NimbusViews<'a>),
    Gash(&'a [GashFishView]),
    Angie(&'a [AngieCorpseView]),
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
    /// PB65 uses signed DWORDs. These are independent of other pets' clocks.
    pub amp_timer: i32,
    pub amp_threshold: i32,
    pub amp_charge: u8,
    pub gash_timer: i32,
    pub gash_eating_ticks: u8,
    pub bomb_threshold: u16,
    pub glint_phase: f64,
    pub meryl_blink: bool,
    /// Independent of the protection clock: active with clock zero is legal.
    pub ward_active: bool,
    pub ward_timer: u16,
    /// The actor owns the source +238 cooldown and transformed-form flag.
    #[serde(default)]
    pub presto_form: Option<PrestoForm>,
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
    /// pre-integration clamp. Zorf/Nimbus overwrite the drawn speed divisor.
    pub fn spawn_tank1(
        id: u64,
        kind: FishPetKind,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> Self {
        Self::spawn(id, kind, false, false, None, rand_range)
    }

    /// Tank 5 uses ordinary Adventure constructor values. Its Board-specific
    /// update suppression is independent of the App's Virtual Tank mode.
    pub fn spawn_tank5(
        id: u64,
        kind: FishPetKind,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> Self {
        Self::spawn(id, kind, false, false, None, rand_range)
    }

    /// Construct a flagged FishTypePet replacement at the old widget pose.
    /// Board owns replacement/removal, identity, sound, and target admission.
    /// The common Fish constructor still consumes its six draws.
    pub fn spawn_presto_form_at(
        id: u64,
        kind: FishPetKind,
        widget_x: i32,
        widget_y: i32,
        virtual_tank: bool,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> Self {
        let mut pet = Self::spawn(
            id,
            kind,
            virtual_tank,
            true,
            Some((widget_x, widget_y)),
            rand_range,
        );
        pet.presto_form = Some(PrestoForm {
            remaining_ticks: if virtual_tank || kind == FishPetKind::Presto {
                0
            } else {
                360
            },
        });
        pet
    }

    /// Source 004f89a0 checks form equality before the +238 readiness gate.
    /// This query consumes no RNG and leaves the actor untouched.
    pub fn presto_change_eligibility(&self, target: FishPetKind) -> PrestoChangeEligibility {
        let Some(form) = self.presto_form else {
            return PrestoChangeEligibility::NotPrestoForm;
        };
        if target == self.kind {
            PrestoChangeEligibility::SameForm
        } else if form.remaining_ticks != 0 {
            PrestoChangeEligibility::CoolingDown
        } else {
            PrestoChangeEligibility::Ready
        }
    }

    fn spawn(
        id: u64,
        kind: FishPetKind,
        virtual_tank: bool,
        flagged: bool,
        at: Option<(i32, i32)>,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> Self {
        let (widget_x, widget_y) =
            at.unwrap_or_else(|| (rand_range(265) as i32 + 105, rand_range(520) as i32 + 20));
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
        // 004ef420's mode-5 check belongs to App Virtual Tank, not Board Tank 5.
        let _virtual_vert_interval = if virtual_tank && kind == FishPetKind::Vert {
            Some(rand_range(200))
        } else {
            None
        };
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
            amp_timer: if kind == FishPetKind::Amp { 300 } else { 0 },
            amp_threshold: if kind == FishPetKind::Amp { 3000 } else { 0 },
            amp_charge: 0,
            gash_timer: if kind == FishPetKind::Gash {
                if flagged {
                    1520
                } else if virtual_tank {
                    0
                } else {
                    -1550
                }
            } else {
                0
            },
            gash_eating_ticks: 0,
            bomb_threshold: if kind == FishPetKind::Shrapnel {
                if virtual_tank {
                    rand_range(200) as u16 + 1080
                } else {
                    rand_range(20) as u16 + 633
                }
            } else {
                0
            },
            glint_phase: 0.0,
            meryl_blink: true,
            ward_active: false,
            ward_timer: 0,
            presto_form: (kind == FishPetKind::Presto).then_some(PrestoForm { remaining_ticks: 0 }),
            published_x: widget_x,
            published_y: widget_y,
            speed_mod: if kind == FishPetKind::Wadsworth {
                4.0
            } else if kind == FishPetKind::Zorf {
                3.0
            } else if matches!(kind, FishPetKind::Nimbus | FishPetKind::Amp) {
                0.5
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
        self.validate_for_tank()
    }

    pub fn validate_tank5(&self) -> Result<(), String> {
        self.validate_for_tank()
    }

    fn validate_for_tank(&self) -> Result<(), String> {
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
            } else if self.kind == FishPetKind::Nimbus {
                matches!(self.speed_mod, 0.5 | 1.8)
            } else if self.kind == FishPetKind::Amp {
                self.speed_mod == 0.5
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
                } else if self.kind == FishPetKind::Nimbus {
                    39
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
            || (self.kind == FishPetKind::Amp && self.amp_charge > 2)
            || (self.kind != FishPetKind::Amp
                && (self.amp_timer != 0 || self.amp_threshold != 0 || self.amp_charge != 0))
            || (self.kind == FishPetKind::Gash && self.gash_eating_ticks > 10)
            || (self.kind != FishPetKind::Gash
                && (self.gash_timer != 0 || self.gash_eating_ticks != 0))
            || (matches!(self.kind, FishPetKind::Gumbo | FishPetKind::Angie)
                && (self.bomb_threshold != 0 || !(-1.0..1.0).contains(&self.glint_phase)))
            || (self.kind == FishPetKind::Amp && !(-1.0..1.0).contains(&self.glint_phase))
            || (!matches!(
                self.kind,
                FishPetKind::Shrapnel | FishPetKind::Gumbo | FishPetKind::Amp | FishPetKind::Angie
            ) && (self.bomb_threshold != 0 || self.glint_phase != 0.0))
            || (self.kind == FishPetKind::Amp && self.bomb_threshold != 0)
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
            || (self.kind == FishPetKind::Presto && self.presto_form.is_none())
            || self
                .presto_form
                .is_some_and(|form| form.remaining_ticks > 360)
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
            FishPetKind::Nimbus => u8::from(self.turn_ticks != 0),
            FishPetKind::Amp => u8::from(self.turn_ticks != 0),
            FishPetKind::Gash => {
                if self.turn_ticks != 0 {
                    1
                } else {
                    u8::from(self.gash_eating_ticks > 0) * 2
                }
            }
            FishPetKind::Angie => u8::from(self.turn_ticks != 0),
            FishPetKind::Presto => u8::from(self.turn_ticks != 0),
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

    pub fn amp_ready(&self) -> bool {
        self.kind == FishPetKind::Amp && self.amp_timer >= self.amp_threshold
    }

    /// W1's 160x80 widget excludes 25 pixels at its bottom. Visibility,
    /// overlaps, combat and modal routing remain Board/input responsibilities.
    pub fn amp_hitbox_contains(&self, x: i32, y: i32) -> bool {
        self.kind == FishPetKind::Amp
            && (self.widget_x..self.widget_x + 160).contains(&x)
            && (self.widget_y..self.widget_y + 55).contains(&y)
    }

    /// PB66 accepts two ready taps and discharges on the third. This method
    /// handles state only; the Board performs victims, coins, shots and input.
    pub fn tap_amp(&mut self, virtual_tank: bool) -> AmpTap {
        if !self.amp_ready() {
            return AmpTap::NotReady;
        }
        if self.amp_charge < 2 {
            self.amp_charge += 1;
            return AmpTap::Charged(self.amp_charge);
        }
        self.amp_timer = -20;
        self.amp_charge = 0;
        self.glint_phase = 0.0;
        if !virtual_tank {
            self.amp_threshold = self.amp_threshold.wrapping_add(200);
        }
        AmpTap::Discharged
    }

    /// The Board supplies the PB65 clock predicate separately from normal
    /// mouse availability: pause/Tank5/registered alien or Bilaterus suppress
    /// charging, while movement and animation continue on an active update.
    pub fn tick_amp(
        &mut self,
        charge_clock_allowed: bool,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> FishPetUpdate {
        assert_eq!(self.kind, FishPetKind::Amp);
        self.tick_inner(
            &[],
            0,
            &[],
            PetTargetViews::None,
            charge_clock_allowed,
            rand_range,
        )
    }

    /// W1 supplies steering and target order; this returns requests rather
    /// than mutating Board-owned health or fish membership. The Board applies
    /// contacts before calling finish_gash_clock with fresh threat membership.
    pub fn tick_gash(
        &mut self,
        aliens: &[PetAlienView],
        fish: &[GashFishView],
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> FishPetUpdate {
        assert_eq!(self.kind, FishPetKind::Gash);
        self.tick_inner(
            aliens,
            0,
            &[],
            PetTargetViews::Gash(fish),
            false,
            rand_range,
        )
    }

    /// PB74 reads corpse lifetime and contact geometry; the Board owns the
    /// revival timer transition and later fresh fish construction.
    pub fn tick_angie(
        &mut self,
        corpses: &[AngieCorpseView],
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> FishPetUpdate {
        assert_eq!(self.kind, FishPetKind::Angie);
        self.tick_inner(
            &[],
            0,
            &[],
            PetTargetViews::Angie(corpses),
            false,
            rand_range,
        )
    }

    /// Only call after the Board's requested ordinary-Fish removal succeeds.
    /// Source contact resets the timer before DropCoin runs in this update.
    pub fn commit_gash_meal(&mut self) {
        assert_eq!(self.kind, FishPetKind::Gash);
        self.gash_timer = 0;
    }

    /// PB70: DropCoin executes after Hungry/contact and tests current Board
    /// pause, Tank5 and registered-threat state. The Board owns that predicate.
    pub fn finish_gash_clock(&mut self, clock_allowed: bool) {
        assert_eq!(self.kind, FishPetKind::Gash);
        if clock_allowed {
            self.gash_timer = self.gash_timer.wrapping_add(1);
        }
    }

    pub fn tick(
        &mut self,
        aliens: &[PetAlienView],
        guppy_count: usize,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> FishPetUpdate {
        assert!(
            !matches!(
                self.kind,
                FishPetKind::Zorf
                    | FishPetKind::Nimbus
                    | FishPetKind::Amp
                    | FishPetKind::Gash
                    | FishPetKind::Angie
            ),
            "Zorf, Nimbus, Amp, Gash and Angie need their subtype updates"
        );
        self.tick_inner(
            aliens,
            guppy_count,
            &[],
            PetTargetViews::None,
            false,
            rand_range,
        )
    }

    pub fn tick_zorf(
        &mut self,
        aliens: &[PetAlienView],
        hungry: &[ZorfHungryView],
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> FishPetUpdate {
        assert_eq!(self.kind, FishPetKind::Zorf, "tick_zorf requires Zorf");
        self.tick_inner(aliens, 0, hungry, PetTargetViews::None, false, rand_range)
    }

    pub fn tick_nimbus(
        &mut self,
        coins: &[NimbusCoinView],
        foods: &[NimbusFoodView],
        enemies_registered: bool,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> FishPetUpdate {
        assert_eq!(self.kind, FishPetKind::Nimbus);
        self.tick_inner(
            &[],
            0,
            &[],
            PetTargetViews::Nimbus(NimbusViews {
                coins,
                foods,
                enemies_registered,
            }),
            false,
            rand_range,
        )
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
        let _motion = self.tick_inner(&[], 0, &[], PetTargetViews::None, false, rand_range);
        ward
    }

    /// PB05 FishTypePet slot-22 still advances common motion in Tank 5,
    /// while slot-81 returns before any subtype action. Board should use this
    /// for every Tank-5 fish-shaped pet, including Amp, Gash and Angie.
    pub fn tick_tank5(&mut self, rand_range: &mut impl FnMut(u64) -> u64) -> FishPetUpdate {
        self.tick_inner(&[], 0, &[], PetTargetViews::Tank5, false, rand_range)
    }

    fn tick_inner(
        &mut self,
        aliens: &[PetAlienView],
        guppy_count: usize,
        hungry: &[ZorfHungryView],
        targets: PetTargetViews<'_>,
        amp_charge_clock_allowed: bool,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> FishPetUpdate {
        let mut update = FishPetUpdate::default();
        let tank5 = matches!(targets, PetTargetViews::Tank5);
        let hunting = self.kind == FishPetKind::Itchy && !aliens.is_empty();
        match targets {
            PetTargetViews::Nimbus(views) => {
                if !self.hunt_nimbus(views, &mut update) {
                    self.wander();
                }
            }
            PetTargetViews::Gash(fish) => {
                if !self.hunt_gash(aliens, fish, &mut update) {
                    self.wander();
                }
            }
            PetTargetViews::Angie(corpses) => {
                if !self.hunt_angie(corpses, &mut update) {
                    self.wander();
                }
            }
            PetTargetViews::None if self.kind == FishPetKind::Gumbo && !aliens.is_empty() => {
                self.hunt_gumbo(aliens);
            }
            PetTargetViews::None if hunting => self.hunt(aliens, &mut update),
            PetTargetViews::None | PetTargetViews::Tank5 => self.wander(),
        }
        self.special_timer = self.special_timer.saturating_add(1);
        self.movement_timer += 1;
        if self.movement_timer > 20 {
            self.movement_timer = 0;
            if rand_range(10) == 0 {
                self.movement_state = rand_range(9) as u8 + 1;
            }
        }
        if !tank5 && self.kind == FishPetKind::Prego && aliens.is_empty() {
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
        if !tank5 && self.kind == FishPetKind::Zorf && aliens.is_empty() {
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
        if !tank5 && self.kind == FishPetKind::Vert && aliens.is_empty() {
            self.coin_timer += 1;
            if self.coin_timer >= 216 {
                self.coin_timer = 0;
                update.gold_at = Some((self.widget_x + 15, self.widget_y + 10));
            }
        }
        if !tank5 && self.kind == FishPetKind::Meryl && aliens.is_empty() {
            self.coin_timer += 1;
            if self.coin_timer == 1300 {
                update.note_at = Some((self.widget_x + 15, self.widget_y - 5));
            } else if self.coin_timer >= 1400 {
                self.coin_timer = 0;
            }
        }
        if !tank5 && self.kind == FishPetKind::Shrapnel && aliens.is_empty() {
            self.coin_timer += 1;
            if self.coin_timer >= self.bomb_threshold {
                self.coin_timer = 0;
                update.bomb_at = Some((self.widget_x + 15, self.widget_y + 10));
            }
        }
        if self.kind == FishPetKind::Amp && amp_charge_clock_allowed {
            self.amp_timer = self.amp_timer.wrapping_add(1);
            update.amp_became_ready = self.amp_timer == self.amp_threshold;
        }
        if self.kind == FishPetKind::Amp {
            self.vy = self.vy.clamp(-0.5, 0.5);
        } else {
            match self.vx {
                0.0 => self.y += 1.0 / self.speed_mod,
                1.0 => self.y += 0.75 / self.speed_mod,
                2.0 => self.y += 0.5 / self.speed_mod,
                3.0 => self.y += 0.25 / self.speed_mod,
                _ => {}
            }
        }
        self.x = self.x.clamp(
            10.0,
            if self.kind == FishPetKind::Amp {
                460.0
            } else {
                540.0
            },
        );
        self.y = self.y.clamp(
            if self.kind == FishPetKind::Nimbus {
                320.0
            } else {
                95.0
            },
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
        if self.kind == FishPetKind::Amp && self.turn_ticks == 0 {
            self.glint_phase += if self.amp_ready() { 0.1 } else { 0.5 };
            if self.glint_phase >= 1.0 {
                self.glint_phase = -1.0;
            }
        }
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
        if let Some(form) = &mut self.presto_form {
            form.remaining_ticks = form.remaining_ticks.saturating_sub(1);
        }
        update
    }

    /// W1 FishTypePet::Hungry/FindNearestFood/HungryBehavior/CollideWithFood.
    /// Contact geometry and conversion requests use installed PB60 instead
    /// where it corrects the secondary coin-contact offset.
    fn hunt_nimbus(&mut self, views: NimbusViews<'_>, update: &mut FishPetUpdate) -> bool {
        let has_objects = !views.coins.is_empty() || !views.foods.is_empty();
        self.speed_mod = if has_objects { 0.5 } else { 1.8 };
        if !has_objects {
            return false;
        }

        // Foods precede coins. Each axis is truncated before integer
        // Euclidean distance, and a strict comparison retains the first tie.
        let mut nearest = None;
        let mut best_distance = 10_000;
        if !views.enemies_registered {
            for food in views.foods.iter().filter(|food| food.eligible) {
                let dx = (self.x + 40.0 - f64::from(food.widget_x + 20)) as i32;
                let dy = (self.y + 40.0 - f64::from(food.widget_y + 20)) as i32;
                let distance = (f64::from(dx * dx + dy * dy).sqrt()) as i32;
                if distance < best_distance {
                    best_distance = distance;
                    nearest = Some((food.widget_x, food.widget_y));
                }
            }
        }
        for coin in views.coins.iter().filter(|coin| coin.eligible) {
            let dx = (self.x + 40.0 - f64::from(coin.widget_x + 36)) as i32;
            let dy = (self.y + 40.0 - f64::from(coin.widget_y + 36)) as i32;
            let distance = (f64::from(dx * dx + dy * dy).sqrt()) as i32;
            if distance < best_distance {
                best_distance = distance;
                nearest = Some((coin.widget_x, coin.widget_y));
            }
        }

        let Some((target_x, target_y)) = nearest else {
            self.speed_mod = 1.8;
            return true; // A nonempty but ineligible list still suppresses wander.
        };
        if self.special_timer >= 5 {
            self.special_timer = 0;
            let cx = self.x + 40.0;
            let cy = self.y + 40.0;
            let tx = f64::from(target_x + 36);
            let ty = f64::from(target_y + 36);
            if cx > tx + 18.0 && self.vx > -2.3 {
                self.vx -= 0.5;
            } else if cx < tx - 18.0 && self.vx < 2.3 {
                self.vx += 0.5;
            } else if cx > tx + 8.0 && self.vx > -1.3 {
                self.vx -= 0.1;
            } else if cx < tx - 8.0 && self.vx < 1.3 {
                self.vx += 0.1;
            } else if cx > tx && self.vx > -0.3 || cx < tx && self.vx < 0.3 {
                self.vx = 0.0;
            }
            if cy > ty + 6.0 && self.vy > -2.0 {
                self.vy -= 0.6;
            } else if cy < ty - 6.0 && self.vy < 3.0 {
                self.vy += 1.0;
            } else if cy > ty && self.vy > -2.0 {
                self.vy -= 0.3;
            } else if cy < ty && self.vy < 3.0 {
                self.vy += 0.5;
            }
        }

        let cx = self.x + 40.0;
        let cy = self.y + 40.0;
        if let Some(coin) = views.coins.iter().find(|coin| {
            coin.eligible
                && (cx - f64::from(coin.widget_x + 36)).abs() < 30.0
                && (cy - f64::from(coin.widget_y + 36)).abs() < 30.0
        }) {
            update.nimbus_coin = Some(NimbusCoinRequest {
                coin_id: coin.id,
                x: coin.widget_x,
                y: self.y - 25.0,
            });
        }
        if !views.enemies_registered
            && let Some(food) = views.foods.iter().find(|food| {
                food.eligible
                    && (cx - f64::from(food.widget_x + 20)).abs() < 30.0
                    && (cy - f64::from(food.widget_y + 20)).abs() < 30.0
            })
        {
            update.nimbus_food = Some(NimbusFoodRequest {
                food_id: food.id,
                x: food.widget_x,
                y: self.y - 30.0,
            });
        }
        true
    }

    fn hunt(&mut self, aliens: &[PetAlienView], update: &mut FishPetUpdate) {
        let mut nearest = None;
        let mut best_distance = 10_000;
        for alien in aliens.iter().filter(|alien| !alien.healing) {
            let dx = (self.x + 40.0 - f64::from(alien.widget_x + alien.center_offset())) as i32;
            let dy = (self.y + 40.0 - f64::from(alien.widget_y + alien.center_offset())) as i32;
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
            let tx = f64::from(target.widget_x + target.center_offset());
            let ty = f64::from(target.widget_y + target.center_offset());
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
                && (self.x + 40.0 - f64::from(alien.widget_x + alien.center_offset())).abs() < 20.0
                && (self.y + 40.0 - f64::from(alien.widget_y + alien.center_offset())).abs() < 20.0
        }) {
            update.damaged_alien = Some(alien.id);
            update.punch_sound = true;
        }
    }

    /// PB70/PB71 target classes and contact; W1 supplies nearest ties,
    /// steering and the near-eat animation. The Board applies each request.
    fn hunt_gash(
        &mut self,
        aliens: &[PetAlienView],
        fish: &[GashFishView],
        update: &mut FishPetUpdate,
    ) -> bool {
        let enemies_registered = !aliens.is_empty();
        let peaceful_hunt =
            !enemies_registered && self.gash_timer > GASH_MEAL_THRESHOLD && !fish.is_empty();
        if !enemies_registered && !peaceful_hunt {
            return false;
        }
        let cx = self.x + 40.0;
        let cy = self.y + 40.0;
        let mut nearest = None;
        let mut best_distance = 10_000;
        if enemies_registered {
            for alien in aliens
                .iter()
                .filter(|alien| alien.bilaterus)
                .chain(aliens.iter().filter(|alien| !alien.bilaterus))
                .filter(|alien| !alien.healing)
            {
                let tx = alien.widget_x + alien.center_offset();
                let ty = alien.widget_y + alien.center_offset();
                let dx = (cx - f64::from(tx)) as i32;
                let dy = (cy - f64::from(ty)) as i32;
                let distance = ((f64::from(dx * dx + dy * dy)).sqrt()) as i32;
                if distance < best_distance {
                    best_distance = distance;
                    nearest = Some((tx, ty));
                }
            }
        } else {
            for target in fish {
                let tx = target.widget_x + 40;
                let ty = target.widget_y + 40;
                let dx = (cx - f64::from(tx)) as i32;
                let dy = (cy - f64::from(ty)) as i32;
                let distance = ((f64::from(dx * dx + dy * dy)).sqrt()) as i32;
                if distance < best_distance {
                    best_distance = distance;
                    nearest = Some((tx, ty));
                }
            }
        }
        if let Some((tx, ty)) = nearest.filter(|_| self.special_timer >= 5) {
            self.special_timer = 0;
            let x_limit = if enemies_registered { 9.0 } else { 8.0 };
            if cx < f64::from(tx) && self.vx < x_limit {
                self.vx += 2.5;
            } else if cx > f64::from(tx) && self.vx > -x_limit {
                self.vx -= 2.5;
            }
            if cy < f64::from(ty) && self.vy < 4.0 {
                self.vy += 1.5;
            } else if cy > f64::from(ty) && self.vy > -4.0 {
                self.vy -= 1.5;
            }
            if self.vx_abs < 5 {
                self.vx_abs += 1;
            }
        }
        // HungryBehavior only calls CollideWithFood when FindNearestFood
        // produced a target. A healing-only alien list still suppresses wander.
        if nearest.is_none() {
            return true;
        }
        if enemies_registered {
            for alien in aliens.iter().filter(|alien| alien.bilaterus) {
                let dx = (cx - f64::from(alien.widget_x + alien.center_offset())).abs();
                let dy = (cy - f64::from(alien.widget_y + alien.center_offset())).abs();
                if dx < 20.0 && dy < 20.0 && !alien.healing {
                    update.gash_group_hit = Some(alien.id);
                    update.punch_sound = true;
                    break;
                }
                if dx < 30.0 && dy < 30.0 && self.gash_eating_ticks == 0 {
                    self.gash_eating_ticks = 10;
                    break;
                }
            }
            // CollideWithFood walks the ordinary Alien list even when the
            // Bilaterus loop just hit or began an eating animation.
            for alien in aliens.iter().filter(|alien| !alien.bilaterus) {
                let dx = (cx - f64::from(alien.widget_x + alien.center_offset())).abs();
                let dy = (cy - f64::from(alien.widget_y + alien.center_offset())).abs();
                if dx < 20.0 && dy < 20.0 && !alien.healing {
                    update.damaged_alien = Some(alien.id);
                    update.punch_sound = true;
                    break;
                }
                if dx < 30.0 && dy < 30.0 && self.gash_eating_ticks == 0 {
                    self.gash_eating_ticks = 10;
                    break;
                }
            }
        } else if peaceful_hunt {
            for target in fish {
                let dx = (cx - f64::from(target.widget_x + 40)).abs();
                let dy = (cy - f64::from(target.widget_y + 40)).abs();
                if dx < 20.0 && dy < 20.0 {
                    update.gash_guppy_remove = Some(target.id);
                    break;
                }
                if dx < 30.0 && dy < 30.0 && self.gash_eating_ticks == 0 {
                    self.gash_eating_ticks = 10;
                    break;
                }
            }
        }
        true
    }

    /// PB74 lifetime/contact gates; W1 supplies pursuit steering and ordering.
    /// This stage never edits corpse membership or constructs the replacement.
    fn hunt_angie(&mut self, corpses: &[AngieCorpseView], update: &mut FishPetUpdate) -> bool {
        if corpses.is_empty() {
            return false;
        }
        let cx = self.x + 40.0;
        let cy = self.y + 40.0;
        let mut nearest = None;
        let mut best_distance = 10_000;
        for corpse in corpses.iter().filter(|corpse| corpse.remaining_ticks > 95) {
            // W1 nearest selection uses +40 even for an Ultra corpse.
            let dx = (cx - f64::from(corpse.widget_x + 40)) as i32;
            let dy = (cy - f64::from(corpse.widget_y + 40)) as i32;
            let distance = ((f64::from(dx * dx + dy * dy)).sqrt()) as i32;
            if distance < best_distance {
                best_distance = distance;
                nearest = Some(corpse);
            }
        }
        if let Some(target) = nearest.filter(|_| self.special_timer >= 5) {
            self.special_timer = 0;
            let tx = f64::from(target.widget_x + if target.ultra { 80 } else { 40 });
            let ty = f64::from(target.widget_y + if target.ultra { 80 } else { 40 });
            if cx > tx + 4.0 && self.vx > -3.0 {
                self.vx -= 1.0;
            } else if cx < tx - 4.0 && self.vx < 3.0 {
                self.vx += 1.0;
            } else if cx > tx + 2.0 && self.vx > -3.0 {
                self.vx -= 0.1;
            } else if cx < tx - 2.0 && self.vx < 3.0 {
                self.vx += 0.1;
            } else if cx > tx && self.vx > -3.0 {
                self.vx -= 0.05;
            } else if cx < tx && self.vx < 3.0 {
                self.vx += 0.05;
            }
            if cy > ty + 3.0 && self.vy > -2.0 {
                self.vy -= 0.6;
            } else if cy < ty - 3.0 && self.vy < 3.0 {
                self.vy += 1.0;
            } else if cy > ty && self.vy > -2.0 {
                self.vy -= 0.3;
            } else if cy < ty && self.vy < 3.0 {
                self.vy += 0.5;
            }
            if self.vx_abs < 5 {
                self.vx_abs += 1;
            }
        }
        // HungryBehavior calls CollideWithFood only when an eligible nearest
        // target exists, then collision scans every corpse in Board order.
        if nearest.is_some() {
            for corpse in corpses.iter().filter(|corpse| corpse.remaining_ticks > 95) {
                let offset = if corpse.ultra { 80 } else { 40 };
                let radius = if corpse.ultra { 60.0 } else { 30.0 };
                let dx = (cx - f64::from(corpse.widget_x + offset)).abs();
                let dy = (cy - f64::from(corpse.widget_y + offset)).abs();
                if dx < radius && dy < radius {
                    // PB74 returns after the first lifetime-eligible contact;
                    // the helper changes 100→10 conditionally, then returns.
                    if corpse.revival_ticks == 100 {
                        update.angie_revive = Some(corpse.id);
                    }
                    break;
                }
            }
        }
        true
    }

    /// W1 FishTypePet::HungryBehavior selects the nearest non-healing alien,
    /// then flees its widget pose; this stage has no Bilaterus target.
    fn hunt_gumbo(&mut self, aliens: &[PetAlienView]) {
        let mut nearest = None;
        let mut best = i32::MAX;
        for alien in aliens.iter().filter(|alien| !alien.healing) {
            let dx = (self.x + 40.0 - f64::from(alien.widget_x + alien.center_offset())) as i32;
            let dy = (self.y + 40.0 - f64::from(alien.widget_y + alien.center_offset())) as i32;
            let distance = ((f64::from(dx * dx + dy * dy)).sqrt()) as i32;
            if distance < best {
                best = distance;
                nearest = Some(alien);
            }
        }
        if let Some(alien) = nearest.filter(|_| self.special_timer >= 5) {
            self.special_timer = 0;
            // Bilaterus group uses +40; ordinary aliens use +80.
            let target_x = alien.widget_x + alien.center_offset();
            let target_y = alien.widget_y + alien.center_offset();
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
        } else if self.kind == FishPetKind::Gash && self.gash_eating_ticks > 0 {
            self.gash_eating_ticks -= 1;
            self.frame = self.gash_eating_ticks;
        } else {
            self.swim_counter += if matches!(self.kind, FishPetKind::Zorf | FishPetKind::Amp)
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
                } else if self.kind == FishPetKind::Nimbus {
                    39
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
            } else if self.kind == FishPetKind::Angie {
                self.glint_phase += 0.1;
                if self.glint_phase >= 1.0 {
                    self.glint_phase = -1.0;
                }
                self.swim_counter / 2
            } else if self.kind == FishPetKind::Nimbus {
                self.swim_counter / 4
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
    fn presto_constructor_distinguishes_raw_and_flagged_forms_and_position_draws() {
        let mut ordinary_draws = Vec::new();
        let raw = FishPetState::spawn_tank1(1, FishPetKind::Presto, &mut |upper| {
            ordinary_draws.push(upper);
            0
        });
        assert_eq!(ordinary_draws, [265, 520, 2, 3, 200, 3, 10, 200]);
        assert_eq!(raw.presto_form, Some(PrestoForm { remaining_ticks: 0 }));
        assert!(raw.validate().is_ok());

        let mut replacement_draws = Vec::new();
        let flagged = FishPetState::spawn_presto_form_at(
            2,
            FishPetKind::Itchy,
            145,
            275,
            false,
            &mut |upper| {
                replacement_draws.push(upper);
                0
            },
        );
        assert_eq!(replacement_draws.as_slice(), &ordinary_draws[2..]);
        assert_eq!((flagged.widget_x, flagged.widget_y), (145, 275));
        assert_eq!(
            flagged.presto_form,
            Some(PrestoForm {
                remaining_ticks: 360
            })
        );
        assert!(flagged.validate().is_ok());

        let mut specialty_draws = Vec::new();
        let shrapnel = FishPetState::spawn_presto_form_at(
            4,
            FishPetKind::Shrapnel,
            145,
            275,
            false,
            &mut |upper| {
                specialty_draws.push(upper);
                0
            },
        );
        assert_eq!(specialty_draws, [2, 3, 200, 3, 10, 200, 20]);
        assert_eq!(shrapnel.bomb_threshold, 633);
        assert_eq!(
            shrapnel.presto_form,
            Some(PrestoForm {
                remaining_ticks: 360
            })
        );
        assert!(shrapnel.validate().is_ok());

        let returned =
            FishPetState::spawn_presto_form_at(5, FishPetKind::Presto, 145, 275, false, &mut |_| 0);
        assert_eq!(
            returned.presto_form,
            Some(PrestoForm { remaining_ticks: 0 })
        );

        let virtual_tank =
            FishPetState::spawn_presto_form_at(3, FishPetKind::Itchy, 145, 275, true, &mut |_| 0);
        assert_eq!(
            virtual_tank.presto_form,
            Some(PrestoForm { remaining_ticks: 0 })
        );

        let flagged_gash =
            FishPetState::spawn_presto_form_at(6, FishPetKind::Gash, 145, 275, false, &mut |_| 0);
        assert_eq!(flagged_gash.gash_timer, 1520);
        assert_eq!(
            flagged_gash.presto_form,
            Some(PrestoForm {
                remaining_ticks: 360
            })
        );
        assert!(flagged_gash.validate().is_ok());
    }

    #[test]
    fn presto_eligibility_is_read_only_and_preserves_form_ownership() {
        let plain = FishPetState::spawn_tank1(1, FishPetKind::Itchy, &mut |_| 0);
        assert_eq!(
            plain.presto_change_eligibility(FishPetKind::Prego),
            PrestoChangeEligibility::NotPrestoForm
        );
        let mut form =
            FishPetState::spawn_presto_form_at(2, FishPetKind::Itchy, 145, 275, false, &mut |_| 0);
        let before = serde_json::to_vec(&form).unwrap();
        assert_eq!(
            form.presto_change_eligibility(FishPetKind::Itchy),
            PrestoChangeEligibility::SameForm
        );
        assert_eq!(
            form.presto_change_eligibility(FishPetKind::Prego),
            PrestoChangeEligibility::CoolingDown
        );
        assert_eq!(serde_json::to_vec(&form).unwrap(), before);
        form.presto_form.as_mut().unwrap().remaining_ticks = 0;
        assert_eq!(
            form.presto_change_eligibility(FishPetKind::Prego),
            PrestoChangeEligibility::Ready
        );
        form.presto_form = Some(PrestoForm {
            remaining_ticks: 361,
        });
        assert!(form.validate().is_err());
        form.presto_form = None;
        assert!(form.validate().is_ok());
        let mut raw = FishPetState::spawn_tank1(3, FishPetKind::Presto, &mut |_| 0);
        raw.presto_form = None;
        assert!(raw.validate().is_err());
    }

    #[test]
    fn presto_clock_advances_once_per_active_update_and_continues_after_save() {
        let mut form =
            FishPetState::spawn_presto_form_at(2, FishPetKind::Itchy, 145, 275, false, &mut |_| 0);
        // Independently recovered ctor360 and one decrement per active update.
        for expected in (1..360).rev() {
            form.tick(&[], 0, &mut |_| 0);
            assert_eq!(form.presto_form.unwrap().remaining_ticks, expected);
        }
        assert_eq!(form.presto_form.unwrap().remaining_ticks, 1);
        assert_eq!(
            form.presto_change_eligibility(FishPetKind::Prego),
            PrestoChangeEligibility::CoolingDown
        );
        let saved = serde_json::to_vec(&form).unwrap();
        let mut resumed: FishPetState = serde_json::from_slice(&saved).unwrap();
        form.tick(&[], 0, &mut |_| 0);
        resumed.tick(&[], 0, &mut |_| 0);
        assert_eq!(resumed.presto_form.unwrap().remaining_ticks, 0);
        assert_eq!(
            serde_json::to_vec(&resumed).unwrap(),
            serde_json::to_vec(&form).unwrap()
        );
        assert_eq!(
            resumed.presto_change_eligibility(FishPetKind::Prego),
            PrestoChangeEligibility::Ready
        );
        resumed.tick(&[], 0, &mut |_| 0);
        assert_eq!(resumed.presto_form.unwrap().remaining_ticks, 0);
    }

    #[test]
    fn tank5_constructor_uses_ordinary_adventure_mode_and_clocks() {
        let mut vert_draws = Vec::new();
        let vert = FishPetState::spawn_tank5(1, FishPetKind::Vert, &mut |upper| {
            vert_draws.push(upper);
            0
        });
        assert_eq!(vert_draws, [265, 520, 2, 3, 200, 3, 10, 200]);
        assert_eq!(vert.coin_timer, 0);
        assert!(vert.validate_tank5().is_ok());

        let mut shrapnel_draws = Vec::new();
        let shrapnel = FishPetState::spawn_tank5(2, FishPetKind::Shrapnel, &mut |upper| {
            shrapnel_draws.push(upper);
            0
        });
        assert_eq!(shrapnel_draws, [265, 520, 2, 3, 200, 3, 10, 200, 20]);
        assert_eq!(shrapnel.bomb_threshold, 633);
        assert!(shrapnel.validate_tank5().is_ok());
        assert!(shrapnel.validate().is_ok());

        let gash = FishPetState::spawn_tank5(3, FishPetKind::Gash, &mut |_| 0);
        assert_eq!(gash.gash_timer, -1550);
        assert!(gash.validate_tank5().is_ok());
    }

    #[test]
    fn tank5_fish_motion_keeps_all_specialty_requests_suppressed() {
        let kinds = [
            FishPetKind::Itchy,
            FishPetKind::Prego,
            FishPetKind::Zorf,
            FishPetKind::Vert,
            FishPetKind::Meryl,
            FishPetKind::Wadsworth,
            FishPetKind::Seymour,
            FishPetKind::Shrapnel,
            FishPetKind::Gumbo,
            FishPetKind::Blip,
            FishPetKind::Nimbus,
            FishPetKind::Amp,
            FishPetKind::Gash,
            FishPetKind::Angie,
        ];
        for (index, kind) in kinds.into_iter().enumerate() {
            let mut pet = FishPetState::spawn_tank5(index as u64 + 1, kind, &mut |_| 0);
            if kind == FishPetKind::Prego {
                pet.birth_timer = 929;
            }
            if kind == FishPetKind::Zorf {
                pet.food_timer = 64;
            }
            if kind == FishPetKind::Vert {
                pet.coin_timer = 215;
            }
            if kind == FishPetKind::Meryl {
                pet.coin_timer = 1299;
            }
            if kind == FishPetKind::Shrapnel {
                pet.coin_timer = pet.bomb_threshold - 1;
            }
            if kind == FishPetKind::Amp {
                pet.amp_timer = pet.amp_threshold - 1;
            }
            let prior = (
                pet.birth_timer,
                pet.food_timer,
                pet.coin_timer,
                pet.amp_timer,
                pet.gash_timer,
                pet.ward_timer,
            );
            let before_y = pet.y;
            assert_eq!(
                pet.tick_tank5(&mut |_| 1),
                FishPetUpdate::default(),
                "{kind:?}"
            );
            assert_eq!(
                (
                    pet.birth_timer,
                    pet.food_timer,
                    pet.coin_timer,
                    pet.amp_timer,
                    pet.gash_timer,
                    pet.ward_timer
                ),
                prior,
                "{kind:?}"
            );
            assert_ne!(pet.y, before_y, "{kind:?} motion must advance");
            assert!(pet.validate_tank5().is_ok(), "{kind:?}");
        }
    }

    fn angie_at(x: i32, y: i32) -> FishPetState {
        let mut pet = actor(FishPetKind::Angie);
        pet.x = f64::from(x);
        pet.y = f64::from(y);
        pet.widget_x = x;
        pet.widget_y = y;
        pet.published_x = x;
        pet.published_y = y;
        pet.vx = 0.0;
        pet.previous_vx = 0.0;
        pet
    }

    fn corpse_at(id: u64, x: i32, y: i32) -> AngieCorpseView {
        AngieCorpseView {
            id,
            widget_x: x,
            widget_y: y,
            remaining_ticks: 96,
            ultra: false,
            revival_ticks: 100,
        }
    }

    #[test]
    fn angie_requires_remaining_lifetime_and_stops_at_first_contact() {
        let mut pet = angie_at(100, 100);
        let expired = AngieCorpseView {
            remaining_ticks: 95,
            ..corpse_at(7, 100, 100)
        };
        assert_eq!(pet.tick_angie(&[expired], &mut |_| 1).angie_revive, None);
        assert_eq!(pet.special_timer, 41); // Nonempty list suppresses wandering.
        let fresh = corpse_at(8, 100, 100);
        assert_eq!(pet.tick_angie(&[fresh], &mut |_| 1).angie_revive, Some(8));
        let already_reviving = AngieCorpseView {
            revival_ticks: 10,
            ..corpse_at(9, 100, 100)
        };
        assert_eq!(
            pet.tick_angie(&[already_reviving, fresh], &mut |_| 1)
                .angie_revive,
            None
        );
        assert_eq!(
            pet.tick_angie(&[fresh, already_reviving], &mut |_| 1)
                .angie_revive,
            Some(8)
        );
        assert!((pet.glint_phase - 0.4).abs() < 1e-9);
        assert_eq!(pet.sprite_row(false), 0);
        pet.turn_ticks = 3;
        assert_eq!(pet.sprite_row(false), 1);
        pet.validate().unwrap();
    }

    #[test]
    fn angie_uses_strict_ordinary_30_and_ultra_60_contact_geometry() {
        let mut ordinary = angie_at(100, 100);
        assert_eq!(
            ordinary
                .tick_angie(&[corpse_at(1, 130, 100)], &mut |_| 1)
                .angie_revive,
            None
        );
        let mut inner = angie_at(100, 100);
        assert_eq!(
            inner
                .tick_angie(&[corpse_at(2, 129, 100)], &mut |_| 1)
                .angie_revive,
            Some(2)
        );
        let mut ultra_edge = angie_at(100, 100);
        let ultra = AngieCorpseView {
            ultra: true,
            ..corpse_at(3, 0, 0)
        };
        assert_eq!(
            ultra_edge.tick_angie(&[ultra], &mut |_| 1).angie_revive,
            None
        );
        let mut ultra_inner = angie_at(100, 100);
        let inside = AngieCorpseView {
            widget_x: 1,
            widget_y: 1,
            ..ultra
        };
        assert_eq!(
            ultra_inner.tick_angie(&[inside], &mut |_| 1).angie_revive,
            Some(3)
        );
    }

    #[test]
    fn angie_steers_on_fifth_step_and_uses_shared_swim_halo_state() {
        let mut pet = angie_at(100, 100);
        pet.special_timer = 4;
        let far = corpse_at(9, 180, 180);
        pet.tick_angie(&[far], &mut |_| 1);
        assert_eq!(pet.vx, 0.0);
        pet.tick_angie(&[far], &mut |_| 1);
        assert_eq!((pet.vx, pet.vy, pet.special_timer), (1.0, 0.5, 1));
        assert!((pet.glint_phase - 0.2).abs() < 1e-9);
        pet.validate().unwrap();
    }

    fn gash_at(x: i32, y: i32) -> FishPetState {
        let mut pet = actor(FishPetKind::Gash);
        pet.x = f64::from(x);
        pet.y = f64::from(y);
        pet.widget_x = x;
        pet.widget_y = y;
        pet.published_x = x;
        pet.published_y = y;
        pet.vx = 0.0;
        pet.previous_vx = 0.0;
        pet
    }

    #[test]
    fn gash_signed_clock_obeys_strict_peaceful_hunt_and_enemy_freeze() {
        let mut pet = gash_at(100, 100);
        let prey = [GashFishView {
            id: 7,
            widget_x: 100,
            widget_y: 100,
        }];
        assert_eq!(pet.gash_timer, -1550);
        pet.gash_timer = GASH_MEAL_THRESHOLD;
        assert_eq!(
            pet.tick_gash(&[], &prey, &mut |_| 1).gash_guppy_remove,
            None
        );
        assert_eq!(pet.gash_timer, GASH_MEAL_THRESHOLD);
        pet.finish_gash_clock(false);
        assert_eq!(pet.gash_timer, GASH_MEAL_THRESHOLD);
        assert_eq!(
            pet.tick_gash(&[], &prey, &mut |_| 1).gash_guppy_remove,
            None
        );
        pet.finish_gash_clock(true);
        assert_eq!(pet.gash_timer, GASH_MEAL_THRESHOLD + 1);
        assert_eq!(
            pet.tick_gash(&[], &prey, &mut |_| 1).gash_guppy_remove,
            Some(7)
        );
        let enemy = [PetAlienView {
            id: 9,
            widget_x: 300,
            widget_y: 100,
            healing: false,
            bilaterus: false,
        }];
        pet.tick_gash(&enemy, &prey, &mut |_| 1);
        pet.finish_gash_clock(false);
        assert_eq!(pet.gash_timer, GASH_MEAL_THRESHOLD + 1);
        let last_enemy = [PetAlienView {
            id: 10,
            widget_x: pet.widget_x - 40,
            widget_y: pet.widget_y - 40,
            ..enemy[0]
        }];
        let hit = pet.tick_gash(&last_enemy, &prey, &mut |_| 1);
        assert_eq!(hit.damaged_alien, Some(10));
        pet.finish_gash_clock(true); // Board removed the last alien after contact.
        assert_eq!(pet.gash_timer, GASH_MEAL_THRESHOLD + 2);
        pet.gash_timer = i32::MAX;
        pet.tick_gash(&[], &[], &mut |_| 1);
        pet.finish_gash_clock(true);
        assert_eq!(pet.gash_timer, i32::MIN);
    }

    #[test]
    fn gash_guppy_request_precedes_motion_and_board_commits_timer_reset() {
        let mut pet = gash_at(100, 100);
        pet.gash_timer = 1571;
        let prey = [GashFishView {
            id: 42,
            widget_x: 119,
            widget_y: 100,
        }];
        let update = pet.tick_gash(&[], &prey, &mut |_| 1);
        assert_eq!(update.gash_guppy_remove, Some(42));
        assert_eq!(pet.gash_timer, 1571);
        pet.commit_gash_meal();
        pet.finish_gash_clock(false);
        assert_eq!(pet.gash_timer, 0);
        pet.gash_timer = 1571;
        let again = pet.tick_gash(&[], &prey, &mut |_| 1);
        assert_eq!(again.gash_guppy_remove, Some(42));
        pet.commit_gash_meal();
        pet.finish_gash_clock(true);
        assert_eq!(pet.gash_timer, 1);
        let mut boundary = gash_at(100, 100);
        boundary.gash_timer = 1571;
        let outside = [GashFishView {
            id: 43,
            widget_x: 120,
            widget_y: 100,
        }];
        assert_eq!(
            boundary
                .tick_gash(&[], &outside, &mut |_| 1)
                .gash_guppy_remove,
            None
        );
        assert_eq!(boundary.gash_eating_ticks, 9);
        assert_eq!(
            (boundary.sprite_row(false), boundary.sprite_frame()),
            (2, 9)
        );
    }

    #[test]
    fn gash_contact_prioritizes_group_and_excludes_healing_damage() {
        let mut pet = gash_at(100, 100);
        let group = PetAlienView {
            id: 71,
            widget_x: 100,
            widget_y: 100,
            healing: false,
            bilaterus: true,
        };
        let ordinary = PetAlienView {
            id: 72,
            widget_x: 60,
            widget_y: 60,
            healing: false,
            bilaterus: false,
        };
        let hit = pet.tick_gash(&[ordinary, group], &[], &mut |_| 1);
        assert_eq!(hit.gash_group_hit, Some(71));
        assert_eq!(hit.damaged_alien, Some(72));
        assert!(hit.punch_sound);
        let mut near_group = gash_at(100, 100);
        let near = PetAlienView {
            widget_x: 125,
            ..group
        };
        let second_list = near_group.tick_gash(&[near, ordinary], &[], &mut |_| 1);
        assert_eq!(second_list.gash_group_hit, None);
        assert_eq!(second_list.damaged_alien, Some(72));
        assert_eq!(near_group.gash_eating_ticks, 9);
        let mut healing = gash_at(100, 100);
        let healer = PetAlienView {
            id: 73,
            healing: true,
            ..ordinary
        };
        let no_hit = healing.tick_gash(&[healer], &[], &mut |_| 1);
        assert_eq!(no_hit.damaged_alien, None);
        assert_eq!(healing.gash_eating_ticks, 0);
        let far_live = PetAlienView {
            id: 74,
            widget_x: 300,
            healing: false,
            ..ordinary
        };
        healing.tick_gash(&[healer, far_live], &[], &mut |_| 1);
        assert_eq!(healing.gash_eating_ticks, 9); // Eligible far target enables contact scan.
        let mut ordinary_hit = gash_at(100, 100);
        let hit = ordinary_hit.tick_gash(&[ordinary], &[], &mut |_| 1);
        assert_eq!(hit.damaged_alien, Some(72));
        let mut edge = gash_at(100, 100);
        let at_twenty = PetAlienView {
            widget_x: 80,
            widget_y: 60,
            ..ordinary
        };
        assert_eq!(
            edge.tick_gash(&[at_twenty], &[], &mut |_| 1).damaged_alien,
            None
        );
    }

    #[test]
    fn gash_steers_toward_nearest_nonhealing_enemy_on_fifth_step() {
        let mut pet = gash_at(100, 100);
        pet.special_timer = 4;
        let enemies = [
            PetAlienView {
                id: 81,
                widget_x: 200,
                widget_y: 60,
                healing: false,
                bilaterus: true,
            },
            PetAlienView {
                id: 82,
                widget_x: 0,
                widget_y: 60,
                healing: false,
                bilaterus: false,
            },
            PetAlienView {
                id: 83,
                widget_x: 60,
                widget_y: 60,
                healing: true,
                bilaterus: false,
            },
        ];
        pet.tick_gash(&enemies, &[], &mut |_| 1);
        assert_eq!(pet.vx, 0.0);
        assert_eq!(pet.special_timer, 5);
        pet.tick_gash(&enemies, &[], &mut |_| 1);
        assert_eq!(pet.vx, -2.5); // The closer ordinary Alien beats the group.
        assert_eq!(pet.special_timer, 1);
    }

    #[test]
    fn gash_eat_animation_and_durable_subtype_state_are_bounded() {
        let mut pet = gash_at(100, 100);
        pet.gash_timer = 1571;
        let near = [GashFishView {
            id: 20,
            widget_x: 125,
            widget_y: 100,
        }];
        pet.tick_gash(&[], &near, &mut |_| 1);
        assert_eq!(
            (pet.gash_eating_ticks, pet.sprite_row(false), pet.frame),
            (9, 2, 9)
        );
        pet.turn_ticks = 3;
        assert_eq!(pet.sprite_row(false), 1);
        pet.validate().unwrap();
        pet.gash_eating_ticks = 11;
        assert!(pet.validate().is_err());
        let mut other = actor(FishPetKind::Itchy);
        other.gash_timer = -1550;
        assert!(other.validate().is_err());
    }

    #[test]
    fn amp_factory_charge_clock_and_three_taps_follow_primary_boundaries() {
        let mut pet = actor(FishPetKind::Amp);
        assert_eq!(
            (pet.amp_timer, pet.amp_threshold, pet.amp_charge),
            (300, 3000, 0)
        );
        assert_eq!(pet.speed_mod, 0.5);
        assert_eq!(pet.tap_amp(false), AmpTap::NotReady);
        pet.amp_timer = 2999;
        assert!(!pet.tick_amp(false, &mut |_| 1).amp_became_ready);
        assert_eq!(pet.amp_timer, 2999);
        assert!(pet.tick_amp(true, &mut |_| 1).amp_became_ready);
        assert!(pet.amp_ready());
        assert!(!pet.tick_amp(true, &mut |_| 1).amp_became_ready);
        assert_eq!(pet.tap_amp(false), AmpTap::Charged(1));
        assert_eq!(pet.tap_amp(false), AmpTap::Charged(2));
        assert_eq!(pet.tap_amp(false), AmpTap::Discharged);
        assert_eq!(
            (pet.amp_timer, pet.amp_threshold, pet.amp_charge),
            (-20, 3200, 0)
        );
        assert_eq!(pet.glint_phase, 0.0);
        assert_eq!(pet.tap_amp(false), AmpTap::NotReady);
        pet.validate().unwrap();
    }

    #[test]
    fn amp_signed_clock_and_threshold_use_wrapping_dword_arithmetic() {
        let mut pet = actor(FishPetKind::Amp);
        pet.amp_timer = i32::MAX;
        pet.amp_threshold = i32::MAX;
        assert!(pet.amp_ready());
        pet.amp_charge = 2;
        assert_eq!(pet.tap_amp(false), AmpTap::Discharged);
        assert_eq!(pet.amp_threshold, i32::MIN + 199);
        pet.amp_timer = i32::MAX;
        pet.amp_threshold = 3000;
        assert!(!pet.tick_amp(true, &mut |_| 1).amp_became_ready);
        assert_eq!(pet.amp_timer, i32::MIN);
        assert!(!pet.amp_ready());
        pet.amp_timer = 3000;
        pet.amp_charge = 2;
        assert_eq!(pet.tap_amp(true), AmpTap::Discharged);
        assert_eq!(pet.amp_threshold, 3000);
    }

    #[test]
    fn amp_motion_and_hitbox_preserve_secondary_geometry() {
        let mut pet = actor(FishPetKind::Amp);
        pet.x = 500.0;
        pet.y = 200.0;
        pet.widget_x = 500;
        pet.widget_y = 200;
        pet.published_x = 500;
        pet.published_y = 200;
        pet.vx = 0.0;
        pet.vy = 3.0;
        pet.movement_state = 3;
        pet.special_timer = 0;
        pet.tick_amp(false, &mut |_| 1);
        assert_eq!(pet.vy, 0.5);
        assert_eq!(pet.x, 460.0);
        assert!(pet.amp_hitbox_contains(pet.widget_x, pet.widget_y));
        assert!(pet.amp_hitbox_contains(pet.widget_x + 159, pet.widget_y + 54));
        assert!(!pet.amp_hitbox_contains(pet.widget_x + 160, pet.widget_y));
        assert!(!pet.amp_hitbox_contains(pet.widget_x, pet.widget_y + 55));
        pet.validate().unwrap();
    }

    #[test]
    fn amp_save_rejects_wrong_subtype_fields_and_excess_taps() {
        let mut pet = actor(FishPetKind::Amp);
        pet.validate().unwrap();
        pet.amp_charge = 3;
        assert!(pet.validate().is_err());
        pet.amp_charge = 0;
        pet.speed_mod = 1.8;
        assert!(pet.validate().is_err());
        let mut other = actor(FishPetKind::Itchy);
        other.amp_threshold = 3000;
        assert!(other.validate().is_err());
    }

    #[test]
    fn meryl_note_clock_freezes_on_registration_and_song_ends_at_1400() {
        let mut pet = actor(FishPetKind::Meryl);
        pet.coin_timer = 1299;
        let alien = [PetAlienView {
            id: 7,
            widget_x: 100,
            widget_y: 100,
            healing: false,
            bilaterus: false,
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
    fn itchy_uses_group_center_40_and_strict_20_contact() {
        let mut pet = actor(FishPetKind::Itchy);
        pet.x = 100.0;
        pet.y = 100.0;
        pet.widget_x = 100;
        pet.widget_y = 100;
        pet.published_x = 100;
        pet.published_y = 100;
        pet.special_timer = 4;
        let mut group = PetAlienView {
            id: 91,
            widget_x: 80,
            widget_y: 100,
            healing: false,
            bilaterus: true,
        };
        assert_eq!(pet.tick(&[group], 0, &mut |_| 1).damaged_alien, None);
        pet.x = 100.0;
        pet.y = 100.0;
        pet.widget_x = 100;
        pet.widget_y = 100;
        group.widget_x = 81;
        let result = pet.tick(&[group], 0, &mut |_| 1);
        assert_eq!(result.damaged_alien, Some(91));
        assert!(result.punch_sound);

        let mut centered = actor(FishPetKind::Itchy);
        centered.x = 100.0;
        centered.y = 100.0;
        centered.widget_x = 100;
        centered.widget_y = 100;
        centered.special_timer = 5;
        group.widget_x = 100;
        centered.tick(&[group], 0, &mut |_| 1);
        assert_eq!(centered.vx, 0.0); // +80 would have steered right.
        assert_eq!(centered.special_timer, 1);
    }

    #[test]
    fn gumbo_flees_from_group_center_40_not_ordinary_alien_center_80() {
        let mut pet = actor(FishPetKind::Gumbo);
        pet.x = 100.0;
        pet.y = 100.0;
        pet.widget_x = 100;
        pet.widget_y = 100;
        pet.special_timer = 5;
        pet.tick(
            &[PetAlienView {
                id: 91,
                widget_x: 250,
                widget_y: 220,
                healing: false,
                bilaterus: true,
            }],
            0,
            &mut |_| 1,
        );
        assert_eq!(pet.vx, 2.0);
        assert_eq!(pet.vy, 1.5);
    }

    fn nimbus_at(x: i32, y: i32) -> FishPetState {
        let mut pet = actor(FishPetKind::Nimbus);
        pet.x = f64::from(x);
        pet.y = f64::from(y);
        pet.widget_x = x;
        pet.widget_y = y;
        pet.published_x = x;
        pet.published_y = y;
        pet.special_timer = 4;
        pet
    }

    #[test]
    fn nimbus_strict_primary_contact_can_convert_one_coin_and_one_food_before_motion() {
        let mut pet = nimbus_at(100, 320);
        let coins = [
            NimbusCoinView {
                id: 7,
                widget_x: 74,
                widget_y: 324,
                eligible: true,
            },
            NimbusCoinView {
                id: 8,
                widget_x: 75,
                widget_y: 324,
                eligible: true,
            },
            NimbusCoinView {
                id: 9,
                widget_x: 104,
                widget_y: 324,
                eligible: true,
            },
        ];
        let foods = [NimbusFoodView {
            id: 11,
            widget_x: 120,
            widget_y: 340,
            eligible: true,
        }];
        let update = pet.tick_nimbus(&coins, &foods, false, &mut |_| 1);
        assert_eq!(
            update.nimbus_coin,
            Some(NimbusCoinRequest {
                coin_id: 8,
                x: 75,
                y: 295.0
            })
        );
        assert_eq!(
            update.nimbus_food,
            Some(NimbusFoodRequest {
                food_id: 11,
                x: 120,
                y: 290.0
            })
        );
        assert_ne!(pet.y, 320.0); // Requests used the pre-movement double Y.
        pet.validate().unwrap();
    }

    #[test]
    fn nimbus_coin_remains_available_during_enemy_registration_but_food_does_not() {
        let mut pet = nimbus_at(100, 320);
        let coins = [NimbusCoinView {
            id: 7,
            widget_x: 104,
            widget_y: 324,
            eligible: true,
        }];
        let foods = [NimbusFoodView {
            id: 11,
            widget_x: 120,
            widget_y: 340,
            eligible: true,
        }];
        let update = pet.tick_nimbus(&coins, &foods, true, &mut |_| 1);
        assert_eq!(update.nimbus_coin.unwrap().coin_id, 7);
        assert_eq!(update.nimbus_food, None);
    }

    #[test]
    fn nimbus_nearest_food_wins_integer_distance_tie_and_ineligible_list_suppresses_wander() {
        let mut pet = nimbus_at(100, 320);
        pet.vx = 0.0;
        pet.special_timer = 5;
        let coins = [NimbusCoinView {
            id: 7,
            widget_x: 164,
            widget_y: 324,
            eligible: true,
        }];
        let foods = [NimbusFoodView {
            id: 11,
            widget_x: 60,
            widget_y: 340,
            eligible: true,
        }];
        pet.tick_nimbus(&coins, &foods, false, &mut |_| 1);
        assert_eq!(pet.vx, -0.5); // Equidistant food steers left; coin would steer right.
        assert_eq!(pet.special_timer, 1);

        let mut ineligible = nimbus_at(100, 320);
        ineligible.special_timer = 40;
        ineligible.tick_nimbus(
            &[NimbusCoinView {
                eligible: false,
                ..coins[0]
            }],
            &[],
            false,
            &mut |_| 1,
        );
        assert_eq!(ineligible.vy, -0.5); // No wander branch despite nonempty coin list.
        assert_eq!(ineligible.speed_mod, 1.8);
        assert_eq!(ineligible.special_timer, 41);

        let mut empty = nimbus_at(100, 320);
        empty.tick_nimbus(&[], &[], false, &mut |_| 1);
        assert_eq!(empty.vy, 0.5);
        assert_eq!(empty.speed_mod, 1.8);
    }

    #[test]
    fn nimbus_constructor_and_forty_count_swim_animation() {
        let mut draws = Vec::new();
        let mut pet = FishPetState::spawn_tank1(1, FishPetKind::Nimbus, &mut |upper| {
            draws.push(upper);
            0
        });
        assert_eq!(draws, [265, 520, 2, 3, 200, 3, 10, 200]);
        assert_eq!(pet.speed_mod, 0.5);
        pet.swim_counter = 39;
        pet.tick_nimbus(&[], &[], false, &mut |_| 1);
        assert_eq!((pet.swim_counter, pet.frame), (0, 0));
        pet.validate().unwrap();
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
            bilaterus: false,
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
                bilaterus: false,
            },
            PetAlienView {
                id: 3,
                widget_x: 40,
                widget_y: 60,
                healing: false,
                bilaterus: false,
            },
            PetAlienView {
                id: 4,
                widget_x: 60,
                widget_y: 60,
                healing: false,
                bilaterus: false,
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
            bilaterus: false,
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
            bilaterus: false,
        };
        let right = PetAlienView {
            id: 3,
            widget_x: 100,
            widget_y: 60,
            healing: false,
            bilaterus: false,
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
            bilaterus: false,
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
            bilaterus: false,
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
            bilaterus: false,
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
            bilaterus: false,
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
                bilaterus: false,
            },
            PetAlienView {
                id: 2,
                widget_x: 300,
                widget_y: 300,
                healing: false,
                bilaterus: false,
            },
        ];
        pet.tick(&aliens, 0, &mut |_| 1);
        assert_eq!((pet.vx, pet.vy), (-9.5, -9.5)); // guard precedes ±2 step
        assert!(pet.gumbo_light_alpha() > 0);
        pet.validate().unwrap();
    }
}
