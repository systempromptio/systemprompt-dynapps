# Límites de wkhtmltopdf 0.12.5 (WebKit ~2012)

El motor de render QWeb→PDF de Odoo es **wkhtmltopdf 0.12.5**, con un WebKit de
**~2012**. Todo lo que el editor genere para QWeb debe caber en ese motor. Diseñar
con CSS moderno produce un preview bonito (paged.js usa el navegador real) pero un
PDF roto.

## NO soportado (evitar en emisión y en snippets)

- **Flexbox** (`display:flex`) — ignorado o inconsistente.
- **CSS Grid** (`display:grid`).
- **`calc()`**.
- **Unidades de viewport** `vw` / `vh` / `vmin` / `vmax`.
- **Webfonts** (`@font-face` con URL) — usar fuentes del sistema.
- **`position:absolute`** entre páginas — frágil, se descoloca al paginar.
- Variables CSS complejas en el PDF (sí valen en el preview del editor).

## SÍ soportado (usar esto para maquetar)

- **Tablas** y `display:table` / `table-cell` — la herramienta de maquetación real.
- **`float`** e **`inline-block`**.
- **`page-break-before: always`** y **`page-break-after: always`** (saltos forzados).
- **`page-break-inside: avoid`** — funciona *casi siempre* (no fiable al 100% en <=0.12.6).
- Bordes, padding, márgenes, colores, tamaños en `mm`/`pt`/`cm`/`px`/`%`.
- Cabeceras/pies por clase `.header`/`.footer` (wkhtml los repite por página).

## Consecuencias de diseño en el editor

- **Columnas** → tabla de maquetación (50/50, 3 iguales, 30/70, 25/50/25), NO flex.
- **Caja/sección** → `div` con borde/padding (inline-block o bloque), no grid.
- **Nº de página** → `<span class="page"/>` / `<span class="topage"/>` (marcadores wkhtml).
- **Pie anclado al fondo físico** → no fiable con absolute; queda como refinamiento.
- El **preview paged.js** es aproximado: valida siempre con "Imprimir prueba" (PDF real).

## Nota de bordes (border-collapse)

`border-style:hidden` **gana** sobre otros en `border-collapse` — por eso "Quitar borde"
usa `hidden`, no `none`. El navegador normaliza `border` a formas como
`border-style: solid hidden hidden; border-image:none` al aplicar por lado; esas formas
son las que rompen el emisor DOCX (ver `docx-status.md`), pero en QWeb/wkhtml funcionan.

Made with report-designer-developer · v0.1.0
