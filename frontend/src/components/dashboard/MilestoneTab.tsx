"use client";
import { useState } from "react";
import { motion } from "framer-motion";
import { useMilestoneStore } from "@/stores/useMilestoneStore";

export function MilestoneTab({ jobId }: { jobId: string }) {
  const { milestones, create, submit, approve, reject, loading } = useMilestoneStore();
  const [title,setTitle]=useState(""); const [desc,setDesc]=useState(""); const [amount,setAmount]=useState("0.1");
  const [msg,setMsg]=useState<string|null>(null);
  async function onCreate(){
    try{ await create(jobId, {title, description:desc, amount: Math.round(parseFloat(amount)*1e9)}); setMsg("Milestone creado"); setTitle(""); setDesc(""); }
    catch(e:any){ setMsg(e.message ?? String(e)); }
  }
  return (
    <div className="space-y-4">
      <div className="grid gap-2 md:grid-cols-3">
        <input value={title} onChange={e=>setTitle(e.target.value)} placeholder="Título" className="input" />
        <input value={amount} onChange={e=>setAmount(e.target.value)} placeholder="SOL" className="input" type="number" step="0.01"/>
        <motion.button whileTap={{scale:0.98}} onClick={onCreate} disabled={loading} className="btn">Crear milestone</motion.button>
      </div>
      <textarea value={desc} onChange={e=>setDesc(e.target.value)} placeholder="Descripción" className="input min-h-20" />
      <div className="space-y-2">
        {milestones.length===0 ? <p className="text-sm" style={{color:"var(--muted)"}}>Sin milestones</p> : milestones.map(m=>(
          <div key={m.index} className="flex flex-wrap items-center justify-between gap-2 rounded-[12px] border p-3" style={{borderColor:"var(--border)", background:"var(--surface-2)"}}>
            <div><div className="text-sm font-semibold" style={{color:"var(--fg)"}}>#{m.index} {m.title}</div><div className="text-xs" style={{color:"var(--muted)"}}>{m.description} · {m.amount/1e9} SOL · {m.status}</div></div>
            <div className="flex gap-1">
              <button onClick={()=>submit(jobId,m.index).then(()=>setMsg(`Submitted #${m.index}`)).catch(e=>setMsg(String(e)))} className="btn btn-ghost px-3 py-1 text-xs">Submit</button>
              <button onClick={()=>approve(jobId,m.index).then(()=>setMsg(`Approved #${m.index}`)).catch(e=>setMsg(String(e)))} className="btn px-3 py-1 text-xs">Approve</button>
              <button onClick={()=>reject(jobId,m.index).then(()=>setMsg(`Rejected #${m.index}`)).catch(e=>setMsg(String(e)))} className="btn btn-secondary px-3 py-1 text-xs">Reject</button>
            </div>
          </div>
        ))}
      </div>
      {msg && <p className="rounded-xl border p-2 text-xs font-mono" style={{borderColor:"var(--border)", background:"var(--surface)", color:"var(--fg)"}}>{msg}</p>}
      <p className="text-xs" style={{color:"var(--muted)"}}>Consume stores/useMilestoneStore → api/milestones/* (create/submit/approve/reject)</p>
    </div>
  );
}
