use anchor_lang::prelude::*;
use anchor_spl::token::spl_token::instruction::AuthorityType;
use anchor_spl::token::{self, Burn, Mint, MintTo, SetAuthority, Token, TokenAccount, Transfer};
use std::str::FromStr;

declare_id!("7v9sLrk92NNLLUfXLJw3o7MycZNvwsTK6kLWfWb8vcVA");

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
pub const USDT_MAIN_MINT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
pub const USDT_OLD_MINT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";

// Portal BTC (si tu t'en sers)
pub const BTC_PORTAL_MINT: &str = "9n4nbM75f5Ui33ZbPYXn59EwSgE8CGsHtAeTH5YFeJ9E";

// LPs
pub const LP_MINT_2: &str = "G94nkBm4ntjiEHNzTpd7GRW9J8H5rqrhW83k5RSHZrBZ";
pub const LP_AUTH_2: &str = "Htopqis52g8nGvvkpnG7Z7XZhgBpqtN9huqUyk6LH9gB";
pub const LP_MINT_3: &str = "DnepvMafJZzDtDcevrqbMUCmDqdNhBLjCTUu1xhR4HeL";
pub const LP_AUTH_3: &str = "CMqD45Kq5oukPvaMDhzav5RxJqZb1xME1MmV71CzCeTw";

// New mints
pub const MINT_H1: &str = "H1hutftquntrMVid7btBsBw4XLjGLVwxfv9exYdMwsr9";
pub const MINT_FR15: &str = "FR15AHXBE8vDpYASFmCbbCD6yso4XiD6XiEXAF6dGLix";
pub const MINT_DB8: &str = "DB8gCpC6Qs1c4CPfvPXT7RcFmqxzY5g6i8Djo5j5A2nq";

// Coffres (token accounts SPL)
pub const USDC_COFFRE: &str = "7vWLrATXnuGTCjmexa7b4roo9Em6VMKr3bdDemJNHNk1";
pub const USDT_COFFRE: &str = "GfG1XmzxSTzvfexRLmzDZCUNCdkdAWe9BBtL62psEcHd";

// Divers
pub const PDA_GT: &str = "GTAs9L3XFdhHEFoo6KWNbFFxMCFRnbVomsbx7deShkLb";
pub const COFFRE_7Q: &str = "7qeqQYVgLaeaDWi4X4Hin9wpauCxZFQB5Bov9zKFev2W";
pub const POOL_ID: &str = "BbvR4zUAwZF8LmVFLXNpDy3CxuYcDwd5isoh7CZFAF5G";

// Off-chain wallets (ne PAS parser en Pubkey)
pub const BTC_WALLET: &str = "bc1qmxnh7wrddrfcjcmyx72cmgf88ykew74gl9v5tl";
pub const ETH_WALLET: &str = "0x7d2f9d6e73f9a57832feb80097148debfac2ffa4";

// Flags
pub const TRUE_CASH_FLAG: bool = true;
pub const CASH_BACKED_FLAG: bool = true;
pub const REAL_PEG_FLAG: bool = true;
pub const SOVEREIGN_FLAG: bool = true;

// Values
pub const USD_MICROS: u64 = 1_000_000;
pub const DEFAULT_TOKEN_VALUE_USD_MICROS: u64 = 1 * USD_MICROS;
pub const DEFAULT_TREASURY_VALUE_USD_MICROS: u64 = 174_000 * USD_MICROS;

// Staking
pub const DEFAULT_CASHBACK_BPS: u16 = 9200; // 92%
pub const DEFAULT_REWARD_INTERVAL_SECONDS: u64 = 300; // 5 min

// Swap fees
pub const DEFAULT_SWAP_FEE_BPS: u16 = 30; // 0.30%
pub const MAX_SWAP_FEE_BPS: u16 = 1000; // 10%

// Auto-bridge threshold (ex: 10 * treasury value)
pub const DEFAULT_FEE_THRESHOLD_USD_MICROS: u64 = 10 * DEFAULT_TREASURY_VALUE_USD_MICROS;

// Main wallets (Solana pubkeys)
pub const MAIN_WALLET: &str = "CMqD45Kq5oukPvaMDhzav5RxJqZb1xME1MmV71CzCeTw"; // Compte C
pub const OKX_WALLET: &str = "GQo15C7g4rbJ7zzhAzSd3SzeMAYpPtNhD8U8P3QrL7MU";

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

// =========================
// PROGRAM
// =========================
#[program]
pub mod sterling_amm_mainnet {
    use super::*;

    // =========================
    // A) CONFIG
    // =========================
    pub fn initialize_config(ctx: Context<InitializeConfig>) -> Result<()> {
        let cfg = &mut ctx.accounts.config;

        cfg.admin = ctx.accounts.admin.key();

        cfg.true_cash = TRUE_CASH_FLAG;
        cfg.cash_backed = CASH_BACKED_FLAG;
        cfg.real_peg = REAL_PEG_FLAG;
        cfg.sovereign = SOVEREIGN_FLAG;

        cfg.token_value_usd_micros_default = DEFAULT_TOKEN_VALUE_USD_MICROS;
        cfg.treasury_value_usd_micros = DEFAULT_TREASURY_VALUE_USD_MICROS;

        cfg.cashback_bps = DEFAULT_CASHBACK_BPS;
        cfg.reward_interval = DEFAULT_REWARD_INTERVAL_SECONDS;

        cfg.allow_fallback_usdt = true;

        cfg.enable_treasury = true;
        cfg.enable_sjbc = true;
        cfg.enable_sjbc2 = true;
        cfg.enable_sjbc3 = true;
        cfg.enable_usdt_main = true;
        cfg.enable_usdt_old = true;
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

        // Coffres SPL (Token Accounts)
        cfg.usdc_coffre = pubkey_from_str(USDC_COFFRE);
        cfg.usdt_coffre = pubkey_from_str(USDT_COFFRE);

        cfg.pda_gt = pubkey_from_str(PDA_GT);
        cfg.coffre_7q = pubkey_from_str(COFFRE_7Q);
        cfg.pool_id = pubkey_from_str(POOL_ID);

        // Destination fees (wallet C)
        cfg.treasury_usdc_ata = pubkey_from_str(TREASURY_USDC_ATA);
        cfg.treasury_usdt_ata = pubkey_from_str(TREASURY_USDT_ATA);

        // Auto-collect (tous les N swaps) : 10 par défaut
        cfg.auto_collect_every_swaps = 10;

        cfg.fee_threshold_usd_micros = DEFAULT_FEE_THRESHOLD_USD_MICROS;

        cfg.bump = *ctx.bumps.get("config").unwrap();
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
        require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
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

    pub fn set_valuation(
        ctx: Context<AdminOnly>,
        token_value_usd_micros_default: u64,
        treasury_value_usd_micros: u64,
    ) -> Result<()> {
        require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
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
        require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        set_enabled_flag(&mut ctx.accounts.config, &mint, enabled)
    }

    // =========================
    // B) ORACLE ON-CHAIN : ValueRegistry
    // =========================
    pub fn init_value_registry(
        ctx: Context<InitValueRegistry>,
        value_usd_micros: u64,
    ) -> Result<()> {
        require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        require!(value_usd_micros > 0, SterlingError::InvalidAmount);

        let reg = &mut ctx.accounts.value_registry;
        reg.mint = ctx.accounts.mint.key();
        reg.value_usd_micros = value_usd_micros;

        reg.true_cash = TRUE_CASH_FLAG;
        reg.cash_backed = CASH_BACKED_FLAG;
        reg.real_peg = REAL_PEG_FLAG;
        reg.sovereign = SOVEREIGN_FLAG;

        reg.updated_at = Clock::get()?.unix_timestamp;
        reg.bump = *ctx.bumps.get("value_registry").unwrap();
        Ok(())
    }

    pub fn set_token_value(ctx: Context<SetTokenValue>, value_usd_micros: u64) -> Result<()> {
        require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
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

    // =========================
    // C) STAKING (vaults) + fallback USDT
    // =========================
    pub fn init_stake_vault(ctx: Context<InitStakeVault>) -> Result<()> {
        require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        let mint = ctx.accounts.mint.key();
        require!(
            is_supported_cash_mint(&mint),
            SterlingError::UnsupportedMint
        );
        require!(
            is_enabled(&ctx.accounts.config, &mint),
            SterlingError::MintDisabled
        );
        Ok(())
    }

    pub fn init_reward_vault(ctx: Context<InitRewardVault>) -> Result<()> {
        require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        let mint = ctx.accounts.mint.key();
        require!(
            is_supported_cash_mint(&mint),
            SterlingError::UnsupportedMint
        );
        require!(
            is_enabled(&ctx.accounts.config, &mint),
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
            is_supported_cash_mint(&stake_mint),
            SterlingError::UnsupportedMint
        );
        require!(
            is_supported_cash_mint(&payout_mint),
            SterlingError::UnsupportedMint
        );
        require!(is_enabled(cfg, &stake_mint), SterlingError::MintDisabled);
        require!(is_enabled(cfg, &payout_mint), SterlingError::MintDisabled);

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
            pos.bump = *ctx.bumps.get("position").unwrap();
        } else {
            require!(
                pos.owner == ctx.accounts.user.key(),
                SterlingError::Unauthorized
            );
        }

        pos.amount = pos.amount.saturating_add(amount);
        Ok(())
    }

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let cfg = &ctx.accounts.config;
        let pos = &mut ctx.accounts.position;

        require!(
            pos.owner == ctx.accounts.user.key(),
            SterlingError::Unauthorized
        );
        require!(pos.amount > 0, SterlingError::InvalidAmount);
        require!(
            now >= pos.last_claim_ts + (cfg.reward_interval as i64),
            SterlingError::TooEarlyClaim
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

        require!(
            ctx.accounts.user_usdt_old_ata.owner == ctx.accounts.user.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_usdt_old_ata.mint == ctx.accounts.usdt_old_mint.key(),
            SterlingError::InvalidAccount
        );

        let reward_amount: u64 = (pos.amount as u128)
            .saturating_mul(cfg.cashback_bps as u128)
            .checked_div(10_000u128)
            .ok_or(SterlingError::MathOverflow)? as u64;

        require!(reward_amount > 0, SterlingError::ZeroReward);
        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[*ctx.bumps.get("config").unwrap()]]];

