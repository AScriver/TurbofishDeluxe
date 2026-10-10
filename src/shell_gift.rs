//! Shell gifts use the production profile manager's existing atomic envelope.
//!
//! The gate, self-transfer behavior and wallet cap are secondary-source-derived
//! from pinned W1 WinFishApp/ProfileMgr. Strict amounts, stable IDs and atomic
//! persistence are project policy; see `docs/shell-gifting.md`.

use crate::{bonus::MAX_SHELL_BALANCE, profiles::Profiles};
use std::{error::Error, fmt, path::Path};

/// Bounds transient text even when a host submits an oversized paste.
pub const MAX_AMOUNT_TEXT_BYTES: usize = 32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShellGiftError {
    NoCurrentProfile,
    DonorChanged,
    NotUnlocked,
    RecipientRequired,
    RecipientMissing,
    AmountRequired,
    InvalidAmount,
    AmountOutOfRange,
    InsufficientShells { requested: u32, available: u32 },
    InvalidProfiles(String),
    TransactionFailed(String),
    DialogClosed,
}

impl fmt::Display for ShellGiftError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoCurrentProfile => formatter.write_str("Choose a current profile first."),
            Self::DonorChanged => {
                formatter.write_str("The current profile changed. Reopen Give shells.")
            }
            Self::NotUnlocked => {
                formatter.write_str("Finish Adventure tank 1 before giving shells.")
            }
            Self::RecipientRequired => formatter.write_str("Choose a recipient."),
            Self::RecipientMissing => formatter.write_str("The chosen recipient no longer exists."),
            Self::AmountRequired => formatter.write_str("Enter a shell amount."),
            Self::InvalidAmount => formatter.write_str("Enter a positive whole number of shells."),
            Self::AmountOutOfRange => write!(
                formatter,
                "The shell amount cannot exceed {MAX_SHELL_BALANCE}."
            ),
            Self::InsufficientShells {
                requested,
                available,
            } => write!(
                formatter,
                "The gift needs {requested} shells; this profile has {available}."
            ),
            Self::InvalidProfiles(reason) => {
                write!(formatter, "The profile manager is invalid: {reason}")
            }
            Self::TransactionFailed(reason) => {
                write!(formatter, "The shell gift was not saved: {reason}")
            }
            Self::DialogClosed => formatter.write_str("The shell gift dialog is closed."),
        }
    }
}

impl Error for ShellGiftError {}

/// Project admission: positive ASCII decimal, optionally surrounded by spaces.
/// Signs, fractions, exponents, separators and overflow are rejected.
pub fn parse_shell_amount(text: &str) -> Result<u32, ShellGiftError> {
    if text.len() > MAX_AMOUNT_TEXT_BYTES {
        return Err(ShellGiftError::AmountOutOfRange);
    }
    let digits = text.trim();
    if digits.is_empty() {
        return Err(ShellGiftError::AmountRequired);
    }
    if !digits.bytes().all(|digit| digit.is_ascii_digit()) {
        return Err(ShellGiftError::InvalidAmount);
    }
    let mut amount = 0_u32;
    for digit in digits.bytes() {
        amount = amount
            .checked_mul(10)
            .and_then(|value| value.checked_add(u32::from(digit - b'0')))
            .filter(|value| *value <= MAX_SHELL_BALANCE)
            .ok_or(ShellGiftError::AmountOutOfRange)?;
    }
    validate_amount(amount)?;
    Ok(amount)
}

fn validate_amount(amount: u32) -> Result<(), ShellGiftError> {
    if amount == 0 {
        Err(ShellGiftError::InvalidAmount)
    } else if amount > MAX_SHELL_BALANCE {
        Err(ShellGiftError::AmountOutOfRange)
    } else {
        Ok(())
    }
}

fn validate_donor(profiles: &Profiles, donor_id: u64) -> Result<(), ShellGiftError> {
    profiles
        .validate()
        .map_err(ShellGiftError::InvalidProfiles)?;
    let donor = profiles.current().ok_or(ShellGiftError::NoCurrentProfile)?;
    if donor.id != donor_id {
        return Err(ShellGiftError::DonorChanged);
    }
    let progress = &donor.session.progress;
    if !progress.adventure_completed && progress.tank == 1 {
        return Err(ShellGiftError::NotUnlocked);
    }
    Ok(())
}

