# Sterling DEX verifiable build workspace

Clean and isolated workspace for deterministic verification of the deployed Solana program.

## Canonical identifiers

- Public project name: `Sterling DEX`
- On-chain program / library name: `sterling_amm`
- Program ID: `7v9sLrk92NNLLUfXLJw3o7MycZNvwsTK6kLWfWb8vcVA`
- Upgrade authority: `CMqD45Kq5oukPvaMDhzav5RxJqZb1xME1MmV71CzCeTw`
- Workspace path: `verifiable-build/sterling_amm_mainnet_7v9_clean`

This workspace is intentionally separated from the legacy mono-workspace to avoid hash drift from unrelated files, old toolchain state, and historical `declare_id!` variants.

## Naming

This repository is published as the verification and operational workspace for `Sterling DEX`.
The Solana program crate and deployed binary remain named `sterling_amm`.

## Goal

Produce `MATCH: TRUE` between:

- local executable hash from `target/deploy/sterling_amm.so`
- on-chain executable hash for program `7v9s...`

## Quick status

Current status is tracked in [`MATCH_STATUS.md`](./MATCH_STATUS.md).

The final migration and live-state summary is tracked in
[`FINAL_STATUS_2026-04-18.md`](./FINAL_STATUS_2026-04-18.md).

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
  https://github.com/<org-or-user>/<repo>
```

Prerequisite:

- Docker daemon must be running locally (required by `solana-verify verify-from-repo`).

## Solscan verification artifacts

This repo is intended to host the deterministic build evidence and the exact command trail used for Solscan contract verification.
