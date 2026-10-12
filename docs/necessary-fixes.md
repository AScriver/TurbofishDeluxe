# Necessary correctness fixes

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

