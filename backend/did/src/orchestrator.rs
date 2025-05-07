mod user;
mod whoami;

use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

pub use self::user::{
    GetUsersResponse, MAX_USERNAME_SIZE, PUBKEY_SIZE, PublicKey, PublicUser, SetUserResponse, User,
};
pub use self::whoami::WhoamiResponse;

/// Orchestrator canister init arguments
#[derive(Debug, CandidType, Serialize, Deserialize)]
pub struct OrchestratorInitArgs {
    pub orbit_station: Principal,
}
