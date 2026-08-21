"use client";
import { useState } from "react";
import { motion } from "framer-motion";

export interface ChatMsg { id:string; author:string; text:string; at:number; me?: boolean }

export function ChatTab({ jobId, initial }: { jobId: string; initial?: ChatMsg[] }) {
  const [msgs,setMsgs]=useState<ChatMsg[]>(initial ?? [
    {id:"1", author:"Cliente", text:"Hola, ¿cuándo entregas el milestone 1?", at:Date.now()-3600000},
    {id:"2", author:"Tú", text:"Hoy a las 18h envío revisión", at:Date.now()-1800000, me:true},
  ]);
  const [text,setText]=useState("");
  function send(){
    if(!text.trim()) return;
    setMsgs(m=>[...m,{id:Math.random().toString(36).slice(2), author:"Tú", text, at:Date.now(), me:true}]);
    setText("");
  }
  return (
    <div className="space-y-3">
      <div className="max-h-[320px] space-y-2 overflow-auto p-1">
        {msgs.map(m=>(
          <motion.div key={m.id} initial={{opacity:0,y:6}} animate={{opacity:1,y:0}} className={`rounded-[12px] p-3 text-sm ${m.me?'ml-8':''}`} style={m.me?{background:"var(--primary)", color:"white"}:{background:"var(--surface-2)", color:"var(--fg)", border:"1px solid var(--border)"}}>
            <div className="text-xs font-semibold" style={m.me?{color:"white"}:{color:"var(--muted)"}}>{m.author} · {new Date(m.at).toLocaleTimeString()}</div>
            <div className="mt-1">{m.text}</div>
          </motion.div>
        ))}
      </div>
      <div className="flex gap-2">
        <input value={text} onChange={e=>setText(e.target.value)} onKeyDown={e=> e.key==='Enter' && send()} placeholder="Escribe mensaje… (chat por job)" className="input flex-1" />
        <button onClick={send} className="btn">Enviar</button>
      </div>
      <p className="text-xs" style={{color:"var(--muted)"}}>Chat por job #{jobId} · polling sync + local state · backend support/open ready</p>
    </div>
  );
}
