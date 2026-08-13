param(
    [Parameter(Mandatory)]
    [string]$Repository,

    [AllowEmptyString()]
    [string]$ReuseSuccessfulE2EArtifact = "",

    [switch]$CommitOnSuccess
)

if ($PSVersionTable.PSVersion.Major -lt 7) {
    throw "Kaspa Gateway full local gate requires PowerShell 7 or later. Launch with (Get-Command pwsh -ErrorAction Stop).Source; Windows PowerShell $($PSVersionTable.PSVersion) is not supported."
}

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false

. (Join-Path $PSScriptRoot "kgw_zero_touch_evidence.ps1")
Assert-KgwZeroTouchPowerShell7
$RequiredPowerShellPath = (Get-Command pwsh -ErrorAction Stop).Source

function Invoke-GateCommand {
    param(
        [Parameter(Mandatory)][string]$Label,
        [Parameter(Mandatory)][string]$FilePath,
        [string[]]$Arguments = @(),
        [Parameter(Mandatory)][string]$WorkingDirectory
    )

    Write-Host "Running: $Label"
    Push-Location $WorkingDirectory
    try {
        & $FilePath @Arguments
        $code = if ($null -eq $LASTEXITCODE) { 0 } else { [int]$LASTEXITCODE }
        if ($code -ne 0) {
            throw "$Label failed with exit code $code"
        }
    }
    finally {
        Pop-Location
    }
}

function Test-PathDirty {
    param([Parameter(Mandatory)][string]$Pathspec)
    $status = & git -C $Repository status --porcelain -- $Pathspec
    return -not [string]::IsNullOrWhiteSpace(($status -join "`n"))
}

function Restore-GeneratedTauriSchemasIfNewlyDirty {
    param([Parameter(Mandatory)][bool]$InitiallyDirty)

    $schemaPath = "apps/kaspa-gateway-desktop/src-tauri/gen/schemas"
    if ($InitiallyDirty) {
        Write-Warning "Generated Tauri schemas were already dirty before the gate; preserving them."
        return
    }

    if (Test-PathDirty -Pathspec $schemaPath) {
        Write-Host "Restoring generated Tauri schema changes produced by local validation"
        & git -C $Repository restore --worktree -- $schemaPath
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to restore generated Tauri schema changes"
        }
    }
}

function Invoke-PowerShellParserChecks {
    $paths = @(
        "tools/kgw_full_local_gate.ps1",
        "tools/kgw_true_raw_log_gate.ps1",
        "tools/kgw_live_raw_log_matrix.ps1",
        "tools/kgw_raw_log_clipboard_capture.ps1",
        "tools/kgw_desktop_diagnostic_launch.ps1",
        "tools/kgw_zero_touch_e2e.ps1",
        "tools/kgw_zero_touch_evidence.ps1",
        "tools/kgw_zero_touch_result_writer_tests.ps1",
        "e2e/helpers/kgw_windows_clipboard.ps1",
        "e2e/helpers/kgw_windows_evidence.ps1"
    )

    foreach ($relativePath in $paths) {
        $path = Join-Path $Repository $relativePath
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
            throw "Missing PowerShell script for parser check: $relativePath"
        }
        $parseErrors = $null
        [void][System.Management.Automation.Language.Parser]::ParseFile($path, [ref]$null, [ref]$parseErrors)
        if ($parseErrors -and $parseErrors.Count -gt 0) {
            $messages = @($parseErrors | ForEach-Object { $_.Message }) -join " | "
            throw "PowerShell parser check failed for ${relativePath}: $messages"
        }
    }
}

