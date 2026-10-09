use crate::{adventure::AdventureSession, install, sim::AdventureState};
use std::{
    env,
    error::Error,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug)]
pub struct Options {
    pub game_dir: Option<PathBuf>,
    pub save_dir: PathBuf,
    pub evidence_dir: Option<PathBuf>,
    pub seed: u64,
    pub new_game: bool,
    pub inspect_assets: bool,
    pub quit_after: Option<f64>,
    pub muted: bool,
}

impl Options {
    pub fn parse() -> Result<Option<Self>, Box<dyn Error>> {
        let mut options = Self {
            game_dir: None,
            save_dir: env::var_os("LOCALAPPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(env::temp_dir)
                .join("TurbofishDeluxe"),
            evidence_dir: None,
            seed: SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos() as u64,
            new_game: false,
            inspect_assets: false,
            quit_after: None,
            muted: false,
        };
        let mut arguments = env::args().skip(1);
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--help" | "-h" => {
                    println!(
                        "Turbofish Deluxe\n\nReads your owned Insaniquarium Deluxe installation.\n\n--game-dir <directory>  Override Steam discovery\n--save-dir <directory>  Separate project saves (default: LOCALAPPDATA/TurbofishDeluxe)\n--new-game              Start a fresh project first tank\n--seed <integer>        Reproducible Rust PRNG (not retail replay parity)\n--evidence-dir <dir>    Write runtime events, state, identity and requested captures\n--inspect-assets       Decode all manifest images/effects without opening a window\n--quit-after <seconds>  Bound a normal-speed runtime check\n--mute                 Disable effect playback\n\nEscape pauses; click Menu or press S to save. F12 captures when evidence is enabled."
                    );
                    return Ok(None);
                }
                "--game-dir" => {
                    options.game_dir =
                        Some(arguments.next().ok_or("--game-dir requires a path")?.into())
                }
                "--save-dir" => {
                    options.save_dir = arguments.next().ok_or("--save-dir requires a path")?.into()
                }
                "--evidence-dir" => {
                    options.evidence_dir = Some(
                        arguments
                            .next()
                            .ok_or("--evidence-dir requires a path")?
                            .into(),
                    )
                }
                "--seed" => {
                    options.seed = arguments
                        .next()
                        .ok_or("--seed requires an integer")?
                        .parse()?
                }
                "--quit-after" => {
                    let seconds: f64 = arguments
                        .next()
                        .ok_or("--quit-after requires seconds")?
                        .parse()?;
                    if !seconds.is_finite() || seconds <= 0.0 {
                        return Err("--quit-after must be positive and finite".into());
                    }
                    options.quit_after = Some(seconds);
                }
                "--new-game" => options.new_game = true,
                "--inspect-assets" => options.inspect_assets = true,
                "--mute" => options.muted = true,
                _ => return Err(format!("Unknown argument {argument}; use --help").into()),
            }
        }
        Ok(Some(options))
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ProjectSave {
    pub format_version: u32,
    pub session: AdventureSession,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LegacyProjectSave {
    pub format_version: u32,
    pub state: AdventureState,
}

pub const SAVE_FORMAT_VERSION: u32 = 16;

pub fn decode_save(bytes: &[u8]) -> Result<AdventureSession, Box<dyn Error>> {
    Ok(decode_save_with_migration(bytes)?.0)
}

fn decode_save_with_migration(bytes: &[u8]) -> Result<(AdventureSession, bool), Box<dyn Error>> {
    let value: serde_json::Value = serde_json::from_slice(bytes)?;
    let version_number = value
        .get("format_version")
        .and_then(serde_json::Value::as_u64);
    if let Some(version @ 1..=6) = version_number {
        validate_legacy_boundary(&value, version)?;
    }
    let (session, migrated) = match value.get("format_version").and_then(|value| value.as_u64()) {
        Some(1) => {
            let legacy: LegacyProjectSave = serde_json::from_value(value)?;
            (
                AdventureSession::from_legacy_board(legacy.state)
                    .map_err(|failure| format!("Cannot migrate project save: {failure:?}"))?,
                true,
            )
        }
        Some(version @ (2 | 3)) => {
            let missing_pet_state = value
                .pointer("/session/board")
                .and_then(serde_json::Value::as_object)
                .is_some_and(|board| !board.contains_key("stinky"));
            if version == 3
                && (!value
                    .pointer("/session/progress")
                    .and_then(serde_json::Value::as_object)
                    .is_some_and(|progress| progress.contains_key("first_stage_best_seconds"))
                    || missing_pet_state)
            {
                return Err(
                    "Incomplete format-three save; required state fields are missing".into(),
                );
            }
            let missing_invasion = value
                .pointer("/session/board")
                .and_then(serde_json::Value::as_object)
                .is_some_and(|board| !board.contains_key("invasion"));
            let mut session = serde_json::from_value::<ProjectSave>(value)?.session;
            if session.progress.level > 2 {
                return Err(
                    "Legacy project save contains a stage its format never supported".into(),
                );
            }
            if version == 2
                && missing_pet_state
                && let Some(board) = &mut session.board
            {
                board.initialize_missing_stinky();
            }
            if missing_invasion && let Some(board) = &mut session.board {
                if board.level == 2 && board.eggs != 0 {
                    return Err("Legacy second-stage save contains unsupported purchases".into());
                }
                board.initialize_legacy_invasion();
            }
            (session, true)
        }
        Some(4) => {
            let has_score_field = value
                .pointer("/session/progress")
                .and_then(serde_json::Value::as_object)
                .is_some_and(|progress| {
                    ["first_stage_best_seconds", "later_stage_best_seconds"]
                        .iter()
                        .all(|field| progress.contains_key(*field))
                });
            let missing_board_field = value
                .pointer("/session/board")
                .and_then(serde_json::Value::as_object)
                .is_some_and(|board| {
                    ["stinky", "upgrades", "invasion", "niko", "pearls"]
                        .iter()
                        .any(|field| !board.contains_key(*field))
                        || board
                            .get("food")
                            .and_then(serde_json::Value::as_array)
                            .is_some_and(|food| {
                                food.iter().any(|pellet| pellet.get("quality").is_none())
                            })
                        || board
                            .get("fish")
                            .and_then(serde_json::Value::as_array)
                            .is_some_and(|fish| {
                                fish.iter()
                                    .any(|entity| entity.get("cannot_be_eaten_ticks").is_none())
                            })
                });
            if !has_score_field || missing_board_field {
                return Err(
                    "Incomplete format-four save; required state fields are missing".into(),
                );
            }
            let mut session = serde_json::from_value::<ProjectSave>(value)?.session;
            if session.progress.level > 3 {
                return Err("Format-four save contains unsupported Adventure progress".into());
            }
            if let Some(board) = &mut session.board {
                board.initialize_legacy_stage13_support();
            }
            (session, true)
        }
        Some(version @ 5..=16) => {
            let complete_progress = value
                .pointer("/session/progress")
                .and_then(serde_json::Value::as_object)
                .is_some_and(|progress| {
                    ["first_stage_best_seconds", "later_stage_best_seconds"]
                        .iter()
                        .all(|field| progress.contains_key(*field))
                        && (version == 5
                            || ["pet_capacity", "selected_pets"]
                                .iter()
                                .all(|field| progress.contains_key(*field)))
                        && (version < 7 || progress.contains_key("shell_balance"))
                });
            let incomplete_board = value
                .pointer("/session/board")
                .and_then(serde_json::Value::as_object)
                .is_some_and(|board| {
                    [
                        "stinky",
                        "upgrades",
                        "invasion",
                        "niko",
                        "pearls",
                        "oscars",
                        "dead_oscars",
                        "oscar_unlocked",
                        "weapon_strength",
                        "weapon_unlocked",
                    ]
                    .iter()
                    .any(|field| !board.contains_key(*field))
                        || (version >= 6
                            && ["fish_pets", "punch_sound_cooldown"]
                                .iter()
                                .any(|field| !board.contains_key(*field)))
                        || (version >= 7
                            && ["potion_unlocked", "potion_armed"]
                                .iter()
                                .any(|field| !board.contains_key(*field)))
                        || (version >= 8
                            && [
                                "starcatcher_unlocked",
                                "starcatchers",
                                "dead_starcatchers",
                                "clyde",
                            ]
                            .iter()
                            .any(|field| !board.contains_key(*field)))
                        || (version >= 10
                            && ["missiles", "rufus"]
                                .iter()
                                .any(|field| !board.contains_key(*field)))
                        || (version >= 11 && !board.contains_key("notes"))
                        || (version >= 14 && !board.contains_key("bomb_shots"))
                        || (version >= 16
                            && ["breeder_unlocked", "breeders", "dead_breeders", "rhubarb"]
                                .iter()
                                .any(|field| !board.contains_key(*field)))
                        || (version >= 15
                            && board
                                .get("missiles")
                                .and_then(serde_json::Value::as_array)
                                .is_some_and(|missiles| {
                                    missiles.iter().any(|missile| {
                                        ["kind", "reflected"]
                                            .iter()
                                            .any(|field| missile.get(*field).is_none())
                                    })
                                }))
                        || (version >= 13
                            && ["gekko_unlocked", "gekkos", "dead_gekkos"]
                                .iter()
                                .any(|field| !board.contains_key(*field)))
                        || (version >= 13
                            && board
                                .get("coins")
                                .and_then(serde_json::Value::as_array)
                                .is_some_and(|coins| {
                                    coins
                                        .iter()
                                        .any(|coin| coin.get("animation_ticks").is_none())
                                }))
                        || (version >= 14
                            && board
                                .get("coins")
                                .and_then(serde_json::Value::as_array)
                                .is_some_and(|coins| {
                                    coins
                                        .iter()
                                        .any(|coin| coin.get("hazard_age_ticks").is_none())
                                }))
                        || (version >= 12
                            && ["grubber_unlocked", "grubbers", "dead_grubbers", "larvae"]
                                .iter()
                                .any(|field| !board.contains_key(*field)))
                        || (version >= 9
                            && board
                                .get("fish_pets")
                                .and_then(serde_json::Value::as_array)
                                .is_some_and(|pets| {
                                    pets.iter().any(|pet| {
                                        pet.get("coin_timer").is_none()
                                            || (version >= 14
                                                && ["bomb_threshold", "glint_phase"]
                                                    .iter()
                                                    .any(|field| pet.get(*field).is_none()))
                                            || (version >= 11 && pet.get("meryl_blink").is_none())
                                            || (version >= 12
                                                && [
                                                    "ward_active",
                                                    "ward_timer",
                                                    "published_x",
                                                    "published_y",
                                                ]
                                                .iter()
                                                .any(|field| pet.get(*field).is_none()))
                                    })
                                }))
                        || board
                            .get("food")
                            .and_then(serde_json::Value::as_array)
                            .is_some_and(|food| {
                                food.iter().any(|pellet| {
                                    pellet.get("quality").is_none()
                                        || (version >= 7
                                            && [
                                                "direction",
                                                "vx",
                                                "vy",
                                                "animation_period",
                                                "free_from_zorf",
                                            ]
                                            .iter()
                                            .any(|field| pellet.get(*field).is_none()))
                                })
                            })
                        || board
                            .get("fish")
                            .and_then(serde_json::Value::as_array)
                            .is_some_and(|fish| {
                                fish.iter()
                                    .any(|entity| entity.get("cannot_be_eaten_ticks").is_none())
                            })
                        || board
                            .get("invasion")
                            .and_then(serde_json::Value::as_object)
                            .is_some_and(|wave| {
                                (version < 11
                                    && !wave.contains_key("kind")
                                    && !wave.contains_key("plan"))
                                    || (version >= 11
                                        && [
                                            "plan",
                                            "actors",
                                            "warps",
                                            "dead_aliens",
                                            "battle_active",
                                        ]
                                        .iter()
                                        .any(|field| !wave.contains_key(*field)))
                                    || (version >= 9 && !wave.contains_key("gus_warning_shown"))
                                    || (version >= 11
                                        && wave
                                            .get("actors")
                                            .and_then(serde_json::Value::as_array)
                                            .is_some_and(|actors| {
                                                actors
                                                    .iter()
                                                    .any(|actor| actor.get("kind").is_none())
                                            }))
                                    || (version >= 14
                                        && wave
                                            .get("actors")
                                            .and_then(serde_json::Value::as_array)
                                            .is_some_and(|actors| {
                                                actors.iter().any(|actor| {
                                                    [
                                                        "phase_ticks",
                                                        "phase_threshold",
                                                        "healing",
                                                        "ever_healed",
                                                        "movement_divisor",
                                                    ]
                                                    .iter()
                                                    .any(|field| actor.get(*field).is_none())
                                                })
                                            }))
                                    || (version >= 11
                                        && wave
                                            .get("dead_aliens")
                                            .and_then(serde_json::Value::as_array)
                                            .is_some_and(|bodies| {
                                                bodies.iter().any(|body| body.get("kind").is_none())
                                            }))
                                    || wave
                                        .get("alien")
                                        .and_then(serde_json::Value::as_object)
                                        .is_some_and(|alien| !alien.contains_key("kind"))
                                    || (version >= 6
                                        && wave
                                            .get("dead_alien")
                                            .and_then(serde_json::Value::as_object)
                                            .is_some_and(|body| !body.contains_key("kind")))
                            })
                        || (version >= 7
                            && board
                                .get("niko")
                                .and_then(serde_json::Value::as_object)
                                .is_some_and(|niko| {
                                    ["anchor_x", "anchor_y"]
                                        .iter()
                                        .any(|field| !niko.contains_key(*field))
                                }))
                });
            if !complete_progress || incomplete_board {
                let label = match version {
                    5 => "five",
                    6 => "six",
                    7 => "seven",
                    8 => "eight",
                    9 => "nine",
                    10 => "ten",
                    11 => "eleven",
                    12 => "twelve",
                    13 => "thirteen",
                    14 => "fourteen",
                    15 => "fifteen",
                    _ => "sixteen",
                };
                return Err(format!(
                    "Incomplete format-{label} save; required state fields are missing"
                )
                .into());
            }
            let mut session = serde_json::from_value::<ProjectSave>(value)?.session;
            if version == 5 {
                if session.progress.level > 4 {
                    return Err("Format-five save contains unsupported Adventure progress".into());
                }
                if let Some(board) = &mut session.board {
                    board.initialize_legacy_alien_body_kind();
                    board.initialize_legacy_stage14_support();
                }
            }
            (session, version < 7)
        }
        _ => {
            return Err("Unsupported project save version; original saves are not imported".into());
        }
    };
    session.validate()?;
    Ok((session, migrated))
}

fn validate_legacy_boundary(value: &serde_json::Value, version: u64) -> Result<(), Box<dyn Error>> {
    let max_level = match version {
        1 => 1,
        2 | 3 => 2,
        4 => 3,
        5 => 4,
        _ => 5,
    };
    if let Some(progress) = value.pointer("/session/progress")
        && (progress.get("tank").and_then(serde_json::Value::as_u64) != Some(1)
            || progress
                .get("level")
                .and_then(serde_json::Value::as_u64)
                .is_none_or(|level| !(1..=max_level).contains(&level))
            || progress
                .get("shell_balance")
                .and_then(serde_json::Value::as_u64)
                .is_some_and(|balance| balance != 0)
            || progress
                .get("unlocked_pets")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|pets| pets.iter().any(|pet| pet.as_str() == Some("Zorf"))))
    {
        return Err("Legacy save claims progress or shells its format never supported".into());
    }
    if value
        .pointer("/session/phase")
        .and_then(serde_json::Value::as_object)
        .is_some_and(|phase| phase.contains_key("Bonus") || phase.contains_key("BonusResults"))
    {
        return Err("Legacy save claims an unsupported bonus phase".into());
    }
    let board = value
        .pointer("/session/board")
        .or_else(|| value.get("state"));
    if let Some(board) = board.filter(|board| !board.is_null())
        && (["potion_unlocked", "potion_armed"]
            .iter()
            .any(|field| board.get(*field).and_then(serde_json::Value::as_bool) == Some(true))
            || board
                .get("pets")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|pets| pets.iter().any(|pet| pet.as_str() == Some("Zorf")))
            || board
                .get("fish")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|fish| {
                    fish.iter().any(|fish| {
                        matches!(
                            fish.get("size").and_then(serde_json::Value::as_str),
                            Some("Star" | "Crowned")
                        )
                    })
                })
            || board
                .get("food")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|food| {
                    food.iter().any(|pellet| {
                        pellet
                            .get("quality")
                            .and_then(serde_json::Value::as_u64)
                            .is_some_and(|quality| quality > 2)
                            || pellet
                                .get("direction")
                                .and_then(serde_json::Value::as_u64)
                                .is_some_and(|direction| direction != 0)
                            || pellet
                                .get("free_from_zorf")
                                .and_then(serde_json::Value::as_bool)
                                == Some(true)
                    })
                }))
    {
        return Err("Legacy save claims new potion, Star or Zorf state".into());
    }
    Ok(())
}

