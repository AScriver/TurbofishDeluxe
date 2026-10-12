//! Offline expectations follow the E125 control/preservation contract, not
//! retail pixel/feel claims. Fixtures never access the installation or saves.

use macroquad::prelude::{Rect, Vec2};
use turbofish_deluxe::{
    adventure::{AdventurePhase, AdventureSession},
    gameplay_modal::{
        GameplayModal, GameplayModalAction, GameplayModalFeedback, GameplayModalState,
        GameplayMode, GameplayPrompt,
    },
    invasion::InvasionTip,
    modal::ModalInput,
    sim::{Action, AdventureState, Event, PetKind},
};

fn state(phase: AdventurePhase) -> GameplayModalState {
    let mut session = AdventureSession::new(42);
    session.phase = phase;
    GameplayModalState::from_session(&session).unwrap()
}

fn confirmation() -> GameplayModalState {
    GameplayModalState {
        mode: GameplayMode::Adventure,
        tank: 2,
        level: 1,
        prompt: GameplayPrompt::PetConfirmation {
            selected: vec![PetKind::Stinky],
            capacity: 4,
        },
        can_accept: true,
    }
}

fn pointer_event(pointer: Vec2, pressed: bool, released: bool) -> ModalInput {
    ModalInput {
        pointer,
        pressed,
        released,
        down: pressed,
        focused: true,
        ..ModalInput::default()
    }
}

fn accept() -> ModalInput {
    ModalInput {
        accept: true,
        focused: true,
        ..ModalInput::default()
    }
}

#[test]
fn session_projection_covers_only_reserved_prompts_and_keeps_mode_tip_and_selection() {
    let cases = [
        (AdventurePhase::FirstTankRescue, GameplayMode::Adventure),
        (
            AdventurePhase::InvasionTutorial {
                tip: InvasionTip::Danger,
            },
            GameplayMode::Adventure,
        ),
        (
            AdventurePhase::TimeTrialInvasionTutorial {
                tip: InvasionTip::BattleTip,
            },
            GameplayMode::TimeTrial,
        ),
        (
            AdventurePhase::ChallengeInvasionTutorial {
                tip: InvasionTip::GusWarning,
            },
            GameplayMode::Challenge,
        ),
        (
            AdventurePhase::GameOver { updates: 31 },
            GameplayMode::Adventure,
        ),
        (
            AdventurePhase::TimeTrialGameOver { updates: 31 },
            GameplayMode::TimeTrial,
        ),
    ];
    for (phase, expected_mode) in cases {
        let projected = state(phase.clone());
        assert_eq!(projected.mode, expected_mode);
        assert!(projected.can_accept);
        if let AdventurePhase::InvasionTutorial { tip }
        | AdventurePhase::TimeTrialInvasionTutorial { tip }
        | AdventurePhase::ChallengeInvasionTutorial { tip } = phase
        {
            assert_eq!(projected.prompt, GameplayPrompt::InvasionTutorial { tip });
        }
    }
    let mut session = AdventureSession::new(42);
    session.progress.unlocked_pets = vec![
        PetKind::Stinky,
        PetKind::Niko,
        PetKind::Itchy,
        PetKind::Prego,
    ];
    session.progress.pet_capacity = 4;
    session.board = None;
    session.phase = AdventurePhase::ChallengePetSelectionConfirmation {
        tank: 3,
        selected: vec![PetKind::Niko],
    };
    let projected = GameplayModalState::from_session(&session).unwrap();
    assert_eq!(
        (projected.mode, projected.tank, projected.level),
        (GameplayMode::Challenge, 3, 0)
    );
    assert_eq!(
        projected.prompt,
        GameplayPrompt::PetConfirmation {
            selected: vec![PetKind::Niko],
            capacity: 4
        }
    );
    session.progress.tank = 5;
    session.progress.level = 1;
    session.phase = AdventurePhase::PetSelectionConfirmation { selected: vec![] };
    assert!(
        !GameplayModalState::from_session(&session)
            .unwrap()
            .can_accept
    );
    for phase in [
        AdventurePhase::Playing,
        AdventurePhase::PetSelection { selected: vec![] },
        AdventurePhase::TimeTrialPetSelection {
            tank: 1,
            selected: vec![],
        },
        AdventurePhase::ChallengeLost,
    ] {
        session.phase = phase;
        assert!(GameplayModalState::from_session(&session).is_none());
    }
}

