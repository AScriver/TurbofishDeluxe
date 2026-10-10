# Turbofish Deluxe

An intended standalone Rust reimplementation of Insaniquarium Deluxe, targeting Windows first and reading assets from each user's owned installation. This is an unofficial fan project, unaffiliated with the game's developers or publisher.

The workflow is installed. Normal-speed Rust runs cover earned1-1 through2-5 completion, feeding/growth/currency, Stinky/Niko/Oscar/Itchy/Prego/Zorf/Clyde/Starcatcher/Vert/Rufus/Meryl, weak/strong/Balrog/Gus/Destructor combat and missiles, upgrades, Wadsworth hatch/ten-pet selection, both shell bonuses/results, Potion→Star and save/reload. A separate controlled run exercises the finale's paired aliens. Fish facing was corrected and visually checked. Full gameplay fidelity remains unverified. Installed binaries are authoritative and WinFish is secondary. See [STATUS.md](STATUS.md) for evidence and gaps.

The complete runtime goal remains in [requirements](docs/requirements.md). The [coverage checklist](docs/compatibility.md) records missing systems. [Tank3-5](docs/adventure-3-5.md) passed earned Rhubarb/third-bonus/selection/reload acceptance and a [native pause follow-up](docs/playtests/2026-10-09-adventure-3-5-bonus-pause-02.md). [Tank4-1](docs/adventure-4-1.md) passed the repaired370-check build and [eight audited native runs](docs/playtests/2026-10-09-adventure-4-1-breeder-rhubarb-nimbus-01.md). [Tank4-2/live Nimbus/Bilaterus/Ultravore](docs/adventure-4-2.md) passes407checks and has partial native evidence preserved on a frozen bundle; detailed acceptance is deferred. Current actions and identities are in [STATUS](STATUS.md). Audio listening and retail comparison remain pending.

## Start here

