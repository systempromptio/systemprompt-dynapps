# Arquitectura de `indaws_report_designer`

## Invariante central: el chip neutral

El editor (OWL sobre `@html_editor`) NO produce QWeb ni DOCX. Produce **HTML con
chips neutrales**: `<span data-*>` (elementos inline) y `<tr data-loop>` (tablas).
Guarda ese HTML en tres campos Text: `body_html`, `header_html`, `footer_html`.

Cada **emisor** traduce esos chips a su formato:
- `_transform_qweb(html)` → QWeb (`t-field`, `t-out`, `t-foreach`, `t-if`, …). **Foco actual.**
- `_transform_docx(html)` → docxtpl (`{{ ctx.path }}`, `{%tr for%}`, `{% if %}`). **Aparcado** (`docx-status.md`).

Ventaja: un elemento nuevo se define una vez como chip y cada emisor decide cómo
pintarlo. El editor nunca conoce la sintaxis de salida.

## Inventario de marcadores `data-*`

| Marcador | Elemento | Emite QWeb |
|----------|----------|-----------|
| `data-field` (+ `data-widget` / `data-image` / `data-hide-empty` / `data-groups`) | Campo drill-down | `t-field="doc.<path>"` (o `line.<path>` en loop) con `t-options`, `t-if`, `groups=` |
| `data-expr` | Expresión / fórmula | `t-out` de expresión validada por ast |
| `data-helper` | Función con nombre | `t-out="doc.env['indaws.report.designer'].browse(id).report_helper('name',doc,line)"` |
| `data-barcode` (+ `data-bc-value` / `-w` / `-h`) | QR / Code128 / EAN13 | `<img t-att-src="'/report/barcode/?...'">` |
| `data-linenum` | Nº de línea (en loop) | `t-out="line_index+1"` |
| `data-i18n` (+ `data-i18n-def`) | Texto traducible | `t-out="{lang:txt}.get(doc.env.context.get('lang'), def)"` |
| `data-loop` (+ `data-group-by` / `data-row-if`) | Fila de tabla de registros | `<tr t-foreach="doc.<o2m>" t-as="line">` (o agrupado / condicional) |
| `data-col-cond` | Condición de columna (JSON) | `t-if` en `th`+`td` (columna aparece/desaparece) |
| `data-show-if` | Visibilidad condicional | `t-if` (merge con hide-empty) |
| `data-style-if` | Estilo condicional (JSON `{expr,css}`) | `t-att-style="'base;'+('css' if expr else '')"` |
| `data-pagebreak` | Salto de página | `page-break-after:always` (sin el texto guía) |
| `data-chart` | Gráfico sobre o2m (spec JSON) | `<img t-att-src="…_chart_png(doc,idx)">` (reportlab.graphics) |
| `data-pagenum` | Nº de página | `<span class="page"/>` / `<span class="topage"/>` (wkhtml) |

## Orden de pases en `_transform_qweb` (CRÍTICO)

```
campos → expr → helper → linenum → barcode → i18n → show/style-if → loops (agrupado o normal) → col-cond → strip
```

Reglas de orden:
- El **pase de campo preserva** `data-show-if`/`data-style-if` (no los borra) para
  que el pase genérico de visibilidad los aplique después.
- El pase de **loops** agrupa runs consecutivos de `tr[data-loop]` en UN `t-foreach`.
  Si el tr tiene `data-group-by` → `t-foreach report_groups(doc,loop,campo)` + cabecera
  de grupo + subtotales (columnas numéricas detectadas por su `t-field`). Si tiene
  `data-row-if` → dentro del foreach un `t-if` por línea (secciones/notas intercaladas).
- El pase **strip** es el ÚLTIMO: quita todos los `data-*` del HTML final. Por eso un
  chip nuevo debe estar en `strip_attributes` (2 listas: qweb y docx) o su atributo
  sobrevive muerto en el PDF.

## `layout_mode` (Selection)

