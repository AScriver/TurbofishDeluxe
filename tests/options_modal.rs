//! Offline contracts from modal-batch-303-307 and qualified W1 slider/options
//! facts in docs/modals-options.md. No native input, game files or live saves.

use macroquad::prelude::{Rect, Vec2, vec2};
use turbofish_deluxe::{
    modal::ModalInput,
    options_modal::{
        OptionsContext, OptionsControl, OptionsControlKind, OptionsFacts, OptionsIntent,
        OptionsModal, OptionsPage, OptionsSettingRequest,
    },
    settings::{GameSettings, SettingChange, SettingsBackend, SettingsCapabilities},
};

fn facts() -> OptionsFacts {
    OptionsFacts {
        settings: GameSettings {
            music_volume: 0.5,
            effects_volume: 0.5,
            ..GameSettings::default()
        },
        capabilities: SettingsCapabilities {
            music_volume: true,
            effects_volume: true,
            fullscreen: true,
            custom_cursor: true,
        },
        context_epoch: 303,
        slider_thumb_width: 24.0,
    }
}

fn input() -> ModalInput {
    ModalInput {
        focused: true,
        pointer: vec2(-100.0, -100.0),
        ..ModalInput::default()
    }
}

fn bounds(modal: &OptionsModal, facts: OptionsFacts, control: OptionsControl) -> Rect {
    modal
        .view(facts)
        .control(control)
        .expect("control in current page")
        .bounds
}

fn center(bounds: Rect) -> Vec2 {
    vec2(bounds.x + bounds.w / 2.0, bounds.y + bounds.h / 2.0)
}

fn click(
    modal: &mut OptionsModal,
    facts: OptionsFacts,
    control: OptionsControl,
) -> Option<OptionsIntent> {
    let pointer = center(bounds(modal, facts, control));
    assert_eq!(
        modal.input(
            &ModalInput {
                pointer,
                pressed: true,
                down: true,
                ..input()
            },
            facts
        ),
        None
    );
    modal.input(
        &ModalInput {
            pointer,
            released: true,
            ..input()
        },
        facts,
    )
}

fn request(intent: Option<OptionsIntent>) -> OptionsSettingRequest {
    let Some(OptionsIntent::Setting(request)) = intent else {
        panic!("expected setting request: {intent:?}")
    };
    request
}

fn volume(modal: &OptionsModal, facts: OptionsFacts, control: OptionsControl) -> f32 {
    let OptionsControlKind::Slider { value } = modal.view(facts).control(control).unwrap().kind
    else {
        panic!("slider")
    };
    value
}

#[test]
fn toggle_requires_same_enabled_captured_release() {
    let mut modal = OptionsModal::options(OptionsContext::Selector, 303);
    let facts = facts();
    let pointer = center(bounds(&modal, facts, OptionsControl::Fullscreen));
    assert_eq!(
        modal.input(
            &ModalInput {
                pointer,
                released: true,
                ..input()
            },
            facts
        ),
        None
    );
    assert_eq!(
        modal.input(
            &ModalInput {
                pointer,
                pressed: true,
                down: true,
                ..input()
            },
            facts
        ),
        None
    );
    assert_eq!(
        modal.input(
            &ModalInput {
                released: true,
                ..input()
            },
            facts
        ),
        None
    );
    let issued = request(click(&mut modal, facts, OptionsControl::Fullscreen));
    assert_eq!(issued.change, SettingChange::Fullscreen(true));
    assert_eq!(issued.previous, facts.settings);
    assert_eq!(issued.context_epoch, 303);
    assert!(!facts.settings.fullscreen);
}

