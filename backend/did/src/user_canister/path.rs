use std::rc::Rc;
use std::str::FromStr;

use candid::CandidType;
use candid::types::{Type, TypeInner};
use ic_stable_structures::Storable;
use ic_stable_structures::storable::Bound;
use serde::{Deserialize, Serialize};

/// Maximum size of a file path.
const MAX_PATH_SIZE: usize = 4096;

/// File Path.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Path(String);

impl CandidType for Path {
    fn _ty() -> candid::types::Type {
        Type(Rc::new(TypeInner::Text))
    }

    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: candid::types::Serializer,
    {
        serializer.serialize_text(&self.0)
    }
}

impl Serialize for Path {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Path {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let p = Path(s);
        p.validate()
            .map_err(|e| serde::de::Error::custom(format!("Invalid path: {}", e)))?;
        Ok(p)
    }
}

impl Storable for Path {
    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_PATH_SIZE as u32 + 2,
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        // write len of path
        let path_len: u16 = self.0.len() as u16;
        let mut bytes = Vec::with_capacity(self.0.len() + 2);
        bytes.extend_from_slice(&path_len.to_le_bytes());
        // write path
        bytes.extend_from_slice(self.0.as_bytes());

        bytes.into()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        // read len of path
        let path_len = u16::from_le_bytes([bytes[0], bytes[1]]) as usize;
        // read path
        let path = String::from_utf8(bytes[2..2 + path_len].to_vec())
            .expect("Invalid UTF-8 sequence in path");

        let p = Self(path);
        p.validate_dangerous(); // Validate the path
        p
    }
}

impl FromStr for Path {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let p = Path(s.to_string());
        p.validate()?;
        Ok(p)
    }
}

impl TryFrom<String> for Path {
    type Error = &'static str;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::from_str(&s)
    }
}

