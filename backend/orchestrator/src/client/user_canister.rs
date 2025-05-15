use candid::Principal;
use ic_cdk::call::{Call, CallResult, Error as CallError};

/// Client for the User canister.
pub struct UserCanisterClient {
    principal: Principal,
}

impl From<Principal> for UserCanisterClient {
    fn from(principal: Principal) -> Self {
        Self { principal }
    }
}

impl UserCanisterClient {
    /// Send a request to the User canister to initialize the alias generator seed.
    pub async fn init_alias_generator_seed(&self) -> CallResult<()> {
        Call::unbounded_wait(self.principal, "init_alias_generator_seed")
            .await
            .map(|_| ())
            .map_err(CallError::from)
    }
}
