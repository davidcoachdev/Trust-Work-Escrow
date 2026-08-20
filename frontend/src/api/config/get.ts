import { apiFetch, parseApiError } from "../client";
import type { ConfigResponse } from "../types";

export async function getConfig(): Promise<ConfigResponse> {
  const res = await apiFetch(`/config`);
  if (!res.ok) throw await parseApiError(res);
  return (await res.json()) as ConfigResponse;
}
