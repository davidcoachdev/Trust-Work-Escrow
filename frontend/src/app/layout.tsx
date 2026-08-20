import type { Metadata } from "next";
import Link from "next/link";
import { Inter } from "next/font/google";
import { AppWalletProvider } from "@/components/WalletProvider";
import { WalletConnect } from "@/components/WalletConnect";
import { HeaderNav } from "@/components/HeaderNav";
import "./globals.css";

const inter = Inter({
  subsets: ["latin"],
  variable: "--font-inter",
  display: "swap",
});

export const metadata: Metadata = {
  title: "Trust Work Escrow v3 — dcdev",
  description: "dApp Trust Work Escrow v3 — Next.js 16 + Solana · tema dcdev (crimson dark)",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="es" className={inter.variable}>
      <body className="min-h-screen antialiased" style={{ background: "var(--bg)", color: "var(--fg)" }}>
        <AppWalletProvider>
          {/* Top gradient bar */}
          <div className="h-[2px] w-full" style={{ background: "var(--gradient)" }} aria-hidden />
          <header
            className="sticky top-0 z-20 backdrop-blur-xl"
            style={{
              background: "rgba(18,8,8,0.82)",
              borderBottom: "1px solid var(--border)",
            }}
          >
            <div className="mx-auto flex max-w-6xl items-center justify-between gap-4 px-6 py-3">
              <div className="flex items-center gap-6">
                <Link
                  href="/"
                  className="font-bold tracking-tight transition hover:opacity-90"
                  style={{ color: "var(--fg)" }}
                >
                  Trust Work Escrow <span style={{ color: "var(--primary)" }}>v3</span>
                  <span
                    className="ml-2 hidden rounded-full px-2 py-0.5 text-[10px] font-bold tracking-widest sm:inline-flex"
                    style={{ background: "var(--gradient)", color: "white" }}
                  >
                    DCDEV
                  </span>
                </Link>
                <HeaderNav />
              </div>
              <WalletConnect />
            </div>
            {/* Mobile nav */}
            <div className="mx-auto flex max-w-6xl gap-2 px-6 pb-3 md:hidden text-sm">
              <Link
                href="/jobs"
                className="rounded-full px-3 py-1.5 text-xs font-medium transition"
                style={{ border: "1px solid var(--border)", color: "var(--muted)" }}
              >
                Jobs
              </Link>
              <Link
                href="/create"
                className="rounded-full px-3 py-1.5 text-xs font-semibold text-white"
                style={{ background: "var(--primary)" }}
              >
                Crear
              </Link>
            </div>
          </header>

          <main className="mx-auto max-w-6xl px-6 py-8 md:py-10">{children}</main>

          <footer
            className="mx-auto max-w-6xl px-6 py-8 text-xs"
            style={{ color: "var(--muted)", borderTop: "1px solid rgba(160,30,30,0.25)" }}
          >
            <div className="flex flex-wrap items-center justify-between gap-3">
              <span>
                Landing es Dioxus · Este frontend es Next.js 16 dApp · Programa{" "}
                <span className="font-mono" style={{ color: "var(--fg)" }}>
                  7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh
                </span>{" "}
                · SDK: trust-escrow-sdk v3
              </span>
              <span className="flex items-center gap-2">
                <span className="h-2 w-2 animate-pulse rounded-full" style={{ background: "var(--primary)" }} />
                tema dcdev · Inter · 8pt grid
              </span>
            </div>
          </footer>
        </AppWalletProvider>
      </body>
    </html>
  );
}
