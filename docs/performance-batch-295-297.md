# Optimized launch, loading and active Virtual Tank work

The user authorized implementation of the three recommendations in the
[follow-up audit](performance-audit-2026-10-11.md), three new Actionables, useful
parallel chats in separate worktrees, and all finished changes pushed to main.
The source baseline is clean `9188cbc24809c545e3422a5b53acc51f2d9fdde5`.
This is E123; the earlier #292-294 batch remains completed and preserved.

## Ownership and integration contract

| Work item | Exclusive maintained ownership | Required outcome |
|---|---|---|
| #295 | `scripts/Start-TurbofishDeluxe.ps1`, optional `scripts/Test-Launcher.ps1`, `docs/performance-launcher.md` | Normal launch selects optimized release; explicit development choice and existing `-Release` callers work; arguments, native preflight and build identity remain truthful. |
| #296 | `src/app/title_runtime.rs`, `docs/performance-loading.md` | Real completed resources update progress without a frame per resource; a bounded cooperative time budget yields and redraws; deadlines, cancellation, failure and Start admission stay correct. |
| #297 | `src/app/virtual_runtime.rs`, `src/virtual_ui.rs`, `src/virtual_ui/fish.rs`, `src/virtual_ui/store.rs`, `src/virtual_ui/setup.rs`, optional `tests/active_virtual_performance.rs`, `docs/performance-active-virtual.md` | Reduce repeated full active views while retaining clocks, captures, fresh session state, modal targets and inactive zero-capture behavior. |

Root owns every other source file, registration, Cargo files, README, this
contract, STATUS, MODLOG, requirements, DESIGN and the performance index. Workers
must request and receive an exact additional reservation before editing outside
their row. No shared `app.rs` edit is initially assigned: #296 retains the
existing async `task_completed` interface and #297 retains the lazy refresh and
input/view parent interfaces. Existing callers must compile without changing
their meaning. The controller public interfaces may acquire narrowly justified
methods within #297's ownership; existing semantics must remain compatible.

The active UI optimization cannot simply delete behaviorful `view` calls.
Advance clocks and clear focus/modal captures independently of allocation;
retain the original dialog target, request-pending guard, post-action fresh
state, Store feedback behavior and Fish finance timing. Avoid persistent caches,
new saved state or unrelated renderer/text improvements. Loader counts represent
successful tasks, and a failed or incomplete loader cannot enable Start. Keep
Title's separate 1x clock and keep gameplay clocks unchanged. Normal release
uses the existing Cargo release configuration; changing development commands
must be explicit, with exact build-profile identity.

## Resource and verification contract

Each new chat owns its worktree, ignored scratch, private Cargo target and
disposable offline fixtures. All workers are sharing a repository with others:
preserve concurrent changes, commit only owned maintained paths, fetch/reconcile
`origin/main` within the worktree, and push main without force. No reset, stash,
history rewrite or unrelated cleanup. Worker-local evidence and artifacts stay
ignored; reports retain useful sanitized checks and identities.

The pinned libopenmpt package at
`C:/Code/TurbofishDeluxe/.scratch/native/libopenmpt-0.8.9/package` may be read,
without downloading or modifying it. Builds and preparation may run in parallel
with bounded job counts. Final before/after timings require a serial lease from
root after heavy builds are held. Report the actual checkout, relevant dirty
sources, toolchain, build flags, artifact digests and fixture identity. Preserve
failed experiments and slower controls. Component/synthetic timing does not
establish native startup or game FPS.

Use Actionables' installed workflow within each exact root ID: discover only
owned/available work in that work item, research before implementation, satisfy
readiness, enter In progress before edits, record qualifying focused validation
and Resolution, then complete and release the claim. Root checks integration
and terminal receipts. Keep focused independently grounded regressions and
necessary formatting, strict lint and locked/offline build checks. Avoid broad
unchanged suites; the known malformed-save diagnostic is outside this batch.

Human ordinary gameplay, desktop/input/audio, services, original installation,
live saves and all frozen/earned lineages remain untouched. No native game or
screensaver scenario is dispatched. Old runtime workers and the coordinator
heartbeat remain paused. The complete runtime and unobserved fidelity remain
open; these tasks establish only their stated engineering outcomes.

