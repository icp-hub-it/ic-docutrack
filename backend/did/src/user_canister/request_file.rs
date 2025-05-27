use candid::CandidType;
use serde::{Deserialize, Serialize};

/// Request file result enum.
///
/// In case of success returns [`RequestFileResponse::Ok`] with the alias name.
#[derive(CandidType, Serialize, Deserialize, Debug, PartialEq)]
pub enum RequestFileResponse {
    /// Return the alias name
    Ok(String),
    FileAlreadyExists,
}

impl RequestFileResponse {
    pub fn unwrap(self) -> String {
        match self {
            RequestFileResponse::Ok(file_id) => file_id,
            e => {
                panic!("Tried to unwrap a {e:?} response")
            }
        }
    }
}
