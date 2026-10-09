# Focused contract regressions for Playtest-Helpers.ps1. These tests use mock
# process/window-message boundaries; they do not launch or control the game.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'Playtest-Helpers.ps1')

$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$scratchRoot = [System.IO.Path]::GetFullPath((Join-Path $repositoryRoot '.scratch/playtests'))
$testRoot = [System.IO.Path]::GetFullPath((Join-Path $scratchRoot ('helper-tests-' + [guid]::NewGuid().ToString('N'))))
if (-not $testRoot.StartsWith($scratchRoot + [System.IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Test directory escaped the project scratch tree.'
}

$script:passCount = 0
$script:failures = [System.Collections.Generic.List[string]]::new()
function Test-Case {
    param([string]$Name, [scriptblock]$Body)
    try { & $Body; $script:passCount++; Write-Output "PASS $Name" }
    catch { $script:failures.Add("${Name}: $($_.Exception.Message)"); Write-Output "FAIL ${Name}: $($_.Exception.Message)" }
}
function Require-True {
    param([bool]$Condition, [string]$Reason)
    if (-not $Condition) { throw $Reason }
}
function Require-Throws {
    param([scriptblock]$Body, [string]$MessagePattern)
    $caught = $null
    try { & $Body | Out-Null } catch { $caught = $_.Exception.Message }
    if ($null -eq $caught) { throw "Expected failure matching '$MessagePattern'." }
    if ($caught -notmatch $MessagePattern) { throw "Wrong failure: $caught" }
}
function New-TestProcess {
    param([bool]$Exited = $false, [bool]$WaitSucceeds = $true)
    $mockProcess = [pscustomobject]@{ Id = 4242; HasExited = $Exited; ExitCode = 0; WaitSucceeds = $WaitSucceeds;
        RefreshCount = 0; WaitCount = 0; KillCount = 0; MainWindowHandle = [IntPtr]::Zero; OnRefresh = $null }
    $mockProcess | Add-Member ScriptMethod Refresh { $this.RefreshCount++; if ($null -ne $this.OnRefresh) { & $this.OnRefresh } }
    $mockProcess | Add-Member ScriptMethod WaitForExit {
        param([int]$Milliseconds)
        $this.WaitCount++
        if ($this.WaitSucceeds) { $this.HasExited = $true }
        return $this.WaitSucceeds
    }
    $mockProcess | Add-Member ScriptMethod Kill { $this.KillCount++; $this.HasExited = $true; $this.WaitSucceeds = $true }
    return $mockProcess
}
function New-TestSnapshot {
    param([long]$Tick = 1, [bool]$Paused = $false)
    return @{ elapsed_seconds = 0.1; paused = $Paused; session_tick = $Tick; phase = 'Board'; progress = @{};
        state = @{ sim = @{ money = 5 }; housekeeping = @{ punch = 1 } } }
}
function New-CompleteSessionJson {
    param([string]$Fields)
    return ('{"progress":{},"phase":"Playing","ticks":0,"next_seed":0,' + $Fields + '}')
}
function New-TestPlaytest {
    param($MockProcess, [string]$Name)
    $runPath = Join-Path $testRoot $Name
    New-Item -ItemType Directory -Path $runPath -Force | Out-Null
    return [pscustomobject]@{ Process = $MockProcess; RunDirectory = $runPath; EvidenceDirectory = $runPath;
        StatePath = (Join-Path $runPath 'state.local.json'); InputLogPath = (Join-Path $runPath 'inputs.local.jsonl'); Cleanup = $null }
}

try {
    New-Item -ItemType Directory -Path $testRoot -Force | Out-Null

    Test-Case 'fresh state schema accepts required keys' {
        Assert-TurbofishSnapshot -Snapshot (New-TestSnapshot)
    }
    Test-Case 'board alias and each missing required key fail' {
        $requiredKeys = @('elapsed_seconds', 'paused', 'session_tick', 'phase', 'progress', 'state')
        foreach ($requiredKey in $requiredKeys) {
            $sample = New-TestSnapshot
            $sample.Remove($requiredKey)
            if ($requiredKey -eq 'state') { $sample.board = @{} }
            Require-Throws { Assert-TurbofishSnapshot -Snapshot $sample } "missing '$requiredKey'"
        }
    }
    Test-Case 'snapshot tick and pause type boundaries fail' {
        $sample = New-TestSnapshot
        $sample.session_tick = -1
        Require-Throws { Assert-TurbofishSnapshot -Snapshot $sample } 'nonnegative integer'
        $sample.session_tick = 1
        $sample.paused = 'false'
        Require-Throws { Assert-TurbofishSnapshot -Snapshot $sample } 'Boolean'
    }

    # Stub only the message boundary: inspect exact WM_KEYDOWN/UP parameters.
    $script:sentMessages = [System.Collections.Generic.List[object]]::new()
    $script:failRelease = $false
    function Send-TurbofishWindowMessage {
        param($Playtest, [uint32]$Message, [long]$WParam, [long]$LParam)
        $script:sentMessages.Add([pscustomobject]@{ Message = $Message; WParam = $WParam; LParam = $LParam })
        if ($script:failRelease -and $Message -eq 257) { throw 'release sentinel' }
    }
    Test-Case 'Escape posts exact scan-code bits and receipt' {
        $script:sentMessages.Clear()
        $playtest = New-TestPlaytest -MockProcess (New-TestProcess) -Name 'input'
        Send-TurbofishKey -Playtest $playtest -Key Escape -HoldMilliseconds 30
        Require-True ($script:sentMessages.Count -eq 2) 'Expected one down and one up message.'
        Require-True ($script:sentMessages[0].Message -eq 256 -and $script:sentMessages[0].WParam -eq 27 -and $script:sentMessages[0].LParam -eq 65537) 'Wrong Escape keydown.'
        Require-True ($script:sentMessages[1].Message -eq 257 -and $script:sentMessages[1].WParam -eq 27 -and $script:sentMessages[1].LParam -eq 3221291009) 'Wrong Escape keyup.'
        $receipt = Read-TurbofishJson -Path $playtest.InputLogPath
        Require-True ($receipt.scan_code -eq 1 -and $receipt.down -eq 65537 -and $receipt.up -eq 3221291009) 'Input receipt differs.'
    }
    Test-Case 'exited owner has no usable window' {
        $playtest = New-TestPlaytest -MockProcess (New-TestProcess -Exited $true) -Name 'exited-window'
        Require-Throws { Assert-TurbofishOwnedWindow -Playtest $playtest } 'exited'
    }
    Test-Case 'live owner without main window cannot receive input' {
        $playtest = New-TestPlaytest -MockProcess (New-TestProcess) -Name 'missing-window'
        Require-Throws { Assert-TurbofishOwnedWindow -Playtest $playtest } 'no main window'
    }
    Test-Case 'key hold failure still releases and does not claim input receipt' {
        $script:sentMessages.Clear()
        $playtest = New-TestPlaytest -MockProcess (New-TestProcess) -Name 'key-hold-failure'
        function Start-Sleep { param([int]$Milliseconds) throw 'hold sentinel' }
        try { Require-Throws { Send-TurbofishKey -Playtest $playtest -Key Escape -HoldMilliseconds 30 } 'hold sentinel' }
        finally { Remove-Item Function:Start-Sleep }
        Require-True ($script:sentMessages.Count -eq 2 -and $script:sentMessages[1].Message -eq 257) 'Key release was omitted after hold failure.'
        Require-True (-not (Test-Path -LiteralPath $playtest.InputLogPath)) 'Failed input wrote a success receipt.'
    }
    Test-Case 'key release failure surfaces and does not claim input receipt' {
        $script:sentMessages.Clear()
        $script:failRelease = $true
        $playtest = New-TestPlaytest -MockProcess (New-TestProcess) -Name 'key-release-failure'
        try { Require-Throws { Send-TurbofishKey -Playtest $playtest -Key Escape -HoldMilliseconds 30 } 'release sentinel' }
        finally { $script:failRelease = $false }
        Require-True ($script:sentMessages.Count -eq 2 -and -not (Test-Path -LiteralPath $playtest.InputLogPath)) 'Failed release wrote a success receipt.'
    }

    Test-Case 'actual malformed publication uses a retryable exception type' {
        $playtest = New-TestPlaytest -MockProcess (New-TestProcess) -Name 'malformed-json'
        Set-Content -LiteralPath $playtest.StatePath -Value '{' -NoNewline
        $caught = $null
        try { Read-TurbofishJson -Path $playtest.StatePath | Out-Null }
        catch { $caught = $_.Exception }
        Require-True ($caught -is [System.IO.IOException] -or $caught -is [System.ArgumentException]) "Malformed JSON raised $($caught.GetType().FullName), outside wait's retry catches."
    }

    Test-Case 'wait retries transient telemetry read error' {
        $script:readAttempts = 0
        function Read-TurbofishJson {
            param([string]$Path)
            $script:readAttempts++
            if ($script:readAttempts -eq 1) { throw [System.IO.IOException]::new('transient missing snapshot') }
            if ($script:readAttempts -eq 2) { throw [System.ArgumentException]::new('transient malformed snapshot') }
            return (New-TestSnapshot -Tick 2)
        }
        $playtest = New-TestPlaytest -MockProcess (New-TestProcess) -Name 'retry'
        $sample = Wait-TurbofishState -Playtest $playtest -Predicate { param($item) $item.session_tick -eq 2 } -Description 'tick two' -TimeoutSeconds 0.3 -AfterSessionTick 1
        Require-True ($sample.session_tick -eq 2 -and $script:readAttempts -eq 3) 'Read retry failed.'
    }
    Test-Case 'wait surfaces predicate programming errors' {
        function Read-TurbofishJson { param([string]$Path) return (New-TestSnapshot) }
        $playtest = New-TestPlaytest -MockProcess (New-TestProcess) -Name 'predicate'
        Require-Throws { Wait-TurbofishState -Playtest $playtest -Predicate { throw 'predicate sentinel' } -Description 'broken predicate' -TimeoutSeconds 0.1 } 'predicate sentinel'
    }
    Test-Case 'wait timeout reports last sample and description' {
        function Read-TurbofishJson { param([string]$Path) return (New-TestSnapshot -Tick 3) }
        $playtest = New-TestPlaytest -MockProcess (New-TestProcess) -Name 'timeout'
        Require-Throws { Wait-TurbofishState -Playtest $playtest -Predicate { $false } -Description 'wanted tick' -TimeoutSeconds 0.05 } 'Timed out.*wanted tick.*session_tick=3'
    }
    Test-Case 'wait reports owned process exit' {
        $playtest = New-TestPlaytest -MockProcess (New-TestProcess -Exited $true) -Name 'wait-exit'
        Require-Throws { Wait-TurbofishState -Playtest $playtest -Predicate { $true } -Description 'exit check' -TimeoutSeconds 0.05 } 'exit check.*owned process exited'
    }
    Test-Case 'nonboolean predicate result cannot cause false success' {
        function Read-TurbofishJson { param([string]$Path) return (New-TestSnapshot -Tick 3) }
        $playtest = New-TestPlaytest -MockProcess (New-TestProcess) -Name 'nonboolean-predicate'
        Require-Throws { Wait-TurbofishState -Playtest $playtest -Predicate { 'false' } -Description 'bad predicate' -TimeoutSeconds 0.05 } 'predicate.*Boolean'
    }

    # Only the ownership assertion is stubbed. The capture request/completion
    # path still reads and writes real files under this test's unique scratch.
    function Assert-TurbofishOwnedWindow { param($Playtest) return [IntPtr]1 }
    Test-Case 'capture waits for newly published nonempty frame' {
        $mockProcess = New-TestProcess
        $playtest = New-TestPlaytest -MockProcess $mockProcess -Name 'capture-success'
        Set-Content -LiteralPath (Join-Path $playtest.EvidenceDirectory 'frame-old.png') -Value 'old' -NoNewline
        $capturePath = Join-Path $playtest.EvidenceDirectory 'capture.request'
        $framePath = Join-Path $playtest.EvidenceDirectory 'frame-new.png'
        $mockProcess.OnRefresh = { Remove-Item -LiteralPath $capturePath; Set-Content -LiteralPath $framePath -Value 'mock bytes' -NoNewline }.GetNewClosure()
        $published = Request-TurbofishCapture -Playtest $playtest -TimeoutSeconds 0.3
        Require-True ($published -eq $framePath -and -not (Test-Path -LiteralPath $capturePath)) 'Capture returned before new frame completion.'
    }
    Test-Case 'capture rejects pending request and times out without new frame' {
        $playtest = New-TestPlaytest -MockProcess (New-TestProcess) -Name 'capture-timeout'
        $capturePath = Join-Path $playtest.EvidenceDirectory 'capture.request'
        New-Item -ItemType File -Path $capturePath | Out-Null
        Require-Throws { Request-TurbofishCapture -Playtest $playtest -TimeoutSeconds 0.1 } 'still pending'
        Remove-Item -LiteralPath $capturePath
        Require-Throws { Request-TurbofishCapture -Playtest $playtest -TimeoutSeconds 0.1 } 'did not produce a new nonempty frame'
    }
    Test-Case 'capture reports process exit before publication' {
        $mockProcess = New-TestProcess
        $playtest = New-TestPlaytest -MockProcess $mockProcess -Name 'capture-exit'
        $mockProcess.OnRefresh = { $mockProcess.HasExited = $true }.GetNewClosure()
        Require-Throws { Request-TurbofishCapture -Playtest $playtest -TimeoutSeconds 0.1 } 'exited before capture completion'
    }

    Test-Case 'reload tolerates object order but rejects shape changes' {
        $base = New-CompleteSessionJson '"board":{"fish":[{"id":1},{"id":2}]},"coins":1'
        $reordered = '{"coins":1,"board":{"fish":[{"id":1},{"id":2}]},"next_seed":0,"ticks":0,"phase":"Playing","progress":{}}'
        Require-True ((Assert-TurbofishReload -ExpectedSessionJson $base -ActualSessionJson $reordered).Equal) 'Object order differed.'
        foreach ($changed in @((New-CompleteSessionJson '"board":{"fish":[{"id":1},{"id":2}]}'),
                (New-CompleteSessionJson '"board":{"fish":[{"id":1},{"id":2}]},"coins":1,"extra":null'),
                (New-CompleteSessionJson '"Board":{"fish":[{"id":1},{"id":2}]},"coins":1'),
                (New-CompleteSessionJson '"board":{"fish":[{"id":1}]},"coins":1'),
                (New-CompleteSessionJson '"board":{"fish":[{"id":1},{"id":2}]},"coins":null'))) {
            Require-Throws { Assert-TurbofishReload -ExpectedSessionJson $base -ActualSessionJson $changed } 'Reload'
        }
    }
    Test-Case 'reload rejects vacuous null sessions' {
        Require-Throws { Assert-TurbofishReload -ExpectedSessionJson 'null' -ActualSessionJson 'null' } 'Reload.*session'
        Require-True ((Compare-TurbofishJson -ExpectedJson 'null' -ActualJson 'null').Equal) 'Generic JSON comparator changed.'
    }
    Test-Case 'reload rejects incomplete session objects' {
        $complete = New-CompleteSessionJson '"board":null'
        Require-True ((Assert-TurbofishReload -ExpectedSessionJson $complete -ActualSessionJson $complete).Equal) 'Null board should be valid.'
        Require-Throws { Assert-TurbofishReload -ExpectedSessionJson '{}' -ActualSessionJson '{}' } 'Reload session is missing'
        foreach ($requiredKey in @('progress', 'board', 'phase', 'ticks', 'next_seed')) {
            $missing = $complete.Replace(('"' + $requiredKey + '":'), ('"omitted_' + $requiredKey + '":'))
            Require-Throws { Assert-TurbofishReload -ExpectedSessionJson $complete -ActualSessionJson $missing } "Reload session is missing '$requiredKey'"
        }
    }
    Test-Case 'reload requires exact u64 integers and negative zero' {
        $base = New-CompleteSessionJson '"board":null,"id":18446744073709551615,"value":-0'
        Require-True ((Assert-TurbofishReload -ExpectedSessionJson $base -ActualSessionJson (New-CompleteSessionJson '"value":-0,"id":18446744073709551615,"board":null')).Equal) 'Exact integers changed.'
        foreach ($changed in @((New-CompleteSessionJson '"board":null,"id":18446744073709551614,"value":-0'),
                (New-CompleteSessionJson '"board":null,"id":18446744073709551615,"value":0'))) {
            try { Require-Throws { Assert-TurbofishReload -ExpectedSessionJson $base -ActualSessionJson $changed } 'Reload differs' }
            catch { throw "Actual $changed : $($_.Exception.Message)" }
        }
    }
    Test-Case 'only declared f32 fields admit equal binary32 spellings' {
        $base = New-CompleteSessionJson '"board":{"fish":[{"x":0.10000000149011612,"id":1,"hunger":0.10000000149011612}],"coins":[{"value":0.10000000149011612}],"breeders":[{"id":1}]}'
        $changed = New-CompleteSessionJson '"board":{"fish":[{"x":0.1,"id":1,"hunger":0.10000000149011612}],"coins":[{"value":0.10000000149011612}],"breeders":[{"id":1}]}'
        $comparison = Assert-TurbofishReload -ExpectedSessionJson $base -ActualSessionJson $changed
        Require-True ($comparison.Equal -and '$.board.fish[0].x' -cin $comparison.TypedEquivalentPaths) 'Declared f32 spelling did not compare by bits.'
        Require-Throws { Assert-TurbofishReload -ExpectedSessionJson $base -ActualSessionJson ($changed.Replace('"x":0.1', '"x":0.10000002')) } 'Reload differs'
        Require-Throws { Assert-TurbofishReload -ExpectedSessionJson (New-CompleteSessionJson '"board":{"fish":[{"x":-0.0}]}') -ActualSessionJson (New-CompleteSessionJson '"board":{"fish":[{"x":0.0}]}') } 'Reload differs'
        foreach ($strictField in @('"hunger":0.10000000149011612', '"value":0.10000000149011612', '"id":1')) {
            $replacement = if ($strictField -eq '"id":1') { '"id":2' } else { $strictField.Replace('0.10000000149011612', '0.1') }
            try { Require-Throws { Assert-TurbofishReload -ExpectedSessionJson $base -ActualSessionJson ($base.Replace($strictField, $replacement)) } 'Reload differs' }
            catch { throw "Actual $replacement : $($_.Exception.Message)" }
        }
    }

    Test-Case 'pause compares selected simulation while housekeeping changes' {
        $before = New-TestSnapshot -Tick 10 -Paused $true
        $after = New-TestSnapshot -Tick 11 -Paused $true
        $after.state.housekeeping.punch = 2
        Assert-TurbofishPausedState -Before $before -After $after -SelectState { param($item) $item.state.sim } -Description 'simulation'
        $after.state.sim.money = 6
        Require-Throws { Assert-TurbofishPausedState -Before $before -After $after -SelectState { param($item) $item.state.sim } -Description 'simulation' } 'Paused simulation changed'
    }
    Test-Case 'pause requires two paused advancing snapshots and selection' {
        $before = New-TestSnapshot -Tick 10 -Paused $true
        $after = New-TestSnapshot -Tick 10 -Paused $true
        Require-Throws { Assert-TurbofishPausedState -Before $before -After $after -SelectState { param($item) $item.state.sim } -Description 'clock' } 'advancing session_tick'
        $after.session_tick = [long]11
        Require-Throws { Assert-TurbofishPausedState -Before $before -After $after -SelectState { $null } -Description 'null projection' } 'non-null state'
        $after.paused = $false
        Require-Throws { Assert-TurbofishPausedState -Before $before -After $after -SelectState { param($item) $item.state.sim } -Description 'unpaused' } 'two paused snapshots'
    }

    Test-Case 'cleanup of already-exited retained owner writes stable receipt' {
        $mockProcess = New-TestProcess -Exited $true
        $playtest = New-TestPlaytest -MockProcess $mockProcess -Name 'cleanup-exited'
        $first = Stop-TurbofishPlaytest -Playtest $playtest -GraceMilliseconds 100
        $second = Stop-TurbofishPlaytest -Playtest $playtest -GraceMilliseconds 100
        $receipt = Read-TurbofishJson -Path (Join-Path $playtest.RunDirectory 'cleanup.local.json')
        Require-True ($first.exited -and -not $first.forced -and $second -eq $first) 'Cleanup receipt/idempotence failed.'
        Require-True ($receipt.exited -and $receipt.process_id -eq 4242 -and $mockProcess.KillCount -eq 0 -and $mockProcess.WaitCount -eq 0) 'Exited owner was acted on.'
    }
    Test-Case 'cleanup forced fallback confirms exit and persists receipt' {
        $mockProcess = New-TestProcess -WaitSucceeds $false
        $playtest = New-TestPlaytest -MockProcess $mockProcess -Name 'cleanup-forced'
        $first = Stop-TurbofishPlaytest -Playtest $playtest -GraceMilliseconds 100
        $second = Stop-TurbofishPlaytest -Playtest $playtest -GraceMilliseconds 100
        $receipt = Read-TurbofishJson -Path (Join-Path $playtest.RunDirectory 'cleanup.local.json')
        Require-True ($first.forced -and $first.exited -and $receipt.forced -and $receipt.exited) 'Forced cleanup receipt missing.'
        Require-True ($mockProcess.KillCount -eq 1 -and $mockProcess.WaitCount -eq 2 -and $second -eq $first) 'Fallback or rerun acted incorrectly.'
    }
}
finally {
    # This script owns only its unique child directory, whose absolute path was checked above.
    if (Test-Path -LiteralPath $testRoot) { Remove-Item -LiteralPath $testRoot -Recurse -Force -ErrorAction Stop }
    Write-Output "Scratch cleanup: $testRoot removed"
}

Write-Output "Helper regressions: $script:passCount passed, $($script:failures.Count) failed"
if ($script:failures.Count -gt 0) { throw ($script:failures -join "`n") }
