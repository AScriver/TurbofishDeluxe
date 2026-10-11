# A66 — Foreground continuation and Windows save checkpoint

First execution:2026-10-11 06:55–06:57 UTC (2026-10-10 America/Phoenix); subsequent attempts02/03 ran later on2026-10-11 at normal1x muted. Title/Menu/Adventure↔Time Trial/pause/save/reopen acceptance remains incomplete. Human playtest, audible output and retail fidelity are not tested. The [A64 report](2026-10-10-mode-continuation-a64-01.md) owns the unchanged prepared fixture and earlier failures; [mode continuation](../mode-continuation.md) owns expected transactions. The final human-playtesting direction supersedes all next-run plans below.

## Identity and authorization

Checkout `2557c2fbc4c4f263baf6303c3bc24a90fa04d5dd` plus the manual-foreground changes in `scripts/Playtest-Helpers.ps1` and `scripts/Test-NativeModeContinuation.ps1`; no Rust changes. Reused checked E119 bytes, rather than rebuilding or repeating a suite. EXE SHA`6FF6C11F55DCFEA8851B99F487FA43E23742A0246DA88A5EA21E5D63AE075425`; source-set SHA`3A60E6D4DE5AC116DD567366F6B963442E25D73A814D4C8FC7E6FCC4F26D6FF9`. Parent receipt `.scratch/evidence/20261011T005522210Z-635e5f79-build.local.json` SHA`A6CD4A5C2BEDC4463FDA0C94F5C06DD851D9475BC6A8B83B12EEB9B6232FFD63` retains source/toolchain identity. Fourteen existing executable/DLL/notice artifacts were copied into `.scratch/runtime/a66-game-bundle-01`; sixteen recorded artifacts matched. Relocation receipt `.scratch/evidence/a66-frozen-build.local.json` SHA`D538AFDB2832E1C5879BF947850390E2217D811219C87438855B7E67B537BCE6` binds exact frozen paths. This is not a new compilation or screensaver package.

Human authorization for live testing was read directly from coordinator chat `01a12706-929c-74b1-92bf-a27e62ef8a87`, user message `01a129b6-4b6f-7790-b2db-28ac26927629` at06:46 UTC. Coordinator automation remains paused. Root held the only native lease; other four outcome lanes remained withheld. The lease required actual owned-window foreground observation, zero automatic activation requests, two sequential processes maximum,300s total wall deadline and120s manual activation wait per startup. Windows' prior foreground denial was not bypassed.

Prepared current37/profile3 fixture, its receipt and byte hashes are unchanged from A64. Save origin is **prepared-not-earned**. The run copied it into `.scratch/playtests/a66-mode-acceptance-01/save`; no E80/genuine earned-save conversion or default-save use. Owned English Steam3320/build2507521.1 installation supplied assets read-only.

## Attempt01 — Activation timeout and receipt collision

Observed PID3964 reached ready Title,219/219 load tasks. Captured Title frame351 SHA`B618E67F2B9B3A77C56CD850D778C8730EB551F824B91AA68ADA31C6731146AC` was visually inspected: title artwork and loading-banner artwork render. Readiness is corroborated by same-draw metadata; this is not retail pixel acceptance or Start-input proof.

Initial foreground was false. The driver wrote `startup-menu-manual-activation.local.json` at06:55:27 UTC and waited120s. No acquired-foreground receipt or scenario input was produced. After the wait, its immutable JSON writer rejected rewriting that same receipt: **`Evidence already exists`**. This driver defect masked the intended activation-timeout message. The final live sideband still showed ready Title with no Session at wall124.45s. A timely activation prompt was not delivered during the run; this attempt does not show that the user declined activation.

Result `.scratch/playtests/a66-mode-acceptance-01/result.local.json` SHA`8A9C0BEA91E4D406291A2F00179547E506EE3545A2FA2D612CB11702387EB3B4`,127.39s, failed/incomplete. Menu entry, Adventure/TT input, conservation, pause, game-written save and second-process reopen are **not tested**. No runtime event file was created before Start; the driver's final review reported it missing. Owned `WM_CLOSE` cleanup worked without foreground: exitedtrue/exit1/forcedfalse/closeerrornull, with direct PID readback confirming no process. Source `TitleRuntime::check_deadline` reports startup cancellation on quit before gameplay; the exit1 does not establish a tank/runtime failure. No game-written progression is accepted.

