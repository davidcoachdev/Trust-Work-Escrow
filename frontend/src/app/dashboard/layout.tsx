"use client";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { motion } from "framer-motion";
import { useEffect } from "react";
import gsap from "gsap";
import { useAuthStore } from "@/stores/useAuthStore";
import { NotificationBell } from "@/components/dashboard/NotificationBell";

const nav = [
  { href:"/dashboard", label:"Dashboard", match:"/dashboard" },
  { href:"/dashboard/freelancer", label:"Freelancer", match:"/dashboard/freelancer" },
  { href:"/dashboard/client", label:"Publisher", match:"/dashboard/client" },
];

export default function DashboardLayout({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const { role, setRole, pubkey, setPubkey } = useAuthStore();

  useEffect(()=>{
    const ctx = gsap.context(()=>{
      gsap.from("[data-dash-nav]", { y:-8, opacity:0, duration:0.45, stagger:0.06, ease:"power3.out"});
    });
    return ()=>ctx.revert();
  },[]);

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-center justify-between gap-3 rounded-[16px] border p-3" style={{background:"var(--surface)", borderColor:"var(--border)"}}>
        <div className="flex items-center gap-2">
          <span className="text-xs font-bold tracking-widest" style={{color:"var(--primary)"}}>DASHBOARD v3</span>
          <span className="hidden text-xs md:inline" style={{color:"var(--muted)"}}>· Roles Freelancer/Publisher · x-pubkey ed25519 · Zustand</span>
        </div>
        <div className="flex items-center gap-2">
          <NotificationBell />
          <select value={role ?? ""} onChange={e=>setRole(e.target.value as any || null)} className="input w-32 py-1.5 text-xs">
            <option value="">Sin rol</option>
            <option value="freelancer">Freelancer</option>
            <option value="client">Publisher/Cliente</option>
          </select>
          <input value={pubkey ?? ""} onChange={e=>setPubkey(e.target.value || null)} placeholder="x-pubkey (base58 32b)" className="input w-40 py-1.5 text-xs font-mono hidden md:block" />
        </div>
      </div>

      <nav data-dash-nav className="flex flex-wrap gap-2">
        {nav.map(n=>{
          const active = pathname===n.href || (n.href!=="/dashboard" && pathname.startsWith(n.match));
          return (
            <Link key={n.href} href={n.href} className="rounded-full px-3.5 py-1.5 text-sm font-medium transition" style={active?{background:"var(--primary)", color:"white"}:{border:"1px solid var(--border)", color:"var(--muted)", background:"var(--surface)"}}>
              {n.label}
            </Link>
          );
        })}
        <Link href="/dashboard/client/create" className="rounded-full px-3.5 py-1.5 text-sm font-semibold" style={{background:"var(--gradient)", color:"white"}}>+ Crear job</Link>
        <Link href="/dashboard/client/disputes" className="rounded-full px-3 py-1.5 text-sm" style={{border:"1px solid var(--border)", color:"var(--muted)"}}>Disputas</Link>
        <Link href="/dashboard/client/history" className="rounded-full px-3 py-1.5 text-sm" style={{border:"1px solid var(--border)", color:"var(--muted)"}}>Historial</Link>
        <Link href="/dashboard/freelancer/history" className="rounded-full px-3 py-1.5 text-sm" style={{border:"1px solid var(--border)", color:"var(--muted)"}}>History F</Link>
      </nav>

      <motion.div initial={{opacity:0,y:10}} animate={{opacity:1,y:0}} transition={{duration:0.4}}>{children}</motion.div>

      <div className="rounded-xl border p-3 text-xs" style={{borderColor:"var(--border)", background:"var(--surface-2)", color:"var(--muted)"}}>
        Auth: <span className="font-mono" style={{color:"var(--fg)"}}>{pubkey ? pubkey.slice(0,16)+"… · role "+role : "sin pubkey · Header x-pubkey ed25519 se inyecta en api/client"}</span> · Stores Zustand useDashboardStore consume api/* (jobs, applications, milestones, disputes, support, arbiterPool) · búsqueda list_jobs_by_status/by_client cursor opaco · tema dcdev #FF3C3C + GSAP stagger + Framer layout
      </div>
    </div>
  );
}
