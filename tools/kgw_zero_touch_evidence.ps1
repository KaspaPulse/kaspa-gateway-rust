Set-StrictMode -Version Latest

function Assert-KgwZeroTouchPowerShell7 {
    param([int]$MajorVersion = $PSVersionTable.PSVersion.Major)

    if ($MajorVersion -lt 7) {
        throw "Kaspa Gateway zero-touch E2E requires PowerShell 7 or later. Resolve PowerShell with (Get-Command pwsh -ErrorAction Stop).Source; Windows PowerShell $MajorVersion is not supported."
    }
}

Assert-KgwZeroTouchPowerShell7

function Get-KgwZeroTouchPwshPath {
    return (Get-Command pwsh -ErrorAction Stop).Source
}

function Get-KgwZeroTouchUtf8NoBomEncoding {
    return [System.Text.UTF8Encoding]::new($false)
}

function Get-KgwZeroTouchSha256ForText {
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

function Add-KgwZeroTouchUtf8TextToStream {
    param(
        [Parameter(Mandatory)][System.IO.Stream]$Stream,
        [Parameter(Mandatory)][AllowEmptyString()][string]$Text
    )

    $bytes = [System.Text.Encoding]::UTF8.GetBytes($Text)
    $Stream.Write($bytes, 0, $bytes.Length)
}

function Add-KgwZeroTouchFileBytesToStream {
    param(
        [Parameter(Mandatory)][System.IO.Stream]$Stream,
        [Parameter(Mandatory)][string]$Path
    )

    $file = [System.IO.File]::Open($Path, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::ReadWrite)
    try {
        $buffer = New-Object byte[] 81920
        while ($true) {
            $read = $file.Read($buffer, 0, $buffer.Length)
            if ($read -le 0) {
                break
            }
            $Stream.Write($buffer, 0, $read)
        }
    }
    finally {
        $file.Dispose()
    }
}

function Get-KgwZeroTouchSha256ForStream {
    param([Parameter(Mandatory)][System.IO.Stream]$Stream)

    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
        $Stream.Position = 0
        $hashBytes = $sha.ComputeHash($Stream)
        return -join ($hashBytes | ForEach-Object { $_.ToString("x2") })
    }
    finally {
        $sha.Dispose()
    }
}

function Get-KgwZeroTouchSha256ForFile {
    param([Parameter(Mandatory)][string]$Path)

    $stream = [System.IO.File]::Open($Path, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::ReadWrite)
    try {
        return Get-KgwZeroTouchSha256ForStream -Stream $stream
    }
    finally {
        $stream.Dispose()
    }
}

function Get-KgwZeroTouchCurrentGitCommit {
    param([Parameter(Mandatory)][string]$Repository)

    $commit = & git -C $Repository rev-parse HEAD
    if ($LASTEXITCODE -ne 0) {
        throw "git rev-parse HEAD failed with exit code $LASTEXITCODE"
    }

    return ($commit -join "`n").Trim()
}

function Get-KgwZeroTouchSourceDiffSha256 {
    param([Parameter(Mandatory)][string]$Repository)

    $diffPath = Join-Path ([System.IO.Path]::GetTempPath()) ("kgw-zero-touch-source-diff-{0}.patch" -f [System.Guid]::NewGuid().ToString("N"))
    $stream = New-Object System.IO.MemoryStream
    try {
        & git -C $Repository diff --binary --no-ext-diff "--output=$diffPath" -- . ":(exclude)artifacts" ":(exclude)graphify-out" ":(exclude)e2e/node_modules"
        if ($LASTEXITCODE -ne 0) {
            throw "git diff failed with exit code $LASTEXITCODE"
        }

        $untracked = @(
            & git -C $Repository ls-files --others --exclude-standard -- . ":(exclude)artifacts" ":(exclude)graphify-out" ":(exclude)e2e/node_modules"
        )
        if ($LASTEXITCODE -ne 0) {
            throw "git ls-files failed with exit code $LASTEXITCODE"
        }

        Add-KgwZeroTouchUtf8TextToStream -Stream $stream -Text "kgw-zero-touch-source-diff-v2`ntracked-diff-bytes`n"
        if (Test-Path -LiteralPath $diffPath -PathType Leaf) {
            Add-KgwZeroTouchFileBytesToStream -Stream $stream -Path $diffPath
        }

        Add-KgwZeroTouchUtf8TextToStream -Stream $stream -Text "`nuntracked-files`n"
        $relativePaths = New-Object System.Collections.Generic.List[string]
        foreach ($relativePath in @($untracked)) {
            if (-not [string]::IsNullOrWhiteSpace($relativePath)) {
                [void]$relativePaths.Add([string]$relativePath)
            }
        }

        $sortedPaths = $relativePaths.ToArray()
        [System.Array]::Sort($sortedPaths, [System.StringComparer]::Ordinal)
        foreach ($relativePath in $sortedPaths) {
            $fullPath = Join-Path $Repository $relativePath
            if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf)) {
                continue
            }

            $item = Get-Item -LiteralPath $fullPath
            $hash = Get-KgwZeroTouchSha256ForFile -Path $fullPath
            Add-KgwZeroTouchUtf8TextToStream -Stream $stream -Text ("{0}`t{1}`t{2}`n" -f $relativePath, $item.Length, $hash)
        }

        return Get-KgwZeroTouchSha256ForStream -Stream $stream
    }
    finally {
        $stream.Dispose()
        if (Test-Path -LiteralPath $diffPath -PathType Leaf) {
            Remove-Item -LiteralPath $diffPath -Force
        }
    }
}

function Read-KgwZeroTouchJsonFile {
    param([Parameter(Mandatory)][string]$Path)

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        return $null
    }

    $text = Get-Content -LiteralPath $Path -Raw
    if ([string]::IsNullOrWhiteSpace($text)) {
        return $null
    }

    try {
        return $text | ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        throw "Invalid JSON in ${Path}: $($_.Exception.Message)"
    }
}

function Get-KgwZeroTouchProperty {
    param(
        [AllowNull()]$Object,
        [Parameter(Mandatory)][string[]]$Names
    )

    if ($null -eq $Object) {
        return $null
    }

    if ($Object -is [System.Collections.IDictionary]) {
        foreach ($name in $Names) {
            if ($Object.Contains($name)) {
                return $Object[$name]
            }
        }
        return $null
    }

    foreach ($name in $Names) {
        $property = $Object.PSObject.Properties[$name]
        if ($null -ne $property) {
            return $property.Value
        }
    }

    return $null
}

function ConvertTo-KgwZeroTouchIntegerOrNull {
    param([AllowNull()]$Value)

    if ($null -eq $Value) {
        return $null
    }
    if ($Value -is [byte] -or
        $Value -is [int16] -or
        $Value -is [int] -or
        $Value -is [int64] -or
        $Value -is [uint16] -or
        $Value -is [uint32]) {
        return [int64]$Value
    }
    $text = [string]$Value
    if ($text -match '^\d+$') {
        return [int64]$text
    }

    return $null
}

