//! Transient profile chooser/forms. Profile IDs and caller-owned transactions
//! remain authoritative; no session, save path, clock or graphics context lives here.

use crate::{
    modal::{ModalButton, ModalControls, ModalInput},
    profiles::Profiles,
};
use macroquad::prelude::Rect;

pub const MAX_NAME_BYTES: usize = 256;
pub const VISIBLE_ROWS: usize = 8;
const CREATE: u64 = 0;
const RENAME: u64 = 1;
const DELETE: u64 = 2;
const PREVIOUS: u64 = 3;
const NEXT: u64 = 4;
const CONFIRM: u64 = 5;
const CANCEL: u64 = 6;
const EDIT: u64 = 7;
const FIRST_PROFILE: u64 = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProfileAction {
    Create { name: String },
    Select { id: u64 },
    Rename { id: u64, name: String },
    Delete { id: u64 },
}

/// Match the complete request when returning a result. A late response cannot
/// resolve a different attempt or silently replace a nested form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileRequest {
    pub sequence: u64,
    pub action: ProfileAction,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProfileIntent {
    Transaction(ProfileRequest),
    Close,
}

pub struct ProfileFrame {
    pub consumed: bool,
    pub intent: Option<ProfileIntent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileRow {
    Create,
    Existing(u64),
}

pub struct ProfileButtonView {
    pub control: ModalButton,
    pub label: &'static str,
    pub pressed: bool,
    pub focused: bool,
}

pub struct ProfileRowView {
    pub control: ModalButton,
    pub row: ProfileRow,
    pub name: String,
    pub selected: bool,
    pub current: bool,
    pub pressed: bool,
    pub focused: bool,
}

pub struct ProfileEditView {
    pub bounds: Rect,
    pub text: String,
    /// UTF-8 byte boundary, never a character index.
    pub cursor: usize,
    pub select_all: bool,
    pub focused: bool,
    pub enabled: bool,
}

impl ProfileEditView {
    /// Keep the caret and nearby text inside the field using caller-owned font
    /// metrics. No GPU or install is needed to exercise this layout boundary.
    pub fn visible_range(
        &self,
        width: f32,
        measure: impl Fn(&str) -> f32,
    ) -> std::ops::Range<usize> {
        let cursor = self.cursor;
        let mut start = cursor;
        for (offset, _) in self.text[..cursor].char_indices().rev() {
            if measure(&self.text[offset..cursor]) > width.max(0.0) {
                break;
            }
            start = offset;
        }
        let mut end = cursor;
        for character in self.text[cursor..].chars() {
            let candidate = end + character.len_utf8();
            if measure(&self.text[start..candidate]) > width.max(0.0) {
                break;
            }
            end = candidate;
        }
        start..end
    }
}

/// All paint and hit bounds share this view on the parent's logical canvas.
pub struct ProfileView {
    pub panel: Rect,
    pub title_bounds: Rect,
    pub title: &'static str,
    pub body_bounds: Rect,
    pub body: String,
    pub list_bounds: Option<Rect>,
    pub rows: Vec<ProfileRowView>,
    pub edit: Option<ProfileEditView>,
    pub buttons: Vec<ProfileButtonView>,
    pub position_bounds: Rect,
    pub position: String,
    pub feedback_bounds: Rect,
    pub feedback: String,
    pub pending: bool,
}

impl ProfileView {
    fn controls(&self) -> Vec<ModalButton> {
        let mut controls: Vec<_> = self.rows.iter().map(|row| row.control).collect();
        if let Some(edit) = &self.edit {
            controls.push(ModalButton {
                id: EDIT,
                bounds: edit.bounds,
                enabled: edit.enabled,
            });
        }
        controls.extend(self.buttons.iter().map(|button| button.control));
        controls
    }
}

struct NameEdit {
    text: String,
    cursor: usize,
    select_all: bool,
}

impl NameEdit {
    fn new(text: String, select_all: bool) -> Self {
        Self {
            cursor: text.len(),
            text,
            select_all,
        }
    }

