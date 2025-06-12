# Cleans the build artifacts
[group('build')]
[confirm("Are you sure you want to clean the build artifacts?")]
clean:
  rm -rf {{WASM_DIR}}
  cargo clean

# Builds all canisters
[group('build')]
build_all_canisters: pre_build build_user_canister build_orchestrator

[group('build')]
build_user_canister:
  just build_canister "user_canister" "user_canister"

[group('build')]
build_orchestrator:
  just build_canister "orchestrator" "orchestrator"

[private]
pre_build: 
  mkdir -p "{{WASM_DIR}}"

[private]
build_canister canister_name wasm_name:
  mkdir -p "{{WASM_DIR}}"
  echo "Building {{canister_name}} Canister"
  cargo build --target wasm32-unknown-unknown --release --package "{{canister_name}}"
  ic-wasm "target/wasm32-unknown-unknown/release/{{canister_name}}.wasm" -o "{{WASM_DIR}}/{{wasm_name}}.wasm" shrink
  candid-extractor "{{WASM_DIR}}/{{wasm_name}}.wasm" > "{{WASM_DIR}}/{{wasm_name}}.did"
  cp "{{WASM_DIR}}/{{wasm_name}}.did" ./candid/{{wasm_name}}.did
  gzip -k "{{WASM_DIR}}/{{wasm_name}}.wasm" --force