function ConvertTo-KgwZeroTouchJsonSafeValue {
    param(
        [AllowNull()][object]$Value,
        [string]$Path = "$"
    )

    if ($null -eq $Value) {
        return $null
    }
    if ($Value -is [string]) {
        return [string]$Value
    }
    if ($Value -is [bool]) {
        return [bool]$Value
    }
    if ($Value -is [byte] -or
        $Value -is [int16] -or
        $Value -is [int] -or
        $Value -is [int64] -or
        $Value -is [uint16] -or
        $Value -is [uint32]) {
        return [int64]$Value
    }
    if ($Value -is [System.Collections.Specialized.OrderedDictionary]) {
        $copy = [ordered]@{}
        foreach ($key in $Value.Keys) {
            if ($null -eq $key -or -not ($key -is [string])) {
                throw "JSON-safe dictionary key at $Path must be a string."
            }
            $copy[$key] = ConvertTo-KgwZeroTouchJsonSafeValue -Value $Value[$key] -Path ("{0}.{1}" -f $Path, $key)
        }
        return $copy
    }
    if ($Value -is [System.Collections.IEnumerable]) {
        $items = New-Object System.Collections.Generic.List[object]
        $index = 0
        foreach ($item in $Value) {
            [void]$items.Add((ConvertTo-KgwZeroTouchJsonSafeValue -Value $item -Path ("{0}[{1}]" -f $Path, $index)))
            $index++
        }
        return ,$items.ToArray()
    }

    throw "JSON-safe value expected at ${Path}; got $($Value.GetType().FullName)."
}

function ConvertTo-KgwZeroTouchSerializableException {
    param([AllowNull()]$Value)

    $exception = $null
    if ($Value -is [System.Management.Automation.ErrorRecord]) {
        $exception = $Value.Exception
    } elseif ($Value -is [System.Exception]) {
        $exception = $Value
    }

    if ($null -eq $exception) {
        return [ordered]@{
            message = [string]$Value
            type = if ($null -eq $Value) { "" } else { $Value.GetType().FullName }
            stack = ""
        }
    }

    return [ordered]@{
        message = [string]$exception.Message
        type = [string]$exception.GetType().FullName
        stack = [string]$exception.StackTrace
    }
}

function Write-KgwZeroTouchJsonFile {
    param(
        [AllowNull()][object]$Value = $null,
        [Parameter(Mandatory)][string]$Path,
        [int]$Depth = 16
    )

    $directory = Split-Path -Parent $Path
    if (-not [string]::IsNullOrWhiteSpace($directory)) {
        New-Item -ItemType Directory -Path $directory -Force | Out-Null
    } else {
        $directory = "."
    }

    $safeValue = ConvertTo-KgwZeroTouchJsonSafeValue -Value $Value
    $json = if ($null -eq $safeValue) {
        "null"
    } else {
        ConvertTo-Json -InputObject $safeValue -Depth $Depth -ErrorAction Stop
    }

    $tempPath = Join-Path $directory (".{0}.{1}.tmp" -f ([System.IO.Path]::GetFileName($Path)), [System.Guid]::NewGuid().ToString("N"))
    $backupPath = Join-Path $directory (".{0}.{1}.bak" -f ([System.IO.Path]::GetFileName($Path)), [System.Guid]::NewGuid().ToString("N"))
    $writer = $null
    try {
        $writer = [System.IO.StreamWriter]::new($tempPath, $false, (Get-KgwZeroTouchUtf8NoBomEncoding))
        $writer.Write($json)
        $writer.Flush()
        $writer.Dispose()
        $writer = $null

        if (Test-Path -LiteralPath $Path -PathType Leaf) {
            [System.IO.File]::Replace($tempPath, $Path, $backupPath)
            if (Test-Path -LiteralPath $backupPath -PathType Leaf) {
                Remove-Item -LiteralPath $backupPath -Force -ErrorAction SilentlyContinue
            }
        } else {
            [System.IO.File]::Move($tempPath, $Path)
        }
    }
    finally {
        if ($null -ne $writer) {
            $writer.Dispose()
        }
        if (Test-Path -LiteralPath $tempPath -PathType Leaf) {
            Remove-Item -LiteralPath $tempPath -Force -ErrorAction SilentlyContinue
        }
        if (Test-Path -LiteralPath $backupPath -PathType Leaf) {
            Remove-Item -LiteralPath $backupPath -Force -ErrorAction SilentlyContinue
        }
    }
}

function Write-KgwZeroTouchEmergencyJsonFile {
    param(
        [Parameter(Mandatory)][System.Collections.Specialized.OrderedDictionary]$Value,
        [Parameter(Mandatory)][string]$Path,
        [int]$Depth = 16
    )

    $directory = Split-Path -Parent $Path
    if (-not [string]::IsNullOrWhiteSpace($directory)) {
        New-Item -ItemType Directory -Path $directory -Force | Out-Null
    }

    $json = ConvertTo-Json -InputObject (ConvertTo-KgwZeroTouchJsonSafeValue -Value $Value) -Depth $Depth -ErrorAction Stop
    [System.IO.File]::WriteAllText($Path, $json, (Get-KgwZeroTouchUtf8NoBomEncoding))
}

function New-KgwZeroTouchWriterFailureResultObject {
    param(
        [Parameter(Mandatory)][string]$Repository,
        [Parameter(Mandatory)][string]$ArtifactDirectory,
        [Parameter(Mandatory)][string]$StartedAt,
        [Parameter(Mandatory)][int]$OriginalExitCode,
        [AllowNull()][string]$OriginalFailedStage,
        [AllowNull()][string]$ExecutablePath,
        [Parameter(Mandatory)]$WriterError,
        [string[]]$ValidationErrors = @()
    )

    $writerException = ConvertTo-KgwZeroTouchSerializableException -Value $WriterError
    $errors = New-Object System.Collections.Generic.List[string]
    foreach ($error in @($ValidationErrors)) {
        if (-not [string]::IsNullOrWhiteSpace([string]$error)) {
            [void]$errors.Add([string]$error)
        }
    }
    [void]$errors.Add("Failed to write full zero-touch result: $($writerException.message)")

    return [ordered]@{
        completed = $true
        success = $false
        exit_code = [int]$OriginalExitCode
        original_exit_code = [int]$OriginalExitCode
        started_at = [string]$StartedAt
        completed_at = (Get-Date).ToUniversalTime().ToString("o")
        git_commit = $null
        source_diff_sha256 = $null
        executable_sha256 = $null
        passed_stages = @()
        failed_stage = "Zero-touch result writing"
        original_failure_stage = if ([string]::IsNullOrWhiteSpace($OriginalFailedStage)) { $null } else { [string]$OriginalFailedStage }
        result_writer_error = $writerException
        clipboard_hashes = @()
        process_ids = @()
        ports = @()
        artifact_directory = [System.IO.Path]::GetFullPath($ArtifactDirectory)
        app_binary = [string]$ExecutablePath
        validation_errors = $errors.ToArray()
    }
}

function Get-KgwZeroTouchRequiredStages {
    return @(
        [pscustomobject]@{
            Name = "Mainnet Node"
            Slug = "mainnet-node"
            Network = "mainnet"
            RuntimeRole = "node"
            OwnerStatusFile = "node-owner-status.json"
            RequiredPorts = @(16110, 16111)
        },
        [pscustomobject]@{
            Name = "Testnet10 Node"
            Slug = "testnet10-node"
            Network = "testnet10"
            RuntimeRole = "node"
            OwnerStatusFile = "node-owner-status.json"
            RequiredPorts = @(16210, 16211)
        },
        [pscustomobject]@{
            Name = "Mainnet Bridge"
            Slug = "mainnet-bridge"
            Network = "mainnet"
            RuntimeRole = "bridge"
            OwnerStatusFile = "bridge-owner-status.json"
            RequiredPorts = @(5556)
        },
        [pscustomobject]@{
            Name = "Testnet10 Bridge"
            Slug = "testnet10-bridge"
            Network = "testnet10"
            RuntimeRole = "bridge"
            OwnerStatusFile = "bridge-owner-status.json"
            RequiredPorts = @(5656)
        }
    )
}

