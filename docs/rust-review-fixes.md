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

## Scope and order

| Order | Actionable | Outcome | Engineering |
|---|---|---|---|
| 1 | #311 | Valid final-candidate Time Trial fixture | Checked; commit follows |
| 2 | #312 | All-entry Sandbox publication projection | Pending |
| 3 | #313 | Covered pending-money debt admission | Pending |
| 4 | #314 | Required nullable `last_laser` | Pending |
| 5 | #315 | Remove proven duplicate schema inventories | Pending |
| 6 | #316 | Deliver applause through the effect path | Pending |
| 7 | #317 | Reuse existing glyph layout for clipped text | Pending |
| 8 | #318 | Retain installation digests instead of buffers | Pending |

All eight tracking records were created and researched/prepared to Ready. The
#311 claim failed with `INTERNAL_ERROR`, `retryMode: never`, correlation
`5eed16e3-d972-4d10-875c-e18e97e64b56`. Scoped list and explicit inspection confirm
Ready version6 and no claim. The server log was not located in the bounded
diagnostic pass; no root cause is asserted. The installed
`actionables-workflow` unavailable-tool fallback permits continued authorized
engineering. No denied request is retried, alternate lifecycle API is used or
tracking completion invented. Actual checked engineering and tracker states
remain separate. Later task claims are attempted only within their own scopes.

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
