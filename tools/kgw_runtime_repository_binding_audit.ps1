$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

# KGW_RUNTIME_REPOSITORY_BINDING_AUDIT_PS1_WRAPPER_R21C
# Compatibility wrapper.
# Canonical implementation lives in:
#   tools/kgw_runtime_repository_binding_gate.cjs

$RepoRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $RepoRoot

node tools/kgw_runtime_repository_binding_gate.cjs --strict --online @args
$Code = $LASTEXITCODE
if ($null -eq $Code) { $Code = 0 }

Write-Host ""
Write-Host "Command exit code: $Code"

if ($Code -ne 0) {
    throw "Runtime repository binding gate failed with exit code $Code"
}

return
