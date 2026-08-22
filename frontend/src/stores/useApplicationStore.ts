"use client";
import { create } from "zustand";
import { ApiError } from "@/api/client";
import type { ApplicationResponse } from "@/api/types";
import * as appApi from "@/api/applications";

interface ApplicationState {
  applications: ApplicationResponse[];
  loading: boolean;
  error: string | null;
  apply: (params: { jobId: string | number; proposal: string; proposalHash: string }) => Promise<ApplicationResponse>;
  accept: (jobId: string | number, index: number) => Promise<ApplicationResponse>;
  clearError: () => void;
  reset: () => void;
}

export const useApplicationStore = create<ApplicationState>((set) => ({
  applications: [],
  loading: false,
  error: null,

  clearError: () => set({ error: null }),
  reset: () => set({ applications: [], loading: false, error: null }),

  apply: async (params) => {
    set({ loading: true, error: null });
    try {
      const res = await appApi.applyToJob(params);
      set((s) => ({ applications: [...s.applications, res] }));
      return res;
    } catch (e) {
      const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error apply";
      set({ error: msg });
      throw e;
    } finally {
      set({ loading: false });
    }
  },

  accept: async (jobId, index) => {
    set({ loading: true, error: null });
    try {
      const res = await appApi.acceptApplication(jobId, index);
      set((s) => ({
        applications: s.applications.map((a) => (a.index === index ? res : a)),
      }));
      return res;
    } catch (e) {
      const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error accept";
      set({ error: msg });
      throw e;
    } finally {
      set({ loading: false });
    }
  },
}));
