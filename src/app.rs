use crate::{
    adventure::{AdventurePhase, AdventureSession, results_offer},
    alien::SylvesterKind,
    assets::{GameAssets, SoundData},
    bonus::{BonusResult, BonusState, PurchaseReceipt, ShellKind, ShellState},
    cli::{self, Options},
    fish_pet::FishPetKind,
    fish_songs::{FishSongPlayer, SongCatalog},
    font::BitmapFont,
    install::{self, InstallIdentity},
    interlude::{InterludeController, MusicStage, RowContent},
    invasion::{InvasionEvent, InvasionTip},
    missile::MissileKind,
    music::{MusicCue, MusicOwner, MusicReport},
    oscar::OscarPose,
    profiles::{Profiles, ScoreTab},
    sandbox::{EntryCode, SelectorKey},
    settings::{GameSettings, SettingChange, SettingsBackend, SettingsCapabilities, SettingsSave},
    sim::{
        Action, AdventureState, CoinKind, Event, FishPose, FishSize, FoodType, PetKind, TICK_MS,
    },
    timing::{InputPress, StepClock, wall_deadline_reached},
    ultra::UltraPose,
    virtual_tank::{NamedFish, StoreProduct, VirtualCoinKind, VirtualFishKind},
};
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;
use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

const IMAGE_IDS: &[&str] = &[
    "IMAGE_AQUARIUM1",
    "IMAGE_AQUARIUM2",
    "IMAGE_AQUARIUM3",
    "IMAGE_AQUARIUM4",
    "IMAGE_AQUARIUM5",
    "IMAGE_AQUARIUM6",
    "IMAGE_MENUBAR",
    "IMAGE_SMALLSWIM",
    "IMAGE_SMALLEAT",
    "IMAGE_SMALLTURN",
    "IMAGE_SMALLDIE",
    "IMAGE_HUNGRYSWIM",
    "IMAGE_HUNGRYEAT",
    "IMAGE_HUNGRYTURN",
    "IMAGE_FOOD",
    "IMAGE_MONEY",
    "IMAGE_MISCITEMS",
    "IMAGE_SPARKS",
    "IMAGE_EXPLOSION",
    "IMAGE_EXPLOSIONSMALL",
    "IMAGE_EXPLOSIONTINY",
    "IMAGE_EGGPIECES",
    "IMAGE_MENUBTNU",
    "IMAGE_MENUBTNO",
    "IMAGE_OPTIONSBUTTON",
    "IMAGE_HATCHSCREEN",
    "IMAGE_HATCHREFLECTION",
    "IMAGE_SCREENTITLE",
    "IMAGE_MAINBUTTON",
    "IMAGE_EGGCRACK1",
    "IMAGE_EGGCRACK2",
    "IMAGE_EGGSHARDS",
    "IMAGE_STINKY",
    "IMAGE_NIKO",
    "IMAGE_PEARL",
    "IMAGE_SYLV",
    "IMAGE_GUS",
    "IMAGE_DESTRUCTOR",
    "IMAGE_PSYCHOSQUID",
    "IMAGE_MISSILE",
    "IMAGE_LASERS",
    "IMAGE_WARPHOLE",
    "IMAGE_WARPGLOW",
    "IMAGE_SCL_OSCAR",
    "IMAGE_LASERUPGRADES",
    "IMAGE_ITCHY",
    "IMAGE_PREGO",
    "IMAGE_BALROG",
    "IMAGE_SCREENBACK",
    "IMAGE_FISHBOX",
    "IMAGE_FISHBOXBUTTON",
    "IMAGE_PETBUTTON",
    "IMAGE_PETBUTTONHOLE",
    "IMAGE_PETBUTTONRING",
    "IMAGE_PETBUTTONREFLECT",
    "IMAGE_SCL_STINKY",
    "IMAGE_SCL_NIKO",
    "IMAGE_SCL_ITCHY",
    "IMAGE_SCL_PREGO",
    "IMAGE_ZORF",
    "IMAGE_SCL_ZORF",
    "IMAGE_CLYDE",
    "IMAGE_SCL_CLYDE",
    "IMAGE_VERT",
    "IMAGE_SCL_VERT",
    "IMAGE_RUFUS",
    "IMAGE_SCL_RUFUS",
    "IMAGE_MERYL",
    "IMAGE_SCL_MERYL",
    "IMAGE_WADSWORTH",
    "IMAGE_SCL_WADSWORTH",
    "IMAGE_SEYMOUR",
    "IMAGE_SCL_SEYMOUR",
    "IMAGE_GRUBBER",
    "IMAGE_SCL_GRUBBER",
    "IMAGE_GEKKO",
    "IMAGE_SCL_GEKKO",
    "IMAGE_SHRAPNEL",
    "IMAGE_SCL_SHRAPNEL",
    "IMAGE_GUMBO",
    "IMAGE_GUMBOLIGHT",
    "IMAGE_SCL_GUMBO",
    "IMAGE_BLIP",
    "IMAGE_SCL_BLIP",
    "IMAGE_RHUBARB",
    "IMAGE_SCL_RHUBARB",
    "IMAGE_NIMBUS",
    "IMAGE_SCL_NIMBUS",
    "IMAGE_AMP",
    "IMAGE_AMPCHARGE",
    "IMAGE_SCL_AMP",
    "IMAGE_GASH",
    "IMAGE_SCL_GASH",
    "IMAGE_ANGIE",
    "IMAGE_SCL_ANGIE",
    "IMAGE_HALO",
    "IMAGE_BOSS",
    "IMAGE_MINISYLV",
    "IMAGE_PRESTO",
    "IMAGE_SCL_PRESTO",
    "IMAGE_BRINKLEY",
    "IMAGE_SCL_BRINKLEY",
    "IMAGE_NOSTRADAMUS",
    "IMAGE_SCL_NOSTRADAMUS",
    "IMAGE_ULTRA",
    "IMAGE_SCL_ULTRA",
    "IMAGE_BILATERUS",
    "IMAGE_BREEDER",
    "IMAGE_HUNGRYBREEDER",
    "IMAGE_SCL_BREEDER",
    "IMAGE_ULYSSES",
    "IMAGE_ENERGYBALL",
    "IMAGE_HEALTHBAR",
    "IMAGE_HEALTHBARTUBE",
    "IMAGE_CROSSHAIR",
    "IMAGE_ZZZ",
    "IMAGE_STARCATCHER",
    "IMAGE_SCL_STARCATCHER",
    "IMAGE_BONUSBUCKET",
    "IMAGE_SHELLS",
    "IMAGE_MONEYBAG",
    "IMAGE_BONUS1",
    "IMAGE_BONUS2",
    "IMAGE_BONUS3",
];
const SOUND_IDS: &[&str] = &[
    "SOUND_APPLAUSE",
    "SOUND_TONE",
    "SOUND_TONEHI",
    "SOUND_TONELO",
    "SOUND_TONESUPERHI",
    "SOUND_DROPFOOD",
    "SOUND_SLURP",
    "SOUND_GROW",
    "SOUND_POINTS",
    "SOUND_BUY",
    "SOUND_BUTTONCLICK",
    "SOUND_HATCH",
    "SOUND_AWOOGA",
    "SOUND_ROAR",
    "SOUND_HIT",
    "SOUND_EXPLOSION1",
    "SOUND_EXPLOSION4",
    "SOUND_EXPLODE",
    "SOUND_MISSLE",
    "SOUND_ZAP",
    "SOUND_NIKOOPEN",
    "SOUND_NIKOCLOSE",
    "SOUND_PEARL",
    "SOUND_CHOMP",
    "SOUND_HEAL",
    "SOUND_EVILLAFF",
    "SOUND_DIE",
    "SOUND_PUNCH",
    "SOUND_BABY",
    "SOUND_SFX",
    "SOUND_SING",
    "SOUND_BONUSCOLLECT",
    "SOUND_BONUSCOUNT",
    "SOUND_UNLEASH",
    "SOUND_ROAR3",
    "SOUND_TREASURE",
    "SOUND_DIAMOND",
    "SOUND_RATTLE",
    "SOUND_SPLASHBIG",
    "SOUND_SONAR",
    "SOUND_EEL1",
    "SOUND_EEL2",
    "SOUND_EEL3",
];

const ADDITIVE_VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 tint;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
    tint = color0 / 255.0;
}"#;

const ADDITIVE_FRAGMENT: &str = r#"#version 100
varying lowp vec2 uv;
varying lowp vec4 tint;
uniform sampler2D Texture;
void main() {
    gl_FragColor = texture2D(Texture, uv) * tint;
}"#;

pub struct Presentation {
    images: HashMap<String, Texture2D>,
    sounds: HashMap<String, Arc<[u8]>>,
    fonts: HashMap<String, RenderedFont>,
    additive: Material,
}

// W1 PetsScreen positions pet IDs 10..14 on the right of the preview instead
// of extending the two left-hand columns into it. Draw and input share this.
fn pet_card_rect(index: usize) -> Rect {
    let (x, y) = match index {
        0..=4 => (25.0, 41.0 + index as f32 * 83.0),
        5..=9 => (119.0, 41.0 + (index - 5) as f32 * 83.0),
        10..=14 => (425.0, 41.0 + (index - 10) as f32 * 83.0),
        15..=19 => (519.0, 41.0 + (index - 15) as f32 * 83.0),
        20..=21 => (226.0, 290.0 + (index - 20) as f32 * 83.0),
        22..=23 => (323.0, 290.0 + (index - 22) as f32 * 83.0),
        _ => unreachable!("there are only 24 source pet cards"),
    };
    Rect::new(x, y, 90.0, 83.0)
}

fn pet_at_pointer(unlocked_pets: &[PetKind], pointer: Vec2) -> Option<PetKind> {
    unlocked_pets
        .iter()
        .enumerate()
        .find(|(index, _)| pet_card_rect(*index).contains(pointer))
        .map(|(_, pet)| *pet)
}

fn presto_choice_rect(index: usize) -> Rect {
    Rect::new(
        24.0 + (index % 5) as f32 * 120.0,
        86.0 + (index / 5) as f32 * 70.0,
        112.0,
        54.0,
    )
}

fn results_offer_rect() -> Rect {
    Rect::new(186.0, 389.0, 264.0, 38.0)
}

fn results_continue_rect(height: f32) -> Rect {
    Rect::new(186.0, 445.0, 264.0, height)
}

fn results_confirm_rect(accept: bool) -> Rect {
    Rect::new(if accept { 160.0 } else { 335.0 }, 306.0, 145.0, 38.0)
}

fn virtual_offer_rect(slot: usize) -> Rect {
    Rect::new(
        28.0 + (slot % 2) as f32 * 300.0,
        110.0 + (slot / 2) as f32 * 82.0,
        284.0,
        76.0,
    )
}

fn wall_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

fn bonus_results_display_balance(result: &BonusResult, wallet: u32) -> u32 {
    if result.purchase.purchased {
        wallet
    } else {
        result.presented_balance()
    }
}

fn presto_choice_at_pointer(unlocked_pets: &[PetKind], pointer: Vec2) -> Option<PetKind> {
    unlocked_pets.iter().copied().find(|pet| {
        crate::time_trial::selectable_pet(*pet)
            && presto_choice_rect(*pet as usize).contains(pointer)
    })
}

fn health_bar_visible_width(health: f64, starting_health: f64, image_width: f32) -> f32 {
    ((f64::from(image_width) * health / starting_health) as i32).clamp(0, image_width as i32) as f32
}

fn arms_held_feed(events: &[Event], gus_click: Option<(i32, i32)>) -> bool {
    match gus_click {
        Some((click_x, click_y)) => events.iter().any(|event| {
            matches!(
                event,
                Event::GusInitialFeedAttempt { x, y, .. }
                    if *x == click_x && *y == click_y
            )
        }),
        None => events
            .iter()
            .any(|event| matches!(event, Event::FoodDropped { .. })),
    }
}

fn death_has_missile_impact(events: &[Event], tick: u64, target_id: u64) -> bool {
    events.iter().any(|event| {
        matches!(
            event,
            Event::MissileImpacted {
                tick: impact_tick,
                target_id: impact_target,
                ..
            } if *impact_tick == tick && *impact_target == target_id
        )
    })
}

fn healing_warning_after_events(
    mut until_tick: Option<u64>,
    events: &[Event],
    playing: bool,
) -> Option<u64> {
    for event in events {
        match event {
            Event::Invasion {
                tick,
                event: InvasionEvent::PsychosquidHealingHit { .. },
            } => until_tick = Some(tick.saturating_add(500)),
            Event::Invasion {
                event:
                    InvasionEvent::PsychosquidPhaseChanged { healing: false, .. }
                    | InvasionEvent::BattleEnded,
                ..
            } => until_tick = None,
            _ => {}
        }
    }
    if playing { until_tick } else { None }
}

struct RenderedFont {
    metrics: BitmapFont,
    atlases: Vec<Texture2D>,
}

impl RenderedFont {
    fn load(game_root: &Path, assets: &GameAssets, name: &str) -> Result<Self, Box<dyn Error>> {
        let metrics = BitmapFont::load(game_root, name)?;
        let mut atlases = Vec::new();
        for layer in &metrics.layers {
            let image = assets.load_image(&layer.image_path.to_string_lossy())?;
            layer.validate_atlas(image.width, image.height)?;
            let atlas = Texture2D::from_rgba8(
                u16::try_from(image.width)?,
                u16::try_from(image.height)?,
                &image.pixels,
            );
            atlas.set_filter(FilterMode::Linear);
            atlases.push(atlas);
        }
        Ok(Self { metrics, atlases })
    }

    fn text(&self, text: &str, x: f32, baseline: f32, color: Color) {
        let characters: Vec<_> = text.chars().collect();
        for (layer, atlas) in self.metrics.layers.iter().zip(&self.atlases) {
            let mut pen = x;
            for (index, character) in characters.iter().enumerate() {
                if let Some(glyph) = layer.glyphs.get(character) {
                    if let Some(rect) = glyph.rect {
                        draw_texture_ex(
                            atlas,
                            pen + glyph.offset_x as f32,
                            baseline - (layer.ascent - glyph.offset_y) as f32,
                            color,
                            DrawTextureParams {
                                source: Some(Rect::new(
                                    rect.x as f32,
                                    rect.y as f32,
                                    rect.width as f32,
                                    rect.height as f32,
                                )),
                                ..Default::default()
                            },
                        );
                    }
                    pen += (glyph.advance + layer.spacing) as f32;
                    if let Some(next) = characters.get(index + 1) {
                        pen += layer
                            .kerning
                            .get(&(*character, *next))
                            .copied()
                            .unwrap_or(0) as f32;
                    }
                }
            }
        }
    }
}

impl Presentation {
    async fn load(
        game_root: &Path,
        assets: &GameAssets,
        muted: bool,
    ) -> Result<Self, Box<dyn Error>> {
        let mut images = HashMap::new();
        for id in IMAGE_IDS {
            let image = assets.load_image(id)?;
            let width = u16::try_from(image.width)?;
            let height = u16::try_from(image.height)?;
            let texture = Texture2D::from_rgba8(width, height, &image.pixels);
            texture.set_filter(FilterMode::Linear);
            images.insert((*id).into(), texture);
        }
        let mut sounds = HashMap::new();
        if !muted {
            for id in SOUND_IDS {
                let bytes = match assets.load_sound(id)? {
                    SoundData::Ogg(bytes) => bytes,
                    SoundData::Pcm16 {
                        sample_rate,
                        samples,
                    } => pcm_wav(sample_rate, &samples),
                };
                let sound: Arc<[u8]> = bytes.into();
                rodio::Decoder::try_from(std::io::Cursor::new(sound.clone()))
                    .map_err(|error| format!("{id}: {error}"))?;
                sounds.insert((*id).into(), sound);
            }
        }
        let mut fonts = HashMap::new();
        for name in [
            "JungleFever10outline",
            "JungleFever17outline",
            "JungleFever15outline",
            "JungleFever12outline",
            "ContinuumBold12",
            "ContinuumBold14",
            "ContinuumBold14outback",
            "Pix118",
        ] {
            fonts.insert(name.into(), RenderedFont::load(game_root, assets, name)?);
        }
        let additive = load_material(
            ShaderSource::Glsl {
                vertex: ADDITIVE_VERTEX,
                fragment: ADDITIVE_FRAGMENT,
            },
            MaterialParams {
                pipeline_params: PipelineParams {
                    color_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Value(BlendValue::SourceAlpha),
                        BlendFactor::One,
                    )),
                    alpha_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Zero,
                        BlendFactor::One,
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .map_err(|error| format!("additive sprite material creation failed: {error}"))?;
        Ok(Self {
            images,
            sounds,
            fonts,
            additive,
        })
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Thin sprite draw helper keeps its source rectangle, transform and tint explicit"
    )]
    fn sprite(
        &self,
        id: &str,
        x: f32,
        y: f32,
        source: Option<Rect>,
        flip: bool,
        scale: f32,
        alpha: f32,
    ) {
        let texture = &self.images[id];
        let source = source.unwrap_or(Rect::new(0.0, 0.0, texture.width(), texture.height()));
        draw_texture_ex(
            texture,
            x,
            y,
            Color::new(1.0, 1.0, 1.0, alpha),
            DrawTextureParams {
                source: Some(source),
                dest_size: Some(vec2(source.w * scale, source.h * scale)),
                flip_x: flip,
                ..Default::default()
            },
        );
    }

    fn additive_sprite(&self, id: &str, x: f32, y: f32, source: Rect, flip: bool, alpha: f32) {
        self.additive_tinted_sprite(id, x, y, source, flip, Color::new(1.0, 1.0, 1.0, alpha));
    }

    fn additive_tinted_sprite(
        &self,
        id: &str,
        x: f32,
        y: f32,
        source: Rect,
        flip: bool,
        tint: Color,
    ) {
        gl_use_material(&self.additive);
        draw_texture_ex(
            &self.images[id],
            x,
            y,
            tint,
            DrawTextureParams {
                source: Some(source),
                flip_x: flip,
                ..Default::default()
            },
        );
        gl_use_default_material();
    }

    fn play(&self, events: &[Event], music: Option<&MusicOwner>) -> Vec<MusicReport> {
        let mut reports = Vec::new();
        for event in events {
            // Amp calls Die(false) for each victim, followed by one death
            // sound for the entire discharge (PB66/W1 HandleMouseDown).
            if let Event::FishDied { fish_id, .. } = event
                && events.iter().any(|other| {
                    matches!(other,
                    Event::AmpDischarged { victim_ids, .. } if victim_ids.contains(fish_id))
                })
            {
                continue;
            }
            let id = match event {
                Event::AmpReady { .. } => "SOUND_EEL2",
                Event::AmpCharged { .. } => "SOUND_EEL3",
                Event::AmpDischarged { .. } => "SOUND_EEL1",
                Event::FoodDropped { potion: false, .. } => "SOUND_DROPFOOD",
                Event::PotionExploded { .. } => "SOUND_EXPLOSION1",
                Event::FoodEaten { .. }
                | Event::BrinkleyFoodEaten { .. }
                | Event::BreederAteFood { .. }
                | Event::Invasion {
                    event: InvasionEvent::GusAteFood { .. },
                    ..
                } => "SOUND_SLURP",
                Event::FishGrew { .. } | Event::BreederGrew { .. } => "SOUND_GROW",
                Event::CoinCollectionStarted {
                    kind: CoinKind::Pearl,
                    ..
                } => "SOUND_PEARL",
                Event::CoinCollectionStarted {
                    kind: CoinKind::ShellPearl,
                    ..
                } => "SOUND_PEARL",
                Event::CoinCollectionStarted {
                    kind: CoinKind::Treasure,
                    ..
                } => "SOUND_TREASURE",
                Event::CoinCollectionStarted {
                    kind: CoinKind::ShellTreasure,
                    ..
                } => "SOUND_BONUSCOLLECT",
                Event::CoinCollectionStarted {
                    kind: CoinKind::ShellDiamond | CoinKind::ShellDiamondPenta,
                    ..
                } => "SOUND_DIAMOND",
                Event::PetCollectedCoin {
                    kind: CoinKind::Pearl | CoinKind::ShellPearl,
                    ..
                } => "SOUND_PEARL",
                Event::PetCollectedCoin {
                    kind: CoinKind::Treasure,
                    ..
                } => "SOUND_TREASURE",
                Event::PetCollectedCoin {
                    kind: CoinKind::ShellTreasure,
                    ..
                } => "SOUND_BONUSCOLLECT",
                Event::PetCollectedCoin {
                    kind: CoinKind::ShellDiamond | CoinKind::ShellDiamondPenta,
                    ..
                } => "SOUND_DIAMOND",
                Event::CoinCollectionStarted {
                    kind: CoinKind::ShrapnelBomb,
                    ..
                } => "SOUND_POINTS",
                Event::CoinCredited {
                    kind:
                        CoinKind::Pearl
                        | CoinKind::ShrapnelBomb
                        | CoinKind::ShellPearl
                        | CoinKind::Treasure
                        | CoinKind::ShellTreasure
                        | CoinKind::ShellDiamond
                        | CoinKind::ShellDiamondPenta,
                    ..
                } => continue,
                Event::CoinCredited { .. } | Event::VirtualCoinCollected { .. } => "SOUND_POINTS",
                Event::GuppyBought { .. }
                | Event::EggBought { .. }
                | Event::FoodQualityBought { .. }
                | Event::FoodQuantityBought { .. }
                | Event::PotionBought { .. }
                | Event::StarcatcherAteStar { .. }
                | Event::WeaponBought { .. }
                | Event::BonusPurchaseCommitted { .. } => "SOUND_BUY",
                Event::OscarBought { .. }
                | Event::StarcatcherBought { .. }
                | Event::GrubberBought { .. }
                | Event::GekkoBought { .. }
                | Event::UltraBought { .. } => "SOUND_GROW",
                Event::OscarAteGuppy { .. }
                | Event::GrubberAteGuppy { .. }
                | Event::GekkoAtePrey { .. }
                | Event::UltraAteOscar { .. }
                | Event::GashAteGuppy { .. } => "SOUND_CHOMP",
                Event::OscarDied { tick, oscar_id }
                    if death_has_missile_impact(events, *tick, *oscar_id) =>
                {
                    continue;
                }
                Event::StarcatcherDied {
                    tick,
                    starcatcher_id,
                } if death_has_missile_impact(events, *tick, *starcatcher_id) => {
                    continue;
                }
                Event::GrubberDied { tick, grubber_id }
                    if death_has_missile_impact(events, *tick, *grubber_id) =>
                {
                    continue;
                }
                Event::GekkoDied { tick, gekko_id }
                    if death_has_missile_impact(events, *tick, *gekko_id) =>
                {
                    continue;
                }
                Event::UltraDied { tick, id } if death_has_missile_impact(events, *tick, *id) => {
                    continue;
                }
                Event::BreederDied { sound: true, .. } => "SOUND_DIE",
                Event::OscarDied { .. }
                | Event::StarcatcherDied { .. }
                | Event::GrubberDied { .. }
                | Event::GekkoDied { .. }
                | Event::UltraDied { .. } => "SOUND_DIE",
                Event::LarvaPickupSound { .. } => "SOUND_POINTS",
                Event::FishPetHit { sound: true, .. } => "SOUND_PUNCH",
                Event::RufusHit { sound: true, .. } => "SOUND_PUNCH",
                Event::MissileLaunched { .. } => "SOUND_MISSLE",
                Event::MissileRemoved { .. } => "SOUND_EXPLODE",
                Event::MissileImpacted { .. } | Event::PetRemoved { .. } => "SOUND_DIE",
                Event::EnergyBallLaunched { first: true, .. } => "SOUND_UNLEASH",
                Event::EnergyBallLaunched { first: false, .. }
                | Event::EnergyBallShot { .. }
                | Event::EnergyBallAlienHit { .. } => continue,
                Event::EnergyBallRemoved { .. } => "SOUND_EXPLOSION4",
                Event::PregoBirth { .. } => "SOUND_BABY",
                Event::BreederBornGuppy { .. } => "SOUND_SFX",
                Event::MerylNoteDropped { .. } => "SOUND_SING",
                Event::ShrapnelBombDropped { .. } => "SOUND_UNLEASH",
                Event::ShrapnelBombExploded { .. } => "SOUND_EXPLODE",
                Event::BlipShopRevealed { .. } | Event::BlipSonar { .. } => "SOUND_SONAR",
                Event::HatchOpened { .. } => "SOUND_HATCH",
                Event::TankFourFinaleHatchOpened { .. } => "SOUND_EVILLAFF",
                Event::FishRevived { .. } => "SOUND_HEAL",
                Event::Invasion { event, .. } => match event {
                    InvasionEvent::WarningStarted(_) => "SOUND_AWOOGA",
                    InvasionEvent::AlienSpawned { .. } => "SOUND_ROAR",
                    InvasionEvent::AlienHit { .. } => "SOUND_HIT",
                    InvasionEvent::AlienDefeated { .. } => "SOUND_EXPLOSION1",
                    InvasionEvent::BossDefeated { .. } => "SOUND_EXPLODE",
                    InvasionEvent::MiniDefeated { .. } => "SOUND_EXPLOSION4",
                    InvasionEvent::LaserFired { .. } => "SOUND_ZAP",
                    InvasionEvent::PsychosquidPhaseChanged {
                        healing: false,
                        forced: false,
                        ..
                    } => "SOUND_ROAR3",
                    InvasionEvent::BilaterusSpawned { .. }
                    | InvasionEvent::BilaterusHeadSwapped { .. } => "SOUND_RATTLE",
                    InvasionEvent::BilaterusHeadHit { .. } => "SOUND_HIT",
                    InvasionEvent::BilaterusFirstHeadDefeated { .. }
                    | InvasionEvent::BilaterusDefeated { .. } => "SOUND_EXPLODE",
                    InvasionEvent::BilaterusPreyEaten { .. } => "SOUND_CHOMP",
                    _ => continue,
                },
                Event::Niko { event, .. } => match event {
                    crate::niko::NikoEvent::OpenSound => "SOUND_NIKOOPEN",
                    crate::niko::NikoEvent::CloseSound => "SOUND_NIKOCLOSE",
                    _ => continue,
                },
                Event::PearlCollectionStarted { .. } => "SOUND_PEARL",
                Event::Bonus {
                    event: crate::bonus::BonusEvent::Claimed { .. },
                    ..
                } => "SOUND_BONUSCOLLECT",
                Event::BonusResultsCommitted { .. } => "SOUND_BONUSCOUNT",
                Event::StageStarted { .. }
                | Event::TimeTrialStarted { .. }
                | Event::TimeTrialPetAcquired { .. }
                | Event::TimeTrialShellsCredited { .. }
                | Event::ChallengeStarted { .. }
                | Event::ChallengeEggBought { .. }
                | Event::ChallengeCompleted { .. }
                | Event::ChallengeShellsCredited { .. }
                | Event::RescueGuppyGranted { .. }
                | Event::PetSelectionChanged { .. }
                | Event::PetSelectionConfirmation { .. }
                | Event::BonusPurchaseOffered { .. } => "SOUND_BUTTONCLICK",
                Event::VirtualTankChanged { .. } => "SOUND_BUTTONCLICK",
                _ => continue,
            };
            for effect_id in std::iter::once(id)
                .chain(matches!(event, Event::EnergyBallRemoved { .. }).then_some("SOUND_EXPLODE"))
                .chain(
                    matches!(
                        event,
                        Event::Invasion {
                            event: InvasionEvent::BossDefeated { .. },
                            ..
                        }
                    )
                    .then_some("SOUND_EXPLOSION1"),
                )
                .chain(matches!(event, Event::UltraBought { .. }).then_some("SOUND_SPLASHBIG"))
                .chain(
                    matches!(event, Event::AmpDischarged { victim_ids, .. }
                    if !victim_ids.is_empty())
                    .then_some("SOUND_DIE"),
                )
                .chain(
                    matches!(
                        event,
                        Event::Invasion {
                            event: InvasionEvent::BilaterusFirstHeadDefeated { .. }
                                | InvasionEvent::BilaterusDefeated { .. },
                            ..
                        }
                    )
                    .then_some("SOUND_EXPLOSION1"),
                )
            {
                if let (Some(sound), Some(music)) = (self.sounds.get(effect_id), music)
                    && let Some(report) = music.play_effect(sound.clone())
                {
                    reports.push(report);
                }
            }
        }
        reports
    }

