"use client";
import { create } from "zustand";
import { ApiError } from "@/api/client";
import type { Job } from "@/api/types";
import * as jobsApi from "@/api/jobs";
import * as appApi from "@/api/applications";
import * as msApi from "@/api/milestones";
import * as disputesApi from "@/api/disputes";
import * as supportApi from "@/api/support";
import * as arbiterApi from "@/api/arbiterPool";

export type DashboardRole = "freelancer" | "client" | null;
export type HistoryRange = "30d" | "90d" | "all";
export interface NotificationItem { id: string; title: string; body: string; at: number; read: boolean; }

interface DashboardState {
  role: DashboardRole;
  jobs: Job[];
  nextCursor: string | null;
  hasMore: boolean;
  loading: boolean;
  error: string | null;
  statusFilter: string | null; // freelancer filter: InProgress|Disputed|Completed or client: Funded|InProgress|Submitted etc
  historyRange: HistoryRange;
  notifications: NotificationItem[];
  polling: boolean;
  // drafts in-memory fallback
  drafts: Record<string, string>;
  setRole: (r: DashboardRole) => void;
  setStatusFilter: (s: string | null) => void;
  setHistoryRange: (r: HistoryRange) => void;
  fetchJobs: (opts?: { cursor?: string | null; limit?: number; status?: string; client?: string; append?: boolean }) => Promise<void>;
  fetchByStatus: (status: string) => Promise<void>;
  fetchByClient: (clientPubkey: string) => Promise<void>;
  refresh: () => Promise<void>;
  pushNotification: (n: Omit<NotificationItem, "id" | "at" | "read">) => void;
  markAllRead: () => void;
  clearError: () => void;
  reset: () => void;
  startPolling: (intervalMs?: number) => () => void;
  // derived actions that delegate to api/* via stores
  applyToJob: typeof appApi.applyToJob;
  acceptApplication: typeof appApi.acceptApplication;
  createMilestone: typeof msApi.createMilestone;
  submitMilestone: typeof msApi.submitMilestone;
  approveMilestone: typeof msApi.approveMilestone;
  rejectMilestone: typeof msApi.rejectMilestone;
  raiseDispute: typeof disputesApi.raiseDispute;
  acceptDispute: typeof disputesApi.acceptDispute;
  submitEvidence: typeof disputesApi.submitEvidence;
  assignArbiter: typeof disputesApi.assignArbiter;
  resolveDispute: typeof disputesApi.resolveDispute;
  requestIntervention: typeof disputesApi.requestPlatformIntervention;
  openSupport: typeof supportApi.openSupportTicket;
  getArbiterPool: typeof arbiterApi.getArbiterPool;
}

const init = {
  role: null as DashboardRole,
  jobs: [] as Job[],
  nextCursor: null as string | null,
  hasMore: false,
  loading: false,
  error: null as string | null,
  statusFilter: null as string | null,
  historyRange: "all" as HistoryRange,
  notifications: [] as NotificationItem[],
  polling: false,
  drafts: {} as Record<string, string>,
};

export const useDashboardStore = create<DashboardState>((set, get) => ({
  ...init,
  setRole: (r) => {
    if (typeof window !== "undefined" && r) localStorage.setItem("twe_role", r);
    set({ role: r });
  },
  setStatusFilter: (s) => set({ statusFilter: s }),
  setHistoryRange: (r) => set({ historyRange: r }),
  clearError: () => set({ error: null }),
  reset: () => set({ ...init }),
  pushNotification: (n) => set(s => ({ notifications: [{ id: Math.random().toString(36).slice(2), at: Date.now(), read: false, ...n }, ...s.notifications].slice(0, 50) })),
  markAllRead: () => set(s => ({ notifications: s.notifications.map(x => ({ ...x, read: true })) })),
  fetchJobs: async (opts) => {
    set({ loading: true, error: null });
    try {
      const status = opts?.status ?? get().statusFilter ?? undefined;
      const data = await jobsApi.listJobs({ cursor: opts?.cursor ?? null, limit: opts?.limit ?? 20, status, client: opts?.client });
      if (opts?.append && opts?.cursor) set({ jobs: [...get().jobs, ...data.jobs], nextCursor: data.nextCursor, hasMore: data.hasMore });
      else set({ jobs: data.jobs, nextCursor: data.nextCursor, hasMore: data.hasMore });
      if (data.jobs.length > 0) {
        // push silent notification for sync
        // no auto push to avoid spam; caller decides
      }
    } catch (e) {
      const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error dashboard fetchJobs";
      set({ error: msg }); throw e;
    } finally { set({ loading: false }); }
  },
  fetchByStatus: async (status) => {
    set({ statusFilter: status });
    await get().fetchJobs({ cursor: null, status });
  },
  fetchByClient: async (clientPubkey) => {
    set({ loading: true, error: null });
    try {
      const data = await jobsApi.listJobs({ cursor: null, limit: 50, client: clientPubkey } as any);
      // if backend ignores client param, filter client-side as fallback (cursor opaco preserved)
      set({ jobs: data.jobs.filter(j => !clientPubkey || j.client === clientPubkey), nextCursor: data.nextCursor, hasMore: data.hasMore });
    } catch (e) {
      const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error fetchByClient";
      set({ error: msg }); throw e;
    } finally { set({ loading: false }); }
  },
  refresh: async () => {
    await get().fetchJobs({ cursor: null });
  },
  startPolling: (intervalMs = 15000) => {
    if (get().polling) return () => {};
    set({ polling: true });
    const id = setInterval(() => { get().refresh().catch(()=>{}); }, intervalMs);
    return () => { clearInterval(id); set({ polling: false }); };
  },
  applyToJob: appApi.applyToJob,
  acceptApplication: appApi.acceptApplication,
  createMilestone: msApi.createMilestone,
  submitMilestone: msApi.submitMilestone,
  approveMilestone: msApi.approveMilestone,
  rejectMilestone: msApi.rejectMilestone,
  raiseDispute: disputesApi.raiseDispute,
  acceptDispute: disputesApi.acceptDispute,
  submitEvidence: disputesApi.submitEvidence,
  assignArbiter: disputesApi.assignArbiter,
  resolveDispute: disputesApi.resolveDispute,
  requestIntervention: disputesApi.requestPlatformIntervention,
  openSupport: supportApi.openSupportTicket,
  getArbiterPool: arbiterApi.getArbiterPool,
}));
