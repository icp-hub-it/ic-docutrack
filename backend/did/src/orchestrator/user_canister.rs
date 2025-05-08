use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

/// Response for `user_canister` query
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum UserCanisterResponse {
    /// The user canister is created and ready to use
    Ok(Principal),
    /// The user canister is being created
    CreationPending,
    /// The user canister creation failed; returns the reason
    CreationFailed { reason: String },
    /// The creation is not started yet
    Uninitialized,
    /// Called with an anonymous caller
    AnonymousCaller,
}
