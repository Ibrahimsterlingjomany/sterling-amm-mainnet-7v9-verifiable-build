#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const { createRequire } = require("module");

const HOME = process.env.HOME || "";
const PROJECT_ROOT = path.resolve(__dirname, "..");
const NODE_MODULE_PATHS = [
  process.env.NODE_PATH || "",
  path.join(HOME, "Sterling_Bridge_M3_local", "node_modules"),
  path.join(HOME, "Sterling_Bridge_M3_local", "backend", "node_modules"),
  path.join(HOME, "Sterling_Bridge_M3", "node_modules"),
  path.join(HOME, "Sterling_Bridge_M3", "backend", "node_modules"),
].filter((entry) => entry && fs.existsSync(entry));

function requireResolved(moduleName) {
  const errors = [];
  for (const base of NODE_MODULE_PATHS) {
    try {
      const req = createRequire(path.join(base, "_index.js"));
      return req(moduleName);
    } catch (error) {
      errors.push(`${base}: ${String(error && error.message ? error.message : error)}`);
    }
  }
  throw new Error(`module_not_found:${moduleName}:${errors.join(" | ")}`);
}

const anchor = requireResolved("@coral-xyz/anchor");
const web3 = requireResolved("@solana/web3.js");

const PROGRAM_ID = new web3.PublicKey(
  process.env.STERLING_AMM_PROGRAM_ID || "7v9sLrk92NNLLUfXLJw3o7MycZNvwsTK6kLWfWb8vcVA"
);
const METADATA_PROGRAM_ID = new web3.PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);
const RPC_URL =
  process.env.RPC_URL ||
  process.env.SOLANA_RPC_URL ||
  "https://solana-rpc.publicnode.com";
const IDL_PATH =
  process.env.IDL_PATH ||
  path.join(PROJECT_ROOT, "target", "idl", "sterling_amm.json");
const KEYPAIR_PATH =
  process.env.ADMIN_KEYPAIR_PATH ||
  process.env.ANCHOR_WALLET ||
  path.join(HOME, ".config", "solana", "id.json");
const SIMULATE = ["1", "true", "yes", "on"].includes(
  String(process.env.SIMULATE || "").trim().toLowerCase()
);
const TARGET_PAIR_ID = String(process.env.TARGET_PAIR_ID || "").trim();

const TARGETS = [
  {
    pairId: "STM-USDC",
    pool: "DBZA9ZYg3MAZtc2Q5vxis4gQEL1RVgCfshDeEEwLUih5",
    expectedLpMint: "7DE91Pj9FnocsgXVqrHGRjzFt9zkNr3iRW9samdiyACr",
    name: "Starling Mint / USDC LP",
    symbol: "STMUSDCLP",
    uri: "https://sterlingchain.net/token-assets/usdc-stm-lp.metadata.json",
  },
  {
    pairId: "SJBC-USDC",
    pool: "9dBirT3kZvxREMNRn9E9hf5izWAkGCFkzGnTrFkKZYNe",
    expectedLpMint: "2rrGfPpcERwtoQ7UrTZp5HV3zvN5jf76WXERBHXG9oNL",
    name: "SJBC USD / USDC LP",
    symbol: "SJBCUSDCLP",
    uri: "https://sterlingchain.net/token-assets/sjbc-usdc-lp.metadata.json",
  },
  {
    pairId: "STM-SJBCUSD",
    pool: "BbvR4zUAwZF8LmVFLXNpDy3CxuYcDwd5isoh7CZFAF5G",
    expectedLpMint: "G94nkBm4ntjiEHNzTpd7GRW9J8H5rqrhW83k5RSHZrBZ",
    name: "Starling Mint / SJBC USD LP",
    symbol: "STMSJBCLP",
    uri: "https://sterlingchain.net/token-assets/stm-sjbcusd-g94-lp.metadata.json",
  },
];

function readIdl() {
  if (!fs.existsSync(IDL_PATH)) {
    throw new Error(`idl_missing:${IDL_PATH}`);
  }
  const idl = JSON.parse(fs.readFileSync(IDL_PATH, "utf8"));
  if (!idl.metadata || typeof idl.metadata !== "object") idl.metadata = {};
  idl.metadata.address = PROGRAM_ID.toBase58();
  idl.address = PROGRAM_ID.toBase58();
  return idl;
}

function readKeypair(filePath) {
  const raw = JSON.parse(fs.readFileSync(filePath, "utf8"));
  return web3.Keypair.fromSecretKey(Uint8Array.from(raw));
}