        if ctx.accounts.reward_vault.amount >= reward_amount {
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
        } else {
            require!(cfg.allow_fallback_usdt, SterlingError::InsufficientRewards);

            if ctx.accounts.usdt_main_vault.amount >= reward_amount {
                token::transfer(
                    CpiContext::new(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: ctx.accounts.usdt_main_vault.to_account_info(),
                            to: ctx.accounts.user_usdt_main_ata.to_account_info(),
                            authority: ctx.accounts.config.to_account_info(),
                        },
                    )
                    .with_signer(signer_seeds),
                    reward_amount,
                )?;
            } else if ctx.accounts.usdt_old_vault.amount >= reward_amount {
                token::transfer(
                    CpiContext::new(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: ctx.accounts.usdt_old_vault.to_account_info(),
                            to: ctx.accounts.user_usdt_old_ata.to_account_info(),
                            authority: ctx.accounts.config.to_account_info(),
                        },
                    )
                    .with_signer(signer_seeds),
                    reward_amount,
                )?;
            } else {
                return err!(SterlingError::InsufficientRewards);
            }
        }

        pos.last_claim_ts = now;
        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        require!(amount > 0, SterlingError::InvalidAmount);

        require!(
            ctx.accounts.user_stake_ata.owner == ctx.accounts.user.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_stake_ata.mint == ctx.accounts.stake_mint.key(),
            SterlingError::InvalidAccount
        );

        let pos = &mut ctx.accounts.position;
        require!(
            pos.owner == ctx.accounts.user.key(),
            SterlingError::Unauthorized
        );
        require!(pos.amount >= amount, SterlingError::InvalidAmount);
        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[*ctx.bumps.get("config").unwrap()]]];

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.stake_vault.to_account_info(),
                    to: ctx.accounts.user_stake_ata.to_account_info(),
                    authority: ctx.accounts.config.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            amount,
        )?;

        pos.amount = pos.amount.saturating_sub(amount);
        Ok(())
    }

    // =========================
    // D) DEX : POOL + LP + SWAPS + FEES VAULTS (base + quote)
    // =========================
    pub fn create_pool(ctx: Context<CreatePool>, fee_bps: u16) -> Result<()> {
        require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        require!(fee_bps <= MAX_SWAP_FEE_BPS, SterlingError::InvalidBps);

        let base_mint = ctx.accounts.base_mint.key();
        let quote_mint = ctx.accounts.quote_mint.key();

        require!(
            ctx.accounts.base_value_registry.mint == base_mint,
            SterlingError::BadRegistry
        );
        require!(
            ctx.accounts.quote_value_registry.mint == quote_mint,
            SterlingError::BadRegistry
        );

        let pool = &mut ctx.accounts.pool;
        pool.owner = ctx.accounts.admin.key();
        pool.base_mint = base_mint;
        pool.quote_mint = quote_mint;

        let (base_vault, _) = Pubkey::find_program_address(
            &[b"pool_vault", pool.key().as_ref(), b"base"],
            ctx.program_id,
        );
        let (quote_vault, _) = Pubkey::find_program_address(
            &[b"pool_vault", pool.key().as_ref(), b"quote"],
            ctx.program_id,
        );
        let (lp_mint, _) =
            Pubkey::find_program_address(&[b"lp_mint", pool.key().as_ref()], ctx.program_id);

        let (fee_vault_base, _) = Pubkey::find_program_address(
            &[b"fee_vault", pool.key().as_ref(), b"base"],
            ctx.program_id,
        );
        let (fee_vault_quote, _) = Pubkey::find_program_address(
            &[b"fee_vault", pool.key().as_ref(), b"quote"],
            ctx.program_id,
        );

        pool.base_vault = base_vault;
        pool.quote_vault = quote_vault;
        pool.lp_mint = lp_mint;
        pool.fee_vault_base = fee_vault_base;
        pool.fee_vault_quote = fee_vault_quote;

        pool.base_value_usd_micros = ctx.accounts.base_value_registry.value_usd_micros;
        pool.quote_value_usd_micros = ctx.accounts.quote_value_registry.value_usd_micros;

        pool.true_cash = ctx.accounts.base_value_registry.true_cash
            && ctx.accounts.quote_value_registry.true_cash;
        pool.cash_backed = ctx.accounts.base_value_registry.cash_backed
            && ctx.accounts.quote_value_registry.cash_backed;
        pool.real_peg =
            ctx.accounts.base_value_registry.real_peg && ctx.accounts.quote_value_registry.real_peg;
        pool.sovereign = ctx.accounts.base_value_registry.sovereign
            && ctx.accounts.quote_value_registry.sovereign;

        require!(
            pool.true_cash && pool.cash_backed && pool.real_peg && pool.sovereign,
            SterlingError::CashFlagsRequired
        );

        pool.fee_bps = fee_bps;
        pool.active = true;

        // compteur swaps
        pool.swap_count = 0;

        pool.bump = *ctx.bumps.get("pool").unwrap();
        emit!(PoolCreated {
            pool: pool.key(),
            base_mint: pool.base_mint,
            quote_mint: pool.quote_mint,
            base_vault: pool.base_vault,
            quote_vault: pool.quote_vault,
            lp_mint: pool.lp_mint,
            fee_bps: pool.fee_bps,
            ts: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    pub fn init_pool_base_vault(ctx: Context<InitPoolBaseVault>) -> Result<()> {
        require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        require!(
            ctx.accounts.pool.base_mint == ctx.accounts.base_mint.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.pool.base_vault == ctx.accounts.base_vault.key(),
            SterlingError::InvalidAccount
        );
        Ok(())
    }

    pub fn init_pool_quote_vault(ctx: Context<InitPoolQuoteVault>) -> Result<()> {
        require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        require!(
            ctx.accounts.pool.quote_mint == ctx.accounts.quote_mint.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.pool.quote_vault == ctx.accounts.quote_vault.key(),
            SterlingError::InvalidAccount
        );
        Ok(())
    }

    pub fn init_pool_lp_mint(ctx: Context<InitPoolLpMint>) -> Result<()> {
        require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        require!(
            ctx.accounts.pool.lp_mint == ctx.accounts.lp_mint.key(),
            SterlingError::InvalidAccount
        );
        Ok(())
    }

    pub fn init_pool_fee_vault_base(ctx: Context<InitPoolFeeVaultBase>) -> Result<()> {
        require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        require!(
            ctx.accounts.pool.fee_vault_base == ctx.accounts.fee_vault_base.key(),
            SterlingError::InvalidAccount
        );
        Ok(())
    }

    pub fn init_pool_fee_vault_quote(ctx: Context<InitPoolFeeVaultQuote>) -> Result<()> {
        require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        require!(
            ctx.accounts.pool.fee_vault_quote == ctx.accounts.fee_vault_quote.key(),
            SterlingError::InvalidAccount
        );
        Ok(())
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_base: u64,
        amount_quote: u64,
        min_lp_out: u64,
    ) -> Result<()> {
        require!(
            amount_base > 0 && amount_quote > 0,
            SterlingError::InvalidAmount
        );

        let pool = &ctx.accounts.pool;
        require!(pool.active, SterlingError::InactivePool);

        require!(
            ctx.accounts.user_base_ata.owner == ctx.accounts.user.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_quote_ata.owner == ctx.accounts.user.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_lp_ata.owner == ctx.accounts.user.key(),
            SterlingError::InvalidAccount
        );

        require!(
            ctx.accounts.user_base_ata.mint == pool.base_mint,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_quote_ata.mint == pool.quote_mint,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_lp_ata.mint == pool.lp_mint,
            SterlingError::InvalidAccount
        );

        require!(
            ctx.accounts.base_vault.key() == pool.base_vault,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.quote_vault.key() == pool.quote_vault,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.lp_mint.key() == pool.lp_mint,
            SterlingError::InvalidAccount
        );

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_base_ata.to_account_info(),
                    to: ctx.accounts.base_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_base,
        )?;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_quote_ata.to_account_info(),
                    to: ctx.accounts.quote_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_quote,
        )?;

        let supply = ctx.accounts.lp_mint.supply;
        let base_after = ctx.accounts.base_vault.amount;
        let quote_after = ctx.accounts.quote_vault.amount;

        let base_before = base_after.saturating_sub(amount_base);
        let quote_before = quote_after.saturating_sub(amount_quote);

        let lp_out: u64 = if supply == 0 {
            integer_sqrt_u128((amount_base as u128).saturating_mul(amount_quote as u128)) as u64
        } else {
            require!(
                base_before > 0 && quote_before > 0,
                SterlingError::MathOverflow
            );

            let x = base_before as u128;
            let y = quote_before as u128;
            let dx = amount_base as u128;
            let dy = amount_quote as u128;

            let k_before = x.checked_mul(y).ok_or(SterlingError::MathOverflow)?;
            let k_after = x
                .saturating_add(dx)
                .checked_mul(y.saturating_add(dy))
                .ok_or(SterlingError::MathOverflow)?;

            let sqrt_before = integer_sqrt_u128(k_before);
            let sqrt_after = integer_sqrt_u128(k_after);

            require!(sqrt_before > 0, SterlingError::MathOverflow);
            require!(sqrt_after > sqrt_before, SterlingError::ZeroLp);

            (supply as u128)
                .checked_mul(sqrt_after.saturating_sub(sqrt_before))
                .ok_or(SterlingError::MathOverflow)?
                .checked_div(sqrt_before)
                .ok_or(SterlingError::MathOverflow)? as u64
        };

        require!(lp_out >= min_lp_out, SterlingError::SlippageExceeded);
        require!(lp_out > 0, SterlingError::ZeroLp);
        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[*ctx.bumps.get("config").unwrap()]]];

        token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.lp_mint.to_account_info(),
                    to: ctx.accounts.user_lp_ata.to_account_info(),
                    authority: ctx.accounts.config.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            lp_out,
        )?;

        Ok(())
    }

    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        lp_amount: u64,
        min_base_out: u64,
        min_quote_out: u64,
    ) -> Result<()> {
        require!(lp_amount > 0, SterlingError::InvalidAmount);

        let pool = &ctx.accounts.pool;
        require!(pool.active, SterlingError::InactivePool);

        require!(
            ctx.accounts.user_base_ata.owner == ctx.accounts.user.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_quote_ata.owner == ctx.accounts.user.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_lp_ata.owner == ctx.accounts.user.key(),
            SterlingError::InvalidAccount
        );

        require!(
            ctx.accounts.user_base_ata.mint == pool.base_mint,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_quote_ata.mint == pool.quote_mint,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_lp_ata.mint == pool.lp_mint,
            SterlingError::InvalidAccount
        );

        require!(
            ctx.accounts.base_vault.key() == pool.base_vault,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.quote_vault.key() == pool.quote_vault,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.lp_mint.key() == pool.lp_mint,
            SterlingError::InvalidAccount
        );

        let supply = ctx.accounts.lp_mint.supply;
        require!(supply > 0, SterlingError::ZeroLp);

        let base_reserve = ctx.accounts.base_vault.amount as u128;
        let quote_reserve = ctx.accounts.quote_vault.amount as u128;

        let base_out = (lp_amount as u128)
            .saturating_mul(base_reserve)
            .checked_div(supply as u128)
            .ok_or(SterlingError::MathOverflow)? as u64;

        let quote_out = (lp_amount as u128)
            .saturating_mul(quote_reserve)
            .checked_div(supply as u128)
            .ok_or(SterlingError::MathOverflow)? as u64;

        require!(
            base_out >= min_base_out && quote_out >= min_quote_out,
            SterlingError::SlippageExceeded
        );

        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.lp_mint.to_account_info(),
                    from: ctx.accounts.user_lp_ata.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            lp_amount,
        )?;
        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[*ctx.bumps.get("config").unwrap()]]];

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
            base_out,
        )?;

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
            quote_out,
        )?;

        Ok(())
    }

    pub fn swap_base_for_quote<'info>(
        ctx: Context<'_, '_, '_, 'info, SwapBaseForQuote<'info>>,
        amount_in: u64,
        min_out: u64,
    ) -> Result<()> {
        require!(amount_in > 0, SterlingError::InvalidAmount);

        let pool = &mut ctx.accounts.pool;
        require!(pool.active, SterlingError::InactivePool);

        require!(
            ctx.accounts.user_base_ata.owner == ctx.accounts.user.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_quote_ata.owner == ctx.accounts.user.key(),
            SterlingError::InvalidAccount
        );

        require!(
            ctx.accounts.user_base_ata.mint == pool.base_mint,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_quote_ata.mint == pool.quote_mint,
            SterlingError::InvalidAccount
        );

        require!(
            ctx.accounts.base_vault.key() == pool.base_vault,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.quote_vault.key() == pool.quote_vault,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.fee_vault_base.key() == pool.fee_vault_base,
            SterlingError::InvalidAccount
        );

        let fee_bps = pool.fee_bps as u128;
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

        // 1) input
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

        // 2) fee -> fee_vault_base
        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[*ctx.bumps.get("config").unwrap()]]];

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

        // 3) threshold event (USD)
        let fee_usd_micros = ((ctx.accounts.base_value_registry.value_usd_micros as u128)
            .saturating_mul(ctx.accounts.fee_vault_base.amount as u128)
            .checked_div(USD_MICROS as u128)
            .ok_or(SterlingError::MathOverflow)?) as u64;

        if fee_usd_micros >= configured_fee_threshold_usd_micros_compat(&ctx.accounts.config)? {
            emit!(FeeThresholdEvent {
                pool: pool.key(),
                fee_vault: ctx.accounts.fee_vault_base.key(),
                fee_amount: ctx.accounts.fee_vault_base.amount,
                fee_value_usd_micros: fee_usd_micros,
                ts: Clock::get()?.unix_timestamp,
            });
        }

        // AutoCollectReadyEvent (tous les N swaps)
        pool.swap_count = pool.swap_count.saturating_add(1);
        let n = configured_auto_collect_every_swaps_compat(&ctx.accounts.config)?;
        if n > 0 && (pool.swap_count % n == 0) {
            emit!(AutoCollectReadyEvent {
                pool: pool.key(),
                side: 0, // base
                fee_vault: ctx.accounts.fee_vault_base.key(),
                fee_amount: ctx.accounts.fee_vault_base.amount,
                ts: Clock::get()?.unix_timestamp,
            });
        }

        maybe_auto_settle_fee_to_usdc(
            &ctx.accounts.config,
            pool.key(),
            &ctx.accounts.fee_vault_base,
            pool.base_mint,
            fee_amount,
            ctx.accounts.base_value_registry.value_usd_micros,
            &ctx.accounts.token_program,
            signer_seeds,
            &ctx.remaining_accounts,
        )?;

        // 4) out
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

        emit!(SwapExecuted {
            pool: pool.key(),
            user: ctx.accounts.user.key(),
            input_mint: pool.base_mint,
            output_mint: pool.quote_mint,
            amount_in,
            amount_out: out,
            fee_amount,
            base_reserve_after: ctx.accounts.base_vault.amount,
            quote_reserve_after: ctx.accounts.quote_vault.amount,
            ts: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn swap_quote_for_base<'info>(
        ctx: Context<'_, '_, '_, 'info, SwapQuoteForBase<'info>>,
        amount_in: u64,
        min_out: u64,
    ) -> Result<()> {
        require!(amount_in > 0, SterlingError::InvalidAmount);

        let pool = &mut ctx.accounts.pool;
        require!(pool.active, SterlingError::InactivePool);

        require!(
            ctx.accounts.user_quote_ata.owner == ctx.accounts.user.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_base_ata.owner == ctx.accounts.user.key(),
            SterlingError::InvalidAccount
        );

        require!(
            ctx.accounts.user_quote_ata.mint == pool.quote_mint,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_base_ata.mint == pool.base_mint,
            SterlingError::InvalidAccount
        );

        require!(
            ctx.accounts.quote_vault.key() == pool.quote_vault,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.base_vault.key() == pool.base_vault,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.fee_vault_quote.key() == pool.fee_vault_quote,
            SterlingError::InvalidAccount
        );

        let fee_bps = pool.fee_bps as u128;
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

        // 1) input
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

        // 2) fee -> fee_vault_quote
        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[*ctx.bumps.get("config").unwrap()]]];

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

        // 3) threshold event (USD)
        let fee_usd_micros = ((ctx.accounts.quote_value_registry.value_usd_micros as u128)
            .saturating_mul(ctx.accounts.fee_vault_quote.amount as u128)
            .checked_div(USD_MICROS as u128)
            .ok_or(SterlingError::MathOverflow)?) as u64;

        if fee_usd_micros >= configured_fee_threshold_usd_micros_compat(&ctx.accounts.config)? {
            emit!(FeeThresholdEvent {
                pool: pool.key(),
                fee_vault: ctx.accounts.fee_vault_quote.key(),
                fee_amount: ctx.accounts.fee_vault_quote.amount,
                fee_value_usd_micros: fee_usd_micros,
                ts: Clock::get()?.unix_timestamp,
            });
        }

        // AutoCollectReadyEvent (tous les N swaps)
        pool.swap_count = pool.swap_count.saturating_add(1);
        let n = configured_auto_collect_every_swaps_compat(&ctx.accounts.config)?;
        if n > 0 && (pool.swap_count % n == 0) {
            emit!(AutoCollectReadyEvent {
                pool: pool.key(),
                side: 1, // quote
                fee_vault: ctx.accounts.fee_vault_quote.key(),
                fee_amount: ctx.accounts.fee_vault_quote.amount,
                ts: Clock::get()?.unix_timestamp,
            });
        }

        maybe_auto_settle_fee_to_usdc(
            &ctx.accounts.config,
            pool.key(),
            &ctx.accounts.fee_vault_quote,
            pool.quote_mint,
            fee_amount,
            ctx.accounts.quote_value_registry.value_usd_micros,
            &ctx.accounts.token_program,
            signer_seeds,
            &ctx.remaining_accounts,
        )?;

        // 4) out
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

        emit!(SwapExecuted {
            pool: pool.key(),
            user: ctx.accounts.user.key(),
            input_mint: pool.quote_mint,
            output_mint: pool.base_mint,
            amount_in,
            amount_out: out,
            fee_amount,
            base_reserve_after: ctx.accounts.base_vault.amount,
            quote_reserve_after: ctx.accounts.quote_vault.amount,
            ts: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    // =========================
    // D2) FEES : collect stable vers wallet C
    // =========================
    pub fn collect_fees_stable_to_treasury(
        ctx: Context<CollectFeesStableToTreasury>,
        side: FeeSide,
    ) -> Result<()> {
        require_admin_compat(&ctx.accounts.config, &ctx.accounts.admin)?;

        let (fee_vault, expected_mint) = match side {
            FeeSide::Base => (&ctx.accounts.fee_vault_base, ctx.accounts.pool.base_mint),
            FeeSide::Quote => (&ctx.accounts.fee_vault_quote, ctx.accounts.pool.quote_mint),
        };
        require!(
            fee_vault.mint == expected_mint,
            SterlingError::InvalidAccount
        );
        let usdc = pubkey_from_str(USDC_MINT);
        let usdt = pubkey_from_str(USDT_MAIN_MINT);

        // ATA de trésorerie (USDC / USDT)
        let treasury_ata = if fee_vault.mint == usdc {
            require!(
                ctx.accounts.treasury_usdc_ata.key() == configured_treasury_usdc_ata_compat(&ctx.accounts.config)?,
                SterlingError::InvalidAccount
            );
            &ctx.accounts.treasury_usdc_ata
        } else if fee_vault.mint == usdt {
            require!(
                ctx.accounts.treasury_usdt_ata.key() == configured_treasury_usdt_ata_compat(&ctx.accounts.config)?,
                SterlingError::InvalidAccount
            );
            &ctx.accounts.treasury_usdt_ata
        } else {
            return err!(SterlingError::UnsupportedMint);
        };

        let amount = fee_vault.amount;
        require!(amount > 0, SterlingError::InvalidAmount);
        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[*ctx.bumps.get("config").unwrap()]]];

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: fee_vault.to_account_info(),
                    to: treasury_ata.to_account_info(),
                    authority: ctx.accounts.config.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            amount,
        )?;

        Ok(())
    }
    pub fn convert_fees_to_usdc(
        ctx: Context<ConvertFeesToUsdc>,
        side: FeeSide,
        burn_amount: u64,
        bank_metadata: String,
    ) -> Result<()> {
        require_admin_compat(&ctx.accounts.config, &ctx.accounts.admin)?;
        require!(burn_amount > 0, SterlingError::InvalidAmount);

        let (fee_vault, expected_mint) = match side {
            FeeSide::Base => (&ctx.accounts.fee_vault_base, ctx.accounts.pool.base_mint),
            FeeSide::Quote => (&ctx.accounts.fee_vault_quote, ctx.accounts.pool.quote_mint),
        };

        require!(
            fee_vault.mint == expected_mint,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.fee_mint.key() == expected_mint,
            SterlingError::InvalidAccount
        );

        let usdc_mint = pubkey_from_str(USDC_MINT);
        require!(
            ctx.accounts.usdc_coffre_ata.key() == configured_usdc_coffre_compat(&ctx.accounts.config)?,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.usdc_coffre_ata.mint == usdc_mint,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.treasury_usdc_ata.key() == configured_treasury_usdc_ata_compat(&ctx.accounts.config)?,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.treasury_usdc_ata.mint == usdc_mint,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.asset_registry.mint == expected_mint,
            SterlingError::BadRegistry
        );

        let denom: u128 = 10u128
            .checked_pow(ctx.accounts.fee_mint.decimals as u32)
            .ok_or(SterlingError::MathOverflow)?;

        let usdc_out: u64 = (burn_amount as u128)
            .checked_mul(ctx.accounts.asset_registry.valuation_usd_micros as u128)
            .ok_or(SterlingError::MathOverflow)?
            .checked_div(denom)
            .ok_or(SterlingError::MathOverflow)? as u64;

        require!(usdc_out > 0, SterlingError::InvalidAmount);
        require!(
            fee_vault.amount >= burn_amount,
            SterlingError::InvalidAmount
        );
        require!(
            ctx.accounts.usdc_coffre_ata.amount >= usdc_out,
            SterlingError::InsufficientLiquidity
        );

        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[*ctx.bumps.get("config").unwrap()]]];

        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.fee_mint.to_account_info(),
                    from: fee_vault.to_account_info(),
                    authority: ctx.accounts.config.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            burn_amount,
        )?;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.usdc_coffre_ata.to_account_info(),
                    to: ctx.accounts.treasury_usdc_ata.to_account_info(),
                    authority: ctx.accounts.config.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            usdc_out,
        )?;

        emit!(FeeConvertedToUsdcEvent {
            pool: ctx.accounts.pool.key(),
            burned_mint: expected_mint,
            burned_amount: burn_amount,
            usdc_released: usdc_out,
            usdc_coffre: ctx.accounts.usdc_coffre_ata.key(),
            treasury_usdc_ata: ctx.accounts.treasury_usdc_ata.key(),
            bank_info: bank_metadata,
            ts: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn reserve_authority_rebind(ctx: Context<ReserveAuthorityRebind>) -> Result<()> {
        require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        require!(
            ctx.accounts.authority_mint.key() == pubkey_from_str(USDC_MINT),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.current_authority_bridge_vault.key() == ctx.accounts.config.usdc_coffre,
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.current_authority_bridge_vault.owner == ctx.accounts.config.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.reserve_token_account.owner
                == ctx.accounts.current_authority_bridge_vault.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.reserve_token_account.key()
                != ctx.accounts.current_authority_bridge_vault.key(),
            SterlingError::InvalidAccount
        );

        let authority_mint_key = ctx.accounts.authority_mint.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"bridge_vault",
            authority_mint_key.as_ref(),
            &[*ctx.bumps.get("current_authority_bridge_vault").unwrap()],
        ]];

        token::set_authority(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                SetAuthority {
                    account_or_mint: ctx.accounts.reserve_token_account.to_account_info(),
                    current_authority: ctx
                        .accounts
                        .current_authority_bridge_vault
                        .to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            AuthorityType::AccountOwner,
            Some(ctx.accounts.new_authority.key()),
        )?;

        Ok(())
    }

    pub fn register_asset(
        ctx: Context<RegisterAsset>,
        valuation_usd_micros: u64,
        is_lp: bool,
    ) -> Result<()> {
        require_admin(&ctx.accounts.config, &ctx.accounts.admin)?;
        require!(valuation_usd_micros > 0, SterlingError::InvalidAmount);

        let asset_reg = &mut ctx.accounts.asset_registry;
        asset_reg.mint = ctx.accounts.token_mint.key();
        asset_reg.valuation_usd_micros = valuation_usd_micros;
        asset_reg.is_lp = is_lp;
        asset_reg.active = true;
        asset_reg.bump = *ctx.bumps.get("asset_registry").unwrap();

        Ok(())
    }

    pub fn sovereign_convert_output_v1(
        ctx: Context<SovereignConvertOutputV1>,
        amount_in: u64,
        min_amount_out: u64,
        route_hint: u8,
        settlement_ref: String,
    ) -> Result<()> {
        require!(amount_in > 0, SterlingError::InvalidAmount);
        require!(
            ctx.accounts.source_asset_registry.mint == ctx.accounts.source_mint.key(),
            SterlingError::BadRegistry
        );
        require!(ctx.accounts.source_asset_registry.active, SterlingError::AssetDisabled);
        require!(
            ctx.accounts.user_source_ata.mint == ctx.accounts.source_mint.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.user_source_ata.owner == ctx.accounts.user.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.output_vault_ata.mint == ctx.accounts.output_mint.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.destination_output_ata.mint == ctx.accounts.output_mint.key(),
            SterlingError::InvalidAccount
        );
        require!(
            ctx.accounts.output_vault_ata.owner == ctx.accounts.config.key(),
            SterlingError::InvalidAccount
        );

        let expected_output_vault = configured_output_vault_compat(
            &ctx.accounts.config.to_account_info(),
            &ctx.accounts.output_mint.key(),
        )?;
        require!(
            ctx.accounts.output_vault_ata.key() == expected_output_vault,
            SterlingError::InvalidAccount
        );

        let output_value_usd_micros =
            configured_output_value_usd_micros(&ctx.accounts.output_mint.key())?;
        let amount_out = quote_output_amount_raw(
            amount_in,
            ctx.accounts.source_asset_registry.valuation_usd_micros,
            ctx.accounts.source_mint.decimals,
            output_value_usd_micros,
            ctx.accounts.output_mint.decimals,
        )?;

        require!(amount_out >= min_amount_out, SterlingError::SlippageExceeded);
        require!(
            ctx.accounts.output_vault_ata.amount >= amount_out,
            SterlingError::InsufficientLiquidity
        );

        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.source_mint.to_account_info(),
                    from: ctx.accounts.user_source_ata.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_in,
        )?;

        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[*ctx.bumps.get("config").unwrap()]]];

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.output_vault_ata.to_account_info(),
                    to: ctx.accounts.destination_output_ata.to_account_info(),
                    authority: ctx.accounts.config.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            amount_out,
        )?;

        emit!(SovereignConvertOutputEvent {
            source_mint: ctx.accounts.source_mint.key(),
            output_mint: ctx.accounts.output_mint.key(),
            amount_in,
            amount_out,
            output_vault: ctx.accounts.output_vault_ata.key(),
            final_destination: ctx.accounts.destination_output_ata.key(),
            route_hint,
            settlement_ref,
            ts: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn sovereign_redeem_to_usdc(
        ctx: Context<SovereignRedeemToUsdc>,
        amount_in: u64,
        bank_metadata: String,
    ) -> Result<()> {
        require!(amount_in > 0, SterlingError::InvalidAmount);
        require!(
            ctx.accounts.asset_registry.mint == ctx.accounts.token_mint.key(),
            SterlingError::BadRegistry
        );

        let usdc_out = quote_output_amount_raw(
            amount_in,
            ctx.accounts.asset_registry.valuation_usd_micros,
            ctx.accounts.token_mint.decimals,
            USD_MICROS,
            6,
        )?;

        require!(usdc_out > 0, SterlingError::InvalidAmount);

        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.token_mint.to_account_info(),
                    from: ctx.accounts.user_token_ata.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_in,
        )?;

        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[*ctx.bumps.get("config").unwrap()]]];

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.usdc_coffre_ata.to_account_info(),
                    to: ctx.accounts.destination_usdc_ata.to_account_info(),
                    authority: ctx.accounts.config.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            usdc_out,
        )?;

        emit!(SovereignSettlementEvent {
            token_mint: ctx.accounts.token_mint.key(),
            amount_burned: amount_in,
            cash_released: usdc_out,
            bank_info: bank_metadata,
            ts: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}
// =========================
// ACCOUNTS (Contexts)
// =========================
#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(
        init,
        payer = admin,
        seeds = [b"config"],
        bump,
        // Large pour les ajouts (config + treasury + auto-collect)
        space = 8 + 768
    )]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AdminOnly<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    pub admin: Signer<'info>,
}
// ---------- ValueRegistry ----------
#[derive(Accounts)]
pub struct InitValueRegistry<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = admin,
        seeds = [b"value_registry", mint.key().as_ref()],
        bump,
        space = 8 + 80
    )]
    pub value_registry: Account<'info, ValueRegistry>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetTokenValue<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    pub admin: Signer<'info>,
    pub mint: Account<'info, Mint>,
    #[account(mut, seeds = [b"value_registry", mint.key().as_ref()], bump)]
    pub value_registry: Account<'info, ValueRegistry>,
}

