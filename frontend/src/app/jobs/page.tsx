"use client";

import { useEffect, useMemo, useState } from "react";
import Link from "next/link";
import { JobCard } from "@/components/JobCard";
import { list_jobs, SdkError, type Job } from "@/lib/sdk";

function Skeleton() {
  return (
    <div className="grid gap-4 md:grid-cols-2">
      {[1, 2, 3, 4].map((i) => (
        <div key={i} className="card animate-pulse">
          <div className="h-4 w-3/4 rounded bg-zinc-200" />
          <div className="mt-3 h-3 w-full rounded bg-zinc-100" />
          <div className="mt-2 h-3 w-5/6 rounded bg-zinc-100" />
          <div className="mt-4 h-3 w-1/2 rounded bg-zinc-100" />
        </div>
      ))}
    </div>
  );
}

export default function JobsPage() {
  const [jobs, setJobs] = useState<Job[]>([]);
  const [cursor, setCursor] = useState<string | null>(null);
  const [hasMore, setHasMore] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState<"all" | Job["status"]>("all");

  async function load(c: string | null = null, append = false) {
    setLoading(true);
    setError(null);
    try {
      const res = await list_jobs(c, 20);
      setJobs((prev) => (append ? [...prev, ...res.jobs] : res.jobs));
      setCursor(res.nextCursor);
      setHasMore(res.hasMore);
    } catch (e: unknown) {
      const msg = e instanceof SdkError ? e.message : e instanceof Error ? e.message : String(e);
      setError(msg);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    load(null, false);
  }, []);

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
      {/* Header */}
      <div className="flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Jobs</h1>
          <p className="text-sm text-zinc-600">
            Explora trabajos con escrow on-chain · <span className="font-mono text-xs">sdk.list_jobs</span> paginado · API + fallback mock
          </p>
        </div>
        <div className="flex items-center gap-2">
          <span className="rounded-full border border-zinc-200 bg-white px-3 py-1 text-xs text-zinc-600">
            {filtered.length} / {jobs.length} jobs
          </span>
          <Link href="/create" className="btn hidden md:inline-flex">
            + Crear job
          </Link>
        </div>
      </div>

      {/* Controls */}
      <div className="card flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
        <div className="flex flex-1 items-center gap-2">
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Buscar por título, descripción o #id…"
            className="input flex-1"
            aria-label="Buscar jobs"
          />
          {query && (
            <button onClick={() => setQuery("")} className="btn btn-secondary" aria-label="Limpiar búsqueda">
              ×
            </button>
          )}
        </div>
        <div className="flex items-center gap-2">
          <label className="text-xs font-medium text-zinc-600">Estado:</label>
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
          <button onClick={() => load(null, false)} disabled={loading} className="btn btn-secondary" aria-label="Refrescar">
            ↻
          </button>
        </div>
      </div>

      {/* Error */}
      {error && (
        <div className="rounded-2xl border border-red-200 bg-red-50 p-4 text-sm text-red-700" role="alert">
          <div className="flex items-start justify-between gap-3">
            <span>Error al cargar jobs: {error}</span>
            <button onClick={() => load(null, false)} className="btn btn-secondary shrink-0">
              Reintentar
            </button>
          </div>
        </div>
      )}

      {/* Content */}
      {loading && jobs.length === 0 ? (
        <Skeleton />
      ) : filtered.length === 0 ? (
        <div className="card text-center">
          <p className="text-sm text-zinc-600">
            {jobs.length === 0 ? "No hay jobs aún. Crea el primero en /create." : "Sin resultados para tu búsqueda."}
          </p>
          <div className="mt-4 flex justify-center gap-2">
            {jobs.length === 0 ? (
              <Link href="/create" className="btn">
                Crear job
              </Link>
            ) : (
              <button onClick={() => { setQuery(""); setStatusFilter("all"); }} className="btn btn-secondary">
                Limpiar filtros
              </button>
            )}
          </div>
        </div>
      ) : (
        <div className="grid gap-4 md:grid-cols-2">
          {filtered.map((j) => (
            <JobCard key={j.jobId} job={j} />
          ))}
        </div>
      )}

      {/* Pagination */}
      {hasMore && filtered.length > 0 && (
        <div className="flex justify-center">
          <button onClick={() => load(cursor, true)} disabled={loading} className="btn" aria-label="Cargar más jobs">
            {loading ? "Cargando…" : "Cargar más"}
          </button>
        </div>
      )}

      <p className="text-center text-xs text-zinc-400">
        Backend: <span className="font-mono">{process.env.NEXT_PUBLIC_API_URL ?? "http://127.0.0.1:3000"}</span> · Programa{" "}
        <span className="font-mono">7a2Yh…5Vh</span>
      </p>
    </div>
  );
}
