"use client";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { motion } from "framer-motion";

const links = [
  { href: "/jobs", label: "Jobs" },
  { href: "/create", label: "Crear job" },
];

export function HeaderNav() {
  const pathname = usePathname();
  return (
    <nav className="hidden md:flex items-center gap-1 text-sm">
      {links.map((l) => {
        const active = pathname === l.href || (l.href !== "/" && pathname.startsWith(l.href));
        return (
          <Link
            key={l.href}
            href={l.href}
            className="relative rounded-full px-3.5 py-1.5 font-medium transition"
            style={
              active
                ? { background: "var(--surface)", color: "var(--fg)", border: "1px solid var(--border)" }
                : { color: "var(--muted)" }
            }
          >
            {active && (
              <motion.span
                layoutId="nav-active"
                className="absolute inset-0 rounded-full"
                style={{ background: "var(--surface)", border: "1px solid var(--border)" , zIndex: -1 }}
                transition={{ type: "spring", stiffness: 400, damping: 30 }}
              />
            )}
            <span className="relative">{l.label}</span>
          </Link>
        );
      })}
    </nav>
  );
}
