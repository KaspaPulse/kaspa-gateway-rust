param(
    [string]$Repository = "D:\Projects\kaspa-gateway-rust"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false

$runId = Get-Date -Format "yyyyMMdd-HHmmss"
$diagnosticRoot = Join-Path $Repository "artifacts\desktop-diagnostics"
$diagnosticDirectory = Join-Path $diagnosticRoot $runId
$stdoutLog = Join-Path $diagnosticDirectory "desktop.stdout.log"
$stderrLog = Join-Path $diagnosticDirectory "desktop.stderr.log"
$launcherLog = Join-Path $diagnosticDirectory "launcher.log"
$processLog = Join-Path $diagnosticDirectory "processes.log"
$portLog = Join-Path $diagnosticDirectory "ports.log"
$childProcessLog = Join-Path $diagnosticDirectory "child-process.log"
$clipboardLog = Join-Path $diagnosticDirectory "clipboard.log"
$clipboardPayloadDirectory = Join-Path $diagnosticDirectory "clipboard-payloads"
$windowsEventLog = Join-Path $diagnosticDirectory "windows-events.log"
$summaryFile = Join-Path $diagnosticDirectory "summary.json"
$zipFile = "$diagnosticDirectory.zip"
$executable = Join-Path $Repository "target\debug\kaspa-gateway-desktop.exe"
$buildLog = Join-Path $diagnosticDirectory "build.log"
$expectedNodePorts = @(16110, 16210, 16310)

New-Item -ItemType Directory -Path $diagnosticDirectory -Force | Out-Null
New-Item -ItemType File -Path $stdoutLog -Force | Out-Null
New-Item -ItemType File -Path $stderrLog -Force | Out-Null
New-Item -ItemType File -Path $launcherLog -Force | Out-Null
New-Item -ItemType File -Path $processLog -Force | Out-Null
New-Item -ItemType File -Path $portLog -Force | Out-Null
New-Item -ItemType File -Path $childProcessLog -Force | Out-Null
New-Item -ItemType File -Path $clipboardLog -Force | Out-Null
New-Item -ItemType Directory -Path $clipboardPayloadDirectory -Force | Out-Null

. (Join-Path $PSScriptRoot "kgw_raw_log_clipboard_capture.ps1")

$script:InitialClipboardFingerprint = $null
$script:LatestClipboardFingerprint = $null
$script:ClipboardEventCaptures = New-Object System.Collections.Generic.List[object]

function Write-Diagnostic {
    param(
        [Parameter(Mandatory)]
        [string]$Message,

        [ValidateSet("INFO", "WARN", "ERROR", "STDOUT", "STDERR", "PROCESS", "PORT")]
        [string]$Level = "INFO"
    )

    $timestamp = Get-Date -Format "yyyy-MM-ddTHH:mm:ss.fffK"
    $line = "[$timestamp][$Level] $Message"

    Add-Content -LiteralPath $launcherLog -Value $line -Encoding utf8
    Write-Host $line
}

function Test-KgwRuntimePollingTrace {
    param([Parameter(Mandatory)][string]$Line)

    if (-not $Line.StartsWith("[KGW_START_TRACE]", [System.StringComparison]::Ordinal)) {
        return $false
    }

    return $Line -match "kgw_runtime_owner_status_v1|kgw_kgw_runtime_logs_v1"
}

function Write-ChildEvidence {
    param([Parameter(Mandatory)][string]$Line)

    Add-Content -LiteralPath $childProcessLog -Value $Line -Encoding utf8
}

function Get-SafeDiagnosticText {
    param([AllowNull()][string]$Value)

    $text = if ($null -eq $Value) { "" } else { [string]$Value }
    $clean = ($text -replace "[`r`n`t]", " ").Trim()

    if ($clean -match "secret|token|private|mnemonic|wallet|address") {
        return "redacted-sensitive-value"
    }

    if ($clean.Length -gt 220) {
        return $clean.Substring(0, 220)
    }

    return $clean
}

function Get-ClipboardFingerprint {
    param([Parameter(Mandatory)][string]$Reason)

    try {
        $text = Get-Clipboard -Raw -ErrorAction Stop
        if ($null -eq $text) {
            $text = ""
        }

        $bytes = [System.Text.Encoding]::UTF8.GetBytes([string]$text)
        $sha = [System.Security.Cryptography.SHA256]::Create()
        try {
            $hashBytes = $sha.ComputeHash($bytes)
        }
        finally {
            $sha.Dispose()
        }

        $hash = -join ($hashBytes | ForEach-Object { $_.ToString("x2") })
        $lineCount = if ([string]::IsNullOrEmpty($text)) { 0 } else { ([regex]::Split([string]$text, "`r`n|`n|`r")).Count }

        return [ordered]@{
            timestamp = (Get-Date).ToString("O")
            reason = $Reason
            available = $true
            character_count = ([string]$text).Length
            line_count = $lineCount
            sha256 = $hash
        }
    }
    catch {
        return [ordered]@{
            timestamp = (Get-Date).ToString("O")
            reason = $Reason
            available = $false
            character_count = $null
            line_count = $null
            sha256 = $null
            error = Get-SafeDiagnosticText -Value $_.Exception.Message
        }
    }
}

function Write-ClipboardSnapshot {
    param([Parameter(Mandatory)][string]$Reason)

    $fingerprint = Get-ClipboardFingerprint -Reason $Reason
    $script:LatestClipboardFingerprint = $fingerprint
    if ($Reason -eq "before-launch") {
        $script:InitialClipboardFingerprint = $fingerprint
    }

    $json = $fingerprint | ConvertTo-Json -Depth 4 -Compress
    Add-Content -LiteralPath $clipboardLog -Value $json -Encoding utf8

    $changed = $false
    if ($script:InitialClipboardFingerprint -and $fingerprint.available -and $script:InitialClipboardFingerprint.available) {
        $changed = [string]$script:InitialClipboardFingerprint.sha256 -ne [string]$fingerprint.sha256
    }

    Write-Diagnostic -Message "ClipboardSnapshot reason=$Reason available=$($fingerprint.available) characters=$($fingerprint.character_count) lines=$($fingerprint.line_count) sha256=$($fingerprint.sha256) changed_from_launch=$changed"
}

function Write-ClipboardEventCapture {
    param(
        [Parameter(Mandatory)][string]$Reason,
        [Parameter(Mandatory)][string]$TraceLine
    )

    $capture = New-KgwRawLogClipboardCaptureFromClipboardV1 `
        -TraceLine $TraceLine `
        -OutputDirectory $clipboardPayloadDirectory `
        -Reason $Reason

    $script:LatestClipboardFingerprint = [ordered]@{
        timestamp = $capture.capture_timestamp
        reason = $Reason
        available = ($null -ne $capture.actual_sha256)
        character_count = $capture.character_count
        line_count = $capture.line_count
        sha256 = $capture.actual_sha256
        event_stage = $capture.event_stage
        payload_file = $capture.payload_file
    }

    [void]$script:ClipboardEventCaptures.Add($capture)
    Add-Content -LiteralPath $clipboardLog -Value (($capture | ConvertTo-Json -Depth 6 -Compress)) -Encoding utf8

    $message = "ClipboardEventCapture reason=$Reason stage=$($capture.event_stage) network=$($capture.network) runtime_role=$($capture.runtime_role) bridge_instance=$($capture.bridge_instance_id) characters=$($capture.character_count) lines=$($capture.line_count) expected_sha256=$($capture.expected_sha256) actual_sha256=$($capture.actual_sha256) sha256_match=$($capture.sha256_match) payload=$($capture.payload_file)"
    if ($capture.valid) {
        Write-Diagnostic -Message $message
    } else {
        Write-Diagnostic -Level "ERROR" -Message "$message errors=$($capture.errors -join ' | ')"
    }
}

function Test-KgwClipboardTrace {
    param([Parameter(Mandatory)][string]$Line)

    return $Line -match "copy_log_|clipboard_write_"
}

function Show-NewLogLines {
    param(
        [Parameter(Mandatory)]
        [string]$Path,

        [Parameter(Mandatory)]
        [ref]$LineCount,

        [Parameter(Mandatory)]
        [string]$Label
    )

    if (-not (Test-Path $Path)) {
        return
    }

    $lines = @(Get-Content -LiteralPath $Path -ErrorAction SilentlyContinue)

    if ($lines.Count -le $LineCount.Value) {
        return
    }

    for ($index = $LineCount.Value; $index -lt $lines.Count; $index++) {
        $line = [string]$lines[$index]

        if ($line.StartsWith("[KGW_CHILD_STDERR]", [System.StringComparison]::Ordinal)) {
            Write-ChildEvidence -Line $line
            Write-Host "[$Label] $line" -ForegroundColor Yellow
        }
        elseif ($line.StartsWith("[KGW_CHILD_STDOUT]", [System.StringComparison]::Ordinal)) {
            Write-ChildEvidence -Line $line
            Write-Host "[$Label] $line" -ForegroundColor DarkCyan
        }
        elseif ($line.StartsWith("[KGW_START_TRACE]", [System.StringComparison]::Ordinal)) {
            if ($line -match "native\.child_pid_recorded|self-worker-exited-during-startup") {
                Write-ChildEvidence -Line $line
            }

            if (Test-KgwClipboardTrace -Line $line) {
                Add-Content -LiteralPath $clipboardLog -Value $line -Encoding utf8

                if ($line -match "copy_log_succeeded|clipboard_write_succeeded") {
                    Write-Host "[$Label] $line" -ForegroundColor Green
                    Write-ClipboardEventCapture -Reason "clipboard-success-trace" -TraceLine $line
                    continue
                }

                if ($line -match "copy_log_failed|clipboard_write_failed") {
                    Write-Host "[$Label] $line" -ForegroundColor Yellow
                    Write-ClipboardSnapshot -Reason "after-clipboard-failure-trace"
                    continue
                }

                Write-Host "[$Label] $line" -ForegroundColor Magenta
                if ($line -match "copy_log_dispatched|clipboard_write_entered") {
                    Write-ClipboardSnapshot -Reason "after-clipboard-dispatch-trace"
                }
                continue
            }

            if (Test-KgwRuntimePollingTrace -Line $line) {
                continue
            }

            Write-Host "[$Label] $line" -ForegroundColor Cyan
        }
        elseif ($Label -eq "STDERR") {
            Write-Host "[$Label] $line" -ForegroundColor Red
        }
        else {
            Write-Host "[$Label] $line" -ForegroundColor Gray
        }
    }

    $LineCount.Value = $lines.Count
}

function Get-CommandLineSummary {
    param([AllowNull()][string]$CommandLine)

    if ([string]::IsNullOrWhiteSpace($CommandLine)) {
        return "unavailable"
    }

    $hasSelfWorker = $CommandLine -match "--kgw-self-worker"
    $hasWebView = $CommandLine -match "embedded-browser-webview|msedgewebview2"
    $hasTauriDesktop = $CommandLine -match "kaspa-gateway-desktop.exe"

    return "length=$($CommandLine.Length);desktop=$hasTauriDesktop;self_worker=$hasSelfWorker;webview=$hasWebView"
}

function Get-GatewayProcesses {
    $allProcesses = @(Get-CimInstance Win32_Process -ErrorAction SilentlyContinue)

    return @(
        $allProcesses |
            Where-Object {
                ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($Repository, [System.StringComparison]::OrdinalIgnoreCase)) -or
                ($_.CommandLine -and $_.CommandLine -match "kaspa-gateway")
            } |
            ForEach-Object {
                [pscustomobject]@{
                    ProcessId = $_.ProcessId
                    ParentProcessId = $_.ParentProcessId
                    Name = $_.Name
                    ExecutablePath = $_.ExecutablePath
                    CommandLineSummary = Get-CommandLineSummary -CommandLine $_.CommandLine
                }
            }
    )
}

function Write-ProcessSnapshot {
    param(
        [ref]$PreviousSnapshot
    )

    $processes = @(Get-GatewayProcesses)
    $snapshot = [string]($processes | ConvertTo-Json -Depth 4 -Compress)

    if ($snapshot -eq $PreviousSnapshot.Value) {
        return
    }

    $timestamp = Get-Date -Format "yyyy-MM-ddTHH:mm:ss.fffK"
    Add-Content -LiteralPath $processLog -Value "[$timestamp]" -Encoding utf8

    if ($processes.Count -eq 0) {
        Add-Content -LiteralPath $processLog -Value "No Kaspa Gateway processes detected." -Encoding utf8
        Write-Diagnostic -Level "PROCESS" -Message "No Kaspa Gateway processes detected."
    }
    else {
        $formatted = $processes | Format-Table -AutoSize | Out-String
        Add-Content -LiteralPath $processLog -Value $formatted -Encoding utf8

        foreach ($item in $processes) {
            Write-Diagnostic -Level "PROCESS" -Message "PID=$($item.ProcessId) ParentPID=$($item.ParentProcessId) Name=$($item.Name) CommandLineSummary=$($item.CommandLineSummary)"
        }
    }

    $PreviousSnapshot.Value = $snapshot
}

function Write-PortSnapshot {
    param(
        [ref]$PreviousSnapshot
    )

    try {
        $connections = @(
            Get-NetTCPConnection -ErrorAction Stop |
                Where-Object {
                    $expectedNodePorts -contains [int]$_.LocalPort
                } |
                Sort-Object OwningProcess, LocalPort, RemotePort |
                Select-Object State, LocalAddress, LocalPort, RemoteAddress, RemotePort, OwningProcess
        )

        $snapshot = [string]($connections | ConvertTo-Json -Depth 4 -Compress)

        if ($snapshot -eq $PreviousSnapshot.Value) {
            return
        }

        $timestamp = Get-Date -Format "yyyy-MM-ddTHH:mm:ss.fffK"
        Add-Content -LiteralPath $portLog -Value "[$timestamp]" -Encoding utf8

        if ($connections.Count -eq 0) {
            Add-Content -LiteralPath $portLog -Value "No matching TCP connections detected." -Encoding utf8
            Write-Diagnostic -Level "PORT" -Message "No matching TCP connections detected."
        }
        else {
            $formatted = $connections | Format-Table -AutoSize | Out-String
            Add-Content -LiteralPath $portLog -Value $formatted -Encoding utf8

            foreach ($connection in $connections) {
                Write-Diagnostic -Level "PORT" -Message "PID=$($connection.OwningProcess) State=$($connection.State) Local=$($connection.LocalAddress):$($connection.LocalPort) Remote=$($connection.RemoteAddress):$($connection.RemotePort)"
            }
        }

        $PreviousSnapshot.Value = $snapshot
    }
    catch {
        Write-Diagnostic -Level "WARN" -Message "TCP snapshot failed: $($_.Exception.Message)"
    }
}

function Get-ChildProcessEvidenceSummary {
    if (-not (Test-Path -LiteralPath $childProcessLog)) {
        return [ordered]@{
            line_count = 0
            pids = @()
            exit_markers = @()
        }
    }

    $lines = @(Get-Content -LiteralPath $childProcessLog -ErrorAction SilentlyContinue)
    $pids = New-Object System.Collections.Generic.HashSet[string]
    $exitMarkers = New-Object System.Collections.Generic.List[string]

    foreach ($line in $lines) {
        foreach ($match in [regex]::Matches($line, '\\?"pid\\?"\s*:\s*(?<pid>[0-9]+)')) {
            if ($match.Groups["pid"].Success) {
                [void]$pids.Add($match.Groups["pid"].Value)
            }
        }

        if ($line -match "self-worker-exited-during-startup") {
            $exitMarkers.Add($line)
        }
    }

    $pidValues = @()
    foreach ($pidValue in $pids) {
        $pidValues += $pidValue
    }

    return [ordered]@{
        line_count = $lines.Count
        pids = $pidValues
        exit_markers = @($exitMarkers)
    }
}

function Get-ClipboardEvidenceSummary {
    if (-not (Test-Path -LiteralPath $clipboardLog)) {
        return [ordered]@{
            log = $clipboardLog
            line_count = 0
            initial = $script:InitialClipboardFingerprint
            latest = $script:LatestClipboardFingerprint
            event_captures = @($script:ClipboardEventCaptures)
            successful_event_capture_count = @($script:ClipboardEventCaptures | Where-Object { $_.valid }).Count
            failed_event_capture_count = @($script:ClipboardEventCaptures | Where-Object { -not $_.valid }).Count
            changed_from_launch = $false
        }
    }

    $lineCount = @(Get-Content -LiteralPath $clipboardLog -ErrorAction SilentlyContinue).Count
    $changed = $false
    if ($script:InitialClipboardFingerprint -and $script:LatestClipboardFingerprint -and $script:InitialClipboardFingerprint.available -and $script:LatestClipboardFingerprint.available) {
        $changed = [string]$script:InitialClipboardFingerprint.sha256 -ne [string]$script:LatestClipboardFingerprint.sha256
    }

    return [ordered]@{
        log = $clipboardLog
        line_count = $lineCount
        initial = $script:InitialClipboardFingerprint
        latest = $script:LatestClipboardFingerprint
        event_captures = @($script:ClipboardEventCaptures)
        successful_event_capture_count = @($script:ClipboardEventCaptures | Where-Object { $_.valid }).Count
        failed_event_capture_count = @($script:ClipboardEventCaptures | Where-Object { -not $_.valid }).Count
        changed_from_launch = $changed
    }
}

Set-Location $Repository

$branch = (git branch --show-current).Trim()
$commit = (git rev-parse HEAD).Trim()
$status = @(git status --short)

Write-Diagnostic -Message "Repository=$Repository"
Write-Diagnostic -Message "Branch=$branch"
Write-Diagnostic -Message "Commit=$commit"
Write-Diagnostic -Message "DiagnosticDirectory=$diagnosticDirectory"
Write-Diagnostic -Message "ExpectedNodePorts=$($expectedNodePorts -join ',')"
Write-Diagnostic -Message "ChildProcessLog=$childProcessLog"
Write-Diagnostic -Message "ClipboardLog=$clipboardLog"
Write-Diagnostic -Message "ClipboardPayloadDirectory=$clipboardPayloadDirectory"

if ($status.Count -eq 0) {
    Write-Diagnostic -Message "WorktreeStatus=clean"
}
else {
    Write-Diagnostic -Level "WARN" -Message "WorktreeStatus=dirty"

    foreach ($line in $status) {
        Write-Diagnostic -Level "WARN" -Message "GitStatus=$line"
    }
}

$existingProcesses = @(
    Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |
        Where-Object {
            $_.Name -eq "kaspa-gateway-desktop.exe" -and
            $_.ExecutablePath -and
            $_.ExecutablePath.StartsWith($Repository, [System.StringComparison]::OrdinalIgnoreCase)
        }
)

foreach ($existingProcess in $existingProcesses) {
    Write-Diagnostic -Level "WARN" -Message "Stopping existing repository-owned process PID=$($existingProcess.ProcessId)"
    Stop-Process -Id $existingProcess.ProcessId -Force -ErrorAction SilentlyContinue
}

Start-Sleep -Seconds 2

if (-not (Test-Path $executable)) {
    Write-Diagnostic -Message "Debug executable was not found. Starting local build."

    cargo build --locked --bin kaspa-gateway-desktop *>&1 |
        Tee-Object -FilePath $buildLog

    if ($LASTEXITCODE -ne 0) {
        Write-Diagnostic -Level "ERROR" -Message "Desktop build failed. BuildLog=$buildLog"
        throw "Desktop build failed."
    }
}

$executableHash = (Get-FileHash -LiteralPath $executable -Algorithm SHA256).Hash
$executableInfo = Get-Item -LiteralPath $executable

Write-Diagnostic -Message "Executable=$executable"
Write-Diagnostic -Message "ExecutableSHA256=$executableHash"
Write-Diagnostic -Message "ExecutableLastWriteTime=$($executableInfo.LastWriteTime.ToString('O'))"
Write-Diagnostic -Message "ExecutableLength=$($executableInfo.Length)"

$env:RUST_BACKTRACE = "full"
$env:RUST_LOG = "kaspa_gateway_desktop=trace,kaspa_gateway_runtime=trace,kaspa_gateway_node=trace,info"
$env:KGW_LOG_LEVEL = "trace"
$env:KGW_START_TRACE = "1"
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--enable-logging=stderr --v=1"
Write-Diagnostic -Message "KGW_START_TRACE=1"
Write-ClipboardSnapshot -Reason "before-launch"

$startedAt = Get-Date
$desktopProcess = $null
$exitCode = $null
$stdoutLineCount = 0
$stderrLineCount = 0
$previousProcessSnapshot = ""
$previousPortSnapshot = ""
$pollCounter = 0

try {
    Write-Diagnostic -Message "Launching the desktop application."

    $desktopProcess = Start-Process `
        -FilePath $executable `
        -WorkingDirectory $Repository `
        -RedirectStandardOutput $stdoutLog `
        -RedirectStandardError $stderrLog `
        -PassThru

    Write-Diagnostic -Message "DesktopPID=$($desktopProcess.Id)"
    Write-Diagnostic -Message "Use the application now. Reproduce the Start failure, inspect Live Node Monitor, then close the desktop application."
    Write-Diagnostic -Message "The launcher will continue monitoring processes, ports, stdout, and stderr."

    while (-not $desktopProcess.HasExited) {
        Show-NewLogLines -Path $stdoutLog -LineCount ([ref]$stdoutLineCount) -Label "STDOUT"
        Show-NewLogLines -Path $stderrLog -LineCount ([ref]$stderrLineCount) -Label "STDERR"
        Write-ProcessSnapshot -PreviousSnapshot ([ref]$previousProcessSnapshot)

        if (($pollCounter % 3) -eq 0) {
            Write-PortSnapshot -PreviousSnapshot ([ref]$previousPortSnapshot)
        }

        $pollCounter++
        Start-Sleep -Seconds 1
        $desktopProcess.Refresh()
    }

    $exitCode = $desktopProcess.ExitCode

    Show-NewLogLines -Path $stdoutLog -LineCount ([ref]$stdoutLineCount) -Label "STDOUT"
    Show-NewLogLines -Path $stderrLog -LineCount ([ref]$stderrLineCount) -Label "STDERR"
    Write-ProcessSnapshot -PreviousSnapshot ([ref]$previousProcessSnapshot)
    Write-PortSnapshot -PreviousSnapshot ([ref]$previousPortSnapshot)

    Write-Diagnostic -Message "Desktop process exited. ExitCode=$exitCode"
}
catch {
    Write-Diagnostic -Level "ERROR" -Message "Diagnostic launcher failure: $($_.Exception.ToString())"
    throw
}
finally {
    $eventStart = $startedAt.AddMinutes(-1)

    try {
        $events = @(
            Get-WinEvent `
                -FilterHashtable @{
                    LogName = "Application"
                    StartTime = $eventStart
                } `
                -ErrorAction Stop |
                Where-Object {
                    $_.Message -match "kaspa-gateway|WebView2|Application Error|Windows Error Reporting"
                } |
                Select-Object TimeCreated, ProviderName, Id, LevelDisplayName, Message
        )

        if ($events.Count -gt 0) {
            $events | Format-List | Out-File -LiteralPath $windowsEventLog -Encoding utf8
            Write-Diagnostic -Message "Captured $($events.Count) matching Windows application events."
        }
        else {
            "No matching Windows application events were found." |
                Set-Content -LiteralPath $windowsEventLog -Encoding utf8

            Write-Diagnostic -Message "No matching Windows application events were found."
        }
    }
    catch {
        "Windows event collection failed: $($_.Exception.Message)" |
            Set-Content -LiteralPath $windowsEventLog -Encoding utf8

        Write-Diagnostic -Level "WARN" -Message "Windows event collection failed: $($_.Exception.Message)"
    }

    $finishedAt = Get-Date
    Write-ClipboardSnapshot -Reason "after-application-close"

    $summary = [ordered]@{
        repository = $Repository
        branch = $branch
        commit = $commit
        executable = $executable
        executable_sha256 = $executableHash
        kgw_start_trace = $env:KGW_START_TRACE
        expected_node_ports = $expectedNodePorts
        child_process_evidence = Get-ChildProcessEvidenceSummary
        clipboard_evidence = Get-ClipboardEvidenceSummary
        desktop_pid = if ($desktopProcess) { $desktopProcess.Id } else { $null }
        exit_code = $exitCode
        started_at = $startedAt.ToString("O")
        finished_at = $finishedAt.ToString("O")
        duration_seconds = [math]::Round(($finishedAt - $startedAt).TotalSeconds, 2)
        worktree_was_clean = ($status.Count -eq 0)
        logs = [ordered]@{
            launcher = $launcherLog
            stdout = $stdoutLog
            stderr = $stderrLog
            child_process = $childProcessLog
            clipboard = $clipboardLog
            processes = $processLog
            ports = $portLog
            windows_events = $windowsEventLog
            build = $buildLog
        }
    }

    $summary |
        ConvertTo-Json -Depth 6 |
        Set-Content -LiteralPath $summaryFile -Encoding utf8

    if (Test-Path $zipFile) {
        Remove-Item -LiteralPath $zipFile -Force
    }

    Compress-Archive `
        -Path "$diagnosticDirectory\*" `
        -DestinationPath $zipFile `
        -CompressionLevel Optimal

    Write-Diagnostic -Message "Diagnostic capture completed."
    Write-Diagnostic -Message "DiagnosticArchive=$zipFile"

    Write-Host ""
    Write-Host "DIAGNOSTIC CAPTURE COMPLETED" -ForegroundColor Green
    Write-Host "ZIP archive:" -ForegroundColor Green
    Write-Host $zipFile -ForegroundColor Cyan
}
