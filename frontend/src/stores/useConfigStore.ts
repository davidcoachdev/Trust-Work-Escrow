"use client";
import { create } from "zustand";
import { ApiError } from "@/api/client";
import type { ConfigResponse, ArbiterPoolResponse } from "@/api/types";
import * as configApi from "@/api/config";
import * as arbiterApi from "@/api/arbiterPool";

interface ConfigState {
  config: ConfigResponse | null;
  arbiterPool: ArbiterPoolResponse | null;
  loading: boolean;
  error: string | null;
  fetchConfig: () => Promise<ConfigResponse>;
  fetchArbiterPool: () => Promise<ArbiterPoolResponse | null>;
  createPool: () => Promise<ArbiterPoolResponse>;
  addArbiter: (arbiter: string) => Promise<ArbiterPoolResponse>;
  removeArbiter: (arbiter: string) => Promise<ArbiterPoolResponse>;
  clearError: () => void;
}

export const useConfigStore = create<ConfigState>((set) => ({
  config: null,
  arbiterPool: null,
  loading: false,
  error: null,
  clearError: () => set({ error: null }),
  fetchConfig: async () => {
    set({ loading: true, error: null });
    try { const c = await configApi.getConfig(); set({ config: c }); return c; } catch (e) { const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error config"; set({ error: msg }); throw e; } finally { set({ loading: false }); }
  },
  fetchArbiterPool: async () => {
    set({ loading: true, error: null });
    try { const p = await arbiterApi.getArbiterPool(); set({ arbiterPool: p }); return p; } catch (e) { if (e instanceof ApiError && e.status === 404) { set({ arbiterPool: null }); return null; } const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error arbiter pool"; set({ error: msg }); throw e; } finally { set({ loading: false }); }
  },
  createPool: async () => {
    set({ loading: true, error: null });
    try { const p = await arbiterApi.createArbiterPool(); set({ arbiterPool: p }); return p; } catch (e) { const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error create pool"; set({ error: msg }); throw e; } finally { set({ loading: false }); }
  },
  addArbiter: async (arbiter) => {
    set({ loading: true, error: null });
    try { const p = await arbiterApi.addArbiter(arbiter); set({ arbiterPool: p }); return p; } catch (e) { const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error add arbiter"; set({ error: msg }); throw e; } finally { set({ loading: false }); }
  },
  removeArbiter: async (arbiter) => {
    set({ loading: true, error: null });
    try { const p = await arbiterApi.removeArbiter(arbiter); set({ arbiterPool: p }); return p; } catch (e) { const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "error remove arbiter"; set({ error: msg }); throw e; } finally { set({ loading: false }); }
  },
}));
