use std::fmt;

/// KGW style in-process lifecycle step.
///
/// This is intentionally dependency-light in this phase. The real kaspad types
/// are not linked until the Cargo dependency policy is explicit and no local
/// clone/source-cache is involved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KgwInprocStep {
    ConvertConfigToKaspadArgs,
    ComputeFileDescriptorBudget,
    CreateCoreWithRuntime,
    SpawnNamedKaspadThread,
    StoreRpcCoreService,
    AttachRpcToApplicationServices,
    ShutdownCore,
    JoinKaspadThread,
}

impl KgwInprocStep {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ConvertConfigToKaspadArgs => "convert-config-to-kaspad-args",
            Self::ComputeFileDescriptorBudget => "compute-file-descriptor-budget",
            Self::CreateCoreWithRuntime => "create-core-with-runtime",
            Self::SpawnNamedKaspadThread => "spawn-named-kaspad-thread",
            Self::StoreRpcCoreService => "store-rpc-core-service",
            Self::AttachRpcToApplicationServices => "attach-rpc-to-application-services",
            Self::ShutdownCore => "shutdown-core",
            Self::JoinKaspadThread => "join-kaspad-thread",
        }
    }
}

/// Runtime state exposed to the app/UI.
///
/// The current phase ports the ownership and lifecycle mechanism, but does not
/// start the real node. Real start must be enabled only after kaspad crates are
/// linked by normal Cargo dependencies, not local clones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KgwInprocRuntimeState {
    ContractOnly,
    BlockedUntilCargoDependenciesAreOwned,
    FutureRuntimeEnabled,
}

impl fmt::Display for KgwInprocRuntimeState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContractOnly => formatter.write_str("contract-only"),
            Self::BlockedUntilCargoDependenciesAreOwned => {
                formatter.write_str("blocked-until-cargo-dependencies-are-owned")
            }
            Self::FutureRuntimeEnabled => formatter.write_str("future-runtime-enabled"),
        }
    }
}

/// Request shape for an in-process node start.
///
/// This is intentionally not `kaspad_lib::args::Args` yet. It is the stable
/// gateway-side input that will later be converted into kaspad args by this
/// same owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KgwInprocStartRequest {
    pub network: String,
    pub app_dir_name: String,
    pub enable_utxo_index: bool,
    pub archival: bool,
    pub rpc_max_clients: u16,
    pub inbound_limit: u16,
    pub outbound_target: u16,
    pub explicit_runtime_opt_in: bool,
}

impl Default for KgwInprocStartRequest {
    fn default() -> Self {
        Self {
            network: "mainnet".to_string(),
            app_dir_name: "kaspa-gateway-rk-node".to_string(),
            enable_utxo_index: true,
            archival: false,
            rpc_max_clients: 128,
            inbound_limit: 128,
            outbound_target: 8,
            explicit_runtime_opt_in: false,
        }
    }
}

impl KgwInprocStartRequest {
    pub fn normalized_network(&self) -> String {
        self.network.trim().to_ascii_lowercase()
    }

    pub fn validate(&self) -> Result<(), KgwInprocError> {
        match self.normalized_network().as_str() {
            "mainnet" | "testnet" | "testnet10" | "testnet12" => {}
            _ => return Err(KgwInprocError::UnsupportedNetwork),
        }

        if self.app_dir_name.trim().is_empty() {
            return Err(KgwInprocError::InvalidAppDirName);
        }

        if self.app_dir_name.contains("..")
            || self.app_dir_name.contains('/')
            || self.app_dir_name.contains('\\')
        {
            return Err(KgwInprocError::InvalidAppDirName);
        }

        Ok(())
    }
}

/// Plan that mirrors kgw InProc without linking or starting kaspad yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KgwInprocPlan {
    pub state: KgwInprocRuntimeState,
    pub owner_crate: &'static str,
    pub thread_name: &'static str,
    pub network: String,
    pub app_dir_name: String,
    pub uses_external_executable: bool,
    pub uses_git_clone: bool,
    pub uses_source_cache: bool,
    pub starts_runtime_now: bool,
    pub exposes_inproc_rpc: bool,
    pub steps: Vec<KgwInprocStep>,
    pub blocked_reason: String,
}

impl KgwInprocPlan {
    pub fn to_log_line(&self) -> String {
        let steps = self
            .steps
            .iter()
            .map(|step| step.as_str())
            .collect::<Vec<_>>()
            .join(",");

        format!(
            "state={};owner={};thread={};network={};app_dir={};external_exe={};git_clone={};source_cache={};starts_now={};inproc_rpc={};steps={};reason={}",
            self.state,
            self.owner_crate,
            self.thread_name,
            self.network,
            self.app_dir_name,
            self.uses_external_executable,
            self.uses_git_clone,
            self.uses_source_cache,
            self.starts_runtime_now,
            self.exposes_inproc_rpc,
            steps,
            self.blocked_reason
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KgwInprocError {
    UnsupportedNetwork,
    InvalidAppDirName,
}

impl fmt::Display for KgwInprocError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedNetwork => formatter.write_str("unsupported network"),
            Self::InvalidAppDirName => formatter.write_str("invalid app dir name"),
        }
    }
}

