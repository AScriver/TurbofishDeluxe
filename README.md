# Turbofish Deluxe

An intended standalone Rust reimplementation of Insaniquarium Deluxe, targeting Windows first and reading assets from each user's owned installation. This is an unofficial fan project, unaffiliated with the game's developers or publisher.

The workflow is installed. Normal-speed Rust runs cover earned1-1 through2-5 completion, feeding/growth/currency, Stinky/Niko/Oscar/Itchy/Prego/Zorf/Clyde/Starcatcher/Vert/Rufus/Meryl, weak/strong/Balrog/Gus/Destructor combat and missiles, upgrades, Wadsworth hatch/ten-pet selection, both shell bonuses/results, Potion→Star and save/reload. A separate controlled run exercises the finale's paired aliens. Fish facing was corrected and visually checked. Full gameplay fidelity remains unverified. Installed binaries are authoritative and WinFish is secondary. See [STATUS.md](STATUS.md) for evidence and gaps.

The complete runtime goal remains in [requirements](docs/requirements.md). The [coverage checklist](docs/compatibility.md) records missing systems. [Tank3-5](docs/adventure-3-5.md) passed earned Rhubarb/third-bonus/selection/reload acceptance and a [native pause follow-up](docs/playtests/2026-10-09-adventure-3-5-bonus-pause-02.md). [Tank4-1](docs/adventure-4-1.md) passed the repaired370-check build and [eight audited native runs](docs/playtests/2026-10-09-adventure-4-1-breeder-rhubarb-nimbus-01.md), including earned Breeder/Rhubarb/Nimbus, current16 reload and Board pause. [Tank4-2/live Nimbus/Bilaterus/Ultravore](docs/adventure-4-2.md) is implemented and passes407checks; native gameplay remains untested at the user-requested pause. Resume instructions and verification identities are in [STATUS](STATUS.md). Audio listening and retail comparison remain pending.

## Start here

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

Steam discovery is automatic; pass `--game-dir <directory>` if needed. Project saves use `%LOCALAPPDATA%/TurbofishDeluxe`, separate from retail saves; use `--save-dir` for isolated checks. Escape pauses; the top Menu also pauses an active tank. S saves; Q while paused or closing the window saves/exits. The working tree uses current format16; [save scope](docs/requirements.md#goal-and-constraints) does not require legacy-save compatibility. Continue is clickable (Enter also works); holding the hatch background skips its intro. Oscar becomes available after Large growth in relevant stages; buying it opens the mapped egg/weapon purchases. Tank4-1 uses Breeders and hides the guppy/Ultra/weapon shop slots. After Prego, choose up to three unlocked pets. Earned Adventure through3-5/Rhubarb and the first three bonuses passed identified execution checks;4-1earned acceptance is running and4-2entry remains gated. Current build/results and remaining systems are in [STATUS](STATUS.md).

`--evidence-dir` writes identity/events/captures and optional complete state snapshots. Current snapshots include `state` (board or null), `phase`, `progress` and `session_tick`; board ticks reset on a fresh stage while session ticks remain monotonic. The event log includes both time scopes. These observation files are private, separate from authoritative project saves.

```powershell
$validatorPath = Join-Path $env:USERPROFILE '.codex\tools\Invoke-CodexPowerShell.ps1'
& $validatorPath -Path .\scripts\Test-Runtime.ps1 -Execute
```

The maintained runtime check preflights native files, runs workspace formatting/lint/tests, then builds and stages DLLs/notices. Invoke `Test-Runtime.ps1 -GameDirectory <owned-install-directory>` to include installed MO3 decoding; without that parameter it explicitly reports those checks skipped. `--inspect-assets` checks actual installed images/effects/fonts without a window;246 declared images,65 effects and15 bitmap fonts passed. Original save compatibility is not required.

### Focused runtime checks

Use `-Check` for faster feedback while editing. The default `All` remains the complete milestone build/regression gate; focused checks do not replace it or the required normal-speed gameplay acceptance.

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

## Local data and licensing

Keep reference downloads and probes under `.scratch/`, local paths/configuration under `local/`, and proprietary content/captures under `private/`; all are ignored. Keep these directories out of attachments and releases. No machine-local configuration is needed to run the workflow checker.

No game binaries or assets are distributed. AI assisted this documentation setup; see [provenance](docs/provenance.md#tools-and-human-input). The guide-derived documentation retains its [MIT notice](THIRD-PARTY-NOTICES.md). A license for future original runtime code has not been chosen.
