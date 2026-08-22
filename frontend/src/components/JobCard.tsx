"use client";
import Link from "next/link";
import type { Job } from "@/api/types";
import { motion } from "framer-motion";

const statusStyles: Record<string, { bg: string; color: string; border: string }> = {
  Open: { bg: "rgba(255,60,60,0.12)", color: "#FF5050", border: "rgba(255,60,60,0.3)" },
  InProgress: { bg: "rgba(180,255,100,0.10)", color: "#B4FF64", border: "rgba(180,255,100,0.25)" },
  Completed: { bg: "rgba(140,70,70,0.18)", color: "#F0D2D2", border: "rgba(160,30,30,0.4)" },
  Disputed: { bg: "rgba(255,180,60,0.12)", color: "#FFC857", border: "rgba(255,180,60,0.3)" },
  Cancelled: { bg: "rgba(42,20,20,0.9)", color: "#8C4646", border: "rgba(160,30,30,0.3)" },
};

export function JobCard({ job, index = 0 }: { job: Job; index?: number }) {
  const overdue = job.deadline * 1000 < Date.now() && job.status === "Open";
  const s = statusStyles[job.status] ?? statusStyles.Cancelled;
  return (
    <motion.div
      initial={{ opacity: 0, y: 18 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: "-30px" }}
      transition={{ duration: 0.45, delay: index * 0.06, ease: [0.25, 0.1, 0.25, 1] }}
      whileHover={{ y: -3, transition: { type: "spring", stiffness: 400, damping: 20 } }}
      className="h-full"
    >
      <Link
        href={`/jobs/${job.jobId}`}
        className="group flex h-full flex-col rounded-[16px] p-5 transition"
        style={{
          background: "var(--surface)",
          border: "1px solid var(--border)",
          boxShadow: "0 1px 0 rgba(255,60,60,0.06) inset",
        }}
      >
        <div className="flex items-start justify-between gap-3">
          <h3
            className="line-clamp-2 font-semibold leading-tight transition group-hover:text-white"
            style={{ color: "var(--fg)" }}
          >
            {job.title}
          </h3>
          <span
            className="shrink-0 rounded-full border px-2.5 py-1 text-xs font-semibold"
            style={{ background: s.bg, color: s.color, borderColor: s.border }}
          >
            {job.status}
          </span>
        </div>
        <p className="mt-2 line-clamp-2 min-h-10 text-sm" style={{ color: "var(--muted)" }}>
          {job.description || "Sin descripción"}
        </p>
        <div className="mt-4 flex flex-wrap items-center gap-2 text-xs" style={{ color: "var(--muted)" }}>
          <span
            className="rounded-full px-2.5 py-1 font-mono font-semibold text-white"
            style={{ background: "var(--gradient)" }}
          >
            {Number(job.amount) / 1e9} SOL
          </span>
          <span className="font-mono" style={{ color: "var(--fg)" }}>
            #{job.jobId}
          </span>
          <span>·</span>
          <span className={overdue ? "font-medium" : ""} style={overdue ? { color: "var(--primary)" } : {}}>
            {new Date(job.deadline * 1000).toLocaleDateString()} {overdue ? "· vencido" : ""}
          </span>
        </div>
        <div className="mt-2 truncate font-mono text-[10px]" style={{ color: "var(--muted)" }}>
          {job.client.slice(0, 16)}…
        </div>
        <div
          className="mt-4 h-px w-full opacity-60 transition group-hover:opacity-100"
          style={{ background: "linear-gradient(90deg, transparent, var(--border), transparent)" }}
        />
      </Link>
    </motion.div>
  );
}
