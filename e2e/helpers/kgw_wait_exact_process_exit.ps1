param(
    [Parameter(Mandatory)][int]$ProcessId,
    [Parameter(Mandatory)][string]$ExpectedExecutable,
    [Parameter(Mandatory)][long]$ExpectedStartTime,
    [Parameter(Mandatory)][string]$OutputPath,
    [int]$TimeoutSeconds = 45
)
$ErrorActionPreference = "Stop"

function Normalize-Executable([string]$Value) {
    $text = [string]$Value
    if ($text.StartsWith('\\?\')) { $text = $text.Substring(4) }
    return $text.Trim().TrimEnd('\').ToLowerInvariant()
}

$deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
$evidence = [ordered]@{
    process_id = $ProcessId
    expected_executable = $ExpectedExecutable
    expected_start_time = $ExpectedStartTime
    exact_identity_exited = $false
    pid_reused = $false
    observed_at = $null
}
while ([DateTime]::UtcNow -lt $deadline) {
    $process = Get-CimInstance Win32_Process -Filter "ProcessId=$ProcessId" -ErrorAction SilentlyContinue
    if (-not $process) {
        $evidence.exact_identity_exited = $true
        break
    }

    $runtime = Get-Process -Id $ProcessId -ErrorAction SilentlyContinue
    if (-not $runtime) {
        $evidence.exact_identity_exited = $true
        break
    }

    $actualExecutable = [string]$process.ExecutablePath
    $actualStartTime = [DateTimeOffset]::new($runtime.StartTime.ToUniversalTime()).ToUnixTimeSeconds()
    $sameExecutable = (Normalize-Executable $actualExecutable) -eq (Normalize-Executable $ExpectedExecutable)
    $sameStartTime = [Math]::Abs($actualStartTime - $ExpectedStartTime) -le 2
    if (-not ($sameExecutable -and $sameStartTime)) {
        $evidence.exact_identity_exited = $true
        $evidence.pid_reused = $true
        $evidence["replacement_executable"] = $actualExecutable
        $evidence["replacement_start_time"] = $actualStartTime
        break
    }
    Start-Sleep -Milliseconds 100
}
if (-not $evidence.exact_identity_exited) {
    throw "exact process identity $ProcessId did not exit within $TimeoutSeconds seconds"
}

$evidence.observed_at = [DateTimeOffset]::UtcNow.ToString('o')
$directory = Split-Path -Parent $OutputPath
if ($directory) { New-Item -ItemType Directory -Path $directory -Force | Out-Null }
$json = $evidence | ConvertTo-Json -Depth 6
[System.IO.File]::WriteAllText($OutputPath, $json, [System.Text.UTF8Encoding]::new($false))
$json
