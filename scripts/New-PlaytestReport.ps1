[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$RunManifestPath,
    [Parameter(Mandatory)][string]$OutputPath,
    [string]$Executor = 'unknown',
    [string]$Goal = 'unknown'
)
. (Join-Path $PSScriptRoot 'Evidence.Common.ps1')
Assert-EvidenceOutputPath -Path $OutputPath -Report | Out-Null
$manifest = Read-EvidenceJson -Path $RunManifestPath
if ($manifest['kind'] -ne 'run' -or $manifest['schema_version'] -ne 1) { throw 'Supply a version1 run manifest.' }
$build = Read-EvidenceJson -Path $manifest['build_identity']['path']
if ($build['kind'] -ne 'build' -or $build['schema_version'] -ne 1) { throw 'Manifest does not reference a version1 build identity.' }
$templatePath = Join-Path $EvidenceRepositoryRoot 'docs/playtests/TEMPLATE.md'
$reportText = Get-Content -LiteralPath $templatePath -Raw
function ConvertTo-ReportText {
    param($Value)
    if ($null -eq $Value -or [string]::IsNullOrEmpty([string]$Value)) { return 'unknown' }
    return ([string]$Value -replace '[\r\n\t`|<>\[\]()]', ' ').Trim()
}
$reportText = $reportText.Replace('# Playtest / comparison report template', '# Playtest / comparison report draft')
$reportText = $reportText.Replace('Copy this to a new dated `.md` report in this directory.', 'Generated from the maintained template; complete the reproduction steps and review before treating this as a report.')
$reportText = $reportText.Replace('- Date/time/timezone: unknown.', '- Date/time/timezone: ' + (ConvertTo-ReportText $manifest['captured_utc']) + ' (UTC receipt time).')
$reportText = $reportText.Replace('- Executor: unknown;', '- Executor: ' + (ConvertTo-ReportText $Executor) + ';')
$reportText = $reportText.Replace('- Goal / claim IDs / milestone: unknown.', '- Goal / claim IDs / milestone: ' + (ConvertTo-ReportText $Goal) + '.')
$executable = @($build['artifacts'] | Where-Object { $_['role'] -eq 'executable' })
if ($executable.Count -ne 1) { throw 'Build identity must identify one executable.' }
$sourceCount = @($build['sources']['files']).Count
$buildSummary = '- Commit: ' + (ConvertTo-ReportText $build['repository']['commit']) + '; branch: ' + (ConvertTo-ReportText $build['repository']['branch']) + '; ' + $sourceCount + ' build inputs; source-set SHA-256: ' + $build['sources']['sha256'] + '. Dirty entries and per-file hashes remain in the private build receipt.'
$reportText = $reportText.Replace('- Commit, branch, dirty tracked changes, relevant untracked source digests: unknown.', $buildSummary)
$reportText = $reportText.Replace('- Executable SHA-256; Cargo.lock/toolchain identity; build profile/features and exact build command: unknown.',
    '- Executable SHA-256: ' + $executable[0]['sha256'] + '; profile: ' + (ConvertTo-ReportText $build['configuration']['profile']) + '; Cargo.lock, exact arguments, environment and full toolchain identity are in the private build receipt. Build association: ' + (ConvertTo-ReportText $build['association']) + '.')
$reportText = $reportText.Replace('- Input sequence, seed/RNG state, time source, tick/update cadence, event-order instrumentation: unknown.',
    '- Seed: ' + (ConvertTo-ReportText $manifest['runtime']['seed']) + '; fixed tick ms: ' + (ConvertTo-ReportText $manifest['runtime']['tick_ms']) + '; elapsed seconds: ' + (ConvertTo-ReportText $manifest['runtime']['elapsed_seconds']) + '. Exact inputs, RNG state, time source and normal-speed evidence still require review of the private scenario and telemetry.')
$reportText = $reportText.Replace('- Test speed, timing mode, wall elapsed and fixed-step session elapsed: unknown. Accelerated runs are exploratory/stress checks; normal-speed acceptance remains separate.',
    '- Test speed: ' + (ConvertTo-ReportText $manifest['runtime']['test_speed']) + 'x; timing mode: ' + (ConvertTo-ReportText $manifest['runtime']['time_mode']) + '; wall elapsed seconds: ' + (ConvertTo-ReportText $manifest['runtime']['elapsed_seconds']) + '; fixed-step session elapsed seconds: ' + (ConvertTo-ReportText $manifest['runtime']['session_elapsed_seconds']) + ' (includes paused session updates). Accelerated runs are exploratory/stress checks; normal-speed acceptance remains separate.')
