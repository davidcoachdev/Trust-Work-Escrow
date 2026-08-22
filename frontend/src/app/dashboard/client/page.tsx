"use client";
import { useEffect, useMemo, useState } from "react";
import Link from "next/link";
import { motion } from "framer-motion";
import gsap from "gsap";
import { useDashboardStore } from "@/stores/useDashboardStore";
import { useAuthStore } from "@/stores/useAuthStore";
import { ClientOnly } from "@/components/dashboard/RoleGuard";
import { ChatTab } from "@/components/dashboard/ChatTab";

export default function ClientDashboard(){
  const { jobs, fetchJobs, fetchByClient, loading, error } = useDashboardStore();
  const pubkey = useAuthStore(s=>s.pubkey);
  const [showChatFor,setShowChatFor]=useState<string|null>(null);

  useEffect(()=>{
    if(pubkey) fetchByClient(pubkey).catch(()=> fetchJobs({cursor:null}).catch(()=>{}));
    else fetchJobs({cursor:null}).catch(()=>{});
    const ctx=gsap.context(()=>{ gsap.from("[data-c-card]",{y:12,opacity:0,duration:0.5,stagger:0.06}); });
    return ()=>ctx.revert();
  },[pubkey, fetchJobs, fetchByClient]);

  const enEjecucion = useMemo(()=> jobs.filter(j=> {
    const s = String(j.status);
    return s==="Open" || s==="InProgress" || s==="Funded" || s==="Submitted";
  }),[jobs]);
  const display = enEjecucion.length ? enEjecucion : jobs.filter(j=> {
    const s = String(j.status);
    return s!=="Completed" && s!=="Cancelled";
  });

  return (
    <ClientOnly>
      <div className="space-y-6">
        <div data-c-card>
          <h1 className="text-2xl font-bold" style={{color:"var(--fg)"}}>Publisher Dashboard — En ejecución</h1>
          <p className="text-sm" style={{color:"var(--muted)"}}>Lista Funded/InProgress/Submitted con freelancer avatar + applicants Vec 50 + chat por job · by_client cursor opaco</p>
        </div>

        {error && <div data-c-card className="card text-sm" style={{borderColor:"rgba(255,60,60,0.3)", background:"rgba(255,60,60,0.08)", color:"#FF8A8A"}}>{error}</div>}

        <div data-c-card className="flex gap-2">
          <Link href="/dashboard/client/create" className="btn">+ Crear job</Link>
          <Link href="/dashboard/client/disputes" className="btn btn-secondary">Disputas</Link>
          <Link href="/dashboard/client/history" className="btn btn-ghost">Historial</Link>
        </div>

        {loading && <div data-c-card className="card text-sm" style={{color:"var(--muted)"}}>Cargando…</div>}

        <motion.div data-c-card initial="hidden" animate="visible" variants={{hidden:{}, visible:{transition:{staggerChildren:0.05}}}} className="grid gap-4 md:grid-cols-2">
          {display.length===0 ? <div className="card text-sm md:col-span-2" style={{color:"var(--muted)"}}>Sin jobs en ejecución. Crea uno.</div> : display.map(j=>(
            <motion.div key={j.jobId} variants={{hidden:{opacity:0,y:10}, visible:{opacity:1,y:0}}} className="card space-y-3">
              <div className="flex items-start justify-between gap-2">
                <h3 className="font-semibold" style={{color:"var(--fg)"}}>{j.title}</h3>
                <span className="shrink-0 rounded-full border px-2 py-0.5 text-xs" style={{borderColor:"var(--border)", background:"var(--surface-2)", color:"var(--muted)"}}>{j.status}</span>
              </div>
              <p className="line-clamp-2 text-sm" style={{color:"var(--muted)"}}>{j.description || "Sin descripción"}</p>
              <div className="flex items-center gap-2">
                <div className="flex items-center gap-2">
                  <div className="grid h-8 w-8 place-items-center rounded-full text-xs font-bold text-white" style={{background:"var(--gradient)"}}>{(j.freelancer ?? "—").slice(0,2).toUpperCase()}</div>
                  <div>
                    <div className="text-xs font-mono" style={{color:"var(--fg)"}}>{j.freelancer ? j.freelancer.slice(0,12)+"…" : "Sin freelancer"}</div>
                    <div className="text-xs" style={{color:"var(--muted)"}}>Applicants Vec 50: {j.applicantsCount ?? 0}/50</div>
                  </div>
                </div>
                <span className="ml-auto rounded-full px-2.5 py-1 text-xs font-mono font-semibold text-white" style={{background:"var(--gradient)"}}>{Number(j.amount)/1e9} SOL</span>
              </div>
              <div className="flex gap-2">
                <button onClick={()=>setShowChatFor(showChatFor===j.jobId?null:j.jobId)} className="btn btn-secondary px-3 py-1 text-xs">{showChatFor===j.jobId?"Cerrar chat":"Chat"}</button>
                <Link href={`/jobs/${j.jobId}`} className="btn btn-ghost px-3 py-1 text-xs">Ver job</Link>
                <Link href={`/dashboard/freelancer/jobs/${j.jobId}`} className="btn btn-ghost px-3 py-1 text-xs">Abrir freelancer</Link>
              </div>
              {showChatFor===j.jobId && <ChatTab jobId={j.jobId} />}
            </motion.div>
          ))}
        </motion.div>
      </div>
    </ClientOnly>
  );
}
