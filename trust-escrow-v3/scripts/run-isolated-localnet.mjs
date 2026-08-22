import { spawn, spawnSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { Keypair, PublicKey } from "@solana/web3.js";

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

export function isRetryableTransactionExpiry(output) {
  return /TransactionExpired(?:TimeoutError|BlockheightExceededError)|block height exceeded|blockhash.*expired/i.test(output);
}

function redact(output, env) {
  return output.replaceAll(env.TRUST_ESCROW_V3_ADVISOR_KEYPAIR, "[REDACTED]");
}

function commandExists(command) {
  return spawnSync("sh", ["-c", `command -v ${command}`], { encoding: "utf8" }).status === 0;
}

async function main() {
  const port = process.env.TRUST_ESCROW_V3_ISOLATED_PORT
    || String(18000 + ((process.pid % 500) * 2));
  const endpoint = `http://127.0.0.1:${port}`;
  const authority = new PublicKey(spawnSync("solana", ["address"], { encoding: "utf8" }).stdout.trim());
  const advisor = Keypair.generate();
  const treasury = Keypair.generate();
  const arbitrationTreasury = Keypair.generate();
  const ledger = mkdtempSync(join(tmpdir(), "trust-escrow-v3-localnet-"));
  const runtimeEnv = {
    ...process.env,
    ANCHOR_PROVIDER_URL: endpoint,
    TRUST_ESCROW_V3_ADVISOR_PUBKEY: advisor.publicKey.toBase58(),
    TRUST_ESCROW_V3_TREASURY_PUBKEY: treasury.publicKey.toBase58(),
    TRUST_ESCROW_V3_ARBITRATION_TREASURY_PUBKEY: arbitrationTreasury.publicKey.toBase58(),
    TRUST_ESCROW_V3_ADVISOR_KEYPAIR: JSON.stringify([...advisor.secretKey]),
    TRUST_ESCROW_V3_EXPECTED_AUTHORITY: authority.toBase58(),
    TRUST_ESCROW_V3_FEE_BPS: "250",
    TRUST_ESCROW_V3_RUN_NONCE: String(Math.floor(Math.random() * 1_000)),
  };
  if (!commandExists("solana-test-validator")) {
    throw new Error("solana-test-validator is required; Surfpool fallback is disabled");
  }
  const child = spawn("solana-test-validator", [
    "--ledger", ledger, "--reset", "--rpc-port", port,
    "--faucet-port", String(Number(port) + 2), "--bind-address", "127.0.0.1",
  ], { env: runtimeEnv, detached: true, stdio: ["ignore", "ignore", "pipe"] });

  const run = (command, args) => {
    const result = spawnSync(command, args, { env: runtimeEnv, stdio: "inherit" });
    if (result.status !== 0) throw new Error(`${command} failed with status ${result.status}`);
  };

  const runTestsWithExpiryRetry = async () => {
    const testArgs = ["test"];
    if (process.env.TRUST_ESCROW_V3_TEST_GREP) {
      testArgs.push("--grep", process.env.TRUST_ESCROW_V3_TEST_GREP);
    }
    const maxAttempts = Number(process.env.TRUST_ESCROW_V3_EXPIRY_RETRIES || "3");
    let lastOutput = "";
    for (let attempt = 1; attempt <= maxAttempts; attempt += 1) {
      runtimeEnv.TRUST_ESCROW_V3_RUN_NONCE = String(Math.floor(Math.random() * 1_000));
      const result = spawnSync("yarn", testArgs, { env: runtimeEnv, encoding: "utf8" });
      lastOutput = `${result.stdout || ""}${result.stderr || ""}`;
      process.stdout.write(redact(result.stdout || "", runtimeEnv));
      process.stderr.write(redact(result.stderr || "", runtimeEnv));
      if (result.status === 0) return;
      if (!isRetryableTransactionExpiry(lastOutput) || attempt === maxAttempts) {
        throw new Error(`yarn test failed after ${attempt} attempt(s)`);
      }
      process.stderr.write(`retrying transaction-expiry failure (${attempt}/${maxAttempts - 1})\n`);
      await sleep(attempt * 1_000);
    }
    throw new Error(`yarn test failed: ${lastOutput}`);
  };

  try {
    for (let attempt = 0; attempt < 60; attempt += 1) {
      const probe = spawnSync("solana", ["cluster-version", "--url", endpoint]);
      if (probe.status === 0) break;
      await sleep(250);
      if (attempt === 59) throw new Error("isolated solana-test-validator did not become ready");
    }

    run("solana", [
      "program", "deploy", "--url", endpoint,
      "--program-id", "target/deploy/trust_escrow_v3-keypair.json",
      "--upgrade-authority", process.env.ANCHOR_WALLET || `${process.env.HOME}/.config/solana/id.json`,
      "target/deploy/trust_escrow_v3.so",
    ]);
    run("solana", ["airdrop", "10", advisor.publicKey.toBase58(), "--url", endpoint]);
    run("solana", ["airdrop", "10", treasury.publicKey.toBase58(), "--url", endpoint]);
    run("solana", ["airdrop", "10", arbitrationTreasury.publicKey.toBase58(), "--url", endpoint]);
    run("node", ["scripts/bootstrap-config.mjs"]);
    run("node", ["scripts/verify-deploy.mjs"]);
    await runTestsWithExpiryRetry();
  } finally {
    try { process.kill(-child.pid, "SIGKILL"); } catch { child.kill("SIGKILL"); }
    child.stderr?.destroy();
    rmSync(ledger, { recursive: true, force: true });
  }
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  await main();
}
