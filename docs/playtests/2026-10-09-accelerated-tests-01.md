# Accelerated test-clock verification — 2026-10-09

Agent-executed local checks on 2026-10-09, approximately 12:53–13:10 America/Phoenix. Result: **passed for the bounded test-clock, helper and evidence scenarios below**. The independent expectation is the explicit [test-speed design contract](../DESIGN.md#explicit-accelerated-test-runs), not a recovered retail speed setting. These runs do not accept Tank 4-2 or establish normal-speed gameplay, audio or retail fidelity. The gameplay chat's frozen E80 A32 run remained separately owned at 1x.

## Build and starting state

- Checkout: `main`, base `5a36243a1d2e853a2126808e8f1f1687295a9165`, with the accelerated changes in `app.rs`, `cli.rs`, new `timing.rs`, module registration, two persistence-test option literals and six maintained playtest/evidence scripts. Concurrent STATUS, provenance and A32 report changes were preserved. Per-file dirty source hashes are retained in the private preparation receipt.
- Associated preparation ID: `86177a741d7ad43d6bec869b288e9e6ef7d7ce74f6ec534ab25e7b5bae4dd928`; source aggregate `1a0ac009bb3b25da8db837a6e743c67524913bb9436d250adfcf86ddd0d234c2`. Successful locked offline debug preparation reported unchanged inputs. Frozen executable SHA-256: `4435CC92F72B705B90D11628EC1C5A6A7A7FAA8DEDB3DDA60CF9EDDCE6151B32`.
- Rust/Cargo 1.95.0, MSVC x64, PowerShell 7.6.6, Windows build 26200; default features, libopenmpt 0.8.9. Build command: `cargo build --locked --manifest-path Cargo.toml --offline`, after the complete maintained gate. Native testing used an immutable copied executable plus five DLLs and eight notices. The standalone frozen identity explicitly retains **unverified source association**; byte comparisons to the successful associated preparation establish the transfer, rather than inventing a Cargo receipt for the copied path.
- Owned original identity: Steam build 250752, `Insaniquarium.exe`, SHA-256 `f54c2c6ee54b00ae6da7f4bde15feb90d0a867aba6b4151b0277c1f81da3ff66`. Runtime before/after inventory checks passed. No retail process or original save was used or changed; no new external source was studied.
- Starting fixture: actual current17 first-tank `Playing` checkpoint from the earlier helper reload, SHA-256 `BD8CAEE3F5BA5C8143E0426E5EFD6C2457D554A3FE05B9AFD6D4DD107C46E8FB`. Every run used a verified byte copy in a fresh isolated save directory. Complete pre-loop identity sessions matched the copied save through the maintained typed reload comparison, including only its declared binary32 equivalences. Source saves remained unchanged.
- Native window: ordinary 960×720 captured output, muted, seed argument 42; saved RNG state is retained in the identity. Fixed update remains 28 ms; factor is process configuration. GPU/driver/frame cap were not independently inventoried.

## Repeatable procedure and observed results

Use the [maintained helper](../playtest-helpers.md) with a prepared executable and its SHA-256, a genuine current17 source checkpoint and its SHA-256, fresh `-RunDirectory`, `-Mute`, `-TestSpeed 1`, `4` or `8`, and `-QuitAfterSeconds 20`. Read the first Playing state and compare identity.start to the complete copied session. Post an owned-window click at `(320,250)`, then collect successive fresh snapshots for at least three wall seconds. Compute `(session_tick_end - session_tick_start) × 0.028 / (elapsed_seconds_end - elapsed_seconds_start)`.

Post Escape to the retained process's verified HWND, wait for paused state and request a capture. Start the pause rate interval at a fresh **post-capture** publication, then wait over 36 session ticks. Compare selected Board tick/actors/food/coins/RNG/IDs; paused session housekeeping is allowed. Post Escape again, wait for resumed Board motion, capture, and close only the retained process handle. Compare final telemetry session to the written save. Rates below are one measured interval per requested factor, not performance guarantees.

| Requested factor | Observed active rate | Observed paused rate | Owned PID | Result |
|---|---:|---:|---:|---|
| 1x | 0.9988x | 0.9907x | 6108 | Passed |
| 4x | 3.9947x | 1.0011x | 2204 | Passed |
| 8x | 7.9986x | 1.0044x | 17828 | Passed |

All three runs retained consistent speed/mode labels, exact initial/final save comparisons, selected Board pause invariants, unchanged installation checks and graceful exit 0 without forced cleanup. Agent visual inspection covered the 4x and 8x pause/resume captures and the 1x resumed capture: tank pixels rendered, the pause panel matched state, the fast-run label displayed the correct factor, and the normal scene had its ordinary tank label. This is narrow visual evidence, not human fidelity judgment.

| Additional check | Independent expectation | Observed result |
|---|---|---|
| Five-second wall limit at 8x | Deadline uses real time within batches | Graceful exit 0; final wall time 5.361 s including final checks, session time 41.580 s; actual final phase FirstTankRescue, retained honestly; PID 18376 |
| Reopen the earlier 8x save at default 1x | Speed is transient; complete saved state reloads | Initial session matched exactly; normal/1x telemetry; graceful five-second-limit exit, final wall 5.421 s and session 5.180 s; final/save matched, phase Playing; PID 15804 |
| Missing default-save case alias | Windows case aliases cannot bypass isolation | Child-only LOCALAPPDATA pointed into fresh scratch; differently cased default path rejected, exit 1; no save/evidence directories created; PID 33212 |
| Missing save/evidence case alias | Outputs must use separate directories | `Shared` versus `shared` rejected, exit 1 before directory allocation; PID 8948 |
| Event order in an accelerated batch | Every gameplay row carries its originating step tick/time | Regression passed; independent inspection of native 8x HoldFeed rows showed consecutive ticks 169→170 and session time 2.380→2.408 s within one physical observation batch |

The unattended deadline case naturally reached FirstTankRescue. No health, currency, victory, encounter skip or save-state edit was supplied. Faster scenarios need input and polling intervals appropriate to their factor.

## Validation and local receipts

The complete maintained `Test-Runtime.ps1` gate, with the owned installation supplied, passed formatting, strict workspace Clippy, locked workspace tests/build and native staging: **347 library + 5 assets + 4 fonts + 60 persistence + 3 binding = 419 checks**, including two owned music cases. Focused timing/CLI/event tests passed all 12 new regressions. Host parser and automatic-variable checks passed. Helper regressions passed 28; evidence-tool contracts passed 53, including legacy unrecorded timing, invalid labels, inconsistent labels and accelerated draft qualification.

Reproduction commands and ownership are in the [README](../../README.md#accelerated-gameplay-tests), [helper guide](../playtest-helpers.md) and [evidence guide](../evidence-tooling.md). Private receipts under `.scratch/efficiency-coordination/test-speed/` retain exact arguments, driver code, PIDs/cleanup, source/artifact SHA-256, state/events and captures:

- `full-4da3d93bd7ce4934bbde0038da315214/validation.local.json` and `build.local.json`: full gate and associated preparation.
- `focused-8e24dc505c7e4f4e8eb34bcd77eab0c4/validation.local.json`; `tools-c158914829144d1b958e41e4791ed2a9/validation.local.json`: focused and tooling execution.
- `native-7b8305ba350f4db291fab78d9e171810/native-verification.local.json`, with `speed-1`, `speed-4`, `speed-8` manifests, measurement receipts and captures: actual rate/pause/input/reload checks.
- `followup-0d6aae76cfa34fa6a7c0991f47276a7f/followups.local.json` and both native manifests: path guards, wall deadline and default-speed reopening.

These raw artifacts are local-only and ignored. Successful transitive artifact/source readback preserves mechanical evidence; generated drafts retain unreviewed gameplay claims as not tested. The sanitized results and procedure here travel with Git.

## Failures and limits

The first full gate rejected an unnecessary borrow in the speed-label formatting call; it was corrected before the passing gate. Independent source review caught Windows missing-leaf case aliases and events stamped at the last batch tick; guards and per-step event stamps were repaired and regression-checked before this build.

Native attempt01 (`native-75ca0ff834234bfd86814cb511a05a6c`) failed its pause-rate checker because it measured only a short interval spanning capture work. The factor was 1x; the result did not demonstrate an accelerated pause. Its original receipts, capture, save and clean PID8704 exit remain historical. Attempt02 used a fresh post-capture sample and a full-second interval, with no runtime-code change, and passed all factors. No third identical attempt was made.

Physical input sampling, presentation and Bilaterus connector observation remain once per rendered frame. More simulation updates per frame can change connector history, later movement and sampled traces. Exact 1x/fast trace parity, Bilaterus combat at fast speed, a complete accelerated adventure milestone, audible output, retail comparison and human feel/fidelity remain **not tested**. Default-path tests and the bounded 1x runs do not replace the required complete normal-speed gameplay acceptance. Future relevant code changes make affected checks stale.

Use acceleration for exploratory earning, stress and focused scenario iteration, followed by the [normal-speed acceptance gate](../DESIGN.md#m1-execution-acceptance) on an identified build. Current gameplay ownership and remaining work stay in [STATUS](../../STATUS.md).
