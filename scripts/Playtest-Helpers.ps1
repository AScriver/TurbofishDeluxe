# Dot-source from a scenario. All outputs stay in a new ignored .scratch run directory.
# The functions own only processes they start; no global input or name-based cleanup.

function Initialize-TurbofishPlaytestNative {
    if (-not $IsWindows) { throw 'Native playtests require Windows.' }
    if ('Turbofish.Playtest.Native' -as [type]) { return }
    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
namespace Turbofish.Playtest {
    public static class Native {
        [StructLayout(LayoutKind.Sequential)]
        public struct Rect { public int Left, Top, Right, Bottom; }
        [DllImport("user32.dll", SetLastError=true)]
        public static extern bool GetClientRect(IntPtr window, out Rect rect);
        [DllImport("user32.dll", SetLastError=true)]
        public static extern uint GetWindowThreadProcessId(IntPtr window, out uint processId);
        [DllImport("user32.dll", SetLastError=true)]
        public static extern bool PostMessageW(IntPtr window, uint message, IntPtr wParam, IntPtr lParam);
    }
}
'@
}

function Read-TurbofishJson {
    [CmdletBinding()]
    param([Parameter(Mandatory)][string]$Path)
    # Delete sharing lets the runtime atomically replace optional telemetry while read.
    $fileStream = [System.IO.FileStream]::new($Path, [System.IO.FileMode]::Open,
        [System.IO.FileAccess]::Read, [System.IO.FileShare]::ReadWrite -bor [System.IO.FileShare]::Delete)
    $reader = [System.IO.StreamReader]::new($fileStream)
    try { return ($reader.ReadToEnd() | ConvertFrom-Json -AsHashtable -Depth 100 -ErrorAction Stop) }
    finally { $reader.Dispose(); $fileStream.Dispose() }
}

function Assert-TurbofishSnapshot {
    [CmdletBinding()]
    param([Parameter(Mandatory)][System.Collections.IDictionary]$Snapshot)
    foreach ($key in @('elapsed_seconds', 'paused', 'session_tick', 'phase', 'progress', 'state')) {
        if (-not $Snapshot.Contains($key)) { throw "Invalid live snapshot: missing '$key' (Board key is 'state')." }
    }
    if ($Snapshot.paused -isnot [bool] -or $Snapshot.session_tick -isnot [long] -or $Snapshot.session_tick -lt 0) {
        throw 'Invalid live snapshot: paused must be Boolean and session_tick a nonnegative integer.'
    }
}

function Assert-TurbofishOwnedWindow {
    [CmdletBinding()]
    param([Parameter(Mandatory)]$Playtest)
    $Playtest.Process.Refresh()
    if ($Playtest.Process.HasExited) { throw "Owned process $($Playtest.Process.Id) exited ($($Playtest.Process.ExitCode))." }
    $windowHandle = $Playtest.Process.MainWindowHandle
    if ($windowHandle -eq [IntPtr]::Zero) { throw 'Owned process has no main window.' }
    $windowProcessId = [uint32]0
    $threadId = [Turbofish.Playtest.Native]::GetWindowThreadProcessId($windowHandle, [ref]$windowProcessId)
    if ($threadId -eq 0 -or $windowProcessId -ne $Playtest.Process.Id) { throw 'Window ownership check failed.' }
    return $windowHandle
}

function Send-TurbofishWindowMessage {
    param([Parameter(Mandatory)]$Playtest, [uint32]$Message, [long]$WParam, [long]$LParam)
    $windowHandle = Assert-TurbofishOwnedWindow -Playtest $Playtest
    if (-not [Turbofish.Playtest.Native]::PostMessageW($windowHandle, $Message, [IntPtr]$WParam, [IntPtr]$LParam)) {
        throw "Owned-window message failed: $([System.Runtime.InteropServices.Marshal]::GetLastWin32Error())."
    }
}

