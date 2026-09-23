$ErrorActionPreference = "Stop"
cd (Resolve-Path (Join-Path $PSScriptRoot ".."))
$ManifestPath = "config\runtime-repository-bindings.json"
$m = Get-Content $ManifestPath -Raw | ConvertFrom-Json

function Get-BindingByFamily([string]$family) {
  foreach ($p in $m.networks.PSObject.Properties) {
    if ($p.Value.family -eq $family) { return $p.Value }
  }
  throw "Missing binding family: $family"
}

function Get-Spec([object]$b) {
  if ($b.rev) { return @{ Key = "rev"; Value = $b.rev } }
  if ($b.branch) { return @{ Key = "branch"; Value = $b.branch } }
  throw "Binding must define branch or rev."
}

function Cargo-Line([string]$alias, [string]$package, [object]$binding) {
  $spec = Get-Spec $binding
  return "$alias = { package = ""$package"", git = ""$($binding.repo)"", $($spec.Key) = ""$($spec.Value)"", optional = true }"
}

function Simple-Cargo-Line([string]$alias, [object]$binding) {
  $spec = Get-Spec $binding
  return "$alias = { git = ""$($binding.repo)"", $($spec.Key) = ""$($spec.Value)"" }"
}

$Utf8NoBom = [System.Text.UTF8Encoding]::new($false)

function Read-Text([string]$path) {
  return [System.IO.File]::ReadAllText((Resolve-Path $path))
}

function Source-Newline([string]$text) {
  if ($text.Contains("`r`n")) { return "`r`n" }
  return "`n"
}

function Write-Text-If-Changed([string]$path, [string]$before, [string]$after) {
  if ($after -ne $before) {
    [System.IO.File]::WriteAllText((Resolve-Path $path), $after, $Utf8NoBom)
  }
}

function Replace-Cargo-Line([string]$path, [string]$alias, [string]$line) {
  $text = Read-Text $path
  $pattern = "(?m)^(?<indent>[ \t]*)" + [regex]::Escape($alias) + "[ \t]*=[ \t]*\{[^\r\n]*\}[ \t]*(?=\r?$)"
  $match = [regex]::Match($text, $pattern)
  if (-not $match.Success) { throw "Missing Cargo alias: $alias in $path" }
  $replacement = $match.Groups["indent"].Value + $line
  $next = $text.Substring(0, $match.Index) + $replacement + $text.Substring($match.Index + $match.Length)
  Write-Text-If-Changed $path $text $next
}

function Get-Family-Groups {
  $groups = [ordered]@{}
  foreach ($p in $m.networks.PSObject.Properties) {
    $network = $p.Name
    $binding = $p.Value
    $family = $binding.family
    if (-not $groups.Contains($family)) {
      $groups[$family] = [ordered]@{ Binding = $binding; Networks = @($network) }
    } else {
      $groups[$family].Networks += $network
    }
  }
  return $groups
}

function Network-Arm([string[]]$networks, [string]$value) {
  $map = @{ mainnet = "Self::Mainnet"; testnet10 = "Self::Testnet10"; testnet13 = "Self::Testnet13" }
  $left = ($networks | ForEach-Object { $map[$_] }) -join " | "
  return "            $left => $value,"
}

function Branch-Function {
  $groups = Get-Family-Groups
  $arms = @()
  foreach ($key in $groups.Keys) {
    $b = $groups[$key].Binding
    $arms += Network-Arm $groups[$key].Networks """$($b.branch)"""
  }
  return @("    pub fn branch(self) -> &'static str {", "        match self {") + $arms + @("        }", "    }") -join "`n"
}

function Revision-Function {
  $groups = Get-Family-Groups
  $arms = @()
  foreach ($key in $groups.Keys) {
    $b = $groups[$key].Binding
    $arms += Network-Arm $groups[$key].Networks """$($b.rev)"""
  }
  return @("    pub fn revision(self) -> &'static str {", "        match self {") + $arms + @("        }", "    }") -join "`n"
}

function Node-Family-Function {
  $groups = Get-Family-Groups
  $arms = @()
  foreach ($key in $groups.Keys) {
    $enum = if ($key -eq "mainline") { "KaspaRuntimeFamily::Mainline" } elseif ($key -eq "tn13") { "KaspaRuntimeFamily::Tn13" } else { throw "Unsupported node family: $key" }
    $arms += Network-Arm $groups[$key].Networks $enum
  }
  return @("    pub fn family(self) -> KaspaRuntimeFamily {", "        match self {") + $arms + @("        }", "    }") -join "`n"
}

