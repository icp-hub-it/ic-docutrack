#!/usr/bin/env python3

from sys import argv, exit

if len(argv) != 2:
    print("Usage: dfx_wasm.py <path_to_wasm_file>")
    exit(1)

wasm_file = argv[1]
if not wasm_file.endswith(".wasm") and not wasm_file.endswith(".wasm.gz"):
    print("Error: The file must be a .wasm or .wasm.gz file.")
    exit(1)

# open file
with open(wasm_file, "rb") as f:
    # read byte one at a time
    bytes = []
    for byte in f.read():
        # print byte as hex
        bytes.append(byte)

    print(f"{'\\'.join(f"{byte:02x}" for byte in bytes)}")

    f.close()
