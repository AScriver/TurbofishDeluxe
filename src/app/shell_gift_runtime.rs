//! Transient shell-gift frontend over the production controller.
//!
//! UserDialog functional layout is secondary-source-derived from registered W1
//! f919b3c: 400px panel, centered 15px list rows, 130px list, amount focus,
//! shell/edit art and paired buttons. Height/accounting/guidance and retained
//! self-transfer are project policies; see docs/modals-shell-gift.md.
//! No images, clocks, sessions, wallets or save paths are owned here.
//!
//! Parent hooks:
//! - Register this child and preload shared modal art plus `REQUIRED_IMAGE_IDS`
//!   and `REQUIRED_FONT_NAMES`.
//! - Open on `SelectorIntent::Dialog(DialogIntent::OpenGiveDialog)`, using the
//!   current authoritative `Profiles`.
//! - While open, route input here before selector/game input and consume the
//!   entire frame, including closing frames. Pause underlying UI/simulation.
//! - Project the parent's single normalized `ModalInput` with
//!   `ShellGiftInput::from_modal`, retaining optional wheel/Page/Ctrl+A fields.
//!   Legacy `poll` also collects once, only while this adapter owns input.
//!   `view`, `input` and `draw` rebuild production views. Parent dims once.
//! - On `Confirm`, outside a live session borrow and under the normal writer
//!   lease, call `confirmation_dialog().unwrap().confirm(&mut profiles, path)`.
//!   The intent's request identifies the displayed proposal, never a wallet
//!   authority. The existing controller rechecks current production state.
//! - Return that exact result through `confirmation_finished`, then refresh
//!   cached session/HUD from `Profiles` on success. Keep result/saver receipts;
//!   never save the old cached session over the committed profile envelope.
//! - Cancel/Escape needs no writer. Drop the adapter after `is_open` becomes
//!   false; retain it and show the fresh view after a failed confirmation.
//! - Higher modals/focus loss must still call `input` with the blocked flag;
//!   capture is discarded and a neutral frame is required before resuming.

use super::{
    Presentation,
    help_runtime::{fill, font_text, image, outline},
    modal_runtime::{draw_body, draw_button, draw_edit_box, draw_panel, draw_title},
};
use crate::{
    modal::{ModalButton, ModalControls, ModalInput},
    profiles::Profiles,
    shell_gift::{
        MAX_AMOUNT_TEXT_BYTES, ShellGiftDialog, ShellGiftError, ShellGiftPreview, ShellGiftRequest,
        ShellGiftView,
    },
};
use macroquad::prelude::*;

pub(super) const REQUIRED_IMAGE_IDS: &[&str] = &[
    "IMAGE_DIALOG",
    "IMAGE_DIALOGBUTTON",
    "IMAGE_EDITBOX",
    "IMAGE_HELPSHELL",
];
pub(super) const REQUIRED_FONT_NAMES: &[&str] = &[
    "Liddie12",
    "JungleFever10outline",
    "JungleFever12outline",
    "JungleFever15outline",
];

const BODY_FONT: &str = "JungleFever10outline";
const LIST_FONT: &str = "Liddie12";
const ROW_HEIGHT: f32 = 15.0;
const VISIBLE_ROWS: usize = 8;
const DETAIL_LINES: usize = 6;
const INK: Color = Color::new(0.94, 0.97, 1.0, 1.0);
const DIM_INK: Color = Color::new(0.83, 0.78, 0.69, 1.0);
const ACCENT: Color = Color::new(1.0, 0.88, 0.29, 1.0);
const LIST_BACKGROUND: Color = Color::new(0.21, 0.17, 0.11, 1.0);

// Fixed control identities never share the profile-ID namespace. List capture
// retains its exact profile ID separately, rather than using a visible row ID.
const RECIPIENT_CONTROL: u64 = 1;
const AMOUNT_CONTROL: u64 = 2;
const DETAILS_CONTROL: u64 = 3;
const CONFIRM_CONTROL: u64 = 4;
const CANCEL_CONTROL: u64 = 5;
const UP_CONTROL: u64 = 6;
const DOWN_CONTROL: u64 = 7;

/// Concrete geometry belongs to the view, shared by input and drawing.
#[derive(Clone, Copy, Debug)]
pub(super) struct ShellGiftLayout {
    pub panel: Rect,
    pub title: Rect,
    pub donor: Rect,
    pub recipient_label: Rect,
    pub recipients: Rect,
    pub scroll_up: Rect,
    pub scroll_down: Rect,
    pub scroll_track: Rect,
    pub scrollbar_visible: bool,
    pub amount_label: Rect,
    pub shell: Rect,
    pub amount: Rect,
    pub details: Rect,
    pub confirm: Rect,
    pub cancel: Rect,
    pub footer: Rect,
}

impl ShellGiftLayout {
    fn new(count: usize) -> Self {
        let scrollbar_visible = count >= VISIBLE_ROWS;
        Self {
            panel: Rect::new(120.0, 16.0, 400.0, 448.0),
            title: Rect::new(156.0, 31.0, 328.0, 25.0),
            donor: Rect::new(156.0, 61.0, 328.0, 18.0),
            recipient_label: Rect::new(156.0, 84.0, 328.0, 18.0),
            recipients: Rect::new(
                166.0,
                105.0,
                if scrollbar_visible { 292.0 } else { 308.0 },
                130.0,
            ),
            scroll_up: Rect::new(458.0, 105.0, 16.0, 16.0),
            scroll_down: Rect::new(458.0, 219.0, 16.0, 16.0),
            scroll_track: Rect::new(458.0, 121.0, 16.0, 98.0),
            scrollbar_visible,
            amount_label: Rect::new(156.0, 246.0, 126.0, 32.0),
            shell: Rect::new(294.0, 250.0, 24.0, 24.0),
            amount: Rect::new(328.0, 246.0, 156.0, 32.0),
            details: Rect::new(156.0, 282.0, 328.0, 112.0),
            confirm: Rect::new(156.0, 398.0, 160.0, 30.0),
            cancel: Rect::new(324.0, 398.0, 160.0, 30.0),
            footer: Rect::new(156.0, 432.0, 328.0, 24.0),
        }
    }

    fn recipient_row(self, visible: usize) -> Rect {
        Rect::new(
            self.recipients.x,
            self.recipients.y + visible as f32 * ROW_HEIGHT,
            self.recipients.w,
            ROW_HEIGHT,
        )
    }
}

pub(super) struct ShellGiftRuntimeView {
    pub gift: ShellGiftView,
    pub layout: ShellGiftLayout,
    pub first_row: usize,
}

