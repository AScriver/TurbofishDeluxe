//! Transient Adventure ending presentation model.
//!
//! Scroll, focus, music and pet-return timing follow pinned W1
//! `InterludeScreen.cpp` (secondary source). The full presentation references
//! a read-only owned-install catalog; no retail prose is bundled. App rendering,
//! audible fades and native/retail comparison remain separate integration work.

use crate::sim::{PetKind, TANK5_PETS};
use std::borrow::Cow;

#[path = "interlude_catalog.rs"]
pub mod catalog;

pub const END_SCROLL: u32 = 7_645;
pub const CATALOG_UNAVAILABLE_MESSAGE: &str = "Finale catalog unavailable";
const LULLABY_CHANGE: u32 = 3_000;
const PET_EFFECT_END: u32 = 1_700;
const PET_SLOTS: usize = 15;
const EFFECT_TIMES: [u32; PET_SLOTS] = [
    733, 773, 813, 853, 893, 965, 1_005, 1_045, 1_085, 1_125, 1_243, 1_283, 1_323, 1_363, 1_403,
];
const PET_X: [i32; PET_SLOTS] = [
    480, 360, 240, 120, 0, 10, 130, 250, 370, 490, 480, 360, 240, 120, 0,
];
const PET_Y: [i32; PET_SLOTS] = [
    1_125, 1_145, 1_135, 1_125, 1_145, 1_295, 1_275, 1_285, 1_295, 1_275, 1_425, 1_445, 1_435,
    1_425, 1_445,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicStage {
    Pending,
    Interlude,
    Lullaby,
    Stopped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterludeEvent {
    StartInterludeMusic,
    CrossfadeToLullaby,
    Heal(PetKind),
    BackToMenu,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowContent {
    Heading(&'static str),
    Text(&'static str),
    Cast(PetKind),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InterludeRow {
    pub content: RowContent,
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResurrectionPose {
    pub pet: PetKind,
    pub x: i32,
    pub y: i32,
    pub frame: u8,
    pub alpha: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FinaleFont {
    Heading,
    Body,
}

impl FinaleFont {
    pub fn resource_key(self) -> &'static str {
        match self {
            Self::Heading => "FONT_LIDDIE18",
            Self::Body => "FONT_LIDDIE15",
        }
    }

    pub fn rgba(self) -> [u8; 4] {
        match self {
            Self::Heading => [215, 255, 255, 255],
            Self::Body => [150, 250, 250, 255],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FinaleColumn {
    Center,
    Left,
    Right,
}

impl FinaleColumn {
    /// Source integer placement uses these sums rather than 320/160/400
    /// anchors. Measure with the selected owned bitmap font before drawing.
    pub fn text_x(self, measured_width: i32) -> i32 {
        let width_sum = match self {
            Self::Center => 650,
            Self::Left => 395,
            Self::Right => 885,
        };
        (width_sum - measured_width) / 2
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FinaleSection {
    Narrative,
    Survivors,
    Cast,
    Credits,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FinaleText {
    Catalog(catalog::TextId),
    DeathCount(u8),
}

impl FinaleText {
    pub fn id(self) -> catalog::TextId {
        match self {
            Self::Catalog(id) => id,
            Self::DeathCount(_) => catalog::TextId::draw_line(300),
        }
    }

    pub fn resolve(self, catalog: &catalog::FinaleCatalog) -> Cow<'_, str> {
        let text = catalog.text(self.id());
        match self {
            Self::Catalog(_) => Cow::Borrowed(text),
            Self::DeathCount(count) => Cow::Owned(text.replacen("%d", &count.to_string(), 1)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CastMember {
    Guppy,
    Carnivore,
    Starcatcher,
    Guppycruncher,
    Beetlemuncher,
    Breeder,
    Ultravore,
    Sylvester,
    Balrog,
    Gus,
    Destructor,
    Ulysses,
    Psychosquid,
    Bilaterus,
    Cyrax,
    Pet(PetKind),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CastVisibility {
    Revealed,
    Concealed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BonusPet {
    Brinkley,
    Nostradamus,
    Stanley,
    Walter,
}

impl BonusPet {
    pub fn kind(self) -> PetKind {
        match self {
            Self::Brinkley => PetKind::Brinkley,
            Self::Nostradamus => PetKind::Nostradamus,
            Self::Stanley => PetKind::Stanley,
            Self::Walter => PetKind::Walter,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FinaleImage {
    SmallSwim,
    Starcatcher,
    Grubber,
    Gekko,
    Breeder,
    Ultra,
    Sylvester,
    Balrog,
    Gus,
    Destructor,
    Ulysses,
    Psychosquid,
    Bilaterus,
    Boss,
    Pet(PetKind),
    BonusEgg(BonusPet),
    AmpCharge,
    Halo,
}

impl FinaleImage {
    pub fn resource_key(self) -> &'static str {
        match self {
            Self::SmallSwim => "IMAGE_SMALLSWIM",
            Self::Starcatcher => "IMAGE_STARCATCHER",
            Self::Grubber => "IMAGE_GRUBBER",
            Self::Gekko => "IMAGE_GEKKO",
            Self::Breeder => "IMAGE_BREEDER",
            Self::Ultra => "IMAGE_ULTRA",
            Self::Sylvester => "IMAGE_SYLV",
            Self::Balrog => "IMAGE_BALROG",
            Self::Gus => "IMAGE_GUS",
            Self::Destructor => "IMAGE_DESTRUCTOR",
            Self::Ulysses => "IMAGE_ULYSSES",
            Self::Psychosquid => "IMAGE_PSYCHOSQUID",
            Self::Bilaterus => "IMAGE_BILATERUS",
            Self::Boss => "IMAGE_BOSS",
            Self::BonusEgg(BonusPet::Brinkley) => "IMAGE_EGGBRINKLEY",
            Self::BonusEgg(BonusPet::Nostradamus) => "IMAGE_EGGNOSTRADAMUS",
            Self::BonusEgg(BonusPet::Stanley) => "IMAGE_EGGSTANLEY",
            Self::BonusEgg(BonusPet::Walter) => "IMAGE_EGGWALTER",
            Self::AmpCharge => "IMAGE_AMPCHARGE",
            Self::Halo => "IMAGE_HALO",
            Self::Pet(kind) => match kind {
                PetKind::Stinky => "IMAGE_STINKY",
                PetKind::Niko => "IMAGE_NIKO",
                PetKind::Itchy => "IMAGE_ITCHY",
                PetKind::Prego => "IMAGE_PREGO",
                PetKind::Zorf => "IMAGE_ZORF",
                PetKind::Clyde => "IMAGE_CLYDE",
                PetKind::Vert => "IMAGE_VERT",
                PetKind::Rufus => "IMAGE_RUFUS",
                PetKind::Meryl => "IMAGE_MERYL",
                PetKind::Wadsworth => "IMAGE_WADSWORTH",
                PetKind::Seymour => "IMAGE_SEYMOUR",
                PetKind::Shrapnel => "IMAGE_SHRAPNEL",
                PetKind::Gumbo => "IMAGE_GUMBO",
                PetKind::Blip => "IMAGE_BLIP",
                PetKind::Rhubarb => "IMAGE_RHUBARB",
                PetKind::Nimbus => "IMAGE_NIMBUS",
                PetKind::Amp => "IMAGE_AMP",
                PetKind::Gash => "IMAGE_GASH",
                PetKind::Angie => "IMAGE_ANGIE",
                PetKind::Presto => "IMAGE_PRESTO",
                PetKind::Brinkley => "IMAGE_BRINKLEY",
                PetKind::Nostradamus => "IMAGE_NOSTRADAMUS",
                PetKind::Stanley => "IMAGE_STANLEY",
                PetKind::Walter => "IMAGE_WALTER",
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FinaleSpriteFrame {
    WholeImage,
    Cell { column: u8, row: u8 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FinaleBlend {
    Normal,
    Additive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FinaleRow {
    pub text: FinaleText,
    pub section: FinaleSection,
    pub font: FinaleFont,
    pub column: FinaleColumn,
    /// Baseline in screen coordinates, including the controller's scroll.
    pub y: i32,
    pub cast: Option<CastMember>,
    pub visibility: CastVisibility,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FinaleSprite {
    pub member: CastMember,
    pub image: FinaleImage,
    pub frame: FinaleSpriteFrame,
    /// Top-left in screen coordinates, including the controller's scroll.
    pub x: i32,
    pub y: i32,
    pub mirror: bool,
    pub rgba: [u8; 4],
    pub blend: FinaleBlend,
    pub visibility: CastVisibility,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FinalePresentation {
    pub background_rgba: [u8; 4],
    /// x, y, width, height. Clip text and sprites, including partial glyphs.
    pub clip: [i32; 4],
    /// Source draw order; rows outside the clip are intentionally retained.
    pub rows: Vec<FinaleRow>,
    /// Preserve order for layered halo/charge and Bilaterus parts.
    pub sprites: Vec<FinaleSprite>,
    pub bubble_fish_start_scroll: u32,
}

struct RowBuilder {
    content_y: i32,
    scroll: i32,
    rows: Vec<FinaleRow>,
}

impl RowBuilder {
    fn push(
        &mut self,
        line: u16,
        spacing: u8,
        font: FinaleFont,
        column: FinaleColumn,
        section: FinaleSection,
    ) -> i32 {
        self.content_y += 35 * i32::from(spacing);
        let y = self.content_y - self.scroll;
        self.rows.push(FinaleRow {
            text: FinaleText::Catalog(catalog::TextId::draw_line(line)),
            section,
            font,
            column,
            y,
            cast: None,
            visibility: CastVisibility::Revealed,
        });
        y
    }

    fn cast(
        &mut self,
        line: u16,
        spacing: u8,
        column: FinaleColumn,
        member: CastMember,
        visibility: CastVisibility,
    ) -> i32 {
        let font = match visibility {
            CastVisibility::Revealed => FinaleFont::Body,
            CastVisibility::Concealed => FinaleFont::Heading,
        };
        let y = self.push(line, spacing, font, column, FinaleSection::Cast);
        let row = self.rows.last_mut().expect("cast row was appended");
        row.cast = Some(member);
        row.visibility = visibility;
        y
    }
}

impl FinaleSprite {
    fn cell(member: CastMember, image: FinaleImage, x: i32, y: i32, column: u8, row: u8) -> Self {
        Self {
            member,
            image,
            frame: FinaleSpriteFrame::Cell { column, row },
            x,
            y,
            mirror: false,
            rgba: [255; 4],
            blend: FinaleBlend::Normal,
            visibility: CastVisibility::Revealed,
        }
    }
}

pub struct InterludeController {
    fallen: Vec<PetKind>,
    slots: [Option<PetKind>; PET_SLOTS],
    update_calls: u32,
    scroll: u32,
    music: MusicStage,
    exited: bool,
    halo_phase: f32,
}

impl InterludeController {
    /// Input contains dead members of the fixed Tank 5 roster. Order is
    /// normalized to that roster, as W1 Board's collection loop does.
    pub fn new(fallen: &[PetKind]) -> Result<Self, &'static str> {
        if fallen.len() > TANK5_PETS.len()
            || fallen.iter().any(|kind| !TANK5_PETS.contains(kind))
            || fallen
                .iter()
                .enumerate()
                .any(|(index, kind)| fallen[..index].contains(kind))
        {
            return Err("fallen pets must be unique members of the Tank 5 roster");
        }
        let ordered: Vec<_> = TANK5_PETS
            .iter()
            .copied()
            .filter(|kind| fallen.contains(kind))
            .collect();
        let mut slots = [None; PET_SLOTS];
        let offset = usize::from(ordered.len() < 4);
        for (slot, pet) in slots.iter_mut().skip(offset).zip(ordered.iter().copied()) {
            *slot = Some(pet);
        }
        Ok(Self {
            fallen: ordered,
            slots,
            update_calls: 0,
            scroll: 0,
            music: MusicStage::Pending,
            exited: false,
            halo_phase: 0.0,
        })
    }

    pub fn fallen(&self) -> &[PetKind] {
        &self.fallen
    }
    pub fn scroll(&self) -> u32 {
        self.scroll
    }
    pub fn music_stage(&self) -> MusicStage {
        self.music
    }
    pub fn can_skip_by_click(&self) -> bool {
        self.scroll >= END_SCROLL
    }
    pub fn exited(&self) -> bool {
        self.exited
    }

    /// One source-style update call. First cue happens even without focus;
    /// scrolling and transition cues require focus.
    pub fn advance(&mut self, has_focus: bool) -> Vec<InterludeEvent> {
        if self.exited {
            return Vec::new();
        }
        self.update_calls = self.update_calls.saturating_add(1);
        let mut events = Vec::with_capacity(2);
        if self.update_calls == 1 {
            self.music = MusicStage::Interlude;
            events.push(InterludeEvent::StartInterludeMusic);
        }
        if !has_focus {
            return events;
        }
        self.halo_phase += 0.01;
        if self.halo_phase > 1.0 {
            self.halo_phase = -1.0;
        }
        if self.scroll == LULLABY_CHANGE {
            self.music = MusicStage::Lullaby;
            events.push(InterludeEvent::CrossfadeToLullaby);
        }
        if self.scroll < END_SCROLL {
            self.scroll += 1;
        }
        if self.scroll < PET_EFFECT_END {
            for (slot, &due) in EFFECT_TIMES.iter().enumerate() {
                if self.scroll == due
                    && let Some(pet) = self.slots[slot]
                {
                    events.push(InterludeEvent::Heal(pet));
                }
            }
        }
        events
    }

    /// Link/button route is always available. A background click uses
    /// `click_after_scroll` and only exits after the scroll completes.
    pub fn back_to_menu(&mut self) -> Option<InterludeEvent> {
        if self.exited {
            return None;
        }
        self.exited = true;
        self.music = MusicStage::Stopped;
        Some(InterludeEvent::BackToMenu)
    }

    pub fn click_after_scroll(&mut self) -> Option<InterludeEvent> {
        self.can_skip_by_click()
            .then(|| self.back_to_menu())
            .flatten()
    }

    /// Compatibility surface for the old summary renderer. It has no catalog
    /// input, so it must report unavailable instead of inventing narration or
    /// exposing unearned bonus pets. New callers use `presentation` and resolve
    /// each row through `catalog::FinaleCatalog`.
    pub fn visible_rows(&self) -> Vec<InterludeRow> {
        vec![InterludeRow {
            content: RowContent::Heading(CATALOG_UNAVAILABLE_MESSAGE),
            x: 320,
            y: 80,
        }]
    }

    /// Full source-informed presentation, without changing progression or
    /// unlock state. `pet_unlocked` reads the existing profile authority.
    /// Apply `clip` when rendering, resolve text from the loaded catalog, and
    /// preserve sprite order. All positions already include scroll.
    pub fn presentation(
        &self,
        mut pet_unlocked: impl FnMut(PetKind) -> bool,
    ) -> FinalePresentation {
        use CastMember as Member;
        use FinaleColumn::{Center, Left, Right};
        use FinaleFont::{Body, Heading};
        use FinaleSection::{Cast, Credits, Narrative, Survivors};

        let mut rows = RowBuilder {
            content_y: 540,
            scroll: self.scroll as i32,
            rows: Vec::with_capacity(100),
        };
        rows.push(177, 0, Heading, Center, Narrative);
        for (line, spacing) in [
            (178, 2),
            (179, 2),
            (180, 1),
            (181, 2),
            (182, 1),
            (183, 3),
            (184, 1),
            (185, 2),
            (186, 1),
        ] {
            rows.push(line, spacing, Body, Center, Narrative);
        }
        rows.content_y += 150;
        let after_survivors = rows.content_y + 150;
        let survivor_lines: &[(u16, u8)] = match self.fallen.len() {
            0 => &[
                (281, 0),
                (282, 1),
                (283, 1),
                (284, 1),
                (285, 3),
                (286, 1),
                (287, 3),
            ],
            1 => &[(291, 5), (292, 1), (293, 1), (294, 2), (295, 1)],
            2..=3 => &[(299, 5), (300, 1), (301, 2), (302, 1)],
            4..=5 => &[(306, 5), (307, 1), (308, 2)],
            6..=10 => &[(312, 7), (313, 1)],
            _ => &[],
        };
        for &(line, spacing) in survivor_lines {
            rows.push(line, spacing, Body, Center, Survivors);
            if line == 300 {
                rows.rows.last_mut().unwrap().text =
                    FinaleText::DeathCount(self.fallen.len() as u8);
            }
        }
        // W1 deliberately resets the cursor; the branch length cannot push
        // cast/credits or bubble admission earlier/later.
        rows.content_y = after_survivors;
        for (line, spacing) in [(318, 8), (319, 1), (320, 4)] {
            rows.push(line, spacing, Body, Center, Narrative);
        }
        rows.push(321, 4, Heading, Center, Cast);

        let mut sprites = self.return_sprites();
        for (member, line, spacing, column) in [
            (Member::Guppy, 322, 4, Left),
            (Member::Carnivore, 324, 0, Right),
            (Member::Starcatcher, 326, 4, Left),
            (Member::Guppycruncher, 328, 0, Right),
            (Member::Beetlemuncher, 330, 4, Left),
            (Member::Breeder, 332, 0, Right),
            (Member::Ultravore, 334, 5, Center),
            (Member::Sylvester, 336, 8, Left),
            (Member::Balrog, 338, 0, Right),
            (Member::Gus, 340, 6, Left),
            (Member::Destructor, 342, 0, Right),
            (Member::Ulysses, 344, 6, Left),
            (Member::Psychosquid, 346, 0, Right),
            (Member::Bilaterus, 349, 6, Left),
            (Member::Cyrax, 359, 0, Right),
            (Member::Pet(PetKind::Stinky), 361, 7, Left),
            (Member::Pet(PetKind::Niko), 363, 0, Right),
            (Member::Pet(PetKind::Itchy), 365, 4, Left),
            (Member::Pet(PetKind::Prego), 367, 0, Right),
            (Member::Pet(PetKind::Zorf), 369, 4, Left),
            (Member::Pet(PetKind::Clyde), 371, 0, Right),
            (Member::Pet(PetKind::Vert), 373, 4, Left),
            (Member::Pet(PetKind::Rufus), 375, 0, Right),
            (Member::Pet(PetKind::Meryl), 377, 4, Left),
            (Member::Pet(PetKind::Wadsworth), 379, 0, Right),
            (Member::Pet(PetKind::Seymour), 381, 4, Left),
            (Member::Pet(PetKind::Shrapnel), 383, 0, Right),
            (Member::Pet(PetKind::Gumbo), 385, 4, Left),
            (Member::Pet(PetKind::Blip), 387, 0, Right),
            (Member::Pet(PetKind::Rhubarb), 389, 4, Left),
            (Member::Pet(PetKind::Nimbus), 391, 0, Right),
            (Member::Pet(PetKind::Amp), 393, 4, Left),
            (Member::Pet(PetKind::Gash), 405, 0, Right),
            (Member::Pet(PetKind::Angie), 407, 4, Left),
            (Member::Pet(PetKind::Presto), 409, 0, Right),
        ] {
            let baseline = rows.cast(line, spacing, column, member, CastVisibility::Revealed);
            self.add_cast_sprites(&mut sprites, member, column, baseline);
        }

        for (pet, concealed_line, revealed_line, column, advance) in [
            (BonusPet::Brinkley, 415, 420, Left, true),
            (BonusPet::Nostradamus, 426, 431, Right, false),
            (BonusPet::Stanley, 438, 443, Left, true),
            (BonusPet::Walter, 449, 454, Right, false),
        ] {
            if advance {
                rows.content_y += 210;
            }
            // W1 line436 mistakenly queries Brinkley for Stanley. Use each
            // pet's own authority to prevent revealing an unearned identity;
            // the installed gate remains an explicit retail follow-up.
            let revealed = pet_unlocked(pet.kind());
            let visibility = if revealed {
                CastVisibility::Revealed
            } else {
                CastVisibility::Concealed
            };
            let member = Member::Pet(pet.kind());
            let baseline = rows.cast(
                if revealed {
                    revealed_line
                } else {
                    concealed_line
                },
                0,
                column,
                member,
                visibility,
            );
            if revealed {
                self.add_cast_sprites(&mut sprites, member, column, baseline);
            } else {
                sprites.push(FinaleSprite {
                    image: FinaleImage::BonusEgg(pet),
                    frame: FinaleSpriteFrame::WholeImage,
                    x: if column == Left { 150 } else { 390 },
                    y: baseline - 150,
                    visibility,
                    ..FinaleSprite::cell(member, FinaleImage::BonusEgg(pet), 0, 0, 0, 0)
                });
            }
        }
        let bubble_fish_start_scroll = rows.content_y as u32;
        rows.push(461, 7, Heading, Center, Credits);
        for (line, spacing, heading) in [
            (463, 3, true),
            (464, 1, false),
            (466, 3, true),
            (467, 1, false),
            (468, 1, false),
            (470, 3, true),
            (471, 1, false),
            (472, 1, false),
            (473, 1, false),
            (475, 3, true),
            (476, 1, false),
            (477, 1, false),
            (479, 3, true),
            (480, 1, false),
            (482, 3, true),
            (483, 1, false),
            (484, 1, false),
            (486, 3, true),
            (487, 1, false),
            (489, 3, true),
            (490, 1, false),
            (492, 3, true),
            (493, 1, false),
            (494, 1, false),
            (495, 1, false),
            (496, 1, false),
            (498, 3, true),
            (499, 1, false),
            (501, 3, true),
            (502, 1, false),
            (503, 1, false),
            (504, 1, false),
            (505, 1, false),
            (507, 7, false),
            (508, 1, true),
            (509, 1, false),
            (510, 1, true),
        ] {
            rows.push(
                line,
                spacing,
                if heading { Heading } else { Body },
                Center,
                Credits,
            );
        }

        FinalePresentation {
            background_rgba: [25, 105, 195, 255],
            clip: [0, 40, 640, 400],
            rows: rows.rows,
            sprites,
            bubble_fish_start_scroll,
        }
    }

    fn animation_frames(&self) -> (u8, u8, u8) {
        let phase = self.scroll % 20;
        (
            (phase / 2) as u8,
            ((self.scroll % 40) / 4) as u8,
            phase.min(19 - phase) as u8,
        )
    }

    fn add_cast_sprites(
        &self,
        sprites: &mut Vec<FinaleSprite>,
        member: CastMember,
        column: FinaleColumn,
        baseline: i32,
    ) {
        use CastMember as Member;
        use FinaleImage as Image;

        let (fast, slow, ping_pong) = self.animation_frames();
        if member == Member::Bilaterus {
            for (x, y_offset, frame, row) in [
                (255, -90, 9, 4),
                (225, -95, 2, 7),
                (195, -100, 2, 6),
                (165, -95, 2, 6),
                (135, -85, 2, 7),
                (105, -80, 2, 6),
                (75, -85, 2, 7),
                (45, -90, ping_pong, 2),
            ] {
                sprites.push(FinaleSprite::cell(
                    member,
                    Image::Bilaterus,
                    x,
                    baseline + y_offset,
                    frame,
                    row,
                ));
            }
            return;
        }
        let (image, x, y_offset, frame, row) = match member {
            Member::Guppy => (Image::SmallSwim, 160, -90, fast, 0),
            Member::Carnivore => (Image::SmallSwim, 400, -90, fast, 4),
            Member::Starcatcher => (Image::Starcatcher, 160, -90, fast, 0),
            Member::Guppycruncher => (Image::Grubber, 400, -100, fast, 0),
            Member::Beetlemuncher => (Image::Gekko, 160, -90, fast, 0),
            Member::Breeder => (Image::Breeder, 400, -90, fast, 6),
            Member::Ultravore => (Image::Ultra, 240, -170, fast, 0),
            Member::Sylvester => (Image::Sylvester, 120, -180, fast, 0),
            Member::Balrog => (Image::Balrog, 360, -180, fast, 0),
            Member::Gus => (Image::Gus, 120, -180, fast, 0),
            Member::Destructor => (Image::Destructor, 360, -180, fast, 0),
            Member::Ulysses => (Image::Ulysses, 120, -180, slow, 0),
            Member::Psychosquid => (Image::Psychosquid, 360, -165, fast, 0),
            Member::Cyrax => (Image::Boss, 360, -180, fast, 0),
            Member::Bilaterus => unreachable!("Bilaterus parts were handled above"),
            Member::Pet(kind) => {
                let x = if kind == PetKind::Amp {
                    120
                } else if column == FinaleColumn::Left {
                    160
                } else {
                    400
                };
                let y_offset = match kind {
                    PetKind::Niko | PetKind::Prego | PetKind::Zorf | PetKind::Meryl => -105,
                    PetKind::Clyde => -95,
                    PetKind::Shrapnel
                    | PetKind::Rhubarb
                    | PetKind::Gash
                    | PetKind::Angie
                    | PetKind::Brinkley
                    | PetKind::Nostradamus
                    | PetKind::Stanley
                    | PetKind::Walter => -100,
                    PetKind::Amp => -75,
                    _ => -90,
                };
                let frame = match kind {
                    PetKind::Niko | PetKind::Vert => ping_pong,
                    PetKind::Clyde | PetKind::Seymour | PetKind::Shrapnel | PetKind::Rhubarb => {
                        slow
                    }
                    _ => fast,
                };
                (Image::Pet(kind), x, y_offset, frame, 0)
            }
        };
        let sprite = FinaleSprite::cell(member, image, x, baseline + y_offset, frame, row);
        sprites.push(sprite);
        match member {
            Member::Pet(PetKind::Amp) => sprites.push(FinaleSprite {
                image: Image::AmpCharge,
                rgba: [255, 255, 200, self.halo_alpha()],
                blend: FinaleBlend::Additive,
                ..sprite
            }),
            Member::Pet(PetKind::Angie) => sprites.push(FinaleSprite {
                image: Image::Halo,
                rgba: [255, 255, 255, self.halo_alpha()],
                blend: FinaleBlend::Additive,
                ..sprite
            }),
            _ => {}
        }
    }

    fn halo_alpha(&self) -> u8 {
        (self.halo_phase.abs() * 255.0) as u8
    }

    fn return_sprites(&self) -> Vec<FinaleSprite> {
        let mut sprites = Vec::new();
        let scroll = self.scroll as i32;
        let (frame, _, _) = self.animation_frames();
        for (minimum_fallen, x, content_y, mirror) in [
            (1, 640 - (scroll - 680) * 3, 1_165, false),
            (6, (scroll - 935) * 3 - 80, 1_315, true),
            (11, 640 - (scroll - 1_190) * 3, 1_465, false),
        ] {
            if self.fallen.len() >= minimum_fallen {
                let sprite = FinaleSprite {
                    mirror,
                    ..FinaleSprite::cell(
                        CastMember::Pet(PetKind::Angie),
                        FinaleImage::Pet(PetKind::Angie),
                        x,
                        content_y - scroll,
                        frame,
                        0,
                    )
                };
                sprites.push(sprite);
                sprites.push(FinaleSprite {
                    image: FinaleImage::Halo,
                    rgba: [255, 255, 255, self.halo_alpha()],
                    blend: FinaleBlend::Additive,
                    ..sprite
                });
            }
        }
        for pose in self.resurrection_poses() {
            sprites.push(FinaleSprite {
                rgba: [255, 255, 255, pose.alpha],
                ..FinaleSprite::cell(
                    CastMember::Pet(pose.pet),
                    FinaleImage::Pet(pose.pet),
                    pose.x,
                    pose.y,
                    pose.frame,
                    0,
                )
            });
        }
        sprites
    }

    pub fn resurrection_poses(&self) -> Vec<ResurrectionPose> {
        if self.scroll >= PET_EFFECT_END {
            return Vec::new();
        }
        let scroll = self.scroll;
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(slot, pet)| {
                let pet = (*pet)?;
                let due = EFFECT_TIMES[slot];
                if scroll < due {
                    return None;
                }
                let frame = match pet {
                    PetKind::Niko | PetKind::Vert => {
                        let phase = scroll % 20;
                        phase.min(19 - phase) as u8
                    }
                    PetKind::Clyde | PetKind::Seymour | PetKind::Shrapnel | PetKind::Rhubarb => {
                        ((scroll % 40) / 4) as u8
                    }
                    _ => ((scroll % 20) / 2) as u8,
                };
                let mut x = PET_X[slot];
                let mut y = PET_Y[slot] - scroll as i32;
                if pet == PetKind::Niko {
                    y -= 20;
                }
                if pet == PetKind::Amp {
                    x -= 40;
                    y += 40;
                }
                Some(ResurrectionPose {
                    pet,
                    x,
                    y,
                    frame,
                    alpha: (scroll - due).min(255) as u8,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row_for(presentation: &FinalePresentation, line: u16) -> &FinaleRow {
        presentation
            .rows
            .iter()
            .find(|row| row.text.id().source_line() == line)
            .unwrap()
    }

    #[test]
    fn death_branches_preserve_source_spacing_and_common_cast_credit_cursor() {
        // Independent W1 Draw:279-321 expectations, including branch-boundary
        // counts. Cursor reset at line316 prevents conditional reflow.
        for (count, lines, first_y) in [
            (0, &[281, 282, 283, 284, 285, 286, 287][..], 1_215),
            (1, &[291, 292, 293, 294, 295][..], 1_390),
            (2, &[299, 300, 301, 302][..], 1_390),
            (3, &[299, 300, 301, 302][..], 1_390),
            (4, &[306, 307, 308][..], 1_390),
            (5, &[306, 307, 308][..], 1_390),
            (6, &[312, 313][..], 1_460),
            (10, &[312, 313][..], 1_460),
            (11, &[][..], 0),
            (18, &[][..], 0),
        ] {
            let controller = InterludeController::new(&TANK5_PETS[..count]).unwrap();
            let presentation = controller.presentation(|_| false);
            let conditional: Vec<_> = presentation
                .rows
                .iter()
                .filter(|row| row.section == FinaleSection::Survivors)
                .collect();
            assert_eq!(
                conditional
                    .iter()
                    .map(|row| row.text.id().source_line())
                    .collect::<Vec<_>>(),
                lines
            );
            if let Some(first) = conditional.first() {
                assert_eq!(first.y, first_y);
            }
            if (2..=3).contains(&count) {
                assert_eq!(
                    row_for(&presentation, 300).text,
                    FinaleText::DeathCount(count as u8)
                );
                assert_eq!(row_for(&presentation, 300).y, 1_425);
            }
            assert_eq!(row_for(&presentation, 177).y, 540);
            assert_eq!(row_for(&presentation, 186).y, 1_065);
            assert_eq!(row_for(&presentation, 318).y, 1_645);
            assert_eq!(row_for(&presentation, 321).y, 1_960);
            assert_eq!(row_for(&presentation, 461).y, 5_635);
            assert_eq!(row_for(&presentation, 510).y, 7_910);
            assert_eq!(presentation.bubble_fish_start_scroll, 5_390);
        }
    }

    #[test]
    fn full_cast_source_columns_frames_segments_and_credits_are_retained() {
        let mut controller = InterludeController::new(&[]).unwrap();
        controller.scroll = 37;
        let presentation = controller.presentation(|_| true);
        let cast: Vec<_> = presentation
            .rows
            .iter()
            .filter_map(|row| row.cast)
            .collect();
        assert_eq!(cast.len(), 39); // W1: seven fish, eight aliens, twenty-four pets.
        assert_eq!(cast[0], CastMember::Guppy);
        assert_eq!(cast[6], CastMember::Ultravore);
        assert_eq!(cast[7], CastMember::Sylvester);
        assert_eq!(cast[14], CastMember::Cyrax);
        assert_eq!(cast[15], CastMember::Pet(PetKind::Stinky));
        assert_eq!(cast[38], CastMember::Pet(PetKind::Walter));
        assert_eq!(
            cast.iter()
                .filter(|member| matches!(member, CastMember::Pet(_)))
                .count(),
            24
        );
        let guppy = row_for(&presentation, 322);
        let carnivore = row_for(&presentation, 324);
        assert_eq!((guppy.y, carnivore.y), (2_063, 2_063));
        assert_eq!(
            (guppy.column, carnivore.column),
            (FinaleColumn::Left, FinaleColumn::Right)
        );
        assert_eq!(FinaleColumn::Center.text_x(101), 274);
        assert_eq!(FinaleColumn::Left.text_x(101), 147);
        assert_eq!(FinaleColumn::Right.text_x(101), 392);
        assert_eq!(guppy.font.resource_key(), "FONT_LIDDIE15");
        assert_eq!(
            row_for(&presentation, 461).font.rgba(),
            [215, 255, 255, 255]
        );
        let parts: Vec<_> = presentation
            .sprites
            .iter()
            .filter(|sprite| sprite.member == CastMember::Bilaterus)
            .collect();
        assert_eq!(parts.len(), 8);
        assert_eq!((parts[0].x, parts[0].y), (255, 3_338));
        assert_eq!(
            parts[0].frame,
            FinaleSpriteFrame::Cell { column: 9, row: 4 }
        );
        assert_eq!(
            parts[7].frame,
            FinaleSpriteFrame::Cell { column: 2, row: 2 }
        );
        let breeder = presentation
            .sprites
            .iter()
            .find(|sprite| sprite.member == CastMember::Breeder)
            .unwrap();
        assert_eq!(breeder.frame, FinaleSpriteFrame::Cell { column: 8, row: 6 });
        let ulysses = presentation
            .sprites
            .iter()
            .find(|sprite| sprite.member == CastMember::Ulysses)
            .unwrap();
        assert_eq!(ulysses.frame, FinaleSpriteFrame::Cell { column: 9, row: 0 });
        let credits: Vec<_> = presentation
            .rows
            .iter()
            .filter(|row| row.section == FinaleSection::Credits)
            .collect();
        assert_eq!(credits.len(), 38);
        assert_eq!(
            credits
                .iter()
                .filter(|row| row.font == FinaleFont::Heading)
                .count(),
            14
        );
        assert_eq!(presentation.background_rgba, [25, 105, 195, 255]);
        assert_eq!(presentation.clip, [0, 40, 640, 400]);
    }

    #[test]
    fn each_bonus_identity_stays_concealed_until_its_own_unlock() {
        let controller = InterludeController::new(&[]).unwrap();
        for earned in [
            None,
            Some(PetKind::Brinkley),
            Some(PetKind::Stanley),
            Some(PetKind::Walter),
        ] {
            let mut queries = Vec::new();
            let presentation = controller.presentation(|pet| {
                queries.push(pet);
                earned == Some(pet)
            });
            assert_eq!(
                queries,
                [
                    PetKind::Brinkley,
                    PetKind::Nostradamus,
                    PetKind::Stanley,
                    PetKind::Walter
                ]
            );
            for (index, pet) in queries.into_iter().enumerate() {
                let row = presentation
                    .rows
                    .iter()
                    .find(|row| row.cast == Some(CastMember::Pet(pet)))
                    .unwrap();
                let revealed = earned == Some(pet);
                assert_eq!(
                    row.visibility,
                    if revealed {
                        CastVisibility::Revealed
                    } else {
                        CastVisibility::Concealed
                    }
                );
                assert_eq!(row.y, if index < 2 { 5_180 } else { 5_390 });
                assert_eq!(
                    row.font,
                    if revealed {
                        FinaleFont::Body
                    } else {
                        FinaleFont::Heading
                    }
                );
                let sprite = presentation
                    .sprites
                    .iter()
                    .find(|sprite| sprite.member == CastMember::Pet(pet))
                    .unwrap();
                assert_eq!(sprite.visibility, row.visibility);
                assert_eq!(sprite.frame == FinaleSpriteFrame::WholeImage, !revealed);
                assert_eq!(sprite.y, row.y - if revealed { 100 } else { 150 });
                if !revealed {
                    assert!(matches!(sprite.image, FinaleImage::BonusEgg(_)));
                }
            }
        }
        assert_eq!(controller.fallen(), &[]);
        assert_eq!(controller.scroll(), 0);
        assert_eq!(
            controller.visible_rows()[0].content,
            RowContent::Heading(CATALOG_UNAVAILABLE_MESSAGE)
        );
    }

    #[test]
    fn snapshot_focus_heal_music_and_exit_contracts_remain_once_only() {
        assert!(InterludeController::new(&[PetKind::Niko, PetKind::Niko]).is_err());
        assert!(InterludeController::new(&[PetKind::Angie]).is_err());
        let mut controller = InterludeController::new(&[PetKind::Gash, PetKind::Niko]).unwrap();
        assert_eq!(controller.fallen(), &[PetKind::Niko, PetKind::Gash]);
        assert_eq!(
            controller.advance(false),
            [InterludeEvent::StartInterludeMusic]
        );
        assert!(controller.advance(false).is_empty());
        assert_eq!(controller.scroll(), 0);
        assert_eq!(controller.halo_phase, 0.0);
        assert_eq!(controller.click_after_scroll(), None);
        let mut events = Vec::new();
        for _ in 0..=END_SCROLL {
            events.extend(
                controller
                    .advance(true)
                    .into_iter()
                    .map(|event| (controller.scroll(), event)),
            );
        }
        assert_eq!(
            events,
            [
                (773, InterludeEvent::Heal(PetKind::Niko)),
                (813, InterludeEvent::Heal(PetKind::Gash)),
                (3_001, InterludeEvent::CrossfadeToLullaby),
            ]
        );
        assert_eq!(controller.scroll(), END_SCROLL);
        assert_eq!(controller.music_stage(), MusicStage::Lullaby);
        assert!(controller.resurrection_poses().is_empty());
        assert_eq!(
            controller.click_after_scroll(),
            Some(InterludeEvent::BackToMenu)
        );
        assert_eq!(controller.back_to_menu(), None);
        assert!(controller.advance(true).is_empty());
        assert_eq!(controller.music_stage(), MusicStage::Stopped);
        assert!(controller.exited());
        let mut early_exit = InterludeController::new(&[]).unwrap();
        assert_eq!(early_exit.back_to_menu(), Some(InterludeEvent::BackToMenu));
        assert_eq!(early_exit.click_after_scroll(), None);
    }

    #[test]
    fn return_layers_use_snapshot_thresholds_and_frozen_focus_pulse() {
        let mut controller = InterludeController::new(&TANK5_PETS[..11]).unwrap();
        for _ in 0..1_100 {
            controller.advance(true);
        }
        let before = controller.presentation(|_| false);
        controller.advance(false);
        assert_eq!(controller.presentation(|_| false), before);
        let halo_passes: Vec<_> = before
            .sprites
            .iter()
            .filter(|sprite| sprite.image == FinaleImage::Halo && sprite.y < 400)
            .collect();
        assert_eq!(halo_passes.len(), 3);
        assert_eq!(
            halo_passes
                .iter()
                .map(|sprite| (sprite.x, sprite.y, sprite.mirror))
                .collect::<Vec<_>>(),
            [(-620, 65, false), (415, 215, true), (910, 365, false)]
        );
        assert!(
            halo_passes
                .iter()
                .all(|sprite| sprite.blend == FinaleBlend::Additive)
        );
        let poses = controller.resurrection_poses();
        assert_eq!(poses.len(), 9); // Slots 0..8 have reached their source cue.
        let niko = poses.iter().find(|pose| pose.pet == PetKind::Niko).unwrap();
        assert_eq!(niko.y, 25); // 1145 - scroll1100 - Niko20.
        assert_eq!(niko.alpha, 255);
    }
}
