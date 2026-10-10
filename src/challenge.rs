//! Challenge shop and lower-time records. Board owns money, eggs, and ticks.

use crate::sim::TICK_MS;
use crate::{bonus::PurchaseReceipt, sim::PetKind};
use serde::{Deserialize, Serialize};

const UNAVAILABLE: i32 = -1;
const PRICE_CAP: i32 = 99_999;
const RESET_COOLDOWN: u16 = 500;
const BASE_PRICES: [[i32; 12]; 4] = [
    [100, -1, 200, 300, 1000, -1, -1, -1, -1, -1, 1000, 5000],
    [100, -1, 200, 300, -1, 250, 750, -1, -1, -1, 1000, 5000],
    [100, -1, 200, 300, -1, -1, -1, 750, 2000, -1, 2000, 15000],
    [-1, 200, 200, 300, 1000, -1, -1, -1, -1, 10000, 5000, 25000],
];

fn baselines(tank: u8) -> Option<&'static [i32; 12]> {
    BASE_PRICES.get(usize::from(tank.checked_sub(1)?))
}

pub fn initial_egg_price(tank: u8) -> Option<i32> {
    Some(baselines(tank)?[11])
}

pub fn inflation_period(tank: u8) -> Option<u64> {
    match tank {
        1 | 2 => Some(150),
        3 => Some(175),
        4 => Some(200),
        _ => None,
    }
}

pub fn shell_reward(tank: u8) -> Option<i32> {
    match tank {
        1 => Some(2000),
        2 => Some(5000),
        3 => Some(10000),
        4 => Some(20000),
        _ => None,
    }
}

/// Normal fixed-step result: whole seconds from active Board updates.
pub fn default_score_seconds(active_ticks: u64) -> i32 {
    let seconds = active_ticks.saturating_mul(u64::from(TICK_MS)) / 1000;
    seconds.min(i32::MAX as u64) as i32
}

/// Only non-egg prices are stored here. Board.egg_price owns slot 11.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChallengeShop {
    pub prices: [i32; 11],
    pub reset_cooldown: u16,
}

impl ChallengeShop {
    pub fn new(tank: u8) -> Option<Self> {
        let baseline = baselines(tank)?;
        let mut prices = [UNAVAILABLE; 11];
        prices.copy_from_slice(&baseline[..11]);
        Some(Self {
            prices,
            reset_cooldown: 0,
        })
    }

    pub fn validate(&self, tank: u8, egg_price: i32) -> Result<(), String> {
        let baseline = baselines(tank).ok_or("invalid Challenge tank")?;
        if self.reset_cooldown > RESET_COOLDOWN
            || self.prices.iter().zip(baseline).any(|(price, base)| {
                if *base == UNAVAILABLE {
                    *price != UNAVAILABLE
                } else {
                    !(*base..=PRICE_CAP).contains(price)
                }
            })
            || !(baseline[11]..=PRICE_CAP).contains(&egg_price)
        {
            return Err("invalid Challenge shop".into());
        }
        Ok(())
    }

    pub fn slot_price(&self, tank: u8, slot: usize, egg_price: i32) -> Option<i32> {
        let baseline = *baselines(tank)?.get(slot)?;
        if baseline == UNAVAILABLE {
            None
        } else if slot == 11 {
            Some(egg_price)
        } else {
            self.prices.get(slot).copied()
        }
    }

    /// Called before Board increments its active tick. Returns whether prices inflated.
    pub fn tick_active(
        &mut self,
        tank: u8,
        preincrement_tick: u64,
        project_eggs: u8,
        available: [bool; 12],
        egg_price: &mut i32,
    ) -> bool {
        let Some(baseline) = baselines(tank) else {
            return false;
        };
        let period = inflation_period(tank).expect("valid Challenge tank has period");
        // Board +3a4/+3a5 are action slots 0/1, regardless of tank.
        let first_pool_available = available[0] || available[1];
        let inflated = first_pool_available
            && self.reset_cooldown == 0
            && project_eggs < 5
            && preincrement_tick.is_multiple_of(period);
        if inflated {
            for (slot, (&base, &is_available)) in baseline.iter().zip(&available).enumerate() {
                if base == UNAVAILABLE || !is_available {
                    continue;
                }
                let price = if slot == 11 {
                    &mut *egg_price
                } else {
                    &mut self.prices[slot]
                };
                *price = price.saturating_add(base / 100).min(PRICE_CAP);
            }
        }
        if self.reset_cooldown > 0 {
            self.reset_cooldown -= 1;
        }
        inflated
    }

