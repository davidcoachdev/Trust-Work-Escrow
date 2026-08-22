"use client";
import { useEffect } from "react";
import { useDashboardStore } from "@/stores/useDashboardStore";
import { FreelancerOnly } from "@/components/dashboard/RoleGuard";
import { HistoryTable } from "@/components/dashboard/HistoryTable";

export default function FreelancerHistory(){
  const { jobs, fetchJobs } = useDashboardStore();
  useEffect(()=>{ fetchJobs({cursor:null, limit:50}).catch(()=>{}); },[fetchJobs]);
  return (
    <FreelancerOnly>
      <div className="space-y-4">
        <h1 className="text-xl font-bold" style={{color:"var(--fg)"}}>Historial & Métricas — Freelancer</h1>
        <p className="text-sm" style={{color:"var(--muted)"}}>Filtros 30d/90d/todo · recharts · tabla cerrados · metrics ganancia/rating</p>
        <HistoryTable jobs={jobs} />
      </div>
    </FreelancerOnly>
  );
}
