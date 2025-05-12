mod create_state;
use std::cell::RefCell;
use std::collections::HashMap;

use candid::Principal;
use did::{StorableFileIdVec, StorablePrincipal};
// use crate::aliases::{AliasGenerator, Randomness};

// use did::backend::File;
// use did::backend::{FileId, ChunkId};
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, StableVec};

pub use self::create_state::{ChunkId, File, FileContent, FileId, FileMetadata, OwnerKey};
use crate::storage::memory::{
    /*ALIAS_GENERATOR_MEMORY_ID ,*/ FILE_ALIAS_INDEX_MEMORY_ID, FILE_CONTENTS_MEMORY_ID,
    FILE_COUNT_MEMORY_ID, FILE_DATA_MEMORY_ID, FILE_SHARES_MEMORY_ID, MEMORY_MANAGER,
    OWNED_FILES_MEMORY_ID,
};

type ContentTuple = (FileId, ChunkId);
//
// Alias generator
// Generates aliases for file requests.
//     static ALIAS_GENERATOR: RefCell<StableCell<AliasGenerator, VirtualMemory<DefaultMemoryImpl>>> =
//         RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(ALIAS_GENERATOR_MEMORY_ID)), AliasGenerator::new(Randomness::try_from(rand_seed).unwrap())).unwrap())
//   ;

/////
thread_local! {
  /// File count incrementer
  static FILE_COUNT: RefCell<StableCell<u64, VirtualMemory<DefaultMemoryImpl>>> =
      RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(FILE_COUNT_MEMORY_ID)), 0).unwrap()
  );

  /// Owned files storage vector
  /// Vector of available file IDs.
  static OWNED_FILES_STORAGE: RefCell<StableVec<FileId, VirtualMemory<DefaultMemoryImpl>>> =
      RefCell::new(StableVec::new(MEMORY_MANAGER.with(|mm| mm.get(OWNED_FILES_MEMORY_ID))).unwrap()
  );

  /// File data storage map
  /// Mapping between file IDs and file information.
  static FILE_DATA_STORAGE: RefCell<StableBTreeMap<FileId, File, VirtualMemory<DefaultMemoryImpl>>> =
      RefCell::new(StableBTreeMap::new(MEMORY_MANAGER.with(|mm| mm.get(FILE_DATA_MEMORY_ID)))
  );

  /// File alias index storage map
  /// Mapping between file aliases and file IDs.
  static FILE_ALIAS_INDEX_STORAGE: RefCell<StableBTreeMap<String, FileId, VirtualMemory<DefaultMemoryImpl>>> =
      RefCell::new(StableBTreeMap::new(MEMORY_MANAGER.with(|mm| mm.get(FILE_ALIAS_INDEX_MEMORY_ID)))
  );

  /// File shares storage map
  /// Mapping between a user's principal and the list of files that are shared with them.
  static FILE_SHARES_STORAGE: RefCell<StableBTreeMap<StorablePrincipal, StorableFileIdVec, VirtualMemory<DefaultMemoryImpl>>> =
      RefCell::new(StableBTreeMap::new(MEMORY_MANAGER.with(|mm| mm.get(FILE_SHARES_MEMORY_ID)))
  );
  /// The contents of the file (stored in stable memory).
  static FILE_CONTENTS_STORAGE: RefCell<StableBTreeMap<ContentTuple, Vec<u8>, VirtualMemory<DefaultMemoryImpl>>> =
      RefCell::new(StableBTreeMap::new(MEMORY_MANAGER.with(|mm| mm.get(FILE_CONTENTS_MEMORY_ID)))
  );



}
/// Accessor to the file count
fn _with_file_count<T, F>(f: F) -> T
where
    F: FnOnce(&StableCell<u64, VirtualMemory<DefaultMemoryImpl>>) -> T,
{
    FILE_COUNT.with_borrow_mut(|file_count| f(file_count))
}
/// Immutable accessor to the file count
fn _with_file_count_value<T, F>(f: F) -> T
where
    F: FnOnce(&u64) -> T,
{
    FILE_COUNT.with_borrow(|file_count| f(file_count.get()))
}

