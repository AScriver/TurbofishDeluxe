# Help examples and composition

This increment fills the omitted instruction and ordinary Help illustrations as an app-lived, render-independent model. Navigation and authored guidance remain in `src/help.rs`; nested public `help::demo` in `src/help_demo.rs` owns the transient clock and typed draw plan. The frontend remains a separate integration owned by the primary session. Code/source checks do not establish native or retail fidelity.

## Design and source contract

The source-informed adaptation follows already registered W1 revision `f919b3c241cfd611c547f1653fb3514f500f761b`, `source/WinFish/HelpScreen.{cpp,h}`, `WinFishCommon.cpp` helpers and `Res.cpp` resource identifiers, under [provenance](provenance.md#runtime-sources). Functional facts inform original Rust types and sampling functions. No upstream source, proprietary pixels, extracted prose or retail executable logic is incorporated. These animation/layout facts remain **secondary-source-derived**; installed-binary contradictions take precedence. This is not a clean-room or distribution-license claim.

| Contract | Pinned source | Behavior and limit |
|---|---|---|
| H1 | `HelpScreen.cpp:18-56,93-130` | New Help widget starts a new clock; page changes preserve it. Ordinary bubble bounds are `(0,80,640,400)`, fish bounds `(0,80,640,350)`; credits request fish configuration `(3,3)`, leaving credits disables spawning and scatters survivors. Bubble lifecycle belongs to the separate BubbleMgr owner. |
| H2 | `HelpScreen.cpp:307-485`; `WinFishCommon.cpp:179-204` | Three instruction cards. Feeding repeats every115 updates, swaps direction on successive loops, turns45-64, eats75-94, shows food40-84 and cursor clicks40-59. Fish travel uses clamped integer interpolation. Combat uses a40-update cycle, a10-update additive flash, an anchored20-update click burst and moving cursor. Third card displays the complete egg sheet. |
| H3 | `HelpScreen.cpp:488-625` | Basics: hungry fish, coin, moving alien, cycling egg. Upgrades: guppy, quality/quantity pellets, cycling weapon. Pets: completed egg, animated Itchy/Stinky, selection illustration. Mode labels use a200-update color pulse. |
| H4 | `HelpScreen.cpp:637-786` | VT: store button, shell, three-layer colored fish and ball. VT fish:27-frame eating/swimming cycle, larger-fish row, Feed button and death/question example. Configuration: Fish/Pets/Tank buttons and saver illustration. Color layers are additive and scoped to each draw command. |
| H5 | `HelpScreen.cpp:134-169,232-304,789-849`; `WinFishCommon.cpp:124-160,207-212` | Background, waves, shared row panels, header and navigation chrome; credits have pulsing/bobbing project attribution rather than copied retail prose. Asset sizes and wave animation metadata come from the caller. |

The Help clock never references Board actors, simulation RNG, wallets or serialization. It uses the existing28ms project update duration; wall-time accumulation is independent of tank pause. Drawing is read-only. Reopening resets the transient clock; selecting another page preserves it. Missing/invalid image metadata omits the affected sprite and reports a typed issue, while authored text/navigation remain available.

Integer geometry uses the640x480 canvas. Fixed source crops retain their checked frame dimensions; full images/cels use validated caller dimensions. All commands carry an explicit canvas/panel clip, layer, blend and tint. The renderer must apply and restore these states for each command. Font selection and alignment are descriptors; the renderer performs glyph measurement and wrapping using its loaded fonts. Source oversized thumbnails retain their original geometry with a canvas clip; instruction moving examples use their own upper-card clip.

Presentation choices remain explicit: direct page tabs are the existing project extension, authored guidance replaces retail prose, header/button heights have safe bounds and missing-art fallbacks, and trigonometric angles use periodic standard PI rather than accumulating W1's approximate-PI drift. Quantity-pellet W1 quirks (`sin(0)`, offsets interpreted in radians and doubled last angle) remain qualified rather than silently corrected. These choices do not establish exact pixel equivalence.

## Integration and verification

Existing public `HelpController::{new,remembered_page,open_instructions,open_pages,view,next_page,previous,select_page,continue_intro,close}` and `HelpView` remain compatible. Additive public APIs are:

| API | Caller responsibility |
|---|---|
| `HelpController::advance_demo(Duration) -> u64` | Call once per active Help frame using elapsed UI time, independent of the paused tank. Returns consumed28ms updates for separately owned ambient effects. Closed Help consumes no time. |
| `HelpController::advance_demo_ticks(u64)` | Alternative for an existing fixed Help update cadence; do not also advance by elapsed time. |
| `HelpController::demo_clock() -> &demo::DemoClock` | Read the transient update counter; opening either Help route resets it, navigation preserves it. |
| `HelpController::render_plan(&impl demo::HelpAssets) -> Option<demo::HelpRenderPlan>` | Sample the active page without advancing anything. Closed Help returns `None`. |
| `demo::HelpSprite::ALL`, `HelpFont::ALL`, `asset_id()` | Preload35 images and5 fonts through the existing read-only asset/font owner. These are resource identifiers, not bundled assets. |
| `demo::ImageMetrics::new(width,height,columns,rows) -> Option<ImageMetrics>` | Supply decoded metadata. Zero/nondivisible/out-of-range dimensions are rejected. `full_rect` and `cell_rect` expose checked geometry. |
| `demo::HelpAssets` | Implement `image_metrics`; optionally provide `wave_cell(update)` from actual asset animation metadata and `animate_credits`. Missing wave timing reports `MissingAnimation`; it is not guessed from grid size. |
| `demo::compose(HelpView,&DemoClock,&impl HelpAssets)` | Standalone sampling; invalid directly constructed page numbers return `None`. |
| `HelpRenderPlan::control_at(HelpPoint) -> Option<HelpAction>` | Use the exact rendered control bounds for pointer input. Dispatch Continue/Return/Previous/Next/Page through the existing controller APIs. |

Primary integration steps:

1. Add Help's listed resources to the existing texture/font cache and implement `HelpAssets` using decoded metadata. Missing art must preserve authored text and functional navigation; keep typed `asset_issues` available for diagnostics.
2. Advance the Help clock during the active Help UI hold, while retaining existing tank pause/progression and accumulated-time clearing on return. Sampling/drawing may occur more often than updates without changing animation state.
3. Render `draws` in stable layer order. Draw separately owned ambient bubbles after Background and before the Ambient wave sprites. Apply each explicit clip, tint, blend and mirrored source crop; restore renderer state afterward. Render the returned controls last, using the same bounds for hit testing. Text descriptors provide a first baseline, alignment and wrap/line-height bounds; font measurement remains with the font renderer.
4. On open/page changes, apply the returned `ambient` configuration to the separately owned BubbleMgr. Credits request `[3,3]`; leaving credits requests `[0,0]` and needs the source-derived outward scatter of surviving fish. The Help model owns no particle lists or RNG.
5. Reconcile the historical animated-example gaps in shared interface/compatibility/design/provenance only when the app actually consumes this model. Primary owns these records and all eventual native acceptance.

### Focused checkpoint

Own checkout `C:\Users\Austin\.codex\worktrees\763e\TurbofishDeluxe`, branch `ascriver/help-animated-examples`, starting from clean `d3189d67a95db61fcdd0393c6672c41f75eb6a9c` (Windows host integration). Only `src/help.rs`, new `src/help_demo.rs` and this document are owned edits. Rust `1.95.0 (59807616e 2026-04-14)`, host `x86_64-pc-windows-msvc`, LLVM22.1.2. Source checkout W1 was clean at the pinned revision. SHA256: HelpScreen.cpp `BF1BFC2EADB458317F4B9E3A32ECFE3D4ADB6FCA239D9C06C35D31239AD96C00`; HelpScreen.h `2C5FDD5B189FB2EC29577432C683D1BE6B01D6EA965501B48A2AB9CB2E1D7194`; WinFishCommon.cpp `1C5460B01F79AF2ECDC5E6DC06DC14ED5A8D3537845D4A2B3A59E7699DD707AA`; Res.cpp `48FF5998636C0383FEF3060F7D6ECC3AEAFF811AA6F828F68849B2778D5722CA`.

Task scripts ran through the host PowerShell parser/automatic-variable validator with `-Execute`. Validation used the existing digest-checked libopenmpt0.8.9 package read-only and this worktree's isolated `target/help-animated`; no new system tool/package was installed. Exact commands:

```powershell
rustfmt --edition 2024 src/help.rs
cargo fmt --all -- --check
cargo clippy --locked --offline --workspace --all-targets -- -D warnings
cargo test --locked --offline -p turbofish-deluxe --lib help::demo::tests::
cargo clippy --locked --offline -p turbofish-deluxe --lib -- -D warnings
cargo build --locked --offline -p turbofish-deluxe
```

Formatting and strict workspace lint passed. After additive resource-preload constants and the instruction text fit adjustment, formatting and strict affected-library lint passed. **Six Help cases passed,548 filtered out**, covering source feeding/turn/eat/cursor windows and integer travel, combat flash/burst anchoring, VT sheet/color-layer transitions, per-page examples and preservation of authored guidance, missing/invalid metadata/bounded clips, and controller page/clock/return/instruction invariants. Fixtures use explicitly synthetic decoded dimensions; they prove source crop/transition contracts and resource-failure handling, not owned-asset pixel fidelity. Locked offline build passed. `scripts/Test-Workflow.ps1` passed35 required files/73 documents/841 local links/230 anchors/45 allowed and37 excluded path checks/146 eligible files; runtime acceptance is outside that check. Private logs remain under `.scratch/help`.

No game process launched, actual save/OS state changed, shared analysis database used, or primary checkout modified. Native input/render/audio/retail comparison and human judgment remain not tested. Actionables was not updated: no `workItemId` supplied.

## Runtime integration

E112 closes the earlier missing-WAVE metadata gap using the [owned animation API](asset-animation.md). Exact native animation/render fidelity and the separately pending credits BubbleMgr join remain unverified.

E111 loads the declared sheets/fonts and uses [`app/help_runtime.rs`](../src/app/help_runtime.rs) for the complete typed illustration/chrome plan. Input hits the same plan's controls; existing caller-owned Help returns remain intact. Help's independent28ms demo clock advances while the retained tank is paused. Image crops, glyphs, tiles and primitives use safe CPU clipping; the runtime crate keeps its unsafe-code prohibition. Missing WAVE animation metadata reports an unavailable illustration rather than guessing frames. Credits ambient BubbleMgr insertion, native input/render/audio, font placement and retail comparison remain pending. [STATUS](../STATUS.md#current-build-verification) owns combined integration checks; the isolated model checks above are historical.

## Deferred details

- Exact installed Help art, pixel/font placement, source quantity-pellet quirks and retail timing comparison remain unverified. Retain source quirks rather than invent fixes.
- BubbleMgr integration, actual visible input/render/audio and human fidelity checks are pending with their owners.
- Existing finale/story catalogs remain separate; no second credits/lore extraction is added.
