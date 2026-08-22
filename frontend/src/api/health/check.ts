import { apiFetch } from "../client";

export interface HealthResponse {
  status: string;
  version?: string;
  uptime?: number;
}

export async function healthCheck(): Promise<HealthResponse> {
  const res = await apiFetch(`/health`);
  if (!res.ok) return { status: "down" };
  return (await res.json()) as HealthResponse;
}

export async function liveCheck(): Promise<HealthResponse> {
  const res = await apiFetch(`/live`);
  if (!res.ok) return { status: "down" };
  return (await res.json()) as HealthResponse;
}

export async function readyCheck(): Promise<HealthResponse> {
  const res = await apiFetch(`/ready`);
  if (!res.ok) return { status: "down" };
  return (await res.json()) as HealthResponse;
}
