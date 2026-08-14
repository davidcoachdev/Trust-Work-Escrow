import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

test("SDK inventory separates the 27 source-backed operations from 18 absent ones", () => {
  const inventory = JSON.parse(
    readFileSync("docs/backend/sdk-operation-inventory.json", "utf8"),
  );
  const source = readFileSync(
    "../backend/sdk/src/client.rs",
    "utf8",
  );
  const current = inventory.operations.filter(
    (operation) => operation.sdk_status === "implemented-current",
  );
  const planned = inventory.operations.filter(
    (operation) => operation.sdk_status === "planned-not-implemented",
  );

  assert.equal(current.length, 27);
  assert.equal(planned.length, 18);
  assert.equal(inventory.document_status, "partially-implemented");
  for (const operation of current) {
    assert.match(operation.sdk_evidence, /backend\/sdk\/src\/client\.rs/);
    assert.notEqual(operation.sdk_method, null);
    for (const method of operation.sdk_method.split(" + ")) {
      assert.match(source, new RegExp(`(?:fn|pub fn)\\s+${method}\\b`));
    }
    assert.equal(operation.runtime_verification, "not-verified-localnet");
  }
  for (const operation of planned) {
    assert.equal(operation.sdk_method, null);
    assert.equal(operation.runtime_verification, "not-applicable");
  }
});
