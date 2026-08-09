pub mod kgw_inproc_node_only;
pub mod kgw_inproc_owner;
pub mod kgw_real_owner_runtime;
pub mod kgw_service_controller;
pub mod official_kaspa_runtime;

pub use kgw_inproc_owner::{
    KgwInprocError, KgwInprocPlan, KgwInprocRuntimeState, KgwInprocStartRequest, KgwInprocStep,
    build_kgw_inproc_plan, kgw_inproc_summary_v1,
};

pub use kgw_inproc_node_only::{KgwInProcNodeOnly, KgwInProcNodeOnlyStatus};

pub use official_kaspa_runtime::{
    KaspaBridgeRuntimeMode, KaspaNodeRuntimeMode, KaspaRuntimeError, KaspaRuntimeFamily,
    KaspaRuntimeNetwork, KaspaRuntimePlan, KaspaRuntimeServiceEvent, KaspaRuntimeServiceEventKind,
    KaspaRuntimeSettings, KaspaRuntimeStep, all_parallel_runtime_plans_v1,
    build_official_kaspa_runtime_plan_v1, official_kaspa_runtime_summary_v1,
    official_node_mainline_dependency_marker_v1, official_node_tn12_dependency_marker_v1,
    runtime_service_events_from_settings_v1,
};

pub const KGW_RUNTIME_OWNER_CRATE: &str = "kaspa-gateway-rk-node";
pub const KGW_RUNTIME_OWNER_MODEL: &str = "kgw-parallel-node-bridge-service-event-owner";
pub const KGW_RUNTIME_THREAD_NAME: &str = "kaspad";
pub const KGW_OFFICIAL_KASPA_REPOSITORY: &str = "https://github.com/kaspanet/rusty-kaspa.git";
pub const KGW_MAINLINE_BRANCH: &str = "master";
pub const KGW_TN12_BRANCH: &str = "tn12";

pub fn runtime_owner_summary_v1() -> &'static str {
    official_kaspa_runtime_summary_v1()
}

pub fn runtime_owner_plan_for_network_v1(
    network: Option<String>,
) -> Result<KaspaRuntimePlan, KaspaRuntimeError> {
    let settings = KaspaRuntimeSettings {
        network: network.unwrap_or_else(|| "mainnet".to_string()),
        node_mode: KaspaNodeRuntimeMode::Disabled,
        bridge_mode: KaspaBridgeRuntimeMode::Disabled,
        ..KaspaRuntimeSettings::default()
    };

    build_official_kaspa_runtime_plan_v1(settings)
}

pub fn runtime_owner_status_for_network_v1(
    network: Option<String>,
) -> Result<String, KaspaRuntimeError> {
    let plan = runtime_owner_plan_for_network_v1(network)?;

    Ok(format!(
        "owner={};model={};official_repo={};network={};family={};branch={};node_mode={};bridge_mode={};running=false;healthy=false;starts_now=false;parallel_safe=true;message={}",
        KGW_RUNTIME_OWNER_CRATE,
        KGW_RUNTIME_OWNER_MODEL,
        KGW_OFFICIAL_KASPA_REPOSITORY,
        plan.network.as_str(),
        plan.family.as_str(),
        plan.branch,
        plan.node_mode.as_str(),
        plan.bridge_mode.as_str(),
        plan.decision
    ))
}

pub fn all_parallel_runtime_plans_log_v1() -> Result<String, KaspaRuntimeError> {
    let plans = all_parallel_runtime_plans_v1()?;

    Ok(plans
        .iter()
        .map(|plan| plan.to_log_line())
        .collect::<Vec<_>>()
        .join("\n"))
}

pub use kgw_service_controller::{
    BridgeNodeKind, KaspadNodeKind, KaspadServiceEvents, KgwNetwork, KgwServiceController,
    KgwServiceError, NodeSettings, RuntimeSlotStatus, exact_kgw_service_controller_summary_v1,
};

pub use kgw_real_owner_runtime::{
    KgwRealOwnerError, KgwRealOwnerRuntime, KgwRuntimeFeatureStatus, KgwRuntimeSessionStatus,
    KgwRuntimeStartPolicy, real_owner_runtime_summary_v1,
};
