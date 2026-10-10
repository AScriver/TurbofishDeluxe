//! Challenge story awards and a read-only catalog from the owned game install.
//! The catalog contains no bundled story prose or executable game logic.

use serde::{Deserialize, Serialize};
use std::{fmt, fs, io, path::Path};

const STORY_COUNT: u8 = 33;
const INSTALLED_EXE_BYTES: u64 = 3_787_832;
const INSTALLED_EXE_SHA256: &str =
    "f54c2c6ee54b00ae6da7f4bde15feb90d0a867aba6b4151b0277c1f81da3ff66";
const TITLE_ANCHOR: &[u8] = b"CYRAX the Evil Alien Mastermind";
const TEXT_ANCHOR: &[u8] = b"After losing the legendary Battle";

// PB05 00552e50 and W1 WinFishApp::SwitchToStoryScreen group order.
const GROUPS: [[u8; 8]; 4] = [
    [0, 1, 2, 3, 4, 20, 24, 25],
    [5, 6, 7, 8, 9, 21, 26, 27],
    [10, 11, 12, 13, 14, 22, 28, 29],
    [15, 16, 17, 18, 19, 23, 30, 31],
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct StoryId(u8);

impl StoryId {
    pub fn new(value: u8) -> Option<Self> {
        (value < STORY_COUNT).then_some(Self(value))
    }

    pub fn get(self) -> u8 {
        self.0
    }

    /// StoryScreen maps 0..23 to pet images and 24..31 to alien images;
    /// 32 also uses its separately loaded talk-show background.
    pub fn portrait_slot(self) -> u8 {
        if self.0 < 24 { self.0 } else { self.0 - 24 }
    }

    pub fn is_pet(self) -> bool {
        self.0 < 24
    }
}

impl TryFrom<u8> for StoryId {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value).ok_or_else(|| format!("Story ID {value} is outside 0..32"))
    }
}

