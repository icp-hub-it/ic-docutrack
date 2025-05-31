NNS_GOVERNANCE_CANISTER:="rrkah-fqaaa-aaaaa-aaaaq-cai"
NNS_INDEX_CANISTER:="r7inp-6aaaa-aaaaa-aaabq-cai"
NNS_LEDGER_CANISTER:="ryjl3-tyaaa-aaaaa-aaaba-cai"

[group('orbit')]
deploy_orbit_local:
  #!/usr/bin/env bash
  set -ex

  echo "Deploying Orbit on local network"

  admin_principal="$(dfx identity get-principal)"
  orchestrator_principal="$(dfx canister --network=local id orchestrator)"

  # create orbit station
  echo "Creating Orbit Station canister on local network"
  dfx canister --network=local create --with-cycles=1_000_000_000_000_000 orbit-station

  just deploy_orbit_upgrader local
  orbit_upgrader="$(dfx canister --network=local id orbit-upgrader)"

  init_args="(
    opt variant {
      Init = record {
        name = \"Orbit Station\";
        fallback_controller = opt principal \"{{NNS_GOVERNANCE_CANISTER}}\";
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
        quorum = opt 1 : opt nat16;
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
deploy_orbit_upgrader network cycles="5_000_000_000_000":
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
