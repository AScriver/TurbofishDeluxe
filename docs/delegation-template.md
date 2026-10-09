# Compact delegation brief

Use one brief per bounded native-agent assignment. Link the existing contract and evidence ledger; summarize only facts that affect this task. Keep unknown or stale evidence explicit. The primary session owns synthesis and final decisions under [AGENTS](../AGENTS.md) and the applicable inherited delegation rules.

Initial evidence agents receive the original problem, necessary primary evidence and their own question in fresh context; disclose inherited hypotheses if isolation is unavailable. Do not supply a favored diagnosis or other agents' conclusions as established facts. Later review or implementation briefs receive only the relevant adjudicated claims and invariants. This template does not require delegation for straightforward work.

## Reusable brief

Copy these fields, replacing placeholders with the shortest sufficient assignment. Mark irrelevant resources or execution identities as not applicable.

```text
Role / bounded task: <role and one concrete outcome; review, design or implementation>
Original problem: <user outcome and the part this assignment advances>
Contract / claims: <paths/anchors, relevant claim IDs and evidence status; exact evidence references>
Build state: <checkout, branch/commit, relevant dirty/untracked inputs or hash receipt; source/executable, toolchain/configuration, original-game identity and seed/fixture/input/time when material; recorded, verified or stale>
Unresolved questions: <bounded questions that can change the result; unknowns remain unknown>
File / resource ownership: <read/edit paths; owner of databases/services/ports/fixtures/PIDs, isolation, serialized operations and cleanup, or none>
Constraints: <authorization/stop gates, task invariants and evidence limits; you are not alone, preserve others' changes and accommodate concurrent work>
Expected output: <claim-linked findings or changes, exact references, checks actually run, failed reproductions/counter-evidence, remaining uncertainty and cleanup status>
```

Reference a receipt for detailed build identity rather than pasting it. A commit alone is insufficient when relevant files are dirty or untracked. Verify affected inputs before reusing evidence after changes; retain unaffected evidence. Any state-changing experiment needs explicit resource ownership, isolation and cleanup. Report source inference, controlled experiments and native observations separately; test success alone does not establish retail fidelity.

## Worked example: 4-2 coverage review

Illustrative brief at the recorded A32 checkpoint, not a dispatched task or a new verification result. Refresh its build/working-tree summary before reuse. Runtime acceptance remains paused in [STATUS](../STATUS.md).

- **Role / bounded task:** Tester; review whether existing Bilaterus regressions cover the specified death and threat-lifetime rules. Source inspection only.
- **Original problem:** Implement the full standalone Rust runtime. A32 adds Tank 4-2 with live Nimbus, Bilaterus and Ultravores; this review checks one part of its correctness before native acceptance.
- **Contract / claims:** [4-2 outcomes](adventure-4-2.md#required-outcome-and-evidence), A32-02/03/04/07; [durable state](adventure-4-2.md#ownership-and-durable-state) and [acceptance](adventure-4-2.md#acceptance). PB61 in the [primary ledger](behavior-contract.md#tank4-2-primary-ledger) is binary-derived; remaining ordered child/fragment rules are labelled secondary-source-derived.
- **Build state:** Checkout `C:\Code\TurbofishDeluxe`; recorded A32 source `4b2d53fd6c6711ad615d083b11f9d41f745ad22f`, current17, Rust/Cargo 1.95.0 MSVC/debug. [E79 and its receipts](../STATUS.md#current-build-verification) record 407 passed checks and source/executable identities; native A32 remains untested. Verify branch/HEAD and relevant dirty/untracked inputs against the checkpoint before treating it as current. [Original build identity](../STATUS.md#environment-and-reference-build) is recorded; seed/input/time is not applicable to this source-only review.
- **Unresolved questions:** Do assertions independently cover first-head death retaining combat with no reward, final death ending the last threat with one diamond, pet-lethal pending head/bone ordering, and targetless fragments staying separate from assigned missile liveness? Does current17 retain the reachable pending/connector state?
- **File / resource ownership:** Read `src/bilaterus.rs`, `src/invasion.rs`, `src/sim.rs` and `tests/persistence.rs`; no maintained-file edits. Return review notes to the primary. Existing [4-2 owners](adventure-4-2.md#ownership-and-durable-state) retain their files. No database/service/port/fixture/process allocation; Cargo/fmt/runtime/database operations are reserved to their owners. Cleanup: not applicable.
- **Constraints:** You are not alone; preserve concurrent work. Keep A31 bundles/earned saves immutable and the original install read-only. Do not launch game/runtime processes, use analysis databases, change saves, add migrations or infer retail timing/fidelity from source tests.
- **Expected output:** Compact matrix keyed by A32 ID: independent expectation/reference, covering file/test symbol and asserted behavior, gaps/counterexamples, and strongly source-supported, disputed or unknown conclusion. State what was inspected, what remains unrun and cleanup status. Return narrow questions requiring adjudication; the primary owns acceptance decisions.
