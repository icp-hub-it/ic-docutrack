use std::collections::BTreeMap;

use candid::Principal;
use did::backend::{
    AliasInfo, BackendInitArgs, FileStatus, GetAliasInfoError, PublicFileMetadata, UploadFileError,FileSharingResponse,UploadFileAtomicRequest
};


use crate::storage::config::Config;
use crate::storage::files::{
    File, FileAliasIndexStorage, FileContent, FileContentsStorage, FileCountStorage,
    FileDataStorage, FileId, FileMetadata, FileSharesStorage, OwnedFilesStorage,
};
use crate::utils::time;

/// API for the backend canister
pub struct Canister;

impl Canister {
    /// Initialize the canister with the given arguments.
    pub fn init(args: BackendInitArgs) {
        Config::set_orbit_station(args.orbit_station);
        Config::set_orchestrator(args.orchestrator);
        Config::set_owner(args.owner);
    }

    pub fn request_file<S: Into<String>>(caller: Principal, request_name: S) -> String {
        let file_id = FileCountStorage::generate_file_id();
        //FIXME: make alias generator work
        let alias = "mock_alias".to_string();
        let file = File {
            metadata: FileMetadata {
                file_name: request_name.into(),
                user_public_key: Config::get_owner_public_key(),
                requester_principal: caller,
                requested_at: time(),
                uploaded_at: None,
            },
            content: FileContent::Pending {
                alias: alias.clone(),
            },
        };
        FileDataStorage::set_file(&file_id, file);
        FileAliasIndexStorage::set_file_id(&alias, &file_id);
        OwnedFilesStorage::add_owned_file(&file_id);

        alias
    }

    pub fn get_requests() -> Vec<PublicFileMetadata> {
        OwnedFilesStorage::get_owned_files()
            .iter()
            .map(|file_id| PublicFileMetadata {
                file_id: *file_id,
                file_name: FileDataStorage::get_file(file_id)
                    .expect("file must exist")
                    .metadata
                    .file_name
                    .clone(),
                shared_with: Self::get_allowed_users(file_id),
                file_status: Self::get_file_status(file_id),
            })
            .collect()
    }

    /// update file
    /// to be triggered by requested file uploads
    pub fn upload_file(
        file_id: FileId,
        file_content: Vec<u8>,
        file_type: String,
        owner_key: [u8; 32],
        num_chunks: u64,
    ) -> Result<(), UploadFileError> {
        let file = FileDataStorage::get_file(&file_id);
        if file.is_none() {
            return Err(UploadFileError::NotRequested);
        }
        let mut file = file.unwrap();
        let shared_keys = BTreeMap::new();

        let alias = match &file.content {
            FileContent::Pending { alias } => {
                let alias = alias.clone();
                if num_chunks == 1 {
                    file.content = FileContent::Uploaded {
                        file_type,
                        owner_key,
                        shared_keys,
                        num_chunks,
                    };
                } else {
                    file.content = FileContent::PartiallyUploaded {
                        file_type,
                        owner_key,
                        shared_keys,
                        num_chunks,
                    };
                }
                file.metadata.uploaded_at = Some(time());
                //persist file
                FileDataStorage::set_file(&file_id, file);

                //add file to the storage
                let chunk_id = 0;
                FileContentsStorage::set_file_contents(&file_id, &chunk_id, file_content);
                alias
            }
            FileContent::Uploaded { .. } | FileContent::PartiallyUploaded { .. } => {
                return Err(UploadFileError::AlreadyUploaded);
            }
        };

        //removing alias from the index
        FileAliasIndexStorage::remove_file_id(&alias);

        Ok(())
    }

    /// Upload file Atomic
    /// to be triggered by owners, no need to request file
    pub fn upload_file_atomic(
        caller: Principal,
        request: UploadFileAtomicRequest,
    ) -> FileId {
        let file_id = FileCountStorage::generate_file_id();
        let content = if request.num_chunks == 1 {
            FileContent::Uploaded {
                file_type: request.file_type,
                owner_key: request.owner_key,
                shared_keys: BTreeMap::new(),
                num_chunks: request.num_chunks,
            }
        } else {
            FileContent::PartiallyUploaded {
                file_type: request.file_type,
                owner_key: request.owner_key,
                shared_keys: BTreeMap::new(),
                num_chunks: request.num_chunks,
            }
        };

        // Aff File to content storage
        let chunk_id = 0;
        FileContentsStorage::set_file_contents(&file_id, &chunk_id, request.content);
        // Add file to the file storage
        let file = File {
            metadata: FileMetadata {
                file_name: request.name,
                user_public_key: Config::get_owner_public_key(),
                requester_principal: caller,
                requested_at: time(),
                uploaded_at: None,
            },
            content,
        };
        FileDataStorage::set_file(&file_id, file);
        
        OwnedFilesStorage::add_owned_file(&file_id);

        file_id
    }