// ---------- Stake / Reward vaults ----------
#[derive(Accounts)]
pub struct InitStakeVault<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = admin,
        seeds = [b"stake_vault", mint.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = config
    )]
    pub stake_vault: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitRewardVault<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = admin,
        seeds = [b"reward_vault", mint.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = config
    )]
    pub reward_vault: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub stake_mint: Account<'info, Mint>,
    pub payout_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user_stake_ata: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"stake_vault", stake_mint.key().as_ref()], bump)]
    pub stake_vault: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = user,
        seeds = [b"stake_pos", user.key().as_ref(), stake_mint.key().as_ref(), payout_mint.key().as_ref()],
        bump,
        space = 8 + 128
    )]
    pub position: Account<'info, StakePosition>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}
#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub user: Signer<'info>,

    pub stake_mint: Account<'info, Mint>,
    pub payout_mint: Account<'info, Mint>,

    #[account(mut, seeds = [b"stake_pos", user.key().as_ref(), stake_mint.key().as_ref(), payout_mint.key().as_ref()], bump)]
    pub position: Account<'info, StakePosition>,

    #[account(mut, seeds = [b"reward_vault", payout_mint.key().as_ref()], bump)]
    pub reward_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_payout_ata: Account<'info, TokenAccount>,

    pub usdt_main_mint: Account<'info, Mint>,
    #[account(mut, seeds = [b"reward_vault", usdt_main_mint.key().as_ref()], bump)]
    pub usdt_main_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_usdt_main_ata: Account<'info, TokenAccount>,

    pub usdt_old_mint: Account<'info, Mint>,
    #[account(mut, seeds = [b"reward_vault", usdt_old_mint.key().as_ref()], bump)]
    pub usdt_old_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_usdt_old_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub stake_mint: Account<'info, Mint>,
    pub payout_mint: Account<'info, Mint>,
    #[account(mut, seeds = [b"stake_pos", user.key().as_ref(), stake_mint.key().as_ref(), payout_mint.key().as_ref()], bump)]
    pub position: Account<'info, StakePosition>,
    #[account(mut)]
    pub user_stake_ata: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"stake_vault", stake_mint.key().as_ref()], bump)]
    pub stake_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

