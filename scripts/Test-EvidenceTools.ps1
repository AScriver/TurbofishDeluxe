# Synthetic bookkeeping contracts. No game executable is launched and no real save is edited.
[CmdletBinding()]
param()
. (Join-Path $PSScriptRoot 'Evidence.Common.ps1')
$fixtureRoot = Join-Path $EvidenceRepositoryRoot ('.scratch/evidence/tool-tests-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $fixtureRoot | Out-Null
$script:EvidenceTestCount = 0
function Assert-EvidenceTest {
    param([bool]$Condition, [string]$Description)
    if (-not $Condition) { throw "Evidence contract failed: $Description" }
    $script:EvidenceTestCount++
}
function Assert-EvidenceFailure {
    param([scriptblock]$Operation, [string]$ExpectedMessage)
    $caughtException = $null
    try { & $Operation | Out-Null } catch { $caughtException = $_ }
    Assert-EvidenceTest -Condition ($null -ne $caughtException) -Description "Expected failure: $ExpectedMessage"
    Assert-EvidenceTest -Condition ($caughtException.Exception.Message -like "*$ExpectedMessage*") -Description "Correct failure: $ExpectedMessage"
}
function Write-EvidenceFixture {
    param([string]$RelativePath, [string]$Content)
    $fixturePath = Join-Path $fixtureRoot $RelativePath
    New-Item -ItemType Directory -Path ([System.IO.Path]::GetDirectoryName($fixturePath)) -Force | Out-Null
    [System.IO.File]::WriteAllText($fixturePath, $Content, [System.Text.UTF8Encoding]::new($false))
    return $fixturePath
}
$executablePath = Write-EvidenceFixture -RelativePath 'bundle/turbofish-deluxe.exe' -Content 'synthetic executable; never launched'
foreach ($nativeName in @('libopenmpt.dll', 'openmpt-mpg123.dll', 'openmpt-ogg.dll', 'openmpt-vorbis.dll', 'openmpt-zlib.dll')) {
    Write-EvidenceFixture -RelativePath ('bundle/' + $nativeName) -Content ('synthetic ' + $nativeName) | Out-Null
}
Write-EvidenceFixture -RelativePath 'bundle/libopenmpt-notices/LICENSE.txt' -Content 'synthetic notice' | Out-Null
$buildPath = Join-Path $fixtureRoot 'build.local.json'
$buildResult = & (Join-Path $PSScriptRoot 'New-BuildIdentity.ps1') -ExecutablePath $executablePath -OutputPath $buildPath
$build = Read-EvidenceJson -Path $buildPath
Assert-EvidenceTest ($build['association'] -like '*unverified*') 'Standalone snapshot never claims a build association'
Assert-EvidenceTest (@($build['sources']['files']).Count -gt 0) 'Current tracked and untracked build inputs are recorded'
& (Join-Path $PSScriptRoot 'Test-Evidence.ps1') -RecordPath $buildPath -OutputPath (Join-Path $fixtureRoot 'build-readback.local.json') -CheckSources | Out-Null
Assert-EvidenceTest ($buildResult.Sha256 -eq (Get-EvidenceFile -Path $buildPath -Role 'test').sha256) 'Written build receipt digest matches readback'
Assert-EvidenceFailure { & (Join-Path $PSScriptRoot 'New-BuildIdentity.ps1') -ExecutablePath $executablePath -OutputPath $buildPath } 'immutable'
Assert-EvidenceFailure { Write-EvidenceText -Path (Join-Path $fixtureRoot '../../../../../outside.local.json') -Text 'blocked' } 'private project files'
Assert-EvidenceFailure { Write-EvidenceText -Path 'docs/playtests/TEMPLATE.md' -Text 'blocked' -Report } 'private project files'
Write-EvidenceFixture -RelativePath 'bundle/turbofish-deluxe.exe' -Content 'tampered executable' | Out-Null
Assert-EvidenceFailure { & (Join-Path $PSScriptRoot 'Test-Evidence.ps1') -RecordPath $buildPath -OutputPath (Join-Path $fixtureRoot 'tampered-readback.local.json') } 'readback failed'
$tamperReadback = Read-EvidenceJson -Path (Join-Path $fixtureRoot 'tampered-readback.local.json')
Assert-EvidenceTest (@($tamperReadback['checks'] | Where-Object { $_['status'] -eq 'changed' }).Count -eq 1) 'Changed executable is identified'
Write-EvidenceFixture -RelativePath 'bundle/turbofish-deluxe.exe' -Content 'synthetic executable; never launched' | Out-Null
$removedLibrary = Join-Path $fixtureRoot 'bundle/openmpt-ogg.dll'
Remove-Item -LiteralPath $removedLibrary
$missingChecks = @(Get-EvidenceChecks -Record $build)
Assert-EvidenceTest (@($missingChecks | Where-Object { $_.status -eq 'missing' }).Count -eq 1) 'Missing dependency is identified'
Write-EvidenceFixture -RelativePath 'bundle/openmpt-ogg.dll' -Content 'synthetic openmpt-ogg.dll' | Out-Null
$extraNotice = Write-EvidenceFixture -RelativePath 'bundle/libopenmpt-notices/extra.txt' -Content 'extra'
$extraChecks = @(Get-EvidenceChecks -Record $build)
Assert-EvidenceTest (@($extraChecks | Where-Object { $_.status -eq 'added' }).Count -eq 1) 'Added native notice changes the inventory'
Remove-Item -LiteralPath $extraNotice
$gameDigest = 'a' * 64
$identity = [ordered]@{
    game = @{ inventory_sha256 = $gameDigest }
    rust_executable_sha256 = (Get-EvidenceFile -Path $executablePath -Role 'test').sha256
    seed = 42; tick_ms = 28; retail_rng_equivalent = $false
    start = @{ board = $null; progress = @{ stage = 'fixture-only' } }
}
$final = [ordered]@{ session = @{ board = $null; tick = 1 }; game_after = @{ inventory_sha256 = $gameDigest }; elapsed_seconds = 0.028; game_unchanged = $true }
$runDirectory = Join-Path $fixtureRoot 'run'
Write-EvidenceFixture -RelativePath 'run/identity.local.json' -Content ($identity | ConvertTo-Json -Depth 20) | Out-Null
Write-EvidenceFixture -RelativePath 'run/final.local.json' -Content ($final | ConvertTo-Json -Depth 20) | Out-Null
Write-EvidenceFixture -RelativePath 'run/events.local.jsonl' -Content '{"elapsed_seconds":0.028,"session_tick":1,"event":{"Fixture":true}}' | Out-Null
Write-EvidenceFixture -RelativePath 'run/frame-1.png' -Content 'synthetic bytes, not an image' | Out-Null
$scenarioPath = Write-EvidenceFixture -RelativePath 'scenario.local.json' -Content (@{ launch_argument_list = @('--evidence-dir', $runDirectory); inputs = 'fixture'; time_source = 'synthetic'; expected_evidence = 'explicit tooling contract'; save_origin = 'synthetic; not gameplay' } | ConvertTo-Json)
$manifestPath = Join-Path $fixtureRoot 'run.local.json'
& (Join-Path $PSScriptRoot 'New-RunManifest.ps1') -RunDirectory $runDirectory -BuildIdentityPath $buildPath -ScenarioPath $scenarioPath -OutputPath $manifestPath | Out-Null
$manifest = Read-EvidenceJson -Path $manifestPath
Assert-EvidenceTest ($manifest['bookkeeping']['issues'].Count -eq 0) 'Complete synthetic telemetry has no bookkeeping issue'
Assert-EvidenceTest ($manifest['review']['behavior'] -eq 'not tested') 'Complete records do not promote behavior acceptance'
Assert-EvidenceTest ($manifest['runtime']['frame_count'] -eq 1) 'Frame bytes are inventoried'
& (Join-Path $PSScriptRoot 'Test-Evidence.ps1') -RecordPath $manifestPath -OutputPath (Join-Path $fixtureRoot 'run-readback.local.json') -CheckSources | Out-Null
$reportPath = Join-Path $fixtureRoot 'draft.md'
& (Join-Path $PSScriptRoot 'New-PlaytestReport.ps1') -RunManifestPath $manifestPath -OutputPath $reportPath -Executor 'agent' -Goal 'synthetic tooling contract' | Out-Null
$reportText = Get-Content -LiteralPath $reportPath -Raw
Assert-EvidenceTest ($reportText.Contains('Result: not tested')) 'Draft defaults to not tested'
Assert-EvidenceTest ($reportText.Contains($build['id']) -and $reportText.Contains($manifest['id'])) 'Draft includes build and run identity'
Assert-EvidenceTest ($reportText -match ('(?m)^- Build receipt ID: ' + $build['id'] + '\.$')) 'Bookkeeping bullet is one complete Markdown line'
Assert-EvidenceTest ($reportText -match '(?m)^- Artifact readback: passed at draft generation\.$') 'Artifact summary is one complete Markdown line'
Assert-EvidenceTest (-not $reportText.Contains($fixtureRoot)) 'Draft does not export private absolute paths from the scenario'
Assert-EvidenceFailure { & (Join-Path $PSScriptRoot 'New-PlaytestReport.ps1') -RunManifestPath $manifestPath -OutputPath $reportPath } 'immutable'
Write-EvidenceFixture -RelativePath 'bundle/turbofish-deluxe.exe' -Content 'transitively changed executable' | Out-Null
Assert-EvidenceFailure { & (Join-Path $PSScriptRoot 'Test-Evidence.ps1') -RecordPath $manifestPath -OutputPath (Join-Path $fixtureRoot 'transitive-readback.local.json') } 'readback failed'
$changedReportPath = Join-Path $fixtureRoot 'changed-build-draft.md'
$changedDraft = & (Join-Path $PSScriptRoot 'New-PlaytestReport.ps1') -RunManifestPath $manifestPath -OutputPath $changedReportPath
Assert-EvidenceTest ($changedDraft.ArtifactReadback -like '*failed*') 'Draft flags changed transitive build artifacts'
$staleBuildRunPath = Join-Path $fixtureRoot 'stale-build-run.local.json'
& (Join-Path $PSScriptRoot 'New-RunManifest.ps1') -RunDirectory $runDirectory -BuildIdentityPath $buildPath -OutputPath $staleBuildRunPath | Out-Null
$staleBuildRun = Read-EvidenceJson -Path $staleBuildRunPath
Assert-EvidenceTest ($staleBuildRun['bookkeeping']['issues'] -contains 'Referenced build artifacts are missing or changed') 'Run manifest retains a known stale-build discrepancy'
Write-EvidenceFixture -RelativePath 'bundle/turbofish-deluxe.exe' -Content 'synthetic executable; never launched' | Out-Null
$tamperedManifest = Read-EvidenceJson -Path $manifestPath
$tamperedManifest['runtime']['seed'] = 99
$tamperedManifestPath = Join-Path $fixtureRoot 'tampered-manifest.local.json'
Write-EvidenceJson -Path $tamperedManifestPath -Record $tamperedManifest | Out-Null
Assert-EvidenceFailure { & (Join-Path $PSScriptRoot 'Test-Evidence.ps1') -RecordPath $tamperedManifestPath -OutputPath (Join-Path $fixtureRoot 'receipt-tamper-readback.local.json') } 'ID does not match'
$invalidIdentity = Read-EvidenceJson -Path (Join-Path $runDirectory 'identity.local.json')
$invalidIdentity['seed'] = 'bad-seed'
$invalidIdentity['tick_ms'] = -1
$invalidFinal = Read-EvidenceJson -Path (Join-Path $runDirectory 'final.local.json')
$invalidFinal['elapsed_seconds'] = 'bad-elapsed'
Write-EvidenceFixture -RelativePath 'run/identity.local.json' -Content ($invalidIdentity | ConvertTo-Json -Depth 20) | Out-Null
Write-EvidenceFixture -RelativePath 'run/final.local.json' -Content ($invalidFinal | ConvertTo-Json -Depth 20) | Out-Null
$invalidTypesPath = Join-Path $fixtureRoot 'invalid-types.local.json'
& (Join-Path $PSScriptRoot 'New-RunManifest.ps1') -RunDirectory $runDirectory -BuildIdentityPath $buildPath -OutputPath $invalidTypesPath | Out-Null
$invalidTypes = Read-EvidenceJson -Path $invalidTypesPath
Assert-EvidenceTest (@($invalidTypes['bookkeeping']['issues']).Count -eq 2) 'Invalid seed/tick/elapsed scalar types and ranges are retained'
Write-EvidenceFixture -RelativePath 'run/identity.local.json' -Content ($identity | ConvertTo-Json -Depth 20) | Out-Null
Write-EvidenceFixture -RelativePath 'run/final.local.json' -Content ($final | ConvertTo-Json -Depth 20) | Out-Null
Write-EvidenceFixture -RelativePath 'run/frame-1.png' -Content 'changed frame' | Out-Null
Assert-EvidenceFailure { & (Join-Path $PSScriptRoot 'Test-Evidence.ps1') -RecordPath $manifestPath -OutputPath (Join-Path $fixtureRoot 'frame-readback.local.json') } 'readback failed'
Write-EvidenceFixture -RelativePath 'run/frame-1.png' -Content 'synthetic bytes, not an image' | Out-Null
Remove-Item -LiteralPath (Join-Path $runDirectory 'final.local.json')
$incompletePath = Join-Path $fixtureRoot 'incomplete.local.json'
& (Join-Path $PSScriptRoot 'New-RunManifest.ps1') -RunDirectory $runDirectory -BuildIdentityPath $buildPath -OutputPath $incompletePath | Out-Null
$incomplete = Read-EvidenceJson -Path $incompletePath
Assert-EvidenceTest ($incomplete['bookkeeping']['status'] -eq 'incomplete-or-inconsistent') 'No final record remains incomplete'
Assert-EvidenceFailure { & (Join-Path $PSScriptRoot 'Test-Evidence.ps1') -RecordPath $incompletePath -OutputPath (Join-Path $fixtureRoot 'incomplete-readback.local.json') } 'readback failed'
Write-EvidenceFixture -RelativePath 'run/final.local.json' -Content '{}' | Out-Null
Write-EvidenceFixture -RelativePath 'run/identity.local.json' -Content '{}' | Out-Null
Write-EvidenceFixture -RelativePath 'run/events.local.jsonl' -Content '{bad json' | Out-Null
$malformedPath = Join-Path $fixtureRoot 'malformed.local.json'
& (Join-Path $PSScriptRoot 'New-RunManifest.ps1') -RunDirectory $runDirectory -BuildIdentityPath $buildPath -OutputPath $malformedPath | Out-Null
$malformed = Read-EvidenceJson -Path $malformedPath
Assert-EvidenceTest (@($malformed['bookkeeping']['issues']).Count -ge 3) 'Malformed and structurally empty telemetry remains visible'
$identity.rust_executable_sha256 = 'b' * 64
$final.game_after.inventory_sha256 = 'c' * 64
Write-EvidenceFixture -RelativePath 'run/identity.local.json' -Content ($identity | ConvertTo-Json -Depth 20) | Out-Null
Write-EvidenceFixture -RelativePath 'run/final.local.json' -Content ($final | ConvertTo-Json -Depth 20) | Out-Null
Write-EvidenceFixture -RelativePath 'run/events.local.jsonl' -Content '{"elapsed_seconds":0.028,"session_tick":1,"diagnostic":"fixture"}' | Out-Null
$mismatchPath = Join-Path $fixtureRoot 'mismatch.local.json'
& (Join-Path $PSScriptRoot 'New-RunManifest.ps1') -RunDirectory $runDirectory -BuildIdentityPath $buildPath -RunProcessId $PID -OutputPath $mismatchPath | Out-Null
$mismatch = Read-EvidenceJson -Path $mismatchPath
Assert-EvidenceTest (@($mismatch['bookkeeping']['issues']).Count -eq 3) 'Executable/install/PID discrepancies are retained'
# Source-set tests are isolated in their own Git repository, never in maintained source.
$sourceRepository = Join-Path $fixtureRoot 'source-repo'
Write-EvidenceFixture -RelativePath 'source-repo/src/lib.rs' -Content 'fixture source' | Out-Null
Write-EvidenceFixture -RelativePath 'source-repo/.gitignore' -Content '.scratch/' | Out-Null
$gitProgram = (Get-Command git -ErrorAction Stop).Source
Invoke-EvidenceCommand -Executable $gitProgram -ArgumentList @('init', '--quiet', $sourceRepository) | Out-Null
Invoke-EvidenceGit -RepositoryRoot $sourceRepository -ArgumentList @('add', '--', '.gitignore', 'src/lib.rs') | Out-Null
Write-EvidenceFixture -RelativePath 'source-repo/src/extra.rs' -Content 'eligible untracked source' | Out-Null
Write-EvidenceFixture -RelativePath 'source-repo/.scratch/private.rs' -Content 'excluded source' | Out-Null
$isolatedSources = Get-EvidenceSources -RepositoryRoot $sourceRepository
Assert-EvidenceTest ($isolatedSources.files.Count -eq 2) 'Tracked and eligible untracked source included, private source excluded'
$sameSources = Get-EvidenceSources -RepositoryRoot $sourceRepository
Assert-EvidenceTest ($sameSources.sha256 -eq $isolatedSources.sha256) 'Unchanged source-set digest is deterministic'
Write-EvidenceFixture -RelativePath 'source-repo/src/lib.rs' -Content 'modified source' | Out-Null
$changedSources = Get-EvidenceSources -RepositoryRoot $sourceRepository
Assert-EvidenceTest ($changedSources.sha256 -ne $isolatedSources.sha256) 'Changed source alters identity'
Remove-Item -LiteralPath (Join-Path $sourceRepository 'src/lib.rs')
$deletedSources = Get-EvidenceSources -RepositoryRoot $sourceRepository
Assert-EvidenceTest (@($deletedSources.files | Where-Object { -not $_.exists }).Count -eq 1) 'Deleted tracked source is detected'
Write-EvidenceFixture -RelativePath 'source-repo/src/new.rs' -Content 'new source' | Out-Null
$addedSources = Get-EvidenceSources -RepositoryRoot $sourceRepository
Assert-EvidenceTest ($addedSources.files.Count -eq 3) 'New eligible source is detected'
$sourceFixtureReceipt = [ordered]@{
    schema_version = 1; kind = 'build'; id = $null
    artifacts = @(Get-EvidenceFile -Path '.gitignore' -Role 'fixture' -RepositoryRoot $sourceRepository)
    artifact_groups = @(); sources = $isolatedSources
}
$sourceFixtureReceipt.id = Get-EvidenceRecordId -Record $sourceFixtureReceipt
$staleSourceChecks = @(Get-EvidenceChecks -Record $sourceFixtureReceipt -CheckSources -RepositoryRoot $sourceRepository)
Assert-EvidenceTest (@($staleSourceChecks | Where-Object { $_.role -eq 'build-input' -and $_.status -eq 'missing' }).Count -eq 1) 'Source readback reports a deleted input'
Assert-EvidenceTest (@($staleSourceChecks | Where-Object { $_.role -eq 'build-input' -and $_.status -eq 'added' }).Count -eq 1) 'Source readback reports a newly eligible input'
$quietIdentity = $identity
$quietIdentity.rust_executable_sha256 = (Get-EvidenceFile -Path $executablePath -Role 'test').sha256
$quietFinal = $final
$quietFinal.game_after.inventory_sha256 = $gameDigest
Write-EvidenceFixture -RelativePath 'run/identity.local.json' -Content ($quietIdentity | ConvertTo-Json -Depth 20) | Out-Null
Write-EvidenceFixture -RelativePath 'run/final.local.json' -Content ($quietFinal | ConvertTo-Json -Depth 20) | Out-Null
Write-EvidenceFixture -RelativePath 'run/events.local.jsonl' -Content '' | Out-Null
$quietManifestPath = Join-Path $fixtureRoot 'quiet-run.local.json'
& (Join-Path $PSScriptRoot 'New-RunManifest.ps1') -RunDirectory $runDirectory -BuildIdentityPath $buildPath -OutputPath $quietManifestPath | Out-Null
$quietManifest = Read-EvidenceJson -Path $quietManifestPath
Assert-EvidenceTest ($quietManifest['runtime']['event_rows'] -eq 0 -and @($quietManifest['bookkeeping']['issues']).Count -eq 0) 'Quiet empty-event run is explicit and does not invent events'
Assert-EvidenceTest ($quietManifest['review']['behavior'] -eq 'not tested') 'Quiet complete records still require behavior review'
Assert-EvidenceTest ($quietManifest['runtime']['time_mode'] -eq 'unrecorded' -and $null -eq $quietManifest['runtime']['test_speed']) 'Historical telemetry never invents a normal-speed claim'

$speedIdentity = $quietIdentity | ConvertTo-Json -Depth 20 | ConvertFrom-Json -AsHashtable -Depth 20
$speedFinal = $quietFinal | ConvertTo-Json -Depth 20 | ConvertFrom-Json -AsHashtable -Depth 20
$speedIdentity['test_speed'] = 4; $speedIdentity['time_mode'] = 'accelerated-test'; $speedIdentity['session_elapsed_seconds'] = 0.0
$speedFinal['test_speed'] = 4; $speedFinal['time_mode'] = 'accelerated-test'; $speedFinal['session_elapsed_seconds'] = 0.112
Write-EvidenceFixture -RelativePath 'speed-run/identity.local.json' -Content ($speedIdentity | ConvertTo-Json -Depth 20) | Out-Null
Write-EvidenceFixture -RelativePath 'speed-run/final.local.json' -Content ($speedFinal | ConvertTo-Json -Depth 20) | Out-Null
$speedEvent = '{"elapsed_seconds":0.028,"session_tick":4,"session_elapsed_seconds":0.112,"test_speed":4,"time_mode":"accelerated-test","event":{"Fixture":true}}'
Write-EvidenceFixture -RelativePath 'speed-run/events.local.jsonl' -Content $speedEvent | Out-Null
$speedRunDirectory = Join-Path $fixtureRoot 'speed-run'
$speedManifestPath = Join-Path $fixtureRoot 'speed-run.local.json'
& (Join-Path $PSScriptRoot 'New-RunManifest.ps1') -RunDirectory $speedRunDirectory -BuildIdentityPath $buildPath -OutputPath $speedManifestPath | Out-Null
$speedManifest = Read-EvidenceJson -Path $speedManifestPath
Assert-EvidenceTest ($speedManifest['runtime']['test_speed'] -eq 4 -and $speedManifest['runtime']['time_mode'] -eq 'accelerated-test' -and @($speedManifest['bookkeeping']['issues']).Count -eq 0) 'Accelerated timing is explicitly retained'
$speedReportPath = Join-Path $fixtureRoot 'speed-draft.md'
& (Join-Path $PSScriptRoot 'New-PlaytestReport.ps1') -RunManifestPath $speedManifestPath -OutputPath $speedReportPath | Out-Null
$speedReportText = Get-Content -LiteralPath $speedReportPath -Raw
Assert-EvidenceTest ($speedReportText.Contains('Test speed: 4x') -and $speedReportText.Contains('normal-speed acceptance remains separate') -and $speedReportText.Contains('Result: not tested')) 'Accelerated draft cannot silently promote normal-speed acceptance'
$speedFinal['test_speed'] = 1; $speedFinal['time_mode'] = 'normal'
Write-EvidenceFixture -RelativePath 'speed-run/final.local.json' -Content ($speedFinal | ConvertTo-Json -Depth 20) | Out-Null
$speedMismatchPath = Join-Path $fixtureRoot 'speed-mismatch.local.json'
& (Join-Path $PSScriptRoot 'New-RunManifest.ps1') -RunDirectory $speedRunDirectory -BuildIdentityPath $buildPath -OutputPath $speedMismatchPath | Out-Null
$speedMismatch = Read-EvidenceJson -Path $speedMismatchPath
Assert-EvidenceTest (@($speedMismatch['bookkeeping']['issues'] | Where-Object { $_ -like '*timing differs from identity*' }).Count -eq 1) 'Mixed normal/accelerated final timing is retained as an issue'
$speedFinal['test_speed'] = 4; $speedFinal['time_mode'] = 'accelerated-test'
Write-EvidenceFixture -RelativePath 'speed-run/final.local.json' -Content ($speedFinal | ConvertTo-Json -Depth 20) | Out-Null
Write-EvidenceFixture -RelativePath 'speed-run/events.local.jsonl' -Content ($speedEvent.Replace('"test_speed":4', '"test_speed":"4"')) | Out-Null
$invalidSpeedPath = Join-Path $fixtureRoot 'invalid-speed.local.json'
& (Join-Path $PSScriptRoot 'New-RunManifest.ps1') -RunDirectory $speedRunDirectory -BuildIdentityPath $buildPath -OutputPath $invalidSpeedPath | Out-Null
$invalidSpeed = Read-EvidenceJson -Path $invalidSpeedPath
Assert-EvidenceTest (@($invalidSpeed['bookkeeping']['issues'] | Where-Object { $_ -like '*Invalid test speed in event*' }).Count -eq 1) 'String-valued speed cannot validate as an integer'
Write-EvidenceFixture -RelativePath 'speed-run/events.local.jsonl' -Content $speedEvent | Out-Null
$speedFinal.Remove('time_mode'); $speedFinal.Remove('test_speed'); $speedFinal.Remove('session_elapsed_seconds')
Write-EvidenceFixture -RelativePath 'speed-run/final.local.json' -Content ($speedFinal | ConvertTo-Json -Depth 20) | Out-Null
$missingSpeedPath = Join-Path $fixtureRoot 'missing-speed.local.json'
& (Join-Path $PSScriptRoot 'New-RunManifest.ps1') -RunDirectory $speedRunDirectory -BuildIdentityPath $buildPath -OutputPath $missingSpeedPath | Out-Null
$missingSpeed = Read-EvidenceJson -Path $missingSpeedPath
Assert-EvidenceTest (@($missingSpeed['bookkeeping']['issues']).Count -gt 0) 'Identified accelerated runs require timing labels in final telemetry'

$testReceiptPath = Join-Path $fixtureRoot 'tests.local.json'
$testReceipt = [ordered]@{ result = 'passed'; contracts = $script:EvidenceTestCount; captured_utc = [DateTimeOffset]::UtcNow.ToString('o'); fixture_directory = Get-EvidenceRelativePath -Path $fixtureRoot; scope = 'synthetic bookkeeping only; no gameplay or native execution' }
Write-EvidenceJson -Path $testReceiptPath -Record $testReceipt | Out-Null
[pscustomobject]@{ Result = 'passed'; Contracts = $script:EvidenceTestCount; Receipt = $testReceiptPath; Gameplay = 'not tested' }
