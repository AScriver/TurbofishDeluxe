//! Renderer-independent Virtual Tank screens. W1 f919b3c supplies qualified
//! presentation rules; the production ledger owns all values and transactions.
//! See docs/virtual-ui.md for integration, source limits and verification.

#[path = "virtual_ui/adapter.rs"]
mod adapter;
#[path = "virtual_ui/fish.rs"]
mod fish;
#[path = "virtual_ui/setup.rs"]
mod setup;
#[path = "virtual_ui/store.rs"]
mod store;
#[cfg(test)]
#[path = "virtual_ui/tests.rs"]
mod tests;

pub use fish::{FishController, FishDetails, FishManagementView, FishRow};
pub use setup::{SettingView, SetupController, SetupView};
pub use store::{StoreController, StoreSlotView, StoreView};

use crate::virtual_tank::{
    BACKDROP_COUNT, FishProperties, FishSize, StoreProduct, VirtualFish, VirtualFishKind,
    VirtualSettings, VirtualTankState,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(self, point: Point) -> bool {
        point.x >= self.x
            && point.x < self.x + self.width
            && point.y >= self.y
            && point.y < self.y + self.height
    }
}

pub const SCREEN: Rect = Rect::new(0.0, 0.0, 640.0, 480.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Layer {
    Background,
    Control,
    Preview,
    Overlay,
    Modal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontId {
    Title,
    Heading,
    Body,
    Price,
    Speech,
}

impl FontId {
    pub const fn catalog_key(self) -> &'static str {
        match self {
            Self::Title => "JungleFever17outline",
            Self::Heading => "JungleFever15outline",
            Self::Body => "JungleFever10outline",
            Self::Price => "ContinuumBold12outline",
            Self::Speech => "BlambotPro8",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageId {
    ScreenBack,
    ScreenTitle,
    MainButton,
    DialogButton,
    LeftButton,
    CenterButton,
    RightButton,
    StoreScreen,
    StoreLeft,
    StoreTop,
    StoreRight,
    StoreLeftHighlight,
    StoreTopHighlight,
    StoreRightHighlight,
    StoreShell,
    SpeechBubble,
    Bubbulator,
    AlienAttractor,
    StoreBackdrops,
    Food,
    TankChooser,
    Backdrop(u8),
    FishBox,
    FishBoxButton,
    PetButton,
    PetButtonHole,
    PetButtonRing,
    PetButtonReflect,
    Checked,
    Unchecked,
    Guppy,
    Ultra,
    Gekko,
    Penta,
    Grubber,
    Breeder,
    Sylvester,
    MiniSylvester,
    Ball,
    Bi,
}

impl ImageId {
    /// A missing resource stays an explicit renderer error, never a substituted asset.
    pub const fn resource_key(self) -> Option<&'static str> {
        Some(match self {
            Self::ScreenBack => "IMAGE_SCREENBACK",
            Self::ScreenTitle => "IMAGE_SCREENTITLE",
            Self::MainButton => "IMAGE_MAINBUTTON",
            Self::DialogButton => "IMAGE_DIALOGBUTTON",
            Self::LeftButton => "IMAGE_LEFTBUTTON",
            Self::CenterButton => "IMAGE_CENTERBUTTON",
            Self::RightButton => "IMAGE_RIGHTBUTTON",
            Self::StoreScreen => "IMAGE_STORESCREEN",
            Self::StoreLeft => "IMAGE_STORELEFTBUTTON",
            Self::StoreTop => "IMAGE_STORETOPBUTTON",
            Self::StoreRight => "IMAGE_STORERIGHTBUTTON",
            Self::StoreLeftHighlight => "IMAGE_STORELEFTHILIGHT",
            Self::StoreTopHighlight => "IMAGE_STORETOPHILIGHT",
            Self::StoreRightHighlight => "IMAGE_STORERIGHTHILIGHT",
            Self::StoreShell => "IMAGE_STORESHELL",
            Self::SpeechBubble => "IMAGE_SPEECHBUBBLE",
            Self::Bubbulator => "IMAGE_BUBBULATOR_STORE",
            Self::AlienAttractor => "IMAGE_AA_STORE",
            Self::StoreBackdrops => "IMAGE_STOREBACKDROPS",
            Self::Food => "IMAGE_FOOD",
            Self::TankChooser => "IMAGE_TANKCHOOSER",
            Self::Backdrop(0) => "IMAGE_TANKBACKDROP1",
            Self::Backdrop(1) => "IMAGE_TANKBACKDROP2",
            Self::Backdrop(2) => "IMAGE_TANKBACKDROP3",
            Self::Backdrop(3) => "IMAGE_TANKBACKDROP4",
            Self::Backdrop(4) => "IMAGE_TANKBACKDROP5",
            Self::Backdrop(5) => "IMAGE_TANKBACKDROP6",
            Self::Backdrop(_) => return None,
            Self::FishBox => "IMAGE_FISHBOX",
            Self::FishBoxButton => "IMAGE_FISHBOXBUTTON",
            Self::PetButton => "IMAGE_PETBUTTON",
            Self::PetButtonHole => "IMAGE_PETBUTTONHOLE",
            Self::PetButtonRing => "IMAGE_PETBUTTONRING",
            Self::PetButtonReflect => "IMAGE_PETBUTTONREFLECT",
            Self::Checked => "IMAGE_CHECKED",
            Self::Unchecked => "IMAGE_UNCHECKED",
            Self::Guppy => "IMAGE_SMALLSWIM",
            Self::Ultra => "IMAGE_ULTRA",
            Self::Gekko => "IMAGE_GEKKO",
            Self::Penta => "IMAGE_STARCATCHER",
            Self::Grubber => "IMAGE_GRUBBER",
            Self::Breeder => "IMAGE_BREEDER",
            Self::Sylvester => "IMAGE_SYLV",
            Self::MiniSylvester => "IMAGE_MINISYLV",
            Self::Ball => "IMAGE_BALLS",
            Self::Bi => "IMAGE_BILATERUS",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cel {
    Whole,
    /// A numbered cel uses the asset catalog's column/row metadata.
    Indexed(u16),
    /// Column/row selection uses the asset catalog's cel dimensions.
    Grid {
        column: u8,
        row: u8,
    },
    Horizontal {
        index: u8,
        count: u8,
    },
    Cell {
        column: u8,
        row: u8,
        width: u16,
        height: u16,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sprite {
    pub image: ImageId,
    pub cel: Cel,
    pub bounds: Rect,
    pub clip: Rect,
    pub layer: Layer,
    pub alpha: f32,
}

impl Sprite {
    fn whole(image: ImageId, bounds: Rect, layer: Layer) -> Self {
        Self {
            image,
            cel: Cel::Whole,
            bounds,
            clip: SCREEN,
            layer,
            alpha: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FishPreview {
    pub kind: VirtualFishKind,
    pub size: FishSize,
    pub properties: FishProperties,
    pub sprite: Sprite,
}

impl FishPreview {
    fn new(
        kind: VirtualFishKind,
        size: FishSize,
        properties: FishProperties,
        bounds: Rect,
        ticks: u64,
    ) -> Self {
        let size_row = match size {
            FishSize::Small => 0,
            FishSize::Medium => 1,
            FishSize::Large | FishSize::Star => 2,
            FishSize::Crowned => 3,
        };
        // Uses the existing frontend's species sheet projection, not a new actor.
        let (image, cell, row) = match kind {
            VirtualFishKind::Guppy => (ImageId::Guppy, 80, size_row),
            VirtualFishKind::Oscar => (ImageId::Guppy, 80, 4),
            VirtualFishKind::Ultra => (ImageId::Ultra, 160, 0),
            VirtualFishKind::Gekko => (ImageId::Gekko, 80, 0),
            VirtualFishKind::Penta => (ImageId::Penta, 80, 0),
            VirtualFishKind::Grubber => (ImageId::Grubber, 80, 0),
            VirtualFishKind::Breeder => (ImageId::Breeder, 80, size_row * 3),
            VirtualFishKind::Sylvester if size == FishSize::Large => (ImageId::Sylvester, 160, 0),
            VirtualFishKind::Sylvester => (ImageId::MiniSylvester, 80, 0),
            VirtualFishKind::Ball => (
                ImageId::Ball,
                50,
                match size {
                    FishSize::Large => 1,
                    FishSize::Medium => 2,
                    _ => 0,
                },
            ),
            VirtualFishKind::Bi => (ImageId::Bi, 80, 5),
        };
        Self {
            kind,
            size,
            properties,
            sprite: Sprite {
                image,
                cel: Cel::Cell {
                    column: (ticks / 2 % 10) as u8,
                    row,
                    width: cell,
                    height: cell,
                },
                bounds,
                clip: bounds,
                layer: Layer::Preview,
                alpha: if properties.invisible { 0.35 } else { 1.0 },
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProductPreview {
    Fish(FishPreview),
    Item(Sprite),
}

fn product_preview(product: &StoreProduct, bounds: Rect, ticks: u64) -> ProductPreview {
    if let StoreProduct::Fish(fish) = product {
        return ProductPreview::Fish(FishPreview::new(
            fish.kind,
            fish.initial_size,
            fish.properties,
            bounds,
            ticks,
        ));
    }
    let (image, cel) = match product {
        StoreProduct::Bubbulator => (ImageId::Bubbulator, Cel::Indexed(0)),
        StoreProduct::AlienAttractor => (ImageId::AlienAttractor, Cel::Whole),
        StoreProduct::Backdrop(index) => (ImageId::StoreBackdrops, Cel::Indexed(u16::from(*index))),
        StoreProduct::FoodUpgrade => (
            ImageId::Food,
            Cel::Grid {
                column: (ticks / 4 % 10) as u8,
                row: 2,
            },
        ),
        StoreProduct::Fish(_) => unreachable!("fish preview handled above"),
    };
    ProductPreview::Item(Sprite {
        image,
        cel,
        bounds,
        clip: bounds,
        layer: Layer::Preview,
        alpha: 1.0,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OfferTarget {
    pub cycle: u64,
    pub slot: usize,
    pub product: StoreProduct,
}

impl OfferTarget {
    pub fn capture(tank: &VirtualTankState, slot: usize) -> Option<Self> {
        let stock = tank.stock.as_ref()?;
        if !stock.valid() {
            return None;
        }
        let product = stock.offers.get(slot)?.as_ref()?;
        Some(Self {
            cycle: stock.cycle,
            slot,
            product: product.clone(),
        })
    }

    pub fn matches(&self, snapshot: &VirtualUiSnapshot) -> bool {
        OfferTarget::capture(snapshot.tank(), self.slot).as_ref() == Some(self)
            && snapshot
                .tank
                .stock
                .as_ref()
                .is_some_and(|stock| stock.bought.get(self.slot) == Some(&false))
    }
}

/// Full paused-screen record prevents a recycled ID from accepting an old dialog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FishTarget(pub VirtualFish);

impl FishTarget {
    pub fn capture(tank: &VirtualTankState, id: u16) -> Option<Self> {
        if usize::from(id) >= tank.fish_capacity() {
            return None;
        }
        let mut matches = tank.active_fish().filter(|fish| fish.id == id);
        let fish = matches.next()?;
        if matches.next().is_some() {
            return None;
        }
        Some(Self(fish.clone()))
    }

    pub fn matches(&self, snapshot: &VirtualUiSnapshot) -> bool {
        Self::capture(snapshot.tank(), self.0.id).as_ref() == Some(self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PendingDialog {
    Purchase { target: OfferTarget, name: String },
    Rename { target: FishTarget, name: String },
    FishSale { target: FishTarget },
    BackdropSale { index: u8 },
}

#[derive(Clone, Debug)]
pub struct VirtualUiSnapshot {
    tank: VirtualTankState,
    pub shells: u32,
    pub now_seconds: u64,
    pub sale_grace_enabled: bool,
    pub dialog: Option<PendingDialog>,
    /// Blocks these screens while another caller-owned modal is active.
    pub external_block: bool,
}

impl VirtualUiSnapshot {
    pub fn capture(
        tank: &VirtualTankState,
        shells: u32,
        now_seconds: u64,
        sale_grace_enabled: bool,
    ) -> Self {
        Self {
            tank: tank.clone(),
            shells,
            now_seconds,
            sale_grace_enabled,
            dialog: None,
            external_block: false,
        }
    }

    pub fn tank(&self) -> &VirtualTankState {
        &self.tank
    }

    pub fn with_dialog(mut self, dialog: PendingDialog) -> Self {
        self.dialog = Some(dialog);
        self
    }

    pub fn selected(&self) -> Option<FishTarget> {
        FishTarget::capture(&self.tank, self.tank.selected_fish?)
    }

    fn sale_value(&self, fish: &VirtualFish) -> Option<u32> {
        fish.sale_value(self.now_seconds, self.sale_grace_enabled)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setting {
    FishNames,
    Bubbulator,
    AlienAttractor,
    DropShells,
    ShowHungry,
}

impl Setting {
    pub fn value(self, settings: &VirtualSettings) -> bool {
        match self {
            Self::FishNames => settings.show_names,
            Self::Bubbulator => settings.bubbulator_shown,
            Self::AlienAttractor => settings.alien_attractor_shown,
            Self::DropShells => settings.drop_shells,
            Self::ShowHungry => settings.always_show_hungry,
        }
    }

    fn available(self, tank: &VirtualTankState) -> bool {
        match self {
            Self::Bubbulator => tank.bubbulator_owned,
            Self::AlienAttractor => tank.alien_attractor_owned,
            _ => true,
        }
    }

    fn toggled(self, settings: &VirtualSettings) -> VirtualSettings {
        let mut next = settings.clone();
        match self {
            Self::FishNames => next.show_names = !next.show_names,
            Self::Bubbulator => next.bubbulator_shown = !next.bubbulator_shown,
            Self::AlienAttractor => next.alien_attractor_shown = !next.alien_attractor_shown,
            Self::DropShells => next.drop_shells = !next.drop_shells,
            Self::ShowHungry => next.always_show_hungry = !next.always_show_hungry,
        }
        next
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
    StoreSlot(usize),
    Fish(u16),
    ToggleSelected,
    RenameSelected,
    SellSelected,
    ShowAll,
    HideAll,
    PreviousBackdrop,
    NextBackdrop,
    SellBackdrop,
    Setting(Setting),
    ReturnToTank,
    DialogAccept,
    DialogCancel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonStyle {
    Main,
    Dialog,
    Left,
    Center,
    Right,
    StoreLeft,
    StoreTop,
    StoreRight,
    Fish,
    Checkbox,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ButtonView {
    pub control: Control,
    pub bounds: Rect,
    pub text: String,
    pub font: FontId,
    /// The production predicate; distinct from temporary modal/focus blocking.
    pub available: bool,
    pub enabled: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub selected: bool,
    pub style: ButtonStyle,
    pub checked: Option<bool>,
}

impl ButtonView {
    pub fn skin(&self) -> Sprite {
        let (image, count) = match self.style {
            ButtonStyle::Main => (ImageId::MainButton, 3),
            ButtonStyle::Dialog => (ImageId::DialogButton, 3),
            ButtonStyle::Left => (ImageId::LeftButton, 3),
            ButtonStyle::Center => (ImageId::CenterButton, 3),
            ButtonStyle::Right => (ImageId::RightButton, 3),
            ButtonStyle::StoreLeft => (ImageId::StoreLeft, 2),
            ButtonStyle::StoreTop => (ImageId::StoreTop, 2),
            ButtonStyle::StoreRight => (ImageId::StoreRight, 2),
            ButtonStyle::Fish => (ImageId::PetButton, 4),
            ButtonStyle::Checkbox => (
                if self.checked == Some(true) {
                    ImageId::Checked
                } else {
                    ImageId::Unchecked
                },
                1,
            ),
        };
        let index = match self.style {
            ButtonStyle::Checkbox => 0,
            ButtonStyle::Fish if !self.available => 0,
            ButtonStyle::Fish if self.pressed || self.selected => 2,
            ButtonStyle::Fish if self.hovered => 3,
            ButtonStyle::Fish => 1,
            ButtonStyle::StoreLeft | ButtonStyle::StoreTop | ButtonStyle::StoreRight => {
                u8::from(self.hovered || self.pressed || self.selected)
            }
            _ if self.pressed => 2,
            _ if self.hovered => 1,
            _ => 0,
        };
        Sprite {
            image,
            cel: if self.style == ButtonStyle::Checkbox {
                Cel::Whole
            } else {
                Cel::Horizontal { index, count }
            },
            bounds: self.bounds,
            clip: SCREEN,
            layer: if matches!(self.control, Control::DialogAccept | Control::DialogCancel) {
                Layer::Modal
            } else {
                Layer::Control
            },
            alpha: if self.available { 1.0 } else { 0.5 },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DialogView {
    pub bounds: Rect,
    pub text_bounds: Rect,
    pub title_font: FontId,
    pub body_font: FontId,
    pub title: &'static str,
    pub guidance: String,
    pub name: Option<String>,
    pub price: Option<u32>,
    pub preview: Option<ProductPreview>,
    pub stale: bool,
    pub buttons: [ButtonView; 2],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiIntent {
    OfferPurchase(OfferTarget),
    SelectFish(u16),
    SetFishShown {
        target: FishTarget,
        shown: bool,
    },
    SetAllFishShown {
        targets: Vec<FishTarget>,
        shown: bool,
    },
    BeginRename(FishTarget),
    BeginFishSale(FishTarget),
    BeginBackdropSale {
        index: u8,
    },
    SwitchBackdrop {
        direction: i8,
    },
    SetSettings(VirtualSettings),
    SetDialogName(String),
    ConfirmPurchase {
        target: OfferTarget,
        name: Option<String>,
    },
    ConfirmRename {
        target: FishTarget,
        name: String,
    },
    ConfirmFishSale(FishTarget),
    ConfirmBackdropSale {
        index: u8,
    },
    DismissDialog,
    ReturnToTank,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiInput {
    PointerMove(Point),
    PointerDown(Point),
    PointerUp(Point),
    PointerLeave,
    /// Keyboard/controller activation still checks current visible enable predicates.
    Activate(Control),
    EditName(String),
    Submit,
    Cancel,
}

/// Ticks are the caller's UI clock (W1 timers assume 100 ticks/sec), not wall time.
#[derive(Clone, Copy, Debug)]
pub struct UiClock {
    pub ticks: u64,
    pub focused: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Stamp {
    Offer(OfferTarget),
    Fish(FishTarget),
    Bulk(Vec<FishTarget>),
    Backdrop(u8, [bool; BACKDROP_COUNT]),
    Settings(VirtualSettings),
    Dialog(PendingDialog),
    Static,
}

fn roster(snapshot: &VirtualUiSnapshot) -> Vec<FishTarget> {
    let tank = snapshot.tank();
    let mut fish: Vec<_> = tank
        .active_fish()
        .filter(|fish| usize::from(fish.id) < tank.fish_capacity())
        .cloned()
        .map(FishTarget)
        .collect();
    fish.sort_by_key(|fish| (fish.0.purchased_at_seconds, fish.0.id));
    fish.truncate(crate::virtual_tank::BUBBULATOR_CAPACITY);
    fish
}

fn stamp(snapshot: &VirtualUiSnapshot, control: Control) -> Option<Stamp> {
    Some(match control {
        Control::StoreSlot(slot) => Stamp::Offer(OfferTarget::capture(snapshot.tank(), slot)?),
        Control::Fish(id) => Stamp::Fish(FishTarget::capture(snapshot.tank(), id)?),
        Control::ToggleSelected | Control::RenameSelected | Control::SellSelected => {
            Stamp::Fish(snapshot.selected()?)
        }
        Control::ShowAll | Control::HideAll => Stamp::Bulk(roster(snapshot)),
        Control::PreviousBackdrop | Control::NextBackdrop | Control::SellBackdrop => {
            Stamp::Backdrop(snapshot.tank.backdrop, snapshot.tank.owned_backdrops)
        }
        Control::Setting(_) => Stamp::Settings(snapshot.tank.settings.clone()),
        Control::DialogAccept | Control::DialogCancel => Stamp::Dialog(snapshot.dialog.clone()?),
        Control::ReturnToTank => Stamp::Static,
    })
}

#[derive(Default, Debug)]
struct Interaction {
    last_clock: Option<UiClock>,
    ticks: u64,
    hover: Option<Control>,
    hover_since: u64,
    down: Option<(Control, Stamp)>,
    dialog: Option<PendingDialog>,
}

impl Interaction {
    fn advance(&mut self, snapshot: &VirtualUiSnapshot, clock: UiClock) -> u64 {
        let elapsed = self
            .last_clock
            .filter(|last| last.focused && clock.focused)
            .map_or(0, |last| clock.ticks.saturating_sub(last.ticks));
        self.ticks = self.ticks.saturating_add(elapsed);
        self.last_clock = Some(clock);
        if !clock.focused || snapshot.external_block || self.dialog != snapshot.dialog {
            self.hover = None;
            self.down = None;
            self.hover_since = self.ticks;
        }
        self.dialog = snapshot.dialog.clone();
        elapsed
    }

    fn button(
        &self,
        snapshot: &VirtualUiSnapshot,
        control: Control,
        bounds: Rect,
        text: &str,
        enabled: bool,
        style: ButtonStyle,
    ) -> ButtonView {
        let modal_control = matches!(control, Control::DialogAccept | Control::DialogCancel);
        let available = enabled;
        let enabled = enabled
            && self.last_clock.is_some_and(|clock| clock.focused)
            && !snapshot.external_block
            && (snapshot.dialog.is_none() || modal_control);
        ButtonView {
            control,
            bounds,
            text: text.to_owned(),
            font: FontId::Body,
            available,
            enabled,
            hovered: self.hover == Some(control)
                && !snapshot.external_block
                && self.last_clock.is_some_and(|clock| clock.focused)
                && (snapshot.dialog.is_none() || modal_control),
            pressed: enabled
                && self.hover == Some(control)
                && self.down.as_ref().is_some_and(|(held, _)| *held == control),
            selected: false,
            style,
            checked: None,
        }
    }

    fn activate(
        &mut self,
        snapshot: &VirtualUiSnapshot,
        input: &UiInput,
        buttons: &[ButtonView],
    ) -> Option<Control> {
        if !self.last_clock.is_some_and(|clock| clock.focused) || snapshot.external_block {
            self.down = None;
            return None;
        }
        let allowed = |control: Control| {
            snapshot.dialog.is_none()
                || matches!(control, Control::DialogAccept | Control::DialogCancel)
        };
        let hit = |point| {
            buttons
                .iter()
                .rev()
                .find(|button| allowed(button.control) && button.bounds.contains(point))
        };
        match input {
            UiInput::PointerMove(point) => {
                let hover = hit(*point).map(|button| button.control);
                if self.hover != hover {
                    self.hover_since = self.ticks;
                }
                self.hover = hover;
            }
            UiInput::PointerDown(point) => {
                self.hover = hit(*point).map(|button| button.control);
                self.hover_since = self.ticks;
                self.down = hit(*point)
                    .filter(|button| button.enabled)
                    .and_then(|button| Some((button.control, stamp(snapshot, button.control)?)));
            }
            UiInput::PointerUp(point) => {
                let hover = hit(*point).map(|button| button.control);
                if self.hover != hover {
                    self.hover_since = self.ticks;
                }
                self.hover = hover;
                let (held, old_stamp) = self.down.take()?;
                let button =
                    hit(*point).filter(|button| button.enabled && button.control == held)?;
                if stamp(snapshot, button.control).as_ref() == Some(&old_stamp) {
                    return Some(button.control);
                }
            }
            UiInput::PointerLeave => {
                self.hover = None;
                self.down = None;
            }
            UiInput::Activate(control) => {
                self.down = None;
                if buttons
                    .iter()
                    .any(|button| button.control == *control && button.enabled && allowed(*control))
                {
                    return Some(*control);
                }
            }
            UiInput::Submit => {
                return buttons
                    .iter()
                    .find(|button| button.control == Control::DialogAccept && button.enabled)
                    .map(|button| button.control);
            }
            UiInput::Cancel => {
                self.down = None;
                return Some(if snapshot.dialog.is_some() {
                    Control::DialogCancel
                } else {
                    Control::ReturnToTank
                });
            }
            UiInput::EditName(_) => {}
        }
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OfferState {
    Available,
    Unavailable,
    Sold,
    AlreadyOwned,
    InsufficientShells,
    FishTankFull,
}

fn offer_state(snapshot: &VirtualUiSnapshot, target: &OfferTarget) -> OfferState {
    if !target.matches(snapshot) {
        return OfferState::Sold;
    }
    let tank = snapshot.tank();
    let owned = match &target.product {
        StoreProduct::Fish(fish) if !fish.valid() => return OfferState::Unavailable,
        StoreProduct::Fish(_) => false,
        StoreProduct::Bubbulator => tank.bubbulator_owned,
        StoreProduct::AlienAttractor => tank.alien_attractor_owned,
        StoreProduct::FoodUpgrade => tank.food_upgrade_owned,
        StoreProduct::Backdrop(index) => match tank.owned_backdrops.get(usize::from(*index)) {
            Some(owned) => *owned,
            None => return OfferState::Unavailable,
        },
    };
    if owned {
        return OfferState::AlreadyOwned;
    }
    if snapshot.shells < target.product.price() {
        return OfferState::InsufficientShells;
    }
    if matches!(target.product, StoreProduct::Fish(_)) && tank.next_fish_id().is_none() {
        return OfferState::FishTankFull;
    }
    OfferState::Available
}

fn can_sell_backdrop(snapshot: &VirtualUiSnapshot, index: u8) -> bool {
    snapshot.tank.backdrop == index
        && snapshot.tank.owned_backdrops.get(usize::from(index)) == Some(&true)
        && snapshot
            .tank
            .owned_backdrops
            .iter()
            .enumerate()
            .any(|(other, owned)| *owned && other != usize::from(index))
}

fn dialog_view(snapshot: &VirtualUiSnapshot, interaction: &Interaction) -> Option<DialogView> {
    let dialog = snapshot.dialog.as_ref()?;
    let (title, guidance, name, price, preview, stale, enabled) = match dialog {
        PendingDialog::Purchase { target, name } => {
            let stale = !target.matches(snapshot);
            let state = offer_state(snapshot, target);
            let needs_name = matches!(target.product, StoreProduct::Fish(_));
            (
                "Confirm purchase",
                state_guidance(state).to_owned(),
                needs_name.then(|| name.clone()),
                Some(target.product.price()),
                Some(product_preview(
                    &target.product,
                    Rect::new(275.0, 177.0, 90.0, 65.0),
                    interaction.ticks,
                )),
                stale,
                state == OfferState::Available && (!needs_name || valid_name(name)),
            )
        }
        PendingDialog::Rename { target, name } => (
            "Rename fish",
            "Choose a fish name.".to_owned(),
            Some(name.clone()),
            None,
            None,
            !target.matches(snapshot),
            target.matches(snapshot) && valid_name(name),
        ),
        PendingDialog::FishSale { target } => {
            let price = snapshot.sale_value(&target.0);
            let note = if target.0.kind == VirtualFishKind::Breeder
                && snapshot
                    .tank
                    .active_fish()
                    .any(|fish| fish.id == target.0.id + 100)
            {
                "The baby stays in the tank."
            } else {
                "Confirm this fish sale."
            };
            (
                "Sell fish",
                note.to_owned(),
                None,
                price,
                None,
                !target.matches(snapshot),
                target.matches(snapshot) && price.is_some(),
            )
        }
        PendingDialog::BackdropSale { index } => (
            "Sell backdrop",
            "Confirm this backdrop sale.".to_owned(),
            None,
            Some(crate::virtual_tank::BACKDROP_SALE_REFUND),
            None,
            !can_sell_backdrop(snapshot, *index),
            can_sell_backdrop(snapshot, *index),
        ),
    };
    Some(DialogView {
        bounds: Rect::new(85.0, 140.0, 470.0, 230.0),
        text_bounds: Rect::new(110.0, 245.0, 420.0, 48.0),
        title_font: FontId::Heading,
        body_font: FontId::Body,
        title,
        guidance: if stale {
            "Selection changed. Cancel and choose again.".to_owned()
        } else {
            guidance
        },
        name,
        price,
        preview,
        stale,
        buttons: [
            interaction.button(
                snapshot,
                Control::DialogAccept,
                Rect::new(140.0, 310.0, 150.0, 38.0),
                if matches!(dialog, PendingDialog::Rename { .. }) {
                    "Save"
                } else {
                    "Confirm"
                },
                enabled,
                ButtonStyle::Dialog,
            ),
            interaction.button(
                snapshot,
                Control::DialogCancel,
                Rect::new(350.0, 310.0, 150.0, 38.0),
                "Cancel",
                true,
                ButtonStyle::Dialog,
            ),
        ],
    })
}

fn valid_name(name: &str) -> bool {
    !name.trim().is_empty() && name.len() <= 64
}

fn dialog_intent(
    snapshot: &VirtualUiSnapshot,
    input: &UiInput,
    control: Option<Control>,
) -> Option<UiIntent> {
    if snapshot.external_block {
        return None;
    }
    if let UiInput::EditName(name) = input
        && name.len() <= 64
        && matches!(snapshot.dialog, Some(PendingDialog::Purchase { ref target, .. })
            if matches!(target.product, StoreProduct::Fish(_)))
    {
        return Some(UiIntent::SetDialogName(name.clone()));
    }
    if let UiInput::EditName(name) = input
        && name.len() <= 64
        && matches!(snapshot.dialog, Some(PendingDialog::Rename { .. }))
    {
        return Some(UiIntent::SetDialogName(name.clone()));
    }
    match control? {
        Control::DialogCancel => Some(UiIntent::DismissDialog),
        Control::DialogAccept => Some(match snapshot.dialog.as_ref()? {
            PendingDialog::Purchase { target, name } => UiIntent::ConfirmPurchase {
                target: target.clone(),
                name: matches!(target.product, StoreProduct::Fish(_))
                    .then(|| name.trim().to_owned()),
            },
            PendingDialog::Rename { target, name } => UiIntent::ConfirmRename {
                target: target.clone(),
                name: name.trim().to_owned(),
            },
            PendingDialog::FishSale { target } => UiIntent::ConfirmFishSale(target.clone()),
            PendingDialog::BackdropSale { index } => {
                UiIntent::ConfirmBackdropSale { index: *index }
            }
        }),
        Control::ReturnToTank if snapshot.dialog.is_none() => Some(UiIntent::ReturnToTank),
        _ => None,
    }
}

fn state_guidance(state: OfferState) -> &'static str {
    match state {
        OfferState::Available => "Select an item to buy with shells.",
        OfferState::Unavailable => "No item is available in this slot.",
        OfferState::Sold => "Sold. Check the next day's stock.",
        OfferState::AlreadyOwned => "Already owned.",
        OfferState::InsufficientShells => "Earn more shells to buy this item.",
        OfferState::FishTankFull => "Free a fish slot or buy the Bubbulator.",
    }
}

fn product_name(product: &StoreProduct) -> String {
    match product {
        StoreProduct::Fish(fish) => fish
            .properties
            .named
            .map_or_else(|| format!("{:?}", fish.kind), |name| format!("{name:?}")),
        StoreProduct::Bubbulator => "Bubbulator".to_owned(),
        StoreProduct::AlienAttractor => "Alien Attractor".to_owned(),
        StoreProduct::FoodUpgrade => "Food Upgrade".to_owned(),
        StoreProduct::Backdrop(index) => format!("Backdrop {}", usize::from(*index) + 1),
    }
}
