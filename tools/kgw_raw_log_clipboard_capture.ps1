Set-StrictMode -Version Latest

if (-not (Get-Variable -Name KgwRawLogClipboardCaptureSequenceV1 -Scope Script -ErrorAction SilentlyContinue)) {
    $script:KgwRawLogClipboardCaptureSequenceV1 = 0
}

function Get-KgwRawLogUtf8NoBomEncodingV1 {
    return New-Object System.Text.UTF8Encoding -ArgumentList $false
}

function Get-KgwRawLogSafeDiagnosticTextV1 {
    param([AllowNull()][string]$Value)

    $text = if ($null -eq $Value) { "" } else { [string]$Value }
    $clean = ($text -replace "[`r`n`t]", " ").Trim()

    if ($clean -match "secret|token|private|mnemonic|wallet|address") {
        return "redacted-sensitive-value"
    }

    if ($clean.Length -gt 360) {
        return $clean.Substring(0, 360)
    }

    return $clean
}

function Get-KgwRawLogSha256V1 {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Text)

    $bytes = [System.Text.Encoding]::UTF8.GetBytes($Text)
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
        $hashBytes = $sha.ComputeHash($bytes)
    }
    finally {
        $sha.Dispose()
    }

    return -join ($hashBytes | ForEach-Object { $_.ToString("x2") })
}

function Get-KgwRawLogLineCountV1 {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Text)

    if ([string]::IsNullOrEmpty($Text)) {
        return 0
    }

    $normalized = $Text.Replace("`r`n", "`n").Replace("`r", "`n")
    return $normalized.Split("`n").Count
}

function Get-KgwRawLogCharacterCountV1 {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Text)

    try {
        return [System.Globalization.StringInfo]::ParseCombiningCharacters($Text).Count
    }
    catch {
        return $Text.Length
    }
}

function Test-KgwRawLogTransportWrapperTextV1 {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Text)

    if ([string]::IsNullOrEmpty($Text)) {
        return $false
    }

    $forbidden = @(
        "kgw_raw_process_log_v1",
        "[KGW_CHILD_STDOUT]",
        "[KGW_CHILD_STDERR]",
        "diagnostic_transport_record",
        ";source=self-worker;",
        ";runtime_role=",
        ";received_ms="
    )

    foreach ($marker in $forbidden) {
        if ($Text.IndexOf($marker, [System.StringComparison]::OrdinalIgnoreCase) -ge 0) {
            return $true
        }
    }

    $trimmed = $Text.TrimStart()
    if ($trimmed.StartsWith("{", [System.StringComparison]::Ordinal) -and
        $trimmed -match '"stage"\s*:' -and
        $trimmed -match '"network"\s*:' -and
        ($trimmed -match '"source"\s*:' -or $trimmed -match '"eventKind"\s*:\s*"diagnostic_transport_record"')) {
        return $true
    }

    return $false
}

function ConvertTo-KgwRawLogSafeFileSegmentV1 {
    param([AllowNull()][string]$Value)

    $clean = if ([string]::IsNullOrWhiteSpace($Value)) { "unknown" } else { [string]$Value }
    $clean = ($clean -replace '[^A-Za-z0-9_.-]+', '_').Trim("_")
    if ([string]::IsNullOrWhiteSpace($clean)) {
        $clean = "unknown"
    }
    if ($clean.Length -gt 80) {
        $clean = $clean.Substring(0, 80)
    }
    return $clean
}

function Get-KgwObjectPropertyValueV1 {
    param(
        [AllowNull()]$Object,
        [Parameter(Mandatory)][string]$Name
    )

    if ($null -eq $Object) {
        return $null
    }

    $property = $Object.PSObject.Properties[$Name]
    if ($null -eq $property) {
        return $null
    }

    return $property.Value
}

