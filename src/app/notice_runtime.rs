//! Owned-art projection of the pure notice controller. Parent owns input/dimming.

use super::{Presentation, modal_runtime};
use crate::{
    game_selector::{LockReason, Mode},
    modal::ModalInput,
    notice_modal::{NoticeIdentity, NoticeIntent, NoticeModal, NoticeRequest, NoticeView},
};

pub(super) struct NoticeFrame {
    pub consumed: bool,
    pub intent: Option<NoticeIntent>,
}

pub(super) struct NoticeRuntime {
    modal: NoticeModal,
}

impl NoticeRuntime {
    pub(super) fn quit(id: u64, guidance: impl Into<String>) -> Self {
        Self {
            modal: NoticeModal::quit(id, guidance),
        }
    }

    pub(super) fn information(
        id: u64,
        title: impl Into<String>,
        body: impl Into<String>,
        guidance: impl Into<String>,
    ) -> Self {
        Self {
            modal: NoticeModal::information(id, title, body, guidance),
        }
    }

    pub(super) fn locked_mode(
        id: u64,
        mode: Mode,
        reason: LockReason,
        guidance: impl Into<String>,
    ) -> Self {
        Self {
            modal: NoticeModal::locked_mode(id, mode, reason, guidance),
        }
    }

    pub(super) fn is_open(&self) -> bool {
        self.modal.is_open()
    }

    pub(super) fn identity(&self) -> NoticeIdentity {
        self.modal.identity()
    }

    pub(super) fn input(&mut self, input: &ModalInput) -> NoticeFrame {
        // Capture before processing: even the closing frame belongs to this modal.
        let consumed = self.modal.is_open();
        NoticeFrame {
            consumed,
            intent: self.modal.input(input),
        }
    }

    pub(super) fn cancel_capture(&mut self) {
        self.modal.cancel_capture();
    }

    pub(super) fn show_error(
        &mut self,
        identity: NoticeIdentity,
        error: impl Into<String>,
    ) -> bool {
        self.modal.show_error(identity, error)
    }

    pub(super) fn quit_finished(
        &mut self,
        request: NoticeRequest,
        result: Result<(), String>,
    ) -> bool {
        self.modal.quit_finished(request, result)
    }

    pub(super) fn view(&self) -> Option<NoticeView<'_>> {
        self.modal.view()
    }

    pub(super) fn draw(&self, presentation: &Presentation) {
        let Some(view) = self.view() else {
            return;
        };
        modal_runtime::draw_panel(presentation, view.panel);
        modal_runtime::draw_title(presentation, view.title, view.title_bounds);
        modal_runtime::draw_body(presentation, view.body, view.body_bounds, true);
        modal_runtime::draw_body(presentation, view.guidance, view.guidance_bounds, true);
        if let Some(feedback) = view.feedback {
            modal_runtime::draw_body(presentation, feedback, view.feedback_bounds, false);
        }
        for button in std::iter::once(view.primary).chain(view.cancel) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notice_runtime_consumes_opening_and_closing_frames_without_input_fallthrough() {
        let mut runtime = NoticeRuntime::information(91, "Information", "Caller message", "");
        let neutral = ModalInput {
            focused: true,
            ..ModalInput::default()
        };
        assert!(runtime.input(&neutral).consumed);
        let frame = runtime.input(&ModalInput {
            cancel: true,
            ..neutral
        });
        assert!(frame.consumed);
        assert_eq!(
            frame.intent,
            Some(NoticeIntent::Dismiss(runtime.identity()))
        );
        assert!(!runtime.is_open());
        assert!(!runtime.input(&ModalInput::default()).consumed);
    }
}
