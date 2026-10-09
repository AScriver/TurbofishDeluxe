# Playtest / comparison report template

Copy this to a new dated `.md` report in this directory. This template is **not tested** and is not a completed playtest. Fill unknown fields explicitly; do not reuse an old result for a changed build. Procedures and claim rules are authoritative in [AGENTS](../../AGENTS.md); M1 acceptance is in [DESIGN](../DESIGN.md#m1-execution-acceptance).

Prefer the [maintained evidence generator](../evidence-tooling.md#commands) to fill mechanical identity and artifact fields consistently. It leaves behavioral verdicts **not tested**; review the draft, supply the actual reproduction steps and assess discrepancies.

## Report identity

- Date/time/timezone: unknown.
- Executor: unknown; identify agent, human, or both and which checks each performed.
- Goal / claim IDs / milestone: unknown.
- Result: not tested. Limit the verdict to scenarios actually exercised.

## Original-game identity

- Store/distribution, version, executable name, PE architecture, SHA-256: unknown.
- Steam build/depot identity if applicable; asset inventory identity/digest: unknown.
- Reference source revision and exact paths/symbols, if consulted: unknown.
- Version/source mismatches: unknown; a version string alone does not prove a match.
- Installation configuration: describe relevant settings; keep the absolute path in ignored local configuration.
- Original save/profile and initial state: unknown. Specify stage/mode, fish types/sizes, hunger/growth state if observable, food level/limit, balance, purchases, pets/enemies, and progression.
- Save protection and before/after file identity checks: not tested. Keep copies and raw data private.

## Rust-build identity

- Commit, branch, dirty tracked changes, relevant untracked source digests: unknown.
- Executable SHA-256; Cargo.lock/toolchain identity; build profile/features and exact build command: unknown.
- Exact launch command and relevant asset/settings configuration: unknown; redact private paths/credentials.
- Project save/reset fixture and initial state: unknown. Never silently reuse an original save.
- Input sequence, seed/RNG state, time source, tick/update cadence, event-order instrumentation: unknown.
- Approximate/unsupported behavior and toggled debug features: unknown. Acceptance uses normal gameplay without forced outcomes.

## Shared test conditions

- OS, GPU/driver, graphics/audio backend: unknown.
- Resolution/scaling, window mode, frame cap, speed, pause state: unknown.
- Reproduction/reset steps and exact ordered actions with times/ticks and coordinates: unknown.
- Expected results come from: unknown; link independently observed original behavior, pinned source, or explicit contract and label its evidence class.
- Comparison method and independently justified tolerances: unknown. If RNG states differ, compare rules/order/bounds and explain limits.

## Scenarios

Replace each procedure with repeatable steps and record original and Rust results separately. Include growth prerequisites before expecting currency. Add cases only for the behavior being claimed.

| ID | Procedure and starting state | Independent expectation / evidence | Original actual | Rust actual | Agent / human | Verdict |
|---|---|---|---|---|---|---|
| P1 | Launch/reset to the identified first tank; inspect guppy movement and animation | Unknown | Not tested | Not tested | Unknown | Not tested |
| P2 | Click at a recorded tank coordinate to drop food; observe eligible consumption | Unknown | Not tested | Not tested | Unknown | Not tested |
| P3 | Continue normal feeding through required growth/eligibility to currency production | Unknown | Not tested | Not tested | Unknown | Not tested |
| P4 | Click the produced coin; record balance before/after and duplicate-click behavior | Unknown | Not tested | Not tested | Unknown | Not tested |
| P5 | Exercise relevant boundaries: food limits, missed clicks, expired currency, hunger/death or pause | Unknown; select only applicable recovered rules | Not tested | Not tested | Unknown | Not tested |
| P6 | Check masks/offsets/UI/input alignment and applicable sound events | Unknown | Not tested | Not tested | Unknown | Not tested |
| P7 | Check missing/invalid asset path and separate project writes; compare install identities | Unknown | Not tested | Not tested | Unknown | Not tested |

## Measurements and evidence

Record ordered events, entity states, simulation ticks/time, consumption, growth, currency generation/collection, and balance; include start/end points for measurements. State whether numbers were measured, source-derived, or hypothetical. Record sample counts and ranges rather than selecting a best result.

Raw log/capture paths and SHA-256: unknown; retain under ignored private directories. Put useful sanitized summaries here so this report survives a fresh clone. Evidence unavailable to another machine must be described as local-only, with reproduction steps. Do not commit screenshots/assets/dumps until their content and tracking policy have been deliberately reviewed.

## Failures, limits, and human playtest

- Actual mismatches, crashes, failed approaches, and their build/configuration: none recorded; not tested.
- Agent-executed checks: not tested.
- Human gameplay, feel, visual/audio judgments: not tested.
- Unsupported scenarios/versions/settings and other unperformed checks: unknown; list explicitly.
- Clean install/save integrity after testing: not tested.
- Evidence made stale by subsequent changes: unknown.

## Verdict and next action

State exactly which scenarios pass/fail/not tested, whether the evidence supports a narrow playable claim, what still needs an agent, and what needs human action. Default: **not tested**. Link any follow-up to the current [STATUS](../../STATUS.md); record change/check results in [MODLOG](../../MODLOG.md). Preserve completed reports; correct conclusions in a new report.
