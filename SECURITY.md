# Security Policy

## Project identity

- Public project name: `Sterling DEX`
- On-chain program / library name: `sterling_amm`
- Program ID: `7v9sLrk92NNLLUfXLJw3o7MycZNvwsTK6kLWfWb8vcVA`

## Reporting a vulnerability

Please report security issues privately through GitHub Security Advisories for this repository:

- https://github.com/Ibrahimsterlingjomany/sterling-amm-mainnet-7v9-verifiable-build/security/advisories/new

If GitHub private reporting is unavailable, open a private contact request through the repository owner profile instead of posting exploit details publicly.

## Scope

This repository is the verifiable build and operational audit workspace for the deployed Solana program above.

Security-relevant scope includes:

- deterministic build inputs for `sterling_amm`
- verification metadata and public audit trail
- migration support used to move legacy on-chain accounts to current layouts
- runtime-normal hash alignment and public verification state

## Current verified live state

- Verified runtime executable hash: `5a8be3af9d8303b6b9912240d2a78810bc260dc8c911bf0b8b6b69081dde37df`
- Verification repo commit pinned on-chain: `c40fb266b5d19bc259da56cd2b7274e0a740ca37`

## Disclosure guidance

- Do not publish private keys, seed phrases, authority material, or exploitable details in public issues.
- Do not post proof-of-concept transaction sequences that can directly drain or lock user funds before maintainers confirm remediation.

## Languages

Preferred disclosure languages:

- French
- English
