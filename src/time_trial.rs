//! Durable project Time Trial bookkeeping. Board actors remain owned by AdventureState.

use crate::bonus::PurchaseReceipt;
use crate::sim::PetKind;
use serde::{Deserialize, Serialize};

pub const TIME_TRIAL_EGG_PRICE: i32 = 100;
pub const TIME_TRIAL_MAX_EGG_PRICE: i32 = 99_999;
pub const TIME_TRIAL_START_BALANCE: i32 = 200;

pub fn limit_seconds(tank: u8) -> Option<u64> {
    match tank {
        1 => Some(300),
        2..=4 => Some(600),
        _ => None,
    }
}

pub const SUPPORTED_PETS: [PetKind; 19] = [
    PetKind::Stinky,
    PetKind::Niko,
    PetKind::Itchy,
    PetKind::Prego,
    PetKind::Zorf,
    PetKind::Clyde,
    PetKind::Vert,
    PetKind::Rufus,
    PetKind::Meryl,
    PetKind::Wadsworth,
    PetKind::Seymour,
    PetKind::Shrapnel,
    PetKind::Gumbo,
    PetKind::Blip,
    PetKind::Rhubarb,
    PetKind::Nimbus,
    PetKind::Amp,
    PetKind::Gash,
    PetKind::Angie,
];

/// Initial choices and Presto forms include purchased raw20–22. Pet eggs retain
/// the separate ordinary raw0..18 candidate set above.
pub fn selectable_pet(pet: PetKind) -> bool {
    SUPPORTED_PETS.contains(&pet)
        || matches!(
            pet,
            PetKind::Presto
                | PetKind::Brinkley
                | PetKind::Nostradamus
                | PetKind::Stanley
                | PetKind::Walter
        )
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TimeTrialScores {
    pub personal_best: [i32; 4],
    /// Project policy: descending score, stable order for equal scores.
    pub top_five: [Vec<i32>; 4],
}

impl TimeTrialScores {
    pub fn record(&mut self, tank: u8, score: i32) -> Result<bool, String> {
        let slot = usize::from(tank.checked_sub(1).ok_or("invalid Time Trial tank")?);
        let best = self
            .personal_best
            .get_mut(slot)
            .ok_or("invalid Time Trial tank")?;
        let improved = score > *best;
        if improved {
            *best = score;
        }
        let list = &mut self.top_five[slot];
        let index = list
            .iter()
            .position(|old| score > *old)
            .unwrap_or(list.len());
        if index < 5 {
            list.insert(index, score);
            list.truncate(5);
        }
        Ok(improved)
    }

    pub fn validate(&self) -> Result<(), String> {
        for (best, entries) in self.personal_best.iter().zip(&self.top_five) {
            if !(0..=9_999_999).contains(best)
                || entries.len() > 5
                || entries.iter().any(|score| !(0..=9_999_999).contains(score))
                || entries.windows(2).any(|pair| pair[0] < pair[1])
                || entries.first().copied().unwrap_or(0) != *best
            {
                return Err("invalid Time Trial records".into());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeTrialRun {
    pub tank: u8,
    pub initial_pets: Vec<PetKind>,
    pub acquired_pets: Vec<PetKind>,
    pub egg_purchases: u32,
    pub egg_maxed: bool,
    pub last_purchase_candidates: Option<u8>,
    pub result: Option<TimeTrialResult>,
}

pub fn price_after_purchases(count: u32) -> i32 {
    let mut price = TIME_TRIAL_EGG_PRICE;
    for _ in 0..count.min(10) {
        price = (price * 2).min(TIME_TRIAL_MAX_EGG_PRICE);
    }
    price
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeTrialResult {
    pub tank: u8,
    pub score: i32,
    pub settled_amount: i32,
    pub settled_ids: Vec<u64>,
    pub personal_best: i32,
    pub shell_balance_before: u32,
    pub credited_shells: u32,
    pub credited: bool,
    pub updates: u32,
    pub purchase: PurchaseReceipt,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn personal_best_requires_strict_improvement_and_top_five_keeps_stable_ties() {
        let mut scores = TimeTrialScores::default();
        for (score, improved) in [
            (100, true),
            (100, false),
            (80, false),
            (120, true),
            (60, false),
            (100, false),
        ] {
            assert_eq!(scores.record(2, score).unwrap(), improved);
        }
        assert_eq!(scores.personal_best[1], 120);
        assert_eq!(scores.top_five[1], [120, 100, 100, 100, 80]);
        scores.validate().unwrap();
    }
}
