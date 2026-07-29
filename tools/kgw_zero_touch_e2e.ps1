param(
    [Parameter(Mandatory)]
    [string]$Repository
)

if ($PSVersionTable.PSVersion.Major -lt 7) {
    throw "Kaspa Gateway zero-touch E2E requires PowerShell 7 or later. Launch with (Get-Command pwsh -ErrorAction Stop).Source; Windows PowerShell $($PSVersionTable.PSVersion) is not supported."
}

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false

. (Join-Path $PSScriptRoot "kgw_zero_touch_evidence.ps1")
Assert-KgwZeroTouchPowerShell7
$RequiredPowerShellPath = (Get-Command pwsh -ErrorAction Stop).Source

function New-RunId {
    return (Get-Date).ToUniversalTime().ToString("yyyyMMddTHHmmssZ") + "-" + ([System.Guid]::NewGuid().ToString("N").Substring(0, 8))
}

function ConvertTo-JsonFile {
    param(
        [AllowNull()][object]$Value = $null,
        [Parameter(Mandatory)][string]$Path,
        [int]$Depth = 8
    )

    $directory = Split-Path -Parent $Path
    if (-not [string]::IsNullOrWhiteSpace($directory)) {
        New-Item -ItemType Directory -Path $directory -Force | Out-Null
    }
    if ($null -eq $Value) {
        "null" | Set-Content -LiteralPath $Path -Encoding utf8
    } else {
        ConvertTo-Json -InputObject $Value -Depth $Depth | Set-Content -LiteralPath $Path -Encoding utf8
    }
}

function Get-AncestorProcessIds {
    $ids = New-Object System.Collections.Generic.HashSet[int]
    $all = @(Get-CimInstance Win32_Process -ErrorAction SilentlyContinue)
    $byId = @{}
    foreach ($process in $all) {
        $byId[[int]$process.ProcessId] = $process
    }

    $current = [int]$PID
    while ($byId.ContainsKey($current)) {
        [void]$ids.Add($current)
        $parent = [int]$byId[$current].ParentProcessId
        if ($parent -le 0 -or $ids.Contains($parent)) {
            break
        }
        $current = $parent
    }

    return ,$ids
}

function Get-KgwOwnedProcessTree {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$ArtifactRoot
    )

    $all = @(Get-CimInstance Win32_Process -ErrorAction SilentlyContinue)
    $childrenByParent = @{}
    foreach ($process in $all) {
        $parent = [string]$process.ParentProcessId
        if (-not $childrenByParent.ContainsKey($parent)) {
            $childrenByParent[$parent] = New-Object System.Collections.Generic.List[object]
        }
        [void]$childrenByParent[$parent].Add($process)
    }

    $roots = New-Object System.Collections.Generic.HashSet[int]
    foreach ($process in $all) {
        $name = [string]$process.Name
        $path = [string]$process.ExecutablePath
        $commandLine = [string]$process.CommandLine
        $pathInRepo = $path -and $path.StartsWith($RepositoryRoot, [System.StringComparison]::OrdinalIgnoreCase)
        $commandInRepo = $commandLine -and $commandLine.IndexOf($RepositoryRoot, [System.StringComparison]::OrdinalIgnoreCase) -ge 0
        $commandInArtifacts = $commandLine -and $commandLine.IndexOf($ArtifactRoot, [System.StringComparison]::OrdinalIgnoreCase) -ge 0

        if ($pathInRepo -and ($name -eq "kaspa-gateway-desktop.exe" -or $commandLine -match "--kgw-self-worker")) {
            [void]$roots.Add([int]$process.ProcessId)
            continue
        }

        if ($commandInRepo -and $commandLine -match "--kgw-self-worker") {
            [void]$roots.Add([int]$process.ProcessId)
            continue
        }

        if (($commandInRepo -or $commandInArtifacts) -and $name -match "^(node|npm|wdio|webdriverio)(\.exe|\.cmd)?$") {
            [void]$roots.Add([int]$process.ProcessId)
        }
    }

    $owned = New-Object System.Collections.Generic.HashSet[int]
    $queue = New-Object System.Collections.Generic.Queue[int]
    foreach ($id in $roots) {
        $queue.Enqueue($id)
    }

    while ($queue.Count -gt 0) {
        $id = $queue.Dequeue()
        if (-not $owned.Add($id)) {
            continue
        }

        $key = [string]$id
        if ($childrenByParent.ContainsKey($key)) {
            foreach ($child in $childrenByParent[$key]) {
                $queue.Enqueue([int]$child.ProcessId)
            }
        }
    }

    $protected = Get-AncestorProcessIds
    return @(
        $all |
            Where-Object { $owned.Contains([int]$_.ProcessId) -and -not $protected.Contains([int]$_.ProcessId) } |
            Sort-Object ParentProcessId, ProcessId |
            Select-Object ProcessId, ParentProcessId, Name, ExecutablePath, CommandLine, CreationDate
    )
}

