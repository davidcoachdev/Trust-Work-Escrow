# Trust Work Escrow — Landing

Landing page oficial de **Trust Work Escrow**, un escrow descentralizado para
freelancers y empleadores, liquidado on-chain en **Solana**.

Construida con **Dioxus 0.7** (Rust → WebAssembly) y **Tailwind CSS v4**, con
theming multi-skin en runtime y i18n first-class (es/en).

---

## Stack

| Capa            | Tecnología                                            |
| --------------- | ---------------------------------------------------- |
| Framework       | [Dioxus](https://dioxuslabs.com) 0.7.9 (web/WASM)   |
| Lenguaje        | Rust (edición 2021)                                  |
| UI              | RSX vía macro `rsx!` en archivos `.rs`               |
| Estilos         | Tailwind CSS v4 (integrado por `dx`)                 |
| Theming         | CSS variables + utilidades Tailwind mapeadas        |
| i18n            | es / en, persistido en `localStorage`                |
| Routing         | Router propio basado en señales (`Route` enum)      |

> **Nota de versiones:** `dx` debe ser `0.7.9` y `dioxus` está fijado a
> `=0.7.9` en `Cargo.toml`. Si no coinciden, `dx` falla con
> *"dx and dioxus versions are incompatible!"*.

---

## Características

- **Multi-tema en runtime**: 3 skins seleccionables por el usuario y
  persistidas — `dcdev` (rojo, marca por defecto), `cyan` y `solana`
  (degradado morado→verde) — combinables con modo **dark/light**.
  Se aplican vía atributos en `<html data-theme>` / `<html data-mode>`.
- **i18n first-class**: español e inglés desde el arranque, con diccionario
  en `src/i18n/mod.rs` (`tr(lang, "key")`). El idioma se guarda en
  `localStorage` y se respeta el `lang` del navegador al primer carga.
- **Componentes reutilizables**: estructura tipo React/Next con componentes
  desacoplados bajo `src/features/*`, escritos con la macro `rsx!` (sintaxis
  RSX/JSX) en archivos `.rs`.
- **Tailwind utilities**: el diseño usa utilidades puras (`bg-bg`,
  `text-fg`, `bg-primary`, `bg-surface`, `border-border`, `gradient`,
  `wrap`…) generadas a partir de los tokens de tema.

---

## Estructura

```text
landing/
├── Cargo.toml              # dioxus = "=0.7.9", feature "web"
├── Dioxus.toml             # [tailwind] input/output, title
├── tailwind.css            # Fuente Tailwind: @import, @source, @theme, base
├── index.html              # Shell HTML (Dioxus monta aquí)
├── assets/
│   ├── tailwind.css        # CSS generado por dx / Tailwind CLI
│   ├── favicon.svg
│   └── og-image.svg
└── src/
    ├── main.rs             # lanza dioxus::launch(app::App)
    ├── app.rs              # providers de contexto (tema, idioma, modo, ruta)
    ├── route.rs            # Route enum + RouteContext
    ├── theme/              # Theme (skins) + Mode (dark/light) + persistencia
    ├── i18n/               # Lang + tr() + persistencia
    ├── ui/                 # componentes compartidos (navbar.rs)
    └── features/
        ├── landing/        # hero, features, how, stats, who, cta, footer
        ├── auth/           # login, signup
        └── contact/        # contact
```

### Secciones de la landing (`src/features/landing/`)

`Hero` → `HowItWorks` (pasos) → `Features` (6 tarjetas) → `Stats`
(métricas) → `ForWhom` (freelancers / empleadores) → `Cta` → `Footer`.

Páginas adicionales: `Login`, `Signup` y `Contact` (router propio).

---

## Theming y Tailwind

`tailwind.css` (raíz) define:

```css
@import "tailwindcss";
@source "./src/**/*.{rs,rsx,html,css}";   /* escanea los .rsx para generar utilidades */

:root, [data-theme="dcdev"] { --bg: #120808; --primary: #ff3c3c; /* ... */ }
[data-theme="cyan"]   { /* ... */ }
[data-theme="solana"] { /* ... */ }
/* variantes [data-mode="light"] por skin */

@theme {
  --color-bg: var(--bg);
  --color-surface: var(--bg-2);
  --color-fg: var(--fg);
  --color-muted: var(--fg-2);
  --color-primary: var(--primary);
  --color-primary-2: var(--primary-2);
  --color-on-primary: var(--on-primary);
  --color-border: var(--border);
}

@utility gradient { background-image: var(--gradient); }
@layer components { .wrap { width: min(1120px, calc(100% - 64px)); margin-inline: auto; } }
```

`dx` detecta `tailwind.css`, corre el watcher de Tailwind y genera
`assets/tailwind.css`, que se linkea en `app.rs` con
`asset!("/assets/tailwind.css")`.

---

## Desarrollo

Requisitos: `rustup` (con target `wasm32-unknown-unknown`), `cargo`,
`dx` 0.7.9 y Node (para Tailwind, opcional — `dx` lo descarga solo).

```bash
# Servir en modo dev (hot-reload)
dx serve

# Build de producción (web)
dx build --release

# Build para web explícito
dx build --release --platform web
```

La app queda en `http://localhost:8080`.

### Generar Tailwind manualmente (opcional)

```bash
npm install -D tailwindcss@4
npx @tailwindcss/cli -i ./tailwind.css -o ./assets/tailwind.css
```

> En este sandbox el `npx @tailwindcss/cli` necesita `tailwindcss` instalado
> localmente para resolver el `@import "tailwindcss"`. `dx serve` lo maneja
> automáticamente.

---

## Nota sobre la extensión de los componentes

Los componentes se escriben en archivos **`.rs`** con la macro `rsx!`. A
diferencia de React (`.jsx`/`.tsx`), **rustc no reconoce `.rsx` como archivo
de módulo**: `dx serve` / `cargo` buscan `navbar.rs` (o `navbar/mod.rs`) y
fallan con `file not found for module` si solo existe `navbar.rsx`. El
resaltado y formateo RSX lo provee el language server de Dioxus sobre los
`.rs` que usan `rsx!`, así que no se pierde la experiencia de edición.

> `dx check` no hace un build real de cargo y puede dar falsos positivos con
> módulos `.rsx`; usá `dx build` o `dx serve` para validar de verdad.

---

## Build / check

```bash
dx check      # valida tipos y RSX sin abrir el navegador
```

---

## Licencia

MIT — ver [`LICENSE`](./LICENSE).
