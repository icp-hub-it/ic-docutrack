use candid::Principal;
use did::orchestrator::FileId;
use did::user_canister::{FileSharingResponse, OwnerKey};

use crate::storage::files::{FileContent, FileDataStorage, FileSharesStorage};

/// Canister share file logic.
pub struct CanisterShareFile;

impl CanisterShareFile {
    /// Do share a file on the canister storage.
    pub fn share_file(
        user_id: Principal,
        file_id: FileId,
        file_key_encrypted_for_user: OwnerKey,
    ) -> FileSharingResponse {
        let mut file = FileDataStorage::get_file(&file_id).unwrap();

        // If uploaded or partially uploaded, Modify File content, add user's decryption key to map
        match &file.content {
            FileContent::Pending { .. } => {
                return FileSharingResponse::PendingError;
            }
            FileContent::Uploaded {
                shared_keys,
                num_chunks,
                owner_key,
                file_type,
            } => {
                if !shared_keys.contains_key(&user_id) {
                    let mut shared_keys = shared_keys.clone();
                    shared_keys.insert(user_id, file_key_encrypted_for_user);
                    file.content = FileContent::Uploaded {
                        num_chunks: *num_chunks,
                        file_type: file_type.clone(),
                        owner_key: *owner_key,
                        shared_keys,
                    };
                }
            }
            FileContent::PartiallyUploaded {
                shared_keys,
                num_chunks,
                owner_key,
                file_type,
            } => {
                if !shared_keys.contains_key(&user_id) {
                    let mut shared_keys = shared_keys.clone();
                    shared_keys.insert(user_id, file_key_encrypted_for_user);
                    file.content = FileContent::Uploaded {
                        num_chunks: *num_chunks,
                        file_type: file_type.clone(),
                        owner_key: *owner_key,
                        shared_keys,
                    };
                }
            }
        };

        //persist file
        FileDataStorage::set_file(&file_id, file);

        //add to file shares storage
        FileSharesStorage::share(&user_id, vec![file_id]);

        FileSharingResponse::Ok
    }
}
