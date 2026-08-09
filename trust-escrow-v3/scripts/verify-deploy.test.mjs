import test from "node:test";
import assert from "node:assert/strict";
import {
  assertExpectedAuthority,
  assertProgramBytesMatch,
  parseProgramDataAccount,
  sha256Hex,
} from "./verify-deploy.mjs";

test("upgrade authority must match the approved manifest", () => {
  assert.doesNotThrow(() => assertExpectedAuthority("authority", "authority"));
  assert.throws(() => assertExpectedAuthority("actual", "expected"), /upgrade authority mismatch/);
});

test("parseProgramDataAccount extracts upgrade authority and deployed bytes", () => {
  const authority = Buffer.alloc(32, 7);
  const payload = Buffer.from([1, 2, 3, 4]);
  const data = Buffer.concat([
    Buffer.from([3, 0, 0, 0]),
    Buffer.alloc(8),
    Buffer.from([1]),
    authority,
    payload,
  ]);

  const parsed = parseProgramDataAccount(data);
  assert.deepEqual(parsed.upgradeAuthority.toBuffer(), authority);
  assert.deepEqual(parsed.programBytes, payload);
});

test("sha256Hex is byte-to-byte deterministic", () => {
  assert.equal(
    sha256Hex(Buffer.from("trust-escrow-v3")),
    "ac64f4dff6a57dedf6b2d19685b6663c8844fb8371e6c444023762dddc90bfe0",
  );
});

test("program bytes accept an exact match", () => {
  const local = Buffer.from([1, 2, 3, 4]);
  assert.doesNotThrow(() => assertProgramBytesMatch(local, Buffer.from(local)));
});

test("program bytes accept zero padding after the local prefix", () => {
  const local = Buffer.from([1, 2, 3, 4]);
  const onChain = Buffer.concat([local, Buffer.alloc(8)]);
  assert.doesNotThrow(() => assertProgramBytesMatch(local, onChain));
});

test("program bytes reject an altered byte in the local prefix", () => {
  const local = Buffer.from([1, 2, 3, 4]);
  const onChain = Buffer.from([1, 2, 9, 4, 0, 0]);
  assert.throws(() => assertProgramBytesMatch(local, onChain), /Program byte mismatch/);
});