#[test]
fn game_over_cannot_activate_at_thirty_and_requires_new_press_after_admission() {
    for phase in [
        AdventurePhase::GameOver { updates: 30 },
        AdventurePhase::TimeTrialGameOver { updates: 30 },
    ] {
        let locked = state(phase);
        let mut modal = GameplayModal::new(1, locked.clone(), 34.0);
        let pointer = modal.view().buttons[0].control.bounds.center();
        assert!(!modal.view().buttons[0].control.enabled);
        assert!(modal.input(&accept()).is_none());
        assert!(modal.input(&pointer_event(pointer, true, false)).is_none());
        let mut admitted = locked;
        admitted.can_accept = true;
        assert!(modal.refresh(&admitted));
        assert!(modal.input(&pointer_event(pointer, false, true)).is_none());
        assert!(modal.input(&pointer_event(pointer, true, false)).is_none());
        let intent = modal.input(&pointer_event(pointer, false, true)).unwrap();
        assert_eq!(modal.action_for(&intent, &admitted), Some(Action::Continue));
    }
    assert!(!state(AdventurePhase::GameOver { updates: 0 }).can_accept);
    assert!(state(AdventurePhase::GameOver { updates: 31 }).can_accept);
}

#[test]
fn view_has_one_bounded_geometry_for_text_drawing_and_controls() {
    // Project layout contract: 640x480 canvas, 36px sides/footer, 16px pair gap.
    for height in [f32::NAN, -10.0, 16.0, 34.0, 40.0, 900.0] {
        let modal = GameplayModal::new(1, confirmation(), height);
        let view = modal.view();
        assert_eq!(view.panel, Rect::new(96.0, 110.0, 448.0, 272.0));
        assert!(view.body.contains("1 of 4"));
        for bounds in [
            view.title_bounds,
            view.body_bounds,
            view.feedback_bounds,
            view.buttons[0].control.bounds,
            view.buttons[1].control.bounds,
        ] {
            assert!(bounds.x >= view.panel.x && bounds.y >= view.panel.y);
            assert!(bounds.right() <= view.panel.right() && bounds.bottom() <= view.panel.bottom());
            assert!(bounds.w > 0.0 && bounds.h > 0.0);
        }
        assert_eq!(view.buttons[0].control.bounds.w, 180.0);
        assert_eq!(
            view.buttons[1].control.bounds.x - view.buttons[0].control.bounds.right(),
            16.0
        );
        assert_eq!(
            view.panel.bottom() - view.buttons[0].control.bounds.bottom(),
            36.0
        );
        assert!(view.feedback_bounds.bottom() < view.buttons[0].control.bounds.y);
    }
}

#[test]
fn captured_release_never_activates_another_button_or_background() {
    let mut modal = GameplayModal::new(1, confirmation(), 34.0);
    let buttons = modal.view().buttons;
    let yes = buttons[0].control.bounds.center();
    let no = buttons[1].control.bounds.center();
    assert!(modal.input(&pointer_event(yes, true, false)).is_none());
    assert!(modal.input(&pointer_event(no, false, true)).is_none());
    assert!(!modal.view().pending);
    assert!(
        modal
            .input(&pointer_event(Vec2::new(5.0, 5.0), true, false))
            .is_none()
    );
    assert!(modal.input(&pointer_event(yes, false, true)).is_none());
    assert!(modal.input(&pointer_event(yes, true, false)).is_none());
    let intent = modal.input(&pointer_event(yes, false, true)).unwrap();
    assert_eq!(intent.action, GameplayModalAction::ConfirmPets);
    assert_eq!(
        modal.action_for(&intent, &confirmation()),
        Some(Action::ConfirmPetSelection { accept: true })
    );
}