/// Accessor to the owned files storage
/// Vector of available file IDs.
fn with_owned_files_storage<T, F>(f: F) -> T
where
    F: FnOnce(&StableVec<FileId, VirtualMemory<DefaultMemoryImpl>>) -> T,
{
    OWNED_FILES_STORAGE.with_borrow_mut(|owned_files| f(owned_files))
}
/// Immutable accessor to the owned files
fn with_owned_files<F>(f: F) -> Vec<u64>
where
    F: Fn(FileId) -> u64,
{
    OWNED_FILES_STORAGE.with_borrow(|owned_files| owned_files.iter().map(f).collect::<Vec<u64>>())
}

/// Accessor to the file data storage
fn _with_file_data_storage<T, F>(f: F) -> T
where
    F: FnOnce(&StableBTreeMap<FileId, File, VirtualMemory<DefaultMemoryImpl>>) -> T,
{
    FILE_DATA_STORAGE.with_borrow(|file_data| f(file_data))
}
/// Immutable accessor to the file data
fn with_file_data<T, F>(file_id: &FileId, f: F) -> Option<T>
where
    F: FnOnce(File) -> T,
{
    FILE_DATA_STORAGE.with_borrow(|file_data| file_data.get(file_id).map(f))
}
/// Accessor to the file alias index storage
fn _with_file_alias_index_storage<T, F>(f: F) -> T
where
    F: FnOnce(&StableBTreeMap<String, FileId, VirtualMemory<DefaultMemoryImpl>>) -> T,
{
    FILE_ALIAS_INDEX_STORAGE.with_borrow(|file_alias_index| f(file_alias_index))
}
/// Immutable accessor to the file alias index
fn with_file_alias_index<T, F>(alias: &String, f: F) -> Option<T>
where
    F: FnOnce(FileId) -> T,
{
    FILE_ALIAS_INDEX_STORAGE.with_borrow(|file_alias_index| file_alias_index.get(alias).map(f))
}
/// Accessor to the file shares storage
fn with_file_shares_storage<T, F>(f: F) -> T
where
    F: FnOnce(
        &StableBTreeMap<StorablePrincipal, StorableFileIdVec, VirtualMemory<DefaultMemoryImpl>>,
    ) -> T,
{
    FILE_SHARES_STORAGE.with_borrow(|file_shares| f(file_shares))
}
/// Immutable accessor to the file shares
fn with_file_shares<T, F>(principal: &StorablePrincipal, f: F) -> Option<T>
where
    F: FnOnce(StorableFileIdVec) -> T,
{
    FILE_SHARES_STORAGE.with_borrow(|file_shares| file_shares.get(principal).map(f))
}
/// Accessor to the file contents storage
fn _with_file_contents_storage<T, F>(f: F) -> T
where
    F: FnOnce(&StableBTreeMap<(FileId, ChunkId), Vec<u8>, VirtualMemory<DefaultMemoryImpl>>) -> T,
{
    FILE_CONTENTS_STORAGE.with_borrow(|file_contents| f(file_contents))
}
/// Immutable accessor to the file contents
fn with_file_contents<T, F>(file_id: &FileId, chunk_id: &ChunkId, f: F) -> Option<T>
where
    F: FnOnce(Vec<u8>) -> T,
{
    FILE_CONTENTS_STORAGE
        .with_borrow(|file_contents| file_contents.get(&(*file_id, *chunk_id)).map(f))
}

// // Accesor to Alias generator
// fn with_alias_generator<T, F>(f: F) -> T
// where
//     F: FnOnce(&mut AliasGenerator) -> T,
// {
//     ALIAS_GENERATOR.with_borrow_mut(|alias_generator| f(alias_generator))
// }
// /// Immutable accessor to the alias generator
// fn with_alias_generator_value<T, F>(f: F) -> T
// where
//     F: FnOnce(&AliasGenerator) -> T,
// {
//     ALIAS_GENERATOR.with_borrow(|alias_generator| f(alias_generator))
// }

// Public API for file count
pub struct FileCountStorage;
impl FileCountStorage {
    /// Get the current file count
    pub fn _get_file_count() -> u64 {
        _with_file_count_value(|&file_count| file_count)
    }

    /// Increment the file count
    pub fn generate_file_id() -> u64 {
        let new = FILE_COUNT.with_borrow_mut(|file_count| {
            let new_count = file_count.get() + 1;
            file_count.set(new_count).expect("Failed to set file count");
            new_count
        });
        new
        // with_file_count(|file_count| {
        //     let new_count = file_count.get() + 1;
        //     file_count.set(new_count);
        //     new_count
        // })
    }
}

