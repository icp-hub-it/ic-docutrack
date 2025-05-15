use candid::Principal;
use did::backend::{
    AliasInfo, FileDownloadResponse, FileSharingResponse, GetAliasInfoError, OwnerKey,
    PublicFileMetadata, UploadFileAtomicRequest, UploadFileContinueRequest, UploadFileError,
    UploadFileRequest,
};
use did::orchestrator::PublicKey;

use super::PocketIcTestEnv;
use crate::TestEnv as _;
use crate::actor::admin;

pub struct BackendClient<'a> {
    pic: &'a PocketIcTestEnv,
}

impl<'a> From<&'a PocketIcTestEnv> for BackendClient<'a> {
    fn from(pic: &'a PocketIcTestEnv) -> Self {
        Self { pic }
    }
}

impl BackendClient<'_> {
    pub async fn public_key(&self, caller: Principal) -> PublicKey {
        self.pic
            .query::<PublicKey>(self.pic.backend(), caller, "public_key", vec![])
            .await
            .expect("Failed to get public key")
    }

    pub async fn set_public_key(&self, public_key: PublicKey) {
        let payload = candid::encode_args((public_key,)).unwrap();
        self.pic
            .update::<()>(self.pic.backend(), admin(), "set_public_key", payload)
            .await
            .expect("Failed to set public key")
    }

    pub async fn get_requests(&self, caller: Principal) -> Vec<PublicFileMetadata> {
        let payload = candid::encode_args(()).unwrap();
        self.pic
            .query::<Vec<PublicFileMetadata>>(self.pic.backend(), caller, "get_requests", payload)
            .await
            .expect("Failed to get requests")
    }

    pub async fn get_shared_files(&self, caller: Principal) -> Vec<PublicFileMetadata> {
        let payload = candid::encode_args(()).unwrap();
        self.pic
            .query::<Vec<PublicFileMetadata>>(
                self.pic.backend(),
                caller,
                "get_shared_files",
                payload,
            )
            .await
            .expect("Failed to get shared files")
    }

    pub async fn get_alias_info(
        &self,
        alias: String,
        caller: Principal,
    ) -> Result<AliasInfo, GetAliasInfoError> {
        let payload = candid::encode_args((alias,)).unwrap();
        self.pic
            .query::<Result<AliasInfo, GetAliasInfoError>>(
                self.pic.backend(),
                caller,
                "get_alias_info",
                payload,
            )
            .await
            .expect("Failed to get alias info")
    }

    pub async fn upload_file(
        &self,
        request: UploadFileRequest,
        caller: Principal,
    ) -> Result<(), UploadFileError> {
        let payload = candid::encode_args((request,)).unwrap();
        self.pic
            .update::<Result<(), UploadFileError>>(
                self.pic.backend(),
                caller,
                "upload_file",
                payload,
            )
            .await
            .expect("Failed to upload file")
    }

    pub async fn upload_file_atomic(
        &self,
        request: UploadFileAtomicRequest,
        caller: Principal,
    ) -> u64 {
        let payload = candid::encode_args((request,)).unwrap();
        self.pic
            .update::<u64>(self.pic.backend(), caller, "upload_file_atomic", payload)
            .await
            .expect("Failed to upload file atomically")
    }

    pub async fn upload_file_continue(
        &self,
        request: UploadFileContinueRequest,
        caller: Principal,
    ) {
        let payload = candid::encode_args((request,)).unwrap();
        self.pic
            .update::<()>(self.pic.backend(), caller, "upload_file_continue", payload)
            .await
            .expect("Failed to continue file upload")
    }

    pub async fn request_file(&self, request_name: String, caller: Principal) -> String {
        let payload = candid::encode_args((request_name,)).unwrap();
        self.pic
            .update::<String>(self.pic.backend(), caller, "request_file", payload)
            .await
            .expect("Failed to request file")
    }

    pub async fn download_file(
        &self,
        file_id: u64,
        chunk_id: u64,
        caller: Principal,
    ) -> FileDownloadResponse {
        let payload = candid::encode_args((file_id, chunk_id)).unwrap();
        self.pic
            .query::<FileDownloadResponse>(self.pic.backend(), caller, "download_file", payload)
            .await
            .expect("Failed to download file")
    }

    pub async fn share_file(
        &self,
        file_id: u64,
        user_id: Principal,
        file_key_encrypted_for_user: OwnerKey,
        caller: Principal,
    ) -> FileSharingResponse {
        let payload = candid::encode_args((user_id, file_id, file_key_encrypted_for_user)).unwrap();
        self.pic
            .update::<FileSharingResponse>(self.pic.backend(), caller, "share_file", payload)
            .await
            .expect("Failed to share file")
    }

    pub async fn share_file_with_users(
        &self,
        user_id: Vec<Principal>,
        file_id: u64,
        file_key_encrypted_for_user: Vec<OwnerKey>,
        caller: Principal,
    ) {
        let payload = candid::encode_args((user_id, file_id, file_key_encrypted_for_user)).unwrap();
        self.pic
            .update::<()>(self.pic.backend(), caller, "share_file_with_users", payload)
            .await
            .expect("Failed to share file with users")
    }

    pub async fn revoke_share(&self, user_id: Principal, file_id: u64, caller: Principal) {
        let payload = candid::encode_args((user_id, file_id)).unwrap();
        self.pic
            .update::<()>(self.pic.backend(), caller, "revoke_share", payload)
            .await
            .expect("Failed to revoke share")
    }
}
