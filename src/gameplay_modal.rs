//! Transient gameplay prompts; the session retains every outcome and clock.
//!
//! Layout adapts functional W1 dialog facts at f919b3c (see the owning report).
//! Guidance is project-authored; no retail dialogue or independent timer is loaded.

use crate::{
    adventure::{AdventurePhase, AdventureSession},
    invasion::InvasionTip,
    modal::{ModalButton, ModalControls, ModalInput},
    sim::{Action, PetKind},
};
use macroquad::prelude::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameplayMode {
    Adventure,
    TimeTrial,
    Challenge,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameplayPrompt {
    PetConfirmation {
        selected: Vec<PetKind>,
        capacity: usize,
    },
    FirstFishRescue,
    InvasionTutorial {
        tip: InvasionTip,
    },
    GameOver,
}

/// An immutable projection of the authoritative phase, including its payload.
/// `can_accept` is admission, not a modal-owned countdown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameplayModalState {
    pub mode: GameplayMode,
    pub tank: u8,
    pub level: u8,
    pub prompt: GameplayPrompt,
    pub can_accept: bool,
}

impl GameplayModalState {
    pub fn from_session(session: &AdventureSession) -> Option<Self> {
        let (mut tank, mut level) = session
            .board
            .as_ref()
            .map_or((session.progress.tank, session.progress.level), |board| {
                (board.tank, board.level)
            });
        let (mode, prompt, can_accept) = match &session.phase {
            AdventurePhase::PetSelectionConfirmation { selected } => {
                tank = session.progress.tank;
                level = session.progress.level;
                (
                    GameplayMode::Adventure,
                    GameplayPrompt::PetConfirmation {
                        selected: selected.clone(),
                        capacity: session.progress.selection_capacity(),
                    },
                    (tank, level) != (5, 1),
                )
            }
            AdventurePhase::ChallengePetSelectionConfirmation {
                tank: selected_tank,
                selected,
            } => {
                tank = *selected_tank;
                level = 0;
                (
                    GameplayMode::Challenge,
                    GameplayPrompt::PetConfirmation {
                        selected: selected.clone(),
                        capacity: session.progress.selection_capacity(),
                    },
                    true,
                )
            }
            AdventurePhase::FirstTankRescue => (
                GameplayMode::Adventure,
                GameplayPrompt::FirstFishRescue,
                true,
            ),
            AdventurePhase::InvasionTutorial { tip } => (
                GameplayMode::Adventure,
                GameplayPrompt::InvasionTutorial { tip: *tip },
                true,
            ),
            AdventurePhase::TimeTrialInvasionTutorial { tip } => (
                GameplayMode::TimeTrial,
                GameplayPrompt::InvasionTutorial { tip: *tip },
                true,
            ),
            AdventurePhase::ChallengeInvasionTutorial { tip } => (
                GameplayMode::Challenge,
                GameplayPrompt::InvasionTutorial { tip: *tip },
                true,
            ),
            AdventurePhase::GameOver { updates } => (
                GameplayMode::Adventure,
                GameplayPrompt::GameOver,
                *updates > 30,
            ),
            AdventurePhase::TimeTrialGameOver { updates } => (
                GameplayMode::TimeTrial,
                GameplayPrompt::GameOver,
                *updates > 30,
            ),
            _ => return None,
        };
        Some(Self {
            mode,
            tank,
            level,
            prompt,
            can_accept,
        })
    }

    /// Admission may change while the same dialog is open. Payload may not.
    pub fn same_phase(&self, other: &Self) -> bool {
        self.mode == other.mode
            && self.tank == other.tank
            && self.level == other.level
            && self.prompt == other.prompt
    }

