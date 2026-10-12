//! Pure Quit/information controls; the caller retains save and mode authority.
//!
//! Owned-art and button roles are secondary-source-derived from registered W1.
//! Geometry, identity, neutral rearming and save feedback are project contracts.

use crate::{
    game_selector::{LockReason, Mode},
    modal::{ModalButton, ModalControls, ModalInput},
};
use macroquad::prelude::Rect;

const PRIMARY: u64 = 1;
const CANCEL: u64 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoticeKind {
    Quit,
    Information,
    LockedMode { mode: Mode, reason: LockReason },
}

/// The parent supplies a fresh ID for each opening, including replacements.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoticeIdentity {
    pub id: u64,
    pub kind: NoticeKind,
}

/// Save feedback must identify the exact pending confirmation, not just Quit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoticeRequest {
    pub identity: NoticeIdentity,
    pub attempt: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoticeIntent {
    Quit(NoticeRequest),
    Cancel(NoticeIdentity),
    Dismiss(NoticeIdentity),
}

#[derive(Clone, Copy, Debug)]
pub struct NoticeButtonView {
    pub control: ModalButton,
    pub label: &'static str,
    pub pressed: bool,
    pub focused: bool,
}

/// The sole geometry authority for hit testing and drawing on the 640x480 UI.
pub struct NoticeView<'a> {
    pub identity: NoticeIdentity,
    pub panel: Rect,
    pub title_bounds: Rect,
    pub body_bounds: Rect,
    pub guidance_bounds: Rect,
    pub feedback_bounds: Rect,
    pub title: &'a str,
    pub body: &'a str,
    pub guidance: &'a str,
    pub feedback: Option<&'a str>,
    pub pending: bool,
    pub primary: NoticeButtonView,
    pub cancel: Option<NoticeButtonView>,
}

pub struct NoticeModal {
    identity: NoticeIdentity,
    title: String,
    body: String,
    guidance: String,
    error: Option<String>,
    controls: ModalControls,
    open: bool,
    armed: bool,
    attempts: u64,
    pending: Option<NoticeRequest>,
}

impl NoticeModal {
    pub fn quit(id: u64, guidance: impl Into<String>) -> Self {
        Self::new(
            id,
            NoticeKind::Quit,
            "Quit",
            "Quit and save current profile?",
            guidance,
        )
    }

    pub fn information(
        id: u64,
        title: impl Into<String>,
        body: impl Into<String>,
        guidance: impl Into<String>,
    ) -> Self {
        Self::new(id, NoticeKind::Information, title, body, guidance)
    }

    /// Lock identity is informational. Dismissal never emits mode admission.
    pub fn locked_mode(
        id: u64,
        mode: Mode,
        reason: LockReason,
        guidance: impl Into<String>,
    ) -> Self {
        Self::new(
            id,
            NoticeKind::LockedMode { mode, reason },
            "Mode unavailable",
            reason.guidance(),
            guidance,
        )
    }

