//! First Adventure tank simulation. Functional rules are source-derived from
//! WinFish f919b3c (see docs/behavior-contract.md); retail parity is untested.
//! Positions are logical 640x480 coordinates and advance in 28 ms ticks.

use crate::{
    alien::{AlienFoodView, AlienRuntimeRequest, PreyView, SylvesterKind},
    bilaterus::BilaterusPreyView,
    breeder::{BreederFoodView, BreederSize, BreederState, DeadBreeder},
    clyde::{ClydeCoinView, ClydeState},
    fish_pet::{
        AmpTap, AngieCorpseView, BrinkleyFoodView, FishPetKind, FishPetState, GashFishView,
        NimbusCoinView, NimbusFoodView, PetAlienView, PrestoChangeEligibility, PrestoForm,
        WardFishView, ZorfHungryView,
    },
    gekko::{DeadGekko, GekkoPrey, GekkoPreyKind, GekkoState},
    grubber::{DeadGrubber, GrubberPrey, GrubberState},
    invasion::{BilaterusChild, Invasion1_2, InvasionEvent, WavePlan},
    larva::{LARVA_VALUE, LarvaState, LarvaUpdate},
    missile::{ClassicMissile, MissilePreyView, StanleyTargetView},
    missile::{MissileKind, MissileShot},
    niko::{NikoEvent, NikoPearl, NikoState, PEARL_VALUE, PearlPhase, PearlUpdate},
    oscar::{DeadOscar, OscarPrey, OscarState},
    rhubarb::{RhubarbPrey, RhubarbState},
    rufus::{RufusAlienView, RufusState},
    starcatcher::{DeadStarcatcher, StarcatcherCoinView, StarcatcherState},
    ultra::{DeadUltra, UltraPrey, UltraState},
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[cfg(test)]
use crate::invasion::EncounterKind;

pub const TICK_MS: u32 = 28;
pub const BOARD_WIDTH: f32 = 640.0;
pub const BOARD_HEIGHT: f32 = 480.0;
pub const FOOD_PRICE: i32 = 5;
pub const GUPPY_PRICE: i32 = 100;
pub const EGG_PRICE: i32 = 150;
pub const SECOND_STAGE_EGG_PRICE: i32 = 500;
pub const THIRD_STAGE_EGG_PRICE: i32 = 2000;
pub const FOURTH_STAGE_EGG_PRICE: i32 = 3000;
pub const FIFTH_STAGE_EGG_PRICE: i32 = 5000;
pub const TANK2_FIRST_EGG_PRICE: i32 = 750;
pub const TANK2_SECOND_EGG_PRICE: i32 = 3000;
pub const TANK2_THIRD_EGG_PRICE: i32 = 5000;
pub const TANK2_FOURTH_EGG_PRICE: i32 = 7500;
pub const TANK2_FIFTH_EGG_PRICE: i32 = 10000;
pub const STARCATCHER_PRICE: i32 = 750;
pub const GRUBBER_PRICE: i32 = 750;
pub const GEKKO_PRICE: i32 = 2000;
pub const BREEDER_PRICE: i32 = 200;
pub const TANK3_FIRST_EGG_PRICE: i32 = 1000;
pub const TANK3_SECOND_EGG_PRICE: i32 = 5000;
pub const TANK3_THIRD_EGG_PRICE: i32 = 7500;
pub const TANK3_FOURTH_EGG_PRICE: i32 = 10_000;
pub const TANK3_FIFTH_EGG_PRICE: i32 = 15_000;
pub const TANK4_FIRST_EGG_PRICE: i32 = 3000;
pub const TANK4_SECOND_EGG_PRICE: i32 = 25_000;
pub const TANK4_THIRD_EGG_PRICE: i32 = 50_000;
pub const TANK4_FOURTH_EGG_PRICE: i32 = 75_000;
pub const TANK4_FINALE_EGG_PRICE: i32 = 99_999;
pub const ULTRA_PRICE: i32 = 10_000;
pub const TANK4_WEAPON_PRICE: i32 = 5000;
pub const POTION_PRICE: i32 = 250;
pub const FOOD_QUALITY_PRICE: i32 = 200;
pub const FOOD_QUANTITY_PRICE: i32 = 300;
pub const OSCAR_PRICE: i32 = 1000;
pub const WEAPON_PRICE: i32 = 1000;
const FIRST_STAGE_COIN_BOTTOM_TICKS: u16 = 150;
const SECOND_STAGE_COIN_BOTTOM_TICKS: u16 = 20;

/// PB66 signed truncation: max(20, (n - 20) / 2 + 20), capped by victims.
fn amp_reward_limit(victims: usize) -> usize {
    if victims <= 20 {
        victims
    } else {
        20 + (victims - 20) / 2
    }
}

const fn first_stage_egg_price() -> i32 {
    EGG_PRICE
}

const fn initial_weapon_strength() -> u8 {
    2
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PetKind {
    Stinky,
    Niko,
    Itchy,
    Prego,
    Zorf,
    Clyde,
    Vert,
    Rufus,
    Meryl,
    Wadsworth,
    Seymour,
    Shrapnel,
    Gumbo,
    Blip,
    Rhubarb,
    Nimbus,
    Amp,
    Gash,
    Angie,
    Presto,
    Brinkley,
    Nostradamus,
    Stanley,
    Walter,
}

/// Fresh ordinary Tank5 entry clears retail selection flags before this
/// fixed registration order. Profile selection is a separate authority.
pub(crate) const TANK5_PETS: [PetKind; 18] = [
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
];

/// A transient view of the single physical actor backing logical Presto.
/// The actor collections remain the only durable current-form authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PrestoActorView {
    pub id: u64,
    pub kind: PetKind,
    pub widget_x: i32,
    pub widget_y: i32,
    pub remaining_ticks: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StinkyOrigin {
    StageStart,
    LegacyV2Resume,
}

/// Live first Adventure pet. The source stores motion and animation on the
/// pet object, separately from the profile's unlocked-pet roster.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StinkyState {
    /// Project identity for Tank5 targeting; ordinary Stinky is not prey.
    pub combat_id: Option<u64>,
    pub presto_form: Option<PrestoForm>,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub target_vx: f64,
    pub previous_vx: f64,
    pub frame: u8,
    pub movement_state: u8,
    pub movement_state_change_timer: u16,
    pub chase_timer: u16,
    pub movement_animation_timer: u8,
    pub turn_animation_timer: i8,
    pub specialty_timer: u8,
    pub angry_timer: u16,
    pub random_timer: u16,
    pub origin: StinkyOrigin,
}

impl StinkyState {
    /// Validate a saved first-tank pet without rejecting the small position
    /// overshoot possible after the source clamps before integrating velocity.
    pub fn validate(&self) -> Result<(), String> {
        if !self.x.is_finite()
            || !self.y.is_finite()
            || !self.vx.is_finite()
            || !self.vy.is_finite()
            || !self.target_vx.is_finite()
            || !self.previous_vx.is_finite()
            || !(0.0..=560.0).contains(&self.x)
            || !(0.0..=480.0).contains(&self.y)
            || self.vx.abs() > 10.0
            || self.vy.abs() > 10.0
            || self.target_vx.abs() > 10.0
            || self.previous_vx.abs() > 10.0
        {
            return Err("invalid Stinky motion".into());
        }
        if self.frame >= 10
            || self
                .presto_form
                .is_some_and(|form| form.remaining_ticks > 360)
            || !(-20..=20).contains(&self.turn_animation_timer)
            || self.specialty_timer > 9
            || self.movement_state > 9
            || self.movement_animation_timer >= 40
            || self.movement_state_change_timer > 20
            || !(250..500).contains(&self.random_timer)
        {
            return Err("invalid Stinky animation or movement counter".into());
        }
        Ok(())
    }

    pub fn sprite_row(&self) -> u8 {
        if self.specialty_timer > 0 {
            2
        } else if self.turn_animation_timer != 0 {
            1
        } else {
            0
        }
    }

    pub fn facing_right(&self) -> bool {
        if self.specialty_timer > 0 {
            self.previous_vx >= 0.0
        } else if self.turn_animation_timer != 0 {
            self.turn_animation_timer > 0
        } else if self.vx.abs() >= 1.0 {
            self.vx >= 0.0
        } else {
            self.previous_vx >= 0.0
        }
    }
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
pub enum FishPose {
    Swim,
    Eat,
    Turn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoinKind {
    Silver,
    Gold,
    Diamond,
    Star,
    DiamondPenta,
    Pearl,
    ShrapnelBomb,
    Treasure,
    ShellSilver,
    ShellGold,
    ShellStar,
    ShellDiamond,
    ShellDiamondPenta,
    ShellPearl,
    ShellTreasure,
}

impl CoinKind {
    pub fn value(self) -> i32 {
        match self {
            Self::Silver | Self::ShellSilver => 15,
            Self::Gold | Self::ShellGold => 35,
            Self::Diamond | Self::DiamondPenta | Self::ShellDiamond | Self::ShellDiamondPenta => {
                200
            }
            Self::Star | Self::ShellStar => 40,
            Self::Pearl | Self::ShellPearl => 500,
            Self::ShrapnelBomb => 150,
            Self::Treasure | Self::ShellTreasure => 2000,
        }
    }

    pub fn is_shell(self) -> bool {
        matches!(
            self,
            Self::ShellSilver
                | Self::ShellGold
                | Self::ShellStar
                | Self::ShellDiamond
                | Self::ShellDiamondPenta
                | Self::ShellPearl
                | Self::ShellTreasure
        )
    }

    fn nimbus_kind(self) -> Self {
        match self {
            Self::Silver => Self::ShellSilver,
            Self::Gold => Self::ShellGold,
            Self::Star => Self::ShellStar,
            Self::Diamond => Self::ShellDiamond,
            Self::DiamondPenta => Self::ShellDiamondPenta,
            Self::Pearl => Self::ShellPearl,
            Self::Treasure => Self::ShellTreasure,
            kind => kind,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fish {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub facing_right: bool,
    pub frame: u8,
    pub turn_ticks: i8,
    pub hunger_visible: bool,
    pub size: FishSize,
    pub hunger: i32,
    pub food_ate: u8,
    pub food_needed_to_grow: u8,
    pub beginner: bool,
    pub eating_ticks: u8,
    pub growth_ticks: u8,
    pub coin_timer: u16,
    pub coin_threshold: u16,
    pub walter: crate::walter::WalterImpact,
    pub alive: bool,
    #[serde(default)]
    pub cannot_be_eaten_ticks: u8,
    speed_mod: f32,
    movement_state: u8,
    movement_timer: u8,
    special_timer: u8,
    x_direction: i8,
    previous_vx: f32,
    swim_counter: u8,
    bought_timer: u8,
}

impl Fish {
    pub fn sprite_pose(&self) -> FishPose {
        if self.turn_ticks != 0 {
            FishPose::Turn
        } else if self.eating_ticks > 0 {
            FishPose::Eat
        } else {
            FishPose::Swim
        }
    }

    /// Fish.cpp::DrawFish uses a centered scale during the ten update growth pulse.
    pub fn growth_scale(&self) -> f32 {
        match self.growth_ticks {
            0 => 1.0,
            4..=10 => 0.5 + (10 - self.growth_ticks) as f32 * 0.1,
            ticks => 1.0 + ticks as f32 / 15.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeadFish {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub size: FishSize,
    pub facing_right: bool,
    pub frame: u8,
    pub opacity: f32,
    pub remaining_ticks: u16,
    pub revival_ticks: i32,
    vx: f32,
    vy: f32,
    speed_mod: f32,
}

impl DeadFish {
    fn from_live(fish: &Fish) -> Self {
        Self {
            id: fish.id,
            x: fish.x,
            y: fish.y,
            size: fish.size,
            facing_right: fish.vx >= 0.0,
            frame: 0,
            opacity: 1.0,
            remaining_ticks: 125,
            revival_ticks: 100,
            vx: fish.vx,
            vy: fish.vy
                - if fish.x < 115.0 || fish.vy < -3.0 {
                    1.0
                } else {
                    2.0
                },
            speed_mod: fish.speed_mod,
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.id == 0
            || ![
                self.x,
                self.y,
                self.vx,
                self.vy,
                self.speed_mod,
                self.opacity,
            ]
            .into_iter()
            .all(f32::is_finite)
            || self.remaining_ticks > 125
            || self.frame > 9
            || !(0.0..=1.0).contains(&self.opacity)
            || ![1.6, 1.8, 2.0].contains(&self.speed_mod)
            || !(self.revival_ticks == 100 || (0..=10).contains(&self.revival_ticks))
        {
            return Err("invalid ordinary guppy corpse state".into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
enum RevivalKind {
    Guppy(FishSize),
    Oscar,
    Ultra,
    Starcatcher,
    Grubber,
    Gekko,
    Breeder(BreederSize),
}

#[derive(Clone, Copy, Debug)]
struct CorpseRevival {
    corpse_id: u64,
    kind: RevivalKind,
    x: i32,
    y: i32,
    facing_right: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TutorialCue {
    Hungry,
    VeryHungry,
    Starving,
    BuyFish,
    BuyEgg,
    CollectCoin,
    EggsRemaining,
    BuyFoodQuality,
    BuyFoodQuantity,
    HoldToFeed,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TutorialState {
    pub buy_fish_hint: bool,
    pub buy_egg_hint: bool,
    pub coin_hint: bool,
    hunger_shown: [bool; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Food {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub frame: u8,
    /// Food +0x194 is distinct from the quality byte at +0x188.
    pub food_type: FoodType,
    /// Food +0x185 latches a collectible pickup before flight and credit.
    pub picked_up: bool,
    pub ineligible_ticks: u8,
    pub removal_ticks: u8,
    #[serde(default)]
    pub quality: u8,
    #[serde(default)]
    pub direction: u8,
    #[serde(default)]
    pub vx: f32,
    #[serde(default)]
    pub vy: f32,
    #[serde(default = "initial_food_animation_period")]
    pub animation_period: u8,
    #[serde(default)]
    pub free_from_zorf: bool,
    #[serde(default)]
    pub nimbus_rising: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FoodType {
    Ordinary,
    Nostradamus,
}

const fn initial_food_animation_period() -> u8 {
    3
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Coin {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub kind: CoinKind,
    pub frame: u8,
    pub animation_ticks: u8,
    pub hazard_age_ticks: i32,
    pub collecting: bool,
    pub bottom_ticks: u16,
    pub fade_ticks: u8,
    pub penta_rising: bool,
}

/// Board-owned finite Shot types 2..5. Type 2 has no Draw image; types 3..5
/// are bomb contact fragments. These are visual effects,
/// not collectible GameObjects and therefore do not consume Board entity IDs.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BombShot {
    pub shot_type: u8,
    pub x: i32,
    pub y: i32,
    pub age_ticks: u8,
    pub frame: i8,
    pub delay_ticks: u8,
    pub alpha: u8,
}

impl BombShot {
    pub fn sprite_frame(&self) -> Option<u8> {
        (self.shot_type != 2 && self.delay_ticks == 0 && self.frame >= 0)
            .then_some(self.frame.max(0) as u8)
    }

    fn tick(&mut self) -> bool {
        if self.delay_ticks > 0 {
            self.delay_ticks -= 1;
            return false;
        }
        self.age_ticks += if self.shot_type != 2 && self.alpha > 150 {
            2
        } else {
            1
        };
        self.frame = (self.age_ticks / 2) as i8 - 1;
        self.age_ticks > if self.shot_type == 2 { 29 } else { 19 }
    }

    fn validate(&self) -> Result<(), String> {
        if !(2..=5).contains(&self.shot_type)
            || self.age_ticks > (if self.shot_type == 2 { 29 } else { 19 })
            || !(if self.shot_type == 2 { -1..=14 } else { -1..=9 }).contains(&self.frame)
            || self.delay_ticks > 2
            || !(50..=249).contains(&self.alpha)
        {
            return Err("invalid Shrapnel contact shot".into());
        }
        Ok(())
    }
}

/// A zero-value Meryl note. It shares neither collection flight nor coin
/// affordability; its visual lifetime is independent of the emitter.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NoteState {
    pub id: u64,
    pub x: i32,
    pub y: i32,
    pub age_ticks: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Action {
    Click { x: f32, y: f32 },
    RightClick { x: f32, y: f32 },
    PrestoPress { pet: PetKind },
    PrestoRelease { pet: Option<PetKind> },
    PrestoCancel,
    BuyGuppy,
    BuyEgg,
    BuyFoodQuality,
    BuyFoodQuantity,
    BuyOscar,
    BuyWeapon,
    BuyPotion,
    BuyStarcatcher,
    BuyGrubber,
    BuyGekko,
    BuyBreeder,
    BuyUltra,
    HoldFeed { x: f32, y: f32, elapsed_ms: u32 },
    HoldFire { x: f32, y: f32, elapsed_ms: u32 },
    TogglePet { pet: PetKind },
    ConfirmPetSelection { accept: bool },
    OfferBonusPurchase,
    ConfirmBonusPurchase { accept: bool },
    OpenMenu,
    PlayAdventure,
    PlayTimeTrial,
    SelectTimeTrialTank { tank: u8 },
    Continue,
    HatchHold { down: bool },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rejection {
    OutsideTank,
    FoodCapacity,
    InsufficientFunds,
    Locked,
    Completed,
    UnsupportedStage,
    MaximumUpgrade,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Event {
    BlipShopRevealed {
        tick: u64,
    },
    BlipSonar {
        tick: u64,
    },
    Invasion {
        tick: u64,
        event: InvasionEvent,
    },
    Niko {
        tick: u64,
        event: NikoEvent,
    },
    PearlCollectionStarted {
        tick: u64,
        pearl_id: u64,
        owner_id: u64,
    },
    PearlExpired {
        tick: u64,
        pearl_id: u64,
    },
    PearlCredited {
        tick: u64,
        pearl_id: u64,
        owner_id: u64,
        amount: i32,
        balance: i32,
    },
    GameOverStarted {
        tick: u64,
    },
    GameSelectorOpened {
        tick: u64,
    },
    TimeTrialStarted {
        tick: u64,
        tank: u8,
    },
    TimeTrialPetAcquired {
        tick: u64,
        pet: PetKind,
        price: i32,
    },
    PrestoChanged {
        tick: u64,
        old_id: u64,
        new_id: u64,
        from: PetKind,
        to: PetKind,
    },
    WalterPunched {
        tick: u64,
        pet_id: u64,
    },
    TimeTrialExpired {
        tick: u64,
        tank: u8,
        score: i32,
    },
    TimeTrialShellsCredited {
        tick: u64,
        amount: u32,
        shell_balance: u32,
    },
    Action {
        tick: u64,
        action: Action,
    },
    Rejected {
        tick: u64,
        reason: Rejection,
    },
    BonusPurchaseOffered {
        tick: u64,
        pet: PetKind,
        price: u32,
    },
    BonusPurchaseCancelled {
        tick: u64,
    },
    BonusPurchaseCommitted {
        tick: u64,
        pet: PetKind,
        price: u32,
        shell_balance: u32,
    },
    BrinkleyFoodEaten {
        tick: u64,
        pet_id: u64,
        food_id: u64,
        meals: u32,
    },
    BrinkleyCoinDropped {
        tick: u64,
        pet_id: u64,
        coin_id: u64,
        kind: CoinKind,
    },
    FoodDropped {
        tick: u64,
        food_id: u64,
        balance: i32,
        #[serde(default)]
        potion: bool,
    },
    FoodEaten {
        tick: u64,
        fish_id: u64,
        food_id: u64,
        hunger: i32,
    },
    FoodExpired {
        tick: u64,
        food_id: u64,
    },
    PotionExploded {
        tick: u64,
        food_id: u64,
        fish_id: Option<u64>,
    },
    PotionBought {
        tick: u64,
        balance: i32,
    },
    ZorfFoodDropped {
        tick: u64,
        pet_id: u64,
        food_id: u64,
    },
    NostradamusFoodDropped {
        tick: u64,
        pet_id: u64,
        food_id: u64,
    },
    NostradamusFoodCollectionStarted {
        tick: u64,
        food_id: u64,
    },
    NostradamusFoodCredited {
        tick: u64,
        food_id: u64,
        balance: i32,
    },
    NostradamusFishConverted {
        tick: u64,
        fish_id: u64,
        food_id: u64,
        pet_id: u64,
        replacement_id: Option<u64>,
    },
    Bonus {
        tick: u64,
        event: crate::bonus::BonusEvent,
    },
    BonusResultsCommitted {
        tick: u64,
        earned: u32,
        previous_balance: u32,
        shell_balance: u32,
    },
    FishGrew {
        tick: u64,
        fish_id: u64,
        size: FishSize,
    },
    FishDied {
        tick: u64,
        fish_id: u64,
    },
    PetRemoved {
        tick: u64,
        pet_id: u64,
    },
    Tutorial {
        tick: u64,
        cue: TutorialCue,
    },
    GuppyBought {
        tick: u64,
        fish_id: u64,
        balance: i32,
    },
    FoodQualityBought {
        tick: u64,
        quality: u8,
        balance: i32,
    },
    FoodQuantityBought {
        tick: u64,
        quantity: u8,
        balance: i32,
    },
    OscarBought {
        tick: u64,
        oscar_id: u64,
        balance: i32,
    },
    UltraBought {
        tick: u64,
        id: u64,
        balance: i32,
    },
    UltraAteOscar {
        tick: u64,
        id: u64,
        oscar_id: u64,
        hunger: i32,
    },
    UltraDied {
        tick: u64,
        id: u64,
    },
    UltraTreasureDropped {
        tick: u64,
        ultra_id: u64,
        coin_id: u64,
    },
    NimbusCoinConverted {
        tick: u64,
        pet_id: u64,
        old_id: u64,
        coin_id: u64,
        kind: CoinKind,
    },
    NimbusFoodConverted {
        tick: u64,
        pet_id: u64,
        old_id: u64,
        food_id: u64,
    },
    StarcatcherBought {
        tick: u64,
        starcatcher_id: u64,
        balance: i32,
    },
    StarcatcherAteStar {
        tick: u64,
        starcatcher_id: u64,
        star_coin_id: u64,
        diamond_coin_id: u64,
    },
    StarcatcherDied {
        tick: u64,
        starcatcher_id: u64,
    },
    GrubberBought {
        tick: u64,
        grubber_id: u64,
        balance: i32,
    },
    GrubberAteGuppy {
        tick: u64,
        grubber_id: u64,
        guppy_id: u64,
    },
    GrubberDied {
        tick: u64,
        grubber_id: u64,
    },
    GrubberBubble {
        tick: u64,
        grubber_id: u64,
        x: i32,
        y: i32,
    },
    GekkoBought {
        tick: u64,
        gekko_id: u64,
        balance: i32,
    },
    GekkoAtePrey {
        tick: u64,
        gekko_id: u64,
        prey_id: u64,
        kind: GekkoPreyKind,
    },
    GekkoDied {
        tick: u64,
        gekko_id: u64,
    },
    GekkoPearlDropped {
        tick: u64,
        gekko_id: u64,
        coin_id: u64,
    },
    GekkoBubble {
        tick: u64,
        gekko_id: u64,
        x: i32,
        y: i32,
    },
    BreederBought {
        tick: u64,
        breeder_id: u64,
        balance: i32,
    },
    BreederAteFood {
        tick: u64,
        breeder_id: u64,
        food_id: u64,
        hunger: i32,
    },
    BreederGrew {
        tick: u64,
        breeder_id: u64,
        size: BreederSize,
    },
    BreederBornGuppy {
        tick: u64,
        breeder_id: u64,
        fish_id: u64,
        x: i32,
        y: i32,
    },
    BreederDied {
        tick: u64,
        breeder_id: u64,
        sound: bool,
    },
    RhubarbSpecialtyStarted {
        tick: u64,
        pet_id: u64,
    },
    RhubarbPushed {
        tick: u64,
        pet_id: u64,
        target_id: u64,
        new_vy: i32,
    },
    ShrapnelBombDropped {
        tick: u64,
        pet_id: u64,
        coin_id: u64,
    },
    ShrapnelBombExploded {
        tick: u64,
        coin_id: u64,
        target_id: Option<u64>,
        fragments: u8,
    },
    LarvaDropped {
        tick: u64,
        grubber_id: u64,
        larva_id: u64,
    },
    LarvaCollectionStarted {
        tick: u64,
        larva_id: u64,
    },
    LarvaPickupSound {
        tick: u64,
        larva_id: u64,
    },
    LarvaCredited {
        tick: u64,
        larva_id: u64,
        amount: i32,
        balance: i32,
    },
    LarvaExpired {
        tick: u64,
        larva_id: u64,
    },
    WadsworthProtectionChanged {
        tick: u64,
        pet_id: u64,
        active: bool,
    },
    WadsworthBubbles {
        tick: u64,
        pet_id: u64,
        positions: [(i32, i32); 2],
    },
    WeaponBought {
        tick: u64,
        strength: u8,
        balance: i32,
    },
    OscarAteGuppy {
        tick: u64,
        oscar_id: u64,
        guppy_id: u64,
    },
    OscarDied {
        tick: u64,
        oscar_id: u64,
    },
    FishPetHit {
        tick: u64,
        pet_id: u64,
        alien_id: u64,
        health: f64,
        sound: bool,
    },
    GashAteGuppy {
        tick: u64,
        pet_id: u64,
        guppy_id: u64,
    },
    AmpReady {
        tick: u64,
        pet_id: u64,
    },
    AmpCharged {
        tick: u64,
        pet_id: u64,
        charge: u8,
    },
    AmpDischarged {
        tick: u64,
        pet_id: u64,
        victim_ids: Vec<u64>,
        coin_ids: Vec<u64>,
        effects: usize,
        replacement_id: Option<u64>,
    },
    GusInitialFeedAttempt {
        tick: u64,
        x: i32,
        y: i32,
    },
    VertGoldDropped {
        tick: u64,
        pet_id: u64,
        coin_id: u64,
    },
    MerylNoteDropped {
        tick: u64,
        pet_id: u64,
        note_id: u64,
    },
    MerylNoteExpired {
        tick: u64,
        note_id: u64,
    },
    MissileLaunched {
        tick: u64,
        missile_id: u64,
        target_id: u64,
    },
    EnergyBallLaunched {
        tick: u64,
        missile_id: u64,
        target_id: u64,
        first: bool,
    },
    EnergyBallShot {
        tick: u64,
        missile_id: u64,
        redirected: bool,
    },
    EnergyBallAlienHit {
        tick: u64,
        missile_id: u64,
        alien_id: u64,
        health: f64,
    },
    EnergyBallRemoved {
        tick: u64,
        missile_id: u64,
    },
    MissileRemoved {
        tick: u64,
        missile_id: u64,
    },
    MissileImpacted {
        tick: u64,
        missile_id: u64,
        target_id: u64,
    },
    StanleyMissileLaunched {
        tick: u64,
        missile_id: u64,
        target_id: u64,
        pet_id: u64,
    },
    StanleyMissileDiverted {
        tick: u64,
        missile_id: u64,
        pet_id: u64,
    },
    StanleyMissileImpacted {
        tick: u64,
        missile_id: u64,
        target_id: u64,
    },
    StanleyMissileRemoved {
        tick: u64,
        missile_id: u64,
    },
    RufusHit {
        tick: u64,
        pet_id: u64,
        alien_id: u64,
        health: f64,
        sound: bool,
    },
    PregoBirth {
        tick: u64,
        pet_id: u64,
        fish_id: u64,
        x: i32,
        y: i32,
    },
    PetSelectionOpened {
        tick: u64,
        capacity: u8,
    },
    PetSelectionChanged {
        tick: u64,
        selected: Vec<PetKind>,
    },
    PetSelectionConfirmation {
        tick: u64,
        selected: Vec<PetKind>,
    },
    PetSelectionAccepted {
        tick: u64,
        selected: Vec<PetKind>,
    },
    CoinDropped {
        tick: u64,
        coin_id: u64,
        fish_id: u64,
        kind: CoinKind,
    },
    AlienDiamondDropped {
        tick: u64,
        alien_id: u64,
        coin_id: u64,
    },
    CoinCollectionStarted {
        tick: u64,
        coin_id: u64,
        kind: CoinKind,
    },
    CoinExpired {
        tick: u64,
        coin_id: u64,
    },
    CoinCredited {
        tick: u64,
        coin_id: u64,
        kind: CoinKind,
        amount: i32,
        balance: i32,
    },
    PetCollectedCoin {
        tick: u64,
        pet: PetKind,
        coin_id: u64,
        kind: CoinKind,
        amount: i32,
        balance: i32,
    },
    EggBought {
        tick: u64,
        pieces: u8,
        balance: i32,
    },
    LevelCompleted {
        tick: u64,
        next_tank: u8,
        next_level: u8,
    },
    StageResultRecorded {
        tick: u64,
        tank: u8,
        level: u8,
        seconds: u64,
        settled_coin_ids: Vec<u64>,
        settled_amount: i32,
        final_balance: i32,
        personal_best_seconds: u64,
    },
    FirstTankRescueStarted {
        tick: u64,
    },
    RescueGuppyGranted {
        tick: u64,
        fish_id: u64,
    },
    PetUnlocked {
        tick: u64,
        pet: PetKind,
    },
    HatchStarted {
        tick: u64,
        pet: PetKind,
    },
    HatchOpened {
        tick: u64,
        pet: PetKind,
    },
    HatchReady {
        tick: u64,
        pet: PetKind,
    },
    TankFourFinaleHatchStarted {
        tick: u64,
    },
    AdventureCompleted {
        tick: u64,
        shells_awarded: u32,
        shell_balance: u32,
    },
    TankFourFinaleHatchOpened {
        tick: u64,
    },
    TankFourFinaleHatchReady {
        tick: u64,
    },
    AngieRevivalStarted {
        tick: u64,
        pet_id: u64,
        corpse_id: u64,
    },
    FishRevived {
        tick: u64,
        corpse_id: u64,
        fish_id: u64,
    },
    StageStarted {
        tick: u64,
        tank: u8,
        level: u8,
    },
}

/// Adventure board. The PRNG is controlled for repeatable Rust runs,
/// but its sequence is deliberately not claimed to match the original game.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdventureState {
    pub tick: u64,
    pub tank: u8,
    pub level: u8,
    pub balance: i32,
    pub eggs: u8,
    pub victory: bool,
    pub guppy_unlocked: bool,
    pub egg_unlocked: bool,
    #[serde(default)]
    pub oscar_unlocked: bool,
    pub starcatcher_unlocked: bool,
    pub grubber_unlocked: bool,
    pub gekko_unlocked: bool,
    pub breeder_unlocked: bool,
    #[serde(default)]
    pub ultra_unlocked: bool,
    #[serde(default)]
    pub weapon_unlocked: bool,
    #[serde(default = "initial_weapon_strength")]
    pub weapon_strength: u8,
    #[serde(default)]
    pub potion_unlocked: bool,
    #[serde(default)]
    pub potion_armed: bool,
    #[serde(default = "first_stage_egg_price")]
    pub egg_price: i32,
    #[serde(default)]
    pub time_trial: bool,
    /// Fresh Adventure Board populated from committed profile flags. This is
    /// distinct from the persistent completed-game/profile cursor.
    pub profile_population: bool,
    /// The won Board keeps cash during replay bonus; shell income is separate.
    pub bonus_active: bool,
    pub bonus_tally: u32,
    #[serde(default)]
    pub pets: Vec<PetKind>,
    #[serde(default)]
    pub stinky: Vec<StinkyState>,
    pub clyde: Vec<ClydeState>,
    pub rufus: Vec<RufusState>,
    pub rhubarb: Vec<RhubarbState>,
    pub missiles: Vec<ClassicMissile>,
    pub fish: Vec<Fish>,
    #[serde(default)]
    pub oscars: Vec<OscarState>,
    pub starcatchers: Vec<StarcatcherState>,
    pub dead_starcatchers: Vec<DeadStarcatcher>,
    pub grubbers: Vec<GrubberState>,
    pub dead_grubbers: Vec<DeadGrubber>,
    pub gekkos: Vec<GekkoState>,
    pub dead_gekkos: Vec<DeadGekko>,
    pub breeders: Vec<BreederState>,
    pub dead_breeders: Vec<DeadBreeder>,
    #[serde(default)]
    pub ultras: Vec<UltraState>,
    #[serde(default)]
    pub dead_ultras: Vec<DeadUltra>,
    pub larvae: Vec<LarvaState>,
    #[serde(default)]
    pub fish_pets: Vec<FishPetState>,
    #[serde(default)]
    pub punch_sound_cooldown: u8,
    #[serde(default)]
    pub dead_oscars: Vec<DeadOscar>,
    pub dead_fish: Vec<DeadFish>,
    pub food: Vec<Food>,
    pub coins: Vec<Coin>,
    pub bomb_shots: Vec<BombShot>,
    pub notes: Vec<NoteState>,
    pub tutorial: TutorialState,
    #[serde(default)]
    pub upgrades: FoodUpgrades,
    #[serde(default)]
    pub invasion: Option<Invasion1_2>,
    #[serde(default)]
    pub niko: Vec<NikoState>,
    #[serde(default)]
    pub pearls: Vec<NikoPearl>,
    next_id: u64,
    rng_state: u64,
    #[serde(skip)]
    held_feed: Option<(f32, f32, u32)>,
    #[serde(skip)]
    held_fire: Option<(f32, f32, u32)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FoodUpgrades {
    pub quality: u8,
    pub quantity: u8,
    pub quality_unlocked: bool,
    pub quantity_unlocked: bool,
}

impl Default for FoodUpgrades {
    fn default() -> Self {
        Self {
            quality: 0,
            quantity: 1,
            quality_unlocked: false,
            quantity_unlocked: false,
        }
    }
}

impl AdventureState {
    pub fn new_adventure(seed: u64) -> Self {
        let mut state = Self::empty_board(seed);
        state.spawn_starter_guppies(true);
        state
    }

    fn empty_board(seed: u64) -> Self {
        Self {
            tick: 0,
            tank: 1,
            level: 1,
            balance: 200,
            eggs: 0,
            victory: false,
            guppy_unlocked: false,
            egg_unlocked: false,
            oscar_unlocked: false,
            starcatcher_unlocked: false,
            grubber_unlocked: false,
            gekko_unlocked: false,
            breeder_unlocked: false,
            ultra_unlocked: false,
            weapon_unlocked: false,
            weapon_strength: 2,
            potion_unlocked: false,
            potion_armed: false,
            egg_price: EGG_PRICE,
            time_trial: false,
            profile_population: false,
            bonus_active: false,
            bonus_tally: 0,
            pets: Vec::new(),
            stinky: Vec::new(),
            clyde: Vec::new(),
            rufus: Vec::new(),
            rhubarb: Vec::new(),
            missiles: Vec::new(),
            fish: Vec::new(),
            oscars: Vec::new(),
            starcatchers: Vec::new(),
            dead_starcatchers: Vec::new(),
            grubbers: Vec::new(),
            dead_grubbers: Vec::new(),
            gekkos: Vec::new(),
            dead_gekkos: Vec::new(),
            breeders: Vec::new(),
            dead_breeders: Vec::new(),
            ultras: Vec::new(),
            dead_ultras: Vec::new(),
            larvae: Vec::new(),
            fish_pets: Vec::new(),
            punch_sound_cooldown: 0,
            dead_oscars: Vec::new(),
            dead_fish: Vec::new(),
            food: Vec::new(),
            coins: Vec::new(),
            bomb_shots: Vec::new(),
            notes: Vec::new(),
            tutorial: TutorialState::default(),
            upgrades: FoodUpgrades::default(),
            invasion: None,
            niko: Vec::new(),
            pearls: Vec::new(),
            next_id: 1,
            rng_state: Self::initial_rng(seed),
            held_feed: None,
            held_fire: None,
        }
    }

    fn spawn_starter_guppies(&mut self, beginner: bool) {
        for _ in 0..2 {
            let x = self.rand_range(520) as f32 + 20.0;
            let y = self.rand_range(265) as f32 + 105.0;
            let mut fish = self.make_fish(x, y, beginner, false);
            fish.food_ate = 2;
            self.fish.push(fish);
        }
    }

    /// Stage 1-2 starts afresh from the progressed profile. The unlocked pet
    /// spawns before the two guppies, as in Board::StartGame.
    pub fn new_second_stage(seed: u64) -> Self {
        let mut state = Self::empty_board(seed);
        state.level = 2;
        state.egg_price = SECOND_STAGE_EGG_PRICE;
        state.invasion = Some(Invasion1_2::new());
        state.pets.push(PetKind::Stinky);
        state.stinky = vec![state.spawn_stinky(StinkyOrigin::StageStart)];
        state.spawn_starter_guppies(false);
        state
    }

    pub fn new_third_stage(seed: u64) -> Self {
        let mut state = Self::empty_board(seed);
        state.level = 3;
        state.egg_price = THIRD_STAGE_EGG_PRICE;
        state.invasion = Some(Invasion1_2::new_strong());
        state.pets = vec![PetKind::Stinky, PetKind::Niko];
        state.stinky = vec![state.spawn_stinky(StinkyOrigin::StageStart)];
        let owner_id = state.id();
        state.niko = vec![NikoState::spawn_tank1(owner_id, &mut |upper| {
            state.rand_range(upper)
        })];
        state.spawn_starter_guppies(false);
        state
    }

    /// Source-backed roster and economy for the ordinary 1-4 Balrog stage.
    pub fn new_fourth_stage(seed: u64) -> Self {
        let mut state = Self::empty_board(seed);
        state.level = 4;
        state.egg_price = FOURTH_STAGE_EGG_PRICE;
        state.invasion = Some(Invasion1_2::new_balrog());
        state.pets = vec![PetKind::Stinky, PetKind::Niko, PetKind::Itchy];
        state.stinky = vec![state.spawn_stinky(StinkyOrigin::StageStart)];
        let owner_id = state.id();
        state.niko = vec![NikoState::spawn_tank1(owner_id, &mut |upper| {
            state.rand_range(upper)
        })];
        state.spawn_fish_pet(FishPetKind::Itchy);
        state.spawn_starter_guppies(false);
        state
    }

    /// First-playthrough pet selection admits zero to three of the four
    /// unlocked pets. The caller controls selection; construction preserves
    /// profile order before the two starter guppies and rejects bad rosters.
    pub fn new_fifth_stage(seed: u64, pets: &[PetKind]) -> Result<Self, String> {
        let canonical = [
            PetKind::Stinky,
            PetKind::Niko,
            PetKind::Itchy,
            PetKind::Prego,
        ];
        if pets.len() > 3
            || pets.iter().any(|pet| !canonical.contains(pet))
            || pets.windows(2).any(|pair| {
                canonical.iter().position(|pet| *pet == pair[0])
                    >= canonical.iter().position(|pet| *pet == pair[1])
            })
        {
            return Err("invalid first-playthrough pet selection".into());
        }
        let mut state = Self::empty_board(seed);
        state.level = 5;
        state.egg_price = FIFTH_STAGE_EGG_PRICE;
        state.invasion = Some(Invasion1_2::new_tank1_finale());
        state.pets = pets.to_vec();
        for pet in pets {
            match pet {
                PetKind::Stinky => {
                    state.stinky = vec![state.spawn_stinky(StinkyOrigin::StageStart)]
                }
                PetKind::Niko => {
                    let owner_id = state.id();
                    state.niko = vec![NikoState::spawn_tank1(owner_id, &mut |upper| {
                        state.rand_range(upper)
                    })];
                }
                PetKind::Itchy => state.spawn_fish_pet(FishPetKind::Itchy),
                PetKind::Prego => state.spawn_fish_pet(FishPetKind::Prego),
                PetKind::Zorf
                | PetKind::Clyde
                | PetKind::Vert
                | PetKind::Rufus
                | PetKind::Meryl
                | PetKind::Wadsworth
                | PetKind::Seymour
                | PetKind::Shrapnel
                | PetKind::Gumbo
                | PetKind::Blip
                | PetKind::Rhubarb
                | PetKind::Nimbus
                | PetKind::Amp
                | PetKind::Gash
                | PetKind::Angie
                | PetKind::Presto
                | PetKind::Brinkley
                | PetKind::Nostradamus
                | PetKind::Stanley
                | PetKind::Walter => {
                    unreachable!("roster checked before construction")
                }
            }
        }
        state.spawn_starter_guppies(false);
        Ok(state)
    }

    /// A fresh second-tank board preserves the selected roster order and
    /// constructs the pets before either ordinary starter guppy.
    pub fn new_tank2_first_stage(seed: u64, pets: &[PetKind]) -> Result<Self, String> {
        let canonical = [
            PetKind::Stinky,
            PetKind::Niko,
            PetKind::Itchy,
            PetKind::Prego,
            PetKind::Zorf,
        ];
        if pets.len() > 3
            || pets.iter().any(|pet| !canonical.contains(pet))
            || pets.windows(2).any(|pair| {
                canonical.iter().position(|pet| *pet == pair[0])
                    >= canonical.iter().position(|pet| *pet == pair[1])
            })
        {
            return Err("invalid second-tank pet selection".into());
        }
        let mut state = Self::empty_board(seed);
        state.tank = 2;
        state.level = 1;
        state.egg_price = TANK2_FIRST_EGG_PRICE;
        state.invasion = Some(Invasion1_2::new_strong());
        state.pets = pets.to_vec();
        for pet in pets {
            match pet {
                PetKind::Stinky => {
                    state.stinky = vec![state.spawn_stinky(StinkyOrigin::StageStart)]
                }
                PetKind::Niko => {
                    let owner_id = state.id();
                    state.niko = vec![NikoState::spawn_tank2(owner_id, &mut |upper| {
                        state.rand_range(upper)
                    })];
                }
                PetKind::Itchy => state.spawn_fish_pet(FishPetKind::Itchy),
                PetKind::Prego => state.spawn_fish_pet(FishPetKind::Prego),
                PetKind::Zorf => state.spawn_fish_pet(FishPetKind::Zorf),
                PetKind::Clyde
                | PetKind::Vert
                | PetKind::Rufus
                | PetKind::Meryl
                | PetKind::Wadsworth
                | PetKind::Seymour
                | PetKind::Shrapnel
                | PetKind::Gumbo
                | PetKind::Blip
                | PetKind::Rhubarb
                | PetKind::Nimbus
                | PetKind::Amp
                | PetKind::Gash
                | PetKind::Angie
                | PetKind::Presto
                | PetKind::Brinkley
                | PetKind::Nostradamus
                | PetKind::Stanley
                | PetKind::Walter => {
                    unreachable!("roster checked before construction")
                }
            }
        }
        state.spawn_starter_guppies(false);
        Ok(state)
    }

    pub fn new_tank2_second_stage(seed: u64, pets: &[PetKind]) -> Result<Self, String> {
        let canonical = [
            PetKind::Stinky,
            PetKind::Niko,
            PetKind::Itchy,
            PetKind::Prego,
            PetKind::Zorf,
            PetKind::Clyde,
        ];
        if pets.len() > 3
            || pets.iter().any(|pet| !canonical.contains(pet))
            || pets.windows(2).any(|pair| {
                canonical.iter().position(|pet| *pet == pair[0])
                    >= canonical.iter().position(|pet| *pet == pair[1])
            })
        {
            return Err("invalid Adventure 2-2 pet selection".into());
        }
        let mut state = Self::empty_board(seed);
        state.tank = 2;
        state.level = 2;
        state.egg_price = TANK2_SECOND_EGG_PRICE;
        state.invasion = Some(Invasion1_2::new_balrog());
        state.pets = pets.to_vec();
        for pet in pets {
            match pet {
                PetKind::Stinky => {
                    state.stinky = vec![state.spawn_stinky(StinkyOrigin::StageStart)]
                }
                PetKind::Niko => {
                    let owner_id = state.id();
                    state.niko = vec![NikoState::spawn_tank2(owner_id, &mut |upper| {
                        state.rand_range(upper)
                    })];
                }
                PetKind::Itchy => state.spawn_fish_pet(FishPetKind::Itchy),
                PetKind::Prego => state.spawn_fish_pet(FishPetKind::Prego),
                PetKind::Zorf => state.spawn_fish_pet(FishPetKind::Zorf),
                PetKind::Clyde => {
                    let id = state.id();
                    let mut rng_state = state.rng_state;
                    state.clyde = vec![ClydeState::spawn_tank2(id, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    })];
                    state.rng_state = rng_state;
                }
                PetKind::Vert
                | PetKind::Rufus
                | PetKind::Meryl
                | PetKind::Wadsworth
                | PetKind::Seymour
                | PetKind::Shrapnel
                | PetKind::Gumbo
                | PetKind::Blip
                | PetKind::Rhubarb
                | PetKind::Nimbus
                | PetKind::Amp
                | PetKind::Gash
                | PetKind::Angie
                | PetKind::Presto
                | PetKind::Brinkley
                | PetKind::Nostradamus
                | PetKind::Stanley
                | PetKind::Walter => {
                    unreachable!("roster checked before construction")
                }
            }
        }
        state.spawn_starter_guppies(false);
        Ok(state)
    }

    pub fn new_tank2_third_stage(seed: u64, pets: &[PetKind]) -> Result<Self, String> {
        let canonical = [
            PetKind::Stinky,
            PetKind::Niko,
            PetKind::Itchy,
            PetKind::Prego,
            PetKind::Zorf,
            PetKind::Clyde,
            PetKind::Vert,
        ];
        if pets.len() > 3
            || pets.iter().any(|pet| !canonical.contains(pet))
            || pets.windows(2).any(|pair| {
                canonical.iter().position(|pet| *pet == pair[0])
                    >= canonical.iter().position(|pet| *pet == pair[1])
            })
        {
            return Err("invalid Adventure 2-3 pet selection".into());
        }
        let mut state = Self::empty_board(seed);
        state.tank = 2;
        state.level = 3;
        state.egg_price = TANK2_THIRD_EGG_PRICE;
        state.invasion = Some(Invasion1_2::new_gus());
        state.pets = pets.to_vec();
        for pet in pets {
            match pet {
                PetKind::Stinky => {
                    state.stinky = vec![state.spawn_stinky(StinkyOrigin::StageStart)]
                }
                PetKind::Niko => {
                    let owner_id = state.id();
                    state.niko = vec![NikoState::spawn_tank2(owner_id, &mut |upper| {
                        state.rand_range(upper)
                    })];
                }
                PetKind::Itchy => state.spawn_fish_pet(FishPetKind::Itchy),
                PetKind::Prego => state.spawn_fish_pet(FishPetKind::Prego),
                PetKind::Zorf => state.spawn_fish_pet(FishPetKind::Zorf),
                PetKind::Clyde => {
                    let id = state.id();
                    let mut rng_state = state.rng_state;
                    state.clyde = vec![ClydeState::spawn_tank2(id, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    })];
                    state.rng_state = rng_state;
                }
                PetKind::Vert => state.spawn_fish_pet(FishPetKind::Vert),
                PetKind::Rufus
                | PetKind::Meryl
                | PetKind::Wadsworth
                | PetKind::Seymour
                | PetKind::Shrapnel
                | PetKind::Gumbo
                | PetKind::Blip
                | PetKind::Rhubarb
                | PetKind::Nimbus
                | PetKind::Amp
                | PetKind::Gash
                | PetKind::Angie
                | PetKind::Presto
                | PetKind::Brinkley
                | PetKind::Nostradamus
                | PetKind::Stanley
                | PetKind::Walter => {
                    unreachable!("roster checked before construction")
                }
            }
        }
        state.spawn_starter_guppies(false);
        Ok(state)
    }

    pub fn new_tank2_fourth_stage(seed: u64, pets: &[PetKind]) -> Result<Self, String> {
        let canonical = [
            PetKind::Stinky,
            PetKind::Niko,
            PetKind::Itchy,
            PetKind::Prego,
            PetKind::Zorf,
            PetKind::Clyde,
            PetKind::Vert,
            PetKind::Rufus,
        ];
        if pets.len() > 3
            || pets.iter().any(|pet| !canonical.contains(pet))
            || pets.windows(2).any(|pair| {
                canonical.iter().position(|pet| *pet == pair[0])
                    >= canonical.iter().position(|pet| *pet == pair[1])
            })
        {
            return Err("invalid Adventure 2-4 pet selection".into());
        }
        let mut state = Self::empty_board(seed);
        state.tank = 2;
        state.level = 4;
        state.egg_price = TANK2_FOURTH_EGG_PRICE;
        state.invasion = Some(Invasion1_2::new_destructor());
        state.pets = pets.to_vec();
        for pet in pets {
            match pet {
                PetKind::Stinky => {
                    state.stinky = vec![state.spawn_stinky(StinkyOrigin::StageStart)]
                }
                PetKind::Niko => {
                    let owner_id = state.id();
                    state.niko = vec![NikoState::spawn_tank2(owner_id, &mut |upper| {
                        state.rand_range(upper)
                    })];
                }
                PetKind::Itchy => state.spawn_fish_pet(FishPetKind::Itchy),
                PetKind::Prego => state.spawn_fish_pet(FishPetKind::Prego),
                PetKind::Zorf => state.spawn_fish_pet(FishPetKind::Zorf),
                PetKind::Clyde => {
                    let id = state.id();
                    let mut rng_state = state.rng_state;
                    state.clyde = vec![ClydeState::spawn_tank2(id, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    })];
                    state.rng_state = rng_state;
                }
                PetKind::Vert => state.spawn_fish_pet(FishPetKind::Vert),
                PetKind::Rufus => {
                    let id = state.id();
                    let mut rng_state = state.rng_state;
                    state.rufus = vec![RufusState::spawn_tank2(id, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    })];
                    state.rng_state = rng_state;
                }
                PetKind::Meryl
                | PetKind::Wadsworth
                | PetKind::Seymour
                | PetKind::Shrapnel
                | PetKind::Gumbo
                | PetKind::Blip
                | PetKind::Rhubarb
                | PetKind::Nimbus
                | PetKind::Amp
                | PetKind::Gash
                | PetKind::Angie
                | PetKind::Presto
                | PetKind::Brinkley
                | PetKind::Nostradamus
                | PetKind::Stanley
                | PetKind::Walter => {
                    unreachable!("roster checked before construction")
                }
            }
        }
        state.spawn_starter_guppies(false);
        Ok(state)
    }

    pub fn new_tank2_fifth_stage(seed: u64, pets: &[PetKind]) -> Result<Self, String> {
        let canonical = [
            PetKind::Stinky,
            PetKind::Niko,
            PetKind::Itchy,
            PetKind::Prego,
            PetKind::Zorf,
            PetKind::Clyde,
            PetKind::Vert,
            PetKind::Rufus,
            PetKind::Meryl,
        ];
        if pets.len() > 3
            || pets.iter().any(|pet| !canonical.contains(pet))
            || pets.windows(2).any(|pair| {
                canonical.iter().position(|pet| *pet == pair[0])
                    >= canonical.iter().position(|pet| *pet == pair[1])
            })
        {
            return Err("invalid Adventure 2-5 pet selection".into());
        }
        let mut state = Self::empty_board(seed);
        state.tank = 2;
        state.level = 5;
        state.egg_price = TANK2_FIFTH_EGG_PRICE;
        // Board::InitLevel chooses the first expected species before
        // Board::StartGame constructs selected pets and starter guppies.
        let first = if state.rand_range(2) != 0 {
            SylvesterKind::Destructor
        } else {
            SylvesterKind::Gus
        };
        state.invasion = Some(Invasion1_2::new_tank2_finale(first));
        state.pets = pets.to_vec();
        for pet in pets {
            match pet {
                PetKind::Stinky => {
                    state.stinky = vec![state.spawn_stinky(StinkyOrigin::StageStart)]
                }
                PetKind::Niko => {
                    let owner_id = state.id();
                    state.niko = vec![NikoState::spawn_tank2(owner_id, &mut |upper| {
                        state.rand_range(upper)
                    })];
                }
                PetKind::Itchy => state.spawn_fish_pet(FishPetKind::Itchy),
                PetKind::Prego => state.spawn_fish_pet(FishPetKind::Prego),
                PetKind::Zorf => state.spawn_fish_pet(FishPetKind::Zorf),
                PetKind::Clyde => {
                    let id = state.id();
                    let mut rng_state = state.rng_state;
                    state.clyde = vec![ClydeState::spawn_tank2(id, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    })];
                    state.rng_state = rng_state;
                }
                PetKind::Vert => state.spawn_fish_pet(FishPetKind::Vert),
                PetKind::Rufus => {
                    let id = state.id();
                    let mut rng_state = state.rng_state;
                    state.rufus = vec![RufusState::spawn_tank2(id, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    })];
                    state.rng_state = rng_state;
                }
                PetKind::Meryl => state.spawn_fish_pet(FishPetKind::Meryl),
                PetKind::Wadsworth
                | PetKind::Seymour
                | PetKind::Shrapnel
                | PetKind::Gumbo
                | PetKind::Blip
                | PetKind::Rhubarb
                | PetKind::Nimbus
                | PetKind::Amp
                | PetKind::Gash
                | PetKind::Angie
                | PetKind::Presto
                | PetKind::Brinkley
                | PetKind::Nostradamus
                | PetKind::Stanley
                | PetKind::Walter => {
                    unreachable!("roster checked before construction")
                }
            }
        }
        state.spawn_starter_guppies(false);
        Ok(state)
    }

    /// Ordinary Tank 3-1 begins after the earned Tank 2 bonus. Selected pets
    /// are constructed before the two starter guppies, preserving Board's
    /// actor-constructor RNG order; the Balrog countdown starts at 3000.
    pub fn new_tank3_first_stage(seed: u64, pets: &[PetKind]) -> Result<Self, String> {
        Self::validate_tank3_selection(pets, false, false, false, false)?;
        let mut state = Self::empty_board(seed);
        state.tank = 3;
        state.level = 1;
        state.egg_price = TANK3_FIRST_EGG_PRICE;
        state.invasion = Some(Invasion1_2::new_balrog());
        state.spawn_tank3_pets(pets);
        state.spawn_starter_guppies(false);
        Ok(state)
    }

    /// The first 3-2 expected alien draw precedes pet and starter-fish
    /// constructors. Later waves use one conditional toggle roll after spawn.
    pub fn new_tank3_second_stage(seed: u64, pets: &[PetKind]) -> Result<Self, String> {
        Self::validate_tank3_selection(pets, true, false, false, false)?;
        let mut state = Self::empty_board(seed);
        state.tank = 3;
        state.level = 2;
        state.egg_price = TANK3_SECOND_EGG_PRICE;
        let first = if state.rand_range(2) == 0 {
            SylvesterKind::Gus
        } else {
            SylvesterKind::Destructor
        };
        state.invasion = Some(Invasion1_2::new_tank3_second_stage(first));
        state.spawn_tank3_pets(pets);
        state.spawn_starter_guppies(false);
        Ok(state)
    }

    pub fn new_tank3_third_stage(seed: u64, pets: &[PetKind]) -> Result<Self, String> {
        Self::validate_tank3_selection(pets, true, true, false, false)?;
        let mut state = Self::empty_board(seed);
        state.tank = 3;
        state.level = 3;
        state.egg_price = TANK3_THIRD_EGG_PRICE;
        state.invasion = Some(Invasion1_2::new_psychosquid());
        state.spawn_tank3_pets(pets);
        state.spawn_starter_guppies(false);
        Ok(state)
    }

    pub fn new_tank3_fourth_stage(seed: u64, pets: &[PetKind]) -> Result<Self, String> {
        Self::validate_tank3_selection(pets, true, true, true, false)?;
        let mut state = Self::empty_board(seed);
        state.tank = 3;
        state.level = 4;
        state.egg_price = TANK3_FOURTH_EGG_PRICE;
        state.invasion = Some(Invasion1_2::new_ulysses());
        state.spawn_tank3_pets(pets);
        state.spawn_starter_guppies(false);
        Ok(state)
    }

    pub fn new_tank3_fifth_stage(seed: u64, pets: &[PetKind]) -> Result<Self, String> {
        Self::validate_tank3_selection(pets, true, true, true, true)?;
        let mut state = Self::empty_board(seed);
        state.tank = 3;
        state.level = 5;
        state.egg_price = TANK3_FIFTH_EGG_PRICE;
        // PB51: the opening bit is consumed before selected pet and starter
        // constructors; later waves make a separate inverted-bit draw.
        let first = if state.rand_range(2) == 0 {
            SylvesterKind::Ulysses
        } else {
            SylvesterKind::Psychosquid
        };
        state.invasion = Some(Invasion1_2::new_tank3_finale(first));
        state.spawn_tank3_pets(pets);
        state.spawn_starter_guppies(false);
        Ok(state)
    }

    pub fn new_tank4_first_stage(seed: u64, pets: &[PetKind]) -> Result<Self, String> {
        Self::new_tank4_stage(seed, pets, 1)
    }

    pub fn new_tank4_second_stage(seed: u64, pets: &[PetKind]) -> Result<Self, String> {
        Self::new_tank4_stage(seed, pets, 2)
    }

    pub fn new_tank4_third_stage(seed: u64, pets: &[PetKind]) -> Result<Self, String> {
        Self::new_tank4_stage(seed, pets, 3)
    }

    pub fn new_tank4_fourth_stage(seed: u64, pets: &[PetKind]) -> Result<Self, String> {
        Self::new_tank4_stage(seed, pets, 4)
    }

    pub fn new_tank4_finale(seed: u64, pets: &[PetKind]) -> Result<Self, String> {
        Self::new_tank4_stage(seed, pets, 5)
    }

    pub fn new_tank5_1(seed: u64, profile_attempts: u32) -> Self {
        let mut state = Self::empty_board(seed);
        state.tank = 5;
        state.level = 1;
        state.egg_price = 0;
        // PB05 00541200: internal egg counter3. Project eggs are counter-1.
        state.eggs = 2;
        state.invasion = Some(Invasion1_2::new_tank5_finale(profile_attempts));
        state.spawn_tank4_pets(&TANK5_PETS);
        state
    }

    /// A fresh profile-selected Adventure board uses the same stage economy
    /// and opening wave, then constructs selected pets before starter fish or
    /// the Tank 5 fixed roster (PB05 005498b0). First-run constructors remain
    /// separate so their historical fixed rosters and draw schedules stay put.
    fn profile_egg_price(tank: u8, level: u8) -> Option<i32> {
        Some(match (tank, level) {
            (1, 1) => EGG_PRICE,
            (1, 2) => SECOND_STAGE_EGG_PRICE,
            (1, 3) => THIRD_STAGE_EGG_PRICE,
            (1, 4) => FOURTH_STAGE_EGG_PRICE,
            (1, 5) => FIFTH_STAGE_EGG_PRICE,
            (2, 1) => TANK2_FIRST_EGG_PRICE,
            (2, 2) => TANK2_SECOND_EGG_PRICE,
            (2, 3) => TANK2_THIRD_EGG_PRICE,
            (2, 4) => TANK2_FOURTH_EGG_PRICE,
            (2, 5) => TANK2_FIFTH_EGG_PRICE,
            (3, 1) => TANK3_FIRST_EGG_PRICE,
            (3, 2) => TANK3_SECOND_EGG_PRICE,
            (3, 3) => TANK3_THIRD_EGG_PRICE,
            (3, 4) => TANK3_FOURTH_EGG_PRICE,
            (3, 5) => TANK3_FIFTH_EGG_PRICE,
            (4, 1) => TANK4_FIRST_EGG_PRICE,
            (4, 2) => TANK4_SECOND_EGG_PRICE,
            (4, 3) => TANK4_THIRD_EGG_PRICE,
            (4, 4) => TANK4_FOURTH_EGG_PRICE,
            (4, 5) => TANK4_FINALE_EGG_PRICE,
            (5, 1) => 0,
            _ => return None,
        })
    }

    pub fn new_profile_stage(
        seed: u64,
        tank: u8,
        level: u8,
        pets: &[PetKind],
        profile_attempts: u32,
    ) -> Result<Self, String> {
        if !matches!((tank, level), (1..=4, 1..=5) | (5, 1))
            || pets.len() > 3
            || pets
                .iter()
                .any(|pet| !crate::time_trial::selectable_pet(*pet))
            || pets
                .iter()
                .enumerate()
                .any(|(index, pet)| pets[..index].contains(pet))
            || pets.windows(2).any(|pair| pair[0] as u8 >= pair[1] as u8)
        {
            return Err("invalid selected Adventure profile roster".into());
        }
        let mut state = Self::empty_board(seed);
        state.tank = tank;
        state.level = level;
        state.profile_population = true;
        state.egg_price = Self::profile_egg_price(tank, level).expect("stage checked above");
        state.invasion = match (tank, level) {
            (1, 1) => None,
            (1, 2) => Some(Invasion1_2::new()),
            (1, 3) | (2, 1) => Some(Invasion1_2::new_strong()),
            (1, 4) | (2, 2) | (3, 1) | (4, 1) => Some(Invasion1_2::new_balrog()),
            (1, 5) => Some(Invasion1_2::new_tank1_finale()),
            (2, 3) => Some(Invasion1_2::new_gus()),
            (2, 4) => Some(Invasion1_2::new_destructor()),
            (2, 5) => {
                let first = if state.rand_range(2) != 0 {
                    SylvesterKind::Destructor
                } else {
                    SylvesterKind::Gus
                };
                Some(Invasion1_2::new_tank2_finale(first))
            }
            (3, 2) => {
                let first = if state.rand_range(2) == 0 {
                    SylvesterKind::Gus
                } else {
                    SylvesterKind::Destructor
                };
                Some(Invasion1_2::new_tank3_second_stage(first))
            }
            (3, 3) => Some(Invasion1_2::new_psychosquid()),
            (3, 4) => Some(Invasion1_2::new_ulysses()),
            (3, 5) => {
                let first = if state.rand_range(2) == 0 {
                    SylvesterKind::Ulysses
                } else {
                    SylvesterKind::Psychosquid
                };
                Some(Invasion1_2::new_tank3_finale(first))
            }
            (4, 2) => Some(Invasion1_2::new_bilaterus()),
            (4, 3) => Some(Invasion1_2::new_tank4_third_stage()),
            (4, 4) => Some(Invasion1_2::new_tank4_fourth_stage()),
            (4, 5) => Some(Invasion1_2::new_tank4_finale()),
            (5, 1) => {
                state.eggs = 2;
                Some(Invasion1_2::new_tank5_finale(profile_attempts))
            }
            _ => unreachable!("stage checked above"),
        };
        for pet in pets {
            state.spawn_profile_pet(*pet);
        }
        match tank {
            1..=3 => state.spawn_starter_guppies(tank == 1 && level == 1),
            4 => {
                let x = state.rand_range(520) as i32 + 20;
                let y = state.rand_range(265) as i32 + 105;
                let id = state.id();
                let mut rng_state = state.rng_state;
                let starter = BreederState::spawn_starter(id, x, y, &mut |upper| {
                    Self::advance_rng(&mut rng_state) % upper
                });
                state.rng_state = rng_state;
                state.breeders.push(starter);
            }
            5 => {
                for pet in TANK5_PETS {
                    state.spawn_profile_pet(pet);
                }
            }
            _ => unreachable!("stage checked above"),
        }
        Ok(state)
    }

    fn spawn_profile_pet(&mut self, kind: PetKind) {
        match kind {
            PetKind::Stinky => {
                let pet = self.spawn_stinky(StinkyOrigin::StageStart);
                self.stinky.push(pet);
            }
            PetKind::Niko => {
                let id = self.id();
                let pet = match self.tank {
                    1 => NikoState::spawn_tank1(id, &mut |upper| self.rand_range(upper)),
                    2 => NikoState::spawn_tank2(id, &mut |upper| self.rand_range(upper)),
                    3 => NikoState::spawn_tank3(id, &mut |upper| self.rand_range(upper)),
                    4 => NikoState::spawn_tank4(id, &mut |upper| self.rand_range(upper)),
                    5 => NikoState::spawn_tank5(id, &mut |upper| self.rand_range(upper)),
                    _ => unreachable!(),
                };
                self.niko.push(pet);
            }
            PetKind::Clyde => {
                let id = self.id();
                let mut rng_state = self.rng_state;
                let pet = if self.tank == 5 {
                    ClydeState::spawn_tank5(id, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    })
                } else {
                    ClydeState::spawn_tank2(id, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    })
                };
                self.rng_state = rng_state;
                self.clyde.push(pet);
            }
            PetKind::Rufus => {
                let id = self.id();
                let mut rng_state = self.rng_state;
                let pet = if self.tank == 5 {
                    RufusState::spawn_tank5(id, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    })
                } else {
                    RufusState::spawn_tank2(id, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    })
                };
                self.rng_state = rng_state;
                self.rufus.push(pet);
            }
            PetKind::Rhubarb => {
                let id = self.id();
                let x = self.rand_range(265) as i32 + 105;
                let y = self.rand_range(520) as i32 + 20;
                let mut rng_state = self.rng_state;
                let pet = if self.tank == 5 {
                    RhubarbState::spawn_tank5(id, x, y, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    })
                } else {
                    RhubarbState::spawn_tank4(id, x, y, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    })
                };
                self.rng_state = rng_state;
                self.rhubarb.push(pet);
            }
            other => self.spawn_fish_pet(
                Self::fish_kind_from_pet(other).expect("profile raw kind is supported"),
            ),
        }
        self.pets.push(kind);
    }

    /// Fresh Time Trial Board uses the mapped level-five tank setup and
    /// constructs its initial pets before the ordinary starter actors.
    pub fn new_time_trial(seed: u64, tank: u8, pets: &[PetKind]) -> Result<Self, String> {
        if !(1..=4).contains(&tank)
            || pets.len() > 3
            || pets
                .iter()
                .any(|pet| !crate::time_trial::selectable_pet(*pet))
            || pets
                .iter()
                .enumerate()
                .any(|(index, pet)| pets[..index].contains(pet))
        {
            return Err("invalid Time Trial tank or initial pets".into());
        }
        let mut state = Self::empty_board(seed);
        state.tank = tank;
        state.level = 5;
        state.time_trial = true;
        state.egg_price = crate::time_trial::TIME_TRIAL_EGG_PRICE;
        state.balance = crate::time_trial::TIME_TRIAL_START_BALANCE;
        state.egg_unlocked = true;
        state.invasion = Some(Invasion1_2::new_time_trial(tank));
        for pet in pets {
            state.spawn_time_trial_pet(*pet)?;
        }
        if tank == 4 {
            let x = state.rand_range(520) as i32 + 20;
            let y = state.rand_range(265) as i32 + 105;
            let id = state.id();
            let mut rng_state = state.rng_state;
            let starter = BreederState::spawn_starter(id, x, y, &mut |upper| {
                Self::advance_rng(&mut rng_state) % upper
            });
            state.rng_state = rng_state;
            state.breeders.push(starter);
        } else {
            state.spawn_starter_guppies(false);
        }
        Ok(state)
    }

    pub(crate) fn time_trial_candidates(&self, unlocked: &[PetKind]) -> Vec<PetKind> {
        crate::time_trial::SUPPORTED_PETS
            .iter()
            .copied()
            .filter(|pet| unlocked.contains(pet) && !self.has_live_pet_kind(*pet))
            .collect()
    }

    pub(crate) fn presto_actor(&self) -> Option<PrestoActorView> {
        for pet in &self.stinky {
            if let (Some(id), Some(form)) = (pet.combat_id, pet.presto_form) {
                return Some(PrestoActorView {
                    id,
                    kind: PetKind::Stinky,
                    widget_x: pet.x as i32,
                    widget_y: pet.y as i32,
                    remaining_ticks: form.remaining_ticks,
                });
            }
        }
        for pet in &self.niko {
            if let Some(form) = pet.presto_form {
                return Some(PrestoActorView {
                    id: pet.owner_id,
                    kind: PetKind::Niko,
                    widget_x: pet.anchor_x,
                    widget_y: pet.anchor_y,
                    remaining_ticks: form.remaining_ticks,
                });
            }
        }
        for pet in &self.clyde {
            if let Some(form) = pet.presto_form {
                return Some(PrestoActorView {
                    id: pet.id,
                    kind: PetKind::Clyde,
                    widget_x: pet.widget_x,
                    widget_y: pet.widget_y,
                    remaining_ticks: form.remaining_ticks,
                });
            }
        }
        for pet in &self.rufus {
            if let Some(form) = pet.presto_form {
                return Some(PrestoActorView {
                    id: pet.id,
                    kind: PetKind::Rufus,
                    widget_x: pet.widget_x,
                    widget_y: pet.widget_y,
                    remaining_ticks: form.remaining_ticks,
                });
            }
        }
        for pet in &self.rhubarb {
            if let Some(form) = pet.presto_form {
                return Some(PrestoActorView {
                    id: pet.id,
                    kind: PetKind::Rhubarb,
                    widget_x: pet.widget_x,
                    widget_y: pet.widget_y,
                    remaining_ticks: form.remaining_ticks,
                });
            }
        }
        self.fish_pets.iter().find_map(|pet| {
            pet.presto_form.map(|form| PrestoActorView {
                id: pet.id,
                kind: Self::pet_kind_from_fish(pet.kind),
                widget_x: pet.widget_x,
                widget_y: pet.widget_y,
                remaining_ticks: form.remaining_ticks,
            })
        })
    }

    pub(crate) fn presto_hit(&self, x: i32, y: i32) -> Option<u64> {
        self.presto_actor()
            .filter(|actor| {
                (actor.widget_x..actor.widget_x + 80).contains(&x)
                    && (actor.widget_y..actor.widget_y + 80).contains(&y)
            })
            .map(|actor| actor.id)
    }

    fn pet_kind_from_fish(kind: FishPetKind) -> PetKind {
        match kind {
            FishPetKind::Itchy => PetKind::Itchy,
            FishPetKind::Prego => PetKind::Prego,
            FishPetKind::Zorf => PetKind::Zorf,
            FishPetKind::Vert => PetKind::Vert,
            FishPetKind::Meryl => PetKind::Meryl,
            FishPetKind::Wadsworth => PetKind::Wadsworth,
            FishPetKind::Seymour => PetKind::Seymour,
            FishPetKind::Shrapnel => PetKind::Shrapnel,
            FishPetKind::Gumbo => PetKind::Gumbo,
            FishPetKind::Blip => PetKind::Blip,
            FishPetKind::Nimbus => PetKind::Nimbus,
            FishPetKind::Amp => PetKind::Amp,
            FishPetKind::Gash => PetKind::Gash,
            FishPetKind::Angie => PetKind::Angie,
            FishPetKind::Presto => PetKind::Presto,
            FishPetKind::Brinkley => PetKind::Brinkley,
            FishPetKind::Nostradamus => PetKind::Nostradamus,
            FishPetKind::Stanley => PetKind::Stanley,
            FishPetKind::Walter => PetKind::Walter,
        }
    }

    fn fish_kind_from_pet(kind: PetKind) -> Option<FishPetKind> {
        match kind {
            PetKind::Itchy => Some(FishPetKind::Itchy),
            PetKind::Prego => Some(FishPetKind::Prego),
            PetKind::Zorf => Some(FishPetKind::Zorf),
            PetKind::Vert => Some(FishPetKind::Vert),
            PetKind::Meryl => Some(FishPetKind::Meryl),
            PetKind::Wadsworth => Some(FishPetKind::Wadsworth),
            PetKind::Seymour => Some(FishPetKind::Seymour),
            PetKind::Shrapnel => Some(FishPetKind::Shrapnel),
            PetKind::Gumbo => Some(FishPetKind::Gumbo),
            PetKind::Blip => Some(FishPetKind::Blip),
            PetKind::Nimbus => Some(FishPetKind::Nimbus),
            PetKind::Amp => Some(FishPetKind::Amp),
            PetKind::Gash => Some(FishPetKind::Gash),
            PetKind::Angie => Some(FishPetKind::Angie),
            PetKind::Presto => Some(FishPetKind::Presto),
            PetKind::Brinkley => Some(FishPetKind::Brinkley),
            PetKind::Nostradamus => Some(FishPetKind::Nostradamus),
            PetKind::Stanley => Some(FishPetKind::Stanley),
            PetKind::Walter => Some(FishPetKind::Walter),
            _ => None,
        }
    }

    /// Construct/attach/remove/detach is one Board-owned transaction. Neither
    /// a partially changed roster nor a stale target can cross a save boundary.
    pub(crate) fn change_presto_form(
        &mut self,
        opener_id: u64,
        target: PetKind,
        events: &mut Vec<Event>,
    ) -> Result<PrestoChangeEligibility, String> {
        let old = self.presto_actor().ok_or("missing Presto incarnation")?;
        if !(self.time_trial || self.profile_population)
            || !self.pets.contains(&PetKind::Presto)
            || old.id != opener_id
            || !crate::time_trial::selectable_pet(target)
        {
            return Err("invalid Presto replacement opener or target".into());
        }
        if old.kind == target {
            return Ok(PrestoChangeEligibility::SameForm);
        }
        if old.remaining_ticks != 0 {
            return Ok(PrestoChangeEligibility::CoolingDown);
        }
        self.commit_presto_replacement(old, target, events)?;
        Ok(PrestoChangeEligibility::Ready)
    }

    fn return_presto_after_walter_hit(
        &mut self,
        old_id: u64,
        events: &mut Vec<Event>,
    ) -> Result<(), String> {
        let old = self.presto_actor().ok_or("missing Presto incarnation")?;
        let valid = self.fish_pets.iter().any(|pet| {
            pet.id == old_id
                && pet.presto_form.is_some()
                && pet.kind != FishPetKind::Presto
                && pet.walter.recoil < 41
                && pet
                    .presto_form
                    .is_some_and(|form| form.remaining_ticks == 0)
        });
        if !valid || old.id != old_id {
            return Err("invalid Walter recoil replacement".into());
        }
        self.commit_presto_replacement(old, PetKind::Presto, events)
    }

    fn commit_presto_replacement(
        &mut self,
        old: PrestoActorView,
        target: PetKind,
        events: &mut Vec<Event>,
    ) -> Result<(), String> {
        let carry = if matches!(
            (old.kind, target),
            (PetKind::Presto, PetKind::Walter) | (PetKind::Walter, PetKind::Presto)
        ) {
            self.fish_pets
                .iter()
                .find(|pet| pet.id == old.id)
                .map(|pet| (pet.walter_punches, pet.walter_cooldown))
        } else {
            None
        };
        let id = self.id();
        let mut rng_state = self.rng_state;
        let mut draw = |upper| Self::advance_rng(&mut rng_state) % upper;
        let (x, y) = (old.widget_x, old.widget_y);
        match target {
            PetKind::Stinky => self
                .stinky
                .push(Self::spawn_stinky_form_at(id, x, y, &mut draw)),
            PetKind::Niko => self
                .niko
                .push(NikoState::spawn_presto_form_at(id, x, y, false, &mut draw)),
            PetKind::Clyde => self
                .clyde
                .push(ClydeState::spawn_presto_form_at(id, x, y, false, &mut draw)),
            PetKind::Rufus => self
                .rufus
                .push(RufusState::spawn_presto_form_at(id, x, y, false, &mut draw)),
            PetKind::Rhubarb => self.rhubarb.push(RhubarbState::spawn_presto_form_at(
                id, x, y, false, &mut draw,
            )),
            other => self.fish_pets.push(FishPetState::spawn_presto_form_at(
                id,
                Self::fish_kind_from_pet(other).ok_or("unsupported Presto form")?,
                x,
                y,
                false,
                &mut draw,
            )),
        }
        self.rng_state = rng_state;
        if let Some((punches, cooldown)) = carry
            && let Some(pet) = self.fish_pets.iter_mut().find(|pet| pet.id == id)
        {
            pet.walter_punches = punches;
            pet.walter_cooldown = cooldown;
        }
        self.stinky.retain(|pet| pet.combat_id != Some(old.id));
        self.niko.retain(|pet| pet.owner_id != old.id);
        self.clyde.retain(|pet| pet.id != old.id);
        self.rufus.retain(|pet| pet.id != old.id);
        self.rhubarb.retain(|pet| pet.id != old.id);
        self.fish_pets.retain(|pet| pet.id != old.id);
        if self.detach_missile_target(old.id, events) {
            self.finish_destructor_battle(events);
        }
        events.push(Event::PrestoChanged {
            tick: self.tick,
            old_id: old.id,
            new_id: id,
            from: old.kind,
            to: target,
        });
        Ok(())
    }

    fn has_live_pet_kind(&self, kind: PetKind) -> bool {
        match kind {
            PetKind::Stinky => !self.stinky.is_empty(),
            PetKind::Niko => !self.niko.is_empty(),
            PetKind::Clyde => !self.clyde.is_empty(),
            PetKind::Rufus => !self.rufus.is_empty(),
            PetKind::Rhubarb => !self.rhubarb.is_empty(),
            PetKind::Presto => {
                self.fish_pets.iter().any(|pet| pet.presto_form.is_some())
                    || self.stinky.iter().any(|pet| pet.presto_form.is_some())
                    || self.niko.iter().any(|pet| pet.presto_form.is_some())
                    || self.clyde.iter().any(|pet| pet.presto_form.is_some())
                    || self.rufus.iter().any(|pet| pet.presto_form.is_some())
                    || self.rhubarb.iter().any(|pet| pet.presto_form.is_some())
            }
            other => self.fish_pets.iter().any(|pet| {
                matches!(
                    (other, pet.kind),
                    (PetKind::Itchy, FishPetKind::Itchy)
                        | (PetKind::Prego, FishPetKind::Prego)
                        | (PetKind::Zorf, FishPetKind::Zorf)
                        | (PetKind::Vert, FishPetKind::Vert)
                        | (PetKind::Meryl, FishPetKind::Meryl)
                        | (PetKind::Wadsworth, FishPetKind::Wadsworth)
                        | (PetKind::Seymour, FishPetKind::Seymour)
                        | (PetKind::Shrapnel, FishPetKind::Shrapnel)
                        | (PetKind::Gumbo, FishPetKind::Gumbo)
                        | (PetKind::Blip, FishPetKind::Blip)
                        | (PetKind::Nimbus, FishPetKind::Nimbus)
                        | (PetKind::Amp, FishPetKind::Amp)
                        | (PetKind::Gash, FishPetKind::Gash)
                        | (PetKind::Angie, FishPetKind::Angie)
                        | (PetKind::Brinkley, FishPetKind::Brinkley)
                        | (PetKind::Nostradamus, FishPetKind::Nostradamus)
                        | (PetKind::Stanley, FishPetKind::Stanley)
                        | (PetKind::Walter, FishPetKind::Walter)
                )
            }),
        }
    }

    pub(crate) fn choose_time_trial_pet(&mut self, candidates: &[PetKind]) -> Option<PetKind> {
        (!candidates.is_empty())
            .then(|| candidates[self.rand_range(candidates.len() as u64) as usize])
    }

    pub(crate) fn spawn_time_trial_pet(&mut self, kind: PetKind) -> Result<(), String> {
        if !self.time_trial
            || !crate::time_trial::selectable_pet(kind)
            || self.has_live_pet_kind(kind)
        {
            return Err("invalid Time Trial pet admission".into());
        }
        match kind {
            PetKind::Stinky => self.stinky = vec![self.spawn_stinky(StinkyOrigin::StageStart)],
            PetKind::Niko => {
                let owner_id = self.id();
                let niko = match self.tank {
                    1 => NikoState::spawn_tank1(owner_id, &mut |upper| self.rand_range(upper)),
                    2 => NikoState::spawn_tank2(owner_id, &mut |upper| self.rand_range(upper)),
                    3 => NikoState::spawn_tank3(owner_id, &mut |upper| self.rand_range(upper)),
                    4 => NikoState::spawn_tank4(owner_id, &mut |upper| self.rand_range(upper)),
                    _ => return Err("invalid Time Trial tank".into()),
                };
                self.niko.push(niko);
            }
            PetKind::Clyde => {
                let id = self.id();
                let mut rng_state = self.rng_state;
                self.clyde.push(ClydeState::spawn_tank2(id, &mut |upper| {
                    Self::advance_rng(&mut rng_state) % upper
                }));
                self.rng_state = rng_state;
            }
            PetKind::Rufus => {
                let id = self.id();
                let mut rng_state = self.rng_state;
                self.rufus.push(RufusState::spawn_tank2(id, &mut |upper| {
                    Self::advance_rng(&mut rng_state) % upper
                }));
                self.rng_state = rng_state;
            }
            PetKind::Rhubarb => {
                let id = self.id();
                let x = self.rand_range(265) as i32 + 105;
                let y = self.rand_range(520) as i32 + 20;
                let mut rng_state = self.rng_state;
                self.rhubarb
                    .push(RhubarbState::spawn_tank4(id, x, y, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    }));
                self.rng_state = rng_state;
            }
            PetKind::Presto => self.spawn_fish_pet(FishPetKind::Presto),
            other => {
                let fish_kind = match other {
                    PetKind::Itchy => FishPetKind::Itchy,
                    PetKind::Prego => FishPetKind::Prego,
                    PetKind::Zorf => FishPetKind::Zorf,
                    PetKind::Vert => FishPetKind::Vert,
                    PetKind::Meryl => FishPetKind::Meryl,
                    PetKind::Wadsworth => FishPetKind::Wadsworth,
                    PetKind::Seymour => FishPetKind::Seymour,
                    PetKind::Shrapnel => FishPetKind::Shrapnel,
                    PetKind::Gumbo => FishPetKind::Gumbo,
                    PetKind::Blip => FishPetKind::Blip,
                    PetKind::Nimbus => FishPetKind::Nimbus,
                    PetKind::Amp => FishPetKind::Amp,
                    PetKind::Gash => FishPetKind::Gash,
                    PetKind::Angie => FishPetKind::Angie,
                    PetKind::Brinkley => FishPetKind::Brinkley,
                    PetKind::Nostradamus => FishPetKind::Nostradamus,
                    PetKind::Stanley => FishPetKind::Stanley,
                    PetKind::Walter => FishPetKind::Walter,
                    _ => return Err("unsupported Time Trial pet".into()),
                };
                self.spawn_fish_pet(fish_kind);
            }
        }
        self.pets.push(kind);
        Ok(())
    }

    fn new_tank4_stage(seed: u64, pets: &[PetKind], level: u8) -> Result<Self, String> {
        let canonical = [
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
        if pets.len() > 3
            || pets.iter().any(|pet| {
                !canonical.contains(pet)
                    || (level == 1 && *pet == PetKind::Nimbus)
                    || (level < 3 && *pet == PetKind::Amp)
                    || (level < 4 && *pet == PetKind::Gash)
                    || (level < 5 && *pet == PetKind::Angie)
            })
            || pets.windows(2).any(|pair| {
                canonical.iter().position(|pet| *pet == pair[0])
                    >= canonical.iter().position(|pet| *pet == pair[1])
            })
        {
            return Err("invalid Tank 4 pet selection".into());
        }
        let mut state = Self::empty_board(seed);
        state.tank = 4;
        state.level = level;
        state.egg_price = match level {
            1 => TANK4_FIRST_EGG_PRICE,
            2 => TANK4_SECOND_EGG_PRICE,
            3 => TANK4_THIRD_EGG_PRICE,
            4 => TANK4_FOURTH_EGG_PRICE,
            5 => TANK4_FINALE_EGG_PRICE,
            _ => unreachable!("unsupported Tank 4 stage"),
        };
        state.invasion = Some(match level {
            1 => Invasion1_2::new_balrog(),
            2 => Invasion1_2::new_bilaterus(),
            3 => Invasion1_2::new_tank4_third_stage(),
            4 => Invasion1_2::new_tank4_fourth_stage(),
            5 => Invasion1_2::new_tank4_finale(),
            _ => unreachable!(),
        });
        state.spawn_tank4_pets(pets);
        // Board::StartGame creates selected pets before the one Breeder, and
        // grants that starter two growth points after construction.
        let x = state.rand_range(520) as i32 + 20;
        let y = state.rand_range(265) as i32 + 105;
        let id = state.id();
        let mut rng_state = state.rng_state;
        let starter = BreederState::spawn_starter(id, x, y, &mut |upper| {
            Self::advance_rng(&mut rng_state) % upper
        });
        state.rng_state = rng_state;
        state.breeders.push(starter);
        Ok(state)
    }

    fn spawn_tank4_pets(&mut self, pets: &[PetKind]) {
        self.pets = pets.to_vec();
        for pet in pets {
            match pet {
                PetKind::Stinky => self.stinky = vec![self.spawn_stinky(StinkyOrigin::StageStart)],
                PetKind::Niko => {
                    let owner_id = self.id();
                    self.niko = vec![if self.tank == 5 {
                        NikoState::spawn_tank5(owner_id, &mut |upper| self.rand_range(upper))
                    } else {
                        NikoState::spawn_tank4(owner_id, &mut |upper| self.rand_range(upper))
                    }];
                }
                PetKind::Itchy => self.spawn_fish_pet(FishPetKind::Itchy),
                PetKind::Prego => self.spawn_fish_pet(FishPetKind::Prego),
                PetKind::Zorf => self.spawn_fish_pet(FishPetKind::Zorf),
                PetKind::Clyde => {
                    let id = self.id();
                    let mut rng_state = self.rng_state;
                    self.clyde = vec![if self.tank == 5 {
                        ClydeState::spawn_tank5(id, &mut |upper| {
                            Self::advance_rng(&mut rng_state) % upper
                        })
                    } else {
                        ClydeState::spawn_tank2(id, &mut |upper| {
                            Self::advance_rng(&mut rng_state) % upper
                        })
                    }];
                    self.rng_state = rng_state;
                }
                PetKind::Vert => self.spawn_fish_pet(FishPetKind::Vert),
                PetKind::Rufus => {
                    let id = self.id();
                    let mut rng_state = self.rng_state;
                    self.rufus = vec![if self.tank == 5 {
                        RufusState::spawn_tank5(id, &mut |upper| {
                            Self::advance_rng(&mut rng_state) % upper
                        })
                    } else {
                        RufusState::spawn_tank2(id, &mut |upper| {
                            Self::advance_rng(&mut rng_state) % upper
                        })
                    }];
                    self.rng_state = rng_state;
                }
                PetKind::Meryl => self.spawn_fish_pet(FishPetKind::Meryl),
                PetKind::Wadsworth => self.spawn_fish_pet(FishPetKind::Wadsworth),
                PetKind::Seymour => self.spawn_fish_pet(FishPetKind::Seymour),
                PetKind::Shrapnel => self.spawn_fish_pet(FishPetKind::Shrapnel),
                PetKind::Gumbo => self.spawn_fish_pet(FishPetKind::Gumbo),
                PetKind::Blip => self.spawn_fish_pet(FishPetKind::Blip),
                PetKind::Rhubarb => {
                    let id = self.id();
                    let x = self.rand_range(265) as i32 + 105;
                    let y = self.rand_range(520) as i32 + 20;
                    let mut rng_state = self.rng_state;
                    self.rhubarb = vec![if self.tank == 5 {
                        RhubarbState::spawn_tank5(id, x, y, &mut |upper| {
                            Self::advance_rng(&mut rng_state) % upper
                        })
                    } else {
                        RhubarbState::spawn_tank4(id, x, y, &mut |upper| {
                            Self::advance_rng(&mut rng_state) % upper
                        })
                    }];
                    self.rng_state = rng_state;
                }
                PetKind::Nimbus => self.spawn_fish_pet(FishPetKind::Nimbus),
                PetKind::Amp => self.spawn_fish_pet(FishPetKind::Amp),
                PetKind::Gash => self.spawn_fish_pet(FishPetKind::Gash),
                PetKind::Angie => self.spawn_fish_pet(FishPetKind::Angie),
                PetKind::Presto
                | PetKind::Brinkley
                | PetKind::Nostradamus
                | PetKind::Stanley
                | PetKind::Walter => {
                    unreachable!("extra pets are not in the fixed Tank4 or Tank5 roster")
                }
            }
        }
    }

    fn validate_tank3_selection(
        pets: &[PetKind],
        allow_seymour: bool,
        allow_shrapnel: bool,
        allow_gumbo: bool,
        allow_blip: bool,
    ) -> Result<(), String> {
        let canonical = [
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
        ];
        if pets.len() > 3
            || pets.iter().any(|pet| {
                !canonical.contains(pet)
                    || *pet == PetKind::Seymour && !allow_seymour
                    || *pet == PetKind::Shrapnel && !allow_shrapnel
                    || *pet == PetKind::Gumbo && !allow_gumbo
                    || *pet == PetKind::Blip && !allow_blip
            })
            || pets.windows(2).any(|pair| {
                canonical.iter().position(|pet| *pet == pair[0])
                    >= canonical.iter().position(|pet| *pet == pair[1])
            })
        {
            return Err("invalid Tank 3 pet selection".into());
        }
        Ok(())
    }

    fn spawn_tank3_pets(&mut self, pets: &[PetKind]) {
        self.pets = pets.to_vec();
        for pet in pets {
            match pet {
                PetKind::Stinky => self.stinky = vec![self.spawn_stinky(StinkyOrigin::StageStart)],
                PetKind::Niko => {
                    let owner_id = self.id();
                    self.niko = vec![NikoState::spawn_tank3(owner_id, &mut |upper| {
                        self.rand_range(upper)
                    })];
                }
                PetKind::Itchy => self.spawn_fish_pet(FishPetKind::Itchy),
                PetKind::Prego => self.spawn_fish_pet(FishPetKind::Prego),
                PetKind::Zorf => self.spawn_fish_pet(FishPetKind::Zorf),
                PetKind::Clyde => {
                    let id = self.id();
                    let mut rng_state = self.rng_state;
                    self.clyde = vec![ClydeState::spawn_tank2(id, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    })];
                    self.rng_state = rng_state;
                }
                PetKind::Vert => self.spawn_fish_pet(FishPetKind::Vert),
                PetKind::Rufus => {
                    let id = self.id();
                    let mut rng_state = self.rng_state;
                    self.rufus = vec![RufusState::spawn_tank2(id, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    })];
                    self.rng_state = rng_state;
                }
                PetKind::Meryl => self.spawn_fish_pet(FishPetKind::Meryl),
                PetKind::Wadsworth => self.spawn_fish_pet(FishPetKind::Wadsworth),
                PetKind::Seymour => self.spawn_fish_pet(FishPetKind::Seymour),
                PetKind::Shrapnel => self.spawn_fish_pet(FishPetKind::Shrapnel),
                PetKind::Gumbo => self.spawn_fish_pet(FishPetKind::Gumbo),
                PetKind::Blip => self.spawn_fish_pet(FishPetKind::Blip),
                PetKind::Rhubarb
                | PetKind::Nimbus
                | PetKind::Amp
                | PetKind::Gash
                | PetKind::Angie
                | PetKind::Presto
                | PetKind::Brinkley
                | PetKind::Nostradamus
                | PetKind::Stanley
                | PetKind::Walter => {
                    unreachable!("pet unlock follows stage 3-5")
                }
            }
        }
    }

    fn spawn_fish_pet(&mut self, kind: FishPetKind) {
        let id = self.id();
        let mut rng_state = self.rng_state;
        let pet = if self.tank == 5 {
            FishPetState::spawn_tank5(id, kind, &mut |upper| {
                Self::advance_rng(&mut rng_state) % upper
            })
        } else {
            FishPetState::spawn_tank1(id, kind, &mut |upper| {
                Self::advance_rng(&mut rng_state) % upper
            })
        };
        self.rng_state = rng_state;
        self.fish_pets.push(pet);
    }

    /// Only invoked while migrating old format-5 project saves. Their board
    /// tick cannot recover elapsed Balrog wave phase or historical Itchy pose.
    pub fn initialize_legacy_stage14_support(&mut self) {
        if self.tank == 1 && self.level == 4 && self.invasion.is_none() {
            self.invasion = Some(Invasion1_2::legacy_v5_balrog_resume());
            if self.fish_pets.is_empty() && self.pets.contains(&PetKind::Itchy) {
                self.spawn_fish_pet(FishPetKind::Itchy);
            }
        }
    }

    /// Old format-5 dead-alien effects predate their explicit variant field.
    /// Recover only from the persisted owning wave at the migration boundary.
    pub fn initialize_legacy_alien_body_kind(&mut self) {
        if let Some(wave) = self.invasion.as_mut()
            && let Some(body) = wave.dead_aliens.first_mut()
            && let WavePlan::Fixed(kind) = wave.plan
        {
            body.kind = kind;
        }
    }

    /// Explicit migration from old format-4 project boards. Existing 1-3
    /// ticks do not reveal the historically elapsed strong-wave countdown.
    pub fn initialize_legacy_stage13_support(&mut self) {
        if self.tank == 1 && self.level == 3 && self.invasion.is_none() {
            self.invasion = Some(Invasion1_2::legacy_v4_strong_resume());
            let grew_large = self.upgrades.quality_unlocked
                || self.fish.iter().any(|fish| fish.size == FishSize::Large)
                || self
                    .dead_fish
                    .iter()
                    .any(|fish| fish.size == FishSize::Large);
            if grew_large {
                self.upgrades.quality_unlocked = true;
                self.upgrades.quantity_unlocked = true;
                self.oscar_unlocked = true;
            }
            self.weapon_strength = 2;
            self.weapon_unlocked = false;
            self.egg_unlocked = false;
        }
    }

    pub fn initialize_legacy_invasion(&mut self) {
        if (self.tank, self.level) == (1, 2) {
            self.upgrades.quality_unlocked = self.egg_unlocked
                || self.fish.iter().any(|fish| fish.size == FishSize::Large)
                || self
                    .dead_fish
                    .iter()
                    .any(|fish| fish.size == FishSize::Large);
            self.egg_unlocked = false;
            self.invasion = Some(Invasion1_2::legacy_v3_resume());
        }
    }

    /// Old v2 project saves stored the roster but no live pet object. This
    /// creates a fresh deterministic state at the load boundary; it cannot
    /// recover where Stinky historically was in that save.
    pub fn initialize_missing_stinky(&mut self) -> bool {
        if (self.tank, self.level) != (1, 2)
            || !self.pets.contains(&PetKind::Stinky)
            || !self.stinky.is_empty()
        {
            return false;
        }
        self.stinky = vec![self.spawn_stinky(StinkyOrigin::LegacyV2Resume)];
        true
    }

    fn spawn_stinky(&mut self, origin: StinkyOrigin) -> StinkyState {
        let combat_id = if self.tank == 5 {
            Some(self.id())
        } else {
            None
        };
        let x = self.rand_range(265) as f64 + 105.0;
        let _unused_y = self.rand_range(520) + 20;
        StinkyState {
            combat_id,
            presto_form: None,
            x,
            // PB05 004eb5c0 fixes unflagged raw0 to00596ed0=370,
            // independently of both application mode and Board tank.
            y: 370.0,
            vx: 0.0,
            vy: 0.0,
            target_vx: 0.0,
            previous_vx: 1.0,
            frame: 0,
            movement_state: self.rand_range(10) as u8,
            movement_state_change_timer: 0,
            chase_timer: 40,
            movement_animation_timer: 0,
            turn_animation_timer: 0,
            specialty_timer: 0,
            angry_timer: 0,
            random_timer: self.rand_range(250) as u16 + 250,
            origin,
        }
    }

    /// PB05 flagged raw0 preserves the old widget pose and consumes only the
    /// two common constructor draws; the ordinary coordinate draws stay above.
    fn spawn_stinky_form_at(
        id: u64,
        x: i32,
        y: i32,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> StinkyState {
        StinkyState {
            combat_id: Some(id),
            presto_form: Some(PrestoForm {
                remaining_ticks: 360,
            }),
            x: f64::from(x),
            y: f64::from(y),
            vx: 0.0,
            vy: 0.0,
            target_vx: 0.0,
            previous_vx: 1.0,
            frame: 0,
            movement_state: rand_range(10) as u8,
            movement_state_change_timer: 0,
            chase_timer: 40,
            movement_animation_timer: 0,
            turn_animation_timer: 0,
            specialty_timer: 0,
            angry_timer: 0,
            random_timer: rand_range(250) as u16 + 250,
            origin: StinkyOrigin::StageStart,
        }
    }

    pub(crate) fn transition_seed(&self) -> u64 {
        self.rng_state
    }

    pub(crate) fn has_live_fish(&self) -> bool {
        self.fish.iter().any(|fish| fish.alive)
            || self.oscars.iter().any(|oscar| oscar.alive)
            || self.starcatchers.iter().any(|actor| actor.alive)
            || self.grubbers.iter().any(|actor| actor.alive)
            || self.gekkos.iter().any(|actor| actor.alive)
            || self.breeders.iter().any(|actor| actor.alive)
            || self.ultras.iter().any(|actor| actor.alive)
    }

    pub(crate) fn has_live_pets(&self) -> bool {
        !self.stinky.is_empty()
            || !self.niko.is_empty()
            || !self.clyde.is_empty()
            || !self.rufus.is_empty()
            || !self.rhubarb.is_empty()
            || !self.fish_pets.is_empty()
    }

    fn living_finale_boss(&self) -> bool {
        self.invasion
            .as_ref()
            .and_then(|wave| wave.finale.as_ref())
            .is_some_and(|finale| finale.boss.is_some())
    }

    fn tank5_pet_prey(&self) -> Vec<PreyView> {
        if self.tank != 5 {
            return Vec::new();
        }
        // PB05 004e8b60 walks registered objects and retains the first tie.
        // Reconstruct that initial registration order from the fixed roster;
        // live membership, rather than the profile flags, determines inclusion.
        TANK5_PETS
            .iter()
            .flat_map(|kind| {
                let members: Vec<(u64, i32, i32)> = match kind {
                    PetKind::Stinky => self
                        .stinky
                        .iter()
                        .filter_map(|pet| Some((pet.combat_id?, pet.x as i32, pet.y as i32)))
                        .collect(),
                    PetKind::Niko => self
                        .niko
                        .iter()
                        .map(|pet| (pet.owner_id, pet.anchor_x, pet.anchor_y))
                        .collect(),
                    PetKind::Clyde => self
                        .clyde
                        .iter()
                        .map(|pet| (pet.id, pet.widget_x, pet.widget_y))
                        .collect(),
                    PetKind::Rufus => self
                        .rufus
                        .iter()
                        .map(|pet| (pet.id, pet.widget_x, pet.widget_y))
                        .collect(),
                    PetKind::Rhubarb => self
                        .rhubarb
                        .iter()
                        .map(|pet| (pet.id, pet.widget_x, pet.widget_y))
                        .collect(),
                    kind => {
                        let fish_kind = match kind {
                            PetKind::Itchy => Some(FishPetKind::Itchy),
                            PetKind::Prego => Some(FishPetKind::Prego),
                            PetKind::Zorf => Some(FishPetKind::Zorf),
                            PetKind::Vert => Some(FishPetKind::Vert),
                            PetKind::Meryl => Some(FishPetKind::Meryl),
                            PetKind::Wadsworth => Some(FishPetKind::Wadsworth),
                            PetKind::Seymour => Some(FishPetKind::Seymour),
                            PetKind::Shrapnel => Some(FishPetKind::Shrapnel),
                            PetKind::Gumbo => Some(FishPetKind::Gumbo),
                            PetKind::Blip => Some(FishPetKind::Blip),
                            PetKind::Nimbus => Some(FishPetKind::Nimbus),
                            PetKind::Amp => Some(FishPetKind::Amp),
                            PetKind::Gash => Some(FishPetKind::Gash),
                            _ => None,
                        };
                        self.fish_pets
                            .iter()
                            .filter(|pet| Some(pet.kind) == fish_kind)
                            .map(|pet| (pet.id, pet.widget_x, pet.widget_y))
                            .collect()
                    }
                };
                members
                    .into_iter()
                    .map(|(id, widget_x, widget_y)| PreyView {
                        id,
                        widget_x,
                        widget_y,
                        width: 80,
                        height: 80,
                        eligible: self.living_finale_boss(),
                    })
            })
            .collect()
    }

    fn remove_tank5_pet(&mut self, prey_id: u64) {
        if self.tank != 5 {
            return;
        }
        self.stinky.retain(|pet| pet.combat_id != Some(prey_id));
        self.niko.retain(|pet| pet.owner_id != prey_id);
        self.clyde.retain(|pet| pet.id != prey_id);
        self.rufus.retain(|pet| pet.id != prey_id);
        self.rhubarb.retain(|pet| pet.id != prey_id);
        self.fish_pets.retain(|pet| pet.id != prey_id);
        if !self.has_live_pets() {
            self.egg_unlocked = false;
        }
    }

    /// Rendering tests the fish's current double position against the
    /// Wadsworth coordinates published from the pet's prior widget pose.
    pub fn hides_fish_at(&self, x: f32, y: f32) -> bool {
        self.fish_pets.iter().any(|pet| {
            pet.kind == FishPetKind::Wadsworth
                && pet.ward_active
                && (f64::from(x) - f64::from(pet.published_x)).abs() < 8.0
                && (f64::from(y) - f64::from(pet.published_y)).abs() < 8.0
        })
    }

    fn ward_snapshot(&self) -> Option<(i32, i32)> {
        self.fish_pets
            .iter()
            .find(|pet| pet.kind == FishPetKind::Wadsworth && pet.ward_active)
            .map(|pet| (pet.published_x, pet.published_y))
    }

    pub fn has_live_blip(&self) -> bool {
        self.fish_pets
            .iter()
            .any(|pet| pet.kind == FishPetKind::Blip)
    }

    /// PB53's normal Adventure information icon. The source's separate
    /// times-fed-today field remains zero in ordinary Adventure; the growth
    /// food counter does not gate this icon.
    pub fn blip_hunger_icon_visible(&self, fish: &Fish) -> bool {
        self.has_live_blip() && fish.alive && fish.hunger < 500
    }

    pub fn blip_breeder_icon_visible(&self, breeder: &BreederState) -> bool {
        self.has_live_blip() && breeder.alive && breeder.hunger < 500
    }

    pub fn blip_ultra_icon_visible(&self, ultra: &UltraState) -> bool {
        self.has_live_blip() && ultra.alive && ultra.hunger < 500
    }

    /// Final draw coordinates and frame, from W1 Board::DrawOverlay.
    /// Its caller/conditions remain secondary-source-derived.
    pub fn blip_crosshair(&self) -> Option<(i32, i32, u8)> {
        if !self.has_live_blip() {
            return None;
        }
        let wave = self.invasion.as_ref()?;
        if !(1..=225).contains(&wave.countdown) {
            return None;
        }
        let coords = wave.warning?;
        Some((coords.first_x + 40, 320, ((self.tick / 4) % 5) as u8))
    }

    /// Explicit source draw-dependent input, observed before frame publication.
    /// Drawing remains read-only; this changes no clock, membership or RNG.
    pub fn observe_rendered_bilaterus_connectors(&mut self) -> bool {
        let Some(wave) = self.invasion.as_mut() else {
            return false;
        };
        let mut changed = false;
        for group in &mut wave.bilaterus {
            changed |= group.observe_rendered_connectors();
        }
        changed
    }

    /// Board::Buy counts coins already flying to the money display as
    /// available, while the purchase still subtracts from raw balance.
    /// Confirmed by W1 Board::Buy and payload FUN_00540b30/FUN_0053a0b0.
    pub fn available_funds(&self) -> i32 {
        let coin_funds = self
            .coins
            .iter()
            .filter(|coin| coin.collecting)
            .fold(self.balance, |funds, coin| {
                funds.saturating_add(coin.kind.value())
            });
        let pearl_funds = self
            .pearls
            .iter()
            .filter(|pearl| pearl.phase == PearlPhase::Collecting)
            .fold(coin_funds, |funds, _| funds.saturating_add(PEARL_VALUE));
        self.larvae
            .iter()
            .filter(|larva| larva.picked_up)
            .fold(pearl_funds, |funds, _| funds.saturating_add(LARVA_VALUE))
    }

    fn validate_tank5(&self) -> Result<(), String> {
        const FISH_PETS: [FishPetKind; 13] = [
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
        ];
        let Some(wave) = self.invasion.as_ref() else {
            return Err("Tank 5 finale wave missing".into());
        };
        let Some(finale) = wave.finale.as_ref() else {
            return Err("Tank 5 finale state missing".into());
        };
        if wave.plan != WavePlan::Tank5Finale
            || (!self.profile_population && self.pets.as_slice() != TANK5_PETS.as_slice())
            || (!self.profile_population
                && self.fish_pets.iter().any(|pet| !FISH_PETS.contains(&pet.kind)))
            || (!self.profile_population && self.fish_pets.windows(2).any(|pair| {
                FISH_PETS.iter().position(|kind| *kind == pair[0].kind)
                    >= FISH_PETS.iter().position(|kind| *kind == pair[1].kind)
            }))
            || !(2..=3).contains(&self.eggs)
            || self.guppy_unlocked
            || self.oscar_unlocked
            || self.starcatcher_unlocked
            || self.grubber_unlocked
            || self.gekko_unlocked
            || self.breeder_unlocked
            || self.ultra_unlocked
            || self.weapon_unlocked
            || self.potion_unlocked
            || self.potion_armed
            || self.upgrades.quality_unlocked
            || self.upgrades.quantity_unlocked
            || self.upgrades.quality != 0
            || self.upgrades.quantity != 1
            || !self.fish.is_empty()
            || !self.dead_fish.is_empty()
            || !self.oscars.is_empty()
            || !self.dead_oscars.is_empty()
            || !self.starcatchers.is_empty()
            || !self.dead_starcatchers.is_empty()
            || !self.grubbers.is_empty()
            || !self.dead_grubbers.is_empty()
            || !self.gekkos.is_empty()
            || !self.dead_gekkos.is_empty()
            || !self.breeders.is_empty()
            || !self.dead_breeders.is_empty()
            || !self.ultras.is_empty()
            || !self.dead_ultras.is_empty()
            || !self.larvae.is_empty()
            || !self.food.is_empty()
            // A Bilaterus retained through boss death can still drop its
            // ordinary diamond; the live boss suppresses subsidiary drops.
            || self
                .coins
                .iter()
                .any(|coin| coin.kind != CoinKind::Diamond || !finale.boss_defeated)
            || !self.notes.is_empty()
            || !self.bomb_shots.is_empty()
            || self.weapon_strength
                != if finale.boss.is_some() || finale.boss_defeated {
                    10
                } else {
                    2
                }
            || self.egg_unlocked
                && (!finale.boss_defeated || finale.boss.is_some() || !self.has_live_pets())
        {
            return Err("Tank 5 roster, shop, or finale gate disagrees".into());
        }
        Ok(())
    }

    pub(crate) fn replay_bonus_identity(&self) -> (u64, u64) {
        (self.rng_state, self.next_id)
    }

    pub(crate) fn begin_replay_bonus(&mut self) {
        self.bonus_active = true;
        self.bonus_tally = 0;
        self.pets.clear();
        self.stinky.clear();
        self.niko.clear();
        self.clyde.clear();
        self.rufus.clear();
        self.rhubarb.clear();
        self.fish_pets.clear();
        self.fish.clear();
        self.dead_fish.clear();
        self.oscars.clear();
        self.dead_oscars.clear();
        self.starcatchers.clear();
        self.dead_starcatchers.clear();
        self.grubbers.clear();
        self.dead_grubbers.clear();
        self.gekkos.clear();
        self.dead_gekkos.clear();
        self.breeders.clear();
        self.dead_breeders.clear();
        self.ultras.clear();
        self.dead_ultras.clear();
        self.larvae.clear();
        self.food.clear();
        self.coins.clear();
        self.pearls.clear();
        self.missiles.clear();
        self.bomb_shots.clear();
        self.notes.clear();
        self.invasion = None;
        self.held_feed = None;
        self.held_fire = None;
    }

    pub(crate) fn finish_replay_bonus(&mut self, seed: u64, next_id: u64) {
        self.rng_state = seed;
        self.next_id = next_id;
    }

    fn validate_replay_bonus(&self) -> Result<(), String> {
        if !self.profile_population
            || !self.bonus_active
            || self.time_trial
            || !matches!((self.tank, self.level), (1..=4, 1..=5))
            || !self.victory
            || self.eggs != 3
            || Self::profile_egg_price(self.tank, self.level) != Some(self.egg_price)
            || self.profile_shop_invalid()
            || self.upgrades.quality > 2
            || !(1..=9).contains(&self.upgrades.quantity)
            || (!self.upgrades.quality_unlocked && self.upgrades.quality > 0)
            || (!self.upgrades.quantity_unlocked && self.upgrades.quantity > 1)
            || !(2..=12).contains(&self.weapon_strength)
            || (self.potion_armed && !self.potion_unlocked)
            || self.bonus_tally > crate::bonus::MAX_SHELL_BALANCE
            || self.next_id == 0
            || self.next_id == u64::MAX
            || self.rng_state == 0
            || !self.pets.is_empty()
            || !self.stinky.is_empty()
            || !self.niko.is_empty()
            || !self.clyde.is_empty()
            || !self.rufus.is_empty()
            || !self.rhubarb.is_empty()
            || !self.fish_pets.is_empty()
            || !self.fish.is_empty()
            || !self.dead_fish.is_empty()
            || !self.oscars.is_empty()
            || !self.dead_oscars.is_empty()
            || !self.starcatchers.is_empty()
            || !self.dead_starcatchers.is_empty()
            || !self.grubbers.is_empty()
            || !self.dead_grubbers.is_empty()
            || !self.gekkos.is_empty()
            || !self.dead_gekkos.is_empty()
            || !self.breeders.is_empty()
            || !self.dead_breeders.is_empty()
            || !self.ultras.is_empty()
            || !self.dead_ultras.is_empty()
            || !self.larvae.is_empty()
            || !self.food.is_empty()
            || !self.coins.is_empty()
            || !self.pearls.is_empty()
            || !self.missiles.is_empty()
            || !self.bomb_shots.is_empty()
            || !self.notes.is_empty()
            || self.invasion.is_some()
        {
            return Err(
                "replay bonus Board retains registered play actors or invalid state".into(),
            );
        }
        Ok(())
    }

    fn validate_profile_population(&self) -> Result<(), String> {
        let (selected, fixed) = if (self.tank, self.level) == (5, 1) {
            let count = self
                .pets
                .len()
                .checked_sub(TANK5_PETS.len())
                .ok_or("Tank 5 selected roster is short")?;
            self.pets.split_at(count)
        } else {
            (self.pets.as_slice(), &[][..])
        };
        if selected.len() > 3
            || selected
                .iter()
                .any(|pet| !crate::time_trial::selectable_pet(*pet))
            || selected
                .windows(2)
                .any(|pair| pair[0] as u8 >= pair[1] as u8)
            || fixed
                != if self.tank == 5 {
                    TANK5_PETS.as_slice()
                } else {
                    &[]
                }
        {
            return Err("invalid profile-selected Adventure roster".into());
        }
        let wave = self.invasion.as_ref().map(|wave| wave.plan);
        let expected_wave = matches!(
            (self.tank, self.level, wave),
            (1, 1, None)
                | (1, 2, Some(WavePlan::Fixed(SylvesterKind::Weak)))
                | (1, 3, Some(WavePlan::Fixed(SylvesterKind::Strong)))
                | (1, 4, Some(WavePlan::Fixed(SylvesterKind::Balrog)))
                | (1, 5, Some(WavePlan::CyclingTank1Finale { .. }))
                | (2, 1, Some(WavePlan::Fixed(SylvesterKind::Strong)))
                | (2, 2, Some(WavePlan::Fixed(SylvesterKind::Balrog)))
                | (2, 3, Some(WavePlan::Fixed(SylvesterKind::Gus)))
                | (2, 4, Some(WavePlan::Fixed(SylvesterKind::Destructor)))
                | (2, 5, Some(WavePlan::CyclingTank2Finale { .. }))
                | (3, 1, Some(WavePlan::Fixed(SylvesterKind::Balrog)))
                | (3, 2, Some(WavePlan::CyclingTank3Second { .. }))
                | (3, 3, Some(WavePlan::Fixed(SylvesterKind::Psychosquid)))
                | (3, 4, Some(WavePlan::Fixed(SylvesterKind::Ulysses)))
                | (3, 5, Some(WavePlan::CyclingTank3Finale { .. }))
                | (4, 1, Some(WavePlan::Fixed(SylvesterKind::Balrog)))
                | (4, 2, Some(WavePlan::FixedBilaterus))
                | (4, 3, Some(WavePlan::CyclingTank4Third { .. }))
                | (4, 4, Some(WavePlan::CyclingTank4Fourth { .. }))
                | (4, 5, Some(WavePlan::CyclingTank4Finale { .. }))
                | (5, 1, Some(WavePlan::Tank5Finale))
        );
        if !expected_wave {
            return Err("profile Adventure wave disagrees with stage".into());
        }
        if self.profile_shop_invalid() {
            return Err("profile Adventure shop disagrees with stage".into());
        }
        let mut ordinary = Vec::new();
        let mut flagged = 0;
        for (kind, form) in self
            .stinky
            .iter()
            .map(|pet| (PetKind::Stinky, pet.presto_form))
            .chain(self.niko.iter().map(|pet| (PetKind::Niko, pet.presto_form)))
            .chain(
                self.clyde
                    .iter()
                    .map(|pet| (PetKind::Clyde, pet.presto_form)),
            )
            .chain(
                self.rufus
                    .iter()
                    .map(|pet| (PetKind::Rufus, pet.presto_form)),
            )
            .chain(
                self.rhubarb
                    .iter()
                    .map(|pet| (PetKind::Rhubarb, pet.presto_form)),
            )
            .chain(
                self.fish_pets
                    .iter()
                    .filter(|pet| !pet.nostra_converted)
                    .map(|pet| (Self::pet_kind_from_fish(pet.kind), pet.presto_form)),
            )
        {
            if form.is_some() {
                flagged += 1;
            } else {
                ordinary.push(kind);
            }
        }
        let mut expected: Vec<_> = self
            .pets
            .iter()
            .copied()
            .filter(|pet| *pet != PetKind::Presto)
            .collect();
        ordinary.sort_by_key(|pet| *pet as u8);
        expected.sort_by_key(|pet| *pet as u8);
        if flagged != usize::from(selected.contains(&PetKind::Presto)) || ordinary != expected {
            return Err("profile physical pets disagree with logical roster".into());
        }
        Ok(())
    }

    fn profile_shop_invalid(&self) -> bool {
        let quality = self.upgrades.quality_unlocked;
        let quantity = self.upgrades.quantity_unlocked;
        match (self.tank, self.level) {
            (1, 1) => quality || quantity || self.weapon_unlocked || self.oscar_unlocked,
            (1, 2) => self.weapon_unlocked || self.oscar_unlocked || self.weapon_strength != 2,
            (1, 3..=5) => {
                quality != quantity
                    || quality != self.oscar_unlocked
                    || self.egg_unlocked && !self.oscar_unlocked
            }
            (2, 1) => {
                self.oscar_unlocked
                    || self.weapon_unlocked
                    || self.weapon_strength != 2
                    || quality != quantity
                    || quality != self.potion_unlocked
                    || quality != self.egg_unlocked
            }
            (2, 2..=5) => {
                self.oscar_unlocked
                    || quality != quantity
                    || quality != self.potion_unlocked
                    || quality != self.starcatcher_unlocked
                    || self.egg_unlocked != self.weapon_unlocked
                    || self.egg_unlocked && !self.starcatcher_unlocked
            }
            (3, 1) => {
                self.oscar_unlocked
                    || self.weapon_unlocked
                    || self.weapon_strength != 2
                    || quality != quantity
                    || quality != self.grubber_unlocked
                    || self.egg_unlocked && !self.grubber_unlocked
            }
            (3, 2..=5) => {
                self.oscar_unlocked
                    || quality != quantity
                    || quality != self.grubber_unlocked
                    || self.egg_unlocked != self.weapon_unlocked
                    || self.egg_unlocked && !self.gekko_unlocked
                    || self.gekko_unlocked && !self.grubber_unlocked
            }
            (4, 1..=5) => {
                self.guppy_unlocked
                    || quality != quantity
                    || quality != self.oscar_unlocked
                    || self.egg_unlocked && !self.oscar_unlocked
                    || if self.level == 1 {
                        self.weapon_unlocked || self.weapon_strength != 2 || self.ultra_unlocked
                    } else {
                        self.weapon_unlocked != self.ultra_unlocked
                            || self.ultra_unlocked != self.egg_unlocked
                    }
            }
            (5, 1) => false,
            _ => unreachable!("profile stage checked above"),
        }
    }

    fn validate_time_trial_header(&self) -> Result<(), String> {
        if !(1..=4).contains(&self.tank)
            || self.profile_population
            || self.level != 5
            || self.next_id == 0
            || self.next_id == u64::MAX
            || self.rng_state == 0
            || self.eggs != 0
            || self.victory
            || self.balance < 0
            || !self.egg_unlocked
            || self.egg_price < crate::time_trial::TIME_TRIAL_EGG_PRICE
            || self.egg_price > crate::time_trial::TIME_TRIAL_MAX_EGG_PRICE
            || self
                .pets
                .iter()
                .any(|pet| !crate::time_trial::selectable_pet(*pet))
            || self.stinky.len() > 2
            || self.niko.len() > 2
            || self.clyde.len() > 2
            || self.rufus.len() > 2
            || self.rhubarb.len() > 2
            || self.upgrades.quality > 2
            || !(1..=9).contains(&self.upgrades.quantity)
            || (!self.upgrades.quality_unlocked && self.upgrades.quality > 0)
            || (!self.upgrades.quantity_unlocked && self.upgrades.quantity > 1)
            || self
                .food
                .iter()
                .filter(|food| food.direction == 0 && food.food_type == FoodType::Ordinary)
                .count()
                > usize::from(self.upgrades.quantity)
            || self.food.iter().any(|food| {
                food.quality > 3
                    || (food.picked_up && food.food_type != FoodType::Nostradamus)
                    || (food.food_type == FoodType::Nostradamus
                        && (food.quality != 0
                            || food.direction != 0
                            || food.free_from_zorf
                            || food.nimbus_rising))
                    || food.direction > 2
                    || !(3..=4).contains(&food.animation_period)
                    || !food.x.is_finite()
                    || !food.y.is_finite()
                    || !food.vx.is_finite()
                    || !food.vy.is_finite()
                    || (food.free_from_zorf
                        && (food.quality != 1
                            || food.direction == 0
                            || !(self.pets.contains(&PetKind::Zorf)
                                || self.pets.contains(&PetKind::Presto))))
                    || (!food.free_from_zorf && food.direction != 0)
                    || (food.nimbus_rising
                        && (!(self.pets.contains(&PetKind::Nimbus)
                            || self.pets.contains(&PetKind::Presto))
                            || food.direction != 0))
                    || (food.quality == 3 && (self.tank != 2 || !self.potion_unlocked))
            })
            || !(2..=12).contains(&self.weapon_strength)
            || self.punch_sound_cooldown > 11
            || (self.potion_armed && !self.potion_unlocked)
            || self.coins.iter().any(|coin| {
                (coin.penta_rising && coin.kind != CoinKind::DiamondPenta && !coin.kind.is_shell())
                    || coin.animation_ticks > 79
                    || coin.frame > 9
                    || !coin.x.is_finite()
                    || !coin.y.is_finite()
                    || (coin.kind == CoinKind::ShrapnelBomb
                        && (!(self.pets.contains(&PetKind::Shrapnel)
                            || self.pets.contains(&PetKind::Presto))
                            || coin.fade_ticks != 0))
                    || (coin.kind.is_shell()
                        && !(self.pets.contains(&PetKind::Nimbus)
                            || self.pets.contains(&PetKind::Presto)))
                    || (coin.kind == CoinKind::Treasure
                        && self.tank != 4
                        && !(self.pets.contains(&PetKind::Brinkley)
                            || self.pets.contains(&PetKind::Presto)))
                    || (coin.kind == CoinKind::DiamondPenta && self.tank != 2)
            })
            || self.invasion.as_ref().is_none_or(
                |wave| !matches!(wave.plan, WavePlan::TimeTrial { tank, .. } if tank == self.tank),
            )
        {
            return Err("invalid Time Trial board counters or roster".into());
        }
        let mut plain_kinds = Vec::new();
        let mut flagged = 0;
        for (kind, forms) in [
            (
                PetKind::Stinky,
                self.stinky
                    .iter()
                    .map(|pet| pet.presto_form)
                    .collect::<Vec<_>>(),
            ),
            (
                PetKind::Niko,
                self.niko.iter().map(|pet| pet.presto_form).collect(),
            ),
            (
                PetKind::Clyde,
                self.clyde.iter().map(|pet| pet.presto_form).collect(),
            ),
            (
                PetKind::Rufus,
                self.rufus.iter().map(|pet| pet.presto_form).collect(),
            ),
            (
                PetKind::Rhubarb,
                self.rhubarb.iter().map(|pet| pet.presto_form).collect(),
            ),
        ] {
            for form in forms {
                if form.is_some() {
                    flagged += 1;
                } else {
                    plain_kinds.push(kind);
                }
            }
        }
        for pet in &self.fish_pets {
            let kind = match pet.kind {
                FishPetKind::Itchy => PetKind::Itchy,
                FishPetKind::Prego => PetKind::Prego,
                FishPetKind::Zorf => PetKind::Zorf,
                FishPetKind::Vert => PetKind::Vert,
                FishPetKind::Meryl => PetKind::Meryl,
                FishPetKind::Wadsworth => PetKind::Wadsworth,
                FishPetKind::Seymour => PetKind::Seymour,
                FishPetKind::Shrapnel => PetKind::Shrapnel,
                FishPetKind::Gumbo => PetKind::Gumbo,
                FishPetKind::Blip => PetKind::Blip,
                FishPetKind::Nimbus => PetKind::Nimbus,
                FishPetKind::Amp => PetKind::Amp,
                FishPetKind::Gash => PetKind::Gash,
                FishPetKind::Angie => PetKind::Angie,
                FishPetKind::Presto => PetKind::Presto,
                FishPetKind::Brinkley => PetKind::Brinkley,
                FishPetKind::Nostradamus => PetKind::Nostradamus,
                FishPetKind::Stanley => PetKind::Stanley,
                FishPetKind::Walter => PetKind::Walter,
            };
            if pet.nostra_converted {
                continue;
            }
            if pet.presto_form.is_some() {
                flagged += 1;
            } else {
                plain_kinds.push(kind);
            }
        }
        if flagged != usize::from(self.pets.contains(&PetKind::Presto))
            || plain_kinds
                .iter()
                .any(|kind| *kind == PetKind::Presto || !self.pets.contains(kind))
            || plain_kinds.len() != self.pets.len() - flagged
            || plain_kinds
                .iter()
                .enumerate()
                .any(|(i, kind)| plain_kinds[..i].contains(kind))
            || self
                .notes
                .iter()
                .any(|note| note.age_ticks > 100 || note.y < 0 || note.y > 550)
            || (!self.notes.is_empty()
                && !self.pets.contains(&PetKind::Meryl)
                && !self.pets.contains(&PetKind::Presto))
        {
            return Err("Time Trial physical pets or notes disagree".into());
        }
        Ok(())
    }

    /// Validate durable board relationships before accepting a project save.
    /// The profile and screen phase are checked by AdventureSession separately.
    pub fn validate(&self) -> Result<(), String> {
        if self.fish.iter().any(|fish| !fish.walter.validate()) {
            return Err("Fish Walter impact state exceeds bounds".into());
        }
        if self.fish_pets.iter().any(|pet| {
            pet.kind == FishPetKind::Nostradamus && !(300..=599).contains(&pet.nostra_threshold)
        }) {
            return Err("Board Nostradamus uses a Virtual Tank threshold".into());
        }
        if self.food.iter().any(|food| {
            food.food_type == FoodType::Nostradamus
                && (food.ineligible_ticks > 20 || food.removal_ticks > 15)
        }) {
            return Err("Nostradamus Food clock exceeds its producer bounds".into());
        }
        if (self
            .food
            .iter()
            .any(|food| food.food_type == FoodType::Nostradamus)
            || self.fish_pets.iter().any(|pet| pet.nostra_converted))
            && (self.tank == 5
                || !self.pets.contains(&PetKind::Nostradamus)
                    && !self.pets.contains(&PetKind::Presto))
        {
            return Err("Nostradamus child state lacks a producing roster".into());
        }
        if self.bonus_active {
            return self.validate_replay_bonus();
        }
        if self.bonus_tally != 0 {
            return Err("inactive Adventure Board retains bonus tally".into());
        }
        let tank5 = (self.tank, self.level) == (5, 1);
        if !self.time_trial
            && !self.profile_population
            && (self.clyde.iter().any(|pet| pet.presto_form.is_some())
                || self.niko.iter().any(|pet| pet.presto_form.is_some())
                || self.rufus.iter().any(|pet| pet.presto_form.is_some())
                || self.rhubarb.iter().any(|pet| pet.presto_form.is_some()))
        {
            return Err("flagged Presto forms require a logical Presto board".into());
        }
        if self.time_trial {
            self.validate_time_trial_header()?;
        } else {
            if !((self.tank == 1 && (1..=5).contains(&self.level))
                || (self.tank == 2 && (1..=5).contains(&self.level))
                || (self.tank == 3 && (1..=5).contains(&self.level))
                || matches!((self.tank, self.level), (4, 1..=5) | (5, 1)))
                || self.next_id == 0
                || self.next_id == u64::MAX
                || self.rng_state == 0
                || (!self.profile_population
                    && self.fish_pets.iter().any(|pet| pet.presto_form.is_some()))
                || (!self.profile_population
                    && (self.stinky.len() > 1
                        || self.niko.len() > 1
                        || self.clyde.len() > 1
                        || self.rufus.len() > 1
                        || self.rhubarb.len() > 1))
                || self.eggs > 3
                || self.victory != (self.eggs == 3)
                || self.upgrades.quality > 2
                || !(1..=9).contains(&self.upgrades.quantity)
                || (!self.upgrades.quality_unlocked && self.upgrades.quality > 0)
                || (!self.upgrades.quantity_unlocked && self.upgrades.quantity > 1)
                || self
                    .food
                    .iter()
                    .filter(|food| food.direction == 0 && food.food_type == FoodType::Ordinary)
                    .count()
                    > usize::from(self.upgrades.quantity)
                || self.food.iter().any(|food| {
                    food.quality > 3
                        || (food.picked_up && food.food_type != FoodType::Nostradamus)
                        || (food.food_type == FoodType::Nostradamus
                            && (food.quality != 0
                                || food.direction != 0
                                || food.free_from_zorf
                                || food.nimbus_rising))
                        || food.direction > 2
                        || !(3..=4).contains(&food.animation_period)
                        || !food.x.is_finite()
                        || !food.y.is_finite()
                        || !food.vx.is_finite()
                        || !food.vy.is_finite()
                        || (food.free_from_zorf
                            && (!(2..=4).contains(&self.tank) && !self.profile_population
                                || !(self.pets.contains(&PetKind::Zorf)
                                    || self.profile_population
                                        && self.pets.contains(&PetKind::Presto))
                                || food.quality != 1
                                || food.direction == 0))
                        || (!food.free_from_zorf && food.direction != 0)
                        || (food.nimbus_rising
                            && (!matches!((self.tank, self.level), (4, 2..=5))
                                && !self.profile_population
                                || self.profile_population
                                    && !(self.pets.contains(&PetKind::Nimbus)
                                        || self.pets.contains(&PetKind::Presto))
                                || food.direction != 0
                                || food.free_from_zorf))
                        || (food.quality == 3 && (self.tank != 2 || !self.potion_unlocked))
                })
                || !(2..=12).contains(&self.weapon_strength)
                || (!tank5 && !self.weapon_unlocked && self.weapon_strength > 2)
                || self.punch_sound_cooldown > 11
                || (!self.profile_population
                    && !matches!((self.tank, self.level), (4, 3..=5) | (5, 1))
                    && self.pets.contains(&PetKind::Amp))
                || (!self.profile_population
                    && !matches!((self.tank, self.level), (4, 4..=5) | (5, 1))
                    && self.pets.contains(&PetKind::Gash))
                || (!self.profile_population
                    && !matches!((self.tank, self.level), (4, 5) | (5, 1))
                    && self.pets.contains(&PetKind::Angie))
                || (!self.profile_population
                    && !matches!((self.tank, self.level), (4, 2..=5) | (5, 1))
                    && self.pets.contains(&PetKind::Nimbus))
                || (!matches!((self.tank, self.level), (4, 2..=5))
                    && (self.ultra_unlocked
                        || !self.ultras.is_empty()
                        || !self.dead_ultras.is_empty()))
                || (!self.profile_population
                    && !matches!((self.tank, self.level), (4, 1..=5) | (5, 1))
                    && self.pets.contains(&PetKind::Rhubarb))
                || !self.profile_population
                    && !matches!((self.tank, self.level), (3, 5) | (4, 1..=5) | (5, 1))
                    && self.pets.contains(&PetKind::Blip)
                || !self.profile_population
                    && !matches!((self.tank, self.level), (3, 4..=5) | (4, 1..=5) | (5, 1))
                    && self.pets.contains(&PetKind::Gumbo)
                || !self.profile_population
                    && !matches!((self.tank, self.level), (3, 3..=5) | (4, 1..=5) | (5, 1))
                    && self.pets.contains(&PetKind::Shrapnel)
                || !self.profile_population
                    && !matches!((self.tank, self.level), (3, 2..=5) | (4, 1..=5) | (5, 1))
                    && self.pets.contains(&PetKind::Seymour)
                || !self.profile_population
                    && !matches!(self.tank, 3..=5)
                    && self.pets.contains(&PetKind::Wadsworth)
                || (!matches!((self.tank, self.level), (4, 1..=5) | (5, 1))
                    && (self.breeder_unlocked
                        || !self.breeders.is_empty()
                        || !self.dead_breeders.is_empty()
                        || (!self.profile_population && !self.rhubarb.is_empty())))
                || (self.tank != 3
                    && (self.grubber_unlocked
                        || !self.grubbers.is_empty()
                        || !self.dead_grubbers.is_empty()
                        || !self.larvae.is_empty()))
                || !matches!((self.tank, self.level), (3, 2..=5))
                    && (self.gekko_unlocked
                        || !self.gekkos.is_empty()
                        || !self.dead_gekkos.is_empty())
                || (self.tank != 2 || !(2..=5).contains(&self.level))
                    && (self.starcatcher_unlocked
                        || !self.starcatchers.is_empty()
                        || !self.dead_starcatchers.is_empty())
                || (!self.profile_population
                    && !self.clyde.is_empty()
                    && !matches!(
                        (self.tank, self.level),
                        (2, 2..=5) | (3, 1..=5) | (4, 1..=5) | (5, 1)
                    ))
                || self.coins.iter().any(|coin| {
                    (coin.penta_rising
                        && coin.kind != CoinKind::DiamondPenta
                        && !coin.kind.is_shell())
                        || coin.animation_ticks > 79
                        || coin.frame > 9
                        || !coin.x.is_finite()
                        || !coin.y.is_finite()
                        || (coin.kind == CoinKind::Pearl
                            && !(self.tank == 3 && (2..=5).contains(&self.level))
                            && !(self.profile_population
                                && (self.pets.contains(&PetKind::Niko)
                                    || self.pets.contains(&PetKind::Presto)
                                    || self.pets.contains(&PetKind::Brinkley))))
                        || (coin.kind == CoinKind::ShrapnelBomb
                            && (!self.profile_population
                                && !matches!((self.tank, self.level), (3, 3..=5) | (4, 1..=5))
                                || !(self.pets.contains(&PetKind::Shrapnel)
                                    || self.profile_population
                                        && self.pets.contains(&PetKind::Presto))
                                || coin.fade_ticks != 0))
                        || (coin.kind.is_shell()
                            && !matches!((self.tank, self.level), (4, 2..=5))
                            && !self.profile_population)
                        || (coin.kind.is_shell()
                            && self.profile_population
                            && !(self.pets.contains(&PetKind::Nimbus)
                                || self.pets.contains(&PetKind::Presto)))
                        || (coin.kind == CoinKind::Treasure
                            && !matches!((self.tank, self.level), (4, 2..=5))
                            && !(self.pets.contains(&PetKind::Brinkley)
                                || self.pets.contains(&PetKind::Presto)))
                        || (coin.kind == CoinKind::DiamondPenta
                            && !(self.tank == 2 && (2..=5).contains(&self.level)))
                })
                || (self.potion_armed && !self.potion_unlocked)
                || (self.tank != 2
                    && (self.potion_unlocked
                        || self.potion_armed
                        || self.food.iter().any(|food| food.quality == 3)))
            {
                return Err("invalid Adventure board counters or upgrades".into());
            }
            let expected_price = if tank5 {
                0
            } else if self.tank == 4 {
                match self.level {
                    1 => TANK4_FIRST_EGG_PRICE,
                    2 => TANK4_SECOND_EGG_PRICE,
                    3 => TANK4_THIRD_EGG_PRICE,
                    4 => TANK4_FOURTH_EGG_PRICE,
                    5 => TANK4_FINALE_EGG_PRICE,
                    _ => unreachable!(),
                }
            } else if self.tank == 3 {
                match self.level {
                    1 => TANK3_FIRST_EGG_PRICE,
                    2 => TANK3_SECOND_EGG_PRICE,
                    3 => TANK3_THIRD_EGG_PRICE,
                    4 => TANK3_FOURTH_EGG_PRICE,
                    5 => TANK3_FIFTH_EGG_PRICE,
                    _ => unreachable!(),
                }
            } else if self.tank == 2 {
                match self.level {
                    1 => TANK2_FIRST_EGG_PRICE,
                    2 => TANK2_SECOND_EGG_PRICE,
                    3 => TANK2_THIRD_EGG_PRICE,
                    4 => TANK2_FOURTH_EGG_PRICE,
                    5 => TANK2_FIFTH_EGG_PRICE,
                    _ => unreachable!(),
                }
            } else {
                match self.level {
                    1 => EGG_PRICE,
                    2 => SECOND_STAGE_EGG_PRICE,
                    3 => THIRD_STAGE_EGG_PRICE,
                    4 => FOURTH_STAGE_EGG_PRICE,
                    5 => FIFTH_STAGE_EGG_PRICE,
                    _ => unreachable!(),
                }
            };
            if self.egg_price != expected_price {
                return Err("wrong egg price for Adventure stage".into());
            }
            if tank5 {
                self.validate_tank5()?;
            }
            if self.profile_population {
                self.validate_profile_population()?;
            } else {
                match (self.tank, self.level) {
                    (4, 1..=5)
                        if self.invasion.as_ref().is_none_or(|wave| {
                            !matches!(
                                (self.level, wave.plan),
                                (1, WavePlan::Fixed(SylvesterKind::Balrog))
                                    | (2, WavePlan::FixedBilaterus)
                                    | (3, WavePlan::CyclingTank4Third { .. })
                                    | (4, WavePlan::CyclingTank4Fourth { .. })
                                    | (5, WavePlan::CyclingTank4Finale { .. })
                            )
                        }) || self.pets.len() > 3
                            || self.pets.windows(2).any(|pair| {
                                let canonical = [
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
                                canonical.iter().position(|pet| *pet == pair[0])
                                    >= canonical.iter().position(|pet| *pet == pair[1])
                            })
                            || self.stinky.is_empty() == self.pets.contains(&PetKind::Stinky)
                            || self.niko.is_empty() == self.pets.contains(&PetKind::Niko)
                            || self.clyde.is_empty() == self.pets.contains(&PetKind::Clyde)
                            || self.rufus.is_empty() == self.pets.contains(&PetKind::Rufus)
                            || self.rhubarb.is_empty() == self.pets.contains(&PetKind::Rhubarb)
                            || self
                                .fish_pets
                                .iter()
                                .map(|pet| pet.kind)
                                .collect::<Vec<_>>()
                                != self
                                    .pets
                                    .iter()
                                    .filter_map(|pet| match pet {
                                        PetKind::Itchy => Some(FishPetKind::Itchy),
                                        PetKind::Prego => Some(FishPetKind::Prego),
                                        PetKind::Zorf => Some(FishPetKind::Zorf),
                                        PetKind::Vert => Some(FishPetKind::Vert),
                                        PetKind::Meryl => Some(FishPetKind::Meryl),
                                        PetKind::Wadsworth => Some(FishPetKind::Wadsworth),
                                        PetKind::Seymour => Some(FishPetKind::Seymour),
                                        PetKind::Shrapnel => Some(FishPetKind::Shrapnel),
                                        PetKind::Gumbo => Some(FishPetKind::Gumbo),
                                        PetKind::Blip => Some(FishPetKind::Blip),
                                        PetKind::Nimbus => Some(FishPetKind::Nimbus),
                                        PetKind::Amp => Some(FishPetKind::Amp),
                                        PetKind::Gash => Some(FishPetKind::Gash),
                                        PetKind::Angie => Some(FishPetKind::Angie),
                                        _ => None,
                                    })
                                    .collect::<Vec<_>>()
                            || self.guppy_unlocked
                            || (self.level == 1
                                && (self.weapon_unlocked
                                    || self.weapon_strength != 2
                                    || self.ultra_unlocked))
                            || (self.level >= 2
                                && (self.weapon_unlocked != self.ultra_unlocked
                                    || self.ultra_unlocked != self.egg_unlocked))
                            || self.starcatcher_unlocked
                            || self.grubber_unlocked
                            || self.gekko_unlocked
                            || self.potion_unlocked
                            || !self.starcatchers.is_empty()
                            || !self.dead_starcatchers.is_empty()
                            || !self.grubbers.is_empty()
                            || !self.dead_grubbers.is_empty()
                            || !self.gekkos.is_empty()
                            || !self.dead_gekkos.is_empty()
                            || self.upgrades.quality_unlocked
                                != self.upgrades.quantity_unlocked
                            || self.upgrades.quality_unlocked != self.oscar_unlocked
                            || (self.egg_unlocked && !self.oscar_unlocked)
                            || (!self.oscars.is_empty() && !self.egg_unlocked)
                            || (!self.dead_oscars.is_empty() && !self.egg_unlocked)
                            || ((!self.ultras.is_empty() || !self.dead_ultras.is_empty())
                                && !self.ultra_unlocked) =>
                    {
                        return Err("first Tank 4 roster or mapped shop gates disagree".into());
                    }
                    (3, 1)
                        if self.invasion.as_ref().is_none_or(|wave| {
                            wave.plan != WavePlan::Fixed(SylvesterKind::Balrog)
                        }) || self.pets.len() > 3
                            || self.pets.windows(2).any(|pair| {
                                let canonical = [
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
                                ];
                                canonical.iter().position(|pet| *pet == pair[0])
                                    >= canonical.iter().position(|pet| *pet == pair[1])
                            })
                            || self.stinky.is_empty() == self.pets.contains(&PetKind::Stinky)
                            || self.niko.is_empty() == self.pets.contains(&PetKind::Niko)
                            || self.clyde.is_empty() == self.pets.contains(&PetKind::Clyde)
                            || self.rufus.is_empty() == self.pets.contains(&PetKind::Rufus)
                            || self
                                .fish_pets
                                .iter()
                                .map(|pet| pet.kind)
                                .collect::<Vec<_>>()
                                != self
                                    .pets
                                    .iter()
                                    .filter_map(|pet| match pet {
                                        PetKind::Itchy => Some(FishPetKind::Itchy),
                                        PetKind::Prego => Some(FishPetKind::Prego),
                                        PetKind::Zorf => Some(FishPetKind::Zorf),
                                        PetKind::Vert => Some(FishPetKind::Vert),
                                        PetKind::Meryl => Some(FishPetKind::Meryl),
                                        PetKind::Wadsworth => Some(FishPetKind::Wadsworth),
                                        _ => None,
                                    })
                                    .collect::<Vec<_>>()
                            || self.oscar_unlocked
                            || self.starcatcher_unlocked
                            || self.weapon_unlocked
                            || self.potion_unlocked
                            || (self.egg_unlocked && !self.grubber_unlocked)
                            || (self.grubber_unlocked != self.upgrades.quality_unlocked)
                            || (self.grubber_unlocked != self.upgrades.quantity_unlocked)
                            || self.weapon_strength != 2
                            || !self.oscars.is_empty()
                            || !self.dead_oscars.is_empty()
                            || !self.starcatchers.is_empty()
                            || !self.dead_starcatchers.is_empty() =>
                    {
                        return Err("third-tank roster or Grubber gates disagree".into());
                    }
                    (3, 2..=5)
                        if self.invasion.as_ref().is_none_or(|wave| {
                            if self.level == 2 {
                                !matches!(wave.plan, WavePlan::CyclingTank3Second { .. })
                            } else if self.level == 5 {
                                !matches!(wave.plan, WavePlan::CyclingTank3Finale { .. })
                            } else if self.level == 4 {
                                wave.plan != WavePlan::Fixed(SylvesterKind::Ulysses)
                            } else {
                                wave.plan != WavePlan::Fixed(SylvesterKind::Psychosquid)
                            }
                        }) || self.pets.len() > 3
                            || self.pets.windows(2).any(|pair| {
                                let canonical = [
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
                                ];
                                canonical.iter().position(|pet| *pet == pair[0])
                                    >= canonical.iter().position(|pet| *pet == pair[1])
                            })
                            || self.stinky.is_empty() == self.pets.contains(&PetKind::Stinky)
                            || self.niko.is_empty() == self.pets.contains(&PetKind::Niko)
                            || self.clyde.is_empty() == self.pets.contains(&PetKind::Clyde)
                            || self.rufus.is_empty() == self.pets.contains(&PetKind::Rufus)
                            || self
                                .fish_pets
                                .iter()
                                .map(|pet| pet.kind)
                                .collect::<Vec<_>>()
                                != self
                                    .pets
                                    .iter()
                                    .filter_map(|pet| match pet {
                                        PetKind::Itchy => Some(FishPetKind::Itchy),
                                        PetKind::Prego => Some(FishPetKind::Prego),
                                        PetKind::Zorf => Some(FishPetKind::Zorf),
                                        PetKind::Vert => Some(FishPetKind::Vert),
                                        PetKind::Meryl => Some(FishPetKind::Meryl),
                                        PetKind::Wadsworth => Some(FishPetKind::Wadsworth),
                                        PetKind::Seymour => Some(FishPetKind::Seymour),
                                        PetKind::Shrapnel => Some(FishPetKind::Shrapnel),
                                        PetKind::Gumbo => Some(FishPetKind::Gumbo),
                                        PetKind::Blip => Some(FishPetKind::Blip),
                                        _ => None,
                                    })
                                    .collect::<Vec<_>>()
                            || self.oscar_unlocked
                            || self.starcatcher_unlocked
                            || self.potion_unlocked
                            || !self.oscars.is_empty()
                            || !self.dead_oscars.is_empty()
                            || !self.starcatchers.is_empty()
                            || !self.dead_starcatchers.is_empty()
                            || self.upgrades.quality_unlocked
                                != self.upgrades.quantity_unlocked
                            || self.upgrades.quality_unlocked != self.grubber_unlocked
                            || self.weapon_unlocked != self.egg_unlocked
                            || (self.egg_unlocked && !self.gekko_unlocked)
                            || (self.gekko_unlocked && !self.grubber_unlocked)
                            || ((!self.gekkos.is_empty()
                                || !self.dead_gekkos.is_empty()
                                || self.coins.iter().any(|coin| coin.kind == CoinKind::Pearl))
                                && !self.weapon_unlocked) =>
                    {
                        return Err("third-tank Gekko roster or purchase gates disagree".into());
                    }
                    (1, 1)
                        if !self.pets.is_empty()
                            || !self.stinky.is_empty()
                            || !self.niko.is_empty()
                            || self.invasion.is_some()
                            || !self.pearls.is_empty()
                            || self.upgrades.quality_unlocked
                            || self.upgrades.quantity_unlocked
                            || self.oscar_unlocked
                            || self.weapon_unlocked
                            || self.weapon_strength != 2
                            || !self.oscars.is_empty()
                            || !self.dead_oscars.is_empty()
                            || !self.fish_pets.is_empty() =>
                    {
                        return Err("first-stage roster or upgrades disagree".into());
                    }
                    (1, 2)
                        if self.pets.as_slice() != [PetKind::Stinky]
                            || self.stinky.is_empty()
                            || self.invasion.is_none()
                            || !self.niko.is_empty()
                            || !self.pearls.is_empty()
                            || self.oscar_unlocked
                            || self.weapon_unlocked
                            || self.weapon_strength != 2
                            || !self.oscars.is_empty()
                            || !self.dead_oscars.is_empty()
                            || !self.fish_pets.is_empty() =>
                    {
                        return Err("second-stage roster or wave disagree".into());
                    }
                    (1, 3)
                        if self.pets.as_slice() != [PetKind::Stinky, PetKind::Niko]
                            || self.stinky.is_empty()
                            || self.niko.is_empty()
                            || self.invasion.as_ref().is_none_or(|wave| {
                                wave.plan != WavePlan::Fixed(SylvesterKind::Strong)
                            })
                            || !self.fish_pets.is_empty() =>
                    {
                        return Err("third-stage roster disagrees".into());
                    }
                    (1, 4)
                        if self.pets.as_slice()
                            != [PetKind::Stinky, PetKind::Niko, PetKind::Itchy]
                            || self.stinky.is_empty()
                            || self.niko.is_empty()
                            || self.invasion.as_ref().is_none_or(|wave| {
                                wave.plan != WavePlan::Fixed(SylvesterKind::Balrog)
                            })
                            || self.fish_pets.len() != 1
                            || self.fish_pets[0].kind != FishPetKind::Itchy =>
                    {
                        return Err("fourth-stage roster or Balrog wave disagrees".into());
                    }
                    (1, 5)
                        if self.invasion.as_ref().is_none_or(|wave| {
                            !matches!(wave.plan, WavePlan::CyclingTank1Finale { .. })
                        }) || self.pets.len() > 3
                            || self.pets.iter().any(|pet| {
                                matches!(
                                    pet,
                                    PetKind::Zorf
                                        | PetKind::Clyde
                                        | PetKind::Vert
                                        | PetKind::Rufus
                                        | PetKind::Meryl
                                )
                            })
                            || self.pets.windows(2).any(|pair| {
                                let canonical = [
                                    PetKind::Stinky,
                                    PetKind::Niko,
                                    PetKind::Itchy,
                                    PetKind::Prego,
                                ];
                                canonical.iter().position(|pet| *pet == pair[0])
                                    >= canonical.iter().position(|pet| *pet == pair[1])
                            })
                            || self.stinky.is_empty() == self.pets.contains(&PetKind::Stinky)
                            || self.niko.is_empty() == self.pets.contains(&PetKind::Niko)
                            || self
                                .fish_pets
                                .iter()
                                .map(|pet| pet.kind)
                                .collect::<Vec<_>>()
                                != self
                                    .pets
                                    .iter()
                                    .filter_map(|pet| match pet {
                                        PetKind::Itchy => Some(FishPetKind::Itchy),
                                        PetKind::Prego => Some(FishPetKind::Prego),
                                        PetKind::Zorf => None,
                                        _ => None,
                                    })
                                    .collect::<Vec<_>>() =>
                    {
                        return Err("fifth-stage selected pet roster disagrees".into());
                    }
                    (2, 1)
                        if self.invasion.as_ref().is_none_or(|wave| {
                            wave.plan != WavePlan::Fixed(SylvesterKind::Strong)
                        }) || self.pets.len() > 3
                            || self.pets.iter().any(|pet| {
                                matches!(
                                    pet,
                                    PetKind::Clyde
                                        | PetKind::Vert
                                        | PetKind::Rufus
                                        | PetKind::Meryl
                                )
                            })
                            || self.pets.windows(2).any(|pair| {
                                let canonical = [
                                    PetKind::Stinky,
                                    PetKind::Niko,
                                    PetKind::Itchy,
                                    PetKind::Prego,
                                    PetKind::Zorf,
                                ];
                                canonical.iter().position(|pet| *pet == pair[0])
                                    >= canonical.iter().position(|pet| *pet == pair[1])
                            })
                            || self.stinky.is_empty() == self.pets.contains(&PetKind::Stinky)
                            || self.niko.is_empty() == self.pets.contains(&PetKind::Niko)
                            || self
                                .fish_pets
                                .iter()
                                .map(|pet| pet.kind)
                                .collect::<Vec<_>>()
                                != self
                                    .pets
                                    .iter()
                                    .filter_map(|pet| match pet {
                                        PetKind::Itchy => Some(FishPetKind::Itchy),
                                        PetKind::Prego => Some(FishPetKind::Prego),
                                        PetKind::Zorf => Some(FishPetKind::Zorf),
                                        _ => None,
                                    })
                                    .collect::<Vec<_>>()
                            || self.oscar_unlocked
                            || self.weapon_unlocked
                            || self.weapon_strength != 2
                            || !self.oscars.is_empty()
                            || !self.dead_oscars.is_empty()
                            || (self.egg_unlocked && !self.upgrades.quality_unlocked)
                            || (self.upgrades.quality_unlocked
                                != self.upgrades.quantity_unlocked)
                            || (self.upgrades.quality_unlocked != self.potion_unlocked)
                            || (self.upgrades.quality_unlocked != self.egg_unlocked) =>
                    {
                        return Err("second-tank roster or upgrade gates disagree".into());
                    }
                    (2, 2..=5)
                        if self.invasion.as_ref().is_none_or(|wave| {
                            if self.level == 5 {
                                !matches!(wave.plan, WavePlan::CyclingTank2Finale { .. })
                            } else {
                                wave.plan
                                    != WavePlan::Fixed(match self.level {
                                        2 => SylvesterKind::Balrog,
                                        3 => SylvesterKind::Gus,
                                        _ => SylvesterKind::Destructor,
                                    })
                            }
                        }) || self.pets.len() > 3
                            || (self.level != 5 && self.pets.contains(&PetKind::Meryl))
                            || (self.level < 4 && self.pets.contains(&PetKind::Rufus))
                            || (self.level == 2 && self.pets.contains(&PetKind::Vert))
                            || self.pets.windows(2).any(|pair| {
                                let canonical = [
                                    PetKind::Stinky,
                                    PetKind::Niko,
                                    PetKind::Itchy,
                                    PetKind::Prego,
                                    PetKind::Zorf,
                                    PetKind::Clyde,
                                    PetKind::Vert,
                                    PetKind::Rufus,
                                    PetKind::Meryl,
                                ];
                                canonical.iter().position(|pet| *pet == pair[0])
                                    >= canonical.iter().position(|pet| *pet == pair[1])
                            })
                            || self.stinky.is_empty() == self.pets.contains(&PetKind::Stinky)
                            || self.niko.is_empty() == self.pets.contains(&PetKind::Niko)
                            || self.clyde.is_empty() == self.pets.contains(&PetKind::Clyde)
                            || self.rufus.is_empty() == self.pets.contains(&PetKind::Rufus)
                            || self
                                .fish_pets
                                .iter()
                                .map(|pet| pet.kind)
                                .collect::<Vec<_>>()
                                != self
                                    .pets
                                    .iter()
                                    .filter_map(|pet| match pet {
                                        PetKind::Itchy => Some(FishPetKind::Itchy),
                                        PetKind::Prego => Some(FishPetKind::Prego),
                                        PetKind::Zorf => Some(FishPetKind::Zorf),
                                        PetKind::Vert => Some(FishPetKind::Vert),
                                        PetKind::Meryl => Some(FishPetKind::Meryl),
                                        _ => None,
                                    })
                                    .collect::<Vec<_>>()
                            || self.oscar_unlocked
                            || !self.oscars.is_empty()
                            || !self.dead_oscars.is_empty()
                            || self.upgrades.quality_unlocked
                                != self.upgrades.quantity_unlocked
                            || self.upgrades.quality_unlocked != self.potion_unlocked
                            || self.upgrades.quality_unlocked != self.starcatcher_unlocked
                            || self.egg_unlocked != self.weapon_unlocked
                            || (self.egg_unlocked && !self.starcatcher_unlocked) =>
                    {
                        return Err("second-tank roster or purchase gates disagree".into());
                    }
                    _ => {}
                }
            }
            if (self.tank, self.level) == (1, 2)
                && ((self.upgrades.quantity_unlocked || self.egg_unlocked)
                    && self.upgrades.quality == 0)
            {
                return Err("upgrade unlock order disagrees".into());
            }
            if self.tank == 1
                && self.level >= 3
                && ((!self.upgrades.quality_unlocked && self.upgrades.quantity_unlocked)
                    || (self.oscar_unlocked && !self.upgrades.quantity_unlocked)
                    || (self.weapon_unlocked && !self.oscar_unlocked)
                    || (self.egg_unlocked && !self.weapon_unlocked)
                    || (!self.oscar_unlocked && !self.oscars.is_empty()))
            {
                return Err("carnivore and weapon unlock order disagrees".into());
            }
            if !(self.notes.is_empty()
                || (self.profile_population
                    && (self.pets.contains(&PetKind::Meryl)
                        || self.pets.contains(&PetKind::Presto)))
                || (self.tank, self.level) == (2, 5)
                || (self.tank == 3
                    && (1..=5).contains(&self.level)
                    && self.pets.contains(&PetKind::Meryl))
                || (matches!((self.tank, self.level), (4, 1..=5))
                    && self.pets.contains(&PetKind::Meryl)))
                || self
                    .notes
                    .iter()
                    .any(|note| note.age_ticks > 100 || note.y < 0 || note.y > 550)
                || (!self.profile_population
                    && self
                        .fish_pets
                        .iter()
                        .filter(|pet| pet.kind == FishPetKind::Meryl)
                        .count()
                        > 1)
            {
                return Err("invalid Meryl note or roster state".into());
            }
        }
        for stinky in &self.stinky {
            stinky.validate()?;
            if (tank5
                || (self.time_trial || self.profile_population) && stinky.presto_form.is_some())
                != stinky.combat_id.is_some()
            {
                return Err("Stinky combat identity disagrees with Adventure stage".into());
            }
        }
        for clyde in &self.clyde {
            clyde.validate()?;
        }
        for rufus in &self.rufus {
            rufus.validate()?;
        }
        for rhubarb in &self.rhubarb {
            rhubarb.validate()?;
        }
        let projectile_stage = self.time_trial
            || self.tank == 2 && (4..=5).contains(&self.level)
            || matches!(
                (self.tank, self.level),
                (3, 2 | 4 | 5) | (4, 4..=5) | (5, 1)
            );
        if self.missiles.iter().any(|missile| {
            if missile.kind == MissileKind::Stanley {
                !self.pets.contains(&PetKind::Stanley) && !self.pets.contains(&PetKind::Presto)
            } else {
                !projectile_stage
            }
        }) {
            return Err("Destructor missiles outside supported Adventure stage".into());
        }
        if !self.time_trial
            && !self.profile_population
            && !self.rufus.is_empty()
            && !matches!((self.tank, self.level), (2, 4..=5) | (3..=5, _))
        {
            return Err("Rufus outside Adventure 2-4".into());
        }
        let mut assigned = HashSet::new();
        for missile in &self.missiles {
            missile.validate()?;
            if tank5
                && self
                    .invasion
                    .as_ref()
                    .and_then(|wave| wave.finale.as_ref())
                    .is_none_or(|finale| finale.boss.is_none())
            {
                return Err("Tank 5 missile without living boss".into());
            }
            if missile.kind != MissileKind::Stanley
                && !self.time_trial
                && !matches!(self.tank, 4 | 5)
                && (matches!((self.tank, self.level), (3, 4 | 5))
                    != (missile.kind == MissileKind::EnergyBall))
            {
                return Err("missile kind disagrees with Adventure stage".into());
            }
            let live_tank5_pet = tank5
                && (self
                    .stinky
                    .iter()
                    .any(|pet| pet.combat_id == missile.target_id)
                    || self
                        .niko
                        .iter()
                        .any(|pet| Some(pet.owner_id) == missile.target_id)
                    || self
                        .clyde
                        .iter()
                        .any(|pet| Some(pet.id) == missile.target_id)
                    || self
                        .rufus
                        .iter()
                        .any(|pet| Some(pet.id) == missile.target_id)
                    || self
                        .rhubarb
                        .iter()
                        .any(|pet| Some(pet.id) == missile.target_id)
                    || self
                        .fish_pets
                        .iter()
                        .any(|pet| Some(pet.id) == missile.target_id));
            if let Some(target_id) = missile.target_id {
                let target_valid = if missile.kind == MissileKind::Stanley {
                    self.invasion.as_ref().is_some_and(|wave| {
                        wave.actors.iter().any(|actor| actor.id == target_id)
                            || wave.bilaterus.iter().any(|group| group.id == target_id)
                    })
                } else {
                    self.live_prey_exists(target_id) || live_tank5_pet
                };
                if !assigned.insert(target_id) || !target_valid {
                    return Err("duplicate or missing missile target".into());
                }
            }
        }
        if !self.time_trial
            && (self.tank, self.level) == (3, 5)
            && self
                .missiles
                .iter()
                .any(|missile| missile.kind != MissileKind::Stanley)
            && self
                .invasion
                .as_ref()
                .is_some_and(|wave| wave.has_registered_kind(SylvesterKind::Psychosquid))
        {
            return Err("Psychosquid cannot coexist with a previous wave projectile".into());
        }
        if !self.missiles.is_empty()
            && self
                .invasion
                .as_ref()
                .is_none_or(|wave| wave.countdown != 3000)
        {
            return Err("classic missiles require frozen Destructor wave".into());
        }
        for actor in &self.starcatchers {
            actor.validate()?;
            if !actor.alive {
                return Err("removed Starcatcher cannot remain on the live board".into());
            }
        }
        for corpse in &self.dead_starcatchers {
            corpse.validate()?;
        }
        for actor in &self.grubbers {
            actor.validate()?;
        }
        for corpse in &self.dead_grubbers {
            corpse.validate()?;
        }
        for actor in &self.gekkos {
            actor.validate()?;
        }
        for corpse in &self.dead_gekkos {
            corpse.validate()?;
        }
        for actor in &self.breeders {
            actor.validate()?;
        }
        for corpse in &self.dead_breeders {
            corpse.validate()?;
        }
        for actor in &self.ultras {
            actor.validate()?;
            if !actor.alive {
                return Err("removed Ultra remains in live list".into());
            }
        }
        for corpse in &self.dead_ultras {
            corpse.validate()?;
        }
        for larva in &self.larvae {
            larva.validate()?;
        }
        if let Some(wave) = &self.invasion {
            wave.validate()?;
            if wave.battle_active != (wave.has_live_alien() || !self.missiles.is_empty()) {
                return Err("invasion battle membership latch disagrees".into());
            }
            if (self.tank, self.level) == (1, 2)
                && wave.plan != WavePlan::Fixed(SylvesterKind::Weak)
            {
                return Err("second-stage wave must be weak".into());
            }
        }
        for oscar in &self.oscars {
            oscar.validate()?;
        }
        for corpse in &self.dead_oscars {
            corpse.validate()?;
        }
        for pet in &self.fish_pets {
            if tank5 {
                pet.validate_tank5()?;
            } else {
                pet.validate()?;
            }
        }
        if self.bomb_shots.iter().any(|shot| {
            if shot.shot_type == 2 {
                !self.time_trial && !matches!((self.tank, self.level), (3, 4 | 5))
            } else {
                (!self.time_trial
                    && !self.profile_population
                    && !matches!((self.tank, self.level), (3, 3..=5) | (4, 1..=5)))
                    || !(self.pets.contains(&PetKind::Shrapnel)
                        || ((self.time_trial || self.profile_population)
                            && self.pets.contains(&PetKind::Presto))
                        || ((self.time_trial || matches!((self.tank, self.level), (4, 3..=5)))
                            && self.pets.contains(&PetKind::Amp)))
            }
        }) {
            return Err("finite shot outside supported Adventure stage".into());
        }
        for shot in &self.bomb_shots {
            shot.validate()?;
        }
        for niko in &self.niko {
            if tank5 {
                niko.validate_tank5()?;
            } else {
                niko.validate()?;
            }
            if (niko.presto_form.is_none()
                && self.tank == 1
                && (niko.anchor_x, niko.anchor_y) != (crate::niko::NIKO_X, crate::niko::NIKO_Y))
                || (niko.presto_form.is_none()
                    && self.tank == 2
                    && (niko.anchor_x, niko.anchor_y)
                        != (crate::niko::NIKO_TANK2_X, crate::niko::NIKO_TANK2_Y))
                || (niko.presto_form.is_none()
                    && self.tank == 3
                    && (niko.anchor_x, niko.anchor_y)
                        != (crate::niko::NIKO_TANK3_X, crate::niko::NIKO_TANK3_Y))
                || (niko.presto_form.is_none()
                    && self.tank == 4
                    && (niko.anchor_x, niko.anchor_y)
                        != (crate::niko::NIKO_TANK4_X, crate::niko::NIKO_TANK4_Y))
            {
                return Err("Niko anchor disagrees with tank".into());
            }
        }
        for pearl in &self.pearls {
            pearl.validate()?;
            if pearl.phase == PearlPhase::Finished
                || (!self.time_trial
                    && !self.profile_population
                    && !tank5
                    && !self.niko.is_empty()
                    && self.niko.iter().all(|niko| pearl.owner_id != niko.owner_id))
            {
                return Err("pearl owner or lifecycle disagrees".into());
            }
        }
        if !tank5
            && !self.time_trial
            && !self.profile_population
            && self.niko.is_empty()
            && !self.pearls.is_empty()
        {
            return Err("pearl without Niko owner".into());
        }
        // The project retains an inactive guppy record beside its same-ID
        // corpse until expiry. In PB05 Fish death unregisters the live actor
        // before creating that corpse; neither record counts as a survivor.
        // Other allocated entities own their ID exclusively.
        let mut ids = HashSet::new();
        let mut register = |id: u64| -> Result<(), String> {
            if id == 0 || id >= self.next_id || !ids.insert(id) {
                return Err("duplicate or unallocated Adventure entity ID".into());
            }
            Ok(())
        };
        for fish in &self.fish {
            register(fish.id)?;
        }
        let mut corpse_ids = HashSet::new();
        for corpse in &self.dead_fish {
            corpse.validate()?;
            if !corpse_ids.insert(corpse.id) {
                return Err("duplicate dead guppy ID".into());
            }
            match self.fish.iter().find(|fish| fish.id == corpse.id) {
                Some(fish) if !fish.alive => {}
                Some(_) => return Err("living guppy shares a corpse ID".into()),
                None => register(corpse.id)?,
            }
        }
        for actor in &self.oscars {
            register(actor.id)?;
        }
        for corpse in &self.dead_oscars {
            register(corpse.id)?;
        }
        for actor in &self.starcatchers {
            register(actor.id)?;
        }
        for corpse in &self.dead_starcatchers {
            register(corpse.id)?;
        }
        for actor in &self.grubbers {
            register(actor.id)?;
        }
        for corpse in &self.dead_grubbers {
            register(corpse.id)?;
        }
        for actor in &self.gekkos {
            register(actor.id)?;
        }
        for corpse in &self.dead_gekkos {
            register(corpse.id)?;
        }
        for actor in &self.breeders {
            register(actor.id)?;
        }
        for corpse in &self.dead_breeders {
            register(corpse.id)?;
        }
        for actor in &self.ultras {
            register(actor.id)?;
        }
        for corpse in &self.dead_ultras {
            register(corpse.id)?;
        }
        for larva in &self.larvae {
            register(larva.id)?;
        }
        for pet in &self.fish_pets {
            register(pet.id)?;
        }
        for stinky in &self.stinky {
            if let Some(combat_id) = stinky.combat_id {
                register(combat_id)?;
            }
        }
        for clyde in &self.clyde {
            register(clyde.id)?;
        }
        for rufus in &self.rufus {
            register(rufus.id)?;
        }
        for rhubarb in &self.rhubarb {
            register(rhubarb.id)?;
        }
        for missile in &self.missiles {
            register(missile.id)?;
        }
        for niko in &self.niko {
            register(niko.owner_id)?;
        }
        if let Some(wave) = &self.invasion {
            for alien in &wave.actors {
                register(alien.id)?;
            }
            for group in &wave.bilaterus {
                register(group.id)?;
            }
            for fragment in &wave.fragments {
                register(fragment.id)?;
            }
            for id in wave.finale_actor_ids() {
                register(id)?;
            }
        }
        for food in &self.food {
            register(food.id)?;
        }
        for coin in &self.coins {
            register(coin.id)?;
        }
        for note in &self.notes {
            register(note.id)?;
        }
        for pearl in &self.pearls {
            register(pearl.id)?;
        }
        if tank5 && self.niko.is_empty() && !self.pearls.is_empty() {
            let owner = self.pearls[0].owner_id;
            if owner == 0
                || owner >= self.next_id
                || ids.contains(&owner)
                || self.pearls.iter().any(|pearl| pearl.owner_id != owner)
            {
                return Err("Tank 5 retired pearl owner is invalid".into());
            }
        }
        if self.time_trial || self.profile_population {
            for pearl in &self.pearls {
                if self.niko.iter().any(|niko| niko.owner_id == pearl.owner_id) {
                    continue;
                }
                if pearl.owner_id == 0
                    || pearl.owner_id >= self.next_id
                    || ids.contains(&pearl.owner_id)
                {
                    return Err("retired pearl owner is invalid".into());
                }
            }
        }
        Ok(())
    }

    fn live_prey_exists(&self, id: u64) -> bool {
        self.fish.iter().any(|fish| fish.id == id && fish.alive)
            || self
                .ultras
                .iter()
                .any(|actor| actor.id == id && actor.alive)
            || self
                .oscars
                .iter()
                .any(|oscar| oscar.id == id && oscar.alive)
            || self
                .starcatchers
                .iter()
                .any(|actor| actor.id == id && actor.alive)
            || self
                .grubbers
                .iter()
                .any(|actor| actor.id == id && actor.alive)
            || self
                .gekkos
                .iter()
                .any(|actor| actor.id == id && actor.alive)
            || self
                .breeders
                .iter()
                .any(|actor| actor.id == id && actor.alive)
    }

    fn remove_missile(&mut self, id: u64, events: &mut Vec<Event>) -> bool {
        let Some(index) = self.missiles.iter().position(|missile| missile.id == id) else {
            return false;
        };
        let missile = self.missiles.remove(index);
        if missile.kind == MissileKind::Stanley {
            events.push(Event::StanleyMissileRemoved {
                tick: self.tick,
                missile_id: id,
            });
        } else if missile.kind == MissileKind::EnergyBall {
            events.push(Event::EnergyBallRemoved {
                tick: self.tick,
                missile_id: id,
            });
        } else {
            events.push(Event::MissileRemoved {
                tick: self.tick,
                missile_id: id,
            });
        }
        true
    }

    fn shoot_first_missile(&mut self, x: i32, y: i32, events: &mut Vec<Event>) -> bool {
        let Some(index) = self.missiles.iter().position(|missile| missile.shot(x, y)) else {
            return false;
        };
        let id = self.missiles[index].id;
        let outcome = self.missiles[index].try_shot(x, y);
        match outcome {
            MissileShot::Miss => return false,
            MissileShot::Destroyed => {
                self.remove_missile(id, events);
                self.finish_destructor_battle(events);
            }
            MissileShot::AcceptedImmune | MissileShot::Redirected => {
                let redirected = outcome == MissileShot::Redirected;
                // PB48/Shot.cpp: every accepted raw1 shot constructs a
                // finite invisible Shot2, even during immunity.
                let alpha = self.rand_range(200) as u8 + 50;
                self.bomb_shots.push(BombShot {
                    shot_type: 2,
                    x: x - 40,
                    y: y - 40,
                    age_ticks: 0,
                    frame: 0,
                    delay_ticks: 0,
                    alpha,
                });
                events.push(Event::EnergyBallShot {
                    tick: self.tick,
                    missile_id: id,
                    redirected,
                });
            }
        }
        true
    }

    fn fire_missile_only_laser(&mut self, x: i32, y: i32, events: &mut Vec<Event>) {
        if let Some(wave) = self.invasion.as_mut() {
            wave.lasers.push(crate::invasion::LaserEffect {
                x: x - 40,
                y: y - 40,
                age_ticks: 0,
            });
            wave.last_laser = Some((x, y));
            events.push(Event::Invasion {
                tick: self.tick,
                event: InvasionEvent::LaserFired { x, y },
            });
        }
    }

    fn detach_missile_target(&mut self, target_id: u64, events: &mut Vec<Event>) -> bool {
        if let Some(id) = self
            .missiles
            .iter()
            .find(|missile| missile.target_id == Some(target_id))
            .map(|missile| missile.id)
        {
            self.remove_missile(id, events);
            true
        } else {
            false
        }
    }

    fn finish_destructor_battle(&mut self, events: &mut Vec<Event>) {
        let Some(wave) = self.invasion.as_mut() else {
            return;
        };
        let ended = wave.finish_if_no_threats(!self.missiles.is_empty());
        self.record_invasion_events(ended, events);
    }

    fn missile_prey_view(&self, target_id: u64) -> Option<MissilePreyView> {
        if self.tank == 5 {
            return self
                .tank5_pet_prey()
                .into_iter()
                .find(|pet| pet.id == target_id && pet.eligible)
                .map(|pet| MissilePreyView {
                    id: pet.id,
                    widget_x: pet.widget_x,
                    widget_y: pet.widget_y,
                });
        }
        if let Some(fish) = self
            .fish
            .iter()
            .find(|fish| fish.id == target_id && fish.alive)
        {
            return Some(MissilePreyView {
                id: target_id,
                widget_x: fish.x as i32,
                widget_y: fish.y as i32,
            });
        }
        if let Some(oscar) = self
            .oscars
            .iter()
            .find(|actor| actor.id == target_id && actor.alive)
        {
            return Some(MissilePreyView {
                id: target_id,
                widget_x: oscar.widget_x,
                widget_y: oscar.widget_y,
            });
        }
        if let Some(actor) = self
            .grubbers
            .iter()
            .find(|actor| actor.id == target_id && actor.alive)
        {
            return Some(MissilePreyView {
                id: target_id,
                widget_x: actor.widget_x,
                widget_y: actor.widget_y,
            });
        }
        if let Some(actor) = self
            .gekkos
            .iter()
            .find(|actor| actor.id == target_id && actor.alive)
        {
            return Some(MissilePreyView {
                id: target_id,
                widget_x: actor.widget_x,
                widget_y: actor.widget_y,
            });
        }
        if let Some(actor) = self
            .breeders
            .iter()
            .find(|actor| actor.id == target_id && actor.alive)
        {
            return Some(MissilePreyView {
                id: target_id,
                widget_x: actor.widget_x,
                widget_y: actor.widget_y,
            });
        }
        self.starcatchers
            .iter()
            .find(|actor| actor.id == target_id && actor.alive)
            .map(|actor| MissilePreyView {
                id: target_id,
                widget_x: actor.widget_x,
                widget_y: actor.widget_y,
            })
    }

    fn stanley_target_view(&self, target_id: u64) -> Option<StanleyTargetView> {
        let wave = self.invasion.as_ref()?;
        if let Some(group) = wave.bilaterus.iter().find(|group| group.id == target_id) {
            return Some(StanleyTargetView {
                id: group.id,
                widget_x: group.widget_x,
                widget_y: group.widget_y,
                group: true,
                kind_six: false,
            });
        }
        wave.actors
            .iter()
            .find(|actor| actor.id == target_id)
            .map(|actor| {
                StanleyTargetView {
                    id: actor.id,
                    widget_x: actor.widget_x,
                    widget_y: actor.widget_y,
                    group: false,
                    // Class+8c==6 is Ultra, not Alien raw6/Ulysses. The current
                    // target factories admit ordinary Alien classes only.
                    kind_six: false,
                }
            })
    }

    fn update_missiles(&mut self, events: &mut Vec<Event>) {
        // The original Board may synchronously remove an earlier reservation
        // while a later reflected ball's collision is still in progress.
        // Schedule identities once, then reacquire each live actor after all
        // preceding death/removal transactions; never reuse a stale index.
        let scheduled_ids = self
            .missiles
            .iter()
            .map(|missile| missile.id)
            .collect::<Vec<_>>();
        for missile_id in scheduled_ids {
            let Some(mut index) = self
                .missiles
                .iter()
                .position(|missile| missile.id == missile_id)
            else {
                continue;
            };
            if self.missiles[index].kind == MissileKind::Stanley {
                let target = self.missiles[index]
                    .target_id
                    .and_then(|target_id| self.stanley_target_view(target_id));
                let bilaterus_present = self
                    .invasion
                    .as_ref()
                    .is_some_and(|wave| !wave.bilaterus.is_empty());
                let threats_present = self
                    .invasion
                    .as_ref()
                    .is_some_and(|wave| !wave.actors.is_empty() || !wave.bilaterus.is_empty());
                let update = self.missiles[index].begin_stanley_tick(
                    target,
                    bilaterus_present,
                    threats_present,
                );
                if update.remove {
                    self.remove_missile(missile_id, events);
                    self.finish_destructor_battle(events);
                    continue;
                }
                if let Some(target_id) = update.impact_target {
                    let mut rng_state = self.rng_state;
                    let mut next_id = self.next_id;
                    let result = if target.is_some_and(|view| view.group) {
                        self.invasion.as_mut().and_then(|wave| {
                            wave.stanley_hit_bilaterus(
                                target_id,
                                || Self::advance_rng(&mut rng_state) as u32,
                                || {
                                    let id = next_id;
                                    next_id += 1;
                                    id
                                },
                            )
                        })
                    } else {
                        self.invasion
                            .as_mut()
                            .and_then(|wave| wave.stanley_hit_alien(target_id))
                    };
                    self.rng_state = rng_state;
                    self.next_id = next_id;
                    if let Some((_, wave_events)) = result {
                        self.record_invasion_events(wave_events, events);
                    }
                    events.push(Event::StanleyMissileImpacted {
                        tick: self.tick,
                        missile_id,
                        target_id,
                    });
                }
                if let Some(index) = self
                    .missiles
                    .iter()
                    .position(|missile| missile.id == missile_id)
                    && self.missiles[index].finish_stanley_tick()
                {
                    self.remove_missile(missile_id, events);
                    self.finish_destructor_battle(events);
                }
                continue;
            }
            if self.missiles[index].diversion_clock > 0
                && self.missiles[index].diversion_clock.saturating_add(1)
                    >= self.missiles[index].diversion_limit
            {
                self.remove_missile(missile_id, events);
                self.finish_destructor_battle(events);
                continue;
            }
            if self.missiles[index].kind == MissileKind::EnergyBall
                && self.missiles[index].reflected
            {
                self.reflected_energy_contact(missile_id, events);
                // A retained fish dying can synchronously detach this ball.
                // The already-started collision routine still committed its
                // later category effects, but must never reinsert/update it.
                if !self.missiles.iter().any(|missile| missile.id == missile_id) {
                    self.finish_destructor_battle(events);
                    continue;
                }
                index = self
                    .missiles
                    .iter()
                    .position(|missile| missile.id == missile_id)
                    .expect("live reflected missile identity was just checked");
            }
            let view = self.missiles[index]
                .target_id
                .and_then(|target_id| self.missile_prey_view(target_id));
            let update = self.missiles[index].tick(view);
            if let Some(target_id) = update.impact_target {
                self.remove_tank5_pet(target_id);
                // OnFoodAte detaches this target, invokes its ordinary Die,
                // then unregisters the missile. No other actor observes the
                // brief in-method target relation before removal.
                if let Some(fish) = self
                    .fish
                    .iter_mut()
                    .find(|fish| fish.id == target_id && fish.alive)
                {
                    fish.alive = false;
                    self.dead_fish.push(DeadFish::from_live(fish));
                    events.push(Event::FishDied {
                        tick: self.tick,
                        fish_id: target_id,
                    });
                } else if let Some(pos) = self.oscars.iter().position(|actor| actor.id == target_id)
                {
                    self.dead_oscars
                        .push(DeadOscar::from_impact(&self.oscars[pos]));
                    self.oscars.remove(pos);
                    events.push(Event::OscarDied {
                        tick: self.tick,
                        oscar_id: target_id,
                    });
                } else if let Some(pos) = self
                    .starcatchers
                    .iter()
                    .position(|actor| actor.id == target_id)
                {
                    self.dead_starcatchers
                        .push(DeadStarcatcher::from_impact(&self.starcatchers[pos]));
                    self.starcatchers.remove(pos);
                    events.push(Event::StarcatcherDied {
                        tick: self.tick,
                        starcatcher_id: target_id,
                    });
                } else if let Some(pos) =
                    self.grubbers.iter().position(|actor| actor.id == target_id)
                {
                    self.dead_grubbers
                        .push(DeadGrubber::from_impact(&self.grubbers[pos]));
                    self.grubbers.remove(pos);
                    events.push(Event::GrubberDied {
                        tick: self.tick,
                        grubber_id: target_id,
                    });
                } else if let Some(pos) = self.gekkos.iter().position(|actor| actor.id == target_id)
                {
                    self.dead_gekkos
                        .push(DeadGekko::from_impact(&self.gekkos[pos]));
                    self.gekkos.remove(pos);
                    events.push(Event::GekkoDied {
                        tick: self.tick,
                        gekko_id: target_id,
                    });
                } else if let Some(pos) =
                    self.breeders.iter().position(|actor| actor.id == target_id)
                {
                    self.dead_breeders
                        .push(DeadBreeder::from_impact(&self.breeders[pos]));
                    self.breeders.remove(pos);
                    events.push(Event::BreederDied {
                        tick: self.tick,
                        breeder_id: target_id,
                        sound: false,
                    });
                }
                self.remove_missile(missile_id, events);
                events.push(Event::MissileImpacted {
                    tick: self.tick,
                    missile_id,
                    target_id,
                });
                self.finish_destructor_battle(events);
            } else if update.remove {
                self.remove_missile(missile_id, events);
                self.finish_destructor_battle(events);
            }
        }
    }

    fn reflected_energy_contact(&mut self, missile_id: u64, events: &mut Vec<Event>) {
        let Some(missile) = self
            .missiles
            .iter()
            .find(|missile| missile.id == missile_id)
        else {
            return;
        };
        let (ball_x, ball_y) = (missile.x, missile.y);
        let (x, y) = (ball_x + 40.0, ball_y + 40.0);
        let contact = |left: f64, top: f64, right: f64, bottom: f64| {
            x > left && x < right && y > top && y < bottom
        };
        // PB05 004fc070 tests ordinary Alien+b8, Bilaterus and boss before
        // either pet/fish category. Raw20 minis register in+bc, which this
        // path omits (00542ee0 instructions00543154..0054317b).
        let alien_id = self.invasion.as_ref().and_then(|wave| {
            wave.actors
                .iter()
                .find(|actor| {
                    !actor.healing
                        && contact(
                            f64::from(actor.widget_x + 30),
                            f64::from(actor.widget_y + 10),
                            f64::from(actor.widget_x + 140),
                            f64::from(actor.widget_y + 150),
                        )
                })
                .map(|actor| actor.id)
        });
        if let Some(alien_id) = alien_id {
            if let Some((health, wave_events)) = self
                .invasion
                .as_mut()
                .and_then(|wave| wave.reflected_energy_hit(alien_id))
            {
                events.push(Event::EnergyBallAlienHit {
                    tick: self.tick,
                    missile_id,
                    alien_id,
                    health,
                });
                self.record_invasion_events(wave_events, events);
                self.remove_missile(missile_id, events);
            }
            return;
        }
        let group_id = self.invasion.as_ref().and_then(|wave| {
            wave.bilaterus
                .iter()
                .find(|group| {
                    let (gx, gy) = group.active_head_position();
                    ball_x + 25.0 > f64::from(gx + 10)
                        && ball_x + 25.0 < f64::from(gx + 70)
                        && ball_y + 25.0 > f64::from(gy + 10)
                        && ball_y + 25.0 < f64::from(gy + 70)
                })
                .map(|group| group.id)
        });
        if let Some(group_id) = group_id {
            let mut rng_state = self.rng_state;
            let mut next_id = self.next_id;
            let result = self
                .invasion
                .as_mut()
                .unwrap()
                .reflected_energy_hit_bilaterus(
                    group_id,
                    || Self::advance_rng(&mut rng_state) as u32,
                    || {
                        let allocated = next_id;
                        next_id += 1;
                        allocated
                    },
                );
            self.rng_state = rng_state;
            self.next_id = next_id;
            if let Some((health, wave_events)) = result {
                events.push(Event::EnergyBallAlienHit {
                    tick: self.tick,
                    missile_id,
                    alien_id: group_id,
                    health,
                });
                self.record_invasion_events(wave_events, events);
                self.remove_missile(missile_id, events);
            }
            return;
        }
        let boss_id = self.invasion.as_ref().and_then(|wave| {
            wave.finale
                .as_ref()?
                .boss
                .as_ref()
                .filter(|boss| {
                    contact(
                        f64::from(boss.widget_x + 30),
                        f64::from(boss.widget_y + 10),
                        f64::from(boss.widget_x + 140),
                        f64::from(boss.widget_y + 150),
                    )
                })
                .map(|boss| boss.id)
        });
        if let Some(boss_id) = boss_id {
            if let Some((health, wave_events)) = self
                .invasion
                .as_mut()
                .unwrap()
                .reflected_energy_hit(boss_id)
            {
                events.push(Event::EnergyBallAlienHit {
                    tick: self.tick,
                    missile_id,
                    alien_id: boss_id,
                    health,
                });
                self.record_invasion_events(wave_events, events);
                self.remove_missile(missile_id, events);
            }
            return;
        }
        if self.tank == 5 && self.living_finale_boss() {
            let fish_pet_id = self
                .fish_pets
                .iter()
                .find(|pet| {
                    contact(
                        f64::from(pet.widget_x + 10),
                        f64::from(pet.widget_y + 10),
                        f64::from(pet.widget_x + 70),
                        f64::from(pet.widget_y + 70),
                    )
                })
                .map(|pet| pet.id);
            if let Some(pet_id) = fish_pet_id {
                self.remove_tank5_pet(pet_id);
                self.detach_missile_target(pet_id, events);
                events.push(Event::PetRemoved {
                    tick: self.tick,
                    pet_id,
                });
            }
            // Continue after the first FishTypePet removal, even if that
            // victim's assigned missile backlink detached this ball.
            let other_id = self
                .tank5_pet_prey()
                .into_iter()
                .find(|pet| {
                    (self
                        .stinky
                        .iter()
                        .any(|actor| actor.combat_id == Some(pet.id))
                        || self.niko.iter().any(|actor| actor.owner_id == pet.id)
                        || self.clyde.iter().any(|actor| actor.id == pet.id)
                        || self.rufus.iter().any(|actor| actor.id == pet.id)
                        || self.rhubarb.iter().any(|actor| actor.id == pet.id))
                        && contact(
                            f64::from(pet.widget_x + 10),
                            f64::from(pet.widget_y + 10),
                            f64::from(pet.widget_x + 70),
                            f64::from(pet.widget_y + 70),
                        )
                })
                .map(|pet| pet.id);
            if let Some(pet_id) = other_id {
                self.remove_tank5_pet(pet_id);
                self.detach_missile_target(pet_id, events);
                events.push(Event::PetRemoved {
                    tick: self.tick,
                    pet_id,
                });
                return;
            }
        }
        // Each lookup is made against current membership after the previous
        // category's Die transaction. The old projectile position is retained
        // locally even if its assigned prey removes it synchronously.
        let warded = self.ward_snapshot().is_some();
        if let Some(fish) = self.fish.iter_mut().find(|fish| {
            fish.alive
                && !(warded && matches!(fish.size, FishSize::Small | FishSize::Medium))
                && contact(
                    f64::from(fish.x as i32 + 10),
                    f64::from(fish.y as i32 + 10),
                    f64::from(fish.x as i32 + 70),
                    f64::from(fish.y as i32 + 70),
                )
        }) {
            let id = fish.id;
            fish.alive = false;
            self.dead_fish.push(DeadFish::from_live(fish));
            events.push(Event::FishDied {
                tick: self.tick,
                fish_id: id,
            });
            self.detach_missile_target(id, events);
        }
        if let Some(pos) = self.oscars.iter().position(|actor| {
            actor.alive
                && contact(
                    f64::from(actor.widget_x + 10),
                    f64::from(actor.widget_y + 10),
                    f64::from(actor.widget_x + 70),
                    f64::from(actor.widget_y + 70),
                )
        }) {
            let id = self.oscars[pos].id;
            self.dead_oscars
                .push(DeadOscar::from_impact(&self.oscars[pos]));
            self.oscars.remove(pos);
            events.push(Event::OscarDied {
                tick: self.tick,
                oscar_id: id,
            });
            self.detach_missile_target(id, events);
        }
        if let Some(pos) = self.starcatchers.iter().position(|actor| {
            actor.alive
                && contact(
                    f64::from(actor.widget_x + 10),
                    f64::from(actor.widget_y + 10),
                    f64::from(actor.widget_x + 70),
                    f64::from(actor.widget_y + 70),
                )
        }) {
            let id = self.starcatchers[pos].id;
            self.dead_starcatchers
                .push(DeadStarcatcher::from_impact(&self.starcatchers[pos]));
            self.starcatchers.remove(pos);
            events.push(Event::StarcatcherDied {
                tick: self.tick,
                starcatcher_id: id,
            });
            self.detach_missile_target(id, events);
        }
        if let Some(pos) = self.grubbers.iter().position(|actor| {
            actor.alive
                && contact(
                    f64::from(actor.widget_x + 10),
                    f64::from(actor.widget_y + 10),
                    f64::from(actor.widget_x + 70),
                    f64::from(actor.widget_y + 70),
                )
        }) {
            let id = self.grubbers[pos].id;
            self.dead_grubbers
                .push(DeadGrubber::from_impact(&self.grubbers[pos]));
            self.grubbers.remove(pos);
            events.push(Event::GrubberDied {
                tick: self.tick,
                grubber_id: id,
            });
            self.detach_missile_target(id, events);
        }
        if let Some(pos) = self.gekkos.iter().position(|actor| {
            actor.alive
                && contact(
                    f64::from(actor.widget_x + 10),
                    f64::from(actor.widget_y + 10),
                    f64::from(actor.widget_x + 70),
                    f64::from(actor.widget_y + 70),
                )
        }) {
            let id = self.gekkos[pos].id;
            self.dead_gekkos
                .push(DeadGekko::from_impact(&self.gekkos[pos]));
            self.gekkos.remove(pos);
            events.push(Event::GekkoDied {
                tick: self.tick,
                gekko_id: id,
            });
            self.detach_missile_target(id, events);
        }
        if let Some(pos) = self.breeders.iter().position(|actor| {
            actor.alive
                && contact(
                    f64::from(actor.widget_x + 10),
                    f64::from(actor.widget_y + 10),
                    f64::from(actor.widget_x + 70),
                    f64::from(actor.widget_y + 70),
                )
        }) {
            let id = self.breeders[pos].id;
            self.dead_breeders
                .push(DeadBreeder::from_impact(&self.breeders[pos]));
            self.breeders.remove(pos);
            events.push(Event::BreederDied {
                tick: self.tick,
                breeder_id: id,
                sound: false,
            });
            self.detach_missile_target(id, events);
        }
    }

    fn update_rufus(&mut self, events: &mut Vec<Event>) {
        if self.rufus.is_empty() {
            return;
        }
        if self.tank == 5 {
            let mut rng_state = self.rng_state;
            for pet in &mut self.rufus {
                pet.tick_tank5(&mut |upper| Self::advance_rng(&mut rng_state) % upper);
            }
            self.rng_state = rng_state;
            return;
        }
        for index in 0..self.rufus.len() {
            let mut aliens = self
                .invasion
                .as_ref()
                .map(|wave| {
                    wave.actors
                        .iter()
                        .map(|actor| RufusAlienView {
                            id: actor.id,
                            widget_x: actor.widget_x,
                            widget_y: actor.widget_y,
                            healing: actor.healing,
                            bilaterus: false,
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if let Some(wave) = &self.invasion {
                let mut groups = wave
                    .bilaterus
                    .iter()
                    .map(|group| {
                        let (widget_x, widget_y) = group.active_head_position();
                        RufusAlienView {
                            id: group.id,
                            widget_x,
                            widget_y,
                            healing: false,
                            bilaterus: true,
                        }
                    })
                    .collect::<Vec<_>>();
                groups.append(&mut aliens);
                aliens = groups;
            }
            let mut rng_state = self.rng_state;
            let pet = &mut self.rufus[index];
            let pet_id = pet.id;
            let update = pet.tick(&aliens, &mut |upper| {
                Self::advance_rng(&mut rng_state) % upper
            });
            if let Some(alien_id) = update.damaged_alien
                && let Some(wave) = self.invasion.as_mut()
                && let Some(health) = if wave.bilaterus_by_id(alien_id).is_some() {
                    wave.pet_hit_bilaterus(alien_id, 2.0)
                } else {
                    wave.actor_by_id_mut(alien_id)
                        .and_then(|actor| actor.rufus_hit())
                }
            {
                let sound = update.punch_requested && self.punch_sound_cooldown == 0;
                if sound {
                    self.punch_sound_cooldown = 10;
                }
                events.push(Event::RufusHit {
                    tick: self.tick,
                    pet_id,
                    alien_id,
                    health,
                    sound,
                });
            }
            self.rng_state = rng_state;
        }
    }

    /// The first-level rescue uses the bought-fish entrance without a purchase.
    pub(crate) fn spawn_bought_guppy(&mut self) -> u64 {
        let x = self.rand_range(520) as f32 + 20.0;
        let _target_y = self.rand_range(265) as f32 + 105.0;
        let mut fish = self.make_fish(x, 30.0, self.level == 1, true);
        fish.vy = self.rand_range(5) as f32 + 18.0;
        fish.bought_timer = self.rand_range(10) as u8 + 45;
        let id = fish.id;
        self.fish.push(fish);
        id
    }

    /// Apply an ordered input without advancing simulation time.
    pub fn apply(&mut self, action: Action) -> Vec<Event> {
        let mut events = vec![Event::Action {
            tick: self.tick,
            action: action.clone(),
        }];
        if self.victory {
            events.push(Event::Rejected {
                tick: self.tick,
                reason: Rejection::Completed,
            });
            return events;
        }
        match action {
            Action::Click { x, y } => {
                if !x.is_finite() || !y.is_finite() {
                    events.push(Event::Rejected {
                        tick: self.tick,
                        reason: Rejection::OutsideTank,
                    });
                } else {
                    self.apply_world_click(x, y, &mut events);
                }
            }
            Action::BuyGuppy => {
                if self.tank == 4 || !self.guppy_unlocked {
                    events.push(Event::Rejected {
                        tick: self.tick,
                        reason: Rejection::Locked,
                    });
                } else if self.available_funds() < GUPPY_PRICE {
                    events.push(Event::Rejected {
                        tick: self.tick,
                        reason: Rejection::InsufficientFunds,
                    });
                } else {
                    self.balance -= GUPPY_PRICE;
                    let id = self.spawn_bought_guppy();
                    self.tutorial.buy_fish_hint = false;
                    events.push(Event::GuppyBought {
                        tick: self.tick,
                        fish_id: id,
                        balance: self.balance,
                    });
                }
            }
            Action::BuyBreeder => {
                if self.tank != 4 || !self.breeder_unlocked {
                    events.push(Event::Rejected {
                        tick: self.tick,
                        reason: Rejection::Locked,
                    });
                } else if self.available_funds() < BREEDER_PRICE {
                    events.push(Event::Rejected {
                        tick: self.tick,
                        reason: Rejection::InsufficientFunds,
                    });
                } else {
                    self.balance -= BREEDER_PRICE;
                    let id = self.id();
                    let mut rng_state = self.rng_state;
                    let actor = BreederState::spawn_bought(id, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    });
                    self.rng_state = rng_state;
                    self.breeders.push(actor);
                    events.push(Event::BreederBought {
                        tick: self.tick,
                        breeder_id: id,
                        balance: self.balance,
                    });
                }
            }
            Action::BuyEgg => {
                if !self.egg_unlocked
                    || (self.tank == 5 && (!self.has_live_pets() || self.living_finale_boss()))
                {
                    events.push(Event::Rejected {
                        tick: self.tick,
                        reason: Rejection::Locked,
                    });
                } else if self.tank == 1 && self.level > 5 {
                    events.push(Event::Rejected {
                        tick: self.tick,
                        reason: Rejection::UnsupportedStage,
                    });
                } else if self.available_funds() < self.egg_price {
                    events.push(Event::Rejected {
                        tick: self.tick,
                        reason: Rejection::InsufficientFunds,
                    });
                } else {
                    self.balance -= self.egg_price;
                    self.eggs += 1;
                    if self.eggs == 1 && self.tutorial.buy_egg_hint {
                        self.tutorial.buy_egg_hint = false;
                        events.push(Event::Tutorial {
                            tick: self.tick,
                            cue: TutorialCue::EggsRemaining,
                        });
                    }
                    events.push(Event::EggBought {
                        tick: self.tick,
                        pieces: self.eggs,
                        balance: self.balance,
                    });
                    if self.eggs == 3 {
                        self.victory = true;
                        events.push(Event::LevelCompleted {
                            tick: self.tick,
                            next_tank: if (self.tank, self.level) == (4, 5) {
                                5
                            } else {
                                self.tank
                            },
                            // Final Tank4 advances to the boss tank directly.
                            next_level: if (self.tank, self.level) == (4, 5) {
                                1
                            } else {
                                self.level + 1
                            },
                        });
                    }
                }
            }
            Action::BuyFoodQuality => self.buy_food_upgrade(true, &mut events),
            Action::BuyFoodQuantity => self.buy_food_upgrade(false, &mut events),
            Action::BuyPotion => {
                if !self.potion_unlocked {
                    events.push(Event::Rejected {
                        tick: self.tick,
                        reason: Rejection::Locked,
                    });
                } else if !self.potion_armed {
                    if self.available_funds() < POTION_PRICE {
                        events.push(Event::Rejected {
                            tick: self.tick,
                            reason: Rejection::InsufficientFunds,
                        });
                    } else {
                        self.balance -= POTION_PRICE;
                        self.potion_armed = true;
                        events.push(Event::PotionBought {
                            tick: self.tick,
                            balance: self.balance,
                        });
                    }
                }
            }
            Action::BuyOscar => {
                if !matches!(self.tank, 1 | 4) || !self.oscar_unlocked {
                    events.push(Event::Rejected {
                        tick: self.tick,
                        reason: Rejection::Locked,
                    });
                } else if self.available_funds() < OSCAR_PRICE {
                    events.push(Event::Rejected {
                        tick: self.tick,
                        reason: Rejection::InsufficientFunds,
                    });
                } else {
                    self.balance -= OSCAR_PRICE;
                    let id = self.id();
                    let mut rng_state = self.rng_state;
                    let oscar = OscarState::spawn_bought(id, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    });
                    self.rng_state = rng_state;
                    self.oscars.push(oscar);
                    if self.tank != 4 {
                        self.weapon_unlocked = true;
                    } else if self.level >= 2 {
                        self.ultra_unlocked = true;
                        self.weapon_unlocked = true;
                    }
                    self.egg_unlocked = true;
                    events.push(Event::OscarBought {
                        tick: self.tick,
                        oscar_id: id,
                        balance: self.balance,
                    });
                }
            }
            Action::BuyUltra => {
                if !matches!((self.tank, self.level), (4, 2..=5)) || !self.ultra_unlocked {
                    events.push(Event::Rejected {
                        tick: self.tick,
                        reason: Rejection::Locked,
                    });
                } else if self.available_funds() < ULTRA_PRICE {
                    events.push(Event::Rejected {
                        tick: self.tick,
                        reason: Rejection::InsufficientFunds,
                    });
                } else {
                    self.balance -= ULTRA_PRICE;
                    let id = self.id();
                    let mut rng_state = self.rng_state;
                    let actor = UltraState::spawn_bought(id, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    });
                    self.rng_state = rng_state;
                    self.ultras.push(actor);
                    events.push(Event::UltraBought {
                        tick: self.tick,
                        id,
                        balance: self.balance,
                    });
                }
            }
            Action::BuyStarcatcher => {
                if !(self.tank == 2 && (2..=5).contains(&self.level) && self.starcatcher_unlocked) {
                    events.push(Event::Rejected {
                        tick: self.tick,
                        reason: Rejection::Locked,
                    });
                } else if self.available_funds() < STARCATCHER_PRICE {
                    events.push(Event::Rejected {
                        tick: self.tick,
                        reason: Rejection::InsufficientFunds,
                    });
                } else {
                    self.balance -= STARCATCHER_PRICE;
                    let id = self.id();
                    let mut rng_state = self.rng_state;
                    let actor = StarcatcherState::spawn_bought(id, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    });
                    self.rng_state = rng_state;
                    self.starcatchers.push(actor);
                    self.weapon_unlocked = true;
                    self.egg_unlocked = true;
                    events.push(Event::StarcatcherBought {
                        tick: self.tick,
                        starcatcher_id: id,
                        balance: self.balance,
                    });
                }
            }
            Action::BuyGrubber => {
                if self.tank != 3 || !(1..=5).contains(&self.level) || !self.grubber_unlocked {
                    events.push(Event::Rejected {
                        tick: self.tick,
                        reason: Rejection::Locked,
                    });
                } else if self.available_funds() < GRUBBER_PRICE {
                    events.push(Event::Rejected {
                        tick: self.tick,
                        reason: Rejection::InsufficientFunds,
                    });
                } else {
                    self.balance -= GRUBBER_PRICE;
                    let grubber_id = self.id();
                    let mut rng_state = self.rng_state;
                    let actor = GrubberState::spawn_bought(grubber_id, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    });
                    self.rng_state = rng_state;
                    self.grubbers.push(actor);
                    if self.level == 1 {
                        self.egg_unlocked = true;
                    } else {
                        self.gekko_unlocked = true;
                    }
                    events.push(Event::GrubberBought {
                        tick: self.tick,
                        grubber_id,
                        balance: self.balance,
                    });
                }
            }
            Action::BuyGekko => {
                if self.tank != 3 || !(2..=5).contains(&self.level) || !self.gekko_unlocked {
                    events.push(Event::Rejected {
                        tick: self.tick,
                        reason: Rejection::Locked,
                    });
                } else if self.available_funds() < GEKKO_PRICE {
                    events.push(Event::Rejected {
                        tick: self.tick,
                        reason: Rejection::InsufficientFunds,
                    });
                } else {
                    self.balance -= GEKKO_PRICE;
                    let gekko_id = self.id();
                    let mut rng_state = self.rng_state;
                    let actor = GekkoState::spawn_bought(gekko_id, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    });
                    self.rng_state = rng_state;
                    self.gekkos.push(actor);
                    self.weapon_unlocked = true;
                    self.egg_unlocked = true;
                    events.push(Event::GekkoBought {
                        tick: self.tick,
                        gekko_id,
                        balance: self.balance,
                    });
                }
            }
            Action::BuyWeapon => {
                let price = match self.tank {
                    3 => 2000,
                    4 => TANK4_WEAPON_PRICE,
                    _ => WEAPON_PRICE,
                };
                if !(self.tank == 1
                    || (self.tank == 2 && (2..=5).contains(&self.level))
                    || (self.tank == 3 && (2..=5).contains(&self.level))
                    || matches!((self.tank, self.level), (4, 2..=5)))
                    || !self.weapon_unlocked
                {
                    events.push(Event::Rejected {
                        tick: self.tick,
                        reason: Rejection::Locked,
                    });
                } else if self.weapon_strength >= 12 {
                    events.push(Event::Rejected {
                        tick: self.tick,
                        reason: Rejection::MaximumUpgrade,
                    });
                } else if self.available_funds() < price {
                    events.push(Event::Rejected {
                        tick: self.tick,
                        reason: Rejection::InsufficientFunds,
                    });
                } else {
                    self.balance -= price;
                    self.weapon_strength += 1;
                    events.push(Event::WeaponBought {
                        tick: self.tick,
                        strength: self.weapon_strength,
                        balance: self.balance,
                    });
                }
            }
            Action::HoldFeed { x, y, elapsed_ms } => {
                // Real held input belongs to the next board update. Do not
                // create food at a no-time input/save boundary.
                self.held_feed = Some((x, y, elapsed_ms));
            }
            Action::HoldFire { x, y, elapsed_ms } => {
                self.held_fire = Some((x, y, elapsed_ms));
            }
            Action::OpenMenu
            | Action::PlayAdventure
            | Action::PlayTimeTrial
            | Action::OfferBonusPurchase
            | Action::ConfirmBonusPurchase { .. }
            | Action::SelectTimeTrialTank { .. }
            | Action::RightClick { .. }
            | Action::PrestoPress { .. }
            | Action::PrestoRelease { .. }
            | Action::PrestoCancel
            | Action::Continue
            | Action::TogglePet { .. }
            | Action::ConfirmPetSelection { .. }
            | Action::HatchHold { .. } => {
                events.push(Event::Rejected {
                    tick: self.tick,
                    reason: Rejection::Locked,
                });
            }
        }
        events
    }

    fn apply_world_click(&mut self, x: f32, y: f32, events: &mut Vec<Event>) {
        let world_x = x as i32;
        let world_y = y as i32;
        // Larva is a separate raised widget rather than an ordinary Coin.
        // Its exact overlap priority against other source widgets is unknown;
        // this Board adaptation gives a visible larva first input, including
        // one already flying to the HUD. A repeated hit still reaches it.
        if let Some(larva) = self
            .larvae
            .iter_mut()
            .rev()
            .find(|larva| larva.contains_world_point(world_x, world_y))
        {
            events.push(Event::LarvaPickupSound {
                tick: self.tick,
                larva_id: larva.id,
            });
            if larva.try_pick_up() {
                events.push(Event::LarvaCollectionStarted {
                    tick: self.tick,
                    larva_id: larva.id,
                });
            }
            return;
        }
        // Niko pearls are sorted above ordinary coins. A collectible's mouse
        // handler only forwards this click to the Board when its WORLD point
        // overlaps an alien widget; PB05 Coin vtable +0xd8 confirms the
        // origin-adjusted forwarding, correcting W1's raw-local expression.
        let pearl_id = self
            .pearls
            .iter()
            .rev()
            .find(|pearl| {
                pearl.phase == PearlPhase::Waiting && pearl.contains_world_point(world_x, world_y)
            })
            .map(|pearl| pearl.id);
        let coin_id = if pearl_id.is_none() {
            self.coins
                .iter()
                .rev()
                .find(|coin| {
                    !coin.collecting
                        && world_x >= coin.x as i32
                        && world_x < coin.x as i32 + 72
                        && world_y >= coin.y as i32
                        && world_y < coin.y as i32 + 72
                })
                .map(|coin| coin.id)
        } else {
            None
        };

        // Food widget dimensions are 40x40. The upstream retail mouse hit
        // edge remains unindexed, so this is the project's bounded input
        // rectangle for its separately registered collectible Food.
        let nostra_food_id = if pearl_id.is_none() && coin_id.is_none() {
            self.food
                .iter()
                .rev()
                .find(|food| {
                    food.food_type == FoodType::Nostradamus
                        && !food.picked_up
                        && world_x >= food.x as i32
                        && world_x < food.x as i32 + 40
                        && world_y >= food.y as i32
                        && world_y < food.y as i32 + 40
                })
                .map(|food| food.id)
        } else {
            None
        };
        let on_collectible = pearl_id.is_some() || coin_id.is_some() || nostra_food_id.is_some();
        // FishTypePet's +0xd8 input slot reaches 004fb680 for raw23.
        // Its handler consumes an eligible click; an ineligible widget lets
        // the Board's other click routes continue. The 80x80 rectangle is a
        // provisional projection of the parent widget pending its inset read.
        if !on_collectible
            && let Some(index) = self.fish_pets.iter().rposition(|pet| {
                pet.walter_hitbox_contains(world_x, world_y) && pet.walter_ready_for_input()
            })
        {
            if self.fish_pets[index].try_punch_walter() {
                events.push(Event::WalterPunched {
                    tick: self.tick,
                    pet_id: self.fish_pets[index].id,
                });
            }
            return;
        }
        // PB66 separates Amp's clock from normal widget mouse availability.
        // Existing raised collectibles retain their input precedence; exact
        // installed overlap/modal routing remains qualified in the contract.
        let threats_registered = !self.missiles.is_empty()
            || self
                .invasion
                .as_ref()
                .is_some_and(Invasion1_2::has_live_alien);
        if !on_collectible
            && !threats_registered
            && let Some(index) = self
                .fish_pets
                .iter()
                .rposition(|pet| pet.amp_ready() && pet.amp_hitbox_contains(world_x, world_y))
        {
            let pet_id = self.fish_pets[index].id;
            match self.fish_pets[index].tap_amp(self.tank == 5) {
                AmpTap::Charged(charge) => events.push(Event::AmpCharged {
                    tick: self.tick,
                    pet_id,
                    charge,
                }),
                AmpTap::Discharged => self.discharge_amp(pet_id, events),
                AmpTap::NotReady => unreachable!("ready Amp was selected"),
            }
            return;
        }
        let on_alien_widget = self.invasion.as_ref().is_some_and(|wave| {
            wave.actors.iter().any(|alien| {
                world_x >= alien.widget_x
                    && world_x < alien.widget_x + 160
                    && world_y >= alien.widget_y
                    && world_y < alien.widget_y + 160
            }) || wave.finale.as_ref().is_some_and(|finale| {
                finale.boss.iter().chain(&finale.children).any(|actor| {
                    world_x >= actor.widget_x
                        && world_x < actor.widget_x + actor.sprite_size()
                        && world_y >= actor.widget_y
                        && world_y < actor.widget_y + actor.sprite_size()
                })
            }) || wave
                .bilaterus
                .iter()
                .any(|group| group.active_head_contains_widget(world_x, world_y))
        });
        let on_missile_widget = self.missiles.iter().any(|missile| {
            world_x >= missile.widget_x
                && world_x < missile.widget_x + 80
                && world_y >= missile.widget_y
                && world_y < missile.widget_y + 80
        });
        let gus_live = self
            .invasion
            .as_ref()
            .is_some_and(|wave| wave.has_registered_kind(SylvesterKind::Gus));
        let gus_feed_delay = self.invasion.as_ref().is_some_and(|wave| {
            wave.actors.iter().any(|alien| {
                if alien.kind != SylvesterKind::Gus {
                    return false;
                }
                let center_x = f64::from(alien.widget_x + 80);
                let center_y = f64::from(alien.widget_y + 80);
                center_x > f64::from(x) - 35.0
                    && center_x < f64::from(x) + 75.0
                    && center_y > f64::from(y) - 35.0
                    && center_y < f64::from(y) + 75.0
            })
        });
        if gus_live && !self.living_finale_boss() && (!on_collectible || on_alien_widget) {
            // Coin forwarding reaches Board MouseDown with world coordinates;
            // Gus's Board route creates food and never invokes generic Shot.
            if (31.0..=586.0).contains(&x) && (61.0..=379.0).contains(&y) {
                events.push(Event::GusInitialFeedAttempt {
                    tick: self.tick,
                    x: world_x,
                    y: world_y,
                });
                self.drop_food_with_price(
                    x,
                    y,
                    if gus_feed_delay { 20 } else { 0 },
                    0,
                    false,
                    events,
                );
            }
            if !on_collectible {
                return;
            }
        } else if (!on_collectible || on_alien_widget || on_missile_widget)
            && self.invasion.is_some()
        {
            let missiles_present = !self.missiles.is_empty();
            let alien_present = self
                .invasion
                .as_ref()
                .is_some_and(Invasion1_2::has_live_alien);
            let mut rng_state = self.rng_state;
            let mut next_id = self.next_id;
            let click = self
                .invasion
                .as_mut()
                .unwrap()
                .click_with_weapon_random_and_ids(
                    world_x,
                    world_y,
                    self.weapon_strength,
                    || Self::advance_rng(&mut rng_state) as u32,
                    || {
                        let allocated = next_id;
                        next_id += 1;
                        allocated
                    },
                );
            self.rng_state = rng_state;
            self.next_id = next_id;
            let suppress_food = click.suppress_food;
            self.record_invasion_events(click.events, events);
            if missiles_present && !alien_present && world_y > 40 {
                self.fire_missile_only_laser(world_x, world_y, events);
            }
            let missile_hit = world_y > 40 && self.shoot_first_missile(world_x, world_y, events);
            if !on_collectible && (suppress_food || missiles_present || missile_hit) {
                return;
            }
        }

        if let Some(pearl_id) = pearl_id {
            let Some(owner_id) = self
                .pearls
                .iter()
                .find(|pearl| pearl.id == pearl_id)
                .map(|pearl| pearl.owner_id)
            else {
                return;
            };
            if let Some(pearl) = self.pearls.iter_mut().find(|pearl| pearl.id == pearl_id)
                && pearl.pick_up(owner_id)
            {
                if let Some(niko) = self.niko.iter_mut().find(|niko| niko.owner_id == owner_id) {
                    niko.mark_pearl_taken(owner_id);
                }
                events.push(Event::PearlCollectionStarted {
                    tick: self.tick,
                    pearl_id,
                    owner_id,
                });
            }
            return;
        }
        if let Some(coin_id) = coin_id {
            if let Some(coin) = self.coins.iter_mut().find(|coin| coin.id == coin_id)
                && !coin.collecting
            {
                coin.collecting = true;
                events.push(Event::CoinCollectionStarted {
                    tick: self.tick,
                    coin_id,
                    kind: coin.kind,
                });
            }
            return;
        }
        if let Some(food_id) = nostra_food_id {
            if let Some(food) = self.food.iter_mut().find(|food| food.id == food_id)
                && !food.picked_up
            {
                food.picked_up = true;
                events.push(Event::NostradamusFoodCollectionStarted {
                    tick: self.tick,
                    food_id,
                });
            }
            return;
        }

        if x > 30.0 && x < 587.0 && y > 60.0 && y < 400.0 {
            if !self.missiles.is_empty()
                || self
                    .invasion
                    .as_ref()
                    .is_some_and(|wave| wave.has_live_alien() || wave.food_delay > 0)
            {
                return;
            }
            self.drop_food(x, y, 0, events);
        } else {
            events.push(Event::Rejected {
                tick: self.tick,
                reason: Rejection::OutsideTank,
            });
        }
    }

    fn drop_food(&mut self, x: f32, y: f32, ineligible_ticks: u8, events: &mut Vec<Event>) {
        if self.tank == 5 {
            events.push(Event::Rejected {
                tick: self.tick,
                reason: Rejection::Locked,
            });
            return;
        }
        let price = if self.potion_armed { 0 } else { FOOD_PRICE };
        self.drop_food_with_price(x, y, ineligible_ticks, price, false, events);
    }

    /// Gus's first click is free; its held repeat buys five coins before
    /// trying capacity and receives no invasion refund on rejection (PB32).
    fn drop_food_with_price(
        &mut self,
        x: f32,
        y: f32,
        ineligible_ticks: u8,
        price: i32,
        keep_charge_on_capacity_rejection: bool,
        events: &mut Vec<Event>,
    ) {
        let rejection = if self.available_funds() < price {
            Some(Rejection::InsufficientFunds)
        } else if self
            .food
            .iter()
            .filter(|food| food.direction == 0 && food.food_type == FoodType::Ordinary)
            .count()
            >= usize::from(self.upgrades.quantity)
        {
            if keep_charge_on_capacity_rejection {
                self.balance -= price;
            }
            Some(Rejection::FoodCapacity)
        } else {
            None
        };
        if let Some(reason) = rejection {
            events.push(Event::Rejected {
                tick: self.tick,
                reason,
            });
            return;
        }
        self.balance -= price;
        let quality = if self.potion_armed {
            3
        } else {
            self.upgrades.quality
        };
        self.potion_armed = false;
        let id = self.id();
        let animation_period = self.rand_range(2) as u8 + 3;
        self.food.push(Food {
            id,
            x: x - 10.0,
            y: y - 10.0,
            frame: 0,
            food_type: FoodType::Ordinary,
            picked_up: false,
            ineligible_ticks,
            removal_ticks: 0,
            quality,
            direction: 0,
            vx: 0.0,
            vy: 0.0,
            animation_period,
            free_from_zorf: false,
            nimbus_rising: false,
        });
        events.push(Event::FoodDropped {
            tick: self.tick,
            food_id: id,
            balance: self.balance,
            potion: quality == 3,
        });
    }

    fn buy_food_upgrade(&mut self, quality: bool, events: &mut Vec<Event>) {
        let unlocked = if quality {
            self.upgrades.quality_unlocked
        } else {
            self.upgrades.quantity_unlocked
        };
        let maxed = if quality {
            self.upgrades.quality >= 2
        } else {
            self.upgrades.quantity >= 9
        };
        let price = if quality {
            FOOD_QUALITY_PRICE
        } else {
            FOOD_QUANTITY_PRICE
        };
        let rejection = if !unlocked {
            Some(Rejection::Locked)
        } else if maxed {
            Some(Rejection::MaximumUpgrade)
        } else if self.available_funds() < price {
            Some(Rejection::InsufficientFunds)
        } else {
            None
        };
        if let Some(reason) = rejection {
            events.push(Event::Rejected {
                tick: self.tick,
                reason,
            });
            return;
        }
        self.balance -= price;
        if quality {
            self.upgrades.quality += 1;
            if (self.tank, self.level) == (1, 2) && !self.upgrades.quantity_unlocked {
                self.upgrades.quantity_unlocked = true;
                self.egg_unlocked = true;
                events.push(Event::Tutorial {
                    tick: self.tick,
                    cue: TutorialCue::BuyFoodQuantity,
                });
            }
            events.push(Event::FoodQualityBought {
                tick: self.tick,
                quality: self.upgrades.quality,
                balance: self.balance,
            });
        } else {
            self.upgrades.quantity += 1;
            if self.upgrades.quantity == 2 {
                events.push(Event::Tutorial {
                    tick: self.tick,
                    cue: TutorialCue::HoldToFeed,
                });
            }
            events.push(Event::FoodQuantityBought {
                tick: self.tick,
                quantity: self.upgrades.quantity,
                balance: self.balance,
            });
        }
    }

    /// One fixed 28 ms convenience update. The session uses the split calls
    /// so it can detect an empty tank after Board::Update but before objects.
    pub fn tick(&mut self) -> Vec<Event> {
        let mut events = self.begin_tick();
        if self
            .invasion
            .as_ref()
            .is_some_and(|wave| wave.pending_modal.is_some())
        {
            return events;
        }
        events.extend(self.update_objects());
        events
    }

    /// Board::Update reduces food delay, checks held feeding with the old
    /// board count, advances that count, then processes the invasion timer.
    pub(crate) fn begin_tick(&mut self) -> Vec<Event> {
        if self.victory {
            return Vec::new();
        }
        self.punch_sound_cooldown = self.punch_sound_cooldown.saturating_sub(1);
        if let Some(wave) = self.invasion.as_mut() {
            wave.update_before_pause();
        }
        if self
            .invasion
            .as_ref()
            .is_some_and(|wave| wave.pending_modal.is_some())
        {
            self.held_feed = None;
            self.held_fire = None;
            return Vec::new();
        }
        let mut events = Vec::new();
        if let Some((x, y, elapsed_ms)) = self.held_feed.take() {
            let feeding_blocked = self.tank == 5
                || self.invasion.as_ref().is_some_and(|wave| {
                    (wave.has_live_alien() && !wave.has_registered_kind(SylvesterKind::Gus))
                        || wave.food_delay > 0
                })
                || !self.missiles.is_empty();
            if !feeding_blocked
                && elapsed_ms > 200
                && self
                    .tick
                    .is_multiple_of(u64::from(16 - self.upgrades.quantity))
                && x.is_finite()
                && y.is_finite()
                && (31.0..=586.0).contains(&x)
                && (61.0..=399.0).contains(&y)
            {
                if self
                    .invasion
                    .as_ref()
                    .is_some_and(|wave| wave.has_registered_kind(SylvesterKind::Gus))
                {
                    self.drop_food_with_price(x, y, 20, FOOD_PRICE, true, &mut events);
                } else {
                    self.drop_food(x, y, 20, &mut events);
                }
            }
        }
        if let Some((x, y, elapsed_ms)) = self.held_fire.take()
            && self.weapon_strength == 12
            && elapsed_ms > 100
            && self.tick.is_multiple_of(5)
            && x.is_finite()
            && y.is_finite()
            && y > 40.0
            && let Some(wave) = self.invasion.as_mut()
            && !wave.has_registered_kind(SylvesterKind::Gus)
        {
            let missiles_present = !self.missiles.is_empty();
            let alien_present = wave.has_live_alien();
            let mut rng_state = self.rng_state;
            let mut next_id = self.next_id;
            let shot = wave.click_with_weapon_random_and_ids(
                x as i32,
                y as i32,
                self.weapon_strength,
                || Self::advance_rng(&mut rng_state) as u32,
                || {
                    let allocated = next_id;
                    next_id += 1;
                    allocated
                },
            );
            self.rng_state = rng_state;
            self.next_id = next_id;
            self.record_invasion_events(shot.events, &mut events);
            if missiles_present && !alien_present {
                self.fire_missile_only_laser(x as i32, y as i32, &mut events);
            }
            self.shoot_first_missile(x as i32, y as i32, &mut events);
        }
        self.advance_board_clock();
        if self.tank == 5 {
            self.egg_unlocked = self.has_live_pets()
                && self
                    .invasion
                    .as_ref()
                    .and_then(|wave| wave.finale.as_ref())
                    .is_some_and(|finale| finale.boss.is_none() && finale.boss_defeated);
        }
        if let Some(wave) = self.invasion.as_mut() {
            let mut rng_state = self.rng_state;
            let mut next_id = self.next_id;
            let missiles_present = !self.missiles.is_empty();
            let sneeze_actors = self
                .fish_pets
                .iter()
                .filter(|pet| pet.kind == FishPetKind::Nostradamus)
                .map(|pet| (pet.id, pet.widget_x, pet.widget_y))
                .collect::<Vec<_>>();
            let mut wave_events = wave.advance_with_threats(
                missiles_present,
                || Self::advance_rng(&mut rng_state) as u32,
                || {
                    let id = next_id;
                    next_id += 1;
                    id
                },
            );
            if let Some(postponed) =
                wave.try_nostradamus_postpone(&sneeze_actors, missiles_present, || {
                    Self::advance_rng(&mut rng_state) as u32
                })
            {
                wave_events.push(postponed);
            }
            self.rng_state = rng_state;
            self.next_id = next_id;
            self.record_invasion_events(wave_events, &mut events);
        }
        if self.has_live_blip()
            && self
                .invasion
                .as_ref()
                .is_some_and(|wave| wave.countdown == 225 && wave.warning.is_some())
        {
            events.push(Event::BlipSonar { tick: self.tick });
        }
        events
    }

    /// The already-paused board still decreases its drop-food delay before
    /// returning. The clock, flash, actors, and held-feeding check do not run.
    pub(crate) fn paused_board_update(&mut self) {
        self.punch_sound_cooldown = self.punch_sound_cooldown.saturating_sub(1);
        self.held_feed = None;
        self.held_fire = None;
        if let Some(wave) = self.invasion.as_mut() {
            wave.update_before_pause();
        }
    }

    /// Board-sorted object update: food, guppies, Oscars, alien, Stinky, Niko, coins,
    /// and the separately sorted Niko pearl list.
    pub(crate) fn update_objects(&mut self) -> Vec<Event> {
        if self.victory
            || self
                .invasion
                .as_ref()
                .is_some_and(|wave| wave.pending_modal.is_some())
        {
            return Vec::new();
        }
        let mut events = Vec::new();
        self.update_dead_fish(&mut events);
        self.update_other_corpses(&mut events);
        self.update_starcatchers(&mut events);
        self.update_food(&mut events);
        self.update_fish(&mut events);
        self.update_oscars(&mut events);
        self.update_ultras(&mut events);
        // Larva widgets precede later animal widgets in W1 Board sorting;
        // a newly emitted Larva begins its own updates on the next tick.
        self.update_larvae(&mut events);
        self.update_grubbers(&mut events);
        self.update_gekkos(&mut events);
        self.update_breeders(&mut events);
        self.update_invasion_objects(&mut events);
        self.update_missiles(&mut events);
        self.update_stinky(&mut events);
        self.update_niko(&mut events);
        self.update_clyde(&mut events);
        self.update_rufus(&mut events);
        self.update_rhubarb(&mut events);
        self.update_fish_pets(&mut events);
        self.update_coins(&mut events);
        // Effects created by a coin update are stepped after coin widgets in
        // this Board adaptation; the source WidgetManager's exact insertion
        // timing relative to a newly added Shot remains unverified.
        self.bomb_shots.retain_mut(|shot| !shot.tick());
        self.update_notes(&mut events);
        self.update_pearls(&mut events);
        events
    }

    /// Board::Update can advance its clock and open the first-level rescue
    /// dialog before the paused object widgets receive another update.
    pub(crate) fn advance_board_clock(&mut self) {
        self.tick += 1;
    }

    pub fn step(&mut self, actions: &[Action]) -> Vec<Event> {
        let mut events = Vec::new();
        for action in actions {
            events.extend(self.apply(action.clone()));
        }
        events.extend(self.tick());
        events
    }

    fn id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn rand(&mut self) -> u64 {
        Self::advance_rng(&mut self.rng_state)
    }

    pub(crate) fn initial_rng(seed: u64) -> u64 {
        if seed == 0 {
            0x9e37_79b9_7f4a_7c15
        } else {
            seed
        }
    }

    pub(crate) fn advance_rng(rng_state: &mut u64) -> u64 {
        let mut value = *rng_state;
        value ^= value >> 12;
        value ^= value << 25;
        value ^= value >> 27;
        *rng_state = value;
        value.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn rand_range(&mut self, upper: u64) -> u64 {
        self.rand() % upper
    }

    fn make_fish(&mut self, x: f32, y: f32, beginner: bool, bought: bool) -> Fish {
        let id = self.id();
        let vx = if self.rand_range(2) == 0 { 0.1 } else { -0.1 };
        let speed_mod = match self.rand_range(3) {
            0 => 2.0,
            1 => 1.8,
            _ => 1.6,
        };
        let hunger = self.rand_range(200) as i32 + 400;
        let food_needed_to_grow = self.rand_range(3) as u8 + 4;
        let movement_state = self.rand_range(10) as u8;
        let coin_threshold = self.rand_range(200) as u16 + 150;
        Fish {
            id,
            x,
            y,
            vx,
            vy: -0.5,
            facing_right: vx >= 0.0,
            frame: 0,
            turn_ticks: 0,
            hunger_visible: false,
            size: FishSize::Small,
            hunger,
            food_ate: 0,
            food_needed_to_grow,
            beginner,
            eating_ticks: 0,
            growth_ticks: 0,
            coin_timer: 0,
            coin_threshold,
            walter: crate::walter::WalterImpact::default(),
            alive: true,
            cannot_be_eaten_ticks: 0,
            speed_mod,
            movement_state,
            movement_timer: 0,
            special_timer: 40,
            x_direction: 1,
            previous_vx: if vx < 0.0 { -1.0 } else { 1.0 },
            swim_counter: 0,
            bought_timer: if bought { 45 } else { 0 },
        }
    }

    fn update_fish(&mut self, events: &mut Vec<Event>) {
        let meryl_song = self.fish_pets.iter().any(FishPetState::song_active);
        let ward = self.ward_snapshot();
        let alien_live = self
            .invasion
            .as_ref()
            .is_some_and(Invasion1_2::has_live_alien);
        let gumbo_widget = if alien_live {
            self.fish_pets
                .iter()
                .find(|pet| pet.kind == FishPetKind::Gumbo)
                .map(|pet| (pet.widget_x, pet.widget_y))
        } else {
            None
        };
        let mut index = 0;
        while index < self.fish.len() {
            if !self.fish[index].alive {
                index += 1;
                continue;
            }
            let choose_new_state =
                self.fish[index].movement_timer >= 20 && self.rand_range(10) == 0;
            let next_state = if choose_new_state {
                Some(self.rand_range(9) as u8 + 1)
            } else {
                None
            };
            if let Some(ward_position) = ward
                && matches!(self.fish[index].size, FishSize::Small | FishSize::Medium)
            {
                self.update_warded_fish(
                    index,
                    ward_position,
                    next_state,
                    alien_live,
                    meryl_song,
                    events,
                );
                index += 1;
                continue;
            }
            let mut coin_drop = None;
            let mut eaten_food = None;
            let mut converted_food = None;
            let mut grew = None;
            let mut died = false;
            let mut poisoned_corpse = None;
            let mut hunger_cue = None;
            'fish_update: {
                let fish = &mut self.fish[index];
                fish.cannot_be_eaten_ticks = fish.cannot_be_eaten_ticks.saturating_sub(1);
                if !alien_live {
                    fish.hunger = (fish.hunger - 1).max(-1000);
                }
                if fish.beginner {
                    hunger_cue = match fish.hunger {
                        -1 => Some(TutorialCue::Hungry),
                        -200 => Some(TutorialCue::VeryHungry),
                        -400 => Some(TutorialCue::Starving),
                        _ => None,
                    };
                }
                if (!fish.beginner && fish.hunger < 1) || (fish.beginner && fish.hunger < -499) {
                    fish.alive = false;
                    died = true;
                    // Fish::Hungry returns false after Die; this Update call
                    // still reaches DropCoin before deferred widget deletion.
                    coin_drop = if alien_live {
                        None
                    } else {
                        Self::coin_due(fish, meryl_song)
                    };
                } else {
                    let fish_cx = fish.x + 40.0;
                    let fish_cy = fish.y + 40.0;
                    let nearest = if fish.hunger < 500 {
                        self.food
                            .iter()
                            .enumerate()
                            .filter(|(_, food)| {
                                food.ineligible_ticks == 0
                                    && !food.picked_up
                                    && (food.food_type == FoodType::Ordinary
                                        || (matches!(
                                            fish.size,
                                            FishSize::Small | FishSize::Medium | FishSize::Large
                                        ) && fish.hunger < 108))
                            })
                            .fold(None, |best: Option<(usize, f32)>, (index, food)| {
                                let distance = (fish_cx - (food.x + 20.0)).powi(2)
                                    + (fish_cy - (food.y + 20.0)).powi(2);
                                if best.is_none_or(|(_, score)| distance < score) {
                                    Some((index, distance))
                                } else {
                                    best
                                }
                            })
                            .map(|(index, _)| index)
                    } else {
                        None
                    };
                    if let Some(food_index) = nearest {
                        let food = &self.food[food_index];
                        if fish_cx > food.x + 5.0
                            && fish_cx < food.x + 35.0
                            && fish_cy > food.y
                            && fish_cy < food.y + 35.0
                        {
                            if food.food_type == FoodType::Nostradamus {
                                converted_food = Some(food.id);
                                break 'fish_update;
                            }
                            eaten_food = Some(food.id);
                            if food.quality == 3 {
                                // Fish::Hungry may kill here, but its caller
                                // still performs the meal and this update's
                                // coin, animation and movement work.
                                match fish.size {
                                    FishSize::Small | FishSize::Medium => {
                                        poisoned_corpse = Some(DeadFish::from_live(fish));
                                        fish.alive = false;
                                        died = true;
                                    }
                                    FishSize::Large => {
                                        fish.hunger = (fish.hunger + 1100).min(1400);
                                        fish.size = FishSize::Star;
                                        grew = Some(FishSize::Star);
                                    }
                                    FishSize::Star | FishSize::Crowned => {
                                        fish.hunger = (fish.hunger + 1100).min(1400);
                                    }
                                }
                            } else {
                                let (nutrition, cap, growth_units) = match food.quality {
                                    0 => (if fish.beginner { 700 } else { 500 }, 800, 1),
                                    1 => (700, 1000, 2),
                                    2 => (1100, 1400, 3),
                                    _ => unreachable!("food quality validated at load"),
                                };
                                fish.hunger = (fish.hunger + nutrition).min(cap);
                                fish.food_ate = fish.food_ate.saturating_add(growth_units);
                                if fish.food_ate >= fish.food_needed_to_grow {
                                    match fish.size {
                                        FishSize::Small => {
                                            fish.size = FishSize::Medium;
                                            fish.food_ate = 0;
                                            fish.growth_ticks = 10;
                                            grew = Some(fish.size);
                                        }
                                        FishSize::Medium => {
                                            fish.size = FishSize::Large;
                                            fish.food_ate = 0;
                                            fish.growth_ticks = 10;
                                            grew = Some(fish.size);
                                        }
                                        FishSize::Large | FishSize::Star
                                            if u16::from(fish.food_ate)
                                                >= u16::from(fish.food_needed_to_grow) * 15 =>
                                        {
                                            fish.size = FishSize::Crowned;
                                            grew = Some(fish.size);
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            if fish.eating_ticks == 0 {
                                fish.eating_ticks = 8;
                            }
                        } else {
                            let dx = food.x + 20.0 - fish_cx;
                            let dy = food.y + 20.0 - fish_cy;
                            if fish.eating_ticks == 0 && dx.abs() < 30.0 && dy.abs() < 22.0 {
                                fish.eating_ticks = 20;
                            }
                            if fish.special_timer > 2 {
                                fish.special_timer = 0;
                                Self::steer_to_food(fish, dx, dy);
                            }
                        }
                    }
                    if nearest.is_none() {
                        if let Some((gx, gy)) = gumbo_widget {
                            Self::steer_to_gumbo(fish, gx, gy);
                        } else if fish.bought_timer == 0 {
                            Self::wander(fish);
                        }
                    }
                    fish.special_timer = fish.special_timer.saturating_add(1);
                    fish.movement_timer += 1;
                    if fish.movement_timer > 20 {
                        fish.movement_timer = 0;
                        if let Some(state) = next_state {
                            fish.movement_state = state;
                        }
                    }
                    // Fish.cpp::Update checks coin production before its final
                    // animation and position integration (lines 429, 519-533).
                    coin_drop = if alien_live {
                        None
                    } else {
                        Self::coin_due(fish, meryl_song)
                    };
                    if fish.bought_timer > 0 {
                        fish.bought_timer -= 1;
                        fish.vy *= 0.9;
                    }
                    fish.x += fish.walter.tick_recoil() as f32;
                    fish.walter.secondary = fish.walter.secondary.saturating_sub(1);
                    Self::advance_animation(fish);
                    fish.facing_right = fish.vx >= 0.0;
                    fish.x = (fish.x + fish.vx / fish.speed_mod).clamp(10.0, 540.0);
                    let min_y = if fish.bought_timer > 0 && fish.vy > 0.0 {
                        30.0
                    } else {
                        95.0
                    };
                    fish.y = (fish.y + fish.vy / fish.speed_mod).clamp(min_y, 370.0);
                    if fish.x <= 10.0 || fish.x >= 540.0 {
                        fish.vx = -fish.vx;
                    }
                    if fish.y <= 95.0 || fish.y >= 370.0 {
                        fish.vy = -fish.vy;
                    }
                    fish.hunger_visible = fish.hunger < 301;
                }
            }
            if let Some(food_id) = converted_food {
                let fish = self.fish.remove(index);
                let old_widget = (fish.x as i32, fish.y as i32);
                if self.detach_missile_target(fish.id, events) {
                    self.finish_destructor_battle(events);
                }
                let pet_id = self.id();
                let mut rng_state = self.rng_state;
                let pet = FishPetState::spawn_nostra_conversion_at(
                    pet_id,
                    old_widget.0,
                    old_widget.1,
                    &mut |upper| Self::advance_rng(&mut rng_state) % upper,
                );
                self.rng_state = rng_state;
                self.fish_pets.push(pet);
                let replacement_id =
                    (!self.has_live_fish() && self.tank != 5).then(|| self.spawn_bought_guppy());
                if let Some(position) = self.food.iter().position(|food| food.id == food_id) {
                    self.food.remove(position);
                }
                events.push(Event::NostradamusFishConverted {
                    tick: self.tick,
                    fish_id: fish.id,
                    food_id,
                    pet_id,
                    replacement_id,
                });
                // Removal shifts the next registered Fish into this index.
                continue;
            }
            if died {
                let fish_id = self.fish[index].id;
                let poison_death = poisoned_corpse.is_some();
                self.dead_fish.push(
                    poisoned_corpse.unwrap_or_else(|| DeadFish::from_live(&self.fish[index])),
                );
                events.push(Event::FishDied {
                    tick: self.tick,
                    fish_id: self.fish[index].id,
                });
                if self.detach_missile_target(fish_id, events) {
                    self.finish_destructor_battle(events);
                }
                if poison_death {
                    events.push(Event::PotionExploded {
                        tick: self.tick,
                        food_id: eaten_food.expect("poison death follows pellet contact"),
                        fish_id: Some(self.fish[index].id),
                    });
                }
            }
            if let Some(cue) = hunger_cue {
                let cue_index = match cue {
                    TutorialCue::Hungry => 0,
                    TutorialCue::VeryHungry => 1,
                    _ => 2,
                };
                if !self.tutorial.hunger_shown[cue_index] {
                    self.tutorial.hunger_shown[cue_index] = true;
                    events.push(Event::Tutorial {
                        tick: self.tick,
                        cue,
                    });
                }
            }
            if let Some(food_id) = eaten_food {
                self.food.retain(|food| food.id != food_id);
                let fish = &self.fish[index];
                events.push(Event::FoodEaten {
                    tick: self.tick,
                    fish_id: fish.id,
                    food_id,
                    hunger: fish.hunger,
                });
            }
            if let Some(size) = grew {
                if size == FishSize::Medium && self.tank != 4 && !self.guppy_unlocked {
                    self.guppy_unlocked = true;
                    self.tutorial.buy_fish_hint = true;
                    events.push(Event::Tutorial {
                        tick: self.tick,
                        cue: TutorialCue::BuyFish,
                    });
                }
                if size == FishSize::Large {
                    if (self.tank, self.level) == (1, 1) && !self.egg_unlocked {
                        self.egg_unlocked = true;
                        self.tutorial.buy_fish_hint = false;
                        self.tutorial.buy_egg_hint = true;
                        events.push(Event::Tutorial {
                            tick: self.tick,
                            cue: TutorialCue::BuyEgg,
                        });
                    } else if (self.tank, self.level) == (1, 2) && !self.upgrades.quality_unlocked {
                        self.upgrades.quality_unlocked = true;
                        events.push(Event::Tutorial {
                            tick: self.tick,
                            cue: TutorialCue::BuyFoodQuality,
                        });
                    } else if self.tank == 1 && self.level >= 3 {
                        // Fish::FishOnGrow unlocks quality, quantity and Oscar
                        // together at 1-3. Oscar purchase later unlocks Egg;
                        self.upgrades.quality_unlocked = true;
                        self.upgrades.quantity_unlocked = true;
                        self.oscar_unlocked = true;
                    }
                    if self.tank == 2 {
                        self.upgrades.quality_unlocked = true;
                        self.upgrades.quantity_unlocked = true;
                        self.potion_unlocked = true;
                        if self.level == 1 {
                            self.egg_unlocked = true;
                        } else if (2..=5).contains(&self.level) {
                            self.starcatcher_unlocked = true;
                        }
                    }
                    if self.tank == 3 {
                        self.upgrades.quality_unlocked = true;
                        self.upgrades.quantity_unlocked = true;
                        self.grubber_unlocked = true;
                    }
                    if self.tank == 4 {
                        self.upgrades.quality_unlocked = true;
                        self.upgrades.quantity_unlocked = true;
                        self.oscar_unlocked = true;
                    }
                }
                events.push(Event::FishGrew {
                    tick: self.tick,
                    fish_id: self.fish[index].id,
                    size,
                });
            }
            if let Some((fish_id, x, y, kind)) = coin_drop {
                let id = self.id();
                self.coins.push(Coin {
                    id,
                    x: f64::from(x),
                    y: f64::from(y),
                    kind,
                    frame: 0,
                    animation_ticks: 0,
                    hazard_age_ticks: 0,
                    collecting: false,
                    bottom_ticks: 0,
                    fade_ticks: 0,
                    penta_rising: false,
                });
                events.push(Event::CoinDropped {
                    tick: self.tick,
                    coin_id: id,
                    fish_id,
                    kind,
                });
                if !self.tutorial.coin_hint {
                    self.tutorial.coin_hint = true;
                    events.push(Event::Tutorial {
                        tick: self.tick,
                        cue: TutorialCue::CollectCoin,
                    });
                }
            }
            index += 1;
        }
    }

    fn update_warded_fish(
        &mut self,
        index: usize,
        ward_position: (i32, i32),
        next_state: Option<u8>,
        alien_live: bool,
        meryl_song: bool,
        events: &mut Vec<Event>,
    ) {
        let fish = &mut self.fish[index];
        let (ward_x, ward_y) = ward_position;
        // DropCoin runs before Move and reads the previous integer widget
        // position, even when Wadsworth moves the doubles during this update.
        let old_widget_x = fish.x.trunc();
        let old_widget_y = fish.y.trunc();
        fish.cannot_be_eaten_ticks = fish.cannot_be_eaten_ticks.saturating_sub(1);
        if fish.walter.recoil == 0 {
            fish.vx = if fish.vx < 0.0 { -0.5 } else { 0.5 };
        }
        if fish.bought_timer == 0 {
            fish.vy = 0.0;
        }
        for (position, goal, far_step) in [(&mut fish.x, ward_x, 5.0), (&mut fish.y, ward_y, 4.0)] {
            let distance = *position - goal as f32;
            if distance > 50.0 {
                *position -= far_step;
            } else if distance < -50.0 {
                *position += far_step;
            } else if distance > 5.0 {
                *position -= 3.0;
            } else if distance < -5.0 {
                *position += 3.0;
            }
        }
        fish.special_timer = fish.special_timer.saturating_add(1);
        fish.movement_timer += 1;
        if fish.movement_timer > 20 {
            fish.movement_timer = 0;
            if let Some(state) = next_state {
                fish.movement_state = state;
            }
        }
        let coin_drop = if alien_live {
            None
        } else {
            Self::coin_due(fish, meryl_song)
                .map(|(id, _, _, kind)| (id, old_widget_x + 5.0, old_widget_y + 10.0, kind))
        };
        if fish.bought_timer > 0 {
            fish.bought_timer -= 1;
            fish.vy *= 0.9;
        }
        fish.x += fish.walter.tick_recoil() as f32;
        fish.walter.secondary = fish.walter.secondary.saturating_sub(1);
        // Fish::Update clamps before its final integration, then eases the
        // horizontal velocity near an edge. The final Move may overshoot.
        fish.x = fish.x.clamp(10.0, 540.0);
        fish.y = fish.y.min(370.0);
        if fish.bought_timer == 0 || fish.vy <= 0.0 {
            fish.y = fish.y.max(95.0);
        }
        if fish.x > 535.0 && fish.vx > 0.1 {
            fish.vx -= 0.1;
        }
        if fish.x < 15.0 && fish.vx < -0.1 {
            fish.vx += 0.1;
        }
        Self::advance_animation(fish);
        fish.facing_right = fish.vx >= 0.0;
        fish.x += fish.vx / fish.speed_mod;
        fish.y += fish.vy / fish.speed_mod;
        fish.hunger_visible = fish.hunger < 301;
        if let Some((fish_id, x, y, kind)) = coin_drop {
            let coin_id = self.id();
            self.coins.push(Coin {
                id: coin_id,
                x: f64::from(x),
                y: f64::from(y),
                kind,
                frame: 0,
                animation_ticks: 0,
                hazard_age_ticks: 0,
                collecting: false,
                bottom_ticks: 0,
                fade_ticks: 0,
                penta_rising: false,
            });
            events.push(Event::CoinDropped {
                tick: self.tick,
                coin_id,
                fish_id,
                kind,
            });
        }
    }

    fn coin_due(fish: &mut Fish, meryl_song: bool) -> Option<(u64, f32, f32, CoinKind)> {
        if fish.size == FishSize::Small {
            return None;
        }
        fish.coin_timer += 1;
        if fish.coin_timer < (if meryl_song { 35 } else { fish.coin_threshold }) {
            return None;
        }
        fish.coin_timer = 0;
        // DropCoin uses the widget's integer position, which Move updated on
        // the previous integration, rather than its floating position.
        Some((
            fish.id,
            fish.x.trunc() + 5.0,
            fish.y.trunc() + 10.0,
            match fish.size {
                FishSize::Small => unreachable!(),
                FishSize::Medium => CoinKind::Silver,
                FishSize::Large => CoinKind::Gold,
                FishSize::Star => CoinKind::Star,
                // Fish::DropCoin passes the crowned size ordinal through as
                // ordinary coin type four, the diamond row/value.
                FishSize::Crowned => CoinKind::Diamond,
            },
        ))
    }

    /// W1 Fish::Update follows the first live selected Gumbo's *widget*
    /// coordinates while a registered alien is present. Hunger/warding have
    /// already taken their earlier branches.
    fn steer_to_gumbo(fish: &mut Fish, gx: i32, gy: i32) {
        let cx = fish.x + 40.0;
        let gx = gx as f32;
        if cx > gx + 50.0 && fish.vx > -4.0 {
            fish.vx -= 1.3;
        } else if cx < gx + 30.0 && fish.vx < 4.0 {
            fish.vx += 1.3;
        } else if cx > gx + 45.0 && fish.vx > -4.0 {
            fish.vx -= 0.2;
        } else if cx < gx + 35.0 && fish.vx < 4.0 {
            fish.vx += 0.2;
        } else if cx > gx + 40.0 && fish.vx > -4.0 {
            fish.vx -= 0.05;
        } else if cx < gx + 40.0 && fish.vx < 4.0 {
            fish.vx += 0.05;
        }
        let cy = fish.y + 40.0;
        let gy = gy as f32;
        if cy > gy + 25.0 && fish.vy > -3.0 {
            fish.vy -= 1.0;
        } else if cy < gy + 15.0 && fish.vy < 4.0 {
            fish.vy += 1.3;
        }
        if cy > gy + 20.0 && fish.vy > -3.0 {
            fish.vy -= 0.5;
        } else if cy < gy + 20.0 && fish.vy < 4.0 {
            fish.vy += 0.7;
        }
        if fish.y <= 95.0 && fish.vy < 0.0 {
            fish.vy = 0.0;
        }
    }

    fn steer_to_food(fish: &mut Fish, dx: f32, dy: f32) {
        let urgent = fish.hunger < 301;
        let horizontal = if dx.abs() > 8.0 {
            if urgent { 1.3 } else { 1.0 }
        } else if dx.abs() > 4.0 {
            if urgent { 0.2 } else { 0.1 }
        } else {
            0.05
        };
        let vertical = if dy.abs() > 6.0 {
            if urgent { 1.15 } else { 0.8 }
        } else {
            if urgent { 0.6 } else { 0.4 }
        };
        let x_limit = if urgent { 4.0 } else { 3.0 };
        let y_max = if urgent { 4.0 } else { 3.0 };
        let y_min = if urgent { -3.0 } else { -2.0 };
        fish.vx = (fish.vx + dx.signum() * horizontal).clamp(-x_limit, x_limit);
        fish.vy = (fish.vy + dy.signum() * vertical).clamp(y_min, y_max);
    }

    fn wander(fish: &mut Fish) {
        let change_velocity = fish.special_timer > 39;
        if change_velocity {
            fish.special_timer = 0;
        }
        match fish.movement_state {
            0 => {
                if change_velocity {
                    fish.vx = Self::approach(fish.vx, 0.0, 0.5);
                }
                fish.vy = 0.5;
                fish.y -= 0.25 / fish.speed_mod;
            }
            1 | 2 => {
                if change_velocity {
                    fish.vx = Self::approach(
                        fish.vx,
                        if fish.movement_state == 1 { 1.0 } else { -1.0 },
                        1.0,
                    );
                }
                fish.vy = -0.5;
                fish.y -= 0.5 / fish.speed_mod;
            }
            3 | 4 => {
                if change_velocity {
                    fish.vx = Self::approach(
                        fish.vx,
                        if fish.movement_state == 3 { -1.0 } else { 1.0 },
                        1.0,
                    );
                    fish.vy = Self::approach(fish.vy, 3.0, 1.0);
                }
                if fish.y > 240.0 {
                    fish.movement_state = 0;
                }
            }
            _ => {
                fish.vy = if fish.y >= 115.0 { -0.5 } else { -0.1 };
                if change_velocity {
                    fish.vx += fish.x_direction as f32;
                    if fish.x > 250.0 {
                        fish.x_direction = -1;
                    } else if fish.x < 175.0 {
                        fish.x_direction = 1;
                    }
                }
            }
        }
    }

    fn approach(value: f32, target: f32, step: f32) -> f32 {
        if value < target {
            (value + step).min(target)
        } else {
            (value - step).max(target)
        }
    }

    fn advance_animation(fish: &mut Fish) {
        if fish.previous_vx < 0.0 && fish.vx > 0.0 {
            fish.turn_ticks = -20;
        } else if fish.previous_vx > 0.0 && fish.vx < 0.0 {
            fish.turn_ticks = 20;
        }
        fish.turn_ticks -= fish.turn_ticks.signum();
        if fish.eating_ticks > 0 {
            fish.eating_ticks -= 1;
        }
        fish.frame = if fish.turn_ticks != 0 {
            (9 - (fish.turn_ticks.abs() / 2)) as u8
        } else if fish.eating_ticks > 0 {
            9 - fish.eating_ticks / 2
        } else {
            fish.swim_counter = (fish.swim_counter + if fish.vx.abs() < 2.0 { 1 } else { 2 }) % 20;
            fish.swim_counter / 2
        };
        if fish.growth_ticks > 0 {
            fish.growth_ticks -= 1;
        }
        if fish.vx != 0.0 {
            fish.previous_vx = fish.vx;
        }
    }

    fn update_dead_fish(&mut self, events: &mut Vec<Event>) {
        let mut expired = Vec::new();
        let mut revivals = Vec::new();
        for dead in &mut self.dead_fish {
            if dead.remaining_ticks == 0 {
                expired.push(dead.id);
                continue;
            }
            if dead.revival_ticks <= 10 {
                dead.revival_ticks -= 1;
                dead.frame = dead.revival_ticks.max(0) as u8;
                if dead.revival_ticks <= 0 {
                    expired.push(dead.id);
                    revivals.push(CorpseRevival {
                        corpse_id: dead.id,
                        kind: RevivalKind::Guppy(dead.size),
                        x: dead.x as i32,
                        y: dead.y as i32,
                        facing_right: dead.facing_right,
                    });
                    continue;
                }
            } else {
                dead.frame = if dead.remaining_ticks >= 106 {
                    (9 - (dead.remaining_ticks - 106) / 2) as u8
                } else if dead.remaining_ticks >= 103 {
                    8
                } else if dead.remaining_ticks >= 101 {
                    7
                } else {
                    6
                };
            }
            if dead.remaining_ticks < 105 {
                dead.opacity = (dead.opacity - 0.02).max(0.0);
            }
            if dead.remaining_ticks > 105 || dead.y > 370.0 {
                dead.remaining_ticks -= 1;
            }
            dead.vx = Self::approach(dead.vx, 0.0, 0.03);
            dead.vy = (dead.vy + 0.05).min(2.0);
            dead.x = (dead.x + dead.vx / dead.speed_mod).clamp(10.0, 540.0);
            dead.y = (dead.y + dead.vy / dead.speed_mod).clamp(85.0, 380.0);
        }
        self.dead_fish.retain(|dead| !expired.contains(&dead.id));
        self.fish
            .retain(|fish| fish.alive || self.dead_fish.iter().any(|dead| dead.id == fish.id));
        for request in revivals {
            self.revive_corpse(request, events);
        }
    }

    fn update_other_corpses(&mut self, events: &mut Vec<Event>) {
        // Preserve the existing category update order. Equivalence to the
        // original shared corpse-list order remains explicitly unproved.
        let mut revivals = Vec::new();
        self.dead_oscars.retain_mut(|corpse| {
            let removed = corpse.tick();
            if removed && corpse.remaining_ticks > 0 && corpse.revival_ticks == 0 {
                revivals.push(CorpseRevival {
                    corpse_id: corpse.id,
                    kind: RevivalKind::Oscar,
                    x: corpse.widget_x,
                    y: corpse.widget_y,
                    facing_right: corpse.facing_right,
                });
            }
            !removed
        });
        self.dead_starcatchers.retain_mut(|corpse| {
            let removed = corpse.tick();
            if removed && corpse.remaining_ticks > 0 && corpse.revival_ticks == 0 {
                revivals.push(CorpseRevival {
                    corpse_id: corpse.id,
                    kind: RevivalKind::Starcatcher,
                    x: corpse.widget_x,
                    y: corpse.widget_y,
                    facing_right: corpse.facing_right,
                });
            }
            !removed
        });
        self.dead_grubbers.retain_mut(|corpse| {
            let removed = corpse.tick();
            if removed && corpse.remaining_ticks > 0 && corpse.revival_ticks == 0 {
                revivals.push(CorpseRevival {
                    corpse_id: corpse.id,
                    kind: RevivalKind::Grubber,
                    x: corpse.widget_x,
                    y: corpse.widget_y,
                    facing_right: corpse.facing_right,
                });
            }
            !removed
        });
        self.dead_gekkos.retain_mut(|corpse| {
            let removed = corpse.tick();
            if removed && corpse.remaining_ticks > 0 && corpse.revival_ticks == 0 {
                revivals.push(CorpseRevival {
                    corpse_id: corpse.id,
                    kind: RevivalKind::Gekko,
                    x: corpse.widget_x,
                    y: corpse.widget_y,
                    facing_right: corpse.facing_right,
                });
            }
            !removed
        });
        self.dead_breeders.retain_mut(|corpse| {
            let removed = corpse.tick();
            if removed && corpse.remaining_ticks > 0 && corpse.revival_ticks == 0 {
                revivals.push(CorpseRevival {
                    corpse_id: corpse.id,
                    kind: RevivalKind::Breeder(corpse.size),
                    x: corpse.widget_x,
                    y: corpse.widget_y,
                    facing_right: corpse.facing_right,
                });
            }
            !removed
        });
        self.dead_ultras.retain_mut(|corpse| {
            let removed = corpse.tick();
            if removed && corpse.remaining_ticks > 0 && corpse.revival_ticks == 0 {
                revivals.push(CorpseRevival {
                    corpse_id: corpse.id,
                    kind: RevivalKind::Ultra,
                    x: corpse.widget_x,
                    y: corpse.widget_y,
                    facing_right: corpse.facing_right,
                });
            }
            !removed
        });
        for request in revivals {
            self.revive_corpse(request, events);
        }
    }

    fn revive_corpse(&mut self, request: CorpseRevival, events: &mut Vec<Event>) {
        if let RevivalKind::Guppy(size) = request.kind {
            let mut fish = self.make_fish(request.x as f32, request.y as f32, false, false);
            fish.size = size;
            fish.vx = if request.facing_right { 1.0 } else { -1.0 };
            fish.previous_vx = fish.vx;
            fish.facing_right = request.facing_right;
            let fish_id = fish.id;
            self.fish.push(fish);
            events.push(Event::FishRevived {
                tick: self.tick,
                corpse_id: request.corpse_id,
                fish_id,
            });
            return;
        }
        let fish_id = self.id();
        let mut rng_state = self.rng_state;
        let mut rng = |upper| Self::advance_rng(&mut rng_state) % upper;
        let (x, y, facing) = (request.x, request.y, request.facing_right);
        match request.kind {
            RevivalKind::Oscar => self
                .oscars
                .push(OscarState::spawn_revived(fish_id, x, y, facing, &mut rng)),
            RevivalKind::Ultra => self
                .ultras
                .push(UltraState::spawn_revived(fish_id, x, y, facing, &mut rng)),
            RevivalKind::Starcatcher => self
                .starcatchers
                .push(StarcatcherState::spawn_revived(fish_id, x, &mut rng)),
            RevivalKind::Grubber => self
                .grubbers
                .push(GrubberState::spawn_revived(fish_id, x, &mut rng)),
            RevivalKind::Gekko => self
                .gekkos
                .push(GekkoState::spawn_revived(fish_id, x, y, facing, &mut rng)),
            RevivalKind::Breeder(size) => self.breeders.push(BreederState::spawn_revived(
                fish_id, x, y, size, facing, &mut rng,
            )),
            RevivalKind::Guppy(_) => unreachable!(),
        }
        self.rng_state = rng_state;
        events.push(Event::FishRevived {
            tick: self.tick,
            corpse_id: request.corpse_id,
            fish_id,
        });
    }

    fn update_food(&mut self, events: &mut Vec<Event>) {
        let mut expired = Vec::new();
        let mut credited = Vec::new();
        for food in &mut self.food {
            if food.ineligible_ticks > 0 {
                food.ineligible_ticks -= 1;
            }
            food.frame = (food.frame + 1) % (food.animation_period * 10);
            if food.picked_up {
                let old_widget_y = food.y as i32;
                food.x += (550.0 - food.x) / 7.0;
                food.y += (30.0 - food.y) / 7.0;
                if old_widget_y < 40 {
                    credited.push(food.id);
                }
                continue;
            }
            if food.removal_ticks > 0 {
                food.removal_ticks -= 1;
                if food.removal_ticks == 0 {
                    expired.push(food.id);
                }
                continue;
            }
            if food.nimbus_rising {
                // Installed Food FUN004f8aa0 orders thresholds narrowly;
                // the secondary source's wider-first checks are contradicted.
                if food.y < 120.0 {
                    food.nimbus_rising = false;
                } else {
                    food.y -= Self::rise_step(f64::from(food.y)) as f32;
                }
            } else {
                food.y += 1.5;
            }
            if food.direction != 0 {
                if food.vy < 0.0 {
                    food.vy += 0.5;
                    food.y += food.vy;
                }
                if food.direction == 2 && food.vx > 0.0 {
                    food.vx -= 0.05;
                    food.x += food.vx;
                } else if food.direction == 1 && food.vx < 0.0 {
                    food.vx += 0.05;
                    food.x += food.vx;
                }
                food.x = food.x.clamp(20.0, 550.0);
            }
            if food.quality == 3 && food.y > 400.0 {
                expired.push(food.id);
                continue;
            }
            if food.y > 410.0 {
                food.removal_ticks = 15;
            }
        }
        for id in expired {
            if self
                .food
                .iter()
                .any(|food| food.id == id && food.quality == 3)
            {
                events.push(Event::PotionExploded {
                    tick: self.tick,
                    food_id: id,
                    fish_id: None,
                });
            }
            self.food.retain(|food| food.id != id);
            events.push(Event::FoodExpired {
                tick: self.tick,
                food_id: id,
            });
        }
        for id in credited {
            // 004f8aa0 removes the Food before passing literal one to the
            // ordinary Board cash receiver. The latch and removal each make
            // reloaded or repeated pickup incapable of a second credit.
            if let Some(position) = self.food.iter().position(|food| food.id == id) {
                self.food.remove(position);
                self.balance = (self.balance + 1).min(9_999_999);
                events.push(Event::NostradamusFoodCredited {
                    tick: self.tick,
                    food_id: id,
                    balance: self.balance,
                });
            }
        }
    }

    fn record_invasion_events(&mut self, wave_events: Vec<InvasionEvent>, events: &mut Vec<Event>) {
        // S3/S4: an ordinary Alien or final Bilaterus group loses its attached
        // raw2 before death/reward event handling and battle completion. A
        // first-head swap keeps the group's reservation. Self-impact has
        // already cleared its own association before calling invasion damage.
        let mut detached_target = false;
        for target_id in wave_events.iter().filter_map(|event| match event {
            InvasionEvent::AlienDefeated { id } | InvasionEvent::BilaterusDefeated { id } => {
                Some(*id)
            }
            _ => None,
        }) {
            detached_target |= self.detach_missile_target(target_id, events);
        }
        let alien_defeated = wave_events.iter().any(|event| {
            matches!(
                event,
                InvasionEvent::AlienDefeated { .. } | InvasionEvent::BilaterusDefeated { .. }
            )
        });
        for event in wave_events {
            match event {
                InvasionEvent::PreyEaten { prey_id, .. }
                | InvasionEvent::BilaterusPreyEaten { prey_id, .. } => {
                    // Alien::CheckCollision removes the live prey directly;
                    // there is no ordinary dead-fish corpse for this path.
                    self.fish.retain(|fish| fish.id != prey_id);
                    self.oscars.retain(|oscar| oscar.id != prey_id);
                    self.starcatchers.retain(|actor| actor.id != prey_id);
                    self.grubbers.retain(|actor| actor.id != prey_id);
                    self.gekkos.retain(|actor| actor.id != prey_id);
                    self.breeders.retain(|actor| actor.id != prey_id);
                    self.ultras.retain(|actor| actor.id != prey_id);
                    self.remove_tank5_pet(prey_id);
                    detached_target |= self.detach_missile_target(prey_id, events);
                }
                InvasionEvent::BossSpawned { .. } => {
                    self.weapon_strength = 10;
                }
                InvasionEvent::BossDefeated { .. } => {
                    // PB05 004f9950 clears the missile list but retains
                    // Bilaterus. No ordinary death rewards accompany cleanup.
                    let missile_ids = self
                        .missiles
                        .iter()
                        .map(|missile| missile.id)
                        .collect::<Vec<_>>();
                    for id in missile_ids {
                        self.remove_missile(id, events);
                    }
                    self.held_feed = None;
                    self.held_fire = None;
                }
                InvasionEvent::GusAteFood { food_id, .. } => {
                    self.food.retain(|food| food.id != food_id);
                }
                InvasionEvent::DiamondDropped { alien_id, x, y } => {
                    let coin_id = self.id();
                    self.coins.push(Coin {
                        id: coin_id,
                        x: f64::from(x),
                        y: f64::from(y),
                        kind: CoinKind::Diamond,
                        frame: 0,
                        animation_ticks: 0,
                        hazard_age_ticks: 0,
                        collecting: false,
                        bottom_ticks: 0,
                        fade_ticks: 0,
                        penta_rising: false,
                    });
                    events.push(Event::AlienDiamondDropped {
                        tick: self.tick,
                        alien_id,
                        coin_id,
                    });
                }
                InvasionEvent::BattleEnded => {
                    self.held_feed = None;
                    self.held_fire = None;
                }
                _ => {}
            }
            events.push(Event::Invasion {
                tick: self.tick,
                event,
            });
        }
        if alien_defeated || detached_target {
            self.finish_destructor_battle(events);
        }
    }

    fn update_starcatchers(&mut self, events: &mut Vec<Event>) {
        let alien_present = self
            .invasion
            .as_ref()
            .is_some_and(Invasion1_2::has_live_alien);
        let mut index = 0;
        while index < self.starcatchers.len() {
            let coins = self
                .coins
                .iter()
                .map(|coin| StarcatcherCoinView {
                    id: coin.id,
                    widget_x: coin.x as i32,
                    widget_y: coin.y as i32,
                    eligible: !coin.collecting && coin.kind == CoinKind::Star,
                })
                .collect::<Vec<_>>();
            let mut rng_state = self.rng_state;
            let update = self.starcatchers[index].tick(&coins, alien_present, &mut |upper| {
                Self::advance_rng(&mut rng_state) % upper
            });
            self.rng_state = rng_state;
            let actor_id = self.starcatchers[index].id;
            if let (Some(star_coin_id), Some((x, y))) = (update.eaten_coin, update.diamond_at) {
                // Commit one meal before the next Starcatcher sees the list.
                if let Some(star_index) = self.coins.iter().position(|coin| {
                    coin.id == star_coin_id && !coin.collecting && coin.kind == CoinKind::Star
                }) {
                    self.coins.remove(star_index);
                    let diamond_coin_id = self.id();
                    self.coins.push(Coin {
                        id: diamond_coin_id,
                        x: f64::from(x),
                        y: f64::from(y),
                        kind: CoinKind::DiamondPenta,
                        frame: 0,
                        animation_ticks: 0,
                        hazard_age_ticks: 0,
                        collecting: false,
                        bottom_ticks: 0,
                        fade_ticks: 0,
                        penta_rising: true,
                    });
                    events.push(Event::StarcatcherAteStar {
                        tick: self.tick,
                        starcatcher_id: actor_id,
                        star_coin_id,
                        diamond_coin_id,
                    });
                    events.push(Event::CoinDropped {
                        tick: self.tick,
                        coin_id: diamond_coin_id,
                        fish_id: actor_id,
                        kind: CoinKind::DiamondPenta,
                    });
                }
            }
            if update.died {
                self.dead_starcatchers
                    .push(DeadStarcatcher::from_live(&self.starcatchers[index]));
                events.push(Event::StarcatcherDied {
                    tick: self.tick,
                    starcatcher_id: actor_id,
                });
                if self.detach_missile_target(actor_id, events) {
                    self.finish_destructor_battle(events);
                }
                self.starcatchers.remove(index);
            } else {
                index += 1;
            }
        }
    }

    fn update_oscars(&mut self, events: &mut Vec<Event>) {
        let alien_present = self
            .invasion
            .as_ref()
            .is_some_and(Invasion1_2::has_live_alien);
        let mut index = 0;
        while index < self.oscars.len() {
            let prey = self
                .fish
                .iter()
                .map(|fish| OscarPrey {
                    id: fish.id,
                    widget_x: fish.x as i32,
                    widget_y: fish.y as i32,
                    eligible: fish.alive
                        && fish.size == FishSize::Small
                        && fish.cannot_be_eaten_ticks == 0,
                })
                .collect::<Vec<_>>();
            let mut rng_state = self.rng_state;
            let result = self.oscars[index].tick(&prey, alien_present, &mut |upper| {
                Self::advance_rng(&mut rng_state) % upper
            });
            self.rng_state = rng_state;
            let oscar_id = self.oscars[index].id;
            if let Some(guppy_id) = result.eaten_prey {
                // Widget sorting updates Oscar after the guppy; immediate
                // removal means a later Oscar or alien sees the new list.
                self.fish.retain(|fish| fish.id != guppy_id);
                events.push(Event::OscarAteGuppy {
                    tick: self.tick,
                    oscar_id,
                    guppy_id,
                });
                if self.detach_missile_target(guppy_id, events) {
                    self.finish_destructor_battle(events);
                }
            }
            if let Some((x, y)) = result.diamond {
                let coin_id = self.id();
                self.coins.push(Coin {
                    id: coin_id,
                    x: f64::from(x),
                    y: f64::from(y),
                    kind: CoinKind::Diamond,
                    frame: 0,
                    animation_ticks: 0,
                    hazard_age_ticks: 0,
                    collecting: false,
                    bottom_ticks: 0,
                    fade_ticks: 0,
                    penta_rising: false,
                });
                events.push(Event::CoinDropped {
                    tick: self.tick,
                    coin_id,
                    fish_id: oscar_id,
                    kind: CoinKind::Diamond,
                });
            }
            if result.died {
                self.dead_oscars
                    .push(DeadOscar::from_live(&self.oscars[index]));
                events.push(Event::OscarDied {
                    tick: self.tick,
                    oscar_id,
                });
                if self.detach_missile_target(oscar_id, events) {
                    self.finish_destructor_battle(events);
                }
                self.oscars.remove(index);
            } else {
                index += 1;
            }
        }
    }

    fn update_ultras(&mut self, events: &mut Vec<Event>) {
        let enemy = self
            .invasion
            .as_ref()
            .is_some_and(Invasion1_2::has_live_alien);
        let mut index = 0;
        while index < self.ultras.len() {
            let prey = self
                .oscars
                .iter()
                .map(|actor| UltraPrey {
                    id: actor.id,
                    widget_x: actor.widget_x,
                    widget_y: actor.widget_y,
                    eligible: actor.alive && actor.cannot_be_eaten_ticks == 0,
                })
                .collect::<Vec<_>>();
            let mut rng_state = self.rng_state;
            let update = self.ultras[index].tick(&prey, enemy, &mut |upper| {
                Self::advance_rng(&mut rng_state) % upper
            });
            self.rng_state = rng_state;
            let id = self.ultras[index].id;
            if let Some(oscar_id) = update.eaten_prey {
                self.oscars.retain(|actor| actor.id != oscar_id);
                events.push(Event::UltraAteOscar {
                    tick: self.tick,
                    id,
                    oscar_id,
                    hunger: self.ultras[index].hunger,
                });
                if self.detach_missile_target(oscar_id, events) {
                    self.finish_destructor_battle(events);
                }
            }
            if let Some((x, y)) = update.treasure {
                let coin_id = self.id();
                self.coins.push(Coin {
                    id: coin_id,
                    x: f64::from(x),
                    y: f64::from(y),
                    kind: CoinKind::Treasure,
                    frame: 0,
                    animation_ticks: 0,
                    hazard_age_ticks: 0,
                    collecting: false,
                    bottom_ticks: 0,
                    fade_ticks: 0,
                    penta_rising: false,
                });
                events.push(Event::UltraTreasureDropped {
                    tick: self.tick,
                    ultra_id: id,
                    coin_id,
                });
            }
            if update.died {
                self.dead_ultras
                    .push(DeadUltra::from_live(&self.ultras[index]));
                events.push(Event::UltraDied {
                    tick: self.tick,
                    id,
                });
                self.ultras.remove(index);
                if self.detach_missile_target(id, events) {
                    self.finish_destructor_battle(events);
                }
            } else {
                index += 1;
            }
        }
    }

    fn update_grubbers(&mut self, events: &mut Vec<Event>) {
        let alien_present = self
            .invasion
            .as_ref()
            .is_some_and(Invasion1_2::has_live_alien);
        let mut index = 0;
        while index < self.grubbers.len() {
            let prey = self
                .fish
                .iter()
                .map(|fish| GrubberPrey {
                    id: fish.id,
                    widget_x: fish.x as i32,
                    widget_y: fish.y as i32,
                    // Wadsworth wards against enemy acquisition, not this diet.
                    eligible: fish.alive
                        && fish.size == FishSize::Small
                        && fish.cannot_be_eaten_ticks == 0,
                })
                .collect::<Vec<_>>();
            let mut rng_state = self.rng_state;
            let update = self.grubbers[index].tick(&prey, alien_present, &mut |upper| {
                Self::advance_rng(&mut rng_state) % upper
            });
            self.rng_state = rng_state;
            let grubber_id = self.grubbers[index].id;
            if let Some(guppy_id) = update.eaten_prey {
                self.fish.retain(|fish| fish.id != guppy_id);
                events.push(Event::GrubberAteGuppy {
                    tick: self.tick,
                    grubber_id,
                    guppy_id,
                });
                if self.detach_missile_target(guppy_id, events) {
                    self.finish_destructor_battle(events);
                }
            }
            for (x, y) in update.bubbles {
                events.push(Event::GrubberBubble {
                    tick: self.tick,
                    grubber_id,
                    x,
                    y,
                });
            }
            // The actor may die in its hunger hook yet still produce on its
            // update tail. Construct now, before the next actor draws RNG.
            if let Some((x, y)) = update.larva_at {
                let larva_id = self.id();
                let larva = LarvaState::spawn(larva_id, x, y, &mut || self.rand());
                self.larvae.push(larva);
                events.push(Event::LarvaDropped {
                    tick: self.tick,
                    grubber_id,
                    larva_id,
                });
            }
            if update.died {
                self.dead_grubbers
                    .push(DeadGrubber::from_live(&self.grubbers[index]));
                events.push(Event::GrubberDied {
                    tick: self.tick,
                    grubber_id,
                });
                if self.detach_missile_target(grubber_id, events) {
                    self.finish_destructor_battle(events);
                }
                self.grubbers.remove(index);
            } else {
                index += 1;
            }
        }
    }

    fn update_larvae(&mut self, events: &mut Vec<Event>) {
        let mut index = 0;
        while index < self.larvae.len() {
            let larva_id = self.larvae[index].id;
            match self.larvae[index].tick() {
                LarvaUpdate::Alive => index += 1,
                LarvaUpdate::Expired => {
                    self.larvae.remove(index);
                    events.push(Event::LarvaExpired {
                        tick: self.tick,
                        larva_id,
                    });
                }
                LarvaUpdate::Credited => {
                    self.larvae.remove(index);
                    self.balance = (self.balance + LARVA_VALUE).min(9_999_999);
                    events.push(Event::LarvaCredited {
                        tick: self.tick,
                        larva_id,
                        amount: LARVA_VALUE,
                        balance: self.balance,
                    });
                }
            }
        }
    }

    fn update_gekkos(&mut self, events: &mut Vec<Event>) {
        let alien_present = self
            .invasion
            .as_ref()
            .is_some_and(Invasion1_2::has_live_alien);
        let mut index = 0;
        while index < self.gekkos.len() {
            // Build a fresh view per actor. The preceding Gekko's removal is
            // committed before this one can select or collide with its prey.
            let prey = self
                .larvae
                .iter()
                .map(|larva| GekkoPrey {
                    id: larva.id,
                    widget_x: larva.widget_x,
                    widget_y: larva.widget_y,
                    kind: GekkoPreyKind::Larva,
                    eligible: !larva.picked_up,
                })
                .collect::<Vec<_>>();
            // Raw-18 Peanuts belong to a later producer. No current CoinKind
            // can represent one; the actor's typed diet already supports it.
            let mut rng_state = self.rng_state;
            let result = self.gekkos[index].tick(&prey, alien_present, &mut |upper| {
                Self::advance_rng(&mut rng_state) % upper
            });
            self.rng_state = rng_state;
            let gekko_id = self.gekkos[index].id;
            if let Some((prey_id, kind)) = result.eaten_prey {
                if kind == GekkoPreyKind::Larva {
                    self.larvae.retain(|larva| larva.id != prey_id);
                }
                events.push(Event::GekkoAtePrey {
                    tick: self.tick,
                    gekko_id,
                    prey_id,
                    kind,
                });
            }
            for (x, y) in result.bubbles {
                events.push(Event::GekkoBubble {
                    tick: self.tick,
                    gekko_id,
                    x,
                    y,
                });
            }
            if let Some((x, y)) = result.pearl_at {
                let coin_id = self.id();
                self.coins.push(Coin {
                    id: coin_id,
                    x: f64::from(x),
                    y: f64::from(y),
                    kind: CoinKind::Pearl,
                    frame: 0,
                    animation_ticks: 0,
                    hazard_age_ticks: 0,
                    collecting: false,
                    bottom_ticks: 0,
                    fade_ticks: 0,
                    penta_rising: false,
                });
                events.push(Event::GekkoPearlDropped {
                    tick: self.tick,
                    gekko_id,
                    coin_id,
                });
                events.push(Event::CoinDropped {
                    tick: self.tick,
                    coin_id,
                    fish_id: gekko_id,
                    kind: CoinKind::Pearl,
                });
            }
            if result.died {
                self.dead_gekkos
                    .push(DeadGekko::from_live(&self.gekkos[index]));
                events.push(Event::GekkoDied {
                    tick: self.tick,
                    gekko_id,
                });
                if self.detach_missile_target(gekko_id, events) {
                    self.finish_destructor_battle(events);
                }
                self.gekkos.remove(index);
            } else {
                index += 1;
            }
        }
    }

    fn update_breeders(&mut self, events: &mut Vec<Event>) {
        let alien_registered = self
            .invasion
            .as_ref()
            .is_some_and(Invasion1_2::has_live_alien);
        // The source reads a separate specialty coordinate channel. No
        // ordinary producer for it has been established in this stage.
        let special_position = None;
        let gumbo = if alien_registered {
            self.fish_pets
                .iter()
                .find(|pet| pet.kind == FishPetKind::Gumbo)
                .map(|pet| (pet.widget_x, pet.widget_y))
        } else {
            None
        };
        let mut index = 0;
        while index < self.breeders.len() {
            let food = self
                .food
                .iter()
                .map(|item| BreederFoodView {
                    id: item.id,
                    widget_x: item.x as i32,
                    widget_y: item.y as i32,
                    quality: item.quality,
                    eligible: item.ineligible_ticks == 0,
                })
                .collect::<Vec<_>>();
            let actor_id = self.breeders[index].id;
            let mut rng_state = self.rng_state;
            let seek_during_invasion = self
                .invasion
                .as_ref()
                .and_then(|wave| wave.actors.first())
                .is_some_and(|actor| actor.kind != SylvesterKind::Gus);
            let update = self.breeders[index].begin_tick(
                &food,
                alien_registered,
                seek_during_invasion,
                special_position,
                gumbo,
                &mut |upper| Self::advance_rng(&mut rng_state) % upper,
            );
            self.rng_state = rng_state;
            // Hunger death snapshots the pre-tail pose. Production and its
            // newborn constructor still run in this same actor update.
            if update.died {
                self.dead_breeders.push(
                    update
                        .death_pose
                        .clone()
                        .expect("death snapshots before steering"),
                );
            }
            for meal in update.meals {
                self.food.retain(|item| item.id != meal.food_id);
                events.push(Event::BreederAteFood {
                    tick: self.tick,
                    breeder_id: actor_id,
                    food_id: meal.food_id,
                    hunger: meal.hunger,
                });
                if meal.grew {
                    if meal.size == BreederSize::Medium {
                        self.breeder_unlocked = true;
                    }
                    events.push(Event::BreederGrew {
                        tick: self.tick,
                        breeder_id: actor_id,
                        size: meal.size,
                    });
                }
            }
            if let Some((x, y)) = update.born_at {
                let newborn = self.make_fish(x as f32, y as f32, false, false);
                let fish_id = newborn.id;
                self.fish.push(newborn);
                events.push(Event::BreederBornGuppy {
                    tick: self.tick,
                    breeder_id: actor_id,
                    fish_id,
                    x,
                    y,
                });
            }
            let mut rng_state = self.rng_state;
            let _bubbles = self.breeders[index]
                .finish_tick(&mut |upper| Self::advance_rng(&mut rng_state) % upper);
            self.rng_state = rng_state;
            if update.died {
                events.push(Event::BreederDied {
                    tick: self.tick,
                    breeder_id: actor_id,
                    sound: true,
                });
                if self.detach_missile_target(actor_id, events) {
                    self.finish_destructor_battle(events);
                }
                self.breeders.remove(index);
            } else {
                index += 1;
            }
        }
    }

    fn bilaterus_prey(&self) -> Vec<BilaterusPreyView> {
        if self.tank == 5 {
            return self
                .tank5_pet_prey()
                .into_iter()
                .map(|pet| BilaterusPreyView {
                    id: pet.id,
                    widget_x: pet.widget_x,
                    widget_y: pet.widget_y,
                    width: 80,
                    height: 80,
                    ultra: false,
                    eligible: pet.eligible,
                })
                .collect();
        }
        let warded = self.ward_snapshot().is_some();
        let mut prey = self
            .fish
            .iter()
            .filter(|fish| fish.alive)
            .map(|fish| BilaterusPreyView {
                id: fish.id,
                widget_x: fish.x as i32,
                widget_y: fish.y as i32,
                width: 80,
                height: 80,
                ultra: false,
                eligible: fish.cannot_be_eaten_ticks == 0
                    && !(warded && matches!(fish.size, FishSize::Small | FishSize::Medium)),
            })
            .collect::<Vec<_>>();
        prey.extend(self.oscars.iter().filter(|actor| actor.alive).map(|actor| {
            BilaterusPreyView {
                id: actor.id,
                widget_x: actor.widget_x,
                widget_y: actor.widget_y,
                width: 80,
                height: 80,
                ultra: false,
                eligible: actor.cannot_be_eaten_ticks == 0,
            }
        }));
        prey.extend(self.ultras.iter().filter(|actor| actor.alive).map(|actor| {
            BilaterusPreyView {
                id: actor.id,
                widget_x: actor.widget_x,
                widget_y: actor.widget_y,
                width: 160,
                height: 160,
                ultra: true,
                eligible: actor.cannot_be_eaten_ticks == 0,
            }
        }));
        prey.extend(
            self.breeders
                .iter()
                .filter(|actor| actor.alive)
                .map(|actor| BilaterusPreyView {
                    id: actor.id,
                    widget_x: actor.widget_x,
                    widget_y: actor.widget_y,
                    width: 80,
                    height: 80,
                    ultra: false,
                    eligible: actor.cannot_be_eaten_ticks == 0,
                }),
        );
        prey
    }

    fn update_bilaterus(&mut self, events: &mut Vec<Event>) {
        let ids = self.invasion.as_ref().map_or_else(Vec::new, |wave| {
            wave.bilaterus.iter().map(|group| group.id).collect()
        });
        for id in ids {
            let (active, begin_events) = self.invasion.as_mut().unwrap().begin_bilaterus_update(id);
            self.record_invasion_events(begin_events, events);
            if !active {
                continue;
            }
            let children = [
                BilaterusChild::ActiveHead,
                BilaterusChild::Bone(0),
                BilaterusChild::Bone(1),
                BilaterusChild::Bone(2),
                BilaterusChild::Bone(3),
                BilaterusChild::Bone(4),
                BilaterusChild::Bone(5),
                BilaterusChild::PassiveHead,
            ];
            for child in children {
                let prey = self.bilaterus_prey();
                let mut rng_state = self.rng_state;
                let child_events = self.invasion.as_mut().unwrap().update_bilaterus_child(
                    id,
                    child,
                    &prey,
                    || Self::advance_rng(&mut rng_state) as u32,
                );
                self.rng_state = rng_state;
                self.record_invasion_events(child_events, events);
            }
            let mut rng_state = self.rng_state;
            let mut next_id = self.next_id;
            let tail = self.invasion.as_mut().unwrap().finish_bilaterus_update(
                id,
                || Self::advance_rng(&mut rng_state) as u32,
                || {
                    let allocated = next_id;
                    next_id += 1;
                    allocated
                },
            );
            self.rng_state = rng_state;
            self.next_id = next_id;
            self.record_invasion_events(tail, events);
        }
    }

    fn update_invasion_objects(&mut self, events: &mut Vec<Event>) {
        if self.tank == 5 {
            self.update_finale_objects(events);
        }
        let warded = self.ward_snapshot().is_some();
        let actor_ids = self.invasion.as_ref().map_or_else(Vec::new, |wave| {
            wave.actors.iter().map(|actor| actor.id).collect()
        });
        for actor_id in actor_ids {
            let mut prey = self
                .fish
                .iter()
                .filter(|fish| fish.alive)
                .map(|fish| PreyView {
                    id: fish.id,
                    widget_x: fish.x as i32,
                    widget_y: fish.y as i32,
                    width: 80,
                    height: 80,
                    eligible: fish.cannot_be_eaten_ticks == 0
                        && !(warded && matches!(fish.size, FishSize::Small | FishSize::Medium)),
                })
                .collect::<Vec<_>>();
            prey.extend(
                self.oscars
                    .iter()
                    .filter(|oscar| oscar.alive)
                    .map(|oscar| PreyView {
                        id: oscar.id,
                        widget_x: oscar.widget_x,
                        widget_y: oscar.widget_y,
                        width: 80,
                        height: 80,
                        eligible: oscar.cannot_be_eaten_ticks == 0,
                    }),
            );
            prey.extend(
                self.starcatchers
                    .iter()
                    .filter(|actor| actor.alive)
                    .map(|actor| PreyView {
                        id: actor.id,
                        widget_x: actor.widget_x,
                        widget_y: actor.widget_y,
                        width: 80,
                        height: 80,
                        eligible: actor.cannot_be_eaten_ticks == 0,
                    }),
            );
            prey.extend(
                self.grubbers
                    .iter()
                    .filter(|actor| actor.alive)
                    .map(|actor| PreyView {
                        id: actor.id,
                        widget_x: actor.widget_x,
                        widget_y: actor.widget_y,
                        width: 80,
                        height: 80,
                        eligible: true,
                    }),
            );
            prey.extend(
                self.gekkos
                    .iter()
                    .filter(|actor| actor.alive)
                    .map(|actor| PreyView {
                        id: actor.id,
                        widget_x: actor.widget_x,
                        widget_y: actor.widget_y,
                        width: 80,
                        height: 80,
                        eligible: true,
                    }),
            );
            prey.extend(
                self.breeders
                    .iter()
                    .filter(|actor| actor.alive)
                    .map(|actor| PreyView {
                        id: actor.id,
                        widget_x: actor.widget_x,
                        widget_y: actor.widget_y,
                        width: 80,
                        height: 80,
                        eligible: actor.cannot_be_eaten_ticks == 0,
                    }),
            );
            prey.extend(self.tank5_pet_prey());
            let food = self
                .food
                .iter()
                .map(|item| AlienFoodView {
                    id: item.id,
                    widget_x: item.x as i32,
                    widget_y: item.y as i32,
                    quality: item.quality,
                    eligible: item.ineligible_ticks == 0,
                })
                .collect::<Vec<_>>();
            if let Some(wave) = self.invasion.as_mut() {
                let mut rng_state = self.rng_state;
                let mut next_id = self.next_id;
                let missiles = &mut self.missiles;
                let mut launches = Vec::new();
                let kind = if wave
                    .actor_by_id(actor_id)
                    .is_some_and(|actor| actor.kind == SylvesterKind::Ulysses)
                {
                    MissileKind::EnergyBall
                } else {
                    MissileKind::Classic
                };
                let wave_events = wave.update_actor_with_runtime(
                    actor_id,
                    &prey,
                    &food,
                    |request| match request {
                        AlienRuntimeRequest::Random => Self::advance_rng(&mut rng_state) as u32,
                        AlienRuntimeRequest::ProbeTarget {
                            center_x,
                            center_y,
                            excluded_prey,
                        } => {
                            let mut best = 0_i64;
                            for candidate in prey.iter().filter(|candidate| {
                                candidate.eligible && Some(candidate.id) != excluded_prey
                            }) {
                                if missiles
                                    .iter()
                                    .any(|missile| missile.target_id == Some(candidate.id))
                                {
                                    continue;
                                }
                                let dx =
                                    i64::from(center_x - candidate.widget_x - candidate.width / 2);
                                let dy =
                                    i64::from(center_y - candidate.widget_y - candidate.height / 2);
                                best = best.max(dx * dx + dy * dy);
                            }
                            u32::from(best > 0)
                        }
                        AlienRuntimeRequest::Launch {
                            slot,
                            x,
                            y,
                            center_x,
                            center_y,
                            excluded_prey,
                            ..
                        } => {
                            let mut farthest = None;
                            let mut best = 0_i64;
                            for candidate in prey.iter().filter(|candidate| {
                                candidate.eligible && Some(candidate.id) != excluded_prey
                            }) {
                                if missiles
                                    .iter()
                                    .any(|missile| missile.target_id == Some(candidate.id))
                                {
                                    continue;
                                }
                                let dx =
                                    i64::from(center_x - candidate.widget_x - candidate.width / 2);
                                let dy =
                                    i64::from(center_y - candidate.widget_y - candidate.height / 2);
                                let d2 = dx * dx + dy * dy;
                                if d2 > best {
                                    best = d2;
                                    farthest = Some(candidate.id);
                                }
                            }
                            let Some(target_id) = farthest else {
                                return 0;
                            };
                            let id = next_id;
                            next_id += 1;
                            let visual_draw = Self::advance_rng(&mut rng_state) as u32;
                            missiles.push(if kind == MissileKind::EnergyBall {
                                ClassicMissile::launch_energy(id, target_id, x, y, visual_draw)
                            } else {
                                ClassicMissile::launch(id, target_id, x, y, visual_draw)
                            });
                            launches.push((id, target_id, slot));
                            1
                        }
                    },
                );
                self.rng_state = rng_state;
                self.next_id = next_id;
                for (missile_id, target_id, slot) in launches {
                    if kind == MissileKind::EnergyBall {
                        events.push(Event::EnergyBallLaunched {
                            tick: self.tick,
                            missile_id,
                            target_id,
                            first: slot == 0,
                        });
                    } else {
                        events.push(Event::MissileLaunched {
                            tick: self.tick,
                            missile_id,
                            target_id,
                        });
                    }
                }
                self.record_invasion_events(wave_events, events);
            }
        }
        self.update_bilaterus(events);
        if let Some(wave) = self.invasion.as_mut() {
            let effect_events = wave.update_effects();
            self.record_invasion_events(effect_events, events);
        }
        self.finish_destructor_battle(events);
    }

    fn update_finale_objects(&mut self, events: &mut Vec<Event>) {
        let prey = self.tank5_pet_prey();
        if let Some(wave) = self.invasion.as_mut() {
            let mut rng_state = self.rng_state;
            let mut next_id = self.next_id;
            let changes = wave.update_finale_boss(
                &prey,
                || Self::advance_rng(&mut rng_state) as u32,
                || {
                    let allocated = next_id;
                    next_id += 1;
                    allocated
                },
            );
            self.rng_state = rng_state;
            self.next_id = next_id;
            self.record_invasion_events(changes, events);
        }
        let child_ids = self
            .invasion
            .as_ref()
            .and_then(|wave| wave.finale.as_ref())
            .map_or_else(Vec::new, |finale| {
                finale.children.iter().map(|child| child.id).collect()
            });
        for id in child_ids {
            let prey = self.tank5_pet_prey();
            if let Some(wave) = self.invasion.as_mut() {
                let mut rng_state = self.rng_state;
                let changes = wave
                    .update_finale_child(id, &prey, || Self::advance_rng(&mut rng_state) as u32);
                self.rng_state = rng_state;
                self.record_invasion_events(changes, events);
            }
        }
    }

    fn update_rhubarb(&mut self, events: &mut Vec<Event>) {
        if self.tank == 5 {
            let mut rng_state = self.rng_state;
            for pet in &mut self.rhubarb {
                pet.begin_tick_tank5();
                pet.finish_tick_tank5(&mut |upper| Self::advance_rng(&mut rng_state) % upper);
            }
            self.rng_state = rng_state;
            return;
        }
        if self.rhubarb.is_empty() {
            return;
        }
        let alien_registered = self
            .invasion
            .as_ref()
            .is_some_and(Invasion1_2::has_live_alien);
        let mut prey = self
            .fish
            .iter()
            .filter(|actor| actor.alive)
            .map(|actor| RhubarbPrey {
                id: actor.id,
                widget_x: actor.x as i32,
                widget_y: actor.y as i32,
                ultra: false,
            })
            .collect::<Vec<_>>();
        prey.extend(
            self.oscars
                .iter()
                .filter(|actor| actor.alive)
                .map(|actor| RhubarbPrey {
                    id: actor.id,
                    widget_x: actor.widget_x,
                    widget_y: actor.widget_y,
                    ultra: false,
                }),
        );
        prey.extend(
            self.gekkos
                .iter()
                .filter(|actor| actor.alive)
                .map(|actor| RhubarbPrey {
                    id: actor.id,
                    widget_x: actor.widget_x,
                    widget_y: actor.widget_y,
                    ultra: false,
                }),
        );
        prey.extend(
            self.breeders
                .iter()
                .filter(|actor| actor.alive)
                .map(|actor| RhubarbPrey {
                    id: actor.id,
                    widget_x: actor.widget_x,
                    widget_y: actor.widget_y,
                    ultra: false,
                }),
        );
        prey.extend(
            self.ultras
                .iter()
                .filter(|actor| actor.alive)
                .map(|actor| RhubarbPrey {
                    id: actor.id,
                    widget_x: actor.widget_x,
                    widget_y: actor.widget_y,
                    ultra: true,
                }),
        );
        for index in 0..self.rhubarb.len() {
            let pet = &mut self.rhubarb[index];
            let pet_id = pet.id;
            let contact = pet.begin_tick(&prey, alien_registered);
            if contact.specialty_started {
                events.push(Event::RhubarbSpecialtyStarted {
                    tick: self.tick,
                    pet_id,
                });
            }
            for target_id in contact.pushed_ids {
                let vy = if self.rand_range(2) == 0 { -20 } else { -30 };
                if let Some(actor) = self
                    .fish
                    .iter_mut()
                    .find(|actor| actor.id == target_id && actor.alive)
                {
                    actor.bought_timer = 50;
                    actor.vy = vy as f32;
                } else if let Some(actor) = self
                    .oscars
                    .iter_mut()
                    .find(|actor| actor.id == target_id && actor.alive)
                {
                    actor.bought_timer = 50;
                    actor.vy = f64::from(vy);
                } else if let Some(actor) = self
                    .gekkos
                    .iter_mut()
                    .find(|actor| actor.id == target_id && actor.alive)
                {
                    actor.bought_timer = 50;
                    actor.vy = f64::from(vy);
                } else if let Some(actor) = self
                    .breeders
                    .iter_mut()
                    .find(|actor| actor.id == target_id && actor.alive)
                {
                    actor.bought_timer = 50;
                    actor.vy = f64::from(vy);
                } else if let Some(actor) = self
                    .ultras
                    .iter_mut()
                    .find(|actor| actor.id == target_id && actor.alive)
                {
                    actor.bought_timer = 50;
                    actor.vy = f64::from(vy);
                }
                events.push(Event::RhubarbPushed {
                    tick: self.tick,
                    pet_id,
                    target_id,
                    new_vy: vy,
                });
            }
            let mut rng_state = self.rng_state;
            self.rhubarb[index].finish_tick(&mut |upper| Self::advance_rng(&mut rng_state) % upper);
            self.rng_state = rng_state;
        }
    }

    fn stanley_launch_target(&self, x: f64, y: f64) -> Option<u64> {
        let wave = self.invasion.as_ref()?;
        let (center_x, center_y) = (x + 40.0, y + 40.0);
        let mut best = 10_000;
        let mut target = None;
        // PB05 004e5fc0 uses one strict shared score across the group vector
        // and then the ordinary Alien vector. Equal scores retain first order.
        for (id, x, y, eligible) in wave
            .bilaterus
            .iter()
            .map(|group| (group.id, group.widget_x + 40, group.widget_y + 40, true))
            .chain(wave.actors.iter().map(|actor| {
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
            // PB05 004e6071..60e8/004e62be..6335 converts each
            // continuous-center delta to integer, then rounds D and sqrt
            // through float32 before the final integer comparison.
            let dx = (f64::from(x) - center_x) as i32;
            let dy = (f64::from(y) - center_y) as i32;
            let squared = i64::from(dx) * i64::from(dx) + i64::from(dy) * i64::from(dy);
            let score = (f64::from(squared as f32).sqrt() as f32) as i32;
            if score < best {
                best = score;
                target = Some(id);
            }
        }
        target
    }

    fn divert_missile_from_stanley(
        &mut self,
        pet_id: u64,
        widget_x: i32,
        widget_y: i32,
        events: &mut Vec<Event>,
    ) -> bool {
        let mut winner: Option<(usize, u8, i64)> = None;
        for (index, missile) in self.missiles.iter().enumerate() {
            let rank = match missile.kind {
                MissileKind::Classic => 0,
                MissileKind::EnergyBall if !missile.reflected => 1,
                MissileKind::EnergyBall => 2,
                MissileKind::Stanley => continue,
            };
            if missile.immunity_ticks >= 11 || missile.diversion_clock != 0 {
                continue;
            }
            let Some(target) = missile.target_id.and_then(|id| self.missile_prey_view(id)) else {
                continue;
            };
            let dx = i64::from(target.widget_x - missile.widget_x);
            let dy = i64::from(target.widget_y - missile.widget_y);
            let score = dx * dx + dy * dy;
            if winner.is_none_or(|(_, best_rank, best_score)| {
                rank > best_rank || rank == best_rank && score < best_score
            }) {
                winner = Some((index, rank, score));
            }
        }
        if let Some((index, _, _)) = winner {
            let missile_id = self.missiles[index].id;
            self.missiles[index].divert_from_stanley(widget_x + 40, widget_y + 10);
            events.push(Event::StanleyMissileDiverted {
                tick: self.tick,
                missile_id,
                pet_id,
            });
            true
        } else {
            false
        }
    }

    /// A derived contact view preserves physical membership as the sole
    /// authority. Each candidate is reacquired after preceding contacts.
    fn walter_glove_contacts(&mut self, parent_id: u64, center: (i32, i32)) {
        let mut targets = Vec::new();
        targets.extend(
            self.fish
                .iter()
                .filter(|fish| fish.alive)
                .map(|fish| (fish.id, 0u8)),
        );
        targets.extend(
            self.oscars
                .iter()
                .filter(|fish| fish.alive)
                .map(|fish| (fish.id, 1)),
        );
        targets.extend(
            self.ultras
                .iter()
                .filter(|fish| fish.alive)
                .map(|fish| (fish.id, 2)),
        );
        targets.extend(
            self.gekkos
                .iter()
                .filter(|fish| fish.alive)
                .map(|fish| (fish.id, 3)),
        );
        targets.extend(
            self.fish_pets
                .iter()
                .filter(|pet| pet.id != parent_id && pet.kind != FishPetKind::Prego)
                .map(|pet| (pet.id, 4)),
        );
        targets.sort_unstable_by_key(|&(id, _)| id);
        let right = self
            .fish_pets
            .iter()
            .find(|pet| pet.id == parent_id)
            .and_then(|pet| pet.glove)
            .is_some_and(|glove| glove.right);
        let mut hit = false;
        for (id, class) in targets {
            let eligible = match class {
                0 => self
                    .fish
                    .iter()
                    .find(|fish| fish.id == id && fish.alive)
                    .map(|fish| {
                        (
                            (fish.x as i32) + 40,
                            (fish.y as i32) + 40,
                            fish.walter.recoil == 0,
                        )
                    }),
                1 => self
                    .oscars
                    .iter()
                    .find(|fish| fish.id == id && fish.alive)
                    .map(|fish| {
                        (
                            fish.widget_x + 40,
                            fish.widget_y + 40,
                            fish.walter.recoil == 0,
                        )
                    }),
                2 => self
                    .ultras
                    .iter()
                    .find(|fish| fish.id == id && fish.alive)
                    .map(|fish| {
                        (
                            fish.widget_x + 80,
                            fish.widget_y + 80,
                            fish.walter.recoil == 0,
                        )
                    }),
                3 => self
                    .gekkos
                    .iter()
                    .find(|fish| fish.id == id && fish.alive)
                    .map(|fish| {
                        (
                            fish.widget_x + 40,
                            fish.widget_y + 40,
                            fish.walter.recoil == 0,
                        )
                    }),
                _ => self
                    .fish_pets
                    .iter()
                    .find(|pet| {
                        pet.id == id && pet.id != parent_id && pet.kind != FishPetKind::Prego
                    })
                    .map(|pet| {
                        (
                            pet.widget_x + if pet.kind == FishPetKind::Amp { 80 } else { 40 },
                            pet.widget_y + 40,
                            pet.walter.recoil == 0 && (pet.presto_form.is_none() || pet.age > 29),
                        )
                    }),
            };
            let Some((x, y, ready)) = eligible else {
                continue;
            };
            if !ready || (x - center.0).abs() >= 40 || (y - center.1).abs() >= 40 {
                continue;
            }
            let amount = (if self.rand_range(2) == 0 { 15.0 } else { 10.0 })
                * (if right { 1.0 } else { -1.0 });
            match class {
                0 => {
                    if let Some(fish) = self
                        .fish
                        .iter_mut()
                        .find(|fish| fish.id == id && fish.alive)
                    {
                        fish.coin_timer = fish.coin_threshold.saturating_sub(5);
                        fish.walter.hit(amount);
                    }
                }
                1 => {
                    if let Some(fish) = self
                        .oscars
                        .iter_mut()
                        .find(|fish| fish.id == id && fish.alive)
                    {
                        fish.coin_timer = fish.coin_threshold.saturating_sub(5);
                        fish.walter.hit(amount);
                    }
                }
                2 => {
                    if let Some(fish) = self
                        .ultras
                        .iter_mut()
                        .find(|fish| fish.id == id && fish.alive)
                    {
                        fish.coin_timer = fish.coin_threshold.saturating_sub(5);
                        fish.walter.hit(amount);
                    }
                }
                3 => {
                    if let Some(fish) = self
                        .gekkos
                        .iter_mut()
                        .find(|fish| fish.id == id && fish.alive)
                    {
                        fish.coin_timer = fish.coin_threshold.saturating_sub(5);
                        fish.walter.hit(amount);
                    }
                }
                _ => {
                    if let Some(pet) = self.fish_pets.iter_mut().find(|pet| pet.id == id) {
                        match pet.kind {
                            FishPetKind::Vert => pet.coin_timer = 216 - 5,
                            FishPetKind::Shrapnel => pet.coin_timer = pet.bomb_threshold - 5,
                            FishPetKind::Amp => pet.amp_timer = pet.amp_threshold - 5,
                            FishPetKind::Nostradamus => {
                                pet.nostra_elapsed = pet.nostra_threshold - 10
                            }
                            _ => {}
                        }
                        pet.walter.hit(amount);
                    }
                }
            }
            hit = true;
        }
        if hit && let Some(parent) = self.fish_pets.iter_mut().find(|pet| pet.id == parent_id) {
            parent.set_glove_feedback();
        }
    }

    fn update_fish_pets(&mut self, events: &mut Vec<Event>) {
        let physical_ids = self.fish_pets.iter().map(|pet| pet.id).collect::<Vec<_>>();
        if self.tank == 5 {
            for pet_id in physical_ids {
                let Some(index) = self.fish_pets.iter().position(|pet| pet.id == pet_id) else {
                    continue;
                };
                let contact = self.fish_pets[index].begin_common_update();
                if let Some(center) = contact {
                    self.walter_glove_contacts(pet_id, center);
                }
                let mut rng_state = self.rng_state;
                let update = self.fish_pets[index]
                    .tick_tank5(&mut |upper| Self::advance_rng(&mut rng_state) % upper);
                self.rng_state = rng_state;
                if update.walter_return {
                    self.return_presto_after_walter_hit(pet_id, events)
                        .expect("validated flagged FishPet recoil return");
                }
            }
            return;
        }
        for physical_id in physical_ids {
            let Some(index) = self.fish_pets.iter().position(|pet| pet.id == physical_id) else {
                continue;
            };
            // Alien::Update and its removal transaction have already run.
            // Include registered nonpositive-health aliens for Itchy contact;
            // they remain members until the next active alien update.
            let mut aliens = self
                .invasion
                .as_ref()
                .map(|wave| {
                    wave.actors
                        .iter()
                        .map(|actor| PetAlienView {
                            id: actor.id,
                            widget_x: actor.widget_x,
                            widget_y: actor.widget_y,
                            healing: actor.healing,
                            bilaterus: false,
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if let Some(wave) = &self.invasion {
                let mut groups = wave
                    .bilaterus
                    .iter()
                    .map(|group| {
                        PetAlienView {
                            id: group.id,
                            // PB71 reads the group widget, not its newly
                            // promoted child's unpublished pose.
                            widget_x: group.widget_x,
                            widget_y: group.widget_y,
                            healing: false,
                            bilaterus: true,
                        }
                    })
                    .collect::<Vec<_>>();
                groups.append(&mut aliens);
                aliens = groups;
            }
            let guppy_count = self.fish.iter().filter(|fish| fish.alive).count();
            let pet_id = physical_id;
            let contact = self.fish_pets[index].begin_common_update();
            if let Some(center) = contact {
                self.walter_glove_contacts(pet_id, center);
            }
            if self.fish_pets[index].kind == FishPetKind::Gash {
                self.fish_pets[index].defer_gash_tail();
            }
            let mut rng_state = self.rng_state;
            let update = if self.fish_pets[index].kind == FishPetKind::Stanley {
                let (widget_x, widget_y) = (
                    self.fish_pets[index].widget_x,
                    self.fish_pets[index].widget_y,
                );
                if self.fish_pets[index].begin_stanley()
                    && self.divert_missile_from_stanley(pet_id, widget_x, widget_y, events)
                {
                    let draw = Self::advance_rng(&mut rng_state) % 100;
                    self.fish_pets[index].commit_stanley_diversion(draw);
                }
                if self.fish_pets[index].finish_stanley_action()
                    && let Some(target_id) =
                        self.stanley_launch_target(self.fish_pets[index].x, self.fish_pets[index].y)
                {
                    let missile_id = self.id();
                    let visual_draw = Self::advance_rng(&mut rng_state) as u32;
                    let launch_x = if self.fish_pets[index].vx >= 0.0 {
                        widget_x + 40
                    } else {
                        widget_x
                    };
                    self.missiles.push(ClassicMissile::launch_stanley(
                        missile_id,
                        target_id,
                        launch_x,
                        widget_y,
                        visual_draw,
                    ));
                    events.push(Event::StanleyMissileLaunched {
                        tick: self.tick,
                        missile_id,
                        target_id,
                        pet_id,
                    });
                }
                self.fish_pets[index]
                    .finish_stanley(&mut |upper| Self::advance_rng(&mut rng_state) % upper)
            } else if self.fish_pets[index].kind == FishPetKind::Nostradamus {
                if let Some((x, y)) = self.fish_pets[index].begin_nostradamus(aliens.is_empty()) {
                    let food_id = self.id();
                    // 005434e0 constructs Food before the common FishTypePet
                    // motion tail, whose own animation may draw from RNG.
                    let animation_period = (Self::advance_rng(&mut rng_state) % 2) as u8 + 3;
                    self.food.push(Food {
                        id: food_id,
                        x: x as f32,
                        y: y as f32,
                        frame: 0,
                        food_type: FoodType::Nostradamus,
                        picked_up: false,
                        ineligible_ticks: 20,
                        removal_ticks: 0,
                        quality: 0,
                        direction: 0,
                        vx: 0.0,
                        vy: 0.0,
                        animation_period,
                        free_from_zorf: false,
                        nimbus_rising: false,
                    });
                    events.push(Event::NostradamusFoodDropped {
                        tick: self.tick,
                        pet_id,
                        food_id,
                    });
                }
                self.fish_pets[index]
                    .finish_nostradamus(&mut |upper| Self::advance_rng(&mut rng_state) % upper)
            } else if self.fish_pets[index].kind == FishPetKind::Brinkley {
                let threats_empty = self
                    .invasion
                    .as_ref()
                    .is_none_or(|wave| wave.actors.is_empty() && wave.bilaterus.is_empty());
                let foods = self
                    .food
                    .iter()
                    .map(|food| BrinkleyFoodView {
                        id: food.id,
                        widget_x: food.x as i32,
                        widget_y: food.y as i32,
                    })
                    .collect::<Vec<_>>();
                let action = self.fish_pets[index].begin_brinkley(threats_empty, &foods);
                // Slot 86 scans independently of the selected target and
                // commits removal/payout before common motion and its RNG.
                if action.target_id.is_some() {
                    let center_x = self.fish_pets[index].x + 40.0;
                    let center_y = self.fish_pets[index].y + 40.0;
                    if let Some(position) = foods.iter().position(|food| {
                        let x = f64::from(food.widget_x);
                        let y = f64::from(food.widget_y);
                        x + 5.0 < center_x
                            && center_x < x + 35.0
                            && y < center_y
                            && center_y < y + 35.0
                    }) {
                        let food_id = self.food.remove(position).id;
                        let payout = self.fish_pets[index].commit_brinkley_meal(false);
                        events.push(Event::BrinkleyFoodEaten {
                            tick: self.tick,
                            pet_id,
                            food_id,
                            meals: self.fish_pets[index].brinkley_meals,
                        });
                        if let Some(raw) = payout {
                            let kind = match raw {
                                10 => CoinKind::Star,
                                11 => CoinKind::Diamond,
                                13 => CoinKind::Pearl,
                                14 => CoinKind::Treasure,
                                _ => unreachable!("Brinkley payout kind"),
                            };
                            let coin_id = self.id();
                            self.coins.push(Coin {
                                id: coin_id,
                                x: f64::from(self.fish_pets[index].widget_x + 5),
                                y: f64::from(self.fish_pets[index].widget_y - 10),
                                kind,
                                frame: 0,
                                animation_ticks: 0,
                                hazard_age_ticks: 0,
                                collecting: false,
                                bottom_ticks: 0,
                                fade_ticks: 0,
                                penta_rising: false,
                            });
                            events.push(Event::BrinkleyCoinDropped {
                                tick: self.tick,
                                pet_id,
                                coin_id,
                                kind,
                            });
                        }
                    }
                }
                self.fish_pets[index].finish_brinkley(action.admitted, &mut |upper| {
                    Self::advance_rng(&mut rng_state) % upper
                })
            } else if self.fish_pets[index].kind == FishPetKind::Amp {
                // The installed charge predicate excludes registered aliens
                // and Bilaterus, independently of missile-only mouse blocking.
                self.fish_pets[index].tick_amp(self.tank != 5 && aliens.is_empty(), &mut |upper| {
                    Self::advance_rng(&mut rng_state) % upper
                })
            } else if self.fish_pets[index].kind == FishPetKind::Angie {
                let corpses = self.angie_corpse_views();
                self.fish_pets[index].tick_angie(&corpses, &mut |upper| {
                    Self::advance_rng(&mut rng_state) % upper
                })
            } else if self.fish_pets[index].kind == FishPetKind::Gash {
                let fish = self
                    .fish
                    .iter()
                    .filter(|fish| fish.alive)
                    .map(|fish| GashFishView {
                        id: fish.id,
                        widget_x: fish.x as i32,
                        widget_y: fish.y as i32,
                    })
                    .collect::<Vec<_>>();
                self.fish_pets[index].tick_gash(&aliens, &fish, &mut |upper| {
                    Self::advance_rng(&mut rng_state) % upper
                })
            } else if self.fish_pets[index].kind == FishPetKind::Nimbus {
                let penta = self.starcatchers.iter().any(|actor| actor.alive);
                let coins = self
                    .coins
                    .iter()
                    .map(|coin| NimbusCoinView {
                        id: coin.id,
                        widget_x: coin.x as i32,
                        widget_y: coin.y as i32,
                        eligible: !coin.collecting
                            && coin.kind != CoinKind::ShrapnelBomb
                            && !(penta
                                && matches!(coin.kind, CoinKind::Star | CoinKind::ShellStar)),
                    })
                    .collect::<Vec<_>>();
                let foods = self
                    .food
                    .iter()
                    .map(|food| NimbusFoodView {
                        id: food.id,
                        widget_x: food.x as i32,
                        widget_y: food.y as i32,
                        eligible: food.direction == 0,
                    })
                    .collect::<Vec<_>>();
                self.fish_pets[index].tick_nimbus(
                    &coins,
                    &foods,
                    !aliens.is_empty(),
                    &mut |upper| Self::advance_rng(&mut rng_state) % upper,
                )
            } else if self.fish_pets[index].kind == FishPetKind::Wadsworth {
                let fish = self
                    .fish
                    .iter()
                    .filter(|fish| fish.alive)
                    .map(|fish| WardFishView {
                        widget_x: fish.x as i32,
                        widget_y: fish.y as i32,
                        small_or_medium: matches!(fish.size, FishSize::Small | FishSize::Medium),
                    })
                    .collect::<Vec<_>>();
                let threat_present = !aliens.is_empty() || !self.missiles.is_empty();
                self.fish_pets[index].tick_wadsworth(threat_present, &fish, &mut |upper| {
                    Self::advance_rng(&mut rng_state) % upper
                })
            } else if self.fish_pets[index].kind == FishPetKind::Zorf {
                let mut hungry = self
                    .fish
                    .iter()
                    .filter(|fish| fish.alive)
                    .map(|fish| ZorfHungryView {
                        id: fish.id,
                        hunger: fish.hunger,
                        ordinary_diet: true,
                    })
                    .collect::<Vec<_>>();
                hungry.extend(
                    self.breeders
                        .iter()
                        .filter(|actor| actor.alive)
                        .map(|actor| ZorfHungryView {
                            id: actor.id,
                            hunger: actor.hunger,
                            ordinary_diet: true,
                        }),
                );
                self.fish_pets[index].tick_zorf(&aliens, &hungry, &mut |upper| {
                    Self::advance_rng(&mut rng_state) % upper
                })
            } else {
                self.fish_pets[index].tick(&aliens, guppy_count, &mut |upper| {
                    Self::advance_rng(&mut rng_state) % upper
                })
            };
            self.rng_state = rng_state;
            let mut forced_return = update.walter_return;
            if let Some(corpse_id) = update.angie_revive {
                self.mark_corpse_for_revival(pet_id, corpse_id, events);
            }
            if update.amp_became_ready {
                events.push(Event::AmpReady {
                    tick: self.tick,
                    pet_id,
                });
            }
            if let Some(request) = update.nimbus_coin
                && let Some(position) = self
                    .coins
                    .iter()
                    .position(|coin| coin.id == request.coin_id)
            {
                let old = self.coins.remove(position);
                let coin_id = self.id();
                let kind = old.kind.nimbus_kind();
                self.coins.push(Coin {
                    id: coin_id,
                    x: f64::from(request.x),
                    y: f64::from(request.y as i32),
                    kind,
                    frame: 0,
                    animation_ticks: 0,
                    hazard_age_ticks: old.hazard_age_ticks,
                    collecting: false,
                    bottom_ticks: 0,
                    fade_ticks: 0,
                    penta_rising: true,
                });
                events.push(Event::NimbusCoinConverted {
                    tick: self.tick,
                    pet_id,
                    old_id: old.id,
                    coin_id,
                    kind,
                });
            }
            if let Some(request) = update.nimbus_food
                && let Some(position) = self.food.iter().position(|food| food.id == request.food_id)
            {
                let old = self.food.remove(position);
                // 00543280 counts only ordinary, undirected Food. The old
                // exotic entry is removed before this capacity check, so a
                // full ordinary list can reject Nimbus' fresh replacement.
                let paid_slots = self
                    .food
                    .iter()
                    .filter(|food| food.direction == 0 && food.food_type == FoodType::Ordinary)
                    .count();
                if paid_slots >= usize::from(self.upgrades.quantity) {
                    // 0053c280 refunds the Board's fixed food price even
                    // though this caller did not pay. It writes cash only.
                    let peaceful = self
                        .invasion
                        .as_ref()
                        .is_none_or(|wave| wave.actors.is_empty() && wave.bilaterus.is_empty());
                    if !self.potion_armed && peaceful {
                        self.balance = (self.balance + FOOD_PRICE).min(9_999_999);
                    }
                } else {
                    let food_id = self.id();
                    let animation_period = self.rand_range(2) as u8 + 3;
                    self.food.push(Food {
                        id: food_id,
                        x: request.x as f32,
                        y: (request.y as i32) as f32,
                        frame: 0,
                        food_type: FoodType::Ordinary,
                        picked_up: false,
                        ineligible_ticks: 0,
                        removal_ticks: 0,
                        quality: old.quality,
                        direction: 0,
                        vx: 0.0,
                        vy: -2.0,
                        animation_period,
                        free_from_zorf: false,
                        nimbus_rising: true,
                    });
                    events.push(Event::NimbusFoodConverted {
                        tick: self.tick,
                        pet_id,
                        old_id: old.id,
                        food_id,
                    });
                }
            }
            if self.fish_pets[index].kind == FishPetKind::Blip && self.tick > 200 {
                let missing = if self.tank == 4 {
                    !self.breeder_unlocked
                        || !self.upgrades.quality_unlocked
                        || !self.upgrades.quantity_unlocked
                        || !self.oscar_unlocked
                        || !self.egg_unlocked
                        || (self.level >= 2 && (!self.ultra_unlocked || !self.weapon_unlocked))
                } else {
                    !self.guppy_unlocked
                        || !self.upgrades.quality_unlocked
                        || !self.upgrades.quantity_unlocked
                        || !self.grubber_unlocked
                        || !self.gekko_unlocked
                        || !self.weapon_unlocked
                        || !self.egg_unlocked
                };
                if missing {
                    if self.tank == 4 {
                        self.breeder_unlocked = true;
                        self.oscar_unlocked = true;
                        if self.level >= 2 {
                            self.ultra_unlocked = true;
                            self.weapon_unlocked = true;
                        }
                    } else {
                        self.guppy_unlocked = true;
                        self.grubber_unlocked = true;
                        self.gekko_unlocked = true;
                        self.weapon_unlocked = true;
                    }
                    self.upgrades.quality_unlocked = true;
                    self.upgrades.quantity_unlocked = true;
                    self.egg_unlocked = true;
                    events.push(Event::BlipShopRevealed { tick: self.tick });
                }
            }
            if let Some(active) = update.ward_transition {
                events.push(Event::WadsworthProtectionChanged {
                    tick: self.tick,
                    pet_id,
                    active,
                });
            }
            if let Some(positions) = update.ward_bubbles {
                events.push(Event::WadsworthBubbles {
                    tick: self.tick,
                    pet_id,
                    positions,
                });
            }
            if self.fish_pets[index].kind == FishPetKind::Gash {
                if let Some(alien_id) = update.gash_group_hit {
                    self.gash_hit_alien(pet_id, alien_id, update.punch_sound, events);
                }
                if let Some(alien_id) = update.damaged_alien {
                    self.gash_hit_alien(pet_id, alien_id, update.punch_sound, events);
                }
                if let Some(guppy_id) = update.gash_guppy_remove
                    && let Some(position) = self
                        .fish
                        .iter()
                        .position(|fish| fish.id == guppy_id && fish.alive)
                {
                    self.fish.remove(position);
                    self.fish_pets[index].commit_gash_meal();
                    events.push(Event::GashAteGuppy {
                        tick: self.tick,
                        pet_id,
                        guppy_id,
                    });
                    if self.detach_missile_target(guppy_id, events) {
                        self.finish_destructor_battle(events);
                    }
                }
                // PB70/W1 Update: DropCoin observes membership AFTER contact.
                let clock_allowed = self.tank != 5
                    && self
                        .invasion
                        .as_ref()
                        .is_none_or(|wave| wave.actors.is_empty() && wave.bilaterus.is_empty());
                self.fish_pets[index].finish_gash_clock(clock_allowed);
                let mut tail_rng = self.rng_state;
                forced_return = self.fish_pets[index]
                    .finish_gash_tail(&mut |upper| Self::advance_rng(&mut tail_rng) % upper);
                self.rng_state = tail_rng;
            } else if let Some(alien_id) = update.damaged_alien
                && let Some(wave) = self.invasion.as_mut()
                && let Some(health) = if wave.bilaterus_by_id(alien_id).is_some() {
                    wave.pet_hit_bilaterus(alien_id, 1.0)
                } else {
                    wave.actor_by_id_mut(alien_id)
                        .and_then(|actor| actor.itchy_hit())
                }
            {
                let sound = update.punch_sound && self.punch_sound_cooldown == 0;
                if sound {
                    self.punch_sound_cooldown = 11;
                }
                events.push(Event::FishPetHit {
                    tick: self.tick,
                    pet_id,
                    alien_id,
                    health,
                    sound,
                });
            }
            if let Some((x, y)) = update.born_at {
                let fish = self.make_fish(x as f32, y as f32, false, false);
                let fish_id = fish.id;
                self.fish.push(fish);
                events.push(Event::PregoBirth {
                    tick: self.tick,
                    pet_id,
                    fish_id,
                    x,
                    y,
                });
            }
            if let Some((x, y)) = update.gold_at {
                let coin_id = self.id();
                self.coins.push(Coin {
                    id: coin_id,
                    x: f64::from(x),
                    y: f64::from(y),
                    kind: CoinKind::Gold,
                    frame: 0,
                    animation_ticks: 0,
                    hazard_age_ticks: 0,
                    collecting: false,
                    bottom_ticks: 0,
                    fade_ticks: 0,
                    penta_rising: false,
                });
                events.push(Event::VertGoldDropped {
                    tick: self.tick,
                    pet_id,
                    coin_id,
                });
            }
            if let Some((x, y)) = update.bomb_at {
                let coin_id = self.id();
                self.coins.push(Coin {
                    id: coin_id,
                    x: f64::from(x),
                    y: f64::from(y),
                    kind: CoinKind::ShrapnelBomb,
                    frame: 0,
                    animation_ticks: 0,
                    hazard_age_ticks: 0,
                    collecting: false,
                    bottom_ticks: 0,
                    fade_ticks: 0,
                    penta_rising: false,
                });
                events.push(Event::ShrapnelBombDropped {
                    tick: self.tick,
                    pet_id,
                    coin_id,
                });
            }
            if let Some((x, y)) = update.note_at {
                let note_id = self.id();
                self.notes.push(NoteState {
                    id: note_id,
                    x,
                    y,
                    age_ticks: 0,
                });
                events.push(Event::MerylNoteDropped {
                    tick: self.tick,
                    pet_id,
                    note_id,
                });
            }
            if let Some(drop) = update.free_food {
                let food_id = self.id();
                let animation_period = self.rand_range(2) as u8 + 3;
                self.food.push(Food {
                    id: food_id,
                    x: drop.x as f32,
                    y: drop.y as f32,
                    frame: 0,
                    food_type: FoodType::Ordinary,
                    picked_up: false,
                    ineligible_ticks: 0,
                    removal_ticks: 0,
                    quality: 1,
                    direction: drop.direction,
                    vx: if drop.direction == 1 { -3.0 } else { 3.0 },
                    vy: -2.0,
                    animation_period,
                    free_from_zorf: true,
                    nimbus_rising: false,
                });
                events.push(Event::ZorfFoodDropped {
                    tick: self.tick,
                    pet_id,
                    food_id,
                });
            }
            if forced_return {
                self.return_presto_after_walter_hit(pet_id, events)
                    .expect("validated flagged FishPet recoil return");
            }
        }
    }

    fn angie_corpse_views(&self) -> Vec<AngieCorpseView> {
        let mut views = self
            .dead_fish
            .iter()
            .map(|corpse| AngieCorpseView {
                id: corpse.id,
                widget_x: corpse.x as i32,
                widget_y: corpse.y as i32,
                remaining_ticks: i32::from(corpse.remaining_ticks),
                revival_ticks: corpse.revival_ticks,
                ultra: false,
            })
            .collect::<Vec<_>>();
        // Explicit category ordering is the existing adapter; original shared
        // DeadFish-list ordering across categories has not been recovered.
        views.extend(self.dead_oscars.iter().map(|corpse| AngieCorpseView {
            id: corpse.id,
            widget_x: corpse.widget_x,
            widget_y: corpse.widget_y,
            remaining_ticks: i32::from(corpse.remaining_ticks),
            revival_ticks: corpse.revival_ticks,
            ultra: false,
        }));
        views.extend(self.dead_starcatchers.iter().map(|corpse| AngieCorpseView {
            id: corpse.id,
            widget_x: corpse.widget_x,
            widget_y: corpse.widget_y,
            remaining_ticks: i32::from(corpse.remaining_ticks),
            revival_ticks: corpse.revival_ticks,
            ultra: false,
        }));
        views.extend(self.dead_grubbers.iter().map(|corpse| AngieCorpseView {
            id: corpse.id,
            widget_x: corpse.widget_x,
            widget_y: corpse.widget_y,
            remaining_ticks: i32::from(corpse.remaining_ticks),
            revival_ticks: corpse.revival_ticks,
            ultra: false,
        }));
        views.extend(self.dead_gekkos.iter().map(|corpse| AngieCorpseView {
            id: corpse.id,
            widget_x: corpse.widget_x,
            widget_y: corpse.widget_y,
            remaining_ticks: i32::from(corpse.remaining_ticks),
            revival_ticks: corpse.revival_ticks,
            ultra: false,
        }));
        views.extend(self.dead_breeders.iter().map(|corpse| AngieCorpseView {
            id: corpse.id,
            widget_x: corpse.widget_x,
            widget_y: corpse.widget_y,
            remaining_ticks: i32::from(corpse.remaining_ticks),
            revival_ticks: corpse.revival_ticks,
            ultra: false,
        }));
        views.extend(self.dead_ultras.iter().map(|corpse| AngieCorpseView {
            id: corpse.id,
            widget_x: corpse.widget_x,
            widget_y: corpse.widget_y,
            remaining_ticks: i32::from(corpse.remaining_ticks),
            revival_ticks: corpse.revival_ticks,
            ultra: true,
        }));
        views
    }

    fn mark_corpse_for_revival(&mut self, pet_id: u64, corpse_id: u64, events: &mut Vec<Event>) {
        let timer = if let Some(corpse) = self
            .dead_fish
            .iter_mut()
            .find(|corpse| corpse.id == corpse_id && corpse.remaining_ticks > 95)
        {
            Some(&mut corpse.revival_ticks)
        } else if let Some(corpse) = self
            .dead_oscars
            .iter_mut()
            .find(|corpse| corpse.id == corpse_id && corpse.remaining_ticks > 95)
        {
            Some(&mut corpse.revival_ticks)
        } else if let Some(corpse) = self
            .dead_starcatchers
            .iter_mut()
            .find(|corpse| corpse.id == corpse_id && corpse.remaining_ticks > 95)
        {
            Some(&mut corpse.revival_ticks)
        } else if let Some(corpse) = self
            .dead_grubbers
            .iter_mut()
            .find(|corpse| corpse.id == corpse_id && corpse.remaining_ticks > 95)
        {
            Some(&mut corpse.revival_ticks)
        } else if let Some(corpse) = self
            .dead_gekkos
            .iter_mut()
            .find(|corpse| corpse.id == corpse_id && corpse.remaining_ticks > 95)
        {
            Some(&mut corpse.revival_ticks)
        } else if let Some(corpse) = self
            .dead_breeders
            .iter_mut()
            .find(|corpse| corpse.id == corpse_id && corpse.remaining_ticks > 95)
        {
            Some(&mut corpse.revival_ticks)
        } else {
            self.dead_ultras
                .iter_mut()
                .find(|corpse| corpse.id == corpse_id && corpse.remaining_ticks > 95)
                .map(|corpse| &mut corpse.revival_ticks)
        };
        if let Some(timer) = timer
            && *timer == 100
        {
            *timer = 10;
            events.push(Event::AngieRevivalStarted {
                tick: self.tick,
                pet_id,
                corpse_id,
            });
        }
    }

    fn gash_hit_alien(
        &mut self,
        pet_id: u64,
        alien_id: u64,
        punch_sound: bool,
        events: &mut Vec<Event>,
    ) {
        let Some(wave) = self.invasion.as_mut() else {
            return;
        };
        let mut rng_state = self.rng_state;
        let mut next_id = self.next_id;
        let result =
            if let Some(group) = wave.bilaterus.iter_mut().find(|group| group.id == alien_id) {
                let (health, transition) = group.gash_hit();
                let changes = transition.map_or_else(Vec::new, |outcome| {
                    wave.commit_bilaterus_transition(
                        alien_id,
                        outcome,
                        || Self::advance_rng(&mut rng_state) as u32,
                        || {
                            let id = next_id;
                            next_id += 1;
                            id
                        },
                    )
                });
                Some((health, changes))
            } else {
                wave.gash_hit_alien(alien_id)
            };
        self.rng_state = rng_state;
        self.next_id = next_id;
        if let Some((health, changes)) = result {
            let sound = punch_sound && self.punch_sound_cooldown == 0;
            if sound {
                self.punch_sound_cooldown = 9;
            }
            events.push(Event::FishPetHit {
                tick: self.tick,
                pet_id,
                alien_id,
                health,
                sound,
            });
            self.record_invasion_events(changes, events);
            self.finish_destructor_battle(events);
        }
    }

    fn update_niko(&mut self, events: &mut Vec<Event>) {
        for mut niko in std::mem::take(&mut self.niko) {
            let mut rng_state = self.rng_state;
            let niko_events = if self.tank == 5 {
                niko.tick_tank5(&mut |upper| Self::advance_rng(&mut rng_state) % upper)
            } else {
                niko.tick(&mut |upper| Self::advance_rng(&mut rng_state) % upper)
            };
            self.rng_state = rng_state;
            for event in niko_events {
                if let NikoEvent::PearlSpawn { owner_id, x, y } = event {
                    let pearl_id = self.id();
                    self.pearls.push(NikoPearl::spawn(pearl_id, owner_id, x, y));
                }
                events.push(Event::Niko {
                    tick: self.tick,
                    event,
                });
            }
            self.niko.push(niko);
        }
    }

    fn update_pearls(&mut self, events: &mut Vec<Event>) {
        for pearl in &mut self.pearls {
            match pearl.tick() {
                PearlUpdate::Expired => events.push(Event::PearlExpired {
                    tick: self.tick,
                    pearl_id: pearl.id,
                }),
                PearlUpdate::Credited { owner_id, amount } => {
                    self.balance = (self.balance + amount).min(9_999_999);
                    events.push(Event::PearlCredited {
                        tick: self.tick,
                        pearl_id: pearl.id,
                        owner_id,
                        amount,
                        balance: self.balance,
                    });
                }
                PearlUpdate::Alive | PearlUpdate::Finished => {}
            }
        }
        self.pearls
            .retain(|pearl| pearl.phase != PearlPhase::Finished);
    }

    fn update_stinky(&mut self, events: &mut Vec<Event>) {
        let alien_live = self
            .invasion
            .as_ref()
            .is_some_and(Invasion1_2::has_live_alien);
        let starcatcher_live = self.starcatchers.iter().any(|actor| actor.alive);
        for mut stinky in std::mem::take(&mut self.stinky) {
            if stinky.presto_form.is_some() && stinky.y < 380.0 {
                stinky.vy += 0.1;
            }
            if let Some(form) = &mut stinky.presto_form {
                form.remaining_ticks = form.remaining_ticks.saturating_sub(1);
            }
            // The installed payload ranks integer coin centers by squared distance
            // from Stinky's double center. W1's recovered source expression differs;
            // equal-distance candidates retain their original coin-list order.
            if self.tank != 5 && !alien_live && (!self.coins.is_empty() || !self.notes.is_empty()) {
                let center_x = stinky.x + 40.0;
                let center_y = stinky.y + 40.0;
                if let Some(target_x) =
                    Self::stinky_target_x(&stinky, &self.coins, &self.notes, starcatcher_live)
                {
                    if stinky.chase_timer > 4 {
                        stinky.chase_timer = 0;
                        let target_x = f64::from(target_x);
                        if center_x > target_x + 48.0 {
                            if stinky.vx > -2.3 {
                                stinky.vx -= 1.0;
                            }
                        } else if center_x > target_x + 40.0 {
                            if stinky.vx > -1.3 {
                                stinky.vx -= 0.5;
                            }
                        } else if center_x > target_x + 36.0 {
                            if stinky.vx > -0.3 {
                                stinky.vx = 0.0;
                            }
                        } else if center_x < target_x + 24.0 {
                            if stinky.vx < 2.3 {
                                stinky.vx += 1.0;
                            }
                        } else if center_x < target_x + 32.0 {
                            if stinky.vx < 1.3 {
                                stinky.vx += 0.5;
                            }
                        } else if center_x < target_x + 36.0 && stinky.vx < 0.3 {
                            stinky.vx = 0.0;
                        }
                    }

                    // ChaseEntity calls overlap on every update with a target,
                    // even when its five-update steering gate has not elapsed.
                    if let Some(index) = self.coins.iter().position(|coin| {
                        if coin.collecting
                            || coin.kind == CoinKind::ShrapnelBomb
                            || (starcatcher_live && coin.kind == CoinKind::Star)
                        {
                            return false;
                        }
                        let x = coin.x.trunc();
                        let y = coin.y.trunc();
                        center_x > x + 16.0
                            && center_x < x + 56.0
                            && center_y > y + 16.0
                            && center_y < y + 56.0
                    }) {
                        let coin = self.coins.remove(index);
                        let amount = coin.kind.value();
                        self.balance = (self.balance + amount).min(9_999_999);
                        stinky.angry_timer = 0;
                        events.push(Event::PetCollectedCoin {
                            tick: self.tick,
                            pet: PetKind::Stinky,
                            coin_id: coin.id,
                            kind: coin.kind,
                            amount,
                            balance: self.balance,
                        });
                        events.push(Event::CoinCredited {
                            tick: self.tick,
                            coin_id: coin.id,
                            kind: coin.kind,
                            amount,
                            balance: self.balance,
                        });
                    }
                }
            } else if self.tank != 5 {
                stinky.target_vx = if stinky.specialty_timer > 0 {
                    0.0
                } else {
                    match stinky.movement_state {
                        0 => 0.0,
                        1 => -0.5,
                        2 => 0.5,
                        _ => stinky.target_vx,
                    }
                };
                if stinky.vx < stinky.target_vx {
                    stinky.vx = (stinky.vx + 0.1).min(stinky.target_vx);
                } else if stinky.vx > stinky.target_vx {
                    stinky.vx = (stinky.vx - 0.1).max(stinky.target_vx);
                }
            }
            if stinky.specialty_timer > 0 {
                stinky.vx = 0.0;
            }

            stinky.movement_state_change_timer += 1;
            stinky.chase_timer = stinky.chase_timer.saturating_add(1);
            if stinky.movement_state_change_timer > 20
                || (stinky.x <= 10.0 && stinky.target_vx <= 0.0)
                || stinky.x >= 540.0
            {
                stinky.movement_state_change_timer = 0;
                if self.rand_range(10) == 0 {
                    stinky.movement_state = self.rand_range(3) as u8;
                }
            }

            if alien_live && self.tank != 5 {
                stinky.specialty_timer = (stinky.specialty_timer + 1).min(9);
                stinky.angry_timer = 0;
            } else {
                stinky.specialty_timer = stinky.specialty_timer.saturating_sub(1);
            }

            stinky.x = stinky.x.clamp(10.0, 550.0);
            let above_floor = stinky.y > 370.0;
            stinky.y = stinky.y.clamp(95.0, 370.0);
            if above_floor {
                stinky.vy = 0.0;
            }
            if stinky.x > 535.0 && stinky.vx > 0.1 {
                stinky.movement_state = 1;
            }
            if stinky.x < 15.0 && stinky.vx < -0.1 {
                stinky.movement_state = 2;
            }
            if stinky.previous_vx < 0.0 && stinky.vx > 0.0 {
                stinky.turn_animation_timer = -20;
            } else if stinky.previous_vx > 0.0 && stinky.vx < 0.0 {
                stinky.turn_animation_timer = 20;
            }
            stinky.turn_animation_timer -= stinky.turn_animation_timer.signum();
            if stinky.turn_animation_timer == 0 {
                if stinky.vx.abs() >= 0.3 {
                    stinky.movement_animation_timer = (stinky.movement_animation_timer + 1) % 20;
                    stinky.frame = stinky.movement_animation_timer / 2;
                } else {
                    stinky.movement_animation_timer = (stinky.movement_animation_timer + 1) % 40;
                    stinky.frame = stinky.movement_animation_timer / 4;
                }
            } else if stinky.turn_animation_timer > 0 {
                stinky.frame = (9 - stinky.turn_animation_timer / 2) as u8;
            } else {
                stinky.frame = (9 + stinky.turn_animation_timer / 2) as u8;
            }
            if stinky.specialty_timer > 0 {
                stinky.frame = stinky.specialty_timer;
            }
            if stinky.vx != stinky.previous_vx
                && stinky.vx != 0.0
                && stinky.previous_vx != 0.0
                && stinky.specialty_timer == 0
            {
                stinky.previous_vx = stinky.vx;
            }
            stinky.x += stinky.vx / 1.2;
            stinky.y += stinky.vy / 1.2;
            self.stinky.push(stinky);
        }
    }

    fn update_clyde(&mut self, events: &mut Vec<Event>) {
        if self.tank == 5 {
            let mut rng_state = self.rng_state;
            for pet in &mut self.clyde {
                pet.tick_tank5(&mut |upper| Self::advance_rng(&mut rng_state) % upper);
            }
            self.rng_state = rng_state;
            return;
        }
        for mut clyde in std::mem::take(&mut self.clyde) {
            let starcatcher_live = self.starcatchers.iter().any(|actor| actor.alive);
            let coin_list_nonempty = !self.coins.is_empty() || !self.notes.is_empty();
            let mut views = self
                .coins
                .iter()
                .map(|coin| ClydeCoinView {
                    id: coin.id,
                    widget_x: coin.x as i32,
                    widget_y: coin.y as i32,
                    eligible: !(coin.collecting
                        || coin.kind == CoinKind::ShrapnelBomb
                        || starcatcher_live && coin.kind == CoinKind::Star),
                    collectible: true,
                })
                .collect::<Vec<_>>();
            views.extend(self.notes.iter().map(|note| ClydeCoinView {
                id: note.id,
                widget_x: note.x,
                widget_y: note.y,
                eligible: true,
                collectible: false,
            }));
            views.sort_by_key(|view| view.id);
            let mut rng_state = self.rng_state;
            let update = clyde.tick(&views, coin_list_nonempty, &mut |upper| {
                Self::advance_rng(&mut rng_state) % upper
            });
            self.rng_state = rng_state;
            if let Some(coin_id) = update.collected_coin
                && let Some(index) = self
                    .coins
                    .iter()
                    .position(|coin| coin.id == coin_id && !coin.collecting)
            {
                let coin = self.coins.remove(index);
                let amount = coin.kind.value();
                self.balance = (self.balance + amount).min(9_999_999);
                events.push(Event::PetCollectedCoin {
                    tick: self.tick,
                    pet: PetKind::Clyde,
                    coin_id,
                    kind: coin.kind,
                    amount,
                    balance: self.balance,
                });
                events.push(Event::CoinCredited {
                    tick: self.tick,
                    coin_id,
                    kind: coin.kind,
                    amount,
                    balance: self.balance,
                });
            }
            self.clyde.push(clyde);
        }
    }

    fn stinky_target_x(
        stinky: &StinkyState,
        coins: &[Coin],
        notes: &[NoteState],
        starcatcher_live: bool,
    ) -> Option<i32> {
        let mut best_distance = 100_000_000_i64;
        let mut best_x = None;
        let mut views = coins
            .iter()
            .filter_map(|coin| {
                (!(coin.collecting
                    || coin.kind == CoinKind::ShrapnelBomb
                    || starcatcher_live && coin.kind == CoinKind::Star))
                    .then_some((coin.id, coin.x.trunc() as i32, coin.y.trunc() as i32))
            })
            .collect::<Vec<_>>();
        views.extend(notes.iter().map(|note| (note.id, note.x, note.y)));
        views.sort_by_key(|(id, _, _)| *id);
        for (_, x, y) in views {
            let dx = ((stinky.x + 40.0) - (f64::from(x) + 40.0)) as i64;
            let dy = ((stinky.y + 40.0) - (f64::from(y) + 40.0)) as i64;
            let distance = dx * dx + dy * dy;
            if distance < best_distance {
                best_distance = distance;
                best_x = Some(x);
            }
        }
        best_x
    }

    fn rise_step(y: f64) -> f64 {
        if y >= 150.0 {
            8.0
        } else if y >= 135.0 {
            2.5
        } else if y >= 130.0 {
            1.5
        } else if y >= 125.0 {
            1.0
        } else {
            0.5
        }
    }

    fn update_coins(&mut self, events: &mut Vec<Event>) {
        let seymour_present = self
            .fish_pets
            .iter()
            .any(|pet| pet.kind == FishPetKind::Seymour);
        let bottom_limit = match (self.tank, self.level) {
            (1, 1) => FIRST_STAGE_COIN_BOTTOM_TICKS,
            (1, 2..=5) => SECOND_STAGE_COIN_BOTTOM_TICKS,
            (2, 1..=5) => SECOND_STAGE_COIN_BOTTOM_TICKS,
            (3, 1..=5) => SECOND_STAGE_COIN_BOTTOM_TICKS,
            (4, 1..=5) | (5, 1) => SECOND_STAGE_COIN_BOTTOM_TICKS,
            _ => unreachable!("coin lifetime for this Adventure stage is not implemented"),
        };
        let bottom_limit = if seymour_present { 200 } else { bottom_limit };
        let mut credited = Vec::new();
        let mut expired = Vec::new();
        for index in 0..self.coins.len() {
            let coin = &mut self.coins[index];
            coin.animation_ticks = (coin.animation_ticks + 1) % 80;
            // General GameObject count; bombs also use it as their arm clock.
            // Nimbus preserves it on replacement for every coin kind.
            coin.hazard_age_ticks = coin.hazard_age_ticks.wrapping_add(1);
            if coin.kind == CoinKind::ShrapnelBomb {
                coin.frame = (coin.animation_ticks / 2) % 5;
            } else {
                // PB60's special-mode predicate is false in ordinary Adventure.
                coin.frame = (coin.animation_ticks / if seymour_present { 4 } else { 2 }) % 10;
            }
            if coin.collecting {
                coin.fade_ticks = 0;
                // Coin::Update tests last tick's integer widget Y before its
                // easing calculation and Move, then credits exactly once.
                if coin.y.trunc() < 40.0 {
                    credited.push((coin.id, coin.kind, coin.kind.value()));
                    continue;
                }
                coin.x += (550.0 - coin.x) / 7.0;
                coin.y += (30.0 - coin.y) / 7.0;
                continue;
            }
            if coin.fade_ticks > 0 {
                coin.fade_ticks -= 1;
                if coin.fade_ticks == 0 {
                    expired.push(coin.id);
                }
                continue;
            }
            if coin.penta_rising {
                if coin.y < 120.0 {
                    // The source flips the phase without moving on this update.
                    coin.penta_rising = false;
                } else {
                    coin.y -= Self::rise_step(coin.y);
                }
                continue;
            }
            coin.y += if seymour_present { 0.8 } else { 1.5 };
            if coin.y >= 370.0 {
                coin.y = 370.0;
                coin.bottom_ticks += 1;
                if coin.bottom_ticks >= bottom_limit {
                    if coin.kind == CoinKind::ShrapnelBomb {
                        expired.push(coin.id);
                    } else {
                        coin.fade_ticks = 5;
                    }
                }
            }
            if coin.kind == CoinKind::ShrapnelBomb && expired.contains(&coin.id) {
                let (coin_id, x, y) = (coin.id, coin.x, coin.y);
                self.explode_bomb(coin_id, None, x, y, events);
                continue;
            }
            if coin.kind == CoinKind::ShrapnelBomb && coin.hazard_age_ticks > 30 {
                let (coin_id, x, y) = (coin.id, coin.x, coin.y);
                if !expired.contains(&coin_id)
                    && let Some(target_id) = self.bomb_contact(x, y, events)
                {
                    expired.push(coin_id);
                    self.explode_bomb(coin_id, Some(target_id), x, y, events);
                }
            }
        }
        for (id, kind, amount) in credited {
            self.coins.retain(|coin| coin.id != id);
            self.balance = (self.balance + amount).min(9_999_999);
            events.push(Event::CoinCredited {
                tick: self.tick,
                coin_id: id,
                kind,
                amount,
                balance: self.balance,
            });
        }
        for id in expired {
            self.coins.retain(|coin| coin.id != id);
            events.push(Event::CoinExpired {
                tick: self.tick,
                coin_id: id,
            });
        }
    }

    fn discharge_amp(&mut self, pet_id: u64, events: &mut Vec<Event>) {
        // PB62/PB66: this list models ordinary Fish class identity -1. Take
        // the live victim snapshot before any removal; other species survive.
        let victims = self
            .fish
            .iter()
            .filter(|fish| fish.alive)
            .cloned()
            .collect::<Vec<_>>();
        let reward_limit = amp_reward_limit(victims.len());
        let mut coin_ids = Vec::new();
        let mut effects = 0;
        let mut detached = false;
        for (index, fish) in victims.iter().enumerate() {
            self.fish.retain(|current| current.id != fish.id);
            self.dead_fish.push(DeadFish::from_live(fish));
            detached |= self.detach_missile_target(fish.id, events);
            events.push(Event::FishDied {
                tick: self.tick,
                fish_id: fish.id,
            });
            if index < reward_limit {
                let coin_id = self.id();
                self.coins.push(Coin {
                    id: coin_id,
                    x: f64::from(fish.x as i32 + 15),
                    y: f64::from(fish.y as i32 + 10),
                    kind: CoinKind::Diamond,
                    frame: 0,
                    animation_ticks: 0,
                    hazard_age_ticks: 0,
                    collecting: false,
                    bottom_ticks: 0,
                    fade_ticks: 0,
                    penta_rising: false,
                });
                coin_ids.push(coin_id);
                events.push(Event::CoinDropped {
                    tick: self.tick,
                    coin_id,
                    fish_id: fish.id,
                    kind: CoinKind::Diamond,
                });
            }
            // The requested type draw is before SpawnShot's shared cap. The
            // safe existing constructor draws alpha only for an actual shot.
            let shot_type = self.rand_range(3) as u8 + 3;
            let laser_count = self.invasion.as_ref().map_or(0, |wave| wave.lasers.len());
            if self.bomb_shots.len() + laser_count < 20 {
                let alpha = self.rand_range(200) as u8 + 50;
                self.bomb_shots.push(BombShot {
                    shot_type,
                    x: fish.x as i32,
                    y: fish.y as i32,
                    age_ticks: 0,
                    frame: 0,
                    delay_ticks: 0,
                    alpha,
                });
                effects += 1;
            }
        }
        if detached {
            self.finish_destructor_battle(events);
        }
        let replacement_id =
            (!self.has_live_fish() && self.tank != 5).then(|| self.spawn_bought_guppy());
        events.push(Event::AmpDischarged {
            tick: self.tick,
            pet_id,
            victim_ids: victims.iter().map(|fish| fish.id).collect(),
            coin_ids,
            effects,
            replacement_id,
        });
    }

    fn bomb_contact(&mut self, x: f64, y: f64, events: &mut Vec<Event>) -> Option<u64> {
        let x = x + 36.0;
        let y = y + 36.0;
        let overlaps = |left: i32, top: i32, size: i32| {
            x > f64::from(left + 10)
                && x < f64::from(left + size - 10)
                && y > f64::from(top + 10)
                && y < f64::from(top + size - 10)
        };
        if let Some(fish) = self
            .fish
            .iter_mut()
            .find(|fish| fish.alive && overlaps(fish.x as i32, fish.y as i32, 80))
        {
            let id = fish.id;
            fish.alive = false;
            self.dead_fish.push(DeadFish::from_live(fish));
            events.push(Event::FishDied {
                tick: self.tick,
                fish_id: id,
            });
            if self.detach_missile_target(id, events) {
                self.finish_destructor_battle(events);
            }
            return Some(id);
        }
        if let Some(pos) = self
            .oscars
            .iter()
            .position(|actor| actor.alive && overlaps(actor.widget_x, actor.widget_y, 80))
        {
            let id = self.oscars[pos].id;
            self.dead_oscars
                .push(DeadOscar::from_impact(&self.oscars[pos]));
            self.oscars.remove(pos);
            events.push(Event::OscarDied {
                tick: self.tick,
                oscar_id: id,
            });
            if self.detach_missile_target(id, events) {
                self.finish_destructor_battle(events);
            }
            return Some(id);
        }
        if let Some(pos) = self
            .gekkos
            .iter()
            .position(|actor| actor.alive && overlaps(actor.widget_x, actor.widget_y, 160))
        {
            let id = self.gekkos[pos].id;
            self.dead_gekkos
                .push(DeadGekko::from_impact(&self.gekkos[pos]));
            self.gekkos.remove(pos);
            events.push(Event::GekkoDied {
                tick: self.tick,
                gekko_id: id,
            });
            if self.detach_missile_target(id, events) {
                self.finish_destructor_battle(events);
            }
            return Some(id);
        }
        if let Some(pos) = self
            .starcatchers
            .iter()
            .position(|actor| actor.alive && overlaps(actor.widget_x, actor.widget_y, 160))
        {
            let id = self.starcatchers[pos].id;
            self.dead_starcatchers
                .push(DeadStarcatcher::from_impact(&self.starcatchers[pos]));
            self.starcatchers.remove(pos);
            events.push(Event::StarcatcherDied {
                tick: self.tick,
                starcatcher_id: id,
            });
            if self.detach_missile_target(id, events) {
                self.finish_destructor_battle(events);
            }
            return Some(id);
        }
        if let Some(pos) = self
            .grubbers
            .iter()
            .position(|actor| actor.alive && overlaps(actor.widget_x, actor.widget_y, 160))
        {
            let id = self.grubbers[pos].id;
            self.dead_grubbers
                .push(DeadGrubber::from_impact(&self.grubbers[pos]));
            self.grubbers.remove(pos);
            events.push(Event::GrubberDied {
                tick: self.tick,
                grubber_id: id,
            });
            if self.detach_missile_target(id, events) {
                self.finish_destructor_battle(events);
            }
            return Some(id);
        }
        if let Some(pos) = self
            .breeders
            .iter()
            .position(|actor| actor.alive && overlaps(actor.widget_x, actor.widget_y, 80))
        {
            let id = self.breeders[pos].id;
            self.dead_breeders
                .push(DeadBreeder::from_impact(&self.breeders[pos]));
            self.breeders.remove(pos);
            events.push(Event::BreederDied {
                tick: self.tick,
                breeder_id: id,
                sound: false,
            });
            if self.detach_missile_target(id, events) {
                self.finish_destructor_battle(events);
            }
            return Some(id);
        }
        if let Some(pos) = self
            .ultras
            .iter()
            .position(|actor| actor.alive && overlaps(actor.widget_x, actor.widget_y, 160))
        {
            let id = self.ultras[pos].id;
            self.dead_ultras
                .push(DeadUltra::from_impact(&self.ultras[pos]));
            self.ultras.remove(pos);
            events.push(Event::UltraDied {
                tick: self.tick,
                id,
            });
            if self.detach_missile_target(id, events) {
                self.finish_destructor_battle(events);
            }
            return Some(id);
        }
        None
    }

    fn explode_bomb(
        &mut self,
        coin_id: u64,
        target_id: Option<u64>,
        x: f64,
        y: f64,
        events: &mut Vec<Event>,
    ) {
        let widget_x = x.trunc() as i32;
        let widget_y = y.trunc() as i32;
        let fragments = self.rand_range(3) as u8 + 2;
        for _ in 0..fragments {
            let shot_type = self.rand_range(3) as u8 + 3;
            let shot_x = widget_x + self.rand_range(30) as i32 - 10;
            let shot_y = widget_y + self.rand_range(30) as i32 - 10;
            // The source constructor reads an uninitialized prior type before
            // assigning the requested one. This Rust path initializes that
            // field to zero: no delay draw, then one alpha draw.
            let alpha = self.rand_range(200) as u8 + 50;
            self.bomb_shots.push(BombShot {
                shot_type,
                x: shot_x,
                y: shot_y,
                age_ticks: 0,
                frame: 0,
                delay_ticks: 0,
                alpha,
            });
        }
        if target_id.is_some() {
            // Contact creates one more Shot after Remove; bottom expiry does
            // not. These two draws follow every fragment constructor draw.
            let shot_type = self.rand_range(3) as u8 + 3;
            let alpha = self.rand_range(200) as u8 + 50;
            self.bomb_shots.push(BombShot {
                shot_type,
                x: widget_x,
                y: widget_y,
                age_ticks: 0,
                frame: 0,
                delay_ticks: 0,
                alpha,
            });
        }
        events.push(Event::ShrapnelBombExploded {
            tick: self.tick,
            coin_id,
            target_id,
            fragments,
        });
    }

    fn update_notes(&mut self, events: &mut Vec<Event>) {
        let mut expired = Vec::new();
        for note in &mut self.notes {
            note.age_ticks += 1;
            if note.age_ticks < 101 && note.y >= 50 {
                note.y -= 1;
            } else {
                expired.push(note.id);
            }
        }
        self.notes.retain(|note| !expired.contains(&note.id));
        for note_id in expired {
            events.push(Event::MerylNoteExpired {
                tick: self.tick,
                note_id,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walter_requires_consumed_pointer_input_and_does_not_auto_punch() {
        let mut board = AdventureState::new_time_trial(0x3952, 1, &[PetKind::Walter]).unwrap();
        board.update_fish_pets(&mut Vec::new());
        let pet = &board.fish_pets[0];
        assert!(pet.glove.is_none());
        assert_eq!(pet.walter_punches, 0);
        board.fish_pets[0].turn_ticks = 0;
        board.fish_pets[0].vx = 1.0;
        let pet = &board.fish_pets[0];
        let (x, y) = (pet.widget_x + 40, pet.widget_y + 40);
        let events = board.apply(Action::Click {
            x: x as f32,
            y: y as f32,
        });
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Event::WalterPunched { .. }))
        );
        assert!(board.fish_pets[0].glove.is_some());
        assert_eq!(board.fish_pets[0].walter_punches, 1);
        let events = board.apply(Action::Click {
            x: x as f32,
            y: y as f32,
        });
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, Event::WalterPunched { .. }))
        );
    }

    #[test]
    fn walter_contact_uses_strict_center_edges_and_current_recoil() {
        let mut board = AdventureState::new_time_trial(0x3950, 1, &[PetKind::Walter]).unwrap();
        let parent = &mut board.fish_pets[0];
        parent.x = 100.0;
        parent.y = 100.0;
        parent.widget_x = 100;
        parent.widget_y = 100;
        parent.published_x = 100;
        parent.published_y = 100;
        parent.walter_action = 35;
        parent.glove = Some(crate::walter::Glove::new(true));
        board.fish[0].x = 120.0;
        board.fish[0].y = 100.0;
        board.fish[1].x = 160.0;
        board.fish[1].y = 100.0;
        board.update_fish_pets(&mut Vec::new());
        assert_eq!(board.fish[0].walter.recoil, 50);
        assert_eq!(board.fish[0].coin_timer, board.fish[0].coin_threshold - 5);
        assert_eq!(board.fish[1].walter.recoil, 0);
        assert_eq!(board.fish_pets[0].glove.unwrap().lifetime, 29);
    }

    #[test]
    fn flagged_walter_returns_after_recoil_40_without_second_pass_tick() {
        let mut board =
            AdventureState::new_time_trial(0x3951, 1, &[PetKind::Presto, PetKind::Itchy]).unwrap();
        let opener = board.presto_actor().unwrap().id;
        board
            .change_presto_form(opener, PetKind::Walter, &mut Vec::new())
            .unwrap();
        board.fish_pets.swap(0, 1);
        let old = board.fish_pets[0].id;
        let peer = board.fish_pets[1].id;
        board.fish_pets[0].walter.recoil = 41;
        board.fish_pets[0].walter.impulse = 10.0;
        board.update_fish_pets(&mut Vec::new());
        assert!(!board.fish_pets.iter().any(|pet| pet.id == old));
        assert_eq!(
            board
                .fish_pets
                .iter()
                .find(|pet| pet.id == peer)
                .unwrap()
                .age,
            1
        );
        let fresh = board
            .fish_pets
            .iter()
            .find(|pet| pet.kind == FishPetKind::Presto)
            .unwrap();
        assert_eq!(fresh.age, 0);
        assert_eq!(fresh.walter.recoil, 0);
        assert!(fresh.glove.is_none());
    }

    #[test]
    fn walter_tank5_commits_glove_contact_and_flagged_return_once() {
        let mut board =
            AdventureState::new_profile_stage(0x3953, 5, 1, &[PetKind::Presto, PetKind::Walter], 0)
                .unwrap();
        let opener = board.presto_actor().unwrap().id;
        board
            .change_presto_form(opener, PetKind::Walter, &mut Vec::new())
            .unwrap();
        let flagged = board.presto_actor().unwrap().id;
        let ordinary = board
            .fish_pets
            .iter()
            .find(|pet| pet.id != flagged && pet.kind == FishPetKind::Walter)
            .unwrap()
            .id;
        {
            let parent = board
                .fish_pets
                .iter_mut()
                .find(|pet| pet.id == ordinary)
                .unwrap();
            parent.widget_x = 100;
            parent.widget_y = 100;
            parent.x = 100.0;
            parent.y = 100.0;
            parent.walter_action = 35;
            parent.glove = Some(crate::walter::Glove::new(true));
        }
        {
            let target = board
                .fish_pets
                .iter_mut()
                .find(|pet| pet.id == flagged)
                .unwrap();
            target.widget_x = 120;
            target.widget_y = 100;
            target.x = 120.0;
            target.y = 100.0;
            target.age = 30;
        }
        let mut events = Vec::new();
        board.update_fish_pets(&mut events);
        assert_eq!(
            board
                .fish_pets
                .iter()
                .find(|pet| pet.id == flagged)
                .unwrap()
                .walter
                .recoil,
            49
        );
        board
            .fish_pets
            .iter_mut()
            .find(|pet| pet.id == ordinary)
            .unwrap()
            .glove = None;
        board
            .fish_pets
            .iter_mut()
            .find(|pet| pet.id == flagged)
            .unwrap()
            .walter
            .recoil = 41;
        board.update_fish_pets(&mut events);
        assert!(!board.fish_pets.iter().any(|pet| pet.id == flagged));
        assert_eq!(board.presto_actor().unwrap().kind, PetKind::Presto);
        assert_eq!(events.iter().filter(|event| matches!(event, Event::PrestoChanged { old_id, .. } if *old_id == flagged)).count(), 1);
        let fresh_id = board.presto_actor().unwrap().id;
        assert_eq!(
            board
                .fish_pets
                .iter()
                .find(|pet| pet.id == fresh_id)
                .unwrap()
                .age,
            0
        );
    }

    #[test]
    fn walter_recoil_one_returns_flagged_form_but_idle_zero_does_not() {
        let mut board = AdventureState::new_time_trial(0x3954, 1, &[PetKind::Presto]).unwrap();
        let opener = board.presto_actor().unwrap().id;
        board
            .change_presto_form(opener, PetKind::Walter, &mut Vec::new())
            .unwrap();
        let old = board.presto_actor().unwrap().id;
        let target = board
            .fish_pets
            .iter_mut()
            .find(|pet| pet.id == old)
            .unwrap();
        target.walter.recoil = 1;
        target.walter.impulse = 10.0;
        board.update_fish_pets(&mut Vec::new());
        assert_eq!(board.presto_actor().unwrap().kind, PetKind::Presto);
        assert_ne!(board.presto_actor().unwrap().id, old);
        let opener = board.presto_actor().unwrap().id;
        board
            .change_presto_form(opener, PetKind::Walter, &mut Vec::new())
            .unwrap();
        let idle = board.presto_actor().unwrap().id;
        board.update_fish_pets(&mut Vec::new());
        assert_eq!(board.presto_actor().unwrap().id, idle);
    }

    #[test]
    fn walter_flagged_gash_owes_meal_and_clock_before_recoil_return() {
        let mut board = AdventureState::new_time_trial(0x3955, 1, &[PetKind::Presto]).unwrap();
        let opener = board.presto_actor().unwrap().id;
        board
            .change_presto_form(opener, PetKind::Gash, &mut Vec::new())
            .unwrap();
        let gash_id = board.presto_actor().unwrap().id;
        let prey_id = board.fish[0].id;
        board.fish[0].x = 100.0;
        board.fish[0].y = 100.0;
        board.fish[0].size = FishSize::Large;
        board.fish[1].x = 500.0;
        board.fish[1].y = 300.0;
        let gash = board
            .fish_pets
            .iter_mut()
            .find(|pet| pet.id == gash_id)
            .unwrap();
        gash.x = 100.0;
        gash.y = 100.0;
        gash.widget_x = 100;
        gash.widget_y = 100;
        gash.gash_timer = 1571;
        gash.walter.recoil = 41;
        gash.walter.impulse = 10.0;
        let mut events = Vec::new();
        board.update_fish_pets(&mut events);
        assert!(events.iter().any(
            |event| matches!(event, Event::GashAteGuppy { guppy_id, .. } if *guppy_id == prey_id)
        ));
        assert!(!board.fish.iter().any(|fish| fish.id == prey_id));
        assert_eq!(board.presto_actor().unwrap().kind, PetKind::Presto);
        assert!(events.iter().any(
            |event| matches!(event, Event::PrestoChanged { old_id, .. } if *old_id == gash_id)
        ));
    }

    #[test]
    fn unflagged_stinky_constructor_uses_primary_anchor_across_adventure_tanks() {
        // PB05 004eb5c0 raw0/flag0→00596ed0=370 has no mode/tank condition.
        let ordinary = AdventureState::new_second_stage(0x370);
        let finale = AdventureState::new_tank5_1(0x370, 0);
        assert_eq!(ordinary.stinky.first().unwrap().y, 370.0);
        assert_eq!(finale.stinky.first().unwrap().y, 370.0);
        assert!(ordinary.validate().is_ok());
        assert!(finale.validate().is_ok());
    }

    #[test]
    fn a35_board_revives_a_fresh_guppy_after_ten_updates_without_credit() {
        // PB74: eligible lifetime is strictly >95; 100->10 is a single
        // transaction, followed by fresh subtype construction, not restoration.
        let mut board = AdventureState::new_tank4_finale(0x4501, &[PetKind::Angie]).unwrap();
        let mut fish = board.make_fish(100.0, 100.0, false, false);
        fish.size = FishSize::Large;
        fish.hunger = 0;
        fish.vx = -1.0;
        fish.alive = false;
        let old_id = fish.id;
        board.dead_fish.push(DeadFish::from_live(&fish));
        board.fish.push(fish);
        board.dead_fish[0].remaining_ticks = 95;
        let pet = &mut board.fish_pets[0];
        pet.x = 100.0;
        pet.y = 100.0;
        pet.widget_x = 100;
        pet.widget_y = 100;
        let mut events = Vec::new();
        board.update_fish_pets(&mut events);
        assert_eq!(board.dead_fish[0].revival_ticks, 100);
        board.dead_fish[0].remaining_ticks = 96;
        let pet = &mut board.fish_pets[0];
        pet.x = 100.0;
        pet.y = 100.0;
        board.update_fish_pets(&mut events);
        assert_eq!(board.dead_fish[0].revival_ticks, 10);
        board.mark_corpse_for_revival(board.fish_pets[0].id, old_id, &mut events);
        assert_eq!(events.iter().filter(|event| matches!(event, Event::AngieRevivalStarted { corpse_id, .. } if *corpse_id == old_id)).count(), 1);
        for remaining in (1..10).rev() {
            board.update_dead_fish(&mut events);
            assert_eq!(board.dead_fish[0].revival_ticks, remaining);
            assert!(board.fish.iter().all(|fish| !fish.alive));
        }
        board.update_dead_fish(&mut events);
        assert!(board.dead_fish.is_empty());
        assert_eq!(board.fish.len(), 1);
        let revived = &board.fish[0];
        assert!(revived.alive && revived.id != old_id);
        assert_eq!(revived.size, FishSize::Large);
        assert!(!revived.facing_right);
        assert!((400..600).contains(&revived.hunger));
        assert_eq!(
            (revived.food_ate, revived.coin_timer, revived.vx),
            (0, 0, -1.0)
        );
        assert_eq!((board.balance, board.coins.len()), (200, 0));
        assert_eq!(events.iter().filter(|event| matches!(event, Event::FishRevived { corpse_id, .. } if *corpse_id == old_id)).count(), 1);
        board.update_dead_fish(&mut events);
        board.validate().unwrap();
    }

    #[test]
    fn a35_expired_body_wins_over_due_revival() {
        // PB74 DeadFishUpdate checks the old remaining lifetime first.
        let mut board = AdventureState::new_tank4_finale(0x4502, &[]).unwrap();
        let mut fish = board.make_fish(100.0, 100.0, false, false);
        fish.alive = false;
        let mut corpse = DeadFish::from_live(&fish);
        corpse.remaining_ticks = 0;
        corpse.revival_ticks = 1;
        board.fish.push(fish);
        board.dead_fish.push(corpse);
        let mut events = Vec::new();
        board.update_dead_fish(&mut events);
        assert!(board.fish.is_empty() && board.dead_fish.is_empty() && events.is_empty());
        board.validate().unwrap();
    }

    #[test]
    fn a35_gash_hits_both_registered_lists_in_group_then_alien_order() {
        // PB71 contact walks Bilaterus then ordinary Alien independently.
        let mut board = AdventureState::new_tank4_finale(0x4503, &[PetKind::Gash]).unwrap();
        let pet = &mut board.fish_pets[0];
        pet.x = 100.0;
        pet.y = 100.0;
        pet.widget_x = 100;
        pet.widget_y = 100;
        let group_id = board.id();
        let alien_id = board.id();
        let group = crate::bilaterus::BilaterusState::spawn(group_id, 100, 100, &mut || 1);
        let before = group.heads[group.active_head].as_ref().unwrap().health;
        let alien =
            crate::alien::WeakSylvester::spawn_kind(SylvesterKind::Balrog, alien_id, 60, 60, 1, 1);
        let alien_before = alien.health;
        let wave = board.invasion.as_mut().unwrap();
        wave.bilaterus.push(group);
        wave.actors.push(alien);
        wave.battle_active = true;
        let mut events = Vec::new();
        board.update_fish_pets(&mut events);
        let wave = board.invasion.as_ref().unwrap();
        let group = &wave.bilaterus[0];
        assert_eq!(
            group.heads[group.active_head].as_ref().unwrap().health,
            before - 3.0
        );
        assert_eq!(wave.actors[0].health, alien_before - 3.0);
        let hits = events
            .iter()
            .filter_map(|event| {
                if let Event::FishPetHit {
                    alien_id, sound, ..
                } = event
                {
                    Some((*alien_id, *sound))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(hits, vec![(group_id, true), (alien_id, false)]);
        assert_eq!(board.punch_sound_cooldown, 9);
        assert_eq!(board.fish_pets[0].gash_timer, -1550);
    }

    #[test]
    fn a35_tank4_mixed_projectiles_validate_without_relaxing_target_ownership() {
        // PB73 paired Destructor/Ulysses can leave both projectile types.
        for level in [4, 5] {
            let mut board = if level == 4 {
                AdventureState::new_tank4_fourth_stage(0x4504, &[]).unwrap()
            } else {
                AdventureState::new_tank4_finale(0x4504, &[]).unwrap()
            };
            let guppy = board.make_fish(200.0, 200.0, false, false);
            let guppy_id = guppy.id;
            board.fish.push(guppy);
            let classic_id = board.id();
            let energy_id = board.id();
            board.missiles.push(ClassicMissile::launch(
                classic_id,
                board.breeders[0].id,
                400,
                100,
                0,
            ));
            board.missiles.push(ClassicMissile::launch_energy(
                energy_id, guppy_id, 400, 100, 0,
            ));
            board.invasion.as_mut().unwrap().countdown = 3000;
            board.invasion.as_mut().unwrap().battle_active = true;
            if level == 5 {
                // Identified controlled state after a completed pair spawn;
                // the pre-first-wave count zero cannot own projectiles.
                board.invasion.as_mut().unwrap().plan = WavePlan::CyclingTank4Finale {
                    next: EncounterKind::Bilaterus,
                    wave_count: 2,
                };
            }
            board.validate().unwrap();
            let restored: AdventureState =
                serde_json::from_value(serde_json::to_value(&board).unwrap()).unwrap();
            restored.validate().unwrap();
            board.missiles[1].target_id = board.missiles[0].target_id;
            assert!(board.validate().unwrap_err().contains("duplicate"));
        }
        let mut earlier = AdventureState::new_tank4_third_stage(0x4505, &[]).unwrap();
        let missile_id = earlier.id();
        earlier.missiles.push(ClassicMissile::launch_energy(
            missile_id,
            earlier.breeders[0].id,
            400,
            100,
            0,
        ));
        earlier.invasion.as_mut().unwrap().countdown = 3000;
        assert!(earlier.validate().is_err());
    }

    fn a34_gash_board() -> AdventureState {
        let mut board = AdventureState::new_tank4_fourth_stage(0x4401, &[PetKind::Gash]).unwrap();
        let gash = &mut board.fish_pets[0];
        gash.x = 100.0;
        gash.y = 100.0;
        gash.widget_x = 100;
        gash.widget_y = 100;
        gash.vx = 0.0;
        gash.vy = 0.0;
        board
    }

    #[test]
    fn a34_entry_uses_primary_price_plan_and_selected_gash_signed_timer() {
        let board = AdventureState::new_tank4_fourth_stage(0x4401, &[PetKind::Gash]).unwrap();
        assert_eq!(
            (board.tank, board.level, board.balance, board.egg_price),
            (4, 4, 200, 75_000)
        );
        assert_eq!(
            (board.breeders.len(), board.breeders[0].food_points),
            (1, 2)
        );
        assert_eq!(board.fish_pets[0].gash_timer, -1550);
        assert_eq!(
            board.invasion.as_ref().unwrap().plan,
            WavePlan::CyclingTank4Fourth {
                next: EncounterKind::Bilaterus
            }
        );
        board.validate().unwrap();
        assert!(AdventureState::new_tank4_third_stage(42, &[PetKind::Gash]).is_err());
        assert!(AdventureState::new_tank4_fourth_stage(42, &[PetKind::Angie]).is_err());
    }

    #[test]
    fn a34_gash_consumption_uses_old_strict_timer_and_removes_without_corpse_or_credit() {
        let mut board = a34_gash_board();
        let mut prey = board.make_fish(100.0, 100.0, false, false);
        prey.size = FishSize::Large;
        prey.cannot_be_eaten_ticks = 10; // PB71 tests class-1, not Oscar's small/immunity predicate.
        let prey_id = prey.id;
        board.fish.push(prey);
        board.breeders[0].x = 100.0;
        board.breeders[0].y = 100.0;
        let breeder = serde_json::to_value(&board.breeders).unwrap();
        let missile_id = board.id();
        board
            .missiles
            .push(ClassicMissile::launch(missile_id, prey_id, 400, 100, 0));
        board.fish_pets[0].gash_timer = 1570;
        let mut events = Vec::new();
        board.update_fish_pets(&mut events);
        assert_eq!(board.fish.len(), 1);
        assert_eq!(board.fish_pets[0].gash_timer, 1571);
        assert!(
            events
                .iter()
                .all(|event| !matches!(event, Event::GashAteGuppy { .. }))
        );
        let gash = &mut board.fish_pets[0];
        gash.x = 100.0;
        gash.y = 100.0;
        board.update_fish_pets(&mut events);
        assert!(board.fish.is_empty());
        assert!(board.dead_fish.is_empty());
        assert!(board.coins.is_empty());
        assert_eq!(board.balance, 200);
        assert!(board.missiles.is_empty());
        assert_eq!(serde_json::to_value(&board.breeders).unwrap(), breeder);
        assert_eq!(board.fish_pets[0].gash_timer, 1);
        assert_eq!(events.iter().filter(|event| matches!(event, Event::GashAteGuppy { guppy_id, .. } if *guppy_id == prey_id)).count(), 1);
        board.update_fish_pets(&mut events);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, Event::GashAteGuppy { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn a34_gash_typed_damage_and_healing_exclusion_use_synchronous_membership() {
        for (kind, damage) in [
            (SylvesterKind::Gus, 0.5),
            (SylvesterKind::Destructor, 0.5),
            (SylvesterKind::Balrog, 3.0),
            (SylvesterKind::Ulysses, 3.0),
            (SylvesterKind::Psychosquid, 3.0),
        ] {
            let mut board = a34_gash_board();
            let pet_id = board.fish_pets[0].id;
            let alien_id = board.id();
            let mut alien = crate::alien::WeakSylvester::spawn_kind(kind, alien_id, 100, 100, 1, 1);
            alien.health = damage;
            let wave = board.invasion.as_mut().unwrap();
            wave.actors.push(alien);
            wave.battle_active = true;
            let mut events = Vec::new();
            board.gash_hit_alien(pet_id, alien_id, true, &mut events);
            assert!(board.invasion.as_ref().unwrap().actors.is_empty());
            assert_eq!(board.punch_sound_cooldown, 9);
            assert_eq!(board.coins.len(), 1);
            assert!(
                events
                    .iter()
                    .any(|event| matches!(event, Event::FishPetHit { health: 0.0, .. }))
            );
            let once = events.len();
            board.gash_hit_alien(pet_id, alien_id, true, &mut events);
            assert_eq!((events.len(), board.coins.len()), (once, 1));
        }
        let mut board = a34_gash_board();
        let pet_id = board.fish_pets[0].id;
        let alien_id = board.id();
        let mut healing = crate::alien::WeakSylvester::spawn_kind(
            SylvesterKind::Psychosquid,
            alien_id,
            100,
            100,
            1,
            1,
        );
        healing.healing = true;
        healing.health = 5.0;
        board.invasion.as_mut().unwrap().actors.push(healing);
        let mut events = Vec::new();
        board.gash_hit_alien(pet_id, alien_id, true, &mut events);
        assert!(events.is_empty());
        assert_eq!(board.invasion.as_ref().unwrap().actors[0].health, 5.0);
    }

    #[test]
    fn a34_gash_group_contacts_transition_during_emergence_and_award_once() {
        let mut board = a34_gash_board();
        let pet_id = board.fish_pets[0].id;
        let alien_id = board.id();
        let mut group = crate::bilaterus::BilaterusState::spawn(alien_id, 100, 100, &mut || 1);
        group.heads[group.active_head].as_mut().unwrap().health = 3.0;
        assert_eq!(group.emergence_ticks, 15);
        let wave = board.invasion.as_mut().unwrap();
        wave.bilaterus.push(group);
        wave.battle_active = true;
        let mut events = Vec::new();
        board.gash_hit_alien(pet_id, alien_id, true, &mut events);
        let wave = board.invasion.as_mut().unwrap();
        assert!(wave.bilaterus[0].first_head_lost);
        assert_eq!((wave.fragments.len(), board.coins.len()), (1, 0));
        let group = &mut wave.bilaterus[0];
        group.heads[group.active_head].as_mut().unwrap().health = 3.0;
        board.gash_hit_alien(pet_id, alien_id, true, &mut events);
        let wave = board.invasion.as_ref().unwrap();
        assert!(wave.bilaterus.is_empty());
        assert_eq!(wave.fragments.len(), 8);
        assert!(!wave.battle_active);
        assert_eq!(
            (board.coins.len(), board.coins[0].kind, board.balance),
            (1, CoinKind::Diamond, 200)
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    Event::Invasion {
                        event: InvasionEvent::BattleEnded,
                        ..
                    }
                ))
                .count(),
            1
        );
        let once = events.len();
        board.gash_hit_alien(pet_id, alien_id, true, &mut events);
        assert_eq!((events.len(), board.coins.len()), (once, 1));
    }

    #[test]
    fn a34_gash_last_enemy_contact_enables_same_update_peaceful_clock() {
        let mut board = a34_gash_board();
        let alien_id = board.id();
        let mut alien = crate::alien::WeakSylvester::spawn_kind(
            SylvesterKind::Destructor,
            alien_id,
            60,
            60,
            1,
            1,
        );
        alien.health = 0.5;
        alien.widget_x = 60;
        alien.widget_y = 60;
        let wave = board.invasion.as_mut().unwrap();
        wave.actors.push(alien);
        wave.battle_active = true;
        board.update_fish_pets(&mut Vec::new());
        assert!(board.invasion.as_ref().unwrap().actors.is_empty());
        assert_eq!(board.fish_pets[0].gash_timer, -1549);
    }

    #[test]
    fn a34_pet_contact_uses_published_group_widget_after_head_promotion() {
        let mut board = a34_gash_board();
        let alien_id = board.id();
        let mut group = crate::bilaterus::BilaterusState::spawn(alien_id, 100, 100, &mut || 1);
        group.first_head_lost = true;
        group.heads[0] = None;
        group.active_head = 1;
        group.heads[1].as_mut().unwrap().widget_x = 400;
        group.heads[1].as_mut().unwrap().widget_y = 300;
        group.heads[1].as_mut().unwrap().health = 100.0;
        group.widget_x = 100;
        group.widget_y = 100;
        let wave = board.invasion.as_mut().unwrap();
        wave.bilaterus.push(group);
        wave.battle_active = true;
        let mut events = Vec::new();
        board.update_fish_pets(&mut events);
        assert_eq!(
            board.invasion.as_ref().unwrap().bilaterus[0].heads[1]
                .as_ref()
                .unwrap()
                .health,
            97.0
        );
        assert!(events.iter().any(
            |event| matches!(event, Event::FishPetHit { alien_id: id, .. } if *id == alien_id)
        ));
    }

    fn a33_ready_amp(guppies: usize) -> AdventureState {
        let mut board = AdventureState::new_tank4_third_stage(0x4301, &[PetKind::Amp]).unwrap();
        let amp = &mut board.fish_pets[0];
        amp.x = 100.0;
        amp.y = 100.0;
        amp.widget_x = 100;
        amp.widget_y = 100;
        amp.published_x = 100;
        amp.published_y = 100;
        amp.amp_timer = 3000;
        for _ in 0..guppies {
            let fish = board.make_fish(300.0, 240.0, false, false);
            board.fish.push(fish);
        }
        board
    }

    fn a33_tap(board: &mut AdventureState) -> Vec<Event> {
        board.apply(Action::Click { x: 120.0, y: 120.0 })
    }

    #[test]
    fn a33_amp_third_routed_tap_uses_primary_snapshot_cap_and_real_coin_entities() {
        // PB66 disagrees with W1's n-5 formula. These fixed independently
        // recovered boundaries also check every victim dies despite the cap.
        for (count, rewards) in [(0, 0), (20, 20), (21, 20), (22, 21), (40, 30)] {
            let mut board = a33_ready_amp(count);
            let victim_ids = board.fish.iter().map(|fish| fish.id).collect::<Vec<_>>();
            let breeder_id = board.breeders[0].id;
            let initial_cash = board.balance;
            assert!(
                a33_tap(&mut board)
                    .iter()
                    .any(|event| matches!(event, Event::AmpCharged { charge: 1, .. }))
            );
            assert!(
                a33_tap(&mut board)
                    .iter()
                    .any(|event| matches!(event, Event::AmpCharged { charge: 2, .. }))
            );
            assert_eq!(board.fish.len(), count);
            let events = a33_tap(&mut board);
            assert!(board.fish.is_empty());
            assert_eq!(
                board
                    .dead_fish
                    .iter()
                    .map(|fish| fish.id)
                    .collect::<Vec<_>>(),
                victim_ids
            );
            assert_eq!(board.breeders[0].id, breeder_id);
            assert_eq!(
                board.balance, initial_cash,
                "discharge creates entities, not cash"
            );
            assert_eq!(board.coins.len(), rewards);
            assert!(board.coins.iter().all(|coin| coin.kind == CoinKind::Diamond
                && coin.kind.value() == 200
                && !coin.collecting));
            assert_eq!(board.bomb_shots.len(), count.min(20));
            assert!(
                board
                    .bomb_shots
                    .iter()
                    .all(|shot| (3..=5).contains(&shot.shot_type))
            );
            assert!(events.iter().any(|event| matches!(event, Event::AmpDischarged { victim_ids: killed, coin_ids, replacement_id: None, .. } if killed == &victim_ids && coin_ids.len() == rewards)));
            assert_eq!(
                (
                    board.fish_pets[0].amp_timer,
                    board.fish_pets[0].amp_threshold,
                    board.fish_pets[0].amp_charge
                ),
                (-20, 3200, 0)
            );
            board.validate().unwrap();
        }
    }

    #[test]
    fn a33_amp_fallback_requires_all_seven_live_species_lists_empty() {
        let mut board = a33_ready_amp(1);
        board.breeders.clear();
        a33_tap(&mut board);
        a33_tap(&mut board);
        let old_id = board.fish[0].id;
        let events = a33_tap(&mut board);
        assert_eq!(board.fish.len(), 1);
        assert_ne!(board.fish[0].id, old_id);
        assert!(board.fish[0].alive);
        assert!(events.iter().any(|event| matches!(event, Event::AmpDischarged { replacement_id: Some(id), .. } if *id == board.fish[0].id)));

        let mut board = a33_ready_amp(1);
        board.breeders.clear();
        let oscar_id = board.id();
        let mut rng_state = board.rng_state;
        board
            .oscars
            .push(OscarState::spawn_bought(oscar_id, &mut |upper| {
                AdventureState::advance_rng(&mut rng_state) % upper
            }));
        board.rng_state = rng_state;
        a33_tap(&mut board);
        a33_tap(&mut board);
        a33_tap(&mut board);
        assert!(board.fish.is_empty());
        assert_eq!(board.oscars[0].id, oscar_id);
    }

    #[test]
    fn a33_amp_reward_collection_credits_once_and_shots_expire() {
        let mut board = a33_ready_amp(1);
        a33_tap(&mut board);
        a33_tap(&mut board);
        a33_tap(&mut board);
        let coin_id = board.coins[0].id;
        let (x, y) = (
            board.coins[0].x as f32 + 20.0,
            board.coins[0].y as f32 + 20.0,
        );
        board.apply(Action::Click { x, y });
        let events = (0..80).flat_map(|_| board.step(&[])).collect::<Vec<_>>();
        assert_eq!(events.iter().filter(|event| matches!(event, Event::CoinCredited { coin_id: id, amount: 200, .. } if *id == coin_id)).count(), 1);
        assert!(!board.coins.iter().any(|coin| coin.id == coin_id));
        assert!(board.bomb_shots.is_empty());
    }

    #[test]
    fn a33_amp_shared_shot_occupancy_preserves_type_draw_before_cap() {
        // PB66 caller draws each type before the shared20 admission test.
        // Safe constructor alpha is drawn only for an admitted effect.
        for (bombs, lasers, admitted) in [(19, 0, 1), (18, 1, 1), (19, 1, 0), (0, 20, 0)] {
            let mut board = a33_ready_amp(3);
            board.bomb_shots = vec![
                BombShot {
                    shot_type: 3,
                    x: 400,
                    y: 300,
                    age_ticks: 0,
                    frame: 0,
                    delay_ticks: 0,
                    alpha: 50
                };
                bombs
            ];
            board.invasion.as_mut().unwrap().lasers = vec![
                crate::invasion::LaserEffect {
                    x: 400,
                    y: 300,
                    age_ticks: 0
                };
                lasers
            ];
            let mut expected_rng = board.rng_state;
            for index in 0..3 {
                AdventureState::advance_rng(&mut expected_rng);
                if index < admitted {
                    AdventureState::advance_rng(&mut expected_rng);
                }
            }
            a33_tap(&mut board);
            a33_tap(&mut board);
            let events = a33_tap(&mut board);
            assert_eq!(board.bomb_shots.len() + lasers, 20);
            assert_eq!(board.rng_state, expected_rng);
            assert!(events.iter().any(|event| matches!(event,
                Event::AmpDischarged { effects, .. } if *effects == admitted)));
        }
    }

    #[test]
    fn a33_amp_routed_combat_click_cannot_discharge_or_consume_a_tap() {
        let mut board = a33_ready_amp(1);
        let alien_id = board.id();
        let wave = board.invasion.as_mut().unwrap();
        wave.actors.push(crate::alien::WeakSylvester::spawn_kind(
            SylvesterKind::Gus,
            alien_id,
            400,
            200,
            1,
            1,
        ));
        wave.battle_active = true;
        let events = a33_tap(&mut board);
        assert!(!events.iter().any(|event| matches!(
            event,
            Event::AmpCharged { .. } | Event::AmpDischarged { .. }
        )));
        assert_eq!(board.fish_pets[0].amp_charge, 0);
        assert_eq!(board.fish.len(), 1);
    }

    fn a32_group(board: &mut AdventureState) -> u64 {
        let id = board.id();
        let group = crate::bilaterus::BilaterusState::spawn(id, 200, 200, &mut || 1);
        let wave = board.invasion.as_mut().unwrap();
        wave.bilaterus.push(group);
        wave.battle_active = true;
        wave.countdown = 3000;
        id
    }

    fn a32_coin(board: &mut AdventureState, kind: CoinKind, x: f64, y: f64) -> u64 {
        let id = board.id();
        board.coins.push(Coin {
            id,
            x,
            y,
            kind,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            collecting: false,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: false,
        });
        id
    }

    fn a32_food(board: &mut AdventureState, y: f32) -> u64 {
        let id = board.id();
        board.food.push(Food {
            id,
            x: 100.0,
            y,
            frame: 0,
            food_type: FoodType::Ordinary,
            picked_up: false,
            ineligible_ticks: 20,
            removal_ticks: 0,
            quality: 2,
            direction: 0,
            vx: 0.0,
            vy: -2.0,
            animation_period: 3,
            free_from_zorf: false,
            nimbus_rising: true,
        });
        id
    }

    #[test]
    fn a32_blip_reveals_mapped_ultra_weapon_and_transactions_use_real_prices() {
        let mut board = AdventureState::new_tank4_second_stage(42, &[PetKind::Blip]).unwrap();
        assert!(board.apply(Action::BuyUltra).iter().any(|event| matches!(
            event,
            Event::Rejected {
                reason: Rejection::Locked,
                ..
            }
        )));
        board.tick = 201;
        board.update_fish_pets(&mut Vec::new());
        assert!(board.ultra_unlocked && board.weapon_unlocked && board.egg_unlocked);
        board.balance = 15_000; // Controlled affordability boundary, not earned gameplay.
        assert!(
            board
                .apply(Action::BuyUltra)
                .iter()
                .any(|event| matches!(event, Event::UltraBought { balance: 5000, .. }))
        );
        assert!(board.apply(Action::BuyWeapon).iter().any(|event| matches!(
            event,
            Event::WeaponBought {
                balance: 0,
                strength: 3,
                ..
            }
        )));
        assert_eq!((board.ultras.len(), board.egg_price), (1, 25_000));
        board.validate().unwrap();
    }

    #[test]
    fn a32_two_ultras_commit_one_oscar_during_group_freeze() {
        let mut board = AdventureState::new_tank4_second_stage(42, &[PetKind::Blip]).unwrap();
        board.tick = 201;
        board.update_fish_pets(&mut Vec::new());
        let oscar_id = board.id();
        let mut oscar = OscarState::spawn_bought(oscar_id, &mut |_| 0);
        oscar.x = 140.0;
        oscar.y = 200.0;
        oscar.widget_x = 140;
        oscar.widget_y = 200;
        board.oscars.push(oscar);
        for _ in 0..2 {
            let id = board.id();
            let mut ultra = UltraState::spawn_bought(id, &mut |_| 0);
            ultra.x = 100.0;
            ultra.y = 150.0;
            ultra.widget_x = 100;
            ultra.widget_y = 150;
            ultra.bought_timer = 0;
            ultra.hunger = 499;
            board.ultras.push(ultra);
        }
        a32_group(&mut board);
        let mut events = Vec::new();
        board.update_ultras(&mut events);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, Event::UltraAteOscar { .. }))
                .count(),
            1
        );
        assert!(board.oscars.is_empty());
        assert_eq!(
            (board.ultras[0].hunger, board.ultras[1].hunger),
            (1300, 499)
        );
        assert_eq!(
            (board.ultras[0].coin_timer, board.ultras[1].coin_timer),
            (0, 0)
        );
        board.validate().unwrap();
    }

    #[test]
    fn a32_nimbus_coin_and_food_replace_once_with_age_quality_and_fresh_rise() {
        let mut board = AdventureState::new_tank4_second_stage(42, &[PetKind::Nimbus]).unwrap();
        let pet = &mut board.fish_pets[0];
        pet.x = 100.0;
        pet.y = 320.0;
        pet.widget_x = 100;
        pet.widget_y = 320;
        let old_coin = a32_coin(&mut board, CoinKind::Treasure, 104.0, 324.0);
        board.coins[0].hazard_age_ticks = 1234;
        let old_food = a32_food(&mut board, 340.0);
        board.food[0].nimbus_rising = false;
        let mut events = Vec::new();
        board.update_fish_pets(&mut events);
        assert_eq!(board.coins.len(), 1);
        assert_eq!(board.food.len(), 1);
        assert_ne!(board.coins[0].id, old_coin);
        assert_ne!(board.food[0].id, old_food);
        assert_eq!(
            (
                board.coins[0].kind,
                board.coins[0].y,
                board.coins[0].hazard_age_ticks
            ),
            (CoinKind::ShellTreasure, 295.0, 1234)
        );
        assert!(board.coins[0].penta_rising);
        assert_eq!(
            (
                board.food[0].y,
                board.food[0].quality,
                board.food[0].ineligible_ticks
            ),
            (290.0, 2, 0)
        );
        assert!(board.food[0].nimbus_rising);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    Event::NimbusCoinConverted { .. } | Event::NimbusFoodConverted { .. }
                ))
                .count(),
            2
        );
        board.validate().unwrap();
    }

    #[test]
    fn time_trial_early_tank_nimbus_shell_conversion_roundtrips() {
        let mut board = AdventureState::new_time_trial(42, 1, &[PetKind::Nimbus]).unwrap();
        let pet = &mut board.fish_pets[0];
        pet.x = 100.0;
        pet.y = 320.0;
        pet.widget_x = 100;
        pet.widget_y = 320;
        a32_coin(&mut board, CoinKind::Diamond, 104.0, 324.0);
        let mut events = Vec::new();
        board.update_fish_pets(&mut events);
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Event::NimbusCoinConverted { .. }))
        );
        assert_eq!(board.coins[0].kind, CoinKind::ShellDiamond);
        board.validate().unwrap();
        let reopened: AdventureState =
            serde_json::from_slice(&serde_json::to_vec(&board).unwrap()).unwrap();
        reopened.validate().unwrap();
        assert_eq!(
            serde_json::to_value(&board).unwrap(),
            serde_json::to_value(&reopened).unwrap()
        );
    }

    #[test]
    fn a32_installed_rise_boundaries_apply_to_food_and_shell_currency() {
        for (y, expected, rising) in [
            (150.0, 142.0, true),
            (135.0, 132.5, true),
            (130.0, 128.5, true),
            (125.0, 124.0, true),
            (120.0, 119.5, true),
            (119.5, 119.5, false),
        ] {
            let mut board = AdventureState::new_tank4_second_stage(42, &[PetKind::Nimbus]).unwrap();
            a32_coin(&mut board, CoinKind::ShellSilver, 100.0, y);
            board.coins[0].penta_rising = true;
            a32_food(&mut board, y as f32);
            board.update_coins(&mut Vec::new());
            board.update_food(&mut Vec::new());
            assert_eq!(
                (board.coins[0].y, board.coins[0].penta_rising),
                (expected, rising)
            );
            assert_eq!(
                (f64::from(board.food[0].y), board.food[0].nimbus_rising),
                (expected, rising)
            );
            if !rising {
                board.update_coins(&mut Vec::new());
                board.update_food(&mut Vec::new());
                assert_eq!(
                    (board.coins[0].y, f64::from(board.food[0].y)),
                    (121.0, 121.0)
                );
            }
            board.validate().unwrap();
        }
    }

    #[test]
    fn a32_two_shots_resolve_promoted_head_and_final_click_never_creates_food() {
        let mut board = AdventureState::new_tank4_second_stage(42, &[]).unwrap();
        let id = a32_group(&mut board);
        let group = board
            .invasion
            .as_mut()
            .unwrap()
            .bilaterus_by_id_mut(id)
            .unwrap();
        group.emergence_ticks = 0;
        for head in group.heads.iter_mut().flatten() {
            head.health = 6.0;
        }
        let first = board.apply(Action::Click { x: 201.0, y: 201.0 });
        assert!(first.iter().any(|event| matches!(
            event,
            Event::Invasion {
                event: InvasionEvent::BilaterusFirstHeadDefeated { .. },
                ..
            }
        )));
        assert!(board.coins.is_empty());
        assert!(board.invasion.as_ref().unwrap().has_live_bilaterus());
        let final_events = board.apply(Action::Click { x: 201.0, y: 201.0 });
        assert!(!board.invasion.as_ref().unwrap().has_live_alien());
        assert!(!board.invasion.as_ref().unwrap().battle_active);
        assert_eq!(board.invasion.as_ref().unwrap().food_delay, 36);
        assert_eq!(
            final_events
                .iter()
                .filter(|event| matches!(
                    event,
                    Event::Invasion {
                        event: InvasionEvent::BattleEnded,
                        ..
                    }
                ))
                .count(),
            1
        );
        assert_eq!(board.invasion.as_ref().unwrap().fragments.len(), 8);
        assert_eq!(board.coins.len(), 1);
        assert_eq!(board.coins[0].kind, CoinKind::Diamond);
        assert_eq!(
            final_events
                .iter()
                .filter(|event| matches!(
                    event,
                    Event::Invasion {
                        event: InvasionEvent::LaserFired { .. },
                        ..
                    }
                ))
                .count(),
            1
        );
        assert!(board.food.is_empty());
        board.validate().unwrap();
    }

    #[test]
    fn a32_child_contact_removes_one_guppy_before_all_following_bones() {
        let mut board = AdventureState::new_tank4_second_stage(42, &[]).unwrap();
        let fish = board.make_fish(200.0, 200.0, false, false);
        let prey_id = fish.id;
        board.fish.push(fish);
        let id = a32_group(&mut board);
        let group = board
            .invasion
            .as_mut()
            .unwrap()
            .bilaterus_by_id_mut(id)
            .unwrap();
        group.emergence_ticks = 0;
        group.active_mut().bite_cooldown = 0;
        let mut events = Vec::new();
        board.update_bilaterus(&mut events);
        assert_eq!(events.iter().filter(|event|matches!(event,Event::Invasion{event:InvasionEvent::BilaterusPreyEaten{prey_id:eaten,..},..} if *eaten==prey_id)).count(),1);
        assert!(!board.fish.iter().any(|fish| fish.id == prey_id));
        board.validate().unwrap();
    }

    #[test]
    fn a32_shell_values_credit_once_and_preserve_original_currency() {
        for (kind, amount) in [
            (CoinKind::Silver, 15),
            (CoinKind::Gold, 35),
            (CoinKind::Star, 40),
            (CoinKind::Diamond, 200),
            (CoinKind::DiamondPenta, 200),
            (CoinKind::Pearl, 500),
            (CoinKind::Treasure, 2000),
        ] {
            let mut board = AdventureState::new_tank4_second_stage(42, &[PetKind::Nimbus]).unwrap();
            let shell = kind.nimbus_kind();
            assert_eq!(shell.value(), amount);
            let id = a32_coin(&mut board, shell, 100.0, 39.0);
            board.coins[0].collecting = true;
            let mut events = Vec::new();
            board.update_coins(&mut events);
            board.update_coins(&mut events);
            assert_eq!(board.balance, 200 + amount);
            assert_eq!(events.iter().filter(|event|matches!(event,Event::CoinCredited{coin_id,kind:credited,amount:actual,..} if *coin_id==id && *credited==shell && *actual==amount)).count(),1);
            board.validate().unwrap();
        }
    }

    fn test_shrapnel_bomb(board: &mut AdventureState, x: f64, y: f64, age: i32) -> u64 {
        let id = board.id();
        board.coins.push(Coin {
            id,
            x,
            y,
            kind: CoinKind::ShrapnelBomb,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: age,
            collecting: false,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: false,
        });
        id
    }

    #[test]
    fn bomb_shot_waits_its_delay_then_uses_alpha_dependent_source_pace() {
        let mut shot = BombShot {
            shot_type: 4,
            x: 100,
            y: 120,
            age_ticks: 0,
            frame: 0,
            delay_ticks: 2,
            alpha: 151,
        };
        shot.validate().unwrap();
        assert_eq!(shot.sprite_frame(), None);
        assert!(!shot.tick());
        assert_eq!((shot.age_ticks, shot.delay_ticks), (0, 1));
        assert!(!shot.tick());
        assert_eq!((shot.age_ticks, shot.delay_ticks), (0, 0));
        assert!(!shot.tick());
        assert_eq!((shot.age_ticks, shot.sprite_frame()), (2, Some(0)));
        while shot.age_ticks < 18 {
            assert!(!shot.tick());
        }
        assert!(shot.tick());
    }

    #[test]
    fn third_tank_third_stage_has_earned_pet_gate_and_reused_purchase_chain() {
        let mut board =
            AdventureState::new_tank3_third_stage(0x3301, &[PetKind::Shrapnel]).unwrap();
        assert_eq!(
            (board.balance, board.egg_price, board.fish.len()),
            (200, 7500, 2)
        );
        assert_eq!(
            board.invasion.as_ref().unwrap().plan,
            WavePlan::Fixed(SylvesterKind::Psychosquid)
        );
        board.validate().unwrap();
        board.upgrades.quality_unlocked = true;
        board.upgrades.quantity_unlocked = true;
        board.grubber_unlocked = true;
        board.balance = GRUBBER_PRICE;
        assert!(
            board
                .apply(Action::BuyGrubber)
                .iter()
                .any(|event| matches!(event, Event::GrubberBought { .. }))
        );
        assert!(board.gekko_unlocked);
        board.balance = GEKKO_PRICE;
        assert!(
            board
                .apply(Action::BuyGekko)
                .iter()
                .any(|event| matches!(event, Event::GekkoBought { .. }))
        );
        assert!(board.weapon_unlocked && board.egg_unlocked);
        board.validate().unwrap();
    }

    #[test]
    fn bomb_arms_on_thirty_first_update_and_first_contact_commits_before_next_bomb() {
        let mut board = AdventureState::new_tank3_third_stage(0x3302, &[]).unwrap();
        board.fish.truncate(1);
        board.fish[0].x = 100.0;
        board.fish[0].y = 100.0;
        let fish_id = board.fish[0].id;
        let first = test_shrapnel_bomb(&mut board, 100.0, 100.0, 29);
        let second = test_shrapnel_bomb(&mut board, 100.0, 100.0, 29);
        let mut events = Vec::new();
        board.update_coins(&mut events);
        assert!(board.fish[0].alive && events.is_empty());
        assert_eq!(board.coins[0].hazard_age_ticks, 30);
        let rng_before_contact = board.rng_state;
        board.update_coins(&mut events);
        assert!(!board.fish[0].alive);
        assert_eq!(board.dead_fish.len(), 1);
        assert_eq!(
            board.coins.iter().map(|coin| coin.id).collect::<Vec<_>>(),
            vec![second]
        );
        assert_eq!(events.iter().filter(|event| matches!(event, Event::ShrapnelBombExploded { coin_id, target_id: Some(id), .. } if *coin_id == first && *id == fish_id)).count(), 1);
        let fragments = events
            .iter()
            .find_map(|event| match event {
                Event::ShrapnelBombExploded {
                    coin_id, fragments, ..
                } if *coin_id == first => Some(*fragments),
                _ => None,
            })
            .unwrap();
        assert_eq!(board.bomb_shots.len(), usize::from(fragments) + 1);
        assert!(
            board
                .bomb_shots
                .iter()
                .all(|shot| (3..=5).contains(&shot.shot_type) && shot.delay_ticks == 0)
        );
        let mut source_order_rng = rng_before_contact;
        for _ in 0..(1 + usize::from(fragments) * 4 + 2) {
            AdventureState::advance_rng(&mut source_order_rng);
        }
        assert_eq!(board.rng_state, source_order_rng);
    }

    #[test]
    fn claimed_bomb_is_safe_and_age_does_not_wrap_with_sprite_animation() {
        let mut board = AdventureState::new_tank3_third_stage(0x3303, &[]).unwrap();
        board.fish[0].x = 100.0;
        board.fish[0].y = 100.0;
        let id = test_shrapnel_bomb(&mut board, 100.0, 100.0, 254);
        board.coins[0].collecting = true;
        board.coins[0].animation_ticks = 79;
        let mut events = Vec::new();
        board.update_coins(&mut events);
        assert!(board.fish[0].alive);
        assert_eq!(
            (
                board.coins[0].hazard_age_ticks,
                board.coins[0].animation_ticks,
                board.coins[0].frame
            ),
            (255, 0, 0)
        );
        board.update_coins(&mut events);
        assert_eq!(board.coins[0].hazard_age_ticks, 256);
        assert!(board.coins.iter().any(|coin| coin.id == id));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, Event::ShrapnelBombExploded { .. }))
        );
    }

    #[test]
    fn unclaimed_bomb_remains_armed_across_byte_and_word_age_boundaries() {
        for (before, after) in [(254, 256), (65_534, 65_536)] {
            let mut board = AdventureState::new_tank3_third_stage(
                0x3320,
                &[PetKind::Seymour, PetKind::Shrapnel],
            )
            .unwrap();
            board.fish[0].x = 100.0;
            board.fish[0].y = 100.0;
            board.fish[1].x = 430.0;
            board.fish[1].y = 300.0;
            let bomb_id = test_shrapnel_bomb(&mut board, 300.0, 100.0, before);
            let mut events = Vec::new();
            board.update_coins(&mut events);
            assert_eq!(board.coins[0].hazard_age_ticks, after - 1);
            assert!(board.fish[0].alive);
            board.coins[0].x = 100.0;
            board.update_coins(&mut events);
            assert!(!board.fish[0].alive);
            assert!(board.coins.iter().all(|coin| coin.id != bomb_id));
            assert!(events.iter().any(|event| matches!(event, Event::ShrapnelBombExploded { coin_id, target_id: Some(_), .. } if *coin_id == bomb_id)));
        }
    }

    #[test]
    fn bomb_bottom_expiry_is_immediate_and_seymour_only_extends_the_floor_clock() {
        let mut board =
            AdventureState::new_tank3_third_stage(0x3304, &[PetKind::Seymour, PetKind::Shrapnel])
                .unwrap();
        let id = test_shrapnel_bomb(&mut board, 400.0, 369.5, 30);
        board.coins[0].bottom_ticks = 199;
        let mut events = Vec::new();
        let rng_before_expiry = board.rng_state;
        board.update_coins(&mut events);
        assert!(board.coins.is_empty());
        assert_eq!(events.iter().filter(|event| matches!(event, Event::ShrapnelBombExploded { coin_id, target_id: None, fragments: 2..=4, .. } if *coin_id == id)).count(), 1);
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Event::CoinExpired { coin_id, .. } if *coin_id == id))
        );
        let fragments = events
            .iter()
            .find_map(|event| match event {
                Event::ShrapnelBombExploded {
                    coin_id, fragments, ..
                } if *coin_id == id => Some(*fragments),
                _ => None,
            })
            .unwrap();
        assert_eq!(board.bomb_shots.len(), usize::from(fragments));
        let mut source_order_rng = rng_before_expiry;
        for _ in 0..(1 + usize::from(fragments) * 4) {
            AdventureState::advance_rng(&mut source_order_rng);
        }
        assert_eq!(board.rng_state, source_order_rng);
        let draws_after = board.rng_state;
        board.update_coins(&mut events);
        assert_eq!(board.rng_state, draws_after);
    }

    #[test]
    fn bomb_uses_large_species_bounds_and_records_ordinary_corpse() {
        let mut board = AdventureState::new_tank3_third_stage(0x3305, &[]).unwrap();
        for fish in &mut board.fish {
            fish.x = 430.0;
            fish.y = 300.0;
        }
        let id = board.id();
        let mut gekko = GekkoState::spawn_bought(id, &mut |_| 0);
        gekko.x = 100.0;
        gekko.y = 100.0;
        gekko.widget_x = 100;
        gekko.widget_y = 100;
        board.gekkos.push(gekko);
        // Center x=140/y=140 is inside Gekko's +10..150 bounds, even though
        // it would miss a wrongly reused 80-pixel Guppy contact box at x=170.
        let bomb_id = test_shrapnel_bomb(&mut board, 134.0, 134.0, 30);
        let mut events = Vec::new();
        board.update_coins(&mut events);
        assert!(board.gekkos.is_empty());
        assert_eq!(board.dead_gekkos.len(), 1);
        assert!(events.iter().any(|event| matches!(event, Event::ShrapnelBombExploded { coin_id, target_id: Some(target), .. } if *coin_id == bomb_id && *target == id)));
    }

    #[test]
    fn collectors_ignore_bomb_even_when_it_overlaps_an_ordinary_coin_pursuit() {
        let mut board =
            AdventureState::new_tank3_third_stage(0x3306, &[PetKind::Stinky, PetKind::Clyde])
                .unwrap();
        let stinky = board.stinky.first_mut().unwrap();
        stinky.x = 100.0;
        stinky.y = 100.0;
        let clyde = board.clyde.first_mut().unwrap();
        clyde.x = 100.0;
        clyde.y = 100.0;
        clyde.widget_x = 100;
        clyde.widget_y = 100;
        let bomb_id = test_shrapnel_bomb(&mut board, 100.0, 100.0, 0);
        let ordinary_id = board.id();
        board.coins.push(Coin {
            id: ordinary_id,
            x: 200.0,
            y: 100.0,
            kind: CoinKind::Silver,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            collecting: false,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: false,
        });
        assert_eq!(
            AdventureState::stinky_target_x(
                board.stinky.first().unwrap(),
                &board.coins,
                &[],
                false
            ),
            Some(200)
        );
        let start_balance = board.balance;
        let mut events = Vec::new();
        board.update_stinky(&mut events);
        board.update_clyde(&mut events);
        assert_eq!(board.balance, start_balance);
        assert!(board.coins.iter().any(|coin| coin.id == bomb_id));
        assert!(board.coins.iter().any(|coin| coin.id == ordinary_id));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, Event::PetCollectedCoin { .. }))
        );
    }

    #[test]
    fn tank3_second_stage_purchase_chain_and_selected_seymour_are_distinct() {
        let mut board =
            AdventureState::new_tank3_second_stage(0x3201, &[PetKind::Seymour]).unwrap();
        assert_eq!(
            (board.balance, board.egg_price, board.weapon_strength),
            (200, 5000, 2)
        );
        assert_eq!(
            board
                .fish_pets
                .iter()
                .filter(|pet| pet.kind == FishPetKind::Seymour)
                .count(),
            1
        );
        assert!(matches!(
            board.invasion.as_ref().unwrap().plan,
            WavePlan::CyclingTank3Second { .. }
        ));
        board.validate().unwrap();
        assert!(board.apply(Action::BuyGekko).iter().any(|event| matches!(
            event,
            Event::Rejected {
                reason: Rejection::Locked,
                ..
            }
        )));
        board.upgrades.quality_unlocked = true;
        board.upgrades.quantity_unlocked = true;
        board.grubber_unlocked = true;
        board.balance = GRUBBER_PRICE;
        board.apply(Action::BuyGrubber);
        assert!(board.gekko_unlocked && !board.weapon_unlocked && !board.egg_unlocked);
        board.balance = GEKKO_PRICE;
        assert!(
            board
                .apply(Action::BuyGekko)
                .iter()
                .any(|event| matches!(event, Event::GekkoBought { balance: 0, .. }))
        );
        assert!(board.weapon_unlocked && board.egg_unlocked);
        board.validate().unwrap();
        assert!(AdventureState::new_tank3_first_stage(0x3201, &[PetKind::Seymour]).is_err());
        assert!(AdventureState::new_tank3_second_stage(0x3201, &[PetKind::Shrapnel]).is_err());
    }

    #[test]
    fn two_gekkos_commit_first_larva_meal_before_second_actor() {
        let mut board = AdventureState::new_tank3_second_stage(0x3202, &[]).unwrap();
        let larva_id = board.id();
        board
            .larvae
            .push(LarvaState::spawn(larva_id, 100, 150, &mut || 0));
        for _ in 0..2 {
            let id = board.id();
            let mut gekko = GekkoState::spawn_bought(id, &mut |_| 0);
            gekko.x = 100.0;
            gekko.y = 150.0;
            gekko.widget_x = 100;
            gekko.widget_y = 150;
            gekko.hunger = 300;
            gekko.bought_timer = 0;
            gekko.vy = 0.0;
            board.gekkos.push(gekko);
        }
        let mut events = Vec::new();
        board.update_gekkos(&mut events);
        assert_eq!(events.iter().filter(|event| matches!(event, Event::GekkoAtePrey { prey_id, .. } if *prey_id == larva_id)).count(), 1);
        assert!(board.larvae.is_empty());
        assert!(board.gekkos[0].hunger > board.gekkos[1].hunger);
    }

    #[test]
    fn live_seymour_slows_only_falling_coins_and_claim_credit_keeps_kind() {
        let mut board = AdventureState::new_tank3_second_stage(0x3203, &[]).unwrap();
        let coin_id = board.id();
        board.coins.push(Coin {
            id: coin_id,
            x: 100.0,
            y: 369.5,
            kind: CoinKind::Pearl,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            collecting: false,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: false,
        });
        board.pets.push(PetKind::Seymour); // An unlocked/selected name alone has no live modifier.
        board.update_coins(&mut Vec::new());
        assert_eq!(
            (
                board.coins[0].y,
                board.coins[0].bottom_ticks,
                board.coins[0].animation_ticks,
                board.coins[0].frame
            ),
            (370.0, 1, 1, 0)
        );
        let pet_id = board.id();
        board.fish_pets.push(FishPetState::spawn_tank1(
            pet_id,
            FishPetKind::Seymour,
            &mut |_| 0,
        ));
        board.coins[0].y = 369.5;
        board.coins[0].bottom_ticks = 0;
        board.update_coins(&mut Vec::new());
        assert_eq!(board.coins[0].y, 370.0);
        assert_eq!(board.coins[0].bottom_ticks, 1);
        board.coins[0].collecting = true;
        board.coins[0].fade_ticks = 4;
        board.coins[0].y = 39.9;
        let mut events = Vec::new();
        board.update_coins(&mut events);
        assert!(events.iter().any(|event| matches!(event,
            Event::CoinCredited { coin_id: id, kind: CoinKind::Pearl, amount: 500, .. } if *id == coin_id)));
        assert!(board.coins.is_empty());
    }

    #[test]
    fn coin_at_exact_bottom_after_fall_starts_floor_clock_immediately() {
        let mut board = AdventureState::new_tank3_second_stage(0x3205, &[]).unwrap();
        let id = board.id();
        board.coins.push(Coin {
            id,
            x: 100.0,
            y: 368.5,
            kind: CoinKind::Silver,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            collecting: false,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: false,
        });
        board.update_coins(&mut Vec::new());
        assert_eq!((board.coins[0].y, board.coins[0].bottom_ticks), (370.0, 1));
        board.update_coins(&mut Vec::new());
        assert_eq!((board.coins[0].y, board.coins[0].bottom_ticks), (370.0, 2));
    }

    #[test]
    fn coin_animation_wraps_before_claimed_flight_and_claim_cancels_prior_fade() {
        let mut board =
            AdventureState::new_tank3_second_stage(0x3204, &[PetKind::Seymour]).unwrap();
        let id = board.id();
        board.coins.push(Coin {
            id,
            x: 100.0,
            y: 100.0,
            kind: CoinKind::Silver,
            frame: 9,
            animation_ticks: 79,
            hazard_age_ticks: 0,
            collecting: true,
            bottom_ticks: 19,
            fade_ticks: 4,
            penta_rising: false,
        });
        board.update_coins(&mut Vec::new());
        assert_eq!(
            (
                board.coins[0].animation_ticks,
                board.coins[0].frame,
                board.coins[0].fade_ticks
            ),
            (0, 0, 0)
        );
        assert!(board.coins[0].y < 100.0);
        // Membership controls subsequent projection without resetting the clock.
        board.fish_pets.clear();
        board.coins[0].collecting = false;
        board.coins[0].animation_ticks = 19;
        board.update_coins(&mut Vec::new());
        assert_eq!(
            (board.coins[0].animation_ticks, board.coins[0].frame),
            (20, 0)
        );
    }

    #[test]
    fn tank3_large_growth_opens_grubber_then_purchase_opens_egg() {
        let mut board = AdventureState::new_tank3_first_stage(0x3101, &[]).unwrap();
        assert_eq!((board.balance, board.egg_price), (200, 1000));
        assert_eq!(board.invasion.as_ref().unwrap().countdown, 3000);
        assert!(board.apply(Action::BuyGrubber).iter().any(|event| matches!(
            event,
            Event::Rejected {
                reason: Rejection::Locked,
                ..
            }
        )));
        let fish = &mut board.fish[0];
        fish.size = FishSize::Medium;
        fish.x = 100.0;
        fish.y = 100.0;
        fish.hunger = 400;
        fish.food_ate = fish.food_needed_to_grow - 1;
        board.food.push(Food {
            id: 100,
            x: 120.0,
            y: 120.0,
            frame: 0,
            food_type: FoodType::Ordinary,
            picked_up: false,
            ineligible_ticks: 0,
            removal_ticks: 0,
            quality: 0,
            direction: 0,
            vx: 0.0,
            vy: 0.0,
            animation_period: 3,
            free_from_zorf: false,
            nimbus_rising: false,
        });
        let mut events = Vec::new();
        board.update_fish(&mut events);
        assert_eq!(board.fish[0].size, FishSize::Large);
        assert!(
            board.grubber_unlocked
                && board.upgrades.quality_unlocked
                && board.upgrades.quantity_unlocked
        );
        assert!(!board.egg_unlocked);
        board.balance = 750;
        assert!(
            board
                .apply(Action::BuyGrubber)
                .iter()
                .any(|event| matches!(event, Event::GrubberBought { balance: 0, .. }))
        );
        assert!(board.egg_unlocked);
        assert_eq!(
            (board.grubbers[0].widget_y, board.grubbers[0].y),
            (65, 65.0)
        );
    }

    #[test]
    fn every_available_single_pet_roster_constructs_a_valid_third_tank_board() {
        let unlocked = [
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
        ];
        for (index, pet) in unlocked.into_iter().enumerate() {
            let board = AdventureState::new_tank3_first_stage(0x3100 + index as u64, &[pet])
                .expect("unlocked single pet belongs in Tank 3-1");
            board
                .validate()
                .unwrap_or_else(|error| panic!("{pet:?}: {error}"));
        }
        assert!(AdventureState::new_tank3_first_stage(7, &[PetKind::Seymour]).is_err());
    }

    #[test]
    fn two_grubbers_commit_one_small_prey_before_next_snapshot() {
        let mut board = AdventureState::new_tank3_first_stage(0x3102, &[]).unwrap();
        board.fish.clear();
        board.grubber_unlocked = true;
        board.upgrades.quality_unlocked = true;
        board.upgrades.quantity_unlocked = true;
        board.balance = 1500;
        board.apply(Action::BuyGrubber);
        board.apply(Action::BuyGrubber);
        assert_eq!(board.grubbers.len(), 2);
        for actor in &mut board.grubbers {
            actor.x = 100.0;
            actor.y = 100.0;
            actor.widget_x = 100;
            actor.widget_y = 100;
            actor.hunger = 800;
            actor.bought_timer = 0;
        }
        let mut fish = board.make_fish(100.0, 100.0, false, false);
        fish.size = FishSize::Small;
        fish.cannot_be_eaten_ticks = 0;
        let fish_id = fish.id;
        board.fish.push(fish);
        let mut events = Vec::new();
        board.update_grubbers(&mut events);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event,
                    Event::GrubberAteGuppy { guppy_id, .. } if *guppy_id == fish_id
                ))
                .count(),
            1
        );
        assert!(board.fish.is_empty());
        assert!(board.dead_fish.is_empty());
    }

    #[test]
    fn due_grubber_produces_on_starvation_tail_and_board_removes_live_actor() {
        let mut board = AdventureState::new_tank3_first_stage(0x3103, &[]).unwrap();
        board.grubber_unlocked = true;
        board.upgrades.quality_unlocked = true;
        board.upgrades.quantity_unlocked = true;
        board.balance = 750;
        board.apply(Action::BuyGrubber);
        let actor = &mut board.grubbers[0];
        actor.x = 100.0;
        actor.y = 300.0;
        actor.widget_x = 100;
        actor.widget_y = 300;
        actor.bought_timer = 0;
        actor.hunger = 1;
        actor.coin_timer = actor.coin_threshold - 1;
        let mut events = Vec::new();
        board.update_grubbers(&mut events);
        assert!(board.grubbers.is_empty());
        assert_eq!(
            (
                board.dead_grubbers[0].widget_x,
                board.dead_grubbers[0].widget_y
            ),
            (100, 300)
        );
        assert_eq!(
            (board.larvae[0].widget_x, board.larvae[0].widget_y),
            (104, 290)
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, Event::GrubberDied { .. }))
                .count(),
            1
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, Event::LarvaDropped { .. }))
                .count(),
            1
        );
        board.validate().unwrap();
    }

    #[test]
    fn claimed_larva_funds_and_arrival_credit_only_once() {
        let mut board = AdventureState::new_tank3_first_stage(0x3104, &[]).unwrap();
        let larva_id = board.id();
        let larva = LarvaState::spawn(larva_id, 100, 319, &mut || board.rand());
        board.larvae.push(larva);
        let mut events = Vec::new();
        board.update_larvae(&mut events);
        // Spawned at Y=319: the first old-position check is already below
        // the strict 320 visibility boundary.
        assert!(board.larvae[0].mouse_visible);
        board.update_larvae(&mut events);
        assert!(board.larvae[0].mouse_visible);
        let click = board.apply(Action::Click { x: 110.0, y: 320.0 });
        assert!(click.iter().any(|event| matches!(event,
            Event::LarvaCollectionStarted { larva_id: id, .. } if *id == larva_id
        )));
        board.balance = 900;
        assert_eq!(board.available_funds(), 1050);
        board.grubber_unlocked = true;
        board.upgrades.quality_unlocked = true;
        board.upgrades.quantity_unlocked = true;
        board.egg_unlocked = true;
        board.apply(Action::BuyEgg);
        assert_eq!(board.balance, -100);
        let mut credits = 0;
        for _ in 0..100 {
            board.update_larvae(&mut events);
            credits = events
                .iter()
                .filter(|event| {
                    matches!(event,
                        Event::LarvaCredited { larva_id: id, .. } if *id == larva_id
                    )
                })
                .count();
            if credits > 0 {
                break;
            }
        }
        assert_eq!(credits, 1);
        assert_eq!(board.balance, 50);
        assert!(board.larvae.is_empty());
        board.update_larvae(&mut events);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, Event::LarvaCredited { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn picked_visible_larva_absorbs_repeat_click_without_another_transaction() {
        // Larva::MouseDown handles repeated hits while its widget is still
        // visible during HUD flight; neither food nor combat receives them.
        let mut board = AdventureState::new_tank3_first_stage(0x3106, &[]).unwrap();
        let larva_id = board.id();
        let mut larva = LarvaState::spawn(larva_id, 100, 200, &mut || board.rand());
        larva.mouse_visible = true;
        board.larvae.push(larva);
        let initial_balance = board.balance;
        let first = board.apply(Action::Click { x: 110.0, y: 210.0 });
        let second = board.apply(Action::Click { x: 110.0, y: 210.0 });
        assert_eq!(
            first
                .iter()
                .filter(|event| matches!(event,
                    Event::LarvaCollectionStarted { larva_id: id, .. } if *id == larva_id
                ))
                .count(),
            1
        );
        assert!(!second.iter().any(|event| matches!(
            event,
            Event::LarvaCollectionStarted { .. }
                | Event::FoodDropped { .. }
                | Event::Invasion {
                    event: InvasionEvent::LaserFired { .. },
                    ..
                }
        )));
        assert_eq!(
            first
                .iter()
                .chain(second.iter())
                .filter(|event| matches!(event,
                    Event::LarvaPickupSound { larva_id: id, .. } if *id == larva_id
                ))
                .count(),
            2
        );
        assert_eq!(board.balance, initial_balance);
        assert!(board.food.is_empty());
        assert!(board.larvae[0].picked_up);
    }

    #[test]
    fn dead_guppy_record_cannot_restart_wadsworth_active_clock() {
        // Fish death retains a Rust record until corpse removal. The source
        // ward scans only members still in its live fish widget list.
        let mut board =
            AdventureState::new_tank3_first_stage(0x3107, &[PetKind::Wadsworth]).unwrap();
        let pet = &mut board.fish_pets[0];
        pet.x = 100.0;
        pet.y = 100.0;
        pet.widget_x = 100;
        pet.widget_y = 100;
        pet.published_x = 100;
        pet.published_y = 100;
        pet.ward_active = true;
        pet.ward_timer = 0;
        board.fish[0].x = 100.0;
        board.fish[0].y = 100.0;
        board.fish[1].x = 300.0;
        board.fish[1].y = 300.0;
        board.fish[1].alive = false;
        let live_id = board.fish[0].id;
        let missile_id = board.id();
        board
            .missiles
            .push(ClassicMissile::launch(missile_id, live_id, 200, 200, 0));
        board.update_fish_pets(&mut Vec::new());
        assert!(board.fish_pets[0].ward_active);
        assert_eq!(board.fish_pets[0].ward_timer, 0);
    }

    #[test]
    fn warded_medium_drops_at_old_widget_before_attraction() {
        // Fish::DropCoin precedes the Wadsworth branch's eventual Move.
        let mut board =
            AdventureState::new_tank3_first_stage(0x3108, &[PetKind::Wadsworth]).unwrap();
        let pet = &mut board.fish_pets[0];
        pet.ward_active = true;
        pet.published_x = 100;
        pet.published_y = 200;
        let fish = &mut board.fish[0];
        fish.size = FishSize::Medium;
        fish.x = 200.0;
        fish.y = 200.0;
        fish.bought_timer = 0;
        fish.coin_timer = fish.coin_threshold - 1;
        board.update_fish(&mut Vec::new());
        assert_eq!((board.coins[0].x, board.coins[0].y), (205.0, 210.0));
        assert!(board.fish[0].x < 200.0);
    }

    #[test]
    fn warded_edge_damps_velocity_before_move_and_allows_overshoot() {
        // W1 Fish::Update (514-533) damps at x>535 and clamps before Move.
        let mut board =
            AdventureState::new_tank3_first_stage(0x3109, &[PetKind::Wadsworth]).unwrap();
        let pet = &mut board.fish_pets[0];
        pet.ward_active = true;
        pet.published_x = 540;
        pet.published_y = 200;
        let fish = &mut board.fish[0];
        fish.x = 539.9;
        fish.y = 200.0;
        fish.vx = 0.5;
        fish.speed_mod = 2.0;
        fish.bought_timer = 0;
        board.update_fish(&mut Vec::new());
        assert!((board.fish[0].vx - 0.4).abs() < 0.00001);
        assert!((board.fish[0].x - 540.1).abs() < 0.0001);
    }

    #[test]
    fn ward_hides_only_strict_near_pose_and_freezes_hunger_without_freezing_motion() {
        let mut board =
            AdventureState::new_tank3_first_stage(0x3105, &[PetKind::Wadsworth]).unwrap();
        let pet = board
            .fish_pets
            .iter_mut()
            .find(|pet| pet.kind == FishPetKind::Wadsworth)
            .unwrap();
        pet.ward_active = true;
        pet.ward_timer = 100;
        pet.published_x = 100;
        pet.published_y = 100;
        assert!(board.hides_fish_at(107.9, 92.1));
        assert!(!board.hides_fish_at(108.0, 100.0));
        let fish = &mut board.fish[0];
        fish.size = FishSize::Medium;
        fish.x = 180.0;
        fish.y = 180.0;
        fish.hunger = 301;
        fish.bought_timer = 0;
        let before = (fish.x, fish.y);
        let mut events = Vec::new();
        board.update_fish(&mut events);
        assert_eq!(board.fish[0].hunger, 301);
        assert_ne!((board.fish[0].x, board.fish[0].y), before);
    }

    #[test]
    fn source_first_tank_initial_state_and_purchase_gates() {
        let mut state = AdventureState::new_adventure(42);
        assert_eq!((state.tank, state.level, state.balance), (1, 1, 200));
        assert_eq!(state.fish.len(), 2);
        assert!(
            state
                .fish
                .iter()
                .all(|fish| fish.size == FishSize::Small && fish.food_ate == 2 && fish.beginner)
        );
        assert!(
            state
                .fish
                .iter()
                .all(|fish| (400..600).contains(&fish.hunger)
                    && (4..=6).contains(&fish.food_needed_to_grow))
        );
        assert!(state.apply(Action::BuyEgg).iter().any(|event| matches!(
            event,
            Event::Rejected {
                reason: Rejection::Locked,
                ..
            }
        )));
    }

    #[test]
    fn clicked_food_uses_board_override_and_cap_refund() {
        let mut state = AdventureState::new_adventure(17);
        let dropped = state.apply(Action::Click { x: 320.0, y: 200.0 });
        assert!(
            dropped
                .iter()
                .any(|event| matches!(event, Event::FoodDropped { balance: 195, .. }))
        );
        assert_eq!(state.food[0].ineligible_ticks, 0);
        assert_eq!((state.food[0].x, state.food[0].y), (310.0, 190.0));
        assert!(
            state
                .apply(Action::Click { x: 350.0, y: 200.0 })
                .iter()
                .any(|event| matches!(
                    event,
                    Event::Rejected {
                        reason: Rejection::FoodCapacity,
                        ..
                    }
                ))
        );
        assert_eq!(state.balance, 195);
    }

    #[test]
    fn coin_click_credits_only_after_arrival_once() {
        let mut state = AdventureState::new_adventure(1);
        state.coins.push(Coin {
            id: 99,
            x: 100.0,
            y: 300.0,
            kind: CoinKind::Silver,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            collecting: false,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: false,
        });
        let click = Action::Click { x: 110.0, y: 310.0 };
        assert!(
            state
                .apply(click.clone())
                .iter()
                .any(|event| matches!(event, Event::CoinCollectionStarted { coin_id: 99, .. }))
        );
        assert_eq!(state.balance, 200);
        assert!(
            !state
                .apply(click)
                .iter()
                .any(|event| matches!(event, Event::CoinCollectionStarted { .. }))
        );
        // After collection starts, the coin stops catching clicks. The next
        // click can buy food, but it cannot credit this coin twice.
        assert_eq!(state.balance, 195);
        let mut credit_count = 0;
        for _ in 0..100 {
            credit_count += state
                .tick()
                .iter()
                .filter(|event| matches!(event, Event::CoinCredited { coin_id: 99, .. }))
                .count();
        }
        assert_eq!(credit_count, 1);
        assert_eq!(state.balance, 210);
    }

    #[test]
    fn three_affordable_eggs_complete_first_level_once() {
        let mut state = AdventureState::new_adventure(3);
        state.egg_unlocked = true;
        state.balance = 450;
        for pieces in 1..=3 {
            assert!(
                state.apply(Action::BuyEgg).iter().any(
                    |event| matches!(event, Event::EggBought { pieces: n, .. } if *n == pieces)
                )
            );
        }
        assert_eq!((state.eggs, state.balance, state.victory), (3, 0, true));
        assert!(state.apply(Action::BuyEgg).iter().any(|event| matches!(
            event,
            Event::Rejected {
                reason: Rejection::Completed,
                ..
            }
        )));
    }

    #[test]
    fn feeding_to_medium_allows_a_naturally_timed_silver_coin() {
        let mut state = AdventureState::new_adventure(11);
        state.fish[0].x = 100.0;
        state.fish[0].y = 100.0;
        state.fish[0].hunger = 400;
        state.fish[0].food_ate = state.fish[0].food_needed_to_grow - 1;
        state.fish[1].x = 500.0;
        state.fish[1].y = 300.0;
        let fish_id = state.fish[0].id;
        state.apply(Action::Click { x: 130.0, y: 130.0 });
        let first_tick = state.tick();
        assert!(
            first_tick.iter().any(
                |event| matches!(event, Event::FoodEaten { fish_id: id, .. } if *id == fish_id)
            )
        );
        assert!(first_tick.iter().any(|event| matches!(event, Event::FishGrew { fish_id: id, size: FishSize::Medium, .. } if *id == fish_id)));
        assert!(state.food.is_empty());
        assert!(state.guppy_unlocked);
        let threshold = state.fish[0].coin_threshold;
        for _ in 1..(threshold - 1) {
            assert!(!state.tick().iter().any(
                |event| matches!(event, Event::CoinDropped { fish_id: id, .. } if *id == fish_id)
            ));
        }
        // The growth tick already advanced the coin timer once.
        assert!(state.tick().iter().any(|event| matches!(event, Event::CoinDropped { fish_id: id, kind: CoinKind::Silver, .. } if *id == fish_id)));
    }

    #[test]
    fn source_starvation_thresholds_keep_beginner_alive_longer() {
        let mut state = AdventureState::new_adventure(8);
        state.fish[0].hunger = -498;
        state.fish[1].beginner = false;
        state.fish[1].hunger = 1;
        let ordinary_id = state.fish[1].id;
        let first = state.tick();
        assert!(state.fish[0].alive);
        assert!(!state.fish[1].alive);
        assert!(first.iter().any(
            |event| matches!(event, Event::FishDied { fish_id, .. } if *fish_id == ordinary_id)
        ));
        let second = state.tick();
        assert!(!state.fish[0].alive);
        assert_eq!(
            second
                .iter()
                .filter(|event| matches!(event, Event::FishDied { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn food_and_coin_expire_after_source_bottom_countdowns() {
        let mut state = AdventureState::new_adventure(9);
        state.fish.clear();
        state.food.push(Food {
            id: 50,
            x: 100.0,
            y: 410.0,
            frame: 0,
            food_type: FoodType::Ordinary,
            picked_up: false,
            ineligible_ticks: 0,
            removal_ticks: 0,
            quality: 0,
            direction: 0,
            vx: 0.0,
            vy: 0.0,
            animation_period: 3,
            free_from_zorf: false,
            nimbus_rising: false,
        });
        state.coins.push(Coin {
            id: 51,
            x: 100.0,
            y: 370.0,
            kind: CoinKind::Gold,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            collecting: false,
            bottom_ticks: 149,
            fade_ticks: 0,
            penta_rising: false,
        });
        state.tick();
        assert_eq!(state.food[0].removal_ticks, 15);
        assert_eq!(state.coins[0].fade_ticks, 5);
        for _ in 0..4 {
            state.tick();
        }
        assert_eq!(state.coins.len(), 1);
        assert!(
            state
                .tick()
                .iter()
                .any(|event| matches!(event, Event::CoinExpired { coin_id: 51, .. }))
        );
        for _ in 0..9 {
            state.tick();
        }
        assert_eq!(state.food.len(), 1);
        assert!(
            state
                .tick()
                .iter()
                .any(|event| matches!(event, Event::FoodExpired { food_id: 50, .. }))
        );
    }

    #[test]
    fn sorted_food_update_can_make_a_collision_this_tick() {
        let mut state = AdventureState::new_adventure(12);
        state.fish[0].x = 100.0;
        state.fish[0].y = 100.0;
        state.fish[0].hunger = 400;
        state.fish[1].x = 500.0;
        state.fish[1].y = 300.0;
        let fish_id = state.fish[0].id;
        // Fish center Y=140. Food begins at Y=105, just outside the strict
        // collision bound; its 1.5-pixel fall opens that bound before fish update.
        state.apply(Action::Click { x: 130.0, y: 115.0 });
        assert!(
            state.tick().iter().any(
                |event| matches!(event, Event::FoodEaten { fish_id: id, .. } if *id == fish_id)
            )
        );
    }

    #[test]
    fn death_tick_can_still_drop_a_coin_before_deferred_deletion() {
        let mut state = AdventureState::new_adventure(13);
        state.fish[0].size = FishSize::Medium;
        state.fish[0].beginner = false;
        state.fish[0].hunger = 1;
        state.fish[0].coin_timer = state.fish[0].coin_threshold - 1;
        let fish_id = state.fish[0].id;
        let events = state.tick();
        assert!(
            events.iter().any(
                |event| matches!(event, Event::FishDied { fish_id: id, .. } if *id == fish_id)
            )
        );
        assert!(events.iter().any(|event| matches!(event, Event::CoinDropped { fish_id: id, kind: CoinKind::Silver, .. } if *id == fish_id)));
        assert!(!state.tick().iter().any(
            |event| matches!(event, Event::CoinDropped { fish_id: id, .. } if *id == fish_id)
        ));
    }

    #[test]
    fn collection_checks_previous_integer_y_before_easing() {
        let mut state = AdventureState::new_adventure(14);
        state.coins.push(Coin {
            id: 90,
            x: 550.0,
            y: 40.5,
            kind: CoinKind::Gold,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            collecting: true,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: false,
        });
        let first = state.tick();
        assert!(
            !first
                .iter()
                .any(|event| matches!(event, Event::CoinCredited { coin_id: 90, .. }))
        );
        assert_eq!(state.balance, 200);
        assert!(state.tick().iter().any(|event| matches!(
            event,
            Event::CoinCredited {
                coin_id: 90,
                amount: 35,
                ..
            }
        )));
        assert_eq!(state.balance, 235);
    }

    #[test]
    fn source_turn_eat_and_growth_pulses_expose_render_state() {
        let mut state = AdventureState::new_adventure(21);
        state.fish[0].x = 100.0;
        state.fish[0].y = 100.0;
        state.fish[0].hunger = 400;
        state.fish[0].previous_vx = -1.0;
        state.fish[0].vx = 1.0;
        state.fish[0].special_timer = 0;
        state.fish[0].movement_state = 3;
        state.tick();
        assert_eq!(state.fish[0].turn_ticks, -19);
        assert_eq!(state.fish[0].sprite_pose(), FishPose::Turn);
        for _ in 0..19 {
            state.tick();
        }
        assert_eq!(state.fish[0].turn_ticks, 0);

        state.fish[0].x = 100.0;
        state.fish[0].y = 100.0;
        state.fish[0].hunger = 400;
        state.apply(Action::Click { x: 155.0, y: 130.0 });
        state.tick();
        assert_eq!(state.fish[0].sprite_pose(), FishPose::Eat);
        assert_eq!(state.fish[0].eating_ticks, 19);

        state.food.clear();
        state.fish[0].x = 100.0;
        state.fish[0].y = 100.0;
        state.fish[0].hunger = 400;
        state.fish[0].food_ate = state.fish[0].food_needed_to_grow - 1;
        state.apply(Action::Click { x: 130.0, y: 130.0 });
        state.tick();
        assert_eq!(state.fish[0].size, FishSize::Medium);
        assert_eq!(state.fish[0].growth_ticks, 9);
        assert!((state.fish[0].growth_scale() - 0.6).abs() < 0.001);
    }

    #[test]
    fn beginner_hunger_tutorial_thresholds_are_once_only() {
        let mut state = AdventureState::new_adventure(22);
        let thresholds = [
            (0, TutorialCue::Hungry),
            (-199, TutorialCue::VeryHungry),
            (-399, TutorialCue::Starving),
        ];
        for (prior_hunger, expected) in thresholds {
            state.fish[0].hunger = prior_hunger;
            assert!(
                state
                    .tick()
                    .iter()
                    .any(|event| matches!(event, Event::Tutorial { cue, .. } if *cue == expected))
            );
        }
        state.fish[0].hunger = 0;
        assert!(!state.tick().iter().any(|event| matches!(
            event,
            Event::Tutorial {
                cue: TutorialCue::Hungry,
                ..
            }
        )));
    }

    #[test]
    fn dead_guppy_animates_at_bottom_until_source_lifetime_ends() {
        let mut state = AdventureState::new_adventure(23);
        state.fish[0].hunger = -499;
        state.fish[0].y = 370.0;
        state.fish[0].vy = 4.0;
        let id = state.fish[0].id;
        state.tick();
        assert_eq!(state.dead_fish[0].id, id);
        assert_eq!(state.dead_fish[0].remaining_ticks, 125);
        for _ in 0..125 {
            state.tick();
        }
        assert_eq!(state.dead_fish[0].remaining_ticks, 0);
        assert_eq!(state.dead_fish[0].opacity, 0.0);
        state.tick();
        assert!(state.dead_fish.is_empty());
        assert!(!state.fish.iter().any(|fish| fish.id == id));
    }

    #[test]
    fn second_stage_stinky_has_live_spawn_state_and_persisted_origin() {
        let mut state = AdventureState::new_second_stage(0x7788);
        let pet = state.stinky.first().unwrap();
        assert!((105.0..370.0).contains(&pet.x));
        assert_eq!(
            (pet.y, pet.vx, pet.vy, pet.previous_vx),
            (370.0, 0.0, 0.0, 1.0)
        );
        assert_eq!(pet.chase_timer, 40);
        assert!(pet.movement_state < 10);
        assert!((250..500).contains(&pet.random_timer));
        assert_eq!(pet.origin, StinkyOrigin::StageStart);
        assert_eq!(state.fish.len(), 2);
        assert!(
            state
                .fish
                .iter()
                .all(|fish| !fish.beginner && fish.food_ate == 2)
        );
        assert!(!state.initialize_missing_stinky());

        state.stinky.clear(); // The previous v2 project-save shape.
        assert!(state.initialize_missing_stinky());
        assert_eq!(
            state.stinky.first().unwrap().origin,
            StinkyOrigin::LegacyV2Resume
        );
        let restored: AdventureState =
            serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
        assert_eq!(
            restored.stinky.first().unwrap().origin,
            StinkyOrigin::LegacyV2Resume
        );
        assert!(!state.initialize_missing_stinky());
        let mut first_stage = AdventureState::new_adventure(0x7788);
        first_stage.pets.push(PetKind::Stinky);
        assert!(!first_stage.initialize_missing_stinky());
    }

    #[test]
    fn stinky_idle_motion_and_animation_advance_from_source_counters() {
        let mut state = AdventureState::new_second_stage(0x55cc);
        let pet = state.stinky.first_mut().unwrap();
        pet.x = 200.0;
        pet.movement_state = 2;
        let initial_x = pet.x;
        state.tick();
        let pet = state.stinky.first().unwrap();
        assert!((pet.vx - 0.1).abs() < 1e-10);
        assert!((pet.x - (initial_x + 0.1 / 1.2)).abs() < 1e-10);
        assert_eq!(pet.movement_animation_timer, 1);
        assert_eq!(pet.sprite_row(), 0);
        assert!(pet.facing_right());
        assert_eq!(pet.chase_timer, 41);
    }

    #[test]
    fn stinky_collects_first_unclaimed_overlap_before_coin_update_once() {
        let mut state = AdventureState::new_second_stage(0x8844);
        let pet = state.stinky.first_mut().unwrap();
        pet.x = 100.0;
        pet.y = 360.0;
        pet.vx = 0.0;
        state.coins.extend([
            Coin {
                id: 90,
                x: 100.0,
                y: 360.0,
                kind: CoinKind::Gold,
                frame: 0,
                animation_ticks: 0,
                hazard_age_ticks: 0,
                collecting: true,
                bottom_ticks: 0,
                fade_ticks: 0,
                penta_rising: false,
            },
            Coin {
                id: 91,
                x: 100.0,
                y: 360.0,
                kind: CoinKind::Silver,
                frame: 0,
                animation_ticks: 0,
                hazard_age_ticks: 0,
                collecting: false,
                bottom_ticks: 0,
                fade_ticks: 1,
                penta_rising: false,
            },
            Coin {
                id: 92,
                x: 100.0,
                y: 360.0,
                kind: CoinKind::Gold,
                frame: 0,
                animation_ticks: 0,
                hazard_age_ticks: 0,
                collecting: false,
                bottom_ticks: 0,
                fade_ticks: 0,
                penta_rising: false,
            },
        ]);
        let events = state.tick();
        assert_eq!(state.balance, 215);
        assert!(!state.coins.iter().any(|coin| coin.id == 91));
        assert!(state.coins.iter().any(|coin| coin.id == 92));
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    Event::PetCollectedCoin {
                        coin_id: 91,
                        amount: 15,
                        ..
                    }
                ))
                .count(),
            1
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    Event::CoinCredited {
                        coin_id: 91,
                        amount: 15,
                        ..
                    }
                ))
                .count(),
            1
        );
        assert!(!events.iter().any(|event| matches!(
            event,
            Event::CoinCredited {
                coin_id: 90 | 92,
                ..
            }
        )));
    }

    #[test]
    fn stinky_overlap_edges_are_strict_and_board_clock_can_advance_alone() {
        let mut state = AdventureState::new_second_stage(0x9977);
        let pet = state.stinky.first_mut().unwrap();
        pet.x = 76.0; // Center equals coin.x + 16: no collision.
        pet.y = 360.0;
        pet.vx = 0.0;
        state.coins.push(Coin {
            id: 10,
            x: 100.0,
            y: 360.0,
            kind: CoinKind::Silver,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            collecting: false,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: false,
        });
        let initial_hunger = state.fish[0].hunger;
        state.advance_board_clock();
        assert_eq!(state.tick, 1);
        assert_eq!(state.fish[0].hunger, initial_hunger);
        assert!(
            !state
                .tick()
                .iter()
                .any(|event| matches!(event, Event::PetCollectedCoin { coin_id: 10, .. }))
        );
        state.stinky.first_mut().unwrap().x = 76.01;
        state.coins[0].y = 360.0;
        assert!(
            state
                .tick()
                .iter()
                .any(|event| matches!(event, Event::PetCollectedCoin { coin_id: 10, .. }))
        );
    }

    #[test]
    fn saved_stinky_validation_allows_post_clamp_overshoot_but_rejects_bad_state() {
        let state = AdventureState::new_second_stage(0x3120);
        let mut pet = state.stinky.into_iter().next().unwrap();
        pet.x = 552.0; // The last integration can move beyond the 550 clamp.
        assert!(pet.validate().is_ok());
        pet.x = f64::NAN;
        assert!(pet.validate().is_err());
        pet.x = 100.0;
        pet.frame = 10;
        assert!(pet.validate().is_err());
        pet.frame = 0;
        pet.turn_animation_timer = 21;
        assert!(pet.validate().is_err());
        pet.turn_animation_timer = 0;
        pet.specialty_timer = 10;
        assert!(pet.validate().is_err());
        pet.specialty_timer = 0;
        pet.movement_state = 10;
        assert!(pet.validate().is_err());
        pet.movement_state = 0;
        pet.movement_animation_timer = 40;
        assert!(pet.validate().is_err());
    }

    #[test]
    fn installed_payload_nearest_center_differs_from_recovered_source_offset() {
        let mut state = AdventureState::new_second_stage(0xa45c);
        let pet = state.stinky.first_mut().unwrap();
        pet.x = 100.0;
        pet.y = 360.0;
        pet.vx = 0.0;
        pet.chase_timer = 40;
        for (id, x) in [(1, 100.0), (2, 136.0)] {
            state.coins.push(Coin {
                id,
                x,
                y: 100.0,
                kind: CoinKind::Silver,
                frame: 0,
                animation_ticks: 0,
                hazard_age_ticks: 0,
                collecting: false,
                bottom_ticks: 0,
                fade_ticks: 0,
                penta_rising: false,
            });
        }
        state.tick();
        // The installed center metric selects x=100 and brakes at its center;
        // W1's left-associative +36 term would select x=136 and accelerate.
        assert_eq!(state.stinky.first().unwrap().vx, 0.0);
        assert_eq!(state.stinky.first().unwrap().chase_timer, 1);
    }

    #[test]
    fn stinky_target_ties_keep_first_coin_and_steering_waits_past_four() {
        let mut tied = AdventureState::new_second_stage(0xa45d);
        let pet = tied.stinky.first_mut().unwrap();
        pet.x = 100.0;
        pet.y = 360.0;
        pet.vx = 0.0;
        pet.chase_timer = 40;
        for (id, x) in [(1, 80.0), (2, 120.0)] {
            tied.coins.push(Coin {
                id,
                x,
                y: 100.0,
                kind: CoinKind::Silver,
                frame: 0,
                animation_ticks: 0,
                hazard_age_ticks: 0,
                collecting: false,
                bottom_ticks: 0,
                fade_ticks: 0,
                penta_rising: false,
            });
        }
        tied.tick();
        assert_eq!(tied.stinky.first().unwrap().vx, -1.0);

        let mut gated = AdventureState::new_second_stage(0xa45e);
        let pet = gated.stinky.first_mut().unwrap();
        pet.x = 300.0;
        pet.y = 360.0;
        pet.vx = 0.0;
        pet.chase_timer = 4;
        gated.coins.push(Coin {
            id: 3,
            x: 100.0,
            y: 100.0,
            kind: CoinKind::Silver,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            collecting: false,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: false,
        });
        gated.tick();
        assert_eq!(
            (
                gated.stinky.first().unwrap().vx,
                gated.stinky.first().unwrap().chase_timer
            ),
            (0.0, 5)
        );
        gated.tick();
        assert_eq!(
            (
                gated.stinky.first().unwrap().vx,
                gated.stinky.first().unwrap().chase_timer
            ),
            (-1.0, 1)
        );
    }

    #[test]
    fn ordinary_coin_bottom_threshold_is_150_only_in_fresh_first_stage() {
        let mut first = AdventureState::new_adventure(0x56);
        first.coins.push(Coin {
            id: 81,
            x: 500.0,
            y: 370.0,
            kind: CoinKind::Silver,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            collecting: false,
            bottom_ticks: 148,
            fade_ticks: 0,
            penta_rising: false,
        });
        first.tick();
        assert_eq!(
            (first.coins[0].bottom_ticks, first.coins[0].fade_ticks),
            (149, 0)
        );
        first.tick();
        assert_eq!(
            (first.coins[0].bottom_ticks, first.coins[0].fade_ticks),
            (150, 5)
        );

        let mut second = AdventureState::new_second_stage(0x56);
        second.coins.push(Coin {
            id: 82,
            x: 500.0,
            y: 370.0,
            kind: CoinKind::Gold,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            collecting: false,
            bottom_ticks: 18,
            fade_ticks: 0,
            penta_rising: false,
        });
        second.tick();
        assert_eq!(
            (second.coins[0].bottom_ticks, second.coins[0].fade_ticks),
            (19, 0)
        );
        second.tick();
        assert_eq!(
            (second.coins[0].bottom_ticks, second.coins[0].fade_ticks),
            (20, 5)
        );
    }

    #[test]
    fn fresh_fourth_stage_first_tick_uses_ordinary_coin_lifetime() {
        // PB09: the 150-update bottom timer is specific to the first
        // Adventure stage. A fresh 1-4 board must update even with no coins.
        let mut fourth = AdventureState::new_fourth_stage(0x57);
        fourth.tick();
        assert_eq!(fourth.tick, 1);
        fourth.coins.push(Coin {
            id: 83,
            x: 500.0,
            y: 370.0,
            kind: CoinKind::Gold,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            collecting: false,
            bottom_ticks: 18,
            fade_ticks: 0,
            penta_rising: false,
        });
        fourth.tick();
        assert_eq!(
            (fourth.coins[0].bottom_ticks, fourth.coins[0].fade_ticks),
            (19, 0)
        );
        fourth.tick();
        assert_eq!(
            (fourth.coins[0].bottom_ticks, fourth.coins[0].fade_ticks),
            (20, 5)
        );
    }

    #[test]
    fn fourth_stage_itchy_can_leave_balrog_pending_death_during_emergence() {
        let mut board = AdventureState::new_fourth_stage(0x580);
        assert_eq!(
            board.invasion.as_ref().unwrap().plan,
            WavePlan::Fixed(SylvesterKind::Balrog)
        );
        assert_eq!(board.invasion.as_ref().unwrap().countdown, 3000);
        board.validate().unwrap();
        let alien_id = board.id();
        let mut alien = crate::alien::WeakSylvester::spawn_kind(
            SylvesterKind::Balrog,
            alien_id,
            100,
            120,
            1,
            1,
        );
        alien.health = 1.0;
        board.invasion.as_mut().unwrap().actors.push(alien);
        board.invasion.as_mut().unwrap().battle_active = true;
        board.fish_pets[0].x = 140.0;
        board.fish_pets[0].y = 160.0;
        board.fish_pets[0].widget_x = 140;
        board.fish_pets[0].widget_y = 160;
        let first = board.update_objects();
        assert!(first.iter().any(|event| matches!(
            event,
            Event::FishPetHit {
                alien_id: id,
                health: 0.0,
                sound: true,
                ..
            } if *id == alien_id
        )));
        assert!(board.invasion.as_ref().unwrap().has_live_alien());
        assert!(
            board
                .invasion
                .as_ref()
                .unwrap()
                .actors
                .first()
                .unwrap()
                .alive
        );
        board.validate().unwrap();
        board
            .invasion
            .as_mut()
            .unwrap()
            .actors
            .first_mut()
            .unwrap()
            .spawn_ticks = 0;
        let removal = board.update_objects();
        assert_eq!(
            removal
                .iter()
                .filter(|event| matches!(event, Event::AlienDiamondDropped { alien_id: id, .. } if *id == alien_id))
                .count(),
            1
        );
        assert!(!board.invasion.as_ref().unwrap().has_live_alien());
    }

    #[test]
    fn fifth_stage_selected_roster_and_prego_birth_are_explicit() {
        assert!(
            AdventureState::new_fifth_stage(0x581, &[])
                .unwrap()
                .pets
                .is_empty()
        );
        assert!(
            AdventureState::new_fifth_stage(
                0x581,
                &[
                    PetKind::Stinky,
                    PetKind::Niko,
                    PetKind::Itchy,
                    PetKind::Prego,
                ]
            )
            .is_err()
        );
        let mut board = AdventureState::new_fifth_stage(0x581, &[PetKind::Prego]).unwrap();
        assert_eq!(board.pets, [PetKind::Prego]);
        assert!(board.stinky.is_empty());
        assert!(board.niko.is_empty());
        assert_eq!(board.fish_pets.len(), 1);
        board.validate().unwrap();
        board.fish_pets[0].birth_timer = 929;
        board.fish_pets[0].x = 100.0;
        board.fish_pets[0].y = 200.0;
        board.fish_pets[0].widget_x = 100;
        board.fish_pets[0].widget_y = 200;
        let event = board.update_objects();
        assert!(
            event
                .iter()
                .any(|event| matches!(event, Event::PregoBirth { x: 107, y: 225, .. }))
        );
        assert_eq!(board.fish.len(), 3);
        assert_eq!(board.fish.last().unwrap().cannot_be_eaten_ticks, 0);
        assert_eq!(board.fish_pets[0].birth_threshold, 930);

        board
            .invasion
            .as_mut()
            .unwrap()
            .actors
            .push(crate::alien::WeakSylvester::spawn_kind(
                SylvesterKind::Balrog,
                801,
                100,
                120,
                1,
                1,
            ));
        board.invasion.as_mut().unwrap().battle_active = true;
        board.fish_pets[0].birth_timer = 929;
        board.update_objects();
        assert_eq!(board.fish_pets[0].birth_timer, 929);
        assert_eq!(board.fish.len(), 3);
    }

    #[test]
    fn sorted_other_pets_update_before_fish_type_prego() {
        // W1 Board::SortObjects/WidgetUpdateAll puts OtherTypePet before
        // FishTypePet. Two independent due events expose that order.
        let mut board =
            AdventureState::new_fifth_stage(0x582, &[PetKind::Niko, PetKind::Prego]).unwrap();
        board.niko.first_mut().unwrap().cycle = 1232;
        board.fish_pets[0].birth_timer = 929;
        let events = board.update_objects();
        let pearl = events
            .iter()
            .position(|event| {
                matches!(
                    event,
                    Event::Niko {
                        event: NikoEvent::PearlSpawn { .. },
                        ..
                    }
                )
            })
            .unwrap();
        let birth = events
            .iter()
            .position(|event| matches!(event, Event::PregoBirth { .. }))
            .unwrap();
        assert!(pearl < birth);
    }

    #[test]
    fn claimed_coin_can_fund_purchase_once_before_arrival_without_early_credit() {
        let mut state = AdventureState::new_adventure(0xc19);
        state.guppy_unlocked = true;
        state.balance = 90;
        state.coins.push(Coin {
            id: 44,
            x: 500.0,
            y: 39.0,
            kind: CoinKind::Silver,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            collecting: true,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: false,
        });
        assert_eq!(state.available_funds(), 105);
        assert!(
            state
                .apply(Action::BuyGuppy)
                .iter()
                .any(|event| matches!(event, Event::GuppyBought { balance: -10, .. }))
        );
        assert_eq!((state.balance, state.available_funds()), (-10, 5));
        assert!(state.apply(Action::BuyGuppy).iter().any(|event| matches!(
            event,
            Event::Rejected {
                reason: Rejection::InsufficientFunds,
                ..
            }
        )));
        assert!(
            state
                .apply(Action::Click { x: 320.0, y: 200.0 })
                .iter()
                .any(|event| matches!(event, Event::FoodDropped { balance: -15, .. }))
        );
        assert_eq!(state.available_funds(), 0);
        let saved = serde_json::to_vec(&state).unwrap();
        let mut resumed: AdventureState = serde_json::from_slice(&saved).unwrap();
        assert_eq!((resumed.balance, resumed.available_funds()), (-15, 0));
        assert!(
            resumed
                .apply(Action::Click { x: 400.0, y: 200.0 })
                .iter()
                .any(|event| matches!(
                    event,
                    Event::Rejected {
                        reason: Rejection::InsufficientFunds,
                        ..
                    }
                ))
        );
        let credited = resumed.tick();
        assert_eq!(
            credited
                .iter()
                .filter(|event| matches!(
                    event,
                    Event::CoinCredited {
                        coin_id: 44,
                        amount: 15,
                        balance: 0,
                        ..
                    }
                ))
                .count(),
            1
        );
        assert_eq!((resumed.balance, resumed.available_funds()), (0, 0));
        assert!(
            !resumed
                .tick()
                .iter()
                .any(|event| matches!(event, Event::CoinCredited { coin_id: 44, .. }))
        );
    }

    #[test]
    fn second_stage_growth_unlocks_quality_then_quantity_and_egg() {
        let mut state = AdventureState::new_second_stage(0xa12);
        state.validate().unwrap();
        assert!(!state.upgrades.quality_unlocked);
        assert!(!state.egg_unlocked);
        state.fish[0].size = FishSize::Medium;
        state.fish[0].food_ate = state.fish[0].food_needed_to_grow - 1;
        state.fish[0].x = 100.0;
        state.fish[0].y = 100.0;
        state.fish[0].hunger = 400;
        state.fish[1].x = 500.0;
        state.fish[1].y = 300.0;
        state.apply(Action::Click { x: 130.0, y: 130.0 });
        assert!(state.tick().iter().any(|event| matches!(
            event,
            Event::FishGrew {
                size: FishSize::Large,
                ..
            }
        )));
        assert!(state.upgrades.quality_unlocked);
        assert!(!state.upgrades.quantity_unlocked);
        assert!(!state.egg_unlocked);
        state.balance = 200;
        assert!(
            state
                .apply(Action::BuyFoodQuality)
                .iter()
                .any(|event| matches!(
                    event,
                    Event::FoodQualityBought {
                        quality: 1,
                        balance: 0,
                        ..
                    }
                ))
        );
        assert!(state.upgrades.quantity_unlocked);
        assert!(state.egg_unlocked);
        state.balance = 300;
        assert!(
            state
                .apply(Action::BuyFoodQuantity)
                .iter()
                .any(|event| matches!(
                    event,
                    Event::FoodQuantityBought {
                        quantity: 2,
                        balance: 0,
                        ..
                    }
                ))
        );
        state.balance = 20;
        state.apply(Action::Click { x: 300.0, y: 200.0 });
        state.apply(Action::Click { x: 350.0, y: 200.0 });
        assert_eq!(state.food.len(), 2);
        assert!(
            state
                .apply(Action::Click { x: 400.0, y: 200.0 })
                .iter()
                .any(|event| matches!(
                    event,
                    Event::Rejected {
                        reason: Rejection::FoodCapacity,
                        ..
                    }
                ))
        );
    }

    #[test]
    fn third_stage_large_growth_unlocks_both_food_slots_but_not_egg() {
        // W1 Fish::FishOnGrow (1680-1715) takes a distinct tank-1 stage-3
        // branch. Egg follows the Oscar purchase in Board.cpp:4152-4165.
        let mut state = AdventureState::new_third_stage(0xa21);
        assert!(!state.upgrades.quality_unlocked);
        assert!(!state.upgrades.quantity_unlocked);
        assert!(!state.egg_unlocked);
        state.fish[0].size = FishSize::Medium;
        state.fish[0].food_ate = state.fish[0].food_needed_to_grow - 1;
        state.fish[0].x = 100.0;
        state.fish[0].y = 100.0;
        state.fish[0].hunger = 400;
        state.fish[1].x = 500.0;
        state.fish[1].y = 300.0;
        state.apply(Action::Click { x: 130.0, y: 130.0 });
        assert!(state.tick().iter().any(|event| matches!(
            event,
            Event::FishGrew {
                size: FishSize::Large,
                ..
            }
        )));
        assert!(state.upgrades.quality_unlocked);
        assert!(state.upgrades.quantity_unlocked);
        assert!(!state.egg_unlocked);
        assert!(state.oscar_unlocked);
        state.validate().unwrap();
        state.balance = 200;
        state.apply(Action::BuyFoodQuality);
        assert!(!state.egg_unlocked);
        state.balance = OSCAR_PRICE;
        let bought = state.apply(Action::BuyOscar);
        assert!(
            bought
                .iter()
                .any(|event| matches!(event, Event::OscarBought { balance: 0, .. }))
        );
        assert_eq!(state.oscars.len(), 1);
        assert!(state.weapon_unlocked);
        assert!(state.egg_unlocked);
        state.balance = WEAPON_PRICE;
        assert!(state.apply(Action::BuyWeapon).iter().any(|event| matches!(
            event,
            Event::WeaponBought {
                strength: 3,
                balance: 0,
                ..
            }
        )));
        state.fish.clear();
        assert!(
            state.has_live_fish(),
            "a live Oscar prevents Game Over without guppies"
        );
        state.validate().unwrap();
    }

    #[test]
    fn max_weapon_hold_fires_on_old_board_count_with_strict_elapsed_gate() {
        // W1 Board.cpp:818-824: strength 12, old count divisible by five,
        // more than 100 ms, and cursor below the menu area.
        let mut state = AdventureState::new_third_stage(0xa22);
        state.weapon_strength = 12;
        state.weapon_unlocked = true;
        state.oscar_unlocked = true;
        state.upgrades.quality_unlocked = true;
        state.upgrades.quantity_unlocked = true;
        let wave = state.invasion.as_mut().unwrap();
        wave.countdown = 3000;
        wave.actors.push(crate::alien::WeakSylvester::spawn_kind(
            SylvesterKind::Strong,
            900,
            100,
            120,
            1,
            1,
        ));
        wave.battle_active = true;
        state.tick = 5;
        state.apply(Action::HoldFire {
            x: 180.0,
            y: 200.0,
            elapsed_ms: 100,
        });
        assert!(!state.begin_tick().iter().any(|event| matches!(
            event,
            Event::Invasion {
                event: InvasionEvent::LaserFired { .. },
                ..
            }
        )));
        state.tick = 10;
        state.apply(Action::HoldFire {
            x: 180.0,
            y: 200.0,
            elapsed_ms: 101,
        });
        assert!(state.begin_tick().iter().any(|event| matches!(
            event,
            Event::Invasion {
                event: InvasionEvent::AlienHit { health: 24.0, .. },
                ..
            }
        )));
        assert_eq!(state.tick, 11);
        state
            .invasion
            .as_mut()
            .unwrap()
            .actors
            .first_mut()
            .unwrap()
            .hit_ticks = 0;
        state.apply(Action::HoldFire {
            x: 180.0,
            y: 200.0,
            elapsed_ms: 999,
        });
        assert!(!state.begin_tick().iter().any(|event| matches!(
            event,
            Event::Invasion {
                event: InvasionEvent::LaserFired { .. },
                ..
            }
        )));
    }

    #[test]
    fn starving_oscar_emits_due_diamond_then_keeps_a_persisted_corpse() {
        // W1 Fish::Update continues production after Die schedules removal.
        let mut state = AdventureState::new_third_stage(0xa23);
        let mut oscar = OscarState::spawn_bought(901, &mut |_| 0);
        oscar.hunger = 1;
        oscar.bought_timer = 0;
        oscar.coin_timer = oscar.coin_threshold - 1;
        state.oscars.push(oscar);
        let events = state.update_objects();
        let diamond = events
            .iter()
            .position(|event| {
                matches!(
                    event,
                    Event::CoinDropped {
                        fish_id: 901,
                        kind: CoinKind::Diamond,
                        ..
                    }
                )
            })
            .unwrap();
        let death = events
            .iter()
            .position(|event| matches!(event, Event::OscarDied { oscar_id: 901, .. }))
            .unwrap();
        assert!(diamond < death);
        assert!(state.oscars.is_empty());
        assert_eq!(state.dead_oscars.len(), 1);
        assert_eq!(state.dead_oscars[0].remaining_ticks, 125);
        state.update_objects();
        assert_eq!(state.dead_oscars[0].remaining_ticks, 124);
        assert_eq!(
            state
                .coins
                .iter()
                .filter(|coin| coin.kind == CoinKind::Diamond)
                .count(),
            1
        );
    }

    #[test]
    fn held_food_uses_old_clock_after_delay_decrement_and_not_save_boundary() {
        let mut allowed = AdventureState::new_second_stage(0xa13);
        allowed.tick = 15; // Quantity one: old count divisible by 16 - 1.
        allowed.invasion.as_mut().unwrap().food_delay = 1;
        allowed.apply(Action::HoldFeed {
            x: 320.0,
            y: 200.0,
            elapsed_ms: 201,
        });
        assert!(allowed.food.is_empty());
        let saved = serde_json::to_vec(&allowed).unwrap();
        let restored: AdventureState = serde_json::from_slice(&saved).unwrap();
        assert!(restored.held_feed.is_none());
        assert!(restored.food.is_empty());
        assert!(
            allowed
                .begin_tick()
                .iter()
                .any(|event| matches!(event, Event::FoodDropped { .. }))
        );
        assert_eq!(allowed.tick, 16);
        assert_eq!(allowed.food[0].ineligible_ticks, 20);

        let mut blocked = AdventureState::new_second_stage(0xa14);
        blocked.tick = 15;
        blocked.invasion.as_mut().unwrap().food_delay = 2;
        blocked.apply(Action::HoldFeed {
            x: 320.0,
            y: 200.0,
            elapsed_ms: 201,
        });
        blocked.begin_tick();
        assert_eq!(blocked.invasion.as_ref().unwrap().food_delay, 1);
        assert!(blocked.food.is_empty());
    }

    #[test]
    fn hidden_live_alien_freezes_hunger_coin_timer_and_stinky_collection() {
        let mut state = AdventureState::new_second_stage(0xa15);
        let wave = state.invasion.as_mut().unwrap();
        wave.countdown = 1;
        wave.warning = Some(crate::invasion::WarningCoords {
            first_x: 20,
            first_y: 105,
            second_x: 469,
            second_y: 299,
        });
        state.fish[0].size = FishSize::Medium;
        state.fish[0].coin_timer = 10;
        state.fish[0].coin_threshold = 11;
        let hunger = state.fish[0].hunger;
        let stinky_x = state.stinky.first().unwrap().x;
        state.coins.push(Coin {
            id: 700,
            x: stinky_x,
            y: 360.0,
            kind: CoinKind::Silver,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            collecting: false,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: false,
        });
        assert!(state.begin_tick().iter().any(|event| matches!(
            event,
            Event::Invasion {
                event: InvasionEvent::AlienSpawned { .. },
                ..
            }
        )));
        state.update_objects();
        assert_eq!(
            state
                .invasion
                .as_ref()
                .unwrap()
                .actors
                .first()
                .unwrap()
                .spawn_ticks,
            14
        );
        assert_eq!(state.fish[0].hunger, hunger);
        assert_eq!(state.fish[0].coin_timer, 10);
        assert_eq!(state.coins.len(), 1);
        assert_eq!(state.stinky.first().unwrap().specialty_timer, 1);
    }

    #[test]
    fn ordinary_coin_forwards_world_click_only_when_over_alien_widget() {
        let mut state = AdventureState::new_second_stage(0xa16);
        let wave = state.invasion.as_mut().unwrap();
        wave.countdown = 3000;
        wave.actors
            .push(crate::alien::WeakSylvester::spawn(900, 100, 120, 1, 1));
        wave.battle_active = true;
        state.coins.push(Coin {
            id: 901,
            x: 160.0,
            y: 180.0,
            kind: CoinKind::Silver,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            collecting: false,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: false,
        });
        let overlap = state.apply(Action::Click { x: 170.0, y: 190.0 });
        assert!(overlap.iter().any(|event| matches!(
            event,
            Event::Invasion {
                event: InvasionEvent::AlienHit {
                    id: 900,
                    health: 44.0
                },
                ..
            }
        )));
        assert!(
            overlap
                .iter()
                .any(|event| matches!(event, Event::CoinCollectionStarted { coin_id: 901, .. }))
        );
        state.coins.push(Coin {
            id: 902,
            x: 350.0,
            y: 200.0,
            kind: CoinKind::Silver,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            collecting: false,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: false,
        });
        let away = state.apply(Action::Click { x: 360.0, y: 210.0 });
        assert!(
            away.iter()
                .any(|event| matches!(event, Event::CoinCollectionStarted { coin_id: 902, .. }))
        );
        assert!(!away.iter().any(|event| matches!(
            event,
            Event::Invasion {
                event: InvasionEvent::LaserFired { .. },
                ..
            }
        )));

        // The integer widget includes its left/top edge for forwarding,
        // while the alien's double-coordinate hit test excludes that edge.
        state.coins.push(Coin {
            id: 903,
            x: 100.0,
            y: 120.0,
            kind: CoinKind::Silver,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            collecting: false,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: false,
        });
        let edge = state.apply(Action::Click { x: 100.0, y: 120.0 });
        assert!(edge.iter().any(|event| matches!(
            event,
            Event::Invasion {
                event: InvasionEvent::LaserFired { .. },
                ..
            }
        )));
        assert!(!edge.iter().any(|event| matches!(
            event,
            Event::Invasion {
                event: InvasionEvent::AlienHit { .. },
                ..
            }
        )));
        assert!(
            edge.iter()
                .any(|event| matches!(event, Event::CoinCollectionStarted { coin_id: 903, .. }))
        );
    }

    #[test]
    fn alien_diamond_has_one_origin_and_player_or_stinky_credit() {
        let mut player = AdventureState::new_second_stage(0xa17);
        player.invasion.as_mut().unwrap().countdown = 3000;
        let mut actor = crate::alien::WeakSylvester::spawn(910, 100, 120, 1, 1);
        actor.health = 6.0;
        player.invasion.as_mut().unwrap().actors.push(actor);
        player.invasion.as_mut().unwrap().battle_active = true;
        let shot = player.apply(Action::Click { x: 180.0, y: 200.0 });
        let coin_id = shot
            .iter()
            .find_map(|event| match event {
                Event::AlienDiamondDropped {
                    alien_id: 910,
                    coin_id,
                    ..
                } => Some(*coin_id),
                _ => None,
            })
            .unwrap();
        assert!(!player.invasion.as_ref().unwrap().has_live_alien());
        assert_eq!(player.balance, 200);
        assert_eq!(
            player
                .coins
                .iter()
                .filter(|coin| coin.id == coin_id && coin.kind == CoinKind::Diamond)
                .count(),
            1
        );
        let claim = player.apply(Action::Click { x: 130.0, y: 150.0 });
        assert!(claim.iter().any(|event| matches!(event, Event::CoinCollectionStarted { coin_id: id, .. } if *id == coin_id)));
        let mut credits = 0;
        for _ in 0..100 {
            credits += player.tick().iter().filter(|event| matches!(event, Event::CoinCredited { coin_id: id, amount: 200, .. } if *id == coin_id)).count();
        }
        assert_eq!(credits, 1);
        assert_eq!(player.balance, 400);

        let mut pet = AdventureState::new_second_stage(0xa18);
        pet.invasion.as_mut().unwrap().countdown = 3000;
        let mut actor = crate::alien::WeakSylvester::spawn(920, 100, 120, 1, 1);
        actor.health = 6.0;
        pet.invasion.as_mut().unwrap().actors.push(actor);
        pet.invasion.as_mut().unwrap().battle_active = true;
        pet.apply(Action::Click { x: 180.0, y: 200.0 });
        let stinky = pet.stinky.first_mut().unwrap();
        stinky.x = 125.0;
        stinky.y = 145.0;
        let collected = pet.update_objects();
        assert!(collected.iter().any(|event| matches!(
            event,
            Event::PetCollectedCoin {
                pet: PetKind::Stinky,
                amount: 200,
                ..
            }
        )));
        assert_eq!(pet.balance, 400);
        assert!(pet.coins.is_empty());
    }

    #[test]
    fn niko_pearl_stays_out_of_stinky_list_and_credits_once_after_flight() {
        let mut state = AdventureState::new_third_stage(0xa19);
        state.validate().unwrap();
        state.niko.first_mut().unwrap().cycle = 1232;
        state.stinky.first_mut().unwrap().x = 96.0;
        state.stinky.first_mut().unwrap().y = 251.0;
        assert!(state.update_objects().iter().any(|event| matches!(
            event,
            Event::Niko {
                event: NikoEvent::PearlSpawn { .. },
                ..
            }
        )));
        assert_eq!(state.pearls.len(), 1);
        state.update_objects();
        assert_eq!(state.pearls.len(), 1);
        assert!(state.coins.is_empty());
        state.balance = 0;
        state.guppy_unlocked = true;
        let pearl_id = state.pearls[0].id;
        assert!(state.apply(Action::Click { x: 100.0, y: 260.0 }).iter().any(|event| matches!(event, Event::PearlCollectionStarted { pearl_id: id, .. } if *id == pearl_id)));
        assert!(state.niko.first().unwrap().pearl_taken);
        assert_eq!(state.available_funds(), 250);
        assert!(
            state
                .apply(Action::BuyGuppy)
                .iter()
                .any(|event| matches!(event, Event::GuppyBought { balance: -100, .. }))
        );
        let mut credits = 0;
        for _ in 0..100 {
            credits += state.tick().iter().filter(|event| matches!(event, Event::PearlCredited { pearl_id: id, amount: 250, .. } if *id == pearl_id)).count();
        }
        assert_eq!(credits, 1);
        assert_eq!(state.balance, 150);
        assert!(state.pearls.is_empty());
    }

    #[test]
    fn modal_pause_and_reload_preserve_wave_without_board_time() {
        let mut state = AdventureState::new_second_stage(0xa20);
        let wave = state.invasion.as_mut().unwrap();
        wave.countdown = 276;
        wave.danger_shown = true;
        wave.pending_modal = Some(crate::invasion::InvasionTip::Danger);
        wave.food_delay = 2;
        wave.post_spawn_flash_ticks = 3;
        state.apply(Action::HoldFeed {
            x: 320.0,
            y: 200.0,
            elapsed_ms: 999,
        });
        let saved = serde_json::to_vec(&state).unwrap();
        let mut resumed: AdventureState = serde_json::from_slice(&saved).unwrap();
        resumed.validate().unwrap();
        assert!(resumed.held_feed.is_none());
        resumed.paused_board_update();
        assert_eq!(resumed.tick, 0);
        assert_eq!(resumed.invasion.as_ref().unwrap().countdown, 276);
        assert_eq!(resumed.invasion.as_ref().unwrap().food_delay, 1);
        assert_eq!(resumed.invasion.as_ref().unwrap().post_spawn_flash_ticks, 3);
    }

    #[test]
    fn fifth_stage_third_egg_opens_bonus_stage_without_direct_second_tank_jump() {
        let mut board = AdventureState::new_fifth_stage(0x1501, &[]).unwrap();
        board.egg_unlocked = true;
        board.weapon_unlocked = true;
        board.oscar_unlocked = true;
        board.upgrades.quality_unlocked = true;
        board.upgrades.quantity_unlocked = true;
        board.eggs = 2;
        board.balance = FIFTH_STAGE_EGG_PRICE;
        let events = board.apply(Action::BuyEgg);
        assert!(events.iter().any(|event| matches!(
            event,
            Event::LevelCompleted {
                next_tank: 1,
                next_level: 6,
                ..
            }
        )));
        assert!(board.victory);
        assert_eq!(board.balance, 0);
        board.validate().unwrap();
    }

    #[test]
    fn selected_second_tank_roster_and_locked_slot_chain() {
        let pets = [PetKind::Niko, PetKind::Zorf];
        let mut board = AdventureState::new_tank2_first_stage(0x2101, &pets).unwrap();
        board.validate().unwrap();
        assert_eq!(board.pets, pets);
        assert_eq!(
            (
                board.niko.first().unwrap().anchor_x,
                board.niko.first().unwrap().anchor_y
            ),
            (175, 163)
        );
        assert_eq!(board.egg_price, 750);
        assert_eq!(board.balance, 200);
        assert_eq!(board.fish.len(), 2);
        assert!(
            board
                .fish
                .iter()
                .all(|fish| fish.food_ate == 2 && !fish.beginner)
        );
        assert_eq!(board.invasion.as_ref().unwrap().countdown, 3000);
        assert!(board.apply(Action::BuyOscar).iter().any(|event| matches!(
            event,
            Event::Rejected {
                reason: Rejection::Locked,
                ..
            }
        )));
        assert!(board.apply(Action::BuyWeapon).iter().any(|event| matches!(
            event,
            Event::Rejected {
                reason: Rejection::Locked,
                ..
            }
        )));
        board.egg_unlocked = true;
        board.eggs = 2;
        board.balance = 750;
        assert!(board.apply(Action::BuyEgg).iter().any(|event| matches!(
            event,
            Event::LevelCompleted {
                next_tank: 2,
                next_level: 2,
                ..
            }
        )));
        assert!(board.victory);
        assert!(AdventureState::new_tank2_first_stage(1, &[PetKind::Zorf, PetKind::Niko]).is_err());
    }

    #[test]
    fn armed_potion_survives_capacity_rejection_and_only_accepted_manual_drop_consumes_it() {
        let mut board = AdventureState::new_tank2_first_stage(0x2102, &[]).unwrap();
        board.upgrades.quality_unlocked = true;
        board.upgrades.quantity_unlocked = true;
        board.potion_unlocked = true;
        board.egg_unlocked = true;
        board.balance = 300;
        board.apply(Action::Click { x: 200.0, y: 200.0 });
        assert_eq!(board.balance, 295);
        board.apply(Action::BuyPotion);
        assert_eq!(board.balance, 45);
        assert!(board.potion_armed);
        board.apply(Action::BuyPotion);
        assert_eq!(board.balance, 45);
        assert!(
            board
                .apply(Action::Click { x: 300.0, y: 200.0 })
                .iter()
                .any(|event| matches!(
                    event,
                    Event::Rejected {
                        reason: Rejection::FoodCapacity,
                        ..
                    }
                ))
        );
        assert!(board.potion_armed);
        board.food.clear();
        assert!(
            board
                .apply(Action::Click { x: 300.0, y: 200.0 })
                .iter()
                .any(|event| matches!(
                    event,
                    Event::FoodDropped {
                        potion: true,
                        balance: 45,
                        ..
                    }
                ))
        );
        assert_eq!(board.food[0].quality, 3);
        assert!(!board.potion_armed);
        board.validate().unwrap();
    }

    #[test]
    fn potion_kills_medium_but_finishes_meal_and_due_silver_coin() {
        let mut board = AdventureState::new_tank2_first_stage(0x2103, &[]).unwrap();
        let fish = &mut board.fish[0];
        fish.x = 100.0;
        fish.y = 100.0;
        fish.size = FishSize::Medium;
        fish.hunger = 400;
        fish.coin_timer = fish.coin_threshold - 1;
        let fish_id = fish.id;
        board.food.push(Food {
            id: 90,
            x: 120.0,
            y: 120.0,
            frame: 0,
            food_type: FoodType::Ordinary,
            picked_up: false,
            ineligible_ticks: 0,
            removal_ticks: 0,
            quality: 3,
            direction: 0,
            vx: 0.0,
            vy: 0.0,
            animation_period: 3,
            free_from_zorf: false,
            nimbus_rising: false,
        });
        let events = board.tick();
        assert!(events.iter().any(|event| matches!(event, Event::PotionExploded { fish_id: Some(id), food_id: 90, .. } if *id == fish_id)));
        assert!(events.iter().any(|event| matches!(event, Event::FoodEaten { fish_id: id, food_id: 90, .. } if *id == fish_id)));
        assert!(events.iter().any(|event| matches!(event, Event::CoinDropped { fish_id: id, kind: CoinKind::Silver, .. } if *id == fish_id)));
        assert_eq!(board.dead_fish[0].x, 100.0);
        assert!(!board.fish[0].alive);
        assert!(board.food.is_empty());
    }

    #[test]
    fn large_potion_keeps_growth_points_and_star_can_crown_later() {
        let mut board = AdventureState::new_tank2_first_stage(0x2104, &[]).unwrap();
        let fish = &mut board.fish[0];
        fish.x = 100.0;
        fish.y = 100.0;
        fish.size = FishSize::Large;
        fish.hunger = 400;
        fish.food_ate = fish.food_needed_to_grow * 15 - 1;
        fish.coin_timer = fish.coin_threshold - 1;
        let old_points = fish.food_ate;
        board.food.push(Food {
            id: 91,
            x: 120.0,
            y: 120.0,
            frame: 0,
            food_type: FoodType::Ordinary,
            picked_up: false,
            ineligible_ticks: 0,
            removal_ticks: 0,
            quality: 3,
            direction: 0,
            vx: 0.0,
            vy: 0.0,
            animation_period: 3,
            free_from_zorf: false,
            nimbus_rising: false,
        });
        let events = board.tick();
        assert_eq!(board.fish[0].size, FishSize::Star);
        assert_eq!(board.fish[0].food_ate, old_points);
        assert!(events.iter().any(|event| matches!(
            event,
            Event::CoinDropped {
                kind: CoinKind::Star,
                ..
            }
        )));
        board.fish[0].hunger = 400;
        let x = board.fish[0].x + 20.0;
        let y = board.fish[0].y + 20.0;
        board.food.push(Food {
            id: 92,
            x,
            y,
            frame: 0,
            food_type: FoodType::Ordinary,
            picked_up: false,
            ineligible_ticks: 0,
            removal_ticks: 0,
            quality: 0,
            direction: 0,
            vx: 0.0,
            vy: 0.0,
            animation_period: 3,
            free_from_zorf: false,
            nimbus_rising: false,
        });
        board.tick();
        assert_eq!(board.fish[0].size, FishSize::Crowned);
        assert_eq!(board.fish[0].food_ate, old_points + 1);
    }

    #[test]
    fn overdue_zorf_drops_free_food_only_after_hunger_crosses_below_300() {
        let mut board = AdventureState::new_tank2_first_stage(0x2105, &[PetKind::Zorf]).unwrap();
        board.fish[0].hunger = 301;
        board.fish[1].hunger = 301;
        board.fish_pets[0].food_timer = 64;
        board.tick();
        assert_eq!(board.fish[0].hunger, 300);
        assert_eq!(board.fish_pets[0].food_timer, 65);
        assert!(board.food.is_empty());
        let events = board.tick();
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Event::ZorfFoodDropped { .. }))
        );
        assert_eq!(board.food.len(), 1);
        assert_eq!(board.food[0].quality, 1);
        assert_eq!(board.food[0].ineligible_ticks, 0);
        assert!(board.food[0].free_from_zorf);
        assert_ne!(board.food[0].direction, 0);
        board.potion_unlocked = true;
        board.potion_armed = true;
        assert!(
            board
                .apply(Action::Click { x: 300.0, y: 200.0 })
                .iter()
                .any(|event| matches!(event, Event::FoodDropped { potion: true, .. }))
        );
        assert!(!board.potion_armed);
        assert_eq!(board.food.len(), 2);
        assert_eq!(board.food[1].quality, 3);
    }

    #[test]
    fn second_tank_starcatcher_purchase_unlocks_weapon_and_egg_only_on_success() {
        let mut board =
            AdventureState::new_tank2_second_stage(0x2201, &[PetKind::Niko, PetKind::Clyde])
                .unwrap();
        assert_eq!((board.balance, board.egg_price), (200, 3000));
        assert_eq!(
            board.invasion.as_ref().unwrap().plan,
            WavePlan::Fixed(SylvesterKind::Balrog)
        );
        assert_eq!(board.invasion.as_ref().unwrap().countdown, 3000);
        assert_eq!(
            (
                board.niko.first().unwrap().anchor_x,
                board.niko.first().unwrap().anchor_y
            ),
            (175, 163)
        );
        assert!(!board.clyde.is_empty());
        assert!(
            board
                .apply(Action::BuyStarcatcher)
                .iter()
                .any(|event| matches!(
                    event,
                    Event::Rejected {
                        reason: Rejection::Locked,
                        ..
                    }
                ))
        );
        board.upgrades.quality_unlocked = true;
        board.upgrades.quantity_unlocked = true;
        board.potion_unlocked = true;
        board.starcatcher_unlocked = true;
        assert!(
            board
                .apply(Action::BuyStarcatcher)
                .iter()
                .any(|event| matches!(
                    event,
                    Event::Rejected {
                        reason: Rejection::InsufficientFunds,
                        ..
                    }
                ))
        );
        assert!(!board.weapon_unlocked && !board.egg_unlocked);
        board.balance = 750;
        let events = board.apply(Action::BuyStarcatcher);
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Event::StarcatcherBought { balance: 0, .. }))
        );
        assert_eq!(board.starcatchers.len(), 1);
        assert!(board.weapon_unlocked && board.egg_unlocked);
        assert_eq!(board.starcatchers[0].cannot_be_eaten_ticks, 0);
        board.validate().unwrap();
    }

    #[test]
    fn two_starcatchers_consume_one_star_once_and_emit_one_distinct_rising_diamond() {
        let mut board = AdventureState::new_tank2_second_stage(0x2202, &[]).unwrap();
        board.starcatchers = [101, 102]
            .map(|id| {
                let mut actor = StarcatcherState::spawn_bought(id, &mut |_| 1);
                actor.x = 100.0;
                actor.y = 200.0;
                actor.widget_x = 100;
                actor.widget_y = 200;
                actor.hunger = 899;
                actor.bought_timer = 0;
                actor
            })
            .to_vec();
        board.coins.push(Coin {
            id: 103,
            x: 100.0,
            y: 200.0,
            kind: CoinKind::Star,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            collecting: false,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: false,
        });
        let events = board.update_objects();
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, Event::StarcatcherAteStar { .. }))
                .count(),
            1
        );
        assert_eq!(board.coins.len(), 1);
        assert_eq!(board.coins[0].kind, CoinKind::DiamondPenta);
        assert_ne!(board.coins[0].id, 103);
        assert!(board.coins[0].penta_rising);
        // GameObject only raises hunger to 300 when it was below 300;
        // 898 + 900 is capped at 1300 for this meal.
        assert_eq!(board.starcatchers[0].hunger, 1300);
        assert_eq!(board.starcatchers[1].hunger, 898);

        board.coins.push(Coin {
            id: 105,
            x: 100.0,
            y: 200.0,
            kind: CoinKind::Star,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            collecting: true,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: false,
        });
        board.starcatchers[0].hunger = 899;
        board.starcatchers[1].hunger = 899;
        let later = board.update_objects();
        assert!(
            !later
                .iter()
                .any(|event| matches!(event, Event::StarcatcherAteStar { .. }))
        );
        assert!(board.coins.iter().any(|coin| coin.id == 105));
    }

    #[test]
    fn special_diamond_turns_without_moving_then_falls_and_credits_two_hundred_once() {
        let mut board = AdventureState::new_tank2_second_stage(0x2203, &[]).unwrap();
        board.coins.push(Coin {
            id: 104,
            x: 500.0,
            y: 119.5,
            kind: CoinKind::DiamondPenta,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            collecting: false,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: true,
        });
        board.update_coins(&mut Vec::new());
        assert_eq!(board.coins[0].y, 119.5);
        assert!(!board.coins[0].penta_rising);
        board.update_coins(&mut Vec::new());
        assert_eq!(board.coins[0].y, 121.0);
        board.coins[0].collecting = true;
        board.coins[0].y = 39.0;
        let mut events = Vec::new();
        board.update_coins(&mut events);
        assert_eq!(board.balance, 400);
        assert!(board.coins.is_empty());
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    Event::CoinCredited {
                        coin_id: 104,
                        amount: 200,
                        ..
                    }
                ))
                .count(),
            1
        );
        board.update_coins(&mut events);
        assert_eq!(board.balance, 400);
    }

    #[test]
    fn current_board_rejects_cross_list_identity_reuse_and_unallocated_actor_ids() {
        let mut board = AdventureState::new_tank2_second_stage(0x2204, &[]).unwrap();
        board.upgrades.quality_unlocked = true;
        board.upgrades.quantity_unlocked = true;
        board.potion_unlocked = true;
        board.starcatcher_unlocked = true;
        board.weapon_unlocked = true;
        board.egg_unlocked = true;
        let guppy_id = board.fish[0].id;
        let mut actor = StarcatcherState::spawn_bought(guppy_id, &mut |_| 1);
        board.starcatchers.push(actor.clone());
        assert!(
            board.validate().is_err(),
            "an alien bite by ID would remove both actors"
        );

        actor.id = board.id();
        board.starcatchers[0] = actor;
        board.validate().unwrap();
        board.starcatchers[0].id = board.next_id;
        assert!(board.validate().is_err(), "next_id has not been allocated");
    }

    #[test]
    fn dead_guppy_and_its_corpse_share_only_their_own_identity() {
        let mut board = AdventureState::new_adventure(0x2205);
        board.fish[0].alive = false;
        let corpse = DeadFish::from_live(&board.fish[0]);
        board.dead_fish.push(corpse.clone());
        board.validate().unwrap();
        board.dead_fish.push(corpse);
        assert!(board.validate().is_err());
        board.dead_fish.pop();
        board.fish[0].alive = true;
        assert!(board.validate().is_err());
        board.dead_fish.clear();
        board.validate().unwrap();
        board.next_id = u64::MAX;
        assert!(
            board.validate().is_err(),
            "the next allocation would overflow"
        );
    }

    #[test]
    fn gus_click_is_free_and_forwarded_coin_claim_does_not_fire_a_laser() {
        let mut board = AdventureState::new_tank2_third_stage(0x2301, &[]).unwrap();
        board.validate().unwrap();
        let gus_id = board.id();
        let mut gus =
            crate::alien::WeakSylvester::spawn_kind(SylvesterKind::Gus, gus_id, 100, 100, 1, 1);
        gus.spawn_ticks = 0;
        board.invasion.as_mut().unwrap().actors.push(gus);
        board.invasion.as_mut().unwrap().battle_active = true;
        let coin_id = board.id();
        board.coins.push(Coin {
            id: coin_id,
            x: 160.0,
            y: 160.0,
            kind: CoinKind::Silver,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            collecting: false,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: false,
        });
        let events = board.apply(Action::Click { x: 180.0, y: 180.0 });
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Event::FoodDropped { balance: 200, .. }))
        );
        assert!(events.iter().any(|event| matches!(event, Event::CoinCollectionStarted { coin_id: id, .. } if *id == coin_id)));
        assert!(!events.iter().any(|event| matches!(
            event,
            Event::Invasion {
                event: InvasionEvent::LaserFired { .. } | InvasionEvent::AlienHit { .. },
                ..
            }
        )));
        assert_eq!((board.balance, board.food[0].ineligible_ticks), (200, 20));
        assert_eq!(
            board
                .invasion
                .as_ref()
                .unwrap()
                .actors
                .first()
                .unwrap()
                .health,
            100.0
        );
    }

    #[test]
    fn gus_held_repeat_charges_five_even_when_armed_capacity_rejects() {
        let mut board = AdventureState::new_tank2_third_stage(0x2302, &[]).unwrap();
        let gus_id = board.id();
        let gus =
            crate::alien::WeakSylvester::spawn_kind(SylvesterKind::Gus, gus_id, 100, 100, 1, 1);
        board.invasion.as_mut().unwrap().actors.push(gus);
        board.invasion.as_mut().unwrap().battle_active = true;
        board.apply(Action::HoldFeed {
            x: 300.0,
            y: 399.0,
            elapsed_ms: 201,
        });
        let accepted = board.begin_tick();
        assert!(
            accepted
                .iter()
                .any(|event| matches!(event, Event::FoodDropped { balance: 195, .. }))
        );
        assert_eq!(board.food[0].ineligible_ticks, 20);
        board.potion_unlocked = true;
        board.potion_armed = true;
        board.tick = 15; // Old board count satisfies the held-feed cadence.
        board.apply(Action::HoldFeed {
            x: 300.0,
            y: 399.0,
            elapsed_ms: 201,
        });
        let rejected = board.begin_tick();
        assert!(rejected.iter().any(|event| matches!(
            event,
            Event::Rejected {
                reason: Rejection::FoodCapacity,
                ..
            }
        )));
        assert_eq!(board.balance, 190);
        assert!(board.potion_armed);
        assert_eq!(board.food.len(), 1);
    }

    #[test]
    fn gus_initial_food_delay_uses_strict_integer_widget_center() {
        let mut board = AdventureState::new_tank2_third_stage(0x2304, &[]).unwrap();
        let gus_id = board.id();
        let mut gus =
            crate::alien::WeakSylvester::spawn_kind(SylvesterKind::Gus, gus_id, 100, 100, 1, 1);
        gus.spawn_ticks = 0;
        board.invasion.as_mut().unwrap().actors.push(gus);
        board.invasion.as_mut().unwrap().battle_active = true;
        board.apply(Action::Click { x: 105.0, y: 180.0 });
        assert_eq!(board.food[0].ineligible_ticks, 0);
        board.food.clear();
        board.apply(Action::Click { x: 106.0, y: 180.0 });
        assert_eq!(board.food[0].ineligible_ticks, 20);
        board.food.clear();
        board.invasion.as_mut().unwrap().actors[0].x = 100.5;
        board.apply(Action::Click { x: 215.0, y: 180.0 });
        assert_eq!(board.food[0].ineligible_ticks, 0);
        assert_eq!(board.balance, 200);
    }

    #[test]
    fn vert_membership_gate_uses_alien_removal_order_and_one_gold_transaction() {
        let mut board = AdventureState::new_tank2_third_stage(0x2303, &[PetKind::Vert]).unwrap();
        board.fish_pets[0].coin_timer = 215;
        let gus_id = board.id();
        let mut gus =
            crate::alien::WeakSylvester::spawn_kind(SylvesterKind::Gus, gus_id, 100, 100, 1, 1);
        gus.spawn_ticks = 15;
        gus.health = 0.0;
        board.invasion.as_mut().unwrap().actors.push(gus);
        board.invasion.as_mut().unwrap().battle_active = true;
        let blocked = board.update_objects();
        assert!(
            !blocked
                .iter()
                .any(|event| matches!(event, Event::VertGoldDropped { .. }))
        );
        assert!(board.invasion.as_ref().unwrap().has_live_alien());
        assert_eq!(board.fish_pets[0].coin_timer, 215);

        board
            .invasion
            .as_mut()
            .unwrap()
            .actors
            .first_mut()
            .unwrap()
            .spawn_ticks = 0;
        let released = board.update_objects();
        assert!(
            released
                .iter()
                .any(|event| matches!(event, Event::VertGoldDropped { .. }))
        );
        assert_eq!(
            board
                .coins
                .iter()
                .filter(|coin| coin.kind == CoinKind::Gold)
                .count(),
            1
        );
        assert!(!board.invasion.as_ref().unwrap().has_live_alien());
        assert!(board.invasion.as_ref().unwrap().dead_aliens.is_empty());
    }

    #[test]
    fn destructor_fresh_search_assigns_three_farthest_unassigned_targets_in_tie_order() {
        let mut board = AdventureState::new_tank2_fourth_stage(0x2401, &[]).unwrap();
        let first = board.fish[0].id;
        let second = board.fish[1].id;
        board.fish[0].x = 400.0;
        board.fish[0].y = 300.0;
        board.fish[1].x = 400.0;
        board.fish[1].y = 300.0;
        let third = board.make_fish(180.0, 180.0, false, false);
        let third_id = third.id;
        board.fish.push(third);
        let alien_id = board.id();
        let mut alien = crate::alien::WeakSylvester::spawn_kind(
            SylvesterKind::Destructor,
            alien_id,
            100,
            100,
            1,
            1,
        );
        alien.spawn_ticks = 0;
        alien.launch_ticks = 75;
        board.invasion.as_mut().unwrap().actors.push(alien);
        board.invasion.as_mut().unwrap().battle_active = true;
        let mut events = Vec::new();
        board.update_invasion_objects(&mut events);
        assert_eq!(
            board
                .missiles
                .iter()
                .map(|missile| missile.target_id)
                .collect::<Vec<_>>(),
            [Some(first), Some(second), Some(third_id)]
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, Event::MissileLaunched { .. }))
                .count(),
            3
        );
        assert!(
            board
                .invasion
                .as_ref()
                .unwrap()
                .actors
                .first()
                .unwrap()
                .reload_ticks
                >= 150
        );
        board.validate().unwrap();
    }

    #[test]
    fn one_click_hits_overlapping_destructor_and_vulnerable_missile_then_missile_only_completion() {
        let mut board = AdventureState::new_tank2_fourth_stage(0x2402, &[]).unwrap();
        let alien_id = board.id();
        let mut alien = crate::alien::WeakSylvester::spawn_kind(
            SylvesterKind::Destructor,
            alien_id,
            100,
            100,
            1,
            1,
        );
        alien.spawn_ticks = 0;
        board.invasion.as_mut().unwrap().actors.push(alien);
        board.invasion.as_mut().unwrap().battle_active = true;
        let missile_id = board.id();
        let mut missile = ClassicMissile::launch(missile_id, board.fish[0].id, 100, 280, 0);
        missile.immunity_ticks = 0;
        board.missiles.push(missile);
        let events = board.apply(Action::Click { x: 140.0, y: 320.0 });
        assert!(events.iter().any(|event| matches!(
            event,
            Event::Invasion {
                event: InvasionEvent::AlienHit { health: 144.0, .. },
                ..
            }
        )));
        assert!(events.iter().any(|event| matches!(event, Event::MissileRemoved { missile_id: id, .. } if *id == missile_id)));
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    Event::Invasion {
                        event: InvasionEvent::LaserFired { .. },
                        ..
                    }
                ))
                .count(),
            1
        );
        assert!(board.missiles.is_empty());
        assert_eq!(board.invasion.as_ref().unwrap().food_delay, 0);
        board.invasion.as_mut().unwrap().actors.clear();
        let next_id = board.id();
        let mut next = ClassicMissile::launch(next_id, board.fish[1].id, 100, 280, 0);
        next.immunity_ticks = 0;
        board.missiles.push(next);
        let end = board.apply(Action::Click { x: 140.0, y: 320.0 });
        assert!(end.iter().any(|event| matches!(
            event,
            Event::Invasion {
                event: InvasionEvent::BattleEnded,
                ..
            }
        )));
        assert_eq!(
            end.iter()
                .filter(|event| matches!(
                    event,
                    Event::Invasion {
                        event: InvasionEvent::LaserFired { .. },
                        ..
                    }
                ))
                .count(),
            1
        );
        assert_eq!(board.invasion.as_ref().unwrap().food_delay, 36);
        assert!(board.food.is_empty());
    }

    #[test]
    fn missile_only_missed_combat_click_still_creates_one_board_laser() {
        let mut board = AdventureState::new_tank2_fourth_stage(0x2406, &[]).unwrap();
        let id = board.id();
        board
            .missiles
            .push(ClassicMissile::launch(id, board.fish[0].id, 100, 280, 0));
        let events = board.apply(Action::Click { x: 400.0, y: 200.0 });
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    Event::Invasion {
                        event: InvasionEvent::LaserFired { .. },
                        ..
                    }
                ))
                .count(),
            1
        );
        assert_eq!(board.missiles.len(), 1);
        assert_eq!(
            board.invasion.as_ref().unwrap().last_laser,
            Some((400, 200))
        );
    }

    #[test]
    fn selected_rufus_leaves_quarter_kill_registered_until_next_alien_update() {
        let mut board = AdventureState::new_tank2_fourth_stage(0x2407, &[PetKind::Rufus]).unwrap();
        board.validate().unwrap();
        let pet_x = board.rufus.first().unwrap().widget_x;
        let alien_id = board.id();
        let mut alien = crate::alien::WeakSylvester::spawn_kind(
            SylvesterKind::Destructor,
            alien_id,
            pet_x - 60,
            120,
            1,
            1,
        );
        alien.spawn_ticks = 0;
        alien.health = 0.25;
        board.invasion.as_mut().unwrap().actors.push(alien);
        board.invasion.as_mut().unwrap().battle_active = true;
        let mut events = Vec::new();
        board.update_rufus(&mut events);
        assert_eq!(
            board
                .invasion
                .as_ref()
                .unwrap()
                .actors
                .first()
                .unwrap()
                .health,
            0.0
        );
        assert!(board.invasion.as_ref().unwrap().has_live_alien());
        board.validate().unwrap();
        board.update_invasion_objects(&mut events);
        assert!(!board.invasion.as_ref().unwrap().has_live_alien());
        assert_eq!(events.iter().filter(|event| matches!(event, Event::AlienDiamondDropped { alien_id: id, .. } if *id == alien_id)).count(), 1);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    Event::Invasion {
                        event: InvasionEvent::BattleEnded,
                        ..
                    }
                ))
                .count(),
            1
        );
    }

    #[test]
    fn pair_commits_first_bite_before_second_actor_sees_prey() {
        let mut board = AdventureState::new_tank2_fifth_stage(0x2501, &[]).unwrap();
        board.fish.truncate(1);
        board.fish[0].x = 140.0;
        board.fish[0].y = 160.0;
        board.fish[0].cannot_be_eaten_ticks = 0;
        let prey_id = board.fish[0].id;
        let wave = board.invasion.as_mut().unwrap();
        wave.countdown = 3000;
        wave.plan = WavePlan::CyclingTank2Finale {
            next: EncounterKind::Single(SylvesterKind::Gus),
        };
        for (id, kind) in [(90, SylvesterKind::Weak), (91, SylvesterKind::Balrog)] {
            let mut actor = crate::alien::WeakSylvester::spawn_kind(kind, id, 100, 120, 1, 1);
            actor.spawn_ticks = 0;
            actor.chase_ticks = 0;
            wave.actors.push(actor);
        }
        wave.battle_active = true;
        let mut events = Vec::new();
        board.update_invasion_objects(&mut events);
        assert_eq!(events.iter().filter(|event| matches!(event, Event::Invasion { event: InvasionEvent::PreyEaten { prey_id: id, .. }, .. } if *id == prey_id)).count(), 1);
        assert!(board.fish.is_empty());
        assert!(board.invasion.as_ref().unwrap().has_live_alien());
    }

    #[test]
    fn meryl_song_uses_fish_before_pet_boundary_and_note_cannot_mask_coin() {
        let mut board = AdventureState::new_tank2_fifth_stage(0x2502, &[PetKind::Meryl]).unwrap();
        board.fish[0].size = FishSize::Medium;
        board.fish[0].coin_threshold = 100;
        board.fish[0].coin_timer = 34;
        board.fish_pets[0].coin_timer = 1299;
        let first = board.update_objects();
        assert!(
            !first
                .iter()
                .any(|event| matches!(event, Event::CoinDropped { .. }))
        );
        assert!(
            first
                .iter()
                .any(|event| matches!(event, Event::MerylNoteDropped { .. }))
        );
        assert!(board.fish_pets[0].song_active());
        assert_eq!(board.notes[0].age_ticks, 1);
        let second = board.update_objects();
        assert!(second.iter().any(|event| matches!(
            event,
            Event::CoinDropped {
                kind: CoinKind::Silver,
                ..
            }
        )));
        board.fish_pets[0].coin_timer = 1399;
        board.update_objects();
        assert!(!board.fish_pets[0].song_active());
        let note_id = board.notes[0].id;
        board.notes[0].x = 100;
        board.notes[0].y = 100;
        let overlapping_coin_id = board.id();
        board.coins.push(Coin {
            id: overlapping_coin_id,
            x: 100.0,
            y: 100.0,
            kind: CoinKind::Silver,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            collecting: false,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: false,
        });
        let coin_id = board.coins.last().unwrap().id;
        let claim = board.apply(Action::Click { x: 110.0, y: 110.0 });
        assert!(claim.iter().any(|event| matches!(event, Event::CoinCollectionStarted { coin_id: id, .. } if *id == coin_id)));
        assert_eq!(board.notes[0].id, note_id);
        board.notes[0].age_ticks = 99;
        board.notes[0].y = 50;
        board.update_notes(&mut Vec::new());
        assert_eq!((board.notes[0].age_ticks, board.notes[0].y), (100, 49));
        board.update_notes(&mut Vec::new());
        assert!(board.notes.is_empty());
    }

    #[test]
    fn missile_impact_makes_guppy_corpse_and_detaches_before_battle_end() {
        let mut board = AdventureState::new_tank2_fourth_stage(0x2403, &[]).unwrap();
        let target_id = board.fish[0].id;
        board.fish[0].x = 125.0;
        board.fish[0].y = 125.0;
        let id = board.id();
        board
            .missiles
            .push(ClassicMissile::launch(id, target_id, 100, 100, 0));
        board.invasion.as_mut().unwrap().battle_active = true;
        let mut events = Vec::new();
        board.update_missiles(&mut events);
        assert!(!board.fish[0].alive);
        assert_eq!(
            board
                .dead_fish
                .iter()
                .filter(|corpse| corpse.id == target_id)
                .count(),
            1
        );
        assert!(board.missiles.is_empty());
        let died = events
            .iter()
            .position(|event| matches!(event, Event::FishDied { .. }))
            .unwrap();
        let removed = events
            .iter()
            .position(|event| matches!(event, Event::MissileRemoved { .. }))
            .unwrap();
        let impact = events
            .iter()
            .position(|event| matches!(event, Event::MissileImpacted { .. }))
            .unwrap();
        let battle = events
            .iter()
            .position(|event| {
                matches!(
                    event,
                    Event::Invasion {
                        event: InvasionEvent::BattleEnded,
                        ..
                    }
                )
            })
            .unwrap();
        assert!(died < removed && removed < impact && impact < battle);
    }

    #[test]
    fn starvation_detaches_assigned_missile_and_reload_preserves_missile_only_state() {
        let mut board = AdventureState::new_tank2_fourth_stage(0x2404, &[]).unwrap();
        let target_id = board.fish[0].id;
        let id = board.id();
        board
            .missiles
            .push(ClassicMissile::launch(id, target_id, 100, 100, 0));
        board.invasion.as_mut().unwrap().battle_active = true;
        board.validate().unwrap();
        let resumed: AdventureState =
            serde_json::from_str(&serde_json::to_string(&board).unwrap()).unwrap();
        resumed.validate().unwrap();
        assert_eq!(resumed.missiles[0].target_id, Some(target_id));
        board.fish[0].hunger = 0;
        let mut events = Vec::new();
        board.update_fish(&mut events);
        assert!(board.missiles.is_empty());
        assert!(events.iter().any(
            |event| matches!(event, Event::MissileRemoved { missile_id, .. } if *missile_id == id)
        ));
    }

    #[test]
    fn current_missile_save_rejects_dangling_and_duplicate_target_assignments() {
        let mut board = AdventureState::new_tank2_fourth_stage(0x2405, &[]).unwrap();
        let target = board.fish[0].id;
        let first = board.id();
        board
            .missiles
            .push(ClassicMissile::launch(first, target, 100, 100, 0));
        board.invasion.as_mut().unwrap().battle_active = true;
        board.validate().unwrap();
        let second = board.id();
        board
            .missiles
            .push(ClassicMissile::launch(second, target, 120, 100, 0));
        assert!(board.validate().is_err());
        board.missiles.pop();
        board.fish[0].alive = false;
        assert!(board.validate().is_err());
    }

    #[test]
    fn ulysses_reserves_two_distinct_farthest_fish_before_reload() {
        let mut board = AdventureState::new_tank3_fourth_stage(0x3401, &[PetKind::Gumbo]).unwrap();
        board.fish[0].x = 100.0;
        board.fish[0].y = 100.0;
        board.fish[1].x = 400.0;
        board.fish[1].y = 300.0;
        for fish in &mut board.fish {
            fish.cannot_be_eaten_ticks = 0;
        }
        // Ulysses occupies the fixed Y=280 lane: center (260,360).
        // Fish centers (140,140) and (440,340) are 62_800 and 32_800
        // squared units away, respectively.
        let first_expected = board.fish[0].id;
        let second_expected = board.fish[1].id;
        let actor_id = board.id();
        let mut actor = crate::alien::WeakSylvester::spawn_kind(
            SylvesterKind::Ulysses,
            actor_id,
            180,
            200,
            1,
            1,
        );
        actor.spawn_ticks = 0;
        actor.launch_ticks = 75;
        let wave = board.invasion.as_mut().unwrap();
        wave.countdown = 3000;
        wave.battle_active = true;
        wave.actors.push(actor);
        let mut events = Vec::new();
        board.update_invasion_objects(&mut events);
        assert_eq!(
            board
                .missiles
                .iter()
                .map(|missile| missile.target_id)
                .collect::<Vec<_>>(),
            [Some(first_expected), Some(second_expected)]
        );
        assert!(
            board
                .missiles
                .iter()
                .all(|missile| missile.kind == MissileKind::EnergyBall && !missile.reflected)
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, Event::EnergyBallLaunched { first: true, .. }))
                .count(),
            1
        );
        board.validate().unwrap();
    }

    #[test]
    fn immune_energy_click_persists_shot2_and_reflection_keeps_last_threat() {
        let mut board = AdventureState::new_tank3_fourth_stage(0x3402, &[]).unwrap();
        let target_id = board.fish[0].id;
        let missile_id = board.id();
        let mut ball = ClassicMissile::launch_energy(missile_id, target_id, 100, 100, 0);
        ball.immunity_ticks = 1;
        board.missiles.push(ball);
        board.invasion.as_mut().unwrap().battle_active = true;
        let mut events = Vec::new();
        assert!(board.shoot_first_missile(140, 140, &mut events));
        assert!(!board.missiles[0].reflected && board.bomb_shots.len() == 1);
        board.missiles[0].immunity_ticks = 0;
        assert!(board.shoot_first_missile(140, 140, &mut events));
        assert!(board.missiles[0].reflected && board.missiles[0].target_id == Some(target_id));
        assert_eq!(board.bomb_shots.len(), 2);
        assert!(!events.iter().any(|event| matches!(
            event,
            Event::Invasion {
                event: InvasionEvent::BattleEnded,
                ..
            }
        )));
        board.validate().unwrap();
    }

    #[test]
    fn accepted_energy_effect_has_source_finite_clock_without_draw_projection() {
        let mut shot = BombShot {
            shot_type: 2,
            x: 100,
            y: 100,
            age_ticks: 0,
            frame: 0,
            delay_ticks: 0,
            alpha: 249,
        };
        for _ in 0..29 {
            assert!(!shot.tick());
            assert_eq!(shot.sprite_frame(), None);
        }
        assert_eq!(shot.age_ticks, 29); // high alpha does not double Shot2 time
        shot.validate().unwrap();
        assert!(shot.tick());
    }

    #[test]
    fn retained_prey_death_detaches_reflected_ball_but_current_typed_scan_continues() {
        let mut board = AdventureState::new_tank3_fourth_stage(0x3403, &[]).unwrap();
        board.fish[0].x = 100.0;
        board.fish[0].y = 100.0;
        board.fish[1].x = 500.0;
        board.fish[1].y = 300.0;
        let target_id = board.fish[0].id;
        let gekko_id = board.id();
        let mut gekko = GekkoState::spawn_bought(gekko_id, &mut |_| 0);
        gekko.x = 100.0;
        gekko.y = 100.0;
        gekko.widget_x = 100;
        gekko.widget_y = 100;
        board.gekkos.push(gekko);
        let missile_id = board.id();
        let mut ball = ClassicMissile::launch_energy(missile_id, target_id, 100, 100, 0);
        ball.immunity_ticks = 0;
        ball.try_shot(140, 140);
        board.missiles.push(ball);
        board.invasion.as_mut().unwrap().battle_active = true;
        let mut events = Vec::new();
        board.update_missiles(&mut events);
        assert!(!board.fish[0].alive);
        assert!(board.gekkos.is_empty());
        assert!(board.missiles.is_empty());
        assert_eq!(events.iter().filter(|event| matches!(event, Event::EnergyBallRemoved { missile_id: id, .. } if *id == missile_id)).count(), 1);
        assert!(events.iter().any(|event| matches!(
            event,
            Event::Invasion {
                event: InvasionEvent::BattleEnded,
                ..
            }
        )));
    }

    #[test]
    fn reflected_ball_reacquires_identity_after_earlier_reservation_detaches() {
        let mut board = AdventureState::new_tank3_fourth_stage(0x3404, &[]).unwrap();
        board.fish[0].x = 100.0;
        board.fish[0].y = 100.0;
        board.fish[1].x = 500.0;
        board.fish[1].y = 300.0;
        let mut third = board.make_fish(450.0, 300.0, false, false);
        third.cannot_be_eaten_ticks = 0;
        let third_id = third.id;
        board.fish.push(third);
        let first_target = board.fish[0].id;
        let second_target = board.fish[1].id;
        let first_id = board.id();
        let second_id = board.id();
        let third_ball_id = board.id();
        board.missiles.push(ClassicMissile::launch_energy(
            first_id,
            first_target,
            400,
            100,
            0,
        ));
        let mut reflected = ClassicMissile::launch_energy(second_id, second_target, 100, 100, 0);
        reflected.immunity_ticks = 0;
        assert_eq!(reflected.try_shot(140, 140), MissileShot::Redirected);
        board.missiles.push(reflected);
        board.missiles.push(ClassicMissile::launch_energy(
            third_ball_id,
            third_id,
            400,
            100,
            0,
        ));
        board.invasion.as_mut().unwrap().battle_active = true;
        let mut events = Vec::new();
        board.update_missiles(&mut events);
        assert!(!board.fish[0].alive);
        assert!(board.missiles.iter().all(|ball| ball.id != first_id));
        assert_eq!(
            board
                .missiles
                .iter()
                .find(|ball| ball.id == second_id)
                .unwrap()
                .x,
            102.5
        );
        assert_eq!(
            board
                .missiles
                .iter()
                .find(|ball| ball.id == third_ball_id)
                .unwrap()
                .x,
            400.5
        );
        assert_eq!(events.iter().filter(|event| matches!(event, Event::EnergyBallRemoved { missile_id, .. } if *missile_id == first_id)).count(), 1);
    }

    // Prepared impact snapshots use independently recovered PB05 collision
    // rectangles; they are unit evidence, not an earned/native playtest.
    fn tank5_reflected_impact_board() -> AdventureState {
        let mut board = AdventureState::new_tank5_1(0x5105, 0);
        let wave = board.invasion.as_mut().unwrap();
        wave.countdown = 1;
        wave.warning = Some(crate::invasion::WarningCoords {
            first_x: 450,
            first_y: 250,
            second_x: 480,
            second_y: 250,
        });
        board.begin_tick();
        board
    }

    fn tank5_reflected_ball(board: &mut AdventureState, target_id: u64, x: i32, y: i32) -> u64 {
        let id = board.id();
        let mut ball = ClassicMissile::launch_energy(id, target_id, x, y, 0);
        ball.reflected = true;
        ball.immunity_ticks = 0;
        board.missiles.push(ball);
        id
    }

    #[test]
    fn tank5_reflected_ball_removes_first_pet_of_each_category_after_boss_miss() {
        let mut board = tank5_reflected_impact_board();
        for pet in &mut board.fish_pets {
            pet.widget_x = 400;
            pet.widget_y = 100;
        }
        for pet in &mut board.fish_pets[..2] {
            pet.widget_x = 100;
            pet.widget_y = 100;
        }
        let first = board.fish_pets[0].id;
        let second = board.fish_pets[1].id;
        let clyde = board.clyde.first_mut().unwrap();
        clyde.widget_x = 100;
        clyde.widget_y = 100;
        let other_id = clyde.id;
        // Removing the first pet detaches its assigned ball synchronously,
        // but the captured position must still reach the OtherTypePet scan.
        let ball = tank5_reflected_ball(&mut board, first, 100, 100);
        let mut events = Vec::new();
        board.reflected_energy_contact(ball, &mut events);
        assert!(!board.fish_pets.iter().any(|pet| pet.id == first));
        assert!(board.fish_pets.iter().any(|pet| pet.id == second));
        assert!(board.clyde.is_empty());
        assert!(board.missiles.is_empty());
        let removed = events
            .iter()
            .filter_map(|event| match event {
                Event::PetRemoved { pet_id, .. } => Some(*pet_id),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(removed, vec![first, other_id]);
        assert_eq!(board.pets.as_slice(), TANK5_PETS.as_slice());
    }

    #[test]
    fn tank5_mini_in_separate_list_does_not_shield_boss_from_reflected_ball() {
        let mut board = tank5_reflected_impact_board();
        let mini_id = board.id();
        let child = crate::alien::WeakSylvester::spawn_mini(mini_id, 100, 100, 1, 1, 1);
        let finale = board.invasion.as_mut().unwrap().finale.as_mut().unwrap();
        finale.children.push(child);
        let boss = finale.boss.as_mut().unwrap();
        boss.widget_x = 100;
        boss.widget_y = 100;
        let old_hp = boss.health;
        let target_id = board.niko.first().unwrap().owner_id;
        let ball = tank5_reflected_ball(&mut board, target_id, 100, 100);
        let mut events = Vec::new();
        board.reflected_energy_contact(ball, &mut events);
        let finale = board.invasion.as_ref().unwrap().finale.as_ref().unwrap();
        assert_eq!(finale.children.len(), 1);
        assert_eq!(finale.children[0].health, 1.0);
        assert_eq!(finale.boss.as_ref().unwrap().health, old_hp - 30.0);
        assert!(board.missiles.is_empty());
        assert!(board.coins.is_empty());
        assert!(!events.iter().any(|event| matches!(event,
            Event::Invasion { event: InvasionEvent::MiniDefeated { id }, .. } if *id == mini_id
        )));
    }

    #[test]
    fn tank5_reflected_boss_strict_edge_misses_but_lethal_hit_cleans_inline() {
        for (ball_x, lethal) in [(470, false), (471, true)] {
            let mut board = tank5_reflected_impact_board();
            let boss = board
                .invasion
                .as_mut()
                .unwrap()
                .finale
                .as_mut()
                .unwrap()
                .boss
                .as_mut()
                .unwrap();
            boss.widget_x = 480;
            boss.widget_y = 250;
            boss.health = 30.0;
            let target_id = board.niko.first().unwrap().owner_id;
            let ball = tank5_reflected_ball(&mut board, target_id, ball_x, 250);
            board.reflected_energy_contact(ball, &mut Vec::new());
            let finale = board.invasion.as_ref().unwrap().finale.as_ref().unwrap();
            assert_eq!(finale.boss.is_none(), lethal);
            assert_eq!(finale.boss_defeated, lethal);
            assert_eq!(board.missiles.is_empty(), lethal);
            assert!(board.coins.is_empty());
            assert!(!board.egg_unlocked);
            if lethal {
                board.begin_tick();
                assert!(board.egg_unlocked);
            }
        }
    }

    #[test]
    fn tank5_reflected_bilaterus_uses_twenty_five_pixel_center_and_inline_head_death() {
        let mut board = tank5_reflected_impact_board();
        let group_id = board.id();
        let mut group = crate::bilaterus::BilaterusState::spawn(group_id, 100, 100, &mut || 1);
        group.active_mut().widget_x = 100;
        group.active_mut().widget_y = 100;
        group.active_mut().health = 30.0;
        board.invasion.as_mut().unwrap().bilaterus.push(group);
        let target_id = board.niko.first().unwrap().owner_id;
        // +25=160 is inside; the ordinary +40=175 would wrongly miss.
        let ball = tank5_reflected_ball(&mut board, target_id, 135, 135);
        board.reflected_energy_contact(ball, &mut Vec::new());
        let wave = board.invasion.as_ref().unwrap();
        assert!(wave.bilaterus[0].first_head_lost);
        assert_eq!(wave.fragments.len(), 1);
        assert!(board.missiles.is_empty());
        assert!(board.coins.is_empty());
    }

    #[test]
    fn reflected_energy_contact_uses_integer_widget_edges_for_alien_and_guppy() {
        let alien_case = |ball_x: f64, ball_y: f64| {
            let mut board = AdventureState::new_tank3_fourth_stage(0x3410, &[]).unwrap();
            for fish in &mut board.fish {
                fish.x = 500.0;
                fish.y = 300.0;
            }
            let alien_id = board.id();
            let mut alien = crate::alien::WeakSylvester::spawn_kind(
                SylvesterKind::Ulysses,
                alien_id,
                100,
                100,
                1,
                1,
            );
            alien.x = 100.9; // published widget remains integer 100
            alien.y = 280.9; // published widget remains integer 280
            board.invasion.as_mut().unwrap().actors.push(alien);
            board.invasion.as_mut().unwrap().battle_active = true;
            let id = board.id();
            let mut ball = ClassicMissile::launch_energy(
                id,
                board.fish[0].id,
                ball_x as i32,
                ball_y as i32,
                0,
            );
            ball.x = ball_x;
            ball.y = ball_y;
            ball.reflected = true;
            ball.immunity_ticks = 0;
            ball.vx = 1.0;
            ball.vy = 1.0;
            board.missiles.push(ball);
            board.reflected_energy_contact(id, &mut Vec::new());
            (
                board.invasion.as_ref().unwrap().actors[0].health,
                board.missiles.len(),
            )
        };
        assert_eq!(alien_case(90.5, 300.0), (190.0, 0)); // X center130.5 > widget100+30
        assert_eq!(alien_case(200.0, 300.0), (220.0, 1)); // X center240 == widget100+140
        assert_eq!(alien_case(90.5, 250.5), (190.0, 0)); // Y center290.5 > widget280+10
        assert_eq!(alien_case(90.5, 390.0), (220.0, 1)); // Y center430 == widget280+150

        let guppy_case = |ball_x: f64, ball_y: f64| {
            let mut board = AdventureState::new_tank3_fourth_stage(0x3411, &[]).unwrap();
            board.fish[0].x = 100.9;
            board.fish[0].y = 100.0;
            board.fish[1].x = 500.0;
            board.fish[1].y = 300.0;
            let target_id = board.fish[1].id;
            let id = board.id();
            let mut ball =
                ClassicMissile::launch_energy(id, target_id, ball_x as i32, ball_y as i32, 0);
            ball.x = ball_x;
            ball.y = ball_y;
            ball.reflected = true;
            ball.immunity_ticks = 0;
            ball.vx = 1.0;
            ball.vy = 1.0;
            board.missiles.push(ball);
            board.invasion.as_mut().unwrap().battle_active = true;
            board.reflected_energy_contact(id, &mut Vec::new());
            board.fish[0].alive
        };
        assert!(!guppy_case(70.5, 90.0)); // X center110.5 > widget100+10
        assert!(guppy_case(130.0, 90.0)); // X center170 == widget100+70
        assert!(!guppy_case(70.5, 70.5)); // Y center110.5 > widget100+10
        assert!(guppy_case(70.5, 130.0)); // Y center170 == widget100+70
    }

    #[test]
    fn tank3_finale_opening_bit_precedes_blip_and_starter_constructor_draws() {
        for (seed, expected_first) in [
            (0x3501, SylvesterKind::Ulysses),
            (0x3502, SylvesterKind::Psychosquid),
        ] {
            let mut draws = AdventureState::initial_rng(seed);
            let first_bit = AdventureState::advance_rng(&mut draws) % 2;
            let pet_x = (AdventureState::advance_rng(&mut draws) % 265) as i32 + 105;
            let pet_y = (AdventureState::advance_rng(&mut draws) % 520) as i32 + 20;
            let board = AdventureState::new_tank3_fifth_stage(seed, &[PetKind::Blip]).unwrap();
            assert_eq!(first_bit == 0, expected_first == SylvesterKind::Ulysses);
            assert_eq!(
                board.invasion.as_ref().unwrap().plan.expected(),
                EncounterKind::Single(expected_first)
            );
            assert_eq!(
                (board.fish_pets[0].widget_x, board.fish_pets[0].widget_y),
                (pet_x, pet_y)
            );
            assert_eq!(
                (board.balance, board.egg_price, board.fish.len()),
                (200, 15_000, 2)
            );
            board.validate().unwrap();
        }
    }

    #[test]
    fn live_blip_reveals_only_mapped_buttons_after_update_200_once_without_money() {
        let mut board = AdventureState::new_tank3_fifth_stage(0x3502, &[PetKind::Blip]).unwrap();
        board.tick = 199;
        assert!(board.apply(Action::BuyGuppy).iter().any(|event| matches!(
            event,
            Event::Rejected {
                reason: Rejection::Locked,
                ..
            }
        )));
        assert!(
            !board
                .tick()
                .iter()
                .any(|event| matches!(event, Event::BlipShopRevealed { .. }))
        );
        assert_eq!(board.tick, 200);
        assert!(!board.guppy_unlocked);
        assert!(board.apply(Action::BuyGuppy).iter().any(|event| matches!(
            event,
            Event::Rejected {
                reason: Rejection::Locked,
                ..
            }
        )));
        assert_eq!(board.tick, 200); // Inputs alone cannot advance the reveal gate.
        let events = board.tick();
        assert_eq!(board.tick, 201);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, Event::BlipShopRevealed { .. }))
                .count(),
            1
        );
        assert!(
            board.guppy_unlocked
                && board.upgrades.quality_unlocked
                && board.upgrades.quantity_unlocked
        );
        assert!(
            board.grubber_unlocked
                && board.gekko_unlocked
                && board.weapon_unlocked
                && board.egg_unlocked
        );
        assert!(!board.oscar_unlocked && !board.starcatcher_unlocked && !board.potion_unlocked);
        assert_eq!(board.balance, 200);
        assert!(
            !board
                .tick()
                .iter()
                .any(|event| matches!(event, Event::BlipShopRevealed { .. }))
        );
        board.validate().unwrap();

        let mut unselected = AdventureState::new_tank3_fifth_stage(0x3502, &[]).unwrap();
        unselected.tick = 200;
        assert!(
            !unselected
                .tick()
                .iter()
                .any(|event| matches!(event, Event::BlipShopRevealed { .. }))
        );
        assert!(!unselected.guppy_unlocked);
    }

    #[test]
    fn blip_information_boundaries_and_current_kind_are_independent_of_future_wave() {
        let mut board = AdventureState::new_tank3_fifth_stage(0x3503, &[PetKind::Blip]).unwrap();
        assert!(board.has_live_blip());
        board.fish[0].hunger = 499;
        board.fish[0].food_ate = 2;
        assert!(board.blip_hunger_icon_visible(&board.fish[0]));
        board.fish[0].hunger = 500;
        assert!(!board.blip_hunger_icon_visible(&board.fish[0]));
        board.fish[0].hunger = 499;
        board.fish[0].food_ate = 3;
        assert!(board.blip_hunger_icon_visible(&board.fish[0]));
        board.tick = 224;
        let wave = board.invasion.as_mut().unwrap();
        wave.warning = Some(crate::invasion::WarningCoords {
            first_x: 100,
            first_y: 130,
            second_x: 200,
            second_y: 250,
        });
        wave.countdown = 226;
        assert_eq!(board.blip_crosshair(), None);
        assert_eq!(
            board
                .begin_tick()
                .iter()
                .filter(|event| matches!(event, Event::BlipSonar { .. }))
                .count(),
            1
        );
        assert_eq!(
            board.blip_crosshair(),
            Some((140, 320, ((board.tick / 4) % 5) as u8))
        );

        let alien_id = board.id();
        let actor = crate::alien::WeakSylvester::spawn_kind(
            SylvesterKind::Ulysses,
            alien_id,
            100,
            120,
            1,
            1,
        );
        let target = board.fish[0].id;
        let missile_id = board.id();
        board.missiles.push(ClassicMissile::launch_energy(
            missile_id, target, 100, 280, 0,
        ));
        let wave = board.invasion.as_mut().unwrap();
        wave.actors.push(actor);
        wave.plan = WavePlan::CyclingTank3Finale {
            next: SylvesterKind::Psychosquid,
        };
        wave.countdown = 3000;
        wave.warning = None;
        wave.battle_active = true;
        board.validate().unwrap();
        board.missiles[0].kind = MissileKind::Classic;
        assert!(board.validate().is_err());
        board.missiles[0].kind = MissileKind::EnergyBall;
        board.invasion.as_mut().unwrap().actors.clear();
        board.validate().unwrap(); // Energy-ball-only tail with a different future expectation.
        board.missiles[0].kind = MissileKind::Classic;
        assert!(board.validate().is_err()); // Classic remains impossible without an actor.
        board.missiles[0].kind = MissileKind::EnergyBall;
        board
            .invasion
            .as_mut()
            .unwrap()
            .actors
            .push(crate::alien::WeakSylvester::spawn_kind(
                SylvesterKind::Psychosquid,
                alien_id,
                100,
                120,
                1,
                1,
            ));
        assert!(board.validate().is_err()); // Scheduler cannot spawn Psychosquid before ball removal.
        board.missiles[0].kind = MissileKind::Classic;
        assert!(board.validate().is_err());
        board.missiles.clear();
        board.validate().unwrap(); // Live Psychosquid alone remains a valid later encounter.
    }

    #[test]
    fn tank4_starter_and_buy_slot_do_not_create_a_guppy_shop() {
        let mut board = AdventureState::new_tank4_first_stage(0x4101, &[]).unwrap();
        assert!(board.has_live_fish());
        assert_eq!(board.breeders.len(), 1);
        assert_eq!(board.breeders[0].food_points, 2);
        let guppy = board.make_fish(200.0, 200.0, false, false);
        board.fish.push(guppy);
        board.fish.last_mut().unwrap().size = FishSize::Medium;
        board.guppy_unlocked = false;
        assert!(board.apply(Action::BuyGuppy).iter().any(|event| matches!(
            event,
            Event::Rejected {
                reason: Rejection::Locked,
                ..
            }
        )));
        assert_eq!(board.fish.len(), 1);
        board.validate().unwrap();
    }

    #[test]
    fn tank4_breeder_starvation_tail_can_commit_due_birth_after_corpse() {
        let mut board = AdventureState::new_tank4_first_stage(0x4102, &[]).unwrap();
        let parent_id = board.breeders[0].id;
        board.breeders[0].size = BreederSize::Medium;
        board.breeders[0].hunger = 1;
        board.breeders[0].birth_threshold = 900;
        board.breeders[0].birth_clock = 899;
        board.update_breeders(&mut Vec::new());
        assert!(board.breeders.is_empty());
        assert_eq!(board.dead_breeders.len(), 1);
        assert_eq!(board.dead_breeders[0].id, parent_id);
        assert_eq!(board.fish.len(), 1);
        assert!(board.fish[0].alive);
        board.validate().unwrap();
    }

    #[test]
    fn tank4_rhubarb_pushes_each_overlapping_category_once_in_list_order() {
        let mut board = AdventureState::new_tank4_first_stage(0x4103, &[PetKind::Rhubarb]).unwrap();
        let mut fish = board.make_fish(110.0, 300.0, false, false);
        fish.bought_timer = 0;
        let fish_id = fish.id;
        board.fish.push(fish);
        let breeder_id = board.breeders[0].id;
        board.breeders[0].x = 110.0;
        board.breeders[0].y = 300.0;
        board.breeders[0].widget_x = 110;
        board.breeders[0].widget_y = 300;
        let pet = board.rhubarb.first_mut().unwrap();
        pet.x = 100.0;
        pet.y = 355.0;
        pet.widget_x = 100;
        pet.widget_y = 355;
        pet.specialty_ticks = 5;
        let mut events = Vec::new();
        board.update_rhubarb(&mut events);
        let pushed = events
            .iter()
            .filter_map(|event| match event {
                Event::RhubarbPushed { target_id, .. } => Some(*target_id),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(pushed, vec![fish_id, breeder_id]);
        assert_eq!(board.fish[0].bought_timer, 50);
        assert_eq!(board.breeders[0].bought_timer, 50);
        assert_eq!(board.rhubarb.first().unwrap().specialty_ticks, 4);
        board.validate().unwrap();
    }

    // These controlled fixtures exercise independently checked W1 rules;
    // they do not supply native earning or original-game observation evidence.
    fn a31_review_balrog_board(pets: &[PetKind]) -> AdventureState {
        let mut board = AdventureState::new_tank4_first_stage(0x4104, pets).unwrap();
        let actor_id = board.id();
        let wave = board.invasion.as_mut().unwrap();
        wave.countdown = 3000;
        wave.battle_active = true;
        wave.actors.push(crate::alien::WeakSylvester::spawn_kind(
            SylvesterKind::Balrog,
            actor_id,
            450,
            100,
            0,
            0,
        ));
        let breeder = &mut board.breeders[0];
        breeder.x = 200.0;
        breeder.y = 200.0;
        breeder.widget_x = 200;
        breeder.widget_y = 200;
        board
    }

    #[test]
    fn a31_review_non_gus_food_contact_preserves_frozen_birth_clock() {
        // Breeder.cpp739-754: a non-Gus first registered alien permits
        // feeding while hunger decrement and production remain frozen.
        let mut board = a31_review_balrog_board(&[]);
        let breeder = &mut board.breeders[0];
        breeder.size = BreederSize::Medium;
        breeder.hunger = 300;
        breeder.food_points = 0;
        breeder.food_needed_to_grow = 7;
        breeder.birth_clock = 900;
        breeder.birth_threshold = 1000;
        board.breeder_unlocked = true;
        board.drop_food(230.0, 215.0, 0, &mut Vec::new());
        let food_id = board.food[0].id;
        let mut events = Vec::new();
        board.update_breeders(&mut events);
        assert!(events.iter().any(|event| matches!(
            event, Event::BreederAteFood { food_id: eaten, .. } if *eaten == food_id
        )));
        assert!(board.food.is_empty());
        assert_eq!(board.breeders[0].hunger, 800);
        assert_eq!(board.breeders[0].birth_clock, 900);
    }

    #[test]
    fn a31_review_starvation_keeps_pre_steering_corpse_velocity() {
        // Breeder.cpp745-767: Die snapshots VX before Update continues
        // into wander, production and the motion/animation tail.
        let mut board = AdventureState::new_tank4_first_stage(0x4105, &[]).unwrap();
        let breeder = &mut board.breeders[0];
        breeder.x = 200.0;
        breeder.y = 200.0;
        breeder.widget_x = 200;
        breeder.widget_y = 200;
        breeder.size = BreederSize::Medium;
        breeder.hunger = 1;
        breeder.vx = -0.1;
        breeder.previous_vx = -0.1;
        breeder.vy = 0.0;
        breeder.movement_state = 0;
        breeder.steering_timer = 40;
        breeder.birth_clock = breeder.birth_threshold - 1;
        board.breeder_unlocked = true;
        board.update_breeders(&mut Vec::new());
        assert!(board.breeders.is_empty());
        assert_eq!(board.fish.len(), 1);
        let corpse = &board.dead_breeders[0];
        assert!(!corpse.facing_right);
        let pose = serde_json::to_value(corpse).unwrap();
        assert_eq!(pose["vx"], -0.1);
        assert_eq!(pose["vy"], -2.0);
    }

    #[test]
    fn a31_review_gumbo_steering_counter_is_a_reachable_save_state() {
        // Breeder.cpp63-70/333: Gumbo's invasion steering does not reset
        // the shared special timer before its unconditional increment.
        let mut board = a31_review_balrog_board(&[PetKind::Gumbo]);
        board.breeders[0].hunger = 550;
        board.breeders[0].steering_timer = 40;
        board.update_breeders(&mut Vec::new());
        board.update_breeders(&mut Vec::new());
        assert_eq!(board.breeders[0].steering_timer, 42);
        board.validate().unwrap();
        for _ in 0..300 {
            board.update_breeders(&mut Vec::new());
        }
        assert_eq!(
            serde_json::to_value(&board.breeders[0]).unwrap()["steering_timer"],
            342
        );
        board.validate().unwrap();
        let saved = serde_json::to_vec(&board).unwrap();
        let restored: AdventureState = serde_json::from_slice(&saved).unwrap();
        restored.validate().unwrap();
        assert_eq!(
            serde_json::to_value(restored).unwrap(),
            serde_json::to_value(board).unwrap()
        );
    }

    #[test]
    fn a31_review_invasion_can_preserve_large_birth_clock_above_new_threshold() {
        // Breeder.cpp531-601: ordinary Medium->Large growth preserves
        // its initial900 or accumulated1398 when invasion freezes production.
        for clock in [900, 1398] {
            let mut board = a31_review_balrog_board(&[]);
            board.breeder_unlocked = true;
            let breeder = &mut board.breeders[0];
            breeder.size = BreederSize::Medium;
            breeder.hunger = 300;
            breeder.food_points = 6;
            breeder.food_needed_to_grow = 7;
            breeder.birth_clock = clock;
            breeder.birth_threshold = 1399;
            board.validate().unwrap();
            let food = [BreederFoodView {
                id: 999,
                widget_x: 220,
                widget_y: 205,
                quality: 0,
                eligible: true,
            }];
            let breeder = &mut board.breeders[0];
            let update = breeder.begin_tick(&food, true, true, None, None, &mut |_| 0);
            assert_eq!(update.meals[0].size, BreederSize::Large);
            assert_eq!(breeder.birth_clock, clock);
            assert_eq!(breeder.birth_threshold, 500);
            assert!(update.born_at.is_none());
            board.validate().unwrap();
            let saved = serde_json::to_vec(&board).unwrap();
            let restored: AdventureState = serde_json::from_slice(&saved).unwrap();
            restored.validate().unwrap();
            assert_eq!(restored.breeders[0].birth_clock, clock);
            assert_eq!(restored.breeders[0].birth_threshold, 500);
        }
    }

    #[test]
    fn a31_review_wadsworth_does_not_supply_breeder_special_globals() {
        // PB05 Update004f4180/Draw004e4490 use DAT005e89c0/c4/c8,
        // distinct from Wadsworth's globals; no nonzero producer is proved.
        for size in [BreederSize::Small, BreederSize::Medium] {
            let mut board =
                AdventureState::new_tank4_first_stage(0x4106, &[PetKind::Wadsworth]).unwrap();
            let ward = &mut board.fish_pets[0];
            ward.ward_active = true;
            ward.published_x = 200;
            ward.published_y = 200;
            let breeder = &mut board.breeders[0];
            breeder.x = 200.0;
            breeder.y = 200.0;
            breeder.widget_x = 200;
            breeder.widget_y = 200;
            breeder.size = size;
            breeder.hunger = 400;
            board.update_breeders(&mut Vec::new());
            assert_eq!(board.breeders[0].hunger, 399, "size={size:?}");
        }
    }

    #[test]
    fn completed_profile_populates_presto_before_starters_and_retains_fixed_tank_five_duplicates() {
        let mut early =
            AdventureState::new_profile_stage(0x3755, 1, 1, &[PetKind::Stinky, PetKind::Presto], 0)
                .unwrap();
        assert_eq!(early.pets, [PetKind::Stinky, PetKind::Presto]);
        assert!(early.presto_actor().unwrap().id < early.fish[0].id);
        let opener = early.presto_actor().unwrap();
        early
            .change_presto_form(opener.id, PetKind::Stinky, &mut Vec::new())
            .unwrap();
        assert_eq!(early.stinky.len(), 2);
        early.validate().unwrap();
        for target in crate::time_trial::SUPPORTED_PETS
            .into_iter()
            .chain([PetKind::Presto])
        {
            let mut board =
                AdventureState::new_profile_stage(0x3757, 1, 1, &[PetKind::Presto], 0).unwrap();
            let initial = board.presto_actor().unwrap();
            let result = board
                .change_presto_form(initial.id, target, &mut Vec::new())
                .unwrap();
            assert_eq!(
                result,
                if target == PetKind::Presto {
                    PrestoChangeEligibility::SameForm
                } else {
                    PrestoChangeEligibility::Ready
                },
                "{target:?}"
            );
            assert_eq!(board.presto_actor().unwrap().kind, target);
            board
                .validate()
                .unwrap_or_else(|error| panic!("{target:?}: {error}"));
        }
        for kind in crate::time_trial::SUPPORTED_PETS {
            let ordinary = AdventureState::new_profile_stage(0x3759, 1, 1, &[kind], 0).unwrap();
            ordinary
                .validate()
                .unwrap_or_else(|error| panic!("ordinary {kind:?}: {error}"));
            let mut finale =
                AdventureState::new_profile_stage(0x375a, 5, 1, &[kind, PetKind::Presto], 0)
                    .unwrap();
            let opener = finale.presto_actor().unwrap();
            finale
                .change_presto_form(opener.id, kind, &mut Vec::new())
                .unwrap();
            finale
                .validate()
                .unwrap_or_else(|error| panic!("Tank 5 {kind:?}: {error}"));
        }

        let tank5 = AdventureState::new_profile_stage(
            0x3756,
            5,
            1,
            &[PetKind::Stinky, PetKind::Niko, PetKind::Presto],
            4,
        )
        .unwrap();
        assert_eq!(
            &tank5.pets[..3],
            &[PetKind::Stinky, PetKind::Niko, PetKind::Presto]
        );
        assert_eq!(&tank5.pets[3..], TANK5_PETS);
        assert_eq!(tank5.stinky.len(), 2);
        assert_eq!(tank5.niko.len(), 2);
        assert_ne!(tank5.stinky[0].combat_id, tank5.stinky[1].combat_id);
        tank5.validate().unwrap();
    }

    #[test]
    fn time_trial_presto_maps_all_twenty_raw_forms_with_new_physical_identity() {
        for target in [
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
            PetKind::Presto,
        ] {
            let mut board = AdventureState::new_time_trial(0x3750, 3, &[PetKind::Presto]).unwrap();
            let first = board.presto_actor().unwrap();
            let before = serde_json::to_value(&board).unwrap();
            let mut events = Vec::new();
            let eligibility = board
                .change_presto_form(first.id, target, &mut events)
                .unwrap();
            if target == PetKind::Presto {
                assert_eq!(eligibility, PrestoChangeEligibility::SameForm);
                assert_eq!(serde_json::to_value(&board).unwrap(), before);
                continue;
            }
            assert_eq!(eligibility, PrestoChangeEligibility::Ready, "{target:?}");
            let next = board.presto_actor().unwrap();
            assert_eq!(next.kind, target);
            assert_eq!(
                (next.widget_x, next.widget_y),
                (first.widget_x, first.widget_y)
            );
            assert!(next.id > first.id);
            assert_eq!(next.remaining_ticks, 360);
            assert_eq!(board.pets, [PetKind::Presto]);
            assert_eq!(
                events
                    .iter()
                    .filter(|event| matches!(event, Event::PrestoChanged { .. }))
                    .count(),
                1
            );
            board
                .validate()
                .unwrap_or_else(|error| panic!("{target:?}: {error}"));
        }
    }

    #[test]
    fn time_trial_presto_counterpart_and_recharge_boundary_preserve_roster_and_rng() {
        let mut board =
            AdventureState::new_time_trial(0x3751, 1, &[PetKind::Stinky, PetKind::Presto]).unwrap();
        let first = board.presto_actor().unwrap();
        board
            .change_presto_form(first.id, PetKind::Stinky, &mut Vec::new())
            .unwrap();
        assert_eq!(board.stinky.len(), 2);
        assert_eq!(
            board
                .stinky
                .iter()
                .filter(|pet| pet.presto_form.is_some())
                .count(),
            1
        );
        board.validate().unwrap();
        let form = board.presto_actor().unwrap();
        let before = serde_json::to_value(&board).unwrap();
        assert_eq!(
            board
                .change_presto_form(form.id, PetKind::Niko, &mut Vec::new())
                .unwrap(),
            PrestoChangeEligibility::CoolingDown
        );
        assert_eq!(serde_json::to_value(&board).unwrap(), before);
        board
            .stinky
            .iter_mut()
            .find(|pet| pet.combat_id == Some(form.id))
            .unwrap()
            .presto_form
            .as_mut()
            .unwrap()
            .remaining_ticks = 1;
        board.update_stinky(&mut Vec::new());
        assert_eq!(board.presto_actor().unwrap().remaining_ticks, 0);
        board
            .change_presto_form(form.id, PetKind::Niko, &mut Vec::new())
            .unwrap();
        assert_eq!(board.stinky.len(), 1);
        assert_eq!(board.niko.len(), 1);
        board.validate().unwrap();
    }

    #[test]
    fn presto_replacement_detaches_an_assignment_to_the_retired_identity() {
        let mut board = AdventureState::new_time_trial(0x3756, 3, &[PetKind::Presto]).unwrap();
        let old_id = board.presto_actor().unwrap().id;
        let missile_id = board.id();
        // Exercise the Board transaction with an assigned missile directly.
        // Ordinary Time Trial projectile targeting does not select pets.
        board
            .missiles
            .push(ClassicMissile::launch(missile_id, old_id, 200, 200, 0));
        board.invasion.as_mut().unwrap().battle_active = true;
        let mut events = Vec::new();
        board
            .change_presto_form(old_id, PetKind::Stinky, &mut events)
            .unwrap();
        assert!(board.missiles.is_empty());
        assert!(events.iter().any(|event| {
            matches!(event, Event::MissileRemoved { missile_id: removed, .. } if *removed == missile_id)
        }));
        assert!(board.presto_actor().unwrap().id > old_id);
        board.validate().unwrap();
    }

    #[test]
    fn flagged_stinky_at_exact_floor_overshoots_before_strict_next_tick_clamp() {
        let mut board = AdventureState::new_time_trial(0x3752, 1, &[PetKind::Presto]).unwrap();
        let first = board.presto_actor().unwrap();
        let fish = board
            .fish_pets
            .iter_mut()
            .find(|pet| pet.id == first.id)
            .unwrap();
        fish.y = 370.0;
        fish.widget_y = 370;
        board
            .change_presto_form(first.id, PetKind::Stinky, &mut Vec::new())
            .unwrap();
        board.update_stinky(&mut Vec::new());
        assert!(board.stinky[0].y > 370.0);
        assert_eq!(board.stinky[0].presto_form.unwrap().remaining_ticks, 359);
        board.update_stinky(&mut Vec::new());
        assert_eq!(board.stinky[0].y, 370.0);
        assert_eq!(board.stinky[0].vy, 0.0);
    }

    #[test]
    fn time_trial_retired_presto_niko_pearls_keep_distinct_owners_and_credit_once() {
        let mut board =
            AdventureState::new_time_trial(0x3753, 2, &[PetKind::Niko, PetKind::Presto]).unwrap();
        let original = board.presto_actor().unwrap();
        board
            .change_presto_form(original.id, PetKind::Niko, &mut Vec::new())
            .unwrap();
        let first_owner = board.presto_actor().unwrap().id;
        let first_pearl = board.id();
        board
            .pearls
            .push(NikoPearl::spawn(first_pearl, first_owner, 100, 100));
        board
            .niko
            .iter_mut()
            .find(|pet| pet.owner_id == first_owner)
            .unwrap()
            .presto_form
            .as_mut()
            .unwrap()
            .remaining_ticks = 0;
        board
            .change_presto_form(first_owner, PetKind::Prego, &mut Vec::new())
            .unwrap();
        let prego_id = board.presto_actor().unwrap().id;
        board
            .fish_pets
            .iter_mut()
            .find(|pet| pet.id == prego_id)
            .unwrap()
            .presto_form
            .as_mut()
            .unwrap()
            .remaining_ticks = 0;
        board
            .change_presto_form(prego_id, PetKind::Niko, &mut Vec::new())
            .unwrap();
        let second_owner = board.presto_actor().unwrap().id;
        let second_pearl = board.id();
        board
            .pearls
            .push(NikoPearl::spawn(second_pearl, second_owner, 170, 100));
        board
            .niko
            .iter_mut()
            .find(|pet| pet.owner_id == second_owner)
            .unwrap()
            .presto_form
            .as_mut()
            .unwrap()
            .remaining_ticks = 0;
        board
            .change_presto_form(second_owner, PetKind::Presto, &mut Vec::new())
            .unwrap();
        assert_ne!(first_owner, second_owner);
        assert_eq!(board.niko.len(), 1); // The unrelated ordinary Niko persists.
        board.validate().unwrap();
        let resumed: AdventureState =
            serde_json::from_slice(&serde_json::to_vec(&board).unwrap()).unwrap();
        resumed.validate().unwrap();
        let mut aliased = board.clone();
        aliased.pearls[0].owner_id = aliased.fish[0].id;
        assert!(aliased.validate().is_err());
        aliased.pearls[0].owner_id = aliased.pearls[1].id;
        assert!(aliased.validate().is_err());
        let events = board.apply(Action::Click { x: 104.0, y: 104.0 });
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Event::PearlCollectionStarted {
            owner_id, .. } if *owner_id == first_owner))
        );
        assert!(!board.niko[0].pearl_taken);
        board.pearls[0].x = 100.0;
        board.pearls[0].y = 35.0;
        board.pearls[0].widget_x = 100;
        board.pearls[0].widget_y = 35;
        let balance = board.balance;
        board.update_pearls(&mut Vec::new());
        assert_eq!(board.balance, balance + PEARL_VALUE);
        board.update_pearls(&mut Vec::new());
        assert_eq!(board.balance, balance + PEARL_VALUE);
        board.validate().unwrap();
    }

    #[test]
    fn time_trial_presto_children_survive_later_form_changes_without_emitter_ids() {
        let mut board = AdventureState::new_time_trial(0x3754, 3, &[PetKind::Presto]).unwrap();
        let zorf_food = board.id();
        board.food.push(Food {
            id: zorf_food,
            x: 110.0,
            y: 120.0,
            frame: 0,
            food_type: FoodType::Ordinary,
            picked_up: false,
            ineligible_ticks: 0,
            removal_ticks: 0,
            quality: 1,
            direction: 1,
            vx: 0.0,
            vy: 0.0,
            animation_period: 3,
            free_from_zorf: true,
            nimbus_rising: false,
        });
        let nimbus_food = board.id();
        board.food.push(Food {
            id: nimbus_food,
            x: 120.0,
            y: 130.0,
            frame: 0,
            food_type: FoodType::Ordinary,
            picked_up: false,
            ineligible_ticks: 0,
            removal_ticks: 0,
            quality: 0,
            direction: 0,
            vx: 0.0,
            vy: 0.0,
            animation_period: 3,
            free_from_zorf: false,
            nimbus_rising: true,
        });
        let bomb_id = board.id();
        board.coins.push(Coin {
            id: bomb_id,
            x: 180.0,
            y: 160.0,
            kind: CoinKind::ShrapnelBomb,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            collecting: false,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: false,
        });
        let shell_id = board.id();
        board.coins.push(Coin {
            id: shell_id,
            x: 200.0,
            y: 160.0,
            kind: CoinKind::ShellGold,
            frame: 0,
            animation_ticks: 0,
            hazard_age_ticks: 0,
            collecting: false,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: false,
        });
        let note_id = board.id();
        board.notes.push(NoteState {
            id: note_id,
            x: 200,
            y: 180,
            age_ticks: 0,
        });
        board.bomb_shots.push(BombShot {
            shot_type: 3,
            x: 180,
            y: 160,
            age_ticks: 0,
            frame: -1,
            delay_ticks: 0,
            alpha: 100,
        });
        let old = board.presto_actor().unwrap();
        board
            .change_presto_form(old.id, PetKind::Itchy, &mut Vec::new())
            .unwrap();
        assert_eq!(board.presto_actor().unwrap().kind, PetKind::Itchy);
        board.validate().unwrap();
        let resumed: AdventureState =
            serde_json::from_slice(&serde_json::to_vec(&board).unwrap()).unwrap();
        resumed.validate().unwrap();
        let mut invalid = resumed;
        invalid
            .coins
            .iter_mut()
            .find(|coin| coin.id == bomb_id)
            .unwrap()
            .fade_ticks = 1;
        assert!(invalid.validate().is_err());
    }

    fn brinkley_food(board: &mut AdventureState, x: f32, y: f32) -> u64 {
        let id = board.id();
        board.food.push(Food {
            id,
            x,
            y,
            frame: 0,
            food_type: FoodType::Ordinary,
            picked_up: false,
            ineligible_ticks: 0,
            removal_ticks: 0,
            quality: 0,
            direction: 0,
            vx: 0.0,
            vy: 0.0,
            animation_period: 3,
            free_from_zorf: false,
            nimbus_rising: false,
        });
        id
    }

    #[test]
    fn brinkley_first_food_vector_strict_contact_and_registered_threat_gates() {
        let mut board = AdventureState::new_time_trial(0x3920, 1, &[PetKind::Brinkley]).unwrap();
        let pet = &mut board.fish_pets[0];
        pet.x = 100.0;
        pet.y = 100.0;
        pet.widget_x = 100;
        pet.widget_y = 100;
        let first = brinkley_food(&mut board, 105.0, 110.0); // x+35 == center: excluded
        let second = brinkley_food(&mut board, 106.0, 110.0);
        board.invasion.as_mut().unwrap().battle_active = true; // latch alone cannot block
        let mut events = Vec::new();
        board.update_fish_pets(&mut events);
        assert_eq!(
            board.food.iter().map(|food| food.id).collect::<Vec<_>>(),
            vec![first]
        );
        assert!(events.iter().any(|event| matches!(event,
            Event::BrinkleyFoodEaten { food_id, meals: 1, .. } if *food_id == second
        )));
        assert_eq!(board.fish_pets[0].brinkley_cooldown, 45);
        board.fish_pets[0].brinkley_cooldown = 1;
        let alien_id = board.id();
        board
            .invasion
            .as_mut()
            .unwrap()
            .actors
            .push(crate::alien::WeakSylvester::spawn(alien_id, 400, 300, 1, 1));
        let third = brinkley_food(&mut board, 106.0, 110.0);
        board.update_fish_pets(&mut Vec::new());
        assert!(board.food.iter().any(|food| food.id == third));
        assert_eq!(board.fish_pets[0].brinkley_meals, 1);
    }

    #[test]
    fn brinkley_third_meal_coin_is_board_owned_after_presto_morph() {
        let mut board = AdventureState::new_time_trial(0x3921, 1, &[PetKind::Presto]).unwrap();
        let first = board.presto_actor().unwrap();
        board
            .change_presto_form(first.id, PetKind::Brinkley, &mut Vec::new())
            .unwrap();
        let pet = board
            .fish_pets
            .iter_mut()
            .find(|pet| pet.presto_form.is_some())
            .unwrap();
        pet.x = 100.0;
        pet.y = 100.0;
        pet.widget_x = 100;
        pet.widget_y = 100;
        pet.brinkley_meals = 134;
        pet.brinkley_cooldown = 1;
        brinkley_food(&mut board, 106.0, 110.0);
        board.update_fish_pets(&mut Vec::new());
        assert_eq!(board.coins.len(), 1);
        assert_eq!(board.coins[0].kind, CoinKind::Treasure);
        assert_eq!((board.coins[0].x, board.coins[0].y), (105.0, 90.0));
        let coin_id = board.coins[0].id;
        let old = board.presto_actor().unwrap();
        board
            .fish_pets
            .iter_mut()
            .find(|pet| pet.id == old.id)
            .unwrap()
            .presto_form
            .as_mut()
            .unwrap()
            .remaining_ticks = 0;
        board
            .change_presto_form(old.id, PetKind::Niko, &mut Vec::new())
            .unwrap();
        assert!(board.coins.iter().any(|coin| coin.id == coin_id));
        board.validate().unwrap();
        let restored: AdventureState =
            serde_json::from_slice(&serde_json::to_vec(&board).unwrap()).unwrap();
        restored.validate().unwrap();
        assert!(restored.coins.iter().any(|coin| coin.id == coin_id));
    }

    #[test]
    fn brinkley_target_differs_from_first_widget_overlap_and_pays_before_motion() {
        let mut board = AdventureState::new_time_trial(0x3922, 1, &[PetKind::Brinkley]).unwrap();
        board.upgrades.quantity_unlocked = true;
        board.upgrades.quantity = 2;
        let pet = &mut board.fish_pets[0];
        pet.x = 100.75;
        pet.y = 100.25;
        pet.widget_x = 100;
        pet.widget_y = 100;
        pet.brinkley_meals = 2;
        pet.published_x = 100;
        pet.published_y = 100;
        let first_overlap = brinkley_food(&mut board, 106.9, 110.9);
        // This is the nearest target, but slot86 removes the first overlap.
        let selected = brinkley_food(&mut board, 120.9, 120.9);
        board.validate().unwrap();
        let old_rng = board.rng_state;
        let mut events = Vec::new();
        board.update_fish_pets(&mut events);
        assert_eq!(
            board.food.iter().map(|food| food.id).collect::<Vec<_>>(),
            vec![selected]
        );
        assert!(events.iter().any(|event| matches!(event,
            Event::BrinkleyFoodEaten { food_id, meals: 3, .. } if *food_id == first_overlap
        )));
        assert_eq!(
            (board.coins[0].x, board.coins[0].y, board.coins[0].kind),
            (105.0, 90.0, CoinKind::Star)
        );
        assert_ne!(board.fish_pets[0].x, 100.75); // Common tail ran after payout.
        assert_eq!(board.fish_pets[0].brinkley_cooldown, 45);
        // The payout is an independent Board child, valid across a save/reopen.
        let restored: AdventureState =
            serde_json::from_slice(&serde_json::to_vec(&board).unwrap()).unwrap();
        restored.validate().unwrap();
        assert_eq!(restored.coins[0].id, board.coins[0].id);
        assert_eq!(restored.rng_state, board.rng_state);
        assert_eq!(board.rng_state, old_rng); // No movement-state draw was due.

        let mut fractional =
            AdventureState::new_time_trial(0x3923, 1, &[PetKind::Brinkley]).unwrap();
        fractional.upgrades.quantity_unlocked = true;
        fractional.upgrades.quantity = 2;
        let pet = &mut fractional.fish_pets[0];
        pet.x = 100.75;
        pet.y = 100.25;
        pet.widget_x = 100;
        pet.widget_y = 100;
        pet.published_x = 100;
        pet.published_y = 100;
        // Continuous Food X would admit this entry, but its widget X does not.
        let outside = brinkley_food(&mut fractional, 105.9, 110.9);
        brinkley_food(&mut fractional, 106.9, 110.9);
        fractional.validate().unwrap();
        fractional.update_fish_pets(&mut Vec::new());
        assert_eq!(
            fractional
                .food
                .iter()
                .map(|food| food.id)
                .collect::<Vec<_>>(),
            vec![outside]
        );
        assert_eq!(fractional.fish_pets[0].brinkley_meals, 1);
        fractional.validate().unwrap();
    }

    #[test]
    fn nostradamus_threshold_produces_independent_food_at_old_widget() {
        let mut board = AdventureState::new_time_trial(0x3930, 1, &[PetKind::Nostradamus]).unwrap();
        let pet_id = board.fish_pets[0].id;
        board.fish_pets[0].nostra_threshold = 300;
        board.fish_pets[0].nostra_elapsed = 299;
        let old_widget = (board.fish_pets[0].widget_x, board.fish_pets[0].widget_y);
        let mut expected_rng = board.rng_state;
        let expected_period = (AdventureState::advance_rng(&mut expected_rng) % 2) as u8 + 3;
        let mut events = Vec::new();
        board.update_fish_pets(&mut events);
        let food = &board.food[0];
        assert_eq!(
            (food.x as i32, food.y as i32),
            (old_widget.0 + 15, old_widget.1 + 10)
        );
        assert_eq!(food.food_type, FoodType::Nostradamus);
        assert_eq!(
            (food.ineligible_ticks, food.animation_period),
            (20, expected_period)
        );
        assert_eq!(board.fish_pets[0].nostra_elapsed, 0);
        assert!(events.iter().any(|event| matches!(event,
            Event::NostradamusFoodDropped { pet_id: id, food_id, .. }
                if *id == pet_id && *food_id == food.id
        )));
        board.validate().unwrap();
        board.food[0].ineligible_ticks = 21;
        assert_eq!(
            board.validate().unwrap_err(),
            "Nostradamus Food clock exceeds its producer bounds"
        );
        board.food[0].ineligible_ticks = 20;
        board.fish_pets[0].nostra_threshold = 1080;
        assert_eq!(
            board.validate().unwrap_err(),
            "Board Nostradamus uses a Virtual Tank threshold"
        );
    }

    #[test]
    fn nostradamus_food_survives_presto_form_change_but_requires_logical_producer() {
        let mut board = AdventureState::new_time_trial(0x3937, 1, &[PetKind::Presto]).unwrap();
        let first = board.presto_actor().unwrap().id;
        board.fish_pets[0]
            .presto_form
            .as_mut()
            .unwrap()
            .remaining_ticks = 0;
        assert_eq!(
            board
                .change_presto_form(first, PetKind::Nostradamus, &mut Vec::new())
                .unwrap(),
            PrestoChangeEligibility::Ready
        );
        let nostra = board.presto_actor().unwrap().id;
        let pet = board
            .fish_pets
            .iter_mut()
            .find(|pet| pet.id == nostra)
            .unwrap();
        pet.nostra_threshold = 300;
        pet.nostra_elapsed = 299;
        board.update_fish_pets(&mut Vec::new());
        assert_eq!(board.food[0].food_type, FoodType::Nostradamus);
        board
            .fish_pets
            .iter_mut()
            .find(|pet| pet.id == nostra)
            .unwrap()
            .presto_form
            .as_mut()
            .unwrap()
            .remaining_ticks = 0;
        board
            .change_presto_form(nostra, PetKind::Stinky, &mut Vec::new())
            .unwrap();
        board.validate().unwrap();
        board.pets[0] = PetKind::Stinky;
        assert_eq!(
            board.validate().unwrap_err(),
            "Nostradamus child state lacks a producing roster"
        );
    }

    #[test]
    fn nostradamus_food_pickup_and_uneatable_clock_do_not_filter_brinkley_contact() {
        let mut board =
            AdventureState::new_time_trial(0x3938, 1, &[PetKind::Brinkley, PetKind::Nostradamus])
                .unwrap();
        let brinkley = board
            .fish_pets
            .iter_mut()
            .find(|pet| pet.kind == FishPetKind::Brinkley)
            .unwrap();
        brinkley.x = 100.0;
        brinkley.y = 100.0;
        brinkley.widget_x = 100;
        brinkley.widget_y = 100;
        brinkley.published_x = 100;
        brinkley.published_y = 100;
        let food_id = brinkley_food(&mut board, 106.0, 110.0);
        board.food[0].food_type = FoodType::Nostradamus;
        board.food[0].ineligible_ticks = 20;
        board.food[0].picked_up = true;
        board.validate().unwrap();
        let mut events = Vec::new();
        board.update_fish_pets(&mut events);
        assert!(board.food.is_empty());
        assert!(events.iter().any(|event| matches!(event,
            Event::BrinkleyFoodEaten { food_id: id, meals: 1, .. } if *id == food_id
        )));
        board.validate().unwrap();
    }

    #[test]
    fn nostradamus_food_nimbus_capacity_failure_removes_old_and_refunds_five_cash() {
        let mut board =
            AdventureState::new_time_trial(0x3939, 1, &[PetKind::Nimbus, PetKind::Nostradamus])
                .unwrap();
        let nimbus = board
            .fish_pets
            .iter_mut()
            .find(|pet| pet.kind == FishPetKind::Nimbus)
            .unwrap();
        nimbus.x = 100.0;
        nimbus.y = 320.0;
        nimbus.widget_x = 100;
        nimbus.widget_y = 320;
        nimbus.published_x = 100;
        nimbus.published_y = 320;
        let ordinary_id = brinkley_food(&mut board, 400.0, 100.0);
        let exotic_id = brinkley_food(&mut board, 100.0, 340.0);
        board.food[1].food_type = FoodType::Nostradamus;
        board.food[1].ineligible_ticks = 20;
        let old_cash = board.balance;
        board.validate().unwrap();
        let mut events = Vec::new();
        board.update_fish_pets(&mut events);
        assert_eq!(
            board.food.iter().map(|food| food.id).collect::<Vec<_>>(),
            vec![ordinary_id]
        );
        assert!(board.food.iter().all(|food| food.id != exotic_id));
        assert_eq!(board.balance, old_cash + FOOD_PRICE);
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, Event::NimbusFoodConverted { .. }))
        );
        board.validate().unwrap();
    }

    #[test]
    fn nostradamus_sneeze_counts_conversion_duplicate_and_reopens_during_postponement() {
        let mut board = AdventureState::new_time_trial(0x3940, 1, &[PetKind::Nostradamus]).unwrap();
        let ordinary_id = board.fish_pets[0].id;
        let converted_id = board.id();
        let mut rng_state = board.rng_state;
        let converted =
            FishPetState::spawn_nostra_conversion_at(converted_id, 100, 200, &mut |upper| {
                AdventureState::advance_rng(&mut rng_state) % upper
            });
        board.rng_state = rng_state;
        board.fish_pets.push(converted);
        let wave = board.invasion.as_mut().unwrap();
        wave.countdown = 31;
        wave.warning = Some(crate::invasion::WarningCoords {
            first_x: 20,
            first_y: 105,
            second_x: 30,
            second_y: 110,
        });
        board.validate().unwrap();
        let mut found = false;
        // The shared stream determines whether the leading branch succeeds.
        // Find an admitted seed without hard-coding actor-constructor draws.
        for seed in 1..1000 {
            let mut candidate = board.clone();
            candidate.rng_state = seed;
            let events = candidate.step(&[]);
            if let Some((effect_positions, repeat_delays)) =
                events.iter().find_map(|event| match event {
                    Event::Invasion {
                        event:
                            InvasionEvent::NostradamusPostponed {
                                effect_positions,
                                repeat_delays,
                            },
                        ..
                    } => Some((effect_positions, repeat_delays)),
                    _ => None,
                })
            {
                assert_eq!(effect_positions.len(), 10);
                assert!(
                    effect_positions[..5]
                        .iter()
                        .all(|(id, _, _)| *id == ordinary_id)
                );
                assert!(
                    effect_positions[5..]
                        .iter()
                        .all(|(id, _, _)| *id == converted_id)
                );
                assert_eq!(repeat_delays.len(), 2);
                assert_eq!(
                    (
                        candidate.invasion.as_ref().unwrap().countdown,
                        candidate.invasion.as_ref().unwrap().sneeze_shake_ticks
                    ),
                    (635, 30)
                );
                candidate.validate().unwrap();
                let mut reopened: AdventureState =
                    serde_json::from_slice(&serde_json::to_vec(&candidate).unwrap()).unwrap();
                reopened.validate().unwrap();
                candidate.paused_board_update();
                reopened.paused_board_update();
                assert_eq!(candidate.invasion.as_ref().unwrap().sneeze_shake_ticks, 30);
                assert_eq!(
                    serde_json::to_value(&candidate).unwrap(),
                    serde_json::to_value(&reopened).unwrap()
                );
                assert_eq!(candidate.step(&[]), reopened.step(&[]));
                assert_eq!(
                    serde_json::to_value(&candidate).unwrap(),
                    serde_json::to_value(&reopened).unwrap()
                );
                found = true;
                break;
            }
        }
        assert!(found, "a seed should reach the single under-50 branch");
    }

    #[test]
    fn nostradamus_food_conversion_uses_post_hunger_gate_and_updates_shifted_fish() {
        let mut board = AdventureState::new_time_trial(0x3931, 1, &[PetKind::Nostradamus]).unwrap();
        let fish_id = board.fish[0].id;
        board.fish[0].x = 100.0;
        board.fish[0].y = 100.0;
        board.fish[0].hunger = 109;
        board.fish[0].beginner = false;
        board.fish[0].bought_timer = 0;
        board.fish.truncate(1);
        let next_id = board.spawn_bought_guppy();
        board.fish[1].x = 300.0;
        board.fish[1].y = 200.0;
        board.fish[1].hunger = 300;
        board.fish[1].beginner = false;
        board.fish[1].bought_timer = 0;
        let food_id = brinkley_food(&mut board, 120.0, 120.0);
        board.food[0].food_type = FoodType::Nostradamus;
        board.validate().unwrap();
        board.update_fish(&mut Vec::new());
        assert!(board.fish.iter().any(|fish| fish.id == fish_id));
        assert_eq!(board.fish[0].hunger, 108); // strict <108 after this update's decrement
        board.fish[0].x = 100.0;
        board.fish[0].y = 100.0;
        let missile_id = board.id();
        board
            .missiles
            .push(ClassicMissile::launch(missile_id, fish_id, 400, 300, 0));
        board.invasion.as_mut().unwrap().battle_active = true;
        let mut events = Vec::new();
        board.update_fish(&mut events);
        assert!(!board.fish.iter().any(|fish| fish.id == fish_id));
        assert_eq!(board.fish[0].id, next_id);
        assert_eq!(board.fish[0].hunger, 298); // the shifted Fish received both updates
        assert!(board.food.iter().all(|food| food.id != food_id));
        assert!(
            board
                .missiles
                .iter()
                .all(|missile| missile.id != missile_id)
        );
        assert!(board.fish_pets.iter().any(|pet| pet.nostra_converted));
        assert!(events.iter().any(|event| matches!(event,
            Event::NostradamusFishConverted { fish_id: id, food_id: eaten, replacement_id: None, .. }
                if *id == fish_id && *eaten == food_id
        )));
        board.validate().unwrap();
        let mut resumed: AdventureState =
            serde_json::from_slice(&serde_json::to_vec(&board).unwrap()).unwrap();
        resumed.validate().unwrap();
        for _ in 0..3 {
            assert_eq!(board.step(&[]), resumed.step(&[]));
            assert_eq!(
                serde_json::to_value(&board).unwrap(),
                serde_json::to_value(&resumed).unwrap()
            );
        }
    }

    #[test]
    fn nostradamus_conversion_replaces_last_seven_category_survivor() {
        let mut board = AdventureState::new_time_trial(0x3932, 1, &[PetKind::Nostradamus]).unwrap();
        let old_id = board.fish[0].id;
        board.fish[0].x = 100.0;
        board.fish[0].y = 100.0;
        board.fish[0].hunger = 107;
        board.fish[0].beginner = false;
        board.fish[0].bought_timer = 0;
        board.fish.truncate(1);
        let food_id = brinkley_food(&mut board, 120.0, 120.0);
        board.food[0].food_type = FoodType::Nostradamus;
        let mut events = Vec::new();
        board.update_fish(&mut events);
        let replacement_id = events.iter().find_map(|event| match event {
            Event::NostradamusFishConverted {
                fish_id,
                food_id: eaten,
                replacement_id,
                ..
            } if *fish_id == old_id && *eaten == food_id => *replacement_id,
            _ => None,
        });
        assert_eq!(board.fish.len(), 1);
        assert_eq!(board.fish[0].id, replacement_id.unwrap());
        assert_eq!(
            board
                .fish_pets
                .iter()
                .filter(|pet| pet.nostra_converted)
                .count(),
            1
        );
        board.validate().unwrap();
    }

    #[test]
    fn nostradamus_food_first_equal_distance_wins_contact() {
        let mut board = AdventureState::new_time_trial(0x3936, 1, &[PetKind::Nostradamus]).unwrap();
        board.fish[0].x = 100.0;
        board.fish[0].y = 100.0;
        board.fish[0].hunger = 107;
        board.fish[0].beginner = false;
        board.fish[0].bought_timer = 0;
        let exotic_id = brinkley_food(&mut board, 120.0, 120.0);
        board.food[0].food_type = FoodType::Nostradamus;
        let ordinary_id = brinkley_food(&mut board, 120.0, 120.0);
        board.update_fish(&mut Vec::new());
        assert_eq!(
            board.food.iter().map(|food| food.id).collect::<Vec<_>>(),
            vec![ordinary_id]
        );
        assert!(board.food.iter().all(|food| food.id != exotic_id));
        assert_eq!(
            board
                .fish_pets
                .iter()
                .filter(|pet| pet.nostra_converted)
                .count(),
            1
        );
        board.validate().unwrap();
    }

    #[test]
    fn nostradamus_food_pickup_before_edible_clock_reopens_and_credits_cash_once() {
        let mut board = AdventureState::new_time_trial(0x3933, 1, &[PetKind::Nostradamus]).unwrap();
        let food_id = brinkley_food(&mut board, 120.0, 200.0);
        board.food[0].food_type = FoodType::Nostradamus;
        board.food[0].ineligible_ticks = 20;
        let original_balance = board.balance;
        let started = board.apply(Action::Click { x: 130.0, y: 210.0 });
        assert!(started.iter().any(|event| matches!(event,
            Event::NostradamusFoodCollectionStarted { food_id: id, .. } if *id == food_id
        )));
        assert!(board.food[0].picked_up);
        board.validate().unwrap();
        let mut reopened: AdventureState =
            serde_json::from_slice(&serde_json::to_vec(&board).unwrap()).unwrap();
        reopened.validate().unwrap();
        let mut credited = 0;
        for _ in 0..100 {
            board.update_food(&mut Vec::new());
            let mut events = Vec::new();
            reopened.update_food(&mut events);
            assert_eq!(
                serde_json::to_value(&board).unwrap(),
                serde_json::to_value(&reopened).unwrap()
            );
            credited += events
                .iter()
                .filter(|event| {
                    matches!(event,
                        Event::NostradamusFoodCredited { food_id: id, .. } if *id == food_id
                    )
                })
                .count();
            if credited != 0 {
                break;
            }
        }
        assert_eq!(credited, 1);
        assert!(reopened.food.iter().all(|food| food.id != food_id));
        assert_eq!(reopened.balance, original_balance + 1);
        reopened.update_food(&mut Vec::new());
        assert_eq!(reopened.balance, original_balance + 1);
        reopened.validate().unwrap();
    }

    #[test]
    fn nostradamus_pickup_checks_old_widget_y_strictly_below_forty() {
        let mut board = AdventureState::new_time_trial(0x3935, 1, &[PetKind::Nostradamus]).unwrap();
        let food_id = brinkley_food(&mut board, 120.0, 40.0);
        board.food[0].food_type = FoodType::Nostradamus;
        board.food[0].picked_up = true;
        let balance = board.balance;
        board.update_food(&mut Vec::new());
        assert_eq!(board.food[0].id, food_id);
        assert!(board.food[0].y < 40.0);
        assert_eq!(board.balance, balance);
        let mut events = Vec::new();
        board.update_food(&mut events);
        assert!(board.food.is_empty());
        assert_eq!(board.balance, balance + 1);
        assert!(events.iter().any(|event| matches!(event,
            Event::NostradamusFoodCredited { food_id: id, .. } if *id == food_id
        )));
        board.validate().unwrap();
    }

    #[test]
    fn stanley_self_lethal_impact_keeps_targetless_child_after_reward_and_reopen() {
        let mut board = AdventureState::new_time_trial(0x3941, 1, &[PetKind::Stanley]).unwrap();
        let alien_id = board.id();
        let mut alien =
            crate::alien::WeakSylvester::spawn_kind(SylvesterKind::Weak, alien_id, 300, 200, 1, 5);
        alien.health = 15.0;
        let wave = board.invasion.as_mut().unwrap();
        wave.actors.push(alien);
        wave.battle_active = true;
        let missile_id = board.id();
        board.missiles.push(ClassicMissile::launch_stanley(
            missile_id, alien_id, 300, 200, 0,
        ));
        board.validate().unwrap();
        let mut events = Vec::new();
        board.update_missiles(&mut events);
        assert!(board.invasion.as_ref().unwrap().actors.is_empty());
        assert_eq!(board.missiles.len(), 1);
        assert_eq!(
            (board.missiles[0].target_id, board.missiles[0].stanley_hit),
            (None, true)
        );
        assert_eq!(events.iter().filter(|event| matches!(event,
            Event::Invasion { event: InvasionEvent::AlienDefeated { id }, .. } if *id == alien_id
        )).count(), 1);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event,
                    Event::StanleyMissileImpacted { missile_id: id, .. } if *id == missile_id
                ))
                .count(),
            1
        );
        board.validate().unwrap();
        let restored: AdventureState =
            serde_json::from_slice(&serde_json::to_vec(&board).unwrap()).unwrap();
        restored.validate().unwrap();
        assert_eq!(restored.missiles[0].target_id, None);
    }

    #[test]
    fn stanley_external_alien_death_detaches_child_before_reward_event() {
        let mut board = AdventureState::new_time_trial(0x3943, 1, &[PetKind::Stanley]).unwrap();
        let alien_id = board.id();
        let mut alien =
            crate::alien::WeakSylvester::spawn_kind(SylvesterKind::Weak, alien_id, 300, 200, 1, 5);
        alien.health = 30.0;
        let wave = board.invasion.as_mut().unwrap();
        wave.actors.push(alien);
        wave.battle_active = true;
        let missile_id = board.id();
        board.missiles.push(ClassicMissile::launch_stanley(
            missile_id, alien_id, 80, 200, 0,
        ));
        board.validate().unwrap();
        let (_, wave_events) = board
            .invasion
            .as_mut()
            .unwrap()
            .reflected_energy_hit(alien_id)
            .unwrap();
        let mut events = Vec::new();
        board.record_invasion_events(wave_events, &mut events);
        assert!(board.missiles.is_empty());
        let removed = events
            .iter()
            .position(|event| {
                matches!(event,
                    Event::StanleyMissileRemoved { missile_id: id, .. } if *id == missile_id
                )
            })
            .unwrap();
        let reward = events.iter().position(|event| matches!(event,
            Event::Invasion { event: InvasionEvent::DiamondDropped { alien_id: id, .. }, .. } if *id == alien_id
        )).unwrap();
        assert!(removed < reward);
        board.validate().unwrap();
    }

    #[test]
    fn stanley_healing_contact_consumes_reservation_without_damage() {
        let mut board = AdventureState::new_time_trial(0x3946, 3, &[PetKind::Stanley]).unwrap();
        let alien_id = board.id();
        let mut alien = crate::alien::WeakSylvester::spawn_kind(
            SylvesterKind::Psychosquid,
            alien_id,
            300,
            200,
            3,
            5,
        );
        alien.healing = true;
        alien.ever_healed = true;
        alien.movement_divisor = 2.0;
        let health = alien.health;
        let wave = board.invasion.as_mut().unwrap();
        wave.actors.push(alien);
        wave.battle_active = true;
        let missile_id = board.id();
        board.missiles.push(ClassicMissile::launch_stanley(
            missile_id, alien_id, 300, 200, 0,
        ));
        board.validate().unwrap();
        let mut events = Vec::new();
        board.update_missiles(&mut events);
        assert_eq!(board.invasion.as_ref().unwrap().actors[0].health, health);
        assert_eq!(board.missiles.len(), 1);
        let child = &board.missiles[0];
        assert!(child.stanley_hit);
        assert_eq!(child.target_id, None);
        assert_eq!(child.movement_divisor, 1.5);
        assert!(events.iter().any(|event| matches!(event,
            Event::StanleyMissileImpacted { missile_id: id, .. } if *id == missile_id
        )));
        assert!(!events.iter().any(|event| matches!(
            event,
            Event::Invasion {
                event: InvasionEvent::AlienDefeated { .. } | InvasionEvent::DiamondDropped { .. },
                ..
            }
        )));
        board.validate().unwrap();
    }

    #[test]
    fn stanley_bilaterus_head_promotion_and_final_death_preserve_correct_children() {
        for self_impact in [false, true] {
            let mut board = AdventureState::new_time_trial(0x3947, 4, &[PetKind::Stanley]).unwrap();
            let group_id = board.id();
            let mut group = crate::bilaterus::BilaterusState::spawn(group_id, 300, 200, &mut || 1);
            group.emergence_ticks = 0;
            board.invasion.as_mut().unwrap().bilaterus.push(group);
            board.invasion.as_mut().unwrap().battle_active = true;
            let mut events = Vec::new();
            for final_head in [false, true] {
                board.invasion.as_mut().unwrap().bilaterus[0]
                    .active_mut()
                    .health = if self_impact { 15.0 } else { 30.0 };
                if !final_head || self_impact {
                    let missile_id = board.id();
                    board.missiles.push(ClassicMissile::launch_stanley(
                        missile_id,
                        group_id,
                        if self_impact { 300 } else { 80 },
                        200,
                        0,
                    ));
                }
                board.validate().unwrap();
                events.clear();
                if self_impact {
                    board.update_missiles(&mut events);
                } else {
                    let mut next_id = board.next_id;
                    let (_, wave_events) = board
                        .invasion
                        .as_mut()
                        .unwrap()
                        .reflected_energy_hit_bilaterus(
                            group_id,
                            || 1,
                            || {
                                let id = next_id;
                                next_id += 1;
                                id
                            },
                        )
                        .unwrap();
                    board.next_id = next_id;
                    board.record_invasion_events(wave_events, &mut events);
                }
                if !final_head {
                    assert!(board.invasion.as_ref().unwrap().bilaterus[0].first_head_lost);
                    assert_eq!(board.missiles.len(), 1);
                    assert_eq!(
                        board.missiles[0].target_id,
                        if self_impact { None } else { Some(group_id) }
                    );
                    assert!(
                        !events
                            .iter()
                            .any(|event| matches!(event, Event::StanleyMissileRemoved { .. }))
                    );
                } else {
                    assert!(board.invasion.as_ref().unwrap().bilaterus.is_empty());
                    if self_impact {
                        assert_eq!(board.missiles.len(), 2);
                        assert!(
                            board
                                .missiles
                                .iter()
                                .all(|child| child.stanley_hit && child.target_id.is_none())
                        );
                    } else {
                        assert!(board.missiles.is_empty());
                        let removed = events
                            .iter()
                            .position(|event| matches!(event, Event::StanleyMissileRemoved { .. }))
                            .unwrap();
                        let reward = events.iter().position(|event| matches!(event,
                            Event::Invasion { event: InvasionEvent::DiamondDropped { alien_id, .. }, .. }
                                if *alien_id == group_id
                        )).unwrap();
                        assert!(removed < reward);
                    }
                }
                board.validate().unwrap();
            }
        }
    }

    #[test]
    fn stanley_fractional_center_tie_keeps_group_then_occupied_falls_through() {
        let mut board = AdventureState::new_time_trial(0x3944, 4, &[PetKind::Stanley]).unwrap();
        let group_id = board.id();
        let alien_id = board.id();
        let group = crate::bilaterus::BilaterusState::spawn(group_id, 101, 100, &mut || 1);
        let alien = crate::alien::WeakSylvester::spawn_kind(
            SylvesterKind::Psychosquid,
            alien_id,
            60,
            60,
            4,
            5,
        );
        let wave = board.invasion.as_mut().unwrap();
        wave.bilaterus.push(group);
        wave.actors.push(alien);
        // Group centre141 and Alien centre140 both truncate to distance0
        // from the pet's continuous centre140.9; vector order wins the tie.
        assert_eq!(board.stanley_launch_target(100.9, 100.0), Some(group_id));
        let missile_id = board.id();
        board.missiles.push(ClassicMissile::launch_stanley(
            missile_id, group_id, 100, 100, 0,
        ));
        assert_eq!(board.stanley_launch_target(100.9, 100.0), Some(alien_id));
    }

    #[test]
    fn stanley_diversion_prefers_hit_energy_over_classic_and_keeps_target_links() {
        let mut board = AdventureState::new_time_trial(0x3945, 1, &[PetKind::Stanley]).unwrap();
        assert!(board.fish.len() >= 2);
        let classic_id = board.id();
        let energy_id = board.id();
        board.missiles.push(ClassicMissile::launch(
            classic_id,
            board.fish[0].id,
            100,
            100,
            0,
        ));
        let mut energy = ClassicMissile::launch_energy(energy_id, board.fish[1].id, 500, 300, 0);
        energy.immunity_ticks = 0;
        energy.reflected = true;
        board.missiles.push(energy);
        let pet_id = board.fish_pets[0].id;
        let (x, y) = (board.fish_pets[0].widget_x, board.fish_pets[0].widget_y);
        let mut events = Vec::new();
        assert!(board.divert_missile_from_stanley(pet_id, x, y, &mut events));
        assert_eq!(
            (
                board.missiles[0].diversion_clock,
                board.missiles[1].diversion_clock
            ),
            (0, 1)
        );
        assert_eq!(board.missiles[1].target_id, Some(board.fish[1].id));
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event,
                    Event::StanleyMissileDiverted { missile_id, .. } if *missile_id == energy_id
                ))
                .count(),
            1
        );
    }
}
