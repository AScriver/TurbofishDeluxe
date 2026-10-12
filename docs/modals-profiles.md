# Profile modal frontend

Actionables#304, bounded worker lane from `4a63144`. Root owns production
joins, resources and every real profile transaction. This report describes
source completion and focused offline verification; it is not retail or native
gameplay acceptance.

## Design and evidence

The transient controller reads profile IDs/names without cloning sessions or
writing files. Chooser selection is separate from the current live profile;
Rename/Delete target the selected stable ID. Nested forms retain their target,
edited text and pending request until the caller returns the actual transaction
result. Failed writes remain visible. Root retains the existing checkpoint,
writer lease, scene/clock reset and copied-score-name behavior.

| Claim | Evidence and scope | Classification |
|---|---|---|
| P1 | `src/profiles.rs:100-102,304-383` admits trimmed non-control names up to256 UTF-8 bytes; `transact` writes a validated draft before replacing memory. Create/select/rename/delete use stable IDs. `docs/shared-interface.md:7-11` preserves copied score names. | Strongly source-supported production authority |
| P2 | W1 `source/WinFish/UserDialog.cpp:15-61,122-137,150-190` supplies dialog/button art, current-first selection, Create entry, scroll list, selected-user Rename/Delete and confirmation. | Secondary-source-derived |
| P3 | W1 `source/WinFish/NewUserDialog.cpp:40-44,58-72` focuses the edit and submits with Enter; `WinFishApp.cpp:2797-2806` preselects the rename text. | Secondary-source-derived |
| P4 | `docs/modal-batch-303-307.md` requires captured release, focus cancellation, bounded geometry, stable-ID requests and real failure feedback. | Explicit product/control contract |

W1 pin: `f919b3c241cfd611c547f1653fb3514f500f761b`, inspected read-only
in the registered private checkout. [Provenance](provenance.md) owns licenses
and study boundaries. Only functional facts informed original Rust. No source
bodies, retail dialogue, asset bytes or derived raw output enter Git. Existing
production/binary corrections win.

Deliberate adaptations: project-authored labels and 640x480 bounded geometry;
existing Unicode/punctuation and256-byte manager admission remain supported
instead of imposing W1's12-character/alphanumeric rule. One release selects a
row; Confirm performs selection. Paging and arrow navigation replace the
framework scrollbar. Exact retail pixels, list timing and double-click behavior
remain unverified.

## Parent integration contract

Register `pub mod profile_modal` in `lib.rs` and the child
`#[path = "app/profile_runtime.rs"] mod profile_runtime` in `app.rs`.
The shared `modal_runtime` owns existing required image/font preloads; this
adapter additionally uses already loaded `ContinuumBold12` for list/edit text.
No adapter-specific image/asset loading or native polling is required.

Runtime API (all `pub(super)`):

- `ProfileRuntime::new(&Profiles) -> Self` opens chooser, initially selecting
  current stable ID or Create for an empty roster.
- `is_open() -> bool`, `cancel_capture()` and `view(&Profiles) -> Option<ProfileView>`.
- `input(&ModalInput, &Profiles) -> ProfileFrame`: `consumed` covers every open
  frame including closing frames; `intent` is `Close` or
  `Transaction(ProfileRequest { sequence, action })`.
- `draw(&Presentation, &Profiles)` draws the common owned-art panel/buttons/
  edit and source-qualified outlined list. Parent draws the dim layer once.
- `transaction_finished(&ProfileRequest, Result<(), String>, &Profiles) -> bool`
  accepts only the matching pending attempt. Failure keeps target/text and
  displays the actual error. Successful Create/Select closes; Rename/Delete
  returns to chooser with status, keeping Create available after last deletion.

`ProfileAction` payloads are `Create { name }`, `Select { id }`,
`Rename { id, name }`, `Delete { id }`. Root maps these to existing
`Profiles::{create,select,rename,delete}`. Keep the current save/checkpoint policy
and create seed `options.seed.wrapping_add(profiles.next_id)`; do not allocate
IDs in the modal. Return the exact request and actual result before dropping or
resetting controllers. Profile-switch scene/HUD/input/song/clock resets remain
in root's existing success branch, outside a live Session borrow.

Exact joins: open for `DialogIntent::ChooseProfile` and empty-profile startup;
route only normalized active-modal input before legacy selector/Board input;
consume opening/closing frames and freeze retained scenes; route requests at
the existing caller-owned transaction boundary; feed results even when the
checkpoint preceding Select/Create fails; draw the adapter instead of the
old `SelectorPanel::Profiles`/nested-form paint path; cancel capture on higher
modal/focus boundaries; remove old profile-form routing after joining it.
Root owns every such `app.rs` edit. Existing saver callers remain excluded and
unchanged.

