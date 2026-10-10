//! Project profiles own whole sessions. Progress, shells, unlocks and records
//! remain in `AdventureSession`; this layer only selects and persists owners.
//!
//! BR08/BR11 identify the retail manager, names and score pages. This is a
//! project save format, not an importer for retail or older project saves.

use crate::{
    adventure::AdventureSession,
    cli::{self, SAVE_FORMAT_VERSION},
};
use serde::{Deserialize, Serialize};
use std::{error::Error, path::Path};

pub const PROFILE_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileEntry {
    pub id: u64,
    pub name: String,
    pub session: AdventureSession,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profiles {
    pub format_version: u32,
    pub next_id: u64,
    pub current_id: Option<u64>,
    pub entries: Vec<ProfileEntry>,
}

fn valid_name(name: &str) -> bool {
    !name.trim().is_empty()
}

impl Profiles {
    pub fn new(name: &str, seed: u64) -> Result<Self, String> {
        Self::from_session(name, AdventureSession::new(seed))
    }

    /// Adopt an already loaded current project session as the first profile.
    /// The caller decides whether/when that current save is superseded.
    pub fn from_session(name: &str, session: AdventureSession) -> Result<Self, String> {
        if !valid_name(name) {
            return Err("profile name is empty".into());
        }
        session.validate()?;
        Ok(Self {
            format_version: PROFILE_FORMAT_VERSION,
            next_id: 2,
            current_id: Some(1),
            entries: vec![ProfileEntry {
                id: 1,
                name: name.trim().to_owned(),
                session,
            }],
        })
    }

    pub fn current(&self) -> Option<&ProfileEntry> {
        self.entries
            .iter()
            .find(|entry| Some(entry.id) == self.current_id)
    }

    /// Mutate the authoritative session, then call `save` at the existing
    /// event-driven save boundary. Profile-list operations use `transact`.
    pub fn current_mut(&mut self) -> Option<&mut ProfileEntry> {
        self.entries
            .iter_mut()
            .find(|entry| Some(entry.id) == self.current_id)
    }

    pub fn find(&self, name: &str) -> Option<&ProfileEntry> {
        self.entries
            .iter()
            .find(|entry| entry.name.to_lowercase() == name.to_lowercase())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.format_version != PROFILE_FORMAT_VERSION || self.next_id == 0 {
            return Err("unsupported profile format".into());
        }
        if self.current_id.is_some() == self.entries.is_empty()
            || self
                .current_id
                .is_some_and(|id| !self.entries.iter().any(|entry| entry.id == id))
        {
            return Err("current profile does not exist".into());
        }
        for (index, entry) in self.entries.iter().enumerate() {
            if entry.id == 0
                || entry.id >= self.next_id
                || !valid_name(&entry.name)
                || entry.name != entry.name.trim()
                || self.entries[..index].iter().any(|previous| {
                    previous.id == entry.id
                        || previous.name.to_lowercase() == entry.name.to_lowercase()
                })
            {
                return Err("invalid or duplicate profile".into());
            }
            entry.session.validate()?;
        }
        Ok(())
    }

    /// Current-only decoder. Each nested session goes through the existing
    /// current-format field guard before Serde can fill an omitted old field.
    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn Error>> {
        let value: serde_json::Value = serde_json::from_slice(bytes)?;
        if value.get("format_version").and_then(|v| v.as_u64())
            != Some(u64::from(PROFILE_FORMAT_VERSION))
        {
            return Err("unsupported profile format".into());
        }
        let entries = value
            .get("entries")
            .and_then(|v| v.as_array())
            .ok_or("profile entries missing")?;
        for entry in entries {
            let session = entry.get("session").ok_or("profile session missing")?;
            let current = serde_json::json!({
                "format_version": SAVE_FORMAT_VERSION,
                "session": session,
            });
            cli::decode_save(&serde_json::to_vec(&current)?)?;
        }
        let mut profiles: Self = serde_json::from_value(value)?;
        // A removed current user falls back to the first remaining profile.
        if !profiles
            .entries
            .iter()
            .any(|entry| Some(entry.id) == profiles.current_id)
        {
            profiles.current_id = profiles.entries.first().map(|entry| entry.id);
        }
        profiles.validate()?;
        Ok(profiles)
    }

    pub fn load_or_new(path: &Path, name: &str, seed: u64) -> Result<Self, Box<dyn Error>> {
        if path.exists() {
            Self::decode(&std::fs::read(path)?)
        } else {
            Ok(Self::new(name, seed)?)
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        self.validate()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        cli::write_json(path, self)
    }

    /// Commit profile metadata and its sessions together. A failed validation
    /// or disk write leaves the in-memory manager unchanged.
    pub fn transact<T>(
        &mut self,
        path: &Path,
        edit: impl FnOnce(&mut Self) -> Result<T, String>,
    ) -> Result<T, Box<dyn Error>> {
        let mut draft = self.clone();
        let result = edit(&mut draft)?;
        draft.save(path)?;
        *self = draft;
        Ok(result)
    }

    pub fn create(&mut self, path: &Path, name: &str, seed: u64) -> Result<u64, Box<dyn Error>> {
        self.transact(path, |draft| {
            let name = name.trim();
            if !valid_name(name) || draft.find(name).is_some() {
                return Err("profile name is empty or already used".into());
            }
            let id = draft.next_id;
            draft.next_id = id.checked_add(1).ok_or("profile IDs exhausted")?;
            draft.entries.push(ProfileEntry {
                id,
                name: name.to_owned(),
                session: AdventureSession::new(seed),
            });
            draft.current_id = Some(id);
            Ok(id)
        })
    }

    pub fn select(&mut self, path: &Path, id: u64) -> Result<(), Box<dyn Error>> {
        self.transact(path, |draft| {
            if !draft.entries.iter().any(|entry| entry.id == id) {
                return Err("profile does not exist".into());
            }
            draft.current_id = Some(id);
            Ok(())
        })
    }

    pub fn rename(&mut self, path: &Path, id: u64, name: &str) -> Result<(), Box<dyn Error>> {
        self.transact(path, |draft| {
            let name = name.trim();
            if !valid_name(name)
                || draft
                    .entries
                    .iter()
                    .any(|entry| entry.id != id && entry.name.to_lowercase() == name.to_lowercase())
            {
                return Err("profile name is empty or already used".into());
            }
            let entry = draft
                .entries
                .iter_mut()
                .find(|entry| entry.id == id)
                .ok_or("profile does not exist")?;
            entry.name = name.to_owned();
            Ok(())
        })
    }

    pub fn delete(&mut self, path: &Path, id: u64) -> Result<(), Box<dyn Error>> {
        self.transact(path, |draft| {
            let index = draft
                .entries
                .iter()
                .position(|entry| entry.id == id)
                .ok_or("profile does not exist")?;
            draft.entries.remove(index);
            if draft.current_id == Some(id) {
                draft.current_id = draft.entries.first().map(|entry| entry.id);
            }
            Ok(())
        })
    }
}

/// View-only high-score projection. No second mutable score ledger.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScoreTab {
    Adventure,
    TimeTrial,
    Challenge,
    Personal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScoreKind {
    Adventure,
    TimeTrial,
    Challenge,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScoreRow {
    pub kind: ScoreKind,
    pub tank: u8,
    pub level: Option<u8>,
    pub personal: Option<u64>,
    pub top_five: Vec<u64>,
}

impl Profiles {
    pub fn score_rows(&self, tab: ScoreTab) -> Vec<ScoreRow> {
        let Some(entry) = self.current() else {
            return Vec::new();
        };
        let session = &entry.session;
        let mut rows = Vec::new();
        if matches!(tab, ScoreTab::Adventure | ScoreTab::Personal) {
            let mut stages = std::collections::BTreeSet::new();
            for profile in &self.entries {
                if profile.session.progress.first_stage_best_seconds.is_some() {
                    stages.insert((1, 1));
                }
                for best in &profile.session.progress.later_stage_best_seconds {
                    stages.insert((best.tank, best.level));
                }
            }
            for (tank, level) in stages {
                let best_for = |session: &AdventureSession| {
                    if (tank, level) == (1, 1) {
                        session.progress.first_stage_best_seconds
                    } else {
                        session
                            .progress
                            .later_stage_best_seconds
                            .iter()
                            .find(|best| (best.tank, best.level) == (tank, level))
                            .map(|best| best.seconds)
                    }
                };
                let mut leaders: Vec<_> = self
                    .entries
                    .iter()
                    .filter_map(|p| best_for(&p.session))
                    .collect();
                leaders.sort_unstable();
                leaders.truncate(5);
                rows.push(ScoreRow {
                    kind: ScoreKind::Adventure,
                    tank,
                    level: Some(level),
                    personal: best_for(session),
                    top_five: leaders,
                });
            }
        }
        if matches!(tab, ScoreTab::TimeTrial | ScoreTab::Personal) {
            for index in 0..4 {
                let mut leaders: Vec<_> = self
                    .entries
                    .iter()
                    .flat_map(|p| p.session.time_trial_scores.top_five[index].iter())
                    .map(|score| *score as u64)
                    .collect();
                leaders.sort_unstable_by(|a, b| b.cmp(a));
                leaders.truncate(5);
                rows.push(ScoreRow {
                    kind: ScoreKind::TimeTrial,
                    tank: index as u8 + 1,
                    level: None,
                    personal: (session.time_trial_scores.personal_best[index] > 0)
                        .then_some(session.time_trial_scores.personal_best[index] as u64),
                    top_five: leaders,
                });
            }
        }
        if matches!(tab, ScoreTab::Challenge | ScoreTab::Personal) {
            for index in 0..4 {
                let mut leaders: Vec<_> = self
                    .entries
                    .iter()
                    .flat_map(|p| p.session.challenge_scores.top_five[index].iter())
                    .map(|score| *score as u64)
                    .collect();
                leaders.sort_unstable();
                leaders.truncate(5);
                rows.push(ScoreRow {
                    kind: ScoreKind::Challenge,
                    tank: index as u8 + 1,
                    level: None,
                    personal: session.challenge_scores.personal_best[index]
                        .map(|score| score as u64),
                    top_five: leaders,
                });
            }
        }
        rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_names_and_dangling_current_are_rejected_without_losing_session_state() {
        let mut profiles = Profiles::new("Alice", 7).unwrap();
        profiles.entries.push(ProfileEntry {
            id: 2,
            name: "alice".into(),
            session: AdventureSession::new(8),
        });
        profiles.next_id = 3;
        assert!(profiles.validate().is_err());
        profiles.entries[1].name = "Bob".into();
        profiles.current_id = Some(99);
        assert!(profiles.validate().is_err());
        profiles.current_id = Some(1);
        assert!(profiles.validate().is_ok());
        assert_eq!(profiles.entries[0].session.progress.shell_balance, 0);
        assert_eq!(profiles.entries[1].session.progress.shell_balance, 0);
    }

    #[test]
    fn current_decoder_rejects_missing_nested_session_field() {
        let profiles = Profiles::new("Alice", 7).unwrap();
        let mut value = serde_json::to_value(&profiles).unwrap();
        value["entries"][0]["session"]["progress"]
            .as_object_mut()
            .unwrap()
            .remove("purchase_cursor");
        assert!(Profiles::decode(&serde_json::to_vec(&value).unwrap()).is_err());
    }
}