// ---------- DEX Contexts ----------
#[derive(Accounts)]
pub struct CreatePool<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub admin: Signer<'info>,

    pub base_mint: Account<'info, Mint>,
    pub quote_mint: Account<'info, Mint>,

    #[account(seeds = [b"value_registry", base_mint.key().as_ref()], bump)]
    pub base_value_registry: Account<'info, ValueRegistry>,
    #[account(seeds = [b"value_registry", quote_mint.key().as_ref()], bump)]
    pub quote_value_registry: Account<'info, ValueRegistry>,

    #[account(
        init,
        payer = admin,
        seeds = [b"pool", base_mint.key().as_ref(), quote_mint.key().as_ref()],
        bump,
        space = 8 + 400
    )]
    pub pool: Account<'info, Pool>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitPoolBaseVault<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Account<'info, Pool>,
    pub base_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = admin,
        seeds = [b"pool_vault", pool.key().as_ref(), b"base"],
        bump,
        token::mint = base_mint,
        token::authority = config
    )]
    pub base_vault: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitPoolQuoteVault<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Account<'info, Pool>,
    pub quote_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = admin,
        seeds = [b"pool_vault", pool.key().as_ref(), b"quote"],
        bump,
        token::mint = quote_mint,
        token::authority = config
    )]
    pub quote_vault: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitPoolLpMint<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Account<'info, Pool>,
    #[account(
        init,
        payer = admin,
        seeds = [b"lp_mint", pool.key().as_ref()],
        bump,
        mint::decimals = 9,
        mint::authority = config
    )]
    pub lp_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitPoolFeeVaultBase<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Account<'info, Pool>,
    pub base_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = admin,
        seeds = [b"fee_vault", pool.key().as_ref(), b"base"],
        bump,
        token::mint = base_mint,
        token::authority = config
    )]
    pub fee_vault_base: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitPoolFeeVaultQuote<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Account<'info, Pool>,
    pub quote_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = admin,
        seeds = [b"fee_vault", pool.key().as_ref(), b"quote"],
        bump,
        token::mint = quote_mint,
        token::authority = config
    )]
    pub fee_vault_quote: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user_base_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_quote_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_lp_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub base_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub quote_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user_base_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_quote_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_lp_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub base_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub quote_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SwapBaseForQuote<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: UncheckedAccount<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user_base_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_quote_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub base_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub quote_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub fee_vault_base: Account<'info, TokenAccount>,
    #[account(seeds = [b"value_registry", pool.base_mint.as_ref()], bump)]
    pub base_value_registry: Account<'info, ValueRegistry>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SwapQuoteForBase<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: UncheckedAccount<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user_quote_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_base_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub quote_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub base_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub fee_vault_quote: Account<'info, TokenAccount>,
    #[account(seeds = [b"value_registry", pool.quote_mint.as_ref()], bump)]
    pub quote_value_registry: Account<'info, ValueRegistry>,
    pub token_program: Program<'info, Token>,
}

