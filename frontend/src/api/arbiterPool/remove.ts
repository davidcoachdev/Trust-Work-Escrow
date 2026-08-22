import { apiFetch, parseApiError } from "../client";
import type { ArbiterPoolResponse } from "../types";

export async function removeArbiter(arbiter: string): Promise<ArbiterPoolResponse> {
  const res = await apiFetch(`/arbiter-pool/arbiters/${encodeURIComponent(arbiter)}`, { method: "DELETE" });
  if (!res.ok) throw await parseApiError(res);
  return (await res.json()) as ArbiterPoolResponse;
}
