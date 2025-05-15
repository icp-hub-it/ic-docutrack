use std::cell::RefCell;

use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::{DefaultMemoryImpl, StableCell};

use super::memory::{ALIAS_GENERATOR_SEED_MEMORY_ID, MEMORY_MANAGER};
use crate::utils::trap;

/// Seed alias for a random array of 32 bytes.
pub type Seed = [u8; 32];

const DEFAULT_SEED: Seed = [0; 32];

thread_local! {
    /// [`Randomness`] seed for the alias generator.
    static ALIAS_GENERATOR_SEED: RefCell<StableCell<Seed, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(ALIAS_GENERATOR_SEED_MEMORY_ID)), DEFAULT_SEED).unwrap()
    );
}

pub struct AliasGeneratorSeed(Seed);

impl AliasGeneratorSeed {
    /// Initializes the alias generator seed
    ///
    /// # Panics
    ///
    /// - If the seed cannot be converted to 32 bytes.
    /// - If the seed cannot be set in the thread local storage.
    /// - If the seed cannot be retrieved from the management canister.
    /// - If the seed cannot be set in the stable memory.
    /// - If the seed is already initialized.
    pub async fn init() {
        if Self::new().get() != DEFAULT_SEED {
            trap("alias generator seed is already initialized");
        }

        let seed = if cfg!(target_family = "wasm") {
            ic_cdk::management_canister::raw_rand()
                .await
                .unwrap_or_else(|e| {
                    trap(format!(
                        "failed to get random seed from management canister: {e}"
                    ))
                })
        } else {
            vec![1; 32]
        };

        let Ok(seed) = seed.try_into() else {
            trap("failed to convert seed to 32 bytes");
        };

        if let Err(err) = ALIAS_GENERATOR_SEED.with_borrow_mut(|cell| cell.set(seed)) {
            trap(format!("failed to set alias generator seed: {err:?}"));
        }
    }

    /// Creates a new [`AliasGeneratorSeed`].
    pub fn new() -> Self {
        ALIAS_GENERATOR_SEED.with_borrow(|seed| {
            let seed = seed.get();
            Self(*seed)
        })
    }

    /// Get the alias generator [`Seed`]
    pub fn get(&self) -> Seed {
        self.0
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[tokio::test]
    async fn test_should_init_seed() {
        AliasGeneratorSeed::init().await;

        let seed = AliasGeneratorSeed::new();
        assert_eq!(seed.get(), [1; 32]);
    }

    #[tokio::test]
    #[should_panic]
    async fn test_should_panic_if_seed_is_already_initialized() {
        AliasGeneratorSeed::init().await;
        AliasGeneratorSeed::init().await;
    }
}
