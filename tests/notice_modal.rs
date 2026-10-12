//! Expectations come from the modal preservation/control contract, not retail parity.

use macroquad::prelude::{Rect, Vec2};
use turbofish_deluxe::{
    game_selector::{LockReason, Mode},
    modal::ModalInput,
    notice_modal::{NoticeIdentity, NoticeIntent, NoticeKind, NoticeModal, NoticeRequest},
};

fn neutral() -> ModalInput {
    ModalInput {
        focused: true,
        ..ModalInput::default()
    }
}

fn arm(modal: &mut NoticeModal) {
    assert_eq!(modal.input(&neutral()), None);
}

fn center(bounds: Rect) -> Vec2 {
    Vec2::new(bounds.x + bounds.w / 2.0, bounds.y + bounds.h / 2.0)
}

fn pointer(point: Vec2, pressed: bool, released: bool, down: bool) -> ModalInput {
    ModalInput {
        pointer: point,
        pressed,
        released,
        down,
        ..neutral()
    }
}

fn confirm(modal: &mut NoticeModal) -> NoticeRequest {
    let point = center(modal.view().unwrap().primary.control.bounds);
    assert_eq!(modal.input(&pointer(point, true, false, true)), None);
    let Some(NoticeIntent::Quit(request)) = modal.input(&pointer(point, false, true, false)) else {
        panic!("captured release must emit one Quit request");
    };
    request
}

#[test]
fn quit_activates_only_on_captured_release_and_waits_for_save_feedback() {
    let mut modal = NoticeModal::quit(10, "Caller save guidance");
    arm(&mut modal);
    let request = confirm(&mut modal);
    assert_eq!(request.identity, modal.identity());
    assert_eq!(request.attempt, 1);
    assert_eq!(modal.pending_request(), Some(request));
    let view = modal.view().unwrap();
    assert!(view.pending);
    assert!(!view.primary.control.enabled);
    assert!(!view.cancel.unwrap().control.enabled);
    assert_eq!(
        modal.input(&ModalInput {
            accept: true,
            ..neutral()
        }),
        None
    );
    assert_eq!(
        modal.input(&ModalInput {
            cancel: true,
            ..neutral()
        }),
        None
    );
    assert!(modal.is_open());
    assert!(modal.quit_finished(request, Ok(())));
    assert!(!modal.is_open());
    assert!(modal.view().is_none());
    assert!(!modal.quit_finished(request, Err("stale failure".into())));
    assert_eq!(
        modal.input(&ModalInput {
            accept: true,
            ..neutral()
        }),
        None
    );
}

#[test]
fn drag_to_another_control_cannot_activate_either_control() {
    let mut modal = NoticeModal::quit(11, "");
    arm(&mut modal);
    let view = modal.view().unwrap();
    let quit = center(view.primary.control.bounds);
    let cancel = center(view.cancel.unwrap().control.bounds);
    assert_eq!(modal.input(&pointer(quit, true, false, true)), None);
    assert_eq!(modal.input(&pointer(cancel, false, true, false)), None);
    assert!(modal.is_open());
    assert!(modal.pending_request().is_none());
    assert_eq!(modal.input(&pointer(cancel, true, false, true)), None);
    assert_eq!(
        modal.input(&pointer(cancel, false, true, false)),
        Some(NoticeIntent::Cancel(modal.identity()))
    );
    assert_eq!(modal.input(&pointer(cancel, false, true, false)), None);
}

#[test]
fn opening_gesture_and_stray_release_cannot_confirm() {
    let mut modal = NoticeModal::quit(12, "");
    let point = center(modal.view().unwrap().primary.control.bounds);
    assert_eq!(modal.input(&pointer(point, true, false, true)), None);
    assert_eq!(modal.input(&pointer(point, false, false, true)), None);
    assert_eq!(modal.input(&pointer(point, false, true, false)), None);
    arm(&mut modal);
    assert_eq!(modal.input(&pointer(point, false, true, false)), None);
    assert!(modal.pending_request().is_none());
    confirm(&mut modal);
}

