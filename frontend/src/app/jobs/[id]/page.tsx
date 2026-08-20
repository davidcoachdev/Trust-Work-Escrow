"use client";

import { useEffect, useState } from "react";
import { useParams } from "next/navigation";
import { useWallet } from "@solana/wallet-adapter-react";
import { get_job, apply, proposalHashFromText, type Job } from "@/lib/sdk";

export default function JobDetailPage() {
  const params = useParams<{ id: string }>();
  const id = params.id;
  const { publicKey } = useWallet();
  const [job, setJob] = useState<Job | null>(null);
  const [proposal, setProposal] = useState("");
  const [msg, setMsg] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [sending, setSending] = useState(false);

  useEffect(() => {
    (async () => {
      const j = await get_job(id);
      setJob(j);
      setLoading(false);
    })();
  }, [id]);

  async function onApply() {
    if (!publicKey) {
      setMsg("Conecta tu wallet primero.");
      return;
    }
    if (!proposal.trim()) {
      setMsg("Escribe una propuesta.");
      return;
    }
    setSending(true);
    setMsg(null);
    try {
      const hash = proposalHashFromText(proposal);
      const res = await apply({
        client: job!.client,
        jobId: Number(id),
        applicationIndex: 0,
        proposalHash: hash,
        proposalText: proposal,
      });
      setMsg(`Aplicación enviada · sig: ${res.signature.slice(0, 24)}…`);
    } catch (e: unknown) {
      setMsg(e instanceof Error ? e.message : String(e));
    } finally {
      setSending(false);
    }
  }

  if (loading) return <div className="card text-sm text-zinc-500">Cargando job #{id}…</div>;
  if (!job) return <div className="card text-sm text-zinc-500">Job #{id} no encontrado.</div>;

  return (
    <div className="space-y-6">
      <div className="card">
        <div className="flex items-start justify-between gap-4">
          <h1 className="text-2xl font-bold">{job.title}</h1>
          <span className="rounded-full bg-emerald-50 px-3 py-1 text-xs font-medium text-emerald-700">{job.status}</span>
        </div>
        <p className="mt-3 text-sm text-zinc-600 whitespace-pre-wrap">{job.description}</p>
        <div className="mt-4 grid gap-2 text-sm">
          <div>
            <span className="label">Job ID</span>
            <div className="font-mono text-xs">{job.jobId}</div>
          </div>
          <div>
            <span className="label">Cliente</span>
            <div className="font-mono text-xs break-all">{job.client}</div>
          </div>
          <div className="flex gap-6">
            <div>
              <span className="label">Monto</span>
              <div className="font-mono text-xs">{Number(job.amount) / 1e9} SOL</div>
            </div>
            <div>
              <span className="label">Deadline</span>
              <div className="text-xs">{new Date(job.deadline * 1000).toLocaleString()}</div>
            </div>
          </div>
        </div>
      </div>

      <div className="card space-y-3">
        <h2 className="font-semibold">Aplicar a este job</h2>
        <p className="text-xs text-zinc-500">
          Llama a <span className="font-mono">sdk.apply</span> (apply_to_job) — requiere wallet conectada. Hash no puede ser cero
          (EmptyProposal).
        </p>
        <label className="label">Propuesta</label>
        <textarea
          className="input min-h-28"
          placeholder="Describe tu propuesta, timeline y entregables…"
          value={proposal}
          onChange={(e) => setProposal(e.target.value)}
        />
        <button onClick={onApply} disabled={sending} className="btn">
          {sending ? "Enviando…" : "Aplicar"}
        </button>
        {!publicKey && <p className="text-xs text-amber-600">Conecta wallet arriba para aplicar.</p>}
        {msg && <p className="rounded-xl bg-zinc-50 p-3 text-xs font-mono break-all">{msg}</p>}
      </div>
    </div>
  );
}
