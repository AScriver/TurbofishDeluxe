//! First Adventure tank simulation. Functional rules are source-derived from
//! WinFish f919b3c (see docs/behavior-contract.md); retail parity is untested.
//! Positions are logical 640x480 coordinates and advance in 28 ms ticks.

use crate::{
    alien::{AlienFoodView, AlienRuntimeRequest, PreyView, SylvesterKind},
    clyde::{ClydeCoinView, ClydeState},
    fish_pet::{FishPetKind, FishPetState, PetAlienView, ZorfHungryView},
    invasion::{Invasion1_2, InvasionEvent},
    missile::{ClassicMissile, MissilePreyView},
    niko::{NikoEvent, NikoPearl, NikoState, PEARL_VALUE, PearlPhase, PearlUpdate},
    oscar::{DeadOscar, OscarPrey, OscarState},
    rufus::{RufusAlienView, RufusState},
    starcatcher::{DeadStarcatcher, StarcatcherCoinView, StarcatcherState},
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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
pub const STARCATCHER_PRICE: i32 = 750;
pub const POTION_PRICE: i32 = 250;
pub const FOOD_QUALITY_PRICE: i32 = 200;
pub const FOOD_QUANTITY_PRICE: i32 = 300;
pub const OSCAR_PRICE: i32 = 1000;
pub const WEAPON_PRICE: i32 = 1000;
const FIRST_STAGE_COIN_BOTTOM_TICKS: u16 = 150;
const SECOND_STAGE_COIN_BOTTOM_TICKS: u16 = 20;

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
}