/// Menu admission only; confirmation performs the same checks afresh.
pub fn gift_dialog_available(profiles: &Profiles) -> bool {
    profiles
        .current_id
        .is_some_and(|id| validate_donor(profiles, id).is_ok())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShellGiftRequest {
    /// Stable ID captured when the dialog opened, not a list index or name.
    pub donor_id: u64,
    pub recipient_id: u64,
    pub amount: u32,
}

/// Immutable accounting for the current production snapshot or committed gift.
/// A self-transfer has zero actual debit, credit and capped excess.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShellGiftPreview {
    pub donor_id: u64,
    pub recipient_id: u64,
    pub requested_debit: u32,
    pub actual_debit: u32,
    pub recipient_credit: u32,
    pub capped_excess: u32,
    pub donor_balance_before: u32,
    pub donor_balance_after: u32,
    pub recipient_balance_before: u32,
    pub recipient_balance_after: u32,
    pub self_transfer: bool,
}

/// Re-resolve both stable IDs and balances without mutating or saving anything.
pub fn preview_shell_gift(
    profiles: &Profiles,
    request: ShellGiftRequest,
) -> Result<ShellGiftPreview, ShellGiftError> {
    validate_amount(request.amount)?;
    validate_donor(profiles, request.donor_id)?;
    let donor = profiles.current().ok_or(ShellGiftError::NoCurrentProfile)?;
    let donor_before = donor.session.progress.shell_balance;
    if donor_before < request.amount {
        return Err(ShellGiftError::InsufficientShells {
            requested: request.amount,
            available: donor_before,
        });
    }
    let recipient = profiles
        .entries
        .iter()
        .find(|entry| entry.id == request.recipient_id)
        .ok_or(ShellGiftError::RecipientMissing)?;
    let recipient_before = recipient.session.progress.shell_balance;
    let self_transfer = request.donor_id == request.recipient_id;
    let actual_debit = if self_transfer { 0 } else { request.amount };
    let recipient_credit = actual_debit.min(MAX_SHELL_BALANCE - recipient_before);
    Ok(ShellGiftPreview {
        donor_id: request.donor_id,
        recipient_id: request.recipient_id,
        requested_debit: request.amount,
        actual_debit,
        recipient_credit,
        capped_excess: actual_debit - recipient_credit,
        donor_balance_before: donor_before,
        donor_balance_after: donor_before - actual_debit,
        recipient_balance_before: recipient_before,
        recipient_balance_after: recipient_before + recipient_credit,
        self_transfer,
    })
}

/// Change only two production wallets through `Profiles::transact`.
///
/// The caller owns the existing exclusive normal-profile writer lease and must
/// pass the authoritative profile file. A self-transfer performs no disk I/O.
/// Validation or persistence failure leaves the complete in-memory manager and
/// existing committed envelope unchanged; no result receipts are rewritten.
pub fn transfer_shells(
    profiles: &mut Profiles,
    path: &Path,
    request: ShellGiftRequest,
) -> Result<ShellGiftPreview, ShellGiftError> {
    let preview = preview_shell_gift(profiles, request)?;
    if preview.self_transfer {
        return Ok(preview);
    }
    profiles
        .transact(path, |draft| {
            let confirmed =
                preview_shell_gift(draft, request).map_err(|error| error.to_string())?;
            for entry in &mut draft.entries {
                if entry.id == confirmed.donor_id {
                    entry.session.progress.shell_balance = confirmed.donor_balance_after;
                } else if entry.id == confirmed.recipient_id {
                    entry.session.progress.shell_balance = confirmed.recipient_balance_after;
                }
            }
            Ok(confirmed)
        })
        .map_err(|error| ShellGiftError::TransactionFailed(error.to_string()))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellGiftRecipient {
    pub id: u64,
    pub name: String,
    pub shell_balance: u32,
    pub is_donor: bool,
}

/// Owned display data, rebuilt from immutable production profiles each frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellGiftView {
    pub is_open: bool,
    pub title: &'static str,
    pub amount_label: &'static str,
    pub confirm_label: &'static str,
    pub cancel_label: &'static str,
    pub donor_id: u64,
    pub donor_name: Option<String>,
    pub donor_shell_balance: Option<u32>,
    pub recipients: Vec<ShellGiftRecipient>,
    pub selected_recipient_id: Option<u64>,
    pub amount_text: String,
    pub preview: Option<ShellGiftPreview>,
    pub capacity_notice: Option<String>,
    pub validation_error: Option<ShellGiftError>,
    pub confirmation_error: Option<ShellGiftError>,
    pub can_confirm: bool,
}

