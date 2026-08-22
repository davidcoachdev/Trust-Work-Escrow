"use client";
import { useEffect, useState } from "react";
import { autoApproveCountdown } from "@/lib/dashboardUtils";

export function AutoApproveBadge({ submittedAt }: { submittedAt: number }) {
  const [tick,setTick]=useState(0);
  useEffect(()=>{const id=setInterval(()=>setTick(t=>t+1),1000); return ()=>clearInterval(id);},[]);
  const { text, pct } = autoApproveCountdown(submittedAt, 7);
  return (
    <div className="flex items-center gap-2">
      <span className="text-xs" style={{color:"var(--muted)"}}>{text}</span>
      <div className="h-1.5 w-24 overflow-hidden rounded-full" style={{background:"var(--surface-2)", border:"1px solid var(--border)"}}>
        <div className="h-full" style={{width:`${pct}%`, background:"var(--gradient)", transition:"width 0.5s"}}/>
      </div>
    </div>
  );
}
