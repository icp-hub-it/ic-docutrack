use candid::Principal;

use crate::aliases::{_AliasGenerator, _Randomness};

fn _get_randomness_seed() -> Vec<u8> {
    // this is an array of u8 of length 8.
    let time_seed = ic_cdk::api::time().to_be_bytes();
    // we need to extend this to an array of size 32 by adding to it an array of size 24 full of 0s.
    let zeroes_arr: [u8; 24] = [0; 24];
    [&time_seed[..], &zeroes_arr[..]].concat()
}

fn _init_alias_generator() -> _AliasGenerator {
    _AliasGenerator::_new(_Randomness::try_from(_get_randomness_seed().as_slice()).unwrap())
}

/// Utility functions to trap the canister.
///
/// The reason of this is that you cannot use [`panic!`] on canisters and you can't use
/// [`ic_cdk::trap`] in test units.
pub fn trap<S>(msg: S) -> !
where
    S: AsRef<str>,
{
    if cfg!(target_family = "wasm") {
        ic_cdk::trap(msg)
    } else {
        panic!("{}", msg.as_ref())
    }
}

/// Returns the caller of a message as [`Principal`].
///
/// The reason of this is that you cannot use [`ic_cdk::api::msg_caller`] on test units.
pub fn msg_caller() -> Principal {
    if cfg!(target_family = "wasm") {
        ic_cdk::api::msg_caller()
    } else {
        Principal::from_slice(&[1; 29])
    }
}

/// Returns current time in nanoseconds
pub fn time() -> u64 {
    if cfg!(target_family = "wasm") {
        ic_cdk::api::time()
    } else {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH");
        time.as_nanos() as u64
    }
}
