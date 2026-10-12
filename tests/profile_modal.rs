//! Offline expectations come from the modal control/preservation contract and
//! cited W1 functional facts in docs/modals-profiles.md. No retail run is claimed.

use macroquad::prelude::{Rect, vec2};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use turbofish_deluxe::{
    highscores::ScoreKind as GlobalScoreKind,
    modal::ModalInput,
    profile_modal::{
        MAX_NAME_BYTES, ProfileAction, ProfileFrame, ProfileIntent, ProfileModal, ProfileRequest,
        ProfileRow, ProfileView, VISIBLE_ROWS,
    },
    profiles::{PROFILE_FORMAT_VERSION, Profiles},
    sim::Event,
};

fn input() -> ModalInput {
    ModalInput {
        focused: true,
        ..ModalInput::default()
    }
}

fn profiles(count: usize) -> Profiles {
    let mut profiles = Profiles::new("Profile 1", 304).unwrap();
    for index in 1..count {
        let mut fresh =
            Profiles::new(&format!("Profile {}", index + 1), 304 + index as u64).unwrap();
        let mut entry = fresh.entries.remove(0);
        entry.id = index as u64 + 1;
        profiles.entries.push(entry);
    }
    if count == 0 {
        profiles.entries.clear();
        profiles.current_id = None;
    }
    profiles.next_id = count as u64 + 1;
    profiles.validate().unwrap();
    profiles
}

fn click(modal: &mut ProfileModal, profiles: &Profiles, bounds: Rect) -> ProfileFrame {
    let pointer = vec2(bounds.x + bounds.w / 2.0, bounds.y + bounds.h / 2.0);
    let pressed = modal.input(
        &ModalInput {
            pointer,
            pressed: true,
            down: true,
            ..input()
        },
        profiles,
    );
    assert!(pressed.consumed && pressed.intent.is_none());
    modal.input(
        &ModalInput {
            pointer,
            released: true,
            ..input()
        },
        profiles,
    )
}

fn button(view: &ProfileView, label: &str) -> Rect {
    view.buttons
        .iter()
        .find(|button| button.label == label)
        .unwrap()
        .control
        .bounds
}

fn click_button(modal: &mut ProfileModal, profiles: &Profiles, label: &str) -> ProfileFrame {
    let bounds = button(&modal.view(profiles).unwrap(), label);
    click(modal, profiles, bounds)
}

fn click_row(modal: &mut ProfileModal, profiles: &Profiles, row: ProfileRow) -> ProfileFrame {
    let bounds = modal
        .view(profiles)
        .unwrap()
        .rows
        .iter()
        .find(|view| view.row == row)
        .unwrap()
        .control
        .bounds;
    click(modal, profiles, bounds)
}

fn request(frame: ProfileFrame) -> ProfileRequest {
    assert!(frame.consumed);
    match frame.intent.unwrap() {
        ProfileIntent::Transaction(request) => request,
        ProfileIntent::Close => panic!("expected transaction"),
    }
}

fn text(modal: &mut ProfileModal, profiles: &Profiles, text: &str) {
    assert!(
        modal
            .input(
                &ModalInput {
                    text: text.into(),
                    ..input()
                },
                profiles
            )
            .intent
            .is_none()
    );
}

#[test]
fn empty_chooser_keeps_create_and_cancel_consumes_its_closing_frame() {
    let profiles = profiles(0);
    let mut modal = ProfileModal::new(&profiles);
    let view = modal.view(&profiles).unwrap();
    assert_eq!(view.rows.len(), 1);
    assert_eq!(view.rows[0].row, ProfileRow::Create);
    assert!(
        !view
            .buttons
            .iter()
            .find(|button| button.label == "Rename")
            .unwrap()
            .control
            .enabled
    );
    click_row(&mut modal, &profiles, ProfileRow::Create);
    assert_eq!(modal.view(&profiles).unwrap().title, "Create profile");
    assert!(modal.view(&profiles).unwrap().edit.unwrap().focused);
    assert!(
        modal
            .input(
                &ModalInput {
                    accept: true,
                    ..input()
                },
                &profiles
            )
            .intent
            .is_none()
    );
    let cancel = modal.input(
        &ModalInput {
            cancel: true,
            ..input()
        },
        &profiles,
    );
    assert!(cancel.consumed && cancel.intent.is_none());
    assert_eq!(modal.view(&profiles).unwrap().title, "Profiles");
    let close = click_button(&mut modal, &profiles, "Cancel");
    assert!(close.consumed && close.intent == Some(ProfileIntent::Close));
    assert!(!modal.is_open());
    assert!(!modal.input(&input(), &profiles).consumed);
}

