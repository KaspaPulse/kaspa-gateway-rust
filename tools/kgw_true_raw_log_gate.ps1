Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Repository = Split-Path -Parent $PSScriptRoot
$FrontendTestRel = "tools/kgw_true_raw_log_frontend_tests.cjs"
$ClipboardCaptureRel = "tools/kgw_raw_log_clipboard_capture.ps1"
$LiveMatrixRel = "tools/kgw_live_raw_log_matrix.ps1"
$TrueRawGateRel = "tools/kgw_true_raw_log_gate.ps1"
$NodeJsRel = "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js"
$BridgeJsRel = "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
$RuntimeRsRel = "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs"
$ZeroTouchE2eRel = "tools/kgw_zero_touch_e2e.ps1"
$ZeroTouchEvidenceRel = "tools/kgw_zero_touch_evidence.ps1"
$ZeroTouchResultWriterTestsRel = "tools/kgw_zero_touch_result_writer_tests.ps1"
$FullLocalGateRel = "tools/kgw_full_local_gate.ps1"
$E2eClipboardHelperRel = "e2e/helpers/kgw_windows_clipboard.ps1"
$E2eEvidenceHelperRel = "e2e/helpers/kgw_windows_evidence.ps1"
$E2eWindowsHelperRel = "e2e/helpers/windows.mjs"

$Failures = New-Object System.Collections.Generic.List[string]
$script:CargoTargetDir = $null
$script:FreshCargoTargetDirUsed = $false

function Add-Failure {
    param([Parameter(Mandatory)][string]$Message)
    $Failures.Add($Message)
}

function Read-RequiredFile {
    param([Parameter(Mandatory)][string]$RelativePath)

    $path = Join-Path $Repository $RelativePath
    if (-not (Test-Path -LiteralPath $path)) {
        Add-Failure "Missing required file: $RelativePath"
        return ""
    }

    return Get-Content -LiteralPath $path -Raw
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

function Test-LibRocksDbArtifactInconsistency {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Text)

    return $Text -match "librocksdb-sys" -and
        $Text -match "(inconsistent|missing|not found|could not find|failed to read|no such file|access is denied|LNK1104|corrupt)"
}

function Invoke-CommandChecked {
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

function Invoke-CargoChecked {
    param(
        [Parameter(Mandatory)][string]$Label,
        [string[]]$Arguments = @()
    )

    Write-Host "Running: $Label"

    try {
        $result = Invoke-CargoProcess -Arguments $Arguments -CargoTargetDir $script:CargoTargetDir
        $text = $result.Text
        if ($text.Trim().Length -gt 0) {
            Write-Host $text
        }

        if ($result.ExitCode -ne 0 -and -not $script:FreshCargoTargetDirUsed -and (Test-LibRocksDbArtifactInconsistency -Text $text)) {
            $script:FreshCargoTargetDirUsed = $true
            $script:CargoTargetDir = Join-Path ([System.IO.Path]::GetTempPath()) ("kgw-true-raw-log-target-{0}-{1}" -f $PID, [System.Guid]::NewGuid().ToString("N"))
            New-Item -ItemType Directory -Path $script:CargoTargetDir -Force | Out-Null
            Write-Host "librocksdb-sys artifacts look inconsistent; retrying with isolated CARGO_TARGET_DIR=$script:CargoTargetDir"

            $retry = Invoke-CargoProcess -Arguments $Arguments -CargoTargetDir $script:CargoTargetDir
            $retryText = $retry.Text
            if ($retryText.Trim().Length -gt 0) {
                Write-Host $retryText
            }
            if ($retry.ExitCode -ne 0) {
                Add-Failure "$Label failed with exit code $($retry.ExitCode) after isolated CARGO_TARGET_DIR retry"
            }
            return
        }

        if ($result.ExitCode -ne 0) {
            Add-Failure "$Label failed with exit code $($result.ExitCode)"
        }
    }
    catch {
        Add-Failure "$Label failed to launch cargo: $($_.Exception.Message)"
    }
}

function Assert-PowerShellSyntax {
    param(
        [Parameter(Mandatory)][string]$RelativePath,
        [Parameter(Mandatory)][string]$Label
    )

    $path = Join-Path $Repository $RelativePath
    $parseErrors = $null
    [void][System.Management.Automation.Language.Parser]::ParseFile($path, [ref]$null, [ref]$parseErrors)
    if ($parseErrors -and $parseErrors.Count -gt 0) {
        foreach ($parseError in $parseErrors) {
            Add-Failure "$Label syntax error: $($parseError.Message)"
        }
    }
}

function Invoke-CargoProcess {
    param(
        [string[]]$Arguments = @(),
        [AllowNull()][string]$CargoTargetDir
    )

    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = "cargo"
    $psi.WorkingDirectory = $Repository
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.Arguments = ($Arguments | ForEach-Object { ConvertTo-ProcessArgument -Argument $_ }) -join " "
    if ($CargoTargetDir) {
        $psi.Environment["CARGO_TARGET_DIR"] = $CargoTargetDir
        Write-Host "Using CARGO_TARGET_DIR=$CargoTargetDir"
    }

    $process = New-Object System.Diagnostics.Process
    $process.StartInfo = $psi
    [void]$process.Start()
    $stdout = $process.StandardOutput.ReadToEnd()
    $stderr = $process.StandardError.ReadToEnd()
    $process.WaitForExit()

    return [pscustomobject]@{
        ExitCode = $process.ExitCode
        Text = ($stdout + $stderr)
    }
}

function ConvertTo-ProcessArgument {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Argument)

    if ($Argument -notmatch '[\s"]') {
        return $Argument
    }

    return '"' + ($Argument -replace '\\', '\\' -replace '"', '\"') + '"'
}