#[test]
fn captured_drag_previews_then_submits_once_outside_rail() {
    let mut modal = OptionsModal::options(OptionsContext::Selector, 303);
    let facts = facts();
    let rail = bounds(&modal, facts, OptionsControl::Music);
    let start = center(rail);
    assert_eq!(
        modal.input(
            &ModalInput {
                pointer: start,
                pressed: true,
                down: true,
                ..input()
            },
            facts
        ),
        None
    );
    assert_eq!(volume(&modal, facts, OptionsControl::Music), 0.5);
    assert_eq!(
        modal.input(
            &ModalInput {
                pointer: vec2(900.0, start.y),
                down: true,
                ..input()
            },
            facts
        ),
        None
    );
    assert_eq!(volume(&modal, facts, OptionsControl::Music), 1.0);
    assert_eq!(facts.settings.music_volume, 0.5);
    let issued = request(modal.input(
        &ModalInput {
            pointer: vec2(900.0, 450.0),
            released: true,
            ..input()
        },
        facts,
    ));
    assert_eq!(issued.change, SettingChange::MusicVolume(1.0));
    assert_eq!(volume(&modal, facts, OptionsControl::Music), 0.5);
    assert_eq!(
        modal.input(
            &ModalInput {
                pointer: vec2(900.0, 450.0),
                released: true,
                ..input()
            },
            facts
        ),
        None
    );
}

#[test]
fn slider_thumb_grab_matches_owned_thumb_geometry_without_jumping() {
    let mut modal = OptionsModal::options(OptionsContext::Selector, 303);
    let facts = facts();
    let rail = bounds(&modal, facts, OptionsControl::Effects);
    let travel = rail.w - facts.slider_thumb_width;
    let pointer = vec2(rail.x + travel * 0.5 + 4.0, rail.y + 12.0);
    modal.input(
        &ModalInput {
            pointer,
            pressed: true,
            down: true,
            ..input()
        },
        facts,
    );
    assert_eq!(volume(&modal, facts, OptionsControl::Effects), 0.5);
    modal.input(
        &ModalInput {
            pointer: pointer + vec2(travel / 4.0, 0.0),
            down: true,
            ..input()
        },
        facts,
    );
    assert_eq!(volume(&modal, facts, OptionsControl::Effects), 0.75);
    let issued = request(modal.input(
        &ModalInput {
            pointer: pointer + vec2(travel / 4.0, 0.0),
            released: true,
            ..input()
        },
        facts,
    ));
    assert_eq!(issued.change, SettingChange::EffectsVolume(0.75));
}

#[test]
fn slider_click_with_coalesced_press_and_release_commits_clamped_value() {
    let mut modal = OptionsModal::options(OptionsContext::Selector, 303);
    let facts = facts();
    let rail = bounds(&modal, facts, OptionsControl::Music);
    let issued = request(modal.input(
        &ModalInput {
            pointer: vec2(rail.x + 1.0, rail.y + 10.0),
            pressed: true,
            released: true,
            ..input()
        },
        facts,
    ));
    assert_eq!(issued.change, SettingChange::MusicVolume(0.0));
}

#[test]
fn focus_traversal_cannot_arm_a_slider_from_same_frame_pointer_events() {
    let mut modal = OptionsModal::options(OptionsContext::Selector, 303);
    let facts = facts();
    let rail = bounds(&modal, facts, OptionsControl::Music);
    let pointer = vec2(rail.x + 1.0, rail.y + 12.0);
    assert_eq!(
        modal.input(
            &ModalInput {
                pointer,
                pressed: true,
                down: true,
                next: true,
                ..input()
            },
            facts
        ),
        None
    );
    assert_eq!(
        volume(&modal, facts, OptionsControl::Music),
        facts.settings.music_volume
    );
    assert!(
        modal
            .view(facts)
            .control(OptionsControl::Music)
            .unwrap()
            .focused
    );
    assert_eq!(
        modal.input(
            &ModalInput {
                pointer,
                released: true,
                ..input()
            },
            facts
        ),
        None
    );
}