// Public API for the owned files storage
pub struct OwnedFilesStorage;
impl OwnedFilesStorage {
    /// Get the list of owned files
    pub fn get_owned_files() -> Vec<FileId> {
        with_owned_files(|owned_files| owned_files)
    }

    /// Add a file ID to the owned files storage
    pub fn add_owned_file(file_id: &FileId) {
        // OWNED_FILES_STORAGE.with_borrow_mut(|owned_files| {
        //     owned_files.push(file_id);
        // });
        let _ = with_owned_files_storage(|owned_files| owned_files.push(file_id));
    }

    //  Remove a file ID from the owned files storage
    // pub fn remove_owned_file(file_id: FileId) {

    //     // OWNED_FILES_STORAGE.with_borrow_mut(|owned_files| {
    //     //     owned_files.retain(|&id| id != file_id);
    //     // });
    //     with_owned_files_storage( |owned_files| {
    //         owned_files.retain(|&id| id != file_id);
    //     });
    // }
}

// Public API for the file data storage
pub struct FileDataStorage;
impl FileDataStorage {
    /// Get a file by its ID
    pub fn get_file(file_id: &FileId) -> Option<File> {
        with_file_data(file_id, |file| file)
    }

    /// Set a file by its ID
    pub fn set_file(file_id: &FileId, file: File) {
        FILE_DATA_STORAGE.with_borrow_mut(|file_data| {
            file_data.insert(*file_id, file);
        });
    }

    /// Remove a file by its ID
    pub fn _remove_file(file_id: &FileId) {
        FILE_DATA_STORAGE.with_borrow_mut(|file_data| {
            file_data.remove(file_id);
        });
    }
}

// Public API for the file alias index storage
pub struct FileAliasIndexStorage;
impl FileAliasIndexStorage {
    /// Get a file ID by its alias
    pub fn get_file_id(alias: &String) -> Option<FileId> {
        with_file_alias_index(alias, |file_id| file_id)
    }

    /// Set a file ID by its alias
    pub fn set_file_id(alias: &str, file_id: &FileId) {
        FILE_ALIAS_INDEX_STORAGE.with_borrow_mut(|file_alias_index| {
            file_alias_index.insert(alias.to_owned(), *file_id);
        });
    }

    /// Remove a file ID by its alias
    pub fn remove_file_id(alias: &String) {
        FILE_ALIAS_INDEX_STORAGE.with_borrow_mut(|file_alias_index| {
            file_alias_index.remove(alias);
        });
    }
}
// Public API for the file shares storage
pub struct FileSharesStorage;
impl FileSharesStorage {
    ///get the whole file shares storage
    pub fn get_file_shares_storage() -> HashMap<Principal, Vec<FileId>> {
        with_file_shares_storage(|file_shares| {
            file_shares
                .iter()
                .map(|(principal, file_ids)| {
                    (
                        principal.0,
                        file_ids.iter().copied().collect::<Vec<FileId>>().to_vec(),
                    )
                })
                .collect::<HashMap<Principal, Vec<FileId>>>()
        })
    }
    /// Get a list of file IDs shared with a principal
    pub fn get_file_shares(principal: &Principal) -> Option<StorableFileIdVec> {
        with_file_shares(&StorablePrincipal(*principal), |file_ids| file_ids)
    }

    /// Set a list of file IDs shared with a principal
    pub fn set_file_shares(principal: &Principal, file_ids: Vec<FileId>) {
        let principal = StorablePrincipal::from(*principal);
        FILE_SHARES_STORAGE.with_borrow_mut(|file_shares| {
            file_shares.insert(principal, StorableFileIdVec::from(file_ids));
        });
    }

    /// Remove a list of file IDs shared with a principal
    pub fn _remove_whole_file_shares_list(principal: &Principal) {
        let principal = StorablePrincipal::from(*principal);
        FILE_SHARES_STORAGE.with_borrow_mut(|file_shares| {
            file_shares.remove(&principal);
        });
    }