impl ShellGiftRuntimeView {
    fn recipient_at(&self, pointer: Vec2) -> Option<u64> {
        if !self.layout.recipients.contains(pointer)
            || pointer.y >= self.layout.recipients.y + VISIBLE_ROWS as f32 * ROW_HEIGHT
        {
            return None;
        }
        let row = ((pointer.y - self.layout.recipients.y) / ROW_HEIGHT) as usize;
        self.gift
            .recipients
            .get(self.first_row + row)
            .map(|row| row.id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ShellGiftIntent {
    Confirm(ShellGiftRequest),
    Cancel,
}

/// An open modal owns the whole input frame even when it closes this frame.
pub(super) struct ShellGiftFrame {
    pub consumed: bool,
    pub intent: Option<ShellGiftIntent>,
}

/// Host-projected input; callers may also construct it without a GPU context.
/// Text is admitted whole or rejected whole, including a pasted sign/fraction.
#[derive(Default)]
pub(super) struct ShellGiftInput {
    pub pointer: Vec2,
    pub focused: bool,
    pub blocked_by_modal: bool,
    pub pressed: bool,
    pub released: bool,
    pub down: bool,
    pub wheel: f32,
    pub keys: Vec<KeyCode>,
    pub shift: bool,
    pub control: bool,
    pub text: String,
    pub text_overflow: bool,
}

impl ShellGiftInput {
    /// Root may add wheel, Page keys, Ctrl+A and whole-paste overflow after this
    /// projection. Character input must already have been collected only once.
    pub(super) fn from_modal(input: ModalInput) -> Self {
        let mut keys = Vec::new();
        for (pressed, key) in [
            (input.accept, KeyCode::Enter),
            (input.cancel, KeyCode::Escape),
            (input.next || input.previous, KeyCode::Tab),
            (input.left, KeyCode::Left),
            (input.right, KeyCode::Right),
            (input.up, KeyCode::Up),
            (input.down_key, KeyCode::Down),
            (input.backspace, KeyCode::Backspace),
            (input.delete, KeyCode::Delete),
            (input.home, KeyCode::Home),
            (input.end, KeyCode::End),
        ] {
            if pressed {
                keys.push(key);
            }
        }
        Self {
            pointer: input.pointer,
            focused: input.focused,
            pressed: input.pressed,
            released: input.released,
            down: input.down,
            keys,
            shift: input.previous,
            text_overflow: input.text.len() > MAX_AMOUNT_TEXT_BYTES,
            text: input.text,
            ..Self::default()
        }
    }

    fn modal_input(&self, focus: Option<u64>) -> ModalInput {
        ModalInput {
            pointer: self.pointer,
            focused: self.focused && !self.blocked_by_modal,
            pressed: self.pressed,
            released: self.released,
            down: self.down,
            accept: (self.key(KeyCode::Enter)
                || self.key(KeyCode::Space) && focus != Some(AMOUNT_CONTROL))
                && !(focus == Some(AMOUNT_CONTROL) && amount_edited(self)),
            cancel: self.key(KeyCode::Escape),
            next: self.key(KeyCode::Tab) && !self.shift,
            previous: self.key(KeyCode::Tab) && self.shift,
            left: self.key(KeyCode::Left),
            right: self.key(KeyCode::Right),
            up: self.key(KeyCode::Up),
            down_key: self.key(KeyCode::Down),
            backspace: self.key(KeyCode::Backspace),
            delete: self.key(KeyCode::Delete),
            home: self.key(KeyCode::Home),
            end: self.key(KeyCode::End),
            text: self.text.clone(),
        }
    }

    /// Call only while this modal owns input. Drains the character queue even
    /// when suspended, preventing delayed text after focus/modality changes.
    #[allow(
        dead_code,
        reason = "Temporary compatibility until root's normalized-input join"
    )]
    pub(super) fn poll(pointer: Vec2, focused: bool, blocked_by_modal: bool) -> Self {
        let control = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);
        let mut input = Self::from_modal(ModalInput {
            pointer,
            focused,
            pressed: is_mouse_button_pressed(MouseButton::Left),
            released: is_mouse_button_released(MouseButton::Left),
            down: is_mouse_button_down(MouseButton::Left),
            ..ModalInput::default()
        });
        input.blocked_by_modal = blocked_by_modal;
        input.wheel = mouse_wheel().1;
        input.keys = get_keys_pressed().into_iter().collect();
        input.shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
        input.control = control;
        while let Some(character) = get_char_pressed() {
            if focused && !blocked_by_modal && !control && !character.is_control() {
                if input.text.len() + character.len_utf8() <= MAX_AMOUNT_TEXT_BYTES {
                    input.text.push(character);
                } else {
                    input.text_overflow = true;
                }
            }
        }
        if focused
            && !blocked_by_modal
            && control
            && input.key(KeyCode::V)
            && let Some(paste) = macroquad::miniquad::window::clipboard_get()
        {
            input.text_overflow = paste.len() > MAX_AMOUNT_TEXT_BYTES;
            if !input.text_overflow {
                input.text = paste;
            }
        }
        input
    }

    fn key(&self, key: KeyCode) -> bool {
        self.keys.contains(&key)
    }
}

pub(super) struct ShellGiftRuntime {
    dialog: ShellGiftDialog,
    controls: ModalControls,
    captured_recipient: Option<u64>,
    first_row: usize,
    detail_scroll: usize,
    cursor: usize,
    select_all: bool,
    armed: bool,
    suspended: bool,
    pending_confirmation: bool,
    returned_error: Option<ShellGiftError>,
    shown_preview: Option<ShellGiftPreview>,
    review_required: bool,
}

impl ShellGiftRuntime {
    pub(super) fn open(profiles: &Profiles) -> Result<Self, ShellGiftError> {
        let mut controls = ModalControls::default();
        controls.set_focus(Some(AMOUNT_CONTROL));
        Ok(Self {
            dialog: ShellGiftDialog::open(profiles)?,
            controls,
            captured_recipient: None,
            first_row: 0,
            detail_scroll: 0,
            cursor: 0,
            select_all: false,
            armed: false,
            suspended: false,
            pending_confirmation: false,
            returned_error: None,
            shown_preview: None,
            review_required: false,
        })
    }

    pub(super) fn is_open(&self) -> bool {
        self.dialog.is_open()
    }

    /// Immutable production data is never cached between frames. Missing IDs
    /// remain missing; refresh cannot select a different row implicitly.
    pub(super) fn refresh(&mut self, profiles: &Profiles) -> ShellGiftView {
        let mut view = self.dialog.view(profiles);
        self.first_row = self
            .first_row
            .min(view.recipients.len().saturating_sub(VISIBLE_ROWS));
        self.cursor = self.cursor.min(view.amount_text.len());
        if let Some(error) = &self.returned_error {
            view.confirmation_error = Some(error.clone());
        }
        view
    }

    /// Producing a view does not acknowledge that a preview was painted.
    pub(super) fn view(&mut self, profiles: &Profiles) -> ShellGiftRuntimeView {
        let gift = self.refresh(profiles);
        ShellGiftRuntimeView {
            layout: ShellGiftLayout::new(gift.recipients.len()),
            gift,
            first_row: self.first_row,
        }
    }

