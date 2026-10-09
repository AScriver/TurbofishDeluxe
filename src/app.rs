use crate::{
    adventure::{AdventurePhase, AdventureSession},
    alien::SylvesterKind,
    assets::{GameAssets, SoundData},
    bonus::{BonusResult, BonusState, ShellKind, ShellState},
    cli::{self, Options},
    fish_pet::FishPetKind,
    font::BitmapFont,
    install::{self, InstallIdentity},
    invasion::{InvasionEvent, InvasionTip},
    oscar::OscarPose,
    sim::{Action, AdventureState, CoinKind, Event, FishPose, FishSize, PetKind, TICK_MS},
};
use macroquad::{
    audio::{PlaySoundParams, Sound, load_sound_from_bytes, play_sound},
    prelude::*,
};
use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

const IMAGE_IDS: &[&str] = &[
    "IMAGE_AQUARIUM1",
    "IMAGE_AQUARIUM2",
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
    "SOUND_ZAP",
    "SOUND_NIKOOPEN",
    "SOUND_NIKOCLOSE",
    "SOUND_PEARL",
    "SOUND_CHOMP",
    "SOUND_DIE",
    "SOUND_PUNCH",
    "SOUND_BABY",
    "SOUND_BONUSCOLLECT",
    "SOUND_BONUSCOUNT",
];

pub struct Presentation {
    images: HashMap<String, Texture2D>,
    sounds: HashMap<String, Sound>,
    fonts: HashMap<String, RenderedFont>,
}