function Get-KgwFirstPresentValueV1 {
    param([AllowNull()][object[]]$Values)

    foreach ($value in $Values) {
        if ($null -eq $value) {
            continue
        }

        $text = [string]$value
        if ($text.Trim().Length -gt 0) {
            return $text
        }
    }

    return $null
}

function ConvertFrom-KgwStartTraceLineV1 {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Line)

    $prefix = "[KGW_START_TRACE] "
    if (-not $Line.StartsWith($prefix, [System.StringComparison]::Ordinal)) {
        return $null
    }

    $outerJson = $Line.Substring($prefix.Length)
    try {
        $outer = $outerJson | ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        return [pscustomobject]@{
            parsed = $false
            error = Get-KgwRawLogSafeDiagnosticTextV1 -Value $_.Exception.Message
            outer = $null
            details = $null
            extra = $null
        }
    }

    $details = $null
    $extra = $null
    $detailsText = Get-KgwObjectPropertyValueV1 -Object $outer -Name "details"
    if ($null -ne $detailsText -and ([string]$detailsText).Trim().Length -gt 0) {
        try {
            $details = ([string]$detailsText) | ConvertFrom-Json -ErrorAction Stop
            $extraValue = Get-KgwObjectPropertyValueV1 -Object $details -Name "extra"
            if ($null -ne $extraValue) {
                if ($extraValue -is [string]) {
                    $extra = ([string]$extraValue) | ConvertFrom-Json -ErrorAction Stop
                } else {
                    $extra = $extraValue
                }
            }
        }
        catch {
            $details = $null
            $extra = $null
        }
    }

    return [pscustomobject]@{
        parsed = $true
        error = $null
        outer = $outer
        details = $details
        extra = $extra
    }
}

function Get-KgwRawLogTraceClipboardMetadataV1 {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$TraceLine)

    $parsed = ConvertFrom-KgwStartTraceLineV1 -Line $TraceLine
    if ($null -eq $parsed -or -not $parsed.parsed) {
        return [pscustomobject]@{
            parsed = $false
            error = if ($null -eq $parsed) { "not a KGW start trace line" } else { $parsed.error }
            source = $null
            stage = $null
            network = $null
            runtime_role = $null
            bridge_instance_id = $null
            expected_sha256 = $null
            expected_line_count = $null
            expected_character_count = $null
        }
    }

    $outer = $parsed.outer
    $details = $parsed.details
    $extra = $parsed.extra

    $network = Get-KgwFirstPresentValueV1 -Values @(
        (Get-KgwObjectPropertyValueV1 -Object $outer -Name "network"),
        (Get-KgwObjectPropertyValueV1 -Object $details -Name "network"),
        (Get-KgwObjectPropertyValueV1 -Object $extra -Name "network")
    )
    $runtimeRole = Get-KgwFirstPresentValueV1 -Values @(
        (Get-KgwObjectPropertyValueV1 -Object $details -Name "runtimeRole"),
        (Get-KgwObjectPropertyValueV1 -Object $details -Name "runtime_role"),
        (Get-KgwObjectPropertyValueV1 -Object $extra -Name "runtimeRole"),
        (Get-KgwObjectPropertyValueV1 -Object $extra -Name "runtime_role")
    )
    $bridgeInstanceId = Get-KgwFirstPresentValueV1 -Values @(
        (Get-KgwObjectPropertyValueV1 -Object $details -Name "bridgeInstanceId"),
        (Get-KgwObjectPropertyValueV1 -Object $details -Name "bridge_instance_id"),
        (Get-KgwObjectPropertyValueV1 -Object $extra -Name "bridgeInstanceId"),
        (Get-KgwObjectPropertyValueV1 -Object $extra -Name "bridge_instance_id")
    )
    $expectedSha = Get-KgwFirstPresentValueV1 -Values @(
        (Get-KgwObjectPropertyValueV1 -Object $details -Name "sha256"),
        (Get-KgwObjectPropertyValueV1 -Object $extra -Name "sha256")
    )
    $expectedLineCount = Get-KgwFirstPresentValueV1 -Values @(
        (Get-KgwObjectPropertyValueV1 -Object $details -Name "lineCount"),
        (Get-KgwObjectPropertyValueV1 -Object $details -Name "line_count"),
        (Get-KgwObjectPropertyValueV1 -Object $extra -Name "lineCount"),
        (Get-KgwObjectPropertyValueV1 -Object $extra -Name "line_count")
    )
    $expectedCharacterCount = Get-KgwFirstPresentValueV1 -Values @(
        (Get-KgwObjectPropertyValueV1 -Object $details -Name "characterCount"),
        (Get-KgwObjectPropertyValueV1 -Object $details -Name "character_count"),
        (Get-KgwObjectPropertyValueV1 -Object $extra -Name "characterCount"),
        (Get-KgwObjectPropertyValueV1 -Object $extra -Name "character_count")
    )

    return [pscustomobject]@{
        parsed = $true
        error = $null
        source = Get-KgwObjectPropertyValueV1 -Object $outer -Name "source"
        stage = Get-KgwObjectPropertyValueV1 -Object $outer -Name "stage"
        network = $network
        runtime_role = $runtimeRole
        bridge_instance_id = $bridgeInstanceId
        expected_sha256 = $expectedSha
        expected_line_count = if ($null -eq $expectedLineCount) { $null } else { [int64]$expectedLineCount }
        expected_character_count = if ($null -eq $expectedCharacterCount) { $null } else { [int64]$expectedCharacterCount }
    }
}