function Test-KgwZeroTouchTextHasTransportWrapper {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Text)

    if ([string]::IsNullOrEmpty($Text)) {
        return $false
    }

    $markers = @(
        "kgw_raw_process_log_v1",
        "[KGW_CHILD_STDOUT]",
        "[KGW_CHILD_STDERR]",
        "diagnostic_transport_record",
        ";source=self-worker;",
        ";runtime_role=",
        ";received_ms="
    )

    foreach ($marker in $markers) {
        if ($Text.IndexOf($marker, [System.StringComparison]::OrdinalIgnoreCase) -ge 0) {
            return $true
        }
    }

    return $false
}

function Get-KgwZeroTouchStatusPid {
    param([AllowNull()]$Status)

    $pidValue = Get-KgwZeroTouchProperty -Object $Status -Names @("pid")
    if ($pidValue -match '^\d+$') {
        return [int]$pidValue
    }

    $statusText = [string](Get-KgwZeroTouchProperty -Object $Status -Names @("status"))
    if ($statusText -match '(?i)(^|;)pid=(\d+)(;|$)') {
        return [int]$Matches[2]
    }

    return $null
}

function Test-KgwZeroTouchPortEvidence {
    param(
        [AllowNull()]$PortState,
        [Parameter(Mandatory)][int]$Port
    )

    foreach ($entry in @($PortState)) {
        if ($null -eq $entry) {
            continue
        }

        $localPort = Get-KgwZeroTouchProperty -Object $entry -Names @("LocalPort", "localPort", "local_port")
        $remotePort = Get-KgwZeroTouchProperty -Object $entry -Names @("RemotePort", "remotePort", "remote_port")
        if (($localPort -match '^\d+$' -and [int]$localPort -eq $Port) -or
            ($remotePort -match '^\d+$' -and [int]$remotePort -eq $Port)) {
            return $true
        }
    }

    return $false
}

function Get-KgwZeroTouchStageEvidence {
    param(
        [Parameter(Mandatory)][string]$ArtifactDirectory,
        [Parameter(Mandatory)]$Stage
    )

    $errors = New-Object System.Collections.Generic.List[string]
    $warnings = New-Object System.Collections.Generic.List[string]
    $caseDirectory = Join-Path $ArtifactDirectory ("cases/{0}" -f $Stage.Slug)
    $ownerStatusPath = Join-Path $caseDirectory $Stage.OwnerStatusFile
    $runtimeLogsPath = Join-Path $caseDirectory "runtime-logs.json"
    $clipboardCapturePath = Join-Path $caseDirectory "clipboard-capture.json"
    $clipboardRawPath = Join-Path $caseDirectory "clipboard.raw.txt"
    $processTreePath = Join-Path $caseDirectory "process-tree.json"
    $portStatePath = Join-Path $caseDirectory "port-state.json"
    $screenshotPath = Join-Path $caseDirectory "current-screenshot.png"
    $selectionPath = Join-Path $caseDirectory "bridge-runtime-selection.json"

    if (-not (Test-Path -LiteralPath $caseDirectory -PathType Container)) {
        $errors.Add("Missing case directory for $($Stage.Name): $caseDirectory")
    }

    $ownerStatus = Read-KgwZeroTouchJsonFile -Path $ownerStatusPath
    $runtimeLogs = Read-KgwZeroTouchJsonFile -Path $runtimeLogsPath
    $clipboardCapture = Read-KgwZeroTouchJsonFile -Path $clipboardCapturePath
    $processTree = Read-KgwZeroTouchJsonFile -Path $processTreePath
    $portState = Read-KgwZeroTouchJsonFile -Path $portStatePath
    $selection = Read-KgwZeroTouchJsonFile -Path $selectionPath

    $processIdValue = Get-KgwZeroTouchStatusPid -Status $ownerStatus
    if ($null -eq $processIdValue -or $processIdValue -le 0) {
        $errors.Add("$($Stage.Name) did not record a real child PID in $($Stage.OwnerStatusFile).")
    } else {
        $foundPid = $false
        foreach ($process in @($processTree)) {
            if ($null -eq $process) {
                continue
            }
            $processId = Get-KgwZeroTouchProperty -Object $process -Names @("ProcessId", "processId", "process_id")
            if ($processId -match '^\d+$' -and [int]$processId -eq $processIdValue) {
                $foundPid = $true
                break
            }
        }
        if (-not $foundPid) {
            $errors.Add("$($Stage.Name) PID $processIdValue was not present in process-tree.json.")
        }
    }

    $requiredPorts = @($Stage.RequiredPorts)
    if ($Stage.RuntimeRole -eq "bridge") {
        $selectedPort = Get-KgwZeroTouchProperty -Object $selection -Names @("runtimeBridgePort", "runtime_bridge_port", "listeningBridgePort", "listening_bridge_port", "bridgePort", "bridge_port")
        if ($selectedPort -match '^\d+$' -and [int]$selectedPort -gt 0) {
            $requiredPorts = @([int]$selectedPort)
        }
    }

    $stagePorts = New-Object System.Collections.Generic.List[object]
    foreach ($port in @($requiredPorts)) {
        $portNumber = [int]$port
        [void]$stagePorts.Add([ordered]@{
            network = $Stage.Network
            runtime_role = $Stage.RuntimeRole
            host = "127.0.0.1"
            port = $portNumber
        })
        if (-not (Test-KgwZeroTouchPortEvidence -PortState $portState -Port $portNumber)) {
            $errors.Add("$($Stage.Name) did not capture TCP port evidence for port $portNumber.")
        }
    }

    if (-not (Test-Path -LiteralPath $screenshotPath -PathType Leaf)) {
        $errors.Add("$($Stage.Name) did not capture a screenshot.")
    }

    $entries = @()
    if ($null -ne $runtimeLogs) {
        $entriesValue = Get-KgwZeroTouchProperty -Object $runtimeLogs -Names @("entries")
        $entries = @($entriesValue)
    }
    if ($entries.Count -eq 0 -or ($entries.Count -eq 1 -and $null -eq $entries[0])) {
        $errors.Add("$($Stage.Name) runtime-logs.json did not contain raw entries.")
    }

    foreach ($entry in $entries) {
        if ($null -eq $entry) {
            continue
        }
        $entryNetwork = [string](Get-KgwZeroTouchProperty -Object $entry -Names @("network"))
        $entryRole = [string](Get-KgwZeroTouchProperty -Object $entry -Names @("runtimeRole", "runtime_role"))
        $entryRawText = [string](Get-KgwZeroTouchProperty -Object $entry -Names @("rawText", "raw_text"))
        if ($entryNetwork.ToLowerInvariant() -ne $Stage.Network) {
            $errors.Add("$($Stage.Name) runtime log entry used network '$entryNetwork'.")
            break
        }
        if ($entryRole.ToLowerInvariant() -ne $Stage.RuntimeRole) {
            $errors.Add("$($Stage.Name) runtime log entry used role '$entryRole'.")
            break
        }
        if (Test-KgwZeroTouchTextHasTransportWrapper -Text $entryRawText) {
            $errors.Add("$($Stage.Name) runtime log entry contained transport wrapper text.")
            break
        }
    }

    $clipboardHash = $null
    $bridgeInstanceId = ""
    if ($null -eq $clipboardCapture) {
        $errors.Add("$($Stage.Name) did not write clipboard-capture.json.")
    } else {
        $event = Get-KgwZeroTouchProperty -Object $clipboardCapture -Names @("event")
        $clipboard = Get-KgwZeroTouchProperty -Object $clipboardCapture -Names @("clipboard")
        $eventNetwork = [string](Get-KgwZeroTouchProperty -Object $event -Names @("network"))
        $eventRole = [string](Get-KgwZeroTouchProperty -Object $event -Names @("runtimeRole", "runtime_role"))
        $bridgeInstanceId = [string](Get-KgwZeroTouchProperty -Object $event -Names @("bridgeInstanceId", "bridge_instance_id"))
        $expectedSha = [string](Get-KgwZeroTouchProperty -Object $event -Names @("expectedSha256", "expected_sha256"))
        $resultSha = [string](Get-KgwZeroTouchProperty -Object $event -Names @("resultSha256", "result_sha256"))
        $resultSource = [string](Get-KgwZeroTouchProperty -Object $event -Names @("resultSource", "result_source", "status"))
        $actualSha = [string](Get-KgwZeroTouchProperty -Object $clipboard -Names @("sha256", "actual_sha256"))

        if ($eventNetwork.ToLowerInvariant() -ne $Stage.Network) {
            $errors.Add("$($Stage.Name) clipboard event network '$eventNetwork' did not match $($Stage.Network).")
        }
        if ($eventRole.ToLowerInvariant() -ne $Stage.RuntimeRole) {
            $errors.Add("$($Stage.Name) clipboard event role '$eventRole' did not match $($Stage.RuntimeRole).")
        }
        if ($Stage.RuntimeRole -eq "bridge" -and [string]::IsNullOrWhiteSpace($bridgeInstanceId)) {
            $errors.Add("$($Stage.Name) clipboard event did not include a bridge instance.")
        }
        if ([string]::IsNullOrWhiteSpace($expectedSha) -or [string]::IsNullOrWhiteSpace($resultSha) -or [string]::IsNullOrWhiteSpace($actualSha)) {
            $errors.Add("$($Stage.Name) clipboard hashes were incomplete.")
        } elseif (-not $expectedSha.Equals($resultSha, [System.StringComparison]::OrdinalIgnoreCase) -or
            -not $expectedSha.Equals($actualSha, [System.StringComparison]::OrdinalIgnoreCase)) {
            $errors.Add("$($Stage.Name) clipboard expected/native/Windows SHA256 values did not match.")
        }

        if (Test-Path -LiteralPath $clipboardRawPath -PathType Leaf) {
            $rawText = [System.IO.File]::ReadAllText($clipboardRawPath, [System.Text.Encoding]::UTF8)
            $rawSha = Get-KgwZeroTouchSha256ForText -Text $rawText
            if (-not [string]::IsNullOrWhiteSpace($actualSha) -and
                -not $rawSha.Equals($actualSha, [System.StringComparison]::OrdinalIgnoreCase)) {
                $errors.Add("$($Stage.Name) clipboard.raw.txt SHA256 did not match clipboard-capture.json.")
            }
            if (Test-KgwZeroTouchTextHasTransportWrapper -Text $rawText) {
                $errors.Add("$($Stage.Name) clipboard.raw.txt contained transport wrapper text.")
            }
        } else {
            $errors.Add("$($Stage.Name) did not write clipboard.raw.txt.")
        }

        $clipboardHash = [ordered]@{
            stage = $Stage.Name
            network = $Stage.Network
            runtime_role = $Stage.RuntimeRole
            bridge_instance_id = $bridgeInstanceId
            expected_sha256 = $expectedSha
            result_sha256 = $resultSha
            result_source = $resultSource
            windows_clipboard_sha256 = $actualSha
            raw_file = $clipboardRawPath
        }
    }

    return [pscustomobject]@{
        name = $Stage.Name
        slug = $Stage.Slug
        network = $Stage.Network
        runtime_role = $Stage.RuntimeRole
        bridge_instance_id = $bridgeInstanceId
        passed = ($errors.Count -eq 0)
        errors = $errors.ToArray()
        warnings = $warnings.ToArray()
        process_id = $processIdValue
        ports = $stagePorts.ToArray()
        clipboard_hash = $clipboardHash
        evidence_files = [ordered]@{
            owner_status = $ownerStatusPath
            runtime_logs = $runtimeLogsPath
            clipboard_capture = $clipboardCapturePath
            clipboard_raw = $clipboardRawPath
            process_tree = $processTreePath
            port_state = $portStatePath
            screenshot = $screenshotPath
        }
    }
}

