[CmdletBinding()]
param(
    [ValidateSet("mainnet", "testnet10")]
    [string[]]$Networks = @("mainnet", "testnet10"),

    [ValidateRange(30, 900)]
    [int]$ReadyTimeoutSeconds = 240,

    [ValidateRange(15, 600)]
    [int]$ObservationSeconds = 45,

    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false

if (-not $IsWindows) {
    throw "The live embedded-runtime smoke test must run on Windows."
}

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $repoRoot

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "cargo was not found. Install Rust with rustup and reopen PowerShell."
}

function Refresh-ProcessPath {
    $machinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $env:Path = (@($machinePath, $userPath) | Where-Object { $_ }) -join ";"
}

function Install-WingetPackage {
    param(
        [Parameter(Mandatory)]
        [string]$Id
    )

    if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
        throw "Missing build dependency $Id and winget is unavailable."
    }

    Write-Host "Installing required build dependency: $Id" -ForegroundColor Yellow
    winget install `
        --id $Id `
        --exact `
        --source winget `
        --silent `
        --accept-source-agreements `
        --accept-package-agreements

    if ($LASTEXITCODE -ne 0) {
        throw "winget could not install required build dependency: $Id"
    }

    Refresh-ProcessPath
}

$requiredRust = [version]"1.91.0"
$rustVersionText = ((& rustc --version) -split "\s+")[1]
if ([version]$rustVersionText -lt $requiredRust) {
    if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
        throw "Rust $requiredRust or newer is required by rusty-kaspa v2.0.1."
    }

    Write-Host "Updating Rust because rusty-kaspa v2.0.1 requires Rust $requiredRust or newer..." -ForegroundColor Yellow
    rustup update stable
    if ($LASTEXITCODE -ne 0) {
        throw "rustup could not update the stable Rust toolchain."
    }
}

if (-not (Get-Command protoc -ErrorAction SilentlyContinue)) {
    Install-WingetPackage -Id "Google.Protobuf"
}

$protocCommand = Get-Command protoc -ErrorAction SilentlyContinue
if (-not $protocCommand) {
    $protocCommand = Get-ChildItem `
        -Path (Join-Path $env:LOCALAPPDATA "Microsoft\WinGet\Packages") `
        -Filter "protoc.exe" `
        -File `
        -Recurse `
        -ErrorAction SilentlyContinue |
        Select-Object -First 1
}
if (-not $protocCommand) {
    throw "protoc is required for Kaspa gRPC generation but was not found after installation."
}
$env:PROTOC = $protocCommand.Source ?? $protocCommand.FullName

$llvmBin = Join-Path $env:ProgramFiles "LLVM\bin"
$libclang = Join-Path $llvmBin "libclang.dll"
if (-not (Test-Path $libclang)) {
    Install-WingetPackage -Id "LLVM.LLVM"
}
if (-not (Test-Path $libclang)) {
    throw "libclang.dll is required for the Kaspa RocksDB build and was not found at $libclang."
}
$env:LIBCLANG_PATH = $llvmBin
if ($env:Path -notlike "*$llvmBin*") {
    $env:Path = "$llvmBin;$env:Path"
}

$llvmAr = Join-Path $llvmBin "llvm-ar.exe"
$ar = Join-Path $llvmBin "ar.exe"
if ((Test-Path $llvmAr) -and -not (Test-Path $ar)) {
    Copy-Item -Path $llvmAr -Destination $ar
}

$profiles = @{
    mainnet = @{
        RpcPort = 16110
        ExpectedNetwork = "mainnet"
    }
    testnet10 = @{
        RpcPort = 16210
        ExpectedNetwork = "testnet10"
    }
}

$computer = Get-CimInstance Win32_ComputerSystem
$logicalCores = [int]$computer.NumberOfLogicalProcessors
$memoryGb = [math]::Round([double]$computer.TotalPhysicalMemory / 1GB, 1)
$driveName = (Split-Path -Qualifier $repoRoot).TrimEnd("\", ":")
$drive = Get-PSDrive -Name $driveName
$freeDiskGb = [math]::Round([double]$drive.Free / 1GB, 1)

$preflight = [ordered]@{
    LogicalCores = $logicalCores
    MemoryGB = $memoryGb
    FreeDiskGB = $freeDiskGb
    MeetsProductionCoreMinimum = $logicalCores -ge 8
    MeetsProductionMemoryMinimum = $memoryGb -ge 16
    MeetsProductionDiskMinimum = $freeDiskGb -ge 640
    FullInitialBlockDownloadRequested = $false
    MainnetTransactionsOrMiningRequested = $false
}

Write-Host "Kaspa live smoke preflight" -ForegroundColor Cyan
Write-Host ("cores={0}; memory_gb={1}; free_disk_gb={2}" -f $logicalCores, $memoryGb, $freeDiskGb)

if (-not $preflight.MeetsProductionDiskMinimum) {
    Write-Warning "Less than 640 GB is free. The short smoke test may run, but do not attempt a full production sync on this drive."
}

foreach ($network in $Networks) {
    $port = [int]$profiles[$network].RpcPort
    if (Get-NetTCPConnection -LocalPort $port -State Listen -ErrorAction SilentlyContinue) {
        throw "RPC port $port for $network is already in use. Stop the existing service before testing."
    }
}

if (-not $SkipBuild) {
    cargo build -p kaspa-gateway-desktop --release --features official-kaspa-runtime-mainline
    if ($LASTEXITCODE -ne 0) {
        throw "Desktop mainline runtime build failed."
    }

    cargo build -p kaspa-gateway-cli --release --bin kgw-live-probe --features live-network-probe
    if ($LASTEXITCODE -ne 0) {
        throw "Live RPC probe build failed."
    }
}

$desktopExe = Join-Path $repoRoot "target\release\kaspa-gateway-desktop.exe"
$probeExe = Join-Path $repoRoot "target\release\kgw-live-probe.exe"

foreach ($path in @($desktopExe, $probeExe)) {
    if (-not (Test-Path $path)) {
        throw "Required executable was not found: $path"
    }
}

$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$reportDir = Join-Path $repoRoot "artifacts\live-network-smoke\$timestamp"
New-Item -ItemType Directory -Path $reportDir -Force | Out-Null

function Invoke-LiveProbe {
    param(
        [Parameter(Mandatory)]
        [string]$Endpoint,

        [Parameter(Mandatory)]
        [string]$ExpectedNetwork
    )

    $nativeOutput = & $probeExe --rpc $Endpoint --expect-network $ExpectedNetwork 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw (($nativeOutput | ForEach-Object { [string]$_ }) -join "`n")
    }

    $jsonLine = ($nativeOutput | Select-Object -Last 1)
    return $jsonLine | ConvertFrom-Json
}

