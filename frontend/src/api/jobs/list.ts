import { apiFetch, parseApiError } from "../client";
import { type Job, type JobResponse, type PaginatedJobs, mapJobResponse } from "../types";

export interface ListJobsParams {
  cursor?: string | null;
  limit?: number;
  status?: string;
  client?: string;
}

export async function listJobs(params: ListJobsParams = {}): Promise<PaginatedJobs> {
  const qs = new URLSearchParams();
  if (params.cursor) qs.set("cursor", params.cursor);
  if (params.limit) qs.set("limit", String(params.limit));
  if (params.status) qs.set("status", params.status);
  if (params.client) qs.set("client", params.client);
  const suffix = qs.toString() ? `?${qs}` : "";
  try {
    const res = await apiFetch(`/jobs${suffix}`);
    if (!res.ok) throw await parseApiError(res);
    const data = await res.json();
    // backend may return {jobs: JobResponse[]} or array
    const raw: JobResponse[] = Array.isArray(data) ? data : data.jobs || data.data || [];
    const jobs: Job[] = raw.map(mapJobResponse);
    // cursor pagination: backend list_jobs uses len as cursor; mimic mock pagination
    const nextCursor = (data as any).nextCursor ?? (data as any).next_cursor ?? null;
    const hasMore = Boolean(nextCursor) || (jobs.length > 0 && params.limit ? jobs.length >= params.limit : false);
    if (jobs.length === 0) return { jobs: [], nextCursor: null, hasMore: false };
    // if backend returns array without pagination, slice client-side when limit provided
    return { jobs, nextCursor: nextCursor ?? null, hasMore };
  } catch (e) {
    if (e instanceof Error && e.name === "ApiError") throw e;
    // fallback to empty for network-offline tests
    throw e;
  }
}
