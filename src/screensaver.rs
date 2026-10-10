//! Disposable Virtual Tank screensaver preview and project-owned preferences.
//!
//! The preview never accepts gameplay input, saves a session, or yields a mutable
//! reference to its source. OS screensaver registration is outside this module.

use crate::{
    adventure::{AdventurePhase, AdventureSession, GameMode},
    sim::{Action, Event},
    virtual_tank::BACKDROP_COUNT,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, path::Path};

const SETTINGS_FORMAT_VERSION: u32 = 1;
const ROTATE_TICKS: u64 = 10_800;
const DIM_CYCLE_TICKS: u64 = 3_000;

/// These are the WinFishApp constructor defaults (WinFishApp.cpp:275-280).
/// There is deliberately no `enabled` field until an OS integration exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScreensaverSettings {
    pub sound: bool,
    pub rotate_backdrops: bool,
    pub periodic_dim: bool,
    pub show_money: bool,
    pub power_save: bool,
}

impl Default for ScreensaverSettings {
    fn default() -> Self {
        Self {
            sound: true,
            rotate_backdrops: true,
            periodic_dim: true,
            show_money: false,
            power_save: false,
        }
    }
}

/// Store this in a project-owned auxiliary path, separate from game saves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScreensaverSettingsFile {
    pub format_version: u32,
    pub settings: ScreensaverSettings,
}

impl Default for ScreensaverSettingsFile {
    fn default() -> Self {
        Self {
            format_version: SETTINGS_FORMAT_VERSION,
            settings: ScreensaverSettings::default(),
        }
    }
}

