[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$gitExecutable = (Get-Command git -ErrorAction Stop).Source

function Invoke-RepositoryGit {
    param([string[]]$NativeArguments)
    $argumentList = @('-C', $repositoryRoot) + $NativeArguments
    $commandOutput = @(& $gitExecutable @argumentList)
    if ($LASTEXITCODE -ne 0) { throw "Git check failed: $($NativeArguments -join ' ')" }
    return $commandOutput
}

$actualRoot = (Invoke-RepositoryGit -NativeArguments @('rev-parse', '--show-toplevel')) -join ''
if ([System.IO.Path]::GetFullPath($actualRoot) -ne $repositoryRoot) { throw 'Run against this project repository, not an ancestor repository.' }
$requiredPaths = @(
    '.gitignore', '.gitattributes', 'AGENTS.md', 'README.md', 'STATUS.md', 'MODLOG.md',
    'THIRD-PARTY-NOTICES.md', 'docs/requirements.md', 'docs/DESIGN.md',
    'docs/provenance.md', 'docs/workflow-setup.md', 'docs/playtests/TEMPLATE.md', 'docs/analysis-tools.md',
    'scripts/Test-Workflow.ps1'
)
foreach ($requiredPath in $requiredPaths) {
    if (-not (Test-Path -LiteralPath (Join-Path $repositoryRoot $requiredPath) -PathType Leaf)) {
        throw "Missing workflow file: $requiredPath"
    }
}

$allowedPaths = $requiredPaths + @(
    'docs/playtests/2026-10-08-example.md', 'docs/evidence/example.md',
    'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'src/main.rs', 'src/assets/reader.rs', 'tests/tank_rules.rs'
)
$excludedPaths = @(
    '.scratch/references/ai-game-modding-guides/AGENTS.md', '.scratch/evidence/setup/toolchain-probe.rs',
    'private/game-data/original.rs', 'local/game-path.md', 'references/third-party/README.md',
    'context/artifacts/handoff.md', 'target/debug/main.rs', '.env', 'src/settings.local.rs',
    'src/game-data/recovered.rs', 'docs/dumps/output.md', 'docs/captures/frame.png',
    'Insaniquarium.exe', 'bass.dll', 'assets/fish.gif', 'assets/tank.jpg', 'assets/mask.png',
    'sounds/eat.au', 'music/theme.mo3', 'music/theme.ogg', 'data/raw.txt', 'data/raw.xml',
    'assets/raw.luc', 'capture.raw', 'process.dmp', 'analysis.gpr', 'unknown-file.txt'
)
foreach ($ignoreCase in @(
    @{ Paths = $allowedPaths; ExpectedIgnored = $false },
    @{ Paths = $excludedPaths; ExpectedIgnored = $true }
)) {
    foreach ($candidatePath in $ignoreCase.Paths) {
        $ignoreArguments = @('-C', $repositoryRoot, 'check-ignore', '--quiet', '--no-index', '--', $candidatePath)
        & $gitExecutable @ignoreArguments
        $ignoreExitCode = $LASTEXITCODE
        if ($ignoreExitCode -notin @(0, 1)) { throw "Ignore check failed: $candidatePath" }
        if (($ignoreExitCode -eq 0) -ne $ignoreCase.ExpectedIgnored) {
            throw "Unexpected ignore behavior: $candidatePath"
        }
    }
}

$candidateFiles = @(Invoke-RepositoryGit -NativeArguments @('ls-files', '--cached', '--others', '--exclude-standard'))
foreach ($candidateFile in $candidateFiles) {
    $candidateSegments = $candidateFile -split '/'
    if ($candidateSegments | Where-Object { $_ -in @('.scratch', 'private', 'local', 'references', 'context', 'target', 'game-data', 'extracted', 'dumps', 'captures') }) {
        throw "Private path is staged/tracked or eligible for addition: $candidateFile"
    }
    if ($candidateFile -match '(?i)(^|/)\.env($|\.)|\.local\.|\.(exe|dll|gif|jpg|png|ogg|au|mo3|luc|dmp|raw|gpr)$') {
        throw "Protected data file is staged/tracked or eligible for addition: $candidateFile"
    }
}
$documentPaths = @($candidateFiles | Where-Object { $_ -like '*.md' } | Sort-Object -Unique)

function Get-HeadingAnchor {
    param([string]$Heading)
    $plainHeading = $Heading.Trim().ToLowerInvariant()
    $plainHeading = [regex]::Replace($plainHeading, '[^\p{L}\p{N}_ -]', '')
    return ($plainHeading -replace ' ', '-')
}

$linkCount = 0
$anchorCount = 0
foreach ($documentPath in $documentPaths) {
    $absoluteDocumentPath = Join-Path $repositoryRoot $documentPath
    $documentContent = Get-Content -LiteralPath $absoluteDocumentPath -Raw
    if ($documentContent -match '(?m)[\t ]+$') { throw "Trailing whitespace: $documentPath" }
    if (-not $documentContent.EndsWith("`n")) { throw "Missing final newline: $documentPath" }
    foreach ($linkMatch in [regex]::Matches($documentContent, '(?<!!)\[[^\]\r\n]+\]\((?<target>[^)\r\n]+)\)')) {
        $linkTarget = $linkMatch.Groups['target'].Value
        if ($linkTarget -match '^[a-zA-Z][a-zA-Z0-9+.-]*:') { continue }
        $linkParts = $linkTarget -split '#', 2
        $relativeTarget = [uri]::UnescapeDataString($linkParts[0])
        $absoluteTarget = if ($relativeTarget.Length -eq 0) { $absoluteDocumentPath } else {
            [System.IO.Path]::GetFullPath((Join-Path (Split-Path -Parent $absoluteDocumentPath) $relativeTarget))
        }
        $rootPrefix = $repositoryRoot + [System.IO.Path]::DirectorySeparatorChar
        if (-not $absoluteTarget.StartsWith($rootPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "Link escapes repository in ${documentPath}: $linkTarget"
        }
        if (-not (Test-Path -LiteralPath $absoluteTarget -PathType Leaf)) {
            throw "Broken local link in ${documentPath}: $linkTarget"
        }
        $linkCount++
        if ($linkParts.Count -eq 2 -and $linkParts[1].Length -gt 0) {
            $targetContent = Get-Content -LiteralPath $absoluteTarget -Raw
            $targetAnchors = @()
            $slugCounts = @{}
            foreach ($headingMatch in [regex]::Matches($targetContent, '(?m)^#{1,6}\s+(.+?)\s*#*\s*$')) {
                $slug = Get-HeadingAnchor -Heading $headingMatch.Groups[1].Value
                if ($slugCounts.ContainsKey($slug)) { $slugCounts[$slug]++ } else { $slugCounts[$slug] = 0 }
                $targetAnchors += if ($slugCounts[$slug] -eq 0) { $slug } else { "$slug-$($slugCounts[$slug])" }
            }
            $requestedAnchor = [uri]::UnescapeDataString($linkParts[1])
            if ($requestedAnchor -cnotin $targetAnchors) {
                throw "Broken heading anchor in ${documentPath}: $linkTarget"
            }
            $anchorCount++
        }
    }
}

$statusArguments = @('diff', '--check')
Invoke-RepositoryGit -NativeArguments $statusArguments | Out-Null
Invoke-RepositoryGit -NativeArguments @('diff', '--cached', '--check') | Out-Null
[pscustomobject]@{
    Result = 'Passed'
    RequiredFiles = $requiredPaths.Count
    Documents = $documentPaths.Count
    LocalLinks = $linkCount
    HeadingAnchors = $anchorCount
    AllowedPathChecks = $allowedPaths.Count
    ExcludedPathChecks = $excludedPaths.Count
    EligibleFiles = $candidateFiles.Count
    RuntimeValidation = 'Not tested; this checks the workflow only.'
} | ConvertTo-Json
