use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct FishRow {
    pub position: usize,
    pub target: Option<FishTarget>,
    pub button: ButtonView,
    pub preview: Option<FishPreview>,
    pub hidden: bool,
    pub name: Option<String>,
    pub name_font: FontId,
    pub ring: Sprite,
    pub reflection: Option<Sprite>,
}

/// Numeric biography references are resolved by the caller's runtime catalog.
#[derive(Clone, Debug, PartialEq)]
pub struct FishDetails {
    pub target: FishTarget,
    pub bounds: Rect,
    pub preview: FishPreview,
    pub name_font: FontId,
    pub body_font: FontId,
    pub date_is_birth: bool,
    pub timestamp_seconds: u64,
    pub hometown: u16,
    pub likes: [u16; 3],
    pub special_interest: Option<&'static str>,
    pub mental_state: u8,
    pub feeds_today: u8,
    pub trait_labels: Vec<&'static str>,
    pub purchase_price: Option<u32>,
    pub sale_value: Option<u32>,
    pub finance_visible: bool,
    pub finance_alpha: f32,
    pub biography_alpha: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FishManagementView {
    pub background: Sprite,
    pub title: &'static str,
    pub title_font: FontId,
    pub panel: Sprite,
    pub rows: Vec<FishRow>,
    pub controls: Vec<ButtonView>,
    pub selected: Option<FishDetails>,
    pub empty_guidance: Option<&'static str>,
    pub dialog: Option<DialogView>,
}

#[derive(Debug, Default)]
pub struct FishController {
    interaction: Interaction,
    finance_alpha: f32,
}

impl FishController {
    pub fn view(&mut self, snapshot: &VirtualUiSnapshot, clock: UiClock) -> FishManagementView {
        let elapsed = self.interaction.advance(snapshot, clock);
        let finance = self.interaction.hover == Some(Control::SellSelected)
            || matches!(snapshot.dialog, Some(PendingDialog::FishSale { .. }));
        let change = elapsed.min(8) as f32 / 8.0;
        self.finance_alpha = if finance {
            (self.finance_alpha + change).min(1.0)
        } else {
            (self.finance_alpha - change).max(0.0)
        };
        let fish = roster(snapshot);
        let selected = snapshot.selected();
        let rows = (0..crate::virtual_tank::BUBBULATOR_CAPACITY)
            .map(|position| {
                let column = position / 5;
                let x = [25.0, 119.0, 425.0, 519.0][column];
                let bounds = Rect::new(x, 41.0 + (position % 5) as f32 * 83.0, 90.0, 83.0);
                let target = fish.get(position).cloned();
                // Empty slots have no routable fish ID; their disabled control is never activated.
                let control = Control::Fish(target.as_ref().map_or(u16::MAX, |fish| fish.0.id));
                let enabled = target
                    .as_ref()
                    .is_some_and(|target| target.matches(snapshot));
                let is_selected = target
                    .as_ref()
                    .is_some_and(|fish| selected.as_ref() == Some(fish));
                let mut button = self.interaction.button(
                    snapshot,
                    control,
                    bounds,
                    "",
                    enabled,
                    ButtonStyle::Fish,
                );
                button.selected = is_selected;
                let preview = target.as_ref().map(|target| {
                    FishPreview::new(
                        target.0.kind,
                        target.0.size,
                        target.0.properties,
                        Rect::new(x + 5.0, bounds.y + 2.0, 80.0, 70.0),
                        self.interaction.ticks,
                    )
                });
                let hidden = target.as_ref().is_some_and(|target| !target.0.shown);
                FishRow {
                    position,
                    name: target.as_ref().map(|target| target.0.name.clone()),
                    target,
                    button,
                    preview,
                    hidden,
                    name_font: FontId::Body,
                    ring: Sprite::whole(ImageId::PetButtonRing, bounds, Layer::Overlay),
                    reflection: enabled
                        .then(|| Sprite::whole(ImageId::PetButtonReflect, bounds, Layer::Overlay)),
                }
            })
            .collect();
        let any_shown = fish.iter().any(|fish| fish.0.shown);
        let bulk_control = if any_shown {
            Control::HideAll
        } else {
            Control::ShowAll
        };
        let selected_shown = selected.as_ref().is_none_or(|fish| fish.0.shown);
        let sale_enabled = selected
            .as_ref()
            .is_some_and(|fish| snapshot.sale_value(&fish.0).is_some());
        let controls = vec![
            self.interaction.button(
                snapshot,
                bulk_control,
                Rect::new(500.0, 4.0, 100.0, 30.0),
                if any_shown { "Hide All" } else { "Show All" },
                !fish.is_empty(),
                ButtonStyle::Main,
            ),
            self.interaction.button(
                snapshot,
                Control::ToggleSelected,
                Rect::new(225.0, 390.0, 58.0, 30.0),
                if selected_shown { "Hide" } else { "Show" },
                selected.is_some(),
                ButtonStyle::Left,
            ),
            self.interaction.button(
                snapshot,
                Control::RenameSelected,
                Rect::new(281.0, 390.0, 74.0, 30.0),
                "Rename",
                selected.is_some(),
                ButtonStyle::Center,
            ),
            self.interaction.button(
                snapshot,
                Control::SellSelected,
                Rect::new(353.0, 390.0, 65.0, 30.0),
                "Sell",
                sale_enabled,
                ButtonStyle::Right,
            ),
            self.interaction.button(
                snapshot,
                Control::ReturnToTank,
                Rect::new(218.0, 428.0, 206.0, 36.0),
                "Return to Tank",
                true,
                ButtonStyle::Dialog,
            ),
        ];
        let details = selected.map(|target| {
            let fish = &target.0;
            FishDetails {
                bounds: Rect::new(234.0, 175.0, 172.0, 205.0),
                preview: FishPreview::new(
                    fish.kind,
                    fish.size,
                    fish.properties,
                    Rect::new(280.0, 95.0, 80.0, 80.0),
                    self.interaction.ticks,
                ),
                name_font: FontId::Heading,
                body_font: FontId::Body,
                date_is_birth: fish.breeder_child,
                timestamp_seconds: fish.purchased_at_seconds,
                hometown: fish.hometown,
                likes: fish.likes,
                special_interest: special_interest(fish.properties),
                mental_state: fish.mental_state,
                feeds_today: fish.fed_today,
                trait_labels: trait_labels(fish.properties),
                purchase_price: (!fish.breeder_child).then_some(fish.purchase_price),
                sale_value: snapshot.sale_value(fish),
                finance_visible: finance || self.finance_alpha > 0.0,
                finance_alpha: self.finance_alpha,
                biography_alpha: 1.0 - self.finance_alpha,
                target,
            }
        });
        FishManagementView {
            background: Sprite::whole(ImageId::ScreenBack, SCREEN, Layer::Background),
            title: "Fish Setup",
            title_font: FontId::Title,
            panel: Sprite::whole(
                ImageId::FishBox,
                Rect::new(216.0, 48.0, 208.0, 350.0),
                Layer::Background,
            ),
            rows,
            controls,
            empty_guidance: details
                .is_none()
                .then_some("Select a fish to view its details."),
            selected: details,
            dialog: dialog_view(snapshot, &self.interaction),
        }
    }

