"use client";
import { create } from "zustand";
import { ApiError } from "@/api/client";
import type { Job } from "@/api/types";
import * as jobsApi from "@/api/jobs";

interface JobState {
  jobs: Job[];
  currentJob: Job | null;
  nextCursor: string | null;
  hasMore: boolean;
  loading: boolean;
  error: string | null;
  // actions — cada acción consume la capa api (que a su vez habla con backend/SDK on-chain)
  fetchJobs: (opts?: { cursor?: string | null; limit?: number }) => Promise<void>;
  fetchJob: (jobId: string | number) => Promise<Job | null>;
  createJob: (params: { title: string; description: string; amount: number; deadline: number }) => Promise<Job>;
  deposit: (jobId: string | number) => Promise<void>;
  cancel: (jobId: string | number) => Promise<void>;
  pause: (jobId: string | number) => Promise<void>;
  unpause: (jobId: string | number) => Promise<void>;
  submitWork: (jobId: string | number) => Promise<void>;
  approveWork: (jobId: string | number) => Promise<void>;
  rejectWork: (jobId: string | number) => Promise<void>;
  clearError: () => void;
  reset: () => void;
}

const initial = {
  jobs: [] as Job[],
  currentJob: null as Job | null,
  nextCursor: null as string | null,
  hasMore: false,
  loading: false,
  error: null as string | null,
};

export const useJobStore = create<JobState>((set, get) => ({
  ...initial,

  clearError: () => set({ error: null }),
  reset: () => set({ ...initial }),

  fetchJobs: async (opts) => {
    set({ loading: true, error: null });
    try {
      const data = await jobsApi.listJobs({ cursor: opts?.cursor ?? get().nextCursor, limit: opts?.limit ?? 20 });
      // on-chain Vec + off-chain metadata ya mapeados en api/types.mapJobResponse
      if (opts?.cursor) {
        set({ jobs: [...get().jobs, ...data.jobs], nextCursor: data.nextCursor, hasMore: data.hasMore });
      } else {
        set({ jobs: data.jobs, nextCursor: data.nextCursor, hasMore: data.hasMore });
      }
    } catch (e) {
      const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error listando jobs";
      set({ error: msg });
      throw e;
    } finally {
      set({ loading: false });
    }
  },

  fetchJob: async (jobId) => {
    set({ loading: true, error: null });
    try {
      const job = await jobsApi.getJob(jobId);
      set({ currentJob: job });
      if (job) {
        // mantener lista sincronizada (fuente de la verdad)
        const exists = get().jobs.find((j) => j.jobId === job.jobId);
        if (!exists) set({ jobs: [job, ...get().jobs] });
        else set({ jobs: get().jobs.map((j) => (j.jobId === job.jobId ? job : j)) });
      }
      return job;
    } catch (e) {
      const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error obteniendo job";
      set({ error: msg });
      throw e;
    } finally {
      set({ loading: false });
    }
  },

  createJob: async (params) => {
    set({ loading: true, error: null });
    try {
      const { job } = await jobsApi.createJob(params);
      set({ jobs: [job, ...get().jobs], currentJob: job });
      return job;
    } catch (e) {
      const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error creando job";
      set({ error: msg });
      throw e;
    } finally {
      set({ loading: false });
    }
  },

  deposit: async (jobId) => {
    set({ loading: true, error: null });
    try {
      await jobsApi.depositFunds(jobId);
      // re-fetch job to reflect status Funded
      await get().fetchJob(jobId);
    } catch (e) {
      const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error depositando";
      set({ error: msg });
      throw e;
    } finally {
      set({ loading: false });
    }
  },

  cancel: async (jobId) => {
    set({ loading: true, error: null });
    try {
      await jobsApi.cancelJob(jobId);
      await get().fetchJob(jobId);
    } catch (e) {
      const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error cancelando";
      set({ error: msg });
      throw e;
    } finally {
      set({ loading: false });
    }
  },

  pause: async (jobId) => {
    set({ loading: true, error: null });
    try {
      await jobsApi.pauseJob(jobId);
    } catch (e) {
      const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error pausando";
      set({ error: msg });
      throw e;
    } finally {
      set({ loading: false });
    }
  },

  unpause: async (jobId) => {
    set({ loading: true, error: null });
    try {
      await jobsApi.unpauseJob(jobId);
    } catch (e) {
      const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error reanudando";
      set({ error: msg });
      throw e;
    } finally {
      set({ loading: false });
    }
  },

  submitWork: async (jobId) => {
    set({ loading: true, error: null });
    try {
      await jobsApi.submitWork(jobId);
      await get().fetchJob(jobId);
    } catch (e) {
      const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error submitWork";
      set({ error: msg });
      throw e;
    } finally {
      set({ loading: false });
    }
  },

  approveWork: async (jobId) => {
    set({ loading: true, error: null });
    try {
      await jobsApi.approveWork(jobId);
      await get().fetchJob(jobId);
    } catch (e) {
      const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error approveWork";
      set({ error: msg });
      throw e;
    } finally {
      set({ loading: false });
    }
  },

  rejectWork: async (jobId) => {
    set({ loading: true, error: null });
    try {
      await jobsApi.rejectWork(jobId);
      await get().fetchJob(jobId);
    } catch (e) {
      const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error rejectWork";
      set({ error: msg });
      throw e;
    } finally {
      set({ loading: false });
    }
  },
}));
