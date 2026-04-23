# MATCH Status (Program `7v9s...`)

Updated: 2026-04-23

## Current result

- `MATCH`: `TRUE`
- Local executable hash: `690b053b728af67e1412b27ccde2c0302159e5358e99230188b7d175d5d5409a`
- On-chain executable hash: `690b053b728af67e1412b27ccde2c0302159e5358e99230188b7d175d5d5409a`

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

- Direct hash verification is now confirmed from the live upgrade artifact used for mainnet deployment.
- `solana-verify get-executable-hash` on `/Users/ibrahimjomanysterling/Sterling_Bridge_M3_local/runtime/activation/probe_project_light/target/deploy/sterling_amm.so` equals the on-chain program hash.
- Docker-based `verify-from-repo` is prepared and may require retry when the `solanafoundation/solana-verifiable-build` image pull is unstable on the local network.

## Notes

- This repo tracks deterministic verification evidence for Solscan and public auditability.
- The canonical live hash is now confirmed as `690b053b728af67e1412b27ccde2c0302159e5358e99230188b7d175d5d5409a`.
