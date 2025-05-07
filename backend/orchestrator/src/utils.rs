use candid::Principal;

/// Utility functions to trap the canister.
///
/// The reason of this is that you cannot use [`panic!`] on canisters and you can't use
/// [`ic_cdk::trap`] in test units.
pub fn trap<S>(msg: S) -> !
where
    S: AsRef<str>,
{
    #[cfg(target_family = "wasm")]
    ic_cdk::trap(msg);
    #[cfg(not(target_family = "wasm"))]
    panic!("{}", msg.as_ref());
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
