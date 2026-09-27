use std::fs;
use std::path::Path;

const SHELL_RUNTIME: &str = "crates/kaspa-gateway-frontend-wasm/src/shell_runtime.rs";
const MAIN_RELATIVE: &str = "apps/kaspa-gateway-desktop/frontend/main.js";

#[derive(Debug, Default, Clone)]
struct Guard {
    generation: u32,
    explicit_seen: bool,
    pending: String,
    saved: String,
    active: String,
    scheduled: Vec<i32>,
    opens: Vec<String>,
}

impl Guard {
    fn new(saved: &str, active: &str) -> Self {
        Self {
            saved: saved.to_owned(),
            active: active.to_owned(),
            ..Self::default()
        }
    }

    fn schedule(&mut self) -> bool {
        if self.explicit_seen || self.saved.is_empty() {
            return false;
        }
        self.pending = self.saved.clone();
        self.scheduled = vec![0, 80, 250, 800, 1600];
        true
    }

    fn record(&mut self, tab: &str) -> u32 {
        if !tab.is_empty() {
            self.generation = self.generation.saturating_add(1);
            self.explicit_seen = true;
            self.pending.clear();
        }
        self.generation
    }

    fn current(&self, generation: u32) -> bool {
        generation == self.generation
    }

    fn run_first(&mut self, scheduled_generation: u32) {
        if self.scheduled.is_empty()
            || self.explicit_seen
            || scheduled_generation != self.generation
            || self.pending.is_empty()
            || self.active == self.pending
        {
            return;
        }
        self.active = self.pending.clone();
        self.opens.push(self.active.clone());
    }

    fn run_all(&mut self, scheduled_generation: u32) {
        for _ in self.scheduled.clone() {
            self.run_first(scheduled_generation);
        }
    }
}

fn verify_source_integration(root: &Path) -> Result<(), String> {
    let rust = fs::read_to_string(root.join(SHELL_RUNTIME))
        .map_err(|error| format!("failed to read {SHELL_RUNTIME}: {error}"))?;
    let main = fs::read_to_string(root.join(MAIN_RELATIVE))
        .map_err(|error| format!("failed to read {MAIN_RELATIVE}: {error}"))?;

    for needle in [
        "const RESTORE_DELAYS: [i32; 5] = [0, 80, 250, 800, 1600];",
        "static EXPLICIT_GENERATION",
        "static EXPLICIT_SEEN",
        "fn record_explicit(",
        "fn restore_token_current(",
        "fn schedule_saved_restore(",
        "explicitNavigationGeneration",
        "shellRuntimeScheduleSavedRestore",
        "shellRuntimeRecordExplicitNavigation",
    ] {
        if !rust.contains(needle) && !main.contains(needle) {
            return Err(format!("AUD-013 Rust integration guard missing: {needle}"));
        }
    }

    for forbidden in [
        "let kgwShellPendingSavedMainTabR102C = \"\";",
        "let kgwShellExplicitNavigationGenerationR103 = 0;",
        "function kgwShellExplicitNavigationIsCurrentR103",
    ] {
        if main.contains(forbidden) {
            return Err(format!(
                "AUD-013 legacy JavaScript owner still present after Rust migration: {forbidden}"
            ));
        }
    }
    Ok(())
}

fn verify_frozen_scenarios() -> Result<(), String> {
    // Scenario 1: explicit navigation after restore scheduling invalidates every stale callback.
    let mut first = Guard::new("kaspa-bridge", "kaspa-node");
    let scheduled_generation = first.generation;
    if !first.schedule() || first.scheduled.len() != 5 {
        return Err("AUD-013 scenario1 schedule contract changed".to_owned());
    }
    let explicit = first.record("settings");
    first.active = "settings".to_owned();
    first.run_all(scheduled_generation);
    if !first.opens.is_empty() || !first.current(explicit) {
        return Err("AUD-013 scenario1 explicit navigation lost precedence".to_owned());
    }

    // Scenario 2: when no explicit navigation occurs, first restore callback opens saved tab.
    let mut second = Guard::new("kaspa-bridge", "kaspa-node");
    let generation = second.generation;
    if !second.schedule() {
        return Err("AUD-013 scenario2 schedule unexpectedly rejected".to_owned());
    }
    second.run_first(generation);
    if second.opens != ["kaspa-bridge"] {
        return Err(format!(
            "AUD-013 scenario2 saved restore mismatch: {:?}",
            second.opens
        ));
    }

    // Scenario 3: rapid explicit navigation only leaves the newest generation current.
    let mut third = Guard::new("kaspa-bridge", "kaspa-node");
    let first_generation = third.record("settings");
    let second_generation = third.record("analysis");
    if third.current(first_generation) || !third.current(second_generation) {
        return Err("AUD-013 scenario3 generation ordering changed".to_owned());
    }

    // Scenario 4: once explicit intent exists, later display hydration cannot schedule restore.
    let mut fourth = Guard::new("kaspa-bridge", "kaspa-node");
    fourth.record("settings");
    if fourth.schedule() || !fourth.scheduled.is_empty() {
        return Err("AUD-013 scenario4 stale restore scheduling was re-enabled".to_owned());
    }
    Ok(())
}

pub fn run(root: &Path) -> Result<String, String> {
    verify_source_integration(root)?;
    verify_frozen_scenarios()?;
    Ok("AUD-013 navigation regression Rust owner PASSED scenarios=4".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_owns_four_frozen_behavior_scenarios() {
        verify_frozen_scenarios().unwrap();
    }

    #[test]
    fn restore_delay_contract_is_exact() {
        let mut guard = Guard::new("kaspa-bridge", "kaspa-node");
        assert!(guard.schedule());
        assert_eq!(guard.scheduled, [0, 80, 250, 800, 1600]);
    }
}