    /// Share file with user
    pub fn share_file(
        user_id: Principal,
        file_id: FileId,
        file_key_encrypted_for_user: [u8; 32],)
        -> FileSharingResponse {


        let mut file = FileDataStorage::get_file(&file_id).unwrap();

        // If uploaded or partially uploaded, Modify File content, add user's decryption key to map
        match &file.content {
            FileContent::Pending { .. } => {
                return FileSharingResponse::PendingError;
            }
            FileContent::Uploaded { shared_keys, num_chunks,owner_key,file_type } => {
                if !shared_keys.contains_key(&user_id) {
                let mut shared_keys = shared_keys.clone();
                shared_keys.insert(user_id, file_key_encrypted_for_user);
                file.content = FileContent::Uploaded {
                    num_chunks : num_chunks.clone(),
                    file_type : file_type.clone(),
                    owner_key : owner_key.clone(),
                    shared_keys,
                };           
             }
            }
            FileContent::PartiallyUploaded {  shared_keys, num_chunks,owner_key,file_type } => {
                if !shared_keys.contains_key(&user_id) {
                let mut shared_keys = shared_keys.clone();
                shared_keys.insert(user_id, file_key_encrypted_for_user);
                file.content = FileContent::Uploaded {
                    num_chunks : num_chunks.clone(),
                    file_type : file_type.clone(),
                    owner_key : owner_key.clone(),
                    shared_keys,
                };
            }
        }
        };
        //persist file
        FileDataStorage::set_file(&file_id, file);
        //add to file shares storage
        FileSharesStorage::set_file_shares(&user_id, vec![file_id]);
        FileSharingResponse::Ok
    }

    /// Share file with users
    pub fn share_file_with_users(
        user_id: Vec<Principal>,
        file_id: FileId,
        file_key_encrypted_for_user: Vec<[u8; 32]>,
    ) {
        for (user, decryption_key) in user_id.iter().zip(file_key_encrypted_for_user.iter()) {
            Self::share_file(*user, file_id, *decryption_key);
        }
    }

    /// Revoke file sharing
    pub fn revoke_file_sharing(user_id: Principal, file_id: FileId){
        // remove file from user shares
        FileSharesStorage::remove_file_shares(&user_id, &file_id);
        // remove user from file shares
        let mut file = FileDataStorage::get_file(&file_id).unwrap();
        match &mut file.content {
            FileContent::Uploaded { shared_keys, .. } => {
                shared_keys.remove(&user_id);
            }
            FileContent::PartiallyUploaded { shared_keys, .. } => {
                shared_keys.remove(&user_id);
            }
            _ => {}
        }
        // persist file
        FileDataStorage::set_file(&file_id, file);
    }

    pub fn get_allowed_users(file_id: &FileId) -> Vec<Principal> {
        FileSharesStorage::get_file_shares_storage()
            .iter()
            .filter(|element| element.1.contains(file_id))
            .map(|(user_principal, _file_vector)| *user_principal)
            .collect()
    }
    pub fn get_file_status(file_id: &FileId) -> FileStatus {
        // unwrap is safe, we know the file exists
        let file = &FileDataStorage::get_file(&file_id).unwrap();
        match &file.content {
            FileContent::Pending { alias } => FileStatus::Pending {
                alias: alias.clone(),
                requested_at: file.metadata.requested_at,
            },
            FileContent::PartiallyUploaded { .. } => FileStatus::PartiallyUploaded,
            FileContent::Uploaded {
                owner_key: own_key, ..
            } => FileStatus::Uploaded {
                uploaded_at: file.metadata.uploaded_at.unwrap(),
                document_key: own_key.clone(),
            },
        }
    }

    pub fn get_shared_files(caller: Principal) -> Vec<PublicFileMetadata> {
        match FileSharesStorage::get_file_shares(&caller) {
            None => vec![],
            Some(file_ids) => file_ids
                .iter()
                .map(|file_id| PublicFileMetadata {
                    file_id: *file_id,
                    file_name: FileDataStorage::get_file(file_id)
                        .expect("file must exist")
                        .metadata
                        .file_name
                        .clone(),
                    shared_with: Self::get_allowed_users(file_id),
                    file_status: Self::get_file_status(file_id),
                })
                .collect(),
        }
    }

