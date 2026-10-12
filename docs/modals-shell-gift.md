# Shell gift modal completion

Actionable #307 owns `src/app/shell_gift_runtime.rs` and this report under the
[modal contract](modal-batch-303-307.md). Root owns production joins, shared
controls/drawing, resources, saves and central records. Source completion does
not establish full modal acceptance or retail fidelity.

## Design and evidence

Read-only W1 was verified at `f919b3c241cfd611c547f1653fb3514f500f761b`.
The registered [provenance](provenance.md) boundary applies. The following facts
are **secondary-source-derived**, not installed-binary or gameplay observations:

| ID | Pinned source | Functional fact |
|---|---|---|
| G1 | `source/WinFish/WinFishApp.cpp:2699-2712`, `DoGiveDialog` | Shell-transfer UserDialog uses a 400px preferred width. |
| G2 | `source/WinFish/UserDialog.cpp:14-69`, constructor | Dialog/button images; centered Liddie12 list with 15px rows; edit shown and rename/delete hidden for transfers; scrollbar hidden below eight profiles. |
| G3 | `source/WinFish/UserDialog.cpp:78-106`, Draw/AddedToManager | Shell image beside owned edit box; initial keyboard focus belongs to amount edit. |
| G4 | `source/WinFish/UserDialog.cpp:118-140`, Resize/GetPreferredHeight | List height 130px, 10px horizontal inset, optional 16px scrollbar; amount near bottom buttons; extra preferred height 190px. |
| G5 | `source/WinFish/WinFishCommon.cpp:38-90`, dialog/edit defaults | Content insets 36px sides, 15px top, 36px bottom; outlined JungleFever header/button/body fonts; Liddie12 edit font. |
| G6 | `source/SexyAppFramework/Dialog.cpp:289-307`, Resize | Paired bottom actions with an 8px gap and equal available widths. |

Adapt functional facts into original Rust using caller-owned images/fonts and
the shared controls. Extend the panel height to retain explicit requested debit,
actual debit, recipient credit, cap loss and scrollable full error text. Keep
project-authored labels and instructions. Preserve the existing Rust donor gate,
self-transfer choice, strict whole amount admission, stable IDs, displayed
preview/revalidation and parent-owned atomic transaction. These deliberate
policies override weaker secondary parsing and name-based lookup.

Shared control IDs identify seven fixed controls; recipient capture separately
retains the exact profile ID. List reorder, removal or scrolling cannot retarget
a release. Opening/focus-loss/blocked/pending frames retire captures and require
a neutral frame before input resumes. No native polling, graphics context or
save write is needed to exercise the model/view.

## Parent API and joins

| API | Caller contract |
|---|---|
| `ShellGiftRuntime::open(&Profiles) -> Result<Self, ShellGiftError>` | Captures donor ID and starts amount focus; opening input needs a neutral frame. |
| `ShellGiftInput::from_modal(ModalInput) -> ShellGiftInput` | Projects one root-normalized frame, retaining whole text/overflow. Root may add wheel, Page keys, Ctrl+A keys/control and blocked modality. Never collect text a second time. |
| `input(&Profiles, &ShellGiftInput) -> ShellGiftFrame` | Consumes every open/closing frame; returns `Confirm(ShellGiftRequest)` or `Cancel`. Shared controls own capture/focus; list capture retains recipient ID. |
| `view(&Profiles) -> ShellGiftRuntimeView` | Fresh production gift data, complete `ShellGiftLayout`, and `first_row`; geometry stays in this view. View creation alone does not approve a preview. |
| `draw(&Presentation, &Profiles, Vec2)` | Uses shared panel/title/body/button/edit skin and owned Liddie12 list/edit text. Root must draw one dim layer. This submits/clips accounting before acknowledging the displayed preview. |
| `confirmation_dialog() -> Option<&mut ShellGiftDialog>` | Available only after an admitted confirmation request; root confirms against live Profiles/path under existing writer lease, outside Session borrow. |
| `confirmation_finished(Result<ShellGiftPreview, ShellGiftError>)` | Feed exact owner/controller result back; success closes, failure stays open with full scrollable error and neutral rearming. |
| `is_open()` / `cancel()` | Existing lifecycle remains compatible. Cancel writes nothing. |