function deriveConfigPda() {
  return web3.PublicKey.findProgramAddressSync([Buffer.from("config")], PROGRAM_ID)[0];
}

function deriveMetadataPda(mint) {
  return web3.PublicKey.findProgramAddressSync(
    [Buffer.from("metadata"), METADATA_PROGRAM_ID.toBuffer(), mint.toBuffer()],
    METADATA_PROGRAM_ID
  )[0];
}

async function assertJsonUri(uri) {
  const response = await fetch(uri, { method: "GET" });
  if (!response.ok) {
    throw new Error(`uri_http_error:${uri}:${response.status}`);
  }
  const text = await response.text();
  JSON.parse(text);
}

async function sendAndConfirmBuilder(builder, connection, payer) {
  const tx = await builder.transaction();
  tx.feePayer = payer.publicKey;

  const latest = await connection.getLatestBlockhash("confirmed");
  tx.recentBlockhash = latest.blockhash;
  tx.sign(payer);

  const signature = await connection.sendRawTransaction(tx.serialize(), {
    skipPreflight: false,
    preflightCommitment: "confirmed",
  });

  const confirmation = await connection.confirmTransaction(
    {
      signature,
      blockhash: latest.blockhash,
      lastValidBlockHeight: latest.lastValidBlockHeight,
    },
    "confirmed"
  );

  if (confirmation.value && confirmation.value.err) {
    throw new Error(`tx_failed:${signature}:${JSON.stringify(confirmation.value.err)}`);
  }

  return signature;
}

async function main() {
  const idl = readIdl();
  const payer = readKeypair(KEYPAIR_PATH);
  const connection = new web3.Connection(RPC_URL, "confirmed");
  const provider = new anchor.AnchorProvider(
    connection,
    new anchor.Wallet(payer),
    { commitment: "confirmed" }
  );
  anchor.setProvider(provider);

  const program = new anchor.Program(idl, PROGRAM_ID, provider);
  const config = deriveConfigPda();
  const results = [];

  for (const target of TARGETS) {
    if (TARGET_PAIR_ID && target.pairId !== TARGET_PAIR_ID) {
      continue;
    }

    const poolPk = new web3.PublicKey(target.pool);
    const expectedLpMint = new web3.PublicKey(target.expectedLpMint);
    await assertJsonUri(target.uri);

    const poolAccount = await program.account.pool.fetch(poolPk);
    const liveLpMint = new web3.PublicKey(poolAccount.lpMint.toBase58());
    if (!liveLpMint.equals(expectedLpMint)) {
      throw new Error(
        `lp_mint_mismatch:${target.pairId}:${liveLpMint.toBase58()}:${expectedLpMint.toBase58()}`
      );
    }

    const metadataPda = deriveMetadataPda(liveLpMint);
    const before = await connection.getAccountInfo(metadataPda, "confirmed");
    const mode = before ? "update" : "create";

    const builder = program.methods
      .upsertPoolLpMetadata(target.name, target.symbol, target.uri)
      .accounts({
        config,
        admin: payer.publicKey,
        pool: poolPk,
        lpMint: liveLpMint,
        metadata: metadataPda,
        tokenMetadataProgram: METADATA_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      });

    if (SIMULATE) {
      const sim = await builder.simulate();
      results.push({
        pair_id: target.pairId,
        pool: poolPk.toBase58(),
        lp_mint: liveLpMint.toBase58(),
        metadata: metadataPda.toBase58(),
        mode,
        simulated: true,
        result: sim,
      });
      continue;
    }

    const signature = await sendAndConfirmBuilder(builder, connection, payer);
    const after = await connection.getAccountInfo(metadataPda, "confirmed");
    results.push({
      pair_id: target.pairId,
      pool: poolPk.toBase58(),
      lp_mint: liveLpMint.toBase58(),
      metadata: metadataPda.toBase58(),
      mode,
      signature,
      metadata_exists_after: !!after,
      uri: target.uri,
      name: target.name,
      symbol: target.symbol,
    });
  }

  console.log(JSON.stringify({
    ok: true,
    simulated: SIMULATE,
    rpc_url: RPC_URL,
    program_id: PROGRAM_ID.toBase58(),
    admin: payer.publicKey.toBase58(),
    config: config.toBase58(),
    results,
  }, null, 2));
}

main().catch((error) => {
  console.error(JSON.stringify({
    ok: false,
    error: String(error && error.message ? error.message : error),
  }, null, 2));
  process.exit(1);
});
