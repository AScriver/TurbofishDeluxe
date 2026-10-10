<#
.SYNOPSIS
Runs one leased, isolated current37 mode/save continuation scenario at normal speed.
.DESCRIPTION
Requires a completed immutable build, prepared-not-earned profile fixture and explicit
serialized native resource lease. No fixture generator, schema conversion or earned
progress is supplied. Requires the app-owned format1 ui-state.local.json sideband
and capture-result.local.json same-draw receipt. Title input uses observed ready Start
bounds; underlying Session phase cannot establish Title/Menu visibility. Native checks
leave captured-pixel and retail/human judgments for review.

Fixture receipt (schema_version 1, kind native-mode-continuation-fixture) must declare
save_origin prepared-not-earned, session_format 37, profile_format 3, fixture_directory,
source_description and files [{relative_path, sha256}]. Flat profiles.json is required;
settings.json and screensaver.json are optional. No bank or lock files are admitted.
Current profile must have active Adventure Playing and parked TimeTrialPlaying Boards,
TT admission, empty VT fish and no VT Board; root owns production-decoder validation.

BuildArtifactRoot resolves the existing build receipt's relative artifact paths; supply
the owning build repository root, not this driver worktree unless they are identical.
Lease receipt (schema_version 1, kind native-mode-continuation-lease) must declare status
granted, driver_sha256, executable_sha256, build_receipt_sha256, build_artifact_root,
fixture_receipt_sha256, run_directory and expires_utc. It authorizes two serialized project
processes and private output. A matching receipt is an authorization prerequisite, not
an interprocess lock; the root must reserve native resources for the whole deadline.

Invoke from a task-specific launcher statically validated and executed through the
host Invoke-CodexPowerShell.ps1 wrapper. Shared helper/CLI contracts remain authoritative.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$ExecutablePath,
    [Parameter(Mandatory)][ValidatePattern('^[a-fA-F0-9]{64}$')][string]$ExecutableSha256,
    [Parameter(Mandatory)][string]$BuildReceiptPath,
    [Parameter(Mandatory)][string]$BuildArtifactRoot,
    [Parameter(Mandatory)][ValidatePattern('^[a-fA-F0-9]{64}$')][string]$BuildReceiptSha256,
    [Parameter(Mandatory)][string]$FixtureDirectory,
    [Parameter(Mandatory)][string]$FixtureReceiptPath,
    [Parameter(Mandatory)][ValidatePattern('^[a-fA-F0-9]{64}$')][string]$FixtureReceiptSha256,
    [Parameter(Mandatory)][string]$NativeLeasePath,
    [Parameter(Mandatory)][ValidatePattern('^[a-fA-F0-9]{64}$')][string]$NativeLeaseSha256,
    [Parameter(Mandatory)][string]$RunDirectory,
    [string]$GameDirectory,
    [ValidateSet(1)][int]$TestSpeed = 1,
    [ValidateRange(60, 300)][int]$WallDeadlineSeconds = 120,
    [switch]$Mute
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'Playtest-Helpers.ps1')
. (Join-Path $PSScriptRoot 'Evidence.Common.ps1')
$scenarioTimer = [System.Diagnostics.Stopwatch]::StartNew()
$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$ownedProcesses = [System.Collections.Generic.List[object]]::new()
$checks = [System.Collections.Generic.List[object]]::new()
$checkpoints = [System.Collections.Generic.List[object]]::new()
$immutableInputs = [System.Collections.Generic.List[object]]::new()
$leaseExpires = [DateTimeOffset]::MaxValue
$runRoot = [System.IO.Path]::GetFullPath($RunDirectory)
$failureMessage = $null
$runAllocated = $false

function Assert-ContinuationPath {
    param([string]$Path, [switch]$Directory)
    if (-not [System.IO.Path]::IsPathFullyQualified($Path)) { throw "Path must be absolute: $Path" }
    $fullPath = [System.IO.Path]::GetFullPath($Path)
    $pathCursor = $fullPath
    while ($pathCursor) {
        if (Test-Path -LiteralPath $pathCursor) {
            if ((Get-Item -LiteralPath $pathCursor -Force).Attributes -band [System.IO.FileAttributes]::ReparsePoint) {
                throw "Reparse points are not admitted: $pathCursor"
            }
        }
        $pathCursor = Split-Path -Parent $pathCursor
    }
    if ($Directory -and -not (Test-Path -LiteralPath $fullPath -PathType Container)) { throw "Directory missing: $fullPath" }
    return $fullPath
}

function Get-ContinuationFile {
    param([string]$Path, [string]$Sha256)
    $fullPath = Assert-ContinuationPath -Path $Path
    if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf)) { throw "File missing: $fullPath" }
    $fileHash = (Get-FileHash -LiteralPath $fullPath -Algorithm SHA256).Hash
    if ($Sha256 -and $fileHash -ine $Sha256) { throw "Digest differs: $fullPath" }
    return [ordered]@{ path = $fullPath; sha256 = $fileHash; length = (Get-Item -LiteralPath $fullPath).Length }
}

function Get-ContinuationBudget {
    param([double]$Limit = 5)
    # Reserve cleanup time; all waits use monotonic scenario time, with lease expiry
    # as an additional wall-clock bound. No wait/input loop can renew its deadline.
    $remaining = [Math]::Min($WallDeadlineSeconds - $scenarioTimer.Elapsed.TotalSeconds - 12,
        ($leaseExpires - [DateTimeOffset]::UtcNow).TotalSeconds - 12)
    if ($remaining -lt 0.1) { throw 'Scenario deadline/lease cleanup reserve reached.' }
    return [Math]::Min($Limit, $remaining)
}

function Write-ContinuationJson {
    param([string]$Name, [object]$Value)
    $outputPath = Join-Path $runRoot $Name
    if (Test-Path -LiteralPath $outputPath) { throw "Evidence already exists: $outputPath" }
    $Value | ConvertTo-Json -Depth 100 | Set-Content -LiteralPath $outputPath -Encoding utf8NoBOM
    return $outputPath
}

function Get-ContinuationFloatPath {
    $floatPaths = @(Get-TurbofishSinglePrecisionPaths)
    foreach ($slot in @('retained_adventure', 'retained_time_trial', 'retained_challenge')) {
        $floatPaths += @(Get-TurbofishSinglePrecisionPaths | ForEach-Object { $_.Replace('$.board.', "`$.$slot.board.") })
    }
    return $floatPaths
}

