use candid::CandidType;
use serde::{Deserialize, Serialize};

/// Response for the `delete_file` method.
#[derive(Debug, CandidType, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeleteFileResponse {
    /// File was deleted successfully.
    Ok,
    /// File was not found.
    FileNotFound,
    /// Failed to unindex the file on the orchestrator.
    FailedToRevokeShare(String),
}

impl DeleteFileResponse {
    /// If the response is not [`DeleteFileResponse::Ok`], this function panics with the given string.
    pub fn expect(self, s: &str) -> Self {
        match self {
            DeleteFileResponse::Ok => self,
            DeleteFileResponse::FileNotFound => panic!("File not found: {}", s),
            DeleteFileResponse::FailedToRevokeShare(err) => {
                panic!("Failed to revoke share: {}: {}", s, err)
            }
        }
    }
}
