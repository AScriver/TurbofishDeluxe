use std::error::Error;
use turbofish_deluxe::{
    app,
    assets::GameAssets,
    cli::{self, Options},
    install,
    screensaver_bank::{NormalWriterLease, ScreensaverSceneLease},
};
use turbofish_screenhost::ScreenCommand;

fn prepare() -> Result<(), Box<dyn Error>> {
    let Some(mut options) = Options::parse()? else {
        return Ok(());
    };
    let game_root = install::discover(options.game_dir.as_deref())?;
    cli::validate_destinations(&mut options, &game_root)?;
    let identity = install::identify(&game_root)?;
    println!(
        "Owned game: {} (SHA-256 {}, Steam build {:?})",
        game_root.display(),
        identity.executable_sha256,
        identity.steam_build
    );
    let assets = GameAssets::open(&game_root)?;
    if options.inspect_assets {
        let mut images = 0;
        let mut failures = Vec::new();
        let mut image_ids: Vec<_> = assets.image_ids().collect();
        image_ids.sort();
        for id in image_ids {
            match assets.load_image(id) {
                Ok(_) => images += 1,
                Err(error) => failures.push(format!("{id}: {error}")),
            }
        }
        let mut sounds = 0;
        for id in assets.sound_ids() {
            match assets.load_sound(id) {
                Ok(_) => sounds += 1,
                Err(error) => failures.push(format!("{id}: {error}")),
            }
        }
        let mut fonts = 0;
        for entry in std::fs::read_dir(game_root.join("data"))? {
            let path = entry?.path();
            if path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("txt"))
            {
                let name = path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .ok_or("Non-UTF8 font name")?;
                let result =
                    turbofish_deluxe::font::BitmapFont::load(&game_root, name).and_then(|font| {
                        for layer in &font.layers {
                            let image = assets
                                .load_image(&layer.image_path.to_string_lossy())
                                .map_err(|error| {
                                    turbofish_deluxe::font::FontError::Invalid(error.to_string())
                                })?;
                            layer.validate_atlas(image.width, image.height)?;
                        }
                        Ok(())
                    });
                match result {
                    Ok(()) => fonts += 1,
                    Err(error) => failures.push(format!("font {name}: {error}")),
                }
            }
        }
        if let Some(directory) = &options.evidence_dir {
            std::fs::create_dir_all(directory)?;
            cli::write_json(
                &directory.join("asset-check.local.json"),
                &serde_json::json!({"identity":identity,"images":images,"sounds":sounds,"fonts":fonts,"failures":failures}),
            )?;
        }
        println!(
            "Decoded {images} manifest images, {sounds} sound effects and {fonts} bitmap fonts; no window or writes to game installation."
        );
        for failure in &failures {
            eprintln!("{failure}");
        }
        if !failures.is_empty() {
            return Err(format!("{} asset checks failed", failures.len()).into());
        }
        return Ok(());
    }
    let screensaver = options.screensaver;
    let profiles_path = options.save_dir.join("profiles.json");
    if matches!(screensaver, Some(ScreenCommand::Fullscreen)) && !profiles_path.is_file() {
        return Err("Screensaver income requires an existing current project profile".into());
    }
    let normal_writer_lease = if screensaver.is_none() {
        Some(NormalWriterLease::try_acquire(&options.save_dir)?)
    } else {
        None
    };
    let saver_scene_lease = if matches!(screensaver, Some(ScreenCommand::Fullscreen)) {
        Some(
            ScreensaverSceneLease::try_acquire(&options.save_dir)?
                .ok_or("Another screensaver is already evolving this project scene")?,
        )
    } else {
        None
    };
    let mut profiles = cli::load_profiles(&options)?;
    if screensaver.is_none() {
        if !profiles_path.exists() || options.new_game {
            // Establish profile generation before another saver can read it.
            cli::save_profiles_checkpoint(&options, &profiles)?;
        }
        let credited = profiles.claim_screensaver(&options.save_dir)?;
        if credited > 0 {
            println!("Claimed {credited} screensaver shells");
        }
        if profiles.adopt_screensaver_scene(&options.save_dir)? {
            println!("Resumed screensaver Virtual Tank continuation");
        }
    }
    let settings = turbofish_deluxe::settings::SettingsSave::load_or_default(
        &options.save_dir.join("settings.json"),
    )?;
    #[cfg(not(windows))]
    if screensaver.is_some() {
        return Err(turbofish_screenhost::HostError::UnsupportedPlatform.into());
    }
    if let Some(command) = screensaver {
        match command {
            ScreenCommand::Preview(parent) | ScreenCommand::Configure(Some(parent)) => {
                parent.validate()?;
            }
            ScreenCommand::Fullscreen | ScreenCommand::Configure(None) => {}
        }
    }
    let mut window = macroquad::miniquad::conf::Conf {
        window_title: "Turbofish Deluxe".into(),
        window_width: 960,
        window_height: 720,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    };
    if let Some(command) = screensaver {
        window.window_resizable = false;
        match command {
            ScreenCommand::Fullscreen => {
                window.window_title = "Turbofish Deluxe Screensaver".into();
                window.fullscreen = true;
            }
            ScreenCommand::Preview(_) => {
                window.window_title = "Turbofish Deluxe Preview".into();
                window.window_width = 320;
                window.window_height = 240;
            }
            ScreenCommand::Configure(_) => {
                window.window_title = "Turbofish Deluxe Screensaver Settings".into();
            }
        }
    }
    macroquad::Window::from_config(window, async move {
        let writer_lease = normal_writer_lease;
        let scene_lease = saver_scene_lease;
        let result = if let Some(command) = screensaver {
            app::run_screensaver(
                options, game_root, identity, assets, profiles, settings, command,
            )
            .await
        } else {
            app::run(options, game_root, identity, assets, profiles, settings).await
        };
        drop(writer_lease);
        drop(scene_lease);
        if let Err(error) = result {
            eprintln!("Runtime failed: {error}");
            std::process::exit(1);
        }
    });
    Ok(())
}

fn main() {
    if let Err(error) = prepare() {
        eprintln!("Turbofish Deluxe: {error}");
        std::process::exit(1);
    }
}