    fn draw_board(&self, state: &AdventureState) {
        self.sprite(
            match if state.sandbox_background > 0 {
                state.sandbox_background
            } else {
                match state.tank {
                    2 => 2,
                    3 => 4,
                    4 => 5,
                    5 => 6,
                    _ => 1,
                }
            } {
                2 => "IMAGE_AQUARIUM2",
                3 => "IMAGE_AQUARIUM3",
                4 => "IMAGE_AQUARIUM4",
                5 => "IMAGE_AQUARIUM5",
                6 => "IMAGE_AQUARIUM6",
                _ => "IMAGE_AQUARIUM1",
            },
            0.0,
            0.0,
            None,
            false,
            1.0,
            1.0,
        );
        for penta in &state.starcatchers {
            let source = Rect::new(
                f32::from(penta.sprite_frame()) * 80.0,
                f32::from(penta.sprite_row()) * 80.0,
                80.0,
                80.0,
            );
            self.sprite(
                "IMAGE_STARCATCHER",
                penta.widget_x as f32,
                penta.widget_y as f32,
                Some(source),
                false,
                1.0,
                1.0,
            );
            if penta.hunger_overlay_alpha() > 0.0 {
                self.sprite(
                    "IMAGE_STARCATCHER",
                    penta.widget_x as f32,
                    penta.widget_y as f32,
                    Some(Rect::new(source.x, 80.0, 80.0, 80.0)),
                    false,
                    1.0,
                    penta.hunger_overlay_alpha(),
                );
            }
        }
        for food in &state.food {
            // W1 Food.cpp 173-188 places the special pellet in row four at
            // its widget origin with 200/255 alpha. Primary draw binding
            // remains pending; the installed sheet has five rows.
            let exotic = food.food_type == FoodType::Nostradamus;
            self.sprite(
                "IMAGE_FOOD",
                if exotic { food.x.trunc() } else { food.x - 5.0 },
                if exotic { food.y.trunc() } else { food.y - 4.0 },
                Some(Rect::new(
                    (food.frame / food.animation_period % 10) as f32 * 40.0,
                    if exotic {
                        160.0
                    } else {
                        f32::from(food.quality) * 40.0
                    },
                    40.0,
                    40.0,
                )),
                false,
                1.0,
                if exotic { 200.0 / 255.0 } else { 1.0 },
            );
        }
        for fish in state.fish.iter().filter(|fish| {
            fish.alive
                && !(matches!(fish.size, FishSize::Small | FishSize::Medium)
                    && state.hides_fish_at(fish.x, fish.y))
        }) {
            let id = match (fish.sprite_pose(), fish.hunger_visible) {
                (FishPose::Swim, false) => "IMAGE_SMALLSWIM",
                (FishPose::Swim, true) => "IMAGE_HUNGRYSWIM",
                (FishPose::Eat, false) => "IMAGE_SMALLEAT",
                (FishPose::Eat, true) => "IMAGE_HUNGRYEAT",
                (FishPose::Turn, false) => "IMAGE_SMALLTURN",
                (FishPose::Turn, true) => "IMAGE_HUNGRYTURN",
            };
            let row = match fish.size {
                FishSize::Small => 0.0,
                FishSize::Medium => 1.0,
                FishSize::Large | FishSize::Star => 2.0,
                FishSize::Crowned => 3.0,
            };
            self.sprite(
                id,
                fish.x - (fish.growth_scale() - 1.0) * 40.0,
                fish.y - (fish.growth_scale() - 1.0) * 40.0,
                Some(Rect::new(
                    (fish.frame % 10) as f32 * 80.0,
                    row * 80.0,
                    80.0,
                    80.0,
                )),
                fish.facing_right,
                fish.growth_scale(),
                if fish.size == FishSize::Star {
                    155.0 / 255.0
                } else {
                    1.0
                },
            );
            if fish.size == FishSize::Star {
                // W1 adds a bright pass over a translucent large-fish pose.
                // Macroquad's standard blend is a visual approximation.
                self.sprite(
                    id,
                    fish.x,
                    fish.y,
                    Some(Rect::new(
                        (fish.frame % 10) as f32 * 80.0,
                        160.0,
                        80.0,
                        80.0,
                    )),
                    fish.facing_right,
                    1.0,
                    200.0 / 255.0,
                );
            }
            if state.blip_hunger_icon_visible(fish) {
                // Fish::Draw puts misc cel 2 at widget-local (0, -5).
                self.sprite(
                    "IMAGE_MISCITEMS",
                    fish.x,
                    fish.y - 5.0,
                    Some(Rect::new(144.0, 0.0, 72.0, 72.0)),
                    false,
                    1.0,
                    1.0,
                );
            }
        }
        for breeder in &state.breeders {
            let image = if breeder.hunger_visible {
                "IMAGE_HUNGRYBREEDER"
            } else {
                "IMAGE_BREEDER"
            };
            let frame_x = f32::from(breeder.sprite_frame()) * 80.0;
            // W1 Breeder::DrawBreeder truncates the growth offset to an int,
            // then expands both sides of the 80px destination rectangle.
            let inset = ((breeder.growth_scale() - 1.0) * 80.0) as i32;
            let x = (breeder.widget_x - inset) as f32;
            let y = (breeder.widget_y - inset) as f32;
            let scale = (80 + 2 * inset) as f32 / 80.0;
            self.sprite(
                image,
                x,
                y,
                Some(Rect::new(
                    frame_x,
                    f32::from(breeder.sprite_row()) * 80.0,
                    80.0,
                    80.0,
                )),
                breeder.facing_right(),
                scale,
                1.0,
            );
            if breeder.hunger_overlay_alpha() > 0 {
                self.sprite(
                    "IMAGE_HUNGRYBREEDER",
                    x,
                    y,
                    Some(Rect::new(
                        frame_x,
                        f32::from(breeder.hungry_sprite_row()) * 80.0,
                        80.0,
                        80.0,
                    )),
                    breeder.facing_right(),
                    scale,
                    f32::from(breeder.hunger_overlay_alpha()) / 255.0,
                );
            }
            if state.blip_breeder_icon_visible(breeder) {
                self.sprite(
                    "IMAGE_MISCITEMS",
                    breeder.widget_x as f32,
                    breeder.widget_y as f32,
                    Some(Rect::new(144.0, 0.0, 72.0, 72.0)),
                    false,
                    1.0,
                    1.0,
                );
            }
        }
        for oscar in state.oscars.iter().filter(|oscar| oscar.alive) {
            let id = match (oscar.sprite_pose(), oscar.hunger_visible()) {
                (OscarPose::Swim, false) => "IMAGE_SMALLSWIM",
                (OscarPose::Swim, true) => "IMAGE_HUNGRYSWIM",
                (OscarPose::Eat, false) => "IMAGE_SMALLEAT",
                (OscarPose::Eat, true) => "IMAGE_HUNGRYEAT",
                (OscarPose::Turn, false) => "IMAGE_SMALLTURN",
                (OscarPose::Turn, true) => "IMAGE_HUNGRYTURN",
            };
            self.sprite(
                id,
                oscar.widget_x as f32,
                oscar.widget_y as f32,
                Some(Rect::new(f32::from(oscar.frame) * 80.0, 320.0, 80.0, 80.0)),
                oscar.facing_right(),
                1.0,
                1.0,
            );
            if oscar.hunger_overlay_alpha() > 0.0 {
                let hungry_id = match oscar.sprite_pose() {
                    OscarPose::Swim => "IMAGE_HUNGRYSWIM",
                    OscarPose::Eat => "IMAGE_HUNGRYEAT",
                    OscarPose::Turn => "IMAGE_HUNGRYTURN",
                };
                self.sprite(
                    hungry_id,
                    oscar.widget_x as f32,
                    oscar.widget_y as f32,
                    Some(Rect::new(f32::from(oscar.frame) * 80.0, 320.0, 80.0, 80.0)),
                    oscar.facing_right(),
                    1.0,
                    oscar.hunger_overlay_alpha(),
                );
            }
        }
        for ultra in state.ultras.iter().filter(|ultra| ultra.alive) {
            let row = match ultra.sprite_pose() {
                UltraPose::Swim => 0.0,
                UltraPose::Eat => 1.0,
                UltraPose::Turn => 2.0,
            };
            let source = Rect::new(f32::from(ultra.frame) * 160.0, row * 160.0, 160.0, 160.0);
            let hungry_tint = Color::from_rgba(250, 215, 95, 255);
            draw_texture_ex(
                &self.images["IMAGE_ULTRA"],
                ultra.widget_x as f32,
                ultra.widget_y as f32,
                if ultra.hunger_visible() {
                    hungry_tint
                } else {
                    WHITE
                },
                DrawTextureParams {
                    source: Some(source),
                    flip_x: ultra.facing_right(),
                    ..Default::default()
                },
            );
            if !ultra.hunger_visible() && ultra.hunger_overlay_alpha() > 0.0 {
                draw_texture_ex(
                    &self.images["IMAGE_ULTRA"],
                    ultra.widget_x as f32,
                    ultra.widget_y as f32,
                    Color::new(
                        250.0 / 255.0,
                        215.0 / 255.0,
                        95.0 / 255.0,
                        ultra.hunger_overlay_alpha(),
                    ),
                    DrawTextureParams {
                        source: Some(source),
                        flip_x: ultra.facing_right(),
                        ..Default::default()
                    },
                );
            }
            if state.blip_ultra_icon_visible(ultra) {
                self.sprite(
                    "IMAGE_MISCITEMS",
                    ultra.widget_x as f32 + 40.0,
                    ultra.widget_y as f32,
                    Some(Rect::new(144.0, 0.0, 72.0, 72.0)),
                    false,
                    1.0,
                    1.0,
                );
            }
        }
        for grubber in &state.grubbers {
            let frame_x = f32::from(grubber.sprite_frame()) * 80.0;
            self.sprite(
                "IMAGE_GRUBBER",
                grubber.widget_x as f32,
                grubber.widget_y as f32,
                Some(Rect::new(
                    frame_x,
                    f32::from(grubber.sprite_row()) * 80.0,
                    80.0,
                    80.0,
                )),
                false,
                1.0,
                1.0,
            );
            let hunger_alpha = f32::from(grubber.hunger_overlay_alpha()) / 255.0;
            if hunger_alpha > 0.0 {
                self.sprite(
                    "IMAGE_GRUBBER",
                    grubber.widget_x as f32,
                    grubber.widget_y as f32,
                    Some(Rect::new(frame_x, 160.0, 80.0, 80.0)),
                    false,
                    1.0,
                    hunger_alpha,
                );
            }
        }
        for gekko in &state.gekkos {
            let frame_x = f32::from(gekko.sprite_frame()) * 80.0;
            self.sprite(
                "IMAGE_GEKKO",
                gekko.widget_x as f32,
                gekko.widget_y as f32,
                Some(Rect::new(
                    frame_x,
                    f32::from(gekko.sprite_row()) * 80.0,
                    80.0,
                    80.0,
                )),
                gekko.facing_right(),
                1.0,
                1.0,
            );
            let hunger_alpha = f32::from(gekko.hunger_overlay_alpha()) / 255.0;
            if hunger_alpha > 0.0 {
                self.sprite(
                    "IMAGE_GEKKO",
                    gekko.widget_x as f32,
                    gekko.widget_y as f32,
                    Some(Rect::new(
                        frame_x,
                        f32::from(gekko.hungry_sprite_row()) * 80.0,
                        80.0,
                        80.0,
                    )),
                    gekko.facing_right(),
                    1.0,
                    hunger_alpha,
                );
            }
        }
        let aliens_present = state
            .invasion
            .as_ref()
            .is_some_and(|wave| wave.has_live_alien());
        for pet in &state.fish_pets {
            let image = match pet.kind {
                FishPetKind::Itchy => "IMAGE_ITCHY",
                FishPetKind::Prego => "IMAGE_PREGO",
                FishPetKind::Zorf => "IMAGE_ZORF",
                FishPetKind::Vert => "IMAGE_VERT",
                FishPetKind::Meryl => "IMAGE_MERYL",
                FishPetKind::Wadsworth => "IMAGE_WADSWORTH",
                FishPetKind::Seymour => "IMAGE_SEYMOUR",
                FishPetKind::Shrapnel => "IMAGE_SHRAPNEL",
                FishPetKind::Gumbo => "IMAGE_GUMBO",
                FishPetKind::Blip => "IMAGE_BLIP",
                FishPetKind::Nimbus => "IMAGE_NIMBUS",
                FishPetKind::Amp => "IMAGE_AMP",
                FishPetKind::Gash => "IMAGE_GASH",
                FishPetKind::Angie => "IMAGE_ANGIE",
                FishPetKind::Presto => "IMAGE_PRESTO",
                FishPetKind::Brinkley => "IMAGE_BRINKLEY",
                FishPetKind::Nostradamus => "IMAGE_NOSTRADAMUS",
                FishPetKind::Stanley => "IMAGE_STANLEY",
                FishPetKind::Walter => "IMAGE_WALTER",
            };
            let (cell_width, cell_height) = if pet.kind == FishPetKind::Amp {
                (160.0, 60.0)
            } else {
                (80.0, 80.0)
            };
            let source = Rect::new(
                f32::from(pet.sprite_frame()) * cell_width,
                f32::from(pet.sprite_row(aliens_present)) * cell_height,
                cell_width,
                cell_height,
            );
            self.sprite(
                image,
                pet.widget_x as f32,
                pet.widget_y as f32,
                Some(source),
                pet.facing_right(),
                1.0,
                1.0,
            );
            if pet.kind == FishPetKind::Walter
                && let Some(glove) = pet.glove
            {
                // IMAGE_WALTER is the installed 10x3, 80px-cell atlas.
                // Row 2/column 5 is its Glove artwork. Its pose comes
                // from the physical parent's persisted child state.
                // Retail Draw ordering and effects remain unverified.
                self.sprite(
                    "IMAGE_WALTER",
                    (glove.x + 40) as f32,
                    glove.y as f32,
                    Some(Rect::new(400.0, 160.0, 80.0, 80.0)),
                    glove.right,
                    1.0,
                    1.0,
                );
            }
            if pet.kind == FishPetKind::Angie {
                self.additive_tinted_sprite(
                    "IMAGE_HALO",
                    pet.widget_x as f32,
                    pet.widget_y as f32,
                    source,
                    pet.facing_right(),
                    Color::from_rgba(255, 255, 255, (pet.glint_phase.abs() * 255.0) as u8),
                );
            }
            if pet.kind == FishPetKind::Amp
                && (pet.amp_ready() || (pet.amp_timer < 0 && !aliens_present))
            {
                let (green, blue) = match pet.amp_charge {
                    0 => (255, 200),
                    1 => (200, 100),
                    _ => (100, 100),
                };
                let alpha = (pet.glint_phase.abs() * 2.0 * 255.0).min(255.0) as u8;
                self.additive_tinted_sprite(
                    "IMAGE_AMPCHARGE",
                    pet.widget_x as f32,
                    pet.widget_y as f32,
                    source,
                    pet.facing_right(),
                    Color::from_rgba(255, green, blue, alpha),
                );
            }
            if pet.kind == FishPetKind::Shrapnel && pet.shrapnel_flash_alpha() > 0 {
                self.additive_sprite(
                    image,
                    pet.widget_x as f32,
                    pet.widget_y as f32,
                    source,
                    pet.facing_right(),
                    f32::from(pet.shrapnel_flash_alpha()) / 255.0,
                );
            }
            if pet.kind == FishPetKind::Gumbo && aliens_present && pet.gumbo_light_alpha() > 0 {
                self.additive_tinted_sprite(
                    "IMAGE_GUMBOLIGHT",
                    pet.widget_x as f32,
                    pet.widget_y as f32,
                    source,
                    pet.facing_right(),
                    Color::from_rgba(255, 255, 0, pet.gumbo_light_alpha()),
                );
            }
            if pet.kind == FishPetKind::Wadsworth
                && !pet.ward_active
                && (aliens_present || !state.missiles.is_empty())
            {
                self.sprite(
                    "IMAGE_ZZZ",
                    pet.widget_x as f32 + 40.0,
                    pet.widget_y as f32 - 15.0,
                    None,
                    false,
                    1.0,
                    1.0,
                );
            }
        }
        for fish in &state.dead_fish {
            let row = match fish.size {
                FishSize::Small => 0.0,
                FishSize::Medium => 1.0,
                FishSize::Large | FishSize::Star => 2.0,
                FishSize::Crowned => 3.0,
            };
            self.sprite(
                "IMAGE_SMALLDIE",
                fish.x,
                fish.y,
                Some(Rect::new(
                    f32::from(fish.frame % 10) * 80.0,
                    row * 80.0,
                    80.0,
                    80.0,
                )),
                fish.facing_right,
                1.0,
                fish.opacity,
            );
        }
        for corpse in &state.dead_breeders {
            self.sprite(
                "IMAGE_HUNGRYBREEDER",
                corpse.widget_x as f32,
                corpse.widget_y as f32,
                Some(Rect::new(
                    f32::from(corpse.frame) * 80.0,
                    f32::from(corpse.sprite_row()) * 80.0,
                    80.0,
                    80.0,
                )),
                corpse.facing_right,
                1.0,
                corpse.opacity,
            );
        }
        for corpse in &state.dead_oscars {
            self.sprite(
                "IMAGE_SMALLDIE",
                corpse.widget_x as f32,
                corpse.widget_y as f32,
                Some(Rect::new(f32::from(corpse.frame) * 80.0, 320.0, 80.0, 80.0)),
                corpse.facing_right,
                1.0,
                corpse.opacity,
            );
        }
        for corpse in &state.dead_ultras {
            let source = Rect::new(f32::from(corpse.frame) * 160.0, 480.0, 160.0, 160.0);
            // W1 DeadFish::Draw warms the first five Ultra death cels, then
            // uses the ordinary corpse fade after the 90-count boundary.
            let tint = if corpse.remaining_ticks >= 90 && corpse.frame < 5 {
                Color::from_rgba(
                    250 + corpse.frame,
                    215 + corpse.frame * 8,
                    95 + corpse.frame * 32,
                    255,
                )
            } else {
                Color::new(1.0, 1.0, 1.0, corpse.opacity)
            };
            draw_texture_ex(
                &self.images["IMAGE_ULTRA"],
                corpse.widget_x as f32,
                (corpse.widget_y + i32::from((90_u16.saturating_sub(corpse.remaining_ticks)) / 2))
                    as f32,
                tint,
                DrawTextureParams {
                    source: Some(source),
                    flip_x: corpse.facing_right,
                    ..Default::default()
                },
            );
        }
        for corpse in &state.dead_starcatchers {
            self.sprite(
                "IMAGE_STARCATCHER",
                corpse.widget_x as f32,
                corpse.widget_y as f32,
                Some(Rect::new(f32::from(corpse.frame) * 80.0, 160.0, 80.0, 80.0)),
                false,
                1.0,
                corpse.opacity,
            );
        }
        for corpse in &state.dead_grubbers {
            self.sprite(
                "IMAGE_GRUBBER",
                corpse.widget_x as f32,
                corpse.widget_y as f32,
                Some(Rect::new(
                    f32::from(corpse.sprite_frame()) * 80.0,
                    f32::from(corpse.sprite_row()) * 80.0,
                    80.0,
                    80.0,
                )),
                false,
                1.0,
                corpse.opacity,
            );
        }
        for corpse in &state.dead_gekkos {
            self.sprite(
                "IMAGE_GEKKO",
                corpse.widget_x as f32,
                corpse.widget_y as f32,
                Some(Rect::new(
                    f32::from(corpse.sprite_frame()) * 80.0,
                    f32::from(corpse.sprite_row()) * 80.0,
                    80.0,
                    80.0,
                )),
                corpse.facing_right,
                1.0,
                corpse.opacity,
            );
        }
        if let Some(wave) = &state.invasion {
            for warp in &wave.warps {
                // WinFish Warp::Draw computes 17 - counter/2, which can
                // exceed both 17-column sheets at its endpoints. The
                // installed renderer's handling is not yet established.
                let frame = 17_i32 - i32::from(warp.remaining_ticks) / 2;
                if (0..17).contains(&frame) {
                    self.sprite(
                        "IMAGE_WARPHOLE",
                        (warp.x + 20) as f32,
                        warp.y as f32,
                        Some(Rect::new(frame as f32 * 60.0, 0.0, 60.0, 220.0)),
                        false,
                        1.0,
                        1.0,
                    );
                    self.sprite(
                        "IMAGE_WARPGLOW",
                        warp.x as f32,
                        warp.y as f32,
                        Some(Rect::new(frame as f32 * 100.0, 0.0, 100.0, 220.0)),
                        false,
                        1.0,
                        0.7,
                    );
                }
            }
            for alien in &wave.actors {
                if alien.spawn_ticks > 9 {
                    continue;
                }
                let inset = if alien.spawn_ticks > 0 {
                    (f32::from(alien.spawn_ticks) / 10.0 * 160.0) as i32
                } else {
                    0
                };
                let size = 160 - inset;
                if size > 0 {
                    let source = Rect::new(
                        f32::from(alien.sprite_frame()) * 160.0,
                        f32::from(alien.sprite_row()) * 160.0,
                        160.0,
                        160.0,
                    );
                    let x = alien.widget_x as f32 + (inset / 2) as f32;
                    let y = alien.widget_y as f32 + (inset / 2) as f32;
                    let scale = size as f32 / 160.0;
                    let alien_image = match alien.kind {
                        SylvesterKind::Balrog => "IMAGE_BALROG",
                        SylvesterKind::Gus => "IMAGE_GUS",
                        SylvesterKind::Destructor => "IMAGE_DESTRUCTOR",
                        SylvesterKind::Ulysses => "IMAGE_ULYSSES",
                        SylvesterKind::Psychosquid => "IMAGE_PSYCHOSQUID",
                        SylvesterKind::Cyrax => "IMAGE_BOSS",
                        SylvesterKind::MiniSylvester => "IMAGE_MINISYLV",
                        SylvesterKind::Weak | SylvesterKind::Strong => "IMAGE_SYLV",
                    };
                    // Gus's eating row uses the velocity facing even when a
                    // turn is in progress (W1 Alien::DrawAlien).
                    let facing_right = if alien.kind == SylvesterKind::Gus && alien.hit_flash() {
                        alien.vx >= 0.0
                    } else {
                        alien.facing_right()
                    };
                    self.sprite(alien_image, x, y, Some(source), facing_right, scale, 1.0);
                    if alien.kind != SylvesterKind::Gus
                        && !(alien.kind == SylvesterKind::Psychosquid && alien.healing)
                        && alien.hit_flash()
                        && alien.spawn_ticks == 0
                    {
                        self.sprite(
                            alien_image,
                            x,
                            y,
                            Some(source),
                            facing_right,
                            scale,
                            (f32::from(alien.hit_ticks) * 25.0 / 255.0).min(1.0),
                        );
                    } else if alien.kind == SylvesterKind::Destructor
                        && (1..10).contains(&alien.special_ticks)
                        && alien.spawn_ticks == 0
                    {
                        let intensity = alien.special_ticks.min(10 - alien.special_ticks);
                        // W1 uses an additive pass; standard alpha is a bounded
                        // macroquad presentation approximation.
                        self.sprite(
                            alien_image,
                            x,
                            y,
                            Some(source),
                            facing_right,
                            scale,
                            f32::from(intensity) / 5.0,
                        );
                    }
                    if alien.spawn_ticks == 0 && state.has_live_blip() && state.tank != 5 {
                        // W1 Alien::DrawAlien clips only the bar; the tube keeps
                        // its full image width. This ratio is source-derived.
                        let bar = &self.images["IMAGE_HEALTHBAR"];
                        let width = health_bar_visible_width(
                            alien.health,
                            alien.kind.starting_health(),
                            bar.width(),
                        );
                        if width > 0.0 {
                            self.sprite(
                                "IMAGE_HEALTHBAR",
                                alien.widget_x as f32 + 10.0,
                                alien.widget_y as f32 + 157.0,
                                Some(Rect::new(0.0, 0.0, width, bar.height())),
                                false,
                                1.0,
                                1.0,
                            );
                        }
                        self.sprite(
                            "IMAGE_HEALTHBARTUBE",
                            alien.widget_x as f32 + 10.0,
                            alien.widget_y as f32 + 157.0,
                            None,
                            false,
                            1.0,
                            1.0,
                        );
                    }
                }
            }
            if let Some(finale) = &wave.finale {
                for alien in finale.boss.iter().chain(&finale.children) {
                    if alien.spawn_ticks > 9 {
                        continue;
                    }
                    let size = alien.sprite_size() as f32;
                    let inset = f32::from(alien.spawn_ticks) / 10.0 * size;
                    let source = Rect::new(
                        f32::from(alien.sprite_frame()) * size,
                        f32::from(alien.sprite_row()) * size,
                        size,
                        size,
                    );
                    let x = alien.widget_x as f32 + inset / 2.0;
                    let y = alien.widget_y as f32 + inset / 2.0;
                    let image = if alien.kind == SylvesterKind::Cyrax {
                        "IMAGE_BOSS"
                    } else {
                        "IMAGE_MINISYLV"
                    };
                    let facing_right = alien.facing_right();
                    self.sprite(
                        image,
                        x,
                        y,
                        Some(source),
                        facing_right,
                        (size - inset) / size,
                        1.0,
                    );
                    if alien.kind == SylvesterKind::Cyrax
                        && alien.hit_flash()
                        && !alien.healing
                        && alien.spawn_ticks == 0
                    {
                        self.additive_sprite(
                            image,
                            x,
                            y,
                            source,
                            facing_right,
                            (f32::from(alien.hit_ticks) * 25.0 / 255.0).min(1.0),
                        );
                    }
                    if alien.kind == SylvesterKind::Cyrax && alien.spawn_ticks == 0 {
                        let bar = &self.images["IMAGE_HEALTHBAR"];
                        let starting_health = f64::from(
                            5000 - 125 * finale.profile_attempts.saturating_sub(1).min(20),
                        );
                        let width =
                            health_bar_visible_width(alien.health, starting_health, bar.width());
                        let bar_x = alien.widget_x as f32 + 10.0;
                        let bar_y = alien.widget_y as f32 + size - 3.0;
                        if width > 0.0 {
                            self.sprite(
                                "IMAGE_HEALTHBAR",
                                bar_x,
                                bar_y,
                                Some(Rect::new(0.0, 0.0, width, bar.height())),
                                false,
                                1.0,
                                1.0,
                            );
                        }
                        self.sprite("IMAGE_HEALTHBARTUBE", bar_x, bar_y, None, false, 1.0, 1.0);
                    }
                }
            }
            for group in &wave.bilaterus {
                // W1 Bilaterus::OrderInManagerChanged places the passive head
                // behind bones 5..0 and the active head in front.
                let draw_head = |index: usize| {
                    let (Some(head), Some(scale), Some((row, frame, facing_right))) = (
                        group.heads[index].as_ref(),
                        group.head_emergence_scale(index),
                        group.head_sprite_pose(index),
                    ) else {
                        return;
                    };
                    let inset = (80.0 * (1.0 - scale) / 2.0) as i32;
                    let x = (head.widget_x + inset) as f32;
                    let y = (head.widget_y + inset) as f32;
                    let source =
                        Rect::new(f32::from(frame) * 80.0, f32::from(row) * 80.0, 80.0, 80.0);
                    self.sprite(
                        "IMAGE_BILATERUS",
                        x,
                        y,
                        Some(source),
                        facing_right,
                        scale,
                        1.0,
                    );
                    if head.hit_ticks > 0 && group.emergence_ticks == 0 {
                        self.additive_sprite(
                            "IMAGE_BILATERUS",
                            x,
                            y,
                            source,
                            facing_right,
                            (f32::from(head.hit_ticks) * 25.0 / 255.0).min(1.0),
                        );
                    }
                    if index == group.active_head
                        && group.emergence_ticks == 0
                        && state.has_live_blip()
                    {
                        let bar = &self.images["IMAGE_HEALTHBAR"];
                        let width = health_bar_visible_width(head.health, 100.0, bar.width());
                        if width > 0.0 {
                            self.sprite(
                                "IMAGE_HEALTHBAR",
                                head.widget_x as f32 - 20.0,
                                head.widget_y as f32 + 77.0,
                                Some(Rect::new(0.0, 0.0, width, bar.height())),
                                false,
                                1.0,
                                1.0,
                            );
                        }
                        self.sprite(
                            "IMAGE_HEALTHBARTUBE",
                            head.widget_x as f32 - 20.0,
                            head.widget_y as f32 + 77.0,
                            None,
                            false,
                            1.0,
                            1.0,
                        );
                    }
                };
                draw_head(1 - group.active_head);
                if group.emergence_ticks == 0 {
                    for index in (0..group.bones.len()).rev() {
                        let bone = &group.bones[index];
                        let predecessor_x = if index == 0 {
                            group.active().x
                        } else {
                            group.bones[index - 1].x
                        };
                        self.sprite(
                            "IMAGE_BILATERUS",
                            bone.widget_x as f32,
                            bone.widget_y as f32,
                            Some(Rect::new(
                                f32::from(bone.sprite_frame(predecessor_x)) * 80.0,
                                f32::from(crate::bilaterus::BilaterusBone::sprite_row(index))
                                    * 80.0,
                                80.0,
                                80.0,
                            )),
                            false,
                            1.0,
                            1.0,
                        );
                    }
                }
                draw_head(group.active_head);
            }
            for fragment in &wave.fragments {
                self.sprite(
                    "IMAGE_BILATERUS",
                    fragment.widget_x as f32,
                    fragment.widget_y as f32,
                    Some(Rect::new(
                        f32::from(fragment.sprite_frame()) * 80.0,
                        f32::from(fragment.sprite_row()) * 80.0,
                        80.0,
                        80.0,
                    )),
                    fragment.kind != crate::bilaterus::FragmentKind::Bone
                        && fragment.facing_right(),
                    1.0,
                    1.0,
                );
            }
            for missile in &state.missiles {
                match missile.kind {
                    MissileKind::Classic => self.sprite(
                        "IMAGE_MISSILE",
                        missile.widget_x as f32,
                        missile.widget_y as f32,
                        Some(Rect::new(f32::from(missile.frame) * 80.0, 0.0, 80.0, 80.0)),
                        false,
                        1.0,
                        1.0,
                    ),
                    MissileKind::EnergyBall => {
                        let x = missile.widget_x as f32;
                        let y = missile.widget_y as f32;
                        self.additive_tinted_sprite(
                            "IMAGE_ENERGYBALL",
                            x,
                            y,
                            Rect::new(f32::from(missile.frame) * 80.0, 0.0, 80.0, 80.0),
                            false,
                            if missile.reflected {
                                Color::from_rgba(175, 175, 50, 255)
                            } else {
                                Color::from_rgba(100, 100, 255, 55)
                            },
                        );
                        self.additive_tinted_sprite(
                            "IMAGE_ENERGYBALL",
                            x,
                            y,
                            Rect::new(400.0, 0.0, 80.0, 80.0),
                            false,
                            Color::from_rgba(100, 100, 255, 255),
                        );
                    }
                    MissileKind::Stanley => self.sprite(
                        "IMAGE_MISSILE",
                        missile.widget_x as f32,
                        missile.widget_y as f32,
                        Some(Rect::new(f32::from(missile.frame) * 80.0, 0.0, 80.0, 80.0)),
                        false,
                        0.625,
                        1.0,
                    ),
                }
            }
            for laser in &wave.lasers {
                let frame = laser.frame();
                if frame < 10 {
                    for row in 0..2 {
                        self.sprite(
                            "IMAGE_LASERS",
                            laser.x as f32,
                            laser.y as f32,
                            Some(Rect::new(
                                f32::from(frame) * 80.0,
                                row as f32 * 80.0,
                                80.0,
                                80.0,
                            )),
                            false,
                            1.0,
                            1.0,
                        );
                    }
                }
            }
            for body in wave
                .dead_aliens
                .iter()
                .filter(|body| body.kind != SylvesterKind::Gus)
            {
                self.sprite(
                    match body.kind {
                        SylvesterKind::Balrog => "IMAGE_BALROG",
                        SylvesterKind::Destructor => "IMAGE_DESTRUCTOR",
                        SylvesterKind::Ulysses => "IMAGE_ULYSSES",
                        SylvesterKind::Psychosquid => "IMAGE_PSYCHOSQUID",
                        SylvesterKind::Cyrax => "IMAGE_BOSS",
                        SylvesterKind::MiniSylvester => "IMAGE_MINISYLV",
                        SylvesterKind::Weak | SylvesterKind::Strong | SylvesterKind::Gus => {
                            "IMAGE_SYLV"
                        }
                    },
                    body.widget_x as f32,
                    body.widget_y as f32,
                    Some(Rect::new(f32::from(body.frame) * 160.0, 0.0, 160.0, 160.0)),
                    body.facing_right,
                    1.0,
                    body.opacity,
                );
            }
        }
        for stinky in &state.stinky {
            self.sprite(
                "IMAGE_STINKY",
                stinky.x as f32,
                stinky.y as f32,
                Some(Rect::new(
                    f32::from(stinky.frame) * 80.0,
                    f32::from(stinky.sprite_row()) * 80.0,
                    80.0,
                    80.0,
                )),
                stinky.facing_right(),
                1.0,
                1.0,
            );
        }
        for clyde in &state.clyde {
            self.sprite(
                "IMAGE_CLYDE",
                clyde.widget_x as f32,
                clyde.widget_y as f32,
                Some(Rect::new(f32::from(clyde.frame) * 80.0, 0.0, 80.0, 80.0)),
                false,
                1.0,
                1.0,
            );
        }
        for rufus in &state.rufus {
            self.sprite(
                "IMAGE_RUFUS",
                rufus.widget_x as f32,
                rufus.widget_y as f32,
                Some(Rect::new(
                    f32::from(rufus.sprite_frame()) * 80.0,
                    f32::from(rufus.sprite_row()) * 80.0,
                    80.0,
                    80.0,
                )),
                false,
                1.0,
                1.0,
            );
        }
        for rhubarb in &state.rhubarb {
            self.sprite(
                "IMAGE_RHUBARB",
                rhubarb.widget_x as f32,
                rhubarb.widget_y as f32,
                Some(Rect::new(
                    f32::from(rhubarb.sprite_frame()) * 80.0,
                    f32::from(rhubarb.sprite_row()) * 80.0,
                    80.0,
                    80.0,
                )),
                false,
                1.0,
                1.0,
            );
        }
        for niko in &state.niko {
            let (column, row) = niko.frame();
            self.sprite(
                "IMAGE_NIKO",
                niko.anchor_x as f32,
                niko.anchor_y as f32,
                Some(Rect::new(
                    f32::from(column) * 80.0,
                    f32::from(row) * 80.0,
                    80.0,
                    80.0,
                )),
                false,
                1.0,
                1.0,
            );
        }
        for pearl in &state.pearls {
            if pearl.should_draw_separately() {
                self.sprite(
                    "IMAGE_PEARL",
                    pearl.widget_x as f32,
                    pearl.widget_y as f32,
                    None,
                    false,
                    1.0,
                    1.0,
                );
            }
        }
        for coin in &state.coins {
            let alpha = if coin.fade_ticks > 0 {
                f32::from(coin.fade_ticks) / 5.0
            } else {
                1.0
            };
            let row = match coin.kind {
                CoinKind::Silver => 0.0,
                CoinKind::Gold => 1.0,
                CoinKind::Diamond | CoinKind::DiamondPenta => 3.0,
                CoinKind::Star | CoinKind::ShellStar => 2.0,
                CoinKind::Treasure => 4.0,
                CoinKind::ShellSilver
                | CoinKind::ShellGold
                | CoinKind::ShellDiamond
                | CoinKind::ShellDiamondPenta
                | CoinKind::ShellPearl => {
                    let shell_row = match coin.kind {
                        CoinKind::ShellSilver => 0.0,
                        CoinKind::ShellGold => 1.0,
                        CoinKind::ShellDiamond | CoinKind::ShellDiamondPenta => 2.0,
                        CoinKind::ShellPearl => 3.0,
                        _ => unreachable!(),
                    };
                    self.sprite(
                        "IMAGE_SHELLS",
                        coin.x as f32 + 20.0,
                        coin.y as f32 + 20.0,
                        Some(Rect::new(
                            f32::from(coin.frame % 20) * 32.0,
                            shell_row * 32.0,
                            32.0,
                            32.0,
                        )),
                        false,
                        1.0,
                        alpha,
                    );
                    continue;
                }
                CoinKind::ShellTreasure => {
                    self.sprite(
                        "IMAGE_MONEYBAG",
                        coin.x as f32 + 18.0,
                        coin.y as f32 + 18.0,
                        None,
                        false,
                        1.0,
                        alpha,
                    );
                    continue;
                }
                CoinKind::Pearl => {
                    self.sprite(
                        "IMAGE_PEARL",
                        coin.x as f32,
                        coin.y as f32,
                        None,
                        false,
                        1.0,
                        alpha,
                    );
                    continue;
                }
                CoinKind::ShrapnelBomb => {
                    self.sprite(
                        "IMAGE_MISCITEMS",
                        coin.x as f32,
                        coin.y as f32,
                        Some(Rect::new(0.0, 0.0, 72.0, 72.0)),
                        false,
                        1.0,
                        alpha,
                    );
                    // W1 draws the opaque RGB SPARKS sheet additively.
                    self.additive_sprite(
                        "IMAGE_SPARKS",
                        coin.x as f32 + 10.0,
                        coin.y as f32 - 10.0,
                        Rect::new(f32::from(coin.frame) * 40.0, 0.0, 40.0, 40.0),
                        false,
                        alpha,
                    );
                    continue;
                }
            };
            self.sprite(
                "IMAGE_MONEY",
                coin.x as f32,
                coin.y as f32,
                Some(Rect::new(
                    f32::from(coin.frame % 10) * 72.0,
                    row * 72.0,
                    72.0,
                    72.0,
                )),
                false,
                1.0,
                alpha,
            );
        }
        for shot in &state.bomb_shots {
            let Some(frame) = shot.sprite_frame() else {
                continue;
            };
            let (image, cell) = match shot.shot_type {
                3 => ("IMAGE_EXPLOSION", 80.0),
                4 => ("IMAGE_EXPLOSIONSMALL", 60.0),
                5 => ("IMAGE_EXPLOSIONTINY", 40.0),
                _ => continue,
            };
            let source = Rect::new(f32::from(frame) * cell, 0.0, cell, cell);
            let alpha = f32::from(shot.alpha) / 255.0;
            if shot.shot_type == 5 {
                self.sprite(
                    image,
                    shot.x as f32,
                    shot.y as f32,
                    Some(source),
                    false,
                    1.0,
                    alpha,
                );
            } else {
                self.additive_sprite(image, shot.x as f32, shot.y as f32, source, false, alpha);
            }
        }
        for larva in &state.larvae {
            self.sprite(
                "IMAGE_MONEY",
                larva.widget_x as f32,
                larva.widget_y as f32,
                Some(Rect::new(f32::from(larva.frame) * 72.0, 360.0, 72.0, 72.0)),
                false,
                1.0,
                1.0,
            );
        }
        for note in &state.notes {
            self.sprite(
                "IMAGE_MISCITEMS",
                note.x as f32,
                note.y as f32,
                Some(Rect::new(72.0, 0.0, 72.0, 72.0)),
                false,
                1.0,
                1.0,
            );
        }
        if let Some((x, y, frame)) = state.blip_crosshair() {
            // W1 Board::DrawOverlay0 follows game-object drawing and uses
            // additive blending. Board supplies the final draw position.
            self.additive_sprite(
                "IMAGE_CROSSHAIR",
                x as f32,
                y as f32,
                Rect::new(f32::from(frame) * 80.0, 0.0, 80.0, 80.0),
                false,
                1.0,
            );
        }
        self.sprite("IMAGE_MENUBAR", 0.0, 0.0, None, false, 1.0, 1.0);
        let shop_price = |slot: usize, ordinary: &str| {
            state
                .challenge_slot_price(slot)
                .map_or_else(|| ordinary.to_owned(), |price| price.to_string())
        };
        if state.tank != 4 && state.guppy_unlocked {
            self.sprite("IMAGE_MENUBTNU", 18.0, 3.0, None, false, 1.0, 1.0);
            self.sprite(
                "IMAGE_SMALLSWIM",
                6.0,
                -15.0,
                Some(Rect::new(
                    ((state.tick / 2) % 10) as f32 * 80.0,
                    0.0,
                    80.0,
                    80.0,
                )),
                false,
                1.0,
                1.0,
            );
            self.fonts["Pix118"].text(
                &shop_price(0, "100"),
                33.0,
                58.0,
                Color::from_rgba(110, 250, 110, 255),
            );
        }
        if state.tank == 4 && state.breeder_unlocked {
            self.sprite("IMAGE_MENUBTNU", 18.0, 3.0, None, false, 1.0, 1.0);
            self.sprite(
                "IMAGE_SCL_BREEDER",
                26.0,
                5.0,
                Some(Rect::new(
                    ((state.tick / 2) % 10) as f32 * 40.0,
                    0.0,
                    40.0,
                    40.0,
                )),
                false,
                1.0,
                1.0,
            );
            self.fonts["Pix118"].text(
                &shop_price(1, "200"),
                33.0,
                58.0,
                Color::from_rgba(110, 250, 110, 255),
            );
        }
        if state.upgrades.quality_unlocked {
            self.sprite("IMAGE_MENUBTNU", 87.0, 3.0, None, false, 1.0, 1.0);
            if state.upgrades.quality < 2 {
                self.sprite(
                    "IMAGE_FOOD",
                    96.0,
                    6.0,
                    Some(Rect::new(
                        0.0,
                        f32::from(state.upgrades.quality + 1) * 40.0,
                        40.0,
                        40.0,
                    )),
                    false,
                    1.0,
                    1.0,
                );
                self.fonts["Pix118"].text(
                    &shop_price(2, "200"),
                    100.0,
                    58.0,
                    Color::from_rgba(110, 250, 110, 255),
                );
            } else {
                self.fonts["Pix118"].text("MAX", 100.0, 58.0, Color::from_rgba(110, 250, 110, 255));
            }
        }
        if state.upgrades.quantity_unlocked {
            self.sprite("IMAGE_MENUBTNU", 144.0, 3.0, None, false, 1.0, 1.0);
            self.fonts["JungleFever17outline"].text(
                &state.upgrades.quantity.saturating_add(1).min(9).to_string(),
                165.0,
                37.0,
                YELLOW,
            );
            self.fonts["Pix118"].text(
                &if state.upgrades.quantity < 9 {
                    shop_price(3, "300")
                } else {
                    "MAX".to_owned()
                },
                157.0,
                58.0,
                Color::from_rgba(110, 250, 110, 255),
            );
        }
        if state.oscar_unlocked {
            self.sprite("IMAGE_MENUBTNU", 217.0, 3.0, None, false, 1.0, 1.0);
            self.sprite(
                "IMAGE_SCL_OSCAR",
                225.0,
                6.0,
                Some(Rect::new(
                    ((state.tick / 2) % 10) as f32 * 40.0,
                    0.0,
                    40.0,
                    40.0,
                )),
                false,
                1.0,
                1.0,
            );
            self.fonts["Pix118"].text(
                &shop_price(4, "1000"),
                229.0,
                58.0,
                Color::from_rgba(110, 250, 110, 255),
            );
        }
        if state.grubber_unlocked {
            self.sprite("IMAGE_MENUBTNU", 217.0, 3.0, None, false, 1.0, 1.0);
            self.sprite(
                "IMAGE_SCL_GRUBBER",
                225.0,
                4.0,
                Some(Rect::new(
                    ((state.tick / 2) % 10) as f32 * 40.0,
                    0.0,
                    40.0,
                    40.0,
                )),
                false,
                1.0,
                1.0,
            );
            self.fonts["Pix118"].text(
                &shop_price(7, "750"),
                229.0,
                58.0,
                Color::from_rgba(110, 250, 110, 255),
            );
        }
        if state.potion_unlocked {
            self.sprite("IMAGE_MENUBTNU", 217.0, 3.0, None, false, 1.0, 1.0);
            self.sprite(
                "IMAGE_FOOD",
                227.0,
                4.0,
                Some(Rect::new(0.0, 120.0, 40.0, 40.0)),
                false,
                1.0,
                1.0,
            );
            self.fonts["Pix118"].text(
                &if state.potion_armed {
                    "READY".to_owned()
                } else {
                    shop_price(5, "250")
                },
                227.0,
                58.0,
                Color::from_rgba(110, 250, 110, 255),
            );
        }
        if state.starcatcher_unlocked {
            self.sprite("IMAGE_MENUBTNU", 290.0, 3.0, None, false, 1.0, 1.0);
            self.sprite(
                "IMAGE_SCL_STARCATCHER",
                298.0,
                5.0,
                Some(Rect::new(
                    ((state.tick / 2) % 10) as f32 * 40.0,
                    0.0,
                    40.0,
                    40.0,
                )),
                false,
                1.0,
                1.0,
            );
            self.fonts["Pix118"].text(
                &shop_price(6, "750"),
                304.0,
                58.0,
                Color::from_rgba(110, 250, 110, 255),
            );
        }
        if state.gekko_unlocked {
            self.sprite("IMAGE_MENUBTNU", 290.0, 3.0, None, false, 1.0, 1.0);
            self.sprite(
                "IMAGE_SCL_GEKKO",
                300.0,
                5.0,
                Some(Rect::new(
                    ((state.tick / 2) % 10) as f32 * 40.0,
                    0.0,
                    40.0,
                    40.0,
                )),
                false,
                1.0,
                1.0,
            );
            self.fonts["Pix118"].text(
                &shop_price(8, "2000"),
                302.0,
                58.0,
                Color::from_rgba(110, 250, 110, 255),
            );
        }
        if state.ultra_unlocked {
            self.sprite("IMAGE_MENUBTNU", 290.0, 3.0, None, false, 1.0, 1.0);
            self.sprite(
                "IMAGE_SCL_ULTRA",
                300.0,
                5.0,
                Some(Rect::new(
                    ((state.tick / 2) % 10) as f32 * 40.0,
                    0.0,
                    40.0,
                    40.0,
                )),
                false,
                1.0,
                1.0,
            );
            self.fonts["Pix118"].text(
                &shop_price(9, "10000"),
                297.0,
                58.0,
                Color::from_rgba(110, 250, 110, 255),
            );
        }
        if state.weapon_unlocked {
            self.sprite("IMAGE_MENUBTNU", 363.0, 3.0, None, false, 1.0, 1.0);
            if state.weapon_strength < 12 {
                self.sprite(
                    "IMAGE_LASERUPGRADES",
                    370.0,
                    6.0,
                    Some(Rect::new(
                        f32::from(state.weapon_strength - 2) * 46.0,
                        0.0,
                        46.0,
                        39.0,
                    )),
                    false,
                    1.0,
                    1.0,
                );
                self.fonts["Pix118"].text(
                    &state.challenge_slot_price(10).map_or_else(
                        || {
                            match state.tank {
                                3 => "2000",
                                4 => "5000",
                                _ => "1000",
                            }
                            .to_owned()
                        },
                        |price| price.to_string(),
                    ),
                    375.0,
                    58.0,
                    Color::from_rgba(110, 250, 110, 255),
                );
            } else {
                self.fonts["Pix118"].text("MAX", 376.0, 58.0, Color::from_rgba(110, 250, 110, 255));
            }
        }
        if state.egg_unlocked {
            self.sprite("IMAGE_MENUBTNU", 436.0, 3.0, None, false, 1.0, 1.0);
            let texture = &self.images["IMAGE_EGGPIECES"];
            let width = texture.width() / 3.0;
            self.sprite(
                "IMAGE_EGGPIECES",
                443.0,
                6.0,
                Some(Rect::new(
                    f32::from(state.eggs.min(2)) * width,
                    0.0,
                    width,
                    texture.height(),
                )),
                false,
                1.0,
                1.0,
            );
            self.fonts["Pix118"].text(
                &state.egg_price.to_string(),
                458.0,
                58.0,
                Color::from_rgba(110, 250, 110, 255),
            );
        }
        let balance = format!("${}", state.balance);
        let balance_width = self.fonts["ContinuumBold12"]
            .metrics
            .measure_width(&balance)
            .unwrap_or(0);
        self.fonts["ContinuumBold12"].text(
            &balance,
            615.0 - balance_width as f32,
            54.0,
            Color::from_rgba(180, 255, 90, 255),
        );
        self.fonts["JungleFever10outline"].text(
            &format!("Tank {}-{}", state.tank, state.level),
            15.0,
            470.0,
            WHITE,
        );
    }