- `custom` — solo el cuerpo del editor (sin cabecera de empresa).
- `odoo` — envuelto en `web.external_layout` (cabecera/pie de la compañía).
- `none` — sin layout.
- Si vacío, cae implícitamente a `external_layout`. **Fijarlo explícito** al generar.

## Cabecera / pie de página

- Campos `header_html` / `footer_html` (2 editores `@html_editor` aparte del cuerpo).
- Emisión QWeb: `<div class="header">…</div>` / `<div class="footer">…</div>`.
  wkhtml **extrae por clase**, no por posición → el orden en el arch da igual, pero el
  arch se reordena header→article→footer por claridad.
- **Bug histórico**: preview/generate mandaban `false` cuando no había cabecera → borraba
  el div → caía a `external_layout` de Odoo → 2 páginas. Enviar siempre el HTML real.
- Repetición física por página la hace wkhtml (clases `.header`/`.footer`).

## Preview con paged.js (v0.4.3 vendorizado)

- `static/lib/pagedjs/` expone `window.Paged`. `preview()` → `_paginate` →
  `new window.Paged.Previewer().preview('<style>@page{...}</style>'+html, [], ref.el)`,
  disparado por `onPatched` + `_pendingHtml`; fallback a HTML plano si falla.
- **Neutralizar `min-height`** de página entera (`.page`/`#wrapwrap`/`main`) o el pie se
  empuja a una 2ª página fantasma.
- Odoo **NO scopea CSS** → nada de `:deep()`; los estilos del preview son globales.

## Editor activo (cuerpo / cabecera / pie)

3 instancias de `@html_editor`. `_setActiveFromTarget` marca cuál está activo según
dónde pinchas; `_ae()` devuelve el editor activo, `_aeSave()` persiste su contenido.
Insertar chip y clic-chip actúan sobre el editor activo.
- El listener de clic-chip va en `t-on-click` de `.ird-sheet` (burbujeo), **NO** en
  `onLoad` (`editor.editable` es null en onLoad → crash `editor.editable is null`).
- `_syncBody` protege editores de cabecera/pie desmontados (`_grab` con editable vivo + try/catch).

## Inspector contextual (pestañas)

Al pinchar un elemento, panel derecho con pestañas según contexto: **Campo / Celda /
Fila / Columna / Tabla / Agrupar**. También menú contextual (clic derecho sobre el
papel) con el Popover de Odoo (`IrdContextMenu`, niveles root/table/chart/snippet).
El **scope** de "insertar campo" sale de la CELDA pinchada (`ctxLoopComodel`), no de la
selección (el clic derecho puede dejar el caret fuera).

Panel **Propiedades** de chip: tipo detectado (badge vía loadFieldInfo), path +
Cambiar campo…, Etiqueta, Formato/Widget (según tipo), Ocultar si vacío, Visible para
grupos, Mostrar si / Estilo si, Quitar.

## `qweb_override` (fuente manual) + Historial

- Pestaña **QWeb** (`showSource`→`get_qweb_source`). Campo `qweb_override` (Text): si se
  rellena, `_emit_qweb` lo usa tal cual (valida XML en `save_qweb_override`;
  `clear_qweb_override` regenera). CodeEditor ace en el frontend.
- **Historial**: `history_json` (lista JSON máx 20), `_push_history(reason)` en Generar y
  antes de plantilla/import; `versions_for_ui()` / `restore_version(i)` (guarda estado
  actual antes de restaurar → deshacer restauración). Frontend remonta editor (editorKey++).

## Paperformat / orientación

- `paperformat_id` (selección de usuario, `report.paperformat` de Odoo) +
  `auto_paperformat_id` (interno A4/orientación cuando no hay selección).
- `_paper_dims` deriva dims del pf (o A4+orientation). `_report_paperformat()` devuelve el
  del usuario o crea/actualiza el auto. Editor usa variables CSS `--ird-pw`/`--ird-ph`.
- Orientación: campo `orientation` (portrait/landscape), `_paper_dims` intercambia w/h.

Made with report-designer-developer · v0.1.0
