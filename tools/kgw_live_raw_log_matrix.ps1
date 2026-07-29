param(
    [Parameter(Mandatory)]
    [string]$Repository
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false

. (Join-Path $PSScriptRoot "kgw_raw_log_clipboard_capture.ps1")

$Repository = (Resolve-Path -LiteralPath $Repository).Path
$runId = Get-Date -Format "yyyyMMdd-HHmmss"
$runDirectory = Join-Path $Repository "artifacts\live-raw-log-matrix\$runId"
$stdoutLog = Join-Path $runDirectory "desktop.stdout.log"
$stderrLog = Join-Path $runDirectory "desktop.stderr.log"
$launcherLog = Join-Path $runDirectory "launcher.log"
$reportFile = Join-Path $runDirectory "report.json"
$buildLog = Join-Path $runDirectory "build.log"
$payloadDirectory = Join-Path $runDirectory "payloads"
$executable = Join-Path $Repository "target\debug\kaspa-gateway-desktop.exe"

New-Item -ItemType Directory -Path $runDirectory -Force | Out-Null
New-Item -ItemType Directory -Path $payloadDirectory -Force | Out-Null
New-Item -ItemType File -Path $stdoutLog -Force | Out-Null
New-Item -ItemType File -Path $stderrLog -Force | Out-Null
New-Item -ItemType File -Path $launcherLog -Force | Out-Null

$script:Report = [ordered]@{
    run_id = $runId
    repository = $Repository
    started_at = (Get-Date).ToString("O")
    finished_at = $null
    executable = $executable
    executable_sha256 = $null
    run_directory = $runDirectory
    logs = [ordered]@{
        stdout = $stdoutLog
        stderr = $stderrLog
        launcher = $launcherLog
        build = $buildLog
    }
    stages = @()
    final_clipboard = $null
    warnings = @()
}
$script:ChildEvidence = New-Object System.Collections.Generic.List[object]
$script:StdoutLineCount = 0
$script:StderrLineCount = 0

function Write-MatrixLog {
    param(
        [Parameter(Mandatory)][string]$Message,
        [ValidateSet("INFO", "WARN", "ERROR", "STDOUT", "STDERR")]
        [string]$Level = "INFO"
    )

    $timestamp = Get-Date -Format "yyyy-MM-ddTHH:mm:ss.fffK"
    $line = "[$timestamp][$Level] $Message"
    Add-Content -LiteralPath $launcherLog -Value $line -Encoding utf8
    Write-Host $line
}

function Save-MatrixReport {
    $script:Report |
        ConvertTo-Json -Depth 12 |
        Set-Content -LiteralPath $reportFile -Encoding utf8
}

function ConvertTo-ProcessArgument {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Argument)

    if ($Argument -notmatch '[\s"]') {
        return $Argument
    }

    return '"' + ($Argument -replace '\\', '\\' -replace '"', '\"') + '"'
}

function Test-LibRocksDbArtifactInconsistency {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Text)

    return $Text -match "librocksdb-sys" -and
        $Text -match "(inconsistent|missing|not found|could not find|failed to read|no such file|access is denied|LNK1104|corrupt)"
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

function Get-NewestDesktopBuildInput {
    $roots = @(
        (Join-Path $Repository "crates"),
        (Join-Path $Repository "apps\kaspa-gateway-desktop\src-tauri"),
        (Join-Path $Repository "Cargo.toml"),
        (Join-Path $Repository "Cargo.lock")
    )

    $items = New-Object System.Collections.Generic.List[object]
    foreach ($root in $roots) {
        if (-not (Test-Path -LiteralPath $root)) {
            continue
        }

        $item = Get-Item -LiteralPath $root
        if ($item.PSIsContainer) {
            Get-ChildItem -LiteralPath $root -Recurse -File -Include *.rs,*.toml,*.lock,*.json |
                ForEach-Object { [void]$items.Add($_) }
        } else {
            [void]$items.Add($item)
        }
    }

    return @($items | Sort-Object LastWriteTimeUtc -Descending | Select-Object -First 1)[0]
}

function Test-DesktopBuildNeeded {
    if (-not (Test-Path -LiteralPath $executable)) {
        return $true
    }

    $newestInput = Get-NewestDesktopBuildInput
    if ($null -eq $newestInput) {
        return $false
    }

    $exeItem = Get-Item -LiteralPath $executable
    return $exeItem.LastWriteTimeUtc -lt $newestInput.LastWriteTimeUtc
}