function Assert-ContinuationEqual {
    param([string]$Description, [object]$Expected, [object]$Actual)
    $comparison = Compare-TurbofishJson -ExpectedJson ($Expected | ConvertTo-Json -Depth 100 -Compress) `
        -ActualJson ($Actual | ConvertTo-Json -Depth 100 -Compress) -SinglePrecisionPaths (Get-ContinuationFloatPath)
    if (-not $comparison.Equal) { throw "${Description}: $($comparison.Differences -join '; ')" }
    $checks.Add([ordered]@{ description = $Description; result = 'passed'; typed_equivalent_paths = $comparison.TypedEquivalentPaths })
}

function Get-ContinuationCurrent {
    param([System.Collections.IDictionary]$Manager)
    if ($Manager.format_version -ne 3 -or $null -eq $Manager.current_id) { throw 'Require current profile3 manager.' }
    $currentEntries = @($Manager.entries | Where-Object { $_.id -eq $Manager.current_id })
    if ($currentEntries.Count -ne 1) { throw 'Current profile must resolve exactly once.' }
    return $currentEntries[0]
}

function Get-ContinuationScene {
    param([System.Collections.IDictionary]$Session)
    return [ordered]@{ phase = $Session.phase; board = $Session.board; next_seed = $Session.next_seed
        time_trial = $Session.time_trial; challenge = $Session.challenge; selected_pets = $Session.progress.selected_pets }
}

function Get-ContinuationShared {
    param([System.Collections.IDictionary]$Manager)
    $current = Get-ContinuationCurrent $Manager
    $sharedSession = [ordered]@{}
    foreach ($sessionKey in $current.session.Keys) {
        if ($sessionKey -cnotin @('mode', 'phase', 'board', 'next_seed', 'time_trial', 'challenge', 'ticks',
            'retained_adventure', 'retained_time_trial', 'retained_challenge', 'progress', 'virtual_saved_at_seconds')) {
            $sharedSession[$sessionKey] = $current.session[$sessionKey]
        }
    }
    $progress = [ordered]@{}
    foreach ($progressKey in $current.session.progress.Keys) {
        if ($progressKey -cne 'selected_pets') { $progress[$progressKey] = $current.session.progress[$progressKey] }
    }
    $managerProjection = [ordered]@{}
    foreach ($managerKey in $Manager.Keys) {
        if ($managerKey -cne 'entries') { $managerProjection[$managerKey] = $Manager[$managerKey] }
    }
    $entryProjection = @($Manager.entries | ForEach-Object {
        $entry = $_
        $projection = [ordered]@{}
        foreach ($entryKey in $entry.Keys) {
            if ($entry.id -ne $Manager.current_id -or $entryKey -cne 'session') { $projection[$entryKey] = $entry[$entryKey] }
        }
        $projection
    })
    return [ordered]@{ manager = $managerProjection; entries = $entryProjection; progress = $progress; session = $sharedSession }
}

function Get-ContinuationSaveToken {
    if (-not (Test-Path -LiteralPath $savePath -PathType Leaf)) { return 'missing' }
    return ('{0}:{1}' -f [System.IO.File]::GetLastWriteTimeUtc($savePath).Ticks, (Get-FileHash -LiteralPath $savePath).Hash)
}

function Wait-ContinuationSave {
    param($Playtest, [string]$PreviousToken, [string]$Mode, [string]$Phase)
    $waitTimer = [System.Diagnostics.Stopwatch]::StartNew()
    $waitLimit = Get-ContinuationBudget
    $lastReadError = $null
    do {
        $Playtest.Process.Refresh()
        if ($Playtest.Process.HasExited) { throw 'Process exited before requested profile publication.' }
        try {
            $beforeReadToken = Get-ContinuationSaveToken
            $manager = Read-TurbofishJson $savePath
            $token = Get-ContinuationSaveToken
            $current = Get-ContinuationCurrent $manager
            if ($token -ceq $beforeReadToken -and $token -cne $PreviousToken -and $current.session.mode -ceq $Mode -and $current.session.phase -ceq $Phase) {
                Assert-ContinuationEqual 'shared progress/wallet/scores/profile accounting conserved' $sharedBaseline (Get-ContinuationShared $manager)
                return $manager
            }
        }
        catch [System.IO.IOException] { $lastReadError = $_.Exception.Message }
        catch [System.ArgumentException] { $lastReadError = $_.Exception.Message }
        Start-Sleep -Milliseconds 25
    } while ($waitTimer.Elapsed.TotalSeconds -lt $waitLimit)
    throw "Timed out awaiting fresh profile publication for $Mode/$Phase; last read error: $lastReadError"
}

function Wait-ContinuationState {
    param($Playtest, [long]$AfterTick, [string]$Description, [scriptblock]$Predicate)
    return Wait-TurbofishState -Playtest $Playtest -AfterSessionTick $AfterTick `
        -Description $Description -Predicate $Predicate -TimeoutSeconds (Get-ContinuationBudget)
}

function Assert-ContinuationUi {
    param($Playtest, [System.Collections.IDictionary]$Observation)
    foreach ($field in @('format_version', 'process_id', 'ui_frame', 'wall_elapsed_seconds', 'ui_stage',
        'title', 'session_tick', 'session_phase', 'coordinate_space')) {
        if (-not $Observation.Contains($field)) { throw "UI observation missing $field; supporting format1 build required." }
    }
    if ($Observation.format_version -ne 1 -or $Observation.process_id -ne $Playtest.Process.Id -or
        $Observation.ui_frame -isnot [long] -or $Observation.ui_frame -lt 1 -or
        $Observation.ui_stage -cnotin @('Title', 'Menu', 'Scene', 'ScreensaverPreview') -or
        $Observation.coordinate_space.width -ne 640 -or $Observation.coordinate_space.height -ne 480 -or
        -not [double]::IsFinite($Observation.wall_elapsed_seconds) -or $Observation.wall_elapsed_seconds -lt 0) {
        throw 'UI observation differs in schema/owned PID/frame/coordinate/timing identity.'
    }
    if ($Observation.ui_stage -ceq 'Title') {
        if ($null -ne $Observation.session_tick -or $null -ne $Observation.session_phase -or $null -eq $Observation.title) {
            throw 'Title observation must have Title payload and null Session identity.'
        }
        foreach ($field in @('loading_status', 'completed_tasks', 'total_tasks', 'ready', 'start_visible', 'start_bounds', 'start_hovered', 'start_pressed')) {
            if (-not $Observation.title.Contains($field)) { throw "Title observation missing $field." }
        }
        if ($Observation.title.ready -isnot [bool] -or $Observation.title.start_visible -isnot [bool] -or
            $Observation.title.loading_status -cnotin @('Running', 'Succeeded', 'Failed')) { throw 'Invalid Title readiness/status types.' }
        if ($Observation.title.loading_status -ceq 'Failed') { throw 'Observed Title loading failure; no input is admitted.' }
    } elseif ($null -ne $Observation.title -or $null -eq $Observation.session_tick -or $null -eq $Observation.session_phase) {
        throw 'This profile scenario requires non-Title UI observations with retained Session identity.'
    }
}

