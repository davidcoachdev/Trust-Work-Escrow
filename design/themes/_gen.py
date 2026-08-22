import json, random, os

random.seed(20260720)

# ---- Temas: paleta dark + sub-paleta light, y meta de design system ----
# on_surface = texto SOBRE superficies (claro si la surface es clara / oscuro si la surface es oscura)
# on_primary = texto SOBRE el boton primario (acento)
THEMES = {
    "dcdev": {
        "label": "dcdev - rojo oscuro / crimson (elegido, actual)",
        "atmos": "Dark crimson - calido, tecnico, enfocado (rojo oscuro sobre casi-negro).",
        "source": "trust-escrow/tui/src/config.rs - funcion dcdev()",
        "font": "Inter (web) - el TUI usa monoespaciada de terminal.",
        "scale": [("H1", 48, 700), ("H2", 32, 700), ("H3", 24, 500),
                  ("Body", 16, 400), ("Small", 14, 400), ("Caption", 12, 400)],
        "bg": "#120808", "bg_deep": "#0A0404", "surface": "#1E0E0E",
        "surface2": "#2A1414", "fg": "#F0D2D2", "muted": "#8C4646",
        "primary": "#FF3C3C", "secondary": "#781414", "border": "#A01E1E",
        "title": "#FF5050", "success": "#B4FF64", "warning": "#FFA028", "error": "#FF1E1E",
        "on_surface": "#F0D2D2", "on_primary": "#F0D2D2", "gradient": "linear-gradient(135deg,#FF3C3C,#781414)",
        "light": {
            "bg": "#FFF6F6", "bg_deep": "#FBE9E9", "surface": "#FCEAEA", "surface2": "#F6DADA",
            "fg": "#2A0E0E", "muted": "#8C5A5A", "primary": "#FF3C3C", "secondary": "#C0392B",
            "border": "#E8C4C4", "title": "#C02828", "success": "#2E7D32", "warning": "#B26A00",
            "error": "#C62828",             "on_surface": "#2A0E0E", "on_primary": "#FFFFFF", "gradient": "linear-gradient(135deg,#FF3C3C,#781414)",
        },
    },
    "cyan": {
        "label": "cyan - azul cian / indigo (default v2, anterior)",
        "atmos": "Dark tech - azul cian / indigo, moderno y limpio.",
        "source": "trust-escrow-v2/tui/src/app/state.rs (Theme)",
        "font": "Inter (web) - el TUI usa monoespaciada de terminal.",
        "scale": [("H1", 48, 700), ("H2", 32, 700), ("H3", 24, 500),
                  ("Body", 16, 400), ("Small", 14, 400), ("Caption", 12, 400)],
        "bg": "#1A1A2E", "bg_deep": "#0E0E1A", "surface": "#20203A",
        "surface2": "#232342", "fg": "#E0E0E0", "muted": "#646482",
        "primary": "#00D4FF", "secondary": "#6464C8", "border": "#505078",
        "title": "#00D4FF", "success": "#50FA7B", "warning": "#FFB74D", "error": "#FF5555",
        "on_surface": "#E0E0E0", "on_primary": "#E0E0E0", "gradient": "linear-gradient(135deg,#00D4FF,#6464C8)",
        "light": {
            "bg": "#F4FBFE", "bg_deep": "#E3F4FB", "surface": "#EAF7FC", "surface2": "#D6EEF6",
            "fg": "#0E2230", "muted": "#5A7E8C", "primary": "#00B8D4", "secondary": "#0096B0",
            "border": "#C2E4EE", "title": "#007C99", "success": "#1B8A4B", "warning": "#B26A00",
            "error": "#C62828",             "on_surface": "#0E2230", "on_primary": "#FFFFFF", "gradient": "linear-gradient(135deg,#00D4FF,#6464C8)",
        },
    },
    "solana": {
        "label": "solana - design system oficial de solana.com/es (morado->verde)",
        "atmos": "High-contrast dark mode con el degradado morado->verde de Solana - moderno, tecnico, enfocado. Tarjetas lilas sobre fondo oscuro.",
        "source": "solana.com/es (design system oficial) - gradiente #9945FF -> #14F195",
        "font": "Diatype (web font) - heading + body. Weight 500, letter-spacing -2.64px en H1.",
        "scale": [("H1", 88, 500), ("H2", 64, 500), ("H3", 24, 500),
                  ("Body L", 24, 500), ("Body", 21, 400), ("Small", 16, 400), ("Caption", 16, 400)],
        "bg": "#121212", "bg_deep": "#0D0C11", "surface": "#ECE4FD",
        "surface2": "#181818", "fg": "#FFFFFF", "muted": "#999999",
        "primary": "#14F195", "secondary": "#9945FF", "border": "#2A2333",
        "title": "#14F195", "success": "#14F195", "warning": "#FFC526", "error": "#F48252",
        "on_surface": "#0D0C11", "on_primary": "#0D0C11",
        "gradient": "linear-gradient(135deg,#9945FF,#14F195)",
        "light": {
            "bg": "#FFFFFF", "bg_deep": "#F3EEFB", "surface": "#ECE4FD", "surface2": "#E0D6F7",
            "fg": "#0D0C11", "muted": "#6B6B6B", "primary": "#0E9E5C", "secondary": "#7A3FF2",
            "border": "#D9D2EC", "title": "#0E9E5C", "success": "#0E9E5C", "warning": "#B8860B",
            "error": "#C0392B", "on_surface": "#0D0C11", "on_primary": "#FFFFFF",
            "gradient": "linear-gradient(135deg,#9945FF,#14F195)",
        },
    },
}