#[test]
fn unsupported_settings_and_capability_loss_cannot_activate() {
    let mut modal = OptionsModal::options(OptionsContext::Selector, 303);
    let mut facts = facts();
    facts.capabilities.music_volume = false;
    facts.capabilities.fullscreen = false;
    for control in [
        OptionsControl::Music,
        OptionsControl::Fullscreen,
        OptionsControl::HardwareAcceleration,
    ] {
        assert!(!modal.view(facts).control(control).unwrap().enabled);
        assert_eq!(click(&mut modal, facts, control), None);
    }
    let pointer = center(bounds(&modal, facts, OptionsControl::CustomCursor));
    modal.input(
        &ModalInput {
            pointer,
            pressed: true,
            down: true,
            ..input()
        },
        facts,
    );
    facts.capabilities.custom_cursor = false;
    assert_eq!(
        modal.input(
            &ModalInput {
                pointer,
                released: true,
                ..input()
            },
            facts
        ),
        None
    );
    facts.capabilities.music_volume = true;
    let pointer = center(bounds(&modal, facts, OptionsControl::Music));
    modal.input(
        &ModalInput {
            pointer,
            pressed: true,
            down: true,
            ..input()
        },
        facts,
    );
    facts.capabilities.music_volume = false;
    assert_eq!(
        modal.input(
            &ModalInput {
                pointer: vec2(900.0, pointer.y),
                released: true,
                ..input()
            },
            facts
        ),
        None
    );
}

#[test]
fn tab_focus_skips_disabled_controls_and_keyboard_adjustment_stays_bounded() {
    let mut modal = OptionsModal::options(OptionsContext::Selector, 303);
    let mut facts = facts();
    facts.capabilities.music_volume = false;
    modal.input(
        &ModalInput {
            next: true,
            ..input()
        },
        facts,
    );
    assert!(
        modal
            .view(facts)
            .control(OptionsControl::Effects)
            .unwrap()
            .focused
    );
    let issued = request(modal.input(
        &ModalInput {
            right: true,
            ..input()
        },
        facts,
    ));
    assert_eq!(issued.change, SettingChange::EffectsVolume(0.55));
    assert!(modal.finish_setting(issued, Ok(())));
    facts.settings.effects_volume = 0.55;
    let issued = request(modal.input(
        &ModalInput {
            end: true,
            ..input()
        },
        facts,
    ));
    assert_eq!(issued.change, SettingChange::EffectsVolume(1.0));
    modal.finish_setting(issued, Ok(()));
    facts.settings.effects_volume = 1.0;
    assert_eq!(
        modal.input(
            &ModalInput {
                right: true,
                ..input()
            },
            facts
        ),
        None
    );
    let issued = request(modal.input(
        &ModalInput {
            home: true,
            ..input()
        },
        facts,
    ));
    assert_eq!(issued.change, SettingChange::EffectsVolume(0.0));
    modal.finish_setting(issued, Ok(()));
    facts.settings.effects_volume = 0.0;
    assert_eq!(
        modal.input(
            &ModalInput {
                left: true,
                ..input()
            },
            facts
        ),
        None
    );
    modal.input(
        &ModalInput {
            next: true,
            ..input()
        },
        facts,
    );
    assert!(
        modal
            .view(facts)
            .control(OptionsControl::Fullscreen)
            .unwrap()
            .focused
    );
    let issued = request(modal.input(
        &ModalInput {
            accept: true,
            ..input()
        },
        facts,
    ));
    assert_eq!(issued.change, SettingChange::Fullscreen(true));
}

#[test]
fn focus_loss_cancel_and_missed_release_retire_slider_preview() {
    for retirement in [
        ModalInput {
            focused: false,
            released: true,
            ..input()
        },
        ModalInput {
            cancel: true,
            ..input()
        },
        input(),
    ] {
        let mut modal = OptionsModal::options(OptionsContext::Paused, 303);
        let facts = facts();
        let pointer = center(bounds(&modal, facts, OptionsControl::Music));
        modal.input(
            &ModalInput {
                pointer,
                pressed: true,
                down: true,
                ..input()
            },
            facts,
        );
        modal.input(
            &ModalInput {
                pointer: vec2(900.0, pointer.y),
                down: true,
                ..input()
            },
            facts,
        );
        assert_eq!(modal.input(&retirement, facts), None);
        assert_eq!(
            modal.input(
                &ModalInput {
                    pointer: vec2(900.0, pointer.y),
                    released: true,
                    ..input()
                },
                facts
            ),
            None
        );
        if modal.page() == OptionsPage::Settings {
            assert_eq!(volume(&modal, facts, OptionsControl::Music), 0.5);
        }
    }
}