#[test]
fn focus_loss_and_cancel_retire_capture_without_dismissing_required_prompt() {
    let rescue = state(AdventurePhase::FirstTankRescue);
    let mut modal = GameplayModal::new(1, rescue, 34.0);
    let pointer = modal.view().buttons[0].control.bounds.center();
    modal.input(&pointer_event(pointer, true, false));
    assert!(
        modal
            .input(&ModalInput {
                focused: false,
                accept: true,
                ..ModalInput::default()
            })
            .is_none()
    );
    assert!(modal.input(&pointer_event(pointer, false, true)).is_none());
    modal.input(&pointer_event(pointer, true, false));
    assert!(
        modal
            .input(&ModalInput {
                focused: true,
                cancel: true,
                ..ModalInput::default()
            })
            .is_none()
    );
    assert!(modal.input(&pointer_event(pointer, false, true)).is_none());
    assert!(!modal.view().pending);
    let mut confirmation = GameplayModal::new(2, confirmation(), 34.0);
    let intent = confirmation
        .input(&ModalInput {
            focused: true,
            cancel: true,
            ..ModalInput::default()
        })
        .unwrap();
    assert_eq!(intent.action, GameplayModalAction::ChooseMorePets);
}

#[test]
fn keyboard_focus_default_and_traversal_preserve_confirmation_choices() {
    let mut modal = GameplayModal::new(1, confirmation(), 34.0);
    let accepted = modal.input(&accept()).unwrap();
    assert_eq!(accepted.action, GameplayModalAction::ConfirmPets);
    assert!(modal.transaction_result(
        &accepted,
        GameplayModalFeedback::Rejected("Try again".to_owned())
    ));
    assert!(
        modal
            .input(&ModalInput {
                next: true,
                focused: true,
                ..ModalInput::default()
            })
            .is_none()
    );
    let declined = modal.input(&accept()).unwrap();
    assert_eq!(declined.action, GameplayModalAction::ChooseMorePets);
}

#[test]
fn stale_phase_payload_mode_admission_and_generation_reject_old_intent() {
    let original = confirmation();
    let mut modal = GameplayModal::new(4, original.clone(), 34.0);
    let intent = modal.request(GameplayModalAction::ConfirmPets).unwrap();
    let mut changed = original.clone();
    changed.prompt = GameplayPrompt::PetConfirmation {
        selected: vec![PetKind::Niko],
        capacity: 4,
    };
    assert!(!modal.refresh(&changed));
    assert!(modal.action_for(&intent, &changed).is_none());
    changed = original.clone();
    changed.can_accept = false;
    assert!(modal.action_for(&intent, &changed).is_none());
    changed = original.clone();
    changed.mode = GameplayMode::Challenge;
    assert!(modal.action_for(&intent, &changed).is_none());
    let mut replacement = GameplayModal::new(5, original.clone(), 34.0);
    let current = replacement
        .request(GameplayModalAction::ConfirmPets)
        .unwrap();
    assert!(
        !replacement.transaction_result(&intent, GameplayModalFeedback::Rejected("old".to_owned()))
    );
    assert!(replacement.action_for(&intent, &original).is_none());
    assert_eq!(
        replacement.action_for(&current, &original),
        Some(Action::ConfirmPetSelection { accept: true })
    );
}

#[test]
fn transaction_feedback_keeps_failure_visible_and_blocks_duplicate_or_late_requests() {
    let mut modal = GameplayModal::new(1, confirmation(), 34.0);
    let first = modal.input(&accept()).unwrap();
    assert!(
        modal
            .view()
            .buttons
            .iter()
            .all(|button| !button.control.enabled)
    );
    assert!(modal.input(&accept()).is_none());
    assert!(modal.request(GameplayModalAction::ChooseMorePets).is_none());
    assert!(modal.transaction_result(
        &first,
        GameplayModalFeedback::Rejected("Selection could not be applied.".to_owned())
    ));
    assert_eq!(
        modal.view().feedback.as_deref(),
        Some("Selection could not be applied.")
    );
    assert!(!modal.view().pending);
    let second = modal.request(GameplayModalAction::ConfirmPets).unwrap();
    assert_ne!(first.request, second.request);
    assert!(!modal.transaction_result(&first, GameplayModalFeedback::Rejected("late".to_owned())));
    assert!(modal.view().feedback.is_none());
    assert!(modal.transaction_result(&second, GameplayModalFeedback::Applied));
    assert!(modal.view().pending);
    assert!(modal.request(GameplayModalAction::ConfirmPets).is_none());
}

