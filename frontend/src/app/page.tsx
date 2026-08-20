"use client";
import Link from "next/link";
import { motion } from "framer-motion";
import { useEffect } from "react";
import gsap from "gsap";

export default function HomePage() {
  useEffect(() => {
    const ctx = gsap.context(() => {
      gsap.from("[data-hero]", { y: 22, opacity: 0, duration: 0.8, stagger: 0.08, ease: "power3.out" });
      gsap.from("[data-card]", { y: 18, opacity: 0, duration: 0.6, stagger: 0.07, ease: "power3.out", delay: 0.35 });
    });
    return () => ctx.revert();
  }, []);

  return (
    <div className="space-y-8 md:space-y-10">
      {/* Hero */}
      <div
        data-hero
        className="relative overflow-hidden rounded-[16px] p-8 md:p-10"
        style={{
          background: "var(--surface)",
          border: "1px solid var(--border)",
        }}
      >
        {/* gradient glow parallax */}
        <div
          data-parallax="0.12"
          className="pointer-events-none absolute -right-20 -top-20 h-72 w-72 rounded-full opacity-25 blur-3xl"
          style={{ background: "var(--gradient)" }}
          aria-hidden
        />
        <div
          data-parallax="0.08"
          className="pointer-events-none absolute -left-10 -bottom-10 h-52 w-52 rounded-full opacity-15 blur-3xl"
          style={{ background: "var(--primary)" }}
          aria-hidden
        />

        <p
          data-hero
          className="text-xs font-bold tracking-[0.18em]"
          style={{ color: "var(--primary)" }}
        >
          TRUST WORK ESCROW v3 — DCDEV
        </p>
        <h1
          data-hero
          className="mt-3 text-4xl font-bold tracking-tight md:text-5xl"
          style={{ color: "var(--fg)", lineHeight: 1.05 }}
        >
          Trabajo sin confianza,
          <br />
          <span
            style={{
              background: "var(--gradient)",
              WebkitBackgroundClip: "text",
              WebkitTextFillColor: "transparent",
              backgroundClip: "text",
            }}
          >
            con escrow.
          </span>
        </h1>
        <p data-hero className="mt-4 max-w-2xl text-sm md:text-base" style={{ color: "var(--muted)" }}>
          Frontend dApp en Next.js 16 + Wallet Adapter + trust-escrow-sdk (programa{" "}
          <span className="font-mono text-xs" style={{ color: "var(--fg)" }}>
            7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh
          </span>
          ). Tema dcdev — crimson oscuro, Inter, 8pt grid, animaciones GSAP + Framer Motion.
        </p>
        <div data-hero className="mt-7 flex flex-wrap gap-3">
          <motion.div whileHover={{ scale: 1.02 }} whileTap={{ scale: 0.98 }}>
            <Link href="/jobs" className="btn" style={{ borderRadius: 12 }}>
              Ver jobs
            </Link>
          </motion.div>
          <motion.div whileHover={{ scale: 1.02 }} whileTap={{ scale: 0.98 }}>
            <Link href="/create" className="btn btn-secondary">
              Crear job
            </Link>
          </motion.div>
          <motion.div whileHover={{ scale: 1.02 }} whileTap={{ scale: 0.98 }}>
            <a
              href="http://127.0.0.1:8899"
              target="_blank"
              className="btn btn-ghost"
              rel="noreferrer"
            >
              Explorer localnet
            </a>
          </motion.div>
        </div>

        {/* bottom gradient divider */}
        <div className="absolute inset-x-0 bottom-0 h-px" style={{ background: "var(--gradient)", opacity: 0.5 }} />
      </div>

      {/* Feature cards */}
      <div className="grid gap-4 md:grid-cols-3">
        {[
          { title: "/jobs", desc: "Lista paginada via sdk.list_jobs (mock + RPC ready). Stagger + hover lift.", href: "/jobs" },
          { title: "/jobs/:id", desc: "Detalle + aplicar via sdk.apply (apply_to_job). Framer Motion entrance.", href: "/jobs" },
          { title: "/create", desc: "Crear job via sdk.create_job (Anchor instruction). GSAP focus states.", href: "/create" },
        ].map((f) => (
          <motion.div
            key={f.title}
            data-card
            whileHover={{ y: -4, transition: { type: "spring", stiffness: 400, damping: 18 } }}
            className="card card-hover"
          >
            <h3 className="font-semibold" style={{ color: "var(--fg)" }}>
              {f.title}
            </h3>
            <p className="mt-1 text-sm" style={{ color: "var(--muted)" }}>
              {f.desc}
            </p>
            <Link
              href={f.href}
              className="mt-4 inline-flex text-xs font-semibold hover:underline"
              style={{ color: "var(--primary)" }}
            >
              Abrir →
            </Link>
          </motion.div>
        ))}
      </div>

      {/* Paleta preview */}
      <motion.div
        initial={{ opacity: 0, y: 16 }}
        whileInView={{ opacity: 1, y: 0 }}
        viewport={{ once: true }}
        transition={{ duration: 0.6 }}
        className="card flex flex-wrap items-center gap-3"
      >
        <span className="text-xs font-semibold" style={{ color: "var(--muted)" }}>
          Paleta dcdev:
        </span>
        {[
          ["bg", "#120808"],
          ["surface", "#1E0E0E"],
          ["primary", "#FF3C3C"],
          ["secondary", "#781414"],
          ["fg", "#F0D2D2"],
          ["muted", "#8C4646"],
          ["border", "#A01E1E"],
        ].map(([name, hex]) => (
          <span key={name} className="inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs font-mono" style={{ borderColor: "var(--border)", background: "var(--surface-2)", color: "var(--fg)" }}>
            <span className="h-3 w-3 rounded-full border" style={{ background: hex, borderColor: "var(--border)" }} />
            {name} {hex}
          </span>
        ))}
        <span className="rounded-full px-3 py-1 text-xs font-semibold text-white" style={{ background: "var(--gradient)" }}>
          gradient
        </span>
      </motion.div>
    </div>
  );
}
