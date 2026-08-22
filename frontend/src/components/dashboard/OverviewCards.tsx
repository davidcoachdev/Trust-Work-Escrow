"use client";
import { motion } from "framer-motion";
import type { Job } from "@/api/types";

export function OverviewCards({ jobs }: { jobs: Job[] }) {
  const activos = jobs.filter(j=> j.status==="Open" || j.status==="InProgress").length;
  const disputa = jobs.filter(j=> j.status==="Disputed").length;
  const cerrados = jobs.filter(j=> j.status==="Completed" || j.status==="Cancelled").length;
  const gananciaLamports = jobs.filter(j=> j.status==="Completed").reduce((a,j)=> a + Number(j.amount), 0);
  const ganancia = (gananciaLamports/1e9).toFixed(2);
  const rating = (4.2 + (gananciaLamports % 7)/10).toFixed(1);
  const cards = [
    { label:"Activos", value:String(activos), sub:"Open + InProgress", color:"var(--primary)" },
    { label:"En disputa", value:String(disputa), sub:"Disputed", color:"#FFC857" },
    { label:"Cerrados", value:String(cerrados), sub:"Completed/Cancelled", color:"var(--fg)" },
    { label:"Ganancia", value:`${ganancia} SOL`, sub:"Σ completed", color:"#B4FF64" },
    { label:"Rating", value:`★ ${rating}`, sub:"mock freelancer", color:"#FF5050" },
  ];
  return (
    <div className="grid gap-3 md:grid-cols-5">
      {cards.map((c,i)=>(
        <motion.div key={c.label} initial={{opacity:0,y:12}} animate={{opacity:1,y:0}} transition={{delay:i*0.06, duration:0.45}} whileHover={{y:-2}} className="card card-hover">
          <div className="text-xs font-semibold tracking-widest" style={{color:"var(--muted)"}}>{c.label.toUpperCase()}</div>
          <div className="mt-1 text-2xl font-bold" style={{color:c.color}}>{c.value}</div>
          <div className="text-xs" style={{color:"var(--muted)"}}>{c.sub}</div>
        </motion.div>
      ))}
    </div>
  );
}
