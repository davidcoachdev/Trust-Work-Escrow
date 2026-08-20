import Link from "next/link";
import type { Job } from "@/api/types";

const statusStyles: Record<string, string> = {
  Open: "bg-emerald-50 text-emerald-700 border-emerald-200",
  InProgress: "bg-blue-50 text-blue-700 border-blue-200",
  Completed: "bg-violet-50 text-violet-700 border-violet-200",
  Disputed: "bg-amber-50 text-amber-700 border-amber-200",
  Cancelled: "bg-zinc-100 text-zinc-600 border-zinc-200",
};

export function JobCard({ job }: { job: Job }) {
  const overdue = job.deadline * 1000 < Date.now() && job.status === "Open";
  return (
    <Link
      href={`/jobs/${job.jobId}`}
      className="group block rounded-2xl border border-zinc-200 bg-white p-5 transition hover:border-zinc-300 hover:shadow-sm"
    >
      <div className="flex items-start justify-between gap-3">
        <h3 className="line-clamp-2 font-semibold leading-tight text-zinc-900 group-hover:text-zinc-950">{job.title}</h3>
        <span className={`shrink-0 rounded-full border px-2.5 py-1 text-xs font-medium ${statusStyles[job.status] ?? "bg-zinc-50 text-zinc-600"}`}>
          {job.status}
        </span>
      </div>
      <p className="mt-2 line-clamp-2 min-h-10 text-sm text-zinc-600">{job.description || "Sin descripción"}</p>
      <div className="mt-4 flex flex-wrap items-center gap-2 text-xs text-zinc-500">
        <span className="rounded-full bg-zinc-900 px-2.5 py-1 font-mono text-white">{Number(job.amount) / 1e9} SOL</span>
        <span className="font-mono">#{job.jobId}</span>
        <span>·</span>
        <span className={overdue ? "text-red-600 font-medium" : ""}>{new Date(job.deadline * 1000).toLocaleDateString()} {overdue ? "· vencido" : ""}</span>
      </div>
      <div className="mt-2 truncate font-mono text-[10px] text-zinc-400">{job.client.slice(0, 16)}…</div>
    </Link>
  );
}