impl ScreensaverSettingsFile {
    pub fn load_or_default(path: &Path) -> Result<Self, Box<dyn Error>> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let stored: Self = serde_json::from_slice(&std::fs::read(path)?)?;
        if stored.format_version != SETTINGS_FORMAT_VERSION {
            return Err("unsupported screensaver settings format".into());
        }
        Ok(stored)
    }

    pub fn save(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        if self.format_version != SETTINGS_FORMAT_VERSION {
            return Err("unsupported screensaver settings format".into());
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        crate::cli::write_json(path, self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreviewError {
    UnsupportedSourcePhase,
    VirtualTankUnavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreviewOverlay {
    /// Black overlay opacity. The caller draws it over the tank, not the UI.
    pub dim_alpha: u8,
    /// Shells collected in this disposable preview, shown only when selected.
    pub shells_collected: Option<u32>,
    pub sound_enabled: bool,
}

/// Owns a clone of the chosen session; no mutation reaches the source session.
/// This type is intentionally neither Clone nor Serialize.
pub struct ScreensaverPreview {
    scene: AdventureSession,
    settings: ScreensaverSettings,
    initial_board_shells: u32,
    rotation_rng: u64,
}

impl ScreensaverPreview {
    pub fn start(
        source: &AdventureSession,
        settings: ScreensaverSettings,
        now_seconds: u64,
    ) -> Result<Self, PreviewError> {
        let mut scene = source.clone();
        match scene.phase {
            AdventurePhase::GameSelector => {
                let day_start_seconds = now_seconds / 86_400 * 86_400;
                // Entry is an action-only boundary. `step` also updates the
                // Board and could collect a pet coin before `advance` gets to
                // discard the cloned wallet credit.
                scene.apply_actions(&[Action::PlayVirtualTank {
                    now_seconds,
                    day_start_seconds,
                }]);
            }
            AdventurePhase::VirtualTank | AdventurePhase::VirtualSetup => {
                scene.mode = GameMode::VirtualTank;
                scene.phase = AdventurePhase::VirtualTank;
            }
            _ => return Err(PreviewError::UnsupportedSourcePhase),
        }
        if scene.phase != AdventurePhase::VirtualTank || scene.virtual_board.is_none() {
            return Err(PreviewError::VirtualTankUnavailable);
        }
        scene.virtual_saved_at_seconds = now_seconds;
        let initial_board_shells = scene.virtual_tank.board_shells;
        let rotation_rng = source.ticks ^ now_seconds.rotate_left(17) ^ 0x9e37_79b9_7f4a_7c15;
        Ok(Self {
            scene,
            settings,
            initial_board_shells,
            rotation_rng,
        })
    }

    /// The app renderer can display this scene but cannot step or save it.
    pub fn scene(&self) -> &AdventureSession {
        &self.scene
    }

    pub fn settings(&self) -> ScreensaverSettings {
        self.settings
    }

    pub fn set_settings(&mut self, settings: ScreensaverSettings) {
        self.settings = settings;
    }

    /// Step only the cloned VT scene. Pet coin collection may run in the normal
    /// Board path, so discard its temporary wallet credit while retaining the
    /// clone's Board shell counter (Board.cpp:2320-24 has a separate saver path).
    pub fn advance(&mut self, now_seconds: u64) -> Vec<Event> {
        let wallet = self.scene.progress.shell_balance;
        self.scene.virtual_saved_at_seconds = now_seconds;
        let mut events = self.scene.step(&[]);
        self.scene.progress.shell_balance = wallet;
        for event in &mut events {
            if let Event::VirtualCoinCollected { shell_balance, .. } = event {
                *shell_balance = wallet;
            }
        }
        self.rotate_backdrop_if_due();
        events
    }

    pub fn overlay(&self) -> PreviewOverlay {
        let tick = self
            .scene
            .virtual_board
            .as_ref()
            .map_or(0, |board| board.tick);
        PreviewOverlay {
            dim_alpha: if self.settings.periodic_dim {
                dim_alpha(tick)
            } else {
                0
            },
            shells_collected: self.settings.show_money.then(|| {
                self.scene
                    .virtual_tank
                    .board_shells
                    .saturating_sub(self.initial_board_shells)
            }),
            sound_enabled: self.settings.sound,
        }
    }

    fn rotate_backdrop_if_due(&mut self) {
        let Some(board) = self.scene.virtual_board.as_ref() else {
            return;
        };
        if !self.settings.rotate_backdrops || board.tick == 0 || board.tick % ROTATE_TICKS != 0 {
            return;
        }
        let tank = &mut self.scene.virtual_tank;
        let mut candidates = [0u8; BACKDROP_COUNT];
        let mut count = 0;
        for (index, owned) in tank.owned_backdrops.iter().enumerate() {
            if *owned && index as u8 != tank.backdrop {
                candidates[count] = index as u8;
                count += 1;
            }
        }
        if count > 0 {
            // Source Board::Update chooses a random owned alternative. This
            // private preview RNG never consumes the live session's RNG.
            self.rotation_rng = self
                .rotation_rng
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            tank.backdrop = candidates[(self.rotation_rng as usize) % count];
        }
    }
}

/// Board.cpp:5625-41: 3000-tick cycle, 1320..1680 dim window, 30-tick
/// linear ramps and alpha 200 plateau. The integer interpolation is a local
/// projection of the source helper's rounding.
fn dim_alpha(tick: u64) -> u8 {
    match tick % DIM_CYCLE_TICKS {
        1320..=1350 => (((tick % DIM_CYCLE_TICKS) - 1320) * 200 / 30) as u8,
        1351..=1649 => 200,
        1650..=1680 => ((1680 - (tick % DIM_CYCLE_TICKS)) * 200 / 30) as u8,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        sim::PetKind,
        virtual_board::VirtualCoinPose,
        virtual_tank::{
            FishBiography, FishOffer, FishProperties, FishSize, PurchaseConfirmation, STORE_SLOTS,
            StoreProduct, VirtualCoinKind, VirtualFishKind,
        },
    };

    #[test]
    fn preview_moves_owned_fish_and_collects_without_changing_the_live_selector() {
        // Prepared, validated progress snapshot, not a claim of earned play.
        let mut live = AdventureSession::new(0x5151);
        live.board = None;
        live.phase = AdventurePhase::GameSelector;
        live.progress.tank = 1;
        live.progress.level = 2;
        live.progress.first_stage_best_seconds = Some(90);
        live.progress.unlocked_pets.push(PetKind::Stinky);
        live.progress.shell_balance = 300;
        live.virtual_tank.selected_pets.push(PetKind::Stinky);

        let offer = FishOffer {
            kind: VirtualFishKind::Guppy,
            properties: FishProperties::default(),
            initial_hunger: 500,
            initial_size: FishSize::Small,
            food_needed_to_grow: 12,
            coin_threshold: 4_320,
        };
        let price = offer.price();
        let mut offers: [Option<StoreProduct>; STORE_SLOTS] = std::array::from_fn(|_| None);
        offers[0] = Some(StoreProduct::Fish(offer));
        live.virtual_tank.install_stock(86_400, offers).unwrap();
        live.virtual_tank
            .confirm_purchase(
                &mut live.progress.shell_balance,
                PurchaseConfirmation {
                    cycle: 86_400,
                    slot: 0,
                    expected_price: price,
                    accepted: true,
                    fish_name: Some("Keel".into()),
                    fish_biography: Some(FishBiography {
                        hometown: 1,
                        likes: [0, 1, 2],
                    }),
                },
                86_420,
                86_400,
            )
            .unwrap();
        live.apply_actions(&[Action::PlayVirtualTank {
            now_seconds: 86_420,
            day_start_seconds: 86_400,
        }]);
        let board = live.virtual_board.as_mut().unwrap();
        let coin_id = board.next_object_id;
        board.next_object_id += 1;
        board.coins.push(VirtualCoinPose {
            id: coin_id,
            x: 90.0,
            y: 410.0,
            kind: VirtualCoinKind::Gold,
            niko_owner: None,
            age: 0,
        });
        live.apply_actions(&[Action::OpenMenu]);
        live.validate().unwrap();
        let live_bytes = serde_json::to_vec(&live).unwrap();
        let live_wallet = live.progress.shell_balance;
        let live_board_shells = live.virtual_tank.board_shells;

        let mut preview =
            ScreensaverPreview::start(&live, ScreensaverSettings::default(), 86_420).unwrap();
        let initial_x = preview.scene().virtual_board.as_ref().unwrap().poses[0].x;
        let events = preview.advance(86_421);
        assert!(events.iter().any(|event| matches!(
            event,
            Event::VirtualCoinCollected {
                kind: VirtualCoinKind::Gold,
                by_pet: true,
                ..
            }
        )));
        assert_ne!(
            preview.scene().virtual_board.as_ref().unwrap().poses[0].x,
            initial_x
        );
        assert_eq!(preview.scene().virtual_tank.fish[0].name, "Keel");
        assert!(preview.scene().virtual_tank.board_shells > live_board_shells);
        assert_eq!(preview.scene().progress.shell_balance, live_wallet);
        assert_eq!(serde_json::to_vec(&live).unwrap(), live_bytes);
        live.validate().unwrap();
    }
}
