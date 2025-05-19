use super::{ChunkId, FILE_CONTENTS_STORAGE, FileId, with_file_contents};

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
    pub fn remove_file_contents(file_id: &FileId, chunk_id: &ChunkId) {
        FILE_CONTENTS_STORAGE.with_borrow_mut(|file_contents| {
            file_contents.remove(&(*file_id, *chunk_id));
        });
    }
}

#[cfg(test)]
mod test {

    use super::*;

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
    }

    #[test]
    fn test_remove_file_contents() {
        let file_id = 1;
        let chunk_id = 1;
        let contents = vec![1, 2, 3, 4, 5];
        FileContentsStorage::set_file_contents(&file_id, &chunk_id, contents.clone());
        assert_eq!(
            FileContentsStorage::get_file_contents(&file_id, &chunk_id),
            Some(contents)
        );

        FileContentsStorage::remove_file_contents(&file_id, &chunk_id);
        assert_eq!(
            FileContentsStorage::get_file_contents(&file_id, &chunk_id),
            None
        );
    }
}
