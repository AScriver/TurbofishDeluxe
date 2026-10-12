# Checkpoint publication error context

Actionable#308 / E126 writer contribution, checked on 2026-10-12
(America/Phoenix). This report owns only `src/persistence.rs`,
`tests/checkpoint_writer.rs` and this document. The
[necessary-fix contract](necessary-fixes.md#checkpoint-publication-error-identity)
reserves both actual observation consumers, their integration tests and final
acceptance for root. All checks below are agent-executed offline checks.

## Result and boundary

Publication failures retain the exact destination-prefixed Display message and
original `std::io::Error` as their standard `Error::source`. The public
`persistence::io_error` helper returns the existing direct or wrapped I/O error,
preserving its kind and raw OS code. It returns no I/O match for unrelated or
serialization errors. It does not decide whether a caller should defer failure.

The existing writer still serializes before touching pending bytes, replaces
the extension with `.pending`, and returns synchronous durable failures. Its
100 ms Windows deadline, 2 ms waits, eligible errors 5/32/33, file predicates
and rename operation are unchanged. Current37/profile3 and save consumers are
unchanged by this contribution.

**Actual observer integration remains pending at this writer checkpoint.** Both
`UiEvidence::record_draw` and `Evidence::record` still use direct downcasts in
the checked base. Root's accepted plan joins the helper while preserving their
existing Windows5/32 deferral policy and tests their actual publication paths
without graphics/input. Helper compilation alone does not complete #308.

## Evidence ledger

| Claim | Class and scope | Evidence |
|---|---|---|
| C308-1 | Strongly source-supported on base716cdd0 | Writer stringifies final rename errors; both actual app observers downcast only direct I/O errors. Contract records exact consumers. |
| C308-2 | Observed on unchanged writer with a changed regression fixture | Persistent owned Windows read lock returns code5/PermissionDenied, preserves old/pending bytes and context, but exposes no I/O source. Red assertion fails. |
| C308-3 | Observed on corrected writer/test inputs identified below | Ten focused tests pass: I/O identity, ordinary source wrapping, no invented codes, serialization/pending failures, bounded lock failure, transient release and retry after unlock. |
| C308-4 | Unknown at this writer checkpoint | Actual observer defer/recovery/fatal behavior after root's joins and final integrated artifact acceptance. Root owns these checks. |

This is project persistence behavior under an explicit contract, not recovered
retail behavior. No original-game source or new external reference was studied
or incorporated for this correction.

## Retained red reproduction

Base commit: `716cdd0e960ec53d60a9a83ce7c53c721003b251`, clean managed
worktree before the test edit. Only `tests/checkpoint_writer.rs` was dirty for
the red run; the runner enforced unchanged `src/persistence.rs`.

The existing persistent-reader fixture was strengthened with a standard
error-source assertion. It owns a unique PID/counter-named directory under
`.scratch/checkpoint-error-context/fixtures/writer`, writes tick1 as the current
generation, and holds a read handle with sharing1|2 (no delete sharing). A
separate direct rename first establishes the actual OS error. The writer then
attempts tick2 publication while that same handle remains open.

```text
cargo test --locked --offline --test checkpoint_writer persistent_windows_denial_is_bounded_preserves_old_and_recovers -- --exact --nocapture --test-threads 1
```

Observed: Cargo exit101, **0 passed / 1 failed / 8 filtered**. Direct rename
reported `Some(5)` / `PermissionDenied`; writer elapsed101.6955 ms. Exact Display
and byte-preservation assertions passed before this assertion failed:
`publication error must retain its original I/O source`. This failure is
retained, not reclassified as a passing test.

Private red evidence: `red-01.log`, `red-01.identity.local.json` and
`red-01-frozen/` under `.scratch/checkpoint-error-context`. Frozen hashes:

| Red input/artifact | SHA-256 |
|---|---|
| Unchanged writer | `82CDAF4CA793E012A1E3F4ADC8FB7B980D12C5C4819147EA3B36F0DA35E2B4BD` |
| Strengthened red test | `A567B3D7C183987A7984BCEA6F42740FAC9221F218E086A0D400629761DB6AA7` |
| Red test executable | `B8C6094DE33D8686F8F48124B1F40577BBC4A2D8BE0346EFD4CC695FBEB99CBA` |

The fixture's drop removed its directory after assertion unwinding and handle
release; the runner recorded zero remaining fixtures.

## Corrected checks and identity

Checks used base716cdd0 plus only the corrected writer and focused tests below.
The report is not a build input. Each check retained the full before/after
hash list of all107 maintained Rust/Cargo build inputs; none changed during
execution. No broad unaffected suite or timing benchmark was run.

```text
rustfmt --check --edition 2024 src/persistence.rs tests/checkpoint_writer.rs
cargo test --locked --offline --test checkpoint_writer -- --nocapture --test-threads 1
cargo clippy --locked --offline --lib --bin turbofish-deluxe --test checkpoint_writer -- -D warnings
cargo build --locked --offline --bin turbofish-deluxe
```

All passed. Writer suite: **10 passed / 0 failed / 0 filtered**. Its persistent
denial lasted100.5066 ms and retained the directly observed code5/kind/context,
old tick1 and pending tick2. After closing the read handle, retry published
tick3 and removed pending bytes. The separate transient-reader test released
its handle after20 ms and published tick2 within the existing retry path.
Other checks preserve serialization-before-write, direct pending-write errors,
unrelated destination bytes, settings validation and current generation
replacement. Synthetic direct/twice-wrapped5/32/33 errors exercise only helper
classification; they do not alter or prove observer policy.

The maintained workflow checker also passed:35 required files,109 documents,
1101 local links,384 heading anchors,45 included/37 excluded path checks and
235 eligible files. This checks repository workflow, not runtime behavior.

| Checked input/artifact | SHA-256 |
|---|---|
| `src/persistence.rs` | `39C283EE2D5CD5E561511393CDFA2FCBD968B161763CD5B0139B9ADF9A9CB37B` |
| `tests/checkpoint_writer.rs` | `5145E55557D61256C700BF7F912270BF62FBB6A572C2EC82308DC81D1C28DA7D` |
| `Cargo.lock` | `1643FD7BE00AF38DB3DA764BE8E0C03CFC427EE368DCAAB7EF17A49B187F153F` |
| `Cargo.toml` | `0642F06EC0072E64EBE85FA1F5AD547C6AD825F6A57C3BDD025C9B7D2584DDFC` |
| Corrected writer test executable | `19D691A81B9044C272275A7A53B8A78100C34CA23A86BD629BB2E3F66B5478D8` |
| Debug runtime executable; observer joins pending | `EBBC5E0D08B3362923F9A0B1A2737AEEEA07900FBFD00293824ADEC0A1078A62` |

Toolchain: Rust1.95.0, commit`59807616e1fa2540724bfbac14d7976d7e4a3860`,
LLVM22.1.2, `x86_64-pc-windows-msvc`; default test/dev profiles, default features,
locked/offline resolution. Owned checkout:
`C:\Users\Austin\.codex\worktrees\c7ad\TurbofishDeluxe`. Isolated Cargo target:
`.scratch/checkpoint-error-context/cargo-target`. The existing native
libopenmpt0.8.9 amd64 import library and five DLLs were reused read-only from
the main project's private package; their exact digests are in each receipt.
Only the child check process's library/DLL environment was configured.

Private logs and identities: `format-01`, `writer-01`, `lint-01`, `build-01`
and `workflow-01` under `.scratch/checkpoint-error-context`, plus validated task-specific
PowerShell runners. The expected red failure is the only failed Cargo check
in this contribution.

The first publication-script attempt stopped on a null-valued expression while
inspecting the detached checkout's empty branch output. Readback confirmed the
same716cdd0 HEAD, no branch/staging/commit mutation and the same three owned
dirty paths. Branch capture was corrected to handle empty native output and
the whole script was parser/automatic-variable validated again before retry.

## Cleanup and handoff

Every test fixture was uniquely contained in this worktree and removed after
the run; the transient release thread joined. All Cargo/check processes exited.
Private logs, source identities and frozen red evidence remain intentionally.
No gameplay/input/render/audio/device run, live-save read/hash, human process
interference, original-install mutation, shared service/database/port change,
earned/frozen lineage edit or screensaver acceptance occurred.

Only the three reserved maintained paths are eligible for this worker's commit.
Reconcile concurrent main without reset/stash/force and preserve root's joins.
Keep #308 In progress with its claim until root confirms actual observer and
integrated artifact acceptance; then record qualifying Resolution/Passed
validation, complete/release and read back normally. This report makes no
playability, audible output, retail or human-feel claim.