function Invoke-DesktopBuildIfNeeded {
    if (-not (Test-DesktopBuildNeeded)) {
        Write-MatrixLog "Debug desktop executable is current enough for this matrix run."
        return
    }

    Write-MatrixLog "Building debug desktop executable."
    $result = Invoke-CargoProcess -Arguments @("build", "--locked", "--bin", "kaspa-gateway-desktop") -CargoTargetDir $null
    $result.Text | Set-Content -LiteralPath $buildLog -Encoding utf8
    if ($result.ExitCode -eq 0) {
        return
    }

    if (-not (Test-LibRocksDbArtifactInconsistency -Text $result.Text)) {
        throw "Desktop build failed with exit code $($result.ExitCode). Build log: $buildLog"
    }

    $targetDir = Join-Path $Repository "artifacts\cargo-targets\live-raw-log-matrix-$runId"
    New-Item -ItemType Directory -Path $targetDir -Force | Out-Null
    $script:Report.warnings += "Retried desktop build with isolated CARGO_TARGET_DIR=$targetDir because librocksdb-sys artifacts looked inconsistent."
    Write-MatrixLog "librocksdb-sys artifacts look inconsistent; retrying with isolated CARGO_TARGET_DIR=$targetDir" -Level "WARN"
    $retry = Invoke-CargoProcess -Arguments @("build", "--locked", "--bin", "kaspa-gateway-desktop") -CargoTargetDir $targetDir
    Add-Content -LiteralPath $buildLog -Value $retry.Text -Encoding utf8
    if ($retry.ExitCode -ne 0) {
        throw "Desktop build failed with exit code $($retry.ExitCode) after isolated CARGO_TARGET_DIR retry. Build log: $buildLog"
    }
}

function Stop-RepositoryGatewayProcesses {
    $processes = @(
        Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |
            Where-Object {
                $_.ExecutablePath -and
                $_.ExecutablePath.StartsWith($Repository, [System.StringComparison]::OrdinalIgnoreCase) -and
                (
                    $_.Name -eq "kaspa-gateway-desktop.exe" -or
                    ($_.CommandLine -and $_.CommandLine -match "kaspa-gateway")
                )
            }
    )

    foreach ($process in $processes) {
        Write-MatrixLog "Stopping repository-owned Kaspa Gateway process PID=$($process.ProcessId) Name=$($process.Name)" -Level "WARN"
        Stop-Process -Id $process.ProcessId -Force -ErrorAction SilentlyContinue
    }
}

function ConvertFrom-KgwChildEvidenceLine {
    param([Parameter(Mandatory)][string]$Line)

    $prefix = $null
    if ($Line.StartsWith("[KGW_CHILD_STDOUT] ", [System.StringComparison]::Ordinal)) {
        $prefix = "[KGW_CHILD_STDOUT] "
    } elseif ($Line.StartsWith("[KGW_CHILD_STDERR] ", [System.StringComparison]::Ordinal)) {
        $prefix = "[KGW_CHILD_STDERR] "
    } else {
        return $null
    }

    try {
        $json = $Line.Substring($prefix.Length) | ConvertFrom-Json -ErrorAction Stop
        return [pscustomobject]@{
            timestamp = (Get-Date).ToString("O")
            prefix = $prefix.Trim()
            network = [string]$json.network
            runtime_role = [string]$json.runtimeRole
            bridge_instance_id = if ($null -eq $json.bridgeInstanceId) { $null } else { [string]$json.bridgeInstanceId }
            stream = [string]$json.stream
            pid = $json.pid
            line = $Line
        }
    }
    catch {
        return [pscustomobject]@{
            timestamp = (Get-Date).ToString("O")
            prefix = $prefix.Trim()
            network = "unknown"
            runtime_role = "unknown"
            bridge_instance_id = $null
            stream = "unknown"
            pid = $null
            line = $Line
        }
    }
}

function Get-MatchingChildEvidence {
    param(
        [Parameter(Mandatory)][string]$Network,
        [Parameter(Mandatory)][string]$RuntimeRole,
        [AllowNull()][string]$BridgeInstanceId
    )

    return @(
        $script:ChildEvidence |
            Where-Object {
                $_.network -eq $Network -and
                $_.runtime_role -eq $RuntimeRole -and
                (
                    [string]::IsNullOrWhiteSpace($BridgeInstanceId) -or
                    [string]::IsNullOrWhiteSpace($_.bridge_instance_id) -or
                    $_.bridge_instance_id -eq $BridgeInstanceId
                )
            }
    )
}

