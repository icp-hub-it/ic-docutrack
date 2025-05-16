mod storable;

use std::collections::{HashMap, HashSet};

use candid::Principal;
use did::StorablePrincipal;

pub use self::storable::SharedFiles;
use super::{FILE_SHARES_STORAGE, FileId, with_file_shares};

// Public API for the file shares storage
pub struct FileSharesStorage;

impl FileSharesStorage {
    /// Get the list of all file shares
    pub fn get_file_shares_storage() -> HashMap<Principal, HashSet<FileId>> {
        FILE_SHARES_STORAGE.with_borrow(|file_shares| {
            file_shares
                .iter()
                .map(|(principal, file_ids)| {
                    let principal = principal.as_principal();
                    (*principal, file_ids.to_hashset())
                })
                .collect()
        })
    }

    /// Get a list of file IDs shared with a principal
    pub fn get_file_shares(principal: &Principal) -> Option<HashSet<FileId>> {
        with_file_shares(&StorablePrincipal(*principal), |file_ids| {
            file_ids.to_hashset()
        })
    }

    /// Insert a list of file IDs shared with a principal
    pub fn share(principal: &Principal, file_ids: Vec<FileId>) {
        let principal = StorablePrincipal::from(*principal);

        FILE_SHARES_STORAGE.with_borrow_mut(|file_shares| {
            let mut file_set = file_shares.get(&principal).unwrap_or_default();
            for file_id in file_ids {
                // Insert the file ID into the list of shares for the principal
                file_set.insert(file_id);
            }
            // Update the list of file IDs for the principal
            file_shares.insert(principal, file_set);
        });
    }

    /// Remove a single file ID from the list of shares for a principal
    pub fn revoke(principal: &Principal, file_id: &FileId) {
        let principal = StorablePrincipal::from(*principal);

        FILE_SHARES_STORAGE.with_borrow_mut(|file_shares| {
            let mut file_set = file_shares.get(&principal).unwrap_or_default();

            // Remove the file ID from the list of shares for the principal
            file_set.remove(file_id);

            if file_set.is_empty() {
                // If the list is empty, remove the principal from the storage
                file_shares.remove(&principal);
            } else {
                // Otherwise, update the list of file IDs for the principal
                file_shares.insert(principal, file_set);
            }
        });
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_file_shares_storage() {
        let principal = Principal::from_slice(&[1; 6]);
        let file_id = 1;
        FileSharesStorage::share(&principal, vec![file_id]);
        assert_eq!(
            FileSharesStorage::get_file_shares(&principal),
            Some(vec![file_id].into_iter().collect::<HashSet<FileId>>())
        );

        // Remove a single file ID from the list of shares for a principal
        FileSharesStorage::revoke(&principal, &file_id);
        assert_eq!(FileSharesStorage::get_file_shares(&principal), None);
    }

    #[test]
    fn test_add_many_shares() {
        let principal = Principal::from_slice(&[1; 6]);
        let file_ids = vec![1, 2, 3, 4, 5];
        FileSharesStorage::share(&principal, file_ids.clone());
        assert_eq!(
            FileSharesStorage::get_file_shares(&principal),
            Some(file_ids.into_iter().collect::<HashSet<FileId>>())
        );

        // add another
        let file_ids = vec![6];
        FileSharesStorage::share(&principal, file_ids.clone());
        assert_eq!(
            FileSharesStorage::get_file_shares(&principal),
            Some(
                vec![1, 2, 3, 4, 5, 6]
                    .into_iter()
                    .collect::<HashSet<FileId>>()
            )
        );

        // revoke one
        FileSharesStorage::revoke(&principal, &3);

        assert_eq!(
            FileSharesStorage::get_file_shares(&principal),
            Some(vec![1, 2, 4, 5, 6].into_iter().collect::<HashSet<FileId>>())
        );
    }
}
