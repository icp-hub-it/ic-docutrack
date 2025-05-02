use candid::{CandidType, Principal};
use serde::Serialize;

#[derive(Debug, CandidType, Serialize)]
pub struct CyclesCanisterInitPayload {
    pub ledger_canister_id: Option<Principal>,
    pub governance_canister_id: Option<Principal>,
    pub minting_account_id: Option<String>,
    pub last_purged_notification: Option<u64>,
    pub exchange_rate_canister: Option<ExchangeRateCanister>,
    pub cycles_ledger_canister_id: Option<Principal>,
}

#[derive(Debug, CandidType, Serialize)]
pub enum ExchangeRateCanister {
    Set(Principal),
}

#[derive(CandidType, Serialize)]
pub struct SetAuthorizedSubnetworkListArgs {
    pub who: Option<Principal>,
    pub subnets: Vec<Principal>,
}

#[derive(CandidType, Serialize)]
pub enum UpdateSubnetTypeArgs {
    Add(String),
    //Remove(String),
}

#[derive(CandidType, Serialize)]
pub struct SubnetListWithType {
    pub subnets: Vec<Principal>,
    pub subnet_type: String,
}

#[derive(CandidType, Serialize)]
pub enum ChangeSubnetTypeAssignmentArgs {
    Add(SubnetListWithType),
}
