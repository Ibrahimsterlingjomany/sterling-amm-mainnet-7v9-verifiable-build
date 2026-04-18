use crate::helpers::pubkey_from_str;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::get_associated_token_address;
use anchor_spl::token::{
    self, Burn, CloseAccount, Mint, MintTo, SetAuthority, Token, TokenAccount, Transfer,
};
use core::convert::TryFrom;
use solana_security_txt::security_txt;
use std::str::FromStr;

declare_id!("7v9sLrk92NNLLUfXLJw3o7MycZNvwsTK6kLWfWb8vcVA");

security_txt! {
    name: "Sterling AMM",
    project_url: "https://sterlingchain.net",
    contacts: "email:contact@sterlingchain.net,link:https://www.sterlingchain.net",
    policy: "https://sterlingchain.net",
    preferred_languages: "fr,en",
    auditors: "Sterling Ibrahim Jomany proprietaire de Sterling AMM/Sterling DEX/Sterling Chain multichain"
}

// =========================
// CONSTANTES (Mints / LP / Coffres)
// =========================

// "Sterling" mints (exemples)
pub const TREASURY_ROOT_MINT: &str = "7oDogFJzuxFpcW3R8hxqWRyAc8UsTBvQ83mWC8Q6C5H7";
pub const SJBC_MINT: &str = "9kued2JXgVk5dzvtipsTdXfBMWihy1E55TwMiXchCoAb";
pub const SJBC2_MINT: &str = "EsNo61QodqHCRjkTGJDeqyK7N4Hunip5PaTYbpPZEsG2";
pub const SJBC3_MINT: &str = "FwCp9JvCxtC88rJNWJSZFcoSUDW8NuSzKKSwt7MPF9of";

// Stablecoins (Solana mainnet)
pub const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

// ✅ USDT correct (Solana mainnet)
pub const USDT_MAIN_MINT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
pub const WSOL_MINT: &str = "So11111111111111111111111111111111111111112";

// Portal BTC (si tu t'en sers)
pub const BTC_PORTAL_MINT: &str = "9n4nbM75f5Ui33ZbPYXn59EwSgE8CGsHtAeTH5YFeJ9E";

// LPs
pub const LP_MINT_2: &str = "G94nkBm4ntjiEHNzTpd7GRW9J8H5rqrhW83k5RSHZrBZ";
pub const LP_AUTH_2: &str = "Htopqis52g8nGvvkpnG7Z7XZhgBpqtN9huqUyk6LH9gB";
pub const LP_MINT_3: &str = "DnepvMafJZzDtDcevrqbMUCmDqdNhBLjCTUu1xhR4HeL";
pub const LP_AUTH_3: &str = "CMqD45Kq5oukPvaMDhzav5RxJqZb1xME1MmV71CzCeTw";

// HTOP sovereign reserve materialization rails
pub const OPERATOR_STM_RESERVE_ATA: &str = "Dnxtm9v64m6QFfVMNUEPDJN19W97L8r45rndPJWmMgqQ";
pub const OPERATOR_SJBC_RESERVE_ATA: &str = "ECXHuS3GZ6FvqVge6drGoKcYQZQbhaiBSq5sYc1J2raV";
pub const HTOP_STM_RESERVE_ATA: &str = "2CRon3SyMyvy2i7hourX99kiuoTKpLgQ3ebogrpfDorq";
pub const HTOP_SJBC_RESERVE_ATA: &str = "HEy89xU9gkEi9FXGLvzT61i3pM2kTW5MzvqcCDsB7EmQ";

// New mints
pub const MINT_H1: &str = "H1hutftquntrMVid7btBsBw4XLjGLVwxfv9exYdMwsr9";
pub const MINT_FR15: &str = "FR15AHXBE8vDpYASFmCbbCD6yso4XiD6XiEXAF6dGLix";
pub const MINT_DB8: &str = "DB8gCpC6Qs1c4CPfvPXT7RcFmqxzY5g6i8Djo5j5A2nq";

// Coffres (token accounts SPL)
//
// IMPORTANT:
// - ces comptes servent au rail technique de settlement;
// - ils ne representent pas, a eux seuls, la valeur economique originelle
//   des claims / tickets;

#[derive(Clone)]
#[allow(dead_code)]
struct SovereignRemainingAccounts<'info> {
    source_primary: Option<AccountInfo<'info>>,
    source_secondary: Option<AccountInfo<'info>>,
    alt_vault: Option<AccountInfo<'info>>,
    alt_destination: Option<AccountInfo<'info>>,
}

#[derive(Clone)]
struct SovereignLiveSwapAccounts<'info> {
    token_mint: AccountInfo<'info>,
    asset_registry: Option<AccountInfo<'info>>,
    user_token_ata: AccountInfo<'info>,
    pool_info: AccountInfo<'info>,
    input_vault: AccountInfo<'info>,
    output_vault: AccountInfo<'info>,
    fee_vault_in: AccountInfo<'info>,
    output_destination: AccountInfo<'info>,
    native_destination: Option<AccountInfo<'info>>,
}

#[derive(Clone)]
struct SovereignEscrowLiveSwapAccounts<'info> {
    source_record: AccountInfo<'info>,
    source_escrow_authority: AccountInfo<'info>,
    source_escrow_ata: AccountInfo<'info>,
    swap_pool_info: AccountInfo<'info>,
    input_vault: AccountInfo<'info>,
    output_vault: AccountInfo<'info>,
    fee_vault_in: AccountInfo<'info>,
    output_destination: AccountInfo<'info>,
    native_destination: Option<AccountInfo<'info>>,
    ledger_info: Option<AccountInfo<'info>>,
}

#[derive(Clone)]
struct SovereignConvertRouteAccounts<'info> {
    token_mint: Option<AccountInfo<'info>>,
    asset_registry: Option<AccountInfo<'info>>,
    user_token_ata: Option<AccountInfo<'info>>,
    usdc_vault: AccountInfo<'info>,
    usdc_destination: AccountInfo<'info>,
    sol_destination: Option<AccountInfo<'info>>,
    source_primary: Option<AccountInfo<'info>>,
    source_secondary: Option<AccountInfo<'info>>,
    source_escrow_authority: Option<AccountInfo<'info>>,
    source_escrow_ata: Option<AccountInfo<'info>>,
    source_escrow_mint: Option<AccountInfo<'info>>,
    alt_vault: Option<AccountInfo<'info>>,
    alt_destination: Option<AccountInfo<'info>>,
}

#[derive(Clone)]
struct SovereignBufferedEscrowAccounts<'info> {
    authority_info: AccountInfo<'info>,
    ata_info: AccountInfo<'info>,
    mint_info: AccountInfo<'info>,
}

#[derive(Clone)]
#[allow(dead_code)]
struct StableFinalOutputTarget<'info> {
    final_output_kind: SovereignFinalOutputKind,
    payout_mint: Pubkey,
    vault_info: Option<AccountInfo<'info>>,
    vault_amount: u64,
    destination_info: AccountInfo<'info>,
    destination_key: Pubkey,
}

#[derive(Clone)]
struct NativeSolOutputTarget<'info> {
    destination_info: AccountInfo<'info>,
    destination_key: Pubkey,
    available_lamports: u64,
}

#[derive(Clone)]
struct SovereignBufferedSettlementTargets<'info> {
    primary_target: Option<StableFinalOutputTarget<'info>>,
    secondary_target: Option<StableFinalOutputTarget<'info>>,
    native_target: Option<NativeSolOutputTarget<'info>>,
}

#[derive(Clone, Copy)]
struct SovereignDebtPayoutPolicy {
    main_wallet: Pubkey,
    canonical_usdc_mint: Pubkey,
    canonical_usdt_mint: Pubkey,
    extra_payout_mint_0: Pubkey,
    extra_payout_mint_1: Pubkey,
    extra_payout_mint_2: Pubkey,
    extra_payout_mint_3: Pubkey,
    treasury_usdc_ata: Pubkey,
    treasury_usdt_ata: Pubkey,
}

#[derive(Clone, Copy)]
struct NativeSolRailConfig {
    native_sol_usd_micros_per_sol: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LiveSovereignSettlementKind {
    Stable,
    NativeSol,
}

#[derive(Clone, Copy)]
struct LivePoolExecutionOutcome {
    source_mint: Pubkey,
    source_amount: u64,
    payout_mint: Pubkey,
    payout_amount: u64,
    destination_key: Pubkey,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LivePoolQuote {
    input_is_base: bool,
    output_mint: Pubkey,
    settlement_kind: LiveSovereignSettlementKind,
    fee_amount: u64,
    payout_amount: u64,
}

#[allow(dead_code)]
fn quote_live_pool_swap_parts(
    pool_active: bool,
    pool_base_mint: Pubkey,
    pool_quote_mint: Pubkey,
    pool_base_vault: Pubkey,
    pool_quote_vault: Pubkey,
    pool_fee_vault_base: Pubkey,
    pool_fee_vault_quote: Pubkey,
    pool_fee_bps: u16,
    source_mint: Pubkey,
    amount_in: u64,
    input_vault_key: Pubkey,
    output_vault_key: Pubkey,
    fee_vault_in_key: Pubkey,
    input_vault_mint: Pubkey,
    output_vault_mint: Pubkey,
    fee_vault_in_mint: Pubkey,
    input_vault_amount: u64,
    output_vault_amount: u64,
    requested_output_kind: SovereignFinalOutputKind,
    config_usdc_mint: Pubkey,
    config_usdt_mint: Pubkey,
    extra_payout_mint_0: Pubkey,
    extra_payout_mint_1: Pubkey,
    extra_payout_mint_2: Pubkey,
    extra_payout_mint_3: Pubkey,
) -> Result<LivePoolQuote> {
    require!(pool_active, SterlingError::InactivePool);
    require!(amount_in > 0, SterlingError::InvalidAmount);

    let input_is_base = if pool_base_mint == source_mint {
        require!(
            input_vault_key == pool_base_vault,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            output_vault_key == pool_quote_vault,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            fee_vault_in_key == pool_fee_vault_base,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            input_vault_mint == pool_base_mint,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            output_vault_mint == pool_quote_mint,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            fee_vault_in_mint == pool_base_mint,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        true
    } else if pool_quote_mint == source_mint {
        require!(
            input_vault_key == pool_quote_vault,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            output_vault_key == pool_base_vault,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            fee_vault_in_key == pool_fee_vault_quote,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            input_vault_mint == pool_quote_mint,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            output_vault_mint == pool_base_mint,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            fee_vault_in_mint == pool_quote_mint,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        false
    } else {
        return err!(SterlingError::InvalidLiveSovereignPoolRoute);
    };

    let output_mint = if input_is_base {
        pool_quote_mint
    } else {
        pool_base_mint
    };
    let settlement_kind = classify_live_sovereign_settlement_kind(
        requested_output_kind,
        output_mint,
        config_usdc_mint,
        config_usdt_mint,
        extra_payout_mint_0,
        extra_payout_mint_1,
        extra_payout_mint_2,
        extra_payout_mint_3,
    )?;

    let fee_amount = (amount_in as u128)
        .saturating_mul(pool_fee_bps as u128)
        .checked_div(10_000u128)
        .ok_or(SterlingError::MathOverflow)? as u64;
    let amount_in_less_fee = (amount_in as u128)
        .saturating_mul(10_000u128.saturating_sub(pool_fee_bps as u128))
        .checked_div(10_000u128)
        .ok_or(SterlingError::MathOverflow)? as u64;
    let x = input_vault_amount as u128;
    let y = output_vault_amount as u128;
    let payout_amount = (amount_in_less_fee as u128)
        .saturating_mul(y)
        .checked_div(x.saturating_add(amount_in_less_fee as u128))
        .ok_or(SterlingError::MathOverflow)? as u64;

    require!(payout_amount > 0, SterlingError::InvalidAmount);
    require!(
        output_vault_amount >= payout_amount,
        SterlingError::InsufficientLiquidity
    );

    Ok(LivePoolQuote {
        input_is_base,
        output_mint,
        settlement_kind,
        fee_amount,
        payout_amount,
    })
}

#[allow(dead_code)]
fn validate_live_output_destination_parts(
    settlement_kind: LiveSovereignSettlementKind,
    beneficiary: Pubkey,
    output_destination_owner: Pubkey,
    output_destination_mint: Pubkey,
    output_destination_amount: u64,
    output_destination_key: Pubkey,
    expected_output_mint: Pubkey,
    config_key: Pubkey,
    native_destination_key: Option<Pubkey>,
    native_destination_is_writable: bool,
    native_destination_executable: bool,
) -> Result<()> {
    match settlement_kind {
        LiveSovereignSettlementKind::Stable => {
            require!(
                output_destination_owner == beneficiary,
                SterlingError::InvalidAccount
            );
            require!(
                output_destination_mint == expected_output_mint,
                SterlingError::InvalidAccount
            );
        }
        LiveSovereignSettlementKind::NativeSol => {
            require!(
                output_destination_mint == expected_output_mint,
                SterlingError::InvalidAccount
            );
            require!(
                output_destination_owner == config_key,
                SterlingError::InvalidLiveSovereignPoolRoute
            );
            let native_destination_key = native_destination_key
                .ok_or_else(|| error!(SterlingError::InvalidNativeSolDestination))?;
            require!(
                native_destination_key == beneficiary,
                SterlingError::InvalidNativeSolDestination
            );
            require!(
                native_destination_is_writable,
                SterlingError::InvalidNativeSolDestination
            );
            require!(
                !native_destination_executable,
                SterlingError::InvalidNativeSolDestination
            );
            require!(
                native_destination_key != config_key,
                SterlingError::InvalidNativeSolDestination
            );
            assert_wsol_temporary_output_account_parts(
                output_destination_mint,
                output_destination_owner,
                output_destination_amount,
                output_destination_key,
                config_key,
                native_destination_key,
            )?;
        }
    }
    Ok(())
}

fn begin_ticket_live_execution(
    ticket: &mut PayoutTicket,
    executor: Pubkey,
    keeper_authority: Pubkey,
) -> Result<()> {
    require!(
        ticket.user == executor || executor == keeper_authority,
        SterlingError::InvalidAccount
    );
    require!(
        ticket.status == PAYOUT_TICKET_STATUS_REQUESTED,
        SterlingError::InvalidState
    );
    require!(ticket.settled_ts == 0, SterlingError::InvalidState);
    require!(
        ticket.funding_state == SOVEREIGN_ESCROW_STATE_FUNDED_IN_ESCROW,
        SterlingError::SovereignEscrowNotFunded
    );
    ticket.funding_state = SOVEREIGN_ESCROW_STATE_EXECUTED_LIVE;
    Ok(())
}

fn finalize_ticket_live_execution(
    ticket: &mut PayoutTicket,
    outcome: LivePoolExecutionOutcome,
    now: i64,
) -> Result<()> {
    require!(
        ticket.status == PAYOUT_TICKET_STATUS_REQUESTED,
        SterlingError::InvalidState
    );
    require!(ticket.settled_ts == 0, SterlingError::InvalidState);
    require!(
        ticket.funding_state == SOVEREIGN_ESCROW_STATE_EXECUTED_LIVE,
        SterlingError::InvalidState
    );

    ticket.payout_mint = outcome.payout_mint;
    ticket.destination_ata = outcome.destination_key;
    ticket.status = PAYOUT_TICKET_STATUS_SETTLED;
    ticket.funding_state = SOVEREIGN_ESCROW_STATE_SETTLED;
    ticket.settled_ts = now;
    Ok(())
}

fn begin_claim_live_execution(claim: &mut SettlementClaim, expected_user: Pubkey) -> Result<()> {
    require!(claim.user == expected_user, SterlingError::InvalidAccount);
    require!(
        claim.status == SETTLEMENT_CLAIM_STATUS_OPEN,
        SterlingError::InvalidState
    );
    require!(claim.settled_ts == 0, SterlingError::InvalidState);
    require!(
        claim.funding_state == SOVEREIGN_ESCROW_STATE_FUNDED_IN_ESCROW,
        SterlingError::SovereignEscrowNotFunded
    );
    claim.funding_state = SOVEREIGN_ESCROW_STATE_EXECUTED_LIVE;
    Ok(())
}

fn begin_claim_live_execution_by_executor(
    claim: &mut SettlementClaim,
    executor: Pubkey,
    keeper_authority: Pubkey,
) -> Result<()> {
    require!(
        claim.user == executor || executor == keeper_authority,
        SterlingError::InvalidAccount
    );
    require!(
        claim.status == SETTLEMENT_CLAIM_STATUS_OPEN,
        SterlingError::InvalidState
    );
    require!(claim.settled_ts == 0, SterlingError::InvalidState);
    require!(
        claim.funding_state == SOVEREIGN_ESCROW_STATE_FUNDED_IN_ESCROW,
        SterlingError::SovereignEscrowNotFunded
    );
    claim.funding_state = SOVEREIGN_ESCROW_STATE_EXECUTED_LIVE;
    Ok(())
}

fn finalize_claim_live_execution(
    claim: &mut SettlementClaim,
    outcome: LivePoolExecutionOutcome,
    now: i64,
) -> Result<()> {
    require!(
        claim.status == SETTLEMENT_CLAIM_STATUS_OPEN,
        SterlingError::InvalidState
    );
    require!(claim.settled_ts == 0, SterlingError::InvalidState);
    require!(
        claim.funding_state == SOVEREIGN_ESCROW_STATE_EXECUTED_LIVE,
        SterlingError::InvalidState
    );

    claim.payout_mint = outcome.payout_mint;
    claim.destination_ata = outcome.destination_key;
    claim.paid_atoms = outcome.payout_amount;
    claim.proof_sig = live_execution_proof_sig();
    claim.settled_ts = now;
    claim.status = SETTLEMENT_CLAIM_STATUS_PAID;
    claim.funding_state = SOVEREIGN_ESCROW_STATE_SETTLED;
    Ok(())
}

fn begin_protocol_debt_lot_live_execution(lot: &mut ProtocolDebtLot) -> Result<()> {
    require!(
        lot.status == PROTOCOL_DEBT_LOT_OPEN || lot.status == PROTOCOL_DEBT_LOT_TICKETED,
        SterlingError::InvalidState
    );
    require!(lot.usd_micros > 0, SterlingError::InvalidAmount);
    require!(
        lot.funding_state == SOVEREIGN_ESCROW_STATE_FUNDED_IN_ESCROW,
        SterlingError::SovereignEscrowNotFunded
    );
    lot.funding_state = SOVEREIGN_ESCROW_STATE_EXECUTED_LIVE;
    Ok(())
}

fn finalize_protocol_debt_lot_live_execution(
    pool: &mut Pool,
    lot: &mut ProtocolDebtLot,
    now: i64,
) -> Result<()> {
    require!(
        lot.funding_state == SOVEREIGN_ESCROW_STATE_EXECUTED_LIVE,
        SterlingError::InvalidState
    );
    mark_protocol_debt_lot_settled(pool, lot, now)?;
    lot.funding_state = SOVEREIGN_ESCROW_STATE_SETTLED;
    Ok(())
}

fn sovereign_final_output_kind_from_u8(value: u8) -> Result<SovereignFinalOutputKind> {
    match value {
        0 => Ok(SovereignFinalOutputKind::Auto),
        1 => Ok(SovereignFinalOutputKind::Usdc),
        2 => Ok(SovereignFinalOutputKind::Usdt),
        3 => Ok(SovereignFinalOutputKind::Sol),
        4 => Ok(SovereignFinalOutputKind::Extra0),
        5 => Ok(SovereignFinalOutputKind::Extra1),
        6 => Ok(SovereignFinalOutputKind::Extra2),
        7 => Ok(SovereignFinalOutputKind::Extra3),
        _ => err!(SterlingError::UnsupportedSovereignFinalOutput),
    }
}

fn require_live_sovereign_source_kind(source_kind: u8) -> Result<()> {
    if matches!(source_kind, 1 | 2 | 3) {
        return Ok(());
    }

    err!(SterlingError::DeferredSovereignRouteNotLive)
}

fn payout_kind_matches_mint(
    requested_output_kind: SovereignFinalOutputKind,
    output_mint: Pubkey,
    config_usdc_mint: Pubkey,
    config_usdt_mint: Pubkey,
    extra_payout_mint_0: Pubkey,
    extra_payout_mint_1: Pubkey,
    extra_payout_mint_2: Pubkey,
    extra_payout_mint_3: Pubkey,
) -> bool {
    match requested_output_kind {
        SovereignFinalOutputKind::Usdc => output_mint == config_usdc_mint,
        SovereignFinalOutputKind::Usdt => output_mint == config_usdt_mint,
        SovereignFinalOutputKind::Extra0 => {
            extra_payout_mint_0 != Pubkey::default() && output_mint == extra_payout_mint_0
        }
        SovereignFinalOutputKind::Extra1 => {
            extra_payout_mint_1 != Pubkey::default() && output_mint == extra_payout_mint_1
        }
        SovereignFinalOutputKind::Extra2 => {
            extra_payout_mint_2 != Pubkey::default() && output_mint == extra_payout_mint_2
        }
        SovereignFinalOutputKind::Extra3 => {
            extra_payout_mint_3 != Pubkey::default() && output_mint == extra_payout_mint_3
        }
        SovereignFinalOutputKind::Auto | SovereignFinalOutputKind::Sol => false,
    }
}

fn is_configured_live_stable_output_mint(
    output_mint: Pubkey,
    config_usdc_mint: Pubkey,
    config_usdt_mint: Pubkey,
    extra_payout_mint_0: Pubkey,
    extra_payout_mint_1: Pubkey,
    extra_payout_mint_2: Pubkey,
    extra_payout_mint_3: Pubkey,
) -> bool {
    output_mint == config_usdc_mint
        || output_mint == config_usdt_mint
        || (extra_payout_mint_0 != Pubkey::default() && output_mint == extra_payout_mint_0)
        || (extra_payout_mint_1 != Pubkey::default() && output_mint == extra_payout_mint_1)
        || (extra_payout_mint_2 != Pubkey::default() && output_mint == extra_payout_mint_2)
        || (extra_payout_mint_3 != Pubkey::default() && output_mint == extra_payout_mint_3)
}

fn is_configured_extra_payout_mint(
    output_mint: Pubkey,
    extra_payout_mint_0: Pubkey,
    extra_payout_mint_1: Pubkey,
    extra_payout_mint_2: Pubkey,
    extra_payout_mint_3: Pubkey,
) -> bool {
    (extra_payout_mint_0 != Pubkey::default() && output_mint == extra_payout_mint_0)
        || (extra_payout_mint_1 != Pubkey::default() && output_mint == extra_payout_mint_1)
        || (extra_payout_mint_2 != Pubkey::default() && output_mint == extra_payout_mint_2)
        || (extra_payout_mint_3 != Pubkey::default() && output_mint == extra_payout_mint_3)
}

fn classify_live_sovereign_settlement_kind(
    requested_output_kind: SovereignFinalOutputKind,
    output_mint: Pubkey,
    config_usdc_mint: Pubkey,
    config_usdt_mint: Pubkey,
    extra_payout_mint_0: Pubkey,
    extra_payout_mint_1: Pubkey,
    extra_payout_mint_2: Pubkey,
    extra_payout_mint_3: Pubkey,
) -> Result<LiveSovereignSettlementKind> {
    match requested_output_kind {
        SovereignFinalOutputKind::Usdc
        | SovereignFinalOutputKind::Usdt
        | SovereignFinalOutputKind::Extra0
        | SovereignFinalOutputKind::Extra1
        | SovereignFinalOutputKind::Extra2
        | SovereignFinalOutputKind::Extra3 => {
            require!(
                payout_kind_matches_mint(
                    requested_output_kind,
                    output_mint,
                    config_usdc_mint,
                    config_usdt_mint,
                    extra_payout_mint_0,
                    extra_payout_mint_1,
                    extra_payout_mint_2,
                    extra_payout_mint_3,
                ),
                SterlingError::UnsupportedSovereignFinalOutput
            );
            Ok(LiveSovereignSettlementKind::Stable)
        }
        SovereignFinalOutputKind::Auto => {
            if is_configured_live_stable_output_mint(
                output_mint,
                config_usdc_mint,
                config_usdt_mint,
                extra_payout_mint_0,
                extra_payout_mint_1,
                extra_payout_mint_2,
                extra_payout_mint_3,
            ) {
                return Ok(LiveSovereignSettlementKind::Stable);
            }
            err!(SterlingError::UnsupportedSovereignFinalOutput)
        }
        SovereignFinalOutputKind::Sol => {
            require!(
                output_mint == pubkey_from_str(WSOL_MINT),
                SterlingError::NativeSolLiveExchangeNotReady
            );
            Ok(LiveSovereignSettlementKind::NativeSol)
        }
    }
}

fn parse_sovereign_remaining_accounts<'info>(
    source_kind: u8,
    remaining_accounts: &[AccountInfo<'info>],
) -> Result<SovereignRemainingAccounts<'info>> {
    let parsed = match source_kind {
        1 | 2 | 3 => match remaining_accounts {
            [] => SovereignRemainingAccounts {
                source_primary: None,
                source_secondary: None,
                alt_vault: None,
                alt_destination: None,
            },
            [alt_vault, alt_destination] => SovereignRemainingAccounts {
                source_primary: None,
                source_secondary: None,
                alt_vault: Some(alt_vault.clone()),
                alt_destination: Some(alt_destination.clone()),
            },
            _ => return err!(SterlingError::InvalidSovereignAccountsLayout),
        },
        4 | 5 => match remaining_accounts {
            [source_primary] => SovereignRemainingAccounts {
                source_primary: Some(source_primary.clone()),
                source_secondary: None,
                alt_vault: None,
                alt_destination: None,
            },
            [source_primary, alt_vault, alt_destination] => SovereignRemainingAccounts {
                source_primary: Some(source_primary.clone()),
                source_secondary: None,
                alt_vault: Some(alt_vault.clone()),
                alt_destination: Some(alt_destination.clone()),
            },
            _ => return err!(SterlingError::InvalidSovereignAccountsLayout),
        },
        6 => match remaining_accounts {
            [source_primary, source_secondary] => SovereignRemainingAccounts {
                source_primary: Some(source_primary.clone()),
                source_secondary: Some(source_secondary.clone()),
                alt_vault: None,
                alt_destination: None,
            },
            [source_primary, source_secondary, alt_vault, alt_destination] => {
                SovereignRemainingAccounts {
                    source_primary: Some(source_primary.clone()),
                    source_secondary: Some(source_secondary.clone()),
                    alt_vault: Some(alt_vault.clone()),
                    alt_destination: Some(alt_destination.clone()),
                }
            }
            _ => return err!(SterlingError::InvalidSovereignAccountsLayout),
        },
        _ => return err!(SterlingError::InvalidAccount),
    };

    Ok(parsed)
}

fn parse_sovereign_convert_route_accounts<'info>(
    source_kind: u8,
    remaining_accounts: &[AccountInfo<'info>],
) -> Result<SovereignConvertRouteAccounts<'info>> {
    fn looks_like_buffered_escrow_triplet<'info>(
        authority_info: &AccountInfo<'info>,
        escrow_ata_info: &AccountInfo<'info>,
        escrow_mint_info: &AccountInfo<'info>,
    ) -> bool {
        load_token_account_snapshot(authority_info).is_err()
            && load_mint_snapshot(authority_info).is_err()
            && load_token_account_snapshot(escrow_ata_info).is_ok()
            && load_mint_snapshot(escrow_mint_info).is_ok()
    }

    match source_kind {
        1 | 2 | 3 => match remaining_accounts {
            [token_mint, asset_registry, user_token_ata, usdc_vault, usdc_destination] => {
                Ok(SovereignConvertRouteAccounts {
                    token_mint: Some(token_mint.clone()),
                    asset_registry: Some(asset_registry.clone()),
                    user_token_ata: Some(user_token_ata.clone()),
                    usdc_vault: usdc_vault.clone(),
                    usdc_destination: usdc_destination.clone(),
                    sol_destination: None,
                    source_primary: None,
                    source_secondary: None,
                    source_escrow_authority: None,
                    source_escrow_ata: None,
                    source_escrow_mint: None,
                    alt_vault: None,
                    alt_destination: None,
                })
            }
            [token_mint, asset_registry, user_token_ata, usdc_vault, usdc_destination, sol_destination] => {
                Ok(SovereignConvertRouteAccounts {
                    token_mint: Some(token_mint.clone()),
                    asset_registry: Some(asset_registry.clone()),
                    user_token_ata: Some(user_token_ata.clone()),
                    usdc_vault: usdc_vault.clone(),
                    usdc_destination: usdc_destination.clone(),
                    sol_destination: Some(sol_destination.clone()),
                    source_primary: None,
                    source_secondary: None,
                    source_escrow_authority: None,
                    source_escrow_ata: None,
                    source_escrow_mint: None,
                    alt_vault: None,
                    alt_destination: None,
                })
            }
            [token_mint, asset_registry, user_token_ata, usdc_vault, usdc_destination, alt_vault, alt_destination] => {
                Ok(SovereignConvertRouteAccounts {
                    token_mint: Some(token_mint.clone()),
                    asset_registry: Some(asset_registry.clone()),
                    user_token_ata: Some(user_token_ata.clone()),
                    usdc_vault: usdc_vault.clone(),
                    usdc_destination: usdc_destination.clone(),
                    sol_destination: None,
                    source_primary: None,
                    source_secondary: None,
                    source_escrow_authority: None,
                    source_escrow_ata: None,
                    source_escrow_mint: None,
                    alt_vault: Some(alt_vault.clone()),
                    alt_destination: Some(alt_destination.clone()),
                })
            }
            [token_mint, asset_registry, user_token_ata, usdc_vault, usdc_destination, alt_vault, alt_destination, sol_destination] => {
                Ok(SovereignConvertRouteAccounts {
                    token_mint: Some(token_mint.clone()),
                    asset_registry: Some(asset_registry.clone()),
                    user_token_ata: Some(user_token_ata.clone()),
                    usdc_vault: usdc_vault.clone(),
                    usdc_destination: usdc_destination.clone(),
                    sol_destination: Some(sol_destination.clone()),
                    source_primary: None,
                    source_secondary: None,
                    source_escrow_authority: None,
                    source_escrow_ata: None,
                    source_escrow_mint: None,
                    alt_vault: Some(alt_vault.clone()),
                    alt_destination: Some(alt_destination.clone()),
                })
            }
            _ => err!(SterlingError::InvalidSovereignAccountsLayout),
        },
        4 | 5 if remaining_accounts.len() >= 6
            && looks_like_buffered_escrow_triplet(
                &remaining_accounts[1],
                &remaining_accounts[2],
                &remaining_accounts[3],
            ) =>
        {
            match remaining_accounts {
                [source_primary, source_escrow_authority, source_escrow_ata, source_escrow_mint, usdc_vault, usdc_destination] => Ok(SovereignConvertRouteAccounts {
                    token_mint: None,
                    asset_registry: None,
                    user_token_ata: None,
                    usdc_vault: usdc_vault.clone(),
                    usdc_destination: usdc_destination.clone(),
                    sol_destination: None,
                    source_primary: Some(source_primary.clone()),
                    source_secondary: None,
                    source_escrow_authority: Some(source_escrow_authority.clone()),
                    source_escrow_ata: Some(source_escrow_ata.clone()),
                    source_escrow_mint: Some(source_escrow_mint.clone()),
                    alt_vault: None,
                    alt_destination: None,
                }),
                [source_primary, source_escrow_authority, source_escrow_ata, source_escrow_mint, usdc_vault, usdc_destination, sol_destination] => {
                    Ok(SovereignConvertRouteAccounts {
                        token_mint: None,
                        asset_registry: None,
                        user_token_ata: None,
                        usdc_vault: usdc_vault.clone(),
                        usdc_destination: usdc_destination.clone(),
                        sol_destination: Some(sol_destination.clone()),
                        source_primary: Some(source_primary.clone()),
                        source_secondary: None,
                        source_escrow_authority: Some(source_escrow_authority.clone()),
                        source_escrow_ata: Some(source_escrow_ata.clone()),
                        source_escrow_mint: Some(source_escrow_mint.clone()),
                        alt_vault: None,
                        alt_destination: None,
                    })
                }
                [source_primary, source_escrow_authority, source_escrow_ata, source_escrow_mint, usdc_vault, usdc_destination, alt_vault, alt_destination] => {
                    Ok(SovereignConvertRouteAccounts {
                        token_mint: None,
                        asset_registry: None,
                        user_token_ata: None,
                        usdc_vault: usdc_vault.clone(),
                        usdc_destination: usdc_destination.clone(),
                        sol_destination: None,
                        source_primary: Some(source_primary.clone()),
                        source_secondary: None,
                        source_escrow_authority: Some(source_escrow_authority.clone()),
                        source_escrow_ata: Some(source_escrow_ata.clone()),
                        source_escrow_mint: Some(source_escrow_mint.clone()),
                        alt_vault: Some(alt_vault.clone()),
                        alt_destination: Some(alt_destination.clone()),
                    })
                }
                [source_primary, source_escrow_authority, source_escrow_ata, source_escrow_mint, usdc_vault, usdc_destination, alt_vault, alt_destination, sol_destination] => {
                    Ok(SovereignConvertRouteAccounts {
                        token_mint: None,
                        asset_registry: None,
                        user_token_ata: None,
                        usdc_vault: usdc_vault.clone(),
                        usdc_destination: usdc_destination.clone(),
                        sol_destination: Some(sol_destination.clone()),
                        source_primary: Some(source_primary.clone()),
                        source_secondary: None,
                        source_escrow_authority: Some(source_escrow_authority.clone()),
                        source_escrow_ata: Some(source_escrow_ata.clone()),
                        source_escrow_mint: Some(source_escrow_mint.clone()),
                        alt_vault: Some(alt_vault.clone()),
                        alt_destination: Some(alt_destination.clone()),
                    })
                }
                _ => err!(SterlingError::InvalidSovereignAccountsLayout),
            }
        }
        4 | 5 => match remaining_accounts {
            [source_primary, usdc_vault, usdc_destination] => Ok(SovereignConvertRouteAccounts {
                token_mint: None,
                asset_registry: None,
                user_token_ata: None,
                usdc_vault: usdc_vault.clone(),
                usdc_destination: usdc_destination.clone(),
                sol_destination: None,
                source_primary: Some(source_primary.clone()),
                source_secondary: None,
                source_escrow_authority: None,
                source_escrow_ata: None,
                source_escrow_mint: None,
                alt_vault: None,
                alt_destination: None,
            }),
            [source_primary, usdc_vault, usdc_destination, sol_destination] => {
                Ok(SovereignConvertRouteAccounts {
                    token_mint: None,
                    asset_registry: None,
                    user_token_ata: None,
                    usdc_vault: usdc_vault.clone(),
                    usdc_destination: usdc_destination.clone(),
                    sol_destination: Some(sol_destination.clone()),
                    source_primary: Some(source_primary.clone()),
                    source_secondary: None,
                    source_escrow_authority: None,
                    source_escrow_ata: None,
                    source_escrow_mint: None,
                    alt_vault: None,
                    alt_destination: None,
                })
            }
            [source_primary, usdc_vault, usdc_destination, alt_vault, alt_destination] => {
                Ok(SovereignConvertRouteAccounts {
                    token_mint: None,
                    asset_registry: None,
                    user_token_ata: None,
                    usdc_vault: usdc_vault.clone(),
                    usdc_destination: usdc_destination.clone(),
                    sol_destination: None,
                    source_primary: Some(source_primary.clone()),
                    source_secondary: None,
                    source_escrow_authority: None,
                    source_escrow_ata: None,
                    source_escrow_mint: None,
                    alt_vault: Some(alt_vault.clone()),
                    alt_destination: Some(alt_destination.clone()),
                })
            }
            [source_primary, usdc_vault, usdc_destination, alt_vault, alt_destination, sol_destination] => {
                Ok(SovereignConvertRouteAccounts {
                    token_mint: None,
                    asset_registry: None,
                    user_token_ata: None,
                    usdc_vault: usdc_vault.clone(),
                    usdc_destination: usdc_destination.clone(),
                    sol_destination: Some(sol_destination.clone()),
                    source_primary: Some(source_primary.clone()),
                    source_secondary: None,
                    source_escrow_authority: None,
                    source_escrow_ata: None,
                    source_escrow_mint: None,
                    alt_vault: Some(alt_vault.clone()),
                    alt_destination: Some(alt_destination.clone()),
                })
            }
            _ => err!(SterlingError::InvalidSovereignAccountsLayout),
        },
        6 if remaining_accounts.len() >= 7
            && looks_like_buffered_escrow_triplet(
                &remaining_accounts[2],
                &remaining_accounts[3],
                &remaining_accounts[4],
            ) =>
        {
            match remaining_accounts {
                [source_primary, source_secondary, source_escrow_authority, source_escrow_ata, source_escrow_mint, usdc_vault, usdc_destination] => {
                    Ok(SovereignConvertRouteAccounts {
                        token_mint: None,
                        asset_registry: None,
                        user_token_ata: None,
                        usdc_vault: usdc_vault.clone(),
                        usdc_destination: usdc_destination.clone(),
                        sol_destination: None,
                        source_primary: Some(source_primary.clone()),
                        source_secondary: Some(source_secondary.clone()),
                        source_escrow_authority: Some(source_escrow_authority.clone()),
                        source_escrow_ata: Some(source_escrow_ata.clone()),
                        source_escrow_mint: Some(source_escrow_mint.clone()),
                        alt_vault: None,
                        alt_destination: None,
                    })
                }
                [source_primary, source_secondary, source_escrow_authority, source_escrow_ata, source_escrow_mint, usdc_vault, usdc_destination, sol_destination] => {
                    Ok(SovereignConvertRouteAccounts {
                        token_mint: None,
                        asset_registry: None,
                        user_token_ata: None,
                        usdc_vault: usdc_vault.clone(),
                        usdc_destination: usdc_destination.clone(),
                        sol_destination: Some(sol_destination.clone()),
                        source_primary: Some(source_primary.clone()),
                        source_secondary: Some(source_secondary.clone()),
                        source_escrow_authority: Some(source_escrow_authority.clone()),
                        source_escrow_ata: Some(source_escrow_ata.clone()),
                        source_escrow_mint: Some(source_escrow_mint.clone()),
                        alt_vault: None,
                        alt_destination: None,
                    })
                }
                [source_primary, source_secondary, source_escrow_authority, source_escrow_ata, source_escrow_mint, usdc_vault, usdc_destination, alt_vault, alt_destination] => {
                    Ok(SovereignConvertRouteAccounts {
                        token_mint: None,
                        asset_registry: None,
                        user_token_ata: None,
                        usdc_vault: usdc_vault.clone(),
                        usdc_destination: usdc_destination.clone(),
                        sol_destination: None,
                        source_primary: Some(source_primary.clone()),
                        source_secondary: Some(source_secondary.clone()),
                        source_escrow_authority: Some(source_escrow_authority.clone()),
                        source_escrow_ata: Some(source_escrow_ata.clone()),
                        source_escrow_mint: Some(source_escrow_mint.clone()),
                        alt_vault: Some(alt_vault.clone()),
                        alt_destination: Some(alt_destination.clone()),
                    })
                }
                [source_primary, source_secondary, source_escrow_authority, source_escrow_ata, source_escrow_mint, usdc_vault, usdc_destination, alt_vault, alt_destination, sol_destination] => {
                    Ok(SovereignConvertRouteAccounts {
                        token_mint: None,
                        asset_registry: None,
                        user_token_ata: None,
                        usdc_vault: usdc_vault.clone(),
                        usdc_destination: usdc_destination.clone(),
                        sol_destination: Some(sol_destination.clone()),
                        source_primary: Some(source_primary.clone()),
                        source_secondary: Some(source_secondary.clone()),
                        source_escrow_authority: Some(source_escrow_authority.clone()),
                        source_escrow_ata: Some(source_escrow_ata.clone()),
                        source_escrow_mint: Some(source_escrow_mint.clone()),
                        alt_vault: Some(alt_vault.clone()),
                        alt_destination: Some(alt_destination.clone()),
                    })
                }
                _ => err!(SterlingError::InvalidSovereignAccountsLayout),
            }
        }
        6 => match remaining_accounts {
            [source_primary, source_secondary, usdc_vault, usdc_destination] => {
                Ok(SovereignConvertRouteAccounts {
                    token_mint: None,
                    asset_registry: None,
                    user_token_ata: None,
                    usdc_vault: usdc_vault.clone(),
                    usdc_destination: usdc_destination.clone(),
                    sol_destination: None,
                    source_primary: Some(source_primary.clone()),
                    source_secondary: Some(source_secondary.clone()),
                    source_escrow_authority: None,
                    source_escrow_ata: None,
                    source_escrow_mint: None,
                    alt_vault: None,
                    alt_destination: None,
                })
            }
            [source_primary, source_secondary, usdc_vault, usdc_destination, sol_destination] => {
                Ok(SovereignConvertRouteAccounts {
                    token_mint: None,
                    asset_registry: None,
                    user_token_ata: None,
                    usdc_vault: usdc_vault.clone(),
                    usdc_destination: usdc_destination.clone(),
                    sol_destination: Some(sol_destination.clone()),
                    source_primary: Some(source_primary.clone()),
                    source_secondary: Some(source_secondary.clone()),
                    source_escrow_authority: None,
                    source_escrow_ata: None,
                    source_escrow_mint: None,
                    alt_vault: None,
                    alt_destination: None,
                })
            }
            [source_primary, source_secondary, usdc_vault, usdc_destination, alt_vault, alt_destination] => {
                Ok(SovereignConvertRouteAccounts {
                    token_mint: None,
                    asset_registry: None,
                    user_token_ata: None,
                    usdc_vault: usdc_vault.clone(),
                    usdc_destination: usdc_destination.clone(),
                    sol_destination: None,
                    source_primary: Some(source_primary.clone()),
                    source_secondary: Some(source_secondary.clone()),
                    source_escrow_authority: None,
                    source_escrow_ata: None,
                    source_escrow_mint: None,
                    alt_vault: Some(alt_vault.clone()),
                    alt_destination: Some(alt_destination.clone()),
                })
            }
            [source_primary, source_secondary, usdc_vault, usdc_destination, alt_vault, alt_destination, sol_destination] => {
                Ok(SovereignConvertRouteAccounts {
                    token_mint: None,
                    asset_registry: None,
                    user_token_ata: None,
                    usdc_vault: usdc_vault.clone(),
                    usdc_destination: usdc_destination.clone(),
                    sol_destination: Some(sol_destination.clone()),
                    source_primary: Some(source_primary.clone()),
                    source_secondary: Some(source_secondary.clone()),
                    source_escrow_authority: None,
                    source_escrow_ata: None,
                    source_escrow_mint: None,
                    alt_vault: Some(alt_vault.clone()),
                    alt_destination: Some(alt_destination.clone()),
                })
            }
            _ => err!(SterlingError::InvalidSovereignAccountsLayout),
        },
        _ => err!(SterlingError::InvalidAccount),
    }
}

fn parse_sovereign_live_swap_accounts<'info>(
    source_kind: u8,
    remaining_accounts: &[AccountInfo<'info>],
) -> Result<SovereignLiveSwapAccounts<'info>> {
    match source_kind {
        1 | 3 => match remaining_accounts {
            [token_mint, asset_registry, user_token_ata, pool_info, input_vault, output_vault, fee_vault_in, output_destination] => {
                Ok(SovereignLiveSwapAccounts {
                    token_mint: token_mint.clone(),
                    asset_registry: Some(asset_registry.clone()),
                    user_token_ata: user_token_ata.clone(),
                    pool_info: pool_info.clone(),
                    input_vault: input_vault.clone(),
                    output_vault: output_vault.clone(),
                    fee_vault_in: fee_vault_in.clone(),
                    output_destination: output_destination.clone(),
                    native_destination: None,
                })
            }
            [token_mint, asset_registry, user_token_ata, pool_info, input_vault, output_vault, fee_vault_in, output_destination, native_destination] => {
                Ok(SovereignLiveSwapAccounts {
                    token_mint: token_mint.clone(),
                    asset_registry: Some(asset_registry.clone()),
                    user_token_ata: user_token_ata.clone(),
                    pool_info: pool_info.clone(),
                    input_vault: input_vault.clone(),
                    output_vault: output_vault.clone(),
                    fee_vault_in: fee_vault_in.clone(),
                    output_destination: output_destination.clone(),
                    native_destination: Some(native_destination.clone()),
                })
            }
            _ => err!(SterlingError::LiveSovereignExchangeRequired),
        },
        2 => match remaining_accounts {
            [token_mint, user_token_ata, pool_info, input_vault, output_vault, fee_vault_in, output_destination] => {
                Ok(SovereignLiveSwapAccounts {
                    token_mint: token_mint.clone(),
                    asset_registry: None,
                    user_token_ata: user_token_ata.clone(),
                    pool_info: pool_info.clone(),
                    input_vault: input_vault.clone(),
                    output_vault: output_vault.clone(),
                    fee_vault_in: fee_vault_in.clone(),
                    output_destination: output_destination.clone(),
                    native_destination: None,
                })
            }
            [token_mint, user_token_ata, pool_info, input_vault, output_vault, fee_vault_in, output_destination, native_destination] => {
                Ok(SovereignLiveSwapAccounts {
                    token_mint: token_mint.clone(),
                    asset_registry: None,
                    user_token_ata: user_token_ata.clone(),
                    pool_info: pool_info.clone(),
                    input_vault: input_vault.clone(),
                    output_vault: output_vault.clone(),
                    fee_vault_in: fee_vault_in.clone(),
                    output_destination: output_destination.clone(),
                    native_destination: Some(native_destination.clone()),
                })
            }
            _ => err!(SterlingError::LiveSovereignExchangeRequired),
        },
        _ => err!(SterlingError::InvalidAccount),
    }
}

fn parse_sovereign_escrow_live_swap_accounts<'info>(
    source_kind: u8,
    remaining_accounts: &[AccountInfo<'info>],
) -> Result<SovereignEscrowLiveSwapAccounts<'info>> {
    match source_kind {
        4 | 5 => match remaining_accounts {
            [source_record, source_escrow_authority, source_escrow_ata, swap_pool_info, input_vault, output_vault, fee_vault_in, output_destination] => {
                Ok(SovereignEscrowLiveSwapAccounts {
                    source_record: source_record.clone(),
                    source_escrow_authority: source_escrow_authority.clone(),
                    source_escrow_ata: source_escrow_ata.clone(),
                    swap_pool_info: swap_pool_info.clone(),
                    input_vault: input_vault.clone(),
                    output_vault: output_vault.clone(),
                    fee_vault_in: fee_vault_in.clone(),
                    output_destination: output_destination.clone(),
                    native_destination: None,
                    ledger_info: None,
                })
            }
            [source_record, source_escrow_authority, source_escrow_ata, swap_pool_info, input_vault, output_vault, fee_vault_in, output_destination, native_destination] => {
                Ok(SovereignEscrowLiveSwapAccounts {
                    source_record: source_record.clone(),
                    source_escrow_authority: source_escrow_authority.clone(),
                    source_escrow_ata: source_escrow_ata.clone(),
                    swap_pool_info: swap_pool_info.clone(),
                    input_vault: input_vault.clone(),
                    output_vault: output_vault.clone(),
                    fee_vault_in: fee_vault_in.clone(),
                    output_destination: output_destination.clone(),
                    native_destination: Some(native_destination.clone()),
                    ledger_info: None,
                })
            }
            _ => err!(SterlingError::LiveSovereignExchangeRequired),
        },
        6 => match remaining_accounts {
            [pool_info, ledger_info, source_escrow_authority, source_escrow_ata, swap_pool_info, input_vault, output_vault, fee_vault_in, output_destination] => {
                Ok(SovereignEscrowLiveSwapAccounts {
                    source_record: pool_info.clone(),
                    source_escrow_authority: source_escrow_authority.clone(),
                    source_escrow_ata: source_escrow_ata.clone(),
                    swap_pool_info: swap_pool_info.clone(),
                    input_vault: input_vault.clone(),
                    output_vault: output_vault.clone(),
                    fee_vault_in: fee_vault_in.clone(),
                    output_destination: output_destination.clone(),
                    native_destination: None,
                    ledger_info: Some(ledger_info.clone()),
                })
            }
            [pool_info, ledger_info, source_escrow_authority, source_escrow_ata, swap_pool_info, input_vault, output_vault, fee_vault_in, output_destination, native_destination] => {
                Ok(SovereignEscrowLiveSwapAccounts {
                    source_record: pool_info.clone(),
                    source_escrow_authority: source_escrow_authority.clone(),
                    source_escrow_ata: source_escrow_ata.clone(),
                    swap_pool_info: swap_pool_info.clone(),
                    input_vault: input_vault.clone(),
                    output_vault: output_vault.clone(),
                    fee_vault_in: fee_vault_in.clone(),
                    output_destination: output_destination.clone(),
                    native_destination: Some(native_destination.clone()),
                    ledger_info: Some(ledger_info.clone()),
                })
            }
            _ => err!(SterlingError::LiveSovereignExchangeRequired),
        },
        _ => err!(SterlingError::InvalidAccount),
    }
}

fn is_native_sol_rail_enabled(cfg: &Config) -> bool {
    cfg.native_sol_enabled && cfg.native_sol_usd_micros_per_sol > 0
}

fn native_sol_marker_pubkey() -> Pubkey {
    anchor_lang::system_program::ID
}

fn build_usdc_output_target<'info>(
    config: &Account<'info, Config>,
    usdc_coffre_info: AccountInfo<'info>,
    destination_usdc_info: AccountInfo<'info>,
) -> Result<StableFinalOutputTarget<'info>> {
    let usdc_coffre_ata = load_token_account_snapshot(&usdc_coffre_info)?;
    let destination_usdc_ata = load_token_account_snapshot(&destination_usdc_info)?;

    require!(
        usdc_coffre_info.key() == config.usdc_coffre,
        SterlingError::StableSettlementVaultMismatch
    );
    require!(
        usdc_coffre_ata.mint == config.usdc_mint,
        SterlingError::StableSettlementVaultMismatch
    );
    require!(
        destination_usdc_ata.mint == config.usdc_mint,
        SterlingError::StableSettlementDestinationMismatch
    );

    Ok(StableFinalOutputTarget {
        final_output_kind: SovereignFinalOutputKind::Usdc,
        payout_mint: config.usdc_mint,
        vault_info: Some(usdc_coffre_info),
        vault_amount: usdc_coffre_ata.amount,
        destination_info: destination_usdc_info.clone(),
        destination_key: destination_usdc_info.key(),
    })
}

fn configured_alt_payout_rail_from_vault(
    config: &Config,
    vault_key: Pubkey,
) -> Option<(SovereignFinalOutputKind, Pubkey, Pubkey)> {
    if vault_key == config.usdt_coffre && config.usdt_mint != Pubkey::default() {
        return Some((
            SovereignFinalOutputKind::Usdt,
            config.usdt_mint,
            config.usdt_coffre,
        ));
    }
    if vault_key == config.extra_payout_vault_ata_0
        && config.extra_payout_mint_0 != Pubkey::default()
    {
        return Some((
            SovereignFinalOutputKind::Extra0,
            config.extra_payout_mint_0,
            config.extra_payout_vault_ata_0,
        ));
    }
    if vault_key == config.extra_payout_vault_ata_1
        && config.extra_payout_mint_1 != Pubkey::default()
    {
        return Some((
            SovereignFinalOutputKind::Extra1,
            config.extra_payout_mint_1,
            config.extra_payout_vault_ata_1,
        ));
    }
    if vault_key == config.extra_payout_vault_ata_2
        && config.extra_payout_mint_2 != Pubkey::default()
    {
        return Some((
            SovereignFinalOutputKind::Extra2,
            config.extra_payout_mint_2,
            config.extra_payout_vault_ata_2,
        ));
    }
    if vault_key == config.extra_payout_vault_ata_3
        && config.extra_payout_mint_3 != Pubkey::default()
    {
        return Some((
            SovereignFinalOutputKind::Extra3,
            config.extra_payout_mint_3,
            config.extra_payout_vault_ata_3,
        ));
    }
    None
}

fn build_optional_alt_output_target<'info>(
    config: &Account<'info, Config>,
    remaining: &SovereignRemainingAccounts<'info>,
) -> Result<Option<StableFinalOutputTarget<'info>>> {
    match (&remaining.alt_vault, &remaining.alt_destination) {
        (None, None) => Ok(None),
        (Some(alt_vault), Some(alt_destination)) => {
            let vault_info = alt_vault.clone();
            let destination_info = alt_destination.clone();
            let vault = load_token_account_snapshot(&vault_info)?;
            let destination = load_token_account_snapshot(&destination_info)?;
            let (final_output_kind, payout_mint, expected_vault_key) =
                configured_alt_payout_rail_from_vault(config, alt_vault.key())
                    .ok_or_else(|| error!(SterlingError::StableSettlementVaultMismatch))?;

            require!(
                alt_vault.key() == expected_vault_key,
                SterlingError::StableSettlementVaultMismatch
            );
            require!(
                vault.mint == payout_mint,
                SterlingError::StableSettlementVaultMismatch
            );
            require!(
                destination.mint == payout_mint,
                SterlingError::StableSettlementDestinationMismatch
            );

            Ok(Some(StableFinalOutputTarget {
                final_output_kind,
                payout_mint,
                vault_info: Some(alt_vault.clone()),
                vault_amount: vault.amount,
                destination_info: alt_destination.clone(),
                destination_key: alt_destination.key(),
            }))
        }
        _ => err!(SterlingError::StableSettlementAccountsMissing),
    }
}

fn build_explicit_extra_fee_rail_targets<'info>(
    config: &Config,
    remaining_accounts: &'info [AccountInfo<'info>],
) -> Result<Vec<StableFinalOutputTarget<'info>>> {
    if remaining_accounts.is_empty() {
        return Ok(Vec::new());
    }
    require!(
        remaining_accounts.len() % 2 == 0,
        SterlingError::StableSettlementAccountsMissing
    );

    let mut targets = Vec::with_capacity(remaining_accounts.len() / 2);
    for pair in remaining_accounts.chunks_exact(2) {
        let vault_info = pair[0].clone();
        let destination_info = pair[1].clone();
        let vault = load_token_account_snapshot(&vault_info)?;
        let destination = load_token_account_snapshot(&destination_info)?;
        let (final_output_kind, payout_mint, expected_vault_key) =
            configured_alt_payout_rail_from_vault(config, vault_info.key())
                .ok_or_else(|| error!(SterlingError::StableSettlementVaultMismatch))?;

        require!(
            matches!(
                final_output_kind,
                SovereignFinalOutputKind::Extra0
                    | SovereignFinalOutputKind::Extra1
                    | SovereignFinalOutputKind::Extra2
                    | SovereignFinalOutputKind::Extra3
            ),
            SterlingError::StableSettlementVaultMismatch
        );
        require!(
            vault_info.key() == expected_vault_key,
            SterlingError::StableSettlementVaultMismatch
        );
        require!(
            vault.mint == payout_mint,
            SterlingError::StableSettlementVaultMismatch
        );
        require!(
            destination.mint == payout_mint,
            SterlingError::StableSettlementDestinationMismatch
        );
        require!(
            destination.owner == config.main_wallet,
            SterlingError::InvalidAccount
        );

        targets.push(StableFinalOutputTarget {
            final_output_kind,
            payout_mint,
            vault_info: Some(vault_info),
            vault_amount: vault.amount,
            destination_info: destination_info.clone(),
            destination_key: destination_info.key(),
        });
    }

    Ok(targets)
}

fn build_optional_native_sol_output_target<'info>(
    config: &Account<'info, Config>,
    destination_info: Option<AccountInfo<'info>>,
) -> Result<Option<NativeSolOutputTarget<'info>>> {
    if !is_native_sol_rail_enabled(config) {
        return Ok(None);
    }

    let destination_info = match destination_info {
        Some(destination_info) => destination_info,
        None => return Ok(None),
    };

    require!(
        destination_info.is_writable,
        SterlingError::InvalidNativeSolDestination
    );
    require!(
        !destination_info.executable,
        SterlingError::InvalidNativeSolDestination
    );
    require!(
        destination_info.key() != config.key(),
        SterlingError::InvalidNativeSolDestination
    );

    let config_info = config.to_account_info();
    let rent_floor = Rent::get()?.minimum_balance(config_info.data_len());
    let protected_lamports = rent_floor.saturating_add(config.native_sol_min_reserve_lamports);
    let available_lamports = config_info.lamports().saturating_sub(protected_lamports);

    Ok(Some(NativeSolOutputTarget {
        destination_key: destination_info.key(),
        destination_info,
        available_lamports,
    }))
}

fn select_stable_output_targets<'info>(
    requested: SovereignFinalOutputKind,
    usdc_target: StableFinalOutputTarget<'info>,
    alt_target: Option<StableFinalOutputTarget<'info>>,
) -> (
    Option<StableFinalOutputTarget<'info>>,
    Option<StableFinalOutputTarget<'info>>,
) {
    match requested {
        SovereignFinalOutputKind::Auto => (Some(usdc_target), alt_target),
        SovereignFinalOutputKind::Usdc => (Some(usdc_target), None),
        SovereignFinalOutputKind::Usdt
        | SovereignFinalOutputKind::Extra0
        | SovereignFinalOutputKind::Extra1
        | SovereignFinalOutputKind::Extra2
        | SovereignFinalOutputKind::Extra3 => (
            alt_target.filter(|target| target.final_output_kind == requested),
            None,
        ),
        SovereignFinalOutputKind::Sol => (None, None),
    }
}

fn usd_micros_to_native_sol_lamports(
    usd_micros: u64,
    native_sol_usd_micros_per_sol: u64,
) -> Result<u64> {
    require!(
        native_sol_usd_micros_per_sol > 0,
        SterlingError::NativeSolRailNotEnabled
    );

    let lamports = (usd_micros as u128)
        .saturating_mul(LAMPORTS_PER_SOL_U64 as u128)
        .checked_div(native_sol_usd_micros_per_sol as u128)
        .ok_or(SterlingError::MathOverflow)?;

    Ok(u64::try_from(lamports).map_err(|_| SterlingError::MathOverflow)?)
}

fn native_sol_payout_lamports<'info>(
    config: NativeSolRailConfig,
    target: &NativeSolOutputTarget<'info>,
    usd_micros: u64,
) -> Result<u64> {
    let payout_lamports =
        usd_micros_to_native_sol_lamports(usd_micros, config.native_sol_usd_micros_per_sol)?;
    require!(payout_lamports > 0, SterlingError::ZeroNativeSolSettlement);
    require!(
        target.available_lamports >= payout_lamports,
        SterlingError::InsufficientNativeSolSettlementLiquidity
    );
    Ok(payout_lamports)
}

fn transfer_native_sol_from_config<'info>(
    config_info: AccountInfo<'info>,
    target: &NativeSolOutputTarget<'info>,
    lamports: u64,
) -> Result<()> {
    require!(lamports > 0, SterlingError::ZeroNativeSolSettlement);
    require!(
        target.destination_info.is_writable,
        SterlingError::InvalidNativeSolDestination
    );

    {
        let mut config_lamports = config_info.try_borrow_mut_lamports()?;
        require!(
            **config_lamports >= lamports,
            SterlingError::InsufficientNativeSolSettlementLiquidity
        );
        **config_lamports = config_lamports.saturating_sub(lamports);
    }

    {
        let mut destination_lamports = target.destination_info.try_borrow_mut_lamports()?;
        **destination_lamports = destination_lamports.saturating_add(lamports);
    }

    Ok(())
}

fn stable_target_vault_info<'info>(
    target: &StableFinalOutputTarget<'info>,
) -> Result<AccountInfo<'info>> {
    target
        .vault_info
        .clone()
        .ok_or_else(|| error!(SterlingError::StableSettlementAccountsMissing))
}

fn transfer_from_stable_target_signed<'info>(
    target: &StableFinalOutputTarget<'info>,
    config_info: AccountInfo<'info>,
    token_program_info: AccountInfo<'info>,
    config_bump: u8,
    amount: u64,
) -> Result<()> {
    require!(amount > 0, SterlingError::ZeroStableSettlement);
    require!(
        target.vault_amount >= amount,
        SterlingError::InsufficientStableSettlementLiquidity
    );

    token::transfer(
        CpiContext::new(
            token_program_info,
            Transfer {
                from: stable_target_vault_info(target)?,
                to: target.destination_info.clone(),
                authority: config_info,
            },
        )
        .with_signer(&[&[b"config", &[config_bump]]]),
        amount,
    )
}

fn emit_sovereign_output_event(
    source_kind: u8,
    source_mint: Pubkey,
    amount_in: u64,
    payout_mint: Pubkey,
    payout_amount: u64,
    destination_ata: Pubkey,
    user: Pubkey,
) -> Result<()> {
    emit!(SovereignOutputExecuted {
        source_kind,
        source_mint,
        amount_in,
        payout_mint,
        payout_amount,
        destination_ata,
        user,
        ts: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

fn sync_pool_registry_runtime_field(
    pool_registry_entry: &mut Account<PoolRegistryEntry>,
    pool_key: Pubkey,
    field_role: u8,
    account_key: Pubkey,
) -> Result<()> {
    require!(
        pool_registry_entry.pool == pool_key,
        SterlingError::InvalidAccount
    );

    match field_role {
        1 => pool_registry_entry.base_vault = account_key,
        2 => pool_registry_entry.quote_vault = account_key,
        3 => pool_registry_entry.lp_mint = account_key,
        4 => pool_registry_entry.fee_vault_base = account_key,
        5 => pool_registry_entry.fee_vault_quote = account_key,
        _ => return err!(SterlingError::InvalidAccount),
    }

    Ok(())
}

fn load_token_account_snapshot(account_info: &AccountInfo) -> Result<TokenAccount> {
    let data = account_info.try_borrow_data()?;
    let mut bytes: &[u8] = &data;
    TokenAccount::try_deserialize(&mut bytes)
}

fn load_config_snapshot(account_info: &AccountInfo) -> Result<Config> {
    let data = account_info.try_borrow_data()?;
    let mut bytes: &[u8] = &data;
    Config::try_deserialize(&mut bytes)
}

#[inline(never)]
fn load_native_sol_rail_config(account_info: &AccountInfo) -> Result<NativeSolRailConfig> {
    let config = load_config_snapshot(account_info)?;
    Ok(NativeSolRailConfig {
        native_sol_usd_micros_per_sol: config.native_sol_usd_micros_per_sol,
    })
}

fn load_mint_snapshot(account_info: &AccountInfo) -> Result<Mint> {
    let data = account_info.try_borrow_data()?;
    let mut bytes: &[u8] = &data;
    Mint::try_deserialize(&mut bytes)
}

fn load_pool_snapshot(account_info: &AccountInfo) -> Result<Pool> {
    let data = account_info.try_borrow_data()?;
    let mut bytes: &[u8] = &data;
    Pool::try_deserialize(&mut bytes)
}

fn load_asset_registry_snapshot(account_info: &AccountInfo) -> Result<AssetRegistry> {
    let data = account_info.try_borrow_data()?;
    let mut bytes: &[u8] = &data;
    AssetRegistry::try_deserialize(&mut bytes)
}

// - un snapshot de coffre n'est pas une preuve live de funding executable.
pub const USDC_COFFRE: &str = "7vWLrATXnuGTCjmexa7b4roo9Em6VMKr3bdDemJNHNk1";
pub const USDT_COFFRE: &str = "7qeqQYVgLaeaDWi4X4Hin9wpauCxZFQB5Bov9zKFev2W";

// Divers
pub const PDA_GT: &str = "GTAs9L3XFdhHEFoo6KWNbFFxMCFRnbVomsbx7deShkLb";
pub const COFFRE_7Q: &str = "7qeqQYVgLaeaDWi4X4Hin9wpauCxZFQB5Bov9zKFev2W";
pub const POOL_ID: &str = "BbvR4zUAwZF8LmVFLXNpDy3CxuYcDwd5isoh7CZFAF5G";

// Flags
pub const TRUE_CASH_FLAG: bool = true;
pub const CASH_BACKED_FLAG: bool = true;
pub const REAL_PEG_FLAG: bool = true;
pub const SOVEREIGN_FLAG: bool = true;

// Values
//
// Modele metier:
// - la valeur des claims / tickets est exprimee d'abord en USD micros;
// - l'USDC est l'asset de sortie actuellement demande pour le settlement;
// - les fees de swap doivent etre reglees en USDC dans le contrat lui-meme;
// - pour les swaps internes, le reglement vise directement l'ATA USDC de tresorerie;
// - si la fee est en un token Sterling valorise on-chain, le contrat peut burn ce token
//   puis liberer la contre-valeur en USDC depuis le coffre USDC vers l'ATA de tresorerie;
// - l'objectif metier est d'eviter tout passage manuel par ticket ou conversion exterieure
//   pour toucher l'USDC issu des fees de swap.
// - le cap ticket est un format de decoupage, pas une preuve de reserve stable.
pub const USD_MICROS: u64 = 1_000_000;
pub const DEFAULT_TOKEN_VALUE_USD_MICROS: u64 = 174_000 * USD_MICROS;
pub const DEFAULT_TREASURY_VALUE_USD_MICROS: u64 = 174_000 * 1_000_000;
pub const DEFAULT_TICKET_CAP_USD_MICROS: u64 = 250_000 * USD_MICROS;
pub const FLOOR_PRICE_ENABLED: bool = true;
pub const FLOOR_PRICE_USD_MICROS: u64 = 92_500 * USD_MICROS;
pub const FLOOR_PROTECTED_BASE_VAULT: &str = "3mRYBWgBKnQuUyvVDcYFqSeNoQTujTsFGra3GWLof9av";
pub const FLOOR_PROTECTED_QUOTE_VAULT: &str = "5z4brtXmcDBhPKLk9YoiZE7fqaourBk26jBuAUHqZDN9";
pub const MIN_HTOP_STM_GUARANTEE_ATOMS: u64 = 17_915_708_621_462_006_032;
pub const MIN_HTOP_SJBC_GUARANTEE_ATOMS: u64 = 16_950_244_075_873_423_613;

// Staking
pub const DEFAULT_CASHBACK_BPS: u16 = 9200; // 92%
pub const DEFAULT_REWARD_INTERVAL_SECONDS: u64 = 300; // 5 min

// Swap fees
pub const DEFAULT_SWAP_FEE_BPS: u16 = 30; // 0.30%
pub const MAX_SWAP_FEE_BPS: u16 = 1000; // 10%

// Auto threshold (USD micros)
pub const DEFAULT_FEE_THRESHOLD_USD_MICROS: u64 = 10 * USD_MICROS;
pub const PROTOCOL_DEBT_LEDGER_SLOTS: usize = 16;
pub const STABLE_CASH_DECIMALS: u8 = 6;
pub const LAMPORTS_PER_SOL_U64: u64 = 1_000_000_000;
pub const DEFAULT_NATIVE_SOL_USD_MICROS_PER_SOL: u64 = 0;
pub const DEFAULT_NATIVE_SOL_MIN_RESERVE_LAMPORTS: u64 = 0;
pub const SOVEREIGN_ESCROW_STATE_REQUESTED: u8 = 0;
pub const SOVEREIGN_ESCROW_STATE_FUNDED_IN_ESCROW: u8 = 1;
pub const SOVEREIGN_ESCROW_STATE_EXECUTED_LIVE: u8 = 2;
pub const SOVEREIGN_ESCROW_STATE_SETTLED: u8 = 3;
pub const SOVEREIGN_ESCROW_STATE_CANCELLED: u8 = 4;
pub const PAYOUT_TICKET_STATUS_REQUESTED: u8 = 0;
pub const PAYOUT_TICKET_STATUS_SETTLED: u8 = 1;
pub const SETTLEMENT_CLAIM_STATUS_OPEN: u8 = 0;
pub const SETTLEMENT_CLAIM_STATUS_PAID: u8 = 1;

// Main wallets (Solana pubkeys)
pub const MAIN_WALLET: &str = "CMqD45Kq5oukPvaMDhzav5RxJqZb1xME1MmV71CzCeTw"; // Compte C
pub const OKX_WALLET: &str = "GQo15C7g4rbJ7zzhAzSd3SzeMAYpPtNhD8U8P3QrL7MU";
pub const EVAN_KEEPER_WALLET: &str = "4vUFufp4Smj71HafZcxwmMWdN8LLmwp1LDdemT1EY8Ei";

// Treasury ATAs (wallet C)
pub const TREASURY_USDC_ATA: &str = "2NUyY9XfzZ6dHZwRtQMt5oBHhZLNdwTBKwVbjrPwEDGN";
pub const TREASURY_USDT_ATA: &str = "GUfkfnKNB1rNkjJWaJ7KnciFsVPEMV3vamA5SYqeeoDD";

// =========================
// TYPES
// =========================
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum FeeSide {
    Base,
    Quote,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SettlementProofKind {
    CertifiedClaimSnapshot = 1,
    LiveFundingExecution = 2,
    ProviderPayoutReceipt = 3,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SettlementFundingStage {
    EconomicValueUsd = 1,
    TechnicalFundingRail = 2,
    ProviderPayout = 3,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SovereignFinalOutputKind {
    Auto = 0,
    Usdc = 1,
    Usdt = 2,
    Sol = 3,
    Extra0 = 4,
    Extra1 = 5,
    Extra2 = 6,
    Extra3 = 7,
}

// =========================
// PROGRAM
// =========================
#[program]
pub mod sterling_amm {
    use super::*;

    // =========================
    // A) CONFIG
    // =========================
    pub fn initialize_config(ctx: Context<InitializeConfig>) -> Result<()> {
        let cfg = &mut ctx.accounts.config;
        let admin_key = ctx.accounts.admin.key();

        cfg.admin = admin_key;

        cfg.true_cash = TRUE_CASH_FLAG;
        cfg.cash_backed = CASH_BACKED_FLAG;
        cfg.real_peg = REAL_PEG_FLAG;
        cfg.sovereign = SOVEREIGN_FLAG;

        cfg.token_value_usd_micros_default = DEFAULT_TOKEN_VALUE_USD_MICROS;
        cfg.treasury_value_usd_micros = DEFAULT_TREASURY_VALUE_USD_MICROS;

        cfg.cashback_bps = DEFAULT_CASHBACK_BPS;
        cfg.reward_interval = DEFAULT_REWARD_INTERVAL_SECONDS;
        cfg.allow_fallback_usdt = true;
        cfg.native_sol_enabled = false;
        cfg.native_sol_usd_micros_per_sol = DEFAULT_NATIVE_SOL_USD_MICROS_PER_SOL;
        cfg.native_sol_min_reserve_lamports = DEFAULT_NATIVE_SOL_MIN_RESERVE_LAMPORTS;

        cfg.enable_treasury = true;
        cfg.enable_sjbc = true;
        cfg.enable_sjbc2 = true;
        cfg.enable_sjbc3 = true;

        // ✅ ajout : USDC géré explicitement
        cfg.enable_usdc = true;

        cfg.enable_usdt_main = true;
        cfg.enable_usdt_old = false;
        cfg.enable_btc_portal = true;

        cfg.main_wallet = pubkey_from_str(MAIN_WALLET);
        cfg.okx_wallet = pubkey_from_str(OKX_WALLET);

        cfg.lp_mint2 = pubkey_from_str(LP_MINT_2);
        cfg.lp_auth2 = pubkey_from_str(LP_AUTH_2);
        cfg.lp_mint3 = pubkey_from_str(LP_MINT_3);
        cfg.lp_auth3 = pubkey_from_str(LP_AUTH_3);

        cfg.mint_h1 = pubkey_from_str(MINT_H1);
        cfg.mint_fr15 = pubkey_from_str(MINT_FR15);
        cfg.mint_db8 = pubkey_from_str(MINT_DB8);

        cfg.usdc_coffre = pubkey_from_str(USDC_COFFRE);
        cfg.usdt_coffre = pubkey_from_str(USDT_COFFRE);

        cfg.pda_gt = pubkey_from_str(PDA_GT);
        cfg.coffre_7q = pubkey_from_str(COFFRE_7Q);
        cfg.pool_id = pubkey_from_str(POOL_ID);

        cfg.treasury_usdc_ata = pubkey_from_str(TREASURY_USDC_ATA);
        cfg.treasury_usdt_ata = pubkey_from_str(TREASURY_USDT_ATA);

        cfg.auto_collect_every_swaps = 10;
        cfg.fee_threshold_usd_micros = DEFAULT_FEE_THRESHOLD_USD_MICROS;

        cfg.keeper_authority = pubkey_from_str(MAIN_WALLET);
        cfg.usdc_mint = pubkey_from_str(USDC_MINT);
        cfg.usdt_mint = pubkey_from_str(USDT_MAIN_MINT);
        cfg.extra_payout_mint_0 = Pubkey::default();
        cfg.extra_payout_mint_1 = Pubkey::default();
        cfg.extra_payout_mint_2 = Pubkey::default();
        cfg.extra_payout_mint_3 = Pubkey::default();
        cfg.extra_payout_vault_ata_0 = Pubkey::default();
        cfg.extra_payout_vault_ata_1 = Pubkey::default();
        cfg.extra_payout_vault_ata_2 = Pubkey::default();
        cfg.extra_payout_vault_ata_3 = Pubkey::default();
        cfg.payout_threshold_usd_micros = 0;
        cfg.max_payout_usd_micros = 0;
        cfg.max_payout_per_window_usd_micros = 0;
        cfg.payout_window_secs = 0;
        cfg.payout_window_start = 0;
        cfg.payout_window_used_usd_micros = 0;
        cfg.lp_cashback_bps = 0;
        cfg.claim_cashback_bps = 0;

        cfg.bump = ctx.bumps.config;

        Ok(())
    }

    pub fn settle_payout_v3_safe_v2(
        ctx: Context<SettlePayoutV3Safe>,
        nonce: u64,
        payout_kind: u8,
        mint_in: Pubkey,
        amount_in: u64,
        usd_micros: u64,
    ) -> Result<()> {
        settle_payout_v3_safe_v2_routed(ctx, nonce, payout_kind, mint_in, amount_in, usd_micros, 0)
    }

    pub fn request_protocol_fee_payout_usdc_v1(
        ctx: Context<SettlePayoutV3Safe>,
        nonce: u64,
        mint_in: Pubkey,
        amount_in: u64,
        usd_micros: u64,
    ) -> Result<()> {
        request_protocol_fee_payout_usdc_v1_routed(ctx, nonce, mint_in, amount_in, usd_micros, 0)
    }

    pub fn reserve_authority_rebind(
        ctx: Context<ReserveAuthorityRebind>,
        bridge_vault_bump: u8,
    ) -> Result<()> {
        crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;

        let authority_mint_key = ctx.accounts.authority_mint.key();
        let reserve_token_account_info = ctx.accounts.reserve_token_account.to_account_info();
        let reserve_token_account = load_token_account_snapshot(&reserve_token_account_info)?;
        require!(
            reserve_token_account_info.key() != ctx.accounts.new_authority.key(),
            SterlingError::InvalidAccount
        );

        let (expected_bridge_vault, expected_bump) = Pubkey::find_program_address(
            &[b"bridge_vault", authority_mint_key.as_ref()],
            ctx.program_id,
        );
        require!(
            bridge_vault_bump == expected_bump,
            SterlingError::InvalidAccount
        );

        let current_authority_key = ctx.accounts.current_authority_bridge_vault.key();
        let use_bridge_vault = current_authority_key == expected_bridge_vault;
        let use_config = current_authority_key == ctx.accounts.config.key();
        require!(use_bridge_vault || use_config, SterlingError::InvalidAccount);
        require!(
            reserve_token_account.owner == current_authority_key,
            SterlingError::InvalidAccount
        );

        if use_bridge_vault {
            let bridge_vault_info = load_token_account_snapshot(
                &ctx.accounts.current_authority_bridge_vault.to_account_info(),
            )?;
            require!(
                bridge_vault_info.mint == authority_mint_key,
                SterlingError::InvalidAccount
            );
            require!(
                bridge_vault_info.owner == ctx.accounts.config.key(),
                SterlingError::InvalidAccount
            );
        }

        if use_bridge_vault {
            let bridge_vault_bump_arr = [bridge_vault_bump];
            let signer_group: [&[u8]; 3] = [
                b"bridge_vault",
                authority_mint_key.as_ref(),
                &bridge_vault_bump_arr,
            ];
            let signer: [&[&[u8]]; 1] = [&signer_group];
            token::set_authority(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    SetAuthority {
                        account_or_mint: reserve_token_account_info,
                        current_authority: ctx.accounts.current_authority_bridge_vault.to_account_info(),
                    },
                )
                .with_signer(&signer),
                anchor_spl::token::spl_token::instruction::AuthorityType::AccountOwner,
                Some(ctx.accounts.new_authority.key()),
            )?;
        } else {
            let config_bump_arr = [ctx.bumps.config];
            let signer_group: [&[u8]; 2] = [b"config", &config_bump_arr];
            let signer: [&[&[u8]]; 1] = [&signer_group];
            token::set_authority(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    SetAuthority {
                        account_or_mint: reserve_token_account_info,
                        current_authority: ctx.accounts.current_authority_bridge_vault.to_account_info(),
                    },
                )
                .with_signer(&signer),
                anchor_spl::token::spl_token::instruction::AuthorityType::AccountOwner,
                Some(ctx.accounts.new_authority.key()),
            )?;
        }

        Ok(())
    }

    pub fn treasury_sweep_reserve(
        ctx: Context<TreasurySweepReserveCompat>,
        amount_atoms: u64,
        usd_micros: u64,
    ) -> Result<()> {
        crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        require!(amount_atoms > 0, SterlingError::InvalidAmount);
        require!(usd_micros > 0, SterlingError::InvalidAmount);

        let reserve_token_account_info = ctx.accounts.reserve_token_account.to_account_info();
        let destination_token_account_info = ctx.accounts.destination_token_account.to_account_info();
        let reserve_token_account = load_token_account_snapshot(&reserve_token_account_info)?;
        let destination_token_account = load_token_account_snapshot(&destination_token_account_info)?;
        let config_key = ctx.accounts.config.key();
        let admin_key = ctx.accounts.admin.key();
        let is_config_owned_reserve = reserve_token_account.owner == config_key;
        let is_operator_materialization =
            reserve_token_account.owner == admin_key && destination_token_account.owner == config_key;

        require!(
            reserve_token_account.mint == destination_token_account.mint,
            SterlingError::InvalidAccount
        );
        require!(
            reserve_token_account_info.key() != destination_token_account_info.key(),
            SterlingError::InvalidAccount
        );
        require!(
            reserve_token_account.amount >= amount_atoms,
            SterlingError::InvalidAmount
        );

        if is_operator_materialization {
            require!(ctx.accounts.config.key() == pubkey_from_str(LP_AUTH_2), SterlingError::InvalidAccount);
            let (expected_source, expected_destination) =
                expected_operator_to_htop_materialization_accounts(reserve_token_account.mint)
                    .ok_or_else(|| error!(SterlingError::UnsupportedMint))?;
            require!(
                reserve_token_account_info.key() == expected_source,
                SterlingError::InvalidAccount
            );
            require!(
                destination_token_account_info.key() == expected_destination,
                SterlingError::InvalidAccount
            );

            token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: reserve_token_account_info.clone(),
                        to: destination_token_account_info.clone(),
                        authority: ctx.accounts.admin.to_account_info(),
                    },
                ),
                amount_atoms,
            )?;

            emit!(HtopReserveMaterializedEvent {
                source_mint: reserve_token_account.mint,
                source_account: reserve_token_account_info.key(),
                destination_htop_reserve: destination_token_account_info.key(),
                amount_atoms,
                usd_micros,
                ts: Clock::get()?.unix_timestamp,
            });
        } else {
            require!(is_config_owned_reserve, SterlingError::InvalidAccount);

            let config_bump_arr = [ctx.bumps.config];
            let signer_seeds: &[&[&[u8]]] = &[&[b"config", &config_bump_arr]];
            token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: reserve_token_account_info,
                        to: destination_token_account_info,
                        authority: ctx.accounts.config.to_account_info(),
                    },
                )
                .with_signer(signer_seeds),
                amount_atoms,
            )?;
        }

        Ok(())
    }

    pub fn sovereign_redeem_to_usdc(
        ctx: Context<SovereignRedeemToUsdc>,
        amount_in: u64,
        bank_metadata: String,
    ) -> Result<()> {
        let _ = bank_metadata;
        let asset_registry_info = ctx.accounts.asset_registry.to_account_info();
        let asset_registry = load_asset_registry_snapshot(&asset_registry_info)?;
        let token_mint_info = ctx.accounts.token_mint.to_account_info();
        let token_mint = load_mint_snapshot(&token_mint_info)?;
        let user_token_ata_info = ctx.accounts.user_token_ata.to_account_info();
        let user_token_ata = load_token_account_snapshot(&user_token_ata_info)?;
        let usdc_coffre_info = ctx.accounts.usdc_coffre_ata.to_account_info();
        let usdc_coffre_ata = load_token_account_snapshot(&usdc_coffre_info)?;
        let destination_usdc_info = ctx.accounts.destination_usdc_ata.to_account_info();
        let destination_usdc_ata = load_token_account_snapshot(&destination_usdc_info)?;

        require!(amount_in > 0, SterlingError::InvalidAmount);
        require!(
            asset_registry.mint == token_mint_info.key(),
            SterlingError::InvalidAccount
        );
        require!(asset_registry.active, SterlingError::BadRegistry);
        require!(!asset_registry.is_lp, SterlingError::InvalidAccount);
        require!(
            asset_registry.decimals == token_mint.decimals,
            SterlingError::InvalidAccount
        );
        require!(
            user_token_ata.owner == ctx.accounts.user.key(),
            SterlingError::InvalidAccount
        );
        require!(
            user_token_ata.mint == token_mint_info.key(),
            SterlingError::InvalidAccount
        );
        require!(
            usdc_coffre_info.key() == ctx.accounts.config.usdc_coffre,
            SterlingError::InvalidAccount
        );
        require!(
            usdc_coffre_ata.mint == ctx.accounts.config.usdc_mint,
            SterlingError::InvalidAccount
        );
        require!(
            destination_usdc_ata.owner == ctx.accounts.user.key(),
            SterlingError::InvalidAccount
        );
        require!(
            destination_usdc_ata.mint == ctx.accounts.config.usdc_mint,
            SterlingError::InvalidAccount
        );

        let usdc_out = compute_usdc_settlement_amount(
            amount_in,
            asset_registry.valuation_usd_micros,
            token_mint.decimals,
        )?;
        require!(usdc_out > 0, SterlingError::InvalidAmount);
        require!(
            usdc_coffre_ata.amount >= usdc_out,
            SterlingError::InsufficientUsdcSettlementLiquidity
        );

        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: token_mint_info.clone(),
                    from: user_token_ata_info,
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_in,
        )?;

        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[ctx.bumps.config]]];
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: usdc_coffre_info,
                    to: destination_usdc_info,
                    authority: ctx.accounts.config.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            usdc_out,
        )?;

        emit!(SovereignOutputExecuted {
            source_kind: 1,
            source_mint: token_mint_info.key(),
            amount_in,
            payout_mint: ctx.accounts.config.usdc_mint,
            payout_amount: usdc_out,
            destination_ata: ctx.accounts.destination_usdc_ata.key(),
            user: ctx.accounts.user.key(),
            ts: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn sovereign_convert_output_v1<'info>(
        ctx: Context<'_, '_, 'info, 'info, SovereignConvertOutputV1Ctx<'info>>,
        amount_in: u64,
        source_kind: u8,
        final_output_kind: u8,
        debt_lot_nonce: u64,
        bank_metadata: String,
    ) -> Result<()> {
        sovereign_convert_output_v1_impl(
            ctx,
            amount_in,
            source_kind,
            final_output_kind,
            debt_lot_nonce,
            bank_metadata,
        )
    }

    pub fn request_protocol_fee_payout_usdc_v1_routed(
        ctx: Context<SettlePayoutV3Safe>,
        nonce: u64,
        mint_in: Pubkey,
        amount_in: u64,
        usd_micros: u64,
        route_hint: u8,
    ) -> Result<()> {
        require!(usd_micros > 0, SterlingError::InvalidAmount);
        enforce_ticket_cap_usd_micros(usd_micros)?;

        let beneficiary = ctx.accounts.user.key();
        let expected_beneficiary = pubkey_from_str(MAIN_WALLET);
        let expected_payout_mint = pubkey_from_str(USDC_MINT);

        require!(
            beneficiary == expected_beneficiary,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.payout_mint.key() == expected_payout_mint,
            SterlingError::InvalidAccount
        );

        require!(
            route_hint == 0 || route_hint == 1,
            SterlingError::InvalidAccount
        );

        // route_hint intentionally ignored here: this entrypoint is USDC-only
        settle_payout_v3_safe_v2_routed(ctx, nonce, 5, mint_in, amount_in, usd_micros, 1)
    }

    pub fn settle_payout_v3_safe_v2_routed(
        ctx: Context<SettlePayoutV3Safe>,
        nonce: u64,
        payout_kind: u8,
        mint_in: Pubkey,
        amount_in: u64,
        usd_micros: u64,
        route_hint: u8,
    ) -> Result<()> {
        require!(usd_micros > 0, SterlingError::InvalidAmount);
        enforce_ticket_cap_usd_micros(usd_micros)?;

        let pool = ctx.accounts.pool.key();
        let user = ctx.accounts.user.key();

        let cfg = &mut ctx.accounts.config;
        require_keeper(cfg, &ctx.accounts.keeper)?;

        // choose payout_mint using route_hint (may fallback to AUTO)
        let payout_mint = select_payout_mint_v3(cfg, route_hint);
        if payout_mint == Pubkey::default() {
            // nothing configured => no-op
            return Ok(());
        }
        require!(
            ctx.accounts.payout_mint.key() == payout_mint,
            SterlingError::InvalidAccount
        );

        // check threshold and limits
        require!(
            usd_micros >= cfg.payout_threshold_usd_micros,
            SterlingError::InvalidAmount
        );
        apply_payout_limits(cfg, usd_micros)?;

        let destination_ata = get_associated_token_address(&user, &payout_mint);
        let (escrow_authority, escrow_bump) =
            expected_ticket_escrow_authority(ctx.accounts.ticket.key());
        let escrow_ata = expected_live_escrow_ata(escrow_authority, mint_in);

        let now = Clock::get()?.unix_timestamp;
        let t = &mut ctx.accounts.ticket;
        require!(
            t.status == PAYOUT_TICKET_STATUS_REQUESTED,
            SterlingError::InvalidState
        );
        // Anti-rewrite lock: this request ticket must be initialized once.
        require!(t.created_ts == 0, SterlingError::InvalidState);
        require!(t.settled_ts == 0, SterlingError::InvalidState);

        t.pool = pool;
        t.payout_mint = payout_mint;
        t.payout_kind = payout_kind;
        t.user = user;
        t.mint_in = mint_in;
        t.amount_in = amount_in;
        t.usd_micros = usd_micros;
        t.destination_ata = destination_ata;
        t.escrow_mint = mint_in;
        t.escrow_ata = escrow_ata;
        t.escrow_amount_locked = amount_in;
        t.nonce = nonce;
        t.created_ts = now;
        t.settled_ts = 0;
        t.status = PAYOUT_TICKET_STATUS_REQUESTED;
        t.funding_state = SOVEREIGN_ESCROW_STATE_REQUESTED;
        t.escrow_bump = escrow_bump;
        t.route_hint = route_hint;
        t.bump = ctx.bumps.ticket;

        emit!(NeedPayoutEvent {
            pool,
            payout_mint,
            payout_kind,
            user,
            mint_in,
            amount_in,
            usd_micros,
            destination_ata,
            nonce,
            liquidity_policy: 1,
            route_hint,
            ts: now,
        });

        Ok(())
    }
    // =========================
    // PAO V3 — SettlementClaim (keeper-only, linked to PayoutTicket)
    // =========================
    pub fn create_claim_from_ticket(ctx: Context<CreateClaimFromTicket>, nonce: u64) -> Result<()> {
        require_keeper(&ctx.accounts.config, &ctx.accounts.keeper)?;

        let t = &ctx.accounts.ticket;

        validate_claimable_ticket(t, nonce, ctx.accounts.user.key(), ctx.accounts.pool.key())?;
        let payout_mint = t.payout_mint;
        require!(
            ctx.accounts.payout_mint.key() == payout_mint,
            SterlingError::InvalidAccount
        );

        let user = ctx.accounts.user.key();
        let destination_ata = get_associated_token_address(&user, &payout_mint);
        // enforce deterministic destination (no redirection)
        require!(
            destination_ata == t.destination_ata,
            SterlingError::InvalidAccount
        );

        let now = Clock::get()?.unix_timestamp;
        let claim_key = ctx.accounts.claim.key();
        let c = &mut ctx.accounts.claim;

        // =====================================================
        // ANTI-REWRITE + IDEMPOTENCE
        // - init_if_needed peut réutiliser un claim existant.
        // - On interdit toute réécriture différente.
        // - Mais on accepte les retries si le claim correspond déjà au ticket.
        // =====================================================
        if c.created_ts != 0 {
            // Le claim existe déjà: il doit déjà correspondre au ticket.
            require!(
                c.status == SETTLEMENT_CLAIM_STATUS_OPEN,
                SterlingError::InvalidState
            );
            require!(c.pool == t.pool, SterlingError::InvalidAccount);
            require!(
                c.payout_mint == t.payout_mint,
                SterlingError::InvalidAccount
            );
            require!(
                c.payout_kind == t.payout_kind,
                SterlingError::InvalidAccount
            );
            require!(c.user == t.user, SterlingError::InvalidAccount);
            require!(c.mint_in == t.mint_in, SterlingError::InvalidAccount);
            require!(c.amount_in == t.amount_in, SterlingError::InvalidAmount);
            require!(c.usd_micros == t.usd_micros, SterlingError::InvalidAmount);
            require!(
                c.destination_ata == t.destination_ata,
                SterlingError::InvalidAccount
            );
            require!(c.nonce == t.nonce, SterlingError::InvalidState);
            require!(c.escrow_mint == t.mint_in, SterlingError::InvalidAccount);
            require!(
                c.escrow_amount_locked == t.amount_in,
                SterlingError::InvalidAmount
            );

            // Retry OK: rien à faire
            return Ok(());
        }

        c.created_ts = now;
        require!(
            c.status == SETTLEMENT_CLAIM_STATUS_OPEN,
            SterlingError::InvalidState
        );

        let (escrow_authority, escrow_bump) = expected_claim_escrow_authority(claim_key);
        let escrow_ata = expected_live_escrow_ata(escrow_authority, t.mint_in);

        c.pool = t.pool;
        c.payout_mint = t.payout_mint;
        c.payout_kind = t.payout_kind;
        c.user = t.user;
        c.mint_in = t.mint_in;
        c.amount_in = t.amount_in;
        c.usd_micros = t.usd_micros;

        c.due_atoms = usd_micros_to_atoms(t.usd_micros, ctx.accounts.payout_mint.decimals)?;
        c.paid_atoms = 0;
        c.proof_sig = [0u8; 64];

        c.destination_ata = t.destination_ata;
        c.escrow_mint = t.mint_in;
        c.escrow_ata = escrow_ata;
        c.escrow_amount_locked = t.amount_in;
        c.nonce = t.nonce;
        c.settled_ts = 0;
        c.status = SETTLEMENT_CLAIM_STATUS_OPEN;
        c.funding_state = SOVEREIGN_ESCROW_STATE_REQUESTED;
        c.escrow_bump = escrow_bump;
        c.bump = ctx.bumps.claim;

        emit!(ClaimCreatedEvent {
            pool: c.pool,
            payout_mint: c.payout_mint,
            payout_kind: c.payout_kind,
            user: c.user,
            usd_micros: c.usd_micros,
            destination_ata: c.destination_ata,
            nonce: c.nonce,
            ts: now,
        });

        Ok(())
    }

    pub fn confirm_ticket_live_escrow_funding(
        ctx: Context<ConfirmTicketLiveEscrowFunding>,
        nonce: u64,
    ) -> Result<()> {
        require_keeper(&ctx.accounts.config, &ctx.accounts.keeper)?;

        let ticket_key = ctx.accounts.ticket.key();
        let ticket = &mut ctx.accounts.ticket;
        require!(ticket.nonce == nonce, SterlingError::InvalidState);
        require!(
            ticket.status == PAYOUT_TICKET_STATUS_REQUESTED,
            SterlingError::InvalidState
        );
        require!(ticket.settled_ts == 0, SterlingError::InvalidState);
        sync_ticket_escrow_funding_state(
            ticket_key,
            ticket,
            &ctx.accounts.source_escrow_ata,
            ctx.accounts.source_escrow_ata.key(),
        )
    }

    pub fn confirm_claim_live_escrow_funding(
        ctx: Context<ConfirmClaimLiveEscrowFunding>,
        nonce: u64,
    ) -> Result<()> {
        require_keeper(&ctx.accounts.config, &ctx.accounts.keeper)?;

        let claim_key = ctx.accounts.claim.key();
        let claim = &mut ctx.accounts.claim;
        require!(claim.nonce == nonce, SterlingError::InvalidState);
        require!(
            claim.status == SETTLEMENT_CLAIM_STATUS_OPEN,
            SterlingError::InvalidState
        );
        require!(claim.settled_ts == 0, SterlingError::InvalidState);
        sync_claim_escrow_funding_state(
            claim,
            claim_key,
            &ctx.accounts.source_escrow_ata,
            ctx.accounts.source_escrow_ata.key(),
        )
    }

    pub fn confirm_protocol_debt_lot_live_escrow_funding(
        ctx: Context<ConfirmProtocolDebtLotLiveEscrowFunding>,
        nonce: u64,
    ) -> Result<()> {
        require_keeper(&ctx.accounts.config, &ctx.accounts.keeper)?;

        let pool_key = ctx.accounts.pool.key();
        let mut ledger_data = ctx.accounts.protocol_debt_ledger.try_borrow_mut_data()?;
        let (lot_index, mut lot, ledger_pool_key) =
            load_protocol_debt_lot_from_ledger_data(&ledger_data, nonce)?;
        require!(ledger_pool_key == pool_key, SterlingError::InvalidAccount);
        require!(
            lot.status == PROTOCOL_DEBT_LOT_OPEN || lot.status == PROTOCOL_DEBT_LOT_TICKETED,
            SterlingError::InvalidState
        );
        sync_protocol_debt_lot_escrow_funding_state(
            pool_key,
            &mut lot,
            &ctx.accounts.source_escrow_ata,
            ctx.accounts.source_escrow_ata.key(),
        )?;
        store_protocol_debt_lot_into_ledger_data(&mut ledger_data, lot_index, &lot)
    }

    pub fn settle_claim_paid(
        ctx: Context<SettleClaimPaid>,
        nonce: u64,
        paid_atoms: u64,
        proof_sig: [u8; 64],
    ) -> Result<()> {
        require_keeper(&ctx.accounts.config, &ctx.accounts.keeper)?;
        require!(paid_atoms > 0, SterlingError::InvalidAmount);
        require_proof_kind_for_stage(&proof_sig, SettlementFundingStage::ProviderPayout)?;

        let claim = &mut ctx.accounts.claim;
        require!(claim.nonce == nonce, SterlingError::InvalidState);
        require!(
            claim.pool == ctx.accounts.pool.key(),
            SterlingError::InvalidAccount
        );
        require!(
            claim.user == ctx.accounts.user.key(),
            SterlingError::InvalidAccount
        );
        require!(
            claim.status == SETTLEMENT_CLAIM_STATUS_OPEN,
            SterlingError::InvalidState
        );
        require!(claim.settled_ts == 0, SterlingError::InvalidState);
        require!(
            paid_atoms >= claim.due_atoms.saturating_sub(claim.paid_atoms),
            SterlingError::InvalidAmount
        );

        let now = Clock::get()?.unix_timestamp;
        claim.paid_atoms = paid_atoms;
        claim.proof_sig = proof_sig;
        claim.settled_ts = now;
        claim.status = SETTLEMENT_CLAIM_STATUS_PAID;
        claim.funding_state = SOVEREIGN_ESCROW_STATE_SETTLED;

        emit!(ClaimPaidEvent {
            pool: claim.pool,
            payout_mint: claim.payout_mint,
            user: claim.user,
            usd_micros: claim.usd_micros,
            paid_atoms: claim.paid_atoms,
            proof_sig: claim.proof_sig,
            destination_ata: claim.destination_ata,
            nonce: claim.nonce,
            ts: now,
        });

        Ok(())
    }

    pub fn settle_protocol_fee_debt(
        ctx: Context<SettleProtocolFeeDebt>,
        paid_usd_micros: u64,
        proof_sig: [u8; 64],
    ) -> Result<()> {
        require_keeper(&ctx.accounts.config, &ctx.accounts.keeper)?;
        require_protocol_fee_debt_payment_proof(&proof_sig)?;
        require!(paid_usd_micros > 0, SterlingError::InvalidAmount);

        let mut ledger_data = ctx.accounts.protocol_debt_ledger.try_borrow_mut_data()?;
        let ledger_pool = protocol_debt_ledger_pool_from_data(&ledger_data)?;
        require!(ledger_pool == ctx.accounts.pool.key(), SterlingError::InvalidAccount);
        let now = Clock::get()?.unix_timestamp;
        let settled_usd_micros = settle_protocol_debt_lots_in_ledger_data(
            &mut ctx.accounts.pool,
            &mut ledger_data,
            paid_usd_micros,
            now,
        )?;

        emit!(ProtocolFeeDebtSettledEvent {
            pool: ctx.accounts.pool.key(),
            paid_usd_micros: settled_usd_micros,
            remaining_usd_micros: ctx.accounts.pool.protocol_fee_debt_usd_micros,
            proof_sig,
            ts: now,
        });

        Ok(())
    }

    pub fn init_protocol_debt_ledger(ctx: Context<InitProtocolDebtLedger>) -> Result<()> {
        crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;

        let mut ledger_data = ctx.accounts.protocol_debt_ledger.try_borrow_mut_data()?;
        initialize_protocol_debt_ledger_data(
            &mut ledger_data,
            ctx.accounts.pool.key(),
            ctx.bumps.protocol_debt_ledger,
        )?;

        Ok(())
    }

    pub fn materialize_protocol_fee_debt_lot_to_ticket(
        ctx: Context<SettlePayoutV3Safe>,
        nonce: u64,
    ) -> Result<()> {
        require_keeper(&ctx.accounts.config, &ctx.accounts.keeper)?;
        require!(
            ctx.remaining_accounts.len() == 1,
            SterlingError::StableSettlementAccountsMissing
        );
        require!(
            ctx.accounts.user.key() == ctx.accounts.config.main_wallet,
            SterlingError::InvalidAccount
        );

        let protocol_debt_ledger_info = ctx
            .remaining_accounts
            .first()
            .cloned()
            .ok_or_else(|| error!(SterlingError::StableSettlementAccountsMissing))?;
        require!(
            protocol_debt_ledger_info.owner == &crate::ID,
            SterlingError::InvalidAccount
        );
        let (expected_protocol_debt_ledger, _) = Pubkey::find_program_address(
            &[b"protocol_debt", ctx.accounts.pool.key().as_ref()],
            ctx.program_id,
        );
        require!(
            protocol_debt_ledger_info.key() == expected_protocol_debt_ledger,
            SterlingError::InvalidAccount
        );

        let (lot_index, mut lot, ledger_pool_key) = {
            let ledger_data = protocol_debt_ledger_info.try_borrow_data()?;
            load_protocol_debt_lot_from_ledger_data(&ledger_data, nonce)?
        };
        require!(
            ledger_pool_key == ctx.accounts.pool.key(),
            SterlingError::InvalidAccount
        );
        require!(
            lot.status == PROTOCOL_DEBT_LOT_OPEN || lot.status == PROTOCOL_DEBT_LOT_TICKETED,
            SterlingError::InvalidState
        );
        require!(lot.usd_micros > 0, SterlingError::InvalidAmount);
        require!(
            lot.funding_state == SOVEREIGN_ESCROW_STATE_REQUESTED,
            SterlingError::InvalidState
        );

        let (expected_payout_mint, expected_destination_ata) =
            protocol_fee_materialization_metadata(&ctx.accounts.config, lot.route_hint)?;
        require!(
            ctx.accounts.payout_mint.key() == expected_payout_mint,
            SterlingError::InvalidAccount
        );

        let ticket_key = ctx.accounts.ticket.key();
        let ticket = &mut ctx.accounts.ticket;
        if ticket.created_ts != 0 {
            validate_materialized_protocol_debt_ticket(
                ticket,
                ctx.accounts.pool.key(),
                ctx.accounts.user.key(),
                expected_payout_mint,
                expected_destination_ata,
                &lot,
                nonce,
            )?;
            return Ok(());
        }

        let now = Clock::get()?.unix_timestamp;
        let (ticket_escrow_authority, ticket_escrow_bump) =
            expected_ticket_escrow_authority(ticket_key);
        let ticket_escrow_ata = expected_live_escrow_ata(ticket_escrow_authority, lot.escrow_mint);

        ticket.pool = ctx.accounts.pool.key();
        ticket.payout_mint = expected_payout_mint;
        ticket.payout_kind = 5;
        ticket.user = ctx.accounts.user.key();
        ticket.mint_in = lot.mint_in;
        ticket.amount_in = lot.amount_in;
        ticket.usd_micros = lot.usd_micros;
        ticket.destination_ata = expected_destination_ata;
        ticket.escrow_mint = lot.escrow_mint;
        ticket.escrow_ata = ticket_escrow_ata;
        ticket.escrow_amount_locked = lot.escrow_amount_locked;
        ticket.nonce = nonce;
        ticket.created_ts = now;
        ticket.settled_ts = 0;
        ticket.status = PAYOUT_TICKET_STATUS_REQUESTED;
        ticket.funding_state = SOVEREIGN_ESCROW_STATE_REQUESTED;
        ticket.escrow_bump = ticket_escrow_bump;
        ticket.route_hint = lot.route_hint;
        ticket.bump = ctx.bumps.ticket;

        if lot.status == PROTOCOL_DEBT_LOT_OPEN {
            lot.status = PROTOCOL_DEBT_LOT_TICKETED;
            let mut ledger_data = protocol_debt_ledger_info.try_borrow_mut_data()?;
            store_protocol_debt_lot_into_ledger_data(&mut ledger_data, lot_index, &lot)?;
            store_protocol_debt_ledger_last_ts(&mut ledger_data, now)?;
        }

        emit!(NeedPayoutEvent {
            pool: ctx.accounts.pool.key(),
            payout_mint: expected_payout_mint,
            payout_kind: 5,
            user: ctx.accounts.user.key(),
            mint_in: lot.mint_in,
            amount_in: lot.amount_in,
            usd_micros: lot.usd_micros,
            destination_ata: expected_destination_ata,
            nonce,
            liquidity_policy: 1,
            route_hint: lot.route_hint,
            ts: now,
        });

        Ok(())
    }

    pub fn set_params(
        ctx: Context<AdminOnly>,
        cashback_bps: u16,
        reward_interval: u64,
        allow_fallback_usdt: bool,
        fee_threshold_usd_micros: u64,
        auto_collect_every_swaps: u64,
    ) -> Result<()> {
        crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        require!(cashback_bps <= 10_000, SterlingError::InvalidBps);
        require!(reward_interval >= 60, SterlingError::InvalidInterval);

        let cfg = &mut ctx.accounts.config;
        cfg.cashback_bps = cashback_bps;
        cfg.reward_interval = reward_interval;
        cfg.allow_fallback_usdt = allow_fallback_usdt;
        cfg.fee_threshold_usd_micros = fee_threshold_usd_micros;
        cfg.auto_collect_every_swaps = auto_collect_every_swaps;
        Ok(())
    }

    pub fn set_native_sol_rail(
        ctx: Context<AdminOnly>,
        enabled: bool,
        native_sol_usd_micros_per_sol: u64,
        native_sol_min_reserve_lamports: u64,
    ) -> Result<()> {
        crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        if enabled {
            require!(
                native_sol_usd_micros_per_sol > 0,
                SterlingError::InvalidAmount
            );
        }

        let cfg = &mut ctx.accounts.config;
        cfg.native_sol_enabled = enabled;
        cfg.native_sol_usd_micros_per_sol = native_sol_usd_micros_per_sol;
        cfg.native_sol_min_reserve_lamports = native_sol_min_reserve_lamports;
        Ok(())
    }

    pub fn set_valuation(
        ctx: Context<AdminOnly>,
        token_value_usd_micros_default: u64,
        treasury_value_usd_micros: u64,
    ) -> Result<()> {
        crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        require!(
            token_value_usd_micros_default > 0,
            SterlingError::InvalidAmount
        );
        require!(treasury_value_usd_micros > 0, SterlingError::InvalidAmount);

        let cfg = &mut ctx.accounts.config;
        cfg.token_value_usd_micros_default = token_value_usd_micros_default;
        cfg.treasury_value_usd_micros = treasury_value_usd_micros;

        Ok(())
    }

    pub fn set_mint_enabled(ctx: Context<AdminOnly>, mint: Pubkey, enabled: bool) -> Result<()> {
        crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        set_enabled_flag(&mut ctx.accounts.config, &mint, enabled)
    }

    pub fn set_live_runtime_config_v2(
        ctx: Context<AdminOnly>,
        keeper_authority: Pubkey,
        main_wallet: Pubkey,
        okx_wallet: Pubkey,
        usdc_mint: Pubkey,
        usdt_mint: Pubkey,
        treasury_usdc_ata: Pubkey,
        treasury_usdt_ata: Pubkey,
        usdc_coffre: Pubkey,
        usdt_coffre: Pubkey,
        pda_gt: Pubkey,
        coffre_7q: Pubkey,
        pool_id: Pubkey,
        allow_fallback_usdt: bool,
        auto_collect_every_swaps: u64,
        fee_threshold_usd_micros: u64,
        payout_threshold_usd_micros: u64,
        max_payout_usd_micros: u64,
        max_payout_per_window_usd_micros: u64,
        payout_window_secs: u64,
        lp_cashback_bps: u16,
        claim_cashback_bps: u16,
    ) -> Result<()> {
        crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        require!(
            keeper_authority != Pubkey::default(),
            SterlingError::InvalidAccount
        );
        require!(
            main_wallet != Pubkey::default(),
            SterlingError::InvalidAccount
        );
        require!(lp_cashback_bps <= 10_000, SterlingError::InvalidBps);
        require!(claim_cashback_bps <= 10_000, SterlingError::InvalidBps);
        require!(
            max_payout_per_window_usd_micros == 0 || payout_window_secs > 0,
            SterlingError::InvalidInterval
        );
        if usdc_coffre != Pubkey::default() && treasury_usdc_ata != Pubkey::default() {
            require!(
                usdc_coffre != treasury_usdc_ata,
                SterlingError::InvalidAccount
            );
        }
        if usdt_coffre != Pubkey::default() && treasury_usdt_ata != Pubkey::default() {
            require!(
                usdt_coffre != treasury_usdt_ata,
                SterlingError::InvalidAccount
            );
        }

        let cfg = &mut ctx.accounts.config;
        cfg.keeper_authority = keeper_authority;
        cfg.main_wallet = main_wallet;
        cfg.okx_wallet = okx_wallet;
        cfg.usdc_mint = usdc_mint;
        cfg.usdt_mint = usdt_mint;
        cfg.treasury_usdc_ata = treasury_usdc_ata;
        cfg.treasury_usdt_ata = treasury_usdt_ata;
        cfg.usdc_coffre = usdc_coffre;
        cfg.usdt_coffre = usdt_coffre;
        cfg.pda_gt = pda_gt;
        cfg.coffre_7q = coffre_7q;
        cfg.pool_id = pool_id;
        cfg.allow_fallback_usdt = allow_fallback_usdt;
        cfg.auto_collect_every_swaps = auto_collect_every_swaps;
        cfg.fee_threshold_usd_micros = fee_threshold_usd_micros;
        cfg.payout_threshold_usd_micros = payout_threshold_usd_micros;
        cfg.max_payout_usd_micros = max_payout_usd_micros;
        cfg.max_payout_per_window_usd_micros = max_payout_per_window_usd_micros;
        cfg.payout_window_secs = payout_window_secs;
        cfg.lp_cashback_bps = lp_cashback_bps;
        cfg.claim_cashback_bps = claim_cashback_bps;
        if max_payout_per_window_usd_micros == 0 || payout_window_secs == 0 {
            cfg.payout_window_start = 0;
            cfg.payout_window_used_usd_micros = 0;
        }
        Ok(())
    }

    pub fn set_extra_payout_rail_v2(
        ctx: Context<AdminOnly>,
        rail_index: u8,
        payout_mint: Pubkey,
        payout_vault_ata: Pubkey,
    ) -> Result<()> {
        crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        require!(rail_index < 4, SterlingError::InvalidAccount);
        if payout_mint == Pubkey::default() {
            require!(
                payout_vault_ata == Pubkey::default(),
                SterlingError::InvalidAccount
            );
        } else {
            require!(
                payout_vault_ata != Pubkey::default(),
                SterlingError::InvalidAccount
            );
        }

        let cfg = &mut ctx.accounts.config;
        match rail_index {
            0 => {
                cfg.extra_payout_mint_0 = payout_mint;
                cfg.extra_payout_vault_ata_0 = payout_vault_ata;
            }
            1 => {
                cfg.extra_payout_mint_1 = payout_mint;
                cfg.extra_payout_vault_ata_1 = payout_vault_ata;
            }
            2 => {
                cfg.extra_payout_mint_2 = payout_mint;
                cfg.extra_payout_vault_ata_2 = payout_vault_ata;
            }
            3 => {
                cfg.extra_payout_mint_3 = payout_mint;
                cfg.extra_payout_vault_ata_3 = payout_vault_ata;
            }
            _ => return err!(SterlingError::InvalidAccount),
        }
        Ok(())
    }

    pub fn backfill_pool_registry_entry(ctx: Context<BackfillPoolRegistryEntry>) -> Result<()> {
        crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        let pool = &ctx.accounts.pool;
        let created_at = if ctx.accounts.pool_registry_entry.created_at != 0 {
            ctx.accounts.pool_registry_entry.created_at
        } else if pool.created_at != 0 {
            pool.created_at
        } else {
            Clock::get()?.unix_timestamp
        };

        ctx.accounts
            .pool_registry_entry
            .set_inner(PoolRegistryEntry {
                pool: pool.key(),
                base_mint: pool.base_mint,
                quote_mint: pool.quote_mint,
                lp_mint: pool.lp_mint,
                base_vault: pool.base_vault,
                quote_vault: pool.quote_vault,
                fee_vault_base: pool.fee_vault_base,
                fee_vault_quote: pool.fee_vault_quote,
                created_at,
                bump: ctx.bumps.pool_registry_entry,
            });

        Ok(())
    }

    // =========================
    // B) ORACLE ON-CHAIN : ValueRegistry
    // =========================
    pub fn init_value_registry(
        ctx: Context<InitValueRegistry>,
        value_usd_micros: u64,
    ) -> Result<()> {
        crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        require!(value_usd_micros > 0, SterlingError::InvalidAmount);

        let reg = &mut ctx.accounts.value_registry;
        reg.mint = ctx.accounts.mint.key();
        reg.value_usd_micros = value_usd_micros;

        reg.true_cash = TRUE_CASH_FLAG;
        reg.cash_backed = CASH_BACKED_FLAG;
        reg.real_peg = REAL_PEG_FLAG;
        reg.sovereign = SOVEREIGN_FLAG;

        reg.updated_at = Clock::get()?.unix_timestamp;
        reg.bump = ctx.bumps.value_registry;
        Ok(())
    }

    pub fn set_token_value(ctx: Context<SetTokenValue>, value_usd_micros: u64) -> Result<()> {
        crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        require!(value_usd_micros > 0, SterlingError::InvalidAmount);
        require!(
            ctx.accounts.value_registry.mint == ctx.accounts.mint.key(),
            SterlingError::BadRegistry
        );

        let reg = &mut ctx.accounts.value_registry;
        reg.value_usd_micros = value_usd_micros;
        reg.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn create_pool(ctx: Context<CreatePool>) -> Result<()> {
        crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        let now = Clock::get()?.unix_timestamp;
        require!(
            ctx.accounts.base_value_registry.mint == ctx.accounts.base_mint.key(),
            SterlingError::BadRegistry
        );
        require!(
            ctx.accounts.quote_value_registry.mint == ctx.accounts.quote_mint.key(),
            SterlingError::BadRegistry
        );

        ctx.accounts.pool.set_inner(Pool {
            owner: ctx.accounts.admin.key(),
            base_mint: ctx.accounts.base_mint.key(),
            quote_mint: ctx.accounts.quote_mint.key(),
            base_vault: Pubkey::default(),
            quote_vault: Pubkey::default(),
            lp_mint: Pubkey::default(),
            fee_vault_base: Pubkey::default(),
            fee_vault_quote: Pubkey::default(),
            base_value_usd_micros: ctx.accounts.base_value_registry.value_usd_micros,
            quote_value_usd_micros: ctx.accounts.quote_value_registry.value_usd_micros,
            true_cash: TRUE_CASH_FLAG,
            cash_backed: CASH_BACKED_FLAG,
            real_peg: REAL_PEG_FLAG,
            sovereign: SOVEREIGN_FLAG,
            fee_bps: DEFAULT_SWAP_FEE_BPS,
            active: true,
            swap_cashback_bps: 0,
            protocol_fee_debt_usd_micros: 0,
            protocol_fee_debt_count: 0,
            protocol_fee_debt_last_ts: 0,
            created_at: now,
            last_swap_ts: 0,
            total_base_volume: 0,
            total_quote_volume: 0,
            swap_count: 0,
            bump: ctx.bumps.pool,
        });

        ctx.accounts
            .pool_registry_entry
            .set_inner(PoolRegistryEntry {
                pool: ctx.accounts.pool.key(),
                base_mint: ctx.accounts.base_mint.key(),
                quote_mint: ctx.accounts.quote_mint.key(),
                lp_mint: Pubkey::default(),
                base_vault: Pubkey::default(),
                quote_vault: Pubkey::default(),
                fee_vault_base: Pubkey::default(),
                fee_vault_quote: Pubkey::default(),
                created_at: now,
                bump: ctx.bumps.pool_registry_entry,
            });

        emit!(PoolCreated {
            pool: ctx.accounts.pool.key(),
            owner: ctx.accounts.admin.key(),
            base_mint: ctx.accounts.base_mint.key(),
            quote_mint: ctx.accounts.quote_mint.key(),
            ts: Clock::get()?.unix_timestamp,
        });

        emit!(PoolRegistered {
            pool: ctx.accounts.pool.key(),
            base_mint: ctx.accounts.base_mint.key(),
            quote_mint: ctx.accounts.quote_mint.key(),
            lp_mint: Pubkey::default(),
            ts: now,
        });

        Ok(())
    }

    pub fn init_pool_base_vault(ctx: Context<InitPoolBaseVault>) -> Result<()> {
        crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        require!(
            ctx.accounts.base_mint.key() == ctx.accounts.pool.base_mint,
            SterlingError::InvalidAccount
        );

        set_pool_base_vault(
            &mut ctx.accounts.pool,
            ctx.accounts.base_mint.key(),
            ctx.accounts.base_vault.key(),
        )?;
        sync_pool_registry_runtime_field(
            &mut ctx.accounts.pool_registry_entry,
            ctx.accounts.pool.key(),
            1,
            ctx.accounts.base_vault.key(),
        )?;

        emit!(PoolRuntimeAddressSet {
            pool: ctx.accounts.pool.key(),
            role: 1,
            mint: ctx.accounts.base_mint.key(),
            account: ctx.accounts.base_vault.key(),
            ts: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn init_pool_quote_vault(ctx: Context<InitPoolQuoteVault>) -> Result<()> {
        crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        require!(
            ctx.accounts.quote_mint.key() == ctx.accounts.pool.quote_mint,
            SterlingError::InvalidAccount
        );

        set_pool_quote_vault(
            &mut ctx.accounts.pool,
            ctx.accounts.quote_mint.key(),
            ctx.accounts.quote_vault.key(),
        )?;
        sync_pool_registry_runtime_field(
            &mut ctx.accounts.pool_registry_entry,
            ctx.accounts.pool.key(),
            2,
            ctx.accounts.quote_vault.key(),
        )?;

        emit!(PoolRuntimeAddressSet {
            pool: ctx.accounts.pool.key(),
            role: 2,
            mint: ctx.accounts.quote_mint.key(),
            account: ctx.accounts.quote_vault.key(),
            ts: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn init_pool_lp_mint(ctx: Context<InitPoolLpMint>) -> Result<()> {
        crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        set_pool_lp_mint(&mut ctx.accounts.pool, ctx.accounts.lp_mint.key())?;
        sync_pool_registry_runtime_field(
            &mut ctx.accounts.pool_registry_entry,
            ctx.accounts.pool.key(),
            3,
            ctx.accounts.lp_mint.key(),
        )?;

        emit!(PoolRuntimeAddressSet {
            pool: ctx.accounts.pool.key(),
            role: 3,
            mint: ctx.accounts.lp_mint.key(),
            account: ctx.accounts.lp_mint.key(),
            ts: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn init_pool_fee_vault_base(ctx: Context<InitPoolFeeVaultBase>) -> Result<()> {
        crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        require!(
            ctx.accounts.base_mint.key() == ctx.accounts.pool.base_mint,
            SterlingError::InvalidAccount
        );

        set_pool_fee_vault_base(
            &mut ctx.accounts.pool,
            ctx.accounts.base_mint.key(),
            ctx.accounts.fee_vault_base.key(),
        )?;
        sync_pool_registry_runtime_field(
            &mut ctx.accounts.pool_registry_entry,
            ctx.accounts.pool.key(),
            4,
            ctx.accounts.fee_vault_base.key(),
        )?;

        emit!(PoolRuntimeAddressSet {
            pool: ctx.accounts.pool.key(),
            role: 4,
            mint: ctx.accounts.base_mint.key(),
            account: ctx.accounts.fee_vault_base.key(),
            ts: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn init_pool_fee_vault_quote(ctx: Context<InitPoolFeeVaultQuote>) -> Result<()> {
        crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        require!(
            ctx.accounts.quote_mint.key() == ctx.accounts.pool.quote_mint,
            SterlingError::InvalidAccount
        );

        set_pool_fee_vault_quote(
            &mut ctx.accounts.pool,
            ctx.accounts.quote_mint.key(),
            ctx.accounts.fee_vault_quote.key(),
        )?;
        sync_pool_registry_runtime_field(
            &mut ctx.accounts.pool_registry_entry,
            ctx.accounts.pool.key(),
            5,
            ctx.accounts.fee_vault_quote.key(),
        )?;

        emit!(PoolRuntimeAddressSet {
            pool: ctx.accounts.pool.key(),
            role: 5,
            mint: ctx.accounts.quote_mint.key(),
            account: ctx.accounts.fee_vault_quote.key(),
            ts: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn register_asset(
        ctx: Context<RegisterAsset>,
        valuation_usd_micros: u64,
        is_lp: bool,
    ) -> Result<()> {
        crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        initialize_asset_registry(
            &mut ctx.accounts.asset_registry,
            ctx.accounts.token_mint.key(),
            valuation_usd_micros,
            ctx.accounts.token_mint.decimals,
            is_lp,
            ctx.bumps.asset_registry,
        )
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        base_amount_in: u64,
        quote_amount_in: u64,
        min_lp_out: u64,
    ) -> Result<()> {
        require!(base_amount_in > 0, SterlingError::InvalidAmount);
        require!(quote_amount_in > 0, SterlingError::InvalidAmount);
        validate_liquidity_accounts(
            &ctx.accounts.pool,
            ctx.accounts.user.key(),
            &ctx.accounts.user_base_ata,
            &ctx.accounts.user_quote_ata,
            &ctx.accounts.user_lp_ata,
            ctx.accounts.base_vault.key(),
            &ctx.accounts.base_vault,
            ctx.accounts.quote_vault.key(),
            &ctx.accounts.quote_vault,
            ctx.accounts.lp_mint.key(),
            &ctx.accounts.lp_mint,
        )?;

        let lp_out = compute_lp_out(
            ctx.accounts.base_vault.amount,
            ctx.accounts.quote_vault.amount,
            ctx.accounts.lp_mint.supply,
            base_amount_in,
            quote_amount_in,
        )?;

        require!(lp_out > 0, SterlingError::ZeroLp);
        require!(lp_out >= min_lp_out, SterlingError::SlippageExceeded);

        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[ctx.bumps.config]]];

        transfer_tokens(
            &ctx.accounts.token_program,
            ctx.accounts.user_base_ata.to_account_info(),
            ctx.accounts.base_vault.to_account_info(),
            ctx.accounts.user.to_account_info(),
            base_amount_in,
        )?;
        transfer_tokens(
            &ctx.accounts.token_program,
            ctx.accounts.user_quote_ata.to_account_info(),
            ctx.accounts.quote_vault.to_account_info(),
            ctx.accounts.user.to_account_info(),
            quote_amount_in,
        )?;
        mint_tokens_signed(
            &ctx.accounts.token_program,
            ctx.accounts.lp_mint.to_account_info(),
            ctx.accounts.user_lp_ata.to_account_info(),
            ctx.accounts.config.to_account_info(),
            signer_seeds,
            lp_out,
        )?;

        emit!(LiquidityAdded {
            pool: ctx.accounts.pool.key(),
            user: ctx.accounts.user.key(),
            base_mint: ctx.accounts.pool.base_mint,
            quote_mint: ctx.accounts.pool.quote_mint,
            base_amount_in,
            quote_amount_in,
            lp_amount_out: lp_out,
            ts: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        lp_amount_in: u64,
        min_base_out: u64,
        min_quote_out: u64,
    ) -> Result<()> {
        require!(lp_amount_in > 0, SterlingError::InvalidAmount);
        validate_liquidity_accounts(
            &ctx.accounts.pool,
            ctx.accounts.user.key(),
            &ctx.accounts.user_base_ata,
            &ctx.accounts.user_quote_ata,
            &ctx.accounts.user_lp_ata,
            ctx.accounts.base_vault.key(),
            &ctx.accounts.base_vault,
            ctx.accounts.quote_vault.key(),
            &ctx.accounts.quote_vault,
            ctx.accounts.lp_mint.key(),
            &ctx.accounts.lp_mint,
        )?;

        let (base_out, quote_out) = compute_liquidity_outs(
            ctx.accounts.base_vault.amount,
            ctx.accounts.quote_vault.amount,
            ctx.accounts.lp_mint.supply,
            lp_amount_in,
        )?;

        require!(base_out > 0, SterlingError::InvalidAmount);
        require!(quote_out > 0, SterlingError::InvalidAmount);
        require!(base_out >= min_base_out, SterlingError::SlippageExceeded);
        require!(quote_out >= min_quote_out, SterlingError::SlippageExceeded);

        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[ctx.bumps.config]]];

        burn_tokens(
            &ctx.accounts.token_program,
            ctx.accounts.lp_mint.to_account_info(),
            ctx.accounts.user_lp_ata.to_account_info(),
            ctx.accounts.user.to_account_info(),
            lp_amount_in,
        )?;
        transfer_tokens_signed(
            &ctx.accounts.token_program,
            ctx.accounts.base_vault.to_account_info(),
            ctx.accounts.user_base_ata.to_account_info(),
            ctx.accounts.config.to_account_info(),
            signer_seeds,
            base_out,
        )?;
        transfer_tokens_signed(
            &ctx.accounts.token_program,
            ctx.accounts.quote_vault.to_account_info(),
            ctx.accounts.user_quote_ata.to_account_info(),
            ctx.accounts.config.to_account_info(),
            signer_seeds,
            quote_out,
        )?;

        emit!(LiquidityRemoved {
            pool: ctx.accounts.pool.key(),
            user: ctx.accounts.user.key(),
            base_mint: ctx.accounts.pool.base_mint,
            quote_mint: ctx.accounts.pool.quote_mint,
            lp_amount_in,
            base_amount_out: base_out,
            quote_amount_out: quote_out,
            ts: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn collect_fees_stable_to_treasury<'info>(
        ctx: Context<'_, '_, 'info, 'info, CollectFeesStableToTreasury<'info>>,
    ) -> Result<()> {
        crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        require!(
            ctx.accounts.fee_vault_base.key() == ctx.accounts.pool.fee_vault_base,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.fee_vault_quote.key() == ctx.accounts.pool.fee_vault_quote,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.treasury_usdc_ata.key() == ctx.accounts.config.treasury_usdc_ata,
            SterlingError::UsdcSettlementTreasuryMismatch
        );
        require!(
            ctx.accounts.treasury_usdt_ata.key() == ctx.accounts.config.treasury_usdt_ata,
            SterlingError::InvalidAccount
        );

        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[ctx.bumps.config]]];
        let pool_key = ctx.accounts.pool.key();
        let extra_fee_targets =
            build_explicit_extra_fee_rail_targets(&ctx.accounts.config, ctx.remaining_accounts)?;

        collect_stable_fee_vault(
            pool_key,
            &ctx.accounts.config,
            &ctx.accounts.token_program,
            &ctx.accounts.fee_vault_base,
            &ctx.accounts.treasury_usdc_ata,
            &ctx.accounts.treasury_usdt_ata,
            &extra_fee_targets,
            signer_seeds,
        )?;
        collect_stable_fee_vault(
            pool_key,
            &ctx.accounts.config,
            &ctx.accounts.token_program,
            &ctx.accounts.fee_vault_quote,
            &ctx.accounts.treasury_usdc_ata,
            &ctx.accounts.treasury_usdt_ata,
            &extra_fee_targets,
            signer_seeds,
        )?;

        Ok(())
    }

    pub fn convert_fees_to_usdc(ctx: Context<ConvertFeesToUsdc>) -> Result<()> {
        crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        require!(
            ctx.accounts.asset_registry.mint == ctx.accounts.fee_mint.key(),
            SterlingError::FeeAssetRegistryMissing
        );
        require!(
            ctx.accounts.asset_registry.active,
            SterlingError::FeeAssetRegistryInactive
        );
        require!(
            ctx.accounts.asset_registry.decimals == ctx.accounts.fee_mint.decimals,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.usdc_coffre_ata.key() == ctx.accounts.config.usdc_coffre,
            SterlingError::UsdcSettlementVaultMismatch
        );
        require!(
            ctx.accounts.treasury_usdc_ata.key() == ctx.accounts.config.treasury_usdc_ata,
            SterlingError::UsdcSettlementTreasuryMismatch
        );
        require!(
            ctx.accounts.usdc_coffre_ata.mint == ctx.accounts.config.usdc_mint,
            SterlingError::UsdcSettlementVaultMismatch
        );
        require!(
            ctx.accounts.treasury_usdc_ata.mint == ctx.accounts.config.usdc_mint,
            SterlingError::UsdcSettlementTreasuryMismatch
        );

        let (source_vault, source_key) =
            if ctx.accounts.fee_mint.key() == ctx.accounts.pool.base_mint {
                (
                    &ctx.accounts.fee_vault_base,
                    ctx.accounts.pool.fee_vault_base,
                )
            } else if ctx.accounts.fee_mint.key() == ctx.accounts.pool.quote_mint {
                (
                    &ctx.accounts.fee_vault_quote,
                    ctx.accounts.pool.fee_vault_quote,
                )
            } else {
                return err!(SterlingError::InvalidAccount);
            };

        require!(
            source_vault.key() == source_key,
            SterlingError::InvalidAccount
        );
        require!(
            source_vault.mint == ctx.accounts.fee_mint.key(),
            SterlingError::InvalidAccount
        );

        let fee_amount = source_vault.amount;
        require!(fee_amount > 0, SterlingError::InvalidAmount);

        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[ctx.bumps.config]]];
        let pool_key = ctx.accounts.pool.key();

        if ctx.accounts.fee_mint.key() == ctx.accounts.config.usdc_mint {
            transfer_tokens_signed(
                &ctx.accounts.token_program,
                source_vault.to_account_info(),
                ctx.accounts.treasury_usdc_ata.to_account_info(),
                ctx.accounts.config.to_account_info(),
                signer_seeds,
                fee_amount,
            )?;

            emit!(FeeCollectedToTreasuryEvent {
                pool: pool_key,
                fee_vault: source_vault.key(),
                mint: ctx.accounts.fee_mint.key(),
                amount: fee_amount,
                treasury_ata: ctx.accounts.treasury_usdc_ata.key(),
                ts: Clock::get()?.unix_timestamp,
            });

            return Ok(());
        }

        let usdc_out = compute_usdc_settlement_amount(
            fee_amount,
            ctx.accounts.asset_registry.valuation_usd_micros,
            ctx.accounts.fee_mint.decimals,
        )?;
        require!(usdc_out > 0, SterlingError::ZeroUsdcSettlement);
        require!(
            ctx.accounts.usdc_coffre_ata.amount >= usdc_out,
            SterlingError::InsufficientUsdcSettlementLiquidity
        );

        burn_tokens_signed(
            &ctx.accounts.token_program,
            ctx.accounts.fee_mint.to_account_info(),
            source_vault.to_account_info(),
            ctx.accounts.config.to_account_info(),
            signer_seeds,
            fee_amount,
        )?;
        transfer_tokens_signed(
            &ctx.accounts.token_program,
            ctx.accounts.usdc_coffre_ata.to_account_info(),
            ctx.accounts.treasury_usdc_ata.to_account_info(),
            ctx.accounts.config.to_account_info(),
            signer_seeds,
            usdc_out,
        )?;

        emit!(FeeConvertedToUsdcEvent {
            pool: pool_key,
            burned_mint: ctx.accounts.fee_mint.key(),
            burned_amount: fee_amount,
            usdc_released: usdc_out,
            usdc_coffre: ctx.accounts.usdc_coffre_ata.key(),
            treasury_usdc_ata: ctx.accounts.treasury_usdc_ata.key(),
            ts: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn swap_base_for_quote<'info>(
        ctx: Context<'_, '_, 'info, 'info, SwapBaseForQuote<'info>>,
        amount_in: u64,
        min_out: u64,
    ) -> Result<()> {
        require!(amount_in > 0, SterlingError::InvalidAmount);
        let pool_key = ctx.accounts.pool.key();
        let pool_active = ctx.accounts.pool.active;
        let base_mint = ctx.accounts.pool.base_mint;
        let quote_mint = ctx.accounts.pool.quote_mint;
        let base_vault = ctx.accounts.pool.base_vault;
        let quote_vault = ctx.accounts.pool.quote_vault;
        let fee_vault_base = ctx.accounts.pool.fee_vault_base;
        let fee_bps = ctx.accounts.pool.fee_bps as u128;
        let swap_cashback_bps = ctx.accounts.pool.swap_cashback_bps;
        require!(pool_active, SterlingError::InactivePool);

        require!(
            ctx.accounts.user_base_ata.owner == ctx.accounts.user.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_quote_ata.owner == ctx.accounts.user.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_base_ata.mint == base_mint,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_quote_ata.mint == quote_mint,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.base_vault.key() == base_vault,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.quote_vault.key() == quote_vault,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.fee_vault_base.key() == fee_vault_base,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.base_mint.key() == base_mint,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.base_value_registry.mint == base_mint,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.base_asset_registry.mint == base_mint,
            SterlingError::FeeAssetRegistryMissing
        );
        require!(
            ctx.accounts.base_asset_registry.active,
            SterlingError::FeeAssetRegistryInactive
        );
        require!(
            ctx.accounts.base_asset_registry.decimals == ctx.accounts.base_mint.decimals,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.fee_vault_base.mint == base_mint,
            SterlingError::InvalidAccount
        );

        let fee_amount = (amount_in as u128 * fee_bps / 10_000) as u64;

        let amount_in_less_fee = (amount_in as u128)
            .saturating_mul(10_000u128.saturating_sub(fee_bps))
            .checked_div(10_000u128)
            .ok_or(SterlingError::MathOverflow)? as u64;

        let x = ctx.accounts.base_vault.amount as u128;
        let y = ctx.accounts.quote_vault.amount as u128;

        let out = (amount_in_less_fee as u128)
            .saturating_mul(y)
            .checked_div(x.saturating_add(amount_in_less_fee as u128))
            .ok_or(SterlingError::MathOverflow)? as u64;

        require!(out >= min_out, SterlingError::SlippageExceeded);
        require!(out > 0, SterlingError::InvalidAmount);
        require!(
            ctx.accounts.quote_vault.amount >= out,
            SterlingError::InsufficientLiquidity
        );

        let (htop_stm_reserve_info_opt, htop_sjbc_reserve_info_opt, extra_fee_remaining_accounts) =
            split_swap_base_remaining_accounts_for_floor(pool_key, ctx.remaining_accounts)?;

        if is_floor_protected_pool_key(pool_key) {
            require_floor_protected_pool_bindings(pool_key, &ctx.accounts.pool)?;

            let htop_stm_reserve_info = htop_stm_reserve_info_opt
                .ok_or_else(|| error!(SterlingError::InvalidAccount))?;
            let htop_sjbc_reserve_info = htop_sjbc_reserve_info_opt
                .ok_or_else(|| error!(SterlingError::InvalidAccount))?;

            validate_target_floor_reserve_accounts(&htop_stm_reserve_info, &htop_sjbc_reserve_info)?;

            let post_base_atoms = ctx.accounts.base_vault.amount.saturating_add(amount_in_less_fee);
            let post_quote_atoms = ctx
                .accounts
                .quote_vault
                .amount
                .checked_sub(out)
                .ok_or(SterlingError::MathOverflow)?;
            enforce_floor_price_post_swap(
                FLOOR_PRICE_USD_MICROS,
                post_base_atoms,
                post_quote_atoms,
            )?;
        }

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_base_ata.to_account_info(),
                    to: ctx.accounts.base_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_in,
        )?;

        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[ctx.bumps.config]]];
        let extra_fee_targets =
            build_explicit_extra_fee_rail_targets(&ctx.accounts.config, extra_fee_remaining_accounts)?;
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.base_vault.to_account_info(),
                    to: ctx.accounts.fee_vault_base.to_account_info(),
                    authority: ctx.accounts.config.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            fee_amount,
        )?;

        let (fee_usd_micros, _accrue_protocol_debt) =
            settle_swap_fee_with_stable_fallback_and_debt(
            &ctx.accounts.config,
            &mut ctx.accounts.pool,
            &ctx.accounts.fee_vault_base,
            &ctx.accounts.base_mint,
            &ctx.accounts.base_asset_registry,
            fee_amount,
            &ctx.accounts.token_program,
            signer_seeds,
            &ctx.accounts.usdc_coffre_ata,
            &ctx.accounts.treasury_usdc_ata,
            &ctx.accounts.usdt_coffre_ata,
            &ctx.accounts.treasury_usdt_ata,
            &extra_fee_targets,
        )?;

        if fee_usd_micros >= ctx.accounts.config.fee_threshold_usd_micros {
            emit!(FeeThresholdEvent {
                pool: pool_key,
                fee_vault: ctx.accounts.fee_vault_base.key(),
                fee_amount,
                fee_value_usd_micros: fee_usd_micros,
                ts: Clock::get()?.unix_timestamp,
            });
        }

        ctx.accounts.pool.swap_count = ctx.accounts.pool.swap_count.saturating_add(1);
        ctx.accounts.pool.last_swap_ts = Clock::get()?.unix_timestamp;
        ctx.accounts.pool.total_base_volume = ctx
            .accounts
            .pool
            .total_base_volume
            .saturating_add(amount_in);
        ctx.accounts.pool.total_quote_volume =
            ctx.accounts.pool.total_quote_volume.saturating_add(out);
        let swap_count = ctx.accounts.pool.swap_count;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.quote_vault.to_account_info(),
                    to: ctx.accounts.user_quote_ata.to_account_info(),
                    authority: ctx.accounts.config.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            out,
        )?;

        let usd_micros_trade: u64 = ((ctx.accounts.base_value_registry.value_usd_micros as u128)
            .saturating_mul(amount_in_less_fee as u128)
            .checked_div(USD_MICROS as u128)
            .ok_or(SterlingError::MathOverflow)?) as u64;

        maybe_emit_payout_event_v3_safe(
            &mut ctx.accounts.config,
            pool_key,
            1,
            ctx.accounts.user.key(),
            base_mint,
            amount_in,
            usd_micros_trade,
            swap_cashback_bps,
            swap_count,
            1,
        )?;

        emit!(SwapExecuted {
            pool: pool_key,
            user: ctx.accounts.user.key(),
            side: FeeSide::Base,
            mint_in: base_mint,
            amount_in,
            mint_out: quote_mint,
            amount_out: out,
            fee_amount,
            fee_value_usd_micros: fee_usd_micros,
            ts: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn swap_quote_for_base<'info>(
        ctx: Context<'_, '_, 'info, 'info, SwapQuoteForBase<'info>>,
        amount_in: u64,
        min_out: u64,
    ) -> Result<()> {
        require!(amount_in > 0, SterlingError::InvalidAmount);
        let pool_key = ctx.accounts.pool.key();
        let pool_active = ctx.accounts.pool.active;
        let base_mint = ctx.accounts.pool.base_mint;
        let quote_mint = ctx.accounts.pool.quote_mint;
        let base_vault = ctx.accounts.pool.base_vault;
        let quote_vault = ctx.accounts.pool.quote_vault;
        let fee_vault_quote = ctx.accounts.pool.fee_vault_quote;
        let fee_bps = ctx.accounts.pool.fee_bps as u128;
        let swap_cashback_bps = ctx.accounts.pool.swap_cashback_bps;
        require!(pool_active, SterlingError::InactivePool);

        require!(
            ctx.accounts.user_quote_ata.owner == ctx.accounts.user.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_base_ata.owner == ctx.accounts.user.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_quote_ata.mint == quote_mint,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_base_ata.mint == base_mint,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.quote_vault.key() == quote_vault,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.base_vault.key() == base_vault,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.fee_vault_quote.key() == fee_vault_quote,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.quote_mint.key() == quote_mint,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.quote_value_registry.mint == quote_mint,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.quote_asset_registry.mint == quote_mint,
            SterlingError::FeeAssetRegistryMissing
        );
        require!(
            ctx.accounts.quote_asset_registry.active,
            SterlingError::FeeAssetRegistryInactive
        );
        require!(
            ctx.accounts.quote_asset_registry.decimals == ctx.accounts.quote_mint.decimals,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.fee_vault_quote.mint == quote_mint,
            SterlingError::InvalidAccount
        );

        let fee_amount = (amount_in as u128 * fee_bps / 10_000) as u64;

        let amount_in_less_fee = (amount_in as u128)
            .saturating_mul(10_000u128.saturating_sub(fee_bps))
            .checked_div(10_000u128)
            .ok_or(SterlingError::MathOverflow)? as u64;

        let x = ctx.accounts.quote_vault.amount as u128;
        let y = ctx.accounts.base_vault.amount as u128;

        let out = (amount_in_less_fee as u128)
            .saturating_mul(y)
            .checked_div(x.saturating_add(amount_in_less_fee as u128))
            .ok_or(SterlingError::MathOverflow)? as u64;

        require!(out >= min_out, SterlingError::SlippageExceeded);
        require!(out > 0, SterlingError::InvalidAmount);
        require!(
            ctx.accounts.base_vault.amount >= out,
            SterlingError::InsufficientLiquidity
        );

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_quote_ata.to_account_info(),
                    to: ctx.accounts.quote_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_in,
        )?;

        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[ctx.bumps.config]]];
        let extra_fee_targets =
            build_explicit_extra_fee_rail_targets(&ctx.accounts.config, ctx.remaining_accounts)?;
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.quote_vault.to_account_info(),
                    to: ctx.accounts.fee_vault_quote.to_account_info(),
                    authority: ctx.accounts.config.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            fee_amount,
        )?;

        let (fee_usd_micros, _accrue_protocol_debt) =
            settle_swap_fee_with_stable_fallback_and_debt(
            &ctx.accounts.config,
            &mut ctx.accounts.pool,
            &ctx.accounts.fee_vault_quote,
            &ctx.accounts.quote_mint,
            &ctx.accounts.quote_asset_registry,
            fee_amount,
            &ctx.accounts.token_program,
            signer_seeds,
            &ctx.accounts.usdc_coffre_ata,
            &ctx.accounts.treasury_usdc_ata,
            &ctx.accounts.usdt_coffre_ata,
            &ctx.accounts.treasury_usdt_ata,
            &extra_fee_targets,
        )?;

        if fee_usd_micros >= ctx.accounts.config.fee_threshold_usd_micros {
            emit!(FeeThresholdEvent {
                pool: pool_key,
                fee_vault: ctx.accounts.fee_vault_quote.key(),
                fee_amount,
                fee_value_usd_micros: fee_usd_micros,
                ts: Clock::get()?.unix_timestamp,
            });
        }

        ctx.accounts.pool.swap_count = ctx.accounts.pool.swap_count.saturating_add(1);
        ctx.accounts.pool.last_swap_ts = Clock::get()?.unix_timestamp;
        ctx.accounts.pool.total_quote_volume = ctx
            .accounts
            .pool
            .total_quote_volume
            .saturating_add(amount_in);
        ctx.accounts.pool.total_base_volume =
            ctx.accounts.pool.total_base_volume.saturating_add(out);
        let swap_count = ctx.accounts.pool.swap_count;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.base_vault.to_account_info(),
                    to: ctx.accounts.user_base_ata.to_account_info(),
                    authority: ctx.accounts.config.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            out,
        )?;

        let usd_micros_trade: u64 = ((ctx.accounts.quote_value_registry.value_usd_micros as u128)
            .saturating_mul(amount_in_less_fee as u128)
            .checked_div(USD_MICROS as u128)
            .ok_or(SterlingError::MathOverflow)?) as u64;

        maybe_emit_payout_event_v3_safe(
            &mut ctx.accounts.config,
            pool_key,
            1,
            ctx.accounts.user.key(),
            quote_mint,
            amount_in,
            usd_micros_trade,
            swap_cashback_bps,
            swap_count,
            1,
        )?;

        emit!(SwapExecuted {
            pool: pool_key,
            user: ctx.accounts.user.key(),
            side: FeeSide::Quote,
            mint_in: quote_mint,
            amount_in,
            mint_out: base_mint,
            amount_out: out,
            fee_amount,
            fee_value_usd_micros: fee_usd_micros,
            ts: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    // =========================
    // C) STAKING (vaults) + fallback USDT
    // =========================
    pub fn init_stake_vault(ctx: Context<InitStakeVault>) -> Result<()> {
        crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        let mint = ctx.accounts.mint.key();
        require!(
            is_supported_staking_reward_mint(&ctx.accounts.config, &mint),
            SterlingError::UnsupportedMint
        );
        require!(
            is_enabled_staking_reward_mint(&ctx.accounts.config, &mint),
            SterlingError::MintDisabled
        );
        Ok(())
    }

    pub fn init_reward_vault(ctx: Context<InitRewardVault>) -> Result<()> {
        crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        let mint = ctx.accounts.mint.key();
        require!(
            is_supported_staking_reward_mint(&ctx.accounts.config, &mint),
            SterlingError::UnsupportedMint
        );
        require!(
            is_enabled_staking_reward_mint(&ctx.accounts.config, &mint),
            SterlingError::MintDisabled
        );
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        require!(amount > 0, SterlingError::InvalidAmount);

        require!(
            ctx.accounts.user_stake_ata.owner == ctx.accounts.user.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_stake_ata.mint == ctx.accounts.stake_mint.key(),
            SterlingError::InvalidAccount
        );

        let cfg = &ctx.accounts.config;
        let stake_mint = ctx.accounts.stake_mint.key();
        let payout_mint = ctx.accounts.payout_mint.key();

        require!(
            is_supported_staking_reward_mint(cfg, &stake_mint),
            SterlingError::UnsupportedMint
        );
        require!(
            is_supported_staking_reward_mint(cfg, &payout_mint),
            SterlingError::UnsupportedMint
        );
        require!(
            is_enabled_staking_reward_mint(cfg, &stake_mint),
            SterlingError::MintDisabled
        );
        require!(
            is_enabled_staking_reward_mint(cfg, &payout_mint),
            SterlingError::MintDisabled
        );

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_stake_ata.to_account_info(),
                    to: ctx.accounts.stake_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;

        let pos = &mut ctx.accounts.position;
        if pos.owner == Pubkey::default() {
            pos.owner = ctx.accounts.user.key();
            pos.stake_mint = stake_mint;
            pos.payout_mint = payout_mint;
            pos.amount = 0;
            pos.last_claim_ts = Clock::get()?.unix_timestamp;
            pos.bump = ctx.bumps.position;
        } else {
            require!(
                pos.owner == ctx.accounts.user.key(),
                SterlingError::Unauthorized
            );
        }

        pos.amount = pos.amount.saturating_add(amount);
        Ok(())
    }

    pub fn claim<'info>(ctx: Context<'_, '_, 'info, 'info, Claim<'info>>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let pos = &mut ctx.accounts.position;
        let pool_id = ctx.accounts.config.pool_id;
        let user_key = ctx.accounts.user.key();
        let stake_mint_key = ctx.accounts.stake_mint.key();
        let payout_mint_key = ctx.accounts.payout_mint.key();
        let reward_interval = ctx.accounts.config.reward_interval;
        let token_value_usd_micros_default = ctx.accounts.config.token_value_usd_micros_default;
        let cashback_bps = effective_claim_cashback_bps(&ctx.accounts.config, stake_mint_key);
        let allow_fallback_usdt = ctx.accounts.config.allow_fallback_usdt;

        // Business rule: staking claims are valued from the staked position only.
        // The staked principal remains isolated in stake_vault and is never used as a
        // cash-settlement source. claim() can settle from payment rails, but never from
        // the staked principal itself.

        require!(
            pos.owner == ctx.accounts.user.key(),
            SterlingError::Unauthorized
        );
        require!(
            pos.stake_mint == stake_mint_key,
            SterlingError::InvalidAccount
        );
        require!(
            pos.payout_mint == payout_mint_key,
            SterlingError::InvalidAccount
        );
        require!(pos.amount > 0, SterlingError::InvalidAmount);
        require!(
            now >= pos.last_claim_ts + (reward_interval as i64),
            SterlingError::TooEarlyClaim
        );
        require!(
            is_supported_staking_reward_mint(&ctx.accounts.config, &stake_mint_key),
            SterlingError::UnsupportedMint
        );
        require!(
            is_supported_staking_reward_mint(&ctx.accounts.config, &payout_mint_key),
            SterlingError::UnsupportedMint
        );
        require!(
            is_enabled_staking_reward_mint(&ctx.accounts.config, &stake_mint_key),
            SterlingError::MintDisabled
        );
        require!(
            is_enabled_staking_reward_mint(&ctx.accounts.config, &payout_mint_key),
            SterlingError::MintDisabled
        );
        require!(
            ctx.accounts.reward_vault.owner == ctx.accounts.config.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.usdc_coffre_vault.owner == ctx.accounts.config.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.usdt_coffre_vault.owner == ctx.accounts.config.key(),
            SterlingError::InvalidAccount
        );

        // ATA checks
        require!(
            ctx.accounts.user_payout_ata.owner == ctx.accounts.user.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_payout_ata.mint == ctx.accounts.payout_mint.key(),
            SterlingError::InvalidAccount
        );

        require!(
            ctx.accounts.user_usdt_main_ata.owner == ctx.accounts.user.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_usdt_main_ata.mint == ctx.accounts.usdt_main_mint.key(),
            SterlingError::InvalidAccount
        );
        let expected_user_usdt_main_ata =
            get_associated_token_address(&user_key, &ctx.accounts.config.usdt_mint);
        require!(
            ctx.accounts.user_usdt_main_ata.key() == expected_user_usdt_main_ata,
            SterlingError::InvalidAccount
        );

        let claim_remaining_accounts = parse_claim_remaining_accounts(
            &ctx.accounts.config,
            stake_mint_key,
            payout_mint_key,
            user_key,
            ctx.accounts.user_usdt_main_ata.key(),
            ctx.remaining_accounts,
            token_value_usd_micros_default,
        )?;
        let stake_value_usd_micros_per_1ui =
            claim_remaining_accounts.stake_value_usd_micros_per_1ui;

        let position_amount = pos.amount;
        let previous_claim_ts = pos.last_claim_ts;

        let reward_usd_micros: u64 = compute_claim_reward_usd_micros(
            position_amount,
            ctx.accounts.stake_mint.decimals,
            stake_value_usd_micros_per_1ui,
            cashback_bps,
        )?;

        let reward_amount: u64 =
            usd_micros_to_atoms(reward_usd_micros, ctx.accounts.payout_mint.decimals)?;
        let reward_amount_usdt_main: u64 =
            usd_micros_to_atoms(reward_usd_micros, ctx.accounts.usdt_main_mint.decimals)?;
        let reward_amount_stable: u64 =
            usd_micros_to_atoms(reward_usd_micros, STABLE_CASH_DECIMALS)?;

        require!(
            reward_usd_micros > 0 && reward_amount > 0,
            SterlingError::ZeroReward
        );

        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[ctx.bumps.config]]];
        let mut paid_direct = false;
        let requested_direct_cash_rail = requested_claim_direct_cash_rail(
            payout_mint_key,
            ctx.accounts.config.usdc_mint,
            ctx.accounts.config.usdt_mint,
            ctx.accounts.config.extra_payout_mint_0,
            ctx.accounts.config.extra_payout_mint_1,
            ctx.accounts.config.extra_payout_mint_2,
            ctx.accounts.config.extra_payout_mint_3,
        );
        let configured_cash_rail_vault = claim_remaining_accounts.configured_cash_rail_vault;
        let configured_cash_rail_amount = claim_remaining_accounts.configured_cash_rail_amount;
        let fallback_usdt_vault = claim_remaining_accounts.fallback_usdt_vault;
        let fallback_usdt_vault_amount = claim_remaining_accounts.fallback_usdt_vault_amount;
        let fallback_usdt_available = allow_fallback_usdt
            && payout_mint_key != ctx.accounts.config.usdt_mint
            && fallback_usdt_vault.is_some()
            && ctx.accounts.user_usdt_main_ata.mint == ctx.accounts.config.usdt_mint;

        // Settlement order for staking claims:
        // 1. reward_vault for the requested payout_mint
        // 2. direct stable cash rail for the requested payout_mint only:
        //    - USDC -> usdc_coffre_vault -> user_payout_ata
        //    - USDT -> usdt_coffre_vault -> user_payout_ata
        // 3. canonical USDT fallback reward_vault -> user_usdt_main_ata when allowed
        // 4. otherwise NeedPayoutEvent
        let claim_settlement_action = choose_claim_settlement_action(
            ctx.accounts.reward_vault.amount,
            reward_amount,
            requested_direct_cash_rail,
            ctx.accounts.usdc_coffre_vault.amount,
            reward_amount_stable,
            ctx.accounts.usdt_coffre_vault.amount,
            reward_amount_stable,
            configured_cash_rail_amount,
            reward_amount,
            fallback_usdt_available,
            fallback_usdt_vault_amount,
            reward_amount_usdt_main,
        );

        match claim_settlement_action {
            ClaimSettlementAction::DirectRewardVault => {
                token::transfer(
                    CpiContext::new(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: ctx.accounts.reward_vault.to_account_info(),
                            to: ctx.accounts.user_payout_ata.to_account_info(),
                            authority: ctx.accounts.config.to_account_info(),
                        },
                    )
                    .with_signer(signer_seeds),
                    reward_amount,
                )?;
                paid_direct = true;
            }
            ClaimSettlementAction::FallbackUsdtVault => {
                let fallback_usdt_vault = fallback_usdt_vault
                    .clone()
                    .ok_or_else(|| error!(SterlingError::InvalidAccount))?;
                token::transfer(
                    CpiContext::new(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: fallback_usdt_vault,
                            to: ctx.accounts.user_usdt_main_ata.to_account_info(),
                            authority: ctx.accounts.config.to_account_info(),
                        },
                    )
                    .with_signer(signer_seeds),
                    reward_amount_usdt_main,
                )?;
                paid_direct = true;
            }
            ClaimSettlementAction::DirectRequestedCashRail => {
                let (from_account, transfer_amount) = match requested_direct_cash_rail {
                    ClaimDirectCashRail::Usdc => (
                        ctx.accounts.usdc_coffre_vault.to_account_info(),
                        reward_amount_stable,
                    ),
                    ClaimDirectCashRail::Usdt => (
                        ctx.accounts.usdt_coffre_vault.to_account_info(),
                        reward_amount_stable,
                    ),
                    ClaimDirectCashRail::ExtraConfigured => (
                        configured_cash_rail_vault
                            .clone()
                            .ok_or_else(|| error!(SterlingError::InvalidAccount))?,
                        reward_amount,
                    ),
                    ClaimDirectCashRail::None => return err!(SterlingError::InvalidAccount),
                };
                token::transfer(
                    CpiContext::new(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: from_account,
                            to: ctx.accounts.user_payout_ata.to_account_info(),
                            authority: ctx.accounts.config.to_account_info(),
                        },
                    )
                    .with_signer(signer_seeds),
                    transfer_amount,
                )?;
                paid_direct = true;
            }
            ClaimSettlementAction::EmitNeedPayoutEvent => {}
        }

        if !paid_direct {
            let nonce: u64 = u64::try_from(previous_claim_ts.max(0))
                .unwrap_or(0)
                .saturating_add(1);
            require!(reward_usd_micros > 0, SterlingError::InvalidAmount);
            enforce_ticket_cap_usd_micros(reward_usd_micros)?;
            require!(
                [
                    ctx.accounts.config.usdc_mint,
                    ctx.accounts.config.usdt_mint,
                    ctx.accounts.config.extra_payout_mint_0,
                    ctx.accounts.config.extra_payout_mint_1,
                    ctx.accounts.config.extra_payout_mint_2,
                    ctx.accounts.config.extra_payout_mint_3
                ]
                .contains(&payout_mint_key),
                SterlingError::UnsupportedPayoutMint
            );
            require!(
                ctx.accounts.config.payout_threshold_usd_micros == 0
                    || reward_usd_micros >= ctx.accounts.config.payout_threshold_usd_micros,
                SterlingError::InvalidAmount
            );
            apply_payout_limits(&mut ctx.accounts.config, reward_usd_micros)?;
            let destination_ata = get_associated_token_address(&user_key, &payout_mint_key);
            emit!(NeedPayoutEvent {
                pool: pool_id,
                payout_mint: payout_mint_key,
                payout_kind: 4,
                user: user_key,
                mint_in: stake_mint_key,
                amount_in: position_amount,
                usd_micros: reward_usd_micros,
                destination_ata,
                nonce,
                liquidity_policy: 1,
                route_hint: 0,
                ts: now,
            });
        }

        pos.last_claim_ts = now;

        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        require!(amount > 0, SterlingError::InvalidAmount);
        require!(
            ctx.accounts.position.owner == ctx.accounts.user.key(),
            SterlingError::Unauthorized
        );
        require!(
            ctx.accounts.position.stake_mint == ctx.accounts.stake_mint.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.position.payout_mint == ctx.accounts.payout_mint.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_stake_ata.owner == ctx.accounts.user.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_stake_ata.mint == ctx.accounts.stake_mint.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.stake_vault.mint == ctx.accounts.stake_mint.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.position.amount >= amount,
            SterlingError::InvalidAmount
        );

        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[ctx.bumps.config]]];
        transfer_tokens_signed(
            &ctx.accounts.token_program,
            ctx.accounts.stake_vault.to_account_info(),
            ctx.accounts.user_stake_ata.to_account_info(),
            ctx.accounts.config.to_account_info(),
            signer_seeds,
            amount,
        )?;

        let position = &mut ctx.accounts.position;
        position.amount = position.amount.saturating_sub(amount);
        if position.amount == 0 {
            position.last_claim_ts = Clock::get()?.unix_timestamp;
        }

        Ok(())
    }

    pub fn migrate_config_v1_to_v2(ctx: Context<MigrateConfigV1ToV2>) -> Result<()> {
        #[cfg(not(any(feature = "migration-full", feature = "migration-ledger")))]
        {
            let _ = ctx;
            return err!(SterlingError::InvalidState);
        }

        #[cfg(any(feature = "migration-full", feature = "migration-ledger"))]
        {
            let config_ai = ctx.accounts.config.to_account_info();
            let (expected_config, _) = Pubkey::find_program_address(&[b"config"], ctx.program_id);
            require!(
                config_ai.key() == expected_config,
                SterlingError::InvalidAccount
            );
            require!(
                config_ai.owner == ctx.program_id,
                SterlingError::InvalidAccount
            );
            require_admin_on_config_info(&config_ai, &ctx.accounts.admin)?;

            let original_len = config_ai.data_len();
            let already_v2 = {
                let data = config_ai.try_borrow_data()?;
                let mut bytes: &[u8] = &data;
                Config::try_deserialize(&mut bytes).is_ok()
            };
            if already_v2 && !config_needs_migration(original_len) {
                return Ok(());
            }

            let legacy = {
                let data = config_ai.try_borrow_data()?;
                deserialize_legacy_config_v1(&data)?
            };
            let (_, bump) = Pubkey::find_program_address(&[b"config"], ctx.program_id);
            realloc_program_account(
                &config_ai,
                &ctx.accounts.admin,
                &ctx.accounts.system_program,
                CONFIG_ACCOUNT_LEN,
            )?;

            let migrated = build_config_from_legacy_v1(&legacy, bump);
            let mut data = config_ai.try_borrow_mut_data()?;
            let mut out: &mut [u8] = &mut data;
            migrated.try_serialize(&mut out)?;
            Ok(())
        }
    }

    pub fn migrate_pool_v1_to_v2(ctx: Context<MigratePoolV1ToV2>) -> Result<()> {
        #[cfg(not(any(feature = "migration-full", feature = "migration-ledger")))]
        {
            let _ = ctx;
            return err!(SterlingError::InvalidState);
        }

        #[cfg(any(feature = "migration-full", feature = "migration-ledger"))]
        {
            let config_ai = ctx.accounts.config.to_account_info();
            let base_mint = ctx.accounts.base_mint.key();
            let quote_mint = ctx.accounts.quote_mint.key();
            let (expected_config, _) = Pubkey::find_program_address(&[b"config"], ctx.program_id);
            require!(
                config_ai.key() == expected_config,
                SterlingError::InvalidAccount
            );
            require!(
                config_ai.owner == ctx.program_id,
                SterlingError::InvalidAccount
            );
            require_admin_on_config_info(&config_ai, &ctx.accounts.admin)?;

            let pool_ai = ctx.accounts.pool.to_account_info();
            let (expected_pool, _) = Pubkey::find_program_address(
                &[b"pool", base_mint.as_ref(), quote_mint.as_ref()],
                ctx.program_id,
            );
            require!(
                pool_ai.key() == expected_pool,
                SterlingError::InvalidAccount
            );
            require!(
                pool_ai.owner == ctx.program_id,
                SterlingError::InvalidAccount
            );

            let original_len = pool_ai.data_len();
            let already_v2 = {
                let data = pool_ai.try_borrow_data()?;
                let mut bytes: &[u8] = &data;
                Pool::try_deserialize(&mut bytes).is_ok()
            };
            if already_v2 && !pool_needs_migration(original_len) {
                return Ok(());
            }

            let legacy = {
                let data = pool_ai.try_borrow_data()?;
                deserialize_legacy_pool_v1(&data)?
            };
            require!(legacy.base_mint == base_mint, SterlingError::InvalidAccount);
            require!(
                legacy.quote_mint == quote_mint,
                SterlingError::InvalidAccount
            );

            let (_, bump) = Pubkey::find_program_address(
                &[b"pool", base_mint.as_ref(), quote_mint.as_ref()],
                ctx.program_id,
            );
            realloc_program_account(
                &pool_ai,
                &ctx.accounts.admin,
                &ctx.accounts.system_program,
                Pool::LEN,
            )?;

            let migrated = build_pool_from_legacy_v1(&legacy, bump);
            let mut data = pool_ai.try_borrow_mut_data()?;
            let mut out: &mut [u8] = &mut data;
            migrated.try_serialize(&mut out)?;
            Ok(())
        }
    }

    pub fn migrate_payout_ticket_v1_to_v2(
        ctx: Context<MigratePayoutTicketV1ToV2>,
        _nonce: u64,
    ) -> Result<()> {
        #[cfg(not(feature = "migration-payout"))]
        {
            let _ = (ctx, _nonce);
            return err!(SterlingError::InvalidState);
        }

        #[cfg(feature = "migration-payout")]
        {
            crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
            let ticket_ai = ctx.accounts.ticket.to_account_info();

            if ticket_ai.data_len() >= PayoutTicket::LEN {
                return Ok(());
            }

            let migrated = {
                let data = ticket_ai.try_borrow_data()?;
                build_payout_ticket_from_legacy_v1(ticket_ai.key(), &data)?
            };

            realloc_program_account(
                &ticket_ai,
                &ctx.accounts.admin,
                &ctx.accounts.system_program,
                PayoutTicket::LEN,
            )?;

            let mut data = ticket_ai.try_borrow_mut_data()?;
            let mut out: &mut [u8] = &mut data;
            migrated.try_serialize(&mut out)?;
            Ok(())
        }
    }

    pub fn migrate_settlement_claim_v1_to_v2(
        ctx: Context<MigrateSettlementClaimV1ToV2>,
        _nonce: u64,
    ) -> Result<()> {
        #[cfg(not(feature = "migration-claim"))]
        {
            let _ = (ctx, _nonce);
            return err!(SterlingError::InvalidState);
        }

        #[cfg(feature = "migration-claim")]
        {
            crate::helpers::require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
            let claim_ai = ctx.accounts.claim.to_account_info();

            if claim_ai.data_len() >= SettlementClaim::LEN {
                return Ok(());
            }

            let migrated = {
                let data = claim_ai.try_borrow_data()?;
                build_settlement_claim_from_legacy_v1(claim_ai.key(), &data)?
            };

            realloc_program_account(
                &claim_ai,
                &ctx.accounts.admin,
                &ctx.accounts.system_program,
                SettlementClaim::LEN,
            )?;

            let mut data = claim_ai.try_borrow_mut_data()?;
            let mut out: &mut [u8] = &mut data;
            migrated.try_serialize(&mut out)?;
            Ok(())
        }
    }

    pub fn migrate_protocol_debt_ledger_v1_to_v2(
        ctx: Context<MigrateProtocolDebtLedgerV1ToV2>,
    ) -> Result<()> {
        #[cfg(not(any(feature = "migration-full", feature = "migration-ledger")))]
        {
            let _ = ctx;
            return err!(SterlingError::InvalidState);
        }

        #[cfg(any(feature = "migration-full", feature = "migration-ledger"))]
        {
            let config_ai = ctx.accounts.config.to_account_info();
            let (expected_config, _) = Pubkey::find_program_address(&[b"config"], ctx.program_id);
            require!(
                config_ai.key() == expected_config,
                SterlingError::InvalidAccount
            );
            require!(
                config_ai.owner == ctx.program_id,
                SterlingError::InvalidAccount
            );
            require_admin_on_config_info(&config_ai, &ctx.accounts.admin)?;

            let ledger_ai = ctx.accounts.protocol_debt_ledger.to_account_info();
            let (expected_ledger, _) = Pubkey::find_program_address(
                &[b"protocol_debt", ctx.accounts.pool.key().as_ref()],
                ctx.program_id,
            );
            require!(
                ledger_ai.key() == expected_ledger,
                SterlingError::InvalidAccount
            );
            require!(
                ledger_ai.owner == ctx.program_id,
                SterlingError::InvalidAccount
            );

            let original_len = ledger_ai.data_len();
            let already_v2 = {
                let data = ledger_ai.try_borrow_data()?;
                data.len() >= ProtocolDebtLedger::LEN
                    && data[..8]
                        == <ProtocolDebtLedger as anchor_lang::Discriminator>::discriminator()
            };
            if already_v2 && !protocol_debt_ledger_needs_migration(original_len) {
                return Ok(());
            }

            let legacy_data = {
                let data = ledger_ai.try_borrow_data()?;
                require!(
                    data.len() >= LEGACY_PROTOCOL_DEBT_LEDGER_V1_MIN_LEN,
                    SterlingError::InvalidAccount
                );
                data.to_vec()
            };
            require!(
                legacy_pk(&legacy_data[8..], 0) == ctx.accounts.pool.key(),
                SterlingError::InvalidAccount
            );

            realloc_program_account(
                &ledger_ai,
                &ctx.accounts.admin,
                &ctx.accounts.system_program,
                ProtocolDebtLedger::LEN,
            )?;

            let mut data = ledger_ai.try_borrow_mut_data()?;
            migrate_protocol_debt_ledger_data_from_legacy_v1(
                &mut data,
                &legacy_data,
                ctx.accounts.pool.key(),
            )?;
            Ok(())
        }
    }
}

#[inline(never)]
fn sovereign_convert_output_v1_impl<'info>(
    ctx: Context<'_, '_, '_, 'info, SovereignConvertOutputV1Ctx<'info>>,
    amount_in: u64,
    source_kind: u8,
    final_output_kind: u8,
    debt_lot_nonce: u64,
    bank_metadata: String,
) -> Result<()> {
    let _ = bank_metadata;
    let requested_output_kind = sovereign_final_output_kind_from_u8(final_output_kind)?;

    if matches!(source_kind, 1 | 2 | 3) {
        require!(amount_in > 0, SterlingError::InvalidAmount);
    }

    match source_kind {
        1 => route_sovereign_valued_output(ctx, amount_in, 1, requested_output_kind, true),
        2 => route_sovereign_direct_output(ctx, amount_in, requested_output_kind),
        3 => route_sovereign_valued_output(ctx, amount_in, 3, requested_output_kind, false),
        4 => route_sovereign_receipt_output(ctx, 4, requested_output_kind),
        5 => route_sovereign_receipt_output(ctx, 5, requested_output_kind),
        6 => route_sovereign_debt_output(ctx, requested_output_kind, debt_lot_nonce),
        _ => err!(SterlingError::InvalidAccount),
    }
}

#[inline(never)]
fn build_sovereign_route_targets<'info>(
    config: &Account<'info, Config>,
    route_accounts: &SovereignConvertRouteAccounts<'info>,
    requested_output_kind: SovereignFinalOutputKind,
) -> Result<(
    Option<StableFinalOutputTarget<'info>>,
    Option<StableFinalOutputTarget<'info>>,
    Option<NativeSolOutputTarget<'info>>,
)> {
    let remaining = SovereignRemainingAccounts {
        source_primary: route_accounts.source_primary.clone(),
        source_secondary: route_accounts.source_secondary.clone(),
        alt_vault: route_accounts.alt_vault.clone(),
        alt_destination: route_accounts.alt_destination.clone(),
    };
    let usdc_target = build_usdc_output_target(
        config,
        route_accounts.usdc_vault.clone(),
        route_accounts.usdc_destination.clone(),
    )?;
    let usdt_target = build_optional_alt_output_target(config, &remaining)?;
    let (primary_target, secondary_target) =
        select_stable_output_targets(requested_output_kind, usdc_target, usdt_target);
    let native_target =
        build_optional_native_sol_output_target(config, route_accounts.sol_destination.clone())?;
    Ok((primary_target, secondary_target, native_target))
}

#[inline(never)]
fn route_sovereign_valued_output<'info>(
    ctx: Context<'_, '_, '_, 'info, SovereignConvertOutputV1Ctx<'info>>,
    amount_in: u64,
    source_kind: u8,
    requested_output_kind: SovereignFinalOutputKind,
    require_non_lp: bool,
) -> Result<()> {
    match choose_sovereign_route_execution_kind(
        source_kind,
        should_prefer_live_sovereign_route(source_kind, ctx.remaining_accounts),
    )? {
        SovereignRouteExecutionKind::LiveSwap => {
            let live_route =
                parse_sovereign_live_swap_accounts(source_kind, ctx.remaining_accounts)?;
            settle_live_pool_exchange(
                &ctx.accounts.config,
                &ctx.accounts.user,
                &ctx.accounts.token_program,
                source_kind,
                amount_in,
                requested_output_kind,
                require_non_lp,
                live_route,
            )
        }
        SovereignRouteExecutionKind::BurnRelease => {
            let route_accounts =
                parse_sovereign_convert_route_accounts(source_kind, ctx.remaining_accounts)?;
            let (primary_target, secondary_target, native_target) = build_sovereign_route_targets(
                &ctx.accounts.config,
                &route_accounts,
                requested_output_kind,
            )?;

            if require_non_lp {
                settle_redeem_to_stable_output(
                    &ctx.accounts.config,
                    &ctx.accounts.user,
                    route_accounts
                        .token_mint
                        .clone()
                        .ok_or_else(|| error!(SterlingError::InvalidSovereignAccountsLayout))?,
                    route_accounts
                        .user_token_ata
                        .clone()
                        .ok_or_else(|| error!(SterlingError::InvalidSovereignAccountsLayout))?,
                    route_accounts
                        .asset_registry
                        .clone()
                        .ok_or_else(|| error!(SterlingError::InvalidSovereignAccountsLayout))?,
                    &ctx.accounts.token_program,
                    amount_in,
                    source_kind,
                    ctx.accounts.config.bump,
                    primary_target,
                    secondary_target,
                    native_target,
                )
            } else {
                settle_valued_asset_output(
                    &ctx.accounts.config,
                    &ctx.accounts.user,
                    route_accounts
                        .token_mint
                        .clone()
                        .ok_or_else(|| error!(SterlingError::InvalidSovereignAccountsLayout))?,
                    route_accounts
                        .user_token_ata
                        .clone()
                        .ok_or_else(|| error!(SterlingError::InvalidSovereignAccountsLayout))?,
                    route_accounts
                        .asset_registry
                        .clone()
                        .ok_or_else(|| error!(SterlingError::InvalidSovereignAccountsLayout))?,
                    &ctx.accounts.token_program,
                    amount_in,
                    source_kind,
                    ctx.accounts.config.bump,
                    false,
                    primary_target,
                    secondary_target,
                    native_target,
                )
            }
        }
        _ => err!(SterlingError::InvalidAccount),
    }
}

#[inline(never)]
fn route_sovereign_direct_output<'info>(
    ctx: Context<'_, '_, '_, 'info, SovereignConvertOutputV1Ctx<'info>>,
    amount_in: u64,
    requested_output_kind: SovereignFinalOutputKind,
) -> Result<()> {
    match choose_sovereign_route_execution_kind(
        2,
        should_prefer_live_sovereign_route(2, ctx.remaining_accounts),
    )? {
        SovereignRouteExecutionKind::LiveSwap => {
            let live_route = parse_sovereign_live_swap_accounts(2, ctx.remaining_accounts)?;
            settle_live_pool_exchange(
                &ctx.accounts.config,
                &ctx.accounts.user,
                &ctx.accounts.token_program,
                2,
                amount_in,
                requested_output_kind,
                false,
                live_route,
            )
        }
        SovereignRouteExecutionKind::DirectTransfer => {
            let route_accounts = parse_sovereign_convert_route_accounts(2, ctx.remaining_accounts)?;
            let (primary_target, secondary_target, native_target) = build_sovereign_route_targets(
                &ctx.accounts.config,
                &route_accounts,
                requested_output_kind,
            )?;

            settle_direct_stable_output(
                &ctx.accounts.config,
                &ctx.accounts.user,
                route_accounts
                    .token_mint
                    .clone()
                    .ok_or_else(|| error!(SterlingError::InvalidSovereignAccountsLayout))?,
                route_accounts
                    .user_token_ata
                    .clone()
                    .ok_or_else(|| error!(SterlingError::InvalidSovereignAccountsLayout))?,
                &ctx.accounts.token_program,
                amount_in,
                2,
                primary_target,
                secondary_target,
                native_target,
            )
        }
        _ => err!(SterlingError::InvalidAccount),
    }
}

#[inline(never)]
fn route_sovereign_receipt_output<'info>(
    ctx: Context<'_, '_, '_, 'info, SovereignConvertOutputV1Ctx<'info>>,
    source_kind: u8,
    requested_output_kind: SovereignFinalOutputKind,
) -> Result<()> {
    match choose_sovereign_route_execution_kind(
        source_kind,
        should_prefer_live_sovereign_route(source_kind, ctx.remaining_accounts),
    )? {
        SovereignRouteExecutionKind::LiveSwap => {
            let live_route =
                parse_sovereign_escrow_live_swap_accounts(source_kind, ctx.remaining_accounts)?;

            match source_kind {
                4 => settle_ticket_live_output(
                    &ctx.accounts.config,
                    &ctx.accounts.user,
                    &ctx.accounts.token_program,
                    requested_output_kind,
                    live_route,
                ),
                5 => settle_claim_live_output(
                    &ctx.accounts.config,
                    &ctx.accounts.user,
                    &ctx.accounts.token_program,
                    requested_output_kind,
                    live_route,
                ),
                _ => err!(SterlingError::InvalidAccount),
            }
        }
        SovereignRouteExecutionKind::BufferedReceipt => {
            let route_accounts =
                parse_sovereign_convert_route_accounts(source_kind, ctx.remaining_accounts)?;
            let buffered_escrow = require_buffered_route_escrow_accounts(&route_accounts)?;
            let (primary_target, secondary_target, native_target) = build_sovereign_route_targets(
                &ctx.accounts.config,
                &route_accounts,
                requested_output_kind,
            )?;

            match source_kind {
                4 => settle_ticket_to_stable(
                    ctx.accounts.config.to_account_info(),
                    ctx.accounts.user.key(),
                    ctx.accounts.token_program.to_account_info(),
                    route_accounts
                        .source_primary
                        .clone()
                        .ok_or_else(|| error!(SterlingError::InvalidSovereignAccountsLayout))?,
                    buffered_escrow.clone(),
                    ctx.accounts.config.bump,
                    primary_target,
                    secondary_target,
                    native_target,
                ),
                5 => settle_claim_to_stable(
                    ctx.accounts.config.to_account_info(),
                    ctx.accounts.user.key(),
                    ctx.accounts.token_program.to_account_info(),
                    route_accounts
                        .source_primary
                        .clone()
                        .ok_or_else(|| error!(SterlingError::InvalidSovereignAccountsLayout))?,
                    buffered_escrow,
                    ctx.accounts.config.bump,
                    primary_target,
                    secondary_target,
                    native_target,
                ),
                _ => err!(SterlingError::InvalidAccount),
            }
        }
        _ => err!(SterlingError::InvalidAccount),
    }
}

#[inline(never)]
fn route_sovereign_debt_output<'info>(
    ctx: Context<'_, '_, '_, 'info, SovereignConvertOutputV1Ctx<'info>>,
    requested_output_kind: SovereignFinalOutputKind,
    debt_lot_nonce: u64,
) -> Result<()> {
    match choose_sovereign_route_execution_kind(
        6,
        should_prefer_live_sovereign_route(6, ctx.remaining_accounts),
    )? {
        SovereignRouteExecutionKind::LiveSwap => {
            let live_route = parse_sovereign_escrow_live_swap_accounts(6, ctx.remaining_accounts)?;
            settle_protocol_debt_live_output(
                &ctx.accounts.config,
                &ctx.accounts.user,
                &ctx.accounts.token_program,
                requested_output_kind,
                debt_lot_nonce,
                live_route,
            )
        }
        SovereignRouteExecutionKind::BufferedDebt => {
            let route_accounts = parse_sovereign_convert_route_accounts(6, ctx.remaining_accounts)?;
            route_sovereign_debt_output_from_accounts(
                &ctx.accounts.config,
                ctx.accounts.config.main_wallet,
                ctx.accounts.config.usdc_mint,
                ctx.accounts.config.usdt_mint,
                ctx.accounts.config.extra_payout_mint_0,
                ctx.accounts.config.extra_payout_mint_1,
                ctx.accounts.config.extra_payout_mint_2,
                ctx.accounts.config.extra_payout_mint_3,
                ctx.accounts.config.treasury_usdc_ata,
                ctx.accounts.config.treasury_usdt_ata,
                ctx.accounts.user.key(),
                ctx.accounts.token_program.to_account_info(),
                &route_accounts,
                requested_output_kind,
                debt_lot_nonce,
                ctx.accounts.config.bump,
            )
        }
        _ => err!(SterlingError::InvalidAccount),
    }
}

#[inline(never)]
fn route_sovereign_debt_output_from_accounts<'info>(
    config: &Account<'info, Config>,
    main_wallet: Pubkey,
    canonical_usdc_mint: Pubkey,
    canonical_usdt_mint: Pubkey,
    extra_payout_mint_0: Pubkey,
    extra_payout_mint_1: Pubkey,
    extra_payout_mint_2: Pubkey,
    extra_payout_mint_3: Pubkey,
    treasury_usdc_ata: Pubkey,
    treasury_usdt_ata: Pubkey,
    user_key: Pubkey,
    token_program_info: AccountInfo<'info>,
    route_accounts: &SovereignConvertRouteAccounts<'info>,
    requested_output_kind: SovereignFinalOutputKind,
    debt_lot_nonce: u64,
    config_bump: u8,
) -> Result<()> {
    let (primary_target, secondary_target, native_target) =
        build_sovereign_route_targets(config, &route_accounts, requested_output_kind)?;

    let payout_policy = SovereignDebtPayoutPolicy {
        main_wallet,
        canonical_usdc_mint,
        canonical_usdt_mint,
        extra_payout_mint_0,
        extra_payout_mint_1,
        extra_payout_mint_2,
        extra_payout_mint_3,
        treasury_usdc_ata,
        treasury_usdt_ata,
    };
    let settlement_targets = SovereignBufferedSettlementTargets {
        primary_target,
        secondary_target,
        native_target,
    };
    let buffered_escrow = require_buffered_route_escrow_accounts(route_accounts)?;

    settle_protocol_debt_to_stable(
        config.to_account_info(),
        user_key,
        token_program_info,
        route_accounts
            .source_primary
            .clone()
            .ok_or_else(|| error!(SterlingError::StableSettlementAccountsMissing))?,
        route_accounts
            .source_secondary
            .clone()
            .ok_or_else(|| error!(SterlingError::StableSettlementAccountsMissing))?,
        debt_lot_nonce,
        buffered_escrow,
        config_bump,
        &payout_policy,
        &settlement_targets,
    )
}

// =========================
// ACCOUNTS
// =========================

// =========================
// PAO V3 ACCOUNTS — SAFE (no ABI break)
// =========================
#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(
        init,
        payer = admin,
        seeds = [b"config"],
        bump,
        // ✅ un peu plus large pour éviter erreur de place si on ajoute 1 bool
        space = 8 + 1400,
    )]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AdminOnly<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct ReserveAuthorityRebind<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    pub admin: Signer<'info>,
    /// CHECK: mint key only, used for PDA derivation
    pub authority_mint: UncheckedAccount<'info>,
    /// CHECK: bridge_vault PDA or config PDA signer, validated on demand
    pub current_authority_bridge_vault: UncheckedAccount<'info>,
    /// CHECK: reserve token account, validated on demand
    #[account(mut)]
    pub reserve_token_account: UncheckedAccount<'info>,
    /// CHECK: authority pubkey only
    pub new_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct TreasurySweepReserveCompat<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    pub admin: Signer<'info>,
    /// CHECK: reserve token account, validated on demand
    #[account(mut)]
    pub reserve_token_account: UncheckedAccount<'info>,
    /// CHECK: destination token account, validated on demand
    #[account(mut)]
    pub destination_token_account: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BackfillPoolRegistryEntry<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Box<Account<'info, Pool>>,
    #[account(
            init_if_needed,
            payer = admin,
            seeds = [b"pool_registry", pool.key().as_ref()],
            bump,
            space = PoolRegistryEntry::LEN
        )]
    pub pool_registry_entry: Box<Account<'info, PoolRegistryEntry>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MigrateConfigV1ToV2<'info> {
    /// CHECK: legacy/full config, parsed manually for migration compatibility
    #[account(mut)]
    pub config: UncheckedAccount<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MigratePoolV1ToV2<'info> {
    /// CHECK: legacy/full config, parsed manually for admin compatibility
    #[account(mut)]
    pub config: UncheckedAccount<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK: mint key only, used for PDA derivation and legacy validation
    pub base_mint: UncheckedAccount<'info>,
    /// CHECK: mint key only, used for PDA derivation and legacy validation
    pub quote_mint: UncheckedAccount<'info>,
    /// CHECK: legacy/full pool, parsed manually for migration compatibility
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(nonce: u64)]
pub struct MigratePayoutTicketV1ToV2<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK: key only, used for PDA constraint
    pub user: UncheckedAccount<'info>,
    /// CHECK: key only, used for PDA constraint
    pub pool: UncheckedAccount<'info>,
    /// CHECK: legacy/full payout ticket, parsed manually for migration compatibility
    #[account(
        mut,
        seeds = [b"payout", user.key().as_ref(), pool.key().as_ref(), &nonce.to_le_bytes()],
        bump,
        owner = crate::ID
    )]
    pub ticket: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(nonce: u64)]
pub struct MigrateSettlementClaimV1ToV2<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK: key only, used for PDA constraint
    pub user: UncheckedAccount<'info>,
    /// CHECK: key only, used for PDA constraint
    pub pool: UncheckedAccount<'info>,
    /// CHECK: legacy/full settlement claim, parsed manually for migration compatibility
    #[account(
        mut,
        seeds = [b"claim", user.key().as_ref(), pool.key().as_ref(), &nonce.to_le_bytes()],
        bump,
        owner = crate::ID
    )]
    pub claim: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MigrateProtocolDebtLedgerV1ToV2<'info> {
    /// CHECK: legacy/full config, parsed manually for admin compatibility
    #[account(mut)]
    pub config: UncheckedAccount<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK: key only, used for PDA derivation
    pub pool: UncheckedAccount<'info>,
    /// CHECK: legacy/full protocol debt ledger, parsed manually for migration compatibility
    #[account(mut)]
    pub protocol_debt_ledger: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

// ---------- ValueRegistry ----------
#[derive(Accounts)]
pub struct InitValueRegistry<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub admin: Signer<'info>,

    pub mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = admin,
        seeds = [b"value_registry", mint.key().as_ref()],
        bump,
        space = 8 + 96
    )]
    pub value_registry: Box<Account<'info, ValueRegistry>>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetTokenValue<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    pub admin: Signer<'info>,

    pub mint: Box<Account<'info, Mint>>,

    #[account(mut, seeds = [b"value_registry", mint.key().as_ref()], bump)]
    pub value_registry: Box<Account<'info, ValueRegistry>>,
}

// ---------- Stake / Reward vaults ----------
#[derive(Accounts)]
pub struct InitStakeVault<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = admin,
        seeds = [b"stake_vault", mint.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = config
    )]
    pub stake_vault: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitRewardVault<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = admin,
        seeds = [b"reward_vault", mint.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = config
    )]
    pub reward_vault: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub stake_mint: Box<Account<'info, Mint>>,
    pub payout_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub user_stake_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut, seeds = [b"stake_vault", stake_mint.key().as_ref()], bump)]
    pub stake_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = user,
        seeds = [b"stake_pos", user.key().as_ref(), stake_mint.key().as_ref(), payout_mint.key().as_ref()],
        bump,
        space = 8 + 128
    )]
    pub position: Box<Account<'info, StakePosition>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub stake_mint: Box<Account<'info, Mint>>,
    pub payout_mint: Box<Account<'info, Mint>>,
    #[account(mut, seeds = [b"stake_pos", user.key().as_ref(), stake_mint.key().as_ref(), payout_mint.key().as_ref()], bump)]
    pub position: Box<Account<'info, StakePosition>>,
    #[account(mut, seeds = [b"reward_vault", payout_mint.key().as_ref()], bump)]
    pub reward_vault: Box<Account<'info, TokenAccount>>,
    // ATA for the requested payout_mint. Direct stable cash rails always settle to
    // this ATA when payout_mint is the requested canonical stable mint.
    #[account(mut)]
    pub user_payout_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        address = config.usdc_coffre @ SterlingError::InvalidAccount,
        constraint = usdc_coffre_vault.mint == config.usdc_mint @ SterlingError::InvalidAccount,
    )]
    pub usdc_coffre_vault: Box<Account<'info, TokenAccount>>,

    #[account(address = config.usdt_mint @ SterlingError::InvalidAccount)]
    pub usdt_main_mint: Box<Account<'info, Mint>>,
    // Canonical fallback destination when payout_mint is not USDT and fallback is allowed.
    #[account(mut)]
    pub user_usdt_main_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        address = config.usdt_coffre @ SterlingError::InvalidAccount,
        constraint = usdt_coffre_vault.mint == config.usdt_mint @ SterlingError::InvalidAccount,
    )]
    pub usdt_coffre_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub stake_mint: Box<Account<'info, Mint>>,
    pub payout_mint: Box<Account<'info, Mint>>,
    #[account(mut, seeds = [b"stake_pos", user.key().as_ref(), stake_mint.key().as_ref(), payout_mint.key().as_ref()], bump)]
    pub position: Box<Account<'info, StakePosition>>,
    #[account(mut)]
    pub user_stake_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut, seeds = [b"stake_vault", stake_mint.key().as_ref()], bump)]
    pub stake_vault: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}

// ---------- DEX Contexts ----------
#[derive(Accounts)]
pub struct CreatePool<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub admin: Signer<'info>,

    pub base_mint: Box<Account<'info, Mint>>,
    pub quote_mint: Box<Account<'info, Mint>>,

    #[account(seeds = [b"value_registry", base_mint.key().as_ref()], bump)]
    pub base_value_registry: Box<Account<'info, ValueRegistry>>,
    #[account(seeds = [b"value_registry", quote_mint.key().as_ref()], bump)]
    pub quote_value_registry: Box<Account<'info, ValueRegistry>>,

    #[account(
        init,
        payer = admin,
        seeds = [b"pool", base_mint.key().as_ref(), quote_mint.key().as_ref()],
        bump,
        space = Pool::LEN,
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        init,
        payer = admin,
        seeds = [b"pool_registry", pool.key().as_ref()],
        bump,
        space = PoolRegistryEntry::LEN,
    )]
    pub pool_registry_entry: Box<Account<'info, PoolRegistryEntry>>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitPoolBaseVault<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Box<Account<'info, Pool>>,
    #[account(mut, seeds = [b"pool_registry", pool.key().as_ref()], bump)]
    pub pool_registry_entry: Box<Account<'info, PoolRegistryEntry>>,
    pub base_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = admin,
        seeds = [b"pool_vault", pool.key().as_ref(), b"base"],
        bump,
        token::mint = base_mint,
        token::authority = config
    )]
    pub base_vault: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitPoolQuoteVault<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Box<Account<'info, Pool>>,
    #[account(mut, seeds = [b"pool_registry", pool.key().as_ref()], bump)]
    pub pool_registry_entry: Box<Account<'info, PoolRegistryEntry>>,
    pub quote_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = admin,
        seeds = [b"pool_vault", pool.key().as_ref(), b"quote"],
        bump,
        token::mint = quote_mint,
        token::authority = config
    )]
    pub quote_vault: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitPoolLpMint<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Box<Account<'info, Pool>>,
    #[account(mut, seeds = [b"pool_registry", pool.key().as_ref()], bump)]
    pub pool_registry_entry: Box<Account<'info, PoolRegistryEntry>>,
    #[account(
        init,
        payer = admin,
        seeds = [b"lp_mint", pool.key().as_ref()],
        bump,
        mint::decimals = 9,
        mint::authority = config
    )]
    pub lp_mint: Box<Account<'info, Mint>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitPoolFeeVaultBase<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Box<Account<'info, Pool>>,
    #[account(mut, seeds = [b"pool_registry", pool.key().as_ref()], bump)]
    pub pool_registry_entry: Box<Account<'info, PoolRegistryEntry>>,
    pub base_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = admin,
        seeds = [b"fee_vault", pool.key().as_ref(), b"base"],
        bump,
        token::mint = base_mint,
        token::authority = config
    )]
    pub fee_vault_base: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitPoolFeeVaultQuote<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Box<Account<'info, Pool>>,
    #[account(mut, seeds = [b"pool_registry", pool.key().as_ref()], bump)]
    pub pool_registry_entry: Box<Account<'info, PoolRegistryEntry>>,
    pub quote_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = admin,
        seeds = [b"fee_vault", pool.key().as_ref(), b"quote"],
        bump,
        token::mint = quote_mint,
        token::authority = config
    )]
    pub fee_vault_quote: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Box<Account<'info, Pool>>,
    #[account(mut)]
    pub user_base_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub user_quote_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub user_lp_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub base_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub quote_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub lp_mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Box<Account<'info, Pool>>,
    #[account(mut)]
    pub user_base_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub user_quote_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub user_lp_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub base_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub quote_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub lp_mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SwapBaseForQuote<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Box<Account<'info, Pool>>,
    #[account(mut)]
    pub user_base_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub user_quote_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub base_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub quote_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub fee_vault_base: Box<Account<'info, TokenAccount>>,
    pub base_mint: Box<Account<'info, Mint>>,
    #[account(seeds = [b"value_registry", pool.base_mint.as_ref()], bump)]
    pub base_value_registry: Box<Account<'info, ValueRegistry>>,
    #[account(seeds = [b"asset", pool.base_mint.as_ref()], bump)]
    pub base_asset_registry: Box<Account<'info, AssetRegistry>>,
    #[account(mut)]
    pub usdc_coffre_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub treasury_usdc_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub usdt_coffre_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub treasury_usdt_ata: Box<Account<'info, TokenAccount>>,
    /// CHECK: lazily parsed only when fee settlement needs the debt rail
    #[account(mut, seeds = [b"protocol_debt", pool.key().as_ref()], bump)]
    pub protocol_debt_ledger: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SwapQuoteForBase<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Box<Account<'info, Pool>>,
    #[account(mut)]
    pub user_quote_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub user_base_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub quote_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub base_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub fee_vault_quote: Box<Account<'info, TokenAccount>>,
    pub quote_mint: Box<Account<'info, Mint>>,
    #[account(seeds = [b"value_registry", pool.quote_mint.as_ref()], bump)]
    pub quote_value_registry: Box<Account<'info, ValueRegistry>>,
    #[account(seeds = [b"asset", pool.quote_mint.as_ref()], bump)]
    pub quote_asset_registry: Box<Account<'info, AssetRegistry>>,
    #[account(mut)]
    pub usdc_coffre_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub treasury_usdc_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub usdt_coffre_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub treasury_usdt_ata: Box<Account<'info, TokenAccount>>,
    /// CHECK: lazily parsed only when fee settlement needs the debt rail
    #[account(mut, seeds = [b"protocol_debt", pool.key().as_ref()], bump)]
    pub protocol_debt_ledger: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

// ---------- FEES collect stable ----------
#[derive(Accounts)]
pub struct CollectFeesStableToTreasury<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    pub admin: Signer<'info>,

    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Box<Account<'info, Pool>>,

    #[account(mut)]
    pub fee_vault_base: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub fee_vault_quote: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub treasury_usdc_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub treasury_usdt_ata: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

// ---------- FEES convert to USDC ----------
#[derive(Accounts)]
pub struct ConvertFeesToUsdc<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    pub admin: Signer<'info>,

    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Box<Account<'info, Pool>>,

    #[account(mut)]
    pub fee_vault_base: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub fee_vault_quote: Box<Account<'info, TokenAccount>>,

    pub fee_mint: Box<Account<'info, Mint>>,

    #[account(seeds = [b"asset", fee_mint.key().as_ref()], bump)]
    pub asset_registry: Box<Account<'info, AssetRegistry>>,

    #[account(mut)]
    pub usdc_coffre_ata: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub treasury_usdc_ata: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

// ---------- Asset registry + Redeem ----------
#[derive(Accounts)]
pub struct RegisterAsset<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub token_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = admin,
        seeds = [b"asset", token_mint.key().as_ref()],
        bump,
        space = 8 + 80
    )]
    pub asset_registry: Box<Account<'info, AssetRegistry>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SovereignRedeemToUsdc<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    pub user: Signer<'info>,

    /// CHECK: validated and deserialized on demand by the sovereign router
    pub token_mint: UncheckedAccount<'info>,
    /// CHECK: validated and deserialized on demand for source_kind 3
    pub asset_registry: UncheckedAccount<'info>,

    /// CHECK: validated and deserialized on demand by the sovereign router
    #[account(mut)]
    pub user_token_ata: UncheckedAccount<'info>,

    /// CHECK: validated and deserialized on demand by the sovereign router
    #[account(mut)]
    pub usdc_coffre_ata: UncheckedAccount<'info>,

    /// CHECK: validated and deserialized on demand by the sovereign router
    #[account(mut)]
    pub destination_usdc_ata: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SovereignConvertOutputV1Ctx<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

// =========================
// STATE
// =========================
#[account]
pub struct Config {
    pub admin: Pubkey,

    pub true_cash: bool,
    pub cash_backed: bool,
    pub real_peg: bool,
    pub sovereign: bool,

    pub cashback_bps: u16,
    pub reward_interval: u64,
    pub allow_fallback_usdt: bool,
    pub native_sol_enabled: bool,
    pub native_sol_usd_micros_per_sol: u64,
    pub native_sol_min_reserve_lamports: u64,

    pub token_value_usd_micros_default: u64,
    pub treasury_value_usd_micros: u64,

    pub enable_treasury: bool,
    pub enable_sjbc: bool,
    pub enable_sjbc2: bool,
    pub enable_sjbc3: bool,

    // ✅ ajout
    pub enable_usdc: bool,

    pub enable_usdt_main: bool,
    pub enable_usdt_old: bool,
    pub enable_btc_portal: bool,

    pub main_wallet: Pubkey,
    pub okx_wallet: Pubkey,

    pub lp_mint2: Pubkey,
    pub lp_auth2: Pubkey,
    pub lp_mint3: Pubkey,
    pub lp_auth3: Pubkey,

    pub mint_h1: Pubkey,
    pub mint_fr15: Pubkey,
    pub mint_db8: Pubkey,

    pub usdc_coffre: Pubkey,
    pub usdt_coffre: Pubkey,

    pub pda_gt: Pubkey,
    pub coffre_7q: Pubkey,

    pub pool_id: Pubkey,

    pub treasury_usdc_ata: Pubkey,
    pub treasury_usdt_ata: Pubkey,

    pub auto_collect_every_swaps: u64,
    pub fee_threshold_usd_micros: u64,

    // =========================
    // PAO V3 (payout architecture) — SAFE (no ABI break)
    // =========================
    pub keeper_authority: Pubkey,

    // canonical stable mints (for payouts)
    pub usdc_mint: Pubkey,
    pub usdt_mint: Pubkey,

    // optional extra payout mints (free choice)
    pub extra_payout_mint_0: Pubkey,
    pub extra_payout_mint_1: Pubkey,
    pub extra_payout_mint_2: Pubkey,
    pub extra_payout_mint_3: Pubkey,

    // treasury vaults for optional extra payout rails
    pub extra_payout_vault_ata_0: Pubkey,
    pub extra_payout_vault_ata_1: Pubkey,
    pub extra_payout_vault_ata_2: Pubkey,
    pub extra_payout_vault_ata_3: Pubkey,

    // threshold (anti-dust) in USD micros
    // 0 = no threshold, but emitted payout usd_micros is always >= 1.
    pub payout_threshold_usd_micros: u64,

    // anti-drain limits (USD micros)
    // ✅ 0 = unlimited (your requirement)
    pub max_payout_usd_micros: u64, // max per payout request (one event)
    pub max_payout_per_window_usd_micros: u64, // max per time window
    pub payout_window_secs: u64,    // short window
    pub payout_window_start: i64,
    pub payout_window_used_usd_micros: u64,

    // cashback knobs (bps)
    pub lp_cashback_bps: u16,
    pub claim_cashback_bps: u16,

    pub bump: u8,
}

#[account]
pub struct StakePosition {
    pub owner: Pubkey,
    pub stake_mint: Pubkey,
    pub payout_mint: Pubkey,
    pub amount: u64,
    pub last_claim_ts: i64,
    pub bump: u8,
}

#[account]
pub struct Pool {
    pub owner: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,

    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub lp_mint: Pubkey,

    pub fee_vault_base: Pubkey,
    pub fee_vault_quote: Pubkey,

    pub base_value_usd_micros: u64,
    pub quote_value_usd_micros: u64,

    pub true_cash: bool,
    pub cash_backed: bool,
    pub real_peg: bool,
    pub sovereign: bool,

    pub fee_bps: u16,
    pub active: bool,
    pub swap_cashback_bps: u16,

    // Dette protocole agrégée ouverte sur le pool.
    // `protocol_fee_debt_count` = nombre logique total de tickets de dette ouverts
    // non soldés, chacun borné à <= 250K USD micros, y compris le backlog overflow.
    pub protocol_fee_debt_usd_micros: u64,
    pub protocol_fee_debt_count: u64,
    pub protocol_fee_debt_last_ts: i64,

    pub created_at: i64,
    pub last_swap_ts: i64,
    pub total_base_volume: u64,
    pub total_quote_volume: u64,

    pub swap_count: u64,

    pub bump: u8,
}

impl Pool {
    pub const LEN: usize = 354;
}

#[account]
pub struct PoolRegistryEntry {
    pub pool: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub lp_mint: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub fee_vault_base: Pubkey,
    pub fee_vault_quote: Pubkey,
    pub created_at: i64,
    pub bump: u8,
}

impl PoolRegistryEntry {
    pub const LEN: usize = 8 + (32 * 8) + 8 + 1;
}

#[cfg(not(any(
    feature = "migration-payout",
    feature = "migration-claim",
    feature = "migration-ledger"
)))]
#[account]
pub struct ProtocolDebtLedger {
    pub pool: Pubkey,
    pub next_nonce: u64,
    // Backlog logique de tickets de dette <= 250K non encore assignés à un slot libre.
    // Il reste payable via rails keeper-compatibles et peut être rematérialisé plus tard.
    pub overflow_usd_micros: u64,
    pub last_ts: i64,
    pub bump: u8,
    // Lots/tickets logiques matérialisables individuellement, chacun <= 250K USD micros.
    pub lots: [ProtocolDebtLot; PROTOCOL_DEBT_LEDGER_SLOTS],
}

#[cfg(any(
    feature = "migration-payout",
    feature = "migration-claim",
    feature = "migration-ledger"
))]
pub struct ProtocolDebtLedger {
    pub pool: Pubkey,
    pub next_nonce: u64,
    pub overflow_usd_micros: u64,
    pub last_ts: i64,
    pub bump: u8,
    pub lots: [ProtocolDebtLot; PROTOCOL_DEBT_LEDGER_SLOTS],
}

#[cfg(any(
    feature = "migration-payout",
    feature = "migration-claim",
    feature = "migration-ledger"
))]
impl anchor_lang::Discriminator for ProtocolDebtLedger {
    const DISCRIMINATOR: [u8; 8] = [0x89, 0x77, 0xc3, 0xdf, 0xa3, 0xf6, 0x67, 0xe5];
}

impl ProtocolDebtLedger {
    pub const LEN: usize =
        8 + 32 + 8 + 8 + 8 + 1 + (ProtocolDebtLot::LEN * PROTOCOL_DEBT_LEDGER_SLOTS);
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProtocolDebtLot {
    pub nonce: u64,
    pub mint_in: Pubkey,
    // `amount_in` conserve le montant source global pour audit quand le lot vient
    // directement d'une fee; il peut valoir 0 pour un lot rematérialisé depuis overflow.
    pub amount_in: u64,
    pub usd_micros: u64,
    pub created_ts: i64,
    pub escrow_mint: Pubkey,
    pub escrow_ata: Pubkey,
    pub escrow_amount_locked: u64,
    pub escrow_bump: u8,
    pub funding_state: u8,
    pub route_hint: u8,
    pub status: u8,
}

impl ProtocolDebtLot {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 8 + 32 + 32 + 8 + 1 + 1 + 1 + 1;
}

#[account]
pub struct ValueRegistry {
    pub mint: Pubkey,
    pub value_usd_micros: u64,
    pub true_cash: bool,
    pub cash_backed: bool,
    pub real_peg: bool,
    pub sovereign: bool,
    pub updated_at: i64,
    pub bump: u8,
}

#[account]
pub struct AssetRegistry {
    pub mint: Pubkey,
    pub valuation_usd_micros: u64,
    pub decimals: u8,
    pub is_lp: bool,
    pub active: bool,
    pub bump: u8,
}

// =========================
// PAO V3 STATE (anti double settle receipt)
// =========================
#[account]
pub struct PayoutTicket {
    pub pool: Pubkey,
    pub payout_mint: Pubkey,
    pub payout_kind: u8,
    pub user: Pubkey,
    pub mint_in: Pubkey,
    pub amount_in: u64,
    pub usd_micros: u64,
    pub destination_ata: Pubkey,
    pub escrow_mint: Pubkey,
    pub escrow_ata: Pubkey,
    pub escrow_amount_locked: u64,
    pub nonce: u64,
    pub created_ts: i64,
    pub settled_ts: i64,
    pub status: u8, // 0=requested, 1=settled
    pub funding_state: u8,
    pub escrow_bump: u8,
    pub route_hint: u8,
    pub bump: u8,
}

impl PayoutTicket {
    pub const LEN: usize =
        8 + (32 + 32 + 1 + 32 + 32 + 8 + 8 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 1 + 1 + 1 + 1 + 1);
}

// =========================
// EVENTS
// =========================
#[derive(Accounts)]
#[instruction(nonce: u64)]
pub struct SettlePayoutV3Safe<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub keeper: Signer<'info>,

    /// CHECK: receipt-only
    pub user: UncheckedAccount<'info>,
    /// CHECK: receipt-only
    pub pool: UncheckedAccount<'info>,

    // payout mint used for validation/logging
    pub payout_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = keeper,
        seeds = [b"payout", user.key().as_ref(), pool.key().as_ref(), &nonce.to_le_bytes()],
        bump,
        space = PayoutTicket::LEN
    )]
    pub ticket: Box<Account<'info, PayoutTicket>>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[event]
pub struct FeeThresholdEvent {
    pub pool: Pubkey,
    pub fee_vault: Pubkey,
    pub fee_amount: u64,
    pub fee_value_usd_micros: u64,
    pub ts: i64,
}

#[event]
pub struct PoolCreated {
    pub pool: Pubkey,
    pub owner: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub ts: i64,
}

#[event]
pub struct PoolRuntimeAddressSet {
    pub pool: Pubkey,
    pub role: u8,
    pub mint: Pubkey,
    pub account: Pubkey,
    pub ts: i64,
}

#[event]
pub struct LiquidityAdded {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_amount_in: u64,
    pub quote_amount_in: u64,
    pub lp_amount_out: u64,
    pub ts: i64,
}

#[event]
pub struct LiquidityRemoved {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub lp_amount_in: u64,
    pub base_amount_out: u64,
    pub quote_amount_out: u64,
    pub ts: i64,
}

#[event]
pub struct SwapExecuted {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub side: FeeSide,
    pub mint_in: Pubkey,
    pub amount_in: u64,
    pub mint_out: Pubkey,
    pub amount_out: u64,
    pub fee_amount: u64,
    pub fee_value_usd_micros: u64,
    pub ts: i64,
}

#[event]
pub struct FeeCollectedToTreasuryEvent {
    pub pool: Pubkey,
    pub fee_vault: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub treasury_ata: Pubkey,
    pub ts: i64,
}

#[event]
pub struct FeeConvertedToUsdcEvent {
    pub pool: Pubkey,
    pub burned_mint: Pubkey,
    pub burned_amount: u64,
    pub usdc_released: u64,
    pub usdc_coffre: Pubkey,
    pub treasury_usdc_ata: Pubkey,
    pub ts: i64,
}

#[event]
pub struct FeeConvertedToStableEvent {
    pub pool: Pubkey,
    pub burned_mint: Pubkey,
    pub burned_amount: u64,
    pub stable_mint: Pubkey,
    pub stable_released: u64,
    pub stable_coffre: Pubkey,
    pub treasury_stable_ata: Pubkey,
    pub ts: i64,
}

#[event]
pub struct FeeRetainedInVaultEvent {
    pub pool: Pubkey,
    pub fee_vault: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub value_usd_micros: u64,
    pub ts: i64,
}

#[event]
pub struct SovereignOutputExecuted {
    pub source_kind: u8,
    pub source_mint: Pubkey,
    pub amount_in: u64,
    pub payout_mint: Pubkey,
    pub payout_amount: u64,
    pub destination_ata: Pubkey,
    pub user: Pubkey,
    pub ts: i64,
}

#[event]
pub struct PoolRegistered {
    pub pool: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub lp_mint: Pubkey,
    pub ts: i64,
}

#[event]
pub struct ProtocolFeeDebtEvent {
    pub pool: Pubkey,
    pub payout_mint: Pubkey,
    pub payout_kind: u8,
    pub beneficiary: Pubkey,
    pub mint_in: Pubkey,
    pub amount_in: u64,
    pub usd_micros: u64,
    pub total_usd_micros: u64,
    pub destination_ata: Pubkey,
    pub nonce: u64,
    pub route_hint: u8,
    pub ts: i64,
}

#[event]
pub struct ProtocolFeeDebtSettledEvent {
    pub pool: Pubkey,
    pub paid_usd_micros: u64,
    pub remaining_usd_micros: u64,
    pub proof_sig: [u8; 64],
    pub ts: i64,
}

// =========================
// PAO V3 EVENTS (bot listens)
// =========================
#[event]
pub struct NeedPayoutEvent {
    pub pool: Pubkey,
    pub payout_mint: Pubkey,
    pub payout_kind: u8, // 1=swap,2=add_lp,3=remove_lp,4=claim,5=fees,6=redeem,7=manual
    pub user: Pubkey,
    pub mint_in: Pubkey,
    pub amount_in: u64,
    pub usd_micros: u64,
    pub destination_ata: Pubkey,
    pub nonce: u64,
    pub liquidity_policy: u8, // 1=DEX->CEX->OTC allowed (bot side)
    pub route_hint: u8,       // 0=auto,1=stableA,2=stableB,3=extra0,4=extra1,5=extra2,6=extra3
    pub ts: i64,
}

#[event]
pub struct HtopReserveMaterializedEvent {
    pub source_mint: Pubkey,
    pub source_account: Pubkey,
    pub destination_htop_reserve: Pubkey,
    pub amount_atoms: u64,
    pub usd_micros: u64,
    pub ts: i64,
}

// =========================
// ERRORS
// =========================
#[error_code]
pub enum SterlingError {
    #[msg("Bad value registry / asset registry")]
    BadRegistry,
    #[msg("Cash flags required")]
    CashFlagsRequired,
    #[msg("Inactive pool")]
    InactivePool,
    #[msg("Insufficient liquidity")]
    InsufficientLiquidity,
    #[msg("Insufficient rewards in vault")]
    InsufficientRewards,
    #[msg("Invalid account")]
    InvalidAccount,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Invalid bps")]
    InvalidBps,
    #[msg("Invalid interval")]
    InvalidInterval,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Mint disabled by config")]
    MintDisabled,
    #[msg("Slippage exceeded")]
    SlippageExceeded,
    #[msg("Too early to claim")]
    TooEarlyClaim,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Unsupported mint")]
    UnsupportedMint,
    #[msg("Zero LP")]
    ZeroLp,
    #[msg("Zero reward")]
    ZeroReward,
    #[msg("Keeper only")]
    KeeperOnly,
    #[msg("Invalid state")]
    InvalidState,
    #[msg("Payout exceeds per-ticket limit")]
    PayoutTooLarge,
    #[msg("Payout exceeds window limit")]
    PayoutWindowExceeded,
    #[msg("Unsupported payout mint")]
    UnsupportedPayoutMint,
    #[msg("USDC settlement accounts missing")]
    UsdcSettlementAccountsMissing,
    #[msg("USDC settlement vault mismatch")]
    UsdcSettlementVaultMismatch,
    #[msg("USDC settlement treasury mismatch")]
    UsdcSettlementTreasuryMismatch,
    #[msg("Insufficient USDC settlement liquidity")]
    InsufficientUsdcSettlementLiquidity,
    #[msg("Fee asset registry missing")]
    FeeAssetRegistryMissing,
    #[msg("Fee asset registry inactive")]
    FeeAssetRegistryInactive,
    #[msg("Zero USDC settlement")]
    ZeroUsdcSettlement,
    #[msg("Unsupported sovereign final output")]
    UnsupportedSovereignFinalOutput,
    #[msg("Stable settlement accounts missing")]
    StableSettlementAccountsMissing,
    #[msg("Stable settlement vault mismatch")]
    StableSettlementVaultMismatch,
    #[msg("Stable settlement destination mismatch")]
    StableSettlementDestinationMismatch,
    #[msg("Insufficient stable settlement liquidity")]
    InsufficientStableSettlementLiquidity,
    #[msg("Legacy internal settlement rail disabled")]
    LegacyInternalSettlementDisabled,
    #[msg("Invalid sovereign escrow binding")]
    InvalidSovereignEscrowBinding,
    #[msg("Sovereign escrow is not funded")]
    SovereignEscrowNotFunded,
    #[msg("Invalid sovereign escrow funding amount")]
    InvalidSovereignEscrowFundingAmount,
    #[msg("Temporary WSOL output account must be empty before live use")]
    WsolOutputAccountNotEmpty,
    #[msg("Zero stable settlement")]
    ZeroStableSettlement,
    #[msg("No executable sovereign rail")]
    NoExecutableSovereignRail,
    #[msg("Native SOL rail not enabled")]
    NativeSolRailNotEnabled,
    #[msg("Debt lot nonce required")]
    DebtLotNonceRequired,
    #[msg("Invalid sovereign remaining accounts layout")]
    InvalidSovereignAccountsLayout,
    #[msg("Invalid native SOL destination")]
    InvalidNativeSolDestination,
    #[msg("Insufficient native SOL settlement liquidity")]
    InsufficientNativeSolSettlementLiquidity,
    #[msg("Zero native SOL settlement")]
    ZeroNativeSolSettlement,
    #[msg("Live sovereign exchange route required")]
    LiveSovereignExchangeRequired,
    #[msg("Deferred sovereign route is not a live exchange rail")]
    DeferredSovereignRouteNotLive,
    #[msg("Invalid live sovereign pool route")]
    InvalidLiveSovereignPoolRoute,
    #[msg("Native SOL live exchange is not fully wired yet")]
    NativeSolLiveExchangeNotReady,
    #[msg("Target pool floor price violation")]
    FloorPriceViolation,
}

// =========================
// HELPERS
// =========================

fn is_true_cash_mint(mint: &Pubkey) -> bool {
    let treasury_root = pubkey_from_str(TREASURY_ROOT_MINT);
    let sjbc = pubkey_from_str(SJBC_MINT);
    let sjbc2 = pubkey_from_str(SJBC2_MINT);
    let sjbc3 = pubkey_from_str(SJBC3_MINT);
    *mint == treasury_root || *mint == sjbc || *mint == sjbc2 || *mint == sjbc3
}

fn is_usdc_mint(mint: &Pubkey) -> bool {
    let usdc = pubkey_from_str(USDC_MINT);
    *mint == usdc
}

fn is_usdt_mint(mint: &Pubkey) -> bool {
    let usdt_main = pubkey_from_str(USDT_MAIN_MINT);
    *mint == usdt_main
}

fn expected_operator_to_htop_materialization_accounts(mint: Pubkey) -> Option<(Pubkey, Pubkey)> {
    let stm_mint = pubkey_from_str("9kued2JXgVk5dzvtipsTdXfBMWihy1E55TwMiXchCoAb");
    if mint == stm_mint {
        return Some((
            pubkey_from_str(OPERATOR_STM_RESERVE_ATA),
            pubkey_from_str(HTOP_STM_RESERVE_ATA),
        ));
    }

    let sjbc_mint = pubkey_from_str("EsNo61QodqHCRjkTGJDeqyK7N4Hunip5PaTYbpPZEsG2");
    if mint == sjbc_mint {
        return Some((
            pubkey_from_str(OPERATOR_SJBC_RESERVE_ATA),
            pubkey_from_str(HTOP_SJBC_RESERVE_ATA),
        ));
    }

    None
}

fn is_floor_protected_pool_key(pool_key: Pubkey) -> bool {
    pool_key == pubkey_from_str(POOL_ID)
}

fn require_floor_protected_pool_bindings(pool_key: Pubkey, pool: &Pool) -> Result<()> {
    require!(pool_key == pubkey_from_str(POOL_ID), SterlingError::InvalidAccount);
    require!(
        pool.base_vault == pubkey_from_str(FLOOR_PROTECTED_BASE_VAULT),
        SterlingError::InvalidAccount
    );
    require!(
        pool.quote_vault == pubkey_from_str(FLOOR_PROTECTED_QUOTE_VAULT),
        SterlingError::InvalidAccount
    );
    require!(pool.base_mint == pubkey_from_str(SJBC_MINT), SterlingError::InvalidAccount);
    require!(
        pool.quote_mint == pubkey_from_str(SJBC2_MINT),
        SterlingError::InvalidAccount
    );
    require!(pool.owner == pubkey_from_str(LP_AUTH_2), SterlingError::InvalidAccount);
    Ok(())
}

fn validate_target_floor_reserve_accounts<'info>(
    htop_stm_reserve_info: &AccountInfo<'info>,
    htop_sjbc_reserve_info: &AccountInfo<'info>,
) -> Result<()> {
    require!(
        htop_stm_reserve_info.key() == pubkey_from_str(HTOP_STM_RESERVE_ATA),
        SterlingError::InvalidAccount
    );
    require!(
        htop_sjbc_reserve_info.key() == pubkey_from_str(HTOP_SJBC_RESERVE_ATA),
        SterlingError::InvalidAccount
    );

    let htop_stm = load_token_account_snapshot(htop_stm_reserve_info)?;
    let htop_sjbc = load_token_account_snapshot(htop_sjbc_reserve_info)?;

    require!(
        htop_stm.mint == pubkey_from_str(SJBC_MINT),
        SterlingError::InvalidAccount
    );
    require!(
        htop_sjbc.mint == pubkey_from_str(SJBC2_MINT),
        SterlingError::InvalidAccount
    );
    require!(
        htop_stm.owner == pubkey_from_str(LP_AUTH_2),
        SterlingError::InvalidAccount
    );
    require!(
        htop_sjbc.owner == pubkey_from_str(LP_AUTH_2),
        SterlingError::InvalidAccount
    );
    require!(
        htop_stm.amount >= MIN_HTOP_STM_GUARANTEE_ATOMS,
        SterlingError::InvalidAmount
    );
    require!(
        htop_sjbc.amount >= MIN_HTOP_SJBC_GUARANTEE_ATOMS,
        SterlingError::InvalidAmount
    );
    Ok(())
}

fn enforce_floor_price_post_swap(
    floor_price_usd_micros: u64,
    post_base_atoms: u64,
    post_quote_atoms: u64,
) -> Result<()> {
    require!(post_base_atoms > 0, SterlingError::InvalidAmount);
    require!(post_quote_atoms > 0, SterlingError::InvalidAmount);
    let lhs = (post_quote_atoms as u128)
        .checked_mul(USD_MICROS as u128)
        .ok_or(SterlingError::MathOverflow)?;
    let rhs = (post_base_atoms as u128)
        .checked_mul(floor_price_usd_micros as u128)
        .ok_or(SterlingError::MathOverflow)?;
    require!(lhs >= rhs, SterlingError::FloorPriceViolation);
    Ok(())
}

fn split_swap_base_remaining_accounts_for_floor<'info>(
    pool_key: Pubkey,
    remaining_accounts: &'info [AccountInfo<'info>],
) -> Result<(
    Option<AccountInfo<'info>>,
    Option<AccountInfo<'info>>,
    &'info [AccountInfo<'info>],
)> {
    if !is_floor_protected_pool_key(pool_key) {
        return Ok((None, None, remaining_accounts));
    }

    require!(FLOOR_PRICE_ENABLED, SterlingError::InvalidState);
    require!(remaining_accounts.len() >= 2, SterlingError::InvalidAccount);
    Ok((
        Some(remaining_accounts[0].clone()),
        Some(remaining_accounts[1].clone()),
        &remaining_accounts[2..],
    ))
}

fn is_portal_btc_mint(mint: &Pubkey) -> bool {
    let btc_portal = pubkey_from_str(BTC_PORTAL_MINT);
    *mint == btc_portal
}

fn is_supported_cash_mint(mint: &Pubkey) -> bool {
    is_true_cash_mint(mint) || is_usdc_mint(mint) || is_usdt_mint(mint) || is_portal_btc_mint(mint)
}

fn is_configured_lp_mint(cfg: &Config, mint: &Pubkey) -> bool {
    (*mint == cfg.lp_mint2 && cfg.lp_mint2 != Pubkey::default())
        || (*mint == cfg.lp_mint3 && cfg.lp_mint3 != Pubkey::default())
}

fn is_supported_staking_reward_mint(cfg: &Config, mint: &Pubkey) -> bool {
    is_supported_cash_mint(mint) || is_configured_lp_mint(cfg, mint)
}

fn is_enabled_staking_reward_mint(cfg: &Config, mint: &Pubkey) -> bool {
    if is_configured_lp_mint(cfg, mint) {
        return true;
    }
    is_enabled(cfg, mint)
}

fn is_enabled(cfg: &Config, mint: &Pubkey) -> bool {
    let treasury_root = pubkey_from_str(TREASURY_ROOT_MINT);
    let sjbc = pubkey_from_str(SJBC_MINT);
    let sjbc2 = pubkey_from_str(SJBC2_MINT);
    let sjbc3 = pubkey_from_str(SJBC3_MINT);
    let usdc = pubkey_from_str(USDC_MINT);
    let usdt_main = pubkey_from_str(USDT_MAIN_MINT);
    let btc_portal = pubkey_from_str(BTC_PORTAL_MINT);

    if *mint == treasury_root {
        return cfg.enable_treasury;
    }
    if *mint == sjbc {
        return cfg.enable_sjbc;
    }
    if *mint == sjbc2 {
        return cfg.enable_sjbc2;
    }
    if *mint == sjbc3 {
        return cfg.enable_sjbc3;
    }
    if *mint == usdc {
        return cfg.enable_usdc;
    }
    if *mint == usdt_main {
        return cfg.enable_usdt_main;
    }
    if *mint == btc_portal {
        return cfg.enable_btc_portal;
    }
    false
}

fn set_enabled_flag(cfg: &mut Config, mint: &Pubkey, enabled: bool) -> Result<()> {
    if is_configured_lp_mint(cfg, mint) {
        require!(enabled, SterlingError::UnsupportedMint);
        return Ok(());
    }

    let treasury_root = pubkey_from_str(TREASURY_ROOT_MINT);
    let sjbc = pubkey_from_str(SJBC_MINT);
    let sjbc2 = pubkey_from_str(SJBC2_MINT);
    let sjbc3 = pubkey_from_str(SJBC3_MINT);
    let usdc = pubkey_from_str(USDC_MINT);
    let usdt_main = pubkey_from_str(USDT_MAIN_MINT);
    let btc_portal = pubkey_from_str(BTC_PORTAL_MINT);

    if *mint == treasury_root {
        cfg.enable_treasury = enabled;
        return Ok(());
    }
    if *mint == sjbc {
        cfg.enable_sjbc = enabled;
        return Ok(());
    }
    if *mint == sjbc2 {
        cfg.enable_sjbc2 = enabled;
        return Ok(());
    }
    if *mint == sjbc3 {
        cfg.enable_sjbc3 = enabled;
        return Ok(());
    }
    if *mint == usdc {
        cfg.enable_usdc = enabled;
        return Ok(());
    }
    if *mint == usdt_main {
        cfg.enable_usdt_main = enabled;
        return Ok(());
    }
    if *mint == btc_portal {
        cfg.enable_btc_portal = enabled;
        return Ok(());
    }

    err!(SterlingError::UnsupportedMint)
}

#[cfg(any(feature = "migration-full", feature = "migration-ledger"))]
const CONFIG_ACCOUNT_LEN: usize = 8 + 1400;
#[cfg(any(feature = "migration-full", feature = "migration-ledger"))]
const LEGACY_CONFIG_V1_MIN_LEN: usize = 8 + 71;
#[cfg(any(feature = "migration-full", feature = "migration-ledger"))]
const LEGACY_POOL_V1_MIN_LEN: usize = 8 + 215;
#[cfg(feature = "migration-payout")]
const LEGACY_PAYOUT_TICKET_V1_MIN_LEN: usize = 8 + 203;
#[cfg(feature = "migration-claim")]
const LEGACY_SETTLEMENT_CLAIM_V1_MIN_LEN: usize = 8 + 283;
#[cfg(any(feature = "migration-full", feature = "migration-ledger"))]
const LEGACY_PROTOCOL_DEBT_LEDGER_V1_MIN_LEN: usize = 1121;
#[cfg(any(feature = "migration-full", feature = "migration-ledger"))]
const LEGACY_PROTOCOL_DEBT_LOT_V1_LEN: usize = 8 + 32 + 8 + 8 + 8 + 1 + 1;

#[cfg(any(feature = "migration-full", feature = "migration-ledger"))]
#[cfg_attr(test, derive(AnchorSerialize, Debug, PartialEq, Eq))]
struct ConfigLegacyV1 {
    admin: Pubkey,
    true_cash: bool,
    cash_backed: bool,
    real_peg: bool,
    sovereign: bool,
    cashback_bps: u16,
    reward_interval: u64,
    allow_fallback_usdt: bool,
    token_value_usd_micros_default: u64,
    treasury_value_usd_micros: u64,
    enable_treasury: bool,
    enable_sjbc: bool,
    enable_sjbc2: bool,
    enable_sjbc3: bool,
    enable_usdc: bool,
    enable_usdt_main: bool,
    enable_usdt_old: bool,
    enable_btc_portal: bool,
}

#[cfg(any(feature = "migration-full", feature = "migration-ledger"))]
#[cfg_attr(test, derive(AnchorSerialize, Debug, PartialEq, Eq))]
struct PoolLegacyV1 {
    owner: Pubkey,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    base_vault: Pubkey,
    quote_vault: Pubkey,
    lp_mint: Pubkey,
    base_value_usd_micros: u64,
    quote_value_usd_micros: u64,
    true_cash: bool,
    cash_backed: bool,
    real_peg: bool,
    sovereign: bool,
    fee_bps: u16,
    active: bool,
}

#[cfg(any(feature = "migration-full", feature = "migration-ledger"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ProtocolDebtLotLegacyV1 {
    nonce: u64,
    mint_in: Pubkey,
    amount_in: u64,
    usd_micros: u64,
    created_ts: i64,
    route_hint: u8,
    status: u8,
}

#[cfg(any(feature = "migration-full", feature = "migration-ledger"))]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
struct ProtocolDebtLedgerLegacyV1 {
    pool: Pubkey,
    next_nonce: u64,
    overflow_usd_micros: u64,
    last_ts: i64,
    bump: u8,
    lots: [ProtocolDebtLotLegacyV1; PROTOCOL_DEBT_LEDGER_SLOTS],
}

#[cfg(any(feature = "migration-full", feature = "migration-ledger"))]
fn config_needs_migration(account_len: usize) -> bool {
    account_len < CONFIG_ACCOUNT_LEN
}

#[cfg(any(feature = "migration-full", feature = "migration-ledger"))]
fn pool_needs_migration(account_len: usize) -> bool {
    account_len < Pool::LEN
}

#[cfg(any(feature = "migration-full", feature = "migration-ledger"))]
fn protocol_debt_ledger_needs_migration(account_len: usize) -> bool {
    account_len < ProtocolDebtLedger::LEN
}

#[cfg(any(feature = "migration-full", feature = "migration-ledger"))]
fn cfg_bool(d: &[u8], o: usize) -> bool {
    d[o] != 0
}

#[cfg(any(
    feature = "migration-payout",
    feature = "migration-claim",
    feature = "migration-ledger"
))]
fn legacy_pk(d: &[u8], o: usize) -> Pubkey {
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&d[o..o + 32]);
    Pubkey::new_from_array(bytes)
}

#[cfg(any(
    feature = "migration-payout",
    feature = "migration-claim",
    feature = "migration-ledger"
))]
fn legacy_u64(d: &[u8], o: usize) -> u64 {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&d[o..o + 8]);
    u64::from_le_bytes(bytes)
}

#[cfg(any(
    feature = "migration-payout",
    feature = "migration-claim",
    feature = "migration-ledger"
))]
fn legacy_i64(d: &[u8], o: usize) -> i64 {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&d[o..o + 8]);
    i64::from_le_bytes(bytes)
}

#[cfg(any(feature = "migration-full", feature = "migration-ledger"))]
fn deserialize_legacy_config_v1(data: &[u8]) -> Result<ConfigLegacyV1> {
    require!(
        data.len() >= LEGACY_CONFIG_V1_MIN_LEN,
        SterlingError::InvalidAccount
    );
    let body = &data[8..];
    Ok(ConfigLegacyV1 {
        admin: legacy_pk(body, 0),
        true_cash: cfg_bool(body, 32),
        cash_backed: cfg_bool(body, 33),
        real_peg: cfg_bool(body, 34),
        sovereign: cfg_bool(body, 35),
        cashback_bps: u16::from_le_bytes([body[36], body[37]]),
        reward_interval: legacy_u64(body, 38),
        allow_fallback_usdt: cfg_bool(body, 46),
        token_value_usd_micros_default: legacy_u64(body, 47),
        treasury_value_usd_micros: legacy_u64(body, 55),
        enable_treasury: cfg_bool(body, 63),
        enable_sjbc: cfg_bool(body, 64),
        enable_sjbc2: cfg_bool(body, 65),
        enable_sjbc3: cfg_bool(body, 66),
        enable_usdc: cfg_bool(body, 67),
        enable_usdt_main: cfg_bool(body, 68),
        enable_usdt_old: cfg_bool(body, 69),
        enable_btc_portal: cfg_bool(body, 70),
    })
}

#[cfg(any(feature = "migration-full", feature = "migration-ledger"))]
fn deserialize_legacy_pool_v1(data: &[u8]) -> Result<PoolLegacyV1> {
    require!(
        data.len() >= LEGACY_POOL_V1_MIN_LEN,
        SterlingError::InvalidAccount
    );
    let body = &data[8..];
    Ok(PoolLegacyV1 {
        owner: legacy_pk(body, 0),
        base_mint: legacy_pk(body, 32),
        quote_mint: legacy_pk(body, 64),
        base_vault: legacy_pk(body, 96),
        quote_vault: legacy_pk(body, 128),
        lp_mint: legacy_pk(body, 160),
        base_value_usd_micros: legacy_u64(body, 192),
        quote_value_usd_micros: legacy_u64(body, 200),
        true_cash: cfg_bool(body, 208),
        cash_backed: cfg_bool(body, 209),
        real_peg: cfg_bool(body, 210),
        sovereign: cfg_bool(body, 211),
        fee_bps: u16::from_le_bytes([body[212], body[213]]),
        active: cfg_bool(body, 214),
    })
}

#[cfg(feature = "migration-payout")]
fn build_payout_ticket_from_legacy_v1(ticket_key: Pubkey, data: &[u8]) -> Result<PayoutTicket> {
    require!(
        data.len() >= LEGACY_PAYOUT_TICKET_V1_MIN_LEN,
        SterlingError::InvalidAccount
    );
    let body = &data[8..];
    let mint_in = legacy_pk(body, 97);
    let amount_in = legacy_u64(body, 129);
    let status = body[201];
    let settled_ts = legacy_i64(body, 193);
    let (escrow_authority, escrow_bump) = expected_ticket_escrow_authority(ticket_key);
    let funding_state = if status == PAYOUT_TICKET_STATUS_SETTLED || settled_ts != 0 {
        SOVEREIGN_ESCROW_STATE_SETTLED
    } else {
        SOVEREIGN_ESCROW_STATE_REQUESTED
    };

    Ok(PayoutTicket {
        pool: legacy_pk(body, 0),
        payout_mint: legacy_pk(body, 32),
        payout_kind: body[64],
        user: legacy_pk(body, 65),
        mint_in,
        amount_in,
        usd_micros: legacy_u64(body, 137),
        destination_ata: legacy_pk(body, 145),
        escrow_mint: mint_in,
        escrow_ata: expected_live_escrow_ata(escrow_authority, mint_in),
        escrow_amount_locked: amount_in,
        nonce: legacy_u64(body, 177),
        created_ts: legacy_i64(body, 185),
        settled_ts,
        status,
        funding_state,
        escrow_bump,
        route_hint: 0,
        bump: body[202],
    })
}

#[cfg(feature = "migration-claim")]
fn build_settlement_claim_from_legacy_v1(
    claim_key: Pubkey,
    data: &[u8],
) -> Result<SettlementClaim> {
    require!(
        data.len() >= LEGACY_SETTLEMENT_CLAIM_V1_MIN_LEN,
        SterlingError::InvalidAccount
    );
    let body = &data[8..];
    let mut proof_sig = [0u8; 64];
    proof_sig.copy_from_slice(&body[161..225]);
    let mint_in = legacy_pk(body, 97);
    let amount_in = legacy_u64(body, 129);
    let due_atoms = legacy_u64(body, 145);
    let paid_atoms = legacy_u64(body, 153);
    let settled_ts = legacy_i64(body, 273);
    let status = body[281];
    let (escrow_authority, escrow_bump) = expected_claim_escrow_authority(claim_key);
    let funding_state = if status == SETTLEMENT_CLAIM_STATUS_PAID
        || settled_ts != 0
        || (due_atoms > 0 && paid_atoms >= due_atoms)
    {
        SOVEREIGN_ESCROW_STATE_SETTLED
    } else {
        SOVEREIGN_ESCROW_STATE_REQUESTED
    };

    Ok(SettlementClaim {
        pool: legacy_pk(body, 0),
        payout_mint: legacy_pk(body, 32),
        payout_kind: body[64],
        user: legacy_pk(body, 65),
        mint_in,
        amount_in,
        usd_micros: legacy_u64(body, 137),
        due_atoms,
        paid_atoms,
        proof_sig,
        destination_ata: legacy_pk(body, 225),
        escrow_mint: mint_in,
        escrow_ata: expected_live_escrow_ata(escrow_authority, mint_in),
        escrow_amount_locked: amount_in,
        nonce: legacy_u64(body, 257),
        created_ts: legacy_i64(body, 265),
        settled_ts,
        status,
        funding_state,
        escrow_bump,
        bump: body[282],
    })
}

#[cfg(any(feature = "migration-full", feature = "migration-ledger"))]
#[inline(never)]
fn migrate_protocol_debt_ledger_data_from_legacy_v1(
    ledger_data: &mut [u8],
    legacy_data: &[u8],
    pool_key: Pubkey,
) -> Result<()> {
    ledger_data[8..].fill(0);
    ledger_data[8..65].copy_from_slice(&legacy_data[8..65]);

    for index in 0..PROTOCOL_DEBT_LEDGER_SLOTS {
        let legacy_start = 65 + (index * LEGACY_PROTOCOL_DEBT_LOT_V1_LEN);
        let legacy_end = legacy_start + LEGACY_PROTOCOL_DEBT_LOT_V1_LEN;
        let legacy_lot = &legacy_data[legacy_start..legacy_end];
        let status = legacy_lot[65];
        if status == PROTOCOL_DEBT_LOT_EMPTY {
            continue;
        }

        let nonce = legacy_u64(legacy_lot, 0);
        let mint_in = legacy_pk(legacy_lot, 8);
        let amount_in = legacy_u64(legacy_lot, 40);
        let funding_state = if status == PROTOCOL_DEBT_LOT_SETTLED {
            SOVEREIGN_ESCROW_STATE_SETTLED
        } else {
            SOVEREIGN_ESCROW_STATE_REQUESTED
        };
        let (escrow_authority, escrow_bump) =
            expected_protocol_debt_lot_escrow_authority(pool_key, nonce);
        let lot = ProtocolDebtLot {
            nonce,
            mint_in,
            amount_in,
            usd_micros: legacy_u64(legacy_lot, 48),
            created_ts: legacy_i64(legacy_lot, 56),
            escrow_mint: mint_in,
            escrow_ata: expected_live_escrow_ata(escrow_authority, mint_in),
            escrow_amount_locked: amount_in,
            escrow_bump,
            funding_state,
            route_hint: legacy_lot[64],
            status,
        };
        store_protocol_debt_lot_into_ledger_data(ledger_data, index, &lot)?;
    }

    Ok(())
}

#[cfg(any(feature = "migration-full", feature = "migration-ledger"))]
fn config_admin_pubkey(ai: &AccountInfo) -> Result<Pubkey> {
    let data = ai.try_borrow_data()?;
    require!(data.len() >= 8 + 32, SterlingError::InvalidAccount);
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&data[8..40]);
    Ok(Pubkey::new_from_array(bytes))
}

#[cfg(any(feature = "migration-full", feature = "migration-ledger"))]
fn require_admin_on_config_info<'info>(
    config_ai: &AccountInfo<'info>,
    admin: &Signer<'info>,
) -> Result<()> {
    require!(
        config_admin_pubkey(config_ai)? == admin.key(),
        SterlingError::Unauthorized
    );
    Ok(())
}

#[cfg(feature = "migration-full")]
#[inline(never)]
fn require_migration_config_admin<'info>(
    config_ai: &AccountInfo<'info>,
    admin: &Signer<'info>,
    program_id: &Pubkey,
) -> Result<()> {
    let (expected_config, _) = Pubkey::find_program_address(&[b"config"], program_id);
    require!(
        config_ai.key() == expected_config,
        SterlingError::InvalidAccount
    );
    require!(config_ai.owner == program_id, SterlingError::InvalidAccount);
    require_admin_on_config_info(config_ai, admin)
}

#[cfg(any(
    feature = "migration-payout",
    feature = "migration-claim",
    feature = "migration-ledger"
))]
fn realloc_program_account<'info>(
    account: &AccountInfo<'info>,
    payer: &Signer<'info>,
    system_program_account: &Program<'info, System>,
    target_len: usize,
) -> Result<()> {
    if account.data_len() >= target_len {
        return Ok(());
    }

    let rent = Rent::get()?;
    let required_lamports = rent.minimum_balance(target_len);
    let current_lamports = account.lamports();
    if required_lamports > current_lamports {
        anchor_lang::system_program::transfer(
            CpiContext::new(
                system_program_account.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: payer.to_account_info(),
                    to: account.clone(),
                },
            ),
            required_lamports.saturating_sub(current_lamports),
        )?;
    }

    account.realloc(target_len, false)?;
    Ok(())
}

#[cfg(any(feature = "migration-full", feature = "migration-ledger"))]
fn default_config_v2_state(admin: Pubkey, bump: u8) -> Config {
    Config {
        admin,
        true_cash: TRUE_CASH_FLAG,
        cash_backed: CASH_BACKED_FLAG,
        real_peg: REAL_PEG_FLAG,
        sovereign: SOVEREIGN_FLAG,
        cashback_bps: DEFAULT_CASHBACK_BPS,
        reward_interval: DEFAULT_REWARD_INTERVAL_SECONDS,
        allow_fallback_usdt: true,
        native_sol_enabled: false,
        native_sol_usd_micros_per_sol: DEFAULT_NATIVE_SOL_USD_MICROS_PER_SOL,
        native_sol_min_reserve_lamports: DEFAULT_NATIVE_SOL_MIN_RESERVE_LAMPORTS,
        token_value_usd_micros_default: DEFAULT_TOKEN_VALUE_USD_MICROS,
        treasury_value_usd_micros: DEFAULT_TREASURY_VALUE_USD_MICROS,
        enable_treasury: true,
        enable_sjbc: true,
        enable_sjbc2: true,
        enable_sjbc3: true,
        enable_usdc: true,
        enable_usdt_main: true,
        enable_usdt_old: false,
        enable_btc_portal: true,
        main_wallet: pubkey_from_str(MAIN_WALLET),
        okx_wallet: pubkey_from_str(OKX_WALLET),
        lp_mint2: pubkey_from_str(LP_MINT_2),
        lp_auth2: pubkey_from_str(LP_AUTH_2),
        lp_mint3: pubkey_from_str(LP_MINT_3),
        lp_auth3: pubkey_from_str(LP_AUTH_3),
        mint_h1: pubkey_from_str(MINT_H1),
        mint_fr15: pubkey_from_str(MINT_FR15),
        mint_db8: pubkey_from_str(MINT_DB8),
        usdc_coffre: pubkey_from_str(USDC_COFFRE),
        usdt_coffre: pubkey_from_str(USDT_COFFRE),
        pda_gt: pubkey_from_str(PDA_GT),
        coffre_7q: pubkey_from_str(COFFRE_7Q),
        pool_id: pubkey_from_str(POOL_ID),
        treasury_usdc_ata: pubkey_from_str(TREASURY_USDC_ATA),
        treasury_usdt_ata: pubkey_from_str(TREASURY_USDT_ATA),
        auto_collect_every_swaps: 10,
        fee_threshold_usd_micros: DEFAULT_FEE_THRESHOLD_USD_MICROS,
        keeper_authority: pubkey_from_str(MAIN_WALLET),
        usdc_mint: pubkey_from_str(USDC_MINT),
        usdt_mint: pubkey_from_str(USDT_MAIN_MINT),
        extra_payout_mint_0: Pubkey::default(),
        extra_payout_mint_1: Pubkey::default(),
        extra_payout_mint_2: Pubkey::default(),
        extra_payout_mint_3: Pubkey::default(),
        extra_payout_vault_ata_0: Pubkey::default(),
        extra_payout_vault_ata_1: Pubkey::default(),
        extra_payout_vault_ata_2: Pubkey::default(),
        extra_payout_vault_ata_3: Pubkey::default(),
        payout_threshold_usd_micros: 0,
        max_payout_usd_micros: 0,
        max_payout_per_window_usd_micros: 0,
        payout_window_secs: 0,
        payout_window_start: 0,
        payout_window_used_usd_micros: 0,
        lp_cashback_bps: 0,
        claim_cashback_bps: 0,
        bump,
    }
}

#[cfg(any(feature = "migration-full", feature = "migration-ledger"))]
fn build_config_from_legacy_v1(legacy: &ConfigLegacyV1, bump: u8) -> Config {
    let mut cfg = default_config_v2_state(legacy.admin, bump);
    cfg.true_cash = legacy.true_cash;
    cfg.cash_backed = legacy.cash_backed;
    cfg.real_peg = legacy.real_peg;
    cfg.sovereign = legacy.sovereign;
    cfg.cashback_bps = legacy.cashback_bps;
    cfg.reward_interval = legacy.reward_interval;
    cfg.allow_fallback_usdt = legacy.allow_fallback_usdt;
    cfg.token_value_usd_micros_default = legacy.token_value_usd_micros_default;
    cfg.treasury_value_usd_micros = legacy.treasury_value_usd_micros;
    cfg.enable_treasury = legacy.enable_treasury;
    cfg.enable_sjbc = legacy.enable_sjbc;
    cfg.enable_sjbc2 = legacy.enable_sjbc2;
    cfg.enable_sjbc3 = legacy.enable_sjbc3;
    cfg.enable_usdc = legacy.enable_usdc;
    cfg.enable_usdt_main = legacy.enable_usdt_main;
    cfg.enable_usdt_old = legacy.enable_usdt_old;
    cfg.enable_btc_portal = legacy.enable_btc_portal;
    cfg
}

#[cfg(any(feature = "migration-full", feature = "migration-ledger"))]
fn build_pool_from_legacy_v1(legacy: &PoolLegacyV1, bump: u8) -> Pool {
    Pool {
        owner: legacy.owner,
        base_mint: legacy.base_mint,
        quote_mint: legacy.quote_mint,
        base_vault: legacy.base_vault,
        quote_vault: legacy.quote_vault,
        lp_mint: legacy.lp_mint,
        fee_vault_base: Pubkey::default(),
        fee_vault_quote: Pubkey::default(),
        base_value_usd_micros: legacy.base_value_usd_micros,
        quote_value_usd_micros: legacy.quote_value_usd_micros,
        true_cash: legacy.true_cash,
        cash_backed: legacy.cash_backed,
        real_peg: legacy.real_peg,
        sovereign: legacy.sovereign,
        fee_bps: legacy.fee_bps,
        active: legacy.active,
        swap_cashback_bps: 0,
        protocol_fee_debt_usd_micros: 0,
        protocol_fee_debt_count: 0,
        protocol_fee_debt_last_ts: 0,
        created_at: 0,
        last_swap_ts: 0,
        total_base_volume: 0,
        total_quote_volume: 0,
        swap_count: 0,
        bump,
    }
}


// =========================
// PAO V3 HELPERS — SAFE (no ABI break)
// =========================
fn require_keeper<'info>(cfg: &Config, keeper: &Signer<'info>) -> Result<()> {
    require!(
        cfg.keeper_authority == keeper.key(),
        SterlingError::KeeperOnly
    );
    Ok(())
}

fn is_supported_payout_mint(cfg: &Config, mint: &Pubkey) -> bool {
    if *mint == cfg.usdc_mint {
        return true;
    }
    if *mint == cfg.usdt_mint {
        return true;
    }
    if *mint == cfg.extra_payout_mint_0 {
        return true;
    }
    if *mint == cfg.extra_payout_mint_1 {
        return true;
    }
    if *mint == cfg.extra_payout_mint_2 {
        return true;
    }
    if *mint == cfg.extra_payout_mint_3 {
        return true;
    }
    false
}

fn select_payout_mint_v3(cfg: &Config, route_hint: u8) -> Pubkey {
    // route_hint mapping (0..6):
    // 0 = AUTO: Stable A (usdc) -> Stable B (usdt) -> extra_0..extra_3
    // 1 = FORCE Stable A (cfg.usdc_mint)
    // 2 = FORCE Stable B (cfg.usdt_mint)
    // 3 = FORCE extra_0
    // 4 = FORCE extra_1
    // 5 = FORCE extra_2
    // 6 = FORCE extra_3

    // helper to perform AUTO selection
    let auto_select = |cfg: &Config| -> Pubkey {
        if cfg.usdc_mint != Pubkey::default() {
            return cfg.usdc_mint;
        }
        if cfg.usdt_mint != Pubkey::default() {
            return cfg.usdt_mint;
        }
        if cfg.extra_payout_mint_0 != Pubkey::default() {
            return cfg.extra_payout_mint_0;
        }
        if cfg.extra_payout_mint_1 != Pubkey::default() {
            return cfg.extra_payout_mint_1;
        }
        if cfg.extra_payout_mint_2 != Pubkey::default() {
            return cfg.extra_payout_mint_2;
        }
        if cfg.extra_payout_mint_3 != Pubkey::default() {
            return cfg.extra_payout_mint_3;
        }
        Pubkey::default()
    };

    // Determine candidate based on route_hint
    let candidate = match route_hint {
        1 => cfg.usdc_mint,
        2 => cfg.usdt_mint,
        3 => cfg.extra_payout_mint_0,
        4 => cfg.extra_payout_mint_1,
        5 => cfg.extra_payout_mint_2,
        6 => cfg.extra_payout_mint_3,
        _ => Pubkey::default(),
    };

    // If no explicit force (route_hint == 0 or candidate == default), do AUTO
    if route_hint == 0 {
        return auto_select(cfg);
    }

    // If candidate is not configured, fallback to AUTO.
    if candidate == Pubkey::default() {
        return auto_select(cfg);
    }

    // For route_hint == 2 (Stable B), allow force if configured regardless of allow_fallback_usdt
    // (candidate already equals cfg.usdt_mint, and we've checked it's supported)

    candidate
}

// =========================
// SBF Size Floor Padding (technical guarantee)
// =========================
mod sbf_size_pad {
    // Padding technique pour garantir SBF >= 470000 bytes (plancher).
    // Cible ~478K (non exacte).
    // Ajuster ce pad si le binaire passe sous le plancher ou doit etre fixe a une taille precise.
    const _SBF_SIZE_FLOOR_BYTES: usize = 470_000;
    const SBF_SIZE_PAD_BYTES: usize = 0;

    #[used]
    #[no_mangle]
    pub static SBF_SIZE_PAD: [u8; SBF_SIZE_PAD_BYTES] = [0u8; SBF_SIZE_PAD_BYTES];
}

fn apply_payout_limits(cfg: &mut Config, usd_micros: u64) -> Result<()> {
    require!(usd_micros > 0, SterlingError::InvalidAmount);
    enforce_ticket_cap_usd_micros(usd_micros)?;

    // ✅ 0 = unlimited
    if cfg.max_payout_usd_micros > 0 {
        require!(
            usd_micros <= cfg.max_payout_usd_micros,
            SterlingError::PayoutTooLarge
        );
    }

    // window only matters if max_payout_per_window_usd_micros > 0
    if cfg.max_payout_per_window_usd_micros > 0 {
        let now = Clock::get()?.unix_timestamp;
        if cfg.payout_window_start == 0 {
            cfg.payout_window_start = now;
            cfg.payout_window_used_usd_micros = 0;
        } else if cfg.payout_window_secs > 0 {
            let elapsed = now.saturating_sub(cfg.payout_window_start);
            if elapsed >= cfg.payout_window_secs as i64 {
                cfg.payout_window_start = now;
                cfg.payout_window_used_usd_micros = 0;
            }
        }

        let new_used = cfg.payout_window_used_usd_micros.saturating_add(usd_micros);
        require!(
            new_used <= cfg.max_payout_per_window_usd_micros,
            SterlingError::PayoutWindowExceeded
        );
        cfg.payout_window_used_usd_micros = new_used;
    }

    Ok(())
}

fn maybe_emit_payout_event_v3_safe(
    cfg: &mut Config,
    pool: Pubkey,
    payout_kind: u8,
    user: Pubkey,
    mint_in: Pubkey,
    amount_in: u64,
    usd_micros_trade: u64,
    cashback_bps: u16,
    nonce: u64,
    route_hint: u8,
) -> Result<()> {
    if cashback_bps == 0 {
        return Ok(());
    }

    // Do not fail user instructions if PAO V3 mints are not configured yet.
    if cfg.usdc_mint == Pubkey::default() && cfg.usdt_mint == Pubkey::default() {
        return Ok(());
    }

    let payout_mint = select_payout_mint_v3(cfg, route_hint);
    if payout_mint == Pubkey::default() {
        return Ok(());
    }
    require!(
        is_supported_payout_mint(cfg, &payout_mint),
        SterlingError::UnsupportedPayoutMint
    );

    let cashback_usd_micros: u64 = (usd_micros_trade as u128)
        .saturating_mul(cashback_bps as u128)
        .checked_div(10_000u128)
        .ok_or(SterlingError::MathOverflow)? as u64;
    enforce_ticket_cap_usd_micros(cashback_usd_micros)?;

    // Guard anti-zero: integer division can round cashback down to 0.
    // We never emit a payout event with usd_micros = 0.
    if cashback_usd_micros == 0 {
        return Ok(());
    }

    // 0 = no anti-dust threshold.
    if cfg.payout_threshold_usd_micros > 0 && cashback_usd_micros < cfg.payout_threshold_usd_micros
    {
        return Ok(());
    }

    apply_payout_limits(cfg, cashback_usd_micros)?;

    let destination_ata = get_associated_token_address(&user, &payout_mint);

    emit!(NeedPayoutEvent {
        pool,
        payout_mint,
        payout_kind,
        user,
        mint_in,
        amount_in,
        usd_micros: cashback_usd_micros,
        destination_ata,
        nonce,
        liquidity_policy: 1,
        route_hint,
        ts: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

fn _integer_sqrt_u128(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    let mut x0 = n / 2 + 1;
    let mut x1 = (x0 + n / x0) / 2;
    while x1 < x0 {
        x0 = x1;
        x1 = (x0 + n / x0) / 2;
    }
    x0
}

// =========================
// VALUE HELPERS (no float)
// =========================
fn pow10_u128(decimals: u8) -> u128 {
    let mut r: u128 = 1;
    let mut i: u8 = 0;
    while i < decimals {
        r = r.saturating_mul(10);
        i = i.saturating_add(1);
    }
    r
}

fn compute_usdc_settlement_amount(
    fee_amount: u64,
    valuation_usd_micros: u64,
    fee_decimals: u8,
) -> Result<u64> {
    let denom = pow10_u128(fee_decimals);
    require!(denom > 0, SterlingError::MathOverflow);

    let usdc_out = (fee_amount as u128)
        .saturating_mul(valuation_usd_micros as u128)
        .checked_div(denom)
        .ok_or(SterlingError::MathOverflow)?;

    Ok(u64::try_from(usdc_out).map_err(|_| SterlingError::MathOverflow)?)
}

#[cfg(test)]
fn ensure_usdc_settlement_liquidity(available_amount: u64, required_amount: u64) -> Result<()> {
    require!(
        available_amount >= required_amount,
        SterlingError::InsufficientUsdcSettlementLiquidity
    );
    Ok(())
}

fn is_expected_stable_account(
    account_key: Pubkey,
    account: &TokenAccount,
    expected_key: Pubkey,
    expected_mint: Pubkey,
    expected_owner: Pubkey,
) -> bool {
    account_key == expected_key && account.mint == expected_mint && account.owner == expected_owner
}

fn protocol_fee_payout_preference(config: &Config) -> Option<(Pubkey, Pubkey, u8)> {
    if config.treasury_usdc_ata != Pubkey::default() && config.usdc_mint != Pubkey::default() {
        return Some((config.usdc_mint, config.treasury_usdc_ata, 1));
    }
    if config.allow_fallback_usdt
        && config.treasury_usdt_ata != Pubkey::default()
        && config.usdt_mint != Pubkey::default()
    {
        return Some((config.usdt_mint, config.treasury_usdt_ata, 2));
    }
    None
}

fn protocol_fee_event_metadata(config: &Config) -> (Pubkey, Pubkey, u8) {
    if let Some(metadata) = protocol_fee_payout_preference(config) {
        return metadata;
    }
    if config.usdc_mint != Pubkey::default() {
        return (config.usdc_mint, config.treasury_usdc_ata, 1);
    }
    if config.usdt_mint != Pubkey::default() {
        return (config.usdt_mint, config.treasury_usdt_ata, 2);
    }
    (Pubkey::default(), Pubkey::default(), 0)
}

fn protocol_fee_materialization_metadata(
    config: &Config,
    route_hint: u8,
) -> Result<(Pubkey, Pubkey)> {
    let payout_mint = select_payout_mint_v3(config, route_hint);
    require!(
        payout_mint != Pubkey::default(),
        SterlingError::UnsupportedPayoutMint
    );

    if payout_mint == config.usdc_mint {
        require!(
            config.treasury_usdc_ata != Pubkey::default(),
            SterlingError::InvalidAccount
        );
        return Ok((payout_mint, config.treasury_usdc_ata));
    }
    if payout_mint == config.usdt_mint {
        require!(
            config.treasury_usdt_ata != Pubkey::default(),
            SterlingError::InvalidAccount
        );
        return Ok((payout_mint, config.treasury_usdt_ata));
    }

    err!(SterlingError::UnsupportedPayoutMint)
}

fn effective_claim_cashback_bps(config: &Config, stake_mint: Pubkey) -> u16 {
    let wants_lp_rate = stake_mint == config.lp_mint2 || stake_mint == config.lp_mint3;
    if wants_lp_rate && config.lp_cashback_bps > 0 {
        return config.lp_cashback_bps;
    }
    if config.claim_cashback_bps > 0 {
        return config.claim_cashback_bps;
    }
    config.cashback_bps
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SwapFeeSettlementAction {
    DirectUsdc,
    DirectUsdt,
    BurnToUsdc,
    BurnToUsdt,
    AccrueProtocolDebt,
}

fn is_canonical_stable_mint(
    mint: Pubkey,
    canonical_usdc_mint: Pubkey,
    canonical_usdt_mint: Pubkey,
) -> bool {
    mint == canonical_usdc_mint || mint == canonical_usdt_mint
}

fn should_defer_swap_fee_settlement_to_fee_vault(
    pool: &Pool,
    canonical_usdc_mint: Pubkey,
    canonical_usdt_mint: Pubkey,
) -> bool {
    !is_canonical_stable_mint(pool.base_mint, canonical_usdc_mint, canonical_usdt_mint)
        && !is_canonical_stable_mint(pool.quote_mint, canonical_usdc_mint, canonical_usdt_mint)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ClaimSettlementAction {
    DirectRewardVault,
    DirectRequestedCashRail,
    FallbackUsdtVault,
    EmitNeedPayoutEvent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ClaimDirectCashRail {
    ExtraConfigured,
    Usdc,
    Usdt,
    None,
}

const PROTOCOL_DEBT_LOT_EMPTY: u8 = 0;
const PROTOCOL_DEBT_LOT_OPEN: u8 = 1;
const PROTOCOL_DEBT_LOT_TICKETED: u8 = 2;
const PROTOCOL_DEBT_LOT_SETTLED: u8 = 3;

#[derive(Clone, Copy)]
struct ProtocolDebtLotPaymentSnapshot {
    mint_in: Pubkey,
    amount_in: u64,
    usd_micros: u64,
}

fn choose_swap_fee_settlement_action(
    fee_mint: Pubkey,
    canonical_usdc_mint: Pubkey,
    canonical_usdt_mint: Pubkey,
    usdc_rail_ready: bool,
    usdt_rail_ready: bool,
    usdc_available: u64,
    usdt_available: u64,
    usdc_out: u64,
) -> SwapFeeSettlementAction {
    if fee_mint == canonical_usdc_mint && usdc_rail_ready {
        return SwapFeeSettlementAction::DirectUsdc;
    }
    if fee_mint == canonical_usdt_mint && usdt_rail_ready {
        return SwapFeeSettlementAction::DirectUsdt;
    }
    if usdc_out > 0 && usdc_rail_ready && usdc_available >= usdc_out {
        return SwapFeeSettlementAction::BurnToUsdc;
    }
    if usdc_out > 0 && usdt_rail_ready && usdt_available >= usdc_out {
        return SwapFeeSettlementAction::BurnToUsdt;
    }
    SwapFeeSettlementAction::AccrueProtocolDebt
}

fn accrue_protocol_fee_debt(pool: &mut Pool, usd_micros: u64, now: i64) -> u64 {
    let debt_usd_micros = usd_micros.max(1);
    pool.protocol_fee_debt_usd_micros = pool
        .protocol_fee_debt_usd_micros
        .saturating_add(debt_usd_micros);
    pool.protocol_fee_debt_last_ts = now;
    pool.protocol_fee_debt_usd_micros
}

fn protocol_fee_debt_shard_count(usd_micros: u64) -> u64 {
    if usd_micros == 0 {
        return 0;
    }
    let debt_usd_micros = usd_micros;
    debt_usd_micros.saturating_add(DEFAULT_TICKET_CAP_USD_MICROS - 1)
        / DEFAULT_TICKET_CAP_USD_MICROS
}

fn reusable_protocol_debt_lot_index(ledger_data: &[u8]) -> Result<Option<usize>> {
    for index in 0..PROTOCOL_DEBT_LEDGER_SLOTS {
        let start = protocol_debt_lot_offset(index);
        let end = start + ProtocolDebtLot::LEN;
        require!(end <= ledger_data.len(), SterlingError::InvalidAccount);
        let lot = ProtocolDebtLot::try_from_slice(&ledger_data[start..end])
            .map_err(|_| error!(SterlingError::InvalidAccount))?;
        if lot.status == PROTOCOL_DEBT_LOT_EMPTY || lot.status == PROTOCOL_DEBT_LOT_SETTLED {
            return Ok(Some(index));
        }
    }

    Ok(None)
}

fn find_protocol_debt_lot_by_nonce_mut(
    ledger: &mut ProtocolDebtLedger,
    nonce: u64,
) -> Result<&mut ProtocolDebtLot> {
    ledger
        .lots
        .iter_mut()
        .find(|lot| lot.nonce == nonce && lot.status != PROTOCOL_DEBT_LOT_EMPTY)
        .ok_or_else(|| error!(SterlingError::InvalidState))
}

fn mark_protocol_debt_lot_settled(
    pool: &mut Pool,
    lot: &mut ProtocolDebtLot,
    now: i64,
) -> Result<()> {
    require!(lot.usd_micros > 0, SterlingError::InvalidAmount);
    require!(
        lot.status == PROTOCOL_DEBT_LOT_OPEN || lot.status == PROTOCOL_DEBT_LOT_TICKETED,
        SterlingError::InvalidState
    );
    require!(
        pool.protocol_fee_debt_usd_micros >= lot.usd_micros,
        SterlingError::InvalidAmount
    );

    pool.protocol_fee_debt_usd_micros -= lot.usd_micros;
    pool.protocol_fee_debt_count = pool.protocol_fee_debt_count.saturating_sub(1);
    pool.protocol_fee_debt_last_ts = now;
    lot.usd_micros = 0;
    lot.status = PROTOCOL_DEBT_LOT_SETTLED;
    Ok(())
}

fn compute_claim_reward_usd_micros(
    staked_amount_atoms: u64,
    stake_mint_decimals: u8,
    stake_value_usd_micros_per_1ui: u64,
    cashback_bps: u16,
) -> Result<u64> {
    let staked_usd_micros = atoms_to_usd_micros(
        staked_amount_atoms,
        stake_mint_decimals,
        stake_value_usd_micros_per_1ui,
    )?;
    let reward_usd_micros: u64 = (staked_usd_micros as u128)
        .saturating_mul(cashback_bps as u128)
        .checked_div(10_000u128)
        .ok_or(SterlingError::MathOverflow)? as u64;
    Ok(reward_usd_micros)
}

fn requested_claim_direct_cash_rail(
    payout_mint: Pubkey,
    canonical_usdc_mint: Pubkey,
    canonical_usdt_mint: Pubkey,
    extra_payout_mint_0: Pubkey,
    extra_payout_mint_1: Pubkey,
    extra_payout_mint_2: Pubkey,
    extra_payout_mint_3: Pubkey,
) -> ClaimDirectCashRail {
    if payout_mint == canonical_usdc_mint {
        return ClaimDirectCashRail::Usdc;
    }
    if payout_mint == canonical_usdt_mint {
        return ClaimDirectCashRail::Usdt;
    }
    if (extra_payout_mint_0 != Pubkey::default() && payout_mint == extra_payout_mint_0)
        || (extra_payout_mint_1 != Pubkey::default() && payout_mint == extra_payout_mint_1)
        || (extra_payout_mint_2 != Pubkey::default() && payout_mint == extra_payout_mint_2)
        || (extra_payout_mint_3 != Pubkey::default() && payout_mint == extra_payout_mint_3)
    {
        return ClaimDirectCashRail::ExtraConfigured;
    }
    ClaimDirectCashRail::None
}

fn canonical_claim_usdt_fallback_vault_address(canonical_usdt_mint: Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"reward_vault", canonical_usdt_mint.as_ref()], &crate::ID).0
}

#[derive(Clone)]
struct ClaimRemainingAccountsLayout<'info> {
    stake_value_usd_micros_per_1ui: u64,
    configured_cash_rail_vault: Option<AccountInfo<'info>>,
    configured_cash_rail_amount: u64,
    fallback_usdt_vault: Option<AccountInfo<'info>>,
    fallback_usdt_vault_amount: u64,
}

fn claim_value_registry_pda(stake_mint: Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"value_registry", stake_mint.as_ref()], &crate::ID).0
}

fn parse_claim_extra_cash_rail_vault<'info>(
    config: &Config,
    payout_mint_key: Pubkey,
    account_info: &'info AccountInfo<'info>,
) -> Result<(AccountInfo<'info>, u64)> {
    let extra_vault = Account::<TokenAccount>::try_from(account_info)?;
    let (_, expected_mint, expected_vault) =
        configured_alt_payout_rail_from_vault(config, account_info.key())
            .ok_or_else(|| error!(SterlingError::InvalidAccount))?;
    require!(
        expected_mint == payout_mint_key,
        SterlingError::InvalidAccount
    );
    require!(
        expected_vault == account_info.key(),
        SterlingError::InvalidAccount
    );
    require!(
        extra_vault.mint == payout_mint_key,
        SterlingError::InvalidAccount
    );
    Ok((account_info.clone(), extra_vault.amount))
}

fn parse_claim_fallback_usdt_pair<'info>(
    config: &Config,
    user_key: Pubkey,
    user_usdt_main_ata_key: Pubkey,
    expected_usdt_fallback_vault: Pubkey,
    fallback_vault_info: &'info AccountInfo<'info>,
    user_usdt_ata_info: &'info AccountInfo<'info>,
) -> Result<(AccountInfo<'info>, u64)> {
    let fallback_vault = Account::<TokenAccount>::try_from(fallback_vault_info)?;
    let user_usdt_ata = Account::<TokenAccount>::try_from(user_usdt_ata_info)?;

    require!(
        fallback_vault_info.key() == expected_usdt_fallback_vault,
        SterlingError::InvalidAccount
    );
    require!(
        fallback_vault.mint == config.usdt_mint,
        SterlingError::InvalidAccount
    );
    require!(
        user_usdt_ata.owner == user_key,
        SterlingError::InvalidAccount
    );
    require!(
        user_usdt_ata_info.key() == user_usdt_main_ata_key,
        SterlingError::InvalidAccount
    );
    require!(
        user_usdt_ata.mint == config.usdt_mint,
        SterlingError::InvalidAccount
    );

    Ok((fallback_vault_info.clone(), fallback_vault.amount))
}

fn parse_claim_remaining_accounts<'info>(
    config: &Config,
    stake_mint_key: Pubkey,
    payout_mint_key: Pubkey,
    user_key: Pubkey,
    user_usdt_main_ata_key: Pubkey,
    remaining_accounts: &'info [AccountInfo<'info>],
    default_stake_value_usd_micros_per_1ui: u64,
) -> Result<ClaimRemainingAccountsLayout<'info>> {
    let expected_value_registry = claim_value_registry_pda(stake_mint_key);
    let mut stake_value_usd_micros_per_1ui = default_stake_value_usd_micros_per_1ui;
    let mut remaining_offset = 0usize;

    if let Some(candidate) = remaining_accounts.first() {
        if candidate.key() == expected_value_registry {
            require!(candidate.owner == &crate::ID, SterlingError::InvalidAccount);
            let data = candidate.try_borrow_data()?;
            let mut value_registry_bytes: &[u8] = &data;
            let value_registry = ValueRegistry::try_deserialize(&mut value_registry_bytes)
                .map_err(|_| error!(SterlingError::InvalidAccount))?;
            require!(
                value_registry.mint == stake_mint_key,
                SterlingError::InvalidAccount
            );
            stake_value_usd_micros_per_1ui = value_registry.value_usd_micros;
            remaining_offset = 1;
        }
    }

    let expected_usdt_fallback_vault =
        canonical_claim_usdt_fallback_vault_address(config.usdt_mint);
    let tail = &remaining_accounts[remaining_offset..];

    let (
        configured_cash_rail_vault,
        configured_cash_rail_amount,
        fallback_usdt_vault,
        fallback_usdt_vault_amount,
    ) = match tail {
        [] => (None, 0, None, 0),
        [single] => {
            let (extra_vault, extra_amount) =
                parse_claim_extra_cash_rail_vault(config, payout_mint_key, single)?;
            (Some(extra_vault), extra_amount, None, 0)
        }
        [first, second] => {
            require!(
                first.key() == expected_usdt_fallback_vault,
                SterlingError::InvalidAccount
            );
            let (fallback_vault, fallback_amount) = parse_claim_fallback_usdt_pair(
                config,
                user_key,
                user_usdt_main_ata_key,
                expected_usdt_fallback_vault,
                first,
                second,
            )?;
            (None, 0, Some(fallback_vault), fallback_amount)
        }
        [first, second, third] => {
            let (extra_vault, extra_amount) =
                parse_claim_extra_cash_rail_vault(config, payout_mint_key, first)?;
            let (fallback_vault, fallback_amount) = parse_claim_fallback_usdt_pair(
                config,
                user_key,
                user_usdt_main_ata_key,
                expected_usdt_fallback_vault,
                second,
                third,
            )?;
            (
                Some(extra_vault),
                extra_amount,
                Some(fallback_vault),
                fallback_amount,
            )
        }
        _ => return err!(SterlingError::InvalidAccount),
    };

    Ok(ClaimRemainingAccountsLayout {
        stake_value_usd_micros_per_1ui,
        configured_cash_rail_vault,
        configured_cash_rail_amount,
        fallback_usdt_vault,
        fallback_usdt_vault_amount,
    })
}

fn choose_claim_settlement_action(
    reward_vault_amount: u64,
    reward_amount: u64,
    requested_direct_cash_rail: ClaimDirectCashRail,
    usdc_cash_rail_amount: u64,
    reward_amount_usdc: u64,
    usdt_cash_rail_amount: u64,
    reward_amount_usdt: u64,
    configured_cash_rail_amount: u64,
    reward_amount_requested: u64,
    allow_fallback_usdt: bool,
    fallback_usdt_vault_amount: u64,
    reward_amount_fallback_usdt: u64,
) -> ClaimSettlementAction {
    if reward_vault_amount >= reward_amount {
        return ClaimSettlementAction::DirectRewardVault;
    }
    match requested_direct_cash_rail {
        ClaimDirectCashRail::Usdc if usdc_cash_rail_amount >= reward_amount_usdc => {
            return ClaimSettlementAction::DirectRequestedCashRail;
        }
        ClaimDirectCashRail::Usdt if usdt_cash_rail_amount >= reward_amount_usdt => {
            return ClaimSettlementAction::DirectRequestedCashRail;
        }
        ClaimDirectCashRail::ExtraConfigured
            if configured_cash_rail_amount >= reward_amount_requested =>
        {
            return ClaimSettlementAction::DirectRequestedCashRail;
        }
        ClaimDirectCashRail::ExtraConfigured
        | ClaimDirectCashRail::Usdc
        | ClaimDirectCashRail::Usdt
        | ClaimDirectCashRail::None => {}
    }
    if allow_fallback_usdt && fallback_usdt_vault_amount >= reward_amount_fallback_usdt {
        return ClaimSettlementAction::FallbackUsdtVault;
    }
    ClaimSettlementAction::EmitNeedPayoutEvent
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SovereignRouteExecutionKind {
    LiveSwap,
    DirectTransfer,
    BurnRelease,
    BufferedReceipt,
    BufferedDebt,
}

fn should_prefer_live_sovereign_route<'info>(
    source_kind: u8,
    remaining_accounts: &[AccountInfo<'info>],
) -> bool {
    match source_kind {
        1 | 2 | 3 => {
            matches!(remaining_accounts.len(), 7..=9)
                && remaining_accounts
                    .get(3)
                    .map(|account| account.owner == &crate::ID)
                    .unwrap_or(false)
        }
        4 | 5 => {
            matches!(remaining_accounts.len(), 8 | 9)
                && remaining_accounts
                    .get(3)
                    .map(|account| account.owner == &crate::ID)
                    .unwrap_or(false)
        }
        6 => {
            matches!(remaining_accounts.len(), 9 | 10)
                && remaining_accounts
                    .get(4)
                    .map(|account| account.owner == &crate::ID)
                    .unwrap_or(false)
        }
        _ => false,
    }
}

fn choose_sovereign_route_execution_kind(
    source_kind: u8,
    prefer_live_route: bool,
) -> Result<SovereignRouteExecutionKind> {
    let execution_kind = match source_kind {
        1 | 3 => {
            if prefer_live_route {
                SovereignRouteExecutionKind::LiveSwap
            } else {
                SovereignRouteExecutionKind::BurnRelease
            }
        }
        2 => {
            if prefer_live_route {
                SovereignRouteExecutionKind::LiveSwap
            } else {
                SovereignRouteExecutionKind::DirectTransfer
            }
        }
        4 | 5 => {
            if prefer_live_route {
                SovereignRouteExecutionKind::LiveSwap
            } else {
                SovereignRouteExecutionKind::BufferedReceipt
            }
        }
        6 => {
            if prefer_live_route {
                SovereignRouteExecutionKind::LiveSwap
            } else {
                SovereignRouteExecutionKind::BufferedDebt
            }
        }
        _ => return err!(SterlingError::InvalidAccount),
    };

    Ok(execution_kind)
}

fn settle_protocol_debt_lots_in_ledger(
    pool: &mut Pool,
    ledger: &mut ProtocolDebtLedger,
    paid_usd_micros: u64,
    now: i64,
) -> Result<u64> {
    require!(
        pool.protocol_fee_debt_usd_micros >= paid_usd_micros,
        SterlingError::InvalidAmount
    );

    let mut remaining = paid_usd_micros;

    for lot in ledger.lots.iter_mut() {
        if remaining == 0 {
            break;
        }
        if !matches!(
            lot.status,
            PROTOCOL_DEBT_LOT_OPEN | PROTOCOL_DEBT_LOT_TICKETED
        ) || lot.usd_micros == 0
        {
            continue;
        }

        if remaining >= lot.usd_micros {
            remaining -= lot.usd_micros;
            mark_protocol_debt_lot_settled(pool, lot, now)?;
            continue;
        }

        lot.usd_micros -= remaining;
        pool.protocol_fee_debt_usd_micros -= remaining;
        pool.protocol_fee_debt_last_ts = now;
        remaining = 0;
        break;
    }

    if remaining > 0 {
        require!(
            ledger.overflow_usd_micros >= remaining,
            SterlingError::InvalidAmount
        );
        let overflow_count_before = protocol_fee_debt_shard_count(ledger.overflow_usd_micros);
        ledger.overflow_usd_micros -= remaining;
        let overflow_count_after = protocol_fee_debt_shard_count(ledger.overflow_usd_micros);
        pool.protocol_fee_debt_count = pool
            .protocol_fee_debt_count
            .saturating_sub(overflow_count_before.saturating_sub(overflow_count_after));
        pool.protocol_fee_debt_usd_micros -= remaining;
        pool.protocol_fee_debt_last_ts = now;
        remaining = 0;
    }

    require!(remaining == 0, SterlingError::InvalidAmount);
    ledger.last_ts = now;
    Ok(paid_usd_micros)
}

#[inline(never)]
fn load_protocol_debt_lot_payment_snapshot<'a, 'b>(
    pool_info: &AccountInfo<'a>,
    ledger_info: &AccountInfo<'b>,
    debt_lot_nonce: u64,
) -> Result<ProtocolDebtLotPaymentSnapshot> {
    require!(pool_info.owner == &crate::ID, SterlingError::InvalidAccount);
    require!(
        ledger_info.owner == &crate::ID,
        SterlingError::InvalidAccount
    );

    let ledger_data = ledger_info.try_borrow_data()?;
    let (_, lot, ledger_pool_key) =
        load_protocol_debt_lot_from_ledger_data(&ledger_data, debt_lot_nonce)?;
    require!(
        ledger_pool_key == pool_info.key(),
        SterlingError::InvalidAccount
    );

    require!(
        lot.status == PROTOCOL_DEBT_LOT_OPEN || lot.status == PROTOCOL_DEBT_LOT_TICKETED,
        SterlingError::InvalidState
    );

    Ok(ProtocolDebtLotPaymentSnapshot {
        mint_in: lot.mint_in,
        amount_in: lot.amount_in,
        usd_micros: lot.usd_micros,
    })
}

#[inline(never)]
fn apply_protocol_debt_lot_settlement<'a, 'b>(
    pool_info: AccountInfo<'a>,
    ledger_info: AccountInfo<'b>,
    debt_lot_nonce: u64,
) -> Result<()> {
    require!(pool_info.owner == &crate::ID, SterlingError::InvalidAccount);
    require!(
        ledger_info.owner == &crate::ID,
        SterlingError::InvalidAccount
    );

    let mut pool_data = pool_info.try_borrow_mut_data()?;
    let mut pool_bytes: &[u8] = &pool_data;
    let mut pool = Pool::try_deserialize(&mut pool_bytes)?;

    let mut ledger_data = ledger_info.try_borrow_mut_data()?;
    let (lot_index, mut lot, ledger_pool_key) =
        load_protocol_debt_lot_from_ledger_data(&ledger_data, debt_lot_nonce)?;
    require!(
        ledger_pool_key == pool_info.key(),
        SterlingError::InvalidAccount
    );
    let now = Clock::get()?.unix_timestamp;
    mark_protocol_debt_lot_settled(&mut pool, &mut lot, now)?;

    let mut pool_out: &mut [u8] = &mut pool_data;
    pool.try_serialize(&mut pool_out)?;
    store_protocol_debt_lot_into_ledger_data(&mut ledger_data, lot_index, &lot)?;
    store_protocol_debt_ledger_last_ts(&mut ledger_data, now)?;

    Ok(())
}

fn record_protocol_fee_debt<'info>(
    config: &Account<'info, Config>,
    pool: &mut Account<'info, Pool>,
    ledger_info: &AccountInfo<'_>,
    fee_mint: Pubkey,
    fee_amount: u64,
    usd_micros: u64,
) -> Result<u64> {
    let now = Clock::get()?.unix_timestamp;
    // Si la conversion stable arrondit a 0, on conserve tout de meme une dette minimale
    // payable de 1 USD micro pour que le swap reste non bloquant sans perdre la creance.
    let debt_usd_micros = usd_micros.max(1);
    let logical_ticket_count = protocol_fee_debt_shard_count(debt_usd_micros);
    let (payout_mint, destination_ata, route_hint) = protocol_fee_event_metadata(config);
    let pool_key = pool.key();
    let total_usd_micros = accrue_protocol_fee_debt(pool, debt_usd_micros, now);
    pool.protocol_fee_debt_count = pool
        .protocol_fee_debt_count
        .saturating_add(logical_ticket_count);

    require!(ledger_info.owner == &crate::ID, SterlingError::InvalidAccount);
    let mut ledger_data = ledger_info.try_borrow_mut_data()?;
    require!(
        ledger_data.len() >= ProtocolDebtLedger::LEN,
        SterlingError::InvalidAccount
    );

    let ledger_pool = Pubkey::new_from_array(
        ledger_data[8..40]
            .try_into()
            .map_err(|_| error!(SterlingError::InvalidAccount))?,
    );
    require!(ledger_pool == pool_key, SterlingError::InvalidAccount);

    let mut next_nonce_bytes = [0u8; 8];
    next_nonce_bytes.copy_from_slice(&ledger_data[40..48]);
    let mut next_nonce = u64::from_le_bytes(next_nonce_bytes);

    let mut overflow_bytes = [0u8; 8];
    overflow_bytes.copy_from_slice(&ledger_data[48..56]);
    let mut overflow_usd_micros = u64::from_le_bytes(overflow_bytes);
    store_protocol_debt_ledger_last_ts(&mut ledger_data, now)?;

    emit!(ProtocolFeeDebtEvent {
        pool: pool_key,
        payout_mint,
        payout_kind: 5,
        beneficiary: config.main_wallet,
        mint_in: fee_mint,
        amount_in: fee_amount,
        usd_micros: debt_usd_micros,
        total_usd_micros,
        destination_ata,
        nonce: next_nonce,
        route_hint,
        ts: now,
    });

    let mut remaining = debt_usd_micros;

    while remaining > 0 {
        let shard_usd_micros = remaining.min(DEFAULT_TICKET_CAP_USD_MICROS);
        if let Some(lot_index) = reusable_protocol_debt_lot_index(&ledger_data)? {
            let nonce = next_nonce;
            next_nonce = next_nonce.saturating_add(1);

            let lot = ProtocolDebtLot {
                nonce,
                mint_in: fee_mint,
                amount_in: fee_amount,
                usd_micros: shard_usd_micros,
                created_ts: now,
                escrow_mint: fee_mint,
                escrow_ata: expected_live_escrow_ata(
                    expected_protocol_debt_lot_escrow_authority(pool_key, nonce).0,
                    fee_mint,
                ),
                escrow_amount_locked: fee_amount,
                escrow_bump: expected_protocol_debt_lot_escrow_authority(pool_key, nonce).1,
                funding_state: SOVEREIGN_ESCROW_STATE_REQUESTED,
                route_hint,
                status: PROTOCOL_DEBT_LOT_OPEN,
            };
            store_protocol_debt_lot_into_ledger_data(&mut ledger_data, lot_index, &lot)?;

            remaining -= shard_usd_micros;

            emit!(NeedPayoutEvent {
                pool: pool_key,
                payout_mint,
                payout_kind: 5,
                user: config.main_wallet,
                mint_in: fee_mint,
                amount_in: fee_amount,
                usd_micros: shard_usd_micros,
                destination_ata,
                nonce,
                liquidity_policy: 1,
                route_hint,
                ts: now,
            });

            continue;
        }

        overflow_usd_micros = overflow_usd_micros.saturating_add(remaining);
        break;
    }

    ledger_data[40..48].copy_from_slice(&next_nonce.to_le_bytes());
    ledger_data[48..56].copy_from_slice(&overflow_usd_micros.to_le_bytes());

    Ok(debt_usd_micros)
}

fn validate_claimable_ticket(
    ticket: &PayoutTicket,
    nonce: u64,
    expected_user: Pubkey,
    expected_pool: Pubkey,
) -> Result<()> {
    require!(expected_user == ticket.user, SterlingError::InvalidAccount);
    require!(expected_pool == ticket.pool, SterlingError::InvalidAccount);
    require!(ticket.created_ts != 0, SterlingError::InvalidState);
    require!(ticket.settled_ts == 0, SterlingError::InvalidState);
    require!(
        ticket.status == PAYOUT_TICKET_STATUS_REQUESTED,
        SterlingError::InvalidState
    );
    require!(ticket.nonce == nonce, SterlingError::InvalidState);
    require!(ticket.usd_micros > 0, SterlingError::InvalidAmount);
    enforce_ticket_cap_usd_micros(ticket.usd_micros)
}

fn validate_materialized_protocol_debt_ticket(
    ticket: &PayoutTicket,
    expected_pool: Pubkey,
    expected_user: Pubkey,
    expected_payout_mint: Pubkey,
    expected_destination_ata: Pubkey,
    lot: &ProtocolDebtLot,
    nonce: u64,
) -> Result<()> {
    require!(ticket.pool == expected_pool, SterlingError::InvalidAccount);
    require!(
        ticket.payout_mint == expected_payout_mint,
        SterlingError::InvalidAccount
    );
    require!(ticket.payout_kind == 5, SterlingError::InvalidAccount);
    require!(ticket.user == expected_user, SterlingError::InvalidAccount);
    require!(ticket.mint_in == lot.mint_in, SterlingError::InvalidAccount);
    require!(
        ticket.amount_in == lot.amount_in,
        SterlingError::InvalidAmount
    );
    require!(
        ticket.usd_micros == lot.usd_micros,
        SterlingError::InvalidAmount
    );
    require!(
        ticket.destination_ata == expected_destination_ata,
        SterlingError::InvalidAccount
    );
    require!(
        ticket.escrow_mint == lot.escrow_mint,
        SterlingError::InvalidAccount
    );
    require!(
        ticket.escrow_amount_locked == lot.escrow_amount_locked,
        SterlingError::InvalidAmount
    );
    require!(ticket.nonce == nonce, SterlingError::InvalidState);
    require!(
        ticket.route_hint == lot.route_hint,
        SterlingError::InvalidState
    );
    require!(ticket.created_ts != 0, SterlingError::InvalidState);
    require!(ticket.settled_ts == 0, SterlingError::InvalidState);
    require!(
        ticket.status == PAYOUT_TICKET_STATUS_REQUESTED,
        SterlingError::InvalidState
    );
    require!(
        ticket.funding_state == SOVEREIGN_ESCROW_STATE_REQUESTED,
        SterlingError::InvalidState
    );
    Ok(())
}

fn validate_protocol_debt_ticket_materialization(
    ticket: &PayoutTicket,
    expected_pool: Pubkey,
    expected_user: Pubkey,
    expected_payout_mint: Pubkey,
    expected_destination_ata: Pubkey,
    lot: &ProtocolDebtLot,
    nonce: u64,
    require_created: bool,
) -> Result<()> {
    require!(ticket.pool == expected_pool, SterlingError::InvalidAccount);
    require!(
        ticket.payout_mint == expected_payout_mint,
        SterlingError::InvalidAccount
    );
    require!(ticket.payout_kind == 5, SterlingError::InvalidAccount);
    require!(ticket.user == expected_user, SterlingError::InvalidAccount);
    require!(ticket.mint_in == lot.mint_in, SterlingError::InvalidAccount);
    require!(
        ticket.amount_in == lot.amount_in,
        SterlingError::InvalidAmount
    );
    require!(
        ticket.usd_micros == lot.usd_micros,
        SterlingError::InvalidAmount
    );
    require!(
        ticket.destination_ata == expected_destination_ata,
        SterlingError::InvalidAccount
    );
    require!(
        ticket.escrow_mint == lot.escrow_mint,
        SterlingError::InvalidAccount
    );
    require!(
        ticket.escrow_ata == lot.escrow_ata,
        SterlingError::InvalidAccount
    );
    require!(
        ticket.escrow_amount_locked == lot.escrow_amount_locked,
        SterlingError::InvalidAmount
    );
    require!(ticket.nonce == nonce, SterlingError::InvalidState);
    require!(
        ticket.status == PAYOUT_TICKET_STATUS_REQUESTED,
        SterlingError::InvalidState
    );
    require!(
        ticket.route_hint == lot.route_hint,
        SterlingError::InvalidState
    );
    require!(
        ticket.funding_state == SOVEREIGN_ESCROW_STATE_REQUESTED,
        SterlingError::InvalidState
    );

    if require_created {
        require!(ticket.created_ts != 0, SterlingError::InvalidState);
        require!(ticket.settled_ts == 0, SterlingError::InvalidState);
    }

    Ok(())
}

fn require_protocol_fee_debt_payment_proof(proof_sig: &[u8; 64]) -> Result<()> {
    require!(*proof_sig != [0u8; 64], SterlingError::InvalidAccount);
    let actual = settlement_proof_kind_from_sig(proof_sig);
    require!(
        matches!(
            actual,
            Some(SettlementProofKind::LiveFundingExecution)
                | Some(SettlementProofKind::ProviderPayoutReceipt)
        ),
        SterlingError::InvalidState
    );
    Ok(())
}

fn settle_swap_fee_with_stable_fallback_and_debt<'info>(
    config: &Account<'info, Config>,
    pool: &mut Account<'info, Pool>,
    fee_vault: &Account<'info, TokenAccount>,
    fee_mint: &Account<'info, Mint>,
    fee_asset_registry: &Account<'info, AssetRegistry>,
    fee_amount: u64,
    token_program: &Program<'info, Token>,
    signer_seeds: &[&[&[u8]]],
    usdc_coffre_ata: &Account<'info, TokenAccount>,
    treasury_usdc_ata: &Account<'info, TokenAccount>,
    usdt_coffre_ata: &Account<'info, TokenAccount>,
    treasury_usdt_ata: &Account<'info, TokenAccount>,
    extra_fee_targets: &[StableFinalOutputTarget<'info>],
) -> Result<(u64, bool)> {
    // Rail robuste:
    // - tentative USDC immediate;
    // - fallback USDT immediate si autorise et disponible;
    // - sinon conservation de la fee en vault + emission d'une creance protocole.
    require!(fee_amount > 0, SterlingError::InvalidAmount);
    let canonical_usdc_mint = pubkey_from_str(USDC_MINT);
    let canonical_usdt_mint = pubkey_from_str(USDT_MAIN_MINT);
    require!(
        fee_vault.mint == fee_mint.key(),
        SterlingError::InvalidAccount
    );
    require!(
        fee_asset_registry.mint == fee_mint.key(),
        SterlingError::FeeAssetRegistryMissing
    );
    require!(
        fee_asset_registry.active,
        SterlingError::FeeAssetRegistryInactive
    );
    require!(
        fee_asset_registry.decimals == fee_mint.decimals,
        SterlingError::InvalidAccount
    );

    let now = Clock::get()?.unix_timestamp;
    let usdc_rail_ready = is_expected_stable_account(
        treasury_usdc_ata.key(),
        treasury_usdc_ata,
        config.treasury_usdc_ata,
        canonical_usdc_mint,
        config.main_wallet,
    ) && is_expected_stable_account(
        usdc_coffre_ata.key(),
        usdc_coffre_ata,
        config.usdc_coffre,
        canonical_usdc_mint,
        config.key(),
    );
    let usdt_rail_ready = config.allow_fallback_usdt
        && is_expected_stable_account(
            treasury_usdt_ata.key(),
            treasury_usdt_ata,
            config.treasury_usdt_ata,
            canonical_usdt_mint,
            config.main_wallet,
        )
        && is_expected_stable_account(
            usdt_coffre_ata.key(),
            usdt_coffre_ata,
            config.usdt_coffre,
            canonical_usdt_mint,
            config.key(),
        );

    let usdc_out = compute_usdc_settlement_amount(
        fee_amount,
        fee_asset_registry.valuation_usd_micros,
        fee_mint.decimals,
    )?;
    let pool_key = pool.key();

    if should_defer_swap_fee_settlement_to_fee_vault(pool, canonical_usdc_mint, canonical_usdt_mint)
    {
        emit!(FeeRetainedInVaultEvent {
            pool: pool_key,
            fee_vault: fee_vault.key(),
            mint: fee_mint.key(),
            amount: fee_amount,
            value_usd_micros: usdc_out,
            ts: now,
        });
        return Ok((usdc_out, false));
    }

    if let Some(target) = extra_fee_targets
        .iter()
        .find(|target| target.payout_mint == fee_mint.key())
    {
        token::transfer(
            CpiContext::new(
                token_program.to_account_info(),
                Transfer {
                    from: fee_vault.to_account_info(),
                    to: target.destination_info.clone(),
                    authority: config.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            fee_amount,
        )?;

        emit!(FeeCollectedToTreasuryEvent {
            pool: pool_key,
            fee_vault: fee_vault.key(),
            mint: fee_mint.key(),
            amount: fee_amount,
            treasury_ata: target.destination_key,
            ts: now,
        });
        return Ok((fee_amount, false));
    }

    let settlement_action = choose_swap_fee_settlement_action(
        fee_mint.key(),
        canonical_usdc_mint,
        canonical_usdt_mint,
        usdc_rail_ready,
        usdt_rail_ready,
        usdc_coffre_ata.amount,
        usdt_coffre_ata.amount,
        usdc_out,
    );

    if settlement_action == SwapFeeSettlementAction::DirectUsdc {
        token::transfer(
            CpiContext::new(
                token_program.to_account_info(),
                Transfer {
                    from: fee_vault.to_account_info(),
                    to: treasury_usdc_ata.to_account_info(),
                    authority: config.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            fee_amount,
        )?;

        emit!(FeeCollectedToTreasuryEvent {
            pool: pool_key,
            fee_vault: fee_vault.key(),
            mint: fee_mint.key(),
            amount: fee_amount,
            treasury_ata: treasury_usdc_ata.key(),
            ts: now,
        });
        return Ok((fee_amount, false));
    }

    if settlement_action == SwapFeeSettlementAction::DirectUsdt {
        token::transfer(
            CpiContext::new(
                token_program.to_account_info(),
                Transfer {
                    from: fee_vault.to_account_info(),
                    to: treasury_usdt_ata.to_account_info(),
                    authority: config.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            fee_amount,
        )?;

        emit!(FeeCollectedToTreasuryEvent {
            pool: pool_key,
            fee_vault: fee_vault.key(),
            mint: fee_mint.key(),
            amount: fee_amount,
            treasury_ata: treasury_usdt_ata.key(),
            ts: now,
        });
        return Ok((fee_amount, false));
    }

    if settlement_action == SwapFeeSettlementAction::AccrueProtocolDebt {
        emit!(FeeRetainedInVaultEvent {
            pool: pool_key,
            fee_vault: fee_vault.key(),
            mint: fee_mint.key(),
            amount: fee_amount,
            value_usd_micros: usdc_out,
            ts: now,
        });
        return Ok((usdc_out, false));
    }

    if settlement_action == SwapFeeSettlementAction::BurnToUsdc {
        token::burn(
            CpiContext::new(
                token_program.to_account_info(),
                Burn {
                    mint: fee_mint.to_account_info(),
                    from: fee_vault.to_account_info(),
                    authority: config.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            fee_amount,
        )?;

        token::transfer(
            CpiContext::new(
                token_program.to_account_info(),
                Transfer {
                    from: usdc_coffre_ata.to_account_info(),
                    to: treasury_usdc_ata.to_account_info(),
                    authority: config.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            usdc_out,
        )?;

        emit!(FeeConvertedToUsdcEvent {
            pool: pool_key,
            burned_mint: fee_mint.key(),
            burned_amount: fee_amount,
            usdc_released: usdc_out,
            usdc_coffre: usdc_coffre_ata.key(),
            treasury_usdc_ata: treasury_usdc_ata.key(),
            ts: now,
        });

        return Ok((usdc_out, false));
    }

    if settlement_action == SwapFeeSettlementAction::BurnToUsdt {
        token::burn(
            CpiContext::new(
                token_program.to_account_info(),
                Burn {
                    mint: fee_mint.to_account_info(),
                    from: fee_vault.to_account_info(),
                    authority: config.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            fee_amount,
        )?;

        token::transfer(
            CpiContext::new(
                token_program.to_account_info(),
                Transfer {
                    from: usdt_coffre_ata.to_account_info(),
                    to: treasury_usdt_ata.to_account_info(),
                    authority: config.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            usdc_out,
        )?;

        emit!(FeeConvertedToStableEvent {
            pool: pool_key,
            burned_mint: fee_mint.key(),
            burned_amount: fee_amount,
            stable_mint: canonical_usdt_mint,
            stable_released: usdc_out,
            stable_coffre: usdt_coffre_ata.key(),
            treasury_stable_ata: treasury_usdt_ata.key(),
            ts: now,
        });

        return Ok((usdc_out, false));
    }

    emit!(FeeRetainedInVaultEvent {
        pool: pool_key,
        fee_vault: fee_vault.key(),
        mint: fee_mint.key(),
        amount: fee_amount,
        value_usd_micros: usdc_out,
        ts: now,
    });
    Ok((usdc_out, false))
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::Discriminator;

    #[cfg(feature = "migration")]
    const PADDED_LEGACY_ACCOUNT_LEN: usize = 8 + 256;

    #[cfg(feature = "migration")]
    fn serialize_legacy_account<T: AnchorSerialize>(value: &T) -> Vec<u8> {
        let mut data = vec![0u8; PADDED_LEGACY_ACCOUNT_LEN];
        let mut body: &mut [u8] = &mut data[8..];
        value.serialize(&mut body).unwrap();
        data
    }

    #[cfg(feature = "migration")]
    fn sample_legacy_config() -> ConfigLegacyV1 {
        ConfigLegacyV1 {
            admin: Pubkey::new_unique(),
            true_cash: true,
            cash_backed: false,
            real_peg: true,
            sovereign: false,
            cashback_bps: 9_123,
            reward_interval: 777,
            allow_fallback_usdt: false,
            token_value_usd_micros_default: 1_234_567,
            treasury_value_usd_micros: 9_876_543,
            enable_treasury: true,
            enable_sjbc: false,
            enable_sjbc2: true,
            enable_sjbc3: false,
            enable_usdc: true,
            enable_usdt_main: false,
            enable_usdt_old: true,
            enable_btc_portal: false,
        }
    }

    #[cfg(feature = "migration")]
    fn sample_legacy_pool() -> PoolLegacyV1 {
        PoolLegacyV1 {
            owner: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            base_vault: Pubkey::new_unique(),
            quote_vault: Pubkey::new_unique(),
            lp_mint: Pubkey::new_unique(),
            base_value_usd_micros: 1_100_000,
            quote_value_usd_micros: 2_200_000,
            true_cash: true,
            cash_backed: true,
            real_peg: false,
            sovereign: true,
            fee_bps: 500,
            active: true,
        }
    }

    #[cfg(feature = "migration")]
    #[test]
    fn migrate_config_v1_to_v2_preserves_legacy_values_and_sets_defaults() {
        let legacy = sample_legacy_config();
        let parsed = deserialize_legacy_config_v1(&serialize_legacy_account(&legacy)).unwrap();
        assert_eq!(parsed, legacy);

        let migrated = build_config_from_legacy_v1(&parsed, 254);

        assert_eq!(migrated.admin, legacy.admin);
        assert_eq!(migrated.true_cash, legacy.true_cash);
        assert_eq!(migrated.cash_backed, legacy.cash_backed);
        assert_eq!(migrated.real_peg, legacy.real_peg);
        assert_eq!(migrated.sovereign, legacy.sovereign);
        assert_eq!(migrated.cashback_bps, legacy.cashback_bps);
        assert_eq!(migrated.reward_interval, legacy.reward_interval);
        assert_eq!(migrated.allow_fallback_usdt, legacy.allow_fallback_usdt);
        assert_eq!(
            migrated.token_value_usd_micros_default,
            legacy.token_value_usd_micros_default
        );
        assert_eq!(
            migrated.treasury_value_usd_micros,
            legacy.treasury_value_usd_micros
        );
        assert_eq!(migrated.enable_treasury, legacy.enable_treasury);
        assert_eq!(migrated.enable_sjbc, legacy.enable_sjbc);
        assert_eq!(migrated.enable_sjbc2, legacy.enable_sjbc2);
        assert_eq!(migrated.enable_sjbc3, legacy.enable_sjbc3);
        assert_eq!(migrated.enable_usdc, legacy.enable_usdc);
        assert_eq!(migrated.enable_usdt_main, legacy.enable_usdt_main);
        assert_eq!(migrated.enable_usdt_old, legacy.enable_usdt_old);
        assert_eq!(migrated.enable_btc_portal, legacy.enable_btc_portal);
        assert_eq!(migrated.main_wallet, pubkey_from_str(MAIN_WALLET));
        assert_eq!(migrated.okx_wallet, pubkey_from_str(OKX_WALLET));
        assert_eq!(migrated.usdc_coffre, pubkey_from_str(USDC_COFFRE));
        assert_eq!(migrated.usdt_coffre, pubkey_from_str(USDT_COFFRE));
        assert_eq!(
            migrated.treasury_usdc_ata,
            pubkey_from_str(TREASURY_USDC_ATA)
        );
        assert_eq!(
            migrated.treasury_usdt_ata,
            pubkey_from_str(TREASURY_USDT_ATA)
        );
        assert_eq!(migrated.auto_collect_every_swaps, 10);
        assert_eq!(
            migrated.fee_threshold_usd_micros,
            DEFAULT_FEE_THRESHOLD_USD_MICROS
        );
        assert_eq!(migrated.keeper_authority, pubkey_from_str(MAIN_WALLET));
        assert_eq!(migrated.usdc_mint, pubkey_from_str(USDC_MINT));
        assert_eq!(migrated.usdt_mint, pubkey_from_str(USDT_MAIN_MINT));
        assert_eq!(migrated.extra_payout_mint_0, Pubkey::default());
        assert_eq!(migrated.extra_payout_vault_ata_0, Pubkey::default());
        assert_eq!(migrated.payout_threshold_usd_micros, 0);
        assert_eq!(migrated.max_payout_usd_micros, 0);
        assert_eq!(migrated.max_payout_per_window_usd_micros, 0);
        assert_eq!(migrated.payout_window_secs, 0);
        assert_eq!(migrated.payout_window_start, 0);
        assert_eq!(migrated.payout_window_used_usd_micros, 0);
        assert_eq!(migrated.lp_cashback_bps, 0);
        assert_eq!(migrated.claim_cashback_bps, 0);
        assert_eq!(migrated.bump, 254);
    }

    #[cfg(feature = "migration")]
    #[test]
    fn migrate_pool_v1_to_v2_preserves_legacy_values_and_sets_defaults() {
        let legacy = sample_legacy_pool();
        let parsed = deserialize_legacy_pool_v1(&serialize_legacy_account(&legacy)).unwrap();
        assert_eq!(parsed, legacy);

        let migrated = build_pool_from_legacy_v1(&parsed, 253);

        assert_eq!(migrated.owner, legacy.owner);
        assert_eq!(migrated.base_mint, legacy.base_mint);
        assert_eq!(migrated.quote_mint, legacy.quote_mint);
        assert_eq!(migrated.base_vault, legacy.base_vault);
        assert_eq!(migrated.quote_vault, legacy.quote_vault);
        assert_eq!(migrated.lp_mint, legacy.lp_mint);
        assert_eq!(migrated.base_value_usd_micros, legacy.base_value_usd_micros);
        assert_eq!(
            migrated.quote_value_usd_micros,
            legacy.quote_value_usd_micros
        );
        assert_eq!(migrated.true_cash, legacy.true_cash);
        assert_eq!(migrated.cash_backed, legacy.cash_backed);
        assert_eq!(migrated.real_peg, legacy.real_peg);
        assert_eq!(migrated.sovereign, legacy.sovereign);
        assert_eq!(migrated.fee_bps, legacy.fee_bps);
        assert_eq!(migrated.active, legacy.active);
        assert_eq!(migrated.fee_vault_base, Pubkey::default());
        assert_eq!(migrated.fee_vault_quote, Pubkey::default());
        assert_eq!(migrated.swap_cashback_bps, 0);
        assert_eq!(migrated.protocol_fee_debt_usd_micros, 0);
        assert_eq!(migrated.protocol_fee_debt_count, 0);
        assert_eq!(migrated.protocol_fee_debt_last_ts, 0);
        assert_eq!(migrated.swap_count, 0);
        assert_eq!(migrated.bump, 253);
    }

    #[cfg(feature = "migration")]
    #[test]
    fn config_migration_is_idempotent_once_full_length_is_reached() {
        assert!(config_needs_migration(PADDED_LEGACY_ACCOUNT_LEN));
        assert!(!config_needs_migration(CONFIG_ACCOUNT_LEN));
    }

    #[cfg(feature = "migration")]
    #[test]
    fn pool_migration_is_idempotent_once_full_length_is_reached() {
        assert!(pool_needs_migration(PADDED_LEGACY_ACCOUNT_LEN));
        assert!(!pool_needs_migration(Pool::LEN));
    }

    #[test]
    fn compute_usdc_settlement_amount_is_identity_for_usdc_fee() {
        let usdc_out = compute_usdc_settlement_amount(250_000, 1_000_000, 6).unwrap();
        assert_eq!(usdc_out, 250_000);
    }

    #[test]
    fn compute_usdc_settlement_amount_converts_non_usdc_fee() {
        let usdc_out = compute_usdc_settlement_amount(500_000_000, 2_000_000, 9).unwrap();
        assert_eq!(usdc_out, 1_000_000);
    }

    #[test]
    fn compute_usdc_settlement_amount_can_round_down_to_zero() {
        let usdc_out = compute_usdc_settlement_amount(1, 1, 9).unwrap();
        assert_eq!(usdc_out, 0);
    }

    #[test]
    fn choose_swap_fee_settlement_action_prefers_direct_usdc() {
        let usdc = Pubkey::new_unique();
        let usdt = Pubkey::new_unique();
        let action = choose_swap_fee_settlement_action(usdc, usdc, usdt, true, true, 0, 0, 123);
        assert_eq!(action, SwapFeeSettlementAction::DirectUsdc);
    }

    #[test]
    fn choose_swap_fee_settlement_action_uses_burn_to_usdc_when_available() {
        let fee_mint = Pubkey::new_unique();
        let usdc = Pubkey::new_unique();
        let usdt = Pubkey::new_unique();
        let action =
            choose_swap_fee_settlement_action(fee_mint, usdc, usdt, true, false, 500, 0, 500);
        assert_eq!(action, SwapFeeSettlementAction::BurnToUsdc);
    }

    #[test]
    fn choose_swap_fee_settlement_action_uses_burn_to_usdt_when_usdc_unavailable() {
        let fee_mint = Pubkey::new_unique();
        let usdc = Pubkey::new_unique();
        let usdt = Pubkey::new_unique();
        let action =
            choose_swap_fee_settlement_action(fee_mint, usdc, usdt, false, true, 0, 700, 700);
        assert_eq!(action, SwapFeeSettlementAction::BurnToUsdt);
    }

    #[test]
    fn choose_swap_fee_settlement_action_falls_back_to_protocol_debt() {
        let fee_mint = Pubkey::new_unique();
        let usdc = Pubkey::new_unique();
        let usdt = Pubkey::new_unique();
        let action =
            choose_swap_fee_settlement_action(fee_mint, usdc, usdt, false, false, 0, 0, 900);
        assert_eq!(action, SwapFeeSettlementAction::AccrueProtocolDebt);
    }

    fn sample_pool(base_mint: Pubkey, quote_mint: Pubkey) -> Pool {
        Pool {
            owner: Pubkey::default(),
            base_mint,
            quote_mint,
            base_vault: Pubkey::default(),
            quote_vault: Pubkey::default(),
            lp_mint: Pubkey::default(),
            fee_vault_base: Pubkey::default(),
            fee_vault_quote: Pubkey::default(),
            base_value_usd_micros: 0,
            quote_value_usd_micros: 0,
            true_cash: false,
            cash_backed: false,
            real_peg: false,
            sovereign: true,
            fee_bps: 0,
            active: true,
            swap_cashback_bps: 0,
            protocol_fee_debt_usd_micros: 0,
            protocol_fee_debt_count: 0,
            protocol_fee_debt_last_ts: 0,
            created_at: 0,
            last_swap_ts: 0,
            total_base_volume: 0,
            total_quote_volume: 0,
            swap_count: 0,
            bump: 0,
        }
    }

    #[test]
    fn defer_swap_fee_settlement_for_fully_sovereign_pool() {
        let usdc = Pubkey::new_unique();
        let usdt = Pubkey::new_unique();
        let pool = sample_pool(Pubkey::new_unique(), Pubkey::new_unique());
        assert!(should_defer_swap_fee_settlement_to_fee_vault(
            &pool, usdc, usdt
        ));
    }

    #[test]
    fn do_not_defer_swap_fee_settlement_when_pool_has_stable_side() {
        let usdc = Pubkey::new_unique();
        let usdt = Pubkey::new_unique();
        let pool = sample_pool(usdc, Pubkey::new_unique());
        assert!(!should_defer_swap_fee_settlement_to_fee_vault(
            &pool, usdc, usdt
        ));
    }

    #[test]
    fn choose_claim_settlement_action_prefers_reward_vault() {
        let action = choose_claim_settlement_action(
            10,
            5,
            ClaimDirectCashRail::Usdc,
            10,
            5,
            10,
            5,
            0,
            5,
            true,
            10,
            5,
        );
        assert_eq!(action, ClaimSettlementAction::DirectRewardVault);
    }

    #[test]
    fn choose_claim_settlement_action_uses_usdt_vault_fallback() {
        let action = choose_claim_settlement_action(
            0,
            5,
            ClaimDirectCashRail::Usdc,
            0,
            5,
            0,
            5,
            0,
            5,
            true,
            8,
            5,
        );
        assert_eq!(action, ClaimSettlementAction::FallbackUsdtVault);
    }

    #[test]
    fn choose_claim_settlement_action_prefers_direct_cash_rail_before_usdt_fallback() {
        let action = choose_claim_settlement_action(
            0,
            5,
            ClaimDirectCashRail::Usdc,
            9,
            5,
            0,
            5,
            0,
            5,
            true,
            7,
            5,
        );
        assert_eq!(action, ClaimSettlementAction::DirectRequestedCashRail);
    }

    #[test]
    fn choose_claim_settlement_action_uses_direct_usdc_cash_rail_without_reward_vault() {
        let action = choose_claim_settlement_action(
            0,
            5,
            ClaimDirectCashRail::Usdc,
            7,
            5,
            0,
            5,
            0,
            5,
            false,
            0,
            5,
        );
        assert_eq!(action, ClaimSettlementAction::DirectRequestedCashRail);
    }

    #[test]
    fn choose_claim_settlement_action_uses_direct_usdt_cash_rail_without_reward_vault() {
        let action = choose_claim_settlement_action(
            0,
            5,
            ClaimDirectCashRail::Usdt,
            0,
            5,
            7,
            5,
            0,
            5,
            false,
            0,
            5,
        );
        assert_eq!(action, ClaimSettlementAction::DirectRequestedCashRail);
    }

    #[test]
    fn choose_claim_settlement_action_uses_configured_extra_cash_rail() {
        let action = choose_claim_settlement_action(
            0,
            7,
            ClaimDirectCashRail::ExtraConfigured,
            0,
            7,
            0,
            7,
            9,
            7,
            false,
            0,
            7,
        );
        assert_eq!(action, ClaimSettlementAction::DirectRequestedCashRail);
    }

    #[test]
    fn choose_claim_settlement_action_falls_back_to_need_payout_when_no_stable_liquidity() {
        let action = choose_claim_settlement_action(
            0,
            5,
            ClaimDirectCashRail::Usdc,
            0,
            5,
            0,
            5,
            0,
            5,
            false,
            0,
            5,
        );
        assert_eq!(action, ClaimSettlementAction::EmitNeedPayoutEvent);
    }

    #[test]
    fn choose_sovereign_route_execution_kind_prefers_live_when_route_is_live() {
        assert_eq!(
            choose_sovereign_route_execution_kind(1, true).unwrap(),
            SovereignRouteExecutionKind::LiveSwap
        );
        assert_eq!(
            choose_sovereign_route_execution_kind(2, true).unwrap(),
            SovereignRouteExecutionKind::LiveSwap
        );
        assert_eq!(
            choose_sovereign_route_execution_kind(4, true).unwrap(),
            SovereignRouteExecutionKind::LiveSwap
        );
        assert_eq!(
            choose_sovereign_route_execution_kind(6, true).unwrap(),
            SovereignRouteExecutionKind::LiveSwap
        );
    }

    #[test]
    fn choose_sovereign_route_execution_kind_uses_buffered_and_burn_release_symmetrically() {
        assert_eq!(
            choose_sovereign_route_execution_kind(1, false).unwrap(),
            SovereignRouteExecutionKind::BurnRelease
        );
        assert_eq!(
            choose_sovereign_route_execution_kind(2, false).unwrap(),
            SovereignRouteExecutionKind::DirectTransfer
        );
        assert_eq!(
            choose_sovereign_route_execution_kind(4, false).unwrap(),
            SovereignRouteExecutionKind::BufferedReceipt
        );
        assert_eq!(
            choose_sovereign_route_execution_kind(5, false).unwrap(),
            SovereignRouteExecutionKind::BufferedReceipt
        );
        assert_eq!(
            choose_sovereign_route_execution_kind(6, false).unwrap(),
            SovereignRouteExecutionKind::BufferedDebt
        );
    }

    #[test]
    fn is_configured_extra_payout_mint_recognizes_only_configured_extras() {
        let extra0 = Pubkey::new_unique();
        let extra1 = Pubkey::new_unique();
        assert!(is_configured_extra_payout_mint(
            extra0,
            extra0,
            extra1,
            Pubkey::default(),
            Pubkey::default(),
        ));
        assert!(is_configured_extra_payout_mint(
            extra1,
            extra0,
            extra1,
            Pubkey::default(),
            Pubkey::default(),
        ));
        assert!(!is_configured_extra_payout_mint(
            Pubkey::new_unique(),
            extra0,
            extra1,
            Pubkey::default(),
            Pubkey::default(),
        ));
    }

    #[test]
    fn requested_claim_direct_cash_rail_is_symmetric_for_usdc_and_usdt() {
        let usdc = Pubkey::new_unique();
        let usdt = Pubkey::new_unique();
        let extra = Pubkey::new_unique();
        assert_eq!(
            requested_claim_direct_cash_rail(
                usdc,
                usdc,
                usdt,
                extra,
                Pubkey::default(),
                Pubkey::default(),
                Pubkey::default(),
            ),
            ClaimDirectCashRail::Usdc
        );
        assert_eq!(
            requested_claim_direct_cash_rail(
                usdt,
                usdc,
                usdt,
                extra,
                Pubkey::default(),
                Pubkey::default(),
                Pubkey::default(),
            ),
            ClaimDirectCashRail::Usdt
        );
        assert_eq!(
            requested_claim_direct_cash_rail(
                extra,
                usdc,
                usdt,
                extra,
                Pubkey::default(),
                Pubkey::default(),
                Pubkey::default(),
            ),
            ClaimDirectCashRail::ExtraConfigured
        );
    }

    #[test]
    fn canonical_claim_usdt_fallback_vault_address_is_stable_for_same_mint() {
        let usdt = Pubkey::new_unique();
        assert_eq!(
            canonical_claim_usdt_fallback_vault_address(usdt),
            canonical_claim_usdt_fallback_vault_address(usdt)
        );
    }

    #[test]
    fn staking_claim_reward_is_value_based_and_independent_from_reward_vault_funding() {
        let reward_usd_micros =
            compute_claim_reward_usd_micros(2_000_000, 6, 2_500_000, 1_000).unwrap();
        let stable_atoms = usd_micros_to_atoms(reward_usd_micros, STABLE_CASH_DECIMALS).unwrap();
        assert_eq!(reward_usd_micros, 500_000);
        assert_eq!(stable_atoms, 500_000);
    }

    #[test]
    fn protocol_fee_debt_shard_count_keeps_single_lot_under_cap() {
        assert_eq!(
            protocol_fee_debt_shard_count(DEFAULT_TICKET_CAP_USD_MICROS),
            1
        );
    }

    #[test]
    fn protocol_fee_debt_shard_count_splits_over_cap() {
        assert_eq!(
            protocol_fee_debt_shard_count(DEFAULT_TICKET_CAP_USD_MICROS + 1),
            2
        );
        assert_eq!(
            protocol_fee_debt_shard_count(DEFAULT_TICKET_CAP_USD_MICROS * 3),
            3
        );
    }

    #[test]
    fn accrue_protocol_fee_debt_state_is_reusable_without_corruption() {
        let mut pool = Pool {
            owner: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            base_vault: Pubkey::new_unique(),
            quote_vault: Pubkey::new_unique(),
            lp_mint: Pubkey::new_unique(),
            fee_vault_base: Pubkey::new_unique(),
            fee_vault_quote: Pubkey::new_unique(),
            base_value_usd_micros: 0,
            quote_value_usd_micros: 0,
            true_cash: true,
            cash_backed: true,
            real_peg: true,
            sovereign: true,
            fee_bps: 30,
            active: true,
            swap_cashback_bps: 0,
            protocol_fee_debt_usd_micros: 0,
            protocol_fee_debt_count: 0,
            protocol_fee_debt_last_ts: 0,
            created_at: 0,
            last_swap_ts: 0,
            total_base_volume: 0,
            total_quote_volume: 0,
            swap_count: 0,
            bump: 1,
        };

        let total_1 = accrue_protocol_fee_debt(&mut pool, 500, 10);
        let total_2 = accrue_protocol_fee_debt(&mut pool, 700, 11);

        assert_eq!(total_1, 500);
        assert_eq!(total_2, 1_200);
        assert_eq!(pool.protocol_fee_debt_usd_micros, 1_200);
        assert_eq!(pool.protocol_fee_debt_count, 0);
        assert_eq!(pool.protocol_fee_debt_last_ts, 11);
    }

    #[test]
    fn pool_len_matches_serialized_layout() {
        let pool = Pool {
            owner: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            base_vault: Pubkey::new_unique(),
            quote_vault: Pubkey::new_unique(),
            lp_mint: Pubkey::new_unique(),
            fee_vault_base: Pubkey::new_unique(),
            fee_vault_quote: Pubkey::new_unique(),
            base_value_usd_micros: 1,
            quote_value_usd_micros: 2,
            true_cash: true,
            cash_backed: true,
            real_peg: true,
            sovereign: true,
            fee_bps: 30,
            active: true,
            swap_cashback_bps: 0,
            protocol_fee_debt_usd_micros: 3,
            protocol_fee_debt_count: 4,
            protocol_fee_debt_last_ts: 5,
            created_at: 0,
            last_swap_ts: 0,
            total_base_volume: 0,
            total_quote_volume: 0,
            swap_count: 6,
            bump: 1,
        };

        let mut bytes = Vec::new();
        pool.try_serialize(&mut bytes).unwrap();
        assert_eq!(bytes.len(), Pool::LEN);
    }

    #[test]
    fn pool_init_helpers_set_all_runtime_addresses() {
        let base_mint = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();
        let mut pool = Pool {
            owner: Pubkey::new_unique(),
            base_mint,
            quote_mint,
            base_vault: Pubkey::default(),
            quote_vault: Pubkey::default(),
            lp_mint: Pubkey::default(),
            fee_vault_base: Pubkey::default(),
            fee_vault_quote: Pubkey::default(),
            base_value_usd_micros: 1,
            quote_value_usd_micros: 2,
            true_cash: true,
            cash_backed: true,
            real_peg: true,
            sovereign: true,
            fee_bps: 30,
            active: true,
            swap_cashback_bps: 0,
            protocol_fee_debt_usd_micros: 0,
            protocol_fee_debt_count: 0,
            protocol_fee_debt_last_ts: 0,
            created_at: 0,
            last_swap_ts: 0,
            total_base_volume: 0,
            total_quote_volume: 0,
            swap_count: 0,
            bump: 1,
        };

        let base_vault = Pubkey::new_unique();
        let quote_vault = Pubkey::new_unique();
        let lp_mint = Pubkey::new_unique();
        let fee_vault_base = Pubkey::new_unique();
        let fee_vault_quote = Pubkey::new_unique();

        set_pool_base_vault(&mut pool, base_mint, base_vault).unwrap();
        set_pool_quote_vault(&mut pool, quote_mint, quote_vault).unwrap();
        set_pool_lp_mint(&mut pool, lp_mint).unwrap();
        set_pool_fee_vault_base(&mut pool, base_mint, fee_vault_base).unwrap();
        set_pool_fee_vault_quote(&mut pool, quote_mint, fee_vault_quote).unwrap();

        assert_eq!(pool.base_vault, base_vault);
        assert_eq!(pool.quote_vault, quote_vault);
        assert_eq!(pool.lp_mint, lp_mint);
        assert_eq!(pool.fee_vault_base, fee_vault_base);
        assert_eq!(pool.fee_vault_quote, fee_vault_quote);
        assert_ne!(pool.base_vault, Pubkey::default());
        assert_ne!(pool.quote_vault, Pubkey::default());
        assert_ne!(pool.lp_mint, Pubkey::default());
        assert_ne!(pool.fee_vault_base, Pubkey::default());
        assert_ne!(pool.fee_vault_quote, Pubkey::default());
    }

    #[test]
    fn initialize_asset_registry_sets_runtime_fields() {
        let mint = Pubkey::new_unique();
        let mut asset_registry = AssetRegistry {
            mint: Pubkey::default(),
            valuation_usd_micros: 0,
            decimals: 0,
            is_lp: false,
            active: false,
            bump: 0,
        };

        initialize_asset_registry(&mut asset_registry, mint, 1_500_000, 6, true, 7).unwrap();

        assert_eq!(asset_registry.mint, mint);
        assert_eq!(asset_registry.valuation_usd_micros, 1_500_000);
        assert_eq!(asset_registry.decimals, 6);
        assert!(asset_registry.is_lp);
        assert!(asset_registry.active);
        assert_eq!(asset_registry.bump, 7);
    }

    #[test]
    fn protocol_debt_ledger_len_matches_serialized_layout() {
        let ledger = ProtocolDebtLedger {
            pool: Pubkey::new_unique(),
            next_nonce: 9,
            overflow_usd_micros: 10,
            last_ts: 11,
            bump: 1,
            lots: [ProtocolDebtLot::default(); PROTOCOL_DEBT_LEDGER_SLOTS],
        };

        let mut bytes = Vec::new();
        ledger.try_serialize(&mut bytes).unwrap();
        assert_eq!(ProtocolDebtLot::LEN, 140);
        assert_eq!(bytes.len(), ProtocolDebtLedger::LEN);
    }

    #[cfg(feature = "migration")]
    #[test]
    fn protocol_debt_ledger_legacy_migration_preserves_header_and_binds_escrow() {
        let pool = Pubkey::new_unique();
        let mint_in = Pubkey::new_unique();
        let nonce = 77u64;
        let amount_in = 1234u64;
        let usd_micros = 9_876_543u64;
        let created_ts = 456i64;
        let route_hint = 6u8;
        let status = PROTOCOL_DEBT_LOT_OPEN;
        let mut legacy_data = vec![0u8; LEGACY_PROTOCOL_DEBT_LEDGER_V1_MIN_LEN];

        legacy_data[8..40].copy_from_slice(pool.as_ref());
        legacy_data[40..48].copy_from_slice(&9u64.to_le_bytes());
        legacy_data[48..56].copy_from_slice(&11u64.to_le_bytes());
        legacy_data[56..64].copy_from_slice(&22i64.to_le_bytes());
        legacy_data[64] = 3;

        let lot_start = 65;
        legacy_data[lot_start..lot_start + 8].copy_from_slice(&nonce.to_le_bytes());
        legacy_data[lot_start + 8..lot_start + 40].copy_from_slice(mint_in.as_ref());
        legacy_data[lot_start + 40..lot_start + 48].copy_from_slice(&amount_in.to_le_bytes());
        legacy_data[lot_start + 48..lot_start + 56].copy_from_slice(&usd_micros.to_le_bytes());
        legacy_data[lot_start + 56..lot_start + 64].copy_from_slice(&created_ts.to_le_bytes());
        legacy_data[lot_start + 64] = route_hint;
        legacy_data[lot_start + 65] = status;

        let mut migrated_data = vec![0u8; ProtocolDebtLedger::LEN];
        migrated_data[..8].copy_from_slice(&ProtocolDebtLedger::discriminator());
        migrate_protocol_debt_ledger_data_from_legacy_v1(&mut migrated_data, &legacy_data, pool)
            .unwrap();

        let mut bytes: &[u8] = &migrated_data;
        let migrated = ProtocolDebtLedger::try_deserialize(&mut bytes).unwrap();
        let (escrow_authority, escrow_bump) =
            expected_protocol_debt_lot_escrow_authority(pool, nonce);

        assert_eq!(migrated.pool, pool);
        assert_eq!(migrated.next_nonce, 9);
        assert_eq!(migrated.overflow_usd_micros, 11);
        assert_eq!(migrated.last_ts, 22);
        assert_eq!(migrated.bump, 3);
        assert_eq!(migrated.lots[0].nonce, nonce);
        assert_eq!(migrated.lots[0].mint_in, mint_in);
        assert_eq!(migrated.lots[0].amount_in, amount_in);
        assert_eq!(migrated.lots[0].usd_micros, usd_micros);
        assert_eq!(migrated.lots[0].created_ts, created_ts);
        assert_eq!(migrated.lots[0].escrow_mint, mint_in);
        assert_eq!(
            migrated.lots[0].escrow_ata,
            expected_live_escrow_ata(escrow_authority, mint_in)
        );
        assert_eq!(migrated.lots[0].escrow_amount_locked, amount_in);
        assert_eq!(migrated.lots[0].escrow_bump, escrow_bump);
        assert_eq!(
            migrated.lots[0].funding_state,
            SOVEREIGN_ESCROW_STATE_REQUESTED
        );
        assert_eq!(migrated.lots[0].route_hint, route_hint);
        assert_eq!(migrated.lots[0].status, status);
    }

    #[cfg(feature = "migration")]
    #[test]
    fn load_protocol_debt_lot_from_legacy_ledger_data_maps_v1_lot() {
        let pool = Pubkey::new_unique();
        let mint_in = Pubkey::new_unique();
        let nonce = 91u64;
        let amount_in = 2222u64;
        let usd_micros = 777_000_111u64;
        let created_ts = 999i64;
        let route_hint = 2u8;
        let mut legacy_data = vec![0u8; LEGACY_PROTOCOL_DEBT_LEDGER_V1_MIN_LEN];

        legacy_data[8..40].copy_from_slice(pool.as_ref());
        let lot_start = 65;
        legacy_data[lot_start..lot_start + 8].copy_from_slice(&nonce.to_le_bytes());
        legacy_data[lot_start + 8..lot_start + 40].copy_from_slice(mint_in.as_ref());
        legacy_data[lot_start + 40..lot_start + 48].copy_from_slice(&amount_in.to_le_bytes());
        legacy_data[lot_start + 48..lot_start + 56].copy_from_slice(&usd_micros.to_le_bytes());
        legacy_data[lot_start + 56..lot_start + 64].copy_from_slice(&created_ts.to_le_bytes());
        legacy_data[lot_start + 64] = route_hint;
        legacy_data[lot_start + 65] = PROTOCOL_DEBT_LOT_OPEN;

        let (index, lot, ledger_pool) =
            load_protocol_debt_lot_from_ledger_data(&legacy_data, nonce).unwrap();
        let (escrow_authority, escrow_bump) =
            expected_protocol_debt_lot_escrow_authority(pool, nonce);

        assert_eq!(index, 0);
        assert_eq!(ledger_pool, pool);
        assert_eq!(lot.nonce, nonce);
        assert_eq!(lot.mint_in, mint_in);
        assert_eq!(lot.amount_in, amount_in);
        assert_eq!(lot.usd_micros, usd_micros);
        assert_eq!(lot.created_ts, created_ts);
        assert_eq!(lot.escrow_mint, mint_in);
        assert_eq!(lot.escrow_ata, expected_live_escrow_ata(escrow_authority, mint_in));
        assert_eq!(lot.escrow_amount_locked, amount_in);
        assert_eq!(lot.escrow_bump, escrow_bump);
        assert_eq!(lot.funding_state, SOVEREIGN_ESCROW_STATE_REQUESTED);
        assert_eq!(lot.route_hint, route_hint);
        assert_eq!(lot.status, PROTOCOL_DEBT_LOT_OPEN);
    }

    #[cfg(feature = "migration")]
    #[test]
    fn store_protocol_debt_lot_into_legacy_ledger_data_preserves_v1_layout() {
        let pool = Pubkey::new_unique();
        let mint_in = Pubkey::new_unique();
        let nonce = 123u64;
        let mut legacy_data = vec![0u8; LEGACY_PROTOCOL_DEBT_LEDGER_V1_MIN_LEN];
        legacy_data[8..40].copy_from_slice(pool.as_ref());

        let lot = ProtocolDebtLot {
            nonce,
            mint_in,
            amount_in: 3333,
            usd_micros: 4444,
            created_ts: 555,
            escrow_mint: mint_in,
            escrow_ata: Pubkey::new_unique(),
            escrow_amount_locked: 3333,
            escrow_bump: 7,
            funding_state: SOVEREIGN_ESCROW_STATE_REQUESTED,
            route_hint: 1,
            status: PROTOCOL_DEBT_LOT_TICKETED,
        };

        store_protocol_debt_lot_into_ledger_data(&mut legacy_data, 0, &lot).unwrap();
        let (index, reloaded, ledger_pool) =
            load_protocol_debt_lot_from_ledger_data(&legacy_data, nonce).unwrap();

        assert_eq!(index, 0);
        assert_eq!(ledger_pool, pool);
        assert_eq!(reloaded.nonce, lot.nonce);
        assert_eq!(reloaded.mint_in, lot.mint_in);
        assert_eq!(reloaded.amount_in, lot.amount_in);
        assert_eq!(reloaded.usd_micros, lot.usd_micros);
        assert_eq!(reloaded.created_ts, lot.created_ts);
        assert_eq!(reloaded.route_hint, lot.route_hint);
        assert_eq!(reloaded.status, PROTOCOL_DEBT_LOT_TICKETED);
    }

    #[test]
    fn protocol_debt_lot_settlement_reduces_aggregate() {
        let mut pool = Pool {
            owner: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            base_vault: Pubkey::new_unique(),
            quote_vault: Pubkey::new_unique(),
            lp_mint: Pubkey::new_unique(),
            fee_vault_base: Pubkey::new_unique(),
            fee_vault_quote: Pubkey::new_unique(),
            base_value_usd_micros: 0,
            quote_value_usd_micros: 0,
            true_cash: true,
            cash_backed: true,
            real_peg: true,
            sovereign: true,
            fee_bps: 30,
            active: true,
            swap_cashback_bps: 0,
            protocol_fee_debt_usd_micros: 700,
            protocol_fee_debt_count: 1,
            protocol_fee_debt_last_ts: 0,
            created_at: 0,
            last_swap_ts: 0,
            total_base_volume: 0,
            total_quote_volume: 0,
            swap_count: 0,
            bump: 1,
        };

        let mut lot = ProtocolDebtLot {
            nonce: 1,
            mint_in: Pubkey::new_unique(),
            amount_in: 42,
            usd_micros: 700,
            created_ts: 1,
            escrow_mint: Pubkey::new_unique(),
            escrow_ata: Pubkey::new_unique(),
            escrow_amount_locked: 42,
            escrow_bump: 1,
            funding_state: SOVEREIGN_ESCROW_STATE_REQUESTED,
            route_hint: 1,
            status: PROTOCOL_DEBT_LOT_OPEN,
        };

        mark_protocol_debt_lot_settled(&mut pool, &mut lot, 12).unwrap();

        assert_eq!(pool.protocol_fee_debt_usd_micros, 0);
        assert_eq!(pool.protocol_fee_debt_count, 0);
        assert_eq!(pool.protocol_fee_debt_last_ts, 12);
        assert_eq!(lot.status, PROTOCOL_DEBT_LOT_SETTLED);
        assert_eq!(lot.usd_micros, 0);
    }

    #[test]
    fn protocol_debt_lot_settlement_accepts_ticketed_lot() {
        let mut pool = Pool {
            owner: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            base_vault: Pubkey::new_unique(),
            quote_vault: Pubkey::new_unique(),
            lp_mint: Pubkey::new_unique(),
            fee_vault_base: Pubkey::new_unique(),
            fee_vault_quote: Pubkey::new_unique(),
            base_value_usd_micros: 0,
            quote_value_usd_micros: 0,
            true_cash: true,
            cash_backed: true,
            real_peg: true,
            sovereign: true,
            fee_bps: 30,
            active: true,
            swap_cashback_bps: 0,
            protocol_fee_debt_usd_micros: 700,
            protocol_fee_debt_count: 1,
            protocol_fee_debt_last_ts: 0,
            created_at: 0,
            last_swap_ts: 0,
            total_base_volume: 0,
            total_quote_volume: 0,
            swap_count: 0,
            bump: 1,
        };

        let mut lot = ProtocolDebtLot {
            nonce: 2,
            mint_in: Pubkey::new_unique(),
            amount_in: 42,
            usd_micros: 700,
            created_ts: 1,
            escrow_mint: Pubkey::new_unique(),
            escrow_ata: Pubkey::new_unique(),
            escrow_amount_locked: 42,
            escrow_bump: 1,
            funding_state: SOVEREIGN_ESCROW_STATE_REQUESTED,
            route_hint: 1,
            status: PROTOCOL_DEBT_LOT_TICKETED,
        };

        mark_protocol_debt_lot_settled(&mut pool, &mut lot, 13).unwrap();

        assert_eq!(pool.protocol_fee_debt_usd_micros, 0);
        assert_eq!(pool.protocol_fee_debt_count, 0);
        assert_eq!(lot.status, PROTOCOL_DEBT_LOT_SETTLED);
    }

    #[test]
    fn settle_protocol_debt_lots_updates_overflow_logical_count() {
        let mut pool = Pool {
            owner: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            base_vault: Pubkey::new_unique(),
            quote_vault: Pubkey::new_unique(),
            lp_mint: Pubkey::new_unique(),
            fee_vault_base: Pubkey::new_unique(),
            fee_vault_quote: Pubkey::new_unique(),
            base_value_usd_micros: 0,
            quote_value_usd_micros: 0,
            true_cash: true,
            cash_backed: true,
            real_peg: true,
            sovereign: true,
            fee_bps: 30,
            active: true,
            swap_cashback_bps: 0,
            protocol_fee_debt_usd_micros: DEFAULT_TICKET_CAP_USD_MICROS * 2 + 5,
            protocol_fee_debt_count: 3,
            protocol_fee_debt_last_ts: 0,
            created_at: 0,
            last_swap_ts: 0,
            total_base_volume: 0,
            total_quote_volume: 0,
            swap_count: 0,
            bump: 1,
        };
        let mut ledger = ProtocolDebtLedger {
            pool: Pubkey::new_unique(),
            next_nonce: 1,
            overflow_usd_micros: DEFAULT_TICKET_CAP_USD_MICROS * 2 + 5,
            last_ts: 0,
            bump: 1,
            lots: [ProtocolDebtLot::default(); PROTOCOL_DEBT_LEDGER_SLOTS],
        };

        let settled = settle_protocol_debt_lots_in_ledger(
            &mut pool,
            &mut ledger,
            DEFAULT_TICKET_CAP_USD_MICROS,
            20,
        )
        .unwrap();

        assert_eq!(settled, DEFAULT_TICKET_CAP_USD_MICROS);
        assert_eq!(
            pool.protocol_fee_debt_usd_micros,
            DEFAULT_TICKET_CAP_USD_MICROS + 5
        );
        assert_eq!(pool.protocol_fee_debt_count, 2);
        assert_eq!(
            ledger.overflow_usd_micros,
            DEFAULT_TICKET_CAP_USD_MICROS + 5
        );
    }

    #[test]
    fn settle_protocol_debt_lots_handles_partial_open_lot_without_dropping_count() {
        let mut pool = Pool {
            owner: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            base_vault: Pubkey::new_unique(),
            quote_vault: Pubkey::new_unique(),
            lp_mint: Pubkey::new_unique(),
            fee_vault_base: Pubkey::new_unique(),
            fee_vault_quote: Pubkey::new_unique(),
            base_value_usd_micros: 0,
            quote_value_usd_micros: 0,
            true_cash: true,
            cash_backed: true,
            real_peg: true,
            sovereign: true,
            fee_bps: 30,
            active: true,
            swap_cashback_bps: 0,
            protocol_fee_debt_usd_micros: 900,
            protocol_fee_debt_count: 1,
            protocol_fee_debt_last_ts: 0,
            created_at: 0,
            last_swap_ts: 0,
            total_base_volume: 0,
            total_quote_volume: 0,
            swap_count: 0,
            bump: 1,
        };
        let mut ledger = ProtocolDebtLedger {
            pool: Pubkey::new_unique(),
            next_nonce: 2,
            overflow_usd_micros: 0,
            last_ts: 0,
            bump: 1,
            lots: [ProtocolDebtLot::default(); PROTOCOL_DEBT_LEDGER_SLOTS],
        };
        ledger.lots[0] = ProtocolDebtLot {
            nonce: 1,
            mint_in: Pubkey::new_unique(),
            amount_in: 50,
            usd_micros: 900,
            created_ts: 1,
            escrow_mint: Pubkey::new_unique(),
            escrow_ata: Pubkey::new_unique(),
            escrow_amount_locked: 50,
            escrow_bump: 1,
            funding_state: SOVEREIGN_ESCROW_STATE_REQUESTED,
            route_hint: 0,
            status: PROTOCOL_DEBT_LOT_OPEN,
        };

        settle_protocol_debt_lots_in_ledger(&mut pool, &mut ledger, 400, 21).unwrap();

        assert_eq!(pool.protocol_fee_debt_usd_micros, 500);
        assert_eq!(pool.protocol_fee_debt_count, 1);
        assert_eq!(ledger.lots[0].usd_micros, 500);
        assert_eq!(ledger.lots[0].status, PROTOCOL_DEBT_LOT_OPEN);
    }

    #[test]
    fn live_sovereign_source_kind_rejects_ticket_claim_and_debt() {
        assert!(require_live_sovereign_source_kind(1).is_ok());
        assert!(require_live_sovereign_source_kind(2).is_ok());
        assert!(require_live_sovereign_source_kind(3).is_ok());
        assert!(require_live_sovereign_source_kind(4).is_err());
        assert!(require_live_sovereign_source_kind(5).is_err());
        assert!(require_live_sovereign_source_kind(6).is_err());
    }

    #[test]
    fn parse_live_swap_accounts_rejects_missing_live_route() {
        let remaining: [AccountInfo; 0] = [];
        assert!(parse_sovereign_live_swap_accounts(2, &remaining).is_err());
        assert!(parse_sovereign_live_swap_accounts(1, &remaining).is_err());
    }

    #[test]
    fn parse_escrow_live_swap_accounts_rejects_missing_live_route() {
        let remaining: [AccountInfo; 0] = [];
        assert!(parse_sovereign_escrow_live_swap_accounts(4, &remaining).is_err());
        assert!(parse_sovereign_escrow_live_swap_accounts(5, &remaining).is_err());
        assert!(parse_sovereign_escrow_live_swap_accounts(6, &remaining).is_err());
    }

    #[test]
    fn classify_live_sovereign_settlement_accepts_configured_stable_outputs() {
        let usdc = Pubkey::new_unique();
        let usdt = Pubkey::new_unique();
        let extra = Pubkey::new_unique();

        assert_eq!(
            classify_live_sovereign_settlement_kind(
                SovereignFinalOutputKind::Usdc,
                usdc,
                usdc,
                usdt,
                extra,
                Pubkey::default(),
                Pubkey::default(),
                Pubkey::default(),
            )
            .unwrap(),
            LiveSovereignSettlementKind::Stable
        );
        assert_eq!(
            classify_live_sovereign_settlement_kind(
                SovereignFinalOutputKind::Usdt,
                usdt,
                usdc,
                usdt,
                extra,
                Pubkey::default(),
                Pubkey::default(),
                Pubkey::default(),
            )
            .unwrap(),
            LiveSovereignSettlementKind::Stable
        );
        assert_eq!(
            classify_live_sovereign_settlement_kind(
                SovereignFinalOutputKind::Extra0,
                extra,
                usdc,
                usdt,
                extra,
                Pubkey::default(),
                Pubkey::default(),
                Pubkey::default(),
            )
            .unwrap(),
            LiveSovereignSettlementKind::Stable
        );
        assert!(classify_live_sovereign_settlement_kind(
            SovereignFinalOutputKind::Usdc,
            Pubkey::new_unique(),
            usdc,
            usdt,
            extra,
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
        )
        .is_err());
    }

    #[test]
    fn classify_live_sovereign_settlement_routes_wsol_to_native_sol_only() {
        let usdc = Pubkey::new_unique();
        let usdt = Pubkey::new_unique();
        let wsol = pubkey_from_str(WSOL_MINT);

        assert_eq!(
            classify_live_sovereign_settlement_kind(
                SovereignFinalOutputKind::Sol,
                wsol,
                usdc,
                usdt,
                Pubkey::default(),
                Pubkey::default(),
                Pubkey::default(),
                Pubkey::default(),
            )
            .unwrap(),
            LiveSovereignSettlementKind::NativeSol
        );
        assert!(classify_live_sovereign_settlement_kind(
            SovereignFinalOutputKind::Sol,
            usdc,
            usdc,
            usdt,
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
        )
        .is_err());
    }

    #[test]
    fn classify_live_sovereign_settlement_auto_rejects_non_stable_outputs() {
        let usdc = Pubkey::new_unique();
        let usdt = Pubkey::new_unique();
        assert!(classify_live_sovereign_settlement_kind(
            SovereignFinalOutputKind::Auto,
            Pubkey::new_unique(),
            usdc,
            usdt,
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
        )
        .is_err());
    }

    #[test]
    fn ensure_usdc_settlement_liquidity_fails_when_coffre_is_insufficient() {
        let err = ensure_usdc_settlement_liquidity(999_999, 1_000_000).unwrap_err();
        assert_eq!(
            err,
            error!(SterlingError::InsufficientUsdcSettlementLiquidity)
        );
    }

    #[test]
    fn validate_claimable_ticket_accepts_already_payable_ticket() {
        let user = Pubkey::new_unique();
        let pool = Pubkey::new_unique();
        let ticket = PayoutTicket {
            pool,
            payout_mint: Pubkey::new_unique(),
            payout_kind: 4,
            user,
            mint_in: Pubkey::new_unique(),
            amount_in: 42,
            usd_micros: 1_500_000,
            destination_ata: Pubkey::new_unique(),
            escrow_mint: Pubkey::new_unique(),
            escrow_ata: Pubkey::new_unique(),
            escrow_amount_locked: 42,
            nonce: 7,
            created_ts: 123,
            settled_ts: 0,
            status: PAYOUT_TICKET_STATUS_REQUESTED,
            funding_state: SOVEREIGN_ESCROW_STATE_REQUESTED,
            escrow_bump: 1,
            route_hint: 0,
            bump: 1,
        };

        assert!(validate_claimable_ticket(&ticket, 7, user, pool).is_ok());
    }

    #[test]
    fn source_escrow_binding_rejects_unfunded_state() {
        let mint = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let escrow_ata = Pubkey::new_unique();

        assert!(assert_source_escrow_binding_parts(
            authority,
            mint,
            42,
            escrow_ata,
            escrow_ata,
            mint,
            42,
            authority,
            SOVEREIGN_ESCROW_STATE_REQUESTED,
        )
        .is_err());
    }

    #[test]
    fn source_escrow_binding_rejects_wrong_locked_amount() {
        let mint = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let escrow_ata = Pubkey::new_unique();

        assert!(assert_source_escrow_binding_parts(
            authority,
            mint,
            41,
            escrow_ata,
            escrow_ata,
            mint,
            42,
            authority,
            SOVEREIGN_ESCROW_STATE_FUNDED_IN_ESCROW,
        )
        .is_err());
    }

    #[test]
    fn source_escrow_binding_rejects_wrong_owner() {
        let mint = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let wrong_authority = Pubkey::new_unique();
        let escrow_ata = Pubkey::new_unique();

        assert!(assert_source_escrow_binding_parts(
            wrong_authority,
            mint,
            42,
            escrow_ata,
            escrow_ata,
            mint,
            42,
            authority,
            SOVEREIGN_ESCROW_STATE_FUNDED_IN_ESCROW,
        )
        .is_err());
    }

    #[test]
    fn source_escrow_binding_rejects_wrong_ata() {
        let mint = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let escrow_ata = Pubkey::new_unique();

        assert!(assert_source_escrow_binding_parts(
            authority,
            mint,
            42,
            Pubkey::new_unique(),
            escrow_ata,
            mint,
            42,
            authority,
            SOVEREIGN_ESCROW_STATE_FUNDED_IN_ESCROW,
        )
        .is_err());
    }

    #[test]
    fn source_escrow_binding_rejects_wrong_mint() {
        let authority = Pubkey::new_unique();
        let escrow_ata = Pubkey::new_unique();

        assert!(assert_source_escrow_binding_parts(
            authority,
            Pubkey::new_unique(),
            42,
            escrow_ata,
            escrow_ata,
            Pubkey::new_unique(),
            42,
            authority,
            SOVEREIGN_ESCROW_STATE_FUNDED_IN_ESCROW,
        )
        .is_err());
    }

    #[test]
    fn live_escrow_route_binding_rejects_wrong_authority_account() {
        let authority = Pubkey::new_unique();
        let mint = pubkey_from_str(USDC_MINT);
        let escrow_ata = expected_live_escrow_ata(authority, mint);

        assert!(assert_live_escrow_route_authority_parts(
            Pubkey::new_unique(),
            authority,
            escrow_ata,
            mint,
            authority,
            escrow_ata,
            mint,
        )
        .is_err());
    }

    #[test]
    fn temporary_wsol_output_account_must_start_empty() {
        let output_key = Pubkey::new_unique();
        let native_destination_key = Pubkey::new_unique();
        let expected_authority = Pubkey::new_unique();

        assert!(assert_wsol_temporary_output_account_parts(
            pubkey_from_str(WSOL_MINT),
            expected_authority,
            1,
            output_key,
            expected_authority,
            native_destination_key,
        )
        .is_err());
    }

    #[test]
    fn legacy_internal_settlement_helper_is_disabled() {
        assert!(legacy_internal_settlement_disabled().is_err());
    }

    fn simulate_live_quote_for_source(
        source_kind: u8,
        requested_output_kind: SovereignFinalOutputKind,
        source_mint: Pubkey,
        pool_base_mint: Pubkey,
        pool_quote_mint: Pubkey,
    ) -> Result<LivePoolQuote> {
        require_live_sovereign_source_kind(source_kind)?;
        let base_vault = Pubkey::new_unique();
        let quote_vault = Pubkey::new_unique();
        let fee_vault_base = Pubkey::new_unique();
        let fee_vault_quote = Pubkey::new_unique();

        let (
            input_vault_key,
            output_vault_key,
            fee_vault_key,
            input_vault_mint,
            output_vault_mint,
            fee_vault_mint,
        ) = if source_mint == pool_base_mint {
            (
                base_vault,
                quote_vault,
                fee_vault_base,
                pool_base_mint,
                pool_quote_mint,
                pool_base_mint,
            )
        } else {
            (
                quote_vault,
                base_vault,
                fee_vault_quote,
                pool_quote_mint,
                pool_base_mint,
                pool_quote_mint,
            )
        };

        quote_live_pool_swap_parts(
            true,
            pool_base_mint,
            pool_quote_mint,
            base_vault,
            quote_vault,
            fee_vault_base,
            fee_vault_quote,
            30,
            source_mint,
            1_000_000,
            input_vault_key,
            output_vault_key,
            fee_vault_key,
            input_vault_mint,
            output_vault_mint,
            fee_vault_mint,
            10_000_000,
            20_000_000,
            requested_output_kind,
            pubkey_from_str(USDC_MINT),
            pubkey_from_str(USDT_MAIN_MINT),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
        )
    }

    fn sample_live_ticket(user: Pubkey) -> PayoutTicket {
        let ticket_key = Pubkey::new_unique();
        let escrow_mint = Pubkey::new_unique();
        let (authority, bump) = expected_ticket_escrow_authority(ticket_key);
        PayoutTicket {
            pool: Pubkey::new_unique(),
            payout_mint: Pubkey::default(),
            payout_kind: 4,
            user,
            mint_in: Pubkey::new_unique(),
            amount_in: 42,
            usd_micros: 1_500_000,
            destination_ata: Pubkey::default(),
            escrow_mint,
            escrow_ata: expected_live_escrow_ata(authority, escrow_mint),
            escrow_amount_locked: 42,
            nonce: 7,
            created_ts: 123,
            settled_ts: 0,
            status: PAYOUT_TICKET_STATUS_REQUESTED,
            funding_state: SOVEREIGN_ESCROW_STATE_FUNDED_IN_ESCROW,
            escrow_bump: bump,
            route_hint: 0,
            bump: 1,
        }
    }

    fn sample_live_claim(user: Pubkey) -> SettlementClaim {
        let claim_key = Pubkey::new_unique();
        let escrow_mint = Pubkey::new_unique();
        let (authority, bump) = expected_claim_escrow_authority(claim_key);
        SettlementClaim {
            pool: Pubkey::new_unique(),
            payout_mint: Pubkey::default(),
            payout_kind: 5,
            user,
            mint_in: Pubkey::new_unique(),
            amount_in: 55,
            usd_micros: 2_000_000,
            due_atoms: 77,
            paid_atoms: 0,
            proof_sig: [0u8; 64],
            destination_ata: Pubkey::default(),
            escrow_mint,
            escrow_ata: expected_live_escrow_ata(authority, escrow_mint),
            escrow_amount_locked: 55,
            nonce: 9,
            created_ts: 456,
            settled_ts: 0,
            status: SETTLEMENT_CLAIM_STATUS_OPEN,
            funding_state: SOVEREIGN_ESCROW_STATE_FUNDED_IN_ESCROW,
            escrow_bump: bump,
            bump: 1,
        }
    }

    fn sample_protocol_debt_pool_and_lot() -> (Pool, ProtocolDebtLot) {
        let pool_key = Pubkey::new_unique();
        let mut pool = Pool {
            owner: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            base_vault: Pubkey::new_unique(),
            quote_vault: Pubkey::new_unique(),
            lp_mint: Pubkey::new_unique(),
            fee_vault_base: Pubkey::new_unique(),
            fee_vault_quote: Pubkey::new_unique(),
            base_value_usd_micros: 0,
            quote_value_usd_micros: 0,
            true_cash: true,
            cash_backed: true,
            real_peg: true,
            sovereign: true,
            fee_bps: 30,
            active: true,
            swap_cashback_bps: 0,
            protocol_fee_debt_usd_micros: 700,
            protocol_fee_debt_count: 1,
            protocol_fee_debt_last_ts: 0,
            created_at: 0,
            last_swap_ts: 0,
            total_base_volume: 0,
            total_quote_volume: 0,
            swap_count: 0,
            bump: 1,
        };
        let nonce = 3;
        let escrow_mint = Pubkey::new_unique();
        let (authority, bump) = expected_protocol_debt_lot_escrow_authority(pool_key, nonce);
        pool.owner = pool_key;
        let lot = ProtocolDebtLot {
            nonce,
            mint_in: Pubkey::new_unique(),
            amount_in: 42,
            usd_micros: 700,
            created_ts: 10,
            escrow_mint,
            escrow_ata: expected_live_escrow_ata(authority, escrow_mint),
            escrow_amount_locked: 42,
            escrow_bump: bump,
            funding_state: SOVEREIGN_ESCROW_STATE_FUNDED_IN_ESCROW,
            route_hint: 0,
            status: PROTOCOL_DEBT_LOT_OPEN,
        };
        (pool, lot)
    }

    #[test]
    fn valued_live_route_quotes_usdc_output() {
        let asset = Pubkey::new_unique();
        let usdc = pubkey_from_str(USDC_MINT);
        let quote =
            simulate_live_quote_for_source(1, SovereignFinalOutputKind::Usdc, asset, asset, usdc)
                .unwrap();
        assert_eq!(quote.settlement_kind, LiveSovereignSettlementKind::Stable);
        assert_eq!(quote.output_mint, usdc);
        assert!(quote.payout_amount > 0);
    }

    #[test]
    fn valued_live_route_quotes_usdt_output() {
        let asset = Pubkey::new_unique();
        let usdt = pubkey_from_str(USDT_MAIN_MINT);
        let quote =
            simulate_live_quote_for_source(1, SovereignFinalOutputKind::Usdt, asset, asset, usdt)
                .unwrap();
        assert_eq!(quote.settlement_kind, LiveSovereignSettlementKind::Stable);
        assert_eq!(quote.output_mint, usdt);
        assert!(quote.payout_amount > 0);
    }

    #[test]
    fn valued_live_route_quotes_sol_output() {
        let asset = Pubkey::new_unique();
        let config_key = Pubkey::new_unique();
        let beneficiary = Pubkey::new_unique();
        let wsol = pubkey_from_str(WSOL_MINT);
        let quote =
            simulate_live_quote_for_source(1, SovereignFinalOutputKind::Sol, asset, asset, wsol)
                .unwrap();
        assert_eq!(
            quote.settlement_kind,
            LiveSovereignSettlementKind::NativeSol
        );
        assert!(validate_live_output_destination_parts(
            quote.settlement_kind,
            beneficiary,
            config_key,
            wsol,
            0,
            Pubkey::new_unique(),
            wsol,
            config_key,
            Some(beneficiary),
            true,
            false,
        )
        .is_ok());
    }

    #[test]
    fn direct_stable_live_route_quotes_usdc_output() {
        let usdc = pubkey_from_str(USDC_MINT);
        let usdt = pubkey_from_str(USDT_MAIN_MINT);
        let quote =
            simulate_live_quote_for_source(2, SovereignFinalOutputKind::Usdc, usdt, usdc, usdt)
                .unwrap();
        assert_eq!(quote.settlement_kind, LiveSovereignSettlementKind::Stable);
        assert_eq!(quote.output_mint, usdc);
    }

    #[test]
    fn direct_stable_live_route_quotes_usdt_output() {
        let usdc = pubkey_from_str(USDC_MINT);
        let usdt = pubkey_from_str(USDT_MAIN_MINT);
        let quote =
            simulate_live_quote_for_source(2, SovereignFinalOutputKind::Usdt, usdc, usdc, usdt)
                .unwrap();
        assert_eq!(quote.settlement_kind, LiveSovereignSettlementKind::Stable);
        assert_eq!(quote.output_mint, usdt);
    }

    #[test]
    fn direct_stable_live_route_quotes_sol_output() {
        let config_key = Pubkey::new_unique();
        let beneficiary = Pubkey::new_unique();
        let usdc = pubkey_from_str(USDC_MINT);
        let wsol = pubkey_from_str(WSOL_MINT);
        let quote =
            simulate_live_quote_for_source(2, SovereignFinalOutputKind::Sol, usdc, wsol, usdc)
                .unwrap();
        assert_eq!(
            quote.settlement_kind,
            LiveSovereignSettlementKind::NativeSol
        );
        assert!(validate_live_output_destination_parts(
            quote.settlement_kind,
            beneficiary,
            config_key,
            wsol,
            0,
            Pubkey::new_unique(),
            wsol,
            config_key,
            Some(beneficiary),
            true,
            false,
        )
        .is_ok());
    }

    #[test]
    fn live_quote_rejects_wrong_pool_route() {
        let asset = Pubkey::new_unique();
        let usdc = pubkey_from_str(USDC_MINT);
        let err = quote_live_pool_swap_parts(
            true,
            asset,
            usdc,
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            30,
            asset,
            1_000_000,
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            asset,
            usdc,
            asset,
            10_000_000,
            20_000_000,
            SovereignFinalOutputKind::Usdc,
            usdc,
            pubkey_from_str(USDT_MAIN_MINT),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
        )
        .unwrap_err();
        assert_eq!(err, error!(SterlingError::InvalidLiveSovereignPoolRoute));
    }

    #[test]
    fn ticket_funded_live_swap_transitions_to_settled() {
        let user = Pubkey::new_unique();
        let mut ticket = sample_live_ticket(user);
        let outcome = LivePoolExecutionOutcome {
            source_mint: ticket.mint_in,
            source_amount: ticket.amount_in,
            payout_mint: pubkey_from_str(USDC_MINT),
            payout_amount: 123,
            destination_key: Pubkey::new_unique(),
        };

        begin_ticket_live_execution(&mut ticket, user, user).unwrap();
        finalize_ticket_live_execution(&mut ticket, outcome, 999).unwrap();

        assert_eq!(ticket.status, PAYOUT_TICKET_STATUS_SETTLED);
        assert_eq!(ticket.funding_state, SOVEREIGN_ESCROW_STATE_SETTLED);
        assert_eq!(ticket.settled_ts, 999);
        assert_eq!(ticket.payout_mint, pubkey_from_str(USDC_MINT));
    }

    #[test]
    fn claim_funded_live_swap_transitions_to_settled() {
        let user = Pubkey::new_unique();
        let mut claim = sample_live_claim(user);
        let outcome = LivePoolExecutionOutcome {
            source_mint: claim.mint_in,
            source_amount: claim.amount_in,
            payout_mint: pubkey_from_str(USDT_MAIN_MINT),
            payout_amount: 321,
            destination_key: Pubkey::new_unique(),
        };

        begin_claim_live_execution(&mut claim, user).unwrap();
        finalize_claim_live_execution(&mut claim, outcome, 1001).unwrap();

        assert_eq!(claim.status, SETTLEMENT_CLAIM_STATUS_PAID);
        assert_eq!(claim.funding_state, SOVEREIGN_ESCROW_STATE_SETTLED);
        assert_eq!(claim.paid_atoms, 321);
        assert_eq!(claim.settled_ts, 1001);
        assert_eq!(claim.proof_sig, live_execution_proof_sig());
    }

    #[test]
    fn protocol_debt_funded_live_swap_transitions_to_settled() {
        let (mut pool, mut lot) = sample_protocol_debt_pool_and_lot();
        begin_protocol_debt_lot_live_execution(&mut lot).unwrap();
        finalize_protocol_debt_lot_live_execution(&mut pool, &mut lot, 77).unwrap();

        assert_eq!(lot.status, PROTOCOL_DEBT_LOT_SETTLED);
        assert_eq!(lot.funding_state, SOVEREIGN_ESCROW_STATE_SETTLED);
        assert_eq!(pool.protocol_fee_debt_usd_micros, 0);
        assert_eq!(pool.protocol_fee_debt_count, 0);
        assert_eq!(pool.protocol_fee_debt_last_ts, 77);
    }

    #[test]
    fn ticket_live_execution_rejects_unfunded_state() {
        let user = Pubkey::new_unique();
        let mut ticket = sample_live_ticket(user);
        ticket.funding_state = SOVEREIGN_ESCROW_STATE_REQUESTED;
        assert!(begin_ticket_live_execution(&mut ticket, user, user).is_err());
    }

    #[test]
    fn claim_live_execution_rejects_unfunded_state() {
        let user = Pubkey::new_unique();
        let mut claim = sample_live_claim(user);
        claim.funding_state = SOVEREIGN_ESCROW_STATE_REQUESTED;
        assert!(begin_claim_live_execution(&mut claim, user).is_err());
    }

    #[test]
    fn protocol_debt_live_execution_rejects_unfunded_state() {
        let (_, mut lot) = sample_protocol_debt_pool_and_lot();
        lot.funding_state = SOVEREIGN_ESCROW_STATE_REQUESTED;
        assert!(begin_protocol_debt_lot_live_execution(&mut lot).is_err());
    }

    #[test]
    fn ticket_double_settlement_is_rejected() {
        let user = Pubkey::new_unique();
        let mut ticket = sample_live_ticket(user);
        let outcome = LivePoolExecutionOutcome {
            source_mint: ticket.mint_in,
            source_amount: ticket.amount_in,
            payout_mint: pubkey_from_str(USDC_MINT),
            payout_amount: 123,
            destination_key: Pubkey::new_unique(),
        };

        begin_ticket_live_execution(&mut ticket, user, user).unwrap();
        finalize_ticket_live_execution(&mut ticket, outcome, 999).unwrap();
        assert!(begin_ticket_live_execution(&mut ticket, user, user).is_err());
        assert!(finalize_ticket_live_execution(&mut ticket, outcome, 1000).is_err());
    }

    #[test]
    fn floor_price_check_accepts_post_swap_price_at_or_above_floor() {
        let post_base_atoms = 10_000_000_000u64;
        let post_quote_atoms = 925_000_000_000_000u64;
        assert!(
            enforce_floor_price_post_swap(FLOOR_PRICE_USD_MICROS, post_base_atoms, post_quote_atoms)
                .is_ok()
        );
    }

    #[test]
    fn floor_price_check_rejects_post_swap_price_below_floor() {
        let post_base_atoms = 10_000_000_000u64;
        let post_quote_atoms = 924_999_999_999_999u64;
        assert!(
            enforce_floor_price_post_swap(FLOOR_PRICE_USD_MICROS, post_base_atoms, post_quote_atoms)
                .is_err()
        );
    }
}

/// amount_atoms: montant SPL brut (atoms)
/// valuation_usd_micros_per_1ui: USD micros pour 1 token UI
fn atoms_to_usd_micros(
    amount_atoms: u64,
    decimals: u8,
    valuation_usd_micros_per_1ui: u64,
) -> Result<u64> {
    let denom = pow10_u128(decimals);
    require!(denom > 0, SterlingError::MathOverflow);

    // usd_micros = amount_atoms * valuation / 10^decimals
    let usd = (amount_atoms as u128)
        .saturating_mul(valuation_usd_micros_per_1ui as u128)
        .checked_div(denom)
        .ok_or(SterlingError::MathOverflow)?;

    Ok(u64::try_from(usd).map_err(|_| SterlingError::MathOverflow)?)
}

/// USD micros -> atoms for payout token decimals (6 = USDC/USDT, 9 = x1000)
fn usd_micros_to_atoms(usd_micros: u64, payout_decimals: u8) -> Result<u64> {
    // atoms = usd_micros * 10^decimals / 1_000_000
    let scale = pow10_u128(payout_decimals);
    let num = (usd_micros as u128).saturating_mul(scale);
    let atoms = num
        .checked_div(USD_MICROS as u128)
        .ok_or(SterlingError::MathOverflow)?;
    Ok(u64::try_from(atoms).map_err(|_| SterlingError::MathOverflow)?)
}

fn enforce_ticket_cap_usd_micros(usd_micros: u64) -> Result<()> {
    require!(
        usd_micros <= DEFAULT_TICKET_CAP_USD_MICROS,
        SterlingError::PayoutTooLarge
    );
    Ok(())
}

fn set_pool_base_vault(pool: &mut Pool, base_mint: Pubkey, base_vault: Pubkey) -> Result<()> {
    require!(pool.base_mint == base_mint, SterlingError::InvalidAccount);
    pool.base_vault = base_vault;
    Ok(())
}

fn set_pool_quote_vault(pool: &mut Pool, quote_mint: Pubkey, quote_vault: Pubkey) -> Result<()> {
    require!(pool.quote_mint == quote_mint, SterlingError::InvalidAccount);
    pool.quote_vault = quote_vault;
    Ok(())
}

fn set_pool_lp_mint(pool: &mut Pool, lp_mint: Pubkey) -> Result<()> {
    pool.lp_mint = lp_mint;
    Ok(())
}

fn set_pool_fee_vault_base(
    pool: &mut Pool,
    base_mint: Pubkey,
    fee_vault_base: Pubkey,
) -> Result<()> {
    require!(pool.base_mint == base_mint, SterlingError::InvalidAccount);
    pool.fee_vault_base = fee_vault_base;
    Ok(())
}

fn set_pool_fee_vault_quote(
    pool: &mut Pool,
    quote_mint: Pubkey,
    fee_vault_quote: Pubkey,
) -> Result<()> {
    require!(pool.quote_mint == quote_mint, SterlingError::InvalidAccount);
    pool.fee_vault_quote = fee_vault_quote;
    Ok(())
}

fn initialize_asset_registry(
    asset_registry: &mut AssetRegistry,
    mint: Pubkey,
    valuation_usd_micros: u64,
    decimals: u8,
    is_lp: bool,
    bump: u8,
) -> Result<()> {
    require!(valuation_usd_micros > 0, SterlingError::InvalidAmount);
    asset_registry.mint = mint;
    asset_registry.valuation_usd_micros = valuation_usd_micros;
    asset_registry.decimals = decimals;
    asset_registry.is_lp = is_lp;
    asset_registry.active = true;
    asset_registry.bump = bump;
    Ok(())
}

fn validate_liquidity_accounts(
    pool: &Pool,
    user: Pubkey,
    user_base_ata: &TokenAccount,
    user_quote_ata: &TokenAccount,
    user_lp_ata: &TokenAccount,
    base_vault_key: Pubkey,
    base_vault: &TokenAccount,
    quote_vault_key: Pubkey,
    quote_vault: &TokenAccount,
    lp_mint_key: Pubkey,
    lp_mint: &Mint,
) -> Result<()> {
    require!(pool.active, SterlingError::InactivePool);
    require!(user_base_ata.owner == user, SterlingError::InvalidAccount);
    require!(user_quote_ata.owner == user, SterlingError::InvalidAccount);
    require!(user_lp_ata.owner == user, SterlingError::InvalidAccount);
    require!(
        user_base_ata.mint == pool.base_mint,
        SterlingError::InvalidAccount
    );
    require!(
        user_quote_ata.mint == pool.quote_mint,
        SterlingError::InvalidAccount
    );
    require!(
        user_lp_ata.mint == lp_mint_key,
        SterlingError::InvalidAccount
    );
    require!(
        base_vault_key == pool.base_vault,
        SterlingError::InvalidAccount
    );
    require!(
        quote_vault_key == pool.quote_vault,
        SterlingError::InvalidAccount
    );
    require!(lp_mint_key == pool.lp_mint, SterlingError::InvalidAccount);
    require!(
        base_vault.mint == pool.base_mint,
        SterlingError::InvalidAccount
    );
    require!(
        quote_vault.mint == pool.quote_mint,
        SterlingError::InvalidAccount
    );
    Ok(())
}

fn compute_lp_out(
    base_reserve: u64,
    quote_reserve: u64,
    lp_supply: u64,
    base_amount_in: u64,
    quote_amount_in: u64,
) -> Result<u64> {
    if lp_supply == 0 {
        require!(base_reserve == 0, SterlingError::InvalidState);
        require!(quote_reserve == 0, SterlingError::InvalidState);
        return u64::try_from(_integer_sqrt_u128(
            (base_amount_in as u128).saturating_mul(quote_amount_in as u128),
        ))
        .map_err(|_| error!(SterlingError::MathOverflow));
    }

    require!(base_reserve > 0, SterlingError::InvalidState);
    require!(quote_reserve > 0, SterlingError::InvalidState);

    let lp_from_base = (base_amount_in as u128)
        .saturating_mul(lp_supply as u128)
        .checked_div(base_reserve as u128)
        .ok_or(SterlingError::MathOverflow)?;
    let lp_from_quote = (quote_amount_in as u128)
        .saturating_mul(lp_supply as u128)
        .checked_div(quote_reserve as u128)
        .ok_or(SterlingError::MathOverflow)?;

    u64::try_from(lp_from_base.min(lp_from_quote)).map_err(|_| error!(SterlingError::MathOverflow))
}

fn compute_liquidity_outs(
    base_reserve: u64,
    quote_reserve: u64,
    lp_supply: u64,
    lp_amount_in: u64,
) -> Result<(u64, u64)> {
    require!(lp_supply > 0, SterlingError::ZeroLp);

    let base_out = (lp_amount_in as u128)
        .saturating_mul(base_reserve as u128)
        .checked_div(lp_supply as u128)
        .ok_or(SterlingError::MathOverflow)? as u64;
    let quote_out = (lp_amount_in as u128)
        .saturating_mul(quote_reserve as u128)
        .checked_div(lp_supply as u128)
        .ok_or(SterlingError::MathOverflow)? as u64;

    Ok((base_out, quote_out))
}

fn transfer_tokens<'info>(
    token_program: &Program<'info, Token>,
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    token::transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from,
                to,
                authority,
            },
        ),
        amount,
    )
}

fn transfer_tokens_signed<'info>(
    token_program: &Program<'info, Token>,
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    amount: u64,
) -> Result<()> {
    token::transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from,
                to,
                authority,
            },
        )
        .with_signer(signer_seeds),
        amount,
    )
}

fn mint_tokens_signed<'info>(
    token_program: &Program<'info, Token>,
    mint: AccountInfo<'info>,
    to: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    amount: u64,
) -> Result<()> {
    token::mint_to(
        CpiContext::new(
            token_program.to_account_info(),
            MintTo {
                mint,
                to,
                authority,
            },
        )
        .with_signer(signer_seeds),
        amount,
    )
}

fn burn_tokens<'info>(
    token_program: &Program<'info, Token>,
    mint: AccountInfo<'info>,
    from: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    token::burn(
        CpiContext::new(
            token_program.to_account_info(),
            Burn {
                mint,
                from,
                authority,
            },
        ),
        amount,
    )
}

fn burn_tokens_signed<'info>(
    token_program: &Program<'info, Token>,
    mint: AccountInfo<'info>,
    from: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    amount: u64,
) -> Result<()> {
    token::burn(
        CpiContext::new(
            token_program.to_account_info(),
            Burn {
                mint,
                from,
                authority,
            },
        )
        .with_signer(signer_seeds),
        amount,
    )
}

fn collect_stable_fee_vault<'info>(
    pool: Pubkey,
    config: &Account<'info, Config>,
    token_program: &Program<'info, Token>,
    fee_vault: &Account<'info, TokenAccount>,
    treasury_usdc_ata: &Account<'info, TokenAccount>,
    treasury_usdt_ata: &Account<'info, TokenAccount>,
    extra_fee_targets: &[StableFinalOutputTarget<'info>],
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    let (treasury_info, treasury_key) = if fee_vault.mint == config.usdc_mint {
        (treasury_usdc_ata.to_account_info(), treasury_usdc_ata.key())
    } else if fee_vault.mint == config.usdt_mint {
        (treasury_usdt_ata.to_account_info(), treasury_usdt_ata.key())
    } else if let Some(target) = extra_fee_targets
        .iter()
        .find(|target| target.payout_mint == fee_vault.mint)
    {
        (target.destination_info.clone(), target.destination_key)
    } else {
        return Ok(());
    };

    if fee_vault.amount == 0 {
        return Ok(());
    }

    transfer_tokens_signed(
        token_program,
        fee_vault.to_account_info(),
        treasury_info,
        config.to_account_info(),
        signer_seeds,
        fee_vault.amount,
    )?;

    emit!(FeeCollectedToTreasuryEvent {
        pool,
        fee_vault: fee_vault.key(),
        mint: fee_vault.mint,
        amount: fee_vault.amount,
        treasury_ata: treasury_key,
        ts: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

fn live_execution_proof_sig() -> [u8; 64] {
    let mut proof = [0u8; 64];
    proof[0] = SettlementProofKind::LiveFundingExecution as u8;
    proof
}

fn legacy_internal_settlement_disabled() -> Result<()> {
    err!(SterlingError::LegacyInternalSettlementDisabled)
}

fn expected_ticket_escrow_authority(ticket_key: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"ticket_live_escrow", ticket_key.as_ref()], &crate::ID)
}

fn expected_claim_escrow_authority(claim_key: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"claim_live_escrow", claim_key.as_ref()], &crate::ID)
}

fn expected_protocol_debt_lot_escrow_authority(pool_key: Pubkey, nonce: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"debt_live_escrow", pool_key.as_ref(), &nonce.to_le_bytes()],
        &crate::ID,
    )
}

fn expected_live_escrow_ata(owner: Pubkey, mint: Pubkey) -> Pubkey {
    get_associated_token_address(&owner, &mint)
}

fn assert_source_escrow_binding(
    source_escrow: &TokenAccount,
    source_escrow_key: Pubkey,
    expected_escrow_ata: Pubkey,
    expected_escrow_mint: Pubkey,
    expected_locked_amount: u64,
    expected_authority: Pubkey,
    funding_state: u8,
) -> Result<()> {
    assert_source_escrow_binding_parts(
        source_escrow.owner,
        source_escrow.mint,
        source_escrow.amount,
        source_escrow_key,
        expected_escrow_ata,
        expected_escrow_mint,
        expected_locked_amount,
        expected_authority,
        funding_state,
    )
}

fn assert_source_escrow_binding_parts(
    source_owner: Pubkey,
    source_mint: Pubkey,
    source_amount: u64,
    source_escrow_key: Pubkey,
    expected_escrow_ata: Pubkey,
    expected_escrow_mint: Pubkey,
    expected_locked_amount: u64,
    expected_authority: Pubkey,
    funding_state: u8,
) -> Result<()> {
    require!(
        funding_state == SOVEREIGN_ESCROW_STATE_FUNDED_IN_ESCROW,
        SterlingError::SovereignEscrowNotFunded
    );
    require!(
        source_escrow_key == expected_escrow_ata,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        source_owner == expected_authority,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        source_mint == expected_escrow_mint,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(expected_locked_amount > 0, SterlingError::InvalidAmount);
    require!(
        source_amount == expected_locked_amount,
        SterlingError::InvalidSovereignEscrowFundingAmount
    );
    Ok(())
}

fn assert_live_escrow_route_authority_parts(
    source_escrow_authority_key: Pubkey,
    source_escrow_owner: Pubkey,
    source_escrow_key: Pubkey,
    source_escrow_mint: Pubkey,
    expected_escrow_authority: Pubkey,
    expected_escrow_ata: Pubkey,
    expected_source_mint: Pubkey,
) -> Result<()> {
    require!(
        source_escrow_authority_key == expected_escrow_authority,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        source_escrow_owner == expected_escrow_authority,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        source_escrow_key == expected_escrow_ata,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        source_escrow_mint == expected_source_mint,
        SterlingError::InvalidSovereignEscrowBinding
    );
    Ok(())
}

fn assert_wsol_temporary_output_account(
    output_destination: &TokenAccount,
    output_destination_key: Pubkey,
    expected_authority: Pubkey,
    native_destination: &AccountInfo,
) -> Result<()> {
    assert_wsol_temporary_output_account_parts(
        output_destination.mint,
        output_destination.owner,
        output_destination.amount,
        output_destination_key,
        expected_authority,
        native_destination.key(),
    )
}

fn assert_wsol_temporary_output_account_parts(
    output_mint: Pubkey,
    output_owner: Pubkey,
    output_amount: u64,
    output_destination_key: Pubkey,
    expected_authority: Pubkey,
    native_destination_key: Pubkey,
) -> Result<()> {
    require!(
        output_mint == pubkey_from_str(WSOL_MINT),
        SterlingError::InvalidLiveSovereignPoolRoute
    );
    require!(
        output_owner == expected_authority,
        SterlingError::InvalidLiveSovereignPoolRoute
    );
    require!(output_amount == 0, SterlingError::WsolOutputAccountNotEmpty);
    require!(
        output_destination_key != native_destination_key,
        SterlingError::InvalidNativeSolDestination
    );
    Ok(())
}

fn sync_ticket_escrow_funding_state(
    ticket_key: Pubkey,
    ticket: &mut PayoutTicket,
    source_escrow: &TokenAccount,
    source_escrow_key: Pubkey,
) -> Result<()> {
    let (expected_authority, expected_bump) = expected_ticket_escrow_authority(ticket_key);
    let expected_escrow_ata = expected_live_escrow_ata(expected_authority, ticket.escrow_mint);
    require!(
        ticket.escrow_bump == expected_bump,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        ticket.escrow_ata == expected_escrow_ata,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        source_escrow_key == ticket.escrow_ata,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        source_escrow.owner == expected_authority,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        source_escrow.mint == ticket.escrow_mint,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        source_escrow.amount == ticket.escrow_amount_locked,
        SterlingError::InvalidSovereignEscrowFundingAmount
    );
    ticket.funding_state = SOVEREIGN_ESCROW_STATE_FUNDED_IN_ESCROW;
    Ok(())
}

fn sync_claim_escrow_funding_state(
    claim: &mut SettlementClaim,
    claim_key: Pubkey,
    source_escrow: &TokenAccount,
    source_escrow_key: Pubkey,
) -> Result<()> {
    let (expected_authority, expected_bump) = expected_claim_escrow_authority(claim_key);
    let expected_escrow_ata = expected_live_escrow_ata(expected_authority, claim.escrow_mint);
    require!(
        claim.escrow_bump == expected_bump,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        claim.escrow_ata == expected_escrow_ata,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        source_escrow_key == claim.escrow_ata,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        source_escrow.owner == expected_authority,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        source_escrow.mint == claim.escrow_mint,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        source_escrow.amount == claim.escrow_amount_locked,
        SterlingError::InvalidSovereignEscrowFundingAmount
    );
    claim.funding_state = SOVEREIGN_ESCROW_STATE_FUNDED_IN_ESCROW;
    Ok(())
}

fn sync_protocol_debt_lot_escrow_funding_state(
    pool_key: Pubkey,
    lot: &mut ProtocolDebtLot,
    source_escrow: &TokenAccount,
    source_escrow_key: Pubkey,
) -> Result<()> {
    let (expected_authority, expected_bump) =
        expected_protocol_debt_lot_escrow_authority(pool_key, lot.nonce);
    let expected_escrow_ata = expected_live_escrow_ata(expected_authority, lot.escrow_mint);
    require!(
        lot.escrow_bump == expected_bump,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        lot.escrow_ata == expected_escrow_ata,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        source_escrow_key == lot.escrow_ata,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        source_escrow.owner == expected_authority,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        source_escrow.mint == lot.escrow_mint,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        source_escrow.amount == lot.escrow_amount_locked,
        SterlingError::InvalidSovereignEscrowFundingAmount
    );
    lot.funding_state = SOVEREIGN_ESCROW_STATE_FUNDED_IN_ESCROW;
    Ok(())
}

fn require_buffered_route_escrow_accounts<'info>(
    route_accounts: &SovereignConvertRouteAccounts<'info>,
) -> Result<SovereignBufferedEscrowAccounts<'info>> {
    Ok(SovereignBufferedEscrowAccounts {
        authority_info: route_accounts
            .source_escrow_authority
            .clone()
            .ok_or_else(|| error!(SterlingError::InvalidSovereignAccountsLayout))?,
        ata_info: route_accounts
            .source_escrow_ata
            .clone()
            .ok_or_else(|| error!(SterlingError::InvalidSovereignAccountsLayout))?,
        mint_info: route_accounts
            .source_escrow_mint
            .clone()
            .ok_or_else(|| error!(SterlingError::InvalidSovereignAccountsLayout))?,
    })
}

fn validate_buffered_route_escrow_accounts<'info>(
    buffered_escrow: &SovereignBufferedEscrowAccounts<'info>,
    expected_authority: Pubkey,
    expected_escrow_ata: Pubkey,
    expected_escrow_mint: Pubkey,
    expected_locked_amount: u64,
    funding_state: u8,
) -> Result<TokenAccount> {
    let source_escrow = load_token_account_snapshot(&buffered_escrow.ata_info)?;
    let _source_mint = load_mint_snapshot(&buffered_escrow.mint_info)?;
    assert_live_escrow_route_authority_parts(
        buffered_escrow.authority_info.key(),
        source_escrow.owner,
        buffered_escrow.ata_info.key(),
        source_escrow.mint,
        expected_authority,
        expected_escrow_ata,
        expected_escrow_mint,
    )?;
    require!(
        buffered_escrow.mint_info.key() == expected_escrow_mint,
        SterlingError::InvalidSovereignEscrowBinding
    );
    assert_source_escrow_binding(
        &source_escrow,
        buffered_escrow.ata_info.key(),
        expected_escrow_ata,
        expected_escrow_mint,
        expected_locked_amount,
        expected_authority,
        funding_state,
    )?;
    Ok(source_escrow)
}

fn burn_tokens_signed_account_info<'info>(
    token_program_info: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    from: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    amount: u64,
) -> Result<()> {
    token::burn(
        CpiContext::new(
            token_program_info,
            Burn {
                mint,
                from,
                authority,
            },
        )
        .with_signer(signer_seeds),
        amount,
    )
}

#[inline(never)]
fn settle_live_pool_exchange_from_escrow<'info>(
    config: &Account<'info, Config>,
    token_program: &Program<'info, Token>,
    source_kind: u8,
    beneficiary: Pubkey,
    source_mint: Pubkey,
    expected_escrow_authority: Pubkey,
    escrow_signer_seeds: &[&[&[u8]]],
    requested_output_kind: SovereignFinalOutputKind,
    route: SovereignEscrowLiveSwapAccounts<'info>,
) -> Result<LivePoolExecutionOutcome> {
    require!(
        matches!(source_kind, 4 | 5 | 6),
        SterlingError::InvalidAccount
    );
    require!(
        route.swap_pool_info.owner == &crate::ID,
        SterlingError::InvalidLiveSovereignPoolRoute
    );

    let source_escrow = load_token_account_snapshot(&route.source_escrow_ata)?;
    let mut swap_pool = load_pool_snapshot(&route.swap_pool_info)?;
    let input_vault = load_token_account_snapshot(&route.input_vault)?;
    let output_vault = load_token_account_snapshot(&route.output_vault)?;
    let fee_vault_in = load_token_account_snapshot(&route.fee_vault_in)?;
    let output_destination = load_token_account_snapshot(&route.output_destination)?;
    let expected_escrow_ata = expected_live_escrow_ata(expected_escrow_authority, source_mint);

    require!(
        !route.source_escrow_authority.executable,
        SterlingError::InvalidSovereignEscrowBinding
    );
    assert_live_escrow_route_authority_parts(
        route.source_escrow_authority.key(),
        source_escrow.owner,
        route.source_escrow_ata.key(),
        source_escrow.mint,
        expected_escrow_authority,
        expected_escrow_ata,
        source_mint,
    )?;

    let amount_in = source_escrow.amount;
    require!(amount_in > 0, SterlingError::InvalidAmount);
    require!(swap_pool.active, SterlingError::InactivePool);

    let input_is_base = if swap_pool.base_mint == source_mint {
        require!(
            route.input_vault.key() == swap_pool.base_vault,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            route.output_vault.key() == swap_pool.quote_vault,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            route.fee_vault_in.key() == swap_pool.fee_vault_base,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            input_vault.mint == swap_pool.base_mint,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            output_vault.mint == swap_pool.quote_mint,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            fee_vault_in.mint == swap_pool.base_mint,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        true
    } else if swap_pool.quote_mint == source_mint {
        require!(
            route.input_vault.key() == swap_pool.quote_vault,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            route.output_vault.key() == swap_pool.base_vault,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            route.fee_vault_in.key() == swap_pool.fee_vault_quote,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            input_vault.mint == swap_pool.quote_mint,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            output_vault.mint == swap_pool.base_mint,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            fee_vault_in.mint == swap_pool.quote_mint,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        false
    } else {
        return err!(SterlingError::InvalidLiveSovereignPoolRoute);
    };

    let output_mint = if input_is_base {
        swap_pool.quote_mint
    } else {
        swap_pool.base_mint
    };
    let settlement_kind = classify_live_sovereign_settlement_kind(
        requested_output_kind,
        output_mint,
        config.usdc_mint,
        config.usdt_mint,
        config.extra_payout_mint_0,
        config.extra_payout_mint_1,
        config.extra_payout_mint_2,
        config.extra_payout_mint_3,
    )?;

    match settlement_kind {
        LiveSovereignSettlementKind::Stable => {
            require!(
                output_destination.owner == beneficiary,
                SterlingError::InvalidAccount
            );
            require!(
                output_destination.mint == output_mint,
                SterlingError::InvalidAccount
            );
        }
        LiveSovereignSettlementKind::NativeSol => {
            require!(
                output_destination.mint == output_mint,
                SterlingError::InvalidAccount
            );
            require!(
                output_destination.owner == config.key(),
                SterlingError::InvalidLiveSovereignPoolRoute
            );
            let native_destination = route
                .native_destination
                .clone()
                .ok_or_else(|| error!(SterlingError::InvalidNativeSolDestination))?;
            require!(
                native_destination.key() == beneficiary,
                SterlingError::InvalidNativeSolDestination
            );
            require!(
                native_destination.is_writable,
                SterlingError::InvalidNativeSolDestination
            );
            require!(
                !native_destination.executable,
                SterlingError::InvalidNativeSolDestination
            );
            require!(
                native_destination.key() != config.key(),
                SterlingError::InvalidNativeSolDestination
            );
            assert_wsol_temporary_output_account(
                &output_destination,
                route.output_destination.key(),
                config.key(),
                &native_destination,
            )?;
        }
    }

    let fee_bps = swap_pool.fee_bps as u128;
    let fee_amount = (amount_in as u128)
        .saturating_mul(fee_bps)
        .checked_div(10_000u128)
        .ok_or(SterlingError::MathOverflow)? as u64;
    let amount_in_less_fee = (amount_in as u128)
        .saturating_mul(10_000u128.saturating_sub(fee_bps))
        .checked_div(10_000u128)
        .ok_or(SterlingError::MathOverflow)? as u64;
    let x = input_vault.amount as u128;
    let y = output_vault.amount as u128;
    let out = (amount_in_less_fee as u128)
        .saturating_mul(y)
        .checked_div(x.saturating_add(amount_in_less_fee as u128))
        .ok_or(SterlingError::MathOverflow)? as u64;

    require!(out > 0, SterlingError::InvalidAmount);
    require!(
        output_vault.amount >= out,
        SterlingError::InsufficientLiquidity
    );

    let config_signer_seeds: &[&[&[u8]]] = &[&[b"config", &[config.bump]]];

    transfer_tokens_signed(
        token_program,
        route.source_escrow_ata.clone(),
        route.input_vault.clone(),
        route.source_escrow_authority.clone(),
        escrow_signer_seeds,
        amount_in,
    )?;

    transfer_tokens_signed(
        token_program,
        route.input_vault.clone(),
        route.fee_vault_in.clone(),
        config.to_account_info(),
        config_signer_seeds,
        fee_amount,
    )?;

    transfer_tokens_signed(
        token_program,
        route.output_vault.clone(),
        route.output_destination.clone(),
        config.to_account_info(),
        config_signer_seeds,
        out,
    )?;

    let (effective_payout_mint, effective_destination_key, effective_payout_amount) =
        if settlement_kind == LiveSovereignSettlementKind::NativeSol {
            let native_destination = route
                .native_destination
                .clone()
                .ok_or_else(|| error!(SterlingError::InvalidNativeSolDestination))?;
            token::close_account(
                CpiContext::new(
                    token_program.to_account_info(),
                    CloseAccount {
                        account: route.output_destination.clone(),
                        destination: native_destination.clone(),
                        authority: config.to_account_info(),
                    },
                )
                .with_signer(config_signer_seeds),
            )?;
            (native_sol_marker_pubkey(), native_destination.key(), out)
        } else {
            (output_mint, route.output_destination.key(), out)
        };

    swap_pool.swap_count = swap_pool.swap_count.saturating_add(1);
    swap_pool.last_swap_ts = Clock::get()?.unix_timestamp;
    if input_is_base {
        swap_pool.total_base_volume = swap_pool.total_base_volume.saturating_add(amount_in);
        swap_pool.total_quote_volume = swap_pool.total_quote_volume.saturating_add(out);
    } else {
        swap_pool.total_quote_volume = swap_pool.total_quote_volume.saturating_add(amount_in);
        swap_pool.total_base_volume = swap_pool.total_base_volume.saturating_add(out);
    }

    let mut swap_pool_data = route.swap_pool_info.try_borrow_mut_data()?;
    let mut swap_pool_out: &mut [u8] = &mut swap_pool_data;
    swap_pool.try_serialize(&mut swap_pool_out)?;

    emit!(SwapExecuted {
        pool: route.swap_pool_info.key(),
        user: beneficiary,
        side: if input_is_base {
            FeeSide::Base
        } else {
            FeeSide::Quote
        },
        mint_in: source_mint,
        amount_in,
        mint_out: effective_payout_mint,
        amount_out: effective_payout_amount,
        fee_amount,
        fee_value_usd_micros: 0,
        ts: Clock::get()?.unix_timestamp,
    });

    Ok(LivePoolExecutionOutcome {
        source_mint,
        source_amount: amount_in,
        payout_mint: effective_payout_mint,
        payout_amount: effective_payout_amount,
        destination_key: effective_destination_key,
    })
}

#[inline(never)]
fn settle_ticket_live_output<'info>(
    config: &Account<'info, Config>,
    user: &Signer<'info>,
    token_program: &Program<'info, Token>,
    requested_output_kind: SovereignFinalOutputKind,
    route: SovereignEscrowLiveSwapAccounts<'info>,
) -> Result<()> {
    require!(
        route.source_record.owner == &crate::ID,
        SterlingError::InvalidAccount
    );

    let source_escrow = load_token_account_snapshot(&route.source_escrow_ata)?;
    let ticket_key = route.source_record.key();

    let mut ticket_data = route.source_record.try_borrow_mut_data()?;
    let mut ticket_bytes: &[u8] = &ticket_data;
    let mut ticket = PayoutTicket::try_deserialize(&mut ticket_bytes)?;

    let (expected_escrow_authority, escrow_bump) = expected_ticket_escrow_authority(ticket_key);
    let expected_escrow_ata =
        expected_live_escrow_ata(expected_escrow_authority, ticket.escrow_mint);
    require!(
        ticket.escrow_bump == escrow_bump,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        ticket.escrow_ata == expected_escrow_ata,
        SterlingError::InvalidSovereignEscrowBinding
    );
    assert_source_escrow_binding(
        &source_escrow,
        route.source_escrow_ata.key(),
        ticket.escrow_ata,
        ticket.escrow_mint,
        ticket.escrow_amount_locked,
        expected_escrow_authority,
        ticket.funding_state,
    )?;
    begin_ticket_live_execution(&mut ticket, user.key(), config.keeper_authority)?;
    let escrow_bump_seed = [escrow_bump];
    let escrow_signer_seeds: &[&[&[u8]]] = &[&[
        b"ticket_live_escrow",
        ticket_key.as_ref(),
        &escrow_bump_seed,
    ]];

    let outcome = settle_live_pool_exchange_from_escrow(
        config,
        token_program,
        4,
        ticket.user,
        ticket.mint_in,
        expected_escrow_authority,
        escrow_signer_seeds,
        requested_output_kind,
        route.clone(),
    )?;

    finalize_ticket_live_execution(&mut ticket, outcome, Clock::get()?.unix_timestamp)?;

    let mut out: &mut [u8] = &mut ticket_data;
    ticket.try_serialize(&mut out)?;

    emit_sovereign_output_event(
        4,
        outcome.source_mint,
        outcome.source_amount,
        outcome.payout_mint,
        outcome.payout_amount,
        outcome.destination_key,
        ticket.user,
    )
}

#[inline(never)]
fn settle_claim_live_output<'info>(
    config: &Account<'info, Config>,
    user: &Signer<'info>,
    token_program: &Program<'info, Token>,
    requested_output_kind: SovereignFinalOutputKind,
    route: SovereignEscrowLiveSwapAccounts<'info>,
) -> Result<()> {
    require!(
        route.source_record.owner == &crate::ID,
        SterlingError::InvalidAccount
    );

    let source_escrow = load_token_account_snapshot(&route.source_escrow_ata)?;
    let claim_key = route.source_record.key();

    let mut claim_data = route.source_record.try_borrow_mut_data()?;
    let mut claim_bytes: &[u8] = &claim_data;
    let mut claim = SettlementClaim::try_deserialize(&mut claim_bytes)?;

    let (expected_escrow_authority, escrow_bump) = expected_claim_escrow_authority(claim_key);
    let expected_escrow_ata =
        expected_live_escrow_ata(expected_escrow_authority, claim.escrow_mint);
    require!(
        claim.escrow_bump == escrow_bump,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        claim.escrow_ata == expected_escrow_ata,
        SterlingError::InvalidSovereignEscrowBinding
    );
    assert_source_escrow_binding(
        &source_escrow,
        route.source_escrow_ata.key(),
        claim.escrow_ata,
        claim.escrow_mint,
        claim.escrow_amount_locked,
        expected_escrow_authority,
        claim.funding_state,
    )?;
    begin_claim_live_execution_by_executor(&mut claim, user.key(), config.keeper_authority)?;
    let escrow_bump_seed = [escrow_bump];
    let escrow_signer_seeds: &[&[&[u8]]] =
        &[&[b"claim_live_escrow", claim_key.as_ref(), &escrow_bump_seed]];

    let outcome = settle_live_pool_exchange_from_escrow(
        config,
        token_program,
        5,
        claim.user,
        claim.mint_in,
        expected_escrow_authority,
        escrow_signer_seeds,
        requested_output_kind,
        route.clone(),
    )?;

    let now = Clock::get()?.unix_timestamp;
    finalize_claim_live_execution(&mut claim, outcome, now)?;

    let mut out: &mut [u8] = &mut claim_data;
    claim.try_serialize(&mut out)?;

    emit!(ClaimPaidEvent {
        pool: claim.pool,
        payout_mint: claim.payout_mint,
        user: claim.user,
        usd_micros: claim.usd_micros,
        paid_atoms: claim.paid_atoms,
        proof_sig: claim.proof_sig,
        destination_ata: claim.destination_ata,
        nonce: claim.nonce,
        ts: now,
    });

    emit_sovereign_output_event(
        5,
        outcome.source_mint,
        outcome.source_amount,
        outcome.payout_mint,
        outcome.payout_amount,
        outcome.destination_key,
        claim.user,
    )
}

fn protocol_debt_ledger_header_len() -> usize {
    8 + 32 + 8 + 8 + 8 + 1
}

fn protocol_debt_lot_offset(index: usize) -> usize {
    protocol_debt_ledger_header_len() + (index * ProtocolDebtLot::LEN)
}

fn legacy_protocol_debt_lot_offset(index: usize) -> usize {
    protocol_debt_ledger_header_len() + (index * (8 + 32 + 8 + 8 + 8 + 1 + 1))
}

fn protocol_debt_ledger_pool_from_data(ledger_data: &[u8]) -> Result<Pubkey> {
    require!(
        ledger_data.len() >= protocol_debt_ledger_header_len(),
        SterlingError::InvalidAccount
    );
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&ledger_data[8..40]);
    Ok(Pubkey::new_from_array(bytes))
}

fn protocol_debt_ledger_overflow_from_data(ledger_data: &[u8]) -> Result<u64> {
    require!(
        ledger_data.len() >= protocol_debt_ledger_header_len(),
        SterlingError::InvalidAccount
    );
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&ledger_data[48..56]);
    Ok(u64::from_le_bytes(bytes))
}

fn store_protocol_debt_ledger_overflow(ledger_data: &mut [u8], value: u64) -> Result<()> {
    require!(
        ledger_data.len() >= protocol_debt_ledger_header_len(),
        SterlingError::InvalidAccount
    );
    ledger_data[48..56].copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn debt_legacy_pk(d: &[u8], o: usize) -> Result<Pubkey> {
    let end = o.saturating_add(32);
    require!(end <= d.len(), SterlingError::InvalidAccount);
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&d[o..end]);
    Ok(Pubkey::new_from_array(bytes))
}

fn debt_legacy_u64(d: &[u8], o: usize) -> Result<u64> {
    let end = o.saturating_add(8);
    require!(end <= d.len(), SterlingError::InvalidAccount);
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&d[o..end]);
    Ok(u64::from_le_bytes(bytes))
}

fn debt_legacy_i64(d: &[u8], o: usize) -> Result<i64> {
    let end = o.saturating_add(8);
    require!(end <= d.len(), SterlingError::InvalidAccount);
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&d[o..end]);
    Ok(i64::from_le_bytes(bytes))
}

fn protocol_debt_lot_from_legacy_slice(
    legacy_lot: &[u8],
    pool_key: Pubkey,
) -> Result<ProtocolDebtLot> {
    require!(
        legacy_lot.len() >= 8 + 32 + 8 + 8 + 8 + 1 + 1,
        SterlingError::InvalidAccount
    );
    let nonce = debt_legacy_u64(legacy_lot, 0)?;
    let mint_in = debt_legacy_pk(legacy_lot, 8)?;
    let amount_in = debt_legacy_u64(legacy_lot, 40)?;
    let status = legacy_lot[65];
    let funding_state = if status == PROTOCOL_DEBT_LOT_SETTLED {
        SOVEREIGN_ESCROW_STATE_SETTLED
    } else {
        SOVEREIGN_ESCROW_STATE_REQUESTED
    };
    let (escrow_authority, escrow_bump) =
        expected_protocol_debt_lot_escrow_authority(pool_key, nonce);
    Ok(ProtocolDebtLot {
        nonce,
        mint_in,
        amount_in,
        usd_micros: debt_legacy_u64(legacy_lot, 48)?,
        created_ts: debt_legacy_i64(legacy_lot, 56)?,
        escrow_mint: mint_in,
        escrow_ata: expected_live_escrow_ata(escrow_authority, mint_in),
        escrow_amount_locked: amount_in,
        escrow_bump,
        funding_state,
        route_hint: legacy_lot[64],
        status,
    })
}

fn load_protocol_debt_lot_at_index(
    ledger_data: &[u8],
    index: usize,
    pool_key: Pubkey,
) -> Result<ProtocolDebtLot> {
    require!(
        index < PROTOCOL_DEBT_LEDGER_SLOTS,
        SterlingError::InvalidAccount
    );
    if ledger_data.len() < ProtocolDebtLedger::LEN {
        let start = legacy_protocol_debt_lot_offset(index);
        let end = start + (8 + 32 + 8 + 8 + 8 + 1 + 1);
        require!(end <= ledger_data.len(), SterlingError::InvalidAccount);
        return protocol_debt_lot_from_legacy_slice(&ledger_data[start..end], pool_key);
    }

    let start = protocol_debt_lot_offset(index);
    let end = start + ProtocolDebtLot::LEN;
    require!(end <= ledger_data.len(), SterlingError::InvalidAccount);
    ProtocolDebtLot::try_from_slice(&ledger_data[start..end])
        .map_err(|_| error!(SterlingError::InvalidAccount))
}

fn load_protocol_debt_lot_from_legacy_ledger_data(
    ledger_data: &[u8],
    debt_lot_nonce: u64,
    pool_key: Pubkey,
) -> Result<(usize, ProtocolDebtLot, Pubkey)> {
    for index in 0..PROTOCOL_DEBT_LEDGER_SLOTS {
        let start = legacy_protocol_debt_lot_offset(index);
        let end = start + (8 + 32 + 8 + 8 + 8 + 1 + 1);
        require!(end <= ledger_data.len(), SterlingError::InvalidAccount);
        let legacy_lot = &ledger_data[start..end];
        let lot = protocol_debt_lot_from_legacy_slice(legacy_lot, pool_key)?;
        if lot.status != PROTOCOL_DEBT_LOT_EMPTY && lot.nonce == debt_lot_nonce {
            return Ok((index, lot, pool_key));
        }
    }

    err!(SterlingError::InvalidAccount)
}

fn load_protocol_debt_lot_from_ledger_data(
    ledger_data: &[u8],
    debt_lot_nonce: u64,
) -> Result<(usize, ProtocolDebtLot, Pubkey)> {
    require!(
        ledger_data.len() >= protocol_debt_ledger_header_len(),
        SterlingError::InvalidAccount
    );
    let pool_key = Pubkey::new_from_array(
        ledger_data[8..40]
            .try_into()
            .map_err(|_| error!(SterlingError::InvalidAccount))?,
    );

    if ledger_data.len() < ProtocolDebtLedger::LEN {
        return load_protocol_debt_lot_from_legacy_ledger_data(ledger_data, debt_lot_nonce, pool_key);
    }

    for index in 0..PROTOCOL_DEBT_LEDGER_SLOTS {
        let start = protocol_debt_lot_offset(index);
        let end = start + ProtocolDebtLot::LEN;
        require!(end <= ledger_data.len(), SterlingError::InvalidAccount);
        let lot = ProtocolDebtLot::try_from_slice(&ledger_data[start..end])
            .map_err(|_| error!(SterlingError::InvalidAccount))?;
        if lot.nonce == debt_lot_nonce {
            return Ok((index, lot, pool_key));
        }
    }

    err!(SterlingError::InvalidAccount)
}

fn store_protocol_debt_lot_into_ledger_data(
    ledger_data: &mut [u8],
    lot_index: usize,
    lot: &ProtocolDebtLot,
) -> Result<()> {
    if ledger_data.len() < ProtocolDebtLedger::LEN {
        let start = legacy_protocol_debt_lot_offset(lot_index);
        let end = start + (8 + 32 + 8 + 8 + 8 + 1 + 1);
        require!(end <= ledger_data.len(), SterlingError::InvalidAccount);
        let legacy_lot = &mut ledger_data[start..end];
        legacy_lot.fill(0);
        legacy_lot[0..8].copy_from_slice(&lot.nonce.to_le_bytes());
        legacy_lot[8..40].copy_from_slice(lot.mint_in.as_ref());
        legacy_lot[40..48].copy_from_slice(&lot.amount_in.to_le_bytes());
        legacy_lot[48..56].copy_from_slice(&lot.usd_micros.to_le_bytes());
        legacy_lot[56..64].copy_from_slice(&lot.created_ts.to_le_bytes());
        legacy_lot[64] = lot.route_hint;
        legacy_lot[65] = lot.status;
        return Ok(());
    }

    let start = protocol_debt_lot_offset(lot_index);
    let end = start + ProtocolDebtLot::LEN;
    require!(end <= ledger_data.len(), SterlingError::InvalidAccount);
    let mut cursor = &mut ledger_data[start..end];
    lot.serialize(&mut cursor)
        .map_err(|_| error!(SterlingError::InvalidAccount))
}

fn store_protocol_debt_ledger_last_ts(ledger_data: &mut [u8], value: i64) -> Result<()> {
    let start = 8 + 32 + 8 + 8;
    let end = start + 8;
    ledger_data[start..end].copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn initialize_protocol_debt_ledger_data(
    ledger_data: &mut [u8],
    pool_key: Pubkey,
    bump: u8,
) -> Result<()> {
    require!(
        ledger_data.len() >= ProtocolDebtLedger::LEN,
        SterlingError::InvalidAccount
    );
    ledger_data.fill(0);
    ledger_data[..8].copy_from_slice(
        &<ProtocolDebtLedger as anchor_lang::Discriminator>::discriminator(),
    );
    ledger_data[8..40].copy_from_slice(pool_key.as_ref());
    ledger_data[40..48].copy_from_slice(&1u64.to_le_bytes());
    ledger_data[48..56].copy_from_slice(&0u64.to_le_bytes());
    ledger_data[56..64].copy_from_slice(&0i64.to_le_bytes());
    ledger_data[64] = bump;
    Ok(())
}

fn settle_protocol_debt_lots_in_ledger_data(
    pool: &mut Pool,
    ledger_data: &mut [u8],
    paid_usd_micros: u64,
    now: i64,
) -> Result<u64> {
    require!(
        pool.protocol_fee_debt_usd_micros >= paid_usd_micros,
        SterlingError::InvalidAmount
    );

    let pool_key = protocol_debt_ledger_pool_from_data(ledger_data)?;
    let mut remaining = paid_usd_micros;

    for index in 0..PROTOCOL_DEBT_LEDGER_SLOTS {
        if remaining == 0 {
            break;
        }
        let mut lot = load_protocol_debt_lot_at_index(ledger_data, index, pool_key)?;
        if !matches!(
            lot.status,
            PROTOCOL_DEBT_LOT_OPEN | PROTOCOL_DEBT_LOT_TICKETED
        ) || lot.usd_micros == 0
        {
            continue;
        }

        if remaining >= lot.usd_micros {
            remaining -= lot.usd_micros;
            mark_protocol_debt_lot_settled(pool, &mut lot, now)?;
        } else {
            lot.usd_micros -= remaining;
            pool.protocol_fee_debt_usd_micros -= remaining;
            pool.protocol_fee_debt_last_ts = now;
            remaining = 0;
        }

        store_protocol_debt_lot_into_ledger_data(ledger_data, index, &lot)?;
    }

    if remaining > 0 {
        let mut overflow_usd_micros = protocol_debt_ledger_overflow_from_data(ledger_data)?;
        require!(
            overflow_usd_micros >= remaining,
            SterlingError::InvalidAmount
        );
        let overflow_count_before = protocol_fee_debt_shard_count(overflow_usd_micros);
        overflow_usd_micros -= remaining;
        let overflow_count_after = protocol_fee_debt_shard_count(overflow_usd_micros);
        store_protocol_debt_ledger_overflow(ledger_data, overflow_usd_micros)?;
        pool.protocol_fee_debt_count = pool
            .protocol_fee_debt_count
            .saturating_sub(overflow_count_before.saturating_sub(overflow_count_after));
        pool.protocol_fee_debt_usd_micros -= remaining;
        pool.protocol_fee_debt_last_ts = now;
        remaining = 0;
    }

    require!(remaining == 0, SterlingError::InvalidAmount);
    store_protocol_debt_ledger_last_ts(ledger_data, now)?;
    Ok(paid_usd_micros)
}

#[inline(never)]
fn load_protocol_debt_live_snapshot(
    pool_info: &AccountInfo,
    ledger_info: &AccountInfo,
    debt_lot_nonce: u64,
    source_escrow: &TokenAccount,
    source_escrow_key: Pubkey,
) -> Result<ProtocolDebtLot> {
    let pool_data = pool_info.try_borrow_data()?;
    let mut pool_bytes: &[u8] = &pool_data;
    let pool = Pool::try_deserialize(&mut pool_bytes)?;

    let ledger_data = ledger_info.try_borrow_data()?;
    let (_, lot, ledger_pool_key) =
        load_protocol_debt_lot_from_ledger_data(&ledger_data, debt_lot_nonce)?;
    let (expected_escrow_authority, expected_escrow_bump) =
        expected_protocol_debt_lot_escrow_authority(pool_info.key(), lot.nonce);
    let expected_escrow_ata = expected_live_escrow_ata(expected_escrow_authority, lot.escrow_mint);

    require!(
        ledger_pool_key == pool_info.key(),
        SterlingError::InvalidAccount
    );
    require!(
        pool.protocol_fee_debt_usd_micros > 0,
        SterlingError::InvalidAmount
    );
    require!(
        lot.status == PROTOCOL_DEBT_LOT_OPEN || lot.status == PROTOCOL_DEBT_LOT_TICKETED,
        SterlingError::InvalidState
    );
    require!(lot.usd_micros > 0, SterlingError::InvalidAmount);
    require!(
        lot.escrow_bump == expected_escrow_bump,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        lot.escrow_ata == expected_escrow_ata,
        SterlingError::InvalidSovereignEscrowBinding
    );
    assert_source_escrow_binding(
        source_escrow,
        source_escrow_key,
        lot.escrow_ata,
        lot.escrow_mint,
        lot.escrow_amount_locked,
        expected_escrow_authority,
        lot.funding_state,
    )?;
    Ok(lot)
}

#[inline(never)]
fn finalize_protocol_debt_live_settlement(
    pool_info: &AccountInfo,
    ledger_info: &AccountInfo,
    debt_lot_nonce: u64,
    now: i64,
) -> Result<(u64, u64)> {
    let mut pool_data = pool_info.try_borrow_mut_data()?;
    let mut pool_bytes: &[u8] = &pool_data;
    let mut pool = Pool::try_deserialize(&mut pool_bytes)?;

    let mut ledger_data = ledger_info.try_borrow_mut_data()?;
    let (lot_index, mut lot, ledger_pool_key) =
        load_protocol_debt_lot_from_ledger_data(&ledger_data, debt_lot_nonce)?;
    require!(
        ledger_pool_key == pool_info.key(),
        SterlingError::InvalidAccount
    );
    let paid_usd_micros = lot.usd_micros;
    begin_protocol_debt_lot_live_execution(&mut lot)?;
    finalize_protocol_debt_lot_live_execution(&mut pool, &mut lot, now)?;

    let remaining_usd_micros = pool.protocol_fee_debt_usd_micros;
    store_protocol_debt_lot_into_ledger_data(&mut ledger_data, lot_index, &lot)?;
    store_protocol_debt_ledger_last_ts(&mut ledger_data, now)?;

    let mut pool_out: &mut [u8] = &mut pool_data;
    pool.try_serialize(&mut pool_out)?;

    Ok((paid_usd_micros, remaining_usd_micros))
}

#[inline(never)]
fn settle_protocol_debt_live_output<'info>(
    config: &Account<'info, Config>,
    user: &Signer<'info>,
    token_program: &Program<'info, Token>,
    requested_output_kind: SovereignFinalOutputKind,
    debt_lot_nonce: u64,
    route: SovereignEscrowLiveSwapAccounts<'info>,
) -> Result<()> {
    require!(
        route.source_record.owner == &crate::ID,
        SterlingError::InvalidAccount
    );
    let ledger_info = route
        .ledger_info
        .clone()
        .ok_or_else(|| error!(SterlingError::LiveSovereignExchangeRequired))?;
    require!(
        ledger_info.owner == &crate::ID,
        SterlingError::InvalidAccount
    );
    require!(
        user.key() == config.main_wallet,
        SterlingError::InvalidAccount
    );

    let source_escrow = load_token_account_snapshot(&route.source_escrow_ata)?;
    let debt_snapshot = load_protocol_debt_live_snapshot(
        &route.source_record,
        &ledger_info,
        debt_lot_nonce,
        &source_escrow,
        route.source_escrow_ata.key(),
    )?;
    let pool_key = route.source_record.key();
    let (expected_escrow_authority, escrow_bump) =
        expected_protocol_debt_lot_escrow_authority(pool_key, debt_snapshot.nonce);
    require!(
        debt_snapshot.escrow_bump == escrow_bump,
        SterlingError::InvalidSovereignEscrowBinding
    );
    let debt_nonce_bytes = debt_snapshot.nonce.to_le_bytes();
    let escrow_bump_seed = [escrow_bump];
    let escrow_signer_seeds: &[&[&[u8]]] = &[&[
        b"debt_live_escrow",
        pool_key.as_ref(),
        &debt_nonce_bytes,
        &escrow_bump_seed,
    ]];

    let outcome = settle_live_pool_exchange_from_escrow(
        config,
        token_program,
        6,
        user.key(),
        debt_snapshot.mint_in,
        expected_escrow_authority,
        escrow_signer_seeds,
        requested_output_kind,
        route.clone(),
    )?;

    let now = Clock::get()?.unix_timestamp;
    let (paid_usd_micros, remaining_usd_micros) = finalize_protocol_debt_live_settlement(
        &route.source_record,
        &ledger_info,
        debt_lot_nonce,
        now,
    )?;

    emit!(ProtocolFeeDebtSettledEvent {
        pool: route.source_record.key(),
        paid_usd_micros,
        remaining_usd_micros,
        proof_sig: live_execution_proof_sig(),
        ts: now,
    });

    emit_sovereign_output_event(
        6,
        outcome.source_mint,
        outcome.source_amount,
        outcome.payout_mint,
        outcome.payout_amount,
        outcome.destination_key,
        user.key(),
    )
}

fn resolve_direct_stable_target<'info>(
    token_mint: Pubkey,
    primary_target: Option<StableFinalOutputTarget<'info>>,
    secondary_target: Option<StableFinalOutputTarget<'info>>,
) -> Result<StableFinalOutputTarget<'info>> {
    for target in [primary_target, secondary_target].into_iter().flatten() {
        if target.payout_mint == token_mint {
            return Ok(target);
        }
    }

    err!(SterlingError::NoExecutableSovereignRail)
}

enum SovereignBufferedOutputTarget<'info> {
    Stable(StableFinalOutputTarget<'info>),
    NativeSol(NativeSolOutputTarget<'info>),
}

fn resolve_buffered_stable_target<'info>(
    payout_atoms: u64,
    primary_target: Option<StableFinalOutputTarget<'info>>,
    secondary_target: Option<StableFinalOutputTarget<'info>>,
) -> Result<StableFinalOutputTarget<'info>> {
    for target in [primary_target, secondary_target].into_iter().flatten() {
        if target.vault_amount >= payout_atoms {
            return Ok(target);
        }
    }

    err!(SterlingError::NoExecutableSovereignRail)
}

fn resolve_buffered_output_target<'info>(
    native_sol_config: NativeSolRailConfig,
    payout_atoms: u64,
    primary_target: Option<StableFinalOutputTarget<'info>>,
    secondary_target: Option<StableFinalOutputTarget<'info>>,
    native_target: Option<NativeSolOutputTarget<'info>>,
) -> Result<SovereignBufferedOutputTarget<'info>> {
    if let Ok(target) =
        resolve_buffered_stable_target(payout_atoms, primary_target, secondary_target)
    {
        return Ok(SovereignBufferedOutputTarget::Stable(target));
    }

    if let Some(target) = native_target {
        native_sol_payout_lamports(native_sol_config, &target, payout_atoms)?;
        return Ok(SovereignBufferedOutputTarget::NativeSol(target));
    }

    err!(SterlingError::NoExecutableSovereignRail)
}

#[inline(never)]
fn settle_live_pool_exchange<'info>(
    config: &Account<'info, Config>,
    user: &Signer<'info>,
    token_program: &Program<'info, Token>,
    source_kind: u8,
    amount_in: u64,
    requested_output_kind: SovereignFinalOutputKind,
    require_non_lp: bool,
    route: SovereignLiveSwapAccounts<'info>,
) -> Result<()> {
    require_live_sovereign_source_kind(source_kind)?;
    require!(
        route.pool_info.owner == &crate::ID,
        SterlingError::InvalidLiveSovereignPoolRoute
    );

    let token_mint = load_mint_snapshot(&route.token_mint)?;
    let user_token_ata = load_token_account_snapshot(&route.user_token_ata)?;
    let mut pool = load_pool_snapshot(&route.pool_info)?;
    let input_vault = load_token_account_snapshot(&route.input_vault)?;
    let output_vault = load_token_account_snapshot(&route.output_vault)?;
    let fee_vault_in = load_token_account_snapshot(&route.fee_vault_in)?;
    let output_destination = load_token_account_snapshot(&route.output_destination)?;

    require!(pool.active, SterlingError::InactivePool);
    require!(
        user_token_ata.owner == user.key(),
        SterlingError::InvalidAccount
    );
    require!(
        user_token_ata.mint == route.token_mint.key(),
        SterlingError::InvalidAccount
    );

    if let Some(asset_registry_info) = route.asset_registry.clone() {
        let asset_registry = load_asset_registry_snapshot(&asset_registry_info)?;
        require!(
            asset_registry.mint == route.token_mint.key(),
            SterlingError::InvalidAccount
        );
        require!(asset_registry.active, SterlingError::BadRegistry);
        if require_non_lp {
            require!(!asset_registry.is_lp, SterlingError::InvalidAccount);
        }
        require!(
            asset_registry.decimals == token_mint.decimals,
            SterlingError::InvalidAccount
        );
    }

    let input_is_base = if pool.base_mint == route.token_mint.key() {
        require!(
            route.input_vault.key() == pool.base_vault,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            route.output_vault.key() == pool.quote_vault,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            route.fee_vault_in.key() == pool.fee_vault_base,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            input_vault.mint == pool.base_mint,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            output_vault.mint == pool.quote_mint,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            fee_vault_in.mint == pool.base_mint,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        true
    } else if pool.quote_mint == route.token_mint.key() {
        require!(
            route.input_vault.key() == pool.quote_vault,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            route.output_vault.key() == pool.base_vault,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            route.fee_vault_in.key() == pool.fee_vault_quote,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            input_vault.mint == pool.quote_mint,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            output_vault.mint == pool.base_mint,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        require!(
            fee_vault_in.mint == pool.quote_mint,
            SterlingError::InvalidLiveSovereignPoolRoute
        );
        false
    } else {
        return err!(SterlingError::InvalidLiveSovereignPoolRoute);
    };

    let output_mint = if input_is_base {
        pool.quote_mint
    } else {
        pool.base_mint
    };
    let settlement_kind = classify_live_sovereign_settlement_kind(
        requested_output_kind,
        output_mint,
        config.usdc_mint,
        config.usdt_mint,
        config.extra_payout_mint_0,
        config.extra_payout_mint_1,
        config.extra_payout_mint_2,
        config.extra_payout_mint_3,
    )?;

    match settlement_kind {
        LiveSovereignSettlementKind::Stable => {
            require!(
                output_destination.owner == user.key(),
                SterlingError::InvalidAccount
            );
            require!(
                output_destination.mint == output_mint,
                SterlingError::InvalidAccount
            );
        }
        LiveSovereignSettlementKind::NativeSol => {
            require!(
                output_destination.mint == output_mint,
                SterlingError::InvalidAccount
            );
            require!(
                output_destination.owner == config.key(),
                SterlingError::InvalidLiveSovereignPoolRoute
            );
            let native_destination = route
                .native_destination
                .clone()
                .ok_or_else(|| error!(SterlingError::InvalidNativeSolDestination))?;
            require!(
                native_destination.key() == user.key(),
                SterlingError::InvalidNativeSolDestination
            );
            require!(
                native_destination.is_writable,
                SterlingError::InvalidNativeSolDestination
            );
            require!(
                !native_destination.executable,
                SterlingError::InvalidNativeSolDestination
            );
            require!(
                native_destination.key() != config.key(),
                SterlingError::InvalidNativeSolDestination
            );
            assert_wsol_temporary_output_account(
                &output_destination,
                route.output_destination.key(),
                config.key(),
                &native_destination,
            )?;
        }
    }

    let fee_bps = pool.fee_bps as u128;
    let fee_amount = (amount_in as u128)
        .saturating_mul(fee_bps)
        .checked_div(10_000u128)
        .ok_or(SterlingError::MathOverflow)? as u64;
    let amount_in_less_fee = (amount_in as u128)
        .saturating_mul(10_000u128.saturating_sub(fee_bps))
        .checked_div(10_000u128)
        .ok_or(SterlingError::MathOverflow)? as u64;
    let x = input_vault.amount as u128;
    let y = output_vault.amount as u128;
    let out = (amount_in_less_fee as u128)
        .saturating_mul(y)
        .checked_div(x.saturating_add(amount_in_less_fee as u128))
        .ok_or(SterlingError::MathOverflow)? as u64;

    require!(out > 0, SterlingError::InvalidAmount);
    require!(
        output_vault.amount >= out,
        SterlingError::InsufficientLiquidity
    );

    transfer_tokens(
        token_program,
        route.user_token_ata.clone(),
        route.input_vault.clone(),
        user.to_account_info(),
        amount_in,
    )?;

    let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[config.bump]]];
    transfer_tokens_signed(
        token_program,
        route.input_vault.clone(),
        route.fee_vault_in.clone(),
        config.to_account_info(),
        signer_seeds,
        fee_amount,
    )?;

    transfer_tokens_signed(
        token_program,
        route.output_vault.clone(),
        route.output_destination.clone(),
        config.to_account_info(),
        signer_seeds,
        out,
    )?;

    let (effective_payout_mint, effective_destination_key) =
        if settlement_kind == LiveSovereignSettlementKind::NativeSol {
            let native_destination = route
                .native_destination
                .clone()
                .ok_or_else(|| error!(SterlingError::InvalidNativeSolDestination))?;
            require!(
                native_destination.is_writable,
                SterlingError::InvalidNativeSolDestination
            );
            token::close_account(
                CpiContext::new(
                    token_program.to_account_info(),
                    CloseAccount {
                        account: route.output_destination.clone(),
                        destination: native_destination.clone(),
                        authority: config.to_account_info(),
                    },
                )
                .with_signer(signer_seeds),
            )?;
            (native_sol_marker_pubkey(), native_destination.key())
        } else {
            (output_mint, route.output_destination.key())
        };

    pool.swap_count = pool.swap_count.saturating_add(1);
    pool.last_swap_ts = Clock::get()?.unix_timestamp;
    if input_is_base {
        pool.total_base_volume = pool.total_base_volume.saturating_add(amount_in);
        pool.total_quote_volume = pool.total_quote_volume.saturating_add(out);
    } else {
        pool.total_quote_volume = pool.total_quote_volume.saturating_add(amount_in);
        pool.total_base_volume = pool.total_base_volume.saturating_add(out);
    }

    let mut pool_data = route.pool_info.try_borrow_mut_data()?;
    let mut pool_out: &mut [u8] = &mut pool_data;
    pool.try_serialize(&mut pool_out)?;

    emit!(SwapExecuted {
        pool: route.pool_info.key(),
        user: user.key(),
        side: if input_is_base {
            FeeSide::Base
        } else {
            FeeSide::Quote
        },
        mint_in: route.token_mint.key(),
        amount_in,
        mint_out: effective_payout_mint,
        amount_out: out,
        fee_amount,
        fee_value_usd_micros: 0,
        ts: Clock::get()?.unix_timestamp,
    });

    emit_sovereign_output_event(
        source_kind,
        route.token_mint.key(),
        amount_in,
        effective_payout_mint,
        out,
        effective_destination_key,
        user.key(),
    )
}

#[inline(never)]
fn settle_direct_stable_output<'info>(
    config: &Account<'info, Config>,
    user: &Signer<'info>,
    token_mint_info: AccountInfo<'info>,
    user_token_ata_info: AccountInfo<'info>,
    token_program: &Program<'info, Token>,
    amount_in: u64,
    source_kind: u8,
    primary_target: Option<StableFinalOutputTarget<'info>>,
    secondary_target: Option<StableFinalOutputTarget<'info>>,
    native_target: Option<NativeSolOutputTarget<'info>>,
) -> Result<()> {
    let native_sol_config = NativeSolRailConfig {
        native_sol_usd_micros_per_sol: config.native_sol_usd_micros_per_sol,
    };
    let user_token_ata = load_token_account_snapshot(&user_token_ata_info)?;
    require!(
        user_token_ata.owner == user.key(),
        SterlingError::InvalidAccount
    );
    require!(
        user_token_ata.mint == token_mint_info.key(),
        SterlingError::InvalidAccount
    );

    if let Ok(target) = resolve_direct_stable_target(
        token_mint_info.key(),
        primary_target.clone(),
        secondary_target.clone(),
    ) {
        require!(
            user_token_ata.mint == target.payout_mint,
            SterlingError::InvalidAccount
        );

        transfer_tokens(
            token_program,
            user_token_ata_info,
            target.destination_info.clone(),
            user.to_account_info(),
            amount_in,
        )?;

        return emit_sovereign_output_event(
            source_kind,
            token_mint_info.key(),
            amount_in,
            target.payout_mint,
            amount_in,
            target.destination_key,
            user.key(),
        );
    }

    let native_target =
        native_target.ok_or_else(|| error!(SterlingError::NoExecutableSovereignRail))?;
    let stable_deposit_destination = if token_mint_info.key() == config.usdc_mint {
        primary_target
            .clone()
            .filter(|target| target.payout_mint == config.usdc_mint)
            .map(|target| (target.destination_info, target.destination_key))
            .or_else(|| {
                secondary_target
                    .clone()
                    .filter(|target| target.payout_mint == config.usdc_mint)
                    .map(|target| (target.destination_info, target.destination_key))
            })
            .ok_or_else(|| error!(SterlingError::StableSettlementAccountsMissing))?
    } else if token_mint_info.key() == config.usdt_mint {
        primary_target
            .clone()
            .filter(|target| target.payout_mint == config.usdt_mint)
            .map(|target| (target.destination_info, target.destination_key))
            .or_else(|| {
                secondary_target
                    .clone()
                    .filter(|target| target.payout_mint == config.usdt_mint)
                    .map(|target| (target.destination_info, target.destination_key))
            })
            .ok_or_else(|| error!(SterlingError::StableSettlementAccountsMissing))?
    } else {
        return err!(SterlingError::UnsupportedSovereignFinalOutput);
    };

    if token_mint_info.key() == config.usdc_mint {
        require!(
            stable_deposit_destination.1 == config.treasury_usdc_ata,
            SterlingError::InvalidAccount
        );
    } else {
        require!(
            stable_deposit_destination.1 == config.treasury_usdt_ata,
            SterlingError::InvalidAccount
        );
    }

    transfer_tokens(
        token_program,
        user_token_ata_info,
        stable_deposit_destination.0,
        user.to_account_info(),
        amount_in,
    )?;

    let payout_lamports = native_sol_payout_lamports(native_sol_config, &native_target, amount_in)?;
    transfer_native_sol_from_config(config.to_account_info(), &native_target, payout_lamports)?;

    emit_sovereign_output_event(
        source_kind,
        token_mint_info.key(),
        amount_in,
        native_sol_marker_pubkey(),
        payout_lamports,
        native_target.destination_key,
        user.key(),
    )
}

#[inline(never)]
fn settle_valued_asset_output<'info>(
    config: &Account<'info, Config>,
    user: &Signer<'info>,
    token_mint_info: AccountInfo<'info>,
    user_token_ata_info: AccountInfo<'info>,
    asset_registry_info: AccountInfo<'info>,
    token_program: &Program<'info, Token>,
    amount_in: u64,
    source_kind: u8,
    config_bump: u8,
    require_non_lp: bool,
    primary_target: Option<StableFinalOutputTarget<'info>>,
    secondary_target: Option<StableFinalOutputTarget<'info>>,
    native_target: Option<NativeSolOutputTarget<'info>>,
) -> Result<()> {
    let native_sol_config = NativeSolRailConfig {
        native_sol_usd_micros_per_sol: config.native_sol_usd_micros_per_sol,
    };
    let asset_registry_info = asset_registry_info.clone();
    let asset_registry = load_asset_registry_snapshot(&asset_registry_info)?;
    let token_mint = load_mint_snapshot(&token_mint_info)?;
    let user_token_ata = load_token_account_snapshot(&user_token_ata_info)?;

    require!(
        asset_registry.mint == token_mint_info.key(),
        SterlingError::InvalidAccount
    );
    require!(asset_registry.active, SterlingError::BadRegistry);
    if require_non_lp {
        require!(!asset_registry.is_lp, SterlingError::InvalidAccount);
    }
    require!(
        user_token_ata.owner == user.key(),
        SterlingError::InvalidAccount
    );
    require!(
        user_token_ata.mint == token_mint_info.key(),
        SterlingError::InvalidAccount
    );

    let payout_atoms = compute_usdc_settlement_amount(
        amount_in,
        asset_registry.valuation_usd_micros,
        token_mint.decimals,
    )?;
    require!(payout_atoms > 0, SterlingError::ZeroStableSettlement);

    let target = resolve_buffered_output_target(
        native_sol_config,
        payout_atoms,
        primary_target,
        secondary_target,
        native_target,
    )?;

    burn_tokens(
        token_program,
        token_mint_info.clone(),
        user_token_ata_info,
        user.to_account_info(),
        amount_in,
    )?;
    match target {
        SovereignBufferedOutputTarget::Stable(target) => {
            transfer_from_stable_target_signed(
                &target,
                config.to_account_info(),
                token_program.to_account_info(),
                config_bump,
                payout_atoms,
            )?;

            emit_sovereign_output_event(
                source_kind,
                token_mint_info.key(),
                amount_in,
                target.payout_mint,
                payout_atoms,
                target.destination_key,
                user.key(),
            )
        }
        SovereignBufferedOutputTarget::NativeSol(target) => {
            let payout_lamports =
                native_sol_payout_lamports(native_sol_config, &target, payout_atoms)?;
            transfer_native_sol_from_config(config.to_account_info(), &target, payout_lamports)?;

            emit_sovereign_output_event(
                source_kind,
                token_mint_info.key(),
                amount_in,
                native_sol_marker_pubkey(),
                payout_lamports,
                target.destination_key,
                user.key(),
            )
        }
    }
}

#[inline(never)]
fn settle_redeem_to_stable_output<'info>(
    config: &Account<'info, Config>,
    user: &Signer<'info>,
    token_mint_info: AccountInfo<'info>,
    user_token_ata_info: AccountInfo<'info>,
    asset_registry_info: AccountInfo<'info>,
    token_program: &Program<'info, Token>,
    amount_in: u64,
    source_kind: u8,
    config_bump: u8,
    primary_target: Option<StableFinalOutputTarget<'info>>,
    secondary_target: Option<StableFinalOutputTarget<'info>>,
    native_target: Option<NativeSolOutputTarget<'info>>,
) -> Result<()> {
    settle_valued_asset_output(
        config,
        user,
        token_mint_info,
        user_token_ata_info,
        asset_registry_info,
        token_program,
        amount_in,
        source_kind,
        config_bump,
        true,
        primary_target,
        secondary_target,
        native_target,
    )
}

#[inline(never)]
fn settle_ticket_to_stable<'a, 'b>(
    config_info: AccountInfo<'a>,
    user_key: Pubkey,
    token_program_info: AccountInfo<'a>,
    ticket_info: AccountInfo<'b>,
    buffered_escrow: SovereignBufferedEscrowAccounts<'a>,
    config_bump: u8,
    primary_target: Option<StableFinalOutputTarget<'a>>,
    secondary_target: Option<StableFinalOutputTarget<'a>>,
    native_target: Option<NativeSolOutputTarget<'a>>,
) -> Result<()> {
    require!(
        ticket_info.owner == &crate::ID,
        SterlingError::InvalidAccount
    );

    let mut ticket_data = ticket_info.try_borrow_mut_data()?;
    let mut ticket_bytes: &[u8] = &ticket_data;
    let mut ticket = PayoutTicket::try_deserialize(&mut ticket_bytes)?;
    let ticket_key = ticket_info.key();
    let config = load_config_snapshot(&config_info)?;
    let (expected_escrow_authority, escrow_bump) = expected_ticket_escrow_authority(ticket_key);
    let expected_escrow_ata =
        expected_live_escrow_ata(expected_escrow_authority, ticket.escrow_mint);

    require!(
        ticket.user == user_key || user_key == config.keeper_authority,
        SterlingError::InvalidAccount
    );
    require!(ticket.status == 0, SterlingError::InvalidState);
    require!(ticket.settled_ts == 0, SterlingError::InvalidState);
    require!(
        ticket.escrow_bump == escrow_bump,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        ticket.escrow_ata == expected_escrow_ata,
        SterlingError::InvalidSovereignEscrowBinding
    );
    validate_buffered_route_escrow_accounts(
        &buffered_escrow,
        expected_escrow_authority,
        ticket.escrow_ata,
        ticket.escrow_mint,
        ticket.escrow_amount_locked,
        ticket.funding_state,
    )?;

    let payout_atoms = usd_micros_to_atoms(ticket.usd_micros, STABLE_CASH_DECIMALS)?;
    require!(payout_atoms > 0, SterlingError::ZeroStableSettlement);

    let escrow_bump_seed = [escrow_bump];
    let escrow_signer_seeds: &[&[&[u8]]] = &[&[
        b"ticket_live_escrow",
        ticket_key.as_ref(),
        &escrow_bump_seed,
    ]];

    burn_tokens_signed_account_info(
        token_program_info.clone(),
        buffered_escrow.mint_info.clone(),
        buffered_escrow.ata_info.clone(),
        buffered_escrow.authority_info.clone(),
        escrow_signer_seeds,
        ticket.escrow_amount_locked,
    )?;

    let now = Clock::get()?.unix_timestamp;
    ticket.status = 1;
    ticket.settled_ts = now;

    let native_sol_config = load_native_sol_rail_config(&config_info)?;
    let payout_amount = match resolve_buffered_output_target(
        native_sol_config,
        payout_atoms,
        primary_target,
        secondary_target,
        native_target,
    )? {
        SovereignBufferedOutputTarget::Stable(target) => {
            transfer_from_stable_target_signed(
                &target,
                config_info.clone(),
                token_program_info.clone(),
                config_bump,
                payout_atoms,
            )?;
            ticket.payout_mint = target.payout_mint;
            ticket.destination_ata = target.destination_key;
            payout_atoms
        }
        SovereignBufferedOutputTarget::NativeSol(target) => {
            let payout_lamports =
                native_sol_payout_lamports(native_sol_config, &target, payout_atoms)?;
            transfer_native_sol_from_config(config_info.clone(), &target, payout_lamports)?;
            ticket.payout_mint = native_sol_marker_pubkey();
            ticket.destination_ata = target.destination_key;
            payout_lamports
        }
    };
    ticket.funding_state = SOVEREIGN_ESCROW_STATE_SETTLED;

    let mut out: &mut [u8] = &mut ticket_data;
    ticket.try_serialize(&mut out)?;

    emit_sovereign_output_event(
        4,
        ticket.mint_in,
        ticket.amount_in,
        ticket.payout_mint,
        payout_amount,
        ticket.destination_ata,
        ticket.user,
    )
}

#[inline(never)]
fn settle_claim_to_stable<'a, 'b>(
    config_info: AccountInfo<'a>,
    user_key: Pubkey,
    token_program_info: AccountInfo<'a>,
    claim_info: AccountInfo<'b>,
    buffered_escrow: SovereignBufferedEscrowAccounts<'a>,
    config_bump: u8,
    primary_target: Option<StableFinalOutputTarget<'a>>,
    secondary_target: Option<StableFinalOutputTarget<'a>>,
    native_target: Option<NativeSolOutputTarget<'a>>,
) -> Result<()> {
    require!(
        claim_info.owner == &crate::ID,
        SterlingError::InvalidAccount
    );

    let mut claim_data = claim_info.try_borrow_mut_data()?;
    let mut claim_bytes: &[u8] = &claim_data;
    let mut claim = SettlementClaim::try_deserialize(&mut claim_bytes)?;
    let claim_key = claim_info.key();
    let config = load_config_snapshot(&config_info)?;
    let (expected_escrow_authority, escrow_bump) = expected_claim_escrow_authority(claim_key);
    let expected_escrow_ata =
        expected_live_escrow_ata(expected_escrow_authority, claim.escrow_mint);

    require!(
        claim.user == user_key || user_key == config.keeper_authority,
        SterlingError::InvalidAccount
    );
    require!(claim.status == 0, SterlingError::InvalidState);
    require!(claim.settled_ts == 0, SterlingError::InvalidState);
    require!(
        claim.escrow_bump == escrow_bump,
        SterlingError::InvalidSovereignEscrowBinding
    );
    require!(
        claim.escrow_ata == expected_escrow_ata,
        SterlingError::InvalidSovereignEscrowBinding
    );
    validate_buffered_route_escrow_accounts(
        &buffered_escrow,
        expected_escrow_authority,
        claim.escrow_ata,
        claim.escrow_mint,
        claim.escrow_amount_locked,
        claim.funding_state,
    )?;

    let payout_atoms = claim.due_atoms.saturating_sub(claim.paid_atoms);
    require!(payout_atoms > 0, SterlingError::ZeroStableSettlement);

    let escrow_bump_seed = [escrow_bump];
    let escrow_signer_seeds: &[&[&[u8]]] =
        &[&[b"claim_live_escrow", claim_key.as_ref(), &escrow_bump_seed]];

    burn_tokens_signed_account_info(
        token_program_info.clone(),
        buffered_escrow.mint_info.clone(),
        buffered_escrow.ata_info.clone(),
        buffered_escrow.authority_info.clone(),
        escrow_signer_seeds,
        claim.escrow_amount_locked,
    )?;

    let now = Clock::get()?.unix_timestamp;
    claim.paid_atoms = claim.due_atoms;
    claim.proof_sig = live_execution_proof_sig();
    claim.settled_ts = now;
    claim.status = 1;

    let native_sol_config = load_native_sol_rail_config(&config_info)?;
    let payout_amount = match resolve_buffered_output_target(
        native_sol_config,
        payout_atoms,
        primary_target,
        secondary_target,
        native_target,
    )? {
        SovereignBufferedOutputTarget::Stable(target) => {
            transfer_from_stable_target_signed(
                &target,
                config_info.clone(),
                token_program_info.clone(),
                config_bump,
                payout_atoms,
            )?;
            claim.payout_mint = target.payout_mint;
            claim.destination_ata = target.destination_key;
            payout_atoms
        }
        SovereignBufferedOutputTarget::NativeSol(target) => {
            let payout_lamports =
                native_sol_payout_lamports(native_sol_config, &target, payout_atoms)?;
            transfer_native_sol_from_config(config_info.clone(), &target, payout_lamports)?;
            claim.payout_mint = native_sol_marker_pubkey();
            claim.destination_ata = target.destination_key;
            payout_lamports
        }
    };
    claim.funding_state = SOVEREIGN_ESCROW_STATE_SETTLED;

    let mut out: &mut [u8] = &mut claim_data;
    claim.try_serialize(&mut out)?;

    emit_sovereign_output_event(
        5,
        claim.mint_in,
        claim.amount_in,
        claim.payout_mint,
        payout_amount,
        claim.destination_ata,
        claim.user,
    )
}

#[inline(never)]
fn execute_protocol_debt_buffered_payout<'info>(
    config_info: AccountInfo<'info>,
    token_program_info: AccountInfo<'info>,
    config_bump: u8,
    payout_atoms: u64,
    payout_policy: &SovereignDebtPayoutPolicy,
    settlement_targets: &SovereignBufferedSettlementTargets<'info>,
) -> Result<(Pubkey, u64, Pubkey)> {
    let native_sol_config = load_native_sol_rail_config(&config_info)?;

    match resolve_buffered_output_target(
        native_sol_config,
        payout_atoms,
        settlement_targets.primary_target.clone(),
        settlement_targets.secondary_target.clone(),
        settlement_targets.native_target.clone(),
    )? {
        SovereignBufferedOutputTarget::Stable(target) => {
            if target.payout_mint == payout_policy.canonical_usdc_mint {
                require!(
                    target.destination_key == payout_policy.treasury_usdc_ata,
                    SterlingError::InvalidAccount
                );
            } else if target.payout_mint == payout_policy.canonical_usdt_mint {
                require!(
                    target.destination_key == payout_policy.treasury_usdt_ata,
                    SterlingError::InvalidAccount
                );
            } else if is_configured_extra_payout_mint(
                target.payout_mint,
                payout_policy.extra_payout_mint_0,
                payout_policy.extra_payout_mint_1,
                payout_policy.extra_payout_mint_2,
                payout_policy.extra_payout_mint_3,
            ) {
                let destination = load_token_account_snapshot(&target.destination_info)?;
                require!(
                    destination.owner == payout_policy.main_wallet,
                    SterlingError::InvalidAccount
                );
                require!(
                    destination.mint == target.payout_mint,
                    SterlingError::InvalidAccount
                );
            } else {
                return err!(SterlingError::UnsupportedSovereignFinalOutput);
            }

            transfer_from_stable_target_signed(
                &target,
                config_info,
                token_program_info,
                config_bump,
                payout_atoms,
            )?;
            Ok((target.payout_mint, payout_atoms, target.destination_key))
        }
        SovereignBufferedOutputTarget::NativeSol(target) => {
            require!(
                target.destination_key == payout_policy.main_wallet,
                SterlingError::InvalidNativeSolDestination
            );
            let payout_lamports =
                native_sol_payout_lamports(native_sol_config, &target, payout_atoms)?;
            transfer_native_sol_from_config(config_info, &target, payout_lamports)?;
            Ok((
                native_sol_marker_pubkey(),
                payout_lamports,
                target.destination_key,
            ))
        }
    }
}

#[inline(never)]
fn settle_protocol_debt_to_stable<'a, 'b, 'c>(
    config_info: AccountInfo<'a>,
    user_key: Pubkey,
    token_program_info: AccountInfo<'a>,
    pool_info: AccountInfo<'b>,
    ledger_info: AccountInfo<'c>,
    debt_lot_nonce: u64,
    buffered_escrow: SovereignBufferedEscrowAccounts<'a>,
    config_bump: u8,
    payout_policy: &SovereignDebtPayoutPolicy,
    settlement_targets: &SovereignBufferedSettlementTargets<'a>,
) -> Result<()> {
    require!(
        user_key == payout_policy.main_wallet,
        SterlingError::InvalidAccount
    );
    let pool_key = pool_info.key();
    let payment_snapshot =
        load_protocol_debt_lot_payment_snapshot(&pool_info, &ledger_info, debt_lot_nonce)?;
    let debt_snapshot = load_protocol_debt_live_snapshot(
        &pool_info,
        &ledger_info,
        debt_lot_nonce,
        &load_token_account_snapshot(&buffered_escrow.ata_info)?,
        buffered_escrow.ata_info.key(),
    )?;
    let (expected_escrow_authority, escrow_bump) =
        expected_protocol_debt_lot_escrow_authority(pool_key, debt_snapshot.nonce);
    let debt_nonce_bytes = debt_snapshot.nonce.to_le_bytes();
    let escrow_bump_seed = [escrow_bump];
    let escrow_signer_seeds: &[&[&[u8]]] = &[&[
        b"debt_live_escrow",
        pool_key.as_ref(),
        &debt_nonce_bytes,
        &escrow_bump_seed,
    ]];

    validate_buffered_route_escrow_accounts(
        &buffered_escrow,
        expected_escrow_authority,
        debt_snapshot.escrow_ata,
        debt_snapshot.escrow_mint,
        debt_snapshot.escrow_amount_locked,
        debt_snapshot.funding_state,
    )?;

    let payout_atoms = usd_micros_to_atoms(payment_snapshot.usd_micros, STABLE_CASH_DECIMALS)?;
    require!(payout_atoms > 0, SterlingError::ZeroStableSettlement);

    burn_tokens_signed_account_info(
        token_program_info.clone(),
        buffered_escrow.mint_info.clone(),
        buffered_escrow.ata_info.clone(),
        buffered_escrow.authority_info.clone(),
        escrow_signer_seeds,
        debt_snapshot.escrow_amount_locked,
    )?;

    let (payout_mint, payout_amount, destination_key) = execute_protocol_debt_buffered_payout(
        config_info.clone(),
        token_program_info.clone(),
        config_bump,
        payout_atoms,
        payout_policy,
        settlement_targets,
    )?;

    let source_mint = payment_snapshot.mint_in;
    let source_amount = payment_snapshot.amount_in;
    apply_protocol_debt_lot_settlement(pool_info, ledger_info, debt_lot_nonce)?;

    emit_sovereign_output_event(
        6,
        source_mint,
        source_amount,
        payout_mint,
        payout_amount,
        destination_key,
        user_key,
    )
}

fn expected_proof_kind_for_stage(stage: SettlementFundingStage) -> SettlementProofKind {
    match stage {
        SettlementFundingStage::EconomicValueUsd => SettlementProofKind::CertifiedClaimSnapshot,
        SettlementFundingStage::TechnicalFundingRail => SettlementProofKind::LiveFundingExecution,
        SettlementFundingStage::ProviderPayout => SettlementProofKind::ProviderPayoutReceipt,
    }
}

fn settlement_proof_kind_from_sig(proof_sig: &[u8; 64]) -> Option<SettlementProofKind> {
    match proof_sig[0] {
        x if x == SettlementProofKind::CertifiedClaimSnapshot as u8 => {
            Some(SettlementProofKind::CertifiedClaimSnapshot)
        }
        x if x == SettlementProofKind::LiveFundingExecution as u8 => {
            Some(SettlementProofKind::LiveFundingExecution)
        }
        x if x == SettlementProofKind::ProviderPayoutReceipt as u8 => {
            Some(SettlementProofKind::ProviderPayoutReceipt)
        }
        _ => None,
    }
}

fn require_proof_kind_for_stage(proof_sig: &[u8; 64], stage: SettlementFundingStage) -> Result<()> {
    require!(*proof_sig != [0u8; 64], SterlingError::InvalidAccount);
    let expected = expected_proof_kind_for_stage(stage);
    let actual = settlement_proof_kind_from_sig(proof_sig);
    require!(actual == Some(expected), SterlingError::InvalidState);
    Ok(())
}

// --------------------
// HELPERS (rooted at crate::helpers)
// --------------------
pub(crate) mod helpers {
    use crate::Config;
    use crate::SterlingError;

    use super::*;

    pub(crate) fn require_admin<'info>(cfg: &Config, admin: &Signer<'info>) -> Result<()> {
        require!(cfg.admin == admin.key(), SterlingError::Unauthorized);
        Ok(())
    }

    pub(crate) fn pubkey_from_str(s: &str) -> Pubkey {
        Pubkey::from_str(s).expect("invalid pubkey string")
    }
}

// =========================
// PAO V3 — SettlementClaim (on-chain debt receipt)
// =========================
#[account]
pub struct SettlementClaim {
    pub pool: Pubkey,
    pub payout_mint: Pubkey,
    pub payout_kind: u8,
    pub user: Pubkey,
    pub mint_in: Pubkey,
    pub amount_in: u64,
    pub usd_micros: u64,
    // Verifiable settlement fields
    pub due_atoms: u64,
    pub paid_atoms: u64,
    pub proof_sig: [u8; 64],
    pub destination_ata: Pubkey,
    pub escrow_mint: Pubkey,
    pub escrow_ata: Pubkey,
    pub escrow_amount_locked: u64,
    pub nonce: u64,
    pub created_ts: i64,
    pub settled_ts: i64,
    pub status: u8, // 0=open, 1=paid
    pub funding_state: u8,
    pub escrow_bump: u8,
    pub bump: u8,
}

impl SettlementClaim {
    pub const LEN: usize =
        8 + (32 + 32 + 1 + 32 + 32 + 8 + 8 + 8 + 8 + 64 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 1 + 1 + 1 + 1);
}
#[event]
pub struct ClaimCreatedEvent {
    pub pool: Pubkey,
    pub payout_mint: Pubkey,
    pub payout_kind: u8,
    pub user: Pubkey,
    pub usd_micros: u64,
    pub destination_ata: Pubkey,
    pub nonce: u64,
    pub ts: i64,
}

#[event]
pub struct ClaimPaidEvent {
    pub pool: Pubkey,
    pub payout_mint: Pubkey,
    pub user: Pubkey,
    pub usd_micros: u64,
    pub paid_atoms: u64,
    pub proof_sig: [u8; 64],
    pub destination_ata: Pubkey,
    pub nonce: u64,
    pub ts: i64,
}
#[derive(Accounts)]
#[instruction(nonce: u64)]
pub struct CreateClaimFromTicket<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub keeper: Signer<'info>,

    /// CHECK: claim beneficiary (receipt-only)
    pub user: UncheckedAccount<'info>,

    /// CHECK: receipt-only
    pub pool: UncheckedAccount<'info>,

    #[account(
        seeds = [b"payout", user.key().as_ref(), pool.key().as_ref(), &nonce.to_le_bytes()],
        bump
    )]
    pub ticket: Box<Account<'info, PayoutTicket>>,

    #[account(address = ticket.payout_mint)]
    pub payout_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = keeper,
        seeds = [b"claim", user.key().as_ref(), pool.key().as_ref(), &nonce.to_le_bytes()],
        bump,
        space = SettlementClaim::LEN
    )]
    pub claim: Box<Account<'info, SettlementClaim>>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(nonce: u64)]
pub struct SettleClaimPaid<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub keeper: Signer<'info>,

    /// CHECK: receipt-only
    pub user: UncheckedAccount<'info>,
    /// CHECK: receipt-only
    pub pool: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"claim", user.key().as_ref(), pool.key().as_ref(), &nonce.to_le_bytes()],
        bump
    )]
    pub claim: Box<Account<'info, SettlementClaim>>,
}

#[derive(Accounts)]
#[instruction(nonce: u64)]
pub struct ConfirmTicketLiveEscrowFunding<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub keeper: Signer<'info>,

    /// CHECK: receipt-only
    pub user: UncheckedAccount<'info>,
    /// CHECK: receipt-only
    pub pool: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"payout", user.key().as_ref(), pool.key().as_ref(), &nonce.to_le_bytes()],
        bump
    )]
    pub ticket: Box<Account<'info, PayoutTicket>>,

    #[account(mut)]
    pub source_escrow_ata: Box<Account<'info, TokenAccount>>,
}

#[derive(Accounts)]
#[instruction(nonce: u64)]
pub struct ConfirmClaimLiveEscrowFunding<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub keeper: Signer<'info>,

    /// CHECK: receipt-only
    pub user: UncheckedAccount<'info>,
    /// CHECK: receipt-only
    pub pool: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"claim", user.key().as_ref(), pool.key().as_ref(), &nonce.to_le_bytes()],
        bump
    )]
    pub claim: Box<Account<'info, SettlementClaim>>,

    #[account(mut)]
    pub source_escrow_ata: Box<Account<'info, TokenAccount>>,
}

#[derive(Accounts)]
#[instruction(nonce: u64)]
pub struct ConfirmProtocolDebtLotLiveEscrowFunding<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub keeper: Signer<'info>,
    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Box<Account<'info, Pool>>,
    /// CHECK: protocol debt ledger is parsed lazily to keep the temp claim build under SBF limits
    #[account(mut, seeds = [b"protocol_debt", pool.key().as_ref()], bump, owner = crate::ID)]
    pub protocol_debt_ledger: UncheckedAccount<'info>,
    #[account(mut)]
    pub source_escrow_ata: Box<Account<'info, TokenAccount>>,
}

#[derive(Accounts)]
pub struct SettleProtocolFeeDebt<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub keeper: Signer<'info>,
    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Box<Account<'info, Pool>>,
    /// CHECK: protocol debt ledger is parsed lazily to keep the temp claim build under SBF limits
    #[account(mut, seeds = [b"protocol_debt", pool.key().as_ref()], bump, owner = crate::ID)]
    pub protocol_debt_ledger: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct InitProtocolDebtLedger<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Box<Account<'info, Pool>>,
    #[account(
        init,
        payer = admin,
        seeds = [b"protocol_debt", pool.key().as_ref()],
        bump,
        space = ProtocolDebtLedger::LEN
    )]
    /// CHECK: initialized and written manually to keep the temp claim build under SBF limits
    pub protocol_debt_ledger: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}