## Verification and limits

Worker source verification passed on
`9d29e1e622f79909ae81bf02b2d4f2e842e08245` plus the three new Rust files below,
this report and exact temporary own-module registrations in `lib.rs`/`app.rs`.
Temporary child registration allowed only `dead_code` for the not-yet-joined
runtime methods; all other affected lint checks remained strict. Those
registrations are restored before staging/reconciliation/commit. Root must
register/join the actual callers and run production lint without that temporary
adapter allowance. The shared core's existing temporary allowance remains
root-owned.

Environment: Windows x64/MSVC; `rustc1.95.0 (59807616e 2026-04-14)`,
`cargo1.95.0 (f2d3ce0bd 2026-03-21)`; default debug/dev profile, bounded two-job
build, locked/offline dependencies. Cargo.lock SHA256
`1643FD7BE00AF38DB3DA764BE8E0C03CFC427EE368DCAAB7EF17A49B187F153F`.
Read-only registered libopenmpt0.8.9 supplies linking/DLL search; no package
installation or native playback occurs. Private target/fixtures/logs are under
`.scratch/modal-profiles/` in this worktree.

| Checked input | SHA256 |
|---|---|
| `src/profile_modal.rs` | `0F24E9ECBE07163676BBE0ECB61B89D043D65BB1FA6C7E4D9F753BDA5CB5FE27` |
| `src/app/profile_runtime.rs` | `2C2EC5E299D058A74EA8420C060879BD7DCEB595C1696A7B5AAF33ACBA60CA5B` |
| `tests/profile_modal.rs` | `5370621FC691884AD8FA447AE2FF24ED93A028E4240970BAF0906AA8636DF5D6` |

Executed through validated task-specific PowerShell scripts:

- `cargo fmt --all -- --check`: passed, final receipt
  `.scratch/modal-profiles/logs/format-20261012T055640.{txt,json}`.
- `cargo test --locked --offline --test profile_modal`:12 passed,0 failed,
  final receipt `test-20261012T055600.{txt,json}` in that private log directory.
- `cargo clippy --locked --offline --lib --bin turbofish-deluxe --test profile_modal -- -D warnings`:
  passed, final receipt `lint-20261012T055641.{txt,json}`.
- `cargo build --locked --offline --bin turbofish-deluxe`: passed,
  receipt `build-20261012T055606.{txt,json}`. Debug EXE SHA256
  `BA588445538F464338F6EEF2C234259E38F3E6D1290A07C3A7471B011F8A39B6`;
  offline test EXE SHA256
  `8213A79A599C50C606601C70F1144D9D46E7FA324102D272E930DE81C9B0BC35`.
- `scripts/Test-Workflow.ps1`: passed35 required files/103 documents,
  45 allowlist and37 exclusion checks after temporary registrations were restored.

Each final receipt records HEAD, dirty-tree identity, exact Cargo arguments
and matching before/after maintained Rust/Cargo input hashes. Tests use
synthetic default-current-format profiles, seed304 plus row index; only the
transaction case writes under its disposable private fixture. All twelve
scenarios are offline agent checks: release/focus cancellation, transient
selection, nested cancellation/empty roster, selected-ID rename, Unicode
cursor/delete/limits, disabled duplicate/stale targets, stale keyboard focus,
pending/retry/result matching, list bounds/shrink, text-window bounds and actual
failed/successful caller-owned transactions. The disk case proves failed write
keeps old bytes/session, retry preserves session/copied score name, deletion
retains earned score name, and profile3 decoding succeeds without migration.

Failures retained: first test compile used nonexistent public
`highscores::GlobalScoreKind` (private alias in `profiles.rs`) and failed E0432;
corrected test import to public `highscores::ScoreKind`, preserving production
code. Full original log `test-20261012T055133.txt` remains private. No tests ran
in that failed attempt. A read-only lookup wrongly expected
`rust-toolchain.toml`; it does not exist, so actual toolchain version commands
provide identity. A PowerShell glob passed to `rg` was replaced by a directory
query. No denied tracking operation or retry occurred: #304 successfully
claimed and followed ordinary Inbox/Researching/Ready/In progress flow.

Cleanup: disposable fixture directory removed after test, no owned process or
lease remains; private build/log evidence retained. No original installation,
human process, real/frozen/earned save, service, saver caller or OS setting was
read/changed for testing. Native input/render/audio, retail fidelity and human
feel remain unperformed. Root production integration is still required; worker
keeps claim In progress and does not record qualifying completion before root
confirmation.