function Bridge-Family-Function {
  $groups = Get-Family-Groups
  $arms = @()
  foreach ($key in $groups.Keys) {
    $enum = if ($key -eq "mainline") { "BridgeRuntimeFamily::Mainline" } elseif ($key -eq "tn13") { "BridgeRuntimeFamily::Tn13" } else { throw "Unsupported bridge family: $key" }
    $arms += Network-Arm $groups[$key].Networks $enum
  }
  return @("    pub fn family(self) -> BridgeRuntimeFamily {", "        match self {") + $arms + @("        }", "    }") -join "`n"
}

function Replace-Rust-Function([string]$path, [string]$signature, [string]$replacement) {
  $text = Read-Text $path
  $start = $text.IndexOf($signature)
  if ($start -lt 0) { throw "Missing Rust function signature: $signature in $path" }
  $lineStart = $text.LastIndexOf("`n", $start)
  if ($lineStart -lt 0) { $lineStart = 0 } else { $lineStart++ }
  $braceStart = $text.IndexOf("{", $start)
  if ($braceStart -lt 0) { throw "Missing Rust function body: $signature in $path" }
  $depth = 0
  for ($i = $braceStart; $i -lt $text.Length; $i++) {
    if ($text[$i] -eq "{") { $depth++ }
    if ($text[$i] -eq "}") { $depth-- }
    if ($depth -eq 0) {
      $newline = Source-Newline $text
      $stableReplacement = $replacement.Replace("`r`n", "`n").Replace("`n", $newline)
      $next = $text.Substring(0, $lineStart) + $stableReplacement + $text.Substring($i + 1)
      Write-Text-If-Changed $path $text $next
      return
    }
  }
  throw "Could not find Rust function end: $signature in $path"
}

$mainline = Get-BindingByFamily "mainline"
$tn13 = Get-BindingByFamily "tn13"

$NodeCargo = "crates\kaspa-gateway-rk-node\Cargo.toml"
$BridgeCargo = "crates\kaspa-gateway-rk-bridge\Cargo.toml"
$CliCargo = "apps\kaspa-gateway-cli\Cargo.toml"
$CoreCargo = "crates\kaspa-gateway-core\Cargo.toml"
$ServiceController = "crates\kaspa-gateway-rk-node\src\kgw_service_controller.rs"
$OfficialRuntime = "crates\kaspa-gateway-rk-node\src\official_kaspa_runtime.rs"
$BridgeRuntime = "crates\kaspa-gateway-rk-bridge\src\lib.rs"

foreach ($family in @("mainline", "tn13")) {
  $binding = Get-BindingByFamily $family
  $nodeAliases = @($binding.nodeAliases)
  $nodePackages = @($binding.packages.node)
  if ($nodeAliases.Count -ne $nodePackages.Count) {
    throw "Node alias/package count mismatch for family: $family"
  }
  for ($i = 0; $i -lt $nodeAliases.Count; $i++) {
    Replace-Cargo-Line $NodeCargo $nodeAliases[$i] (Cargo-Line $nodeAliases[$i] $nodePackages[$i] $binding)
  }

  $bridgeAliases = @($binding.bridgeAliases)
  $bridgePackages = @($binding.packages.bridge)
  if ($bridgeAliases.Count -ne $bridgePackages.Count) {
    throw "Bridge alias/package count mismatch for family: $family"
  }
  for ($i = 0; $i -lt $bridgeAliases.Count; $i++) {
    Replace-Cargo-Line $BridgeCargo $bridgeAliases[$i] (Cargo-Line $bridgeAliases[$i] $bridgePackages[$i] $binding)
  }
}

Replace-Cargo-Line $CliCargo "kaspa-grpc-client-live" (Cargo-Line "kaspa-grpc-client-live" "kaspa-grpc-client" $mainline)
Replace-Cargo-Line $CliCargo "kaspa-rpc-core-live" (Cargo-Line "kaspa-rpc-core-live" "kaspa-rpc-core" $mainline)
Replace-Cargo-Line $CoreCargo "kaspa-addresses" (Simple-Cargo-Line "kaspa-addresses" $mainline)

Replace-Rust-Function $ServiceController "pub fn branch(self) -> &'static str" (Branch-Function)
Replace-Rust-Function $ServiceController "pub fn revision(self) -> &'static str" (Revision-Function)
Replace-Rust-Function $OfficialRuntime "pub fn branch(self) -> &'static str" (Branch-Function)
Replace-Rust-Function $OfficialRuntime "pub fn revision(self) -> &'static str" (Revision-Function)
Replace-Rust-Function $OfficialRuntime "pub fn family(self) -> KaspaRuntimeFamily" (Node-Family-Function)
Replace-Rust-Function $BridgeRuntime "pub fn branch(self) -> &'static str" (Branch-Function)
Replace-Rust-Function $BridgeRuntime "pub fn revision(self) -> &'static str" (Revision-Function)
Replace-Rust-Function $BridgeRuntime "pub fn family(self) -> BridgeRuntimeFamily" (Bridge-Family-Function)

Write-Host "KGW runtime repository binding apply"
Write-Host "status=PASS"
