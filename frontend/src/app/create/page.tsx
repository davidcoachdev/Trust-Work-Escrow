"use client";

import { useMemo, useState } from "react";
import { useRouter } from "next/navigation";
import { useWallet } from "@solana/wallet-adapter-react";
import { create_job, SdkError, MAX_TITLE_LEN, MAX_DESC_LEN } from "@/lib/sdk";

export default function CreatePage() {
  const router = useRouter();
  const { publicKey } = useWallet();
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [amountSol, setAmountSol] = useState("0.5");
  const [deadlineDays, setDeadlineDays] = useState("7");
  const [msg, setMsg] = useState<string | null>(null);
  const [msgType, setMsgType] = useState<"success" | "error" | "info">("info");
  const [sending, setSending] = useState(false);
  const [touched, setTouched] = useState<Record<string, boolean>>({});

  const amountNum = useMemo(() => {
    const n = parseFloat(amountSol);
    return Number.isFinite(n) ? n : NaN;
  }, [amountSol]);

  const amountLamports = useMemo(() => {
    if (!Number.isFinite(amountNum) || amountNum <= 0) return 0;
    return Math.round(amountNum * 1e9);
  }, [amountNum]);

  const feeLamports = useMemo(() => Math.floor(amountLamports * 250 / 10000), [amountLamports]);

  // Inline validation
  const errors = useMemo(() => {
    const e: Record<string, string> = {};
    if (touched.title || title) {
      if (!title.trim()) e.title = "Título requerido";
      else if (title.trim().length > MAX_TITLE_LEN) e.title = `Máximo ${MAX_TITLE_LEN} caracteres`;
    }
    if (description.length > MAX_DESC_LEN) e.description = `Máximo ${MAX_DESC_LEN} caracteres`;
    if (touched.amountSol || amountSol) {
      if (!Number.isFinite(amountNum) || amountNum <= 0) e.amountSol = "Monto debe ser > 0";
      else if (amountNum > 10000) e.amountSol = "Máximo 10,000 SOL";
    }
    const days = parseInt(deadlineDays, 10);
    if (!Number.isFinite(days) || days < 1) e.deadlineDays = "Mínimo 1 día";
    else if (days > 365 * 5) e.deadlineDays = "Máximo 5 años (1825 días)";
    if (!publicKey) e.wallet = "Conecta tu wallet para crear";
    return e;
  }, [title, description, amountSol, deadlineDays, amountNum, publicKey, touched]);

  const isValid = Object.keys(errors).filter((k) => k !== "wallet").length === 0 && title.trim().length > 0 && Number.isFinite(amountNum) && amountNum > 0;

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setTouched({ title: true, amountSol: true, deadlineDays: true });
    if (!publicKey) {
      setMsgType("error");
      setMsg("Conecta tu wallet primero.");
      return;
    }
    if (!isValid) {
      setMsgType("error");
      setMsg("Corrige los errores del formulario.");
      return;
    }
    setSending(true);
    setMsg(null);
    try {
      const deadline = Math.floor(Date.now() / 1000) + parseInt(deadlineDays, 10) * 86400;
      const jobId = Math.floor(Date.now() / 1000) % 100000;
      const res = await create_job({ jobId, amount: amountLamports, deadline, title: title.trim(), description: description.trim() });
      setMsgType("success");
      setMsg(`Job creado #${res.job.jobId} · sig: ${res.signature.slice(0, 28)}…`);
      setTimeout(() => router.push(`/jobs/${res.job.jobId}`), 1100);
    } catch (err: unknown) {
      const message = err instanceof SdkError ? `${err.message}${err.code ? ` (${err.code})` : ""}` : err instanceof Error ? err.message : String(err);
      setMsgType("error");
      setMsg(message);
    } finally {
      setSending(false);
    }
  }

  return (
    <div className="mx-auto max-w-2xl space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Crear job</h1>
        <p className="text-sm text-zinc-600">
          Publica un trabajo con escrow on-chain. Llama a <span className="font-mono">sdk.create_job</span> → <span className="font-mono text-xs">POST /jobs</span> en el backend. Requiere wallet conectada.
        </p>
      </div>

      {/* Preview fee */}
      <div className="rounded-2xl border border-zinc-200 bg-zinc-50 p-4 text-xs">
        <div className="flex flex-wrap gap-4">
          <span><span className="font-semibold">Monto:</span> {Number.isFinite(amountNum) ? `${amountNum} SOL` : "—"} <span className="text-zinc-500">({amountLamports.toLocaleString()} lamports)</span></span>
          <span><span className="font-semibold">Fee 2.5%:</span> {(feeLamports / 1e9).toFixed(4)} SOL</span>
          <span><span className="font-semibold">Recibe freelancer:</span> {Number.isFinite(amountNum) ? `${(amountNum - feeLamports / 1e9).toFixed(4)} SOL` : "—"}</span>
        </div>
      </div>

      <form onSubmit={onSubmit} className="card space-y-5" noValidate>
        <div>
          <label className="label flex justify-between">
            <span>Título <span className="text-red-500">*</span></span>
            <span className={`text-xs ${title.length > MAX_TITLE_LEN ? "text-red-600" : "text-zinc-400"}`}>{title.length}/{MAX_TITLE_LEN}</span>
          </label>
          <input
            className={`input ${errors.title ? "border-red-300 focus:border-red-400" : ""}`}
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            onBlur={() => setTouched((p) => ({ ...p, title: true }))}
            placeholder="Ej. Landing DeFi — Next.js + Solana"
            required
            maxLength={MAX_TITLE_LEN + 20}
            aria-invalid={!!errors.title}
            aria-describedby={errors.title ? "err-title" : undefined}
          />
          {errors.title && <p id="err-title" className="mt-1 text-xs text-red-600">{errors.title}</p>}
        </div>

        <div>
          <label className="label flex justify-between">
            <span>Descripción</span>
            <span className={`text-xs ${description.length > MAX_DESC_LEN ? "text-red-600" : "text-zinc-400"}`}>{description.length}/{MAX_DESC_LEN}</span>
          </label>
          <textarea
            className={`input min-h-28 ${errors.description ? "border-red-300" : ""}`}
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="Detalles, entregables, criterios de aceptación, links de referencia…"
            maxLength={MAX_DESC_LEN + 50}
            aria-invalid={!!errors.description}
          />
          {errors.description ? <p className="mt-1 text-xs text-red-600">{errors.description}</p> : <p className="mt-1 text-xs text-zinc-400">Opcional pero recomendado. Máx {MAX_DESC_LEN} caracteres.</p>}
        </div>

        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <div>
            <label className="label">Monto (SOL) <span className="text-red-500">*</span></label>
            <input
              className={`input ${errors.amountSol ? "border-red-300" : ""}`}
              type="number"
              step="0.01"
              min="0"
              value={amountSol}
              onChange={(e) => setAmountSol(e.target.value)}
              onBlur={() => setTouched((p) => ({ ...p, amountSol: true }))}
              required
              aria-invalid={!!errors.amountSol}
            />
            {errors.amountSol && <p className="mt-1 text-xs text-red-600">{errors.amountSol}</p>}
          </div>
          <div>
            <label className="label">Deadline (días) <span className="text-red-500">*</span></label>
            <input
              className={`input ${errors.deadlineDays ? "border-red-300" : ""}`}
              type="number"
              min="1"
              max="1825"
              value={deadlineDays}
              onChange={(e) => setDeadlineDays(e.target.value)}
              onBlur={() => setTouched((p) => ({ ...p, deadlineDays: true }))}
              required
              aria-invalid={!!errors.deadlineDays}
            />
            {errors.deadlineDays ? <p className="mt-1 text-xs text-red-600">{errors.deadlineDays}</p> : <p className="mt-1 text-xs text-zinc-400">Días desde hoy hasta vencimiento.</p>}
          </div>
        </div>

        <button type="submit" disabled={sending || !isValid} className="btn w-full" aria-label="Crear job">
          {sending ? "Creando…" : publicKey ? "Crear job" : "Conecta wallet para crear"}
        </button>

        {!publicKey && <p className="rounded-xl bg-amber-50 p-3 text-xs text-amber-700">Conecta wallet arriba para crear. En test/dev puedes crear sin signer real (fallback mock).</p>}

        {msg && (
          <div
            className={`rounded-xl p-3 text-xs font-mono break-all ${msgType === "success" ? "bg-emerald-50 text-emerald-800 border border-emerald-200" : msgType === "error" ? "bg-red-50 text-red-700 border border-red-200" : "bg-zinc-50 text-zinc-700 border"}`}
            role={msgType === "error" ? "alert" : "status"}
          >
            {msg}
          </div>
        )}
      </form>

      <p className="text-xs text-zinc-400">
        Programa: <span className="font-mono">7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh</span> · Cluster: {process.env.NEXT_PUBLIC_CLUSTER ?? "localnet"} · API:{" "}
        <span className="font-mono">{process.env.NEXT_PUBLIC_API_URL ?? "http://127.0.0.1:3000"}</span>
      </p>
    </div>
  );
}