#[test]
fn context_or_committed_setting_replacement_cancels_old_capture() {
    for replace_context in [true, false] {
        let mut modal = OptionsModal::options(OptionsContext::Selector, 303);
        let mut facts = facts();
        let pointer = center(bounds(&modal, facts, OptionsControl::Fullscreen));
        modal.input(
            &ModalInput {
                pointer,
                pressed: true,
                down: true,
                ..input()
            },
            facts,
        );
        if replace_context {
            facts.context_epoch += 1;
        } else {
            facts.settings.custom_cursor = true;
        }
        assert_eq!(
            modal.input(
                &ModalInput {
                    pointer,
                    released: true,
                    ..input()
                },
                facts
            ),
            None
        );
    }
}

#[derive(Default)]
struct Backend {
    fail: bool,
    fullscreen: bool,
    calls: usize,
}

impl SettingsBackend for Backend {
    fn capabilities(&self) -> SettingsCapabilities {
        facts().capabilities
    }
    fn set_music_volume(&mut self, _: f32) -> Result<(), String> {
        unreachable!()
    }
    fn set_effects_volume(&mut self, _: f32) -> Result<(), String> {
        unreachable!()
    }
    fn set_custom_cursor(&mut self, _: bool) -> Result<(), String> {
        unreachable!()
    }
    fn set_fullscreen(&mut self, value: bool) -> Result<(), String> {
        self.calls += 1;
        if self.fail {
            Err("display rejected fullscreen".into())
        } else {
            self.fullscreen = value;
            Ok(())
        }
    }
}

#[test]
fn actual_transaction_failure_and_identified_feedback_keep_modal_open() {
    let mut modal = OptionsModal::options(OptionsContext::Selector, 303);
    let mut facts = facts();
    let mut backend = Backend {
        fail: true,
        ..Backend::default()
    };
    let issued = request(click(&mut modal, facts, OptionsControl::Fullscreen));
    assert!(modal.view(facts).controls.iter().all(|item| !item.enabled));
    assert_eq!(
        modal.input(
            &ModalInput {
                accept: true,
                cancel: true,
                ..input()
            },
            facts
        ),
        None
    );
    let result = facts.settings.change(&mut backend, issued.change);
    assert!(result.is_err());
    assert!(!facts.settings.fullscreen);
    let mut stale = issued;
    stale.sequence += 1;
    assert!(!modal.finish_setting(stale, Ok(())));
    assert!(modal.finish_setting(issued, result));
    assert_eq!(modal.page(), OptionsPage::Settings);
    assert_eq!(modal.view(facts).status, "display rejected fullscreen");
    assert!(!modal.finish_setting(issued, Ok(())));
    backend.fail = false;
    let retry = request(click(&mut modal, facts, OptionsControl::Fullscreen));
    assert!(retry.sequence > issued.sequence);
    let result = facts.settings.change(&mut backend, retry.change);
    assert!(modal.finish_setting(retry, result));
    assert!(facts.settings.fullscreen && backend.fullscreen);
    assert_eq!(backend.calls, 2);
    assert!(matches!(
        modal
            .view(facts)
            .control(OptionsControl::Fullscreen)
            .unwrap()
            .kind,
        OptionsControlKind::Checkbox { checked: true }
    ));
}

#[test]
fn stale_pending_response_cannot_change_replaced_context() {
    let mut modal = OptionsModal::options(OptionsContext::Selector, 303);
    let mut facts = facts();
    let issued = request(click(&mut modal, facts, OptionsControl::Fullscreen));
    facts.context_epoch += 1;
    modal.input(&input(), facts);
    assert!(!modal.finish_setting(issued, Ok(())));
    assert!(modal.view(facts).controls.iter().all(|item| !item.enabled));
}

