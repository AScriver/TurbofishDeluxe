# Shell gifts between project profiles

This bounded increment supplies a callable wallet transaction and transient give
dialog. The primary integration owner registers the module and joins selector
input, rendering and the existing profile writer. It does not add another wallet,
save format, current-profile selection or persistent dialog state.

## Evidence and deliberate adaptations

The existing [provenance](provenance.md#runtime-sources) boundary applies. W1 is
WinFish revision `f919b3c241cfd611c547f1653fb3514f500f761b`, read back clean during
this increment. Only already registered sources were studied:

| ID | Evidence class | Source and contract |
|---|---|---|
| SG1 | Secondary-source-derived | `source/WinFish/WinFishApp.cpp:2699-2712`, `DoGiveDialog`: require a current profile with finished Adventure or Adventure tank other than 1. |
| SG2 | Secondary-source-derived | `source/WinFish/WinFishApp.cpp:1601-1635`, `ApplyGiveShells`: check donor funds and resolve the recipient at confirmation; self-transfer closes without changing either wallet; cancellation closes without a transfer. |
| SG3 | Secondary-source-derived | `source/WinFish/ProfileMgr.cpp:567-574`, `UserProfile::AddShells`: clamp each wallet to `0..=9_999_999`. Recipient capacity does not reduce the requested donor debit. |
| SG4 | Project policy | Use stable profile IDs, strict positive whole-number amounts bounded by the wallet cap, and revalidate the opener/current identity, recipient and balances against production state. No signed, fractional, exponent, overflow or malformed amount is admitted. |
| SG5 | Project policy | `Profiles::transact` commits the complete validated profile envelope before replacing memory. Only two `session.progress.shell_balance` fields may change. Error, cancellation and self-transfer do not write. Keep all session receipts and the existing normal-writer ownership contract. |
| SG6 | Project policy | Preview and result expose requested debit, actual donor debit, recipient credit and capped excess. Show capacity loss before confirmation; never describe the full debit as recipient credit. |

SG1-SG3 have not been checked against the installed executable or an original-game
run. Source-informed Rust and project-authored labels do not establish retail
layout, text, entry/input fidelity or native acceptance. `UserDialog` was not
studied, and no upstream source or install-derived assets/prose are incorporated.
Zero-amount rejection, strict text admission and atomic single-envelope
persistence are explicit project choices, not recovered original dialog rules.

## Implementation contract

Use the existing authoritative `Profiles` and its current project profile path.
The normal app must keep its established exclusive writer lease for the entire
operation. This module does not claim interprocess locking, compare-and-swap or
power-loss durability beyond the existing profile writer. Do not save an old app
session over the manager after a successful transfer.

Session validation remains authoritative. A wallet change that would invalidate
a retained result receipt must fail atomically rather than editing the receipt,
histories or other session fields to make it fit. The app can defer such a gift
until the affected profile leaves that result state.

The dialog stores only the opener's stable ID, selected recipient ID, bounded
amount text, transient open/error state. Views copy current production names and
balances for display; they are never wallet authorities. Confirmation resolves
the current production entries again, including after rename, reorder, deletion,
selection changes or a change in available funds. Self-transfer closes only after
the ordinary amount, gate and funds checks pass.

## Integration and checks

The new [module](../src/shell_gift.rs) exposes free functions; it does not add an
inherent implementation to an external profile type:

| API | Purpose |
|---|---|
| `gift_dialog_available(&Profiles)` | Source-qualified gate for a current production manager. |
| `parse_shell_amount(&str)` | Bound text length to 32 bytes and admit positive ASCII decimal through `9_999_999`; trim surrounding whitespace, preserve leading-zero meaning. |
| `preview_shell_gift(&Profiles, ShellGiftRequest)` | Resolve stable donor/recipient IDs and current balances; return explicit debit/credit/cap accounting without saving. |
| `transfer_shells(&mut Profiles, &Path, ShellGiftRequest)` | Recheck admission, edit only two wallets in `Profiles::transact`, return committed accounting; self-transfer skips disk I/O. |
| `ShellGiftDialog::{open,select_recipient,set_amount_text,view,confirm,cancel}` | Transient controller and owned production snapshot for names, choices, amount, validation/error state, accounting and disabled confirmation. |

The preview exposes `requested_debit`, `actual_debit`, `recipient_credit` and
`capped_excess` separately. A valid self-transfer has its requested amount but
zero actual debit/credit/excess; it closes with a project-authored `Close` label.
An oversized paste clears input, records a validation error and disables giving;
it is never truncated into a different valid amount. Save failure retains the
open dialog/input with a confirmation error, so an explicit retry is possible.

Primary wiring steps:

1. Register `pub mod shell_gift;` in `src/lib.rs`. Keep one optional
   `ShellGiftDialog` in transient app state, outside serialized sessions.
2. Handle the selector worker's `OpenGiveDialog` outcome by opening the controller
   against the authoritative `Profiles`; display its gate error if unavailable.
3. Render `view(&profiles)` and handle modal input before underlying selector
   input. Select by `ShellGiftRecipient.id`, edit via `set_amount_text`, honor
   `can_confirm`, and display both validation/confirmation errors plus the full
   `capacity_notice` before the user gives capped shells.
4. On Give, call `confirm(&mut profiles, &save_dir.join("profiles.json"))` while
   retaining the existing normal-writer lease. Use returned committed accounting,
   close on success/self-transfer, retain on error. Refresh HUD/session reads
   from the authoritative manager; do not overwrite it with an old app snapshot.
5. On Cancel/Escape, call `cancel` and dismiss without any profile save. Names,
   balances and choices must come from a fresh view, not an old display snapshot.

Focused verification uses an ignored Cargo harness which reexports the real
maintained crate's `adventure`, `bonus`, `highscores`, `profiles`, `sim` and
`time_trial` modules, and includes this maintained module by path. No production
types are copied or replaced. It owns an isolated ignored target, process-local
native configuration and disposable uniquely named temp save directories; it
never opens real saves or a game process.

Native/render/input, retail comparison and human feel remain pending; this model
alone is not a playable-path claim.

### 2026-10-10 focused model verification

Observed implementation checks, America/Phoenix. Checkout was initially clean at
`d7100587fe3c16bd504ffc21c22ab66a7f3df7cf`; only this module and this document were
new maintained inputs. Tested module SHA-256:
`72C13DBBD2E2C7F5872733B950A0FCF73A11697C4A5C22BD543C23F705E0791D`.
Production profile/session/cap/score/writer and Cargo digests are retained in
ignored `.scratch/shell-gift/verify-inputs-02.local.json`. W1 source readback:
WinFishApp SHA-256
`B524A9E02D2DFE8646637A7FC6DB5DFEBE5A67712133EA06103690CE7EC94270`,
ProfileMgr SHA-256
`A1E79AC8B0F850E37918BB3FB649411AD4495A81855CBC77E02EC7E0B7868D05`.

Rust `1.95.0 (59807616e 2026-04-14)`, Cargo
`1.95.0 (f2d3ce0bd 2026-03-21)`, MSVC/debug. The harness's separate ignored lockfile
has SHA-256
`C326E48E1B9AF07C60EA3CFA0CF11224E2428C78EC21A1CAA368809ECCEB2AA4`;
the root Cargo files were unchanged. Artificial production-session fixtures use
seeds `701/702/703`, a validated tank2 selector donor, live first-stage recipients,
copied-name global history and a nonzero saver watermark. Separate checks prepare
a finished-Adventure tank1 cursor and a retained bonus result. These are explicit
data-integrity fixtures, not earned progression or native gameplay evidence.

Passed commands, invoked through parser/automatic-variable-validated task scripts:

```powershell
rustfmt --check --edition 2024 src/shell_gift.rs
cargo clippy --offline --locked --manifest-path .scratch/shell-gift/harness/Cargo.toml --all-targets -- -D warnings
cargo test --offline --locked --manifest-path .scratch/shell-gift/harness/Cargo.toml --lib shell_gift::tests::
cargo build --offline --locked --manifest-path .scratch/shell-gift/harness/Cargo.toml
```

Five focused tests passed:

- Ordinary transfer, partial recipient capacity and a full recipient; exact
  complete-manager comparison changes only two wallets, and production decode
  reads back the committed envelope.
- Malformed/negative/overflow/oversized/zero amounts, insufficient funds, missing
  recipient and foreign donor; cancellation and self-transfer preserve memory
  and byte-identical disk, with no file creation for self-transfer.
- Current snapshots after recipient rename/reorder, changed capacity/funds,
  recipient deletion and a changed current profile; IDs do not retarget another
  recipient and failed confirmation keeps the dialog open.
- An owned directory at the writer's `.pending` path forces a real persistence
  failure; both wallets and prior committed bytes remain unchanged. Removing
  that obstruction permits one successful explicit retry and save readback.
- Finished-Adventure tank1 admission, a receipt-incompatible recipient rejected
  atomically without changing the result, and a self-selection closing without
  saving.

The first validation attempt passed formatting then stopped before tests because
the fresh worktree lacked process-local `OPENMPT_LIB_DIR`. Failure log
`.scratch/shell-gift/verify-01.log` SHA-256
`A7CBEC46092F76E8661DD2DAF2ADDAAA4A927507636482849D83B257B50A1B1F`.
The next attempt used the existing official package, the maintained launcher's
offline preflight (import/five DLL hashes and notices), and exact local DLL copies
beside the isolated test executable. It passed strict lint, all five cases and
build; `.scratch/shell-gift/verify-02.log` SHA-256
`9BB6D641264062384B5E31F8A0908F51C333D5819AEA8666F16F7B5AC72D1A8A`.
Test executable `shell_gift_harness-eb58eee303280aae.exe` SHA-256
`D37FE2010846AA2C757256501FA5D20D7628FA70404EB7B708581DF4BA36E5F3`.

All disposable test directories and the deliberate pending-path obstruction were
removed by their owners; Cargo/test processes exited successfully. Ignored
harness, build artifacts and receipts remain for review. No real save, original
installation, shared service/process, OS configuration or frozen acceptance
lineage was opened or changed. Actionables was not updated: no workItemId was
supplied. Main app registration and native acceptance remain with the integration
owner.

Workflow checks passed: 35 required files, 78 documents, 852 local links,
237 heading anchors, 45 allowed/37 excluded paths and 158 eligible files.
Before any Git mutation the script validator rejected an unbraced variable before
a colon in the private commit helper. Delimiting that variable corrected the
parser error; no Git mutation occurred during the rejected attempt. Runtime
checks were unaffected and were not repeated for this helper/document edit.

### Main reconciliation and current receipt verification

Owned-file implementation commit `4d848f68f31fc74da3db154bb7ced406f2c7253d`
was reconciled with concurrent main in merge
`460efc3c2d2fdbbcab4f3e84c53afc81dfa491c8`. Incoming main changes the production
manager to profile format3 and adds `screensaver_scene_consumed`. Gift production
logic remained unchanged; the disposable fixture now sets that receipt to9 as
well as its existing claimed watermark17, so full-envelope comparisons prove
both survive ordinary/capped transfers and failed writes.

On the merge plus that one fixture change, formatting, strict isolated-harness
lint, the five wallet/save cases and build passed again. The new production
profile/session/writer inputs made those checks material; no native, broader
runtime suite or unrelated passing filters were repeated. Current module SHA-256
`A71E9B5634B4C2158ABBEE4411975B8F745618214495F53BCFE4FF7978F576CE`;
`.scratch/shell-gift/verify-inputs-03.local.json` owns the updated production
input digests. Current log `.scratch/shell-gift/verify-03.log` SHA-256
`96A631FEB78BDD5F72B1D2DFE498A5C52631124BACE5A1C07B1A30538A3D8C7A`;
current test executable SHA-256
`CD86F6FE3C06B2B53BB36048694DB1E4281B3B21C20E3A54C5D39C006A237EFE`.
The previous receipts remain historical. All owned temp saves were cleaned and
processes exited; module registration/app/modal/render/native joins remain the
primary owner's responsibility. The incoming selector module now defines the
agreed `OpenGiveDialog` outcome; this chunk did not edit it.
