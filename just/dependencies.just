# Orbit canister version
ORBIT_STATION_VERSION:="0.5.0"
ORBIT_UPGRADER_VERSION:="0.2.0"

[group('dependencies')]
fetch_all_canisters: fetch_orbit_canisters fetch_cycles_minting_canisters

[group('dependencies')]
fetch_orbit_canisters:
  echo "Fetching Orbit canisters..."
  curl --fail -L -o "{{WASM_DIR}}/orbit-station.wasm.gz" "https://github.com/dfinity/orbit/releases/download/%40orbit%2Fstation-v{{ORBIT_STATION_VERSION}}/station.wasm.gz"
  curl --fail -L -o "{{WASM_DIR}}/orbit-station.did" "https://raw.githubusercontent.com/dfinity/orbit/refs/tags/%40orbit/station-v0.5.0/core/station/api/spec.did"
  curl --fail -L -o "{{WASM_DIR}}/orbit-upgrader.wasm.gz" "https://github.com/dfinity/orbit/releases/download/%40orbit%2Fupgrader-v{{ORBIT_UPGRADER_VERSION}}/upgrader.wasm.gz"

[group('dependencies')]
fetch_cycles_minting_canisters:
  echo "Fetching cycles minting canisters..."
  curl --fail -L -o "{{WASM_DIR}}/cycles-minting-canister.did" "https://raw.githubusercontent.com/dfinity/ic/ee52ab3056cf5f39b09b08de70bdd20485c8b2dc/rs/nns/cmc/cmc.did"
  curl --fail -L -o "{{WASM_DIR}}/cycles-minting-canister.wasm.gz" "https://download.dfinity.systems/ic/ee52ab3056cf5f39b09b08de70bdd20485c8b2dc/canisters/cycles-minting-canister.wasm.gz" 
  curl --fail -L -o "{{WASM_DIR}}/icp-ledger.did" "https://raw.githubusercontent.com/dfinity/ic/d87954601e4b22972899e9957e800406a0a6b929/rs/rosetta-api/icp_ledger/ledger.did"
  curl --fail -L -o "{{WASM_DIR}}/icp-ledger.wasm.gz" "https://download.dfinity.systems/ic/d87954601e4b22972899e9957e800406a0a6b929/canisters/ledger-canister.wasm.gz"
  curl --fail -L -o "{{WASM_DIR}}/icp-index.did" "https://raw.githubusercontent.com/dfinity/ic/d87954601e4b22972899e9957e800406a0a6b929/rs/rosetta-api/icp_ledger/index/index.did"
  curl --fail -L -o "{{WASM_DIR}}/icp-index.wasm.gz" "https://download.dfinity.systems/ic/d87954601e4b22972899e9957e800406a0a6b929/canisters/ic-icp-index-canister.wasm.gz"
