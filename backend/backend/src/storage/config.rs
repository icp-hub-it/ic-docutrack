use std::cell::RefCell;

use candid::Principal;
use did::StorablePrincipal;
use did::orchestrator::PublicKey;
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::{DefaultMemoryImpl, StableCell};

use super::memory::{
    MEMORY_MANAGER, ORBIT_STATION_MEMORY_ID, ORCHESTRATOR_MEMORY_ID, OWNER_MEMORY_ID,
    OWNER_PUBLIC_KEY_MEMORY_ID,
};

thread_local! {
    /// Orbit station
    static ORBIT_STATION: RefCell<StableCell<StorablePrincipal, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(ORBIT_STATION_MEMORY_ID)), Principal::anonymous().into()).unwrap()
    );
    /// Owner
    static OWNER: RefCell<StableCell<StorablePrincipal, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(OWNER_MEMORY_ID)), Principal::anonymous().into()).unwrap()
    );
    /// Owner public key
    static OWNER_PUBLIC_KEY: RefCell<StableCell<PublicKey, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(OWNER_PUBLIC_KEY_MEMORY_ID)), [0;32]).unwrap()
    );
    /// Orchestrator
    static ORCHESTRATOR: RefCell<StableCell<StorablePrincipal, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(ORCHESTRATOR_MEMORY_ID)), Principal::anonymous().into()).unwrap()
    );


}

/// Canister configuration
pub struct Config;
impl Config {
    /// Get the orbit station [`Principal`]
    pub fn get_orbit_station() -> Principal {
        ORBIT_STATION.with_borrow(|cell| cell.get().0)
    }

    /// Set the orbit station [`Principal`]
    pub fn set_orbit_station(principal: Principal) {
        if let Err(err) = ORBIT_STATION.with_borrow_mut(|cell| cell.set(principal.into())) {
            ic_cdk::trap(format!("Failed to set orbit station: {:?}", err));
        }
    }

    /// Get the owner [`Principal`]
    pub fn get_owner() -> Principal {
        OWNER.with_borrow(|cell| cell.get().0)
    }

    /// Set the owner [`Principal`]
    pub fn set_owner(principal: Principal) {
        if let Err(err) = OWNER.with_borrow_mut(|cell| cell.set(principal.into())) {
            ic_cdk::trap(format!("Failed to set owner: {:?}", err));
        }
    }
    /// Get the owner public key [`PublicKey`]
    pub fn get_owner_public_key() -> PublicKey {
        // OWNER_PUBLIC_KEY.with_borrow(|cell| cell.get())
        OWNER_PUBLIC_KEY.with_borrow(|cell| *cell.get())
    }
    /// Set the owner public key [`PublicKey`]
    pub fn set_owner_public_key(public_key: PublicKey) {
        if let Err(err) = OWNER_PUBLIC_KEY.with_borrow_mut(|cell| cell.set(public_key.into())) {
            ic_cdk::trap(format!("Failed to set owner public key: {:?}", err));
        }
    }
    /// Get the orchestrator [`Principal`]
    pub fn get_orchestrator() -> Principal {
        ORCHESTRATOR.with_borrow(|cell| cell.get().0)
    }
    /// Set the orchestrator [`Principal`]
    pub fn set_orchestrator(principal: Principal) {
        if let Err(err) = ORCHESTRATOR.with_borrow_mut(|cell| cell.set(principal.into())) {
            ic_cdk::trap(format!("Failed to set orchestrator: {:?}", err));
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_orbit_station() {
        let principal = Principal::from_slice(&[1; 29]);
        Config::set_orbit_station(principal);
        assert_eq!(Config::get_orbit_station(), principal);
    }

    #[test]
    fn test_owner() {
        let principal = Principal::from_slice(&[2; 29]);
        Config::set_owner(principal);
        assert_eq!(Config::get_owner(), principal);
    }

    #[test]
    fn test_orchestrator() {
        let principal = Principal::from_slice(&[3; 29]);
        Config::set_orchestrator(principal);
        assert_eq!(Config::get_orchestrator(), principal);
    }
    #[test]
    fn test_owner_public_key() {
        let public_key = [4; 32];
        Config::set_owner_public_key(public_key);
        assert_eq!(Config::get_owner_public_key(), public_key);
    }
}