#[test]
fn pause_navigation_restores_pause_before_resume_and_emits_explicit_destinations() {
    let mut modal = OptionsModal::pause(303);
    let facts = facts();
    assert_eq!(modal.page(), OptionsPage::Pause);
    assert!(
        modal
            .view(facts)
            .control(OptionsControl::Resume)
            .unwrap()
            .focused
    );
    assert_eq!(click(&mut modal, facts, OptionsControl::OpenOptions), None);
    assert_eq!(modal.page(), OptionsPage::Settings);
    assert_eq!(click(&mut modal, facts, OptionsControl::Close), None);
    assert_eq!(modal.page(), OptionsPage::Pause);
    assert_eq!(
        click(&mut modal, facts, OptionsControl::Help),
        Some(OptionsIntent::Help)
    );
    assert_eq!(
        click(&mut modal, facts, OptionsControl::MainMenu),
        Some(OptionsIntent::MainMenu)
    );
    assert_eq!(
        click(&mut modal, facts, OptionsControl::Resume),
        Some(OptionsIntent::Resume)
    );
    assert_eq!(
        modal.input(
            &ModalInput {
                cancel: true,
                ..input()
            },
            facts
        ),
        Some(OptionsIntent::Resume)
    );
}

#[test]
fn selector_options_keep_help_stories_and_existing_saver_routes() {
    let mut modal = OptionsModal::options(OptionsContext::Selector, 303);
    let facts = facts();
    for (control, intent) in [
        (OptionsControl::Help, OptionsIntent::Help),
        (OptionsControl::Stories, OptionsIntent::Stories),
        (OptionsControl::Screensaver, OptionsIntent::Screensaver),
        (OptionsControl::Close, OptionsIntent::Close),
    ] {
        assert_eq!(click(&mut modal, facts, control), Some(intent));
    }
    assert!(
        modal
            .view(facts)
            .control(OptionsControl::MainMenu)
            .is_none()
    );
    assert_eq!(
        modal.input(
            &ModalInput {
                cancel: true,
                ..input()
            },
            facts
        ),
        Some(OptionsIntent::Close)
    );
}

#[test]
fn owned_layout_separates_controls_status_and_text_within_canvas() {
    let facts = facts();
    for modal in [
        OptionsModal::options(OptionsContext::Selector, 303),
        OptionsModal::options(OptionsContext::Paused, 303),
        OptionsModal::pause(303),
    ] {
        let view = modal.view(facts);
        let contains = |outer: Rect, inner: Rect| {
            inner.w > 0.0
                && inner.h > 0.0
                && inner.x >= outer.x
                && inner.y >= outer.y
                && inner.x + inner.w <= outer.x + outer.w
                && inner.y + inner.h <= outer.y + outer.h
        };
        assert!(contains(Rect::new(0.0, 0.0, 640.0, 480.0), view.panel));
        assert!(contains(view.panel, view.title_bounds));
        assert!(contains(view.panel, view.status_bounds));
        for (index, control) in view.controls.iter().enumerate() {
            assert!(contains(view.panel, control.bounds));
            assert!(contains(view.panel, control.paint_bounds));
            assert!(contains(view.panel, control.label_bounds));
            if let Some(value_bounds) = control.value_bounds {
                assert!(contains(view.panel, value_bounds));
            }
            assert!(!control.bounds.overlaps(&view.status_bounds));
            assert!(!control.bounds.overlaps(&view.title_bounds));
            for other in &view.controls[index + 1..] {
                assert!(!control.bounds.overlaps(&other.bounds));
            }
        }
    }
}

#[test]
fn malformed_settings_or_missing_thumb_disable_setting_requests() {
    let mut modal = OptionsModal::options(OptionsContext::Selector, 303);
    let mut facts = facts();
    facts.slider_thumb_width = 0.0;
    assert_eq!(click(&mut modal, facts, OptionsControl::Music), None);
    facts.settings.music_volume = f32::NAN;
    for control in [
        OptionsControl::Music,
        OptionsControl::Effects,
        OptionsControl::Fullscreen,
        OptionsControl::CustomCursor,
    ] {
        assert!(!modal.view(facts).control(control).unwrap().enabled);
        assert_eq!(click(&mut modal, facts, control), None);
    }
}
