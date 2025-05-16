use std::borrow::Cow;
use std::collections::HashSet;

use ic_stable_structures::Storable;
use ic_stable_structures::storable::Bound;

use crate::utils::trap;

pub type FileId = u64;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SharedFiles(HashSet<FileId>);

impl SharedFiles {
    pub fn to_hashset(&self) -> HashSet<FileId> {
        self.0.clone()
    }

    pub fn insert(&mut self, file_id: FileId) {
        self.0.insert(file_id);
    }

    pub fn remove(&mut self, file_id: &FileId) {
        self.0.remove(file_id);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Vec<FileId>> for SharedFiles {
    fn from(vec: Vec<FileId>) -> Self {
        SharedFiles(vec.into_iter().collect())
    }
}

impl Storable for SharedFiles {
    const BOUND: Bound = Bound::Unbounded; // No fixed size limit

    fn to_bytes(&self) -> Cow<[u8]> {
        // Create a buffer to hold the serialized data
        let mut buf = Vec::new();

        // Serialize the length of the vector as a 64
        buf.extend_from_slice(&(self.len() as u64).to_le_bytes());

        // Serialize each FileId (u64) in little-endian format
        for file_id in self.0.iter() {
            buf.extend_from_slice(&file_id.to_le_bytes());
        }

        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        // Ensure there are enough bytes for the length (u32 = 4 bytes)
        if bytes.len() < 4 {
            trap("Invalid byte array: not enough bytes for length");
        }

        // Read the length (first 8 bytes) as u64
        let len = u64::from_le_bytes(bytes[0..8].try_into().unwrap()) as usize;

        // Ensure the remaining bytes match the expected length (len * 8 bytes for u64)
        if bytes.len() < 8 + len * 8 {
            trap("Invalid byte array: not enough bytes for FileId hash set");
        }

        // Deserialize each u64 into the hashset
        let mut result = HashSet::with_capacity(len);
        for i in 0..len {
            let start = 8 + i * 8;
            let end = start + 8;
            let file_id = u64::from_le_bytes(bytes[start..end].try_into().unwrap());
            result.insert(file_id);
        }

        SharedFiles(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_files_storable() {
        let mut file_id_set = SharedFiles::default();
        file_id_set.insert(1);
        file_id_set.insert(2);
        file_id_set.insert(3);

        assert_eq!(file_id_set.len(), 3);
        assert!(file_id_set.0.contains(&1),);
        assert!(file_id_set.0.contains(&2),);
        assert!(file_id_set.0.contains(&3),);
        assert!(!file_id_set.0.contains(&4),);

        // Serialize and deserialize
        let serialized = file_id_set.to_bytes();
        let deserialized = SharedFiles::from_bytes(serialized);
        assert_eq!(file_id_set.len(), deserialized.len());
        assert_eq!(file_id_set.0, deserialized.0);
    }
}
