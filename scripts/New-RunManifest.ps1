[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$RunDirectory,
    [Parameter(Mandatory)][string]$BuildIdentityPath,
    [string]$OutputPath,
    [string]$ScenarioPath,
    [string]$StartingSavePath,
    [string]$FinalSavePath,
    [string[]]$AdditionalArtifacts = @(),
    [Nullable[int]]$RunProcessId
)
. (Join-Path $PSScriptRoot 'Evidence.Common.ps1')
if (-not $OutputPath) { $OutputPath = New-EvidenceOutputName -Kind 'run' }
Assert-EvidenceOutputPath -Path $OutputPath | Out-Null
$fullRunDirectory = Get-EvidenceFullPath -Path $RunDirectory
if (-not (Test-Path -LiteralPath $fullRunDirectory -PathType Container)) { throw 'Run directory is missing.' }
$buildRecord = Read-EvidenceJson -Path $BuildIdentityPath
if ($buildRecord['schema_version'] -ne 1 -or $buildRecord['kind'] -ne 'build') { throw 'Supply a version1 build identity.' }
$buildFile = Get-EvidenceFile -Path $BuildIdentityPath -Role 'build-identity'
$artifacts = @($buildFile)
$runtimeRecords = @{}
$issues = [System.Collections.Generic.List[string]]::new()
$buildChecks = @(Get-EvidenceChecks -Record $buildRecord)
if (@($buildChecks | Where-Object { $_.status -notin @('match', 'not-captured') }).Count -gt 0) { $issues.Add('Referenced build artifacts are missing or changed') }
$eventCount = 0
foreach ($runtimeFile in @('identity.local.json', 'events.local.jsonl', 'final.local.json', 'state.local.json')) {
    $runtimePath = Join-Path $fullRunDirectory $runtimeFile
    $fileRecord = Get-EvidenceFile -Path $runtimePath -Role 'runtime-telemetry' -Required ($runtimeFile -ne 'state.local.json')
    $artifacts += $fileRecord
    if (-not $fileRecord.exists) {
        if ($fileRecord.required) { $issues.Add("Missing $runtimeFile") }
        continue
    }
    if ($runtimeFile.EndsWith('.json')) {
        try {
            $runtimeRecords[$runtimeFile] = Read-EvidenceJson -Path $runtimePath
            if ($runtimeRecords[$runtimeFile] -isnot [System.Collections.IDictionary]) { throw 'Expected a JSON object.' }
        } catch { $issues.Add("Malformed $runtimeFile"); $runtimeRecords.Remove($runtimeFile) }
    } else {
        $eventLineNumber = 0
        foreach ($eventLine in [System.IO.File]::ReadLines($runtimePath)) {
            $eventLineNumber++
            try {
                $eventRecord = ConvertFrom-Json -InputObject $eventLine -AsHashtable -Depth 100
                if ($eventRecord -isnot [System.Collections.IDictionary] -or
                    (-not $eventRecord.Contains('event') -and -not $eventRecord.Contains('music') -and -not $eventRecord.Contains('diagnostic'))) {
                    throw 'Unrecognized telemetry row.'
                }
                if (-not (Test-EvidenceNumber -Value $eventRecord['elapsed_seconds']) -or
                    (-not (Test-EvidenceNumber -Value $eventRecord['session_tick'] -Integer) -and
                     -not (Test-EvidenceNumber -Value $eventRecord['tick'] -Integer))) { throw 'Invalid event time/tick.' }
                $eventCount++
            } catch { $issues.Add("Malformed events.local.jsonl line $eventLineNumber") }
        }
    }
}
$frameGroup = Get-EvidenceFileGroup -Directory $fullRunDirectory -Filter 'frame-*.png' -Role 'frame'
$artifacts += $frameGroup.files
foreach ($saveItem in @(@{ Path = $StartingSavePath; Role = 'starting-save' }, @{ Path = $FinalSavePath; Role = 'final-save' })) {
    if ($saveItem.Path) { $artifacts += Get-EvidenceFile -Path $saveItem.Path -Role $saveItem.Role }
}
$scenario = [ordered]@{ status = 'unknown'; launch_argument_list = @(); inputs = 'unknown'; time_source = 'unknown'; expected_evidence = 'unknown'; save_origin = 'unknown' }
if ($ScenarioPath) {
    $scenario = Read-EvidenceJson -Path $ScenarioPath
    if ($scenario -isnot [System.Collections.IDictionary]) { throw 'Scenario must be a JSON object.' }
    $artifacts += Get-EvidenceFile -Path $ScenarioPath -Role 'scenario'
}
foreach ($additionalPath in $AdditionalArtifacts) { $artifacts += Get-EvidenceFile -Path $additionalPath -Role 'additional-evidence' }
$runtimeIdentity = $runtimeRecords['identity.local.json']
$runtimeFinal = $runtimeRecords['final.local.json']
$executableMatch = 'unknown'
$installationCheck = 'unknown'
if ($runtimeIdentity) {
    if (-not $runtimeIdentity['rust_executable_sha256'] -or -not $runtimeIdentity.Contains('seed') -or
        -not $runtimeIdentity.Contains('tick_ms') -or -not $runtimeIdentity.Contains('start') -or
        -not (Get-EvidenceField -Record $runtimeIdentity -Keys @('game', 'inventory_sha256'))) { $issues.Add('Incomplete runtime identity') }
    if (-not (Test-EvidenceNumber -Value $runtimeIdentity['seed'] -Integer) -or
        -not (Test-EvidenceNumber -Value $runtimeIdentity['tick_ms'] -Minimum 1 -Integer) -or
        $runtimeIdentity['retail_rng_equivalent'] -isnot [bool] -or
        $runtimeIdentity['start'] -isnot [System.Collections.IDictionary] -or
        (Get-EvidenceField -Record $runtimeIdentity -Keys @('game', 'inventory_sha256')) -notmatch '^[a-fA-F0-9]{64}$' -or
        $runtimeIdentity['rust_executable_sha256'] -notmatch '^[a-fA-F0-9]{64}$') { $issues.Add('Invalid runtime identity types or ranges') }
    $expectedExecutable = @($buildRecord['artifacts'] | Where-Object { $_['role'] -eq 'executable' })
    if ($expectedExecutable.Count -ne 1) { throw 'Build identity must contain exactly one executable.' }
    $executableMatch = if ($runtimeIdentity['rust_executable_sha256'] -eq $expectedExecutable[0]['sha256']) { 'match' } else { 'mismatch' }
    if ($executableMatch -ne 'match') { $issues.Add('Runtime executable differs from build identity') }
}
if ($runtimeFinal) {
    if (-not $runtimeFinal['session'] -or -not (Get-EvidenceField -Record $runtimeFinal -Keys @('game_after', 'inventory_sha256')) -or
        -not $runtimeFinal.Contains('elapsed_seconds') -or $runtimeFinal['game_unchanged'] -isnot [bool]) { $issues.Add('Incomplete runtime final record') }
    if (-not (Test-EvidenceNumber -Value $runtimeFinal['elapsed_seconds']) -or
        $runtimeFinal['session'] -isnot [System.Collections.IDictionary] -or
        (Get-EvidenceField -Record $runtimeFinal -Keys @('game_after', 'inventory_sha256')) -notmatch '^[a-fA-F0-9]{64}$') { $issues.Add('Invalid final telemetry types or ranges') }
    if ($runtimeIdentity) {
        $installationCheck = if ($runtimeFinal['game_unchanged'] -eq $true -and
            (Get-EvidenceField -Record $runtimeIdentity -Keys @('game', 'inventory_sha256')) -eq (Get-EvidenceField -Record $runtimeFinal -Keys @('game_after', 'inventory_sha256'))) { 'unchanged-inventory' } else { 'mismatch-or-incomplete' }
        if ($installationCheck -ne 'unchanged-inventory') { $issues.Add('Original installation inventory did not match') }
    }
}
$processCheck = 'unknown; process ID not supplied'
if ($null -ne $RunProcessId) {
    if ($RunProcessId -le 0) { throw 'Run process ID must be positive.' }
    $processCheck = if (Get-Process -Id $RunProcessId -ErrorAction SilentlyContinue) { 'PID-present; identity or cleanup requires review' } else { 'PID-absent-at-capture' }
    if ($processCheck -ne 'PID-absent-at-capture') { $issues.Add('Supplied process ID is still present') }
}
$runRecord = [ordered]@{
    schema_version = 1
    kind = 'run'
    id = $null
    captured_utc = [DateTimeOffset]::UtcNow.ToString('o')
    run_directory = Get-EvidenceRelativePath -Path $fullRunDirectory
    build_identity = [ordered]@{ id = $buildRecord['id']; path = $buildFile.path; sha256 = $buildFile.sha256 }
    scenario = $scenario
    runtime = [ordered]@{
        seed = if ($runtimeIdentity) { $runtimeIdentity['seed'] } else { $null }
        tick_ms = if ($runtimeIdentity) { $runtimeIdentity['tick_ms'] } else { $null }
        retail_rng_equivalent = if ($runtimeIdentity) { $runtimeIdentity['retail_rng_equivalent'] } else { $null }
        elapsed_seconds = if ($runtimeFinal) { $runtimeFinal['elapsed_seconds'] } else { $null }
        executable_match = $executableMatch
        installation = $installationCheck
        supplied_process_id = $RunProcessId
        process_check = $processCheck
        frame_count = $frameGroup.paths.Count
        event_rows = $eventCount
    }
    bookkeeping = [ordered]@{ status = if ($issues.Count -eq 0) { 'complete-records; behavior not assessed' } else { 'incomplete-or-inconsistent' }; issues = @($issues) }
    artifacts = $artifacts
    artifact_groups = @([ordered]@{ directory = $frameGroup.directory; filter = $frameGroup.filter; recurse = $false; role = 'frame'; paths = $frameGroup.paths })
    review = [ordered]@{ behavior = 'not tested'; discrepancies = 'not reviewed'; visuals = 'not tested'; audio = 'not tested'; human = 'not tested' }
}
$runRecord.id = Get-EvidenceRecordId -Record $runRecord
$captureFailures = @(Get-EvidenceFileChecks -Files $runRecord.artifacts | Where-Object { $_.status -notin @('match', 'not-captured', 'missing-at-capture') })
if ($captureFailures.Count -gt 0) { throw 'Run artifacts changed during capture; wait for the writer to stop and create a new manifest.' }
$finalFrameGroup = Get-EvidenceFileGroup -Directory $fullRunDirectory -Filter 'frame-*.png' -Role 'frame'
if (($finalFrameGroup.paths -join "`n") -cne ($frameGroup.paths -join "`n")) { throw 'Frame inventory changed during capture; wait for the writer to stop.' }
$writtenPath = Write-EvidenceJson -Path $OutputPath -Record $runRecord
[pscustomobject]@{ Kind = 'run'; Id = $runRecord.id; Path = $writtenPath; Sha256 = (Get-FileHash -LiteralPath $writtenPath).Hash.ToLowerInvariant(); Status = $runRecord.bookkeeping.status; Issues = @($issues) }
