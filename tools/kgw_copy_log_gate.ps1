Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Repository = Split-Path -Parent $PSScriptRoot
$NodeJsRel = "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js"
$FrontendTestRel = "tools/kgw_start_button_frontend_tests.cjs"
$LibRsRel = "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs"
$CargoTomlRel = "apps/kaspa-gateway-desktop/src-tauri/Cargo.toml"

$NodeJs = Join-Path $Repository $NodeJsRel
$FrontendTest = Join-Path $Repository $FrontendTestRel
$LibRs = Join-Path $Repository $LibRsRel
$CargoToml = Join-Path $Repository $CargoTomlRel

Push-Location $Repository

$Failures = New-Object System.Collections.Generic.List[string]

function Add-Failure {
    param([Parameter(Mandatory)][string]$Message)
    $Failures.Add($Message)
}

function Read-RequiredFile {
    param([Parameter(Mandatory)][string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        Add-Failure "Missing required file: $Path"
        return ""
    }

    return Get-Content -LiteralPath $Path -Raw
}

function Extract-Between {
    param(
        [Parameter(Mandatory)][AllowEmptyString()][string]$Text,
        [Parameter(Mandatory)][string]$Start,
        [Parameter(Mandatory)][string]$End
    )

    $startIndex = $Text.IndexOf($Start, [System.StringComparison]::Ordinal)
    if ($startIndex -lt 0) { return "" }

    $endIndex = $Text.IndexOf($End, $startIndex + $Start.Length, [System.StringComparison]::Ordinal)
    if ($endIndex -le $startIndex) { return "" }

    return $Text.Substring($startIndex, $endIndex - $startIndex)
}

function Assert-Contains {
    param(
        [Parameter(Mandatory)][AllowEmptyString()][string]$Text,
        [Parameter(Mandatory)][string]$Needle,
        [Parameter(Mandatory)][string]$Message
    )

    if (-not $Text.Contains($Needle)) {
        Add-Failure "$Message Missing: $Needle"
    }
}

function Assert-NotContains {
    param(
        [Parameter(Mandatory)][AllowEmptyString()][string]$Text,
        [Parameter(Mandatory)][string]$Needle,
        [Parameter(Mandatory)][string]$Message
    )

    if ($Text.Contains($Needle)) {
        Add-Failure "$Message Forbidden: $Needle"
    }
}

function Run-LocalCommand {
    param(
        [Parameter(Mandatory)][string]$Label,
        [Parameter(Mandatory)][string]$FilePath,
        [string[]]$Arguments = @()
    )

    Write-Host "Running: $Label"
    & $FilePath @Arguments
    if ($LASTEXITCODE -ne 0) {
        Add-Failure "$Label failed with exit code $LASTEXITCODE"
    }
}

$nodeSource = Read-RequiredFile -Path $NodeJs
$testSource = Read-RequiredFile -Path $FrontendTest
$libSource = Read-RequiredFile -Path $LibRs
$cargoSource = Read-RequiredFile -Path $CargoToml

$renderStart = $nodeSource.IndexOf("function renderNetworkPanel", [System.StringComparison]::Ordinal)
if ($renderStart -lt 0) {
    Add-Failure "Could not find renderNetworkPanel."
} else {
    $renderSource = $nodeSource.Substring($renderStart)
    $copyIndex = $renderSource.IndexOf('data-node-action="copy-log"', [System.StringComparison]::Ordinal)
    $clearIndex = $renderSource.IndexOf('data-node-action="clear-log"', [System.StringComparison]::Ordinal)
    $settingsPanelIndex = if ($copyIndex -ge 0) { $renderSource.LastIndexOf('data-node-inner-panel="settings"', $copyIndex, [System.StringComparison]::Ordinal) } else { -1 }
    $logPanelIndex = if ($copyIndex -ge 0) { $renderSource.LastIndexOf('data-node-inner-panel="log"', $copyIndex, [System.StringComparison]::Ordinal) } else { -1 }
    $settingsPanel = Extract-Between -Text $renderSource -Start 'data-node-inner-panel="settings"' -End 'data-node-inner-panel="log"'

    Assert-NotContains -Text $settingsPanel -Needle 'data-node-action="copy-log"' -Message "Copy Log must not exist in Settings."
    if ($copyIndex -lt 0 -or $logPanelIndex -lt 0 -or ($settingsPanelIndex -ge 0 -and $settingsPanelIndex -gt $logPanelIndex)) {
        Add-Failure "Copy Log must exist in Live Node Monitor."
    }
    if ($clearIndex -lt 0 -or $logPanelIndex -lt 0 -or $clearIndex -lt $logPanelIndex) {
        Add-Failure "Clear Log must exist in Live Node Monitor."
    }
}

$copyMarkupMatches = [regex]::Matches($nodeSource, 'data-node-action="copy-log"')
if ($copyMarkupMatches.Count -ne 1) {
    Add-Failure "Copy Log markup must appear exactly once in the node template; found $($copyMarkupMatches.Count)."
}

$handlerMatches = [regex]::Matches($nodeSource, 'action === "copy-log" \|\| action === "clear-log"')
if ($handlerMatches.Count -ne 1) {
    Add-Failure "The delegated Copy Log handler must be attached exactly once; found $($handlerMatches.Count)."
}

$handlerStart = $nodeSource.IndexOf("async function kgwNodeHandleLogActionV29", [System.StringComparison]::Ordinal)
if ($handlerStart -lt 0) {
    Add-Failure "The node log action handler was not found."
    $handlerSource = ""
} else {
    $handlerSource = $nodeSource.Substring($handlerStart)
}

$copyBlock = Extract-Between -Text $handlerSource -Start 'if (action === "copy-log") {' -End 'if (action === "clear-log") {'
Assert-Contains -Text $copyBlock -Needle "kgwNodeTraceActiveNetworkR1" -Message "Copy Log must resolve the active network."
Assert-Contains -Text $copyBlock -Needle "kgwNodeReadClipboardRawLogBufferV1(copyNetwork)" -Message "Copy Log must read the selected active-network raw buffer."
Assert-Contains -Text $copyBlock -Needle "buffer.isPlaceholder" -Message "Copy Log must reject placeholder log text."
Assert-Contains -Text $copyBlock -Needle "non-empty raw log buffer" -Message "Copy Log must reject empty logs with an explicit error."
Assert-Contains -Text $copyBlock -Needle "await kgwNodeDispatchClipboardWriteV1" -Message "Copy Log must await native clipboard success."
Assert-Contains -Text $copyBlock -Needle "kgwNodeCopyLogFailureV1" -Message "Copy Log native failure must remain an error."
Assert-Contains -Text $copyBlock -Needle 'frontend.copy_log_succeeded' -Message "Copy Log success trace must exist."
Assert-NotContains -Text $copyBlock -Needle "navigator.clipboard.writeText" -Message "Copy Log must not use browser clipboard as the primary path."
Assert-NotContains -Text $copyBlock -Needle "document.execCommand" -Message "Copy Log must not use execCommand as the primary path."

$awaitIndex = $copyBlock.IndexOf("await kgwNodeDispatchClipboardWriteV1", [System.StringComparison]::Ordinal)
$successFeedbackIndex = $copyBlock.IndexOf('kgwNodeTranslateRuntimeV29("log.copied"', [System.StringComparison]::Ordinal)
if ($awaitIndex -lt 0 -or $successFeedbackIndex -lt 0 -or $successFeedbackIndex -lt $awaitIndex) {
    Add-Failure "Copied feedback must be displayed only after the awaited native clipboard write."
}

Assert-Contains -Text $nodeSource -Needle 'frontend.copy_log_click_observed' -Message "Copy Log physical click trace stage is required."
Assert-Contains -Text $nodeSource -Needle 'frontend.copy_log_network_resolved' -Message "Copy Log network trace stage is required."
Assert-Contains -Text $nodeSource -Needle 'frontend.copy_log_content_prepared' -Message "Copy Log content trace stage is required."
Assert-Contains -Text $nodeSource -Needle 'frontend.copy_log_dispatched' -Message "Copy Log dispatch trace stage is required."
Assert-Contains -Text $nodeSource -Needle 'frontend.copy_log_failed' -Message "Copy Log failure trace stage is required."
Assert-Contains -Text $nodeSource -Needle "kgw_copy_text_to_clipboard_v1" -Message "Copy Log must call the project-owned native clipboard command."
Assert-Contains -Text $nodeSource -Needle "kgwNodeNormalizeClipboardLineEndingsV1" -Message "Copy Log must normalize line endings safely."
Assert-Contains -Text $nodeSource -Needle "kgwNodeSha256HexV1" -Message "Copy Log should include SHA-256 metadata when practical."
Assert-NotContains -Text $nodeSource -Needle "completeClipboardContent" -Message "Trace metadata must not expose clipboard content."

Assert-Contains -Text $testSource -Needle "Copy Log must invoke native clipboard exactly once" -Message "Frontend tests must prove exactly one clipboard invoke."
Assert-Contains -Text $testSource -Needle "Copy Log must pass testnet10 when testnet10 is active" -Message "Frontend tests must prove active-network isolation."
Assert-Contains -Text $testSource -Needle "Copy Log must not mix mainnet into testnet10" -Message "Frontend tests must prevent network mixing."
Assert-Contains -Text $testSource -Needle "Large Copy Log text must not be silently truncated or reordered" -Message "Frontend tests must cover large raw logs."
Assert-Contains -Text $testSource -Needle "Copy Log trace must exclude raw clipboard content" -Message "Frontend tests must verify trace redaction."

Assert-Contains -Text $libSource -Needle "kgw_copy_text_to_clipboard_v1" -Message "Native clipboard command must be registered."
Assert-Contains -Text $libSource -Needle "native.clipboard_write_entered" -Message "Native clipboard entry trace is required."
Assert-Contains -Text $libSource -Needle "native.clipboard_write_succeeded" -Message "Native clipboard success trace is required."
Assert-Contains -Text $libSource -Needle "native.clipboard_write_failed" -Message "Native clipboard failure trace is required."
Assert-Contains -Text $libSource -Needle "tauri_plugin_clipboard_manager::ClipboardExt" -Message "Native clipboard must use the official Tauri clipboard-manager plugin."
Assert-Contains -Text $cargoSource -Needle "tauri-plugin-clipboard-manager" -Message "Desktop Cargo.toml must depend on the official clipboard-manager plugin."

Run-LocalCommand "Copy Log frontend syntax" "node" @("--check", $FrontendTestRel)
Run-LocalCommand "Copy Log frontend regression tests" "node" @($FrontendTestRel)
Run-LocalCommand "targeted Tauri clipboard tests" "cargo" @("test", "-p", "kaspa-gateway-desktop", "kgw_clipboard")

if ($Failures.Count -gt 0) {
    Write-Host "KGW Copy Log gate FAILED" -ForegroundColor Red
    foreach ($failure in $Failures) {
        Write-Host "- $failure" -ForegroundColor Red
    }
    Pop-Location
    exit 1
}

Write-Host "KGW Copy Log gate PASSED"
Pop-Location
