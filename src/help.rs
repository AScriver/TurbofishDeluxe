//! App-lived Help navigation. Game progress and the Help return target belong to the caller.
//!
//! The page split and navigation follow pinned WinFish HelpScreen.cpp (constructor,
//! Draw, and ButtonDepress). The short guidance below is written for this runtime.

pub const LAST_PAGE: u8 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HelpReturn {
    Selector,
    PausedTank,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HelpExit {
    Continue,
    ReturnTo(HelpReturn),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HelpView {
    /// Page 0 is the first-play instruction screen; ordinary Help is pages 1–8.
    pub page: u8,
    pub title: &'static str,
    pub lines: &'static [&'static str],
    pub return_to: HelpReturn,
}

impl HelpView {
    pub fn is_instructions(self) -> bool {
        self.page == 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HelpSession {
    page: u8,
    return_to: HelpReturn,
}

/// Keep this controller alive with the app, not with a save or individual tank.
#[derive(Clone, Debug)]
pub struct HelpController {
    active: Option<HelpSession>,
    last_ordinary_page: u8,
}

impl Default for HelpController {
    fn default() -> Self {
        Self {
            active: None,
            last_ordinary_page: 1,
        }
    }
}

impl HelpController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn remembered_page(&self) -> u8 {
        self.last_ordinary_page
    }

    pub fn open_instructions(&mut self, return_to: HelpReturn) {
        self.remember_current_page();
        self.active = Some(HelpSession { page: 0, return_to });
    }

    pub fn open_pages(&mut self, return_to: HelpReturn) {
        self.remember_current_page();
        self.active = Some(HelpSession {
            page: self.last_ordinary_page,
            return_to,
        });
    }

    pub fn view(&self) -> Option<HelpView> {
        let session = self.active?;
        let (title, lines) = page_content(session.page);
        Some(HelpView {
            page: session.page,
            title,
            lines,
            return_to: session.return_to,
        })
    }

    /// Hidden on the instruction page, as in the source Help screen.
    pub fn next_page(&mut self) -> bool {
        let Some(session) = self.active.as_mut() else {
            return false;
        };
        if session.page == 0 {
            return false;
        }
        session.page = if session.page == LAST_PAGE {
            1
        } else {
            session.page + 1
        };
        true
    }

    /// Hidden on the instruction page, as in the source Help screen.
    pub fn previous(&mut self) -> bool {
        let Some(session) = self.active.as_mut() else {
            return false;
        };
        if session.page == 0 {
            return false;
        }
        session.page = if session.page == 1 {
            LAST_PAGE
        } else {
            session.page - 1
        };
        true
    }

    /// Select an ordinary tab. Invalid page numbers and the instruction page do nothing.
    pub fn select_page(&mut self, page: u8) -> bool {
        if !(1..=LAST_PAGE).contains(&page) {
            return false;
        }
        let Some(session) = self.active.as_mut() else {
            return false;
        };
        if session.page == 0 {
            return false;
        }
        session.page = page;
        true
    }

    /// The instruction Continue button starts the caller's first game flow.
    pub fn continue_intro(&mut self) -> Option<HelpExit> {
        if self.active?.page != 0 {
            return None;
        }
        self.active = None;
        Some(HelpExit::Continue)
    }

    /// Return to the screen that opened Help. The caller retains the paused tank.
    pub fn close(&mut self) -> Option<HelpExit> {
        let session = self.active.take()?;
        if session.page != 0 {
            self.last_ordinary_page = session.page;
        }
        Some(HelpExit::ReturnTo(session.return_to))
    }

    fn remember_current_page(&mut self) {
        if let Some(session) = self.active
            && session.page != 0
        {
            self.last_ordinary_page = session.page;
        }
    }
}

fn page_content(page: u8) -> (&'static str, &'static [&'static str]) {
    match page {
        0 => (
            "Instructions",
            &[
                "Click in the tank to feed your fish.",
                "Collect coins to afford fish and upgrades.",
                "Protect the tank when aliens arrive.",
                "Continue to begin your first game.",
            ],
        ),
        1 => (
            "The Basics",
            &[
                "Feed fish before they go hungry; growing fish earn better coins.",
                "Click coins to add their value to your tank funds.",
                "Click aliens to fire your weapon and defend your fish.",
                "Buy three egg pieces to finish an Adventure level.",
            ],
        ),
        2 => (
            "Upgrades",
            &[
                "The top bar offers fish, food, weapons, and egg pieces.",
                "Food quality improves each feeding; quantity raises the drop limit.",
                "Weapon upgrades help against tougher alien attacks.",
                "A purchase needs enough tank funds and an unlocked button.",
            ],
        ),
        3 => (
            "Pets",
            &[
                "Adventure rewards new pets as you complete levels.",
                "Each pet helps in a different way, such as collecting coins or fighting.",
                "Choose which available pets accompany you into a tank.",
            ],
        ),
        4 => (
            "Game Modes",
            &[
                "Adventure advances through tanks and unlocks pets.",
                "Time Trial rewards the funds you earn before time runs out.",
                "Challenge adds rising prices and harder invasions.",
                "Virtual Tank lets you care for fish using shells and its store.",
            ],
        ),
        5 => (
            "Virtual Tank",
            &[
                "Use shells to buy fish and items for your aquarium.",
                "Visit the store to see its current stock.",
                "Name and care for the fish you keep there.",
            ],
        ),
        6 => (
            "Virtual Tank Fish",
            &[
                "Virtual Tank fish keep their own feeding and growth progress.",
                "Drop ordinary food in the tank or use Feed for special diets.",
                "Check each fish's needs when planning what to buy.",
            ],
        ),
        7 => (
            "Configuring Virtual Tank",
            &[
                "Fish controls show your collection and its display choices.",
                "Pets controls choose which helpers appear in the tank.",
                "Tank controls change the backdrop and other display options.",
            ],
        ),
        8 => (
            "Credits",
            &[
                "Insaniquarium Deluxe is the original game.",
                "Turbofish Deluxe is an independent Rust runtime.",
                "Artwork and sound load from your game installation.",
            ],
        ),
        _ => unreachable!("HelpController stores only pages 0 through 8"),
    }
}