    fn new(
        id: u64,
        kind: NoticeKind,
        title: impl Into<String>,
        body: impl Into<String>,
        guidance: impl Into<String>,
    ) -> Self {
        let mut controls = ModalControls::default();
        controls.set_focus(Some(if kind == NoticeKind::Quit {
            CANCEL
        } else {
            PRIMARY
        }));
        Self {
            identity: NoticeIdentity { id, kind },
            title: title.into(),
            body: body.into(),
            guidance: guidance.into(),
            error: None,
            controls,
            open: true,
            armed: false,
            attempts: 0,
            pending: None,
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn identity(&self) -> NoticeIdentity {
        self.identity
    }

    pub fn pending_request(&self) -> Option<NoticeRequest> {
        self.pending
    }

    /// Covering modals/focus changes must discard capture even without an event.
    pub fn cancel_capture(&mut self) {
        self.controls.cancel_capture();
        self.armed = false;
    }

    /// Display exact caller errors without replacing the caller's guidance.
    pub fn show_error(&mut self, identity: NoticeIdentity, error: impl Into<String>) -> bool {
        if !self.open || self.identity != identity || self.pending.is_some() {
            return false;
        }
        self.error = Some(error.into());
        self.cancel_capture();
        true
    }

    /// Only the actual caller-owned save result can finish a pending Quit.
    pub fn quit_finished(&mut self, request: NoticeRequest, result: Result<(), String>) -> bool {
        if !self.open || self.pending != Some(request) {
            return false;
        }
        self.pending = None;
        self.cancel_capture();
        match result {
            Ok(()) => self.open = false,
            Err(error) => {
                self.error = Some(error);
                self.controls.set_focus(Some(CANCEL));
            }
        }
        true
    }

    pub fn input(&mut self, input: &ModalInput) -> Option<NoticeIntent> {
        if !self.open {
            return None;
        }
        if !input.focused {
            self.cancel_capture();
            return None;
        }
        if self.pending.is_some() {
            self.controls.cancel_capture();
            return None;
        }
        if !self.armed {
            self.armed =
                !input.pressed && !input.released && !input.down && !input.accept && !input.cancel;
            return None;
        }
        if input.cancel {
            return Some(self.close());
        }

        let view = self.view()?;
        let mut buttons = vec![view.primary.control];
        if let Some(cancel) = view.cancel {
            buttons.push(cancel.control);
        }
        let control_input = ModalInput {
            pointer: input.pointer,
            pressed: input.pressed,
            released: input.released,
            down: input.down,
            focused: input.focused,
            accept: input.accept,
            next: input.next || input.right || input.down_key,
            previous: input.previous || input.left || input.up,
            ..ModalInput::default()
        };
        match self.controls.update(&control_input, &buttons)? {
            PRIMARY if self.identity.kind == NoticeKind::Quit => {
                self.attempts = self.attempts.checked_add(1)?;
                let request = NoticeRequest {
                    identity: self.identity,
                    attempt: self.attempts,
                };
                self.pending = Some(request);
                self.controls.cancel_capture();
                Some(NoticeIntent::Quit(request))
            }
            PRIMARY | CANCEL => Some(self.close()),
            _ => None,
        }
    }

    fn close(&mut self) -> NoticeIntent {
        self.open = false;
        self.cancel_capture();
        if self.identity.kind == NoticeKind::Quit {
            NoticeIntent::Cancel(self.identity)
        } else {
            NoticeIntent::Dismiss(self.identity)
        }
    }

    pub fn view(&self) -> Option<NoticeView<'_>> {
        if !self.open {
            return None;
        }
        let quit = self.identity.kind == NoticeKind::Quit;
        let pending = self.pending.is_some();
        let primary_enabled = !pending && (!quit || self.attempts < u64::MAX);
        Some(NoticeView {
            identity: self.identity,
            panel: Rect::new(100.0, 78.0, 440.0, 324.0),
            title_bounds: Rect::new(136.0, 96.0, 368.0, 34.0),
            body_bounds: Rect::new(136.0, 145.0, 368.0, 90.0),
            guidance_bounds: Rect::new(136.0, 239.0, 368.0, 40.0),
            feedback_bounds: Rect::new(136.0, 283.0, 368.0, 54.0),
            title: &self.title,
            body: &self.body,
            guidance: &self.guidance,
            feedback: if pending {
                Some("Saving current profile...")
            } else {
                self.error.as_deref()
            },
            pending,
            primary: self.button(
                PRIMARY,
                Rect::new(if quit { 144.0 } else { 238.0 }, 349.0, 164.0, 34.0),
                if quit { "Quit" } else { "OK" },
                primary_enabled,
            ),
            cancel: quit.then(|| {
                self.button(
                    CANCEL,
                    Rect::new(332.0, 349.0, 164.0, 34.0),
                    "Cancel",
                    !pending,
                )
            }),
        })
    }

    fn button(
        &self,
        id: u64,
        bounds: Rect,
        label: &'static str,
        enabled: bool,
    ) -> NoticeButtonView {
        NoticeButtonView {
            control: ModalButton {
                id,
                bounds,
                enabled,
            },
            label,
            pressed: enabled && self.controls.is_pressed(id),
            focused: enabled && self.controls.is_focused(id),
        }
    }
}
