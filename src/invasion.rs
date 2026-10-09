//! Ordinary weak Sylvester wave in Adventure 1-2.
//!
//! This owns the board countdown, live actor membership and finite visual
//! effects, while the caller owns board input routing, prey removal, coins,
//! menu pauses, sound and the existing RNG/id streams. The branch rules are
//! secondary source-derived from WinFish f919b3c (`Board.cpp`, `Alien.cpp`,
//! `Warp.cpp`, `Shot.cpp`, `DeadAlien.cpp`). Installed-game parity is untested.
//! Render frame endpoints for the warp and death particles remain unresolved.

use serde::{Deserialize, Serialize};

use crate::alien::{
    AlienFoodView, AlienRuntimeRequest, PreyView, ShotResult, SylvesterKind, WeakSylvester,
};
use crate::bilaterus::{
    BilaterusFragment, BilaterusPreyView, BilaterusShot, BilaterusState, BilaterusTransition,
    FragmentKind,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvasionOrigin {
    StageStart,
    /// Initial state synthesized for a pre-invasion-field version-3 save.
    /// Old board ticks do not reveal how much of the invasion timer elapsed.
    LegacyV3Resume,
    /// Synthesized timer for an old format-4 stage-3 board without a wave.
    LegacyV4Resume,
    /// Synthesized Balrog timer for a pre-format-6 stage-4 board.
    LegacyV5Resume,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvasionTip {
    Danger,
    BattleTip,
    GusWarning,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WarningCoords {
    pub first_x: i32,
    pub first_y: i32,
    pub second_x: i32,
    pub second_y: i32,
}

/// What the next warning will bring. Registered actors retain their own
/// species after this expectation changes at spawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncounterKind {
    Single(SylvesterKind),
    WeakBalrogPair,
    PsychosquidBalrogPair,
    DestructorUlyssesPair,
    BalrogBilaterusPair,
    Bilaterus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WavePlan {
    Fixed(SylvesterKind),
    FixedBilaterus,
    CyclingTank1Finale {
        next: EncounterKind,
    },
    CyclingTank2Finale {
        next: EncounterKind,
    },
    CyclingTank3Second {
        next: SylvesterKind,
    },
    CyclingTank3Finale {
        next: SylvesterKind,
    },
    CyclingTank4Third {
        next: EncounterKind,
    },
    CyclingTank4Fourth {
        next: EncounterKind,
    },
    CyclingTank4Finale {
        next: EncounterKind,
        wave_count: u32,
    },
}

impl WavePlan {
    pub fn expected(self) -> EncounterKind {
        match self {
            Self::Fixed(kind) => EncounterKind::Single(kind),
            Self::FixedBilaterus => EncounterKind::Bilaterus,
            Self::CyclingTank1Finale { next }
            | Self::CyclingTank2Finale { next }
            | Self::CyclingTank4Third { next }
            | Self::CyclingTank4Fourth { next }
            | Self::CyclingTank4Finale { next, .. } => next,
            Self::CyclingTank3Second { next } | Self::CyclingTank3Finale { next } => {
                EncounterKind::Single(next)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InvasionEvent {
    ModalOpened(InvasionTip),
    WarningStarted(WarningCoords),
    BattleMusicStarted,
    AlienSpawned {
        id: u64,
        x: i32,
        y: i32,
    },
    BilaterusSpawned {
        id: u64,
        x: i32,
        y: i32,
    },
    BilaterusHeadSwapped {
        id: u64,
    },
    BilaterusHeadHit {
        id: u64,
        health: f64,
    },
    BilaterusFirstHeadDefeated {
        id: u64,
    },
    BilaterusDefeated {
        id: u64,
    },
    BilaterusPreyEaten {
        group_id: u64,
        prey_id: u64,
    },
    BilaterusFragmentSpawned {
        id: u64,
        kind: FragmentKind,
    },
    PreyEaten {
        alien_id: u64,
        prey_id: u64,
    },
    GusAteFood {
        alien_id: u64,
        food_id: u64,
        damage: u8,
    },
    LaserFired {
        x: i32,
        y: i32,
    },
    AlienHit {
        id: u64,
        health: f64,
    },
    PsychosquidHealingHit {
        id: u64,
        health: f64,
    },
    PsychosquidPhaseChanged {
        id: u64,
        healing: bool,
        forced: bool,
    },
    AlienDefeated {
        id: u64,
    },
    DiamondDropped {
        alien_id: u64,
        x: i32,
        y: i32,
    },
    BattleEnded,
    FoodDelayCleared,
    /// A finite body reaches a seven-count particle trigger. The board owns
    /// actual particles and their RNG consumption, which is not modeled here.
    DeathBurstDue {
        x: i32,
        y: i32,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct InvasionClick {
    /// A live alien blocks feeding even for menu-area clicks (y <= 40).
    pub suppress_food: bool,
    pub events: Vec<InvasionEvent>,
}

/// Source child order is active head, bone 0..5, passive head. Callers must
/// rebuild prey membership between calls, as each child can consume one prey.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BilaterusChild {
    ActiveHead,
    Bone(u8),
    PassiveHead,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WarpEffect {
    pub x: i32,
    pub y: i32,
    /// Starts at 36. Source removes it on the update after it reaches zero.
    pub remaining_ticks: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LaserEffect {
    pub x: i32,
    pub y: i32,
    /// Ordinary type-zero laser is removed when this exceeds 14.
    pub age_ticks: u8,
}

impl LaserEffect {
    pub fn frame(&self) -> u8 {
        self.age_ticks.saturating_sub(1)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeadAlienEffect {
    #[serde(default)] // Pre-format-6 corpses were weak or strong Sylvester.
    pub kind: SylvesterKind,
    pub x: f64,
    pub y: f64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub vx: f64,
    pub vy: f64,
    pub frame: u8,
    pub facing_right: bool,
    pub opacity: f32,
    /// Starts at 125. Source removes it on the next update after zero.
    pub remaining_ticks: u8,
}

/// `board_update` must precede `objects_update` for a normal game update.
/// Pausing a modal means neither is called until `acknowledge_modal`.
#[allow(non_camel_case_types)] // The public name identifies Adventure tank 1, stage 2.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Invasion1_2 {
    pub origin: InvasionOrigin,
    pub plan: WavePlan,
    pub countdown: i32,
    pub danger_shown: bool,
    pub battle_tip_shown: bool,
    pub gus_warning_shown: bool,
    pub pending_modal: Option<InvasionTip>,
    pub warning: Option<WarningCoords>,
    pub actors: Vec<WeakSylvester>,
    /// Bilaterus is a separate registered combat identity, never an Alien.
    pub bilaterus: Vec<BilaterusState>,
    /// Targetless visual effects do not hold the battle or wave countdown.
    pub fragments: Vec<BilaterusFragment>,
    /// Set by the spawn transaction and cleared only after the last alien
    /// and missile leaves. It distinguishes a finished battle from peace.
    pub battle_active: bool,
    pub food_delay: u8,
    pub last_laser: Option<(i32, i32)>,
    pub post_spawn_flash_ticks: u8,
    pub warps: Vec<WarpEffect>,
    pub lasers: Vec<LaserEffect>,
    pub dead_aliens: Vec<DeadAlienEffect>,
}

impl Invasion1_2 {
    pub fn new() -> Self {
        Self::with_origin(InvasionOrigin::StageStart, SylvesterKind::Weak)
    }

    pub fn new_strong() -> Self {
        Self::with_origin(InvasionOrigin::StageStart, SylvesterKind::Strong)
    }

    pub fn new_balrog() -> Self {
        Self::with_origin(InvasionOrigin::StageStart, SylvesterKind::Balrog)
    }

    pub fn new_gus() -> Self {
        Self::with_origin(InvasionOrigin::StageStart, SylvesterKind::Gus)
    }

    pub fn new_destructor() -> Self {
        Self::with_origin(InvasionOrigin::StageStart, SylvesterKind::Destructor)
    }

    pub fn new_psychosquid() -> Self {
        Self::with_origin(InvasionOrigin::StageStart, SylvesterKind::Psychosquid)
    }

    pub fn new_ulysses() -> Self {
        Self::with_origin(InvasionOrigin::StageStart, SylvesterKind::Ulysses)
    }

    pub fn new_bilaterus() -> Self {
        let mut wave = Self::with_origin(InvasionOrigin::StageStart, SylvesterKind::Balrog);
        wave.plan = WavePlan::FixedBilaterus;
        wave
    }

    pub fn new_tank1_finale() -> Self {
        let mut wave = Self::new_balrog();
        wave.plan = WavePlan::CyclingTank1Finale {
            next: EncounterKind::Single(SylvesterKind::Balrog),
        };
        wave
    }

    pub fn new_tank2_finale(first: SylvesterKind) -> Self {
        assert!(matches!(
            first,
            SylvesterKind::Gus | SylvesterKind::Destructor
        ));
        let mut wave = Self::with_origin(InvasionOrigin::StageStart, first);
        wave.plan = WavePlan::CyclingTank2Finale {
            next: EncounterKind::Single(first),
        };
        wave
    }

    pub fn new_tank3_second_stage(first: SylvesterKind) -> Self {
        assert!(matches!(
            first,
            SylvesterKind::Gus | SylvesterKind::Destructor
        ));
        let mut wave = Self::with_origin(InvasionOrigin::StageStart, first);
        wave.plan = WavePlan::CyclingTank3Second { next: first };
        wave
    }

    pub fn new_tank3_finale(first: SylvesterKind) -> Self {
        assert!(matches!(
            first,
            SylvesterKind::Ulysses | SylvesterKind::Psychosquid
        ));
        let mut wave = Self::with_origin(InvasionOrigin::StageStart, first);
        wave.plan = WavePlan::CyclingTank3Finale { next: first };
        wave
    }

    pub fn new_tank4_third_stage() -> Self {
        let mut wave = Self::new_gus();
        wave.plan = WavePlan::CyclingTank4Third {
            next: EncounterKind::Single(SylvesterKind::Gus),
        };
        wave
    }

    pub fn new_tank4_fourth_stage() -> Self {
        let mut wave = Self::new_bilaterus();
        wave.plan = WavePlan::CyclingTank4Fourth {
            next: EncounterKind::Bilaterus,
        };
        wave
    }

    pub fn new_tank4_finale() -> Self {
        let mut wave = Self::new_bilaterus();
        wave.plan = WavePlan::CyclingTank4Finale {
            next: EncounterKind::Bilaterus,
            wave_count: 0,
        };
        wave
    }

    pub fn legacy_v5_balrog_resume() -> Self {
        Self::with_origin(InvasionOrigin::LegacyV5Resume, SylvesterKind::Balrog)
    }

    pub fn legacy_v4_strong_resume() -> Self {
        Self::with_origin(InvasionOrigin::LegacyV4Resume, SylvesterKind::Strong)
    }

    pub fn legacy_v3_resume() -> Self {
        Self::with_origin(InvasionOrigin::LegacyV3Resume, SylvesterKind::Weak)
    }

    fn with_origin(origin: InvasionOrigin, kind: SylvesterKind) -> Self {
        Self {
            origin,
            plan: WavePlan::Fixed(kind),
            countdown: if kind == SylvesterKind::Weak {
                1750
            } else {
                3000
            },
            danger_shown: false,
            battle_tip_shown: false,
            gus_warning_shown: false,
            pending_modal: None,
            warning: None,
            actors: Vec::new(),
            bilaterus: Vec::new(),
            fragments: Vec::new(),
            battle_active: false,
            food_delay: 0,
            last_laser: None,
            post_spawn_flash_ticks: 0,
            warps: Vec::new(),
            lasers: Vec::new(),
            dead_aliens: Vec::new(),
        }
    }

    pub fn acknowledge_modal(&mut self) -> Option<InvasionTip> {
        self.pending_modal.take()
    }

    pub fn has_live_alien(&self) -> bool {
        !self.actors.is_empty() || !self.bilaterus.is_empty()
    }

    pub fn has_live_bilaterus(&self) -> bool {
        !self.bilaterus.is_empty()
    }

    pub fn bilaterus_by_id(&self, id: u64) -> Option<&BilaterusState> {
        self.bilaterus.iter().find(|group| group.id == id)
    }
    pub fn bilaterus_by_id_mut(&mut self, id: u64) -> Option<&mut BilaterusState> {
        self.bilaterus.iter_mut().find(|group| group.id == id)
    }

    /// Board commits each returned prey before calling the next child. This
    /// wrapper deliberately updates only one child, never an entire group.
    pub fn update_bilaterus_child(
        &mut self,
        id: u64,
        child: BilaterusChild,
        prey: &[BilaterusPreyView],
        mut next_random: impl FnMut() -> u32,
    ) -> Vec<InvasionEvent> {
        let Some(group) = self.bilaterus_by_id_mut(id) else {
            return Vec::new();
        };
        let eaten = match child {
            BilaterusChild::ActiveHead => group.update_active(prey, &mut next_random),
            BilaterusChild::Bone(index) => group.update_bone(usize::from(index), prey),
            BilaterusChild::PassiveHead => {
                group.update_passive();
                None
            }
        };
        eaten
            .map(|prey_id| {
                vec![InvasionEvent::BilaterusPreyEaten {
                    group_id: id,
                    prey_id,
                }]
            })
            .unwrap_or_default()
    }

    pub fn begin_bilaterus_update(&mut self, id: u64) -> (bool, Vec<InvasionEvent>) {
        let Some(group) = self.bilaterus_by_id_mut(id) else {
            return (false, Vec::new());
        };
        let (children_active, swapped) = group.begin_update_with_swap();
        let events = if swapped {
            vec![InvasionEvent::BilaterusHeadSwapped { id }]
        } else {
            Vec::new()
        };
        (children_active, events)
    }

    pub fn finish_bilaterus_update(
        &mut self,
        id: u64,
        next_random: impl FnMut() -> u32,
        next_id: impl FnMut() -> u64,
    ) -> Vec<InvasionEvent> {
        let transition = self
            .bilaterus_by_id_mut(id)
            .and_then(BilaterusState::finish_update);
        transition
            .map(|outcome| self.commit_bilaterus_transition(id, outcome, next_random, next_id))
            .unwrap_or_default()
    }

    pub fn pet_hit_bilaterus(&mut self, id: u64, amount: f64) -> Option<f64> {
        self.bilaterus_by_id_mut(id)
            .map(|group| group.pet_damage(amount))
    }

    /// This is the shot transaction only; the Board emits its shared laser
    /// once after trying the group and any ordinary projectile list.
    pub fn shoot_bilaterus(
        &mut self,
        id: u64,
        x: i32,
        y: i32,
        weapon: u8,
        next_random: impl FnMut() -> u32,
        next_id: impl FnMut() -> u64,
    ) -> Vec<InvasionEvent> {
        let Some(group) = self.bilaterus_by_id_mut(id) else {
            return Vec::new();
        };
        match group.shoot(x, y, weapon) {
            BilaterusShot::Miss => Vec::new(),
            BilaterusShot::Hit { health, transition } => {
                let mut events = vec![InvasionEvent::BilaterusHeadHit { id, health }];
                if let Some(outcome) = transition {
                    events.extend(self.commit_bilaterus_transition(
                        id,
                        outcome,
                        next_random,
                        next_id,
                    ));
                }
                events
            }
        }
    }

    pub fn commit_bilaterus_transition(
        &mut self,
        id: u64,
        outcome: BilaterusTransition,
        mut next_random: impl FnMut() -> u32,
        mut next_id: impl FnMut() -> u64,
    ) -> Vec<InvasionEvent> {
        let mut events = Vec::new();
        let spawns = match outcome {
            BilaterusTransition::FirstHeadLost { fragment } => {
                events.push(InvasionEvent::BilaterusFirstHeadDefeated { id });
                vec![fragment]
            }
            BilaterusTransition::Defeated {
                diamond_at,
                fragments,
            } => {
                self.bilaterus.retain(|group| group.id != id);
                events.push(InvasionEvent::BilaterusDefeated { id });
                events.push(InvasionEvent::DiamondDropped {
                    alien_id: id,
                    x: diamond_at.0,
                    y: diamond_at.1,
                });
                fragments.to_vec()
            }
        };
        for spec in spawns {
            let fragment_id = next_id();
            self.fragments.push(BilaterusFragment::spawn(
                fragment_id,
                spec,
                &mut next_random,
            ));
            events.push(InvasionEvent::BilaterusFragmentSpawned {
                id: fragment_id,
                kind: spec.kind,
            });
        }
        events
    }

    pub fn has_registered_kind(&self, kind: SylvesterKind) -> bool {
        self.actors.iter().any(|actor| actor.kind == kind)
    }

    pub fn actor_by_id(&self, id: u64) -> Option<&WeakSylvester> {
        self.actors.iter().find(|actor| actor.id == id)
    }

    pub fn actor_by_id_mut(&mut self, id: u64) -> Option<&mut WeakSylvester> {
        self.actors.iter_mut().find(|actor| actor.id == id)
    }

    /// Reflected raw1 projectile damage bypasses the ordinary click/weapon
    /// branch, but transfers defeated membership through the same reward path.
    pub fn reflected_energy_hit(&mut self, id: u64) -> Option<(f64, Vec<InvasionEvent>)> {
        let actor = self.actor_by_id_mut(id)?;
        actor.health -= 30.0;
        actor.hit_ticks = 10;
        let health = actor.health;
        let events = if health <= 0.0 {
            self.remove_registered_alien(id)
        } else {
            Vec::new()
        };
        Some((health, events))
    }

    /// Gash's contact calls the ordinary Alien death route immediately at
    /// zero HP, unlike the next-update death left by Itchy and Rufus contact.
    pub fn gash_hit_alien(&mut self, id: u64) -> Option<(f64, Vec<InvasionEvent>)> {
        let health = self.actor_by_id_mut(id)?.gash_hit()?;
        let events = if health <= 0.0 {
            self.remove_registered_alien(id)
        } else {
            Vec::new()
        };
        Some((health, events))
    }

    /// Board::Update reduces food delay before its first pause return at
    /// Board.cpp:610–615. Call this once for an already-paused board update;
    /// ordinary `board_update` calls it itself.
    pub fn update_before_pause(&mut self) {
        self.food_delay = self.food_delay.saturating_sub(1);
    }

    fn spawn_ordinary(
        &mut self,
        kind: SylvesterKind,
        x: i32,
        y: i32,
        next_random: &mut impl FnMut() -> u32,
        next_id: &mut impl FnMut() -> u64,
    ) -> InvasionEvent {
        let id = next_id();
        let actor = WeakSylvester::spawn_kind(kind, id, x, y, next_random(), next_random());
        let spawn_y = actor.widget_y;
        self.warps.push(WarpEffect {
            x: x + 30,
            y: spawn_y - 40,
            remaining_ticks: 36,
        });
        self.actors.push(actor);
        InvasionEvent::AlienSpawned { id, x, y: spawn_y }
    }

    fn spawn_bilaterus(
        &mut self,
        x: i32,
        y: i32,
        next_random: &mut impl FnMut() -> u32,
        next_id: &mut impl FnMut() -> u64,
    ) -> InvasionEvent {
        let id = next_id();
        let group = BilaterusState::spawn(id, x, y, next_random);
        let (x, y) = group.active_head_position();
        self.warps.push(WarpEffect {
            x: x - 10,
            y: y - 70,
            remaining_ticks: 36,
        });
        self.bilaterus.push(group);
        InvasionEvent::BilaterusSpawned { id, x, y }
    }

    /// Runs the board's wave transition after its clock advances and before
    /// entity updates. Warning consumes both coordinate pairs; each actor
    /// constructor then consumes its own source-derived draw sequence.
    pub fn board_update(
        &mut self,
        next_random: impl FnMut() -> u32,
        next_id: impl FnMut() -> u64,
    ) -> Vec<InvasionEvent> {
        self.update_before_pause();
        self.advance_after_pre_pause(next_random, next_id)
    }

    /// The caller has already performed the pre-pause delay decrement and
    /// its held-feeding check using the old board clock.
    pub fn advance_after_pre_pause(
        &mut self,
        next_random: impl FnMut() -> u32,
        next_id: impl FnMut() -> u64,
    ) -> Vec<InvasionEvent> {
        self.advance_with_threats(false, next_random, next_id)
    }

    pub fn advance_with_threats(
        &mut self,
        missiles_present: bool,
        mut next_random: impl FnMut() -> u32,
        mut next_id: impl FnMut() -> u64,
    ) -> Vec<InvasionEvent> {
        if self.pending_modal.is_some() {
            return Vec::new();
        }
        self.post_spawn_flash_ticks = self.post_spawn_flash_ticks.saturating_sub(1);
        if self.has_live_alien() || missiles_present {
            return Vec::new();
        }
        if self.countdown <= 0 {
            // A decoded state with no positive countdown is malformed;
            // never decrement it further or synthesize repeated spawns.
            return Vec::new();
        }

        self.countdown -= 1;
        match self.countdown {
            276 if self.plan == WavePlan::Fixed(SylvesterKind::Weak) => {
                let tip = if !self.danger_shown {
                    self.danger_shown = true;
                    Some(InvasionTip::Danger)
                } else if !self.battle_tip_shown {
                    self.battle_tip_shown = true;
                    Some(InvasionTip::BattleTip)
                } else {
                    None
                };
                if let Some(tip) = tip {
                    self.pending_modal = Some(tip);
                    vec![InvasionEvent::ModalOpened(tip)]
                } else {
                    Vec::new()
                }
            }
            276 if self.plan == WavePlan::Fixed(SylvesterKind::Gus) && !self.gus_warning_shown => {
                self.gus_warning_shown = true;
                self.pending_modal = Some(InvasionTip::GusWarning);
                vec![InvasionEvent::ModalOpened(InvasionTip::GusWarning)]
            }
            275 => {
                let coords = WarningCoords {
                    first_x: (next_random() % 450) as i32 + 20,
                    first_y: (next_random() % 195) as i32 + 105,
                    second_x: (next_random() % 450) as i32 + 20,
                    second_y: (next_random() % 195) as i32 + 105,
                };
                self.warning = Some(coords);
                vec![InvasionEvent::WarningStarted(coords)]
            }
            1 => vec![InvasionEvent::BattleMusicStarted],
            count if count <= 0 => {
                let Some(coords) = self.warning.take() else {
                    // A malformed save cannot produce a source-grounded
                    // spawn; retain zero so validation can reject it.
                    self.countdown = 0;
                    return Vec::new();
                };
                let encounter = self.plan.expected();
                let mut events = Vec::new();
                match encounter {
                    EncounterKind::Bilaterus => events.push(self.spawn_bilaterus(
                        coords.first_x,
                        coords.first_y,
                        &mut next_random,
                        &mut next_id,
                    )),
                    EncounterKind::Single(kind) => events.push(self.spawn_ordinary(
                        kind,
                        coords.first_x,
                        coords.first_y,
                        &mut next_random,
                        &mut next_id,
                    )),
                    EncounterKind::WeakBalrogPair
                    | EncounterKind::PsychosquidBalrogPair
                    | EncounterKind::DestructorUlyssesPair => {
                        let (first, second) = match encounter {
                            EncounterKind::WeakBalrogPair => {
                                (SylvesterKind::Weak, SylvesterKind::Balrog)
                            }
                            EncounterKind::PsychosquidBalrogPair => {
                                (SylvesterKind::Psychosquid, SylvesterKind::Balrog)
                            }
                            EncounterKind::DestructorUlyssesPair => {
                                (SylvesterKind::Destructor, SylvesterKind::Ulysses)
                            }
                            _ => unreachable!(),
                        };
                        events.push(self.spawn_ordinary(
                            first,
                            coords.first_x,
                            coords.first_y,
                            &mut next_random,
                            &mut next_id,
                        ));
                        events.push(self.spawn_ordinary(
                            second,
                            coords.second_x,
                            coords.second_y,
                            &mut next_random,
                            &mut next_id,
                        ));
                    }
                    EncounterKind::BalrogBilaterusPair => {
                        events.push(self.spawn_ordinary(
                            SylvesterKind::Balrog,
                            coords.first_x,
                            coords.first_y,
                            &mut next_random,
                            &mut next_id,
                        ));
                        events.push(self.spawn_bilaterus(
                            coords.second_x,
                            coords.second_y,
                            &mut next_random,
                            &mut next_id,
                        ));
                    }
                }
                // Board::Update selects the next encounter after every actor
                // constructor. Its else-if suppresses the %20 draw on a
                // successful %10 toggle.
                self.plan = match self.plan {
                    WavePlan::Fixed(kind) => WavePlan::Fixed(kind),
                    WavePlan::FixedBilaterus => WavePlan::FixedBilaterus,
                    WavePlan::CyclingTank1Finale { .. } => WavePlan::CyclingTank1Finale {
                        next: if !next_random().is_multiple_of(2) {
                            EncounterKind::Single(SylvesterKind::Balrog)
                        } else {
                            EncounterKind::WeakBalrogPair
                        },
                    },
                    WavePlan::CyclingTank2Finale { .. } => {
                        let mut next = if encounter == EncounterKind::WeakBalrogPair {
                            if !next_random().is_multiple_of(2) {
                                SylvesterKind::Gus
                            } else {
                                SylvesterKind::Destructor
                            }
                        } else if let EncounterKind::Single(kind) = encounter {
                            kind
                        } else {
                            unreachable!()
                        };
                        let next = if next_random().is_multiple_of(10) {
                            next = if next == SylvesterKind::Destructor {
                                SylvesterKind::Gus
                            } else {
                                SylvesterKind::Destructor
                            };
                            EncounterKind::Single(next)
                        } else if next_random().is_multiple_of(20) {
                            EncounterKind::WeakBalrogPair
                        } else {
                            EncounterKind::Single(next)
                        };
                        WavePlan::CyclingTank2Finale { next }
                    }
                    WavePlan::CyclingTank3Second { next } => WavePlan::CyclingTank3Second {
                        next: if next_random().is_multiple_of(10) {
                            if next == SylvesterKind::Gus {
                                SylvesterKind::Destructor
                            } else {
                                SylvesterKind::Gus
                            }
                        } else {
                            next
                        },
                    },
                    WavePlan::CyclingTank3Finale { .. } => WavePlan::CyclingTank3Finale {
                        // PB51: the actor was constructed above; the next
                        // expectation uses a separate inverted low-bit draw.
                        next: if next_random().is_multiple_of(2) {
                            SylvesterKind::Psychosquid
                        } else {
                            SylvesterKind::Ulysses
                        },
                    },
                    WavePlan::CyclingTank4Third { .. } => WavePlan::CyclingTank4Third {
                        // PB68 samples the next raw 4/10 only after the
                        // current actor constructors have consumed their RNG.
                        next: if next_random().is_multiple_of(2) {
                            EncounterKind::PsychosquidBalrogPair
                        } else {
                            EncounterKind::Single(SylvesterKind::Gus)
                        },
                    },
                    WavePlan::CyclingTank4Fourth { .. } => WavePlan::CyclingTank4Fourth {
                        // PB69: raw8 or raw11 is chosen after constructing
                        // the current group or both ordered ordinary actors.
                        next: if next_random().is_multiple_of(2) {
                            EncounterKind::DestructorUlyssesPair
                        } else {
                            EncounterKind::Bilaterus
                        },
                    },
                    WavePlan::CyclingTank4Finale { wave_count, .. } => {
                        let choice = next_random() % 5;
                        let next = match choice {
                            0 | 1 => EncounterKind::PsychosquidBalrogPair,
                            2 | 3 => EncounterKind::DestructorUlyssesPair,
                            4 if wave_count > 4 => EncounterKind::BalrogBilaterusPair,
                            4 => EncounterKind::Bilaterus,
                            _ => unreachable!(),
                        };
                        WavePlan::CyclingTank4Finale {
                            next,
                            wave_count: wave_count.saturating_add(1),
                        }
                    }
                };
                self.post_spawn_flash_ticks = 35;
                self.countdown = 3000;
                self.battle_active = true;
                events
            }
            _ => Vec::new(),
        }
    }

    /// Runs the already registered alien and finite visual entities. The
    /// caller supplies its object-order prey view and the shared RNG stream.
    pub fn objects_update(
        &mut self,
        prey: &[PreyView],
        next_random: impl FnMut() -> u32,
    ) -> Vec<InvasionEvent> {
        self.objects_update_with_food(prey, &[], next_random)
    }

    pub fn objects_update_with_food(
        &mut self,
        prey: &[PreyView],
        food: &[AlienFoodView],
        mut next_random: impl FnMut() -> u32,
    ) -> Vec<InvasionEvent> {
        self.objects_update_with_runtime(prey, food, |request| match request {
            AlienRuntimeRequest::Random => next_random(),
            AlienRuntimeRequest::Launch { .. } | AlienRuntimeRequest::ProbeTarget { .. } => 0,
        })
    }

    pub fn objects_update_with_runtime(
        &mut self,
        prey: &[PreyView],
        food: &[AlienFoodView],
        mut runtime: impl FnMut(AlienRuntimeRequest) -> u32,
    ) -> Vec<InvasionEvent> {
        if self.pending_modal.is_some() {
            return Vec::new();
        }
        assert!(
            self.actors.len() <= 1,
            "paired aliens require per-actor Board settlement"
        );
        let mut events = Vec::new();
        let ids: Vec<_> = self.actors.iter().map(|actor| actor.id).collect();
        for id in ids {
            events.extend(self.update_actor_with_runtime(id, prey, food, &mut runtime));
        }
        events.extend(self.update_effects());
        events.extend(self.finish_if_no_threats(false));
        events
    }

    /// Board calls this for each registered actor in source list order, then
    /// commits prey/food removal before preparing the next actor's view.
    pub fn update_actor_with_runtime(
        &mut self,
        id: u64,
        prey: &[PreyView],
        food: &[AlienFoodView],
        runtime: impl FnMut(AlienRuntimeRequest) -> u32,
    ) -> Vec<InvasionEvent> {
        if self.pending_modal.is_some() {
            return Vec::new();
        }
        let Some(actor) = self.actor_by_id_mut(id) else {
            return Vec::new();
        };
        let update = actor.update_with_runtime(prey, food, runtime);
        let mut events = Vec::new();
        if let Some((food_id, damage)) = update.food_eaten {
            events.push(InvasionEvent::GusAteFood {
                alien_id: id,
                food_id,
                damage,
            });
        }
        if let Some(prey_id) = update.prey_eaten {
            events.push(InvasionEvent::PreyEaten {
                alien_id: id,
                prey_id,
            });
        }
        if let Some(healing) = update.phase_changed {
            events.push(InvasionEvent::PsychosquidPhaseChanged {
                id,
                healing,
                forced: false,
            });
        }
        if update.defeated {
            events.extend(self.remove_registered_alien(id));
        }
        events
    }

    pub fn update_effects(&mut self) -> Vec<InvasionEvent> {
        if self.pending_modal.is_some() {
            return Vec::new();
        }
        let mut events = Vec::new();
        self.warps.retain(|warp| warp.remaining_ticks > 0);
        for warp in &mut self.warps {
            warp.remaining_ticks -= 1;
        }
        for laser in &mut self.lasers {
            laser.age_ticks += 1;
        }
        self.lasers.retain(|laser| laser.age_ticks <= 14);
        self.dead_aliens.retain(|body| body.remaining_ticks > 0);
        for body in &mut self.dead_aliens {
            body.remaining_ticks -= 1;
            if body.remaining_ticks < 105 {
                body.opacity = (body.opacity - 0.02).max(0.0);
            }
            if body.remaining_ticks % 7 == 0 && body.remaining_ticks > 50 {
                events.push(InvasionEvent::DeathBurstDue {
                    x: body.widget_x,
                    y: body.widget_y,
                });
            }
            if body.vx <= 0.0 {
                body.vx += 0.03;
            } else {
                body.vx -= 0.03;
            }
            if body.vy < 2.0 {
                body.vy += 0.05;
            }
            body.x = body.x.clamp(-10.0, 490.0) + body.vx;
            body.y += body.vy;
            body.widget_x = body.x as i32;
            body.widget_y = body.y as i32;
        }
        self.fragments.retain_mut(BilaterusFragment::tick);
        events
    }

    /// Handles the board's click priority around an ordinary live alien.
    /// The caller routes menu-area clicks separately but still suppresses
    /// feeding while `suppress_food` is true.
    pub fn click(&mut self, x: i32, y: i32) -> InvasionClick {
        self.click_with_weapon(x, y, 2)
    }

    pub fn click_with_weapon(&mut self, x: i32, y: i32, weapon: u8) -> InvasionClick {
        self.click_with_weapon_and_random(x, y, weapon, || 0)
    }

    pub fn click_with_weapon_and_random(
        &mut self,
        x: i32,
        y: i32,
        weapon: u8,
        next_random: impl FnMut() -> u32,
    ) -> InvasionClick {
        assert!(
            self.bilaterus.is_empty(),
            "Bilaterus click requires fragment IDs"
        );
        self.click_with_weapon_random_and_ids(x, y, weapon, next_random, || unreachable!())
    }

    /// Captures initial combat membership before damage so a lethal final
    /// click still suppresses feeding and emits one Board laser. Group heads
    /// are tried before ordinary Alien actors, retaining the shared RNG/ID
    /// stream for targetless death fragments.
    pub fn click_with_weapon_random_and_ids(
        &mut self,
        x: i32,
        y: i32,
        weapon: u8,
        mut next_random: impl FnMut() -> u32,
        mut next_id: impl FnMut() -> u64,
    ) -> InvasionClick {
        let mut events = Vec::new();
        if self.food_delay > 0
            && self.last_laser.is_some_and(|(last_x, last_y)| {
                let dx = i64::from(last_x) - i64::from(x);
                let dy = i64::from(last_y) - i64::from(y);
                dx * dx + dy * dy > 2500
            })
        {
            self.food_delay = 0;
            events.push(InvasionEvent::FoodDelayCleared);
        }
        let mut result = InvasionClick {
            suppress_food: self.has_live_alien() || self.food_delay > 0,
            events,
        };
        if y <= 40 {
            return result;
        }
        if !self.has_live_alien() {
            return result;
        }
        let group_ids: Vec<u64> = self.bilaterus.iter().map(|group| group.id).collect();
        let mut group_hit = false;
        for id in group_ids {
            let hit = self.shoot_bilaterus(id, x, y, weapon, &mut next_random, &mut next_id);
            if !hit.is_empty() {
                result.events.extend(hit);
                group_hit = true;
                break;
            }
        }
        let ordinary_count = if group_hit { 0 } else { self.actors.len() };
        for index in 0..ordinary_count {
            let alien_id = self.actors[index].id;
            let was_healing = self.actors[index].healing;
            let shot_result =
                self.actors[index].shot_with_weapon_and_random(x, y, weapon, &mut next_random);
            match shot_result {
                ShotResult::Miss => continue,
                ShotResult::Hit { health } => {
                    if was_healing {
                        result.events.push(InvasionEvent::PsychosquidHealingHit {
                            id: alien_id,
                            health,
                        });
                    } else {
                        result.events.push(InvasionEvent::AlienHit {
                            id: alien_id,
                            health,
                        });
                        if self.actors[index].healing {
                            result.events.push(InvasionEvent::PsychosquidPhaseChanged {
                                id: alien_id,
                                healing: true,
                                forced: true,
                            });
                        }
                    }
                    break;
                }
                ShotResult::Defeated { .. } => {
                    let health = self.actors[index].health;
                    result.events.push(InvasionEvent::AlienHit {
                        id: alien_id,
                        health,
                    });
                    result.events.extend(self.remove_registered_alien(alien_id));
                    break;
                }
            }
        }
        self.lasers.push(LaserEffect {
            x: x - 40,
            y: y - 40,
            age_ticks: 0,
        });
        self.last_laser = Some((x, y));
        result.events.push(InvasionEvent::LaserFired { x, y });
        result
    }

    /// Both a player shot and a completed alien update transfer ownership
    /// through this one removal path. The registered list is cleared before
    /// any subsequent pet contact, wave check, or click can reward it again.
    fn remove_registered_alien(&mut self, id: u64) -> Vec<InvasionEvent> {
        let index = self
            .actors
            .iter()
            .position(|actor| actor.id == id)
            .expect("alien removal requires registration");
        let dead = self.actors.remove(index);
        let body_x = dead.x as i32;
        let body_y = dead.y as i32;
        if dead.kind != SylvesterKind::Gus {
            self.dead_aliens.push(DeadAlienEffect {
                kind: dead.kind,
                x: f64::from(body_x),
                y: f64::from(body_y),
                widget_x: body_x,
                widget_y: body_y,
                vx: 0.0,
                vy: 0.0,
                frame: dead.frame,
                facing_right: dead.vx > 0.0,
                opacity: 1.0,
                remaining_ticks: 125,
            });
        }
        vec![
            InvasionEvent::AlienDefeated { id: dead.id },
            InvasionEvent::DiamondDropped {
                alien_id: dead.id,
                x: dead.widget_x + 25,
                y: dead.widget_y + 25,
            },
        ]
    }

    /// The board calls this after alien and missile transactions. Peaceful
    /// empty waves do not synthesize another end event.
    pub fn finish_if_no_threats(&mut self, missiles_present: bool) -> Vec<InvasionEvent> {
        if self.battle_active && !self.has_live_alien() && !missiles_present {
            self.battle_active = false;
            self.food_delay = 36;
            vec![InvasionEvent::BattleEnded]
        } else {
            Vec::new()
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        let fixed = match self.plan {
            WavePlan::Fixed(kind) => Some(kind),
            _ => None,
        };
        let fixed_weak = fixed == Some(SylvesterKind::Weak);
        let fixed_gus = fixed == Some(SylvesterKind::Gus);
        let actor_kinds_valid = match self.plan {
            WavePlan::Fixed(kind) => {
                self.bilaterus.is_empty()
                    && self.fragments.is_empty()
                    && self.actors.len() <= 1
                    && self.actors.iter().all(|actor| actor.kind == kind)
            }
            WavePlan::FixedBilaterus => {
                self.actors.is_empty()
                    && self.dead_aliens.is_empty()
                    && self.bilaterus.len() <= 1
                    && self.fragments.len() <= 8
            }
            WavePlan::CyclingTank1Finale { next } => {
                self.bilaterus.is_empty()
                    && self.fragments.is_empty()
                    && matches!(
                        next,
                        EncounterKind::Single(SylvesterKind::Balrog)
                            | EncounterKind::WeakBalrogPair
                    )
                    && matches!(
                        self.actors.as_slice(),
                        [] | [WeakSylvester {
                            kind: SylvesterKind::Balrog,
                            ..
                        }] | [WeakSylvester {
                            kind: SylvesterKind::Weak,
                            ..
                        }] | [
                            WeakSylvester {
                                kind: SylvesterKind::Weak,
                                ..
                            },
                            WeakSylvester {
                                kind: SylvesterKind::Balrog,
                                ..
                            }
                        ]
                    )
            }
            WavePlan::CyclingTank2Finale { next } => {
                self.bilaterus.is_empty()
                    && self.fragments.is_empty()
                    && matches!(
                        next,
                        EncounterKind::Single(SylvesterKind::Gus | SylvesterKind::Destructor)
                            | EncounterKind::WeakBalrogPair
                    )
                    && matches!(
                        self.actors.as_slice(),
                        [] | [WeakSylvester {
                            kind: SylvesterKind::Gus
                                | SylvesterKind::Destructor
                                | SylvesterKind::Weak
                                | SylvesterKind::Balrog,
                            ..
                        }] | [
                            WeakSylvester {
                                kind: SylvesterKind::Weak,
                                ..
                            },
                            WeakSylvester {
                                kind: SylvesterKind::Balrog,
                                ..
                            }
                        ]
                    )
            }
            WavePlan::CyclingTank3Second { next } => {
                self.bilaterus.is_empty()
                    && self.fragments.is_empty()
                    && matches!(next, SylvesterKind::Gus | SylvesterKind::Destructor)
                    && self.actors.len() <= 1
                    && self.actors.iter().all(|actor| {
                        matches!(actor.kind, SylvesterKind::Gus | SylvesterKind::Destructor)
                    })
            }
            WavePlan::CyclingTank3Finale { next } => {
                self.bilaterus.is_empty()
                    && self.fragments.is_empty()
                    && matches!(next, SylvesterKind::Ulysses | SylvesterKind::Psychosquid)
                    && self.actors.len() <= 1
                    && self.actors.iter().all(|actor| {
                        matches!(
                            actor.kind,
                            SylvesterKind::Ulysses | SylvesterKind::Psychosquid
                        )
                    })
            }
            WavePlan::CyclingTank4Third { next } => {
                self.bilaterus.is_empty()
                    && self.fragments.is_empty()
                    && matches!(
                        next,
                        EncounterKind::Single(SylvesterKind::Gus)
                            | EncounterKind::PsychosquidBalrogPair
                    )
                    && matches!(
                        self.actors.as_slice(),
                        [] | [WeakSylvester {
                            kind: SylvesterKind::Gus
                                | SylvesterKind::Psychosquid
                                | SylvesterKind::Balrog,
                            ..
                        }] | [
                            WeakSylvester {
                                kind: SylvesterKind::Psychosquid,
                                ..
                            },
                            WeakSylvester {
                                kind: SylvesterKind::Balrog,
                                ..
                            }
                        ]
                    )
            }
            WavePlan::CyclingTank4Fourth { next } => {
                matches!(
                    next,
                    EncounterKind::Bilaterus | EncounterKind::DestructorUlyssesPair
                ) && self.bilaterus.len() <= 1
                    && self.fragments.len() <= 8
                    && (self.bilaterus.is_empty() || self.actors.is_empty())
                    && matches!(
                        self.actors.as_slice(),
                        [] | [WeakSylvester {
                            kind: SylvesterKind::Destructor | SylvesterKind::Ulysses,
                            ..
                        }] | [
                            WeakSylvester {
                                kind: SylvesterKind::Destructor,
                                ..
                            },
                            WeakSylvester {
                                kind: SylvesterKind::Ulysses,
                                ..
                            }
                        ]
                    )
            }
            WavePlan::CyclingTank4Finale { next, wave_count } => {
                matches!(
                    next,
                    EncounterKind::Bilaterus
                        | EncounterKind::PsychosquidBalrogPair
                        | EncounterKind::DestructorUlyssesPair
                        | EncounterKind::BalrogBilaterusPair
                ) && (wave_count != 0 || next == EncounterKind::Bilaterus)
                    && (next != EncounterKind::BalrogBilaterusPair || wave_count >= 6)
                    && (wave_count != 0
                        || self.actors.is_empty()
                            && self.bilaterus.is_empty()
                            && self.fragments.is_empty()
                            && self.dead_aliens.is_empty()
                            && !self.battle_active)
                    && self.bilaterus.len() <= 1
                    && self.fragments.len() <= 8
                    && (self.bilaterus.is_empty()
                        || matches!(
                            self.actors.as_slice(),
                            [] | [WeakSylvester {
                                kind: SylvesterKind::Balrog,
                                ..
                            }]
                        ))
                    && matches!(
                        self.actors.as_slice(),
                        [] | [WeakSylvester {
                            kind: SylvesterKind::Psychosquid
                                | SylvesterKind::Balrog
                                | SylvesterKind::Destructor
                                | SylvesterKind::Ulysses,
                            ..
                        }] | [
                            WeakSylvester {
                                kind: SylvesterKind::Psychosquid,
                                ..
                            },
                            WeakSylvester {
                                kind: SylvesterKind::Balrog,
                                ..
                            }
                        ] | [
                            WeakSylvester {
                                kind: SylvesterKind::Destructor,
                                ..
                            },
                            WeakSylvester {
                                kind: SylvesterKind::Ulysses,
                                ..
                            }
                        ]
                    )
            }
        };
        if !(0..=3000).contains(&self.countdown)
            || self.food_delay > 36
            || self.post_spawn_flash_ticks > 35
            || self.pending_modal == Some(InvasionTip::Danger) && !self.danger_shown
            || self.pending_modal == Some(InvasionTip::BattleTip) && !self.battle_tip_shown
            || self.pending_modal == Some(InvasionTip::GusWarning) && !self.gus_warning_shown
            || self.pending_modal.is_some() && self.countdown != 276
            || self.countdown == 0
            || (1..=275).contains(&self.countdown) && self.warning.is_none()
            || self.warning.is_some() && !(1..=275).contains(&self.countdown)
            || !self.actors.is_empty() && self.countdown != 3000
            || !self.actors.is_empty() && !self.battle_active
            || !self.bilaterus.is_empty() && self.countdown != 3000
            || !self.bilaterus.is_empty() && !self.battle_active
            || self.actors.iter().any(|actor| !actor.alive)
            || !actor_kinds_valid
            || !fixed_weak && (self.danger_shown || self.battle_tip_shown)
            || !fixed_gus && self.gus_warning_shown
            || !fixed_weak && !fixed_gus && self.pending_modal.is_some()
            || fixed_weak && self.pending_modal == Some(InvasionTip::GusWarning)
            || fixed_gus
                && matches!(
                    self.pending_modal,
                    Some(InvasionTip::Danger | InvasionTip::BattleTip)
                )
            || self.warps.len() > 2
            || self.warps.iter().any(|warp| warp.remaining_ticks > 36)
            || self.lasers.iter().any(|laser| laser.age_ticks > 14)
            || self.dead_aliens.len() > 2
            || self.dead_aliens.iter().any(|body| {
                body.kind == SylvesterKind::Gus
                    || fixed.is_some_and(|kind| body.kind != kind)
                    || body.remaining_ticks > 125
                    || body.frame > 9
                    || !body.x.is_finite()
                    || !body.y.is_finite()
                    || !body.vx.is_finite()
                    || !body.vy.is_finite()
                    || !body.opacity.is_finite()
                    || !(0.0..=1.0).contains(&body.opacity)
            })
            || self.warning.is_some_and(|coords| {
                !(20..=469).contains(&coords.first_x)
                    || !(105..=299).contains(&coords.first_y)
                    || !(20..=469).contains(&coords.second_x)
                    || !(105..=299).contains(&coords.second_y)
            })
        {
            return Err("invalid Adventure 1-2 invasion state".into());
        }
        for actor in &self.actors {
            actor.validate()?;
        }
        for group in &self.bilaterus {
            group.validate()?;
        }
        for fragment in &self.fragments {
            fragment.validate()?;
        }
        Ok(())
    }
}

impl Default for Invasion1_2 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bilaterus_wave_consumes_all_constructor_draws_and_fragments_do_not_hold_battle() {
        let mut wave = Invasion1_2::new_bilaterus();
        wave.warning = Some(WarningCoords {
            first_x: 105,
            first_y: 160,
            second_x: 410,
            second_y: 290,
        });
        wave.countdown = 1;
        let mut draws = 0;
        let events = wave.board_update(
            || {
                draws += 1;
                1
            },
            || 91,
        );
        assert_eq!(draws, 11);
        assert_eq!(
            events,
            [InvasionEvent::BilaterusSpawned {
                id: 91,
                x: 105,
                y: 160
            }]
        );
        assert!(wave.has_live_alien());
        assert!(wave.has_live_bilaterus());
        assert!(wave.actors.is_empty());
        wave.validate().unwrap();
        let mut group = wave.bilaterus.pop().unwrap();
        group.first_head_lost = true;
        group.heads[0] = None;
        group.active_head = 1;
        group.emergence_ticks = 0;
        group.heads[1].as_mut().unwrap().health = 0.0;
        wave.bilaterus.push(group);
        let transition = wave.bilaterus[0].finish_update();
        let mut next_id = 100;
        let end = wave.commit_bilaterus_transition(
            91,
            transition.unwrap(),
            || 1,
            || {
                next_id += 1;
                next_id
            },
        );
        assert!(
            end.iter()
                .any(|event| matches!(event, InvasionEvent::DiamondDropped { alien_id: 91, .. }))
        );
        assert_eq!(wave.fragments.len(), 7);
        assert!(!wave.has_live_alien());
        assert_eq!(
            wave.finish_if_no_threats(false),
            [InvasionEvent::BattleEnded]
        );
        assert_eq!(wave.fragments.len(), 7); // visual tail does not hold liveness
        assert!(wave.finish_if_no_threats(false).is_empty());
    }

    #[test]
    fn final_group_click_keeps_initial_combat_suppression_and_one_laser() {
        let mut wave = Invasion1_2::new_bilaterus();
        let mut group = BilaterusState::spawn(71, 100, 120, &mut || 1);
        group.first_head_lost = true;
        group.heads[0] = None;
        group.active_head = 1;
        group.active_mut().health = 26.0;
        group.emergence_ticks = 0;
        wave.bilaterus.push(group);
        wave.battle_active = true;
        let mut id = 80;
        let click = wave.click_with_weapon_random_and_ids(
            120,
            140,
            12,
            || 1,
            || {
                id += 1;
                id
            },
        );
        assert!(click.suppress_food);
        assert_eq!(
            click
                .events
                .iter()
                .filter(|e| matches!(e, InvasionEvent::LaserFired { .. }))
                .count(),
            1
        );
        assert_eq!(
            click
                .events
                .iter()
                .filter(|e| matches!(e, InvasionEvent::DiamondDropped { .. }))
                .count(),
            1
        );
        assert!(!wave.has_live_alien());
        assert_eq!(wave.fragments.len(), 7);
    }

    #[test]
    fn periodic_head_swap_event_precedes_children_and_never_draws_rng() {
        let mut wave = Invasion1_2::new_bilaterus();
        let mut group = BilaterusState::spawn(71, 100, 120, &mut || 1);
        group.emergence_ticks = 0;
        group.swap_ticks = 999;
        group.bones[0].widget_x = 101;
        group.bones[5].widget_x = 106;
        wave.bilaterus.push(group);
        wave.battle_active = true;
        let (children, events) = wave.begin_bilaterus_update(71);
        assert!(children);
        assert_eq!(events, [InvasionEvent::BilaterusHeadSwapped { id: 71 }]);
        assert_eq!(wave.bilaterus[0].active_head, 1);
        assert_eq!(wave.bilaterus[0].bones[0].widget_x, 106);
        let (_, following) = wave.begin_bilaterus_update(71);
        assert!(following.is_empty());
    }

    #[test]
    fn tank3_finale_spawns_current_kind_before_independent_inverted_bit() {
        for (bit, expected_next) in [(0, SylvesterKind::Psychosquid), (1, SylvesterKind::Ulysses)] {
            let mut wave = Invasion1_2::new_tank3_finale(SylvesterKind::Ulysses);
            wave.warning = Some(WarningCoords {
                first_x: 100,
                first_y: 140,
                second_x: 200,
                second_y: 220,
            });
            wave.countdown = 1;
            let mut draws = [1, 4, bit].into_iter();
            let events = wave.board_update(|| draws.next().expect("exactly three draws"), || 7);
            assert_eq!(draws.next(), None);
            assert_eq!(
                events,
                [InvasionEvent::AlienSpawned {
                    id: 7,
                    x: 100,
                    y: 280
                }]
            );
            assert_eq!(wave.actors[0].kind, SylvesterKind::Ulysses);
            assert_eq!(wave.plan.expected(), EncounterKind::Single(expected_next));
            assert_eq!(wave.countdown, 3000);
            wave.validate().unwrap();
        }
    }

    #[test]
    fn tank4_third_samples_next_only_after_current_gus_constructor() {
        for (bit, expected_next) in [
            (0, EncounterKind::PsychosquidBalrogPair),
            (1, EncounterKind::Single(SylvesterKind::Gus)),
        ] {
            let mut wave = Invasion1_2::new_tank4_third_stage();
            assert_eq!(
                wave.plan.expected(),
                EncounterKind::Single(SylvesterKind::Gus)
            );
            wave.warning = Some(WarningCoords {
                first_x: 100,
                first_y: 140,
                second_x: 300,
                second_y: 220,
            });
            wave.countdown = 1;
            let mut draws = [1, 4, bit].into_iter();
            let events = wave.board_update(|| draws.next().expect("three draws"), || 70);
            assert_eq!(draws.next(), None);
            assert!(matches!(
                events.as_slice(),
                [InvasionEvent::AlienSpawned { id: 70, .. }]
            ));
            assert_eq!(wave.actors[0].kind, SylvesterKind::Gus);
            assert_eq!(wave.plan.expected(), expected_next);
            assert_eq!(wave.countdown, 3000);
            wave.validate().unwrap();
        }
    }

    #[test]
    fn tank4_third_pair_registers_source_order_and_retains_battle_until_both_leave() {
        let mut wave = Invasion1_2::new_tank4_third_stage();
        wave.plan = WavePlan::CyclingTank4Third {
            next: EncounterKind::PsychosquidBalrogPair,
        };
        wave.warning = Some(WarningCoords {
            first_x: 100,
            first_y: 140,
            second_x: 300,
            second_y: 220,
        });
        wave.countdown = 1;
        let mut draws = [1, 4, 2, 5, 1].into_iter();
        let mut ids = [71, 72].into_iter();
        let events = wave.board_update(
            || draws.next().expect("both constructors then one choice"),
            || ids.next().expect("one ID per actor"),
        );
        assert_eq!(draws.next(), None);
        assert_eq!(ids.next(), None);
        assert_eq!(
            wave.plan.expected(),
            EncounterKind::Single(SylvesterKind::Gus)
        );
        assert!(matches!(
            events.as_slice(),
            [
                InvasionEvent::AlienSpawned { id: 71, .. },
                InvasionEvent::AlienSpawned { id: 72, .. }
            ]
        ));
        assert_eq!(
            wave.actors
                .iter()
                .map(|actor| actor.kind)
                .collect::<Vec<_>>(),
            [SylvesterKind::Psychosquid, SylvesterKind::Balrog]
        );
        assert_eq!(wave.actors[0].widget_x, 100);
        assert_eq!(wave.actors[1].widget_x, 300);
        let encoded = serde_json::to_vec(&wave).unwrap();
        let restored: Invasion1_2 = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(restored.plan, wave.plan);
        restored.validate().unwrap();
        let mut reversed = restored.clone();
        reversed.actors.swap(0, 1);
        assert!(reversed.validate().is_err());
        assert!(wave.finish_if_no_threats(false).is_empty());
        let first = wave.remove_registered_alien(71);
        assert!(
            first
                .iter()
                .any(|event| matches!(event, InvasionEvent::DiamondDropped { alien_id: 71, .. }))
        );
        assert_eq!(wave.actors[0].id, 72);
        assert!(wave.finish_if_no_threats(false).is_empty());
        wave.validate().unwrap();
        let second = wave.remove_registered_alien(72);
        assert!(
            second
                .iter()
                .any(|event| matches!(event, InvasionEvent::DiamondDropped { alien_id: 72, .. }))
        );
        assert_eq!(wave.dead_aliens.len(), 2);
        assert_eq!(
            wave.finish_if_no_threats(false),
            [InvasionEvent::BattleEnded]
        );
        assert!(wave.finish_if_no_threats(false).is_empty());
        wave.validate().unwrap();
    }

    #[test]
    fn tank4_fourth_spawns_bilaterus_before_one_next_choice_draw() {
        for (bit, expected_next) in [
            (0, EncounterKind::DestructorUlyssesPair),
            (1, EncounterKind::Bilaterus),
        ] {
            let mut wave = Invasion1_2::new_tank4_fourth_stage();
            assert_eq!(wave.plan.expected(), EncounterKind::Bilaterus);
            wave.warning = Some(WarningCoords {
                first_x: 105,
                first_y: 160,
                second_x: 410,
                second_y: 290,
            });
            wave.countdown = 1;
            let mut draws = 0;
            let events = wave.board_update(
                || {
                    draws += 1;
                    if draws == 12 { bit } else { 1 }
                },
                || 91,
            );
            assert_eq!(draws, 12); // Eleven group draws, then PB69's choice.
            assert_eq!(
                events,
                [InvasionEvent::BilaterusSpawned {
                    id: 91,
                    x: 105,
                    y: 160
                }]
            );
            assert_eq!(wave.plan.expected(), expected_next);
            assert_eq!(wave.countdown, 3000);
            assert!(wave.has_live_bilaterus());
            wave.validate().unwrap();

            let mut group = wave.bilaterus.pop().unwrap();
            group.first_head_lost = true;
            group.heads[0] = None;
            group.active_head = 1;
            group.emergence_ticks = 0;
            group.heads[1].as_mut().unwrap().health = 0.0;
            wave.bilaterus.push(group);
            let transition = wave.bilaterus[0].finish_update().unwrap();
            let mut fragment_id = 100;
            let end = wave.commit_bilaterus_transition(
                91,
                transition,
                || 1,
                || {
                    fragment_id += 1;
                    fragment_id
                },
            );
            assert_eq!(
                end.iter()
                    .filter(|event| matches!(
                        event,
                        InvasionEvent::DiamondDropped { alien_id: 91, .. }
                    ))
                    .count(),
                1
            );
            assert!(!wave.has_live_alien());
            assert!(wave.finish_if_no_threats(true).is_empty());
            assert!(
                wave.advance_with_threats(
                    true,
                    || panic!("missiles hold countdown"),
                    || panic!("missiles cannot spawn actors")
                )
                .is_empty()
            );
            assert_eq!(wave.countdown, 3000);
            assert_eq!(
                wave.finish_if_no_threats(false),
                [InvasionEvent::BattleEnded]
            );
            assert!(wave.finish_if_no_threats(false).is_empty());
            wave.validate().unwrap();
        }
    }

    #[test]
    fn tank4_fourth_pair_keeps_constructor_order_rewards_and_survivor_liveness() {
        let mut wave = Invasion1_2::new_tank4_fourth_stage();
        wave.plan = WavePlan::CyclingTank4Fourth {
            next: EncounterKind::DestructorUlyssesPair,
        };
        wave.warning = Some(WarningCoords {
            first_x: 100,
            first_y: 140,
            second_x: 300,
            second_y: 220,
        });
        wave.countdown = 1;
        let mut draws = [1, 4, 2, 5, 1].into_iter();
        let mut ids = [71, 72].into_iter();
        let events = wave.board_update(
            || draws.next().expect("both constructors then one choice"),
            || ids.next().expect("one ID per actor"),
        );
        assert_eq!(draws.next(), None);
        assert_eq!(ids.next(), None);
        assert_eq!(wave.plan.expected(), EncounterKind::Bilaterus);
        assert!(matches!(
            events.as_slice(),
            [
                InvasionEvent::AlienSpawned {
                    id: 71,
                    x: 100,
                    y: 280
                },
                InvasionEvent::AlienSpawned {
                    id: 72,
                    x: 300,
                    y: 280
                }
            ]
        ));
        assert_eq!(
            wave.actors
                .iter()
                .map(|actor| actor.kind)
                .collect::<Vec<_>>(),
            [SylvesterKind::Destructor, SylvesterKind::Ulysses]
        );
        let encoded = serde_json::to_vec(&wave).unwrap();
        let restored: Invasion1_2 = serde_json::from_slice(&encoded).unwrap();
        restored.validate().unwrap();
        let mut reversed = restored.clone();
        reversed.actors.swap(0, 1);
        assert!(reversed.validate().is_err());
        let mut wrong_next = restored.clone();
        wrong_next.plan = WavePlan::CyclingTank4Fourth {
            next: EncounterKind::Single(SylvesterKind::Gus),
        };
        assert!(wrong_next.validate().is_err());
        let mut mixed = restored;
        mixed
            .bilaterus
            .push(BilaterusState::spawn(73, 110, 160, &mut || 1));
        assert!(mixed.validate().is_err());

        assert!(wave.finish_if_no_threats(false).is_empty());
        let first = wave.remove_registered_alien(71);
        assert!(
            first
                .iter()
                .any(|event| matches!(event, InvasionEvent::DiamondDropped { alien_id: 71, .. }))
        );
        assert_eq!(wave.actors[0].id, 72);
        assert!(wave.finish_if_no_threats(false).is_empty());
        wave.validate().unwrap();
        let second = wave.remove_registered_alien(72);
        assert!(
            second
                .iter()
                .any(|event| matches!(event, InvasionEvent::DiamondDropped { alien_id: 72, .. }))
        );
        assert_eq!(wave.dead_aliens.len(), 2);
        assert!(wave.finish_if_no_threats(true).is_empty());
        assert_eq!(
            wave.finish_if_no_threats(false),
            [InvasionEvent::BattleEnded]
        );
        assert!(wave.finish_if_no_threats(false).is_empty());
        wave.validate().unwrap();
    }

    #[test]
    fn tank4_finale_samples_one_modulo_five_choice_after_group_and_uses_old_wave_count() {
        for (old_count, roll, expected) in [
            (0, 0, EncounterKind::PsychosquidBalrogPair),
            (0, 1, EncounterKind::PsychosquidBalrogPair),
            (0, 2, EncounterKind::DestructorUlyssesPair),
            (0, 3, EncounterKind::DestructorUlyssesPair),
            (4, 4, EncounterKind::Bilaterus),
            (5, 4, EncounterKind::BalrogBilaterusPair),
        ] {
            let mut wave = Invasion1_2::new_tank4_finale();
            assert_eq!(wave.plan.expected(), EncounterKind::Bilaterus);
            wave.plan = WavePlan::CyclingTank4Finale {
                next: EncounterKind::Bilaterus,
                wave_count: old_count,
            };
            wave.warning = Some(WarningCoords {
                first_x: 105,
                first_y: 160,
                second_x: 410,
                second_y: 290,
            });
            wave.countdown = 1;
            let mut draws = 0;
            let events = wave.board_update(
                || {
                    draws += 1;
                    if draws == 12 { roll } else { 1 }
                },
                || 91,
            );
            assert_eq!(draws, 12); // Existing group consumes eleven; choice consumes one.
            assert_eq!(
                events,
                [InvasionEvent::BilaterusSpawned {
                    id: 91,
                    x: 105,
                    y: 160
                }]
            );
            assert_eq!(wave.plan.expected(), expected);
            assert!(
                matches!(wave.plan, WavePlan::CyclingTank4Finale { wave_count, .. }
                if wave_count == old_count + 1)
            );
            wave.validate().unwrap();
        }
    }

    #[test]
    fn tank4_finale_raw_twelve_spawns_balrog_then_second_coordinate_group() {
        let mut wave = Invasion1_2::new_tank4_finale();
        wave.plan = WavePlan::CyclingTank4Finale {
            next: EncounterKind::BalrogBilaterusPair,
            wave_count: 6,
        };
        wave.warning = Some(WarningCoords {
            first_x: 100,
            first_y: 140,
            second_x: 300,
            second_y: 220,
        });
        wave.countdown = 1;
        let mut draws = 0;
        let mut ids = [71, 72].into_iter();
        let events = wave.board_update(
            || {
                draws += 1;
                if draws == 14 { 4 } else { 1 }
            },
            || ids.next().expect("ordinary then group IDs"),
        );
        assert_eq!(draws, 14); // Balrog2 + Bilaterus11 + one next choice.
        assert_eq!(ids.next(), None);
        assert_eq!(
            events,
            [
                InvasionEvent::AlienSpawned {
                    id: 71,
                    x: 100,
                    y: 140
                },
                InvasionEvent::BilaterusSpawned {
                    id: 72,
                    x: 300,
                    y: 220
                },
            ]
        );
        assert_eq!(wave.actors[0].kind, SylvesterKind::Balrog);
        assert_eq!(wave.plan.expected(), EncounterKind::BalrogBilaterusPair);
        assert_eq!(wave.countdown, 3000);
        wave.validate().unwrap();
        let encoded = serde_json::to_vec(&wave).unwrap();
        let restored: Invasion1_2 = serde_json::from_slice(&encoded).unwrap();
        restored.validate().unwrap();
        let mut invalid = restored.clone();
        invalid.actors.push(WeakSylvester::spawn_kind(
            SylvesterKind::Psychosquid,
            73,
            150,
            150,
            1,
            1,
        ));
        assert!(invalid.validate().is_err());
        let mut early = restored;
        early.plan = WavePlan::CyclingTank4Finale {
            next: EncounterKind::BalrogBilaterusPair,
            wave_count: 5,
        };
        assert!(early.validate().is_err());

        assert!(wave.finish_if_no_threats(false).is_empty());
        let first = wave.remove_registered_alien(71);
        assert_eq!(
            first
                .iter()
                .filter(|event| matches!(event, InvasionEvent::DiamondDropped { alien_id: 71, .. }))
                .count(),
            1
        );
        assert!(wave.has_live_bilaterus());
        assert!(wave.finish_if_no_threats(false).is_empty());
        wave.validate().unwrap();
        let mut group = wave.bilaterus.pop().unwrap();
        group.first_head_lost = true;
        group.heads[0] = None;
        group.active_head = 1;
        group.emergence_ticks = 0;
        group.heads[1].as_mut().unwrap().health = 0.0;
        wave.bilaterus.push(group);
        let transition = wave.bilaterus[0].finish_update().unwrap();
        let mut fragment_id = 100;
        let second = wave.commit_bilaterus_transition(
            72,
            transition,
            || 1,
            || {
                fragment_id += 1;
                fragment_id
            },
        );
        assert_eq!(
            second
                .iter()
                .filter(|event| matches!(event, InvasionEvent::DiamondDropped { alien_id: 72, .. }))
                .count(),
            1
        );
        assert!(!wave.has_live_alien());
        assert!(wave.finish_if_no_threats(true).is_empty());
        assert_eq!(
            wave.finish_if_no_threats(false),
            [InvasionEvent::BattleEnded]
        );
        wave.validate().unwrap();
    }

    #[test]
    fn tank4_finale_existing_pairs_keep_order_and_sample_after_both_constructors() {
        for (current, first, second) in [
            (
                EncounterKind::PsychosquidBalrogPair,
                SylvesterKind::Psychosquid,
                SylvesterKind::Balrog,
            ),
            (
                EncounterKind::DestructorUlyssesPair,
                SylvesterKind::Destructor,
                SylvesterKind::Ulysses,
            ),
        ] {
            let mut wave = Invasion1_2::new_tank4_finale();
            wave.plan = WavePlan::CyclingTank4Finale {
                next: current,
                wave_count: 2,
            };
            wave.warning = Some(WarningCoords {
                first_x: 100,
                first_y: 140,
                second_x: 300,
                second_y: 220,
            });
            wave.countdown = 1;
            let mut draws = [1, 4, 2, 5, 3].into_iter();
            let mut ids = [71, 72].into_iter();
            let events = wave.board_update(
                || {
                    draws
                        .next()
                        .expect("two ordinary constructors then next choice")
                },
                || ids.next().expect("one ID per actor"),
            );
            assert_eq!(draws.next(), None);
            assert_eq!(ids.next(), None);
            assert!(matches!(
                events.as_slice(),
                [
                    InvasionEvent::AlienSpawned { id: 71, x: 100, .. },
                    InvasionEvent::AlienSpawned { id: 72, x: 300, .. }
                ]
            ));
            assert_eq!(
                wave.actors
                    .iter()
                    .map(|actor| actor.kind)
                    .collect::<Vec<_>>(),
                [first, second]
            );
            assert_eq!(wave.plan.expected(), EncounterKind::DestructorUlyssesPair);
            assert!(matches!(
                wave.plan,
                WavePlan::CyclingTank4Finale { wave_count: 3, .. }
            ));
            wave.validate().unwrap();
        }
    }

    #[test]
    fn gash_contact_removes_only_lethal_ordinary_alien_once() {
        let mut wave = Invasion1_2::new_tank4_fourth_stage();
        let mut destructor =
            WeakSylvester::spawn_kind(SylvesterKind::Destructor, 81, 100, 140, 1, 1);
        destructor.health = 1.0;
        destructor.spawn_ticks = 0;
        wave.actors.push(destructor);
        wave.battle_active = true;
        assert_eq!(wave.gash_hit_alien(81), Some((0.5, Vec::new())));
        assert_eq!(wave.actors.len(), 1);
        let (health, events) = wave.gash_hit_alien(81).unwrap();
        assert_eq!(health, 0.0);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, InvasionEvent::AlienDefeated { id: 81 }))
                .count(),
            1
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, InvasionEvent::DiamondDropped { alien_id: 81, .. }))
                .count(),
            1
        );
        assert!(wave.actors.is_empty());
        assert_eq!(wave.dead_aliens.len(), 1);
        assert_eq!(wave.gash_hit_alien(81), None);

        let mut healing = WeakSylvester::spawn_kind(SylvesterKind::Psychosquid, 82, 200, 160, 1, 1);
        healing.healing = true;
        let before = healing.health;
        let mut healing_wave = Invasion1_2::new_psychosquid();
        healing_wave.actors.push(healing);
        assert_eq!(healing_wave.gash_hit_alien(82), None);
        assert_eq!(healing_wave.actors[0].health, before);
    }

    #[test]
    fn psychosquid_healing_shot_reports_only_heal_event_and_timed_exit_reports_phase() {
        let mut wave = Invasion1_2::new_psychosquid();
        let mut actor = WeakSylvester::spawn_kind(SylvesterKind::Psychosquid, 201, 100, 120, 1, 1);
        actor.spawn_ticks = 0;
        actor.healing = true;
        actor.ever_healed = true;
        actor.movement_divisor = 2.0;
        actor.phase_ticks = 399;
        wave.actors.push(actor);
        wave.battle_active = true;
        let shot = wave.click_with_weapon_and_random(110, 130, 2, || {
            panic!("accepted healing shot draws no RNG")
        });
        assert!(shot.events.iter().any(|event| matches!(
            event,
            InvasionEvent::PsychosquidHealingHit {
                id: 201,
                health: 266.0
            }
        )));
        assert!(
            !shot
                .events
                .iter()
                .any(|event| matches!(event, InvasionEvent::AlienHit { .. }))
        );
        let events = wave.update_actor_with_runtime(201, &[], &[], |_| 41);
        assert!(events.iter().any(|event| matches!(
            event,
            InvasionEvent::PsychosquidPhaseChanged {
                id: 201,
                healing: false,
                forced: false
            }
        )));
        assert_eq!(wave.actors[0].movement_divisor, 0.5);
    }

    #[test]
    fn first_warning_pauses_at_276_then_consumes_all_four_coordinate_draws() {
        let mut wave = Invasion1_2::new();
        assert_eq!(wave.countdown, 1750);
        wave.countdown = 277;
        assert_eq!(
            wave.board_update(|| panic!("no random draw at modal"), || panic!("no actor")),
            vec![InvasionEvent::ModalOpened(InvasionTip::Danger)]
        );
        assert_eq!(wave.countdown, 276);
        assert_eq!(
            wave.board_update(|| panic!("paused"), || panic!("paused")),
            Vec::new()
        );
        assert_eq!(wave.countdown, 276);
        assert_eq!(wave.acknowledge_modal(), Some(InvasionTip::Danger));
        let mut draws = [0_u32, 194, 449, 0].into_iter();
        assert_eq!(
            wave.board_update(|| draws.next().unwrap(), || panic!("no actor yet")),
            vec![InvasionEvent::WarningStarted(WarningCoords {
                first_x: 20,
                first_y: 299,
                second_x: 469,
                second_y: 105,
            })]
        );
        assert_eq!(draws.next(), None);
        wave.validate().unwrap();
    }

    #[test]
    fn destructor_warning_y_does_not_relocate_actor_or_its_spawn_warp() {
        let mut wave = Invasion1_2::new_destructor();
        wave.warning = Some(WarningCoords {
            first_x: 100,
            first_y: 120,
            second_x: 200,
            second_y: 220,
        });
        wave.countdown = 1;
        let events = wave.board_update(|| 1, || 77);
        assert_eq!(
            events,
            vec![InvasionEvent::AlienSpawned {
                id: 77,
                x: 100,
                y: 280
            }]
        );
        assert_eq!(wave.actors[0].widget_y, 280);
        assert_eq!(wave.warps[0].y, 240);
    }

    #[test]
    fn gus_warning_uses_one_modal_then_four_warning_draws() {
        let mut wave = Invasion1_2::new_gus();
        assert_eq!(wave.countdown, 3000);
        wave.countdown = 277;
        assert_eq!(
            wave.board_update(|| panic!("no RNG at modal"), || panic!("no spawn")),
            vec![InvasionEvent::ModalOpened(InvasionTip::GusWarning)]
        );
        assert_eq!(wave.countdown, 276);
        wave.acknowledge_modal();
        let mut draws = 0;
        wave.board_update(
            || {
                draws += 1;
                0
            },
            || panic!("no spawn"),
        );
        assert_eq!((wave.countdown, draws), (275, 4));
        assert!(wave.gus_warning_shown);
    }

    #[test]
    fn gus_food_and_prey_transaction_precedes_single_diamond_without_corpse() {
        let mut wave = Invasion1_2::new_gus();
        let mut actor = WeakSylvester::spawn_kind(SylvesterKind::Gus, 77, 100, 100, 1, 1);
        actor.spawn_ticks = 0;
        actor.chase_ticks = 0;
        actor.health = 4.0;
        wave.actors.push(actor);
        wave.battle_active = true;
        let food = [AlienFoodView {
            id: 78,
            widget_x: 120,
            widget_y: 120,
            quality: 0,
            eligible: true,
        }];
        let prey = [PreyView {
            id: 79,
            widget_x: 140,
            widget_y: 140,
            width: 80,
            height: 80,
            eligible: true,
        }];
        let events = wave.objects_update_with_food(&prey, &food, || 1);
        assert!(matches!(
            events.as_slice(),
            [
                InvasionEvent::GusAteFood {
                    food_id: 78,
                    damage: 4,
                    ..
                },
                InvasionEvent::PreyEaten { prey_id: 79, .. },
                InvasionEvent::AlienDefeated { id: 77 },
                InvasionEvent::DiamondDropped { alien_id: 77, .. },
                InvasionEvent::BattleEnded,
            ]
        ));
        assert!(wave.actors.is_empty());
        assert!(wave.dead_aliens.is_empty());
        assert!(
            !wave
                .objects_update_with_food(&prey, &food, || 1)
                .iter()
                .any(|event| matches!(event, InvasionEvent::DiamondDropped { .. }))
        );
    }

    #[test]
    fn music_then_spawn_registers_live_actor_and_freezes_3000() {
        let mut wave = Invasion1_2::new();
        wave.countdown = 2;
        wave.warning = Some(WarningCoords {
            first_x: 101,
            first_y: 167,
            second_x: 400,
            second_y: 299,
        });
        assert_eq!(
            wave.board_update(|| panic!("no draw at music"), || panic!("no actor")),
            vec![InvasionEvent::BattleMusicStarted]
        );
        let mut draws = [0_u32, 9].into_iter();
        let mut ids = [42_u64].into_iter();
        assert_eq!(
            wave.board_update(|| draws.next().unwrap(), || ids.next().unwrap()),
            vec![InvasionEvent::AlienSpawned {
                id: 42,
                x: 101,
                y: 167
            }]
        );
        assert_eq!(draws.next(), None);
        assert_eq!(ids.next(), None);
        assert_eq!(wave.countdown, 3000);
        assert_eq!(wave.actors[0].spawn_ticks, 15);
        assert_eq!((wave.warps[0].x, wave.warps[0].y), (131, 127));
        assert!(
            wave.board_update(|| panic!("live freezes timer"), || panic!("live"))
                .is_empty()
        );
        assert_eq!(wave.countdown, 3000);
        wave.validate().unwrap();
    }

    #[test]
    fn second_wave_opens_battle_tip_only_once() {
        let mut wave = Invasion1_2::new();
        wave.danger_shown = true;
        wave.countdown = 277;
        assert_eq!(
            wave.board_update(|| panic!("modal"), || panic!("modal")),
            vec![InvasionEvent::ModalOpened(InvasionTip::BattleTip)]
        );
        wave.acknowledge_modal();
        wave.countdown = 277;
        assert!(
            wave.board_update(|| panic!("no warning yet"), || panic!("no actor"))
                .is_empty()
        );
        assert_eq!(wave.pending_modal, None);
    }

    #[test]
    fn missed_laser_still_fires_and_death_releases_invasion_once() {
        let mut wave = Invasion1_2::new();
        wave.countdown = 3000;
        let mut actor = WeakSylvester::spawn(5, 100, 120, 1, 1);
        actor.health = 6.0;
        actor.x = 100.75;
        actor.y = 120.9;
        wave.actors.push(actor);
        wave.battle_active = true;
        let miss = wave.click(10, 41);
        assert!(miss.suppress_food);
        assert_eq!(
            miss.events,
            vec![InvasionEvent::LaserFired { x: 10, y: 41 }]
        );
        let mut kill = wave.click(180, 200);
        kill.events.extend(wave.finish_if_no_threats(false));
        assert!(kill.suppress_food);
        assert_eq!(
            kill.events,
            vec![
                InvasionEvent::AlienHit { id: 5, health: 0.0 },
                InvasionEvent::AlienDefeated { id: 5 },
                InvasionEvent::DiamondDropped {
                    alien_id: 5,
                    x: 125,
                    y: 145
                },
                InvasionEvent::LaserFired { x: 180, y: 200 },
                InvasionEvent::BattleEnded,
            ]
        );
        assert!(!wave.has_live_alien());
        assert_eq!(wave.food_delay, 36);
        let body = &wave.dead_aliens[0];
        assert_eq!((body.x, body.y), (100.0, 120.0));
        assert_eq!((body.widget_x, body.widget_y), (100, 120));
        let near = wave.click(180, 200);
        assert!(near.events.is_empty());
        assert!(near.suppress_food);
        assert_eq!(wave.food_delay, 36);
        let far = wave.click(231, 200);
        assert_eq!(far.events, vec![InvasionEvent::FoodDelayCleared]);
        assert!(!far.suppress_food);
        assert_eq!(wave.food_delay, 0);
    }

    #[test]
    fn paused_board_update_still_reduces_food_delay_once() {
        let mut wave = Invasion1_2::new();
        wave.food_delay = 36;
        wave.post_spawn_flash_ticks = 35;
        wave.pending_modal = Some(InvasionTip::Danger);
        wave.danger_shown = true;
        wave.countdown = 276;
        wave.update_before_pause();
        assert_eq!(wave.food_delay, 35);
        assert_eq!(wave.post_spawn_flash_ticks, 35);
        assert_eq!(wave.countdown, 276);
        assert!(wave.actors.is_empty());
        wave.board_update(|| panic!("paused"), || panic!("paused"));
        assert_eq!(wave.food_delay, 34);
        assert_eq!(wave.post_spawn_flash_ticks, 35);
        assert_eq!(wave.countdown, 276);
    }

    #[test]
    fn actor_contact_reports_one_old_position_prey_id() {
        let mut wave = Invasion1_2::new();
        wave.countdown = 3000;
        let mut actor = WeakSylvester::spawn(9, 100, 120, 1, 1);
        actor.spawn_ticks = 0;
        actor.chase_ticks = 0;
        wave.actors.push(actor);
        wave.battle_active = true;
        let prey = [PreyView {
            id: 77,
            widget_x: 140,
            widget_y: 160,
            width: 80,
            height: 80,
            eligible: true,
        }];
        assert_eq!(
            wave.objects_update(&prey, || panic!("chase uses no RNG")),
            vec![InvasionEvent::PreyEaten {
                alien_id: 9,
                prey_id: 77
            }]
        );
    }

    #[test]
    fn effect_counters_survive_save_and_expire_on_source_boundaries() {
        let mut wave = Invasion1_2::new();
        wave.warps.push(WarpEffect {
            x: 130,
            y: 100,
            remaining_ticks: 36,
        });
        wave.lasers.push(LaserEffect {
            x: 80,
            y: 100,
            age_ticks: 0,
        });
        let encoded = serde_json::to_string(&wave).unwrap();
        let mut resumed: Invasion1_2 = serde_json::from_str(&encoded).unwrap();
        for _ in 0..14 {
            resumed.objects_update(&[], || panic!("no actor"));
        }
        assert_eq!(resumed.lasers.len(), 1);
        assert_eq!(resumed.lasers[0].frame(), 13);
        resumed.objects_update(&[], || panic!("no actor"));
        assert!(resumed.lasers.is_empty());
        for _ in 0..21 {
            resumed.objects_update(&[], || panic!("no actor"));
        }
        assert_eq!(resumed.warps[0].remaining_ticks, 0);
        resumed.objects_update(&[], || panic!("no actor"));
        assert!(resumed.warps.is_empty());
    }

    #[test]
    fn strong_stage_starts_at_3000_without_first_stage_two_modals() {
        let mut wave = Invasion1_2::new_strong();
        assert_eq!(wave.countdown, 3000);
        assert_eq!(wave.plan, WavePlan::Fixed(SylvesterKind::Strong));
        wave.countdown = 277;
        assert!(
            wave.board_update(|| panic!("modal needs no RNG"), || panic!("no spawn"))
                .is_empty()
        );
        assert_eq!(wave.countdown, 276);
        assert!(wave.pending_modal.is_none());
        assert!(!wave.danger_shown);
        let mut draws = [0_u32, 0, 1, 1].into_iter();
        assert!(matches!(
            wave.board_update(|| draws.next().unwrap(), || panic!("no spawn"))
                .as_slice(),
            [InvasionEvent::WarningStarted(_)]
        ));
        assert_eq!(draws.next(), None);
        wave.countdown = 1;
        let spawned = wave.board_update(|| 1, || 99);
        assert!(matches!(
            spawned.as_slice(),
            [InvasionEvent::AlienSpawned { id: 99, .. }]
        ));
        assert_eq!(wave.actors[0].kind, SylvesterKind::Strong);
        assert_eq!(wave.actors[0].health, 60.0);
    }

    #[test]
    fn balrog_pending_death_finishes_old_widget_contact_then_drops_at_new_widget() {
        // W1 Alien.cpp: contact precedes integration; removal follows Move
        // and animation. PB21 fixes Balrog HP/divisor, PB22 shared removal.
        let mut wave = Invasion1_2::new_balrog();
        let mut actor = WeakSylvester::spawn_kind(SylvesterKind::Balrog, 44, 100, 120, 1, 1);
        actor.spawn_ticks = 0;
        actor.chase_ticks = 0;
        actor.health = 0.0; // Itchy contact leaves this actor registered.
        wave.actors.push(actor);
        wave.battle_active = true;
        let prey = [PreyView {
            id: 77,
            widget_x: 140,
            widget_y: 160,
            width: 80,
            height: 80,
            eligible: true,
        }];
        let events = wave.objects_update(&prey, || panic!("chase consumes no RNG"));
        assert!(matches!(
            events.first(),
            Some(InvasionEvent::PreyEaten {
                alien_id: 44,
                prey_id: 77
            })
        ));
        let body = &wave.dead_aliens[0];
        assert_eq!(body.kind, SylvesterKind::Balrog);
        assert!(body.widget_x > 100);
        assert!(events.iter().any(|event| matches!(
            event,
            InvasionEvent::DiamondDropped {
                alien_id: 44,
                x: 127,
                y: 145
            }
        )));
        assert!(wave.actors.is_empty());
        assert_eq!(wave.food_delay, 36);
        assert!(
            !wave
                .objects_update(&[], || 1)
                .iter()
                .any(|event| matches!(event, InvasionEvent::DiamondDropped { .. }))
        );
    }

    #[test]
    fn pending_death_waits_through_emergence_and_pause_but_shot_can_claim_once() {
        let mut wave = Invasion1_2::new_balrog();
        let mut actor = WeakSylvester::spawn_kind(SylvesterKind::Balrog, 45, 100, 120, 1, 1);
        actor.health = 0.0;
        actor.hit_ticks = 5;
        wave.actors.push(actor);
        wave.battle_active = true;
        for _ in 0..6 {
            assert!(
                wave.objects_update(&[], || panic!("emergence needs no RNG"))
                    .is_empty()
            );
        }
        assert_eq!(wave.actors[0].hit_ticks, 5);
        wave.pending_modal = Some(InvasionTip::Danger);
        assert!(
            wave.objects_update(&[], || panic!("pause needs no RNG"))
                .is_empty()
        );
        assert!(!wave.actors.is_empty());
        wave.pending_modal = None;
        // Positive flash blocks a shot even at zero HP; the next active
        // alien update removes it through the shared transaction.
        assert!(
            !wave
                .click(180, 200)
                .events
                .iter()
                .any(|event| matches!(event, InvasionEvent::AlienDefeated { .. }))
        );
        assert_eq!(
            wave.objects_update(&[], || 1)
                .iter()
                .filter(|event| matches!(event, InvasionEvent::DiamondDropped { .. }))
                .count(),
            1
        );

        let mut race = Invasion1_2::new_balrog();
        let mut pending = WeakSylvester::spawn_kind(SylvesterKind::Balrog, 46, 100, 120, 1, 1);
        pending.health = 0.0;
        pending.hit_ticks = 0;
        race.actors.push(pending);
        race.battle_active = true;
        assert_eq!(
            race.click(180, 200)
                .events
                .iter()
                .filter(|event| matches!(event, InvasionEvent::DiamondDropped { .. }))
                .count(),
            1
        );
        assert!(race.actors.is_empty());
        assert!(
            !race
                .objects_update(&[], || 1)
                .iter()
                .any(|event| matches!(event, InvasionEvent::DiamondDropped { .. }))
        );
    }

    #[test]
    fn balrog_base_weapon_requires_22_accepted_hits() {
        let mut wave = Invasion1_2::new_balrog();
        let mut actor = WeakSylvester::spawn_kind(SylvesterKind::Balrog, 47, 100, 120, 1, 1);
        actor.spawn_ticks = 0;
        wave.actors.push(actor);
        wave.battle_active = true;
        for hit in 1..=21 {
            wave.actors[0].hit_ticks = 0;
            let events = wave.click_with_weapon(180, 200, 2).events;
            assert!(
                events.iter().any(|event| matches!(event,
                InvasionEvent::AlienHit { id: 47, health } if *health == f64::from(130 - 6 * hit)))
            );
            assert!(!wave.actors.is_empty());
        }
        wave.actors[0].hit_ticks = 0;
        assert_eq!(
            wave.click_with_weapon(180, 200, 2)
                .events
                .iter()
                .filter(|event| matches!(event, InvasionEvent::AlienDefeated { id: 47 }))
                .count(),
            1
        );
    }

    #[test]
    fn tank2_pair_registers_weak_then_balrog_before_conditional_future_draws() {
        let mut wave = Invasion1_2::new_tank2_finale(SylvesterKind::Gus);
        wave.plan = WavePlan::CyclingTank2Finale {
            next: EncounterKind::WeakBalrogPair,
        };
        wave.warning = Some(WarningCoords {
            first_x: 100,
            first_y: 120,
            second_x: 300,
            second_y: 200,
        });
        wave.countdown = 1;
        // Two constructor draws per actor, then the old type-9 low bit,
        // failed %10 toggle and successful %20 pair roll.
        let mut draws = [1, 1, 1, 1, 1, 1, 0].into_iter();
        let mut ids = [40, 41].into_iter();
        let events = wave.board_update(|| draws.next().unwrap(), || ids.next().unwrap());
        assert_eq!(draws.next(), None);
        assert_eq!(ids.next(), None);
        assert!(matches!(
            events.as_slice(),
            [
                InvasionEvent::AlienSpawned { id: 40, .. },
                InvasionEvent::AlienSpawned { id: 41, .. }
            ]
        ));
        assert_eq!(
            wave.actors
                .iter()
                .map(|actor| actor.kind)
                .collect::<Vec<_>>(),
            [SylvesterKind::Weak, SylvesterKind::Balrog]
        );
        assert_eq!(wave.warps.len(), 2);
        assert_eq!(wave.plan.expected(), EncounterKind::WeakBalrogPair);
        wave.validate().unwrap();
    }

    #[test]
    fn tank3_second_stage_spawns_one_kind_then_uses_one_toggle_draw() {
        let mut wave = Invasion1_2::new_tank3_second_stage(SylvesterKind::Gus);
        wave.warning = Some(WarningCoords {
            first_x: 100,
            first_y: 120,
            second_x: 300,
            second_y: 200,
        });
        wave.countdown = 1;
        // Two actor constructor draws, followed by exactly one reschedule draw.
        let mut draws = [7, 9, 20].into_iter();
        let events = wave.board_update(|| draws.next().expect("no extra schedule draw"), || 40);
        assert_eq!(draws.next(), None);
        assert!(matches!(
            events.as_slice(),
            [InvasionEvent::AlienSpawned { id: 40, .. }]
        ));
        assert_eq!(wave.actors.len(), 1);
        assert_eq!(wave.actors[0].kind, SylvesterKind::Gus);
        assert_eq!(
            wave.plan,
            WavePlan::CyclingTank3Second {
                next: SylvesterKind::Destructor
            }
        );
        wave.validate().unwrap();
    }

    #[test]
    fn pair_first_death_does_not_end_battle_or_discard_second_corpse() {
        let mut wave = Invasion1_2::new_tank2_finale(SylvesterKind::Gus);
        wave.plan = WavePlan::CyclingTank2Finale {
            next: EncounterKind::Single(SylvesterKind::Gus),
        };
        let mut first = WeakSylvester::spawn_kind(SylvesterKind::Weak, 40, 100, 120, 1, 1);
        let mut second = WeakSylvester::spawn_kind(SylvesterKind::Balrog, 41, 300, 200, 1, 1);
        first.spawn_ticks = 0;
        second.spawn_ticks = 0;
        first.health = 6.0;
        second.health = 6.0;
        wave.actors = vec![first, second];
        wave.battle_active = true;
        let first_events = wave.click(180, 200).events;
        assert!(
            first_events
                .iter()
                .any(|event| matches!(event, InvasionEvent::DiamondDropped { alien_id: 40, .. }))
        );
        assert!(
            !first_events
                .iter()
                .any(|event| matches!(event, InvasionEvent::BattleEnded))
        );
        assert!(wave.finish_if_no_threats(false).is_empty());
        assert_eq!(wave.actors.len(), 1);
        assert_eq!(wave.dead_aliens.len(), 1);
        let second_events = wave.click(380, 280).events;
        assert!(
            second_events
                .iter()
                .any(|event| matches!(event, InvasionEvent::DiamondDropped { alien_id: 41, .. }))
        );
        assert_eq!(wave.dead_aliens.len(), 2);
        assert_eq!(
            wave.finish_if_no_threats(false),
            [InvasionEvent::BattleEnded]
        );
        assert!(wave.finish_if_no_threats(false).is_empty());
    }
}
