//! Physical Virtual Tank Board state. VirtualTankState owns durable fish;
//! Profile owns the shell wallet. This module owns transient poses and objects.

use crate::sim::PetKind;
use crate::virtual_tank::{
    FeedResult, VirtualCoinKind, VirtualFishKind, VirtualFoodKind, VirtualProduction,
    VirtualTankState,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
/// Mode-five Board state contains only physical poses and short-lived objects.
/// VirtualTankState owns every fish identity and its durable attributes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VirtualBoardState {
    pub tick: u64,
    pub poses: Vec<VirtualFishPose>,
    pub food: Vec<VirtualFoodPose>,
    pub coins: Vec<VirtualCoinPose>,
    pub next_object_id: u64,
    pub pets: Vec<VirtualPetPose>,
    pub prey: Vec<VirtualPreyPose>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VirtualFishPose {
    pub fish_id: u16,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VirtualFoodPose {
    pub x: f32,
    pub y: f32,
    pub kind: VirtualFoodKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VirtualPetPose {
    pub kind: PetKind,
    pub x: f32,
    pub y: f32,
    pub timer: u32,
    pub threshold: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VirtualPreyPose {
    pub kind: VirtualFoodKind,
    pub x: f32,
    pub y: f32,
    pub age: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VirtualCoinPose {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub kind: VirtualCoinKind,
}

impl VirtualBoardState {
    pub fn new(tank: &VirtualTankState) -> Self {
        let mut board = Self {
            tick: 0,
            poses: Vec::new(),
            food: Vec::new(),
            coins: Vec::new(),
            next_object_id: 1,
            pets: Vec::new(),
            prey: Vec::new(),
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
            let threshold = match kind {
                PetKind::Zorf => 65,
                PetKind::Prego => 930,
                PetKind::Meryl => 1_400,
                PetKind::Vert | PetKind::Shrapnel | PetKind::Nostradamus => 1_080,
                _ => 1,
            };
            self.pets.push(VirtualPetPose {
                kind,
                x: 90.0 + (index as f32 * 65.0),
                y: if matches!(kind, PetKind::Stinky | PetKind::Clyde) {
                    415.0
                } else {
                    240.0
                },
                timer: 0,
                threshold,
            });
        }
        self.pets.sort_by_key(|pet| pet.kind as usize);
    }

    pub fn clear_ephemeral(&mut self) {
        self.food.clear();
        self.coins.clear();
        self.prey.clear();
        self.pets.clear();
    }

    pub fn validate(&self, tank: &VirtualTankState) -> Result<(), String> {
        let expected = tank.active_fish().filter(|fish| fish.shown).count();
        if self.poses.len() != expected
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
            })
            || self.food.iter().any(|food| {
                !food.x.is_finite()
                    || !food.y.is_finite()
                    || !(20.0..=620.0).contains(&food.x)
                    || !(55.0..=455.0).contains(&food.y)
            })
            || self.coins.iter().any(|coin| {
                coin.id == 0
                    || coin.id >= self.next_object_id
                    || !coin.x.is_finite()
                    || !coin.y.is_finite()
                    || !(20.0..=620.0).contains(&coin.x)
                    || !(55.0..=455.0).contains(&coin.y)
            })
            || self.pets.len() != tank.selected_pets.len()
            || self.pets.iter().any(|pet| {
                !tank.selected_pets.contains(&pet.kind)
                    || !pet.x.is_finite()
                    || !(20.0..=620.0).contains(&pet.x)
                    || !pet.y.is_finite()
                    || !(55.0..=455.0).contains(&pet.y)
                    || pet.threshold == 0
            })
            || self.prey.iter().any(|prey| {
                !matches!(
                    prey.kind,
                    VirtualFoodKind::Guppy | VirtualFoodKind::Oscar | VirtualFoodKind::Ultra
                ) || !prey.x.is_finite()
                    || !(20.0..=620.0).contains(&prey.x)
                    || !prey.y.is_finite()
                    || !(55.0..=455.0).contains(&prey.y)
            })
        {
            return Err("Virtual Tank Board and durable fish disagree".into());
        }
        let mut ids = HashSet::new();
        if self.coins.iter().any(|coin| !ids.insert(coin.id)) {
            return Err("Virtual Tank coin identities repeat".into());
        }
        let mut pet_kinds = HashSet::new();
        if self
            .pets
            .iter()
            .any(|pet| !pet_kinds.insert(pet.kind as usize))
        {
            return Err("Virtual Tank pet identities repeat".into());
        }
        Ok(())
    }

    pub fn feed(&mut self, x: f32, y: f32) -> bool {
        self.feed_kind(x, y, VirtualFoodKind::Basic)
    }

    pub fn feed_kind(&mut self, x: f32, y: f32, kind: VirtualFoodKind) -> bool {
        if !(20.0..=620.0).contains(&x) || !(55.0..=455.0).contains(&y) || self.food.len() >= 12 {
            return false;
        }
        self.food.push(VirtualFoodPose { x, y, kind });
        true
    }

    pub fn collect(
        &mut self,
        x: f32,
        y: f32,
        tank: &mut VirtualTankState,
        shells: &mut u32,
    ) -> Option<u32> {
        let index = self
            .coins
            .iter()
            .position(|coin| (coin.x - x).abs() < 25.0 && (coin.y - y).abs() < 25.0)?;
        let coin = self.coins.remove(index);
        Some(tank.collect_coin(shells, coin.kind, 1, false))
    }

    /// Coin collectors act on actual Board coin poses; Profile's wallet is
    /// still credited only through the durable tank collection transaction.
    pub fn collect_pet_coins(&mut self, tank: &mut VirtualTankState, shells: &mut u32) -> u32 {
        let mut earned: u32 = 0;
        let mut index = 0;
        while index < self.coins.len() {
            let coin = &self.coins[index];
            let reached = self.pets.iter().any(|pet| match pet.kind {
                PetKind::Stinky => coin.y >= 385.0 && (coin.x - pet.x).abs() < 24.0,
                PetKind::Clyde => (coin.x - pet.x).abs() < 28.0 && (coin.y - pet.y).abs() < 28.0,
                _ => false,
            });
            if reached {
                let coin = self.coins.remove(index);
                earned = earned.saturating_add(tank.collect_coin(shells, coin.kind, 1, false));
            } else {
                index += 1;
            }
        }
        earned
    }

    pub fn update(
        &mut self,
        tank: &mut VirtualTankState,
        now_seconds: u64,
        day_start_seconds: u64,
    ) {
        self.tick = self.tick.saturating_add(1);
        self.sync_fish(tank);
        self.sync_pets(tank);
        for food in &mut self.food {
            food.y = (food.y + 1.2).min(455.0);
        }
        self.food.retain(|food| food.y < 455.0);
        for coin in &mut self.coins {
            coin.y = (coin.y + 0.9).min(455.0);
        }
        self.coins.retain(|coin| coin.y < 455.0);
        for prey in &mut self.prey {
            prey.age = prey.age.saturating_add(1);
            prey.x = (prey.x + 0.25).min(610.0);
        }
        let mut pet_coins = Vec::new();
        let mut pet_food = Vec::new();
        let mut pet_prey = Vec::new();
        for pet in &mut self.pets {
            if matches!(pet.kind, PetKind::Stinky | PetKind::Clyde)
                && let Some(target) = self.coins.iter().min_by(|a, b| {
                    let da = (a.x - pet.x).abs()
                        + if pet.kind == PetKind::Clyde {
                            (a.y - pet.y).abs()
                        } else {
                            0.0
                        };
                    let db = (b.x - pet.x).abs()
                        + if pet.kind == PetKind::Clyde {
                            (b.y - pet.y).abs()
                        } else {
                            0.0
                        };
                    da.total_cmp(&db)
                })
            {
                pet.x = (pet.x + (target.x - pet.x).clamp(-1.5, 1.5)).clamp(20.0, 620.0);
                if pet.kind == PetKind::Clyde {
                    pet.y = (pet.y + (target.y - pet.y).clamp(-1.2, 1.2)).clamp(55.0, 455.0);
                }
            }
            pet.timer = pet.timer.saturating_add(1);
            if pet.timer < pet.threshold {
                continue;
            }
            match pet.kind {
                PetKind::Zorf
                    if tank.active_fish().any(|fish| {
                        fish.shown
                            && fish.hunger < 300
                            && matches!(
                                fish.kind,
                                VirtualFishKind::Guppy | VirtualFishKind::Breeder
                            )
                            && fish.preferred_food() == VirtualFoodKind::Basic
                    }) =>
                {
                    pet_food.push((pet.x, pet.y, VirtualFoodKind::Basic));
                    pet.timer = 0;
                }
                PetKind::Prego if self.prey.len() + pet_prey.len() < 10 => {
                    pet_prey.push((pet.x, pet.y));
                    pet.timer = 0;
                    pet.threshold = if self.prey.len() > 5 { 1_230 } else { 930 };
                }
                PetKind::Vert if tank.settings.drop_shells => {
                    pet_coins.push((pet.x, pet.y, VirtualCoinKind::Gold));
                    pet.timer = 0;
                }
                PetKind::Shrapnel => {
                    pet_coins.push((pet.x, pet.y, VirtualCoinKind::ShrapnelBomb));
                    pet.timer = 0;
                }
                PetKind::Nostradamus => {
                    pet_food.push((pet.x, pet.y, VirtualFoodKind::Snot));
                    pet.timer = 0;
                }
                _ if pet.timer >= pet.threshold => pet.timer = 0,
                _ => {}
            }
        }
        for (x, y, kind) in pet_coins {
            self.coins.push(VirtualCoinPose {
                id: self.next_object_id,
                x,
                y,
                kind,
            });
            self.next_object_id = self.next_object_id.saturating_add(1);
        }
        for (x, y, kind) in pet_food {
            if self.food.len() < 12 {
                self.food.push(VirtualFoodPose { x, y, kind });
            }
        }
        for (x, y) in pet_prey {
            self.prey.push(VirtualPreyPose {
                kind: VirtualFoodKind::Guppy,
                x,
                y,
                age: 0,
            });
        }
        let meryl = self
            .pets
            .iter()
            .any(|pet| pet.kind == PetKind::Meryl && pet.timer >= 1_300);
        let mut newborns = Vec::new();
        let mut cookie_due = Vec::new();
        for pose in &mut self.poses {
            pose.x += pose.vx;
            if pose.x <= 24.0 || pose.x >= 616.0 {
                pose.x = pose.x.clamp(24.0, 616.0);
                pose.vx = -pose.vx;
            }
            let Some(preferred) = tank
                .active_fish()
                .find(|fish| fish.id == pose.fish_id)
                .map(|fish| fish.preferred_food())
            else {
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
                let kind = if food.kind == VirtualFoodKind::Basic && tank.food_upgrade_owned {
                    VirtualFoodKind::Upgraded
                } else {
                    food.kind
                };
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
            if let Some((kind, is_prey, index)) = food_kind {
                let kind = if kind == VirtualFoodKind::Basic && tank.food_upgrade_owned {
                    VirtualFoodKind::Upgraded
                } else {
                    kind
                };
                if let Ok(result) =
                    tank.feed_fish(pose.fish_id, kind, now_seconds, day_start_seconds)
                {
                    if is_prey {
                        self.prey.remove(index);
                    } else {
                        self.food.remove(index);
                    }
                    if let FeedResult::Coin(coin_kind) = result
                        && tank.settings.drop_shells
                    {
                        self.coins.push(VirtualCoinPose {
                            id: self.next_object_id,
                            x: pose.x,
                            y: pose.y,
                            kind: coin_kind,
                        });
                        self.next_object_id = self.next_object_id.saturating_add(1);
                    }
                }
            }
            if let Some(fish) = tank
                .fish
                .iter_mut()
                .find(|fish| fish.id == pose.fish_id && fish.alive)
            {
                match fish.tick_production(
                    tank.settings.drop_shells,
                    false,
                    0,
                    tank.board_shells,
                    meryl,
                ) {
                    Some(VirtualProduction::Coin(kind)) => {
                        self.coins.push(VirtualCoinPose {
                            id: self.next_object_id,
                            x: pose.x,
                            y: pose.y,
                            kind,
                        });
                        self.next_object_id = self.next_object_id.saturating_add(1);
                    }
                    Some(VirtualProduction::GrubberLarvaDue) if self.food.len() < 12 => {
                        self.food.push(VirtualFoodPose {
                            x: pose.x,
                            y: pose.y,
                            kind: VirtualFoodKind::Larva,
                        });
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
                    self.prey.push(VirtualPreyPose { kind, x, y, age: 0 });
                } else if self.food.len() < 12 {
                    self.food.push(VirtualFoodPose { x, y, kind });
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
    }
}
