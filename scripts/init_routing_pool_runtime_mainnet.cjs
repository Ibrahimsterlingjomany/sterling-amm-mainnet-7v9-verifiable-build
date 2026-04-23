#!/usr/bin/env node

const fs = require("fs");
const os = require("os");
const path = require("path");
const anchor = require("@coral-xyz/anchor");
const { PublicKey, Connection, Keypair, SystemProgram } = require("@solana/web3.js");
const { TOKEN_PROGRAM_ID } = require("@solana/spl-token");

const PROGRAM_ID = new PublicKey("7v9sLrk92NNLLUfXLJw3o7MycZNvwsTK6kLWfWb8vcVA");
const EXPECTED_ADMIN = "CMqD45Kq5oukPvaMDhzav5RxJqZb1xME1MmV71CzCeTw";
const DEFAULT_KEYPAIR = path.join(os.homedir(), ".config/solana/sjbc-mint.json");
const DEFAULT_IDL = path.resolve(process.cwd(), "idl", "sterling_amm.json");
const TOKEN_ACCOUNT_SPACE = 165;
const OBSERVED_PROTOCOL_DEBT_LEDGER_SPACE = 2305;
const DEFAULT_POOLS = [
  "DBZA9ZYg3MAZtc2Q5vxis4gQEL1RVgCfshDeEEwLUih5",
  "9dBirT3kZvxREMNRn9E9hf5izWAkGCFkzGnTrFkKZYNe",
];

