import test from "node:test";
import assert from "node:assert/strict";
import { Keypair } from "@solana/web3.js";
import { validateProgramKeypair } from "./preflight.mjs";

const fixture = Keypair.fromSeed(Uint8Array.from({ length: 32 }, (_, index) => index + 1));

test("rechaza un deploy keypair cuyo pubkey no coincide con el Program ID", () => {
  assert.throws(
    () => validateProgramKeypair({ keypairBytes: fixture.secretKey, expectedProgramId: Keypair.generate().publicKey.toBase58() }),
    /program ID mismatch/,
  );
});

test("rechaza un deploy keypair ausente antes de continuar", () => {
  assert.throws(
    () => validateProgramKeypair({ keypairBytes: null, expectedProgramId: fixture.publicKey.toBase58() }),
    /Missing deploy keypair/,
  );
});

test("acepta el deploy keypair cuyo pubkey coincide con el Program ID", () => {
  assert.equal(
    validateProgramKeypair({ keypairBytes: fixture.secretKey, expectedProgramId: fixture.publicKey.toBase58() }),
    fixture.publicKey.toBase58(),
  );
});
