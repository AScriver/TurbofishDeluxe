# Necessary correctness fixes

## Renewed necessary-fix audit

On2026-10-12 the user requested another pass until no necessary finding remains.
This bounded pass inspected clean main
`2426b3ba87c315a5a5b7d3ab39c2051062f59b6d`. Three distinct source reviews
found no new verified normal-play defect; no Actionable or implementation chat
was created. Root owns these records and all source paths remain unchanged.

| Claim | Evidence and adjudication | Outcome |
|---|---|---|
| N1: Adventure actor removal, death and progression | `sim.rs::update_objects` and removal loops retain the shifted index; inactive guppies cannot die twice, corpse expiry removes the retained record and revival allocates a fresh ID. `invasion.rs` removes dead threats before reward; completion also waits for missiles. `adventure.rs` checks survivors before object updates, so final death is recognized on the next step, as its rescue regression explicitly expects. | Strongly source-supported; no additional necessary candidate |
| N2: Audio resource and settings ownership | `music.rs::MusicOwner` owns its bounded worker queues and joins on Drop; the worker owns output/module/effect lifetimes. App cue/pause changes use the existing atomic request; settings changes validate and roll back on save failure. `--mute` skips audio-owner creation and required decode failures prevent partial Title readiness. | Strongly source-supported; no additional necessary candidate |
| N3: Selector entry character leaking into Sandbox | The proposed same-frame final `a` leak is falsified: `app/menu_runtime.rs:321` drains Macroquad characters before `app.rs:6968` checks EntryCode keys. Pinned Macroquad0.4.16 `input.rs::get_char_pressed` consumes that queue. The later Sandbox collector at `app.rs:7815` cannot consume the same character again. | Source-supported counter-evidence; no fix dispatched |

The actor review covered normal Adventure death/removal/update order, not every
mode interaction. The audio review did not establish a serious consequence or
ordinary valid-asset trigger for documented nonfatal device/module failure;
automatic device recovery is not an established requirement. The Sandbox review
does not prove whether a platform can deliver a distinct delayed character on a
later frame; that remains unknown rather than a verified ordinary-play defect.
No synthetic scenario was promoted into a necessary fix.

This is source inspection, with existing relevant regression contracts inspected
but no new test, Cargo gate, native game or audio run. Records-only workflow and
whitespace checks accompany publication; the accepted E127 source/executable
identity is checked unchanged. Earlier focused test/build results remain usable
for those unchanged bytes. Human gameplay and unobserved native/retail/audio
fidelity remain pending. This closes this bounded pass, not all possible bugs.
Reported issues or new concrete evidence can start a fresh scoped Actionable.
Completed workers, native lanes and heartbeat remain closed.

## Tank chooser dispatch correction

E127 / Actionable#309 starts from clean main
`fdd3b67de2a7802769b7afef5434d8a65528196b`. A new bounded source pass found
that Time Trial and Challenge chooser actions are queued while every chooser
frame sets `ui_owned_input`, hence `ui_active`. The normal fixed-step queue drain
cannot run; Select, Menu and Escape remain pending until exit/deadline can flush
them. This is strongly source-supported by `app.rs`'s chooser handler and fixed
step branch, not a native gameplay observation. The domain and chooser model
already implement these actions; their actual parent dispatch is the defect.

| Owner | Exact maintained paths and responsibility |
|---|---|
| New #309 worker | `src/app.rs` (normal-game chooser collection/dispatch and focused regressions only), `src/app/tank_selection_runtime.rs` (narrow production-called offline dispatch boundary if needed), new `docs/chooser-dispatch.md`: fix the actual join and check both modes |
| Root | `AGENTS.md`, `STATUS.md`, `MODLOG.md`, this contract and ignored coordination/evidence; final source integration, build acceptance and tracking readback |

