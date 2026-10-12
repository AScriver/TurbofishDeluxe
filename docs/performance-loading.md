# Cooperative Title resource loading

Actionables #296 implements PA2 under the [performance batch contract](performance-batch-295-297.md).
The source baseline is clean `bb34308`, over runtime baseline `9188cbc`.
This is an original adapter scheduling policy, not a recovered retail rule.

## Decision and invariants

Recorded before implementation: keep the existing async `task_completed` API and
report every real successful task immediately. Redraw and yield after an 8 ms
wall-work burst instead of after every completion. Always flush the final
completion once, including a load that fits entirely inside one burst. Reset the burst budget only
after the real frame wait returns, so waiting is not counted as new work.
A slow synchronous operation can exceed the budget; cancellation and yielding
remain bounded between caller-owned operations, as before.

Check quit/deadline at every completion and around each frame yield. Keep exact
completed/total counts, atomic rejection of excess tasks, explicit all-resources
success, and terminal failure. Running with a full count cannot enable Start.
No caller, gameplay/save schema, asset source, or dependency changes are needed.

Title keeps its separate 1x 28 ms `StepClock` and existing 200 ms stall clamp.
Sample monotonic elapsed wall time once per actual Title draw in loading and
Start waiting. Resource callbacks never accumulate the previous frame duration;
elapsed work and frame waiting are counted once across the loading/Start boundary.

## Deterministic scheduling results

The actual private `LoadingSchedule` and existing `TitleScreen` model ran with
synthetic dimensions, seven completed bootstrap tasks, and either 185 or 140
remaining successful tasks. These are disposable complete plans of 192/147
tasks, chosen from the audit's conservative base-resource lower bounds. They
do not enumerate the full runtime's additional UI/backend/material work.
The fake clock starts at zero; every yield adds a fake 1/60-second frame wait.
No wall timer, sleep, task decoder, GPU or game process supplies these results.

| Task work in fake clock | Remaining tasks | Before completion yields | After completion yields |
|---|---:|---:|---:|
| Instant | 185 | 185 | 1 |
| Instant | 140 | 140 | 1 |
| 1/1024 second = 0.9765625 ms | 185 | 185 | 21 |
| 1/1024 second = 0.9765625 ms | 140 | 140 | 16 |
| 1/64 second = 15.625 ms, slow control | 185 | 185 | 185 |
| 1/64 second = 15.625 ms, slow control | 140 | 140 | 140 |

Before counts follow the inspected baseline's unconditional
`task_completed` → `draw_and_yield` → `next_frame` call chain. After counts
were executed against the production schedule in
`synthetic_resource_schedules_reduce_yields_and_keep_slow_control`.
For the middle case, nine tasks cross the 8 ms budget; final partial bursts
flush once. The fake waiting component changes from 3.0833 to 0.3500 seconds
for 185 tasks, or 2.3333 to 0.2667 seconds for 140. This arithmetic is not a
measured startup duration or speedup. The slow controls show no yield reduction.
Title's independent animated fill/readiness and actual decoder costs still
affect startup; fewer completion yields do not prove earlier usable Start.

## Focused verification

Twenty focused regressions passed: nine new loader/schedule checks, five
existing non-saver Title contracts, and six existing clock checks. Expectations
come from the explicit batch contract, the unchanged Title loading model, and
its recorded source-qualified fill/readiness behavior. These tests establish
adapter/model contracts, not retail fidelity. They cover immediate exact counts,
8 ms boundary, final flush, resumed-frame budget, slow calls, per-task and
post-yield quit/deadline, atomic overflow rejection, incomplete/full-Running
Start denial, terminal failure, normal Start capture and independent 1x time.
One hundred callbacks cause zero UI ticks by themselves; draw times 62.5/93.75 ms
produce two/three total 28 ms ticks. A 10-second stall retains the existing
200 ms clamp (seven ticks), without multiplying time by task count.

Commands ran in worktree `151f` with a private
`.scratch/performance/296/target`, `CARGO_BUILD_JOBS=2`, and the read-only pinned
libopenmpt 0.8.9 package's import-library/DLL directories. No compiler-profile
override was introduced. Parser/automatic-variable checks passed for each
task-specific PowerShell runner before execution.

