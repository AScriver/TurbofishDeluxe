# Tank selection controller

This chunk adds the renderer-independent four-card Time Trial/Challenge chooser in [tank_selection.rs](../src/tank_selection.rs). It is an unregistered module: the primary session owns library registration and actual app, asset, audio and session joins. No Profile, save schema, score, award or navigation authority moves into this controller. Native/render/audio/retail/human acceptance remains pending.

## Source and build identity

Behavior is **secondary-source-derived** from registered W1 revision `f919b3c241cfd611c547f1653fb3514f500f761b`, read back clean for the studied files:

- `source/WinFish/TankScreen.cpp`: constructor, `AddedToManager`, `Draw`, `Update`, `ButtonPress`, `ButtonDepress`, `DrawBolts`; associated header supplies the four-card screen shape.
- `source/WinFish/WinFishCommon.cpp`: `MakeDialogButton`, `MakeDialogButton2`, `DefaultDialogButtonSettings`, `DrawHorzCrease`, `DrawVertCrease` supply button sheets, label offsets and unscaled crease tiling.
- Already registered `source/SexyAppFramework/DialogButton.cpp:41–87` supplies crop selection, pressed label translation and the font-ascent/padding label baseline. No framework implementation is incorporated.

Study and original Rust adaptation retain [provenance boundaries](provenance.md#runtime-sources). No upstream C++, raw decompiler output, original prose, assets or executable logic is bundled. Installed PB05 eligibility/story corrections in [Time Trial](time-trial.md) and [Challenge](challenge.md) remain authoritative; this module does not reinterpret them. Titles and control names identify the existing modes; descriptions and lock explanations are project-authored.

Worktree began at clean detached `d7100587fe3c16bd504ffc21c22ab66a7f3df7cf`, not the earlier reported `f8eba73`. Relevant production inputs were clean. Rust/Cargo `1.95.0`, Windows MSVC/debug, offline scratch harness; no gameplay seed, clock, save or running game applies. Production `Cargo.lock` SHA-256 `1643FD7BE00AF38DB3DA764BE8E0C03CFC427EE368DCAAB7EF17A49B187F153F`. Actual tests use the production crate by path, production `DecodedImage`, `BitmapFont`, `FontLayer` and `Glyph`; types are not redefined to simulate integration.

## Ownership and controller contract

`TankSelection::new(SelectionMode, TankFacts)` fixes the mode for that scene. `TankFacts.available[0..4]` is an immutable caller snapshot of actual production tank availability; `has_ordinary_stories` is supplied from actual earned ordinary stories. The controller neither reads a mutable Profile nor calculates unlocks. A changed snapshot is installed with `set_facts` before processing input, cancelling captures even when a different card changed.

`scene(&impl TankResources)` returns `Result<TankScene, SceneUnavailable>`. `TankResources` exposes complete decoded image metadata and references to the real production `BitmapFont`s. `ImageMetadata::from(&DecodedImage)` supports decoded caches. `OwnedTankResources::load(&GameAssets, &Path)` is a read-only convenience adapter: it decodes owned images, loads actual font scripts, validates font atlas bounds, retains metadata/fonts and drops decoded pixels. The app should reuse its existing caches instead of loading a second set.

Missing images/fonts, invalid dimensions/grids/button sheets, missing glyphs and invalid font metrics are explicit unavailable results. There is no zero-size or fabricated-size fallback. A failed resource check during input cancels capture and emits no intention. The primary must present its ordinary resource-unavailable UI and safe exit path when a scene cannot be composed. Resources unused by the current scene need not exist in a cache implementation; the convenience loader deliberately loads the complete chooser catalog.

W1's source eligibility is distinct by mode: Time Trial card index `i` is enabled when profile tank≥`i+2` or finished; Challenge indices1–3 consume the preceding Challenge completion flag. Production mode entry guards Challenge's first card. These are references for the primary's existing eligibility authority, not another policy implementation in this UI. The authored lock explanation names Adventure tank `i+1` in Time Trial and Challenge tank `i` for later Challenge cards. An unavailable first Challenge card reports the prerequisite Adventure completion without inventing a Challenge tank0.

`TankScene.cards` always has four entries. Unavailable cards retain the frame, authored explanation and white tank label, omit the backdrop and button skin, and remain unselectable. `TankScene.hit_test` considers only enabled controls. `input(TankInput, resources)` consumes primary-pointer move/down/up, focus, modal and cancel events. A valid first down emits `PlayButtonSound(Click)` once; only up on the same still-enabled control with unchanged bounds emits `SelectTank { mode, tank }`, `Menu` or `Stories`. Repeated downs, release without capture, release over another control and nonfinite pointers cannot select. Focus loss, modal transitions, explicit cancel, availability/story changes and resource failures invalidate captures.

`update()` only marks dirty, matching W1 `Update`; `take_dirty()` consumes that flag. No simulation, timer, RNG, story awards or animation clock is introduced. Hover/pressed crops are immediate; exact source hover blending is deferred.

## Source layout and resources

Logical viewport is 640×480. Preserve command order and clip to the viewport; `Layer` documents ownership and is not a sorting key.

| Element | Geometry or rule |
|---|---|
| Screen | Image box at `(-5,-5,650,490)` |
| Top cards | Origins `(70,71)` and `(350,71)` |
| Lower cards | Origins `(70,264)` and `(350,264)`; y=284 when Stories is visible |
| Card bounds | Actual `IMAGE_TANKCHOOSER` width/height |
| Card backdrop | Natural size at origin+(16,10); displayed tanks 1,2,3,4 map to backdrop resources 1,2,4,5 |
| Tank buttons | Origin+(14,134), width193, actual MainButton sheet height |
| Lock explanation | Origin+(25,43), rectangle170×200; horizontal centering/wrapping with actual Small font, source line metrics, viewport clipping |
| Menu | `(525,4,80, actual MainButton height)`; source ID99 |
| Stories | Challenge plus earned ordinary stories; `(245,247,150, actual DialogButton height)`; source ID4 |
| Creases | Horizontal `(13,130,614)` and `(13,320,614)`; second y=350 with Stories. Vertical `(180,100,369)` and `(460,100,369)`. Actual perpendicular size, unscaled tiles, clipped final tile |
| Bolts | `(50,110)`/`(30,150)` with source horizontal/vertical reflections; `(320,100)`/`(320,158)` with vertical reflections. Natural image dimensions |
| Title | Image box `(20,0,600, actual title height)`; gold title baseline25; white authored mode description baseline60 |
| Menu title hole | Image box `(524,3,82, actual hole height)` |
| Button states | Three equal horizontal crops: normal0, hover1, pressed2. Image-box skin remains at control bounds; pressed label translates+(1,1). Label horizontal offset−2; baseline uses actual font ascent/padding |

`DrawCommand` carries typed image resource, frame, pixel crop, destination, projection and layer; text carries typed font, placement and RGBA. `Natural` does not scale. `HorizontalTile`/`VerticalTile` repeat the crop without scaling and clip the last copy. `ImageBox` uses the existing app image-box tiler, with source borders derived from the supplied crop.

The image catalog is `IMAGE_SCREENBACK`, `IMAGE_HORZCREASE`, `IMAGE_VERTCREASE`, `IMAGE_BOLT`, `IMAGE_SCREENTITLE`, `IMAGE_SCREENTITLEHOLE`, `IMAGE_TANKCHOOSER`, `IMAGE_TANKBACKDROP1`, `IMAGE_TANKBACKDROP2`, `IMAGE_TANKBACKDROP4`, `IMAGE_TANKBACKDROP5`, `IMAGE_MAINBUTTON`, `IMAGE_DIALOGBUTTON`. MainButton declares three columns; DialogButton's manifest grid is1×1 but W1 divides its painted sheet into three states. Each crop is calculated from actual complete sheet dimensions.

Fonts are `FONT_JUNGLEFEVER17OUTLINE` → `JungleFever17outline` (title), `FONT_JUNGLEFEVER10OUTLINE` → `JungleFever10outline` (description/locks/Menu), and `FONT_JUNGLEFEVER12OUTLINE` → `JungleFever12outline` (tank/Stories labels). Button sound is `SOUND_BUTTONCLICK`; the primary respects mute and the existing audio owner.

## Primary integration

1. Register `pub mod tank_selection;` in the production library. Add a transient chooser to the app for `AdventurePhase::TimeTrialTankSelection`/`ChallengeTankSelection`; recreate on mode/profile/scene changes and call `cancel` before removing it.
2. Supply production availability for all four tanks and `session.progress.stories.seen_mask != 0` for ordinary-story visibility; that field was verified in the base. Do not use the separate Cyrax flag or an award/navigation routine as this visibility gate. Keep the session's authoritative action guards aligned with the already established Time Trial/Challenge contracts; the chooser does not repair or replace them. The observed base's TT action guard only checked supported tank IDs, so the primary must adjudicate its availability join before relying on UI alone.
3. Implement `TankResources` on `Presentation`: return actual texture width/height and `image_cells` metadata, and the real `RenderedFont.metrics`. Load the missing chooser/crease/bolt/title-hole/backdrop/DialogButton IDs through existing `GameAssets`; retain original alpha/grid handling. Reuse already loaded font atlases.
4. Replace the base's vertical-list render block around `src/app.rs:4566` with ordered scene commands. Map image-box commands to the existing image-box helper, natural commands to unscaled pixel draws, crease commands to unscaled clipped tiling, and text to actual fonts. Wrap lock text in the supplied rectangle using owned font metrics. Do not keep the old blanket selector backdrop over this scene.
5. Replace the vertical-list input route around `src/app.rs:7024`; feed actual down/up/move and focus/modal changes after refreshing facts. Map successful typed `SelectTank` to existing `Action::SelectTimeTrialTank`/`SelectChallengeTank` using `tank.number()`. Map `Menu` to existing `Action::OpenMenu`. Map `Stories` through the primary's earned Story-browser route with a return to the retained chooser. The controller does not enter modes, award stories or invent a return destination.
6. Route `PlayButtonSound` once through existing audio, avoiding a second generic click sound. Use the descriptor control bounds for native input checks after integration. On unavailable resources, discard stale composition/capture and show the primary's safe unavailable UI.

## Checks and remaining acceptance

Focused formatting, strict lint, six targeted regression cases and debug build **passed**. The ignored `.scratch/tank-selection` harness includes the maintained module by path and depends on the actual production crate; its target, scripts and artifacts stay within this worktree. No root Cargo/lib/app file is changed. Commands were run through `Invoke-CodexPowerShell.ps1 -Path .scratch/tank-selection/check.ps1 -Execute`, after parser/automatic-variable validation:

```powershell
rustfmt --edition 2024 --check src/tank_selection.rs
cargo clippy --manifest-path .scratch/tank-selection/Cargo.toml --offline --locked --all-targets -- -D warnings
cargo test --manifest-path .scratch/tank-selection/Cargo.toml --offline --locked --lib
cargo build --manifest-path .scratch/tank-selection/Cargo.toml --offline --locked
cargo run --manifest-path .scratch/tank-selection/Cargo.toml --offline --locked --quiet
```

Final safety review added viewport clipping to hit-testing and bounded font ascent to prevent malformed metadata from overflowing label geometry. Only its two affected input/metadata cases were revalidated, with formatting, strict lint and build; other passing cases were not rerun. The filters were `missing_or_invalid_metadata_is_unavailable_and_cancels_input` and `locked_buttons_and_release_without_matching_capture_cannot_select`. Final maintained source SHA-256 `555E800B81ECB424D7020A55D26CB411D21C3F213B8FCDF89059A4079DCDFF82`; harness lock SHA-256 `74C050757B4F794DA19DCA5EBAB3718A85604AFE1D98B91F6174C1AA01B9081F`; final scratch metadata-probe executable SHA-256 `F84EF6C6F836FE828AB21FE5373E6A5E48CBD4AEFB442FD55057A42C859D284F`. These identify the clean base plus this source change; unrelated merges do not imply a fresh native result.

First verification stopped in the production `turbofish-openmpt` build script before any chooser case: the new worktree did not inherit `OPENMPT_LIB_DIR`. The follow-up uses the existing official cached package read-only, verifies its recorded import-library/five-DLL digests, and supplies library/PATH variables only to the validated child process. No download or system configuration change is needed.

The six targeted cases cover the two source layouts, locked/captured input, source backdrop mapping, ordinary Stories gate, typed actions/pressed crops and missing metadata. Expected geometry/mapping/gates come from the pinned source and supplied contract; test image dimensions and font glyph metrics are explicit project fixtures, not retail pixel/fidelity claims. A read-only owned-install metadata probe is separate from those cases.

The owned metadata probe composed both modes successfully (37 commands for the chosen partially locked Time Trial snapshot,39 for Challenge with Stories) without a game window or audio output. The actual manifest SHA-256 is `DECFCFD152C0328CDB99EB2D98F56567AA13A4CB009CF7CE280338F19BFDE5AE`. Observed complete image sizes: ScreenBack240×240; horizontal crease3×8; vertical crease8×3; Bolt7×7; title90×44; title hole93×31; chooser224×172; each of the four selected backdrops190×121; MainButton270×29 (3×1 grid); DialogButton288×33 (1×1 manifest grid). All remaining listed images declare1×1. These are metadata observations, not rendered retail comparison. Font scripts/atlases loaded and validated; runtime code derives these sizes instead of embedding them.

Validated `scripts/Test-Workflow.ps1` passed:35 required files,78 documents,854 local links,237 heading anchors,45 allowed/37 excluded path checks and158 eligible files. Staged whitespace/ownership checks apply only to the two assigned maintained files. Scratch build/probe/test processes exited normally; no ongoing resource needs cleanup. The harness and artifacts remain ignored in the owning worktree.

Deferred: main registration/joins, normal-speed native input/render/save/reopen acceptance, exact wrapped-text/hover-blend pixels, audible comparison, retail comparison and human feel/fidelity. No game process, original-save write, shared database/port/service or Windows setting is used by this chunk; earned/frozen E80 lineage is untouched. No workItemId was supplied; Actionables was not updated.