All recorded immutable EXE/build/fixture/lease inputs and copied dependencies matched after cleanup. Original installation and protected earned/frozen directories were not mutation targets. Native resource reservation is released with the failed result.

## Attempt01 correction and then-planned next action

The driver now writes a distinct immutable `*-manual-activation-result.local.json` for observed foreground or timeout, preserving the original wait receipt. The shared helper's `RequireOwnedForeground` option rechecks the retained HWND owner and actual foreground before each keyboard/mouse message; it never activates. `WM_CLOSE` remains focus-independent. Manual mode requires an exact helper/driver/wait/policy lease and rejects automatic activation permission. Existing defaults are unchanged.

Attempt01 driver SHA`7E50878975147729F918717EEE9D2DEF787CAF6608A855EA1E2DE1EAF79BDFDF`; helper SHA`3969A72903AA6EC3998DB23B0A5DCDA2EEA70E5AFF424FD97E395BE73CDF23F6`. Corrected driver SHA`65C8EF5A8A2FCCFBA2800461728B2012752CAC59DD3B988E83F793A98912C0A3` passed parser/automatic-variable validation with execution disabled. At that checkpoint the corrected result-writing path was **not native-tested**; subsequent positive-path observations are below. No Cargo, unit suite, decoder repeat or second native attempt accompanied that initial correction.

The then-planned next run required the user's availability to activate the actual project window through titlebar/taskbar, without Start/game controls, and repeat after one serial reopen. Fresh exact lease/run paths and actual foreground readback were required. This plan is historical; current autonomous gameplay is stopped.

## Attempt02 — Foreground observed and access-denied exit

Human readiness changed at20:00 UTC. Root held fresh lease
`.scratch/coordination/a66-mode-native-lease-02.local.json`
(SHA`A39F2AD5C8D1CDACA0F4B428153E6AD7FB47368DD02145B9BB97ECBA60BDC041`)
and reused frozen E119/unchanged prepared current37/profile3 input. PID7380,
HWND26939476 was already foreground: observed in approximately0.0066s, with zero
automatic activation requests. The distinct positive activation result receipt
was written successfully. Title233→Menu270 passed, followed by complete typed
startup readback except the declared VT wall timestamp refresh. Shared
progress/wallet/scores/profile accounting and same-mode Adventure save/active
scene conservation passed.

Stderr reported **`Runtime failed: Access is denied. (os error 5)`**. The driver
failed **`Process exited awaiting actual Scene UI draw`**, but final telemetry
contained Scene319/Playing/tick0 at wall5.8265s; the driver had not consumed that
draw before exit. Do not interpret this as proof that Scene never drew. No Scene
capture was accepted. Title/Menu captures are retained but their new pixels were
not inspected. Empty domain events are not rejection proof for this same-mode
route. `profiles.json` and `profiles.pending` both remained14546 bytes; their
private hashes and final sideband preserve the failed publication generation.
The exact native failing call is not traced; a later checkpoint is the leading
source-supported explanation.

Run `.scratch/playtests/a66-mode-acceptance-02` lasted8.5775939s.
Result SHA`3F9F4875C70C63A6F065D83B57B735920381CE762E728429B78202C241960EC2`;
cleanup exited1/no force/closeerrornull. Immutable inputs matched and the resource
was released. TT transition, pause and second-process reopen were not performed.

## Windows host probes and bounded correction

The path-based PowerShell hash reader holds `File.OpenRead` sharing while polling
the live save. Platform source and licensing are in [provenance](../provenance.md).
The original `cli::write_json` published via pending-file rename. Isolated probe
files in `.scratch/probes` tested the platform mechanism without game/save/install
mutation:

| Probe | Actual result and conclusion |
|---|---|
| `a66-hash-sharing-01` | Rename was denied; the first script caught IOException but not UnauthorizedAccessException, so no success receipt was written. Preserve the failed files. |
| `a66-hash-sharing-02` | Old reader denial and success after close were observed. A delete-shared reader also denied rename; this falsified the proposed sharing-only fix. |
| `a66-replace-file-01` | The alternative failed before replacement because PowerShell coerced a null backup path to an empty string. This is a probe argument failure, not API failure evidence. |
| `a66-replace-file-02` | Corrected `[NullString]::Value` let File.Replace publish a new generation while the reader retained the old one. The candidate was not adopted because its documented partial-failure semantics would widen the publication contract. |

