//! Transient shell-gift frontend over the production controller.
//!
//! Project-authored geometry on the parent's 640x480 logical canvas: panel
//! (32,12,576,456), four 30px recipient rows, a 32px amount field, scrollable
//! accounting/errors and two release-activated buttons. This is not a recovered
//! UserDialog layout. No images, clocks, sessions, wallets or save paths are owned.
//!
//! Parent hooks:
//! - Register this child and preload `REQUIRED_FONT_NAMES` (no image dependency).
//! - Open on `SelectorIntent::Dialog(DialogIntent::OpenGiveDialog)`, using the
//!   current authoritative `Profiles`.
//! - While open, route input here before selector/game input and consume the
//!   entire frame, including closing frames. Pause underlying UI/simulation.
//! - Pass logical pointer coordinates and actual owned-window focus/modality to
//!   `ShellGiftInput::poll`; `input` and `draw` rebuild production views.
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
    help_runtime::{fill, font_text, outline},
};
use crate::{
    profiles::Profiles,
    shell_gift::{
        MAX_AMOUNT_TEXT_BYTES, ShellGiftDialog, ShellGiftError, ShellGiftPreview, ShellGiftRequest,
        ShellGiftView,
    },
};
use macroquad::prelude::*;

pub(super) const REQUIRED_IMAGE_IDS: &[&str] = &[];
pub(super) const REQUIRED_FONT_NAMES: &[&str] = &["ContinuumBold12", "JungleFever17outline"];

const BODY_FONT: &str = "ContinuumBold12";
const TITLE_FONT: &str = "JungleFever17outline";
const SCREEN: Rect = Rect::new(0.0, 0.0, 640.0, 480.0);
const PANEL: Rect = Rect::new(32.0, 12.0, 576.0, 456.0);
const RECIPIENTS: Rect = Rect::new(48.0, 100.0, 514.0, 120.0);
const SCROLL_UP: Rect = Rect::new(568.0, 100.0, 24.0, 22.0);
const SCROLL_DOWN: Rect = Rect::new(568.0, 198.0, 24.0, 22.0);
const SCROLL_TRACK: Rect = Rect::new(568.0, 124.0, 24.0, 72.0);
const AMOUNT: Rect = Rect::new(182.0, 232.0, 410.0, 32.0);
const DETAILS: Rect = Rect::new(48.0, 276.0, 544.0, 120.0);
const CONFIRM: Rect = Rect::new(300.0, 410.0, 140.0, 34.0);
const CANCEL: Rect = Rect::new(452.0, 410.0, 140.0, 34.0);
const ROW_HEIGHT: f32 = 30.0;
const VISIBLE_ROWS: usize = 4;
const DETAIL_LINES: usize = 6;
const INK: Color = Color::new(0.94, 0.97, 1.0, 1.0);
const DIM_INK: Color = Color::new(0.65, 0.73, 0.80, 1.0);
const ACCENT: Color = Color::new(0.43, 0.84, 1.0, 1.0);
const SURFACE: Color = Color::new(0.08, 0.15, 0.23, 1.0);

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
    /// Call only while this modal owns input. Drains the character queue even
    /// when suspended, preventing delayed text after focus/modality changes.
    pub(super) fn poll(pointer: Vec2, focused: bool, blocked_by_modal: bool) -> Self {
        let control = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);
        let mut input = Self {
            pointer,
            focused,
            blocked_by_modal,
            pressed: is_mouse_button_pressed(MouseButton::Left),
            released: is_mouse_button_released(MouseButton::Left),
            down: is_mouse_button_down(MouseButton::Left),
            wheel: mouse_wheel().1,
            keys: get_keys_pressed().into_iter().collect(),
            shift: is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift),
            control,
            ..Self::default()
        };
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Focus {
    Recipients,
    Amount,
    Details,
    Confirm,
    Cancel,
}

