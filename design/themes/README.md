# Temas — cómo agregar uno nuevo

Cada tema es una carpeta con esta forma:

```
themes/<tema>/
├── ui-kit/
│   ├── tokens.md                         ← base de marca (colores, tipografía)
│   ├── README.md                         ← cómo cargar la librería
│   └── trust-work-ui-kit-dark.excalidrawlib ← 8 componentes (dark)
   └── trust-work-ui-kit-light.excalidrawlib ← 8 componentes (light)
├── wireframes/
│   ├── landing.dark.excalidraw         ← wireframe landing (dark)
│   └── landing.light.excalidraw        ← wireframe landing (light)
└── high-fi/                            ← (vacío) alta fidelidad en Penpot
```

## Pasos para crear `nuevo`
1. Copiá `dcdev/` a `nuevo/`.
2. Editá `nuevo/ui-kit/tokens.md` con tu paleta.
3. Agregá la entrada en el dict `THEMES` de `_gen.py` (una línea con los hex)
   y corré:
   ```bash
   python3 design/themes/_gen.py
   ```
   → regenera el `ui-kit` y la `landing` de **todos** los temas.
4. Listo. El generador usa rutas relativas, así funciona desde cualquier lado
   y el JSON de Excalidraw queda siempre válido.

## Componentes del UI kit (por tema)
Button / Primary · Button / Secondary · Input / Field · Card ·
Badge / Pill · Navbar · Section / Heading · Footer
