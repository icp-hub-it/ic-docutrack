STATION_CANDID:="./.artifact/orbit-station.did"

[group("station")]
get_request request_id:
  #!/usr/bin/env bash

  set -ex
  dfx canister call --candid {{STATION_CANDID}} --output=json --network=local orbit-station get_request "(record { request_id = \"{{request_id}}\"; with_full_info = opt true })" | jq .
