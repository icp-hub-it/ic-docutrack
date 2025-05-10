use candid::Principal;

use did::backend::{
    BackendInitArgs, FileStatus, GetAliasInfoError, PublicFileMetadata,
};
use did::backend::AliasInfo;

use crate::storage::config::Config;
use crate::utils::time;
use crate::storage::files::{
    FileMetadata,
    File,
    FileContent,
    FileId,
    FileCountStorage,
    OwnedFilesStorage,
    FileDataStorage,
    FileAliasIndexStorage,
    FileSharesStorage,

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
                shared_with: Self::get_allowed_users(*file_id),
                file_status: Self::get_file_status(*file_id),
            }
        })
        .collect()
    }
    pub fn get_allowed_users(file_id: FileId) -> Vec<Principal> {
        FileSharesStorage::get_file_shares_storage()
            .iter()
            .filter(|element| element.1.contains(&file_id))
            .map(|(user_principal, _file_vector)| {
                *user_principal.as_principal()
            })
            .collect()
           
        }
    pub fn get_file_status(file_id: FileId) -> FileStatus {
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
                    shared_with: Self::get_allowed_users(*file_id),
                    file_status: Self::get_file_status(*file_id),
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



}
