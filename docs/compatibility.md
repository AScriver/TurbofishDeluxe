# Runtime coverage

The full product scope is [requirements](requirements.md). The installed game binaries are the authority. A Rust execution result and a WinFish-derived expectation are separate forms of evidence; neither implies retail fidelity. Update this checklist as each system is recovered and exercised.

| System | Implementation / Rust evidence | Primary-game evidence and remaining work |
|---|---|---|
| Install discovery/identity | Steam metadata, override, PE/hash/inventory implemented and exercised | Current installed files identified; no inferred gameplay compatibility |
| Images/masks/resource manifest |246 entries decoded, synthetic edge regressions | Actual asset formats observed; binary renderer equivalence pending |
| Bitmap fonts |15 scripts/atlases parsed; current HUD uses owned fonts | Actual Windows-1252 glyphs/metrics observed; visual matching pending |
| Fixed tick/update order/RNG | Controllable28ms Rust ticks; ordered food/fish/Stinky/coin path, regression cases | Embedded game constructor/window timer supports28ms default; live cadence/overrides, cross-list order and retail RNG unresolved |
| Guppy movement/animation | Swim/eat/turn/growth/death projections implemented; corrected heading agrees with sampled motion in run03 | Exact retail movement/turn dynamics pending |
| Feeding/hunger/growth/death | First-level functional rules/tests; live feeding/growth observed | Installed-game thresholds, timing, starvation/tutorial/game-over to confirm |
| Currency | Silver/gold production, expiry/collection/balance, aggregate claimed-value affordability; natural player/Stinky collection observed | PB09–PB12 support ordinary values, bottom thresholds and pet geometry/payment; broader modifiers/order remain pending |
| First-tank purchases/eggs | Three earned eggs/hatch/advance, hold shortcut/rescue and active-board personal-best score observed. Terminal collecting-coin settlement has regression evidence only | Primary score/reward/settlement caller, full hatch presentation and broader profile score behavior pending |
| Project persistence | Atomic format7;formats1–6 migrate. Selected1-5, in-flight Bonus, results and Star session/RNG reload exercised; strict validation | Project bonus recovery is deliberate durability; original-save compatibility/broader profile semantics absent |
| Window/input/pause | Real owned-window clicks, scaling, corrected projection observed | Retail UI layout, focus behavior and transitions pending |
| Sound effects |65 effects decoded; initial event playback implemented | Audible output, exact event selection/rate/volume comparison pending |
| Music | Not implemented | MO3 tracker order/loop semantics to recover from installed runtime |
| Later Adventure stages | Earned1-2 through1-5/hatches exercised; selected early2-1, Potion→Star and natural40credit/reload passed |2-1 completion/Clyde/Starcatcher2-2 next; primary/full later-stage acceptance pending |
| Additional fish | Oscar and Potion→Star guppy/naturalStar coins exercised; Crowned implemented with regression evidence only; Ultra/Gekko/Penta/Grubber/Breeder/special fish absent | PB18–PB19/PB24/PB27 support partial behavior; retail/RNG/audio comparison pending |
| Aliens/combat | Weak/strong Sylvester and Balrog warning/chase/hits/diamond/body/laser/warp implemented; natural fights exercised | PB13–PB17/PB21–PB22 support stats and input/removal. Other enemies/formations, particles and complete dispatch pending |
| Pets | Stinky/Niko/Itchy/Prego/Zorf implemented/exercised; three-pet selection/reload passed | PB10–PB11/PB16/PB20–PB24 support partial rules; remaining pets/modifiers/retail timing pending |
| Upgrades | Food/weapon upgrades and tank2-1 armed Potion implemented; ordinary purchases/transformation observed | Remaining tank-specific purchases, primary prices/gates/full acceptance pending |
| Adventure ending/bonus/rewards | First shell bonus/manual start/collection/timeout/results and one539award across reload observed | PB25–PB26 support clock/flight; combo/values/sort partly secondary; final ending/remaining bonuses/store absent |
| Time Trial | Not implemented | Tank selection, time limits/scoring/pet egg behavior pending |
| Challenge | Not implemented | Escalating waves/prices, victory/scoring/bonus flow pending |
| Sandbox/relax | Not implemented | Availability and rules to verify against installed game |
| Virtual Tank/store | Not implemented | Fish properties, shells, feeding/background/pet/store/simulation/persistence pending |
| Menus/options/profile/highscores | Minimal pause/save plus natural GameOver/selector/help/reentry and separate later-stage results exercised | Retail screen artwork, restart confirmation, settings/profiles/unlocks/highscores pending |
| Full-runtime acceptance | **Not achieved** | Per-system primary evidence, normal playable paths, visual/audio comparisons and human playtests remain required |

Current reports cover [first loop](playtests/2026-10-08-m1-03.md), [progression](playtests/2026-10-08-m2-01.md), [Stinky/score](playtests/2026-10-08-stinky-score-01.md), [1-2](playtests/2026-10-08-adventure-1-2-01.md), [1-3](playtests/2026-10-08-adventure-1-3-01.md), [1-4](playtests/2026-10-08-adventure-1-4-01.md) and [1-5/bonus/early2-1](playtests/2026-10-08-adventure-1-5-bonus-2-1-01.md), retaining failures. Rules are in behavior/stage contracts; [Clyde/Starcatcher2-2](adventure-2-2.md) is next. Blockers/next action are in [STATUS](../STATUS.md); internal milestones do not end the goal.