function New-KgwRawLogClipboardCaptureFromTextV1 {
    param(
        [Parameter(Mandatory)][AllowEmptyString()][string]$Text,
        [Parameter(Mandatory)][AllowEmptyString()][string]$TraceLine,
        [Parameter(Mandatory)][string]$OutputDirectory,
        [string]$Reason = "event-time"
    )

    New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
    $script:KgwRawLogClipboardCaptureSequenceV1++

    $metadata = Get-KgwRawLogTraceClipboardMetadataV1 -TraceLine $TraceLine
    $actualSha = Get-KgwRawLogSha256V1 -Text $Text
    $lineCount = Get-KgwRawLogLineCountV1 -Text $Text
    $characterCount = Get-KgwRawLogCharacterCountV1 -Text $Text
    $transportWrapper = Test-KgwRawLogTransportWrapperTextV1 -Text $Text
    $expectedSha = if ($metadata.expected_sha256) { [string]$metadata.expected_sha256 } else { "" }
    $shaMatch = $expectedSha.Length -gt 0 -and $expectedSha.Equals($actualSha, [System.StringComparison]::OrdinalIgnoreCase)
    $captureTimestamp = (Get-Date).ToString("O")

    $errors = New-Object System.Collections.Generic.List[string]
    if (-not $metadata.parsed) {
        $errors.Add("Trace line could not be parsed: $($metadata.error)")
    }
    if ($expectedSha.Length -eq 0) {
        $errors.Add("Trace line did not include expected SHA256.")
    }
    if (-not $shaMatch) {
        $errors.Add("Expected SHA256 does not match event-time clipboard SHA256.")
    }
    if ($transportWrapper) {
        $errors.Add("Clipboard payload contains diagnostic transport text and was not saved as raw output.")
    }

    $stageSegment = ConvertTo-KgwRawLogSafeFileSegmentV1 -Value $metadata.stage
    $networkSegment = ConvertTo-KgwRawLogSafeFileSegmentV1 -Value $metadata.network
    $roleSegment = ConvertTo-KgwRawLogSafeFileSegmentV1 -Value $metadata.runtime_role
    $instanceSegment = ConvertTo-KgwRawLogSafeFileSegmentV1 -Value $metadata.bridge_instance_id
    $timestampSegment = (Get-Date -Format "yyyyMMdd-HHmmss-fff")
    $captureId = "{0}-{1:D4}-{2}-{3}-{4}-{5}" -f $timestampSegment, $script:KgwRawLogClipboardCaptureSequenceV1, $stageSegment, $networkSegment, $roleSegment, $instanceSegment
    $payloadFile = Join-Path $OutputDirectory "$captureId.raw.txt"
    $metadataFile = Join-Path $OutputDirectory "$captureId.capture.json"

    $rawPayloadSaved = $false
    if (-not $transportWrapper) {
        [System.IO.File]::WriteAllText($payloadFile, $Text, (Get-KgwRawLogUtf8NoBomEncodingV1))
        $rawPayloadSaved = $true
    } else {
        $payloadFile = $null
    }

    $capture = [ordered]@{
        capture_id = $captureId
        reason = $Reason
        event_stage = $metadata.stage
        trace_source = $metadata.source
        capture_timestamp = $captureTimestamp
        network = $metadata.network
        runtime_role = $metadata.runtime_role
        bridge_instance_id = $metadata.bridge_instance_id
        line_count = $lineCount
        character_count = $characterCount
        expected_line_count = $metadata.expected_line_count
        expected_character_count = $metadata.expected_character_count
        expected_sha256 = $expectedSha
        actual_sha256 = $actualSha
        sha256_match = $shaMatch
        transport_wrapper_detected = $transportWrapper
        raw_payload_saved = $rawPayloadSaved
        payload_file = $payloadFile
        metadata_file = $metadataFile
        valid = ($errors.Count -eq 0)
        errors = @($errors)
    }

    $capture |
        ConvertTo-Json -Depth 6 |
        Set-Content -LiteralPath $metadataFile -Encoding utf8

    return [pscustomobject]$capture
}