function Invoke-GraphifyRefresh {
    $graphPath = Join-Path $Repository "graphify-out/graph.json"
    $query = "How does the zero-touch WebdriverIO E2E suite verify raw node and bridge logs and clipboard hashes?"
    if (Test-Path -LiteralPath $graphPath) {
        Invoke-GateCommand `
            -Label "Graphify code-only incremental refresh" `
            -FilePath "graphify" `
            -Arguments @("update", ".") `
            -WorkingDirectory $Repository
    } else {
        Invoke-GateCommand `
            -Label "Graphify code-only extraction" `
            -FilePath "graphify" `
            -Arguments @("extract", ".", "--code-only") `
            -WorkingDirectory $Repository
    }

    Invoke-GateCommand `
        -Label "Graphify post-change query" `
        -FilePath "graphify" `
        -Arguments @("query", $query, "--budget", "4000") `
        -WorkingDirectory $Repository
}

function Commit-ScopedChanges {
    $paths = @(
        ".gitignore",
        "Cargo.lock",
        "apps/kaspa-gateway-desktop/frontend/index.html",
        "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
        "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.template.js",
        "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
        "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.template.js",
        "apps/kaspa-gateway-desktop/src-tauri/Cargo.toml",
        "apps/kaspa-gateway-desktop/src-tauri/build.rs",
        "apps/kaspa-gateway-desktop/src-tauri/tauri.conf.json",
        "apps/kaspa-gateway-desktop/src-tauri/tauri.e2e.conf.json",
        "apps/kaspa-gateway-desktop/src-tauri/capabilities-e2e",
        "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs",
        "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs",
        "apps/kaspa-gateway-desktop/src-tauri/tests/integrated_runtime_ipc_smoke_tests.rs",
        "tools/kgw_desktop_diagnostic_launch.ps1",
        "tools/kgw_live_raw_log_matrix.ps1",
        "tools/kgw_raw_log_clipboard_capture.ps1",
        "tools/kgw_true_raw_log_frontend_tests.cjs",
        "tools/kgw_true_raw_log_gate.ps1",
        "tools/kgw_zero_touch_e2e.ps1",
        "tools/kgw_zero_touch_evidence.ps1",
        "tools/kgw_zero_touch_result_writer_tests.ps1",
        "tools/kgw_full_local_gate.ps1",
        "e2e/package.json",
        "e2e/package-lock.json",
        "e2e/wdio.conf.mjs",
        "e2e/capabilities",
        "e2e/helpers",
        "e2e/specs"
    )

    & git -C $Repository add -- $paths
    if ($LASTEXITCODE -ne 0) {
        throw "git add failed"
    }

    & git -C $Repository commit -m "fix: automate and verify true raw node and bridge logs"
    if ($LASTEXITCODE -ne 0) {
        throw "git commit failed"
    }
}

function Assert-SuccessfulE2EArtifactReusable {
    param([Parameter(Mandatory)][string]$ArtifactDirectory)

    $resolved = (Resolve-Path -LiteralPath $ArtifactDirectory -ErrorAction Stop).Path
    $result = Test-KgwZeroTouchResultIntegrity -Repository $Repository -ArtifactDirectory $resolved
    if (-not $result.passed) {
        $message = "Rejected E2E artifact reuse: " + (($result.errors | ForEach-Object { [string]$_ }) -join " | ")
        throw $message
    }

    $stageText = @($result.evidence.passed_stages) -join ", "
    Write-Host "Reusing verified zero-touch E2E artifact: $resolved"
    Write-Host "Verified reused stages: $stageText"
    Write-Host "Live WebdriverIO E2E was not rerun because reuse validation passed."
    return $resolved
}

$Repository = (Resolve-Path -LiteralPath $Repository).Path
$env:KGW_REQUIRED_PWSH_PATH = $RequiredPowerShellPath
$schemaInitiallyDirty = Test-PathDirty -Pathspec "apps/kaspa-gateway-desktop/src-tauri/gen/schemas"

