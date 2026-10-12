#[cfg(test)]
use crate::sim::AdventureState;
use crate::{adventure::AdventureSession, install, profiles::Profiles};
use std::{
    env,
    error::Error,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use turbofish_screenhost::{ScreenCommand, parse_command};

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
    /// Process launch mode; never serialized into project saves.
    pub screensaver: Option<ScreenCommand>,
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
            screensaver: None,
        };
        let mut arguments = arguments.into_iter().peekable();
        let mut explicit_save_dir = false;
        let mut explicit_seed = false;
        let mut explicit_test_speed = false;
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--help" | "-h" => {
                    println!(
                        "Turbofish Deluxe\n\nReads your owned Insaniquarium Deluxe installation.\n\n--game-dir <directory>  Override Steam discovery\n--save-dir <directory>  Separate project saves (default: LOCALAPPDATA/TurbofishDeluxe)\n--new-game              Start a fresh project first tank\n--seed <integer>        Reproducible Rust PRNG (not retail replay parity)\n--evidence-dir <dir>    Write runtime events, state, identity and requested captures\n--inspect-assets       Decode all manifest images/effects without opening a window\n--quit-after <seconds>  Wall-clock run limit (required above 1x)\n--test-speed <1..8>    Opt-in accelerated test simulation\n--mute                 Disable effect playback\n/s                     Full-screen screensaver\n/p <decimal HWND>      Embed screensaver preview (/p:HWND also works)\n/c[:decimal HWND]      Screensaver preferences\n\nScreensaver /s and /p accept --game-dir, --save-dir, --evidence-dir, --mute and --quit-after. /c does not accept --evidence-dir. Above 1x requires explicit isolated --save-dir, --evidence-dir, --mute and --quit-after.\nEscape pauses; click Menu or press S to save. F12 captures when evidence is enabled."
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
                    explicit_seed = true;
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
                    explicit_test_speed = true;
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
                _ if argument.starts_with('/') => {
                    if options.screensaver.is_some() {
                        return Err("Only one screensaver switch is allowed".into());
                    }
                    let switch = argument
                        .split_once(':')
                        .map_or(argument.as_str(), |(s, _)| s)
                        .to_owned();
                    let mut switch_args = vec![argument];
                    if switch.eq_ignore_ascii_case("/p") && switch_args[0].find(':').is_none() {
                        switch_args.push(arguments.next().ok_or("/p requires a decimal HWND")?);
                    } else if switch.eq_ignore_ascii_case("/c")
                        && switch_args[0].find(':').is_none()
                        && arguments
                            .peek()
                            .is_some_and(|next| next.bytes().all(|byte| byte.is_ascii_digit()))
                    {
                        switch_args.push(arguments.next().expect("peeked decimal HWND"));
                    }
                    options.screensaver = Some(parse_command(&switch_args)?);
                }
                _ => return Err(format!("Unknown argument {argument}; use --help").into()),
            }
        }
        if options.screensaver.is_some()
            && (options.new_game || options.inspect_assets || explicit_seed || explicit_test_speed)
        {
            return Err("Screensaver launch does not accept --new-game, --seed, --inspect-assets or --test-speed".into());
        }
        if matches!(options.screensaver, Some(ScreenCommand::Configure(_)))
            && options.evidence_dir.is_some()
        {
            return Err("Screensaver /c has no scene to capture; omit --evidence-dir".into());
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

pub const SAVE_FORMAT_VERSION: u32 = 37;

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
    decode_save_value(serde_json::from_slice(bytes)?)
}

fn decode_save_value(mut value: serde_json::Value) -> Result<AdventureSession, Box<dyn Error>> {
    if value
        .get("format_version")
        .and_then(serde_json::Value::as_u64)
        != Some(u64::from(SAVE_FORMAT_VERSION))
    {
        return Err("Unsupported project save version; original saves are not imported".into());
    }
    let session = value
        .get_mut("session")
        .ok_or("Current project session missing")?
        .take();
    decode_current_session(session)
}

