$ErrorActionPreference = "Stop"
cd (Resolve-Path (Join-Path $PSScriptRoot ".."))
$m = Get-Content "config\runtime-repository-bindings.json" -Raw | ConvertFrom-Json

function Get-BindingByFamily([string]$family) {
  foreach ($p in $m.networks.PSObject.Properties) {
    if ($p.Value.family -eq $family) { return $p.Value }
  }
  throw "Missing binding family: $family"
}

function Assert-Contains([string]$path, [string]$needle, [string]$label) {
  $text = Get-Content $path -Raw
  if (-not $text.Contains($needle)) {
    throw "Audit failed: $label missing in $path. Expected: $needle"
  }
}

function Spec-Key([object]$b) { if ($b.rev) { "rev" } else { "branch" } }
function Spec-Value([object]$b) { if ($b.rev) { $b.rev } else { $b.branch } }
function Cargo-Line([string]$alias, [string]$package, [object]$binding) {
  $key = Spec-Key $binding
  $value = Spec-Value $binding
  return "$alias = { package = ""$package"", git = ""$($binding.repo)"", $key = ""$value"", optional = true }"
}

$mainline = Get-BindingByFamily "mainline"
$tn12 = Get-BindingByFamily "tn12"
$NodeCargo = "crates\kaspa-gateway-rk-node\Cargo.toml"
$BridgeCargo = "crates\kaspa-gateway-rk-bridge\Cargo.toml"
$ServiceController = "crates\kaspa-gateway-rk-node\src\kgw_service_controller.rs"
$OfficialRuntime = "crates\kaspa-gateway-rk-node\src\official_kaspa_runtime.rs"
$BridgeRuntime = "crates\kaspa-gateway-rk-bridge\src\lib.rs"

Assert-Contains $NodeCargo (Cargo-Line "kaspad-lib-mainline" "kaspad" $mainline) "mainline kaspad"
Assert-Contains $NodeCargo (Cargo-Line "kaspa-core-mainline" "kaspa-core" $mainline) "mainline kaspa-core"
Assert-Contains $NodeCargo (Cargo-Line "kaspa-utils-mainline" "kaspa-utils" $mainline) "mainline kaspa-utils"
Assert-Contains $NodeCargo (Cargo-Line "kaspad-lib-tn12" "kaspad" $tn12) "tn12 kaspad"
Assert-Contains $NodeCargo (Cargo-Line "kaspa-core-tn12" "kaspa-core" $tn12) "tn12 kaspa-core"
Assert-Contains $NodeCargo (Cargo-Line "kaspa-utils-tn12" "kaspa-utils" $tn12) "tn12 kaspa-utils"
Assert-Contains $BridgeCargo (Cargo-Line "kaspa-stratum-bridge-mainline" "kaspa-stratum-bridge" $mainline) "mainline bridge"
Assert-Contains $BridgeCargo (Cargo-Line "kaspa-stratum-bridge-tn12" "kaspa-stratum-bridge" $tn12) "tn12 bridge"

Assert-Contains $ServiceController "Self::Mainnet => ""master""" "service mainnet branch"
Assert-Contains $ServiceController "Self::Testnet10 | Self::Testnet12 => ""RKStratumTN12""" "service testnet branch"
Assert-Contains $OfficialRuntime "Self::Mainnet => ""master""" "official mainnet branch"
Assert-Contains $OfficialRuntime "Self::Testnet10 | Self::Testnet12 => ""RKStratumTN12""" "official testnet branch"
Assert-Contains $OfficialRuntime "Self::Mainnet => KaspaRuntimeFamily::Mainline" "official mainline family"
Assert-Contains $OfficialRuntime "Self::Testnet10 | Self::Testnet12 => KaspaRuntimeFamily::Tn12" "official tn12 family"
Assert-Contains $BridgeRuntime "Self::Mainnet => ""master""" "bridge mainnet branch"
Assert-Contains $BridgeRuntime "Self::Testnet10 | Self::Testnet12 => ""RKStratumTN12""" "bridge testnet branch"
Assert-Contains $BridgeRuntime "Self::Mainnet => BridgeRuntimeFamily::Mainline" "bridge mainline family"
Assert-Contains $BridgeRuntime "Self::Testnet10 | Self::Testnet12 => BridgeRuntimeFamily::Tn12" "bridge tn12 family"

Write-Host "KGW runtime repository binding audit"
foreach ($p in $m.networks.PSObject.Properties) {
  $n = $p.Name
  $b = $p.Value
  $branch = if ($b.branch) { $b.branch } else { "" }
  $rev = if ($b.rev) { $b.rev } else { "" }
  Write-Host "network=$n;family=$($b.family);branch=$branch;rev=$rev;node_repo=$($b.repo);bridge_repo=$($b.repo);feature=$($b.feature)"
}
Write-Host "status=PASS"
