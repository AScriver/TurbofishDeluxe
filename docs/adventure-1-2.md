# Adventure1-2 recovery and next implementation

This bounded ledger supplements the [behavior contract](behavior-contract.md). W1 is pinned in [provenance](provenance.md); paths below are relative to its `source/WinFish/`. **All A12 claims are secondary-source-derived** unless a primary correction is explicitly added. The installed PB05 payload remains authoritative. No original-game execution has occurred.

## Source contract

| ID | Independent expectation | W1 evidence |
|---|---|---|
| A12-01 | Fresh1-2: cash200, pellet5, quality0/quantity1, weapon2; weak Sylvester timer1750, egg500, no Oscar/weapon slot | `Board.cpp:2700–2711,3843–3876` |
| A12-02 | Medium growth unlocks guppy buying; Large growth unlocks quality. First quality purchase unlocks quantity and egg. Finished-profile suppresses inspected tutorial text, not these gates | `Fish.cpp:1658–1708`; `Board.cpp:2683,4121–4135` |
| A12-03 | Quality0→1→2 costs200 each; quantity1→9 costs300 each; maxed upgrades stop buying. Each pellet snapshots its quality | `Board.cpp:3858–3859,4093–4149,4620–4647,4910`; `Food.cpp:248` |
| A12-04 | Ordinary food0/1/2 adds500/700/1100 hunger, caps800/1000/1400, and gives1/2/3 growth units. Beginner food0 adds another200. Growth consumes4–6 units then resets credit | `Fish.cpp:723–734,779–805,1524–1532,1548–1555` |
| A12-05 | Held feeding needs elapsed press time>200ms and old board count modulo`16−quantity`=0. Logical bounds remain x31–586/y61–399; simultaneous ordinary pellets respect quantity | `Board.cpp:803–815,4881–4910` |
| A12-06 | When no live alien/bilaterus/missile, decrement invasion counter. At276 first danger modal, later battle-tip modal, tracked by flags3/4.275 shows warning/sound and selects coordinates.1 starts battle music;0 spawns and resets3000. Live alien freezes countdown | `Board.cpp:916–925,947–1013,1094–1106`; modal pause `WinFishApp.cpp:714–718` |
| A12-07 | Warning selects first x20–469/y105–299 and consumes a second pair even for one alien. Registration immediately suppresses fish hunger/coins and retracts Stinky, including invisible emergence | `Board.cpp:1002–1005,3133–3140,5430–5439`; `Alien.cpp:1361–1369`; `Fish.cpp:430–431,936–973`; `OtherTypePet.cpp:602–617,637–644` |
| A12-08 | Weak Sylvester160×160,50HP, divisor2, VX±3/VY0, chase/eat delay100, spawn15, movement state0–9. First six updates decrement spawn and return; later emergence≤10 adds horizontal±3 | `Alien.cpp:22–63,137–175` |
| A12-09 | Nearest prey uses integer widget centers and strict-smaller squared distance. Positive cannot-be-eaten delay excludes prey; Stinky is not prey. Chase accelerates axes0.1 with guards±1.8 | `Alien.cpp:793–913`; source set pointer-order ties cannot establish retail order |
| A12-10 | With old chase delay<1, strict abs(dx)<45/abs(dy)<65 contact removes first eligible overlap, not necessarily selected nearest. Weak eating does not reset delay; max one each update | `Alien.cpp:912–921,969–1014,457–465` |
| A12-11 | Clamp doubles before integration: X−10..490/Y85..290, velocity/divisor2. Decrement hit/chase counters afterwards. Copy doubles to integer widget position **before** animation, which can change doubles again when abs(VX)>1.6 | `Alien.cpp:258–302,403–404,1142–1266`; animation counters therefore belong to simulation |
| A12-12 | Strict interior160×160 shot rectangle uses doubles; hit timer must0. Weapon2 gives6 damage, nonlethal immunity10; nine accepted hits kill50HP. Early-return spawn updates do not decrement immunity; hidden spawn is not explicitly immune | `Alien.cpp:526–549,652–658`; `Board.cpp:2708` |
| A12-13 | Local shot60/100 bands push away: corners set both axes±5; cardinal bands set one±6, preserve other; center leaves velocity. Ordered comparisons decide equal edges. Weapon2 has no held autofire | `Alien.cpp:612–649`; `Board.cpp:818–824` |
| A12-14 | Board invasion clicks y>40 become laser attempts, including misses, with no food purchase. Last live removal resets held input, restores music/fish input and sets food delay36; a click>50px from preceding laser point clears delay | `Board.cpp:1490–1493,1532–1565,1587–1595,3107–3117` |
| A12-15 | Lethal shot creates one diamond at prior integer alien position+(25,25), removes live membership/restores board, then creates death effects/body. Diamond200 follows ordinary credit; dead body does not count as invasion | `Alien.cpp:652–655,1271–1284,1332–1358`; `Coin.cpp:725–729`. Ordinary bottom lifetime follows PB09 |
| A12-16 | Third egg uses the established completion transaction, unlocks Niko and advances1-3. Fresh1-3 has200, two small guppies with two food units, Stinky+Niko, quality0/quantity1, strong Sylvester timer3000, egg2000 | `Board.cpp:2177–2214,2700–2711,3293–3360,3877–3880`; `GameObject.h:69–73`; `HatchScreen.cpp:630–638`; `WinFishApp.cpp:1869–1901` |

## State and verification consequences

Board owns upgrade levels/gates, warning countdown and coordinates, modal flags, food delay and weapon. Pellet owns its quality. Alien owns double motion, prior integer widget coordinates, HP, spawn/chase/hit/state/animation counters and prior direction. Preserve these atomically with fish, live pets, coins, progress and RNG. Derive targets each update; list membership determines invasion, not rendered warp/body pixels. Normal source sorting is food→guppies→alien→Stinky→shots→coins (`Board.cpp:1951–2108`).

Owned metadata/header observations: sylv1600×320/10×2 (160px cells), food400×200/10×5 (40px), lasers800×480/10×6 (80px), warphole1020×220/17×1 (60×220), warpglow1700×220/17×1 (100×220). Companion masks exist. AWOOGA/ROAR/HIT/EXPLOSION1 are OGG, zap/explode are AU. These were not playback-tested. Warp additive draw/frame boundaries and alien hit-flash blending need primary/visual checks.

Next acceptance: buy quality using earned coins after Large growth; quantity/egg unlock correctly; endure first warning/warp, shoot weak Sylvester through nine accepted hits, collect its natural diamond, finish three500 eggs, hatch Niko and save/reload1-3. No debug money, forced spawns or completion. Tests need independent boundaries for nutrition/gates, old-counter warning/modal timing, spawn updates1–6 versus7, hit immunity, contact45/65, single reward, player/pet credit, and reload during warning/warp/retract.

Primary priorities are animation-driven motion/widget lag, hidden-spawn shooting/cooldown, concrete pointer/list ordering, shot/third-egg action order, and price/gate initialization. W1 warp draw can calculate an out-of-sheet endpoint (`Warp.cpp:42–44`); do not invent a clamp and call it recovered behavior. Original cadence/RNG, original-game/audio comparison and human feel remain open. Implemented status is authoritative in [STATUS](../STATUS.md).
