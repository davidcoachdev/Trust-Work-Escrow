"use client";
import { create } from "zustand";
import { ApiError } from "@/api/client";
import type { DisputeResponse, EvidenceResponse } from "@/api/types";
import * as disputesApi from "@/api/disputes";

interface DisputeState {
  dispute: DisputeResponse | null;
  evidence: EvidenceResponse[];
  loading: boolean;
  error: string | null;
  raise: (jobId: string | number) => Promise<DisputeResponse>;
  accept: (jobId: string | number) => Promise<DisputeResponse>;
  submitEvidence: (jobId: string | number, content: string, contentHash: string) => Promise<EvidenceResponse>;
  assignArbiter: (jobId: string | number) => Promise<DisputeResponse>;
  resolve: (jobId: string | number, pct: number) => Promise<DisputeResponse>;
  platformResolve: (jobId: string | number, pct: number) => Promise<DisputeResponse>;
  requestIntervention: (jobId: string | number) => Promise<DisputeResponse>;
  finalize: (jobId: string | number) => Promise<DisputeResponse>;
  clearError: () => void;
  reset: () => void;
}

export const useDisputeStore = create<DisputeState>((set) => ({
  dispute: null,
  evidence: [],
  loading: false,
  error: null,
  clearError: () => set({ error: null }),
  reset: () => set({ dispute: null, evidence: [], loading: false, error: null }),

  raise: async (jobId) => {
    set({ loading: true, error: null });
    try {
      const d = await disputesApi.raiseDispute(jobId);
      set({ dispute: d });
      return d;
    } catch (e) {
      const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error raise dispute";
      set({ error: msg }); throw e;
    } finally { set({ loading: false }); }
  },
  accept: async (jobId) => {
    set({ loading: true, error: null });
    try {
      const d = await disputesApi.acceptDispute(jobId);
      set({ dispute: d }); return d;
    } catch (e) { const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error accept dispute"; set({ error: msg }); throw e; } finally { set({ loading: false }); }
  },
  submitEvidence: async (jobId, content, contentHash) => {
    set({ loading: true, error: null });
    try {
      const ev = await disputesApi.submitEvidence(jobId, content, contentHash);
      set((s) => ({ evidence: [...s.evidence, ev] }));
      return ev;
    } catch (e) { const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error submit evidence"; set({ error: msg }); throw e; } finally { set({ loading: false }); }
  },
  assignArbiter: async (jobId) => {
    set({ loading: true, error: null });
    try { const d = await disputesApi.assignArbiter(jobId); set({ dispute: d }); return d; } catch (e) { const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error assign arbiter"; set({ error: msg }); throw e; } finally { set({ loading: false }); }
  },
  resolve: async (jobId, pct) => {
    set({ loading: true, error: null });
    try { const d = await disputesApi.resolveDispute(jobId, pct); set({ dispute: d }); return d; } catch (e) { const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error resolve"; set({ error: msg }); throw e; } finally { set({ loading: false }); }
  },
  platformResolve: async (jobId, pct) => {
    set({ loading: true, error: null });
    try { const d = await disputesApi.resolvePlatformCase(jobId, pct); set({ dispute: d }); return d; } catch (e) { const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error platform resolve"; set({ error: msg }); throw e; } finally { set({ loading: false }); }
  },
  requestIntervention: async (jobId) => {
    set({ loading: true, error: null });
    try { const d = await disputesApi.requestPlatformIntervention(jobId); set({ dispute: d }); return d; } catch (e) { const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error request intervention"; set({ error: msg }); throw e; } finally { set({ loading: false }); }
  },
  finalize: async (jobId) => {
    set({ loading: true, error: null });
    try { const d = await disputesApi.finalizeDisputePayouts(jobId); set({ dispute: d }); return d; } catch (e) { const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error finalize"; set({ error: msg }); throw e; } finally { set({ loading: false }); }
  },
}));