function parseArgs(argv) {
  const out = {
    rpc: process.env.SOLANA_RPC_URL || "https://api.mainnet-beta.solana.com",
    idlPath: process.env.IDL_PATH || DEFAULT_IDL,
    keypairPath: process.env.KEYPAIR_PATH || process.env.SOLANA_KEYPAIR || DEFAULT_KEYPAIR,
    execute: false,
    pools: [],
    reportPath: null,
  };

  for (let i = 2; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--execute") out.execute = true;
    else if (arg === "--rpc" && argv[i + 1]) out.rpc = argv[++i];
    else if (arg === "--idl" && argv[i + 1]) out.idlPath = argv[++i];
    else if (arg === "--keypair" && argv[i + 1]) out.keypairPath = argv[++i];
    else if (arg === "--pool" && argv[i + 1]) out.pools.push(argv[++i]);
    else if (arg === "--report" && argv[i + 1]) out.reportPath = argv[++i];
    else throw new Error(`Unknown argument: ${arg}`);
  }

  if (out.pools.length === 0) out.pools = [...DEFAULT_POOLS];
  return out;
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function readKeypair(filePath) {
  return Keypair.fromSecretKey(Uint8Array.from(readJson(filePath)));
}

function pda(seeds) {
  return PublicKey.findProgramAddressSync(seeds, PROGRAM_ID)[0];
}

function deriveConfigPda() {
  return pda([Buffer.from("config")]);
}

function derivePoolRegistryEntry(pool) {
  return pda([Buffer.from("pool_registry"), pool.toBuffer()]);
}

function deriveFeeVault(pool, side) {
  return pda([Buffer.from("fee_vault"), pool.toBuffer(), Buffer.from(side)]);
}

function deriveProtocolDebtLedger(pool) {
  return pda([Buffer.from("protocol_debt"), pool.toBuffer()]);
}

function fmtPk(value) {
  return value instanceof PublicKey ? value.toBase58() : String(value);
}

async function loadProgram(connection, admin, idlPath) {
  const idl = readJson(idlPath);
  const provider = new anchor.AnchorProvider(
    connection,
    new anchor.Wallet(admin),
    { commitment: "confirmed", preflightCommitment: "confirmed" }
  );
  return new anchor.Program(idl, PROGRAM_ID, provider);
}

async function inspectTokenAccount(connection, address) {
  const info = await connection.getParsedAccountInfo(address, "confirmed");
  const parsed = info?.value?.data?.parsed?.info;
  if (!parsed) {
    return { exists: false, address: address.toBase58() };
  }
  return {
    exists: true,
    address: address.toBase58(),
    mint: parsed.mint,
    owner: parsed.owner,
    amountRaw: parsed.tokenAmount.amount,
    amountUi: parsed.tokenAmount.uiAmountString,
    decimals: parsed.tokenAmount.decimals,
  };
}

async function inspectPool(program, connection, poolAddress) {
  const pool = new PublicKey(poolAddress);
  const poolData = await program.account.pool.fetch(pool);
  const poolRegistryEntry = derivePoolRegistryEntry(pool);
  const feeVaultBase = deriveFeeVault(pool, "base");
  const feeVaultQuote = deriveFeeVault(pool, "quote");
  const protocolDebtLedger = deriveProtocolDebtLedger(pool);
  const registryInfo = await connection.getAccountInfo(poolRegistryEntry, "confirmed");
  const ledgerInfo = await connection.getAccountInfo(protocolDebtLedger, "confirmed");
  const feeBaseInfo = await inspectTokenAccount(connection, feeVaultBase);
  const feeQuoteInfo = await inspectTokenAccount(connection, feeVaultQuote);

  let registryDecoded = null;
  if (registryInfo) {
    registryDecoded = await program.account.poolRegistryEntry.fetch(poolRegistryEntry);
  }

  const actionsNeeded = [];
  if (!registryInfo) actionsNeeded.push("backfillPoolRegistryEntry");
  if (!feeBaseInfo.exists) actionsNeeded.push("initPoolFeeVaultBase");
  if (!feeQuoteInfo.exists) actionsNeeded.push("initPoolFeeVaultQuote");
  if (!ledgerInfo) actionsNeeded.push("initProtocolDebtLedger");

  return {
    pool: pool.toBase58(),
    baseMint: fmtPk(poolData.baseMint),
    quoteMint: fmtPk(poolData.quoteMint),
    baseVault: fmtPk(poolData.baseVault),
    quoteVault: fmtPk(poolData.quoteVault),
    lpMint: fmtPk(poolData.lpMint),
    feeBps: String(poolData.feeBps),
    active: poolData.active === true,
    poolRegistryEntry: {
      address: poolRegistryEntry.toBase58(),
      exists: !!registryInfo,
      decoded: registryDecoded
        ? {
            pool: fmtPk(registryDecoded.pool),
            baseMint: fmtPk(registryDecoded.baseMint),
            quoteMint: fmtPk(registryDecoded.quoteMint),
            lpMint: fmtPk(registryDecoded.lpMint),
            baseVault: fmtPk(registryDecoded.baseVault),
            quoteVault: fmtPk(registryDecoded.quoteVault),
            feeVaultBase: fmtPk(registryDecoded.feeVaultBase),
            feeVaultQuote: fmtPk(registryDecoded.feeVaultQuote),
          }
        : null,
    },
    feeVaultBase: feeBaseInfo,
    feeVaultQuote: feeQuoteInfo,
    protocolDebtLedger: {
      address: protocolDebtLedger.toBase58(),
      exists: !!ledgerInfo,
      lamports: ledgerInfo ? ledgerInfo.lamports : null,
      space: ledgerInfo ? ledgerInfo.data.length : null,
    },
    actionsNeeded,
  };
}

async function executeAction(program, admin, poolReport, actionName) {
  const pool = new PublicKey(poolReport.pool);
  const poolRegistryEntry = new PublicKey(poolReport.poolRegistryEntry.address);
  const baseMint = new PublicKey(poolReport.baseMint);
  const quoteMint = new PublicKey(poolReport.quoteMint);
  const feeVaultBase = new PublicKey(poolReport.feeVaultBase.address);
  const feeVaultQuote = new PublicKey(poolReport.feeVaultQuote.address);
  const protocolDebtLedger = new PublicKey(poolReport.protocolDebtLedger.address);
  const config = deriveConfigPda();

  if (actionName === "backfillPoolRegistryEntry") {
    return program.methods.backfillPoolRegistryEntry().accounts({
      config,
      admin: admin.publicKey,
      pool,
      poolRegistryEntry,
      systemProgram: SystemProgram.programId,
    }).rpc({ commitment: "confirmed" });
  }

  if (actionName === "initPoolFeeVaultBase") {
    return program.methods.initPoolFeeVaultBase().accounts({
      config,
      admin: admin.publicKey,
      pool,
      poolRegistryEntry,
      baseMint,
      feeVaultBase,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    }).rpc({ commitment: "confirmed" });
  }

  if (actionName === "initPoolFeeVaultQuote") {
    return program.methods.initPoolFeeVaultQuote().accounts({
      config,
      admin: admin.publicKey,
      pool,
      poolRegistryEntry,
      quoteMint,
      feeVaultQuote,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    }).rpc({ commitment: "confirmed" });
  }

  if (actionName === "initProtocolDebtLedger") {
    return program.methods.initProtocolDebtLedger().accounts({
      config,
      admin: admin.publicKey,
      pool,
      protocolDebtLedger,
      systemProgram: SystemProgram.programId,
    }).rpc({ commitment: "confirmed" });
  }

  throw new Error(`Unsupported action: ${actionName}`);
}

async function main() {
  const args = parseArgs(process.argv);
  const admin = readKeypair(args.keypairPath);
  if (admin.publicKey.toBase58() !== EXPECTED_ADMIN) {
    throw new Error(
      `Wrong signer. Expected ${EXPECTED_ADMIN}, got ${admin.publicKey.toBase58()}`
    );
  }

  const connection = new Connection(args.rpc, "confirmed");
  const program = await loadProgram(connection, admin, args.idlPath);
  const feeVaultRentLamports = await connection.getMinimumBalanceForRentExemption(TOKEN_ACCOUNT_SPACE);
  const protocolDebtLedgerRentLamports =
    await connection.getMinimumBalanceForRentExemption(OBSERVED_PROTOCOL_DEBT_LEDGER_SPACE);

  const report = {
    rpc: args.rpc,
    idlPath: args.idlPath,
    admin: admin.publicKey.toBase58(),
    config: deriveConfigPda().toBase58(),
    execute: args.execute,
    rentHints: {
      feeVaultTokenAccountLamports: feeVaultRentLamports,
      feeVaultTokenAccountSol: feeVaultRentLamports / 1_000_000_000,
      protocolDebtLedgerLamports: protocolDebtLedgerRentLamports,
      protocolDebtLedgerSol: protocolDebtLedgerRentLamports / 1_000_000_000,
    },
    pools: [],
  };

  for (const poolAddress of args.pools) {
    const poolReport = await inspectPool(program, connection, poolAddress);
    poolReport.estimatedMissingRentLamports =
      (poolReport.feeVaultBase.exists ? 0 : feeVaultRentLamports) +
      (poolReport.feeVaultQuote.exists ? 0 : feeVaultRentLamports) +
      (poolReport.protocolDebtLedger.exists ? 0 : protocolDebtLedgerRentLamports);
    poolReport.estimatedMissingRentSol =
      poolReport.estimatedMissingRentLamports / 1_000_000_000;
    poolReport.signatures = [];
    poolReport.errors = [];

    if (args.execute) {
      for (const actionName of poolReport.actionsNeeded) {
        try {
          const signature = await executeAction(program, admin, poolReport, actionName);
          poolReport.signatures.push({ action: actionName, signature });
        } catch (error) {
          poolReport.errors.push({
            action: actionName,
            error: error instanceof Error ? error.message : String(error),
          });
        }
      }
      const refreshed = await inspectPool(program, connection, poolAddress);
      poolReport.postExecute = {
        actionsNeeded: refreshed.actionsNeeded,
        feeVaultBase: refreshed.feeVaultBase,
        feeVaultQuote: refreshed.feeVaultQuote,
        protocolDebtLedger: refreshed.protocolDebtLedger,
      };
    }

    report.pools.push(poolReport);
  }

  if (args.reportPath) {
    fs.writeFileSync(args.reportPath, JSON.stringify(report, null, 2));
  }

  console.log(JSON.stringify(report, null, 2));
}

main().catch((error) => {
  console.error(error instanceof Error ? error.stack || error.message : String(error));
  process.exit(1);
});
