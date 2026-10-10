use super::*;
use crate::virtual_tank::{
    FishBiography, FishOffer, PurchaseConfirmation, PurchaseResult, StoreStock,
};

fn clock(ticks: u64) -> UiClock {
    UiClock {
        ticks,
        focused: true,
    }
}

fn stocked() -> VirtualTankState {
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
    tank.stock = Some(StoreStock {
        cycle: 42,
        offers,
        bought: [false; 8],
    });
    tank
}

fn with_fish(count: u16) -> VirtualTankState {
    let mut tank = stocked();
    let mut shells = 1_000_000;
    for id in 0..count {
        tank.stock.as_mut().unwrap().bought[3] = false;
        let result = tank
            .confirm_purchase(
                &mut shells,
                PurchaseConfirmation {
                    cycle: 42,
                    slot: 3,
                    expected_price: 25,
                    accepted: true,
                    fish_name: Some(format!("Fish {id}")),
                    fish_biography: Some(FishBiography {
                        hometown: 1,
                        likes: [1, 2, 3],
                    }),
                },
                u64::from(id) + 100,
                0,
            )
            .unwrap();
        assert_eq!(result, PurchaseResult::Fish { id, price: 25 });
    }
    tank.stock.as_mut().unwrap().bought[3] = false;
    tank
}

fn snapshot(tank: &VirtualTankState) -> VirtualUiSnapshot {
    VirtualUiSnapshot::capture(tank, 100_000, 200_000, true)
}

fn slot_point() -> Point {
    Point { x: 210.0, y: 60.0 }
}

#[test]
fn store_uses_production_prices_gates_and_never_mutates_snapshot() {
    let tank = stocked();
    let original = tank.clone();
    let mut controller = StoreController::default();
    let price = tank.stock.as_ref().unwrap().offers[3]
        .as_ref()
        .unwrap()
        .price();
    let poor = VirtualUiSnapshot::capture(&tank, price - 1, 200_000, true);
    let view = controller.view(&poor, clock(0));
    assert_eq!(view.slots[3].price, Some(price));
    assert_eq!(view.slots[3].state, OfferState::InsufficientShells);
    assert!(!view.slots[3].button.enabled);
    assert!(view.slots[3].preview.is_some());
    assert!(
        controller
            .input(&poor, clock(0), &UiInput::Activate(Control::StoreSlot(3)))
            .is_none()
    );
    let enough = VirtualUiSnapshot::capture(&tank, price, 200_000, true);
    assert!(
        matches!(controller.input(&enough, clock(1), &UiInput::Activate(Control::StoreSlot(3))),
        Some(UiIntent::OfferPurchase(target)) if target.product.price() == price)
    );
    assert!(
        controller
            .input(
                &enough,
                clock(1),
                &UiInput::Activate(Control::StoreSlot(usize::MAX))
            )
            .is_none()
    );
    assert_eq!(tank, original);
    assert_eq!(enough.tank(), &original);
}

#[test]
fn press_release_rejects_refreshed_stock_and_focus_loss() {
    let tank = stocked();
    let original = snapshot(&tank);
    let mut controller = StoreController::default();
    controller.input(&original, clock(0), &UiInput::PointerDown(slot_point()));
    assert!(controller.view(&original, clock(0)).slots[3].button.pressed);
    let mut changed = tank.clone();
    changed.stock.as_mut().unwrap().cycle += 1;
    assert!(
        controller
            .input(
                &snapshot(&changed),
                clock(1),
                &UiInput::PointerUp(slot_point())
            )
            .is_none()
    );
    controller.input(&original, clock(2), &UiInput::PointerDown(slot_point()));
    controller.view(
        &original,
        UiClock {
            ticks: 3,
            focused: false,
        },
    );
    assert!(
        controller
            .input(&original, clock(4), &UiInput::PointerUp(slot_point()))
            .is_none()
    );
    controller.input(&original, clock(5), &UiInput::PointerDown(slot_point()));
    assert!(
        controller
            .input(
                &original,
                clock(6),
                &UiInput::PointerUp(Point { x: 400.0, y: 460.0 })
            )
            .is_none()
    );
    controller.input(&original, clock(7), &UiInput::PointerDown(slot_point()));
    assert!(matches!(
        controller.input(&original, clock(8), &UiInput::PointerUp(slot_point())),
        Some(UiIntent::OfferPurchase(_))
    ));
    controller.input(&original, clock(9), &UiInput::PointerDown(slot_point()));
    let mut blocked = original.clone();
    blocked.external_block = true;
    controller.view(&blocked, clock(10));
    assert!(
        controller
            .input(&original, clock(11), &UiInput::PointerUp(slot_point()))
            .is_none()
    );
}

