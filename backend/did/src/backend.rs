mod file;

use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

pub use self::file::{
    AliasInfo, FileData, FileDownloadResponse, FileSharingResponse, FileStatus, GetAliasInfoError,
    PublicFileMetadata, UploadFileAtomicRequest, UploadFileContinueRequest, UploadFileError,
    UploadFileRequest,OwnerKey,ENCRYPTION_KEY_SIZE
};

#[derive(Debug, CandidType, Serialize, Deserialize)]
pub struct BackendInitArgs {
    pub orchestrator: Principal,
    pub owner: Principal,
    pub orbit_station: Principal,
}