// ---------- FEES collect stable ----------
#[derive(Accounts)]
pub struct CollectFeesStableToTreasury<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: UncheckedAccount<'info>,
    pub admin: Signer<'info>,

    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Account<'info, Pool>,

    #[account(mut)]
    pub fee_vault_base: Account<'info, TokenAccount>,
    #[account(mut)]
    pub fee_vault_quote: Account<'info, TokenAccount>,

    // Treasury ATAs (wallet C)
    #[account(mut)]
    pub treasury_usdc_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub treasury_usdt_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

// ---------- FEES convert to USDC ----------
#[derive(Accounts)]
pub struct ConvertFeesToUsdc<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: UncheckedAccount<'info>,
    pub admin: Signer<'info>,

    #[account(mut, seeds = [b"pool", pool.base_mint.as_ref(), pool.quote_mint.as_ref()], bump)]
    pub pool: Account<'info, Pool>,

    #[account(mut)]
    pub fee_vault_base: Account<'info, TokenAccount>,
    #[account(mut)]
    pub fee_vault_quote: Account<'info, TokenAccount>,

    pub fee_mint: Account<'info, Mint>,

    #[account(seeds = [b"asset", fee_mint.key().as_ref()], bump)]
    pub asset_registry: Account<'info, AssetRegistry>,

    #[account(mut)]
    pub usdc_coffre_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub treasury_usdc_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ReserveAuthorityRebind<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    pub admin: Signer<'info>,

    pub authority_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"bridge_vault", authority_mint.key().as_ref()],
        bump,
        token::mint = authority_mint,
        token::authority = config
    )]
    pub current_authority_bridge_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub reserve_token_account: Account<'info, TokenAccount>,

    /// CHECK: target authority can be a wallet or a PDA controlled by the upgraded program.
    pub new_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

