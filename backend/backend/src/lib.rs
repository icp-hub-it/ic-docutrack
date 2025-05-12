mod aliases;
mod canister;
mod storage;
mod utils;

use did::backend::{
    AliasInfo, BackendInitArgs, GetAliasInfoError, PublicFileMetadata, UploadFileError,
    UploadFileRequest, FileSharingResponse,
};
use candid::Principal;
use ic_cdk_macros::{init, query, update};
use utils::msg_caller; //post_upgrade, pre_upgrade,

use self::canister::Canister;

#[init]
pub fn init(args: BackendInitArgs) {
    Canister::init(args);
}

#[query]
fn get_requests() -> Vec<PublicFileMetadata> {
    Canister::get_requests()
}

#[query]
fn get_shared_files() -> Vec<PublicFileMetadata> {
    Canister::get_shared_files(msg_caller())
}

#[query]
fn get_alias_info(alias: String) -> Result<AliasInfo, GetAliasInfoError> {
    Canister::get_alias_info(alias)
}

#[update]
fn upload_file(request: UploadFileRequest) -> Result<(), UploadFileError> {

    Canister::upload_file(
        request.file_id,
        request.file_content,
        request.file_type,
        request.owner_key,
        request.num_chunks,
    )
}

// #[update]
// fn upload_file_atomic(request: UploadFileAtomicRequest) -> u64 {
//     with_state_mut(|s| crate::api::upload_file_atomic(ic_cdk::api::msg_caller(), request, s))
// }

// #[update]
// fn upload_file_continue(request: UploadFileContinueRequest) {
//     with_state_mut(|s| crate::api::upload_file_continue(request, s))
// }

#[update]
fn request_file(request_name: String) -> String {
    Canister::request_file(msg_caller(), request_name)
}

// #[query]
// fn download_file(file_id: u64, chunk_id: u64) -> FileDownloadResponse {
//     with_state(|s| crate::api::download_file(s, file_id, chunk_id, ic_cdk::api::msg_caller()))
// }

#[update]
fn share_file(
    user_id: Principal,
    file_id: u64,
    file_key_encrypted_for_user: [u8; 32],
) -> FileSharingResponse {
    Canister::share_file(user_id, file_id, file_key_encrypted_for_user)
}

#[update]
fn share_file_with_users(
    user_id: Vec<Principal>,
    file_id: u64,
    file_key_encrypted_for_user: Vec<[u8; 32]>,
) {

    Canister::share_file_with_users(
        user_id,
        file_id,
        file_key_encrypted_for_user,
    )
}

//TODO review response
#[update]
fn revoke_share(user_id: Principal, file_id: u64) {
    Canister::revoke_file_sharing(user_id, file_id);
}

ic_cdk::export_candid!();
