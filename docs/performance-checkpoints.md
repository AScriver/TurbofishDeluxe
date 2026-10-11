# Profile checkpoint cost and publication boundaries

Actionable #294 preserves current37/profile3, all profile/retained-scene payloads,
Sandbox origin projection, synchronous saves and the existing bounded atomic
publication retry. This is project persistence work, with no new retail-behavior
claim. Only disposable offline fixtures are used; gameplay belongs to the human.

## Decision

Ordinary sessions need no checkpoint copy when mode is not Sandbox and both
transient Sandbox fields are absent. Borrow that complete manager for the same
validation/encoding/publication path. Otherwise clone the manager once and apply
the existing current-session projection in place. Retain the public cloning
projection for other callers. Include private Sandbox origin in the predicate:
checking mode alone could change validation of residual transient fields.

The unchanged `Profiles::record_events` bool contract proves a completed score
publication and in-memory commit. Its save covers the following checkpoint only
when projection is identity and no later profile mutation intervenes. Invalidate
that proof before selector/saver operations. Unscored transitions, explicit saves
and projection-changing Sandbox checkpoints still publish synchronously.

An exit save can cover final cleanup only when no subsequent durable field
changes. Keep the final wall-time stamp; if it differs, save again. All exits
without an earlier successful checkpoint retain their final save. Publication
errors propagate and a failed final save restores the previous in-memory stamp;
no asynchronous queue or serializer/schema change is involved.

## Evidence ledger

| ID | Class and scope | Evidence |
|---|---|---|
| C1 | Observed offline and source-supported; copy removal | Ordinary preparation borrows; Sandbox clones one manager without another current-session clone. Preparation timings below use the production helper. |
| C2 | Observed offline and source-supported; Sandbox exception | Complete baseline/patched bytes match; production decode, residual-board and private-origin-only cases pass. Public projection restores mode/phase/seed and clears both transient fields. |
| C3 | Observed offline; score boundary | All three score-result families save once; locked-destination follow-ups create no pending file. Denied score publication retains old bytes and in-memory authority. |
| C4 | Source-supported app wiring plus observed helper tests | Post-score saver branches invalidate equivalence. Identical exit timestamps skip; changed timestamps and fallback exits publish. GUI paths are not executed. |

Coordinator granted only adjacent projection helpers/public-method refactor in
`adventure.rs` and the existing Quit arm/final guard beyond the original lane.
Other workers own score-event and Virtual UI refresh code. Central records remain
coordinator-owned.

## Offline measurements

The ignored `cli::checkpoint_tests::measure_checkpoint_costs` harness
uses fixed seed `0xa294`, fixed synthetic profile nonces, one ordinary profile,
eight profiles with retained Time Trial Boards, and eight with active Sandbox.
Each fixture validated and decoded through the production current-format
decoder. Complete projected bytes matched before measuring. Nine samples
measure checkpoint preparation alone (2,000 iterations), full validation/encoding/
publication (16 iterations), and ordinary result score/checkpoint pairs (16
prebuilt managers). No test fixture claims earned gameplay.

Baseline mechanically extracts existing copies into a private `Cow::Owned`
helper and adds an always-write score-follow-up shim; checkpoint behavior remains
baseline behavior. Both executables were frozen before merging other performance
lanes. Coordinator's serial lease ran2026-10-11 at16:09 MST (23:09 UTC), with
all other heavy builds stopped; baseline/after cases passed in2.46s/1.44s and the
lease was released promptly. Host: AMD Ryzen7 9800X3D,8 cores/16 logical CPUs,
Windows10.0.26200.0. Rust1.95.0 MSVC, Cargo1.95.0; dev opt-level2, debug0, LTO
disabled,16 codegen units and four build jobs, worker-local target/output.

Preparation includes owning/dropping the projected manager but no validation,
encoding or disk work. Full checkpoint includes all those operations. Result
pairs use prebuilt fresh managers, one qualifying synthetic Time Trial event,
and the production score-follow-up decision; input setup is outside timing.

| Fixture | Preparation median, microseconds (range), before -> after | Full checkpoint median, milliseconds (range), before -> after | Score/result pair median, milliseconds (range), before -> after |
|---|---|---|---|
| One ordinary profile | 1.468 (1.210-2.288) -> 0.010 (0.009-0.015) | 1.259 (1.170-1.308) -> 1.193 (1.065-1.343) | 2.493 (2.308-2.763) -> 1.262 (1.157-1.447) |
| Eight profiles, retained Time Trial scenes | 13.181 (10.923-18.647) -> 0.011 (0.009-0.035) | 2.352 (1.722-3.623) -> 1.800 (1.595-2.359) | 4.515 (3.872-5.281) -> 2.136 (1.854-2.368) |
| Eight profiles, current Sandbox | 12.917 (10.966-21.939) -> 9.987 (9.130-12.892) | 1.838 (1.743-3.175) -> 1.907 (1.655-2.339) | Not measured |

Ordinary saves remove both copies; Sandbox preparation removes the extra session
copy and measured22.7% lower median. Qualifying ordinary result pairs publish
once instead of twice, with49.4%/52.7% lower medians. Full checkpoint ranges
overlap and Sandbox's full median rose3.7% despite lower copy cost; filesystem/
host variability prevents attributing every end-to-end difference to copying.
These are offline fixtures, not FPS, retail fidelity or native playability gains.