Push-Location $Repository
try {
    . (Join-Path $Repository $ClipboardCaptureRel)

    $nodeSource = Read-RequiredFile -RelativePath $NodeJsRel
    $bridgeSource = Read-RequiredFile -RelativePath $BridgeJsRel
    $runtimeSource = Read-RequiredFile -RelativePath $RuntimeRsRel
    $frontendTestSource = Read-RequiredFile -RelativePath $FrontendTestRel
    $clipboardCaptureSource = Read-RequiredFile -RelativePath $ClipboardCaptureRel
    $liveMatrixSource = Read-RequiredFile -RelativePath $LiveMatrixRel
    $zeroTouchSource = Read-RequiredFile -RelativePath $ZeroTouchE2eRel
    $zeroTouchEvidenceSource = Read-RequiredFile -RelativePath $ZeroTouchEvidenceRel
    $fullLocalGateSource = Read-RequiredFile -RelativePath $FullLocalGateRel
    $e2eWindowsHelperSource = Read-RequiredFile -RelativePath $E2eWindowsHelperRel

    Assert-NotContains -Text $runtimeSource -Needle "kgw_raw_process_log_v1" -Message "Rust runtime source must not serialize the old raw-process envelope."

    $rawTextEnvelopePattern = @'
rawText\s*:\s*["'][^"']*kgw_raw_process_log_v1
'@.Trim()
    if (-not [regex]::IsMatch($frontendTestSource, $rawTextEnvelopePattern, [System.Text.RegularExpressions.RegexOptions]::IgnoreCase)) {
        Add-Failure "Frontend fixtures must include a transport-envelope rejection case."
    }
    if ([regex]::IsMatch($nodeSource + "`n" + $bridgeSource, 'appendLog\s*\([^)]*kgw_raw_process_log_v1', [System.Text.RegularExpressions.RegexOptions]::IgnoreCase)) {
        Add-Failure "Frontend code must not append the old transport envelope into a UI buffer."
    }

    Assert-Contains -Text $runtimeSource -Needle "KgwRuntimeRawLogEntryV1" -Message "Rust must carry typed raw log entries."
    Assert-Contains -Text $runtimeSource -Needle "sequence" -Message "Rust raw log entries must include a monotonic sequence."
    Assert-Contains -Text $runtimeSource -Needle "raw_text" -Message "Rust raw log entries must keep raw child text separate."
    Assert-Contains -Text $nodeSource -Needle "kgwNodeApplyRuntimeLogReportV1" -Message "Node UI must consume typed raw log reports."
    Assert-Contains -Text $bridgeSource -Needle "kgwBridgeApplyRuntimeLogReportV1" -Message "Bridge UI must consume typed raw log reports."
    Assert-Contains -Text $nodeSource -Needle "kgwNodeRawLogTextHasTransportWrapperV1" -Message "Node UI must reject transport wrapper text before display or copy."
    Assert-Contains -Text $bridgeSource -Needle "kgwBridgeRawLogTextHasTransportWrapperV1" -Message "Bridge UI must reject transport wrapper text before display or copy."
    Assert-Contains -Text $nodeSource -Needle 'runtimeRole: metadata.runtimeRole || "node"' -Message "Node Copy Log must carry runtime role metadata to native clipboard traces."
    Assert-Contains -Text $bridgeSource -Needle 'runtimeRole: metadata.runtimeRole || "bridge"' -Message "Bridge Copy Log must carry runtime role metadata to native clipboard traces."
    Assert-Contains -Text $bridgeSource -Needle 'bridgeInstanceId: metadata.bridgeInstanceId || ""' -Message "Bridge Copy Log must carry bridge instance metadata to native clipboard traces."
    Assert-Contains -Text $frontendTestSource -Needle "Node Copy Log equals visible raw output byte-for-byte after newline normalization" -Message "Frontend tests must verify node Copy Log raw payload."
    Assert-Contains -Text $frontendTestSource -Needle "Bridge Copy Log equals visible raw output byte-for-byte after newline normalization" -Message "Frontend tests must verify bridge Copy Log raw payload."
    Assert-Contains -Text $frontendTestSource -Needle "Bridge status summaries must be rejected as raw process output" -Message "Frontend tests must reject generated bridge summaries as raw output."
    Assert-Contains -Text $frontendTestSource -Needle "Missing bridge output must not reach native clipboard" -Message "Frontend tests must cover missing bridge output."
    Assert-Contains -Text $frontendTestSource -Needle "transport" -Message "Frontend tests must cover transport wrapper rejection."
    Assert-Contains -Text $frontendTestSource -Needle "Sequence ordering is preserved" -Message "Frontend tests must cover sequence ordering."
    Assert-Contains -Text $frontendTestSource -Needle "must not mix into" -Message "Frontend tests must cover network and role isolation."
    Assert-Contains -Text $clipboardCaptureSource -Needle "New-KgwRawLogClipboardCaptureFromClipboardV1" -Message "PowerShell tooling must capture clipboard payloads at event time."
    Assert-Contains -Text $clipboardCaptureSource -Needle "expected_sha256" -Message "PowerShell clipboard capture metadata must record expected SHA256."
    Assert-Contains -Text $clipboardCaptureSource -Needle "actual_sha256" -Message "PowerShell clipboard capture metadata must record actual SHA256."
    Assert-Contains -Text $clipboardCaptureSource -Needle "sha256_match" -Message "PowerShell clipboard capture must compare expected and actual SHA256."
    Assert-Contains -Text $liveMatrixSource -Needle 'Name = "Testnet10 Node"' -Message "Live matrix must guide Testnet10 Node first."
    Assert-Contains -Text $liveMatrixSource -Needle 'Name = "Mainnet Bridge"' -Message "Live matrix must guide Mainnet Bridge second."
    Assert-Contains -Text $liveMatrixSource -Needle 'Name = "Testnet10 Bridge"' -Message "Live matrix must guide Testnet10 Bridge third."
    Assert-Contains -Text $liveMatrixSource -Needle "Set-ClipboardSentinel" -Message "Live matrix must set a clipboard sentinel before each stage."
    Assert-Contains -Text $liveMatrixSource -Needle "Wait-ForStageCapture" -Message "Live matrix must capture Copy Log at event time before advancing."
    Assert-Contains -Text $zeroTouchSource -Needle '(Get-Command pwsh -ErrorAction Stop).Source' -Message "Zero-touch launcher must resolve the PowerShell 7 executable."
    Assert-Contains -Text $zeroTouchSource -Needle 'KGW_REQUIRED_PWSH_PATH' -Message "Zero-touch launcher must pass the resolved PowerShell 7 executable to child helpers."
    Assert-Contains -Text $zeroTouchEvidenceSource -Needle 'Write-KgwZeroTouchJsonFile' -Message "Zero-touch evidence helper must provide the shared strict JSON writer."
    Assert-Contains -Text $zeroTouchEvidenceSource -Needle 'Write-KgwZeroTouchEmergencyJsonFile' -Message "Zero-touch evidence helper must provide the emergency fallback writer."
    Assert-Contains -Text $fullLocalGateSource -Needle '$RequiredPowerShellPath' -Message "Full local gate must launch PowerShell scripts with resolved pwsh."
    Assert-NotContains -Text $fullLocalGateSource -Needle ('-FilePath ' + '"powershell"') -Message "Full local gate must not launch Windows PowerShell."
    Assert-Contains -Text $e2eWindowsHelperSource -Needle 'KGW_REQUIRED_PWSH_PATH' -Message "WDIO helper must inherit the resolved PowerShell 7 executable."

    $captureSelfTest = Test-KgwRawLogClipboardCaptureSelfTestV1
    if (-not $captureSelfTest.passed) {
        foreach ($error in $captureSelfTest.errors) {
            Add-Failure "Event-time clipboard capture deterministic self-test failed: $error"
        }
    }

    Assert-PowerShellSyntax -RelativePath $ClipboardCaptureRel -Label "Raw clipboard capture helper"
    Assert-PowerShellSyntax -RelativePath $LiveMatrixRel -Label "Live raw log matrix"
    Assert-PowerShellSyntax -RelativePath "tools/kgw_desktop_diagnostic_launch.ps1" -Label "Desktop diagnostic launcher"
    Assert-PowerShellSyntax -RelativePath $TrueRawGateRel -Label "True raw log gate"
    Assert-PowerShellSyntax -RelativePath $ZeroTouchE2eRel -Label "Zero-touch E2E launcher"
    Assert-PowerShellSyntax -RelativePath $ZeroTouchEvidenceRel -Label "Zero-touch evidence helper"
    Assert-PowerShellSyntax -RelativePath $ZeroTouchResultWriterTestsRel -Label "Zero-touch result writer tests"
    Assert-PowerShellSyntax -RelativePath $FullLocalGateRel -Label "Full local gate"
    Assert-PowerShellSyntax -RelativePath $E2eClipboardHelperRel -Label "E2E clipboard helper"
    Assert-PowerShellSyntax -RelativePath $E2eEvidenceHelperRel -Label "E2E evidence helper"

    Invoke-CommandChecked "Node frontend syntax" "node" @("--check", $NodeJsRel)
    Invoke-CommandChecked "Bridge frontend syntax" "node" @("--check", $BridgeJsRel)
    Invoke-CommandChecked "True raw log frontend syntax" "node" @("--check", $FrontendTestRel)
    Invoke-CommandChecked "True raw log frontend tests" "node" @($FrontendTestRel)

    Invoke-CargoChecked "Rust typed raw log tests" @("test", "-p", "kaspa-gateway-desktop", "typed_raw_log", "--test", "integrated_runtime_ipc_smoke_tests", "--", "--nocapture")
    Invoke-CargoChecked "Rust child raw log fixture test" @("test", "-p", "kaspa-gateway-desktop", "child_stdout_and_stderr_fixtures_survive_unchanged", "--test", "integrated_runtime_ipc_smoke_tests", "--", "--nocapture")
    Invoke-CargoChecked "Rust network role and bridge instance isolation test" @("test", "-p", "kaspa-gateway-desktop", "raw_log_buffers_are_isolated_by_network_role_and_bridge_instance", "--test", "integrated_runtime_ipc_smoke_tests", "--", "--nocapture")
    Invoke-CargoChecked "Desktop debug build" @("build", "--locked", "--bin", "kaspa-gateway-desktop")

    if ($Failures.Count -gt 0) {
        Write-Host "KGW true raw log gate FAILED" -ForegroundColor Red
        foreach ($failure in $Failures) {
            Write-Host "- $failure" -ForegroundColor Red
        }
        exit 1
    }

    Write-Host "KGW true raw log gate PASSED"
    if ($script:FreshCargoTargetDirUsed) {
        Write-Host "Used isolated CARGO_TARGET_DIR=$script:CargoTargetDir"
    }
}
finally {
    Pop-Location
}
