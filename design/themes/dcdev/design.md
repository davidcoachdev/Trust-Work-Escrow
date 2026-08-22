# Design System - Trust Work Escrow - tema `dcdev`

> Derivado de: trust-escrow/tui/src/config.rs - funcion dcdev() (extraido 2026-07-20).

## 1. Visual Theme & Atmosphere

Dark crimson - calido, tecnico, enfocado (rojo oscuro sobre casi-negro).

## 2. Color Palette & Roles

### Primary
- **Primary Accent** (`#FF3C3C`) - CTAs, links, highlights de marca.
- **Secondary Accent** (`#781414`) - hover states, complementary highlights.
- **Brand Gradient** (`linear-gradient(135deg,#FF3C3C,#781414)`) - hero, CTAs destacados, divisor de seccion.
- **Background** (`#120808`) - page background, canvas principal.
- **Background Secondary** (`#1E0E0E`) - cards, surfaces, secciones alternas.

### Text
- **Text Primary** (`#F0D2D2`) - headings y body.
- **Text Secondary** (`#8C4646`) - muted text, captions, placeholders.

### Borders
- **Border** (`#A01E1E`) - dividers, outlines, input borders.

### Full Palette (Dark)
| # | Hex | Role |
|---|-----|------|
| 1 | `#120808` | role |
| 2 | `#0A0404` | role |
| 3 | `#1E0E0E` | role |
| 4 | `#2A1414` | role |
| 5 | `#F0D2D2` | role |
| 6 | `#8C4646` | role |
| 7 | `#FF3C3C` | role |
| 8 | `#781414` | role |
| 9 | `#A01E1E` | role |
| 10 | `#B4FF64` | role |


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
  background: #FF3C3C;
  color: #F0D2D2;
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
  color: #FF3C3C;
  border-radius: 12px;
  padding: 12px 20px;
  border: 1px solid #FF3C3C;
  cursor: pointer;
}
```

### Card
```css
.card {
  background: #1E0E0E;
  color: #F0D2D2;
  border: 1px solid #A01E1E;
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
- Usar `#120808` como fondo principal.
- Usar `#FF3C3C` como unico acento/CTA dominante.
- Mantener 32px como base de espaciado - todos los gaps son multiplos.
- Usar esquinas redondeadas (12px+) en elementos interactivos.
- Usar `#F0D2D2` para texto sobre superficies.

### Don't
- No usar colores fuera de la paleta extraida sin justificacion.
- No introducir superficies blancas puras que rompan el paleta oscuro (salvo surface clara intencional).
- No usar esquinas filudas - se sienten hostiles en este lenguaje redondeado.

## 7. Light Variant

Version clara del tema para entornos con fondo claro (archivo `landing.light.excalidraw`
y libreria `trust-work-ui-kit-dark.excalidrawlib` (dark) y `trust-work-ui-kit-light.excalidrawlib` (claro)).

| Token | Hex |
|-------|-----|
| `bg` (light) | `#FFF6F6` |
| `fg` (light) | `#2A0E0E` |
| `primary` (light) | `#FF3C3C` |
| `surface` (light) | `#FCEAEA` |
| `border` (light) | `#E8C4C4` |
| `on-surface` (light) | `#2A0E0E` |

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
 