function Save-KgwProcessSnapshot {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$ArtifactRoot,
        [Parameter(Mandatory)][string]$Name
    )

    $snapshot = Get-KgwOwnedProcessTree -RepositoryRoot $RepositoryRoot -ArtifactRoot $ArtifactRoot
    ConvertTo-JsonFile -Value $snapshot -Path (Join-Path $ArtifactRoot "$Name-process-tree.json") -Depth 8
    return $snapshot
}

function Stop-KgwOwnedProcessTree {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$ArtifactRoot,
        [Parameter(Mandatory)][string]$Phase
    )

    $snapshot = Save-KgwProcessSnapshot -RepositoryRoot $RepositoryRoot -ArtifactRoot $ArtifactRoot -Name $Phase
    foreach ($process in @($snapshot | Sort-Object ProcessId -Descending)) {
        try {
            Stop-Process -Id ([int]$process.ProcessId) -Force -ErrorAction Stop
        }
        catch {
            Write-Warning "Failed to stop repository-owned process $($process.ProcessId): $($_.Exception.Message)"
        }
    }
}

function Invoke-LoggedCommand {
    param(
        [Parameter(Mandatory)][string]$Label,
        [Parameter(Mandatory)][string]$FilePath,
        [string[]]$Arguments = @(),
        [Parameter(Mandatory)][string]$WorkingDirectory,
        [Parameter(Mandatory)][string]$LogPath
    )

    Write-Host "Running: $Label"
    $directory = Split-Path -Parent $LogPath
    if (-not [string]::IsNullOrWhiteSpace($directory)) {
        New-Item -ItemType Directory -Path $directory -Force | Out-Null
    }
    New-Item -ItemType File -Path $LogPath -Force | Out-Null

    Push-Location $WorkingDirectory
    try {
        $previousErrorActionPreference = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        & $FilePath @Arguments 2>&1 | ForEach-Object {
            $line = if ($_ -is [System.Management.Automation.ErrorRecord]) {
                $_.ToString()
            } else {
                [string]$_
            }
            Add-Content -LiteralPath $LogPath -Value $line -Encoding utf8
            Write-Host $line
        }
        $code = if ($null -eq $LASTEXITCODE) { 0 } else { [int]$LASTEXITCODE }
        return $code
    }
    finally {
        $ErrorActionPreference = $previousErrorActionPreference
        Pop-Location
    }
}

$Repository = (Resolve-Path -LiteralPath $Repository).Path
$StartedAt = (Get-Date).ToUniversalTime().ToString("o")
$RunId = if ($env:KGW_ZERO_TOUCH_RUN_ID) { $env:KGW_ZERO_TOUCH_RUN_ID } else { New-RunId }
$ArtifactRoot = if ($env:KGW_ZERO_TOUCH_ARTIFACT_DIR) {
    $env:KGW_ZERO_TOUCH_ARTIFACT_DIR
} else {
    Join-Path $Repository ("artifacts/zero-touch-e2e/{0}" -f $RunId)
}
$ArtifactRoot = [System.IO.Path]::GetFullPath($ArtifactRoot)
$E2eDir = Join-Path $Repository "e2e"
$SrcTauriDir = Join-Path $Repository "apps/kaspa-gateway-desktop/src-tauri"
$E2eConfigPath = Join-Path $SrcTauriDir "tauri.e2e.conf.json"
$CargoTargetDir = if ($env:CARGO_TARGET_DIR) {
    $env:CARGO_TARGET_DIR
} else {
    Join-Path $Repository "target/kgw-zero-touch-e2e"
}
$CargoTargetDir = [System.IO.Path]::GetFullPath($CargoTargetDir)
$AppBinary = Join-Path $CargoTargetDir "debug/kaspa-gateway-desktop.exe"
$Summary = [ordered]@{
    repository = $Repository
    run_id = $RunId
    artifact_root = $ArtifactRoot
    cargo_target_dir = $CargoTargetDir
    app_binary = $AppBinary
    commands = New-Object System.Collections.Generic.List[object]
}
$exitCode = 0
$FailedStage = $null
$EvidenceSummary = $null