impl From<StoryId> for u8 {
    fn from(value: StoryId) -> Self {
        value.0
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoryState {
    /// Profile +0x7c in PB05: one bit for each ordinary story, IDs 0..31.
    pub seen_mask: u32,
    /// Profile +0x80 in PB05: Cyrax story ID 32 is separate from the mask.
    pub cyrax_unlocked: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StorySelection {
    pub id: StoryId,
    pub newly_unlocked: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidStoryTank(pub u8);

impl fmt::Display for InvalidStoryTank {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Challenge story requires tank 1..4, got {}",
            self.0
        )
    }
}

impl std::error::Error for InvalidStoryTank {}

impl StoryState {
    pub fn is_unlocked(&self, id: StoryId) -> bool {
        if id.0 == 32 {
            self.cyrax_unlocked
        } else {
            self.seen_mask & (1u32 << id.0) != 0
        }
    }

    pub fn unlocked_count(&self) -> u32 {
        self.seen_mask.count_ones() + u32::from(self.cyrax_unlocked)
    }

    /// Select a Challenge story after the result transaction. `roll(limit)`
    /// and extra-pet eligibility come from the caller's existing authorities.
    /// PB05 00552e50 gates only IDs 20..23 on IsPetUnlocked; all other IDs
    /// remain eligible even though W1 checks every pet story.
    pub fn select_after_challenge(
        &mut self,
        tank: u8,
        bonus_cursor: u32,
        mut extra_pet_unlocked: impl FnMut(u8) -> bool,
        mut roll: impl FnMut(u64) -> u64,
    ) -> Result<StorySelection, InvalidStoryTank> {
        let group = GROUPS
            .get(usize::from(tank.saturating_sub(1)))
            .filter(|_| (1..=4).contains(&tank))
            .ok_or(InvalidStoryTank(tank))?;

        if self.seen_mask == u32::MAX {
            if !self.cyrax_unlocked && bonus_cursor > 5 {
                self.cyrax_unlocked = true;
                return Ok(StorySelection {
                    id: StoryId(32),
                    newly_unlocked: true,
                });
            }
            let limit = if self.cyrax_unlocked { 33 } else { 32 };
            return Ok(StorySelection {
                id: StoryId((roll(limit) % limit) as u8),
                newly_unlocked: false,
            });
        }

        for id in group.iter().copied() {
            if self.seen_mask & (1u32 << id) == 0
                && (!(20..=23).contains(&id) || extra_pet_unlocked(id))
            {
                self.seen_mask |= 1u32 << id;
                return Ok(StorySelection {
                    id: StoryId(id),
                    newly_unlocked: true,
                });
            }
        }

        let seen: Vec<u8> = group
            .iter()
            .copied()
            .filter(|id| self.seen_mask & (1u32 << id) != 0)
            .collect();
        // PB05 pushes ID 0 if the selected group contains no seen entries.
        let id = if seen.is_empty() {
            0
        } else {
            seen[(roll(seen.len() as u64) % seen.len() as u64) as usize]
        };
        Ok(StorySelection {
            id: StoryId(id),
            newly_unlocked: false,
        })
    }

    pub fn open_browser(&self, preferred: StoryId) -> Option<StoryBrowser> {
        if self.is_unlocked(preferred) {
            return Some(StoryBrowser { current: preferred });
        }
        (1..=STORY_COUNT)
            .map(|distance| StoryId((preferred.0 + distance) % STORY_COUNT))
            .find(|id| self.is_unlocked(*id))
            .map(|current| StoryBrowser { current })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoryBrowser {
    pub current: StoryId,
}

impl StoryBrowser {
    /// StoryScreen::ChangeStory wraps across 33 entries and skips unearned
    /// stories. Browse never changes Profile unlock state.
    pub fn step(&mut self, state: &StoryState, direction: i8) -> StoryId {
        if direction != -1 && direction != 1 {
            return self.current;
        }
        let mut candidate = i16::from(self.current.0);
        for _ in 0..STORY_COUNT {
            candidate = (candidate + i16::from(direction) + i16::from(STORY_COUNT))
                % i16::from(STORY_COUNT);
            let id = StoryId(candidate as u8);
            if state.is_unlocked(id) {
                self.current = id;
                break;
            }
        }
        self.current
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoryEntry {
    pub title: String,
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct StoryCatalog {
    entries: [StoryEntry; STORY_COUNT as usize],
}

impl StoryCatalog {
    /// Read only the recognized owned executable. Unknown builds fail clearly
    /// rather than silently displaying mismatched or copied story content.
    pub fn load(executable: &Path) -> io::Result<Self> {
        let metadata = fs::metadata(executable)?;
        if metadata.len() != INSTALLED_EXE_BYTES {
            return Err(unsupported_catalog());
        }
        let bytes = fs::read(executable)?;
        if !crate::install::sha256(&bytes).eq_ignore_ascii_case(INSTALLED_EXE_SHA256) {
            return Err(unsupported_catalog());
        }
        let titles = reverse_strings(&bytes, TITLE_ANCHOR, 80).ok_or_else(unsupported_catalog)?;
        let texts = reverse_strings(&bytes, TEXT_ANCHOR, 1_024).ok_or_else(unsupported_catalog)?;
        let entries = titles
            .into_iter()
            .zip(texts)
            .map(|(title, text)| StoryEntry { title, text })
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| unsupported_catalog())?;
        Ok(Self { entries })
    }

    pub fn entry(&self, id: StoryId) -> &StoryEntry {
        &self.entries[usize::from(id.0)]
    }
}

fn unsupported_catalog() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "Story catalog unavailable for this owned executable build",
    )
}

fn reverse_strings(bytes: &[u8], anchor: &[u8], max_len: usize) -> Option<[String; 33]> {
    let mut matches = bytes
        .windows(anchor.len())
        .enumerate()
        .filter_map(|(index, data)| (data == anchor).then_some(index));
    let mut cursor = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    let mut values = Vec::with_capacity(STORY_COUNT as usize);
    for _ in 0..STORY_COUNT {
        let tail = bytes.get(cursor..)?;
        let end = tail.iter().position(|byte| *byte == 0)?;
        if end == 0 || end > max_len {
            return None;
        }
        values.push(std::str::from_utf8(tail.get(..end)?).ok()?.to_owned());
        cursor = cursor.checked_add(end + 1)?;
        while bytes.get(cursor) == Some(&0) {
            cursor += 1;
        }
    }
    values.reverse();
    values.try_into().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_gate_only_waits_for_extra_pet_stories() {
        let mut state = StoryState {
            seen_mask: 0b1_1111,
            cyrax_unlocked: false,
        };
        let selected = state
            .select_after_challenge(1, 0, |_| false, |_| 0)
            .unwrap();
        assert_eq!(selected.id.get(), 24);
        assert!(selected.newly_unlocked);
        assert!(!state.is_unlocked(StoryId(20)));
    }

    #[test]
    fn cyrax_waits_for_complete_mask_and_cursor() {
        let mut state = StoryState {
            seen_mask: u32::MAX,
            cyrax_unlocked: false,
        };
        assert_ne!(
            state
                .select_after_challenge(4, 5, |_| true, |_| 0)
                .unwrap()
                .id
                .get(),
            32
        );
        let cyrax = state.select_after_challenge(4, 6, |_| true, |_| 0).unwrap();
        assert_eq!(cyrax.id.get(), 32);
        assert!(cyrax.newly_unlocked);
        assert!(state.cyrax_unlocked);
    }
}
