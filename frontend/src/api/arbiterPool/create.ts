import { apiFetch, parseApiError } from "../client";
import type { ArbiterPoolResponse } from "../types";

export async function createArbiterPool(): Promise<ArbiterPoolResponse> {
  const res = await apiFetch(`/arbiter-pool`, { method: "POST" });
  if (!res.ok) throw await parseApiError(res);
  return (await res.json()) as ArbiterPoolResponse;
}
