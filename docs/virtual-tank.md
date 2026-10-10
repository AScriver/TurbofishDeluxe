# Virtual Tank and store

Current33 integrates store/setup, current34 adds special diets/basic pet effects, and current35 joins combat, remaining pet interactions and audio/traits. Focused compile/lint/regressions passed; native acceptance remains not tested. Gameplay sprites are next. [STATUS](../STATUS.md) owns current verification. [BR05/06](binary-recovery.md#br05--virtual-tank-setup-and-store) owns the primary recovery outline; [provenance](provenance.md) owns source and license boundaries.

## Ownership and integration

Profile remains the sole shell wallet. `VirtualTankState` owns durable fish identities, traits, inventory, stock, backdrop ownership, settings and selected pets. A separate session `virtual_board` owns transient positions, food and coins keyed to those fish IDs. Visiting Virtual Tank/store preserves the Adventure Board and earned history; it does not replace Adventure's resumable scene. Current33 requires its new serialized keys and rejects prior-envelope relabeling, without new defaults or migration/backfill.

The integrated path covers selector entry, eight persisted daily store offers, accepted/cancelled checkout with name, fish/backdrop sales, show/hide and rename, backdrop selection, pet/settings setup, active feeding/growth/production, collected-shell credit and reopening. Model transactions validate before changing wallet/ownership; this atomicity is a deliberate project policy where retail allocation rollback is unproved.

Cursor5's results offer costs45,000 shells, advances to6 and raises Profile capacity to at least7. Ordinary Adventure/TT choices remain capped4; Virtual Tank uses the separate maximum7. The subsequent Sandbox gate remains cursor>5 and raises Profile capacity to11, with the same ordinary/VT consumers retaining their caps.

## Evidence and deliberate qualifications

Installed PB05 is authoritative, SHA`312021E970ABD8AFE417D24E99FF75AC8379EF2A93BC1E04D52D4F3D7F070BB1`, base00400000. Main anchors:0053a890 capacity/free identity;0052bf90 store price;0054b900 confirmation;0052bd70 debit/attach;0054eec0 name;0052a090 sale;00519470 backdrop last-owned guard/refund;00501200 wallet clamp;004f1ba0 virtual identity/feeding;004f5050→004d54e0→0053c1e0 collection credit. Export success alone is not proof of runtime behavior.

Pinned W1`f919b3c241cfd611c547f1653fb3514f500f761b` supplies secondary behavior from `source/WinFish/{StoreScreen,StoreButtonWidget,SimFishScreen,SimSetupScreen,VirtualDialog,ProfileMgr,GameObject,Board,Fish,Coin,Breeder,Oscar,Ultra,Gekko,Penta,BiFish,BallFish,SylvesterFish}.{cpp,h}`. Daily traits/prices, constructor vitals, food/production and calendar metadata are source-informed original Rust; retail RNG sequence, full actor/visual parity and exact PB05 coin values remain unverified. Named Santa state exists, but inspected store stock does not construct it. Breeder offspring identity/promotion has an unresolved W1 inconsistency retained as a detail follow-up.

W1 Board load after more than120 seconds prunes ephemeral actors and refreshes calendar metadata. No elapsed hunger/growth/coin catch-up loop was found; the implementation must not fabricate one. Same-day sale grace and fish-size sale modifiers follow source predicates, rather than applying ordinary guppy-size multipliers to every bought fish. Native input/render/audio, retail comparisons and human judgment remain pending.

## Current33 implementation checkpoint

E101 passed formatting, strict workspace lint, two model integrity cases, five affected music cases, one current33 persistence case and locked build/staging. The identified executable and runtime files are frozen privately; [MODLOG](../MODLOG.md#2026-10-10--verify-virtual-tank-and-preserve-current33) owns receipts and the earlier borrow/lint failures. This establishes focused code checks, not playable or complete VT behavior.

Current movement, placement and some coin presentation are project projections. General non-Guppy feeding and broader pet effects remain incomplete; Cookie food, special-fish diets, primary coin values, daily RNG/local-calendar parity and backdrop mapping need further integration or comparison. These gaps stay explicit while the other whole systems are implemented first.

## Current34 actor checkpoint

The physical Board now lives in `src/virtual_board.rs`, with required typed food, ephemeral prey and selected pet poses. Source-informed diets/meal production cover the store fish families, including Grubber larva, Cookie food and Penta's third-feed coin rule. Zorf/Prego/Vert/Shrapnel/Nostradamus/Meryl have basic production effects; Stinky/Clyde collect by contact into the one Profile wallet. A pose alone does not establish another pet's function. The Penta regression passed within E102; broader integration passed compile/lint/build. Reopening retains the durable ledger and prunes ephemera after the existing source timeout rather than inventing offline catch-up.

Current34 requires the new Board fields without migration/backfill. Its input/render path projects typed food/prey/pets. Species motion, exact growth/vitals, RNG/local-calendar parity and primary coin values remain qualified; remaining pet functions and Alien Attractor combat are open.

## Next combat and pet increment

Keep `VirtualBoardState` as the sole live combat, object and pet pose authority. Stable food/prey IDs share its object allocator with coins; purchased fish retain their separate durable ledger IDs and are protected from alien/pet predation. A new combat state reuses ordinary threat mechanics, owns one encounter/countdown and distinguishes no-reward timeout removal from paid defeat. Hidden attractor freezes the countdown; no offline simulation is invented. Existing pet poses gain Niko/Brinkley/Walter/Presto state and functional source interactions without embedding duplicate actor coordinates/timers. Presto keeps its selected roster identity while changing one active form.

Pinned W1 `Board.cpp`, `Alien.cpp`, `Bilaterus.cpp`, `Missle.cpp`, `FishTypePet.cpp` and `OtherTypePet.cpp` supply this bulk contract. Private bounded briefs`.scratch/runtime/a47-virtual-combat/contract-01.md` and`.scratch/runtime/a48-virtual-pets/contract-01.md` retain exact line follow-ups. Source-exact RNG, overlap/input ordering and contact timing remain deferred. Run only changed-behavior/data-integrity checks after convergence. Later native acceptance must exercise store transactions, feeding/pets/combat/collection, setup and save/reopen/pause on an identified isolated build; it remains deferred now.

## Current35 combat and pet checkpoint

`VirtualCombatState` owns one seven-choice encounter, warning/attractor anchors, projectiles and next encounter, using the Board's stable allocator and existing session RNG. Hidden attractor freezes its countdown. Active threats continue to update and expire after the source age bound without a defeat diamond. Purchased fish never enter combat prey views; direct feeding, Gash and Amp affect transient prey only. Wadsworth wards those guppies, Gumbo/Rhubarb steer them, Itchy/Rufus/Gash contact threats and Stanley owns launched projectiles. Walter's glove affects physical recoil without invented alien damage. Presto retains one selected identity, changes its active form and pauses the Board through its modal dialog.

Niko has owner-keyed clickable pearls; Stinky/Clyde cannot claim them. Brinkley meal tiers, Zorf interaction, Amp charging/discharge, Nimbus coin steering and Seymour descent integrate with the existing production pass. Source-disabled VT Nimbus food targeting and Blip targeting remain disabled. Missile-killed transient fish create Board-owned corpses and Angie may revive them into fresh transient IDs; direct alien consumption creates no corpse. Source chain: W1 `Missle.cpp:193–205` → `Fish.cpp:1476–1484` → `Board.cpp:444`, `FishTypePet.cpp:676–679,1164–1177,1353–1371`, `DeadFish.cpp:16–48,95–126`. Corpses and revivals never create a purchased fish.

Current35 requires combat, object IDs, form/specialty clocks and corpse state without migration/backfill. E103 passed two combat cases and one active-state ownership/save/reopen/setup-freeze case plus lint/build. [MODLOG](../MODLOG.md#2026-10-10--integrate-virtual-combat-and-pets-and-preserve-current35) owns identities and retained fixture failures. This is broad functional integration, not a per-pet acceptance matrix. Exact target geometry, cross-list/contact timing, weapon/input overlap priority, motion, RNG, death rendering, retail/audio/human parity remain unverified.

## Current35 audio and trait checkpoint

E104 joins [VT cues and typed effects](audio.md#virtual-tank-audio-join), upgraded manual food, Speedy displacement, Backward sprite facing, Voracious vocalization and the Bubbulator device/bubbles. Profile and Board remain the only wallet/live-state authorities; no serialized fields or migration/backfill were added. The affected current35 case checks upgraded food, one typed collection credit, repeated-click rejection, active-state save/reopen/setup freeze and existing required-field/object-ID guards. One new cue case and that affected case passed with lint/build; [MODLOG](../MODLOG.md#2026-10-10--join-virtual-audio-and-trait-effects) owns identities and the style-only first failure.

Pinned W1 sources: `Fish.cpp:521–530,550–577` (Speedy and Backward), `GameObject.cpp:650–657` (Voracious animation/voice), `StoreScreen.cpp:396` and `Board.cpp:4910–4922` (food upgrade/drop), `BubbleMgr.cpp:229–290` and `Board.cpp:660–678,1658–1663,4769–4798` (Bubbulator). These are secondary-source-derived rules. Speedy divisors are scaled into current pose velocities. Voracious uses a projected150-tick fish-ID-staggered cadence near eligible food; no feeding/growth multiplier was invented. Existing pellets retain their drop-time quality.

The Bubbulator uses original device/bubble sheets and source-shaped bursts. Its500-tick cycle, particle positions/rise and current backdrop placement are project projections rather than recovered retail clocks/RNG. Exact visual and audible timing remains unverified. Source-disabled VT Blip targeting remains disabled; the typed sonar effect is available without an invented idle producer. Next render change uses species/size/hunger sheets and typed food/prey/coins from existing Board state. Exact eat/turn/palette/order and native/human comparison remain deferred.