function Wait-ContinuationUi {
    param($Playtest, [string]$Stage, [long]$AfterFrame = -1, [double]$Limit = 5, [switch]$ReadyTitle)
    $waitTimer = [System.Diagnostics.Stopwatch]::StartNew()
    $waitLimit = Get-ContinuationBudget -Limit $Limit
    $lastReadError = $null
    do {
        $Playtest.Process.Refresh()
        if ($Playtest.Process.HasExited) { throw "Process exited awaiting actual $Stage UI draw." }
        $observation = $null
        try { $observation = Read-TurbofishJson $Playtest.UiStatePath }
        catch [System.IO.IOException] { $lastReadError = $_.Exception.Message }
        catch [System.ArgumentException] { $lastReadError = $_.Exception.Message }
        if ($null -ne $observation) {
            Assert-ContinuationUi $Playtest $observation
            $titleAdmitted = -not $ReadyTitle -or ($observation.ui_stage -ceq 'Title' -and
                $observation.title.start_visible -and $observation.title.ready -and $observation.title.loading_status -ceq 'Succeeded')
            if ($observation.ui_stage -ceq $Stage -and $observation.ui_frame -gt $AfterFrame -and $titleAdmitted) {
                return $observation
            }
        }
        Start-Sleep -Milliseconds 25
    } while ($waitTimer.Elapsed.TotalSeconds -lt $waitLimit)
    throw "Timed out awaiting fresh owned-PID $Stage UI frame after $AfterFrame; format1 UI build required; last read error: $lastReadError"
}

function Wait-ContinuationScene {
    param($Playtest, [long]$AfterMenuFrame, [string]$Phase)
    $observation = Wait-ContinuationUi -Playtest $Playtest -Stage Scene -AfterFrame $AfterMenuFrame
    if ($observation.session_phase -cne $Phase) { throw "Actual Scene draw is not expected phase $Phase." }
    $checks.Add([ordered]@{ description = 'fresh owned-PID Menu-to-Scene transition'; result = 'passed'
        after_menu_ui_frame = $AfterMenuFrame; scene_ui_frame = $observation.ui_frame; session_phase = $observation.session_phase })
    return $observation
}

function Save-ContinuationCheckpoint {
    param($Playtest, [string]$Name, [ValidateSet('Title', 'Menu', 'Scene')][string]$Stage = 'Menu', [long]$AfterUiFrame = -1)
    $beforeUi = Wait-ContinuationUi -Playtest $Playtest -Stage $Stage -AfterFrame $AfterUiFrame
    $framePath = Request-TurbofishCapture -Playtest $Playtest -TimeoutSeconds (Get-ContinuationBudget -Limit 3)
    $captureReceiptPath = Join-Path $Playtest.EvidenceDirectory 'capture-result.local.json'
    $captureReceipt = Read-TurbofishJson $captureReceiptPath
    $observation = $captureReceipt.observation
    Assert-ContinuationUi $Playtest $observation
    $expectedFrameName = 'frame-ui-{0}-{1:000000}.png' -f $Playtest.Process.Id, $observation.ui_frame
    if ($captureReceipt.frame -cne [System.IO.Path]::GetFileName($framePath) -or $captureReceipt.frame -cne $expectedFrameName -or
        $observation.ui_stage -cne $Stage -or $observation.ui_frame -le $beforeUi.ui_frame) {
        throw 'Capture is not the requested fresh owned-PID stage with a matching same-draw receipt.'
    }
    $sample = $null
    $eventRecord = $null
    if ($Stage -cne 'Title') {
        $sample = Read-TurbofishJson $Playtest.StatePath
        Assert-TurbofishSnapshot $sample
        if ($Stage -ceq 'Menu' -and ($sample.session_tick -ne $observation.session_tick -or
            ($sample.phase | ConvertTo-Json -Compress -Depth 100) -cne ($observation.session_phase | ConvertTo-Json -Compress -Depth 100))) {
            throw 'Frozen Menu live Session identity differs from its actual captured draw.'
        }
        $sample['ui_observation'] = $observation
        $eventRecord = Get-ContinuationFile (Join-Path $Playtest.EvidenceDirectory 'events.local.jsonl')
    }
    $retainedFrame = Join-Path $captureRoot ($Name + '.png')
    if (Test-Path -LiteralPath $retainedFrame) { throw 'Checkpoint frame already exists.' }
    $frameRecord = Get-ContinuationFile $framePath
    # Unique UI-frame native captures stay in evidence for standard manifests.
    # A checked checkpoint copy and receipt preserve exact selected pixels/binding.
    Copy-Item -LiteralPath $framePath -Destination $retainedFrame -ErrorAction Stop
    $retainedRecord = Get-ContinuationFile $retainedFrame $frameRecord.sha256
    $checkpoint = [ordered]@{ name = $Name; process_id = $Playtest.Process.Id; captured_utc = [DateTimeOffset]::UtcNow.ToString('o')
        frame = $retainedRecord; native_frame = $frameRecord; capture_receipt = $captureReceipt; sample = $sample
        events_prefix = $eventRecord; inputs_path = $Playtest.InputLogPath
        ui_freshness = 'owned PID, increasing ui_frame, explicit stage and exact PNG/same-draw observation binding'
        visual_verdict = 'not reviewed; UI observation/capture files alone do not establish pixel fidelity' }
    $checkpointPath = Write-ContinuationJson -Name ($Name + '.local.json') -Value $checkpoint
    $checkpoints.Add([ordered]@{ name = $Name; path = $checkpointPath; frame = $retainedRecord })
    if ($Stage -ceq 'Title') { return $observation }
    return $sample
}

