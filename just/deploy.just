
[group('deploy')]
dfx_start: dfx_prepare_env dfx_stop
  # start the local dfx replica
  dfx start --background --replica --clean --host 127.0.0.1:8000 --artificial-delay 0

  # fabricate cycles
  wallet_principal=$(dfx identity get-wallet) && dfx ledger fabricate-cycles --t 1000000 --canister $wallet_principal
  sleep 2

[group('deploy')]
dfx_stop:
  dfx stop

[group('deploy')]
dfx_deploy_local install_mode="install":
  #!/usr/bin/env bash
  set -ex

  if [ "{{install_mode}}" = "install" ]; then
      just dfx_start
      echo "Creating canisters on local network"
      just create_canisters local 1000000000000000
      # install the orbit station canister
      echo "Deploying Orbit Station canister on local network"
      just deploy_orbit_local
      # Install the orchestrator canister
      echo "Deploying Orchestrator canister on local network"
      just deploy_canisters local "install"
  elif [ "{{install_mode}}" = "upgrade" ] || [ "{{install_mode}}" = "reinstall" ]; then
      just deploy_canisters local "{{install_mode}}"
  else
      echo "Usage: $0 <create|upgrade|reinstall>"
      exit 1
  fi

[private]
dfx_prepare_env: 
  dfx identity new --storage-mode=plaintext --force admin
  dfx identity use admin

[private]
create_canisters network cycles:
  dfx canister --network={{network}} create --with-cycles={{cycles}} orchestrator

[private]
deploy_canisters network install_mode:
  #!/usr/bin/env bash
  set -ex

  just build_all_canisters

  just deploy_orchestrator_canister {{network}} {{install_mode}}

[private]
deploy_orchestrator_canister network install_mode:
  #!/usr/bin/env bash
  set -ex

  orbit_station=$(dfx canister --network={{network}} id orbit-station)

  # Get orchestrator admin name on the station
  orbit_station_admin="$(just get_orbit_station_uuid docutrack-orchestrator {{network}})"

  orchestrator_init_args="(
    variant { 
      Init = record {
        orbit_station_admin = \"$orbit_station_admin\";
        orbit_station = principal \"$orbit_station\";
      }
    }
  )"

  echo "Installing the orchestrator canister with init args: $orchestrator_init_args"

  dfx canister install --mode={{install_mode}} --yes --network={{network}} --argument="$orchestrator_init_args" --wasm ./.artifact/orchestrator.wasm.gz orchestrator
