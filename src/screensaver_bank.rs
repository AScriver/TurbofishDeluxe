//! Project-owned screensaver earnings. The saver only appends positive credit;
//! the normal profile writer claims a cumulative watermark with the wallet save.
//! The bank and profile file intentionally use different locks so a running
//! normal game does not block a screensaver process from banking earnings.

use std::collections::HashSet;
use std::error::Error;
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{self, ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

const FORMAT_VERSION: u32 = 1;
const MAX_BANK_BYTES: usize = 1024 * 1024;

pub type BankResult<T> = Result<T, Box<dyn Error>>;

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
            if entry.cumulative == 0 || !keys.insert((entry.profile_id, &entry.generation)) {
                return Err(invalid_data("invalid or duplicate screensaver bank entry").into());
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
        let file: Self = serde_json::from_slice(&bytes)?;
        file.validate()?;
        Ok(file)
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
        key.validate()?;
        fs::create_dir_all(&self.save_dir)?;
        let _lock = lock_bank(&self.save_dir.join("screensaver-bank.lock"))?;
        let bank = BankFile::read(&self.save_dir.join("screensaver-bank.json"))?;
        Ok(bank
            .entries
            .iter()
            .find(|entry| entry.profile_id == key.profile_id && entry.generation == key.generation)
            .map_or(0, |entry| entry.cumulative))
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
                });
                delta
            };
        if serde_json::to_vec_pretty(&bank)?.len() > MAX_BANK_BYTES {
            return Err(invalid_data("screensaver bank exceeds size limit").into());
        }
        crate::cli::write_json(&path, &bank)?;
        Ok(cumulative)
    }
}
