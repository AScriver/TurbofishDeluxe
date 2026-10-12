use super::*;
use crate::virtual_tank::{PurchaseResult, STORE_SLOTS};

// W1 StoreScreen construction. Slots preserve production stock indices.
const SLOT_RECTS: [Rect; STORE_SLOTS] = [
    Rect::new(1.0, 5.0, 109.0, 140.0),
    Rect::new(1.0, 143.0, 109.0, 140.0),
    Rect::new(1.0, 282.0, 109.0, 140.0),
    Rect::new(193.0, 38.0, 98.0, 140.0),
    Rect::new(303.0, 38.0, 98.0, 140.0),
    Rect::new(414.0, 38.0, 98.0, 140.0),
    Rect::new(525.0, 38.0, 98.0, 140.0),
    Rect::new(489.0, 339.0, 121.0, 100.0),
];
const RETURN_BOUNDS: Rect = Rect::new(6.0, 440.0, 113.0, 34.0);

#[derive(Clone, Debug, PartialEq)]
pub struct StoreSlotView {
    pub slot: usize,
    pub target: Option<OfferTarget>,
    pub state: OfferState,
    pub affordable: bool,
    pub fish_capacity_available: bool,
    pub price: Option<u32>,
    pub button: ButtonView,
    pub preview: Option<ProductPreview>,
    pub highlight: Option<Sprite>,
    pub price_bounds: Rect,
    pub price_font: FontId,
    pub price_label: String,
    pub shell_icon: Option<Sprite>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StoreView {
    pub background: Sprite,
    pub shell_balance: u32,
    pub slots: Vec<StoreSlotView>,
    pub return_button: ButtonView,
    pub speech: Option<String>,
    pub speech_bubble: Option<Sprite>,
    pub speech_bounds: Rect,
    pub speech_font: FontId,
    pub dialog: Option<DialogView>,
}

#[derive(Debug, Default)]
pub struct StoreController {
    interaction: Interaction,
    purchase_feedback_ticks: u64,
}

impl StoreController {
    /// Call only after the production transaction succeeds, not after an input.
    pub fn purchase_result(&mut self, result: &PurchaseResult) {
        if !matches!(result, PurchaseResult::Cancelled) {
            self.purchase_feedback_ticks = 108;
            self.interaction.hover = None;
            self.interaction.down = None;
        }
    }

    /// Advance feedback and captures without allocating product presentation data.
    pub fn advance(&mut self, snapshot: &VirtualUiSnapshot, clock: UiClock) {
        let elapsed = self.interaction.advance(snapshot, clock);
        self.purchase_feedback_ticks = self.purchase_feedback_ticks.saturating_sub(elapsed);
    }

    pub fn view(&mut self, snapshot: &VirtualUiSnapshot, clock: UiClock) -> StoreView {
        self.advance(snapshot, clock);
        let slots = SLOT_RECTS.iter().enumerate().map(|(slot, bounds)| {
            let (target, state) = slot_state(snapshot, slot);
            let style = match slot { 0..=2 => ButtonStyle::StoreLeft,
                3..=6 => ButtonStyle::StoreTop, _ => ButtonStyle::StoreRight };
            let modal_selected = matches!(snapshot.dialog, Some(PendingDialog::Purchase { ref target, .. })
                if target.slot == slot && target.matches(snapshot));
            let label = target.as_ref().map_or_else(|| "Unavailable".to_owned(),
                |target| product_name(&target.product));
            let mut button = self.interaction.button(snapshot, Control::StoreSlot(slot), *bounds,
                &label, state == OfferState::Available, style);
            button.selected = modal_selected;
            button.font = FontId::Price;
            let price = target.as_ref().map(|target| target.product.price());
            let affordable = price.is_some_and(|price| snapshot.shells >= price);
            let capacity = target.as_ref().is_none_or(|target|
                !matches!(target.product, StoreProduct::Fish(_)) || snapshot.tank.next_fish_id().is_some());
            let show_preview = !matches!(state, OfferState::Sold | OfferState::Unavailable | OfferState::AlreadyOwned);
            let preview_bounds = Rect::new(bounds.x + 8.0, bounds.y + 12.0,
                bounds.width - 16.0, if slot == 7 { 62.0 } else { 86.0 });
            let preview = target.as_ref().filter(|_| show_preview).map(|target|
                product_preview(&target.product, preview_bounds, self.interaction.ticks));
            let (highlight_image, dx, dy) = match slot {
                0..=2 => (ImageId::StoreLeftHighlight, 4.0, 4.0),
                3..=6 => (ImageId::StoreTopHighlight, -7.0, -2.0),
                _ => (ImageId::StoreRightHighlight, -4.0, -3.0),
            };
            let highlight = (button.hovered || button.pressed || modal_selected).then(||
                Sprite::whole(highlight_image, Rect::new(bounds.x + dx, bounds.y + dy,
                    bounds.width, bounds.height), Layer::Overlay));
            let price_bounds = Rect::new(bounds.x + 3.0, bounds.y + if slot == 7 { 84.0 } else { 108.0 },
                bounds.width - 6.0, 22.0);
            let price_label = match state {
                OfferState::Sold | OfferState::AlreadyOwned => "SOLD".to_owned(),
                OfferState::Unavailable => "Unavailable".to_owned(),
                _ => price.map_or_else(String::new, |price| price.to_string()),
            };
            StoreSlotView { slot, target, state, affordable, fish_capacity_available: capacity,
                price, button, preview, highlight, price_bounds, price_font: FontId::Price, price_label,
                shell_icon: show_preview.then(|| Sprite::whole(ImageId::StoreShell,
                    Rect::new(price_bounds.x, price_bounds.y + 3.0, 18.0, 18.0), Layer::Overlay)), }
        }).collect();
        let speech = if self.purchase_feedback_ticks > 0 {
            Some("Purchase complete. Your tank inventory is updated.".to_owned())
        } else if self
            .interaction
            .ticks
            .saturating_sub(self.interaction.hover_since)
            > 15
        {
            match self.interaction.hover {
                Some(Control::StoreSlot(slot)) => {
                    let target = OfferTarget::capture(snapshot.tank(), slot);
                    Some(target.as_ref().map_or_else(
                        || state_guidance(OfferState::Unavailable).to_owned(),
                        |target| {
                            let state = offer_state(snapshot, target);
                            if state == OfferState::Available {
                                product_guidance(&target.product)
                            } else {
                                state_guidance(state).to_owned()
                            }
                        },
                    ))
                }
                _ => None,
            }
        } else {
            None
        };
        StoreView {
            // W1 paints this over the product widgets; caller must honor the layer.
            background: Sprite::whole(ImageId::StoreScreen, SCREEN, Layer::Overlay),
            shell_balance: snapshot.shells,
            slots,
            return_button: self.interaction.button(
                snapshot,
                Control::ReturnToTank,
                RETURN_BOUNDS,
                "Back",
                true,
                ButtonStyle::Dialog,
            ),
            speech_bubble: speech.as_ref().map(|_| {
                Sprite::whole(
                    ImageId::SpeechBubble,
                    Rect::new(390.0, 220.0, 220.0, 110.0),
                    Layer::Overlay,
                )
            }),
            speech,
            speech_bounds: Rect::new(393.0, 235.0, 200.0, 85.0),
            speech_font: FontId::Speech,
            dialog: dialog_view(snapshot, &self.interaction),
        }
    }

