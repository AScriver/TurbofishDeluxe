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
