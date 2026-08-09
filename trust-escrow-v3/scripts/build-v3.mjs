import { spawnSync } from "node:child_process";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

export function resolveCargoBuildSbf(environment = process.env) {
  if (environment.CARGO_BUILD_SBF) return environment.CARGO_BUILD_SBF;
  if (environment.SOLANA_BIN_DIR) return join(environment.SOLANA_BIN_DIR, "cargo-build-sbf");
  return "cargo-build-sbf";
}

function run(command, args, env = process.env) {
  const result = spawnSync(command, args, { env, stdio: "inherit" });
  if (result.status !== 0) {
    throw new Error(`${command} failed with status ${result.status}`);
  }
}

export function build() {
  const env = { ...process.env };
  const cargoBuildSbf = resolveCargoBuildSbf(env);

  run(cargoBuildSbf, [
    "--arch", "v3",
    "--sbf-out-dir", "target/deploy",
    "--manifest-path", "programs/trust-escrow-v3/Cargo.toml",
  ], env);

  run("anchor", [
    "idl", "build",
    "-p", "trust-escrow-v3",
    "-o", "target/idl/escrow.json",
    "-t", "target/types/escrow.ts",
  ], env);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) build();
