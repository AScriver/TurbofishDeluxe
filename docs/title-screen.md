# Title screen and loading controller

This model increment adds a transient, renderer-independent controller in
[`src/title_screen.rs`](../src/title_screen.rs). Shared startup integration belongs
to the main checkout. The controller owns presentation progress, animation and
Start input only. The caller owns real loading, texture readiness, errors, clocks,
audio and the final application transition. No timer declares assets ready.

## Runtime integration

E111 registers the controller and [`app/title_runtime.rs`](../src/app/title_runtime.rs). Startup counts actual successful image uploads, enabled sound decodes, font loads, material creation and backend readiness. Seven bootstrap images are uploaded once and reused; the owned title/mask overlay is composed locally. Loading yields painted frames and honors quit/deadline. Ordinary Start currently preserves the selected saved scene; the incoming transient selector join owns the remaining ordinary routing. Fullscreen saver auto-finishes into its clone; embedded/configuration previews skip interactive Title. Title has an independent1x28ms UI clock. Native loading/input/mask/audio and retail timing are not tested. [STATUS](../STATUS.md#current-build-verification) owns combined integration checks.

## Design and source boundary

Recorded before implementation: pass actual resource dimensions and loading task
snapshots into one transient Rust controller. Emit ordered draw descriptors and
typed, once-only finish events. Do not introduce a second session, simulation,
profile, save, loader or native host authority. Failure is terminal; retries use a
fresh controller after caller cleanup.

Read-only source: W1 WinFish revision
`f919b3c241cfd611c547f1653fb3514f500f761b`,
`source/WinFish/TitleScreen.{cpp,h}`, startup/loading/finish and mask helpers in
`WinFishApp.cpp`, resource IDs in `Res.{cpp,h}`, and the already registered
framework progress/hit-testing paths. Functional facts inform original Rust;
no C++ implementation, proprietary art, trial/registration controls or original
prose is incorporated. [Provenance](provenance.md#runtime-sources) owns the existing
source-informed study/license limits. Installed PB05 remains authoritative.
[BR08](binary-recovery.md#br08--menus-profiles-and-options) records title update
`0051a7f0` only at outline level; no new primary or retail comparison is claimed.

| Claim | Independent source expectation | Class / limit |
|---|---|---|
| TS1 | `TitleScreen::Update`: increment widget age; decrement existing blend first; fill accelerates 1,2,3 pixels/update, capped at bar width; readiness is the following branch after fill catches reported progress | Secondary-source-derived; visual fill can overshoot fractional progress |
| TS2 | `TitleScreen::Draw`: loader at180,415; label at230,415; pressed hover shifts bar/Play/snail Y by1; loading draw is clipped only before readiness; alpha is counter×255/8 | Secondary-source-derived; no exact retail visual claim |
| TS3 | Constructor/Draw/Update: Stinky starts X160; 80×80 row0, ten frames, one frame/four updates, mirrored; draw while X<510; after readiness move toward canvas width | Secondary-source-derived; not gameplay Stinky state |
| TS4 | Ordinary completion exposes Start and sets blend8; `ButtonPress` requests click sound; `ButtonDepress` finishes title | Secondary-source-derived; pointer capture/cancel safety is explicit project policy |
| TS5 | Saver completion waits for readiness and Stinky X≥511; `TitleScreenIsFinished` chooses VT Board, clears backlog, ordinary branch chooses selector | Secondary-source-derived; controller emits route only, does not own host or Board |
| TS6 | `SexyAppBase::GetLoadingThreadProgress`: waiting/zero tasks→0, actual completed/total bounded at1, loaded→1 | Secondary-source-derived; malformed counts rejected by project policy |
| TS7 | `WinFishApp.cpp:480–481,1102–1152`: final overlay combines title-page RGB at384,388 with mask alpha, clipped to title image | Secondary-source-derived; raw mask is not a colored foreground texture |
| TS8 | Successful completion, failure terminality, bounded metadata and once-only finish are admission requirements | Explicit project safety contract; not claimed as retail error/reentry behavior |

The source also contains trial, registration, purchase, account and update-dialog
paths. These are outside this ordinary title outcome and are deliberately absent.

## Public API

`TitleScreen::new(TitleMode, logical_viewport_width, TitleAssets)` validates the
bootstrap metadata and begins waiting at fill 0/Stinky X160. Supply 640 for the
current logical canvas, independently of physical window/preview dimensions.
Image dimensions must be nonzero and fit the existing renderer's u16 texture
boundary; Stinky must contain ten 80×80 cells in row 0; the mask must intersect the
title page. Saver logical width must be at least 511 so its completion threshold
is reachable. These rejection rules are project safety policies.

`LoadingReport { completed_tasks, total_tasks, status }` accepts real task counts
and `Waiting`, `Running`, `Succeeded` or `Failed`. `progress()` gives the validated
source-style fraction. Unknown/zero-task pending work remains 0; explicit successful
zero-work completion gives 1. Counts above the total, completed work while Waiting,
or incomplete Succeeded reports fail atomically. Counts/totals may change while
pending; displayed fill never rewinds. A full task count while Running cannot
enable Start. `Succeeded` asserts that every required resource/backend is usable,
and requires completed=total. The caller must decide which optional unavailable
subsystems have been handled successfully; the controller cannot make that choice.

`report_loading(report)` changes loader status but does not animate or enable
Start by itself. `advance()` consumes exactly one caller-owned UI update and may
return `TitleEvent::Finished(FinishTarget::ScreensaverBoard)`. Successful ordinary
loading reaches readiness only after smoothing reaches the final width; the next
branch sets blend 8 and enables Start immediately, as W1 does. Existing blend is
decremented before this tick's progress/readiness branch. A late `Failed` report
before finish revokes readiness, cancels capture and freezes presentation; later
success cannot recover that instance. Render error details with caller-owned text
and dispose/reconstruct the controller if offering retry. Do not keep running after
a rejected report: handle its `TitleError` through the startup error path.

`pointer_move(x,y)`, `pointer_press(x,y)`, `pointer_release(x,y)` use logical
coordinates. Route only the primary button. Press inside a ready Start emits
`StartPressed` for the existing click-sound owner. An armed inside release emits
`Finished(GameSelector)` once. Unarmed, outside, canceled, loading, saver and
nonfinite-coordinate input cannot finish. Right/bottom edges are exclusive.
Dragging out removes pressed appearance; dragging back retains capture until
release or `cancel_pointer()`. Call cancellation on focus loss/modal interruption.
`can_start()` exposes ordinary admission. The eight blend ticks do not delay
Start admission; they affect appearance only.

`view()` has no clock or state side effect. It returns ordered `TitleLayer`s,
Start hit/hover/press state, the current report/fill and ready/finished flags.
`ImageDraw` contains the resource ID, source/destination rectangles, logical clip,
alpha and X mirroring. Preserve the ordered background→bar→labels→Stinky→mask
sequence. The bar clip is present even at width0; treat zero-area clips as no draw.
Before ordinary readiness, Loading uses that same clip. After readiness, Loading
and Play use complementary alpha without clipping. `MaskedTitleDraw` explicitly
requests title-page RGB plus mask alpha; drawing raw mask RGB is incorrect.
Finished views contain no layers. Events are caller-consumed requests rather
than persistent state changes.

## Exact main integration steps

1. Add `pub mod title_screen;` in `src/lib.rs` after joining concurrent owners'
   additions. This worktree intentionally does not edit the library declaration.
2. At `app::run`, replace the single startup await of `Presentation::load` with
   a cooperative startup phase. Decode/upload the seven `TitleImage::ALL`
   bootstrap images first and provide their actual dimensions to `TitleAssets`.
   Resource names are returned by `TitleImage::resource_id()`. Current `IMAGE_IDS`
   includes Stinky but does not wire these six title/loader resources. Keep Stinky
   shared with normal gameplay and release only title-exclusive textures on exit.
   Bootstrap failure goes straight to the caller's startup error path; fabricated
   metadata or missing textures must not be passed as ready assets.
3. `Presentation::load` currently loops over `IMAGE_IDS`, optional `SOUND_IDS`,
   eight fonts and the additive material, without frame yields. Enumerate the
   actual required work before reporting its total. Increment only after each
   successful image upload, sound decode, font upload or material creation.
   Yield with the existing `next_frame().await` while rendering the title. Title
   smoothing can continue between real operations; no task is completed by a
   clock. Keeping the original monolithic await cannot display live loading.
4. Startup also currently starts `MusicOwner`, loads `StoryCatalog` and loads
   `SongCatalog` after Presentation. Include any required preparation in the
   actual plan, handle optional failures with the existing unavailable states,
   and report Succeeded only after startup resources and required state are
   usable. Never submit a simulated 33-task source count or replay retail
   registration/update/purchase tasks. Failed required work reports Failed with
   the last valid counts, cancels input and retains caller error diagnostics.
5. Drive `advance()` from the existing logical 28ms UI cadence with a small
   bounded startup accumulator. It is independent of session/gameplay updates.
   Process current loader status before UI updates and pointer input, then draw
   `view()` each actual frame. Transform mouse coordinates through the existing
   logical-canvas projection; call pointer move each frame, route left press and
   release, and call cancellation on focus loss. Render descriptors at native
   logical dimensions, intersect clips with the viewport and restore clip/color
   state after each layer. Composite the mask once using decoded title RGB and
   decoded mask alpha at 384,388, or perform equivalent rendering; it remains
   install-derived runtime data outside Git. Source version text is outside this
   model; any version label must use the project's version and caller-owned font.
6. `StartPressed` routes `SOUND_BUTTONCLICK` through the existing sound owner.
   `Finished(GameSelector)` retires title state and uses the existing selector
   path. Clear UI/gameplay accumulated time and any captured pointer/held-feed
   state before handing over; reset run/evidence timing consistently so loading
   wait cannot age the Board or become earned gameplay. Existing save/profile
   and startup Virtual Tank ownership stay with the app. Do not apply an
   AdventurePhase or serialize this controller.
7. If the screensaver owner joins startup, use `TitleMode::Screensaver` and
   consume `Finished(ScreensaverBoard)` after ready Stinky reaches 511. No Start
   widget is admitted there. W1's finish caller removes title assets, switches
   to VT and clears update backlog. Main must route to its existing disposable
   saver scene/host; this controller creates no Board, bank, profile, OS parent,
   registry entry or native loop. Preview/configuration policy remains with that
   owner rather than being guessed here.

## Focused verification and identities

Base checkout: detached `d3189d67a95db61fcdd0393c6672c41f75eb6a9c`, then branch
`ascriver/title-screen-model`; initially clean. Exclusive maintained changes are
the module and this note. W1 reference checkout was clean at the pinned revision.
Toolchain: Rust 1.95.0 MSVC (`59807616e`, LLVM 22.1.2), Cargo 1.95.0
(`f2d3ce0bd`). The ignored `.scratch/title-screen/Cargo.toml` points its library
path at the maintained module, declares an independent workspace, forbids unsafe
code and has no dependencies. Build output is isolated in
`target/title-screen-module`; no central Cargo/lib file is changed.

Final checked module SHA-256:
`A8BC5680E8DF672459E000B29D9CEF67A13182D56BBE5E57CA5DBDF64940F918`.
Final focused test executable SHA-256:
`1D841C147AF683DC03374BCA4E9B50EE0281B5BF06C5AD220CC573ABE09E0C9B`.
This executable runs deterministic module tests; it is not a native game build.

Executed through the host parser/automatic-variable validator's `-Execute`
boundary:

```powershell
cargo fmt --manifest-path .scratch/title-screen/Cargo.toml
cargo fmt --manifest-path .scratch/title-screen/Cargo.toml -- --check
cargo test --offline --manifest-path .scratch/title-screen/Cargo.toml --target-dir target/title-screen-module
cargo clippy --offline --manifest-path .scratch/title-screen/Cargo.toml --target-dir target/title-screen-module --all-targets -- -D warnings
cargo build --offline --manifest-path .scratch/title-screen/Cargo.toml --target-dir target/title-screen-module
```

Six focused regression cases passed: TS1 fill/ramp/overshoot/branch ordering;
TS2/TS3/TS7 clipping/blend/pose/hover/mask order; TS5 automatic saver completion;
and TS8 no invented success, capture/cancel/edges/failure/once-only dispatch,
atomic task/metadata validation. The `source_` cases use W1 expectations; the
`safety_` cases check explicit project admission policies. Fixtures use synthetic
dimensions and deterministic integer ticks. These are controller observations,
not retail or integrated rendering evidence. No native gameplay, broad suite,
shared Ghidra database, original install/save, profile, process, port or OS setting
was touched. Main owns central records and integrated native acceptance.

After adding the saver logical-width reachability guard, only the affected
`safety_loading_reports_and_asset_metadata_validate_atomically` case was rerun,
with formatting, strict lint and module build (all passed). Unaffected passing cases were
retained rather than repeated. The repository workflow checker and staged
whitespace check also validate the maintained note/module boundary.

`scripts/Test-Workflow.ps1` passed: 35 required files, 73 documents, 843 local
links, 231 heading anchors, 45 allowed paths, 37 excluded paths and 146 eligible
files. Those counts identify this isolated checkout before main reconciliation;
they do not describe later concurrent commits or runtime behavior.

Deferred details: exact installed-title update/layout/frame fidelity and focus
comparison; native clip/mask/blend/mirrored texture inspection; actual startup
loading responsiveness, missing-resource UI, click sound and normal-speed title
to selector/saver acceptance. Human feel remains untested. Scratch harness,
validation scripts and local build artifacts remain ignored in this worktree;
no running helper or gameplay process requires cleanup. Actionables was not
updated: no scoped workItemId was supplied; main owns tracker reconciliation.
