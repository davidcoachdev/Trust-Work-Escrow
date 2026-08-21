export const MAX_TITLE_LEN = 100;
export const MAX_DESC_LEN = 500;
export const DRAFT_KEY = "twe_drafts_client_create_v3";
export const DRAFT_MEMORY: Record<string, string> = {};

export interface DraftCreate {
  title: string;
  description: string;
  amountSol: string;
  deadlineDays: string;
  updatedAt: number;
}

export function saveDraft(d: DraftCreate) {
  DRAFT_MEMORY[DRAFT_KEY] = JSON.stringify(d);
  try { localStorage.setItem(DRAFT_KEY, JSON.stringify(d)); } catch {}
}
export function loadDraft(): DraftCreate | null {
  try {
    const raw = localStorage.getItem(DRAFT_KEY) || DRAFT_MEMORY[DRAFT_KEY];
    if (!raw) return null;
    return JSON.parse(raw) as DraftCreate;
  } catch { return null; }
}
export function clearDraft() {
  delete DRAFT_MEMORY[DRAFT_KEY];
  try { localStorage.removeItem(DRAFT_KEY); } catch {}
}

export function countdown(deadline: number): { text: string; overdue: boolean; days: number; hours: number } {
  const diff = deadline * 1000 - Date.now();
  const overdue = diff <= 0;
  const abs = Math.abs(diff);
  const days = Math.floor(abs / 86400000);
  const hours = Math.floor((abs % 86400000) / 3600000);
  const mins = Math.floor((abs % 3600000) / 60000);
  if (overdue) return { text: `Vencido hace ${days}d ${hours}h`, overdue: true, days, hours };
  return { text: `${days}d ${hours}h ${mins}m restantes`, overdue: false, days, hours };
}

export function autoApproveCountdown(submittedAt: number, days = 7): { text: string; pct: number; remainingMs: number } {
  const deadline = submittedAt + days * 86400;
  const remaining = deadline * 1000 - Date.now();
  const pct = Math.max(0, Math.min(100, ((days * 86400 * 1000 - remaining) / (days * 86400 * 1000)) * 100));
  if (remaining <= 0) return { text: "Auto-aprobado", pct: 100, remainingMs: 0 };
  const d = Math.floor(remaining / 86400000);
  const h = Math.floor((remaining % 86400000) / 3600000);
  return { text: `Auto-approve en ${d}d ${h}h`, pct, remainingMs: remaining };
}

export function toCsv(rows: Record<string, string | number>[]): string {
  if (rows.length === 0) return "";
  const headers = Object.keys(rows[0]);
  const esc = (v: unknown) => `"${String(v ?? "").replace(/"/g, '""')}"`;
  return [headers.map(esc).join(","), ...rows.map(r => headers.map(h => esc(r[h])).join(","))].join("\n");
}

export function downloadCsv(filename: string, csv: string) {
  const blob = new Blob([csv], { type: "text/csv;charset=utf-8;" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url; a.download = filename; a.click();
  URL.revokeObjectURL(url);
}

export function filterByRange<T extends { createdAt?: number; deadline?: number }>(items: T[], range: "30d"|"90d"|"all"): T[] {
  if (range === "all") return items;
  const days = range === "30d" ? 30 : 90;
  const cutoff = Date.now() - days * 86400000;
  return items.filter(i => {
    const ts = (i.createdAt ?? i.deadline ?? 0) * 1000;
    if (!ts) return true;
    return ts >= cutoff;
  });
}

export interface MetricAgg {
  totalGastado: number;
  totalFee: number;
  disputasPct: number;
  avgDays: number;
  count: number;
}

export function computeMetrics(jobs: { amount: string; status: string; createdAt?: number; deadline?: number }[]): MetricAgg {
  const totalGastado = jobs.reduce((a, j) => a + Number(j.amount), 0);
  const fee = Math.floor(totalGastado * 0.025);
  const disputas = jobs.filter(j => j.status === "Disputed").length;
  const disputasPct = jobs.length ? (disputas / jobs.length) * 100 : 0;
  const durations = jobs.map(j => j.deadline && j.createdAt ? (j.deadline - j.createdAt) / 86400 : 7).filter(n => Number.isFinite(n));
  const avgDays = durations.length ? durations.reduce((a, b) => a + b, 0) / durations.length : 0;
  return { totalGastado, totalFee: fee, disputasPct, avgDays, count: jobs.length };
}