/// Admit the current session shape, including required nullable fields.
/// Both standalone envelopes and profile entries use this direct decoder.
pub(crate) fn decode_current_session(
    value: serde_json::Value,
) -> Result<AdventureSession, Box<dyn Error>> {
    for board in [
        "/board",
        "/retained_adventure/board",
        "/retained_time_trial/board",
        "/retained_challenge/board",
    ]
    .iter()
    .filter_map(|path| value.pointer(path).and_then(serde_json::Value::as_object))
    {
        let other_fields = ["stinky", "niko", "clyde", "rufus", "rhubarb"];
        if other_fields
            .iter()
            .any(|field| board.get(*field).is_none_or(|pet| !pet.is_array()))
        {
            return Err("Format-twenty-three OtherPet fields must be arrays".into());
        }
    }
    {
        let session = value
            .pointer("")
            .and_then(serde_json::Value::as_object)
            .ok_or("Format-twenty-six session missing")?;
        if ["mode", "time_trial_scores", "time_trial"]
            .iter()
            .any(|field| !session.contains_key(*field))
            || [
                "/board",
                "/retained_adventure/board",
                "/retained_time_trial/board",
                "/retained_challenge/board",
            ]
            .iter()
            .filter_map(|path| value.pointer(path).filter(|board| !board.is_null()))
            .any(|board| {
                board
                    .get("time_trial")
                    .and_then(serde_json::Value::as_bool)
                    .is_none()
            })
        {
            return Err("Incomplete format-twenty-six mode or Time Trial state".into());
        }
    }
    let mode_boards: Vec<_> = [
        "/board",
        "/retained_adventure/board",
        "/retained_time_trial/board",
        "/retained_challenge/board",
    ]
    .iter()
    .filter_map(|path| value.pointer(path).filter(|board| !board.is_null()))
    .collect();
    {
        let purchase = |result: &serde_json::Value| {
            result.get("purchase").is_some_and(|receipt| {
                ["offered_cursor", "confirming", "purchased"]
                    .iter()
                    .all(|field| receipt.get(*field).is_some())
            })
        };
        if value.pointer("/progress/purchase_cursor").is_none()
            || value
                .pointer("/phase/BonusResults/result")
                .is_some_and(|result| !purchase(result))
            || [
                "/time_trial/result",
                "/retained_time_trial/time_trial/result",
            ]
            .iter()
            .filter_map(|path| value.pointer(path).filter(|result| !result.is_null()))
            .any(|result| result.get("updates").is_none() || !purchase(result))
            || mode_boards.iter().any(|board| {
                board
                    .get("fish_pets")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|pets| {
                        pets.iter().any(|pet| {
                            pet.get("brinkley_meals").is_none()
                                || pet.get("brinkley_cooldown").is_none()
                        })
                    })
            })
        {
            return Err("Incomplete format-twenty-seven purchase or Brinkley state".into());
        }
    }
    if mode_boards.iter().any(|board| {
        board
            .get("fish_pets")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|pets| {
                pets.iter().any(|pet| {
                    ["nostra_elapsed", "nostra_threshold", "nostra_converted"]
                        .iter()
                        .any(|field| pet.get(*field).is_none())
                })
            })
            || board
                .get("food")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|food| {
                    food.iter().any(|item| {
                        ["food_type", "picked_up"]
                            .iter()
                            .any(|field| item.get(*field).is_none())
                    })
                })
            || board
                .get("invasion")
                .filter(|wave| !wave.is_null())
                .is_some_and(|wave| wave.get("sneeze_shake_ticks").is_none())
    }) {
        return Err("Incomplete format-twenty-eight Nostradamus state".into());
    }
    if mode_boards.iter().any(|board| {
        board
            .get("fish_pets")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|pets| {
                pets.iter().any(|pet| {
                    pet.get("stanley_action_ticks").is_none()
                        || pet.get("stanley_diversion_cooldown").is_none()
                })
            })
            || board
                .get("missiles")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|missiles| {
                    missiles.iter().any(|missile| {
                        [
                            "target_id",
                            "stanley_hit",
                            "diversion_clock",
                            "diversion_limit",
                            "diversion_x",
                            "diversion_y",
                            "movement_divisor",
                        ]
                        .iter()
                        .any(|field| missile.get(*field).is_none())
                    })
                })
    }) {
        return Err("Incomplete format-twenty-nine Stanley state".into());
    }
    if mode_boards.iter().any(|board| {
        ["fish", "oscars", "ultras", "gekkos", "fish_pets"]
            .iter()
            .any(|kind| {
                board
                    .get(*kind)
                    .and_then(serde_json::Value::as_array)
                    .is_none_or(|actors| {
                        actors.iter().any(|actor| {
                            actor.get("walter").is_none()
                                || (*kind == "fish_pets"
                                    && [
                                        "age",
                                        "walter_action",
                                        "walter_punches",
                                        "walter_cooldown",
                                        "glove",
                                    ]
                                    .iter()
                                    .any(|field| actor.get(*field).is_none()))
                        })
                    })
            })
    }) {
        return Err("Incomplete format-thirty Walter state".into());
    }
    if value.pointer("/challenge").is_none()
        || value.pointer("/challenge_scores").is_none()
        || ["/challenge", "/retained_challenge/challenge"]
            .iter()
            .filter_map(|path| value.pointer(path).filter(|run| !run.is_null()))
            .any(|run| run.get("result").is_none())
        || value
            .pointer("/progress/challenge_completed")
            .and_then(serde_json::Value::as_array)
            .is_none_or(|flags| flags.len() != 4 || flags.iter().any(|flag| !flag.is_boolean()))
        || [
            "/board",
            "/retained_adventure/board",
            "/retained_time_trial/board",
            "/retained_challenge/board",
        ]
        .iter()
        .filter_map(|path| value.pointer(path).filter(|board| !board.is_null()))
        .any(|board| board.get("challenge_shop").is_none())
    {
        return Err("Incomplete format-thirty-two Challenge state".into());
    }
    {
        let session = value
            .pointer("")
            .ok_or("Format-thirty-three session missing")?;
        if [
            "virtual_tank",
            "virtual_board",
            "virtual_saved_at_seconds",
            "virtual_seed",
        ]
        .iter()
        .any(|key| session.get(*key).is_none())
            || session.get("virtual_tank").is_some_and(|tank| {
                [
                    "fish",
                    "selected_fish",
                    "selected_pets",
                    "owned_backdrops",
                    "backdrop",
                    "bubbulator_owned",
                    "alien_attractor_owned",
                    "food_upgrade_owned",
                    "settings",
                    "stock",
                    "board_shells",
                ]
                .iter()
                .any(|key| tank.get(*key).is_none())
                    || tank.get("settings").is_some_and(|settings| {
                        [
                            "show_names",
                            "bubbulator_shown",
                            "alien_attractor_shown",
                            "drop_shells",
                            "always_show_hungry",
                        ]
                        .iter()
                        .any(|key| settings.get(*key).is_none())
                    })
                    || tank
                        .get("stock")
                        .filter(|stock| !stock.is_null())
                        .is_some_and(|stock| {
                            ["cycle", "offers", "bought"]
                                .iter()
                                .any(|key| stock.get(*key).is_none())
                        })
                    || tank
                        .get("fish")
                        .and_then(serde_json::Value::as_array)
                        .is_none_or(|fish| {
                            fish.iter().any(|fish| {
                                [
                                    "id",
                                    "name",
                                    "kind",
                                    "properties",
                                    "shown",
                                    "alive",
                                    "purchased_at_seconds",
                                    "bought_today",
                                    "purchase_price",
                                    "size",
                                    "hunger",
                                    "food_ate",
                                    "food_needed_to_grow",
                                    "growth_ticks",
                                    "coin_ticks",
                                    "coin_threshold",
                                    "breeder_child",
                                    "hometown",
                                    "likes",
                                    "day_start_seconds",
                                    "last_mental_day_seconds",
                                    "fed_today",
                                    "mental_state",
                                    "has_given_birth",
                                ]
                                .iter()
                                .any(|key| fish.get(*key).is_none())
                                    || fish.get("properties").is_some_and(|properties| {
                                        [
                                            "invisible",
                                            "voracious",
                                            "speedy",
                                            "singing",
                                            "backward",
                                            "exotic_diet",
                                            "named",
                                            "colors",
                                            "color_roll",
                                        ]
                                        .iter()
                                        .any(|key| properties.get(*key).is_none())
                                    })
                            })
                        })
            })
            || session
                .get("virtual_board")
                .filter(|board| !board.is_null())
                .is_some_and(|board| {
                    ["tick", "poses", "food", "coins", "next_object_id"]
                        .iter()
                        .any(|key| board.get(*key).is_none())
                        || board
                            .get("poses")
                            .and_then(serde_json::Value::as_array)
                            .is_none_or(|poses| {
                                poses.iter().any(|pose| {
                                    ["fish_id", "x", "y", "vx"]
                                        .iter()
                                        .any(|key| pose.get(*key).is_none())
                                })
                            })
                        || board
                            .get("food")
                            .and_then(serde_json::Value::as_array)
                            .is_none_or(|food| {
                                food.iter().any(|item| {
                                    ["x", "y"].iter().any(|key| item.get(*key).is_none())
                                })
                            })
                        || board
                            .get("coins")
                            .and_then(serde_json::Value::as_array)
                            .is_none_or(|coins| {
                                coins.iter().any(|coin| {
                                    ["id", "x", "y", "kind"]
                                        .iter()
                                        .any(|key| coin.get(*key).is_none())
                                })
                            })
                })
        {
            return Err("Incomplete format-thirty-three Virtual Tank state".into());
        }
    }
    {
        let tank = value
            .pointer("/last_non_adventure_tank")
            .and_then(serde_json::Value::as_u64)
            .ok_or("Format-thirty-four Sandbox tank missing")?;
        if !(1..=5).contains(&tank)
            || value.pointer("/sandbox_board").is_some()
            || value
                .pointer("/finale_fallen_pets")
                .and_then(serde_json::Value::as_array)
                .is_none()
            || value.pointer("/progress/tank5_best_survivors").is_none()
        {
            return Err("Invalid format-thirty-four Sandbox state".into());
        }
        if value
            .pointer("/virtual_board")
            .is_some_and(|board| !board.is_null())
        {
            let board = value.pointer("/virtual_board").expect("checked above");
            if ["pets", "prey"].iter().any(|key| {
                board
                    .get(*key)
                    .and_then(serde_json::Value::as_array)
                    .is_none()
            }) || board
                .get("food")
                .and_then(serde_json::Value::as_array)
                .is_none_or(|food| food.iter().any(|item| item.get("kind").is_none()))
            {
                return Err("Incomplete format-thirty-four Virtual Board state".into());
            }
        }
    }
    if let Some(board) = value
        .pointer("/virtual_board")
        .filter(|board| !board.is_null())
    {
        let missing = |item: &serde_json::Value, keys: &[&str]| {
            keys.iter().any(|key| item.get(*key).is_none())
        };
        let incomplete_items = |key: &str, keys: &[&str]| {
            board
                .get(key)
                .and_then(serde_json::Value::as_array)
                .is_none_or(|items| items.iter().any(|item| missing(item, keys)))
        };
        if board.get("combat").is_none_or(|combat| {
            missing(
                combat,
                &["wave", "missiles", "next_after_spawn", "active_age"],
            )
        }) || incomplete_items("poses", &["walter_recoil_ticks"])
            || incomplete_items("food", &["id", "ineligible_ticks"])
            || incomplete_items("prey", &["id", "walter_recoil_ticks"])
            || incomplete_items(
                "corpses",
                &["id", "kind", "x", "y", "life", "revival_ticks"],
            )
            || incomplete_items("coins", &["niko_owner", "age"])
            || incomplete_items(
                "pets",
                &[
                    "active_form",
                    "vx",
                    "previous_vx",
                    "niko_pearl_taken",
                    "brinkley_meals",
                    "brinkley_cooldown",
                    "zorf_brinkley_cycles",
                    "walter_action_ticks",
                    "glove",
                    "amp_charge",
                ],
            )
        {
            return Err("Incomplete format-thirty-five Virtual Tank state".into());
        }
    }
    if value.pointer("/progress/stories").is_none_or(|stories| {
        ["seen_mask", "cyrax_unlocked"]
            .iter()
            .any(|key| stories.get(*key).is_none())
    }) {
        return Err("Incomplete format-thirty-six Story state".into());
    }
    if [
        "retained_adventure",
        "retained_time_trial",
        "retained_challenge",
    ]
    .iter()
    .any(|slot| {
        value
            .pointer("")
            .and_then(|session| session.get(*slot))
            .is_none_or(|scene| {
                !scene.is_null()
                    && [
                        "phase",
                        "board",
                        "next_seed",
                        "time_trial",
                        "challenge",
                        "selected_pets",
                    ]
                    .iter()
                    .any(|field| scene.get(*field).is_none())
            })
    }) {
        return Err("Incomplete format-thirty-seven retained mode scene".into());
    }
    let complete_progress = value
        .pointer("/progress")
        .and_then(serde_json::Value::as_object)
        .is_some_and(|progress| {
            ["first_stage_best_seconds", "later_stage_best_seconds"]
                .iter()
                .all(|field| progress.contains_key(*field))
                && (["pet_capacity", "selected_pets"]
                    .iter()
                    .all(|field| progress.contains_key(*field)))
                && (progress.contains_key("shell_balance"))
                && (["cyrax_attempts", "adventure_completed"]
                    .iter()
                    .all(|field| progress.contains_key(*field)))
        });
    let incomplete_board = [
        "/board",
        "/retained_adventure/board",
        "/retained_time_trial/board",
        "/retained_challenge/board",
    ]
    .iter()
    .filter_map(|path| value.pointer(path).and_then(serde_json::Value::as_object))
    .any(|board| {
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
            || (["fish_pets", "punch_sound_cooldown"]
                .iter()
                .any(|field| !board.contains_key(*field)))
            || (["potion_unlocked", "potion_armed"]
                .iter()
                .any(|field| !board.contains_key(*field)))
            || ([
                "starcatcher_unlocked",
                "starcatchers",
                "dead_starcatchers",
                "clyde",
            ]
            .iter()
            .any(|field| !board.contains_key(*field)))
            || (["missiles", "rufus"]
                .iter()
                .any(|field| !board.contains_key(*field)))
            || (!board.contains_key("notes"))
            || (!board.contains_key("bomb_shots"))
            || (["breeder_unlocked", "breeders", "dead_breeders", "rhubarb"]
                .iter()
                .any(|field| !board.contains_key(*field)))
            || (incomplete_physical_group_board(board))
            || ([
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
            || (board
                .get("stinky")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|pets| {
                    pets.iter().any(|pet| {
                        pet.get("combat_id").is_none() || (pet.get("presto_form").is_none())
                    })
                }))
            || (["clyde", "niko", "rufus", "rhubarb"].iter().any(|field| {
                board
                    .get(*field)
                    .and_then(serde_json::Value::as_array)
                    .is_none_or(|pets| {
                        pets.iter().any(|pet| {
                            pet.get("presto_form").is_none()
                                || (*field == "niko"
                                    && ["x", "y", "vy"].iter().any(|part| pet.get(*part).is_none()))
                        })
                    })
            }))
            || (board
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
            || (board
                .get("missiles")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|missiles| {
                    missiles.iter().any(|missile| {
                        ["kind", "reflected"]
                            .iter()
                            .any(|field| missile.get(*field).is_none())
                    })
                }))
            || (["gekko_unlocked", "gekkos", "dead_gekkos"]
                .iter()
                .any(|field| !board.contains_key(*field)))
            || (board
                .get("coins")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|coins| {
                    coins
                        .iter()
                        .any(|coin| coin.get("animation_ticks").is_none())
                }))
            || (board
                .get("coins")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|coins| {
                    coins
                        .iter()
                        .any(|coin| coin.get("hazard_age_ticks").is_none())
                }))
            || (["grubber_unlocked", "grubbers", "dead_grubbers", "larvae"]
                .iter()
                .any(|field| !board.contains_key(*field)))
            || (board
                .get("fish_pets")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|pets| {
                    pets.iter().any(|pet| {
                        pet.get("coin_timer").is_none()
                            || (pet.get("presto_form").is_none())
                            || (["gash_timer", "gash_eating_ticks"]
                                .iter()
                                .any(|field| pet.get(*field).is_none()))
                            || (["amp_timer", "amp_threshold", "amp_charge"]
                                .iter()
                                .any(|field| pet.get(*field).is_none()))
                            || (["bomb_threshold", "glint_phase"]
                                .iter()
                                .any(|field| pet.get(*field).is_none()))
                            || pet.get("meryl_blink").is_none()
                            || (["ward_active", "ward_timer", "published_x", "published_y"]
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
                            || ([
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
                    ["plan", "actors", "warps", "dead_aliens", "battle_active"]
                        .iter()
                        .any(|field| !wave.contains_key(*field))
                        || !wave.contains_key("gus_warning_shown")
                        || wave
                            .get("actors")
                            .and_then(serde_json::Value::as_array)
                            .is_some_and(|actors| {
                                actors.iter().any(|actor| actor.get("kind").is_none())
                            })
                        || wave
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
                            })
                        || wave
                            .get("dead_aliens")
                            .and_then(serde_json::Value::as_array)
                            .is_some_and(|bodies| {
                                bodies.iter().any(|body| body.get("kind").is_none())
                            })
                        || wave
                            .get("alien")
                            .and_then(serde_json::Value::as_object)
                            .is_some_and(|alien| !alien.contains_key("kind"))
                        || wave
                            .get("dead_alien")
                            .and_then(serde_json::Value::as_object)
                            .is_some_and(|body| !body.contains_key("kind"))
                })
            || (board
                .get("niko")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|pets| {
                    pets.iter()
                        .any(|pet| pet.get("anchor_x").is_none() || pet.get("anchor_y").is_none())
                }))
    });
    if !complete_progress || incomplete_board {
        return Err(
            "Incomplete format-thirty-seven save; required state fields are missing".into(),
        );
    }
    let session: AdventureSession = serde_json::from_value(value)?;
    session.validate()?;
    Ok(session)
}

