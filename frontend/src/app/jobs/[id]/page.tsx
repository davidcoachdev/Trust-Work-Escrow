"use client";

import { useEffect, useState } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
import { useWallet } from "@solana/wallet-adapter-react";
import { motion, AnimatePresence } from "framer-motion";
import gsap from "gsap";
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

  useEffect(() => {
    if (!initialLoading && job) {
      const ctx = gsap.context(() => {
        gsap.from("[data-detail-card]", { y: 18, opacity: 0, duration: 0.6, stagger: 0.08, ease: "power3.out" });
      });
      return () => ctx.revert();
    }
  }, [initialLoading, job]);

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
      await apply({ jobId: Number(id), proposal, proposalHash: hash });
      setMsgType("success");
      setMsg(`Aplicación enviada · hash ${hash.slice(0, 16)}… · POST /jobs/${id}/apply OK`);
      gsap.to("[data-apply-card]", { scale: 1.01, duration: 0.18, yoyo: true, repeat: 1, ease: "power2.inOut" });
    } catch (e: unknown) {
      setMsgType("error");
      const m = e instanceof ApiError ? `${e.message}${e.code ? ` (${e.code})` : ""}` : e instanceof Error ? e.message : String(e);
      setMsg(m);
    }
  }

  if (initialLoading || jobLoading) return <div className="card animate-pulse text-sm" style={{ color: "var(--muted)" }}>Cargando job #{id}… (store → api/jobs/get)</div>;
  if (loadingError) return <div className="card text-sm" style={{ borderColor: "rgba(255,60,60,0.4)", background: "rgba(255,60,60,0.08)", color: "#FF8A8A" }}>Error: {loadingError} · <Link href="/jobs" className="underline">Volver a jobs</Link></div>;
  if (!job) return <div className="card text-center text-sm" style={{ color: "var(--muted)" }}>Job #{id} no encontrado. <Link href="/jobs" className="underline" style={{ color: "var(--primary)" }}>Volver</Link></div>;

  const isOpen = job.status === "Open";
  const charsLeft = MAX_PROPOSAL_LEN - proposal.length;
  const busy = applyLoading;

  return (
    <div className="space-y-6">
      <motion.div initial={{ opacity: 0, x: -8 }} animate={{ opacity: 1, x: 0 }} transition={{ duration: 0.35 }}>
        <Link href="/jobs" className="inline-flex text-xs transition hover:underline" style={{ color: "var(--muted)" }}>← Volver a jobs</Link>
      </motion.div>

      <motion.div
        data-detail-card
        initial={{ opacity: 0, y: 16 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.55, ease: [0.25, 0.1, 0.25, 1] }}
        className="card"
      >
        <div className="flex items-start justify-between gap-4">
          <h1 className="text-2xl font-bold tracking-tight" style={{ color: "var(--fg)" }}>{job.title}</h1>
          <motion.span
            initial={{ scale: 0.9, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            transition={{ delay: 0.25, type: "spring", stiffness: 400 }}
            className="shrink-0 rounded-full px-3 py-1 text-xs font-semibold"
            style={isOpen ? { background: "rgba(255,60,60,0.12)", color: "var(--primary)", border: "1px solid rgba(255,60,60,0.3)" } : { background: "var(--surface-2)", color: "var(--muted)", border: "1px solid var(--border)" }}
          >
            {job.status}
          </motion.span>
        </div>
        <p className="mt-3 whitespace-pre-wrap text-sm" style={{ color: "var(--muted)" }}>{job.description || "Sin descripción"}</p>
        <div className="mt-4 grid gap-3 text-sm md:grid-cols-2">
          <div>
            <span className="label">Job ID</span>
            <div className="font-mono text-xs" style={{ color: "var(--fg)" }}>{job.jobId}</div>
          </div>
          <div>
            <span className="label">Cliente</span>
            <div className="break-all font-mono text-xs" style={{ color: "var(--fg)" }}>{job.client}</div>
          </div>
          <div>
            <span className="label">Monto</span>
            <div className="font-mono text-xs" style={{ color: "var(--fg)" }}>{Number(job.amount) / 1e9} SOL <span style={{ color: "var(--muted)" }}>· {job.amount} lamports</span></div>
          </div>
          <div>
            <span className="label">Deadline</span>
            <div className="text-xs" style={{ color: "var(--muted)" }}>{new Date(job.deadline * 1000).toLocaleString()} {job.deadline * 1000 < Date.now() && <span style={{ color: "var(--primary)" }}>(vencido)</span>}</div>
          </div>
          {job.freelancer && (
            <div>
              <span className="label">Freelancer</span>
              <div className="break-all font-mono text-xs" style={{ color: "var(--fg)" }}>{job.freelancer}</div>
            </div>
          )}
        </div>
        <div className="mt-5 h-px w-full" style={{ background: "var(--gradient)", opacity: 0.3 }} />
      </motion.div>

      <motion.div
        data-detail-card
        data-apply-card
        initial={{ opacity: 0, y: 16 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.55, delay: 0.12, ease: [0.25, 0.1, 0.25, 1] }}
        className="card space-y-3"
      >
        <div className="flex items-center justify-between">
          <h2 className="font-semibold" style={{ color: "var(--fg)" }}>Aplicar a este job</h2>
          {!isOpen && <span className="rounded-full px-2.5 py-1 text-xs" style={{ background: "rgba(255,200,80,0.1)", color: "#FFC857", border: "1px solid rgba(255,200,80,0.2)" }}>Solo jobs Open aceptan aplicaciones</span>}
        </div>
        <p className="text-xs" style={{ color: "var(--muted)" }}>
          Zustand <span className="font-mono" style={{ color: "var(--fg)" }}>useApplicationStore.apply</span> → <span className="font-mono text-xs" style={{ color: "var(--fg)" }}>api/applications/apply → POST /jobs/:id/apply</span> (SDK: proposal_hash on-chain + proposal off-chain).
        </p>
        <div>
          <label className="label flex justify-between">
            <span style={{ color: "var(--muted)" }}>Propuesta</span>
            <span className="text-xs" style={{ color: charsLeft < 0 ? "var(--primary)" : "var(--muted)" }}>{proposal.length}/{MAX_PROPOSAL_LEN}</span>
          </label>
          <motion.textarea
            whileFocus={{ scale: 1.005 }}
            className="input min-h-28"
            style={charsLeft < 0 ? { borderColor: "var(--primary)" } : {}}
            placeholder="Describe tu propuesta, timeline y entregables…"
            value={proposal}
            onChange={(e) => setProposal(e.target.value)}
            maxLength={MAX_PROPOSAL_LEN + 20}
            disabled={!isOpen}
          />
        </div>
        <motion.button
          whileHover={{ scale: busy || !isOpen ? 1 : 1.015, y: busy || !isOpen ? 0 : -1 }}
          whileTap={{ scale: 0.97 }}
          onClick={onApply}
          disabled={busy || !isOpen}
          className="btn"
          aria-label="Aplicar a job"
        >
          {busy ? "Enviando…" : "Aplicar"}
        </motion.button>
        {!publicKey && <p className="text-xs" style={{ color: "#FFC857" }}>Conecta wallet arriba para aplicar.</p>}
        {!isOpen && <p className="text-xs" style={{ color: "var(--muted)" }}>Este job no está abierto.</p>}
        <AnimatePresence>
          {msg && (
            <motion.p
              initial={{ opacity: 0, y: 8, scale: 0.98 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, y: -6 }}
              className="rounded-xl p-3 text-xs font-mono break-all border"
              style={msgType === "success" ? { background: "rgba(180,255,100,0.08)", color: "#B4FF64", borderColor: "rgba(180,255,100,0.3)" } : { background: "rgba(255,60,60,0.08)", color: "#FF8A8A", borderColor: "rgba(255,60,60,0.3)" }}
              role={msgType === "error" ? "alert" : "status"}
            >
              {msg}
            </motion.p>
          )}
        </AnimatePresence>
      </motion.div>
    </div>
  );
}
