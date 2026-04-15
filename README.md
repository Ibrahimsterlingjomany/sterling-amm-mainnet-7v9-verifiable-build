# sterling_amm_mainnet_7v9 verifiable build workspace

Clean and isolated workspace for deterministic verification of the deployed Solana program.

## Canonical identifiers

- Program name: `sterling_amm_mainnet`
- Program ID: `7v9sLrk92NNLLUfXLJw3o7MycZNvwsTK6kLWfWb8vcVA`
- Upgrade authority: `CMqD45Kq5oukPvaMDhzav5RxJqZb1xME1MmV71CzCeTw`
- Workspace path: `verifiable-build/sterling_amm_mainnet_7v9`

This workspace is intentionally separated from the legacy mono-workspace to avoid hash drift from unrelated files, old toolchain state, and historical `declare_id!` variants.

## Goal

Produce `MATCH: TRUE` between:

- local executable hash from `target/deploy/sterling_amm_mainnet.so`
- on-chain executable hash for program `7v9s...`

## Quick status

Current status is tracked in [`MATCH_STATUS.md`](./MATCH_STATUS.md).

## Reproducible flow

1. Build in verifiable mode:

```bash
solana-verify build . --library-name sterling_amm_mainnet
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
  https://github.com/<org-or-user>/<repo>
```

## Solscan verification artifacts

This repo is intended to host the deterministic build evidence and the exact command trail used for Solscan contract verification.
