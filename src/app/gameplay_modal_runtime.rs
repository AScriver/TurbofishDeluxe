//! Caller-owned adapter for gameplay prompts. No domain, clock or save writes.

use super::{Presentation, modal_runtime};
use crate::{
    adventure::AdventureSession,
    gameplay_modal::{
        GameplayModal, GameplayModalAction, GameplayModalFeedback, GameplayModalIntent,
        GameplayModalState, GameplayModalView,
    },
    modal::ModalInput,
    sim::Action,
};

#[derive(Default)]
pub(super) struct GameplayModalRuntime {
    modal: Option<GameplayModal>,
    generation: u64,
    last_session_tick: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{adventure::AdventurePhase, invasion::InvasionTip};

    fn press(runtime: &mut GameplayModalRuntime) {
        let pointer = runtime.view().unwrap().buttons[0].control.bounds.center();
        assert!(
            runtime
                .input(&ModalInput {
                    pointer,
                    focused: true,
                    pressed: true,
                    down: true,
                    ..ModalInput::default()
                })
                .is_none()
        );
    }

    fn release(runtime: &mut GameplayModalRuntime) -> Option<GameplayModalIntent> {
        let pointer = runtime.view().unwrap().buttons[0].control.bounds.center();
        runtime.input(&ModalInput {
            pointer,
            focused: true,
            released: true,
            ..ModalInput::default()
        })
    }

    #[test]
    fn runtime_revalidates_current_phase_and_retires_old_capture() {
        let mut session = AdventureSession::new(42);
        session.phase = AdventurePhase::InvasionTutorial {
            tip: InvasionTip::Danger,
        };
        let mut runtime = GameplayModalRuntime::default();
        runtime.sync(&session, 34.0);
        assert!(runtime.is_active());
        press(&mut runtime);
        session.phase = AdventurePhase::InvasionTutorial {
            tip: InvasionTip::GusWarning,
        };
        runtime.sync(&session, 34.0);
        assert!(release(&mut runtime).is_none());
        let intent = runtime.request(GameplayModalAction::Continue).unwrap();
        assert_eq!(
            runtime.action_for(&intent, &session),
            Some(Action::Continue)
        );
        session.phase = AdventurePhase::TimeTrialInvasionTutorial {
            tip: InvasionTip::GusWarning,
        };
        assert!(runtime.action_for(&intent, &session).is_none());
        runtime.sync(&session, 34.0);
        assert!(
            !runtime
                .transaction_result(&intent, GameplayModalFeedback::Rejected("stale".to_owned()))
        );
        session.phase = AdventurePhase::Playing;
        runtime.sync(&session, 34.0);
        assert!(!runtime.is_active());
        assert!(runtime.view().is_none());
    }

    #[test]
    fn runtime_keeps_capture_for_same_phase_and_increments_identity_on_replacement() {
        let mut session = AdventureSession::new(42);
        session.ticks = 80;
        session.phase = AdventurePhase::GameOver { updates: 31 };
        let mut runtime = GameplayModalRuntime::default();
        runtime.sync(&session, 34.0);
        press(&mut runtime);
        session.ticks += 1;
        session.phase = AdventurePhase::GameOver { updates: 32 };
        runtime.sync(&session, 34.0);
        let first = release(&mut runtime).unwrap();
        assert!(
            runtime.transaction_result(&first, GameplayModalFeedback::Rejected("retry".to_owned()))
        );
        runtime.clear();
        runtime.sync(&session, 34.0);
        let second = runtime.request(GameplayModalAction::Continue).unwrap();
        assert_ne!(first.generation, second.generation);
        assert!(!runtime.transaction_result(&first, GameplayModalFeedback::Applied));
        session.ticks = 0;
        runtime.sync(&session, 34.0);
        let third = runtime.request(GameplayModalAction::Continue).unwrap();
        assert_ne!(second.generation, third.generation);
        assert!(runtime.action_for(&second, &session).is_none());
    }

    #[test]
    fn higher_priority_modal_cancels_underlying_press() {
        let mut session = AdventureSession::new(42);
        session.phase = AdventurePhase::FirstTankRescue;
        let mut runtime = GameplayModalRuntime::default();
        runtime.sync(&session, 34.0);
        press(&mut runtime);
        runtime.cancel_capture();
        assert!(release(&mut runtime).is_none());
        assert!(runtime.is_active());
    }
}

impl GameplayModalRuntime {
    /// Call before input/drawing on every frame, including non-modal phases.
    /// This retires old captures and keeps the domain's admission current.
    pub(super) fn sync(&mut self, session: &AdventureSession, button_height: f32) {
        let rolled_back = self
            .last_session_tick
            .is_some_and(|previous| session.ticks < previous);
        self.last_session_tick = Some(session.ticks);
        let Some(state) = GameplayModalState::from_session(session) else {
            self.modal = None;
            return;
        };
        if !rolled_back
            && self
                .modal
                .as_mut()
                .is_some_and(|modal| modal.refresh(&state))
        {
            return;
        }
        let Some(generation) = self.generation.checked_add(1) else {
            self.modal = None;
            return;
        };
        self.generation = generation;
        self.modal = Some(GameplayModal::new(generation, state, button_height));
    }

    pub(super) fn is_active(&self) -> bool {
        self.modal.is_some()
    }

    pub(super) fn input(&mut self, input: &ModalInput) -> Option<GameplayModalIntent> {
        self.modal.as_mut()?.input(input)
    }

    /// Root preserves existing Y/N aliases without bypassing the controller.
    pub(super) fn request(&mut self, action: GameplayModalAction) -> Option<GameplayModalIntent> {
        self.modal.as_mut()?.request(action)
    }

    pub(super) fn action_for(
        &self,
        intent: &GameplayModalIntent,
        session: &AdventureSession,
    ) -> Option<Action> {
        self.modal
            .as_ref()?
            .action_for(intent, &GameplayModalState::from_session(session)?)
    }

    pub(super) fn transaction_result(
        &mut self,
        intent: &GameplayModalIntent,
        feedback: GameplayModalFeedback,
    ) -> bool {
        self.modal
            .as_mut()
            .is_some_and(|modal| modal.transaction_result(intent, feedback))
    }

    pub(super) fn view(&self) -> Option<GameplayModalView> {
        self.modal.as_ref().map(GameplayModal::view)
    }

    /// Higher-priority UI/focus loss must retire any behind-modal press.
    pub(super) fn cancel_capture(&mut self) {
        if let Some(modal) = &mut self.modal {
            modal.cancel_capture();
        }
    }

    /// Root calls on profile/session replacement even if ticks/phase coincide.
    /// Keep generation monotonic so old request feedback remains invalid.
    pub(super) fn clear(&mut self) {
        self.modal = None;
        self.last_session_tick = None;
    }

    /// The parent draws its dim layer once. Every text/control helper receives
    /// the same view rectangle used by the pure controller.
    pub(super) fn draw(&self, presentation: &Presentation) {
        let Some(view) = self.view() else { return };
        modal_runtime::draw_panel(presentation, view.panel);
        modal_runtime::draw_title(presentation, view.title, view.title_bounds);
        modal_runtime::draw_body(presentation, &view.body, view.body_bounds, true);
        if let Some(message) = &view.feedback {
            modal_runtime::draw_body(presentation, message, view.feedback_bounds, true);
        }
        for button in &view.buttons {
            modal_runtime::draw_button(
                presentation,
                button.control.bounds,
                button.label,
                button.control.enabled,
                button.pressed,
                button.focused,
            );
        }
    }
}
