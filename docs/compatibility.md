# Runtime coverage

The full product scope is [requirements](requirements.md). The installed game binaries are the authority. A Rust execution result and a WinFish-derived expectation are separate forms of evidence; neither implies retail fidelity. Update this checklist as each system is recovered and exercised.

| System | Implementation / Rust evidence | Primary-game evidence and remaining work |
|---|---|---|
| Install discovery/identity | Steam metadata, override, PE/hash/inventory implemented and exercised | Current installed files identified; no inferred gameplay compatibility |
| Images/masks/resource manifest |246 entries decoded, synthetic edge regressions | Actual asset formats observed; binary renderer equivalence pending |
| Bitmap fonts |15 scripts/atlases parsed; current HUD uses owned fonts | Actual Windows-1252 glyphs/metrics observed; visual matching pending |
| Fixed tick/update order/RNG | Controllable28ms Rust ticks; ordered food/fish/coin path, regression cases | Embedded game constructor/window timer supports28ms default; live cadence/overrides and retail RNG unresolved |
| Guppy movement/animation | Swim/eat/turn/growth/death projections implemented; corrected heading agrees with sampled motion in run03 | Exact retail movement/turn dynamics pending |
| Feeding/hunger/growth/death | First-level functional rules/tests; live feeding/growth observed | Installed-game thresholds, timing, starvation/tutorial/game-over to confirm |
| Currency | Silver/gold production, expiry/collection/balance; normal-run silver loop observed | Retail values/timers/geometry/order to confirm |
| First-tank purchases/eggs | Ordinary three earned eggs/hatch/advance observed; held hatch and next-update free rescue corrected/tested | Primary numeric/reward comparison, elapsed-score/settlement and full hatch presentation pending |
| Project persistence | Atomic format2 session with format1 migration; save on hatch/advance/rescue/normal close. Close/reload and accepted-click boundary exercised | Original-save compatibility and broader profile/highscore semantics not implemented |
| Window/input/pause | Real owned-window clicks, scaling, corrected projection observed | Retail UI layout, focus behavior and transitions pending |
| Sound effects |65 effects decoded; initial event playback implemented | Audible output, exact event selection/rate/volume comparison pending |
| Music | Not implemented | MO3 tracker order/loop semantics to recover from installed runtime |
| Later Adventure stages | Secondary stage/price/enemy tables recovered; not implemented | Primary verification and full stage acceptance pending |
| Additional fish | Oscar, Ultra, Gekko, Penta, Grubber, Breeder and special fish not implemented | Individual behaviors/animation/audio/progression pending |
| Aliens/combat | Not implemented | Sylvester variants, Balrog, Gus, Destructor, Ulysees, Psychosquid, Bilaterus, Cyrax and formations pending |
| Pets |24 secondary-source names identified; not implemented | Unlock/select/individual abilities including transformations pending |
| Upgrades | Not implemented | Food quality/capacity, weapons and tank-specific purchases pending |
| Adventure ending/bonus/rewards | Not implemented | Normal completion, unlocks, shells and post-completion behavior pending |
| Time Trial | Not implemented | Tank selection, time limits/scoring/pet egg behavior pending |
| Challenge | Not implemented | Escalating waves/prices, victory/scoring/bonus flow pending |
| Sandbox/relax | Not implemented | Availability and rules to verify against installed game |
| Virtual Tank/store | Not implemented | Fish properties, shells, feeding/background/pet/store/simulation/persistence pending |
| Menus/options/profile/highscores | Minimal first-tank pause/save only | Original screen transitions, settings/profiles/unlocks/highscores pending |
| Full-runtime acceptance | **Not achieved** | Per-system primary evidence, normal playable paths, visual/audio comparisons and human playtests remain required |

Current execution evidence is in [run03](playtests/2026-10-08-m1-03.md); [runs01–02](playtests/2026-10-08-m1-01.md) retain failures. Recovered functional hypotheses are in [behavior contract](behavior-contract.md), and current blockers/next action are in [STATUS](../STATUS.md). Internal milestones do not end the active goal.
