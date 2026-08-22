"use client";
import { create } from "zustand";

export type Role = "freelancer" | "client" | null;

interface AuthState {
  pubkey: string | null;
  role: Role;
  message: string | null;
  signature: string | null;
  setPubkey: (k: string | null) => void;
  setRole: (r: Role) => void;
  setAuth: (pubkey: string, message: string, signature: string) => void;
  clear: () => void;
}

function persistPubkey(pubkey: string | null, message: string | null, signature: string | null) {
  if (typeof window === "undefined") return;
  try {
    if (pubkey) localStorage.setItem("twe_pubkey", pubkey);
    else localStorage.removeItem("twe_pubkey");
    if (message) localStorage.setItem("twe_message", message);
    else localStorage.removeItem("twe_message");
    if (signature) localStorage.setItem("twe_signature", signature);
    else localStorage.removeItem("twe_signature");
    if (pubkey) sessionStorage.setItem("twe_pubkey", pubkey);
  } catch {}
}

export const useAuthStore = create<AuthState>((set, get) => ({
  pubkey: typeof window !== "undefined" ? localStorage.getItem("twe_pubkey") : null,
  role: (typeof window !== "undefined" ? (localStorage.getItem("twe_role") as Role) : null) || null,
  message: typeof window !== "undefined" ? localStorage.getItem("twe_message") : null,
  signature: typeof window !== "undefined" ? localStorage.getItem("twe_signature") : null,
  setPubkey: (k) => {
    persistPubkey(k, get().message, get().signature);
    set({ pubkey: k });
  },
  setRole: (r) => {
    if (typeof window !== "undefined") {
      if (r) localStorage.setItem("twe_role", r);
      else localStorage.removeItem("twe_role");
    }
    set({ role: r });
  },
  setAuth: (pubkey, message, signature) => {
    persistPubkey(pubkey, message, signature);
    set({ pubkey, message, signature });
  },
  clear: () => {
    persistPubkey(null, null, null);
    if (typeof window !== "undefined") localStorage.removeItem("twe_role");
    set({ pubkey: null, role: null, message: null, signature: null });
  },
}));