function Invoke-ContinuationProcess {
    param([int]$Index)
    foreach ($inputRecord in $immutableInputs) { $null = Get-ContinuationFile $inputRecord.path $inputRecord.sha256 }
    Initialize-TurbofishPlaytestNative
    $processRoot = Join-Path $runRoot ('process-' + $Index)
    $evidenceRoot = Join-Path $processRoot 'evidence'
    New-Item -ItemType Directory -Path $evidenceRoot -ErrorAction Stop | Out-Null
    # Shared Start-TurbofishPlaytest only copies adventure.json and creates its
    # own save directory. This narrow launcher permits unchanged profile-directory
    # fixtures and actual reopen of the same game-written profiles.json.
    $nativeArguments = @('--seed', '42', '--save-dir', $saveRoot, '--evidence-dir', $evidenceRoot,
        '--quit-after', ([Math]::Floor((Get-ContinuationBudget -Limit 300))).ToString(), '--test-speed', $TestSpeed.ToString())
    if ($GameDirectory) { $nativeArguments += @('--game-dir', $resolvedGameDirectory) }
    if ($Mute) { $nativeArguments += '--mute' }
    $startInfo = [System.Diagnostics.ProcessStartInfo]::new($executableRecord.path)
    $startInfo.UseShellExecute = $false
    $startInfo.WorkingDirectory = $repositoryRoot
    foreach ($nativeArgument in $nativeArguments) { $startInfo.ArgumentList.Add($nativeArgument) }
    $retainedProcess = [System.Diagnostics.Process]::new()
    $retainedProcess.StartInfo = $startInfo
    if (-not $retainedProcess.Start()) { throw 'Owned project process did not start.' }
    $playtest = [pscustomobject]@{ Process = $retainedProcess; RunDirectory = $processRoot; EvidenceDirectory = $evidenceRoot
        SavePath = $savePath; StatePath = (Join-Path $evidenceRoot 'state.local.json')
        InputLogPath = (Join-Path $processRoot 'inputs.local.jsonl'); UiStatePath = (Join-Path $evidenceRoot 'ui-state.local.json')
        Cleanup = $null; TestSpeed = $TestSpeed }
    $ownedProcesses.Add($playtest)
    [ordered]@{ process_id = $retainedProcess.Id; arguments = $nativeArguments; executable = $executableRecord
        build_receipt = $buildRecord; fixture_receipt = $fixtureRecord; native_lease = $leaseRecord
        test_speed = 1; time_mode = 'normal'; purpose = 'normal1x native UI/save timing and scene conservation'
        save_origin = 'prepared-not-earned'; started_utc = [DateTimeOffset]::UtcNow.ToString('o') } |
        ConvertTo-Json -Depth 20 | Set-Content -LiteralPath (Join-Path $processRoot 'launch.local.json') -Encoding utf8NoBOM
    $windowTimer = [System.Diagnostics.Stopwatch]::StartNew()
    $windowLimit = Get-ContinuationBudget -Limit 10
    do {
        $retainedProcess.Refresh()
        if ($retainedProcess.HasExited) { throw "Process exited during startup ($($retainedProcess.ExitCode))." }
        if ($retainedProcess.MainWindowHandle -ne [IntPtr]::Zero) {
            Assert-TurbofishOwnedWindow -Playtest $playtest | Out-Null
            return $playtest
        }
        Start-Sleep -Milliseconds 25
    } while ($windowTimer.Elapsed.TotalSeconds -lt $windowLimit)
    throw 'Owned project window did not appear within bounded startup wait.'
}

