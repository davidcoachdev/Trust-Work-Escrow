import { apiFetch, parseApiError } from "../client";
import { type Job, type JobResponse, type CreateJobParams, mapJobResponse, validateCreateJobParams } from "../types";
import { ApiError } from "../client";

export async function createJob(params: CreateJobParams): Promise<{ job: Job; raw: JobResponse }> {
  const err = validateCreateJobParams(params);
  if (err) throw new ApiError(err, { status: 400, code: "bad_request" });
  const res = await apiFetch(`/jobs`, {
    method: "POST",
    body: JSON.stringify({
      title: params.title.trim(),
      description: params.description.trim(),
      amount: params.amount,
      deadline: params.deadline,
    }),
  });
  if (!res.ok) throw await parseApiError(res);
  const data = (await res.json()) as JobResponse;
  return { job: mapJobResponse(data), raw: data };
}
