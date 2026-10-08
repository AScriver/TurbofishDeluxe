# Turbofish Deluxe

An intended standalone Rust reimplementation of Insaniquarium Deluxe, targeting Windows first and reading assets from each user's owned installation. This is an unofficial fan project, unaffiliated with the game's developers or publisher.

The workflow is installed and the first-tank Rust feeding/growth/coin loop passed a normal-speed window run. Fish facing was corrected and visually checked. Full gameplay fidelity remains unverified. The installed game binaries are authoritative and WinFish is secondary. See [STATUS.md](STATUS.md) for current evidence and gaps.

The complete runtime goal remains in [requirements](docs/requirements.md). The [coverage checklist](docs/compatibility.md) records missing systems; the next integrated task is [first Adventure completion](docs/DESIGN.md#m2-next-integrated-outcome).

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

Use the installed Rust1.95 MSVC toolchain from PowerShell7. Direct dependencies and transitives are pinned. Formatting, warnings-as-errors lint,26 regression tests and native build passed. The [controlled window report](docs/playtests/2026-10-08-m1-03.md) identifies the tested build and limits.

```powershell
$cargoExecutable = (Get-Command cargo).Source
$buildArguments = @('build', '--locked')
& $cargoExecutable @buildArguments
$runtimeExecutable = '.\target\debug\turbofish-deluxe.exe'
$runtimeArguments = @('--new-game', '--seed', '42')
& $runtimeExecutable @runtimeArguments
```

Steam discovery is automatic; pass `--game-dir <directory>` if needed. Project saves use `%LOCALAPPDATA%/TurbofishDeluxe`, separate from retail saves; use `--save-dir` for isolated checks. `--evidence-dir` records identity, normal-speed events/state, and requested screenshots. Escape pauses; S saves; Q while paused saves/exits. These commands currently support the first tank only.

```powershell
$validatorPath = Join-Path $env:USERPROFILE '.codex\tools\Invoke-CodexPowerShell.ps1'
& $validatorPath -Path .\scripts\Test-Runtime.ps1 -Execute
```

The maintained runtime check runs formatting, warnings-as-errors lint, tests and native build. `--inspect-assets` checks actual installed images/effects/fonts without a window;246 declared images,65 effects and15 bitmap fonts passed. Original save compatibility is not implemented.

## Local data and licensing

Keep reference downloads and probes under `.scratch/`, local paths/configuration under `local/`, and proprietary content/captures under `private/`; all are ignored. Keep these directories out of attachments and releases. No machine-local configuration is needed to run the workflow checker.

No game binaries or assets are distributed. AI assisted this documentation setup; see [provenance](docs/provenance.md#tools-and-human-input). The guide-derived documentation retains its [MIT notice](THIRD-PARTY-NOTICES.md). A license for future original runtime code has not been chosen.