// ---------- Asset registry + Redeem ----------
#[derive(Accounts)]
pub struct RegisterAsset<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub token_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = admin,
        seeds = [b"asset", token_mint.key().as_ref()],
        bump,
        space = 8 + 64
    )]
    pub asset_registry: Account<'info, AssetRegistry>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SovereignConvertOutputV1<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    pub user: Signer<'info>,

    pub source_mint: Account<'info, Mint>,
    #[account(seeds = [b"asset", source_mint.key().as_ref()], bump)]
    pub source_asset_registry: Account<'info, AssetRegistry>,

    #[account(mut)]
    pub user_source_ata: Account<'info, TokenAccount>,

    pub output_mint: Account<'info, Mint>,

    #[account(mut)]
    pub output_vault_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub destination_output_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SovereignRedeemToUsdc<'info> {
    #[account(mut, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    pub user: Signer<'info>,

    pub token_mint: Account<'info, Mint>,
    #[account(seeds = [b"asset", token_mint.key().as_ref()], bump)]
    pub asset_registry: Account<'info, AssetRegistry>,

    #[account(mut)]
    pub user_token_ata: Account<'info, TokenAccount>,

    // doit être USDC mint + authority = config PDA
    #[account(mut)]
    pub usdc_coffre_ata: Account<'info, TokenAccount>,

    // ton ATA USDC (wallet C ou user)
    #[account(mut)]
    pub destination_usdc_ata: Account<'info, TokenAccount>,

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

    pub token_value_usd_micros_default: u64,
    pub treasury_value_usd_micros: u64,

    pub enable_treasury: bool,
    pub enable_sjbc: bool,
    pub enable_sjbc2: bool,
    pub enable_sjbc3: bool,
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

    // Coffres (token accounts SPL)
    pub usdc_coffre: Pubkey,
    pub usdt_coffre: Pubkey,

    pub pda_gt: Pubkey,
    pub coffre_7q: Pubkey,

    pub pool_id: Pubkey,

    // Treasury destinations (wallet C)
    pub treasury_usdc_ata: Pubkey,
    pub treasury_usdt_ata: Pubkey,

    // Auto-collect cadence (N swaps). 0 = disable
    pub auto_collect_every_swaps: u64,

    pub fee_threshold_usd_micros: u64,

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

    // fees séparés (base/quote)
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

    // compteur swaps (pour AutoCollectReadyEvent)
    pub swap_count: u64,

    pub bump: u8,
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
    pub is_lp: bool,
    pub active: bool,
    pub bump: u8,
}

