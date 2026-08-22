"use client";
import { useEffect, useState } from "react";
import { countdown } from "@/lib/dashboardUtils";

export function DeadlineCountdown({ deadline }: { deadline: number }) {
  const [tick, setTick] = useState(0);
  useEffect(()=>{ const id=setInterval(()=>setTick(t=>t+1),1000); return ()=>clearInterval(id); },[]);
  const { text, overdue } = countdown(deadline);
  return <span className="rounded-full border px-2.5 py-1 text-xs font-mono" style={overdue?{background:"rgba(255,60,60,0.12)",color:"var(--primary)",borderColor:"rgba(255,60,60,0.3)"}:{background:"var(--surface-2)",color:"var(--muted)",borderColor:"var(--border)"}}>{text}</span>;
}
