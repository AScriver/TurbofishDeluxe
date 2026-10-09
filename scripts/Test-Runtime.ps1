[CmdletBinding()]
param([string]$GameDirectory)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
Set-Location -LiteralPath $repositoryRoot
$launcher = Join-Path $PSScriptRoot 'Start-TurbofishDeluxe.ps1'
& $launcher -PreflightOnly
$nativePackage = Join-Path $repositoryRoot '.scratch/native/libopenmpt-0.8.9/package'
$env:OPENMPT_LIB_DIR = Join-Path $nativePackage 'lib/amd64'
$nativeRuntimeDirectory = Join-Path $nativePackage 'bin/amd64'
$env:PATH = $nativeRuntimeDirectory + [System.IO.Path]::PathSeparator + $env:PATH
$cargoExecutable = (Get-Command cargo -ErrorAction Stop).Source
$validationCommands = @(
    @('fmt', '--all', '--', '--check'),
    @('clippy', '--locked', '--workspace', '--all-targets', '--', '-D', 'warnings'),
    @('test', '--locked', '--workspace', '--all-targets')
)
foreach ($validationArguments in $validationCommands) {
    & $cargoExecutable @validationArguments
    if ($LASTEXITCODE -ne 0) { throw "Runtime validation failed: $($validationArguments -join ' ')" }
}
if ($GameDirectory) {
    $ownedGameDirectory = (Resolve-Path -LiteralPath $GameDirectory).ProviderPath
    foreach ($track in @('Insaniq2.mo3', 'Alien.mo3', 'Lullaby.mo3')) {
        if (-not (Test-Path -LiteralPath (Join-Path $ownedGameDirectory "music/$track") -PathType Leaf)) {
            throw "Owned music file missing: $track"
        }
    }
    $env:OWNED_GAME_DIR = $ownedGameDirectory
    $ownedArguments = @('test', '--locked', '-p', 'turbofish-openmpt', '--test', 'binding', '--', '--ignored')
    & $cargoExecutable @ownedArguments
    if ($LASTEXITCODE -ne 0) { throw 'Owned music decoder regression failed' }
} else {
    Write-Output 'OWNED_MUSIC_TESTS_SKIPPED: pass -GameDirectory to run installed MO3 decoder checks'
}
& $launcher -Offline -PrepareOnly
