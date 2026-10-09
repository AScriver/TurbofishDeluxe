# Adventure1-3 contract and next increment

W1 is pinned in [provenance](provenance.md); paths below are relative to `source/WinFish/`. These A13 claims are **secondary-source-derived** except the actor constants/table association confirmed in [PB17](behavior-contract.md#primary-binary-findings). The installed PB05 payload remains authoritative. No original gameplay has been observed. Current1-3 has live Niko but lacks Oscar, the strong wave, weapon buying and completion.

## Evidence ledger

| ID | Independent expectation | W1 evidence / primary limit |
|---|---|---|
| A13-01 | Fresh1-3: cash200, quality0/quantity1, weapon2, strong wave3000. Prices guppy100/quality200/quantity300/Oscar1000/weapon1000/egg2000; slots0/1/2/3/5/6 | `Board.cpp:2699–2711,3843–3879`; price/gate fields not yet primary-confirmed |
| A13-02 | Medium growth opens guppy; Large opens quality, quantity and Oscar. First successful Oscar purchase opens weapon/egg. Weapon increments through12 at1000 each | `Fish.cpp:1658–1678,1711–1715`, `Board.cpp:4129–4132,4152–4165,4222–4227` |
| A13-03 | Bought Oscar: x20–539, entrance y40/VY23–27/timer45–54;80×80, lower movement bound360, initial hunger600–799 | `Board.cpp:5278–5290`, `Oscar.cpp:38–44`, `Fish.cpp:1497–1498`; exact constructor RNG ordering remains unconfirmed |
| A13-04 | Oscar stays fixed type/size5; meals do not grow it. Hunger gains900 capped1300; shared pre-hook can raise hunger below300 to300 under its animation condition | `Oscar.cpp:38–44,58–87`, `GameObject.h:33–42`, `GameObject.cpp:463–470` |
| A13-05 | Hunger<500 hunts only small guppies, never pellets/medium/large/Oscars. Aliens freeze hunger decrement and coin production, while hunting continues. Hunger<1 kills it | `Oscar.cpp:154–169,376–405,430–454`, `GameObject.cpp:318–350`, `Fish.cpp:430–431` |
| A13-06 | Nearest truncates each double subtraction toward zero before integer squared distance; strict smaller retains first tie. Consumption separately scans guppy order. Steering centers use int(X+40),int(Y+50), with special movement timer>2; contact runs without that steering delay | `Oscar.cpp:248–253,371–405,430–448` |
| A13-07 | Strict contact uses double Oscar center(X+40,Y+45) inside prey integer(x+10..70,y+22..58). One prey removed without corpse. Existing eating animation permits consumption; broad anticipation starts20, actual consume starts8 only if current counter0, then inherited animation decrements that tick | `Oscar.cpp:438–454`, `Fish.cpp:1372–1373` |
| A13-08 | Production interval150–349 is drawn once at creation; eligible updates increment and reset0, emitting diamond200 at widget+(5,10). Ordinary hunger does not suppress it. Starvation schedules removal but the rest of that Fish update, including due production, continues | `Fish.cpp:108–116,430–436,519–533,1469–1484,1545–1547`, `Oscar.cpp:143–169,460–477`; deferred deletion is material |
| A13-09 | Shared Sylvester: strong HP60/divisor1.6 versus weak50/2. Strong corner pushes are±4/cardinal±4.8, scaled by divisor; same sprite/update,3000 wave, no first1-2 danger/battle-tip modals | `Alien.cpp:60–68,612–648,1035–1039`, `Board.cpp:947–966,3877–3879`; PB17 confirms constructor numbers/class/update, not all steering or stage dispatch |
| A13-10 | Per-hit damage reads Board weapon×3. At strength12, held fire uses old board count divisible5, elapsed>100ms and cursorY>40 | `Alien.cpp:545–549`, `Board.cpp:818–824`; PB13 supports damage field and strict bounds/cooldown |
| A13-11 | Oscars count as surviving fish and valid alien prey. Removing the last guppy does not open Game Over while an Oscar lives. Guppy→Oscar→Alien sorting makes immediate prey removal visible to subsequent actors | `Board.cpp:1958–1969,2014–2108,3215–3220`, `Alien.cpp:826–864,969–1004`, `Oscar.cpp:460–477`; exact retail cross-list tie order remains unknown |
| A13-12 | Third egg settles claimed coins/pearls and records score once, then advances1-4/rewards Itchy(ID2) before retiring board. Hatch Continue starts fresh1-4 with Stinky/Niko/Itchy | `Board.cpp:2177–2184,3293–3360`, `ProfileMgr.cpp:555–564,581–599`, `GameObject.h:69–74`, `WinFishApp.cpp:2362–2373`, `HatchScreen.cpp:630–638`; profile mapping not primary-confirmed |

Oscar presentation uses inherited fish atlases, source rowY320 with80×80 cells and hungry swim/turn/eat variants (`Oscar.cpp:172–198`, `Fish.cpp:1595–1602`). It does not require an invented Oscar texture. Hungry steering is conditional acceleration, not a hard velocity clamp (`Oscar.cpp:245–373`). Exact RNG parity, double/widget lag and inherited animation/death ordering need independent checks.

## Implementation boundary

Preserve the existing runtime. Add an explicit ordinary Oscar actor with owned double motion/animation/hunger/production state; Board retains globally unique IDs, ordered prey membership, purchases and coin creation. Generalize Sylvester only across its two recovered variants, pass Board weapon into shots, and make wave initialization/tutorial gates stage-specific. Persist the new actor, gates, weapon and variant atomically with existing state. Extend survival and alien prey/removal to Oscars.

Extend stage score/reward validation through1-3 and Itchy hatch after those actors work. Next1-4 is a separate increment: its Itchy/Balrog behaviors are not established by this note. Keep an explicit incomplete boundary until their contract and normal-speed run exist.

## Acceptance

Regress independently grounded unlock/payment boundaries, Oscar prey selection versus collision, strict edges, competing predators, hunger/production/deferred death, Oscar-only survival and alien predation. Strong base weapon takes ten accepted hits; weapon3 takes seven. Strong shot knockback must use its divisor and1-3 must not open1-2-only modals. Save/reload retains all actor/owner/RNG state; completion settles once and preserves Itchy across hatch reload.

Require a normal-speed run beginning from earned Niko progress, purchasing Oscar and feeding it with real bought small guppies, collecting its naturally produced diamond, fighting a natural strong wave, and purchasing three2000 eggs through earned money. Inspect heading/animation/input and record losses/failures. Build/tests alone do not establish playability. Retail/audio/human comparisons remain separate acceptance work.