def gen(theme, P):
    base = f"/home/dcdebian/Proyects/Trust-Work-Escrow/design/themes/{theme}"
    os.makedirs(f"{base}/ui-kit", exist_ok=True)
    os.makedirs(f"{base}/wireframes", exist_ok=True)
    os.makedirs(f"{base}/high-fi", exist_ok=True)
    open(f"{base}/high-fi/.gitkeep", "w").close()

    def el(over):
        b = {"id": over["id"], "type": over["type"], "x": over["x"], "y": over["y"],
             "width": over["width"], "height": over["height"], "angle": 0,
             "strokeColor": over.get("strokeColor", "#1e1e1e"),
             "backgroundColor": over.get("backgroundColor", "transparent"),
             "fillStyle": "solid", "strokeWidth": over.get("strokeWidth", 2),
             "strokeStyle": "solid", "roughness": over.get("roughness", 1), "opacity": 100,
             "groupIds": [], "frameId": None, "index": over.get("index", "a0"),
             "roundness": over.get("roundness", None), "seed": random.randint(1, 999999999),
             "version": 1, "versionNonce": random.randint(1, 999999999),
             "isDeleted": False, "boundElements": None, "updated": 1700000000000,
             "link": None, "locked": False}
        if over["type"] == "text":
            b.update({"text": over.get("text", ""), "fontSize": over.get("fontSize", 16),
                      "fontFamily": over.get("fontFamily", 2), "textAlign": over.get("textAlign", "left"),
                      "verticalAlign": "top", "containerId": None, "originalText": over.get("text", ""),
                      "autoResize": False, "lineHeight": 1.25})
        if over["type"] == "arrow":
            b.update({"startBinding": None, "endBinding": None, "startArrowhead": None,
                      "endArrowhead": "arrow", "points": over.get("points", [[0, 0], [80, 0]])})
        return b

    def build(specs):
        return [el({**s, "id": s.get("id", f"e{i}"), "index": f"a{i}"}) for i, s in enumerate(specs)]

    def rect(x, y, w, h, bg, stroke, sw=2, rn=True, op=100):
        return {"type": "rectangle", "x": x, "y": y, "width": w, "height": h,
                "backgroundColor": bg, "strokeColor": stroke, "strokeWidth": sw,
                "roundness": {"type": 3} if rn else None, "opacity": op}

    def txt(x, y, t, c, size=16, ff=2, w=300, h=24, a="left"):
        return {"type": "text", "x": x, "y": y, "width": w, "height": h, "text": t,
                "strokeColor": c, "fontSize": size, "fontFamily": ff, "textAlign": a}

    # ---- Componentes (reciben paleta para dark/light) ----
    def btn_primary(pal):
        r = rect(0, 0, 180, 48, pal["primary"], pal["primary"])
        t = txt(0, 0, "Get Started", pal["on_primary"], 16, 2, 180, 24)
        t["x"] = 52; t["y"] = 13; t["textAlign"] = "center"; t["width"] = 80
        return "Button / Primary", [r, t]

    def btn_secondary(pal):
        r = rect(0, 0, 160, 48, "transparent", pal["primary"])
        t = txt(0, 0, "Learn More", pal["primary"], 16, 2)
        t["x"] = 38; t["y"] = 13; t["textAlign"] = "center"; t["width"] = 84
        return "Button / Secondary", [r, t]

    def btn_login(pal, ox=0, oy=0):
        r = rect(ox, oy, 84, 36, "transparent", pal["primary"])
        t = txt(ox, oy + 9, "Log in", pal["primary"], 13, 2, 84, 20); t["textAlign"] = "center"
        return "Button / Login", [r, t]

    def btn_signup(pal, ox=0, oy=0):
        r = rect(ox, oy, 104, 36, pal["primary"], pal["primary"])
        t = txt(ox, oy + 9, "Sign up", pal["on_primary"], 13, 2, 104, 20); t["textAlign"] = "center"
        return "Button / Sign up", [r, t]

    def input_field(pal):
        r = rect(0, 0, 300, 48, pal["surface2"], pal["border"])
        t = txt(0, 0, "your@email.com", pal["muted"], 14, 2); t["x"] = 14; t["y"] = 15
        return "Input / Field", [r, t]

    def card(pal):
        r = rect(0, 0, 280, 180, pal["surface"], pal["border"])
        t1 = txt(0, 0, "Card title", pal["on_surface"], 18, 2); t1["x"] = 16; t1["y"] = 16
        t2 = txt(0, 0, "Short description text goes here.", pal["on_surface"], 14, 2, 250, 20)
        t2["x"] = 16; t2["y"] = 46
        return "Card", [r, t1, t2]

    def badge(pal):
        r = rect(0, 0, 96, 28, pal["success"], pal["success"])
        t = txt(0, 0, "Active", pal["on_surface"], 13, 2); t["x"] = 22; t["y"] = 6; t["width"] = 52
        return "Badge / Pill", [r, t]

    def lang_sw(pal, ox=0, oy=0):
        r = rect(ox, oy, 36, 36, pal["surface2"], pal["border"])
        t = txt(ox, oy + 9, "ES", pal["fg"], 13, 2, 36, 20); t["textAlign"] = "center"
        return "Language Switcher", [r, t]

    def theme_sw(pal, ox=0, oy=0):
        r = rect(ox, oy, 36, 36, pal["surface2"], pal["border"])
        c = rect(ox + 10, oy + 10, 16, 16, pal["primary"], pal["primary"])
        d = rect(ox + 21, oy + 19, 8, 8, pal["secondary"], pal["secondary"])
        return "Theme Switcher", [r, c, d]

    def navbar(pal):
        r = rect(0, 0, 960, 64, pal["bg"], pal["border"], rn=False)
        logo = txt(0, 0, "Trust Work", pal["primary"], 20, 2); logo["x"] = 24; logo["y"] = 22
        nav = txt(0, 0, "Home    Jobs    Docs", pal["muted"], 15, 2, 220, 20); nav["x"] = 270; nav["y"] = 24
        els = [r, logo, nav]
        for e in theme_sw(pal, 512, 14)[1]:
            els.append(e)
        for e in lang_sw(pal, 556, 16)[1]:
            els.append(e)
        for e in btn_login(pal, 624, 14)[1]:
            els.append(e)
        for e in btn_signup(pal, 720, 14)[1]:
            els.append(e)
        return "Navbar", els

    def heading(pal):
        t = txt(0, 0, "How it works", pal["fg"], 32, 2, 360, 40)
        line = rect(0, 44, 56, 5, pal["primary"], pal["primary"], rn=True)
        return "Section / Heading", [t, line]

    def footer(pal):
        r = rect(0, 0, 960, 120, pal["bg_deep"], pal["border"], rn=False)
        t1 = txt(0, 0, "Trust Work Escrow - escrow on Solana", pal["muted"], 14, 2, 420, 20); t1["x"] = 24; t1["y"] = 24
        t2 = txt(0, 0, "(c) 2026", pal["muted"], 12, 2); t2["x"] = 24; t2["y"] = 92
        return "Footer", [r, t1, t2]

    def build_library(pal):
        comps = [lambda: btn_primary(pal), lambda: btn_secondary(pal), lambda: input_field(pal),
                  lambda: card(pal), lambda: badge(pal), lambda: navbar(pal),
                  lambda: heading(pal), lambda: footer(pal),
                  lambda: lang_sw(pal), lambda: theme_sw(pal),
                  lambda: btn_login(pal), lambda: btn_signup(pal)]
        items = []
        for fn in comps:
            name, els = fn()
            items.append({"id": f"lib-{random.randint(1000,9999)}", "status": "published", "name": name,
                          "created": 1700000000000, "lastUpdated": 1700000000000, "elements": build(els)})
        return {"type": "excalidrawlib", "version": 2, "source": "https://excalidraw.com", "libraryItems": items}

    def build_landing(pal):
        L = []
        def add(e):
            e["id"] = f"l{len(L)}"; e["index"] = f"a{len(L)}"; L.append(el(e)); return e

        nb = navbar(pal)
        for e in nb[1]:
            e["x"] += 40; e["y"] += 20; e["id"] = f"l{len(L)}"; e["index"] = f"a{len(L)}"; L.append(el(e))
        h1 = txt(40, 140, "Trust Work Escrow", pal["fg"], 48, 2, 600, 56); add(h1)
        sub = txt(40, 212, "Decentralized escrow for freelancers & clients on Solana.", pal["muted"], 18, 2, 560, 24); add(sub)
        bp = btn_primary(pal); bp[1][0]["x"] = 40; bp[1][0]["y"] = 280; bp[1][1]["x"] = 92; bp[1][1]["y"] = 293
        for e in bp[1]:
            e["id"] = f"l{len(L)}"; e["index"] = f"a{len(L)}"; L.append(el(e))
        bs = btn_secondary(pal); bs[1][0]["x"] = 240; bs[1][0]["y"] = 280; bs[1][1]["x"] = 278; bs[1][1]["y"] = 293
        for e in bs[1]:
            e["id"] = f"l{len(L)}"; e["index"] = f"a{len(L)}"; L.append(el(e))
        ill = rect(620, 130, 380, 320, pal["surface"], pal["border"]); add(ill)
        ilt = txt(640, 280, "escrow flow\ndiagram", pal["primary"], 18, 3, 340, 48); add(ilt)
        fh = heading(pal); fh[1][0]["x"] = 40; fh[1][0]["y"] = 420; fh[1][1]["x"] = 40; fh[1][1]["y"] = 464
        for e in fh[1]:
            e["id"] = f"l{len(L)}"; e["index"] = f"a{len(L)}"; L.append(el(e))
        for i, t in enumerate(["Secure escrow", "Instant payout", "On-chain disputes"]):
            cx = 40 + i * 300
            c = card(pal); c[1][0]["x"] = cx; c[1][0]["y"] = 500; c[1][1]["x"] = cx + 16; c[1][1]["y"] = 516
            c[1][2]["x"] = cx + 16; c[1][2]["y"] = 546
            for e in c[1]:
                e["id"] = f"l{len(L)}"; e["index"] = f"a{len(L)}"; L.append(el(e))
        hw = heading(pal); hw[1][0].update({"text": "How it works", "x": 40, "y": 740}); hw[1][1]["x"] = 40; hw[1][1]["y"] = 784
        for e in hw[1]:
            e["id"] = f"l{len(L)}"; e["index"] = f"a{len(L)}"; L.append(el(e))
        for i, s in enumerate(["1. Create job", "2. Fund escrow", "3. Release on approval"]):
            cx = 40 + i * 300
            st = rect(cx, 820, 280, 80, pal["surface2"], pal["border"]); add(st)
            stx = txt(cx + 16, 845, s, pal["fg"], 15, 2, 250, 24); add(stx)
        ft = footer(pal); ft[1][0]["x"] = 40; ft[1][0]["y"] = 960; ft[1][1]["x"] = 64; ft[1][1]["y"] = 984; ft[1][2]["x"] = 64; ft[1][2]["y"] = 1052
        for e in ft[1]:
            e["id"] = f"l{len(L)}"; e["index"] = f"a{len(L)}"; L.append(el(e))
        return {"type": "excalidraw", "version": 2, "source": "https://excalidraw.com",
                "elements": L, "appState": {"gridSize": None, "viewBackgroundColor": pal["bg"]}, "files": {}}

    Pl = P["light"]

    # ---- tokens.md ----
    def color_rows(pal):
        return "| `bg` | `%s` | fondo base |\n| `bg-deep` | `%s` | secciones / footer |\n| `surface` | `%s` | tarjetas / paneles |\n| `surface-2` | `%s` | inputs / hover |\n| `fg` | `%s` | texto principal |\n| `muted` | `%s` | texto secundario |\n| `primary` | `%s` | accent / marca |\n| `secondary` | `%s` | highlight |\n| `title` | `%s` | titulos |\n| `border` | `%s` | bordes |\n| `on-surface` | `%s` | texto SOBRE surface |\n| `on-primary` | `%s` | texto SOBRE boton primario |\n| `gradient` | `%s` | degradado de marca |" % (
            pal["bg"], pal["bg_deep"], pal["surface"], pal["surface2"], pal["fg"], pal["muted"],
            pal["primary"], pal["secondary"], pal["title"], pal["border"], pal["on_surface"], pal["on_primary"], pal["gradient"])
    tokens = f"""# Trust Work Escrow - Design Tokens - tema `{theme}`

{P['label']}
Fuente: {P['source']}

## Color - Dark
| Token | Hex | Uso |
|--------|------|------|
{color_rows(P)}

## Color - Light
| Token | Hex | Uso |
|--------|------|------|
{color_rows(Pl)}

## Typography
{P['font']}
Escala: """ + " - ".join(f"{n} {sz}px/{wt}" for n, sz, wt in P["scale"]) + """

## Spacing (base 32px / 8pt grid)
4 - 8 - 12 - 16 - 24 - 32 - 48 - 64 - 96

## Radius
`sm` 6 - `md` 12 - `lg` 16 - `pill` 999

## Flujo
Wireframe (Excalidraw, low-fi) -> Penpot (alta fidelidade + tokens) -> codigo.
"""

    # ---- design.md (estilo Solana) ----
    full = "".join(
        f"| {i+1} | `{v}` | role |\n"
        for i, v in enumerate([P["bg"], P["bg_deep"], P["surface"], P["surface2"],
                               P["fg"], P["muted"], P["primary"], P["secondary"], P["border"], P["success"]])
    )
    css_primary = (".btn-primary {\n  background: " + P["primary"] + ";\n  color: " + P["on_primary"]
                    + ";\n  border-radius: 12px;\n  padding: 12px 20px;\n  "
                    "font-weight: 500;\n  border: none;\n  cursor: pointer;\n}")
    css_secondary = (".btn-secondary {\n  background: transparent;\n  color: " + P["primary"] + ";\n  "
                      "border-radius: 12px;\n  padding: 12px 20px;\n  "
                      "border: 1px solid " + P["primary"] + ";\n  cursor: pointer;\n}")
    css_card = (".card {\n  background: " + P["surface"] + ";\n  color: " + P["on_surface"]
                + ";\n  border: 1px solid " + P["border"] + ";\n  border-radius: 16px;\n  padding: 24px;\n}")
    scale_tbl = "".join(f"| {n} | {sz}px | {wt} |\n" for n, sz, wt in P["scale"])
    design = f"""# Design System - Trust Work Escrow - tema `{theme}`

> Derivado de: {P['source']} (extraido 2026-07-20).

## 1. Visual Theme & Atmosphere

{P['atmos']}

## 2. Color Palette & Roles

### Primary
- **Primary Accent** (`{P['primary']}`) - CTAs, links, highlights de marca.
- **Secondary Accent** (`{P['secondary']}`) - hover states, complementary highlights.
- **Brand Gradient** (`{P['gradient']}`) - hero, CTAs destacados, divisor de seccion.
- **Background** (`{P['bg']}`) - page background, canvas principal.
- **Background Secondary** (`{P['surface']}`) - cards, surfaces, secciones alternas.

### Text
- **Text Primary** (`{P['fg']}`) - headings y body.
- **Text Secondary** (`{P['muted']}`) - muted text, captions, placeholders.

### Borders
- **Border** (`{P['border']}`) - dividers, outlines, input borders.

### Full Palette (Dark)
| # | Hex | Role |
|---|-----|------|
{full}

## 3. Typography

- **Font:** {P['font']}

| Role | Size | Weight |
|------|------|--------|
{scale_tbl}

## 4. Component Stylings

### Primary Button
```css
{css_primary}
```

### Secondary Button
```css
{css_secondary}
```

### Card
```css
{css_card}
```

## 5. Layout Principles

- **Base spacing unit:** 32px - usar multiplos (64, 96, 128...).
- **Radius:** button 12px, card 16px, pill 999.

### Spacing Scale
| Token | Value |
|--------|-------|
| spacing-1 | 32px |
| spacing-2 | 64px |
| spacing-3 | 96px |

### Border Radius
| Token | Value | Element |
|--------|-------|----------|
| radius-button | 12px | button |
| radius-card | 16px | card |

## 6. Do's and Don'ts

### Do
- Usar `{P['bg']}` como fondo principal.
- Usar `{P['primary']}` como unico acento/CTA dominante.
- Mantener 32px como base de espaciado - todos los gaps son multiplos.
- Usar esquinas redondeadas (12px+) en elementos interactivos.
- Usar `{P['on_surface']}` para texto sobre superficies.

### Don't
- No usar colores fuera de la paleta extraida sin justificacion.
- No introducir superficies blancas puras que rompan el paleta oscuro (salvo surface clara intencional).
- No usar esquinas filudas - se sienten hostiles en este lenguaje redondeado.

## 7. Light Variant

Version clara del tema para entornos con fondo claro (archivo `landing.light.excalidraw`
y libreria `trust-work-ui-kit-dark.excalidrawlib` (dark) y `trust-work-ui-kit-light.excalidrawlib` (claro)).

| Token | Hex |
|-------|-----|
| `bg` (light) | `{Pl['bg']}` |
| `fg` (light) | `{Pl['fg']}` |
| `primary` (light) | `{Pl['primary']}` |
| `surface` (light) | `{Pl['surface']}` |
| `border` (light) | `{Pl['border']}` |
| `on-surface` (light) | `{Pl['on_surface']}` |

## 8. Theming & i18n (modelo global)

La landing y la app son **multi-tema** y **multi-idioma** por disenyo. Esto se
define desde el comienzo (no es un parche posterior).

### Temas (skins seleccionables por el usuario)
- Temas disponibles: `dcdev` (default, rojo crimson - marca), `cyan`, `solana`.
- El usuario elige su tema en el **header** (Theme Switcher) y la preferencia se
  **persiste** (localStorage). La proxima carga de la landing usa el tema guardado;
  si no hay preferencia, cae a `dcdev`.
- Todos los colores vienen de `tokens.md` (CSS variables: `--bg`, `--primary`, etc.);
  un solo set de componentes sirve para los 3 temas.
- `dcdev` es la marca; `cyan`/`solana` son skins alternativos que el usuario/cliente
  puede elegir (white-label).

### Idiomas (i18n)
- Idiomas iniciales: **Espanol (ES, default Latam)** e **Ingles (EN)**.
- Mas idiomas se agregan despues (solo sumar `design/i18n/<lang>.json`).
- Auto-detect del idioma del browser con fallback a ES.
- Todo el copy vive en `design/i18n/en.json` y `design/i18n/es.json` como **keys**;
  los wireframes muestran el copy en ingles como referencia, pero en codigo se
  resuelve via i18n (no hardcoded).
- El Language Switcher esta en el **header** (junto al Theme Switcher): muestra el
  idioma **seleccionado** como icono (p.ej. `ES`) y despliega el resto (EN, ...) al
  hacer click. Soporta idiomas multiples (no solo EN/ES).
 """

    readme = f"""# Trust Work - UI Kit (Excalidraw) - tema `{theme}`

Biblioteca de componentes reutilizables para armar wireframes de la landing y de
la app con el lenguaje visual de este tema. Colores y tipografía en
[`tokens.md`](./tokens.md) y especificación completa en [`design.md`](./design.md).

## Cómo cargar en Excalidraw
1. Abrí excalidraw.com (o el editor de VSCode).
2. Menú `O` -> **Library**.
3. **Load library** -> `trust-work-ui-kit-dark.excalidrawlib` (dark) o `-light` (claro).
4. Arrastrá los componentes a la pantalla.

## Incluye (dark + light)
Button / Primary - Button / Secondary - Input / Field - Card -
Badge / Pill - Navbar - Section / Heading - Footer -
Language Switcher - Theme Switcher -
Button / Login - Button / Sign up

## Punto de partida
`../wireframes/landing.dark.excalidraw` y `../wireframes/landing.light.excalidraw`.
"""

    # ---- escritura: dark + light ----
    with open(f"{base}/ui-kit/trust-work-ui-kit-dark.excalidrawlib", "w") as f:
        json.dump(build_library(P), f, indent=2)
    with open(f"{base}/ui-kit/trust-work-ui-kit-light.excalidrawlib", "w") as f:
        json.dump(build_library(Pl), f, indent=2)
    with open(f"{base}/ui-kit/tokens.md", "w") as f:
        f.write(tokens)
    with open(f"{base}/ui-kit/README.md", "w") as f:
        f.write(readme)
    with open(f"{base}/design.md", "w") as f:
        f.write(design)
    with open(f"{base}/wireframes/landing.dark.excalidraw", "w") as f:
        json.dump(build_landing(P), f, indent=2)
    with open(f"{base}/wireframes/landing.light.excalidraw", "w") as f:
        json.dump(build_landing(Pl), f, indent=2)
    print(f"[{theme}] lib(dark+light) landing(dark+light) tokens+design OK")


for name, pal in THEMES.items():
    gen(name, pal)
print("done")
