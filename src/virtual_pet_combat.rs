//! Contacts between Virtual Tank pet poses and its physical combat objects.
//! W1 FishTypePet.cpp::CollideWithFood, OtherTypePet.cpp::CollideWithObject,
//! BoxingGlove.cpp::Update/HandleFishHit, and Fish.cpp's Gumbo branch supply
//! the broad rules. Exact motion and contact ordering remain unverified.

use crate::sim::PetKind;
use crate::virtual_board::{VirtualBoardState, VirtualPreyPose};
use crate::virtual_combat::VirtualCombatEvent;
use crate::virtual_tank::VirtualFoodKind;
use serde::{Deserialize, Serialize};

/// A transient missile-killed fish. It is never a durable purchased fish.
/// W1 `Fish::Die` creates `DeadFish` with life 125 and an Angie revival clock.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VirtualCorpsePose {
    pub id: u64,
    pub kind: VirtualFoodKind,
    pub x: f32,
    pub y: f32,
    pub life: u8,
    pub revival_ticks: Option<u8>,
}

impl VirtualCorpsePose {
    pub fn from_prey(id: u64, prey: &VirtualPreyPose) -> Self {
        Self {
            id,
            kind: prey.kind,
            x: prey.x,
            y: prey.y,
            life: 125,
            revival_ticks: None,
        }
    }

    pub fn validate(&self, next_object_id: u64) -> bool {
        self.id > 0
            && self.id < next_object_id
            && matches!(
                self.kind,
                VirtualFoodKind::Guppy | VirtualFoodKind::Oscar | VirtualFoodKind::Ultra
            )
            && self.x.is_finite()
            && (20.0..=620.0).contains(&self.x)
            && self.y.is_finite()
            && (55.0..=455.0).contains(&self.y)
            && (1..=125).contains(&self.life)
            && self
                .revival_ticks
                .is_none_or(|ticks| (1..=10).contains(&ticks))
    }
}

#[derive(Clone, Copy)]
struct Threat {
    id: u64,
    x: f32,
    y: f32,
    group: bool,
}

fn threats(board: &VirtualBoardState) -> Vec<Threat> {
    board
        .combat
        .wave
        .bilaterus
        .iter()
        .map(|group| Threat {
            id: group.id,
            x: group.widget_x as f32 + 40.0,
            y: group.widget_y as f32 + 40.0,
            group: true,
        })
        .chain(
            board
                .combat
                .wave
                .actors
                .iter()
                .filter(|actor| !actor.healing)
                .map(|actor| Threat {
                    id: actor.id,
                    x: actor.widget_x as f32 + 80.0,
                    y: actor.widget_y as f32 + 80.0,
                    group: false,
                }),
        )
        .collect()
}

/// Ordinary Wadsworth protects small guppies while a physical threat is live.
/// The caller applies this to `PreyView.eligible`; purchased fish never enter
/// the combat prey list in the first place.
pub fn warded_prey(board: &VirtualBoardState, kind: VirtualFoodKind) -> bool {
    kind == VirtualFoodKind::Guppy
        && (board.combat.wave.has_live_alien() || !board.combat.missiles.is_empty())
        && board
            .pets
            .iter()
            .any(|pet| pet.active_form == PetKind::Wadsworth)
}

