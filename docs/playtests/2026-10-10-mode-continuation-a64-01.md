# A64 — Normal mode continuation acceptance

Date:2026-10-10 America/Phoenix (execution2026-10-11 UTC). Agent-run scenario; human playtest, retail comparison and audio/feel judgment are unperformed. [Mode contract](../mode-continuation.md) and [UI observation API](../playtest-helpers.md#ui-stage-observation) own expectations. Screensaver is excluded under the current [requirements](../requirements.md#goal-and-constraints).

## Prepared inputs and intended path

Fresh production constructors/actions admit Time Trial1 and retain it while Adventure Tank2-1 is active. Utility performs no simulation steps, conversion or earned-save import. Production `Profiles::decode` accepts current37/profile3; folder `.scratch/fixtures/a64-mode-01` contains only `profiles.json` SHA`66FA5EDA139DB30C3A32BB09568B54E8F594013F1CD873873C27927405A74F05`. Separate fixture receipt SHA`A6B36742D4795F7818E0337C4DA3B288A996776A06703006EFCAE1505134DDE0`. This is **prepared, not earned**; existing E80 genuine lineage remains isolated.

One bounded normal1x muted scenario is intended to cover observed ready Title bounds → actual Menu → Adventure → ongoing Time Trial → pause/save/Menu → identical retained Adventure and shared accounting → graceful quit → game-written manager reopen → Adventure/pause/save/quit. Two processes run sequentially; owned-window input, event/state readback and same-draw captures must agree. No success may be inferred from fixture admission or build alone.

## Attempt01 — Startup failed

Run `.scratch/playtests/a64-mode-acceptance-01`, root-owned exclusive native lease,240s maximum. Checkout `5ae0d74` plus scope/handoff documentation changes; runtime bytes are frozen E117`9f20cb2`, EXE SHA`B71942C7E48B3B647A50B2CB9D637BF4CD829B28BDBDADED81309A25063017F8`. Original game: owned English Steam3320/build2507521.1; wrapper SHA`F54C2C6EE54B00AE6DA7F4BDE15FEB90D0A867ABA6B4151B0277C1F81DA3FF66`.

The original launcher receipt identifies `target/debug`; a parent-linked **relocation receipt**, not a new compile/package, binds the existing frozen normal EXE and dependencies to exact paths. `.scratch/evidence/a64-frozen-game-build.local.json` SHA`5CC111CF999388675E0A55540C5E0C1F87C3A8825341744C7410C05CE6131FF0` preserves launcher/source/toolchain association;16 artifact checks matched. Finalized driver at execution SHA`1CE4D0404B195D0C59A0EF491CC13199DE34F8B319ED119440A6B71FBCF2F214`; validated private invocation uses explicit1x/Mute for native input/save timing.

**Observed failure:** process13608 reported missing `data/Tiny.txt` and exited before an actual Title UI snapshot or runtime event file. Duration6.38s; immutable fixture/build/lease inputs remained unchanged. Result `.scratch/playtests/a64-mode-acceptance-01/result.local.json` SHA`13423E29BEF6FFD6611D1DCA6F2C603D77E7062195DD4D0B8267CE33170781F3`. Only receipt membership passed; Title/Menu/mode/pause/save/reopen and visual checks were **not tested**. No game-written progression is accepted from this attempt.

Cleanup receipt proves `exited=true,exit_code=1,forced=false,close_error=null`; direct retained-PID readback found no process. The driver's broad error text called the nonzero runtime exit unsuccessful cleanup; that wording is corrected separately without changing the failed acceptance outcome. Original installation and protected earned/frozen inputs were not written.

## Correction and next action

**Observed installed metadata:** `properties/resources.xml:39` maps logical `TINY` to `Pix118.txt`; the installed file exists. [Provenance](../provenance.md) owns its identity. Correct the selector's required font and profile-link lookup to `Pix118`, preserving the logical selector font. No new getter/string test is justified; rebuild changed code and rerun the same affected native scenario on identified new bytes. Save format/core fixture behavior is unaffected.

Correction passed format, strict workspace/all-target lint and locked build with **zero tests**. [STATUS](../../STATUS.md#current-build-verification) owns the corrected frozen source/build identity. Preserve every attempt and use new run paths/leases; required remaining game scenarios and retail/audio/human acceptance stay pending.

## Attempt02 — Font correction passes, driver live-log read fails

Run`.scratch/playtests/a64-mode-acceptance-02`, normal1x/Mute,240s bound, frozen corrected EXE SHA`03051CEC81293C15F5096D8E702FC0F9F7565721F4B6E8547A3041F7490551FF`. Source state is`5ae0d74` plus the selector resource correction; source-set SHA`F4B19EF78F80F8F1CD44FD9FCED1FA6A2F0AF4DEE95EA060F3B564609C334AF5`. Exact relocated build receipt SHA`7C877B531DE062B633264C426C8482BD55AF902BF9B4DC3AF39C8EB84ECC9878`; mode fixture unchanged.

**Observed:** owned PID42436 produced Title frames, accepted ready-bound Start, and produced actual Menu frame264 with `Playing` retained Session/tick0. Agent inspected the same-draw Title/menu PNGs: title/background and selector/buttons/profile-link font render. Title retained its loading-banner artwork; the long prepared profile name extends beyond its speech bubble. These visual details are not resolved retail/pixel-fidelity claims. Title checkpoint image SHA`1871EC05D42748FC24A4B8E08A36347D25E33B805F987241DE3AE36307E89704`.

The driver failed at Menu checkpoint when `Get-FileHash` opened the game's active `events.local.jsonl` with incompatible sharing. This is a driver failure, not the prior missing-font runtime failure. Result SHA`2F1485FE905ED014000DEFAE80F95B3D7D4338061F974FA8318BC2219E9A74CC`,8.24s. Cleanup `exited=true,exit_code=0,forced=false,close_error=null`; no retained process remained. Immutable inputs stayed unchanged. Mode/pause/save/reopen acceptance was **not tested**.

Correction records only the live log's path/observed length, explicitly without a digest claim. The existing after-cleanup event index supplies the closed-log hash/content review; no extra snapshot helper or reader suite is introduced. Parser/automatic-variable validation passed. Retry03 runs the same affected scenario and game bytes, with no rebuild or unaffected tests.

## Attempt03 — Menu route did not publish

Fresh run`.scratch/playtests/a64-mode-acceptance-03`, normal1x/Mute240s, same corrected game/fixture, updated driver and exact root-only lease. Ready Title→Menu and production reopen comparison passed, with only the declared VT wall timestamp refreshed. Declared f32 fields compare by their actual source type; no blanket tolerance is used.

Adventure click`(465,81)` recorded client`(678,115)` inside source button bounds`(357,48,217,66)`. No fresh Adventure/Playing profile publication appeared within5s. Final UI remained Menu/frame1027/retained Session tick0; total13.03s. Empty domain events alone cannot establish input rejection: UI pointer/button events are not domain events, and same-mode resume may return no domain events. Successful route still must atomically publish before Menu closes; these requirements stay intact. Cleanup PID29112 exited0 without force; immutable inputs matched.

Bounded source review identified focus, capture/modal and transaction guards. `MenuRuntime::input_frame` returns before mouse polling when the captured still-owned window is not foreground. Actual foreground during attempt03 was unknown, so no production behavior was changed from this inference alone. Result SHA`6B6EFABD1CE540E98AB4545E1FC3287EFD54B8AD71F22834FC2C7511894F5B60`.

## Attempt04 — Foreground access denied

Run`.scratch/playtests/a64-mode-acceptance-04`, same game/fixture and1x/Mute240s. Exact lease permitted a new explicit owned-window focus request. Input receipt directly records `already_foreground=false,activation_requested=true,acquired=false` for PID15308. Windows refused foreground; the helper posted no scenario input. This is an **observed environment blocker**, not proof of a route transaction bug. No fifth retry is authorized merely by elapsed time.

The first focus integration also applied to `WM_CLOSE`, preventing ordinary cleanup; the retained process was then killed through its existing owned handle. Receipt honestly records exitedtrue/exit-1/forcedtrue/closeerror. Resource readback found no remaining process. Result SHA`63C454DD80A74238D1E47BC379BDFFD5008FADE882FE2239D7D8A3A225AAB69B`. Corrected helper now gates only keyboard/mouse input and always permits ownership-checked `WM_CLOSE`; that restores the earlier source cleanup path. Final helper/driver parser and automatic-variable validation passed without executing either script. Subsequent native cleanup behavior remains **not tested**. No game-written data from this forced attempt is accepted as lineage.

Final tooling identities: helper SHA`FD180F1A444D19EC9CE498821A95D9F7145E8FAB5CD2FC0BD3D24F0C562BD2EE`, mode driver SHA`EDF1F6992E7FA3EF89EB10C55406A3A08DADAD0A907D35D82570E2F43D96960A`. These are post-attempt04 tooling identities, not retroactive identities for that run. No game rebuild, unit suite or fifth native attempt followed the cleanup correction.

Native continuation is **incomplete**. It needs an actual foreground project window before further input; root asked whether the user can activate it on the next launch or prefers native checks deferred. Other prepared game lanes remain ready without implicit process/window permissions. Runtime focus guards remain intact; no thread attachment, third-party focus, global input or OS restriction bypass is used. Retail/audio/human judgment stays pending.