    /// Board calls after successful payment and piece-counter update.
    pub fn paid_reset(&mut self, tank: u8) -> Result<(), String> {
        let baseline = baselines(tank).ok_or("invalid Challenge tank")?;
        self.prices.copy_from_slice(&baseline[..11]);
        self.reset_cooldown = RESET_COOLDOWN;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChallengeScores {
    pub personal_best: [Option<i32>; 4],
    pub top_five: [Vec<i32>; 4],
}

impl Default for ChallengeScores {
    fn default() -> Self {
        Self::new()
    }
}

impl ChallengeScores {
    pub fn new() -> Self {
        Self {
            personal_best: [None; 4],
            top_five: std::array::from_fn(|_| Vec::new()),
        }
    }

    pub fn record(&mut self, tank: u8, seconds: i32) -> Result<bool, String> {
        let slot = usize::from(tank.checked_sub(1).ok_or("invalid Challenge tank")?);
        let best = self
            .personal_best
            .get_mut(slot)
            .ok_or("invalid Challenge tank")?;
        if seconds < 0 {
            return Err("invalid Challenge score".into());
        }
        let improved = best.is_none_or(|old| seconds < old);
        if improved {
            *best = Some(seconds);
        }
        let list = &mut self.top_five[slot];
        let index = list
            .iter()
            .position(|old| seconds < *old)
            .unwrap_or(list.len());
        if index < 5 {
            list.insert(index, seconds);
            list.truncate(5);
        }
        Ok(improved)
    }

    pub fn validate(&self) -> Result<(), String> {
        for (best, entries) in self.personal_best.iter().zip(&self.top_five) {
            if best.is_some_and(|score| score < 0)
                || entries.len() > 5
                || entries.iter().any(|score| *score < 0)
                || entries.windows(2).any(|pair| pair[0] > pair[1])
            {
                return Err("invalid Challenge records".into());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChallengeRun {
    pub tank: u8,
    pub initial_pets: Vec<PetKind>,
    #[serde(deserialize_with = "Option::<ChallengeResult>::deserialize")]
    pub result: Option<ChallengeResult>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChallengeResult {
    pub tank: u8,
    pub seconds: i32,
    pub personal_best: i32,
    pub shell_balance_before: u32,
    pub credited_shells: u32,
    pub updates: u32,
    pub purchase: PurchaseReceipt,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn challenge_source_tank_prices_periods_and_rewards() {
        let expected = [
            (
                [100, -1, 200, 300, 1000, -1, -1, -1, -1, -1, 1000],
                5000,
                150,
                2000,
            ),
            (
                [100, -1, 200, 300, -1, 250, 750, -1, -1, -1, 1000],
                5000,
                150,
                5000,
            ),
            (
                [100, -1, 200, 300, -1, -1, -1, 750, 2000, -1, 2000],
                15000,
                175,
                10000,
            ),
            (
                [-1, 200, 200, 300, 1000, -1, -1, -1, -1, 10000, 5000],
                25000,
                200,
                20000,
            ),
        ];
        for (index, (prices, egg, period, reward)) in expected.into_iter().enumerate() {
            let tank = index as u8 + 1;
            let shop = ChallengeShop::new(tank).unwrap();
            assert_eq!(shop.prices, prices);
            assert_eq!(initial_egg_price(tank), Some(egg));
            assert_eq!(inflation_period(tank), Some(period));
            assert_eq!(shell_reward(tank), Some(reward));
            shop.validate(tank, egg).unwrap();
        }
        assert!(ChallengeShop::new(0).is_none());
        assert!(ChallengeShop::new(5).is_none());
        assert_eq!(default_score_seconds(35), 0);
        assert_eq!(default_score_seconds(36), 1);
    }

    #[test]
    fn challenge_inflation_uses_zero_tick_or_gate_cooldown_and_source_availability() {
        // Supplied flags represent constructed widgets, not fresh-entry setup.
        let mut shop = ChallengeShop::new(4).unwrap();
        let mut egg = initial_egg_price(4).unwrap();
        let mut available = [false; 12];
        available[2] = true; // Food Quantity alone cannot open the first pool.
        available[3] = true;
        assert!(!shop.tick_active(4, 0, 0, available, &mut egg));
        available[1] = true; // Tank 4's Breeder is action slot 1.
        available[11] = true;
        assert!(shop.tick_active(4, 0, 0, available, &mut egg));
        assert_eq!(shop.prices[1], 202);
        assert_eq!(shop.prices[2], 202);
        assert_eq!(shop.prices[3], 303);
        assert_eq!(egg, 25250);
        assert!(!shop.tick_active(4, 199, 0, available, &mut egg));
        assert!(shop.tick_active(4, 200, 0, available, &mut egg));
        assert!(!shop.tick_active(4, 400, 5, available, &mut egg));
        shop.paid_reset(4).unwrap();
        assert_eq!(shop.prices[2], 200);
        assert_eq!(egg, 25500);
        assert_eq!(shop.reset_cooldown, 500);
        assert!(!shop.tick_active(4, 600, 0, available, &mut egg));
        assert_eq!(shop.reset_cooldown, 499);
    }

    #[test]
    fn challenge_inflation_caps_each_available_price_and_preserves_unavailable_slots() {
        let mut shop = ChallengeShop::new(1).unwrap();
        let mut egg = 99_998;
        shop.prices[0] = 99_998;
        shop.prices[2] = 99_998;
        let mut available = [true; 12];
        available[2] = false;
        assert!(shop.tick_active(1, 150, 0, available, &mut egg));
        assert_eq!(shop.prices[0], 99_999);
        assert_eq!(shop.prices[2], 99_998);
        assert_eq!(shop.prices[1], -1);
        assert_eq!(egg, 99_999);
    }

    #[test]
    fn challenge_scores_use_strict_personal_minimum_and_stable_ascending_five() {
        let mut scores = ChallengeScores::new();
        assert_eq!(scores.personal_best[0], None);
        for (time, improved) in [
            (12, true),
            (8, true),
            (8, false),
            (10, false),
            (14, false),
            (16, false),
        ] {
            assert_eq!(scores.record(1, time), Ok(improved));
        }
        assert_eq!(scores.personal_best[0], Some(8));
        assert_eq!(scores.top_five[0], [8, 8, 10, 12, 14]);
        assert_eq!(scores.record(1, 17), Ok(false));
        assert_eq!(scores.top_five[0], [8, 8, 10, 12, 14]);
        scores.validate().unwrap();
        scores.personal_best[0] = None; // Distinct record authorities remain valid.
        scores.validate().unwrap();
    }
}
