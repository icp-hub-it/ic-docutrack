CANDID:="./.artifact/orchestrator.did"

[group("orchestrator")]
set_user identity username:
  #!/usr/bin/env bash

  set -ex
  dfx canister call --candid {{CANDID}} --identity {{identity}} --network=local orchestrator set_user "(\"{{username}}\", blob \"\\00\")"

[group("orchestrator")]
user_canister identity:
  #!/usr/bin/env bash

  set -ex
  dfx canister call --candid {{CANDID}} --identity {{identity}} --network=local orchestrator user_canister

[group("orchestrator")]
whoami identity:
  #!/usr/bin/env bash

  set -ex
  dfx canister call --candid {{CANDID}} --identity {{identity}} --network=local orchestrator who_am_i