```text
cargo fmt --all -- --check
cargo test --locked --offline -p turbofish-deluxe --lib app::title_runtime::tests:: -- --nocapture
cargo test --locked --offline -p turbofish-deluxe --lib title_screen::tests:: -- --skip source_saver_finishes_after_ready_threshold_once_without_start_button
cargo test --locked --offline -p turbofish-deluxe --lib timing::tests::
cargo clippy --locked --offline -p turbofish-deluxe --lib --bin turbofish-deluxe --tests -- -D warnings
cargo build --locked --offline -p turbofish-deluxe --bin turbofish-deluxe
```

Formatting, strict library/binary/test-target lint and the locked/offline
normal-bin build passed. Workflow validation passed 35 required files, 93 documents,
1028 local links, 344 anchors, 45 included/37 excluded path checks and 198 eligible
files. Whitespace passed. No native runtime gate was invoked.
No failed executed regression or slower CPU timing was discarded. The separate
known malformed-save diagnostic was outside these filters and remains unresolved.

## Identity and limits

Checks ran on `bb3430822d63e34a90fff61b9f16e4bd6ec62a78` plus the owned
`src/app/title_runtime.rs` changes and this report; no other runtime/Cargo inputs
changed. Rust 1.95.0 `59807616e1fa2540724bfbac14d7976d7e4a3860`, Cargo 1.95.0
`f2d3ce0bd`, `x86_64-pc-windows-msvc`, ordinary unoptimized dev/test profiles.
The baseline source was captured byte-for-byte from Git, without compilation;
before evidence is a source/count proof rather than a baseline CPU benchmark.

| Input | SHA-256 |
|---|---|
| Baseline loader Git bytes | `E86FE0AB14C6079988BA80EA7236D1D6036645871BAF635D55315A8CD06A17AB` |
| Checked working-tree loader | `3ACB763C55995B9116AD731592F0FEA2CC906745AC249A7255B5CAE9372B2F8F` |
| Unchanged Title model | `A8BC5680E8DF672459E000B29D9CEF67A13182D56BBE5E57CA5DBDF64940F918` |
| Unchanged clock | `86A476517E21EA9EB82163121E8BB2FABD8D307B60C878AF0848D148CED6B8FE` |
| Cargo.lock | `1643FD7BE00AF38DB3DA764BE8E0C03CFC427EE368DCAAB7EF17A49B187F153F` |
| Executed loader/Title/clock test EXE | `0B8B7F16B71EB9FCA0CFAAB3855452410F3B1C244AC22AF9F39D4915A6FB22D0` |
| Built normal debug EXE | `8EF527EFE816ACA45DF5A64BC97F5A10DE8D12F5B3057D4815242253B8E6781C` |
| Built runtime rlib | `9517955D10BE856DDDFEE5F41970989958551160AE25C6D37840783266DF3600` |
| Native libopenmpt import library | `537758E3D065281E887C0D3310BE471FCB322DE82F74F5DF969624D7A16156CF` |
| Native libopenmpt DLL | `9FA3131204F44A3B157F22F9F0AE11C5CCCB5BEFA37E91A2476567AA0F4B0D94` |

The checked identity receipt inventories 87 Rust/Cargo/toolchain inputs and
ten native/build artifacts, with compiler overrides explicitly absent. Its
SHA-256 is `9FA89669952B008E43F503E2FFF97282B7AB35DDBC48C073A794FED482ED74E2`.
It identifies the dirty source state used by all Cargo gates and the complete
fake-clock fixture definition. The first identity-runner execution failed on
strict PowerShell `$LASTEXITCODE` scope after a parenthesized native pipeline;
no receipt was written. Direct native invocation followed by separate line
selection fixed it. Parser/automatic-variable validation and corrected capture
passed; `.scratch/performance/296/identity-failure-01.txt` preserves the attempt.
No runtime source changed and no Cargo gate was repeated for this tooling fix.

Raw logs, synthetic fixture definition, validated runners, source/artifact
inventories and digests stay private under `.scratch/performance/296/`.
No installation, live/frozen/earned save, native window/input/audio, screensaver
scenario, or paused automation was accessed. Fixture mode is Application,
input is successful callbacks with explicit fake timestamps, and no seed/save
is involved. No new recovered source or proprietary material was used.
Actual GPU scheduling, startup wall time, Title appearance/input/audio, ordinary
gameplay and retail/human fidelity remain unmeasured. Root owns joined integration
and central records after workers reconcile main.
