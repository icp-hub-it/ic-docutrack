use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::FileId;

/// Response for the `upload_file_atomic` method.
#[derive(CandidType, Serialize, Deserialize, Debug, PartialEq)]
pub enum UploadFileAtomicResponse {
    /// The file was uploaded successfully.
    Ok(FileId),
    /// File already exists.
    FileAlreadyExists,
}

impl UploadFileAtomicResponse {
    pub fn unwrap(self) -> FileId {
        match self {
            UploadFileAtomicResponse::Ok(file_id) => file_id,
            e => {
                panic!("Tried to unwrap a {e:?} response")
            }
        }
    }
}
