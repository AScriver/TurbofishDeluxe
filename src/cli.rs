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
    pub test_speed: u8,
}

impl Options {
    pub fn time_mode(&self) -> &'static str {
        if self.test_speed == 1 {
            "normal"
        } else {
            "accelerated-test"
        }
    }

    pub fn parse() -> Result<Option<Self>, Box<dyn Error>> {
        Self::parse_from(env::args().skip(1))
    }

    fn parse_from(
        arguments: impl IntoIterator<Item = String>,
    ) -> Result<Option<Self>, Box<dyn Error>> {
        let mut options = Self {
            game_dir: None,
            save_dir: default_save_dir(),
            evidence_dir: None,
            seed: SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos() as u64,
            new_game: false,
            inspect_assets: false,
            quit_after: None,
            muted: false,
            test_speed: 1,
        };
        let mut arguments = arguments.into_iter();
        let mut explicit_save_dir = false;
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--help" | "-h" => {
                    println!(
                        "Turbofish Deluxe\n\nReads your owned Insaniquarium Deluxe installation.\n\n--game-dir <directory>  Override Steam discovery\n--save-dir <directory>  Separate project saves (default: LOCALAPPDATA/TurbofishDeluxe)\n--new-game              Start a fresh project first tank\n--seed <integer>        Reproducible Rust PRNG (not retail replay parity)\n--evidence-dir <dir>    Write runtime events, state, identity and requested captures\n--inspect-assets       Decode all manifest images/effects without opening a window\n--quit-after <seconds>  Wall-clock run limit (required above 1x)\n--test-speed <1..8>    Opt-in accelerated test simulation\n--mute                 Disable effect playback\n\nAbove 1x requires explicit isolated --save-dir, --evidence-dir, --mute and --quit-after.\nEscape pauses; click Menu or press S to save. F12 captures when evidence is enabled."
                    );
                    return Ok(None);
                }
                "--game-dir" => {
                    options.game_dir =
                        Some(arguments.next().ok_or("--game-dir requires a path")?.into())
                }
                "--save-dir" => {
                    options.save_dir = arguments.next().ok_or("--save-dir requires a path")?.into();
                    explicit_save_dir = true;
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
                "--test-speed" => {
                    let factor: u8 = arguments
                        .next()
                        .ok_or("--test-speed requires an integer from 1 through 8")?
                        .parse()
                        .map_err(|_| "--test-speed requires an integer from 1 through 8")?;
                    if !(1..=8).contains(&factor) {
                        return Err("--test-speed must be from 1 through 8".into());
                    }
                    options.test_speed = factor;
                }
                "--inspect-assets" => options.inspect_assets = true,
                "--mute" => options.muted = true,
                _ => return Err(format!("Unknown argument {argument}; use --help").into()),
            }
        }
        if options.test_speed > 1
            && (!explicit_save_dir
                || options.evidence_dir.is_none()
                || !options.muted
                || options.quit_after.is_none()
                || options.save_dir.as_os_str().is_empty()
                || options
                    .evidence_dir
                    .as_ref()
                    .is_some_and(|path| path.as_os_str().is_empty()))
        {
            return Err("Accelerated tests require explicit --save-dir, --evidence-dir, --mute and --quit-after".into());
        }
        Ok(Some(options))
    }
}

fn default_save_dir() -> PathBuf {
    env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir)
        .join("TurbofishDeluxe")
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

pub const SAVE_FORMAT_VERSION: u32 = 24;

#[cfg(test)]
mod current_twenty_one_tests {
    use super::*;

    #[test]
    fn current_save_requires_profile_fields_and_rejects_old_version_label_on_new_shape() {
        let session = AdventureSession::new(42);
        let mut value = serde_json::to_value(ProjectSave {
            format_version: SAVE_FORMAT_VERSION,
            session,
        })
        .unwrap();
        for field in ["cyrax_attempts", "adventure_completed"] {
            let mut incomplete = value.clone();
            incomplete["session"]["progress"]
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(decode_save(&serde_json::to_vec(&incomplete).unwrap()).is_err());
        }
        value["format_version"] = serde_json::json!(20);
        value["session"]["progress"]
            .as_object_mut()
            .unwrap()
            .remove("cyrax_attempts");
        value["session"]["progress"]
            .as_object_mut()
            .unwrap()
            .remove("adventure_completed");
        assert!(decode_save(&serde_json::to_vec(&value).unwrap()).is_err());
    }