#[test]
fn purchase_dialog_blocks_background_and_refuses_stale_or_blank_names() {
    let tank = stocked();
    let target = OfferTarget::capture(&tank, 3).unwrap();
    let blank = snapshot(&tank).with_dialog(PendingDialog::Purchase {
        target: target.clone(),
        name: " ".to_owned(),
    });
    let mut store = StoreController::default();
    assert!(
        store
            .view(&blank, clock(0))
            .slots
            .iter()
            .all(|slot| !slot.button.enabled)
    );
    assert!(store.input(&blank, clock(0), &UiInput::Submit).is_none());
    assert!(
        store
            .input(&blank, clock(0), &UiInput::Activate(Control::StoreSlot(2)))
            .is_none()
    );
    assert!(
        store
            .input(&blank, clock(0), &UiInput::EditName("é".repeat(33)))
            .is_none()
    );
    assert_eq!(
        store.input(&blank, clock(0), &UiInput::EditName("Pearl".to_owned())),
        Some(UiIntent::SetDialogName("Pearl".to_owned()))
    );
    let named = snapshot(&tank).with_dialog(PendingDialog::Purchase {
        target: target.clone(),
        name: " Pearl ".to_owned(),
    });
    assert!(
        matches!(store.input(&named, clock(1), &UiInput::Submit), Some(UiIntent::ConfirmPurchase { name: Some(name), .. }) if name == "Pearl")
    );
    let mut newer = tank;
    newer.stock.as_mut().unwrap().cycle += 1;
    let stale = snapshot(&newer).with_dialog(PendingDialog::Purchase {
        target,
        name: "Pearl".to_owned(),
    });
    assert!(store.view(&stale, clock(2)).dialog.unwrap().stale);
    assert!(store.input(&stale, clock(2), &UiInput::Submit).is_none());
    assert_eq!(
        store.input(&stale, clock(2), &UiInput::Cancel),
        Some(UiIntent::DismissDialog)
    );
}

#[test]
fn full_fish_roster_keeps_bubbulator_offer_available_and_sold_previews_removed() {
    let mut tank = with_fish(10);
    let mut store = StoreController::default();
    let view = store.view(&snapshot(&tank), clock(0));
    assert_eq!(view.slots[3].state, OfferState::FishTankFull);
    assert!(!view.slots[3].button.enabled);
    assert_eq!(view.slots[2].state, OfferState::Available);
    assert!(view.slots[2].button.enabled);
    tank.bubbulator_owned = true;
    tank.stock.as_mut().unwrap().bought[2] = true;
    let view = store.view(&snapshot(&tank), clock(1));
    assert_eq!(view.slots[3].state, OfferState::Available);
    assert_eq!(view.slots[2].state, OfferState::Sold);
    assert_eq!(view.slots[2].price_label, "SOLD");
    assert!(view.slots[2].preview.is_none());
    assert_eq!(tank.fish_capacity(), 20);
}

#[test]
fn fish_grid_and_visibility_follow_owned_ids_with_safe_empty_selection() {
    let mut fish = FishController::default();
    let empty = snapshot(&VirtualTankState::default());
    let view = fish.view(&empty, clock(0));
    assert_eq!(view.rows.len(), 20);
    assert!(view.rows.iter().all(|row| !row.button.enabled));
    assert!(view.selected.is_none());
    assert!(
        fish.input(&empty, clock(0), &UiInput::Activate(Control::HideAll))
            .is_none()
    );
    let mut tank = with_fish(2);
    tank.fish.reverse();
    tank.selected_fish = Some(19);
    assert!(fish.view(&snapshot(&tank), clock(1)).selected.is_none());
    assert!(
        fish.input(
            &snapshot(&tank),
            clock(1),
            &UiInput::Activate(Control::SellSelected)
        )
        .is_none()
    );
    tank.selected_fish = Some(0);
    let view = fish.view(&snapshot(&tank), clock(2));
    assert_eq!(view.rows[0].target.as_ref().unwrap().0.id, 0);
    assert_eq!(view.selected.as_ref().unwrap().likes, [1, 2, 3]);
    assert_eq!(view.controls[0].control, Control::HideAll);
    assert!(
        matches!(fish.input(&snapshot(&tank), clock(2), &UiInput::Activate(Control::HideAll)), Some(UiIntent::SetAllFishShown { shown: false, targets }) if targets.len() == 2)
    );
    tank.set_all_fish_shown(false);
    let view = fish.view(&snapshot(&tank), clock(3));
    assert!(view.rows[0].hidden);
    assert_eq!(view.controls[0].control, Control::ShowAll);
    assert!(
        matches!(fish.input(&snapshot(&tank), clock(3), &UiInput::Activate(Control::ToggleSelected)), Some(UiIntent::SetFishShown { shown: true, target }) if target.0.id == 0)
    );
}

