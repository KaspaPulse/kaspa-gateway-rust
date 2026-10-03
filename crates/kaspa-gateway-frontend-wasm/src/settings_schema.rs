//! Owned settings schema; messages and default values preserve the existing contract.

pub(super) const BRIDGE_MANAGED: &[(&str, &str)] = &[
    ("testnet", "Managed by the selected network."),
    (
        "appdir",
        "Managed by KaspaGateway; isolated for this network.",
    ),
    (
        "inprocessAppdirMirror",
        "Managed by KaspaGateway; same isolated node directory.",
    ),
    ("inprocessNetworkArgs", "Managed by the selected network."),
    (
        "inprocessConfigfile",
        "Unsupported in managed mode: network and database ownership are managed.",
    ),
    (
        "inprocessOverrideParamsFile",
        "Unsupported in managed mode: network parameters are managed.",
    ),
    (
        "inprocessDevnet",
        "Unsupported: choose a supported network tab.",
    ),
    (
        "inprocessSimnet",
        "Unsupported: choose a supported network tab.",
    ),
    (
        "healthCheckPort",
        "Unsupported by the embedded bridge: no health HTTP service is owned.",
    ),
    (
        "webDashboardPort",
        "Unsupported by the embedded bridge: no dashboard HTTP service is owned.",
    ),
    (
        "logToFile",
        "Unsupported in managed mode; real process logs remain available in Monitor.",
    ),
    (
        "approxGeoLookup",
        "Unsupported by the embedded bridge runtime.",
    ),
];

pub(super) const BRIDGE_OPTIONAL: &[&str] = &[
    "config",
    "coinbaseTagSuffix",
    "inprocessRpcListenBorsh",
    "inprocessRpcListenJson",
    "inprocessListen",
    "inprocessAddPeer",
    "inprocessConnect",
    "inprocessPerfMetricsIntervalSec",
];

pub(super) const BRIDGE_REQUIRED: &[&str] = &[
    "nodeMode",
    "kaspadAddress",
    "stratumPort",
    "minShareDiff",
    "blockWaitTime",
    "printStats",
    "varDiff",
    "sharesPerMin",
    "varDiffStats",
    "extranonceSize",
    "pow2Clamp",
    "inprocessRpcListen",
    "inprocessLogLevel",
    "inprocessRamScale",
    "inprocessOutpeers",
    "inprocessMaxInpeers",
    "inprocessAsyncThreads",
    "internalCpuMinerAddress",
    "internalCpuMinerThreads",
];

pub(super) const NODE_DANGEROUS: &[(&str, &str)] = &[
    (
        "resetDb",
        "Deletes this network's database on every Start while enabled.",
    ),
    (
        "unsafeRpc",
        "Allows RPC outside loopback; use only on a trusted, protected network.",
    ),
    (
        "enableUnsyncedMining",
        "Allows mining before synchronization on a test network.",
    ),
];

pub(super) const NODE_ENDPOINTS: &[(&str, &str, &str, &str, bool)] = &[
    (
        "rpcListenEnabled",
        "rpcListenHost",
        "rpcListenPort",
        "rpcListen",
        true,
    ),
    (
        "rpcBorshEnabled",
        "rpcBorshHost",
        "rpcBorshPort",
        "rpcListenBorsh",
        true,
    ),
    (
        "rpcJsonEnabled",
        "rpcJsonHost",
        "rpcJsonPort",
        "rpcListenJson",
        true,
    ),
    (
        "listenEnabled",
        "listenHost",
        "listenPort",
        "p2pListen",
        false,
    ),
    (
        "externalIpEnabled",
        "externalIpHost",
        "externalIpPort",
        "externalIp",
        false,
    ),
    (
        "connectEnabled",
        "connectHost",
        "connectPort",
        "connectPeers",
        false,
    ),
    (
        "addPeerEnabled",
        "addPeerHost",
        "addPeerPort",
        "addPeers",
        false,
    ),
];

pub(super) const NODE_MANAGED: &[(&str, &str)] = &[
    (
        "appDir",
        "Managed by KaspaGateway; isolated for this network.",
    ),
    (
        "configFile",
        "Unsupported in managed mode: network and database ownership are managed.",
    ),
    (
        "overrideParamsFile",
        "Unsupported in managed mode: network parameters are managed.",
    ),
    ("testnet", "Managed by the selected network."),
    ("netsuffix", "Managed by the selected network."),
    (
        "noGrpc",
        "Unavailable in managed mode: gRPC is required for startup verification.",
    ),
    (
        "rpcListenEnabled",
        "Required in managed mode for startup verification.",
    ),
];

pub(super) const NODE_OPTIONAL: &[&str] = &[
    "uaComment",
    "retentionDays",
    "maxTrackedAddresses",
    "perfMetricsInterval",
    "rocksDbPreset",
    "rocksDbCacheSize",
    "rocksDbWalDir",
    "logDir",
];

pub(super) const NODE_REQUIRED: &[(&str, &str)] = &[
    ("logLevel", "info"),
    ("asyncThreads", "16"),
    ("ramScale", "1"),
    ("rpcMaxClients", "16"),
    ("outPeers", "8"),
    ("maxInPeers", "32"),
];
