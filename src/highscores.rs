//! Bounded global score tables with copied player names.
//!
//! Ordering and capacities adapt pinned W1 `HighScoreMgr.cpp` as secondary
//! source. These entries belong in the project's atomic Profiles envelope;
//! retail `highscores.dat` defaults/import are intentionally separate.

use serde::{Deserialize, Serialize};
use std::array;

const ADVENTURE_TABLES: usize = 21;
const MODE_TABLES: usize = 4;
const MAX_NAME_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScoreKind {
    Adventure,
    TimeTrial,
    Challenge,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScoreEntry {
    pub name: String,
    pub score: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GlobalScores {
    adventure: [Vec<ScoreEntry>; ADVENTURE_TABLES],
    time_trial: [Vec<ScoreEntry>; MODE_TABLES],
    challenge: [Vec<ScoreEntry>; MODE_TABLES],
}

impl Default for GlobalScores {
    fn default() -> Self {
        Self {
            adventure: array::from_fn(|_| Vec::new()),
            time_trial: array::from_fn(|_| Vec::new()),
            challenge: array::from_fn(|_| Vec::new()),
        }
    }
}

impl GlobalScores {
    /// `level` is required only for Adventure. A `true` result means the
    /// copied-name row entered a table; full tables reject nonqualifying ties.
    pub fn record(
        &mut self,
        kind: ScoreKind,
        tank: u8,
        level: Option<u8>,
        name: &str,
        score: i32,
    ) -> Result<bool, String> {
        valid_entry(name, score)?;
        self.validate()?;
        let (rows, limit, descending) = self.table_mut(kind, tank, level)?;
        let position = rows
            .iter()
            .position(|entry| {
                if descending {
                    score > entry.score
                } else {
                    score < entry.score
                }
            })
            .unwrap_or(rows.len());
        if position == rows.len() && rows.len() >= limit {
            return Ok(false);
        }
        rows.insert(
            position,
            ScoreEntry {
                name: name.to_owned(),
                score,
            },
        );
        rows.truncate(limit);
        Ok(true)
    }

    pub fn rows(
        &self,
        kind: ScoreKind,
        tank: u8,
        level: Option<u8>,
    ) -> Result<&[ScoreEntry], String> {
        let index = table_index(kind, tank, level)?;
        Ok(match kind {
            ScoreKind::Adventure => self.adventure[index].as_slice(),
            ScoreKind::TimeTrial => self.time_trial[index].as_slice(),
            ScoreKind::Challenge => self.challenge[index].as_slice(),
        })
    }

    /// Validate after serde decode and before the enclosing atomic save.
    pub fn validate(&self) -> Result<(), String> {
        for (index, rows) in self.adventure.iter().enumerate() {
            validate_table(rows, if index == 20 { 3 } else { 1 }, index == 20)?;
        }
        for rows in &self.time_trial {
            validate_table(rows, 5, true)?;
        }
        for rows in &self.challenge {
            validate_table(rows, 5, false)?;
        }
        Ok(())
    }

    fn table_mut(
        &mut self,
        kind: ScoreKind,
        tank: u8,
        level: Option<u8>,
    ) -> Result<(&mut Vec<ScoreEntry>, usize, bool), String> {
        let index = table_index(kind, tank, level)?;
        Ok(match kind {
            ScoreKind::Adventure if tank == 5 => (&mut self.adventure[index], 3, true),
            ScoreKind::Adventure => (&mut self.adventure[index], 1, false),
            ScoreKind::TimeTrial => (&mut self.time_trial[index], 5, true),
            ScoreKind::Challenge => (&mut self.challenge[index], 5, false),
        })
    }
}

fn table_index(kind: ScoreKind, tank: u8, level: Option<u8>) -> Result<usize, String> {
    match kind {
        ScoreKind::Adventure if (1..=4).contains(&tank) => {
            let level = level.ok_or("Adventure score needs level")?;
            if !(1..=5).contains(&level) {
                return Err("invalid Adventure level".into());
            }
            Ok(usize::from(tank - 1) * 5 + usize::from(level - 1))
        }
        ScoreKind::Adventure if tank == 5 && level == Some(1) => Ok(20),
        ScoreKind::Adventure => Err("invalid Adventure stage".into()),
        ScoreKind::TimeTrial | ScoreKind::Challenge
            if (1..=4).contains(&tank) && level.is_none() =>
        {
            Ok(usize::from(tank - 1))
        }
        ScoreKind::TimeTrial | ScoreKind::Challenge => {
            Err("mode score needs tank 1..4 and no level".into())
        }
    }
}

fn valid_entry(name: &str, score: i32) -> Result<(), String> {
    if name.is_empty()
        || name.trim() != name
        || name.len() > MAX_NAME_BYTES
        || name.chars().any(char::is_control)
    {
        return Err("invalid score name".into());
    }
    if score < 0 {
        return Err("negative score".into());
    }
    Ok(())
}

fn validate_table(rows: &[ScoreEntry], limit: usize, descending: bool) -> Result<(), String> {
    if rows.len() > limit {
        return Err("score table exceeds capacity".into());
    }
    for (index, entry) in rows.iter().enumerate() {
        valid_entry(&entry.name, entry.score)?;
        if index > 0 {
            let previous = rows[index - 1].score;
            if (descending && previous < entry.score) || (!descending && previous > entry.score) {
                return Err("score table order is invalid".into());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copied_names_stable_ties_and_caps_survive_roundtrip() {
        let mut scores = GlobalScores::default();
        let mut owner = String::from("Original");
        for (name, value) in [
            (owner.as_str(), 100),
            ("Second", 100),
            ("Third", 90),
            ("Fourth", 80),
            ("Fifth", 70),
            ("Tie", 100),
        ] {
            assert!(
                scores
                    .record(ScoreKind::TimeTrial, 1, None, name, value)
                    .unwrap()
            );
        }
        owner = "Renamed".into();
        assert_eq!(owner, "Renamed");
        let rows = scores.rows(ScoreKind::TimeTrial, 1, None).unwrap();
        assert_eq!(
            rows.iter().map(|row| row.name.as_str()).collect::<Vec<_>>(),
            ["Original", "Second", "Tie", "Third", "Fourth"]
        );
        assert!(
            !scores
                .record(ScoreKind::TimeTrial, 1, None, "Late", 70)
                .unwrap()
        );
        assert!(
            scores
                .record(ScoreKind::Adventure, 1, Some(1), "Slow", 200)
                .unwrap()
        );
        assert!(
            !scores
                .record(ScoreKind::Adventure, 1, Some(1), "Equal", 200)
                .unwrap()
        );
        assert!(
            scores
                .record(ScoreKind::Adventure, 1, Some(1), "Fast", 100)
                .unwrap()
        );
        assert!(
            scores
                .record(ScoreKind::Adventure, 5, Some(1), "Survivor", 17)
                .unwrap()
        );
        scores.validate().unwrap();
        let decoded: GlobalScores =
            serde_json::from_slice(&serde_json::to_vec(&scores).unwrap()).unwrap();
        decoded.validate().unwrap();
        assert_eq!(decoded, scores);
    }
}
