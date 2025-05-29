import "./just/build.just"
import "./just/code_check.just"
import "./just/dependencies.just"
import "./just/deploy.just"
import "./just/orbit.just"
import "./just/orchestrator.just"
import "./just/test.just"

export RUST_BACKTRACE := "full"
WASM_DIR := env("WASM_DIR", "./.artifact")

# Lists all the available commands
default:
  @just --list
