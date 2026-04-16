# MATCH Status (Program `7v9s...`)

Updated: 2026-04-16

## Current result

- `MATCH`: `PENDING PUBLIC VERIFY`
- Local executable hash: `1e6a4ce1ed7c915da2da1192b43cf3dbbda83470619ecb10fd7f3809af591055`
- On-chain executable hash: `1e6a4ce1ed7c915da2da1192b43cf3dbbda83470619ecb10fd7f3809af591055`

## Recheck command

```bash
bash scripts/check_match.sh
```

## verify-from-repo command

```bash
solana-verify verify-from-repo \
  --program-id 7v9sLrk92NNLLUfXLJw3o7MycZNvwsTK6kLWfWb8vcVA \
  --library-name sterling_amm \
  https://github.com/Ibrahimsterlingjomany/sterling-amm-mainnet-7v9-verifiable-build
```

Latest run note:

- The canonical verification repo now rebuilds the same local hash as the deployed program.
- Next step is publishing the verification record from the synced GitHub commit.

## Target state

- `MATCH`: `TRUE`
- Same hash output for local and on-chain.

## Notes

- This repo tracks deterministic verification evidence for Solscan and public auditability.
- The mismatch is build reproducibility related; it is not a proof of program compromise by itself.