impl std::error::Error for KgwInprocError {}

pub const KGW_INPROC_STEPS: &[KgwInprocStep] = &[
    KgwInprocStep::ConvertConfigToKaspadArgs,
    KgwInprocStep::ComputeFileDescriptorBudget,
    KgwInprocStep::CreateCoreWithRuntime,
    KgwInprocStep::SpawnNamedKaspadThread,
    KgwInprocStep::StoreRpcCoreService,
    KgwInprocStep::AttachRpcToApplicationServices,
    KgwInprocStep::ShutdownCore,
    KgwInprocStep::JoinKaspadThread,
];

/// Build the accepted kgw InProc plan.
///
/// This function is the canonical next-step owner. It intentionally refuses
/// clone/source-cache behavior and keeps runtime start disabled in this phase.
pub fn build_kgw_inproc_plan(
    request: KgwInprocStartRequest,
) -> Result<KgwInprocPlan, KgwInprocError> {
    request.validate()?;

    let reason = if request.explicit_runtime_opt_in {
        "explicit opt-in received, but real kaspad crates are not linked by owned Cargo dependencies yet; no clone/source-cache fallback is allowed"
    } else {
        "runtime opt-in is false; plan is contract-only and no node is started"
    };

    Ok(KgwInprocPlan {
        state: if request.explicit_runtime_opt_in {
            KgwInprocRuntimeState::BlockedUntilCargoDependenciesAreOwned
        } else {
            KgwInprocRuntimeState::ContractOnly
        },
        owner_crate: "kaspa-gateway-rk-node",
        thread_name: "kaspad",
        network: request.normalized_network(),
        app_dir_name: request.app_dir_name.trim().to_string(),
        uses_external_executable: false,
        uses_git_clone: false,
        uses_source_cache: false,
        starts_runtime_now: false,
        exposes_inproc_rpc: true,
        steps: KGW_INPROC_STEPS.to_vec(),
        blocked_reason: reason.to_string(),
    })
}

/// Compatibility summary for existing UI/log/status layers.
pub fn kgw_inproc_summary_v1() -> &'static str {
    "KGW InProc mechanism ported to kaspa-gateway-rk-node owner: config->kaspad args, fd budget, create_core_with_runtime, named kaspad thread, stored inproc RPC, app-service attach, shutdown, join. No clone/source-cache. Runtime start remains disabled until kaspad crates are linked through owned Cargo dependencies."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_never_uses_clone_or_source_cache() {
        let plan = build_kgw_inproc_plan(KgwInprocStartRequest::default()).unwrap();

        assert!(!plan.uses_git_clone);
        assert!(!plan.uses_source_cache);
        assert!(!plan.uses_external_executable);
        assert!(!plan.starts_runtime_now);
        assert!(plan.exposes_inproc_rpc);
        assert_eq!(plan.thread_name, "kaspad");
    }

    #[test]
    fn plan_contains_kgw_inproc_lifecycle() {
        let plan = build_kgw_inproc_plan(KgwInprocStartRequest::default()).unwrap();

        assert!(
            plan.steps
                .contains(&KgwInprocStep::ConvertConfigToKaspadArgs)
        );
        assert!(
            plan.steps
                .contains(&KgwInprocStep::ComputeFileDescriptorBudget)
        );
        assert!(plan.steps.contains(&KgwInprocStep::CreateCoreWithRuntime));
        assert!(plan.steps.contains(&KgwInprocStep::SpawnNamedKaspadThread));
        assert!(plan.steps.contains(&KgwInprocStep::StoreRpcCoreService));
        assert!(
            plan.steps
                .contains(&KgwInprocStep::AttachRpcToApplicationServices)
        );
        assert!(plan.steps.contains(&KgwInprocStep::ShutdownCore));
        assert!(plan.steps.contains(&KgwInprocStep::JoinKaspadThread));
    }

    #[test]
    fn explicit_opt_in_is_blocked_until_owned_dependencies_exist() {
        let request = KgwInprocStartRequest {
            explicit_runtime_opt_in: true,
            ..KgwInprocStartRequest::default()
        };

        let plan = build_kgw_inproc_plan(request).unwrap();

        assert_eq!(
            plan.state,
            KgwInprocRuntimeState::BlockedUntilCargoDependenciesAreOwned
        );
        assert!(!plan.starts_runtime_now);
    }

    #[test]
    fn supports_all_gateway_node_networks() {
        for network in ["mainnet", "testnet", "testnet10", "testnet12"] {
            let request = KgwInprocStartRequest {
                network: network.to_string(),
                ..KgwInprocStartRequest::default()
            };

            let plan = build_kgw_inproc_plan(request).unwrap();
            assert_eq!(plan.network, network);
        }
    }

    #[test]
    fn rejects_path_like_app_dir_names() {
        let request = KgwInprocStartRequest {
            app_dir_name: "../bad".to_string(),
            ..KgwInprocStartRequest::default()
        };

        assert_eq!(
            build_kgw_inproc_plan(request).unwrap_err(),
            KgwInprocError::InvalidAppDirName
        );
    }
}
