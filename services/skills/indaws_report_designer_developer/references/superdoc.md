# SuperDoc — editor DOCX en navegador (reference acoplada al bundle)

> **ALCANCE.** Todo lo de aquí está **acoplado a la versión vendorizada** de SuperDoc
> (`indaws_docx_designer/static/lib/superdoc/superdoc.min.js`, ~5.9 MB). NO es verdad
> universal de Odoo ni de ProseMirror: son hallazgos de ingeniería inversa del bundle
> minificado de ESTA build. Antes de fiarte de cualquier nombre/unidad, **reconfírmalo
> contra el bundle** (ver §S8). Si actualizas el `.min.js`, revalida todo.

## Contexto del pivot (OnlyOffice → SuperDoc)

`indaws_docx_designer` **pivotó de OnlyOffice a SuperDoc**. Motivo: OnlyOffice exige un
Document Server (contenedor externo) → **no corre en Odoo.sh**. SuperDoc es un editor DOCX
100% navegador (ProseMirror + Yjs + JSZip), sin server → **Odoo.sh-native**. Solo cambió la
CAPA de edición; el backend (modelo, render docxtpl, IA, semilla QWeb→DOCX) se reutiliza.
Licencia SuperDoc: **AGPL-3** (herramienta interna entregada como fuente → OK).

- Frontend: `static/src/superdoc_editor/` (`superdoc_editor.js/.xml/.scss`, `table_builder.js/.xml`, `field_wizard.js/.xml`).
- Acción: `action_open_superdoc()` → client action `indaws_docx_designer.superdoc` `{designer_id, attachment_id, name}`.
- Boot: `superdoc_boot()` → `{name, attachment_id, res_model, fields, records, catalog}`.
- Embebido: `new SD({selector, toolbar:'#id', document:url, documentMode:'editing', onReady})`.

## S2 — API del editor (Tiptap-like) y carga del bundle

- El editor activo (`activeEditor`) expone una API **estilo Tiptap**:
  `activeEditor.state` / `.view` / `.commands` (`insertContent`, `insertContentAt`, `deleteRange`) /
  `.chain()` / `.on('update'|'transaction')` / `.registerPlugin()`. Constructor con `onReady` / `onEditorUpdate`.
- El global exporta `Editor` **+ helpers**, pero **NO `Plugin` ni `Decoration`** de ProseMirror.
  → No puedes crear decoraciones de la forma canónica; el resaltado de tags es best-effort
    (extracción de clases en runtime, **frágil**).
- **Bundle self-contained** (~5.9 MB): **lazy-load por `<script>`**, **NO** en `assets_backend`
  (metería 5.9 MB en todos los backends). Cargar solo cuando se abre el client action.

## S1 — Ancho de tabla (el gotcha que más cuesta)

El ancho total de una tabla en SuperDoc **NO es el `colwidth` de las celdas**: es
`tableProperties.tableWidth` **del NODO tabla**.

- Unidades OOXML: `pct` (5000 = 100%), `dxa` (twips), `px` (= twips/15). `type:"auto"` → `null` →
  el navegador estira la tabla. Acompañar SIEMPRE de `tableLayout:"fixed"`.
- Anchos de **columna** = `colwidth` (px) en la 1ª fila, o el attr `grid` (twips) de la tabla.
- **`insertContent` (HTML) IGNORA `style=width` y `data-colwidth` para el ancho TOTAL** → no basta
  con inyectar HTML con `width:100%`. Hay que fijarlo por **transacción ProseMirror** (ver S7).
- Preferir `pct` (relativo a la página) sobre `dxa` para que el ancho sea porcentual real.
- Implementación de referencia: `_applyTableWidth` → `tableProperties.tableWidth = {value: pct*50, type:"pct"}` + `tableLayout:"fixed"`.

## S7 — Cambiar atributos de un nodo por transacción ProseMirror (en OWL)

Como no hay `Plugin`/`Decoration` expuestos, para mutar attrs de un nodo (p.ej. el ancho de S1):

1. Localizar el nodo recorriendo `state.doc.descendants((node, pos) => …)`.
2. `tr = state.tr.setNodeMarkup(pos, null, {...node.attrs, ...cambios})`.
3. `view.dispatch(tr)`.

`pos` de un descendiente = `posDelNodoPadre + 1 + posRelativa`. (Regla ProseMirror: la posición
interna del primer hijo es padre+1.)

## S3 — Celdas y filas (docxtpl, refuerza `report-docx-template`)

- Toda celda **DEBE** contener un `<p>`; una celda "pelada" rompe el export.
- `{%tr%}` en la **misma fila** peta (docxtpl 0.16.7 **y** 0.20) → usar for/endfor en **filas
  separadas** o `{%p%}`. (Ver `report-docx-template/references/docxtpl-syntax.md`.)

## S8 — Ingeniería inversa de un bundle minificado (técnica repetible)

Cuando integras JS de terceros minificado y **sin docs de API** (SuperDoc aquí):

1. **Export map**: grep del patrón `Nombre:()=>impl` (rollup/esbuild) para ver qué exporta el global.
2. **Definiciones de nodo**: buscar `addAttributes(){return{…}}`, `parseHTML`, `renderHTML`,
   `addCommands` (patrón Tiptap/ProseMirror-node) para saber attrs y comandos reales.
3. **Funciones de render / unidades**: localizar dónde se escribe el OOXML (`w:tblW`, `pct`, `dxa`)
   para confirmar **unidades** antes de emitir (no asumir px vs twips).
4. **Confirmar contra el código, no contra la intuición**: los nombres cortos (`r1e`, `jfe`) cambian
   entre builds → cada dato de esta reference es válido para ESTE `.min.js`.

## Debug (ver también `odoo-owl-component-builder`)

- **S5 — debug-in-the-UI**: sin consola del usuario, instrumentar con `notification.add()`
  volcando valores reales (`getBoundingClientRect`, ancho px de tabla/contenedor) para tener
  verdad de campo en vez de suponer. Fue lo que destapó la regresión de ancho al 100%.

## Estado / pendientes (handoff)

- **Regresión ancho 100%** (en curso): 50% encoge bien, 100% no vuelve a ancho completo pese a que
  `_applyTableWidth` es simétrico. Instrumentado con toast `X% · tabla=NNNpx cont=MMMpx` (S5).
- Migrar wizard de tabla a `ModelFieldSelector` para columnas y campos `line.*` (necesita comodel del bucle).
- ~~Retirar OnlyOffice del manifest~~ **HECHO** (verificado 2026-08-07: `depends = ["base","web","indaws_report_docx","indaws_report_designer"]` y **0** referencias a onlyoffice en todo el addon). Lo que puede quedar es entorno, no código: el conector clonado y el contenedor `onlyoffice-ds` del compose de EUROLOTES.

Made with report-designer-developer · v0.2.0
