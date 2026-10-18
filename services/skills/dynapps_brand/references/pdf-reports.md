# Dynapps PDF Reports — brand layer

> **Page-break mechanics, multi-page tables, footer pinning, tfoot/specificity gotchas live
> in the single source of truth:** `pdf-renderer/references/playwright-pdf-guide.md`
> (sections "Page Break Control" + "Multi-page report patterns 9–14"). This file only adds the
> **Dynapps brand specifics** on top of that engine.

PDF engine: **`pdf-renderer`** (Playwright). Render with:
```bash
node <pdf-renderer>/scripts/html-to-pdf.js report.html report.pdf --margin=0 --prefer-css-page-size --delay=400
```

## Canonical report template + scripts (SSOT — live here in dynapps-brand)
Any skill that needs a Dynapps branded report PDF (migration reports, etc.) **uses these**;
it must NOT hand-roll/clone the CSS (that is how the same artifacts spread to every report
with no single place to fix them):
- **Template**: `assets/templates/migration-report-template.html` — full brand shell
  (cover, footer band, summary cards, tables, findings) with `{{placeholders}}`. Copy it
  and fill content; do not re-author the CSS. **La CSS es genérica** (no específica de
  migración): sirve para cualquier informe de marca (rescue pack, auditoría, análisis). Reusa
  el `<style>`, la cover y los componentes (`.panel`, tablas, `.find`, `.sc`); solo cambia el
  contenido, el badge de la cover y el nº de secciones. Los `.sc .num` admiten notas 0-10
  (`4,9<span>/10</span>`), no solo horas.
