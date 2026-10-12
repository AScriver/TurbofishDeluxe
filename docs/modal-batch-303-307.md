# Modal frontend completion batch

## Shared core checkpoint

The five ordinary Inbox claims succeeded and reached In progress. Root's
`src/modal.rs` and `src/app/modal_runtime.rs` implement the reserved API;
`src/lib.rs` and the parent resource list register them. Screen/pure-model
registration remains root-owned at each handoff. A temporary private-module
dead-code allowance exists only until the actual adapters join the painter.
Six shared control contracts passed, including captured release/one emission,
drag-off, disabled/removed targets, focus/cancel retirement, enabled keyboard
traversal and empty/all-disabled input. Formatting and strict lib/bin/core-test
lint passed; no broad suite or native run. Exact checked commands are
`cargo test --locked --offline -j 2 --test modal_controls` and
`cargo clippy --locked --offline -j 2 --lib --bin turbofish-deluxe --test modal_controls -- -D warnings`.

Source W1 readback matches the registered pin. `WinFishCommon.cpp:14-71,88-96`
supports default fonts/colors, three horizontal button states, +1px pressed
translation/-2px label offset and owned checked/unchecked images.
`MoneyDialog.cpp:55-59` supports edit-box art. Installed owned
`properties/resources.xml:47-49,97-100` declares dialog/button/edit/check/slider
assets. Rendering uses the existing project image-box and clipped font helpers;
parent owns one dim layer. Root slider bounds are the whole widget; thumb left
is `x + value * max(width - thumb_width, 0)`. Controllers must align drag/value
geometry with that rule or report a narrow layout correction. Source-informed
art/control composition is not observed retail pixel/native acceptance.

E125 starts from clean main `a4897097e0d79083f459cd259d4bd41d6506b42c`.
The user explicitly requested Actionables for unfinished modal screens,
implementation in new parallel chats/worktrees where independent, and all
checked changes committed/pushed to main. The five items below are newly
created Inbox tasks. Use ordinary claim/research/Ready/In progress flow, not
the previous batch's failing bulk-preparation route. If a required tracking
tool is unavailable, retain the exact error and use the installed workflow's
explicit unavailable-tool fallback without denied retries or invented states.

## Exclusive ownership

| Actionable | Complete outcome | Worker-owned maintained files |
|---|---|---|
| #303 | Owned-art Options/pause controller and frontend | `src/options_modal.rs`, `src/app/options_runtime.rs`, `tests/options_modal.rs`, `docs/modals-options.md` |
| #304 | Owned-art chooser/create/rename/delete profile frontend | `src/profile_modal.rs`, `src/app/profile_runtime.rs`, `tests/profile_modal.rs`, `docs/modals-profiles.md` |
| #305 | Owned-art Quit and information/locked-mode frontend | `src/notice_modal.rs`, `src/app/notice_runtime.rs`, `tests/notice_modal.rs`, `docs/modals-notices.md` |
| #306 | Owned-art pet confirmation/rescue/tutorial/Game Over frontend | `src/gameplay_modal.rs`, `src/app/gameplay_modal_runtime.rs`, `tests/gameplay_modal.rs`, `docs/modals-gameplay.md` |
| #307 | Owned-art shell-transfer modal and retained existing transaction/input | `src/app/shell_gift_runtime.rs`, `docs/modals-shell-gift.md` |

Root owns shared `src/modal.rs`, `src/app/modal_runtime.rs`,
`tests/modal_controls.rs`, `src/lib.rs`, every `src/app.rs` join, assets,
clocks/pause, existing domain/settings/profile/save integration, provenance,
central records, private coordination ledger and final decisions. No worker
edits an existing source outside its exact list. Reserve any extra source/test
file explicitly first. Parent routing/input/render/resource/save joins are
required before an outcome is engineering-complete; module compilation alone
does not qualify.

All workers are sharing the repository: preserve other changes and adapt to
concurrent commits. Root will register the four pure modules and supply the
common interface below. Until registration is published, worker-local exact
temporary registration of only its own new modules in `lib.rs`/`app.rs` is
allowed for focused compilation, and must be restored before staging,
reconciliation or commit. A private scratch harness may exercise unregistered
code instead. Do not reset, stash or alter another worker's changes.

## Shared interface reservation

`crate::modal` uses the existing macroquad `Rect`/`Vec2` geometry only; its
offline controls do not read the keyboard, graphics context, session or files.
Root supplies:

