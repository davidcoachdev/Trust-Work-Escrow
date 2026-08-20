import type { Metadata } from "next";
import Link from "next/link";
import { AppWalletProvider } from "@/components/WalletProvider";
import { WalletConnect } from "@/components/WalletConnect";
import "./globals.css";

export const metadata: Metadata = {
  title: "Trust Work Escrow v3",
  description: "dApp frontend — Trust Work Escrow v3 (Next.js 16 + Solana)",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="es">
      <body className="min-h-screen bg-white text-zinc-900 antialiased">
        <AppWalletProvider>
          <header className="sticky top-0 z-10 border-b border-zinc-200 bg-white/80 backdrop-blur">
            <div className="mx-auto flex max-w-6xl items-center justify-between gap-4 px-6 py-3">
              <div className="flex items-center gap-6">
                <Link href="/" className="font-bold tracking-tight">
                  Trust Work Escrow <span className="text-emerald-600">v3</span>
                </Link>
                <nav className="hidden md:flex items-center gap-1 text-sm">
                  <Link href="/jobs" className="rounded-full px-3 py-1.5 hover:bg-zinc-100">
                    Jobs
                  </Link>
                  <Link href="/create" className="rounded-full bg-zinc-900 px-3 py-1.5 text-white hover:bg-zinc-800">
                    Crear job
                  </Link>
                </nav>
              </div>
              <WalletConnect />
            </div>
            <div className="mx-auto flex max-w-6xl gap-1 px-6 pb-3 md:hidden text-sm">
              <Link href="/jobs" className="rounded-full border px-3 py-1">
                Jobs
              </Link>
              <Link href="/create" className="rounded-full bg-zinc-900 px-3 py-1 text-white">
                Crear
              </Link>
            </div>
          </header>
          <main className="mx-auto max-w-6xl px-6 py-8">{children}</main>
          <footer className="mx-auto max-w-6xl px-6 py-8 text-xs text-zinc-400">
            Landing es Dioxus · Este frontend es Next.js 16 dApp · Programa{" "}
            <span className="font-mono">7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh</span> · SDK: trust-escrow-sdk v3
          </footer>
        </AppWalletProvider>
      </body>
    </html>
  );
}