// W1 PetsScreen places pet IDs 0..4 in the first column and 5..9 in the
// second. Use this for both drawing and hit testing.
fn pet_card_rect(index: usize) -> Rect {
    let column = index / 5;
    let row = index % 5;
    Rect::new(
        25.0 + column as f32 * 94.0,
        41.0 + row as f32 * 83.0,
        90.0,
        83.0,
    )
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
                let sound = load_sound_from_bytes(&bytes)
                    .await
                    .map_err(|error| format!("{id}: {error:?}"))?;
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
        Ok(Self {
            images,
            sounds,
            fonts,
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

    fn play(&self, events: &[Event]) {
        for event in events {
            let id = match event {
                Event::FoodDropped { potion: false, .. } => "SOUND_DROPFOOD",
                Event::PotionExploded { .. } => "SOUND_EXPLOSION1",
                Event::FoodEaten { .. } => "SOUND_SLURP",
                Event::FishGrew { .. } => "SOUND_GROW",
                Event::CoinCredited { .. } => "SOUND_POINTS",
                Event::GuppyBought { .. }
                | Event::EggBought { .. }
                | Event::FoodQualityBought { .. }
                | Event::FoodQuantityBought { .. }
                | Event::PotionBought { .. }
                | Event::StarcatcherAteStar { .. }
                | Event::WeaponBought { .. } => "SOUND_BUY",
                Event::OscarBought { .. } | Event::StarcatcherBought { .. } => "SOUND_GROW",
                Event::OscarAteGuppy { .. } => "SOUND_CHOMP",
                Event::OscarDied { .. } | Event::StarcatcherDied { .. } => "SOUND_DIE",
                Event::FishPetHit { sound: true, .. } => "SOUND_PUNCH",
                Event::PregoBirth { .. } => "SOUND_BABY",
                Event::HatchOpened { .. } => "SOUND_HATCH",
                Event::Invasion { event, .. } => match event {
                    InvasionEvent::WarningStarted(_) => "SOUND_AWOOGA",
                    InvasionEvent::AlienSpawned { .. } => "SOUND_ROAR",
                    InvasionEvent::AlienHit { .. } => "SOUND_HIT",
                    InvasionEvent::AlienDefeated { .. } => "SOUND_EXPLOSION1",
                    InvasionEvent::LaserFired { .. } => "SOUND_ZAP",
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
                | Event::RescueGuppyGranted { .. }
                | Event::PetSelectionChanged { .. }
                | Event::PetSelectionConfirmation { .. } => "SOUND_BUTTONCLICK",
                _ => continue,
            };
            if let Some(sound) = self.sounds.get(id) {
                play_sound(
                    sound,
                    PlaySoundParams {
                        looped: false,
                        volume: 0.75,
                    },
                );
            }
        }
    }

    fn draw_board(&self, state: &AdventureState) {
        self.sprite(
            if state.tank == 2 {
                "IMAGE_AQUARIUM2"
            } else {
                "IMAGE_AQUARIUM1"
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
            self.sprite(
                "IMAGE_FOOD",
                food.x - 5.0,
                food.y - 4.0,
                Some(Rect::new(
                    (food.frame / food.animation_period % 10) as f32 * 40.0,
                    f32::from(food.quality) * 40.0,
                    40.0,
                    40.0,
                )),
                false,
                1.0,
                1.0,
            );
        }
        for fish in state.fish.iter().filter(|fish| fish.alive) {
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
        let aliens_present = state
            .invasion
            .as_ref()
            .is_some_and(|wave| wave.alien.is_some());
        for pet in &state.fish_pets {
            let image = match pet.kind {
                FishPetKind::Itchy => "IMAGE_ITCHY",
                FishPetKind::Prego => "IMAGE_PREGO",
                FishPetKind::Zorf => "IMAGE_ZORF",
            };
            self.sprite(
                image,
                pet.widget_x as f32,
                pet.widget_y as f32,
                Some(Rect::new(
                    f32::from(pet.sprite_frame()) * 80.0,
                    f32::from(pet.sprite_row(aliens_present)) * 80.0,
                    80.0,
                    80.0,
                )),
                pet.facing_right(),
                1.0,
                1.0,
            );
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
        if let Some(wave) = &state.invasion {
            if let Some(warp) = &wave.warp {
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
            if let Some(alien) = &wave.alien
                && alien.spawn_ticks <= 9
            {
                let inset = if alien.spawn_ticks > 0 {
                    (f32::from(alien.spawn_ticks) / 10.0 * 160.0) as i32
                } else {
                    0
                };
                let size = 160 - inset;
                if size > 0 {
                    let source = Rect::new(
                        f32::from(alien.frame) * 160.0,
                        f32::from(alien.sprite_row()) * 160.0,
                        160.0,
                        160.0,
                    );
                    let x = alien.widget_x as f32 + (inset / 2) as f32;
                    let y = alien.widget_y as f32 + (inset / 2) as f32;
                    let scale = size as f32 / 160.0;
                    let alien_image = if alien.kind == SylvesterKind::Balrog {
                        "IMAGE_BALROG"
                    } else {
                        "IMAGE_SYLV"
                    };
                    self.sprite(
                        alien_image,
                        x,
                        y,
                        Some(source),
                        alien.facing_right(),
                        scale,
                        1.0,
                    );
                    if alien.hit_flash() && alien.spawn_ticks == 0 {
                        self.sprite(
                            alien_image,
                            x,
                            y,
                            Some(source),
                            alien.facing_right(),
                            scale,
                            (f32::from(alien.hit_ticks) * 25.0 / 255.0).min(1.0),
                        );
                    }
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
            if let Some(body) = &wave.dead_alien {
                self.sprite(
                    if body.kind == SylvesterKind::Balrog {
                        "IMAGE_BALROG"
                    } else {
                        "IMAGE_SYLV"
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
        if let Some(stinky) = &state.stinky {
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
        if let Some(clyde) = &state.clyde {
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
        if let Some(niko) = &state.niko {
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
            let row = match coin.kind {
                CoinKind::Silver => 0.0,
                CoinKind::Gold => 1.0,
                CoinKind::Diamond | CoinKind::DiamondPenta => 3.0,
                CoinKind::Star => 2.0,
            };
            let alpha = if coin.fade_ticks > 0 {
                f32::from(coin.fade_ticks) / 5.0
            } else {
                1.0
            };
            self.sprite(
                "IMAGE_MONEY",
                coin.x,
                coin.y,
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
        self.sprite("IMAGE_MENUBAR", 0.0, 0.0, None, false, 1.0, 1.0);
        if state.guppy_unlocked {
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
            self.fonts["Pix118"].text("100", 33.0, 58.0, Color::from_rgba(110, 250, 110, 255));
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
                self.fonts["Pix118"].text("200", 100.0, 58.0, Color::from_rgba(110, 250, 110, 255));
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
                if state.upgrades.quantity < 9 {
                    "300"
                } else {
                    "MAX"
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
            self.fonts["Pix118"].text("1000", 229.0, 58.0, Color::from_rgba(110, 250, 110, 255));
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
                if state.potion_armed { "READY" } else { "250" },
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
            self.fonts["Pix118"].text("750", 304.0, 58.0, Color::from_rgba(110, 250, 110, 255));
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
                    "1000",
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
        self.sprite("IMAGE_AQUARIUM1", 0.0, 0.0, None, false, 1.0, 1.0);
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

    fn draw_bonus_results(&self, result: &BonusResult) {
        self.sprite("IMAGE_HATCHSCREEN", 0.0, 0.0, None, false, 1.0, 1.0);
        let title = &self.images["IMAGE_SCREENTITLE"];
        self.image_box(
            "IMAGE_SCREENTITLE",
            Rect::new(0.0, 0.0, title.width(), title.height()),
            Rect::new(20.0, 0.0, 600.0, title.height()),
        );
        self.centered_text(
            "JungleFever17outline",
            "BONUS RESULTS",
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
            &result.presented_balance().to_string(),
            398.0,
            364.0,
            YELLOW,
        );
        self.sprite("IMAGE_HATCHREFLECTION", 240.0, 60.0, None, false, 1.0, 1.0);
        let height = self.images["IMAGE_MAINBUTTON"].height();
        self.main_button(
            Rect::new(186.0, 445.0, 264.0, height),
            if result.updates >= 30 {
                "Click Here To Continue"
            } else {
                "Please Wait..."
            },
        );
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
            };
            self.sprite(
                id,
                278.0,
                y,
                Some(Rect::new(column as f32 * 80.0, 0.0, 80.0, 80.0)),
                false,
                1.0,
                1.0,
            );
            self.centered_text(
                "JungleFever15outline",
                match pet {
                    PetKind::Stinky => "STINKY the Snail",
                    PetKind::Niko => "NIKO the Oyster",
                    PetKind::Itchy => "ITCHY the Swordfish",
                    PetKind::Prego => "PREGO the Momma Fish",
                    PetKind::Zorf => "ZORF the Sea Horse",
                    PetKind::Clyde => "CLYDE the Jellyfish",
                    PetKind::Vert => "VERT the Skeleton",
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
        for (index, pet) in session.progress.unlocked_pets.iter().take(7).enumerate() {
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
            };
            let image = &self.images[icon];
            let column = if matches!(*pet, PetKind::Niko | PetKind::Vert) {
                let phase = session.ticks % 18;
                if phase > 9 { 18 - phase } else { phase }
            } else if *pet == PetKind::Clyde {
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
            };
            let column = if matches!(pet, PetKind::Niko | PetKind::Vert) {
                let phase = session.ticks % 18;
                if phase > 9 { 18 - phase } else { phase }
            } else if pet == PetKind::Clyde {
                session.ticks / 4 % 10
            } else {
                session.ticks % 20 / 2
            };
            self.sprite(
                image,
                280.0,
                y,
                Some(Rect::new(column as f32 * 80.0, 0.0, 80.0, 80.0)),
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
                "to take to the next level",
                90.0,
                WHITE,
            );
        }
        let height = self.images["IMAGE_MAINBUTTON"].height();
        self.main_button(Rect::new(225.0, 250.0, 186.0, height), "Continue");
        self.main_button(Rect::new(525.0, 4.0, 80.0, height), "Menu");
    }

    fn draw(&self, session: &AdventureSession, paused: bool, pointer: Vec2) {
        match session.phase {
            AdventurePhase::Hatch { pet, updates } => self.draw_hatch(pet, updates),
            AdventurePhase::Bonus { ref state } => self.draw_bonus(state),
            AdventurePhase::BonusResults { ref result } => self.draw_bonus_results(result),
            AdventurePhase::PetSelection { ref selected }
            | AdventurePhase::PetSelectionConfirmation { ref selected } => {
                self.draw_pet_selection(session, selected, pointer);
            }
            AdventurePhase::Playing
            | AdventurePhase::FirstTankRescue
            | AdventurePhase::InvasionTutorial { .. }
            | AdventurePhase::GameOver { .. } => {
                if let Some(board) = &session.board {
                    self.draw_board(board);
                }
            }
            AdventurePhase::GameSelector | AdventurePhase::HelpScreen => {
                self.sprite("IMAGE_HATCHSCREEN", 0.0, 0.0, None, false, 1.0, 1.0);
            }
        }
        if matches!(
            session.phase,
            AdventurePhase::PetSelectionConfirmation { .. }
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
        if let AdventurePhase::InvasionTutorial { tip } = session.phase {
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

struct Evidence {
    root: PathBuf,
    events: BufWriter<File>,
    last_state_tick: u64,
    snapshot_deferred: bool,
    last_paused: bool,
}

impl Evidence {
    fn open(
        root: &Path,
        identity: &InstallIdentity,
        session: &AdventureSession,
        seed: u64,
    ) -> Result<Self, Box<dyn Error>> {
        fs::create_dir_all(root)?;
        cli::write_json(
            &root.join("identity.local.json"),
            &serde_json::json!({"game": identity, "rust_executable_sha256": install::sha256(&fs::read(std::env::current_exe()?)?), "seed": seed, "tick_ms": TICK_MS, "retail_rng_equivalent": false, "start": session}),
        )?;
        Ok(Self {
            root: root.into(),
            events: BufWriter::new(File::create(root.join("events.local.jsonl"))?),
            last_state_tick: u64::MAX,
            snapshot_deferred: false,
            last_paused: false,
        })
    }
    fn record(
        &mut self,
        events: &[Event],
        session: &AdventureSession,
        elapsed: f64,
        paused: bool,
    ) -> Result<(), Box<dyn Error>> {
        for event in events {
            serde_json::to_writer(
                &mut self.events,
                &serde_json::json!({"elapsed_seconds": elapsed,"session_tick":session.ticks,"event":event}),
            )?;
            writeln!(&mut self.events)?;
        }
        self.events.flush()?;
        if self.last_state_tick != session.ticks || self.last_paused != paused || !events.is_empty()
        {
            let publication = cli::write_json(
                &self.root.join("state.local.json"),
                &serde_json::json!({"elapsed_seconds":elapsed, "paused":paused, "session_tick":session.ticks, "phase":session.phase, "progress":session.progress, "state":session.board}),
            );
            match publication {
                Ok(()) => {
                    if self.snapshot_deferred {
                        serde_json::to_writer(
                            &mut self.events,
                            &serde_json::json!({"diagnostic":"snapshot_publication_recovered","tick":session.ticks,"elapsed_seconds":elapsed}),
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
                            &serde_json::json!({"diagnostic":"snapshot_publication_deferred","tick":session.ticks,"elapsed_seconds":elapsed,"error":error.to_string()}),
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
    fn capture(&self, tick: u64) {
        let path = self.root.join(format!("frame-{tick:06}.png"));
        get_screen_data().export_png(&path.to_string_lossy());
    }
}

pub async fn run(
    options: Options,
    game_root: PathBuf,
    identity: InstallIdentity,
    assets: GameAssets,
    mut session: AdventureSession,
) -> Result<(), Box<dyn Error>> {
    let presentation = Presentation::load(&game_root, &assets, options.muted).await?;
    let mut evidence = options
        .evidence_dir
        .as_ref()
        .map(|root| Evidence::open(root, &identity, &session, options.seed))
        .transpose()?;
    let started = get_time();
    let mut accumulator = 0.0_f64;
    let mut pending_actions = Vec::new();
    let mut paused = false;
    let mut hatch_pointer_owned = false;
    let mut hatch_background_down = false;
    let mut feed_press_at = None::<f64>;
    let mut held_feed_at = None::<f64>;
    let mut held_fire_at = None::<f64>;
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
        if is_key_pressed(KeyCode::Escape) {
            paused = !paused;
            accumulator = 0.0;
            feed_press_at = None;
            held_feed_at = None;
            held_fire_at = None;
        }
        let button_height = presentation.images["IMAGE_MAINBUTTON"].height();
        let menu_rect = if matches!(
            session.phase,
            AdventurePhase::Hatch { .. } | AdventurePhase::PetSelection { .. }
        ) {
            Rect::new(525.0, 4.0, 80.0, button_height)
        } else {
            Rect::new(525.0, 3.0, 101.0, 29.0)
        };
        if is_mouse_button_pressed(MouseButton::Left) {
            if menu_rect.contains(pointer)
                && matches!(
                    session.phase,
                    AdventurePhase::Playing
                        | AdventurePhase::Hatch { .. }
                        | AdventurePhase::PetSelection { .. }
                )
            {
                if matches!(
                    session.phase,
                    AdventurePhase::Hatch { .. } | AdventurePhase::PetSelection { .. }
                ) {
                    pending_actions.push(Action::OpenMenu);
                } else {
                    paused = !paused;
                    accumulator = 0.0;
                }
                feed_press_at = None;
                held_feed_at = None;
                held_fire_at = None;
            } else if !paused {
                let action = match session.phase {
                    AdventurePhase::BonusResults { ref result }
                        if result.updates >= 30
                            && Rect::new(186.0, 445.0, 264.0, button_height).contains(pointer) =>
                    {
                        Action::Continue
                    }
                    AdventurePhase::PetSelection { .. } => {
                        let pet_at_pointer = session
                            .progress
                            .unlocked_pets
                            .iter()
                            .take(7)
                            .enumerate()
                            .find(|(index, _)| pet_card_rect(*index).contains(pointer))
                            .map(|(_, pet)| *pet);
                        if let Some(pet) = pet_at_pointer {
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
                    AdventurePhase::PetSelectionConfirmation { .. }
                        if Rect::new(155.0, 300.0, 145.0, button_height).contains(pointer) =>
                    {
                        Action::ConfirmPetSelection { accept: true }
                    }
                    AdventurePhase::PetSelectionConfirmation { .. }
                        if Rect::new(340.0, 300.0, 145.0, button_height).contains(pointer) =>
                    {
                        Action::ConfirmPetSelection { accept: false }
                    }
                    AdventurePhase::Hatch { updates, .. }
                        if updates > 170
                            && Rect::new(186.0, 445.0, 264.0, button_height).contains(pointer) =>
                    {
                        Action::Continue
                    }
                    AdventurePhase::Hatch { .. } => {
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
                    AdventurePhase::HelpScreen
                        if Rect::new(186.0, 310.0, 264.0, button_height).contains(pointer) =>
                    {
                        Action::Continue
                    }
                    AdventurePhase::Playing
                        if Rect::new(18.0, 3.0, 58.0, 60.0).contains(pointer) =>
                    {
                        Action::BuyGuppy
                    }
                    AdventurePhase::Playing
                        if session
                            .board
                            .as_ref()
                            .is_some_and(|board| board.potion_unlocked)
                            && Rect::new(217.0, 3.0, 58.0, 60.0).contains(pointer) =>
                    {
                        Action::BuyPotion
                    }
                    AdventurePhase::Playing
                        if session
                            .board
                            .as_ref()
                            .is_some_and(|board| board.upgrades.quality_unlocked)
                            && Rect::new(87.0, 3.0, 58.0, 60.0).contains(pointer) =>
                    {
                        Action::BuyFoodQuality
                    }
                    AdventurePhase::Playing
                        if session
                            .board
                            .as_ref()
                            .is_some_and(|board| board.upgrades.quantity_unlocked)
                            && Rect::new(144.0, 3.0, 58.0, 60.0).contains(pointer) =>
                    {
                        Action::BuyFoodQuantity
                    }
                    AdventurePhase::Playing
                        if session
                            .board
                            .as_ref()
                            .is_some_and(|board| board.oscar_unlocked)
                            && Rect::new(217.0, 3.0, 58.0, 60.0).contains(pointer) =>
                    {
                        Action::BuyOscar
                    }
                    AdventurePhase::Playing
                        if session
                            .board
                            .as_ref()
                            .is_some_and(|board| board.starcatcher_unlocked)
                            && Rect::new(290.0, 3.0, 58.0, 60.0).contains(pointer) =>
                    {
                        Action::BuyStarcatcher
                    }
                    AdventurePhase::Playing
                        if session
                            .board
                            .as_ref()
                            .is_some_and(|board| board.weapon_unlocked)
                            && Rect::new(363.0, 3.0, 58.0, 60.0).contains(pointer) =>
                    {
                        Action::BuyWeapon
                    }
                    AdventurePhase::Playing
                        if Rect::new(436.0, 3.0, 58.0, 60.0).contains(pointer) =>
                    {
                        Action::BuyEgg
                    }
                    _ => Action::Click {
                        x: pointer.x,
                        y: pointer.y,
                    },
                };
                if matches!(action, Action::Click { .. })
                    && matches!(session.phase, AdventurePhase::Playing)
                    && (31.0..=586.0).contains(&pointer.x)
                    && (61.0..=399.0).contains(&pointer.y)
                {
                    feed_press_at = Some(get_time());
                }
                if matches!(action, Action::Click { .. })
                    && pointer.y > 40.0
                    && session.board.as_ref().is_some_and(|board| {
                        board.weapon_strength == 12
                            && board
                                .invasion
                                .as_ref()
                                .is_some_and(|wave| wave.has_live_alien())
                    })
                {
                    held_fire_at = Some(get_time());
                }
                pending_actions.push(action);
            }
        }
        if is_mouse_button_released(MouseButton::Left) {
            hatch_pointer_owned = false;
            feed_press_at = None;
            held_feed_at = None;
            held_fire_at = None;
        }
        if matches!(session.phase, AdventurePhase::Hatch { .. }) {
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
                | AdventurePhase::InvasionTutorial { .. }
                | AdventurePhase::HelpScreen
                | AdventurePhase::Hatch { .. }
                | AdventurePhase::PetSelection { .. }
        ) || matches!(session.phase, AdventurePhase::GameOver { updates } if updates > 30)
            || matches!(session.phase, AdventurePhase::BonusResults { ref result } if result.updates >= 30);
        if !paused && is_key_pressed(KeyCode::Enter) && enter_continues {
            pending_actions.push(Action::Continue);
        }
        if !paused
            && matches!(
                session.phase,
                AdventurePhase::PetSelectionConfirmation { .. }
            )
        {
            if is_key_pressed(KeyCode::Y) || is_key_pressed(KeyCode::Enter) {
                pending_actions.push(Action::ConfirmPetSelection { accept: true });
            } else if is_key_pressed(KeyCode::N) {
                pending_actions.push(Action::ConfirmPetSelection { accept: false });
            }
        }
        let save_requested = is_key_pressed(KeyCode::S);
        let exit_requested = is_quit_requested()
            || (paused && is_key_pressed(KeyCode::Q))
            || options.quit_after.is_some_and(|limit| elapsed >= limit);
        let mut events = Vec::new();
        let mut phase_transitioned = false;
        if save_requested || exit_requested {
            // Inputs already accepted by this window belong to the checkpoint.
            // Apply them without inventing an extra simulation tick on save/exit.
            events.extend(session.apply_actions(&pending_actions));
            pending_actions.clear();
            if feed_press_at.is_some()
                && events
                    .iter()
                    .any(|event| matches!(event, Event::FoodDropped { .. }))
                && is_mouse_button_down(MouseButton::Left)
            {
                held_feed_at = feed_press_at.take();
            }
        }
        if !exit_requested {
            accumulator += f64::from(get_frame_time()).min(0.2);
            while accumulator >= f64::from(TICK_MS) / 1000.0 {
                if paused {
                    session.paused_step();
                } else {
                    let mut step_actions = std::mem::take(&mut pending_actions);
                    let pressed_feed = feed_press_at.is_some()
                        && step_actions
                            .iter()
                            .any(|action| matches!(action, Action::Click { .. }));
                    if let Some(press_at) = held_feed_at
                        && is_mouse_button_down(MouseButton::Left)
                        && matches!(session.phase, AdventurePhase::Playing)
                        && !session
                            .board
                            .as_ref()
                            .and_then(|board| board.invasion.as_ref())
                            .is_some_and(|wave| wave.has_live_alien())
                    {
                        step_actions.push(Action::HoldFeed {
                            x: pointer.x,
                            y: pointer.y,
                            elapsed_ms: ((get_time() - press_at) * 1000.0).max(0.0) as u32,
                        });
                    }
                    if let Some(press_at) = held_fire_at
                        && is_mouse_button_down(MouseButton::Left)
                        && matches!(session.phase, AdventurePhase::Playing)
                    {
                        step_actions.push(Action::HoldFire {
                            x: pointer.x,
                            y: pointer.y,
                            elapsed_ms: ((get_time() - press_at) * 1000.0).max(0.0) as u32,
                        });
                    }
                    let previous_phase = std::mem::discriminant(&session.phase);
                    let step_events = session.step(&step_actions);
                    phase_transitioned |= previous_phase != std::mem::discriminant(&session.phase);
                    if feed_press_at.is_some()
                        && step_events
                            .iter()
                            .any(|event| matches!(event, Event::FoodDropped { .. }))
                        && is_mouse_button_down(MouseButton::Left)
                    {
                        held_feed_at = feed_press_at.take();
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
                    }) || !matches!(session.phase, AdventurePhase::Playing)
                    {
                        feed_press_at = None;
                        held_feed_at = None;
                        held_fire_at = None;
                    }
                    events.extend(step_events);
                }
                accumulator -= f64::from(TICK_MS) / 1000.0;
                if phase_transitioned
                    || events
                        .iter()
                        .any(|event| matches!(event, Event::HatchStarted { .. }))
                {
                    accumulator = 0.0;
                    break;
                }
            }
        }
        if phase_transitioned
            || save_requested
            || exit_requested
            || events.iter().any(|event| {
                matches!(
                    event,
                    Event::FirstTankRescueStarted { .. }
                        | Event::GameOverStarted { .. }
                        | Event::GameSelectorOpened { .. }
                        | Event::Invasion {
                            event: InvasionEvent::ModalOpened(_),
                            ..
                        }
                        | Event::HatchStarted { .. }
                        | Event::StageStarted { .. }
                        | Event::RescueGuppyGranted { .. }
                        | Event::BonusResultsCommitted { .. }
                        | Event::Bonus {
                            event: crate::bonus::BonusEvent::Started { .. }
                                | crate::bonus::BonusEvent::Claimed { .. }
                                | crate::bonus::BonusEvent::Credited { .. },
                            ..
                        }
                )
            })
        {
            cli::save_session(&options, &session)?;
        }
        presentation.play(&events);
        if let Some(evidence) = evidence.as_mut() {
            evidence.record(&events, &session, elapsed, paused)?;
        }
        if exit_requested {
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
        presentation.draw(&session, paused, pointer);
        set_default_camera();
        if let Some(evidence) = evidence.as_ref() {
            let request = evidence.root.join("capture.request");
            if is_key_pressed(KeyCode::F12) || request.exists() {
                evidence.capture(session.ticks);
                if request.exists() {
                    fs::remove_file(request)?;
                }
            }
        }
        next_frame().await;
    }
    cli::save_session(&options, &session)?;
    let after = install::identify(&game_root)?;
    if after.inventory_sha256 != identity.inventory_sha256 {
        return Err("Owned installation changed during run; inspect before continuing".into());
    }
    if let Some(evidence) = evidence.as_mut() {
        evidence.record(&[], &session, get_time() - started, paused)?;
        cli::write_json(
            &evidence.root.join("final.local.json"),
            &serde_json::json!({"session":session,"game_after":after,"elapsed_seconds":get_time()-started,"game_unchanged":true}),
        )?;
    }
    Ok(())
}