    fn retire_capture(&mut self) {
        self.controls.cancel_capture();
        self.captured_recipient = None;
    }

    fn buttons(&self, view: &ShellGiftRuntimeView) -> [ModalButton; 7] {
        let enabled = !self.pending_confirmation && !self.suspended;
        let layout = view.layout;
        [
            ModalButton {
                id: RECIPIENT_CONTROL,
                bounds: layout.recipients,
                enabled: enabled && !view.gift.recipients.is_empty(),
            },
            ModalButton {
                id: AMOUNT_CONTROL,
                bounds: layout.amount,
                enabled,
            },
            ModalButton {
                id: DETAILS_CONTROL,
                bounds: layout.details,
                enabled,
            },
            ModalButton {
                id: CONFIRM_CONTROL,
                bounds: layout.confirm,
                enabled: enabled && view.gift.can_confirm,
            },
            ModalButton {
                id: CANCEL_CONTROL,
                bounds: layout.cancel,
                enabled,
            },
            ModalButton {
                id: UP_CONTROL,
                bounds: layout.scroll_up,
                enabled: enabled && layout.scrollbar_visible && self.first_row > 0,
            },
            ModalButton {
                id: DOWN_CONTROL,
                bounds: layout.scroll_down,
                enabled: enabled
                    && layout.scrollbar_visible
                    && self.first_row + VISIBLE_ROWS < view.gift.recipients.len(),
            },
        ]
    }

    /// Only primary's confirmation dispatch may borrow the actual controller.
    /// No profile manager/path or writer is owned by this adapter.
    pub(super) fn confirmation_dialog(&mut self) -> Option<&mut ShellGiftDialog> {
        self.pending_confirmation.then_some(&mut self.dialog)
    }

    /// Feed the actual controller result (or an owner/lease failure) back.
    /// Failure keeps input/open state; success closes without any further save.
    pub(super) fn confirmation_finished(
        &mut self,
        result: Result<ShellGiftPreview, ShellGiftError>,
    ) {
        if !self.pending_confirmation {
            return;
        }
        self.pending_confirmation = false;
        self.retire_capture();
        self.armed = false;
        self.detail_scroll = 0;
        match result {
            Ok(_) => {
                self.returned_error = None;
                self.dialog.cancel();
            }
            Err(error) => self.returned_error = Some(error),
        }
    }

    pub(super) fn input(&mut self, profiles: &Profiles, input: &ShellGiftInput) -> ShellGiftFrame {
        let mut frame = ShellGiftFrame {
            consumed: self.is_open(),
            intent: None,
        };
        if !frame.consumed {
            return frame;
        }
        let mut view = self.view(profiles);
        self.suspended = !input.focused || input.blocked_by_modal;
        if self.suspended || self.pending_confirmation {
            self.retire_capture();
            self.armed = false;
            return frame;
        }
        if !self.armed {
            self.armed = !input.down && !input.pressed;
            return frame;
        }
        if input.key(KeyCode::Escape) {
            self.cancel();
            frame.intent = Some(ShellGiftIntent::Cancel);
            return frame;
        }
        if input.key(KeyCode::Tab) {
            self.retire_capture();
            self.select_all = false;
        }
        // Drain a pending amount edit before a pointer can move focus to Give.
        // Any resulting preview still needs a later paint and explicit accept.
        let edited_before_activation =
            self.controls.focus() == Some(AMOUNT_CONTROL) && amount_edited(input);
        if edited_before_activation {
            self.amount_keys(&view.gift.amount_text, input);
            view = self.view(profiles);
        }
        if input.wheel.is_finite() && input.wheel != 0.0 {
            let upwards = input.wheel > 0.0;
            if view.layout.recipients.contains(input.pointer)
                || view.layout.scrollbar_visible && view.layout.scroll_track.contains(input.pointer)
            {
                self.scroll_rows(view.gift.recipients.len(), upwards, 1);
                self.retire_capture();
                view.first_row = self.first_row;
            } else if view.layout.details.contains(input.pointer) {
                self.scroll_details(upwards, 1);
            }
        }
        if input.pressed {
            self.captured_recipient = view.recipient_at(input.pointer);
            self.select_all = false;
            if view.layout.amount.contains(input.pointer) {
                self.cursor = view.gift.amount_text.len();
            }
        }
        let modal_input = input.modal_input(self.controls.focus());
        let buttons = self.buttons(&view);
        let activation = self.controls.update(&modal_input, &buttons);
        match self.controls.focus() {
            Some(RECIPIENT_CONTROL) => self.recipient_keys(&view.gift, input),
            Some(AMOUNT_CONTROL) if !edited_before_activation => {
                self.amount_keys(&view.gift.amount_text, input)
            }
            Some(DETAILS_CONTROL) => {
                if input.key(KeyCode::Up) || input.key(KeyCode::PageUp) {
                    self.scroll_details(true, 1);
                }
                if input.key(KeyCode::Down) || input.key(KeyCode::PageDown) {
                    self.scroll_details(false, 1);
                }
                if input.key(KeyCode::Home) {
                    self.detail_scroll = 0;
                }
            }
            _ => {}
        }
        match activation {
            Some(RECIPIENT_CONTROL) if input.released => {
                if let Some(id) = self.captured_recipient.take()
                    && view.recipient_at(input.pointer) == Some(id)
                {
                    self.select_recipient(id);
                }
            }
            Some(RECIPIENT_CONTROL) => {
                self.controls.set_focus(Some(AMOUNT_CONTROL));
                self.select_all = true;
                self.cursor = view.gift.amount_text.len();
            }
            Some(UP_CONTROL) => self.scroll_rows(view.gift.recipients.len(), true, 1),
            Some(DOWN_CONTROL) => self.scroll_rows(view.gift.recipients.len(), false, 1),
            Some(CONFIRM_CONTROL) => frame.intent = self.request_confirmation(profiles),
            Some(AMOUNT_CONTROL) if modal_input.accept => {
                frame.intent = self.request_confirmation(profiles)
            }
            Some(CANCEL_CONTROL) => {
                self.cancel();
                frame.intent = Some(ShellGiftIntent::Cancel);
            }
            _ => {}
        }
        if input.released || !input.down {
            self.captured_recipient = None;
        }
        frame
    }

    pub(super) fn cancel(&mut self) {
        self.dialog.cancel();
        self.retire_capture();
        self.returned_error = None;
        self.pending_confirmation = false;
    }

    fn request_confirmation(&mut self, profiles: &Profiles) -> Option<ShellGiftIntent> {
        let view = self.refresh(profiles);
        let gift = view.preview.filter(|_| view.can_confirm)?;
        // A fresh snapshot is authoritative, but new capacity/funds/identity
        // accounting must be painted before it can be accepted by the user.
        if self.shown_preview != Some(gift) {
            self.review_required = true;
            self.detail_scroll = 0;
            return None;
        }
        self.review_required = false;
        self.pending_confirmation = true;
        self.retire_capture();
        Some(ShellGiftIntent::Confirm(ShellGiftRequest {
            donor_id: gift.donor_id,
            recipient_id: gift.recipient_id,
            amount: gift.requested_debit,
        }))
    }

