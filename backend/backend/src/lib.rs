mod aliases;
mod canister;
mod storage;
mod utils;

use did::backend::BackendInitArgs;
// use candid::Principal;
use ic_cdk_macros::{init, query, update};
use utils::msg_caller; //post_upgrade, pre_upgrade,

use self::canister::Canister;
use did::backend::{
   PublicFileMetadata,
   GetAliasInfoError, 
   AliasInfo,
};


// use self::storage::config::Config;


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
    // with_state(|s| crate::api::get_shared_files(s, ic_cdk::api::msg_caller()))
    Canister::get_shared_files(msg_caller())
}

#[query]
fn get_alias_info(alias: String) -> Result<AliasInfo, GetAliasInfoError> {
    // with_state(|s| crate::api::get_alias_info(s, alias))
    Canister::get_alias_info(alias)
}

// #[update]
// fn upload_file(request: UploadFileRequest) -> Result<(), UploadFileError> {
//     with_state_mut(|s| {
//         crate::api::upload_file(
//             request.file_id,
//             request.file_content,
//             request.file_type,
//             request.owner_key,
//             request.num_chunks,
//             s,
//         )
//     })
// }

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
    // with_state_mut(|s| crate::api::request_file(ic_cdk::api::msg_caller(), request_name, s))
    Canister::request_file(msg_caller(), request_name)
}

// #[query]
// fn download_file(file_id: u64, chunk_id: u64) -> FileDownloadResponse {
//     with_state(|s| crate::api::download_file(s, file_id, chunk_id, ic_cdk::api::msg_caller()))
// }

// #[update]
// fn share_file(
//     user_id: Principal,
//     file_id: u64,
//     file_key_encrypted_for_user: Vec<u8>,
// ) -> FileSharingResponse {
//     with_state_mut(|s| {
//         crate::api::share_file(
//             s,
//             ic_cdk::api::msg_caller(),
//             user_id,
//             file_id,
//             file_key_encrypted_for_user,
//         )
//     })
// }

// #[update]
// fn share_file_with_users(
//     user_id: Vec<Principal>,
//     file_id: u64,
//     file_key_encrypted_for_user: Vec<Vec<u8>>,
// ) {
//     with_state_mut(|s| {
//         for (id, key) in user_id.iter().zip(file_key_encrypted_for_user.iter()) {
//             crate::api::share_file(s, ic_cdk::api::msg_caller(), *id, file_id, key.clone());
//         }
//     });
// }

// #[update]
// fn revoke_share(user_id: Principal, file_id: u64) -> FileSharingResponse {
//     with_state_mut(|s| crate::api::revoke_share(s, ic_cdk::api::msg_caller(), user_id, file_id))
// }

// pub mod api;
// mod memory;
// mod upgrade;

// use std::cell::RefCell;
// use std::collections::BTreeMap;
// use std::ops::Bound::{Excluded, Included};


// use ic_stable_structures::{StableBTreeMap};
// use memory::Memory;
// use serde::{Deserialize, Serialize};
// use did::StorableFileIdVec;

// use crate::aliases::{AliasGenerator, Randomness};
// use crate::api::UploadFileAtomicRequest;

// thread_local! {
//     /// Initialize the state randomness with the current time.
//     static STATE: RefCell<State> = RefCell::new(State::new(&get_randomness_seed()[..]));
// }

// #[derive(Serialize, Deserialize)]
// pub struct State {
//     // Keeps track of how many files have been requested so far
//     // and is used to assign IDs to newly requested files.
//     file_count: u64,
 
//     /// Mapping between file IDs and file information.
//     pub file_data: BTreeMap<u64, File>,

//     /// Mapping between file aliases (randomly generated links) and file ID.
//     pub file_alias_index: BTreeMap<String, u64>,
    
    
//     /// Mapping between a user's principal and the list of files that are shared with them.
//     // #[serde(skip, default = "init_file_shares")]
//     // pub file_shares: StableBTreeMap<Principal, StorableFileIdVec, Memory>,

