#!/usr/bin/env python3

import base64
import hashlib
import zlib
from sys import argv, exit


def principal_str_to_bytes(principal_str: str) -> bytes:
    # Rimuove i trattini
    cleaned = principal_str.replace("-", "").upper()

    # La base32 standard in Python richiede che la stringa sia paddingata con '='
    # Deve essere un multiplo di 8 caratteri
    padding = "=" * ((8 - len(cleaned) % 8) % 8)
    b32_str = cleaned + padding

    # Decodifica base32
    decoded = base64.b32decode(b32_str)

    # Verifica CRC32 (i primi 4 byte)
    checksum = decoded[:4]
    data = decoded[4:]
    expected_checksum = zlib.crc32(data).to_bytes(4, byteorder="big")

    if checksum != expected_checksum:
        raise ValueError("Checksum non valido per il principal")

    return data  # sono i bytes del principal


def generate_account_id(owner: bytes, subaccount: bytes) -> bytes:
    # SHA-224
    hasher = hashlib.sha224()
    hasher.update(b"\x0aaccount-id")
    hasher.update(owner)
    hasher.update(subaccount)
    hash_digest = hasher.digest()  # 28 bytes

    # CRC32
    crc32 = zlib.crc32(hash_digest) & 0xFFFFFFFF
    crc32_bytes = crc32.to_bytes(4, byteorder="big")

    # Final result: 4 bytes of CRC32 + 28 bytes of hash
    result = crc32_bytes + hash_digest
    return result  # 32 bytes


if len(argv) != 2:
    print("Usage: get_account_id.py <principal>")
    exit(1)

principal = argv[1]

principal_bytes = principal_str_to_bytes(principal)

# Subaccount is empty, so we use 32 bytes of zeroes
subaccount = bytes(32)
account_id = generate_account_id(principal_bytes, subaccount)

# Print the account ID as a hex string
# print("0x" + account_id.hex().upper())

print(f"{'\\'.join(f"{byte:02x}" for byte in account_id)}")