try {
    Invoke-PowerShellParserChecks

    Invoke-GateCommand `
        -Label "Rust formatting check" `
        -FilePath "cargo" `
        -Arguments @("fmt", "--all", "--", "--check") `
        -WorkingDirectory $Repository

    Invoke-GateCommand `
        -Label "Desktop E2E feature cargo check" `
        -FilePath "cargo" `
        -Arguments @("check", "--locked", "-p", "kaspa-gateway-desktop", "--features", "e2e-test") `
        -WorkingDirectory $Repository

    Invoke-GateCommand `
        -Label "Rust typed raw log tests" `
        -FilePath "cargo" `
        -Arguments @("test", "-p", "kaspa-gateway-desktop", "typed_raw_log", "--test", "integrated_runtime_ipc_smoke_tests", "--", "--nocapture") `
        -WorkingDirectory $Repository

    Invoke-GateCommand `
        -Label "Rust child raw log fixture test" `
        -FilePath "cargo" `
        -Arguments @("test", "-p", "kaspa-gateway-desktop", "child_stdout_and_stderr_fixtures_survive_unchanged", "--test", "integrated_runtime_ipc_smoke_tests", "--", "--nocapture") `
        -WorkingDirectory $Repository

    Invoke-GateCommand `
        -Label "Rust raw log isolation test" `
        -FilePath "cargo" `
        -Arguments @("test", "-p", "kaspa-gateway-desktop", "raw_log_buffers_are_isolated_by_network_and_role_with_process_wide_bridge_output", "--test", "integrated_runtime_ipc_smoke_tests", "--", "--nocapture") `
        -WorkingDirectory $Repository

    Invoke-GateCommand `
        -Label "Node frontend syntax" `
        -FilePath "node" `
        -Arguments @("--check", "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js") `
        -WorkingDirectory $Repository

    Invoke-GateCommand `
        -Label "Bridge frontend syntax" `
        -FilePath "node" `
        -Arguments @("--check", "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js") `
        -WorkingDirectory $Repository

    Invoke-GateCommand `
        -Label "True raw log frontend tests" `
        -FilePath "node" `
        -Arguments @("tools/kgw_true_raw_log_frontend_tests.cjs") `
        -WorkingDirectory $Repository

    Invoke-GateCommand `
        -Label "E2E workspace checks" `
        -FilePath "npm" `
        -Arguments @("run", "check") `
        -WorkingDirectory (Join-Path $Repository "e2e")

    Invoke-GateCommand `
        -Label "Zero-touch result writer tests" `
        -FilePath $RequiredPowerShellPath `
        -Arguments @("-NoLogo", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", "tools/kgw_zero_touch_result_writer_tests.ps1") `
        -WorkingDirectory $Repository

    Invoke-GateCommand `
        -Label "True raw log gate" `
        -FilePath $RequiredPowerShellPath `
        -Arguments @("-NoLogo", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", "tools/kgw_true_raw_log_gate.ps1") `
        -WorkingDirectory $Repository

    if ([string]::IsNullOrWhiteSpace($ReuseSuccessfulE2EArtifact)) {
        Invoke-GateCommand `
            -Label "Zero-touch live E2E suite" `
            -FilePath $RequiredPowerShellPath `
            -Arguments @("-NoLogo", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", "tools/kgw_zero_touch_e2e.ps1", "-Repository", $Repository) `
            -WorkingDirectory $Repository
    } else {
        [void](Assert-SuccessfulE2EArtifactReusable -ArtifactDirectory $ReuseSuccessfulE2EArtifact)
    }

    Invoke-GraphifyRefresh
    Restore-GeneratedTauriSchemasIfNewlyDirty -InitiallyDirty $schemaInitiallyDirty

    Invoke-GateCommand `
        -Label "Git diff whitespace check" `
        -FilePath "git" `
        -Arguments @("diff", "--check") `
        -WorkingDirectory $Repository

    if ($CommitOnSuccess) {
        Commit-ScopedChanges
        $sha = (& git -C $Repository rev-parse HEAD).Trim()
        Write-Host "Committed local changes at $sha"
    } else {
        Write-Host "CommitOnSuccess was not provided; no commit was created."
    }
}
catch {
    Write-Error $_.Exception.Message
    exit 1
}

Write-Host "KGW full local gate passed. No push was performed."
exit 0
