use std::collections::HashSet;

use super::{FileId, with_owned_files, with_owned_files_storage};

// Public API for the owned files storage
pub struct OwnedFilesStorage;

impl OwnedFilesStorage {
    /// Get the list of owned files
    pub fn get_owned_files() -> HashSet<FileId> {
        with_owned_files(|owned_files| owned_files)
    }

    /// Add a file ID to the owned files storage
    pub fn add_owned_file(file_id: &FileId) {
        let _ = with_owned_files_storage(|owned_files| owned_files.insert(*file_id, ()));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_owned_files_storage() {
        let file_id = 1;
        OwnedFilesStorage::add_owned_file(&file_id);
        assert_eq!(
            OwnedFilesStorage::get_owned_files(),
            vec![file_id].into_iter().collect::<HashSet<_>>()
        );

        let file_id = 2;
        OwnedFilesStorage::add_owned_file(&file_id);
        assert_eq!(
            OwnedFilesStorage::get_owned_files(),
            vec![1, 2].into_iter().collect::<HashSet<_>>()
        );
    }
}
