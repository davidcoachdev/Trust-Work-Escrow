import { apiFetch, parseApiError } from "../client";

export interface VerifyAuthResponse {
  valid: boolean;
  pubkey?: string;
  message?: string;
}

export async function verifyAuth(pubkey: string, message?: string, signature?: string): Promise<VerifyAuthResponse> {
  const res = await apiFetch(`/auth/verify`, {
    method: "POST",
    body: JSON.stringify({ pubkey, message, signature }),
  });
  if (!res.ok) throw await parseApiError(res);
  return (await res.json()) as VerifyAuthResponse;
}
