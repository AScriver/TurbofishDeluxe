//! Native input projection for the existing Virtual Tank UI and pet chooser.
//! Session transactions, navigation and saves remain parent-owned.

use super::{Presentation, pet_at_pointer};
use crate::{
    adventure::AdventureSession,
    sim::{Action, PetKind},
    virtual_ui::{Point, UiInput},
};
use macroquad::prelude::*;
use std::cell::Cell;

/// Sample one native frame. Caller sends each returned input to VirtualRuntime
/// in order, and refreshes from the Session after every accepted request.
pub(super) fn inputs(
    runtime: &super::virtual_runtime::VirtualRuntime,
    pointer: Vec2,
    focused: bool,
    blocked: bool,
) -> Vec<UiInput> {
    if !focused || blocked {
        return vec![UiInput::PointerLeave, UiInput::Cancel];
    }
    let mut frame = Vec::new();
    if pointer.x.is_finite()
        && pointer.y.is_finite()
        && (0.0..640.0).contains(&pointer.x)
        && (0.0..480.0).contains(&pointer.y)
    {
        let point = Point {
            x: pointer.x,
            y: pointer.y,
        };
        frame.push(UiInput::PointerMove(point));
        if is_mouse_button_pressed(MouseButton::Left) {
            frame.push(UiInput::PointerDown(point));
        }
        if is_mouse_button_released(MouseButton::Left) {
            frame.push(UiInput::PointerUp(point));
        }
    } else {
        frame.push(UiInput::PointerLeave);
    }
    if let Some(name) = runtime.dialog_name() {
        let mut edited = name.to_owned();
        let mut changed = false;
        if is_key_pressed(KeyCode::Backspace) && edited.pop().is_some() {
            changed = true;
        }
        while let Some(character) = get_char_pressed() {
            if !character.is_control() && edited.len() + character.len_utf8() <= 64 {
                edited.push(character);
                changed = true;
            }
        }
        if changed {
            frame.push(UiInput::EditName(edited));
        }
    }
    if is_key_pressed(KeyCode::Escape) {
        frame.push(UiInput::Cancel);
    } else if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
        frame.push(UiInput::Submit);
    }
    frame
}

#[derive(Debug, Default)]
pub(super) struct PetPickerFrame {
    pub actions: Vec<Action>,
    pub close: bool,
}

#[derive(Clone, Copy)]
enum Capture {
    Pet(PetKind),
    Back,
    Menu,
}

/// Temporary chooser entered from Virtual Tank Setup. The durable selection
/// remains `session.virtual_tank.selected_pets` and the Session rechecks it.
#[derive(Default)]
pub(super) struct VirtualPetPicker {
    captured: Option<Capture>,
    button_height: Cell<f32>,
}

impl VirtualPetPicker {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn input(
        &mut self,
        session: &AdventureSession,
        pointer: Vec2,
        focused: bool,
        blocked: bool,
    ) -> PetPickerFrame {
        let mut frame = PetPickerFrame::default();
        if !focused || blocked || !pointer.x.is_finite() || !pointer.y.is_finite() {
            self.captured = None;
            return frame;
        }
        if is_key_pressed(KeyCode::Escape) {
            self.captured = None;
            frame.close = true;
            return frame;
        }
        let back = self.back_rect();
        let menu = self.menu_rect();
        if is_mouse_button_pressed(MouseButton::Left) {
            self.captured = pet_at_pointer(&session.progress.unlocked_pets, pointer)
                .filter(|pet| can_toggle(session, *pet))
                .map(Capture::Pet)
                .or_else(|| {
                    back.is_some_and(|rect| rect.contains(pointer))
                        .then_some(Capture::Back)
                })
                .or_else(|| {
                    menu.is_some_and(|rect| rect.contains(pointer))
                        .then_some(Capture::Menu)
                });
        }
        if is_mouse_button_released(MouseButton::Left) {
            match self.captured.take() {
                Some(Capture::Pet(pet))
                    if pet_at_pointer(&session.progress.unlocked_pets, pointer) == Some(pet)
                        && can_toggle(session, pet) =>
                {
                    frame.actions.push(Action::VirtualTogglePet { pet });
                }
                Some(Capture::Back) if back.is_some_and(|rect| rect.contains(pointer)) => {
                    frame.close = true
                }
                Some(Capture::Menu) if menu.is_some_and(|rect| rect.contains(pointer)) => {
                    frame.close = true
                }
                _ => {}
            }
        }
        frame
    }

    pub(super) fn draw(
        &self,
        presentation: &Presentation,
        session: &AdventureSession,
        pointer: Vec2,
    ) {
        presentation.draw_pet_selection(session, &session.virtual_tank.selected_pets, pointer);
        self.button_height
            .set(presentation.images["IMAGE_MAINBUTTON"].height());
        // The generic chooser labels these controls Continue/Menu; this
        // transient subpage returns to VT Setup without either Session action.
        if let Some(back) = self.back_rect() {
            presentation.main_button(back, "Back to Tank Setup");
        }
        if let Some(menu) = self.menu_rect() {
            presentation.main_button(menu, "Back");
        }
    }

    fn back_rect(&self) -> Option<Rect> {
        (self.button_height.get() > 0.0)
            .then(|| Rect::new(225.0, 250.0, 186.0, self.button_height.get()))
    }

    fn menu_rect(&self) -> Option<Rect> {
        (self.button_height.get() > 0.0)
            .then(|| Rect::new(525.0, 4.0, 80.0, self.button_height.get()))
    }
}

fn can_toggle(session: &AdventureSession, pet: PetKind) -> bool {
    session.progress.unlocked_pets.contains(&pet)
        && (session.virtual_tank.selected_pets.contains(&pet)
            || session.virtual_tank.selected_pets.len()
                < usize::from(session.progress.pet_capacity).min(7))
}