    fn centered_text(&self, font: &str, text: &str, baseline: f32, color: Color) {
        let rendered = &self.fonts[font];
        let width = rendered.metrics.measure_width(text).unwrap_or(0) as f32;
        rendered.text(text, 320.0 - width / 2.0, baseline, color);
    }

    // Tile a source image's thirds so its corners keep their original size.
    // The original widgets use this image-box convention, not uniform stretch.
    fn image_box(&self, id: &str, source: Rect, destination: Rect) {
        let texture = &self.images[id];
        let edge_x = (source.w / 3.0).floor().min(destination.w / 2.0);
        let edge_y = (source.h / 3.0).floor().min(destination.h / 2.0);
        let source_x = [source.x, source.x + edge_x, source.x + source.w - edge_x];
        let source_y = [source.y, source.y + edge_y, source.y + source.h - edge_y];
        let source_w = [edge_x, source.w - 2.0 * edge_x, edge_x];
        let source_h = [edge_y, source.h - 2.0 * edge_y, edge_y];
        let dest_x = [
            destination.x,
            destination.x + edge_x,
            destination.x + destination.w - edge_x,
        ];
        let dest_y = [
            destination.y,
            destination.y + edge_y,
            destination.y + destination.h - edge_y,
        ];
        let dest_w = [edge_x, destination.w - 2.0 * edge_x, edge_x];
        let dest_h = [edge_y, destination.h - 2.0 * edge_y, edge_y];
        for row in 0..3 {
            for column in 0..3 {
                if source_w[column] <= 0.0 || source_h[row] <= 0.0 {
                    continue;
                }
                let mut y = 0.0;
                while y < dest_h[row] {
                    let height = source_h[row].min(dest_h[row] - y);
                    let mut x = 0.0;
                    while x < dest_w[column] {
                        let width = source_w[column].min(dest_w[column] - x);
                        draw_texture_ex(
                            texture,
                            dest_x[column] + x,
                            dest_y[row] + y,
                            WHITE,
                            DrawTextureParams {
                                source: Some(Rect::new(
                                    source_x[column],
                                    source_y[row],
                                    width,
                                    height,
                                )),
                                ..Default::default()
                            },
                        );
                        x += width;
                    }
                    y += height;
                }
            }
        }
    }

    fn main_button(&self, destination: Rect, label: &str) {
        let texture = &self.images["IMAGE_MAINBUTTON"];
        self.image_box(
            "IMAGE_MAINBUTTON",
            Rect::new(0.0, 0.0, texture.width() / 3.0, texture.height()),
            destination,
        );
        let rendered = &self.fonts["JungleFever12outline"];
        let width = rendered.metrics.measure_width(label).unwrap_or(0) as f32;
        rendered.text(
            label,
            destination.x + (destination.w - width) / 2.0,
            destination.y + destination.h * 0.72,
            Color::from_rgba(255, 240, 0, 255),
        );
    }

    fn draw_bonus_shell(&self, shell: &ShellState) {
        let x = shell.x as f32;
        let y = shell.y as f32;
        if shell.kind == ShellKind::Treasure {
            self.sprite(
                "IMAGE_MONEYBAG",
                x + 18.0,
                y + 18.0,
                None,
                false,
                1.0,
                shell.alpha(),
            );
        } else {
            let row = match shell.kind {
                ShellKind::Silver => 0.0,
                ShellKind::Gold => 1.0,
                ShellKind::Diamond => 2.0,
                ShellKind::Pearl => 3.0,
                ShellKind::Treasure => unreachable!(),
            };
            let image = &self.images["IMAGE_SHELLS"];
            self.sprite(
                "IMAGE_SHELLS",
                x + 20.0,
                y + 20.0,
                Some(Rect::new(
                    f32::from(shell.sprite_frame()) * image.width() / 20.0,
                    row * image.height() / 4.0,
                    image.width() / 20.0,
                    image.height() / 4.0,
                )),
                false,
                1.0,
                shell.alpha(),
            );
        }
    }

    fn draw_bonus(&self, bonus: &BonusState) {
        self.sprite(
            match bonus.origin_tank {
                2 => "IMAGE_AQUARIUM2",
                3 => "IMAGE_AQUARIUM4",
                4 => "IMAGE_AQUARIUM5",
                _ => "IMAGE_AQUARIUM1",
            },
            0.0,
            0.0,
            None,
            false,
            1.0,
            1.0,
        );
        let timer = bonus.tick.saturating_sub(bonus.initial_count);
        let bucket_y = if bonus.started_at.is_some() {
            Some(265.0)
        } else if timer > 129 {
            if timer < 150 {
                let progress = (timer.saturating_sub(130) as f32 / 20.0).clamp(0.0, 1.0);
                Some(270.0 * progress * progress)
            } else if timer < 152 {
                Some(270.0 - (timer.saturating_sub(150) as f32 * 2.5))
            } else {
                Some(265.0)
            }
        } else {
            None
        };
        if let Some(y) = bucket_y {
            self.sprite("IMAGE_BONUSBUCKET", 260.0, y, None, false, 1.0, 1.0);
            if bonus.started_at.is_some() {
                self.centered_text(
                    "ContinuumBold14",
                    &bonus.shells_earned.to_string(),
                    y + 151.0,
                    Color::from_rgba(240, 163, 59, 255),
                );
            }
        }
        if bonus.draw_order.is_empty() {
            for shell in &bonus.shells {
                self.draw_bonus_shell(shell);
            }
        } else {
            for id in &bonus.draw_order {
                if let Some(shell) = bonus.shells.iter().find(|shell| shell.id == *id) {
                    self.draw_bonus_shell(shell);
                }
            }
        }
        if bonus.started_at.is_none() {
            self.centered_text("JungleFever17outline", "BONUS ROUND", 235.0, YELLOW);
            self.centered_text(
                "ContinuumBold14",
                "Collect as many shells as you can!",
                260.0,
                Color::from_rgba(180, 250, 90, 255),
            );
            let examples = [
                (ShellKind::Silver, "1"),
                (ShellKind::Gold, "2"),
                (ShellKind::Diamond, "5"),
                (ShellKind::Pearl, "10"),
                (ShellKind::Treasure, "20"),
            ];
            for (index, (kind, value)) in examples.iter().enumerate() {
                let x = 180.0 + index as f32 * 60.0;
                if *kind == ShellKind::Treasure {
                    self.sprite("IMAGE_MONEYBAG", x, 268.0, None, false, 1.0, 1.0);
                } else {
                    let row = match kind {
                        ShellKind::Silver => 0.0,
                        ShellKind::Gold => 1.0,
                        ShellKind::Diamond => 2.0,
                        ShellKind::Pearl => 3.0,
                        ShellKind::Treasure => unreachable!(),
                    };
                    let image = &self.images["IMAGE_SHELLS"];
                    self.sprite(
                        "IMAGE_SHELLS",
                        x,
                        280.0,
                        Some(Rect::new(
                            0.0,
                            row * image.height() / 4.0,
                            image.width() / 20.0,
                            image.height() / 4.0,
                        )),
                        false,
                        1.0,
                        1.0,
                    );
                }
                self.fonts["ContinuumBold12"].text(value, x + 11.0, 330.0, WHITE);
            }
            self.centered_text("JungleFever10outline", "Click to start", 365.0, WHITE);
            if timer < 148 {
                let frame = (148 - timer) / 36;
                if frame < 3 {
                    let id = ["IMAGE_BONUS1", "IMAGE_BONUS2", "IMAGE_BONUS3"][frame as usize];
                    let within_frame = (148 - timer) % 36;
                    let scale = ((36 - within_frame).min(5) as f32 * 0.8 / 5.0) + 0.2;
                    let image = &self.images[id];
                    let width = image.width() * scale;
                    self.sprite(
                        id,
                        (640.0 - width) / 2.0,
                        90.0 - (scale - 1.0) * image.height() * 0.5,
                        None,
                        false,
                        scale,
                        ((within_frame.min(20) as f32) / 20.0).clamp(0.0, 1.0),
                    );
                }
            }
        }
        let seconds = bonus.remaining_seconds();
        self.fonts["ContinuumBold12"].text(
            &format!("Time Remaining: {}:{:02}", seconds / 60, seconds % 60),
            465.0,
            470.0,
            WHITE,
        );
    }

    fn draw_results_purchase(&self, receipt: &PurchaseReceipt, updates: u32, balance: u32) {
        let Some((outcome, price)) = results_offer(receipt.offered_cursor) else {
            return;
        };
        let name = match outcome {
            crate::sim::BonusPurchaseOutcome::Pet(PetKind::Brinkley) => "BRINKLEY",
            crate::sim::BonusPurchaseOutcome::Pet(PetKind::Nostradamus) => "NOSTRADAMUS",
            crate::sim::BonusPurchaseOutcome::Pet(PetKind::Stanley) => "STANLEY",
            crate::sim::BonusPurchaseOutcome::Pet(PetKind::Walter) => "WALTER",
            crate::sim::BonusPurchaseOutcome::CapacityAtLeast(4) => "FOURTH PET SLOT",
            crate::sim::BonusPurchaseOutcome::CapacityAtLeast(7) => "SEVEN PET SLOTS",
            _ => unreachable!("implemented results offer"),
        };
        if receipt.purchased {
            self.centered_text(
                "JungleFever10outline",
                &format!("{name} purchased"),
                412.0,
                YELLOW,
            );
        } else if updates >= 30 {
            self.main_button(
                results_offer_rect(),
                &format!("{name} - {},000 shells", price / 1000),
            );
        }
        if receipt.confirming {
            draw_rectangle(
                95.0,
                220.0,
                450.0,
                150.0,
                Color::new(0.02, 0.08, 0.15, 0.96),
            );
            self.centered_text(
                "JungleFever15outline",
                &format!("Buy {name}?"),
                258.0,
                YELLOW,
            );
            self.centered_text(
                "JungleFever10outline",
                &format!("{},000 shells - available {balance}", price / 1000),
                284.0,
                WHITE,
            );
            self.main_button(results_confirm_rect(true), "Buy");
            self.main_button(results_confirm_rect(false), "Cancel");
        }
    }

    fn draw_bonus_results(&self, result: &BonusResult, balance: u32) {
        self.sprite("IMAGE_HATCHSCREEN", 0.0, 0.0, None, false, 1.0, 1.0);
        let title = &self.images["IMAGE_SCREENTITLE"];
        self.image_box(
            "IMAGE_SCREENTITLE",
            Rect::new(0.0, 0.0, title.width(), title.height()),
            Rect::new(20.0, 0.0, 600.0, title.height()),
        );
        self.centered_text(
            "JungleFever17outline",
            if result.origin_tank == 5 {
                "ADVENTURE RESULTS"
            } else {
                "BONUS RESULTS"
            },
            25.0,
            Color::from_rgba(255, 200, 0, 255),
        );
        draw_rectangle(
            175.0,
            230.0,
            300.0,
            225.0,
            Color::new(0.03, 0.12, 0.2, 0.72),
        );
        self.fonts["JungleFever12outline"].text("Shells", 205.0, 286.0, YELLOW);
        draw_line(205.0, 295.0, 445.0, 295.0, 1.0, WHITE);
        self.fonts["JungleFever10outline"].text("Bonus Reward", 205.0, 324.0, WHITE);
        self.fonts["JungleFever10outline"].text(
            &result
                .presented_balance()
                .saturating_sub(result.previous_balance)
                .to_string(),
            398.0,
            324.0,
            YELLOW,
        );
        self.fonts["JungleFever10outline"].text("New Balance", 205.0, 364.0, WHITE);
        self.fonts["JungleFever10outline"].text(
            &bonus_results_display_balance(result, balance).to_string(),
            398.0,
            364.0,
            YELLOW,
        );
        self.sprite("IMAGE_HATCHREFLECTION", 240.0, 60.0, None, false, 1.0, 1.0);
        let height = self.images["IMAGE_MAINBUTTON"].height();
        self.main_button(
            results_continue_rect(height),
            if result.updates >= 30 {
                "Click Here To Continue"
            } else {
                "Please Wait..."
            },
        );
        self.draw_results_purchase(&result.purchase, result.updates, balance);
    }

