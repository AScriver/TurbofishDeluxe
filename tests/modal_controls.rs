use macroquad::prelude::{Rect, Vec2};
use turbofish_deluxe::modal::{ModalButton, ModalControls, ModalInput};

fn buttons() -> [ModalButton; 3] {
    [
        ModalButton {
            id: 1,
            bounds: Rect::new(0.0, 0.0, 50.0, 30.0),
            enabled: true,
        },
        ModalButton {
            id: 2,
            bounds: Rect::new(60.0, 0.0, 50.0, 30.0),
            enabled: false,
        },
        ModalButton {
            id: 3,
            bounds: Rect::new(120.0, 0.0, 50.0, 30.0),
            enabled: true,
        },
    ]
}
fn input() -> ModalInput {
    ModalInput {
        pointer: Vec2::new(20.0, 10.0),
        focused: true,
        ..Default::default()
    }
}

#[test]
fn activation_requires_captured_release_and_emits_once() {
    let mut controls = ModalControls::default();
    assert_eq!(
        controls.update(
            &ModalInput {
                released: true,
                ..input()
            },
            &buttons()
        ),
        None
    );
    assert_eq!(
        controls.update(
            &ModalInput {
                pressed: true,
                down: true,
                ..input()
            },
            &buttons()
        ),
        None
    );
    assert!(controls.is_pressed(1));
    assert_eq!(
        controls.update(
            &ModalInput {
                released: true,
                ..input()
            },
            &buttons()
        ),
        Some(1)
    );
    assert_eq!(
        controls.update(
            &ModalInput {
                released: true,
                ..input()
            },
            &buttons()
        ),
        None
    );
}

#[test]
fn release_over_another_button_does_not_activate() {
    let mut controls = ModalControls::default();
    controls.update(
        &ModalInput {
            pressed: true,
            down: true,
            ..input()
        },
        &buttons(),
    );
    assert_eq!(
        controls.update(
            &ModalInput {
                pointer: Vec2::new(140.0, 10.0),
                released: true,
                ..input()
            },
            &buttons()
        ),
        None
    );
}

#[test]
fn disabled_and_removed_captures_cannot_activate() {
    for remove in [false, true] {
        let mut controls = ModalControls::default();
        controls.update(
            &ModalInput {
                pressed: true,
                down: true,
                ..input()
            },
            &buttons(),
        );
        let mut current = buttons().to_vec();
        if remove {
            current.remove(0);
        } else {
            current[0].enabled = false;
        }
        assert_eq!(
            controls.update(
                &ModalInput {
                    released: true,
                    ..input()
                },
                &current
            ),
            None
        );
    }
}

#[test]
fn focus_loss_and_cancel_retire_capture() {
    for event in [
        ModalInput {
            focused: false,
            ..input()
        },
        ModalInput {
            cancel: true,
            ..input()
        },
    ] {
        let mut controls = ModalControls::default();
        controls.update(
            &ModalInput {
                pressed: true,
                down: true,
                ..input()
            },
            &buttons(),
        );
        assert_eq!(controls.update(&event, &buttons()), None);
        assert_eq!(
            controls.update(
                &ModalInput {
                    released: true,
                    ..input()
                },
                &buttons()
            ),
            None
        );
    }
}

#[test]
fn keyboard_focus_skips_disabled_controls_and_wraps() {
    let mut controls = ModalControls::default();
    controls.update(&input(), &buttons());
    assert_eq!(controls.focus(), Some(1));
    controls.update(
        &ModalInput {
            next: true,
            accept: true,
            ..input()
        },
        &buttons(),
    );
    assert_eq!(controls.focus(), Some(3));
    assert_eq!(
        controls.update(
            &ModalInput {
                accept: true,
                ..input()
            },
            &buttons()
        ),
        Some(3)
    );
    controls.update(
        &ModalInput {
            next: true,
            ..input()
        },
        &buttons(),
    );
    assert_eq!(controls.focus(), Some(1));
    controls.update(
        &ModalInput {
            previous: true,
            ..input()
        },
        &buttons(),
    );
    assert_eq!(controls.focus(), Some(3));
}

#[test]
fn empty_or_all_disabled_controls_have_no_keyboard_action() {
    let mut controls = ModalControls::default();
    controls.set_focus(Some(999));
    assert_eq!(
        controls.update(
            &ModalInput {
                accept: true,
                next: true,
                ..input()
            },
            &[]
        ),
        None
    );
    let mut unavailable = buttons();
    unavailable
        .iter_mut()
        .for_each(|button| button.enabled = false);
    assert_eq!(
        controls.update(
            &ModalInput {
                accept: true,
                ..input()
            },
            &unavailable
        ),
        None
    );
}
