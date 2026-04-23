# sterling_amm 7v9 verifiable build workspace

Clean and isolated workspace for deterministic verification of the deployed Solana program.

## Canonical identifiers

- Program crate/library: `sterling_amm`
- Program ID: `7v9sLrk92NNLLUfXLJw3o7MycZNvwsTK6kLWfWb8vcVA`
- Upgrade authority: `CMqD45Kq5oukPvaMDhzav5RxJqZb1xME1MmV71CzCeTw`
- Workspace path: `verifiable-build/sterling_amm_mainnet_7v9`

This workspace is intentionally separated from the legacy mono-workspace to avoid hash drift from unrelated files, old toolchain state, and historical `declare_id!` variants.

## Goal

Produce `MATCH: TRUE` between:

- local executable hash from `target/deploy/sterling_amm.so`
- on-chain executable hash for program `7v9s...`

## Quick status

Current status is tracked in [`MATCH_STATUS.md`](./MATCH_STATUS.md).

## Reproducible flow

1. Build in verifiable mode:

```bash
solana-verify build . --library-name sterling_amm
```

2. Check local hash and on-chain hash in one command:

```bash
bash scripts/check_match.sh
```

3. If hashes differ, align toolchain and dependencies, then rebuild and recheck.

## Verify directly from GitHub

After pushing this repo, run:

```bash
solana-verify verify-from-repo \
  --program-id 7v9sLrk92NNLLUfXLJw3o7MycZNvwsTK6kLWfWb8vcVA \
  --library-name sterling_amm \
  https://github.com/Ibrahimsterlingjomany/sterling-amm-mainnet-7v9-verifiable-build
```

Prerequisite:

- Docker daemon must be running locally (required by `solana-verify verify-from-repo`).

## Current verified hash

The deployed program and the live upgrade artifact currently match on the executable hash:

- `690b053b728af67e1412b27ccde2c0302159e5358e99230188b7d175d5d5409a`

Direct commands used:

```bash
solana-verify get-executable-hash \
  /Users/ibrahimjomanysterling/Sterling_Bridge_M3_local/runtime/activation/probe_project_light/target/deploy/sterling_amm.so

solana-verify get-program-hash \
  7v9sLrk92NNLLUfXLJw3o7MycZNvwsTK6kLWfWb8vcVA
```

## Solscan verification artifacts

This repo is intended to host the deterministic build evidence and the exact command trail used for Solscan contract verification.
