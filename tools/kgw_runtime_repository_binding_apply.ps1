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

function Replace-Cargo-Line([string]$path, [string]$alias, [string]$line) {
  $lines = Get-Content $path
  $found = $false
  for ($i = 0; $i -lt $lines.Count; $i++) {
    if ($lines[$i].TrimStart().StartsWith("$alias = {")) {
      $lines[$i] = $line
      $found = $true
      break
    }
  }
  if (-not $found) { throw "Missing Cargo alias: $alias in $path" }
  Set-Content $path $lines -Encoding UTF8
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
  $map = @{ mainnet = "Self::Mainnet"; testnet10 = "Self::Testnet10"; testnet12 = "Self::Testnet12" }
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
    $enum = if ($key -eq "mainline") { "KaspaRuntimeFamily::Mainline" } elseif ($key -eq "tn12") { "KaspaRuntimeFamily::Tn12" } else { throw "Unsupported node family: $key" }
    $arms += Network-Arm $groups[$key].Networks $enum
  }
  return @("    pub fn family(self) -> KaspaRuntimeFamily {", "        match self {") + $arms + @("        }", "    }") -join "`n"
}

function Bridge-Family-Function {
  $groups = Get-Family-Groups
  $arms = @()
  foreach ($key in $groups.Keys) {
    $enum = if ($key -eq "mainline") { "BridgeRuntimeFamily::Mainline" } elseif ($key -eq "tn12") { "BridgeRuntimeFamily::Tn12" } else { throw "Unsupported bridge family: $key" }
    $arms += Network-Arm $groups[$key].Networks $enum
  }
  return @("    pub fn family(self) -> BridgeRuntimeFamily {", "        match self {") + $arms + @("        }", "    }") -join "`n"
}

function Replace-Rust-Function([string]$path, [string]$signature, [string]$replacement) {
  $text = Get-Content $path -Raw
  $start = $text.IndexOf($signature)
  if ($start -lt 0) { throw "Missing Rust function signature: $signature in $path" }
  $braceStart = $text.IndexOf("{", $start)
  if ($braceStart -lt 0) { throw "Missing Rust function body: $signature in $path" }
  $depth = 0
  for ($i = $braceStart; $i -lt $text.Length; $i++) {
    if ($text[$i] -eq "{") { $depth++ }
    if ($text[$i] -eq "}") { $depth-- }
    if ($depth -eq 0) {
      $next = $text.Substring(0, $start) + $replacement + $text.Substring($i + 1)
      Set-Content $path $next -Encoding UTF8
      return
    }
  }
  throw "Could not find Rust function end: $signature in $path"
}

$mainline = Get-BindingByFamily "mainline"
$tn12 = Get-BindingByFamily "tn12"

$NodeCargo = "crates\kaspa-gateway-rk-node\Cargo.toml"
$BridgeCargo = "crates\kaspa-gateway-rk-bridge\Cargo.toml"
$ServiceController = "crates\kaspa-gateway-rk-node\src\kgw_service_controller.rs"
$OfficialRuntime = "crates\kaspa-gateway-rk-node\src\official_kaspa_runtime.rs"
$BridgeRuntime = "crates\kaspa-gateway-rk-bridge\src\lib.rs"

Replace-Cargo-Line $NodeCargo "kaspad-lib-mainline" (Cargo-Line "kaspad-lib-mainline" "kaspad" $mainline)
Replace-Cargo-Line $NodeCargo "kaspa-core-mainline" (Cargo-Line "kaspa-core-mainline" "kaspa-core" $mainline)
Replace-Cargo-Line $NodeCargo "kaspa-utils-mainline" (Cargo-Line "kaspa-utils-mainline" "kaspa-utils" $mainline)
Replace-Cargo-Line $NodeCargo "kaspad-lib-tn12" (Cargo-Line "kaspad-lib-tn12" "kaspad" $tn12)
Replace-Cargo-Line $NodeCargo "kaspa-core-tn12" (Cargo-Line "kaspa-core-tn12" "kaspa-core" $tn12)
Replace-Cargo-Line $NodeCargo "kaspa-utils-tn12" (Cargo-Line "kaspa-utils-tn12" "kaspa-utils" $tn12)

Replace-Cargo-Line $BridgeCargo "kaspa-stratum-bridge-mainline" (Cargo-Line "kaspa-stratum-bridge-mainline" "kaspa-stratum-bridge" $mainline)
Replace-Cargo-Line $BridgeCargo "kaspa-stratum-bridge-tn12" (Cargo-Line "kaspa-stratum-bridge-tn12" "kaspa-stratum-bridge" $tn12)

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