impl Path {
    /// Creates a new [`Path`] from a string.
    ///
    /// It is equivalent to `Path::try_from(path)`.
    pub fn new(path: impl ToString) -> Result<Self, &'static str> {
        Self::try_from(path.to_string())
    }

    /// Returns components of the path as an iterator.
    pub fn components(&self) -> impl Iterator<Item = &str> {
        // Skip the leading empty component
        self.0.split('/').filter(|s| !s.is_empty())
    }

    /// Get the parent [`Path`] of this path.
    pub fn parent(&self) -> Option<Path> {
        let components: Vec<&str> = self.components().collect();
        if components.is_empty() {
            None // No parent for root or empty path
        } else {
            let mut p = String::from("/");
            p.push_str(&components[..components.len() - 1].join("/"));
            Some(Self(p))
        }
    }

    /// Get the file name from the path.
    pub fn file_name(&self) -> Option<&str> {
        self.components().last()
    }

    /// Returns `true` if the path is a directory.
    pub fn is_dir(&self) -> bool {
        // A path is considered a directory if it ends with a '/'
        self.0.ends_with('/')
    }

    /// Returns `true` if the path is a file.
    pub fn is_file(&self) -> bool {
        // A path is considered a file if it does not end with a '/'
        !self.is_dir()
    }

    /// Validates the path.
    ///
    /// A [`Path`] is valid if it:
    ///
    /// - Is not empty.
    /// - Starts with `/`
    /// - Does not contain `..` or `.` segments.
    ///
    /// # Panics
    ///
    /// If the path is invalid, this method will panic.
    fn validate_dangerous(&self) {
        self.validate().expect("Path validation failed");
    }

    /// Validates the path and returns a [`Result`].
    /// A [`Path`] is valid if it:
    ///
    /// - Is not empty.
    /// - Starts with `/`
    /// - Does not contain `..` or `.` segments.
    fn validate(&self) -> Result<(), &'static str> {
        let p = self.0.as_str();
        if p.is_empty() {
            return Err("Path cannot be empty");
        }
        if !p.starts_with('/') {
            return Err("Path must start with '/'");
        }
        if self.components().any(|c| c == ".." || c == ".") {
            return Err("Path cannot contain '..' or '.' segments");
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use candid::{Decode, Encode};

    use super::*;

    #[test]
    fn test_should_create_valid_path() {
        let path = Path::new("/valid/path".to_string());
        assert!(path.is_ok());
    }

    #[test]
    fn test_should_fail_on_empty_path() {
        let path = Path::new("".to_string());
        assert!(path.is_err());
        assert_eq!(path.err(), Some("Path cannot be empty"));
    }

    #[test]
    fn test_should_fail_on_invalid_start() {
        let path = Path::new("invalid/path".to_string());
        assert!(path.is_err());
        assert_eq!(path.err(), Some("Path must start with '/'"));
    }

    #[test]
    fn test_should_fail_on_invalid_segments() {
        let path = Path::new("/invalid/../path".to_string());
        assert!(path.is_err());
        assert_eq!(path.err(), Some("Path cannot contain '..' or '.' segments"));

        let path = Path::new("/invalid/./path".to_string());
        assert!(path.is_err());
        assert_eq!(path.err(), Some("Path cannot contain '..' or '.' segments"));
    }

    #[test]
    fn test_should_get_parent_path() {
        let path = Path::new("/valid/path".to_string()).unwrap();
        assert_eq!(path.parent(), Some(Path("/valid".to_string())));

        let root_path = Path::new("/".to_string()).unwrap();
        assert_eq!(root_path.parent(), None);

        let single_component_path = Path::new("/single".to_string()).unwrap();
        assert_eq!(single_component_path.parent(), Some(Path("/".to_string())));
    }

    #[test]
    fn test_should_split_path_into_components() {
        let path = Path::new("/valid/path/with/components".to_string()).unwrap();
        let components: Vec<&str> = path.components().collect();
        assert_eq!(components, vec!["valid", "path", "with", "components"]);
    }

    #[test]
    fn test_should_serialize_and_deserialize_path() {
        let path = Path::new("/valid/path".to_string()).unwrap();
        let serialized = Encode!(&path).unwrap();
        let deserialized = Decode!(&serialized, Path).unwrap();
        assert_eq!(path, deserialized);
    }

    #[test]
    fn test_path_storable_roundtrip() {
        let path = Path::new("/valid/path".to_string()).unwrap();
        let bytes = path.to_bytes();
        let deserialized_path = Path::from_bytes(bytes);
        assert_eq!(path, deserialized_path);
    }

    #[test]
    #[should_panic]
    fn test_should_panic_when_deserializing_invalid_path() {
        let invalid_path = Path("invalid/path".to_string());
        let serialized = Encode!(&invalid_path).unwrap();
        let _deserialized = Decode!(&serialized, Path).unwrap();
    }

    #[test]
    fn test_should_get_file_name() {
        let path = Path::new("/valid/path/file.txt".to_string()).unwrap();
        assert_eq!(path.file_name(), Some("file.txt"));

        let root_path = Path::new("/file.txt".to_string()).unwrap();
        assert_eq!(root_path.file_name(), Some("file.txt"));

        let single_component_path = Path::new("/single".to_string()).unwrap();
        assert_eq!(single_component_path.file_name(), Some("single"));

        let empty_path = Path::new("/".to_string()).unwrap();
        assert_eq!(empty_path.file_name(), None);
    }

    #[test]
    fn test_should_identify_directory_and_file_paths() {
        let dir_path = Path::new("/valid/path/".to_string()).unwrap();
        assert!(dir_path.is_dir());
        assert!(!dir_path.is_file());

        let file_path = Path::new("/valid/path/file.txt".to_string()).unwrap();
        assert!(!file_path.is_dir());
        assert!(file_path.is_file());

        let root_dir = Path::new("/".to_string()).unwrap();
        assert!(root_dir.is_dir());
        assert!(!root_dir.is_file());
    }
}
