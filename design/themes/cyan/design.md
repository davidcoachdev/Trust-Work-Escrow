# Design System - Trust Work Escrow - tema `cyan`

> Derivado de: trust-escrow-v2/tui/src/app/state.rs (Theme) (extraido 2026-07-20).

## 1. Visual Theme & Atmosphere

Dark tech - azul cian / indigo, moderno y limpio.

## 2. Color Palette & Roles

### Primary
- **Primary Accent** (`#00D4FF`) - CTAs, links, highlights de marca.
- **Secondary Accent** (`#6464C8`) - hover states, complementary highlights.
- **Brand Gradient** (`linear-gradient(135deg,#00D4FF,#6464C8)`) - hero, CTAs destacados, divisor de seccion.
- **Background** (`#1A1A2E`) - page background, canvas principal.
- **Background Secondary** (`#20203A`) - cards, surfaces, secciones alternas.

### Text
- **Text Primary** (`#E0E0E0`) - headings y body.
- **Text Secondary** (`#646482`) - muted text, captions, placeholders.

### Borders
- **Border** (`#505078`) - dividers, outlines, input borders.

### Full Palette (Dark)
| # | Hex | Role |
|---|-----|------|
| 1 | `#1A1A2E` | role |
| 2 | `#0E0E1A` | role |
| 3 | `#20203A` | role |
| 4 | `#232342` | role |
| 5 | `#E0E0E0` | role |
| 6 | `#646482` | role |
| 7 | `#00D4FF` | role |
| 8 | `#6464C8` | role |
| 9 | `#505078` | role |
| 10 | `#50FA7B` | role |


## 3. Typography

- **Font:** Inter (web) - el TUI usa monoespaciada de terminal.

| Role | Size | Weight |
|------|------|--------|
| H1 | 48px | 700 |
| H2 | 32px | 700 |
| H3 | 24px | 500 |
| Body | 16px | 400 |
| Small | 14px | 400 |
| Caption | 12px | 400 |


## 4. Component Stylings

### Primary Button
```css
.btn-primary {
  background: #00D4FF;
  color: #E0E0E0;
  border-radius: 12px;
  padding: 12px 20px;
  font-weight: 500;
  border: none;
  cursor: pointer;
}
```

### Secondary Button
```css
.btn-secondary {
  background: transparent;
  color: #00D4FF;
  border-radius: 12px;
  padding: 12px 20px;
  border: 1px solid #00D4FF;
  cursor: pointer;
}
```

### Card
```css
.card {
  background: #20203A;
  color: #E0E0E0;
  border: 1px solid #505078;
  border-radius: 16px;
  padding: 24px;
}
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
- Usar `#1A1A2E` como fondo principal.
- Usar `#00D4FF` como unico acento/CTA dominante.
- Mantener 32px como base de espaciado - todos los gaps son multiplos.
- Usar esquinas redondeadas (12px+) en elementos interactivos.
- Usar `#E0E0E0` para texto sobre superficies.

### Don't
- No usar colores fuera de la paleta extraida sin justificacion.
- No introducir superficies blancas puras que rompan el paleta oscuro (salvo surface clara intencional).
- No usar esquinas filudas - se sienten hostiles en este lenguaje redondeado.

## 7. Light Variant

Version clara del tema para entornos con fondo claro (archivo `landing.light.excalidraw`
y libreria `trust-work-ui-kit-dark.excalidrawlib` (dark) y `trust-work-ui-kit-light.excalidrawlib` (claro)).

| Token | Hex |
|-------|-----|
| `bg` (light) | `#F4FBFE` |
| `fg` (light) | `#0E2230` |
| `primary` (light) | `#00B8D4` |
| `surface` (light) | `#EAF7FC` |
| `border` (light) | `#C2E4EE` |
| `on-surface` (light) | `#0E2230` |

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
 