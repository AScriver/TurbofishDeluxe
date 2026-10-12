# Rust review corrections

The user authorized all eight reviewed findings on 2026-10-12, in the ordered
refactoring plan below, with one Actionable per outcome and local commits after
focused verification. Root owns all edits and integration. Preserve unrelated
work, current session37/profile3, pending-money timing, native ownership and the
human gameplay boundary. No native game, audio device, owned installation, live
save or normal-launch artifact is part of this batch.

Baseline is clean `02a295112ba46b9693eb88ff89c74cbfc630fc48`, Rust1.95.0,
`x86_64-pc-windows-msvc`. Review source is unchanged from `2dd559d`; the later
commit only adds documentation. Existing workspace tests produced900 passes,
one stale Time Trial fixture failure and11 ignored tests. Format, strict lint,
locked/offline compilation/build and workflow passed. External synthetic probes
reproduced Sandbox publication rejection on reopen, covered-debt save rejection,
and missing `last_laser` acceptance. Applause rejection and the three code
simplifications are source-supported; no native/fidelity result is claimed.

Concurrent guppy boundary tests entered the original checkout during #311
validation. The repaired Time Trial test passed, but the three unrelated new
guppy cases failed and their formatting was incomplete. Preserve these real
mixed-tree results as historical; they do not qualify this batch's baseline.
Root moved only its own source/record hunks to the managed `rust-review-fixes`
worktree on `ascriver/rust-review-fixes`, based on the identified clean baseline.
The unrelated guppy files and tests remain in the original checkout. Validate
this isolated source before each commit; revalidate affected checks after joining
any later main changes.

All eight correction commits now join main `5fd37f2`, which contains the
independently completed guppy boundary correction. Source merges automatically;
STATUS/MODLOG conflicts are resolved by retaining both records. Combined checks
run in the isolated worktree before its integration commit and before a guarded
fast-forward of the still-clean original checkout. No unrelated source or
evidence is discarded. Final combined results are recorded below.

## Scope and order

| Order | Actionable | Outcome | Engineering |
|---|---|---|---|
| 1 | #311 | Valid final-candidate Time Trial fixture | Committed `c7b06fe` |
| 2 | #312 | All-entry Sandbox publication projection | Committed `8f80305` |
| 3 | #313 | Covered pending-money debt admission | Committed `587b6b6` |
| 4 | #314 | Required nullable `last_laser` | Committed `2ec1a62` |
| 5 | #315 | Remove proven duplicate schema inventories | Committed `f270e92` |
| 6 | #316 | Deliver applause through the effect path | Committed `88fbea5` |
| 7 | #317 | Reuse existing glyph layout for clipped text | Committed `35f875f` |
| 8 | #318 | Retain installation digests instead of buffers | Committed `469fde8` |

All eight tracking records were created and researched/prepared to Ready. The
#311 claim failed with `INTERNAL_ERROR`, `retryMode: never`, correlation
`5eed16e3-d972-4d10-875c-e18e97e64b56`. Scoped list and explicit inspection confirm
Ready version6 and no claim. The server log was not located in the bounded
diagnostic pass; no root cause is asserted. The installed
`actionables-workflow` unavailable-tool fallback permits continued authorized
engineering. No denied request is retried, alternate lifecycle API is used or
tracking completion invented. Actual checked engineering and tracker states
remain separate. Later task claims are attempted only within their own scopes.

#312's scoped claim also failed without mutation (`INTERNAL_ERROR`,
`retryMode: never`, correlation `fb22ed39-460d-40a7-9e3d-59541fe8150f`).
Explicit inspection confirms Ready version6 and no claim.

## Verification environment

The validated task-local `fix-checks.ps1` runs Cargo with the repository manifest,
`--locked --offline`, existing private libopenmpt0.8.9 import library/DLL path,
and an external `CARGO_TARGET_DIR`. Tests are compiled with
`cargo test --workspace --all-targets --no-run -j 2 --message-format json-render-diagnostics`;
selected generated executables run with `--test-threads 2` from external scratch.
This prevents cwd-relative test fixtures from entering the repository. Formatting
uses `cargo fmt --all -- --check`; strict lint uses
`cargo clippy --workspace --all-targets -j 2 -- -D warnings`; isolated build uses
`cargo build --bin turbofish-deluxe -j 2`. Workflow uses
`scripts/Test-Workflow.ps1`; commits use a validated script that checks HEAD and
the index, stages only the current outcome's owned paths, and reads back HEAD.
Logs and synthetic fixtures stay in task-local temporary storage.

## Final-candidate fixture

