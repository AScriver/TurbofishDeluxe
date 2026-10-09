//! Ordinary weak Sylvester wave in Adventure 1-2.
//!
//! This owns the board countdown, live actor membership and finite visual
//! effects, while the caller owns board input routing, prey removal, coins,
//! menu pauses, sound and the existing RNG/id streams. The branch rules are
//! secondary source-derived from WinFish f919b3c (`Board.cpp`, `Alien.cpp`,
//! `Warp.cpp`, `Shot.cpp`, `DeadAlien.cpp`). Installed-game parity is untested.
//! Render frame endpoints for the warp and death particles remain unresolved.

use serde::{Deserialize, Serialize};

use crate::alien::{AlienFoodView, PreyView, ShotResult, SylvesterKind, WeakSylvester};

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
    #[serde(default)] // Pre-format-5 stage-2 waves were necessarily weak.
    pub kind: SylvesterKind,
    pub countdown: i32,
    pub danger_shown: bool,
    pub battle_tip_shown: bool,
    pub gus_warning_shown: bool,
    pub pending_modal: Option<InvasionTip>,
    pub warning: Option<WarningCoords>,
    pub alien: Option<WeakSylvester>,
    pub food_delay: u8,
    pub last_laser: Option<(i32, i32)>,
    pub post_spawn_flash_ticks: u8,
    pub warp: Option<WarpEffect>,
    pub lasers: Vec<LaserEffect>,
    pub dead_alien: Option<DeadAlienEffect>,
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
            kind,
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
            alien: None,
            food_delay: 0,
            last_laser: None,
            post_spawn_flash_ticks: 0,
            warp: None,
            lasers: Vec::new(),
            dead_alien: None,
        }
    }

    pub fn acknowledge_modal(&mut self) -> Option<InvasionTip> {
        self.pending_modal.take()
    }

    pub fn has_live_alien(&self) -> bool {
        self.alien.is_some()
    }

    /// Board::Update reduces food delay before its first pause return at
    /// Board.cpp:610–615. Call this once for an already-paused board update;
    /// ordinary `board_update` calls it itself.
    pub fn update_before_pause(&mut self) {
        self.food_delay = self.food_delay.saturating_sub(1);
    }

    /// Runs the board's wave transition after its clock advances and before
    /// entity updates. RNG is requested four times at warning (including the
    /// unused second coordinate), then twice at spawn for the actor.
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
        mut next_random: impl FnMut() -> u32,
        mut next_id: impl FnMut() -> u64,
    ) -> Vec<InvasionEvent> {
        if self.pending_modal.is_some() {
            return Vec::new();
        }
        self.post_spawn_flash_ticks = self.post_spawn_flash_ticks.saturating_sub(1);
        if self.alien.is_some() {
            return Vec::new();
        }
        if self.countdown <= 0 {
            // A decoded state with no positive countdown is malformed;
            // never decrement it further or synthesize repeated spawns.
            return Vec::new();
        }

        self.countdown -= 1;
        match self.countdown {
            276 if self.kind == SylvesterKind::Weak => {
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
            276 if self.kind == SylvesterKind::Gus && !self.gus_warning_shown => {
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
                let id = next_id();
                let actor = WeakSylvester::spawn_kind(
                    self.kind,
                    id,
                    coords.first_x,
                    coords.first_y,
                    next_random(),
                    next_random(),
                );
                self.warp = Some(WarpEffect {
                    x: coords.first_x + 30,
                    y: coords.first_y - 40,
                    remaining_ticks: 36,
                });
                self.alien = Some(actor);
                self.post_spawn_flash_ticks = 35;
                self.countdown = 3000;
                vec![InvasionEvent::AlienSpawned {
                    id,
                    x: coords.first_x,
                    y: coords.first_y,
                }]
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
        next_random: impl FnMut() -> u32,
    ) -> Vec<InvasionEvent> {
        if self.pending_modal.is_some() {
            return Vec::new();
        }
        let mut events = Vec::new();
        let mut defeated_on_update = false;
        if let Some(actor) = self.alien.as_mut() {
            let update = actor.update_with_food(prey, food, next_random);
            if let Some((food_id, damage)) = update.food_eaten {
                events.push(InvasionEvent::GusAteFood {
                    alien_id: actor.id,
                    food_id,
                    damage,
                });
            }
            if let Some(prey_id) = update.prey_eaten {
                events.push(InvasionEvent::PreyEaten {
                    alien_id: actor.id,
                    prey_id,
                });
            }
            defeated_on_update = update.defeated;
        }
        if let Some(warp) = self.warp.as_mut() {
            if warp.remaining_ticks == 0 {
                self.warp = None;
            } else {
                warp.remaining_ticks -= 1;
            }
        }
        for laser in &mut self.lasers {
            laser.age_ticks += 1;
        }
        self.lasers.retain(|laser| laser.age_ticks <= 14);

        if let Some(body) = self.dead_alien.as_mut() {
            if body.remaining_ticks == 0 {
                self.dead_alien = None;
            } else {
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
        }
        if defeated_on_update {
            events.extend(self.remove_registered_alien());
        }
        events
    }

    /// Handles the board's click priority around an ordinary live alien.
    /// The caller routes menu-area clicks separately but still suppresses
    /// feeding while `suppress_food` is true.
    pub fn click(&mut self, x: i32, y: i32) -> InvasionClick {
        self.click_with_weapon(x, y, 2)
    }

    pub fn click_with_weapon(&mut self, x: i32, y: i32, weapon: u8) -> InvasionClick {
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
            suppress_food: self.alien.is_some() || self.food_delay > 0,
            events,
        };
        if y <= 40 {
            return result;
        }
        let Some((alien_id, shot_result)) = self
            .alien
            .as_mut()
            .map(|actor| (actor.id, actor.shot_with_weapon(x, y, weapon)))
        else {
            return result;
        };
        match shot_result {
            ShotResult::Miss => {}
            ShotResult::Hit { health } => result.events.push(InvasionEvent::AlienHit {
                id: alien_id,
                health,
            }),
            ShotResult::Defeated { .. } => {
                let health = self
                    .alien
                    .as_ref()
                    .expect("shot alien was registered")
                    .health;
                // The input-time impact precedes the shared removal path.
                result.events.push(InvasionEvent::AlienHit {
                    id: alien_id,
                    health,
                });
                result.events.extend(self.remove_registered_alien());
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
    fn remove_registered_alien(&mut self) -> Vec<InvasionEvent> {
        let dead = self
            .alien
            .take()
            .expect("alien removal requires registration");
        let body_x = dead.x as i32;
        let body_y = dead.y as i32;
        self.dead_alien = (dead.kind != SylvesterKind::Gus).then_some(DeadAlienEffect {
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
        self.food_delay = 36;
        vec![
            InvasionEvent::AlienDefeated { id: dead.id },
            InvasionEvent::DiamondDropped {
                alien_id: dead.id,
                x: dead.widget_x + 25,
                y: dead.widget_y + 25,
            },
            InvasionEvent::BattleEnded,
        ]
    }

    pub fn validate(&self) -> Result<(), String> {
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
            || self.alien.is_some() && self.countdown != 3000
            || self.alien.as_ref().is_some_and(|actor| !actor.alive)
            || self
                .alien
                .as_ref()
                .is_some_and(|actor| actor.kind != self.kind)
            || self.kind != SylvesterKind::Weak && (self.danger_shown || self.battle_tip_shown)
            || self.kind != SylvesterKind::Gus && self.gus_warning_shown
            || self.kind != SylvesterKind::Weak
                && self.kind != SylvesterKind::Gus
                && self.pending_modal.is_some()
            || self.kind == SylvesterKind::Weak
                && self.pending_modal == Some(InvasionTip::GusWarning)
            || self.kind == SylvesterKind::Gus
                && matches!(
                    self.pending_modal,
                    Some(InvasionTip::Danger | InvasionTip::BattleTip)
                )
            || self.kind == SylvesterKind::Gus && self.dead_alien.is_some()
            || self
                .warp
                .as_ref()
                .is_some_and(|warp| warp.remaining_ticks > 36)
            || self.lasers.iter().any(|laser| laser.age_ticks > 14)
            || self.dead_alien.as_ref().is_some_and(|body| {
                body.kind != self.kind
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
        if let Some(actor) = &self.alien {
            actor.validate()?;
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
        wave.alien = Some(actor);
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
        assert!(wave.alien.is_none());
        assert!(wave.dead_alien.is_none());
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
        assert_eq!(wave.alien.as_ref().unwrap().spawn_ticks, 15);
        assert_eq!(
            (wave.warp.as_ref().unwrap().x, wave.warp.as_ref().unwrap().y),
            (131, 127)
        );
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
        wave.alien = Some(actor);
        let miss = wave.click(10, 41);
        assert!(miss.suppress_food);
        assert_eq!(
            miss.events,
            vec![InvasionEvent::LaserFired { x: 10, y: 41 }]
        );
        let kill = wave.click(180, 200);
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
                InvasionEvent::BattleEnded,
                InvasionEvent::LaserFired { x: 180, y: 200 },
            ]
        );
        assert!(!wave.has_live_alien());
        assert_eq!(wave.food_delay, 36);
        let body = wave.dead_alien.as_ref().unwrap();
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
        assert!(wave.alien.is_none());
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
        wave.alien = Some(actor);
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
        wave.warp = Some(WarpEffect {
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
        assert_eq!(resumed.warp.as_ref().unwrap().remaining_ticks, 0);
        resumed.objects_update(&[], || panic!("no actor"));
        assert!(resumed.warp.is_none());
    }

    #[test]
    fn strong_stage_starts_at_3000_without_first_stage_two_modals() {
        let mut wave = Invasion1_2::new_strong();
        assert_eq!(wave.countdown, 3000);
        assert_eq!(wave.kind, SylvesterKind::Strong);
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
        assert_eq!(wave.alien.as_ref().unwrap().kind, SylvesterKind::Strong);
        assert_eq!(wave.alien.as_ref().unwrap().health, 60.0);
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
        wave.alien = Some(actor);
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
        let body = wave.dead_alien.as_ref().unwrap();
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
        assert!(wave.alien.is_none());
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
        wave.alien = Some(actor);
        for _ in 0..6 {
            assert!(
                wave.objects_update(&[], || panic!("emergence needs no RNG"))
                    .is_empty()
            );
        }
        assert_eq!(wave.alien.as_ref().unwrap().hit_ticks, 5);
        wave.pending_modal = Some(InvasionTip::Danger);
        assert!(
            wave.objects_update(&[], || panic!("pause needs no RNG"))
                .is_empty()
        );
        assert!(wave.alien.is_some());
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
        race.alien = Some(pending);
        assert_eq!(
            race.click(180, 200)
                .events
                .iter()
                .filter(|event| matches!(event, InvasionEvent::DiamondDropped { .. }))
                .count(),
            1
        );
        assert!(race.alien.is_none());
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
        wave.alien = Some(actor);
        for hit in 1..=21 {
            wave.alien.as_mut().unwrap().hit_ticks = 0;
            let events = wave.click_with_weapon(180, 200, 2).events;
            assert!(
                events.iter().any(|event| matches!(event,
                InvasionEvent::AlienHit { id: 47, health } if *health == f64::from(130 - 6 * hit)))
            );
            assert!(wave.alien.is_some());
        }
        wave.alien.as_mut().unwrap().hit_ticks = 0;
        assert_eq!(
            wave.click_with_weapon(180, 200, 2)
                .events
                .iter()
                .filter(|event| matches!(event, InvasionEvent::AlienDefeated { id: 47 }))
                .count(),
            1
        );
    }
}
