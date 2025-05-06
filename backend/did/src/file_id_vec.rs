use std::borrow::Cow;
use ic_stable_structures::Storable;
use ic_stable_structures::storable::Bound;


pub type FileId = u64;
pub struct StorableFileIdVec(pub Vec<FileId>);
impl StorableFileIdVec {
  pub fn new() -> Self {
      StorableFileIdVec(Vec::new())
  }

  pub fn add(&mut self, file_id: FileId) {
      self.0.push(file_id);
  }

  pub fn get(&self, index: usize) -> Option<FileId> {
      self.0.get(index).copied()
  }
  pub fn get_mut(&mut self, index: usize) -> Option<&mut FileId> {
    self.0.get_mut(index)
 }

  pub fn len(&self) -> usize {
      self.0.len()
  }
  pub fn contains(&self, file_id: &FileId) -> bool {
    self.0.contains(file_id)
  }
  pub fn retain<F>(&mut self, f: F)
  where
      F: FnMut(&FileId) -> bool,
  {
      self.0.retain(f);
  }
  pub fn iter(&self) -> std::slice::Iter<FileId> {
    self.0.iter()
}

  
}
impl From<Vec<FileId>> for StorableFileIdVec {
  fn from(vec: Vec<FileId>) -> Self {
      StorableFileIdVec(vec)
  }
}

impl Storable for StorableFileIdVec {
  fn to_bytes(&self) -> Cow<[u8]> {
      // Create a buffer to hold the serialized data
      let mut buf = Vec::new();

      // Serialize the length of the vector as a u32 (to save space)
      buf.extend_from_slice(&(self.len() as u32).to_le_bytes());

      // Serialize each FileId (u64) in little-endian format
      for file_id in self.0.iter() {
          buf.extend_from_slice(&file_id.to_le_bytes());
      }

      Cow::Owned(buf)
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
      // Ensure there are enough bytes for the length (u32 = 4 bytes)
      if bytes.len() < 4 {
          return StorableFileIdVec(Vec::new()); // Or panic, depending on your error handling
      }

      // Read the length (first 4 bytes) as u32
      let len = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;

      // Ensure the remaining bytes match the expected length (len * 8 bytes for u64)
      if bytes.len() < 4 + len * 8 {
          return StorableFileIdVec(Vec::new()); // Or panic
      }

      // Deserialize each u64 into the vector
      let mut result = Vec::with_capacity(len);
      for i in 0..len {
          let start = 4 + i * 8;
          let end = start + 8;
          let file_id = u64::from_le_bytes(bytes[start..end].try_into().unwrap());
          result.push(file_id);
      }

      StorableFileIdVec(result)
  }


  const BOUND: Bound = Bound::Bounded {
      max_size: 1_000_000, // Maximum bytes for the serialized Vec<FileId>
      is_fixed_size: false,
  };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_id_vec() {
        let mut file_id_vec = StorableFileIdVec::new();
        file_id_vec.add(1);
        file_id_vec.add(2);
        file_id_vec.add(3);

        assert_eq!(file_id_vec.len(), 3);
        assert_eq!(file_id_vec.get(0), Some(1));
        assert_eq!(file_id_vec.get(1), Some(2));
        assert_eq!(file_id_vec.get(2), Some(3));
        assert_eq!(file_id_vec.get(3), None);

        // Serialize and deserialize
        let serialized = file_id_vec.to_bytes();
        let deserialized = StorableFileIdVec::from_bytes(serialized);
        assert_eq!(file_id_vec.len(), deserialized.len());
        assert_eq!(file_id_vec.get(0), deserialized.get(0));
        assert_eq!(file_id_vec.get(1), deserialized.get(1));
        assert_eq!(file_id_vec.get(2), deserialized.get(2));
        assert_eq!(file_id_vec.get(3), deserialized.get(3));
        assert_eq!(file_id_vec.0, deserialized.0);
    }
}