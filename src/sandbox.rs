//! Sandbox admission and keyboard actions. PB05 005177e0/00541a20 establish
//! the cursor gate, capacity floor and setup values. Key mappings follow W1
//! f919b3c GameSelector::SandboxActivated and Board::KeyChar. Relax is a
//! separate configuration; its retail activation is still unverified.

use crate::{alien::SylvesterKind, invasion::EncounterKind, sim::PetKind};

pub const CAPACITY_FLOOR: u8 = 11;

pub fn admitted(purchase_cursor: u8) -> bool {
    purchase_cursor > 5
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectorKey {
    Up,
    Down,
    Left,
    Right,
    B,
    A,
    Other,
}

const ENTRY_SEQUENCE: [SelectorKey; 10] = [
    SelectorKey::Up,
    SelectorKey::Up,
    SelectorKey::Down,
    SelectorKey::Down,
    SelectorKey::Left,
    SelectorKey::Right,
    SelectorKey::Left,
    SelectorKey::Right,
    SelectorKey::B,
    SelectorKey::A,
];

/// Transient selector input, never part of a profile or resumable Board.
#[derive(Default)]
pub struct EntryCode {
    matched: usize,
}

impl EntryCode {
    pub fn push(&mut self, key: SelectorKey) -> bool {
        // Preserve matching suffixes so an extra leading Up does not make
        // the next full sequence unreachable. This is project input policy.
        let mut prefix = [SelectorKey::Other; 11];
        prefix[..self.matched].copy_from_slice(&ENTRY_SEQUENCE[..self.matched]);
        prefix[self.matched] = key;
        let length = self.matched + 1;
        self.matched = (1..=length.min(ENTRY_SEQUENCE.len()))
            .rev()
            .find(|&candidate| prefix[length - candidate..length] == ENTRY_SEQUENCE[..candidate])
            .unwrap_or(0);
        if self.matched == ENTRY_SEQUENCE.len() {
            self.matched = 0;
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.matched = 0;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FishSpawn {
    Guppy,
    Oscar,
    StarGuppy,
    Penta,
    Grubber,
    Gekko,
    Breeder,
    Ultra,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Fish(FishSpawn),
    Alien(EncounterKind),
    Pet(PetKind),
    Background(i8),
}

/// W1 tests only the first Alien-list member for Gus before accepting other
/// enemy shortcuts. The caller supplies that list-order fact explicitly.
pub fn command(character: char, first_alien_is_gus: bool) -> Option<Command> {
    use Command::{Alien, Background, Fish, Pet};
    use FishSpawn::*;
    use PetKind::*;
    let enemy = match character {
        'z' => Some(EncounterKind::Single(SylvesterKind::Strong)),
        'x' => Some(EncounterKind::Single(SylvesterKind::Balrog)),
        'c' => return Some(Alien(EncounterKind::Single(SylvesterKind::Gus))),
        'v' => Some(EncounterKind::Single(SylvesterKind::Destructor)),
        'b' => Some(EncounterKind::Single(SylvesterKind::Ulysses)),
        'n' => Some(EncounterKind::Single(SylvesterKind::Psychosquid)),
        'm' => Some(EncounterKind::Bilaterus),
        _ => None,
    };
    if let Some(kind) = enemy {
        return (!first_alien_is_gus).then_some(Alien(kind));
    }
    Some(match character {
        '1' => Fish(Guppy),
        '2' => Fish(Oscar),
        '3' => Fish(StarGuppy),
        '4' => Fish(Penta),
        '5' => Fish(Grubber),
        '6' => Fish(Gekko),
        '7' => Fish(Breeder),
        '8' => Fish(Ultra),
        'q' => Pet(Stinky),
        'w' => Pet(Niko),
        'e' => Pet(Itchy),
        'r' => Pet(Prego),
        't' => Pet(Zorf),
        'y' => Pet(Clyde),
        'u' => Pet(Vert),
        'i' => Pet(Rufus),
        'o' => Pet(Meryl),
        'p' => Pet(Wadsworth),
        'a' => Pet(Seymour),
        's' => Pet(Shrapnel),
        'd' => Pet(Gumbo),
        'f' => Pet(Blip),
        'g' => Pet(Rhubarb),
        'h' => Pet(Nimbus),
        'j' => Pet(Amp),
        'k' => Pet(Gash),
        'l' => Pet(Angie),
        ';' => Pet(Presto),
        'A' => Pet(Brinkley),
        'S' => Pet(Nostradamus),
        'D' => Pet(Walter),
        'F' => Pet(Stanley),
        '+' => Background(1),
        '-' => Background(-1),
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InitialValues {
    pub cash: i32,
    pub food_limit: u8,
    pub weapon_level: u8,
}

pub fn initial_values(relax: bool) -> InitialValues {
    if relax {
        InitialValues {
            cash: 200,
            food_limit: 3,
            weapon_level: 8,
        }
    } else {
        InitialValues {
            cash: 999_999,
            food_limit: 1,
            weapon_level: 2,
        }
    }
}
