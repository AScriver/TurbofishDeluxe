//! Transient Store/Fish Setup/Tank Setup adapter over the production VT UI.
//!
//! Parent owns profile/session admission, wall/UI clocks, action dispatch, assets,
//! pet choice, screensaver configuration and persistence. Refresh after every
//! request, before another input. Captured dialog targets are never recaptured.
//! W1-derived presentation qualifications remain in docs/virtual-ui.md.

#[path = "virtual_runtime/render.rs"]
mod render;

pub(super) use render::{BiographyCatalog, ResourceIssue, draw, validate_assets};

use crate::{
    adventure::AdventurePhase,
    sim::Action,
    virtual_tank::PurchaseResult,
    virtual_ui::{
        ButtonStyle, ButtonView, Control, FishController, FishManagementView, FontId,
        PendingDialog, Point, Rect, SetupController, SetupView, StoreController, StoreView,
        UiClock, UiInput, UiIntent, VirtualUiSnapshot,
    },
};

/// Resource IDs are loaded from the owned install by the parent, never bundled.
pub(super) const REQUIRED_IMAGE_IDS: &[&str] = &[
    "IMAGE_SCREENBACK",
    "IMAGE_SCREENTITLE",
    "IMAGE_MAINBUTTON",
    "IMAGE_DIALOG",
    "IMAGE_DIALOGBUTTON",
    "IMAGE_LEFTBUTTON",
    "IMAGE_CENTERBUTTON",
    "IMAGE_RIGHTBUTTON",
    "IMAGE_STORESCREEN",
    "IMAGE_STORELEFTBUTTON",
    "IMAGE_STORETOPBUTTON",
    "IMAGE_STORERIGHTBUTTON",
    "IMAGE_STORELEFTHILIGHT",
    "IMAGE_STORETOPHILIGHT",
    "IMAGE_STORERIGHTHILIGHT",
    "IMAGE_STORESHELL",
    "IMAGE_SPEECHBUBBLE",
    "IMAGE_BUBBULATOR_STORE",
    "IMAGE_AA_STORE",
    "IMAGE_STOREBACKDROPS",
    "IMAGE_FOOD",
    "IMAGE_TANKCHOOSER",
    "IMAGE_TANKBACKDROP1",
    "IMAGE_TANKBACKDROP2",
    "IMAGE_TANKBACKDROP3",
    "IMAGE_TANKBACKDROP4",
    "IMAGE_TANKBACKDROP5",
    "IMAGE_TANKBACKDROP6",
    "IMAGE_FISHBOX",
    "IMAGE_PETBUTTON",
    "IMAGE_PETBUTTONRING",
    "IMAGE_PETBUTTONREFLECT",
    "IMAGE_CHECKED",
    "IMAGE_UNCHECKED",
    "IMAGE_SMALLSWIM",
    "IMAGE_ULTRA",
    "IMAGE_GEKKO",
    "IMAGE_STARCATCHER",
    "IMAGE_GRUBBER",
    "IMAGE_BREEDER",
    "IMAGE_SYLV",
    "IMAGE_MINISYLV",
    "IMAGE_BALLS",
    "IMAGE_BILATERUS",
];