The worker is the sole source editor for this correction. Preserve concurrent
changes and all closed transfers. Dispatch accepted chooser actions once before
the UI pause gate, without a fabricated simulation tick. Keep production Session
eligibility authoritative, focused/overlay blocking, whole closing-frame input
ownership and existing pause/clock/save policies. Do not route arbitrary pending
gameplay through a modal or retain a chooser click for later exit. A helper's
passing tests alone are insufficient: the actual loop must call it.

Reproduce the original dispatch starvation with an isolated offline harness,
then check eligible/locked selection, Menu/Escape, both modes, blocked input,
one-time consumption and unchanged Session/Board clocks. Run focused tests,
formatting, strict affected lint and a locked/offline normal-bin build. Use only
worker-owned caches/temporary fixtures and restore any temporary registration
before publication. Keep failed evidence. Commit only owned files, reconcile and
push main without force/reset/stash, then await root's joined acceptance before
normal Actionables Done/release. No native game/window/audio run, real/live save
inspection, original installation mutation or saver work is part of this task.

The save-admission pass found only a manually supplied maximum-tick overflow
candidate with no normal-play reachability; it was not promoted to a necessary
fix. Virtual Tank/store/gift accounting inspection found no additional concrete
normal-play defect in this pass. Repeat the bounded audit after #309 is joined.

## Tank chooser completion and follow-up audit

Actionable#309's engineering is accepted at pushed main
`e90df055a5bfbcaa40b211d9e9bbad0a4121b1c5`. Root fetched and fast-forwarded
the clean checkout, reviewed the actual normal-loop dispatch and matched the
committed inputs to the checked worker build. Actionable#309 is terminal Done
version12 with qualifying Passed validation and a released claim, read back on
2026-10-12. Its exact source/test/report transfer is closed; root resumes ownership
of those paths. The [worker report](chooser-dispatch.md)
owns original extracted-gate red evidence, eight focused regressions, exact
commands, failed test compilation, cleanup and the evidence limits. All earlier
source transfers remain closed.

The actual chooser now collects Select/Menu/Escape in its frame-local vector and
calls the production Session dispatcher before the UI pause gate. Eligibility
remains authoritative in Session; a first transition consumes the remaining
frame. Focus, selector panels, normal modals, pause and earlier closing-frame
owners block input. Events and transitions join existing clock/input/checkpoint
handling, while whole-frame ownership prevents the same input from reaching the
destination. No simulation tick is invented and no gameplay queue is flushed
through the chooser.

Root's private `.scratch/coordination/necessary-309-root-acceptance.local.json`
records122 matched maintained build inputs,148 matched worker artifact rows and
17 preserved E126 artifact rows. All five successful worker validation receipts
identify the same source set before/after: eight focused tests, formatting,
strict lib/bin/test lint, locked/offline normal debug build and workflow. Root's
workflow and whitespace checks also passed. Unchanged Cargo checks were reused
instead of rerun. The normal-launch artifact was not replaced.

| Accepted identity | Value |
|---|---|
| Source set | `f3ad1057978dae2f5e148274730039dfe8da9141760b442a50d1352791b1c9e0` |
| Frozen worker receipt ID | `13caf58e3dd2a03e4c403e56325bbb7df2ab4e47559c1016b67cafae4a699e22` |
| Frozen normal-game EXE SHA-256 | `6ce06ab8be581547f0749b36fe7edd17d137d63972500028c284d8242641bc6b` |

The bounded follow-up audit found no additional necessary normal-play fix.
The earlier save-admission and Virtual Tank/store/gift reviews remain usable
because those sources did not change. The neighboring UI-owner review checked
the actual selector, VT picker, gameplay modal and Challenge-interlude joins:
the first three apply their transitions before their respective barrier; the
interlude does not raise the earlier `ui_active` gate. They do not reproduce this
queue-starvation mechanism. Root reviewed the final chooser join, eligibility,
blocking, first-transition consumption and existing save/transition consumers;
temporary red registration is absent. The manually supplied maximum-tick
candidate still lacks normal-play reachability and receives no new Actionable.

