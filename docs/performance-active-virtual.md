# Active Virtual Tank view allocation

Actionable #297 starts from clean `bb34308` over runtime baseline `9188cbc`.
The [batch contract](performance-batch-295-297.md) reserves only the adapter,
shared controller module and Fish/Store/Setup controllers for this change.
The parent refresh/input/draw interfaces and fresh snapshot policy stay intact.

## Evidence and implementation decision

Source inspection confirms five full views in an idle focused active frame:
pre-input refresh, navigation input, controller input, pre-draw refresh and draw.
The first four build roster/detail/sprite/label data that input does not need.
`Interaction::advance` also changes clocks, hover and pointer captures;
Fish finance alpha and Store purchase feedback depend on its elapsed time.
The [existing integration contract](virtual-ui.md#runtime-integration) requires
fresh snapshots after each accepted or rejected request and retained original
dialog targets. These are project behavior contracts, not new retail evidence.

| Claim | Evidence and scope | State |
|---|---|---|
| AV1 | `virtual_runtime.rs` refresh/input/view and all three controller input methods construct full views; clean `bb34308`. | Strongly source-supported |
| AV2 | Navigation depends on page, fixed bounds and focus/modal predicates, without roster/render data. | Strongly source-supported |
| AV3 | Controller view advances clocks and captures; Fish finance and Store feedback must preserve ordering before pointer input. | Strongly source-supported |
| AV4 | Separate old/new optimized binaries produced byte-identical full traces; seventeen focused cases, strict lint/build and all six serial measurement cases passed. | Observed offline within identified fixtures |

Controllers now separate advancement from full presentation construction and
share current control geometry and availability between input and view. The
adapter advances refresh/navigation without constructing a full page. Controller
input uses transient hit controls; ordinary Fish input borrows sorted roster
records. Stamps/intentions retain owned targets, and draw/modal outputs remain
complete. The focused
idle adapter path constructs one full view for draw instead of five. The
existing identity stamps, dialog capture, request-pending guard and lazy inactive
capture policy remain intact.
Full `view` remains behaviorful for existing callers and builds fresh output.
The parent loop, renderer, domain, save format, dependencies and source
provenance are unchanged.

## Offline verification

Disposable production-purchased one-fish and twenty-fish fixtures from #293
validate through the production domain. Separate pre-change/changed binaries
exercise the same timed frame path and compare complete deterministic
state/view/request traces and fixture bytes. Focused checks cover finance and
feedback timing, focus/modal capture invalidation, changed dialog targets,
navigation and stale purchase day rejection. The existing #293 checks remain.

The actual adapter refresh/input/refresh/view path ran in both optimized
offline builds under the coordinator's serial timing lease. All samples,
flags, hashes and retained failed attempts are recorded here or in private
receipts. Formatting, affected strict lint and locked offline build passed.
No game process, real save, original install,
native input, audio or renderer scenario is part of this task. Native FPS and
retail/human fidelity remain unmeasured.

Both optimized adapter test binaries are prepared. The changed build passed
eight focused adapter cases, including the five preserved #293 checks, plus
finance/feedback clocks, stale purchase-day rejection and the full trace.
Two timing cases were ignored and 631 unrelated cases were filtered.
Complete trace length is 1,511,915 bytes; both SHA-256 values are
`094ca90fec00278b6481ca6da341e24bbf6f7e357d3bb26ddb9f061850bc5f5e`.
The #293 retained state digest remains
`c02b2c40d121bf58d88277fc9bb6a1226d744eed8cb7aec3a96bb7cf16c9fc37`.
The validated identity comparison read both full traces and confirmed byte
equality, independently of the logged hash. The complete tracked source sets
differ only in the five owned paths. Formatting, strict library/binary/test
Clippy and all nine existing controller regressions passed. The affected full
trace was rechecked after the final navigation presentation correction, with
unchanged bytes/digest. Seventeen unique focused cases pass; no broad unit
suite ran. The isolated optimized executable build passed. All heavy preparation
stopped before serial timing.

The final pre-join workflow/whitespace checks passed:35 required files,
93 documents,1029 local links,345 anchors,45 included and37 excluded path
checks,198 eligible files. Incoming peer changes are reconciled afterward;
root owns the joined release gate and central records.

The exact verification commands, with private target/native-library paths and
the profile overrides below, are:

```text
cargo fmt --all -- --check
cargo clippy --locked --offline --package turbofish-deluxe --lib --bin turbofish-deluxe --tests -- -D warnings
cargo test --locked --offline --package turbofish-deluxe --lib app::virtual_runtime::tests:: -- --nocapture --test-threads=1
cargo test --locked --offline --package turbofish-deluxe --lib app::virtual_runtime::tests::active_complete_state_view_request_trace -- --exact --nocapture --test-threads=1
cargo test --locked --offline --package turbofish-deluxe --lib virtual_ui::tests:: -- --nocapture --test-threads=1
cargo build --locked --offline --package turbofish-deluxe --bin turbofish-deluxe
```

The eight adapter checks used the prepared library test executable directly
after `cargo test ... --lib --no-run --message-format=json`; the commands above
give the equivalent Cargo selection. After localized lint corrections, only
the affected trace and remaining controller/build gates ran. Failed format,
compile and lint attempts remain below instead of counting as passes.

The pre-change adapter is the production `bb34308` code plus only the new
offline harness in `src/app/virtual_runtime.rs`. The changed adapter includes
the five identified source edits and the same timed frame closure. A later
lint-only fixture initializer correction is outside the timed loop and changes
neither fixture values nor the state/view/request trace cases. Both use Rust1.95.0
`59807616e1fa2540724bfbac14d7976d7e4a3860`, x86_64-pc-windows-msvc,
locked/offline dependencies, opt-level3, debug0, default dev/test codegen and
debug assertions, no LTO and two Cargo jobs. Private targets, traces and source
hash inventories live under `.scratch/performance/297/`.

| Baseline artifact | SHA-256 |
|---|---|
| Optimized old adapter test executable | `78EB359B76AEAA224B567AE06F5E54AC129DD6F6E0D53C3E5182EE239C7FE492` |
| Final optimized changed adapter test executable | `515D7BE90BED48FC563470BD0376A4BC556E4D003FD7A70BED8DAF4A9674F578` |
| Isolated optimized game executable, not launched | `F34771FDD1E183FDC111A47A23F8AC8A371CBA85DF935A0644179B27AD49793F` |
| Cargo.lock | `1643FD7BE00AF38DB3DA764BE8E0C03CFC427EE368DCAAB7EF17A49B187F153F` |
| Read-only libopenmpt0.8.9 import library | `537758E3D065281E887C0D3310BE471FCB322DE82F74F5DF969624D7A16156CF` |
| Read-only libopenmpt0.8.9 DLL | `9FA3131204F44A3B157F22F9F0AE11C5CCCB5BEFA37E91A2476567AA0F4B0D94` |

| Source | Old build SHA-256 | Final changed build SHA-256 |
|---|---|---|
| `src/app/virtual_runtime.rs` | `9511AA103BA525B61F468C82234B6B8000941033899D8EEAB7B573C8DD066F31` | `630D531E789CCDA2D128E068BC4D9A19402405A47E619ADDE872ABA33CA09A7E` |
| `src/virtual_ui.rs` | `505B6D1740941A0746DCF616A7E5F3BCB4F68773AABA8EA992CFB44891A5F65B` | `CBC07493B6C7D7CEBD8ED6B8881238CE9AD038F8F12695B0AD3DBBC62F0FDFA5` |
| `src/virtual_ui/fish.rs` | `8EDECA9B92F4378CF17AC765C8B7BEC328C00754978397757974FF4117037B09` | `E69A7948B6B14C8C1FA745E6A33452566C93715A7ACD2B434B5B57FEB4EF3F98` |
| `src/virtual_ui/setup.rs` | `D7592AF81CECD047F22194B0E509139DBBA543C9D9AF4F262202310D0721BB8A` | `07E09EBA51A5EC8DB62BBC7292439752E201950B782F8BFD3B5FD646E137DADB` |
| `src/virtual_ui/store.rs` | `D61F88C64918F1FBDAFFD24D86DA0901055FCE75E7A93AAF7D5143D466644B19` | `97D4B46553407204EE88A016CB8CE62B67BB2EB01361EB5707FB65613110556C` |

Source inventories and actual compile receipts in each artifact directory
identify the entire source/dependency set, not just its commit. The old/source
hash includes the new private-output-capable test harness; no original game or
user save contributes these fixtures.

## Serial adapter measurements

Root granted the lease at 2026-10-12T00:44:54Z after all three lanes and root
held heavy jobs. Both prepared harness processes exited successfully and the
lease was explicitly released at 00:45:50Z. No build ran during timing. This
coordinates project resource use without promising an idle host or stopping
human gameplay.

The ignored inline case runs the real adapter twice per frame for refresh,
once for idle pointer input and once for the draw view, including destruction
of the full view. Each case has 100 warmup frames and nine samples of 20,000
frames with `black_box` inputs/results. There is no timing assertion. Both
snapshots remain fresh every frame. Inputs are profile1, wallet100000,
wall seconds86500, day86400, sale grace enabled, focused UI ticks0–99 for
warmup then100–180099, and PointerMove at (320,470). No simulation tick or
random roll runs. The fixtures are prepared state, not earned progression.

```text
artifacts/baseline/active-tests.exe app::virtual_runtime::tests::measure_active_refresh_input_views --ignored --exact --nocapture --test-threads=1
artifacts/changed/active-tests.exe app::virtual_runtime::tests::measure_active_refresh_input_views --ignored --exact --nocapture --test-threads=1
```

Paths above are relative to private `.scratch/performance/297/` with the
read-only pinned native DLL directory on the process-local PATH. The validated
`measure.ps1` orchestrates both without Cargo. Each binary passed the fixture
byte-conservation assertion; complete retained state/view digests matched
between binaries for all six cases. The earlier complete trace was compared
byte for byte, covering the changing input/dialog/focus/clock cases separately.

All values below are microseconds per **adapter frame**, median (minimum–maximum)
over nine samples. Every case is retained; no median became slower.

| Fish | Page | Old adapter | Changed adapter |
|---|---|---|---|
| 1 | Store | 6.526 (6.071–7.477) | 2.100 (1.658–2.429) |
| 1 | Fish Setup | 6.830 (6.329–9.289) | 2.051 (1.898–2.124) |
| 1 | Tank Setup | 3.773 (3.481–5.000) | 1.072 (0.985–1.282) |
| 20 | Store | 9.021 (8.275–9.507) | 3.466 (3.171–4.557) |
| 20 | Fish Setup | 26.312 (25.074–31.277) | 6.228 (6.008–6.884) |
| 20 | Tank Setup | 5.697 (5.339–6.927) | 2.304 (2.164–2.505) |

The twenty-fish Fish Setup median falls 76.3%, about20.08 microseconds. This
measures snapshot/input/view allocation in the adapter, excluding the parent
app loop, native input sampling and GPU rendering. It establishes no whole-game
FPS, normal startup, retail fidelity or human-feel gain. Compiler/allocator/host
load can change these small times. No persistent cache or saved field follows.

Fixture SHA-256 values remain #293's
`6c15071d966c5ee8da9e237112ecbcc619961a7b3283c3519ab6c7fbc90c5d71`
and `04a6cf466efdec5d0dd077dbb3bf5b36290a52f589a0726ad1a0b7c5b5836690`
for one/twenty fish. Private `timings.json` preserves every sample, per-case
state digest and both executable identities. Raw timing log SHA-256 values are
`907C412F7BFEF9F7E1262088F82A57F8023245D533E553759150F9D4BE16FFC3`
(old) and `0DBFCFDBD8753AC10E6A36ECAF5735FCCDB4AAACD917EED3E7F69B7E24C08576`
(changed). The PowerShell console's direct dictionary table rendered blank
columns; native logs and numeric JSON remained intact, and reading that JSON
produced the table above without repeating timing.

## Retained preparation failures

The first pre-change optimized harness compiled, then passed six adapter cases
and failed two new fixtures before any performance measurement or controller
change. The finance case preset a subpage before the first profile admission;
the real profile reset correctly discarded it. The trace case expected another
guppy purchase in a full twenty-fish tank; the real capacity gate correctly
rejected it. Neither is a production defect. The corrected finance fixture
uses ordinary adapter navigation after admission. The full-tank trace retains
the legitimate guppy rejection; purchase dialog checks use the one-fish case.
Original logs and binary stay in private
`.scratch/performance/297/artifacts/failed-baseline-01/`. Only harness setup
changed; the old production adapter remains the baseline.

The second pre-change preparation passed the finance correction but failed
the attempted full-tank Food Upgrade stock at `install_stock` with
`InvalidOffer`. Source review confirmed product slots are constrained and
same-day stock replacement is guarded. The fallback was removed; no domain
admission was relaxed. Its logs/binary remain in `artifacts/failed-baseline-02/`.
After two failed preparations, the trace was narrowed to natural capacity
rejection on twenty fish and valid purchase on one fish. Only the failed trace
reruns on the corrected baseline; already passed old-source cases stay valid.

The first changed-source compile stopped at three `E0515` diagnostics for
returning inline navigation-array temporaries with a static lifetime. Explicit
immutable navigation constants replace those arrays; bounds and predicates
remain the same. The full compiler receipt is retained as
`artifacts/failed-changed-compile-01.jsonl`. No changed-source case ran before
this compile correction.

The first strict lib/bin/tests lint stopped at the benchmark fixture's
`field_reassign_with_default`. Its equivalent struct initializer correction
is outside the timed loop. The full failed lint receipt is retained as
`artifacts/failed-changed-lint-01.log`. Production code and the eight already
passed adapter cases are unaffected; lint/format are rechecked, and only the
remaining controller/build gates run afterward.

The next lint exposed the retained full view's typed navigation field becoming
unused in production after input moved to static entries. Source review after
the two lint failures retained this output contract and uses each constructed
button's navigation to calculate hover/press appearance. No lint was suppressed
and no output field removed. The receipt is
`artifacts/failed-changed-lint-02.log`; the affected complete trace is rechecked.

The navigation correction's first format check requested a two-line expression
wrap and stopped before further lint/tests. That wrap was applied exactly;
the receipt stays in `artifacts/failed-changed-format-01.log`.
