# Options and pause modal frontend

Actionable #303 owns `src/options_modal.rs`, `src/app/options_runtime.rs`,
`tests/options_modal.rs` and this report under the
[modal contract](modal-batch-303-307.md). Root owns production routing, shared
controls/drawing/assets and all settings, save, session and clock transactions.

## Design and evidence boundary

The controller is transient and accepts immutable current settings/capabilities
plus a caller-supplied modal context identity. It owns hit geometry, keyboard
focus, captured slider previews and identified setting requests. It never
changes a backend, writes a save, advances a clock or reconstructs a session.
Settings are displayed from the caller's committed state. Only the existing
`GameSettings::change` and settings writer may commit a request; failures return
to the still-open dialog. Each open/scene/profile boundary needs a new context
identity so obsolete captures and transaction feedback cannot affect it.

Pause has Resume, Options, Help and Main Menu controls. Options opened from
pause return to that pause view; Escape from pause requests Resume. Selector
Options keep Help, Stories and the existing Saver entry. Saver handling remains
with its existing caller. Main Menu and Help are explicit caller intentions;
the controller cannot discard the retained game or restore pause itself.

Buttons and toggles activate on captured release or focused accept through
`ModalControls`. Volumes preview while captured and emit one bounded setting
request on release, including release outside the rail. Keyboard Left/Right
adjust five percentage points; Home/End select zero/one. This commit-on-release
policy is a deliberate project adaptation to avoid repeated durable writes;
W1 reports volume changes during drag as well as at release. Focus loss,
cancellation, context mismatch and capability removal retire previews.
Hardware acceleration is disabled because the runtime has no corresponding
backend operation. Audio controls follow actual backend capabilities.

Drawing uses root's owned dialog/button/check/slider helpers and bitmap fonts.
The project layout reserves separate settings/status/action rows within the
640x480 canvas; it is source-informed rather than a retail pixel-parity claim.
No proprietary dialogue, source body, image, font or analysis output is tracked.

## Qualified source facts

