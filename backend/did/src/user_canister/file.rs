use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

pub const ENCRYPTION_KEY_SIZE: usize = 32;
// User decryption key
pub type OwnerKey = [u8; ENCRYPTION_KEY_SIZE];

/// Public file metadata
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PublicFileMetadata {
    pub file_id: u64,
    pub file_name: String,
    pub file_status: FileStatus,
    pub shared_with: Vec<Principal>,
}

/// File status
/// - `pending`: The file is pending upload.
/// - `partially_uploaded`: The file is partially uploaded.
/// - `uploaded`: The file is fully uploaded and available for download.
/// - `not_found`: The file is not found.
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum FileStatus {
    #[serde(rename = "pending")]
    Pending { alias: String, requested_at: u64 },
    #[serde(rename = "partially_uploaded")]
    PartiallyUploaded,
    #[serde(rename = "uploaded")]
    Uploaded {
        uploaded_at: u64,
        document_key: OwnerKey,
    },
}

/// File alias info error
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum GetAliasInfoError {
    #[serde(rename = "not_found")]
    NotFound,
}

/// File alias info
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AliasInfo {
    pub file_id: u64,
    pub file_name: String,
}

/// File data
#[derive(CandidType, Serialize, Deserialize, Debug, PartialEq)]
pub struct FileData {
    pub contents: Vec<u8>,
    pub file_type: String,
    pub owner_key: OwnerKey,
    pub num_chunks: u64,
}

/// Download for file download
#[derive(CandidType, Serialize, Deserialize, PartialEq, Debug)]
pub enum FileDownloadResponse {
    #[serde(rename = "not_found_file")]
    NotFoundFile,
    #[serde(rename = "not_uploaded_file")]
    NotUploadedFile,
    #[serde(rename = "permission_error")]
    PermissionError,
    #[serde(rename = "found_file")]
    FoundFile(FileData),
}

/// File upload error
/// - `not_requested`: The file is not requested.
/// - `already_uploaded`: The file is already uploaded.
#[derive(CandidType, Serialize, Deserialize)]
pub enum UploadFileError {
    #[serde(rename = "not_requested")]
    NotRequested,
    #[serde(rename = "already_uploaded")]
    AlreadyUploaded,
}

/// File upload response
/// - `pending_error`: The file is pending upload.
/// - `permission_error`: The file is not shared with the user.
/// - `file_not_found`: The file is not found.
/// - `ok`: The file is uploaded successfully.
#[derive(CandidType, Serialize, Deserialize, Debug, PartialEq)]
pub enum FileSharingResponse {
    #[serde(rename = "pending_error")]
    PendingError,
    #[serde(rename = "permission_error")]
    PermissionError,
    #[serde(rename = "file_not_found")]
    FileNotFound,
    #[serde(rename = "ok")]
    Ok,
}

/// File upload request
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UploadFileRequest {
    pub file_id: u64,
    pub file_content: Vec<u8>,
    pub file_type: String,
    pub owner_key: OwnerKey,
    pub num_chunks: u64,
}

/// File upload atomic request
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UploadFileAtomicRequest {
    pub name: String,
    pub content: Vec<u8>,
    pub owner_key: OwnerKey,
    pub file_type: String,
    pub num_chunks: u64,
}

/// File upload continue request
/// This is used to send a chunk of the file to the canister.
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UploadFileContinueRequest {
    pub file_id: u64,
    pub chunk_id: u64,
    pub contents: Vec<u8>,
}