function Open-ContinuationStartup {
    param($Playtest, [string]$Name)
    $identityPath = Join-Path $Playtest.EvidenceDirectory 'identity.local.json'
    $readyTitle = Wait-ContinuationUi -Playtest $Playtest -Stage Title -ReadyTitle -Limit 30
    $capturedTitle = Save-ContinuationCheckpoint -Playtest $Playtest -Name ($Name + '-title') -Stage Title -AfterUiFrame $readyTitle.ui_frame
    if (-not $capturedTitle.title.start_visible -or -not $capturedTitle.title.ready -or $capturedTitle.title.loading_status -cne 'Succeeded') {
        throw 'Captured Title is not ready for Start input.'
    }
    $startBounds = $capturedTitle.title.start_bounds
    foreach ($field in @('x', 'y', 'width', 'height')) { if (-not $startBounds.Contains($field)) { throw "Observed Start bounds missing $field." } }
    if ($startBounds.width -le 0 -or $startBounds.height -le 0 -or $startBounds.x -lt 0 -or $startBounds.y -lt 0 -or
        $startBounds.x + $startBounds.width -gt 640 -or $startBounds.y + $startBounds.height -gt 480) { throw 'Observed Start rectangle is outside logical viewport.' }
    $startX = $startBounds.x + $startBounds.width / 2.0
    $startY = $startBounds.y + $startBounds.height / 2.0
    Send-TurbofishClick -Playtest $Playtest -X $startX -Y $startY -Reason 'ready Title Start press/release in observed same-draw bounds'
    $menuUi = Wait-ContinuationUi -Playtest $Playtest -Stage Menu -AfterFrame $capturedTitle.ui_frame
    $identity = Read-TurbofishJson $identityPath
    if ($identity.rust_executable_sha256 -ine $ExecutableSha256 -or $identity.test_speed -ne 1 -or $identity.time_mode -cne 'normal') {
        throw 'Runtime identity does not match leased normal1x executable.'
    }
    $null = Wait-TurbofishState -Playtest $Playtest -Description 'complete main-loop snapshot after Title route' `
        -Predicate { param($sample) $sample.phase -ceq 'Playing' -and $null -ne $sample.state } -TimeoutSeconds (Get-ContinuationBudget)
    $null = Save-ContinuationCheckpoint -Playtest $Playtest -Name $Name -Stage Menu -AfterUiFrame $menuUi.ui_frame
    $checks.Add([ordered]@{ description = 'ready Title input and fresh owned-PID Title-to-Menu draw transition'; result = 'passed'
        title_ui_frame = $capturedTitle.ui_frame; menu_ui_frame = $menuUi.ui_frame; observed_start_bounds = $startBounds })
    return $identity
}

function Suspend-ContinuationBoard {
    param($Playtest, $Before, [string]$Phase)
    Send-TurbofishKey -Playtest $Playtest -Key Escape
    $pausedSample = Wait-ContinuationState -Playtest $Playtest -AfterTick $Before.session_tick -Description 'fresh Escape pause' -Predicate {
        param($sample) $sample.paused -and $sample.phase -ceq $Phase -and $null -ne $sample.state
    }
    $heldSample = Wait-ContinuationState -Playtest $Playtest -AfterTick ($pausedSample.session_tick + 6) -Description 'paused housekeeping with frozen simulation' -Predicate {
        param($sample) $sample.paused -and $sample.phase -ceq $Phase
    }
    Assert-TurbofishPausedState -Before $pausedSample -After $heldSample -Description 'Board tick/fish/food/coins/RNG/IDs' -SelectState {
        param($sample)
        $frozen = [ordered]@{}
        foreach ($stateKey in @('tick', 'fish', 'dead_fish', 'food', 'coins', 'rng_state', 'next_id')) {
            $frozen[$stateKey] = $sample.state[$stateKey]
        }
        return $frozen
    }
    $checks.Add([ordered]@{ description = "fresh $Phase pause/resume boundary"; result = 'passed'
        paused_session_ticks = @($pausedSample.session_tick, $heldSample.session_tick); frozen_board_tick = $heldSample.state.tick })
    return $heldSample
}

function Open-ContinuationMenu {
    param($Playtest, [string]$Name)
    $sceneUi = Wait-ContinuationUi -Playtest $Playtest -Stage Scene
    Send-TurbofishClick -Playtest $Playtest -X 575 -Y 17 -Reason 'open actual transient tank Menu'
    $menuUi = Wait-ContinuationUi -Playtest $Playtest -Stage Menu -AfterFrame $sceneUi.ui_frame
    $firstSample = Save-ContinuationCheckpoint -Playtest $Playtest -Name ($Name + '-open') -Stage Menu -AfterUiFrame $menuUi.ui_frame
    $secondSample = Save-ContinuationCheckpoint -Playtest $Playtest -Name ($Name + '-hold') -Stage Menu -AfterUiFrame $firstSample.ui_observation.ui_frame
    Assert-ContinuationEqual 'transient Menu freezes actual phase/tick/progress/Board between consumed frames' `
        ([ordered]@{ tick = $firstSample.session_tick; phase = $firstSample.phase; progress = $firstSample.progress; board = $firstSample.state; paused = $firstSample.paused }) `
        ([ordered]@{ tick = $secondSample.session_tick; phase = $secondSample.phase; progress = $secondSample.progress; board = $secondSample.state; paused = $secondSample.paused })
    return $secondSample
}

function Save-ContinuationPaused {
    param($Playtest, $PausedSample, [string]$Mode, [string]$Phase)
    $previousToken = Get-ContinuationSaveToken
    Send-TurbofishKey -Playtest $Playtest -Key S
    $manager = Wait-ContinuationSave -Playtest $Playtest -PreviousToken $previousToken -Mode $Mode -Phase $Phase
    $current = Get-ContinuationCurrent $manager
    if ($current.session.ticks -lt $PausedSample.session_tick -or $current.session.board.tick -ne $PausedSample.state.tick) {
        throw 'Paused save is stale or evolved the Board clock.'
    }
    return $manager
}

function Close-ContinuationThroughMenu {
    param($Playtest, $MenuSample)
    Send-TurbofishClick -Playtest $Playtest -X 558 -Y 425 -Reason 'selector Quit dialog'
    $null = Save-ContinuationCheckpoint -Playtest $Playtest -Name ('process-' + $ownedProcesses.Count + '-quit-dialog')
    Send-TurbofishClick -Playtest $Playtest -X 215 -Y 289 -Reason 'confirm ordinary Quit and save'
    $exitTimer = [System.Diagnostics.Stopwatch]::StartNew()
    $exitLimit = Get-ContinuationBudget -Limit 10
    do {
        $Playtest.Process.Refresh()
        if ($Playtest.Process.HasExited) { break }
        Start-Sleep -Milliseconds 25
    } while ($exitTimer.Elapsed.TotalSeconds -lt $exitLimit)
    if (-not $Playtest.Process.HasExited) { throw 'Menu Quit did not exit; retained-PID finally cleanup will run.' }
    $cleanup = Stop-TurbofishPlaytest -Playtest $Playtest -GraceMilliseconds 1000
    if ($cleanup.forced -or $cleanup.exit_code -ne 0) { throw 'Scenario process did not exit gracefully.' }
    $final = Read-TurbofishJson (Join-Path $Playtest.EvidenceDirectory 'final.local.json')
    $manager = Read-TurbofishJson $savePath
    $current = Get-ContinuationCurrent $manager
    Assert-ContinuationEqual 'complete final session equals game-written profile session' $final.session $current.session
    Assert-ContinuationEqual 'shared accounting unchanged on save/exit' $sharedBaseline (Get-ContinuationShared $manager)
    if ($final.session.ticks -ne $MenuSample.session_tick -or -not $final.game_unchanged -or $final.test_speed -ne 1 -or $final.time_mode -cne 'normal') {
        throw 'Final menu tick/timing/install-integrity receipt differs.'
    }
    return $manager
}

function Assert-ContinuationReopen {
    param($SavedSession, $StartedSession)
    $savedProjection = [ordered]@{}
    $startedProjection = [ordered]@{}
    foreach ($sessionKey in $SavedSession.Keys) { if ($sessionKey -cne 'virtual_saved_at_seconds') { $savedProjection[$sessionKey] = $SavedSession[$sessionKey] } }
    foreach ($sessionKey in $StartedSession.Keys) { if ($sessionKey -cne 'virtual_saved_at_seconds') { $startedProjection[$sessionKey] = $StartedSession[$sessionKey] } }
    # Only the source-documented app wall timestamp changes on an empty VT reopen;
    # no other scene/accounting field or floating-point tolerance is excluded.
    Assert-ContinuationEqual 'complete current37 reopen except documented VT wall timestamp' $savedProjection $startedProjection
    $nowSeconds = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
    if ($StartedSession.virtual_saved_at_seconds -lt $SavedSession.virtual_saved_at_seconds -or
        $StartedSession.virtual_saved_at_seconds -gt $nowSeconds + 2) { throw 'Unexpected reopen wall timestamp.' }
    $checks.Add([ordered]@{ description = 'documented virtual_saved_at_seconds refresh'; before = $SavedSession.virtual_saved_at_seconds
        after = $StartedSession.virtual_saved_at_seconds; authority = 'AdventureSession::resume_virtual_from_save' })
}

try {
    if (-not $IsWindows -or $TestSpeed -ne 1) { throw 'Native mode continuation requires Windows and explicit normal1x timing.' }
    $runRoot = Assert-ContinuationPath $runRoot
    $scratchRoot = Join-Path $repositoryRoot '.scratch'
    if (-not $runRoot.StartsWith($scratchRoot + [System.IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase) -or
        (Test-Path -LiteralPath $runRoot)) { throw 'RunDirectory must be a new directory in this worktree .scratch.' }
    $executableRecord = Get-ContinuationFile $ExecutablePath $ExecutableSha256
    if ([System.IO.Path]::GetFileName($executableRecord.path) -cne 'turbofish-deluxe.exe') { throw 'Only the project executable is admitted.' }
    $buildRecord = Get-ContinuationFile $BuildReceiptPath $BuildReceiptSha256
    $buildArtifactDirectory = Assert-ContinuationPath $BuildArtifactRoot -Directory
    $fixtureRecord = Get-ContinuationFile $FixtureReceiptPath $FixtureReceiptSha256
    $leaseRecord = Get-ContinuationFile $NativeLeasePath $NativeLeaseSha256
    foreach ($inputRecord in @($executableRecord, $buildRecord, $fixtureRecord, $leaseRecord)) { $immutableInputs.Add($inputRecord) }
    $buildReceipt = Read-TurbofishJson $buildRecord.path
    if ($buildReceipt.schema_version -ne 1 -or $buildReceipt.kind -cne 'build' -or
        $buildReceipt.association -cne 'launcher-success-with-unchanged-inputs') { throw 'Require a completed associated build receipt.' }
    $executableArtifacts = @($buildReceipt.artifacts | Where-Object { $_.role -ceq 'executable' -and $_.sha256 -ieq $ExecutableSha256 })
    if ($executableArtifacts.Count -ne 1 -or (Get-EvidenceFullPath -Path $executableArtifacts[0].path -RepositoryRoot $buildArtifactDirectory) -ine $executableRecord.path) {
        throw 'Frozen executable is not uniquely bound by build receipt.'
    }
    $buildChecks = @(Get-EvidenceChecks -Record $buildReceipt -RepositoryRoot $buildArtifactDirectory)
    if (@($buildChecks | Where-Object { $_.status -notin @('match', 'not-captured') }).Count -ne 0) { throw 'Frozen build artifact readback failed.' }
    foreach ($artifact in $buildReceipt.artifacts) {
        if ($artifact.exists) {
            $artifactPath = Get-EvidenceFullPath -Path $artifact.path -RepositoryRoot $buildArtifactDirectory
            $immutableInputs.Add((Get-ContinuationFile $artifactPath $artifact.sha256))
        }
    }
    $fixtureRoot = Assert-ContinuationPath $FixtureDirectory -Directory
    if ($runRoot.StartsWith($fixtureRoot + '\', [StringComparison]::OrdinalIgnoreCase) -or
        $runRoot.StartsWith([System.IO.Path]::GetDirectoryName($executableRecord.path) + '\', [StringComparison]::OrdinalIgnoreCase)) {
        throw 'RunDirectory overlaps immutable fixture or frozen executable directory.'
    }
    $fixtureReceipt = Read-TurbofishJson $fixtureRecord.path
    if ($fixtureReceipt.schema_version -ne 1 -or $fixtureReceipt.kind -cne 'native-mode-continuation-fixture' -or
        $fixtureReceipt.save_origin -cne 'prepared-not-earned' -or $fixtureReceipt.session_format -ne 37 -or $fixtureReceipt.profile_format -ne 3 -or
        [System.IO.Path]::GetFullPath($fixtureReceipt.fixture_directory) -ine $fixtureRoot -or [string]::IsNullOrWhiteSpace($fixtureReceipt.source_description)) {
        throw 'Fixture receipt must identify the prepared current37/profile3 directory and its origin.'
    }
    $fixtureFiles = @($fixtureReceipt.files)
    if ($fixtureFiles.Count -lt 1 -or $fixtureFiles.Count -gt 3) { throw 'Fixture must contain only the declared profile/preferences files.' }
    $fixtureNames = @($fixtureFiles | ForEach-Object { $_.relative_path })
    if ('profiles.json' -cnotin $fixtureNames -or @($fixtureNames | Sort-Object -Unique -CaseSensitive).Count -ne $fixtureNames.Count) { throw 'Fixture needs unique profiles.json.' }
    foreach ($fixtureFile in $fixtureFiles) {
        if ($fixtureFile.relative_path -cnotin @('profiles.json', 'settings.json', 'screensaver.json') -or $fixtureFile.sha256 -cnotmatch '^[a-fA-F0-9]{64}$') {
            throw 'Unsupported fixture file/path/hash; bank, legacy and lock files are not admitted.'
        }
        $fixtureInput = Get-ContinuationFile (Join-Path $fixtureRoot $fixtureFile.relative_path) $fixtureFile.sha256
        if ($fixtureInput.length -gt 16777216) { throw 'Fixture file exceeds bounded 16 MiB input.' }
        $immutableInputs.Add($fixtureInput)
    }
    $actualFixtureNames = @(Get-ChildItem -LiteralPath $fixtureRoot -Force | ForEach-Object { $_.Name })
    Assert-ContinuationEqual 'fixture membership matches receipt' (@($fixtureNames | Sort-Object -CaseSensitive)) (@($actualFixtureNames | Sort-Object -CaseSensitive))
    $sourceProfilePath = Join-Path $fixtureRoot 'profiles.json'
    $sourceJson = [System.IO.File]::ReadAllText($sourceProfilePath)
    $null = Compare-TurbofishJson -ExpectedJson $sourceJson -ActualJson $sourceJson
    $sourceManager = Read-TurbofishJson $sourceProfilePath
    $sourceCurrent = Get-ContinuationCurrent $sourceManager
    $sourceSession = $sourceCurrent.session
    if ($sourceSession.mode -cne 'Adventure' -or $sourceSession.phase -cne 'Playing' -or $null -eq $sourceSession.board -or
        $null -ne $sourceSession.retained_adventure -or $null -ne $sourceSession.retained_challenge -or
        $null -eq $sourceSession.retained_time_trial -or $sourceSession.retained_time_trial.phase -cne 'TimeTrialPlaying' -or
        $null -eq $sourceSession.retained_time_trial.board -or $null -eq $sourceSession.retained_time_trial.time_trial -or
        ($sourceSession.progress.tank -lt 2 -and -not $sourceSession.progress.adventure_completed) -or
        @($sourceSession.virtual_tank.fish).Count -ne 0 -or $null -ne $sourceSession.virtual_board) {
        throw 'Fixture needs admitted Adventure/retained TT Playing scenes, no other retained scene, and empty VT.'
    }
    $sharedBaseline = Get-ContinuationShared $sourceManager
    $lease = Read-TurbofishJson $leaseRecord.path
    $driverSha256 = (Get-FileHash -LiteralPath $PSCommandPath).Hash
    if ($lease.schema_version -ne 1 -or $lease.kind -cne 'native-mode-continuation-lease' -or $lease.status -cne 'granted' -or
        $lease.driver_sha256 -ine $driverSha256 -or $lease.executable_sha256 -ine $ExecutableSha256 -or
        $lease.build_receipt_sha256 -ine $BuildReceiptSha256 -or $lease.fixture_receipt_sha256 -ine $FixtureReceiptSha256 -or
        [System.IO.Path]::GetFullPath($lease.build_artifact_root) -ine $buildArtifactDirectory -or
        [System.IO.Path]::GetFullPath($lease.run_directory) -ine $runRoot) { throw 'Explicit resource lease does not bind this driver/build/fixture/destination.' }
    $leaseExpires = [DateTimeOffset]::Parse($lease.expires_utc, [Globalization.CultureInfo]::InvariantCulture)
    if (($leaseExpires - [DateTimeOffset]::UtcNow).TotalSeconds -lt $WallDeadlineSeconds) { throw 'Resource lease does not cover full scenario/cleanup deadline.' }
    $resolvedGameDirectory = if ($GameDirectory) { Assert-ContinuationPath $GameDirectory -Directory } else { $null }
    if ($GameDirectory -and $runRoot.StartsWith($resolvedGameDirectory + '\', [StringComparison]::OrdinalIgnoreCase)) { throw 'Output overlaps owned installation.' }
    $saveRoot = Join-Path $runRoot 'save'
    $savePath = Join-Path $saveRoot 'profiles.json'
    $captureRoot = Join-Path $runRoot 'captures'
    New-Item -ItemType Directory -Path $saveRoot, $captureRoot -ErrorAction Stop | Out-Null
    $runAllocated = $true
    foreach ($fixtureFile in $fixtureFiles) {
        $destination = Join-Path $saveRoot $fixtureFile.relative_path
        Copy-Item -LiteralPath (Join-Path $fixtureRoot $fixtureFile.relative_path) -Destination $destination -ErrorAction Stop
        $null = Get-ContinuationFile $destination $fixtureFile.sha256
    }
    $null = Write-ContinuationJson 'preflight.local.json' ([ordered]@{ build_id = $buildReceipt.id; build = $buildRecord; build_checks = $buildChecks
        fixture = $fixtureRecord; source_current_profile_id = $sourceCurrent.id; lease = $leaseRecord; driver_sha256 = $driverSha256
        helper_sha256 = (Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'Playtest-Helpers.ps1')).Hash
        save_origin = 'prepared-not-earned'; test_speed = $TestSpeed; muted = [bool]$Mute; purpose = 'normal1x native UI/save timing'
        ui_capability = 'app-owned ui-state.local.json format1 and same-draw capture-result.local.json required; native check unperformed until leased' })

    $firstProcess = Invoke-ContinuationProcess 1
    $firstIdentity = Open-ContinuationStartup $firstProcess 'startup-menu'
    Assert-ContinuationReopen $sourceSession $firstIdentity.start
    $firstMenu = Read-TurbofishJson $firstProcess.StatePath
    $firstMenuUi = Wait-ContinuationUi -Playtest $firstProcess -Stage Menu
    $saveToken = Get-ContinuationSaveToken
    Send-TurbofishClick -Playtest $firstProcess -X 465 -Y 81 -Reason 'same-mode Adventure resume from startup Menu'
    $sameModeSave = Wait-ContinuationSave $firstProcess $saveToken 'Adventure' 'Playing'
    Assert-ContinuationEqual 'same-mode startup resume preserves complete active scene' (Get-ContinuationScene $firstIdentity.start) (Get-ContinuationScene (Get-ContinuationCurrent $sameModeSave).session)
    $firstSceneUi = Wait-ContinuationScene $firstProcess $firstMenuUi.ui_frame 'Playing'
    $adventureLive = Wait-ContinuationState $firstProcess $firstMenu.session_tick 'normal-speed Adventure resume advances Board' {
        param($sample) -not $sample.paused -and $sample.phase -ceq 'Playing' -and $sample.state.tick -ge $firstMenu.state.tick + 3
    }
    $null = Save-ContinuationCheckpoint -Playtest $firstProcess -Name 'adventure-live' -Stage Scene -AfterUiFrame $firstSceneUi.ui_frame
    $adventurePaused = Suspend-ContinuationBoard $firstProcess $adventureLive 'Playing'
    $adventureMenu = Open-ContinuationMenu $firstProcess 'adventure-menu'
    if ($adventureMenu.state.tick -ne $adventurePaused.state.tick) { throw 'Opening Menu evolved paused Adventure Board clock.' }
    $saveToken = Get-ContinuationSaveToken
    Send-TurbofishClick -Playtest $firstProcess -X 465 -Y 166 -Reason 'admitted retained Time Trial route'
    $timeTrialSave = Wait-ContinuationSave $firstProcess $saveToken 'TimeTrial' 'TimeTrialPlaying'
    $timeTrialSession = (Get-ContinuationCurrent $timeTrialSave).session
    $parkedAdventure = $timeTrialSession.retained_adventure
    $expectedAdventure = [ordered]@{ phase = $adventureMenu.phase; board = $adventureMenu.state; next_seed = $firstIdentity.start.next_seed
        time_trial = $null; challenge = $null; selected_pets = $adventureMenu.progress.selected_pets }
    Assert-ContinuationEqual 'Menu route parks full Adventure payload without an extra Board update' $expectedAdventure $parkedAdventure
    Assert-ContinuationEqual 'TT route resumes supplied retained payload' $sourceSession.retained_time_trial (Get-ContinuationScene $timeTrialSession)
    if ($timeTrialSession.ticks -ne $adventureMenu.session_tick -or $null -ne $timeTrialSession.retained_time_trial) { throw 'Route invented a tick or duplicated TT ownership.' }
    $timeTrialSceneUi = Wait-ContinuationScene $firstProcess $adventureMenu.ui_observation.ui_frame 'TimeTrialPlaying'
    $ttLive = Wait-ContinuationState $firstProcess $adventureMenu.session_tick 'normal-speed TT Board progresses' {
        param($sample) -not $sample.paused -and $sample.phase -ceq 'TimeTrialPlaying' -and $sample.state.tick -ge $sourceSession.retained_time_trial.board.tick + 3
    }
    $null = Save-ContinuationCheckpoint -Playtest $firstProcess -Name 'time-trial-live' -Stage Scene -AfterUiFrame $timeTrialSceneUi.ui_frame
    $ttPaused = Suspend-ContinuationBoard $firstProcess $ttLive 'TimeTrialPlaying'
    $ttMenu = Open-ContinuationMenu $firstProcess 'time-trial-menu'
    if ($ttMenu.state.tick -ne $ttPaused.state.tick) { throw 'Opening Menu evolved paused TT Board clock.' }
    $saveToken = Get-ContinuationSaveToken
    Send-TurbofishClick -Playtest $firstProcess -X 465 -Y 81 -Reason 'return conserved Adventure from transient TT Menu'
    $returnSave = Wait-ContinuationSave $firstProcess $saveToken 'Adventure' 'Playing'
    $returnSession = (Get-ContinuationCurrent $returnSave).session
    Assert-ContinuationEqual 'Adventure resumes byte-value-equivalent parked phase/Board/run/seed/selection' $parkedAdventure (Get-ContinuationScene $returnSession)
    $parkedTimeTrial = $returnSession.retained_time_trial
    Assert-ContinuationEqual 'parked TT Board equals actual frozen TT Menu Board' ([ordered]@{ board = $ttMenu.state }) ([ordered]@{ board = $parkedTimeTrial.board })
    if ($returnSession.ticks -ne $ttMenu.session_tick -or $null -ne $returnSession.retained_adventure -or $parkedTimeTrial.phase -cne 'TimeTrialPlaying') {
        throw 'Return route invented a tick or duplicated active Adventure ownership.'
    }
    $null = Wait-ContinuationScene $firstProcess $ttMenu.ui_observation.ui_frame 'Playing'
    $adventureReturned = Wait-ContinuationState $firstProcess $ttMenu.session_tick 'returned Adventure Board advances' {
        param($sample) -not $sample.paused -and $sample.phase -ceq 'Playing' -and $sample.state.tick -ge $parkedAdventure.board.tick + 3
    }
    $adventurePaused = Suspend-ContinuationBoard $firstProcess $adventureReturned 'Playing'
    $pausedSave = Save-ContinuationPaused $firstProcess $adventurePaused 'Adventure' 'Playing'
    Assert-ContinuationEqual 'TT payload stays frozen while Adventure runs/pauses/saves' $parkedTimeTrial (Get-ContinuationCurrent $pausedSave).session.retained_time_trial
    $exitMenu = Open-ContinuationMenu $firstProcess 'save-exit-menu'
    $closedSave = Close-ContinuationThroughMenu $firstProcess $exitMenu
    $closedSession = (Get-ContinuationCurrent $closedSave).session
    $null = Write-ContinuationJson 'first-game-written-profile.local.json' $closedSave

    # Same isolated directory; no extracting sessions, generating an envelope,
    # replaying credits or replacing the game-written profiles before reopen.
    $secondProcess = Invoke-ContinuationProcess 2
    $secondIdentity = Open-ContinuationStartup $secondProcess 'reopen-menu'
    Assert-ContinuationReopen $closedSession $secondIdentity.start
    Assert-ContinuationEqual 'parked TT conserved through actual graceful file reopen' $parkedTimeTrial $secondIdentity.start.retained_time_trial
    $secondMenu = Read-TurbofishJson $secondProcess.StatePath
    $secondMenuUi = Wait-ContinuationUi -Playtest $secondProcess -Stage Menu
    $saveToken = Get-ContinuationSaveToken
    Send-TurbofishClick -Playtest $secondProcess -X 465 -Y 81 -Reason 'same-mode Adventure resume after actual file reopen'
    $reopenedSave = Wait-ContinuationSave $secondProcess $saveToken 'Adventure' 'Playing'
    Assert-ContinuationEqual 'reopened same-mode resume preserves full conserved scene' (Get-ContinuationScene $secondIdentity.start) (Get-ContinuationScene (Get-ContinuationCurrent $reopenedSave).session)
    $reopenedSceneUi = Wait-ContinuationScene $secondProcess $secondMenuUi.ui_frame 'Playing'
    $reopenedLive = Wait-ContinuationState $secondProcess $secondMenu.session_tick 'reopened Adventure resumes at normal speed' {
        param($sample) -not $sample.paused -and $sample.phase -ceq 'Playing' -and $sample.state.tick -ge $closedSession.board.tick + 3
    }
    $null = Save-ContinuationCheckpoint -Playtest $secondProcess -Name 'reopened-adventure-live' -Stage Scene -AfterUiFrame $reopenedSceneUi.ui_frame
    $reopenedPaused = Suspend-ContinuationBoard $secondProcess $reopenedLive 'Playing'
    Send-TurbofishKey -Playtest $secondProcess -Key Escape
    $resumedAfterPause = Wait-ContinuationState $secondProcess $reopenedPaused.session_tick 'explicit Escape unpause after reopen' {
        param($sample) -not $sample.paused -and $sample.phase -ceq 'Playing' -and $sample.state.tick -ge $reopenedPaused.state.tick + 3
    }
    $finalPaused = Suspend-ContinuationBoard $secondProcess $resumedAfterPause 'Playing'
    $null = Save-ContinuationPaused $secondProcess $finalPaused 'Adventure' 'Playing'
    $finalMenu = Open-ContinuationMenu $secondProcess 'final-menu'
    $finalManager = Close-ContinuationThroughMenu $secondProcess $finalMenu
    Assert-ContinuationEqual 'TT complete payload conserved through both process lifetimes' $parkedTimeTrial (Get-ContinuationCurrent $finalManager).session.retained_time_trial
    $null = Write-ContinuationJson 'final-game-written-profile.local.json' $finalManager
}
catch {
    $failureMessage = $_.Exception.Message
}
finally {
    foreach ($playtest in $ownedProcesses) {
        try {
            $cleanup = Stop-TurbofishPlaytest -Playtest $playtest -GraceMilliseconds 1000
            if ($cleanup.forced -or $cleanup.exit_code -ne 0 -or -not $cleanup.exited) {
                $failureMessage = "$failureMessage Retained-process cleanup was forced/unsuccessful for $($cleanup.process_id)."
            }
        }
        catch { $failureMessage = "$failureMessage Cleanup error: $($_.Exception.Message)" }
    }
    foreach ($inputRecord in $immutableInputs) {
        try { $null = Get-ContinuationFile $inputRecord.path $inputRecord.sha256 }
        catch { $failureMessage = "$failureMessage Immutable input changed: $($_.Exception.Message)" }
    }
    if ($runAllocated) {
        try {
            $events = [System.Collections.Generic.List[object]]::new()
            foreach ($playtest in $ownedProcesses) {
                $eventPath = Join-Path $playtest.EvidenceDirectory 'events.local.jsonl'
                $lineNumber = 0
                if (-not (Test-Path -LiteralPath $eventPath -PathType Leaf)) { throw 'Runtime event evidence is missing.' }
                foreach ($line in [System.IO.File]::ReadLines($eventPath)) {
                    $lineNumber++
                    if ($lineNumber -gt 20000) { throw 'Event evidence exceeded bounded scenario inventory.' }
                    $row = $line | ConvertFrom-Json -AsHashtable -Depth 100
                    if ($row.Contains('event')) {
                        if ($row.test_speed -ne 1 -or $row.time_mode -cne 'normal' -or $null -eq $row.session_tick) { throw 'Event row lacks normal1x/tick identity.' }
                        if ($row.event.Contains('Rejected')) { throw "Rejected input at ${eventPath}:${lineNumber}." }
                        $events.Add([ordered]@{ path = $eventPath; line = $lineNumber; sha256 = (Get-FileHash -LiteralPath $eventPath).Hash
                            session_tick = $row.session_tick; event = $row.event })
                    }
                }
                $events.Add([ordered]@{ path = $eventPath; rows = $lineNumber; sha256 = (Get-FileHash -LiteralPath $eventPath).Hash
                    limitation = 'retained/same-mode route transactions emit no dedicated mode event; saved payloads and consumed frames supply the boundary evidence' })
            }
            $null = Write-ContinuationJson 'event-index.local.json' $events.ToArray()
        }
        catch { $failureMessage = "$failureMessage Event review error: $($_.Exception.Message)" }
        $result = [ordered]@{ result = $(if ($failureMessage) { 'failed' } else { 'passed-awaiting-visual-review' }); failure = $failureMessage
            scope = 'prepared current37 mode/save/pause native conservation; no earned progression or full runtime acceptance'
            native_conservation = $(if ($failureMessage) { 'failed/incomplete' } else { 'passed' })
            title_menu_stage_checks = $(if ($failureMessage) { 'failed/incomplete' } else { 'passed: owned PID, fresh UI frame, observed ready bounds and same-draw captures' })
            captured_visuals = 'pending pixel review; stage telemetry does not establish pixel fidelity'
            retail_audio_human = 'not tested'; test_speed = 1; time_mode = 'normal'; wall_elapsed_seconds = $scenarioTimer.Elapsed.TotalSeconds
            checks = $checks.ToArray(); checkpoints = $checkpoints.ToArray(); cleanup = @($ownedProcesses | ForEach-Object { $_.Cleanup })
            immutable_inputs = $immutableInputs.ToArray(); save_origin = 'prepared-not-earned' }
        $resultPath = Write-ContinuationJson 'result.local.json' $result
        [pscustomobject]@{ Result = $result.result; RunDirectory = $runRoot; Receipt = $resultPath; Failure = $failureMessage }
    }
}
if ($failureMessage) { throw $failureMessage }
