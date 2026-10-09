# Shared primitives for the maintained evidence commands; dot-source, do not execute.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$EvidenceRepositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))

function Invoke-EvidenceCommand {
    param([string]$Executable, [string[]]$ArgumentList)
    $commandLines = @(& $Executable @ArgumentList 2>&1)
    if ($LASTEXITCODE -ne 0) { throw "Evidence command failed: $Executable $($ArgumentList -join ' ')" }
    return ($commandLines -join "`n").TrimEnd("`r", "`n")
}

function Invoke-EvidenceGit {
    param([string]$RepositoryRoot, [string[]]$ArgumentList)
    $gitProgram = (Get-Command git -ErrorAction Stop).Source
    return Invoke-EvidenceCommand -Executable $gitProgram -ArgumentList (@('-C', $RepositoryRoot) + $ArgumentList)
}

function Get-EvidenceDigest {
    param([string]$Text)
    $textBytes = [System.Text.Encoding]::UTF8.GetBytes($Text)
    return [System.Convert]::ToHexString([System.Security.Cryptography.SHA256]::HashData($textBytes)).ToLowerInvariant()
}

function Get-EvidenceFullPath {
    param([string]$Path, [string]$RepositoryRoot = $EvidenceRepositoryRoot)
    if (-not [System.IO.Path]::IsPathRooted($Path)) { $Path = Join-Path $RepositoryRoot $Path }
    return [System.IO.Path]::GetFullPath($Path)
}