    pub fn input(
        &mut self,
        snapshot: &VirtualUiSnapshot,
        clock: UiClock,
        input: &UiInput,
    ) -> Option<UiIntent> {
        self.advance(snapshot, clock);
        if !clock.focused || snapshot.external_block {
            return None;
        }
        if matches!(input, UiInput::PointerMove(_)) && self.purchase_feedback_ticks > 0 {
            self.purchase_feedback_ticks = self.purchase_feedback_ticks.saturating_sub(3);
            self.interaction.hover = None;
            return None;
        }
        let slots: [_; STORE_SLOTS] = std::array::from_fn(|slot| {
            self.interaction.control(
                snapshot,
                Control::StoreSlot(slot),
                SLOT_RECTS[slot],
                slot_state(snapshot, slot).1 == OfferState::Available,
            )
        });
        let back = self
            .interaction
            .control(snapshot, Control::ReturnToTank, RETURN_BOUNDS, true);
        let dialog = dialog_specs(snapshot)
            .map(|specs| specs.map(|spec| spec.input(&self.interaction, snapshot)));
        let control = self.interaction.activate(
            snapshot,
            input,
            slots
                .into_iter()
                .chain(std::iter::once(back))
                .chain(dialog.into_iter().flatten()),
        );
        if !clock.focused {
            return None;
        }
        if snapshot.dialog.is_some() || matches!(control, Some(Control::ReturnToTank)) {
            return dialog_intent(snapshot, input, control);
        }
        if let Some(Control::StoreSlot(slot)) = control {
            return OfferTarget::capture(snapshot.tank(), slot).map(UiIntent::OfferPurchase);
        }
        None
    }
}

fn slot_state(snapshot: &VirtualUiSnapshot, slot: usize) -> (Option<OfferTarget>, OfferState) {
    let target = OfferTarget::capture(snapshot.tank(), slot);
    let state = if snapshot
        .tank
        .stock
        .as_ref()
        .is_some_and(|stock| stock.bought[slot])
    {
        OfferState::Sold
    } else {
        target.as_ref().map_or(OfferState::Unavailable, |target| {
            offer_state(snapshot, target)
        })
    };
    (target, state)
}

fn product_guidance(product: &StoreProduct) -> String {
    match product {
        StoreProduct::Fish(_) => {
            format!("{}: choose a name before purchase.", product_name(product))
        }
        StoreProduct::Bubbulator => {
            "Bubbulator: increases purchased-fish capacity to 20.".to_owned()
        }
        StoreProduct::AlienAttractor => {
            "Alien Attractor: enables tank encounters when shown.".to_owned()
        }
        StoreProduct::FoodUpgrade => "Food Upgrade: improves newly dropped food.".to_owned(),
        StoreProduct::Backdrop(_) => "Backdrop: adds a background to Tank Setup.".to_owned(),
    }
}
