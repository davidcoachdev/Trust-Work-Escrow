"use client";
import { useEffect, useState } from "react";
import { useAuthStore } from "@/stores/useAuthStore";

export function FreelancerOnly({ children }: { children: React.ReactNode }) {
  const role = useAuthStore(s => s.role);
  const [mounted, setMounted] = useState(false);
  useEffect(()=>setMounted(true),[]);
  if (!mounted) return <div className="card text-sm" style={{color:"var(--muted)"}}>Cargando…</div>;
  if (role !== "freelancer") return <div className="card border text-sm" style={{borderColor:"rgba(255,60,60,0.3)", background:"rgba(255,60,60,0.08)", color:"#FF8A8A"}}>Acceso solo Freelancer. Cambia rol en /dashboard. <span className="font-mono">role={String(role)}</span></div>;
  return <>{children}</>;
}
export function ClientOnly({ children }: { children: React.ReactNode }) {
  const role = useAuthStore(s => s.role);
  const [mounted, setMounted] = useState(false);
  useEffect(()=>setMounted(true),[]);
  if (!mounted) return <div className="card text-sm" style={{color:"var(--muted)"}}>Cargando…</div>;
  if (role !== "client") return <div className="card border text-sm" style={{borderColor:"rgba(255,60,60,0.3)", background:"rgba(255,60,60,0.08)", color:"#FF8A8A"}}>Acceso solo Publisher/Cliente. Cambia rol en /dashboard. <span className="font-mono">role={String(role)}</span></div>;
  return <>{children}</>;
}
