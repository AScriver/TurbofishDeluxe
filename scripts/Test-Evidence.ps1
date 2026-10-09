[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$RecordPath,
    [string]$OutputPath,
    [switch]$CheckSources
)
. (Join-Path $PSScriptRoot 'Evidence.Common.ps1')
if (-not $OutputPath) { $OutputPath = New-EvidenceOutputName -Kind 'readback' }
Assert-EvidenceOutputPath -Path $OutputPath | Out-Null
$recordFile = Get-EvidenceFile -Path $RecordPath -Role 'checked-record'
$record = Read-EvidenceJson -Path $RecordPath
$checkRows = @(Get-EvidenceChecks -Record $record -CheckSources:$CheckSources)
$failedRows = @($checkRows | Where-Object { $_.status -notin @('match', 'not-captured') })
$recordIssues = @()
if ($record['kind'] -eq 'run') { $recordIssues = @($record['bookkeeping']['issues']) }
if (-not $recordFile.exists -or $recordFile.sha256 -ne (Get-EvidenceFile -Path $RecordPath -Role 'checked-record').sha256) {
    throw 'Receipt changed during readback.'
}
$passed = $failedRows.Count -eq 0 -and $recordIssues.Count -eq 0
$readback = [ordered]@{
    schema_version = 1
    kind = 'readback'
    captured_utc = [DateTimeOffset]::UtcNow.ToString('o')
    record = $recordFile
    record_id = $record['id']
    sources_checked = [bool]$CheckSources
    result = if ($passed) { 'passed' } else { 'failed' }
    checks = $checkRows
    issues = $recordIssues
    behavior = 'not assessed; artifact readback does not establish acceptance'
}
$writtenPath = Write-EvidenceJson -Path $OutputPath -Record $readback
[pscustomobject]@{ Result = $readback.result; CheckedFiles = $checkRows.Count; FailedFiles = $failedRows.Count; Issues = $recordIssues; Path = $writtenPath; Sha256 = (Get-FileHash -LiteralPath $writtenPath).Hash.ToLowerInvariant() }
if (-not $passed) { throw "Evidence readback failed; inspect $writtenPath" }
