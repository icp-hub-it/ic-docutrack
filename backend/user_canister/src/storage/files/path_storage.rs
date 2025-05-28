use did::user_canister::Path;

use super::{FILE_ID_TO_PATH, FILE_PATH_TO_ID, FileId};

/// Storage for file paths in the user canister.
pub struct PathStorage;

impl PathStorage {
    /// Create a new file path storage entry.
    pub fn create(file_id: FileId, path: Path) {
        FILE_ID_TO_PATH.with_borrow_mut(|map| {
            map.insert(file_id, path.clone());
        });
        FILE_PATH_TO_ID.with_borrow_mut(|map| {
            map.insert(path, file_id);
        });
    }

    /// Exists check for a file path.
    pub fn exists(path: &Path) -> bool {
        FILE_PATH_TO_ID.with_borrow(|map| map.contains_key(path))
    }

    /// Remove a [`FileId`] from the storage.
    pub fn unlink(file_id: FileId) {
        let path_to_remove = FILE_ID_TO_PATH.with_borrow_mut(|map| map.remove(&file_id));
        if let Some(path) = path_to_remove {
            FILE_PATH_TO_ID.with_borrow_mut(|map| {
                map.remove(&path);
            });
        }
    }

    /// Rename a file path in the storage.
    #[allow(dead_code)]
    pub fn rename(file_id: FileId, new_path: Path) {
        // Remove the old path
        Self::unlink(file_id);
        // Set the new path
        Self::create(file_id, new_path);
    }

    /// Get the [`Path`] for a given [`FileId`].
    pub fn read_link(file_id: &FileId) -> Option<Path> {
        FILE_ID_TO_PATH.with_borrow(|map| map.get(file_id))
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_should_insert_and_read_link() {
        let file_id = 1;
        let path = Path::new("/test/file.txt").expect("invalid path");

        PathStorage::create(file_id, path.clone());
        assert!(PathStorage::exists(&path));
        assert_eq!(PathStorage::read_link(&file_id), Some(path));
    }

    #[test]
    fn test_should_unlink() {
        let file_id = 2;
        let path = Path::new("/test/file2.txt").expect("invalid path");

        PathStorage::create(file_id, path.clone());
        assert!(PathStorage::exists(&path));

        PathStorage::unlink(file_id);
        assert!(!PathStorage::exists(&path));
        assert_eq!(PathStorage::read_link(&file_id), None);
    }

    #[test]
    fn test_should_rename() {
        let file_id = 3;
        let old_path = Path::new("/test/old_file.txt").expect("invalid path");
        let new_path = Path::new("/test/new_file.txt").expect("invalid path");

        PathStorage::create(file_id, old_path.clone());
        assert!(PathStorage::exists(&old_path));
        assert_eq!(
            PathStorage::read_link(&file_id).expect("no such file"),
            old_path
        );

        PathStorage::rename(file_id, new_path.clone());
        assert!(!PathStorage::exists(&old_path));
        assert!(PathStorage::exists(&new_path));
        assert_eq!(PathStorage::read_link(&file_id), Some(new_path));
    }
}