#[test]
fn chooser_selection_is_transient_and_confirm_carries_the_selected_id() {
    let profiles = profiles(3);
    let mut modal = ProfileModal::new(&profiles);
    assert_eq!(
        modal.view(&profiles).unwrap().rows[1].row,
        ProfileRow::Existing(1)
    );
    assert!(
        click_row(&mut modal, &profiles, ProfileRow::Existing(3))
            .intent
            .is_none()
    );
    assert_eq!(profiles.current_id, Some(1));
    let selected = request(click_button(&mut modal, &profiles, "Confirm"));
    assert_eq!(selected.action, ProfileAction::Select { id: 3 });
    assert!(modal.is_open());
    assert!(modal.transaction_finished(&selected, Ok(()), &profiles));
    assert!(!modal.is_open());
}

#[test]
fn row_capture_requires_same_release_and_survives_neither_focus_loss_nor_reorder() {
    let mut profiles = profiles(3);
    let mut modal = ProfileModal::new(&profiles);
    let row = modal.view(&profiles).unwrap().rows[2].control.bounds;
    let pointer = vec2(row.x + 10.0, row.y + 5.0);
    modal.input(
        &ModalInput {
            pointer,
            pressed: true,
            down: true,
            ..input()
        },
        &profiles,
    );
    modal.input(
        &ModalInput {
            pointer: vec2(1.0, 1.0),
            released: true,
            ..input()
        },
        &profiles,
    );
    assert!(
        modal
            .view(&profiles)
            .unwrap()
            .rows
            .iter()
            .any(|row| row.row == ProfileRow::Existing(1) && row.selected)
    );
    modal.input(
        &ModalInput {
            pointer,
            pressed: true,
            down: true,
            ..input()
        },
        &profiles,
    );
    modal.input(
        &ModalInput {
            focused: false,
            down: true,
            ..input()
        },
        &profiles,
    );
    modal.input(
        &ModalInput {
            pointer,
            released: true,
            ..input()
        },
        &profiles,
    );
    assert!(
        modal
            .view(&profiles)
            .unwrap()
            .rows
            .iter()
            .any(|row| row.row == ProfileRow::Existing(1) && row.selected)
    );
    modal.input(
        &ModalInput {
            pointer,
            pressed: true,
            down: true,
            ..input()
        },
        &profiles,
    );
    profiles.current_id = Some(3);
    modal.input(
        &ModalInput {
            pointer,
            released: true,
            ..input()
        },
        &profiles,
    );
    assert!(
        modal
            .view(&profiles)
            .unwrap()
            .rows
            .iter()
            .any(|row| row.row == ProfileRow::Existing(1) && row.selected)
    );
}

#[test]
fn removed_keyboard_selection_cannot_activate_the_shifted_row() {
    let mut profiles = profiles(3);
    let mut modal = ProfileModal::new(&profiles);
    click_row(&mut modal, &profiles, ProfileRow::Existing(2));
    profiles.entries.remove(1);
    let frame = modal.input(
        &ModalInput {
            accept: true,
            ..input()
        },
        &profiles,
    );
    assert!(frame.intent.is_none());
    assert_eq!(modal.view(&profiles).unwrap().title, "Create profile");
    assert_eq!(profiles.current_id, Some(1));
}

#[test]
fn rename_targets_selection_and_preselects_text_for_unicode_editing() {
    let profiles = profiles(2);
    let mut modal = ProfileModal::new(&profiles);
    click_row(&mut modal, &profiles, ProfileRow::Existing(2));
    click_button(&mut modal, &profiles, "Rename");
    assert!(modal.view(&profiles).unwrap().edit.unwrap().select_all);
    text(&mut modal, &profiles, "Zoë🐟");
    modal.input(
        &ModalInput {
            left: true,
            ..input()
        },
        &profiles,
    );
    modal.input(
        &ModalInput {
            backspace: true,
            ..input()
        },
        &profiles,
    );
    assert_eq!(modal.view(&profiles).unwrap().edit.unwrap().text, "Zo🐟");
    modal.input(
        &ModalInput {
            home: true,
            ..input()
        },
        &profiles,
    );
    modal.input(
        &ModalInput {
            delete: true,
            ..input()
        },
        &profiles,
    );
    text(&mut modal, &profiles, "A");
    let view = modal.view(&profiles).unwrap();
    let edit = view.edit.unwrap();
    assert_eq!(edit.text, "Ao🐟");
    assert!(edit.text.is_char_boundary(edit.cursor));
    let rename = request(modal.input(
        &ModalInput {
            accept: true,
            ..input()
        },
        &profiles,
    ));
    assert_eq!(
        rename.action,
        ProfileAction::Rename {
            id: 2,
            name: "Ao🐟".into()
        }
    );
    assert_eq!(profiles.current_id, Some(1));
}

