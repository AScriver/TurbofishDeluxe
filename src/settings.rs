//! Shared options state and explicit backend application. BR08 identifies two
//! volume sliders and fullscreen/custom-cursor controls; hardware acceleration
//! is omitted until this renderer has an actual switch for it.

use serde::{Deserialize, Serialize};
use std::{error::Error, path::Path};

pub const SETTINGS_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameSettings {
    pub music_volume: f32,
    pub effects_volume: f32,
    pub fullscreen: bool,
    pub custom_cursor: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            music_volume: 1.0,
            effects_volume: 1.0,
            fullscreen: false,
            custom_cursor: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettingsCapabilities {
    pub music_volume: bool,
    pub effects_volume: bool,
    pub fullscreen: bool,
    pub custom_cursor: bool,
}

pub trait SettingsBackend {
    fn capabilities(&self) -> SettingsCapabilities;
    fn set_music_volume(&mut self, value: f32) -> Result<(), String>;
    fn set_effects_volume(&mut self, value: f32) -> Result<(), String>;
    fn set_fullscreen(&mut self, value: bool) -> Result<(), String>;
    fn set_custom_cursor(&mut self, value: bool) -> Result<(), String>;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SettingChange {
    MusicVolume(f32),
    EffectsVolume(f32),
    Fullscreen(bool),
    CustomCursor(bool),
}

impl GameSettings {
    pub fn validate(&self) -> Result<(), String> {
        if !self.music_volume.is_finite()
            || !(0.0..=1.0).contains(&self.music_volume)
            || !self.effects_volume.is_finite()
            || !(0.0..=1.0).contains(&self.effects_volume)
        {
            return Err("option volume must be between zero and one".into());
        }
        Ok(())
    }

    /// A control changes only when the live backend accepts it. Unsupported
    /// controls can be shown disabled; no hardware behavior is simulated.
    pub fn change(
        &mut self,
        backend: &mut impl SettingsBackend,
        change: SettingChange,
    ) -> Result<(), String> {
        let mut candidate = *self;
        match change {
            SettingChange::MusicVolume(value) => candidate.music_volume = value,
            SettingChange::EffectsVolume(value) => candidate.effects_volume = value,
            SettingChange::Fullscreen(value) => candidate.fullscreen = value,
            SettingChange::CustomCursor(value) => candidate.custom_cursor = value,
        }
        candidate.validate()?;
        let capabilities = backend.capabilities();
        match change {
            SettingChange::MusicVolume(value) if capabilities.music_volume => {
                backend.set_music_volume(value)?
            }
            SettingChange::EffectsVolume(value) if capabilities.effects_volume => {
                backend.set_effects_volume(value)?
            }
            SettingChange::Fullscreen(value) if capabilities.fullscreen => {
                backend.set_fullscreen(value)?
            }
            SettingChange::CustomCursor(value) if capabilities.custom_cursor => {
                backend.set_custom_cursor(value)?
            }
            _ => return Err("option is unsupported by this runtime".into()),
        }
        *self = candidate;
        Ok(())
    }

    /// Reapply saved values when a backend starts. Capability mismatches are
    /// explicit errors so UI cannot claim an option took effect when it did not.
    pub fn apply(&self, backend: &mut impl SettingsBackend) -> Result<(), String> {
        self.validate()?;
        let capabilities = backend.capabilities();
        let default = Self::default();
        if (!capabilities.music_volume && self.music_volume != default.music_volume)
            || (!capabilities.effects_volume && self.effects_volume != default.effects_volume)
            || (!capabilities.fullscreen && self.fullscreen != default.fullscreen)
            || (!capabilities.custom_cursor && self.custom_cursor != default.custom_cursor)
        {
            return Err("saved options exceed runtime capabilities".into());
        }
        if capabilities.music_volume {
            backend.set_music_volume(self.music_volume)?;
        }
        if capabilities.effects_volume {
            backend.set_effects_volume(self.effects_volume)?;
        }
        if capabilities.fullscreen {
            backend.set_fullscreen(self.fullscreen)?;
        }
        if capabilities.custom_cursor {
            backend.set_custom_cursor(self.custom_cursor)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettingsSave {
    pub format_version: u32,
    pub settings: GameSettings,
}

impl SettingsSave {
    pub fn load_or_default(path: &Path) -> Result<Self, Box<dyn Error>> {
        if !path.exists() {
            return Ok(Self {
                format_version: SETTINGS_FORMAT_VERSION,
                settings: GameSettings::default(),
            });
        }
        let saved: Self = serde_json::from_slice(&std::fs::read(path)?)?;
        if saved.format_version != SETTINGS_FORMAT_VERSION {
            return Err("unsupported settings format".into());
        }
        saved.settings.validate()?;
        Ok(saved)
    }

    pub fn save(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        if self.format_version != SETTINGS_FORMAT_VERSION {
            return Err("unsupported settings format".into());
        }
        self.settings.validate()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        crate::cli::write_json(path, self)
    }
}
