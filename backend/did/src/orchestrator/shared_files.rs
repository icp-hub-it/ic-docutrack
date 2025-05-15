use std::collections::{HashMap, HashSet};

use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

/// File ID type
pub type FileId = u64;

/// Result for `share_file` and `share_file_with_users` methods
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum ShareFileResponse {
    /// The file was shared successfully
    Ok,
    /// There is no user with the given principal
    NoSuchUser(Principal),
    /// Endpoint was not called by a user canister
    Unauthorized,
}

/// Result for `revoke_share_file` methods
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum RevokeShareFileResponse {
    /// The file was unshared successfully
    Ok,
    /// There is no user with the given principal
    NoSuchUser(Principal),
    /// Endpoint was not called by a user canister
    Unauthorized,
}

/// Result for `shared_files` method
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum SharedFilesResponse {
    /// List of shared files
    SharedFiles(HashMap<Principal, HashSet<FileId>>),
    /// No such user
    NoSuchUser,
    /// Anonymous user
    AnonymousUser,
}