    fn draw_hatch(&self, pet: PetKind, updates: u32) {
        self.sprite("IMAGE_HATCHSCREEN", 0.0, 0.0, None, false, 1.0, 1.0);
        let title = &self.images["IMAGE_SCREENTITLE"];
        self.image_box(
            "IMAGE_SCREENTITLE",
            Rect::new(0.0, 0.0, title.width(), title.height()),
            Rect::new(20.0, 0.0, 600.0, title.height()),
        );
        self.fonts["JungleFever17outline"].text(
            "You have found:",
            215.0,
            25.0,
            Color::from_rgba(255, 200, 0, 255),
        );
        if updates < 141 {
            let (id, columns, frame) = if updates < 81 {
                ("IMAGE_EGGCRACK1", 13.0, updates.saturating_sub(56) / 2)
            } else {
                ("IMAGE_EGGCRACK2", 10.0, updates.saturating_sub(120) / 2)
            };
            let image = &self.images[id];
            let width = image.width() / columns;
            self.sprite(
                id,
                258.0,
                60.0,
                Some(Rect::new(
                    frame.min(columns as u32 - 1) as f32 * width,
                    0.0,
                    width,
                    image.height(),
                )),
                false,
                1.0,
                1.0,
            );
        } else {
            let (id, y, column) = match pet {
                PetKind::Stinky => ("IMAGE_STINKY", 100.0, updates % 20 / 2),
                PetKind::Niko => {
                    let phase = updates % 20;
                    (
                        "IMAGE_NIKO",
                        70.0,
                        if phase > 9 { 19 - phase } else { phase },
                    )
                }
                PetKind::Itchy => ("IMAGE_ITCHY", 90.0, updates % 20 / 2),
                PetKind::Prego => ("IMAGE_PREGO", 90.0, updates % 20 / 2),
                PetKind::Zorf => ("IMAGE_ZORF", 90.0, updates % 20 / 2),
                PetKind::Clyde => ("IMAGE_CLYDE", 100.0, updates % 40 / 4),
                PetKind::Vert => {
                    let phase = updates % 20;
                    (
                        "IMAGE_VERT",
                        90.0,
                        if phase > 9 { 19 - phase } else { phase },
                    )
                }
                PetKind::Rufus => ("IMAGE_RUFUS", 90.0, updates % 20 / 2),
                PetKind::Meryl => ("IMAGE_MERYL", 90.0, updates % 20 / 2),
                PetKind::Wadsworth => ("IMAGE_WADSWORTH", 90.0, updates % 20 / 2),
                PetKind::Seymour => ("IMAGE_SEYMOUR", 90.0, updates % 40 / 4),
                PetKind::Shrapnel => ("IMAGE_SHRAPNEL", 90.0, updates % 40 / 4),
                PetKind::Gumbo => ("IMAGE_GUMBO", 90.0, updates % 20 / 2),
                PetKind::Blip => ("IMAGE_BLIP", 90.0, updates % 20 / 2),
                PetKind::Rhubarb => ("IMAGE_RHUBARB", 90.0, updates % 40 / 4),
                PetKind::Nimbus => ("IMAGE_NIMBUS", 90.0, updates % 20 / 2),
                PetKind::Amp => ("IMAGE_AMP", 100.0, updates % 20 / 2),
                PetKind::Gash => ("IMAGE_GASH", 90.0, updates % 20 / 2),
                PetKind::Angie => ("IMAGE_ANGIE", 90.0, updates % 20 / 2),
                PetKind::Presto => ("IMAGE_PRESTO", 90.0, updates % 20 / 2),
                PetKind::Brinkley => ("IMAGE_BRINKLEY", 90.0, updates % 20 / 2),
                PetKind::Nostradamus => ("IMAGE_NOSTRADAMUS", 90.0, updates % 20 / 2),
                PetKind::Stanley => ("IMAGE_STANLEY", 90.0, updates % 20 / 2),
                PetKind::Walter => ("IMAGE_WALTER", 90.0, updates % 40 / 4),
            };
            let (preview_x, preview_width, preview_height) = if pet == PetKind::Amp {
                (236.0, 160.0, 60.0)
            } else {
                (278.0, 80.0, 80.0)
            };
            self.sprite(
                id,
                preview_x,
                y,
                Some(Rect::new(
                    column as f32 * preview_width,
                    0.0,
                    preview_width,
                    preview_height,
                )),
                false,
                1.0,
                1.0,
            );
            self.centered_text(
                if matches!(
                    pet,
                    PetKind::Wadsworth | PetKind::Shrapnel | PetKind::Rhubarb
                ) {
                    "JungleFever12outline"
                } else {
                    "JungleFever15outline"
                },
                match pet {
                    PetKind::Stinky => "STINKY the Snail",
                    PetKind::Niko => "NIKO the Oyster",
                    PetKind::Itchy => "ITCHY the Swordfish",
                    PetKind::Prego => "PREGO the Momma Fish",
                    PetKind::Zorf => "ZORF the Sea Horse",
                    PetKind::Clyde => "CLYDE the Jellyfish",
                    PetKind::Vert => "VERT the Skeleton",
                    PetKind::Rufus => "RUFUS the Fiddler Crab",
                    PetKind::Meryl => "MERYL the Mermaid",
                    PetKind::Wadsworth => "WADSWORTH the Whale",
                    PetKind::Seymour => "SEYMOUR the Turtle",
                    PetKind::Shrapnel => "SHRAPNEL the Robot Fish",
                    PetKind::Gumbo => "GUMBO the Angler",
                    PetKind::Blip => "BLIP the Porpoise",
                    PetKind::Rhubarb => "RHUBARB the Hermit Crab",
                    PetKind::Nimbus => "NIMBUS the Manta Ray",
                    PetKind::Amp => "AMP the Electric Eel",
                    PetKind::Gash => "GASH the Shark",
                    PetKind::Angie => "ANGIE the Angelfish",
                    PetKind::Presto => "PRESTO",
                    PetKind::Brinkley => "BRINKLEY",
                    PetKind::Nostradamus => "NOSTRADAMUS",
                    PetKind::Stanley => "STANLEY",
                    PetKind::Walter => "WALTER",
                },
                260.0,
                Color::from_rgba(255, 200, 0, 255),
            );
        }
        self.sprite("IMAGE_HATCHREFLECTION", 240.0, 60.0, None, false, 1.0, 1.0);
        if updates > 170 {
            let description = match pet {
                PetKind::Stinky => [
                    "STINKY roams around the",
                    "bottom of your tank, catching",
                    "any coins you may have missed.",
                ],
                PetKind::Niko => [
                    "NIKO produces pearls that",
                    "you can click on for a",
                    "hefty sum of money.",
                ],
                PetKind::Itchy => [
                    "ITCHY helps you by attacking",
                    "aliens when they appear.",
                    "",
                ],
                PetKind::Prego => [
                    "PREGO helps populate your",
                    "tank by giving birth to a new",
                    "baby guppy every so often.",
                ],
                PetKind::Zorf => ["ZORF gives you a hand in", "keeping your fish fed.", ""],
                PetKind::Clyde => [
                    "CLYDE drifts slowly through",
                    "your tank, collecting any",
                    "coins it passes by.",
                ],
                PetKind::Vert => [
                    "VERT drops gold coins just like",
                    "a large guppy, but doesn't need",
                    "fish food to survive.",
                ],
                PetKind::Rufus => [
                    "RUFUS guards the tank floor,",
                    "dealing heavy damage to aliens",
                    "that come within reach.",
                ],
                PetKind::Meryl => [
                    "MERYL's song cheers up all the",
                    "guppies in the tank, making",
                    "them drop coins faster.",
                ],
                PetKind::Wadsworth => [
                    "WADSWORTH helps by sheltering",
                    "your baby and medium guppies",
                    "from hungry aliens.",
                ],
                PetKind::Seymour => [
                    "SEYMOUR's presence makes all",
                    "coins and diamonds drift",
                    "at a slower rate.",
                ],
                PetKind::Shrapnel => [
                    "SHRAPNEL drops bombs that",
                    "blow up fish on contact but",
                    "give lots of cash when clicked.",
                ],
                PetKind::Gumbo => [
                    "GUMBO attracts guppies using",
                    "the lantern on his head,",
                    "luring them away from aliens.",
                ],
                PetKind::Blip => [
                    "BLIP provides you with info",
                    "that helps you better combat",
                    "aliens and keep your fish fed.",
                ],
                PetKind::Rhubarb => [
                    "RHUBARB snaps his claws at",
                    "fish, keeping them off the",
                    "bottom of your tank.",
                ],
                PetKind::Nimbus => [
                    "NIMBUS tosses any coins or",
                    "food he catches back up",
                    "toward the top of the tank.",
                ],
                PetKind::Amp => [
                    "AMP can electrocute your",
                    "entire tank, killing your fish",
                    "and turning them into diamonds.",
                ],
                PetKind::Gash => [
                    "GASH helps fight aliens.",
                    "He occasionally eats a guppy",
                    "while the tank is peaceful.",
                ],
                PetKind::Angie => ["ANGIE can resurrect", "dead fish.", ""],
                PetKind::Presto => ["PRESTO has joined", "your pet collection.", ""],
                PetKind::Brinkley => ["BRINKLEY eats food", "and drops coins.", ""],
                PetKind::Nostradamus => ["NOSTRADAMUS makes", "special food.", ""],
                PetKind::Stanley => ["STANLEY launches", "missiles at aliens.", ""],
                PetKind::Walter => ["WALTER punches fish", "with his glove.", ""],
            };
            for (index, line) in description.iter().enumerate() {
                self.centered_text(
                    "JungleFever10outline",
                    line,
                    300.0 + index as f32 * 20.0,
                    WHITE,
                );
            }
            self.fonts["JungleFever10outline"].text(
                "Your game has been saved.",
                221.0,
                390.0,
                Color::from_rgba(255, 255, 100, 255),
            );
        }
        let height = self.images["IMAGE_MAINBUTTON"].height();
        self.main_button(
            Rect::new(186.0, 445.0, 264.0, height),
            if updates > 170 {
                "Click Here to Continue"
            } else {
                "Please Wait..."
            },
        );
        self.main_button(Rect::new(525.0, 4.0, 80.0, height), "Menu");
    }

    fn draw_pet_selection(&self, session: &AdventureSession, selected: &[PetKind], pointer: Vec2) {
        let backdrop = &self.images["IMAGE_SCREENBACK"];
        self.image_box(
            "IMAGE_SCREENBACK",
            Rect::new(0.0, 0.0, backdrop.width(), backdrop.height()),
            Rect::new(-5.0, -5.0, 650.0, 490.0),
        );
        let title = &self.images["IMAGE_SCREENTITLE"];
        self.image_box(
            "IMAGE_SCREENTITLE",
            Rect::new(0.0, 0.0, title.width(), title.height()),
            Rect::new(20.0, 0.0, 600.0, title.height()),
        );
        self.sprite("IMAGE_FISHBOX", 216.0, 48.0, None, false, 1.0, 1.0);
        self.sprite("IMAGE_FISHBOXBUTTON", 221.0, 243.0, None, false, 1.0, 1.0);
        self.fonts["JungleFever17outline"].text(
            "Choose Your Pets",
            215.0,
            25.0,
            Color::from_rgba(255, 200, 0, 255),
        );
        let mut hovered = None;
        for (index, pet) in session.progress.unlocked_pets.iter().enumerate() {
            let card = pet_card_rect(index);
            if card.contains(pointer) {
                hovered = Some(*pet);
            }
            self.sprite("IMAGE_PETBUTTONHOLE", card.x, card.y, None, false, 1.0, 1.0);
            let button_width = self.images["IMAGE_PETBUTTON"].width() / 4.0;
            self.sprite(
                "IMAGE_PETBUTTON",
                card.x,
                card.y,
                Some(Rect::new(
                    button_width * if selected.contains(pet) { 2.0 } else { 1.0 },
                    0.0,
                    button_width,
                    self.images["IMAGE_PETBUTTON"].height(),
                )),
                false,
                1.0,
                1.0,
            );
            let icon = match pet {
                PetKind::Stinky => "IMAGE_SCL_STINKY",
                PetKind::Niko => "IMAGE_SCL_NIKO",
                PetKind::Itchy => "IMAGE_SCL_ITCHY",
                PetKind::Prego => "IMAGE_SCL_PREGO",
                PetKind::Zorf => "IMAGE_SCL_ZORF",
                PetKind::Clyde => "IMAGE_SCL_CLYDE",
                PetKind::Vert => "IMAGE_SCL_VERT",
                PetKind::Rufus => "IMAGE_SCL_RUFUS",
                PetKind::Meryl => "IMAGE_SCL_MERYL",
                PetKind::Wadsworth => "IMAGE_SCL_WADSWORTH",
                PetKind::Seymour => "IMAGE_SCL_SEYMOUR",
                PetKind::Shrapnel => "IMAGE_SCL_SHRAPNEL",
                PetKind::Gumbo => "IMAGE_SCL_GUMBO",
                PetKind::Blip => "IMAGE_SCL_BLIP",
                PetKind::Rhubarb => "IMAGE_SCL_RHUBARB",
                PetKind::Nimbus => "IMAGE_SCL_NIMBUS",
                PetKind::Amp => "IMAGE_SCL_AMP",
                PetKind::Gash => "IMAGE_SCL_GASH",
                PetKind::Angie => "IMAGE_SCL_ANGIE",
                PetKind::Presto => "IMAGE_SCL_PRESTO",
                PetKind::Brinkley => "IMAGE_SCL_BRINKLEY",
                PetKind::Nostradamus => "IMAGE_SCL_NOSTRADAMUS",
                PetKind::Stanley => "IMAGE_SCL_STANLEY",
                PetKind::Walter => "IMAGE_SCL_WALTER",
            };
            let image = &self.images[icon];
            let column = if matches!(*pet, PetKind::Niko | PetKind::Vert) {
                let phase = session.ticks % 18;
                if phase > 9 { 18 - phase } else { phase }
            } else if matches!(
                *pet,
                PetKind::Clyde | PetKind::Seymour | PetKind::Shrapnel | PetKind::Rhubarb
            ) {
                session.ticks / 4 % 10
            } else {
                session.ticks / 2 % 10
            };
            self.sprite(
                icon,
                card.x + 14.0,
                card.y + 10.0,
                Some(Rect::new(
                    column as f32 * image.width() / 10.0,
                    0.0,
                    image.width() / 10.0,
                    image.height(),
                )),
                false,
                1.0,
                1.0,
            );
            self.sprite("IMAGE_PETBUTTONRING", card.x, card.y, None, false, 1.0, 1.0);
            self.sprite(
                "IMAGE_PETBUTTONREFLECT",
                card.x,
                card.y,
                None,
                false,
                1.0,
                1.0,
            );
        }
        if let Some(pet) = hovered.or_else(|| selected.last().copied()) {
            let (image, name, description, y) = match pet {
                PetKind::Stinky => (
                    "IMAGE_STINKY",
                    "STINKY the Snail",
                    [
                        "STINKY roams around the",
                        "bottom of your tank, catching",
                        "any coins you may have missed.",
                    ],
                    90.0,
                ),
                PetKind::Niko => (
                    "IMAGE_NIKO",
                    "NIKO the Oyster",
                    [
                        "NIKO produces pearls that",
                        "you can click on for a",
                        "hefty sum of money.",
                    ],
                    85.0,
                ),
                PetKind::Itchy => (
                    "IMAGE_ITCHY",
                    "ITCHY the Swordfish",
                    [
                        "ITCHY helps you by attacking",
                        "aliens when they appear.",
                        "",
                    ],
                    90.0,
                ),
                PetKind::Prego => (
                    "IMAGE_PREGO",
                    "PREGO the Momma Fish",
                    [
                        "PREGO helps populate your",
                        "tank by giving birth to a new",
                        "baby guppy every so often.",
                    ],
                    90.0,
                ),
                PetKind::Zorf => (
                    "IMAGE_ZORF",
                    "ZORF the Sea Horse",
                    ["ZORF gives you a hand in", "keeping your fish fed.", ""],
                    90.0,
                ),
                PetKind::Clyde => (
                    "IMAGE_CLYDE",
                    "CLYDE the Jellyfish",
                    [
                        "CLYDE drifts slowly through",
                        "your tank, collecting any",
                        "coins it passes by.",
                    ],
                    90.0,
                ),
                PetKind::Vert => (
                    "IMAGE_VERT",
                    "VERT the Skeleton",
                    [
                        "VERT drops gold coins just like",
                        "a large guppy, but doesn't need",
                        "fish food to survive.",
                    ],
                    90.0,
                ),
                PetKind::Rufus => (
                    "IMAGE_RUFUS",
                    "RUFUS the Fiddler Crab",
                    [
                        "RUFUS guards the tank floor,",
                        "dealing heavy damage to aliens",
                        "that come within reach.",
                    ],
                    90.0,
                ),
                PetKind::Meryl => (
                    "IMAGE_MERYL",
                    "MERYL the Mermaid",
                    [
                        "MERYL's song cheers up all the",
                        "guppies in the tank, making",
                        "them drop coins faster.",
                    ],
                    90.0,
                ),
                PetKind::Wadsworth => (
                    "IMAGE_WADSWORTH",
                    "WADSWORTH the Whale",
                    [
                        "WADSWORTH helps by sheltering",
                        "your baby and medium guppies",
                        "from hungry aliens.",
                    ],
                    90.0,
                ),
                PetKind::Seymour => (
                    "IMAGE_SEYMOUR",
                    "SEYMOUR the Turtle",
                    [
                        "SEYMOUR's presence makes all",
                        "coins and diamonds drift",
                        "at a slower rate.",
                    ],
                    90.0,
                ),
                PetKind::Shrapnel => (
                    "IMAGE_SHRAPNEL",
                    "SHRAPNEL the Robot Fish",
                    [
                        "SHRAPNEL drops bombs that",
                        "blow up fish on contact but",
                        "give lots of cash when clicked.",
                    ],
                    90.0,
                ),
                PetKind::Gumbo => (
                    "IMAGE_GUMBO",
                    "GUMBO the Angler",
                    [
                        "GUMBO attracts guppies using",
                        "the lantern on his head,",
                        "luring them away from aliens.",
                    ],
                    90.0,
                ),
                PetKind::Blip => (
                    "IMAGE_BLIP",
                    "BLIP the Porpoise",
                    [
                        "BLIP provides you with info",
                        "that helps you better combat",
                        "aliens and keep your fish fed.",
                    ],
                    90.0,
                ),
                PetKind::Rhubarb => (
                    "IMAGE_RHUBARB",
                    "RHUBARB the Hermit Crab",
                    [
                        "RHUBARB snaps his claws at",
                        "fish, keeping them off the",
                        "bottom of your tank.",
                    ],
                    90.0,
                ),
                PetKind::Nimbus => (
                    "IMAGE_NIMBUS",
                    "NIMBUS the Manta Ray",
                    [
                        "NIMBUS tosses any coins or",
                        "food he catches back up",
                        "toward the top of the tank.",
                    ],
                    90.0,
                ),
                PetKind::Amp => (
                    "IMAGE_AMP",
                    "AMP the Electric Eel",
                    [
                        "AMP can electrocute your",
                        "entire tank, killing your fish",
                        "and turning them into diamonds.",
                    ],
                    90.0,
                ),
                PetKind::Gash => (
                    "IMAGE_GASH",
                    "GASH the Shark",
                    [
                        "GASH helps fight aliens.",
                        "He occasionally eats a guppy",
                        "while the tank is peaceful.",
                    ],
                    90.0,
                ),
                PetKind::Angie => (
                    "IMAGE_ANGIE",
                    "ANGIE the Angelfish",
                    ["ANGIE can resurrect", "dead fish.", ""],
                    90.0,
                ),
                PetKind::Presto => (
                    "IMAGE_PRESTO",
                    "PRESTO",
                    ["PRESTO has joined", "your pet collection.", ""],
                    90.0,
                ),
                PetKind::Brinkley => (
                    "IMAGE_BRINKLEY",
                    "BRINKLEY",
                    ["BRINKLEY eats food", "and drops coins.", ""],
                    90.0,
                ),
                PetKind::Nostradamus => (
                    "IMAGE_NOSTRADAMUS",
                    "NOSTRADAMUS",
                    ["NOSTRADAMUS makes", "special food.", ""],
                    90.0,
                ),
                PetKind::Stanley => (
                    "IMAGE_STANLEY",
                    "STANLEY",
                    ["STANLEY launches", "missiles at aliens.", ""],
                    90.0,
                ),
                PetKind::Walter => (
                    "IMAGE_WALTER",
                    "WALTER",
                    ["WALTER punches fish", "with his glove.", ""],
                    90.0,
                ),
            };
            let column = if matches!(pet, PetKind::Niko | PetKind::Vert) {
                let phase = session.ticks % 18;
                if phase > 9 { 18 - phase } else { phase }
            } else if matches!(
                pet,
                PetKind::Clyde | PetKind::Seymour | PetKind::Shrapnel | PetKind::Rhubarb
            ) {
                session.ticks / 4 % 10
            } else {
                session.ticks % 20 / 2
            };
            let (preview_x, preview_width, preview_height) = if pet == PetKind::Amp {
                (240.0, 160.0, 60.0)
            } else {
                (280.0, 80.0, 80.0)
            };
            self.sprite(
                image,
                preview_x,
                y,
                Some(Rect::new(
                    column as f32 * preview_width,
                    0.0,
                    preview_width,
                    preview_height,
                )),
                false,
                1.0,
                1.0,
            );
            self.centered_text(
                "JungleFever15outline",
                name,
                180.0,
                Color::from_rgba(255, 200, 0, 255),
            );
            for (index, line) in description.iter().enumerate() {
                self.centered_text(
                    "JungleFever10outline",
                    line,
                    200.0 + index as f32 * 15.0,
                    WHITE,
                );
            }
        }
        let remaining = session
            .progress
            .selection_capacity()
            .saturating_sub(selected.len());
        if remaining > 0 {
            self.centered_text(
                "JungleFever10outline",
                &format!(
                    "Choose {remaining} more {}",
                    if remaining == 1 { "pet" } else { "pets" }
                ),
                75.0,
                WHITE,
            );
            self.centered_text(
                "JungleFever10outline",
                if matches!(session.phase, AdventurePhase::TimeTrialPetSelection { .. }) {
                    "for the timed tank"
                } else if matches!(
                    session.phase,
                    AdventurePhase::ChallengePetSelection { .. }
                        | AdventurePhase::ChallengePetSelectionConfirmation { .. }
                ) {
                    "for the challenge tank"
                } else {
                    "to take to the next level"
                },
                90.0,
                WHITE,
            );
        }
        let height = self.images["IMAGE_MAINBUTTON"].height();
        self.main_button(Rect::new(225.0, 250.0, 186.0, height), "Continue");
        self.main_button(Rect::new(525.0, 4.0, 80.0, height), "Menu");
    }

    fn draw_finale_hatch(&self, updates: u32) {
        self.sprite("IMAGE_HATCHSCREEN", 0.0, 0.0, None, false, 1.0, 1.0);
        self.centered_text(
            "JungleFever17outline",
            "You have found:",
            25.0,
            Color::from_rgba(255, 200, 0, 255),
        );
        if updates > 139 {
            // Source-qualified special raw999 appearance. The screen clock
            // reconstructs its pose across reload; visual fidelity is pending.
            let travel = (updates.saturating_sub(140).min(21) * 12)
                + (updates.saturating_sub(161).min(10) * 8)
                + (updates.saturating_sub(171).min(10) * 5)
                + (updates.saturating_sub(181).min(10) * 2)
                + updates.saturating_sub(191).min(29);
            let phase = updates % 16;
            let bob = if updates >= 220 {
                -(phase.min(4) as i32) + phase.saturating_sub(8).min(4) as i32
            } else {
                0
            };
            let y = 480.0 - travel as f32 + bob as f32;
            self.sprite(
                "IMAGE_BOSS",
                238.0,
                y,
                Some(Rect::new(
                    (updates % 20 / 2) as f32 * 160.0,
                    0.0,
                    160.0,
                    160.0,
                )),
                false,
                1.0,
                1.0,
            );
        }
        self.sprite("IMAGE_HATCHREFLECTION", 240.0, 60.0, None, false, 1.0, 1.0);
        if updates > 170 {
            self.centered_text(
                "JungleFever15outline",
                "Evil Alien Mastermind",
                260.0,
                Color::from_rgba(255, 200, 0, 255),
            );
            self.centered_text(
                "JungleFever10outline",
                "This creature devours the contents of your tank.",
                310.0,
                WHITE,
            );
            self.centered_text(
                "JungleFever10outline",
                "Your game has been saved.",
                390.0,
                Color::from_rgba(255, 255, 100, 255),
            );
        }
        let height = self.images["IMAGE_MAINBUTTON"].height();
        self.main_button(
            Rect::new(186.0, 445.0, 264.0, height),
            if updates > 170 {
                "Click Here to Continue"
            } else {
                "Please Wait..."
            },
        );
        self.main_button(Rect::new(525.0, 4.0, 80.0, height), "Menu");
    }

    fn draw_adventure_finale_interlude(&self, interlude: Option<&InterludeController>) {
        self.sprite("IMAGE_SCREENBACK", 0.0, 0.0, None, false, 1.0, 1.0);
        if let Some(interlude) = interlude {
            for row in interlude.visible_rows() {
                let (line, size, color) = match row.content {
                    RowContent::Heading(line) => (line.to_owned(), 28.0, YELLOW),
                    RowContent::Text(line) => (line.to_owned(), 21.0, WHITE),
                    RowContent::Cast(pet) => (format!("{pet:?}"), 21.0, WHITE),
                };
                let width = measure_text(&line, None, size as u16, 1.0).width;
                draw_text(&line, row.x as f32 - width / 2.0, row.y as f32, size, color);
            }
            for pose in interlude.resurrection_poses() {
                draw_circle(
                    pose.x as f32 + 40.0,
                    pose.y as f32 + 40.0,
                    17.0,
                    Color::new(1.0, 0.9, 0.45, f32::from(pose.alpha) / 255.0),
                );
                draw_text(
                    format!("{:?}", pose.pet),
                    pose.x as f32,
                    pose.y as f32 + 75.0,
                    16.0,
                    WHITE,
                );
            }
        }
        let height = self.images["IMAGE_MAINBUTTON"].height();
        self.main_button(Rect::new(186.0, 445.0, 264.0, height), "Main Menu");
    }

    fn draw_virtual_tank(&self, session: &AdventureSession) {
        let background = match session.virtual_tank.backdrop {
            0 => "IMAGE_AQUARIUM1",
            1 => "IMAGE_AQUARIUM2",
            2 => "IMAGE_AQUARIUM4",
            3 => "IMAGE_AQUARIUM5",
            _ => "IMAGE_AQUARIUM6",
        };
        self.sprite(background, 0.0, 0.0, None, false, 1.0, 1.0);
        if let Some(board) = &session.virtual_board {
            for food in &board.food {
                let color = match food.kind {
                    crate::virtual_tank::VirtualFoodKind::Basic => ORANGE,
                    crate::virtual_tank::VirtualFoodKind::Upgraded => YELLOW,
                    crate::virtual_tank::VirtualFoodKind::Snot => GREEN,
                    crate::virtual_tank::VirtualFoodKind::Star => GOLD,
                    _ => WHITE,
                };
                draw_circle(food.x, food.y, 6.0, color);
            }
            for prey in &board.prey {
                draw_circle(prey.x, prey.y, 11.0, SKYBLUE);
                draw_text(
                    format!("{:?}", prey.kind),
                    prey.x - 14.0,
                    prey.y - 13.0,
                    13.0,
                    WHITE,
                );
            }
            for pose in &board.poses {
                let Some(fish) = session
                    .virtual_tank
                    .active_fish()
                    .find(|fish| fish.id == pose.fish_id)
                else {
                    continue;
                };
                let (image, frame_size) = match fish.kind {
                    VirtualFishKind::Guppy => ("IMAGE_SMALLSWIM", 80.0),
                    VirtualFishKind::Oscar => ("IMAGE_SCL_OSCAR", 40.0),
                    VirtualFishKind::Ultra | VirtualFishKind::Bi => ("IMAGE_SCL_ULTRA", 40.0),
                    VirtualFishKind::Gekko => ("IMAGE_SCL_GEKKO", 40.0),
                    VirtualFishKind::Penta => ("IMAGE_SCL_STARCATCHER", 40.0),
                    VirtualFishKind::Grubber => ("IMAGE_SCL_GRUBBER", 40.0),
                    VirtualFishKind::Breeder => ("IMAGE_SCL_BREEDER", 40.0),
                    VirtualFishKind::Sylvester | VirtualFishKind::Ball => ("IMAGE_SYLV", 80.0),
                };
                self.sprite(
                    image,
                    pose.x - frame_size / 2.0,
                    pose.y - frame_size / 2.0,
                    Some(Rect::new(
                        (board.tick % 10) as f32 * frame_size,
                        0.0,
                        frame_size,
                        frame_size,
                    )),
                    pose.vx > 0.0,
                    1.0,
                    if fish.properties.invisible { 0.35 } else { 1.0 },
                );
                if session.virtual_tank.settings.show_names {
                    self.fonts["Pix118"].text(&fish.name, pose.x - 18.0, pose.y - 26.0, WHITE);
                }
            }
            for corpse in &board.corpses {
                // Physical state remains on the Board. This compact death
                // projection does not claim retail animation timing parity.
                let (image, size, row) = match corpse.kind {
                    crate::virtual_tank::VirtualFoodKind::Guppy => ("IMAGE_SMALLDIE", 80.0, 0.0),
                    crate::virtual_tank::VirtualFoodKind::Ultra => ("IMAGE_ULTRA", 160.0, 480.0),
                    _ => ("IMAGE_SMALLDIE", 80.0, 320.0),
                };
                self.sprite(
                    image,
                    corpse.x - size / 2.0,
                    corpse.y - size / 2.0,
                    Some(Rect::new(
                        f32::from(((125 - corpse.life) / 4).min(9)) * size,
                        row,
                        size,
                        size,
                    )),
                    false,
                    1.0,
                    (f32::from(corpse.life) / 30.0).min(1.0),
                );
            }
            for coin in &board.coins {
                if matches!(
                    coin.kind,
                    VirtualCoinKind::Pearl | VirtualCoinKind::NikoPearl
                ) {
                    self.sprite(
                        "IMAGE_PEARL",
                        coin.x - 20.0,
                        coin.y - 20.0,
                        None,
                        false,
                        1.0,
                        1.0,
                    );
                } else {
                    self.sprite(
                        "IMAGE_SHELLS",
                        coin.x - 16.0,
                        coin.y - 16.0,
                        Some(Rect::new(0.0, 0.0, 32.0, 32.0)),
                        false,
                        1.0,
                        1.0,
                    );
                }
            }
            for pet in &board.pets {
                let image_id = format!("IMAGE_SCL_{:?}", pet.active_form).to_uppercase();
                if let Some(image) = self.images.get(&image_id) {
                    let width = image.width() / 10.0;
                    let height = image.height();
                    self.sprite(
                        &image_id,
                        pet.x - width / 2.0,
                        pet.y - height / 2.0,
                        Some(Rect::new(
                            (board.tick % 10) as f32 * width,
                            0.0,
                            width,
                            height,
                        )),
                        false,
                        1.0,
                        1.0,
                    );
                }
                if let Some(glove) = pet.glove {
                    self.sprite(
                        "IMAGE_WALTER",
                        (glove.x + 40) as f32,
                        glove.y as f32,
                        Some(Rect::new(400.0, 160.0, 80.0, 80.0)),
                        glove.right,
                        1.0,
                        1.0,
                    );
                }
            }
            self.draw_virtual_combat(&board.combat.wave, &board.combat.missiles);
        }
        draw_rectangle(0.0, 0.0, 640.0, 70.0, Color::new(0.02, 0.08, 0.13, 0.94));
        self.centered_text(
            "JungleFever15outline",
            &format!("VIRTUAL TANK   SHELLS {}", session.progress.shell_balance),
            50.0,
            YELLOW,
        );
        self.main_button(Rect::new(16.0, 4.0, 110.0, 32.0), "Store");
        self.main_button(Rect::new(140.0, 4.0, 110.0, 32.0), "Setup");
        self.main_button(Rect::new(514.0, 4.0, 110.0, 32.0), "Menu");

        match &session.phase {
            AdventurePhase::VirtualStore | AdventurePhase::VirtualPurchaseConfirm { .. } => {
                draw_rectangle(10.0, 65.0, 620.0, 390.0, Color::new(0.02, 0.08, 0.14, 0.96));
                self.centered_text("JungleFever17outline", "DAILY STORE", 102.0, YELLOW);
                if let Some(stock) = &session.virtual_tank.stock {
                    for (slot, product) in stock.offers.iter().enumerate() {
                        let rect = virtual_offer_rect(slot);
                        draw_rectangle(
                            rect.x,
                            rect.y,
                            rect.w,
                            rect.h,
                            Color::new(0.08, 0.2, 0.27, 0.96),
                        );
                        let label = match product {
                            Some(StoreProduct::Fish(offer)) => {
                                format!("{:?}  {} shells", offer.kind, offer.price())
                            }
                            Some(other) => format!("{other:?}  {} shells", other.price()),
                            None => "Unavailable".to_owned(),
                        };
                        self.fonts["JungleFever10outline"].text(
                            &format!(
                                "{}: {}{}",
                                slot + 1,
                                label,
                                if stock.bought[slot] { "  SOLD" } else { "" }
                            ),
                            rect.x + 8.0,
                            rect.y + 45.0,
                            WHITE,
                        );
                    }
                }
                self.main_button(Rect::new(500.0, 445.0, 110.0, 32.0), "Back");
            }
            AdventurePhase::VirtualSetup
            | AdventurePhase::VirtualFishSaleConfirm { .. }
            | AdventurePhase::VirtualBackdropSaleConfirm
            | AdventurePhase::VirtualRename { .. } => {
                draw_rectangle(10.0, 65.0, 620.0, 390.0, Color::new(0.02, 0.08, 0.14, 0.96));
                self.fonts["JungleFever10outline"].text("Fish", 20.0, 94.0, YELLOW);
                for (index, fish) in session.virtual_tank.active_fish().enumerate() {
                    let x = 20.0 + (index / 10) as f32 * 145.0;
                    let y = 115.0 + (index % 10) as f32 * 27.0;
                    let short_name: String = fish.name.chars().take(10).collect();
                    self.fonts["Pix118"].text(
                        &format!(
                            "{} {} {}",
                            fish.id,
                            short_name,
                            if fish.shown { "on" } else { "off" }
                        ),
                        x,
                        y,
                        WHITE,
                    );
                }
                if let Some(id) = session.virtual_tank.selected_fish {
                    self.fonts["Pix118"].text(&format!("Selected {id}"), 20.0, 411.0, YELLOW);
                    self.main_button(Rect::new(20.0, 417.0, 94.0, 30.0), "Hide/Show");
                    self.main_button(Rect::new(120.0, 417.0, 82.0, 30.0), "Rename");
                    self.main_button(Rect::new(208.0, 417.0, 76.0, 30.0), "Sell");
                }
                self.fonts["JungleFever10outline"].text(
                    "Pets (tap to toggle)",
                    330.0,
                    93.0,
                    YELLOW,
                );
                for (index, pet) in session.progress.unlocked_pets.iter().enumerate() {
                    let x = 334.0 + (index % 3) as f32 * 95.0;
                    let y = 120.0 + (index / 3) as f32 * 30.0;
                    self.fonts["Pix118"].text(
                        &format!(
                            "{}{:?}",
                            if session.virtual_tank.selected_pets.contains(pet) {
                                "*"
                            } else {
                                " "
                            },
                            pet
                        ),
                        x,
                        y,
                        WHITE,
                    );
                }
                self.fonts["Pix118"].text(
                    &format!(
                        "Backdrop {}   Drop shells {}",
                        session.virtual_tank.backdrop, session.virtual_tank.settings.drop_shells
                    ),
                    330.0,
                    392.0,
                    WHITE,
                );
                self.main_button(Rect::new(330.0, 416.0, 50.0, 30.0), "<");
                self.main_button(Rect::new(385.0, 416.0, 50.0, 30.0), ">");
                self.main_button(Rect::new(440.0, 416.0, 65.0, 30.0), "Sell");
                self.main_button(Rect::new(510.0, 416.0, 90.0, 30.0), "Shells");
                self.main_button(Rect::new(515.0, 445.0, 100.0, 30.0), "Back");
            }
            _ => {}
        }
        if let AdventurePhase::VirtualPurchaseConfirm { price, name, .. } = &session.phase {
            draw_rectangle(85.0, 150.0, 470.0, 210.0, Color::new(0.02, 0.1, 0.15, 0.98));
            self.centered_text(
                "JungleFever15outline",
                &format!("Buy for {price} shells?"),
                195.0,
                YELLOW,
            );
            self.centered_text(
                "JungleFever10outline",
                &format!("Fish name: {name}"),
                239.0,
                WHITE,
            );
            self.main_button(Rect::new(140.0, 305.0, 150.0, 38.0), "Buy");
            self.main_button(Rect::new(350.0, 305.0, 150.0, 38.0), "Cancel");
        }
        if let AdventurePhase::VirtualRename { name, .. } = &session.phase {
            draw_rectangle(85.0, 150.0, 470.0, 210.0, Color::new(0.02, 0.1, 0.15, 0.98));
            self.centered_text("JungleFever15outline", "Rename fish", 195.0, YELLOW);
            self.centered_text("JungleFever10outline", name, 239.0, WHITE);
            self.main_button(Rect::new(140.0, 305.0, 150.0, 38.0), "Save");
            self.main_button(Rect::new(350.0, 305.0, 150.0, 38.0), "Cancel");
        }
        if matches!(
            session.phase,
            AdventurePhase::VirtualFishSaleConfirm { .. }
                | AdventurePhase::VirtualBackdropSaleConfirm
        ) {
            draw_rectangle(85.0, 150.0, 470.0, 210.0, Color::new(0.02, 0.1, 0.15, 0.98));
            self.centered_text("JungleFever15outline", "Confirm sale?", 195.0, YELLOW);
            self.main_button(Rect::new(140.0, 305.0, 150.0, 38.0), "Sell");
            self.main_button(Rect::new(350.0, 305.0, 150.0, 38.0), "Cancel");
        }
        if let AdventurePhase::VirtualPrestoDialog { pressed } = &session.phase {
            self.draw_presto_dialog(&session.progress.unlocked_pets, *pressed);
        }
    }