    pub fn get_alias_info(alias: String) -> Result<AliasInfo, GetAliasInfoError> {
        let file_id = FileAliasIndexStorage::get_file_id(&alias);
        if file_id.is_none() {
            return Err(GetAliasInfoError::NotFound);
        }
        let file_id = file_id.unwrap();
        let file = FileDataStorage::get_file(&file_id).unwrap();

        Ok(AliasInfo {
            file_id,
            file_name: file.metadata.file_name.clone(),
        })
    }

    // pub fn generate_alias() -> String {

    // }
}

#[cfg(test)]
mod test {
    use candid::Principal;

    use super::*;

    #[test]
    fn test_should_init_canister() {
        let orbit_station = Principal::from_text("rwlgt-iiaaa-aaaaa-aaaaa-cai").unwrap();
        let orchestrator = Principal::from_slice(&[0, 1, 2, 3]);
        let owner = Principal::from_slice(&[4, 5, 6, 7]);
        Canister::init(BackendInitArgs {
            orbit_station,
            orchestrator,
            owner,
        });

        assert_eq!(Config::get_orbit_station(), orbit_station);
        assert_eq!(Config::get_orchestrator(), orchestrator);
        assert_eq!(Config::get_owner(), owner);
    }

    #[test]
    fn test_should_request_file() {
        let caller = Principal::from_slice(&[0, 1, 2, 3]);
        let file_name = "test_file.txt";
        let alias = Canister::request_file(caller, file_name);
        assert_eq!(alias, "mock_alias");
    }