    #[test]
    fn current_save_requires_present_stinky_combat_identity_field() {
        let mut session = AdventureSession::new(42);
        session.board = Some(AdventureState::new_second_stage(42));
        session.progress.level = 2;
        session
            .progress
            .unlocked_pets
            .push(crate::sim::PetKind::Stinky);
        let mut value = serde_json::to_value(ProjectSave {
            format_version: SAVE_FORMAT_VERSION,
            session,
        })
        .unwrap();
        assert!(value["session"]["board"]["stinky"][0]["combat_id"].is_null());
        value["session"]["board"]["stinky"][0]
            .as_object_mut()
            .unwrap()
            .remove("combat_id");
        assert!(decode_save(&serde_json::to_vec(&value).unwrap()).is_err());
    }
}

pub fn decode_save(bytes: &[u8]) -> Result<AdventureSession, Box<dyn Error>> {
    Ok(decode_save_with_migration(bytes)?.0)
}

fn decode_save_with_migration(bytes: &[u8]) -> Result<(AdventureSession, bool), Box<dyn Error>> {
    let value: serde_json::Value = serde_json::from_slice(bytes)?;
    let version_number = value
        .get("format_version")
        .and_then(serde_json::Value::as_u64);
    if let Some(board) = value
        .pointer("/session/board")
        .and_then(serde_json::Value::as_object)
    {
        let other_fields = ["stinky", "niko", "clyde", "rufus", "rhubarb"];
        if version_number.is_some_and(|version| version >= 23)
            && other_fields
                .iter()
                .any(|field| board.get(*field).is_none_or(|pet| !pet.is_array()))
        {
            return Err("Format-twenty-three OtherPet fields must be arrays".into());
        }
        if version_number.is_some_and(|version| version < 23)
            && other_fields
                .iter()
                .any(|field| board.get(*field).is_some_and(|pet| !pet.is_array()))
        {
            return Err(
                "Option-shaped OtherPet saves are unsupported by format twenty-three".into(),
            );
        }
    }
    if version_number == Some(24) {
        let session = value
            .get("session")
            .and_then(serde_json::Value::as_object)
            .ok_or("Format-twenty-four session missing")?;
        if ["mode", "time_trial_scores", "time_trial"]
            .iter()
            .any(|field| !session.contains_key(*field))
            || value
                .pointer("/session/board")
                .filter(|board| !board.is_null())
                .is_some_and(|board| {
                    board
                        .get("time_trial")
                        .and_then(serde_json::Value::as_bool)
                        .is_none()
                })
        {
            return Err("Incomplete format-twenty-four mode or Time Trial state".into());
        }
    } else if version_number.is_some_and(|version| version < 24)
        && value.pointer("/session/mode").is_some()
    {
        return Err("Older save cannot claim Time Trial mode".into());
    }
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
        Some(version @ 5..=24) => {
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
                        && (version < 21
                            || ["cyrax_attempts", "adventure_completed"]
                                .iter()
                                .all(|field| progress.contains_key(*field)))
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
                        || (version >= 17 && incomplete_format_seventeen_board(board))
                        || (version >= 20
                            && [
                                "dead_fish",
                                "dead_oscars",
                                "dead_ultras",
                                "dead_breeders",
                                "dead_starcatchers",
                                "dead_grubbers",
                                "dead_gekkos",
                            ]
                            .iter()
                            .any(|field| {
                                board
                                    .get(*field)
                                    .and_then(serde_json::Value::as_array)
                                    .is_none_or(|corpses| {
                                        corpses
                                            .iter()
                                            .any(|corpse| corpse.get("revival_ticks").is_none())
                                    })
                            }))
                        || ((21..23).contains(&version)
                            && board
                                .get("stinky")
                                .filter(|stinky| !stinky.is_null())
                                .is_some_and(|stinky| stinky.get("combat_id").is_none()))
                        || (version >= 23
                            && board
                                .get("stinky")
                                .and_then(serde_json::Value::as_array)
                                .is_some_and(|pets| {
                                    pets.iter().any(|pet| pet.get("combat_id").is_none())
                                }))
                        || (version >= 24
                            && ["clyde", "niko", "rufus", "rhubarb"].iter().any(|field| {
                                board
                                    .get(*field)
                                    .and_then(serde_json::Value::as_array)
                                    .is_none_or(|pets| {
                                        pets.iter().any(|pet| {
                                            pet.get("presto_form").is_none()
                                                || (*field == "niko"
                                                    && ["x", "y", "vy"]
                                                        .iter()
                                                        .any(|part| pet.get(*part).is_none()))
                                        })
                                    })
                            }))
                        || (version >= 21
                            && board
                                .get("invasion")
                                .filter(|wave| !wave.is_null())
                                .is_some_and(|wave| {
                                    wave.get("finale").is_none()
                                        || wave
                                            .get("finale")
                                            .filter(|finale| !finale.is_null())
                                            .is_some_and(|finale| {
                                                [
                                                    "boss",
                                                    "children",
                                                    "boss_defeated",
                                                    "profile_attempts",
                                                    "ordinary_ticks",
                                                    "child_ticks",
                                                    "secondary_coords",
                                                ]
                                                .iter()
                                                .any(|field| finale.get(*field).is_none())
                                            })
                                }))
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
                                            || (version >= 22 && pet.get("presto_form").is_none())
                                            || (version >= 19
                                                && ["gash_timer", "gash_eating_ticks"]
                                                    .iter()
                                                    .any(|field| pet.get(*field).is_none()))
                                            || (version >= 18
                                                && ["amp_timer", "amp_threshold", "amp_charge"]
                                                    .iter()
                                                    .any(|field| pet.get(*field).is_none()))
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
                        || (version >= 23
                            && board
                                .get("niko")
                                .and_then(serde_json::Value::as_array)
                                .is_some_and(|pets| {
                                    pets.iter().any(|pet| {
                                        pet.get("anchor_x").is_none()
                                            || pet.get("anchor_y").is_none()
                                    })
                                }))
                        || ((7..23).contains(&version)
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
                    16 => "sixteen",
                    17 => "seventeen",
                    18 => "eighteen",
                    19 => "nineteen",
                    20 => "twenty",
                    21 => "twenty-one",
                    22 => "twenty-two",
                    23 => "twenty-three",
                    24 => "twenty-four",
                    _ => unreachable!("bounded format range"),
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

// Format seventeen persists the physical group and Ultra actors. Reject
// missing fields before Serde can mistake an incomplete current save for an
// older shape; historical versions keep their existing loader path.
fn incomplete_format_seventeen_board(board: &serde_json::Map<String, serde_json::Value>) -> bool {
    use serde_json::Value;
    fn missing(value: &Value, fields: &[&str]) -> bool {
        fields.iter().any(|field| value.get(*field).is_none())
    }
    fn incomplete_items(value: &Value, fields: &[&str]) -> bool {
        value
            .as_array()
            .is_none_or(|items| items.iter().any(|item| missing(item, fields)))
    }
    if ["ultra_unlocked", "ultras", "dead_ultras"]
        .iter()
        .any(|field| !board.contains_key(*field))
    {
        return true;
    }
    if incomplete_items(
        &board["ultras"],
        &[
            "id",
            "alive",
            "x",
            "y",
            "widget_x",
            "widget_y",
            "vx",
            "vy",
            "hunger",
            "cannot_be_eaten_ticks",
            "frame",
            "turn_ticks",
            "eating_ticks",
            "coin_timer",
            "coin_threshold",
            "bought_timer",
            "speed_mod",
            "previous_vx",
            "movement_state",
            "movement_timer",
            "special_timer",
            "x_direction",
            "vx_abs",
            "swim_counter",
            "speedy_speed_ticks",
            "hunger_shown",
            "hunger_animation_ticks",
        ],
    ) || incomplete_items(
        &board["dead_ultras"],
        &[
            "id",
            "x",
            "y",
            "widget_x",
            "widget_y",
            "frame",
            "opacity",
            "facing_right",
            "remaining_ticks",
            "vx",
            "vy",
            "speed_mod",
        ],
    ) {
        return true;
    }
    if board
        .get("food")
        .is_some_and(|food| incomplete_items(food, &["nimbus_rising"]))
    {
        return true;
    }
    let Some(wave) = board.get("invasion").filter(|wave| !wave.is_null()) else {
        return false;
    };
    if missing(wave, &["bilaterus", "fragments"])
        || incomplete_items(
            &wave["fragments"],
            &[
                "id",
                "kind",
                "x",
                "y",
                "widget_x",
                "widget_y",
                "vx",
                "vy",
                "frame",
                "loop_count",
            ],
        )
    {
        return true;
    }
    wave["bilaterus"].as_array().is_none_or(|groups| {
        groups.iter().any(|group| {
            if missing(
                group,
                &[
                    "id",
                    "heads",
                    "active_head",
                    "bones",
                    "emergence_ticks",
                    "swap_ticks",
                    "first_head_lost",
                    "widget_x",
                    "widget_y",
                ],
            ) {
                return true;
            }
            group["heads"].as_array().is_none_or(|heads| {
                heads.iter().any(|head| {
                    !head.is_null()
                        && missing(
                            head,
                            &[
                                "x",
                                "y",
                                "widget_x",
                                "widget_y",
                                "vx",
                                "vy",
                                "health",
                                "hit_ticks",
                                "bite_cooldown",
                                "frame",
                                "swim_ticks",
                                "turn_ticks",
                                "movement_state",
                                "movement_ticks",
                                "movement_vx",
                                "movement_vy",
                                "facing_velocity",
                                "follow_vx",
                                "follow_vy",
                                "follow_x",
                                "follow_y",
                                "follow_ticks",
                                "connector_left",
                                "turn_suppressed",
                                "back",
                            ],
                        )
                })
            }) || incomplete_items(
                &group["bones"],
                &[
                    "x",
                    "y",
                    "widget_x",
                    "widget_y",
                    "vx",
                    "vy",
                    "facing_velocity",
                    "bite_cooldown",
                    "follow_ticks",
                    "follow_vx",
                    "follow_vy",
                ],
            )
        })
    })
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
    if options.test_speed > 1 {
        if !options.muted || options.evidence_dir.is_none() || options.quit_after.is_none() {
            return Err("Accelerated tests require --evidence-dir, --mute and --quit-after".into());
        }
        let default = resolved_destination(&default_save_dir())?;
        let save = resolved_destination(&options.save_dir)?;
        let evidence = resolved_destination(options.evidence_dir.as_deref().unwrap())?;
        let working_directory = env::current_dir()?.canonicalize()?;
        let owned_install = resolved_destination(game_root)?;
        validate_accelerated_paths(
            &save,
            &evidence,
            &default,
            &working_directory,
            &owned_install,
        )?;
    }
    Ok(())
}

fn resolved_destination(path: &std::path::Path) -> Result<PathBuf, Box<dyn Error>> {
    let mut ancestor = path;
    let mut missing = Vec::new();
    while !ancestor.exists() {
        missing.push(
            ancestor
                .file_name()
                .ok_or("Invalid write destination")?
                .to_os_string(),
        );
        ancestor = ancestor.parent().ok_or("Invalid write destination")?;
    }
    let mut resolved = ancestor.canonicalize()?;
    for component in missing.into_iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn same_output_path(left: &std::path::Path, right: &std::path::Path) -> bool {
    let mut left = left.components();
    let mut right = right.components();
    loop {
        match (left.next(), right.next()) {
            (None, None) => return true,
            (Some(left), Some(right)) if same_output_component(left, right) => {}
            _ => return false,
        }
    }
}

fn output_within(path: &std::path::Path, parent: &std::path::Path) -> bool {
    let mut path = path.components();
    for component in parent.components() {
        if !path
            .next()
            .is_some_and(|part| same_output_component(part, component))
        {
            return false;
        }
    }
    true
}

fn same_output_component(left: std::path::Component<'_>, right: std::path::Component<'_>) -> bool {
    #[cfg(windows)]
    {
        left.as_os_str().to_string_lossy().to_lowercase()
            == right.as_os_str().to_string_lossy().to_lowercase()
    }
    #[cfg(not(windows))]
    {
        left == right
    }
}

fn validate_accelerated_paths(
    save: &std::path::Path,
    evidence: &std::path::Path,
    default: &std::path::Path,
    working_directory: &std::path::Path,
    owned_install: &std::path::Path,
) -> Result<(), Box<dyn Error>> {
    for path in [save, evidence] {
        if !path
            .components()
            .any(|component| matches!(component, std::path::Component::Normal(_)))
            || same_output_path(path, working_directory)
        {
            return Err("Accelerated output requires dedicated directories".into());
        }
        if output_within(path, default) {
            return Err(
                "Accelerated output must be separate from the default project save directory"
                    .into(),
            );
        }
        if output_within(path, owned_install) {
            return Err(
                "Accelerated output must be separate from the owned game installation".into(),
            );
        }
    }
    if same_output_path(save, evidence) {
        return Err("Accelerated save and evidence directories must be separate".into());
    }
    Ok(())
}

#[cfg(test)]
mod test_speed_tests {
    use super::*;

    fn parse(arguments: &[&str]) -> Result<Options, Box<dyn Error>> {
        Options::parse_from(arguments.iter().map(|argument| argument.to_string()))?
            .ok_or_else(|| "Unexpected help request".into())
    }

    #[test]
    fn default_and_explicit_one_preserve_normal_mode_without_extra_flags() {
        let default = parse(&[]).unwrap();
        let explicit = parse(&["--test-speed", "1"]).unwrap();
        assert_eq!((default.test_speed, default.time_mode()), (1, "normal"));
        assert_eq!(
            (
                default.save_dir,
                default.evidence_dir,
                default.quit_after,
                default.muted
            ),
            (
                explicit.save_dir,
                explicit.evidence_dir,
                explicit.quit_after,
                explicit.muted
            )
        );
    }

    #[test]
    fn rejects_missing_invalid_and_out_of_range_factors() {
        for arguments in [
            vec!["--test-speed"],
            vec!["--test-speed", "0"],
            vec!["--test-speed", "9"],
            vec!["--test-speed", "-1"],
            vec!["--test-speed", "1.5"],
            vec!["--test-speed", "nope"],
        ] {
            assert!(parse(&arguments).is_err(), "accepted {arguments:?}");
        }
    }

    #[test]
    fn acceleration_requires_every_safety_argument() {
        let required = [
            "--save-dir",
            "isolated-save",
            "--evidence-dir",
            "isolated-evidence",
            "--mute",
            "--quit-after",
            "1",
        ];
        for missing in [0, 2, 4, 5] {
            let mut arguments = vec!["--test-speed", "8"];
            for (index, argument) in required.iter().enumerate() {
                if index != missing
                    && !(missing == 0 && index == 1)
                    && !(missing == 2 && index == 3)
                    && !(missing == 5 && index == 6)
                {
                    arguments.push(argument);
                }
            }
            assert!(
                parse(&arguments).is_err(),
                "accepted missing index {missing}"
            );
        }
        let mut valid = vec!["--test-speed", "8"];
        valid.extend(required);
        assert_eq!(parse(&valid).unwrap().time_mode(), "accelerated-test");
        for invalid in ["0", "NaN", "inf", "-1"] {
            let mut arguments = valid.clone();
            *arguments.last_mut().unwrap() = invalid;
            assert!(parse(&arguments).is_err());
        }
    }

    #[test]
    fn accelerated_destinations_reject_default_and_owned_install() {
        let game_root = env::current_dir().unwrap().canonicalize().unwrap();
        let mut options = parse(&[
            "--test-speed",
            "2",
            "--save-dir",
            "isolated-save",
            "--evidence-dir",
            "isolated-evidence",
            "--mute",
            "--quit-after",
            "1",
        ])
        .unwrap();
        options.save_dir = env::temp_dir().join("turbofish-test-speed-save");
        options.evidence_dir = Some(env::temp_dir().join("turbofish-test-speed-evidence"));
        assert!(validate_destinations(&mut options, &game_root).is_ok());
        options.save_dir = default_save_dir();
        assert!(validate_destinations(&mut options, &game_root).is_err());
        options.save_dir = env::temp_dir().join("turbofish-test-speed-save");
        options.evidence_dir = Some(default_save_dir().join("evidence"));
        assert!(validate_destinations(&mut options, &game_root).is_err());
        options.evidence_dir = Some(game_root.join("private-evidence"));
        assert!(validate_destinations(&mut options, &game_root).is_err());
        options.save_dir = game_root.join("private-save");
        options.evidence_dir = Some(env::temp_dir().join("turbofish-test-speed-evidence"));
        assert!(validate_destinations(&mut options, &game_root).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn missing_leaf_case_aliases_and_drive_root_are_not_isolated() {
        use std::path::Path;

        let default = Path::new(r"C:\Users\Sample\AppData\Local\TurbofishDeluxe");
        let working = Path::new(r"C:\Work\Turbofish");
        let owned = Path::new(r"D:\Steam\Insaniquarium");
        let save = Path::new(r"C:\Users\Sample\AppData\Local\TURBOFISHDELUXE\run1");
        let evidence = Path::new(r"C:\Scratch\evidence");
        assert!(validate_accelerated_paths(save, evidence, default, working, owned).is_err());
        let save = Path::new(r"C:\Scratch\Foo");
        let evidence = Path::new(r"C:\Scratch\foo");
        assert!(validate_accelerated_paths(save, evidence, default, working, owned).is_err());
        let evidence = Path::new(r"D:\steam\INSANIQUARIUM\run1");
        assert!(validate_accelerated_paths(save, evidence, default, working, owned).is_err());
        let evidence = Path::new(r"C:\");
        assert!(validate_accelerated_paths(save, evidence, default, working, owned).is_err());
    }
}
