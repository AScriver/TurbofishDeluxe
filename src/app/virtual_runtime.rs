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

struct NavigationEntry {
    navigation: Navigation,
    text: &'static str,
    bounds: Rect,
}

const CLOSE_NAVIGATION: [NavigationEntry; 1] = [NavigationEntry {
    navigation: Navigation::CloseUnavailableDialog,
    text: "Cancel",
    bounds: Rect::new(245.0, 310.0, 150.0, 38.0),
}];
const FISH_NAVIGATION: [NavigationEntry; 1] = [NavigationEntry {
    navigation: Navigation::TankSetup,
    text: "Tank Setup",
    bounds: Rect::new(8.0, 4.0, 135.0, 30.0),
}];
const TANK_NAVIGATION: [NavigationEntry; 3] = [
    NavigationEntry {
        navigation: Navigation::FishSetup,
        text: "Fish Setup",
        bounds: Rect::new(8.0, 4.0, 135.0, 30.0),
    },
    NavigationEntry {
        navigation: Navigation::PetChooser,
        text: "Pets",
        bounds: Rect::new(498.0, 4.0, 130.0, 30.0),
    },
    NavigationEntry {
        navigation: Navigation::ScreensaverSettings,
        text: "Screensaver",
        bounds: SetupController::SCREENSAVER_ENTRY_BOUNDS,
    },
];

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
    fn input(
        &mut self,
        input: &UiInput,
        buttons: &[NavigationEntry],
        enabled: bool,
    ) -> Option<Navigation> {
        let hit = |point: Point| {
            buttons
                .iter()
                .rev()
                .find(|entry| entry.bounds.contains(point))
        };
        match input {
            UiInput::PointerMove(point) => self.hover = hit(*point).map(|entry| entry.navigation),
            UiInput::PointerDown(point) => {
                self.hover = hit(*point).map(|entry| entry.navigation);
                self.down = hit(*point)
                    .filter(|_| enabled)
                    .map(|entry| entry.navigation);
            }
            UiInput::PointerUp(point) => {
                self.hover = hit(*point).map(|entry| entry.navigation);
                let held = self.down.take()?;
                if hit(*point).is_some_and(|entry| enabled && entry.navigation == held) {
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
        let _ = self.advance();
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

    fn advance(&mut self) -> Option<()> {
        let snapshot = self.snapshot.as_ref()?;
        let clock = self.clock?;
        match self.page? {
            Page::Store => self.store.advance(snapshot, clock),
            Page::FishSetup => self.fish.advance(snapshot, clock),
            Page::TankSetup => self.setup.advance(snapshot, clock),
        }
        Some(())
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
            navigation: self.navigation_buttons(),
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
        self.advance()?;
        let navigation = self.navigation_entries();
        let enabled = self.navigation_enabled();
        if let Some(target) = self.chrome.input(input, navigation, enabled) {
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
        self.advance()?;
        if !self.navigation_enabled()
            || !self
                .navigation_entries()
                .iter()
                .any(|entry| entry.navigation == target)
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

    fn navigation_entries(&self) -> &'static [NavigationEntry] {
        let Some(page) = self.page else {
            return &[];
        };
        if self.dialog_issue.is_some() {
            &CLOSE_NAVIGATION
        } else {
            match page {
                Page::Store => &[],
                Page::FishSetup => &FISH_NAVIGATION,
                Page::TankSetup => &TANK_NAVIGATION,
            }
        }
    }

    fn navigation_enabled(&self) -> bool {
        self.clock.is_some_and(|clock| clock.focused)
            && !self.caller_blocked
            && (self.dialog_issue.is_some() || self.pending.is_none())
    }

    fn navigation_buttons(&self) -> Vec<NavigationButton> {
        let enabled = self.navigation_enabled();
        self.navigation_entries()
            .iter()
            .map(|entry| {
                let mut button = NavigationButton {
                    navigation: entry.navigation,
                    button: ButtonView {
                        // Navigation has its own typed routing, never this placeholder control.
                        control: Control::ReturnToTank,
                        bounds: entry.bounds,
                        text: entry.text.to_owned(),
                        font: FontId::Body,
                        available: true,
                        enabled,
                        hovered: false,
                        pressed: false,
                        selected: false,
                        style: ButtonStyle::Dialog,
                        checked: None,
                    },
                };
                button.button.hovered = enabled && self.chrome.hover == Some(button.navigation);
                button.button.pressed =
                    enabled && self.chrome.down == Some(button.navigation) && button.button.hovered;
                button
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
    fn measure_active_refresh_input_views() {
        const ITERATIONS: u64 = 20_000;
        for count in [1, 20] {
            let tank = fixture(count);
            let bytes = serde_json::to_vec(&tank).unwrap();
            for page in [Page::Store, Page::FishSetup, Page::TankSetup] {
                let phase = if page == Page::Store {
                    AdventurePhase::VirtualStore
                } else {
                    AdventurePhase::VirtualSetup
                };
                let mut runtime = VirtualRuntime {
                    profile_id: Some(1),
                    setup_page: Some(page),
                    ..VirtualRuntime::default()
                };
                let frame = |runtime: &mut VirtualRuntime, ticks| {
                    let capture =
                        || VirtualUiSnapshot::capture(black_box(&tank), 100_000, NOW, true);
                    runtime.refresh(1, black_box(&phase), capture, clock(ticks), DAY);
                    assert!(
                        black_box(runtime.input(black_box(&UiInput::PointerMove(Point {
                            x: 320.0,
                            y: 470.0,
                        }))))
                        .is_none()
                    );
                    runtime.refresh(1, black_box(&phase), capture, clock(ticks), DAY);
                    drop(black_box(runtime.view().unwrap()));
                };
                for ticks in 0..100 {
                    frame(&mut runtime, ticks);
                }
                let mut samples = Vec::new();
                for sample in 0..9 {
                    let start = Instant::now();
                    for offset in 0..ITERATIONS {
                        frame(black_box(&mut runtime), 100 + sample * ITERATIONS + offset);
                    }
                    samples.push(start.elapsed().as_secs_f64() * 1_000_000.0 / ITERATIONS as f64);
                }
                assert_eq!(bytes, serde_json::to_vec(&tank).unwrap());
                println!(
                    "A297 timing fish={count} page={page:?} samples_us={samples:?} fixture_sha256={} state_sha256={}",
                    sha256(&bytes),
                    sha256(state(&mut runtime).as_bytes())
                );
            }
        }
    }

    fn refresh_clock(
        runtime: &mut VirtualRuntime,
        tank: &VirtualTankState,
        phase: &AdventurePhase,
        clock: UiClock,
        blocked: bool,
    ) {
        runtime.refresh(
            1,
            phase,
            || {
                let mut snapshot = VirtualUiSnapshot::capture(tank, 100_000, NOW, true);
                snapshot.external_block = blocked;
                snapshot
            },
            clock,
            DAY,
        );
    }

    #[test]
    fn active_refresh_preserves_finance_and_feedback_clocks() {
        let tank = fixture(1);
        let mut runtime = VirtualRuntime::default();
        let phase = AdventurePhase::VirtualSetup;
        refresh_clock(&mut runtime, &tank, &phase, clock(0), false);
        runtime.navigate(Navigation::FishSetup).unwrap();
        refresh_clock(&mut runtime, &tank, &phase, clock(0), false);
        runtime.input(&UiInput::PointerMove(Point { x: 360.0, y: 400.0 }));
        refresh_clock(&mut runtime, &tank, &phase, clock(4), false);
        let finance = |runtime: &mut VirtualRuntime| {
            let PageView::Fish(view) = runtime.view().unwrap().content else {
                panic!("Fish page")
            };
            view.selected.unwrap().finance_alpha
        };
        assert_eq!(finance(&mut runtime), 0.5); // Explicit eight-tick fade contract.
        assert_eq!(finance(&mut runtime), 0.5); // Repeated same-clock draw is idempotent.
        refresh_clock(
            &mut runtime,
            &tank,
            &phase,
            UiClock {
                ticks: 100,
                focused: false,
            },
            false,
        );
        assert_eq!(finance(&mut runtime), 0.5);
        refresh_clock(&mut runtime, &tank, &phase, clock(120), false);
        assert_eq!(finance(&mut runtime), 0.5); // Unattended time is excluded.
        refresh_clock(&mut runtime, &tank, &phase, clock(124), false);
        assert_eq!(finance(&mut runtime), 0.0); // Focus loss cleared hover.

        refresh_clock(
            &mut runtime,
            &tank,
            &AdventurePhase::VirtualStore,
            clock(0),
            false,
        );
        runtime.purchase_result(&PurchaseResult::Accessory { price: 20_000 });
        refresh_clock(
            &mut runtime,
            &tank,
            &AdventurePhase::VirtualStore,
            clock(105),
            false,
        );
        let speech = |runtime: &mut VirtualRuntime| {
            let PageView::Store(view) = runtime.view().unwrap().content else {
                panic!("Store page")
            };
            view.speech
        };
        assert_eq!(
            speech(&mut runtime).as_deref(),
            Some("Purchase complete. Your tank inventory is updated.")
        );
        runtime.input(&UiInput::PointerMove(Point { x: 210.0, y: 60.0 }));
        assert!(speech(&mut runtime).is_none()); // 108 - 105 elapsed - 3 pointer movement.
        runtime.input(&UiInput::PointerMove(Point { x: 210.0, y: 60.0 }));
        refresh_clock(
            &mut runtime,
            &tank,
            &AdventurePhase::VirtualStore,
            clock(120),
            false,
        );
        assert!(speech(&mut runtime).is_none()); // Guidance waits strictly more than 15 ticks.
        refresh_clock(
            &mut runtime,
            &tank,
            &AdventurePhase::VirtualStore,
            clock(121),
            false,
        );
        assert!(speech(&mut runtime).is_some());
        assert_eq!(tank, fixture(1));
    }

    #[test]
    fn active_purchase_day_rejects_original_dialog_without_retargeting() {
        let tank = fixture(1);
        let mut runtime = VirtualRuntime::default();
        refresh(&mut runtime, &tank, &AdventurePhase::VirtualStore);
        runtime
            .input(&UiInput::Activate(Control::StoreSlot(3)))
            .unwrap();
        let phase = AdventurePhase::VirtualPurchaseConfirm {
            cycle: DAY,
            slot: 3,
            price: 25,
            name: "Pearl".into(),
        };
        runtime.refresh(
            1,
            &phase,
            || VirtualUiSnapshot::capture(&tank, 100_000, NOW, true),
            clock(11),
            DAY + 1,
        );
        let original = runtime.pending.clone();
        let PageView::Store(view) = runtime.view().unwrap().content else {
            panic!("Store page")
        };
        let dialog = view.dialog.unwrap();
        assert!(dialog.stale);
        assert!(!dialog.buttons[0].enabled);
        assert!(runtime.input(&UiInput::Submit).is_none());
        assert_eq!(runtime.pending, original);
        assert!(matches!(
            runtime.input(&UiInput::Cancel).unwrap().kind,
            RequestKind::Session {
                intent: UiIntent::DismissDialog,
                ..
            }
        ));
    }

    fn trace_input(
        transcript: &mut String,
        runtime: &mut VirtualRuntime,
        label: &str,
        input: &UiInput,
    ) -> Option<RuntimeRequest> {
        transcript.push_str(&format!("{label} before={}\n", state(runtime)));
        let request = runtime.input(input);
        transcript.push_str(&format!(
            "{label} input={input:?} request={request:?} after={}\n",
            state(runtime)
        ));
        request
    }

    #[test]
    fn active_complete_state_view_request_trace() {
        let mut transcript = String::new();
        for count in [1, 20] {
            let mut tank = fixture(count);
            let bytes = serde_json::to_vec(&tank).unwrap();
            transcript.push_str(&format!("fish={count} fixture_sha256={}\n", sha256(&bytes)));
            let mut runtime = VirtualRuntime::default();
            let store = AdventurePhase::VirtualStore;
            let point = Point { x: 210.0, y: 60.0 };
            refresh_clock(&mut runtime, &tank, &store, clock(10), false);
            assert!(
                trace_input(
                    &mut transcript,
                    &mut runtime,
                    "hover",
                    &UiInput::PointerMove(point)
                )
                .is_none()
            );
            assert!(
                trace_input(
                    &mut transcript,
                    &mut runtime,
                    "press",
                    &UiInput::PointerDown(point)
                )
                .is_none()
            );
            refresh_clock(
                &mut runtime,
                &tank,
                &store,
                UiClock {
                    ticks: 20,
                    focused: false,
                },
                false,
            );
            assert!(
                trace_input(
                    &mut transcript,
                    &mut runtime,
                    "lost focus",
                    &UiInput::PointerUp(point)
                )
                .is_none()
            );
            refresh_clock(&mut runtime, &tank, &store, clock(30), false);
            assert!(
                trace_input(
                    &mut transcript,
                    &mut runtime,
                    "restored focus",
                    &UiInput::PointerUp(point)
                )
                .is_none()
            );
            runtime.input(&UiInput::PointerDown(point));
            refresh_clock(&mut runtime, &tank, &store, clock(31), true);
            assert!(
                trace_input(
                    &mut transcript,
                    &mut runtime,
                    "external modal",
                    &UiInput::PointerUp(point)
                )
                .is_none()
            );
            refresh_clock(&mut runtime, &tank, &store, clock(32), false);
            assert!(
                trace_input(
                    &mut transcript,
                    &mut runtime,
                    "modal closed",
                    &UiInput::PointerUp(point)
                )
                .is_none()
            );
            runtime.input(&UiInput::PointerDown(point));
            assert_eq!(
                trace_input(
                    &mut transcript,
                    &mut runtime,
                    "released offer",
                    &UiInput::PointerUp(point)
                )
                .is_some(),
                count == 1 // Full tank keeps its legitimate capacity rejection.
            );
            assert!(
                trace_input(
                    &mut transcript,
                    &mut runtime,
                    "request pending",
                    &UiInput::Submit
                )
                .is_none()
            );
            if count == 1 {
                let purchase = AdventurePhase::VirtualPurchaseConfirm {
                    cycle: DAY,
                    slot: 3,
                    price: 25,
                    name: "Pearl".into(),
                };
                refresh_clock(&mut runtime, &tank, &purchase, clock(33), false);
                assert_eq!(
                    trace_input(
                        &mut transcript,
                        &mut runtime,
                        "edit name",
                        &UiInput::EditName("Jade".into())
                    )
                    .is_some(),
                    count == 1
                );
                refresh_clock(&mut runtime, &tank, &purchase, clock(34), false);
                assert!(
                    trace_input(&mut transcript, &mut runtime, "confirm", &UiInput::Submit)
                        .is_some()
                );
                tank.stock.as_mut().unwrap().cycle += 1;
                refresh_clock(&mut runtime, &tank, &purchase, clock(35), false);
                assert!(
                    trace_input(
                        &mut transcript,
                        &mut runtime,
                        "stale offer",
                        &UiInput::Submit
                    )
                    .is_none()
                );
            }
            refresh_clock(&mut runtime, &tank, &store, clock(36), false);
            runtime.purchase_result(&PurchaseResult::Accessory { price: 20_000 });
            assert!(
                trace_input(
                    &mut transcript,
                    &mut runtime,
                    "purchase feedback",
                    &UiInput::PointerMove(point)
                )
                .is_none()
            );
            let setup = AdventurePhase::VirtualSetup;
            refresh_clock(&mut runtime, &tank, &setup, clock(40), false);
            transcript.push_str(&format!(
                "navigation request={:?}\n",
                runtime.navigate(Navigation::FishSetup)
            ));
            refresh_clock(&mut runtime, &tank, &setup, clock(41), false);
            trace_input(
                &mut transcript,
                &mut runtime,
                "finance hover",
                &UiInput::PointerMove(Point { x: 360.0, y: 400.0 }),
            );
            refresh_clock(&mut runtime, &tank, &setup, clock(45), false);
            trace_input(
                &mut transcript,
                &mut runtime,
                "finance fade",
                &UiInput::PointerLeave,
            );
            assert!(
                trace_input(
                    &mut transcript,
                    &mut runtime,
                    "rename",
                    &UiInput::Activate(Control::RenameSelected)
                )
                .is_some()
            );
            let rename = AdventurePhase::VirtualRename {
                id: 0,
                name: "Pearl".into(),
            };
            tank.rename_fish(0, "Replacement").unwrap();
            refresh_clock(&mut runtime, &tank, &rename, clock(46), false);
            assert!(
                trace_input(
                    &mut transcript,
                    &mut runtime,
                    "stale fish",
                    &UiInput::Submit
                )
                .is_none()
            );
            assert!(
                trace_input(
                    &mut transcript,
                    &mut runtime,
                    "cancel stale fish",
                    &UiInput::Cancel
                )
                .is_some()
            );
            refresh_clock(&mut runtime, &tank, &setup, clock(47), false);
            transcript.push_str(&format!(
                "tank navigation request={:?}\n",
                runtime.navigate(Navigation::TankSetup)
            ));
            refresh_clock(&mut runtime, &tank, &setup, clock(48), false);
            let nav_point = Point { x: 30.0, y: 10.0 };
            trace_input(
                &mut transcript,
                &mut runtime,
                "navigation press",
                &UiInput::PointerDown(nav_point),
            );
            refresh_clock(
                &mut runtime,
                &tank,
                &setup,
                UiClock {
                    ticks: 60,
                    focused: false,
                },
                false,
            );
            assert!(
                trace_input(
                    &mut transcript,
                    &mut runtime,
                    "navigation focus loss",
                    &UiInput::PointerUp(nav_point)
                )
                .is_none()
            );
            refresh_clock(&mut runtime, &tank, &setup, clock(70), false);
            assert!(
                trace_input(
                    &mut transcript,
                    &mut runtime,
                    "navigation released",
                    &UiInput::PointerUp(nav_point)
                )
                .is_none()
            );
            assert_ne!(bytes, serde_json::to_vec(&tank).unwrap()); // Only explicit fixture changes above.
        }
        if let Some(path) = std::env::var_os("TURBOFISH_ACTIVE_TRACE_PATH") {
            std::fs::write(path, &transcript).unwrap();
        }
        println!(
            "A297 complete_trace bytes={} sha256={}",
            transcript.len(),
            sha256(transcript.as_bytes())
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
