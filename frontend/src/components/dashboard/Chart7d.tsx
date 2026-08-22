"use client";
import { motion } from "framer-motion";
import { ResponsiveContainer, AreaChart, Area, XAxis, YAxis, Tooltip, CartesianGrid } from "recharts";
import type { Job } from "@/api/types";

export function Chart7d({ jobs }: { jobs: Job[] }) {
  const days = 7;
  const data = Array.from({length:days},(_,i)=>{
    const d = new Date(); d.setDate(d.getDate() - (days-1-i));
    const key = d.toLocaleDateString("es",{weekday:"short"});
    const count = jobs.filter(j=> {
      const ts = j.createdAt ? j.createdAt*1000 : j.deadline*1000;
      const jd = new Date(ts);
      return jd.toDateString()===d.toDateString();
    }).length;
    // fallback mock when jobs empty: sine
    const val = jobs.length===0 ? Math.round(2+ Math.sin(i)*1.5 + i%2) : count;
    return { name:key, value: Math.max(0,val) };
  });
  return (
    <motion.div initial={{opacity:0,y:10}} animate={{opacity:1,y:0}} transition={{duration:0.5}} className="card">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-semibold" style={{color:"var(--fg)"}}>Actividad 7d</h3>
        <span className="text-xs" style={{color:"var(--muted)"}}>jobs/día · recharts</span>
      </div>
      <div className="mt-3 h-[180px]">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={data} margin={{top:5,right:10,left:0,bottom:0}}>
            <defs>
              <linearGradient id="dcgrad" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stopColor="#FF3C3C" stopOpacity={0.45}/>
                <stop offset="100%" stopColor="#FF3C3C" stopOpacity={0}/>
              </linearGradient>
            </defs>
            <CartesianGrid strokeDasharray="3 3" stroke="rgba(160,30,30,0.18)" />
            <XAxis dataKey="name" tick={{fill:"#8C4646", fontSize:11}} axisLine={false} tickLine={false}/>
            <YAxis tick={{fill:"#8C4646", fontSize:11}} axisLine={false} tickLine={false} allowDecimals={false}/>
            <Tooltip contentStyle={{background:"#1E0E0E", border:"1px solid #A01E1E", borderRadius:12, color:"#F0D2D2"}}/>
            <Area type="monotone" dataKey="value" stroke="#FF3C3C" strokeWidth={2} fill="url(#dcgrad)" dot={{r:3, stroke:"#FF3C3C", fill:"#1E0E0E"}} activeDot={{r:5}}/>
          </AreaChart>
        </ResponsiveContainer>
      </div>
    </motion.div>
  );
}
