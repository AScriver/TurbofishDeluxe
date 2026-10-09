[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$ExecutablePath,
    [Parameter(Mandatory)][string]$ExecutableSha256,
    [string]$RunDirectory,
    [string]$SourceSavePath,
    [string]$SourceSaveSha256,
    [string]$GameDirectory,
    [switch]$Mute,
    [switch]$ExerciseFoodClick
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'Playtest-Helpers.ps1')

$launchParameters = @{ ExecutablePath = $ExecutablePath; ExecutableSha256 = $ExecutableSha256; QuitAfterSeconds = 45; Mute = $Mute }
foreach ($parameterName in @('RunDirectory', 'SourceSavePath', 'SourceSaveSha256', 'GameDirectory')) {
    if ($PSBoundParameters.ContainsKey($parameterName)) { $launchParameters[$parameterName] = $PSBoundParameters[$parameterName] }
}
$playtest = $null
try {
    $playtest = Start-TurbofishPlaytest @launchParameters
    $before = Wait-TurbofishState -Playtest $playtest -Description 'active Playing Board' -Predicate {
        param($sample) $sample.phase -ceq 'Playing' -and $null -ne $sample.state
    }
    $identityPath = Join-Path $playtest.EvidenceDirectory 'identity.local.json'
    $identity = Read-TurbofishJson $identityPath
    $reloadCheck = $null
    if ($SourceSavePath) {
        $source = Read-TurbofishJson $SourceSavePath
        $reloadCheck = Assert-TurbofishReload -ExpectedSessionJson ($source.session | ConvertTo-Json -Depth 100 -Compress) -ActualSessionJson ($identity.start | ConvertTo-Json -Depth 100 -Compress)
    }
    if ($ExerciseFoodClick) {
        $originalFoodCount = @($before.state.food).Count
        Send-TurbofishClick -Playtest $playtest -X 320 -Y 250 -Reason 'normal-speed food input'
        $before = Wait-TurbofishState -Playtest $playtest -AfterSessionTick $before.session_tick -Description 'ordinary food drop from logical click' -Predicate {
            param($sample) @($sample.state.food).Count -gt $originalFoodCount
        }
    }
    # Exercise logical coordinates only while paused: this click cannot feed/claim.
    Send-TurbofishKey -Playtest $playtest -Key Escape
    $paused = Wait-TurbofishState -Playtest $playtest -AfterSessionTick $before.session_tick -Description 'Escape pause' -Predicate { param($sample) $sample.paused }
    $pausedFrame = Request-TurbofishCapture -Playtest $playtest
    Send-TurbofishClick -Playtest $playtest -X 320 -Y 240 -Reason 'paused input routing'
    $held = Wait-TurbofishState -Playtest $playtest -AfterSessionTick ($paused.session_tick + 12) -Description 'pause hold with fresh session time' -Predicate { param($sample) $sample.paused }
    # Board time/actors/RNG are frozen; exclude documented pause housekeeping
    # (punch cooldown, invasion food delay, render-observed connector flags).
    $selectFrozenState = {
        param($sample)
        $projection = [ordered]@{}
        foreach ($key in @('tick', 'fish', 'dead_fish', 'food', 'coins', 'rng_state', 'next_id')) {
            if (-not $sample.state.Contains($key)) { throw "Pause projection missing state.$key" }
            $projection[$key] = $sample.state[$key]
        }
        return $projection
    }
    Assert-TurbofishPausedState -Before $paused -After $held -SelectState $selectFrozenState -Description 'Board clock/fish/food/coins/RNG/IDs'
    Send-TurbofishKey -Playtest $playtest -Key Escape
    $resumeTick = $held.state.tick
    $resumed = Wait-TurbofishState -Playtest $playtest -AfterSessionTick $held.session_tick -Description 'Escape resume and Board progress' -Predicate {
        param($sample) -not $sample.paused -and $sample.state.tick -ge $resumeTick + 5
    }
    $resumedFrame = Request-TurbofishCapture -Playtest $playtest
    $cleanup = Stop-TurbofishPlaytest -Playtest $playtest
    if ($cleanup.forced -or $cleanup.exit_code -ne 0) { throw 'Native scenario did not finish gracefully.' }
    $final = Read-TurbofishJson (Join-Path $playtest.EvidenceDirectory 'final.local.json')
    $saved = Read-TurbofishJson $playtest.SavePath
    $saveCheck = Assert-TurbofishReload -ExpectedSessionJson ($final.session | ConvertTo-Json -Depth 100 -Compress) -ActualSessionJson ($saved.session | ConvertTo-Json -Depth 100 -Compress)
    if (-not $final.game_unchanged) { throw 'Runtime installation-integrity receipt is false.' }
    [ordered]@{
        result = 'passed'; scope = 'helper native pause/reload smoke; gameplay acceptance remains separate'
        process_id = $playtest.Process.Id; executable_sha256 = $identity.rust_executable_sha256
        food_click_exercised = [bool]$ExerciseFoodClick
        initial_reload = $reloadCheck; final_save_comparison = $saveCheck
        paused = $paused; held = $held; resumed = $resumed; paused_frame = $pausedFrame; resumed_frame = $resumedFrame
        format_version = $saved.format_version; game_unchanged = $final.game_unchanged; cleanup = $cleanup
    } | ConvertTo-Json -Depth 100 | Set-Content -LiteralPath (Join-Path $playtest.RunDirectory 'result.local.json') -Encoding utf8NoBOM
    [pscustomobject]@{ Result = 'Passed'; RunDirectory = $playtest.RunDirectory; ProcessId = $playtest.Process.Id
        PauseSessionTicks = @($paused.session_tick, $held.session_tick); BoardTicks = @($paused.state.tick, $held.state.tick, $resumed.state.tick)
        TypedFinalSavePaths = $saveCheck.TypedEquivalentPaths.Count; GameUnchanged = $final.game_unchanged; Cleanup = $cleanup }
}
catch {
    if ($null -ne $playtest) {
        [ordered]@{ result = 'failed'; error = $_.Exception.Message; failed_utc = [DateTime]::UtcNow.ToString('o') } |
            ConvertTo-Json | Set-Content -LiteralPath (Join-Path $playtest.RunDirectory 'failure.local.json') -Encoding utf8NoBOM
    }
    throw
}
finally {
    if ($null -ne $playtest) { Stop-TurbofishPlaytest -Playtest $playtest | Out-Null }
}