    /// Remove a single file ID from the list of shares for a principal
    pub fn remove_file_shares(principal: &Principal, file_id: &FileId) {
        FILE_SHARES_STORAGE.with_borrow_mut(|file_shares| {
            let mut updated = file_shares
                .get(&StorablePrincipal::from(*principal))
                .unwrap_or_default()
                .0
                .clone();
            // Remove the file ID from the list
            updated.retain(|id| id != file_id);
            // If the list is empty, remove the principal from the storage
            if updated.is_empty() {
                file_shares.remove(&StorablePrincipal::from(*principal));
                return;
            }
            // Otherwise, update the list of file IDs for the principal
            file_shares.insert(
                StorablePrincipal::from(*principal),
                StorableFileIdVec::from(updated),
            );
        });
    }
}
// Public API for the file contents storage
pub struct FileContentsStorage;
impl FileContentsStorage {
    /// Get the contents of a file by its ID and chunk ID
    pub fn get_file_contents(file_id: &FileId, chunk_id: &ChunkId) -> Option<Vec<u8>> {
        with_file_contents(file_id, chunk_id, |contents| contents)
    }

    /// Set the contents of a file by its ID and chunk ID
    pub fn set_file_contents(file_id: &FileId, chunk_id: &ChunkId, contents: Vec<u8>) {
        FILE_CONTENTS_STORAGE.with_borrow_mut(|file_contents| {
            file_contents.insert((*file_id, *chunk_id), contents);
        });
    }

    /// Remove the contents of a file by its ID and chunk ID
    pub fn _remove_file_contents(file_id: &FileId, chunk_id: &ChunkId) {
        FILE_CONTENTS_STORAGE.with_borrow_mut(|file_contents| {
            file_contents.remove(&(*file_id, *chunk_id));
        });
    }
}

// Generates aliases for file requests.
//     #[serde(skip, default = "init_alias_generator")]
//     alias_generator: AliasGenerator,

/// Public API for the alias generator
///
#[cfg(test)]
mod test {

    use self::create_state::{File, FileContent, FileMetadata};
    use super::*;

    #[test]
    fn test_file_count_storage() {
        FileCountStorage::generate_file_id();
        assert_eq!(FileCountStorage::_get_file_count(), 1);
    }

    #[test]
    fn test_owned_files_storage() {
        let file_id = 1;
        OwnedFilesStorage::add_owned_file(&file_id);
        assert_eq!(OwnedFilesStorage::get_owned_files(), vec![file_id]);

        // OwnedFilesStorage::remove_owned_file(file_id);
        // assert_eq!(OwnedFilesStorage::get_owned_files(), Vec::<FileId>::new());
    }

    #[test]
    fn test_file_data_storage() {
        let file_id = 1;
        let file = File {
            metadata: FileMetadata {
                file_name: "test_file".to_string(),
                user_public_key: [1; 32],
                requester_principal: Principal::from_slice(&[1; 29]),
                requested_at: 0,
                uploaded_at: None,
            },
            content: FileContent::Pending {
                alias: "test_alias".to_string(),
            },
        };
        FileDataStorage::set_file(&file_id, file.clone());
        assert_eq!(FileDataStorage::get_file(&file_id), Some(file));

        FileDataStorage::_remove_file(&file_id);
        assert_eq!(FileDataStorage::get_file(&file_id), None);
    }

    #[test]
    fn test_file_alias_index_storage() {
        let alias = "test_alias".to_string();
        let file_id = 1;
        FileAliasIndexStorage::set_file_id(&alias, &file_id);
        assert_eq!(FileAliasIndexStorage::get_file_id(&alias), Some(file_id));

        FileAliasIndexStorage::remove_file_id(&alias);
        assert_eq!(FileAliasIndexStorage::get_file_id(&alias), None);
    }

    #[test]
    fn test_file_shares_storage() {
        let principal = Principal::from_slice(&[1; 6]);
        let file_id = 1;
        FileSharesStorage::set_file_shares(&principal, vec![file_id]);
        assert_eq!(
            FileSharesStorage::get_file_shares(&principal),
            Some(StorableFileIdVec(vec![file_id]))
        );

        // Remove a single file ID from the list of shares for a principal
        FileSharesStorage::remove_file_shares(&principal, &file_id);
        assert_eq!(FileSharesStorage::get_file_shares(&principal), None);
    }

    #[test]
    fn test_file_contents_storage() {
        let file_id = 1;
        let chunk_id = 1;
        let contents = vec![1, 2, 3, 4, 5];
        FileContentsStorage::set_file_contents(&file_id, &chunk_id, contents.clone());
        assert_eq!(
            FileContentsStorage::get_file_contents(&file_id, &chunk_id),
            Some(contents)
        );

        FileContentsStorage::_remove_file_contents(&file_id, &chunk_id);
        assert_eq!(
            FileContentsStorage::get_file_contents(&file_id, &chunk_id),
            None
        );
    }
}
