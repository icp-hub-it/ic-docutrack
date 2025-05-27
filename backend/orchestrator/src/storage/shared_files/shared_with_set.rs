use std::collections::HashSet;

use candid::Principal;
use ic_stable_structures::Storable;
use ic_stable_structures::storable::Bound;

/// A storable [`HashSet`] of [`Principal`]s.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SharedWithSet(pub HashSet<Principal>);

impl Storable for SharedWithSet {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = Vec::with_capacity(512);
        let set_len = self.0.len() as u64;
        bytes.extend_from_slice(&set_len.to_le_bytes());

        // write each principal
        for principal in &self.0 {
            let principal_bytes = principal.as_slice();
            let principal_len = principal_bytes.len() as u8;
            bytes.push(principal_len);
            bytes.extend_from_slice(principal_bytes);
        }

        bytes.into()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        // read length of the vector
        let mut offset = 0;
        let set_len = u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap()) as usize;
        offset += 8;
        let mut principals = HashSet::with_capacity(set_len);

        for _ in 0..set_len {
            // read length of each principal
            let principal_len = bytes[offset] as usize;
            offset += 1;

            // read the principal bytes
            let principal_bytes = &bytes[offset..offset + principal_len];
            offset += principal_len;

            // convert bytes to Principal
            let principal = Principal::from_slice(principal_bytes);
            principals.insert(principal);
        }

        Self(principals)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_principal_vec_roundtrip() {
        let principals: HashSet<_> = vec![
            Principal::from_slice(b"aaaaa-aaaaa-aaaaa-aaaaa-aaaaa"),
            Principal::from_slice(b"bbbbb-bbbbb-bbbbb-bbbbb-bbbbb"),
            Principal::from_slice(&[1; 6]),
            Principal::anonymous(),
        ]
        .into_iter()
        .collect();
        let principal_vec = SharedWithSet(principals.clone());

        let bytes = principal_vec.to_bytes();
        let deserialized_principal_vec = SharedWithSet::from_bytes(bytes);

        assert_eq!(principal_vec, deserialized_principal_vec);
        assert_eq!(principals, deserialized_principal_vec.0);
    }
}
