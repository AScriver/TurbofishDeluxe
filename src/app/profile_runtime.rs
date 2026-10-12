//! Caller-owned profile modal presentation. Common modal drawing owns skins;
//! the pure view owns every hit/layout rectangle. No native input polling or
//! profile writes occur here. Root draws the dim layer once and owns routing.

use super::{Presentation, help_runtime, modal_runtime};
use crate::{
    modal::ModalInput,
    profile_modal::{ProfileFrame, ProfileModal, ProfileRequest, ProfileView},
    profiles::Profiles,
    ui,
};
use macroquad::prelude::*;

pub(super) struct ProfileRuntime {
    modal: ProfileModal,
}

impl ProfileRuntime {
    pub(super) fn new(profiles: &Profiles) -> Self {
        Self {
            modal: ProfileModal::new(profiles),
        }
    }

    pub(super) fn is_open(&self) -> bool {
        self.modal.is_open()
    }

    pub(super) fn input(&mut self, input: &ModalInput, profiles: &Profiles) -> ProfileFrame {
        self.modal.input(input, profiles)
    }

    pub(super) fn view(&self, profiles: &Profiles) -> Option<ProfileView> {
        self.modal.view(profiles)
    }

    pub(super) fn cancel_capture(&mut self) {
        self.modal.cancel_capture();
    }

    pub(super) fn transaction_finished(
        &mut self,
        request: &ProfileRequest,
        result: Result<(), String>,
        profiles: &Profiles,
    ) -> bool {
        self.modal.transaction_finished(request, result, profiles)
    }

    pub(super) fn draw(&self, presentation: &Presentation, profiles: &Profiles) {
        let Some(view) = self.view(profiles) else {
            return;
        };
        modal_runtime::draw_panel(presentation, view.panel);
        modal_runtime::draw_title(presentation, view.title, view.title_bounds);
        modal_runtime::draw_body(presentation, &view.body, view.body_bounds, true);
        if let Some(list) = view.list_bounds {
            let brown = Color::from_rgba(53, 43, 29, 255);
            help_runtime::fill(list, list, brown);
            help_runtime::outline(list, list, 1.0, BLACK);
            let font = &presentation.fonts["ContinuumBold12"];
            let ascent = font
                .metrics
                .layers
                .iter()
                .map(|layer| layer.ascent)
                .max()
                .unwrap_or(1) as f32;
            for row in &view.rows {
                let bounds = row.control.bounds;
                let ink = if row.selected || row.pressed {
                    help_runtime::fill(bounds, list, Color::from_rgba(255, 225, 73, 255));
                    BLACK
                } else {
                    WHITE
                };
                let content = ui::inset(bounds, 2.0);
                let text = if row.current {
                    format!("* {}", row.name)
                } else {
                    row.name.clone()
                };
                let width = font.metrics.measure_width(&text).unwrap_or(0) as f32;
                help_runtime::font_text(
                    font,
                    &text,
                    ui::centered_text_x_clamped(content, width),
                    ui::centered_ascent_baseline(content, ascent),
                    ink,
                    content,
                );
                if row.focused {
                    help_runtime::outline(bounds, list, 1.0, YELLOW);
                }
            }
        }
        if let Some(edit) = &view.edit {
            let content = ui::inset(edit.bounds, 8.0);
            let font = &presentation.fonts["ContinuumBold12"];
            let range = edit.visible_range((content.w - 2.0).max(0.0), |text| {
                font.metrics.measure_width(text).unwrap_or(0) as f32
            });
            let text = &edit.text[range.clone()];
            // Common drawing puts its caret at the string end. This editor
            // supplies the actual insertion caret after drawing the same skin.
            modal_runtime::draw_edit_box(presentation, edit.bounds, text, false);
            if edit.focused && edit.enabled {
                if edit.select_all {
                    help_runtime::fill(content, content, Color::new(0.23, 0.38, 0.55, 1.0));
                    let ascent = font
                        .metrics
                        .layers
                        .iter()
                        .map(|layer| layer.ascent)
                        .max()
                        .unwrap_or(1) as f32;
                    help_runtime::font_text(
                        font,
                        text,
                        content.x,
                        ui::centered_ascent_baseline(content, ascent),
                        WHITE,
                        content,
                    );
                } else {
                    let prefix = &edit.text[range.start..edit.cursor];
                    let width = font.metrics.measure_width(prefix).unwrap_or(0) as f32;
                    let x = (content.x + width + 1.0).min(content.right() - 1.0);
                    draw_line(x, content.y + 1.0, x, content.bottom() - 1.0, 1.0, YELLOW);
                }
                help_runtime::outline(edit.bounds, edit.bounds, 1.0, YELLOW);
            }
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
        if !view.position.is_empty() {
            modal_runtime::draw_body(presentation, &view.position, view.position_bounds, true);
        }
        if !view.feedback.is_empty() {
            modal_runtime::draw_body(presentation, &view.feedback, view.feedback_bounds, false);
        }
    }
}