function Get-KgwZeroTouchEvidenceSummary {
    param([Parameter(Mandatory)][string]$ArtifactDirectory)

    $artifactDirectory = [System.IO.Path]::GetFullPath($ArtifactDirectory)
    $errors = New-Object System.Collections.Generic.List[string]
    $warnings = New-Object System.Collections.Generic.List[string]
    $stageReports = New-Object System.Collections.Generic.List[object]
    $clipboardHashes = New-Object System.Collections.Generic.List[object]
    $processIds = New-Object System.Collections.Generic.List[object]
    $ports = New-Object System.Collections.Generic.List[object]

    $observationsPath = Join-Path $artifactDirectory "zero-touch-observations.json"
    $observations = Read-KgwZeroTouchJsonFile -Path $observationsPath
    if ($null -eq $observations) {
        $errors.Add("Missing zero-touch-observations.json.")
    } else {
        foreach ($pidEntry in @((Get-KgwZeroTouchProperty -Object $observations -Names @("pids")))) {
            if ($null -eq $pidEntry) {
                continue
            }
            $pidValue = ConvertTo-KgwZeroTouchIntegerOrNull -Value (Get-KgwZeroTouchProperty -Object $pidEntry -Names @("pid"))
            if ($null -eq $pidValue) {
                continue
            }
            [void]$processIds.Add([ordered]@{
                network = [string](Get-KgwZeroTouchProperty -Object $pidEntry -Names @("network"))
                runtime_role = [string](Get-KgwZeroTouchProperty -Object $pidEntry -Names @("runtimeRole", "runtime_role"))
                pid = $pidValue
            })
        }
        foreach ($portEntry in @((Get-KgwZeroTouchProperty -Object $observations -Names @("ports")))) {
            if ($null -eq $portEntry) {
                continue
            }
            $portValue = ConvertTo-KgwZeroTouchIntegerOrNull -Value (Get-KgwZeroTouchProperty -Object $portEntry -Names @("port"))
            if ($null -eq $portValue) {
                continue
            }
            [void]$ports.Add([ordered]@{
                network = [string](Get-KgwZeroTouchProperty -Object $portEntry -Names @("network"))
                runtime_role = [string](Get-KgwZeroTouchProperty -Object $portEntry -Names @("runtimeRole", "runtime_role"))
                host = [string](Get-KgwZeroTouchProperty -Object $portEntry -Names @("host"))
                port = $portValue
                purpose = [string](Get-KgwZeroTouchProperty -Object $portEntry -Names @("purpose"))
            })
        }
    }

    foreach ($stage in Get-KgwZeroTouchRequiredStages) {
        $stageReport = Get-KgwZeroTouchStageEvidence -ArtifactDirectory $artifactDirectory -Stage $stage
        [void]$stageReports.Add($stageReport)
        if (-not $stageReport.passed) {
            foreach ($error in $stageReport.errors) {
                $errors.Add($error)
            }
        }
        foreach ($warning in $stageReport.warnings) {
            $warnings.Add($warning)
        }
        if ($null -ne $stageReport.clipboard_hash) {
            [void]$clipboardHashes.Add($stageReport.clipboard_hash)
        }
        if ($null -ne $stageReport.process_id -and $stageReport.process_id -gt 0) {
            [void]$processIds.Add([ordered]@{
                network = $stageReport.network
                runtime_role = $stageReport.runtime_role
                bridge_instance_id = $stageReport.bridge_instance_id
                pid = [int64]$stageReport.process_id
            })
        }
        foreach ($port in @($stageReport.ports)) {
            [void]$ports.Add($port)
        }
    }

    $stageReportArray = $stageReports.ToArray()
    $passedStages = @($stageReportArray | Where-Object { $_.passed } | ForEach-Object { $_.name })
    $failedStage = $null
    foreach ($stageReport in $stageReportArray) {
        if (-not $stageReport.passed) {
            $failedStage = $stageReport.name
            break
        }
    }

    return [pscustomobject]@{
        passed = ($errors.Count -eq 0)
        validation_errors = $errors.ToArray()
        warnings = $warnings.ToArray()
        passed_stages = @($passedStages)
        failed_stage = $failedStage
        clipboard_hashes = $clipboardHashes.ToArray()
        process_ids = $processIds.ToArray()
        ports = $ports.ToArray()
        stages = $stageReportArray
    }
}