New-Item -ItemType Directory -Path $ArtifactRoot -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $ArtifactRoot "localappdata") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $ArtifactRoot "appdata") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $ArtifactRoot "backend-traces") -Force | Out-Null
ConvertTo-JsonFile -Value $Summary -Path (Join-Path $ArtifactRoot "zero-touch-script-start.json") -Depth 8

$previousEnvironment = @{
    KGW_REPOSITORY = $env:KGW_REPOSITORY
    KGW_ZERO_TOUCH_ARTIFACT_DIR = $env:KGW_ZERO_TOUCH_ARTIFACT_DIR
    KGW_E2E_RUN_ID = $env:KGW_E2E_RUN_ID
    KGW_E2E_APP_BINARY = $env:KGW_E2E_APP_BINARY
    KGW_UI_TRACE_FILE = $env:KGW_UI_TRACE_FILE
    KGW_UI_TRACE_DIR = $env:KGW_UI_TRACE_DIR
    KGW_START_TRACE = $env:KGW_START_TRACE
    KGW_REQUIRED_PWSH_PATH = $env:KGW_REQUIRED_PWSH_PATH
    TAURI_CONFIG = $env:TAURI_CONFIG
    LOCALAPPDATA = $env:LOCALAPPDATA
    APPDATA = $env:APPDATA
    CARGO_TARGET_DIR = $env:CARGO_TARGET_DIR
}

