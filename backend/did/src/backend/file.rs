
use candid::{CandidType, Principal };
use serde::{Deserialize, Serialize};


#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PublicFileMetadata {
    pub file_id: u64,
    pub file_name: String,
    pub file_status: FileStatus,
    pub shared_with: Vec<Principal>,
}
    

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum FileStatus {
    #[serde(rename = "pending")]
    Pending { alias: String, requested_at: u64 },
    #[serde(rename = "partially_uploaded")]
    PartiallyUploaded,
    #[serde(rename = "uploaded")]
    Uploaded {
        uploaded_at: u64,
        document_key: [u8; 32],
    },
}


#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum GetAliasInfoError {
    #[serde(rename = "not_found")]
    NotFound,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AliasInfo {
    pub file_id: u64,
    pub file_name: String,
    // pub user: PublicUser,
}



#[derive(CandidType, Serialize, Deserialize, Debug, PartialEq)]
pub struct FileData {
    contents: Vec<u8>,
    file_type: String,
    owner_key: Vec<u8>,
    num_chunks: u64,
}

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

#[derive(CandidType, Serialize, Deserialize)]
pub enum UploadFileError {
    #[serde(rename = "not_requested")]
    NotRequested,
    #[serde(rename = "already_uploaded")]
    AlreadyUploaded,
}

#[derive(CandidType, Serialize, Deserialize, Debug, PartialEq)]
pub enum FileSharingResponse {
    #[serde(rename = "pending_error")]
    PendingError,
    #[serde(rename = "permission_error")]
    PermissionError,
    #[serde(rename = "ok")]
    Ok,
}


#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UploadFileRequest {
    pub file_id: u64,
    pub file_content: Vec<u8>,
    pub file_type: String,
    pub owner_key: Vec<u8>,
    pub num_chunks: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UploadFileContinueRequest {
    pub file_id: u64,
    pub chunk_id: u64,
    pub contents: Vec<u8>,
}


