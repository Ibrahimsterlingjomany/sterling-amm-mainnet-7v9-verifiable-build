#!/usr/bin/env bash
set -euo pipefail

TARGET_PROGRAM_ID="${TARGET_PROGRAM_ID:-7v9sLrk92NNLLUfXLJw3o7MycZNvwsTK6kLWfWb8vcVA}"
TARGET_SO_PATH="${TARGET_SO_PATH:-target/deploy/sterling_amm.so}"

if ! command -v solana-verify >/dev/null 2>&1; then
  echo "ERROR: solana-verify not found in PATH" >&2
  exit 1
fi

if [ ! -f "$TARGET_SO_PATH" ]; then
  echo "ERROR: missing local binary: $TARGET_SO_PATH" >&2
  echo "Run: solana-verify build . --library-name sterling_amm_mainnet" >&2
  exit 1
fi

LOCAL_HASH="$(solana-verify get-executable-hash "$TARGET_SO_PATH" | tail -n 1 | tr -d '\r')"
ONCHAIN_HASH="$(solana-verify get-program-hash "$TARGET_PROGRAM_ID" | tail -n 1 | tr -d '\r')"

echo "program_id=$TARGET_PROGRAM_ID"
echo "local_hash=$LOCAL_HASH"
echo "onchain_hash=$ONCHAIN_HASH"

if [ "$LOCAL_HASH" = "$ONCHAIN_HASH" ]; then
  echo "MATCH=TRUE"
  exit 0
fi

echo "MATCH=FALSE"
exit 2
