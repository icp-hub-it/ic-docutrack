use candid::{CandidType, Principal};
use serde::Serialize;

#[derive(CandidType, Serialize)]
pub struct NnsIndexCanisterInitPayload {
    pub ledger_id: Principal,
}
