export const PROGRAM_ID = "7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh";
export const RPC_URL = process.env.NEXT_PUBLIC_RPC_URL ?? "http://127.0.0.1:8899";
export const CLUSTER = (process.env.NEXT_PUBLIC_CLUSTER ?? "localnet") as "localnet" | "devnet" | "mainnet";

export function isMainnetBlocked(cluster: string): boolean {
  return cluster === "mainnet" && process.env.NEXT_PUBLIC_ALLOW_MAINNET !== "1";
}