function Read-NewLogLines {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][ref]$LineCount,
        [Parameter(Mandatory)][string]$Label
    )

    if (-not (Test-Path -LiteralPath $Path)) {
        return @()
    }

    $lines = @(Get-Content -LiteralPath $Path -ErrorAction SilentlyContinue)
    if ($lines.Count -le $LineCount.Value) {
        return @()
    }

    $newLines = New-Object System.Collections.Generic.List[string]
    for ($index = $LineCount.Value; $index -lt $lines.Count; $index++) {
        $line = [string]$lines[$index]
        $newLines.Add($line)

        if ($line.StartsWith("[KGW_CHILD_STD", [System.StringComparison]::Ordinal)) {
            $evidence = ConvertFrom-KgwChildEvidenceLine -Line $line
            if ($null -ne $evidence) {
                $script:ChildEvidence.Add($evidence)
            }
            Write-MatrixLog "[$Label] $line" -Level $Label
        } elseif ($line.StartsWith("[KGW_START_TRACE]", [System.StringComparison]::Ordinal) -and $line -match "copy_log_|clipboard_write_") {
            Write-MatrixLog "[$Label] $line" -Level $Label
        }
    }

    $LineCount.Value = $lines.Count
    return @($newLines)
}

function Test-StageCaptureMatches {
    param(
        [Parameter(Mandatory)]$Stage,
        [Parameter(Mandatory)]$Capture
    )

    if (-not $Capture.valid) {
        return $false
    }

    if ([string]$Capture.network -ne [string]$Stage.Network) {
        return $false
    }

    if ([string]$Capture.runtime_role -ne [string]$Stage.RuntimeRole) {
        return $false
    }

    if ($Stage.RuntimeRole -eq "bridge" -and [string]::IsNullOrWhiteSpace([string]$Capture.bridge_instance_id)) {
        return $false
    }

    return $true
}

