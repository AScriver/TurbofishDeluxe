# Shared checkpoint persistence

Actionable #300 is the bounded persistence lane in the
[E124 contract](refactor-batch-298-302.md). Baseline is
`a55f50afcb272ace4fa1cb775c4c85ec71d7bd88`, including root's public module
registration and temporary CLI-writer re-export scaffold. Only
`src/persistence.rs`, `src/settings.rs`, `tests/checkpoint_writer.rs` and this
report belong to this lane.

## Evidence and preserved policy

The source-supported baseline is `cli::write_json` and `publish_checkpoint` at
`src/cli.rs:2211-2247`. This is existing original project code; the
[publication design](DESIGN.md#windows-checkpoint-publication) and
[provenance](provenance.md) retain its platform evidence. No new external source,
dependency, save schema, retail behavior or screensaver rule is introduced.

The shared API remains
`write_json(path: &Path, value: &impl Serialize) -> Result<(), Box<dyn Error>>`.
The implementation moves without changing these contracts:

- Replace the filename extension with `pending`; do not append an extension.
- Serialize to pretty JSON before writing the pending file. Serialization
  errors preserve both existing current and pending bytes.
- Publish by rename. Never delete the current generation to force replacement.
- On Windows, retry only OS errors 5, 32 or 33 while both paths are files and
  the original 100ms deadline has not expired. Keep 2ms sleeps. Other failures
  and platforms return immediately.
- A persistent publication failure preserves the current generation and leaves
  the written pending generation. Preserve the destination-bearing
  `Cannot publish checkpoint ...` diagnostic. Earlier serialization/write errors
  retain their original error types.
- Keep directory creation, domain validation and writer ownership with callers.
  `SettingsSave::save` validates first and creates its parent before publication.

## Focused offline checks

`tests/checkpoint_writer.rs` exercises the existing contract through the new
module API: exact JSON and pending filename, replacement, serialization failure,
pending-write failure, missing parents, publication errors and retained pending
bytes, validated settings replacement/rejection, and transient/persistent Windows
read locks. Expected bytes and error behavior come from the recorded contract;
tests do not establish retail fidelity or gameplay acceptance.

Checks use Rust 1.95.0 (`59807616e1fa2540724bfbac14d7976d7e4a3860`), Cargo
1.95.0, the worktree-local ignored `.scratch/refactor-persistence/target`, two
build jobs and disposable fixtures under
`.scratch/refactor-persistence/fixtures/checkpoint-writer`. The existing
libopenmpt 0.8.9 package is read-only. Fixture cleanup resolves each exact leaf
and verifies it stays under the owned fixture root before removal.

The nine checks passed on the baseline scaffold and again with the extracted
writer/settings caller. The persistent Windows denial returned after
102.6521ms before extraction and 102.819ms afterward; both runs retained the old
bytes and then published successfully after releasing the reader. Those are
observed host timings, not a scheduler guarantee. A 2s test upper bound catches
unbounded waiting while leaving scheduling tolerance; the exact 100ms/2ms policy
is also preserved by source equality.

| Executed command | Result |
|---|---|
| `cargo test --locked --offline --jobs 2 --test checkpoint_writer -- --nocapture` | 9/9 passed before and after extraction |
| `rustfmt --edition 2024 --check src/persistence.rs src/settings.rs tests/checkpoint_writer.rs` | Passed |
| `cargo clippy --locked --offline --jobs 2 --lib --bin turbofish-deluxe --test checkpoint_writer -- -D warnings` | Passed |
| `cargo build --locked --offline --jobs 2 --lib --bin turbofish-deluxe` | Passed; no launch |
| Private source-equality/identity/cleanup script | Extracted function block equals the baseline CLI block after line-ending normalization; zero disposable fixture directories remain |
| `git diff --check` | Passed |

The extracted checks identify baseline commit `a55f50a` plus dirty
`src/persistence.rs`, `src/settings.rs`, and new `tests/checkpoint_writer.rs`;
the fourth changed path is this report. Host is Windows `10.0.26200.0`.
The private `extracted-identity.local.json` retains complete source/artifact
hashes and working-tree readback. Relevant SHA-256 values are:

| File | SHA-256 |
|---|---|
| `src/persistence.rs` | `82CDAF4CA793E012A1E3F4ADC8FB7B980D12C5C4819147EA3B36F0DA35E2B4BD` |
| `src/settings.rs` | `DC49A29368A73013994FF0B9EBD910D6A1294000361501464BFABA07C7004132` |
| `tests/checkpoint_writer.rs` | `CC36D439E05E4FAB6EFD9C4A931F7984A851EDA053D3CD2EEB5B5EF8C27A7A12` |
| `Cargo.lock` | `1643FD7BE00AF38DB3DA764BE8E0C03CFC427EE368DCAAB7EF17A49B187F153F` |
| Extracted checkpoint test EXE | `1139FB4BCEC7AA736AA9ACAD4943FA9AC9F5CB4515C482B672CB578ED11601A4` |
| Debug runtime EXE | `80CDE79AACAE05D05C5AC3304405CD415963BA81EBF2C7C5C9C70F1D11809CD9` |

The pre-extraction test executable digest was not retained; its log and
baseline commit plus unchanged test-source identity identify that historical
check. Current extracted-code evidence has the artifact identities above.
Validation scripts live only in ignored scratch, and are parser/automatic-variable
checked through the host PowerShell validator before execution. Logs remain private.
No validation check failed.
No game launch, input/audio scenario, installed-game access, human process,
live/frozen/earned save, service or tool installation belongs to these checks.

## Integration and tracking

Root registered `persistence` in `src/lib.rs` at `a55f50a`. This lane joined
`SettingsSave::save` and verified its durable replacement and rejection paths.
Root must wait for #299's CLI/profile
handoff, then replace the CLI implementation and private publication helper with
`pub use crate::persistence::write_json;` and route `Profiles::save` directly to
the new module. Existing `cli::write_json` consumers remain compatible through
that re-export, including preserved saver code. The compiling contribution
temporarily retains the baseline CLI implementation while root waits for that
handoff. These joins and affected offline checks are required before claiming
complete shared ownership.

Tracking is unavailable: the first scoped claim returned `INTERNAL_ERROR`,
correlation `2ab6d217-6c02-43bf-87a1-95d78a666c16`, with `retryMode: never`.
Scoped reconciliation confirmed #300 remains Ready v6 and unclaimed. Root
identified the shared activity-type validation failure and recorded use of the
installed workflow's required-tool availability fallback. No claim was retried,
invented or bypassed; no In progress or Done tracking result is claimed.
