[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Executable,
    [string]$DumpbinPath,
    [string]$ReportPath
)
$ErrorActionPreference = "Stop"
$Executable = (Resolve-Path -LiteralPath $Executable -ErrorAction Stop).Path
if (-not $DumpbinPath) {
    $available = Get-Command dumpbin.exe -ErrorAction SilentlyContinue
    if ($available) { $DumpbinPath = $available.Source }
    else {
        $vswhere = Join-Path ([Environment]::GetFolderPath("ProgramFilesX86")) "Microsoft Visual Studio\Installer\vswhere.exe"
        if (-not (Test-Path -LiteralPath $vswhere -PathType Leaf)) { throw "dumpbin.exe and vswhere.exe are unavailable." }
        $DumpbinPath = & $vswhere -latest -products "*" -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -find "VC\Tools\MSVC\**\bin\Hostx64\x64\dumpbin.exe" | Select-Object -First 1
    }
}
if (-not $DumpbinPath -or -not (Test-Path -LiteralPath $DumpbinPath -PathType Leaf)) { throw "Cannot inspect PE imports without dumpbin.exe." }
$output = @(& $DumpbinPath /DEPENDENTS $Executable 2>&1)
if ($LASTEXITCODE -ne 0) { throw "PE dependency inspection failed: $($output -join [Environment]::NewLine)" }
$imports = @($output | ForEach-Object { if ([string]$_ -match '^\s+([^\s]+\.dll)\s*$') { $Matches[1].ToUpperInvariant() } } | Sort-Object -Unique)
if ($imports.Count -eq 0) { throw "No PE imports were parsed; refusing to report a pass." }
$runtimeDlls = @($imports | Where-Object { $_ -match '^(?:MSVCP|MSVCR|VCRUNTIME|CONCRT)[0-9].*\.DLL$' })
$report = [ordered]@{
    executable = $Executable
    sha256 = (Get-FileHash -LiteralPath $Executable -Algorithm SHA256).Hash.ToLowerInvariant()
    checkedAtUtc = (Get-Date).ToUniversalTime().ToString("o")
    importedDlls = $imports
    externalMsvcRuntimeDlls = $runtimeDlls
    passed = ($runtimeDlls.Count -eq 0)
}
if ($ReportPath) { $report | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $ReportPath -Encoding UTF8 }
if ($runtimeDlls.Count -ne 0) { throw "Windows artifact requires separately installed MSVC runtime DLLs: $($runtimeDlls -join ', ')" }
$report | ConvertTo-Json -Depth 5
