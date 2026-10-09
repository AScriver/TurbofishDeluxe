[CmdletBinding()]
param(
    [switch]$Release,
    [switch]$PrepareOnly,
    [switch]$PreflightOnly,
    [switch]$Offline,
    [string]$NativePackageDirectory,
    [string]$BuildIdentityPath,
    [string[]]$BuildEvidencePaths = @(),
    [string[]]$GameArguments = @()
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
. (Join-Path $PSScriptRoot 'Evidence.Common.ps1')
if ($BuildIdentityPath) { Assert-EvidenceOutputPath -Path $BuildIdentityPath | Out-Null }
$cacheRoot = Join-Path $repositoryRoot '.scratch/native/libopenmpt-0.8.9'
$archivePath = Join-Path $cacheRoot 'libopenmpt-0.8.9+release.dev.windows.vs2022.zip'
$cachedPackageRoot = Join-Path $cacheRoot 'package'
$packageRoot = $cachedPackageRoot
$downloadUrl = 'https://lib.openmpt.org/files/libopenmpt/dev/libopenmpt-0.8.9+release.dev.windows.vs2022.zip'
$archiveDigest = 'A23E375E576E3A6997FFEEB6CF34488E9020E2A5D8226856B02628B6A87C5E8E'
$importDigest = '537758E3D065281E887C0D3310BE471FCB322DE82F74F5DF969624D7A16156CF'
$nativeFiles = @(
    @{ Name = 'libopenmpt.dll'; Digest = '9FA3131204F44A3B157F22F9F0AE11C5CCCB5BEFA37E91A2476567AA0F4B0D94' },
    @{ Name = 'openmpt-mpg123.dll'; Digest = 'F15EDD24576D88FA08A153C4A7F9D9D7956449E4FC67758D660AD35D9E1696DF' },
    @{ Name = 'openmpt-ogg.dll'; Digest = 'BBEEA26588404A00AE964F1C4ACEB6D6474E5885F8337491A4C69E29E8516EB6' },
    @{ Name = 'openmpt-vorbis.dll'; Digest = '7393D017E2930D57C12C016DE21C6FA658F862481758BA394823F6AC5E013DF2' },
    @{ Name = 'openmpt-zlib.dll'; Digest = '966D96E9AAF5CBD9E1B600435AF1180A3A643ACE17A0E782A5BECF378B25445B' }
)

function Assert-FileDigest {
    param([string]$Path, [string]$Expected, [string]$Description)
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { throw "Missing $Description`: $Path" }
    $actualDigest = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash
    if ($actualDigest -ne $Expected) { throw "$Description hash mismatch: $Path" }
}

if ($NativePackageDirectory) {
    $packageRoot = (Resolve-Path -LiteralPath $NativePackageDirectory).ProviderPath
    $archiveStatus = 'not-supplied; package files checked individually'
} else {
    New-Item -ItemType Directory -Path $cacheRoot -Force | Out-Null
    if (-not (Test-Path -LiteralPath $archivePath -PathType Leaf)) {
        if ($Offline) { throw "Pinned libopenmpt archive absent in offline mode: $archivePath" }
        $partialArchive = Join-Path $cacheRoot ([guid]::NewGuid().ToString('N') + '.partial.zip')
        Invoke-WebRequest -Uri $downloadUrl -OutFile $partialArchive -ErrorAction Stop
        Assert-FileDigest -Path $partialArchive -Expected $archiveDigest -Description 'downloaded libopenmpt archive'
        Move-Item -LiteralPath $partialArchive -Destination $archivePath
    }
    Assert-FileDigest -Path $archivePath -Expected $archiveDigest -Description 'libopenmpt archive'
    $archiveStatus = $archiveDigest
    if (-not (Test-Path -LiteralPath $packageRoot -PathType Container)) {
        $stagingDirectory = Join-Path $cacheRoot ([guid]::NewGuid().ToString('N') + '.extracting')
        Expand-Archive -LiteralPath $archivePath -DestinationPath $stagingDirectory
        $packageRoot = $stagingDirectory
    }
}

$importLibrary = Join-Path $packageRoot 'lib/amd64/libopenmpt.lib'
$nativeDirectory = Join-Path $packageRoot 'bin/amd64'
Assert-FileDigest -Path $importLibrary -Expected $importDigest -Description 'libopenmpt import library'
foreach ($nativeFile in $nativeFiles) {
    Assert-FileDigest -Path (Join-Path $nativeDirectory $nativeFile.Name) -Expected $nativeFile.Digest -Description $nativeFile.Name
}
$nativeLicensePath = Join-Path $packageRoot 'LICENSE.txt'
$componentLicensesPath = Join-Path $packageRoot 'Licenses'
if (-not (Test-Path -LiteralPath $nativeLicensePath -PathType Leaf) -or
    -not (Test-Path -LiteralPath $componentLicensesPath -PathType Container)) {
    throw 'Official libopenmpt package notices are missing'
}
if (-not $NativePackageDirectory -and $packageRoot -ne $cachedPackageRoot) {
    $cachePrefix = [System.IO.Path]::GetFullPath($cacheRoot).TrimEnd([System.IO.Path]::DirectorySeparatorChar) + [System.IO.Path]::DirectorySeparatorChar
    if (-not ([System.IO.Path]::GetFullPath($packageRoot)).StartsWith($cachePrefix, [System.StringComparison]::OrdinalIgnoreCase) -or
        -not ([System.IO.Path]::GetFullPath($cachedPackageRoot)).StartsWith($cachePrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw 'Native cache move escaped the project scratch directory'
    }
    Move-Item -LiteralPath $packageRoot -Destination $cachedPackageRoot
    $packageRoot = $cachedPackageRoot
    $nativeDirectory = Join-Path $packageRoot 'bin/amd64'
    $nativeLicensePath = Join-Path $packageRoot 'LICENSE.txt'
    $componentLicensesPath = Join-Path $packageRoot 'Licenses'
}
Write-Output "NATIVE_PREFLIGHT_OK archive=$archiveStatus package=$packageRoot"
if ($PreflightOnly) { return }

$cargoExecutable = (Get-Command cargo -ErrorAction Stop).Source
$env:OPENMPT_LIB_DIR = Join-Path $packageRoot 'lib/amd64'
$env:CARGO_TARGET_DIR = Join-Path $repositoryRoot 'target'
$buildArguments = @('build', '--locked', '--manifest-path', (Join-Path $repositoryRoot 'Cargo.toml'))
if ($Release) { $buildArguments += '--release' }
if ($Offline) { $buildArguments += '--offline' }
$sourcesBeforeBuild = Get-EvidenceSources
& $cargoExecutable @buildArguments
if ($LASTEXITCODE -ne 0) { throw "Turbofish build failed: $LASTEXITCODE" }

$profileDirectory = if ($Release) { 'release' } else { 'debug' }
$outputDirectory = (Resolve-Path -LiteralPath (Join-Path $env:CARGO_TARGET_DIR $profileDirectory)).ProviderPath
$targetPrefix = [System.IO.Path]::GetFullPath($env:CARGO_TARGET_DIR).TrimEnd([System.IO.Path]::DirectorySeparatorChar) + [System.IO.Path]::DirectorySeparatorChar
if (-not $outputDirectory.StartsWith($targetPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw 'Executable directory escaped the project target directory'
}
$executable = Join-Path $outputDirectory 'turbofish-deluxe.exe'
if (-not (Test-Path -LiteralPath $executable -PathType Leaf)) { throw "Built executable missing: $executable" }
foreach ($nativeFile in $nativeFiles) {
    $destination = Join-Path $outputDirectory $nativeFile.Name
    Copy-Item -LiteralPath (Join-Path $nativeDirectory $nativeFile.Name) -Destination $destination -Force
    Assert-FileDigest -Path $destination -Expected $nativeFile.Digest -Description "copied $($nativeFile.Name)"
}
$noticeDirectory = Join-Path $outputDirectory 'libopenmpt-notices'
# Copying a directory into an existing destination nests another Licenses
# directory. Recreate only this bounded build output before staging notices.
$resolvedNoticeDirectory = [System.IO.Path]::GetFullPath($noticeDirectory)
$outputPrefix = $outputDirectory.TrimEnd([System.IO.Path]::DirectorySeparatorChar) + [System.IO.Path]::DirectorySeparatorChar
if (-not $resolvedNoticeDirectory.StartsWith($outputPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw 'Native notice directory escaped the build output directory'
}
if (Test-Path -LiteralPath $noticeDirectory) {
    $existingNoticeDirectory = Get-Item -LiteralPath $noticeDirectory -Force
    if (($existingNoticeDirectory.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'Native notice destination is a reparse point' }
    Remove-Item -LiteralPath $resolvedNoticeDirectory -Recurse -Force
}
New-Item -ItemType Directory -Path $noticeDirectory -Force | Out-Null
Copy-Item -LiteralPath $nativeLicensePath -Destination (Join-Path $noticeDirectory 'LICENSE.txt') -Force
Copy-Item -LiteralPath $componentLicensesPath -Destination (Join-Path $noticeDirectory 'Licenses') -Recurse -Force
Write-Output "NATIVE_RUNTIME_READY executable=$executable notices=$noticeDirectory"
$identityParameters = @{
    ExecutablePath = $executable
    BuildProfile = $profileDirectory
    BuildArguments = $buildArguments
    NativePackageDirectory = $packageRoot
    AdditionalArtifacts = $BuildEvidencePaths
    SourcesBeforeBuild = $sourcesBeforeBuild
}
if ($BuildIdentityPath) { $identityParameters.OutputPath = $BuildIdentityPath }
& (Join-Path $PSScriptRoot 'New-BuildIdentity.ps1') @identityParameters
if ($PrepareOnly) { return }

& $executable @GameArguments
if ($LASTEXITCODE -ne 0) { throw "Turbofish runtime exited with code $LASTEXITCODE" }
