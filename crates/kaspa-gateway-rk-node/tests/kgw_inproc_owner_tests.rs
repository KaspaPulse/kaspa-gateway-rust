use kaspa_gateway_rk_node::{
    KgwInProcNodeOnly, KgwInprocRuntimeState, KgwInprocStartRequest, KgwInprocStep,
    build_kgw_inproc_plan, kgw_inproc_summary_v1,
};

#[test]
fn kgw_inproc_owner_is_single_canonical_owner() {
    let plan = build_kgw_inproc_plan(KgwInprocStartRequest::default()).unwrap();

    assert_eq!(plan.owner_crate, "kaspa-gateway-rk-node");
    assert_eq!(plan.thread_name, "kaspad");
    assert!(!plan.uses_external_executable);
    assert!(!plan.uses_git_clone);
    assert!(!plan.uses_source_cache);
    assert!(!plan.starts_runtime_now);
    assert!(plan.exposes_inproc_rpc);
}

#[test]
fn kgw_inproc_lifecycle_matches_reference_shape() {
    let plan = build_kgw_inproc_plan(KgwInprocStartRequest::default()).unwrap();

    let expected = [
        KgwInprocStep::ConvertConfigToKaspadArgs,
        KgwInprocStep::ComputeFileDescriptorBudget,
        KgwInprocStep::CreateCoreWithRuntime,
        KgwInprocStep::SpawnNamedKaspadThread,
        KgwInprocStep::StoreRpcCoreService,
        KgwInprocStep::AttachRpcToApplicationServices,
        KgwInprocStep::ShutdownCore,
        KgwInprocStep::JoinKaspadThread,
    ];

    for step in expected {
        assert!(plan.steps.contains(&step), "missing step: {step:?}");
    }
}

#[test]
fn explicit_opt_in_does_not_start_without_owned_kaspad_dependencies() {
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
    assert!(plan.blocked_reason.contains("no clone"));
}

#[test]
fn lightweight_handle_delegates_to_canonical_owner() {
    let handle = KgwInProcNodeOnly::new();
    let plan = handle.plan(KgwInprocStartRequest::default()).unwrap();

    assert!(!handle.is_running());
    assert_eq!(plan.owner_crate, "kaspa-gateway-rk-node");
    assert!(!plan.uses_git_clone);
    assert!(!plan.uses_source_cache);
}

#[test]
fn summary_declares_no_clone_no_source_cache() {
    let summary = kgw_inproc_summary_v1();

    assert!(summary.contains("No clone"));
    assert!(summary.contains("source-cache"));
}
