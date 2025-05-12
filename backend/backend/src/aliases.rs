use std::convert::{TryFrom, TryInto};

use rand::SeedableRng;
use rand::seq::IndexedRandom;
use rand_chacha::ChaCha20Rng;

// List of English adjective words
const _ADJECTIVES: &[&str] = &include!(concat!(env!("OUT_DIR"), "/adjectives.rs"));
// List of English noun words
const _NOUNS: &[&str] = &include!(concat!(env!("OUT_DIR"), "/nouns.rs"));

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct _Randomness([u8; 32]);

impl TryFrom<&[u8]> for _Randomness {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(_Randomness(
            value[0..32]
                .try_into()
                .map_err(|_| "_Randomness is not 32 bytes")?,
        ))
    }
}

impl _Randomness {
    pub fn _get(&self) -> [u8; 32] {
        self.0
    }
}

pub struct _AliasGenerator {
    rng: ChaCha20Rng,
}

impl _AliasGenerator {
    /// Creates a new `AliasGenerator`.
    pub fn _new(randomness: _Randomness) -> Self {
        Self {
            rng: ChaCha20Rng::from_seed(randomness._get()),
        }
    }

    /// Returns the next unique alias from this `AliasGenerator`.
    pub fn _next(&mut self) -> String {
        let adjective = _ADJECTIVES.choose(&mut self.rng).unwrap();
        let noun = _NOUNS.choose(&mut self.rng).unwrap();
        format!("{adjective}-{noun}")
    }
}
