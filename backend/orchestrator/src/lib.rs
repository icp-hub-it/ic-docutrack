mod config;
mod memory;

use candid::Principal;
use config::Config;
use did::orchestrator::OrchestratorInitArgs;
use ic_cdk_macros::{init, query};

#[init]
pub fn init(args: OrchestratorInitArgs) {
    Config::set_orbit_station(args.orbit_station);
}

#[query]
pub fn orbit_station() -> Principal {
    Config::get_orbit_station()
}

ic_cdk::export_candid!();