#[test]
fn escape_wins_over_accept_and_clears_capture() {
    let mut modal = NoticeModal::quit(13, "");
    arm(&mut modal);
    let point = center(modal.view().unwrap().primary.control.bounds);
    assert_eq!(modal.input(&pointer(point, true, false, true)), None);
    assert_eq!(
        modal.input(&ModalInput {
            cancel: true,
            accept: true,
            ..pointer(point, false, true, false)
        }),
        Some(NoticeIntent::Cancel(modal.identity()))
    );
    assert_eq!(modal.input(&pointer(point, false, true, false)), None);
    assert!(modal.pending_request().is_none());
}

#[test]
fn focus_loss_and_covering_modals_require_neutral_input_before_rearming() {
    let mut modal = NoticeModal::quit(14, "");
    arm(&mut modal);
    let point = center(modal.view().unwrap().primary.control.bounds);
    assert_eq!(modal.input(&pointer(point, true, false, true)), None);
    assert_eq!(modal.input(&ModalInput::default()), None);
    assert_eq!(modal.input(&pointer(point, false, true, false)), None);
    assert_eq!(
        modal.input(&ModalInput {
            accept: true,
            ..neutral()
        }),
        None
    );
    arm(&mut modal);
    assert_eq!(modal.input(&pointer(point, true, false, true)), None);
    modal.cancel_capture();
    assert_eq!(modal.input(&pointer(point, false, true, false)), None);
    arm(&mut modal);
    confirm(&mut modal);
}

#[test]
fn quit_defaults_to_cancel_and_keyboard_navigation_can_confirm() {
    let mut cancel = NoticeModal::quit(15, "");
    assert!(cancel.view().unwrap().cancel.unwrap().focused);
    arm(&mut cancel);
    assert_eq!(
        cancel.input(&ModalInput {
            accept: true,
            ..neutral()
        }),
        Some(NoticeIntent::Cancel(cancel.identity()))
    );

    let mut quit = NoticeModal::quit(16, "");
    arm(&mut quit);
    assert_eq!(
        quit.input(&ModalInput {
            next: true,
            ..neutral()
        }),
        None
    );
    assert!(quit.view().unwrap().primary.focused);
    assert!(matches!(
        quit.input(&ModalInput {
            accept: true,
            ..neutral()
        }),
        Some(NoticeIntent::Quit(_))
    ));
    assert_eq!(
        quit.input(&ModalInput {
            accept: true,
            ..neutral()
        }),
        None
    );
}

#[test]
fn failed_save_preserves_exact_guidance_and_error_and_requires_fresh_retry() {
    let mut modal = NoticeModal::quit(17, "Unsaved caller state stays active.");
    arm(&mut modal);
    let first = confirm(&mut modal);
    assert!(modal.quit_finished(
        first,
        Err("save failed: access denied\nprofile unchanged".into())
    ));
    let view = modal.view().unwrap();
    assert_eq!(view.guidance, "Unsaved caller state stays active.");
    assert_eq!(
        view.feedback,
        Some("save failed: access denied\nprofile unchanged")
    );
    assert!(!view.pending);
    assert!(view.cancel.unwrap().focused);
    assert_eq!(
        modal.input(&ModalInput {
            accept: true,
            ..neutral()
        }),
        None
    );
    arm(&mut modal);
    let second = confirm(&mut modal);
    assert_eq!(second.identity, first.identity);
    assert_ne!(second.attempt, first.attempt);
    assert!(!modal.quit_finished(first, Ok(())));
    assert!(!modal.quit_finished(first, Err("old failure".into())));
    assert_eq!(modal.pending_request(), Some(second));
    assert!(modal.quit_finished(second, Ok(())));
}

