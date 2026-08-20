"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { useWallet } from "@solana/wallet-adapter-react";
import { create_job } from "@/lib/sdk";

export default function CreatePage() {
  const router = useRouter();
  const { publicKey } = useWallet();
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [amountSol, setAmountSol] = useState("0.5");
  const [deadlineDays, setDeadlineDays] = useState("7");
  const [msg, setMsg] = useState<string | null>(null);
  const [sending, setSending] = useState(false);

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!publicKey) {
      setMsg("Conecta tu wallet primero.");
      return;
    }
    setSending(true);
    setMsg(null);
    try {
      const amount = Math.round(parseFloat(amountSol) * 1e9);
      const deadline = Math.floor(Date.now() / 1000) + parseInt(deadlineDays, 10) * 86400;
      // jobId mock incremental; en prod viene de counter on-chain
      const jobId = Math.floor(Date.now() / 1000) % 100000;
      const res = await create_job({ jobId, amount, deadline, title, description });
      setMsg(`Job creado #${res.job.jobId} · sig: ${res.signature.slice(0, 24)}…`);
      setTimeout(() => router.push(`/jobs/${res.job.jobId}`), 900);
    } catch (err: unknown) {
      setMsg(err instanceof Error ? err.message : String(err));
    } finally {
      setSending(false);
    }
  }

  return (
    <div className="mx-auto max-w-2xl space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Crear job</h1>
        <p className="text-sm text-zinc-600">
          Llama a <span className="font-mono">sdk.create_job</span> (Anchor create_job). Requiere wallet.
        </p>
      </div>

      <form onSubmit={onSubmit} className="card space-y-4">
        <div>
          <label className="label">Título</label>
          <input className="input" value={title} onChange={(e) => setTitle(e.target.value)} placeholder="Ej. Landing DeFi" required />
        </div>
        <div>
          <label className="label">Descripción</label>
          <textarea className="input min-h-24" value={description} onChange={(e) => setDescription(e.target.value)} placeholder="Detalles, entregables…" required />
        </div>
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="label">Monto (SOL)</label>
            <input className="input" type="number" step="0.01" min="0" value={amountSol} onChange={(e) => setAmountSol(e.target.value)} required />
          </div>
          <div>
            <label className="label">Deadline (días)</label>
            <input className="input" type="number" min="1" value={deadlineDays} onChange={(e) => setDeadlineDays(e.target.value)} required />
          </div>
        </div>

        <button type="submit" disabled={sending} className="btn w-full">
          {sending ? "Creando…" : "Crear job"}
        </button>

        {!publicKey && <p className="text-xs text-amber-600">Conecta wallet arriba para crear.</p>}
        {msg && <p className="rounded-xl bg-zinc-50 p-3 text-xs font-mono break-all">{msg}</p>}
      </form>

      <p className="text-xs text-zinc-400">
        Programa: <span className="font-mono">7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh</span> · Cluster:{" "}
        {process.env.NEXT_PUBLIC_CLUSTER ?? "localnet"}
      </p>
    </div>
  );
}