    fn select_recipient(&mut self, id: u64) {
        self.dialog.select_recipient(Some(id));
        self.returned_error = None;
        self.detail_scroll = 0;
        self.shown_preview = None;
    }

    fn scroll_rows(&mut self, count: usize, upwards: bool, steps: usize) {
        self.first_row = if upwards {
            self.first_row.saturating_sub(steps)
        } else {
            self.first_row
                .saturating_add(steps)
                .min(count.saturating_sub(VISIBLE_ROWS))
        };
    }

    fn scroll_details(&mut self, upwards: bool, steps: usize) {
        self.detail_scroll = if upwards {
            self.detail_scroll.saturating_sub(steps)
        } else {
            self.detail_scroll.saturating_add(steps)
        };
    }

    fn recipient_keys(&mut self, view: &ShellGiftView, input: &ShellGiftInput) {
        let index = view
            .recipients
            .iter()
            .position(|row| Some(row.id) == view.selected_recipient_id);
        let last = view.recipients.len().checked_sub(1);
        let target = if input.key(KeyCode::Home) {
            last.map(|_| 0)
        } else if input.key(KeyCode::End) {
            last
        } else if input.key(KeyCode::Up) || input.key(KeyCode::PageUp) {
            last.map(|last| {
                index.map_or(last, |index| {
                    index.saturating_sub(if input.key(KeyCode::PageUp) {
                        VISIBLE_ROWS
                    } else {
                        1
                    })
                })
            })
        } else if input.key(KeyCode::Down) || input.key(KeyCode::PageDown) {
            last.map(|last| {
                index.map_or(0, |index| {
                    (index
                        + if input.key(KeyCode::PageDown) {
                            VISIBLE_ROWS
                        } else {
                            1
                        })
                    .min(last)
                })
            })
        } else {
            None
        };
        if let Some(index) = target {
            self.select_recipient(view.recipients[index].id);
            if index < self.first_row {
                self.first_row = index;
            }
            if index >= self.first_row + VISIBLE_ROWS {
                self.first_row = index + 1 - VISIBLE_ROWS;
            }
        }
    }

    fn amount_keys(&mut self, text: &str, input: &ShellGiftInput) {
        if input.control && input.key(KeyCode::A) {
            self.select_all = true;
            self.cursor = text.len();
        }
        if input.key(KeyCode::Home) {
            self.cursor = 0;
            self.select_all = false;
        }
        if input.key(KeyCode::End) {
            self.cursor = text.len();
            self.select_all = false;
        }
        if input.key(KeyCode::Left) {
            self.cursor = text[..self.cursor]
                .char_indices()
                .next_back()
                .map_or(0, |(index, _)| index);
            self.select_all = false;
        }
        if input.key(KeyCode::Right) {
            self.cursor += text[self.cursor..].chars().next().map_or(0, char::len_utf8);
            self.select_all = false;
        }
        if !amount_edited(input) {
            return;
        }
        let mut edited = text.to_owned();
        if self.select_all {
            edited.clear();
            self.cursor = 0;
            self.select_all = false;
        } else if input.key(KeyCode::Backspace) && self.cursor > 0 {
            let previous = edited[..self.cursor]
                .char_indices()
                .next_back()
                .map_or(0, |(index, _)| index);
            edited.replace_range(previous..self.cursor, "");
            self.cursor = previous;
        } else if input.key(KeyCode::Delete) && self.cursor < edited.len() {
            let next = self.cursor
                + edited[self.cursor..]
                    .chars()
                    .next()
                    .map_or(0, char::len_utf8);
            edited.replace_range(self.cursor..next, "");
        }
        if input.text_overflow
            || input.text.len() > MAX_AMOUNT_TEXT_BYTES.saturating_sub(edited.len())
        {
            // The controller owns the oversize error/clear semantics. Never
            // submit the accepted prefix of an oversized paste/character batch.
            self.dialog
                .set_amount_text(&"0".repeat(MAX_AMOUNT_TEXT_BYTES + 1));
            self.cursor = 0;
        } else {
            edited.insert_str(self.cursor, &input.text);
            self.cursor += input.text.len();
            self.dialog.set_amount_text(&edited);
        }
        self.returned_error = None;
        self.detail_scroll = 0;
        self.shown_preview = None;
    }

