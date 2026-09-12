function readPort(env, name, fallback) {
  const raw = String(env?.[name] ?? "").trim();
  if (!raw) return fallback;
  const value = Number(raw);
  if (!Number.isInteger(value) || value < 1024 || value > 65535) {
    throw new Error(`${name} must be an integer TCP port in 1024..65535; got ${raw}`);
  }
  return value;
}

export function runtimePortProfile(env = process.env) {
  return {
    mainnet: {
      rpcPort: readPort(env, "KGW_E2E_MAINNET_RPC_PORT", 16110),
      p2pPort: readPort(env, "KGW_E2E_MAINNET_P2P_PORT", 16111),
      bridgePort: readPort(env, "KGW_E2E_MAINNET_BRIDGE_PORT", 5556),
    },
    testnet10: {
      rpcPort: readPort(env, "KGW_E2E_TESTNET10_RPC_PORT", 16210),
      p2pPort: readPort(env, "KGW_E2E_TESTNET10_P2P_PORT", 16211),
      bridgePort: readPort(env, "KGW_E2E_TESTNET10_BRIDGE_PORT", 5656),
    },
  };
}
