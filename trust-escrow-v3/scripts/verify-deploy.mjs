import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { resolve } from "node:path";
import * as anchor from "@coral-xyz/anchor";
import { Connection, PublicKey } from "@solana/web3.js";

export const EXPECTED_ENDPOINT = "http://127.0.0.1:8899";
export const PROGRAM_ID = "J1c4QsjbV9bFEPrFQZZGe8GrGWFxNhtAhhrxJFK2xc1h";
const PROGRAM_TAG = 2;
const PROGRAM_DATA_TAG = 3;
const PROGRAM_DATA_HEADER_BYTES = 45;

export function sha256Hex(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

export function assertExpectedAuthority(actual, expected) {
  if (actual !== expected) throw new Error(`upgrade authority mismatch: actual=${actual} expected=${expected}`);
}

export function assertProgramBytesMatch(localBytes, onChainBytes) {
  if (onChainBytes.length < localBytes.length) {
    throw new Error(`Program byte mismatch: on-chain payload is shorter than local binary`);
  }
  for (let index = 0; index < localBytes.length; index += 1) {
    if (onChainBytes[index] !== localBytes[index]) {
      throw new Error(`Program byte mismatch at offset ${index}`);
    }
  }
  for (let index = localBytes.length; index < onChainBytes.length; index += 1) {
    if (onChainBytes[index] !== 0) {
      throw new Error(`Program byte mismatch: non-zero ProgramData padding at offset ${index}`);
    }
  }
}

function assertApprovedLocalEndpoint(endpoint) {
  const parsed = new URL(endpoint);
  if (parsed.protocol !== "http:" || parsed.hostname !== "127.0.0.1") {
    throw new Error(`Refusing non-localnet endpoint: ${endpoint}`);
  }
}

export function parseProgramDataAccount(data) {
  if (data.length < PROGRAM_DATA_HEADER_BYTES || data.readUInt32LE(0) !== PROGRAM_DATA_TAG) {
    throw new Error("Invalid upgradeable ProgramData account");
  }
  const authorityOption = data[12];
  const upgradeAuthority = authorityOption === 0 ? null : new PublicKey(data.subarray(13, 45));
  return { upgradeAuthority, programBytes: data.subarray(PROGRAM_DATA_HEADER_BYTES) };
}

function parseProgramAccount(data) {
  if (data.length < 36 || data.readUInt32LE(0) !== PROGRAM_TAG) {
    throw new Error("Invalid upgradeable Program account");
  }
  return new PublicKey(data.subarray(4, 36));
}

function requiredPublicKey(name) {
  const value = process.env[name];
  if (!value) throw new Error(`Missing ${name}; provide public identity only`);
  return new PublicKey(value);
}

async function main() {
  const endpoint = process.env.ANCHOR_PROVIDER_URL || EXPECTED_ENDPOINT;
  assertApprovedLocalEndpoint(endpoint);

  const root = resolve(process.cwd());
  const idl = JSON.parse(readFileSync(resolve(root, "target/idl/escrow.json"), "utf8"));
  const anchorToml = readFileSync(resolve(root, "Anchor.toml"), "utf8");
  if (!anchorToml.includes(`trust_escrow_v3 = \"${PROGRAM_ID}\"`)) {
    throw new Error("Anchor.toml program ID mismatch");
  }
  if (idl.address && idl.address !== PROGRAM_ID) throw new Error("IDL program ID mismatch");

  const localBinaryPath = resolve(root, "target/deploy/trust_escrow_v3.so");
  if (!existsSync(localBinaryPath)) throw new Error("Missing target/deploy/trust_escrow_v3.so; run yarn build first");
  const localBytes = readFileSync(localBinaryPath);
  const programId = new PublicKey(PROGRAM_ID);
  const connection = new Connection(endpoint, "confirmed");
  const programInfo = await connection.getAccountInfo(programId, "confirmed");
  if (!programInfo) throw new Error("Program account not found on the approved localnet endpoint");
  const programDataAddress = parseProgramAccount(programInfo.data);
  const programDataInfo = await connection.getAccountInfo(programDataAddress, "confirmed");
  if (!programDataInfo) throw new Error("ProgramData account not found");
  const { upgradeAuthority, programBytes } = parseProgramDataAccount(programDataInfo.data);
  assertProgramBytesMatch(localBytes, programBytes);
  const localHash = sha256Hex(localBytes);
  const onChainHash = sha256Hex(programBytes.subarray(0, localBytes.length));

  const expectedAuthority = requiredPublicKey("TRUST_ESCROW_V3_EXPECTED_AUTHORITY");
  assertExpectedAuthority(upgradeAuthority?.toBase58() ?? null, expectedAuthority.toBase58());
  const expectedAdvisor = requiredPublicKey("TRUST_ESCROW_V3_ADVISOR_PUBKEY");
  const expectedTreasury = requiredPublicKey("TRUST_ESCROW_V3_TREASURY_PUBKEY");
  const expectedArbitrationTreasury = requiredPublicKey("TRUST_ESCROW_V3_ARBITRATION_TREASURY_PUBKEY");
  const expectedFeeBps = Number(process.env.TRUST_ESCROW_V3_FEE_BPS || "250");
  if (!Number.isInteger(expectedFeeBps) || expectedFeeBps < 0 || expectedFeeBps > 10_000) throw new Error("Invalid expected fee bps");

  const provider = new anchor.AnchorProvider(connection, anchor.Wallet.local(), { commitment: "confirmed" });
  const program = new anchor.Program(idl, provider);
  const [configAddress] = PublicKey.findProgramAddressSync([Buffer.from("config")], programId);
  const config = await program.account.config.fetch(configAddress);
  const configMatches = config.authority.equals(expectedAuthority)
    && config.advisor.equals(expectedAdvisor)
    && config.treasury.equals(expectedTreasury)
    && config.arbitrationTreasury.equals(expectedArbitrationTreasury)
    && config.feeBps === expectedFeeBps;
  if (!configMatches) throw new Error("On-chain Config does not match the approved manifest");

  const commit = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
  console.log(JSON.stringify({
    verifiedAt: new Date().toISOString(),
    commit,
    endpoint,
    programId: PROGRAM_ID,
    programData: programDataAddress.toBase58(),
    localSha256: localHash,
    onChainSha256: onChainHash,
    upgradeAuthority: upgradeAuthority?.toBase58() ?? null,
    config: configAddress.toBase58(),
    authority: config.authority.toBase58(),
    advisor: config.advisor.toBase58(),
    treasury: config.treasury.toBase58(),
    arbitrationTreasury: config.arbitrationTreasury.toBase58(),
    feeBps: config.feeBps,
  }));
}

if (import.meta.url === `file://${process.argv[1]}`) await main();
