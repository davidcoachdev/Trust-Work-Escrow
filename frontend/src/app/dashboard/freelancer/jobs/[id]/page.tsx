"use client";
import { useEffect, useState } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
import { motion, AnimatePresence } from "framer-motion";
import gsap from "gsap";
import { useJobStore } from "@/stores/useJobStore";
import { FreelancerOnly } from "@/components/dashboard/RoleGuard";
import { DeadlineCountdown } from "@/components/dashboard/DeadlineCountdown";
import { AutoApproveBadge } from "@/components/dashboard/AutoApproveBadge";
import { ChatTab } from "@/components/dashboard/ChatTab";
import { EvidenceTab } from "@/components/dashboard/EvidenceTab";
import { MilestoneTab } from "@/components/dashboard/MilestoneTab";
import { DisputeTab } from "@/components/dashboard/DisputeTab";
import { PaymentsTab } from "@/components/dashboard/PaymentsTab";

const tabs = [
  { id:"chat", label:"Chat" },
  { id:"evidencias", label:"Evidencias" },
  { id:"milestones", label:"Milestones" },
  { id:"disputa", label:"Disputa" },
  { id:"pagos", label:"Pagos" },
] as const;

export default function FreelancerJobDetail(){
  const params = useParams<{id:string}>();
  const id = params.id;
  const { currentJob, fetchJob } = useJobStore();
  const [tab,setTab]=useState<typeof tabs[number]["id"]>("chat");
  const [err,setErr]=useState<string|null>(null);
  const [loading,setLoading]=useState(true);
  const job = currentJob && currentJob.jobId===id ? currentJob : null;

  useEffect(()=>{
    let c=false;
    (async()=>{
      try{ setLoading(true); const j=await fetchJob(id); if(!c && !j) setErr(`Job #${id} no encontrado`);}catch(e:any){ if(!c) setErr(e.message ?? String(e)); } finally{ if(!c) setLoading(false); }
    })();
    return ()=>{c=true;};
  },[id, fetchJob]);

  useEffect(()=>{
    if(!loading && job){
      const ctx = gsap.context(()=>{ gsap.from("[data-fj-tab]",{y:8, opacity:0, duration:0.35, stagger:0.05}); });
      return ()=>ctx.revert();
    }
  },[loading, job, tab]);

  if(loading) return <div className="card text-sm" style={{color:"var(--muted)"}}>Cargando job #{id}…</div>;
  if(err) return <div className="card text-sm" style={{borderColor:"rgba(255,60,60,0.3)", background:"rgba(255,60,60,0.08)", color:"#FF8A8A"}}>{err} <Link href="/dashboard/freelancer" className="underline">Volver</Link></div>;
  if(!job) return <div className="card text-sm" style={{color:"var(--muted)"}}>Job #{id} no encontrado</div>;

  return (
    <FreelancerOnly>
      <div className="space-y-4">
        <Link href="/dashboard/freelancer" className="text-xs hover:underline" style={{color:"var(--muted)"}}>← Freelancer</Link>
        <div className="card">
          <div className="flex flex-wrap items-start justify-between gap-3">
            <h1 className="text-xl font-bold" style={{color:"var(--fg)"}}>{job.title} <span className="font-mono text-xs" style={{color:"var(--muted)"}}>#{job.jobId} · {job.status}</span></h1>
            <DeadlineCountdown deadline={job.deadline} />
          </div>
          <p className="mt-2 text-sm" style={{color:"var(--muted)"}}>{job.description}</p>
          <div className="mt-3 flex flex-wrap gap-2 text-xs">
            <span className="rounded-full px-2.5 py-1 font-mono" style={{background:"var(--gradient)", color:"white"}}>{Number(job.amount)/1e9} SOL</span>
            <span style={{color:"var(--muted)"}}>Cliente {job.client.slice(0,16)}…</span>
            <AutoApproveBadge submittedAt={job.deadline - 5*86400} />
          </div>
        </div>

        <div className="flex flex-wrap gap-2" data-fj-tab>
          {tabs.map(t=>(
            <button key={t.id} onClick={()=>setTab(t.id)} className={`rounded-full px-3.5 py-1.5 text-sm font-medium ${tab===t.id?'text-white':''}`} style={tab===t.id?{background:"var(--primary)"}:{border:"1px solid var(--border)", background:"var(--surface)", color:"var(--muted)"}}>{t.label}</button>
          ))}
        </div>

        <AnimatePresence mode="wait">
          <motion.div key={tab} initial={{opacity:0,y:8}} animate={{opacity:1,y:0}} exit={{opacity:0,y:-6}} transition={{duration:0.25}} className="card">
            {tab==="chat" && <ChatTab jobId={job.jobId} />}
            {tab==="evidencias" && <EvidenceTab jobId={job.jobId} />}
            {tab==="milestones" && <MilestoneTab jobId={job.jobId} />}
            {tab==="disputa" && <DisputeTab jobId={job.jobId} />}
            {tab==="pagos" && <PaymentsTab jobId={job.jobId} amount={job.amount} />}
          </motion.div>
        </AnimatePresence>
      </div>
    </FreelancerOnly>
  );
}
