//! Explicit fresh-ledger presets informed by pinned W1 HighScoreMgr.
//! Numeric rules are secondary-source-derived. Names are caller-supplied or
//! read from an exactly identified owned executable, never bundled here.

use super::{GlobalScores, ScoreKind, valid_entry};
use sha2::{Digest, Sha256};
use std::{fmt, fs::File, io, io::Read, path::Path};

const OWNED_EXE_BYTES: usize = 3_787_832;
const OWNED_EXE_SHA256: &str = "f54c2c6ee54b00ae6da7f4bde15feb90d0a867aba6b4151b0277c1f81da3ff66";

// Read-only file offsets for the identified English Steam executable. These
// point to strings, not code or retail score/save serialization structures.
const ADVENTURE_NAME_OFFSETS: [usize; 20] = [
    3300138, 3300130, 3300122, 3300110, 3300098, 3300090, 3300078, 3300070, 3300062, 3300050,
    3300042, 3300030, 3300022, 3300010, 3300002, 3299994, 3299986, 3299978, 3299966, 3299954,
];
const TIME_TRIAL_NAME_OFFSETS: [usize; 20] = [
    3300470, 3300466, 3300458, 3300450, 3300442, 3300434, 3300426, 3300422, 3300414, 3300406,
    3300398, 3300390, 3300382, 3300374, 3300366, 3300358, 3300350, 3300342, 3300334, 3300326,
];
const CHALLENGE_NAME_OFFSETS: [usize; 20] = [
    3300318, 3300310, 3300302, 3293138, 3300294, 3300282, 3300274, 3300266, 3300258, 3300246,
    3300238, 3300230, 3300214, 3300202, 3300194, 3300182, 3300174, 3300166, 3300158, 3300150,
];
const PRESET_SECONDS: [i32; 5] = [3300, 3360, 3420, 3480, 3540];
const PRESET_MONEY: [i32; 5] = [1000, 900, 800, 700, 600];

/// Each ordinary Adventure row is a stage; mode rows are ranks within a tank.
/// Public fields also permit deliberately project-authored labels/fixtures.
/// This is not profile data and has no implicit persistence or admission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresetNames {
    pub adventure: [[String; 5]; 4],
    pub time_trial: [[String; 5]; 4],
    pub challenge: [[String; 5]; 4],
}

#[derive(Debug)]
pub enum PresetError {
    Io(io::Error),
    CatalogUnavailable,
    InvalidName(String),
    InvalidLedger(String),
    LedgerNotEmpty,
}

impl fmt::Display for PresetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "Owned score catalog: {error}"),
            Self::CatalogUnavailable => {
                formatter.write_str("Score presets unavailable for this owned executable build")
            }
            Self::InvalidName(error) => write!(formatter, "Invalid preset name: {error}"),
            Self::InvalidLedger(error) => write!(formatter, "Invalid score ledger: {error}"),
            Self::LedgerNotEmpty => {
                formatter.write_str("Score presets require an entirely empty fresh ledger")
            }
        }
    }
}