// Physical groups and Ultra actors require their complete current state;
// nullable heads remain distinct from omitted actor fields.
fn incomplete_physical_group_board(board: &serde_json::Map<String, serde_json::Value>) -> bool {
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

pub fn load_session(options: &Options) -> Result<AdventureSession, Box<dyn Error>> {
    let path = options.save_dir.join("adventure.json");
    if options.new_game || !path.exists() {
        return Ok(AdventureSession::new(options.seed));
    }
    decode_save(&std::fs::read(path)?)
}

/// Profiles are the runtime owner; standalone saves are not adopted.
pub fn load_profiles(options: &Options) -> Result<Profiles, Box<dyn Error>> {
    let profiles_path = options.save_dir.join("profiles.json");
    if profiles_path.exists() {
        let mut profiles = Profiles::decode(&std::fs::read(&profiles_path)?)?;
        if options.new_game {
            if let Some(current) = profiles.current_mut() {
                current.reset_session(options.seed);
            } else {
                return Ok(Profiles::new("Player", options.seed)?);
            }
        }
        return Ok(profiles);
    }
    Ok(Profiles::new("Player", options.seed)?)
}

pub fn save_profiles_checkpoint(
    options: &Options,
    profiles: &Profiles,
) -> Result<(), Box<dyn Error>> {
    profiles.save(&options.save_dir.join("profiles.json"))
}

/// `score_published` is valid only immediately after a successful score save,
/// with no intervening profile mutation. Every profile save already contains
/// the complete checkpoint projection, including Sandbox origins.
pub(crate) fn save_profiles_checkpoint_after_score(
    options: &Options,
    profiles: &Profiles,
    score_published: bool,
) -> Result<bool, Box<dyn Error>> {
    if score_published {
        return Ok(false);
    }
    save_profiles_checkpoint(options, profiles)?;
    Ok(true)
}

/// Preserve the final wall-time stamp. An earlier exit checkpoint covers this
/// boundary only if its complete projected payload remains unchanged.
pub(crate) fn save_final_profiles_checkpoint(
    options: &Options,
    profiles: &mut Profiles,
    exit_checkpoint_published: bool,
    now_seconds: u64,
) -> Result<bool, Box<dyn Error>> {
    let previous_timestamp = profiles.current_mut().map(|current| {
        std::mem::replace(&mut current.session.virtual_saved_at_seconds, now_seconds)
    });
    if exit_checkpoint_published
        && previous_timestamp.is_none_or(|previous| previous == now_seconds)
    {
        return Ok(false);
    }
    if let Err(error) = save_profiles_checkpoint(options, profiles) {
        if let (Some(current), Some(previous)) = (profiles.current_mut(), previous_timestamp) {
            current.session.virtual_saved_at_seconds = previous;
        }
        return Err(error);
    }
    Ok(true)
}

#[cfg(test)]
mod checkpoint_tests {
    use super::*;
    use crate::{
        adventure::{AdventurePhase, GameMode, PetKind},
        profiles::ProfileEntry,
        sim::{Action, Event},
    };
    use sha2::{Digest, Sha256};
    use std::{
        hint::black_box,
        sync::atomic::{AtomicU64, Ordering},
        time::Instant,
    };

    struct FixtureDirectory(PathBuf);

    impl FixtureDirectory {
        fn new(label: &str) -> Self {
            static SEQUENCE: AtomicU64 = AtomicU64::new(0);
            let path = env::current_dir()
                .unwrap()
                .join(".scratch/performance/294/fixtures")
                .join(format!(
                    "{label}-{}-{}",
                    std::process::id(),
                    SEQUENCE.fetch_add(1, Ordering::Relaxed)
                ));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn options(&self) -> Options {
            Options {
                game_dir: None,
                save_dir: self.0.clone(),
                evidence_dir: None,
                seed: 0xa294,
                new_game: false,
                inspect_assets: false,
                quit_after: None,
                muted: true,
                test_speed: 1,
                screensaver: None,
            }
        }
    }

    impl Drop for FixtureDirectory {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    fn fixture(profile_count: u64, retained: bool, sandbox: bool) -> Profiles {
        let mut profiles = Profiles::new("Fixture 1", 0xa294).unwrap();
        profiles.entries.clear();
        profiles.next_id = profile_count + 1;
        for id in 1..=profile_count {
            let mut session = AdventureSession::new(0xa294 + id);
            if retained {
                session.progress.tank = 2;
                session.progress.level = 1;
                session.progress.unlocked_pets = vec![
                    PetKind::Stinky,
                    PetKind::Niko,
                    PetKind::Itchy,
                    PetKind::Prego,
                    PetKind::Zorf,
                ];
                session.progress.selected_pets = vec![PetKind::Stinky, PetKind::Niko];
                session.board = Some(
                    AdventureState::new_tank2_first_stage(
                        0xa294 + id,
                        &session.progress.selected_pets,
                    )
                    .unwrap(),
                );
                session.select_mode(GameMode::TimeTrial, 1_000, 0).unwrap();
                session.apply_actions(&[Action::SelectTimeTrialTank { tank: 1 }, Action::Continue]);
                assert_eq!(session.phase, AdventurePhase::TimeTrialPlaying);
                for _ in 0..8 {
                    session.step(&[]);
                }
                session.select_mode(GameMode::Adventure, 1_000, 0).unwrap();
                assert!(session.retained_time_trial.is_some());
            }
            profiles.entries.push(ProfileEntry {
                id,
                name: format!("Fixture {id}"),
                session,
                screensaver_generation: format!("{id:032x}"),
                screensaver_claimed: 0,
                screensaver_scene_consumed: 0,
            });
        }
        if sandbox {
            let session = &mut profiles.current_mut().unwrap().session;
            session.progress.unlocked_pets.extend([
                PetKind::Brinkley,
                PetKind::Nostradamus,
                PetKind::Stanley,
                PetKind::Walter,
            ]);
            session.progress.purchase_cursor = 6;
            session.progress.pet_capacity = 7;
            session.select_mode(GameMode::Sandbox, 1_000, 0).unwrap();
            session.apply_actions(&[
                Action::SandboxKeyChar { character: '2' },
                Action::SandboxKeyChar { character: 'D' },
            ]);
            assert_eq!(session.mode, GameMode::Sandbox);
            assert!(session.sandbox_board.is_some());
        }
        profiles.validate().unwrap();
        profiles
    }

    fn legacy_checkpoint(profiles: &Profiles) -> Profiles {
        let mut snapshot = profiles.clone();
        if let Some(current) = snapshot.current_mut() {
            current.session = current.session.checkpoint_without_sandbox();
        }
        snapshot
    }

    fn digest(bytes: &[u8]) -> String {
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    fn result_event() -> Event {
        Event::TimeTrialExpired {
            tick: 8,
            tank: 1,
            score: 500,
        }
    }

    #[test]
    fn checkpoints_preserve_complete_current_profiles_and_retained_scenes() {
        for (name, profiles) in [
            ("single", fixture(1, false, false)),
            ("retained", fixture(8, true, false)),
            ("sandbox", fixture(8, true, true)),
            (
                "empty",
                Profiles {
                    current_id: None,
                    entries: vec![],
                    ..fixture(1, false, false)
                },
            ),
        ] {
            let directory = FixtureDirectory::new(name);
            let memory_before = format!("{profiles:?}");
            let expected = serde_json::to_vec_pretty(&legacy_checkpoint(&profiles)).unwrap();
            save_profiles_checkpoint(&directory.options(), &profiles).unwrap();
            let bytes = std::fs::read(directory.0.join("profiles.json")).unwrap();
            assert_eq!(bytes, expected, "complete projection: {name}");
            let reopened = Profiles::decode(&bytes).unwrap();
            assert_eq!(serde_json::to_vec_pretty(&reopened).unwrap(), expected);
            assert_eq!(reopened.entries.len(), profiles.entries.len());
            assert_eq!(
                reopened.format_version,
                crate::profiles::PROFILE_FORMAT_VERSION
            );
            assert_eq!(format!("{profiles:?}"), memory_before);
            if name == "sandbox" {
                let current = &reopened.current().unwrap().session;
                assert_eq!(current.mode, GameMode::Adventure);
                assert_eq!(current.phase, AdventurePhase::Playing);
                assert!(current.sandbox_board.is_none());
                assert!(current.retained_time_trial.is_some());
                assert_eq!(
                    serde_json::to_value(&current.board).unwrap(),
                    serde_json::to_value(&profiles.current().unwrap().session.board).unwrap()
                );
            }
        }
    }

    #[test]
    fn checkpoint_projection_clears_residual_board_and_private_origin() {
        for keep_board in [false, true] {
            let mut profiles = fixture(8, true, true);
            let session = &mut profiles.current_mut().unwrap().session;
            session.mode = GameMode::Adventure;
            session.phase = AdventurePhase::Playing;
            if !keep_board {
                session.sandbox_board = None;
            }
            // Leaving the mode fields manually retains private sandbox_origin.
            // The existing projection clears it even outside Sandbox mode.
            assert!(session.validate().is_err());
            assert!(session.checkpoint_needs_sandbox_projection());
            let before = format!("{profiles:?}");
            let checkpoint = profiles.checkpoint();
            assert!(matches!(checkpoint, std::borrow::Cow::Owned(_)));
            checkpoint.validate().unwrap();
            assert_eq!(
                serde_json::to_vec(&*checkpoint).unwrap(),
                serde_json::to_vec(&legacy_checkpoint(&profiles)).unwrap()
            );
            assert!(
                !checkpoint
                    .current()
                    .unwrap()
                    .session
                    .checkpoint_needs_sandbox_projection()
            );
            assert_eq!(format!("{profiles:?}"), before);
        }
    }

    #[test]
    fn scored_results_publish_once_when_projection_is_identity() {
        let events = [
            result_event(),
            Event::StageResultRecorded {
                tick: 8,
                tank: 1,
                level: 1,
                seconds: 42,
                settled_coin_ids: vec![],
                settled_amount: 0,
                final_balance: 0,
                personal_best_seconds: 42,
                survivors: None,
            },
            Event::ChallengeCompleted {
                tick: 8,
                tank: 1,
                seconds: 42,
            },
        ];
        for event in events {
            let directory = FixtureDirectory::new("scored-result");
            let options = directory.options();
            let mut profiles = fixture(8, true, false);
            let published = profiles
                .record_events(&directory.0.join("profiles.json"), &[event])
                .unwrap();
            assert!(published);
            let before = std::fs::read(directory.0.join("profiles.json")).unwrap();
            #[cfg(windows)]
            let _reader = {
                use std::os::windows::fs::OpenOptionsExt;
                // Replacement would fail and leave a pending generation if the
                // equivalent follow-up attempted another publication.
                std::fs::OpenOptions::new()
                    .read(true)
                    .share_mode(1 | 2)
                    .open(directory.0.join("profiles.json"))
                    .unwrap()
            };
            let followup =
                save_profiles_checkpoint_after_score(&options, &profiles, published).unwrap();
            assert!(!followup);
            assert!(!directory.0.join("profiles.pending").exists());
            assert_eq!(u8::from(published) + u8::from(followup), 1);
            assert_eq!(
                std::fs::read(directory.0.join("profiles.json")).unwrap(),
                before
            );
            assert_eq!(
                serde_json::to_vec_pretty(&Profiles::decode(&before).unwrap()).unwrap(),
                serde_json::to_vec_pretty(&profiles).unwrap()
            );
        }
    }

    #[test]
    fn unscored_transitions_explicit_saves_and_later_mutations_still_publish() {
        let directory = FixtureDirectory::new("save-boundaries");
        let options = directory.options();
        let mut profiles = fixture(1, false, false);
        let published = profiles
            .record_events(&directory.0.join("profiles.json"), &[])
            .unwrap();
        assert!(!published);
        assert!(save_profiles_checkpoint_after_score(&options, &profiles, published).unwrap());
        profiles
            .current_mut()
            .unwrap()
            .session
            .apply_actions(&[Action::OpenMenu]);
        profiles.validate().unwrap();
        assert!(save_profiles_checkpoint_after_score(&options, &profiles, false).unwrap());
        profiles
            .current_mut()
            .unwrap()
            .session
            .virtual_saved_at_seconds = 1_234;
        assert!(save_profiles_checkpoint_after_score(&options, &profiles, false).unwrap());

        assert!(
            profiles
                .record_events(&directory.0.join("profiles.json"), &[result_event()])
                .unwrap()
        );
        // App invalidates the score proof before any subsequent saver transaction.
        profiles
            .current_mut()
            .unwrap()
            .session
            .progress
            .shell_balance = 19;
        assert!(save_profiles_checkpoint_after_score(&options, &profiles, false).unwrap());
        let reopened =
            Profiles::decode(&std::fs::read(directory.0.join("profiles.json")).unwrap()).unwrap();
        assert_eq!(
            reopened.current().unwrap().session.progress.shell_balance,
            19
        );
        assert_eq!(
            reopened.current().unwrap().session.virtual_saved_at_seconds,
            1_234
        );

        for score in 501..=505 {
            profiles
                .global_scores
                .record(
                    crate::highscores::ScoreKind::TimeTrial,
                    1,
                    None,
                    "Other",
                    score,
                )
                .unwrap();
        }
        let published = profiles
            .record_events(&directory.0.join("profiles.json"), &[result_event()])
            .unwrap();
        assert!(
            !published,
            "a nonqualifying result is not a durable score save"
        );
        assert!(save_profiles_checkpoint_after_score(&options, &profiles, published).unwrap());
    }

    #[test]
    fn sandbox_score_publication_already_contains_origin_projection() {
        let directory = FixtureDirectory::new("sandbox-score");
        let options = directory.options();
        let mut profiles = fixture(8, true, true);
        let seed_before = serde_json::to_value(
            profiles
                .current()
                .unwrap()
                .session
                .checkpoint_without_sandbox(),
        )
        .unwrap()["next_seed"]
            .clone();
        assert!(
            profiles
                .record_events(&directory.0.join("profiles.json"), &[result_event()])
                .unwrap()
        );
        let raw: serde_json::Value =
            serde_json::from_slice(&std::fs::read(directory.0.join("profiles.json")).unwrap())
                .unwrap();
        assert_eq!(raw["entries"][0]["session"]["mode"], "Adventure");
        let before = format!("{profiles:?}");
        assert!(!save_profiles_checkpoint_after_score(&options, &profiles, true).unwrap());
        let bytes = std::fs::read(directory.0.join("profiles.json")).unwrap();
        let reopened = Profiles::decode(&bytes).unwrap();
        assert_eq!(
            reopened.current().unwrap().session.mode,
            GameMode::Adventure
        );
        assert_eq!(
            reopened.current().unwrap().session.phase,
            AdventurePhase::Playing
        );
        assert_eq!(
            serde_json::to_value(&reopened.current().unwrap().session).unwrap()["next_seed"],
            seed_before
        );
        assert_eq!(
            bytes,
            serde_json::to_vec_pretty(&legacy_checkpoint(&profiles)).unwrap()
        );
        assert_eq!(format!("{profiles:?}"), before);
    }

    #[test]
    fn profile_transactions_publish_sandbox_origins_and_keep_live_sessions() {
        for operation in ["select", "create", "rename", "delete", "score"] {
            let directory = FixtureDirectory::new(operation);
            let options = directory.options();
            let path = directory.0.join("profiles.json");
            let mut profiles = fixture(2, true, true);
            save_profiles_checkpoint(&options, &profiles).unwrap();
            Profiles::decode(&std::fs::read(&path).unwrap()).unwrap();
            let live_before = format!("{:?}", profiles.entries[0].session);
            match operation {
                "select" => profiles.select(&path, 2).unwrap(),
                "create" => {
                    profiles.create(&path, "New profile", 42).unwrap();
                }
                "rename" => profiles.rename(&path, 1, "Renamed").unwrap(),
                "delete" => profiles.delete(&path, 2).unwrap(),
                "score" => {
                    assert!(profiles.record_events(&path, &[result_event()]).unwrap());
                }
                _ => unreachable!(),
            }
            let reopened = Profiles::decode(&std::fs::read(&path).unwrap()).unwrap();
            assert_eq!(reopened.current_id, profiles.current_id);
            assert_eq!(reopened.entries.len(), profiles.entries.len());
            assert_eq!(reopened.entries[0].session.mode, GameMode::Adventure);
            assert!(reopened.entries[0].session.retained_time_trial.is_some());
            assert_eq!(format!("{:?}", profiles.entries[0].session), live_before);
            save_profiles_checkpoint(&options, &profiles).unwrap();
            Profiles::decode(&std::fs::read(&path).unwrap()).unwrap();
        }
    }

    #[test]
    fn profile_publication_projects_every_inactive_sandbox_entry() {
        let directory = FixtureDirectory::new("inactive-sandbox");
        let path = directory.0.join("profiles.json");
        let mut profiles = fixture(3, true, true);
        profiles.entries[1].session = profiles.entries[0].session.clone();
        profiles.current_id = Some(3);
        profiles.validate().unwrap();
        let before = format!("{profiles:?}");
        profiles.save(&path).unwrap();
        let reopened = Profiles::decode(&std::fs::read(&path).unwrap()).unwrap();
        assert!(
            reopened
                .entries
                .iter()
                .all(|entry| entry.session.mode == GameMode::Adventure)
        );
        assert_eq!(format!("{profiles:?}"), before);
    }

    #[test]
    fn exit_cleanup_skips_only_an_identical_successful_checkpoint() {
        for sandbox in [false, true] {
            let directory = FixtureDirectory::new("exit-checkpoint");
            let options = directory.options();
            let mut profiles = fixture(8, true, sandbox);
            profiles
                .current_mut()
                .unwrap()
                .session
                .virtual_saved_at_seconds = 10;
            save_profiles_checkpoint(&options, &profiles).unwrap();
            let before = std::fs::read(directory.0.join("profiles.json")).unwrap();
            assert!(!save_final_profiles_checkpoint(&options, &mut profiles, true, 10).unwrap());
            assert_eq!(
                std::fs::read(directory.0.join("profiles.json")).unwrap(),
                before
            );
            assert!(save_final_profiles_checkpoint(&options, &mut profiles, true, 11).unwrap());
            let reopened =
                Profiles::decode(&std::fs::read(directory.0.join("profiles.json")).unwrap())
                    .unwrap();
            assert_eq!(
                reopened.current().unwrap().session.virtual_saved_at_seconds,
                11
            );
            assert!(
                save_final_profiles_checkpoint(&options, &mut profiles, false, 11).unwrap(),
                "fallback exit always publishes"
            );
        }
        let directory = FixtureDirectory::new("empty-exit");
        let options = directory.options();
        let mut profiles = Profiles {
            current_id: None,
            entries: vec![],
            ..fixture(1, false, false)
        };
        assert!(save_final_profiles_checkpoint(&options, &mut profiles, false, 10).unwrap());
        assert!(!save_final_profiles_checkpoint(&options, &mut profiles, true, 11).unwrap());
        Profiles::decode(&std::fs::read(directory.0.join("profiles.json")).unwrap()).unwrap();
    }

    #[test]
    fn invalid_inactive_profile_preserves_previous_file_and_input_manager() {
        let directory = FixtureDirectory::new("invalid-profile");
        let options = directory.options();
        let mut profiles = fixture(8, true, false);
        save_profiles_checkpoint(&options, &profiles).unwrap();
        let previous = std::fs::read(directory.0.join("profiles.json")).unwrap();
        profiles.entries[7].id = 0;
        let memory_before = format!("{profiles:?}");
        assert!(save_profiles_checkpoint(&options, &profiles).is_err());
        assert_eq!(
            std::fs::read(directory.0.join("profiles.json")).unwrap(),
            previous
        );
        assert_eq!(format!("{profiles:?}"), memory_before);
        Profiles::decode(&previous).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn denied_publications_preserve_old_file_scores_and_final_timestamp() {
        use std::os::windows::fs::OpenOptionsExt;
        for sandbox in [false, true] {
            let directory = FixtureDirectory::new("denied-profile");
            let options = directory.options();
            let mut profiles = fixture(8, true, sandbox);
            save_profiles_checkpoint(&options, &profiles).unwrap();
            let path = directory.0.join("profiles.json");
            let previous = std::fs::read(&path).unwrap();
            let reader = std::fs::OpenOptions::new()
                .read(true)
                .share_mode(1 | 2)
                .open(&path)
                .unwrap();
            profiles
                .current_mut()
                .unwrap()
                .session
                .progress
                .shell_balance = 23;
            let memory_before = format!("{profiles:?}");
            assert!(profiles.select(&path, 2).is_err());
            assert_eq!(format!("{profiles:?}"), memory_before);
            assert_eq!(std::fs::read(&path).unwrap(), previous);
            assert!(save_profiles_checkpoint(&options, &profiles).is_err());
            assert_eq!(format!("{profiles:?}"), memory_before);
            assert!(profiles.record_events(&path, &[result_event()]).is_err());
            assert_eq!(format!("{profiles:?}"), memory_before);
            assert!(save_final_profiles_checkpoint(&options, &mut profiles, false, 123).is_err());
            assert_eq!(format!("{profiles:?}"), memory_before);
            assert_eq!(std::fs::read(&path).unwrap(), previous);
            Profiles::decode(&previous).unwrap();
            drop(reader);
            save_profiles_checkpoint(&options, &profiles).unwrap();
            assert!(!path.with_extension("pending").exists());
            Profiles::decode(&std::fs::read(&path).unwrap()).unwrap();
        }
    }

    #[test]
    #[ignore = "offline timing requires a serial coordinator lease"]
    fn measure_checkpoint_costs() {
        let label =
            env::var("TURBOFISH_CHECKPOINT_LABEL").unwrap_or_else(|_| "unidentified".into());
        for (name, profiles) in [
            ("single", fixture(1, false, false)),
            ("retained8", fixture(8, true, false)),
            ("sandbox8", fixture(8, true, true)),
        ] {
            let directory = FixtureDirectory::new(&format!("measure-{label}-{name}"));
            let options = directory.options();
            let expected = serde_json::to_vec_pretty(&legacy_checkpoint(&profiles)).unwrap();
            let mut copies_us = Vec::new();
            let mut publications_us = Vec::new();
            for _ in 0..9 {
                let started = Instant::now();
                for _ in 0..2_000 {
                    black_box(black_box(&profiles).checkpoint());
                }
                copies_us.push(started.elapsed().as_secs_f64() * 1e6 / 2_000.0);
                let started = Instant::now();
                for _ in 0..16 {
                    save_profiles_checkpoint(black_box(&options), black_box(&profiles)).unwrap();
                }
                publications_us.push(started.elapsed().as_secs_f64() * 1e6 / 16.0);
            }
            let bytes = std::fs::read(directory.0.join("profiles.json")).unwrap();
            assert_eq!(bytes, expected);
            Profiles::decode(&bytes).unwrap();
            let mut result_us = Vec::new();
            let mut result_sha256 = None;
            if name != "sandbox8" {
                for _ in 0..9 {
                    let mut sample_profiles: Vec<_> = (0..16).map(|_| profiles.clone()).collect();
                    let started = Instant::now();
                    for sample in &mut sample_profiles {
                        let score_published = sample
                            .record_events(&directory.0.join("profiles.json"), &[result_event()])
                            .unwrap();
                        assert!(score_published);
                        save_profiles_checkpoint_after_score(&options, sample, score_published)
                            .unwrap();
                    }
                    result_us.push(started.elapsed().as_secs_f64() * 1e6 / 16.0);
                }
                let result_bytes = std::fs::read(directory.0.join("profiles.json")).unwrap();
                Profiles::decode(&result_bytes).unwrap();
                let mut expected_result = legacy_checkpoint(&profiles);
                expected_result
                    .global_scores
                    .record(
                        crate::highscores::ScoreKind::TimeTrial,
                        1,
                        None,
                        "Fixture 1",
                        500,
                    )
                    .unwrap();
                assert_eq!(
                    result_bytes,
                    serde_json::to_vec_pretty(&expected_result).unwrap()
                );
                result_sha256 = Some(digest(&result_bytes));
            }
            println!(
                "{}",
                serde_json::json!({
                    "label":label, "fixture":name, "profile_count":profiles.entries.len(),
                    "seed":0xa294_u64, "fixture_sha256":digest(&serde_json::to_vec(&profiles).unwrap()),
                    "checkpoint_sha256":digest(&expected), "checkpoint_bytes":expected.len(),
                    "result_sha256":result_sha256,
                    "samples":9, "copy_iterations":2_000, "publication_iterations":16,
                    "copy_us":copies_us, "publication_us":publications_us, "result_us":result_us,
                })
            );
        }
    }
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

pub use crate::persistence::write_json;

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
