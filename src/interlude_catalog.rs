//! Read-only finale text catalog for the identified owned English executable.
//! Numeric slices are checked against pinned W1 DrawStringHelper calls; no
//! narration, credit prose or executable game logic is bundled here.

use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

const EXECUTABLE_BYTES: u64 = 3_787_832;
const EXECUTABLE_SHA256: &str = "f54c2c6ee54b00ae6da7f4bde15feb90d0a867aba6b4151b0277c1f81da3ff66";
pub const TEXT_COUNT: usize = 116;
const MAX_TEXT_BYTES: usize = 128;

// (W1 InterludeScreen.cpp source line, owned file offset, ASCII byte length).
// Exact digest gates every slice, including shared/pooled literals. This is
// data extraction only: the owned executable is never loaded or executed.
const TEXT_SLICES: [(u16, usize, usize); TEXT_COUNT] = [
    (177, 0x32a62e, 16),
    (178, 0x32d012, 48),
    (179, 0x32cfe6, 43),
    (180, 0x32cfca, 24),
    (181, 0x32cfa6, 34),
    (182, 0x32cf92, 19),
    (183, 0x32cf5e, 48),
    (184, 0x32cf3e, 30),
    (185, 0x32cf0e, 44),
    (186, 0x32ceda, 48),
    (281, 0x32cebe, 14),
    (282, 0x32cea2, 26),
    (283, 0x32ce8a, 20),
    (284, 0x32ce72, 22),
    (285, 0x32ce5a, 20),
    (286, 0x32ce42, 21),
    (287, 0x32ce26, 26),
    (291, 0x32ce0a, 24),
    (292, 0x32cdf2, 23),
    (293, 0x32cdda, 21),
    (294, 0x32cdba, 30),
    (295, 0x32cda2, 20),
    (299, 0x32cd8e, 16),
    (300, 0x32cd7a, 19),
    (301, 0x32cd66, 19),
    (302, 0x32cd4e, 21),
    (306, 0x32cd2e, 30),
    (307, 0x32cd12, 24),
    (308, 0x32cd02, 12),
    (312, 0x32ccd2, 47),
    (313, 0x32ccc6, 9),
    (318, 0x32ccb2, 19),
    (319, 0x32cc96, 24),
    (320, 0x32cc7a, 25),
    (321, 0x32cc72, 4),
    (322, 0x32cc6a, 5),
    (324, 0x32cc5e, 9),
    (326, 0x32cc52, 11),
    (328, 0x32cc42, 13),
    (330, 0x32cc32, 13),
    (332, 0x32cc2a, 7),
    (334, 0x32cc1e, 9),
    (336, 0x32cc12, 9),
    (338, 0x32cc0a, 6),
    (340, 0x32cc06, 3),
    (342, 0x32cbfa, 10),
    (344, 0x32cbf2, 7),
    (346, 0x32cbe6, 11),
    (349, 0x32cbda, 9),
    (359, 0x32cbd2, 5),
    (361, 0x324086, 6),
    (363, 0x32407e, 4),
    (365, 0x324076, 5),
    (367, 0x32406e, 5),
    (369, 0x324066, 4),
    (371, 0x32405e, 5),
    (373, 0x324056, 4),
    (375, 0x32404e, 5),
    (377, 0x324046, 5),
    (379, 0x32403a, 9),
    (381, 0x32cbca, 7),
    (383, 0x32cbbe, 8),
    (385, 0x32cbb6, 5),
    (387, 0x32cbae, 4),
    (389, 0x32cba6, 7),
    (391, 0x32cb9e, 6),
    (393, 0x32cb9a, 3),
    (405, 0x32cb92, 4),
    (407, 0x32cb8a, 5),
    (409, 0x32cb82, 6),
    (415, 0x32c2c6, 1),
    (420, 0x32b59e, 8),
    (426, 0x32c2c6, 1),
    (431, 0x32cb76, 11),
    (438, 0x32c2c6, 1),
    (443, 0x32cb6e, 7),
    (449, 0x32c2c6, 1),
    (454, 0x32cb66, 6),
    (461, 0x32cb5e, 7),
    (463, 0x32c88a, 11),
    (464, 0x32c896, 10),
    (466, 0x32c87e, 8),
    (467, 0x32cb4e, 13),
    (468, 0x32cb3e, 13),
    (470, 0x32c852, 11),
    (471, 0x32c896, 10),
    (472, 0x32cb32, 10),
    (473, 0x32cb22, 15),
    (475, 0x32c7fe, 3),
    (476, 0x32cb12, 12),
    (477, 0x32cb02, 13),
    (479, 0x32c7ce, 16),
    (480, 0x32c896, 10),
    (482, 0x32c7c6, 5),
    (483, 0x32caf2, 14),
    (484, 0x32c896, 10),
    (486, 0x32c796, 16),
    (487, 0x32cae6, 11),
    (489, 0x32c772, 7),
    (490, 0x32c766, 11),
    (492, 0x32c762, 2),
    (493, 0x32cada, 11),
    (494, 0x32caca, 12),
    (495, 0x32caba, 12),
    (496, 0x32caaa, 13),
    (498, 0x32ca9a, 15),
    (499, 0x32ca86, 16),
    (501, 0x32ca76, 14),
    (502, 0x32ca62, 19),
    (503, 0x32ca52, 15),
    (504, 0x32ca46, 11),
    (505, 0x32ca36, 12),
    (507, 0x32ca22, 18),
    (508, 0x32ca06, 25),
    (509, 0x32ca02, 3),
    (510, 0x34803a, 12),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextId(u8);

impl TextId {
    /// Stable crosswalk into the pinned W1 Draw function, without stored prose.
    pub fn from_source_line(line: u16) -> Option<Self> {
        TEXT_SLICES
            .binary_search_by_key(&line, |entry| entry.0)
            .ok()
            .map(|index| Self(index as u8))
    }

    pub fn source_line(self) -> u16 {
        TEXT_SLICES[usize::from(self.0)].0
    }

    pub(super) fn draw_line(line: u16) -> Self {
        Self::from_source_line(line).expect("finale row has a registered catalog slice")
    }
}

#[derive(Clone, Debug)]
pub struct FinaleCatalog {
    entries: [String; TEXT_COUNT],
}

impl FinaleCatalog {
    /// The file stays read-only. The read limit also bounds a file that changes
    /// after metadata inspection; length and SHA-256 are checked on read bytes.
    pub fn load(executable: &Path) -> io::Result<Self> {
        let file = File::open(executable).map_err(unavailable_io)?;
        let metadata = file.metadata().map_err(unavailable_io)?;
        if !metadata.is_file() || metadata.len() != EXECUTABLE_BYTES {
            return Err(unavailable("unsupported owned executable length/type"));
        }
        let mut bytes = Vec::with_capacity(EXECUTABLE_BYTES as usize);
        file.take(EXECUTABLE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(unavailable_io)?;
        Self::from_bytes(&bytes)
    }

    pub fn text(&self, id: TextId) -> &str {
        &self.entries[usize::from(id.0)]
    }

    fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() as u64 != EXECUTABLE_BYTES
            || !crate::install::sha256(bytes).eq_ignore_ascii_case(EXECUTABLE_SHA256)
        {
            return Err(unavailable("unrecognized owned executable identity"));
        }
        let entries = TEXT_SLICES
            .iter()
            .map(|&(line, offset, length)| {
                let text = decode_text(bytes, offset, length)?;
                if (line == 300
                    && (text.matches("%d").count() != 1 || text.matches('%').count() != 1))
                    || (line != 300 && text.contains('%'))
                {
                    return Err(unavailable("malformed death-count template"));
                }
                Ok(text)
            })
            .collect::<io::Result<Vec<_>>>()?
            .try_into()
            .map_err(|_| unavailable("incomplete catalog"))?;
        Ok(Self { entries })
    }
}

fn decode_text(bytes: &[u8], offset: usize, length: usize) -> io::Result<String> {
    if !(1..=MAX_TEXT_BYTES).contains(&length) {
        return Err(unavailable("invalid text length"));
    }
    let end = offset
        .checked_add(length)
        .ok_or_else(|| unavailable("invalid text offset"))?;
    let text = bytes
        .get(offset..end)
        .filter(|text| {
            text.iter()
                .all(|byte| byte.is_ascii_graphic() || *byte == b' ')
        })
        .filter(|_| bytes.get(end) == Some(&0))
        .ok_or_else(|| unavailable("missing or malformed text slice"))?;
    Ok(std::str::from_utf8(text)
        .map_err(|_| unavailable("invalid text encoding"))?
        .to_owned())
}

fn unavailable(reason: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("Finale catalog unavailable: {reason}"),
    )
}

