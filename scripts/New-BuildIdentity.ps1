[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$ExecutablePath,
    [string]$OutputPath,
    [ValidateSet('debug', 'release')][string]$BuildProfile = 'debug',
    [string[]]$Features = @(),
    [string[]]$BuildArguments = @(),
    [string]$NativePackageDirectory,
    [string]$OriginalExecutablePath,
    [string[]]$AdditionalArtifacts = @(),
    [System.Collections.IDictionary]$SourcesBeforeBuild
)
. (Join-Path $PSScriptRoot 'Evidence.Common.ps1')
if (-not $OutputPath) { $OutputPath = New-EvidenceOutputName -Kind 'build' }
Assert-EvidenceOutputPath -Path $OutputPath | Out-Null
$sourceSnapshot = Get-EvidenceSources
if (@($sourceSnapshot.files | Where-Object { -not $_.exists }).Count -gt 0) { throw 'A build input is missing; cannot seal the source identity.' }
if ($SourcesBeforeBuild -and $SourcesBeforeBuild['sha256'] -ne $sourceSnapshot.sha256) { throw 'Build inputs changed during preparation; rebuild before capturing an associated identity.' }
$executableRecord = Get-EvidenceFile -Path $ExecutablePath -Role 'executable'
if (-not $executableRecord.exists) { throw 'Built executable is missing.' }
$executableDirectory = [System.IO.Path]::GetDirectoryName((Get-EvidenceFullPath -Path $ExecutablePath))
$artifacts = @($executableRecord)
foreach ($nativeName in @('libopenmpt.dll', 'openmpt-mpg123.dll', 'openmpt-ogg.dll', 'openmpt-vorbis.dll', 'openmpt-zlib.dll')) {
    $artifacts += Get-EvidenceFile -Path (Join-Path $executableDirectory $nativeName) -Role 'native-runtime'
}
$nativeGroup = Get-EvidenceFileGroup -Directory $executableDirectory -Filter '*.dll' -Role 'native-runtime'
$noticeDirectory = Join-Path $executableDirectory 'libopenmpt-notices'
$noticeGroup = Get-EvidenceFileGroup -Directory $noticeDirectory -Recurse $true -Role 'native-notice'
$artifacts += Get-EvidenceFile -Path (Join-Path $noticeDirectory 'LICENSE.txt') -Role 'native-notice'
foreach ($groupFile in @($nativeGroup.files) + @($noticeGroup.files)) {
    if ($groupFile.path -cnotin @($artifacts | ForEach-Object { $_.path })) { $artifacts += $groupFile }
}
if (-not $NativePackageDirectory) { $NativePackageDirectory = '.scratch/native/libopenmpt-0.8.9/package' }
foreach ($packageFile in @(
    (Join-Path $NativePackageDirectory 'lib/amd64/libopenmpt.lib'),
    '.scratch/native/libopenmpt-0.8.9/libopenmpt-0.8.9+release.dev.windows.vs2022.zip'
)) {
    if (Test-Path -LiteralPath (Get-EvidenceFullPath -Path $packageFile) -PathType Leaf) {
        $artifacts += Get-EvidenceFile -Path $packageFile -Role 'native-build-input'
    }
}
foreach ($additionalPath in $AdditionalArtifacts) { $artifacts += Get-EvidenceFile -Path $additionalPath -Role 'additional-evidence' }
$originalGame = [ordered]@{ status = 'unknown'; executable_name = $null; file_version = $null; product_version = $null; sha256 = $null }
if ($OriginalExecutablePath) {
    $originalFile = Get-EvidenceFile -Path $OriginalExecutablePath -Role 'original-game'
    if (-not $originalFile.exists) { throw 'Original executable is missing; no version or digest was captured.' }
    $originalVersion = (Get-Item -LiteralPath (Get-EvidenceFullPath -Path $OriginalExecutablePath)).VersionInfo
    $originalGame = [ordered]@{
        status = 'file-metadata-only'
        executable_name = [System.IO.Path]::GetFileName($OriginalExecutablePath)
        file_version = $originalVersion.FileVersion
        product_version = $originalVersion.ProductVersion
        sha256 = $originalFile.sha256
    }
    $artifacts += $originalFile
}
$rustProgram = (Get-Command rustc -ErrorAction Stop).Source
$cargoProgram = (Get-Command cargo -ErrorAction Stop).Source
$rustupProgram = Get-Command rustup -ErrorAction SilentlyContinue
$toolchainIdentity = [ordered]@{
    rustc = Invoke-EvidenceCommand -Executable $rustProgram -ArgumentList @('-vV')
    cargo = Invoke-EvidenceCommand -Executable $cargoProgram -ArgumentList @('-V')
    active_toolchain = if ($rustupProgram) { Invoke-EvidenceCommand -Executable $rustupProgram.Source -ArgumentList @('show', 'active-toolchain') } else { 'unknown' }
    powershell = $PSVersionTable.PSVersion.ToString()
    os = [System.Runtime.InteropServices.RuntimeInformation]::OSDescription
}
$buildEnvironment = [ordered]@{}
foreach ($environmentName in @('RUSTUP_TOOLCHAIN', 'RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'CARGO_BUILD_TARGET', 'CARGO_TARGET_DIR', 'OPENMPT_LIB_DIR')) {
    $buildEnvironment[$environmentName] = [Environment]::GetEnvironmentVariable($environmentName)
}
$configuration = [ordered]@{ profile = $BuildProfile; features = @($Features); argument_list = @($BuildArguments); environment = $buildEnvironment }
$headCommit = Invoke-EvidenceGit -RepositoryRoot $EvidenceRepositoryRoot -ArgumentList @('rev-parse', 'HEAD')
$repositoryIdentity = [ordered]@{
    commit = $headCommit
    branch = Invoke-EvidenceGit -RepositoryRoot $EvidenceRepositoryRoot -ArgumentList @('branch', '--show-current')
    dirty_status = Invoke-EvidenceGit -RepositoryRoot $EvidenceRepositoryRoot -ArgumentList @('status', '--porcelain=v1', '--untracked-files=all')
    tracked_diff_sha256 = Get-EvidenceDigest -Text (Invoke-EvidenceGit -RepositoryRoot $EvidenceRepositoryRoot -ArgumentList @('diff', '--binary', 'HEAD', '--', 'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'src', 'tests', 'crates', 'scripts', '.cargo'))
}
$buildRecord = [ordered]@{
    schema_version = 1
    kind = 'build'
    id = $null
    captured_utc = [DateTimeOffset]::UtcNow.ToString('o')
    repository = $repositoryIdentity
    sources = $sourceSnapshot
    toolchain = $toolchainIdentity
    configuration = $configuration
    association = if ($SourcesBeforeBuild) { 'launcher-success-with-unchanged-inputs' } else { 'snapshot-only; build/source association unverified' }
    original_game = $originalGame
    artifacts = $artifacts
    artifact_groups = @($nativeGroup, $noticeGroup | ForEach-Object {
        [ordered]@{ directory = $_.directory; filter = $_.filter; recurse = $_.recurse; role = $_.role; paths = $_.paths }
    })
    review = [ordered]@{ behavior = 'not tested'; retail_fidelity = 'not tested'; human = 'not tested' }
}
$buildRecord.id = Get-EvidenceRecordId -Record $buildRecord
# Seal only a consistent snapshot; these checks establish bytes, not package authenticity.
$captureFailures = @(Get-EvidenceChecks -Record $buildRecord -CheckSources | Where-Object { $_.status -notin @('match', 'not-captured') })
if ($captureFailures.Count -gt 0) { throw 'Build evidence is missing or changed during capture; no receipt was written.' }
$writtenPath = Write-EvidenceJson -Path $OutputPath -Record $buildRecord
[pscustomobject]@{ Kind = 'build'; Id = $buildRecord.id; Path = $writtenPath; Sha256 = (Get-FileHash -LiteralPath $writtenPath).Hash.ToLowerInvariant(); Association = $buildRecord.association }