- Current31 adds the 40,000-shell [fourth pet slot](docs/extra-pets.md#current31-implementation-checkpoint) after Brinkley, Nostradamus, Stanley and Walter. Adventure/replay and initial Time Trial selections consume four choices; acquired TT pets and Tank5's fixed roster stay separate. [Checkpoint checks](MODLOG.md#2026-10-10--verify-fourth-slot-and-preserve-current31) passed. [Completed Adventure replay](docs/adventure-replay.md) preserves earned history and retained-Board results. Next: Challenge and remaining modes. The [broad recovery](docs/binary-recovery.md) covers all remaining systems; native/manual acceptance stays deferred.
- Agents: read [AGENTS.md](AGENTS.md), then follow its startup order.
- Engineering approach and pending choices: [docs/DESIGN.md](docs/DESIGN.md).
- Change history: [MODLOG.md](MODLOG.md).
- Repeatable comparisons: [playtest template](docs/playtests/TEMPLATE.md).
- Native input, state waits, pause/reload and cleanup: [maintained playtest helpers](docs/playtest-helpers.md).
- Studied sources, attribution, licenses: [provenance](docs/provenance.md).
- Workflow source mapping and adaptations: [setup note](docs/workflow-setup.md).
- Existing reverse-engineering tools, invocation, and verified limits: [shared analysis tools](docs/analysis-tools.md).

## Workflow checks

Run from the repository root in PowerShell 7 with Git on PATH:

```powershell
$gitExecutable = (Get-Command git).Source
$gitArguments = @('status', '--short', '--branch')
& $gitExecutable @gitArguments
$powerShellExecutable = (Get-Command pwsh).Source
$workflowArguments = @('-NoProfile', '-File', '.\scripts\Test-Workflow.ps1')
& $powerShellExecutable @workflowArguments
```

The checker validates required documents, local Markdown links and heading anchors, whitespace, allowlist behavior, and protected-path exclusions without staging or changing files. It does not validate gameplay, remote URLs, content licensing, or automatic instruction discovery by a fresh Codex session.

On the current machine, use the existing PowerShell parser/automatic-variable validator before executing a changed script:

```powershell
$validatorPath = Join-Path $env:USERPROFILE '.codex\tools\Invoke-CodexPowerShell.ps1'
& $validatorPath -Path .\scripts\Test-Workflow.ps1 -Execute
```

That host helper requires PSScriptAnalyzer; it was available for setup. It is optional for a fresh clone's workflow checker and is not vendored or installed by this repository. Environment findings and exact verification results are in [STATUS](STATUS.md#setup-verification).

## Build and run

Use Rust1.95 MSVC and PowerShell7. The accepted A24 working tree passed formatting, warnings-as-errors lint,226 tests (223 runtime and three binding checks, including the owned music cases), native build/DLL staging and [normal-speed Destructor/Rufus/Meryl execution](docs/playtests/2026-10-08-adventure-2-4-destructor-meryl-01.md). Earlier identified window reports cover the [first loop](docs/playtests/2026-10-08-m1-03.md), [first-stage completion/persistence/rescue](docs/playtests/2026-10-08-m2-01.md), [Stinky/score](docs/playtests/2026-10-08-stinky-score-01.md), [1-2](docs/playtests/2026-10-08-adventure-1-2-01.md), [1-3](docs/playtests/2026-10-08-adventure-1-3-01.md), [1-4/selection/Prego](docs/playtests/2026-10-08-adventure-1-4-01.md), [1-5/bonus/early2-1](docs/playtests/2026-10-08-adventure-1-5-bonus-2-1-01.md), [2-1/Clyde/2-2/Vert](docs/playtests/2026-10-08-adventure-2-1-clyde-2-2-vert-01.md) and [2-3/Gus/earned Rufus](docs/playtests/2026-10-08-adventure-2-3-gus-rufus-01.md). Full retail fidelity remains open.

```powershell
$launcherPath = '.\scripts\Start-TurbofishDeluxe.ps1'
$runtimeArguments = @('--new-game', '--seed', '42')
& $launcherPath -GameArguments $runtimeArguments
```

The launcher verifies the pinned general-purpose native audio package, builds and stages its DLLs/notices beside the executable. `-PrepareOnly` stages without launching; `-Offline` uses the verified cache. This native package is required even with `--mute`. Details and license boundaries are in [audio design](docs/audio.md).

Steam discovery is automatic; pass `--game-dir <directory>` if needed. Project saves use `%LOCALAPPDATA%/TurbofishDeluxe`, separate from retail saves; use `--save-dir` for isolated checks. Escape or the top Menu pauses an active tank. S saves; Q while paused or closing the window saves/exits. Latest verified project saves use format29; [save scope](docs/requirements.md#goal-and-constraints) does not require new legacy compatibility. Continue is clickable (Enter also works); holding the hatch background skips its intro. Choose up to three unlocked pets when selection is available. Verified code supports late Adventure, Time Trial, logical Presto, completed replay and the Brinkley/Nostradamus/Stanley purchases; its complete619-check gate establishes code behavior. Earned Adventure through4-1 and three bonuses has identified native acceptance;4-2 has partial evidence, and later native acceptance remains deferred. [STATUS](STATUS.md) owns build identities, results and remaining systems.

`--evidence-dir` writes identity/events/captures and optional complete state snapshots. Current snapshots include `state` (board or null), `phase`, `progress` and `session_tick`; board ticks reset on a fresh stage while session ticks remain monotonic. The event log includes both time scopes. These observation files are private, separate from authoritative project saves.

```powershell
$validatorPath = Join-Path $env:USERPROFILE '.codex\tools\Invoke-CodexPowerShell.ps1'
& $validatorPath -Path .\scripts\Test-Runtime.ps1 -Execute
```

The maintained runtime check preflights native files, runs workspace formatting/lint/tests, then builds and stages DLLs/notices. Invoke `Test-Runtime.ps1 -GameDirectory <owned-install-directory>` to include installed MO3 decoding; without that parameter it explicitly reports those checks skipped. `--inspect-assets` checks actual installed images/effects/fonts without a window;246 declared images,65 effects and15 bitmap fonts passed. Original save compatibility is not required.

### Focused runtime checks

Use `-Check` for faster feedback while editing. The default `All` remains the complete milestone build/regression gate; focused checks do not replace it. The user deferred native/manual acceptance while reversing and implementation continue; normal-speed evidence is still required for a playability claim.

| Check | Runs |
|---|---|
| `Format` | Workspace formatting check, without native preflight |
| `Lint` | Strict workspace Clippy for all targets |
| `Tests` | All workspace test targets, without formatting/lint or runtime staging |
| `Unit` | Runtime library unit tests |
| `Assets`, `Fonts`, `Persistence` | The named runtime integration test suite |
| `Binding` | Native decoder binding tests; owned music cases remain ignored |
| `OwnedMusic` | Ignored installed MO3 decoder tests; requires `-GameDirectory` |
| `Build` | Native build and DLL/notice staging, without launching |

```powershell
.\scripts\Test-Runtime.ps1 -Check Format
.\scripts\Test-Runtime.ps1 -Check Unit -TestFilter 'bilaterus::tests::'
.\scripts\Test-Runtime.ps1 -Check Persistence -TestFilter 'current_tank_four_second'
.\scripts\Test-Runtime.ps1 -Check Assets
.\scripts\Test-Runtime.ps1 -Check Build
```

`-TestFilter` is a test-name substring supported by `Unit`, `Assets`, `Fonts`, `Persistence`, `Binding` and `OwnedMusic`. It is rejected for `All`, so the complete gate always runs unfiltered. Check Cargo's reported test count when filtering. All checks except `Format` verify the native audio package and configure its import library/DLL path; only `All` and `Build` stage the executable. `-GameDirectory` is supported by `All` and `OwnedMusic` only.

After changing a PowerShell script, run the host validator first; its `-ArgumentList` forwards focused options:

```powershell
$validatorPath = Join-Path $env:USERPROFILE '.codex\tools\Invoke-CodexPowerShell.ps1'
& $validatorPath -Path .\scripts\Test-Runtime.ps1 -Execute -ArgumentList @('-Check', 'Format')
```

### Accelerated gameplay tests

Use `--test-speed <1..8>` for explicitly accelerated exploratory/stress runs. Default gameplay remains 1x. Above 1x requires a dedicated `--save-dir`, a separate `--evidence-dir`, `--mute` and a positive wall-clock `--quit-after` limit. The save format and game rules are unchanged. For example, from an already prepared build:

```powershell
.\target\debug\turbofish-deluxe.exe --new-game --seed 42 --test-speed 8 --save-dir .scratch/speed-example/save --evidence-dir .scratch/speed-example/evidence --mute --quit-after 30
```

Future automated gameplay tests explicitly select 8x: use `Start-TurbofishPlaytest -TestSpeed 8 -Mute` or `Test-Playtest.ps1 -TestSpeed 8 -Mute` for fresh isolated runs with owned-window input and cleanup. Use 4x when the scenario's input/polling cannot keep up at 8x and record why. Explicit 1x is reserved for checks that need normal timing, including milestone acceptance and render/audio/retail comparison. The [helper guide](docs/playtest-helpers.md) owns usage; the helper API retains its 1x default for older frozen executables, so new automated scenarios must supply the speed explicitly. The visible banner and telemetry identify the factor. `elapsed_seconds` remains wall time; `session_elapsed_seconds` is the run's fixed-step session time, including paused session updates. Global pause continues at 1x.

The 28ms updates remain intact, but more updates between physical rendered frames can change Bilaterus connector observations and later motion. Accelerated tests do not establish identical 1x traces, normal-speed playability, audio or retail fidelity. Keep the [normal-speed acceptance gate](docs/DESIGN.md#m1-execution-acceptance), and record the actual factor/inputs/observation limits in each report.

## Evidence bookkeeping

Successful launcher builds now create immutable build identities automatically. [Evidence tooling](docs/evidence-tooling.md) documents explicit output paths, single-run manifests, artifact/source readback, private scenario inputs and generated playtest drafts. Raw receipts stay ignored; generated behavioral verdicts remain **not tested** until reviewed. Readback checks a run's referenced build as well as its own artifacts, and `-CheckSources` identifies exactly which inputs changed.

```powershell
.\scripts\Start-TurbofishDeluxe.ps1 -Offline -PrepareOnly -BuildIdentityPath .scratch/evidence/example-build.local.json
.\scripts\Test-Evidence.ps1 -RecordPath .scratch/evidence/example-build.local.json -CheckSources
```

Run manifests and report drafts consume completed evidence; they do not launch gameplay. The tooling passed 47 synthetic contracts, independent negative-case rechecks, a real prepared-build identity/readback and read-only historical telemetry consumption. The paused A32 executable remains unchanged.

## Local data and licensing

Keep reference downloads and probes under `.scratch/`, local paths/configuration under `local/`, and proprietary content/captures under `private/`; all are ignored. Keep these directories out of attachments and releases. No machine-local configuration is needed to run the workflow checker.

No game binaries or assets are distributed. AI assisted this documentation setup; see [provenance](docs/provenance.md#tools-and-human-input). The guide-derived documentation retains its [MIT notice](THIRD-PARTY-NOTICES.md). A license for future original runtime code has not been chosen.
