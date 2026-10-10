//! Read-only biography text from the recognized owned game executable.
//! PB05 pointer tables, not literal order, define the numeric ID mapping.

use std::{collections::HashMap, fs, io, path::Path};

const INSTALLED_EXE_BYTES: u64 = 3_787_832;
const INSTALLED_EXE_SHA256: &str =
    "f54c2c6ee54b00ae6da7f4bde15feb90d0a867aba6b4151b0277c1f81da3ff66";
const EMBEDDED_OFFSET: usize = 0x1881a2;
const IMAGE_BASE: u32 = 0x0040_0000;
const RDATA_RANGE: std::ops::Range<usize> = 0x196000..0x1df000;

pub struct OwnedBiographyCatalog {
    pub hometowns: HashMap<u16, String>,
    pub special_hometowns: HashMap<u16, String>,
    pub likes: HashMap<u16, String>,
}

impl OwnedBiographyCatalog {
    /// No game code is executed; only the identified pointer/string data is read.
    pub fn load(executable: &Path) -> io::Result<Self> {
        if fs::metadata(executable)?.len() != INSTALLED_EXE_BYTES {
            return Err(unavailable());
        }
        let bytes = fs::read(executable)?;
        if !crate::install::sha256(&bytes).eq_ignore_ascii_case(INSTALLED_EXE_SHA256) {
            return Err(unavailable());
        }
        Ok(Self {
            hometowns: read_table(&bytes, 0x1e0430, 375).ok_or_else(unavailable)?,
            special_hometowns: read_table(&bytes, 0x1e0414, 6).ok_or_else(unavailable)?,
            likes: read_table(&bytes, 0x1dfee8, 331).ok_or_else(unavailable)?,
        })
    }
}

fn read_table(bytes: &[u8], table: usize, count: u16) -> Option<HashMap<u16, String>> {
    let mut values = HashMap::with_capacity(usize::from(count));
    for id in 0..count {
        let pointer_offset = EMBEDDED_OFFSET
            .checked_add(table)?
            .checked_add(usize::from(id) * 4)?;
        let pointer = u32::from_le_bytes(
            bytes
                .get(pointer_offset..pointer_offset + 4)?
                .try_into()
                .ok()?,
        );
        // PB05 .rdata has identical file/RVA positions. Reject any other section.
        let text_offset = usize::try_from(pointer.checked_sub(IMAGE_BASE)?).ok()?;
        if !RDATA_RANGE.contains(&text_offset) {
            return None;
        }
        let start = EMBEDDED_OFFSET.checked_add(text_offset)?;
        let tail = bytes.get(start..start.checked_add(129)?)?;
        let end = tail.iter().position(|byte| *byte == 0)?;
        if end == 0 || !tail[..end].iter().all(|byte| (32..=126).contains(byte)) {
            return None;
        }
        values.insert(id, std::str::from_utf8(&tail[..end]).ok()?.to_owned());
    }
    Some(values)
}

fn unavailable() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "Virtual Tank biography catalog unavailable for this owned executable build",
    )
}
