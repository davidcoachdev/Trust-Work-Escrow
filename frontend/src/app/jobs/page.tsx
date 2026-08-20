"use client";

import { useEffect, useMemo, useState } from "react";
import Link from "next/link";
import { motion, AnimatePresence } from "framer-motion";
import gsap from "gsap";
import { JobCard } from "@/components/JobCard";
import { useJobStore } from "@/stores/useJobStore";
import type { Job } from "@/api/types";

function Skeleton() {
  return (
    <div className="grid gap-4 md:grid-cols-2">
      {[1, 2, 3, 4].map((i) => (
        <div key={i} className="card animate-pulse" style={{ background: "var(--surface)", borderColor: "var(--border)" }}>
          <div className="h-4 w-3/4 rounded" style={{ background: "var(--surface-2)" }} />
          <div className="mt-3 h-3 w-full rounded" style={{ background: "var(--surface-2)", opacity: 0.7 }} />
          <div className="mt-2 h-3 w-5/6 rounded" style={{ background: "var(--surface-2)", opacity: 0.5 }} />
          <div className="mt-4 h-3 w-1/2 rounded" style={{ background: "var(--surface-2)" }} />
        </div>
      ))}
    </div>
  );
}

export default function JobsPage() {
  const { jobs, hasMore, nextCursor, loading, error, fetchJobs, clearError } = useJobStore();
  const [query, setQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState<"all" | Job["status"]>("all");

  useEffect(() => {
    fetchJobs({ cursor: null }).catch(() => {});
  }, [fetchJobs]);

  useEffect(() => {
    const ctx = gsap.context(() => {
      gsap.from("[data-jobs-header]", { y: 14, opacity: 0, duration: 0.6, ease: "power3.out" });
      gsap.from("[data-jobs-filters]", { y: 12, opacity: 0, duration: 0.5, delay: 0.12, ease: "power3.out" });
    });
    return () => ctx.revert();
  }, []);

  async function load(c: string | null = null, append = false) {
    try {
      if (append && nextCursor) await fetchJobs({ cursor: nextCursor });
      else await fetchJobs({ cursor: c });
    } catch {}
  }

  const filtered = useMemo(() => {
    let out = jobs;
    if (query.trim()) {
      const q = query.toLowerCase();
      out = out.filter((j) => j.title.toLowerCase().includes(q) || j.description.toLowerCase().includes(q) || j.jobId.includes(q));
    }
    if (statusFilter !== "all") out = out.filter((j) => j.status === statusFilter);
    return out;
  }, [jobs, query, statusFilter]);

  return (
    <div className="space-y-6">
      <div data-jobs-header className="flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight" style={{ color: "var(--fg)" }}>
            Jobs
          </h1>
          <p className="text-sm" style={{ color: "var(--muted)" }}>
            Explora trabajos con escrow on-chain · <span className="font-mono text-xs" style={{ color: "var(--fg)" }}>useJobStore → api/jobs/list</span> · Zustand como fuente de la verdad
          </p>
        </div>
        <div className="flex items-center gap-2">
          <span
            className="rounded-full border px-3 py-1 text-xs"
            style={{ borderColor: "var(--border)", background: "var(--surface)", color: "var(--muted)" }}
          >
            {filtered.length} / {jobs.length} jobs
          </span>
          <Link href="/create" className="btn hidden md:inline-flex">
            + Crear job
          </Link>
        </div>
      </div>

      <motion.div
        data-jobs-filters
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.45 }}
        className="card flex flex-col gap-3 md:flex-row md:items-center md:justify-between"
      >
        <div className="flex flex-1 items-center gap-2">
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Buscar por título, descripción o #id…"
            className="input flex-1"
            aria-label="Buscar jobs"
          />
          {query && (
            <motion.button
              whileHover={{ scale: 1.04 }}
              whileTap={{ scale: 0.96 }}
              onClick={() => setQuery("")}
              className="btn btn-secondary shrink-0"
              aria-label="Limpiar búsqueda"
            >
              ×
            </motion.button>
          )}
        </div>
        <div className="flex items-center gap-2">
          <label className="text-xs font-medium" style={{ color: "var(--muted)" }}>
            Estado:
          </label>
          <select
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value as typeof statusFilter)}
            className="input w-36"
            aria-label="Filtrar por estado"
          >
            <option value="all">Todos</option>
            <option value="Open">Open</option>
            <option value="InProgress">InProgress</option>
            <option value="Completed">Completed</option>
            <option value="Disputed">Disputed</option>
            <option value="Cancelled">Cancelled</option>
          </select>
          <motion.button
            whileHover={{ scale: 1.03, rotate: 90 }}
            whileTap={{ scale: 0.95 }}
            transition={{ type: "spring", stiffness: 400 }}
            onClick={() => load(null, false)}
            disabled={loading}
            className="btn btn-secondary"
            aria-label="Refrescar"
          >
            ↻
          </motion.button>
        </div>
      </motion.div>

      <AnimatePresence mode="wait">
        {error && (
          <motion.div
            initial={{ opacity: 0, y: -6 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0 }}
            className="rounded-[16px] border p-4 text-sm"
            style={{ borderColor: "rgba(255,60,60,0.4)", background: "rgba(255,60,60,0.08)", color: "#FF8A8A" }}
            role="alert"
          >
            <div className="flex items-start justify-between gap-3">
              <span>Error al cargar jobs: {error}</span>
              <button
                onClick={() => {
                  clearError();
                  load(null, false);
                }}
                className="btn btn-secondary shrink-0"
              >
                Reintentar
              </button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {loading && jobs.length === 0 ? (
        <Skeleton />
      ) : filtered.length === 0 ? (
        <motion.div initial={{ opacity: 0, scale: 0.98 }} animate={{ opacity: 1, scale: 1 }} className="card text-center">
          <p className="text-sm" style={{ color: "var(--muted)" }}>
            {jobs.length === 0 ? "No hay jobs aún. Crea el primero en /create." : "Sin resultados para tu búsqueda."}
          </p>
          <div className="mt-4 flex justify-center gap-2">
            {jobs.length === 0 ? (
              <Link href="/create" className="btn">
                Crear job
              </Link>
            ) : (
              <button
                onClick={() => {
                  setQuery("");
                  setStatusFilter("all");
                }}
                className="btn btn-secondary"
              >
                Limpiar filtros
              </button>
            )}
          </div>
        </motion.div>
      ) : (
        <motion.div
          initial="hidden"
          animate="visible"
          variants={{ hidden: {}, visible: { transition: { staggerChildren: 0.06 } } }}
          className="grid gap-4 md:grid-cols-2"
        >
          {filtered.map((j, idx) => (
            <JobCard key={j.jobId} job={j} index={idx} />
          ))}
        </motion.div>
      )}

      {hasMore && filtered.length > 0 && (
        <div className="flex justify-center">
          <motion.button
            whileHover={{ scale: 1.03, y: -1 }}
            whileTap={{ scale: 0.97 }}
            onClick={() => load(nextCursor, true)}
            disabled={loading}
            className="btn"
            aria-label="Cargar más jobs"
          >
            {loading ? "Cargando…" : "Cargar más"}
          </motion.button>
        </div>
      )}

      <p className="text-center text-xs" style={{ color: "var(--muted)" }}>
        Backend: <span className="font-mono" style={{ color: "var(--fg)" }}>{process.env.NEXT_PUBLIC_API_URL ?? "http://127.0.0.1:3000"}</span> · Zustand store + api/jobs · on-chain Vec + off-chain metadata · Programa{" "}
        <span className="font-mono" style={{ color: "var(--fg)" }}>7a2Yh…5Vh</span>
      </p>
    </div>
  );
}