function ConvertTo-KgwZeroTouchClipboardHashRecords {
    param([AllowNull()]$Records)

    $items = New-Object System.Collections.Generic.List[object]
    foreach ($record in @($Records)) {
        if ($null -eq $record) {
            continue
        }
        [void]$items.Add([ordered]@{
            stage = [string](Get-KgwZeroTouchProperty -Object $record -Names @("stage"))
            network = [string](Get-KgwZeroTouchProperty -Object $record -Names @("network"))
            runtime_role = [string](Get-KgwZeroTouchProperty -Object $record -Names @("runtime_role", "runtimeRole"))
            bridge_instance_id = [string](Get-KgwZeroTouchProperty -Object $record -Names @("bridge_instance_id", "bridgeInstanceId"))
            expected_sha256 = [string](Get-KgwZeroTouchProperty -Object $record -Names @("expected_sha256", "expectedSha256"))
            result_sha256 = [string](Get-KgwZeroTouchProperty -Object $record -Names @("result_sha256", "resultSha256"))
            result_source = [string](Get-KgwZeroTouchProperty -Object $record -Names @("result_source", "resultSource"))
            windows_clipboard_sha256 = [string](Get-KgwZeroTouchProperty -Object $record -Names @("windows_clipboard_sha256", "windowsClipboardSha256"))
            raw_file = [string](Get-KgwZeroTouchProperty -Object $record -Names @("raw_file", "rawFile"))
        })
    }

    return $items.ToArray()
}

function ConvertTo-KgwZeroTouchProcessIdRecords {
    param([AllowNull()]$Records)

    $items = New-Object System.Collections.Generic.List[object]
    foreach ($record in @($Records)) {
        if ($null -eq $record) {
            continue
        }
        $pidValue = ConvertTo-KgwZeroTouchIntegerOrNull -Value (Get-KgwZeroTouchProperty -Object $record -Names @("pid"))
        if ($null -eq $pidValue) {
            continue
        }
        [void]$items.Add([ordered]@{
            network = [string](Get-KgwZeroTouchProperty -Object $record -Names @("network"))
            runtime_role = [string](Get-KgwZeroTouchProperty -Object $record -Names @("runtime_role", "runtimeRole"))
            bridge_instance_id = [string](Get-KgwZeroTouchProperty -Object $record -Names @("bridge_instance_id", "bridgeInstanceId"))
            pid = $pidValue
        })
    }

    return $items.ToArray()
}

function ConvertTo-KgwZeroTouchPortRecords {
    param([AllowNull()]$Records)

    $items = New-Object System.Collections.Generic.List[object]
    foreach ($record in @($Records)) {
        if ($null -eq $record) {
            continue
        }
        $portValue = ConvertTo-KgwZeroTouchIntegerOrNull -Value (Get-KgwZeroTouchProperty -Object $record -Names @("port"))
        if ($null -eq $portValue) {
            continue
        }
        [void]$items.Add([ordered]@{
            network = [string](Get-KgwZeroTouchProperty -Object $record -Names @("network"))
            runtime_role = [string](Get-KgwZeroTouchProperty -Object $record -Names @("runtime_role", "runtimeRole"))
            host = [string](Get-KgwZeroTouchProperty -Object $record -Names @("host"))
            port = $portValue
            purpose = [string](Get-KgwZeroTouchProperty -Object $record -Names @("purpose"))
            endpoint = ("{0}:{1}" -f ([string](Get-KgwZeroTouchProperty -Object $record -Names @("host"))), $portValue)
        })
    }

    return $items.ToArray()
}

function Get-KgwZeroTouchWdioEvidenceSummary {
    param([Parameter(Mandatory)][string]$ArtifactDirectory)

    $artifactDirectory = [System.IO.Path]::GetFullPath($ArtifactDirectory)
    $errors = New-Object System.Collections.Generic.List[string]
    $jsonFiles = @(Get-ChildItem -LiteralPath (Join-Path $artifactDirectory "json") -Filter "*.json" -File -ErrorAction SilentlyContinue | Sort-Object FullName)
    $junitFiles = @(Get-ChildItem -LiteralPath (Join-Path $artifactDirectory "junit") -Filter "*.xml" -File -ErrorAction SilentlyContinue | Sort-Object FullName)
    $expectedTestNames = @(
        "Mainnet Node regression copies only real child raw lines",
        "Testnet10 Node copies isolated Testnet10 child raw lines",
        "Mainnet Bridge copies only bridge child raw lines",
        "Testnet10 Bridge copies isolated Testnet10 bridge child raw lines",
        "Testnet12 stays disabled by default and policy blocks zero-touch launch"
    )

    if ($jsonFiles.Count -eq 0) {
        $errors.Add("Missing WebdriverIO JSON report.")
    }
    if ($junitFiles.Count -eq 0) {
        $errors.Add("Missing WebdriverIO JUnit report.")
    }

    $jsonTestNames = New-Object System.Collections.Generic.List[string]
    $jsonPassed = 0
    $jsonFailed = 0
    $jsonSkipped = 0
    $jsonSpecs = New-Object System.Collections.Generic.List[string]
    foreach ($file in $jsonFiles) {
        $report = Read-KgwZeroTouchJsonFile -Path $file.FullName
        foreach ($spec in @((Get-KgwZeroTouchProperty -Object $report -Names @("specs")))) {
            if (-not [string]::IsNullOrWhiteSpace([string]$spec)) {
                [void]$jsonSpecs.Add([string]$spec)
            }
        }
        $state = Get-KgwZeroTouchProperty -Object $report -Names @("state")
        $jsonPassed += [int](ConvertTo-KgwZeroTouchIntegerOrNull -Value (Get-KgwZeroTouchProperty -Object $state -Names @("passed")))
        $jsonFailed += [int](ConvertTo-KgwZeroTouchIntegerOrNull -Value (Get-KgwZeroTouchProperty -Object $state -Names @("failed")))
        $jsonSkipped += [int](ConvertTo-KgwZeroTouchIntegerOrNull -Value (Get-KgwZeroTouchProperty -Object $state -Names @("skipped")))
        foreach ($suite in @((Get-KgwZeroTouchProperty -Object $report -Names @("suites")))) {
            foreach ($test in @((Get-KgwZeroTouchProperty -Object $suite -Names @("tests")))) {
                $name = [string](Get-KgwZeroTouchProperty -Object $test -Names @("name"))
                $stateValue = [string](Get-KgwZeroTouchProperty -Object $test -Names @("state"))
                if (-not [string]::IsNullOrWhiteSpace($name)) {
                    [void]$jsonTestNames.Add($name)
                }
                if ($stateValue -ne "passed") {
                    $errors.Add("WebdriverIO JSON test '$name' state was '$stateValue'.")
                }
            }
        }
    }

    $junitTestNames = New-Object System.Collections.Generic.List[string]
    $junitTests = 0
    $junitFailures = 0
    $junitErrors = 0
    $junitSkipped = 0
    foreach ($file in $junitFiles) {
        try {
            [xml]$xml = Get-Content -LiteralPath $file.FullName -Raw
            foreach ($suite in @($xml.SelectNodes("//testsuite"))) {
                $junitTests += [int]$suite.tests
                $junitFailures += [int]$suite.failures
                $junitErrors += [int]$suite.errors
                $junitSkipped += [int]$suite.skipped
            }
            foreach ($case in @($xml.SelectNodes("//testcase"))) {
                [void]$junitTestNames.Add([string]$case.name)
            }
        }
        catch {
            $errors.Add("Invalid WebdriverIO JUnit report '$($file.FullName)': $($_.Exception.Message)")
        }
    }

    if ($jsonSpecs.Count -ne 1) {
        $errors.Add("Expected exactly one WebdriverIO spec, found $($jsonSpecs.Count).")
    }
    if ($jsonPassed -ne 5 -or $jsonFailed -ne 0 -or $jsonSkipped -ne 0) {
        $errors.Add("WebdriverIO JSON state expected passed=5 failed=0 skipped=0, found passed=$jsonPassed failed=$jsonFailed skipped=$jsonSkipped.")
    }
    if ($junitTests -ne 5 -or $junitFailures -ne 0 -or $junitErrors -ne 0 -or $junitSkipped -ne 0) {
        $errors.Add("WebdriverIO JUnit expected tests=5 failures=0 errors=0 skipped=0, found tests=$junitTests failures=$junitFailures errors=$junitErrors skipped=$junitSkipped.")
    }
    foreach ($expectedName in $expectedTestNames) {
        if ($jsonTestNames -notcontains $expectedName) {
            $errors.Add("WebdriverIO JSON report did not contain expected test '$expectedName'.")
        }
    }

    return [ordered]@{
        passed = ($errors.Count -eq 0)
        errors = $errors.ToArray()
        json_files = @($jsonFiles | ForEach-Object { [string]$_.FullName })
        junit_files = @($junitFiles | ForEach-Object { [string]$_.FullName })
        spec_count = $jsonSpecs.Count
        tests_passed = [int]$jsonPassed
        tests_failed = [int]$jsonFailed
        tests_skipped = [int]$jsonSkipped
        junit_tests = [int]$junitTests
        junit_failures = [int]$junitFailures
        junit_errors = [int]$junitErrors
        junit_skipped = [int]$junitSkipped
        test_names = $jsonTestNames.ToArray()
    }
}

