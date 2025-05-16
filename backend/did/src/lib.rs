//! Candid types for ic-docutrack canisters
// use std::borrow::Cow;
// use ic_stable_structures::Storable;
// use ic_stable_structures::storable::Bound;

#[rustfmt::skip]
#[allow(clippy::all)]
#[allow(deprecated)]
pub mod orbit_station;
pub mod orchestrator;
mod principal;
pub mod user_canister;
pub mod utils;

pub use self::principal::StorablePrincipal;
