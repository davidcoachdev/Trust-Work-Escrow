import { apiFetch, parseApiError, ApiError } from "../client";
import { validateProposalHash } from "../types";
import type { ApplicationResponse } from "../types";

export interface ApplyToJobParams {
  jobId: string | number;
  proposal: string;
  proposalHash: string;
}

export async function applyToJob(params: ApplyToJobParams): Promise<ApplicationResponse> {
  const hErr = validateProposalHash(params.proposalHash);
  if (hErr) throw new ApiError(hErr, { status: 400, code: "bad_request" });
  const res = await apiFetch(`/jobs/${encodeURIComponent(String(params.jobId))}/apply`, {
    method: "POST",
    body: JSON.stringify({ proposal_hash: params.proposalHash, proposal: params.proposal }),
  });
  if (!res.ok) throw await parseApiError(res);
  return (await res.json()) as ApplicationResponse;
}