    fn update(&mut self, input: &ModalInput) -> bool {
        let before = self.text.clone();
        if input.home {
            self.cursor = 0;
            self.select_all = false;
        } else if input.end {
            self.cursor = self.text.len();
            self.select_all = false;
        } else if input.left {
            self.cursor = if self.select_all {
                0
            } else {
                self.text[..self.cursor]
                    .char_indices()
                    .next_back()
                    .map_or(0, |(offset, _)| offset)
            };
            self.select_all = false;
        } else if input.right {
            self.cursor = if self.select_all {
                self.text.len()
            } else {
                self.cursor
                    + self.text[self.cursor..]
                        .chars()
                        .next()
                        .map_or(0, char::len_utf8)
            };
            self.select_all = false;
        }
        if input.backspace || input.delete {
            if self.select_all {
                self.text.clear();
                self.cursor = 0;
                self.select_all = false;
            } else if input.backspace && self.cursor > 0 {
                let start = self.text[..self.cursor]
                    .char_indices()
                    .next_back()
                    .map_or(0, |(offset, _)| offset);
                self.text.replace_range(start..self.cursor, "");
                self.cursor = start;
            } else if input.delete
                && let Some(character) = self.text[self.cursor..].chars().next()
            {
                self.text
                    .replace_range(self.cursor..self.cursor + character.len_utf8(), "");
            }
        }
        for character in input
            .text
            .chars()
            .filter(|character| !character.is_control())
        {
            let retained = if self.select_all { 0 } else { self.text.len() };
            if retained + character.len_utf8() > MAX_NAME_BYTES {
                break;
            }
            if self.select_all {
                self.text.clear();
                self.cursor = 0;
                self.select_all = false;
            }
            self.text.insert(self.cursor, character);
            self.cursor += character.len_utf8();
        }
        self.text != before
    }
}

enum Form {
    Create(NameEdit),
    Rename { id: u64, edit: NameEdit },
    Delete { id: u64, name: String },
}

impl Form {
    fn edit(&self) -> Option<&NameEdit> {
        match self {
            Self::Create(edit) | Self::Rename { edit, .. } => Some(edit),
            Self::Delete { .. } => None,
        }
    }

    fn edit_mut(&mut self) -> Option<&mut NameEdit> {
        match self {
            Self::Create(edit) | Self::Rename { edit, .. } => Some(edit),
            Self::Delete { .. } => None,
        }
    }
}

pub struct ProfileModal {
    open: bool,
    controls: ModalControls,
    selected: ProfileRow,
    first_row: usize,
    known_ids: Vec<u64>,
    known_current: Option<u64>,
    form: Option<Form>,
    pending: Option<ProfileRequest>,
    sequence: u64,
    feedback: String,
}

impl ProfileModal {
    pub fn new(profiles: &Profiles) -> Self {
        let selected = profiles
            .current()
            .map_or(ProfileRow::Create, |entry| ProfileRow::Existing(entry.id));
        let mut modal = Self {
            open: true,
            controls: ModalControls::default(),
            selected,
            first_row: 0,
            known_ids: Self::ordered_ids(profiles),
            known_current: profiles.current_id,
            form: None,
            pending: None,
            sequence: 0,
            feedback: String::new(),
        };
        modal.focus_selected();
        modal
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn pending_request(&self) -> Option<&ProfileRequest> {
        self.pending.as_ref()
    }

    pub fn cancel_capture(&mut self) {
        self.controls.cancel_capture();
    }

    fn ordered_ids(profiles: &Profiles) -> Vec<u64> {
        profiles
            .current()
            .map(|entry| entry.id)
            .into_iter()
            .chain(
                profiles
                    .entries
                    .iter()
                    .filter(|entry| Some(entry.id) != profiles.current_id)
                    .map(|entry| entry.id),
            )
            .collect()
    }

    fn synchronize(&mut self, profiles: &Profiles) {
        let ids = Self::ordered_ids(profiles);
        let roster_changed = ids != self.known_ids || self.known_current != profiles.current_id;
        if roster_changed {
            // Row-control indices only identify this unchanged roster. Stable
            // domain IDs travel in requests; a reorder cannot retarget a hold.
            self.controls.cancel_capture();
            self.known_ids = ids;
            self.known_current = profiles.current_id;
        }
        self.first_row = self
            .first_row
            .min(self.known_ids.len().saturating_sub(VISIBLE_ROWS - 1));
        if let ProfileRow::Existing(id) = self.selected
            && !self.known_ids.contains(&id)
        {
            self.selected = ProfileRow::Create;
            self.feedback = "Selected profile no longer exists. Choose a profile.".into();
        }
        if roster_changed && self.form.is_none() {
            self.focus_selected();
        }
    }

    fn focus_selected(&mut self) {
        let index = match self.selected {
            ProfileRow::Create => 0,
            ProfileRow::Existing(id) => self
                .known_ids
                .iter()
                .position(|candidate| *candidate == id)
                .map_or(0, |index| index + 1),
        };
        if index < self.first_row {
            self.first_row = index;
        } else if index >= self.first_row + VISIBLE_ROWS {
            self.first_row = index + 1 - VISIBLE_ROWS;
        }
        self.controls.set_focus(Some(if index == 0 {
            CREATE
        } else {
            FIRST_PROFILE + (index - 1) as u64
        }));
    }

    fn begin_form(&mut self, form: Form) {
        self.controls.cancel_capture();
        self.controls
            .set_focus(Some(if form.edit().is_some() { EDIT } else { CANCEL }));
        self.form = Some(form);
        self.feedback.clear();
    }

    fn validation_error(&self, profiles: &Profiles) -> Option<&'static str> {
        let form = self.form.as_ref()?;
        let target = match form {
            Form::Create(_) => None,
            Form::Rename { id, .. } | Form::Delete { id, .. } => Some(*id),
        };
        if target.is_some_and(|id| !profiles.entries.iter().any(|entry| entry.id == id)) {
            return Some("This profile no longer exists. Cancel and choose again.");
        }
        if let Some(edit) = form.edit() {
            let name = edit.text.trim();
            if name.is_empty() {
                return Some("Enter a profile name.");
            }
            if name.len() > MAX_NAME_BYTES || name.chars().any(char::is_control) {
                return Some("Profile name is too long or contains a control character.");
            }
            let folded = name.to_lowercase();
            if profiles
                .entries
                .iter()
                .any(|entry| Some(entry.id) != target && entry.name.to_lowercase() == folded)
            {
                return Some("That profile name is already used.");
            }
        }
        None
    }