This pass establishes one corrected join and no further verified finding in
its bounded scope. It is not a bug-free or full-runtime completion claim.
Native gameplay, retail/audio comparison and human judgment remain unrun where
unobserved; ordinary gameplay remains human-owned. Preserve previous workers,
all failed evidence, frozen builds and E80/earned saves. Future human reports or
new verified defects get scoped Actionables and fresh ownership; no idle worker
or heartbeat is started just to repeat this clean pass.

## Completion

Actionable#308 is terminal Done version12 with qualifying Passed validation and
released claim, read back on2026-10-12. Worker source
`f75ad305a342d1d1148db136810bb27359fca974` and root's actual observer joins
`89dd0e23bab1d1ade56feb4d90c0fc49a2399b6d` are pushed to main. The identified
source/artifact bytes and14 focused checks below are accepted; the writer/test/
report transfer is closed. Keep the completed worktree/private evidence and all
failed/Partial tracking history. Root resumes shared ownership.

The bounded necessary-fix pass confirmed this one source/host mismatch; no
additional independently verified fix was dispatched. Future confirmed defects
or human reports get new Actionables and fresh disjoint ownership before
implementation. Completed workers, old native lanes and heartbeat remain closed.
Human ordinary gameplay continues; full-runtime/retail/audio acceptance is not
claimed by this completed correctness increment.

## Joined checkpoint for Actionable 308

Checked worker source `f75ad305a342d1d1148db136810bb27359fca974` is fetched
from pushed main. [Writer report](checkpoint-error-context.md) owns its
unchanged-writer red failure, ten focused checks, source hashes, retained script
failure and cleanup. Root joins both actual optional observers to `io_error`;
`record_draw` calls its extracted publication method, allowing offline testing
of the real UI writer without a graphics context. Windows5/32 eligibility and
all durable save behavior remain unchanged.

Root first ran `cargo test --locked --offline -j 2 --lib observation_publication_tests`
with the unchanged writer and original direct-downcast guards: two reader-lock
cases failed on OS5; two invalid-destination cases passed. Red receipt
`.scratch/coordination/necessary-308-original-observers.local.json` captures
716cdd0 plus the extraction/test dirty tree, unchanged writer SHA
`82CDAF4CA793E012A1E3F4ADC8FB7B980D12C5C4819147EA3B36F0DA35E2B4BD`,
app SHA`77D08529BA710C11DED0E796A2EF53B1FFD62F3F73B3C1E5A84B837028D687F9`
and test EXE SHA`50F635450486DCC5D8682F2CBC935162446BEC11CF8C906D19DDE3AAE153FDE4`.
The failed072652 log and original app snapshot remain private and unchanged.

After helper/caller integration, all four actual observer cases passed:
previous complete snapshots survive locks, latest UI/state publishes after
unlock, state emits one defer/one recovery diagnostic, and invalid destinations
remain errors without advancing publication state. Together with the matched
ten writer cases,14 focused contracts passed. Formatting, strict lib/bin/
checkpoint-target lint and locked/offline normal-bin debug build passed; sources
stayed unchanged throughout the joined gate. Logs are
`necessary-308-20261012-073455-*` in coordination scratch. No broad suite was
repeated. Only root's existing private debug cache was reused; E125's identified
release bytes remain preserved.

Final frozen build receipt `.scratch/coordination/necessary-308-final-build.local.json`
has ID`4424c959d520b0907ad99352d29c9aade145c7b2be09b6084b21fe09fece6b7c`,
receipt SHA`bb83720ceb7402b1346399170b0b27a53019b8978a1fe6900523015cfb66b5f9`,
source-set digest`1fc42c5db070331c525754f6d3407b38b8ed9b583a69b3c85692c7b76fbff37c`
and EXE SHA`b9d8a955b2af7bdf82f8ee7e5e7d8a056c1606c8350a575e477758a7eb5d8a53`.
All122 maintained build inputs and139 source/artifact rows match. Rust/Cargo1.95
MSVC, default dev profile and locked/offline native dependencies identify this
build. Receipt associates f75ad30 plus root's recorded dirty app join; subsequent
commit/central-record changes do not change those input bytes.