    fn draw_virtual_combat(
        &self,
        wave: &crate::invasion::Invasion1_2,
        missiles: &[crate::missile::ClassicMissile],
    ) {
        if wave.warning.is_some() {
            self.centered_text("JungleFever17outline", "DANGER!", 114.0, YELLOW);
        }
        for warp in &wave.warps {
            let frame = 17_i32 - i32::from(warp.remaining_ticks) / 2;
            if (0..17).contains(&frame) {
                self.sprite(
                    "IMAGE_WARPHOLE",
                    (warp.x + 20) as f32,
                    warp.y as f32,
                    Some(Rect::new(frame as f32 * 60.0, 0.0, 60.0, 220.0)),
                    false,
                    1.0,
                    1.0,
                );
                self.sprite(
                    "IMAGE_WARPGLOW",
                    warp.x as f32,
                    warp.y as f32,
                    Some(Rect::new(frame as f32 * 100.0, 0.0, 100.0, 220.0)),
                    false,
                    1.0,
                    0.7,
                );
            }
        }
        for alien in &wave.actors {
            if alien.spawn_ticks > 9 {
                continue;
            }
            let inset = f32::from(alien.spawn_ticks) * 16.0;
            let image = match alien.kind {
                SylvesterKind::Balrog => "IMAGE_BALROG",
                SylvesterKind::Gus => "IMAGE_GUS",
                SylvesterKind::Destructor => "IMAGE_DESTRUCTOR",
                SylvesterKind::Ulysses => "IMAGE_ULYSSES",
                SylvesterKind::Psychosquid => "IMAGE_PSYCHOSQUID",
                SylvesterKind::Cyrax => "IMAGE_BOSS",
                SylvesterKind::MiniSylvester => "IMAGE_MINISYLV",
                SylvesterKind::Weak | SylvesterKind::Strong => "IMAGE_SYLV",
            };
            self.sprite(
                image,
                alien.widget_x as f32 + inset / 2.0,
                alien.widget_y as f32 + inset / 2.0,
                Some(Rect::new(
                    f32::from(alien.sprite_frame()) * 160.0,
                    f32::from(alien.sprite_row()) * 160.0,
                    160.0,
                    160.0,
                )),
                alien.facing_right(),
                (160.0 - inset) / 160.0,
                1.0,
            );
        }
        for group in &wave.bilaterus {
            for index in 0..2 {
                if let (Some(head), Some(scale), Some((row, frame, facing_right))) = (
                    group.heads[index].as_ref(),
                    group.head_emergence_scale(index),
                    group.head_sprite_pose(index),
                ) {
                    let inset = 80.0 * (1.0 - scale) / 2.0;
                    self.sprite(
                        "IMAGE_BILATERUS",
                        head.widget_x as f32 + inset,
                        head.widget_y as f32 + inset,
                        Some(Rect::new(
                            f32::from(frame) * 80.0,
                            f32::from(row) * 80.0,
                            80.0,
                            80.0,
                        )),
                        facing_right,
                        scale,
                        1.0,
                    );
                }
            }
            if group.emergence_ticks == 0 {
                for index in (0..group.bones.len()).rev() {
                    let bone = &group.bones[index];
                    let predecessor_x = if index == 0 {
                        group.active().x
                    } else {
                        group.bones[index - 1].x
                    };
                    self.sprite(
                        "IMAGE_BILATERUS",
                        bone.widget_x as f32,
                        bone.widget_y as f32,
                        Some(Rect::new(
                            f32::from(bone.sprite_frame(predecessor_x)) * 80.0,
                            f32::from(crate::bilaterus::BilaterusBone::sprite_row(index)) * 80.0,
                            80.0,
                            80.0,
                        )),
                        false,
                        1.0,
                        1.0,
                    );
                }
            }
        }
        for fragment in &wave.fragments {
            self.sprite(
                "IMAGE_BILATERUS",
                fragment.widget_x as f32,
                fragment.widget_y as f32,
                Some(Rect::new(
                    f32::from(fragment.sprite_frame()) * 80.0,
                    f32::from(fragment.sprite_row()) * 80.0,
                    80.0,
                    80.0,
                )),
                fragment.kind != crate::bilaterus::FragmentKind::Bone && fragment.facing_right(),
                1.0,
                1.0,
            );
        }
        for missile in missiles {
            let (image, scale) = match missile.kind {
                MissileKind::Classic | MissileKind::EnergyBall => ("IMAGE_MISSILE", 1.0),
                MissileKind::Stanley => ("IMAGE_MISSILE", 0.625),
            };
            self.sprite(
                image,
                missile.widget_x as f32,
                missile.widget_y as f32,
                Some(Rect::new(f32::from(missile.frame) * 80.0, 0.0, 80.0, 80.0)),
                false,
                scale,
                1.0,
            );
        }
        for laser in &wave.lasers {
            let frame = laser.frame();
            if frame < 10 {
                for row in 0..2 {
                    self.sprite(
                        "IMAGE_LASERS",
                        laser.x as f32,
                        laser.y as f32,
                        Some(Rect::new(
                            f32::from(frame) * 80.0,
                            row as f32 * 80.0,
                            80.0,
                            80.0,
                        )),
                        false,
                        1.0,
                        1.0,
                    );
                }
            }
        }
        for body in &wave.dead_aliens {
            let image = match body.kind {
                SylvesterKind::Balrog => "IMAGE_BALROG",
                SylvesterKind::Gus => "IMAGE_GUS",
                SylvesterKind::Destructor => "IMAGE_DESTRUCTOR",
                SylvesterKind::Ulysses => "IMAGE_ULYSSES",
                SylvesterKind::Psychosquid => "IMAGE_PSYCHOSQUID",
                SylvesterKind::Cyrax => "IMAGE_BOSS",
                SylvesterKind::MiniSylvester => "IMAGE_MINISYLV",
                SylvesterKind::Weak | SylvesterKind::Strong => "IMAGE_SYLV",
            };
            self.sprite(
                image,
                body.widget_x as f32,
                body.widget_y as f32,
                Some(Rect::new(f32::from(body.frame) * 160.0, 0.0, 160.0, 160.0)),
                body.facing_right,
                1.0,
                body.opacity,
            );
        }
    }

    fn draw_presto_dialog(&self, unlocked_pets: &[PetKind], pressed: Option<PetKind>) {
        draw_rectangle(0.0, 0.0, 640.0, 480.0, Color::new(0.0, 0.0, 0.0, 0.82));
        self.centered_text("JungleFever17outline", "Choose Presto's form", 48.0, WHITE);
        for pet in unlocked_pets {
            if crate::time_trial::selectable_pet(*pet) {
                let rect = presto_choice_rect(*pet as usize);
                self.main_button(rect, &format!("{pet:?}"));
                if pressed == Some(*pet) {
                    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, YELLOW);
                }
            }
        }
        self.centered_text(
            "JungleFever10outline",
            "Press and release the same form. Esc cancels.",
            440.0,
            WHITE,
        );
    }

    fn draw(
        &self,
        session: &AdventureSession,
        paused: bool,
        pointer: Vec2,
        healing_warning_until_tick: Option<u64>,
        interlude: Option<&InterludeController>,
    ) {
        match session.phase {
            AdventurePhase::VirtualTank
            | AdventurePhase::VirtualStore
            | AdventurePhase::VirtualPurchaseConfirm { .. }
            | AdventurePhase::VirtualSetup
            | AdventurePhase::VirtualFishSaleConfirm { .. }
            | AdventurePhase::VirtualBackdropSaleConfirm
            | AdventurePhase::VirtualRename { .. }
            | AdventurePhase::VirtualPrestoDialog { .. } => self.draw_virtual_tank(session),
            AdventurePhase::Hatch { pet, updates } => self.draw_hatch(pet, updates),
            AdventurePhase::TankFourFinaleHatch { updates } => self.draw_finale_hatch(updates),
            AdventurePhase::AdventureFinaleInterlude => {
                self.draw_adventure_finale_interlude(interlude)
            }
            AdventurePhase::Bonus { ref state } => self.draw_bonus(state),
            AdventurePhase::BonusResults { ref result } => {
                self.draw_bonus_results(result, session.progress.shell_balance)
            }
            AdventurePhase::PetSelection { ref selected }
            | AdventurePhase::PetSelectionConfirmation { ref selected }
            | AdventurePhase::TimeTrialPetSelection { ref selected, .. }
            | AdventurePhase::ChallengePetSelection { ref selected, .. }
            | AdventurePhase::ChallengePetSelectionConfirmation { ref selected, .. } => {
                self.draw_pet_selection(session, selected, pointer);
            }
            AdventurePhase::SandboxPlaying
            | AdventurePhase::SandboxPrestoDialog { .. }
            | AdventurePhase::Playing
            | AdventurePhase::TimeTrialPlaying
            | AdventurePhase::TimeTrialPrestoDialog { .. }
            | AdventurePhase::AdventurePrestoDialog { .. }
            | AdventurePhase::TimeTrialInvasionTutorial { .. }
            | AdventurePhase::TimeTrialTimesUp
            | AdventurePhase::TimeTrialResults
            | AdventurePhase::TimeTrialGameOver { .. }
            | AdventurePhase::ChallengePlaying
            | AdventurePhase::ChallengePrestoDialog { .. }
            | AdventurePhase::ChallengeInvasionTutorial { .. }
            | AdventurePhase::ChallengeResults
            | AdventurePhase::ChallengeLost
            | AdventurePhase::ChallengeInterlude { .. }
            | AdventurePhase::FirstTankRescue
            | AdventurePhase::InvasionTutorial { .. }
            | AdventurePhase::GameOver { .. } => {
                if let Some(board) = if matches!(
                    session.phase,
                    AdventurePhase::SandboxPlaying | AdventurePhase::SandboxPrestoDialog { .. }
                ) {
                    session.sandbox_board.as_ref()
                } else {
                    session.board.as_ref()
                } {
                    self.draw_board(board);
                    if session.phase == AdventurePhase::SandboxPlaying {
                        draw_text(
                            "SANDBOX: keys spawn fish, pets, aliens; egg exits",
                            70.0,
                            430.0,
                            18.0,
                            YELLOW,
                        );
                    }
                    if let AdventurePhase::TimeTrialPrestoDialog { pressed, .. }
                    | AdventurePhase::AdventurePrestoDialog { pressed, .. }
                    | AdventurePhase::ChallengePrestoDialog { pressed, .. }
                    | AdventurePhase::SandboxPrestoDialog { pressed, .. } = session.phase
                    {
                        self.draw_presto_dialog(&session.progress.unlocked_pets, pressed);
                    }
                    if board.time_trial {
                        let limit = crate::time_trial::limit_seconds(board.tank).unwrap_or(0);
                        let left = limit.saturating_sub(board.tick.saturating_mul(28) / 1000);
                        self.centered_text(
                            "JungleFever10outline",
                            &format!(
                                "TIME {:02}:{:02}   PET EGG ${}{}",
                                left / 60,
                                left % 60,
                                board.egg_price,
                                if session.time_trial.as_ref().is_some_and(|run| run.egg_maxed) {
                                    " MAX"
                                } else {
                                    ""
                                }
                            ),
                            422.0,
                            YELLOW,
                        );
                    }
                    if board.challenge_shop.is_some() {
                        self.centered_text(
                            "JungleFever10outline",
                            &format!(
                                "CHALLENGE {:02}:{:02}   EGG ${}",
                                crate::challenge::default_score_seconds(board.tick) / 60,
                                crate::challenge::default_score_seconds(board.tick) % 60,
                                board.egg_price
                            ),
                            422.0,
                            YELLOW,
                        );
                    }
                }
            }
            AdventurePhase::GameSelector
            | AdventurePhase::HelpScreen
            | AdventurePhase::TimeTrialTankSelection
            | AdventurePhase::ChallengeTankSelection => {
                self.sprite("IMAGE_HATCHSCREEN", 0.0, 0.0, None, false, 1.0, 1.0);
            }
        }
        if matches!(session.phase, AdventurePhase::Playing)
            && let (Some(board), Some(until)) = (&session.board, healing_warning_until_tick)
        {
            let remaining = until.saturating_sub(board.tick);
            if remaining > 0 && remaining % 32 < 27 {
                let message = "Stop shooting! Alien regains health!";
                let width = self.fonts["ContinuumBold14"]
                    .metrics
                    .measure_width(message)
                    .unwrap_or(0) as f32;
                let x = 90.0 + (460.0 - width) / 2.0 + 2.0;
                self.fonts["ContinuumBold14outback"].text(
                    message,
                    x,
                    462.0,
                    Color::from_rgba(0, 75, 0, 255),
                );
                self.fonts["ContinuumBold14"].text(
                    message,
                    x,
                    462.0,
                    Color::from_rgba(180, 250, 90, 255),
                );
            }
        }
        if matches!(
            session.phase,
            AdventurePhase::Playing | AdventurePhase::TimeTrialPlaying
        ) && session.board.as_ref().is_some_and(|board| {
            board
                .invasion
                .as_ref()
                .is_some_and(|wave| wave.sneeze_shake_ticks > 0)
        }) {
            // Project presentation: the retail message lifetime is not yet
            // established, so the saved source-owned shake clock bounds it.
            self.centered_text(
                "ContinuumBold14outback",
                "Attack Postponed by Sneeze of Power!",
                462.0,
                YELLOW,
            );
        }
        if matches!(
            session.phase,
            AdventurePhase::PetSelectionConfirmation { .. }
                | AdventurePhase::ChallengePetSelectionConfirmation { .. }
        ) {
            draw_rectangle(
                110.0,
                150.0,
                420.0,
                210.0,
                Color::new(0.02, 0.1, 0.15, 0.96),
            );
            self.centered_text("JungleFever15outline", "Select More Pets?", 194.0, YELLOW);
            self.centered_text(
                "JungleFever10outline",
                "Continue with fewer than 3 pets?",
                232.0,
                WHITE,
            );
            let height = self.images["IMAGE_MAINBUTTON"].height();
            self.main_button(Rect::new(155.0, 300.0, 145.0, height), "Yes");
            self.main_button(Rect::new(340.0, 300.0, 145.0, height), "No");
        }
        if session.phase == AdventurePhase::FirstTankRescue {
            draw_rectangle(90.0, 150.0, 460.0, 200.0, Color::new(0.02, 0.1, 0.15, 0.96));
            self.centered_text(
                "JungleFever15outline",
                "YOUR LAST FISH HAS DIED!",
                186.0,
                YELLOW,
            );
            self.centered_text(
                "JungleFever10outline",
                "We'll give you another fish to keep playing.",
                225.0,
                WHITE,
            );
            self.centered_text(
                "JungleFever10outline",
                "Make sure you keep it well fed!",
                251.0,
                WHITE,
            );
            self.main_button(
                Rect::new(
                    186.0,
                    295.0,
                    264.0,
                    self.images["IMAGE_MAINBUTTON"].height(),
                ),
                "Click to Continue",
            );
        }
        if let AdventurePhase::InvasionTutorial { tip }
        | AdventurePhase::ChallengeInvasionTutorial { tip }
        | AdventurePhase::TimeTrialInvasionTutorial { tip } = session.phase
        {
            draw_rectangle(72.0, 123.0, 496.0, 252.0, Color::new(0.02, 0.1, 0.15, 0.96));
            let (title, lines) = match tip {
                InvasionTip::Danger => (
                    "DANGER!",
                    [
                        "Aliens are approaching your tank!",
                        "Click on them to defend your fish.",
                    ],
                ),
                InvasionTip::BattleTip => (
                    "ALIEN ATTACK!",
                    [
                        "Click an alien repeatedly to defeat it.",
                        "Keep your fish safe!",
                    ],
                ),
                InvasionTip::GusWarning => (
                    "WARNING!",
                    [
                        "Lasers cannot hurt this alien.",
                        "Click in the tank to feed it instead.",
                    ],
                ),
            };
            self.centered_text("JungleFever17outline", title, 174.0, YELLOW);
            for (index, line) in lines.iter().enumerate() {
                self.centered_text(
                    "JungleFever10outline",
                    line,
                    220.0 + index as f32 * 28.0,
                    WHITE,
                );
            }
            self.main_button(
                Rect::new(
                    186.0,
                    310.0,
                    264.0,
                    self.images["IMAGE_MAINBUTTON"].height(),
                ),
                "Click to Continue",
            );
        }
        if let AdventurePhase::GameOver { updates } = session.phase {
            draw_rectangle(72.0, 123.0, 496.0, 252.0, Color::new(0.02, 0.1, 0.15, 0.96));
            self.centered_text("JungleFever17outline", "GAME OVER", 182.0, YELLOW);
            if updates > 30 {
                self.main_button(
                    Rect::new(
                        186.0,
                        310.0,
                        264.0,
                        self.images["IMAGE_MAINBUTTON"].height(),
                    ),
                    "Click to Continue",
                );
            }
        }
        if session.phase == AdventurePhase::GameSelector {
            self.centered_text("JungleFever17outline", "INSANIQUARIUM", 112.0, YELLOW);
            self.main_button(
                Rect::new(
                    186.0,
                    250.0,
                    264.0,
                    self.images["IMAGE_MAINBUTTON"].height(),
                ),
                "Adventure",
            );
            if session.progress.tank >= 2 || session.progress.adventure_completed {
                self.main_button(
                    Rect::new(
                        186.0,
                        310.0,
                        264.0,
                        self.images["IMAGE_MAINBUTTON"].height(),
                    ),
                    "Time Trial",
                );
            }
            if session.progress.adventure_completed {
                self.main_button(
                    Rect::new(
                        186.0,
                        370.0,
                        264.0,
                        self.images["IMAGE_MAINBUTTON"].height(),
                    ),
                    "Challenge",
                );
            }
            self.main_button(
                Rect::new(
                    186.0,
                    430.0,
                    264.0,
                    self.images["IMAGE_MAINBUTTON"].height(),
                ),
                "Virtual Tank",
            );
        }
        if matches!(
            session.phase,
            AdventurePhase::TimeTrialTankSelection | AdventurePhase::ChallengeTankSelection
        ) {
            self.centered_text(
                "JungleFever17outline",
                if session.mode == crate::adventure::GameMode::Challenge {
                    "CHALLENGE"
                } else {
                    "TIME TRIAL"
                },
                105.0,
                YELLOW,
            );
            for tank in 1..=4 {
                if session.mode == crate::adventure::GameMode::Challenge
                    && tank > 1
                    && !session.progress.challenge_completed[tank as usize - 2]
                {
                    continue;
                }
                let y = 150.0 + (tank - 1) as f32 * 66.0;
                self.main_button(
                    Rect::new(186.0, y, 264.0, self.images["IMAGE_MAINBUTTON"].height()),
                    &format!("Tank {tank}"),
                );
            }
            self.main_button(
                Rect::new(525.0, 4.0, 80.0, self.images["IMAGE_MAINBUTTON"].height()),
                "Menu",
            );
        }
        if session.phase == AdventurePhase::ChallengeResults {
            draw_rectangle(72.0, 123.0, 496.0, 252.0, Color::new(0.02, 0.1, 0.15, 0.96));
            self.centered_text("JungleFever17outline", "CHALLENGE COMPLETE", 170.0, YELLOW);
            if let Some(result) = session
                .challenge
                .as_ref()
                .and_then(|run| run.result.as_ref())
            {
                self.centered_text(
                    "JungleFever10outline",
                    &format!(
                        "Tank {}  Time {}s  Best {}s  Shells +{}",
                        result.tank, result.seconds, result.personal_best, result.credited_shells
                    ),
                    225.0,
                    WHITE,
                );
                self.draw_results_purchase(
                    &result.purchase,
                    result.updates,
                    session.progress.shell_balance,
                );
            }
            self.main_button(
                results_continue_rect(self.images["IMAGE_MAINBUTTON"].height()),
                "Selector",
            );
            self.main_button(
                Rect::new(
                    186.0,
                    365.0,
                    264.0,
                    self.images["IMAGE_MAINBUTTON"].height(),
                ),
                "Story",
            );
        }
        if session.phase == AdventurePhase::ChallengeLost {
            draw_rectangle(72.0, 123.0, 496.0, 252.0, Color::new(0.02, 0.1, 0.15, 0.96));
            self.centered_text("JungleFever17outline", "GAME OVER", 182.0, YELLOW);
            self.main_button(
                Rect::new(
                    186.0,
                    310.0,
                    264.0,
                    self.images["IMAGE_MAINBUTTON"].height(),
                ),
                "Continue",
            );
        }
        if let AdventurePhase::ChallengeInterlude { tank } = session.phase {
            draw_rectangle(72.0, 123.0, 496.0, 252.0, Color::new(0.02, 0.1, 0.15, 0.96));
            self.centered_text(
                "JungleFever17outline",
                &format!("TANK {tank} COMPLETE"),
                180.0,
                YELLOW,
            );
            self.main_button(
                Rect::new(
                    186.0,
                    310.0,
                    264.0,
                    self.images["IMAGE_MAINBUTTON"].height(),
                ),
                "Menu",
            );
        }
        if session.phase == AdventurePhase::TimeTrialTimesUp {
            draw_rectangle(72.0, 123.0, 496.0, 252.0, Color::new(0.02, 0.1, 0.15, 0.96));
            self.centered_text("JungleFever17outline", "TIME'S UP!", 180.0, YELLOW);
            if let Some(result) = session
                .time_trial
                .as_ref()
                .and_then(|run| run.result.as_ref())
            {
                self.centered_text(
                    "JungleFever10outline",
                    &format!(
                        "Score ${}   Shells +{}",
                        result.score, result.credited_shells
                    ),
                    235.0,
                    WHITE,
                );
            }
            self.main_button(
                Rect::new(
                    186.0,
                    310.0,
                    264.0,
                    self.images["IMAGE_MAINBUTTON"].height(),
                ),
                "See Results",
            );
        }
        if session.phase == AdventurePhase::TimeTrialResults {
            draw_rectangle(72.0, 123.0, 496.0, 252.0, Color::new(0.02, 0.1, 0.15, 0.96));
            self.centered_text("JungleFever17outline", "TIME TRIAL RESULTS", 180.0, YELLOW);
            if let Some(result) = session
                .time_trial
                .as_ref()
                .and_then(|run| run.result.as_ref())
            {
                self.centered_text(
                    "JungleFever10outline",
                    &format!(
                        "Tank {}  Score ${}  Best ${}",
                        result.tank, result.score, result.personal_best
                    ),
                    235.0,
                    WHITE,
                );
                self.draw_results_purchase(
                    &result.purchase,
                    result.updates,
                    session.progress.shell_balance,
                );
            }
            self.main_button(
                results_continue_rect(self.images["IMAGE_MAINBUTTON"].height()),
                "Continue",
            );
        }
        if let AdventurePhase::TimeTrialGameOver { updates } = session.phase {
            draw_rectangle(72.0, 123.0, 496.0, 252.0, Color::new(0.02, 0.1, 0.15, 0.96));
            self.centered_text("JungleFever17outline", "GAME OVER", 182.0, YELLOW);
            if updates > 30 {
                self.main_button(
                    Rect::new(
                        186.0,
                        310.0,
                        264.0,
                        self.images["IMAGE_MAINBUTTON"].height(),
                    ),
                    "Continue",
                );
            }
        }
        if session.phase == AdventurePhase::HelpScreen {
            self.centered_text("JungleFever17outline", "ADVENTURE", 112.0, YELLOW);
            self.centered_text(
                "JungleFever10outline",
                "Feed your fish, collect coins, and defend the tank.",
                210.0,
                WHITE,
            );
            self.main_button(
                Rect::new(
                    186.0,
                    310.0,
                    264.0,
                    self.images["IMAGE_MAINBUTTON"].height(),
                ),
                "Start",
            );
        }
        if paused {
            draw_rectangle(
                150.0,
                170.0,
                340.0,
                130.0,
                Color::new(0.02, 0.1, 0.15, 0.94),
            );
            draw_text("Paused", 266.0, 207.0, 28.0, WHITE);
            draw_text("Esc: resume    S: save", 203.0, 247.0, 21.0, WHITE);
            draw_text("Q: save and exit", 235.0, 277.0, 21.0, WHITE);
        }
    }
}

