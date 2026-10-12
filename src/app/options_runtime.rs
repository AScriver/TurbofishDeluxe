//! Owned-art Options/pause adapter. Native polling and transactions stay in app.

use super::{Presentation, modal_runtime};
use crate::{
    modal::ModalInput,
    options_modal::{
        OptionsContext, OptionsControlKind, OptionsFacts, OptionsIntent, OptionsModal, OptionsPage,
        OptionsSettingRequest, OptionsView,
    },
    settings::{GameSettings, SettingsCapabilities},
};

pub(super) fn facts(
    presentation: &Presentation,
    settings: GameSettings,
    capabilities: SettingsCapabilities,
    context_epoch: u64,
) -> OptionsFacts {
    OptionsFacts {
        settings,
        capabilities,
        context_epoch,
        slider_thumb_width: presentation
            .images
            .get("IMAGE_SLIDERTHUMB")
            .map_or(0.0, |image| image.width()),
    }
}

pub(super) struct OptionsRuntime {
    model: OptionsModal,
}

impl OptionsRuntime {
    pub(super) fn options(context: OptionsContext, context_epoch: u64) -> Self {
        Self {
            model: OptionsModal::options(context, context_epoch),
        }
    }

    pub(super) fn pause(context_epoch: u64) -> Self {
        Self {
            model: OptionsModal::pause(context_epoch),
        }
    }

    pub(super) fn input(
        &mut self,
        input: &ModalInput,
        facts: OptionsFacts,
    ) -> Option<OptionsIntent> {
        self.model.input(input, facts)
    }

    pub(super) fn finish_setting(
        &mut self,
        request: OptionsSettingRequest,
        result: Result<(), String>,
    ) -> bool {
        self.model.finish_setting(request, result)
    }

    pub(super) fn set_error(&mut self, error: impl Into<String>) {
        self.model.set_error(error);
    }

    pub(super) fn cancel_capture(&mut self) {
        self.model.cancel_capture();
    }

    pub(super) fn page(&self) -> OptionsPage {
        self.model.page()
    }

    pub(super) fn view(&self, facts: OptionsFacts) -> OptionsView {
        self.model.view(facts)
    }

    /// Parent paints one dim layer first. No keyboard/character polling here.
    pub(super) fn draw(&self, presentation: &Presentation, facts: OptionsFacts) {
        let view = self.view(facts);
        modal_runtime::draw_panel(presentation, view.panel);
        modal_runtime::draw_title(presentation, view.title, view.title_bounds);
        for item in &view.controls {
            match item.kind {
                OptionsControlKind::Button => modal_runtime::draw_button(
                    presentation,
                    item.paint_bounds,
                    item.label,
                    item.enabled,
                    item.pressed,
                    item.focused,
                ),
                OptionsControlKind::Checkbox { checked } => {
                    modal_runtime::draw_checkbox(
                        presentation,
                        item.paint_bounds,
                        checked,
                        item.enabled,
                        item.focused,
                    );
                    modal_runtime::draw_body(presentation, item.label, item.label_bounds, false);
                }
                OptionsControlKind::Slider { value } => {
                    modal_runtime::draw_slider(
                        presentation,
                        item.paint_bounds,
                        value,
                        item.enabled,
                        item.focused,
                    );
                    modal_runtime::draw_body(presentation, item.label, item.label_bounds, false);
                    if let Some(bounds) = item.value_bounds {
                        let label = if item.enabled {
                            format!("{:.0}%", value * 100.0)
                        } else {
                            "--".into()
                        };
                        modal_runtime::draw_body(presentation, &label, bounds, true);
                    }
                }
            }
        }
        modal_runtime::draw_body(presentation, &view.status, view.status_bounds, false);
    }
}
