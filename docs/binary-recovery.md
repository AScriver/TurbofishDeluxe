# Remaining-runtime binary recovery

Current phase: first pass in progress. The user's2026-10-09 direction prioritizes coverage across every remaining subsystem after the coherent [A35 checkpoint](adventure-4-5.md). Native/manual acceptance stays deferred. Complete this phase only when every row below has an inspected usable behavior outline, evidence references and explicit gaps; then resume coherent implementation automatically. [Requirements](requirements.md) owns scope; [compatibility](compatibility.md) owns implementation/acceptance coverage; [STATUS](../STATUS.md) owns the handoff.

## Identity and private evidence

Reuse the identified `private/ghidra/insaniquarium-steam-250752/InsaniquariumSteam250752` database, program `embedded-popcapgame.exe`, imagebase00400000, x86:LE:32/default/windows. Authority: installed English1.1, Steam3320/build250752, embedded PB05 payload SHA`312021E970ABD8AFE417D24E99FF75AC8379EF2A93BC1E04D52D4F3D7F070BB1`. Source/license boundaries are in [provenance](provenance.md). This phase uses static identified-binary evidence; no game/runtime process or save is mutated.

The collector alone opens the database, read-only/noanalysis. Indexed raw function inventory/call edges/RTTI tables, per-function pseudocode/status and batch manifests remain ignored in `private/recovery-first-pass/`; scripts/logs and isolated investigator notes remain in `.scratch/runtime/recovery-first-pass/`. Independent reviewers never open the shared database or mutate collector status. The primary maintains sanitized findings and adjudicates conflicts.

Initial inventory:9,193 functions and37,428 call references, including framework/library code. These counts do not imply game subsystems were recovered. Selected functions run in resumable bounded batches with20-second per-function timeouts, atomic result/status records and retained failures. `005073e0` resource-registration decompilation timed out; the failure is retained, with caller/table inspection deferred unless it blocks a usable outline. No unbounded retry.

Function inventory SHA`07B8052631DBE35622F5E7126292D934AF669A632B7BE0972FB237975F43C965`; call-edge SHA`42872F31A7ECCC69931A4FB6506EB8CA29D03660848C5E2442B4714DB50498D6`. Root hash reconciliation and final attempted/decompiled/reviewed counts are pending. Collector exports use `reviewed=false`; separate address logs record semantic inspection. Decompilation success is not recovered behavior. Small details are investigated immediately only when they materially block recovery or safe implementation; other gaps retain exact follow-up addresses.

## Coverage map

| Remaining family | First-pass responsibility and required outline | Current state |
|---|---|---|
| Adventure finale / post-final flow | Boss construction, child spawn/targeting, pet loss, health/combat, death cleanup, completion/reward/ending/repeated Adventure | Reviewing roots; usable complete outline pending |
| Remaining pets / special fish | Enumerated type/factory/vtable families beyond current roster, ownership/state/update/input/production/death/transform | Dispatch exports underway; outline pending |
| Time Trial | Entry/tank gates, setup/clock/scoring, purchases/pet eggs, timeout/results/rewards/save/reopen | Reviewing primary Board/menu roots; outline pending |
| Challenge | Entry/tank gates, escalating waves/prices, egg/reset/victory/results/rewards/save/reopen | Reviewing primary Board/menu roots; outline pending |
| Sandbox / relax | Availability, separate mode/flag values, setup/input/update/progression and persistence | Primary dispatch/flag associations pending |
| Virtual Tank simulation | Object/property/name/age/diet/growth/death/shell state, list ownership/update order, offline/live clocks/capacity | Screen roots reviewed; simulation outline pending |
| Virtual Tank setup / store | Selection/hide/sell/rename, backgrounds/accessories/pets/screensaver, products/stock/price/purchase/confirm/refund | Callback/factory exports underway; outline pending |
| Menus / screen flow / options | Main/title/story/help/pause/restart, screen ownership, focus/input, settings application/storage | Entry/draw roots reviewed; handler outline pending |
| Profiles / unlocks | Create/name/select/delete, current profile ownership/progression/currency/unlock state | Profile operations pending |
| Highscores / records | Mode/tank keys, score insertion/sort/storage/display and results acknowledgement | Screen constructor reviewed; record operations pending |
| Persistence | Retail sections/load-failure/ownership/order; current project-save requirements and safe isolation | Retail roots pending; original-save import remains out of scope |
| Audio / effects | Resources/event dispatch, cue/mixer/pause/fade/loop ownership; particles/shot/death/effect update/draw; playlists | Playlist roots reviewed; dispatch/lifecycle outline pending |

Every eventual outline records main state, authority/ownership, update order, transactions and transitions with binary addresses and evidence class. Qualified W1 names do not silently become primary facts. Exact timings/visual/audio/RNG fidelity and human judgment stay explicit gaps wherever unverified. Missing exports indicate incomplete review, not absent behavior.