fn update_corpses(board: &mut VirtualBoardState) {
    // DeadFish.cpp:51-126: float/fade until expiration; Angie may start its
    // 10-tick resurrection only while source life remains above 95.
    for corpse in &mut board.corpses {
        corpse.y = (corpse.y + 1.0).min(455.0);
        if corpse.revival_ticks.is_none() && (corpse.y >= 370.0 || corpse.life > 105) {
            corpse.life = corpse.life.saturating_sub(1);
        }
    }
    for pet in &mut board.pets {
        if pet.active_form != PetKind::Angie {
            continue;
        }
        let target_index = board
            .corpses
            .iter()
            .enumerate()
            .filter(|(_, corpse)| corpse.life > 95 && corpse.revival_ticks.is_none())
            .min_by(|a, b| {
                let da = (a.1.x - pet.x).powi(2) + (a.1.y - pet.y).powi(2);
                let db = (b.1.x - pet.x).powi(2) + (b.1.y - pet.y).powi(2);
                da.total_cmp(&db)
            })
            .map(|(index, _)| index);
        if let Some(index) = target_index {
            let corpse = &mut board.corpses[index];
            pet.previous_vx = pet.vx;
            pet.vx = (corpse.x - pet.x).clamp(-2.0, 2.0);
            pet.x = (pet.x + pet.vx).clamp(20.0, 620.0);
            pet.y = (pet.y + (corpse.y - pet.y).clamp(-2.0, 2.0)).clamp(55.0, 455.0);
            let contact = if corpse.kind == VirtualFoodKind::Ultra {
                60.0
            } else {
                30.0
            };
            if (corpse.x - pet.x).abs() < contact && (corpse.y - pet.y).abs() < contact {
                corpse.revival_ticks = Some(10);
            }
        }
    }
    let mut revived = Vec::new();
    for corpse in &mut board.corpses {
        if let Some(ticks) = &mut corpse.revival_ticks {
            *ticks = ticks.saturating_sub(1);
            if *ticks == 0 {
                revived.push((corpse.kind, corpse.x, corpse.y));
            }
        }
    }
    board
        .corpses
        .retain(|corpse| corpse.life > 0 && corpse.revival_ticks != Some(0));
    for (kind, x, y) in revived {
        board.prey.push(VirtualPreyPose {
            id: board.next_object_id,
            kind,
            x,
            y,
            age: 0,
            walter_recoil_ticks: 0,
        });
        board.next_object_id = board.next_object_id.saturating_add(1);
    }
}

