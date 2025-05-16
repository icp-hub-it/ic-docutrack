use rand::SeedableRng;
use rand::seq::IndexedRandom;
use rand_chacha::ChaCha20Rng;

use crate::utils::trap;

// List of English adjective words
const ADJECTIVES: &[&str] = &include!(concat!(env!("OUT_DIR"), "/adjectives.rs"));
// List of English noun words
const NOUNS: &[&str] = &include!(concat!(env!("OUT_DIR"), "/nouns.rs"));

type Seed = [u8; 32];

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Randomness(Seed);

impl Randomness {
    /// Try to create a new [`Randomness`] instance.
    ///
    /// # Panics
    ///
    /// If the call to the management canister fails.
    pub async fn new() -> Self {
        if cfg!(target_family = "wasm") {
            let Ok(seed_vec) = ic_cdk::management_canister::raw_rand().await else {
                trap("Failed to get randomness from management canister");
            };

            let Ok(seed) = seed_vec.try_into() else {
                trap("Failed to convert randomness to 32 bytes");
            };

            Self(seed)
        } else {
            Self([0; 32])
        }
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
