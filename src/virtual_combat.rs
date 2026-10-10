//! Virtual Tank's attractor encounter and transient combat objects.
//!
//! W1 Board.cpp:901-1095,3180-3194,4330-4364 and Alien.cpp:403-418
//! supply these mode-five rules. The installed game's exact RNG and visual
//! scheduling remain unverified. Durable purchased fish live elsewhere and
//! are deliberately absent from every prey view passed into this module.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::alien::{AlienFoodView, AlienRuntimeRequest, PreyView, SylvesterKind};
use crate::bilaterus::BilaterusPreyView;
use crate::invasion::{
    BilaterusChild, EncounterKind, Invasion1_2, InvasionEvent, LaserEffect, WarningCoords, WavePlan,
};
use crate::missile::{
    ClassicMissile, MissileKind, MissilePreyView, MissileShot, StanleyTargetView,
};
use crate::sim::PetKind;

const VIRTUAL_WEAPON: u8 = 8;
const VIRTUAL_THREAT_LIFETIME: u32 = 4_320;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VirtualCombatState {
    /// Existing registered alien, group, effect, and countdown authority.
    pub wave: Invasion1_2,
    /// Board-owned transient projectiles; targets can only be ephemeral prey.
    pub missiles: Vec<ClassicMissile>,
    /// Selected at spawn but installed after the current physical encounter
    /// and its type-specific effects have retired.
    #[serde(deserialize_with = "Option::<EncounterKind>::deserialize")]
    pub next_after_spawn: Option<EncounterKind>,
    /// One encounter is live at a time in mode five.
    pub active_age: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VirtualCombatEvent {
    Invasion(InvasionEvent),
    MissileLaunched {
        id: u64,
        target_id: u64,
        kind: MissileKind,
        slot: u8,
    },
    MissileImpacted {
        id: u64,
        prey_id: u64,
    },
    MissileRemoved {
        id: u64,
    },
    MissileRedirected {
        id: u64,
    },
    ReflectedHit {
        missile_id: u64,
        alien_id: u64,
        health: f64,
    },
    StanleyImpacted {
        missile_id: u64,
        alien_id: u64,
        health: f64,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum VirtualCombatClick {
    /// Gus consumes the click as free food, with its source contact delay.
    FeedGus {
        ineligible_ticks: u8,
    },
    Shot(Vec<VirtualCombatEvent>),
    NoCombat {
        suppress_food: bool,
    },
}

fn chosen_encounter(draw: u32) -> EncounterKind {
    match draw % 7 {
        0 => EncounterKind::Single(SylvesterKind::Strong),
        1 => EncounterKind::Single(SylvesterKind::Balrog),
        2 => EncounterKind::Single(SylvesterKind::Gus),
        3 => EncounterKind::Single(SylvesterKind::Destructor),
        4 => EncounterKind::Single(SylvesterKind::Psychosquid),
        5 => EncounterKind::Single(SylvesterKind::Ulysses),
        _ => EncounterKind::Bilaterus,
    }
}

fn is_virtual_encounter(encounter: EncounterKind) -> bool {
    matches!(
        encounter,
        EncounterKind::Single(
            SylvesterKind::Strong
                | SylvesterKind::Balrog
                | SylvesterKind::Gus
                | SylvesterKind::Destructor
                | SylvesterKind::Psychosquid
                | SylvesterKind::Ulysses
        ) | EncounterKind::Bilaterus
    )
}

fn wave_for(encounter: EncounterKind) -> Invasion1_2 {
    let mut wave = match encounter {
        EncounterKind::Single(SylvesterKind::Strong) => Invasion1_2::new_strong(),
        EncounterKind::Single(SylvesterKind::Balrog) => Invasion1_2::new_balrog(),
        EncounterKind::Single(SylvesterKind::Gus) => Invasion1_2::new_gus(),
        EncounterKind::Single(SylvesterKind::Destructor) => Invasion1_2::new_destructor(),
        EncounterKind::Single(SylvesterKind::Psychosquid) => Invasion1_2::new_psychosquid(),
        EncounterKind::Single(SylvesterKind::Ulysses) => Invasion1_2::new_ulysses(),
        EncounterKind::Bilaterus => Invasion1_2::new_bilaterus(),
        _ => unreachable!("mode-five encounter is one of seven kinds"),
    };
    // Board.cpp:949 excludes Virtual Tank from ordinary tutorial modals.
    wave.gus_warning_shown = encounter == EncounterKind::Single(SylvesterKind::Gus);
    wave
}

fn attractor_anchor(backdrop: u8) -> Option<(i32, i32)> {
    // W1 Board.cpp:4801-4828 uses background IDs one through six.
    Some(match backdrop {
        0 => (267, 320),
        1 => (255, 310),
        2 => (260, 310),
        3 => (270, 345),
        4 => (292, 320),
        5 => (210, 352),
        _ => return None,
    })
}

fn consume_invasion(
    changes: Vec<InvasionEvent>,
    consumed_prey: &mut HashSet<u64>,
    consumed_food: &mut HashSet<u64>,
    output: &mut Vec<VirtualCombatEvent>,
) {
    for event in changes {
        match event {
            InvasionEvent::PreyEaten { prey_id, .. }
            | InvasionEvent::BilaterusPreyEaten { prey_id, .. } => {
                consumed_prey.insert(prey_id);
            }
            InvasionEvent::GusAteFood { food_id, .. } => {
                consumed_food.insert(food_id);
            }
            _ => {}
        }
        output.push(VirtualCombatEvent::Invasion(event));
    }
}

impl VirtualCombatState {
    pub fn new(first_roll: u32) -> Self {
        Self {
            wave: wave_for(chosen_encounter(first_roll)),
            missiles: Vec::new(),
            next_after_spawn: None,
            active_age: 0,
        }
    }

    fn stanley_target_view(&self, target_id: u64) -> Option<StanleyTargetView> {
        if let Some(group) = self.wave.bilaterus_by_id(target_id) {
            return Some(StanleyTargetView {
                id: target_id,
                widget_x: group.widget_x,
                widget_y: group.widget_y,
                group: true,
                kind_six: false,
            });
        }
        self.wave
            .actor_by_id(target_id)
            .map(|actor| StanleyTargetView {
                id: target_id,
                widget_x: actor.widget_x,
                widget_y: actor.widget_y,
                group: false,
                kind_six: false,
            })
    }

    /// Called when the Board-owned Stanley pose completes its launch action.
    /// The pose and its timer stay with VirtualPetPose; this module owns only
    /// the launched physical missile and its alien/group reservation.
    pub fn launch_stanley(
        &mut self,
        pet_x: f32,
        pet_y: f32,
        pet_vx: f32,
        mut roll: impl FnMut() -> u32,
        mut alloc_id: impl FnMut() -> u64,
    ) -> Option<VirtualCombatEvent> {
        let (center_x, center_y) = (f64::from(pet_x) + 40.0, f64::from(pet_y) + 40.0);
        let mut best = 10_000;
        let mut target_id = None;
        for (id, x, y, eligible) in self
            .wave
            .bilaterus
            .iter()
            .map(|group| (group.id, group.widget_x + 40, group.widget_y + 40, true))
            .chain(self.wave.actors.iter().map(|actor| {
                (
                    actor.id,
                    actor.widget_x + 80,
                    actor.widget_y + 80,
                    !actor.healing,
                )
            }))
        {
            if !eligible
                || self
                    .missiles
                    .iter()
                    .any(|missile| missile.target_id == Some(id))
            {
                continue;
            }
            let dx = (f64::from(x) - center_x) as i32;
            let dy = (f64::from(y) - center_y) as i32;
            let squared = i64::from(dx) * i64::from(dx) + i64::from(dy) * i64::from(dy);
            let score = (f64::from(squared as f32).sqrt() as f32) as i32;
            if score < best {
                best = score;
                target_id = Some(id);
            }
        }
        let target_id = target_id?;
        let id = alloc_id();
        let x = pet_x as i32 + if pet_vx >= 0.0 { 40 } else { 0 };
        self.missiles.push(ClassicMissile::launch_stanley(
            id,
            target_id,
            x,
            pet_y as i32,
            roll(),
        ));
        Some(VirtualCombatEvent::MissileLaunched {
            id,
            target_id,
            kind: MissileKind::Stanley,
            slot: 0,
        })
    }

    /// Run one active Board update. `prey` contains transient Prego/Cookie
    /// children only; the caller must never include purchased fish or pets.
    /// Returned consumption IDs are settled by VirtualBoard before its next
    /// update, while this pass prevents duplicate contacts within the tick.
    pub fn tick(
        &mut self,
        shown: bool,
        backdrop: u8,
        prey: &[PreyView],
        food: &[AlienFoodView],
        mut roll: impl FnMut() -> u32,
        mut alloc_id: impl FnMut() -> u64,
    ) -> Vec<VirtualCombatEvent> {
        let mut output = Vec::new();
        let mut consumed_prey = HashSet::new();
        let mut consumed_food = HashSet::new();
        self.wave.update_before_pause();

        if shown {
            let wave_events = if self.wave.countdown == 276
                && !self.wave.has_live_alien()
                && self.missiles.is_empty()
            {
                // The shared wave would draw four random coordinates. VT
                // instead fixes the first at the current attractor and draws
                // only the second pair (W1 Board.cpp:3180-3194).
                if let Some((anchor_x, anchor_y)) = attractor_anchor(backdrop) {
                    self.wave.countdown = 275;
                    self.wave.sneeze_shake_ticks = self.wave.sneeze_shake_ticks.saturating_sub(1);
                    self.wave.post_spawn_flash_ticks =
                        self.wave.post_spawn_flash_ticks.saturating_sub(1);
                    let coords = WarningCoords {
                        first_x: anchor_x
                            + if self.wave.plan == WavePlan::FixedBilaterus {
                                1
                            } else {
                                -39
                            },
                        first_y: anchor_y - 180,
                        second_x: (roll() % 450) as i32 + 20,
                        second_y: (roll() % 195) as i32 + 105,
                    };
                    self.wave.warning = Some(coords);
                    vec![InvasionEvent::WarningStarted(coords)]
                } else {
                    Vec::new()
                }
            } else {
                self.wave
                    .advance_with_threats(!self.missiles.is_empty(), &mut roll, &mut alloc_id)
            };
            let spawned = wave_events.iter().any(|event| {
                matches!(
                    event,
                    InvasionEvent::AlienSpawned { .. } | InvasionEvent::BilaterusSpawned { .. }
                )
            });
            consume_invasion(
                wave_events,
                &mut consumed_prey,
                &mut consumed_food,
                &mut output,
            );
            if spawned {
                self.active_age = 0;
                self.next_after_spawn = Some(chosen_encounter(roll()));
            }
        }

        // Snapshot physical missile IDs: a newly launched child does not get
        // a second update in its constructor tick.
        let missile_ids: Vec<u64> = self.missiles.iter().map(|missile| missile.id).collect();
        for id in missile_ids {
            let Some(index) = self.missiles.iter().position(|missile| missile.id == id) else {
                continue;
            };
            if self.missiles[index].kind == MissileKind::Stanley {
                let target = self.missiles[index]
                    .target_id
                    .and_then(|target_id| self.stanley_target_view(target_id));
                let change = self.missiles[index].begin_stanley_tick(
                    target,
                    !self.wave.bilaterus.is_empty(),
                    self.wave.has_live_alien(),
                );
                if change.remove {
                    self.missiles.remove(index);
                    output.push(VirtualCombatEvent::MissileRemoved { id });
                    continue;
                }
                if let Some(alien_id) = change.impact_target {
                    let result = if target.is_some_and(|view| view.group) {
                        self.wave
                            .stanley_hit_bilaterus(alien_id, &mut roll, &mut alloc_id)
                    } else {
                        self.wave.stanley_hit_alien(alien_id)
                    };
                    if let Some((health, changes)) = result {
                        output.push(VirtualCombatEvent::StanleyImpacted {
                            missile_id: id,
                            alien_id,
                            health,
                        });
                        consume_invasion(
                            changes,
                            &mut consumed_prey,
                            &mut consumed_food,
                            &mut output,
                        );
                    }
                }
                if self.missiles[index].finish_stanley_tick() {
                    self.missiles.remove(index);
                    output.push(VirtualCombatEvent::MissileRemoved { id });
                }
                continue;
            }
            if self.missiles[index].kind == MissileKind::EnergyBall
                && self.missiles[index].reflected
                && self.reflected_contact(index, &mut roll, &mut alloc_id, &mut output)
            {
                continue;
            }
            let target = self.missiles[index].target_id.and_then(|target_id| {
                prey.iter()
                    .find(|view| {
                        view.id == target_id && view.eligible && !consumed_prey.contains(&view.id)
                    })
                    .map(|view| MissilePreyView {
                        id: view.id,
                        widget_x: view.widget_x,
                        widget_y: view.widget_y,
                    })
            });
            let change = self.missiles[index].tick(target);
            if let Some(prey_id) = change.impact_target {
                consumed_prey.insert(prey_id);
                output.push(VirtualCombatEvent::MissileImpacted { id, prey_id });
            }
            if change.remove {
                self.missiles.remove(index);
                output.push(VirtualCombatEvent::MissileRemoved { id });
            }
        }

        // Virtual Tank admits only one ordinary actor or one Bilaterus group.
        if let Some(id) = self.wave.actors.first().map(|actor| actor.id) {
            let live_prey: Vec<_> = prey
                .iter()
                .copied()
                .filter(|view| !consumed_prey.contains(&view.id))
                .collect();
            let live_food: Vec<_> = food
                .iter()
                .copied()
                .filter(|view| !consumed_food.contains(&view.id))
                .collect();
            let kind = self.wave.actors[0].kind;
            let missiles = &mut self.missiles;
            let mut launches = Vec::new();
            let changes =
                self.wave
                    .update_actor_with_runtime(
                        id,
                        &live_prey,
                        &live_food,
                        |request| match request {
                            AlienRuntimeRequest::Random => roll(),
                            AlienRuntimeRequest::ProbeTarget {
                                center_x,
                                center_y,
                                excluded_prey,
                            } => u32::from(
                                select_missile_target(
                                    &live_prey,
                                    missiles,
                                    center_x,
                                    center_y,
                                    excluded_prey,
                                )
                                .is_some(),
                            ),
                            AlienRuntimeRequest::Launch {
                                slot,
                                x,
                                y,
                                center_x,
                                center_y,
                                excluded_prey,
                            } => {
                                let Some(target_id) = select_missile_target(
                                    &live_prey,
                                    missiles,
                                    center_x,
                                    center_y,
                                    excluded_prey,
                                ) else {
                                    return 0;
                                };
                                let missile_id = alloc_id();
                                let visual_draw = roll();
                                let missile = if kind == SylvesterKind::Ulysses {
                                    ClassicMissile::launch_energy(
                                        missile_id,
                                        target_id,
                                        x,
                                        y,
                                        visual_draw,
                                    )
                                } else {
                                    ClassicMissile::launch(missile_id, target_id, x, y, visual_draw)
                                };
                                let missile_kind = missile.kind;
                                missiles.push(missile);
                                launches.push(VirtualCombatEvent::MissileLaunched {
                                    id: missile_id,
                                    target_id,
                                    kind: missile_kind,
                                    slot,
                                });
                                1
                            }
                        },
                    );
            output.extend(launches);
            consume_invasion(changes, &mut consumed_prey, &mut consumed_food, &mut output);
        } else if let Some(id) = self.wave.bilaterus.first().map(|group| group.id) {
            let (active, changes) = self.wave.begin_bilaterus_update(id);
            consume_invasion(changes, &mut consumed_prey, &mut consumed_food, &mut output);
            if active {
                for child in [
                    BilaterusChild::ActiveHead,
                    BilaterusChild::Bone(0),
                    BilaterusChild::Bone(1),
                    BilaterusChild::Bone(2),
                    BilaterusChild::Bone(3),
                    BilaterusChild::Bone(4),
                    BilaterusChild::Bone(5),
                    BilaterusChild::PassiveHead,
                ] {
                    let live_prey: Vec<_> = prey
                        .iter()
                        .filter(|view| !consumed_prey.contains(&view.id))
                        .map(|view| BilaterusPreyView {
                            id: view.id,
                            widget_x: view.widget_x,
                            widget_y: view.widget_y,
                            width: view.width,
                            height: view.height,
                            eligible: view.eligible,
                            ultra: false,
                        })
                        .collect();
                    let changes = self
                        .wave
                        .update_bilaterus_child(id, child, &live_prey, &mut roll);
                    consume_invasion(changes, &mut consumed_prey, &mut consumed_food, &mut output);
                }
                let changes = self
                    .wave
                    .finish_bilaterus_update(id, &mut roll, &mut alloc_id);
                consume_invasion(changes, &mut consumed_prey, &mut consumed_food, &mut output);
            }
        }

        if let Some(id) = self
            .wave
            .actors
            .first()
            .map(|actor| actor.id)
            .or_else(|| self.wave.bilaterus.first().map(|group| group.id))
        {
            self.active_age = self.active_age.saturating_add(1);
            if self.active_age > VIRTUAL_THREAT_LIFETIME
                && let Some(changes) = self
                    .wave
                    .expire_virtual_threat(id, !self.missiles.is_empty())
            {
                consume_invasion(changes, &mut consumed_prey, &mut consumed_food, &mut output);
            }
        }
        if !self.wave.has_live_alien() {
            self.active_age = 0;
        }
        self.detach_dead_alien_targets(&mut output);
        // A prey removal detaches its physical missile reservations before
        // the next actor/save snapshot. Otherwise a consumed child could
        // leave a projectile pointing at an absent Board object.
        self.missiles.retain(|missile| {
            let detached = missile
                .target_id
                .is_some_and(|target_id| consumed_prey.contains(&target_id));
            if detached {
                output.push(VirtualCombatEvent::MissileRemoved { id: missile.id });
            }
            !detached
        });
        let changes = self.wave.update_effects();
        consume_invasion(changes, &mut consumed_prey, &mut consumed_food, &mut output);
        self.finish_after_contacts(&mut output);
        output
    }

    fn reflected_contact(
        &mut self,
        missile_index: usize,
        roll: &mut impl FnMut() -> u32,
        alloc_id: &mut impl FnMut() -> u64,
        output: &mut Vec<VirtualCombatEvent>,
    ) -> bool {
        let missile = &self.missiles[missile_index];
        let id = missile.id;
        let (x, y) = (missile.x + 40.0, missile.y + 40.0);
        let ordinary = self.wave.actors.iter().find(|actor| {
            !actor.healing
                && x > f64::from(actor.widget_x + 30)
                && x < f64::from(actor.widget_x + 140)
                && y > f64::from(actor.widget_y + 10)
                && y < f64::from(actor.widget_y + 150)
        });
        if let Some(alien_id) = ordinary.map(|actor| actor.id)
            && let Some((health, changes)) = self.wave.reflected_energy_hit(alien_id)
        {
            output.push(VirtualCombatEvent::ReflectedHit {
                missile_id: id,
                alien_id,
                health,
            });
            output.extend(changes.into_iter().map(VirtualCombatEvent::Invasion));
            self.missiles.remove(missile_index);
            output.push(VirtualCombatEvent::MissileRemoved { id });
            return true;
        }
        let group = self.wave.bilaterus.iter().find(|group| {
            let (gx, gy) = group.active_head_position();
            missile.x + 25.0 > f64::from(gx + 10)
                && missile.x + 25.0 < f64::from(gx + 70)
                && missile.y + 25.0 > f64::from(gy + 10)
                && missile.y + 25.0 < f64::from(gy + 70)
        });
        if let Some(alien_id) = group.map(|group| group.id)
            && let Some((health, changes)) = self
                .wave
                .reflected_energy_hit_bilaterus(alien_id, roll, alloc_id)
        {
            output.push(VirtualCombatEvent::ReflectedHit {
                missile_id: id,
                alien_id,
                health,
            });
            output.extend(changes.into_iter().map(VirtualCombatEvent::Invasion));
            self.missiles.remove(missile_index);
            output.push(VirtualCombatEvent::MissileRemoved { id });
            return true;
        }
        false
    }

    /// The caller routes coins and pet/Presto input before this Board click.
    /// Food admitted by this result is free in Virtual Tank, not a wallet buy.
    pub fn click(
        &mut self,
        x: i32,
        y: i32,
        mut roll: impl FnMut() -> u32,
        mut alloc_id: impl FnMut() -> u64,
    ) -> VirtualCombatClick {
        if let Some(gus) = self
            .wave
            .actors
            .first()
            .filter(|actor| actor.kind == SylvesterKind::Gus)
        {
            if (31..=586).contains(&x) && (61..=379).contains(&y) {
                let (cx, cy) = (gus.widget_x + 80, gus.widget_y + 80);
                let close = cx > x - 35 && cx < x + 75 && cy > y - 35 && cy < y + 75;
                return VirtualCombatClick::FeedGus {
                    ineligible_ticks: if close { 20 } else { 0 },
                };
            }
            return VirtualCombatClick::NoCombat {
                suppress_food: true,
            };
        }
        if y <= 40 {
            return VirtualCombatClick::NoCombat {
                suppress_food: self.wave.has_live_alien()
                    || !self.missiles.is_empty()
                    || self.wave.food_delay > 0,
            };
        }
        if !self.wave.has_live_alien() && self.missiles.is_empty() {
            return VirtualCombatClick::NoCombat {
                suppress_food: self.wave.food_delay > 0,
            };
        }
        let mut output = if self.wave.has_live_alien() {
            self.wave
                .click_with_weapon_random_and_ids(x, y, VIRTUAL_WEAPON, &mut roll, &mut alloc_id)
                .events
                .into_iter()
                .map(VirtualCombatEvent::Invasion)
                .collect::<Vec<_>>()
        } else {
            self.wave.lasers.push(LaserEffect {
                x: x - 40,
                y: y - 40,
                age_ticks: 0,
            });
            self.wave.last_laser = Some((x, y));
            vec![VirtualCombatEvent::Invasion(InvasionEvent::LaserFired {
                x,
                y,
            })]
        };
        if let Some(index) = self.missiles.iter().position(|missile| missile.shot(x, y)) {
            let id = self.missiles[index].id;
            match self.missiles[index].try_shot(x, y) {
                MissileShot::Destroyed => {
                    self.missiles.remove(index);
                    output.push(VirtualCombatEvent::MissileRemoved { id });
                }
                MissileShot::Redirected => {
                    output.push(VirtualCombatEvent::MissileRedirected { id });
                }
                MissileShot::AcceptedImmune | MissileShot::Miss => {}
            }
        }
        self.detach_dead_alien_targets(&mut output);
        self.finish_after_contacts(&mut output);
        VirtualCombatClick::Shot(output)
    }

    /// Apply a contact selected from VirtualBoard's existing pet pose and
    /// active Presto form. The Board owns contact timing and never supplies
    /// purchased fish as a combat target.
    pub fn hit_from_pet(
        &mut self,
        active_form: PetKind,
        target_id: u64,
        mut roll: impl FnMut() -> u32,
        mut alloc_id: impl FnMut() -> u64,
    ) -> Option<(f64, Vec<VirtualCombatEvent>)> {
        let (health, changes) = if self.wave.bilaterus_by_id(target_id).is_some() {
            match active_form {
                PetKind::Itchy => (self.wave.pet_hit_bilaterus(target_id, 1.0)?, Vec::new()),
                PetKind::Rufus => (self.wave.pet_hit_bilaterus(target_id, 2.0)?, Vec::new()),
                PetKind::Gash => {
                    let group = self.wave.bilaterus_by_id_mut(target_id)?;
                    let (health, transition) = group.gash_hit();
                    let changes = transition.map_or_else(Vec::new, |outcome| {
                        self.wave.commit_bilaterus_transition(
                            target_id,
                            outcome,
                            &mut roll,
                            &mut alloc_id,
                        )
                    });
                    (health, changes)
                }
                _ => return None,
            }
        } else {
            match active_form {
                PetKind::Itchy => (
                    self.wave.actor_by_id_mut(target_id)?.itchy_hit()?,
                    Vec::new(),
                ),
                PetKind::Rufus => (
                    self.wave.actor_by_id_mut(target_id)?.rufus_hit()?,
                    Vec::new(),
                ),
                PetKind::Gash => self.wave.gash_hit_alien(target_id)?,
                _ => return None,
            }
        };
        let mut output = changes
            .into_iter()
            .map(VirtualCombatEvent::Invasion)
            .collect::<Vec<_>>();
        self.detach_dead_alien_targets(&mut output);
        self.finish_after_contacts(&mut output);
        Some((health, output))
    }

    fn detach_dead_alien_targets(&mut self, output: &mut Vec<VirtualCombatEvent>) {
        let live_ids: HashSet<u64> = self
            .wave
            .actors
            .iter()
            .map(|actor| actor.id)
            .chain(self.wave.bilaterus.iter().map(|group| group.id))
            .collect();
        self.missiles.retain(|missile| {
            let detached = missile.kind == MissileKind::Stanley
                && missile
                    .target_id
                    .is_some_and(|target_id| !live_ids.contains(&target_id));
            if detached {
                output.push(VirtualCombatEvent::MissileRemoved { id: missile.id });
            }
            !detached
        });
    }

    /// Call after external pet or Walter hits through `wave` so the physical
    /// membership, missile hold, and next expected kind advance together.
    pub fn finish_after_contacts(&mut self, output: &mut Vec<VirtualCombatEvent>) {
        output.extend(
            self.wave
                .finish_if_no_threats(!self.missiles.is_empty())
                .into_iter()
                .map(VirtualCombatEvent::Invasion),
        );
        if !self.wave.has_live_alien() {
            self.active_age = 0;
        }
        if self.wave.has_live_alien()
            || !self.missiles.is_empty()
            || !self.wave.fragments.is_empty()
            || !self.wave.dead_aliens.is_empty()
            || !self.wave.warps.is_empty()
        {
            return;
        }
        if let Some(next) = self.next_after_spawn.take() {
            self.wave.plan = match next {
                EncounterKind::Single(kind) => WavePlan::Fixed(kind),
                EncounterKind::Bilaterus => WavePlan::FixedBilaterus,
                _ => unreachable!("validated mode-five encounter"),
            };
            self.wave.gus_warning_shown = next == EncounterKind::Single(SylvesterKind::Gus);
        }
    }

    pub fn validate(&self, transient_prey_ids: &[u64]) -> Result<(), String> {
        let current = self.wave.plan.expected();
        if !matches!(
            self.wave.plan,
            WavePlan::Fixed(_) | WavePlan::FixedBilaterus
        ) || !is_virtual_encounter(current)
            || self
                .next_after_spawn
                .is_some_and(|next| !is_virtual_encounter(next))
            || self.wave.pending_modal.is_some()
            || self.wave.danger_shown
            || self.wave.battle_tip_shown
            || self.wave.gus_warning_shown != (current == EncounterKind::Single(SylvesterKind::Gus))
            || self.active_age > VIRTUAL_THREAT_LIFETIME
            || (!self.wave.has_live_alien() && self.active_age != 0)
            || (self.next_after_spawn.is_none() && self.wave.has_live_alien())
            || self.wave.actors.len() + self.wave.bilaterus.len() > 1
        {
            return Err("invalid Virtual Tank combat state".into());
        }
        let mut ids = HashSet::new();
        let mut reservations = HashSet::new();
        for id in self
            .wave
            .actors
            .iter()
            .map(|actor| actor.id)
            .chain(self.wave.bilaterus.iter().map(|group| group.id))
        {
            ids.insert(id);
        }
        let prey_ids: HashSet<u64> = transient_prey_ids.iter().copied().collect();
        for missile in &self.missiles {
            missile.validate()?;
            let target_valid = match missile.kind {
                MissileKind::Classic | MissileKind::EnergyBall => missile
                    .target_id
                    .is_some_and(|target_id| prey_ids.contains(&target_id)),
                MissileKind::Stanley => missile
                    .target_id
                    .is_none_or(|target_id| self.stanley_target_view(target_id).is_some()),
            };
            if !ids.insert(missile.id)
                || !target_valid
                || missile
                    .target_id
                    .is_some_and(|target_id| !reservations.insert(target_id))
            {
                return Err("invalid Virtual Tank missile ownership".into());
            }
        }
        self.wave.validate()
    }
}

fn select_missile_target(
    prey: &[PreyView],
    missiles: &[ClassicMissile],
    center_x: i32,
    center_y: i32,
    excluded_prey: Option<u64>,
) -> Option<u64> {
    let mut best = 0_i64;
    let mut selected = None;
    for candidate in prey.iter().filter(|candidate| {
        candidate.eligible
            && Some(candidate.id) != excluded_prey
            && !missiles
                .iter()
                .any(|missile| missile.target_id == Some(candidate.id))
    }) {
        let dx = i64::from(center_x - candidate.widget_x - candidate.width / 2);
        let dy = i64::from(center_y - candidate.widget_y - candidate.height / 2);
        let distance = dx * dx + dy * dy;
        if distance > best {
            best = distance;
            selected = Some(candidate.id);
        }
    }
    selected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_attractor_hidden_clock_and_backdrop_anchor() {
        let mut combat = VirtualCombatState::new(0);
        assert!(combat.tick(false, 0, &[], &[], || 0, || 1).is_empty());
        assert_eq!(combat.wave.countdown, 3000);
        combat.wave.countdown = 276;
        let events = combat.tick(true, 5, &[], &[], || 0, || 1);
        assert_eq!(combat.wave.countdown, 275);
        assert!(events.iter().any(|event| matches!(
            event,
            VirtualCombatEvent::Invasion(InvasionEvent::WarningStarted(WarningCoords {
                first_x: 171,
                first_y: 172,
                second_x: 20,
                second_y: 105,
            }))
        )));
    }

    #[test]
    fn virtual_age_timeout_skips_defeat_reward() {
        let mut combat = VirtualCombatState::new(0);
        combat.wave.countdown = 1;
        combat.wave.warning = Some(WarningCoords {
            first_x: 228,
            first_y: 140,
            second_x: 20,
            second_y: 105,
        });
        let mut next_id = 1;
        combat.tick(
            true,
            0,
            &[],
            &[],
            || 1,
            || {
                let id = next_id;
                next_id += 1;
                id
            },
        );
        assert_eq!(combat.wave.actors.len(), 1);
        combat.active_age = VIRTUAL_THREAT_LIFETIME;
        let events = combat.tick(
            false,
            0,
            &[],
            &[],
            || 1,
            || {
                let id = next_id;
                next_id += 1;
                id
            },
        );
        assert!(combat.wave.actors.is_empty());
        assert!(combat.wave.dead_aliens.is_empty());
        assert!(!events.iter().any(|event| matches!(
            event,
            VirtualCombatEvent::Invasion(InvasionEvent::DiamondDropped { .. })
        )));
        assert!(events.iter().any(|event| matches!(
            event,
            VirtualCombatEvent::Invasion(InvasionEvent::BattleEnded)
        )));
    }
}
