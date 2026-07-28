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
$windowsEventLog = Join-Path $diagnosticDirectory "windows-events.log"
$summaryFile = Join-Path $diagnosticDirectory "summary.json"
$zipFile = "$diagnosticDirectory.zip"
$executable = Join-Path $Repository "target\debug\kaspa-gateway-desktop.exe"
$buildLog = Join-Path $diagnosticDirectory "build.log"
$expectedNodePorts = @(16110, 17110, 18110, 16210, 17210, 18210, 16310, 17310, 18310)

New-Item -ItemType Directory -Path $diagnosticDirectory -Force | Out-Null
New-Item -ItemType File -Path $stdoutLog -Force | Out-Null
New-Item -ItemType File -Path $stderrLog -Force | Out-Null
New-Item -ItemType File -Path $launcherLog -Force | Out-Null
New-Item -ItemType File -Path $processLog -Force | Out-Null
New-Item -ItemType File -Path $portLog -Force | Out-Null

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

        if ($line.StartsWith("[KGW_START_TRACE]", [System.StringComparison]::Ordinal)) {
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

Set-Location $Repository

$branch = (git branch --show-current).Trim()
$commit = (git rev-parse HEAD).Trim()
$status = @(git status --short)

Write-Diagnostic -Message "Repository=$Repository"
Write-Diagnostic -Message "Branch=$branch"
Write-Diagnostic -Message "Commit=$commit"
Write-Diagnostic -Message "DiagnosticDirectory=$diagnosticDirectory"
Write-Diagnostic -Message "ExpectedNodePorts=$($expectedNodePorts -join ',')"

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

    $summary = [ordered]@{
        repository = $Repository
        branch = $branch
        commit = $commit
        executable = $executable
        executable_sha256 = $executableHash
        kgw_start_trace = $env:KGW_START_TRACE
        expected_node_ports = $expectedNodePorts
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
    Write-Host "Upload this archive:" -ForegroundColor Green
    Write-Host $zipFile -ForegroundColor Cyan
}
