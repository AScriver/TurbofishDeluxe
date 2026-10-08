use std::error::Error;
use turbofish_deluxe::{
    app,
    assets::GameAssets,
    cli::{self, Options},
    install,
};

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
    let state = cli::load_state(&options)?;
    macroquad::Window::from_config(
        macroquad::miniquad::conf::Conf {
            window_title: "Turbofish Deluxe".into(),
            window_width: 960,
            window_height: 720,
            window_resizable: true,
            high_dpi: true,
            ..Default::default()
        },
        async move {
            if let Err(error) = app::run(options, game_root, identity, assets, state).await {
                eprintln!("Runtime failed: {error}");
                std::process::exit(1);
            }
        },
    );
    Ok(())
}

fn main() {
    if let Err(error) = prepare() {
        eprintln!("Turbofish Deluxe: {error}");
        std::process::exit(1);
    }
}