/// Transient controller; never serialize it or clone a session into it.
#[derive(Clone, Debug)]
pub struct ShellGiftDialog {
    donor_id: u64,
    recipient_id: Option<u64>,
    amount_text: String,
    amount_input_error: Option<ShellGiftError>,
    last_error: Option<ShellGiftError>,
    is_open: bool,
}

impl ShellGiftDialog {
    pub fn open(profiles: &Profiles) -> Result<Self, ShellGiftError> {
        let donor_id = profiles
            .current_id
            .ok_or(ShellGiftError::NoCurrentProfile)?;
        validate_donor(profiles, donor_id)?;
        Ok(Self {
            donor_id,
            recipient_id: None,
            amount_text: String::new(),
            amount_input_error: None,
            last_error: None,
            is_open: true,
        })
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// A copied name or old row index is never used to resolve confirmation.
    pub fn select_recipient(&mut self, recipient_id: Option<u64>) {
        if self.is_open {
            self.recipient_id = recipient_id;
            self.last_error = None;
        }
    }

    pub fn set_amount_text(&mut self, text: &str) {
        if !self.is_open {
            return;
        }
        self.last_error = None;
        self.amount_text.clear();
        self.amount_input_error = if text.len() > MAX_AMOUNT_TEXT_BYTES {
            // Never retain truncated input that could become a different gift.
            Some(ShellGiftError::AmountOutOfRange)
        } else {
            self.amount_text.push_str(text);
            None
        };
    }

    fn request(&self) -> Result<ShellGiftRequest, ShellGiftError> {
        if !self.is_open {
            return Err(ShellGiftError::DialogClosed);
        }
        if let Some(error) = &self.amount_input_error {
            return Err(error.clone());
        }
        Ok(ShellGiftRequest {
            donor_id: self.donor_id,
            recipient_id: self.recipient_id.ok_or(ShellGiftError::RecipientRequired)?,
            amount: parse_shell_amount(&self.amount_text)?,
        })
    }

    pub fn view(&self, profiles: &Profiles) -> ShellGiftView {
        let donor = profiles
            .entries
            .iter()
            .find(|entry| entry.id == self.donor_id);
        let (preview, validation_error) = match self
            .request()
            .and_then(|request| preview_shell_gift(profiles, request))
        {
            Ok(preview) => (Some(preview), None),
            Err(error) => (None, Some(error)),
        };
        ShellGiftView {
            is_open: self.is_open,
            title: "Give shells",
            amount_label: "Shell amount",
            confirm_label: if preview.is_some_and(|gift| gift.self_transfer) { "Close" } else { "Give" },
            cancel_label: "Cancel",
            donor_id: self.donor_id,
            donor_name: donor.map(|entry| entry.name.clone()),
            donor_shell_balance: donor.map(|entry| entry.session.progress.shell_balance),
            recipients: profiles
                .entries
                .iter()
                .map(|entry| ShellGiftRecipient {
                    id: entry.id,
                    name: entry.name.clone(),
                    shell_balance: entry.session.progress.shell_balance,
                    is_donor: entry.id == self.donor_id,
                })
                .collect(),
            selected_recipient_id: self.recipient_id,
            amount_text: self.amount_text.clone(),
            capacity_notice: preview.filter(|gift| gift.capped_excess > 0).map(|gift| {
                format!(
                    "Giving {} deducts {} shells. The recipient receives {}; {} shells exceed the wallet limit.",
                    gift.requested_debit, gift.actual_debit, gift.recipient_credit, gift.capped_excess
                )
            }),
            preview,
            validation_error,
            confirmation_error: self.last_error.clone(),
            can_confirm: self.is_open && preview.is_some(),
        }
    }

    /// Recheck production state, persist once, and close only after success.
    /// Errors retain the dialog and input so the caller can display or retry.
    pub fn confirm(
        &mut self,
        profiles: &mut Profiles,
        path: &Path,
    ) -> Result<ShellGiftPreview, ShellGiftError> {
        let result = self
            .request()
            .and_then(|request| transfer_shells(profiles, path, request));
        match &result {
            Ok(_) => {
                self.is_open = false;
                self.last_error = None;
            }
            Err(error) => self.last_error = Some(error.clone()),
        }
        result
    }

    /// Closing has no profile/file argument, and cannot debit or save anything.
    pub fn cancel(&mut self) {
        self.is_open = false;
        self.last_error = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{adventure::AdventurePhase, highscores::ScoreKind, sim::PetKind};
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            static SEQUENCE: AtomicU64 = AtomicU64::new(0);
            let directory = std::env::temp_dir().join(format!(
                "turbofish-shell-gift-{}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&directory).unwrap();
            Self(directory)
        }

        fn path(&self) -> PathBuf {
            self.0.join("profiles.json")
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    fn fixture(path: &Path) -> Profiles {
        let mut profiles = Profiles::new("Donor", 701).unwrap();
        profiles.create(path, "Recipient", 702).unwrap();
        profiles.create(path, "Unrelated", 703).unwrap();
        profiles.select(path, 1).unwrap();
        profiles
            .transact(path, |draft| {
                let donor = draft.current_mut().unwrap();
                donor.session.progress.tank = 2;
                donor.session.progress.level = 1;
                donor.session.progress.unlocked_pets = vec![
                    PetKind::Stinky,
                    PetKind::Niko,
                    PetKind::Itchy,
                    PetKind::Prego,
                    PetKind::Zorf,
                ];
                donor.session.progress.first_stage_best_seconds = Some(42);
                donor.session.progress.shell_balance = 1000;
                donor.session.board = None;
                donor.session.phase = AdventurePhase::GameSelector;
                donor.screensaver_claimed = 17;
                donor.screensaver_scene_consumed = 9;
                draft.entries[1].session.progress.shell_balance = 20;
                draft.entries[2].session.progress.shell_balance = 77;
                draft.global_scores.record(
                    ScoreKind::Adventure,
                    1,
                    Some(1),
                    "Historical name",
                    42,
                )?;
                Ok(())
            })
            .unwrap();
        profiles
    }

    fn request(recipient_id: u64, amount: u32) -> ShellGiftRequest {
        ShellGiftRequest {
            donor_id: 1,
            recipient_id,
            amount,
        }
    }

    fn assert_unchanged(profiles: &Profiles, path: &Path, memory: &serde_json::Value, disk: &[u8]) {
        assert_eq!(&serde_json::to_value(profiles).unwrap(), memory);
        assert_eq!(fs::read(path).unwrap(), disk);
    }

    #[test]
    fn two_wallet_accounting_and_source_cap_persist_only_wallet_changes() {
        let directory = TestDirectory::new();
        let path = directory.path();
        let mut profiles = fixture(&path);
        let mut expected = serde_json::to_value(&profiles).unwrap();
        let result = transfer_shells(&mut profiles, &path, request(2, 250)).unwrap();
        assert_eq!(
            (
                result.requested_debit,
                result.actual_debit,
                result.recipient_credit,
                result.capped_excess
            ),
            (250, 250, 250, 0)
        );
        expected["entries"][0]["session"]["progress"]["shell_balance"] = 750.into();
        expected["entries"][1]["session"]["progress"]["shell_balance"] = 270.into();
        assert_eq!(serde_json::to_value(&profiles).unwrap(), expected);
        let reopened = Profiles::decode(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(serde_json::to_value(reopened).unwrap(), expected);

        profiles
            .transact(&path, |draft| {
                draft.entries[1].session.progress.shell_balance = 9_999_989;
                Ok(())
            })
            .unwrap();
        let mut expected = serde_json::to_value(&profiles).unwrap();
        let result = transfer_shells(&mut profiles, &path, request(2, 40)).unwrap();
        assert_eq!(
            (
                result.requested_debit,
                result.actual_debit,
                result.recipient_credit,
                result.capped_excess
            ),
            (40, 40, 10, 30)
        );
        expected["entries"][0]["session"]["progress"]["shell_balance"] = 710.into();
        expected["entries"][1]["session"]["progress"]["shell_balance"] = 9_999_999.into();
        assert_eq!(serde_json::to_value(&profiles).unwrap(), expected);
        assert_eq!(
            serde_json::to_value(Profiles::decode(&fs::read(&path).unwrap()).unwrap()).unwrap(),
            expected
        );
        let full = transfer_shells(&mut profiles, &path, request(2, 1)).unwrap();
        assert_eq!(
            (full.actual_debit, full.recipient_credit, full.capped_excess),
            (1, 0, 1)
        );
        assert_eq!(
            profiles.current().unwrap().session.progress.shell_balance,
            709
        );
    }

    #[test]
    fn malformed_errors_self_transfer_and_cancel_do_not_write() {
        let directory = TestDirectory::new();
        let path = directory.path();
        let mut profiles = fixture(&path);
        let memory = serde_json::to_value(&profiles).unwrap();
        let disk = fs::read(&path).unwrap();
        for text in [
            "",
            "-1",
            "+1",
            "0",
            "1.5",
            "1e3",
            "9,000",
            "10000000",
            "4294967296",
            "１２",
        ] {
            assert!(parse_shell_amount(text).is_err(), "{text}");
        }
        assert_eq!(parse_shell_amount(" 00040 ").unwrap(), 40);
        assert_eq!(parse_shell_amount("9999999").unwrap(), 9_999_999);
        for bad in [
            request(2, 0),
            request(2, u32::MAX),
            request(2, 1001),
            request(99, 1),
            ShellGiftRequest {
                donor_id: 2,
                ..request(2, 1)
            },
        ] {
            assert!(transfer_shells(&mut profiles, &path, bad).is_err());
        }
        let self_gift = transfer_shells(
            &mut profiles,
            &directory.0.join("unwritten/profiles.json"),
            request(1, 1000),
        )
        .unwrap();
        assert!(self_gift.self_transfer);
        assert_eq!(
            (
                self_gift.requested_debit,
                self_gift.actual_debit,
                self_gift.recipient_credit,
                self_gift.capped_excess
            ),
            (1000, 0, 0, 0)
        );
        assert!(!directory.0.join("unwritten").exists());
        let mut dialog = ShellGiftDialog::open(&profiles).unwrap();
        assert!(!dialog.view(&profiles).can_confirm);
        dialog.select_recipient(Some(2));
        dialog.set_amount_text("5");
        assert!(dialog.view(&profiles).can_confirm);
        dialog.set_amount_text(&"9".repeat(1000));
        assert!(!dialog.view(&profiles).can_confirm);
        assert_eq!(
            dialog.confirm(&mut profiles, &path),
            Err(ShellGiftError::AmountOutOfRange)
        );
        dialog.set_amount_text("5");
        dialog.cancel();
        assert!(!dialog.is_open());
        assert_eq!(
            dialog.confirm(&mut profiles, &path),
            Err(ShellGiftError::DialogClosed)
        );
        assert_unchanged(&profiles, &path, &memory, &disk);

        profiles.select(&path, 2).unwrap();
        assert!(!gift_dialog_available(&profiles));
        assert_eq!(
            ShellGiftDialog::open(&profiles).unwrap_err(),
            ShellGiftError::NotUnlocked
        );
    }

    #[test]
    fn dialog_uses_fresh_ids_balances_and_capacity_after_stale_views() {
        let directory = TestDirectory::new();
        let path = directory.path();
        let mut profiles = fixture(&path);
        let mut dialog = ShellGiftDialog::open(&profiles).unwrap();
        dialog.select_recipient(Some(2));
        dialog.set_amount_text("40");
        let old_view = dialog.view(&profiles);
        assert!(old_view.can_confirm);
        assert_eq!(old_view.preview.unwrap().recipient_credit, 40);
        profiles.rename(&path, 2, "Renamed recipient").unwrap();
        profiles
            .transact(&path, |draft| {
                draft.entries[1].session.progress.shell_balance = 9_999_989;
                draft.entries.swap(1, 2);
                Ok(())
            })
            .unwrap();
        let view = dialog.view(&profiles);
        assert_eq!(
            view.recipients
                .iter()
                .find(|entry| entry.id == 2)
                .unwrap()
                .name,
            "Renamed recipient"
        );
        assert!(view.capacity_notice.is_some());
        let gift = dialog.confirm(&mut profiles, &path).unwrap();
        assert_eq!(
            (gift.actual_debit, gift.recipient_credit, gift.capped_excess),
            (40, 10, 30)
        );
        assert!(!dialog.is_open());
        assert_eq!(
            profiles
                .entries
                .iter()
                .find(|entry| entry.id == 2)
                .unwrap()
                .session
                .progress
                .shell_balance,
            9_999_999
        );
        assert_eq!(
            profiles
                .entries
                .iter()
                .find(|entry| entry.id == 3)
                .unwrap()
                .session
                .progress
                .shell_balance,
            77
        );

        let mut dialog = ShellGiftDialog::open(&profiles).unwrap();
        dialog.select_recipient(Some(3));
        dialog.set_amount_text("40");
        profiles
            .transact(&path, |draft| {
                draft.current_mut().unwrap().session.progress.shell_balance = 39;
                Ok(())
            })
            .unwrap();
        let memory = serde_json::to_value(&profiles).unwrap();
        let disk = fs::read(&path).unwrap();
        assert_eq!(
            dialog.confirm(&mut profiles, &path),
            Err(ShellGiftError::InsufficientShells {
                requested: 40,
                available: 39
            })
        );
        assert!(dialog.is_open());
        assert!(!dialog.view(&profiles).can_confirm);
        assert!(dialog.view(&profiles).confirmation_error.is_some());
        assert_unchanged(&profiles, &path, &memory, &disk);
        dialog.set_amount_text("1");
        profiles.delete(&path, 3).unwrap();
        assert_eq!(
            dialog.confirm(&mut profiles, &path),
            Err(ShellGiftError::RecipientMissing)
        );
        profiles.select(&path, 2).unwrap();
        assert_eq!(
            dialog.confirm(&mut profiles, &path),
            Err(ShellGiftError::DonorChanged)
        );
    }

    #[test]
    fn failed_atomic_save_keeps_both_wallets_and_dialog_retryable() {
        let directory = TestDirectory::new();
        let path = directory.path();
        let mut profiles = fixture(&path);
        let memory = serde_json::to_value(&profiles).unwrap();
        let disk = fs::read(&path).unwrap();
        let pending = path.with_extension("pending");
        fs::create_dir(&pending).unwrap();
        let mut dialog = ShellGiftDialog::open(&profiles).unwrap();
        dialog.select_recipient(Some(2));
        dialog.set_amount_text("50");
        assert!(matches!(
            dialog.confirm(&mut profiles, &path),
            Err(ShellGiftError::TransactionFailed(_))
        ));
        assert!(dialog.is_open());
        assert!(dialog.view(&profiles).confirmation_error.is_some());
        assert_unchanged(&profiles, &path, &memory, &disk);
        fs::remove_dir(&pending).unwrap();
        let gift = dialog.confirm(&mut profiles, &path).unwrap();
        assert_eq!(
            (gift.donor_balance_after, gift.recipient_balance_after),
            (950, 70)
        );
        assert!(!dialog.is_open());
        assert_eq!(
            serde_json::to_value(Profiles::decode(&fs::read(path).unwrap()).unwrap()).unwrap(),
            serde_json::to_value(profiles).unwrap()
        );
    }

    #[test]
    fn completed_adventure_replay_gate_and_result_receipts_remain_authoritative() {
        use crate::bonus::{BonusResult, PurchaseReceipt};

        let directory = TestDirectory::new();
        let path = directory.path();
        let mut profiles = fixture(&path);
        profiles
            .transact(&path, |draft| {
                let progress = &mut draft.current_mut().unwrap().session.progress;
                progress.tank = 1;
                progress.adventure_completed = true;
                progress.adventure_completions = 1;
                progress.unlocked_pets = crate::time_trial::SUPPORTED_PETS
                    .into_iter()
                    .chain([PetKind::Presto])
                    .collect();
                let recipient = &mut draft.entries[1].session;
                recipient.progress.tank = 2;
                recipient.progress.level = 1;
                recipient.progress.unlocked_pets = vec![
                    PetKind::Stinky,
                    PetKind::Niko,
                    PetKind::Itchy,
                    PetKind::Prego,
                    PetKind::Zorf,
                ];
                recipient.board = None;
                recipient.phase = AdventurePhase::BonusResults {
                    result: BonusResult {
                        origin_tank: 1,
                        origin_level: 6,
                        earned: 20,
                        previous_balance: 0,
                        updates: 0,
                        purchase: PurchaseReceipt::new(0),
                    },
                };
                Ok(())
            })
            .unwrap();
        assert!(gift_dialog_available(&profiles));
        let memory = serde_json::to_value(&profiles).unwrap();
        let disk = fs::read(&path).unwrap();
        assert!(matches!(
            transfer_shells(&mut profiles, &path, request(2, 1)),
            Err(ShellGiftError::TransactionFailed(_))
        ));
        assert_unchanged(&profiles, &path, &memory, &disk);

        let mut dialog = ShellGiftDialog::open(&profiles).unwrap();
        dialog.select_recipient(Some(1));
        dialog.set_amount_text("1");
        assert_eq!(dialog.view(&profiles).confirm_label, "Close");
        assert!(dialog.confirm(&mut profiles, &path).unwrap().self_transfer);
        assert!(!dialog.is_open());
        assert_unchanged(&profiles, &path, &memory, &disk);
    }
}
