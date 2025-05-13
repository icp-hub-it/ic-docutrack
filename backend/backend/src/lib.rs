mod aliases;
mod canister;
mod storage;
mod utils;

use candid::Principal;
use did::{backend::{
    AliasInfo, BackendInitArgs, FileDownloadResponse, FileSharingResponse, GetAliasInfoError, OwnerKey, PublicFileMetadata, UploadFileAtomicRequest, UploadFileContinueRequest, UploadFileError, UploadFileRequest
}, orchestrator::PublicKey};
use ic_cdk_macros::{init, query, update};
use storage::config::Config;
use utils::msg_caller;

use self::canister::Canister;

#[init]
pub fn init(args: BackendInitArgs) {
    Canister::init(args);
}

//done
#[query]
fn public_key() -> PublicKey{
    Config::get_owner_public_key()
}
//done
#[update]
fn set_public_key(public_key: PublicKey){
    Config::set_owner_public_key(public_key);
}
//done
#[query]
fn get_requests() -> Vec<PublicFileMetadata> {
    Canister::get_requests()
}

#[query]
fn get_shared_files(user_id: Principal) -> Vec<PublicFileMetadata> {

    Canister::get_shared_files(user_id)
}
//done
#[query]
fn get_alias_info(alias: String) -> Result<AliasInfo, GetAliasInfoError> {
    Canister::get_alias_info(alias)
}

//done
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

//done
#[update]
fn upload_file_atomic(request: UploadFileAtomicRequest) -> u64 {
    Canister::upload_file_atomic(msg_caller(), request)
}

//done
#[update]
fn upload_file_continue(request: UploadFileContinueRequest) {
    Canister::upload_file_continue(request)
}
//done
#[update]
fn request_file(request_name: String) -> String {
    Canister::request_file(request_name)
}
//done
#[query]
fn download_file(file_id: u64, chunk_id: u64) -> FileDownloadResponse {
    Canister::download_file(file_id, chunk_id, msg_caller())
}

#[update]
fn share_file(
    user_id: Principal,
    file_id: u64,
    file_key_encrypted_for_user: OwnerKey,
) -> FileSharingResponse {
    Canister::share_file(user_id, file_id, file_key_encrypted_for_user)
}

#[update]
fn share_file_with_users(
    user_id: Vec<Principal>,
    file_id: u64,
    file_key_encrypted_for_user: Vec<OwnerKey>,
) {
    Canister::share_file_with_users(user_id, file_id, file_key_encrypted_for_user)
}

//TODO review response
#[update]
fn revoke_share(user_id: Principal, file_id: u64) {
    Canister::revoke_file_sharing(user_id, file_id);
}

ic_cdk::export_candid!();
