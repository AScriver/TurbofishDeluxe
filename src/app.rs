use crate::{
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
];
const SOUND_IDS: &[&str] = &[
    "SOUND_DROPFOOD",
    "SOUND_SLURP",
    "SOUND_GROW",
    "SOUND_POINTS",
    "SOUND_BUY",
    "SOUND_BUTTONCLICK",
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

    fn draw(&self, state: &AdventureState, paused: bool) {
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
            self.fonts["Pix118"].text("150", 458.0, 58.0, Color::from_rgba(110, 250, 110, 255));
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
        if state.victory {
            draw_rectangle(
                100.0,
                145.0,
                440.0,
                170.0,
                Color::new(0.02, 0.1, 0.15, 0.96),
            );
            draw_text("Tank 1-1 complete", 192.0, 192.0, 28.0, WHITE);
            draw_text("Three egg pieces purchased", 168.0, 234.0, 23.0, YELLOW);
            draw_text(
                "Progress saved — Esc opens the menu",
                154.0,
                274.0,
                21.0,
                WHITE,
            );
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
}

impl Evidence {
    fn open(
        root: &Path,
        identity: &InstallIdentity,
        state: &AdventureState,
        seed: u64,
    ) -> Result<Self, Box<dyn Error>> {
        fs::create_dir_all(root)?;
        cli::write_json(
            &root.join("identity.local.json"),
            &serde_json::json!({"game": identity, "rust_executable_sha256": install::sha256(&fs::read(std::env::current_exe()?)?), "seed": seed, "tick_ms": TICK_MS, "retail_rng_equivalent": false, "start": state}),
        )?;
        Ok(Self {
            root: root.into(),
            events: BufWriter::new(File::create(root.join("events.local.jsonl"))?),
            last_state_tick: u64::MAX,
            snapshot_deferred: false,
        })
    }
    fn record(
        &mut self,
        events: &[Event],
        state: &AdventureState,
        elapsed: f64,
        paused: bool,
    ) -> Result<(), Box<dyn Error>> {
        for event in events {
            serde_json::to_writer(
                &mut self.events,
                &serde_json::json!({"elapsed_seconds": elapsed,"event":event}),
            )?;
            writeln!(&mut self.events)?;
        }
        self.events.flush()?;
        if self.last_state_tick != state.tick {
            let publication = cli::write_json(
                &self.root.join("state.local.json"),
                &serde_json::json!({"elapsed_seconds":elapsed, "paused":paused, "state":state}),
            );
            match publication {
                Ok(()) => {
                    if self.snapshot_deferred {
                        serde_json::to_writer(
                            &mut self.events,
                            &serde_json::json!({"diagnostic":"snapshot_publication_recovered","tick":state.tick,"elapsed_seconds":elapsed}),
                        )?;
                        writeln!(&mut self.events)?;
                        self.events.flush()?;
                    }
                    self.last_state_tick = state.tick;
                    self.snapshot_deferred = false;
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
                            &serde_json::json!({"diagnostic":"snapshot_publication_deferred","tick":state.tick,"elapsed_seconds":elapsed,"error":error.to_string()}),
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
    mut state: AdventureState,
) -> Result<(), Box<dyn Error>> {
    let presentation = Presentation::load(&game_root, &assets, options.muted).await?;
    let mut evidence = options
        .evidence_dir
        .as_ref()
        .map(|root| Evidence::open(root, &identity, &state, options.seed))
        .transpose()?;
    let started = get_time();
    let mut accumulator = 0.0_f64;
    let mut pending_actions = Vec::new();
    let mut paused = false;
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
        if is_mouse_button_pressed(MouseButton::Left) {
            if pointer.x >= 525.0 && pointer.x < 626.0 && pointer.y >= 3.0 && pointer.y < 32.0 {
                paused = !paused;
                accumulator = 0.0;
            } else if !paused {
                let action = if pointer.x >= 18.0
                    && pointer.x < 76.0
                    && pointer.y >= 3.0
                    && pointer.y < 63.0
                {
                    Action::BuyGuppy
                } else if pointer.x >= 436.0
                    && pointer.x < 494.0
                    && pointer.y >= 3.0
                    && pointer.y < 63.0
                {
                    Action::BuyEgg
                } else {
                    Action::Click {
                        x: pointer.x,
                        y: pointer.y,
                    }
                };
                pending_actions.push(action);
            }
        }
        if is_key_pressed(KeyCode::S) {
            cli::save_state(&options, &state)?;
        }
        if (paused && is_key_pressed(KeyCode::Q))
            || options.quit_after.is_some_and(|limit| elapsed >= limit)
        {
            break;
        }
        let mut events = Vec::new();
        if !paused {
            accumulator += f64::from(get_frame_time()).min(0.2);
            while accumulator >= f64::from(TICK_MS) / 1000.0 {
                events.extend(state.step(&pending_actions));
                pending_actions.clear();
                accumulator -= f64::from(TICK_MS) / 1000.0;
                if state.victory {
                    accumulator = 0.0;
                    break;
                }
            }
        }
        presentation.play(&events);
        if let Some(evidence) = evidence.as_mut() {
            evidence.record(&events, &state, elapsed, paused)?;
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
        presentation.draw(&state, paused);
        set_default_camera();
        if let Some(evidence) = evidence.as_ref() {
            let request = evidence.root.join("capture.request");
            if is_key_pressed(KeyCode::F12) || request.exists() {
                evidence.capture(state.tick);
                if request.exists() {
                    fs::remove_file(request)?;
                }
            }
        }
        next_frame().await;
    }
    cli::save_state(&options, &state)?;
    let after = install::identify(&game_root)?;
    if after.inventory_sha256 != identity.inventory_sha256 {
        return Err("Owned installation changed during run; inspect before continuing".into());
    }
    if let Some(evidence) = evidence.as_mut() {
        evidence.record(&[], &state, get_time() - started, paused)?;
        cli::write_json(
            &evidence.root.join("final.local.json"),
            &serde_json::json!({"state":state,"game_after":after,"elapsed_seconds":get_time()-started,"game_unchanged":true}),
        )?;
    }
    Ok(())
}