#[test]
fn fish_rename_and_sale_dialogs_reject_recycled_ids_and_current_sale_grace() {
    let mut tank = with_fish(1);
    tank.selected_fish = Some(0);
    let target = FishTarget::capture(&tank, 0).unwrap();
    let mut fish = FishController::default();
    let recent = VirtualUiSnapshot::capture(&tank, 100_000, 101, true);
    assert!(
        fish.input(&recent, clock(0), &UiInput::Activate(Control::SellSelected))
            .is_none()
    );
    let sale = snapshot(&tank).with_dialog(PendingDialog::FishSale {
        target: target.clone(),
    });
    assert_eq!(fish.view(&sale, clock(1)).dialog.unwrap().price, Some(12));
    assert_eq!(
        fish.input(&sale, clock(1), &UiInput::Submit),
        Some(UiIntent::ConfirmFishSale(target.clone()))
    );
    let renamed = snapshot(&tank).with_dialog(PendingDialog::Rename {
        target: target.clone(),
        name: "Fin".to_owned(),
    });
    assert!(
        matches!(fish.input(&renamed, clock(2), &UiInput::Submit), Some(UiIntent::ConfirmRename { name, .. }) if name == "Fin")
    );
    // The ID is unchanged, but this is a different owned record.
    tank.fish[0].name = "Replacement".to_owned();
    let stale = snapshot(&tank).with_dialog(PendingDialog::FishSale { target });
    assert!(fish.view(&stale, clock(3)).dialog.unwrap().stale);
    assert!(fish.input(&stale, clock(3), &UiInput::Submit).is_none());
    assert_eq!(
        fish.input(&stale, clock(3), &UiInput::Cancel),
        Some(UiIntent::DismissDialog)
    );
}

#[test]
fn setup_hides_unowned_devices_guards_last_backdrop_and_preserves_other_settings() {
    let mut tank = VirtualTankState::default();
    let mut setup = SetupController::default();
    let view = setup.view(&snapshot(&tank), clock(0));
    assert_eq!(view.settings.len(), 3);
    assert_eq!(
        view.unavailable_devices,
        [Setting::Bubbulator, Setting::AlienAttractor]
    );
    assert!(
        setup
            .input(
                &snapshot(&tank),
                clock(0),
                &UiInput::Activate(Control::Setting(Setting::Bubbulator))
            )
            .is_none()
    );
    assert!(
        setup
            .input(
                &snapshot(&tank),
                clock(0),
                &UiInput::Activate(Control::SellBackdrop)
            )
            .is_none()
    );
    tank.bubbulator_owned = true;
    tank.owned_backdrops[2] = true;
    let view = setup.view(&snapshot(&tank), clock(1));
    assert_eq!(view.settings.len(), 4);
    assert!(view.controls[0].enabled);
    let Some(UiIntent::SetSettings(settings)) = setup.input(
        &snapshot(&tank),
        clock(1),
        &UiInput::Activate(Control::Setting(Setting::Bubbulator)),
    ) else {
        panic!("owned device should toggle");
    };
    assert!(settings.bubbulator_shown);
    assert_eq!(settings.drop_shells, tank.settings.drop_shells);
    assert_eq!(settings.show_names, tank.settings.show_names);
    let sale = snapshot(&tank).with_dialog(PendingDialog::BackdropSale { index: 0 });
    assert_eq!(
        setup.view(&sale, clock(2)).dialog.unwrap().price,
        Some(10_000)
    );
    assert_eq!(
        setup.input(&sale, clock(2), &UiInput::Submit),
        Some(UiIntent::ConfirmBackdropSale { index: 0 })
    );
    tank.backdrop = 2;
    let stale = snapshot(&tank).with_dialog(PendingDialog::BackdropSale { index: 0 });
    assert!(setup.input(&stale, clock(3), &UiInput::Submit).is_none());
    tank.backdrop = u8::MAX;
    assert!(setup.view(&snapshot(&tank), clock(4)).backdrop.is_none());
}