// =========================
// EVENTS
// =========================
#[event]
pub struct FeeThresholdEvent {
    pub pool: Pubkey,
    pub fee_vault: Pubkey,
    pub fee_amount: u64,
    pub fee_value_usd_micros: u64,
    pub ts: i64,
}

#[event]
pub struct AutoCollectReadyEvent {
    pub pool: Pubkey,
    pub side: u8, // 0 = base, 1 = quote
    pub fee_vault: Pubkey,
    pub fee_amount: u64,
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
    pub bank_info: String,
    pub ts: i64,
}

#[event]
pub struct SovereignSettlementEvent {
    pub token_mint: Pubkey,
    pub amount_burned: u64,
    pub cash_released: u64,
    pub bank_info: String,
    pub ts: i64,
}

#[event]
pub struct SovereignConvertOutputEvent {
    pub source_mint: Pubkey,
    pub output_mint: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub output_vault: Pubkey,
    pub final_destination: Pubkey,
    pub route_hint: u8,
    pub settlement_ref: String,
    pub ts: i64,
}

#[event]
pub struct PoolCreated {
    pub pool: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub lp_mint: Pubkey,
    pub fee_bps: u16,
    pub ts: i64,
}

#[event]
pub struct SwapExecuted {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee_amount: u64,
    pub base_reserve_after: u64,
    pub quote_reserve_after: u64,
    pub ts: i64,
}

// =========================
// ERRORS
// =========================
#[error_code]
pub enum SterlingError {
    #[msg("Unsupported mint")]
    UnsupportedMint,
    #[msg("Mint disabled by config")]
    MintDisabled,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Too early to claim")]
    TooEarlyClaim,
    #[msg("Insufficient rewards in vault")]
    InsufficientRewards,
    #[msg("Zero reward")]
    ZeroReward,
    #[msg("Zero LP")]
    ZeroLp,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid bps")]
    InvalidBps,
    #[msg("Invalid interval")]
    InvalidInterval,
    #[msg("Inactive pool")]
    InactivePool,
    #[msg("Insufficient liquidity")]
    InsufficientLiquidity,
    #[msg("Slippage exceeded")]
    SlippageExceeded,
    #[msg("Bad value registry")]
    BadRegistry,
    #[msg("Cash flags required")]
    CashFlagsRequired,
    #[msg("Invalid account")]
    InvalidAccount,
    #[msg("Asset disabled")]
    AssetDisabled,
}

// =========================
// HELPERS
// =========================
fn require_admin<'info>(cfg: &Config, admin: &Signer<'info>) -> Result<()> {
    require!(cfg.admin == admin.key(), SterlingError::Unauthorized);
    Ok(())
}

fn require_admin_compat<'info>(cfg: &AccountInfo<'info>, admin: &Signer<'info>) -> Result<()> {
    let expected = config_pubkey_field(cfg, 0, cfg.key())?;
    require!(expected == admin.key(), SterlingError::Unauthorized);
    Ok(())
}

fn config_pubkey_field(ai: &AccountInfo, large_offset: usize, compact_fallback: Pubkey) -> Result<Pubkey> {
    let d = ai.try_borrow_data()?;
    require!(d.len() >= 8 + 32, SterlingError::InvalidAccount);
    let body = &d[8..];
    if body.len() < 600 {
        return Ok(compact_fallback);
    }
    let end = large_offset.saturating_add(32);
    require!(body.len() >= end, SterlingError::InvalidAccount);
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&body[large_offset..end]);
    Ok(Pubkey::new_from_array(bytes))
}

fn configured_usdc_coffre_compat(ai: &AccountInfo) -> Result<Pubkey> {
    config_pubkey_field(ai, 390, pubkey_from_str(USDC_COFFRE))
}

fn configured_usdt_coffre_compat(ai: &AccountInfo) -> Result<Pubkey> {
    config_pubkey_field(ai, 422, pubkey_from_str(USDT_COFFRE))
}

fn configured_treasury_usdc_ata_compat(ai: &AccountInfo) -> Result<Pubkey> {
    config_pubkey_field(ai, 550, pubkey_from_str(TREASURY_USDC_ATA))
}

fn configured_treasury_usdt_ata_compat(ai: &AccountInfo) -> Result<Pubkey> {
    config_pubkey_field(ai, 582, pubkey_from_str(TREASURY_USDT_ATA))
}

fn configured_output_vault_compat(ai: &AccountInfo, output_mint: &Pubkey) -> Result<Pubkey> {
    if *output_mint == pubkey_from_str(USDC_MINT) {
        return configured_usdc_coffre_compat(ai);
    }
    if is_usdt_mint(output_mint) {
        return configured_usdt_coffre_compat(ai);
    }
    err!(SterlingError::UnsupportedMint)
}

fn configured_output_value_usd_micros(output_mint: &Pubkey) -> Result<u64> {
    if *output_mint == pubkey_from_str(USDC_MINT) || is_usdt_mint(output_mint) {
        return Ok(USD_MICROS);
    }
    err!(SterlingError::UnsupportedMint)
}