function Invoke-StageValidation {
    param(
        [Parameter(Mandatory)]$Stage,
        [Parameter(Mandatory)]$Capture
    )

    $errors = New-Object System.Collections.Generic.List[string]
    $warnings = New-Object System.Collections.Generic.List[string]

    if (-not $Capture.valid) {
        foreach ($error in $Capture.errors) {
            $errors.Add($error)
        }
    }
    if ([string]$Capture.network -ne [string]$Stage.Network) {
        $errors.Add("Capture network '$($Capture.network)' did not match expected '$($Stage.Network)'.")
    }
    if ([string]$Capture.runtime_role -ne [string]$Stage.RuntimeRole) {
        $errors.Add("Capture runtime role '$($Capture.runtime_role)' did not match expected '$($Stage.RuntimeRole)'.")
    }
    if ($Stage.RuntimeRole -eq "bridge" -and [string]::IsNullOrWhiteSpace([string]$Capture.bridge_instance_id)) {
        $errors.Add("Bridge capture did not include bridge instance identity.")
    }

    $payloadText = ""
    if ($Capture.payload_file -and (Test-Path -LiteralPath $Capture.payload_file)) {
        $payloadText = [System.IO.File]::ReadAllText($Capture.payload_file, [System.Text.Encoding]::UTF8)
    } else {
        $errors.Add("Valid raw payload file was not saved.")
    }

    $acceptance = Test-KgwRawLogPayloadAcceptanceV1 `
        -Text $payloadText `
        -Network $Stage.Network `
        -RuntimeRole $Stage.RuntimeRole `
        -BridgeInstanceId $Capture.bridge_instance_id

    foreach ($error in $acceptance.errors) {
        $errors.Add($error)
    }
    foreach ($warning in $acceptance.warnings) {
        $warnings.Add($warning)
    }

    $childEvidence = Get-MatchingChildEvidence `
        -Network $Stage.Network `
        -RuntimeRole $Stage.RuntimeRole `
        -BridgeInstanceId $Capture.bridge_instance_id

    if ($Stage.RuntimeRole -eq "bridge" -and $childEvidence.Count -eq 0) {
        $errors.Add("No matching bridge child stdout/stderr diagnostic evidence was observed before Copy Log.")
    }

    $stagePayloadFile = Join-Path $runDirectory ("stage-{0:D2}-{1}.raw.txt" -f $Stage.Index, $Stage.Slug)
    if ($Capture.payload_file -and (Test-Path -LiteralPath $Capture.payload_file)) {
        Copy-Item -LiteralPath $Capture.payload_file -Destination $stagePayloadFile -Force
    } else {
        $stagePayloadFile = $null
    }

    return [pscustomobject]@{
        stage = $Stage.Name
        network = $Stage.Network
        runtime_role = $Stage.RuntimeRole
        bridge_instance_id = $Capture.bridge_instance_id
        passed = ($errors.Count -eq 0)
        errors = @($errors)
        warnings = @($warnings)
        capture = $Capture
        stage_payload_file = $stagePayloadFile
        child_evidence_count = $childEvidence.Count
        child_evidence = @($childEvidence | Select-Object -First 12)
    }
}

function Wait-ForStageCapture {
    param(
        [Parameter(Mandatory)]$Stage,
        [Parameter(Mandatory)]$DesktopProcess,
        [Parameter(Mandatory)][int]$TimeoutSeconds
    )

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    $captures = New-Object System.Collections.Generic.List[object]

    while ((Get-Date) -lt $deadline) {
        $DesktopProcess.Refresh()
        if ($DesktopProcess.HasExited) {
            throw "Desktop process exited before $($Stage.Name) Copy Log was captured. ExitCode=$($DesktopProcess.ExitCode)"
        }

        $newLines = @()
        $newLines += Read-NewLogLines -Path $stdoutLog -LineCount ([ref]$script:StdoutLineCount) -Label "STDOUT"
        $newLines += Read-NewLogLines -Path $stderrLog -LineCount ([ref]$script:StderrLineCount) -Label "STDERR"

        foreach ($line in $newLines) {
            if (-not ($line.StartsWith("[KGW_START_TRACE]", [System.StringComparison]::Ordinal))) {
                continue
            }
            if ($line -notmatch "native\.clipboard_write_succeeded|frontend\.copy_log_succeeded") {
                continue
            }

            $capture = New-KgwRawLogClipboardCaptureFromClipboardV1 `
                -TraceLine $line `
                -OutputDirectory $payloadDirectory `
                -Reason ("matrix-{0}" -f $Stage.Slug)
            $captures.Add($capture)

            Write-MatrixLog "Captured event-time clipboard stage=$($capture.event_stage) network=$($capture.network) runtime_role=$($capture.runtime_role) bridge_instance=$($capture.bridge_instance_id) expected_sha256=$($capture.expected_sha256) actual_sha256=$($capture.actual_sha256) sha256_match=$($capture.sha256_match) payload=$($capture.payload_file)"

            if (Test-StageCaptureMatches -Stage $Stage -Capture $capture) {
                return $capture
            }
        }

        Start-Sleep -Milliseconds 250
    }

    $captureSummary = @($captures | ForEach-Object { "$($_.event_stage)/$($_.network)/$($_.runtime_role)/$($_.bridge_instance_id)/valid=$($_.valid)" }) -join "; "
    throw "Timed out waiting for $($Stage.Name) Copy Log event. Captures seen: $captureSummary"
}

function Set-ClipboardSentinel {
    param([Parameter(Mandatory)][string]$Sentinel)

    Set-Clipboard -Value $Sentinel
    Write-MatrixLog "Clipboard sentinel set. sentinel_sha256=$(Get-KgwRawLogSha256V1 -Text $Sentinel)"
}

function Get-FinalClipboardInfo {
    try {
        $text = Get-Clipboard -Raw -ErrorAction Stop
        if ($null -eq $text) {
            $text = ""
        }
        return [ordered]@{
            timestamp = (Get-Date).ToString("O")
            informational_only = $true
            line_count = Get-KgwRawLogLineCountV1 -Text ([string]$text)
            character_count = Get-KgwRawLogCharacterCountV1 -Text ([string]$text)
            sha256 = Get-KgwRawLogSha256V1 -Text ([string]$text)
            transport_wrapper_detected = Test-KgwRawLogTransportWrapperTextV1 -Text ([string]$text)
        }
    }
    catch {
        return [ordered]@{
            timestamp = (Get-Date).ToString("O")
            informational_only = $true
            error = Get-KgwRawLogSafeDiagnosticTextV1 -Value $_.Exception.Message
        }
    }
}

