use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct SettingView {
    pub setting: Setting,
    pub button: ButtonView,
    pub label_bounds: Rect,
    pub guidance: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SetupView {
    pub background: Sprite,
    pub title: &'static str,
    pub title_font: FontId,
    pub backdrop: Option<Sprite>,
    pub backdrop_frame: Sprite,
    pub backdrop_index: u8,
    pub owned_backdrop_count: usize,
    pub controls: Vec<ButtonView>,
    pub settings: Vec<SettingView>,
    pub unavailable_devices: Vec<Setting>,
    /// The caller places its screensaver entry here and owns its action/lifecycle.
    pub screensaver_entry_bounds: Rect,
    pub guidance: &'static str,
    pub guidance_bounds: Rect,
    pub guidance_font: FontId,
    pub dialog: Option<DialogView>,
}

#[derive(Debug, Default)]
pub struct SetupController {
    interaction: Interaction,
}

impl SetupController {
    pub fn view(&mut self, snapshot: &VirtualUiSnapshot, clock: UiClock) -> SetupView {
        self.interaction.advance(snapshot, clock);
        let tank = snapshot.tank();
        let owned_count = tank.owned_backdrops.iter().filter(|owned| **owned).count();
        let valid_backdrop = tank.owned_backdrops.get(usize::from(tank.backdrop)) == Some(&true);
        // With one backdrop the source wraps to itself; disable the no-op as a UI policy.
        let navigate = valid_backdrop && owned_count > 1;
        let controls = vec![
            self.interaction.button(
                snapshot,
                Control::PreviousBackdrop,
                Rect::new(74.0, 274.0, 65.0, 32.0),
                "Prev",
                navigate,
                ButtonStyle::Left,
            ),
            self.interaction.button(
                snapshot,
                Control::NextBackdrop,
                Rect::new(204.0, 274.0, 65.0, 32.0),
                "Next",
                navigate,
                ButtonStyle::Right,
            ),
            self.interaction.button(
                snapshot,
                Control::SellBackdrop,
                Rect::new(137.0, 274.0, 70.0, 32.0),
                "Sell",
                can_sell_backdrop(snapshot, tank.backdrop),
                ButtonStyle::Center,
            ),
            self.interaction.button(
                snapshot,
                Control::ReturnToTank,
                Rect::new(225.0, 420.0, 186.0, 36.0),
                "Return to Tank",
                true,
                ButtonStyle::Dialog,
            ),
        ];
        let mut unavailable_devices = Vec::new();
        let settings = [
            Setting::FishNames,
            Setting::Bubbulator,
            Setting::AlienAttractor,
            Setting::DropShells,
            Setting::ShowHungry,
        ]
        .into_iter()
        .enumerate()
        .filter_map(|(row, setting)| {
            if !setting.available(tank) {
                unavailable_devices.push(setting);
                return None;
            }
            let mut button = self.interaction.button(
                snapshot,
                Control::Setting(setting),
                Rect::new(340.0, 140.0 + row as f32 * 45.0, 46.0, 45.0),
                setting_label(setting),
                true,
                ButtonStyle::Checkbox,
            );
            button.checked = Some(setting.value(&tank.settings));
            Some(SettingView {
                label_bounds: Rect::new(390.0, button.bounds.y + 5.0, 220.0, 40.0),
                setting,
                button,
                guidance: setting_guidance(setting),
            })
        })
        .collect();
        let guidance = match self.interaction.hover {
            Some(Control::Setting(setting)) => setting_guidance(setting),
            Some(Control::PreviousBackdrop | Control::NextBackdrop) => {
                "Choose from your owned backdrops."
            }
            Some(Control::SellBackdrop) => {
                if owned_count > 1 {
                    "Sell this backdrop for 10,000 shells."
                } else {
                    "Keep at least one backdrop."
                }
            }
            _ => "Choose a backdrop and tank display settings.",
        };
        SetupView {
            background: Sprite::whole(ImageId::ScreenBack, SCREEN, Layer::Background),
            title: "Tank Setup",
            title_font: FontId::Title,
            backdrop_frame: Sprite::whole(
                ImageId::TankChooser,
                Rect::new(60.0, 140.0, 226.0, 130.0),
                Layer::Control,
            ),
            backdrop: valid_backdrop.then(|| {
                Sprite::whole(
                    ImageId::Backdrop(tank.backdrop),
                    Rect::new(76.0, 150.0, 194.0, 110.0),
                    Layer::Preview,
                )
            }),
            backdrop_index: tank.backdrop,
            owned_backdrop_count: owned_count,
            controls,
            settings,
            unavailable_devices,
            screensaver_entry_bounds: Rect::new(80.0, 320.0, 186.0, 36.0),
            guidance,
            guidance_bounds: Rect::new(60.0, 45.0, 520.0, 48.0),
            guidance_font: FontId::Body,
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
        buttons.extend(view.settings.iter().map(|setting| setting.button.clone()));
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
            Control::PreviousBackdrop => Some(UiIntent::SwitchBackdrop { direction: -1 }),
            Control::NextBackdrop => Some(UiIntent::SwitchBackdrop { direction: 1 }),
            Control::SellBackdrop => Some(UiIntent::BeginBackdropSale {
                index: snapshot.tank.backdrop,
            }),
            Control::Setting(setting) if setting.available(snapshot.tank()) => Some(
                UiIntent::SetSettings(setting.toggled(&snapshot.tank.settings)),
            ),
            _ => None,
        }
    }
}

fn setting_label(setting: Setting) -> &'static str {
    match setting {
        Setting::FishNames => "Fish Names",
        Setting::Bubbulator => "Bubbulator",
        Setting::AlienAttractor => "Alien Attractor",
        Setting::DropShells => "Drop Shells",
        Setting::ShowHungry => "Show Hungry",
    }
}

fn setting_guidance(setting: Setting) -> &'static str {
    match setting {
        Setting::FishNames => "Show fish names in the tank.",
        Setting::Bubbulator => "Show your purchased Bubbulator.",
        Setting::AlienAttractor => "Enable encounters with your purchased Alien Attractor.",
        Setting::DropShells => "Allow fish to produce shells.",
        Setting::ShowHungry => "Show hunger even after three qualifying feeds today.",
    }
}
