"use client";
import { useEffect } from "react";
import { useDashboardStore } from "@/stores/useDashboardStore";
import { ClientOnly } from "@/components/dashboard/RoleGuard";
import { HistoryTable } from "@/components/dashboard/HistoryTable";

export default function ClientHistoryPage(){
  const { jobs, fetchJobs } = useDashboardStore();
  useEffect(()=>{ fetchJobs({cursor:null, limit:50}).catch(()=>{}); },[fetchJobs]);
  return (
    <ClientOnly>
      <div className="space-y-4">
        <h1 className="text-2xl font-bold" style={{color:"var(--fg)"}}>Cerrados / Historial — Publisher</h1>
        <p className="text-sm" style={{color:"var(--muted)"}}>/dashboard/client/history · tabla Released/Resolved/Cancelled + métricas total gastado/fee/disputas %/tiempo + export CSV · filtros 30d/90d/todo · recharts</p>
        <HistoryTable jobs={jobs} />
      </div>
    </ClientOnly>
  );
}
