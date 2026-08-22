"use client";
import { create } from "zustand";
import { ApiError } from "@/api/client";
import type { MilestoneResponse, CreateMilestoneParams } from "@/api/types";
import * as msApi from "@/api/milestones";

interface MilestoneState {
  milestones: MilestoneResponse[];
  loading: boolean;
  error: string | null;
  create: (jobId: string | number, params: CreateMilestoneParams) => Promise<MilestoneResponse>;
  submit: (jobId: string | number, index: number) => Promise<MilestoneResponse>;
  approve: (jobId: string | number, index: number) => Promise<MilestoneResponse>;
  reject: (jobId: string | number, index: number) => Promise<MilestoneResponse>;
  clearError: () => void;
  reset: () => void;
}

export const useMilestoneStore = create<MilestoneState>((set) => ({
  milestones: [],
  loading: false,
  error: null,
  clearError: () => set({ error: null }),
  reset: () => set({ milestones: [], loading: false, error: null }),

  create: async (jobId, params) => {
    set({ loading: true, error: null });
    try {
      const res = await msApi.createMilestone(jobId, params);
      set((s) => ({ milestones: [...s.milestones, res] }));
      return res;
    } catch (e) {
      const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error create milestone";
      set({ error: msg });
      throw e;
    } finally {
      set({ loading: false });
    }
  },
  submit: async (jobId, index) => {
    set({ loading: true, error: null });
    try {
      const res = await msApi.submitMilestone(jobId, index);
      set((s) => ({ milestones: s.milestones.map((m) => (m.index === index ? res : m)) }));
      return res;
    } catch (e) {
      const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error submit milestone";
      set({ error: msg });
      throw e;
    } finally {
      set({ loading: false });
    }
  },
  approve: async (jobId, index) => {
    set({ loading: true, error: null });
    try {
      const res = await msApi.approveMilestone(jobId, index);
      set((s) => ({ milestones: s.milestones.map((m) => (m.index === index ? res : m)) }));
      return res;
    } catch (e) {
      const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error approve milestone";
      set({ error: msg });
      throw e;
    } finally {
      set({ loading: false });
    }
  },
  reject: async (jobId, index) => {
    set({ loading: true, error: null });
    try {
      const res = await msApi.rejectMilestone(jobId, index);
      set((s) => ({ milestones: s.milestones.map((m) => (m.index === index ? res : m)) }));
      return res;
    } catch (e) {
      const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error reject milestone";
      set({ error: msg });
      throw e;
    } finally {
      set({ loading: false });
    }
  },
}));
