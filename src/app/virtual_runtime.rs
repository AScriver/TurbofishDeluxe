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
    /// Capture from the real ledger/current Profile wallet/caller time only when
    /// this adapter owns an active page. Inactive phases still clear all state.
    /// Its external_block remains caller-owned. The adapter supplies only its
    /// retained modal target and copies the phase's editable name, never its fish.
    pub(super) fn refresh(
        &mut self,
        profile_id: u64,
        phase: &AdventurePhase,
        capture_snapshot: impl FnOnce() -> VirtualUiSnapshot,
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
        let mut snapshot = capture_snapshot();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        install::sha256,
        virtual_tank::{
            FishBiography, FishOffer, FishProperties, FishSize, PurchaseConfirmation, StoreProduct,
            VirtualFishKind, VirtualTankState,
        },
    };
    use std::{cell::Cell, hint::black_box, time::Instant};

    const DAY: u64 = 86_400;
    const NOW: u64 = DAY + 100;

    fn fixture(count: u16) -> VirtualTankState {
        let mut tank = VirtualTankState::default();
        let mut offers = std::array::from_fn(|_| None);
        offers[2] = Some(StoreProduct::Bubbulator);
        offers[3] = Some(StoreProduct::Fish(FishOffer {
            kind: VirtualFishKind::Guppy,
            properties: FishProperties::default(),
            initial_hunger: 400,
            initial_size: FishSize::Small,
            food_needed_to_grow: 12,
            coin_threshold: 1000,
        }));
        tank.install_stock(DAY, offers).unwrap();
        let mut shells = 1_000_000;
        for id in 0..count {
            tank.stock.as_mut().unwrap().bought[3] = false;
            let result = tank
                .confirm_purchase(
                    &mut shells,
                    PurchaseConfirmation {
                        cycle: DAY,
                        slot: 3,
                        expected_price: 25,
                        accepted: true,
                        fish_name: Some(format!("Fixture fish {id}")),
                        fish_biography: Some(FishBiography {
                            hometown: 1,
                            likes: [1, 2, 3],
                        }),
                    },
                    NOW,
                    DAY,
                )
                .unwrap();
            assert_eq!(result, PurchaseResult::Fish { id, price: 25 });
            if id == 9 && count > 10 {
                tank.confirm_purchase(
                    &mut shells,
                    PurchaseConfirmation {
                        cycle: DAY,
                        slot: 2,
                        expected_price: 20_000,
                        accepted: true,
                        fish_name: None,
                        fish_biography: None,
                    },
                    NOW,
                    DAY,
                )
                .unwrap();
            }
        }
        tank.stock.as_mut().unwrap().bought[3] = false;
        if count != 0 {
            tank.select_fish(0).unwrap();
        }
        tank.validate().unwrap();
        tank
    }

    fn clock(ticks: u64) -> UiClock {
        UiClock {
            ticks,
            focused: true,
        }
    }

    fn refresh(runtime: &mut VirtualRuntime, tank: &VirtualTankState, phase: &AdventurePhase) {
        runtime.refresh(
            1,
            phase,
            || VirtualUiSnapshot::capture(tank, 100_000, NOW, true),
            clock(10),
            DAY,
        );
    }

    /// Recreates the old caller's eager capture followed by the same routing.
    /// The black box keeps discarded capture work observable in optimized tests.
    fn refresh_policy(
        runtime: &mut VirtualRuntime,
        tank: &VirtualTankState,
        phase: &AdventurePhase,
        eager: bool,
        captures: &Cell<u64>,
    ) {
        let capture = || {
            captures.set(captures.get() + 1);
            black_box(VirtualUiSnapshot::capture(
                black_box(tank),
                100_000,
                NOW,
                true,
            ))
        };
        if eager {
            let snapshot = capture();
            runtime.refresh(1, phase, || snapshot, clock(10), DAY);
        } else {
            runtime.refresh(1, phase, capture, clock(10), DAY);
        }
    }

    // Includes all retained controller/capture/clock state and the complete views.
    // Debug strings compare values exactly; hashes are only portable log receipts.
    fn state(runtime: &mut VirtualRuntime) -> String {
        let view = runtime.view().map(|view| {
            let content = match view.content {
                PageView::Store(view) => format!("{view:?}"),
                PageView::Fish(view) => format!("{view:?}"),
                PageView::Setup(view) => format!("{view:?}"),
            };
            (
                content,
                view.navigation
                    .into_iter()
                    .map(|button| (button.navigation, button.button))
                    .collect::<Vec<_>>(),
                view.dialog_issue,
            )
        });
        format!(
            "{:?}",
            (
                runtime.profile_id,
                runtime.page,
                runtime.setup_page,
                (&runtime.store, &runtime.fish, &runtime.setup),
                &runtime.pending,
                &runtime.snapshot,
                runtime.clock,
                runtime.day_start_seconds,
                (
                    runtime.caller_blocked,
                    runtime.dialog_issue,
                    runtime.request_pending
                ),
                (runtime.chrome.hover, runtime.chrome.down),
                view
            )
        )
    }

    #[test]
    fn inactive_refresh_skips_capture_and_clears_stale_controllers() {
        let tank = fixture(1);
        for phase in [
            AdventurePhase::Playing,
            AdventurePhase::TimeTrialPlaying,
            AdventurePhase::ChallengePlaying,
            AdventurePhase::SandboxPlaying,
            AdventurePhase::GameSelector,
            AdventurePhase::HelpScreen,
            AdventurePhase::VirtualTank,
            AdventurePhase::VirtualPrestoDialog { pressed: None },
        ] {
            let mut runtime = VirtualRuntime::default();
            refresh(&mut runtime, &tank, &AdventurePhase::VirtualStore);
            assert!(
                runtime
                    .input(&UiInput::Activate(Control::StoreSlot(3)))
                    .is_some()
            );
            runtime.purchase_result(&PurchaseResult::Accessory { price: 20_000 });
            runtime.setup_page = Some(Page::FishSetup);
            runtime.chrome.down = Some(Navigation::FishSetup);
            runtime.refresh(
                1,
                &phase,
                || panic!("inactive phase captured a tank: {phase:?}"),
                clock(20),
                DAY,
            );
            assert_eq!(state(&mut runtime), state(&mut VirtualRuntime::default()));
            assert!(runtime.input(&UiInput::Submit).is_none());
        }
    }

    #[test]
    fn active_pages_capture_once_and_refresh_rejected_requests_before_input() {
        let mut tank = fixture(1);
        let captures = Cell::new(0);
        for phase in [
            AdventurePhase::VirtualStore,
            AdventurePhase::VirtualSetup,
            AdventurePhase::VirtualPurchaseConfirm {
                cycle: DAY,
                slot: 3,
                price: 25,
                name: "Pearl".into(),
            },
            AdventurePhase::VirtualRename {
                id: 0,
                name: "Pearl".into(),
            },
            AdventurePhase::VirtualFishSaleConfirm { id: 0 },
            AdventurePhase::VirtualBackdropSaleConfirm,
        ] {
            let mut runtime = VirtualRuntime::default();
            let before = captures.get();
            refresh_policy(&mut runtime, &tank, &phase, false, &captures);
            assert!(runtime.active());
            assert_eq!(captures.get(), before + 1);
        }
        let mut runtime = VirtualRuntime::default();
        runtime.refresh(
            1,
            &AdventurePhase::VirtualStore,
            || VirtualUiSnapshot::capture(&tank, 24, NOW, true),
            clock(0),
            DAY,
        );
        assert!(
            runtime
                .input(&UiInput::Activate(Control::StoreSlot(3)))
                .is_none()
        );
        refresh(&mut runtime, &tank, &AdventurePhase::VirtualStore);
        assert!(
            runtime
                .input(&UiInput::Activate(Control::StoreSlot(3)))
                .is_some()
        );
        assert!(runtime.input(&UiInput::Submit).is_none()); // Dispatch is still pending.
        tank.stock.as_mut().unwrap().cycle += 1;
        refresh(&mut runtime, &tank, &AdventurePhase::VirtualStore); // Rejected request readback.
        assert!(runtime.pending.is_none());
        let request = runtime
            .input(&UiInput::Activate(Control::StoreSlot(3)))
            .unwrap();
        assert!(
            matches!(request.kind, RequestKind::Session { intent: UiIntent::OfferPurchase(target), .. } if target.cycle == DAY + 1)
        );
    }

    #[test]
    fn modal_targets_survive_fresh_snapshots_but_not_profile_changes() {
        let mut tank = fixture(1);
        let original = tank.fish[0].clone();
        let mut runtime = VirtualRuntime::default();
        refresh(&mut runtime, &tank, &AdventurePhase::VirtualSetup);
        assert!(runtime.navigate(Navigation::FishSetup).is_some());
        refresh(&mut runtime, &tank, &AdventurePhase::VirtualSetup);
        assert_eq!(runtime.page(), Some(Page::FishSetup));
        assert!(
            runtime
                .input(&UiInput::Activate(Control::RenameSelected))
                .is_some()
        );
        let phase = AdventurePhase::VirtualRename {
            id: 0,
            name: "Pearl".into(),
        };
        refresh(&mut runtime, &tank, &phase);
        assert_eq!(runtime.dialog_name(), Some("Pearl"));
        tank.rename_fish(0, "Replacement").unwrap();
        refresh(&mut runtime, &tank, &phase);
        assert!(
            matches!(runtime.pending.as_ref(), Some(PendingDialog::Rename { target, .. }) if target.0 == original)
        );
        assert_eq!(runtime.snapshot.as_ref().unwrap().tank(), &tank);
        assert!(runtime.input(&UiInput::Submit).is_none()); // No silent retarget to replacement.
        runtime.refresh(
            2,
            &phase,
            || VirtualUiSnapshot::capture(&tank, 100_000, NOW, true),
            clock(11),
            DAY,
        );
        assert!(runtime.pending.is_none());
        assert_eq!(runtime.setup_page, Some(Page::FishSetup));
        assert_eq!(runtime.dialog_issue, Some(DialogIssue::MissingCapture));
        assert!(matches!(
            runtime.input(&UiInput::Cancel).unwrap().kind,
            RequestKind::Navigate(Navigation::CloseUnavailableDialog)
        ));
        runtime.refresh(
            3,
            &AdventurePhase::VirtualSetup,
            || VirtualUiSnapshot::capture(&tank, 100_000, NOW, true),
            clock(12),
            DAY,
        );
        assert_eq!(runtime.page(), Some(Page::TankSetup));
    }

    #[test]
    fn refreshed_focus_and_external_modal_cancel_pointer_captures() {
        let tank = fixture(1);
        let point = Point { x: 210.0, y: 60.0 };
        for focused in [false, true] {
            let mut runtime = VirtualRuntime::default();
            refresh(&mut runtime, &tank, &AdventurePhase::VirtualStore);
            runtime.input(&UiInput::PointerDown(point));
            runtime.refresh(
                1,
                &AdventurePhase::VirtualStore,
                || {
                    let mut snapshot = VirtualUiSnapshot::capture(&tank, 100_000, NOW + 1, true);
                    snapshot.external_block = focused;
                    snapshot
                },
                UiClock { ticks: 20, focused },
                DAY,
            );
            assert!(runtime.input(&UiInput::PointerUp(point)).is_none());
            refresh(&mut runtime, &tank, &AdventurePhase::VirtualStore);
            assert!(runtime.input(&UiInput::PointerUp(point)).is_none());
            assert!(
                runtime
                    .input(&UiInput::Activate(Control::StoreSlot(3)))
                    .is_some()
            );
        }
    }

    #[test]
    fn eager_and_lazy_refresh_preserve_complete_state_views_and_requests() {
        let mut tank = fixture(1);
        let original = tank.clone();
        let mut eager = VirtualRuntime::default();
        let mut lazy = VirtualRuntime::default();
        let eager_captures = Cell::new(0);
        let lazy_captures = Cell::new(0);
        let phases = [
            AdventurePhase::Playing,
            AdventurePhase::VirtualStore,
            AdventurePhase::VirtualPurchaseConfirm {
                cycle: DAY,
                slot: 3,
                price: 25,
                name: "Pearl".into(),
            },
            AdventurePhase::VirtualPurchaseConfirm {
                cycle: DAY,
                slot: 3,
                price: 25,
                name: "Pearl".into(),
            },
            AdventurePhase::VirtualSetup,
            AdventurePhase::VirtualSetup,
            AdventurePhase::VirtualRename {
                id: 0,
                name: "Pearl".into(),
            },
            AdventurePhase::TimeTrialPlaying,
        ];
        let inputs = [
            UiInput::Submit,
            UiInput::Activate(Control::StoreSlot(3)),
            UiInput::EditName("Pearl".into()),
            UiInput::Submit,
            UiInput::PointerLeave,
            UiInput::Activate(Control::RenameSelected),
            UiInput::Submit,
            UiInput::Cancel,
        ];
        for (index, (phase, input)) in phases.iter().zip(&inputs).enumerate() {
            if index == 3 {
                tank.stock.as_mut().unwrap().cycle += 1; // Stale captured purchase.
            }
            if index == 6 {
                tank.rename_fish(0, "Replacement").unwrap(); // Stale captured fish.
            }
            refresh_policy(&mut eager, &tank, phase, true, &eager_captures);
            refresh_policy(&mut lazy, &tank, phase, false, &lazy_captures);
            assert_eq!(state(&mut eager), state(&mut lazy), "refresh {index}");
            if index == 4 {
                assert_eq!(
                    format!("{:?}", eager.navigate(Navigation::FishSetup)),
                    format!("{:?}", lazy.navigate(Navigation::FishSetup))
                );
            } else {
                assert_eq!(
                    format!("{:?}", eager.input(input)),
                    format!("{:?}", lazy.input(input)),
                    "input {index}"
                );
            }
            assert_eq!(state(&mut eager), state(&mut lazy), "request {index}");
        }
        assert_eq!(eager_captures.get(), 8);
        assert_eq!(lazy_captures.get(), 6);
        assert_ne!(tank, original); // Only the explicit fixture mutations above changed it.
        println!(
            "A293 equivalence state_sha256={}",
            sha256(state(&mut lazy).as_bytes())
        );
    }

    #[test]
    #[ignore = "offline timing only; run with the coordinator's serial timing lease"]
    fn measure_inactive_refresh_snapshot_work() {
        const ITERATIONS: u64 = 20_000;
        const SAMPLES: usize = 9;
        for count in [1, 20] {
            let tank = fixture(count);
            let input = serde_json::to_vec(&tank).unwrap();
            for phase in [
                AdventurePhase::Playing,
                AdventurePhase::TimeTrialPlaying,
                AdventurePhase::VirtualSetup,
            ] {
                let mut samples = [Vec::new(), Vec::new()];
                for sample in 0..SAMPLES {
                    // Alternate first policy to reduce systematic ordering bias.
                    for policy in [sample % 2, 1 - sample % 2] {
                        let eager = policy == 0;
                        let mut runtime = VirtualRuntime::default();
                        let captures = Cell::new(0);
                        for _ in 0..100 {
                            refresh_policy(&mut runtime, &tank, &phase, eager, &captures);
                        }
                        captures.set(0);
                        let start = Instant::now();
                        for _ in 0..ITERATIONS {
                            // The ordinary frame has a pre-input and pre-draw refresh.
                            refresh_policy(
                                black_box(&mut runtime),
                                &tank,
                                black_box(&phase),
                                eager,
                                &captures,
                            );
                            refresh_policy(
                                black_box(&mut runtime),
                                &tank,
                                black_box(&phase),
                                eager,
                                &captures,
                            );
                        }
                        let elapsed =
                            start.elapsed().as_secs_f64() * 1_000_000.0 / ITERATIONS as f64;
                        samples[policy].push(elapsed);
                        assert_eq!(
                            captures.get(),
                            if eager || runtime.active() {
                                ITERATIONS * 2
                            } else {
                                0
                            }
                        );
                    }
                }
                let mut eager = VirtualRuntime::default();
                let mut lazy = VirtualRuntime::default();
                let captures = Cell::new(0);
                refresh_policy(&mut eager, &tank, &phase, true, &captures);
                refresh_policy(&mut lazy, &tank, &phase, false, &captures);
                assert_eq!(state(&mut eager), state(&mut lazy));
                assert_eq!(serde_json::to_vec(&tank).unwrap(), input);
                println!(
                    "A293 {}",
                    serde_json::json!({
                        "fish": count, "phase": phase, "iterations": ITERATIONS, "samples": SAMPLES,
                        "input_sha256": sha256(&input), "state_sha256": sha256(state(&mut lazy).as_bytes()),
                        "eager_us_per_pair": samples[0], "lazy_us_per_pair": samples[1],
                        "eager_captures_per_pair": 2, "lazy_captures_per_pair": if lazy.active() { 2 } else { 0 },
                    })
                );
            }
        }
    }
}
