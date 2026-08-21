"use client";
import { useMemo, useState } from "react";
import { motion } from "framer-motion";
import type { Job } from "@/api/types";
import { filterByRange, computeMetrics, toCsv, downloadCsv } from "@/lib/dashboardUtils";
import { Chart7d } from "./Chart7d";

export function HistoryTable({ jobs }: { jobs: Job[] }) {
  const [range,setRange]=useState<"30d"|"90d"|"all">("all");
  const filtered = useMemo(()=> filterByRange(jobs.map(j=> ({...j, createdAt:j.createdAt})), range) as Job[], [jobs, range]);
  const closed = useMemo(()=> filtered.filter(j=> j.status==="Completed" || String(j.status)==="Released" || String(j.status)==="Resolved" || j.status==="Cancelled"), [filtered]);
  const metrics = useMemo(()=> computeMetrics(filtered as any), [filtered]);
  const rows = closed.map(j=> ({ jobId:j.jobId, title:j.title, status:j.status, amount:(Number(j.amount)/1e9).toFixed(4), fee:(Number(j.amount)*0.025/1e9).toFixed(4), deadline:new Date(j.deadline*1000).toLocaleDateString() }));
  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          {(["30d","90d","all"] as const).map(r=>(
            <button key={r} onClick={()=>setRange(r)} className={`rounded-full border px-3 py-1 text-xs font-semibold ${range===r?'text-white':''}`} style={range===r?{background:"var(--primary)", borderColor:"var(--primary)"}:{borderColor:"var(--border)", color:"var(--muted)", background:"var(--surface)"}}>{r==="all"?"Todo":r}</button>
          ))}
        </div>
        <div className="flex gap-2">
          <button onClick={()=> downloadCsv(`historial-${range}.csv`, toCsv(rows as any))} className="btn btn-secondary px-3 py-1 text-xs">Export CSV</button>
        </div>
      </div>

      <div className="grid gap-3 md:grid-cols-4">
        <div className="card"><div className="text-xs" style={{color:"var(--muted)"}}>Total gastado</div><div className="text-lg font-bold" style={{color:"var(--fg)"}}>{(metrics.totalGastado/1e9).toFixed(4)} SOL</div></div>
        <div className="card"><div className="text-xs" style={{color:"var(--muted)"}}>Fee total 2.5%</div><div className="text-lg font-bold" style={{color:"var(--primary)"}}>{(metrics.totalFee/1e9).toFixed(4)} SOL</div></div>
        <div className="card"><div className="text-xs" style={{color:"var(--muted)"}}>Disputas %</div><div className="text-lg font-bold" style={{color:"#FFC857"}}>{metrics.disputasPct.toFixed(1)}%</div></div>
        <div className="card"><div className="text-xs" style={{color:"var(--muted)"}}>Duración media</div><div className="text-lg font-bold" style={{color:"#B4FF64"}}>{metrics.avgDays.toFixed(1)} d</div><div className="text-xs" style={{color:"var(--muted)"}}>{closed.length} cerrados / {filtered.length} total</div></div>
      </div>

      <Chart7d jobs={filtered} />

      <motion.div initial={{opacity:0,y:8}} animate={{opacity:1,y:0}} className="overflow-auto rounded-[16px] border" style={{borderColor:"var(--border)", background:"var(--surface)"}}>
        <table className="w-full text-sm">
          <thead style={{background:"var(--surface-2)", color:"var(--muted)"}}>
            <tr><th className="px-3 py-2 text-left">Job</th><th className="px-3 py-2 text-left">Estado</th><th className="px-3 py-2 text-right">Monto</th><th className="px-3 py-2 text-right">Fee</th><th className="px-3 py-2 text-left">Deadline</th></tr>
          </thead>
          <tbody>
            {closed.length===0 ? <tr><td colSpan={5} className="px-3 py-6 text-center" style={{color:"var(--muted)"}}>Sin cerrados en rango {range}</td></tr> : closed.map(j=>(
              <tr key={j.jobId} className="border-t" style={{borderColor:"rgba(160,30,30,0.15)"}}>
                <td className="px-3 py-2 font-medium" style={{color:"var(--fg)"}}>#{j.jobId} {j.title.slice(0,32)}</td>
                <td className="px-3 py-2"><span className="rounded-full border px-2 py-0.5 text-xs" style={{borderColor:"var(--border)", background:"var(--surface-2)", color:"var(--muted)"}}>{j.status}</span></td>
                <td className="px-3 py-2 text-right font-mono" style={{color:"var(--fg)"}}>{(Number(j.amount)/1e9).toFixed(4)} SOL</td>
                <td className="px-3 py-2 text-right font-mono" style={{color:"var(--primary)"}}>{(Number(j.amount)*0.025/1e9).toFixed(4)}</td>
                <td className="px-3 py-2" style={{color:"var(--muted)"}}>{new Date(j.deadline*1000).toLocaleDateString()}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </motion.div>
      <p className="text-xs" style={{color:"var(--muted)"}}>Historial & Métricas ambos roles: filtros 30d/90d/todo, recharts, tabla Released/Resolved/Cancelled + métricas + export CSV</p>
    </div>
  );
}
