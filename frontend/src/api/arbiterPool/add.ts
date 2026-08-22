import { apiFetch, parseApiError } from "../client";
import type { ArbiterPoolResponse } from "../types";

export async function addArbiter(arbiter: string): Promise<ArbiterPoolResponse> {
  const res = await apiFetch(`/arbiter-pool/arbiters`, {
    method: "POST",
    body: JSON.stringify({ arbiter }),
  });
  if (!res.ok) throw await parseApiError(res);
  return (await res.json()) as ArbiterPoolResponse;
}