impl CoinKind {
    pub fn value(self) -> i32 {
        match self {
            Self::Silver => 15,
            Self::Gold => 35,
            Self::Diamond => 200,
            Self::Star => 40,
            Self::DiamondPenta => 200,
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
}

const fn initial_food_animation_period() -> u8 {
    3
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Coin {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub kind: CoinKind,
    pub frame: u8,
    pub collecting: bool,
    pub bottom_ticks: u16,
    pub fade_ticks: u8,
    pub penta_rising: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Action {
    Click { x: f32, y: f32 },
    BuyGuppy,
    BuyEgg,
    BuyFoodQuality,
    BuyFoodQuantity,
    BuyOscar,
    BuyWeapon,
    BuyPotion,
    BuyStarcatcher,
    HoldFeed { x: f32, y: f32, elapsed_ms: u32 },
    HoldFire { x: f32, y: f32, elapsed_ms: u32 },
    TogglePet { pet: PetKind },
    ConfirmPetSelection { accept: bool },
    OpenMenu,
    PlayAdventure,
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
    Action {
        tick: u64,
        action: Action,
    },
    Rejected {
        tick: u64,
        reason: Rejection,
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
    MissileLaunched {
        tick: u64,
        missile_id: u64,
        target_id: u64,
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
    },
    CoinExpired {
        tick: u64,
        coin_id: u64,
    },
    CoinCredited {
        tick: u64,
        coin_id: u64,
        amount: i32,
        balance: i32,
    },
    PetCollectedCoin {
        tick: u64,
        pet: PetKind,
        coin_id: u64,
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
    pub pets: Vec<PetKind>,
    #[serde(default)]
    pub stinky: Option<StinkyState>,
    pub clyde: Option<ClydeState>,
    pub rufus: Option<RufusState>,
    pub missiles: Vec<ClassicMissile>,
    pub fish: Vec<Fish>,
    #[serde(default)]
    pub oscars: Vec<OscarState>,
    pub starcatchers: Vec<StarcatcherState>,
    pub dead_starcatchers: Vec<DeadStarcatcher>,
    #[serde(default)]
    pub fish_pets: Vec<FishPetState>,
    #[serde(default)]
    pub punch_sound_cooldown: u8,
    #[serde(default)]
    pub dead_oscars: Vec<DeadOscar>,
    pub dead_fish: Vec<DeadFish>,
    pub food: Vec<Food>,
    pub coins: Vec<Coin>,
    pub tutorial: TutorialState,
    #[serde(default)]
    pub upgrades: FoodUpgrades,
    #[serde(default)]
    pub invasion: Option<Invasion1_2>,
    #[serde(default)]
    pub niko: Option<NikoState>,
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
            weapon_unlocked: false,
            weapon_strength: 2,
            potion_unlocked: false,
            potion_armed: false,
            egg_price: EGG_PRICE,
            pets: Vec::new(),
            stinky: None,
            clyde: None,
            rufus: None,
            missiles: Vec::new(),
            fish: Vec::new(),
            oscars: Vec::new(),
            starcatchers: Vec::new(),
            dead_starcatchers: Vec::new(),
            fish_pets: Vec::new(),
            punch_sound_cooldown: 0,
            dead_oscars: Vec::new(),
            dead_fish: Vec::new(),
            food: Vec::new(),
            coins: Vec::new(),
            tutorial: TutorialState::default(),
            upgrades: FoodUpgrades::default(),
            invasion: None,
            niko: None,
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
        state.stinky = Some(state.spawn_stinky(StinkyOrigin::StageStart));
        state.spawn_starter_guppies(false);
        state
    }

    pub fn new_third_stage(seed: u64) -> Self {
        let mut state = Self::empty_board(seed);
        state.level = 3;
        state.egg_price = THIRD_STAGE_EGG_PRICE;
        state.invasion = Some(Invasion1_2::new_strong());
        state.pets = vec![PetKind::Stinky, PetKind::Niko];
        state.stinky = Some(state.spawn_stinky(StinkyOrigin::StageStart));
        let owner_id = state.id();
        state.niko = Some(NikoState::spawn_tank1(owner_id, &mut |upper| {
            state.rand_range(upper)
        }));
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
        state.stinky = Some(state.spawn_stinky(StinkyOrigin::StageStart));
        let owner_id = state.id();
        state.niko = Some(NikoState::spawn_tank1(owner_id, &mut |upper| {
            state.rand_range(upper)
        }));
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
        state.invasion = Some(Invasion1_2::new_balrog());
        state.pets = pets.to_vec();
        for pet in pets {
            match pet {
                PetKind::Stinky => {
                    state.stinky = Some(state.spawn_stinky(StinkyOrigin::StageStart))
                }
                PetKind::Niko => {
                    let owner_id = state.id();
                    state.niko = Some(NikoState::spawn_tank1(owner_id, &mut |upper| {
                        state.rand_range(upper)
                    }));
                }
                PetKind::Itchy => state.spawn_fish_pet(FishPetKind::Itchy),
                PetKind::Prego => state.spawn_fish_pet(FishPetKind::Prego),
                PetKind::Zorf
                | PetKind::Clyde
                | PetKind::Vert
                | PetKind::Rufus
                | PetKind::Meryl => {
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
                    state.stinky = Some(state.spawn_stinky(StinkyOrigin::StageStart))
                }
                PetKind::Niko => {
                    let owner_id = state.id();
                    state.niko = Some(NikoState::spawn_tank2(owner_id, &mut |upper| {
                        state.rand_range(upper)
                    }));
                }
                PetKind::Itchy => state.spawn_fish_pet(FishPetKind::Itchy),
                PetKind::Prego => state.spawn_fish_pet(FishPetKind::Prego),
                PetKind::Zorf => state.spawn_fish_pet(FishPetKind::Zorf),
                PetKind::Clyde | PetKind::Vert | PetKind::Rufus | PetKind::Meryl => {
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
                    state.stinky = Some(state.spawn_stinky(StinkyOrigin::StageStart))
                }
                PetKind::Niko => {
                    let owner_id = state.id();
                    state.niko = Some(NikoState::spawn_tank2(owner_id, &mut |upper| {
                        state.rand_range(upper)
                    }));
                }
                PetKind::Itchy => state.spawn_fish_pet(FishPetKind::Itchy),
                PetKind::Prego => state.spawn_fish_pet(FishPetKind::Prego),
                PetKind::Zorf => state.spawn_fish_pet(FishPetKind::Zorf),
                PetKind::Clyde => {
                    let id = state.id();
                    let mut rng_state = state.rng_state;
                    state.clyde = Some(ClydeState::spawn_tank2(id, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    }));
                    state.rng_state = rng_state;
                }
                PetKind::Vert | PetKind::Rufus | PetKind::Meryl => {
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
                    state.stinky = Some(state.spawn_stinky(StinkyOrigin::StageStart))
                }
                PetKind::Niko => {
                    let owner_id = state.id();
                    state.niko = Some(NikoState::spawn_tank2(owner_id, &mut |upper| {
                        state.rand_range(upper)
                    }));
                }
                PetKind::Itchy => state.spawn_fish_pet(FishPetKind::Itchy),
                PetKind::Prego => state.spawn_fish_pet(FishPetKind::Prego),
                PetKind::Zorf => state.spawn_fish_pet(FishPetKind::Zorf),
                PetKind::Clyde => {
                    let id = state.id();
                    let mut rng_state = state.rng_state;
                    state.clyde = Some(ClydeState::spawn_tank2(id, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    }));
                    state.rng_state = rng_state;
                }
                PetKind::Vert => state.spawn_fish_pet(FishPetKind::Vert),
                PetKind::Rufus | PetKind::Meryl => {
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
                    state.stinky = Some(state.spawn_stinky(StinkyOrigin::StageStart))
                }
                PetKind::Niko => {
                    let owner_id = state.id();
                    state.niko = Some(NikoState::spawn_tank2(owner_id, &mut |upper| {
                        state.rand_range(upper)
                    }));
                }
                PetKind::Itchy => state.spawn_fish_pet(FishPetKind::Itchy),
                PetKind::Prego => state.spawn_fish_pet(FishPetKind::Prego),
                PetKind::Zorf => state.spawn_fish_pet(FishPetKind::Zorf),
                PetKind::Clyde => {
                    let id = state.id();
                    let mut rng_state = state.rng_state;
                    state.clyde = Some(ClydeState::spawn_tank2(id, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    }));
                    state.rng_state = rng_state;
                }
                PetKind::Vert => state.spawn_fish_pet(FishPetKind::Vert),
                PetKind::Rufus => {
                    let id = state.id();
                    let mut rng_state = state.rng_state;
                    state.rufus = Some(RufusState::spawn_tank2(id, &mut |upper| {
                        Self::advance_rng(&mut rng_state) % upper
                    }));
                    state.rng_state = rng_state;
                }
                PetKind::Meryl => unreachable!("roster checked before construction"),
            }
        }
        state.spawn_starter_guppies(false);
        Ok(state)
    }

    fn spawn_fish_pet(&mut self, kind: FishPetKind) {
        let id = self.id();
        let mut rng_state = self.rng_state;
        let pet = FishPetState::spawn_tank1(id, kind, &mut |upper| {
            Self::advance_rng(&mut rng_state) % upper
        });
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
            && let Some(body) = wave.dead_alien.as_mut()
        {
            body.kind = wave.kind;
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
            || self.stinky.is_some()
        {
            return false;
        }
        self.stinky = Some(self.spawn_stinky(StinkyOrigin::LegacyV2Resume));
        true
    }

    fn spawn_stinky(&mut self, origin: StinkyOrigin) -> StinkyState {
        let x = self.rand_range(265) as f64 + 105.0;
        let _unused_y = self.rand_range(520) + 20;
        StinkyState {
            x,
            y: 360.0,
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

    pub(crate) fn transition_seed(&self) -> u64 {
        self.rng_state
    }

    pub(crate) fn has_live_fish(&self) -> bool {
        self.fish.iter().any(|fish| fish.alive)
            || self.oscars.iter().any(|oscar| oscar.alive)
            || self.starcatchers.iter().any(|actor| actor.alive)
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
        self.pearls
            .iter()
            .filter(|pearl| pearl.phase == PearlPhase::Collecting)
            .fold(coin_funds, |funds, _| funds.saturating_add(PEARL_VALUE))
    }

    /// Validate durable board relationships before accepting a project save.
    /// The profile and screen phase are checked by AdventureSession separately.
    pub fn validate(&self) -> Result<(), String> {
        if !((self.tank == 1 && (1..=5).contains(&self.level))
            || (self.tank == 2 && (1..=4).contains(&self.level)))
            || self.next_id == 0
            || self.next_id == u64::MAX
            || self.rng_state == 0
            || self.eggs > 3
            || self.victory != (self.eggs == 3)
            || self.upgrades.quality > 2
            || !(1..=9).contains(&self.upgrades.quantity)
            || (!self.upgrades.quality_unlocked && self.upgrades.quality > 0)
            || (!self.upgrades.quantity_unlocked && self.upgrades.quantity > 1)
            || (self.tank == 1 && self.food.len() > usize::from(self.upgrades.quantity))
            || (self.tank == 2
                && self.food.iter().filter(|food| !food.free_from_zorf).count()
                    > usize::from(self.upgrades.quantity))
            || self.food.iter().any(|food| {
                food.quality > 3
                    || food.direction > 2
                    || !(3..=4).contains(&food.animation_period)
                    || !food.x.is_finite()
                    || !food.y.is_finite()
                    || !food.vx.is_finite()
                    || !food.vy.is_finite()
                    || (food.free_from_zorf
                        && (self.tank != 2
                            || !self.pets.contains(&PetKind::Zorf)
                            || food.quality != 1
                            || food.direction == 0))
                    || (!food.free_from_zorf && food.direction != 0)
                    || (food.quality == 3 && (self.tank != 2 || !self.potion_unlocked))
            })
            || !(2..=12).contains(&self.weapon_strength)
            || (!self.weapon_unlocked && self.weapon_strength > 2)
            || self.punch_sound_cooldown > 11
            || (self.tank != 2 || !(2..=4).contains(&self.level))
                && (self.starcatcher_unlocked
                    || !self.starcatchers.is_empty()
                    || !self.dead_starcatchers.is_empty()
                    || self.clyde.is_some())
            || self.coins.iter().any(|coin| {
                (coin.penta_rising && coin.kind != CoinKind::DiamondPenta)
                    || (coin.kind == CoinKind::DiamondPenta
                        && !(self.tank == 2 && (2..=4).contains(&self.level)))
            })
            || (self.potion_armed && !self.potion_unlocked)
            || (self.tank == 1
                && (self.potion_unlocked
                    || self.potion_armed
                    || self.food.iter().any(|food| food.quality == 3)))
        {
            return Err("invalid Adventure board counters or upgrades".into());
        }
        let expected_price = if self.tank == 2 {
            match self.level {
                1 => TANK2_FIRST_EGG_PRICE,
                2 => TANK2_SECOND_EGG_PRICE,
                3 => TANK2_THIRD_EGG_PRICE,
                4 => TANK2_FOURTH_EGG_PRICE,
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
        match (self.tank, self.level) {
            (1, 1)
                if !self.pets.is_empty()
                    || self.stinky.is_some()
                    || self.niko.is_some()
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
                    || self.stinky.is_none()
                    || self.invasion.is_none()
                    || self.niko.is_some()
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
                    || self.stinky.is_none()
                    || self.niko.is_none()
                    || self
                        .invasion
                        .as_ref()
                        .is_none_or(|wave| wave.kind != SylvesterKind::Strong)
                    || !self.fish_pets.is_empty() =>
            {
                return Err("third-stage roster disagrees".into());
            }
            (1, 4)
                if self.pets.as_slice() != [PetKind::Stinky, PetKind::Niko, PetKind::Itchy]
                    || self.stinky.is_none()
                    || self.niko.is_none()
                    || self
                        .invasion
                        .as_ref()
                        .is_none_or(|wave| wave.kind != SylvesterKind::Balrog)
                    || self.fish_pets.len() != 1
                    || self.fish_pets[0].kind != FishPetKind::Itchy =>
            {
                return Err("fourth-stage roster or Balrog wave disagrees".into());
            }
            (1, 5)
                if self
                    .invasion
                    .as_ref()
                    .is_none_or(|wave| wave.kind != SylvesterKind::Balrog)
                    || self.pets.len() > 3
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
                    || self.stinky.is_some() != self.pets.contains(&PetKind::Stinky)
                    || self.niko.is_some() != self.pets.contains(&PetKind::Niko)
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
                if self
                    .invasion
                    .as_ref()
                    .is_none_or(|wave| wave.kind != SylvesterKind::Strong)
                    || self.pets.len() > 3
                    || self.pets.iter().any(|pet| {
                        matches!(
                            pet,
                            PetKind::Clyde | PetKind::Vert | PetKind::Rufus | PetKind::Meryl
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
                    || self.stinky.is_some() != self.pets.contains(&PetKind::Stinky)
                    || self.niko.is_some() != self.pets.contains(&PetKind::Niko)
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
                    || (self.upgrades.quality_unlocked != self.upgrades.quantity_unlocked)
                    || (self.upgrades.quality_unlocked != self.potion_unlocked)
                    || (self.upgrades.quality_unlocked != self.egg_unlocked) =>
            {
                return Err("second-tank roster or upgrade gates disagree".into());
            }
            (2, 2..=4)
                if self.invasion.as_ref().is_none_or(|wave| {
                    wave.kind
                        != match self.level {
                            2 => SylvesterKind::Balrog,
                            3 => SylvesterKind::Gus,
                            _ => SylvesterKind::Destructor,
                        }
                }) || self.pets.len() > 3
                    || self.pets.contains(&PetKind::Meryl)
                    || (self.level != 4 && self.pets.contains(&PetKind::Rufus))
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
                        ];
                        canonical.iter().position(|pet| *pet == pair[0])
                            >= canonical.iter().position(|pet| *pet == pair[1])
                    })
                    || self.stinky.is_some() != self.pets.contains(&PetKind::Stinky)
                    || self.niko.is_some() != self.pets.contains(&PetKind::Niko)
                    || self.clyde.is_some() != self.pets.contains(&PetKind::Clyde)
                    || self.rufus.is_some() != self.pets.contains(&PetKind::Rufus)
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
                                _ => None,
                            })
                            .collect::<Vec<_>>()
                    || self.oscar_unlocked
                    || !self.oscars.is_empty()
                    || !self.dead_oscars.is_empty()
                    || self.upgrades.quality_unlocked != self.upgrades.quantity_unlocked
                    || self.upgrades.quality_unlocked != self.potion_unlocked
                    || self.upgrades.quality_unlocked != self.starcatcher_unlocked
                    || self.egg_unlocked != self.weapon_unlocked
                    || (self.egg_unlocked && !self.starcatcher_unlocked) =>
            {
                return Err("second-tank roster or purchase gates disagree".into());
            }
            _ => {}
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
        if let Some(stinky) = &self.stinky {
            stinky.validate()?;
        }
        if let Some(clyde) = &self.clyde {
            clyde.validate()?;
        }
        if let Some(rufus) = &self.rufus {
            rufus.validate()?;
        }
        if (self.tank, self.level) != (2, 4) && (!self.missiles.is_empty() || self.rufus.is_some())
        {
            return Err("Destructor missiles or Rufus outside Adventure 2-4".into());
        }
        let mut assigned = HashSet::new();
        for missile in &self.missiles {
            missile.validate()?;
            if !assigned.insert(missile.target_id) || !self.live_prey_exists(missile.target_id) {
                return Err("duplicate or missing classic missile target".into());
            }
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
        if let Some(wave) = &self.invasion {
            wave.validate()?;
            if (self.tank, self.level) == (1, 2) && wave.kind != SylvesterKind::Weak {
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
            pet.validate()?;
        }
        if let Some(niko) = &self.niko {
            niko.validate()?;
            if (self.tank == 1
                && (niko.anchor_x, niko.anchor_y) != (crate::niko::NIKO_X, crate::niko::NIKO_Y))
                || (self.tank == 2
                    && (niko.anchor_x, niko.anchor_y)
                        != (crate::niko::NIKO_TANK2_X, crate::niko::NIKO_TANK2_Y))
            {
                return Err("Niko anchor disagrees with tank".into());
            }
            for pearl in &self.pearls {
                pearl.validate()?;
                if pearl.owner_id != niko.owner_id || pearl.phase == PearlPhase::Finished {
                    return Err("pearl owner or lifecycle disagrees".into());
                }
            }
        }
        if self.niko.is_none() && !self.pearls.is_empty() {
            return Err("pearl without Niko owner".into());
        }
        // Source death leaves a guppy's inactive list entry beside its
        // same-identity corpse until the corpse expires. Other allocated
        // entities own their ID exclusively, including pet and alien actors.
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
        for pet in &self.fish_pets {
            register(pet.id)?;
        }
        if let Some(clyde) = &self.clyde {
            register(clyde.id)?;
        }
        if let Some(rufus) = &self.rufus {
            register(rufus.id)?;
        }
        for missile in &self.missiles {
            register(missile.id)?;
        }
        if let Some(niko) = &self.niko {
            register(niko.owner_id)?;
        }
        if let Some(alien) = self.invasion.as_ref().and_then(|wave| wave.alien.as_ref()) {
            register(alien.id)?;
        }
        for food in &self.food {
            register(food.id)?;
        }
        for coin in &self.coins {
            register(coin.id)?;
        }
        for pearl in &self.pearls {
            register(pearl.id)?;
        }
        Ok(())
    }

    fn live_prey_exists(&self, id: u64) -> bool {
        self.fish.iter().any(|fish| fish.id == id && fish.alive)
            || self
                .oscars
                .iter()
                .any(|oscar| oscar.id == id && oscar.alive)
            || self
                .starcatchers
                .iter()
                .any(|actor| actor.id == id && actor.alive)
    }

    fn remove_missile(&mut self, id: u64, events: &mut Vec<Event>) -> bool {
        let Some(index) = self.missiles.iter().position(|missile| missile.id == id) else {
            return false;
        };
        self.missiles.remove(index);
        events.push(Event::MissileRemoved {
            tick: self.tick,
            missile_id: id,
        });
        true
    }

    fn shoot_first_missile(&mut self, x: i32, y: i32, events: &mut Vec<Event>) -> bool {
        let Some(id) = self
            .missiles
            .iter()
            .find(|missile| missile.shot(x, y))
            .map(|missile| missile.id)
        else {
            return false;
        };
        self.remove_missile(id, events);
        self.finish_destructor_battle(events);
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
            .find(|missile| missile.target_id == target_id)
            .map(|missile| missile.id)
        {
            self.remove_missile(id, events);
            true
        } else {
            false
        }
    }

    fn finish_destructor_battle(&mut self, events: &mut Vec<Event>) {
        if (self.tank, self.level) != (2, 4) || !self.missiles.is_empty() {
            return;
        }
        let Some(wave) = self.invasion.as_mut() else {
            return;
        };
        if wave.alien.is_some() || wave.food_delay == 36 {
            return;
        }
        wave.food_delay = 36;
        self.held_feed = None;
        self.held_fire = None;
        events.push(Event::Invasion {
            tick: self.tick,
            event: InvasionEvent::BattleEnded,
        });
    }

    fn missile_prey_view(&self, target_id: u64) -> Option<MissilePreyView> {
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
        self.starcatchers
            .iter()
            .find(|actor| actor.id == target_id && actor.alive)
            .map(|actor| MissilePreyView {
                id: target_id,
                widget_x: actor.widget_x,
                widget_y: actor.widget_y,
            })
    }

    fn update_missiles(&mut self, events: &mut Vec<Event>) {
        let mut index = 0;
        while index < self.missiles.len() {
            let missile_id = self.missiles[index].id;
            let view = self.missile_prey_view(self.missiles[index].target_id);
            let update = self.missiles[index].tick(view);
            if let Some(target_id) = update.impact_target {
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
            } else {
                index += 1;
            }
        }
    }

    fn update_rufus(&mut self, events: &mut Vec<Event>) {
        let Some(pet) = self.rufus.as_mut() else {
            return;
        };
        let aliens = self
            .invasion
            .as_ref()
            .and_then(|wave| wave.alien.as_ref())
            .map(|actor| {
                vec![RufusAlienView {
                    id: actor.id,
                    widget_x: actor.widget_x,
                    widget_y: actor.widget_y,
                    healing: false,
                }]
            })
            .unwrap_or_default();
        let mut rng_state = self.rng_state;
        let pet_id = pet.id;
        let update = pet.tick(&aliens, &mut |upper| {
            Self::advance_rng(&mut rng_state) % upper
        });
        self.rng_state = rng_state;
        if let Some(alien_id) = update.damaged_alien
            && let Some(actor) = self
                .invasion
                .as_mut()
                .and_then(|wave| wave.alien.as_mut())
                .filter(|actor| actor.id == alien_id)
            && let Some(health) = actor.rufus_hit()
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
                if !self.guppy_unlocked {
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
            Action::BuyEgg => {
                if !self.egg_unlocked {
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
                            next_tank: self.tank,
                            next_level: self.level + 1,
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
                if self.tank != 1 || !self.oscar_unlocked {
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
                    self.weapon_unlocked = true;
                    self.egg_unlocked = true;
                    events.push(Event::OscarBought {
                        tick: self.tick,
                        oscar_id: id,
                        balance: self.balance,
                    });
                }
            }
            Action::BuyStarcatcher => {
                if !(self.tank == 2 && (2..=4).contains(&self.level) && self.starcatcher_unlocked) {
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
            Action::BuyWeapon => {
                if (self.tank != 1 && !(self.tank == 2 && (2..=4).contains(&self.level)))
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
                } else if self.available_funds() < WEAPON_PRICE {
                    events.push(Event::Rejected {
                        tick: self.tick,
                        reason: Rejection::InsufficientFunds,
                    });
                } else {
                    self.balance -= WEAPON_PRICE;
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

        let on_collectible = pearl_id.is_some() || coin_id.is_some();
        let on_alien_widget = self.invasion.as_ref().is_some_and(|wave| {
            wave.alien.as_ref().is_some_and(|alien| {
                world_x >= alien.widget_x
                    && world_x < alien.widget_x + 160
                    && world_y >= alien.widget_y
                    && world_y < alien.widget_y + 160
            })
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
            .is_some_and(|wave| wave.kind == SylvesterKind::Gus && wave.has_live_alien());
        let gus_feed_delay = self.invasion.as_ref().is_some_and(|wave| {
            wave.kind == SylvesterKind::Gus
                && wave.alien.as_ref().is_some_and(|alien| {
                    let center_x = f64::from(alien.widget_x + 80);
                    let center_y = f64::from(alien.widget_y + 80);
                    center_x > f64::from(x) - 35.0
                        && center_x < f64::from(x) + 75.0
                        && center_y > f64::from(y) - 35.0
                        && center_y < f64::from(y) + 75.0
                })
        });
        if gus_live && (!on_collectible || on_alien_widget) {
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
            let click = self.invasion.as_mut().unwrap().click_with_weapon(
                world_x,
                world_y,
                self.weapon_strength,
            );
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
            let Some(owner_id) = self.niko.as_ref().map(|niko| niko.owner_id) else {
                return;
            };
            if let Some(pearl) = self.pearls.iter_mut().find(|pearl| pearl.id == pearl_id)
                && pearl.pick_up(owner_id)
            {
                if let Some(niko) = self.niko.as_mut() {
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
        } else if self.food.len() >= usize::from(self.upgrades.quantity) {
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
            ineligible_ticks,
            removal_ticks: 0,
            quality,
            direction: 0,
            vx: 0.0,
            vy: 0.0,
            animation_period,
            free_from_zorf: false,
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
            let feeding_blocked = self.invasion.as_ref().is_some_and(|wave| {
                (wave.has_live_alien() && wave.kind != SylvesterKind::Gus) || wave.food_delay > 0
            }) || !self.missiles.is_empty();
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
                    .is_some_and(|wave| wave.kind == SylvesterKind::Gus && wave.has_live_alien())
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
            && wave.kind != SylvesterKind::Gus
        {
            let missiles_present = !self.missiles.is_empty();
            let alien_present = wave.has_live_alien();
            let shot = wave.click_with_weapon(x as i32, y as i32, self.weapon_strength);
            self.record_invasion_events(shot.events, &mut events);
            if missiles_present && !alien_present {
                self.fire_missile_only_laser(x as i32, y as i32, &mut events);
            }
            self.shoot_first_missile(x as i32, y as i32, &mut events);
        }
        self.advance_board_clock();
        if let Some(wave) = self.invasion.as_mut() {
            let mut rng_state = self.rng_state;
            let mut next_id = self.next_id;
            let wave_events = wave.advance_with_threats(
                !self.missiles.is_empty(),
                || Self::advance_rng(&mut rng_state) as u32,
                || {
                    let id = next_id;
                    next_id += 1;
                    id
                },
            );
            self.rng_state = rng_state;
            self.next_id = next_id;
            self.record_invasion_events(wave_events, &mut events);
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
        self.update_dead_fish();
        self.dead_oscars.retain_mut(|corpse| !corpse.tick());
        self.dead_starcatchers.retain_mut(|corpse| !corpse.tick());
        self.update_starcatchers(&mut events);
        self.update_food(&mut events);
        self.update_fish(&mut events);
        self.update_oscars(&mut events);
        self.update_invasion_objects(&mut events);
        self.update_missiles(&mut events);
        self.update_stinky(&mut events);
        self.update_niko(&mut events);
        self.update_clyde(&mut events);
        self.update_rufus(&mut events);
        self.update_fish_pets(&mut events);
        self.update_coins(&mut events);
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
        let alien_live = self
            .invasion
            .as_ref()
            .is_some_and(Invasion1_2::has_live_alien);
        for index in 0..self.fish.len() {
            if !self.fish[index].alive {
                continue;
            }
            let choose_new_state =
                self.fish[index].movement_timer >= 20 && self.rand_range(10) == 0;
            let next_state = if choose_new_state {
                Some(self.rand_range(9) as u8 + 1)
            } else {
                None
            };
            let coin_drop;
            let mut eaten_food = None;
            let mut grew = None;
            let mut died = false;
            let mut poisoned_corpse = None;
            let mut hunger_cue = None;
            {
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
                        Self::coin_due(fish)
                    };
                } else {
                    let fish_cx = fish.x + 40.0;
                    let fish_cy = fish.y + 40.0;
                    let nearest = if fish.hunger < 500 {
                        self.food
                            .iter()
                            .enumerate()
                            .filter(|(_, food)| food.ineligible_ticks == 0)
                            .min_by(|(_, a), (_, b)| {
                                let da = (fish_cx - (a.x + 20.0)).powi(2)
                                    + (fish_cy - (a.y + 20.0)).powi(2);
                                let db = (fish_cx - (b.x + 20.0)).powi(2)
                                    + (fish_cy - (b.y + 20.0)).powi(2);
                                da.total_cmp(&db)
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
                    if nearest.is_none() && fish.bought_timer == 0 {
                        Self::wander(fish);
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
                        Self::coin_due(fish)
                    };
                    if fish.bought_timer > 0 {
                        fish.bought_timer -= 1;
                        fish.vy *= 0.9;
                    }
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
                if size == FishSize::Medium && !self.guppy_unlocked {
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
                        } else if (2..=4).contains(&self.level) {
                            self.starcatcher_unlocked = true;
                        }
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
                    x,
                    y,
                    kind,
                    frame: 0,
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
        }
    }

    fn coin_due(fish: &mut Fish) -> Option<(u64, f32, f32, CoinKind)> {
        if fish.size == FishSize::Small {
            return None;
        }
        fish.coin_timer += 1;
        if fish.coin_timer < fish.coin_threshold {
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

    fn update_dead_fish(&mut self) {
        let mut expired = Vec::new();
        for dead in &mut self.dead_fish {
            if dead.remaining_ticks == 0 {
                expired.push(dead.id);
                continue;
            }
            dead.frame = if dead.remaining_ticks >= 106 {
                (9 - (dead.remaining_ticks - 106) / 2) as u8
            } else if dead.remaining_ticks >= 103 {
                8
            } else if dead.remaining_ticks >= 101 {
                7
            } else {
                6
            };
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
    }

    fn update_food(&mut self, events: &mut Vec<Event>) {
        let mut expired = Vec::new();
        for food in &mut self.food {
            if food.ineligible_ticks > 0 {
                food.ineligible_ticks -= 1;
            }
            food.frame = (food.frame + 1) % (food.animation_period * 10);
            if food.removal_ticks > 0 {
                food.removal_ticks -= 1;
                if food.removal_ticks == 0 {
                    expired.push(food.id);
                }
                continue;
            }
            food.y += 1.5;
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
    }

    fn record_invasion_events(&mut self, wave_events: Vec<InvasionEvent>, events: &mut Vec<Event>) {
        let defeated_destructor = (self.tank, self.level) == (2, 4)
            && wave_events
                .iter()
                .any(|event| matches!(event, InvasionEvent::AlienDefeated { .. }));
        let mut detached_target = false;
        for event in wave_events {
            match event {
                InvasionEvent::PreyEaten { prey_id, .. } => {
                    // Alien::CheckCollision removes the live prey directly;
                    // there is no ordinary dead-fish corpse for this path.
                    self.fish.retain(|fish| fish.id != prey_id);
                    self.oscars.retain(|oscar| oscar.id != prey_id);
                    self.starcatchers.retain(|actor| actor.id != prey_id);
                    detached_target |= self.detach_missile_target(prey_id, events);
                }
                InvasionEvent::GusAteFood { food_id, .. } => {
                    self.food.retain(|food| food.id != food_id);
                }
                InvasionEvent::DiamondDropped { alien_id, x, y } => {
                    let coin_id = self.id();
                    self.coins.push(Coin {
                        id: coin_id,
                        x: x as f32,
                        y: y as f32,
                        kind: CoinKind::Diamond,
                        frame: 0,
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
        if defeated_destructor || detached_target {
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
                        x: x as f32,
                        y: y as f32,
                        kind: CoinKind::DiamondPenta,
                        frame: 0,
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
                    x: x as f32,
                    y: y as f32,
                    kind: CoinKind::Diamond,
                    frame: 0,
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

    fn update_invasion_objects(&mut self, events: &mut Vec<Event>) {
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
                eligible: fish.cannot_be_eaten_ticks == 0,
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
            let wave_events =
                wave.objects_update_with_runtime(&prey, &food, |request| match request {
                    AlienRuntimeRequest::Random => Self::advance_rng(&mut rng_state) as u32,
                    AlienRuntimeRequest::Launch {
                        x,
                        y,
                        center_x,
                        center_y,
                        ..
                    } => {
                        let mut farthest = None;
                        let mut best = 0_i64;
                        for candidate in prey.iter().filter(|candidate| candidate.eligible) {
                            if missiles
                                .iter()
                                .any(|missile| missile.target_id == candidate.id)
                            {
                                continue;
                            }
                            let dx = i64::from(center_x - candidate.widget_x - candidate.width / 2);
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
                        missiles.push(ClassicMissile::launch(id, target_id, x, y, visual_draw));
                        launches.push((id, target_id));
                        1
                    }
                });
            self.rng_state = rng_state;
            self.next_id = next_id;
            for (missile_id, target_id) in launches {
                events.push(Event::MissileLaunched {
                    tick: self.tick,
                    missile_id,
                    target_id,
                });
            }
            self.record_invasion_events(wave_events, events);
        }
    }

    fn update_fish_pets(&mut self, events: &mut Vec<Event>) {
        for index in 0..self.fish_pets.len() {
            // Alien::Update and its removal transaction have already run.
            // Include registered nonpositive-health aliens for Itchy contact;
            // they remain members until the next active alien update.
            let aliens = self
                .invasion
                .as_ref()
                .and_then(|wave| wave.alien.as_ref())
                .map(|actor| {
                    vec![PetAlienView {
                        id: actor.id,
                        widget_x: actor.widget_x,
                        widget_y: actor.widget_y,
                        healing: false,
                    }]
                })
                .unwrap_or_default();
            let guppy_count = self.fish.iter().filter(|fish| fish.alive).count();
            let pet_id = self.fish_pets[index].id;
            let mut rng_state = self.rng_state;
            let update = if self.fish_pets[index].kind == FishPetKind::Zorf {
                let hungry = self
                    .fish
                    .iter()
                    .filter(|fish| fish.alive)
                    .map(|fish| ZorfHungryView {
                        id: fish.id,
                        hunger: fish.hunger,
                        ordinary_diet: true,
                    })
                    .collect::<Vec<_>>();
                self.fish_pets[index].tick_zorf(&aliens, &hungry, &mut |upper| {
                    Self::advance_rng(&mut rng_state) % upper
                })
            } else {
                self.fish_pets[index].tick(&aliens, guppy_count, &mut |upper| {
                    Self::advance_rng(&mut rng_state) % upper
                })
            };
            self.rng_state = rng_state;
            if let Some(alien_id) = update.damaged_alien
                && let Some(actor) = self
                    .invasion
                    .as_mut()
                    .and_then(|wave| wave.alien.as_mut())
                    .filter(|actor| actor.id == alien_id)
                && let Some(health) = actor.itchy_hit()
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
                    x: x as f32,
                    y: y as f32,
                    kind: CoinKind::Gold,
                    frame: 0,
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
            if let Some(drop) = update.free_food {
                let food_id = self.id();
                let animation_period = self.rand_range(2) as u8 + 3;
                self.food.push(Food {
                    id: food_id,
                    x: drop.x as f32,
                    y: drop.y as f32,
                    frame: 0,
                    ineligible_ticks: 0,
                    removal_ticks: 0,
                    quality: 1,
                    direction: drop.direction,
                    vx: if drop.direction == 1 { -3.0 } else { 3.0 },
                    vy: -2.0,
                    animation_period,
                    free_from_zorf: true,
                });
                events.push(Event::ZorfFoodDropped {
                    tick: self.tick,
                    pet_id,
                    food_id,
                });
            }
        }
    }

    fn update_niko(&mut self, events: &mut Vec<Event>) {
        let Some(mut niko) = self.niko.take() else {
            return;
        };
        let mut rng_state = self.rng_state;
        let niko_events = niko.tick(&mut |upper| Self::advance_rng(&mut rng_state) % upper);
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
        self.niko = Some(niko);
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
        let Some(mut stinky) = self.stinky.take() else {
            return;
        };

        // The installed payload ranks integer coin centers by squared distance
        // from Stinky's double center. W1's recovered source expression differs;
        // equal-distance candidates retain their original coin-list order.
        if !alien_live && !self.coins.is_empty() {
            let center_x = stinky.x + 40.0;
            let center_y = stinky.y + 40.0;
            if let Some(target_index) =
                Self::stinky_target_index(&stinky, &self.coins, starcatcher_live)
            {
                if stinky.chase_timer > 4 {
                    stinky.chase_timer = 0;
                    let target_x = f64::from(self.coins[target_index].x.trunc());
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
                    if coin.collecting || (starcatcher_live && coin.kind == CoinKind::Star) {
                        return false;
                    }
                    let x = f64::from(coin.x.trunc());
                    let y = f64::from(coin.y.trunc());
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
                        amount,
                        balance: self.balance,
                    });
                    events.push(Event::CoinCredited {
                        tick: self.tick,
                        coin_id: coin.id,
                        amount,
                        balance: self.balance,
                    });
                }
            }
        } else {
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

        if alien_live {
            stinky.specialty_timer = (stinky.specialty_timer + 1).min(9);
            stinky.angry_timer = 0;
        } else {
            stinky.specialty_timer = stinky.specialty_timer.saturating_sub(1);
        }

        stinky.x = stinky.x.clamp(10.0, 550.0);
        stinky.y = stinky.y.clamp(95.0, 370.0);
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
        self.stinky = Some(stinky);
    }

    fn update_clyde(&mut self, events: &mut Vec<Event>) {
        let Some(mut clyde) = self.clyde.take() else {
            return;
        };
        let starcatcher_live = self.starcatchers.iter().any(|actor| actor.alive);
        let coin_list_nonempty = !self.coins.is_empty();
        let views = self
            .coins
            .iter()
            .map(|coin| ClydeCoinView {
                id: coin.id,
                widget_x: coin.x as i32,
                widget_y: coin.y as i32,
                eligible: !(coin.collecting || starcatcher_live && coin.kind == CoinKind::Star),
            })
            .collect::<Vec<_>>();
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
                amount,
                balance: self.balance,
            });
            events.push(Event::CoinCredited {
                tick: self.tick,
                coin_id,
                amount,
                balance: self.balance,
            });
        }
        self.clyde = Some(clyde);
    }

    fn stinky_target_index(
        stinky: &StinkyState,
        coins: &[Coin],
        starcatcher_live: bool,
    ) -> Option<usize> {
        let mut best_distance = 100_000_000_i64;
        let mut best_index = None;
        for (index, coin) in coins.iter().enumerate() {
            if coin.collecting || (starcatcher_live && coin.kind == CoinKind::Star) {
                continue;
            }
            let dx = ((stinky.x + 40.0) - (f64::from(coin.x.trunc()) + 40.0)) as i64;
            let dy = ((stinky.y + 40.0) - (f64::from(coin.y.trunc()) + 40.0)) as i64;
            let distance = dx * dx + dy * dy;
            if distance < best_distance {
                best_distance = distance;
                best_index = Some(index);
            }
        }
        best_index
    }

    fn update_coins(&mut self, events: &mut Vec<Event>) {
        let bottom_limit = match (self.tank, self.level) {
            (1, 1) => FIRST_STAGE_COIN_BOTTOM_TICKS,
            (1, 2..=5) => SECOND_STAGE_COIN_BOTTOM_TICKS,
            (2, 1..=4) => SECOND_STAGE_COIN_BOTTOM_TICKS,
            _ => unreachable!("coin lifetime for this Adventure stage is not implemented"),
        };
        let mut credited = Vec::new();
        let mut expired = Vec::new();
        for coin in &mut self.coins {
            coin.frame = (coin.frame + 1) % 10;
            if coin.collecting {
                // Coin::Update tests last tick's integer widget Y before its
                // easing calculation and Move, then credits exactly once.
                if coin.y.trunc() < 40.0 {
                    credited.push((coin.id, coin.kind.value()));
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
            if coin.kind == CoinKind::DiamondPenta && coin.penta_rising {
                if coin.y < 120.0 {
                    // The source flips the phase without moving on this update.
                    coin.penta_rising = false;
                } else {
                    coin.y -= if coin.y >= 150.0 {
                        8.0
                    } else if coin.y >= 135.0 {
                        2.5
                    } else if coin.y >= 130.0 {
                        1.5
                    } else if coin.y >= 125.0 {
                        1.0
                    } else {
                        0.5
                    };
                }
                continue;
            }
            coin.y = (coin.y + 1.5).min(370.0);
            if coin.y >= 370.0 {
                coin.bottom_ticks += 1;
                if coin.bottom_ticks >= bottom_limit {
                    coin.fade_ticks = 5;
                }
            }
        }
        for (id, amount) in credited {
            self.coins.retain(|coin| coin.id != id);
            self.balance = (self.balance + amount).min(9_999_999);
            events.push(Event::CoinCredited {
                tick: self.tick,
                coin_id: id,
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
            ineligible_ticks: 0,
            removal_ticks: 0,
            quality: 0,
            direction: 0,
            vx: 0.0,
            vy: 0.0,
            animation_period: 3,
            free_from_zorf: false,
        });
        state.coins.push(Coin {
            id: 51,
            x: 100.0,
            y: 370.0,
            kind: CoinKind::Gold,
            frame: 0,
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
        let pet = state.stinky.as_ref().unwrap();
        assert!((105.0..370.0).contains(&pet.x));
        assert_eq!(
            (pet.y, pet.vx, pet.vy, pet.previous_vx),
            (360.0, 0.0, 0.0, 1.0)
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

        state.stinky = None; // The previous v2 project-save shape.
        assert!(state.initialize_missing_stinky());
        assert_eq!(
            state.stinky.as_ref().unwrap().origin,
            StinkyOrigin::LegacyV2Resume
        );
        let restored: AdventureState =
            serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
        assert_eq!(
            restored.stinky.as_ref().unwrap().origin,
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
        let pet = state.stinky.as_mut().unwrap();
        pet.x = 200.0;
        pet.movement_state = 2;
        let initial_x = pet.x;
        state.tick();
        let pet = state.stinky.as_ref().unwrap();
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
        let pet = state.stinky.as_mut().unwrap();
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
        let pet = state.stinky.as_mut().unwrap();
        pet.x = 76.0; // Center equals coin.x + 16: no collision.
        pet.y = 360.0;
        pet.vx = 0.0;
        state.coins.push(Coin {
            id: 10,
            x: 100.0,
            y: 360.0,
            kind: CoinKind::Silver,
            frame: 0,
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
        state.stinky.as_mut().unwrap().x = 76.01;
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
        let mut pet = state.stinky.unwrap();
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
        let pet = state.stinky.as_mut().unwrap();
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
                collecting: false,
                bottom_ticks: 0,
                fade_ticks: 0,
                penta_rising: false,
            });
        }
        state.tick();
        // The installed center metric selects x=100 and brakes at its center;
        // W1's left-associative +36 term would select x=136 and accelerate.
        assert_eq!(state.stinky.as_ref().unwrap().vx, 0.0);
        assert_eq!(state.stinky.as_ref().unwrap().chase_timer, 1);
    }

    #[test]
    fn stinky_target_ties_keep_first_coin_and_steering_waits_past_four() {
        let mut tied = AdventureState::new_second_stage(0xa45d);
        let pet = tied.stinky.as_mut().unwrap();
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
                collecting: false,
                bottom_ticks: 0,
                fade_ticks: 0,
                penta_rising: false,
            });
        }
        tied.tick();
        assert_eq!(tied.stinky.as_ref().unwrap().vx, -1.0);

        let mut gated = AdventureState::new_second_stage(0xa45e);
        let pet = gated.stinky.as_mut().unwrap();
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
            collecting: false,
            bottom_ticks: 0,
            fade_ticks: 0,
            penta_rising: false,
        });
        gated.tick();
        assert_eq!(
            (
                gated.stinky.as_ref().unwrap().vx,
                gated.stinky.as_ref().unwrap().chase_timer
            ),
            (0.0, 5)
        );
        gated.tick();
        assert_eq!(
            (
                gated.stinky.as_ref().unwrap().vx,
                gated.stinky.as_ref().unwrap().chase_timer
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
        assert_eq!(board.invasion.as_ref().unwrap().kind, SylvesterKind::Balrog);
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
        board.invasion.as_mut().unwrap().alien = Some(alien);
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
        assert!(board.invasion.as_ref().unwrap().alien.is_some());
        assert!(
            board
                .invasion
                .as_ref()
                .unwrap()
                .alien
                .as_ref()
                .unwrap()
                .alive
        );
        board.validate().unwrap();
        board
            .invasion
            .as_mut()
            .unwrap()
            .alien
            .as_mut()
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
        assert!(board.invasion.as_ref().unwrap().alien.is_none());
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
        assert!(board.stinky.is_none());
        assert!(board.niko.is_none());
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

        board.invasion.as_mut().unwrap().alien = Some(crate::alien::WeakSylvester::spawn_kind(
            SylvesterKind::Balrog,
            801,
            100,
            120,
            1,
            1,
        ));
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
        board.niko.as_mut().unwrap().cycle = 1232;
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
        wave.alien = Some(crate::alien::WeakSylvester::spawn_kind(
            SylvesterKind::Strong,
            900,
            100,
            120,
            1,
            1,
        ));
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
            .alien
            .as_mut()
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
        let stinky_x = state.stinky.as_ref().unwrap().x as f32;
        state.coins.push(Coin {
            id: 700,
            x: stinky_x,
            y: 360.0,
            kind: CoinKind::Silver,
            frame: 0,
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
                .alien
                .as_ref()
                .unwrap()
                .spawn_ticks,
            14
        );
        assert_eq!(state.fish[0].hunger, hunger);
        assert_eq!(state.fish[0].coin_timer, 10);
        assert_eq!(state.coins.len(), 1);
        assert_eq!(state.stinky.as_ref().unwrap().specialty_timer, 1);
    }

    #[test]
    fn ordinary_coin_forwards_world_click_only_when_over_alien_widget() {
        let mut state = AdventureState::new_second_stage(0xa16);
        let wave = state.invasion.as_mut().unwrap();
        wave.countdown = 3000;
        wave.alien = Some(crate::alien::WeakSylvester::spawn(900, 100, 120, 1, 1));
        state.coins.push(Coin {
            id: 901,
            x: 160.0,
            y: 180.0,
            kind: CoinKind::Silver,
            frame: 0,
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
        player.invasion.as_mut().unwrap().alien = Some(actor);
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
        pet.invasion.as_mut().unwrap().alien = Some(actor);
        pet.apply(Action::Click { x: 180.0, y: 200.0 });
        let stinky = pet.stinky.as_mut().unwrap();
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
        state.niko.as_mut().unwrap().cycle = 1232;
        state.stinky.as_mut().unwrap().x = 96.0;
        state.stinky.as_mut().unwrap().y = 251.0;
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
        assert!(state.niko.as_ref().unwrap().pearl_taken);
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
                board.niko.as_ref().unwrap().anchor_x,
                board.niko.as_ref().unwrap().anchor_y
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
            ineligible_ticks: 0,
            removal_ticks: 0,
            quality: 3,
            direction: 0,
            vx: 0.0,
            vy: 0.0,
            animation_period: 3,
            free_from_zorf: false,
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
            ineligible_ticks: 0,
            removal_ticks: 0,
            quality: 3,
            direction: 0,
            vx: 0.0,
            vy: 0.0,
            animation_period: 3,
            free_from_zorf: false,
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
            ineligible_ticks: 0,
            removal_ticks: 0,
            quality: 0,
            direction: 0,
            vx: 0.0,
            vy: 0.0,
            animation_period: 3,
            free_from_zorf: false,
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
        board.potion_unlocked = true;
        board.potion_armed = true;
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
    }

    #[test]
    fn second_tank_starcatcher_purchase_unlocks_weapon_and_egg_only_on_success() {
        let mut board =
            AdventureState::new_tank2_second_stage(0x2201, &[PetKind::Niko, PetKind::Clyde])
                .unwrap();
        assert_eq!((board.balance, board.egg_price), (200, 3000));
        assert_eq!(board.invasion.as_ref().unwrap().kind, SylvesterKind::Balrog);
        assert_eq!(board.invasion.as_ref().unwrap().countdown, 3000);
        assert_eq!(
            (
                board.niko.as_ref().unwrap().anchor_x,
                board.niko.as_ref().unwrap().anchor_y
            ),
            (175, 163)
        );
        assert!(board.clyde.is_some());
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
        board.invasion.as_mut().unwrap().alien = Some(gus);
        let coin_id = board.id();
        board.coins.push(Coin {
            id: coin_id,
            x: 160.0,
            y: 160.0,
            kind: CoinKind::Silver,
            frame: 0,
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
                .alien
                .as_ref()
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
        board.invasion.as_mut().unwrap().alien = Some(gus);
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
        board.invasion.as_mut().unwrap().alien = Some(gus);
        board.apply(Action::Click { x: 105.0, y: 180.0 });
        assert_eq!(board.food[0].ineligible_ticks, 0);
        board.food.clear();
        board.apply(Action::Click { x: 106.0, y: 180.0 });
        assert_eq!(board.food[0].ineligible_ticks, 20);
        board.food.clear();
        board.invasion.as_mut().unwrap().alien.as_mut().unwrap().x = 100.5;
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
        board.invasion.as_mut().unwrap().alien = Some(gus);
        let blocked = board.update_objects();
        assert!(
            !blocked
                .iter()
                .any(|event| matches!(event, Event::VertGoldDropped { .. }))
        );
        assert!(board.invasion.as_ref().unwrap().alien.is_some());
        assert_eq!(board.fish_pets[0].coin_timer, 215);

        board
            .invasion
            .as_mut()
            .unwrap()
            .alien
            .as_mut()
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
        assert!(board.invasion.as_ref().unwrap().alien.is_none());
        assert!(board.invasion.as_ref().unwrap().dead_alien.is_none());
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
        board.invasion.as_mut().unwrap().alien = Some(alien);
        let mut events = Vec::new();
        board.update_invasion_objects(&mut events);
        assert_eq!(
            board
                .missiles
                .iter()
                .map(|missile| missile.target_id)
                .collect::<Vec<_>>(),
            [first, second, third_id]
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
                .alien
                .as_ref()
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
        board.invasion.as_mut().unwrap().alien = Some(alien);
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
        board.invasion.as_mut().unwrap().alien = None;
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
        let pet_x = board.rufus.as_ref().unwrap().widget_x;
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
        board.invasion.as_mut().unwrap().alien = Some(alien);
        let mut events = Vec::new();
        board.update_rufus(&mut events);
        assert_eq!(
            board
                .invasion
                .as_ref()
                .unwrap()
                .alien
                .as_ref()
                .unwrap()
                .health,
            0.0
        );
        assert!(board.invasion.as_ref().unwrap().alien.is_some());
        board.validate().unwrap();
        board.update_invasion_objects(&mut events);
        assert!(board.invasion.as_ref().unwrap().alien.is_none());
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
    fn missile_impact_makes_guppy_corpse_and_detaches_before_battle_end() {
        let mut board = AdventureState::new_tank2_fourth_stage(0x2403, &[]).unwrap();
        let target_id = board.fish[0].id;
        board.fish[0].x = 125.0;
        board.fish[0].y = 125.0;
        let id = board.id();
        board
            .missiles
            .push(ClassicMissile::launch(id, target_id, 100, 100, 0));
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
        board.validate().unwrap();
        let resumed: AdventureState =
            serde_json::from_str(&serde_json::to_string(&board).unwrap()).unwrap();
        resumed.validate().unwrap();
        assert_eq!(resumed.missiles[0].target_id, target_id);
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
}
