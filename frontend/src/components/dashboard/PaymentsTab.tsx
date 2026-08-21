"use client";
import { motion } from "framer-motion";
import { useJobStore } from "@/stores/useJobStore";

export function PaymentsTab({ jobId, amount }: { jobId: string; amount: string }) {
  const { deposit, approveWork, submitWork, rejectWork, loading } = useJobStore();
  const lamports = Number(amount);
  const fee = Math.floor(lamports*0.025);
  const net = lamports - fee;
  return (
    <div className="space-y-3">
      <div className="grid gap-3 md:grid-cols-3">
        <div className="card" style={{background:"var(--surface-2)"}}><div className="text-xs" style={{color:"var(--muted)"}}>Monto total</div><div className="font-mono font-bold" style={{color:"var(--fg)"}}>{(lamports/1e9).toFixed(4)} SOL</div></div>
        <div className="card" style={{background:"var(--surface-2)"}}><div className="text-xs" style={{color:"var(--muted)"}}>Fee 2.5%</div><div className="font-mono font-bold" style={{color:"var(--primary)"}}>{(fee/1e9).toFixed(4)} SOL</div></div>
        <div className="card" style={{background:"var(--surface-2)"}}><div className="text-xs" style={{color:"var(--muted)"}}>Neto freelancer</div><div className="font-mono font-bold" style={{color:"#B4FF64"}}>{(net/1e9).toFixed(4)} SOL</div></div>
      </div>
      <div className="flex flex-wrap gap-2">
        <motion.button whileTap={{scale:0.98}} onClick={()=>deposit(jobId)} disabled={loading} className="btn">Deposit</motion.button>
        <motion.button whileTap={{scale:0.98}} onClick={()=>submitWork(jobId)} disabled={loading} className="btn btn-secondary">Submit Work</motion.button>
        <motion.button whileTap={{scale:0.98}} onClick={()=>approveWork(jobId)} disabled={loading} className="btn" style={{background:"#B4FF64", color:"#0A0404"}}>Approve & Release</motion.button>
        <motion.button whileTap={{scale:0.98}} onClick={()=>rejectWork(jobId)} disabled={loading} className="btn btn-ghost">Reject</motion.button>
      </div>
      <p className="text-xs" style={{color:"var(--muted)"}}>Pagos escrow on-chain · useJobStore → api/jobs/deposit/approve etc. · auto-approve 7d tras submit si cliente no responde (mostrado en badge)</p>
    </div>
  );
}
