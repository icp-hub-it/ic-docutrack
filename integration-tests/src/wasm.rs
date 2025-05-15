use std::path::Path;

pub enum Canister {
    CyclesMinting,
    IcpIndex,
    IcpLedger,
    OrbitStation,
    OrbitUpgrader,
    Orchestrator,
    User,
}

impl Canister {
    pub fn as_path(&self) -> &'static Path {
        match self {
            Canister::CyclesMinting => Path::new("../.artifact/cycles-minting-canister.wasm.gz"),
            Canister::IcpIndex => Path::new("../.artifact/icp-index.wasm.gz"),
            Canister::IcpLedger => Path::new("../.artifact/icp-ledger.wasm.gz"),
            Canister::OrbitStation => Path::new("../.artifact/orbit-station.wasm.gz"),
            Canister::OrbitUpgrader => Path::new("../.artifact/orbit-upgrader.wasm.gz"),
            Canister::Orchestrator => Path::new("../.artifact/orchestrator.wasm.gz"),
            Canister::User => Path::new("../.artifact/user_canister.wasm.gz"),
        }
    }
}