if ($build['original_game']['sha256']) {
    $reportText = $reportText.Replace('- Store/distribution, version, executable name, PE architecture, SHA-256: unknown.',
        '- Original executable: ' + (ConvertTo-ReportText $build['original_game']['executable_name']) + '; file version: ' + (ConvertTo-ReportText $build['original_game']['file_version']) + '; SHA-256: ' + $build['original_game']['sha256'] + '. Store/distribution and PE architecture: unknown; file metadata alone does not establish compatibility.')
}
$manifestFile = Get-EvidenceFile -Path $RunManifestPath -Role 'run-manifest'
$checkRows = @(Get-EvidenceChecks -Record $manifest)
$failedRows = @($checkRows | Where-Object { $_.status -notin @('match', 'not-captured') })
$integrityResult = if ($failedRows.Count -eq 0 -and @($manifest['bookkeeping']['issues']).Count -eq 0) { 'passed at draft generation' } else { 'failed or incomplete; inspect private records' }
$receiptLines = @(
    '', '## Automated bookkeeping', '',
    'These fields are mechanically captured. This draft does not assess behavior or discrepancies. Raw records and hashes are local-only; reproduce using the evidence commands and the scenario steps filled in above.', '',
    ('- Build receipt ID: ' + $build['id'] + '.'),
    ('- Run receipt ID: ' + $manifest['id'] + '; SHA-256: ' + $manifestFile.sha256 + '.'),
    ('- Artifact readback: ' + $integrityResult + '.'),
    ('- Runtime executable identity: ' + (ConvertTo-ReportText $manifest['runtime']['executable_match']) + '.'),
    ('- Original installation: ' + (ConvertTo-ReportText $manifest['runtime']['installation']) + '.'),
    ('- Owned process check: ' + (ConvertTo-ReportText $manifest['runtime']['process_check']) + '.'),
    ('- Recorded frames: ' + $manifest['runtime']['frame_count'] + '; visual review: not tested.'),
    '', '| Artifact role | Captured files | Missing required |', '|---|---:|---:|'
)
foreach ($roleGroup in ($manifest['artifacts'] | Group-Object { $_['role'] } | Sort-Object Name)) {
    $missingCount = @($roleGroup.Group | Where-Object { $_['required'] -and -not $_['exists'] }).Count
    $capturedCount = @($roleGroup.Group | Where-Object { $_['exists'] }).Count
    $receiptLines += '| ' + (ConvertTo-ReportText $roleGroup.Name) + ' | ' + $capturedCount + ' | ' + $missingCount + ' |'
}
# Relative template links must remain valid even for drafts in ignored scratch space.
$reportDirectory = [System.IO.Path]::GetDirectoryName((Get-EvidenceFullPath -Path $OutputPath))
$templateDirectory = [System.IO.Path]::GetDirectoryName($templatePath)
$reportText = [regex]::Replace($reportText, '(?<!!)\[[^\]\r\n]+\]\((?<target>[^)\r\n]+)\)', {
    param($linkMatch)
    $linkTarget = $linkMatch.Groups['target'].Value
    if ($linkTarget -match '^[a-zA-Z][a-zA-Z0-9+.-]*:') { return $linkMatch.Value }
    $linkParts = $linkTarget -split '#', 2
    $targetPath = [System.IO.Path]::GetFullPath((Join-Path $templateDirectory $linkParts[0]))
    $newTarget = [System.IO.Path]::GetRelativePath($reportDirectory, $targetPath).Replace('\', '/')
    if ($linkParts.Count -gt 1) { $newTarget += '#' + $linkParts[1] }
    return $linkMatch.Value.Replace('(' + $linkTarget + ')', '(' + $newTarget + ')')
})
$writtenPath = Write-EvidenceText -Path $OutputPath -Text ($reportText.TrimEnd() + "`n" + ($receiptLines -join "`n")) -Report
[pscustomobject]@{ Kind = 'report-draft'; Path = $writtenPath; Sha256 = (Get-FileHash -LiteralPath $writtenPath).Hash.ToLowerInvariant(); Verdict = 'not tested'; ArtifactReadback = $integrityResult }
