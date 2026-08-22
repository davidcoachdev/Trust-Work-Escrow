import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";
import { Keypair } from "@solana/web3.js";

export function validateProgramKeypair({ keypairBytes, expectedProgramId }) {
  if (!keypairBytes) throw new Error("Missing deploy keypair; run anchor build first");
  const derivedProgramId = Keypair.fromSecretKey(Uint8Array.from(keypairBytes)).publicKey.toBase58();
  if (derivedProgramId !== expectedProgramId) {
    throw new Error(`Deploy keypair program ID mismatch: expected ${expectedProgramId}, got ${derivedProgramId}`);
  }
  return derivedProgramId;
}

export function preflight() {
  const expectedEndpoint = "http://127.0.0.1:8899";
  const endpoint = process.env.ANCHOR_PROVIDER_URL || expectedEndpoint;
  const parsedEndpoint = new URL(endpoint);
  if (parsedEndpoint.protocol !== "http:" || parsedEndpoint.hostname !== "127.0.0.1") {
    throw new Error(`Refusing non-localnet endpoint: ${endpoint}`);
  }

  const programId = "J1c4QsjbV9bFEPrFQZZGe8GrGWFxNhtAhhrxJFK2xc1h";
  const anchor = readFileSync("Anchor.toml", "utf8");
  if (!anchor.includes(`trust_escrow_v3 = \"${programId}\"`)) {
    throw new Error("Anchor.toml program ID mismatch");
  }
  const keypairPath = "target/deploy/trust_escrow_v3-keypair.json";
  if (!existsSync(keypairPath)) {
    validateProgramKeypair({ keypairBytes: null, expectedProgramId: programId });
  }
  const keypair = JSON.parse(readFileSync(keypairPath, "utf8"));
  validateProgramKeypair({ keypairBytes: keypair, expectedProgramId: programId });

  const address = execFileSync("solana", ["address"], { encoding: "utf8" }).trim();
  if (!address) throw new Error("No localnet payer identity available");
  console.log(JSON.stringify({ cluster: "localnet", endpoint, programId, payer: address }));
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) preflight();
