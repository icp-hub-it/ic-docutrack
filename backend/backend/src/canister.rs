use std::collections::BTreeMap;

use candid::Principal;
use did::backend::{
    AliasInfo, BackendInitArgs, FileData, FileDownloadResponse, FileSharingResponse, FileStatus,
    GetAliasInfoError, OwnerKey, PublicFileMetadata, UploadFileAtomicRequest,
    UploadFileContinueRequest, UploadFileError,
};
use did::utils::{msg_caller, trap};

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
        Config::set_orchestrator(args.orchestrator);
        Config::set_owner(args.owner);
    }

    /// Request a file
    pub fn request_file<S: Into<String>>(caller: Principal, request_name: S) -> String {
        if caller != Config::get_owner() {
            trap("Only the owner can request a file");
        }
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

    /// Get active requests for the caller
    ///
    // FIXME: maybe rename this function or see in what context is used
    // FIXME: maybe more suitable name is get_owned_files ??
    pub fn get_requests(caller: Principal) -> Vec<PublicFileMetadata> {
        if caller != Config::get_owner() {
            trap("Only the owner can get request a file");
        }
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

    /// upload a file with the given [`FileId`] and file content.
    ///
    /// to be triggered by requested file uploads
    pub fn upload_file(
        file_id: FileId,
        file_content: Vec<u8>,
        file_type: String,
        owner_key: OwnerKey,
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
    pub fn upload_file_atomic(caller: Principal, request: UploadFileAtomicRequest) -> FileId {
        if caller != Config::get_owner() {
            trap("Only the owner can upload a file");
        }
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
                uploaded_at: Some(time()),
            },
            content,
        };
        FileDataStorage::set_file(&file_id, file);

        OwnedFilesStorage::add_owned_file(&file_id);

        file_id
    }

    /// Upload file continue
    pub fn upload_file_continue(request: UploadFileContinueRequest) {
        let file = FileDataStorage::get_file(&request.file_id);
        if file.is_none() {
            return;
        }
        let mut file = file.unwrap();
        let chunk_id = request.chunk_id;

        // Update file content
        //TODO CONSIDER PARTIAL: UPLOAD IN DISORDER
        //TODO maybe add a check to verify all chunks are uploaded before marking as uploaded
        //FIXME: mmm dont like it much. packet order is not guaranteed
        match &file.content {
            FileContent::Uploaded { .. } => {
                return;
            }
            FileContent::PartiallyUploaded {
                num_chunks,
                file_type,
                owner_key,
                shared_keys,
            } => {
                if chunk_id == *num_chunks - 1 {
                    file.content = FileContent::Uploaded {
                        file_type: file_type.clone(),
                        owner_key: *owner_key,
                        shared_keys: shared_keys.clone(),
                        num_chunks: *num_chunks,
                    };
                }
            }
            _ => {}
        }
        // Add file to the content storage
        FileContentsStorage::set_file_contents(&request.file_id, &chunk_id, request.contents);
        // if let FileContent::PartiallyUploaded { num_chunks, .. } = &file.content {
        //     if chunk_id == *num_chunks - 1 {
        //         file.content = FileContent::Uploaded {
        //             file_type: "text/plain".to_string(),
        //             owner_key: [0; 32],
        //             shared_keys: BTreeMap::new(),
        //             num_chunks: *num_chunks,
        //         };
        //     }
        // }
        // Persist file
        FileDataStorage::set_file(&request.file_id, file);
    }

    /// Share file with user
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

        // TODO: index file on the orchestrator

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
        file_key_encrypted_for_user: Vec<OwnerKey>,
    ) {
        for (user, decryption_key) in user_id.iter().zip(file_key_encrypted_for_user.iter()) {
            Self::share_file(*user, file_id, *decryption_key);
        }

        // TODO: index files on the orchestrator
    }

    /// Revoke file sharing
    pub fn revoke_file_sharing(user_id: Principal, file_id: FileId) {
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

        // TODO: revoke files on the orchestrator
    }

    /// Download file
    pub fn download_file(
        file_id: FileId,
        chunk_id: u64,
        caller: Principal,
    ) -> FileDownloadResponse {
        let file = FileDataStorage::get_file(&file_id);
        if file.is_none() {
            return FileDownloadResponse::NotFoundFile;
        }
        let file = file.unwrap();
        // Check if the file is shared with the caller or if the caller is the owner
        let file_c = match &file.content {
            FileContent::Pending { .. } => {
                return FileDownloadResponse::NotUploadedFile;
            }
            FileContent::Uploaded {
                shared_keys,
                num_chunks,
                file_type,
                owner_key,
            } => {
                if !shared_keys.contains_key(&caller) && caller != file.metadata.requester_principal
                {
                    return FileDownloadResponse::PermissionError;
                }
                let num_chunks = *num_chunks;
                let file_type = file_type.clone();
                // if the caller is the owner, use the owner key
                // else use the shared key
                let owner_key = match caller == file.metadata.requester_principal {
                    true => *owner_key,
                    false => *shared_keys.get(&caller).unwrap(),
                };

                (num_chunks, file_type, owner_key)
            }
            FileContent::PartiallyUploaded { .. } => {
                return FileDownloadResponse::NotUploadedFile;
            }
        };
        let contents = FileContentsStorage::get_file_contents(&file_id, &chunk_id);
        if contents.is_none() {
            return FileDownloadResponse::NotFoundFile;
        }
        let contents = contents.unwrap();
        FileDownloadResponse::FoundFile(FileData {
            num_chunks: file_c.0,
            contents,
            file_type: file_c.1,
            owner_key: file_c.2,
        })
    }

    /// Get the list of users that have access to the file by its [`FileId`]
    pub fn get_allowed_users(file_id: &FileId) -> Vec<Principal> {
        FileSharesStorage::get_file_shares_storage()
            .iter()
            .filter(|element| element.1.contains(file_id))
            .map(|(user_principal, _file_vector)| *user_principal)
            .collect()
    }

    /// Get [`FileStatus`] of the file by its [`FileId`]
    pub fn get_file_status(file_id: &FileId) -> FileStatus {
        // unwrap is safe, we know the file exists
        let file = &FileDataStorage::get_file(file_id).unwrap();
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
                document_key: *own_key,
            },
        }
    }

    /// Get the list of files shared with the user by its [`Principal`]
    pub fn get_shared_files(user_id: Principal) -> Vec<PublicFileMetadata> {
        let caller = msg_caller();
        if caller == Principal::anonymous() {
            trap("Anonymous user cannot get shared files");
        }
        match FileSharesStorage::get_file_shares(&user_id) {
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

    /// Get the alias info by its [`String`]
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
}

#[cfg(test)]
mod test {
    use candid::Principal;

    use super::*;

    #[test]
    fn test_should_init_canister() {
        let orchestrator = Principal::from_slice(&[0, 1, 2, 3]);
        let owner = Principal::from_slice(&[4, 5, 6, 7]);
        Canister::init(BackendInitArgs {
            orchestrator,
            owner,
        });

        assert_eq!(Config::_get_orchestrator(), orchestrator);
        assert_eq!(Config::get_owner(), owner);
    }

    #[test]
    fn test_should_request_file() {
        let file_name = "test_file.txt".to_string();
        let caller = init();
        let alias = Canister::request_file(caller, file_name.clone());
        assert_eq!(alias, "mock_alias");
    }

    #[test]
    fn test_should_get_requests() {
        let file_name = "test_file.txt";
        let caller = Principal::from_slice(&[0, 1, 2, 3]);
        Canister::init(BackendInitArgs {
            orchestrator: Principal::from_slice(&[0, 1, 2, 3, 4]),
            owner: caller,
        });
        Canister::request_file(caller, file_name);
        let requests = Canister::get_requests(caller);
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].file_name, file_name);
    }

    #[test]
    fn test_should_upload_file() {
        let file_name = "test_file.txt";
        let caller = Principal::from_slice(&[0, 1, 2, 3]);
        Canister::init(BackendInitArgs {
            orchestrator: Principal::from_slice(&[0, 1, 2, 3]),
            owner: caller,
        });
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
        assert_eq!(
            file.content,
            FileContent::Uploaded {
                file_type,
                owner_key,
                shared_keys: BTreeMap::new(),
                num_chunks,
            }
        );
    }

    #[test]
    fn test_should_upload_file_atomic() {
        let caller = Principal::from_slice(&[0, 1, 2, 3]);
        Canister::init(BackendInitArgs {
            orchestrator: Principal::from_slice(&[0, 1, 2, 3]),
            owner: caller,
        });
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
        assert_eq!(
            file.content,
            FileContent::Uploaded {
                file_type: "text/plain".to_string(),
                owner_key,
                shared_keys: BTreeMap::new(),
                num_chunks,
            }
        );
        // Check if the file content was stored correctly
        let file_content_stored = FileContentsStorage::get_file_contents(&file_id, &0).unwrap();
        assert_eq!(file_content_stored, file_content);
    }

    #[test]
    fn test_should_upload_file_continue() {
        let caller = init();
        let file_name = "test_file.txt";
        let file_content = vec![1, 2, 3];
        let file_type = "text/plain".to_string();
        let owner_key = [0; 32];
        let num_chunks = 2;
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
        assert_eq!(
            file.content,
            FileContent::PartiallyUploaded {
                file_type: "text/plain".to_string(),
                owner_key,
                shared_keys: BTreeMap::new(),
                num_chunks,
            }
        );

        // Upload the second chunk
        Canister::upload_file_continue(UploadFileContinueRequest {
            file_id,
            chunk_id: 1,
            contents: vec![4, 5, 6],
        });

        // Check if the file content was stored correctly
        let file_content_stored_0 = FileContentsStorage::get_file_contents(&file_id, &0).unwrap();
        assert_eq!(file_content_stored_0, vec![1, 2, 3]);

        let file_content_stored_1 = FileContentsStorage::get_file_contents(&file_id, &1).unwrap();
        assert_eq!(file_content_stored_1, vec![4, 5, 6]);
        // Check if the file content was updated correctly
        let file = FileDataStorage::get_file(&file_id).unwrap();
        assert_eq!(
            file.content,
            FileContent::Uploaded {
                file_type: "text/plain".to_string(),
                owner_key,
                shared_keys: BTreeMap::new(),
                num_chunks,
            }
        );
    }

    #[test]
    fn test_should_download_file() {
        let owner = init();
        let file_name = "test_file.txt";
        let alias = Canister::request_file(owner, file_name);
        let file_id = FileAliasIndexStorage::get_file_id(&alias).unwrap();
        let file_content = vec![1, 2, 3];
        let file_type = "text/plain".to_string();
        let owner_key = [0; 32];
        let num_chunks = 1;
        let _ = Canister::upload_file(
            file_id,
            file_content.clone(),
            file_type.clone(),
            owner_key,
            num_chunks,
        );
        // Download the file as the owner
        let result = Canister::download_file(file_id, 0, owner);
        assert_eq!(
            result,
            FileDownloadResponse::FoundFile(FileData {
                contents: file_content.clone(),
                file_type: file_type.clone(),
                owner_key,
                num_chunks
            })
        );
        // Download the file as a shared user
        let user_id = Principal::from_slice(&[4, 5, 6, 7]);
        let file_key_encrypted_for_user = [6; 32];
        Canister::share_file(user_id, file_id, file_key_encrypted_for_user);
        let result = Canister::download_file(file_id, 0, user_id);
        assert_eq!(
            result,
            FileDownloadResponse::FoundFile(FileData {
                contents: file_content,
                file_type,
                owner_key: [6; 32],
                num_chunks
            })
        );
    }

    #[test]
    fn test_should_share_a_file() {
        let file_name = "test_file.txt";
        let caller = init();
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

        assert_eq!(
            file.content,
            FileContent::Uploaded {
                file_type: file_type.clone(),
                owner_key,
                shared_keys: BTreeMap::from([(user_id, file_key_encrypted_for_user)]),
                num_chunks,
            }
        );
        // Check if the file is shared with the user
        let shared_files = Canister::get_shared_files(user_id);
        assert_eq!(shared_files.len(), 1);
        assert_eq!(shared_files[0].file_id, file_id);
    }

    #[test]
    fn should_share_file_with_users() {
        let file_name = "test_file.txt";
        let caller = init();
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
        let file_name = "test_file.txt";
        let caller = init();
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
        assert_eq!(
            file.content,
            FileContent::Uploaded {
                file_type: file_type.clone(),
                owner_key,
                shared_keys: BTreeMap::new(),
                num_chunks,
            }
        );
    }

    #[test]
    fn test_should_get_alias_info() {
        let file_name = "test_file.txt";
        let caller = init();
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

    fn init() -> Principal {
        let caller = Principal::from_slice(&[0, 1, 2, 3]);
        Canister::init(BackendInitArgs {
            orchestrator: Principal::from_slice(&[0, 1, 2, 3]),
            owner: caller,
        });

        caller
    }
}
