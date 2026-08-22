/**
 * api/client — root HTTP client for Trust Work Escrow v3.
 * Centraliza base URL, timeouts, error mapping y helpers.
 * Consumido por todos los endpoints en frontend/src/api/*.
 */

export const API_URL = (
  process.env.NEXT_PUBLIC_API_URL ||
  process.env.NEXT_PUBLIC_BACKEND_URL ||
  "http://127.0.0.1:3000"
).replace(/\/$/, "");

export const RPC_URL_FALLBACK =
  process.env.NEXT_PUBLIC_RPC_URL || "http://127.0.0.1:8899";

export class ApiError extends Error {
  status?: number;
  code?: string;
  constructor(message: string, opts?: { status?: number; code?: string }) {
    super(message);
    this.name = "ApiError";
    this.status = opts?.status;
    this.code = opts?.code;
  }
}

function getAuthHeaders(): Record<string, string> {
  if (typeof window === "undefined") return {};
  try {
    const pubkey = localStorage.getItem("twe_pubkey") || sessionStorage.getItem("twe_pubkey") || "";
    const sig = localStorage.getItem("twe_signature") || "";
    const msg = localStorage.getItem("twe_message") || "";
    const h: Record<string, string> = {};
    if (pubkey) h["x-pubkey"] = pubkey;
    if (sig) h["x-signature"] = sig;
    if (msg) h["x-message"] = msg;
    return h;
  } catch {
    return {};
  }
}

/** Fetch wrapper con timeout y prefix automático de API_URL */
export async function apiFetch(
  path: string,
  init?: RequestInit,
  timeoutMs = 5000
): Promise<Response> {
  const controller = new AbortController();
  const t = setTimeout(() => controller.abort(), timeoutMs);
  try {
    const url = path.startsWith("http") ? path : `${API_URL}${path.startsWith("/") ? path : `/${path}`}`;
    const authHeaders = getAuthHeaders();
    const res = await fetch(url, {
      ...init,
      signal: controller.signal,
      headers: {
        "content-type": "application/json",
        ...authHeaders,
        ...(init?.headers || {}),
      },
    });
    return res;
  } finally {
    clearTimeout(t);
  }
}

export async function parseApiError(res: Response): Promise<ApiError> {
  let msg = `API ${res.status} ${res.statusText}`;
  let code: string | undefined;
  try {
    const body = await res.clone().json();
    if (body?.error) msg = body.error;
    if (body?.code) code = body.code;
    if (body?.message) msg = body.message;
  } catch {
    try {
      const text = await res.text();
      if (text) msg = text.slice(0, 400);
    } catch {}
  }
  return new ApiError(msg, { status: res.status, code });
}

/** Helper para leer JSON tipado o lanzar ApiError */
export async function apiJson<T>(path: string, init?: RequestInit, timeoutMs?: number): Promise<T> {
  const res = await apiFetch(path, init, timeoutMs);
  if (!res.ok) throw await parseApiError(res);
  return (await res.json()) as T;
}