fn pcm_wav(sample_rate: u32, samples: &[i16]) -> Vec<u8> {
    let data_bytes = samples.len() as u32 * 2;
    let mut bytes = Vec::with_capacity(data_bytes as usize + 44);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(data_bytes + 36).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    bytes.extend_from_slice(&2_u16.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_bytes.to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    bytes
}

#[derive(Clone, Copy)]
struct EventStamp {
    session_tick: u64,
    wall_elapsed_seconds: f64,
    observation_context: &'static str,
}

fn stamp_events(
    stamps: &mut Vec<EventStamp>,
    events: &[Event],
    session_tick: u64,
    wall_elapsed_seconds: f64,
    observation_context: &'static str,
) {
    stamps.extend(events.iter().map(|_| EventStamp {
        session_tick,
        wall_elapsed_seconds,
        observation_context,
    }));
}

struct Evidence {
    root: PathBuf,
    events: BufWriter<File>,
    last_state_tick: u64,
    snapshot_deferred: bool,
    last_paused: bool,
    test_speed: u8,
    start_session_tick: u64,
}

impl Evidence {
    fn open(
        root: &Path,
        identity: &InstallIdentity,
        session: &AdventureSession,
        seed: u64,
        test_speed: u8,
    ) -> Result<Self, Box<dyn Error>> {
        fs::create_dir_all(root)?;
        let time_mode = if test_speed == 1 {
            "normal"
        } else {
            "accelerated-test"
        };
        cli::write_json(
            &root.join("identity.local.json"),
            &serde_json::json!({"game": identity, "rust_executable_sha256": install::sha256(&fs::read(std::env::current_exe()?)?), "seed": seed, "tick_ms": TICK_MS, "test_speed": test_speed, "time_mode": time_mode, "session_elapsed_seconds": 0.0, "session_elapsed_policy": "run-start session tick delta * 28 ms, including paused session updates", "wall_elapsed_policy": "wall time observed after each accelerated step or accepted-actions boundary; state and music observed after the physical frame", "hold_input_clock": if test_speed == 1 { "wall time at each consumed step" } else { "28 ms per consumed unpaused step; press origin is the last consumed step" }, "connector_observation_policy": "once per actual rendered frame after the update batch, including pause; accelerated traces may differ", "retail_rng_equivalent": false, "start": session}),
        )?;
        Ok(Self {
            root: root.into(),
            events: BufWriter::new(File::create(root.join("events.local.jsonl"))?),
            last_state_tick: u64::MAX,
            snapshot_deferred: false,
            last_paused: false,
            test_speed,
            start_session_tick: session.ticks,
        })
    }
    fn time_mode(&self) -> &'static str {
        if self.test_speed == 1 {
            "normal"
        } else {
            "accelerated-test"
        }
    }
    fn session_elapsed_seconds(&self, session: &AdventureSession) -> f64 {
        session.ticks.saturating_sub(self.start_session_tick) as f64 * f64::from(TICK_MS) / 1000.0
    }
    fn record(
        &mut self,
        events: &[Event],
        event_stamps: &[EventStamp],
        session: &AdventureSession,
        elapsed: f64,
        paused: bool,
    ) -> Result<(), Box<dyn Error>> {
        let session_elapsed_seconds = self.session_elapsed_seconds(session);
        let time_mode = self.time_mode();
        let test_speed = self.test_speed;
        Self::write_event_rows(
            &mut self.events,
            events,
            event_stamps,
            session.ticks,
            self.start_session_tick,
            elapsed,
            test_speed,
        )?;
        self.events.flush()?;
        if self.last_state_tick != session.ticks || self.last_paused != paused || !events.is_empty()
        {
            let publication = cli::write_json(
                &self.root.join("state.local.json"),
                &serde_json::json!({"elapsed_seconds":elapsed, "session_elapsed_seconds":session_elapsed_seconds, "test_speed":test_speed, "time_mode":time_mode, "paused":paused, "session_tick":session.ticks, "phase":session.phase, "progress":session.progress, "state":session.board}),
            );
            match publication {
                Ok(()) => {
                    if self.snapshot_deferred {
                        serde_json::to_writer(
                            &mut self.events,
                            &serde_json::json!({"diagnostic":"snapshot_publication_recovered","tick":session.ticks,"elapsed_seconds":elapsed,"session_elapsed_seconds":session_elapsed_seconds,"test_speed":test_speed,"time_mode":time_mode}),
                        )?;
                        writeln!(&mut self.events)?;
                        self.events.flush()?;
                    }
                    self.last_state_tick = session.ticks;
                    self.snapshot_deferred = false;
                    self.last_paused = paused;
                }
                Err(error)
                    if error
                        .downcast_ref::<std::io::Error>()
                        .is_some_and(|failure| {
                            cfg!(windows) && matches!(failure.raw_os_error(), Some(5 | 32))
                        }) =>
                {
                    // Ordinary Windows readers may deny replacement. Preserve the
                    // last complete snapshot and retry the latest state next frame.
                    // Optional observation must not abort the game or block a tick.
                    if !self.snapshot_deferred {
                        serde_json::to_writer(
                            &mut self.events,
                            &serde_json::json!({"diagnostic":"snapshot_publication_deferred","tick":session.ticks,"elapsed_seconds":elapsed,"session_elapsed_seconds":session_elapsed_seconds,"test_speed":test_speed,"time_mode":time_mode,"error":error.to_string()}),
                        )?;
                        writeln!(&mut self.events)?;
                        self.events.flush()?;
                    }
                    self.snapshot_deferred = true;
                }
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }
    fn write_event_rows(
        writer: &mut impl Write,
        events: &[Event],
        event_stamps: &[EventStamp],
        frame_session_tick: u64,
        start_session_tick: u64,
        frame_wall_elapsed: f64,
        test_speed: u8,
    ) -> Result<(), Box<dyn Error>> {
        if test_speed > 1 && events.len() != event_stamps.len() {
            return Err("Accelerated event timing lost its event-to-step mapping".into());
        }
        for (index, event) in events.iter().enumerate() {
            let stamp = if test_speed > 1 {
                event_stamps[index]
            } else {
                EventStamp {
                    session_tick: frame_session_tick,
                    wall_elapsed_seconds: frame_wall_elapsed,
                    observation_context: "physical_frame",
                }
            };
            let session_elapsed_seconds =
                stamp.session_tick.saturating_sub(start_session_tick) as f64 * f64::from(TICK_MS)
                    / 1000.0;
            let time_mode = if test_speed == 1 {
                "normal"
            } else {
                "accelerated-test"
            };
            serde_json::to_writer(
                &mut *writer,
                &serde_json::json!({"elapsed_seconds":stamp.wall_elapsed_seconds,"session_elapsed_seconds":session_elapsed_seconds,"test_speed":test_speed,"time_mode":time_mode,"session_tick":stamp.session_tick,"observation_context":stamp.observation_context,"event":event}),
            )?;
            writeln!(writer)?;
        }
        Ok(())
    }
    fn capture(&self, tick: u64) {
        let path = self.root.join(format!("frame-{tick:06}.png"));
        get_screen_data().export_png(&path.to_string_lossy());
    }

    fn record_music(
        &mut self,
        reports: &[MusicReport],
        elapsed: f64,
        session: &AdventureSession,
    ) -> Result<(), Box<dyn Error>> {
        let session_elapsed_seconds = self.session_elapsed_seconds(session);
        let time_mode = self.time_mode();
        let test_speed = self.test_speed;
        for report in reports {
            serde_json::to_writer(
                &mut self.events,
                &serde_json::json!({"elapsed_seconds": elapsed, "session_elapsed_seconds":session_elapsed_seconds,"test_speed":test_speed,"time_mode":time_mode,"session_tick":session.ticks,"observation_context":"physical_frame", "music": report}),
            )?;
            writeln!(&mut self.events)?;
        }
        self.events.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod accelerated_event_timing_tests {
    use super::*;

    #[test]
    fn early_batch_and_no_step_exit_events_keep_their_own_session_time() {
        let mut session = AdventureSession::new(17);
        let mut events = Vec::new();
        let mut stamps = Vec::new();
        for step in 1..=5 {
            let step_events = if step == 1 {
                session.step(&[Action::BuyEgg])
            } else {
                session.step(&[])
            };
            stamp_events(&mut stamps, &step_events, session.ticks, 1.0, "step");
            events.extend(step_events);
        }
        let accepted_on_exit = session.apply_actions(&[Action::BuyEgg]);
        stamp_events(
            &mut stamps,
            &accepted_on_exit,
            session.ticks,
            1.1,
            "accepted_actions",
        );
        events.extend(accepted_on_exit);
        assert!(events.len() > 2);

        let mut written = Vec::new();
        Evidence::write_event_rows(&mut written, &events, &stamps, session.ticks, 0, 2.0, 8)
            .unwrap();
        let rows: Vec<serde_json::Value> = std::str::from_utf8(&written)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(rows.len(), events.len());
        assert_eq!(rows[0]["session_tick"], 1);
        assert_eq!(rows[0]["session_elapsed_seconds"], 0.028);
        assert_eq!(rows[0]["elapsed_seconds"], 1.0);
        assert_eq!(rows[0]["observation_context"], "step");
        assert_eq!(rows.last().unwrap()["session_tick"], 5);
        assert_eq!(rows.last().unwrap()["session_elapsed_seconds"], 0.14);
        assert_eq!(rows.last().unwrap()["elapsed_seconds"], 1.1);
        assert_eq!(
            rows.last().unwrap()["observation_context"],
            "accepted_actions"
        );

        let mut normal_written = Vec::new();
        Evidence::write_event_rows(&mut normal_written, &events, &[], session.ticks, 0, 2.0, 1)
            .unwrap();
        let normal_first: serde_json::Value = serde_json::from_str(
            std::str::from_utf8(&normal_written)
                .unwrap()
                .lines()
                .next()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(normal_first["session_tick"], 5);
        assert_eq!(normal_first["elapsed_seconds"], 2.0);
        assert!(
            Evidence::write_event_rows(&mut Vec::new(), &events, &[], session.ticks, 0, 2.0, 8)
                .is_err()
        );
    }
}

struct RuntimeSettingsBackend<'a> {
    audio: Option<&'a MusicOwner>,
}

impl SettingsBackend for RuntimeSettingsBackend<'_> {
    fn capabilities(&self) -> SettingsCapabilities {
        SettingsCapabilities {
            music_volume: self.audio.is_some(),
            effects_volume: self.audio.is_some(),
            fullscreen: true,
            custom_cursor: true,
        }
    }

    fn set_music_volume(&mut self, value: f32) -> Result<(), String> {
        self.audio
            .ok_or("music output unavailable")?
            .set_music_volume(value)
    }

    fn set_effects_volume(&mut self, value: f32) -> Result<(), String> {
        self.audio
            .ok_or("effect output unavailable")?
            .set_effects_volume(value)
    }

    fn set_fullscreen(&mut self, value: bool) -> Result<(), String> {
        macroquad::window::set_fullscreen(value);
        Ok(())
    }

    fn set_custom_cursor(&mut self, value: bool) -> Result<(), String> {
        show_mouse(!value);
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SelectorPanel {
    Profiles,
    Options,
    Scores,
    Help,
}

#[derive(Clone, Copy)]
enum ProfileForm {
    Create,
    Rename(u64),
    Delete(u64),
}

enum UiCommand {
    Create(String),
    Select(u64),
    Rename(u64, String),
    Delete(u64),
    Setting(SettingChange),
    Quit,
}

struct SelectorUi {
    panel: Option<SelectorPanel>,
    form: Option<ProfileForm>,
    name: String,
    page: usize,
    score_tab: ScoreTab,
    score_focus: Option<usize>,
    error: String,
}

impl Default for SelectorUi {
    fn default() -> Self {
        Self {
            panel: None,
            form: None,
            name: String::new(),
            page: 0,
            score_tab: ScoreTab::Adventure,
            score_focus: None,
            error: String::new(),
        }
    }
}

fn ui_button(rect: Rect, label: &str) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.05, 0.2, 0.26, 0.96),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, YELLOW);
    draw_text(label, rect.x + 8.0, rect.y + rect.h * 0.7, 18.0, WHITE);
}

fn typed_profile_name(name: &mut String) {
    if is_key_pressed(KeyCode::Backspace) {
        name.pop();
    }
    while let Some(character) = get_char_pressed() {
        if !character.is_control() && name.len() + character.len_utf8() <= 64 {
            name.push(character);
        }
    }
}

impl SelectorUi {
    /// Returns whether this frame's selector input belongs to UI and any
    /// persistent action to commit before borrowing the live session.
    fn input(
        &mut self,
        profiles: &Profiles,
        settings: GameSettings,
        pointer: Vec2,
        selector_open: bool,
    ) -> (bool, Option<UiCommand>) {
        if !selector_open {
            self.panel = None;
            self.form = None;
            return (false, None);
        }
        let click = is_mouse_button_pressed(MouseButton::Left);
        if let Some(panel) = self.panel {
            if let Some(form) = self.form {
                if matches!(form, ProfileForm::Create | ProfileForm::Rename(_)) {
                    typed_profile_name(&mut self.name);
                }
                if is_key_pressed(KeyCode::Escape)
                    || (click && Rect::new(350.0, 310.0, 145.0, 38.0).contains(pointer))
                {
                    self.form = None;
                    self.error.clear();
                    return (true, None);
                }
                let confirm = is_key_pressed(KeyCode::Enter)
                    || (click && Rect::new(145.0, 310.0, 145.0, 38.0).contains(pointer));
                if confirm {
                    self.form = None;
                    return (
                        true,
                        Some(match form {
                            ProfileForm::Create => UiCommand::Create(self.name.clone()),
                            ProfileForm::Rename(id) => UiCommand::Rename(id, self.name.clone()),
                            ProfileForm::Delete(id) => UiCommand::Delete(id),
                        }),
                    );
                }
                return (true, None);
            }
            if is_key_pressed(KeyCode::Escape)
                || (click && Rect::new(510.0, 73.0, 65.0, 30.0).contains(pointer))
            {
                if panel == SelectorPanel::Scores && self.score_focus.is_some() {
                    self.score_focus = None;
                } else {
                    self.panel = None;
                }
                self.error.clear();
                return (true, None);
            }
            if click {
                match panel {
                    SelectorPanel::Profiles => {
                        for (row, entry) in profiles
                            .entries
                            .iter()
                            .skip(self.page * 6)
                            .take(6)
                            .enumerate()
                        {
                            if Rect::new(95.0, 132.0 + row as f32 * 36.0, 440.0, 32.0)
                                .contains(pointer)
                            {
                                self.panel = None;
                                return (true, Some(UiCommand::Select(entry.id)));
                            }
                        }
                        if Rect::new(85.0, 365.0, 130.0, 36.0).contains(pointer) {
                            self.name.clear();
                            self.form = Some(ProfileForm::Create);
                        } else if Rect::new(225.0, 365.0, 130.0, 36.0).contains(pointer) {
                            if let Some(entry) = profiles.current() {
                                self.name = entry.name.clone();
                                self.form = Some(ProfileForm::Rename(entry.id));
                            }
                        } else if Rect::new(365.0, 365.0, 130.0, 36.0).contains(pointer) {
                            if let Some(entry) = profiles.current() {
                                self.form = Some(ProfileForm::Delete(entry.id));
                            }
                        } else if Rect::new(85.0, 327.0, 85.0, 30.0).contains(pointer) {
                            self.page = self.page.saturating_sub(1);
                        } else if Rect::new(455.0, 327.0, 85.0, 30.0).contains(pointer)
                            && (self.page + 1) * 6 < profiles.entries.len()
                        {
                            self.page += 1;
                        }
                    }
                    SelectorPanel::Options => {
                        for (index, y) in [145.0, 205.0].into_iter().enumerate() {
                            if Rect::new(230.0, y, 270.0, 28.0).contains(pointer) {
                                let value = ((pointer.x - 230.0) / 270.0).clamp(0.0, 1.0);
                                return (
                                    true,
                                    Some(UiCommand::Setting(if index == 0 {
                                        SettingChange::MusicVolume(value)
                                    } else {
                                        SettingChange::EffectsVolume(value)
                                    })),
                                );
                            }
                        }
                        if Rect::new(230.0, 260.0, 270.0, 32.0).contains(pointer) {
                            return (
                                true,
                                Some(UiCommand::Setting(SettingChange::Fullscreen(
                                    !settings.fullscreen,
                                ))),
                            );
                        }
                        if Rect::new(230.0, 305.0, 270.0, 32.0).contains(pointer) {
                            return (
                                true,
                                Some(UiCommand::Setting(SettingChange::CustomCursor(
                                    !settings.custom_cursor,
                                ))),
                            );
                        }
                    }
                    SelectorPanel::Scores => {
                        if self.score_focus.is_some() {
                            return (true, None);
                        }
                        for (index, tab) in [
                            ScoreTab::Adventure,
                            ScoreTab::TimeTrial,
                            ScoreTab::Challenge,
                            ScoreTab::Personal,
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            if Rect::new(76.0 + index as f32 * 122.0, 110.0, 118.0, 30.0)
                                .contains(pointer)
                            {
                                self.score_tab = tab;
                                self.page = 0;
                                self.score_focus = None;
                            }
                        }
                        for index in 0..8 {
                            if Rect::new(80.0, 145.0 + index as f32 * 25.0, 480.0, 23.0)
                                .contains(pointer)
                                && self.page * 8 + index < profiles.score_rows(self.score_tab).len()
                            {
                                self.score_focus = Some(self.page * 8 + index);
                                return (true, None);
                            }
                        }
                        if Rect::new(85.0, 365.0, 85.0, 30.0).contains(pointer) {
                            self.page = self.page.saturating_sub(1);
                        } else if Rect::new(455.0, 365.0, 85.0, 30.0).contains(pointer)
                            && (self.page + 1) * 8 < profiles.score_rows(self.score_tab).len()
                        {
                            self.page += 1;
                        }
                    }
                    SelectorPanel::Help => {}
                }
            }
            return (true, None);
        }
        if click {
            for (index, panel) in [
                SelectorPanel::Profiles,
                SelectorPanel::Options,
                SelectorPanel::Scores,
                SelectorPanel::Help,
            ]
            .into_iter()
            .enumerate()
            {
                if Rect::new(8.0 + index as f32 * 124.0, 8.0, 118.0, 32.0).contains(pointer) {
                    self.panel = Some(panel);
                    self.page = 0;
                    self.score_focus = None;
                    self.error.clear();
                    return (true, None);
                }
            }
            if Rect::new(504.0, 8.0, 118.0, 32.0).contains(pointer) {
                return (true, Some(UiCommand::Quit));
            }
        }
        (false, None)
    }

    fn draw(&self, profiles: &Profiles, settings: GameSettings, caps: SettingsCapabilities) {
        if self.panel.is_none() {
            for (index, title) in ["Profiles", "Options", "Scores", "Help", "Quit"]
                .into_iter()
                .enumerate()
            {
                ui_button(
                    Rect::new(8.0 + index as f32 * 124.0, 8.0, 118.0, 32.0),
                    title,
                );
            }
            if let Some(entry) = profiles.current() {
                draw_text(format!("Player: {}", entry.name), 20.0, 74.0, 20.0, WHITE);
            }
            return;
        }
        draw_rectangle(60.0, 63.0, 520.0, 357.0, Color::new(0.01, 0.08, 0.13, 0.97));
        draw_rectangle_lines(60.0, 63.0, 520.0, 357.0, 2.0, YELLOW);
        ui_button(Rect::new(510.0, 73.0, 65.0, 30.0), "Back");
        match self.panel.expect("panel checked above") {
            SelectorPanel::Profiles => {
                draw_text("PROFILES", 85.0, 100.0, 29.0, YELLOW);
                for (row, entry) in profiles
                    .entries
                    .iter()
                    .skip(self.page * 6)
                    .take(6)
                    .enumerate()
                {
                    ui_button(
                        Rect::new(95.0, 132.0 + row as f32 * 36.0, 440.0, 32.0),
                        &format!(
                            "{}{}",
                            if profiles.current_id == Some(entry.id) {
                                "> "
                            } else {
                                ""
                            },
                            entry.name
                        ),
                    );
                }
                ui_button(Rect::new(85.0, 327.0, 85.0, 30.0), "Prev");
                ui_button(Rect::new(455.0, 327.0, 85.0, 30.0), "Next");
                ui_button(Rect::new(85.0, 365.0, 130.0, 36.0), "Create");
                ui_button(Rect::new(225.0, 365.0, 130.0, 36.0), "Rename");
                ui_button(Rect::new(365.0, 365.0, 130.0, 36.0), "Delete");
            }
            SelectorPanel::Options => {
                draw_text("OPTIONS", 85.0, 100.0, 29.0, YELLOW);
                for (label, value, enabled, y) in [
                    ("Music", settings.music_volume, caps.music_volume, 145.0),
                    (
                        "Effects",
                        settings.effects_volume,
                        caps.effects_volume,
                        205.0,
                    ),
                ] {
                    draw_text(label, 95.0, y + 20.0, 22.0, WHITE);
                    draw_rectangle(230.0, y, 270.0, 28.0, DARKGRAY);
                    draw_rectangle(
                        230.0,
                        y,
                        270.0 * value,
                        28.0,
                        if enabled { YELLOW } else { GRAY },
                    );
                }
                ui_button(
                    Rect::new(230.0, 260.0, 270.0, 32.0),
                    &format!("Fullscreen: {}", settings.fullscreen),
                );
                ui_button(
                    Rect::new(230.0, 305.0, 270.0, 32.0),
                    &format!("Custom cursor: {}", settings.custom_cursor),
                );
                draw_text(
                    "Hardware acceleration: unavailable",
                    95.0,
                    386.0,
                    18.0,
                    GRAY,
                );
            }
            SelectorPanel::Scores => {
                draw_text("HIGH SCORES", 85.0, 100.0, 29.0, YELLOW);
                if let Some(index) = self.score_focus {
                    if let Some(row) = profiles.score_rows(self.score_tab).get(index) {
                        draw_text(
                            format!(
                                "{:?} tank {}{}",
                                row.kind,
                                row.tank,
                                row.level.map_or(String::new(), |n| format!("-{n}"))
                            ),
                            85.0,
                            145.0,
                            22.0,
                            YELLOW,
                        );
                        draw_text(
                            format!(
                                "Personal: {}",
                                row.personal.map_or("--".into(), |score| score.to_string())
                            ),
                            85.0,
                            180.0,
                            19.0,
                            WHITE,
                        );
                        for (index, entry) in row.global.iter().enumerate() {
                            draw_text(
                                format!("{}. {}   {}", index + 1, entry.name, entry.score),
                                100.0,
                                220.0 + index as f32 * 31.0,
                                20.0,
                                WHITE,
                            );
                        }
                    }
                    return;
                }
                for (index, label) in ["Adventure", "Time Trial", "Challenge", "Personal"]
                    .into_iter()
                    .enumerate()
                {
                    ui_button(
                        Rect::new(76.0 + index as f32 * 122.0, 110.0, 118.0, 30.0),
                        label,
                    );
                }
                for (index, row) in profiles
                    .score_rows(self.score_tab)
                    .iter()
                    .skip(self.page * 8)
                    .take(8)
                    .enumerate()
                {
                    draw_text(
                        format!(
                            "{:?} {}{}   You: {}   Top: {}",
                            row.kind,
                            row.tank,
                            row.level.map_or(String::new(), |n| format!("-{n}")),
                            row.personal.map_or("--".into(), |n| n.to_string()),
                            row.global.first().map_or("--".into(), |entry| format!(
                                "{} {}",
                                entry.name, entry.score
                            ))
                        ),
                        83.0,
                        163.0 + index as f32 * 24.0,
                        17.0,
                        WHITE,
                    );
                }
                ui_button(Rect::new(85.0, 365.0, 85.0, 30.0), "Prev");
                ui_button(Rect::new(455.0, 365.0, 85.0, 30.0), "Next");
            }
            SelectorPanel::Help => {
                draw_text("HELP", 85.0, 100.0, 29.0, YELLOW);
                for (index, line) in [
                    "Adventure: feed fish, collect coins, buy eggs.",
                    "Time Trial: score before the tank timer expires.",
                    "Challenge: finish each tank against rising prices.",
                    "Virtual Tank: keep fish, visit store, collect shells.",
                    "Sandbox: enter selector code after seventh pet slot.",
                    "Esc pauses tanks; S saves; Q exits while paused.",
                ]
                .into_iter()
                .enumerate()
                {
                    draw_text(line, 90.0, 150.0 + index as f32 * 38.0, 20.0, WHITE);
                }
            }
        }
        if !self.error.is_empty() {
            draw_text(&self.error, 85.0, 410.0, 18.0, ORANGE);
        }
        if let Some(form) = self.form {
            draw_rectangle(
                115.0,
                160.0,
                410.0,
                195.0,
                Color::new(0.02, 0.08, 0.12, 0.99),
            );
            draw_rectangle_lines(115.0, 160.0, 410.0, 195.0, 2.0, YELLOW);
            draw_text(
                match form {
                    ProfileForm::Create => "Create profile",
                    ProfileForm::Rename(_) => "Rename profile",
                    ProfileForm::Delete(_) => "Delete current profile?",
                },
                140.0,
                200.0,
                25.0,
                WHITE,
            );
            if !matches!(form, ProfileForm::Delete(_)) {
                draw_text(&self.name, 145.0, 265.0, 24.0, YELLOW);
            }
            ui_button(Rect::new(145.0, 310.0, 145.0, 38.0), "Confirm");
            ui_button(Rect::new(350.0, 310.0, 145.0, 38.0), "Cancel");
        }
    }
}

pub async fn run(
    options: Options,
    game_root: PathBuf,
    identity: InstallIdentity,
    assets: GameAssets,
    mut profiles: Profiles,
    mut settings: SettingsSave,
) -> Result<(), Box<dyn Error>> {
    let opened_at = wall_seconds();
    if let Some(current) = profiles.current_mut() {
        current
            .session
            .resume_virtual_from_save(opened_at, opened_at / 86_400 * 86_400);
    }
    let presentation = Presentation::load(&game_root, &assets, options.muted).await?;
    let (mut music, mut music_unavailable) = if options.muted {
        (None, None)
    } else {
        match MusicOwner::start(game_root.clone()) {
            Ok(owner) => (Some(owner), None),
            Err(error) => (None, Some(format!("Music thread unavailable: {error}"))),
        }
    };
    {
        let mut backend = RuntimeSettingsBackend {
            audio: music.as_ref(),
        };
        if let Some(owner) = music.as_ref() {
            owner.set_music_volume(settings.settings.music_volume)?;
            owner.set_effects_volume(settings.settings.effects_volume)?;
        }
        backend.set_fullscreen(settings.settings.fullscreen)?;
        backend.set_custom_cursor(settings.settings.custom_cursor)?;
    }
    let mut evidence = options
        .evidence_dir
        .as_ref()
        .zip(profiles.current())
        .map(|(root, entry)| {
            Evidence::open(
                root,
                &identity,
                &entry.session,
                options.seed,
                options.test_speed,
            )
        })
        .transpose()?;
    let started = get_time();
    let mut start_session_tick = profiles.current().map_or(0, |entry| entry.session.ticks);
    let mut clock = StepClock::new(options.test_speed);
    let mut pending_actions = Vec::new();
    let mut paused = false;
    let mut hatch_pointer_owned = false;
    let mut hatch_background_down = false;
    let mut feed_press_at = None::<(InputPress, Option<(i32, i32)>)>;
    let mut held_feed_at = None::<InputPress>;
    let mut held_fire_at = None::<InputPress>;
    let mut healing_warning_until_tick = None::<u64>;
    let mut selector_ui = SelectorUi::default();
    let mut entry_code = EntryCode::default();
    let song_catalog = match SongCatalog::load_from_install(&game_root) {
        Ok(catalog) => Some(catalog),
        Err(error) => {
            eprintln!("Fish songs unavailable: {error}");
            None
        }
    };
    let mut song_player = FishSongPlayer::default();
    let mut singing_feeds = HashMap::<u16, u8>::new();
    let mut pending_song_ticks = None::<u8>;
    let mut pending_song_kind = None::<NamedFish>;
    let mut song_sequence = 0_usize;
    let mut song_id = 0_i32;
    let mut song_title = String::new();
    let mut song_title_until = 0.0_f64;
    let mut interlude = None::<InterludeController>;
    prevent_quit();
    loop {
        let elapsed = get_time() - started;
        let scale = (screen_width() / 640.0).min(screen_height() / 480.0);
        let offset = vec2(
            (screen_width() - 640.0 * scale) / 2.0,
            (screen_height() - 480.0 * scale) / 2.0,
        );
        let (mouse_x, mouse_y) = mouse_position();
        let pointer = (vec2(mouse_x, mouse_y) - offset) / scale;
        let selector_open = profiles.current().is_none()
            || profiles
                .current()
                .is_some_and(|entry| entry.session.phase == AdventurePhase::GameSelector);
        if profiles.current().is_none() {
            selector_ui.panel = Some(SelectorPanel::Profiles);
        }
        let (ui_owned_input, ui_command) =
            selector_ui.input(&profiles, settings.settings, pointer, selector_open);
        let ui_active = selector_ui.panel.is_some() || profiles.current().is_none();
        if let Some(command) = ui_command {
            let profiles_path = options.save_dir.join("profiles.json");
            let result = match command {
                UiCommand::Create(name) => {
                    if profiles.current().is_some() {
                        cli::save_profiles_checkpoint(&options, &profiles)?;
                    }
                    profiles
                        .create(
                            &profiles_path,
                            &name,
                            options.seed.wrapping_add(profiles.next_id),
                        )
                        .map(|_| true)
                }
                UiCommand::Select(id) => {
                    cli::save_profiles_checkpoint(&options, &profiles)?;
                    profiles.select(&profiles_path, id).map(|_| true)
                }
                UiCommand::Rename(id, name) => {
                    profiles.rename(&profiles_path, id, &name).map(|_| false)
                }
                UiCommand::Delete(id) => profiles.delete(&profiles_path, id).map(|_| true),
                UiCommand::Setting(change) => {
                    let previous = settings.settings;
                    let mut backend = RuntimeSettingsBackend {
                        audio: music.as_ref(),
                    };
                    match settings.settings.change(&mut backend, change) {
                        Err(error) => Err(error.into()),
                        Ok(()) => match settings.save(&options.save_dir.join("settings.json")) {
                            Err(error) => {
                                settings.settings = previous;
                                let _ = match change {
                                    SettingChange::MusicVolume(_) => {
                                        backend.set_music_volume(previous.music_volume)
                                    }
                                    SettingChange::EffectsVolume(_) => {
                                        backend.set_effects_volume(previous.effects_volume)
                                    }
                                    SettingChange::Fullscreen(_) => {
                                        backend.set_fullscreen(previous.fullscreen)
                                    }
                                    SettingChange::CustomCursor(_) => {
                                        backend.set_custom_cursor(previous.custom_cursor)
                                    }
                                };
                                Err(error)
                            }
                            Ok(()) => Ok(false),
                        },
                    }
                }
                UiCommand::Quit => {
                    cli::save_profiles_checkpoint(&options, &profiles)?;
                    break;
                }
            };
            match result {
                Ok(switched) => {
                    selector_ui.error.clear();
                    if switched {
                        if let Some(owner) = music.as_ref() {
                            let _ = owner.stop_song(song_id);
                        }
                        song_player.clear();
                        singing_feeds.clear();
                        pending_song_ticks = None;
                        pending_song_kind = None;
                        song_title.clear();
                        interlude = None;
                        paused = false;
                        pending_actions.clear();
                        clock.clear_backlog();
                        feed_press_at = None;
                        held_feed_at = None;
                        held_fire_at = None;
                        entry_code = EntryCode::default();
                        if let Some(entry) = profiles.current() {
                            start_session_tick = entry.session.ticks;
                        }
                    }
                }
                Err(error) => selector_ui.error = error.to_string(),
            }
        }
        if profiles.current().is_none() {
            if is_quit_requested() {
                break;
            }
            clear_background(BLACK);
            let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, 640.0, 480.0));
            camera.zoom.y = 2.0 / 480.0;
            camera.viewport = Some((
                offset.x as i32,
                offset.y as i32,
                (640.0 * scale) as i32,
                (480.0 * scale) as i32,
            ));
            set_camera(&camera);
            selector_ui.draw(
                &profiles,
                settings.settings,
                RuntimeSettingsBackend {
                    audio: music.as_ref(),
                }
                .capabilities(),
            );
            if settings.settings.custom_cursor {
                draw_circle(pointer.x, pointer.y, 5.0, YELLOW);
            }
            set_default_camera();
            next_frame().await;
            continue;
        }
        let session = &mut profiles.current_mut().ok_or("No current profile")?.session;
        if session.phase == AdventurePhase::AdventureFinaleInterlude && interlude.is_none() {
            interlude = InterludeController::new(&session.finale_fallen_pets).ok();
        } else if session.phase != AdventurePhase::AdventureFinaleInterlude {
            interlude = None;
        }
        if session.phase == AdventurePhase::GameSelector && !ui_active {
            for (pressed, key) in [
                (is_key_pressed(KeyCode::Up), SelectorKey::Up),
                (is_key_pressed(KeyCode::Down), SelectorKey::Down),
                (is_key_pressed(KeyCode::Left), SelectorKey::Left),
                (is_key_pressed(KeyCode::Right), SelectorKey::Right),
                (is_key_pressed(KeyCode::B), SelectorKey::B),
                (is_key_pressed(KeyCode::A), SelectorKey::A),
            ] {
                if pressed && entry_code.push(key) {
                    pending_actions.push(Action::OpenSandbox);
                }
            }
        } else {
            entry_code.reset();
        }
        if session.phase == AdventurePhase::SandboxPlaying && !paused {
            while let Some(character) = get_char_pressed() {
                pending_actions.push(Action::SandboxKeyChar { character });
            }
        }
        if !ui_owned_input && !ui_active && is_key_pressed(KeyCode::Escape) {
            if matches!(session.phase, AdventurePhase::VirtualPurchaseConfirm { .. }) {
                let now_seconds = wall_seconds();
                pending_actions.push(Action::VirtualConfirmPurchase {
                    accept: false,
                    now_seconds,
                    day_start_seconds: now_seconds / 86_400 * 86_400,
                });
            } else if let AdventurePhase::VirtualFishSaleConfirm { id } = session.phase {
                pending_actions.push(Action::VirtualSellFish {
                    id,
                    accept: false,
                    now_seconds: wall_seconds(),
                });
            } else if matches!(session.phase, AdventurePhase::VirtualBackdropSaleConfirm) {
                pending_actions.push(Action::VirtualSellBackdrop { accept: false });
            } else if matches!(session.phase, AdventurePhase::VirtualRename { .. }) {
                pending_actions.push(Action::OpenMenu);
            } else if matches!(
                session.phase,
                AdventurePhase::VirtualStore | AdventurePhase::VirtualSetup
            ) {
                pending_actions.push(Action::Continue);
            } else if matches!(
                session.phase,
                AdventurePhase::TimeTrialPrestoDialog { .. }
                    | AdventurePhase::AdventurePrestoDialog { .. }
                    | AdventurePhase::ChallengePrestoDialog { .. }
                    | AdventurePhase::SandboxPrestoDialog { .. }
                    | AdventurePhase::VirtualPrestoDialog { .. }
            ) && !paused
            {
                pending_actions.push(Action::PrestoCancel);
            } else if matches!(
                session.phase,
                AdventurePhase::BonusResults { ref result } if result.purchase.confirming
            ) || (session.phase == AdventurePhase::TimeTrialResults
                && session
                    .time_trial
                    .as_ref()
                    .and_then(|run| run.result.as_ref())
                    .is_some_and(|result| result.purchase.confirming))
                || (session.phase == AdventurePhase::ChallengeResults
                    && session
                        .challenge
                        .as_ref()
                        .and_then(|run| run.result.as_ref())
                        .is_some_and(|result| result.purchase.confirming))
            {
                pending_actions.push(Action::ConfirmBonusPurchase { accept: false });
            } else {
                paused = !paused;
            }
            clock.clear_backlog();
            feed_press_at = None;
            held_feed_at = None;
            held_fire_at = None;
        }
        let button_height = presentation.images["IMAGE_MAINBUTTON"].height();
        let menu_rect = if matches!(
            session.phase,
            AdventurePhase::Hatch { .. }
                | AdventurePhase::TankFourFinaleHatch { .. }
                | AdventurePhase::PetSelection { .. }
                | AdventurePhase::TimeTrialPetSelection { .. }
                | AdventurePhase::TimeTrialTankSelection
                | AdventurePhase::ChallengePetSelection { .. }
                | AdventurePhase::ChallengePetSelectionConfirmation { .. }
                | AdventurePhase::ChallengeTankSelection
        ) {
            Rect::new(525.0, 4.0, 80.0, button_height)
        } else {
            Rect::new(525.0, 3.0, 101.0, 29.0)
        };
        if !paused && !ui_active {
            let current_name = match &session.phase {
                AdventurePhase::VirtualPurchaseConfirm { name, .. }
                | AdventurePhase::VirtualRename { name, .. } => Some(name.clone()),
                _ => None,
            };
            if let Some(mut name) = current_name {
                let original = name.clone();
                if is_key_pressed(KeyCode::Backspace) {
                    name.pop();
                }
                while let Some(character) = get_char_pressed() {
                    if (character.is_ascii_graphic() || character == ' ')
                        && name.len() + character.len_utf8() <= 64
                    {
                        name.push(character);
                    }
                }
                if name != original {
                    pending_actions.push(match session.phase {
                        AdventurePhase::VirtualRename { .. } => Action::VirtualSetRename { name },
                        _ => Action::VirtualSetPurchaseName { name },
                    });
                }
            }
        }
        if !ui_owned_input && !ui_active && is_mouse_button_pressed(MouseButton::Left) {
            if matches!(
                session.phase,
                AdventurePhase::TimeTrialPrestoDialog { .. }
                    | AdventurePhase::AdventurePrestoDialog { .. }
                    | AdventurePhase::ChallengePrestoDialog { .. }
                    | AdventurePhase::SandboxPrestoDialog { .. }
                    | AdventurePhase::VirtualPrestoDialog { .. }
            ) && !paused
            {
                if let Some(pet) =
                    presto_choice_at_pointer(&session.progress.unlocked_pets, pointer)
                {
                    pending_actions.push(Action::PrestoPress { pet });
                }
            } else if menu_rect.contains(pointer)
                && matches!(
                    session.phase,
                    AdventurePhase::Playing
                        | AdventurePhase::TimeTrialPlaying
                        | AdventurePhase::ChallengePlaying
                        | AdventurePhase::SandboxPlaying
                        | AdventurePhase::TimeTrialTankSelection
                        | AdventurePhase::TimeTrialPetSelection { .. }
                        | AdventurePhase::ChallengeTankSelection
                        | AdventurePhase::ChallengePetSelection { .. }
                        | AdventurePhase::ChallengePetSelectionConfirmation { .. }
                        | AdventurePhase::Hatch { .. }
                        | AdventurePhase::TankFourFinaleHatch { .. }
                        | AdventurePhase::PetSelection { .. }
                )
            {
                if matches!(
                    session.phase,
                    AdventurePhase::Hatch { .. }
                        | AdventurePhase::TankFourFinaleHatch { .. }
                        | AdventurePhase::PetSelection { .. }
                        | AdventurePhase::TimeTrialTankSelection
                        | AdventurePhase::TimeTrialPetSelection { .. }
                        | AdventurePhase::ChallengeTankSelection
                        | AdventurePhase::ChallengePetSelection { .. }
                        | AdventurePhase::ChallengePetSelectionConfirmation { .. }
                ) {
                    pending_actions.push(Action::OpenMenu);
                } else {
                    paused = !paused;
                    clock.clear_backlog();
                }
                feed_press_at = None;
                held_feed_at = None;
                held_fire_at = None;
            } else if !paused {
                let action = match session.phase {
                    AdventurePhase::VirtualTank => {
                        let now_seconds = wall_seconds();
                        if Rect::new(16.0, 4.0, 110.0, 32.0).contains(pointer) {
                            Action::OpenVirtualStore {
                                now_seconds,
                                day_start_seconds: now_seconds / 86_400 * 86_400,
                            }
                        } else if Rect::new(140.0, 4.0, 110.0, 32.0).contains(pointer) {
                            Action::OpenVirtualSetup
                        } else if Rect::new(514.0, 4.0, 110.0, 32.0).contains(pointer) {
                            Action::OpenMenu
                        } else {
                            Action::VirtualTankClick {
                                x: pointer.x,
                                y: pointer.y,
                                now_seconds,
                                day_start_seconds: now_seconds / 86_400 * 86_400,
                            }
                        }
                    }
                    AdventurePhase::VirtualStore => {
                        if Rect::new(500.0, 445.0, 110.0, 32.0).contains(pointer)
                            || Rect::new(514.0, 4.0, 110.0, 32.0).contains(pointer)
                        {
                            Action::Continue
                        } else if let Some(slot) =
                            (0..8).find(|slot| virtual_offer_rect(*slot).contains(pointer))
                        {
                            Action::VirtualStoreOffer { slot }
                        } else {
                            Action::Click {
                                x: pointer.x,
                                y: pointer.y,
                            }
                        }
                    }
                    AdventurePhase::VirtualPurchaseConfirm { .. } => {
                        let now_seconds = wall_seconds();
                        if Rect::new(140.0, 305.0, 150.0, 38.0).contains(pointer) {
                            Action::VirtualConfirmPurchase {
                                accept: true,
                                now_seconds,
                                day_start_seconds: now_seconds / 86_400 * 86_400,
                            }
                        } else if Rect::new(350.0, 305.0, 150.0, 38.0).contains(pointer) {
                            Action::VirtualConfirmPurchase {
                                accept: false,
                                now_seconds,
                                day_start_seconds: now_seconds / 86_400 * 86_400,
                            }
                        } else {
                            Action::Click {
                                x: pointer.x,
                                y: pointer.y,
                            }
                        }
                    }
                    AdventurePhase::VirtualSetup => {
                        let selected_id = session.virtual_tank.selected_fish;
                        if Rect::new(515.0, 445.0, 100.0, 30.0).contains(pointer)
                            || Rect::new(514.0, 4.0, 110.0, 32.0).contains(pointer)
                        {
                            Action::Continue
                        } else if let Some(id) = session
                            .virtual_tank
                            .active_fish()
                            .enumerate()
                            .find(|(index, _)| {
                                Rect::new(
                                    20.0 + (*index / 10) as f32 * 145.0,
                                    99.0 + (*index % 10) as f32 * 27.0,
                                    140.0,
                                    27.0,
                                )
                                .contains(pointer)
                            })
                            .map(|(_, fish)| fish.id)
                        {
                            Action::VirtualSelectFish { id }
                        } else if let Some(id) = selected_id
                            .filter(|_| Rect::new(20.0, 417.0, 94.0, 30.0).contains(pointer))
                        {
                            let shown = session
                                .virtual_tank
                                .active_fish()
                                .find(|fish| fish.id == id)
                                .is_some_and(|fish| !fish.shown);
                            Action::VirtualShowFish { id, shown }
                        } else if let Some(id) = selected_id
                            .filter(|_| Rect::new(120.0, 417.0, 82.0, 30.0).contains(pointer))
                        {
                            Action::VirtualBeginRename { id }
                        } else if let Some(id) = selected_id
                            .filter(|_| Rect::new(208.0, 417.0, 76.0, 30.0).contains(pointer))
                        {
                            Action::VirtualOfferFishSale { id }
                        } else if Rect::new(330.0, 416.0, 50.0, 30.0).contains(pointer) {
                            Action::VirtualSwitchBackdrop { direction: -1 }
                        } else if Rect::new(385.0, 416.0, 50.0, 30.0).contains(pointer) {
                            Action::VirtualSwitchBackdrop { direction: 1 }
                        } else if Rect::new(440.0, 416.0, 65.0, 30.0).contains(pointer) {
                            Action::VirtualOfferBackdropSale
                        } else if Rect::new(510.0, 416.0, 90.0, 30.0).contains(pointer) {
                            let mut settings = session.virtual_tank.settings.clone();
                            settings.drop_shells = !settings.drop_shells;
                            Action::VirtualSetSettings { settings }
                        } else if let Some(pet) = session
                            .progress
                            .unlocked_pets
                            .iter()
                            .enumerate()
                            .find(|(index, _)| {
                                Rect::new(
                                    330.0 + (*index % 3) as f32 * 95.0,
                                    102.0 + (*index / 3) as f32 * 30.0,
                                    93.0,
                                    29.0,
                                )
                                .contains(pointer)
                            })
                            .map(|(_, pet)| *pet)
                        {
                            Action::VirtualTogglePet { pet }
                        } else {
                            Action::Click {
                                x: pointer.x,
                                y: pointer.y,
                            }
                        }
                    }
                    AdventurePhase::VirtualFishSaleConfirm { id } => {
                        let now_seconds = wall_seconds();
                        if Rect::new(140.0, 305.0, 150.0, 38.0).contains(pointer) {
                            Action::VirtualSellFish {
                                id,
                                accept: true,
                                now_seconds,
                            }
                        } else if Rect::new(350.0, 305.0, 150.0, 38.0).contains(pointer) {
                            Action::VirtualSellFish {
                                id,
                                accept: false,
                                now_seconds,
                            }
                        } else {
                            Action::Click {
                                x: pointer.x,
                                y: pointer.y,
                            }
                        }
                    }
                    AdventurePhase::VirtualBackdropSaleConfirm => {
                        if Rect::new(140.0, 305.0, 150.0, 38.0).contains(pointer) {
                            Action::VirtualSellBackdrop { accept: true }
                        } else if Rect::new(350.0, 305.0, 150.0, 38.0).contains(pointer) {
                            Action::VirtualSellBackdrop { accept: false }
                        } else {
                            Action::Click {
                                x: pointer.x,
                                y: pointer.y,
                            }
                        }
                    }
                    AdventurePhase::VirtualRename { id, ref name } => {
                        if Rect::new(140.0, 305.0, 150.0, 38.0).contains(pointer) {
                            Action::VirtualRenameFish {
                                id,
                                name: name.clone(),
                            }
                        } else if Rect::new(350.0, 305.0, 150.0, 38.0).contains(pointer) {
                            Action::OpenMenu
                        } else {
                            Action::Click {
                                x: pointer.x,
                                y: pointer.y,
                            }
                        }
                    }
                    AdventurePhase::ChallengeResults
                        if session
                            .challenge
                            .as_ref()
                            .and_then(|run| run.result.as_ref())
                            .is_some_and(|result| result.purchase.confirming)
                            && results_confirm_rect(true).contains(pointer) =>
                    {
                        Action::ConfirmBonusPurchase { accept: true }
                    }
                    AdventurePhase::ChallengeResults
                        if session
                            .challenge
                            .as_ref()
                            .and_then(|run| run.result.as_ref())
                            .is_some_and(|result| result.purchase.confirming)
                            && results_confirm_rect(false).contains(pointer) =>
                    {
                        Action::ConfirmBonusPurchase { accept: false }
                    }
                    AdventurePhase::ChallengeResults
                        if session
                            .challenge
                            .as_ref()
                            .and_then(|run| run.result.as_ref())
                            .is_some_and(|result| {
                                !result.purchase.confirming
                                    && results_offer(result.purchase.offered_cursor).is_some()
                                    && result.updates >= 30
                            })
                            && results_offer_rect().contains(pointer) =>
                    {
                        Action::OfferBonusPurchase
                    }
                    AdventurePhase::ChallengeResults
                        if results_continue_rect(button_height).contains(pointer) =>
                    {
                        Action::Continue
                    }
                    AdventurePhase::ChallengeResults
                        if Rect::new(186.0, 365.0, 264.0, button_height).contains(pointer) =>
                    {
                        Action::ChallengeStory
                    }
                    AdventurePhase::ChallengeLost | AdventurePhase::ChallengeInterlude { .. }
                        if Rect::new(186.0, 310.0, 264.0, button_height).contains(pointer) =>
                    {
                        Action::Continue
                    }
                    AdventurePhase::BonusResults { ref result }
                        if result.purchase.confirming
                            && results_confirm_rect(true).contains(pointer) =>
                    {
                        Action::ConfirmBonusPurchase { accept: true }
                    }
                    AdventurePhase::BonusResults { ref result }
                        if result.purchase.confirming
                            && results_confirm_rect(false).contains(pointer) =>
                    {
                        Action::ConfirmBonusPurchase { accept: false }
                    }
                    AdventurePhase::BonusResults { ref result }
                        if !result.purchase.confirming
                            && results_offer(result.purchase.offered_cursor).is_some()
                            && result.updates >= 30
                            && results_offer_rect().contains(pointer) =>
                    {
                        Action::OfferBonusPurchase
                    }
                    AdventurePhase::BonusResults { ref result }
                        if result.updates >= 30
                            && results_continue_rect(button_height).contains(pointer) =>
                    {
                        Action::Continue
                    }
                    AdventurePhase::TimeTrialResults
                        if session
                            .time_trial
                            .as_ref()
                            .and_then(|run| run.result.as_ref())
                            .is_some_and(|result| result.purchase.confirming)
                            && results_confirm_rect(true).contains(pointer) =>
                    {
                        Action::ConfirmBonusPurchase { accept: true }
                    }
                    AdventurePhase::TimeTrialResults
                        if session
                            .time_trial
                            .as_ref()
                            .and_then(|run| run.result.as_ref())
                            .is_some_and(|result| result.purchase.confirming)
                            && results_confirm_rect(false).contains(pointer) =>
                    {
                        Action::ConfirmBonusPurchase { accept: false }
                    }
                    AdventurePhase::TimeTrialResults
                        if session
                            .time_trial
                            .as_ref()
                            .and_then(|run| run.result.as_ref())
                            .is_some_and(|result| {
                                !result.purchase.confirming
                                    && results_offer(result.purchase.offered_cursor).is_some()
                                    && result.updates >= 30
                            })
                            && results_offer_rect().contains(pointer) =>
                    {
                        Action::OfferBonusPurchase
                    }
                    AdventurePhase::AdventureFinaleInterlude
                        if Rect::new(186.0, 445.0, 264.0, button_height).contains(pointer) =>
                    {
                        if let Some(controller) = interlude.as_mut() {
                            let _ = controller.back_to_menu();
                        }
                        Action::OpenMenu
                    }
                    AdventurePhase::AdventureFinaleInterlude
                        if interlude
                            .as_ref()
                            .is_some_and(InterludeController::can_skip_by_click) =>
                    {
                        if let Some(controller) = interlude.as_mut() {
                            let _ = controller.click_after_scroll();
                        }
                        Action::OpenMenu
                    }
                    AdventurePhase::PetSelection { .. } => {
                        if let Some(pet) = pet_at_pointer(&session.progress.unlocked_pets, pointer)
                        {
                            Action::TogglePet { pet }
                        } else if Rect::new(225.0, 250.0, 186.0, button_height).contains(pointer) {
                            Action::Continue
                        } else {
                            Action::Click {
                                x: pointer.x,
                                y: pointer.y,
                            }
                        }
                    }
                    AdventurePhase::TimeTrialPetSelection { .. }
                    | AdventurePhase::ChallengePetSelection { .. } => {
                        if let Some(pet) = session
                            .progress
                            .unlocked_pets
                            .iter()
                            .copied()
                            .enumerate()
                            .find(|(index, _)| pet_card_rect(*index).contains(pointer))
                            .map(|(_, pet)| pet)
                        {
                            Action::TogglePet { pet }
                        } else if Rect::new(225.0, 250.0, 186.0, button_height).contains(pointer) {
                            Action::Continue
                        } else {
                            Action::Click {
                                x: pointer.x,
                                y: pointer.y,
                            }
                        }
                    }
                    AdventurePhase::TimeTrialTankSelection
                    | AdventurePhase::ChallengeTankSelection => {
                        let chosen = (1..=4).find(|tank| {
                            Rect::new(
                                186.0,
                                150.0 + (*tank - 1) as f32 * 66.0,
                                264.0,
                                button_height,
                            )
                            .contains(pointer)
                        });
                        chosen.map_or(
                            Action::Click {
                                x: pointer.x,
                                y: pointer.y,
                            },
                            |tank| {
                                if session.phase == AdventurePhase::ChallengeTankSelection {
                                    Action::SelectChallengeTank { tank }
                                } else {
                                    Action::SelectTimeTrialTank { tank }
                                }
                            },
                        )
                    }
                    AdventurePhase::TimeTrialTimesUp
                        if Rect::new(186.0, 310.0, 264.0, button_height).contains(pointer) =>
                    {
                        Action::Continue
                    }
                    AdventurePhase::TimeTrialResults
                        if results_continue_rect(button_height).contains(pointer) =>
                    {
                        Action::Continue
                    }
                    AdventurePhase::TimeTrialGameOver { updates }
                        if updates > 30
                            && Rect::new(186.0, 310.0, 264.0, button_height).contains(pointer) =>
                    {
                        Action::Continue
                    }
                    AdventurePhase::TimeTrialInvasionTutorial { .. }
                    | AdventurePhase::ChallengeInvasionTutorial { .. }
                        if Rect::new(186.0, 310.0, 264.0, button_height).contains(pointer) =>
                    {
                        Action::Continue
                    }
                    AdventurePhase::PetSelectionConfirmation { .. }
                    | AdventurePhase::ChallengePetSelectionConfirmation { .. }
                        if Rect::new(155.0, 300.0, 145.0, button_height).contains(pointer) =>
                    {
                        Action::ConfirmPetSelection { accept: true }
                    }
                    AdventurePhase::PetSelectionConfirmation { .. }
                    | AdventurePhase::ChallengePetSelectionConfirmation { .. }
                        if Rect::new(340.0, 300.0, 145.0, button_height).contains(pointer) =>
                    {
                        Action::ConfirmPetSelection { accept: false }
                    }
                    AdventurePhase::Hatch { updates, .. }
                    | AdventurePhase::TankFourFinaleHatch { updates }
                        if updates > 170
                            && Rect::new(186.0, 445.0, 264.0, button_height).contains(pointer) =>
                    {
                        Action::Continue
                    }
                    AdventurePhase::Hatch { .. } | AdventurePhase::TankFourFinaleHatch { .. } => {
                        hatch_pointer_owned = true;
                        hatch_background_down = is_mouse_button_down(MouseButton::Left);
                        Action::HatchHold {
                            down: hatch_background_down,
                        }
                    }
                    AdventurePhase::FirstTankRescue
                        if Rect::new(186.0, 295.0, 264.0, button_height).contains(pointer) =>
                    {
                        Action::Continue
                    }
                    AdventurePhase::InvasionTutorial { .. }
                        if Rect::new(186.0, 310.0, 264.0, button_height).contains(pointer) =>
                    {
                        Action::Continue
                    }
                    AdventurePhase::GameOver { updates }
                        if updates > 30
                            && Rect::new(186.0, 310.0, 264.0, button_height).contains(pointer) =>
                    {
                        Action::Continue
                    }
                    AdventurePhase::GameSelector
                        if Rect::new(186.0, 250.0, 264.0, button_height).contains(pointer) =>
                    {
                        Action::PlayAdventure
                    }
                    AdventurePhase::GameSelector
                        if Rect::new(186.0, 310.0, 264.0, button_height).contains(pointer) =>
                    {
                        Action::PlayTimeTrial
                    }
                    AdventurePhase::GameSelector
                        if session.progress.adventure_completed
                            && Rect::new(186.0, 370.0, 264.0, button_height).contains(pointer) =>
                    {
                        Action::PlayChallenge
                    }
                    AdventurePhase::GameSelector
                        if Rect::new(186.0, 430.0, 264.0, button_height).contains(pointer) =>
                    {
                        let now_seconds = wall_seconds();
                        Action::PlayVirtualTank {
                            now_seconds,
                            day_start_seconds: now_seconds / 86_400 * 86_400,
                        }
                    }
                    AdventurePhase::HelpScreen
                        if Rect::new(186.0, 310.0, 264.0, button_height).contains(pointer) =>
                    {
                        Action::Continue
                    }
                    AdventurePhase::SandboxPlaying
                        if Rect::new(436.0, 3.0, 58.0, 60.0).contains(pointer) =>
                    {
                        Action::BuyEgg
                    }
                    AdventurePhase::Playing
                    | AdventurePhase::TimeTrialPlaying
                    | AdventurePhase::ChallengePlaying
                        if session
                            .board
                            .as_ref()
                            .is_some_and(|board| board.tank == 4 && board.breeder_unlocked)
                            && Rect::new(18.0, 3.0, 58.0, 60.0).contains(pointer) =>
                    {
                        Action::BuyBreeder
                    }
                    AdventurePhase::Playing
                    | AdventurePhase::TimeTrialPlaying
                    | AdventurePhase::ChallengePlaying
                        if session
                            .board
                            .as_ref()
                            .is_some_and(|board| board.tank != 4 && board.guppy_unlocked)
                            && Rect::new(18.0, 3.0, 58.0, 60.0).contains(pointer) =>
                    {
                        Action::BuyGuppy
                    }
                    AdventurePhase::Playing
                    | AdventurePhase::TimeTrialPlaying
                    | AdventurePhase::ChallengePlaying
                        if session
                            .board
                            .as_ref()
                            .is_some_and(|board| board.potion_unlocked)
                            && Rect::new(217.0, 3.0, 58.0, 60.0).contains(pointer) =>
                    {
                        Action::BuyPotion
                    }
                    AdventurePhase::Playing
                    | AdventurePhase::TimeTrialPlaying
                    | AdventurePhase::ChallengePlaying
                        if session
                            .board
                            .as_ref()
                            .is_some_and(|board| board.upgrades.quality_unlocked)
                            && Rect::new(87.0, 3.0, 58.0, 60.0).contains(pointer) =>
                    {
                        Action::BuyFoodQuality
                    }
                    AdventurePhase::Playing
                    | AdventurePhase::TimeTrialPlaying
                    | AdventurePhase::ChallengePlaying
                        if session
                            .board
                            .as_ref()
                            .is_some_and(|board| board.upgrades.quantity_unlocked)
                            && Rect::new(144.0, 3.0, 58.0, 60.0).contains(pointer) =>
                    {
                        Action::BuyFoodQuantity
                    }
                    AdventurePhase::Playing
                    | AdventurePhase::TimeTrialPlaying
                    | AdventurePhase::ChallengePlaying
                        if session
                            .board
                            .as_ref()
                            .is_some_and(|board| board.oscar_unlocked)
                            && Rect::new(217.0, 3.0, 58.0, 60.0).contains(pointer) =>
                    {
                        Action::BuyOscar
                    }
                    AdventurePhase::Playing
                    | AdventurePhase::TimeTrialPlaying
                    | AdventurePhase::ChallengePlaying
                        if session
                            .board
                            .as_ref()
                            .is_some_and(|board| board.grubber_unlocked)
                            && Rect::new(217.0, 3.0, 58.0, 60.0).contains(pointer) =>
                    {
                        Action::BuyGrubber
                    }
                    AdventurePhase::Playing
                    | AdventurePhase::TimeTrialPlaying
                    | AdventurePhase::ChallengePlaying
                        if session
                            .board
                            .as_ref()
                            .is_some_and(|board| board.starcatcher_unlocked)
                            && Rect::new(290.0, 3.0, 58.0, 60.0).contains(pointer) =>
                    {
                        Action::BuyStarcatcher
                    }
                    AdventurePhase::Playing
                    | AdventurePhase::TimeTrialPlaying
                    | AdventurePhase::ChallengePlaying
                        if session
                            .board
                            .as_ref()
                            .is_some_and(|board| board.gekko_unlocked)
                            && Rect::new(290.0, 3.0, 58.0, 60.0).contains(pointer) =>
                    {
                        Action::BuyGekko
                    }
                    AdventurePhase::Playing
                    | AdventurePhase::TimeTrialPlaying
                    | AdventurePhase::ChallengePlaying
                        if session
                            .board
                            .as_ref()
                            .is_some_and(|board| board.ultra_unlocked)
                            && Rect::new(290.0, 3.0, 58.0, 60.0).contains(pointer) =>
                    {
                        Action::BuyUltra
                    }
                    AdventurePhase::Playing
                    | AdventurePhase::TimeTrialPlaying
                    | AdventurePhase::ChallengePlaying
                        if session
                            .board
                            .as_ref()
                            .is_some_and(|board| board.weapon_unlocked)
                            && Rect::new(363.0, 3.0, 58.0, 60.0).contains(pointer) =>
                    {
                        Action::BuyWeapon
                    }
                    AdventurePhase::Playing
                    | AdventurePhase::TimeTrialPlaying
                    | AdventurePhase::ChallengePlaying
                        if session
                            .board
                            .as_ref()
                            .is_some_and(|board| board.egg_unlocked)
                            && Rect::new(436.0, 3.0, 58.0, 60.0).contains(pointer) =>
                    {
                        Action::BuyEgg
                    }
                    _ => Action::Click {
                        x: pointer.x,
                        y: pointer.y,
                    },
                };
                if matches!(action, Action::Click { .. })
                    && matches!(
                        session.phase,
                        AdventurePhase::Playing | AdventurePhase::ChallengePlaying
                    )
                    && (31.0..=586.0).contains(&pointer.x)
                    && (61.0..=399.0).contains(&pointer.y)
                {
                    // An out-of-range Gus press still belongs to the Gus
                    // route. It must not borrow another click's FoodDropped.
                    let gus_initial_attempt = session.board.as_ref().is_some_and(|board| {
                        board
                            .invasion
                            .as_ref()
                            .is_some_and(|wave| wave.has_registered_kind(SylvesterKind::Gus))
                    });
                    let gus_click =
                        gus_initial_attempt.then_some((pointer.x as i32, pointer.y as i32));
                    feed_press_at = Some((clock.press(get_time()), gus_click));
                }
                if matches!(action, Action::Click { .. })
                    && pointer.y > 40.0
                    && session.board.as_ref().is_some_and(|board| {
                        board.weapon_strength == 12
                            && (board
                                .invasion
                                .as_ref()
                                .is_some_and(|wave| wave.has_live_alien())
                                || !board.missiles.is_empty())
                    })
                {
                    held_fire_at = Some(clock.press(get_time()));
                }
                pending_actions.push(action);
            }
        }
        if !ui_active && is_mouse_button_released(MouseButton::Left) {
            if matches!(
                session.phase,
                AdventurePhase::TimeTrialPrestoDialog { .. }
                    | AdventurePhase::AdventurePrestoDialog { .. }
                    | AdventurePhase::ChallengePrestoDialog { .. }
                    | AdventurePhase::SandboxPrestoDialog { .. }
                    | AdventurePhase::VirtualPrestoDialog { .. }
            ) && !paused
            {
                pending_actions.push(Action::PrestoRelease {
                    pet: presto_choice_at_pointer(&session.progress.unlocked_pets, pointer),
                });
            }
            hatch_pointer_owned = false;
            feed_press_at = None;
            held_feed_at = None;
            held_fire_at = None;
        }
        if !paused
            && !ui_active
            && is_mouse_button_pressed(MouseButton::Right)
            && matches!(
                session.phase,
                AdventurePhase::TimeTrialPlaying
                    | AdventurePhase::Playing
                    | AdventurePhase::ChallengePlaying
                    | AdventurePhase::SandboxPlaying
                    | AdventurePhase::VirtualTank
            )
        {
            pending_actions.push(Action::RightClick {
                x: pointer.x,
                y: pointer.y,
            });
        }
        if matches!(
            session.phase,
            AdventurePhase::Hatch { .. } | AdventurePhase::TankFourFinaleHatch { .. }
        ) {
            let held = hatch_pointer_owned && is_mouse_button_down(MouseButton::Left);
            if held != hatch_background_down {
                pending_actions.push(Action::HatchHold { down: held });
                hatch_background_down = held;
            }
        } else {
            hatch_pointer_owned = false;
            hatch_background_down = false;
        }
        let enter_continues = matches!(
            session.phase,
            AdventurePhase::FirstTankRescue
                | AdventurePhase::TimeTrialPetSelection { .. }
                | AdventurePhase::ChallengePetSelection { .. }
                | AdventurePhase::ChallengeLost
                | AdventurePhase::ChallengeResults
                | AdventurePhase::ChallengeInterlude { .. }
                | AdventurePhase::ChallengeInvasionTutorial { .. }
                | AdventurePhase::TimeTrialTimesUp
                | AdventurePhase::TimeTrialResults
                | AdventurePhase::TimeTrialInvasionTutorial { .. }
                | AdventurePhase::InvasionTutorial { .. }
                | AdventurePhase::HelpScreen
                | AdventurePhase::Hatch { .. }
                | AdventurePhase::TankFourFinaleHatch { .. }
                | AdventurePhase::PetSelection { .. }
        ) || matches!(session.phase, AdventurePhase::GameOver { updates } | AdventurePhase::TimeTrialGameOver { updates } if updates > 30)
            || matches!(session.phase, AdventurePhase::BonusResults { ref result } if result.updates >= 30);
        if !paused && !ui_active && is_key_pressed(KeyCode::Enter) && enter_continues {
            pending_actions.push(Action::Continue);
        }
        if !paused
            && !ui_active
            && session.phase == AdventurePhase::AdventureFinaleInterlude
            && is_key_pressed(KeyCode::Enter)
        {
            if let Some(controller) = interlude.as_mut() {
                let _ = controller.back_to_menu();
            }
            pending_actions.push(Action::OpenMenu);
        }
        if !paused
            && !ui_active
            && matches!(
                session.phase,
                AdventurePhase::PetSelectionConfirmation { .. }
                    | AdventurePhase::ChallengePetSelectionConfirmation { .. }
            )
        {
            if is_key_pressed(KeyCode::Y) || is_key_pressed(KeyCode::Enter) {
                pending_actions.push(Action::ConfirmPetSelection { accept: true });
            } else if is_key_pressed(KeyCode::N) {
                pending_actions.push(Action::ConfirmPetSelection { accept: false });
            }
        }
        let save_requested = !ui_active && is_key_pressed(KeyCode::S);
        let mut exit_requested = is_quit_requested()
            || (paused && is_key_pressed(KeyCode::Q))
            || wall_deadline_reached(elapsed, options.quit_after);
        let mut events = Vec::new();
        let mut event_stamps = Vec::new();
        let mut phase_transitioned = false;
        if let Some(owner) = music.as_mut() {
            owner.sync(session, paused);
        }
        if save_requested || exit_requested {
            // Inputs already accepted by this window belong to the checkpoint.
            // Apply them without inventing an extra simulation tick on save/exit.
            let accepted_events = session.apply_actions(&pending_actions);
            if options.test_speed > 1 {
                stamp_events(
                    &mut event_stamps,
                    &accepted_events,
                    session.ticks,
                    get_time() - started,
                    "accepted_actions",
                );
            }
            events.extend(accepted_events);
            pending_actions.clear();
            if feed_press_at.is_some_and(|(_, gus_click)| arms_held_feed(&events, gus_click))
                && is_mouse_button_down(MouseButton::Left)
            {
                held_feed_at = feed_press_at.take().map(|(pressed_at, _)| pressed_at);
            }
        }
        if !exit_requested {
            clock.add_frame(get_frame_time(), paused || ui_active);
            while clock.step_due() {
                if options.test_speed > 1
                    && wall_deadline_reached(get_time() - started, options.quit_after)
                {
                    exit_requested = true;
                    // This frame's accepted input is checkpointed without a
                    // fabricated simulation step, as for a normal exit.
                    let accepted_events = session.apply_actions(&pending_actions);
                    stamp_events(
                        &mut event_stamps,
                        &accepted_events,
                        session.ticks,
                        get_time() - started,
                        "accepted_actions",
                    );
                    events.extend(accepted_events);
                    pending_actions.clear();
                    break;
                }
                if paused || ui_active {
                    session.paused_step();
                } else {
                    let before_fish_hunger: HashMap<_, _> =
                        if session.phase == AdventurePhase::VirtualTank {
                            session
                                .virtual_tank
                                .fish
                                .iter()
                                .filter(|fish| fish.alive && fish.properties.singing)
                                .map(|fish| (fish.id, fish.hunger))
                                .collect()
                        } else {
                            HashMap::new()
                        };
                    let mut step_actions = std::mem::take(&mut pending_actions);
                    let pressed_feed = feed_press_at.is_some()
                        && step_actions
                            .iter()
                            .any(|action| matches!(action, Action::Click { .. }));
                    if let Some(press_at) = held_feed_at
                        && is_mouse_button_down(MouseButton::Left)
                        && matches!(
                            session.phase,
                            AdventurePhase::Playing | AdventurePhase::ChallengePlaying
                        )
                        && !session
                            .board
                            .as_ref()
                            .and_then(|board| board.invasion.as_ref())
                            .is_some_and(|wave| {
                                wave.has_live_alien()
                                    && !wave.has_registered_kind(SylvesterKind::Gus)
                            })
                    {
                        step_actions.push(Action::HoldFeed {
                            x: pointer.x,
                            y: pointer.y,
                            elapsed_ms: clock.held_elapsed_ms(press_at, get_time()),
                        });
                    }
                    if let Some(press_at) = held_fire_at
                        && is_mouse_button_down(MouseButton::Left)
                        && matches!(
                            session.phase,
                            AdventurePhase::Playing | AdventurePhase::ChallengePlaying
                        )
                    {
                        step_actions.push(Action::HoldFire {
                            x: pointer.x,
                            y: pointer.y,
                            elapsed_ms: clock.held_elapsed_ms(press_at, get_time()),
                        });
                    }
                    let previous_phase = std::mem::discriminant(&session.phase);
                    let step_events = session.step(&step_actions);
                    if session.phase == AdventurePhase::AdventureFinaleInterlude {
                        if interlude.is_none() {
                            interlude = InterludeController::new(&session.finale_fallen_pets).ok();
                        }
                        if let Some(controller) = interlude.as_mut() {
                            let _ = controller.advance(true);
                        }
                    }
                    if session.phase == AdventurePhase::VirtualTank {
                        for fish in session
                            .virtual_tank
                            .fish
                            .iter()
                            .filter(|fish| fish.alive && fish.properties.singing)
                        {
                            if before_fish_hunger
                                .get(&fish.id)
                                .is_some_and(|old| fish.hunger > *old + 1)
                            {
                                let count = singing_feeds.entry(fish.id).or_default();
                                *count = count.wrapping_add(1);
                                if (*count).is_multiple_of(5) && !song_player.busy() {
                                    pending_song_ticks = Some(5);
                                    pending_song_kind = fish.properties.named;
                                }
                            }
                        }
                        if let Some(left) = pending_song_ticks.as_mut() {
                            *left = left.saturating_sub(1);
                            if *left == 0 {
                                pending_song_ticks = None;
                                if let Some(catalog) = song_catalog.as_ref() {
                                    let special = pending_song_kind.take();
                                    let eligible: Vec<_> = catalog
                                        .songs()
                                        .iter()
                                        .filter(|song| {
                                            if song.property("test").is_some() {
                                                return false;
                                            }
                                            match special {
                                                Some(NamedFish::Kilgore) => {
                                                    song.property("kilgore").is_some()
                                                }
                                                Some(NamedFish::Ludwig) => {
                                                    !song.is_long()
                                                        && (song.property("beethoven").is_some()
                                                            || song
                                                                .property("beethovenrare")
                                                                .is_some())
                                                }
                                                Some(NamedFish::Santa) => {
                                                    !song.is_long()
                                                        && (song.property("santa").is_some()
                                                            || song.property("santarare").is_some())
                                                }
                                                _ => {
                                                    !song.is_long()
                                                        && song.property("beethovenrare").is_none()
                                                        && song.property("santarare").is_none()
                                                }
                                            }
                                        })
                                        .collect();
                                    if let Some(song) =
                                        eligible.get(song_sequence % eligible.len().max(1))
                                    {
                                        let selected = if song_sequence % 5 == 4 {
                                            song.property("short")
                                                .and_then(|stem| {
                                                    catalog.songs().iter().find(|candidate| {
                                                        candidate.is_long()
                                                            && candidate.property("long")
                                                                == Some(stem)
                                                    })
                                                })
                                                .unwrap_or(*song)
                                        } else {
                                            *song
                                        };
                                        song_sequence += 1;
                                        song_id = song_id.saturating_add(1);
                                        let applause = selected.is_long()
                                            && !matches!(
                                                special,
                                                Some(NamedFish::Kilgore | NamedFish::Santa)
                                            );
                                        if song_player
                                            .start(
                                                selected,
                                                song_id,
                                                (get_time() * 1000.0) as u64,
                                                applause,
                                            )
                                            .is_ok()
                                        {
                                            song_title = selected
                                                .property("title")
                                                .unwrap_or(&selected.file_name)
                                                .to_owned();
                                            if selected.is_long() {
                                                song_title.push_str(" (extended)");
                                            }
                                            song_title_until = get_time() + 5.0;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if let Some(owner) = music.as_mut() {
                        owner.sync(session, paused);
                    }
                    phase_transitioned |= previous_phase != std::mem::discriminant(&session.phase);
                    if feed_press_at
                        .is_some_and(|(_, gus_click)| arms_held_feed(&step_events, gus_click))
                        && is_mouse_button_down(MouseButton::Left)
                    {
                        held_feed_at = feed_press_at.take().map(|(pressed_at, _)| pressed_at);
                    }
                    if pressed_feed {
                        feed_press_at = None;
                    }
                    if step_events.iter().any(|event| {
                        matches!(
                            event,
                            Event::Invasion {
                                event: InvasionEvent::AlienSpawned { .. }
                                    | InvasionEvent::BattleEnded,
                                ..
                            }
                        )
                    }) || !matches!(
                        session.phase,
                        AdventurePhase::Playing | AdventurePhase::ChallengePlaying
                    ) {
                        feed_press_at = None;
                        held_feed_at = None;
                        held_fire_at = None;
                    }
                    if options.test_speed > 1 {
                        stamp_events(
                            &mut event_stamps,
                            &step_events,
                            session.ticks,
                            get_time() - started,
                            "step",
                        );
                    }
                    events.extend(step_events);
                }
                clock.consume_step(paused || ui_active);
                if phase_transitioned
                    || events.iter().any(|event| {
                        matches!(
                            event,
                            Event::HatchStarted { .. } | Event::TankFourFinaleHatchStarted { .. }
                        )
                    })
                {
                    clock.clear_backlog();
                    break;
                }
            }
        }
        // Bilaterus observes its passive connector pose during the board's
        // render pass. Publish that durable observation before saving or
        // recording this frame, including when the board is paused.
        let observed_connector_change = !exit_requested
            && matches!(
                session.phase,
                AdventurePhase::Playing
                    | AdventurePhase::TimeTrialPlaying
                    | AdventurePhase::ChallengePlaying
                    | AdventurePhase::ChallengePrestoDialog { .. }
                    | AdventurePhase::ChallengeInvasionTutorial { .. }
                    | AdventurePhase::ChallengeResults
                    | AdventurePhase::ChallengeLost
                    | AdventurePhase::ChallengeInterlude { .. }
                    | AdventurePhase::TimeTrialInvasionTutorial { .. }
                    | AdventurePhase::TimeTrialTimesUp
                    | AdventurePhase::TimeTrialResults
                    | AdventurePhase::TimeTrialGameOver { .. }
                    | AdventurePhase::FirstTankRescue
                    | AdventurePhase::InvasionTutorial { .. }
                    | AdventurePhase::GameOver { .. }
            )
            && session
                .board
                .as_mut()
                .is_some_and(AdventureState::observe_rendered_bilaterus_connectors);
        let save_due = phase_transitioned
            || observed_connector_change
            || save_requested
            || exit_requested
            || events.iter().any(|event| {
                matches!(
                    event,
                    Event::FirstTankRescueStarted { .. }
                        | Event::GameOverStarted { .. }
                        | Event::GameSelectorOpened { .. }
                        | Event::TimeTrialStarted { .. }
                        | Event::TimeTrialPetAcquired { .. }
                        | Event::PrestoChanged { .. }
                        | Event::WalterPunched { .. }
                        | Event::TimeTrialExpired { .. }
                        | Event::TimeTrialShellsCredited { .. }
                        | Event::ChallengeStarted { .. }
                        | Event::ChallengeEggBought { .. }
                        | Event::ChallengePricesInflated { .. }
                        | Event::ChallengeCompleted { .. }
                        | Event::ChallengeShellsCredited { .. }
                        | Event::ChallengeLost { .. }
                        | Event::VirtualTankChanged { .. }
                        | Event::VirtualCoinCollected { .. }
                        | Event::Invasion {
                            event: InvasionEvent::ModalOpened(_),
                            ..
                        }
                        | Event::HatchStarted { .. }
                        | Event::TankFourFinaleHatchStarted { .. }
                        | Event::StageStarted { .. }
                        | Event::StageResultRecorded { .. }
                        | Event::RescueGuppyGranted { .. }
                        | Event::BonusResultsCommitted { .. }
                        | Event::BonusPurchaseOffered { .. }
                        | Event::BonusPurchaseCancelled { .. }
                        | Event::BonusPurchaseCommitted { .. }
                        | Event::Bonus {
                            event: crate::bonus::BonusEvent::Started { .. }
                                | crate::bonus::BonusEvent::Claimed { .. }
                                | crate::bonus::BonusEvent::Credited { .. },
                            ..
                        }
                )
            });
        if save_due {
            session.virtual_saved_at_seconds = wall_seconds();
        }
        healing_warning_until_tick = healing_warning_after_events(
            healing_warning_until_tick,
            &events,
            matches!(session.phase, AdventurePhase::Playing),
        );
        let mut music_reports = presentation.play(&events, music.as_ref());
        let song_now_ms = (get_time() * 1000.0) as u64;
        if paused || ui_active || session.phase != AdventurePhase::VirtualTank {
            song_player.pause(song_now_ms);
        } else {
            song_player.resume(song_now_ms);
            for command in song_player.advance(song_now_ms) {
                if let (Some(owner), Some(bytes)) = (
                    music.as_ref(),
                    presentation.sounds.get(command.sample.sound_id()),
                ) && let Some(report) = owner.play_tone(
                    bytes.clone(),
                    command.song_id,
                    command.pitch_semitones,
                    command.gain,
                ) {
                    music_reports.push(report);
                }
            }
        }
        if let Some(owner) = music.as_mut() {
            owner.sync(session, paused);
            if session.phase == AdventurePhase::AdventureFinaleInterlude {
                let cue = match interlude.as_ref().map(InterludeController::music_stage) {
                    Some(MusicStage::Lullaby) => MusicCue::Lullaby,
                    _ => MusicCue::Interlude,
                };
                owner.sync_cue(Some(cue), paused);
            }
            music_reports.extend(owner.drain_reports());
            if let Some(reason) = owner.unavailable() {
                music_unavailable.get_or_insert_with(|| reason.to_owned());
            }
        }
        for report in &music_reports {
            match report {
                MusicReport::Unavailable { reason } => eprintln!("Music unavailable: {reason}"),
                MusicReport::EffectFailed { reason } | MusicReport::Overload { reason } => {
                    eprintln!("Audio warning: {reason}");
                }
                _ => {}
            }
        }
        if let Some(evidence) = evidence.as_mut() {
            let evidence_elapsed = if options.test_speed > 1 && exit_requested {
                get_time() - started
            } else {
                elapsed
            };
            evidence.record(&events, &event_stamps, session, evidence_elapsed, paused)?;
            evidence.record_music(&music_reports, evidence_elapsed, session)?;
        }
        if exit_requested {
            profiles.record_events(&options.save_dir.join("profiles.json"), &events)?;
            cli::save_profiles_checkpoint(&options, &profiles)?;
            break;
        }
        clear_background(BLACK);
        let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, 640.0, 480.0));
        // Camera2D's screen matrix flips Y already in macroquad 0.4.16.
        // A positive zoom makes logical Y grow down, matching pointer coordinates.
        camera.zoom.y = 2.0 / 480.0;
        camera.viewport = Some((
            offset.x as i32,
            offset.y as i32,
            (640.0 * scale) as i32,
            (480.0 * scale) as i32,
        ));
        set_camera(&camera);
        let capture_tick = session.ticks;
        presentation.draw(
            session,
            paused,
            pointer,
            healing_warning_until_tick,
            interlude.as_ref(),
        );
        if session.phase == AdventurePhase::VirtualTank && get_time() < song_title_until {
            draw_text(&song_title, 24.0, 105.0, 20.0, YELLOW);
        }
        if session.phase == AdventurePhase::GameSelector {
            selector_ui.draw(
                &profiles,
                settings.settings,
                RuntimeSettingsBackend {
                    audio: music.as_ref(),
                }
                .capabilities(),
            );
        }
        if settings.settings.custom_cursor {
            draw_circle(pointer.x, pointer.y, 5.0, YELLOW);
            draw_circle_lines(pointer.x, pointer.y, 8.0, 1.5, BLACK);
        }
        if options.test_speed > 1 {
            draw_rectangle(8.0, 449.0, 132.0, 22.0, Color::new(0.0, 0.0, 0.0, 0.8));
            draw_text(
                format!("Test speed {}x", options.test_speed),
                12.0,
                465.0,
                18.0,
                YELLOW,
            );
        }
        if music_unavailable.is_some() {
            draw_text("Music unavailable", 425.0, 470.0, 18.0, YELLOW);
        }
        set_default_camera();
        if let Some(evidence) = evidence.as_ref() {
            let request = evidence.root.join("capture.request");
            if is_key_pressed(KeyCode::F12) || request.exists() {
                evidence.capture(capture_tick);
                if request.exists() {
                    fs::remove_file(request)?;
                }
            }
        }
        profiles.record_events(&options.save_dir.join("profiles.json"), &events)?;
        if save_due {
            cli::save_profiles_checkpoint(&options, &profiles)?;
        }
        next_frame().await;
    }
    if let Some(current) = profiles.current_mut() {
        current.session.virtual_saved_at_seconds = wall_seconds();
    }
    cli::save_profiles_checkpoint(&options, &profiles)?;
    let after = install::identify(&game_root)?;
    if after.inventory_sha256 != identity.inventory_sha256 {
        return Err("Owned installation changed during run; inspect before continuing".into());
    }
    if let (Some(evidence), Some(current)) = (evidence.as_mut(), profiles.current()) {
        let session = &current.session;
        evidence.record(&[], &[], session, get_time() - started, paused)?;
        cli::write_json(
            &evidence.root.join("final.local.json"),
            &serde_json::json!({"session":session,"game_after":after,"elapsed_seconds":get_time()-started,"session_elapsed_seconds":session.ticks.saturating_sub(start_session_tick) as f64 * f64::from(TICK_MS) / 1000.0,"test_speed":options.test_speed,"time_mode":options.time_mode(),"game_unchanged":true}),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod feed_input_tests {
    use super::*;

    #[test]
    fn bonus_purchase_display_uses_debited_wallet_after_immutable_award_countup() {
        let mut result = BonusResult {
            origin_tank: 1,
            origin_level: 6,
            earned: 5_000,
            previous_balance: 20_000,
            updates: 130,
            purchase: PurchaseReceipt::new(0),
        };
        assert_eq!(bonus_results_display_balance(&result, 25_000), 25_000);
        result.purchase.purchased = true;
        assert_eq!(bonus_results_display_balance(&result, 5_000), 5_000);
        assert_eq!((result.previous_balance, result.earned), (20_000, 5_000));
    }

    #[test]
    fn earned_presto_card_can_be_selected_for_completed_adventure_replay() {
        let unlocked = [PetKind::Stinky, PetKind::Presto];
        assert_eq!(
            pet_at_pointer(&unlocked, vec2(50.0, 80.0)),
            Some(PetKind::Stinky)
        );
        assert_eq!(
            pet_at_pointer(&unlocked, vec2(50.0, 165.0)),
            Some(PetKind::Presto)
        );
    }

    #[test]
    fn earned_eighth_pet_accepts_the_observed_native_click() {
        let unlocked = [
            PetKind::Stinky,
            PetKind::Niko,
            PetKind::Itchy,
            PetKind::Prego,
            PetKind::Zorf,
            PetKind::Clyde,
            PetKind::Vert,
            PetKind::Rufus,
        ];
        assert_eq!(
            pet_at_pointer(&unlocked, vec2(165.333_33, 248.0)),
            Some(PetKind::Rufus)
        );
        assert_eq!(
            pet_at_pointer(&unlocked, vec2(165.333_33, 165.0)),
            Some(PetKind::Vert)
        );
    }

    #[test]
    fn tenth_pet_uses_the_source_second_column_fifth_row() {
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
        assert_eq!(
            pet_at_pointer(&unlocked, vec2(165.0, 414.0)),
            Some(PetKind::Wadsworth)
        );
    }

    #[test]
    fn eleventh_pet_uses_first_right_hand_card_without_displacing_tenth() {
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
            PetKind::Seymour,
        ];
        assert_eq!(pet_card_rect(10), Rect::new(425.0, 41.0, 90.0, 83.0));
        assert_eq!(
            pet_at_pointer(&unlocked, vec2(470.0, 82.0)),
            Some(PetKind::Seymour)
        );
        assert_eq!(
            pet_at_pointer(&unlocked, vec2(165.0, 414.0)),
            Some(PetKind::Wadsworth)
        );
        assert_eq!(pet_at_pointer(&unlocked, vec2(320.0, 82.0)), None);
    }

    #[test]
    fn twelfth_pet_uses_second_right_hand_card() {
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
            PetKind::Seymour,
            PetKind::Shrapnel,
        ];
        assert_eq!(pet_card_rect(11), Rect::new(425.0, 124.0, 90.0, 83.0));
        assert_eq!(
            pet_at_pointer(&unlocked, vec2(470.0, 165.0)),
            Some(PetKind::Shrapnel)
        );
        assert_eq!(
            pet_at_pointer(&unlocked, vec2(470.0, 82.0)),
            Some(PetKind::Seymour)
        );
    }

    #[test]
    fn thirteenth_pet_uses_third_right_hand_card() {
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
            PetKind::Seymour,
            PetKind::Shrapnel,
            PetKind::Gumbo,
        ];
        assert_eq!(pet_card_rect(12), Rect::new(425.0, 207.0, 90.0, 83.0));
        assert_eq!(
            pet_at_pointer(&unlocked, vec2(470.0, 248.0)),
            Some(PetKind::Gumbo)
        );
        assert_eq!(
            pet_at_pointer(&unlocked, vec2(470.0, 165.0)),
            Some(PetKind::Shrapnel)
        );
    }

    #[test]
    fn blip_uses_fourth_right_hand_card_without_displacing_gumbo() {
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
            PetKind::Seymour,
            PetKind::Shrapnel,
            PetKind::Gumbo,
            PetKind::Blip,
        ];
        assert_eq!(pet_card_rect(13), Rect::new(425.0, 290.0, 90.0, 83.0));
        assert_eq!(
            pet_at_pointer(&unlocked, vec2(470.0, 331.0)),
            Some(PetKind::Blip)
        );
        assert_eq!(
            pet_at_pointer(&unlocked, vec2(470.0, 248.0)),
            Some(PetKind::Gumbo)
        );
    }

    #[test]
    fn rhubarb_uses_fifth_right_hand_card_and_preserves_blip_hit_area() {
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
            PetKind::Seymour,
            PetKind::Shrapnel,
            PetKind::Gumbo,
            PetKind::Blip,
            PetKind::Rhubarb,
        ];
        assert_eq!(pet_card_rect(14), Rect::new(425.0, 373.0, 90.0, 83.0));
        assert_eq!(
            pet_at_pointer(&unlocked, vec2(470.0, 414.0)),
            Some(PetKind::Rhubarb)
        );
        assert_eq!(
            pet_at_pointer(&unlocked, vec2(470.0, 331.0)),
            Some(PetKind::Blip)
        );
    }

    #[test]
    fn nimbus_starts_the_next_column_without_displacing_rhubarb() {
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
            PetKind::Seymour,
            PetKind::Shrapnel,
            PetKind::Gumbo,
            PetKind::Blip,
            PetKind::Rhubarb,
            PetKind::Nimbus,
        ];
        assert_eq!(pet_card_rect(15), Rect::new(519.0, 41.0, 90.0, 83.0));
        assert_eq!(
            pet_at_pointer(&unlocked, vec2(560.0, 82.0)),
            Some(PetKind::Nimbus)
        );
        assert_eq!(
            pet_at_pointer(&unlocked, vec2(470.0, 414.0)),
            Some(PetKind::Rhubarb)
        );
    }

    #[test]
    fn amp_uses_second_card_in_third_column_and_preserves_nimbus() {
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
            PetKind::Seymour,
            PetKind::Shrapnel,
            PetKind::Gumbo,
            PetKind::Blip,
            PetKind::Rhubarb,
            PetKind::Nimbus,
            PetKind::Amp,
        ];
        assert_eq!(pet_card_rect(16), Rect::new(519.0, 124.0, 90.0, 83.0));
        assert_eq!(
            pet_at_pointer(&unlocked, vec2(560.0, 165.0)),
            Some(PetKind::Amp)
        );
        assert_eq!(
            pet_at_pointer(&unlocked, vec2(560.0, 82.0)),
            Some(PetKind::Nimbus)
        );
    }

    #[test]
    fn alien_health_bar_clips_visual_width_without_changing_health() {
        // W1 Alien::DrawAlien multiplies the image width by HP/max HP,
        // truncates to an integer and bounds the source rectangle.
        assert_eq!(health_bar_visible_width(-4.0, 100.0, 127.0), 0.0);
        assert_eq!(health_bar_visible_width(50.0, 100.0, 127.0), 63.0);
        assert_eq!(health_bar_visible_width(100.0, 100.0, 127.0), 127.0);
        assert_eq!(health_bar_visible_width(150.0, 100.0, 127.0), 127.0);
    }

    #[test]
    fn healing_warning_requires_an_accepted_shot_and_clears_on_exit() {
        let entered = Event::Invasion {
            tick: 11,
            event: InvasionEvent::PsychosquidPhaseChanged {
                id: 7,
                healing: true,
                forced: false,
            },
        };
        let hit = Event::Invasion {
            tick: 12,
            event: InvasionEvent::PsychosquidHealingHit {
                id: 7,
                health: 269.0,
            },
        };
        let exited = Event::Invasion {
            tick: 13,
            event: InvasionEvent::PsychosquidPhaseChanged {
                id: 7,
                healing: false,
                forced: false,
            },
        };
        assert_eq!(healing_warning_after_events(None, &[entered], true), None);
        assert_eq!(healing_warning_after_events(None, &[hit], true), Some(512));
        assert_eq!(
            healing_warning_after_events(Some(512), &[exited], true),
            None
        );
        assert_eq!(healing_warning_after_events(Some(512), &[], false), None);
    }

    #[test]
    fn missile_death_audio_is_keyed_to_the_same_target_and_tick() {
        let events = [
            Event::GrubberDied {
                tick: 41,
                grubber_id: 7,
            },
            Event::MissileImpacted {
                tick: 41,
                missile_id: 9,
                target_id: 7,
            },
        ];
        assert!(death_has_missile_impact(&events, 41, 7));
        assert!(!death_has_missile_impact(&events, 40, 7));
        assert!(!death_has_missile_impact(&events, 41, 8));
    }

    #[test]
    fn gus_hold_arms_only_from_the_pressed_click_route() {
        let unrelated = [
            Event::GusInitialFeedAttempt {
                tick: 9,
                x: 300,
                y: 200,
            },
            Event::Rejected {
                tick: 9,
                reason: crate::sim::Rejection::FoodCapacity,
            },
            Event::FoodDropped {
                tick: 9,
                food_id: 17,
                balance: 200,
                potion: false,
            },
        ];
        assert!(!arms_held_feed(&unrelated, Some((350, 200))));
        assert!(!arms_held_feed(&unrelated, Some((350, 390))));
        assert!(arms_held_feed(&unrelated, Some((300, 200))));
        assert!(arms_held_feed(&unrelated, None));
    }
}