    fn button(
        &self,
        id: u64,
        bounds: Rect,
        label: &'static str,
        enabled: bool,
    ) -> ProfileButtonView {
        ProfileButtonView {
            control: ModalButton {
                id,
                bounds,
                enabled,
            },
            label,
            pressed: self.controls.is_pressed(id),
            focused: self.controls.is_focused(id),
        }
    }

    pub fn view(&self, profiles: &Profiles) -> Option<ProfileView> {
        if !self.open {
            return None;
        }
        let ready = self.pending.is_none();
        let mut view = ProfileView {
            panel: Rect::new(120.0, 32.0, 400.0, 416.0),
            title_bounds: Rect::new(144.0, 49.0, 352.0, 40.0),
            title: "Profiles",
            body_bounds: Rect::new(154.0, 91.0, 332.0, 26.0),
            body: "Choose a profile, then confirm.".into(),
            list_bounds: Some(Rect::new(154.0, 124.0, 306.0, 144.0)),
            rows: Vec::new(),
            edit: None,
            buttons: Vec::new(),
            position_bounds: Rect::new(154.0, 274.0, 332.0, 18.0),
            position: String::new(),
            feedback_bounds: Rect::new(154.0, 333.0, 332.0, 36.0),
            feedback: if ready {
                self.feedback.clone()
            } else {
                "Saving profile changes...".into()
            },
            pending: !ready,
        };
        if let Some(form) = &self.form {
            view.panel = Rect::new(120.0, 105.0, 400.0, 272.0);
            view.title_bounds = Rect::new(144.0, 124.0, 352.0, 36.0);
            view.body_bounds = Rect::new(154.0, 170.0, 332.0, 42.0);
            view.list_bounds = None;
            view.feedback_bounds = Rect::new(154.0, 254.0, 332.0, 47.0);
            view.title = match form {
                Form::Create(_) => "Create profile",
                Form::Rename { .. } => "Rename profile",
                Form::Delete { .. } => "Delete profile",
            };
            view.body = match form {
                Form::Delete { name, .. } => format!(
                    "Remove {name} and its saved progress? Earned score names stay recorded."
                ),
                _ => "Enter the profile name.".into(),
            };
            if let Some(edit) = form.edit() {
                view.edit = Some(ProfileEditView {
                    bounds: Rect::new(168.0, 219.0, 304.0, 28.0),
                    text: edit.text.clone(),
                    cursor: edit.cursor,
                    select_all: edit.select_all,
                    focused: self.controls.is_focused(EDIT),
                    enabled: ready,
                });
            }
            let error = self.validation_error(profiles);
            if ready && view.feedback.is_empty() {
                view.feedback = error.unwrap_or_default().into();
            }
            view.buttons.push(self.button(
                CONFIRM,
                Rect::new(156.0, 311.0, 145.0, 36.0),
                if matches!(form, Form::Delete { .. }) {
                    "Delete"
                } else {
                    "Confirm"
                },
                ready && error.is_none(),
            ));
            view.buttons.push(self.button(
                CANCEL,
                Rect::new(339.0, 311.0, 145.0, 36.0),
                "Cancel",
                ready,
            ));
            return Some(view);
        }
        let ids = Self::ordered_ids(profiles);
        let total = ids.len() + 1;
        let first = self.first_row.min(total.saturating_sub(VISIBLE_ROWS));
        for (visible, index) in (first..total.min(first + VISIBLE_ROWS)).enumerate() {
            let row = if index == 0 {
                ProfileRow::Create
            } else {
                ProfileRow::Existing(ids[index - 1])
            };
            let id = if index == 0 {
                CREATE
            } else {
                FIRST_PROFILE + (index - 1) as u64
            };
            let name = match row {
                ProfileRow::Create => "Create profile...".to_owned(),
                ProfileRow::Existing(id) => profiles
                    .entries
                    .iter()
                    .find(|entry| entry.id == id)?
                    .name
                    .clone(),
            };
            view.rows.push(ProfileRowView {
                control: ModalButton {
                    id,
                    bounds: Rect::new(154.0, 124.0 + visible as f32 * 18.0, 306.0, 18.0),
                    enabled: ready,
                },
                row,
                name,
                selected: row == self.selected,
                current: matches!(row, ProfileRow::Existing(id) if Some(id) == profiles.current_id),
                pressed: self.controls.is_pressed(id),
                focused: self.controls.is_focused(id),
            });
        }
        view.position = format!(
            "{}-{} of {}",
            first + 1,
            total.min(first + VISIBLE_ROWS),
            total
        );
        let selected_exists =
            matches!(self.selected, ProfileRow::Existing(id) if ids.contains(&id));
        view.buttons = vec![
            self.button(
                PREVIOUS,
                Rect::new(464.0, 124.0, 40.0, 26.0),
                "Up",
                ready && first > 0,
            ),
            self.button(
                NEXT,
                Rect::new(464.0, 242.0, 40.0, 26.0),
                "Down",
                ready && first + VISIBLE_ROWS < total,
            ),
            self.button(
                RENAME,
                Rect::new(156.0, 295.0, 145.0, 32.0),
                "Rename",
                ready && selected_exists,
            ),
            self.button(
                DELETE,
                Rect::new(339.0, 295.0, 145.0, 32.0),
                "Delete",
                ready && selected_exists,
            ),
            self.button(
                CONFIRM,
                Rect::new(156.0, 378.0, 145.0, 36.0),
                "Confirm",
                ready,
            ),
            self.button(
                CANCEL,
                Rect::new(339.0, 378.0, 145.0, 36.0),
                "Cancel",
                ready,
            ),
        ];
        Some(view)
    }

