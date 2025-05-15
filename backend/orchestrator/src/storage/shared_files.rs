mod user_shared_files;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use candid::Principal;
use did::StorablePrincipal;
use did::orchestrator::FileId;
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};

use self::user_shared_files::UserSharedFiles;
use crate::storage::memory::{MEMORY_MANAGER, SHARED_FILES_MEMORY_ID};

thread_local! {
    /// Shared files. Maps users to their shared files, grouped by the user canister.
    static SHARED_FILES: RefCell<StableBTreeMap<StorablePrincipal, UserSharedFiles, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableBTreeMap::new(MEMORY_MANAGER.with(|mm| mm.get(SHARED_FILES_MEMORY_ID)))
    );
}

/// Accessor for Storage for shared files.
///
/// Maps users to their shared files, grouped by the user canister.
pub struct SharedFilesStorage;

impl SharedFilesStorage {
    /// Share a file with a user for the provided user canister.
    ///
    /// Marks the file as shared for the user canister
    pub fn share_file(user: Principal, user_canister: Principal, file_id: FileId) {
        SHARED_FILES.with_borrow_mut(|shared_files| {
            let storable_user = StorablePrincipal::from(user);
            if !shared_files.contains_key(&storable_user) {
                shared_files.insert(storable_user, UserSharedFiles::default());
            }

            let mut user_shared_files = shared_files
                .get(&storable_user)
                .expect("user shared files must exist at this point");

            user_shared_files.insert_file(user_canister, file_id);

            shared_files.insert(storable_user, user_shared_files);
        })
    }

    /// Revoke a file share for a user for the provided user canister.
    pub fn revoke_share(user: Principal, user_canister: Principal, file_id: FileId) {
        SHARED_FILES.with_borrow_mut(|shared_files| {
            let storable_user = StorablePrincipal::from(user);
            if !shared_files.contains_key(&storable_user) {
                return;
            }

            let mut user_shared_files = shared_files
                .get(&storable_user)
                .expect("user shared files must exist at this point");

            user_shared_files.remove_file(user_canister, file_id);

            // If the user has no more files, remove the user from the map.
            if user_shared_files.is_empty() {
                shared_files.remove(&storable_user);
            } else {
                shared_files.insert(storable_user, user_shared_files);
            }
        })
    }

    /// For a user, get the list of file IDs shared for each user canister.
    pub fn get_shared_files(user: Principal) -> HashMap<Principal, HashSet<FileId>> {
        SHARED_FILES.with_borrow(|shared_files| {
            let storable_user = StorablePrincipal::from(user);

            shared_files
                .get(&storable_user)
                .map(|user_shared_files| user_shared_files.get_files())
                .unwrap_or_default()
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_should_insert_and_get_files_for_users() {
        let alice = Principal::from_slice(&[1; 29]);
        let bob = Principal::from_slice(&[2; 29]);

        let user_canister_a = Principal::from_slice(&[3; 29]);
        let user_canister_b = Principal::from_slice(&[4; 29]);

        // insert
        SharedFilesStorage::share_file(alice, user_canister_a, 1);
        SharedFilesStorage::share_file(alice, user_canister_b, 2);
        SharedFilesStorage::share_file(bob, user_canister_a, 1);

        // check
        let alice_files = SharedFilesStorage::get_shared_files(alice);
        assert_eq!(alice_files.len(), 2);
        assert!(alice_files.contains_key(&user_canister_a));
        assert!(alice_files.contains_key(&user_canister_b));
        assert!(alice_files[&user_canister_a].contains(&1));
        assert!(alice_files[&user_canister_b].contains(&2));

        let bob_files = SharedFilesStorage::get_shared_files(bob);
        assert_eq!(bob_files.len(), 1);
        assert!(bob_files.contains_key(&user_canister_a));
        assert!(bob_files[&user_canister_a].contains(&1));
    }

    #[test]
    fn test_should_revoke_file() {
        let alice = Principal::from_slice(&[1; 29]);
        let user_canister_a = Principal::from_slice(&[3; 29]);

        // insert
        SharedFilesStorage::share_file(alice, user_canister_a, 1);
        SharedFilesStorage::share_file(alice, user_canister_a, 2);

        // revoke
        SharedFilesStorage::revoke_share(alice, user_canister_a, 1);

        // check
        let alice_files = SharedFilesStorage::get_shared_files(alice);
        assert_eq!(alice_files.len(), 1);
        assert!(alice_files.contains_key(&user_canister_a));
        assert!(!alice_files[&user_canister_a].contains(&1));
        assert!(alice_files[&user_canister_a].contains(&2));

        // revoke the last file
        SharedFilesStorage::revoke_share(alice, user_canister_a, 2);

        // check
        let alice_files = SharedFilesStorage::get_shared_files(alice);
        assert_eq!(alice_files.len(), 0);
    }
}
