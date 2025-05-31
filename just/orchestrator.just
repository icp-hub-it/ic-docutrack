ORCHESTRATOR_CANDID:="./.artifact/orchestrator.did"


[group("orchestrator")]
set_user identity username:
  #!/usr/bin/env bash

  set -ex
  dfx canister call --candid {{ORCHESTRATOR_CANDID}} --output=json --identity {{identity}} --network=local orchestrator set_user "(\"{{username}}\", blob \"\\00\")" | jq .

[group("orchestrator")]
user_canister identity:
  #!/usr/bin/env bash

  set -ex
  dfx canister call --candid {{ORCHESTRATOR_CANDID}} --output=json --identity {{identity}} --network=local orchestrator user_canister | jq .

[group("orchestrator")]
whoami identity:
  #!/usr/bin/env bash

  set -ex
  dfx canister call --candid {{ORCHESTRATOR_CANDID}} --output=json --identity {{identity}} --network=local orchestrator who_am_i | jq .
