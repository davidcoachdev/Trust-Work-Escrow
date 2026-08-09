import test from "node:test";
import assert from "node:assert/strict";
import { resolveCargoBuildSbf } from "./build-v3.mjs";

test("usa CARGO_BUILD_SBF cuando se configura explícitamente", () => {
  assert.equal(
    resolveCargoBuildSbf({ CARGO_BUILD_SBF: "/opt/solana/bin/cargo-build-sbf" }),
    "/opt/solana/bin/cargo-build-sbf",
  );
});

test("acepta SOLANA_BIN_DIR sin imponer un path de usuario", () => {
  assert.equal(
    resolveCargoBuildSbf({ SOLANA_BIN_DIR: "/opt/solana/bin" }),
    "/opt/solana/bin/cargo-build-sbf",
  );
});

test("detecta cargo-build-sbf desde PATH por defecto", () => {
  assert.equal(resolveCargoBuildSbf({}), "cargo-build-sbf");
});
