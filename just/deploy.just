
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
dfx_deploy_local install_mode orbit_station orbit_station_admin:
  #!/usr/bin/env bash
  set -ex

  if [ "{{install_mode}}" = "install" ]; then
      just dfx_start
      just create_canisters "$NETWORK" 1000000000000000
      just deploy_canisters "$NETWORK" "install" "{{orbit_station}}" "{{orbit_station_admin}}"
  elif [ "{{install_mode}}" = "upgrade" ] || [ "{{install_mode}}" = "reinstall" ]; then
      just deploy_canisters "$NETWORK" "{{install_mode}}" "{{orbit_station}}" "{{orbit_station_admin}}"
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
deploy_canisters network install_mode orbit_station orbit_station_admin:
  #!/usr/bin/env bash
  set -ex

  just build_all_canisters

  # orchestrator ID
  orchestrator_id=$(dfx canister --network={{network}} id orchestrator)

  just deploy_orchestrator_canister {{network}} {{install_mode}} {{orbit_station}} {{orbit_station_admin}}

[private]
deploy_orchestrator_canister network install_mode orbit_station orbit_station_admin:
  #!/usr/bin/env bash
  set -ex

  orchestrator_init_args="(
    variant { 
      Init = record {
        orbit_station_admin = \"$orbit_station_admin";
        orbit_station = principal \"$orbit_station\";
      }
    }
  )"

  echo "Installing the orbit station canister with init args: $orchestrator_init_args"

  dfx canister install --mode={{install_mode}} --yes --network={{network}} --argument="$orchestrator_init_args" orchestrator
