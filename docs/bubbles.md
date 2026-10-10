# Transient bubble effects

## Scope and ownership

This increment supplies an independent Rust bubble-effects module for later caller integration. It owns only `src/bubbles.rs`, this document and the `pub mod bubbles` declaration in `src/lib.rs`. It does not change simulation, frontend, Virtual Tank, screensaver, saves, dependencies or central handoff records. Effects are transient; there is no profile, gameplay actor, save, clock scheduler or RNG authority in this module.

Implementation checkout: `C:\Users\Austin\.codex\worktrees\aa3f\TurbofishDeluxe`, initially clean detached HEAD `4c87d559f590975b036e84eb226bc774405238d2` (installed song-parser fix), rather than the dispatch's older `be850c1`. The dirty primary checkout is not an implementation input.

## Evidence and design decision

All bubble rules below are **secondary-source-derived**, independently read from clean pinned WinFish `f919b3c241cfd611c547f1653fb3514f500f761b`, `source/WinFish/BubbleMgr.{cpp,h}`. Existing [provenance](provenance.md#runtime-sources) permits source-informed adaptation of functional facts into original Rust structures, without upstream source files, wholesale textual translation, framework code, assets or proprietary output. Recovered game source is not blanket MIT. Installed PB05 remains authoritative for known contradictions; no new binary recovery or retail observation is performed here.

Decision recorded before implementation: use one transient `BubbleManager` with caller-supplied bounded RNG (`FnMut(u64) -> u64`, one draw in `0..upper` per invocation), separate configurable automatic-spawn limits, and explicitly optional per-call admission caps. Keep source branch/update/draw order. Emit typed render operations, leaving asset access, graphics state, clipping, timing, pause, scene depth and event routing to existing callers. Manager state never enters project persistence.

| Contract | Pinned source | Rule and limit |
|---|---|---|
| B01 | `BubbleMgr.cpp:46–61,67–113` | Defaults: 640×480 bounds; bubbles max10/chance10, fish max0/chance10, fish vertical speed0 and clipping enabled. Update existing bubbles/removal, automatic bubble attempt, existing fish/removal, automatic fish attempt, in that order. Newly spawned objects do not move during their spawning update. |
| B02 | `BubbleMgr.cpp:145–186,229–279` | Explicit bubble construction consumes conditional type reroll, initial Y/X draws even when overwritten, then speed. Automatic placement draws X before construction; width≤20 suppresses placement/construction. Automatic limits do not cap explicit calls. Fish require height>80, then speed/size/palette/Y/side draws. |
| B03 | `BubbleMgr.cpp:229–279,288–292` | Bubble type first draw0..3; reroll exactly once only for3. Initial position X150..171/Y400..405, six rising speeds2/2.5/3/3.2/2.3/2.8. Every update rolls horizontal shake0..1 and subtracts vertical speed. Remove strictly above bubble bounds top. |
| B04 | `BubbleMgr.cpp:294–349,383–391` | Decorative fish speeds1.3/1.7/2/2.5/3/3.5; size weights50%/30%/20%; paired30-entry tints use channel+5 saturated at255. Left entry X=boundsX−80 with positive speed; right entry X=boundsRight with negative speed. Move X/Y and decrement positive turn timer. Remove for zero horizontal speed or strictly outside X=[boundsX−80,boundsRight], without vertical culling. |
| B05 | `BubbleMgr.cpp:115–143,281–286,351–381` | Bubbles precede fish; bubble additive cel, then each fish normal base/additive color2/additive color1. Swim source80×80, columns10 advancing every2 updates; turn timer1..5 chooses turn sheet with sourceX=(5−timer)×160 and reverses mirror. Bob uses source's PI approximation and sin(5°×update)×3. Fish clip configurable. Source constructs clipped bubble graphics but actually draws through original graphics: bubble clip metadata preserves this un-clipped call. |
| B06 | `BubbleMgr.cpp:188–220` | Config changes do not clear existing effects. Changing default fish vertical speed immediately changes all existing fish; equal-value setter is a no-op. `UpdateALot` performs500 ordinary ordered updates, not direct population synthesis. |

Source file SHA-256: `BubbleMgr.cpp` = `949D424DDDB70B4295AE9B650F53B4E7610A3CB2A63143FF3A3BB322E7AF2643`; `BubbleMgr.h` = `3C94DFC484940E83E2552F2CC9847972A65CB1B20738D9048C4237028EF4980E`. The reference was inspected read-only and remained at the clean pin.

## Public API

- `BubbleManager::new(BubbleConfig)` / `Default`: empty transient collections. `config`, `bubbles`, `fish` expose state; `fish_mut` permits caller-directed decorative motion/turn changes without a second fish model.
- `spawn_bubble(x, y, rng)`: explicit construction with source RNG consumption. `spawn_bubble_capped(x, y, cap, rng)` returns admission success and consumes no constructor draws when rejected. `spawn_random_bubble` and `spawn_random_bubble_fish` return geometry admission success; direct calls intentionally do not use automatic caps.
- `update(rng)` performs one fixed update. `update_a_lot(rng)` performs exactly500 updates for initial population. `clear` retires effects while retaining configuration. Source-default constructor placement remains available through `Bubble::new` and `BubbleFish::new`; their `update` helpers expose the same lifecycles.
- Bounds, bubble/fish cap-and-chance, fish clipping and default fish vertical-speed setters preserve existing state. Vertical-speed changes retime existing fish. Limits use Rust `usize`; negative caps are outside the public contract. Chance uses the source's direct signed comparison against0..99. Coordinates/velocities must remain finite; clocks wrap safely in Rust, without a retail long-duration overflow claim.
- `render_ops` yields `BubbleRenderOp` in stable paint order, with `BubbleSheet`/`resource_key`, cel-or-rectangle source, fractional position, mirror, optional `BubbleTint`, `BubbleBlend` and optional clipping. There is no renderer dependency, asset loading or random draw during projection. Individual effects expose the same descriptor helpers.

## Integration handoff

The main implementation chat owns these joins. This change compiles the module but does not wire it into any screen or change an existing RNG stream.

1. **Board/ordinary events:** W1 `Board.cpp:132–134` sets bubble bounds `(0,82,640,398)` and automatic spawning `(0,0)`. Tick the manager on each unpaused simulation update before effect requests, as `Board.cpp:614–621` does. Admit existing `Event::GrubberBubble`/`GekkoBubble` coordinates and both ordered `WadsworthBubbles.positions` through `spawn_bubble_capped(...,30,rng)`. Current event definitions/emitters are in `src/sim.rs:1112–1142,1221–1225,8807–8813,8910–8916,10283–10288`. Emitters already supply positions; do not reroll those coordinates. Preserve effects after their originating actor disappears.
2. **Ambient Board requests:** W1 `Board.cpp:1696–1705` distinguishes ordinary cap30 from ambient cap50. For ambient calls, check `bubbles().len()<50` **before** the caller's coordinate RNG draws (X150..171/Y400..405), then construct the particle. Source ambient coordinates use `Seed::Next`, while particle construction/update use `Rand`. Pass the chosen existing streams explicitly; this module does not merge them or establish replay parity. Coordinate RNG for already evaluated explicit requests may have been consumed before capped construction.
3. **Virtual Tank Bubbulator:** `src/app.rs:3537–3591` currently draws a deterministic projection. Replace its projected particles only when an existing owner has adopted the runtime. W1 `Board.cpp:660–678` requests5..9 bubbles at500-cycle ticks13/33/53, each X=baseX+5..45 and Y=baseY−15..24, cap30. Count and coordinate draws remain caller-owned, including coordinates for requests refused at cap. The source sound request occurs on the first burst only. Current `VirtualBubbulatorPulsed` event/sound coverage must be reconciled with those three particle bursts; rendering cannot synthesize RNG or duplicate them.
4. **Interlude:** W1 `InterludeScreen.cpp:23–33` creates two managers with bubble bounds `(0,-20,640,520)`, max10/chance3, and separately runs500 warm-up updates on each. Second manager disables fish clipping and initially sets VY−1; fish cap starts0. Its update runs both managers **before** changing fish bounds/default speed/cap (`:119–157`): early fish bounds `(0,200,640,240)`/VY−1, later `(0,30,640,420)`/VY0, then fish cap3/chance3 after the existing finale threshold. Use the existing finale controller's threshold, not a new timer. Manager2 draws within the main interlude; manager1 draws in its overlay (`:165,514–519`).
5. **Renderer/lifetime:** Resolve bubble cel through owned image metadata and fish rectangles as explicit80×80 source regions. Paint normal base, additive color2, additive color1; apply tint, mirror and caller-intersected fish clips, then restore blend/color/clip state. Place ordinary effects at the existing Board overlay (`Board.cpp:5573–5575`). Retire/clear effects when that transient scene is replaced; keep them out of profile/current-save schema and disposable-preview accounting.

## Focused verification

**Observed 2026-10-10, America/Phoenix:** formatting, strict workspace/all-target lint, all eight new-module tests and an isolated debug build passed. Exactly548 other library cases were filtered out; no full gameplay suite ran. Test fixtures use explicit independently read B01–B06 draw scripts, positions, edge predicates, counters and source appearance facts; no gameplay launch or implementation-generated oracle is used.

The host `Invoke-CodexPowerShell.ps1` validated the entire ignored `.scratch/bubbles/check.ps1` before each execution (parser and automatic-variable checks passed), using `-Stage Format` followed by `-Stage Validate -RunName checks-03`. Exact successful commands:

```text
rustfmt --edition 2024 src/bubbles.rs
cargo fmt --all -- --check
cargo clippy --locked --offline --workspace --all-targets -- -D warnings
cargo test --locked --offline -p turbofish-deluxe --lib bubbles::tests::
cargo build --locked --offline -p turbofish-deluxe
```

Rust `1.95.0 (59807616e 2026-04-14)`, Cargo `1.95.0 (f2d3ce0bd 2026-03-21)`, `x86_64-pc-windows-msvc`, debug/default features. `CARGO_TARGET_DIR` was this worktree's `.scratch/bubbles/target`. The unchanged official libopenmpt0.8.9 package was preflighted by `scripts/Start-TurbofishDeluxe.ps1 -PreflightOnly -Offline -NativePackageDirectory <read-only pinned package>`, with its import library and DLL digests checked; `OPENMPT_LIB_DIR`/process-local PATH referenced it for linking/tests. No source or native package was copied from the dirty primary tree. DLL staging/game launch was not performed.

Build identity is the clean base above plus only `src/bubbles.rs`, its `src/lib.rs` declaration and this document. SHA-256 receipts for the successful source/build inputs:

| Input/artifact | SHA-256 |
|---|---|
| `src/bubbles.rs` | `D860518F4190486C83A83171854B32F822E12F3A70CDB8CF3E4C7465869FF03C` |
| `src/lib.rs` | `C507E59334BC4CBF03584689C4A6B5ABC387520F57DD54DD46DAD849797E9553` |
| `Cargo.lock` (unchanged) | `FF19F66FF0205FD6880C793EE082E26D94397B6EF4CE62DCA80FFB485ACEC5A1` |
| `.scratch/bubbles/target/debug/turbofish-deluxe.exe` | `CC3E591C7CAED61C31BE3506E4EE8E3B9BD99089D54F80B088D3D45D13A95E6A` |

The successful transcript is `.scratch/bubbles/checks-03.log`, SHA-256 `3FF5A236305BCBDE3F81A11F6FF78E79C63D3417CB2CD11E8E7B846462C4ECCE`; the local commit containing this report owns the final implementation identity. The validated `-Stage Workflow` invocation ran `scripts/Test-Workflow.ps1` successfully:32 required files,73 documents,832 links,224 anchors,42 included/35 excluded paths,142 eligible files. Scoped `git diff --check` passed; staged whitespace is checked before committing. Central authoritative records remain the main chat's integration responsibility.

Failed attempts are retained under this worktree's ignored `.scratch/bubbles/`:

- `checks-01.log` (SHA-256 `A750873DF78A85A984000AC9FFE953918E1DE3202A323CA331FE8383816B5D80`): strict lint stopped on the negated float retirement comparison (`clippy::neg_cmp_op_on_partial_ord`); no test/build ran. Explicit `partial_cmp != Less` preserves the source's strict predicate.
- `checks-02.log` (SHA-256 `20E9B22BFF94200E5A3C8D4F4223194B0DA94783C5BA4561004593D8D230652C`): strict lint passed; all eight cases stopped in their scripted-RNG fixture. Rust's field capture copied the cursor and dropped the enclosing exhaustion guard before the returned closure ran. Calling the guard's `next` method now captures the whole fixture. This was fixture ownership failure, not observed runtime behavior. The native build was not reached.

## Deferred details and cleanup

Native/visual/retail/RNG parity remains pending. Follow up on PB05 bubble clipping versus W1's unused clipped graphics copy, the turn sheet's160-pixel stride with80-pixel rectangle, exact renderer rounding/tint blending and shared-stream placement during integration. Those uncertainties do not block the compile-ready source-derived effects subsystem. No retail fidelity or playability claim is made.

No native game/saver/GUI process, service, port, save or analysis database is created or changed. Only ignored scripts/logs/build output stay in this worktree's `.scratch/bubbles/`; the pinned WinFish checkout and native package are read-only. Owned installation, primary dirty sources and genuine E80 lineage are untouched. Actionables was not updated because no top-level `workItemId` was supplied.
