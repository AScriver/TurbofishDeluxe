# Turbofish Deluxe

An intended standalone Rust reimplementation of Insaniquarium Deluxe, targeting Windows first and reading assets from each user's owned installation. This is an unofficial fan project, unaffiliated with the game's developers or publisher.

The workflow is installed. Normal-speed Rust runs cover feeding/growth/coins, earned first-stage completion, Stinky collection and save/reload. Fish facing was corrected and visually checked. Full gameplay fidelity remains unverified. The installed game binaries are authoritative and WinFish is secondary. See [STATUS.md](STATUS.md) for current evidence and gaps.

The complete runtime goal remains in [requirements](docs/requirements.md). The [coverage checklist](docs/compatibility.md) records missing systems; the next integrated task is [Adventure1-3 Oscar, strong Sylvester and progression](docs/adventure-1-3.md).

## Start here

- Agents: read [AGENTS.md](AGENTS.md), then follow its startup order.
- Engineering approach and pending choices: [docs/DESIGN.md](docs/DESIGN.md).
- Change history: [MODLOG.md](MODLOG.md).
- Repeatable comparisons: [playtest template](docs/playtests/TEMPLATE.md).
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

Use the installed Rust1.95 MSVC toolchain from PowerShell7. Direct dependencies and transitives are pinned. Formatting, warnings-as-errors lint,50 regression tests and native build passed. Identified window reports cover the [first loop](docs/playtests/2026-10-08-m1-03.md), [first-stage completion/persistence/rescue](docs/playtests/2026-10-08-m2-01.md) and [Stinky/score/save checks](docs/playtests/2026-10-08-stinky-score-01.md); full retail fidelity remains open.

```powershell
$cargoExecutable = (Get-Command cargo).Source
$buildArguments = @('build', '--locked')
& $cargoExecutable @buildArguments
$runtimeExecutable = '.\target\debug\turbofish-deluxe.exe'
$runtimeArguments = @('--new-game', '--seed', '42')
& $runtimeExecutable @runtimeArguments
```

Steam discovery is automatic; pass `--game-dir <directory>` if needed. Project saves use `%LOCALAPPDATA%/TurbofishDeluxe`, separate from retail saves; use `--save-dir` for isolated checks. Escape pauses; S saves; Q while paused or closing the window saves/exits. Versions1–3 project saves migrate atomically to format4; [migration rules](docs/adventure-1-2.md#persistence-and-current-validation) keep unknown historical state explicit. Continue is clickable (Enter also works); holding the hatch background skips its intro. The1-2 upgrades/combat/Niko build passes85 tests and [normal-speed earned play, pearl reload and Game Over/reentry](docs/playtests/2026-10-08-adventure-1-2-01.md). Later stages remain incomplete.

`--evidence-dir` writes identity/events/captures and optional complete state snapshots. Current snapshots include `state` (board or null), `phase`, `progress` and `session_tick`; board ticks reset on a fresh stage while session ticks remain monotonic. The event log includes both time scopes. These observation files are private, separate from authoritative project saves.

```powershell
$validatorPath = Join-Path $env:USERPROFILE '.codex\tools\Invoke-CodexPowerShell.ps1'
& $validatorPath -Path .\scripts\Test-Runtime.ps1 -Execute
```

The maintained runtime check runs formatting, warnings-as-errors lint, tests and native build. `--inspect-assets` checks actual installed images/effects/fonts without a window;246 declared images,65 effects and15 bitmap fonts passed. Original save compatibility is not implemented.

## Local data and licensing

Keep reference downloads and probes under `.scratch/`, local paths/configuration under `local/`, and proprietary content/captures under `private/`; all are ignored. Keep these directories out of attachments and releases. No machine-local configuration is needed to run the workflow checker.

No game binaries or assets are distributed. AI assisted this documentation setup; see [provenance](docs/provenance.md#tools-and-human-input). The guide-derived documentation retains its [MIT notice](THIRD-PARTY-NOTICES.md). A license for future original runtime code has not been chosen.