E120 retains safe Rust rename publication with a bounded retry and explicit path
in its error. No destination deletion/native replacement dependency/schema change
was introduced. Its new transient-reader test publishes the complete new JSON
after a20ms held reader; the existing sustained-denial test proves the old
complete generation survives failure and later publication recovers. Both passed;
107 unrelated cases stayed filtered. Formatting, strict production/persistence
lint and locked build/staging passed once after the correction. Exact commands,
toolchain/source/build receipts and frozen E120 EXE are owned by
[STATUS](../../STATUS.md#current-build-verification). No broad suite or repeated
passing filter followed. Corrected mode driver
SHA`A8668AD498B931C0A98722F9010F89F583E1B987D70CC64032C57F7788B5C7BF`
passed parser/automatic-variable checks; shared helper bytes remained unchanged.
These are platform/persistence observations, not retail behavior proof.

## Attempt03 — Corrected build and incomplete route

Fresh lease `.scratch/coordination/a66-mode-native-lease-03.local.json`
SHA`8432167B3A3925A9795914C599E1BEE273254FD4BFF17E4B482F53AAD20ECFAC`
bound frozen E120, the corrected driver and unchanged prepared-not-earned fixture.
PID38888/HWND2559154 was already foreground, with zero automatic activation
requests. Title221→Menu258 and complete typed startup readback passed except the
declared VT wall timestamp refresh. The driver posted Title Start then Adventure.
It failed **`Timed out awaiting fresh profile publication for Adventure/Playing; last read error: `**.
Final telemetry remained Menu711/Playing retained Session/tick0 at wall11.1904s.
This attempt does not prove Adventure input acceptance, mode conservation or
native effectiveness of the save fix; the cause of the incomplete route is
unknown. No repeated access-denied runtime exit was recorded.

Run `.scratch/playtests/a66-mode-acceptance-03` lasted14.3190613s;
result SHA`EB19CC68E43DD4A2275CF74B545F2BF65E86CFF1B0C2784ED942B2A8DA2C9260`.
Owned cleanup completed20:21:16 UTC, exited0/no force/closeerrornull; no second
process was started. Native resource and monitor handoff are released-with-result.
The subsequent human-testing instruction ended further autonomous scenarios;
there was no fourth attempt. The positive activation-receipt path is observed;
the corrected timeout branch, mode/pause/reopen, current tank visuals and audio
remain not tested/incomplete. Prepared inputs and old results remain intact.

## Default profile and human launch handoff

The user's normal launcher output independently reported **`unsupported profile format`**.
Read-only inspection found default project `profiles.json` at format1; current
support is3. `load_profiles` decodes that existing manager before `--new-game`
resets the selected session. The user explicitly allowed clearing old profiles.
At20:13:17 UTC root moved only that stale file into
`.scratch/archives/a66-stale-default-profile-01/profiles.json`, byte-exact
SHA`AAC63E392DF9E7B3B76E233D171F3753B909E94C19CC0B8D5BADEAF3833E4687`.
The adjacent `receipt.local.json` records one file changed and the original
default profile absent. Settings/lock, retail installation/saves, genuine E80
test lineage and all prepared fixtures were untouched. No deletion or migration
was added. A new current manager on next normal startup is source-supported;
actual human startup/play remains unconfirmed.

Normal-speed launch, audio enabled, from PowerShell:

```powershell
Set-Location C:\Code\TurbofishDeluxe
& .\scripts\Start-TurbofishDeluxe.ps1 -GameArguments @('--new-game', '--seed', '42')
```

On later runs, omit `--new-game` to retain and resume progress. No test-speed,
mute, isolated-fixture or profile override is needed for ordinary play. The
maintained launcher stages required native dependencies and writes build identity.
The human now owns normal gameplay and reports issues. Root alone performs
engineering/focused checks; coordinator heartbeat stays paused and four prepared
lanes stay inactive. Do not interfere with a human-owned game process or hash its
live saves for a test. Full requirements remain unchanged; unreported gameplay,
retail comparison, audible fidelity and human judgment remain pending.
