mod file;

use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

pub use self::file::{FileStatus, GetAliasInfoError, PublicFileMetadata,AliasInfo
};


#[derive(Debug, CandidType ,Serialize , Deserialize)]
pub struct BackendInitArgs {
    pub orchestrator: Principal,
    pub owner: Principal,
    pub orbit_station: Principal,
}
