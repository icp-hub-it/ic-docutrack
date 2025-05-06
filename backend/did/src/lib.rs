//! Candid types for ic-docutrack canisters

#[rustfmt::skip]
#[allow(clippy::all)]
#[allow(deprecated)]
pub mod orbit_station;
pub mod orchestrator;
mod principal;

pub use self::principal::StorablePrincipal;
