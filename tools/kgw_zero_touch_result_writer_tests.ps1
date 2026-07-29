if ($PSVersionTable.PSVersion.Major -lt 7) {
    throw "Kaspa Gateway zero-touch result writer tests require PowerShell 7 or later. Launch with (Get-Command pwsh -ErrorAction Stop).Source; Windows PowerShell $($PSVersionTable.PSVersion) is not supported."
}

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false

$Repository = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot "kgw_zero_touch_evidence.ps1")

$Failures = New-Object System.Collections.Generic.List[string]
$TestRoot = Join-Path $Repository ("artifacts/zero-touch-result-writer-tests/{0}" -f [System.Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $TestRoot -Force | Out-Null

function Add-TestFailure {
    param([Parameter(Mandatory)][string]$Message)
    [void]$Failures.Add($Message)
}

function Assert-True {
    param(
        [Parameter(Mandatory)][bool]$Condition,
        [Parameter(Mandatory)][string]$Message
    )
    if (-not $Condition) {
        Add-TestFailure $Message
    }
}

function Assert-Equal {
    param(
        [AllowNull()]$Actual,
        [AllowNull()]$Expected,
        [Parameter(Mandatory)][string]$Message
    )
    if ($Actual -ne $Expected) {
        Add-TestFailure "$Message Expected '$Expected', got '$Actual'."
    }
}

function Read-TestJson {
    param([Parameter(Mandatory)][string]$Path)
    return Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json -ErrorAction Stop
}

try {
    $successPath = Join-Path $TestRoot "success.json"
    $successValue = [ordered]@{
        completed = $true
        success = $true
        exit_code = 0
        labels = @("Mainnet Node", "Testnet10 Bridge")
        nested = [ordered]@{
            pid = 1234
            port = 16110
            endpoint = "127.0.0.1:16110"
        }
    }
    Write-KgwZeroTouchJsonFile -Value $successValue -Path $successPath -Depth 8
    $successJson = Read-TestJson -Path $successPath
    Assert-Equal -Actual ([bool]$successJson.success) -Expected $true -Message "successful result serialization"
    Assert-Equal -Actual ([int]$successJson.nested.pid) -Expected 1234 -Message "PID serialized as integer"
    Assert-Equal -Actual ([int]$successJson.nested.port) -Expected 16110 -Message "port serialized as integer"

    $bomBytes = [System.IO.File]::ReadAllBytes($successPath)
    $hasBom = $bomBytes.Length -ge 3 -and $bomBytes[0] -eq 0xef -and $bomBytes[1] -eq 0xbb -and $bomBytes[2] -eq 0xbf
    Assert-True -Condition (-not $hasBom) -Message "result JSON must be UTF-8 without BOM"

    $atomicPath = Join-Path $TestRoot "atomic.json"
    [System.IO.File]::WriteAllText($atomicPath, '{"version":1}', (Get-KgwZeroTouchUtf8NoBomEncoding))
    Write-KgwZeroTouchJsonFile -Value ([ordered]@{ version = 2; replaced = $true }) -Path $atomicPath -Depth 4
    $atomicJson = Read-TestJson -Path $atomicPath
    Assert-Equal -Actual ([int]$atomicJson.version) -Expected 2 -Message "atomic replacement wrote the new value"
    $leftoverTemps = @(Get-ChildItem -LiteralPath $TestRoot -Force -File | Where-Object { $_.Name -like ".atomic.json.*.tmp" })
    Assert-Equal -Actual $leftoverTemps.Count -Expected 0 -Message "atomic replacement left no temp file"

    $failedPath = Join-Path $TestRoot "failed-primary.json"
    [System.IO.File]::WriteAllText($failedPath, '{"stable":true}', (Get-KgwZeroTouchUtf8NoBomEncoding))
    $primaryFailed = $false
    try {
        Write-KgwZeroTouchJsonFile -Value ([ordered]@{ bad = (Get-Item -LiteralPath $PSScriptRoot) }) -Path $failedPath -Depth 4
    }
    catch {
        $primaryFailed = $true
        Assert-True -Condition ($_.Exception.Message -match "FileInfo|DirectoryInfo|JSON-safe") -Message "failed serialization reports rejected complex object type"
    }
    Assert-True -Condition $primaryFailed -Message "primary writer must reject complex .NET objects"
    Assert-Equal -Actual ([System.IO.File]::ReadAllText($failedPath, [System.Text.Encoding]::UTF8)) -Expected '{"stable":true}' -Message "failed primary serialization must preserve existing result file"

    $exceptionRecord = $null
    try {
        throw "synthetic writer failure"
    }
    catch {
        $exceptionRecord = ConvertTo-KgwZeroTouchSerializableException -Value $_
    }
    Assert-Equal -Actual $exceptionRecord.message -Expected "synthetic writer failure" -Message "exception message serialization"
    Assert-True -Condition (-not [string]::IsNullOrWhiteSpace($exceptionRecord.type)) -Message "exception type serialization"
    Assert-True -Condition ($null -ne $exceptionRecord.stack) -Message "exception stack serialization"

    $fallbackPath = Join-Path $TestRoot "fallback.json"
    $fallback = New-KgwZeroTouchWriterFailureResultObject `
        -Repository $Repository `
        -ArtifactDirectory $TestRoot `
        -StartedAt "2026-07-29T00:00:00.0000000Z" `
        -OriginalExitCode 37 `
        -OriginalFailedStage "WebdriverIO zero-touch live matrix" `
        -ExecutablePath "" `
        -WriterError $exceptionRecord `
        -ValidationErrors @("original validation detail")
    Write-KgwZeroTouchEmergencyJsonFile -Value $fallback -Path $fallbackPath -Depth 8
    $fallbackJson = Read-TestJson -Path $fallbackPath
    Assert-Equal -Actual ([bool]$fallbackJson.success) -Expected $false -Message "failed result serialization fallback success flag"
    Assert-Equal -Actual ([int]$fallbackJson.exit_code) -Expected 37 -Message "fallback preserves original exit code"
    Assert-Equal -Actual ([int]$fallbackJson.original_exit_code) -Expected 37 -Message "fallback records original exit code"
    Assert-Equal -Actual ([string]$fallbackJson.failed_stage) -Expected "Zero-touch result writing" -Message "fallback identifies result-writing stage"
    Assert-True -Condition (-not [string]::IsNullOrWhiteSpace([string]$fallbackJson.result_writer_error.message)) -Message "fallback serializes writer exception"

    $rejectedOldPowerShell = $false
    try {
        Assert-KgwZeroTouchPowerShell7 -MajorVersion 5
    }
    catch {
        $rejectedOldPowerShell = $true
    }
    Assert-True -Condition $rejectedOldPowerShell -Message "PowerShell major version below 7 must be rejected"
    Assert-KgwZeroTouchPowerShell7 -MajorVersion 7
}
catch {
    Add-TestFailure "Unexpected test harness error: $($_.Exception.Message)"
}

if ($Failures.Count -gt 0) {
    Write-Host "KGW zero-touch result writer tests FAILED" -ForegroundColor Red
    foreach ($failure in $Failures) {
        Write-Host "- $failure" -ForegroundColor Red
    }
    exit 1
}

Write-Host "KGW zero-touch result writer tests PASSED"
Write-Host "Test artifacts: $TestRoot"
exit 0