/// One Board contact pass before combat target snapshots are made. All actor
/// positions, timers, and object IDs remain on VirtualBoardState.
pub fn update_contacts(
    board: &mut VirtualBoardState,
    rand_range: &mut impl FnMut(u64) -> u64,
) -> Vec<VirtualCombatEvent> {
    let mut output = Vec::new();
    update_corpses(board);
    let targets = threats(board);
    let alien_present = board.combat.wave.has_live_alien();
    let threatened = alien_present || !board.combat.missiles.is_empty();

    // W1 Fish.cpp:309-394 steers nonvirtual fish toward Gumbo during invasion.
    // The transient prey pose has no velocity field; move it toward the pet.
    if threatened
        && let Some(gumbo) = board
            .pets
            .iter()
            .find(|pet| pet.active_form == PetKind::Gumbo)
    {
        for prey in &mut board.prey {
            prey.x = (prey.x + (gumbo.x - prey.x).clamp(-1.3, 1.3)).clamp(20.0, 620.0);
            prey.y = (prey.y + (gumbo.y - prey.y).clamp(-1.0, 1.0)).clamp(55.0, 455.0);
        }
    }

    // W1 OtherTypePet.cpp:1122+ gives Rhubarb a periodic upward push on low
    // fish. Only transient prey is moved here; durable identities stay put.
    for pet in &mut board.pets {
        if pet.active_form != PetKind::Rhubarb || pet.timer < 20 {
            continue;
        }
        let Some(prey) = board.prey.iter_mut().find(|prey| {
            prey.y > (if threatened { 260.0 } else { 270.0 }) && (prey.x - pet.x).abs() < 50.0
        }) else {
            continue;
        };
        prey.y = (prey.y - (if rand_range(2) == 0 { 20.0 } else { 30.0 })).max(55.0);
        pet.timer = 0;
    }

    // BoxingGlove.cpp:21-100 admits targets while the glove has >12 ticks.
    // Each fish/prey pose owns a 50-tick recoil guard (added by VirtualBoard).
    for pet in &mut board.pets {
        if pet.active_form != PetKind::Walter {
            continue;
        }
        let Some(mut glove) = pet.glove else {
            continue;
        };
        if glove.lifetime <= 12 {
            continue;
        }
        let (hit_x, hit_y) = glove.center();
        let direction = if glove.right { 1.0 } else { -1.0 };
        let mut hit = false;
        for fish in &mut board.poses {
            if fish.walter_recoil_ticks == 0
                && (fish.x - hit_x as f32).abs() < 40.0
                && (fish.y - hit_y as f32).abs() < 40.0
            {
                fish.walter_recoil_ticks = 50;
                fish.x = (fish.x + direction * (if rand_range(2) == 0 { 15.0 } else { 10.0 }))
                    .clamp(20.0, 620.0);
                fish.vx = (fish.vx + direction).clamp(-4.0, 4.0);
                hit = true;
            }
        }
        for prey in &mut board.prey {
            if prey.walter_recoil_ticks == 0
                && (prey.x - hit_x as f32).abs() < 40.0
                && (prey.y - hit_y as f32).abs() < 40.0
            {
                prey.walter_recoil_ticks = 50;
                prey.x = (prey.x + direction * (if rand_range(2) == 0 { 15.0 } else { 10.0 }))
                    .clamp(20.0, 620.0);
                hit = true;
            }
        }
        if hit && glove.feedback == 0 {
            glove.feedback = 5;
            pet.glove = Some(glove);
        }
    }

    for pet in &mut board.pets {
        match pet.active_form {
            PetKind::Itchy | PetKind::Gash | PetKind::Rufus if !targets.is_empty() => {
                let (center_x, center_y) = (pet.x, pet.y);
                let target = targets.iter().min_by(|a, b| {
                    let da = (a.x - center_x).powi(2) + (a.y - center_y).powi(2);
                    let db = (b.x - center_x).powi(2) + (b.y - center_y).powi(2);
                    da.total_cmp(&db)
                });
                let Some(target) = target else { continue };
                pet.previous_vx = pet.vx;
                pet.vx = (target.x - center_x).clamp(-2.0, 2.0);
                pet.x = (pet.x + pet.vx).clamp(20.0, 620.0);
                pet.y = (pet.y + (target.y - center_y).clamp(-2.0, 2.0)).clamp(55.0, 455.0);
                let dx = (pet.x - target.x).abs();
                let dy = (pet.y - target.y).abs();
                let touching = match (pet.active_form, target.group) {
                    (PetKind::Rufus, true) => dx < 10.0 && dy < 10.0,
                    (PetKind::Rufus, false) => {
                        pet.x > target.x - 50.0
                            && pet.x < target.x + 60.0
                            && pet.y > target.y - 70.0
                            && pet.y < target.y + 70.0
                    }
                    _ => dx < 20.0 && dy < 20.0,
                };
                let combat = &mut board.combat;
                let next_id = &mut board.next_object_id;
                if touching
                    && let Some((_, changes)) = combat.hit_from_pet(
                        pet.active_form,
                        target.id,
                        || rand_range(u64::from(u32::MAX)) as u32,
                        || {
                            let id = *next_id;
                            *next_id = (*next_id).saturating_add(1);
                            id
                        },
                    )
                {
                    output.extend(changes);
                }
            }
            PetKind::Gash if !alien_present && pet.timer > 1_570 => {
                let target_index = board
                    .prey
                    .iter()
                    .enumerate()
                    .filter(|(_, prey)| prey.kind == VirtualFoodKind::Guppy)
                    .min_by(|a, b| {
                        let da = (a.1.x - pet.x).powi(2) + (a.1.y - pet.y).powi(2);
                        let db = (b.1.x - pet.x).powi(2) + (b.1.y - pet.y).powi(2);
                        da.total_cmp(&db)
                    })
                    .map(|(index, _)| index);
                if let Some(index) = target_index {
                    let target = &board.prey[index];
                    pet.previous_vx = pet.vx;
                    pet.vx = (target.x - pet.x).clamp(-2.0, 2.0);
                    pet.x = (pet.x + pet.vx).clamp(20.0, 620.0);
                    pet.y = (pet.y + (target.y - pet.y).clamp(-2.0, 2.0)).clamp(55.0, 455.0);
                    if (target.x - pet.x).abs() < 20.0 && (target.y - pet.y).abs() < 20.0 {
                        board.prey.remove(index);
                        pet.timer = 0;
                    }
                }
            }
            PetKind::Stanley if pet.timer > 100 => {
                pet.timer = 0;
                let combat = &mut board.combat;
                let next_id = &mut board.next_object_id;
                if let Some(change) = combat.launch_stanley(
                    pet.x - 40.0,
                    pet.y - 40.0,
                    pet.vx,
                    || rand_range(u64::from(u32::MAX)) as u32,
                    || {
                        let id = *next_id;
                        *next_id = (*next_id).saturating_add(1);
                        id
                    },
                ) {
                    output.push(change);
                }
            }
            _ => {}
        }
    }
    output
}
