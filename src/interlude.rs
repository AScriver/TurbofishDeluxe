//! Transient Adventure ending presentation model.
//!
//! Scroll, focus, music and pet-return timing follow pinned W1
//! `InterludeScreen.cpp` (secondary source). Text rows below are short project
//! summaries: retail narration, artwork and audible fades are not established.

use crate::sim::{PetKind, TANK5_PETS};

pub const END_SCROLL: u32 = 7_645;
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

pub struct InterludeController {
    fallen: Vec<PetKind>,
    slots: [Option<PetKind>; PET_SLOTS],
    update_calls: u32,
    scroll: u32,
    music: MusicStage,
    exited: bool,
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

    /// Rows lie in 640x480 screen coordinates. The app owns fonts, images,
    /// clipping, and exact wording. This deliberately does not copy the
    /// lengthy proprietary retail narrative or credits prose.
    pub fn visible_rows(&self) -> Vec<InterludeRow> {
        let mut rows = Vec::new();
        for &(content_y, content) in &[
            (540, RowContent::Heading("Adventure complete")),
            (610, RowContent::Text("The final battle is over.")),
            (785, RowContent::Text("The aquarium is safe.")),
            (925, RowContent::Text("Some pets fell in the battle.")),
            (1_030, RowContent::Text("Angie brings them home.")),
            (1_480, RowContent::Heading("Thanks for playing")),
            (1_620, RowContent::Heading("Cast")),
            (3_640, RowContent::Heading("Credits")),
            (3_990, RowContent::Text("Game design and programming")),
            (4_550, RowContent::Text("Art and character design")),
            (5_110, RowContent::Text("Music and sound")),
            (5_670, RowContent::Text("Production and quality assurance")),
            (6_230, RowContent::Text("Special thanks")),
            (6_790, RowContent::Text("Flying Bear Entertainment")),
            (7_350, RowContent::Text("PopCap Games")),
        ] {
            let y = content_y - self.scroll as i32;
            if (40..=440).contains(&y) {
                rows.push(InterludeRow { content, x: 320, y });
            }
        }
        for (index, &pet) in [
            PetKind::Stinky,
            PetKind::Niko,
            PetKind::Itchy,
            PetKind::Prego,
            PetKind::Zorf,
            PetKind::Clyde,
            PetKind::Vert,
            PetKind::Rufus,
            PetKind::Meryl,
            PetKind::Wadsworth,
            PetKind::Seymour,
            PetKind::Shrapnel,
            PetKind::Gumbo,
            PetKind::Blip,
            PetKind::Rhubarb,
            PetKind::Nimbus,
            PetKind::Amp,
            PetKind::Gash,
            PetKind::Angie,
            PetKind::Presto,
            PetKind::Brinkley,
            PetKind::Nostradamus,
            PetKind::Stanley,
            PetKind::Walter,
        ]
        .iter()
        .enumerate()
        {
            let y = 1_770 + (index / 2) as i32 * 140 - self.scroll as i32;
            if (40..=440).contains(&y) {
                rows.push(InterludeRow {
                    content: RowContent::Cast(pet),
                    x: if index % 2 == 0 { 160 } else { 400 },
                    y,
                });
            }
        }
        rows
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
