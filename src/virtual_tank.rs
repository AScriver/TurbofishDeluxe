//! Durable Virtual Tank ownership and transactions. PB05 BR05/BR06 establish
//! slot, purchase, sale, and currency boundaries; W1 f919b3c supplies the
//! labelled fish/store properties. Board movement and retail calendar RNG live
//! outside this model. Offline time never advances fish simulation here.

use crate::bonus::MAX_SHELL_BALANCE;
use crate::sim::PetKind;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const FISH_CAPACITY: usize = 10;
pub const BUBBULATOR_CAPACITY: usize = 20;
pub const VIRTUAL_PET_LIMIT: usize = 7;
pub const BACKDROP_COUNT: usize = 6;
pub const STORE_SLOTS: usize = 8;
pub const BACKDROP_SALE_REFUND: u32 = 10_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VirtualFishKind {
    Guppy,
    Oscar,
    Ultra,
    Gekko,
    Penta,
    Grubber,
    Breeder,
    Sylvester,
    Ball,
    Bi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FishSize {
    Small,
    Medium,
    Large,
    Star,
    Crowned,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NamedFish {
    Rocky,
    Ludwig,
    Cookie,
    JohnnyV,
    Kilgore,
    Santa,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExoticDiet {
    None,
    Pizza,
    IceCream,
    Chicken,
    Guppy,
    Oscar,
    Ultra,
    Cookie,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FishColors {
    #[default]
    Normal,
    Special,
    Rainbow,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FishProperties {
    pub invisible: bool,
    pub voracious: bool,
    pub speedy: bool,
    pub singing: bool,
    pub backward: bool,
    pub exotic_diet: Option<ExoticDiet>,
    pub named: Option<NamedFish>,
    pub colors: FishColors,
    /// W1 SetFishColors receives a 0..999 roll; the palette conversion is
    /// performed by the fish actor, not this ownership model.
    pub color_roll: Option<u16>,
}

impl FishProperties {
    /// W1 GameObject::GetAttribute. The named-fish prefix has its own price.
    pub fn attribute(self) -> u8 {
        if self.named.is_some() {
            return 8;
        }
        let mut count = 0;
        let mut last = 9;
        for (active, id) in [
            (self.invisible, 0),
            (self.voracious, 1),
            (self.speedy, 2),
            (self.singing, 3),
            (self.backward, 4),
            (
                matches!(
                    self.exotic_diet,
                    Some(ExoticDiet::Pizza | ExoticDiet::IceCream | ExoticDiet::Chicken)
                ),
                5,
            ),
            (
                matches!(
                    self.exotic_diet,
                    Some(ExoticDiet::Guppy | ExoticDiet::Oscar | ExoticDiet::Ultra)
                ),
                6,
            ),
        ] {
            if active {
                count += 1;
                last = id;
            }
        }
        if count > 1 { 7 } else { last }
    }

    /// W1 StoreScreen::IsSpecialFishValid. This is stock admission, not a
    /// claim that arbitrary imported fish from the retail save are invalid.
    pub fn valid_for_store(self, kind: VirtualFishKind) -> bool {
        if self.voracious && matches!(kind, VirtualFishKind::Ball | VirtualFishKind::Penta) {
            return false;
        }
        if self.backward
            && matches!(
                kind,
                VirtualFishKind::Penta | VirtualFishKind::Grubber | VirtualFishKind::Ball
            )
        {
            return false;
        }
        if matches!(
            self.exotic_diet,
            Some(ExoticDiet::Pizza | ExoticDiet::IceCream | ExoticDiet::Chicken)
        ) && kind == VirtualFishKind::Ball
        {
            return false;
        }
        if matches!(
            self.exotic_diet,
            Some(ExoticDiet::Guppy | ExoticDiet::Oscar | ExoticDiet::Ultra)
        ) && matches!(kind, VirtualFishKind::Penta | VirtualFishKind::Grubber)
        {
            return false;
        }
        true
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FishOffer {
    pub kind: VirtualFishKind,
    pub properties: FishProperties,
    /// Constructor values supplied with the actual store product. Store RNG
    /// and constructor dispatch have not been joined to a project generator.
    pub initial_hunger: i32,
    pub initial_size: FishSize,
    pub food_needed_to_grow: u16,
    pub coin_threshold: u32,
}

impl FishOffer {
    /// W1 GameObject::GetShellCost, with all observed base-kind and attribute
    /// branches. The installed store handler reads the widget's price field.
    pub fn price(&self) -> u32 {
        let base = match self.kind {
            VirtualFishKind::Guppy => match self.properties.colors {
                FishColors::Normal => 25,
                FishColors::Special => 500,
                FishColors::Rainbow => 2_500,
            },
            VirtualFishKind::Oscar => match self.properties.colors {
                FishColors::Normal => 100,
                FishColors::Special => 1_000,
                FishColors::Rainbow => 5_000,
            },
            VirtualFishKind::Ultra | VirtualFishKind::Bi => 10_000,
            VirtualFishKind::Gekko => 3_500,
            VirtualFishKind::Penta | VirtualFishKind::Grubber => 2_500,
            VirtualFishKind::Breeder => 5_000,
            VirtualFishKind::Sylvester => 15_000,
            VirtualFishKind::Ball => 20_000,
        };
        match self.properties.attribute() {
            0 | 3 | 6 => 30_000,
            1 | 2 | 4 | 5 => 25_000,
            7 => 40_000,
            8 => 50_000,
            _ => ((base / 5) * 5).max(10),
        }
    }

    pub fn valid(&self) -> bool {
        (self.food_needed_to_grow > 0
            || matches!(self.kind, VirtualFishKind::Penta | VirtualFishKind::Grubber))
            && self.coin_threshold > 0
            && self.properties.valid_for_store(self.kind)
    }
}

/// W1 actor constructor *ranges*, with draws owned by the caller. The
/// original constructors also roll movement and appearance between these
/// fields, so this does not establish exact retail RNG-stream parity.
pub fn source_offer(
    kind: VirtualFishKind,
    properties: FishProperties,
    mut roll: impl FnMut() -> u32,
) -> FishOffer {
    let (initial_hunger, food_needed_to_grow, coin_threshold, initial_size) = match kind {
        VirtualFishKind::Guppy
        | VirtualFishKind::Gekko
        | VirtualFishKind::Sylvester
        | VirtualFishKind::Ball
        | VirtualFishKind::Bi => (
            (roll() % 200 + 400) as i32,
            (roll() % 9 + 12) as u16,
            roll() % 200 + 4_320,
            if kind == VirtualFishKind::Sylvester {
                FishSize::Medium
            } else {
                FishSize::Small
            },
        ),
        VirtualFishKind::Oscar | VirtualFishKind::Ultra => (
            (roll() % 200 + 600) as i32,
            (roll() % 9 + 12) as u16,
            roll() % 200 + 4_320,
            FishSize::Small,
        ),
        VirtualFishKind::Penta | VirtualFishKind::Grubber => (
            (roll() % 200 + 900) as i32,
            0,
            if kind == VirtualFishKind::Penta {
                3
            } else {
                roll() % 200 + 4_320
            },
            FishSize::Small,
        ),
        VirtualFishKind::Breeder => (
            (roll() % 200 + 400) as i32,
            (roll() % 9 + 12) as u16,
            roll() % 400 + 1_000,
            FishSize::Small,
        ),
    };
    FishOffer {
        kind,
        properties,
        initial_hunger,
        initial_size,
        food_needed_to_grow,
        coin_threshold,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FishBiography {
    /// W1 BoughtSetup rolls this only after a fish purchase commits.
    pub hometown: u16,
    pub likes: [u16; 3],
}

impl FishBiography {
    pub fn valid(self) -> bool {
        (1..=374).contains(&self.hometown)
            && self.likes.iter().all(|like| *like <= 330)
            && self.likes[0] != self.likes[1]
            && self.likes[0] != self.likes[2]
            && self.likes[1] != self.likes[2]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualFish {
    /// 0..19 purchased fish; a breeder child can temporarily use parent+100.
    pub id: u16,
    pub name: String,
    pub kind: VirtualFishKind,
    pub properties: FishProperties,
    pub shown: bool,
    pub alive: bool,
    pub purchased_at_seconds: u64,
    pub bought_today: bool,
    pub purchase_price: u32,
    pub size: FishSize,
    pub hunger: i32,
    pub food_ate: u16,
    pub food_needed_to_grow: u16,
    pub growth_ticks: u8,
    pub coin_ticks: u32,
    pub coin_threshold: u32,
    pub breeder_child: bool,
    pub hometown: u16,
    pub likes: [u16; 3],
    pub day_start_seconds: u64,
    pub last_mental_day_seconds: u64,
    pub fed_today: u8,
    pub mental_state: u8,
    pub has_given_birth: bool,
}

impl VirtualFish {
    /// W1 GameObject daily-state transition; accepts caller's local-day start
    /// and wall time. It does not age hunger, position, growth, or coin clocks.
    pub fn refresh_calendar(&mut self, now_seconds: u64, day_start_seconds: u64) {
        if self.bought_today
            && (now_seconds < self.purchased_at_seconds
                || now_seconds.saturating_sub(self.purchased_at_seconds) >= 86_400)
        {
            self.bought_today = false;
        }
        if day_start_seconds != self.day_start_seconds {
            // W1 Unk03 uses raw local-day-start seconds, then subtracts one.
            // That usually drains this 0..6 score on the next calendar day;
            // retail equivalence of this odd arithmetic remains unchecked.
            if day_start_seconds < self.last_mental_day_seconds {
                self.last_mental_day_seconds = day_start_seconds;
            }
            let decay = day_start_seconds
                .saturating_sub(self.last_mental_day_seconds)
                .saturating_sub(1);
            self.mental_state = self
                .mental_state
                .saturating_sub(decay.min(u8::MAX as u64) as u8);
            self.fed_today = 0;
            self.day_start_seconds = day_start_seconds;
        }
    }

    pub fn record_qualifying_feed(&mut self, now_seconds: u64, day_start_seconds: u64) -> bool {
        self.refresh_calendar(now_seconds, day_start_seconds);
        if self.fed_today >= 3 {
            return false;
        }
        self.fed_today += 1;
        if self.fed_today == 3 {
            self.mental_state = self.mental_state.saturating_add(1).min(6);
            self.last_mental_day_seconds = day_start_seconds;
        }
        self.properties.named != Some(NamedFish::Rocky)
    }

    /// W1 GameObject::GetShellPrice blocks recent purchases while the sale
    /// grace rule is active. Breeder offspring bypass it and gain growth
    /// multipliers; other species apply only their own overrides.
    pub fn sale_value(&self, now_seconds: u64, sale_grace_enabled: bool) -> Option<u32> {
        if sale_grace_enabled
            && self.bought_today
            && (now_seconds < self.purchased_at_seconds
                || now_seconds - self.purchased_at_seconds <= 86_400)
            && !self.breeder_child
        {
            return None;
        }
        let half = self.purchase_price / 2;
        let multiplier = match (self.kind, self.size) {
            (VirtualFishKind::Guppy, FishSize::Medium) if self.breeder_child => 2,
            (VirtualFishKind::Guppy, FishSize::Large) if self.breeder_child => 3,
            (VirtualFishKind::Guppy, FishSize::Crowned) if self.breeder_child => 5,
            (VirtualFishKind::Breeder | VirtualFishKind::Ball, FishSize::Medium) => 2,
            (VirtualFishKind::Breeder | VirtualFishKind::Ball, FishSize::Large) => 3,
            (VirtualFishKind::Sylvester, FishSize::Large) => 3,
            _ => 1,
        };
        Some(half.saturating_mul(multiplier))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoreProduct {
    Fish(FishOffer),
    Bubbulator,
    Backdrop(u8),
    AlienAttractor,
    FoodUpgrade,
}

impl StoreProduct {
    pub fn price(&self) -> u32 {
        match self {
            Self::Fish(fish) => fish.price(),
            Self::Bubbulator | Self::FoodUpgrade => 20_000,
            Self::Backdrop(_) => 15_000,
            Self::AlienAttractor => 50_000,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoreStock {
    /// Generated by the caller from a pinned calendar/PRNG stream. Same key
    /// retains bought slots; a new key resets them, as W1 ProfileMgr does.
    pub cycle: u64,
    pub offers: [Option<StoreProduct>; STORE_SLOTS],
    pub bought: [bool; STORE_SLOTS],
}

fn roll_below(roll: &mut impl FnMut() -> u32, upper: u32) -> u32 {
    roll() % upper
}

fn with_random_colors(roll: &mut impl FnMut() -> u32) -> FishProperties {
    FishProperties {
        colors: if roll_below(roll, 30) == 0 {
            FishColors::Rainbow
        } else {
            FishColors::Special
        },
        color_roll: Some(roll_below(roll, 1_000) as u16),
        ..FishProperties::default()
    }
}

fn named_product(
    name: NamedFish,
    roll: &mut impl FnMut() -> u32,
) -> (VirtualFishKind, FishProperties) {
    let mut properties = FishProperties {
        named: Some(name),
        ..FishProperties::default()
    };
    let kind = match name {
        NamedFish::Rocky => {
            properties.colors = FishColors::Special;
            properties.voracious = true;
            properties.exotic_diet = Some(ExoticDiet::Ultra);
            VirtualFishKind::Guppy
        }
        NamedFish::Ludwig => {
            properties.colors = FishColors::Special;
            properties.singing = true;
            VirtualFishKind::Oscar
        }
        NamedFish::Cookie => {
            properties.exotic_diet = Some(ExoticDiet::Cookie);
            VirtualFishKind::Gekko
        }
        NamedFish::JohnnyV => {
            properties.colors = FishColors::Rainbow;
            properties.color_roll = Some(roll_below(roll, 1_000) as u16);
            properties.exotic_diet = Some(ExoticDiet::Pizza);
            VirtualFishKind::Oscar
        }
        NamedFish::Kilgore => {
            properties.voracious = true;
            properties.singing = true;
            VirtualFishKind::Ultra
        }
        // W1 MakeSpecialFish has no Santa constructor. It is never selected
        // by the inspected GetSpecial8Fish candidate list.
        NamedFish::Santa => VirtualFishKind::Guppy,
    };
    (kind, properties)
}

fn random_special_fish(
    roll: &mut impl FnMut() -> u32,
    possible_classic: bool,
    skip: Option<VirtualFishKind>,
    trait_code: u8,
) -> (VirtualFishKind, FishProperties) {
    if possible_classic && roll_below(roll, 100) <= 75 {
        let kind = if roll_below(roll, 2) == 0 {
            VirtualFishKind::Guppy
        } else {
            VirtualFishKind::Oscar
        };
        let properties = if roll_below(roll, 100) <= 75 {
            with_random_colors(roll)
        } else {
            FishProperties::default()
        };
        return (kind, properties);
    }
    const WEIGHTS: [(u32, VirtualFishKind); 8] = [
        (26, VirtualFishKind::Penta),
        (25, VirtualFishKind::Grubber),
        (25, VirtualFishKind::Gekko),
        (10, VirtualFishKind::Breeder),
        (5, VirtualFishKind::Ultra),
        (3, VirtualFishKind::Bi),
        (3, VirtualFishKind::Ball),
        (3, VirtualFishKind::Sylvester),
    ];
    for _ in 0..100 {
        let draw = roll_below(roll, 100);
        let mut cumulative = 0;
        let kind = WEIGHTS
            .iter()
            .find_map(|(weight, kind)| {
                cumulative += *weight;
                (draw < cumulative).then_some(*kind)
            })
            .unwrap_or(VirtualFishKind::Penta);
        let compatible = match trait_code {
            1 => !matches!(kind, VirtualFishKind::Ball | VirtualFishKind::Penta),
            4 => !matches!(
                kind,
                VirtualFishKind::Penta | VirtualFishKind::Grubber | VirtualFishKind::Ball
            ),
            5 => kind != VirtualFishKind::Ball,
            6 => !matches!(kind, VirtualFishKind::Penta | VirtualFishKind::Grubber),
            _ => true,
        };
        if Some(kind) != skip && compatible {
            return (kind, FishProperties::default());
        }
    }
    // W1 retries without the skip after 100 attempts. This bounded fallback
    // also terminates for deliberately constant caller RNGs.
    let kind = WEIGHTS
        .iter()
        .find(|(_, kind)| Some(*kind) != skip)
        .map_or(VirtualFishKind::Gekko, |(_, kind)| *kind);
    (kind, FishProperties::default())
}

fn deterministic_trait_code(cycle: u64) -> u8 {
    let index = cycle % 100;
    if index.is_multiple_of(10) {
        return (((index / 10) & 1) + 7) as u8;
    }
    if index.is_multiple_of(2) {
        return 5;
    }
    match index % 20 {
        1 | 5 | 11 | 15 => 3,
        3 | 13 => 1,
        7 => 2,
        9 => 0,
        17 => 6,
        19 => 4,
        _ => 5,
    }
}

fn random_trait_code(roll: &mut impl FnMut() -> u32) -> u8 {
    let draw = roll_below(roll, 100);
    match draw {
        0..=47 => 5,
        48..=67 => 1,
        68..=77 => 3,
        78..=82 => 2,
        83..=87 => 0,
        88..=92 => 6,
        93..=97 => 4,
        _ => 7,
    }
}

fn choose_exotic_diet(kind: VirtualFishKind, roll: &mut impl FnMut() -> u32) -> ExoticDiet {
    let allowed: &[ExoticDiet] = match kind {
        VirtualFishKind::Ultra => &[ExoticDiet::Guppy, ExoticDiet::Ultra],
        VirtualFishKind::Oscar | VirtualFishKind::Grubber | VirtualFishKind::Bi => {
            &[ExoticDiet::Oscar, ExoticDiet::Ultra]
        }
        VirtualFishKind::Sylvester => &[ExoticDiet::Guppy],
        _ => &[ExoticDiet::Guppy, ExoticDiet::Oscar, ExoticDiet::Ultra],
    };
    for _ in 0..64 {
        let candidate = match roll_below(roll, 3) {
            0 => ExoticDiet::Guppy,
            1 => ExoticDiet::Oscar,
            _ => ExoticDiet::Ultra,
        };
        if allowed.contains(&candidate) {
            return candidate;
        }
    }
    allowed[0]
}

fn apply_trait_code(
    mut properties: FishProperties,
    kind: VirtualFishKind,
    code: u8,
    roll: &mut impl FnMut() -> u32,
) -> FishProperties {
    match code {
        0 => properties.invisible = true,
        1 => properties.voracious = true,
        2 => properties.speedy = true,
        3 => properties.singing = true,
        4 => properties.backward = true,
        5 => {
            properties.exotic_diet = Some(match roll_below(roll, 3) {
                0 => ExoticDiet::Pizza,
                1 => ExoticDiet::IceCream,
                _ => ExoticDiet::Chicken,
            });
        }
        6 => properties.exotic_diet = Some(choose_exotic_diet(kind, roll)),
        7 | 8 => match roll_below(roll, 7) {
            0 => {
                properties.invisible = true;
                properties.voracious = true;
            }
            1 => {
                properties.invisible = true;
                properties.singing = true;
            }
            2 => {
                properties.voracious = true;
                properties.singing = true;
            }
            3 => {
                properties.backward = true;
                properties.speedy = true;
            }
            4 => {
                properties.voracious = true;
                properties.speedy = true;
            }
            5 => {
                properties.invisible = true;
                properties.voracious = true;
                properties.singing = true;
            }
            _ => {
                match roll_below(roll, 5) {
                    0 => properties.invisible = true,
                    1 => properties.voracious = true,
                    2 => properties.speedy = true,
                    3 => properties.singing = true,
                    _ => properties.backward = true,
                }
                let diet_code = if roll_below(roll, 2) == 0 { 6 } else { 5 };
                properties = apply_trait_code(properties, kind, diet_code, roll);
            }
        },
        _ => {}
    }
    properties
}

fn special_properties(
    kind: VirtualFishKind,
    first_code: u8,
    deterministic: bool,
    roll: &mut impl FnMut() -> u32,
) -> Result<FishProperties, VirtualTankError> {
    for attempt in 0..100 {
        let code = if deterministic && attempt <= 50 {
            first_code
        } else {
            random_trait_code(roll)
        };
        let properties = apply_trait_code(FishProperties::default(), kind, code, roll);
        if properties.valid_for_store(kind) {
            return Ok(properties);
        }
    }
    Err(VirtualTankError::InvalidOffer)
}

fn add_store_fish(
    offers: &mut [Option<StoreProduct>; STORE_SLOTS],
    slot: usize,
    kind: VirtualFishKind,
    properties: FishProperties,
    construct: &mut impl FnMut(VirtualFishKind, FishProperties) -> FishOffer,
) -> Result<(), VirtualTankError> {
    let offer = construct(kind, properties);
    if offer.kind != kind || offer.properties != properties || !offer.valid() {
        return Err(VirtualTankError::InvalidOffer);
    }
    offers[slot] = Some(StoreProduct::Fish(offer));
    Ok(())
}

/// W1 StoreScreen slot/category/rarity shape with caller-owned deterministic
/// draws and actor constructors. Reset `roll` from the same cycle seed to
/// reproduce a stock; installed MTRand parity has not been established.
pub fn generate_store_stock(
    tank: &VirtualTankState,
    cycle: u64,
    mut roll: impl FnMut() -> u32,
    mut construct: impl FnMut(VirtualFishKind, FishProperties) -> FishOffer,
) -> Result<[Option<StoreProduct>; STORE_SLOTS], VirtualTankError> {
    let mut offers: [Option<StoreProduct>; STORE_SLOTS] = std::array::from_fn(|_| None);
    add_store_fish(
        &mut offers,
        0,
        VirtualFishKind::Guppy,
        FishProperties::default(),
        &mut construct,
    )?;
    add_store_fish(
        &mut offers,
        1,
        VirtualFishKind::Oscar,
        FishProperties::default(),
        &mut construct,
    )?;
    add_store_fish(
        &mut offers,
        3,
        VirtualFishKind::Guppy,
        with_random_colors(&mut roll),
        &mut construct,
    )?;
    add_store_fish(
        &mut offers,
        4,
        VirtualFishKind::Oscar,
        with_random_colors(&mut roll),
        &mut construct,
    )?;
    let (kind5, properties5) = random_special_fish(&mut roll, false, None, 9);
    add_store_fish(&mut offers, 5, kind5, properties5, &mut construct)?;
    let code6 = deterministic_trait_code(cycle);
    let named_candidates = [
        NamedFish::Rocky,
        NamedFish::Ludwig,
        NamedFish::JohnnyV,
        NamedFish::Kilgore,
    ]
    .into_iter()
    .filter(|name| {
        !tank
            .active_fish()
            .any(|fish| fish.properties.named == Some(*name))
    })
    .collect::<Vec<_>>();
    let (kind6, properties6) = if code6 == 8 && !named_candidates.is_empty() {
        named_product(
            named_candidates[roll_below(&mut roll, named_candidates.len() as u32) as usize],
            &mut roll,
        )
    } else {
        let (kind, mut properties) = random_special_fish(&mut roll, true, Some(kind5), code6);
        let traits = special_properties(kind, code6, true, &mut roll)?;
        properties.invisible = traits.invisible;
        properties.voracious = traits.voracious;
        properties.speedy = traits.speedy;
        properties.singing = traits.singing;
        properties.backward = traits.backward;
        properties.exotic_diet = traits.exotic_diet;
        (kind, properties)
    };
    add_store_fish(&mut offers, 6, kind6, properties6, &mut construct)?;
    if !tank.bubbulator_owned {
        offers[2] = Some(StoreProduct::Bubbulator);
    } else if !tank.alien_attractor_owned {
        offers[2] = Some(StoreProduct::AlienAttractor);
    } else if !tank.food_upgrade_owned {
        offers[2] = Some(StoreProduct::FoodUpgrade);
    } else if !tank
        .active_fish()
        .any(|fish| fish.properties.named == Some(NamedFish::Cookie))
    {
        let (kind, properties) = named_product(NamedFish::Cookie, &mut roll);
        add_store_fish(&mut offers, 2, kind, properties, &mut construct)?;
    } else {
        let (kind, mut properties) = random_special_fish(&mut roll, false, None, 9);
        if roll_below(&mut roll, 2) == 0 {
            let code = random_trait_code(&mut roll);
            properties = special_properties(kind, code, false, &mut roll)?;
        }
        add_store_fish(&mut offers, 2, kind, properties, &mut construct)?;
    }
    let mut backdrops = Vec::new();
    for index in 0..BACKDROP_COUNT {
        if !tank.owned_backdrops[index] && (index != 5 || roll_below(&mut roll, 5) == 0) {
            backdrops.push(index as u8);
        }
    }
    if backdrops.is_empty() {
        let (kind, mut properties) = random_special_fish(&mut roll, true, None, 9);
        if roll_below(&mut roll, 2) == 0 {
            let code = random_trait_code(&mut roll);
            let traits = special_properties(kind, code, false, &mut roll)?;
            properties.invisible = traits.invisible;
            properties.voracious = traits.voracious;
            properties.speedy = traits.speedy;
            properties.singing = traits.singing;
            properties.backward = traits.backward;
            properties.exotic_diet = traits.exotic_diet;
        }
        add_store_fish(&mut offers, 7, kind, properties, &mut construct)?;
    } else {
        offers[7] = Some(StoreProduct::Backdrop(
            backdrops[(cycle as usize) % backdrops.len()],
        ));
    }
    let stock = StoreStock {
        cycle,
        offers: offers.clone(),
        bought: [false; STORE_SLOTS],
    };
    if !stock.valid() {
        return Err(VirtualTankError::InvalidOffer);
    }
    Ok(offers)
}

impl StoreStock {
    pub fn valid(&self) -> bool {
        self.offers.iter().enumerate().all(|(slot, product)| {
            product.as_ref().is_none_or(|product| {
                !matches!(product, StoreProduct::Fish(fish) if !fish.valid())
                    && !matches!(product, StoreProduct::Backdrop(index) if *index as usize >= BACKDROP_COUNT)
                    && (!matches!(slot, 0 | 1 | 3 | 4 | 5 | 6) || matches!(product, StoreProduct::Fish(_)))
                    && (slot != 2 || !matches!(product, StoreProduct::Backdrop(_)))
                    && (slot != 7 || matches!(product, StoreProduct::Fish(_) | StoreProduct::Backdrop(_)))
            })
        }) && self.bought.iter().enumerate().all(|(slot, bought)| !*bought || self.offers[slot].is_some())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualSettings {
    pub show_names: bool,
    pub bubbulator_shown: bool,
    pub alien_attractor_shown: bool,
    pub drop_shells: bool,
    pub always_show_hungry: bool,
}

impl Default for VirtualSettings {
    fn default() -> Self {
        Self {
            show_names: true,
            bubbulator_shown: false,
            alien_attractor_shown: false,
            drop_shells: true,
            always_show_hungry: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualTankState {
    pub fish: Vec<VirtualFish>,
    pub selected_fish: Option<u16>,
    pub selected_pets: Vec<PetKind>,
    pub owned_backdrops: [bool; BACKDROP_COUNT],
    pub backdrop: u8,
    pub bubbulator_owned: bool,
    pub alien_attractor_owned: bool,
    pub food_upgrade_owned: bool,
    pub settings: VirtualSettings,
    pub stock: Option<StoreStock>,
    /// Board+400 accumulation remains separate from the Profile shell wallet.
    pub board_shells: u32,
}

impl Default for VirtualTankState {
    fn default() -> Self {
        Self {
            fish: Vec::new(),
            selected_fish: None,
            selected_pets: Vec::new(),
            owned_backdrops: [true, false, false, false, false, false],
            backdrop: 0,
            bubbulator_owned: false,
            alien_attractor_owned: false,
            food_upgrade_owned: false,
            settings: VirtualSettings::default(),
            stock: None,
            board_shells: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VirtualTankError {
    InvalidState,
    InvalidOffer,
    StaleOffer,
    SoldOut,
    InsufficientShells,
    FishTankFull,
    InvalidName,
    FishMissing,
    FishNotSellable,
    LastBackdrop,
    BackdropNotOwned,
    PetSelection,
    AccessoryRequired,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PurchaseConfirmation {
    pub cycle: u64,
    pub slot: usize,
    pub expected_price: u32,
    pub accepted: bool,
    pub fish_name: Option<String>,
    /// Generated for an accepted fish checkout, not for daily stock display.
    pub fish_biography: Option<FishBiography>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PurchaseResult {
    Cancelled,
    Fish { id: u16, price: u32 },
    Accessory { price: u32 },
    Backdrop { index: u8, price: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuppyFood {
    Basic,
    Upgraded,
    Snot,
    Star,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeedResult {
    Fed,
    Grew(FishSize),
    Died,
}

/// W1 Coin::GetValue mode-5 values. PB05 `004d53a0` is still the exact
/// primary comparison gap; these values are secondary-source-derived.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VirtualCoinKind {
    Silver,
    Gold,
    Star,
    Diamond,
    Pearl,
    Treasure,
    NikoPearl,
    ShrapnelBomb,
    Peanut,
}

impl VirtualCoinKind {
    pub fn shell_value(self, combo: u8) -> u32 {
        let unit: u32 = match self {
            Self::Silver | Self::Peanut => 1,
            Self::Gold => 2,
            Self::Star => 3,
            Self::Diamond | Self::ShrapnelBomb => 5,
            Self::Pearl | Self::NikoPearl => 10,
            Self::Treasure => 20,
        };
        unit.saturating_mul(u32::from(combo))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VirtualProduction {
    Coin(VirtualCoinKind),
    CookieFoodDue,
    BreederBirthDue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OfflineReopen {
    /// W1 Board::Unk15 removes transient nonvirtual actors on loads older
    /// than 120 seconds. Board must apply this to its own actor collections.
    pub clear_ephemeral_actors: bool,
}

impl VirtualFish {
    /// Board determines when actor's hunger updater executes (food/alien and
    /// song gates differ by kind). A nonnegative virtual identity bypasses
    /// ordinary ShouldDie; special-food death still uses mark_fish_dead.
    pub fn tick_hunger(&mut self, should_decrement: bool) {
        if self.alive && should_decrement {
            self.hunger = self.hunger.saturating_sub(1).max(-1_000);
        }
    }

    fn standard_coin(&self) -> Option<VirtualCoinKind> {
        match self.kind {
            VirtualFishKind::Guppy if self.properties.named == Some(NamedFish::Rocky) => {
                Some(VirtualCoinKind::Treasure)
            }
            VirtualFishKind::Guppy if self.properties.named == Some(NamedFish::Santa) => matches!(
                self.size,
                FishSize::Large | FishSize::Star | FishSize::Crowned
            )
            .then_some(VirtualCoinKind::Treasure),
            VirtualFishKind::Guppy if self.breeder_child => Some(match self.size {
                FishSize::Medium => VirtualCoinKind::Diamond,
                FishSize::Large => VirtualCoinKind::Pearl,
                FishSize::Crowned => VirtualCoinKind::Treasure,
                _ => VirtualCoinKind::Gold,
            }),
            VirtualFishKind::Guppy => match self.size {
                FishSize::Small => None,
                FishSize::Medium => Some(VirtualCoinKind::Silver),
                FishSize::Large => Some(VirtualCoinKind::Gold),
                FishSize::Star => Some(VirtualCoinKind::Star),
                FishSize::Crowned => Some(VirtualCoinKind::Diamond),
            },
            VirtualFishKind::Oscar | VirtualFishKind::Penta => Some(VirtualCoinKind::Diamond),
            VirtualFishKind::Ultra => Some(VirtualCoinKind::Treasure),
            VirtualFishKind::Gekko if self.properties.named == Some(NamedFish::Cookie) => None,
            VirtualFishKind::Gekko | VirtualFishKind::Sylvester
                if self.size == FishSize::Medium =>
            {
                Some(VirtualCoinKind::Pearl)
            }
            VirtualFishKind::Gekko => Some(VirtualCoinKind::Pearl),
            VirtualFishKind::Sylvester => Some(VirtualCoinKind::Treasure),
            VirtualFishKind::Bi => Some(VirtualCoinKind::Gold),
            VirtualFishKind::Ball => Some(match self.size {
                FishSize::Medium => VirtualCoinKind::Gold,
                FishSize::Large => VirtualCoinKind::Diamond,
                _ => VirtualCoinKind::Silver,
            }),
            VirtualFishKind::Grubber | VirtualFishKind::Breeder => None,
        }
    }

    /// Tick only after Board's active/pause/update-order gate. The source
    /// timer is reduced by 540 per mental point, with a 360-tick floor; it
    /// never grants shells until an actual coin is collected.
    pub fn tick_production(
        &mut self,
        drop_shells: bool,
        screensaver: bool,
        screensaver_limit: u32,
        board_shells: u32,
        meryl_active: bool,
    ) -> Option<VirtualProduction> {
        if !self.alive {
            return None;
        }
        if self.kind == VirtualFishKind::Breeder && self.size == FishSize::Small {
            return None;
        }
        self.coin_ticks = self.coin_ticks.saturating_add(1);
        if self.kind == VirtualFishKind::Breeder {
            if self.coin_ticks < self.coin_threshold {
                return None;
            }
            self.coin_ticks = 0;
            self.coin_threshold = 2_160;
            return (self.size == FishSize::Large).then_some(VirtualProduction::BreederBirthDue);
        }
        if self.kind == VirtualFishKind::Penta {
            if self.coin_ticks <= 2 {
                return None;
            }
            self.coin_ticks = 0;
            return (drop_shells && (!screensaver || board_shells <= screensaver_limit))
                .then_some(VirtualProduction::Coin(VirtualCoinKind::Diamond));
        }
        if self.properties.named == Some(NamedFish::Cookie) && self.kind == VirtualFishKind::Gekko {
            if self.coin_ticks > 540 {
                self.coin_ticks = 0;
                return Some(VirtualProduction::CookieFoodDue);
            }
            return None;
        }
        let base = if self.kind == VirtualFishKind::Guppy && meryl_active {
            35
        } else {
            self.coin_threshold
        };
        let threshold = if base > 360 {
            base.saturating_sub(u32::from(self.mental_state) * 540)
                .max(360)
        } else {
            base
        };
        if self.coin_ticks < threshold
            || !drop_shells
            || (screensaver && board_shells > screensaver_limit)
            || self.hunger < 0
        {
            return None;
        }
        self.coin_ticks = 0;
        self.standard_coin().map(VirtualProduction::Coin)
    }
}

impl VirtualTankState {
    /// W1 Board::LoadGame applies timestamp cleanup and Unk13 metadata update.
    /// It does not run elapsed simulation ticks for persistent fish.
    pub fn resume_from_save(
        &mut self,
        saved_at_seconds: u64,
        now_seconds: u64,
        day_start_seconds: u64,
    ) -> OfflineReopen {
        for fish in &mut self.fish {
            if fish.alive {
                fish.refresh_calendar(now_seconds, day_start_seconds);
            }
        }
        OfflineReopen {
            clear_ephemeral_actors: now_seconds.saturating_sub(saved_at_seconds) > 120,
        }
    }

    pub fn fish_capacity(&self) -> usize {
        if self.bubbulator_owned {
            BUBBULATOR_CAPACITY
        } else {
            FISH_CAPACITY
        }
    }

    pub fn next_fish_id(&self) -> Option<u16> {
        (0..self.fish_capacity() as u16)
            .find(|id| !self.fish.iter().any(|fish| fish.alive && fish.id == *id))
    }

    pub fn active_fish(&self) -> impl Iterator<Item = &VirtualFish> {
        self.fish.iter().filter(|fish| fish.alive)
    }

    /// Check deserialized/project state before mutation. A dead record may
    /// retain an old ID, but active records and breeder children stay unique.
    pub fn validate(&self) -> Result<(), VirtualTankError> {
        if self.backdrop as usize >= BACKDROP_COUNT
            || !self.owned_backdrops[self.backdrop as usize]
            || !self.owned_backdrops.iter().any(|owned| *owned)
            || (self.settings.bubbulator_shown && !self.bubbulator_owned)
            || (self.settings.alien_attractor_shown && !self.alien_attractor_owned)
            || self.selected_pets.len() > VIRTUAL_PET_LIMIT
            || self.stock.as_ref().is_some_and(|stock| !stock.valid())
        {
            return Err(VirtualTankError::InvalidState);
        }
        let mut ids = HashSet::new();
        for fish in self.active_fish() {
            if fish.name.trim().is_empty()
                || (fish.food_needed_to_grow == 0
                    && !matches!(fish.kind, VirtualFishKind::Penta | VirtualFishKind::Grubber))
                || fish.coin_threshold == 0
                || (fish.id < 100 && fish.id as usize >= self.fish_capacity())
                || (fish.id >= 100 && fish.id - 100 >= BUBBULATOR_CAPACITY as u16)
                || !ids.insert(fish.id)
            {
                return Err(VirtualTankError::InvalidState);
            }
        }
        if self
            .selected_fish
            .is_some_and(|id| id as usize >= self.fish_capacity() || !ids.contains(&id))
            || self
                .selected_pets
                .iter()
                .enumerate()
                .any(|(index, pet)| self.selected_pets[..index].contains(pet))
        {
            return Err(VirtualTankError::InvalidState);
        }
        Ok(())
    }

    /// Stock creation is separate from checkout: W1's day seed and MTRand
    /// remain secondary and do not share this project's fixed-step RNG.
    pub fn install_stock(
        &mut self,
        cycle: u64,
        offers: [Option<StoreProduct>; STORE_SLOTS],
    ) -> Result<(), VirtualTankError> {
        let next = StoreStock {
            cycle,
            offers,
            bought: [false; STORE_SLOTS],
        };
        if !next.valid() {
            return Err(VirtualTankError::InvalidOffer);
        }
        if let Some(stock) = &self.stock
            && stock.cycle == cycle
        {
            return if stock.offers == next.offers {
                Ok(())
            } else {
                Err(VirtualTankError::StaleOffer)
            };
        }
        self.stock = Some(next);
        Ok(())
    }

    /// Every fallible check precedes debit/attachment. Retail debits before
    /// object creation and does not prove rollback on creation failure.
    pub fn confirm_purchase(
        &mut self,
        shells: &mut u32,
        confirmation: PurchaseConfirmation,
        now_seconds: u64,
        day_start_seconds: u64,
    ) -> Result<PurchaseResult, VirtualTankError> {
        if !confirmation.accepted {
            return Ok(PurchaseResult::Cancelled);
        }
        self.validate()?;
        let stock = self.stock.as_ref().ok_or(VirtualTankError::StaleOffer)?;
        if stock.cycle != confirmation.cycle || confirmation.slot >= STORE_SLOTS {
            return Err(VirtualTankError::StaleOffer);
        }
        if stock.bought[confirmation.slot] {
            return Err(VirtualTankError::SoldOut);
        }
        let product = stock.offers[confirmation.slot]
            .clone()
            .ok_or(VirtualTankError::SoldOut)?;
        let price = product.price();
        if price != confirmation.expected_price {
            return Err(VirtualTankError::StaleOffer);
        }
        if *shells < price {
            return Err(VirtualTankError::InsufficientShells);
        }
        let fish_id = match &product {
            StoreProduct::Fish(_) => {
                Some(self.next_fish_id().ok_or(VirtualTankError::FishTankFull)?)
            }
            StoreProduct::Bubbulator if self.bubbulator_owned => {
                return Err(VirtualTankError::SoldOut);
            }
            StoreProduct::Backdrop(index) if self.owned_backdrops[*index as usize] => {
                return Err(VirtualTankError::SoldOut);
            }
            StoreProduct::AlienAttractor if self.alien_attractor_owned => {
                return Err(VirtualTankError::SoldOut);
            }
            StoreProduct::FoodUpgrade if self.food_upgrade_owned => {
                return Err(VirtualTankError::SoldOut);
            }
            _ => None,
        };
        let fish_name = if fish_id.is_some() {
            let name = confirmation
                .fish_name
                .as_deref()
                .ok_or(VirtualTankError::InvalidName)?
                .trim();
            if name.is_empty() {
                return Err(VirtualTankError::InvalidName);
            }
            Some(name.to_owned())
        } else {
            None
        };
        let biography = if fish_id.is_some() {
            let biography = confirmation
                .fish_biography
                .ok_or(VirtualTankError::InvalidOffer)?;
            if !biography.valid() {
                return Err(VirtualTankError::InvalidOffer);
            }
            Some(biography)
        } else {
            None
        };
        *shells -= price;
        self.stock.as_mut().expect("stock checked").bought[confirmation.slot] = true;
        match product {
            StoreProduct::Fish(offer) => {
                let id = fish_id.expect("fish slot checked");
                self.fish.retain(|fish| fish.alive || fish.id != id);
                self.fish.push(VirtualFish {
                    id,
                    name: fish_name.expect("fish name checked"),
                    kind: offer.kind,
                    properties: offer.properties,
                    shown: true,
                    alive: true,
                    purchased_at_seconds: now_seconds,
                    bought_today: true,
                    purchase_price: price,
                    size: offer.initial_size,
                    hunger: offer.initial_hunger,
                    food_ate: 0,
                    food_needed_to_grow: offer.food_needed_to_grow,
                    growth_ticks: 0,
                    coin_ticks: 0,
                    coin_threshold: offer.coin_threshold,
                    breeder_child: false,
                    hometown: biography.expect("fish biography checked").hometown,
                    likes: biography.expect("fish biography checked").likes,
                    day_start_seconds,
                    last_mental_day_seconds: day_start_seconds.saturating_sub(1),
                    fed_today: 0,
                    mental_state: 3,
                    has_given_birth: false,
                });
                Ok(PurchaseResult::Fish { id, price })
            }
            StoreProduct::Bubbulator => {
                self.bubbulator_owned = true;
                self.settings.bubbulator_shown = true;
                Ok(PurchaseResult::Accessory { price })
            }
            StoreProduct::Backdrop(index) => {
                self.owned_backdrops[index as usize] = true;
                self.backdrop = index;
                Ok(PurchaseResult::Backdrop { index, price })
            }
            StoreProduct::AlienAttractor => {
                self.alien_attractor_owned = true;
                self.settings.alien_attractor_shown = true;
                Ok(PurchaseResult::Accessory { price })
            }
            StoreProduct::FoodUpgrade => {
                self.food_upgrade_owned = true;
                Ok(PurchaseResult::Accessory { price })
            }
        }
    }

    pub fn select_fish(&mut self, id: u16) -> Result<(), VirtualTankError> {
        if id as usize >= self.fish_capacity() || !self.active_fish().any(|fish| fish.id == id) {
            return Err(VirtualTankError::FishMissing);
        }
        self.selected_fish = Some(id);
        Ok(())
    }

    pub fn rename_fish(&mut self, id: u16, name: &str) -> Result<(), VirtualTankError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(VirtualTankError::InvalidName);
        }
        let fish = self
            .fish
            .iter_mut()
            .find(|fish| fish.alive && fish.id == id)
            .ok_or(VirtualTankError::FishMissing)?;
        fish.name = name.to_owned();
        Ok(())
    }

    pub fn set_fish_shown(&mut self, id: u16, shown: bool) -> Result<(), VirtualTankError> {
        let fish = self
            .fish
            .iter_mut()
            .find(|fish| fish.alive && fish.id == id)
            .ok_or(VirtualTankError::FishMissing)?;
        fish.shown = shown;
        Ok(())
    }

    pub fn set_all_fish_shown(&mut self, shown: bool) {
        for fish in self.fish.iter_mut().filter(|fish| fish.alive) {
            fish.shown = shown;
        }
    }

    pub fn mark_fish_dead(&mut self, id: u16) -> Result<(), VirtualTankError> {
        self.validate()?;
        let index = self
            .fish
            .iter()
            .position(|fish| fish.alive && fish.id == id)
            .ok_or(VirtualTankError::FishMissing)?;
        let breeder = self.fish[index].kind == VirtualFishKind::Breeder;
        self.fish[index].alive = false;
        if breeder
            && let Some(child) = self
                .fish
                .iter_mut()
                .find(|fish| fish.alive && fish.id == id + 100)
        {
            child.id = id;
        }
        if self.selected_fish == Some(id) && !self.active_fish().any(|fish| fish.id == id) {
            self.selected_fish = None;
        }
        Ok(())
    }

    /// W1 Breeder::GiveBirth creates a purchased-ID guppy from a large mother,
    /// at most once per mother, using the next free slot. Board supplies the
    /// new Fish constructor's hunger/growth values and physical attachment.
    pub fn birth_from_breeder(
        &mut self,
        parent_id: u16,
        now_seconds: u64,
        day_start_seconds: u64,
        initial_hunger: i32,
        food_needed_to_grow: u16,
        coin_threshold: u32,
    ) -> Result<u16, VirtualTankError> {
        self.validate()?;
        if food_needed_to_grow == 0 || coin_threshold == 0 {
            return Err(VirtualTankError::InvalidOffer);
        }
        if parent_id as usize >= self.fish_capacity() {
            return Err(VirtualTankError::FishMissing);
        }
        let parent_index = self
            .fish
            .iter()
            .position(|fish| {
                fish.alive
                    && fish.id == parent_id
                    && fish.kind == VirtualFishKind::Breeder
                    && fish.size == FishSize::Large
                    && !fish.has_given_birth
            })
            .ok_or(VirtualTankError::FishMissing)?;
        let child_id = self.next_fish_id().ok_or(VirtualTankError::FishTankFull)?;
        let parent = &self.fish[parent_index];
        let child = VirtualFish {
            id: child_id,
            name: format!("{} JR.", parent.name),
            kind: VirtualFishKind::Guppy,
            properties: parent.properties,
            shown: true,
            alive: true,
            purchased_at_seconds: now_seconds,
            bought_today: true,
            purchase_price: parent.purchase_price,
            size: FishSize::Small,
            hunger: initial_hunger,
            food_ate: 0,
            food_needed_to_grow,
            growth_ticks: 0,
            coin_ticks: 0,
            coin_threshold,
            breeder_child: true,
            hometown: 0,
            likes: parent.likes,
            day_start_seconds,
            last_mental_day_seconds: day_start_seconds.saturating_sub(1),
            fed_today: 0,
            mental_state: 3,
            has_given_birth: false,
        };
        self.fish[parent_index].has_given_birth = true;
        self.fish.retain(|fish| fish.alive || fish.id != child_id);
        self.fish.push(child);
        Ok(child_id)
    }

    /// Apply the PB05/W1 Fish::OnFoodAte mode-5 branch after Board has
    /// established contact and diet eligibility. Other kinds have distinct
    /// actor handlers and use record_qualifying_feed plus their own state.
    pub fn feed_guppy(
        &mut self,
        id: u16,
        food: GuppyFood,
        now_seconds: u64,
        day_start_seconds: u64,
    ) -> Result<FeedResult, VirtualTankError> {
        let fish = self
            .fish
            .iter_mut()
            .find(|fish| fish.alive && fish.id == id && fish.kind == VirtualFishKind::Guppy)
            .ok_or(VirtualTankError::FishMissing)?;
        if food == GuppyFood::Snot {
            fish.hunger = fish.hunger.min(30);
            return Ok(FeedResult::Fed);
        }
        let counts = fish.record_qualifying_feed(now_seconds, day_start_seconds);
        let before = fish.size;
        match food {
            GuppyFood::Basic => {
                fish.hunger = fish.hunger.saturating_add(500).min(800);
                if counts {
                    fish.food_ate = fish.food_ate.saturating_add(1);
                }
            }
            GuppyFood::Upgraded => {
                fish.hunger = fish.hunger.saturating_add(700).min(1_000);
                if counts {
                    fish.food_ate = fish.food_ate.saturating_add(1);
                }
            }
            GuppyFood::Star => {
                if matches!(fish.size, FishSize::Small | FishSize::Medium) {
                    fish.alive = false;
                    if self.selected_fish == Some(id) {
                        self.selected_fish = None;
                    }
                    return Ok(FeedResult::Died);
                }
                fish.hunger = fish.hunger.saturating_add(1_100).min(1_400);
                if fish.size == FishSize::Large {
                    fish.size = FishSize::Star;
                }
            }
            GuppyFood::Other => {
                fish.hunger = fish.hunger.saturating_add(500).min(800);
                if counts {
                    fish.food_ate = fish.food_ate.saturating_add(2);
                }
            }
            GuppyFood::Snot => unreachable!(),
        }
        if fish.food_ate >= fish.food_needed_to_grow {
            fish.size = match fish.size {
                FishSize::Small => FishSize::Medium,
                FishSize::Medium => FishSize::Large,
                size => size,
            };
            if fish.size != before && matches!(before, FishSize::Small | FishSize::Medium) {
                fish.food_ate = 0;
                fish.growth_ticks = 10;
            }
            if fish.food_ate >= fish.food_needed_to_grow.saturating_mul(8)
                && matches!(fish.size, FishSize::Large | FishSize::Star)
                && fish.properties.colors == FishColors::Normal
            {
                fish.size = FishSize::Crowned;
            }
        }
        if fish.size != before {
            Ok(FeedResult::Grew(fish.size))
        } else {
            Ok(FeedResult::Fed)
        }
    }

    pub fn can_drop_coin(&self, screensaver: bool, screensaver_limit: u32) -> bool {
        self.settings.drop_shells && (!screensaver || self.board_shells <= screensaver_limit)
    }

    /// The caller supplies Board's coin value after a real collection. A
    /// screensaver increments Board's counter without immediate Profile credit.
    pub fn collect_shells(&mut self, shells: &mut u32, value: u32, screensaver: bool) {
        self.board_shells = self.board_shells.saturating_add(value);
        if !screensaver {
            *shells = shells.saturating_add(value).min(MAX_SHELL_BALANCE);
        }
    }

    pub fn collect_coin(
        &mut self,
        shells: &mut u32,
        kind: VirtualCoinKind,
        combo: u8,
        screensaver: bool,
    ) -> u32 {
        let value = kind.shell_value(combo);
        self.collect_shells(shells, value, screensaver);
        value
    }

    pub fn sell_fish(
        &mut self,
        shells: &mut u32,
        id: u16,
        accepted: bool,
        now_seconds: u64,
        sale_grace_enabled: bool,
    ) -> Result<u32, VirtualTankError> {
        if !accepted {
            return Ok(0);
        }
        self.validate()?;
        if id as usize >= self.fish_capacity() {
            return Err(VirtualTankError::FishMissing);
        }
        let index = self
            .fish
            .iter()
            .position(|fish| fish.alive && fish.id == id)
            .ok_or(VirtualTankError::FishMissing)?;
        let refund = self.fish[index]
            .sale_value(now_seconds, sale_grace_enabled)
            .ok_or(VirtualTankError::FishNotSellable)?;
        let breeder = self.fish[index].kind == VirtualFishKind::Breeder;
        self.fish[index].alive = false;
        if breeder
            && let Some(child) = self
                .fish
                .iter_mut()
                .find(|fish| fish.alive && fish.id == id + 100)
        {
            child.id = id;
        }
        if !self.active_fish().any(|fish| fish.id == id) && self.selected_fish == Some(id) {
            self.selected_fish = None;
        }
        *shells = shells.saturating_add(refund).min(MAX_SHELL_BALANCE);
        Ok(refund)
    }

    pub fn switch_backdrop(&mut self, direction: i8) -> Result<u8, VirtualTankError> {
        if direction != -1 && direction != 1 {
            return Err(VirtualTankError::InvalidState);
        }
        for step in 1..=BACKDROP_COUNT {
            let index = (self.backdrop as i32 + direction as i32 * step as i32)
                .rem_euclid(BACKDROP_COUNT as i32) as u8;
            if self.owned_backdrops[index as usize] {
                self.backdrop = index;
                return Ok(index);
            }
        }
        Err(VirtualTankError::BackdropNotOwned)
    }

    pub fn sell_backdrop(
        &mut self,
        shells: &mut u32,
        accepted: bool,
    ) -> Result<u32, VirtualTankError> {
        if !accepted {
            return Ok(0);
        }
        self.validate()?;
        let old = self.backdrop as usize;
        if !self
            .owned_backdrops
            .iter()
            .enumerate()
            .any(|(index, owned)| index != old && *owned)
        {
            return Err(VirtualTankError::LastBackdrop);
        }
        let next = (1..BACKDROP_COUNT)
            .map(|step| (old + step) % BACKDROP_COUNT)
            .find(|index| self.owned_backdrops[*index])
            .ok_or(VirtualTankError::LastBackdrop)?;
        self.owned_backdrops[old] = false;
        self.backdrop = next as u8;
        *shells = shells
            .saturating_add(BACKDROP_SALE_REFUND)
            .min(MAX_SHELL_BALANCE);
        Ok(BACKDROP_SALE_REFUND)
    }

    pub fn set_settings(&mut self, settings: VirtualSettings) -> Result<(), VirtualTankError> {
        if (settings.bubbulator_shown && !self.bubbulator_owned)
            || (settings.alien_attractor_shown && !self.alien_attractor_owned)
        {
            return Err(VirtualTankError::AccessoryRequired);
        }
        self.settings = settings;
        Ok(())
    }

    /// VT selection has its own seven-pet ceiling. AdventureProgress retains
    /// the ordinary four-pet ceiling even after Profile capacity reaches 7.
    pub fn set_selected_pets(
        &mut self,
        selected: &[PetKind],
        unlocked: &[PetKind],
        profile_capacity: u8,
    ) -> Result<(), VirtualTankError> {
        let limit = usize::from(profile_capacity)
            .min(unlocked.len())
            .min(VIRTUAL_PET_LIMIT);
        if selected.len() > limit
            || selected
                .iter()
                .enumerate()
                .any(|(index, pet)| selected[..index].contains(pet))
            || selected.iter().any(|pet| !unlocked.contains(pet))
        {
            return Err(VirtualTankError::PetSelection);
        }
        self.selected_pets = selected.to_vec();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_backdrop_sale_preserves_ownership_and_wallet() {
        let mut tank = VirtualTankState::default();
        let mut shells = 20;
        assert_eq!(
            tank.sell_backdrop(&mut shells, true),
            Err(VirtualTankError::LastBackdrop)
        );
        assert_eq!(shells, 20);
        assert_eq!(tank.backdrop, 0);
        assert_eq!(
            tank.owned_backdrops,
            [true, false, false, false, false, false]
        );
    }

    #[test]
    fn invalid_fish_name_cannot_debit_or_consume_stock() {
        let mut tank = VirtualTankState::default();
        let mut offers: [Option<StoreProduct>; STORE_SLOTS] = std::array::from_fn(|_| None);
        offers[0] = Some(StoreProduct::Fish(FishOffer {
            kind: VirtualFishKind::Guppy,
            properties: FishProperties::default(),
            initial_hunger: 500,
            initial_size: FishSize::Small,
            food_needed_to_grow: 12,
            coin_threshold: 4_320,
        }));
        tank.install_stock(1, offers).unwrap();
        let mut shells = 50;
        let confirmation = PurchaseConfirmation {
            cycle: 1,
            slot: 0,
            expected_price: 25,
            accepted: true,
            fish_name: Some("  ".to_owned()),
            fish_biography: Some(FishBiography {
                hometown: 1,
                likes: [0, 1, 2],
            }),
        };
        assert_eq!(
            tank.confirm_purchase(&mut shells, confirmation, 100, 0),
            Err(VirtualTankError::InvalidName)
        );
        assert_eq!(shells, 50);
        assert!(tank.fish.is_empty());
        assert!(!tank.stock.as_ref().unwrap().bought[0]);
    }
}