    #[test]
    fn test_should_get_requests() {
        let caller = Principal::from_slice(&[0, 1, 2, 3]);
        let file_name = "test_file.txt";
        Canister::request_file(caller, file_name);
        let requests = Canister::get_requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].file_name, file_name);
    }

    #[test]
    fn test_should_upload_file() {
        let caller = Principal::from_slice(&[0, 1, 2, 3]);
        let file_name = "test_file.txt";
        let alias = Canister::request_file(caller, file_name);
        let file_id = FileAliasIndexStorage::get_file_id(&alias).unwrap();
        let file_content = vec![1, 2, 3];
        let file_type = "text/plain".to_string();
        let owner_key = [0; 32];
        let num_chunks = 1;
        let result = Canister::upload_file(
            file_id,
            file_content.clone(),
            file_type.clone(),
            owner_key,
            num_chunks,
        );
        assert!(result.is_ok());
        let file = FileDataStorage::get_file(&file_id).unwrap();
        assert_eq!(file.content, FileContent::Uploaded {
            file_type,
            owner_key,
            shared_keys: BTreeMap::new(),
            num_chunks,
        });
    }

    #[test]
    fn test_should_upload_file_atomic() {
        let caller = Principal::from_slice(&[0, 1, 2, 3]);
        let file_name = "test_file.txt";
        let file_content = vec![1, 2, 3];
        let file_type = "text/plain".to_string();
        let owner_key = [0; 32];
        let num_chunks = 1;
        let file_id = Canister::upload_file_atomic(
            caller,
            UploadFileAtomicRequest {
                name: file_name.to_string(),
                content: file_content.clone(),
                file_type,
                owner_key,
                num_chunks,
            },
        );
        assert_eq!(file_id, 1);
        
        // Check if the file was uploaded correctly
        let file = FileDataStorage::get_file(&file_id).unwrap();
        assert_eq!(file.content, FileContent::Uploaded {
            file_type: "text/plain".to_string(),
            owner_key,
            shared_keys: BTreeMap::new(),
            num_chunks,
        });
        // Check if the file content was stored correctly
        let file_content_stored = FileContentsStorage::get_file_contents(&file_id, &0).unwrap();
        assert_eq!(file_content_stored, file_content);
    }

    #[test]
    fn test_should_share_a_file() {
        let caller = Principal::from_slice(&[0, 1, 2, 3]);
        let file_name = "test_file.txt";
        let alias = Canister::request_file(caller, file_name);
        let file_id = FileAliasIndexStorage::get_file_id(&alias).unwrap();
        let user_id = Principal::from_slice(&[4, 5, 6, 7]);
        let file_key_encrypted_for_user = [0; 32];
        let result = Canister::share_file(user_id, file_id, file_key_encrypted_for_user);
        assert_eq!(result, FileSharingResponse::PendingError);
        // Upload the file first
        let file_content = vec![1, 2, 3];
        let file_type = "text/plain".to_string();
        let owner_key = [0; 32];
        let num_chunks = 1;
        let res = Canister::upload_file(
            file_id,
            file_content,
            file_type.clone(),
            owner_key,
            num_chunks,
        );
        assert!(res.is_ok());
        // Now share the file
        let result = Canister::share_file(user_id, file_id, file_key_encrypted_for_user);
        assert_eq!(result, FileSharingResponse::Ok);
        let file = FileDataStorage::get_file(&file_id).unwrap();
       
        assert_eq!(file.content, FileContent::Uploaded {
            file_type:file_type.clone(),
            owner_key,
            shared_keys: BTreeMap::from([(user_id, file_key_encrypted_for_user)]),
            num_chunks,
        });
        // Check if the file is shared with the user
        let shared_files = Canister::get_shared_files(user_id);
        assert_eq!(shared_files.len(), 1);
        assert_eq!(shared_files[0].file_id, file_id);

    }

    #[test]
    fn should_share_file_with_users() {
        let caller = Principal::from_slice(&[0, 1, 2, 3]);
        let file_name = "test_file.txt";
        let alias = Canister::request_file(caller, file_name);
        let file_id = FileAliasIndexStorage::get_file_id(&alias).unwrap();
        //upload the file first
        let file_content = vec![1, 2, 3];
        let file_type = "text/plain".to_string();
        let owner_key = [0; 32];
        let num_chunks = 1;
        let res = Canister::upload_file(
            file_id,
            file_content,
            file_type.clone(),
            owner_key,
            num_chunks,
        );
        assert!(res.is_ok());
        // Now share the file with multiple users
        
        let user_ids = vec![
            Principal::from_slice(&[4, 5, 6, 7]),
            Principal::from_slice(&[8, 9, 10, 11]),
        ];
        let file_key_encrypted_for_user = vec![[2; 32], [1; 32]];
        Canister::share_file_with_users(user_ids.clone(), file_id, file_key_encrypted_for_user);
        for user_id in user_ids {
            let shared_files = Canister::get_shared_files(user_id);
            assert_eq!(shared_files.len(), 1);
            assert_eq!(shared_files[0].file_id, file_id);
        }
    }
    #[test]
    fn test_should_revoke_file_sharing() {
        let caller = Principal::from_slice(&[0, 1, 2, 3]);
        let file_name = "test_file.txt";
        let alias = Canister::request_file(caller, file_name);
        let file_id = FileAliasIndexStorage::get_file_id(&alias).unwrap();
        //upload the file first
        let file_content = vec![1, 2, 3];
        let file_type = "text/plain".to_string();
        let owner_key = [0; 32];
        let num_chunks = 1;
        let res = Canister::upload_file(
            file_id,
            file_content,
            file_type.clone(),
            owner_key,
            num_chunks,
        );
        assert!(res.is_ok());
        // Now share the file with  user
        let user_id = Principal::from_slice(&[4, 5, 6, 7]);
        let file_key_encrypted_for_user = [0; 32];
        Canister::share_file(user_id, file_id, file_key_encrypted_for_user);
        // Revoke sharing
        Canister::revoke_file_sharing(user_id, file_id);
        // Check if the user can still access the shared files
        let shared_files = Canister::get_shared_files(user_id);
        assert_eq!(shared_files.len(), 0);
        // check if file has its sharing revoked
        let file = FileDataStorage::get_file(&file_id).unwrap();
        assert_eq!(file.content, FileContent::Uploaded {
            file_type:file_type.clone(),
            owner_key,
            shared_keys: BTreeMap::new(),
            num_chunks,
        });
    }
    // #[test]
    // fn test_should_get_shared_files() {
    //     let caller = Principal::from_slice(&[0, 1, 2, 3]);
    //     let file_name = "test_file.txt";
    //     Canister::request_file(caller, file_name);
    //     let shared_files = Canister::get_shared_files(caller);
    //     assert_eq!(shared_files.len(), 1);
    //     assert_eq!(shared_files[0].file_name, file_name);
    // }
    #[test]
    fn test_should_get_alias_info() {
        let caller = Principal::from_slice(&[0, 1, 2, 3]);
        let file_name = "test_file.txt";
        let alias = Canister::request_file(caller, file_name);
        let alias_info = Canister::get_alias_info(alias.clone());
        assert!(alias_info.is_ok());
        let alias_info = alias_info.unwrap();
        assert_eq!(alias_info.file_name, file_name);
    }
    #[test]
    fn test_should_get_alias_info_not_found() {
        let alias = "non_existent_alias".to_string();
        let alias_info = Canister::get_alias_info(alias);
        assert!(alias_info.is_err());
        assert_eq!(alias_info.unwrap_err(), GetAliasInfoError::NotFound);
    }
}
