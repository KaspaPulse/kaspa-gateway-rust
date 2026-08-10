use crate::kgw_inproc_owner::{KgwInprocPlan, KgwInprocStartRequest, build_kgw_inproc_plan};

/// Stable status for the main app/runtime layer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KgwInProcNodeOnlyStatus {
    /// KGW style mechanism has been ported as the canonical owner contract.
    KgwMechanismPorted,

    /// Real runtime is not currently started.
    NotRunning,
}

/// Lightweight handle used by Tauri/runtime code.
///
/// This handle does not spawn a process, does not clone repositories, and does
/// not require a local source cache. It delegates planning to the canonical
/// `kgw_inproc_owner` module.
#[derive(Clone, Debug, Default)]
pub struct KgwInProcNodeOnly;

impl KgwInProcNodeOnly {
    pub fn new() -> Self {
        Self
    }

    pub fn is_running(&self) -> bool {
        false
    }

    pub fn status(&self) -> KgwInProcNodeOnlyStatus {
        KgwInProcNodeOnlyStatus::KgwMechanismPorted
    }

    pub fn ownership_note(&self) -> &'static str {
        "KGW style InProc node mechanism is owned by crates/kaspa-gateway-rk-node::kgw_inproc_owner; no clone, no source-cache, no external executable."
    }

    pub fn plan(
        &self,
        request: KgwInprocStartRequest,
    ) -> Result<KgwInprocPlan, crate::kgw_inproc_owner::KgwInprocError> {
        build_kgw_inproc_plan(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_handle_reports_kgw_mechanism_ported() {
        let node = KgwInProcNodeOnly::new();

        assert!(!node.is_running());
        assert_eq!(node.status(), KgwInProcNodeOnlyStatus::KgwMechanismPorted);
        assert!(node.ownership_note().contains("no clone"));
        assert!(node.ownership_note().contains("no source-cache"));
    }

    #[test]
    fn main_handle_returns_inproc_plan() {
        let node = KgwInProcNodeOnly::new();
        let plan = node.plan(KgwInprocStartRequest::default()).unwrap();

        assert_eq!(plan.owner_crate, "kaspa-gateway-rk-node");
        assert!(!plan.uses_git_clone);
        assert!(!plan.uses_source_cache);
    }
}