function New-KgwRawLogClipboardCaptureFromClipboardV1 {
    param(
        [Parameter(Mandatory)][AllowEmptyString()][string]$TraceLine,
        [Parameter(Mandatory)][string]$OutputDirectory,
        [string]$Reason = "event-time",
        [AllowNull()][scriptblock]$ClipboardReader = $null
    )

    try {
        if ($null -ne $ClipboardReader) {
            $text = & $ClipboardReader
        } else {
            $text = Get-Clipboard -Raw -ErrorAction Stop
        }
        if ($null -eq $text) {
            $text = ""
        }

        return New-KgwRawLogClipboardCaptureFromTextV1 `
            -Text ([string]$text) `
            -TraceLine $TraceLine `
            -OutputDirectory $OutputDirectory `
            -Reason $Reason
    }
    catch {
        New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
        $metadata = Get-KgwRawLogTraceClipboardMetadataV1 -TraceLine $TraceLine
        $timestampSegment = Get-Date -Format "yyyyMMdd-HHmmss-fff"
        $captureId = "{0}-{1:D4}-clipboard-unavailable" -f $timestampSegment, (++$script:KgwRawLogClipboardCaptureSequenceV1)
        $metadataFile = Join-Path $OutputDirectory "$captureId.capture.json"
        $capture = [ordered]@{
            capture_id = $captureId
            reason = $Reason
            event_stage = $metadata.stage
            trace_source = $metadata.source
            capture_timestamp = (Get-Date).ToString("O")
            network = $metadata.network
            runtime_role = $metadata.runtime_role
            bridge_instance_id = $metadata.bridge_instance_id
            line_count = $null
            character_count = $null
            expected_line_count = $metadata.expected_line_count
            expected_character_count = $metadata.expected_character_count
            expected_sha256 = $metadata.expected_sha256
            actual_sha256 = $null
            sha256_match = $false
            transport_wrapper_detected = $false
            raw_payload_saved = $false
            payload_file = $null
            metadata_file = $metadataFile
            valid = $false
            errors = @("Clipboard read failed: $(Get-KgwRawLogSafeDiagnosticTextV1 -Value $_.Exception.Message)")
        }
        $capture |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath $metadataFile -Encoding utf8
        return [pscustomobject]$capture
    }
}