## Completion and joined verification

All three work items are Done with qualifying Passed validation and released
claims. Each worker fetched/reconciled main in its own worktree and pushed
owned maintained files without force. Root fast-forwarded source/report main
to `d64e0ccb87d41aba03f48a616d801068d832868b`.

| Actionable | Owned commit | Owning report |
|---|---|---|
| #295 | `f3a67389d25cb19e73dafa23d1b1bbaca0ec4a02` | [Launcher](performance-launcher.md) |
| #296 | `cbf03b8cd5b1b2a76848bafccedeb1806bc07dab` | [Loading](performance-loading.md) |
| #297 | `981754523799aa60523fd20a9304ecca6d15c8f0` | [Active Virtual Tank](performance-active-virtual.md) |

The three source transfers are closed. Serial timing leases are released and
all measurement processes exited. Private worktrees, receipts, binaries and
failed experiments remain preserved. No further source edit or benchmark is
assigned to these lanes.

Final joined formatting, strict lib/bin/test-target Clippy and an actual default
release launcher `-Offline -PrepareOnly` build passed in #295's retained
worktree/target. The gate used Rust/Cargo1.95 MSVC, locked/offline dependencies,
release thin-LTO/codegen1, no profile/Rust/target overrides, jobs2 and the
read-only pinned libopenmpt package. No game, focused regression or benchmark
was rerun. Worker source and all four frozen old artifacts remained byte-exact;
all build/compiler processes exited and its checkout stayed clean.

Exact commands, with worker-local environment/package setup, were:

```text
cargo fmt --all -- --check
cargo clippy --locked --offline -p turbofish-deluxe --lib --bin turbofish-deluxe --tests -- -D warnings
scripts/Start-TurbofishDeluxe.ps1 -Offline -PrepareOnly -NativePackageDirectory C:/Code/TurbofishDeluxe/.scratch/native/libopenmpt-0.8.9/package -BuildIdentityPath .scratch/performance/295/joined-295-297.local.json
```

Strict Clippy completed in50.96seconds; actual default release compilation
completed in2m34s against retained dependencies. These are command durations,
not startup or gameplay timings. The quiet format command exited zero and is
recorded in the gate receipt, with no separate format log.

| Joined identity | SHA-256 |
|---|---|
| Unchanged source fingerprint,102 build inputs | `b3655ff3f897b95b4405a363a7e170b89e0ed407b68168935aad072da45452c5` |
| Actual default release EXE,7,880,192bytes | `16E6BC6F00EA47A70F0E0DE24067F593B791F8C467D27137F3FDB4939DA29E6E` |
| Launcher-success build identity | `106AE2532578001582EC71625EC8632B47ED0A95B09E35D6790EAA540F514F70` |
| Complete117-check worker readback | `EF4159614DBD598529EB38212DE7ADFC02128D53943BDABFC3D88CEE8855C30F` |
| Joined gate receipt | `57319EDDEDC1BE721600CE5CF9561D4E7DC72683351FC6B03A7AE7FE8D7173AE` |

Private receipts are `.scratch/performance/295/joined-295-297.local.json`,
`joined-295-297-readback.local.json` and `joined-gate-01.local.json` in the
launcher worktree. The associated identity inventories staged DLLs/notices,
toolchain/environment, source and executable; association is
`launcher-success-with-unchanged-inputs`. Pre-join build identities remain
historical, while unchanged domain/asset measurements keep their stated scope.

Root's validated private `.scratch/coordination/verify-performance-join.ps1`
matched its entire102-file source inventory,117 worker source/artifact rows,
nine selected report source/dependency hashes and all three owned ancestors.
The worker additionally matched twelve report hashes, including unchanged
Title model/clock and Cargo.toml. Root's result is
`.scratch/coordination/joined-readback-295-297.local.json`. Central records change
no checked build input. Read-only tracker reconciliation confirmed Done versions
10/9/10 for #295/#296/#297, all with null claims. Root retains central-record
publication ownership; human gameplay and paused automation policy remain current.
