"use client";
import { useEffect, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useDashboardStore } from "@/stores/useDashboardStore";

export function NotificationBell({ pollingMs = 15000 }: { pollingMs?: number }) {
  const { notifications, markAllRead, refresh, startPolling } = useDashboardStore();
  const [open, setOpen] = useState(false);
  const unread = notifications.filter(n=>!n.read).length;
  useEffect(() => {
    const stop = startPolling(pollingMs);
    return stop;
  }, [pollingMs, startPolling]);
  useEffect(()=>{ refresh().catch(()=>{}); }, [refresh]);

  return (
    <div className="relative">
      <motion.button whileTap={{scale:0.96}} onClick={()=>setOpen(v=>!v)} className="relative btn btn-ghost px-3 py-2" aria-label="Notificaciones">
        <span>🔔</span>
        {unread>0 && <span className="absolute -right-1 -top-1 grid h-5 min-w-5 place-items-center rounded-full px-1 text-[10px] font-bold text-white" style={{background:"var(--primary)"}}>{unread}</span>}
      </motion.button>
      <AnimatePresence>
        {open && (
          <motion.div initial={{opacity:0,y:6,scale:0.98}} animate={{opacity:1,y:0,scale:1}} exit={{opacity:0,y:6}} className="absolute right-0 z-30 mt-2 w-80 rounded-[16px] p-3 shadow-xl" style={{background:"var(--surface)", border:"1px solid var(--border)"}}>
            <div className="flex items-center justify-between">
              <span className="text-sm font-semibold" style={{color:"var(--fg)"}}>Notificaciones</span>
              <button onClick={markAllRead} className="text-xs underline" style={{color:"var(--primary)"}}>Marcar leídas</button>
            </div>
            <div className="mt-2 max-h-72 space-y-2 overflow-auto">
              {notifications.length===0 ? <p className="text-xs" style={{color:"var(--muted)"}}>Sin notificaciones · polling {pollingMs/1000}s</p> : notifications.slice(0,10).map(n=>(
                <div key={n.id} className="rounded-xl p-2" style={{background:"var(--surface-2)", border:"1px solid var(--border)"}}>
                  <div className="text-xs font-semibold" style={{color:"var(--fg)"}}>{n.title}</div>
                  <div className="text-xs" style={{color:"var(--muted)"}}>{n.body}</div>
                  <div className="text-[10px]" style={{color:"var(--muted)"}}>{new Date(n.at).toLocaleTimeString()}</div>
                </div>
              ))}
            </div>
            <button onClick={()=>{ refresh().catch(()=>{}); setOpen(false); }} className="btn btn-secondary mt-3 w-full">Sync ahora</button>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
