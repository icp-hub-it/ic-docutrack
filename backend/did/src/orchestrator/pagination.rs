use candid::CandidType;
use serde::{Deserialize, Serialize};

/// Pagination struct for paginated responses
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct Pagination {
    /// The number of items to skip
    pub offset: u64,
    /// The number of items to return
    pub limit: u64,
}
