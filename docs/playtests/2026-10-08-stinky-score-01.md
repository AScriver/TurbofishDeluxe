# Stinky collection, first-stage score and format3 persistence

**Result:** A normal-speed Rust run from an untouched, genuinely earned1-2 save showed Stinky collecting a naturally produced silver coin without a player click. A separate fresh first tank earned three eggs and recorded126 active-board seconds. Both saves reloaded successfully. These are agent execution checks; original-game, audible and human comparisons remain not tested.

## Build and resource identities

Executed2026-10-09 approximately00:56–01:01 UTC (2026-10-08 America/Phoenix). Original build: Steam App3320/build250752, main SHA-256 `f54c2c6ee54b00ae6da7f4bde15feb90d0a867aba6b4151b0277c1f81da3ff66`. Environment and owned installation identity are authoritative in [STATUS](../../STATUS.md). The original executable was not run.

- Rust frozen debug executable SHA-256: `353ab1f58b2b0de781499e6607a8929c198fecad75a75adabb38237d403976a4`.
- Build HEAD: `162c501cdee2d3d342fefa57b34d3afadfc3dbe1`, with dirty `src/adventure.rs`, `src/app.rs`, `src/cli.rs`, `src/sim.rs` and `tests/persistence.rs`. Commit alone does not identify this build.
- Relevant source SHA-256: adventure `22e4d79b22684d790be51331a334c4927ebe3212187f9f92b535aa3e9827344b`; app `6ceb9fe9c83b837f13cecea6b691749d8064c618b01de4a8adc37ea980a8b10d`; cli `fefbeded9a6a9ea5d2338b93a9c0aa8f469ec4e35bcd65045e14fe2cf8a665ac`; sim `764bde27b54632d4a4c4a9d3e9407b33a22e108dac3be3031de73217c4cf493b`. Complete relevant-file receipt: `.scratch/runtime/tester/build-identity-m2-stinky-01.local.json`.
- Rust1.95.0/MSVC, locked debug/audio build; format/strict Clippy/50 tests/native build passed. Hardware/driver identity and audible output unknown.
- Owned assets, logical640×480/window960×720, normal28ms wall-clock updates. Targeted messages went only to the owned Rust window. No forced coins, growth, money, victory or accelerated clock.
- Separate copied project saves and private evidence; no original saves, registry, installation writes or global input.

## Repeatable scenarios and observations

| Scenario | Expected contract | Actual result | Verdict / limit |
|---|---|---|---|
| Resume the earned1-2 save | Initialize newly supported live Stinky explicitly while preserving stage, cash and roster | Source/copy SHA-256 `c5ac7dd3f91952e06859c4401990aecd248bbe3512bb8dd573fabd0213762da4`; session5694/board6,200 cash/two live Small guppies/Stinky roster. Startup migrated format2→3, origin `LegacyV2Resume`, Stinky X290 | Passed actual migration; old live placement was unknown, not recovered |
| Feed and let Stinky collect | A natural eligible coin is credited once and removed before coin motion; no player coin click | Eleven5-cost food drops; fish1 grew Medium at board354. Silver14 dropped at board569/session6257 and was caught at582/6270. Exactly one `PetCollectedCoin` and one corresponding `CoinCredited`, amount15/final cash160; coin absent thereafter | Passed normal input path; primary contact/nearest rules PB10–PB11, remaining movement rules secondary |
| Inspect motion and close | Upright world, appropriate direction/turn poses, final/save agreement and clean exit | Primary inspected actual frames006258 and006273: head-right toward the falling coin, then head-left/turn after collection. HUD145→160. Run16.950s, final session6273/board585, both fish alive, save/final Stinky and RNG agree, exit0 | Passed sampled visual check; full animation/retail feel unmeasured |
| Reload during pet movement | Retain all serialized motion/RNG state; no reinitialization or second credit | Before-save SHA-256 `0c53ced690e015998d934cb43f64934950e3d9e256eaa7e497d2beba6b2c866e`. Start session6273, Stinky fields exact, RNG `12065585567899109333` and next seed preserved.145 subsequent ticks moved X260.5→260.83333333333303 and VX−2.7→0.5; origin unchanged; no coin14 recredit | Passed typed state equivalence; see numeric-text limitation below |
| Fresh first-stage score | Whole seconds use active board ticks, record once with third earned egg, persist personal best | Seed42, both original fish survived/grown Large, no rescue. Exactly three paid eggs. `StageResultRecorded` at board/session4508, `floor(4508*28/1000)=126`, best126, final balance15. Final session4520/hatch12 saved format3 and best126 | Passed actual ordinary completion and clock calculation; score/settlement caller remains secondary |
| Score reload | Preserve recorded best and hatch state | Start4520 matches saved session, best126 retained at start/final, normal close exit0 | Passed |

The fresh completion had no collecting coins left at retirement: coin97 had credited at4506. Terminal pending-coin settlement therefore **was not exercised in the window**. Independent regression cases cover settlement, aggregate affordability/temporary negative raw cash, duplicate-credit prevention, pause/rescue clock boundaries and malformed/missing modern save fields.

## Comparison limits and retained failures

The first reload comparison found six whole-board JSON numeric-text differences. Telemetry's untyped JSON widens f32 positions to exact f64 numbers, while typed saves serialize shorter f32 decimals. A follow-up bit comparison showed every affected f32 identical; Stinky's f64 fields were exactly equal. Retain the original text-comparison failure and the narrower successful check; do not claim byte-identical JSON.

The rescue-clock correction in this build advances the board clock on rescue detection before pausing and performs the normal update on an explicit resumed step. This supersedes only the affected clock claims in the [historical M2 report](2026-10-08-m2-01.md); its prior builds/results remain recorded.

Before/after installation inventory digest matched `6e03aca251d24dc68b0e72e88e06bd50b40775ebe0a09f182e4c2d35ff32f0ca` in all completed runs. All four owned PIDs exited. No services/ports or original-game resources were changed. Actual audio playback, original Stinky/score behavior, human judgment, aliases/other pets, aliens/upgrades and later completion remain **not tested**.

Private evidence directories: `.scratch/evidence/runtime/m2-stinky-01`, `m2-stinky-01-reload`, `m2-score-01`, `m2-score-01-reload`. Targeted scripts/check receipts and numeric comparison are under `.scratch/runtime/tester/`; save copies under `.scratch/playtests/`. Raw proprietary pixels stay ignored.

Continue with [Adventure1-2](../adventure-1-2.md), retaining the full active goal in [STATUS](../../STATUS.md).
