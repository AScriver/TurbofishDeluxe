# Bounded code-quality refactor batch

The user explicitly authorized Actionables for the five inspected cleanups,
one new chat and Git worktree per cleanup, parallel implementation, and coherent
commits pushed to main as work is verified. E124 starts from clean main
`ae9c114117ceeef63ae248ffc91e92a612b28c85`. Existing behavior, current37 save
schema and the human gameplay policy remain the contracts.

## Exclusive ownership

| Actionable | Complete outcome | Worker-owned maintained files |
|---|---|---|
| #298 | Shared glyph layout used by normal and clipped Help text | `src/font.rs`, `tests/fonts.rs`, `docs/refactor-font-rendering.md` |
| #299 | Direct shared current-format session admission and profile decode | `src/cli.rs`, `src/profiles.rs`, `tests/persistence.rs`, `docs/refactor-save-codec.md` |
| #300 | Existing checkpoint writer moved to persistence | `src/persistence.rs`, `src/settings.rs`, `tests/checkpoint_writer.rs`, `docs/refactor-persistence.md` |
| #301 | Matching UI drawing/layout primitives reused by render consumers | `src/ui.rs`, `src/app/virtual_runtime/render.rs`, `src/app/records_runtime.rs`, `tests/ui_primitives.rs`, `docs/refactor-ui-primitives.md` |
| #302 | Focused normal-game input, modal and audio state ownership | `src/app.rs`, `src/app/runtime_state.rs`, `docs/refactor-runtime-state.md` |

Root owns `src/lib.rs`, `src/app/help_runtime.rs`, central records, the ignored
coordination ledger, joined checks and final decisions. No other existing file
is transferred. Additional paths require a recorded exact reservation first.
Do not resume previous workers, old prepared outcome chats or the heartbeat.

## Shared interfaces and integration

Preparation and disjoint implementation run concurrently. All workers are
sharing the repository: preserve other changes and accommodate concurrent
commits. The #302 worker owns all of `app.rs` until its explicit source handoff;
root applies font/UI consumer joins only after that transfer closes. #299 owns
all of `cli.rs` and `profiles.rs` until its handoff; the writer's implementation
and publication semantics stay unchanged during that lane. Root then replaces
CLI writer ownership with a compatibility re-export and joins remaining profile
call sites. No worker edits those shared files merely to register another lane.

Root registers the new public `persistence` and `ui` modules. Their initial
scaffolds contain only the existing CLI writer re-export and an empty documented
UI module, so baseline builds stay coherent. Workers may inspect a local module
registration in their own worktree before root publishes it, but must restore
that exact temporary change before staging, reconciliation or commit. Never
reset, stash or revert unrelated files. Fetch root's scaffold before publishing
any consumer that depends on its registration.

The persistence API retains `write_json(path: &Path, value: &impl Serialize)`
and its existing result type. The codec preserves current-format JSON and field
validation/normalization contracts, rejects historical formats, and removes the
embedded-session byte round trip. No new migrations or format bump. Existing
CLI decoder entry points remain compatible for callers. Missing-field errors
must be ordinary errors, including fields currently beyond the old diagnostic
label table. Screensaver saved metadata remains intact.

#298 publishes a shared glyph-placement API from `font.rs` for root's normal
and Help renderer joins. Preserve layer order, advance, kerning, missing-glyph
and rectangle-less-glyph behavior. Clipping remains a drawing concern. #301
publishes concrete geometry/style/text helpers and routes its owned consumers
through them; root joins matching `app.rs` buttons afterward. Keep screen skins,
colors, layout, clipping and draw order. Existing Virtual Tank interaction and
domain controllers remain authoritative. No general GUI or engine framework.

#302 groups only related normal-game state and explicit reset/transition
methods. Preserve event ordering, original clock roles, pause restoration,
capture cancellation, result/save guards and existing domain mode admission.
Screensaver code is outside refactor scope. Agree any font/UI API assumption
with the recorded contract rather than editing another worker's source.

## Resource and validation contract

### Recorded tracking-tool failure

All five new chats received a claim `INTERNAL_ERROR` with `retryMode: never`.
Scoped readbacks remain Ready version6, unclaimed. Correlation
`7765257a-9a7b-451d-9ecf-2b5ac440c80d` identifies the shared installed Actionables
failure: task-detail decoding rejects an activity type at `activity[1].type`
and the claim transaction rolls back. Root verified the server log read-only;
the database, service and installed release remain untouched.

The installed Actionables workflow's unavailable-tool fallback applies:
"If the server or required tool remains unavailable, continue the user's work
and report the tracking limitation." Authorized engineering proceeds under the
exact file/resource contract while tracking stays explicitly Ready/unclaimed.
Do not invent claims, retry the denied request, bypass via another API, report
tracking completion or rewrite history. Root records actual checked engineering
completion separately and reconciles tracking only after the service recovers.
The existing font-test file is `tests/fonts.rs`; it replaces the originally
mistaken singular path in the initial task description.

Each chat owns its worktree, ignored scratch, private Cargo target and disposable
offline fixtures. Builds may run concurrently with bounded job counts. Reuse
the existing read-only libopenmpt package at
`C:/Code/TurbofishDeluxe/.scratch/native/libopenmpt-0.8.9/package`; do not download
or install tools. No services, ports, databases, original installation, human
game process, live saves, frozen builds or earned E80/earning06 lineages may be
changed, inspected for testing or cleaned up. No native game/input/audio run.

Follow the installed Actionables workflow within the exact top-level ID.
Tasks are researched and Ready; list mine then available in that work item,
claim, inspect readiness and enter In progress before implementation. Keep
credentials out of text, files and logs. Record actual focused tests, formatting,
strict affected-target lint and locked/offline compilation. Use independently
grounded placement/state/transaction expectations, not implementation-mirror
tests. No redundant broad suites or native acceptance checklist.

Commit only owned maintained paths. Fetch/reconcile `origin/main` inside the
worktree and push `HEAD:main` without force; retain concurrent commits. Use a
validated task-specific PowerShell script for Git mutations. Check failures,
non-fast-forward races and conflicts explicitly; do not push unresolved work.
Report commit identity, exact file scope, checks, failures and remaining joins.

A helper or module compile is not complete integration. Keep an Actionable open
until its production consumers are joined and affected checks pass. Root checks
each contribution and coordinates the remaining joins; worker records
Resolution and qualifying validation, completes and releases its claim after
that confirmation. Prepared reports remain preparation until checks run.
Retain private failures/artifacts. Tests establish the stated engineering
contracts, not retail fidelity, measured FPS or full gameplay acceptance.
