"use client";
import { useState } from "react";
import { motion } from "framer-motion";
import { useDisputeStore } from "@/stores/useDisputeStore";

export function EvidenceTab({ jobId }: { jobId: string }) {
  const { evidence, submitEvidence, loading } = useDisputeStore();
  const [text,setText]=useState("");
  const [msg,setMsg]=useState<string|null>(null);
  async function onSubmit(){
    if(!text.trim()) return;
    try{
      const hash = await sha256Hex(text);
      await submitEvidence(jobId, text, hash);
      setMsg(`Evidencia subida · hash ${hash.slice(0,16)}…`);
      setText("");
    }catch(e:any){ setMsg(e.message ?? String(e)); }
  }
  return (
    <div className="space-y-3">
      <div className="card" style={{background:"var(--surface-2)"}}>
        <div className="text-xs font-semibold" style={{color:"var(--muted)"}}>Evidencias on-chain (content_hash) · api/disputes/evidence</div>
        {evidence.length===0 ? <p className="mt-2 text-sm" style={{color:"var(--muted)"}}>Sin evidencias aún. Sube la primera.</p> : <ul className="mt-2 space-y-1">{evidence.map(ev=>(
          <li key={ev.index} className="flex items-center justify-between rounded-xl border p-2 text-xs font-mono" style={{borderColor:"var(--border)", background:"var(--surface)"}}>
            <span>#{ev.index} · {ev.author.slice(0,12)}…</span><span style={{color:"var(--muted)"}}>{ev.content_hash.slice(0,16)}…</span>
          </li>
        ))}</ul>}
      </div>
      <textarea value={text} onChange={e=>setText(e.target.value)} placeholder="Describe evidencia (se hashea sha256)…" className="input min-h-24" />
      <motion.button whileTap={{scale:0.98}} onClick={onSubmit} disabled={loading} className="btn">{loading?"Subiendo…":"Subir evidencia"}</motion.button>
      {msg && <p className="rounded-xl border p-2 text-xs font-mono" style={{borderColor:"var(--border)", background:"rgba(180,255,100,0.08)", color:"#B4FF64"}}>{msg}</p>}
    </div>
  );
}
async function sha256Hex(text:string): Promise<string>{
  const enc = new TextEncoder().encode(text);
  const buf = await crypto.subtle.digest("SHA-256", enc as unknown as BufferSource);
  return Array.from(new Uint8Array(buf)).map(b=>b.toString(16).padStart(2,"0")).join("");
}