    pub fn input(
        &mut self,
        snapshot: &VirtualUiSnapshot,
        clock: UiClock,
        input: &UiInput,
    ) -> Option<UiIntent> {
        let view = self.view(snapshot, clock);
        let mut buttons = view.controls;
        buttons.extend(view.rows.iter().map(|row| row.button.clone()));
        if let Some(dialog) = &view.dialog {
            buttons.extend_from_slice(&dialog.buttons);
        }
        let control = self.interaction.activate(snapshot, input, &buttons);
        if !clock.focused {
            return None;
        }
        if snapshot.dialog.is_some() || matches!(control, Some(Control::ReturnToTank)) {
            return dialog_intent(snapshot, input, control);
        }
        match control? {
            Control::Fish(id) => Some(UiIntent::SelectFish(id)),
            Control::ToggleSelected => snapshot.selected().map(|target| UiIntent::SetFishShown {
                shown: !target.0.shown,
                target,
            }),
            Control::RenameSelected => snapshot.selected().map(UiIntent::BeginRename),
            Control::SellSelected => snapshot.selected().map(UiIntent::BeginFishSale),
            Control::ShowAll => Some(UiIntent::SetAllFishShown {
                targets: roster(snapshot),
                shown: true,
            }),
            Control::HideAll => Some(UiIntent::SetAllFishShown {
                targets: roster(snapshot),
                shown: false,
            }),
            _ => None,
        }
    }
}

fn trait_labels(properties: FishProperties) -> Vec<&'static str> {
    let mut labels = Vec::new();
    for (enabled, label) in [
        (properties.invisible, "Invisible"),
        (properties.voracious, "Voracious"),
        (properties.speedy, "Speedy"),
        (properties.singing, "Singing"),
        (properties.backward, "Backward swimmer"),
    ] {
        if enabled {
            labels.push(label);
        }
    }
    if properties.named.is_some() {
        labels.push("Named special fish");
    }
    if properties.colors == crate::virtual_tank::FishColors::Rainbow {
        labels.push("Rainbow colors");
    } else if properties.colors == crate::virtual_tank::FishColors::Special {
        labels.push("Special colors");
    }
    labels
}

fn special_interest(properties: FishProperties) -> Option<&'static str> {
    use crate::virtual_tank::ExoticDiet;
    Some(match properties.attribute() {
        0 => "Stealth",
        1 => "Eating",
        2 => "Speed",
        3 => "Singing",
        4 => "Swimming backward",
        5 | 6 => match properties.exotic_diet? {
            ExoticDiet::Pizza => "Pizza",
            ExoticDiet::IceCream => "Ice cream",
            ExoticDiet::Chicken => "Chicken",
            ExoticDiet::Guppy => "Guppies",
            ExoticDiet::Oscar => "Carnivores",
            ExoticDiet::Ultra => "Ultravores",
            ExoticDiet::Cookie => "Cookie food",
            ExoticDiet::None => return None,
        },
        7 => "Multiple special traits",
        // Named notes need a caller-owned catalog, not a copied retail paragraph.
        _ => return None,
    })
}
