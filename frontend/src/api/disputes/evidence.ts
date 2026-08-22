import { apiFetch, parseApiError } from "../client";
import type { EvidenceResponse } from "../types";

export async function submitEvidence(jobId: string | number, content: string, contentHash: string): Promise<EvidenceResponse> {
  const res = await apiFetch(`/jobs/${encodeURIComponent(String(jobId))}/disputes/evidence`, {
    method: "POST",
    body: JSON.stringify({ content, content_hash: contentHash }),
  });
  if (!res.ok) throw await parseApiError(res);
  return (await res.json()) as EvidenceResponse;
}
