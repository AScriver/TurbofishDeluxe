//! Shared offline controls for the concrete normal-game modal adapters.
//! State/transactions and native input polling remain caller-owned.

use macroquad::prelude::{Rect, Vec2};

#[derive(Clone, Debug, Default)]
pub struct ModalInput {
    pub pointer: Vec2,
    pub pressed: bool,
    pub released: bool,
    pub down: bool,
    pub focused: bool,
    pub accept: bool,
    pub cancel: bool,
    pub next: bool,
    pub previous: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down_key: bool,
    pub backspace: bool,
    pub delete: bool,
    pub home: bool,
    pub end: bool,
    pub text: String,
}

#[derive(Clone, Copy, Debug)]
pub struct ModalButton {
    pub id: u64,
    pub bounds: Rect,
    pub enabled: bool,
}

#[derive(Clone, Debug, Default)]
pub struct ModalControls {
    captured: Option<u64>,
    focus: Option<u64>,
    hovered: Option<u64>,
}

impl ModalControls {
    pub fn cancel_capture(&mut self) {
        self.captured = None;
        self.hovered = None;
    }

    pub fn set_focus(&mut self, focus: Option<u64>) {
        self.focus = focus;
    }

    pub fn focus(&self) -> Option<u64> {
        self.focus
    }

    pub fn is_pressed(&self, id: u64) -> bool {
        self.captured == Some(id) && self.hovered == Some(id)
    }

    pub fn is_focused(&self, id: u64) -> bool {
        self.focus == Some(id)
    }

    pub fn update(&mut self, input: &ModalInput, buttons: &[ModalButton]) -> Option<u64> {
        if !input.focused || input.cancel {
            self.cancel_capture();
            return None;
        }
        let enabled = |id| {
            buttons
                .iter()
                .any(|button| button.id == id && button.enabled)
        };
        if self.focus.is_none_or(|id| !enabled(id)) {
            self.focus = buttons
                .iter()
                .find(|button| button.enabled)
                .map(|button| button.id);
        }
        if self.captured.is_some_and(|id| !enabled(id)) {
            self.captured = None;
        }
        self.hovered = buttons
            .iter()
            .find(|button| button.enabled && button.bounds.contains(input.pointer))
            .map(|button| button.id);
        if input.next || input.previous {
            self.cancel_capture();
            let current = buttons
                .iter()
                .position(|button| Some(button.id) == self.focus);
            let count = buttons.len();
            if count > 0 {
                let start = current.unwrap_or(if input.previous { 0 } else { count - 1 });
                for offset in 1..=count {
                    let index = if input.previous {
                        (start + count - offset) % count
                    } else {
                        (start + offset) % count
                    };
                    if buttons[index].enabled {
                        self.focus = Some(buttons[index].id);
                        break;
                    }
                }
            }
            return None;
        }
        if input.pressed {
            self.captured = self.hovered;
            if self.captured.is_some() {
                self.focus = self.captured;
            }
        }
        if input.released {
            let captured = self.captured.take();
            if captured.is_some() && captured == self.hovered {
                return captured;
            }
        }
        if input.accept {
            self.cancel_capture();
            return self.focus.filter(|id| enabled(*id));
        }
        if !input.down && !input.pressed {
            self.captured = None;
        }
        None
    }
}
