"use client";
import { useEffect, useState, useMemo } from "react";
import Link from "next/link";
import { motion } from "framer-motion";
import gsap from "gsap";
import { useDashboardStore } from "@/stores/useDashboardStore";
import { FreelancerOnly } from "@/components/dashboard/RoleGuard";
import { OverviewCards } from "@/components/dashboard/OverviewCards";
import { Chart7d } from "@/components/dashboard/Chart7d";
import { DeadlineCountdown } from "@/components/dashboard/DeadlineCountdown";
import { AutoApproveBadge } from "@/components/dashboard/AutoApproveBadge";

const filters = [
  { id:"all", label:"Todos" },
  { id:"InProgress", label:"En curso" },
  { id:"Disputed", label:"En disputa" },
  { id:"Completed", label:"Cerrados" },
];

export default function FreelancerDashboard() {
  const { jobs, fetchByStatus, fetchJobs, loading, error, statusFilter } = useDashboardStore();
  const [active,setActive]=useState<string>("all");

  useEffect(()=>{
    fetchJobs({ cursor:null }).catch(()=>{});
    const ctx = gsap.context(()=>{ gsap.from("[data-f-card]",{ y:14, opacity:0, duration:0.5, stagger:0.06, ease:"power3.out"}); });
    return ()=>ctx.revert();
  },[fetchJobs]);

  const filtered = useMemo(()=>{
    if(active==="all") return jobs;
    if(active==="InProgress") return jobs.filter(j=> j.status==="InProgress" || j.status==="Open");
    return jobs.filter(j=> j.status===active);
  },[jobs, active]);

  async function onFilter(id:string){
    setActive(id);
    if(id==="all") await fetchJobs({ cursor:null }).catch(()=>{});
    else await fetchByStatus(id==="InProgress"?"InProgress":id).catch(()=>{});
  }

  return (
    <FreelancerOnly>
      <div className="space-y-6">
        <div data-f-card>
          <h1 className="text-2xl font-bold" style={{color:"var(--fg)"}}>Freelancer Dashboard</h1>
          <p className="text-sm" style={{color:"var(--muted)"}}>Global overview · Activos/En disputa/Cerrados/Ganancia/Rating · gráfico 7d · filtros En curso/En disputa/Cerrados · cursor opaco · polling</p>
        </div>

        <div data-f-card><OverviewCards jobs={jobs} /></div>
        <div data-f-card><Chart7d jobs={jobs} /></div>

        <div data-f-card className="flex flex-wrap gap-2">
          {filters.map(f=>(
            <button key={f.id} onClick={()=>onFilter(f.id)} className={`rounded-full px-3.5 py-1.5 text-sm font-medium ${active===f.id?'text-white':''}`} style={active===f.id?{background:"var(--primary)"}:{border:"1px solid var(--border)", background:"var(--surface)", color:"var(--muted)"}}>{f.label}</button>
          ))}
          <span className="ml-2 self-center text-xs" style={{color:"var(--muted)"}}>{loading?"Cargando…": `${filtered.length} jobs · statusFilter=${statusFilter ?? "null"}`}</span>
        </div>

        {error && <div className="card text-sm" style={{borderColor:"rgba(255,60,60,0.3)", background:"rgba(255,60,60,0.08)", color:"#FF8A8A"}}>{error}</div>}

        <motion.div data-f-card initial="hidden" animate="visible" variants={{hidden:{}, visible:{transition:{staggerChildren:0.05}}}} className="grid gap-4 md:grid-cols-2">
          {filtered.length===0 ? <div className="card text-sm md:col-span-2" style={{color:"var(--muted)"}}>Sin trabajos en filtro {active}. Prueba cargar más o cambiar filtro.</div> : filtered.map((j,idx)=>(
            <motion.div key={j.jobId} variants={{hidden:{opacity:0,y:12}, visible:{opacity:1,y:0}}} whileHover={{y:-3}} className="h-full">
              <Link href={`/dashboard/freelancer/jobs/${j.jobId}`} className="flex h-full flex-col rounded-[16px] p-5" style={{background:"var(--surface)", border:"1px solid var(--border)"}}>
                <div className="flex items-start justify-between gap-2">
                  <h3 className="font-semibold leading-tight" style={{color:"var(--fg)"}}>{j.title}</h3>
                  <span className="shrink-0 rounded-full border px-2 py-0.5 text-xs" style={{borderColor:"var(--border)", background:"var(--surface-2)", color:"var(--muted)"}}>{j.status}</span>
                </div>
                <p className="mt-1 line-clamp-2 text-sm" style={{color:"var(--muted)"}}>{j.description || "Sin descripción"}</p>
                <div className="mt-3 flex flex-wrap items-center gap-2">
                  <span className="rounded-full px-2.5 py-1 text-xs font-mono font-semibold text-white" style={{background:"var(--gradient)"}}>{Number(j.amount)/1e9} SOL</span>
                  <DeadlineCountdown deadline={j.deadline} />
                </div>
                {j.status==="InProgress" && <div className="mt-2"><AutoApproveBadge submittedAt={j.deadline - 6*86400} /></div>}
                <div className="mt-2 text-xs" style={{color:"var(--muted)"}}>#{j.jobId} · {j.client.slice(0,12)}… · applicants {j.applicantsCount ?? 0}/50</div>
              </Link>
            </motion.div>
          ))}
        </motion.div>

        <div className="flex justify-center">
          <Link href="/dashboard/freelancer/history" className="btn btn-secondary">Ver Historial & Métricas</Link>
        </div>
      </div>
    </FreelancerOnly>
  );
}