function Test-KgwRawLogPayloadAcceptanceV1 {
    param(
        [Parameter(Mandatory)][AllowEmptyString()][string]$Text,
        [Parameter(Mandatory)][string]$Network,
        [Parameter(Mandatory)][string]$RuntimeRole,
        [AllowNull()][string]$BridgeInstanceId = $null
    )

    $errors = New-Object System.Collections.Generic.List[string]
    $warnings = New-Object System.Collections.Generic.List[string]
    $networkValue = ([string]$Network).Trim().ToLowerInvariant()
    $roleValue = ([string]$RuntimeRole).Trim().ToLowerInvariant()

    if ([string]::IsNullOrWhiteSpace($Text)) {
        $errors.Add("Raw payload is empty.")
    }

    if (Test-KgwRawLogTransportWrapperTextV1 -Text $Text) {
        $errors.Add("Raw payload contains transport wrapper text.")
    }

    if ($roleValue -eq "node" -and $networkValue -eq "testnet10") {
        if ($Text -notmatch "(?i)\bkaspad\b|kaspa") {
            $errors.Add("Testnet10 Node raw payload does not contain direct kaspad stdout/stderr evidence.")
        }
        if ($Text -notmatch "(?i)testnet10|testnet-10|tn10|kaspa-gateway-testnet10") {
            $errors.Add("Testnet10 Node raw payload does not contain Testnet10 application or data directory evidence.")
        }
        if ($Text -match "(?i)mainnet.*(kaspad|kaspa-gateway|appdata|data|db)|kaspa-gateway-mainnet|\\mainnet\\|/mainnet/") {
            $errors.Add("Testnet10 Node raw payload contains Mainnet node path evidence.")
        }
        if ($Text -notmatch "(?i)\brpc\b|\bp2p\b|listen|port|16210|16211|16310") {
            $warnings.Add("Testnet10 Node raw payload did not expose RPC or P2P evidence in the captured lines.")
        }
    }

    if ($roleValue -eq "bridge") {
        if ($Text -match "(?i)parallel-owned-self-worker status;role=bridge|no bridge worker status yet|Controller diagnostics are available separately|diagnosticLineCount=") {
            $errors.Add("Bridge status summary was captured instead of raw child process output.")
        }
        if ($Text -notmatch "(?i)\bbridge\b|stratum|rk-bridge|test-self-worker stdout role=bridge|test-self-worker stderr role=bridge") {
            $errors.Add("Bridge raw payload does not contain bridge child stdout/stderr evidence.")
        }
        if ($Text -match "(?i)test-self-worker stdout role=node|test-self-worker stderr role=node|runtimeRole.?node") {
            $errors.Add("Bridge raw payload contains node buffer records.")
        }
        if ([string]::IsNullOrWhiteSpace($BridgeInstanceId)) {
            $warnings.Add("Bridge instance identity was not provided by the capture trace.")
        }
    }

    return [pscustomobject]@{
        passed = ($errors.Count -eq 0)
        errors = @($errors)
        warnings = @($warnings)
        line_count = Get-KgwRawLogLineCountV1 -Text $Text
        character_count = Get-KgwRawLogCharacterCountV1 -Text $Text
        sha256 = Get-KgwRawLogSha256V1 -Text $Text
    }
}

