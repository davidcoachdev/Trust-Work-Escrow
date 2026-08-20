"use client";

import { useEffect, useState } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
import { useWallet } from "@solana/wallet-adapter-react";
import { useJobStore } from "@/stores/useJobStore";
import { useApplicationStore } from "@/stores/useApplicationStore";
import { ApiError } from "@/api/client";
import { MAX_PROPOSAL_LEN } from "@/api/types";

function hashProposalText(text: string): string {
  if (!text.trim()) return "0".repeat(64);
  let h = 0;
  for (let i = 0; i < text.length; i++) h = (h * 31 + text.charCodeAt(i)) >>> 0;
  return h.toString(16).padStart(64, "0").slice(0, 64);
}

export default function JobDetailPage() {
  const params = useParams<{ id: string }>();
  const id = params.id;
  const { publicKey } = useWallet();
  const { currentJob, fetchJob, loading: jobLoading } = useJobStore();
  const { apply, loading: applyLoading } = useApplicationStore();
  const [proposal, setProposal] = useState("");
  const [msg, setMsg] = useState<string | null>(null);
  const [msgType, setMsgType] = useState<"success" | "error">("error");
  const [loadingError, setLoadingError] = useState<string | null>(null);
  const [initialLoading, setInitialLoading] = useState(true);

  const job = currentJob && currentJob.jobId === id ? currentJob : null;

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        setInitialLoading(true);
        const j = await fetchJob(id);
        if (!cancelled && !j) setLoadingError(`Job #${id} no encontrado`);
      } catch (e: unknown) {
        if (!cancelled) setLoadingError(e instanceof Error ? e.message : String(e));
      } finally {
        if (!cancelled) setInitialLoading(false);
      }
    })();
    return () => { cancelled = true; };
  }, [id, fetchJob]);

  async function onApply() {
    if (!publicKey) {
      setMsgType("error");
      setMsg("Conecta tu wallet primero.");
      return;
    }
    if (!proposal.trim()) {
      setMsgType("error");
      setMsg("Escribe una propuesta (mín 10 caracteres).");
      return;
    }
    if (proposal.length > MAX_PROPOSAL_LEN) {
      setMsgType("error");
      setMsg(`Propuesta excede ${MAX_PROPOSAL_LEN} caracteres`);
      return;
    }
    if (!job) return;
    setMsg(null);
    try {
      const hash = hashProposalText(proposal);
      // Zustand → api/applications/apply.ts → POST /jobs/:id/apply (backend SDK: proposal_hash on-chain + proposal off-chain)
      await apply({ jobId: Number(id), proposal, proposalHash: hash });
      setMsgType("success");
      setMsg(`Aplicación enviada · hash ${hash.slice(0, 16)}… · POST /jobs/${id}/apply OK`);
    } catch (e: unknown) {
      setMsgType("error");
      const m = e instanceof ApiError ? `${e.message}${e.code ? ` (${e.code})` : ""}` : e instanceof Error ? e.message : String(e);
      setMsg(m);
    }
  }

  if (initialLoading || jobLoading) return <div className="card animate-pulse text-sm text-zinc-500">Cargando job #{id}… (store → api/jobs/get)</div>;
  if (loadingError) return <div className="card border-red-200 bg-red-50 text-sm text-red-700">Error: {loadingError} · <Link href="/jobs" className="underline">Volver a jobs</Link></div>;
  if (!job) return <div className="card text-center text-sm text-zinc-500">Job #{id} no encontrado. <Link href="/jobs" className="underline">Volver</Link></div>;

  const isOpen = job.status === "Open";
  const charsLeft = MAX_PROPOSAL_LEN - proposal.length;
  const busy = applyLoading;

  return (
    <div className="space-y-6">
      <Link href="/jobs" className="inline-flex text-xs text-zinc-500 hover:text-zinc-700">← Volver a jobs</Link>

      <div className="card">
        <div className="flex items-start justify-between gap-4">
          <h1 className="text-2xl font-bold tracking-tight">{job.title}</h1>
          <span className={`shrink-0 rounded-full px-3 py-1 text-xs font-medium ${isOpen ? "bg-emerald-50 text-emerald-700" : "bg-zinc-100 text-zinc-600"}`}>{job.status}</span>
        </div>
        <p className="mt-3 whitespace-pre-wrap text-sm text-zinc-600">{job.description || "Sin descripción"}</p>
        <div className="mt-4 grid gap-3 text-sm md:grid-cols-2">
          <div>
            <span className="label">Job ID</span>
            <div className="font-mono text-xs">{job.jobId}</div>
          </div>
          <div>
            <span className="label">Cliente</span>
            <div className="break-all font-mono text-xs">{job.client}</div>
          </div>
          <div>
            <span className="label">Monto</span>
            <div className="font-mono text-xs">{Number(job.amount) / 1e9} SOL <span className="text-zinc-400">· {job.amount} lamports</span></div>
          </div>
          <div>
            <span className="label">Deadline</span>
            <div className="text-xs">{new Date(job.deadline * 1000).toLocaleString()} {job.deadline * 1000 < Date.now() && <span className="text-red-600">(vencido)</span>}</div>
          </div>
          {job.freelancer && (
            <div>
              <span className="label">Freelancer</span>
              <div className="break-all font-mono text-xs">{job.freelancer}</div>
            </div>
          )}
        </div>
      </div>

      <div className="card space-y-3">
        <div className="flex items-center justify-between">
          <h2 className="font-semibold">Aplicar a este job</h2>
          {!isOpen && <span className="rounded-full bg-amber-50 px-2.5 py-1 text-xs text-amber-700">Solo jobs Open aceptan aplicaciones</span>}
        </div>
        <p className="text-xs text-zinc-500">
          Zustand <span className="font-mono">useApplicationStore.apply</span> → <span className="font-mono text-xs">api/applications/apply → POST /jobs/:id/apply</span> (SDK: proposal_hash on-chain + proposal off-chain).
        </p>
        <div>
          <label className="label flex justify-between">
            <span>Propuesta</span>
            <span className={`text-xs ${charsLeft < 0 ? "text-red-600" : "text-zinc-400"}`}>{proposal.length}/{MAX_PROPOSAL_LEN}</span>
          </label>
          <textarea
            className={`input min-h-28 ${charsLeft < 0 ? "border-red-300" : ""}`}
            placeholder="Describe tu propuesta, timeline y entregables…"
            value={proposal}
            onChange={(e) => setProposal(e.target.value)}
            maxLength={MAX_PROPOSAL_LEN + 20}
            disabled={!isOpen}
          />
        </div>
        <button onClick={onApply} disabled={busy || !isOpen} className="btn" aria-label="Aplicar a job">
          {busy ? "Enviando…" : "Aplicar"}
        </button>
        {!publicKey && <p className="text-xs text-amber-600">Conecta wallet arriba para aplicar.</p>}
        {!isOpen && <p className="text-xs text-zinc-500">Este job no está abierto.</p>}
        {msg && (
          <p className={`rounded-xl p-3 text-xs font-mono break-all border ${msgType === "success" ? "bg-emerald-50 text-emerald-800 border-emerald-200" : "bg-red-50 text-red-700 border-red-200"}`} role={msgType === "error" ? "alert" : "status"}>
            {msg}
          </p>
        )}
      </div>
    </div>
  );
}