The [completion](#completion) records published joins and normal qualifying
Actionables Done/release readback. No native, audible,
retail, FPS, human-feel or full-runtime acceptance follows from these offline
checks. All uniquely contained disposable locks/fixtures were released/removed;
human processes/saves, original install, frozen/earned lineages and excluded
saver work remain untouched.

The user authorizes only necessary new fixes after the completed modal batch,
with an Actionable for each verified finding and independent new chats/worktrees.
Completed modal/refactor workers, prepared native scenarios and heartbeat stay
closed. Ordinary gameplay remains human-owned; no native run or live-save
inspection is authorized by this engineering work.

## Checkpoint publication error identity

E126 / Actionable#308 starts from main `23bd3c4914d7cf9186ecdb0821891d2ecc9c044f`.
Source-supported defect: `persistence::write_json` formats the final rename
failure as a String, while `UiEvidence::record_draw` and `Evidence::record`
downcast to `std::io::Error` to defer Windows reader conflicts. These publication
errors cannot pass either guard. An optional observer lock can therefore take
the fatal path rather than retain the last snapshot and retry the latest one.
Serialization and pending-file failures currently preserve their error types.
The defect is distinct from completed E120 retry logic and E124 writer extraction.

| Owner | Exact maintained paths and responsibility |
|---|---|
| New #308 worker | `src/persistence.rs`, `tests/checkpoint_writer.rs`, new `docs/checkpoint-error-context.md`: preserve contextual publication errors and underlying I/O identity; add focused regressions and report |
| Root | Both actual optional observation consumers and focused integration tests in `src/app.rs`; `AGENTS.md`, `STATUS.md`, `MODLOG.md`, this contract and central completion/build evidence |

The narrow shared interface is
`persistence::io_error(error: &(dyn std::error::Error + 'static)) -> Option<&std::io::Error>`
with an explicit lifetime if Rust requires it. It must return the underlying
I/O error for contextual publication failures and direct pending-file errors,
and no fabricated OS code for serialization/unrelated failures. Preserve the
existing destination-prefixed Display text and standard error-source chain.
Root alone joins the two guards; helper compilation without those callers does
not complete this outcome. Reserve additional paths/interfaces explicitly first.

Keep current37/profile3, serialized writer ownership, pending path/bytes,
serialization-before-write, old complete destination on failure and100ms
Windows retry rules unchanged. Durable save failures remain errors; only the
existing optional-observer reader-conflict policy defers them. No new retry,
schema, migration, swallowed general error or asynchronous writer is needed.

First reproduce loss of I/O identity on the unchanged writer with an owned
disposable persistent Windows read-lock fixture; preserve its failed result.
Then verify raw OS code/kind/context, transient lock success, persistent bounded
failure, old/pending bytes and retry after unlock. Root checks actual optional
classification/defer/recovery and fatal non-reader errors without graphics or
gameplay execution. Use focused formatting, strict affected-target lint and
locked/offline compilation; do not repeat broad unaffected suites.

The worker owns its new worktree, ignored scratch, private Cargo target and
disposable fixtures. Read-only native package reuse is allowed from project
`.scratch/native/libopenmpt-0.8.9/package`; no device/audio output runs or timing
benchmarks are assigned. Root owns final joined build/artifact checks. No shared
process, service, database, port, human game/save, original-install mutation,
earned/frozen lineage or screensaver work is permitted. Every other worker is
closed; future independent fixes require explicit fresh reservations.

The worker uses top-level `workItemId:308`, ordinary Inbox claim/research/Ready/
In progress flow and the installed Actionables workflow. It commits only owned
paths, fetches/reconciles origin/main without reset/stash/force, and pushes
checked progress to main. It is not alone in the repository; preserve concurrent
changes. Retain its claim until root confirms actual joined acceptance, then
record qualifying Resolution/Passed validation and complete/release normally.

