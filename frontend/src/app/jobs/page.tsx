"use client";

import { useEffect, useState } from "react";
import { JobCard } from "@/components/JobCard";
import { list_jobs, type Job } from "@/lib/sdk";

export default function JobsPage() {
  const [jobs, setJobs] = useState<Job[]>([]);
  const [cursor, setCursor] = useState<string | null>(null);
  const [hasMore, setHasMore] = useState(false);
  const [loading, setLoading] = useState(true);

  async function load(c: string | null = null, append = false) {
    setLoading(true);
    const res = await list_jobs(c, 20);
    setJobs((prev) => (append ? [...prev, ...res.jobs] : res.jobs));
    setCursor(res.nextCursor);
    setHasMore(res.hasMore);
    setLoading(false);
  }

  useEffect(() => {
    load(null, false);
  }, []);

  return (
    <div className="space-y-6">
      <div className="flex items-end justify-between">
        <div>
          <h1 className="text-2xl font-bold">Jobs</h1>
          <p className="text-sm text-zinc-600">sdk.list_jobs — paginado con cursor</p>
        </div>
        <span className="text-xs text-zinc-500">{jobs.length} jobs</span>
      </div>

      {loading && jobs.length === 0 ? (
        <div className="card text-sm text-zinc-500">Cargando…</div>
      ) : jobs.length === 0 ? (
        <div className="card text-sm text-zinc-500">No hay jobs aún. Crea el primero en /create.</div>
      ) : (
        <div className="grid gap-4 md:grid-cols-2">
          {jobs.map((j) => (
            <JobCard key={j.jobId} job={j} />
          ))}
        </div>
      )}

      {hasMore && (
        <button onClick={() => load(cursor, true)} disabled={loading} className="btn">
          {loading ? "Cargando…" : "Cargar más"}
        </button>
      )}
    </div>
  );
}
