"use client";

import { useMemo, useState, useEffect, useRef } from "react";
import { useRouter } from "next/navigation";
import { useWallet } from "@solana/wallet-adapter-react";
import { motion, AnimatePresence } from "framer-motion";
import gsap from "gsap";
import { useJobStore } from "@/stores/useJobStore";
import { ApiError } from "@/api/client";
import { MAX_TITLE_LEN, MAX_DESC_LEN } from "@/api/types";

export default function CreatePage() {
  const router = useRouter();
  const { publicKey } = useWallet();
  const { createJob, loading: storeLoading } = useJobStore();
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [amountSol, setAmountSol] = useState("0.5");
  const [deadlineDays, setDeadlineDays] = useState("7");
  const [msg, setMsg] = useState<string | null>(null);
  const [msgType, setMsgType] = useState<"success" | "error" | "info">("info");
  const [sending, setSending] = useState(false);
  const [touched, setTouched] = useState<Record<string, boolean>>({});
  const formRef = useRef<HTMLFormElement>(null);

  useEffect(() => {
    const ctx = gsap.context(() => {
      gsap.from("[data-create-header]", { y: 16, opacity: 0, duration: 0.6, ease: "power3.out" });
      gsap.from("[data-create-card]", { y: 20, opacity: 0, duration: 0.7, delay: 0.14, ease: "power3.out" });
      gsap.from("[data-create-summary]", { y: 12, opacity: 0, duration: 0.5, delay: 0.08, ease: "power3.out" });
    });
    return () => ctx.revert();
  }, []);

  const amountNum = useMemo(() => {
    const n = parseFloat(amountSol);
    return Number.isFinite(n) ? n : NaN;
  }, [amountSol]);

  const amountLamports = useMemo(() => {
    if (!Number.isFinite(amountNum) || amountNum <= 0) return 0;
    return Math.round(amountNum * 1e9);
  }, [amountNum]);

  const feeLamports = useMemo(() => Math.floor(amountLamports * 250 / 10000), [amountLamports]);

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
      gsap.fromTo("[data-msg]", { x: -6 }, { x: 0, duration: 0.35, ease: "power2.out" });
      return;
    }
    if (!isValid) {
      setMsgType("error");
      setMsg("Corrige los errores del formulario.");
      gsap.from("[data-field-error]", { x: -4, opacity: 0, duration: 0.3, stagger: 0.06 });
      return;
    }
    setSending(true);
    setMsg(null);
    try {
      const deadline = Math.floor(Date.now() / 1000) + parseInt(deadlineDays, 10) * 86400;
      const job = await createJob({ title: title.trim(), description: description.trim(), amount: amountLamports, deadline });
      setMsgType("success");
      setMsg(`Job creado #${job.jobId} · ${job.title}`);
      gsap.to(formRef.current, { scale: 1.01, duration: 0.18, yoyo: true, repeat: 1, ease: "power2.inOut" });
      setTimeout(() => router.push(`/jobs/${job.jobId}`), 900);
    } catch (err: unknown) {
      const message = err instanceof ApiError ? `${err.message}${err.code ? ` (${err.code})` : ""}` : err instanceof Error ? err.message : String(err);
      setMsgType("error");
      setMsg(message);
    } finally {
      setSending(false);
    }
  }

  const busy = sending || storeLoading;

  return (
    <div className="mx-auto max-w-2xl space-y-6">
      <div data-create-header>
        <h1 className="text-2xl font-bold tracking-tight" style={{ color: "var(--fg)" }}>
          Crear job
        </h1>
        <p className="text-sm" style={{ color: "var(--muted)" }}>
          Publica un trabajo con escrow on-chain. Zustand <span className="font-mono" style={{ color: "var(--fg)" }}>useJobStore.createJob</span> → <span className="font-mono text-xs" style={{ color: "var(--fg)" }}>api/jobs/create → POST /jobs</span> (backend SDK + on-chain Vec + off-chain title/description). Requiere wallet.
        </p>
      </div>

      <motion.div
        data-create-summary
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, delay: 0.12 }}
        className="rounded-[16px] border p-4 text-xs"
        style={{ borderColor: "var(--border)", background: "var(--surface)" }}
      >
        <div className="flex flex-wrap gap-4">
          <span style={{ color: "var(--muted)" }}>
            <span className="font-semibold" style={{ color: "var(--fg)" }}>Monto:</span> {Number.isFinite(amountNum) ? `${amountNum} SOL` : "—"}{" "}
            <span style={{ color: "var(--muted)" }}>({amountLamports.toLocaleString()} lamports)</span>
          </span>
          <span style={{ color: "var(--muted)" }}>
            <span className="font-semibold" style={{ color: "var(--fg)" }}>Fee 2.5%:</span> {(feeLamports / 1e9).toFixed(4)} SOL
          </span>
          <span style={{ color: "var(--muted)" }}>
            <span className="font-semibold" style={{ color: "var(--fg)" }}>Recibe freelancer:</span> {Number.isFinite(amountNum) ? `${(amountNum - feeLamports / 1e9).toFixed(4)} SOL` : "—"}
          </span>
        </div>
        <div className="mt-3 h-px w-full" style={{ background: "var(--gradient)", opacity: 0.35 }} />
      </motion.div>

      <form ref={formRef} onSubmit={onSubmit} data-create-card className="card space-y-5" noValidate>
        <div>
          <label className="label flex justify-between">
            <span style={{ color: "var(--muted)" }}>
              Título <span style={{ color: "var(--primary)" }}>*</span>
            </span>
            <span className="text-xs" style={{ color: title.length > MAX_TITLE_LEN ? "var(--primary)" : "var(--muted)" }}>{title.length}/{MAX_TITLE_LEN}</span>
          </label>
          <motion.input
            whileFocus={{ scale: 1.005 }}
            className="input"
            style={errors.title ? { borderColor: "var(--primary)" } : {}}
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            onBlur={() => setTouched((p) => ({ ...p, title: true }))}
            placeholder="Ej. Landing DeFi — Next.js + Solana"
            required
            maxLength={MAX_TITLE_LEN + 20}
            aria-invalid={!!errors.title}
            aria-describedby={errors.title ? "err-title" : undefined}
          />
          <AnimatePresence>
            {errors.title && <motion.p data-field-error id="err-title" initial={{ opacity: 0, y: -4 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0 }} className="mt-1 text-xs" style={{ color: "var(--primary)" }}>{errors.title}</motion.p>}
          </AnimatePresence>
        </div>

        <div>
          <label className="label flex justify-between">
            <span style={{ color: "var(--muted)" }}>Descripción</span>
            <span className="text-xs" style={{ color: description.length > MAX_DESC_LEN ? "var(--primary)" : "var(--muted)" }}>{description.length}/{MAX_DESC_LEN}</span>
          </label>
          <textarea
            className="input min-h-28"
            style={errors.description ? { borderColor: "var(--primary)" } : {}}
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="Detalles, entregables, criterios de aceptación, links de referencia…"
            maxLength={MAX_DESC_LEN + 50}
            aria-invalid={!!errors.description}
          />
          {errors.description ? <p data-field-error className="mt-1 text-xs" style={{ color: "var(--primary)" }}>{errors.description}</p> : <p className="mt-1 text-xs" style={{ color: "var(--muted)" }}>Opcional pero recomendado. Máx {MAX_DESC_LEN} caracteres.</p>}
        </div>

        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <div>
            <label className="label" style={{ color: "var(--muted)" }}>Monto (SOL) <span style={{ color: "var(--primary)" }}>*</span></label>
            <input
              className="input"
              style={errors.amountSol ? { borderColor: "var(--primary)" } : {}}
              type="number"
              step="0.01"
              min="0"
              value={amountSol}
              onChange={(e) => setAmountSol(e.target.value)}
              onBlur={() => setTouched((p) => ({ ...p, amountSol: true }))}
              required
              aria-invalid={!!errors.amountSol}
            />
            {errors.amountSol && <p data-field-error className="mt-1 text-xs" style={{ color: "var(--primary)" }}>{errors.amountSol}</p>}
          </div>
          <div>
            <label className="label" style={{ color: "var(--muted)" }}>Deadline (días) <span style={{ color: "var(--primary)" }}>*</span></label>
            <input
              className="input"
              style={errors.deadlineDays ? { borderColor: "var(--primary)" } : {}}
              type="number"
              min="1"
              max="1825"
              value={deadlineDays}
              onChange={(e) => setDeadlineDays(e.target.value)}
              onBlur={() => setTouched((p) => ({ ...p, deadlineDays: true }))}
              required
              aria-invalid={!!errors.deadlineDays}
            />
            {errors.deadlineDays ? <p data-field-error className="mt-1 text-xs" style={{ color: "var(--primary)" }}>{errors.deadlineDays}</p> : <p className="mt-1 text-xs" style={{ color: "var(--muted)" }}>Días desde hoy hasta vencimiento.</p>}
          </div>
        </div>

        <motion.button
          whileHover={{ scale: busy || !isValid ? 1 : 1.01, y: busy || !isValid ? 0 : -1 }}
          whileTap={{ scale: 0.98 }}
          type="submit"
          disabled={busy || !isValid}
          className="btn w-full"
          style={{ borderRadius: 12 }}
          aria-label="Crear job"
        >
          {busy ? "Creando…" : publicKey ? "Crear job" : "Conecta wallet para crear"}
        </motion.button>

        {!publicKey && <p className="rounded-xl p-3 text-xs" style={{ background: "rgba(255,60,60,0.08)", border: "1px solid rgba(255,60,60,0.2)", color: "var(--muted)" }}>Conecta wallet arriba para crear. En test/dev puedes crear sin signer real (fallback mock).</p>}

        <AnimatePresence>
          {msg && (
            <motion.div
              data-msg
              initial={{ opacity: 0, y: 8, scale: 0.98 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, y: -6 }}
              transition={{ type: "spring", stiffness: 400, damping: 24 }}
              className="rounded-xl p-3 text-xs font-mono break-all border"
              style={
                msgType === "success"
                  ? { background: "rgba(180,255,100,0.08)", color: "#B4FF64", borderColor: "rgba(180,255,100,0.3)" }
                  : msgType === "error"
                  ? { background: "rgba(255,60,60,0.08)", color: "#FF8A8A", borderColor: "rgba(255,60,60,0.3)" }
                  : { background: "var(--surface-2)", color: "var(--fg)", borderColor: "var(--border)" }
              }
              role={msgType === "error" ? "alert" : "status"}
            >
              {msg}
            </motion.div>
          )}
        </AnimatePresence>
      </form>

      <p className="text-xs" style={{ color: "var(--muted)" }}>
        Programa: <span className="font-mono" style={{ color: "var(--fg)" }}>7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh</span> · Zustand store + api/jobs/create · off-chain metadata (title/description) + on-chain Vec
      </p>
    </div>
  );
}
