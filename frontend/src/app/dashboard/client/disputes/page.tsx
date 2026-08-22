"use client";
import { useEffect, useState } from "react";
import { motion } from "framer-motion";
import gsap from "gsap";
import { useDashboardStore } from "@/stores/useDashboardStore";
import { useDisputeStore } from "@/stores/useDisputeStore";
import { ClientOnly } from "@/components/dashboard/RoleGuard";
import { useJobStore } from "@/stores/useJobStore";

export default function ClientDisputesPage(){
  const { jobs, fetchJobs } = useDashboardStore();
  const { raise, accept, submitEvidence, assignArbiter, resolve, loading } = useDisputeStore();
  const { fetchJob } = useJobStore();
  const [jobId,setJobId]=useState("");
  const [msg,setMsg]=useState<string|null>(null);
  const [evidenceText,setEvidenceText]=useState("");
  const [pct,setPct]=useState(50);

  useEffect(()=>{
    fetchJobs({cursor:null, limit:30}).catch(()=>{});
    const ctx=gsap.context(()=>{ gsap.from("[data-disp-card]",{y:12, opacity:0, duration:0.45, stagger:0.07});});
    return ()=>ctx.revert();
  },[fetchJobs]);

  const disputed = jobs.filter(j=> j.status==="Disputed");

  async function wrap(p:Promise<any>, label:string){
    try{ const r=await p; setMsg(`${label} OK: ${JSON.stringify(r).slice(0,140)}`); }
    catch(e:any){ setMsg(`${label} ERR: ${e.message ?? String(e)}`); }
  }

  return (
    <ClientOnly>
      <div className="space-y-6">
        <div data-disp-card>
          <h1 className="text-2xl font-bold" style={{color:"var(--fg)"}}>Disputas — Publisher</h1>
          <p className="text-sm" style={{color:"var(--muted)"}}>Tab separado /dashboard/client/disputes (raise/accept/evidence/assignArbiter/resolve) · useDisputeStore → api/disputes/* · En disputas {disputed.length}</p>
        </div>

        <div data-disp-card className="card space-y-3">
          <div className="flex flex-wrap gap-2">
            <input value={jobId} onChange={e=>setJobId(e.target.value)} placeholder="Job ID (ej. 0)" className="input w-32" />
            <button onClick={()=>wrap(raise(jobId),"raise")} disabled={!jobId||loading} className="btn px-3 py-1 text-xs">Raise</button>
            <button onClick={()=>wrap(accept(jobId),"accept")} disabled={!jobId||loading} className="btn btn-secondary px-3 py-1 text-xs">Accept</button>
            <button onClick={()=>wrap(assignArbiter(jobId),"assignArbiter")} disabled={!jobId||loading} className="btn btn-ghost px-3 py-1 text-xs">AssignArbiter</button>
          </div>
          <div className="flex gap-2">
            <input value={evidenceText} onChange={e=>setEvidenceText(e.target.value)} placeholder="Evidencia texto" className="input flex-1" />
            <button onClick={async ()=>{
              if(!jobId||!evidenceText.trim()) return;
              const hash = await sha256Hex(evidenceText);
              wrap(submitEvidence(jobId, evidenceText, hash), "evidence");
            }} disabled={!jobId||loading} className="btn btn-secondary px-3 py-1 text-xs">Evidence</button>
          </div>
          <div className="flex items-center gap-2">
            <input type="range" min={0} max={100} value={pct} onChange={e=>setPct(parseInt(e.target.value))} className="flex-1"/>
            <span className="text-xs font-mono" style={{color:"var(--fg)"}}>{pct}% client</span>
            <button onClick={()=>wrap(resolve(jobId,pct),"resolve")} disabled={!jobId||loading} className="btn px-3 py-1 text-xs">Resolve</button>
          </div>
          {msg && <motion.p initial={{opacity:0,y:6}} animate={{opacity:1,y:0}} className="rounded-xl border p-2 text-xs font-mono break-all" style={{borderColor:"var(--border)", background:"var(--surface-2)", color:"var(--fg)"}}>{msg}</motion.p>}
        </div>

        <div data-disp-card className="card">
          <div className="text-sm font-semibold" style={{color:"var(--fg)"}}>Jobs en disputa ({disputed.length})</div>
          <div className="mt-2 grid gap-2 md:grid-cols-2">
            {disputed.length===0 ? <p className="text-sm" style={{color:"var(--muted)"}}>Sin disputas. Jobs totales {jobs.length}. Usa raise para abrir.</p> : disputed.map(j=>(
              <div key={j.jobId} className="rounded-xl border p-3" style={{borderColor:"var(--border)", background:"var(--surface-2)"}}>
                <div className="text-sm font-semibold" style={{color:"var(--fg)"}}>#{j.jobId} {j.title}</div>
                <div className="text-xs" style={{color:"var(--muted)"}}>{j.status} · {Number(j.amount)/1e9} SOL · {j.client.slice(0,12)}…</div>
                <button onClick={()=>setJobId(j.jobId)} className="btn btn-ghost mt-2 px-3 py-1 text-xs">Seleccionar</button>
              </div>
            ))}
          </div>
          <div className="mt-3 grid gap-2">
            {jobs.slice(0,4).map(j=>(
              <div key={j.jobId} className="flex items-center justify-between rounded-xl border p-2 text-xs" style={{borderColor:"var(--border)", background:"var(--surface)"}}>
                <span style={{color:"var(--fg)"}}>#{j.jobId} {j.title.slice(0,24)} · {j.status}</span>
                <button onClick={()=>setJobId(j.jobId)} className="btn btn-secondary px-2 py-0.5 text-xs">Usar</button>
              </div>
            ))}
          </div>
        </div>
      </div>
    </ClientOnly>
  );
}
async function sha256Hex(text:string):Promise<string>{
  const enc=new TextEncoder().encode(text);
  const buf=await crypto.subtle.digest("SHA-256", enc as unknown as BufferSource);
  return Array.from(new Uint8Array(buf)).map(b=>b.toString(16).padStart(2,"0")).join("");
}
