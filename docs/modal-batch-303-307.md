# Modal frontend completion batch

## Completion and joined verification

E125 engineering is complete on pushed source `ea9ed918240de62d69bf13ee8b5787546b2c71d2`.
All five Actionables are terminal Done with qualifying Passed validation and
released claims, independently read back on2026-10-12. Their source transfers
are closed; keep worktrees, checked reports and private evidence. Subsequent
necessary fixes use new Actionables/chats and fresh ownership reservations.

| Actionable | Checked owned source | Owning report | Done version |
|---|---|---|---|
| #303 Options/pause | `398cdab23477b24c1693f0d1978e641ac7bd89d8` | [Options](modals-options.md) | 13 |
| #304 Profiles | `b7e5c5f1a4071ef5ab8f39920489d5ba0de96191`, reconciled `d445c51` | [Profiles](modals-profiles.md) | 13 |
| #305 Quit/notices | `19bb37c6cc3cf5b467d7e32d587214905c9e9610` | [Notices](modals-notices.md) | 14 |
| #306 Gameplay | `66f70da`, Challenge loss `a6828e8`, test ordering `b7c3808`, report `01a50dc` | [Gameplay](modals-gameplay.md) | 15 |
| #307 Shell gifts | `6a33e669ab3d249545d6f5bb1fbeeaf901377005`, final `ea9ed918240de62d69bf13ee8b5787546b2c71d2` | [Gifts](modals-shell-gift.md) | 14 |

Shared core `9d29e1e` and actual parent joins `008eadb` are ancestors of the
final checked source, as are all owned source commits. The joined checkpoint
below owns caller/clock/transaction behavior. The final gift change removes
the deprecated native collector after the normalized parent join and clips a
balance-first donor header; its transaction API remains unchanged.

Root's focused `cargo test --locked --offline -j 2` checks total72 unique cases:
six shared controls,15 Options,12 profiles,11 notices,12 gameplay, four parent
writer/lifetime tests, eight gift transactions, three gameplay-adapter and one
notice-adapter case. Five integration targets are `modal_controls`,
`options_modal`, `profile_modal`, `notice_modal`, `gameplay_modal`; selected
`--lib` filters are `modal_join_tests`, `app::shell_gift_runtime::tests`,
`app::gameplay_modal_runtime::tests`, `app::notice_runtime::tests`.
Formatting and strict production/five-target Clippy passed; affected gameplay/
gift/parent checks passed after the Challenge extension, and eight gift cases
plus formatting/strict lint passed after the final gift change. Worker strict
`--lib --tests` lint passed after the mechanical test-module-order correction.
No unchanged broad suite was repeated.

`cargo build --locked --offline -j 2 --release --bin turbofish-deluxe` passed
with Rust/Cargo1.95 MSVC. Private receipt
`.scratch/coordination/modal-final-20261012-065700-release.local.json` has ID
`bd7150279145c784bd0ffa4e64b1980e56e64fac8d465e591ee2715b6683b41e`
and SHA`bd9f70c986b3e391c1682dd406d6340933bc252f553e813e5d506cd028ca1490`.
All122 maintained build-input hashes stayed unchanged during compilation;
139 source/artifact rows matched on readback. Release EXE in the private root
target has SHA`fd37c61048abf51ad3e455920866b773bff807a23f20abc0f4c12eae0a75d826`.
Source-set digest is `d421045c5d5ff978ab3fcb28b350d406d829af1c83d24aa56ee2e4e1d5593954`.
Final logs are `modal-joins-20261012-065615-*` and
`modal-final-20261012-065700-*` in coordination scratch; earlier failed logs
remain preserved. Only central records were dirty during the release build.

The joined checkpoint retains root's two failed input assumptions and format
stop. Gameplay's wrong test import was corrected; later strict unit-test lint
caught `items_after_test_module`, resolved by moving that existing test module.
The gift report retains its earlier write-failure fixture containment repair;
owned disposable fixtures are empty after cleanup. These failures produced no
new gameplay-rule claim. Tracking retained failed/Partial validations before
qualifying final Passed records; completion did not erase them.

Human ordinary gameplay remains the acceptance path. No native game/input/
render/audio comparison, retail fidelity, human-feel or full-runtime acceptance
was performed. No live/earned/frozen save or human process was inspected; the
original install and completed excluded saver paths remain preserved. Later
central-record changes do not invalidate this identified source/artifact gate.

Final workflow verification passed35 required files,107 documents,1096 local
links,379 heading anchors,45 included/37 excluded path checks and233 eligible
files. Publication checks whitespace, receipt/source hashes and remote main
readback without rebuilding unchanged code.

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

## Joined source checkpoint

Shared core is pushed at `9d29e1e`. Checked owned source commits are
`398cdab` (#303), `b7e5c5f` (#304), `19bb37c` (#305), `66f70da` (#306)
and `6a33e66` (#307), reconciled on pushed main `befdb53`. #306's additional
Challenge-loss projection is pushed at `a6828e8`. Root's production
joins are in `src/app.rs`/`src/lib.rs`: active-modal-only native collection,
owned-art drawing/resources with one parent dim layer, setting application/
save/rollback with exact request feedback, atomic profile operations after
checkpoint admission, Quit save-before-exit feedback and retry, retained
pause/help/menu lifetimes, and phase/payload-checked gameplay actions.
Game Over counters keep their existing normal tick admission; an active
gameplay prompt owns input without creating a global simulation pause.
Independent options/pause opening epochs survive a nested Quit dialog.
Existing finished saver input/painters/routes and saved game rules remain with
their previous owners.

Root ran `cargo test --locked --offline -j 2` for the five modal integration
targets and the selected `--lib` filters `modal_join_tests`,
`app::shell_gift_runtime::tests`, `app::gameplay_modal_runtime::tests` and
`app::notice_runtime::tests`:71 passed. The Challenge-loss follow-up adds one
independent immediate-admission/no-reward case; the affected root/gameplay/gift
filters passed again, bringing the checked total to72. Formatting and strict lib/bin/five-test
Clippy passed. Four parent tests exercise the real checkpoint/profile writers
on owned disposable temporary fixtures, failed-create allocation, failed
selection plus successful retry/reopen, Quit failure/retry and pause lifetime.
The first parent-test attempt passed two/failed two: its fixture incorrectly
assumed current profile sorted last and Quit retry retained Quit focus. Source
readback established current-first ordering and safe Cancel focus after a
failed save; corrected input sequences passed all four. No production rule
changed for those failures. Private timestamped check logs remain in the root
coordination scratch directory.

Challenge loss now uses the same Game Over modal with its existing immediate
Continue admission. Root removed its old flat painter and direct pointer/Enter
dispatch, retaining Challenge result screens and game rules. The affected
gate first found only formatting in the higher-priority gift draw condition;
formatting corrected it and the gate passed (`20261012-063252` logs).
Remaining bounded joins: #307's
temporary deprecated `poll` is removed after the normalized parent join is
published. Release artifact identity/workflow checks and qualifying Actionables
completion still follow. Source checks establish no native/retail/pixel/audio/
human playability claim. Owned manifest declarations for the shared seven
images plus `HELPSHELL`/`Liddie12` were read back; no asset bytes are committed.

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
