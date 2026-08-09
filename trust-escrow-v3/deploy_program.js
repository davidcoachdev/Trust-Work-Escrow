const { spawnSync } = require("node:child_process");

const endpoint = process.env.ANCHOR_PROVIDER_URL || "http://127.0.0.1:8899";
if (!/^https?:\/\/127\.0\.0\.1(:\d+)?$/.test(endpoint)) {
  throw new Error(`Refusing deployment to unapproved endpoint: ${endpoint}`);
}

const result = spawnSync("anchor", ["deploy", "--provider.cluster", "localnet"], {
  stdio: "inherit",
  env: { ...process.env, ANCHOR_PROVIDER_URL: endpoint },
});

if (result.error) throw result.error;
process.exit(result.status ?? 1);
