[group('orbit')]
deploy_orbit_local: deploy_cycles_management_canisters install_nns_ledger_canister install_nns_index_canister install_cycles_ledger_canister install_cycles_minting_canister
  #!/usr/bin/env bash
  set -ex

  echo "Deploying Orbit on local network"

  admin_principal="$(dfx identity get-principal)"
  orchestrator_principal="$(dfx canister --network=local id orchestrator)"

  # create orbit station
  echo "Creating Orbit Station canister on local network"
  dfx canister --network=local create --with-cycles=1000000000000000 orbit-station

  just deploy_orbit_upgrader local
  orbit_upgrader="$(dfx canister --network=local id orbit-upgrader)"

  init_args="(
    variant {
      Init = record {
        name = \"Orbit Station\";
        fallback_controller = principal \"rrkah-fqaaa-aaaaa-aaaaq-cai\";
        upgrader = variant {
          Id = principal \"$orbit_upgrader\"
        };
        admins = vec {
          record {
            name = \"orbit-admin\";
            identity = principal \"$admin_principal\";
          };
          record {
            name = \"docutrack-orchestrator\";
            identity = principal \"$orchestrator_principal\";
          }
        };
        quorum = opt 1;
      }
    }
  )"

  orbit_station="$(dfx canister --network=local id orbit-station)"
  echo "Orbit Station canister ID: $orbit_station"

  echo "Adding Orbit Station canister as a controller for the Orbit Upgrader canister"
  dfx canister update-settings --yes --network=local --add-controller "$orbit_station" orbit-station
  echo "Installing Orbit canister with init args: $init_args"
  dfx canister install --mode=reinstall --yes --wasm=./.artifact/orbit-station.wasm.gz --network=local --argument="$init_args" orbit-station
  echo "Orbit Station canister installed successfully"

  just wait_for_orbit_station_healthy local

[private]
deploy_orbit network:
  #!/usr/bin/env bash
  set -ex
  echo "Deploying Orbit on {{network}} network"

  cd /tmp
  rm -rf /tmp/orbit
  git clone git@github.com:dfinity/orbit.git
  cd orbit/
  git checkout @orbit/station-v0.5.0

[private]
deploy_cycles_management_canisters:
  #!/usr/bin/env bash
  set -ex
  echo "Deploying cycles management canisters on local network"

[private]
install_nns_ledger_canister:
  #!/usr/bin/env bash
  set -ex
  echo "Installing NNS ledger canister on local network"

  admin_principal="$(dfx identity get-principal)"
  admin_account="$(just get_account_id $admin_principal)"

  init_args="(
    variant { 
      Init = record {
        minting_account = \"$admin_account\";
        initial_values = vec {
          record {
            \"$admin_account\";
            record {
              e8s = 1_000_000_000_000 : nat64;
            }
          }
        };
        send_whitelist = vec { };
        transfer_fee = opt record {
          e8s = 10_000;
        };
        token_symbol = opt \"ICP\";
        token_name = opt \"Internet Computer\";
      }
    }
  )"

  echo "Installing NNS ledger canister with init args: $init_args"
  dfx canister --network=local create --with-cycles=1000000000000000 --specified-id="ryjl3-tyaaa-aaaaa-aaaba-cai" nns_ledger
  dfx canister install --mode=reinstall --yes --wasm=./.artifact/icp-ledger.wasm.gz --network=local --argument="$init_args" nns_ledger

[private]
install_nns_index_canister:
  #!/usr/bin/env bash
  set -ex
  echo "Installing NNS index canister on local network"

  nns_ledger="$(dfx canister --network=local id nns_ledger)"

  init_args="(
    record {
      ledger_id = principal \"$nns_ledger\";
    }
  )"

  echo "Installing NNS index canister with init args: $init_args"
  dfx canister --network=local create --with-cycles=1000000000000000 --specified-id="r7inp-6aaaa-aaaaa-aaabq-cai" nns_index
  dfx canister install --mode=reinstall --yes --wasm=./.artifact/icp-index.wasm.gz --network=local --argument="$init_args" nns_index

[private]
install_cycles_ledger_canister:
  #!/usr/bin/env bash
  set -ex
  echo "Installing cycles ledger canister on local network"

  init_args="(
    variant {
      Init = record {
        max_blocks_per_request = 1 : nat64;
      }
    }
  )"

  echo "Installing cycles ledger canister with init args: $init_args"
  dfx canister --network=local create --with-cycles=1000000000000000 --specified-id="um5iw-rqaaa-aaaaq-qaaba-cai" cycles_ledger
  dfx canister install --mode=reinstall --yes --wasm=./.artifact/cycles-ledger.wasm.gz --network=local --argument="$init_args" cycles_ledger

[private]
install_cycles_minting_canister:
  #!/usr/bin/env bash
  set -ex
  echo "Installing cycles minting canister on local network"

  nns_ledger="$(dfx canister --network=local id nns_ledger)"
  nns_index="$(dfx canister --network=local id nns_index)"
  cycles_ledger="$(dfx canister --network=local id cycles_ledger)"

  init_args="(
    opt record {
      ledger_canister_id = opt principal \"$nns_ledger\";
      governance_canister_id = opt principal \"rrkah-fqaaa-aaaaa-aaaaq-cai\";
      cycles_ledger_canister_id = opt principal \"$cycles_ledger\";
      last_purged_notification = opt 0;
    }
  )"

  echo "Installing cycles minting canister with init args: $init_args"
  dfx canister --network=local create --with-cycles=1000000000000000 --specified-id="rkp4c-7iaaa-aaaaa-aaaca-cai" cmc
  dfx canister install --mode=reinstall --yes --wasm=./.artifact/cycles-minting-canister.wasm.gz --network=local --argument="$init_args" cmc

  # Setup
  
[private]
deploy_orbit_upgrader network cycles="1000000000000000":
  #!/usr/bin/env bash
  set -ex
  
  echo "Deploying Orbit Upgrader canister on {{network}} network"

  orbit_station=$(dfx canister --network={{network}} id orbit-station)

  echo "Creating Orbit Upgrader canister on {{network}} network"
  dfx canister --network={{network}} create --with-cycles={{cycles}} orbit-upgrader

  init_args="(
    record {
      target_canister = principal \"$orbit_station\";
    }
  )"

  echo "Installing Orbit Upgrader canister with init args: $init_args"
  dfx canister install --mode=reinstall --yes --wasm=./.artifact/orbit-upgrader.wasm.gz --network={{network}} --argument="$init_args" orbit-upgrader
  dfx canister update-settings --yes --network={{network}} --add-controller "$orbit_station" orbit-upgrader

[private]
get_account_id principal:
  python3 ./scripts/get_account_id.py --text {{principal}}

[private]
get_orbit_station_uuid username network:
  #!/usr/bin/env bash

  set -e
  args="(
    record {
      search_term = opt \"{{username}}\";
    }
  )"

  uuid=$(dfx canister call --network={{network}} orbit-station --output json --query list_users "$args" | jq -r .Ok.users[0].id)
  echo $uuid

[private]
wait_for_orbit_station_healthy network:
  #!/usr/bin/env bash

  while [ 1 ]; do
     VAL=$(dfx canister call --output json --network={{network}} orbit-station health_status | grep "Healthy")
     RC=$?
     if [ $RC -eq 0 ]; then
       echo "Orbit Station is healthy"
       break
     else
       echo "Waiting for Orbit Station to be healthy..."
       sleep 5
     fi
  done
