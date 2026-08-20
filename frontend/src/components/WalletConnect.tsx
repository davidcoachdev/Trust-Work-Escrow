"use client";

import dynamic from "next/dynamic";
import { useWallet } from "@solana/wallet-adapter-react";
import { motion } from "framer-motion";

const WalletMultiButtonDynamic = dynamic(
  async () => (await import("@solana/wallet-adapter-react-ui")).WalletMultiButton,
  {
    ssr: false,
    loading: () => (
      <button className="btn" style={{ background: "var(--primary)", color: "white", borderRadius: 12 }}>
        Conectar wallet
      </button>
    ),
  }
);

export function WalletConnect() {
  const { publicKey } = useWallet();
  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.96 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ duration: 0.4, ease: [0.25, 0.1, 0.25, 1] }}
      className="flex items-center gap-3"
    >
      <WalletMultiButtonDynamic />
      {publicKey && (
        <motion.span
          initial={{ opacity: 0, x: 8 }}
          animate={{ opacity: 1, x: 0 }}
          className="hidden rounded-full border px-2.5 py-1 font-mono text-xs sm:inline-flex"
          style={{ background: "var(--surface)", borderColor: "var(--border)", color: "var(--muted)" }}
        >
          {publicKey.toBase58().slice(0, 4)}…{publicKey.toBase58().slice(-4)}
        </motion.span>
      )}
    </motion.div>
  );
}