fn quote_output_amount_raw(
    amount_in: u64,
    source_value_usd_micros: u64,
    source_decimals: u8,
    output_value_usd_micros: u64,
    output_decimals: u8,
) -> Result<u64> {
    require!(output_value_usd_micros > 0, SterlingError::InvalidAmount);

    let source_scale: u128 = 10u128
        .checked_pow(source_decimals as u32)
        .ok_or(SterlingError::MathOverflow)?;
    let output_scale: u128 = 10u128
        .checked_pow(output_decimals as u32)
        .ok_or(SterlingError::MathOverflow)?;

    let source_value_total: u128 = (amount_in as u128)
        .checked_mul(source_value_usd_micros as u128)
        .ok_or(SterlingError::MathOverflow)?
        .checked_div(source_scale)
        .ok_or(SterlingError::MathOverflow)?;

    let amount_out: u128 = source_value_total
        .checked_mul(output_scale)
        .ok_or(SterlingError::MathOverflow)?
        .checked_div(output_value_usd_micros as u128)
        .ok_or(SterlingError::MathOverflow)?;

    require!(amount_out > 0, SterlingError::InvalidAmount);
    require!(amount_out <= u64::MAX as u128, SterlingError::MathOverflow);
    Ok(amount_out as u64)
}

fn config_u64_field(ai: &AccountInfo, large_offset: usize, compact_fallback: u64) -> Result<u64> {
    let d = ai.try_borrow_data()?;
    require!(d.len() >= 8, SterlingError::InvalidAccount);
    let body = &d[8..];
    if body.len() < 600 {
        return Ok(compact_fallback);
    }
    let end = large_offset.saturating_add(8);
    require!(body.len() >= end, SterlingError::InvalidAccount);
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&body[large_offset..end]);
    Ok(u64::from_le_bytes(bytes))
}

fn configured_auto_collect_every_swaps_compat(ai: &AccountInfo) -> Result<u64> {
    config_u64_field(ai, 582, 10)
}

fn configured_fee_threshold_usd_micros_compat(ai: &AccountInfo) -> Result<u64> {
    config_u64_field(ai, 590, DEFAULT_FEE_THRESHOLD_USD_MICROS)
}

fn maybe_auto_settle_fee_to_usdc<'info>(
    config: &UncheckedAccount<'info>,
    pool: Pubkey,
    fee_vault: &Account<'info, TokenAccount>,
    fee_mint_key: Pubkey,
    fee_amount: u64,
    fee_value_usd_micros: u64,
    token_program: &Program<'info, Token>,
    signer_seeds: &[&[&[u8]]],
    remaining_accounts: &[AccountInfo<'info>],
) -> Result<()> {
    if fee_amount == 0 || remaining_accounts.is_empty() {
        return Ok(());
    }

    let treasury_usdc_ata: Account<'info, TokenAccount> = Account::try_from(&remaining_accounts[0])?;
    let usdc_mint = pubkey_from_str(USDC_MINT);
    require!(
        treasury_usdc_ata.key() == configured_treasury_usdc_ata_compat(&config.to_account_info())?,
        SterlingError::InvalidAccount
    );
    require!(treasury_usdc_ata.mint == usdc_mint, SterlingError::InvalidAccount);

    if fee_mint_key == usdc_mint {
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
            pool,
            fee_vault: fee_vault.key(),
            mint: fee_mint_key,
            amount: fee_amount,
            treasury_ata: treasury_usdc_ata.key(),
            ts: Clock::get()?.unix_timestamp,
        });
        return Ok(());
    }

    if remaining_accounts.len() < 3 {
        return Ok(());
    }

    let usdc_coffre_ata: Account<'info, TokenAccount> = Account::try_from(&remaining_accounts[1])?;
    let fee_mint: Account<'info, Mint> = Account::try_from(&remaining_accounts[2])?;
    require!(fee_mint.key() == fee_mint_key, SterlingError::InvalidAccount);
    require!(
        usdc_coffre_ata.key() == configured_usdc_coffre_compat(&config.to_account_info())?,
        SterlingError::InvalidAccount
    );
    require!(usdc_coffre_ata.mint == usdc_mint, SterlingError::InvalidAccount);
    require!(fee_vault.amount >= fee_amount, SterlingError::InvalidAmount);

    let denom: u128 = 10u128
        .checked_pow(fee_mint.decimals as u32)
        .ok_or(SterlingError::MathOverflow)?;
    let usdc_out: u64 = (fee_amount as u128)
        .checked_mul(fee_value_usd_micros as u128)
        .ok_or(SterlingError::MathOverflow)?
        .checked_div(denom)
        .ok_or(SterlingError::MathOverflow)? as u64;

    require!(usdc_out > 0, SterlingError::InvalidAmount);
    require!(usdc_coffre_ata.amount >= usdc_out, SterlingError::InsufficientLiquidity);

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
        pool,
        burned_mint: fee_mint_key,
        burned_amount: fee_amount,
        usdc_released: usdc_out,
        usdc_coffre: usdc_coffre_ata.key(),
        treasury_usdc_ata: treasury_usdc_ata.key(),
        bank_info: "AUTO_SWAP_FEE_TO_USDC".to_string(),
        ts: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

fn pubkey_from_str(s: &str) -> Pubkey {
    Pubkey::from_str(s).expect("invalid pubkey string")
}

fn is_true_cash_mint(mint: &Pubkey) -> bool {
    let treasury_root = pubkey_from_str(TREASURY_ROOT_MINT);
    let sjbc = pubkey_from_str(SJBC_MINT);
    let sjbc2 = pubkey_from_str(SJBC2_MINT);
    let sjbc3 = pubkey_from_str(SJBC3_MINT);
    *mint == treasury_root || *mint == sjbc || *mint == sjbc2 || *mint == sjbc3
}

fn is_usdt_mint(mint: &Pubkey) -> bool {
    let usdt_main = pubkey_from_str(USDT_MAIN_MINT);
    let usdt_old = pubkey_from_str(USDT_OLD_MINT);
    *mint == usdt_main || *mint == usdt_old
}

fn is_portal_btc_mint(mint: &Pubkey) -> bool {
    let btc_portal = pubkey_from_str(BTC_PORTAL_MINT);
    *mint == btc_portal
}

fn is_supported_cash_mint(mint: &Pubkey) -> bool {
    is_true_cash_mint(mint) || is_usdt_mint(mint) || is_portal_btc_mint(mint)
}

fn is_enabled(cfg: &Config, mint: &Pubkey) -> bool {
    let treasury_root = pubkey_from_str(TREASURY_ROOT_MINT);
    let sjbc = pubkey_from_str(SJBC_MINT);
    let sjbc2 = pubkey_from_str(SJBC2_MINT);
    let sjbc3 = pubkey_from_str(SJBC3_MINT);
    let usdt_main = pubkey_from_str(USDT_MAIN_MINT);
    let usdt_old = pubkey_from_str(USDT_OLD_MINT);
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
    if *mint == usdt_main {
        return cfg.enable_usdt_main;
    }
    if *mint == usdt_old {
        return cfg.enable_usdt_old;
    }
    if *mint == btc_portal {
        return cfg.enable_btc_portal;
    }
    false
}

fn set_enabled_flag(cfg: &mut Config, mint: &Pubkey, enabled: bool) -> Result<()> {
    let treasury_root = pubkey_from_str(TREASURY_ROOT_MINT);
    let sjbc = pubkey_from_str(SJBC_MINT);
    let sjbc2 = pubkey_from_str(SJBC2_MINT);
    let sjbc3 = pubkey_from_str(SJBC3_MINT);
    let usdt_main = pubkey_from_str(USDT_MAIN_MINT);
    let usdt_old = pubkey_from_str(USDT_OLD_MINT);
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
    if *mint == usdt_main {
        cfg.enable_usdt_main = enabled;
        return Ok(());
    }
    if *mint == usdt_old {
        cfg.enable_usdt_old = enabled;
        return Ok(());
    }
    if *mint == btc_portal {
        cfg.enable_btc_portal = enabled;
        return Ok(());
    }

    err!(SterlingError::UnsupportedMint)
}

fn integer_sqrt_u128(n: u128) -> u128 {
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
