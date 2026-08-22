"use client";
import { useEffect, useState, useMemo } from "react";
import { motion, AnimatePresence } from "framer-motion";
import gsap from "gsap";
import { useJobStore } from "@/stores/useJobStore";
import { ClientOnly } from "@/components/dashboard/RoleGuard";
import { MAX_TITLE_LEN, saveDraft, loadDraft, clearDraft, type DraftCreate } from "@/lib/dashboardUtils";
import { ApiError } from "@/api/client";
import { MAX_DESC_LEN } from "@/api/types";
import { useAuthStore } from "@/stores/useAuthStore";

export default function ClientCreatePage(){
  const { createJob, loading } = useJobStore();
  const pubkey = useAuthStore(s=>s.pubkey);
  const [title,setTitle]=useState("");
  const [description,setDescription]=useState("");
  const [amountSol,setAmountSol]=useState("0.5");
  const [deadlineDays,setDeadlineDays]=useState("7");
  const [msg,setMsg]=useState<string|null>(null);
  const [msgType,setMsgType]=useState<"success"|"error">("error");
  const [draftLoaded,setDraftLoaded]=useState(false);

  useEffect(()=>{
    const d = loadDraft();
    if(d){ setTitle(d.title); setDescription(d.description); setAmountSol(d.amountSol); setDeadlineDays(d.deadlineDays); setDraftLoaded(true); }
    const ctx = gsap.context(()=>{ gsap.from("[data-create-cc]",{ y:14, opacity:0, duration:0.5, stagger:0.08, ease:"power3.out"}); });
    return ()=>ctx.revert();
  },[]);

  useEffect(()=>{
    const draft: DraftCreate = { title, description, amountSol, deadlineDays, updatedAt: Date.now() };
    saveDraft(draft);
  },[title, description, amountSol, deadlineDays]);

  const amountLamports = useMemo(()=> Math.round((parseFloat(amountSol)||0)*1e9),[amountSol]);
  const fee = useMemo(()=> Math.floor(amountLamports*0.025),[amountLamports]);

  async function onSubmit(e:React.FormEvent){
    e.preventDefault();
    if(title.trim().length>MAX_TITLE_LEN){ setMsgType("error"); setMsg(`Título máximo ${MAX_TITLE_LEN} chars`); return; }
    if(!title.trim()){ setMsgType("error"); setMsg("Título requerido"); return; }
    if(parseFloat(amountSol)<=0){ setMsgType("error"); setMsg("Monto inválido"); return; }
    const deadline = Math.floor(Date.now()/1000) + parseInt(deadlineDays,10)*86400;
    try{
      const job = await createJob({ title:title.trim(), description:description.trim(), amount:amountLamports, deadline });
      setMsgType("success"); setMsg(`Job creado #${job.jobId} · ${job.title} · fee ${(fee/1e9).toFixed(4)} SOL`);
      clearDraft();
      gsap.to("[data-create-form]",{scale:1.01,duration:0.18, yoyo:true, repeat:1});
    }catch(err:any){
      const m = err instanceof ApiError ? err.message : err.message ?? String(err);
      setMsgType("error"); setMsg(m);
    }
  }

  return (
    <ClientOnly>
      <div className="mx-auto max-w-2xl space-y-6">
        <div data-create-cc>
          <h1 className="text-2xl font-bold" style={{color:"var(--fg)"}}>Crear Job — Publisher</h1>
          <p className="text-sm" style={{color:"var(--muted)"}}>Form title/description/amount/deadline (MAX_TITLE 100) + borradores localStorage + InMemory · useDashboardStore + useJobStore → api/jobs/create · x-pubkey {pubkey?pubkey.slice(0,10)+"…":"sin auth"}</p>
          {draftLoaded && <p className="mt-1 text-xs" style={{color:"#B4FF64"}}>✓ Borrador restaurado desde localStorage (InMemory fallback)</p>}
        </div>

        <div data-create-cc className="rounded-[16px] border p-3 text-xs" style={{borderColor:"var(--border)", background:"var(--surface)"}}>
          <span style={{color:"var(--muted)"}}>Monto {isFinite(parseFloat(amountSol))? `${parseFloat(amountSol)} SOL (${amountLamports.toLocaleString()} lamports)` :"—"} · Fee 2.5% {(fee/1e9).toFixed(4)} SOL · InMemory drafts + localStorage key twe_drafts_client_create_v3</span>
        </div>

        <form data-create-form onSubmit={onSubmit} className="card space-y-4" data-create-cc noValidate>
          <div>
            <label className="label flex justify-between"><span>Título *</span><span className="text-xs" style={{color:title.length>MAX_TITLE_LEN?"var(--primary)":"var(--muted)"}}>{title.length}/{MAX_TITLE_LEN}</span></label>
            <input value={title} onChange={e=>setTitle(e.target.value)} placeholder="Ej. Landing DeFi" className="input" maxLength={MAX_TITLE_LEN+20} style={title.length>MAX_TITLE_LEN?{borderColor:"var(--primary)"}:{}}/>
          </div>
          <div>
            <label className="label flex justify-between"><span>Descripción</span><span className="text-xs" style={{color:description.length>MAX_DESC_LEN?"var(--primary)":"var(--muted)"}}>{description.length}/{MAX_DESC_LEN}</span></label>
            <textarea value={description} onChange={e=>setDescription(e.target.value)} placeholder="Detalles, entregables, aceptación…" className="input min-h-28" maxLength={MAX_DESC_LEN+50}/>
          </div>
          <div className="grid grid-cols-2 gap-3">
            <div><label className="label">Monto (SOL) *</label><input value={amountSol} onChange={e=>setAmountSol(e.target.value)} type="number" step="0.01" className="input"/></div>
            <div><label className="label">Deadline (días) *</label><input value={deadlineDays} onChange={e=>setDeadlineDays(e.target.value)} type="number" min={1} max={1825} className="input"/></div>
          </div>
          <div className="flex gap-2">
            <motion.button whileTap={{scale:0.98}} type="submit" disabled={loading} className="btn flex-1">{loading?"Creando…":"Crear job (MAX_TITLE 100)"}</motion.button>
            <button type="button" onClick={()=>{ clearDraft(); setTitle(""); setDescription(""); setAmountSol("0.5"); setDeadlineDays("7"); setMsg("Borrador limpiado (localStorage + InMemory)"); setMsgType("error"); }} className="btn btn-secondary">Limpiar borrador</button>
          </div>
          <AnimatePresence>
            {msg && <motion.div initial={{opacity:0,y:6}} animate={{opacity:1,y:0}} exit={{opacity:0}} className="rounded-xl border p-3 text-xs font-mono break-all" style={msgType==="success"?{background:"rgba(180,255,100,0.08)", color:"#B4FF64", borderColor:"rgba(180,255,100,0.3)"}:{background:"rgba(255,60,60,0.08)", color:"#FF8A8A", borderColor:"rgba(255,60,60,0.3)"}}>{msg}</motion.div>}
          </AnimatePresence>
        </form>

        <div data-create-cc className="card" style={{background:"var(--surface-2)"}}>
          <div className="text-xs font-semibold" style={{color:"var(--muted)"}}>Borradores</div>
          <div className="mt-2 flex gap-2">
            <button onClick={()=>{ const d=loadDraft(); setMsg(d?`Draft: ${d.title.slice(0,30)} · ${d.updatedAt}`:"Sin borrador"); setMsgType("error");}} className="btn btn-ghost px-3 py-1 text-xs">Cargar draft</button>
            <button onClick={()=>{ saveDraft({title, description, amountSol, deadlineDays, updatedAt:Date.now()}); setMsg("Draft guardado localStorage + InMemory"); setMsgType("success");}} className="btn btn-secondary px-3 py-1 text-xs">Guardar draft</button>
          </div>
          <p className="mt-2 text-xs" style={{color:"var(--muted)"}}>EnMemory fallback DRAFT_MEMORY + localStorage persist · auto-guardado en cada keystroke</p>
        </div>
      </div>
    </ClientOnly>
  );
}