Read-only W1 revision `f919b3c241cfd611c547f1653fb3514f500f761b`, from the
registered private reference checkout, was read back on 2026-10-11
(America/Phoenix). Existing installed-binary corrections and Rust transaction
authorities take precedence. [Provenance](provenance.md#runtime-sources) owns
the recovered-game/framework licensing and source-informed study boundary.

| Fact | Pinned source locator | Class and limits |
|---|---|---|
| Two volume controls clamp to 0..1; fullscreen/custom cursor checkboxes reflect existing state | `source/WinFish/OptionsDialog.cpp:13-48`, constructor | Secondary-source-derived; hardware toggle has no project backend |
| Main-menu return depends on entry context | `OptionsDialog.cpp:43-44,259-265`, constructor/`ButtonDepress` | Secondary-source-derived; Rust retains its established mode/save owner |
| Music/effects labels and checkbox labels use an outline font | `OptionsDialog.cpp:105-116`, `Draw` | Secondary-source-derived; authored layout/text use owned resources |
| Dialog layout groups volume rows, toggles and footer actions | `OptionsDialog.cpp:119-185`, `Resize`/`GetPreferredHeight` | Secondary-source-derived; `Resize` is marked AI Generated upstream, not recovered retail geometry |
| Help and menu are separate actions; sliders apply backend gain | `OptionsDialog.cpp:219-265`, `SliderVal`/`ButtonDepress` | Secondary-source-derived; commercial registration/web/update actions are excluded |
| Horizontal drag clamps and reports release even outside original track | `source/SexyAppFramework/Slider.cpp:79-168`, mouse methods | Secondary-source-derived; transactional release policy is project adaptation |
| Dialog/button/check images and font roles are centralized | `source/WinFish/WinFishCommon.cpp:13-21,38-50,88-94` | Secondary-source-derived; root loads owned resources at runtime |
| Options creation and application clear the update backlog | `source/WinFish/WinFishApp.cpp:1860-1866,2036-2046` | Secondary-source-derived; root preserves existing clock reset and setting transactions |

## Implementation and verification

The original Rust model, concrete view geometry and owned-art runtime adapter
are implemented. Fifteen focused offline contracts, formatting, strict affected
target lint and locked/offline build passed against root's shared core
`9d29e1e`. Only exact own-module worker-local registrations were used, with a
temporary dead-code allowance on the unjoined runtime adapter. Both
registrations were restored before staging; root still owns production joins.

Checked build: Rust `1.95.0` / host `x86_64-pc-windows-msvc`, debug profile,
locked dependencies, two Cargo jobs, read-only shared libopenmpt `0.8.9`
amd64 package, private target directory. The changed inputs were the three
owned Rust files and the two temporary registrations. All 91 maintained
Rust/Cargo inputs were recorded in private `corrected02-Full-identity.json`;
test/configuration data are in-memory epoch303, synthetic24px thumb,
initial volumes0.5, no RNG or elapsed-time input. Supporting executable SHA256:
`DB403E8011C661FC14A3C8DD70E8C63F51CAEA366B0DB2B7769A7AE9A97C6E32`.
It was compiled, never launched. Original-game execution/version comparison
was not performed for this batch; existing PB05 identity remains registered.

Exact successful commands, run in the worker checkout through validated
task-specific PowerShell with the shared package's import-library/runtime
directories supplied only to that process:

```text
rustfmt --edition 2024 src/options_modal.rs src/app/options_runtime.rs tests/options_modal.rs
cargo fmt --all -- --check
cargo test --locked --offline --jobs 2 --test options_modal
cargo clippy --locked --offline --jobs 2 --lib --bin turbofish-deluxe --test options_modal -- -D warnings
cargo build --locked --offline --jobs 2 --lib --bin turbofish-deluxe
```

Focused checks exercise matching captured release, coalesced press/release,
thumb offset and clamp/preview/release, capability removal, disabled actions,
keyboard focus/volume endpoints, cancellation/focus loss/missed release, stale
context/settings, duplicate/pending/stale transaction feedback, a failing then
successful actual `GameSettings::change` backend call, pause/options/help/menu
intentions, retained selector story/saver entries and nonoverlapping bounded
control/text/status geometry. These prove the explicit control/transaction
contracts and qualified source-informed model; they do not prove retail
fidelity or production integration.

`scripts/Test-Workflow.ps1` also passed35 required files,103 documents,1,073
local links,45 allowlist and37 exclusion cases. It checks workflow only.
Private `corrected02-{format,tests,lint,build}.log` retains final check output;
source/build identities stay in the worker's ignored `.scratch/options303`.

## Failures and corrections

- Initial formatting stopped at the temporary `lib.rs` registration's module
  order; no compile/tests ran. Moved only that registration into alphabetical
  order, then the first full gate passed14 contracts. Private
  `initial01-format.log` is retained.
- A bounded coalesced-input counterexample exposed Tab plus mouse press arming
  a slider after shared controls had already handled traversal. The new test
  failed with preview0.0 versus committed0.5. The controller now returns
  immediately after traversal, retiring capture without starting a drag. The
  retained regression passed in the final15-case gate. Exact failed command:
  `cargo test --locked --offline --jobs 2 --test options_modal
  focus_traversal_cannot_arm_a_slider_from_same_frame_pointer_events`;
  private `traversal-before-fix.log` preserves failure101. Earlier successful
  input-check evidence is historical where that correction affects it.
- Read-only startup lookup used unsupported `Show-TextRange` StartLine/EndLine
  flags and two nonexistent source/build-script paths. Corrected to the
  documented `Range`, `app/menu_runtime.rs` and `scripts/Test-Runtime.ps1`.
  No requested mutation followed those failed lookups.
- The first Actionables transition supplied an unsupported `workItemId` key;
  schema validation rejected it before mutation. The corrected tool call
  succeeded. Ordinary claim/research/Ready/In progress flow is working for
  #303; no tracking outage, denied retry or workaround is claimed.

## Tracking and resource handoff

Actionable #303 is In progress with its worker claim retained for root's
prompt production join verification. Worker source acceptance is complete;
qualifying completion/Resolution remains pending that verification. No native
process, lease, service, database, port, human-owned resource or original/
real/frozen/earned save was used. The private Cargo target and immutable logs/
identity files are retained for audit. No live helper requires cleanup.

## Exact parent API and required joins

Register `pub mod options_modal;` in `src/lib.rs` and
`#[path = "app/options_runtime.rs"] mod options_runtime;` in `src/app.rs`.
Every production join remains root-owned.

- `OptionsRuntime::options(OptionsContext::Selector, epoch)` opens selector
  Options; `OptionsRuntime::pause(epoch)` opens pause. Direct pause Options can
  use `OptionsContext::Paused`. The epoch must be fresh for each opening,
  profile/scene replacement and lifetime boundary.
- `options_runtime::facts(&presentation, settings.settings, backend.capabilities(),
  epoch)` supplies immutable `OptionsFacts`, including the loaded owned
  `IMAGE_SLIDERTHUMB` width. Input travel is the whole widget width minus that
  exact width, matching `modal_runtime::draw_slider`; grabbing the thumb retains
  the pointer offset. No texture means sliders are disabled.
- `runtime.input(&modal_input, facts) -> Option<OptionsIntent>` consumes already
  normalized events once. `runtime.view(facts)` owns all hit/paint/label/value/
  status geometry. `runtime.draw(&presentation, facts)` paints owned art;
  parent paints one dim layer. `runtime.page()` distinguishes Pause/Settings.
- `OptionsIntent::Setting(request)` carries `context_epoch`, monotonic
  `sequence`, `previous: GameSettings` and `change: SettingChange`. Check current
  epoch and exact previous settings before running the existing real backend
  application/settings-save transaction. Preserve its rollback behavior on
  writer failure. Then call `runtime.finish_setting(request, Result<(), String>)`.
  Only the exact pending request is accepted; committed display comes from the
  next facts, and errors stay in the dialog. Pending requests suppress all
  additional setting/navigation actions.
- `Close` returns selector Options to its existing caller. In paused Options,
  Back/Escape changes the model to Pause and emits no external intention;
  Escape on Pause emits `Resume`. `Resume` uses existing `ModalPause` restoration,
  input-hold cancellation and backlog reset. `Help` uses current Help return and
  pause restoration contracts. `MainMenu` uses the existing transient selector
  over the retained session. Neither modal moves, recreates, saves nor discards
  a Board/session itself.
- `Stories` and `Screensaver` occur only in selector Options; route them through
  the existing story/saver callers. Do not replace excluded saver input/drawing
  or introduce new saver work. `runtime.set_error(message)` retains navigation
  failures. `runtime.cancel_capture()` retires slider/button holds when parent
  suspends the adapter.
- While active, bypass old Options press-only polling/drawing and paused keyboard
  hint overlay; prevent behind-modal feeding, firing, scene keys and repeated
  Escape processing. Keep current pause/Help/song/clock/checkpoint ownership.

No native game, input, rendering, audio or human playtest result is claimed.
Full modal/runtime acceptance requires root's actual production input/render/
assets/clock/save joins and focused joined verification.