#[test]
fn rescue_intent_uses_existing_once_only_domain_transaction_without_advancing_clock() {
    let mut session = AdventureSession::new(42);
    session.board.as_mut().unwrap().fish.clear();
    session.step(&[]);
    assert_eq!(session.phase, AdventurePhase::FirstTankRescue);
    let tick = session.ticks;
    let board_tick = session.board.as_ref().unwrap().tick;
    let balance = session.board.as_ref().unwrap().balance;
    let snapshot = GameplayModalState::from_session(&session).unwrap();
    let mut modal = GameplayModal::new(1, snapshot.clone(), 34.0);
    let intent = modal.input(&accept()).unwrap();
    let action = modal.action_for(&intent, &snapshot).unwrap();
    let events = session.apply_actions(&[action]);
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, Event::RescueGuppyGranted { .. }))
            .count(),
        1
    );
    assert_eq!(session.phase, AdventurePhase::Playing);
    assert_eq!(
        (
            session.ticks,
            session.board.as_ref().unwrap().tick,
            session.board.as_ref().unwrap().balance
        ),
        (tick, board_tick, balance)
    );
    assert_eq!(
        session
            .board
            .as_ref()
            .unwrap()
            .fish
            .iter()
            .filter(|fish| fish.alive)
            .count(),
        1
    );
    assert!(GameplayModalState::from_session(&session).is_none());
    assert!(modal.request(GameplayModalAction::Continue).is_none());
}

fn second_stage_session() -> AdventureSession {
    let mut session = AdventureSession::new(42);
    session.progress.level = 2;
    session.progress.unlocked_pets = vec![PetKind::Stinky];
    session.board = Some(AdventureState::new_second_stage(42));
    session
}

#[test]
fn tutorial_intent_acknowledges_existing_wave_without_advancing_or_replacing_board() {
    let mut session = second_stage_session();
    session
        .board
        .as_mut()
        .unwrap()
        .invasion
        .as_mut()
        .unwrap()
        .countdown = 277;
    session.step(&[]);
    assert_eq!(
        session.phase,
        AdventurePhase::InvasionTutorial {
            tip: InvasionTip::Danger
        }
    );
    let snapshot = GameplayModalState::from_session(&session).unwrap();
    let board_tick = session.board.as_ref().unwrap().tick;
    let fish_before = serde_json::to_value(&session.board.as_ref().unwrap().fish).unwrap();
    let mut modal = GameplayModal::new(1, snapshot.clone(), 34.0);
    let intent = modal.input(&accept()).unwrap();
    session.apply_actions(&[modal.action_for(&intent, &snapshot).unwrap()]);
    assert_eq!(session.phase, AdventurePhase::Playing);
    let board = session.board.as_ref().unwrap();
    assert_eq!(board.tick, board_tick);
    assert_eq!(board.invasion.as_ref().unwrap().countdown, 276);
    assert_eq!(board.invasion.as_ref().unwrap().pending_modal, None);
    assert_eq!(serde_json::to_value(&board.fish).unwrap(), fish_before);
}

#[test]
fn admitted_game_over_intent_returns_to_menu_and_preserves_existing_progress() {
    let mut session = second_stage_session();
    let progress = session.progress.clone();
    session.board.as_mut().unwrap().fish.clear();
    session.step(&[]);
    assert_eq!(session.phase, AdventurePhase::GameOver { updates: 0 });
    for _ in 0..30 {
        session.step(&[]);
    }
    let mut modal =
        GameplayModal::new(1, GameplayModalState::from_session(&session).unwrap(), 34.0);
    assert!(modal.input(&accept()).is_none());
    session.step(&[]);
    let admitted = GameplayModalState::from_session(&session).unwrap();
    assert!(modal.refresh(&admitted));
    let intent = modal.input(&accept()).unwrap();
    let events = session.apply_actions(&[modal.action_for(&intent, &admitted).unwrap()]);
    assert_eq!(session.phase, AdventurePhase::GameSelector);
    assert!(session.board.is_none());
    assert_eq!(session.progress, progress);
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, Event::GameSelectorOpened { .. }))
            .count(),
        1
    );
}
