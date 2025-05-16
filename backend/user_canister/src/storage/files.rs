mod create_state;
mod data_storage;
mod file_alias_index;
mod file_contents;
mod file_count;
mod owned_files;
mod shared_files;

use std::cell::RefCell;
use std::collections::HashSet;

use did::StorablePrincipal;
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell};

pub use self::create_state::{ChunkId, File, FileContent, FileId, FileMetadata};
pub use self::data_storage::FileDataStorage;
pub use self::file_alias_index::FileAliasIndexStorage;
pub use self::file_contents::FileContentsStorage;
pub use self::file_count::FileCountStorage;
pub use self::owned_files::OwnedFilesStorage;
pub use self::shared_files::FileSharesStorage;
use self::shared_files::SharedFiles;
use crate::storage::memory::{
    FILE_ALIAS_INDEX_MEMORY_ID, FILE_CONTENTS_MEMORY_ID, FILE_COUNT_MEMORY_ID, FILE_DATA_MEMORY_ID,
    FILE_SHARES_MEMORY_ID, MEMORY_MANAGER, OWNED_FILES_MEMORY_ID,
};

type ContentTuple = (FileId, ChunkId);

thread_local! {
  /// File count incrementer
  static FILE_COUNT: RefCell<StableCell<u64, VirtualMemory<DefaultMemoryImpl>>> =
      RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(FILE_COUNT_MEMORY_ID)), 0).unwrap()
  );

  /// Owned files storage vector
  /// Vector of available file IDs.
  static OWNED_FILES_STORAGE: RefCell<StableBTreeMap<FileId, (), VirtualMemory<DefaultMemoryImpl>>> =
      RefCell::new(StableBTreeMap::new(MEMORY_MANAGER.with(|mm| mm.get(OWNED_FILES_MEMORY_ID)))
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
  static FILE_SHARES_STORAGE: RefCell<StableBTreeMap<StorablePrincipal, SharedFiles, VirtualMemory<DefaultMemoryImpl>>> =
      RefCell::new(StableBTreeMap::new(MEMORY_MANAGER.with(|mm| mm.get(FILE_SHARES_MEMORY_ID)))
  );

  /// The contents of the file (stored in stable memory).
  static FILE_CONTENTS_STORAGE: RefCell<StableBTreeMap<ContentTuple, Vec<u8>, VirtualMemory<DefaultMemoryImpl>>> =
      RefCell::new(StableBTreeMap::new(MEMORY_MANAGER.with(|mm| mm.get(FILE_CONTENTS_MEMORY_ID)))
  );

}

/// Accessor to the owned files storage
/// Hashset of available file IDs.
fn with_owned_files_storage<T, F>(f: F) -> T
where
    F: FnOnce(&mut StableBTreeMap<FileId, (), VirtualMemory<DefaultMemoryImpl>>) -> T,
{
    OWNED_FILES_STORAGE.with_borrow_mut(|owned_files| f(owned_files))
}

/// Immutable accessor to the owned files
fn with_owned_files<F>(f: F) -> HashSet<u64>
where
    F: Fn(FileId) -> u64,
{
    OWNED_FILES_STORAGE
        .with_borrow(|owned_files| owned_files.keys().map(f).collect::<HashSet<u64>>())
}

/// Immutable accessor to the file data
fn with_file_data<T, F>(file_id: &FileId, f: F) -> Option<T>
where
    F: FnOnce(File) -> T,
{
    FILE_DATA_STORAGE.with_borrow(|file_data| file_data.get(file_id).map(f))
}

/// Immutable accessor to the file alias index
fn with_file_alias_index<T, F>(alias: &String, f: F) -> Option<T>
where
    F: FnOnce(FileId) -> T,
{
    FILE_ALIAS_INDEX_STORAGE.with_borrow(|file_alias_index| file_alias_index.get(alias).map(f))
}

/// Immutable accessor to the file shares
fn with_file_shares<T, F>(principal: &StorablePrincipal, f: F) -> Option<T>
where
    F: FnOnce(SharedFiles) -> T,
{
    FILE_SHARES_STORAGE.with_borrow(|file_shares| file_shares.get(principal).map(f))
}

/// Immutable accessor to the file contents
fn with_file_contents<T, F>(file_id: &FileId, chunk_id: &ChunkId, f: F) -> Option<T>
where
    F: FnOnce(Vec<u8>) -> T,
{
    FILE_CONTENTS_STORAGE
        .with_borrow(|file_contents| file_contents.get(&(*file_id, *chunk_id)).map(f))
}