impl std::error::Error for PresetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl PresetNames {
    /// Bounded, read-only static catalog access; never launches the game.
    /// Unknown identity returns unavailable, without guessing or fallback names.
    pub fn load_owned(executable: &Path) -> Result<Self, PresetError> {
        let file = File::open(executable).map_err(PresetError::Io)?;
        if file.metadata().map_err(PresetError::Io)?.len() != OWNED_EXE_BYTES as u64 {
            return Err(PresetError::CatalogUnavailable);
        }
        let mut bytes = Vec::with_capacity(OWNED_EXE_BYTES);
        file.take(OWNED_EXE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(PresetError::Io)?;
        Self::decode_owned(&bytes)
    }

    fn decode_owned(bytes: &[u8]) -> Result<Self, PresetError> {
        if bytes.len() != OWNED_EXE_BYTES
            || Sha256::digest(bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
                != OWNED_EXE_SHA256
        {
            return Err(PresetError::CatalogUnavailable);
        }
        Ok(Self {
            adventure: read_names(bytes, &ADVENTURE_NAME_OFFSETS)?,
            time_trial: read_names(bytes, &TIME_TRIAL_NAME_OFFSETS)?,
            challenge: read_names(bytes, &CHALLENGE_NAME_OFFSETS)?,
        })
    }

    fn validate(&self) -> Result<(), PresetError> {
        for name in self
            .adventure
            .iter()
            .chain(&self.time_trial)
            .chain(&self.challenge)
            .flatten()
        {
            valid_entry(name, 0).map_err(PresetError::InvalidName)?;
        }
        Ok(())
    }
}

fn read_names(bytes: &[u8], offsets: &[usize; 20]) -> Result<[[String; 5]; 4], PresetError> {
    let mut names = std::array::from_fn(|_| std::array::from_fn(|_| String::new()));
    for (index, &offset) in offsets.iter().enumerate() {
        let tail = bytes.get(offset..).ok_or(PresetError::CatalogUnavailable)?;
        let end = tail
            .iter()
            .take(super::MAX_NAME_BYTES + 1)
            .position(|&byte| byte == 0)
            .ok_or(PresetError::CatalogUnavailable)?;
        let name =
            std::str::from_utf8(&tail[..end]).map_err(|_| PresetError::CatalogUnavailable)?;
        valid_entry(name, 0).map_err(|_| PresetError::CatalogUnavailable)?;
        names[index / 5][index % 5] = name.to_owned();
    }
    Ok(names)
}

/// Construct a new ledger explicitly. W1 seeds no Tank5 rows: capacity three
/// belongs to earned survivor scores, not invented default survivors.
pub fn fresh(names: &PresetNames) -> Result<GlobalScores, PresetError> {
    names.validate()?;
    let mut scores = GlobalScores::default();
    for tank in 1..=4 {
        let tank_index = usize::from(tank - 1);
        for row in 0..5 {
            for (kind, level, name, score) in [
                (
                    ScoreKind::Adventure,
                    Some(row as u8 + 1),
                    &names.adventure[tank_index][row],
                    PRESET_SECONDS[row],
                ),
                (
                    ScoreKind::TimeTrial,
                    None,
                    &names.time_trial[tank_index][row],
                    PRESET_MONEY[row],
                ),
                (
                    ScoreKind::Challenge,
                    None,
                    &names.challenge[tank_index][row],
                    PRESET_SECONDS[row],
                ),
            ] {
                scores
                    .record(kind, tank, level, name, score)
                    .map_err(PresetError::InvalidLedger)?;
            }
        }
    }
    Ok(scores)
}

/// Atomic in-memory admission, with no file/profile access. Every existing row,
/// even in an otherwise empty ledger, blocks seeding. The caller decides when a
/// genuinely fresh manager should receive/persist the candidate.
pub fn admit_fresh(scores: &mut GlobalScores, names: &PresetNames) -> Result<(), PresetError> {
    scores.validate().map_err(PresetError::InvalidLedger)?;
    if scores
        .adventure
        .iter()
        .chain(&scores.time_trial)
        .chain(&scores.challenge)
        .any(|rows| !rows.is_empty())
    {
        return Err(PresetError::LedgerNotEmpty);
    }
    let candidate = fresh(names)?;
    *scores = candidate;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names() -> PresetNames {
        PresetNames {
            adventure: std::array::from_fn(|tank| {
                std::array::from_fn(|level| format!("Adventure {tank} {level}"))
            }),
            time_trial: std::array::from_fn(|tank| {
                std::array::from_fn(|rank| format!("Money {tank} {rank}"))
            }),
            challenge: std::array::from_fn(|tank| {
                std::array::from_fn(|rank| format!("Challenge {tank} {rank}"))
            }),
        }
    }

    #[test]
    fn explicit_presets_keep_units_caps_and_existing_tie_policy() {
        let mut scores = fresh(&names()).unwrap();
        for tank in 1..=4 {
            for (level, seconds) in [3300, 3360, 3420, 3480, 3540].into_iter().enumerate() {
                let rows = scores
                    .rows(ScoreKind::Adventure, tank, Some(level as u8 + 1))
                    .unwrap();
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].score, seconds);
            }
            for (kind, values) in [
                (ScoreKind::TimeTrial, [1000, 900, 800, 700, 600]),
                (ScoreKind::Challenge, [3300, 3360, 3420, 3480, 3540]),
            ] {
                assert_eq!(
                    scores
                        .rows(kind, tank, None)
                        .unwrap()
                        .iter()
                        .map(|row| row.score)
                        .collect::<Vec<_>>(),
                    values
                );
            }
        }
        assert!(
            scores
                .rows(ScoreKind::Adventure, 5, Some(1))
                .unwrap()
                .is_empty()
        );
        assert!(
            !scores
                .record(ScoreKind::Adventure, 1, Some(1), "Equal", 3300)
                .unwrap()
        );
        assert!(
            scores
                .record(ScoreKind::Adventure, 1, Some(1), "Fast", 3299)
                .unwrap()
        );
        assert!(
            !scores
                .record(ScoreKind::TimeTrial, 1, None, "Late", 600)
                .unwrap()
        );
        assert!(
            scores
                .record(ScoreKind::TimeTrial, 1, None, "Tied", 900)
                .unwrap()
        );
        let rows = scores.rows(ScoreKind::TimeTrial, 1, None).unwrap();
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[1].name, "Money 0 1");
        assert_eq!(rows[2].name, "Tied");
        assert!(
            !scores
                .record(ScoreKind::Challenge, 1, None, "Late", 3540)
                .unwrap()
        );
        for value in [16, 17, 18] {
            assert!(
                scores
                    .record(ScoreKind::Adventure, 5, Some(1), "Saved", value)
                    .unwrap()
            );
        }
        assert!(
            !scores
                .record(ScoreKind::Adventure, 5, Some(1), "Equal", 16)
                .unwrap()
        );
        assert_eq!(
            scores.rows(ScoreKind::Adventure, 5, Some(1)).unwrap()[0].score,
            18
        );
        scores.validate().unwrap();
    }

    #[test]
    fn admission_is_atomic_and_never_backfills_earned_rows() {
        let empty = GlobalScores::default();
        let mut scores = empty.clone();
        let mut invalid = names();
        invalid.challenge[3][4] = "bad\nname".into();
        assert!(matches!(
            admit_fresh(&mut scores, &invalid),
            Err(PresetError::InvalidName(_))
        ));
        assert_eq!(scores, empty);
        scores
            .record(ScoreKind::Adventure, 5, Some(1), "Earned", 0)
            .unwrap();
        let earned = scores.clone();
        assert!(matches!(
            admit_fresh(&mut scores, &names()),
            Err(PresetError::LedgerNotEmpty)
        ));
        assert_eq!(scores, earned);
        let mut fresh_scores = GlobalScores::default();
        admit_fresh(&mut fresh_scores, &names()).unwrap();
        let admitted = fresh_scores.clone();
        assert!(matches!(
            admit_fresh(&mut fresh_scores, &names()),
            Err(PresetError::LedgerNotEmpty)
        ));
        assert_eq!(fresh_scores, admitted);
        let encoded = serde_json::to_value(&empty).unwrap();
        assert_eq!(encoded.as_object().unwrap().len(), 3);
        assert!(
            encoded["adventure"]
                .as_array()
                .unwrap()
                .iter()
                .all(|rows| rows.as_array().unwrap().is_empty())
        );
    }

    #[test]
    fn owned_catalog_rejects_unknown_identity_and_bounded_bad_strings() {
        assert!(matches!(
            PresetNames::decode_owned(&[]),
            Err(PresetError::CatalogUnavailable)
        ));
        assert!(matches!(
            PresetNames::decode_owned(&vec![0; OWNED_EXE_BYTES]),
            Err(PresetError::CatalogUnavailable)
        ));
        let offsets = [0; 20];
        assert!(read_names(b"fixture\0", &offsets).is_ok());
        assert!(read_names(b"bad\nname\0", &offsets).is_err());
        assert!(read_names(&vec![b'x'; super::super::MAX_NAME_BYTES + 2], &offsets).is_err());
    }
}
