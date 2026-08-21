"use client";
import Link from "next/link";
import { motion } from "framer-motion";
import { useEffect } from "react";
import gsap from "gsap";
import { useAuthStore } from "@/stores/useAuthStore";
import { useDashboardStore } from "@/stores/useDashboardStore";

export default function DashboardRoot() {
  const { role, setRole } = useAuthStore();
  const { jobs, refresh } = useDashboardStore();
  useEffect(()=>{
    const ctx = gsap.context(()=>{ gsap.from("[data-dash-card]", { y:16, opacity:0, duration:0.5, stagger:0.07, ease:"power3.out"}); });
    return ()=>ctx.revert();
  },[]);
  useEffect(()=>{ refresh().catch(()=>{}); },[refresh]);

  return (
    <div className="space-y-6">
      <div data-dash-card>
        <h1 className="text-2xl font-bold" style={{color:"var(--fg)"}}>Dashboard Trust Work Escrow v3</h1>
        <p className="text-sm" style={{color:"var(--muted)"}}>Elige rol · Freelancer global overview o Publisher (Cliente) · separación completa · Notificaciones bell + sync polling · validator 7a2Y UP</p>
      </div>
      <div className="grid gap-4 md:grid-cols-2">
        <motion.div data-dash-card whileHover={{y:-3}} className={`card card-hover cursor-pointer ${role==="freelancer"?"ring-2":""}`} style={role==="freelancer"?{borderColor:"var(--primary)"}:{}} onClick={()=>setRole("freelancer")}>
          <div className="text-xs font-bold tracking-widest" style={{color:"var(--primary)"}}>ROL FREELANCER</div>
          <h2 className="mt-1 text-lg font-bold" style={{color:"var(--fg)"}}>Dashboard Freelancer</h2>
          <p className="mt-1 text-sm" style={{color:"var(--muted)"}}>Overview cards Activos/En disputa/Cerrados/Ganancia/Rating, gráfico 7d, lista En curso/En disputa/Cerrados, detalle /jobs/[id] con tabs Chat/Evidencias/Milestones/Disputa/Pagos, countdown, auto-approve 7d</p>
          <Link href="/dashboard/freelancer" className="btn mt-4" onClick={e=>e.stopPropagation()}>Abrir Freelancer</Link>
        </motion.div>
        <motion.div data-dash-card whileHover={{y:-3}} className={`card card-hover cursor-pointer ${role==="client"?"ring-2":""}`} style={role==="client"?{borderColor:"var(--primary)"}:{}} onClick={()=>setRole("client")}>
          <div className="text-xs font-bold tracking-widest" style={{color:"var(--primary)"}}>ROL PUBLISHER / CLIENTE</div>
          <h2 className="mt-1 text-lg font-bold" style={{color:"var(--fg)"}}>Dashboard Publisher</h2>
          <p className="mt-1 text-sm" style={{color:"var(--muted)"}}>/client/create form MAX_TITLE 100 + borradores, En ejecución Funded/InProgress/Submitted + avatar + applicants Vec50 + chat, Disputas raise/assign/resolve, Historial Released/Resolved/Cancelled + métricas + CSV</p>
          <Link href="/dashboard/client" className="btn mt-4" onClick={e=>e.stopPropagation()}>Abrir Publisher</Link>
        </motion.div>
      </div>
      <div data-dash-card className="card" style={{background:"var(--surface-2)"}}>
        <div className="text-xs font-semibold" style={{color:"var(--muted)"}}>Jobs cache · {jobs.length} loaded · useDashboardStore → api/jobs/list (cursor opaco + status + by_client) · polling 15s</div>
        <div className="mt-2 flex gap-2 text-xs" style={{color:"var(--muted)"}}>
          <span>Rol actual: <b style={{color:"var(--fg)"}}>{role ?? "— selecciona —"}</b></span>
          <span>·</span>
          <span>Tema dcdev crimson #FF3C3C + GSAP stagger + Framer layout</span>
        </div>
      </div>
    </div>
  );
}
