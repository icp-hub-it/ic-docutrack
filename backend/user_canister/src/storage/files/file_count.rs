use super::FILE_COUNT;

// Public API for file count
pub struct FileCountStorage;

impl FileCountStorage {
    /// Get the current file count as the next file ID.
    ///
    /// Then increment by one for the next call.
    ///
    /// In this way the file ID is 0-based, while the FILE count actually tells how many files are in the system.
    pub fn generate_file_id() -> u64 {
        let new = FILE_COUNT.with_borrow_mut(|file_count| {
            let new_count = *file_count.get();
            file_count
                .set(new_count + 1)
                .expect("Failed to set file count");
            new_count
        });
        new
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_file_count_storage() {
        let new_id = FileCountStorage::generate_file_id();
        assert_eq!(new_id, 0);
        let new_id = FileCountStorage::generate_file_id();
        assert_eq!(new_id, 1);
        let new_id = FileCountStorage::generate_file_id();
        assert_eq!(new_id, 2);
    }
}
