# Design — Trust Work Escrow

Sistema de diseño **multi-tema** para la landing y la app. Cada tema vive en
`themes/<nombre>/` con su propio `tokens.md`, `ui-kit/` y `wireframes/`.

## Temas disponibles
| Tema    | Paleta                              | Estado                  |
|----------|--------------------------------------|-------------------------|
| `dcdev`  | rojo oscuro / crimson               | **elegido (actual)**     |
| `cyan`   | azul cian / índigo (default v2)     | anterior                |
| `solana` | morado→verde (gradiente Solana, solana.com/es) | referencia               |

## Theming e i18n (desde el arranque)
- **Temas (skins):** `dcdev` (default / marca), `cyan`, `solana`. El usuario elige en el
  **header** (Theme Switcher) y la preferencia se persiste; la próxima carga de la
  landing usa el tema guardado, o `dcdev` si no hay preferencia.
- **Idiomas:** ES (default Latam) + EN. Diccionarios en [`i18n/en.json`](./i18n/en.json)
  y [`i18n/es.json`](./i18n/es.json); para sumar un idioma basta crear `i18n/<lang>.json`.
  Auto-detect del idioma del browser con fallback a ES.
- Ambos selectores (idioma y tema) viven en el **header** (navbar) de cada wireframe.

## Cómo usar un tema
1. Elegí `themes/<tema>/`.
2. Abrí `themes/<tema>/wireframes/landing.dark.excalidraw` (o `landing.light.excalidraw` para la variante clara) en excalidraw.com.
3. Cargá `themes/<tema>/ui-kit/trust-work-ui-kit-dark.excalidrawlib` (dark) o `...-light.excalidrawlib` (claro)
   (Library → Load library).
4. Alta fidelidad: Penpot con los tokens de `tokens.md`.

## Flujo
Wireframe (Excalidraw, low-fi) → Penpot (alta fidelidad + tokens) → código
(ui-craft / frontend-design).

## Agregar un tema
Ver `themes/README.md`.