pub fn load_session(options: &Options) -> Result<AdventureSession, Box<dyn Error>> {
    let path = options.save_dir.join("adventure.json");
    if options.new_game || !path.exists() {
        return Ok(AdventureSession::new(options.seed));
    }
    let (session, migrated) = decode_save_with_migration(&std::fs::read(path)?)?;
    if migrated {
        save_session(options, &session)?;
    }
    Ok(session)
}

pub fn save_session(options: &Options, session: &AdventureSession) -> Result<(), Box<dyn Error>> {
    session.validate()?;
    std::fs::create_dir_all(&options.save_dir)?;
    write_json(
        &options.save_dir.join("adventure.json"),
        &ProjectSave {
            format_version: SAVE_FORMAT_VERSION,
            session: session.clone(),
        },
    )
}

pub fn write_json(
    path: &std::path::Path,
    value: &impl serde::Serialize,
) -> Result<(), Box<dyn Error>> {
    let pending = path.with_extension("pending");
    let bytes = serde_json::to_vec_pretty(value)?;
    std::fs::write(&pending, bytes)?;
    std::fs::rename(pending, path)?;
    Ok(())
}

pub fn validate_destinations(
    options: &mut Options,
    game_root: &std::path::Path,
) -> Result<(), Box<dyn Error>> {
    options.save_dir = install::project_destination(&options.save_dir, game_root)?;
    if let Some(path) = options.evidence_dir.as_mut() {
        *path = install::project_destination(path, game_root)?;
    }
    Ok(())
}
