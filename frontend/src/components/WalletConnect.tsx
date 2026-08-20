"use client";

import dynamic from "next/dynamic";
import { useWallet } from "@solana/wallet-adapter-react";

const WalletMultiButtonDynamic = dynamic(
  async () => (await import("@solana/wallet-adapter-react-ui")).WalletMultiButton,
  { ssr: false, loading: () => <button className="btn">Conectar wallet</button> }
);

export function WalletConnect() {
  const { publicKey } = useWallet();
  return (
    <div className="flex items-center gap-3">
      <WalletMultiButtonDynamic />
      {publicKey && (
        <span className="text-xs font-mono text-zinc-500">
          {publicKey.toBase58().slice(0, 4)}…{publicKey.toBase58().slice(-4)}
        </span>
      )}
    </div>
  );
}