/// Keys and file stems coincide. Insert under these exact Presentation font keys.
pub(super) const REQUIRED_FONT_STEMS: &[&str] = &[
    "JungleFever17outline",
    "JungleFever15outline",
    "JungleFever10outline",
    "ContinuumBold12outline",
    "BlambotPro8",
];
pub(super) const REQUIRED_SOUND_IDS: &[&str] = &["SOUND_BUTTONCLICK", "SOUND_BUY"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Page {
    Store,
    FishSetup,
    TankSetup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Navigation {
    FishSetup,
    TankSetup,
    PetChooser,
    ScreensaverSettings,
    /// A session dialog entered without this adapter's original captured target.
    /// Parent uses its existing safe OpenMenu exit, then refreshes this adapter.
    CloseUnavailableDialog,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UiSound {
    ButtonClick,
    PurchaseComplete,
}

impl UiSound {
    pub(super) const fn resource_id(self) -> &'static str {
        match self {
            Self::ButtonClick => "SOUND_BUTTONCLICK",
            Self::PurchaseComplete => "SOUND_BUY",
        }
    }
}

#[derive(Debug)]
pub(super) enum RequestKind {
    /// Dispatch the complete batch in order through the existing session owner.
    Session {
        intent: UiIntent,
        actions: Vec<Action>,
    },
    Navigate(Navigation),
}

#[derive(Debug)]
pub(super) struct RuntimeRequest {
    pub kind: RequestKind,
    /// Parent plays this once; transaction success feedback has a separate receipt.
    pub sound: Option<UiSound>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DialogIssue {
    MissingCapture,
    PhaseChanged,
}

pub(super) enum PageView {
    Store(StoreView),
    Fish(FishManagementView),
    Setup(SetupView),
}

pub(super) struct NavigationButton {
    pub navigation: Navigation,
    pub button: ButtonView,
}

pub(super) struct RuntimeView {
    pub content: PageView,
    pub navigation: Vec<NavigationButton>,
    pub dialog_issue: Option<DialogIssue>,
}

#[derive(Default)]
struct ChromeInteraction {
    hover: Option<Navigation>,
    down: Option<Navigation>,
}

impl ChromeInteraction {
    fn input(&mut self, input: &UiInput, buttons: &[NavigationButton]) -> Option<Navigation> {
        let hit = |point: Point| {
            buttons
                .iter()
                .rev()
                .find(|entry| entry.button.bounds.contains(point))
        };
        match input {
            UiInput::PointerMove(point) => self.hover = hit(*point).map(|entry| entry.navigation),
            UiInput::PointerDown(point) => {
                self.hover = hit(*point).map(|entry| entry.navigation);
                self.down = hit(*point)
                    .filter(|entry| entry.button.enabled)
                    .map(|entry| entry.navigation);
            }
            UiInput::PointerUp(point) => {
                self.hover = hit(*point).map(|entry| entry.navigation);
                let held = self.down.take()?;
                if hit(*point).is_some_and(|entry| entry.button.enabled && entry.navigation == held)
                {
                    return Some(held);
                }
            }
            UiInput::PointerLeave => *self = Self::default(),
            _ => {}
        }
        None
    }
}

/// One instance per app; all state is discarded when profile or open screen changes.
#[derive(Default)]
pub(super) struct VirtualRuntime {
    profile_id: Option<u64>,
    page: Option<Page>,
    setup_page: Option<Page>,
    store: StoreController,
    fish: FishController,
    setup: SetupController,
    pending: Option<PendingDialog>,
    snapshot: Option<VirtualUiSnapshot>,
    clock: Option<UiClock>,
    day_start_seconds: u64,
    caller_blocked: bool,
    dialog_issue: Option<DialogIssue>,
    request_pending: bool,
    chrome: ChromeInteraction,
}

impl VirtualRuntime {
    /// `snapshot` comes from the real ledger/current Profile wallet/caller time.
    /// Its external_block remains caller-owned. The adapter supplies only its
    /// retained modal target and copies the phase's editable name, never its fish.
    pub(super) fn refresh(
        &mut self,
        profile_id: u64,
        phase: &AdventurePhase,
        mut snapshot: VirtualUiSnapshot,
        clock: UiClock,
        day_start_seconds: u64,
    ) {
        if self.profile_id != Some(profile_id) {
            *self = Self::default();
            self.profile_id = Some(profile_id);
        }
        let page = match phase {
            AdventurePhase::VirtualStore | AdventurePhase::VirtualPurchaseConfirm { .. } => {
                Some(Page::Store)
            }
            AdventurePhase::VirtualSetup => Some(self.setup_page.unwrap_or(Page::TankSetup)),
            AdventurePhase::VirtualRename { .. }
            | AdventurePhase::VirtualFishSaleConfirm { .. } => Some(Page::FishSetup),
            AdventurePhase::VirtualBackdropSaleConfirm => Some(Page::TankSetup),
            _ => None,
        };
        let Some(page) = page else {
            *self = Self::default();
            return;
        };
        if self.page != Some(page) {
            self.change_page(page);
        }
        self.caller_blocked = snapshot.external_block;
        self.dialog_issue = None;
        if is_dialog_phase(phase) {
            match self.pending.as_mut() {
                Some(dialog) if dialog_matches_phase(dialog, phase) => match (dialog, phase) {
                    (
                        PendingDialog::Purchase { name, .. },
                        AdventurePhase::VirtualPurchaseConfirm { name: current, .. },
                    )
                    | (
                        PendingDialog::Rename { name, .. },
                        AdventurePhase::VirtualRename { name: current, .. },
                    ) => name.clone_from(current),
                    _ => {}
                },
                Some(_) => self.dialog_issue = Some(DialogIssue::PhaseChanged),
                None => self.dialog_issue = Some(DialogIssue::MissingCapture),
            }
        } else {
            self.pending = None;
        }
        snapshot.dialog = if self.dialog_issue.is_none() {
            self.pending.clone()
        } else {
            None
        };
        snapshot.external_block |= self.dialog_issue.is_some();
        if !clock.focused
            || snapshot.external_block
            || self
                .snapshot
                .as_ref()
                .is_some_and(|previous| previous.dialog != snapshot.dialog)
        {
            self.chrome = ChromeInteraction::default();
        }
        self.snapshot = Some(snapshot);
        self.clock = Some(clock);
        self.day_start_seconds = day_start_seconds;
        self.request_pending = false;
        // Advance the active controller even during focus loss/external modals;
        // its existing clock policy clears captures and excludes unattended time.
        let _ = self.view();
    }

    pub(super) fn active(&self) -> bool {
        self.page.is_some()
    }
    pub(super) fn page(&self) -> Option<Page> {
        self.page
    }

    pub(super) fn dialog_name(&self) -> Option<&str> {
        match self.pending.as_ref()? {
            PendingDialog::Purchase { name, .. } | PendingDialog::Rename { name, .. } => Some(name),
            _ => None,
        }
    }

    pub(super) fn view(&mut self) -> Option<RuntimeView> {
        let snapshot = self.snapshot.as_ref()?;
        let clock = self.clock?;
        let mut content = match self.page? {
            Page::Store => PageView::Store(self.store.view(snapshot, clock)),
            Page::FishSetup => PageView::Fish(self.fish.view(snapshot, clock)),
            Page::TankSetup => PageView::Setup(self.setup.view(snapshot, clock)),
        };
        // Production day validation lives in session_actions; make its disabled
        // state visible as well, without replacing the original OfferTarget.
        if matches!(snapshot.dialog, Some(PendingDialog::Purchase { ref target, .. }) if target.cycle != self.day_start_seconds)
            && let PageView::Store(view) = &mut content
            && let Some(dialog) = &mut view.dialog
        {
            dialog.stale = true;
            dialog.guidance = "Store day changed. Cancel and choose again.".to_owned();
            dialog.buttons[0].available = false;
            dialog.buttons[0].enabled = false;
        }
        Some(RuntimeView {
            navigation: self.navigation_buttons(&content),
            content,
            dialog_issue: self.dialog_issue,
        })
    }

    pub(super) fn input(&mut self, input: &UiInput) -> Option<RuntimeRequest> {
        if self.request_pending {
            return None;
        }
        let clock = self.clock?;
        if !clock.focused || self.caller_blocked {
            self.chrome = ChromeInteraction::default();
            return None;
        }
        if self.dialog_issue.is_some() && matches!(input, UiInput::Cancel | UiInput::Submit) {
            return self.navigate(Navigation::CloseUnavailableDialog);
        }
        let navigation = self.view()?.navigation;
        if let Some(target) = self.chrome.input(input, &navigation) {
            return self.navigate(target);
        }
        if self.dialog_issue.is_some() {
            return None;
        }
        let snapshot = self.snapshot.as_ref()?;
        let intent = match self.page? {
            Page::Store => self.store.input(snapshot, clock, input),
            Page::FishSetup => self.fish.input(snapshot, clock, input),
            Page::TankSetup => self.setup.input(snapshot, clock, input),
        }?;
        let actions = intent.session_actions(snapshot, self.day_start_seconds)?;
        // Capture only from the original intention. Admission/rejection is read
        // back from the actual session phase on the next refresh.
        match &intent {
            UiIntent::OfferPurchase(target) => {
                self.pending = Some(PendingDialog::Purchase {
                    target: target.clone(),
                    name: String::new(),
                })
            }
            UiIntent::BeginRename(target) => {
                self.pending = Some(PendingDialog::Rename {
                    target: target.clone(),
                    name: target.0.name.clone(),
                })
            }
            UiIntent::BeginFishSale(target) => {
                self.pending = Some(PendingDialog::FishSale {
                    target: target.clone(),
                })
            }
            UiIntent::BeginBackdropSale { index } => {
                self.pending = Some(PendingDialog::BackdropSale { index: *index })
            }
            _ => {}
        }
        let sound = (!matches!(
            intent,
            UiIntent::SetDialogName(_)
                | UiIntent::ConfirmPurchase { .. }
                | UiIntent::ConfirmRename { .. }
                | UiIntent::ConfirmFishSale(_)
                | UiIntent::ConfirmBackdropSale { .. }
        ))
        .then_some(UiSound::ButtonClick);
        self.request_pending = true;
        Some(RuntimeRequest {
            kind: RequestKind::Session { intent, actions },
            sound,
        })
    }

    /// Keyboard navigation uses exactly the same visible/modal/focus predicates.
    /// Only Fish/Tank subpages change here; other routes are requests to parent.
    pub(super) fn navigate(&mut self, target: Navigation) -> Option<RuntimeRequest> {
        if self.request_pending {
            return None;
        }
        if !self
            .view()?
            .navigation
            .iter()
            .any(|entry| entry.navigation == target && entry.button.enabled)
        {
            return None;
        }
        match target {
            Navigation::FishSetup => self.change_page(Page::FishSetup),
            Navigation::TankSetup => self.change_page(Page::TankSetup),
            Navigation::PetChooser
            | Navigation::ScreensaverSettings
            | Navigation::CloseUnavailableDialog => self.chrome = ChromeInteraction::default(),
        }
        self.request_pending = true;
        Some(RuntimeRequest {
            kind: RequestKind::Navigate(target),
            sound: Some(UiSound::ButtonClick),
        })
    }

    /// Call only with the actual successful production receipt. Cancelled
    /// transactions generate neither feedback nor a purchase-complete sound.
    pub(super) fn purchase_result(&mut self, result: &PurchaseResult) -> Option<UiSound> {
        self.store.purchase_result(result);
        (!matches!(result, PurchaseResult::Cancelled)).then_some(UiSound::PurchaseComplete)
    }

    fn change_page(&mut self, page: Page) {
        match self.page {
            Some(Page::Store) => self.store = StoreController::default(),
            Some(Page::FishSetup) => self.fish = FishController::default(),
            Some(Page::TankSetup) => self.setup = SetupController::default(),
            None => {}
        }
        self.page = Some(page);
        if page != Page::Store {
            self.setup_page = Some(page);
        }
        self.chrome = ChromeInteraction::default();
    }

    fn navigation_buttons(&self, content: &PageView) -> Vec<NavigationButton> {
        let Some(page) = self.page else {
            return Vec::new();
        };
        let enabled = self.clock.is_some_and(|clock| clock.focused)
            && !self.caller_blocked
            && self.pending.is_none()
            && self.dialog_issue.is_none();
        let entries = if self.dialog_issue.is_some() {
            vec![(
                Navigation::CloseUnavailableDialog,
                "Cancel",
                Rect::new(245.0, 310.0, 150.0, 38.0),
            )]
        } else {
            match page {
                Page::Store => Vec::new(),
                Page::FishSetup => vec![(
                    Navigation::TankSetup,
                    "Tank Setup",
                    Rect::new(8.0, 4.0, 135.0, 30.0),
                )],
                Page::TankSetup => {
                    let PageView::Setup(setup) = content else {
                        return Vec::new();
                    };
                    vec![
                        (
                            Navigation::FishSetup,
                            "Fish Setup",
                            Rect::new(8.0, 4.0, 135.0, 30.0),
                        ),
                        (
                            Navigation::PetChooser,
                            "Pets",
                            Rect::new(498.0, 4.0, 130.0, 30.0),
                        ),
                        (
                            Navigation::ScreensaverSettings,
                            "Screensaver",
                            setup.screensaver_entry_bounds,
                        ),
                    ]
                }
            }
        };
        entries
            .into_iter()
            .map(|(navigation, text, bounds)| {
                let enabled = if navigation == Navigation::CloseUnavailableDialog {
                    self.clock.is_some_and(|clock| clock.focused) && !self.caller_blocked
                } else {
                    enabled
                };
                NavigationButton {
                    navigation,
                    button: ButtonView {
                        // Navigation has its own typed routing, never this placeholder control.
                        control: Control::ReturnToTank,
                        bounds,
                        text: text.to_owned(),
                        font: FontId::Body,
                        available: true,
                        enabled,
                        hovered: enabled && self.chrome.hover == Some(navigation),
                        pressed: enabled
                            && self.chrome.down == Some(navigation)
                            && self.chrome.hover == Some(navigation),
                        selected: false,
                        style: ButtonStyle::Dialog,
                        checked: None,
                    },
                }
            })
            .collect()
    }
}

fn is_dialog_phase(phase: &AdventurePhase) -> bool {
    matches!(
        phase,
        AdventurePhase::VirtualPurchaseConfirm { .. }
            | AdventurePhase::VirtualRename { .. }
            | AdventurePhase::VirtualFishSaleConfirm { .. }
            | AdventurePhase::VirtualBackdropSaleConfirm
    )
}

fn dialog_matches_phase(dialog: &PendingDialog, phase: &AdventurePhase) -> bool {
    match (dialog, phase) {
        (
            PendingDialog::Purchase { target, .. },
            AdventurePhase::VirtualPurchaseConfirm {
                cycle, slot, price, ..
            },
        ) => target.cycle == *cycle && target.slot == *slot && target.product.price() == *price,
        (PendingDialog::Rename { target, .. }, AdventurePhase::VirtualRename { id, .. })
        | (PendingDialog::FishSale { target }, AdventurePhase::VirtualFishSaleConfirm { id }) => {
            target.0.id == *id
        }
        (PendingDialog::BackdropSale { .. }, AdventurePhase::VirtualBackdropSaleConfirm) => true,
        _ => false,
    }
}