    fn permits(&self, action: GameplayModalAction) -> bool {
        match (&self.prompt, action) {
            (GameplayPrompt::PetConfirmation { .. }, GameplayModalAction::ConfirmPets) => {
                self.can_accept
            }
            (GameplayPrompt::PetConfirmation { .. }, GameplayModalAction::ChooseMorePets) => true,
            (GameplayPrompt::PetConfirmation { .. }, GameplayModalAction::Continue) => false,
            (_, GameplayModalAction::Continue) => self.can_accept,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameplayModalAction {
    Continue,
    ConfirmPets,
    ChooseMorePets,
}

impl GameplayModalAction {
    pub const fn id(self) -> u64 {
        match self {
            Self::Continue => 1,
            Self::ConfirmPets => 2,
            Self::ChooseMorePets => 3,
        }
    }
}

/// Generation belongs to the transient caller, not a persisted game format.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameplayModalIntent {
    pub generation: u64,
    pub request: u64,
    pub state: GameplayModalState,
    pub action: GameplayModalAction,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameplayModalFeedback {
    Applied,
    Rejected(String),
}

#[derive(Clone, Debug)]
pub struct GameplayButtonView {
    pub control: ModalButton,
    pub action: GameplayModalAction,
    pub label: &'static str,
    pub pressed: bool,
    pub focused: bool,
}

/// Drawing and hit testing consume the very same control rectangles.
#[derive(Clone, Debug)]
pub struct GameplayModalView {
    pub panel: Rect,
    pub title_bounds: Rect,
    pub body_bounds: Rect,
    pub feedback_bounds: Rect,
    pub title: &'static str,
    pub body: String,
    pub feedback: Option<String>,
    pub buttons: Vec<GameplayButtonView>,
    pub pending: bool,
}

pub struct GameplayModal {
    generation: u64,
    state: GameplayModalState,
    button_height: f32,
    controls: ModalControls,
    pending: Option<GameplayModalIntent>,
    next_request: u64,
    feedback: Option<String>,
}

impl GameplayModal {
    pub fn new(generation: u64, state: GameplayModalState, button_height: f32) -> Self {
        let button_height = if button_height.is_finite() {
            button_height.clamp(16.0, 40.0)
        } else {
            34.0
        };
        let default_action = match state.prompt {
            GameplayPrompt::PetConfirmation { .. } if state.can_accept => {
                GameplayModalAction::ConfirmPets
            }
            GameplayPrompt::PetConfirmation { .. } => GameplayModalAction::ChooseMorePets,
            _ => GameplayModalAction::Continue,
        };
        let mut controls = ModalControls::default();
        controls.set_focus(Some(default_action.id()));
        Self {
            generation,
            state,
            button_height,
            controls,
            pending: None,
            next_request: 1,
            feedback: None,
        }
    }

    pub fn state(&self) -> &GameplayModalState {
        &self.state
    }

    /// Return false on changed mode/stage/payload so the caller replaces this
    /// instance, retires capture and issues a fresh generation.
    pub fn refresh(&mut self, current: &GameplayModalState) -> bool {
        if !self.state.same_phase(current) {
            return false;
        }
        if self.state.can_accept && !current.can_accept {
            self.controls.cancel_capture();
        }
        self.state.can_accept = current.can_accept;
        true
    }

    pub fn cancel_capture(&mut self) {
        self.controls.cancel_capture();
    }

    pub fn view(&self) -> GameplayModalView {
        // Fixed virtual 640x480 layout; W1-informed 36px side/footer insets.
        // Its dimensions are project-authored, not a retail pixel claim.
        let panel = Rect::new(96.0, 110.0, 448.0, 272.0);
        let footer = Rect::new(132.0, 346.0 - self.button_height, 376.0, self.button_height);
        let (title, body, entries) = match &self.state.prompt {
            GameplayPrompt::PetConfirmation { selected, capacity } => (
                "Continue with Fewer Pets?",
                format!(
                    "You selected {} of {capacity} pets.\nContinue with this selection?",
                    selected.len()
                ),
                vec![
                    (
                        GameplayModalAction::ConfirmPets,
                        "Yes",
                        Rect::new(footer.x, footer.y, 180.0, footer.h),
                    ),
                    (
                        GameplayModalAction::ChooseMorePets,
                        "No",
                        Rect::new(328.0, footer.y, 180.0, footer.h),
                    ),
                ],
            ),
            GameplayPrompt::FirstFishRescue => (
                "Your Last Fish Has Died!",
                "We'll give you another fish to keep playing.\nMake sure you keep it well fed!"
                    .to_owned(),
                vec![(GameplayModalAction::Continue, "Click to Continue", footer)],
            ),
            GameplayPrompt::InvasionTutorial { tip } => {
                let (title, body) = match tip {
                    InvasionTip::Danger => (
                        "DANGER!",
                        "Aliens are approaching your tank!\nClick on them to defend your fish.",
                    ),
                    InvasionTip::BattleTip => (
                        "ALIEN ATTACK!",
                        "Click an alien repeatedly to defeat it.\nKeep your fish safe!",
                    ),
                    InvasionTip::GusWarning => (
                        "WARNING!",
                        "Lasers cannot hurt this alien.\nClick in the tank to feed it instead.",
                    ),
                };
                (
                    title,
                    body.to_owned(),
                    vec![(GameplayModalAction::Continue, "Click to Continue", footer)],
                )
            }
            GameplayPrompt::GameOver => (
                "GAME OVER",
                "Continue to return to the menu.".to_owned(),
                vec![(GameplayModalAction::Continue, "Continue", footer)],
            ),
        };
        let buttons = entries
            .into_iter()
            .map(|(action, label, bounds)| GameplayButtonView {
                control: ModalButton {
                    id: action.id(),
                    bounds,
                    enabled: self.pending.is_none() && self.state.permits(action),
                },
                action,
                label,
                pressed: self.controls.is_pressed(action.id()),
                focused: self.controls.is_focused(action.id()),
            })
            .collect();
        GameplayModalView {
            panel,
            title_bounds: Rect::new(132.0, 129.0, 376.0, 42.0),
            body_bounds: Rect::new(132.0, 183.0, 376.0, 82.0),
            feedback_bounds: Rect::new(132.0, 269.0, 376.0, 29.0),
            title,
            body,
            feedback: self.feedback.clone(),
            buttons,
            pending: self.pending.is_some(),
        }
    }

    pub fn input(&mut self, input: &ModalInput) -> Option<GameplayModalIntent> {
        let controls = self
            .view()
            .buttons
            .into_iter()
            .map(|button| button.control)
            .collect::<Vec<_>>();
        let activated = self.controls.update(input, &controls);
        if !input.focused {
            return None;
        }
        if input.cancel {
            return self.request(GameplayModalAction::ChooseMorePets);
        }
        let action = match activated? {
            1 => GameplayModalAction::Continue,
            2 => GameplayModalAction::ConfirmPets,
            3 => GameplayModalAction::ChooseMorePets,
            _ => return None,
        };
        self.request(action)
    }

    /// Semantic Y/N shortcuts still pass through admission and pending guards.
    pub fn request(&mut self, action: GameplayModalAction) -> Option<GameplayModalIntent> {
        if self.pending.is_some() || !self.state.permits(action) {
            return None;
        }
        let request = self.next_request;
        self.next_request = request.checked_add(1)?;
        let intent = GameplayModalIntent {
            generation: self.generation,
            request,
            state: self.state.clone(),
            action,
        };
        self.controls.cancel_capture();
        self.feedback = None;
        self.pending = Some(intent.clone());
        Some(intent)
    }

    /// Parent revalidates against the current session immediately before
    /// applying this action; this function never mutates the game.
    pub fn action_for(
        &self,
        intent: &GameplayModalIntent,
        current: &GameplayModalState,
    ) -> Option<Action> {
        if self.pending.as_ref() != Some(intent)
            || !intent.state.same_phase(current)
            || !current.permits(intent.action)
        {
            return None;
        }
        Some(match intent.action {
            GameplayModalAction::Continue => Action::Continue,
            GameplayModalAction::ConfirmPets => Action::ConfirmPetSelection { accept: true },
            GameplayModalAction::ChooseMorePets => Action::ConfirmPetSelection { accept: false },
        })
    }

    /// Late feedback for an old request cannot clear a newer request. Applied
    /// stays pending until the parent observes the real phase transition.
    pub fn transaction_result(
        &mut self,
        intent: &GameplayModalIntent,
        feedback: GameplayModalFeedback,
    ) -> bool {
        if self.pending.as_ref() != Some(intent) {
            return false;
        }
        if let GameplayModalFeedback::Rejected(message) = feedback {
            self.pending = None;
            self.feedback = Some(message);
        }
        self.controls.cancel_capture();
        true
    }
}