#[test]
fn stale_opening_feedback_and_unrequested_feedback_do_not_mutate_a_notice() {
    let mut first = NoticeModal::quit(18, "old");
    arm(&mut first);
    let old_request = confirm(&mut first);
    let mut next = NoticeModal::quit(19, "new");
    assert!(!next.quit_finished(old_request, Ok(())));
    assert!(!next.show_error(first.identity(), "old error"));
    assert_eq!(next.view().unwrap().guidance, "new");
    assert_eq!(next.view().unwrap().feedback, None);
    arm(&mut next);
    let next_request = confirm(&mut next);
    assert!(!next.quit_finished(old_request, Ok(())));
    assert_eq!(next.pending_request(), Some(next_request));
    assert!(!next.show_error(next.identity(), "bypass pending feedback"));
}

#[test]
fn information_preserves_caller_text_and_acknowledges_once() {
    let mut modal = NoticeModal::information(
        20,
        "Caller title",
        "Caller message\nsecond line",
        "Caller guidance",
    );
    let identity = modal.identity();
    assert!(modal.show_error(identity, "Caller diagnostic"));
    let view = modal.view().unwrap();
    assert_eq!(view.title, "Caller title");
    assert_eq!(view.body, "Caller message\nsecond line");
    assert_eq!(view.guidance, "Caller guidance");
    assert_eq!(view.feedback, Some("Caller diagnostic"));
    assert!(view.cancel.is_none());
    arm(&mut modal);
    let point = center(modal.view().unwrap().primary.control.bounds);
    assert_eq!(modal.input(&pointer(point, true, false, true)), None);
    assert_eq!(
        modal.input(&pointer(point, false, true, false)),
        Some(NoticeIntent::Dismiss(identity))
    );
    assert_eq!(
        modal.input(&ModalInput {
            accept: true,
            ..neutral()
        }),
        None
    );
}

#[test]
fn locked_mode_carries_original_lock_identity_and_never_admits_a_mode() {
    let mut modal = NoticeModal::locked_mode(
        21,
        Mode::Challenge,
        LockReason::CompleteAdventure,
        "Caller lock detail",
    );
    let identity = NoticeIdentity {
        id: 21,
        kind: NoticeKind::LockedMode {
            mode: Mode::Challenge,
            reason: LockReason::CompleteAdventure,
        },
    };
    assert_eq!(modal.identity(), identity);
    assert_eq!(
        modal.view().unwrap().body,
        LockReason::CompleteAdventure.guidance()
    );
    assert_eq!(modal.view().unwrap().guidance, "Caller lock detail");
    arm(&mut modal);
    assert_eq!(
        modal.input(&ModalInput {
            cancel: true,
            ..neutral()
        }),
        Some(NoticeIntent::Dismiss(identity))
    );
    assert_eq!(
        modal.input(&ModalInput {
            accept: true,
            ..neutral()
        }),
        None
    );
}

#[test]
fn view_bounds_keep_all_text_and_controls_disjoint_inside_the_panel() {
    let text = "long caller text ".repeat(300);
    for modal in [
        NoticeModal::quit(22, &text),
        NoticeModal::information(23, "Information", &text, &text),
    ] {
        let view = modal.view().unwrap();
        let mut regions = vec![
            view.title_bounds,
            view.body_bounds,
            view.guidance_bounds,
            view.feedback_bounds,
            view.primary.control.bounds,
        ];
        if let Some(cancel) = view.cancel {
            regions.push(cancel.control.bounds);
        }
        for (index, bounds) in regions.iter().enumerate() {
            assert!(bounds.w > 0.0 && bounds.h > 0.0);
            assert!(bounds.x >= view.panel.x && bounds.y >= view.panel.y);
            assert!(bounds.x + bounds.w <= view.panel.x + view.panel.w);
            assert!(bounds.y + bounds.h <= view.panel.y + view.panel.h);
            assert!(
                !regions
                    .iter()
                    .skip(index + 1)
                    .any(|other| bounds.overlaps(other))
            );
        }
        assert_eq!(view.guidance, text);
        assert!(view.panel.x >= 0.0 && view.panel.y >= 0.0);
        assert!(view.panel.x + view.panel.w <= 640.0);
        assert!(view.panel.y + view.panel.h <= 480.0);
    }
}
