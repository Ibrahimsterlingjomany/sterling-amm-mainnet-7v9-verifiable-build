# MATCH Status (Program `7v9s...`)

Updated: 2026-04-15

## Current result

- `MATCH`: `FALSE`
- Local executable hash: `0f72bfbcf021333a6c0c96de7880c7707d41f1d9af0da2c161b381c99d3cf429`
- On-chain executable hash: `16c7748627f7114c1a155654ee1c788bd1cd4ad40f8a5f302f2aeaf3994ef85d`

## Recheck command

```bash
bash scripts/check_match.sh
```

## Target state

- `MATCH`: `TRUE`
- Same hash output for local and on-chain.

## Notes

- This repo tracks deterministic verification evidence for Solscan and public auditability.
- The mismatch is build reproducibility related; it is not a proof of program compromise by itself.
