use std::collections::HashSet;

use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

use super::FileId;

/// Public file metadata which is stored for the shared files info
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PublicFileMetadata {
    pub file_id: FileId,
    pub file_name: String,
    /// Users the file is shared with
    pub shared_with: HashSet<Principal>,
}
