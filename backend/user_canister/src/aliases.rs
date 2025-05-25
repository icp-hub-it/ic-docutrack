use uuid::{ClockSequence, NoContext};

use crate::utils::trap;

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

/// A generator for human-readable file alias for files.
pub struct AliasGenerator {
    rng: Randomness,
}

impl AliasGenerator {
    /// Creates a new [`AliasGenerator`].
    pub fn new(randomness: Randomness) -> Self {
        Self { rng: randomness }
    }

    /// Generate a new alias using uuid v7.
    ///
    /// We cannot use directly [`uuid::Uuid::new_v7`] because it uses rng under the hood and this causes the canister to
    /// trap immediately after the installation.
    ///
    /// We instead have to use [`uuid::Builder::from_unix_timestamp_millis`] to generate the uuid with the same logic being used
    /// in the [`uuid::Uuid::new_v7`] method.
    ///
    /// It is not ideal, but it works.
    pub fn generate_uuidv7(&mut self) -> String {
        let timestamp_nanos = crate::utils::time();
        let timestamp_secs = timestamp_nanos / 1_000_000_000;
        let timestamp_subsec_nanos = (timestamp_nanos % 1_000_000_000) as u32;

        // get randomness from the management canister
        let seed = self.rng.get();
        let mut counter_and_random: u128 = u128::from_le_bytes(
            seed[0..16]
                .try_into()
                .expect("cannot be shorter than 16 bytes"),
        );

        let context = NoContext;
        let mut counter = context
            .generate_timestamp_sequence(timestamp_secs, timestamp_subsec_nanos)
            .0 as u128;

        let mut counter_bits = context.usable_bits() as u32;

        // If the counter intersects the variant field then shift around it.
        // This ensures that any bits set in the counter that would intersect
        // the variant are still preserved
        if counter_bits > 12 {
            let mask = u128::MAX << (counter_bits - 12);

            counter = (counter & !mask) | ((counter & mask) << 2);

            counter_bits += 2;
        }

        counter_and_random &= u128::MAX.overflowing_shr(counter_bits).0;
        counter_and_random |= counter
            .overflowing_shl(128u32.saturating_sub(counter_bits))
            .0;

        let timestamp_millis = timestamp_nanos / 1_000_000;

        let uuid = uuid::Builder::from_unix_timestamp_millis(
            timestamp_millis,
            &counter_and_random.to_be_bytes()[..10]
                .try_into()
                .expect("cannot be shorter than 10 bytes"),
        )
        .into_uuid();

        uuid.to_string()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_alias_generator() {
        let mut alias_generator = AliasGenerator::new(Randomness([0; 32]));
        let alias = alias_generator.generate_uuidv7();
        assert_eq!(alias.len(), 36);
        assert!(alias.chars().all(|c| c.is_ascii_hexdigit() || c == '-'));
    }
}
