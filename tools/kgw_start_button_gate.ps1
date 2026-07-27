$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$Failures = New-Object System.Collections.Generic.List[string]

function Add-Failure {
    param([Parameter(Mandatory)][string]$Message)
    [void]$Failures.Add($Message)
}

function Run-LocalCommand {
    param(
        [Parameter(Mandatory)][string]$Label,
        [Parameter(Mandatory)][string]$FilePath,
        [string[]]$Arguments = @()
    )

    Write-Host "Running: $Label"
    $process = Start-Process -FilePath $FilePath -ArgumentList $Arguments -WorkingDirectory $RepoRoot -NoNewWindow -Wait -PassThru
    if ($process.ExitCode -ne 0) {
        Add-Failure "$Label failed with exit code $($process.ExitCode)"
    }
}

function Read-RequiredFile {
    param([Parameter(Mandatory)][string]$RelativePath)
    $Path = Join-Path $RepoRoot $RelativePath
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        Add-Failure "Missing required file: $RelativePath"
        return ""
    }
    return Get-Content -Raw -LiteralPath $Path
}

$NodeJsRel = "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js"
$NodeJs = Read-RequiredFile $NodeJsRel

if ($NodeJs) {
    $settingsIndex = $NodeJs.IndexOf('data-node-inner-panel="settings"')
    $logIndex = $NodeJs.IndexOf('data-node-inner-panel="log"')
    $renderEndIndex = $NodeJs.IndexOf('</div>`;', $logIndex)

    if ($settingsIndex -lt 0 -or $logIndex -lt 0 -or $renderEndIndex -lt 0 -or $settingsIndex -ge $logIndex) {
        Add-Failure "Could not verify Settings and Live Node Monitor panel ordering."
    }
    else {
        $settingsBlock = $NodeJs.Substring($settingsIndex, $logIndex - $settingsIndex)
        $logBlock = $NodeJs.Substring($logIndex, $renderEndIndex - $logIndex)

        if ($settingsBlock -notmatch 'data-node-action="start"') { Add-Failure "Settings panel is missing Start." }
        if ($settingsBlock -notmatch 'data-node-action="stop"') { Add-Failure "Settings panel is missing Stop." }
        if ($settingsBlock -notmatch 'data-node-network-enabled') { Add-Failure "Settings panel is missing network enable control." }
        if ($logBlock -match 'data-node-action="start"') { Add-Failure "Live Node Monitor contains Start." }
        if ($logBlock -match 'data-node-action="stop"') { Add-Failure "Live Node Monitor contains Stop." }
        if ($logBlock -match 'data-node-network-enabled') { Add-Failure "Live Node Monitor contains network enable control." }
    }

    $startCount = ([regex]::Matches($NodeJs, '<button[^>]+data-node-action="start"')).Count
    $stopCount = ([regex]::Matches($NodeJs, '<button[^>]+data-node-action="stop"')).Count
    if ($startCount -ne 1) { Add-Failure "Expected exactly one Start control template, found $startCount." }
    if ($stopCount -ne 1) { Add-Failure "Expected exactly one Stop control template, found $stopCount." }

    if ($NodeJs -match '<button[^>]+id=[^>]+data-node-action="start"' -or $NodeJs -match '<button[^>]+data-node-action="start"[^>]+id=') {
        Add-Failure "Start control uses an ID that can duplicate across networks."
    }
    if ($NodeJs -match '<button[^>]+id=[^>]+data-node-action="stop"' -or $NodeJs -match '<button[^>]+data-node-action="stop"[^>]+id=') {
        Add-Failure "Stop control uses an ID that can duplicate across networks."
    }

    $syntheticPatterns = @(
        'appendLog\([^)]*initialized',
        'appendLog\([^)]*start response',
        'appendLog\([^)]*started successfully',
        'appendLog\([^)]*synchronized',
        'appendLog\([^)]*connected',
        'appendLog\([^)]*Node settings saved successfully',
        'appendLog\([^)]*Node defaults restored successfully'
    )

    foreach ($Pattern in $syntheticPatterns) {
        if ($NodeJs -match $Pattern) {
            Add-Failure "Synthetic raw-log message detected by pattern: $Pattern"
        }
    }
}

Run-LocalCommand "frontend regression syntax" "node" @("--check", "tools/kgw_start_button_frontend_tests.cjs")
Run-LocalCommand "frontend start button regression tests" "node" @("tools/kgw_start_button_frontend_tests.cjs")
Run-LocalCommand "targeted Tauri IPC regression tests" "cargo" @("test", "--manifest-path", "apps/kaspa-gateway-desktop/src-tauri/Cargo.toml", "--test", "integrated_runtime_ipc_smoke_tests")

if ($Failures.Count -gt 0) {
    Write-Host "KGW start button gate FAILED"
    foreach ($Failure in $Failures) {
        Write-Host ("- " + $Failure)
    }
    exit 1
}

Write-Host "KGW start button gate PASSED"
exit 0
