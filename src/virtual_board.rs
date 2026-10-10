//! Physical Virtual Tank Board state. VirtualTankState owns durable fish;
//! Profile owns the shell wallet. This module owns transient poses and objects.

use crate::alien::{AlienFoodView, PreyView};
use crate::invasion::InvasionEvent;
use crate::missile::MissileKind;
use crate::sim::{Event, PetKind, VirtualPetAction};
use crate::virtual_combat::{VirtualCombatClick, VirtualCombatEvent, VirtualCombatState};
use crate::virtual_tank::{
    FeedResult, VirtualCoinKind, VirtualFishKind, VirtualFoodKind, VirtualProduction,
    VirtualTankState,
};
use crate::walter::Glove;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
/// Mode-five Board state contains only physical poses and short-lived objects.
/// VirtualTankState owns every fish identity and its durable attributes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VirtualBoardState {
    pub tick: u64,
    pub poses: Vec<VirtualFishPose>,
    pub food: Vec<VirtualFoodPose>,
    pub coins: Vec<VirtualCoinPose>,
    pub next_object_id: u64,
    pub pets: Vec<VirtualPetPose>,
    pub prey: Vec<VirtualPreyPose>,
    pub corpses: Vec<crate::virtual_pet_combat::VirtualCorpsePose>,
    pub combat: VirtualCombatState,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VirtualFishPose {
    pub fish_id: u16,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub walter_recoil_ticks: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VirtualFoodPose {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub kind: VirtualFoodKind,
    pub ineligible_ticks: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VirtualPetPose {
    /// Selected roster identity, including a Presto form's original identity.
    pub kind: PetKind,
    pub active_form: PetKind,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub previous_vx: f32,
    pub timer: u32,
    pub threshold: u32,
    pub niko_pearl_taken: bool,
    pub brinkley_meals: u32,
    pub brinkley_cooldown: u16,
    pub zorf_brinkley_cycles: u32,
    pub walter_action_ticks: u8,
    pub glove: Option<Glove>,
    pub amp_charge: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VirtualPreyPose {
    pub id: u64,
    pub kind: VirtualFoodKind,
    pub x: f32,
    pub y: f32,
    pub age: u32,
    pub walter_recoil_ticks: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VirtualCoinPose {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub kind: VirtualCoinKind,
    /// Niko pearls are claimed by click and belong to one selected pet.
    pub niko_owner: Option<PetKind>,
    pub age: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VirtualPetClick {
    WalterPunch,
    AmpCharge,
    AmpShock,
}

fn pet_threshold(kind: PetKind) -> u32 {
    match kind {
        PetKind::Zorf => 65,
        PetKind::Prego => 930,
        PetKind::Meryl => 1_400,
        PetKind::Vert | PetKind::Shrapnel | PetKind::Nostradamus => 1_080,
        PetKind::Niko => 1_450,
        PetKind::Amp => 3_000,
        PetKind::Stanley => 100,
        PetKind::Gash => 1_570,
        _ => 1,
    }
}

impl VirtualBoardState {
    /// The current Board clock projects the source's 500-update effect cycle.
    /// A separate retail particle RNG/hidden-device clock remains unverified.
    pub fn bubbulator_cycle(&self) -> u16 {
        (self.tick % 500) as u16
    }

    pub fn new(tank: &VirtualTankState) -> Self {
        let mut board = Self {
            tick: 0,
            poses: Vec::new(),
            food: Vec::new(),
            coins: Vec::new(),
            next_object_id: 1,
            pets: Vec::new(),
            prey: Vec::new(),
            corpses: Vec::new(),
            combat: VirtualCombatState::new(0),
        };
        board.sync_fish(tank);
        board.sync_pets(tank);
        board
    }

    pub fn sync_fish(&mut self, tank: &VirtualTankState) {
        self.poses.retain(|pose| {
            tank.active_fish()
                .any(|fish| fish.id == pose.fish_id && fish.shown)
        });
        for fish in tank.active_fish().filter(|fish| fish.shown) {
            if !self.poses.iter().any(|pose| pose.fish_id == fish.id) {
                let ordinal = u32::from(fish.id);
                self.poses.push(VirtualFishPose {
                    fish_id: fish.id,
                    x: 96.0 + (ordinal.wrapping_mul(61) % 440) as f32,
                    y: 96.0 + (ordinal.wrapping_mul(43) % 280) as f32,
                    vx: if ordinal % 2 == 0 { 0.65 } else { -0.65 },
                    walter_recoil_ticks: 0,
                });
            }
        }
        self.poses.sort_by_key(|pose| pose.fish_id);
    }

    pub fn sync_pets(&mut self, tank: &VirtualTankState) {
        self.pets
            .retain(|pet| tank.selected_pets.contains(&pet.kind));
        for (index, kind) in tank.selected_pets.iter().copied().enumerate() {
            if self.pets.iter().any(|pet| pet.kind == kind) {
                continue;
            }
            self.pets.push(VirtualPetPose {
                kind,
                active_form: kind,
                x: 90.0 + (index as f32 * 65.0),
                y: if matches!(kind, PetKind::Stinky | PetKind::Clyde) {
                    415.0
                } else {
                    240.0
                },
                vx: 0.65,
                previous_vx: 0.65,
                timer: if kind == PetKind::Amp { 300 } else { 0 },
                threshold: pet_threshold(kind),
                niko_pearl_taken: false,
                brinkley_meals: 0,
                brinkley_cooldown: 0,
                zorf_brinkley_cycles: 0,
                walter_action_ticks: 0,
                glove: None,
                amp_charge: 0,
            });
        }
        self.pets.sort_by_key(|pet| pet.kind as usize);
        self.coins.retain(|coin| {
            coin.niko_owner.is_none_or(|owner| {
                self.pets
                    .iter()
                    .any(|pet| pet.kind == owner && pet.active_form == PetKind::Niko)
            })
        });
    }

    pub fn clear_ephemeral(&mut self) {
        self.food.clear();
        self.coins.clear();
        self.prey.clear();
        self.corpses.clear();
        self.pets.clear();
        self.combat = VirtualCombatState::new(0);
    }

    pub fn validate(&self, tank: &VirtualTankState) -> Result<(), String> {
        let expected = tank.active_fish().filter(|fish| fish.shown).count();
        if self.next_object_id == 0
            || self.next_object_id == u64::MAX
            || self.poses.len() != expected
            || self.poses.iter().enumerate().any(|(index, pose)| {
                (index > 0 && self.poses[index - 1].fish_id >= pose.fish_id)
                    || !tank
                        .active_fish()
                        .any(|fish| fish.id == pose.fish_id && fish.shown)
                    || !pose.x.is_finite()
                    || !(20.0..=620.0).contains(&pose.x)
                    || !pose.y.is_finite()
                    || !(55.0..=455.0).contains(&pose.y)
                    || !pose.vx.is_finite()
                    || pose.vx.abs() > 4.0
                    || pose.walter_recoil_ticks > 50
            })
            || self.food.iter().any(|food| {
                food.id == 0
                    || food.ineligible_ticks > 20
                    || food.id >= self.next_object_id
                    || !food.x.is_finite()
                    || !food.y.is_finite()
                    || !(20.0..=620.0).contains(&food.x)
                    || !(55.0..=455.0).contains(&food.y)
            })
            || self.coins.iter().any(|coin| {
                coin.id == 0
                    || coin.id >= self.next_object_id
                    || (coin.kind == VirtualCoinKind::NikoPearl) != coin.niko_owner.is_some()
                    || coin.niko_owner.is_some_and(|owner| {
                        !self
                            .pets
                            .iter()
                            .any(|pet| pet.kind == owner && pet.active_form == PetKind::Niko)
                    })
                    || (coin.kind == VirtualCoinKind::NikoPearl && coin.age >= 217)
                    || !coin.x.is_finite()
                    || !coin.y.is_finite()
                    || !(20.0..=620.0).contains(&coin.x)
                    || !(55.0..=455.0).contains(&coin.y)
            })
            || self.pets.len() != tank.selected_pets.len()
            || self.pets.iter().any(|pet| {
                !tank.selected_pets.contains(&pet.kind)
                    || (pet.kind != PetKind::Presto && pet.active_form != pet.kind)
                    || !pet.x.is_finite()
                    || !(20.0..=620.0).contains(&pet.x)
                    || !pet.y.is_finite()
                    || !(55.0..=455.0).contains(&pet.y)
                    || pet.threshold == 0
                    || !pet.vx.is_finite()
                    || !pet.previous_vx.is_finite()
                    || pet.vx.abs() > 4.0
                    || pet.previous_vx.abs() > 4.0
                    || pet.brinkley_cooldown > 108
                    || pet.walter_action_ticks > 40
                    || pet.amp_charge > 2
                    || pet.glove.is_some_and(|glove| {
                        pet.active_form != PetKind::Walter || !glove.validate()
                    })
            })
            || self.prey.iter().any(|prey| {
                prey.id == 0
                    || prey.walter_recoil_ticks > 50
                    || prey.id >= self.next_object_id
                    || !matches!(
                        prey.kind,
                        VirtualFoodKind::Guppy | VirtualFoodKind::Oscar | VirtualFoodKind::Ultra
                    )
                    || !prey.x.is_finite()
                    || !(20.0..=620.0).contains(&prey.x)
                    || !prey.y.is_finite()
                    || !(55.0..=455.0).contains(&prey.y)
            })
        {
            return Err("Virtual Tank Board and durable fish disagree".into());
        }
        let mut ids = HashSet::new();
        if self
            .corpses
            .iter()
            .any(|corpse| !corpse.validate(self.next_object_id))
        {
            return Err("Virtual Tank corpse state is invalid".into());
        }
        if self.coins.iter().any(|coin| !ids.insert(coin.id))
            || self.food.iter().any(|food| !ids.insert(food.id))
            || self.prey.iter().any(|prey| !ids.insert(prey.id))
            || self.corpses.iter().any(|corpse| !ids.insert(corpse.id))
        {
            return Err("Virtual Tank object identities repeat".into());
        }
        let mut pet_kinds = HashSet::new();
        if self
            .pets
            .iter()
            .any(|pet| !pet_kinds.insert(pet.kind as usize))
        {
            return Err("Virtual Tank pet identities repeat".into());
        }
        for id in self
            .combat
            .wave
            .actors
            .iter()
            .map(|actor| actor.id)
            .chain(self.combat.wave.bilaterus.iter().map(|group| group.id))
            .chain(
                self.combat
                    .wave
                    .fragments
                    .iter()
                    .map(|fragment| fragment.id),
            )
            .chain(self.combat.missiles.iter().map(|missile| missile.id))
        {
            if id == 0 || id >= self.next_object_id || !ids.insert(id) {
                return Err("Virtual Tank combat identities repeat or escape allocator".into());
            }
        }
        self.combat
            .validate(&self.prey.iter().map(|prey| prey.id).collect::<Vec<_>>())
    }

    pub fn feed(&mut self, x: f32, y: f32) -> bool {
        self.feed_kind(x, y, VirtualFoodKind::Basic)
    }

    /// The purchased food upgrade changes pellets created after checkout.
    /// Existing pellets and food supplied by pets keep their original kind.
    pub fn feed_for_tank(&mut self, tank: &VirtualTankState, x: f32, y: f32) -> bool {
        self.feed_kind(
            x,
            y,
            if tank.food_upgrade_owned {
                VirtualFoodKind::Upgraded
            } else {
                VirtualFoodKind::Basic
            },
        )
    }

    pub fn feed_kind(&mut self, x: f32, y: f32, kind: VirtualFoodKind) -> bool {
        if !(20.0..=620.0).contains(&x) || !(55.0..=455.0).contains(&y) || self.food.len() >= 12 {
            return false;
        }
        self.food.push(VirtualFoodPose {
            id: self.next_object_id,
            x,
            y,
            kind,
            ineligible_ticks: 0,
        });
        self.next_object_id = self.next_object_id.saturating_add(1);
        true
    }

    pub fn click_combat(
        &mut self,
        x: f32,
        y: f32,
        tank: &VirtualTankState,
        drop_shells: bool,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> (bool, Vec<Event>) {
        let next_id = &mut self.next_object_id;
        let outcome = self.combat.click(
            x as i32,
            y as i32,
            || rand_range(u64::from(u32::MAX) + 1) as u32,
            || {
                let id = *next_id;
                *next_id = next_id.saturating_add(1);
                id
            },
        );
        match outcome {
            VirtualCombatClick::FeedGus { ineligible_ticks } => {
                if self.feed_for_tank(tank, x, y) {
                    self.food
                        .last_mut()
                        .expect("accepted food")
                        .ineligible_ticks = ineligible_ticks;
                    let food = self.food.last().expect("accepted food");
                    return (
                        true,
                        vec![Event::VirtualFoodDropped {
                            tick: self.tick,
                            food_id: food.id,
                            kind: food.kind,
                        }],
                    );
                }
                (true, Vec::new())
            }
            VirtualCombatClick::Shot(changes) => {
                (true, self.settle_combat_events(changes, drop_shells))
            }
            VirtualCombatClick::NoCombat { suppress_food } => (suppress_food, Vec::new()),
        }
    }

    /// Only transient Prego/Cookie children enter combat's prey authority.
    pub fn update_combat(
        &mut self,
        tank: &VirtualTankState,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> Vec<Event> {
        self.update_combat_with_saver_limit(tank, None, rand_range)
    }

    pub(crate) fn update_combat_with_saver_limit(
        &mut self,
        tank: &VirtualTankState,
        saver_production_limit: Option<u32>,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> Vec<Event> {
        let can_drop_coin = tank.can_drop_coin(
            saver_production_limit.is_some(),
            saver_production_limit.unwrap_or(0),
        );
        let contact_changes = crate::virtual_pet_combat::update_contacts(self, rand_range);
        let mut events = self.settle_combat_events(contact_changes, can_drop_coin);
        let prey: Vec<_> = self
            .prey
            .iter()
            .map(|prey| PreyView {
                id: prey.id,
                widget_x: prey.x as i32 - 40,
                widget_y: prey.y as i32 - 40,
                width: 80,
                height: 80,
                eligible: !crate::virtual_pet_combat::warded_prey(self, prey.kind),
            })
            .collect();
        let food: Vec<_> = self
            .food
            .iter()
            .map(|food| AlienFoodView {
                id: food.id,
                widget_x: food.x as i32 - 20,
                widget_y: food.y as i32 - 20,
                quality: if food.kind == VirtualFoodKind::Upgraded {
                    2
                } else {
                    1
                },
                eligible: food.ineligible_ticks == 0,
            })
            .collect();
        let next_id = &mut self.next_object_id;
        let changes = self.combat.tick(
            tank.settings.alien_attractor_shown,
            tank.backdrop,
            &prey,
            &food,
            || rand_range(u64::from(u32::MAX) + 1) as u32,
            || {
                let id = *next_id;
                *next_id = next_id.saturating_add(1);
                id
            },
        );
        events.extend(self.settle_combat_events(changes, can_drop_coin));
        events
    }

    pub(crate) fn drop_coin(&mut self, x: f32, y: f32, kind: VirtualCoinKind) {
        self.coins.push(VirtualCoinPose {
            id: self.next_object_id,
            x: x.clamp(20.0, 620.0),
            y: y.clamp(55.0, 455.0),
            kind,
            niko_owner: None,
            age: 0,
        });
        self.next_object_id = self.next_object_id.saturating_add(1);
    }

    /// Discharge removes temporary guppies; purchased fish never enter this list.
    pub fn discharge_amp(&mut self, drop_shells: bool) -> bool {
        let positions: Vec<_> = self
            .prey
            .iter()
            .filter(|prey| prey.kind == VirtualFoodKind::Guppy)
            .map(|prey| (prey.x, prey.y))
            .collect();
        let had_victims = !positions.is_empty();
        self.prey.retain(|prey| prey.kind != VirtualFoodKind::Guppy);
        if drop_shells {
            for (x, y) in positions {
                self.drop_coin(x, y, VirtualCoinKind::Diamond);
            }
        }
        had_victims
    }

    fn settle_combat_events(
        &mut self,
        changes: Vec<VirtualCombatEvent>,
        drop_shells: bool,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        for change in changes {
            match change {
                VirtualCombatEvent::Invasion(event) => {
                    if matches!(&event, InvasionEvent::DiamondDropped { .. }) && !drop_shells {
                        continue;
                    }
                    match &event {
                        InvasionEvent::PreyEaten { prey_id, .. }
                        | InvasionEvent::BilaterusPreyEaten { prey_id, .. } => {
                            self.prey.retain(|prey| prey.id != *prey_id)
                        }
                        InvasionEvent::GusAteFood { food_id, .. } => {
                            self.food.retain(|food| food.id != *food_id)
                        }
                        InvasionEvent::DiamondDropped { alien_id, x, y } if drop_shells => {
                            let coin_id = self.next_object_id;
                            self.drop_coin(*x as f32, *y as f32, VirtualCoinKind::Diamond);
                            events.push(Event::AlienDiamondDropped {
                                tick: self.tick,
                                alien_id: *alien_id,
                                coin_id,
                            });
                        }
                        _ => {}
                    }
                    events.push(Event::Invasion {
                        tick: self.tick,
                        event,
                    });
                }
                VirtualCombatEvent::MissileLaunched {
                    id,
                    target_id,
                    kind,
                    slot,
                } => {
                    if kind == MissileKind::EnergyBall {
                        events.push(Event::EnergyBallLaunched {
                            tick: self.tick,
                            missile_id: id,
                            target_id,
                            first: slot == 0,
                        });
                    } else {
                        events.push(Event::MissileLaunched {
                            tick: self.tick,
                            missile_id: id,
                            target_id,
                        });
                    }
                }
                VirtualCombatEvent::MissileImpacted { id, prey_id } => {
                    if let Some(index) = self.prey.iter().position(|prey| prey.id == prey_id) {
                        let prey = self.prey.remove(index);
                        let corpse = crate::virtual_pet_combat::VirtualCorpsePose::from_prey(
                            self.next_object_id,
                            &prey,
                        );
                        self.next_object_id = self.next_object_id.saturating_add(1);
                        self.corpses.push(corpse);
                    }
                    events.push(Event::MissileImpacted {
                        tick: self.tick,
                        missile_id: id,
                        target_id: prey_id,
                    });
                }
                VirtualCombatEvent::MissileRemoved { id } => events.push(Event::MissileRemoved {
                    tick: self.tick,
                    missile_id: id,
                }),
                VirtualCombatEvent::MissileRedirected { id } => {
                    events.push(Event::EnergyBallShot {
                        tick: self.tick,
                        missile_id: id,
                        redirected: true,
                    })
                }
                VirtualCombatEvent::ReflectedHit {
                    missile_id,
                    alien_id,
                    health,
                } => events.push(Event::EnergyBallAlienHit {
                    tick: self.tick,
                    missile_id,
                    alien_id,
                    health,
                }),
                VirtualCombatEvent::StanleyImpacted {
                    missile_id,
                    alien_id,
                    ..
                } => events.push(Event::StanleyMissileImpacted {
                    tick: self.tick,
                    missile_id,
                    target_id: alien_id,
                }),
            }
        }
        events
    }

    /// Input admission only. Adventure owns dialogs and sound events; this
    /// Board pose owns the resulting glove and specialty clocks.
    pub fn click_pet(&mut self, x: f32, y: f32) -> Option<(PetKind, VirtualPetClick)> {
        let pet = self
            .pets
            .iter_mut()
            .find(|pet| (pet.x - x).abs() < 40.0 && (pet.y - y).abs() < 40.0)?;
        match pet.active_form {
            PetKind::Walter if pet.glove.is_none() && pet.walter_action_ticks == 0 => {
                pet.glove = Some(Glove::new(pet.vx >= 0.0));
                pet.walter_action_ticks = 40;
                Some((pet.kind, VirtualPetClick::WalterPunch))
            }
            PetKind::Amp if pet.timer >= pet.threshold => {
                if pet.amp_charge < 2 {
                    pet.amp_charge += 1;
                    Some((pet.kind, VirtualPetClick::AmpCharge))
                } else {
                    pet.amp_charge = 0;
                    pet.timer = 0;
                    Some((pet.kind, VirtualPetClick::AmpShock))
                }
            }
            _ => None,
        }
    }

    pub fn open_presto_dialog(&self, x: f32, y: f32) -> bool {
        self.pets.iter().any(|pet| {
            pet.kind == PetKind::Presto && (pet.x - x).abs() < 40.0 && (pet.y - y).abs() < 40.0
        })
    }

    /// The selected Presto identity survives every form replacement. The
    /// caller supplies the Profile unlock roster and handles the dialog.
    pub fn morph_presto(&mut self, target: PetKind, unlocked: &[PetKind]) -> bool {
        if !unlocked.contains(&target) {
            return false;
        }
        let Some(pet) = self.pets.iter_mut().find(|pet| pet.kind == PetKind::Presto) else {
            return false;
        };
        if pet.active_form == target {
            return false;
        }
        self.coins
            .retain(|coin| coin.niko_owner != Some(PetKind::Presto));
        pet.active_form = target;
        pet.timer = if target == PetKind::Amp { 300 } else { 0 };
        pet.threshold = pet_threshold(target);
        pet.niko_pearl_taken = false;
        pet.brinkley_meals = 0;
        pet.brinkley_cooldown = 0;
        pet.zorf_brinkley_cycles = 0;
        pet.walter_action_ticks = 0;
        pet.glove = None;
        pet.amp_charge = 0;
        true
    }

    pub fn collect(
        &mut self,
        x: f32,
        y: f32,
        tank: &mut VirtualTankState,
        shells: &mut u32,
    ) -> Option<(VirtualCoinKind, u32)> {
        let index = self
            .coins
            .iter()
            .position(|coin| (coin.x - x).abs() < 25.0 && (coin.y - y).abs() < 25.0)?;
        let coin = self.coins.remove(index);
        if let Some(owner) = coin.niko_owner
            && let Some(pet) = self.pets.iter_mut().find(|pet| pet.kind == owner)
        {
            pet.niko_pearl_taken = true;
        }
        Some((coin.kind, tank.collect_coin(shells, coin.kind, 1, false)))
    }

    /// Coin collectors act on actual Board coin poses; Profile's wallet is
    /// still credited only through the durable tank collection transaction.
    pub fn collect_pet_coins(
        &mut self,
        tank: &mut VirtualTankState,
        shells: &mut u32,
    ) -> Vec<(VirtualCoinKind, u32, u32)> {
        let mut collected = Vec::new();
        let mut index = 0;
        while index < self.coins.len() {
            let coin = &self.coins[index];
            let reached = coin.niko_owner.is_none()
                && self.pets.iter().any(|pet| match pet.active_form {
                    PetKind::Stinky => coin.y >= 385.0 && (coin.x - pet.x).abs() < 24.0,
                    PetKind::Clyde => {
                        (coin.x - pet.x).abs() < 28.0 && (coin.y - pet.y).abs() < 28.0
                    }
                    _ => false,
                });
            if reached {
                let coin = self.coins.remove(index);
                let amount = tank.collect_coin(shells, coin.kind, 1, false);
                collected.push((coin.kind, amount, *shells));
            } else {
                index += 1;
            }
        }
        collected
    }

    pub fn update(
        &mut self,
        tank: &mut VirtualTankState,
        now_seconds: u64,
        day_start_seconds: u64,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> Vec<Event> {
        self.update_with_saver_limit(tank, now_seconds, day_start_seconds, None, rand_range)
    }

    pub(crate) fn update_with_saver_limit(
        &mut self,
        tank: &mut VirtualTankState,
        now_seconds: u64,
        day_start_seconds: u64,
        saver_production_limit: Option<u32>,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> Vec<Event> {
        self.tick = self.tick.saturating_add(1);
        let can_drop_coin = tank.can_drop_coin(
            saver_production_limit.is_some(),
            saver_production_limit.unwrap_or(0),
        );
        let bubbulator_pulsed = tank.settings.bubbulator_shown && self.bubbulator_cycle() == 13;
        let mut events = Vec::new();
        if bubbulator_pulsed {
            events.push(Event::VirtualBubbulatorPulsed { tick: self.tick });
        }
        self.sync_fish(tank);
        self.sync_pets(tank);
        for pose in &mut self.poses {
            pose.walter_recoil_ticks = pose.walter_recoil_ticks.saturating_sub(1);
        }
        for food in &mut self.food {
            food.ineligible_ticks = food.ineligible_ticks.saturating_sub(1);
            food.y = (food.y + 1.2).min(455.0);
        }
        self.food.retain(|food| food.y < 455.0);
        let seymour = self
            .pets
            .iter()
            .any(|pet| pet.active_form == PetKind::Seymour);
        for coin in &mut self.coins {
            if coin.kind == VirtualCoinKind::NikoPearl {
                coin.age = coin.age.saturating_add(1);
            } else {
                coin.y = (coin.y + if seymour { 0.8 } else { 1.5 }).min(455.0);
            }
        }
        self.coins.retain(|coin| {
            coin.y < 455.0 && (coin.kind != VirtualCoinKind::NikoPearl || coin.age < 217)
        });
        for prey in &mut self.prey {
            prey.walter_recoil_ticks = prey.walter_recoil_ticks.saturating_sub(1);
            prey.age = prey.age.saturating_add(1);
            prey.x = (prey.x + 0.25).min(610.0);
        }
        let mut pet_coins = Vec::new();
        let mut pet_food = Vec::new();
        let mut pet_prey = Vec::new();
        let brinkley_active = self
            .pets
            .iter()
            .any(|pet| pet.active_form == PetKind::Brinkley);
        for pet in &mut self.pets {
            let form = pet.active_form;
            if matches!(form, PetKind::Stinky | PetKind::Clyde)
                && let Some(target) = self
                    .coins
                    .iter()
                    .filter(|coin| coin.niko_owner.is_none())
                    .min_by(|a, b| {
                        let da = (a.x - pet.x).abs()
                            + if form == PetKind::Clyde {
                                (a.y - pet.y).abs()
                            } else {
                                0.0
                            };
                        let db = (b.x - pet.x).abs()
                            + if form == PetKind::Clyde {
                                (b.y - pet.y).abs()
                            } else {
                                0.0
                            };
                        da.total_cmp(&db)
                    })
            {
                pet.x = (pet.x + (target.x - pet.x).clamp(-1.5, 1.5)).clamp(20.0, 620.0);
                if form == PetKind::Clyde {
                    pet.y = (pet.y + (target.y - pet.y).clamp(-1.2, 1.2)).clamp(55.0, 455.0);
                }
            }
            if form == PetKind::Nimbus
                && let Some(target) = self
                    .coins
                    .iter()
                    .filter(|coin| {
                        coin.niko_owner.is_none()
                            && !matches!(
                                coin.kind,
                                VirtualCoinKind::ShrapnelBomb | VirtualCoinKind::Peanut
                            )
                    })
                    .min_by(|a, b| {
                        let da = (a.x - pet.x).abs() + (a.y - pet.y).abs();
                        let db = (b.x - pet.x).abs() + (b.y - pet.y).abs();
                        da.total_cmp(&db)
                    })
            {
                pet.previous_vx = pet.vx;
                pet.vx = (target.x - pet.x).clamp(-1.0, 1.0);
                pet.x = (pet.x + pet.vx).clamp(20.0, 620.0);
                pet.y = (pet.y + (target.y - pet.y).clamp(-1.0, 1.0)).clamp(55.0, 455.0);
            }
            if form == PetKind::Nimbus
                && let Some(coin) = self.coins.iter_mut().find(|coin| {
                    coin.niko_owner.is_none()
                        && !matches!(
                            coin.kind,
                            VirtualCoinKind::ShrapnelBomb | VirtualCoinKind::Peanut
                        )
                        && (coin.x - pet.x).abs() < 30.0
                        && (coin.y - pet.y).abs() < 30.0
                })
            {
                coin.y = (pet.y - 25.0).max(55.0);
            }
            if form == PetKind::Brinkley {
                pet.brinkley_cooldown = pet.brinkley_cooldown.saturating_sub(1);
                if pet.brinkley_cooldown == 0
                    && let Some((index, target)) = self.food.iter().enumerate().min_by(|a, b| {
                        let da = (a.1.x - pet.x).abs() + (a.1.y - pet.y).abs();
                        let db = (b.1.x - pet.x).abs() + (b.1.y - pet.y).abs();
                        da.total_cmp(&db)
                    })
                {
                    let (target_x, target_y) = (target.x, target.y);
                    pet.previous_vx = pet.vx;
                    pet.vx = (target_x - pet.x).clamp(-2.0, 2.0);
                    pet.x = (pet.x + pet.vx).clamp(20.0, 620.0);
                    pet.y = (pet.y + (target_y - pet.y).clamp(-2.0, 2.0)).clamp(55.0, 455.0);
                    if (target_x - pet.x).abs() < 20.0 && (target_y - pet.y).abs() < 20.0 {
                        self.food.remove(index);
                        pet.brinkley_cooldown = 108;
                        pet.brinkley_meals = pet.brinkley_meals.saturating_add(1);
                        if can_drop_coin && pet.brinkley_meals.is_multiple_of(3) {
                            let kind = if pet.brinkley_meals.is_multiple_of(135) {
                                VirtualCoinKind::Treasure
                            } else if pet.brinkley_meals.is_multiple_of(45) {
                                VirtualCoinKind::Pearl
                            } else if pet.brinkley_meals.is_multiple_of(15) {
                                VirtualCoinKind::Diamond
                            } else {
                                VirtualCoinKind::Star
                            };
                            pet_coins.push((pet.x, pet.y, kind, None));
                        }
                    }
                }
            }
            if pet.walter_action_ticks > 0 {
                pet.walter_action_ticks -= 1;
            }
            if let Some(mut glove) = pet.glove {
                glove.x = pet.x as i32 - if glove.right { 20 } else { 60 };
                glove.y = pet.y as i32;
                glove.lifetime = glove.lifetime.saturating_sub(1);
                pet.glove = (glove.lifetime > 0).then_some(glove);
            }
            pet.timer = pet.timer.saturating_add(1);
            if form == PetKind::Niko {
                if pet.timer == 1_233 {
                    pet.niko_pearl_taken = false;
                    pet_coins.push((pet.x, pet.y, VirtualCoinKind::NikoPearl, Some(pet.kind)));
                    events.push(Event::VirtualPetActed {
                        tick: self.tick,
                        pet: pet.kind,
                        action: VirtualPetAction::NikoOpen,
                    });
                } else if pet.timer >= 1_450 {
                    pet.timer = rand_range(50) as u32;
                    events.push(Event::VirtualPetActed {
                        tick: self.tick,
                        pet: pet.kind,
                        action: VirtualPetAction::NikoClose,
                    });
                }
                continue;
            }
            if form == PetKind::Meryl && pet.timer == 1_300 {
                events.push(Event::VirtualPetActed {
                    tick: self.tick,
                    pet: pet.kind,
                    action: VirtualPetAction::MerylSing,
                });
            }
            if form == PetKind::Amp && pet.timer == pet.threshold {
                events.push(Event::VirtualPetActed {
                    tick: self.tick,
                    pet: pet.kind,
                    action: VirtualPetAction::AmpReady,
                });
            }
            if pet.timer < pet.threshold {
                continue;
            }
            match form {
                PetKind::Zorf => {
                    let hungry = tank.active_fish().any(|fish| {
                        fish.shown
                            && fish.hunger < 300
                            && matches!(
                                fish.kind,
                                VirtualFishKind::Guppy | VirtualFishKind::Breeder
                            )
                            && fish.preferred_food() == VirtualFoodKind::Basic
                    });
                    if brinkley_active {
                        pet.zorf_brinkley_cycles = pet.zorf_brinkley_cycles.saturating_add(1);
                    }
                    if hungry || (brinkley_active && pet.zorf_brinkley_cycles.is_multiple_of(4)) {
                        pet_food.push((pet.x, pet.y, VirtualFoodKind::Basic));
                        pet.timer = 0;
                    }
                }
                PetKind::Prego if self.prey.len() + pet_prey.len() < 10 => {
                    pet_prey.push((pet.x, pet.y));
                    events.push(Event::VirtualPetActed {
                        tick: self.tick,
                        pet: pet.kind,
                        action: VirtualPetAction::PregoBirth,
                    });
                    pet.timer = 0;
                    pet.threshold = if self.prey.len() > 5 { 1_230 } else { 930 };
                }
                PetKind::Vert if can_drop_coin => {
                    pet_coins.push((pet.x, pet.y, VirtualCoinKind::Gold, None));
                    pet.timer = 0;
                }
                PetKind::Shrapnel => {
                    pet_coins.push((pet.x, pet.y, VirtualCoinKind::ShrapnelBomb, None));
                    events.push(Event::VirtualPetActed {
                        tick: self.tick,
                        pet: pet.kind,
                        action: VirtualPetAction::ShrapnelDrop,
                    });
                    pet.timer = 0;
                }
                PetKind::Nostradamus => {
                    pet_food.push((pet.x, pet.y, VirtualFoodKind::Snot));
                    pet.timer = 0;
                }
                PetKind::Amp => {}
                PetKind::Stanley | PetKind::Gash | PetKind::Wadsworth | PetKind::Rhubarb => {}
                _ if pet.timer >= pet.threshold => pet.timer = 0,
                _ => {}
            }
        }
        for (x, y, kind, niko_owner) in pet_coins {
            self.coins.push(VirtualCoinPose {
                id: self.next_object_id,
                x,
                y,
                kind,
                niko_owner,
                age: 0,
            });
            self.next_object_id = self.next_object_id.saturating_add(1);
        }
        for (x, y, kind) in pet_food {
            if self.food.len() < 12 {
                self.food.push(VirtualFoodPose {
                    id: self.next_object_id,
                    x,
                    y,
                    kind,
                    ineligible_ticks: 0,
                });
                self.next_object_id = self.next_object_id.saturating_add(1);
            }
        }
        for (x, y) in pet_prey {
            self.prey.push(VirtualPreyPose {
                id: self.next_object_id,
                kind: VirtualFoodKind::Guppy,
                x,
                y,
                age: 0,
                walter_recoil_ticks: 0,
            });
            self.next_object_id = self.next_object_id.saturating_add(1);
        }
        let meryl = self
            .pets
            .iter()
            .any(|pet| pet.active_form == PetKind::Meryl && pet.timer >= 1_300);
        let mut newborns = Vec::new();
        let mut cookie_due = Vec::new();
        for pose in &mut self.poses {
            // W1 Fish::Update divides displacement by 0.3 for an idle Speedy
            // fish and 0.8 after finding nearby food. Our pose velocity uses
            // a normalized scale, so retain that faster-idle/slower-feeding
            // relationship without introducing actor-specific speed state.
            let fish = tank.active_fish().find(|fish| fish.id == pose.fish_id);
            let speedy = fish.is_some_and(|fish| fish.properties.speedy);
            let preferred = fish.map(|fish| fish.preferred_food());
            let food_nearby = self.food.iter().any(|food| {
                let edible = preferred.is_some_and(|kind| {
                    food.kind == kind
                        || (food.kind == VirtualFoodKind::Upgraded
                            && kind == VirtualFoodKind::Basic)
                        || (fish.is_some_and(|fish| fish.kind == VirtualFishKind::Guppy)
                            && matches!(food.kind, VirtualFoodKind::Snot | VirtualFoodKind::Star))
                });
                edible && (food.x - pose.x).abs() < 100.0 && (food.y - pose.y).abs() < 100.0
            }) || self.prey.iter().any(|prey| {
                prey.age >= 40
                    && preferred == Some(prey.kind)
                    && (prey.x - pose.x).abs() < 100.0
                    && (prey.y - pose.y).abs() < 100.0
            });
            if food_nearby
                && fish.is_some_and(|fish| fish.properties.voracious)
                && self.tick % 150 == u64::from(pose.fish_id) % 150
            {
                events.push(Event::VirtualFishVocalized {
                    tick: self.tick,
                    fish_id: pose.fish_id,
                });
            }
            pose.x += pose.vx
                * if speedy {
                    if food_nearby { 1.5 } else { 3.0 }
                } else {
                    1.0
                };
            if pose.x <= 24.0 || pose.x >= 616.0 {
                pose.x = pose.x.clamp(24.0, 616.0);
                pose.vx = -pose.vx;
            }
            let Some(preferred) = preferred else {
                continue;
            };
            if let Some(fish) = tank
                .fish
                .iter_mut()
                .find(|fish| fish.id == pose.fish_id && fish.alive)
            {
                fish.tick_hunger(true);
            }
            let food_index = self.food.iter().position(|food| {
                let kind = food.kind;
                (kind == preferred
                    || (kind == VirtualFoodKind::Upgraded && preferred == VirtualFoodKind::Basic)
                    || (matches!(kind, VirtualFoodKind::Snot | VirtualFoodKind::Star)
                        && tank.active_fish().any(|fish| {
                            fish.id == pose.fish_id && fish.kind == VirtualFishKind::Guppy
                        })))
                    && (food.x - pose.x).abs() < 30.0
                    && (food.y - pose.y).abs() < 26.0
            });
            let food_kind = if let Some(index) = food_index {
                let kind = self.food[index].kind;
                Some((kind, false, index))
            } else {
                self.prey
                    .iter()
                    .position(|prey| {
                        prey.age >= 40
                            && prey.kind == preferred
                            && (prey.x - pose.x).abs() < 30.0
                            && (prey.y - pose.y).abs() < 26.0
                    })
                    .map(|index| (self.prey[index].kind, true, index))
            };
            if let Some((kind, is_prey, index)) = food_kind
                && let Ok(result) =
                    tank.feed_fish(pose.fish_id, kind, now_seconds, day_start_seconds)
            {
                if is_prey {
                    self.prey.remove(index);
                } else {
                    self.food.remove(index);
                }
                events.push(Event::VirtualFishFed {
                    tick: self.tick,
                    fish_id: pose.fish_id,
                    prey: is_prey,
                });
                if matches!(result, FeedResult::Grew(_)) {
                    events.push(Event::VirtualFishGrew {
                        tick: self.tick,
                        fish_id: pose.fish_id,
                    });
                }
                if let FeedResult::Coin(coin_kind) = result
                    && can_drop_coin
                {
                    self.coins.push(VirtualCoinPose {
                        id: self.next_object_id,
                        x: pose.x,
                        y: pose.y,
                        kind: coin_kind,
                        niko_owner: None,
                        age: 0,
                    });
                    self.next_object_id = self.next_object_id.saturating_add(1);
                }
            }
            if let Some(fish) = tank
                .fish
                .iter_mut()
                .find(|fish| fish.id == pose.fish_id && fish.alive)
            {
                match fish.tick_production(
                    tank.settings.drop_shells,
                    saver_production_limit.is_some(),
                    saver_production_limit.unwrap_or(0),
                    tank.board_shells,
                    meryl,
                ) {
                    Some(VirtualProduction::Coin(kind)) => {
                        self.coins.push(VirtualCoinPose {
                            id: self.next_object_id,
                            x: pose.x,
                            y: pose.y,
                            kind,
                            niko_owner: None,
                            age: 0,
                        });
                        self.next_object_id = self.next_object_id.saturating_add(1);
                    }
                    Some(VirtualProduction::GrubberLarvaDue) if self.food.len() < 12 => {
                        self.food.push(VirtualFoodPose {
                            id: self.next_object_id,
                            x: pose.x,
                            y: pose.y,
                            kind: VirtualFoodKind::Larva,
                            ineligible_ticks: 0,
                        });
                        self.next_object_id = self.next_object_id.saturating_add(1);
                    }
                    Some(VirtualProduction::CookieFoodDue) => cookie_due.push((pose.x, pose.y)),
                    Some(VirtualProduction::BreederBirthDue) => newborns.push(pose.fish_id),
                    _ => {}
                }
            }
        }
        for (x, y) in cookie_due {
            let needed = tank
                .active_fish()
                .filter(|fish| fish.shown)
                .map(|fish| fish.preferred_food())
                .find(|kind| {
                    if *kind == VirtualFoodKind::Basic {
                        return false;
                    }
                    let demand = tank
                        .active_fish()
                        .filter(|fish| fish.shown && fish.preferred_food() == *kind)
                        .count();
                    let supply = self.food.iter().filter(|food| food.kind == *kind).count()
                        + self.prey.iter().filter(|prey| prey.kind == *kind).count();
                    demand > supply
                });
            if let Some(kind) = needed {
                if matches!(
                    kind,
                    VirtualFoodKind::Guppy | VirtualFoodKind::Oscar | VirtualFoodKind::Ultra
                ) {
                    self.prey.push(VirtualPreyPose {
                        id: self.next_object_id,
                        kind,
                        x,
                        y,
                        age: 0,
                        walter_recoil_ticks: 0,
                    });
                    self.next_object_id = self.next_object_id.saturating_add(1);
                } else if self.food.len() < 12 {
                    self.food.push(VirtualFoodPose {
                        id: self.next_object_id,
                        x,
                        y,
                        kind,
                        ineligible_ticks: 0,
                    });
                    self.next_object_id = self.next_object_id.saturating_add(1);
                }
            }
        }
        for parent_id in newborns {
            let properties = tank
                .active_fish()
                .find(|fish| fish.id == parent_id)
                .map(|fish| fish.properties);
            if let Some(properties) = properties {
                let mut seed = self.tick ^ (u64::from(parent_id) << 32);
                let offer =
                    crate::virtual_tank::source_offer(VirtualFishKind::Guppy, properties, || {
                        seed = seed
                            .wrapping_mul(6364136223846793005)
                            .wrapping_add(1442695040888963407);
                        (seed >> 32) as u32
                    });
                let _ = tank.birth_from_breeder(
                    parent_id,
                    now_seconds,
                    day_start_seconds,
                    offer.initial_hunger,
                    offer.food_needed_to_grow,
                    offer.coin_threshold,
                );
            }
        }
        self.sync_fish(tank);
        self.sync_pets(tank);
        events
    }
}
