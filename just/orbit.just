[group('orbit')]
deploy_orbit_local: deploy_cycles_management_canisters install_nns_ledger_canister install_nns_index_canister create_nns_exchange_canister install_cycles_minting_canister
  #!/usr/bin/env bash
  set -ex

  echo "Deploying Orbit on local network"

  admin_principal="$(dfx identity get-principal)"
  orchestrator_principal="$(dfx canister --network=local id ochestrator)"

  init_args="(
    variant {
      Init = record {
        name = \"Orbit Station\";
        fallback_controller = principal \"rrkah-fqaaa-aaaaa-aaaaq-cai\";
        upgrader = variant {
          Deploy = record {
            initial_cycles = opt 5_000_000_000_000;
            wasm_module = blob \"$(python3 ./scripts/dfx_wasm.py ./.artifact/orbit-upgrader.wasm.gz)\";
          };
        };
        admins = vec {
          record {
            name = \"orbit-admin\";
            identity = principal \"$admin_principal\";
          };
          record {
            name = \"docutrack-orchestrator\";
            identity = principal \"$orchestrator_principal\";
          };
        };
        quorum = opt 1;
      }
    }
  )"

  echo "Installing Orbit canister with init args: $init_args"
  dfx canister --network=local create --with-cycles=1000000000000000 --argument="$init_args" orbit-station

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
        minting_account = blob \"$admin_account\";
        initial_values = vec {
          $admin_account = record { 
            e8s = 1_000_000_000_000;
          };
        };
        send_whitelist = vec { };
        transfer_fee = record {
          e8s = 10_000;
        };
        token_symbol = \"ICP\";
        token_name = \"Internet Computer\";
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
create_nns_exchange_canister:
  #!/usr/bin/env bash
  set -ex
  echo "Creating NNS exchange canister on local network"

  dfx canister --network=local create --with-cycles=1000000000000000 --specified-id="uf6dk-hyaaa-aaaaq-qaaaq-cai" nns_exchange

[private]
install_cycles_minting_canister:
  #!/usr/bin/env bash
  set -ex
  echo "Installing cycles minting canister on local network"

  nns_ledger="$(dfx canister --network=local id nns_ledger)"
  nns_index="$(dfx canister --network=local id nns_index)"
  nns_exchange="$(dfx canister --network=local id nns_exchange)"

  init_args="(
    record {
      ledger_canister_id = principal \"$nns_ledger\";
      governance_canister_id = principal \"rrkah-fqaaa-aaaaa-aaaaq-cai\";
      exchange_rate_canister = principal \"$nns_exchange\";
      cycles_ledger_canister = principal \"um5iw-rqaaa-aaaaq-qaaba-cai\";
      last_purged_notification = opt 0;
    }
  )"

  echo "Installing cycles minting canister with init args: $init_args"
  dfx canister --network=local create --with-cycles=1000000000000000 --specified-id="rkp4c-7iaaa-aaaaa-aaaca-cai" cycles_minting
  dfx canister install --mode=reinstall --yes --wasm=./.artifact/cycles-minting-canister.wasm.gz --network=local --argument="$init_args" cycles_minting

  # Setup
  

[private]
get_account_id principal:
  python3 ./scripts/get_account_id.py {{principal}}

[private]
get_orbit_station_uuid username network:
  #!/usr/bin/env bash

  set -e
  args="(
    search_term = \"{{username}}\";
  )"

  response="$(dfx canister call --network={{network}} orbit-station --query list_users $args)"
  echo $response
