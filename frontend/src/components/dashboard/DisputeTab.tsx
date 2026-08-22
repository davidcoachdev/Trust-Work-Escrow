"use client";
import { useState } from "react";
import { motion } from "framer-motion";
import { useDisputeStore } from "@/stores/useDisputeStore";

export function DisputeTab({ jobId }: { jobId: string }) {
  const { dispute, raise, accept, assignArbiter, resolve, requestIntervention, platformResolve, finalize, loading } = useDisputeStore();
  const [pct,setPct]=useState(50);
  const [msg,setMsg]=useState<string|null>(null);
  async function wrap(p:Promise<any>, ok:string){ try{ const r=await p; setMsg(`${ok}: ${JSON.stringify(r).slice(0,120)}`);}catch(e:any){ setMsg(e.message ?? String(e)); } }
  return (
    <div className="space-y-3">
      <div className="card" style={{background:"var(--surface-2)"}}>
        <div className="text-sm font-semibold" style={{color:"var(--fg)"}}>Disputa · {dispute ? `${dispute.status} · arbiter ${dispute.arbiter ?? "—"} · client ${dispute.client_payout_percent}%` : "Sin disputa"}</div>
        <div className="mt-2 flex flex-wrap gap-2">
          <button onClick={()=>wrap(raise(jobId),"Raised")} disabled={loading} className="btn px-3 py-1 text-xs">Raise</button>
          <button onClick={()=>wrap(accept(jobId),"Accepted")} disabled={loading} className="btn btn-secondary px-3 py-1 text-xs">Accept</button>
          <button onClick={()=>wrap(assignArbiter(jobId),"AssignArbiter")} disabled={loading} className="btn btn-ghost px-3 py-1 text-xs">AssignArbiter</button>
          <button onClick={()=>wrap(requestIntervention(jobId),"RequestIntervention")} disabled={loading} className="btn btn-secondary px-3 py-1 text-xs">RequestIntervention</button>
        </div>
        <div className="mt-3 flex items-center gap-2">
          <input type="range" min={0} max={100} value={pct} onChange={e=>setPct(parseInt(e.target.value))} className="flex-1" />
          <span className="text-xs font-mono" style={{color:"var(--fg)"}}>{pct}% client</span>
          <button onClick={()=>wrap(resolve(jobId,pct),`Resolve ${pct}%`)} disabled={loading} className="btn px-3 py-1 text-xs">Resolve</button>
          <button onClick={()=>wrap(platformResolve(jobId,pct),`PlatformResolve ${pct}%`)} disabled={loading} className="btn btn-ghost px-3 py-1 text-xs">PlatformResolve</button>
          <button onClick={()=>wrap(finalize(jobId),"Finalize")} disabled={loading} className="btn btn-secondary px-3 py-1 text-xs">Finalize</button>
        </div>
      </div>
      {msg && <motion.p initial={{opacity:0,y:6}} animate={{opacity:1,y:0}} className="rounded-xl border p-2 text-xs font-mono break-all" style={{borderColor:"var(--border)", background:"var(--surface)", color:"var(--fg)"}}>{msg}</motion.p>}
      <p className="text-xs" style={{color:"var(--muted)"}}>Flujo completo: raise/accept/evidence/assignArbiter/resolve · useDisputeStore → api/disputes/*</p>
    </div>
  );
}
