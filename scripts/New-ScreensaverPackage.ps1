[CmdletBinding()]
param([Parameter(Mandatory)][string]$BuildIdentityPath)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repositoryRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
. (Join-Path $PSScriptRoot 'Evidence.Common.ps1')
$buildPath = Get-EvidenceFullPath -Path $BuildIdentityPath
$buildRecord = Get-Content -LiteralPath $buildPath -Raw | ConvertFrom-Json
if ($buildRecord.schema_version -ne 1 -or $buildRecord.kind -ne 'build' -or $buildRecord.association -ne 'launcher-success-with-unchanged-inputs') { throw 'Use an associated maintained-launcher build receipt.' }
if ($buildRecord.id -notmatch '^[a-fA-F0-9]{64}$') { throw 'Invalid build identity.' }
$buildProfile = $buildRecord.configuration.profile
if ('crates/turbofish-screenhost/src/lib.rs' -notin @($buildRecord.sources.files.path)) { throw 'This build predates the native screensaver host.' }
if ($buildProfile -notin @('debug','release')) { throw 'Unsupported build profile.' }
$sourcePrefix = "target/$buildProfile/"
$packageRoot = [IO.Path]::GetFullPath((Join-Path $repositoryRoot "target/screensaver/$($buildRecord.id)"))
if (-not $packageRoot.StartsWith($repositoryRoot + '\target\screensaver\', [StringComparison]::OrdinalIgnoreCase)) { throw 'Invalid package path.' }
if (Test-Path -LiteralPath $packageRoot) { throw 'This immutable package already exists.' }
$copyRows = @()
foreach ($artifactRow in $buildRecord.artifacts) {
    if (-not $artifactRow.path.StartsWith($sourcePrefix, [StringComparison]::Ordinal)) { continue }
    $sourcePath = Get-EvidenceFullPath -Path $artifactRow.path
    if ((Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256).Hash -ne $artifactRow.sha256) { throw "Changed build artifact: $($artifactRow.path)" }
    $relativePath = $artifactRow.path.Substring($sourcePrefix.Length)
    if ($relativePath -eq 'turbofish-deluxe.exe') { $relativePath = 'TurbofishDeluxe.scr' }
    $destinationPath = [IO.Path]::GetFullPath((Join-Path $packageRoot $relativePath))
    if (-not $destinationPath.StartsWith($packageRoot + '\', [StringComparison]::OrdinalIgnoreCase)) { throw 'Package artifact escapes output.' }
    $copyRows += [ordered]@{ source=$sourcePath; destination=$destinationPath; sha256=$artifactRow.sha256 }
}
if (-not ($copyRows | Where-Object { [IO.Path]::GetFileName($_.destination) -eq 'TurbofishDeluxe.scr' })) { throw 'Runtime executable missing from receipt.' }
if (@($copyRows | Where-Object { $_.destination.EndsWith('.dll', [StringComparison]::OrdinalIgnoreCase) }).Count -ne 5) { throw 'Expected five staged runtime libraries.' }
New-Item -ItemType Directory -Path $packageRoot | Out-Null
foreach ($copyRow in $copyRows) {
    New-Item -ItemType Directory -Path ([IO.Path]::GetDirectoryName($copyRow.destination)) -Force | Out-Null
    Copy-Item -LiteralPath $copyRow.source -Destination $copyRow.destination
    if ((Get-FileHash -LiteralPath $copyRow.destination -Algorithm SHA256).Hash -ne $copyRow.sha256) { throw 'Package copy mismatch.' }
}
Copy-Item -LiteralPath (Join-Path $repositoryRoot 'crates/turbofish-screenhost/LICENSE') -Destination (Join-Path $packageRoot 'screenhost-LICENSE.txt')
[ordered]@{
    build=$buildPath; build_sha256=(Get-FileHash -LiteralPath $buildPath -Algorithm SHA256).Hash
    artifacts=$copyRows; host_license_sha256=(Get-FileHash -LiteralPath (Join-Path $packageRoot 'screenhost-LICENSE.txt') -Algorithm SHA256).Hash
    installation='not installed; no registry or Windows configuration changes'
    native_execution='not tested'
} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $packageRoot 'package.local.json') -Encoding utf8NoBOM
Write-Output "SCREENSAVER_PACKAGE path=$packageRoot artifacts=$($copyRows.Count) native_execution=not-tested installation=not-installed"
