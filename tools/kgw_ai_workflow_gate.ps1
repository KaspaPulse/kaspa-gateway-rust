$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$Failures = New-Object System.Collections.Generic.List[string]

function Add-Failure {
    param(
        [Parameter(Mandatory)]
        [string]$Message
    )

    [void]$Failures.Add($Message)
}

function Read-RequiredFile {
    param(
        [Parameter(Mandatory)]
        [string]$RelativePath
    )

    $Path = Join-Path $RepoRoot $RelativePath
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        Add-Failure "Missing required file: $RelativePath"
        return ""
    }

    return Get-Content -Raw -LiteralPath $Path
}

function Test-JsonFile {
    param(
        [Parameter(Mandatory)]
        [string]$RelativePath
    )

    $Text = Read-RequiredFile $RelativePath
    if (-not $Text) {
        return
    }

    try {
        $null = $Text | ConvertFrom-Json
    }
    catch {
        Add-Failure "Invalid JSON in $RelativePath`: $($_.Exception.Message)"
    }
}

function Require-AgentsPattern {
    param(
        [Parameter(Mandatory)]
        [string]$Text,

        [Parameter(Mandatory)]
        [string]$Pattern,

        [Parameter(Mandatory)]
        [string]$Description
    )

    if ($Text -notmatch $Pattern) {
        Add-Failure "AGENTS.md is missing required policy: $Description"
    }
}

$AgentsText = Read-RequiredFile "AGENTS.md"
$null = Read-RequiredFile ".codex/skills/graphify/SKILL.md"
Test-JsonFile ".codex/hooks.json"
Test-JsonFile "graphify-out/graph.json"

if ($AgentsText) {
    $GraphifySections = [regex]::Matches($AgentsText, "(?im)^##\s+.*graphify.*$")
    if ($GraphifySections.Count -gt 1) {
        Add-Failure "AGENTS.md contains duplicate Graphify sections."
    }

    Require-AgentsPattern $AgentsText "(?is)local-first workflow.*build, test, and run locally" "local-first build/test/run rule"
    Require-AgentsPattern $AgentsText "(?is)display the real native process stdout and stderr" "raw stdout and stderr display rule"
    Require-AgentsPattern $AgentsText "(?is)database directories, ports, runtime state, logs, and operating-system processes isolated by network" "network isolation rule"
    Require-AgentsPattern $AgentsText "(?is)testnet12.*experimental.*disabled by default.*explicit opt-in" "testnet12 explicit opt-in rule"
    Require-AgentsPattern $AgentsText "(?is)definition of done.*formatting.*javascript.*rust.*graphify" "definition-of-done test rules"
    Require-AgentsPattern $AgentsText "(?is)do not push" "no-push rule"
}

if ($Failures.Count -gt 0) {
    Write-Host "KGW AI workflow gate FAILED"
    foreach ($Failure in $Failures) {
        Write-Host ("- " + $Failure)
    }
    exit 1
}

Write-Host "KGW AI workflow gate PASSED"
Write-Host "AGENTS.md, Graphify skill, hooks JSON, graph JSON, and required workflow policies are present."
exit 0
