/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  transpilePackages: [
    "@solana/wallet-adapter-react-ui",
    "@solana/wallet-adapter-react",
    "@solana/wallet-adapter-base",
  ],
  turbopack: {},
  webpack: (config) => {
    config.externals.push("pino-pretty", "lokijs", "encoding");
    return config;
  },
};

export default nextConfig;
