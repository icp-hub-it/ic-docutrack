use super::{FILE_ALIAS_INDEX_STORAGE, FileId, with_file_alias_index};

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_file_alias_index_storage() {
        let alias = "test_alias".to_string();
        let file_id = 1;
        FileAliasIndexStorage::set_file_id(&alias, &file_id);
        assert_eq!(FileAliasIndexStorage::get_file_id(&alias), Some(file_id));

        FileAliasIndexStorage::remove_file_id(&alias);
        assert_eq!(FileAliasIndexStorage::get_file_id(&alias), None);
    }
}