    pub(super) fn draw(&mut self, presentation: &Presentation, profiles: &Profiles, pointer: Vec2) {
        if !self.is_open() {
            return;
        }
        let view = self.view(profiles);
        let layout = view.layout;
        let gift = &view.gift;
        draw_panel(presentation, layout.panel);
        draw_title(presentation, gift.title, layout.title);
        draw_body(
            presentation,
            &format!(
                "From {}: {} shells",
                gift.donor_name.as_deref().unwrap_or("[missing profile]"),
                gift.donor_shell_balance.unwrap_or(0)
            ),
            layout.donor,
            false,
        );
        draw_body(
            presentation,
            "Choose recipient",
            layout.recipient_label,
            true,
        );
        fill(layout.recipients, layout.panel, LIST_BACKGROUND);
        for (visible, row) in gift
            .recipients
            .iter()
            .skip(view.first_row)
            .take(VISIBLE_ROWS)
            .enumerate()
        {
            let bounds = layout.recipient_row(visible);
            let selected = Some(row.id) == gift.selected_recipient_id;
            if bounds.contains(pointer) || self.captured_recipient == Some(row.id) {
                fill(bounds, layout.recipients, Color::new(0.30, 0.24, 0.16, 1.0));
            }
            let name = format!("{}{}", row.name, if row.is_donor { " (you)" } else { "" });
            label(
                presentation,
                LIST_FONT,
                &name,
                bounds,
                if selected { ACCENT } else { INK },
                true,
            );
            if selected && self.controls.is_focused(RECIPIENT_CONTROL) {
                outline(bounds, layout.recipients, 1.0, ACCENT);
            }
        }
        outline(
            layout.recipients,
            layout.panel,
            1.0,
            if self.controls.is_focused(RECIPIENT_CONTROL) {
                ACCENT
            } else {
                DIM_INK
            },
        );
        if layout.scrollbar_visible {
            draw_button(
                presentation,
                layout.scroll_up,
                "^",
                view.first_row > 0 && !self.suspended && !self.pending_confirmation,
                self.controls.is_pressed(UP_CONTROL),
                self.controls.is_focused(UP_CONTROL),
            );
            draw_button(
                presentation,
                layout.scroll_down,
                "v",
                view.first_row + VISIBLE_ROWS < gift.recipients.len()
                    && !self.suspended
                    && !self.pending_confirmation,
                self.controls.is_pressed(DOWN_CONTROL),
                self.controls.is_focused(DOWN_CONTROL),
            );
            fill(layout.scroll_track, layout.panel, LIST_BACKGROUND);
            let count = gift.recipients.len().max(VISIBLE_ROWS);
            let thumb_height =
                (layout.scroll_track.h * VISIBLE_ROWS as f32 / count as f32).max(8.0);
            let maximum = count.saturating_sub(VISIBLE_ROWS);
            let fraction = if maximum == 0 {
                0.0
            } else {
                view.first_row as f32 / maximum as f32
            };
            fill(
                Rect::new(
                    layout.scroll_track.x + 3.0,
                    layout.scroll_track.y + fraction * (layout.scroll_track.h - thumb_height),
                    layout.scroll_track.w - 6.0,
                    thumb_height,
                ),
                layout.panel,
                DIM_INK,
            );
        }
        draw_body(presentation, gift.amount_label, layout.amount_label, false);
        if let Some(texture) = presentation.images.get("IMAGE_HELPSHELL") {
            image(
                texture,
                Rect::new(0.0, 0.0, texture.width(), texture.height()),
                layout.shell,
                layout.panel,
                WHITE,
                false,
            );
        }
        draw_edit_box(presentation, layout.amount, "", false);
        let amount_bounds = Rect::new(
            layout.amount.x + 8.0,
            layout.amount.y + 4.0,
            layout.amount.w - 16.0,
            24.0,
        );
        // Shared helper owns the skin. UserDialog's edit/list font and this
        // cursor viewport own the text, avoiding a second end-of-text caret.
        let (amount_display, caret_offset) =
            amount_window(&gift.amount_text, self.cursor, amount_bounds.w, |text| {
                text_width(presentation, LIST_FONT, text)
            });
        label(
            presentation,
            LIST_FONT,
            amount_display,
            amount_bounds,
            BLACK,
            false,
        );
        if self.select_all && self.controls.is_focused(AMOUNT_CONTROL) {
            outline(amount_bounds, layout.amount, 1.0, ACCENT);
        } else if self.controls.is_focused(AMOUNT_CONTROL)
            && !self.suspended
            && !self.pending_confirmation
        {
            let caret = amount_bounds.x + caret_offset;
            fill(
                Rect::new(caret, layout.amount.y + 7.0, 1.0, 18.0),
                layout.amount,
                ACCENT,
            );
        }

        let lines = detail_lines(
            presentation,
            gift,
            profiles.current_id,
            layout.details.w - 16.0,
        );
        self.detail_scroll = self
            .detail_scroll
            .min(lines.len().saturating_sub(DETAIL_LINES));
        fill(layout.details, layout.panel, LIST_BACKGROUND);
        for (index, (text, color)) in lines
            .iter()
            .skip(self.detail_scroll)
            .take(DETAIL_LINES)
            .enumerate()
        {
            label(
                presentation,
                BODY_FONT,
                text,
                Rect::new(
                    layout.details.x + 8.0,
                    layout.details.y + 2.0 + index as f32 * 18.0,
                    layout.details.w - 16.0,
                    18.0,
                ),
                *color,
                false,
            );
        }
        outline(
            layout.details,
            layout.panel,
            1.0,
            if self.controls.is_focused(DETAILS_CONTROL) {
                ACCENT
            } else {
                DIM_INK
            },
        );
        // A view alone is not a painted preview. Acknowledge only after every
        // accounting/capacity line was submitted within the visible clip.
        let accounting_lines = accounting_messages(gift)
            .iter()
            .map(|(message, _)| wrap_text(presentation, message, layout.details.w - 16.0).len())
            .sum::<usize>();
        self.preview_painted(
            gift.preview,
            self.detail_scroll == 0
                && accounting_lines <= DETAIL_LINES
                && presentation.fonts.contains_key(BODY_FONT),
        );
        draw_button(
            presentation,
            layout.confirm,
            if self.pending_confirmation {
                "Saving..."
            } else {
                gift.confirm_label
            },
            gift.can_confirm && !self.pending_confirmation && !self.suspended,
            self.controls.is_pressed(CONFIRM_CONTROL),
            self.controls.is_focused(CONFIRM_CONTROL),
        );
        draw_button(
            presentation,
            layout.cancel,
            gift.cancel_label,
            !self.pending_confirmation && !self.suspended,
            self.controls.is_pressed(CANCEL_CONTROL),
            self.controls.is_focused(CANCEL_CONTROL),
        );
        let footer = if self.suspended {
            "Input paused: focus or another modal."
        } else if self.review_required {
            "Preview changed. Review, then Give again."
        } else if REQUIRED_FONT_NAMES
            .iter()
            .any(|name| !presentation.fonts.contains_key(*name))
        {
            "Font unavailable; giving waits for text."
        } else if lines.len() > DETAIL_LINES {
            "Scroll details for full error; Esc cancels."
        } else {
            "Tab: focus | Arrows: scroll | Esc: cancel"
        };
        label(
            presentation,
            BODY_FONT,
            footer,
            layout.footer,
            DIM_INK,
            false,
        );
    }

    fn preview_painted(&mut self, preview: Option<ShellGiftPreview>, fully_visible: bool) {
        if fully_visible {
            self.shown_preview = preview;
        } else if self.shown_preview != preview {
            self.shown_preview = None;
        }
    }
}

fn amount_edited(input: &ShellGiftInput) -> bool {
    input.key(KeyCode::Backspace)
        || input.key(KeyCode::Delete)
        || !input.text.is_empty()
        || input.text_overflow
}

fn amount_window(
    text: &str,
    cursor: usize,
    width: f32,
    measure: impl Fn(&str) -> f32,
) -> (&str, f32) {
    let width = (width - 2.0).max(0.0);
    let mut start = 0;
    while start < cursor && measure(&text[start..cursor]) > width {
        start += text[start..].chars().next().map_or(0, char::len_utf8);
    }
    let mut end = text.len();
    while end > cursor && measure(&text[start..end]) > width {
        end = text[..end]
            .char_indices()
            .next_back()
            .map_or(0, |(index, _)| index);
    }
    (&text[start..end], measure(&text[start..cursor]).min(width))
}

fn accounting_messages(view: &ShellGiftView) -> Vec<(String, Color)> {
    let mut messages = Vec::new();
    if let Some(gift) = view.preview {
        messages.push((
            format!(
                "Debit {} / requested {}",
                gift.actual_debit, gift.requested_debit
            ),
            INK,
        ));
        messages.push((
            format!(
                "Credit {} / cap loss {}",
                gift.recipient_credit, gift.capped_excess
            ),
            if gift.capped_excess > 0 { YELLOW } else { INK },
        ));
        if gift.self_transfer {
            messages.push((
                "Giving to yourself closes without changing either wallet.".into(),
                DIM_INK,
            ));
        }
    }
    if let Some(notice) = &view.capacity_notice {
        messages.push((notice.clone(), YELLOW));
    }
    messages
}

