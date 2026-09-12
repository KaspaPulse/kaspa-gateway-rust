param(
    [Parameter(Mandatory)][int]$ProcessId,
    [Parameter(Mandatory)][string]$ExpectedExecutable,
    [Parameter(Mandatory)][long]$ExpectedStartTime,
    [Parameter(Mandatory)][string]$OutputPath
)
$ErrorActionPreference = "Stop"

function Normalize-Executable([string]$Value) {
    $text = [string]$Value
    if ($text.StartsWith('\\?\')) { $text = $text.Substring(4) }
    return $text.Trim().TrimEnd('\').ToLowerInvariant()
}

$process = Get-CimInstance Win32_Process -Filter "ProcessId=$ProcessId"
if (-not $process) { throw "owned process $ProcessId is not running" }
$runtime = Get-Process -Id $ProcessId -ErrorAction Stop
$actualExecutable = [string]$process.ExecutablePath
$actualStartTime = [DateTimeOffset]::new($runtime.StartTime.ToUniversalTime()).ToUnixTimeSeconds()
if ((Normalize-Executable $actualExecutable) -ne (Normalize-Executable $ExpectedExecutable)) {
    throw "owned process executable mismatch for PID $ProcessId"
}
if ([Math]::Abs($actualStartTime - $ExpectedStartTime) -gt 2) {
    throw "owned process start-time mismatch for PID $ProcessId"
}

$evidence = [ordered]@{
    process_id = $ProcessId
    expected_executable = $ExpectedExecutable
    actual_executable = $actualExecutable
    expected_start_time = $ExpectedStartTime
    actual_start_time = $actualStartTime
    killed = $false
    observed_at = [DateTimeOffset]::UtcNow.ToString('o')
}
Stop-Process -Id $ProcessId -Force -ErrorAction Stop
$deadline = [DateTime]::UtcNow.AddSeconds(15)
while ([DateTime]::UtcNow -lt $deadline) {
    if (-not (Get-Process -Id $ProcessId -ErrorAction SilentlyContinue)) { break }
    Start-Sleep -Milliseconds 100
}
if (Get-Process -Id $ProcessId -ErrorAction SilentlyContinue) {
    throw "owned process $ProcessId did not terminate after force kill"
}
$evidence.killed = $true
$directory = Split-Path -Parent $OutputPath
if ($directory) { New-Item -ItemType Directory -Path $directory -Force | Out-Null }
$json = $evidence | ConvertTo-Json -Depth 6
[System.IO.File]::WriteAllText($OutputPath, $json, [System.Text.UTF8Encoding]::new($false))
$json