function Test-KgwRawLogClipboardCaptureSelfTestV1 {
    $errors = New-Object System.Collections.Generic.List[string]
    $outputDirectory = Join-Path ([System.IO.Path]::GetTempPath()) ("kgw-raw-clipboard-capture-self-test-{0}-{1}" -f $PID, [System.Guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null

    $raw = "test-self-worker stdout role=bridge network=testnet10`r`ntest-self-worker stderr role=bridge network=testnet10"
    $sha = Get-KgwRawLogSha256V1 -Text $raw
    $trace = '[KGW_START_TRACE] {"timestamp":1,"source":"frontend","stage":"frontend.copy_log_succeeded","network":"testnet10","action":"copy-log","result":"ok","details":"{\"runtimeRole\":\"bridge\",\"bridgeInstanceId\":\"bridge-a\",\"characterCount\":' + (Get-KgwRawLogCharacterCountV1 -Text $raw) + ',\"lineCount\":' + (Get-KgwRawLogLineCountV1 -Text $raw) + ',\"sha256\":\"' + $sha + '\"}"}'

    $capture = New-KgwRawLogClipboardCaptureFromTextV1 -Text $raw -TraceLine $trace -OutputDirectory $outputDirectory -Reason "self-test"
    if (-not $capture.valid) {
        $errors.Add("Event-time capture should pass with matching SHA256.")
    }
    if (-not (Test-Path -LiteralPath $capture.payload_file)) {
        $errors.Add("Event-time capture did not save the raw payload file.")
    } elseif ([System.IO.File]::ReadAllText($capture.payload_file, [System.Text.Encoding]::UTF8) -ne $raw) {
        $errors.Add("Saved raw payload file does not exactly match event-time clipboard text.")
    }
    if ($capture.expected_sha256 -ne $capture.actual_sha256) {
        $errors.Add("Expected and actual SHA256 should be equal for the successful capture.")
    }

    $laterClipboard = "[KGW_CHILD_STDOUT] diagnostic_transport_record"
    $laterSha = Get-KgwRawLogSha256V1 -Text $laterClipboard
    if ($capture.actual_sha256 -eq $laterSha) {
        $errors.Add("Later clipboard mutation unexpectedly changed the earlier capture hash.")
    }
    if ((Test-Path -LiteralPath $capture.payload_file) -and ([System.IO.File]::ReadAllText($capture.payload_file, [System.Text.Encoding]::UTF8) -ne $raw)) {
        $errors.Add("Later clipboard mutation invalidated the earlier saved payload.")
    }

    $mismatchCapture = New-KgwRawLogClipboardCaptureFromTextV1 -Text "changed later" -TraceLine $trace -OutputDirectory $outputDirectory -Reason "self-test-mismatch"
    if ($mismatchCapture.valid -or $mismatchCapture.sha256_match) {
        $errors.Add("Capture with mismatched SHA256 should fail.")
    }

    $wrapperCapture = New-KgwRawLogClipboardCaptureFromTextV1 -Text "[KGW_CHILD_STDERR] diagnostic_transport_record" -TraceLine $trace -OutputDirectory $outputDirectory -Reason "self-test-wrapper"
    if ($wrapperCapture.raw_payload_saved -or $wrapperCapture.valid) {
        $errors.Add("Diagnostic transport output must not be saved as a raw payload.")
    }

    $status = Test-KgwRawLogPayloadAcceptanceV1 -Text "parallel-owned-self-worker status;role=bridge;network=mainnet;running=false;message=no bridge worker status yet" -Network "mainnet" -RuntimeRole "bridge" -BridgeInstanceId "bridge-a"
    if ($status.passed) {
        $errors.Add("Bridge status summary should be rejected as raw process output.")
    }

    $missing = Test-KgwRawLogPayloadAcceptanceV1 -Text "" -Network "testnet10" -RuntimeRole "bridge" -BridgeInstanceId "bridge-a"
    if ($missing.passed) {
        $errors.Add("Missing bridge output should fail acceptance.")
    }

    $transport = Test-KgwRawLogPayloadAcceptanceV1 -Text "kgw_raw_process_log_v1;network=mainnet;source=self-worker;runtime_role=bridge;received_ms=1;line=fake" -Network "mainnet" -RuntimeRole "bridge" -BridgeInstanceId "bridge-a"
    if ($transport.passed) {
        $errors.Add("Transport wrappers should fail acceptance.")
    }

    return [pscustomobject]@{
        passed = ($errors.Count -eq 0)
        errors = @($errors)
        output_directory = $outputDirectory
    }
}
