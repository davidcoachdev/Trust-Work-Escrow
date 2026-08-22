import { apiFetch, parseApiError, ApiError } from "../client";
import { type Job, type JobResponse, mapJobResponse } from "../types";

export async function getJob(jobId: string | number): Promise<Job | null> {
  const res = await apiFetch(`/jobs/${encodeURIComponent(String(jobId))}`);
  if (res.status === 404) return null;
  if (!res.ok) throw await parseApiError(res);
  const data = (await res.json()) as JobResponse;
  return mapJobResponse(data);
}

export async function getJobOrThrow(jobId: string | number): Promise<Job> {
  const job = await getJob(jobId);
  if (!job) throw new ApiError(`job ${jobId} not found`, { status: 404, code: "not_found" });
  return job;
}
