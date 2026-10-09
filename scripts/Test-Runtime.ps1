<#
.SYNOPSIS
Runs the complete runtime gate or one focused development check.
.PARAMETER Check
All (the default) runs formatting, lint, workspace tests, and build/staging.
Other values run only the named check; only Build stages the runtime.
.PARAMETER TestFilter
Runs tests whose names contain this substring in the selected test suite.
Allowed only with Unit, Assets, Fonts, Persistence, Binding, or OwnedMusic.
.PARAMETER GameDirectory
Owned installation for All's optional music tests or the OwnedMusic check.
.EXAMPLE
.\scripts\Test-Runtime.ps1 -Check Unit -TestFilter 'bilaterus::tests::'
#>
[CmdletBinding()]
param(
    [string]$GameDirectory,
    [ValidateSet('All', 'Format', 'Lint', 'Tests', 'Unit', 'Assets', 'Fonts', 'Persistence', 'Binding', 'OwnedMusic', 'Build')]
    [string]$Check = 'All',
    [ValidateNotNullOrEmpty()]
    [string]$TestFilter
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if ($PSBoundParameters.ContainsKey('TestFilter')) {
    if ($Check -notin @('Unit', 'Assets', 'Fonts', 'Persistence', 'Binding', 'OwnedMusic')) {
        throw '-TestFilter requires a focused test suite; the complete gate cannot be filtered.'
    }
    if ([string]::IsNullOrWhiteSpace($TestFilter) -or $TestFilter.StartsWith('-')) {
        throw '-TestFilter must be a nonblank test-name substring, not an option.'
    }
}
if ($GameDirectory -and $Check -notin @('All', 'OwnedMusic')) {
    throw '-GameDirectory is supported only with All or OwnedMusic.'
}
if ($Check -eq 'OwnedMusic' -and -not $GameDirectory) {
    throw '-Check OwnedMusic requires -GameDirectory pointing to an owned installation.'
}
$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
Set-Location -LiteralPath $repositoryRoot
$launcher = Join-Path $PSScriptRoot 'Start-TurbofishDeluxe.ps1'
if ($GameDirectory) {
    $ownedGameDirectory = (Resolve-Path -LiteralPath $GameDirectory).ProviderPath
    foreach ($track in @('Insaniq2.mo3', 'Alien.mo3', 'Lullaby.mo3')) {
        if (-not (Test-Path -LiteralPath (Join-Path $ownedGameDirectory "music/$track") -PathType Leaf)) {
            throw "Owned music file missing: $track"
        }
    }
    $env:OWNED_GAME_DIR = $ownedGameDirectory
}
# Formatting does not compile/link and must work without the native package.
if ($Check -ne 'Format') {
    & $launcher -PreflightOnly
    $nativePackage = Join-Path $repositoryRoot '.scratch/native/libopenmpt-0.8.9/package'
    $env:OPENMPT_LIB_DIR = Join-Path $nativePackage 'lib/amd64'
    $nativeRuntimeDirectory = Join-Path $nativePackage 'bin/amd64'
    $env:PATH = $nativeRuntimeDirectory + [System.IO.Path]::PathSeparator + $env:PATH
}
$cargoExecutable = (Get-Command cargo -ErrorAction Stop).Source
$validationCommands = @()
switch ($Check) {
    'All' {
        $validationCommands = @(
            @('fmt', '--all', '--', '--check'),
            @('clippy', '--locked', '--workspace', '--all-targets', '--', '-D', 'warnings'),
            @('test', '--locked', '--workspace', '--all-targets')
        )
        if ($GameDirectory) {
            $validationCommands += ,@('test', '--locked', '-p', 'turbofish-openmpt', '--test', 'binding', '--', '--ignored')
        }
    }
    'Format' { $validationCommands = ,@('fmt', '--all', '--', '--check') }
    'Lint' { $validationCommands = ,@('clippy', '--locked', '--workspace', '--all-targets', '--', '-D', 'warnings') }
    'Tests' { $validationCommands = ,@('test', '--locked', '--workspace', '--all-targets') }
    'Build' { }
    default {
        $testPackage = if ($Check -in @('Binding', 'OwnedMusic')) { 'turbofish-openmpt' } else { 'turbofish-deluxe' }
        $targetArguments = switch ($Check) {
            'Unit' { @('--lib') }
            'Assets' { @('--test', 'assets') }
            'Fonts' { @('--test', 'fonts') }
            'Persistence' { @('--test', 'persistence') }
            'Binding' { @('--test', 'binding') }
            'OwnedMusic' { @('--test', 'binding') }
        }
        $testArguments = @('test', '--locked', '-p', $testPackage) + $targetArguments
        if ($TestFilter) { $testArguments += $TestFilter }
        if ($Check -eq 'OwnedMusic') { $testArguments += @('--', '--ignored') }
        $validationCommands = ,$testArguments
    }
}
foreach ($validationArguments in $validationCommands) {
    Write-Output "RUNTIME_VALIDATION check=$Check command=cargo $($validationArguments -join ' ')"
    & $cargoExecutable @validationArguments
    if ($LASTEXITCODE -ne 0) { throw "Runtime validation failed: $($validationArguments -join ' ')" }
}
if ($Check -eq 'All' -and -not $GameDirectory) {
    Write-Output 'OWNED_MUSIC_TESTS_SKIPPED: pass -GameDirectory to run installed MO3 decoder checks'
}
if ($Check -in @('All', 'Build')) { & $launcher -Offline -PrepareOnly }
Write-Output "RUNTIME_VALIDATION_PASSED check=$Check"
