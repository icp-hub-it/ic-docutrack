use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

/// Orchestrator canister init arguments
#[derive(Debug, CandidType, Serialize, Deserialize)]
pub struct OrchestratorInitArgs {
    pub orbit_station: Principal,
}
