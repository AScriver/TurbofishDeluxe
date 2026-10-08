# Runtime approach and first milestone

Product scope: [requirements](requirements.md). Current evidence and environment: [STATUS](../STATUS.md). Implementation is authorized through the full runtime scope. Recovered rules are authoritative in [behavior contract](behavior-contract.md).

## Chosen route

Use route 5, engine recreation: one standalone Rust program reads the player's own game data and owns simulation, presentation, input, and project persistence. The route comes from the guides recorded in [provenance](provenance.md#primary-reference). There is no host/guest bridge or loader requirement.

Recover the smallest original-game behavior contract, choose the necessary libraries, then integrate a real vertical slice. Do not scaffold networking, a plugin SDK, a general engine, or a multi-crate workspace just because an example uses them.

## Intended boundaries

| Responsibility | Intended boundary | Decision status |
|---|---|---|
| Simulation and game state | Own fish, food, currency and ordered updates; controlled tick/actions/PRNG | Explicit serializable Rust state; effective retail cadence/RNG still unresolved |
| Asset access | Read install assets and metadata without mutating the install | Direct decoding, no conversion cache for the first slice |
| Presentation | Render owned sprite frames/masks/fonts; trigger audio from events | macroquad chosen; corrected camera/facing exercised, full fidelity pending |
| Input | Convert actual window input to ordered simulation actions | Logical640×480 inverse letterbox mapping exercised |
| Persistence | Separate project saves/configuration from retail data | Versioned JSON under LOCALAPPDATA/TurbofishDeluxe or override; broader progression pending |
| Validation | Independently derived rule tests plus original/Rust scenarios and runtime evidence | Required; no gameplay baseline exists yet |

The first implementation uses one Rust 2024 crate, macroquad for the Windows window/2D/input/sound path, image for direct owned-image decoding and separate explicit simulation state. Dependencies are recorded in [provenance](provenance.md#rust-dependencies). The owned assets use sprite sheets and companion masks; no conversion cache is needed for the first slice. A 28ms fixed-tick accumulator separates the source-derived simulation cadence from rendering. Cap stalled-frame catch-up at the source framework's 200ms bound and record stalls. Do not silently accelerate simulation for an acceptance run.

This is a source-informed implementation of secondary functional expectations with new Rust organization; no WinFish/PopLib source files are imported. PRNG control and rendering separation aid testing but do not establish exact retail PRNG sequence. Owned bitmap fonts are parsed; MO3 tracker order/loop audio and full save semantics remain pending. The installed main EXE's base-app10ms field does not establish the game tick; identify the actual game payload before changing the28ms hypothesis.

## Next task: original-game contract

Recover the first Adventure tank's guppy/food/currency loop against the identified local build, without relying on an assumed version match, and implement it once its evidence permits it. Continue automatically after validation to the next milestone; this recovery step and M1 are internal gates in the full runtime goal.

Existing [shared analysis tools](analysis-tools.md) are available for bounded static analysis and instrumentation where the recovery needs them. Inspect existing game-specific research first, then select the tool and record the target identity and observation limits; none was used on the game during setup.

Acceptance criteria for the recovery step:

1. Reconcile Steam metadata, executable identity, and asset inventory from STATUS. Identify the actual launcher and assets needed for M1; record resource metadata and decoding assumptions with evidence. Keep raw material private.
2. Pin WinFish and insaniquarium-port, inspect their relevant behavior/resource code and licenses, and record study/reuse boundaries in provenance. Inspect the mashup only where its runtime design adds useful information. Do not treat another game's constants as evidence for this game.
3. Record a repeatable original-game scenario using the [playtest template](playtests/TEMPLATE.md): first Adventure stage, starting fish/size/balance, purchases/settings, timed clicks, food eligibility, consumption, any growth prerequisite, currency generation/value/lifetime, and collection. Record movement/animation and available audio observations. Timing and update cadence remain unknown until checked.
4. Write a compact behavior evidence map with stable claim IDs, build/revision references, expected results, observation method, limits, and disagreements between source and retail. Cover update order and RNG limits; do not invent numeric tolerances. Derive tolerances from observations or label a deliberate approximation.
5. Choose and document the smallest architecture and dependencies supporting the recovered slice, license boundaries, exact runnable commands, and meaningful regression cases. Leave missing GUI access or required human evidence explicitly open while advancing independent asset/source work.

Do not request another project intake. Ask for a path only if the recorded installation is no longer available. No gameplay behavior was measured during setup.

## M1 execution acceptance

The product behaviors are specified in [requirements M1](requirements.md#m1-first-real-tank-interaction-loop). Accept them only when all of these proof conditions are met:

- An identified Rust build runs at normal gameplay speed through the complete user input path using the owned assets, without the original executable supplying the runtime.
- A scenario demonstrates animation/movement, click-to-food, eligible consumption, naturally earned currency, click-to-collection, and the resulting balance change. Coin production follows whatever fish size/growth prerequisites the recovered original rules require.
- Instrumentation records ordered actions, simulation time/ticks, entity IDs and states, consumption, currency spawn/collection, and balance changes. Control RNG for tests; report uncertainty in retail RNG comparisons.
- Visual inspection checks sprite frames, transparency/masks, coordinates, and responsive clicks. Audio events used by this slice are compared or specifically listed as missing. A human playtest and judgment of feel are separately identified; pending human checks limit the claim.
- Independent regression cases cover recovered transitions and concrete edge cases, including avoiding duplicate collection or duplicate food consumption. Original expectations and generated implementation output are not conflated.
- Missing/invalid game paths produce a useful failure. The original install remains unchanged and project writes stay separate. Record executable/asset identity before and after the applicable run.
- A completed execution report identifies original and Rust builds, dirty changes, configuration, inputs/timing, actual results, failures, and not tested cases. Update the handoff and claims only to the extent this evidence supports.

## Open questions

Effective original cadence/order, RNG equivalence, exact movement/hunger/growth/coin timing, renderer equivalence, audible sound selection, MO3 music, complete progression/save semantics and distribution/code licensing remain unresolved. The local version string does not establish WinFish compatibility. These are recovery/design tasks, not setup blockers.

M2 and M3 retain their full scope in [requirements](requirements.md#playable-milestones). Advance only after the evidence supports the previous increment; do not redefine the first loop as completion of the runtime.

## M2 next integrated outcome

The [run03 report](playtests/2026-10-08-m1-03.md) supports the first Rust interaction loop. Retail parity checks remain open while independent progression work advances.

Recover third-egg handling, hatch/reward/profile advancement, next-stage initialization and all-fish-dead behavior from exact W1 callers, then confirm/correct against the actual installed game payload. Extend the existing state rather than replacing the working feeding/collection loop. Keep project progression separate from original profiles; retain compatibility with existing project snapshots or explicitly migrate them.

Acceptance: complete the first tank through ordinary feeding/growth/earned money and three affordable eggs at normal speed; display the recovered completion flow, advance once to the correct next starting state, save/reload that progress, and exercise starvation/game-over/restart. No forced victory or debug balance in the execution report. Tests cover independently grounded progression boundaries and save transitions; the identified window run, source/binary evidence, audio gaps and human checks must remain distinct. After this outcome is validated, select the next missing system from [coverage](compatibility.md) automatically.