function Get-KgwZeroTouchTestnet12PolicyEvidence {
    param([Parameter(Mandatory)][string]$ArtifactDirectory)

    $artifactDirectory = [System.IO.Path]::GetFullPath($ArtifactDirectory)
    $errors = New-Object System.Collections.Generic.List[string]
    $caseDirectory = Join-Path $artifactDirectory "cases/testnet12-policy"
    $policyPath = Join-Path $caseDirectory "testnet12-policy.json"
    $processTreePath = Join-Path $caseDirectory "process-tree.json"
    $screenshotPath = Join-Path $caseDirectory "current-screenshot.png"
    $domStatePath = Join-Path $caseDirectory "current-dom-state.json"
    $pageSourcePath = Join-Path $caseDirectory "current-page-source.html"
    $policy = Read-KgwZeroTouchJsonFile -Path $policyPath

    if ($null -eq $policy) {
        $errors.Add("Missing Testnet12 policy evidence: $policyPath")
    } else {
        if ([bool](Get-KgwZeroTouchProperty -Object $policy -Names @("nodeEnabled"))) {
            $errors.Add("Testnet12 node policy was enabled; expected disabled by default.")
        }
        if ([bool](Get-KgwZeroTouchProperty -Object $policy -Names @("bridgeEnabled"))) {
            $errors.Add("Testnet12 bridge policy was enabled; expected disabled by default.")
        }
        $startBlocked = Get-KgwZeroTouchProperty -Object $policy -Names @("startBlocked")
        if ([bool](Get-KgwZeroTouchProperty -Object $startBlocked -Names @("ok"))) {
            $errors.Add("Testnet12 start was not blocked.")
        }
        $blockError = [string](Get-KgwZeroTouchProperty -Object $startBlocked -Names @("error"))
        if ($blockError -notmatch '(?i)experimental|opt-in|disabled|policy') {
            $errors.Add("Testnet12 block reason did not mention experimental opt-in policy.")
        }
        $status = [string](Get-KgwZeroTouchProperty -Object $policy -Names @("status"))
        if ($status -match '(?i)pid=\d+' -or $status -match '(?i)running=true') {
            $errors.Add("Testnet12 policy status indicates a runtime started.")
        }
    }
    foreach ($path in @($processTreePath, $screenshotPath, $domStatePath, $pageSourcePath)) {
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
            $errors.Add("Missing Testnet12 policy evidence file: $path")
        }
    }

    return [ordered]@{
        passed = ($errors.Count -eq 0)
        errors = $errors.ToArray()
        evidence_files = @($policyPath, $processTreePath, $screenshotPath, $domStatePath, $pageSourcePath)
    }
}

function Get-KgwZeroTouchRecoveryEvidenceFiles {
    param([Parameter(Mandatory)][string]$ArtifactDirectory)

    $artifactDirectory = [System.IO.Path]::GetFullPath($ArtifactDirectory)
    $files = New-Object System.Collections.Generic.List[string]
    foreach ($relativePath in @(
            "zero-touch-process.stdout.log",
            "zero-touch-process.stderr.log",
            "wdio-run.log",
            "json/wdio-0-0.json",
            "junit/wdio-0-0.xml",
            "zero-touch-observations.json",
            "zero-touch-report.md",
            "zero-touch-script-summary.json",
            "zero-touch-result.json",
            "cases/testnet12-policy/testnet12-policy.json",
            "cases/testnet12-policy/process-tree.json",
            "cases/testnet12-policy/current-screenshot.png"
        )) {
        $path = Join-Path $artifactDirectory $relativePath
        if (Test-Path -LiteralPath $path -PathType Leaf) {
            [void]$files.Add($path)
        }
    }

    foreach ($stage in Get-KgwZeroTouchRequiredStages) {
        $caseDirectory = Join-Path $artifactDirectory ("cases/{0}" -f $stage.Slug)
        foreach ($name in @($stage.OwnerStatusFile, "runtime-logs.json", "clipboard-capture.json", "clipboard.raw.txt", "process-tree.json", "port-state.json", "current-screenshot.png")) {
            $path = Join-Path $caseDirectory $name
            if (Test-Path -LiteralPath $path -PathType Leaf) {
                [void]$files.Add($path)
            }
        }
    }

    return ,$files.ToArray()
}