    fn submit(&mut self, profiles: &Profiles) -> Option<ProfileIntent> {
        if let Some(error) = self.validation_error(profiles) {
            self.feedback = error.into();
            return None;
        }
        let action = match &self.form {
            Some(Form::Create(edit)) => ProfileAction::Create {
                name: edit.text.trim().into(),
            },
            Some(Form::Rename { id, edit }) => ProfileAction::Rename {
                id: *id,
                name: edit.text.trim().into(),
            },
            Some(Form::Delete { id, .. }) => ProfileAction::Delete { id: *id },
            None => match self.selected {
                ProfileRow::Create => {
                    self.begin_form(Form::Create(NameEdit::new(String::new(), false)));
                    return None;
                }
                ProfileRow::Existing(id) if profiles.entries.iter().any(|entry| entry.id == id) => {
                    ProfileAction::Select { id }
                }
                ProfileRow::Existing(_) => return None,
            },
        };
        let Some(sequence) = self.sequence.checked_add(1) else {
            self.feedback = "Cannot allocate another profile request.".into();
            return None;
        };
        self.sequence = sequence;
        let request = ProfileRequest { sequence, action };
        self.pending = Some(request.clone());
        self.controls.cancel_capture();
        Some(ProfileIntent::Transaction(request))
    }

