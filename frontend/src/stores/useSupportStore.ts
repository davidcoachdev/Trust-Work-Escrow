"use client";
import { create } from "zustand";
import { ApiError } from "@/api/client";
import type { SupportTicketResponse } from "@/api/types";
import * as supportApi from "@/api/support";

interface SupportState {
  ticket: SupportTicketResponse | null;
  loading: boolean;
  error: string | null;
  open: (jobId: string | number) => Promise<SupportTicketResponse>;
  resolve: (jobId: string | number) => Promise<SupportTicketResponse>;
  clearError: () => void;
}

export const useSupportStore = create<SupportState>((set) => ({
  ticket: null,
  loading: false,
  error: null,
  clearError: () => set({ error: null }),
  open: async (jobId) => {
    set({ loading: true, error: null });
    try { const t = await supportApi.openSupportTicket(jobId); set({ ticket: t }); return t; } catch (e) { const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error open support"; set({ error: msg }); throw e; } finally { set({ loading: false }); }
  },
  resolve: async (jobId) => {
    set({ loading: true, error: null });
    try { const t = await supportApi.resolveSupportTicket(jobId); set({ ticket: t }); return t; } catch (e) { const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error resolve support"; set({ error: msg }); throw e; } finally { set({ loading: false }); }
  },
}));
