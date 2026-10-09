# Maintained native playtest helpers

Use [Playtest-Helpers.ps1](../scripts/Playtest-Helpers.ps1) for new experiments. Dot-source it from a scenario; keep machine-specific fixtures, scripts and raw receipts under ignored `.scratch/`. The old private tester scripts and completed reports remain historical. The October 9 inventory found 357 scripts: the original 355 plus the two newer A32 freeze/preparation scripts. Consolidation replaces repeated code in future scenarios, without rewriting those receipts.

[Test-Playtest.ps1](../scripts/Test-Playtest.ps1) is the maintained replacement for the repeated live pause/reload mechanics. It runs an identified Playing Board at normal speed, optionally checks a genuine copied save against pre-loop identity, posts Escape and a paused logical click, holds selected simulation state while session time advances, resumes, captures both views, closes, and compares the final session against the persisted save. `-ExerciseFoodClick` additionally checks an ordinary active food drop; use a starting Board with food capacity for that option. This smoke checks tooling; it does not earn a milestone or establish retail fidelity. [Verification report](playtests/2026-10-09-playtest-helpers-01.md).

## Run and validate

Use PowerShell 7 on Windows and an already built/staged project executable. The helper never builds, stages DLLs, launches the retail game, changes its installation, or reuses default project saves. Supply an independently checked SHA-256 from the build receipt. In a task-specific script validated through the host PowerShell wrapper:

```powershell
$repositoryRoot = 'C:\Code\TurbofishDeluxe'
$parameters = @{
    ExecutablePath = Join-Path $repositoryRoot 'target/debug/turbofish-deluxe.exe'
    ExecutableSha256 = '<64 hexadecimal characters from the checked build receipt>'
    Mute = $true
    ExerciseFoodClick = $true
}
& (Join-Path $repositoryRoot 'scripts/Test-Playtest.ps1') @parameters
```

The default run directory is a fresh `.scratch/playtests/helpers-<UUID>/`, containing separate `save/`, `evidence/`, launch/input/cleanup and result or failure receipts. `-RunDirectory` accepts a new directory inside this repository's `.scratch/`; existing directories and reparse-point parents are rejected. `-SourceSavePath` and `-SourceSaveSha256` must be supplied together. The source is byte-copied and verified, without schema conversion or gameplay edits. `-GameDirectory` overrides normal owned-install discovery. Argument arrays preserve spaces in all paths.

Validate the helper, scenario and regression script before execution:

```powershell
$validatorPath = Join-Path $env:USERPROFILE '.codex/tools/Invoke-CodexPowerShell.ps1'
& $validatorPath -Path ./scripts/Playtest-Helpers.ps1
& $validatorPath -Path ./scripts/Test-Playtest.ps1
& $validatorPath -Path ./scripts/Test-PlaytestHelpers.ps1 -Execute
& $validatorPath -Path ./scripts/Test-Workflow.ps1 -Execute
```

The host wrapper requires the already installed PSScriptAnalyzer. Validate the entire task-specific launcher before its first mutation, then use the wrapper's `-Execute`; keep structured scenario arguments in that script instead of forwarding a nested array across the wrapper's `pwsh -File` boundary. [Runtime checks](../README.md#workflow-checks) remain separate. Rust source is untouched by these helpers; their tests do not replace the runtime gate.

## Shared functions and contracts

| Function | Contract |
|---|---|
| `Start-TurbofishPlaytest` | SHA-checked `turbofish-deluxe.exe`; optional SHA-checked copied save; new private output; retained process object; bounded owned-window acquisition; startup failure cleanup. Always enforces isolated save/evidence arguments and a safety exit deadline. |
| `Send-TurbofishKey` | Escape/Enter/S/Q/F12 with explicit scan codes and down/up transition bits; bounded hold and release in `finally`. Escape is scan1, down65537/up3221291009. |
| `Send-TurbofishClick` | Logical640x480 coordinates; current client size/letterboxing sampled per click; window-local move/down/up, release in `finally`. |
| `Assert-TurbofishOwnedWindow` | Refresh returned process; reject exit/missing window; check HWND owner PID. Every posted message rechecks ownership and its native success result. No global input, foreground activation or title/name-based target selection. |
| `Read-TurbofishJson` / `Assert-TurbofishSnapshot` | File sharing permits atomic replacement; live keys are `elapsed_seconds`, `paused`, `session_tick`, `phase`, `progress`, **`state`**. Missing keys/schema errors are explicit; nullable Board is valid. |
| `Wait-TurbofishState` | Monotonic deadline, process liveness, retry only I/O/JSON read failures; optional exclusive `-AfterSessionTick` for freshness. Predicate must return one Boolean; exceptions surface immediately. Timeout names the expectation and last sample/read error. |
| `Request-TurbofishCapture` | Reject an already pending request; wait for consumption and a new nonempty frame, bounded by timeout/process liveness. A request file alone is not success. |
| `Compare-TurbofishJson` | Recursive case-sensitive keys, missing versus null, ordered arrays, exact numeric values without rounding, signed zero, bounded path diagnostics. Duplicate keys are rejected. |
| `Assert-TurbofishReload` | Require non-null objects with progress/board/phase/ticks/next_seed, then compare complete sessions using only the source-inventoried f32 path list. Report bit-equivalent decimal differences; IDs/RNG/f64 currency/actors remain exact. |
| `Assert-TurbofishPausedState` | Requires two paused snapshots with advancing session time and an explicit non-null `-SelectState`. Compares only the selected frozen simulation state, strictly. |
| `Stop-TurbofishPlaytest` | Close retained owned process, wait, kill only that process handle if needed, confirm exit and write a receipt. Idempotent; a forced exit is reported and fails the maintained smoke. Evidence is preserved. |

## Scenario rules

Always put owned process cleanup in `finally`. Use `Wait-TurbofishState -AfterSessionTick $before.session_tick` for an action's response; successful parsing alone does not prove fresh publication. Use a descriptive expectation and required properties under strict mode. Unit phases are strings, while payload phases are tagged objects; a Bonus Board is null and its simulation is `phase.Bonus.state`.

Compare source `session` against `identity.local.json` **`start`**, and `final.local.json` **`session`** against saved `session`. The first live sample can already have advanced. Live snapshots omit `next_seed`, so they cannot establish complete reload identity.

Pause freezes the chosen simulation, not every enclosing field. `AdventureSession::paused_step` advances session ticks, performs Board cooldown/input/invasion-food-delay housekeeping, and A32 paused rendering can observe Bilaterus connector flags. For Bonus, select `phase.Bonus.state`; for other scenarios, explicitly select the invariant being tested. The maintained smoke selects Board tick/fish/dead fish/food/coins/RNG/IDs. It does not claim that unselected actors or every Board field froze.

The reload f32 list covers fish movement fields, dead-fish movement/opacity, food movement and explicitly declared corpse opacity fields. Similar names in Breeders, pets, coins, aliens and Bilaterus are f64 and remain strict. When serialized types change, recheck [source declarations](../src/sim.rs) and the helper's `Get-TurbofishSinglePrecisionPaths`, update focused regressions, then rerun affected native checks. No epsilon or blanket float coercion is permitted.
