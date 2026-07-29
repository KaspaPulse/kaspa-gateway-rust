param(
    [Parameter(Mandatory)]
    [ValidateSet("read", "write")]
    [string]$Mode,

    [AllowEmptyString()]
    [string]$Value = "",

    [AllowEmptyString()]
    [string]$OutputPath = ""
)

if ($PSVersionTable.PSVersion.Major -lt 7) {
    throw "Kaspa Gateway zero-touch E2E helper scripts require PowerShell 7 or later. Launch with (Get-Command pwsh -ErrorAction Stop).Source; Windows PowerShell $($PSVersionTable.PSVersion) is not supported."
}

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Get-Utf8NoBomEncoding {
    return New-Object System.Text.UTF8Encoding -ArgumentList $false
}

function Get-Sha256 {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Text)
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($Text)
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
        $hashBytes = $sha.ComputeHash($bytes)
        return -join ($hashBytes | ForEach-Object { $_.ToString("x2") })
    }
    finally {
        $sha.Dispose()
    }
}

function Get-LineCount {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Text)
    if ([string]::IsNullOrEmpty($Text)) {
        return 0
    }
    return $Text.Replace("`r`n", "`n").Replace("`r", "`n").Split("`n").Count
}

function Invoke-ClipboardOperation {
    param(
        [Parameter(Mandatory)][scriptblock]$Operation,
        [Parameter(Mandatory)][string]$Label
    )

    $lastError = $null
    for ($attempt = 1; $attempt -le 20; $attempt++) {
        try {
            return & $Operation
        }
        catch {
            $lastError = $_.Exception.Message
            Start-Sleep -Milliseconds (50 * $attempt)
        }
    }

    throw "$Label failed after retries: $lastError"
}

if ($Mode -eq "write") {
    Invoke-ClipboardOperation -Label "Set-Clipboard" -Operation {
        Set-Clipboard -Value $Value
    } | Out-Null
    [pscustomobject]@{
        mode = "write"
        character_count = $Value.Length
        line_count = Get-LineCount -Text $Value
        sha256 = Get-Sha256 -Text $Value
    } | ConvertTo-Json -Depth 4 -Compress
    exit 0
}

$text = Invoke-ClipboardOperation -Label "Get-Clipboard" -Operation {
    Get-Clipboard -Raw -ErrorAction Stop
}
if ($null -eq $text) {
    $text = ""
}

if (-not [string]::IsNullOrWhiteSpace($OutputPath)) {
    $directory = Split-Path -Parent $OutputPath
    if (-not [string]::IsNullOrWhiteSpace($directory)) {
        New-Item -ItemType Directory -Path $directory -Force | Out-Null
    }
    [System.IO.File]::WriteAllText($OutputPath, [string]$text, (Get-Utf8NoBomEncoding))
}

[pscustomobject]@{
    mode = "read"
    output_path = $OutputPath
    character_count = ([string]$text).Length
    line_count = Get-LineCount -Text ([string]$text)
    sha256 = Get-Sha256 -Text ([string]$text)
} | ConvertTo-Json -Depth 4 -Compress
