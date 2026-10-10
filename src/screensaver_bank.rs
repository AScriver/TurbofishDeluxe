//! Project-owned screensaver earnings and pending Virtual Tank continuation.
//! The normal profile writer claims a cumulative credit watermark and scene
//! sequence receipt with its authoritative profile save.
//! The bank and profile file intentionally use different locks so a running
//! normal game does not block a screensaver process from banking earnings.

use std::collections::HashSet;
use std::error::Error;
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{self, ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::adventure::VirtualContinuation;

const FORMAT_VERSION: u32 = 2;
const MAX_BANK_BYTES: usize = 1024 * 1024;

pub type BankResult<T> = Result<T, Box<dyn Error>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingScene {
    pub sequence: u64,
    pub baseline_digest: String,
    pub continuation: VirtualContinuation,
}

impl PendingScene {
    fn validate(&self) -> BankResult<()> {
        if self.sequence == 0 || !valid_digest(&self.baseline_digest) {
            return Err(invalid_data("invalid screensaver scene receipt or baseline").into());
        }
        self.continuation.validate().map_err(invalid_scene)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct BankSnapshot {
    pub cumulative: u64,
    pub pending_scene: Option<PendingScene>,
}

fn valid_digest(digest: &str) -> bool {
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn invalid_scene(message: String) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, message)
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BankKey {
    pub profile_id: u64,
    /// Opaque, nonzero 128-bit generation token encoded as 32 lowercase hex digits.
    pub generation: String,
}

impl BankKey {
    pub fn validate(&self) -> BankResult<()> {
        if self.profile_id == 0
            || self.generation.len() != 32
            || !self
                .generation
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            || self.generation.bytes().all(|byte| byte == b'0')
        {
            return Err(invalid_data("invalid screensaver profile key").into());
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BankFile {
    format_version: u32,
    entries: Vec<BankEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BankEntry {
    profile_id: u64,
    generation: String,
    cumulative: u64,
    pending_scene: Option<PendingScene>,
}

impl Default for BankFile {
    fn default() -> Self {
        Self {
            format_version: FORMAT_VERSION,
            entries: Vec::new(),
        }
    }
}

impl BankFile {
    fn validate(&self) -> BankResult<()> {
        if self.format_version != FORMAT_VERSION {
            return Err(invalid_data("unsupported screensaver bank version").into());
        }
        let mut keys = HashSet::new();
        for entry in &self.entries {
            BankKey {
                profile_id: entry.profile_id,
                generation: entry.generation.clone(),
            }
            .validate()?;
            if (entry.cumulative == 0 && entry.pending_scene.is_none())
                || !keys.insert((entry.profile_id, &entry.generation))
            {
                return Err(invalid_data("invalid or duplicate screensaver bank entry").into());
            }
            if let Some(scene) = &entry.pending_scene {
                scene.validate()?;
            }
        }
        Ok(())
    }

    fn read(path: &Path) -> BankResult<Self> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error.into()),
        };
        let mut bytes = Vec::new();
        file.take((MAX_BANK_BYTES + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_BANK_BYTES {
            return Err(invalid_data("screensaver bank exceeds size limit").into());
        }
        let value: serde_json::Value = serde_json::from_slice(&bytes)?;
        let entries = value
            .get("entries")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| invalid_data("screensaver bank entries missing"))?;
        if entries
            .iter()
            .any(|entry| entry.get("pending_scene").is_none())
        {
            return Err(invalid_data("screensaver bank scene field missing").into());
        }
        let file: Self = serde_json::from_value(value)?;
        file.validate()?;
        Ok(file)
    }

    fn write(&self, path: &Path) -> BankResult<()> {
        self.validate()?;
        if serde_json::to_vec_pretty(self)?.len() > MAX_BANK_BYTES {
            return Err(invalid_data("screensaver bank exceeds size limit").into());
        }
        crate::cli::write_json(path, self)
    }
}

fn invalid_data(message: &'static str) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, message)
}

fn open_lock_file(path: &Path) -> BankResult<File> {
    Ok(OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)?)
}

fn lock_file(path: &Path) -> BankResult<File> {
    let file = open_lock_file(path)?;
    file.try_lock()?;
    Ok(file)
}

fn lock_bank(path: &Path) -> BankResult<File> {
    let file = open_lock_file(path)?;
    for attempt in 0..=10 {
        match file.try_lock() {
            Ok(()) => return Ok(file),
            Err(TryLockError::WouldBlock) if attempt < 10 => {
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(error) => return Err(error.into()),
        }
    }
    unreachable!("final bank lock attempt returns or errors")
}

/// Hold for the entire normal-app profile load/save lifetime. Every normal
/// profile writer must acquire this lease before loading its profile snapshot.
#[must_use = "keep the lease alive until the normal profile writer exits"]
pub struct NormalWriterLease {
    _file: File,
}

impl NormalWriterLease {
    pub fn try_acquire(save_dir: &Path) -> BankResult<Self> {
        fs::create_dir_all(save_dir)?;
        Ok(Self {
            _file: lock_file(&save_dir.join("profiles.lock"))?,
        })
    }
}

/// Hold through a standalone saver run. Normal scene adoption briefly acquires
/// the same lock and defers when another saver still owns the scene branch.
#[must_use = "keep the lease alive while a screensaver scene can change"]
pub struct ScreensaverSceneLease {
    _file: File,
}

impl ScreensaverSceneLease {
    pub fn try_acquire(save_dir: &Path) -> BankResult<Option<Self>> {
        fs::create_dir_all(save_dir)?;
        let file = open_lock_file(&save_dir.join("screensaver-scene.lock"))?;
        match file.try_lock() {
            Ok(()) => Ok(Some(Self { _file: file })),
            Err(TryLockError::WouldBlock) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }
}

/// Short locked operations on a separate, private cumulative-credit file.
pub struct ScreensaverBank {
    save_dir: PathBuf,
}

impl ScreensaverBank {
    pub fn new(save_dir: impl Into<PathBuf>) -> Self {
        Self {
            save_dir: save_dir.into(),
        }
    }

    pub fn read(&self, key: &BankKey) -> BankResult<u64> {
        Ok(self.read_snapshot(key)?.cumulative)
    }

    pub fn read_snapshot(&self, key: &BankKey) -> BankResult<BankSnapshot> {
        key.validate()?;
        fs::create_dir_all(&self.save_dir)?;
        let _lock = lock_bank(&self.save_dir.join("screensaver-bank.lock"))?;
        let bank = BankFile::read(&self.save_dir.join("screensaver-bank.json"))?;
        Ok(bank
            .entries
            .iter()
            .find(|entry| entry.profile_id == key.profile_id && entry.generation == key.generation)
            .map_or(
                BankSnapshot {
                    cumulative: 0,
                    pending_scene: None,
                },
                |entry| BankSnapshot {
                    cumulative: entry.cumulative,
                    pending_scene: entry.pending_scene.clone(),
                },
            ))
    }

    /// Returns the durable cumulative total. A lock-contention error means
    /// the caller must retry without discarding this earned delta.
    pub fn append(&self, key: &BankKey, delta: u64) -> BankResult<u64> {
        key.validate()?;
        if delta == 0 {
            return Err(invalid_data("screensaver bank delta must be positive").into());
        }
        fs::create_dir_all(&self.save_dir)?;
        let _lock = lock_bank(&self.save_dir.join("screensaver-bank.lock"))?;
        let path = self.save_dir.join("screensaver-bank.json");
        let mut bank = BankFile::read(&path)?;
        let cumulative =
            if let Some(entry) = bank.entries.iter_mut().find(|entry| {
                entry.profile_id == key.profile_id && entry.generation == key.generation
            }) {
                entry.cumulative = entry
                    .cumulative
                    .checked_add(delta)
                    .ok_or_else(|| invalid_data("screensaver bank total overflow"))?;
                entry.cumulative
            } else {
                bank.entries.push(BankEntry {
                    profile_id: key.profile_id,
                    generation: key.generation.clone(),
                    cumulative: delta,
                    pending_scene: None,
                });
                delta
            };
        bank.write(&path)?;
        Ok(cumulative)
    }

    /// Persist one scene and its earned positive credit in the same bank
    /// replacement. Zero credit still records a moved or changed scene.
    pub fn checkpoint(
        &self,
        key: &BankKey,
        delta: u64,
        baseline_digest: &str,
        continuation: VirtualContinuation,
    ) -> BankResult<BankSnapshot> {
        key.validate()?;
        if !valid_digest(baseline_digest) {
            return Err(invalid_data("invalid screensaver scene baseline").into());
        }
        continuation.validate().map_err(invalid_scene)?;
        fs::create_dir_all(&self.save_dir)?;
        let _lock = lock_bank(&self.save_dir.join("screensaver-bank.lock"))?;
        let path = self.save_dir.join("screensaver-bank.json");
        let mut bank = BankFile::read(&path)?;
        let entry_index = if let Some(index) = bank.entries.iter().position(|entry| {
            entry.profile_id == key.profile_id && entry.generation == key.generation
        }) {
            index
        } else {
            bank.entries.push(BankEntry {
                profile_id: key.profile_id,
                generation: key.generation.clone(),
                cumulative: 0,
                pending_scene: None,
            });
            bank.entries.len() - 1
        };
        let entry = &mut bank.entries[entry_index];
        let cumulative = entry
            .cumulative
            .checked_add(delta)
            .ok_or_else(|| invalid_data("screensaver bank total overflow"))?;
        let sequence = entry
            .pending_scene
            .as_ref()
            .map_or(0, |scene| scene.sequence)
            .checked_add(1)
            .ok_or_else(|| invalid_data("screensaver scene sequence overflow"))?;
        let pending_scene = PendingScene {
            sequence,
            baseline_digest: baseline_digest.to_owned(),
            continuation,
        };
        entry.cumulative = cumulative;
        entry.pending_scene = Some(pending_scene.clone());
        bank.write(&path)?;
        Ok(BankSnapshot {
            cumulative,
            pending_scene: Some(pending_scene),
        })
    }
}