fn unavailable_io(error: io::Error) -> io::Error {
    io::Error::new(error.kind(), format!("Finale catalog unavailable: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_identity_and_malformed_bounded_slices() {
        for bytes in [Vec::new(), vec![0; EXECUTABLE_BYTES as usize]] {
            let error = FinaleCatalog::from_bytes(&bytes).unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidData);
            assert!(error.to_string().starts_with("Finale catalog unavailable:"));
        }
        assert_eq!(decode_text(b"OK\0", 0, 2).unwrap(), "OK");
        for (bytes, offset, length) in [
            (&b"OK"[..], 0, 2),
            (&b"O\0\0"[..], 0, 2),
            (&b"\xff\0"[..], 0, 1),
            (&b"\n\0"[..], 0, 1),
            (&b"OK\0"[..], usize::MAX, 2),
            (&b"OK\0"[..], 0, 0),
            (&b"OK\0"[..], 0, MAX_TEXT_BYTES + 1),
        ] {
            assert!(decode_text(bytes, offset, length).is_err());
        }
    }

    #[test]
    fn source_keys_are_unique_bounded_and_missing_file_is_unavailable() {
        assert!(TEXT_SLICES.windows(2).all(|pair| pair[0].0 < pair[1].0));
        assert!(TextId::from_source_line(176).is_none());
        for &(line, offset, length) in &TEXT_SLICES {
            assert_eq!(TextId::from_source_line(line).unwrap().source_line(), line);
            assert!((1..=MAX_TEXT_BYTES).contains(&length));
            assert!(offset + length < EXECUTABLE_BYTES as usize);
        }
        let error = FinaleCatalog::load(Path::new("")).unwrap_err();
        assert!(error.to_string().starts_with("Finale catalog unavailable:"));
    }

    #[test]
    #[ignore = "requires OWNED_GAME_DIR pointing to the recognized owned installation"]
    fn owned_finale_catalog_covers_all_source_rows_without_writing_install() {
        let root = std::env::var_os("OWNED_GAME_DIR").expect("owned installation");
        let executable = Path::new(&root).join("InsaniquariumDeluxe.exe");
        let before = std::fs::read(&executable).unwrap();
        let catalog = FinaleCatalog::load(&executable).unwrap();
        assert_eq!(catalog.entries.len(), 116);
        assert_eq!(catalog.text(TextId::draw_line(178)).len(), 48);
        assert_eq!(catalog.text(TextId::draw_line(510)).len(), 12);
        assert_eq!(catalog.text(TextId::draw_line(415)).len(), 1);
        let template = catalog.text(TextId::draw_line(300));
        assert_eq!(template.len(), 19);
        assert_eq!(template.matches("%d").count(), 1);
        for count in 0..=18 {
            let controller =
                super::super::InterludeController::new(&crate::sim::TANK5_PETS[..count]).unwrap();
            for row in controller.presentation(|_| false).rows {
                let text = row.text.resolve(&catalog);
                assert!(!text.is_empty());
                assert!(!text.contains('%'));
            }
        }
        assert_eq!(before, std::fs::read(&executable).unwrap());
    }
}
