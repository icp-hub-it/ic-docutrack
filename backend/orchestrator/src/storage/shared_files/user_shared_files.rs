use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use candid::Principal;
use did::orchestrator::FileId;
use ic_stable_structures::Storable;
use ic_stable_structures::storable::Bound;

/// Map of shared files for each user.
///
/// Association between the user canister and the list of file IDs shared for that user.
///
/// ## Encoding
///
/// - The first 8 bytes are the length of the map.
/// - For each user canister:
///  - 1 byte: length of the user canister ID.
///  - N bytes: user canister ID.
///  - 8 bytes: length of the file ID list.
///  - For each file ID:
///    - 8 byte: file ID
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UserSharedFiles(HashMap<Principal, HashSet<FileId>>);

impl UserSharedFiles {
    /// Returns whether the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Insert a file ID into the list of shared files for a user.
    pub fn insert_file(&mut self, user: Principal, file_id: FileId) {
        self.0.entry(user).or_default().insert(file_id);
    }

    /// Get the list of file IDs shared for a user.
    ///
    /// If the map is empty after removing the file, the user entry is removed from the map.
    pub fn remove_file(&mut self, user: Principal, file_id: FileId) {
        if let Some(files) = self.0.get_mut(&user) {
            files.remove(&file_id);
            if files.is_empty() {
                self.0.remove(&user);
            }
        }
    }

    /// Get the list of file IDs shared for each user canister.
    pub fn get_files(&self) -> HashMap<Principal, HashSet<FileId>> {
        let mut files = HashMap::new();
        for (user, file_ids) in &self.0 {
            files.insert(*user, file_ids.clone());
        }

        files
    }
}

impl Storable for UserSharedFiles {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        // write the number of user canisters
        let len = self.0.len() as u64;
        bytes.extend_from_slice(&len.to_le_bytes());

        // iterate over the user canisters
        for (user_canister, file_ids) in &self.0 {
            let user_canister_bytes = user_canister.as_slice();
            // write the length of the user canister ID
            bytes.push(user_canister_bytes.len() as u8);
            // write the user canister ID
            bytes.extend_from_slice(user_canister_bytes);

            // write the number of file IDs
            let file_ids_len = file_ids.len() as u64;
            bytes.extend_from_slice(&file_ids_len.to_le_bytes());
            // write the file IDs
            for file_id in file_ids {
                bytes.extend_from_slice(&file_id.to_le_bytes());
            }
        }

        bytes.into()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        // read the number of user canisters
        let map_len =
            u64::from_le_bytes(bytes[0..8].try_into().expect("invalid user map len")) as usize;
        let mut offset = 8;
        // allocate map
        let mut map = HashMap::with_capacity(map_len);
        // iterate over the user canisters
        for _ in 0..map_len {
            // read the length of the user canister ID
            let user_canister_len = bytes[offset] as usize;
            offset += 1;
            // read the user canister ID
            let user_canister = Principal::from_slice(&bytes[offset..offset + user_canister_len]);
            offset += user_canister_len;

            // read the number of file IDs
            let file_ids_len = u64::from_le_bytes(
                bytes[offset..offset + 8]
                    .try_into()
                    .expect("Invalid file IDs length"),
            ) as usize;
            offset += 8;
            // allocate file IDs
            let mut file_ids = HashSet::with_capacity(file_ids_len);
            // read the file IDs
            for _ in 0..file_ids_len {
                let file_id = FileId::from_le_bytes(
                    bytes[offset..offset + 8]
                        .try_into()
                        .expect("Invalid file ID length"),
                );
                offset += 8;
                file_ids.insert(file_id);
            }

            // insert the user canister and file IDs into the map
            map.insert(user_canister, file_ids);
        }

        Self(map)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_should_insert_file() {
        let mut user_shared_files = UserSharedFiles::default();
        let user = Principal::from_slice(&[1; 29]);
        let file_id = 1;

        user_shared_files.insert_file(user, file_id);

        assert_eq!(user_shared_files.0.len(), 1);
        assert!(user_shared_files.0.contains_key(&user));
        assert!(user_shared_files.0[&user].contains(&file_id));

        // insert another file ID
        user_shared_files.insert_file(user, 2);
        assert_eq!(user_shared_files.0.len(), 1);
        assert!(user_shared_files.0.contains_key(&user));
        assert!(user_shared_files.0[&user].contains(&file_id));
        assert!(user_shared_files.0[&user].contains(&2));
    }

    #[test]
    fn test_should_remove_file() {
        let mut user_shared_files = UserSharedFiles::default();
        let user = Principal::from_slice(&[1; 29]);
        let file_id = 1;

        user_shared_files.insert_file(user, file_id);
        user_shared_files.remove_file(user, file_id);

        // check that user canister is removed
        assert!(!user_shared_files.0.contains_key(&user));

        // insert two
        user_shared_files.insert_file(user, file_id);
        user_shared_files.insert_file(user, 2);

        // remove 1
        user_shared_files.remove_file(user, file_id);
        // check that user canister is still present
        assert!(user_shared_files.0.contains_key(&user));
        // check that file ID 1 is removed
        assert!(!user_shared_files.0[&user].contains(&file_id));
        // check that file ID 2 is still present
        assert!(user_shared_files.0[&user].contains(&2));
    }

    #[test]
    fn test_should_get_files() {
        let mut user_shared_files = UserSharedFiles::default();
        let user = Principal::from_slice(&[1; 29]);
        let user_2 = Principal::from_slice(&[2; 29]);

        user_shared_files.insert_file(user, 1);
        user_shared_files.insert_file(user, 2);

        user_shared_files.insert_file(user_2, 1);
        user_shared_files.insert_file(user_2, 2);

        let user_canisters_shares = user_shared_files.get_files();

        assert_eq!(user_canisters_shares.len(), 2);
        assert!(user_canisters_shares.contains_key(&user));
        assert!(user_canisters_shares[&user].contains(&1));
        assert!(user_canisters_shares[&user].contains(&2));

        assert!(user_canisters_shares.contains_key(&user_2));
        assert!(user_canisters_shares[&user_2].contains(&1));
        assert!(user_canisters_shares[&user_2].contains(&2));
    }

    #[test]
    fn test_storable_roundtrip() {
        let mut user_shared_files = UserSharedFiles::default();
        let user = Principal::from_slice(&[1; 29]);
        let user_2 = Principal::from_slice(&[2; 29]);

        user_shared_files.insert_file(user, 1);
        user_shared_files.insert_file(user, 2);

        user_shared_files.insert_file(user_2, 1);
        user_shared_files.insert_file(user_2, 2);

        let bytes = user_shared_files.to_bytes();
        let decoded = UserSharedFiles::from_bytes(bytes);
        assert_eq!(decoded, user_shared_files);
    }
}
