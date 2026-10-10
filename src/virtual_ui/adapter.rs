use super::*;
use crate::sim::Action;

impl UiIntent {
    /// Maps an intention to the existing session actions without executing them.
    /// Dispatch the returned batch in order, without interleaving another input.
    /// The session still validates phase, calendar, balance, stock and ownership.
    pub fn session_actions(
        &self,
        snapshot: &VirtualUiSnapshot,
        day_start_seconds: u64,
    ) -> Option<Vec<Action>> {
        if snapshot.external_block {
            return None;
        }
        let now_seconds = snapshot.now_seconds;
        if snapshot.dialog.is_some() {
            let action = match (self, snapshot.dialog.as_ref()?) {
                (Self::SetDialogName(name), PendingDialog::Purchase { target, .. })
                    if name.len() <= 64 && matches!(target.product, StoreProduct::Fish(_)) =>
                {
                    Action::VirtualSetPurchaseName { name: name.clone() }
                }
                (Self::SetDialogName(name), PendingDialog::Rename { .. }) if name.len() <= 64 => {
                    Action::VirtualSetRename { name: name.clone() }
                }
                (
                    Self::ConfirmPurchase { target, name },
                    PendingDialog::Purchase {
                        target: pending, ..
                    },
                ) if target == pending
                    && target.cycle == day_start_seconds
                    && offer_state(snapshot, target) == OfferState::Available =>
                {
                    let mut actions = Vec::new();
                    if matches!(target.product, StoreProduct::Fish(_)) {
                        let name = name.as_ref().filter(|name| valid_name(name))?;
                        actions.push(Action::VirtualSetPurchaseName {
                            name: name.trim().to_owned(),
                        });
                    }
                    actions.push(Action::VirtualConfirmPurchase {
                        accept: true,
                        now_seconds,
                        day_start_seconds,
                    });
                    return Some(actions);
                }
                (
                    Self::ConfirmRename { target, name },
                    PendingDialog::Rename {
                        target: pending, ..
                    },
                ) if target == pending && target.matches(snapshot) && valid_name(name) => {
                    return Some(vec![
                        Action::VirtualSetRename {
                            name: name.trim().to_owned(),
                        },
                        Action::VirtualRenameFish {
                            id: target.0.id,
                            name: name.trim().to_owned(),
                        },
                    ]);
                }
                (Self::ConfirmFishSale(target), PendingDialog::FishSale { target: pending })
                    if target == pending
                        && target.matches(snapshot)
                        && snapshot.sale_value(&target.0).is_some() =>
                {
                    Action::VirtualSellFish {
                        id: target.0.id,
                        accept: true,
                        now_seconds,
                    }
                }
                (
                    Self::ConfirmBackdropSale { index },
                    PendingDialog::BackdropSale { index: pending },
                ) if index == pending && can_sell_backdrop(snapshot, *index) => {
                    Action::VirtualSellBackdrop { accept: true }
                }
                (Self::DismissDialog, PendingDialog::Purchase { target, .. })
                    if target.cycle == day_start_seconds =>
                {
                    Action::VirtualConfirmPurchase {
                        accept: false,
                        now_seconds,
                        day_start_seconds,
                    }
                }
                (Self::DismissDialog, PendingDialog::FishSale { target }) => {
                    Action::VirtualSellFish {
                        id: target.0.id,
                        accept: false,
                        now_seconds,
                    }
                }
                (Self::DismissDialog, PendingDialog::BackdropSale { .. }) => {
                    Action::VirtualSellBackdrop { accept: false }
                }
                // Existing OpenMenu safely exits an expired purchase; it returns to the tank.
                (
                    Self::DismissDialog,
                    PendingDialog::Rename { .. } | PendingDialog::Purchase { .. },
                ) => Action::OpenMenu,
                _ => return None,
            };
            return Some(vec![action]);
        }
        let action = match self {
            Self::OfferPurchase(target)
                if offer_state(snapshot, target) == OfferState::Available =>
            {
                Action::VirtualStoreOffer { slot: target.slot }
            }
            Self::SelectFish(id) if FishTarget::capture(snapshot.tank(), *id).is_some() => {
                Action::VirtualSelectFish { id: *id }
            }
            Self::SetFishShown { target, shown } if target.matches(snapshot) => {
                Action::VirtualShowFish {
                    id: target.0.id,
                    shown: *shown,
                }
            }
            Self::SetAllFishShown { targets, shown }
                if !targets.is_empty() && *targets == roster(snapshot) =>
            {
                // Match production set_all_fish_shown: include live breeder children as well.
                return Some(
                    snapshot
                        .tank
                        .active_fish()
                        .map(|fish| Action::VirtualShowFish {
                            id: fish.id,
                            shown: *shown,
                        })
                        .collect(),
                );
            }
            Self::BeginRename(target) if target.matches(snapshot) => {
                Action::VirtualBeginRename { id: target.0.id }
            }
            Self::BeginFishSale(target)
                if target.matches(snapshot) && snapshot.sale_value(&target.0).is_some() =>
            {
                Action::VirtualOfferFishSale { id: target.0.id }
            }
            Self::BeginBackdropSale { index } if can_sell_backdrop(snapshot, *index) => {
                Action::VirtualOfferBackdropSale
            }
            Self::SwitchBackdrop { direction }
                if matches!(*direction, -1 | 1)
                    && snapshot
                        .tank
                        .owned_backdrops
                        .get(usize::from(snapshot.tank.backdrop))
                        == Some(&true)
                    && snapshot
                        .tank
                        .owned_backdrops
                        .iter()
                        .filter(|owned| **owned)
                        .count()
                        > 1 =>
            {
                Action::VirtualSwitchBackdrop {
                    direction: *direction,
                }
            }
            Self::SetSettings(settings)
                if [
                    Setting::FishNames,
                    Setting::Bubbulator,
                    Setting::AlienAttractor,
                    Setting::DropShells,
                    Setting::ShowHungry,
                ]
                .iter()
                .any(|setting| {
                    setting.available(snapshot.tank())
                        && *settings == setting.toggled(&snapshot.tank.settings)
                }) =>
            {
                Action::VirtualSetSettings {
                    settings: settings.clone(),
                }
            }
            Self::ReturnToTank => Action::Continue,
            _ => return None,
        };
        Some(vec![action])
    }
}
