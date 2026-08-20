import { apiFetch, parseApiError } from "../client";
import type { ArbiterPoolResponse } from "../types";

export async function getArbiterPool(): Promise<ArbiterPoolResponse> {
  const res = await apiFetch(`/arbiter-pool`);
  if (!res.ok) throw await parseApiError(res);
  return (await res.json()) as ArbiterPoolResponse;
}
