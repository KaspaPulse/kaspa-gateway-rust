param(
    [Parameter(Mandatory)]
    [string]$Repository,

    [Parameter(Mandatory)]
    [string]$OutputDirectory,

    [AllowEmptyString()]
    [string]$Ports = "",

    [AllowEmptyString()]
    [string]$DesktopPid = ""
)

if ($PSVersionTable.PSVersion.Major -lt 7) {
    throw "Kaspa Gateway zero-touch E2E helper scripts require PowerShell 7 or later. Launch with (Get-Command pwsh -ErrorAction Stop).Source; Windows PowerShell $($PSVersionTable.PSVersion) is not supported."
}

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Repository = (Resolve-Path -LiteralPath $Repository).Path
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null

$allProcesses = @(Get-CimInstance Win32_Process -ErrorAction SilentlyContinue)
$childrenByParent = @{}
foreach ($process in $allProcesses) {
    $parent = [string]$process.ParentProcessId
    if (-not $childrenByParent.ContainsKey($parent)) {
        $childrenByParent[$parent] = New-Object System.Collections.Generic.List[object]
    }
    [void]$childrenByParent[$parent].Add($process)
}

$rootIds = New-Object System.Collections.Generic.HashSet[int]
foreach ($process in $allProcesses) {
    $path = [string]$process.ExecutablePath
    $commandLine = [string]$process.CommandLine
    if (($path -and $path.StartsWith($Repository, [System.StringComparison]::OrdinalIgnoreCase)) -or
        ($commandLine -and $commandLine.IndexOf($Repository, [System.StringComparison]::OrdinalIgnoreCase) -ge 0 -and $commandLine -match "kaspa-gateway")) {
        [void]$rootIds.Add([int]$process.ProcessId)
    }
}

if ($DesktopPid -match '^\d+$') {
    [void]$rootIds.Add([int]$DesktopPid)
}

$wantedIds = New-Object System.Collections.Generic.HashSet[int]
$queue = New-Object System.Collections.Generic.Queue[int]
foreach ($id in $rootIds) {
    $queue.Enqueue($id)
}

while ($queue.Count -gt 0) {
    $id = $queue.Dequeue()
    if (-not $wantedIds.Add($id)) {
        continue
    }
    $key = [string]$id
    if ($childrenByParent.ContainsKey($key)) {
        foreach ($child in $childrenByParent[$key]) {
            $queue.Enqueue([int]$child.ProcessId)
        }
    }
}

$processTree = @(
    $allProcesses |
        Where-Object { $wantedIds.Contains([int]$_.ProcessId) } |
        Sort-Object ParentProcessId, ProcessId |
        Select-Object ProcessId, ParentProcessId, Name, ExecutablePath, CommandLine, CreationDate
)
$processTree | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $OutputDirectory "process-tree.json") -Encoding utf8

$portNumbers = @(
    $Ports.Split(",", [System.StringSplitOptions]::RemoveEmptyEntries) |
        ForEach-Object { $_.Trim() } |
        Where-Object { $_ -match '^\d+$' } |
        ForEach-Object { [int]$_ }
)

$portState = @()
if ($portNumbers.Count -gt 0) {
    $portState = @(
        Get-NetTCPConnection -ErrorAction SilentlyContinue |
            Where-Object { $portNumbers -contains [int]$_.LocalPort -or $portNumbers -contains [int]$_.RemotePort } |
            Select-Object LocalAddress, LocalPort, RemoteAddress, RemotePort, State, OwningProcess, CreationTime
    )
}
$portState | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $OutputDirectory "port-state.json") -Encoding utf8

[pscustomobject]@{
    process_tree_file = Join-Path $OutputDirectory "process-tree.json"
    port_state_file = Join-Path $OutputDirectory "port-state.json"
    process_count = $processTree.Count
    port_record_count = $portState.Count
} | ConvertTo-Json -Depth 4 -Compress