#[test]
fn name_admission_uses_manager_rules_and_text_only_reaches_focused_edit() {
    let profiles = profiles(2);
    let mut modal = ProfileModal::new(&profiles);
    click_row(&mut modal, &profiles, ProfileRow::Create);
    text(&mut modal, &profiles, "  pRoFiLe 1  ");
    let view = modal.view(&profiles).unwrap();
    assert!(!view.buttons[0].control.enabled);
    assert!(view.feedback.contains("already used"));
    modal.input(
        &ModalInput {
            home: true,
            ..input()
        },
        &profiles,
    );
    // Return to a fresh field, retaining the ordinary cancellation boundary.
    modal.input(
        &ModalInput {
            cancel: true,
            ..input()
        },
        &profiles,
    );
    click_row(&mut modal, &profiles, ProfileRow::Create);
    text(&mut modal, &profiles, &"🐟".repeat(65));
    assert_eq!(
        modal.view(&profiles).unwrap().edit.unwrap().text.len(),
        MAX_NAME_BYTES
    );
    modal.input(
        &ModalInput {
            next: true,
            ..input()
        },
        &profiles,
    );
    text(&mut modal, &profiles, "ignored");
    assert_eq!(
        modal.view(&profiles).unwrap().edit.unwrap().text.len(),
        MAX_NAME_BYTES
    );
    let create = request(modal.input(
        &ModalInput {
            accept: true,
            ..input()
        },
        &profiles,
    ));
    assert_eq!(
        create.action,
        ProfileAction::Create {
            name: "🐟".repeat(64)
        }
    );
}

#[test]
fn stale_form_never_retargets_a_recreated_name_and_delete_defaults_to_cancel() {
    let mut profiles = profiles(2);
    let mut modal = ProfileModal::new(&profiles);
    click_row(&mut modal, &profiles, ProfileRow::Existing(2));
    click_button(&mut modal, &profiles, "Delete");
    let view = modal.view(&profiles).unwrap();
    assert!(
        view.buttons
            .iter()
            .find(|button| button.label == "Cancel")
            .unwrap()
            .focused
    );
    let removed = profiles.entries.pop().unwrap();
    let mut replacement = Profiles::new(&removed.name, 999).unwrap().entries.remove(0);
    replacement.id = 3;
    profiles.entries.push(replacement);
    profiles.next_id = 4;
    modal.input(&input(), &profiles);
    let view = modal.view(&profiles).unwrap();
    assert!(!view.buttons[0].control.enabled);
    assert!(
        click_button(&mut modal, &profiles, "Delete")
            .intent
            .is_none()
    );
    assert!(modal.pending_request().is_none());
    assert!(
        modal
            .input(
                &ModalInput {
                    accept: true,
                    ..input()
                },
                &profiles
            )
            .intent
            .is_none()
    );
    assert_eq!(modal.view(&profiles).unwrap().title, "Profiles");
}

#[test]
fn pending_request_blocks_duplicates_and_failure_retains_form_and_matching_attempt() {
    let profiles = profiles(2);
    let mut modal = ProfileModal::new(&profiles);
    click_row(&mut modal, &profiles, ProfileRow::Existing(2));
    click_button(&mut modal, &profiles, "Rename");
    text(&mut modal, &profiles, "  Updated  ");
    let first = request(modal.input(
        &ModalInput {
            accept: true,
            ..input()
        },
        &profiles,
    ));
    assert_eq!(
        first.action,
        ProfileAction::Rename {
            id: 2,
            name: "Updated".into()
        }
    );
    assert!(modal.view(&profiles).unwrap().pending);
    assert!(
        modal
            .input(
                &ModalInput {
                    accept: true,
                    cancel: true,
                    text: "lost".into(),
                    ..input()
                },
                &profiles
            )
            .intent
            .is_none()
    );
    let mut wrong = first.clone();
    wrong.sequence += 1;
    assert!(!modal.transaction_finished(&wrong, Ok(()), &profiles));
    assert!(modal.transaction_finished(&first, Err("disk write denied".into()), &profiles));
    let view = modal.view(&profiles).unwrap();
    assert_eq!(view.title, "Rename profile");
    assert_eq!(view.edit.unwrap().text, "  Updated  ");
    assert_eq!(view.feedback, "disk write denied");
    let second = request(modal.input(
        &ModalInput {
            accept: true,
            ..input()
        },
        &profiles,
    ));
    assert!(second.sequence > first.sequence && second.action == first.action);
    assert!(!modal.transaction_finished(&first, Ok(()), &profiles));
    assert_eq!(modal.pending_request(), Some(&second));
}