impl Focus {
    fn cycle(self, backwards: bool) -> Self {
        let order = [
            Self::Recipients,
            Self::Amount,
            Self::Details,
            Self::Confirm,
            Self::Cancel,
        ];
        let index = order.iter().position(|focus| *focus == self).unwrap_or(0);
        order[(index + if backwards { order.len() - 1 } else { 1 }) % order.len()]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Control {
    Recipient(u64),
    Amount,
    Details,
    ScrollUp,
    ScrollDown,
    Confirm,
    Cancel,
}

impl Control {
    fn focus(self) -> Focus {
        match self {
            Self::Recipient(_) | Self::ScrollUp | Self::ScrollDown => Focus::Recipients,
            Self::Amount => Focus::Amount,
            Self::Details => Focus::Details,
            Self::Confirm => Focus::Confirm,
            Self::Cancel => Focus::Cancel,
        }
    }
}

pub(super) struct ShellGiftRuntime {
    dialog: ShellGiftDialog,
    focus: Focus,
    capture: Option<Control>,
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
        Ok(Self {
            dialog: ShellGiftDialog::open(profiles)?,
            focus: Focus::Recipients,
            capture: None,
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
        self.capture = None;
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
        let view = self.refresh(profiles);
        self.suspended = !input.focused || input.blocked_by_modal;
        if self.suspended || self.pending_confirmation {
            self.capture = None;
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
            self.focus = self.focus.cycle(input.shift);
            self.capture = None;
            self.select_all = false;
        }
        if input.wheel.is_finite() && input.wheel != 0.0 {
            let upwards = input.wheel > 0.0;
            if RECIPIENTS.contains(input.pointer) || SCROLL_TRACK.contains(input.pointer) {
                self.scroll_rows(view.recipients.len(), upwards, 1);
                self.capture = None;
            } else if DETAILS.contains(input.pointer) {
                self.scroll_details(upwards, 1);
            }
        }
        if input.pressed {
            self.capture = self.hit(&view, input.pointer);
            if let Some(control) = self.capture {
                self.focus = control.focus();
                self.select_all = false;
                if control == Control::Amount {
                    self.cursor = view.amount_text.len();
                }
            }
        }
        if input.released {
            let captured = self.capture.take();
            if let Some(control) = captured
                && self.hit(&view, input.pointer) == Some(control)
            {
                match control {
                    Control::Recipient(id) => self.select_recipient(id),
                    Control::ScrollUp => self.scroll_rows(view.recipients.len(), true, 1),
                    Control::ScrollDown => self.scroll_rows(view.recipients.len(), false, 1),
                    Control::Confirm => frame.intent = self.request_confirmation(profiles),
                    Control::Cancel => {
                        self.cancel();
                        frame.intent = Some(ShellGiftIntent::Cancel);
                    }
                    Control::Amount | Control::Details => {}
                }
            }
        } else if !input.down {
            self.capture = None;
        }
        if frame.intent.is_some() {
            return frame;
        }
        match self.focus {
            Focus::Recipients => self.recipient_keys(&view, input),
            Focus::Amount => self.amount_keys(&view.amount_text, input),
            Focus::Details => {
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
            Focus::Confirm | Focus::Cancel => {}
        }
        if input.key(KeyCode::Enter) || (input.key(KeyCode::Space) && self.focus != Focus::Amount) {
            if self.focus == Focus::Cancel {
                self.cancel();
                frame.intent = Some(ShellGiftIntent::Cancel);
            } else if self.focus == Focus::Confirm || self.focus == Focus::Amount {
                // A submitted edit needs a painted preview before a later Give
                // action. Never edit/cap the amount and confirm on the same frame.
                if self.focus != Focus::Amount || !amount_edited(input) {
                    frame.intent = self.request_confirmation(profiles);
                }
            } else if self.focus == Focus::Recipients {
                self.focus = Focus::Amount;
                self.select_all = true;
            }
        }
        frame
    }

    pub(super) fn cancel(&mut self) {
        self.dialog.cancel();
        self.capture = None;
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
        self.capture = None;
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
    }

    fn hit(&self, view: &ShellGiftView, pointer: Vec2) -> Option<Control> {
        if RECIPIENTS.contains(pointer) {
            let index = self.first_row + ((pointer.y - RECIPIENTS.y) / ROW_HEIGHT) as usize;
            return view
                .recipients
                .get(index)
                .map(|row| Control::Recipient(row.id));
        }
        if AMOUNT.contains(pointer) {
            Some(Control::Amount)
        } else if DETAILS.contains(pointer) {
            Some(Control::Details)
        } else if SCROLL_UP.contains(pointer) && self.first_row > 0 {
            Some(Control::ScrollUp)
        } else if SCROLL_DOWN.contains(pointer)
            && self.first_row + VISIBLE_ROWS < view.recipients.len()
        {
            Some(Control::ScrollDown)
        } else if CONFIRM.contains(pointer) && view.can_confirm {
            Some(Control::Confirm)
        } else if CANCEL.contains(pointer) {
            Some(Control::Cancel)
        } else {
            None
        }
    }

    pub(super) fn draw(&mut self, presentation: &Presentation, profiles: &Profiles, pointer: Vec2) {
        if !self.is_open() {
            return;
        }
        let view = self.refresh(profiles);
        fill(SCREEN, SCREEN, Color::new(0.0, 0.0, 0.0, 0.72));
        fill(PANEL, SCREEN, SURFACE);
        outline(PANEL, SCREEN, 2.0, ACCENT);
        label(
            presentation,
            TITLE_FONT,
            view.title,
            Rect::new(48.0, 20.0, 544.0, 30.0),
            INK,
            false,
        );
        let donor = format!(
            "From {} (#{}): {} shells",
            view.donor_name.as_deref().unwrap_or("Unavailable profile"),
            view.donor_id,
            view.donor_shell_balance
                .map_or_else(|| "unavailable".into(), |balance| balance.to_string())
        );
        label(
            presentation,
            BODY_FONT,
            &donor,
            Rect::new(48.0, 52.0, 544.0, 20.0),
            INK,
            false,
        );
        let selected = view
            .selected_recipient_id
            .map_or_else(|| "choose one".into(), |id| format!("selected #{}", id));
        label(
            presentation,
            BODY_FONT,
            &format!("Recipient ({selected})"),
            Rect::new(48.0, 76.0, 544.0, 20.0),
            DIM_INK,
            false,
        );
        fill(RECIPIENTS, SCREEN, Color::new(0.04, 0.08, 0.13, 1.0));
        for (visible, row) in view
            .recipients
            .iter()
            .skip(self.first_row)
            .take(VISIBLE_ROWS)
            .enumerate()
        {
            let bounds = Rect::new(
                RECIPIENTS.x,
                RECIPIENTS.y + visible as f32 * ROW_HEIGHT,
                RECIPIENTS.w,
                ROW_HEIGHT,
            );
            let control = Control::Recipient(row.id);
            let selected = Some(row.id) == view.selected_recipient_id;
            if selected || bounds.contains(pointer) {
                fill(bounds, RECIPIENTS, Color::new(0.12, 0.30, 0.42, 1.0));
            }
            let name = format!("{}{}", row.name, if row.is_donor { " (you)" } else { "" });
            label(
                presentation,
                BODY_FONT,
                &name,
                Rect::new(bounds.x + 8.0, bounds.y + 5.0, 340.0, 20.0),
                INK,
                false,
            );
            label(
                presentation,
                BODY_FONT,
                &format!("{} shells", row.shell_balance),
                Rect::new(bounds.x + 358.0, bounds.y + 5.0, 148.0, 20.0),
                DIM_INK,
                false,
            );
            if selected && self.focus == Focus::Recipients || self.capture == Some(control) {
                outline(bounds, RECIPIENTS, 1.0, ACCENT);
            }
        }
        outline(
            RECIPIENTS,
            SCREEN,
            1.0,
            if self.focus == Focus::Recipients {
                ACCENT
            } else {
                DIM_INK
            },
        );
        button(
            presentation,
            "^",
            SCROLL_UP,
            self.first_row > 0,
            self.capture == Some(Control::ScrollUp),
            false,
        );
        button(
            presentation,
            "v",
            SCROLL_DOWN,
            self.first_row + VISIBLE_ROWS < view.recipients.len(),
            self.capture == Some(Control::ScrollDown),
            false,
        );
        fill(SCROLL_TRACK, SCREEN, Color::new(0.04, 0.08, 0.13, 1.0));
        let count = view.recipients.len().max(VISIBLE_ROWS);
        let thumb_height = (SCROLL_TRACK.h * VISIBLE_ROWS as f32 / count as f32).max(8.0);
        let maximum = count.saturating_sub(VISIBLE_ROWS);
        let fraction = if maximum == 0 {
            0.0
        } else {
            self.first_row as f32 / maximum as f32
        };
        fill(
            Rect::new(
                SCROLL_TRACK.x + 3.0,
                SCROLL_TRACK.y + fraction * (SCROLL_TRACK.h - thumb_height),
                SCROLL_TRACK.w - 6.0,
                thumb_height,
            ),
            SCREEN,
            DIM_INK,
        );
        label(
            presentation,
            BODY_FONT,
            view.amount_label,
            Rect::new(48.0, 236.0, 126.0, 24.0),
            INK,
            false,
        );
        fill(AMOUNT, SCREEN, Color::new(0.04, 0.08, 0.13, 1.0));
        outline(
            AMOUNT,
            SCREEN,
            1.0,
            if self.focus == Focus::Amount {
                ACCENT
            } else {
                DIM_INK
            },
        );
        let amount_bounds = Rect::new(AMOUNT.x + 8.0, AMOUNT.y + 5.0, AMOUNT.w - 16.0, 22.0);
        if self.select_all && self.focus == Focus::Amount {
            fill(amount_bounds, AMOUNT, Color::new(0.15, 0.35, 0.50, 1.0));
        }
        label(
            presentation,
            BODY_FONT,
            &view.amount_text,
            amount_bounds,
            INK,
            false,
        );
        if self.focus == Focus::Amount && !self.select_all && !self.suspended {
            let prefix = &view.amount_text[..self.cursor];
            let caret = amount_bounds.x
                + text_width(presentation, BODY_FONT, prefix).min(amount_bounds.w - 1.0);
            fill(Rect::new(caret, AMOUNT.y + 7.0, 1.0, 18.0), AMOUNT, ACCENT);
        }
        let lines = detail_lines(presentation, &view, profiles.current_id);
        self.detail_scroll = self
            .detail_scroll
            .min(lines.len().saturating_sub(DETAIL_LINES));
        let accounting_lines = 2 + view.capacity_notice.as_ref().map_or(0, |notice| {
            wrap_text(presentation, notice, DETAILS.w - 16.0).len()
        });
        if self.detail_scroll == 0 && accounting_lines <= DETAIL_LINES {
            self.shown_preview = view.preview;
        } else if self.shown_preview != view.preview {
            self.shown_preview = None;
        }
        fill(DETAILS, SCREEN, Color::new(0.04, 0.08, 0.13, 1.0));
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
                    DETAILS.x + 8.0,
                    DETAILS.y + 4.0 + index as f32 * 18.0,
                    DETAILS.w - 16.0,
                    18.0,
                ),
                *color,
                false,
            );
        }
        outline(
            DETAILS,
            SCREEN,
            1.0,
            if self.focus == Focus::Details {
                ACCENT
            } else {
                DIM_INK
            },
        );
        button(
            presentation,
            if self.pending_confirmation {
                "Saving..."
            } else {
                view.confirm_label
            },
            CONFIRM,
            view.can_confirm && !self.pending_confirmation && !self.suspended,
            self.capture == Some(Control::Confirm),
            self.focus == Focus::Confirm,
        );
        button(
            presentation,
            view.cancel_label,
            CANCEL,
            !self.pending_confirmation && !self.suspended,
            self.capture == Some(Control::Cancel),
            self.focus == Focus::Cancel,
        );
        let footer = if self.suspended {
            "Input paused: window focus or another modal."
        } else if self.review_required {
            "Accounting changed. Review this preview, then Give again."
        } else if REQUIRED_FONT_NAMES
            .iter()
            .any(|name| !presentation.fonts.contains_key(*name))
        {
            "Font unavailable: built-in text fallback in use."
        } else if lines.len() > DETAIL_LINES {
            "Scroll inside details for full error. Tab changes focus; Esc cancels."
        } else {
            "Tab changes focus; arrows/Page keys scroll; Ctrl+A selects amount; Esc cancels."
        };
        label(
            presentation,
            BODY_FONT,
            footer,
            Rect::new(48.0, 446.0, 544.0, 18.0),
            DIM_INK,
            false,
        );
    }
}

fn amount_edited(input: &ShellGiftInput) -> bool {
    input.key(KeyCode::Backspace)
        || input.key(KeyCode::Delete)
        || !input.text.is_empty()
        || input.text_overflow
}

fn detail_lines(
    presentation: &Presentation,
    view: &ShellGiftView,
    current_id: Option<u64>,
) -> Vec<(String, Color)> {
    let mut messages = Vec::new();
    if current_id != Some(view.donor_id) {
        messages.push((ShellGiftError::DonorChanged.to_string(), YELLOW));
    }
    if let Some(gift) = view.preview {
        messages.push((
            format!(
                "Requested {} | actual donor debit {}",
                gift.requested_debit, gift.actual_debit
            ),
            INK,
        ));
        messages.push((
            format!(
                "Recipient credit {} | cap loss {}",
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
            wrap_text(presentation, &text, DETAILS.w - 16.0)
                .into_iter()
                .map(move |line| (line, color))
        })
        .collect()
}

fn wrap_text(presentation: &Presentation, text: &str, width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let proposed = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if text_width(presentation, BODY_FONT, &proposed) <= width {
            line = proposed;
            continue;
        }
        if !line.is_empty() {
            lines.push(std::mem::take(&mut line));
        }
        for character in word.chars() {
            let proposed = format!("{line}{character}");
            if !line.is_empty() && text_width(presentation, BODY_FONT, &proposed) > width {
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
        measure_text(&text, None, if stem == TITLE_FONT { 24 } else { 15 }, 1.0).width
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
    let size = if stem == TITLE_FONT { 24.0 } else { 15.0 };
    let baseline = area.y + (area.h + size) / 2.0 - 2.0;
    if let Some(font) = presentation.fonts.get(stem) {
        font_text(font, &text, x, baseline, color, area);
    } else {
        draw_text_ex(
            &text,
            x,
            baseline,
            TextParams {
                font_size: size as u16,
                color,
                ..Default::default()
            },
        );
    }
}

fn button(
    presentation: &Presentation,
    text: &str,
    bounds: Rect,
    enabled: bool,
    pressed: bool,
    focused: bool,
) {
    fill(
        bounds,
        SCREEN,
        if enabled && pressed {
            Color::new(0.09, 0.22, 0.32, 1.0)
        } else {
            Color::new(0.16, 0.27, 0.35, 1.0)
        },
    );
    outline(
        bounds,
        SCREEN,
        if focused { 2.0 } else { 1.0 },
        if enabled { ACCENT } else { DIM_INK },
    );
    label(
        presentation,
        BODY_FONT,
        text,
        bounds,
        if enabled { INK } else { DIM_INK },
        true,
    );
}