Root's required joins: retain selector give admission, opening/closing input
consumption and clock freeze; project its single poll to `from_modal`; preload
`IMAGE_DIALOG`, `IMAGE_DIALOGBUTTON`, `IMAGE_EDITBOX`, `IMAGE_HELPSHELL`, `Liddie12`
and the existing `JungleFever10outline/12outline/15outline` font stems; dim once;
retain caller-owned confirmation/write lease and refresh from committed Profiles.
No save schema, wallet authority, gameplay clock or saver caller changes.

The amount edit uses the shared blank edit skin, then draws Liddie12 text with
its own cursor viewport. This prevents a duplicate end caret from the shared
helper and keeps the final digits of a long leading-zero amount visible. Full
amount input remains strict and untruncated. The eight-row list uses owned dialog
buttons for scrolling; exact scrollbar skin and retail offsets remain deferred.

Temporary compatibility need at base `9d29e1e`: `app.rs` still calls legacy
`ShellGiftInput::poll`. That method must remain for the initial compiling handoff
and be removed in an owned follow-up after root publishes the normalized caller.
Root explicitly permitted a narrow temporary `allow(dead_code)` on that method
so the normalized caller can join and pass lint before its immediate removal.
It is not an additional simultaneous character drain. Human gameplay and exact
owned-art/native/audio/retail comparison remain unperformed.

## Verification and handoff

Checkout: worker worktree `35f7`, reconciled to shared-core `9d29e1e` with only
the owned module/report changed. Toolchain `rustc1.95.0 (59807616e)`,
`cargo1.95.0 (f2d3ce0bd)`, `x86_64-pc-windows-msvc`; private Cargo target under
`.scratch/modal307/target`, two jobs, locked/offline dependencies and read-only
libopenmpt0.8.9 amd64 import library/DLL directory. No window, audio, game process,
real/frozen/earned save, E80 lineage or installed asset was tested or changed.

Commands executed through task-specific PowerShell scripts validated by the host
parser/automatic-variable wrapper:

```text
rustfmt --edition 2024 src/app/shell_gift_runtime.rs
cargo fmt --all --check
cargo test --offline --locked --lib app::shell_gift_runtime::tests::
cargo clippy --offline --locked --lib --bin turbofish-deluxe -- -D warnings
cargo clippy --offline --locked --lib --tests -- -D warnings
cargo build --offline --locked --lib --bin turbofish-deluxe
scripts/Test-Workflow.ps1
git diff --check
```

Initial source-check results: eight focused tests passed; formatting, strict
production/test Clippy, locked/offline debug build and workflow checks passed.
Workflow verified103 documents/1073 local links,45 allowlist checks and37 private
path exclusions. Tests cover G1-G6 layout bounds/amount focus;
stale recipient release/rename/deletion; disabled actions, opening and focus/
modality loss; whole malformed/oversized paste and UTF8 cursor editing; changed/
unpainted accounting, pending feedback and simultaneous edit/Give; full long error
wrapping/cap visibility and amount viewport; actual caller atomic failure and
successful current-format save reload. Preview acknowledgement and 8px font
metrics in offline tests are explicit simulations, not GPU/font/pixel proof.

The first eight-test run passed, but cleanup inspection found an intentionally
failed directory-target publication left its `.pending` sibling outside the
disposable fixture directory. Preserved the exact task-owned file/log under
ignored `.scratch/modal307/retained-failures` and `verify-test-01.log`; changed the
invalid target to a child directory so both it and the pending sibling stay
inside the disposable owner. Reran the focused suite: eight passed; fixture root
empty. This corrects test isolation only, not the production writer or domain.

Final initial source identity is base
`9d29e1e622f79909ae81bf02b2d4f2e842e08245` plus the owned module/report. Module
SHA-256 `84E70128F37E342FCD2C0AD09CF9DDA5D6BDB3E9CB8D19825EA9D3C2EF8D32C8`;
debug runtime SHA-256
`F629BE5EFCF895A4EE81DC43F446CB0A34640CCF2E512BD598F2D79901D0C226`;
offline test executable SHA-256
`A50E3CECCCB711965D006D73FC53D48B5E2E85DA988ABAB8F5BA699084F6C400`.
The test body was checked before the later compatibility lint annotation;
affected lint/build/format checks were refreshed after it, without a behavioral
change. `.scratch/modal307/verify-inputs-02.local.json` retains exact shared
inputs, lockfile and log digests. No original-game executable/configuration,
gameplay RNG/input/time identity or runtime observation exists for these offline
checks. Fixtures use project-created seeds701 onward and remain disposable.

