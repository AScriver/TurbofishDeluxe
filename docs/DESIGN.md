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
| Validation | Independently derived rule tests plus original/Rust scenarios and runtime evidence | Rust execution reports exist; original comparison baseline pending |

The first implementation uses one Rust 2024 crate, macroquad for the Windows window/2D/input/sound path, image for direct owned-image decoding and separate explicit simulation state. Dependencies are recorded in [provenance](provenance.md#rust-dependencies). The owned assets use sprite sheets and companion masks; no conversion cache is needed for the first slice. A 28ms fixed-tick accumulator separates simulation from rendering; primary PB06 supports the constructor's timer configuration, with actual cadence still unmeasured. Cap stalled-frame catch-up at the source framework's 200ms bound and record stalls. Do not silently accelerate simulation for an acceptance run.

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

This historical M2 design recovered third-egg handling, hatch/reward/profile advancement, next-stage initialization and all-fish-dead behavior from exact W1 callers, then checked against the actual installed payload. It extended existing state and kept project progression separate from original profiles. The former snapshot-migration plan below is historical; current save scope is authoritative in [requirements](requirements.md#goal-and-constraints).

Acceptance: complete the first tank through ordinary feeding/growth/earned money and three affordable eggs at normal speed; display the recovered completion flow, advance once to the correct next starting state, save/reload that progress, and exercise starvation/game-over/restart. No forced victory or debug balance in the execution report. Tests cover independently grounded progression boundaries and save transitions; the identified window run, source/binary evidence, audio gaps and human checks must remain distinct. After this outcome is validated, select the next missing system from [coverage](compatibility.md) automatically.

The identified [M2 report](playtests/2026-10-08-m2-01.md) verifies earned victory, hatch/next-stage persistence and first-level rescue in Rust. The later [Stinky/score report](playtests/2026-10-08-stinky-score-01.md) adds elapsed-score and pet execution evidence; full presentation and primary score/settlement confirmation remain unfinished.

## Next integrated outcome: Stinky and victory bookkeeping

Preserve the existing board and session. Implement ordinary Stinky as serializable live pet state, acting between fish and coins, using the independently reviewed SK contract. Keep unlocked profile pets distinct from live motion state. Existing format2 saves lack that motion state; initialize the newly supported entity explicitly during migration and record that the historical position was unknown. Resolve the targeting-expression discrepancy against PB05 before codifying it.

Add first-stage elapsed-score and collecting-coin settlement only after tracing their initialization, pause/rescue boundaries and recording caller. Store the result atomically with progression; do not infer an old missing score from session ticks that include hatch time.

Acceptance: normal-speed play from an actually earned1-2 save produces a coin that visible Stinky catches without a player click, credits it once and removes it before coin motion/expiry. Exercise player competition and reload during movement; paired frames must show appropriate direction/turn poses. Independently grounded regressions cover ordering, overlap, timers, coin lifetime and migration. Score/settlement tests cover the actual recovered clock and victory boundary. Record absent aliens/upgrades and retail/audio/human comparisons explicitly, then continue those systems automatically.

This checkpoint passed50 regression tests and actual Stinky/score/reload runs identified in the [report](playtests/2026-10-08-stinky-score-01.md). Primary PB10 corrected the source's suspicious nearest expression; PB09/PB11/PB12 support lifetime/contact/aggregate-payment rules. Terminal settlement remains regression-tested without a window example. Existing format2 state migrates explicitly to format3; missing old scores remain unknown.

The [Adventure1-2](adventure-1-2.md) checkpoint passes85 tests and [actual earned play/reload/Game Over](playtests/2026-10-08-adventure-1-2-01.md), committed582f510. [Adventure1-3](adventure-1-3.md) adds Oscar hunting/production/survival, strong Sylvester, weapon gates and Itchy reward; its earned run and rebuilt hatch/next-stage reload passed, committedad838f2. Historical build identities remain separate.

The [Adventure1-4](adventure-1-4.md) increment passed127 tests and [earned1-4/selection/Prego/reload](playtests/2026-10-08-adventure-1-4-01.md). Alien health and registered membership remain distinct during Itchy's lethal contact; removal is an exactly-once alien/Board transaction. Unlocked profile pets remain separate from active selection. PB20–PB22 adjudicates the suspicious spawn/target geometry. At that checkpoint early1-5 was exercised and completion gated; current execution status is in STATUS.

The [1-5/bonus/early2-1 outcome](adventure-1-5-bonus-2-1.md) passed [identified execution](playtests/2026-10-08-adventure-1-5-bonus-2-1-01.md). Bonus state is carried by the session phase with no ordinary fish Board. It owns absolute Board clock, shells, click-order/combo and uncredited total; profile owns capped shell balance. Results entry consumes active bonus/commits stage and balance once; count-up never awards again. Preserve manual versus automatic start and Board-before-shell completion timing. Strict format7 migrates old progress with zero shell balance and rejects legacy claims to new phases.

Zorf extends FishTypePet; Board owns its free ordinary food identity/direction/cap interaction. Potion arm is Board state until accepted manual placement, then the food/fish state owns consumption and transformation. Implement source-backed Star conversion and lethal-meal continuation, tank-aware2-1 purchases and selected five-pet availability. Accept only after a normal-speed earned1-5→Zorf→bonus→2-1 run, one results credit/reload and visible Zorf/Star behavior; keep retail/audio/human limits open while continuing the full goal.

After that acceptance, integrate [Clyde/Starcatcher2-2](adventure-2-2.md) in the existing crate. Use a pure serializable Starcatcher actor and a separate Clyde actor; Board retains ordered entity lists, identities and all consume/credit/remove transactions. This extends the existing Oscar/FishTypePet pattern without a general species abstraction. Update Starcatchers before food/guppies according to the labeled W1 order, and apply live-Starcatcher star exclusion to both Stinky and Clyde. Preserve the special diamond's rise phase and actor corpse state. Primary ordering and retail RNG parity remain open.

Bump the project-save schema to8 for these new live entities and require their fields, including explicit optional actor fields and the special diamond's motion phase. Follow the current-save scope in [requirements](requirements.md): no new migration or historical-state reconstruction. Existing older codec work and reports are retained as history, without a compatibility promise. An explicitly prepared private fixture can carry earned prior gameplay forward when a strict diff proves only known-empty new fields and the format number changed. Enable2-1 completion only when Clyde hatch/selection/fresh2-2 work together. Vert may be unlocked by earned2-2 completion while2-3 entry remains gated until its actual Gus/Vert implementation is ready. That gate is a temporary implementation boundary, not reduced product scope.

The178-test gate and [earned2-1/Clyde/2-2/Vert report](playtests/2026-10-08-adventure-2-1-clyde-2-2-vert-01.md) accept that bounded path. Next, [Gus/Vert2-3](adventure-2-3.md) extends the existing Alien kind with finite double health and ordered food/prey contact; no separate alien framework. Quarter-point Itchy damage must survive pending death and reload. Board owns free initial Gus feeding, charged held-repeat/cap behavior, immediate removals and exactly-once defeat; Gus leaves no ordinary alien corpse. Vert extends FishTypePet motion with a persisted216-update Gold clock, suppressed by registered invasion membership. Reuse existing render projections with owned Gus/Vert sheets. Actor/presentation source owners work in separate files; primary integration owns session/current-save changes and the gate. Keep2-4 entry gated until Destructor/Rufus follows; after earned2-3 acceptance continue automatically.