| Fixture | Input SHA-256 | Complete checkpoint SHA-256 / byte count | Result-pair checkpoint SHA-256 |
|---|---|---|---|
| Single | `4059c582c5a1ca7ffefd3f43c7bb5ebd5954bf2d09f62481599dae1096ec447a` | `21fca8faa75cbf548b72ea2928ccdcdb503e658109794b1a56e7b3547bd1d31a` /7221 | `2a2c3468cd7370c522c5e1e89533b70245604097485dd4d61fe34480304ddf96` |
| Retained8 | `12d45c73d3c06ad0edcce9e396e6429c73a5bf074957c491060f9f98822f4275` | `16694be468561f2d0ec4f43c7e6d80c246478182bf05bc13556fbd1564855e7f` /116495 | `e550c72f02700b2b09e3dc70d433469e7468e9b4af23fb83ed9e7dd774e392cb` |
| Sandbox8 | `65028a6fb0e7e43a196911b5dd7194d62c132d63f606b81b4fe1f0ad975bb7ba` | `9804a8e6d54969122e543969e3fed68cd3bd9f5018b1c0ce72290dcd4692294d` /116592 | Not measured |

All paired hashes, byte counts, fixture shapes, iteration counts and decoder
results match. Copy/result timing builds retain the unchanged baseline
`record_events`; worker#292's later optimization is excluded from these timings.

## Build identity and focused checks

Measurement base commit: `0d971b3c935841b48703cb757f8c2bf085d099d5`, plus the owned
checkpoint/app/projection changes and fixed inline test harness described here.
Baseline binary SHA-256
`63772BDD8EA587666E225BEC6598F69620252B4B0DB940487B377DD4AF972113`;
patched binary SHA-256
`DB7A65593E21BAC38F9B96BA0F53BD6B1F4DB2DFD63FE104A67C8068E6CBAA53`.
Baseline `cli.rs` source SHA-256
`6F1BBB4D4356D88CF44D59613396CEC4F0BB9D2A9A571C3BECA86F3315724F5B`;
patched `cli.rs`
`0CAFDD675DF53F8471A1B27E9C8B034913FF87DB287420411B1182A51C1FE885`,
`adventure.rs`
`D020F2065F9F4838B6CF1B6D01396E9C2DFE68CA9DD91303E4C1291FF471402A`,
`app.rs`
`15CF638DC8EA61A74F6D489DCAD26BF084553F91905239DD99A50C42848DE2E2`.
Cargo.lock SHA-256
`1643FD7BE00AF38DB3DA764BE8E0C03CFC427EE368DCAAB7EF17A49B187F153F`.
Private source snapshots, binary identities, nine samples and comparison are in
`.scratch/performance/294/`; retained logs are `baseline-Tests.log`,
`after-Tests.log`, `baseline-timing.log`, `after-timing.log` and `comparison.json`.

Preparation failure retained: the first test build rejected `{:x}` formatting
of the SHA-256 digest array (`E0277`, no `LowerHex`, sha2 0.11). Changed the fixture
identity helper to format individual bytes; this was test-harness preparation,
with no game/save defect or usable timing result.

Baseline complete projection/production decoding passed for the single, retained,
Sandbox and empty-manager fixtures (one focused test, one timing case ignored).
Eight patched focused tests passed in0.88s, with only the timing case ignored.
They exercise origin-only/residual-board projection, all three score-result
families, unscored transitions, explicit saves, intervening mutations,
nonqualifying results, Sandbox raw-score projection, identical/changed exit
timestamps, empty-profile exits, invalid inactive profiles, and sustained Windows
publication denial with old-file/score/final-stamp preservation. Locked result
destinations and absence of a pending file prove equivalent follow-ups do not
attempt publication. These are offline persistence tests, not GUI scenarios.
Formatting and workflow/link/allowlist checks passed. Fixture and measurement code are
byte-identical after the change, SHA-256
`83B422DB64B2D4243B5C3BDEDCB1D271799077EA66462A72FA9CD3953636D048`.
Integrated main reconciliation, affected checks and final lint/build remain
pending; the following exact command scope will be recorded after execution.

```powershell
& (Get-Command cargo).Source @('fmt', '--all', '--', '--check')
& (Get-Command cargo).Source @('test', '--locked', '--offline', '--lib', 'checkpoint_tests::', '--', '--nocapture', '--test-threads=1')
& (Get-Command cargo).Source @('clippy', '--locked', '--offline', '--lib', '--bin', 'turbofish-deluxe', '--tests', '--', '-D', 'warnings')
& (Get-Command cargo).Source @('build', '--locked', '--offline', '--bin', 'turbofish-deluxe')
```

Configure the read-only libopenmpt0.8.9 amd64 import library/DLL paths and an
isolated `CARGO_TARGET_DIR`, with the profile overrides above. The ignored timing
test runs only under a serial lease; call the frozen test executable with
`cli::checkpoint_tests::measure_checkpoint_costs --ignored --exact --nocapture
--test-threads=1`. Each mutation/validation script was parser/automatic-variable
checked and executed through `Invoke-CodexPowerShell.ps1`.

No original installation, real/live saves, normal launcher artifacts, frozen E80
or earning06 lineage is read or changed. The unrelated malformed-save diagnostic
panic at `cli.rs:1247` remains outside this lane. Native/render/audio/retail/human
acceptance is unperformed here.
