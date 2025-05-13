mod file;

use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

pub use self::file::{
    AliasInfo, ENCRYPTION_KEY_SIZE, FileData, FileDownloadResponse, FileSharingResponse,
    FileStatus, GetAliasInfoError, OwnerKey, PublicFileMetadata, UploadFileAtomicRequest,
    UploadFileContinueRequest, UploadFileError, UploadFileRequest,
};

#[derive(Debug, CandidType, Serialize, Deserialize)]
pub struct BackendInitArgs {
    pub orchestrator: Principal,
    pub owner: Principal,
    pub orbit_station: Principal,
}