Push-Location $Repository
$desktopProcess = $null
try {
    Write-MatrixLog "Repository=$Repository"
    Write-MatrixLog "RunDirectory=$runDirectory"
    Stop-RepositoryGatewayProcesses
    Start-Sleep -Seconds 1

    Invoke-DesktopBuildIfNeeded
    if (-not (Test-Path -LiteralPath $executable)) {
        throw "Debug desktop executable was not found after build: $executable"
    }

    $script:Report.executable_sha256 = (Get-FileHash -LiteralPath $executable -Algorithm SHA256).Hash
    Save-MatrixReport

    $env:RUST_BACKTRACE = "full"
    $env:RUST_LOG = "kaspa_gateway_desktop=trace,kaspa_gateway_runtime=trace,kaspa_gateway_node=trace,info"
    $env:KGW_LOG_LEVEL = "trace"
    $env:KGW_START_TRACE = "1"
    $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--enable-logging=stderr --v=1"

    Write-MatrixLog "Launching desktop with KGW_START_TRACE=1."
    $desktopProcess = Start-Process `
        -FilePath $executable `
        -WorkingDirectory $Repository `
        -RedirectStandardOutput $stdoutLog `
        -RedirectStandardError $stderrLog `
        -PassThru
    Write-MatrixLog "DesktopPID=$($desktopProcess.Id)"

    $stages = @(
        [pscustomobject]@{ Index = 1; Name = "Testnet10 Node"; Slug = "testnet10-node"; Network = "testnet10"; RuntimeRole = "node" },
        [pscustomobject]@{ Index = 2; Name = "Mainnet Bridge"; Slug = "mainnet-bridge"; Network = "mainnet"; RuntimeRole = "bridge" },
        [pscustomobject]@{ Index = 3; Name = "Testnet10 Bridge"; Slug = "testnet10-bridge"; Network = "testnet10"; RuntimeRole = "bridge" }
    )

    foreach ($stage in $stages) {
        Write-Host ""
        Write-Host "PHYSICAL STAGE: $($stage.Name)" -ForegroundColor Cyan
        Write-Host "1. Use the desktop app to select $($stage.Name)."
        Write-Host "2. Start the runtime from Settings only, then open its Live Monitor."
        Write-Host "3. Wait until native child stdout/stderr is visible."
        Write-Host "4. Click Copy Log while this script is watching."
        Write-Host "The script will capture the clipboard immediately on the success event and validate before advancing."

        $sentinel = "KGW_LIVE_RAW_LOG_MATRIX_SENTINEL|run=$runId|stage=$($stage.Index)|$([System.Guid]::NewGuid().ToString("N"))"
        Set-ClipboardSentinel -Sentinel $sentinel
        [void](Read-Host "Press Enter to start watching for $($stage.Name), then perform the steps above")

        $capture = Wait-ForStageCapture -Stage $stage -DesktopProcess $desktopProcess -TimeoutSeconds 600
        $result = Invoke-StageValidation -Stage $stage -Capture $capture
        $script:Report.stages += $result
        Save-MatrixReport

        if (-not $result.passed) {
            foreach ($error in $result.errors) {
                Write-MatrixLog "$($stage.Name) failed: $error" -Level "ERROR"
            }
            throw "$($stage.Name) validation failed. Report: $reportFile"
        }

        foreach ($warning in $result.warnings) {
            Write-MatrixLog "$($stage.Name) warning: $warning" -Level "WARN"
        }
        Write-MatrixLog "$($stage.Name) PASSED payload=$($result.stage_payload_file)"
    }

    $script:Report.final_clipboard = Get-FinalClipboardInfo
    $script:Report.finished_at = (Get-Date).ToString("O")
    Save-MatrixReport
    Write-MatrixLog "Live raw log matrix PASSED. Report=$reportFile"
}
catch {
    $script:Report.final_clipboard = Get-FinalClipboardInfo
    $script:Report.finished_at = (Get-Date).ToString("O")
    $script:Report.warnings += "Matrix failed: $(Get-KgwRawLogSafeDiagnosticTextV1 -Value $_.Exception.Message)"
    Save-MatrixReport
    Write-MatrixLog "Live raw log matrix FAILED: $($_.Exception.Message)" -Level "ERROR"
    throw
}
finally {
    if ($desktopProcess -and -not $desktopProcess.HasExited) {
        Write-MatrixLog "Stopping desktop process launched by this matrix. PID=$($desktopProcess.Id)"
        Stop-Process -Id $desktopProcess.Id -Force -ErrorAction SilentlyContinue
    }
    Pop-Location
}
