# Design System - Trust Work Escrow - tema `solana`

> Derivado de: solana.com/es (design system oficial) - gradiente #9945FF -> #14F195 (extraido 2026-07-20).

## 1. Visual Theme & Atmosphere

High-contrast dark mode con el degradado morado->verde de Solana - moderno, tecnico, enfocado. Tarjetas lilas sobre fondo oscuro.

## 2. Color Palette & Roles

### Primary
- **Primary Accent** (`#14F195`) - CTAs, links, highlights de marca.
- **Secondary Accent** (`#9945FF`) - hover states, complementary highlights.
- **Brand Gradient** (`linear-gradient(135deg,#9945FF,#14F195)`) - hero, CTAs destacados, divisor de seccion.
- **Background** (`#121212`) - page background, canvas principal.
- **Background Secondary** (`#ECE4FD`) - cards, surfaces, secciones alternas.

### Text
- **Text Primary** (`#FFFFFF`) - headings y body.
- **Text Secondary** (`#999999`) - muted text, captions, placeholders.

### Borders
- **Border** (`#2A2333`) - dividers, outlines, input borders.

### Full Palette (Dark)
| # | Hex | Role |
|---|-----|------|
| 1 | `#121212` | role |
| 2 | `#0D0C11` | role |
| 3 | `#ECE4FD` | role |
| 4 | `#181818` | role |
| 5 | `#FFFFFF` | role |
| 6 | `#999999` | role |
| 7 | `#14F195` | role |
| 8 | `#9945FF` | role |
| 9 | `#2A2333` | role |
| 10 | `#14F195` | role |


## 3. Typography

- **Font:** Diatype (web font) - heading + body. Weight 500, letter-spacing -2.64px en H1.

| Role | Size | Weight |
|------|------|--------|
| H1 | 88px | 500 |
| H2 | 64px | 500 |
| H3 | 24px | 500 |
| Body L | 24px | 500 |
| Body | 21px | 400 |
| Small | 16px | 400 |
| Caption | 16px | 400 |


## 4. Component Stylings

### Primary Button
```css
.btn-primary {
  background: #14F195;
  color: #0D0C11;
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
  color: #14F195;
  border-radius: 12px;
  padding: 12px 20px;
  border: 1px solid #14F195;
  cursor: pointer;
}
```

### Card
```css
.card {
  background: #ECE4FD;
  color: #0D0C11;
  border: 1px solid #2A2333;
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
- Usar `#121212` como fondo principal.
- Usar `#14F195` como unico acento/CTA dominante.
- Mantener 32px como base de espaciado - todos los gaps son multiplos.
- Usar esquinas redondeadas (12px+) en elementos interactivos.
- Usar `#0D0C11` para texto sobre superficies.

### Don't
- No usar colores fuera de la paleta extraida sin justificacion.
- No introducir superficies blancas puras que rompan el paleta oscuro (salvo surface clara intencional).
- No usar esquinas filudas - se sienten hostiles en este lenguaje redondeado.

## 7. Light Variant

Version clara del tema para entornos con fondo claro (archivo `landing.light.excalidraw`
y libreria `trust-work-ui-kit-dark.excalidrawlib` (dark) y `trust-work-ui-kit-light.excalidrawlib` (claro)).

| Token | Hex |
|-------|-----|
| `bg` (light) | `#FFFFFF` |
| `fg` (light) | `#0D0C11` |
| `primary` (light) | `#0E9E5C` |
| `surface` (light) | `#ECE4FD` |
| `border` (light) | `#D9D2EC` |
| `on-surface` (light) | `#0D0C11` |

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
 