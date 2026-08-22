import test from "node:test";
import assert from "node:assert/strict";
import { isRetryableTransactionExpiry } from "./run-isolated-localnet.mjs";

test("solo clasifica expiraciones de transacción como reintentables", () => {
  assert.equal(
    isRetryableTransactionExpiry("TransactionExpiredTimeoutError: block height exceeded"),
    true,
  );
  assert.equal(isRetryableTransactionExpiry("Error: AlreadyApplied"), false);
  assert.equal(isRetryableTransactionExpiry("Error: InvalidApplicationIndex"), false);
});
