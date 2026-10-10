# Virtual Tank screen controllers

## Ownership decision

The screens read an immutable snapshot of the existing `VirtualTankState`, the
Profile shell balance and caller wall time. They return typed intentions. The
caller remains the only owner of session phases, stock generation, biographies,
fish names, wallet transactions, persistence and screensaver configuration.
Controllers own only pointer capture, hover, selection presentation and animation
clocks. No saved state or schema changes belong to this module.

This bounded increment owns `src/virtual_ui.rs`, its child modules and this note.
The starting checkout was clean at `d7821846e3dc2d6d2c32157eaf326d109195720f`.
App registration, rendering, asset loading and session integration are assigned
to the primary chat. Required native/manual/render/audio acceptance remains open.

## Source and explicit limits

The already [registered reference](provenance.md#runtime-sources) is W1
`f919b3c241cfd611c547f1653fb3514f500f761b`, read from `source/WinFish`.
`StoreScreen.cpp:26–72,133–257,264–355`,
`StoreButtonWidget.cpp:9–34,43–158,190–204`,
`SimFishScreen.cpp:745–844,923–1199,1201–1343`,
`FishButtonWidget.cpp:7–61` and `SimSetupScreen.cpp:12–60,121–297`
inform the screen responsibilities, grid, hover/press/sold feedback, selected
details and backdrop/device predicates. These rules are **secondary-source-derived**.
Installed-binary authority and all earlier contradictions remain in force.

Production `StoreProduct::price`, `VirtualFish::sale_value`,
`VirtualTankState::{next_fish_id,fish_capacity}` and current ownership/settings
remain authoritative inputs. W1's full-refund fallback for a recent fish is not
adopted over the existing production sale-grace contract. Pointer cancellation,
stale-target rejection, empty-selection disabling, and omission of unowned
device controls are deliberate project safety policies.

Only short labels and project-authored guidance are maintained. Hometown/likes
stay numeric references for a caller-supplied runtime catalog; no retail prose
table is copied. Preview frames reuse current production species sheets;
special-color compositing and exact per-species retail preview offsets remain
qualified presentation details.

## Runtime integration

E114 registers `app/virtual_runtime.rs`, its painter and
`app/virtual_input_runtime.rs` in the normal app. Owned resource/font/sound
manifests feed actual loading; missing required assets fail visibly. Store,
Fish Setup, Tank Setup and their captured dialogs replace the former placeholder
input and rendering. The same current Session dispatches each ordered action
batch without a fabricated simulation tick, then refreshes the snapshot before
another input. Rejected actions are read back too; stale targets are never
recaptured. The production purchase result is a transient event, supplying
feedback/audio exactly once without inferring success from a wallet change.

UI time uses a monotonic100Hz clock with focus cancellation, independently of
gameplay acceleration. Pet choice is a transient overlay over the existing
setup, using actual unlocked pets and current VT capacity; the Session rechecks
every toggle. Screensaver settings/preview use the existing configuration and
isolated preview owner. Closing overlays and immediate phase transitions consume
their input frame and freeze retained simulation. Profile switches discard all
captures/controllers. Current36/profile3/bank2 and existing saves are unchanged.

The two handed-off adapters were committed separately (`2c2bd4a`, gift
`34e5dbe`/`2d6c2bd`) and reconciled into root `f41308c`; root owns the actual
routing/assets/save joins. A60VTGift01 passed format/lint/build; review then
closed a closing-frame cancellation hazard and02 passed the changed-code gate.
**No tests or native runs were performed.** [STATUS](../STATUS.md#current-build-verification)
and [MODLOG](../MODLOG.md#2026-10-10--join-virtual-tank-and-shell-gift-frontends)
own receipts. Biography text still uses explicit numeric/unavailable labels;
the owned catalog is a follow-up. Species preview palette/offsets, date wording,
native visuals/input/audio, retail comparison and human judgment remain pending.

## Direct integration

Register `pub mod virtual_ui;` in the caller-owned library, then keep one
`StoreController`, `FishController` and `SetupController` per open screen/profile.
Each exposes `view(&snapshot, clock)` and `input(&snapshot, clock, &UiInput)`.
`VirtualUiSnapshot::capture(&session.virtual_tank,
session.progress.shell_balance, now_seconds, true)` reads existing production
values. Feed a monotonic `UiClock` in100Hz UI ticks plus current window focus;
controllers freeze on lost focus without counting the unattended interval.
Reset controllers on profile changes/closed screens. There is no system clock,
RNG, asset I/O, save write or gameplay tick in this module.

Use a frontend subpage flag to show Fish Setup separately while the current
session stays in `AdventurePhase::VirtualSetup`. That phase already admits fish,
settings, pet and backdrop actions. Preserve the caller's existing pet chooser;
this increment does not replace its roster/limits. Keep the screensaver entry in
`SetupView::screensaver_entry_bounds` and block it whenever the snapshot has a
dialog/external modal or the window lacks focus.

On `OfferPurchase`, `BeginRename`, `BeginFishSale` or `BeginBackdropSale`, retain
the intention's captured target and create the corresponding `PendingDialog`.
Attach it with `snapshot.with_dialog(...)` while the session's existing phase
owns the dialog. Update only its editable name when `SetDialogName` arrives.
**Do not recapture targets on every frame:** retaining the original target is
what rejects changed stock, changed fish records and recycled fish IDs. Rebuild
the inventory snapshot after every successful production action. External
caller-owned dialogs use `snapshot.external_block = true`.

`UiIntent::session_actions(&snapshot, day_start_seconds)` supplies concrete
`sim::Action` batches and repeats target/ownership checks. Dispatch batches in
order without another input between them. Fish purchases set the name before
`VirtualConfirmPurchase`; rename commits set the dialog name before
`VirtualRenameFish`. The session alone generates biographies and owns wallet,
Board synchronization and saving. `SetAllFishShown` maps the whole live ledger,
including breeder children, to existing `VirtualShowFish` actions; the visible
grid remains capped20 and excludes child IDs100+. Empty/stale selections emit
no destructive intention. Neither adapters nor controllers execute actions.

Cancel uses the existing production negative confirmations, returning to Store
or Setup. Rename cancel uses `OpenMenu`; an expired purchase also uses that
existing safe exit, which returns to the tank. Source-specific phase/navigation
changes remain caller-owned. After an actual successful purchase, call
`StoreController::purchase_result(&result)` for the108-tick feedback; cancellation
does not trigger it. Pointer movement consumes feedback at the W1 three-tick
rate. Hover guidance starts only after15 focused UI ticks.

Views provide button state/skin/font, price and speech geometry, preview
asset/cel/clip/layer, typed fish detail data and modal geometry. `ButtonView`
separates the production `available` predicate from focus/modal `enabled`, so
blocking input does not render a populated fish slot as empty. Route pointer
move/down/up/leave, keyboard activation, name editing, submit and cancel through
the controller. Held captures clear on modal/focus changes. Render fish names
and biography text within their view bounds; hometown/like IDs need a runtime
catalog or concise numeric/unknown labels, not copied retail descriptions.

For Store, draw widget bodies/previews, Store chrome, then slot prices/highlights,
shell balance/speech and Back; draw the modal last. The chrome's `Overlay` layer
preserves the W1 widget order. Fish Setup uses background/panel, grid bodies and
previews, rings/reflections/hidden labels, selected details, controls, then modal.
Setup uses background, chooser/backdrop, controls/settings/guidance, then modal.
`Cel::Indexed`/`Grid` use actual asset-catalog cel metadata; `Cell` uses explicit
production fish-sheet rectangles. Missing assets must remain an explicit
renderer/loader error. No native render or layout acceptance is claimed.

At starting`d782184`, `app.rs` does not load the newly used Store/Setup chrome:
`IMAGE_STORESCREEN`, `IMAGE_STORELEFTBUTTON`, `IMAGE_STORETOPBUTTON`,
`IMAGE_STORERIGHTBUTTON`, `IMAGE_STORELEFTHILIGHT`, `IMAGE_STORETOPHILIGHT`,
`IMAGE_STORERIGHTHILIGHT`, `IMAGE_STORESHELL`, `IMAGE_SPEECHBUBBLE`,
`IMAGE_BUBBULATOR_STORE`, `IMAGE_AA_STORE`, `IMAGE_TANKCHOOSER`,
`IMAGE_TANKBACKDROP1` through`6`, `IMAGE_CHECKED`, `IMAGE_UNCHECKED`,
`IMAGE_DIALOGBUTTON`, `IMAGE_LEFTBUTTON`, `IMAGE_CENTERBUTTON` and
`IMAGE_RIGHTBUTTON`. Verify their owned-install resource metadata when joining
the renderer. Existing fish/pet-button sheets are reused. `FontId` keeps existing
JungleFever faces and additionally names `ContinuumBold12outline` and
`BlambotPro8`; those two faces also need loading or an explicitly documented
presentation projection. Resource names come from registered W1 symbols, not
from a new tracked asset dump. `WinFishCommon.cpp:13–33,50–70,88–95` additionally
supplies dialog-button thirds and separate checked/unchecked assets;
`Fish.cpp:819–897` supplies the qualified two-tick preview frame cadence.

## Verification record

First isolated harness compile stopped with `E0583`: importing the module by
`#[path]` changed Rust's implicit child-module search directory. Explicit child
paths correct that harness/production registration boundary. Tests and build did
not execute in this failed attempt. The next strict lint stopped only on the
common button builder's argument count; selection now uses the returned view's
existing field rather than a redundant builder argument. Neither failed attempt
ran tests or build. The first test execution passed five and failed three before
UI assertions: the fixture placed a Bubbulator in slot0, which production
`StoreStock::valid` reserves for fish. Correct the fixture to accessory slot2;
offer capture now also delegates stock validation to the production predicate.
No build followed that failed run. The current validation result follows below.

The corrected harness passed formatting, strict lint/compile, eight contract
checks and build. A subsequent source review closed held-pointer survival across
external modals, distinguished availability from modal blocking and added the
direct session adapter contract. Rechecked only affected press/release, purchase
modal and focused-clock/feedback cases plus the new adapter case: four passed,
for nine distinct maintained cases. Later cel descriptors defer sheet grid
dimensions to the actual asset catalog instead of inventing them.

Commands run in ignored `.scratch/virtual-ui/harness`, importing the maintained
module with production `sim`/`virtual_tank` types through a path dependency:

```powershell
rustfmt --edition 2024 src/virtual_ui.rs
cargo clippy --offline --locked --manifest-path .scratch/virtual-ui/harness/Cargo.toml --all-targets -- -D warnings
cargo test --offline --locked --manifest-path .scratch/virtual-ui/harness/Cargo.toml
cargo test --offline --locked --manifest-path .scratch/virtual-ui/harness/Cargo.toml press_release_
cargo test --offline --locked --manifest-path .scratch/virtual-ui/harness/Cargo.toml purchase_dialog_
cargo test --offline --locked --manifest-path .scratch/virtual-ui/harness/Cargo.toml hover_feedback_
cargo test --offline --locked --manifest-path .scratch/virtual-ui/harness/Cargo.toml session_adapters_
cargo build --offline --locked --manifest-path .scratch/virtual-ui/harness/Cargo.toml
rustfmt --edition 2024 --check src/virtual_ui.rs
```

The full eight-case run precedes the localized four-case follow-up; these are
not12 distinct tests. Rust`1.95.0 (59807616e 2026-04-14)` and existing pinned
dependencies were used. Harness Cargo.lock SHA256
`8D23C17116588E237FD97FE9267292E29965C20060895329695F8D0268EC734F`
identifies the isolated dependency resolution. The own target/manifest/lockfile
and validated PowerShell scripts remain ignored within this worktree. Native
libopenmpt was read from the existing private package for dependency linking;
no installation/configuration/service/game/save resources were changed. No
gameplay speed flag applies to formatting/lint/fixed contract tests. Native,
manual, render/audio/retail comparison and runtime integration remain pending.

The host-validated `scripts/Test-Workflow.ps1` passed35 required files,
74 documents,842 local links,231 heading anchors,45 allowed-path and37 excluded-
path checks,153 eligible files. Scoped staged whitespace is checked before the
owned commit. Validation identity is base`d782184` plus only the six new owned
Rust modules and this note; core/model/app/Cargo files are unchanged in this
increment. Incoming unrelated main commits do not convert these checks into
native acceptance.

After fetching main at`ae57ee4c00c018f11eeec76975db3669260fcd9d`, the owned
implementation commit`b540ac2ef160bcd2436fab5decbf963c9e6ffd4d` merged without
conflicts as`2a4781b4355114241d17cc5296df2a8f21a51ce5`. The incoming changes
leave `virtual_tank.rs`, `sim.rs`, Cargo files and VT transaction handlers
unchanged; `adventure.rs` adds only separate saver-continuation methods.
Strict harness lint/compile and build passed against the merged production
crate. No unchanged contract filters were repeated for this reconciliation.
The receipt in ignored `.scratch/virtual-ui/validation.local.json` retains the
owned source, dependency lock, unit executable and merged library digests.