#[test]
fn text_change_invalidates_held_confirmation_and_keyboard_tab_targets_focus() {
    let profiles = profiles(2);
    let mut modal = ProfileModal::new(&profiles);
    modal.input(
        &ModalInput {
            next: true,
            ..input()
        },
        &profiles,
    );
    let selected = request(modal.input(
        &ModalInput {
            accept: true,
            ..input()
        },
        &profiles,
    ));
    assert_eq!(selected.action, ProfileAction::Select { id: 2 });
    modal.transaction_finished(&selected, Err("retry".into()), &profiles);
    click_button(&mut modal, &profiles, "Rename");
    text(&mut modal, &profiles, "New");
    let confirm = button(&modal.view(&profiles).unwrap(), "Confirm");
    let pointer = vec2(confirm.x + 5.0, confirm.y + 5.0);
    modal.input(
        &ModalInput {
            pointer,
            pressed: true,
            down: true,
            ..input()
        },
        &profiles,
    );
    // Shift focus back to the edit; a release from the old confirmation is inert.
    modal.input(
        &ModalInput {
            previous: true,
            down: true,
            ..input()
        },
        &profiles,
    );
    text(&mut modal, &profiles, "er");
    assert!(
        modal
            .input(
                &ModalInput {
                    pointer,
                    released: true,
                    ..input()
                },
                &profiles
            )
            .intent
            .is_none()
    );
    assert_eq!(modal.view(&profiles).unwrap().edit.unwrap().text, "Newer");
}

#[test]
fn long_list_paging_and_keyboard_selection_stay_bounded_after_shrink() {
    let mut profiles = profiles(40);
    let mut modal = ProfileModal::new(&profiles);
    for _ in 0..100 {
        modal.input(
            &ModalInput {
                down_key: true,
                ..input()
            },
            &profiles,
        );
    }
    let view = modal.view(&profiles).unwrap();
    assert_eq!(view.rows.len(), VISIBLE_ROWS);
    assert_eq!(view.rows.last().unwrap().row, ProfileRow::Existing(40));
    assert!(
        !view
            .buttons
            .iter()
            .find(|button| button.label == "Down")
            .unwrap()
            .control
            .enabled
    );
    let list = view.list_bounds.unwrap();
    for row in view.rows {
        assert!(row.control.bounds.x >= list.x && row.control.bounds.right() <= list.right());
        assert!(row.control.bounds.y >= list.y && row.control.bounds.bottom() <= list.bottom());
    }
    profiles.entries.truncate(1);
    modal.input(&input(), &profiles);
    let view = modal.view(&profiles).unwrap();
    assert_eq!(view.rows.len(), 2);
    assert_eq!(view.rows[0].row, ProfileRow::Create);
    for _ in 0..20 {
        click_button(&mut modal, &profiles, "Down");
        click_button(&mut modal, &profiles, "Up");
    }
    assert_eq!(modal.view(&profiles).unwrap().rows.len(), 2);
}

#[test]
fn long_unicode_edit_window_contains_caret_and_fits_caller_font_width() {
    let profiles = profiles(0);
    let mut modal = ProfileModal::new(&profiles);
    click_row(&mut modal, &profiles, ProfileRow::Create);
    text(&mut modal, &profiles, &"é🐟".repeat(30));
    let edit = modal.view(&profiles).unwrap().edit.unwrap();
    let measure = |text: &str| text.chars().count() as f32 * 8.0;
    let range = edit.visible_range(80.0, measure);
    assert_eq!(range.end, edit.cursor);
    assert!(range.start > 0);
    assert_eq!(measure(&edit.text[range.clone()]), 80.0);
    assert!(edit.text.is_char_boundary(range.start) && edit.text.is_char_boundary(range.end));
    modal.input(
        &ModalInput {
            home: true,
            ..input()
        },
        &profiles,
    );
    let edit = modal.view(&profiles).unwrap().edit.unwrap();
    let range = edit.visible_range(80.0, measure);
    assert_eq!(range.start, 0);
    assert!(range.end > edit.cursor && measure(&edit.text[range]) <= 80.0);
}