- **Render + footer band**: `scripts/gen.js` (content.pdf + band.pdf) then `scripts/stamp.py`
  (stamps the band flush to every page's bottom edge):
  ```bash
  # 4º arg opcional = etiqueta IZQUIERDA de la banda (default "Dynapps — Odoo Migration Services").
  # Pásalo en informes que NO son de migración (rescue pack, auditoría) para no mentir en el pie.
  node <dynapps-brand>/scripts/gen.js report.html <outdir> "© 2026 Dynapps · <Client> · Confidencial" "Dynapps — Rescue Pack"
  python3 <dynapps-brand>/scripts/stamp.py <outdir>/content.pdf <outdir>/band.pdf report.pdf
  ```
- **Cada sección en página nueva** (opcional): añade `break-before:page` a la regla `.panel`
  (`.panel{…;break-before:page;}`). La cover es full-bleed → la 1ª sección cae en la pág. 2 sin
  hoja en blanco. Útil en informes de secciones largas (rescue pack, auditoría).
- **QA (mandatory)**: `python3 <dynapps-brand>/scripts/verify.py report.pdf` → must print
  `RESULT: OK` (cover full-bleed + lateral margins + no lateral hairlines) before delivering.

## Dynapps-specific rules
- **PDFs are always light**: snow/white body, white cards. `--midnight #050543` is reserved
  for the **cover**, the **TOTAL card**, the **Critical-Points panel** and the **footer band**.
  (Colors/typography → `references/brand-system.md`, extended palette.)
- **Embed Readex Pro by absolute `file:///` path** in `@font-face` (Playwright headless may not
  fetch Google Fonts). Weights used: 300/400/500/600/700.
- **Logo**: white SVG (`assets/logos/Dynapps_Logo_Horizontal_Color_White.svg`) on dark cover/footer;
  reference by absolute `file:///` path.
- **Cover**: full-bleed `--midnight`, top nav (logo + a "MIGRATION REPORT" pill) with a thin
  gradient/divider line, client name in `--orange`, version line in white, gradient line above
  the cover footer (brand signature).
  - ⚠️ **Full-bleed = `width:100%;height:100vh`. NEVER `width:210mm;height:297mm`** to "match"
    A4. In the Playwright render a physical `mm` does **not** map to 100% of the page box
    (it renders at ~93%), leaving a white strip on the **right and bottom** of the cover.
    Size full-bleed elements relative to the page (`%`/`vh`), never with mm equal to the paper.
- **Footer band (every page)**: `--deep #030330` with the gradient line on top
  (Violet→Carmine→Orange), white logo + "Dynapps — Odoo Migration Services" left,
  confidential/meta right. Must appear on **every** page.
  - ⚠️ Two dead ends, both verified: **`position: fixed` no longer repeats per page**
    in current Chromium (paints only where it flows, usually mis-positioned on the last
    page); and **Chromium's native footer** (`displayHeaderFooter`+`footerTemplate`)
    leaves a **~5mm white strip below the band** — its box is inset from the paper edge
    and its height is content-driven, so a full-bleed band can't touch the bottom edge.
  - ✅ **Reliable method — render then STAMP** (the band is a separate PDF overlaid on
    every page with PyMuPDF/`fitz`, so it sits flush to the paper edge):
    1. **Content** — render the HTML with **no footer**, full-bleed cover, and the band
       height reserved at the bottom of content pages:
       ```css
       @page { size:A4; margin:13mm 0 18mm; }   /* 18mm bottom reserved (14mm band + 4mm gap) */
       @page:first { margin:0; }                /* full-bleed cover */
       ```
       Render via Playwright directly with `{preferCSSPageSize:true, printBackground:true}`
       and **no margin option** (the wrapper `html-to-pdf.js` forces a uniform margin
       that overrides `@page:first{margin:0}` and breaks the full-bleed cover).
    2. **Band** — render a 1-page PDF sized exactly `@page{size:210mm 14mm;margin:0}`:
       dark `#030330` bar, 3px gradient line on top, white logo + service left,
       `©…·Confidential` right (per project).
    3. **Stamp** — overlay the band on the bottom edge of every page:
       ```python
       import fitz
       doc, band = fitz.open("content.pdf"), fitz.open("band.pdf")
       bpt = 14/25.4*72
       for p in doc:
           p.show_pdf_page(fitz.Rect(0, p.rect.height-bpt, p.rect.width, p.rect.height), band, 0)
       doc.save("final.pdf", garbage=4, deflate=True)
       ```
    Scripts: **`scripts/gen.js`** (content+band) + **`scripts/stamp.py`** (here in
    dynapps-brand — the SSOT). The band stamps on the cover too (consistent) — so **omit
    the cover's own `.coverfoot`** to avoid a duplicate signature.
- Accent word in each section heading uses `--carmine` (or `--violet`); section eyebrows
  `01 · 02 · 03 · 04`.
- ⚠️ **Dark total/subtotal rows (`tfoot`)**: a generic `td.h{color:var(--midnight)}`
  (class) out-specifies `tfoot td{color:white}` (elements), so the hours value in a
  dark total bar renders **dark-on-dark / invisible**. Add `tfoot td.h{color:var(--tw)}`
  and `td.h{white-space:nowrap}` (stops "21 h" wrapping to two lines in the narrow column).
- **Don't orphan a table's total row**: `tr{break-inside:avoid}` + `tfoot{break-before:avoid}`
  keep the total attached to its last data row; mark a small table's panel `keep`
  (`break-inside:avoid`) so it stays whole, but NOT a multi-page table (Chromium then
  breaks it anyway and the `keep` is wasted).
- ⚠️ **No `border`/`border-radius`/`box-shadow` on a container that can span a page break**
  (e.g. the section `.panel` wrapping a long table). When such a box is forced to break
  (content taller than one page), Chromium draws its **side borders down every page
  fragment** → stray vertical hairlines on both edges + a broken rounded box at the seam.
  `break-inside:avoid` does **not** save it (ignored when the content can't fit a page).
  Keep panels border-less; let the **inner** elements carry the styling (tables with their
  own header/footer bars and row separators, `.find` cards, summary `.sc`) — those break cleanly.
- ⚠️ **Nothing may widen its column** (symptom: content "shifted to the right", right margin
  smaller than the left or running off the page). A long unbreakable token — typically inline
  `code` like `relative_uom_id` or a dotted model name — sets the min-content width of its
  cell/card, and `1fr` columns or flex items grow past the page margin. Rules in the template:
  grids use `repeat(N,minmax(0,1fr))` (never bare `1fr`), flex/grid children carry
  `min-width:0`, and text/`code`/table cells use `overflow-wrap:anywhere`. Inline `code` that
  wraps also needs `box-decoration-break:clone`, or Chromium paints its background as one strip
  that can extend off the page. `verify.py` fails the render when the content is closer than
  7 mm to an edge or the right margin is >2 mm smaller than the left.

## Notas al pie / asteriscos (patrón de marca)
Para aclaraciones, salvedades o condiciones (recargos, "a confirmar", supuestos) usa **nota con
asterisco** anclada al término, no texto inline que ensucie la tabla/celda:
- Marca el término con `&#42;` (ej. `Traspaso a cloud&#42;`) — `&#42;` evita que el `*` se
  interprete como markdown/bold.
- Debajo del bloque (no en el pie de marca de la página, que es la banda Dynapps), una `.note`:
  `<p class="note">&#42; <aclaración>.</p>` Varias notas → `<br>` entre ellas o `*`, `**`, `***`.
- Estilo `.note`: **ya viene en la plantilla canónica**, no lo copies a mano. Hasta el
  2026-08-14 esta sección lo especificaba pero la plantilla no lo traía, así que cada informe
  salía con las aclaraciones a tamaño de cuerpo — parecían parte del argumento en vez de una
  salvedad. Si lo tocas, conserva las dos propiedades de salto:
  ```css
  .note{font-size:8.4pt;color:var(--grey-400);margin-top:10px;line-height:1.45;
    break-inside:avoid;break-before:avoid;}
  ```
  ⚠️ **`break-before:avoid` no es decorativo.** Sin él, una nota al final de sección se queda
  **huérfana en una página propia**. `break-inside:avoid` solo impide que la nota se parta; no
  impide que empiece página.
  **No hace falta creérselo**: `scripts/check_note_break.sh` lo demuestra en diez segundos —
  renderiza el mismo contenido con y sin la propiedad y dice si la nota queda sola. Con 45 líneas
  de cuerpo (donde cae justo en el borde de un A4 con estos márgenes) el resultado es
  `sin → huérfana sí · con → huérfana no`. Córrelo antes de tocar esa línea, y si algún día Chrome
  cambia, el script lo dirá antes de que alguien la borre por parecer redundante.
- La plantilla trae además `.note strong` (gris más oscuro, para el término que se aclara) y
  `.note code` (7.8pt, para que un identificador dentro de la nota no la desequilibre).
- La nota es **de sección**, va dentro del `.panel`. NO la metas en la banda de pie (la banda es
  fija: logo + confidencial). Mantén números limpios en las celdas; el matiz va al asterisco.

For everything about how pages break, repeat headers, keep blocks together, and pin footers,
follow the shared guide — do not re-invent it here.
