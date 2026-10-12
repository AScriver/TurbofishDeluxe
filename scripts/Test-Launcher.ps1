[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$NativePackageDirectory
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$runRoot = Join-Path $repositoryRoot ('.scratch/performance/295/launcher-' + [guid]::NewGuid().ToString('N'))
$fixtureRoot = Join-Path $runRoot 'repository'
$fixtureScripts = Join-Path $fixtureRoot 'scripts'
$shimDirectory = Join-Path $runRoot 'shim'
$gitExecutable = (Get-Command git -ErrorAction Stop).Source
$rustExecutable = (Get-Command rustc -ErrorAction Stop).Source
$savedEnvironment = @{}
foreach ($environmentName in @('PATH', 'OPENMPT_LIB_DIR', 'CARGO_TARGET_DIR', 'TURBOFISH_LAUNCHER_RECORD', 'TURBOFISH_LAUNCHER_FAIL_BUILD')) {
    $savedEnvironment[$environmentName] = [Environment]::GetEnvironmentVariable($environmentName)
}
$passedCases = [System.Collections.Generic.List[string]]::new()

function Invoke-TestNative {
    param([string]$Executable, [string[]]$ArgumentList)
    & $Executable @ArgumentList
    if ($LASTEXITCODE -ne 0) { throw "Launcher test command failed: $Executable (exit $LASTEXITCODE)" }
}

function Assert-LauncherCheck {
    param([bool]$Condition, [string]$Message)
    if (-not $Condition) { throw $Message }
}

function Read-RecordedArgumentList {
    param([string]$Path)
    $argumentText = [System.IO.File]::ReadAllText($Path)
    return ,($argumentText.Split([char]0))
}

try {
    New-Item -ItemType Directory -Path $fixtureScripts, $shimDirectory -Force | Out-Null
    foreach ($sourceName in @('Start-TurbofishDeluxe.ps1', 'Evidence.Common.ps1', 'New-BuildIdentity.ps1', 'Test-Evidence.ps1')) {
        Copy-Item -LiteralPath (Join-Path $PSScriptRoot $sourceName) -Destination (Join-Path $fixtureScripts $sourceName)
    }
    foreach ($sourceName in @('Cargo.toml', 'Cargo.lock', '.gitignore')) {
        Copy-Item -LiteralPath (Join-Path $repositoryRoot $sourceName) -Destination (Join-Path $fixtureRoot $sourceName)
    }
    $toolchainFile = Join-Path $repositoryRoot 'rust-toolchain.toml'
    if (Test-Path -LiteralPath $toolchainFile -PathType Leaf) { Copy-Item -LiteralPath $toolchainFile -Destination (Join-Path $fixtureRoot 'rust-toolchain.toml') }
    $fixtureLauncher = Join-Path $fixtureScripts 'Start-TurbofishDeluxe.ps1'
    Assert-LauncherCheck ((Get-FileHash -LiteralPath $fixtureLauncher).Hash -eq (Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'Start-TurbofishDeluxe.ps1')).Hash) 'Launcher copy differs from maintained source.'
    Invoke-TestNative -Executable $gitExecutable -ArgumentList @('-C', $fixtureRoot, 'init', '--quiet')
    Invoke-TestNative -Executable $gitExecutable -ArgumentList @('-C', $fixtureRoot, '-c', 'user.name=Launcher test', '-c', 'user.email=launcher-test@example.invalid', 'commit', '--quiet', '--allow-empty', '-m', 'test: initialize disposable launcher fixture')

    # This executable only records arguments and copies itself. It never builds
    # or launches the game. The real launcher and evidence helpers are unchanged.
    $shimSource = @'
use std::{env, fs, path::PathBuf, process};
fn main() {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let executable = env::current_exe().unwrap();
    let is_cargo = executable.file_stem().unwrap() == "cargo";
    if is_cargo && arguments == ["-V"] {
        println!("cargo (launcher recording shim; no compilation)");
        return;
    }
    let record = PathBuf::from(env::var("TURBOFISH_LAUNCHER_RECORD").unwrap());
    if !is_cargo {
        fs::write(record.with_extension("runtime-arguments"), arguments.join("\0")).unwrap();
        return;
    }
    assert_eq!(arguments.first().map(String::as_str), Some("build"));
    fs::write(record.with_extension("cargo-arguments"), arguments.join("\0")).unwrap();
    if env::var("TURBOFISH_LAUNCHER_FAIL_BUILD").as_deref() == Ok("1") {
        eprintln!("intentional launcher test build failure");
        process::exit(73);
    }
    let profile = if arguments.iter().any(|value| value == "--release") { "release" } else { "debug" };
    let output = PathBuf::from(env::var("CARGO_TARGET_DIR").unwrap()).join(profile);
    fs::create_dir_all(&output).unwrap();
    fs::copy(executable, output.join("turbofish-deluxe.exe")).unwrap();
}
'@
    $shimSourcePath = Join-Path $shimDirectory 'cargo.rs'
    [System.IO.File]::WriteAllText($shimSourcePath, $shimSource)
    Invoke-TestNative -Executable $rustExecutable -ArgumentList @('--edition', '2024', '--crate-name', 'launcher_recording_shim', $shimSourcePath, '-o', (Join-Path $shimDirectory 'cargo.exe'))
    $env:PATH = $shimDirectory + [System.IO.Path]::PathSeparator + $savedEnvironment['PATH']
    $env:TURBOFISH_LAUNCHER_FAIL_BUILD = '0'
    $selectionCases = @(
        @{ Name = 'default'; Switches = @{}; Profile = 'release'; Launch = $false },
        @{ Name = 'release'; Switches = @{ Release = $true }; Profile = 'release'; Launch = $false },
        @{ Name = 'development'; Switches = @{ Development = $true }; Profile = 'debug'; Launch = $false },
        @{ Name = 'release-false'; Switches = @{ Release = $false }; Profile = 'release'; Launch = $false },
        @{ Name = 'development-false'; Switches = @{ Development = $false }; Profile = 'release'; Launch = $false },
        @{ Name = 'development-release-false'; Switches = @{ Development = $true; Release = $false }; Profile = 'debug'; Launch = $false },
        @{ Name = 'default-forwarding'; Switches = @{}; Profile = 'release'; Launch = $true },
        @{ Name = 'development-forwarding'; Switches = @{ Development = $true }; Profile = 'debug'; Launch = $true }
    )
    $gameArguments = @('--save-dir', 'disposable path with spaces', '--seed', '42', 'quoted "value"', '&;$%literal', ('Unicode fish ' + [char]0x9b5a), '', "line`nbreak")
    foreach ($selectionCase in $selectionCases) {
        $caseName = $selectionCase.Name
        $recordBase = Join-Path $runRoot $caseName
        $env:TURBOFISH_LAUNCHER_RECORD = $recordBase
        $receiptPath = Join-Path $fixtureRoot ('.scratch/' + $caseName + '.local.json')
        $launcherParameters = @{
            NativePackageDirectory = $NativePackageDirectory
            Offline = $true
            PrepareOnly = -not $selectionCase.Launch
            BuildIdentityPath = $receiptPath
            GameArguments = $gameArguments
        }
        foreach ($switchName in $selectionCase.Switches.Keys) { $launcherParameters[$switchName] = $selectionCase.Switches[$switchName] }
        & $fixtureLauncher @launcherParameters | Out-Null
        $actualArguments = Read-RecordedArgumentList -Path ($recordBase + '.cargo-arguments')
        $expectedArguments = @('build', '--locked', '--manifest-path', (Join-Path $fixtureRoot 'Cargo.toml'))
        if ($selectionCase.Profile -eq 'release') { $expectedArguments += '--release' }
        $expectedArguments += '--offline'
        Assert-LauncherCheck (($actualArguments | ConvertTo-Json -Compress) -ceq ($expectedArguments | ConvertTo-Json -Compress)) "Cargo arguments differ for $caseName."
        $receipt = Get-Content -LiteralPath $receiptPath -Raw | ConvertFrom-Json
        Assert-LauncherCheck ($receipt.configuration.profile -ceq $selectionCase.Profile) "Receipt profile differs for $caseName."
        Assert-LauncherCheck (($receipt.configuration.argument_list | ConvertTo-Json -Compress) -ceq ($actualArguments | ConvertTo-Json -Compress)) "Receipt arguments differ for $caseName."
        $executableArtifact = @($receipt.artifacts | Where-Object role -eq 'executable')[0]
        Assert-LauncherCheck ($executableArtifact.path -ceq ('target/' + $selectionCase.Profile + '/turbofish-deluxe.exe')) "Receipt executable path differs for $caseName."
        Assert-LauncherCheck ($receipt.association -ceq 'launcher-success-with-unchanged-inputs') "Source association differs for $caseName."
        & (Join-Path $fixtureScripts 'Test-Evidence.ps1') -RecordPath $receiptPath -OutputPath (Join-Path $fixtureRoot ('.scratch/' + $caseName + '-readback.local.json')) -CheckSources | Out-Null
        if ($selectionCase.Launch) {
            $actualGameArguments = Read-RecordedArgumentList -Path ($recordBase + '.runtime-arguments')
            Assert-LauncherCheck (($actualGameArguments | ConvertTo-Json -Compress) -ceq ($gameArguments | ConvertTo-Json -Compress)) "Game arguments differ for $caseName."
        } else {
            Assert-LauncherCheck (-not (Test-Path -LiteralPath ($recordBase + '.runtime-arguments'))) "PrepareOnly ran an executable for $caseName."
        }
        $passedCases.Add($caseName)
    }

    # Repeat builds above also check bounded notice replacement, avoiding nested
    # Licenses directories; complete readback checks every staged native file.
    Assert-LauncherCheck (-not (Test-Path -LiteralPath (Join-Path $fixtureRoot 'target/release/libopenmpt-notices/Licenses/Licenses'))) 'Repeated staging nested the notices.'
    $env:TURBOFISH_LAUNCHER_RECORD = Join-Path $runRoot 'preflight'
    & $fixtureLauncher -PreflightOnly -Offline -NativePackageDirectory $NativePackageDirectory | Out-Null
    Assert-LauncherCheck (-not (Test-Path -LiteralPath ($env:TURBOFISH_LAUNCHER_RECORD + '.cargo-arguments'))) 'PreflightOnly invoked Cargo.'
    $passedCases.Add('preflight-only')

    $env:TURBOFISH_LAUNCHER_RECORD = Join-Path $runRoot 'conflict'
    $conflictRejected = $false
    try { & $fixtureLauncher -Release -Development -PreflightOnly -Offline -NativePackageDirectory (Join-Path $runRoot 'absent-package') | Out-Null }
    catch { $conflictRejected = $_.Exception.Message -ceq '-Release and -Development cannot be used together.' }
    Assert-LauncherCheck $conflictRejected 'Conflicting switches did not fail before native preflight.'
    Assert-LauncherCheck (-not (Test-Path -LiteralPath ($env:TURBOFISH_LAUNCHER_RECORD + '.cargo-arguments'))) 'Conflict invoked Cargo.'
    $passedCases.Add('reject-conflict-before-preflight')

    $env:TURBOFISH_LAUNCHER_RECORD = Join-Path $runRoot 'build-failure'
    $env:TURBOFISH_LAUNCHER_FAIL_BUILD = '1'
    $failedReceiptPath = Join-Path $fixtureRoot '.scratch/build-failure.local.json'
    $buildFailureRejected = $false
    try { & $fixtureLauncher -Offline -NativePackageDirectory $NativePackageDirectory -BuildIdentityPath $failedReceiptPath -GameArguments $gameArguments | Out-Null }
    catch { $buildFailureRejected = $_.Exception.Message -ceq 'Turbofish build failed: 73' }
    Assert-LauncherCheck $buildFailureRejected 'Cargo failure did not stop launch.'
    Assert-LauncherCheck (-not (Test-Path -LiteralPath $failedReceiptPath)) 'Failed build wrote a receipt.'
    Assert-LauncherCheck (-not (Test-Path -LiteralPath ($env:TURBOFISH_LAUNCHER_RECORD + '.runtime-arguments'))) 'Failed build ran an executable.'
    $passedCases.Add('reject-build-failure')
    [pscustomobject]@{ Result = 'Passed'; Cases = $passedCases.Count; Names = @($passedCases); EvidenceDirectory = $runRoot; NativeGameLaunched = $false }
} finally {
    foreach ($environmentName in $savedEnvironment.Keys) { [Environment]::SetEnvironmentVariable($environmentName, $savedEnvironment[$environmentName], 'Process') }
}
