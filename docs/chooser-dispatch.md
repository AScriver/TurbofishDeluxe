# Normal-game tank chooser dispatch

## Contract and scope

Actionable #309 corrects the normal Time Trial and Challenge chooser joins under
the [necessary-fix ownership contract](necessary-fixes.md#tank-chooser-dispatch-correction).
The worker owns only chooser collection/dispatch and focused tests in `app.rs`,
the narrow `app/tank_selection_runtime.rs` dispatcher, and this report. Root owns
central records, joined build acceptance and completion. Screensaver code and
all earlier completed transfers remain closed. No native gameplay, installation
or live/earned save access is part of these checks.

## Evidence and chosen correction

The clean starting checkout is `f099494e4df73b6b28b4de4a9d83ed42a8c5d664`, whose
runtime source is unchanged from defect checkpoint
`fdd3b67de2a7802769b7afef5434d8a65528196b`.

| Claim | Evidence and limit | Status |
|---|---|---|
| C309-1: normal chooser actions starve | Original `app.rs:7033,7049` queue Escape/Menu/selection; `7060-7076` owns UI; `8679-8725` pauses the normal queue drain. Exit/deadline can flush the queue at `8664,8687`. No native run. | Reproduced in the extracted offline gate; original source references remain historical |
| C309-2: Session already owns admission without a tick | `adventure.rs:2343-2346,2480-2513,2720-2754` implements locked/eligible selection and Menu using `apply_actions`. | Strongly source-supported |
| C309-3: higher owners must block chooser | `app.rs:6892-6996` establishes overlay/modal/menu frame ownership; chooser's original modal projection at `7011-7015` omits selector panels/normal modals, pause and closing-frame ownership. | Strongly source-supported |

The actual normal loop collects chooser actions in a separate frame-local vector
and consumes them through the production-called Session dispatcher before the UI
pause gate. It admits only matching chooser selection and Menu actions, preserves
Session's eligibility checks, and stops after the first phase transition.
Unfocused input and every existing higher frame owner block dispatch. Events and
transition state return to the existing frontend joins, while UI ownership lasts
through the entire closing frame. Pending gameplay is not dispatched through the
chooser. Existing idle paused-step, save, clock, schema and simulation rules stay
unchanged.

## Original red reproduction

On2026-10-12 the private harness extracted the original queue, UI projection,
fixed-step gate/drain, Session call and exit-flush statements. The only timing
substitution is `get_frame_time()` to deterministic0.028. Unrelated owners are
false, one current synthetic profile exists, and no GPU/window is created.
Temporary test registration changes no runtime behavior and was removed before
the correction. This controlled extraction proves the join failure under those
conditions; it is not a full-loop native observation.

`cargo test --locked --offline -j 2 --lib chooser_original_dispatch_tests`
returned101: four immediate-selection/Menu/Escape expectations failed with
`TimeTrialTankSelection`/`ChallengeTankSelection` retained; both exit-flush
witnesses passed after three chooser frames retained one pending selection.
Two `unused_mut` warnings come from unchanged extracted declarations. All122
maintained source inputs stayed unchanged during the run.

Private evidence in `.scratch/chooser-309` includes `original-app.rs`,
`original-tank-selection-runtime.rs`, `red-registered-app.rs`, the fragment
receipt/generated harness, immutable `original-red-test.exe`, and
`20261012-091219-368-red.log`/`.local.json`.

| Original identity | SHA-256 |
|---|---|
| Original `app.rs` | `c9e550514560337992be6ea6d944e088bb9f3ee771a6fdfce3d54272a3125bb3` |
| Original chooser adapter | `d5b329f206b6893be446ad0b461d2d5fe301bfa0d8cac3f53dbf6a7fffca1bfa` |
| App with temporary test registration | `f841dd505664abef2616f0856a4271557dbbaec4a9dcc78b5c724ec16518d5cd` |
| Red test executable | `1f37fdebd619959ea7e107822d35542188f01a14f767db3f8b70291702599f4c` |
| Red maintained-source set | `1a16cac19754da0f2d31401cfb34ba88836b9546384a2b7634d36aa0fb9a2c03` |
| Red log | `01ea18fc8a8cc9d54512504d95293039c2c8198cc5c1646b5468a8944ce70893` |

## Corrected checks and limits

Eight `app::chooser_dispatch_tests` passed with
`cargo test --locked --offline -j 2 --lib chooser_dispatch_tests`:

- Both mode selections call real Session admission exactly once and open the
  pet-selection phase, preserving tick77, the next seed and absent Board.
- Locked/invalid choices reject without state changes; stale controller facts
  cannot bypass current Session admission.
- Menu and Escape return both modes to the selector without a tick or later
  exit-flush action.
- Unfocused/modal input is consumed without state changes or delayed replay.
- The first transition drops remaining selection/Menu/Continue actions before
  the destination can use them; gameplay/wrong-mode/stale-screen input is filtered.
- The production parent gate blocks panels, actual normal-modal state, pause and
  earlier frame owners even after a modal closes; a fresh next-frame choice works.

These tests use fresh in-memory Sessions with seed309/tick77; the ordinary
eligible fixtures pass current Session validation. The admission-race check
deliberately changes unlock facts after capture. No live save or asset is read.
The actual caller uses `dispatch_actions` before `ui_active`, joins returned
events and transition to existing frontend state, retains `ui_owned_input`, and
clears transition backlog/holds. Existing event stamps, checkpoint decisions and
fixed-step transition barrier receive those events; idle chooser paused steps
and all saver code remain unchanged. This caller ordering is source-checked,
not GPU-executed.

The first green compile failed with `E0616` because a test read private
`AdventureSession.next_seed`. The assertion now uses serialized state; no
production visibility changed. Its failed
`20261012-091809-092-tests.log`/`.local.json` remain. The passing focused run is
`20261012-091908-466-tests.log`/`.local.json`; formatting check is
`20261012-091948-770-format.log`/`.local.json`. All122 inputs match before/after
each check. Strict affected lint and the normal-bin build also passed on those
same bytes; passing unaffected Cargo checks were not repeated.

| Check | Exact command and result | Private receipt/log stem |
|---|---|---|
| Focused regressions | `cargo test --locked --offline -j 2 --lib chooser_dispatch_tests`:8 passed,681 filtered | `20261012-091908-466-tests` |
| Formatting | `cargo fmt --all -- --check`:passed | `20261012-091948-770-format` |
| Strict lint | `cargo clippy --locked --offline -j 2 --lib --bin turbofish-deluxe --tests -- -D warnings`:passed; test targets compiled, no broad test suite run | `20261012-092009-814-lint` |
| Normal binary | `cargo build --locked --offline -j 2 --bin turbofish-deluxe`:passed, default dev/debug | `20261012-092230-945-build` |
| Workflow | `pwsh -NoProfile -File scripts/Test-Workflow.ps1`:passed, including local links/anchors and45 included/37 excluded path checks | `20261012-092241-148-workflow` |

Each named check has an immutable `.log` and `.local.json` in the worker scratch.
Rust/Cargo1.95.0 MSVC (`rustc59807616e`, host`x86_64-pc-windows-msvc`), default
features and locked/offline dependencies identify this gate. It used a private
Cargo home/target and the pinned read-only libopenmpt0.8.9 package; neither the
root cache nor launcher artifacts changed.

The frozen `.scratch/chooser-309/checked-build.local.json` has record ID
`13caf58e3dd2a03e4c403e56325bbb7df2ab4e47559c1016b67cafae4a699e22`
and file SHA
`dd58bd82da1310e170517ec9823ab3b9073360c24fda1a16ff31a9a7f2e49a11`.
It records the build at baseline`f099494` plus exactly the two dirty runtime
paths, all122 maintained build inputs before/after, seven logs, original red
evidence and six native import-library/DLL identities. Every source is copied
and digest-checked in `checked-build/sources`; normal/test executables are frozen
as `checked-build/turbofish-deluxe.exe` and `checked-build/chooser-tests.exe`.

| Checked identity | SHA-256 |
|---|---|
| Maintained-source set | `f3ad1057978dae2f5e148274730039dfe8da9141760b442a50d1352791b1c9e0` |
| Corrected `app.rs` | `e3fd54f06322362e8cf881e65835d76ba82d67955c46daa4addc05bea6146edf` |
| Chooser dispatcher | `c912a118ad73ca858eb39d3c11e39417da1e2e585e8b9f356baefd5a7bf1c157` |
| Normal-game EXE | `6ce06ab8be581547f0749b36fe7edd17d137d63972500028c284d8242641bc6b` |
| Chooser test EXE | `f37110efa32c608304d5f136d685b926f13f144d12b6c9d700468b773c708413` |

Worker Cargo home/target, scripts, snapshots and logs stay under the worker's
ignored `.scratch/chooser-309`; native dependency reuse is read-only. No services,
ports, databases, game processes, real/live/frozen/earned saves or original
installation state are allocated or touched. Disposable state is in-memory;
no external fixture cleanup is needed. Preserved evidence/cache are intentional.
No native/retail/audio/playability claim follows from these offline checks.

Worker engineering and its focused gate are complete. The source/report are
published together; the Git history identifies their owned commit. Root's
explicit joined acceptance remains the completion gate; Actionable #309 stays
In progress until that acceptance, preserving the original failed evidence.