    pub fn input(&mut self, input: &ModalInput, profiles: &Profiles) -> ProfileFrame {
        let mut frame = ProfileFrame {
            consumed: self.open,
            intent: None,
        };
        if !self.open {
            return frame;
        }
        self.synchronize(profiles);
        if !input.focused || self.pending.is_some() {
            self.controls.cancel_capture();
            return frame;
        }
        if input.cancel {
            self.controls.cancel_capture();
            if self.form.take().is_some() {
                self.feedback.clear();
                self.focus_selected();
            } else {
                self.open = false;
                frame.intent = Some(ProfileIntent::Close);
            }
            return frame;
        }
        if self.form.is_none() && (input.up || input.down_key) {
            let current = match self.selected {
                ProfileRow::Create => 0,
                ProfileRow::Existing(id) => self
                    .known_ids
                    .iter()
                    .position(|candidate| *candidate == id)
                    .map_or(0, |index| index + 1),
            };
            let index = if input.up {
                current.saturating_sub(1)
            } else {
                (current + 1).min(self.known_ids.len())
            };
            self.selected = if index == 0 {
                ProfileRow::Create
            } else {
                ProfileRow::Existing(self.known_ids[index - 1])
            };
            self.controls.cancel_capture();
            self.focus_selected();
            self.feedback.clear();
        }
        let accept = input.accept && !input.next && !input.previous;
        let edit_accept = accept
            && self.form.as_ref().is_some_and(|form| form.edit().is_some())
            && self.controls.is_focused(EDIT);
        if self.controls.is_focused(EDIT)
            && let Some(edit) = self.form.as_mut().and_then(Form::edit_mut)
            && edit.update(input)
        {
            // Text changes invalidate any held confirmation and its old label.
            self.controls.cancel_capture();
            self.feedback.clear();
        }
        let view = self.view(profiles).expect("open profile modal has a view");
        let list_accept = accept && self.form.is_none() && view.rows.iter().any(|row| row.focused);
        let activated = self.controls.update(input, &view.controls());
        if edit_accept || list_accept {
            if list_accept {
                self.selected = view
                    .rows
                    .iter()
                    .find(|row| row.focused)
                    .expect("focused row exists")
                    .row;
            }
            frame.intent = self.submit(profiles);
            return frame;
        }
        let Some(id) = activated else {
            return frame;
        };
        if id == CANCEL {
            self.controls.cancel_capture();
            if self.form.take().is_some() {
                self.feedback.clear();
                self.focus_selected();
            } else {
                self.open = false;
                frame.intent = Some(ProfileIntent::Close);
            }
        } else if id == CONFIRM {
            frame.intent = self.submit(profiles);
        } else if self.form.is_none() {
            match id {
                CREATE => self.begin_form(Form::Create(NameEdit::new(String::new(), false))),
                RENAME | DELETE => {
                    if let ProfileRow::Existing(target) = self.selected
                        && let Some(entry) =
                            profiles.entries.iter().find(|entry| entry.id == target)
                    {
                        self.begin_form(if id == RENAME {
                            Form::Rename {
                                id: target,
                                edit: NameEdit::new(entry.name.clone(), true),
                            }
                        } else {
                            Form::Delete {
                                id: target,
                                name: entry.name.clone(),
                            }
                        });
                    }
                }
                PREVIOUS | NEXT => {
                    self.first_row = if id == PREVIOUS {
                        self.first_row.saturating_sub(VISIBLE_ROWS)
                    } else {
                        (self.first_row + VISIBLE_ROWS)
                            .min(self.known_ids.len().saturating_sub(VISIBLE_ROWS - 1))
                    };
                    self.controls.cancel_capture();
                }
                _ if id >= FIRST_PROFILE => {
                    if let Some(target) = self.known_ids.get((id - FIRST_PROFILE) as usize) {
                        self.selected = ProfileRow::Existing(*target);
                        self.feedback.clear();
                    }
                }
                _ => {}
            }
        }
        frame
    }

    /// Accept only the matching caller result. Failed writes retain the form,
    /// text and target; successful metadata edits return to the chooser.
    pub fn transaction_finished(
        &mut self,
        request: &ProfileRequest,
        result: Result<(), String>,
        profiles: &Profiles,
    ) -> bool {
        if self.pending.as_ref() != Some(request) {
            return false;
        }
        self.pending = None;
        self.controls.cancel_capture();
        match result {
            Err(error) => {
                self.feedback = error.chars().take(1024).collect();
                if self.feedback.is_empty() {
                    self.feedback = "Profile change could not be saved. Try again.".into();
                }
            }
            Ok(()) => {
                self.form = None;
                match request.action {
                    ProfileAction::Create { .. } | ProfileAction::Select { .. } => {
                        self.open = false
                    }
                    ProfileAction::Rename { .. } => self.feedback = "Profile renamed.".into(),
                    ProfileAction::Delete { .. } => {
                        self.selected = profiles
                            .current()
                            .map_or(ProfileRow::Create, |entry| ProfileRow::Existing(entry.id));
                        self.feedback = "Profile deleted.".into();
                    }
                }
                self.synchronize(profiles);
                self.focus_selected();
            }
        }
        true
    }
}
