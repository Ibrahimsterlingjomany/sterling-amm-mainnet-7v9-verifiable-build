# Sterling DEX final status - 2026-04-18

## Identity

- Public GitHub project name: `Sterling DEX`
- On-chain program / library name: `sterling_amm`
- Program ID: `7v9sLrk92NNLLUfXLJw3o7MycZNvwsTK6kLWfWb8vcVA`

## Live verified runtime

- Live executable hash: `5a8be3af9d8303b6b9912240d2a78810bc260dc8c911bf0b8b6b69081dde37df`
- Verified commit pinned on-chain: `c40fb266b5d19bc259da56cd2b7274e0a740ca37`
- Verification refresh tx: `61G4tw4DZxyixgmQpjGsRB9hPJXFa9dv9fRFUCtcrLNBuEJhU6MNaM38aKstpYvjTAGcVBKRq66jwbBUNjy3MNyX`

## Final table

| Sujet | Statut |
|---|---|
| Runtime normal verified restored live | `FAIT` |
| Migration V1 -> V2 des `PayoutTicket` | `FAIT` |
| Migration V1 -> V2 des `SettlementClaim` | `FAIT` |
| Migration V1 -> V2 des `ProtocolDebtLedger` | `FAIT` |
| Config, pools, vaults, rails live conserves | `FAIT` |
| Pool registry live utile | `FAIT` |
| Asset/value registries principaux live utiles | `FAIT` |
| Asset/value registries secondaires non critiques | `PARTIEL / OPTIONNEL` |
| Match True runtime normal | `FAIT` |

## Notes

- The runtime normal remains the canonical public verified state.
- Temporary migration binaries were used only for short admin passes, then removed from the live state.
- This branch may contain migration support work used operationally, while the live verification proof remains pinned to the exact verified runtime commit above.
