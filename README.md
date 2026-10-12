# Turbofish Deluxe

This project requires a valid license for Insaniquarium Deluxe. You can purchase the game on [Steam](https://store.steampowered.com/app/3320/Insaniquarium_Deluxe/).

A standalone Rust reimplementation of Insaniquarium Deluxe, targeting Windows first and reading assets from each user's owned installation. This is an unofficial fan project, unaffiliated with the game's developers or publisher.

Implemented systems include Adventure through the finale and replay, Time Trial, Challenge, Virtual Tank/store, Sandbox and Relax code paths, profiles/highscores, menus and audio. The [coverage checklist](docs/compatibility.md) separates implementation from remaining behavior and acceptance gaps; Relax's retail activation remains unknown. Full-runtime acceptance and retail fidelity are unverified. Installed binaries are authoritative and WinFish is secondary.

Recorded normal-speed Adventure acceptance reaches [Tank4-1](docs/playtests/2026-10-09-adventure-4-1-breeder-rhubarb-nimbus-01.md) and three shell bonuses. [Tank4-2](docs/playtests/2026-10-09-adventure-4-2-native-a32-01.md) has partial native evidence on its frozen build; later paths and other modes still need gameplay acceptance. These historical runs do not accept later source changes. Ordinary gameplay belongs to the human, who reports issues for targeted fixes. [STATUS](STATUS.md) owns current build identities and next actions; [requirements](docs/requirements.md) retain the complete runtime goal. Audible and retail comparisons remain pending where unobserved.

## Start here

- Latest checked changes: [eight Rust review corrections](docs/rust-review-fixes.md#combined-completion), [guppy wall-turn order](docs/guppy-turn-boundaries.md) and [current-only save admission](docs/refactor-save-codec.md#remaining-development-save-compatibility). The review corrections and guppy fix are joined on main; detailed checks and limitations stay in their owning records.
- [Completed modal frontends](docs/modal-batch-303-307.md#completion-and-joined-verification) cover options/pause, profile forms, quit/notices, pet confirmation/rescue/tutorial/Game Over and shell transfers.
- [Performance improvements](docs/performance.md) now cover optimized default launches, batched resource loading, active Virtual Tank views, profile events and checkpoints. The human plays normally and reports issues; focused tests cover detailed rules and regressions. Earlier native reports and [retained modes](docs/mode-continuation.md) remain preserved in [STATUS](STATUS.md). Further screensaver work is outside [completion scope](docs/requirements.md#goal-and-constraints); completed code stays. Unobserved retail/audio/human judgment remains pending.
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

Use Rust 1.95 with the Windows MSVC toolchain and PowerShell 7. The [joined source check](docs/rust-review-fixes.md#combined-completion) passed 912 tests, with zero failures and 11 ignored, plus formatting, strict Clippy, compilation and an isolated debug build. That recorded check includes the guppy boundary correction and all eight review fixes; it did not run gameplay or refresh the normal-launch executable.

```powershell
$launcherPath = '.\scripts\Start-TurbofishDeluxe.ps1'
& $launcherPath
```

Normal launch resumes the current profile or creates one when `profiles.json` is absent. Use `-GameArguments @('--new-game', '--seed', '42')` only for an intentional reset of the active session; add a dedicated `--save-dir` for an isolated fresh run.

The launcher builds optimized release by default; the first build can take longer. Use `-Development` for an explicit debug build; existing `-Release` commands remain supported. It verifies the pinned general-purpose native audio package and stages its DLLs/notices beside the executable. `-PrepareOnly` stages without launching; `-Offline` uses the verified cache. This native package is required even with `--mute`. Details and license boundaries are in [audio design](docs/audio.md).

Steam discovery is automatic; pass `--game-dir <directory>` through `-GameArguments` if needed. Project saves use `%LOCALAPPDATA%/TurbofishDeluxe`, separate from retail saves; use `--save-dir` for isolated checks. Normal startup reads the current format3 `profiles.json` manager without importing `adventure.json`. Current standalone tooling uses format37. Old versions, retired migration tags and missing required state are rejected without rewriting their files. [Save scope](docs/requirements.md#goal-and-constraints) excludes development-save compatibility.

Escape pauses an active tank; top Menu opens the transient selector over ongoing modes. S saves; Q while paused or closing the window saves/exits. Continue is clickable (Enter also works); holding the hatch background skips its intro. F1 opens paged Help from a tank; selector Stories browses earned Challenge lore. Ordinary selection supports up to four unlocked pets, with seven Virtual Tank slots after the earned capacity purchase.

`--evidence-dir` writes identity/events/captures and optional complete state snapshots. Current snapshots include `state` (board or null), `phase`, `progress` and `session_tick`; board ticks reset on a fresh stage while session ticks remain monotonic. The event log includes both time scopes. These observation files are private, separate from authoritative project saves.

```powershell
$validatorPath = Join-Path $env:USERPROFILE '.codex\tools\Invoke-CodexPowerShell.ps1'
& $validatorPath -Path .\scripts\Test-Runtime.ps1 -Execute
```

The maintained runtime check preflights native files, runs workspace formatting/lint/tests, then builds and stages DLLs/notices. Invoke `Test-Runtime.ps1 -GameDirectory <owned-install-directory>` to include installed MO3 decoding; without that parameter it explicitly reports those checks skipped. `--inspect-assets` checks actual installed images/effects/fonts without a window;246 declared images,65 effects and15 bitmap fonts passed. Original save compatibility is not required.

### Focused runtime checks

Use `-Check` for necessary changed-behavior verification while editing. The default `All` remains available; use focused checks and broaden only for a failure or unresolved correctness risk. Do not repeat passing unaffected checks. The [current verification plan](docs/requirements.md#acceptance-and-working-style) assigns ordinary gameplay to the human and stops autonomous gameplay scenarios. Normal-speed evidence is still required for a playability claim.

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

### Screensaver host and package

Preserved tooling, outside [completion scope](docs/requirements.md#goal-and-constraints). No further screensaver work or acceptance is scheduled. From a prepared Windows build, standard saver switches use the project runtime and separate project preferences. Commands shown below; native execution remains not tested:

```powershell
.\target\release\turbofish-deluxe.exe /c --save-dir .scratch/saver/save
.\target\release\turbofish-deluxe.exe /s --save-dir .scratch/saver/save --evidence-dir .scratch/saver/evidence --mute --quit-after 30
$buildIdentityPath = Read-Host 'Associated host build receipt path'
.\scripts\New-ScreensaverPackage.ps1 -BuildIdentityPath $buildIdentityPath
```

The package command verifies associated build artifacts and places a byte-identical `.scr`, DLLs/notices and host license under ignored `target/screensaver/<build-id>`. It does not register or activate Windows settings. `/s` requires an existing current project profile and banks income plus narrow VT continuation on graceful exit; normal profile entry claims/adopts receipts once. Previews do not earn persistent shells. Current37/profile3/bank2 rejects older or incomplete records without migration/backfill. `/p <HWND>` supports a supplied preview parent and `/c <HWND>` uses project-window ownership; actual embedding/ownership/continuation/focus/dismissal/monitor/audio acceptance remains pending. [Screensaver](docs/screensaver.md) owns limitations.

After changing a PowerShell script, run the host validator first; its `-ArgumentList` forwards focused options:

```powershell
$validatorPath = Join-Path $env:USERPROFILE '.codex\tools\Invoke-CodexPowerShell.ps1'
& $validatorPath -Path .\scripts\Test-Runtime.ps1 -Execute -ArgumentList @('-Check', 'Format')
```

### Accelerated gameplay tests

Autonomous gameplay scenarios are stopped under the [current verification plan](docs/requirements.md#acceptance-and-working-style). These commands remain available for a later explicitly authorized targeted reproduction. Use `--test-speed <1..8>` for accelerated exploratory/stress runs. Default gameplay remains 1x. Above 1x requires a dedicated `--save-dir`, a separate `--evidence-dir`, `--mute` and a positive wall-clock `--quit-after` limit. The save format and game rules are unchanged. For example, from an already prepared build:

```powershell
.\target\release\turbofish-deluxe.exe --new-game --seed 42 --test-speed 8 --save-dir .scratch/speed-example/save --evidence-dir .scratch/speed-example/evidence --mute --quit-after 30
```

For an authorized targeted automated diagnostic, explicitly select 8x with `Start-TurbofishPlaytest -TestSpeed 8 -Mute` or `Test-Playtest.ps1 -TestSpeed 8 -Mute`, isolated saves/evidence, a wall deadline and owned-process cleanup. Use 4x only with a recorded input/polling reason. Explicit 1x is reserved for normal timing, render/audio or retail comparison with that purpose recorded. The [helper guide](docs/playtest-helpers.md) owns usage; its API retains 1x for older frozen callers. The visible banner and telemetry identify the factor. `elapsed_seconds` remains wall time; `session_elapsed_seconds` is fixed-step session time, including paused session updates. Global pause continues at 1x.

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

No game binaries or assets are distributed. AI assisted documentation and implementation; see [provenance](docs/provenance.md#tools-and-human-input). The guide-derived documentation retains its [MIT notice](THIRD-PARTY-NOTICES.md). The runtime-code license and distribution plan remain undecided.