#[test]
fn hover_feedback_and_preview_clocks_pause_on_focus_loss_and_external_modals() {
    let snapshot = snapshot(&stocked());
    let mut store = StoreController::default();
    store.input(&snapshot, clock(0), &UiInput::PointerMove(slot_point()));
    assert!(store.view(&snapshot, clock(15)).speech.is_none());
    assert!(store.view(&snapshot, clock(16)).speech.is_some());
    let preview = store.view(&snapshot, clock(16)).slots[3].preview.clone();
    assert_eq!(
        store
            .view(
                &snapshot,
                UiClock {
                    ticks: 10_000,
                    focused: false
                }
            )
            .slots[3]
            .preview,
        preview
    );
    assert_eq!(
        store.view(&snapshot, clock(10_001)).slots[3].preview,
        preview
    );
    store.purchase_result(&PurchaseResult::Cancelled);
    assert!(store.view(&snapshot, clock(10_001)).speech.is_none());
    store.purchase_result(&PurchaseResult::Fish { id: 0, price: 25 });
    assert!(
        store
            .view(&snapshot, clock(10_001))
            .speech
            .unwrap()
            .starts_with("Purchase complete")
    );
    assert!(store.view(&snapshot, clock(10_109)).speech.is_none());
    let mut blocked = snapshot;
    blocked.external_block = true;
    assert!(
        store
            .input(&blocked, clock(10_110), &UiInput::Cancel)
            .is_none()
    );
    assert!(
        store
            .view(&blocked, clock(10_110))
            .slots
            .iter()
            .all(|slot| !slot.button.enabled)
    );
}

#[test]
fn session_adapters_keep_current_targets_and_order_name_before_commit() {
    use crate::sim::Action;
    let mut tank = with_fish(1);
    tank.selected_fish = Some(0);
    let original = tank.clone();
    let plain = snapshot(&tank);
    let offer = OfferTarget::capture(&tank, 3).unwrap();
    let purchase = plain.clone().with_dialog(PendingDialog::Purchase {
        target: offer.clone(),
        name: "Fin".to_owned(),
    });
    let intent = UiIntent::ConfirmPurchase {
        target: offer.clone(),
        name: Some("Fin".to_owned()),
    };
    let actions = intent.session_actions(&purchase, 42).unwrap();
    assert_eq!(actions.len(), 2);
    assert!(matches!(&actions[0], Action::VirtualSetPurchaseName { name } if name == "Fin"));
    assert!(matches!(
        actions[1],
        Action::VirtualConfirmPurchase {
            accept: true,
            now_seconds: 200_000,
            day_start_seconds: 42
        }
    ));
    assert!(intent.session_actions(&purchase, 43).is_none());
    assert!(
        UiIntent::OfferPurchase(offer)
            .session_actions(&purchase, 42)
            .is_none()
    );
    let target = plain.selected().unwrap();
    let rename = plain.clone().with_dialog(PendingDialog::Rename {
        target: target.clone(),
        name: "New".to_owned(),
    });
    let actions = UiIntent::ConfirmRename {
        target: target.clone(),
        name: "New".to_owned(),
    }
    .session_actions(&rename, 42)
    .unwrap();
    assert!(matches!(&actions[0], Action::VirtualSetRename { name } if name == "New"));
    assert!(matches!(&actions[1], Action::VirtualRenameFish { id: 0, name } if name == "New"));
    tank.fish[0].name = "Replacement".to_owned();
    let stale = snapshot(&tank).with_dialog(PendingDialog::Rename {
        target: target.clone(),
        name: "New".to_owned(),
    });
    assert!(
        UiIntent::ConfirmRename {
            target,
            name: "New".to_owned()
        }
        .session_actions(&stale, 42)
        .is_none()
    );
    // Bulk actions use the live production inventory, including non-grid offspring.
    let mut tank = original.clone();
    let mut child = tank.fish[0].clone();
    child.id = 100;
    child.breeder_child = true;
    child.name = "Child".to_owned();
    tank.fish.push(child);
    assert!(tank.validate().is_ok());
    let snapshot = snapshot(&tank);
    let actions = UiIntent::SetAllFishShown {
        targets: roster(&snapshot),
        shown: false,
    }
    .session_actions(&snapshot, 42)
    .unwrap();
    assert_eq!(actions.len(), 2);
    assert!(matches!(
        actions[0],
        Action::VirtualShowFish {
            id: 0,
            shown: false
        }
    ));
    assert!(matches!(
        actions[1],
        Action::VirtualShowFish {
            id: 100,
            shown: false
        }
    ));
    assert_eq!(plain.tank(), &original);
}