No compile/test/lint defect was observed. Final parent joins and tracker
completion remain root-gated; #307 stays In progress with a nonqualifying Partial
validation record and retained claim. Fixture directory is empty; ignored build
targets and first cleanup failure receipts are retained. No test helper or
native/game process remains running. Source and sanitized report are the handoff.

## Normalized caller follow-up

Initial core `6a33e669ab3d249545d6f5bb1fbeeaf901377005` was reconciled/pushed in
`befdb53ea938a917ea76e0c3d1e48a30b6afbc5e`. Root published the production joins
in `008eadb841c8edd366660f8263d82d17c139eeac`: one active normalized collector,
Help/Story/Hall draw priority, one dim layer and live confirmation/feedback outside
the Session borrow. Root reported72 focused joined checks plus strict production/
five-modal-target lint and workflow passed. Those are root-executed checks;
ordinary native gameplay/render/audio/retail comparison stays unperformed.

After fetching that parent join, removed legacy `ShellGiftInput::poll` and its
temporary lint annotation. The adapter now contains no native event collector or
clipboard access. `from_modal` is the single input projection. No temporary
`lib.rs`/`app.rs` registration or caller edits were used or left behind.

Also moved donor balance first in the existing header, using sanitized clipped
bitmap text. `Profiles::valid_name` admits256-byte names; a long name could
previously wrap the balance out of its one-line bounds. Missing donor balance
now reads unavailable instead of zero. This is a bounded presentation correction,
with no geometry, wallet, transaction, profile-name policy or parent API change.

Affected gift suite passed again: eight tests,0 failures; strict production lint
passed. Strict `--lib --tests` Clippy found an unrelated current joined-source
ordering error in `src/app/gameplay_modal_runtime.rs:22`: `mod tests` preceding
`impl GameplayModalRuntime` at126 triggered `items_after_test_module`. Preserved
the receipt and reported it to root/the file owner; did not edit that unowned
source. Remaining build/document checks continued independently. Final static
results, correction and input hashes follow after owner reconciliation.

The gameplay owner moved its test module after the implementation in
`b7c3808`, published with the report in
`01a50dcdab78f0ea29bb192d1ac7da81ca5d608f`. After fetching that mechanical
ordering correction, reran only the failed strict `--lib --tests` lint: passed.
The preceding eight gift tests, strict production lint, locked/offline debug
build, formatting and workflow checks remain applicable; no gameplay behavior
or shared API changed in that correction. Workflow checked107 documents,
1082 local links,367 anchors,45 allowlist cases and37 excluded paths. The
fixture directory remains empty. No temporary registration or native event
collector remains in the adapter.

Follow-up checked source is base
`01a50dcdab78f0ea29bb192d1ac7da81ca5d608f` plus these two owned files. Module
SHA-256 `60C355BFDAF405D65B06A2EAF6DE78FFBE193007A94AB176342EC38917FF1B8E`.
The debug runtime built at `008eadb` plus this same module has SHA-256
`1E26AF642D257FFFA339621F086800B8CE25087721C2938D367C9154517FEBBD`;
the corresponding focused test executable has SHA-256
`A0EB96EC5A5F7B78F9A65D7514CE519AF072CEB5BC1B4BA608F011DEC29E86E6`.
These executables predate the test-only ordering correction; they were not
rebuilt or launched afterward. The root owns the final isolated release build
and joined artifact acceptance. Exact shared/Cargo input and receipt hashes are
retained in ignored `.scratch/modal307/verify-inputs-04.local.json`; the failed
lint receipt remains `verify-testlint-postjoin-failed-01.log`. #307 retains its
In progress claim until that final acceptance. Native rendering, ordinary
gameplay, audio and retail fidelity are still unperformed by this worker.
