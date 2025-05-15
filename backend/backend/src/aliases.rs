use rand::SeedableRng;
use rand::seq::IndexedRandom;
use rand_chacha::ChaCha20Rng;

use crate::storage::alias_generator_seed::{AliasGeneratorSeed, Seed};

// List of English adjective words
const ADJECTIVES: &[&str] = &include!(concat!(env!("OUT_DIR"), "/adjectives.rs"));
// List of English noun words
const NOUNS: &[&str] = &include!(concat!(env!("OUT_DIR"), "/nouns.rs"));

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Randomness(Seed);

impl Default for Randomness {
    fn default() -> Self {
        Self::new()
    }
}

impl Randomness {
    pub fn new() -> Self {
        Self(AliasGeneratorSeed::new().get())
    }

    pub fn get(&self) -> Seed {
        self.0
    }
}

pub struct AliasGenerator {
    rng: ChaCha20Rng,
}

impl AliasGenerator {
    /// Creates a new `AliasGenerator`.
    pub fn new(randomness: Randomness) -> Self {
        Self {
            rng: ChaCha20Rng::from_seed(randomness.get()),
        }
    }

    /// Returns the next unique alias from this `AliasGenerator`.
    pub fn next(&mut self) -> String {
        let adjective = ADJECTIVES.choose(&mut self.rng).unwrap();
        let noun = NOUNS.choose(&mut self.rng).unwrap();
        format!("{adjective}-{noun}")
    }
}