static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    directory: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(".scratch/modal-profiles/fixtures");
        fs::create_dir_all(&root).unwrap();
        let root = root.canonicalize().unwrap();
        let directory = root.join(format!(
            "{}-{}",
            std::process::id(),
            FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self { root, directory }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let directory = self.directory.canonicalize().unwrap();
        assert!(directory.starts_with(&self.root) && directory != self.root);
        fs::remove_dir_all(directory).unwrap();
    }
}

#[test]
fn caller_owned_real_failure_and_retry_keep_sessions_and_copied_score_names() {
    let fixture = Fixture::new();
    let path = fixture.directory.join("profiles.json");
    let blocked = fixture.directory.join("blocked.json");
    fs::create_dir(&blocked).unwrap();
    let mut profiles = Profiles::new("Owner", 304).unwrap();
    profiles
        .record_events(
            &path,
            &[Event::StageResultRecorded {
                tick: 1,
                tank: 1,
                level: 1,
                seconds: 42,
                settled_coin_ids: vec![],
                settled_amount: 0,
                final_balance: 0,
                personal_best_seconds: 42,
                survivors: None,
            }],
        )
        .unwrap();
    let saved = fs::read(&path).unwrap();
    let session = serde_json::to_vec(&profiles.current().unwrap().session).unwrap();
    let mut modal = ProfileModal::new(&profiles);
    click_button(&mut modal, &profiles, "Rename");
    text(&mut modal, &profiles, "Renamed");
    let failed = request(modal.input(
        &ModalInput {
            accept: true,
            ..input()
        },
        &profiles,
    ));
    let result = profiles
        .rename(&blocked, 1, "Renamed")
        .map_err(|error| error.to_string());
    assert!(result.is_err());
    assert_eq!(fs::read(&path).unwrap(), saved);
    assert_eq!(profiles.current().unwrap().name, "Owner");
    assert_eq!(
        serde_json::to_vec(&profiles.current().unwrap().session).unwrap(),
        session
    );
    assert!(modal.transaction_finished(&failed, result, &profiles));
    assert!(modal.is_open() && !modal.view(&profiles).unwrap().feedback.is_empty());
    let retry = request(modal.input(
        &ModalInput {
            accept: true,
            ..input()
        },
        &profiles,
    ));
    let ProfileAction::Rename { id, name } = &retry.action else {
        panic!("expected rename")
    };
    let result = profiles
        .rename(&path, *id, name)
        .map_err(|error| error.to_string());
    assert!(result.is_ok());
    assert!(modal.transaction_finished(&retry, result, &profiles));
    assert_eq!(modal.view(&profiles).unwrap().feedback, "Profile renamed.");
    assert_eq!(
        serde_json::to_vec(&profiles.current().unwrap().session).unwrap(),
        session
    );
    assert_eq!(
        profiles
            .global_scores
            .rows(GlobalScoreKind::Adventure, 1, Some(1))
            .unwrap()[0]
            .name,
        "Owner"
    );
    click_button(&mut modal, &profiles, "Delete");
    let delete = request(click_button(&mut modal, &profiles, "Delete"));
    assert_eq!(delete.action, ProfileAction::Delete { id: 1 });
    let result = profiles.delete(&path, 1).map_err(|error| error.to_string());
    assert!(modal.transaction_finished(&delete, result, &profiles));
    let reopened = Profiles::decode(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(reopened.format_version, PROFILE_FORMAT_VERSION);
    assert_eq!(PROFILE_FORMAT_VERSION, 3);
    assert!(reopened.entries.is_empty() && reopened.current_id.is_none());
    assert_eq!(
        reopened
            .global_scores
            .rows(GlobalScoreKind::Adventure, 1, Some(1))
            .unwrap()[0]
            .name,
        "Owner"
    );
    assert_eq!(
        modal.view(&profiles).unwrap().rows[0].row,
        ProfileRow::Create
    );
}
