import Link from "next/link";
import type { Job } from "@/lib/sdk";

export function JobCard({ job }: { job: Job }) {
  return (
    <Link
      href={`/jobs/${job.jobId}`}
      className="block rounded-2xl border border-zinc-200 p-5 hover:border-zinc-300 hover:bg-zinc-50 transition"
    >
      <div className="flex items-start justify-between gap-4">
        <h3 className="font-semibold text-zinc-900">{job.title}</h3>
        <span className="shrink-0 rounded-full bg-emerald-50 px-2.5 py-1 text-xs font-medium text-emerald-700">
          {job.status}
        </span>
      </div>
      <p className="mt-2 line-clamp-2 text-sm text-zinc-600">{job.description}</p>
      <div className="mt-4 flex items-center gap-3 text-xs text-zinc-500">
        <span className="font-mono">{Number(job.amount) / 1e9} SOL</span>
        <span>·</span>
        <span>#{job.jobId}</span>
        <span>·</span>
        <span>{new Date(job.deadline * 1000).toLocaleDateString()}</span>
      </div>
    </Link>
  );
}
