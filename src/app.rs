use crate::{
    adventure::{AdventurePhase, AdventureSession},
    assets::{GameAssets, SoundData},
    cli::{self, Options},
    font::BitmapFont,
    install::{self, InstallIdentity},
    sim::{Action, AdventureState, CoinKind, Event, FishPose, FishSize, TICK_MS},
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
];
const SOUND_IDS: &[&str] = &[
    "SOUND_DROPFOOD",
    "SOUND_SLURP",
    "SOUND_GROW",
    "SOUND_POINTS",
    "SOUND_BUY",
    "SOUND_BUTTONCLICK",
    "SOUND_HATCH",
];

pub struct Presentation {
    images: HashMap<String, Texture2D>,
    sounds: HashMap<String, Sound>,
    fonts: HashMap<String, RenderedFont>,
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
                Event::FoodDropped { .. } => "SOUND_DROPFOOD",
                Event::FoodEaten { .. } => "SOUND_SLURP",
                Event::FishGrew { .. } => "SOUND_GROW",
                Event::CoinCredited { .. } => "SOUND_POINTS",
                Event::GuppyBought { .. } | Event::EggBought { .. } => "SOUND_BUY",
                Event::HatchOpened { .. } => "SOUND_HATCH",
                Event::StageStarted { .. } | Event::RescueGuppyGranted { .. } => {
                    "SOUND_BUTTONCLICK"
                }
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
        self.sprite("IMAGE_AQUARIUM1", 0.0, 0.0, None, false, 1.0, 1.0);
        for food in &state.food {
            self.sprite(
                "IMAGE_FOOD",
                food.x - 5.0,
                food.y - 4.0,
                Some(Rect::new(
                    (food.frame / 3 % 10) as f32 * 40.0,
                    0.0,
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
                FishSize::Large => 2.0,
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
                1.0,
            );
        }
        for fish in &state.dead_fish {
            let row = match fish.size {
                FishSize::Small => 0.0,
                FishSize::Medium => 1.0,
                FishSize::Large => 2.0,
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
        for coin in &state.coins {
            let row = if coin.kind == CoinKind::Silver {
                0.0
            } else {
                1.0
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

    fn draw_hatch(&self, updates: u32) {
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
            self.sprite(
                "IMAGE_STINKY",
                278.0,
                100.0,
                Some(Rect::new((updates % 20 / 2) as f32 * 80.0, 0.0, 80.0, 80.0)),
                false,
                1.0,
                1.0,
            );
            self.centered_text(
                "JungleFever15outline",
                "STINKY the Snail",
                260.0,
                Color::from_rgba(255, 200, 0, 255),
            );
        }
        self.sprite("IMAGE_HATCHREFLECTION", 240.0, 60.0, None, false, 1.0, 1.0);
        if updates > 170 {
            self.centered_text(
                "JungleFever10outline",
                "STINKY roams around the",
                300.0,
                WHITE,
            );
            self.centered_text(
                "JungleFever10outline",
                "bottom of your tank, catching",
                320.0,
                WHITE,
            );
            self.centered_text(
                "JungleFever10outline",
                "any coins you may have missed.",
                340.0,
                WHITE,
            );
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

    fn draw(&self, session: &AdventureSession, paused: bool) {
        match session.phase {
            AdventurePhase::Hatch { updates, .. } => self.draw_hatch(updates),
            AdventurePhase::Playing | AdventurePhase::FirstTankRescue => {
                if let Some(board) = &session.board {
                    self.draw_board(board);
                }
            }
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
        }
        let button_height = presentation.images["IMAGE_MAINBUTTON"].height();
        let menu_rect = if matches!(session.phase, AdventurePhase::Hatch { .. }) {
            Rect::new(525.0, 4.0, 80.0, button_height)
        } else {
            Rect::new(525.0, 3.0, 101.0, 29.0)
        };
        if is_mouse_button_pressed(MouseButton::Left) {
            if menu_rect.contains(pointer) {
                paused = !paused;
                accumulator = 0.0;
            } else if !paused {
                let action = match session.phase {
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
                    AdventurePhase::Playing
                        if Rect::new(18.0, 3.0, 58.0, 60.0).contains(pointer) =>
                    {
                        Action::BuyGuppy
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
                pending_actions.push(action);
            }
        }
        if is_mouse_button_released(MouseButton::Left) {
            hatch_pointer_owned = false;
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
        if !paused
            && is_key_pressed(KeyCode::Enter)
            && matches!(
                session.phase,
                AdventurePhase::FirstTankRescue | AdventurePhase::Hatch { .. }
            )
        {
            pending_actions.push(Action::Continue);
        }
        let save_requested = is_key_pressed(KeyCode::S);
        let exit_requested = is_quit_requested()
            || (paused && is_key_pressed(KeyCode::Q))
            || options.quit_after.is_some_and(|limit| elapsed >= limit);
        let mut events = Vec::new();
        if save_requested || exit_requested {
            // Inputs already accepted by this window belong to the checkpoint.
            // Apply them without inventing an extra simulation tick on save/exit.
            events.extend(session.apply_actions(&pending_actions));
            pending_actions.clear();
        }
        if !paused && !exit_requested {
            accumulator += f64::from(get_frame_time()).min(0.2);
            while accumulator >= f64::from(TICK_MS) / 1000.0 {
                events.extend(session.step(&pending_actions));
                pending_actions.clear();
                accumulator -= f64::from(TICK_MS) / 1000.0;
                if events
                    .iter()
                    .any(|event| matches!(event, Event::HatchStarted { .. }))
                {
                    accumulator = 0.0;
                    break;
                }
            }
        }
        if save_requested
            || exit_requested
            || events.iter().any(|event| {
                matches!(
                    event,
                    Event::FirstTankRescueStarted { .. }
                        | Event::HatchStarted { .. }
                        | Event::StageStarted { .. }
                        | Event::RescueGuppyGranted { .. }
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
        presentation.draw(&session, paused);
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