The existing test never reached its target because `selection_session` lacked
Time Trial admission. A tank-only correction would also fail the earned-roster
contract. Reuse the valid Time Trial progress, retain three initial pets and
test the final purchase after buying the first of the two remaining candidates.
The synthetic300 balance covers prices100 and200; next price is400. Validate
before purchases and after the existing max/locked/no-Adventure-victory checks.
Production gates and the shared Adventure fixture are unchanged.

All16 existing test executables pass in the isolated worktree:901 passed,
zero failed and11 ignored. The previously failing final-candidate test passes;
production Time Trial admission and the other tests remain intact. Formatting,
strict workspace/all-target Clippy, locked/offline build and workflow checks pass.
The mixed-tree failures above remain historical. No gameplay, live save or
audible output was tested.

## Shared profile publication

`Profiles::save` now publishes `Profiles::checkpoint`, which borrows ordinary
state and clones only when any entry needs Sandbox projection. Every entry is
projected in that snapshot; live sessions and transaction rollback remain intact.
CLI checkpointing uses this shared publication boundary, removing its competing
current-entry projection. A successful score publication already contains the
durable projection, so its immediate redundant write is skipped. Durable
projection validation does not certify discarded transient Sandbox fields.

Three focused regressions failed before the correction: profile transactions,
inactive Sandbox entries and score publication. All pass after it. Nine profile
persistence, ten writer, ten CLI checkpoint and four modal join tests pass
(33 passes, one ignored). The attempted Adventure Sandbox filter matched zero
tests and supplies no coverage. Strict lint, formatting, isolated build and
workflow pass. A bounded adversarial source review found no uncovered mutation
between score publication and the skip, and confirmed final timestamp rollback.
No gameplay, live save or audio device was used.

## Covered pending money

Time Trial and Challenge validation now reject negative `available_funds`,
matching their existing purchase contract, rather than rejecting negative cash
before claimed collectibles arrive. No collection or purchase timing changes.
Two synthetic persistence regressions exercise coins, pearls and larvae in both
modes, saving/reopening profiles and standalone sessions, rejecting uncovered
debt, comparing the next32 updates, and requiring exactly one arrival credit.
Each corrected fixture reproduced its mode's old validation rejection. Initial
Challenge fixture attempts also failed admission (pet confirmation, earlier tank
completion and matching score prerequisites); these fixture failures are retained
in the external logs and are not production evidence.

Both regressions pass. Existing claimed-money, Time Trial and Challenge suites
also passed during focused verification. Formatting, strict lint, compilation,
isolated build and workflow pass. #313's claim failed without mutation,
correlation `d2d57eb1-bf92-43ef-a446-5a2ef172be32`; inspection confirms Ready
version6, unclaimed. No gameplay or live saves were used.

## Combined completion

All eight outcomes are committed in execution order. The joined supporting
source is `469fde877ac3faf3d85245e5185e0af941bb1f32` plus the automatic merge of
main `5fd37f241bfbfbf8e47d96a335969ca413440f71`; remaining record edits describe
that join. A diff against main confirms the guppy source is preserved and the
only additional sim changes are the two covered-funds predicates. Both
STATUS/MODLOG histories and the incoming provenance/report are retained.

All16 workspace/all-target test executables complete:912 passed, zero failed,
11 ignored. Formatting, strict workspace/all-target Clippy, locked/offline
compilation and debug build pass. Workflow passes115 documents,1160 local
links,416 anchors,45 allowed and37 excluded path checks before the final
record entry; it is rerun on final records before commit. Whitespace checks pass.
The build uses Rust1.95.0 (`59807616e`), `x86_64-pc-windows-msvc`, existing
libopenmpt0.8.9 and external Cargo/test storage. Its executable SHA256 is
`78b994ef9c16987a86428ffcc4d99c6ed5b4cef4f48b58ce9af99e43906dc212`.
No native game or audio device was opened. No live save, owned install,
normal-launch artifact or human process was read/changed. Synthetic tests do
not establish retail, audible or human gameplay acceptance.

Final scoped inspections of #311-318 all confirm Ready version6, unclaimed.
The non-retryable claim errors prevent claim-owned resolution, validation and
Done transitions; engineering completion does not invent those tracker writes.
The validated integration runner commits only the reviewed join/records and
fast-forwards the original main checkout only if its HEAD and clean state still
match. No push, force, reset, stash or unrelated cleanup is performed.

## Fish-song applause