function Write-TurbofishInputReceipt {
    param($Playtest, [System.Collections.IDictionary]$InputRecord)
    $InputRecord.utc = [DateTime]::UtcNow.ToString('o')
    $InputRecord.process_id = $Playtest.Process.Id
    $InputRecord | ConvertTo-Json -Compress -Depth 8 | Add-Content -LiteralPath $Playtest.InputLogPath -Encoding utf8NoBOM
}

function Start-TurbofishPlaytest {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][string]$ExecutablePath,
        [Parameter(Mandatory)][ValidatePattern('^[a-fA-F0-9]{64}$')][string]$ExecutableSha256,
        [string]$RunDirectory,
        [string]$SourceSavePath,
        [ValidatePattern('^[a-fA-F0-9]{64}$')][string]$SourceSaveSha256,
        [uint64]$Seed = 42,
        [ValidateRange(5, 3600)][int]$QuitAfterSeconds = 30,
        [ValidateRange(1, 8)][int]$TestSpeed = 1,
        [string]$GameDirectory,
        [switch]$Mute
    )
    if ($TestSpeed -gt 1 -and -not $Mute) { throw 'Accelerated playtests require -Mute; audio fidelity is not exercised.' }
    Initialize-TurbofishPlaytestNative
    $repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
    $executable = (Resolve-Path -LiteralPath $ExecutablePath -ErrorAction Stop).ProviderPath
    if ([System.IO.Path]::GetFileName($executable) -cne 'turbofish-deluxe.exe') { throw 'Use the project turbofish-deluxe.exe, never the retail executable.' }
    if ((Get-FileHash -LiteralPath $executable -Algorithm SHA256).Hash -ne $ExecutableSha256) { throw 'Executable digest differs.' }
    if ([bool]$SourceSavePath -ne [bool]$SourceSaveSha256) { throw 'Source save path and SHA-256 must be supplied together.' }
    if ($SourceSavePath -and (Get-FileHash -LiteralPath $SourceSavePath -Algorithm SHA256).Hash -ne $SourceSaveSha256) { throw 'Source save digest differs.' }
    $scratchRoot = Join-Path $repositoryRoot '.scratch'
    if (-not $RunDirectory) { $RunDirectory = Join-Path $scratchRoot ('playtests/helpers-' + [guid]::NewGuid().ToString('N')) }
    $runRoot = [System.IO.Path]::GetFullPath($RunDirectory)
    if (-not $runRoot.StartsWith($scratchRoot + [System.IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Run directory must be inside this repository .scratch tree.'
    }
    if (Test-Path -LiteralPath $runRoot) { throw 'Run directory already exists; preserve its evidence and choose a new path.' }
    # Refuse junction/symlink parents rather than letting a private output escape.
    $ancestorPath = Split-Path -Parent $runRoot
    while ($ancestorPath -and $ancestorPath.Length -ge $repositoryRoot.Length) {
        if ((Test-Path -LiteralPath $ancestorPath) -and ((Get-Item -LiteralPath $ancestorPath).Attributes -band [System.IO.FileAttributes]::ReparsePoint)) {
            throw "Run directory has a reparse-point parent: $ancestorPath"
        }
        $ancestorPath = Split-Path -Parent $ancestorPath
    }
    $saveRoot = Join-Path $runRoot 'save'
    $evidenceRoot = Join-Path $runRoot 'evidence'
    New-Item -ItemType Directory -Path $saveRoot, $evidenceRoot -ErrorAction Stop | Out-Null
    if ($SourceSavePath) {
        $copiedSave = Join-Path $saveRoot 'adventure.json'
        Copy-Item -LiteralPath $SourceSavePath -Destination $copiedSave -ErrorAction Stop
        if ((Get-FileHash -LiteralPath $copiedSave -Algorithm SHA256).Hash -ne $SourceSaveSha256) { throw 'Source save copy differs.' }
    }
    $nativeArguments = @('--seed', $Seed.ToString(), '--save-dir', $saveRoot, '--evidence-dir', $evidenceRoot,
        '--quit-after', $QuitAfterSeconds.ToString())
    if (-not $SourceSavePath) { $nativeArguments += '--new-game' }
    if ($GameDirectory) { $nativeArguments += @('--game-dir', (Resolve-Path -LiteralPath $GameDirectory -ErrorAction Stop).ProviderPath) }
    if ($Mute) { $nativeArguments += '--mute' }
    if ($TestSpeed -gt 1) { $nativeArguments += @('--test-speed', $TestSpeed.ToString()) }
    $startInfo = [System.Diagnostics.ProcessStartInfo]::new($executable)
    $startInfo.UseShellExecute = $false
    $startInfo.WorkingDirectory = $repositoryRoot
    foreach ($nativeArgument in $nativeArguments) { $startInfo.ArgumentList.Add($nativeArgument) }
    $ownedProcess = [System.Diagnostics.Process]::new()
    $ownedProcess.StartInfo = $startInfo
    $processStarted = $false
    $playtest = [pscustomobject]@{
        Process = $ownedProcess; RunDirectory = $runRoot; EvidenceDirectory = $evidenceRoot
        SavePath = (Join-Path $saveRoot 'adventure.json'); StatePath = (Join-Path $evidenceRoot 'state.local.json')
        InputLogPath = (Join-Path $runRoot 'inputs.local.jsonl'); Cleanup = $null
        TestSpeed = $TestSpeed
    }
    try {
        if (-not $ownedProcess.Start()) { throw 'Project process did not start.' }
        $processStarted = $true
        [ordered]@{ process_id = $ownedProcess.Id; executable = $executable; executable_sha256 = $ExecutableSha256
            arguments = $nativeArguments; source_save = $SourceSavePath; source_save_sha256 = $SourceSaveSha256
            test_speed = $TestSpeed; time_mode = $(if ($TestSpeed -eq 1) { 'normal' } else { 'accelerated-test' })
            helper_sha256 = (Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash; started_utc = [DateTime]::UtcNow.ToString('o')
        } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $runRoot 'launch.local.json') -Encoding utf8NoBOM
        $windowWait = [System.Diagnostics.Stopwatch]::StartNew()
        do {
            $ownedProcess.Refresh()
            if ($ownedProcess.HasExited) { throw "Project exited before creating a window ($($ownedProcess.ExitCode))." }
            if ($ownedProcess.MainWindowHandle -ne [IntPtr]::Zero) {
                Assert-TurbofishOwnedWindow -Playtest $playtest | Out-Null
                return $playtest
            }
            Start-Sleep -Milliseconds 25
        } while ($windowWait.Elapsed.TotalSeconds -lt 10)
        throw 'Timed out waiting for owned window after 10 seconds.'
    }
    catch {
        if ($processStarted) { Stop-TurbofishPlaytest -Playtest $playtest | Out-Null }
        throw
    }
}

function Send-TurbofishKey {
    [CmdletBinding()]
    param([Parameter(Mandatory)]$Playtest, [Parameter(Mandatory)][ValidateSet('Escape', 'Enter', 'S', 'Q', 'F12')][string]$Key,
        [ValidateRange(30, 1000)][int]$HoldMilliseconds = 60)
    $keyTable = @{ Escape = @(27, 1); Enter = @(13, 28); S = @(83, 31); Q = @(81, 16); F12 = @(123, 88) }
    $virtualKey, $scanCode = $keyTable[$Key]
    $downBits = [long](1 -bor ($scanCode -shl 16))
    $upBits = $downBits -bor [long]3221225472 # Previous/transition bits 30 and 31.
    Send-TurbofishWindowMessage -Playtest $Playtest -Message 0x0100 -WParam $virtualKey -LParam $downBits
    try { Start-Sleep -Milliseconds $HoldMilliseconds }
    finally {
        if (-not $Playtest.Process.HasExited) { Send-TurbofishWindowMessage -Playtest $Playtest -Message 0x0101 -WParam $virtualKey -LParam $upBits }
    }
    Write-TurbofishInputReceipt -Playtest $Playtest -InputRecord ([ordered]@{ kind = 'key'; key = $Key; scan_code = $scanCode; down = $downBits; up = $upBits })
}

function Send-TurbofishClick {
    [CmdletBinding()]
    param([Parameter(Mandatory)]$Playtest, [Parameter(Mandatory)][ValidateRange(0, 639)][double]$X,
        [Parameter(Mandatory)][ValidateRange(0, 479)][double]$Y, [string]$Reason = 'scenario click')
    $windowHandle = Assert-TurbofishOwnedWindow -Playtest $Playtest
    $clientRect = [Turbofish.Playtest.Native+Rect]::new()
    if (-not [Turbofish.Playtest.Native]::GetClientRect($windowHandle, [ref]$clientRect)) { throw 'GetClientRect failed.' }
    $width = $clientRect.Right - $clientRect.Left
    $height = $clientRect.Bottom - $clientRect.Top
    $scale = [Math]::Min($width / 640.0, $height / 480.0)
    if ($scale -le 0) { throw 'Owned window has an empty client area.' }
    $clientX = [int][Math]::Round(($width - 640 * $scale) / 2 + $X * $scale)
    $clientY = [int][Math]::Round(($height - 480 * $scale) / 2 + $Y * $scale)
    if ($clientX -gt 32767 -or $clientY -gt 32767) { throw 'Client coordinate exceeds signed window-message range.' }
    $position = [long]($clientX -bor ($clientY -shl 16))
    Send-TurbofishWindowMessage -Playtest $Playtest -Message 0x0200 -WParam 0 -LParam $position
    Start-Sleep -Milliseconds 30
    Send-TurbofishWindowMessage -Playtest $Playtest -Message 0x0201 -WParam 1 -LParam $position
    try { Start-Sleep -Milliseconds 60 }
    finally {
        if (-not $Playtest.Process.HasExited) { Send-TurbofishWindowMessage -Playtest $Playtest -Message 0x0202 -WParam 0 -LParam $position }
    }
    Write-TurbofishInputReceipt -Playtest $Playtest -InputRecord ([ordered]@{ kind = 'click'; x = $X; y = $Y; client_x = $clientX; client_y = $clientY; reason = $Reason })
}

function Wait-TurbofishState {
    [CmdletBinding()]
    param([Parameter(Mandatory)]$Playtest, [Parameter(Mandatory)][scriptblock]$Predicate,
        [Parameter(Mandatory)][string]$Description, [ValidateRange(0.05, 3600)][double]$TimeoutSeconds = 5,
        [long]$AfterSessionTick = -1)
    $timer = [System.Diagnostics.Stopwatch]::StartNew()
    $lastSample = 'no complete snapshot'
    $lastReadError = $null
    do {
        $Playtest.Process.Refresh()
        if ($Playtest.Process.HasExited) { throw "Awaiting ${Description}: owned process exited ($($Playtest.Process.ExitCode))." }
        $snapshot = $null
        try { $snapshot = Read-TurbofishJson -Path $Playtest.StatePath }
        catch [System.IO.IOException] { $lastReadError = $_.Exception.Message }
        catch [System.ArgumentException] { $lastReadError = $_.Exception.Message }
        if ($null -ne $snapshot) {
            Assert-TurbofishSnapshot -Snapshot $snapshot
            $lastSample = "session_tick=$($snapshot.session_tick), paused=$($snapshot.paused), phase=$($snapshot.phase | ConvertTo-Json -Compress -Depth 8)"
            # Predicate errors are scenario mistakes, not transient file-read errors.
            if ($snapshot.session_tick -gt $AfterSessionTick) {
                $predicateResult = & $Predicate $snapshot
                if ($predicateResult -isnot [bool]) { throw 'State predicate must return one Boolean value.' }
                if ($predicateResult) { return $snapshot }
            }
        }
        Start-Sleep -Milliseconds 25
    } while ($timer.Elapsed.TotalSeconds -lt $TimeoutSeconds)
    throw "Timed out after ${TimeoutSeconds}s awaiting ${Description}; last $lastSample; read error: $lastReadError"
}

function Request-TurbofishCapture {
    [CmdletBinding()]
    param([Parameter(Mandatory)]$Playtest, [ValidateRange(0.1, 30)][double]$TimeoutSeconds = 3)
    Assert-TurbofishOwnedWindow -Playtest $Playtest | Out-Null
    $requestPath = Join-Path $Playtest.EvidenceDirectory 'capture.request'
    if (Test-Path -LiteralPath $requestPath) { throw 'A capture request is still pending; do not coalesce requests.' }
    $beforeFiles = @(Get-ChildItem -LiteralPath $Playtest.EvidenceDirectory -Filter 'frame-*.png' -File | ForEach-Object Name)
    New-Item -ItemType File -Path $requestPath -ErrorAction Stop | Out-Null
    $timer = [System.Diagnostics.Stopwatch]::StartNew()
    do {
        $Playtest.Process.Refresh()
        if ($Playtest.Process.HasExited) { throw 'Owned process exited before capture completion.' }
        if (-not (Test-Path -LiteralPath $requestPath)) {
            $newFrame = Get-ChildItem -LiteralPath $Playtest.EvidenceDirectory -Filter 'frame-*.png' -File |
                Where-Object { $_.Name -cnotin $beforeFiles -and $_.Length -gt 0 } | Sort-Object Name | Select-Object -First 1
            if ($null -ne $newFrame) { return $newFrame.FullName }
        }
        Start-Sleep -Milliseconds 25
    } while ($timer.Elapsed.TotalSeconds -lt $TimeoutSeconds)
    throw 'Capture request did not produce a new nonempty frame before timeout.'
}

function Get-TurbofishSinglePrecisionPaths {
    # Exact serialized f32 fields, source inventory: sim.rs and corpse actor structs.
    $paths = [System.Collections.Generic.List[string]]::new()
    foreach ($field in @('x', 'y', 'vx', 'vy', 'speed_mod', 'previous_vx')) { $paths.Add("$.board.fish[*].$field") }
    foreach ($field in @('x', 'y', 'opacity', 'vx', 'vy', 'speed_mod')) { $paths.Add("$.board.dead_fish[*].$field") }
    foreach ($field in @('x', 'y', 'vx', 'vy')) { $paths.Add("$.board.food[*].$field") }
    foreach ($actor in @('dead_oscars', 'dead_starcatchers', 'dead_grubbers', 'dead_gekkos', 'dead_breeders', 'dead_ultras', 'invasion.dead_aliens')) {
        $paths.Add("$.board.$actor[*].opacity")
    }
    return $paths.ToArray()
}

function Get-TurbofishExactNumber {
    param([string]$Text)
    $numberMatch = [regex]::Match($Text, '^(?<sign>-?)(?<whole>\d+)(?:\.(?<fraction>\d+))?(?:[eE](?<exponent>[+-]?\d+))?$')
    if (-not $numberMatch.Success) { throw "Invalid JSON number: $Text" }
    $fraction = $numberMatch.Groups['fraction'].Value
    $coefficient = ($numberMatch.Groups['whole'].Value + $fraction).TrimStart('0')
    $exponent = -[long]$fraction.Length
    if ($numberMatch.Groups['exponent'].Success) { $exponent += [long]$numberMatch.Groups['exponent'].Value }
    if ($coefficient.Length -eq 0) { return ($numberMatch.Groups['sign'].Value + '0') }
    $trimmed = $coefficient.TrimEnd('0')
    $exponent += $coefficient.Length - $trimmed.Length
    return ($numberMatch.Groups['sign'].Value + $trimmed + 'e' + $exponent)
}

function Compare-TurbofishJson {
    [CmdletBinding()]
    param([Parameter(Mandatory)][string]$ExpectedJson, [Parameter(Mandatory)][string]$ActualJson,
        [string[]]$SinglePrecisionPaths = @())
    $expectedDocument = [System.Text.Json.JsonDocument]::Parse($ExpectedJson)
    $actualDocument = $null
    try {
        $actualDocument = [System.Text.Json.JsonDocument]::Parse($ActualJson)
        $differences = [System.Collections.Generic.List[string]]::new()
        $typedEquivalent = [System.Collections.Generic.List[string]]::new()
        $singlePatterns = @($SinglePrecisionPaths | ForEach-Object { '^' + [regex]::Escape($_).Replace('\[\*]', '\[\d+\]') + '$' })
        function Compare-JsonNode {
            param([System.Text.Json.JsonElement]$ExpectedNode, [System.Text.Json.JsonElement]$ActualNode, [string]$NodePath)
            if ($ExpectedNode.ValueKind -ne $ActualNode.ValueKind) { $differences.Add("${NodePath}: type differs"); return }
            switch ($ExpectedNode.ValueKind.ToString()) {
                'Object' {
                    $expectedNames = @($ExpectedNode.EnumerateObject() | ForEach-Object Name)
                    $actualNames = @($ActualNode.EnumerateObject() | ForEach-Object Name)
                    if (@($expectedNames | Sort-Object -Unique -CaseSensitive).Count -ne $expectedNames.Count -or
                        @($actualNames | Sort-Object -Unique -CaseSensitive).Count -ne $actualNames.Count) { throw "Duplicate JSON keys at $NodePath" }
                    foreach ($name in $expectedNames) {
                        if ($name -cnotin $actualNames) { $differences.Add("${NodePath}.${name}: missing") }
                        else { Compare-JsonNode $ExpectedNode.GetProperty($name) $ActualNode.GetProperty($name) "$NodePath.$name" }
                    }
                    foreach ($name in $actualNames) { if ($name -cnotin $expectedNames) { $differences.Add("${NodePath}.${name}: unexpected") } }
                }
                'Array' {
                    if ($ExpectedNode.GetArrayLength() -ne $ActualNode.GetArrayLength()) { $differences.Add("${NodePath}: array length differs"); return }
                    for ($index = 0; $index -lt $ExpectedNode.GetArrayLength(); $index++) { Compare-JsonNode $ExpectedNode[$index] $ActualNode[$index] "$NodePath[$index]" }
                }
                'Number' {
                    $expectedText = $ExpectedNode.GetRawText(); $actualText = $ActualNode.GetRawText()
                    $isSingle = @($singlePatterns | Where-Object { $NodePath -cmatch $_ }).Count -gt 0
                    if ($isSingle) {
                        $expectedFloat = $ExpectedNode.GetSingle(); $actualFloat = $ActualNode.GetSingle()
                        if (-not [float]::IsFinite($expectedFloat) -or -not [float]::IsFinite($actualFloat)) { throw "Non-finite binary32 at $NodePath" }
                        if ([BitConverter]::SingleToInt32Bits($expectedFloat) -ne [BitConverter]::SingleToInt32Bits($actualFloat)) { $differences.Add("${NodePath}: binary32 differs ($expectedText / $actualText)") }
                        elseif ((Get-TurbofishExactNumber $expectedText) -cne (Get-TurbofishExactNumber $actualText)) { $typedEquivalent.Add($NodePath) }
                    }
                    elseif ((Get-TurbofishExactNumber $expectedText) -cne (Get-TurbofishExactNumber $actualText)) { $differences.Add("${NodePath}: number differs ($expectedText / $actualText)") }
                }
                'String' { if ($ExpectedNode.GetString() -cne $ActualNode.GetString()) { $differences.Add("${NodePath}: string differs") } }
                default { if ($ExpectedNode.GetRawText() -cne $ActualNode.GetRawText()) { $differences.Add("${NodePath}: value differs") } }
            }
        }
        Compare-JsonNode $expectedDocument.RootElement $actualDocument.RootElement '$'
        return [pscustomobject]@{ Equal = ($differences.Count -eq 0); DifferenceCount = $differences.Count
            Differences = @($differences | Select-Object -First 20); TypedEquivalentPaths = $typedEquivalent.ToArray() }
    }
    finally { $expectedDocument.Dispose(); if ($null -ne $actualDocument) { $actualDocument.Dispose() } }
}

function Assert-TurbofishReload {
    [CmdletBinding()]
    param([Parameter(Mandatory)][string]$ExpectedSessionJson, [Parameter(Mandatory)][string]$ActualSessionJson)
    # Missing or partial session pairs must not become a vacuous reload success.
    foreach ($sessionJson in @($ExpectedSessionJson, $ActualSessionJson)) {
        $sessionDocument = [System.Text.Json.JsonDocument]::Parse($sessionJson)
        try {
            if ($sessionDocument.RootElement.ValueKind -ne [System.Text.Json.JsonValueKind]::Object) { throw 'Reload requires complete non-null session objects.' }
            foreach ($sessionKey in @('progress', 'board', 'phase', 'ticks', 'next_seed')) {
                $sessionProperty = [System.Text.Json.JsonElement]::new()
                if (-not $sessionDocument.RootElement.TryGetProperty($sessionKey, [ref]$sessionProperty)) { throw "Reload session is missing '$sessionKey'." }
            }
        }
        finally { $sessionDocument.Dispose() }
    }
    $comparison = Compare-TurbofishJson -ExpectedJson $ExpectedSessionJson -ActualJson $ActualSessionJson -SinglePrecisionPaths (Get-TurbofishSinglePrecisionPaths)
    if (-not $comparison.Equal) { throw "Reload differs at $($comparison.DifferenceCount) paths: $($comparison.Differences -join '; ')" }
    return $comparison
}

function Assert-TurbofishPausedState {
    [CmdletBinding()]
    param([Parameter(Mandatory)][System.Collections.IDictionary]$Before, [Parameter(Mandatory)][System.Collections.IDictionary]$After,
        [Parameter(Mandatory)][scriptblock]$SelectState, [Parameter(Mandatory)][string]$Description)
    Assert-TurbofishSnapshot $Before; Assert-TurbofishSnapshot $After
    if (-not $Before.paused -or -not $After.paused -or $After.session_tick -le $Before.session_tick) {
        throw 'Pause check requires two paused snapshots with advancing session_tick.'
    }
    $beforeState = & $SelectState $Before; $afterState = & $SelectState $After
    if ($null -eq $beforeState -or $null -eq $afterState) { throw 'Pause projection must identify non-null state.' }
    $comparison = Compare-TurbofishJson -ExpectedJson ($beforeState | ConvertTo-Json -Depth 100 -Compress) -ActualJson ($afterState | ConvertTo-Json -Depth 100 -Compress)
    if (-not $comparison.Equal) { throw "Paused ${Description} changed: $($comparison.Differences -join '; ')" }
}

function Stop-TurbofishPlaytest {
    [CmdletBinding()]
    param([Parameter(Mandatory)]$Playtest, [ValidateRange(100, 30000)][int]$GraceMilliseconds = 10000)
    if ($null -ne $Playtest.Cleanup) { return $Playtest.Cleanup }
    $ownedProcess = $Playtest.Process
    $forced = $false
    $closeError = $null
    $ownedProcess.Refresh()
    if (-not $ownedProcess.HasExited) {
        try { Send-TurbofishWindowMessage -Playtest $Playtest -Message 0x0010 -WParam 0 -LParam 0 }
        catch { $closeError = $_.Exception.Message }
        if (-not $ownedProcess.WaitForExit($GraceMilliseconds)) {
            # Kill the retained process handle, never a reused PID or process name.
            $forced = $true
            $ownedProcess.Kill()
            if (-not $ownedProcess.WaitForExit(5000)) { throw "Owned process $($ownedProcess.Id) survived forced cleanup." }
        }
    }
    $receipt = [pscustomobject]@{ process_id = $ownedProcess.Id; exited = $ownedProcess.HasExited
        exit_code = $ownedProcess.ExitCode; forced = $forced; close_error = $closeError; finished_utc = [DateTime]::UtcNow.ToString('o') }
    $receipt | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $Playtest.RunDirectory 'cleanup.local.json') -Encoding utf8NoBOM
    $Playtest.Cleanup = $receipt
    return $receipt
}