function Wait-RpcReady {
    param(
        [Parameter(Mandatory)]
        [System.Diagnostics.Process]$Process,

        [Parameter(Mandatory)]
        [string]$Endpoint,

        [Parameter(Mandatory)]
        [string]$ExpectedNetwork
    )

    $deadline = (Get-Date).AddSeconds($ReadyTimeoutSeconds)
    $lastError = ""

    while ((Get-Date) -lt $deadline) {
        if ($Process.HasExited) {
            throw "Node process exited before RPC became ready. exit_code=$($Process.ExitCode)"
        }

        try {
            return Invoke-LiveProbe -Endpoint $Endpoint -ExpectedNetwork $ExpectedNetwork
        }
        catch {
            $lastError = $_.Exception.Message
            Start-Sleep -Seconds 5
        }
    }

    throw "RPC did not become ready within $ReadyTimeoutSeconds seconds. last_error=$lastError"
}

$results = @()

foreach ($network in $Networks) {
    $profile = $profiles[$network]
    $rpcPort = [int]$profile.RpcPort
    $endpoint = "grpc://127.0.0.1:$rpcPort"
    $appDir = Join-Path $env:LOCALAPPDATA "KaspaGateway\nodes\$network"
    $stdoutPath = Join-Path $reportDir "$network.stdout.log"
    $stderrPath = Join-Path $reportDir "$network.stderr.log"
    $process = $null
    $startedAt = Get-Date

    Write-Host "Starting live $network node smoke test..." -ForegroundColor Cyan

    try {
        $process = Start-Process `
            -FilePath $desktopExe `
            -ArgumentList @(
                "--kgw-self-worker", "node",
                "--network", $network,
                "--appdir", "`"$appDir`"",
                "--rpc", "127.0.0.1:$rpcPort"
            ) `
            -RedirectStandardOutput $stdoutPath `
            -RedirectStandardError $stderrPath `
            -WindowStyle Hidden `
            -PassThru

        $first = Wait-RpcReady `
            -Process $process `
            -Endpoint $endpoint `
            -ExpectedNetwork $profile.ExpectedNetwork

        Start-Sleep -Seconds $ObservationSeconds

        if ($process.HasExited) {
            throw "Node process exited during the observation window. exit_code=$($process.ExitCode)"
        }

        $second = Invoke-LiveProbe `
            -Endpoint $endpoint `
            -ExpectedNetwork $profile.ExpectedNetwork

        $peerCount = [int]$second.peerCount
        $daaProgressed = [uint64]$second.virtualDaaScore -gt [uint64]$first.virtualDaaScore
        $blockProgressed = [uint64]$second.blockCount -gt [uint64]$first.blockCount
        $headerProgressed = [uint64]$second.headerCount -gt [uint64]$first.headerCount
        $chainProgressObserved = $daaProgressed -or $blockProgressed -or $headerProgressed
        $healthy = $peerCount -gt 0
        $observationWarning = if ($chainProgressObserved) {
            $null
        } else {
            "No chain counter changed during the short observation window. This is informational during initial block download; use an extended synchronization test for production readiness."
        }

        $result = [ordered]@{
            Network = $network
            Success = $healthy
            ProcessStayedAlive = $true
            RpcReady = $true
            RpcEndpoint = $endpoint
            ServerVersion = [string]$second.serverVersion
            PeerCount = $peerCount
            IsSynced = [bool]$second.isSynced
            FirstVirtualDaaScore = [uint64]$first.virtualDaaScore
            SecondVirtualDaaScore = [uint64]$second.virtualDaaScore
            FirstBlockCount = [uint64]$first.blockCount
            SecondBlockCount = [uint64]$second.blockCount
            FirstHeaderCount = [uint64]$first.headerCount
            SecondHeaderCount = [uint64]$second.headerCount
            DaaProgressed = $daaProgressed
            BlockProgressed = $blockProgressed
            HeaderProgressed = $headerProgressed
            ChainProgressObserved = $chainProgressObserved
            ObservationWarning = $observationWarning
            FullSyncRequiredForProduction = -not [bool]$second.isSynced
            DurationSeconds = [math]::Round(((Get-Date) - $startedAt).TotalSeconds, 1)
            StdoutLog = $stdoutPath
            StderrLog = $stderrPath
        }

        if (-not $healthy) {
            $result.Failure = "RPC opened, but no connected peers were reported."
        }

        $results += [pscustomobject]$result
    }
    catch {
        $results += [pscustomobject][ordered]@{
            Network = $network
            Success = $false
            ProcessStayedAlive = [bool]($process -and -not $process.HasExited)
            RpcReady = $false
            RpcEndpoint = $endpoint
            Failure = $_.Exception.Message
            DurationSeconds = [math]::Round(((Get-Date) - $startedAt).TotalSeconds, 1)
            StdoutLog = $stdoutPath
            StderrLog = $stderrPath
        }
    }
    finally {
        if ($process -and -not $process.HasExited) {
            Stop-Process -Id $process.Id -Force
            $process.WaitForExit()
        }

        $releaseDeadline = (Get-Date).AddSeconds(20)
        while ((Get-Date) -lt $releaseDeadline) {
            if (-not (Get-NetTCPConnection -LocalPort $rpcPort -State Listen -ErrorAction SilentlyContinue)) {
                break
            }
            Start-Sleep -Milliseconds 500
        }
    }
}

$report = [ordered]@{
    SchemaVersion = 1
    GeneratedAt = (Get-Date).ToString("o")
    TestKind = "short-live-network-smoke"
    StableRuntime = "rusty-kaspa v2.0.1"
    StableRuntimeCommit = "cfafeb4c093fa37a303f1b9f19c58f986b870ce3"
    TestedNetworks = $Networks
    ExperimentalTestnet12Started = $false
    Preflight = $preflight
    Results = $results
    Passed = -not ($results | Where-Object { -not $_.Success })
}

$reportPath = Join-Path $reportDir "report.json"
$report | ConvertTo-Json -Depth 8 | Set-Content -Path $reportPath -Encoding utf8NoBOM

Write-Host "Live smoke report: $reportPath" -ForegroundColor Cyan

if (-not $report.Passed) {
    $results | Format-Table Network, Success, RpcReady, PeerCount, DaaProgressed, Failure -AutoSize
    throw "One or more live network smoke tests failed. Review report.json and the captured logs."
}

$results | Format-Table Network, Success, ServerVersion, PeerCount, IsSynced, DaaProgressed -AutoSize
Write-Host "MAINNET AND TESTNET10 LIVE SMOKE TESTS PASSED" -ForegroundColor Green