function New-KgwZeroTouchResultObject {
    param(
        [Parameter(Mandatory)][string]$Repository,
        [Parameter(Mandatory)][string]$ArtifactDirectory,
        [Parameter(Mandatory)][string]$StartedAt,
        [Parameter(Mandatory)][int]$ExitCode,
        [AllowNull()][string]$FailedStage,
        [AllowNull()][string]$ExecutablePath,
        [AllowNull()]$EvidenceSummary,
        [switch]$RecoveredFromCompletedWdioRun,
        [AllowNull()][string]$OriginalFailureStage,
        [AllowNull()][string]$RecoveryTimestamp,
        [string[]]$RecoveryEvidenceFiles = @(),
        [string[]]$AdditionalValidationErrors = @()
    )

    $artifactDirectory = [System.IO.Path]::GetFullPath($ArtifactDirectory)
    $executableSha = $null
    if (-not [string]::IsNullOrWhiteSpace($ExecutablePath) -and (Test-Path -LiteralPath $ExecutablePath -PathType Leaf)) {
        $executableSha = Get-KgwZeroTouchSha256ForFile -Path $ExecutablePath
    }

    if ($null -eq $EvidenceSummary) {
        try {
            $EvidenceSummary = Get-KgwZeroTouchEvidenceSummary -ArtifactDirectory $artifactDirectory
        }
        catch {
            $EvidenceSummary = [pscustomobject]@{
                passed = $false
                validation_errors = @($_.Exception.Message)
                warnings = @()
                passed_stages = @()
                failed_stage = $null
                clipboard_hashes = @()
                process_ids = @()
                ports = @()
                stages = @()
            }
        }
    }

    $effectiveFailedStage = $FailedStage
    if ([string]::IsNullOrWhiteSpace($effectiveFailedStage)) {
        $effectiveFailedStage = [string](Get-KgwZeroTouchProperty -Object $EvidenceSummary -Names @("failed_stage"))
    }

    $validationErrors = New-Object System.Collections.Generic.List[string]
    foreach ($error in @((Get-KgwZeroTouchProperty -Object $EvidenceSummary -Names @("validation_errors")))) {
        if (-not [string]::IsNullOrWhiteSpace([string]$error)) {
            [void]$validationErrors.Add([string]$error)
        }
    }
    foreach ($error in @($AdditionalValidationErrors)) {
        if (-not [string]::IsNullOrWhiteSpace([string]$error)) {
            [void]$validationErrors.Add([string]$error)
        }
    }

    $result = [ordered]@{
        completed = $true
        success = ($ExitCode -eq 0 -and [bool](Get-KgwZeroTouchProperty -Object $EvidenceSummary -Names @("passed")))
        exit_code = [int]$ExitCode
        started_at = [string]$StartedAt
        completed_at = (Get-Date).ToUniversalTime().ToString("o")
        git_commit = Get-KgwZeroTouchCurrentGitCommit -Repository $Repository
        source_diff_sha256 = Get-KgwZeroTouchSourceDiffSha256 -Repository $Repository
        executable_sha256 = $executableSha
        passed_stages = @((Get-KgwZeroTouchProperty -Object $EvidenceSummary -Names @("passed_stages")) | ForEach-Object { [string]$_ })
        failed_stage = $effectiveFailedStage
        clipboard_hashes = @(ConvertTo-KgwZeroTouchClipboardHashRecords -Records (Get-KgwZeroTouchProperty -Object $EvidenceSummary -Names @("clipboard_hashes")))
        process_ids = @(ConvertTo-KgwZeroTouchProcessIdRecords -Records (Get-KgwZeroTouchProperty -Object $EvidenceSummary -Names @("process_ids")))
        ports = @(ConvertTo-KgwZeroTouchPortRecords -Records (Get-KgwZeroTouchProperty -Object $EvidenceSummary -Names @("ports")))
        artifact_directory = $artifactDirectory
        app_binary = [string]$ExecutablePath
        validation_errors = $validationErrors.ToArray()
    }

    if ($RecoveredFromCompletedWdioRun) {
        $result["recovered_from_completed_wdio_run"] = $true
        $result["original_failure_stage"] = [string]$OriginalFailureStage
        $result["recovery_timestamp"] = if ([string]::IsNullOrWhiteSpace($RecoveryTimestamp)) { (Get-Date).ToUniversalTime().ToString("o") } else { [string]$RecoveryTimestamp }
        $result["recovery_evidence_files"] = @($RecoveryEvidenceFiles | ForEach-Object { [string]$_ })
    }

    return $result
}

