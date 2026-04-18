# MATCH Status (Program `7v9s...`)

Updated: 2026-04-18

## Current result

- `MATCH`: `TRUE`
- Live runtime executable hash: `5a8be3af9d8303b6b9912240d2a78810bc260dc8c911bf0b8b6b69081dde37df`
- On-chain executable hash: `5a8be3af9d8303b6b9912240d2a78810bc260dc8c911bf0b8b6b69081dde37df`
- Verification repo commit pinned on-chain: `c40fb266b5d19bc259da56cd2b7274e0a740ca37`
- Verification refresh tx: `61G4tw4DZxyixgmQpjGsRB9hPJXFa9dv9fRFUCtcrLNBuEJhU6MNaM38aKstpYvjTAGcVBKRq66jwbBUNjy3MNyX`

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

- The live program has been returned to the verified runtime normal.
- Public verification data has been refreshed on-chain after the temporary migration passes.
- Legacy V1 -> V2 migrations are complete for payout tickets, settlement claims, and protocol debt ledgers.

## Target state

- `MATCH`: `TRUE`
- Same hash output for local and on-chain.

## Notes

- This repo tracks deterministic verification evidence for Solscan and public auditability.
- Public project name: `Sterling DEX`
- Program crate and deployed binary name: `sterling_amm`
