# Virtual Tank and store

Current33 integrates the path below; focused compile/lint/regressions passed, while native acceptance remains not tested. [STATUS](../STATUS.md) owns current verification. [BR05/06](binary-recovery.md#br05--virtual-tank-setup-and-store) owns the primary recovery outline; [provenance](provenance.md) owns source and license boundaries.

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

## Next work

Continue Sandbox/Relax, profiles/options/highscores and singing-fish audio, then close the remaining VT actor/diet/pet coverage. Do not repeat full unaffected matrices or refine every fidelity detail before finishing system coverage. Deferred native acceptance must exercise store purchase/cancel/sale, ownership/wallet conservation, feeding/collection, setup and save/reopen/pause on an identified isolated build.
