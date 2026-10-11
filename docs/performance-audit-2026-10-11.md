# Performance audit after Actionables 292–294

Audited clean `703cc79d577ebd868fecc8caad26d3d4435a6bbe` on 2026-10-11,
America/Phoenix. This is an audit of the current Rust runtime and launcher.
The earlier [measured fixes](performance.md#frame-profile-and-checkpoint-results)
are already present. The recommendations below are not implemented by this audit.

## Ranked findings

| ID | Priority | Finding | Next change and acceptance |
|---|---|---|---|
| PA1 | High | Ordinary launcher and documented play command choose a debug build. | Make normal play use an optimized build, with an explicit development option. Keep exact profile/build identity and compare deterministic outputs plus offline decode/render preparation costs. No measured FPS gain is claimed yet. |
| PA2 | High | Loading progress yields a frame after every successfully loaded resource. | Batch successful task accounting within a bounded frame-time budget, then redraw/yield. Preserve accurate progress, cancellation/deadline checks, failure state and the requirement that every resource is ready before Start. |
| PA3 | Low | An idle active VT UI frame constructs the full page view five times. | Separate controller clock/admission maintenance from allocating a full view; share current controls for input/navigation and rebuild when view state changes. Preserve refresh-after-request, hover/press feedback, focus clearing and captured modal targets. Measure the complete adapter path before claiming a frame gain. |

PA1 and PA2 are the strongest next targets. PA3 is bounded allocation reduction;
its measured component cost is too small to call it a major gameplay bottleneck.

## PA1: normal launch profile

[Start-TurbofishDeluxe](../scripts/Start-TurbofishDeluxe.ps1) declares an optional
`-Release` switch at line 3. Lines 88–99 invoke ordinary `cargo build` and select
`target/debug` unless that switch is supplied. The [README](../README.md#build-and-run)
normal play example omits it. [Cargo.toml](../Cargo.toml) has release LTO/codegen
configuration but no optimized development profile. No `CARGO_PROFILE_*`
override or Cargo configuration file was found in the inspected workspace,
ancestor or user Cargo locations for this audit session.

The launcher already supports `-Release`; using that supported path does not
require a renderer rewrite. This finding establishes the source/default
configuration, not the profile of an uninspected human-owned process. The prior
workers' optimized private debug/test builds do not alter the launcher's default.
An optimized normal-play default needs an identified build and deterministic
state/event checks; cross-profile timing and native FPS remain unmeasured here.
Preserve original game timing and save rules when changing compiler settings.

## PA2: resource-count frame pacing

`Presentation::load` in [app.rs](../src/app.rs), lines 557–621, awaits
`TitleRuntime::task_completed` after each non-bootstrap image, sound and font.
[TitleRuntime](../src/app/title_runtime.rs), lines 164–178, then calls
`draw_and_yield`; line 215 awaits `next_frame`. Resource lists are deduplicated,
and seven title resources are reused correctly, but every remaining completion
still yields separately.

The base lists alone contain 139 unique images, 8 fonts and 45 sounds. Subtracting
all seven bootstrap resources conservatively gives **at least 185 completion
frame yields for ordinary unmuted loading**, or **140 when muted**. Additional
UI resource sets and backend/material tasks are excluded from this lower bound.
With instant task work and a constant 60 FPS presentation cadence, those yields
correspond to about **3.08 seconds unmuted / 2.33 seconds muted**. This is a
source-derived scheduling calculation, not observed startup wall time; task
costs and actual presentation cadence affect a real run.

Keep loading responsive and honest: count every real successful task immediately,
check cancellation/deadlines between tasks, and present progress after a bounded
work budget. Do not substitute synthetic completion counts, skip error checks,
or allow Start while resources remain unavailable. A focused scheduler/model
check can verify these contracts without playing through the game.

## PA3: repeated active VT view construction

For a focused active Store/Fish/Tank Setup page, no clicks or keys, and one
PointerMove/PointerLeave input, the normal path is:

1. Pre-input `refresh_virtual_ui` at `app.rs:6769` builds/discards a view through
   `VirtualRuntime::refresh` at `virtual_runtime.rs:288`.
2. `VirtualRuntime::input` at line 343 builds a full view for navigation.
3. The selected controller's `input` builds another view: `fish.rs:224`,
   `store.rs:171` or `setup.rs:159`.
4. Pre-draw refresh at `app.rs:8504` builds/discards another view.
5. Draw obtains the full view again at `app.rs:8513`.

This happens even with an unmoved pointer: [native input projection](../src/app/virtual_input_runtime.rs)
adds PointerMove/PointerLeave every focused idle frame. More inputs and accepted
requests can add further work. The [runtime](../src/app/virtual_runtime.rs) creates
navigation vectors/strings; [fish views](../src/virtual_ui/fish.rs) clone/sort
the roster and clone targets/names; [store views](../src/virtual_ui/store.rs)
allocate offer labels, previews, controls and guidance. The inactive snapshot
guard from #293 is intact; this is a separate active-page finding.

View construction also advances clocks and clears captures on focus/modal
changes. Removing discarded calls without replacing that maintenance is unsafe.
Repeated-view equality was checked only for an unchanged snapshot at one clock;
it does not establish correctness across input, clock, profile, modal or wallet
changes. Those remain required checks for a future implementation.

## Offline component measurements

An ignored private harness calls the actual public controller `view` methods
and `VirtualBoardState::sync_fish` from the previously verified integrated
library. Root confirmed that runtime/crate/dependency sources at this audit
commit are unchanged from its `a45114d` source build. The library uses development
opt-level 2, debug 0 and no LTO; the standalone harness uses opt-level 3,
debug 0 and one codegen unit. These measurements do not represent the normal
launcher's default profile or a full native frame.

Fixtures reuse #293's production stock/purchase recipe: one fish or twenty
purchased guppies with the required Bubbulator, deterministic timestamps and
synthetic names/biographies. Both pass `VirtualTankState::validate`; neither
is earned progression or a real save. Each case has 100 warmup calls and nine
samples of 20,000 calls, with `black_box` inputs/results and complete returned
view destruction included. No timing assertion is used.

| Actual component call | 1 fish median, microseconds | 20 fish median, microseconds |
|---|---:|---:|
| FishController::view | 1.016260 | 4.685990 |
| StoreController::view | 1.104825 | 1.464570 |
| SetupController::view | 0.400315 | 0.435000 |
| unchanged VirtualBoardState::sync_fish | 0.003230 | 0.195985 |

Five times the twenty-fish FishController view median is about 23.43 microseconds. This is
arithmetic over a measured component, not a measured whole frame; it excludes
runtime navigation, snapshot creation, input work and GPU rendering. It helps
rank PA3 honestly. The unchanged roster synchronization is already below one
microsecond in these supported fixtures, so a persistent index or broad
simulation search rewrite is not justified by this audit.

Complete tank bytes stayed unchanged. Five repeated same-clock controller views
matched exactly; unchanged roster synchronization retained complete board bytes.
Tank SHA-256 values are `6c15071d966c5ee8da9e237112ecbcc619961a7b3283c3519ab6c7fbc90c5d71`
(one fish) and `04a6cf466efdec5d0dd077dbb3bf5b36290a52f589a0726ad1a0b7c5b5836690`
(twenty fish), matching the earlier fixture inputs.

Private harness, validated PowerShell runner, full sample distributions, resource
counts and identities are in `.scratch/performance/audit-20261011/`. Compiler:
Rust 1.95.0 `59807616e1fa2540724bfbac14d7976d7e4a3860`, x86_64-pc-windows-msvc.
Measured library SHA-256:
`DA17EE789792B2300E40F758E884A85442814081E51F75432E96F2280113A917`.
Harness SHA-256:
`0E32268728131347AD270F8CD10084F52109594F2234DC9CF7C81EBA1E936F3A`.
Offline executable SHA-256:
`E1E16FFD8BCA0796F3AFFD8B75D6D388D3655D8A5D2EFB91BFDCA2478E588455`.

## Lower priority and limits

Bitmap text in `app.rs:510` allocates a character vector per draw. VT paragraph
rendering in `app/virtual_runtime/render.rs:582–603` wraps strings and rescans
glyph heights per paragraph. Precomputing immutable font height and avoiding
character-vector allocation are source-supported cleanup candidates, but no
separate timing or major impact was established; keep them below PA1–PA3.

Audio already bounds command/effect queues and sleeps when it has no work;
unchanged cues do not enqueue commands. No audio rewrite follows without a
measured issue. No new spatial structure, async save queue, save schema change,
dependency or upstream reference was introduced. The known malformed-save
diagnostic failure remains a separate correctness follow-up.

The harness compiled and completed successfully through the PowerShell parser
and automatic-variable validator. Documentation/workflow checks passed: 35
required files, 91 documents, 1020 local links, 344 heading anchors, 45 included
and 37 excluded path checks, and 196 eligible files. Whitespace checks passed.
No game window, audio device, live save or
original-game process was accessed. Existing Cargo gates were not repeated
because runtime sources did not change. Native FPS, startup wall time,
cross-profile gains and input/render/audio/human fidelity remain unmeasured.
Actionables returned no owned work item; no new work-item scope was supplied,
so no backlog was discovered, created or updated. Prior lanes and automation
remain paused.