function Test-KgwZeroTouchResultIntegrity {
    param(
        [Parameter(Mandatory)][string]$Repository,
        [Parameter(Mandatory)][string]$ArtifactDirectory
    )

    $artifactDirectory = [System.IO.Path]::GetFullPath($ArtifactDirectory)
    $resultPath = Join-Path $artifactDirectory "zero-touch-result.json"
    $errors = New-Object System.Collections.Generic.List[string]
    $result = Read-KgwZeroTouchJsonFile -Path $resultPath
    if ($null -eq $result) {
        $errors.Add("Missing zero-touch-result.json.")
        return [pscustomobject]@{ passed = $false; errors = $errors.ToArray(); result = $null; evidence = $null }
    }

    if (-not [bool](Get-KgwZeroTouchProperty -Object $result -Names @("completed"))) {
        $errors.Add("zero-touch-result.json completed is not true.")
    }
    if (-not [bool](Get-KgwZeroTouchProperty -Object $result -Names @("success"))) {
        $errors.Add("zero-touch-result.json success is not true.")
    }
    $exitCode = Get-KgwZeroTouchProperty -Object $result -Names @("exit_code")
    if (-not ($exitCode -match '^\d+$') -or [int]$exitCode -ne 0) {
        $errors.Add("zero-touch-result.json exit_code is not 0.")
    }

    $wdio = Get-KgwZeroTouchWdioEvidenceSummary -ArtifactDirectory $artifactDirectory
    if (-not [bool]$wdio.passed) {
        foreach ($error in @($wdio.errors)) {
            $errors.Add($error)
        }
    }

    $policy = Get-KgwZeroTouchTestnet12PolicyEvidence -ArtifactDirectory $artifactDirectory
    if (-not [bool]$policy.passed) {
        foreach ($error in @($policy.errors)) {
            $errors.Add($error)
        }
    }

    $currentCommit = Get-KgwZeroTouchCurrentGitCommit -Repository $Repository
    $artifactCommit = [string](Get-KgwZeroTouchProperty -Object $result -Names @("git_commit"))
    if ($artifactCommit -ne $currentCommit) {
        $errors.Add("Artifact git commit '$artifactCommit' does not match current commit '$currentCommit'.")
    }

    $currentDiff = Get-KgwZeroTouchSourceDiffSha256 -Repository $Repository
    $artifactDiff = [string](Get-KgwZeroTouchProperty -Object $result -Names @("source_diff_sha256"))
    if ($artifactDiff -ne $currentDiff) {
        $errors.Add("Artifact source diff SHA256 '$artifactDiff' does not match current '$currentDiff'.")
    }

    $appBinary = [string](Get-KgwZeroTouchProperty -Object $result -Names @("app_binary"))
    if ([string]::IsNullOrWhiteSpace($appBinary)) {
        $summary = Read-KgwZeroTouchJsonFile -Path (Join-Path $artifactDirectory "zero-touch-script-summary.json")
        $appBinary = [string](Get-KgwZeroTouchProperty -Object $summary -Names @("app_binary"))
    }
    if ([string]::IsNullOrWhiteSpace($appBinary)) {
        $appBinary = Join-Path $Repository "target/kgw-zero-touch-e2e/debug/kaspa-gateway-desktop.exe"
    }
    if (-not (Test-Path -LiteralPath $appBinary -PathType Leaf)) {
        $errors.Add("E2E desktop executable is missing: $appBinary")
    } else {
        $currentExecutableSha = Get-KgwZeroTouchSha256ForFile -Path $appBinary
        $artifactExecutableSha = [string](Get-KgwZeroTouchProperty -Object $result -Names @("executable_sha256"))
        if ($artifactExecutableSha -ne $currentExecutableSha) {
            $errors.Add("Artifact executable SHA256 '$artifactExecutableSha' does not match current '$currentExecutableSha'.")
        }
    }

    $requiredStageNames = @(Get-KgwZeroTouchRequiredStages | ForEach-Object { $_.Name })
    $passedStages = @((Get-KgwZeroTouchProperty -Object $result -Names @("passed_stages")) | ForEach-Object { [string]$_ })
    foreach ($stageName in $requiredStageNames) {
        if ($passedStages -notcontains $stageName) {
            $errors.Add("Artifact did not record required passed stage '$stageName'.")
        }
    }

    $evidence = Get-KgwZeroTouchEvidenceSummary -ArtifactDirectory $artifactDirectory
    if (-not $evidence.passed) {
        foreach ($error in $evidence.validation_errors) {
            $errors.Add($error)
        }
    }

    $resultClipboardHashes = ConvertTo-KgwZeroTouchClipboardHashRecords -Records (Get-KgwZeroTouchProperty -Object $result -Names @("clipboard_hashes"))
    foreach ($expected in @(ConvertTo-KgwZeroTouchClipboardHashRecords -Records $evidence.clipboard_hashes)) {
        $expectedStage = [string](Get-KgwZeroTouchProperty -Object $expected -Names @("stage"))
        $expectedNetwork = [string](Get-KgwZeroTouchProperty -Object $expected -Names @("network"))
        $expectedRole = [string](Get-KgwZeroTouchProperty -Object $expected -Names @("runtime_role"))
        $expectedBridgeInstanceId = [string](Get-KgwZeroTouchProperty -Object $expected -Names @("bridge_instance_id"))
        $expectedSha = [string](Get-KgwZeroTouchProperty -Object $expected -Names @("expected_sha256"))
        $expectedResultSha = [string](Get-KgwZeroTouchProperty -Object $expected -Names @("result_sha256"))
        $expectedWindowsSha = [string](Get-KgwZeroTouchProperty -Object $expected -Names @("windows_clipboard_sha256"))
        $match = @(
            $resultClipboardHashes |
                Where-Object {
                    [string](Get-KgwZeroTouchProperty -Object $_ -Names @("stage")) -eq $expectedStage -and
                    [string](Get-KgwZeroTouchProperty -Object $_ -Names @("network")) -eq $expectedNetwork -and
                    [string](Get-KgwZeroTouchProperty -Object $_ -Names @("runtime_role")) -eq $expectedRole -and
                    [string](Get-KgwZeroTouchProperty -Object $_ -Names @("bridge_instance_id")) -eq $expectedBridgeInstanceId -and
                    [string](Get-KgwZeroTouchProperty -Object $_ -Names @("expected_sha256")) -eq $expectedSha -and
                    [string](Get-KgwZeroTouchProperty -Object $_ -Names @("result_sha256")) -eq $expectedResultSha -and
                    [string](Get-KgwZeroTouchProperty -Object $_ -Names @("windows_clipboard_sha256")) -eq $expectedWindowsSha
                }
        )
        if ($match.Count -eq 0) {
            $errors.Add("Artifact result did not record validated clipboard hash evidence for '$expectedStage'.")
        }
    }

    $resultProcessIds = ConvertTo-KgwZeroTouchProcessIdRecords -Records (Get-KgwZeroTouchProperty -Object $result -Names @("process_ids"))
    foreach ($expected in @(ConvertTo-KgwZeroTouchProcessIdRecords -Records $evidence.process_ids)) {
        $expectedNetwork = [string](Get-KgwZeroTouchProperty -Object $expected -Names @("network"))
        $expectedRole = [string](Get-KgwZeroTouchProperty -Object $expected -Names @("runtime_role"))
        $expectedBridgeInstanceId = [string](Get-KgwZeroTouchProperty -Object $expected -Names @("bridge_instance_id"))
        $expectedPid = [int64](Get-KgwZeroTouchProperty -Object $expected -Names @("pid"))
        $match = @(
            $resultProcessIds |
                Where-Object {
                    [string](Get-KgwZeroTouchProperty -Object $_ -Names @("network")) -eq $expectedNetwork -and
                    [string](Get-KgwZeroTouchProperty -Object $_ -Names @("runtime_role")) -eq $expectedRole -and
                    [string](Get-KgwZeroTouchProperty -Object $_ -Names @("bridge_instance_id")) -eq $expectedBridgeInstanceId -and
                    [int64](Get-KgwZeroTouchProperty -Object $_ -Names @("pid")) -eq $expectedPid
                }
        )
        if ($match.Count -eq 0) {
            $errors.Add("Artifact result did not record validated PID evidence for '$expectedRole/$expectedNetwork/$expectedPid'.")
        }
    }

    $resultPorts = ConvertTo-KgwZeroTouchPortRecords -Records (Get-KgwZeroTouchProperty -Object $result -Names @("ports"))
    foreach ($expected in @(ConvertTo-KgwZeroTouchPortRecords -Records $evidence.ports)) {
        $expectedNetwork = [string](Get-KgwZeroTouchProperty -Object $expected -Names @("network"))
        $expectedRole = [string](Get-KgwZeroTouchProperty -Object $expected -Names @("runtime_role"))
        $expectedHost = [string](Get-KgwZeroTouchProperty -Object $expected -Names @("host"))
        $expectedPort = [int64](Get-KgwZeroTouchProperty -Object $expected -Names @("port"))
        $match = @(
            $resultPorts |
                Where-Object {
                    [string](Get-KgwZeroTouchProperty -Object $_ -Names @("network")) -eq $expectedNetwork -and
                    [string](Get-KgwZeroTouchProperty -Object $_ -Names @("runtime_role")) -eq $expectedRole -and
                    [string](Get-KgwZeroTouchProperty -Object $_ -Names @("host")) -eq $expectedHost -and
                    [int64](Get-KgwZeroTouchProperty -Object $_ -Names @("port")) -eq $expectedPort
                }
        )
        if ($match.Count -eq 0) {
            $errors.Add("Artifact result did not record validated port evidence for '$expectedRole/$expectedNetwork/${expectedHost}:$expectedPort'.")
        }
    }

    if ([bool](Get-KgwZeroTouchProperty -Object $result -Names @("recovered_from_completed_wdio_run"))) {
        $originalFailureStage = [string](Get-KgwZeroTouchProperty -Object $result -Names @("original_failure_stage"))
        if ($originalFailureStage -ne "Zero-touch result writing") {
            $errors.Add("Recovered artifact original_failure_stage was '$originalFailureStage', expected 'Zero-touch result writing'.")
        }
        $recoveryTimestamp = [string](Get-KgwZeroTouchProperty -Object $result -Names @("recovery_timestamp"))
        if ([string]::IsNullOrWhiteSpace($recoveryTimestamp)) {
            $errors.Add("Recovered artifact did not record recovery_timestamp.")
        }
        $recoveryEvidenceFiles = @((Get-KgwZeroTouchProperty -Object $result -Names @("recovery_evidence_files")) | ForEach-Object { [string]$_ })
        if ($recoveryEvidenceFiles.Count -eq 0) {
            $errors.Add("Recovered artifact did not record recovery_evidence_files.")
        }
        foreach ($path in $recoveryEvidenceFiles) {
            if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
                $errors.Add("Recovered artifact references missing recovery evidence file: $path")
            }
        }
    }

    return [pscustomobject]@{
        passed = ($errors.Count -eq 0)
        errors = $errors.ToArray()
        result = $result
        evidence = $evidence
        wdio = $wdio
        testnet12_policy = $policy
    }
}
