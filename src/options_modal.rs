//! Transient Options/pause views. Settings, saves, clocks and sessions remain
//! caller-owned; W1 functional facts and project adaptations are in the report.

use crate::{
    modal::{ModalButton, ModalControls, ModalInput},
    settings::{GameSettings, SettingChange, SettingsCapabilities},
};
use macroquad::prelude::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptionsContext {
    Selector,
    Paused,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptionsPage {
    Settings,
    Pause,
}

/// `context_epoch` must change for each open, profile or scene replacement.
/// Thumb width comes from the same owned texture used by the drawing helper.
#[derive(Clone, Copy, Debug)]
pub struct OptionsFacts {
    pub settings: GameSettings,
    pub capabilities: SettingsCapabilities,
    pub context_epoch: u64,
    pub slider_thumb_width: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum OptionsControl {
    Music = 1,
    Effects,
    Fullscreen,
    CustomCursor,
    HardwareAcceleration,
    Help,
    Stories,
    Screensaver,
    MainMenu,
    Close,
    Resume,
    OpenOptions,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OptionsControlKind {
    Button,
    Slider { value: f32 },
    Checkbox { checked: bool },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OptionsControlView {
    pub control: OptionsControl,
    pub bounds: Rect,
    pub paint_bounds: Rect,
    pub label_bounds: Rect,
    pub value_bounds: Option<Rect>,
    pub label: &'static str,
    pub kind: OptionsControlKind,
    pub enabled: bool,
    pub pressed: bool,
    pub focused: bool,
}

#[derive(Debug, PartialEq)]
pub struct OptionsView {
    pub panel: Rect,
    pub title: &'static str,
    pub title_bounds: Rect,
    pub status: String,
    pub status_bounds: Rect,
    pub controls: Vec<OptionsControlView>,
}

impl OptionsView {
    pub fn control(&self, control: OptionsControl) -> Option<&OptionsControlView> {
        self.controls.iter().find(|item| item.control == control)
    }
}

/// Parent revalidates the epoch and previous settings before applying/saving.
/// An acknowledgement must match the entire request, including its sequence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OptionsSettingRequest {
    pub context_epoch: u64,
    pub sequence: u64,
    pub previous: GameSettings,
    pub change: SettingChange,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OptionsIntent {
    Setting(OptionsSettingRequest),
    Close,
    Resume,
    Help,
    MainMenu,
    Stories,
    Screensaver,
}

struct SliderDrag {
    control: OptionsControl,
    value: f32,
    grab_offset: f32,
    thumb_width: f32,
}

pub struct OptionsModal {
    context: OptionsContext,
    context_epoch: u64,
    page: OptionsPage,
    controls: ModalControls,
    drag: Option<SliderDrag>,
    pending: Option<OptionsSettingRequest>,
    sequence: u64,
    last_settings: Option<GameSettings>,
    error: String,
}

impl OptionsModal {
    pub fn options(context: OptionsContext, context_epoch: u64) -> Self {
        let mut controls = ModalControls::default();
        controls.set_focus(Some(OptionsControl::Close as u64));
        Self {
            context,
            context_epoch,
            page: OptionsPage::Settings,
            controls,
            drag: None,
            pending: None,
            sequence: 0,
            last_settings: None,
            error: String::new(),
        }
    }

    pub fn pause(context_epoch: u64) -> Self {
        let mut modal = Self::options(OptionsContext::Paused, context_epoch);
        modal.show_page(OptionsPage::Pause);
        modal
    }

    pub fn page(&self) -> OptionsPage {
        self.page
    }

    pub fn cancel_capture(&mut self) {
        self.drag = None;
        self.controls.cancel_capture();
    }

    /// Navigation failures can also be reported without losing the modal.
    pub fn set_error(&mut self, message: impl Into<String>) {
        self.error = message.into();
    }

    pub fn finish_setting(
        &mut self,
        request: OptionsSettingRequest,
        result: Result<(), String>,
    ) -> bool {
        if self.pending != Some(request) {
            return false;
        }
        self.pending = None;
        self.error = result.err().unwrap_or_default();
        true
    }

    pub fn input(&mut self, input: &ModalInput, facts: OptionsFacts) -> Option<OptionsIntent> {
        if facts.context_epoch != self.context_epoch {
            self.cancel_capture();
            self.pending = None;
            return None;
        }
        if self
            .last_settings
            .is_some_and(|previous| previous != facts.settings)
        {
            self.cancel_capture();
        }
        self.last_settings = Some(facts.settings);
        if !input.focused || self.pending.is_some() {
            self.cancel_capture();
            return None;
        }
        if input.cancel {
            self.cancel_capture();
            return self.close();
        }
        if input.next || input.previous {
            self.cancel_capture();
        }

        let view = self.view(facts);
        let buttons: Vec<_> = view
            .controls
            .iter()
            .map(|item| ModalButton {
                id: item.control as u64,
                bounds: item.bounds,
                enabled: item.enabled,
            })
            .collect();
        let activated = self.controls.update(input, &buttons);
        if input.next || input.previous {
            return None;
        }

        if let Some(drag) = self.drag.as_ref()
            && (!view.control(drag.control).is_some_and(|item| item.enabled)
                || facts.slider_thumb_width != drag.thumb_width)
        {
            self.cancel_capture();
            return None;
        }
        if let Some(mut drag) = self.drag.take() {
            let bounds = view.control(drag.control)?.bounds;
            if !input.pointer.is_finite() {
                self.cancel_capture();
                return None;
            }
            if input.down || input.released {
                drag.value =
                    slider_value(bounds, drag.thumb_width, input.pointer.x, drag.grab_offset);
            }
            if input.released {
                self.controls.cancel_capture();
                return self.request_volume(drag.control, drag.value, facts);
            }
            if input.down {
                self.drag = Some(drag);
            }
            return None;
        }
        if input.pressed
            && input.pointer.is_finite()
            && let Some(item) = view.controls.iter().find(|item| {
                item.enabled
                    && matches!(item.kind, OptionsControlKind::Slider { .. })
                    && item.bounds.contains(input.pointer)
            })
            && let OptionsControlKind::Slider { value } = item.kind
        {
            let thumb_left = item.bounds.x + value * (item.bounds.w - facts.slider_thumb_width);
            let grab_offset = if (thumb_left..=thumb_left + facts.slider_thumb_width)
                .contains(&input.pointer.x)
            {
                input.pointer.x - thumb_left
            } else {
                facts.slider_thumb_width / 2.0
            };
            self.drag = Some(SliderDrag {
                control: item.control,
                value: slider_value(
                    item.bounds,
                    facts.slider_thumb_width,
                    input.pointer.x,
                    grab_offset,
                ),
                grab_offset,
                thumb_width: facts.slider_thumb_width,
            });
            self.controls.cancel_capture();
            if input.released {
                let drag = self.drag.take()?;
                return self.request_volume(drag.control, drag.value, facts);
            }
            return None;
        }

        if let Some(focus) = self.controls.focus()
            && let Some(item) = view
                .controls
                .iter()
                .find(|item| item.enabled && item.control as u64 == focus)
            && let OptionsControlKind::Slider { value } = item.kind
        {
            let value = if input.home {
                Some(0.0)
            } else if input.end {
                Some(1.0)
            } else if input.left != input.right {
                Some((value + if input.right { 0.05 } else { -0.05 }).clamp(0.0, 1.0))
            } else {
                None
            };
            if let Some(value) = value {
                self.controls.cancel_capture();
                return self.request_volume(item.control, value, facts);
            }
        }
        let control = view
            .controls
            .iter()
            .find(|item| Some(item.control as u64) == activated)?
            .control;
        match control {
            OptionsControl::Fullscreen => {
                self.request(SettingChange::Fullscreen(!facts.settings.fullscreen), facts)
            }
            OptionsControl::CustomCursor => self.request(
                SettingChange::CustomCursor(!facts.settings.custom_cursor),
                facts,
            ),
            OptionsControl::Close => self.close(),
            OptionsControl::Resume => Some(OptionsIntent::Resume),
            OptionsControl::OpenOptions => {
                self.show_page(OptionsPage::Settings);
                None
            }
            OptionsControl::Help => Some(OptionsIntent::Help),
            OptionsControl::MainMenu => Some(OptionsIntent::MainMenu),
            OptionsControl::Stories => Some(OptionsIntent::Stories),
            OptionsControl::Screensaver => Some(OptionsIntent::Screensaver),
            _ => None,
        }
    }

    fn show_page(&mut self, page: OptionsPage) {
        self.cancel_capture();
        self.page = page;
        self.error.clear();
        self.controls.set_focus(Some(if page == OptionsPage::Pause {
            OptionsControl::Resume as u64
        } else {
            OptionsControl::Close as u64
        }));
    }

    fn close(&mut self) -> Option<OptionsIntent> {
        if self.page == OptionsPage::Pause {
            Some(OptionsIntent::Resume)
        } else if self.context == OptionsContext::Paused {
            self.show_page(OptionsPage::Pause);
            None
        } else {
            Some(OptionsIntent::Close)
        }
    }

    fn request_volume(
        &mut self,
        control: OptionsControl,
        value: f32,
        facts: OptionsFacts,
    ) -> Option<OptionsIntent> {
        let change = match control {
            OptionsControl::Music if value != facts.settings.music_volume => {
                SettingChange::MusicVolume(value)
            }
            OptionsControl::Effects if value != facts.settings.effects_volume => {
                SettingChange::EffectsVolume(value)
            }
            _ => return None,
        };
        self.request(change, facts)
    }

    fn request(&mut self, change: SettingChange, facts: OptionsFacts) -> Option<OptionsIntent> {
        self.sequence = self.sequence.checked_add(1)?;
        let request = OptionsSettingRequest {
            context_epoch: self.context_epoch,
            sequence: self.sequence,
            previous: facts.settings,
            change,
        };
        self.error.clear();
        self.pending = Some(request);
        self.cancel_capture();
        Some(OptionsIntent::Setting(request))
    }

    pub fn view(&self, facts: OptionsFacts) -> OptionsView {
        let active = self.pending.is_none() && facts.context_epoch == self.context_epoch;
        let mut view = if self.page == OptionsPage::Pause {
            OptionsView {
                panel: Rect::new(128.0, 92.0, 384.0, 306.0),
                title: "Paused",
                title_bounds: Rect::new(156.0, 108.0, 328.0, 36.0),
                status_bounds: Rect::new(156.0, 326.0, 328.0, 48.0),
                status: String::new(),
                controls: vec![
                    button(
                        OptionsControl::Resume,
                        "Resume",
                        Rect::new(156.0, 162.0, 328.0, 36.0),
                    ),
                    button(
                        OptionsControl::OpenOptions,
                        "Options",
                        Rect::new(156.0, 212.0, 158.0, 36.0),
                    ),
                    button(
                        OptionsControl::Help,
                        "Help",
                        Rect::new(326.0, 212.0, 158.0, 36.0),
                    ),
                    button(
                        OptionsControl::MainMenu,
                        "Main Menu",
                        Rect::new(156.0, 262.0, 328.0, 36.0),
                    ),
                ],
            }
        } else {
            let valid_settings = facts.settings.validate().is_ok();
            let slider_geometry = facts.slider_thumb_width.is_finite()
                && (0.0..224.0).contains(&facts.slider_thumb_width)
                && facts.slider_thumb_width > 0.0;
            let mut controls = vec![
                slider(
                    OptionsControl::Music,
                    "Music",
                    80.0,
                    facts.settings.music_volume,
                    valid_settings && slider_geometry && facts.capabilities.music_volume,
                ),
                slider(
                    OptionsControl::Effects,
                    "Effects",
                    120.0,
                    facts.settings.effects_volume,
                    valid_settings && slider_geometry && facts.capabilities.effects_volume,
                ),
                checkbox(
                    OptionsControl::Fullscreen,
                    "Fullscreen",
                    164.0,
                    facts.settings.fullscreen,
                    valid_settings && facts.capabilities.fullscreen,
                ),
                checkbox(
                    OptionsControl::CustomCursor,
                    "Custom cursor",
                    206.0,
                    facts.settings.custom_cursor,
                    valid_settings && facts.capabilities.custom_cursor,
                ),
                checkbox(
                    OptionsControl::HardwareAcceleration,
                    "Hardware acceleration (unavailable)",
                    248.0,
                    false,
                    false,
                ),
            ];
            if self.context == OptionsContext::Selector {
                controls.extend([
                    button(
                        OptionsControl::Help,
                        "Help",
                        Rect::new(128.0, 340.0, 120.0, 34.0),
                    ),
                    button(
                        OptionsControl::Stories,
                        "Stories",
                        Rect::new(260.0, 340.0, 120.0, 34.0),
                    ),
                    button(
                        OptionsControl::Screensaver,
                        "Saver",
                        Rect::new(392.0, 340.0, 120.0, 34.0),
                    ),
                ]);
            } else {
                controls.extend([
                    button(
                        OptionsControl::Help,
                        "Help",
                        Rect::new(128.0, 340.0, 186.0, 34.0),
                    ),
                    button(
                        OptionsControl::MainMenu,
                        "Main Menu",
                        Rect::new(326.0, 340.0, 186.0, 34.0),
                    ),
                ]);
            }
            controls.push(button(
                OptionsControl::Close,
                if self.context == OptionsContext::Paused {
                    "Back"
                } else {
                    "OK"
                },
                Rect::new(128.0, 394.0, 384.0, 36.0),
            ));
            OptionsView {
                panel: Rect::new(100.0, 22.0, 440.0, 436.0),
                title: "Options",
                title_bounds: Rect::new(128.0, 34.0, 384.0, 34.0),
                status_bounds: Rect::new(128.0, 290.0, 384.0, 40.0),
                status: String::new(),
                controls,
            }
        };
        view.status = if !self.error.is_empty() {
            self.error.clone()
        } else if self.pending.is_some() {
            "Applying change...".into()
        } else if self.page == OptionsPage::Pause {
            "Your game stays paused until you resume.".into()
        } else {
            "Drag volume or use Left/Right. Unavailable controls are disabled.".into()
        };
        for item in &mut view.controls {
            item.enabled &= active;
            item.focused = item.enabled && self.controls.is_focused(item.control as u64);
            item.pressed = item.enabled && self.controls.is_pressed(item.control as u64);
            if let Some(drag) = self
                .drag
                .as_ref()
                .filter(|drag| drag.control == item.control && item.enabled)
            {
                item.kind = OptionsControlKind::Slider { value: drag.value };
                item.pressed = true;
            }
        }
        view
    }
}

fn button(control: OptionsControl, label: &'static str, bounds: Rect) -> OptionsControlView {
    OptionsControlView {
        control,
        bounds,
        paint_bounds: bounds,
        label_bounds: bounds,
        value_bounds: None,
        label,
        kind: OptionsControlKind::Button,
        enabled: true,
        pressed: false,
        focused: false,
    }
}

fn slider(
    control: OptionsControl,
    label: &'static str,
    y: f32,
    value: f32,
    enabled: bool,
) -> OptionsControlView {
    OptionsControlView {
        kind: OptionsControlKind::Slider {
            value: if value.is_finite() {
                value.clamp(0.0, 1.0)
            } else {
                0.0
            },
        },
        label_bounds: Rect::new(128.0, y + 4.0, 126.0, 24.0),
        value_bounds: Some(Rect::new(496.0, y + 4.0, 36.0, 24.0)),
        enabled,
        ..button(control, label, Rect::new(266.0, y, 224.0, 32.0))
    }
}

fn checkbox(
    control: OptionsControl,
    label: &'static str,
    y: f32,
    checked: bool,
    enabled: bool,
) -> OptionsControlView {
    OptionsControlView {
        kind: OptionsControlKind::Checkbox { checked },
        paint_bounds: Rect::new(128.0, y, 32.0, 32.0),
        label_bounds: Rect::new(170.0, y + 4.0, 342.0, 28.0),
        enabled,
        ..button(control, label, Rect::new(128.0, y, 384.0, 36.0))
    }
}

fn slider_value(bounds: Rect, thumb_width: f32, pointer_x: f32, grab_offset: f32) -> f32 {
    ((pointer_x - bounds.x - grab_offset) / (bounds.w - thumb_width)).clamp(0.0, 1.0)
}
