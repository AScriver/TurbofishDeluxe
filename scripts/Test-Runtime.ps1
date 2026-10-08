[CmdletBinding()]
param()
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
Set-Location -LiteralPath $repositoryRoot
$cargoExecutable = (Get-Command cargo -ErrorAction Stop).Source
$validationCommands = @(
    @('fmt', '--all', '--', '--check'),
    @('clippy', '--locked', '--all-targets', '--', '-D', 'warnings'),
    @('test', '--locked', '--all-targets'),
    @('build', '--locked')
)
foreach ($validationArguments in $validationCommands) {
    & $cargoExecutable @validationArguments
    if ($LASTEXITCODE -ne 0) { throw "Runtime validation failed: $($validationArguments -join ' ')" }
}
