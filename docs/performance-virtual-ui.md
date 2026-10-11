# Inactive Virtual Tank UI snapshot capture

Actionable #293 starts from clean `0d971b3c935841b48703cb757f8c2bf085d099d5`.
Source inspection found `app::refresh_virtual_ui` capturing the complete
`VirtualTankState` before `VirtualRuntime::refresh` discarded inactive phases.
The normal loop refreshes before input and again before draw; requests also
refresh immediately after dispatch. This is project runtime evidence, not a
new claim about the original game.

The private refresh method now takes a snapshot factory and invokes it only
after an active page is admitted. Inactive refresh still clears all controllers,
profile identity, snapshot, clock, modal targets and input captures. Active
refresh captures once every time; profile resets, page transitions, caller
blocking, focus policy and captured dialog targets retain the existing order.
No active cache, saved field, dependency or new source reference is added.
The [existing integration contract](virtual-ui.md#runtime-integration) remains
authoritative for refresh after each request before another input.

## Offline verification and measurements

Focused inline tests use disposable deterministic tanks constructed through
production stock/purchase paths and checked with `VirtualTankState::validate`.
Their independent expectations are the explicit #293 contract: no capture for
inactive phases, full stale-state reset, fresh active data before input, and
no silent recapture of dialog targets. Tests also compare complete retained
state, views and returned requests against the former eager-capture order.

The ignored measurement case compares eager capture followed by inactive
discard against lazy refresh in the same binary. It recreates the baseline
call order over the unchanged routing/controller bodies; it is not a separately
built parent executable. Capture and inputs cross `black_box` boundaries so
discarded work cannot disappear from the eager control. The harness checks
capture counts and exact state/view equality, and verifies input bytes stay
unchanged. Both policies share fixtures, clock, wallet and profile identity.
Nine samples alternate policy order, with 100 warmup refreshes per policy and
20,000 pairs per sample. Each pair represents the two normal frame refreshes.
Fixtures contain one fish or the valid maximum 20 ordinary purchased fish,
with the required Bubbulator purchase. No real save is read.

Five focused regressions passed, with one timing case ignored and 622 unrelated
unit cases filtered. The complete eager/lazy state, view and request comparison
passed; final comparison SHA-256 is
`c02b2c40d121bf58d88277fc9bb6a1226d744eed8cb7aec3a96bb7cf16c9fc37`.
Formatting, strict production library/binary Clippy, isolated executable build
and workflow checks passed. Exact commands, with `CARGO_TARGET_DIR` pointing
into worker-local `.scratch/performance/293/target`, were:

```text
cargo fmt --all -- --check
cargo clippy --locked --offline --package turbofish-deluxe --lib --bin turbofish-deluxe -- -D warnings
cargo test --locked --offline --package turbofish-deluxe --lib app::virtual_runtime::tests:: -- --nocapture --test-threads=1
cargo build --locked --offline --package turbofish-deluxe --bin turbofish-deluxe
```

The validated `scripts/Test-Workflow.ps1` also passed: 35 required files, 88
documents, 998 local links, 337 anchors, 45 included/37 excluded path checks.
No unrelated unit/integration suite ran. No failure occurred in these focused
checks; the existing malformed-save diagnostic panic stays outside this scope.

Measured build: baseline commit above plus only the identified app refresh and
runtime/test edits, Rust 1.95.0 `59807616e1fa2540724bfbac14d7976d7e4a3860`,
Cargo 1.95.0 `f2d3ce0bd`, `x86_64-pc-windows-msvc`, locked/offline dependencies.
Dev/test profiles use opt-level3, debug0, default debug assertions and no release
LTO. Four Cargo jobs were used for preparation; all compilation stopped before
requesting the serial timing lease. The read-only native libopenmpt0.8.9 package
supplied the import library/runtime path. No normal launcher artifacts changed.

| Identified artifact | SHA-256 |
|---|---|
| `src/app.rs` | `1878fbc775b75e836f4877a8f44485624b5fac9a7cb10bfd9f24c2cd88bbc8eb` |
| `src/app/virtual_runtime.rs` | `a1621d0115588fbe0a260102af75939d0eea5c82f5e051c15ce0236c002ae13e` |
| `Cargo.lock` | `1643fd7be00af38db3da764be8e0c03cfc427ee368dcaab7ef17a49b187f153f` |
| Optimized test binary | `b5877ef016c079cd3b88843623a94d13133c7589f8a1d89e51fc55072d7e91d8` |
| Isolated game executable | `1baf783ccda43b69c961ecba9297e9690bd2d0537d806703a7bbd3cec34437cf` |

Private identity and preparation command receipts stay in
`.scratch/performance/293/identity.json` and `verify-20261011T225528Z.log`.
The deterministic fixture uses wallet100000, wall seconds86500, local day86400,
UI ticks10, profile1 and focused input.

## Serial timing result

Coordinator granted #293 the first timing lease at 2026-10-11T23:01:18Z after
all three lanes confirmed heavy compilation had stopped. Other lanes held
builds; root performed only light source/metadata review. The prepared test
binary ran without compilation, passed one measurement test with 627 unrelated
cases filtered, and finished the harness in2.86 seconds (5.49 seconds for the
validated invocation). The lease was released immediately after completion.
Human normal gameplay was untouched; this is not an idle-machine guarantee.

```text
cargo test --locked --offline --package turbofish-deluxe --lib app::virtual_runtime::tests::measure_inactive_refresh_snapshot_work -- --ignored --exact --nocapture --test-threads=1
```

All values below are microseconds for **two refreshes**, reported as median
(minimum–maximum) across nine samples. The active Setup controls retain both
captures and overlapping ranges; no active-phase speedup is claimed.

| Fish | Phase | Eager baseline order | Lazy capture | Captures before / after |
|---|---|---|---|---|
| 1 | Adventure Playing | 0.252 (0.206–0.306) | 0.049 (0.036–0.104) | 2 / 0 |
| 1 | Time Trial Playing | 0.250 (0.207–0.267) | 0.045 (0.035–0.061) | 2 / 0 |
| 1 | Active Tank Setup | 2.134 (1.721–4.086) | 2.001 (1.651–2.909) | 2 / 2 |
| 20 | Adventure Playing | 2.144 (1.915–2.379) | 0.043 (0.035–0.057) | 2 / 0 |
| 20 | Time Trial Playing | 2.055 (1.893–2.149) | 0.047 (0.039–0.100) | 2 / 0 |
| 20 | Active Tank Setup | 3.349 (3.123–3.599) | 3.172 (3.092–3.705) | 2 / 2 |

Exact state/view equality passed for every measurement case; serialized fixture
bytes were unchanged. Input SHA-256 is
`6c15071d966c5ee8da9e237112ecbcc619961a7b3283c3519ab6c7fbc90c5d71` for
one fish and `04a6cf466efdec5d0dd077dbb3bf5b36290a52f589a0726ad1a0b7c5b5836690`
for20. All inactive states share the cleared-state hash above. Active Setup
state/view hashes are `4b0f0857b665db846995851da8541b2bfcbc78c6a6725b68a5ee3bad2b9904a1`
and `c2c49cfda2ec6b07ca8a74d85436cbd55852e089ccbb52517100a78cd34ba4e5`
for one and20 fish respectively. Assertions compare full values, not hashes.

The native-output log `measure-20261011T230133Z.log.output.log` has SHA-256
`a82c398428069ddc60a0535f62e59a29df51949d996f6b09f2afedde91bfc1c6`;
the validated summary is `.scratch/performance/293/comparison.json`.
This removes two complete tank clones, including fish-name allocations, from
ordinary inactive UI frames. The 20-fish controls eliminate roughly2.0–2.1
microseconds per refresh pair on this host. Compiler, allocator and host load
affect those times; capture-count elimination and semantic checks are the
stronger result. No additional cache or optimization follows these data.

Native gameplay, FPS, audible output and retail fidelity are unmeasured. Timing
isolates snapshot/routing work and does not establish a whole-frame gain.