try {
    if (-not (Test-Path -LiteralPath $E2eConfigPath)) {
        $FailedStage = "E2E Tauri config validation"
        throw "Missing E2E Tauri config: $E2eConfigPath"
    }
    if (-not (Test-Path -LiteralPath (Join-Path $E2eDir "package-lock.json"))) {
        $FailedStage = "E2E dependency lock validation"
        throw "Missing locked E2E dependencies: e2e/package-lock.json"
    }

    $env:KGW_REPOSITORY = $Repository
    $env:KGW_ZERO_TOUCH_ARTIFACT_DIR = $ArtifactRoot
    $env:KGW_E2E_RUN_ID = $RunId
    $env:KGW_E2E_APP_BINARY = $AppBinary
    $env:KGW_UI_TRACE_FILE = "1"
    $env:KGW_UI_TRACE_DIR = Join-Path $ArtifactRoot "backend-traces"
    $env:KGW_START_TRACE = "1"
    $env:KGW_REQUIRED_PWSH_PATH = $RequiredPowerShellPath
    $env:TAURI_CONFIG = Get-Content -LiteralPath $E2eConfigPath -Raw
    $env:LOCALAPPDATA = Join-Path $ArtifactRoot "localappdata"
    $env:APPDATA = Join-Path $ArtifactRoot "appdata"
    $env:CARGO_TARGET_DIR = $CargoTargetDir

    Save-KgwProcessSnapshot -RepositoryRoot $Repository -ArtifactRoot $ArtifactRoot -Name "pre-run" | Out-Null
    Stop-KgwOwnedProcessTree -RepositoryRoot $Repository -ArtifactRoot $ArtifactRoot -Phase "pre-run-cleanup"

    $nodeModules = Join-Path $E2eDir "node_modules"
    $wdioCli = Join-Path $nodeModules "@wdio/cli/package.json"
    if (-not (Test-Path -LiteralPath $wdioCli)) {
        $code = Invoke-LoggedCommand `
            -Label "Install locked E2E dependencies" `
            -FilePath "npm" `
            -Arguments @("ci") `
            -WorkingDirectory $E2eDir `
            -LogPath (Join-Path $ArtifactRoot "npm-ci.log")
        if ($code -ne 0) {
            [void]$Summary.commands.Add([ordered]@{ label = "npm ci"; status = "failed"; exit_code = $code; log = "npm-ci.log" })
            $exitCode = $code
            $FailedStage = "Install locked E2E dependencies"
            throw "$FailedStage failed with exit code $code"
        }
        [void]$Summary.commands.Add([ordered]@{ label = "npm ci"; status = "passed"; exit_code = 0; log = "npm-ci.log" })
    } else {
        Write-Host "Locked E2E dependencies already installed; skipping npm ci"
        [void]$Summary.commands.Add([ordered]@{ label = "npm ci"; status = "skipped-existing"; exit_code = 0; log = "" })
    }

    $code = Invoke-LoggedCommand `
        -Label "E2E JavaScript syntax checks" `
        -FilePath "npm" `
        -Arguments @("run", "check") `
        -WorkingDirectory $E2eDir `
        -LogPath (Join-Path $ArtifactRoot "npm-run-check.log")
    if ($code -ne 0) {
        [void]$Summary.commands.Add([ordered]@{ label = "npm run check"; status = "failed"; exit_code = $code; log = "npm-run-check.log" })
        $exitCode = $code
        $FailedStage = "E2E JavaScript syntax checks"
        throw "$FailedStage failed with exit code $code"
    }
    [void]$Summary.commands.Add([ordered]@{ label = "npm run check"; status = "passed"; exit_code = 0; log = "npm-run-check.log" })

    $code = Invoke-LoggedCommand `
        -Label "Build desktop E2E binary once" `
        -FilePath "cargo" `
        -Arguments @("build", "--locked", "-p", "kaspa-gateway-desktop", "--bin", "kaspa-gateway-desktop", "--features", "e2e-test") `
        -WorkingDirectory $Repository `
        -LogPath (Join-Path $ArtifactRoot "cargo-build-e2e.log")
    if ($code -ne 0) {
        [void]$Summary.commands.Add([ordered]@{ label = "cargo build --locked -p kaspa-gateway-desktop --bin kaspa-gateway-desktop --features e2e-test"; status = "failed"; exit_code = $code; log = "cargo-build-e2e.log" })
        $exitCode = $code
        $FailedStage = "Build desktop E2E binary once"
        throw "$FailedStage failed with exit code $code"
    }
    [void]$Summary.commands.Add([ordered]@{ label = "cargo build --locked -p kaspa-gateway-desktop --bin kaspa-gateway-desktop --features e2e-test"; status = "passed"; exit_code = 0; log = "cargo-build-e2e.log" })

    if (-not (Test-Path -LiteralPath $AppBinary)) {
        $FailedStage = "Build desktop E2E binary once"
        throw "E2E desktop binary was not created at $AppBinary"
    }

    $code = Invoke-LoggedCommand `
        -Label "WebdriverIO zero-touch live matrix" `
        -FilePath "npm" `
        -Arguments @("run", "e2e") `
        -WorkingDirectory $E2eDir `
        -LogPath (Join-Path $ArtifactRoot "wdio-run.log")
    if ($code -ne 0) {
        [void]$Summary.commands.Add([ordered]@{ label = "npm run e2e"; status = "failed"; exit_code = $code; log = "wdio-run.log" })
        $exitCode = $code
        $FailedStage = "WebdriverIO zero-touch live matrix"
        throw "$FailedStage failed with exit code $code"
    }
    [void]$Summary.commands.Add([ordered]@{ label = "npm run e2e"; status = "passed"; exit_code = 0; log = "wdio-run.log" })

    $EvidenceSummary = Get-KgwZeroTouchEvidenceSummary -ArtifactDirectory $ArtifactRoot
    $Summary["evidence_validation"] = $EvidenceSummary
    if (-not $EvidenceSummary.passed) {
        $exitCode = 1
        $FailedStage = if ($EvidenceSummary.failed_stage) { $EvidenceSummary.failed_stage } else { "Zero-touch evidence validation" }
        throw "Zero-touch evidence validation failed: $($EvidenceSummary.validation_errors -join ' | ')"
    }
}
catch {
    if ($exitCode -eq 0) {
        $exitCode = 1
    }
    if ([string]::IsNullOrWhiteSpace($FailedStage)) {
        $FailedStage = "Zero-touch script"
    }
    $Summary["error"] = $_.Exception.Message
    [Console]::Error.WriteLine($_.Exception.Message)
}
finally {
    try {
        Stop-KgwOwnedProcessTree -RepositoryRoot $Repository -ArtifactRoot $ArtifactRoot -Phase "finally-cleanup"
        Save-KgwProcessSnapshot -RepositoryRoot $Repository -ArtifactRoot $ArtifactRoot -Name "post-cleanup" | Out-Null
    }
    catch {
        Write-Warning "Final process cleanup failed: $($_.Exception.Message)"
        if ($exitCode -eq 0) {
            $exitCode = 1
        }
        if ([string]::IsNullOrWhiteSpace($FailedStage)) {
            $FailedStage = "Final process cleanup"
        }
        $Summary["cleanup_error"] = $_.Exception.Message
    }

    foreach ($entry in @($previousEnvironment.GetEnumerator())) {
        if ($null -eq $entry.Value) {
            Remove-Item -Path ("Env:{0}" -f $entry.Key) -ErrorAction SilentlyContinue
        } else {
            Set-Item -Path ("Env:{0}" -f $entry.Key) -Value ([string]$entry.Value)
        }
    }

    $resultPath = Join-Path $ArtifactRoot "zero-touch-result.json"
    try {
        if ($null -eq $EvidenceSummary) {
            $EvidenceSummary = Get-KgwZeroTouchEvidenceSummary -ArtifactDirectory $ArtifactRoot
        }
        if ($exitCode -eq 0 -and -not $EvidenceSummary.passed) {
            $exitCode = 1
            if ([string]::IsNullOrWhiteSpace($FailedStage)) {
                $FailedStage = if ($EvidenceSummary.failed_stage) { $EvidenceSummary.failed_stage } else { "Zero-touch evidence validation" }
            }
        }
        $result = New-KgwZeroTouchResultObject `
            -Repository $Repository `
            -ArtifactDirectory $ArtifactRoot `
            -StartedAt $StartedAt `
            -ExitCode $exitCode `
            -FailedStage $FailedStage `
            -ExecutablePath $AppBinary `
            -EvidenceSummary $EvidenceSummary
        Write-KgwZeroTouchJsonFile -Value $result -Path $resultPath -Depth 16
        $Summary["result_file"] = $resultPath
        $Summary["passed_stages"] = $result.passed_stages
        $Summary["failed_stage"] = $result.failed_stage
    }
    catch {
        $originalExitCode = $exitCode
        $originalFailedStage = $FailedStage
        $writerError = $_
        $fallback = New-KgwZeroTouchWriterFailureResultObject `
            -Repository $Repository `
            -ArtifactDirectory $ArtifactRoot `
            -StartedAt $StartedAt `
            -OriginalExitCode $originalExitCode `
            -OriginalFailedStage $originalFailedStage `
            -ExecutablePath $AppBinary `
            -WriterError $writerError `
            -ValidationErrors @((Get-KgwZeroTouchProperty -Object $EvidenceSummary -Names @("validation_errors")))
        try {
            $fallback.git_commit = Get-KgwZeroTouchCurrentGitCommit -Repository $Repository
            $fallback.source_diff_sha256 = Get-KgwZeroTouchSourceDiffSha256 -Repository $Repository
            if (Test-Path -LiteralPath $AppBinary -PathType Leaf) {
                $fallback.executable_sha256 = Get-KgwZeroTouchSha256ForFile -Path $AppBinary
            }
        }
        catch {
            $metadataError = ConvertTo-KgwZeroTouchSerializableException -Value $_
            $errors = New-Object System.Collections.Generic.List[string]
            foreach ($error in @($fallback.validation_errors)) {
                [void]$errors.Add([string]$error)
            }
            [void]$errors.Add("Fallback metadata collection failed: $($metadataError.message)")
            $fallback.validation_errors = $errors.ToArray()
        }
        Write-KgwZeroTouchEmergencyJsonFile -Value $fallback -Path $resultPath -Depth 16
        $exitCode = if ($originalExitCode -ne 0) { $originalExitCode } else { 1 }
        $FailedStage = "Zero-touch result writing"
        $Summary["result_file"] = $resultPath
        $Summary["failed_stage"] = $FailedStage
        $Summary["result_writer_error"] = ConvertTo-KgwZeroTouchSerializableException -Value $writerError
    }

    $Summary["finished_at"] = (Get-Date).ToUniversalTime().ToString("o")
    $Summary["exit_code"] = $exitCode
    ConvertTo-JsonFile -Value $Summary -Path (Join-Path $ArtifactRoot "zero-touch-script-summary.json") -Depth 8
    $report = @(
        "# Kaspa Gateway Zero-Touch E2E Script Report",
        "",
        "Artifact root: $ArtifactRoot",
        "Run ID: $RunId",
        "Exit code: $exitCode",
        "App binary: $AppBinary",
        "Cargo target dir: $CargoTargetDir"
    )
    if ($Summary.Contains("error")) {
        $report += ""
        $report += "Error: $($Summary["error"])"
    }
    $report -join "`n" | Set-Content -LiteralPath (Join-Path $ArtifactRoot "zero-touch-script-report.md") -Encoding utf8
}

exit $exitCode
