use std::collections::BTreeMap;

use candid::Principal;

use did::backend::{
    BackendInitArgs, FileStatus, GetAliasInfoError, PublicFileMetadata,
};
use did::backend::{AliasInfo,UploadFileError};

use crate::storage::config::Config;
use crate::utils::time;
use crate::storage::files::{
    File, FileAliasIndexStorage, FileContent, FileContentsStorage, FileCountStorage, FileDataStorage, FileId, FileMetadata, FileSharesStorage, OwnedFilesStorage

};



/// API for the backend canister
pub struct Canister;

impl Canister {
    /// Initialize the canister with the given arguments.
    pub fn init(args: BackendInitArgs) {
        Config::set_orbit_station(args.orbit_station);
        Config::set_orchestrator(args.orchestrator);
        Config::set_owner(args.owner);
    }

    pub fn request_file<S: Into<String>>(
        caller: Principal,
        request_name: S) -> String {
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
        .map(|file_id| { 
            PublicFileMetadata {
                file_id: *file_id,
                file_name: FileDataStorage::get_file(file_id)
                    .expect("file must exist")
                    .metadata
                    .file_name
                    .clone(),
                shared_with: Self::get_allowed_users(file_id),
                file_status: Self::get_file_status(file_id),
            }
        })
        .collect()
    }

    /// update file
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

    pub fn get_allowed_users(file_id: &FileId) -> Vec<Principal> {
        FileSharesStorage::get_file_shares_storage()
            .iter()
            .filter(|element| element.1.contains(file_id))
            .map(|(user_principal, _file_vector)| {
                *user_principal.as_principal()
            })
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

    pub fn get_shared_files(caller : Principal) -> Vec<PublicFileMetadata> {
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

    pub fn get_alias_info(alias: String) -> Result<AliasInfo, GetAliasInfoError>{
        let file_id = FileAliasIndexStorage::get_file_id(&alias);
        if file_id.is_none() {
            return Err(GetAliasInfoError::NotFound);
        }
        let file_id = file_id.unwrap();
        let file = FileDataStorage::get_file(&file_id).unwrap();
        
        Ok(AliasInfo {
            file_id: file_id,
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
        Canister::init(BackendInitArgs { orbit_station, orchestrator, owner });
        

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
        let result = Canister::upload_file(file_id, file_content.clone(), file_type.clone(), owner_key, num_chunks);
        assert!(result.is_ok());
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