//     /// The contents of the file (stored in stable memory).
//     #[serde(skip, default = "init_file_contents")]
//     pub file_contents: StableBTreeMap<(FileId, ChunkId), Vec<u8>, Memory>,

//     // Generates aliases for file requests.
//     #[serde(skip, default = "init_alias_generator")]
//     alias_generator: AliasGenerator,
// }

// impl State {
//     pub(crate) fn generate_file_id(&mut self) -> u64 {
//         // The file ID is an auto-incrementing integer.

//         let file_id = self.file_count;
//         self.file_count += 1;
//         file_id
//     }

//     fn new(rand_seed: &[u8]) -> Self {
//         Self {
//             file_count: 0,
//             file_data: BTreeMap::new(),
//             file_alias_index: BTreeMap::new(),
//             file_shares: init_file_shares(),
//             alias_generator: AliasGenerator::new(Randomness::try_from(rand_seed).unwrap()),
//             file_contents: init_file_contents(),
//         }
//     }

//     /// Returns the number of uploaded chunks for the given file id
//     pub(crate) fn num_chunks_uploaded(&self, file_id: u64) -> u64 {
//         self.file_contents
//             .range((Included((file_id, 0u64)), Excluded(((file_id + 1), 0u64))))
//             .count() as u64
//     }
// }

// impl Default for State {
//     fn default() -> Self {
//         State::new(vec![0; 32].as_slice())
//     }
// }

// /// A helper method to read the state.
// ///
// /// Precondition: the state is already initialized.
// pub fn with_state<R>(f: impl FnOnce(&State) -> R) -> R {
//     STATE.with(|cell| f(&cell.borrow()))
// }

// /// A helper method to mutate the state.
// ///
// /// Precondition: the state is already initialized.
// pub fn with_state_mut<R>(f: impl FnOnce(&mut State) -> R) -> R {
//     STATE.with(|cell| f(&mut cell.borrow_mut()))
// }

// /// Returns an unused file alias.
// pub fn generate_alias() -> String {
//     with_state_mut(|s| s.alias_generator.next())
// }


// #[cfg(target_arch = "wasm32")]
// pub fn get_time() -> u64 {
//     ic_cdk::api::time()
// }

// #[cfg(not(target_arch = "wasm32"))]
// pub fn get_time() -> u64 {
//     // This is used only in tests and we need a fixed value we can test against.
//     12345
// }

// fn get_randomness_seed() -> Vec<u8> {
//     // this is an array of u8 of length 8.
//     let time_seed = ic_cdk::api::time().to_be_bytes();
//     // we need to extend this to an array of size 32 by adding to it an array of size 24 full of 0s.
//     let zeroes_arr: [u8; 24] = [0; 24];
//     [&time_seed[..], &zeroes_arr[..]].concat()
// }

// fn init_alias_generator() -> AliasGenerator {
//     AliasGenerator::new(Randomness::try_from(get_randomness_seed().as_slice()).unwrap())
// }

// fn init_file_contents() -> StableBTreeMap<(FileId, ChunkId), Vec<u8>, Memory> {
//     StableBTreeMap::init(crate::memory::get_file_contents_memory())
// }

// fn init_file_shares() -> StableBTreeMap<Principal, StorableFileIdVec, Memory> {
//     StableBTreeMap::init(crate::memory::get_file_shares_memory())
// }


/// GetRandom fixup to allow getrandom compilation.
/// A getrandom implementation that always fails
///
/// This is a workaround for the fact that the `getrandom` crate does not compile
/// for the `wasm32-unknown-ic` target. This is a dummy implementation that always
/// fails with `Error::UNSUPPORTED`.
// #[unsafe(no_mangle)]
// unsafe extern "Rust" fn __getrandom_v03_custom(
//     _dest: *mut u8,
//     _len: usize,
// ) -> Result<(), getrandom::Error> {
//     Err(getrandom::Error::UNSUPPORTED)
// }



ic_cdk::export_candid!();



