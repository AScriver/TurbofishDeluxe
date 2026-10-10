//! Project profiles own whole sessions. Progress, shells, unlocks and records
//! remain in `AdventureSession`; this layer only selects and persists owners.
//!
//! BR08/BR11 identify the retail manager, names and score pages. This is a
//! project save format, not an importer for retail or older project saves.

use crate::{
    adventure::AdventureSession,
    cli::{self, SAVE_FORMAT_VERSION},
    highscores::{GlobalScores, ScoreEntry, ScoreKind as GlobalScoreKind},
    sim::Event,
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
    pub global_scores: GlobalScores,
}

fn valid_name(name: &str) -> bool {
    !name.trim().is_empty() && name.len() <= 256 && !name.chars().any(char::is_control)
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
            global_scores: GlobalScores::default(),
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
        self.global_scores.validate()?;
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

    /// Record a completed run once at the same atomic save boundary as its
    /// authoritative session. Names are copied at achievement time.
    pub fn record_events(&mut self, path: &Path, events: &[Event]) -> Result<bool, Box<dyn Error>> {
        let Some(current) = self.current() else {
            return Ok(false);
        };
        let name = current.name.clone();
        let mut draft = self.clone();
        let mut changed = false;
        for event in events {
            let target = match event {
                Event::StageResultRecorded {
                    tank,
                    level,
                    seconds,
                    survivors,
                    ..
                } => {
                    let score = if *tank == 5 {
                        survivors.and_then(|count| i32::try_from(count).ok())
                    } else {
                        i32::try_from(*seconds).ok()
                    };
                    score.map(|score| (GlobalScoreKind::Adventure, *tank, Some(*level), score))
                }
                Event::TimeTrialExpired { tank, score, .. } => {
                    Some((GlobalScoreKind::TimeTrial, *tank, None, *score))
                }
                Event::ChallengeCompleted { tank, seconds, .. } => {
                    Some((GlobalScoreKind::Challenge, *tank, None, *seconds))
                }
                _ => None,
            };
            if let Some((kind, tank, level, score)) = target {
                changed |= draft
                    .global_scores
                    .record(kind, tank, level, &name, score)?;
            }
        }
        if changed {
            draft.save(path)?;
            *self = draft;
        }
        Ok(changed)
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
    pub global: Vec<ScoreEntry>,
}

impl Profiles {
    pub fn score_rows(&self, tab: ScoreTab) -> Vec<ScoreRow> {
        let Some(entry) = self.current() else {
            return Vec::new();
        };
        let session = &entry.session;
        let mut rows = Vec::new();
        if matches!(tab, ScoreTab::Adventure | ScoreTab::Personal) {
            for tank in 1..=5 {
                for level in 1..=if tank == 5 { 1 } else { 5 } {
                    let personal = if tank == 5 {
                        session.progress.tank5_best_survivors.map(u64::from)
                    } else if (tank, level) == (1, 1) {
                        session.progress.first_stage_best_seconds
                    } else {
                        session
                            .progress
                            .later_stage_best_seconds
                            .iter()
                            .find(|best| (best.tank, best.level) == (tank, level))
                            .map(|best| best.seconds)
                    };
                    rows.push(ScoreRow {
                        kind: ScoreKind::Adventure,
                        tank,
                        level: Some(level),
                        personal,
                        global: self
                            .global_scores
                            .rows(GlobalScoreKind::Adventure, tank, Some(level))
                            .unwrap_or_default()
                            .to_vec(),
                    });
                }
            }
        }
        if matches!(tab, ScoreTab::TimeTrial | ScoreTab::Personal) {
            for index in 0..4 {
                rows.push(ScoreRow {
                    kind: ScoreKind::TimeTrial,
                    tank: index as u8 + 1,
                    level: None,
                    personal: (session.time_trial_scores.personal_best[index] > 0)
                        .then_some(session.time_trial_scores.personal_best[index] as u64),
                    global: self
                        .global_scores
                        .rows(GlobalScoreKind::TimeTrial, index as u8 + 1, None)
                        .unwrap_or_default()
                        .to_vec(),
                });
            }
        }
        if matches!(tab, ScoreTab::Challenge | ScoreTab::Personal) {
            for index in 0..4 {
                rows.push(ScoreRow {
                    kind: ScoreKind::Challenge,
                    tank: index as u8 + 1,
                    level: None,
                    personal: session.challenge_scores.personal_best[index]
                        .map(|score| score as u64),
                    global: self
                        .global_scores
                        .rows(GlobalScoreKind::Challenge, index as u8 + 1, None)
                        .unwrap_or_default()
                        .to_vec(),
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

        let directory = std::env::temp_dir().join(format!(
            "turbofish-profiles-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        ));
        let path = directory.join("profiles.json");
        let mut saved = Profiles::new("Alice", 7).unwrap();
        saved
            .record_events(
                &path,
                &[Event::StageResultRecorded {
                    tick: 1,
                    tank: 1,
                    level: 1,
                    seconds: 42,
                    settled_coin_ids: vec![],
                    settled_amount: 0,
                    final_balance: 0,
                    personal_best_seconds: 42,
                    survivors: None,
                }],
            )
            .unwrap();
        let bob = saved.create(&path, "Bob", 8).unwrap();
        saved.current_mut().unwrap().session.progress.shell_balance = 77;
        saved.save(&path).unwrap();
        saved.select(&path, 1).unwrap();
        saved.rename(&path, 1, "Renamed").unwrap();
        saved.delete(&path, 1).unwrap();
        let loaded = Profiles::decode(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(loaded.current_id, Some(bob));
        assert_eq!(loaded.current().unwrap().session.progress.shell_balance, 77);
        assert_eq!(
            loaded
                .global_scores
                .rows(GlobalScoreKind::Adventure, 1, Some(1))
                .unwrap()[0]
                .name,
            "Alice"
        );
        std::fs::remove_file(&path).unwrap();
        std::fs::remove_dir(&directory).unwrap();
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
