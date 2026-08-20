import Link from "next/link";

export default function HomePage() {
  return (
    <div className="space-y-8">
      <div className="rounded-3xl border border-zinc-200 bg-gradient-to-br from-zinc-50 to-white p-8">
        <p className="text-xs font-semibold tracking-widest text-emerald-600">TRUST WORK ESCROW v3</p>
        <h1 className="mt-2 text-4xl font-bold tracking-tight">Trabajo sin confianza, con escrow.</h1>
        <p className="mt-3 max-w-2xl text-zinc-600">
          Frontend dApp en Next.js 16 + Wallet Adapter + trust-escrow-sdk (programa{" "}
          <span className="font-mono text-xs">7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh</span>). Landing original en Dioxus →
          nueva dApp en Next.js.
        </p>
        <div className="mt-6 flex flex-wrap gap-3">
          <Link href="/jobs" className="btn">
            Ver jobs
          </Link>
          <Link href="/create" className="btn btn-secondary">
            Crear job
          </Link>
          <a href="http://127.0.0.1:8899" target="_blank" className="btn btn-secondary">
            Explorer localnet
          </a>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-3">
        <div className="card">
          <h3 className="font-semibold">/jobs</h3>
          <p className="mt-1 text-sm text-zinc-600">Lista paginada via sdk.list_jobs (mock + RPC ready).</p>
        </div>
        <div className="card">
          <h3 className="font-semibold">/jobs/:id</h3>
          <p className="mt-1 text-sm text-zinc-600">Detalle + aplicar via sdk.apply (apply_to_job).</p>
        </div>
        <div className="card">
          <h3 className="font-semibold">/create</h3>
          <p className="mt-1 text-sm text-zinc-600">Crear job via sdk.create_job (Anchor instruction).</p>
        </div>
      </div>
    </div>
  );
}