fn detail_lines(
    presentation: &Presentation,
    view: &ShellGiftView,
    current_id: Option<u64>,
    width: f32,
) -> Vec<(String, Color)> {
    let mut messages = accounting_messages(view);
    if current_id != Some(view.donor_id) {
        messages.push((ShellGiftError::DonorChanged.to_string(), YELLOW));
    }
    if let Some(error) = &view.validation_error {
        messages.push((error.to_string(), YELLOW));
    }
    if let Some(error) = &view.confirmation_error
        && view.validation_error.as_ref() != Some(error)
    {
        messages.push((error.to_string(), ORANGE));
    }
    messages
        .into_iter()
        .flat_map(|(text, color)| {
            wrap_text(presentation, &text, width)
                .into_iter()
                .map(move |line| (line, color))
        })
        .collect()
}

fn wrap_text(presentation: &Presentation, text: &str, width: f32) -> Vec<String> {
    wrap_measured(text, width, |text| {
        text_width(presentation, BODY_FONT, text)
    })
}

fn wrap_measured(text: &str, width: f32, measure: impl Fn(&str) -> f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let proposed = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if measure(&proposed) <= width {
            line = proposed;
            continue;
        }
        if !line.is_empty() {
            lines.push(std::mem::take(&mut line));
        }
        for character in word.chars() {
            let proposed = format!("{line}{character}");
            if !line.is_empty() && measure(&proposed) > width {
                lines.push(std::mem::take(&mut line));
            }
            line.push(character);
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

fn display_text(presentation: &Presentation, stem: &str, text: &str) -> String {
    if let Some(font) = presentation.fonts.get(stem) {
        text.chars()
            .map(|character| {
                if font.metrics.measure_width(&character.to_string()).is_ok() {
                    character
                } else {
                    '?'
                }
            })
            .collect()
    } else {
        text.chars()
            .map(|character| if character.is_ascii() { character } else { '?' })
            .collect()
    }
}

fn text_width(presentation: &Presentation, stem: &str, text: &str) -> f32 {
    let text = display_text(presentation, stem, text);
    if let Some(font) = presentation.fonts.get(stem) {
        font.metrics.measure_width(&text).unwrap_or(0) as f32
    } else {
        // Only a diagnostic layout estimate. Missing fonts cannot acknowledge
        // accounting and root owns missing-resource diagnostics.
        text.chars().count() as f32 * 8.0
    }
}

fn label(
    presentation: &Presentation,
    stem: &str,
    text: &str,
    area: Rect,
    color: Color,
    centered: bool,
) {
    let mut text = display_text(presentation, stem, text);
    if text_width(presentation, stem, &text) > area.w {
        while !text.is_empty() && text_width(presentation, stem, &format!("{text}...")) > area.w {
            text.pop();
        }
        text.push_str("...");
    }
    let width = text_width(presentation, stem, &text);
    let x = area.x
        + if centered {
            ((area.w - width) / 2.0).max(0.0)
        } else {
            0.0
        };
    if let Some(font) = presentation.fonts.get(stem) {
        let ascent = font
            .metrics
            .layers
            .iter()
            .map(|layer| layer.ascent)
            .max()
            .unwrap_or(1)
            .max(1) as f32;
        let baseline = area.y + (area.h - ascent) / 2.0 + ascent;
        font_text(font, &text, x, baseline, color, area);
    } else {
        draw_body(presentation, &text, area, centered);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{adventure::AdventurePhase, sim::PetKind};
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn fixture(count: usize) -> Profiles {
        let mut profiles = Profiles::new("Donor", 701).unwrap();
        for index in 2..=count {
            let mut entry = Profiles::new(&format!("User{index:02}"), 700 + index as u64)
                .unwrap()
                .entries
                .remove(0);
            entry.id = index as u64;
            entry.session.progress.shell_balance = 20;
            profiles.entries.push(entry);
        }
        profiles.next_id = count as u64 + 1;
        let donor = profiles.current_mut().unwrap();
        donor.session.progress.tank = 2;
        donor.session.progress.level = 1;
        donor.session.progress.unlocked_pets = vec![
            PetKind::Stinky,
            PetKind::Niko,
            PetKind::Itchy,
            PetKind::Prego,
            PetKind::Zorf,
        ];
        donor.session.progress.first_stage_best_seconds = Some(42);
        donor.session.progress.shell_balance = 1000;
        donor.session.board = None;
        donor.session.phase = AdventurePhase::GameSelector;
        profiles.validate().unwrap();
        profiles
    }

    fn neutral() -> ShellGiftInput {
        ShellGiftInput::from_modal(ModalInput {
            focused: true,
            ..ModalInput::default()
        })
    }

    fn key(key: KeyCode) -> ShellGiftInput {
        ShellGiftInput {
            keys: vec![key],
            ..neutral()
        }
    }

    fn text(text: &str) -> ShellGiftInput {
        ShellGiftInput::from_modal(ModalInput {
            focused: true,
            text: text.into(),
            ..ModalInput::default()
        })
    }

    fn armed(profiles: &Profiles) -> ShellGiftRuntime {
        let mut runtime = ShellGiftRuntime::open(profiles).unwrap();
        assert!(runtime.input(profiles, &neutral()).consumed);
        runtime
    }

    // This simulates the rendering acknowledgement, not a native/pixel check.
    fn acknowledge_preview(runtime: &mut ShellGiftRuntime, profiles: &Profiles) {
        let preview = runtime.view(profiles).gift.preview;
        runtime.preview_painted(preview, true);
    }

    fn proposal(profiles: &Profiles, amount: &str) -> ShellGiftRuntime {
        let mut runtime = armed(profiles);
        runtime.select_recipient(2);
        runtime.input(profiles, &text(amount));
        assert!(runtime.view(profiles).gift.can_confirm);
        acknowledge_preview(&mut runtime, profiles);
        runtime
    }

    fn press(runtime: &mut ShellGiftRuntime, profiles: &Profiles, pointer: Vec2) -> ShellGiftFrame {
        runtime.input(
            profiles,
            &ShellGiftInput {
                pointer,
                pressed: true,
                down: true,
                ..neutral()
            },
        )
    }

    fn release(
        runtime: &mut ShellGiftRuntime,
        profiles: &Profiles,
        pointer: Vec2,
    ) -> ShellGiftFrame {
        runtime.input(
            profiles,
            &ShellGiftInput {
                pointer,
                released: true,
                ..neutral()
            },
        )
    }

    #[test]
    fn source_layout_and_initial_amount_focus_fit_logical_canvas() {
        let profiles = fixture(12);
        let mut runtime = armed(&profiles);
        assert_eq!(runtime.controls.focus(), Some(AMOUNT_CONTROL));
        let view = runtime.view(&profiles);
        let layout = view.layout;
        // G1-G6 independent source facts plus the explicit safe-bounds contract.
        assert_eq!(
            (layout.panel.w, layout.recipients.h, ROW_HEIGHT),
            (400.0, 130.0, 15.0)
        );
        assert_eq!(layout.cancel.x - layout.confirm.right(), 8.0);
        assert_eq!(layout.confirm.w, layout.cancel.w);
        assert_eq!(layout.panel.bottom() - layout.confirm.bottom(), 36.0);
        assert_eq!(layout.scroll_up.w, 16.0);
        for bounds in [
            layout.title,
            layout.donor,
            layout.recipient_label,
            layout.recipients,
            layout.scroll_up,
            layout.scroll_down,
            layout.scroll_track,
            layout.amount_label,
            layout.shell,
            layout.amount,
            layout.details,
            layout.confirm,
            layout.cancel,
            layout.footer,
        ] {
            assert!(layout.panel.contains(vec2(bounds.x, bounds.y)));
            assert!(layout.panel.contains(vec2(bounds.right(), bounds.bottom())));
        }
        assert!(
            Rect::new(0.0, 0.0, 640.0, 480.0)
                .contains(vec2(layout.panel.right(), layout.panel.bottom()))
        );
        assert_eq!(
            view.recipient_at(vec2(layout.recipients.x + 5.0, layout.recipients.y + 120.0)),
            None
        );
        assert!(ShellGiftLayout::new(8).scrollbar_visible);
        assert!(!ShellGiftLayout::new(7).scrollbar_visible);
        assert_eq!(
            ShellGiftLayout::new(7).recipients.w - layout.recipients.w,
            16.0
        );
    }

    #[test]
    fn recipient_release_cannot_retarget_reordered_or_deleted_profile() {
        let mut profiles = fixture(3);
        let mut runtime = armed(&profiles);
        let row = runtime.view(&profiles).layout.recipient_row(1).center();
        assert_eq!(press(&mut runtime, &profiles, row).intent, None);
        profiles.entries.swap(1, 2);
        assert_eq!(release(&mut runtime, &profiles, row).intent, None);
        assert_eq!(runtime.view(&profiles).gift.selected_recipient_id, None);
        press(&mut runtime, &profiles, row);
        release(&mut runtime, &profiles, row);
        assert_eq!(runtime.view(&profiles).gift.selected_recipient_id, Some(3));
        profiles.entries[1].name = "Renamed".into();
        profiles.entries.swap(1, 2);
        let view = runtime.view(&profiles);
        assert_eq!(view.gift.selected_recipient_id, Some(3));
        assert_eq!(
            view.gift
                .recipients
                .iter()
                .find(|entry| entry.id == 3)
                .unwrap()
                .name,
            "Renamed"
        );
        press(&mut runtime, &profiles, row);
        profiles.entries.retain(|entry| entry.id != 2);
        release(&mut runtime, &profiles, row);
        assert_eq!(runtime.view(&profiles).gift.selected_recipient_id, Some(3));
        profiles.entries.retain(|entry| entry.id != 3);
        runtime.input(&profiles, &text("7"));
        let view = runtime.view(&profiles);
        assert_eq!(view.gift.selected_recipient_id, Some(3));
        assert!(!view.gift.can_confirm);
    }

    #[test]
    fn opening_disabled_actions_and_focus_loss_consume_without_activation() {
        let profiles = fixture(3);
        let mut runtime = ShellGiftRuntime::open(&profiles).unwrap();
        let layout = runtime.view(&profiles).layout;
        assert_eq!(
            press(&mut runtime, &profiles, layout.cancel.center()).intent,
            None
        );
        assert_eq!(
            release(&mut runtime, &profiles, layout.cancel.center()).intent,
            None
        );
        assert!(runtime.is_open());
        press(&mut runtime, &profiles, layout.confirm.center());
        assert_eq!(
            release(&mut runtime, &profiles, layout.confirm.center()).intent,
            None
        );
        assert!(runtime.confirmation_dialog().is_none());
        for blocked in [false, true] {
            press(&mut runtime, &profiles, layout.cancel.center());
            let frame = runtime.input(
                &profiles,
                &ShellGiftInput {
                    focused: blocked,
                    blocked_by_modal: blocked,
                    down: true,
                    ..ShellGiftInput::default()
                },
            );
            assert!(frame.consumed);
            assert_eq!(frame.intent, None);
            assert_eq!(
                release(&mut runtime, &profiles, layout.cancel.center()).intent,
                None
            );
            assert_eq!(
                release(&mut runtime, &profiles, layout.cancel.center()).intent,
                None
            );
            assert!(runtime.is_open());
        }
        press(&mut runtime, &profiles, layout.cancel.center());
        let frame = release(&mut runtime, &profiles, layout.cancel.center());
        assert!(frame.consumed);
        assert_eq!(frame.intent, Some(ShellGiftIntent::Cancel));
        assert!(!runtime.input(&profiles, &neutral()).consumed);
    }

    #[test]
    fn normalized_text_is_whole_strict_and_utf8_cursor_safe() {
        let profiles = fixture(3);
        let mut runtime = armed(&profiles);
        runtime.select_recipient(2);
        for invalid in ["-7", "1.5", "1e2", "é7"] {
            runtime.select_all = true;
            runtime.input(&profiles, &text(invalid));
            assert_eq!(runtime.view(&profiles).gift.amount_text, invalid);
            assert!(!runtime.view(&profiles).gift.can_confirm);
        }
        runtime.input(&profiles, &key(KeyCode::Home));
        runtime.input(&profiles, &key(KeyCode::Right));
        runtime.input(&profiles, &key(KeyCode::Backspace));
        assert_eq!(runtime.view(&profiles).gift.amount_text, "7");
        assert!(runtime.view(&profiles).gift.can_confirm);
        runtime.input(&profiles, &text(&"9".repeat(MAX_AMOUNT_TEXT_BYTES + 1)));
        let view = runtime.view(&profiles);
        assert_eq!(view.gift.amount_text, "");
        assert_eq!(
            view.gift.validation_error,
            Some(ShellGiftError::AmountOutOfRange)
        );
        assert!(!view.gift.can_confirm);
        runtime.input(
            &profiles,
            &ShellGiftInput {
                control: true,
                keys: vec![KeyCode::A],
                text: "0050".into(),
                ..neutral()
            },
        );
        assert_eq!(
            runtime
                .view(&profiles)
                .gift
                .preview
                .unwrap()
                .requested_debit,
            50
        );
    }

    #[test]
    fn view_and_same_frame_edit_cannot_approve_unpainted_or_changed_accounting() {
        let mut profiles = fixture(3);
        let mut runtime = armed(&profiles);
        runtime.select_recipient(2);
        let mut edit_and_accept = text("50");
        edit_and_accept.keys.push(KeyCode::Enter);
        assert_eq!(runtime.input(&profiles, &edit_and_accept).intent, None);
        assert_eq!(
            runtime
                .view(&profiles)
                .gift
                .preview
                .unwrap()
                .requested_debit,
            50
        );
        assert_eq!(runtime.input(&profiles, &key(KeyCode::Enter)).intent, None);
        acknowledge_preview(&mut runtime, &profiles);
        profiles.entries[1].session.progress.shell_balance = 9_999_990;
        assert_eq!(runtime.input(&profiles, &key(KeyCode::Enter)).intent, None);
        assert!(runtime.review_required);
        let preview = runtime.view(&profiles).gift.preview.unwrap();
        assert_eq!(
            (
                preview.actual_debit,
                preview.recipient_credit,
                preview.capped_excess
            ),
            (50, 9, 41)
        );
        runtime.preview_painted(Some(preview), false);
        assert_eq!(runtime.input(&profiles, &key(KeyCode::Enter)).intent, None);
        acknowledge_preview(&mut runtime, &profiles);
        assert_eq!(
            runtime.input(&profiles, &key(KeyCode::Enter)).intent,
            Some(ShellGiftIntent::Confirm(ShellGiftRequest {
                donor_id: 1,
                recipient_id: 2,
                amount: 50
            }))
        );
        assert_eq!(runtime.input(&profiles, &key(KeyCode::Escape)).intent, None);
        runtime.confirmation_finished(Err(ShellGiftError::TransactionFailed(
            "test owner rejected write".into(),
        )));
        assert!(runtime.is_open());
        assert_eq!(
            runtime.view(&profiles).gift.confirmation_error,
            Some(ShellGiftError::TransactionFailed(
                "test owner rejected write".into()
            ))
        );
        assert_eq!(runtime.input(&profiles, &key(KeyCode::Enter)).intent, None);
    }

    #[test]
    fn amount_edit_on_give_press_invalidates_previously_painted_proposal() {
        let profiles = fixture(3);
        let mut runtime = proposal(&profiles, "50");
        let confirm = runtime.view(&profiles).layout.confirm.center();
        let mut input = text("7");
        input.pointer = confirm;
        input.pressed = true;
        input.down = true;
        assert_eq!(runtime.input(&profiles, &input).intent, None);
        assert_eq!(runtime.view(&profiles).gift.amount_text, "507");
        assert_eq!(release(&mut runtime, &profiles, confirm).intent, None);
        assert!(runtime.confirmation_dialog().is_none());
    }

    #[test]
    fn accounting_and_long_error_wrapping_keep_full_text_and_bounds() {
        let mut profiles = fixture(3);
        profiles.entries[0].session.progress.shell_balance = 9_999_999;
        profiles.entries[1].session.progress.shell_balance = 9_999_999;
        let mut runtime = proposal(&profiles, "9999999");
        let view = runtime.view(&profiles);
        let width = view.layout.details.w - 16.0;
        // Deterministic synthetic 8px metrics exercise the clipping contract;
        // they do not prove owned-font/native visual parity.
        let accounting = accounting_messages(&view.gift)
            .into_iter()
            .flat_map(|(message, _)| {
                wrap_measured(&message, width, |text| text.chars().count() as f32 * 8.0)
            })
            .collect::<Vec<_>>();
        assert!(accounting.len() <= DETAIL_LINES);
        assert!(
            accounting
                .iter()
                .all(|line| line.chars().count() as f32 * 8.0 <= width)
        );
        assert!(accounting.join(" ").contains("Credit 0 / cap loss 9999999"));
        let long = "database write failed ".repeat(24) + &"X".repeat(100);
        let lines = wrap_measured(&long, width, |text| text.chars().count() as f32 * 8.0);
        assert!(lines.len() > DETAIL_LINES);
        assert!(
            lines
                .iter()
                .all(|line| line.chars().count() as f32 * 8.0 <= width)
        );
        assert_eq!(lines.concat().replace(' ', ""), long.replace(' ', ""));
        let amount = "0".repeat(30) + "50";
        let (window, caret) =
            amount_window(&amount, amount.len(), view.layout.amount.w - 16.0, |text| {
                text.chars().count() as f32 * 8.0
            });
        assert!(window.ends_with("50"));
        assert!(window.chars().count() as f32 * 8.0 <= view.layout.amount.w - 16.0);
        assert!(caret <= view.layout.amount.w - 16.0);
    }

    struct DisposableDirectory {
        path: PathBuf,
        root: PathBuf,
    }

    impl DisposableDirectory {
        fn new() -> Self {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".scratch/modal307/fixtures");
            fs::create_dir_all(&root).unwrap();
            let root = root.canonicalize().unwrap();
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = root.join(format!("transaction-{}-{nanos}", std::process::id()));
            fs::create_dir(&path).unwrap();
            Self { path, root }
        }
    }

    impl Drop for DisposableDirectory {
        fn drop(&mut self) {
            assert_eq!(
                self.path.canonicalize().unwrap().parent(),
                Some(self.root.as_path())
            );
            fs::remove_dir_all(&self.path).unwrap();
        }
    }

    #[test]
    fn actual_caller_transaction_feedback_preserves_failure_and_closes_success() {
        let directory = DisposableDirectory::new();
        let path = directory.path.join("profiles.json");
        let mut profiles = fixture(3);
        profiles.save(&path).unwrap();
        let baseline = serde_json::to_value(&profiles).unwrap();
        let baseline_disk = fs::read(&path).unwrap();
        // A failed atomic write can retain a sibling .pending file. Keep both
        // the directory target and that sibling inside this disposable owner.
        let invalid_target = directory.path.join("failure-directory");
        fs::create_dir(&invalid_target).unwrap();
        let mut runtime = proposal(&profiles, "250");
        assert!(matches!(
            runtime.input(&profiles, &key(KeyCode::Enter)).intent,
            Some(ShellGiftIntent::Confirm(_))
        ));
        let failure = runtime
            .confirmation_dialog()
            .unwrap()
            .confirm(&mut profiles, &invalid_target);
        assert!(matches!(failure, Err(ShellGiftError::TransactionFailed(_))));
        runtime.confirmation_finished(failure);
        assert!(runtime.is_open());
        assert_eq!(serde_json::to_value(&profiles).unwrap(), baseline);
        assert_eq!(fs::read(&path).unwrap(), baseline_disk);
        assert!(runtime.view(&profiles).gift.confirmation_error.is_some());
        runtime.input(&profiles, &neutral());
        acknowledge_preview(&mut runtime, &profiles);
        assert!(matches!(
            runtime.input(&profiles, &key(KeyCode::Enter)).intent,
            Some(ShellGiftIntent::Confirm(_))
        ));
        let result = runtime
            .confirmation_dialog()
            .unwrap()
            .confirm(&mut profiles, &path);
        assert_eq!(result.as_ref().unwrap().recipient_credit, 250);
        runtime.confirmation_finished(result);
        assert!(!runtime.is_open());
        let reopened = Profiles::decode(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(reopened.entries[0].session.progress.shell_balance, 750);
        assert_eq!(reopened.entries[1].session.progress.shell_balance, 270);
        assert_eq!(reopened.entries[2].session.progress.shell_balance, 20);
    }
}