The actual app loop calls `MusicOwner::play_song_command`, whose only routing
decision sends applause to the existing effect path and sends real samples to
the existing validated tone path. Existing public tone/effect methods remain.
The scheduler-to-owner regression first reproduced rejection of generated
applause (`song_id=-1`), then passed with unowned, full-gain/unpitched effect
commands. All four tone samples retain song ownership, pitch/gain and rejection
of invalid IDs, pitch and gain. The existing device-free owner/channel fixture
supplies the command evidence; no audio device is opened.

All12 focused song/music checks, formatting, strict lint, compilation, isolated
build and workflow pass. Audible playback remains untested. #316's non-retryable
claim failed without mutation, correlation `c3a5c92a-a53d-4724-b20a-0f22131c9b71`;
inspection confirms Ready version6, unclaimed. No gameplay or live saves used.

## Clipped text layout

Menu, finale and tank selection drawing reuse `BitmapFont::for_each_glyph`.
Remove three `Vec<char>` allocations and their duplicate pen/kerning loops;
net50 source lines removed. Existing layer order, independent advances,
rectangle-less/missing/Unicode glyph behavior and floating-point positions
come from the already tested walker. Checked atlas lookup skips missing layers
as the previous zip did. Caller clipping, texture parameters and underline
remain unchanged. No trait, renderer layer or new test abstraction is added.

All27 existing font/layout checks pass, as do formatting, strict lint,
compilation, isolated build and workflow. The attempted library `font::tests`
filter matched zero and supplies no coverage; all ten font integration tests ran.
Native pixels/rendering remain untested. #317's non-retryable claim failed
without mutation, correlation `04af7c59-507d-4e9b-9bba-73dfd04f0028`; inspection
confirms Ready version6, unclaimed. No gameplay or live saves used.

## Installation inventory metadata

`install::inventory` retains each normalized path, byte length and raw32-byte
SHA256, releasing file contents immediately. `identify` releases executable
bytes after deriving its hash, length and PE machine. Sorting and the existing
path/NUL/u64-LE/raw-digest framing are unchanged. No cache, streaming layer,
parallel read or dependency is added. The retained content bound becomes one
file plus per-file metadata; this is source-supported, not a measured benchmark.

A synthetic installation matches independent .NET hash constants before and
after the change, including Steam metadata. It detects content changes and
rejects denied asset reads, an outside-root inventory entry and malformed PE.
The first independent calculation script failed on PowerShell array-cast
precedence before any file I/O; an explicit byte-array variable corrected it.
All three installation checks, formatting, strict lint, compilation, isolated
build and workflow pass. #318's non-retryable claim failed without mutation,
correlation `bf5364e4-ed99-4bea-be36-53117b862f41`; inspection confirms Ready
version6, unclaimed. Only temporary synthetic files were read/written; no owned
installation, gameplay, audio device or live save was used.

## Required nullable laser state

`Invasion1_2::last_laser` uses the existing required-nullable Serde pattern.
Omitted keys reject in raw sessions and both containers, including retained
boards; explicit null remains valid. The new regression first reproduced missing
key acceptance, then passed all active/retained and null/coordinate combinations.
Reopened next-click food suppression and delay clearing preserve both cases.
All44 focused persistence/invasion checks, formatting, strict lint, compilation,
isolated build and workflow pass. #314's non-retryable claim failed without
mutation, correlation `6befece8-242b-4100-9290-d600efe1a92b`; inspection confirms
Ready version6, unclaimed. No gameplay or live save was used.

## Duplicate schema inventories

Remove343 lines of OtherPet, purchase, special-pet, Challenge, Story and physical
group field inventories, their helper and the now-unused board collection.
Typed decoding retains required fields, scalar types, list shapes and lengths.
A targeted source challenge falsified the initial complete-deletion claim:
ordinary Serde structs accept positional arrays, and some old key checks also
forced objects. A new Story-array regression reproduced that acceptance despite
the first121 persistence checks passing. Retain54 lines of compact object-shape
checks at exactly the previously constrained paths (results/receipts, Story,
Challenge runs and selected actor/physical-group elements). Net CLI reduction is
289 lines; no allocation, field-name inventory or new architectural layer is added.
Existing nullable, forbidden-key, format and domain guards remain. Error wording
for removed duplicate checks may now come from Serde.

All122 persistence and17 CLI checks pass (139 passes, one ignored), including
the new array rejection. Formatting, strict lint, compilation, isolated build
and workflow pass. #315's non-retryable claim failed without mutation,
correlation `6e42f9d6-73ac-4877-897c-1f0f79b0f6ef`; inspection confirms Ready
version6, unclaimed. No gameplay or live saves were used.