function Get-EvidenceRelativePath {
    param([string]$Path, [string]$RepositoryRoot = $EvidenceRepositoryRoot)
    $fullPath = Get-EvidenceFullPath -Path $Path -RepositoryRoot $RepositoryRoot
    $rootPrefix = $RepositoryRoot.TrimEnd('\', '/') + [System.IO.Path]::DirectorySeparatorChar
    if ($fullPath.StartsWith($rootPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        return [System.IO.Path]::GetRelativePath($RepositoryRoot, $fullPath).Replace('\', '/')
    }
    return $fullPath
}

function Assert-EvidenceOutputPath {
    param([string]$Path, [string]$RepositoryRoot = $EvidenceRepositoryRoot, [switch]$Report)
    $fullPath = Get-EvidenceFullPath -Path $Path -RepositoryRoot $RepositoryRoot
    $relativePath = Get-EvidenceRelativePath -Path $fullPath -RepositoryRoot $RepositoryRoot
    $privatePath = $relativePath -match '^(\.scratch|private|local)/.+'
    $reportPath = $Report -and $relativePath -match '^docs/playtests/[^/]+\.md$' -and
        [System.IO.Path]::GetFileName($relativePath) -ne 'TEMPLATE.md'
    if (-not $privatePath -and -not $reportPath) { throw 'Evidence outputs must be private project files (or a new docs/playtests Markdown report).' }
    # Lexical containment alone is insufficient when a private directory is a junction.
    $walkPath = $fullPath
    while ($walkPath -and $walkPath -ne $RepositoryRoot) {
        if (Test-Path -LiteralPath $walkPath) {
            $walkItem = Get-Item -LiteralPath $walkPath -Force
            if ($walkItem.Attributes -band [System.IO.FileAttributes]::ReparsePoint) { throw "Evidence output crosses a reparse point: $walkPath" }
        }
        $walkPath = [System.IO.Path]::GetDirectoryName($walkPath)
    }
    if (Test-Path -LiteralPath $fullPath) { throw "Evidence records are immutable; choose a new output path: $relativePath" }
    return $fullPath
}

function Write-EvidenceText {
    param([string]$Path, [string]$Text, [string]$RepositoryRoot = $EvidenceRepositoryRoot, [switch]$Report)
    $fullPath = Assert-EvidenceOutputPath -Path $Path -RepositoryRoot $RepositoryRoot -Report:$Report
    $parentPath = [System.IO.Path]::GetDirectoryName($fullPath)
    New-Item -ItemType Directory -Path $parentPath -Force | Out-Null
    $temporaryPath = Join-Path $parentPath ([guid]::NewGuid().ToString('N') + '.partial')
    try {
        [System.IO.File]::WriteAllText($temporaryPath, $Text.TrimEnd("`r", "`n") + "`n", [System.Text.UTF8Encoding]::new($false))
        [System.IO.File]::Move($temporaryPath, $fullPath, $false)
    } finally {
        if (Test-Path -LiteralPath $temporaryPath -PathType Leaf) { Remove-Item -LiteralPath $temporaryPath }
    }
    return $fullPath
}

function Write-EvidenceJson {
    param([string]$Path, $Record, [string]$RepositoryRoot = $EvidenceRepositoryRoot)
    return Write-EvidenceText -Path $Path -Text ($Record | ConvertTo-Json -Depth 100) -RepositoryRoot $RepositoryRoot
}

function Read-EvidenceJson {
    param([string]$Path, [string]$RepositoryRoot = $EvidenceRepositoryRoot)
    $fullPath = Get-EvidenceFullPath -Path $Path -RepositoryRoot $RepositoryRoot
    return (Get-Content -LiteralPath $fullPath -Raw | ConvertFrom-Json -AsHashtable -Depth 100)
}

function Get-EvidenceField {
    param($Record, [string[]]$Keys)
    $fieldValue = $Record
    foreach ($fieldKey in $Keys) {
        if ($fieldValue -isnot [System.Collections.IDictionary] -or -not $fieldValue.Contains($fieldKey)) { return $null }
        $fieldValue = $fieldValue[$fieldKey]
    }
    return $fieldValue
}

function Test-EvidenceNumber {
    param($Value, [double]$Minimum = 0, [switch]$Integer)
    if ($null -eq $Value -or $Value.GetType().Name -notin @('Byte', 'SByte', 'Int16', 'UInt16', 'Int32', 'UInt32', 'Int64', 'UInt64', 'Single', 'Double', 'Decimal', 'BigInteger')) { return $false }
    $numericValue = [double]$Value
    return [double]::IsFinite($numericValue) -and $numericValue -ge $Minimum -and
        (-not $Integer -or [Math]::Truncate($numericValue) -eq $numericValue)
}

function Get-EvidenceRecordId {
    param([System.Collections.IDictionary]$Record)
    $identityFields = [ordered]@{}
    foreach ($fieldName in ($Record.Keys | Sort-Object)) {
        if ($fieldName -notin @('id', 'captured_utc')) { $identityFields[$fieldName] = $Record[$fieldName] }
    }
    return Get-EvidenceDigest -Text ($identityFields | ConvertTo-Json -Depth 100 -Compress)
}

function Get-EvidenceFile {
    param([string]$Path, [string]$Role, [bool]$Required = $true, [string]$RepositoryRoot = $EvidenceRepositoryRoot)
    $fullPath = Get-EvidenceFullPath -Path $Path -RepositoryRoot $RepositoryRoot
    $fileExists = Test-Path -LiteralPath $fullPath -PathType Leaf
    return [ordered]@{
        path = Get-EvidenceRelativePath -Path $fullPath -RepositoryRoot $RepositoryRoot
        role = $Role
        required = $Required
        exists = [bool]$fileExists
        bytes = if ($fileExists) { (Get-Item -LiteralPath $fullPath).Length } else { $null }
        sha256 = if ($fileExists) { (Get-FileHash -LiteralPath $fullPath -Algorithm SHA256).Hash.ToLowerInvariant() } else { $null }
    }
}

function Get-EvidenceSources {
    param([string]$RepositoryRoot = $EvidenceRepositoryRoot)
    $actualRoot = Invoke-EvidenceGit -RepositoryRoot $RepositoryRoot -ArgumentList @('rev-parse', '--show-toplevel')
    if ((Get-EvidenceFullPath -Path $actualRoot) -ne $RepositoryRoot) { throw 'Evidence source identity requires the repository root.' }
    $listedPaths = Invoke-EvidenceGit -RepositoryRoot $RepositoryRoot -ArgumentList @('ls-files', '-z', '--cached', '--others', '--exclude-standard')
    $buildPaths = @($listedPaths.Split([char]0) | Where-Object {
        $_ -match '^(Cargo\.(toml|lock)|rust-toolchain(\.toml)?|build\.rs|\.cargo/.*|(?:src|tests)/.*\.rs|crates/.*(?:Cargo\.toml|build\.rs|\.rs)|scripts/[^/]+\.ps1)$'
    } | Sort-Object -Unique)
    $sourceFiles = @($buildPaths | ForEach-Object { Get-EvidenceFile -Path $_ -Role 'build-input' -RepositoryRoot $RepositoryRoot })
    $sourceFingerprint = ($sourceFiles | ForEach-Object { "$($_.path)`t$($_.exists)`t$($_.bytes)`t$($_.sha256)" }) -join "`n"
    return [ordered]@{ sha256 = Get-EvidenceDigest -Text $sourceFingerprint; files = $sourceFiles }
}

function Get-EvidenceFileGroup {
    param([string]$Directory, [string]$Filter = '*', [bool]$Recurse = $false, [string]$Role, [string]$RepositoryRoot = $EvidenceRepositoryRoot)
    $fullDirectory = Get-EvidenceFullPath -Path $Directory -RepositoryRoot $RepositoryRoot
    $groupFiles = @()
    if (Test-Path -LiteralPath $fullDirectory -PathType Container) {
        $groupFiles = @(Get-ChildItem -LiteralPath $fullDirectory -Filter $Filter -File -Recurse:$Recurse |
            Sort-Object FullName | ForEach-Object { Get-EvidenceFile -Path $_.FullName -Role $Role -RepositoryRoot $RepositoryRoot })
    }
    return [ordered]@{
        directory = Get-EvidenceRelativePath -Path $fullDirectory -RepositoryRoot $RepositoryRoot
        filter = $Filter
        recurse = $Recurse
        role = $Role
        paths = @($groupFiles | ForEach-Object { $_.path })
        files = $groupFiles
    }
}

function Get-EvidenceFileChecks {
    param([object[]]$Files, [string]$RepositoryRoot = $EvidenceRepositoryRoot)
    foreach ($fileRecord in $Files) {
        if (-not $fileRecord['path'] -or $fileRecord['required'] -isnot [bool] -or $fileRecord['exists'] -isnot [bool]) {
            throw 'Invalid evidence file record.'
        }
        if ($fileRecord['exists'] -and ($fileRecord['sha256'] -notmatch '^[a-f0-9]{64}$' -or $null -eq $fileRecord['bytes'])) {
            throw 'Invalid captured file digest/size.'
        }
        $currentFile = Get-EvidenceFile -Path $fileRecord['path'] -Role $fileRecord['role'] -RepositoryRoot $RepositoryRoot
        $fileStatus = if (-not $fileRecord['exists']) {
            if ($fileRecord['required']) { 'missing-at-capture' } elseif ($currentFile.exists) { 'added' } else { 'not-captured' }
        } elseif (-not $currentFile.exists) { 'missing' }
        elseif ($currentFile.sha256 -ne $fileRecord['sha256'] -or $currentFile.bytes -ne $fileRecord['bytes']) { 'changed' }
        else { 'match' }
        [ordered]@{ path = $fileRecord['path']; role = $fileRecord['role']; status = $fileStatus }
    }
}

function Get-EvidenceChecks {
    param($Record, [switch]$CheckSources, [string]$RepositoryRoot = $EvidenceRepositoryRoot)
    if ($Record['schema_version'] -ne 1 -or $Record['kind'] -notin @('build', 'run') -or
        $Record['id'] -notmatch '^[a-f0-9]{64}$' -or -not $Record.Contains('artifacts') -or @($Record['artifacts']).Count -eq 0) {
        throw 'Unsupported or malformed evidence receipt.'
    }
    if (-not $Record.Contains('artifact_groups') -or ($Record['kind'] -eq 'run' -and
        (-not $Record['bookkeeping'] -or -not $Record['build_identity']))) { throw 'Incomplete evidence receipt schema.' }
    if ((Get-EvidenceRecordId -Record $Record) -ne $Record['id']) { throw 'Evidence receipt ID does not match its contents.' }
    $checkRows = @(Get-EvidenceFileChecks -Files $Record['artifacts'] -RepositoryRoot $RepositoryRoot)
    $buildRecord = $Record
    if ($Record['kind'] -eq 'run') {
        $buildReference = $Record['build_identity']
        $buildRecord = Read-EvidenceJson -Path $buildReference['path'] -RepositoryRoot $RepositoryRoot
        if ($buildRecord['kind'] -ne 'build') { throw 'Run receipt does not reference a build identity.' }
        $checkRows += @(Get-EvidenceChecks -Record $buildRecord -RepositoryRoot $RepositoryRoot)
        if ($buildReference['id'] -ne $buildRecord['id']) {
            $checkRows += [ordered]@{ path = $buildReference['path']; role = 'build-identity'; status = 'identity-mismatch' }
        }
        $referenceArtifact = @($Record['artifacts'] | Where-Object { $_['role'] -eq 'build-identity' })
        if ($referenceArtifact.Count -ne 1 -or $referenceArtifact[0]['path'] -cne $buildReference['path'] -or
            $referenceArtifact[0]['sha256'] -ne $buildReference['sha256']) { throw 'Run build reference and artifact identity disagree.' }
    }
    foreach ($fileGroup in $Record['artifact_groups']) {
        $currentGroup = Get-EvidenceFileGroup -Directory $fileGroup['directory'] -Filter $fileGroup['filter'] -Recurse $fileGroup['recurse'] -Role $fileGroup['role'] -RepositoryRoot $RepositoryRoot
        $expectedPaths = @($fileGroup['paths'])
        foreach ($newPath in $currentGroup.paths) {
            if ($newPath -cnotin $expectedPaths) { $checkRows += [ordered]@{ path = $newPath; role = $fileGroup['role']; status = 'added' } }
        }
    }
    if ($CheckSources) {
        if ($buildRecord['kind'] -ne 'build' -or $buildRecord['schema_version'] -ne 1 -or -not $buildRecord['sources']['files']) { throw 'Build source identity is unavailable or malformed.' }
        $checkRows += @(Get-EvidenceFileChecks -Files $buildRecord['sources']['files'] -RepositoryRoot $RepositoryRoot)
        $currentSources = Get-EvidenceSources -RepositoryRoot $RepositoryRoot
        $expectedSourcePaths = @($buildRecord['sources']['files'] | ForEach-Object { $_['path'] })
        foreach ($newSource in $currentSources.files) {
            if ($newSource.path -cnotin $expectedSourcePaths) { $checkRows += [ordered]@{ path = $newSource.path; role = 'build-input'; status = 'added' } }
        }
    }
    return $checkRows
}

function New-EvidenceOutputName {
    param([string]$Kind)
    return '.scratch/evidence/' + [DateTimeOffset]::UtcNow.ToString('yyyyMMddTHHmmssfffZ') + '-' + [guid]::NewGuid().ToString('N').Substring(0, 8) + '-' + $Kind + '.local.json'
}