- `ModalInput`: `pointer: Vec2`; boolean `pressed`, `released`, `down`,
  `focused`, `accept`, `cancel`, `next`, `previous`, `left`, `right`, `up`,
  `down_key`, `backspace`, `delete`, `home`, `end`; `text: String`.
  `Default` means no event. Root normalizes native events exactly once for the
  active modal. No simultaneous adapters drain character input or activate a
  behind-modal screen.
- `ModalButton { id: u64, bounds: Rect, enabled: bool }`.
- `ModalControls::default()`, `update(&ModalInput, &[ModalButton]) -> Option<u64>`,
  `cancel_capture()`, `set_focus(Option<u64>)`, `focus() -> Option<u64>`,
  `is_pressed(id: u64) -> bool`, `is_focused(id: u64) -> bool`.
  Activation is captured release over the original still-enabled control, or
  one accept key on current enabled focus. Focus loss/cancel retires capture;
  disabled/removed controls cannot activate. Next/previous traverses enabled
  controls. Parent/model owns cancellation, pending requests and semantic IDs.

`app::modal_runtime` supplies caller-owned drawing only, all functions
`pub(super)`: `draw_panel(&Presentation, Rect)`,
`draw_title(&Presentation, &str, Rect)`,
`draw_body(&Presentation, &str, Rect, bool)` (last flag centers text),
`draw_button(&Presentation, Rect, &str, bool, bool, bool)` (enabled/pressed/focus),
`draw_edit_box(&Presentation, Rect, &str, bool)` (focus),
`draw_checkbox(&Presentation, Rect, bool, bool, bool)` (checked/enabled/focus),
`draw_slider(&Presentation, Rect, f32, bool, bool)` (value/enabled/focus).
These use owned dialog/button/edit/slider/check images and existing bitmap
fonts; body/edit text is clipped to supplied bounds. Parent draws a dim layer
once; adapters may use existing `ui` geometry and `help_runtime` crop helpers.
Root owns required resource registration and missing-resource diagnostics.
No independent texture/backend or generic UI framework is introduced.

Workers choose concrete screen view/runtime APIs within their ownership and
report exact constructor/input/view/draw calls plus their required joins.
Keep hit geometry in their view, rather than duplicating literals in root.
Profile/setting/notice intentions must retain payloads and allow root to report
real transaction failures without dismissing the dialog. Gameplay intentions
must carry their phase identity and respect existing session admission/delays.
Shell gifts retain the existing pending-confirmation/revalidation boundary.

## Source, resource and evidence contract

Use read-only registered W1 `f919b3c241cfd611c547f1653fb3514f500f761b`, at the
reference path in the private orchestration ledger, as qualified secondary
evidence for functional layout/behavior. Existing installed-binary corrections
and project domain authorities win. Bounded source families are
`OptionsDialog`, `UserDialog`, `NewUserDialog`, `MoneyDialog`,
`WinFishCommon::DefaultDialogSettings`/button/checkbox helpers,
`WinFishApp` dialog creation and the already registered relevant Board phase
branches, plus framework Dialog/DialogButton/Slider/EditWidget/ListWidget
functional layout/input. [Provenance](provenance.md) owns the pin/license/study
boundary. Adapt facts into original Rust; do not copy C++, extracted dialogue,
game assets, raw output or proprietary screenshots into Git. Preserve existing
project-authored guidance where original prose has not been independently
admitted. Do not start broad recovery or retail execution for this batch.

Every chat owns its worktree, ignored scratch, private Cargo target and
disposable offline fixtures. Reuse the read-only libopenmpt0.8.9 package already
under the project `.scratch/native`; bounded builds may run concurrently.
No timing benchmark/native desktop lease is assigned. No game launch/input/
audio run, human process, real/frozen/earned save, E80/earning06 lineage,
original installation, service/database/port/tool install or saver work may be
changed or inspected for testing. Preserve every excluded saver call/adapter.
Ordinary gameplay remains human-owned; exact retail visual/audio/feel remains
unverified unless the human supplies evidence.

Use necessary focused contracts for capture release, focus/cancel, stale
identity, disabled actions, delayed gameplay admission, text/layout bounds and
real transaction feedback. Expected results come from a cited source fact or
the explicit preservation/control contract, not the implementation itself.
Run formatting, strict affected-target lint and locked/offline compilation.
Commit only owned maintained paths through a validated task-specific PowerShell
script, fetch/reconcile `origin/main` in the worktree, push `HEAD:main` without
force and report exact commits/checks/failures/remaining joins/cleanup.
Root checks inputs/ancestry, joins actual consumers and runs affected joined
checks. Only then may a worker record qualifying Resolution/validation and
complete/release its claim. Preserve historical reports and private failures.
