---
name: indaws-report-designer-developer
version: 0.2.8
effort: high
model: opus
description: >-
  Extiende, mantiene y depura los DOS addons Odoo v18 del report designer:
  indaws_report_designer (chips sobre @html_editor, salida QWeb/wkhtmltopdf) e
  indaws_docx_designer (editor DOCX en navegador con SuperDoc + render docxtpl de
  indaws_report_docx; OnlyOffice queda como histórico en retirada). Cubre chip
  neutral y orden de pases _transform_qweb/_transform_docx, seguridad expr/helpers,
  límites wkhtmltopdf, y en DOCX: SuperDoc embebido, docx.template/file_ids,
  gotchas docxtpl 0.20 y html4docx. NOT usar el editor (→ report-designer-author);
  NOT generar plantilla docx desde mockup (→ report-docx-template); NOT QWeb
  genérico (→ odoo-qweb-report-builder). Triggers: "extender indaws_report_designer",
  "indaws_docx_designer", "editor DOCX SuperDoc", "_transform_qweb", "report_emitter",
  "docxtpl no renderiza", "editor visual de informes falla".
allowed-tools: [Read, Write, Edit, Bash, Glob, Grep, AskUserQuestion]
---

# Report Designer Developer v0.2

> Convenciones de código y conexión a Odoo: `references/shared/coding-standards.md` y `references/shared/odoo-connection.md`.

Skill para **extender/mantener el código** de los DOS addons hermanos del report designer (emisor, chips, inspector, preview, integración OnlyOffice). *Usar* los editores = `report-designer-author`.

| Módulo | Editor | Salida | Elegir cuando… |
|--------|--------|--------|----------------|
| `indaws_report_designer` | chips sobre `@html_editor` | **QWeb** (wkhtmltopdf) | editar/heredar el **estándar de Odoo** (es QWeb) o lógica dinámica rica (`tax_totals`, sub-templates). 100% fiel vía herencia |
| `indaws_docx_designer` | **SuperDoc** (editor DOCX en navegador) | **DOCX** (docxtpl) → PDF | se quiere **tacto Word** y maquetar libre; NO edita el estándar. Plantilla se **genera** con `report-docx-template` y se rellena en SuperDoc → `references/superdoc.md` |

## Agent Routing

| Fase | Modelo | Runs as |
|------|--------|---------|
| PASO 1 — Situar el cambio | opus | Principal (con `AskUserQuestion`) |
| PASO 2 — Leer arquitectura | opus | Principal |
| PASO 3 — Delegar diseño de bajo nivel | opus | subagent por skill (owl/qweb/orm builders) |
| PASO 4 — Implementar | opus | Principal |
| PASO 5 — Test + ciclo EUROLOTES | opus | Principal (Bash) |

## Ubicación y entorno

- **Addons** en `$PROJECTS_ROOT/indawsodoo/18/`: `indaws_report_designer` (QWeb) e `indaws_docx_designer` (DOCX; `depends = ["base","web","indaws_report_docx","indaws_report_designer"]` — verificado en el manifest 2026-08-07; **`onlyoffice_odoo` ya NO es dependencia**: 0 referencias a onlyoffice en todo el addon).
- **DOCX — PIVOT OnlyOffice → SuperDoc**: el editor DOCX **pivotó** a **SuperDoc** (editor DOCX 100% navegador, sin server → Odoo.sh-native; frontend `static/src/superdoc_editor/`, bundle vendorizado lazy-load) → `references/superdoc.md`. El camino OnlyOffice (Document Server Docker `8085` + conector `onlyoffice-odoo` rama 18.0) queda como histórico en `references/docx-architecture.md`: la retirada del manifest **ya está hecha** (0 refs a onlyoffice en el addon), así que ese fichero es documentación de un camino abandonado, no de una dep viva.
- **Test = EUROLOTES v18** (`$PROJECTS_ROOT/EUROLOTES18`, BD `EUROLOTES`, Odoo host `18069`).
- **Ciclo de recarga (regla dura, NUNCA `restart`)** — [[docker-odoo-reload-cycle]]:
  ```bash
  cd $PROJECTS_ROOT/EUROLOTES18 && docker compose down && docker compose up -d
  docker compose exec -T odoo bash -lc '$ODOO_SRC/odoo18/odoo-bin -c /etc/odoo/odoo.conf -d EUROLOTES -u indaws_report_designer --stop-after-init --http-port=8072'
  ```
  Tests: añade `--test-enable --test-tags /indaws_report_designer`. El `8072` evita choque con el server ya arriba.
- **Frontend OWL NO es testeable por CLI** → validar en navegador (F12). Los tests CLI cubren el emisor.

## Referencias (leer ANTES de tocar código)

| Ref | Cuándo |
|-----|--------|
| `references/architecture.md` | Siempre — chip neutral, inventario `data-*`, orden de pases `_transform_qweb`, layout_mode, cabecera/pie, preview paged.js, editor activo, inspector |
| `references/add-a-feature.md` | **Receta clave** — añadir un elemento insertable nuevo (frontend + emisor + strip + test + ciclo) |
| `references/security.md` | Al tocar expresiones (`data-expr`), helpers (`indaws.report.helper`) o `safe_eval` |
| `references/wkhtml-limits.md` | **(QWeb-only)** Al diseñar maquetación o depurar render — motor wkhtmltopdf 0.12.5 (WebKit ~2012) |
| `references/chip-language.md` | **(QWeb-only)** Contrato de chips para generar informes con IA + bucle validate→crear→generar |
| `references/files-map.md` | Para localizar dónde vive cada cosa en AMBOS módulos (models/, static/, tests/) |
| `references/docx-architecture.md` | **(DOCX)** Al tocar `indaws_docx_designer`/OnlyOffice: arquitectura, arranque del DS+conector, render docx.template, panel de campos, IA, y gotchas verificados |
| `references/superdoc.md` | **(DOCX, PIVOT)** El editor DOCX **pivotó de OnlyOffice → SuperDoc** (editor en navegador, Odoo.sh-native). API Tiptap-like del bundle, ancho de tabla (`tableProperties.tableWidth`), transacción PM, e ingeniería inversa del bundle. **Acoplado a la build vendorizada** |

## Workflow

### PASO 1 — Situar el cambio [STOP]

Con `AskUserQuestion` si no está claro: ¿es un **elemento insertable nuevo** (chip, tabla, gráfico…), un **fix del emisor** (QWeb malformado, campo no resuelve), un **fix de UI** (inspector, preview, editor activo) o **seguridad** (expr/helper)?

- Insertable nuevo → seguir `references/add-a-feature.md` al pie de la letra.
- Fix emisor → `references/architecture.md` (orden de pases) + reproducir con test en `tests/test_report_designer.py`.
- Fix UI → `references/files-map.md` (static/src) + validar en navegador.

### PASO 2 — Leer la arquitectura [OPUS]

Leer `references/architecture.md` **entero** antes de editar. El invariante central: el editor produce HTML con **chips neutrales** `<span data-*>`/`<tr data-loop>`; **cada emisor los traduce** (`_transform_qweb`, `_transform_docx`). El orden de los pases importa (un pase que borra atributos antes de tiempo rompe pases posteriores).

### PASO 3 — Delegar diseño de bajo nivel [subagents]

- Componentes OWL / integración `@html_editor` → `odoo-owl-component-builder`.
- QWeb generado / pitfalls wkhtml → `odoo-qweb-report-builder` + `references/wkhtml-limits.md`.
- Cambios de modelo/campos (`indaws.report.designer`, `indaws.report.helper`) → `odoo-orm-modeler`.
- Convenciones suite indaws / dependencias → `common-skills:indaws-core-auditor`.

### PASO 4 — Implementar

Editar el addon siguiendo la arquitectura. Si es un insertable, tocar **los tres frentes** (frontend, emisor, listas `strip_attributes`) — omitir uno deja el chip visible en el PDF final o sin efecto.

### PASO 5 — Test + ciclo EUROLOTES

1. Añadir/actualizar test en `tests/test_report_designer.py` (patrón: instanciar designer, insertar chip en `body_html`, correr `_transform_qweb`, assert QWeb bien formado y `t-*` esperados).
2. Correr el ciclo down→up→`-u … --test-enable --test-tags /indaws_report_designer`.
3. Frontend → eyeball en navegador (paged.js/Wysiwyg no testeables por CLI).

## GOTCHAS

1. **Un chip nuevo se olvida en `strip_attributes`** → el `data-*` sobrevive al PDF como atributo muerto o rompe el pase de limpieza. Hay **DOS listas** (`_transform_qweb` y `_transform_docx`); añadir el atributo a ambas.
2. **Orden de pases en `_transform_qweb`** — campos→expr→helper→linenum→barcode→i18n→show/style-if→loops→col-cond→strip. El pase de campo **preserva** `data-show-if`/`data-style-if` para el pase genérico posterior; si los borra, la visibilidad condicional deja de aplicar.
3. **`t-field` QWeb es ESTRICTO**: un bind inexistente/vacío tumba TODO el informe (KeyError). El resolutor hace un subíndice desnudo `field = record._fields[field_name]` (`odoo18/odoo/addons/base/models/ir_qweb.py:2451`, `IrQWeb._get_field`; en v19 `:2745`) → nombre que no existe = `KeyError` en render; y el render es **todo-o-nada** (el `except Exception` de `_render` re-lanza, `odoo19/odoo/addons/base/models/ir_qweb.py:835-870`, `IrQweb._render_iterall`), así que el cliente se queda sin documento, no con un hueco. El emisor valida con `_resolve_field(path, model)` → campo inválido se emite vacío. Nunca emitir `t-field` sin validar el path. **Y hay un modo donde `t-field` está prohibido de raíz**: el helper module-level `render(template_name, values, load, **options)` (`odoo/addons/base/models/ir_qweb.py:2720`, `render` v18 / `:2975` v19) renderiza *"without database and outside the registry"* y su docstring avisa — *"(Widget, field, or asset rendering is not implemented.)"* —; su `MockIrQWeb._get_field` lanza `NotImplementedError("Fields are not allowed in this rendering mode. Please use env['ir.qweb']._render")` (`:2762` v18 / `:3031` v19), y lo mismo para widgets (`:2765`/`:3034`) y assets (`:2768`/`:3037`). Si un test o una herramienta renderiza por esa vía, **cualquier** `t-field` del QWeb emitido explota — con un mensaje que da la solución. **Y dos renombrados de v19 en ese mismo fichero, medidos 2026-08-17**: la excepción `QWebException` (v17 `:514`, v18 `:522`) es **`QWebError`** (v19 `:534`) con **0 ocurrencias del nombre viejo** en CE19/EE19 y sin alias; y la clase del modelo `IrQWeb` (v17 `:565`, v18 `:573`) es **`IrQweb`** con w minúscula (v19 `:683`). Lo primero rompe un `except`/`import`; lo segundo solo rompe si **importas la clase** — heredar por `_inherit = 'ir.qweb'` da igual, y la prueba es que el propio v19 sigue declarando `class IrQWeb` en `bus/models/ir_qweb.py:5`, `IrQWeb` e `im_livechat/models/ir_qweb.py:5`.
4. **Cabecera/pie**: wkhtml extrae por clase `.header`/`.footer` (da igual el orden en el arch), pero **no** por posición. Mandar `false` en header/footer borra la cabecera → cae a `external_layout` de Odoo → 2 páginas. Enviar siempre el HTML real.
5. **paged.js preview**: hay que neutralizar `min-height` de página entera (`.page/#wrapwrap/main`) o el pie se empuja a una 2ª página fantasma. Odoo NO scopea CSS → nada de `:deep()`.
6. **Editor activo**: cuerpo/cabecera/pie son 3 editores `@html_editor`; `_setActiveFromTarget`/`_ae()`/`_aeSave()` deciden sobre cuál actúa insertar/clic-chip. El listener de clic-chip va en `t-on-click` de `.ird-sheet` (burbujeo), NO en `onLoad` (`editor.editable` es null en onLoad → crash).
7. **`layout_mode`** (custom/odoo/none): si vacío cae a `external_layout` de Odoo. Fijarlo explícito al generar.
8. **Loops agrupados vs normales**: runs consecutivos de `tr[data-loop]` se agrupan en UN `t-foreach`; condición de fila (`data-row-if`) y agrupación (`data-group-by`) cambian la forma del foreach. Ver `references/architecture.md`.

**Histórico OnlyOffice (no vigente)** (detalle en `references/docx-architecture.md`): 9. **Conector `onlyoffice-odoo` rama 18.0** (`master`=v19 `type="jsonrpc"` peta en v18) + **`pyjwt` horneado en la imagen** (pip no persiste al recrear el contenedor).
10. **`docxtpl 0.20` NO acepta `{%tr%}` en la misma fila** → filas for/endfor **separadas** o `{%p%}`. Y **`html4docx` aplana la rejilla** → la semilla mecánica no vale; generar estructura con `report-docx-template`.
11. **`docx.template`**: cuerpo en **`file_ids`** (`position="header"`, `lang=False`), no en `file`; FK required → borrar+recrear, no `(5,0,0)`. Editor = **conector base** `onlyoffice_editor`, NO `onlyoffice_odoo_templates` (solo formularios).

## NEVER

- Emitir `t-field`/`t-out` sin validar el path contra el modelo (`_resolve_field`).
- Ejecutar `eval` de expresiones de usuario sin pasar por `_expr_is_safe` (whitelist ast) — ver `references/security.md`.
- `docker compose restart` para recargar código — siempre down→up→`-u` [[docker-odoo-reload-cycle]].
- Diseñar maquetación con flex/grid/calc/vw-vh/webfonts — wkhtml 0.12.5 no los soporta (`references/wkhtml-limits.md`).
- Añadir un chip sin actualizar las DOS listas `strip_attributes` ni sin test.
- git commit/push sin pedirlo al usuario.

## ALWAYS

- Reproducir el bug/feature con un test del emisor antes de dar por bueno (la suite de tests es la red: `/indaws_report_designer` para QWeb, `/indaws_docx_designer` para DOCX).
- Tocar los 3 frentes de un insertable (frontend + emisor + strip) — `references/add-a-feature.md`.
- Validar el frontend OWL en navegador; el CLI no lo cubre.
- **Provenance marker (ALWAYS):** cada archivo generado (salvo `.csv`/`.json`) termina con `Made with report-designer-developer · v<version del frontmatter de esta skill>` en comentario del tipo de archivo, idempotente. Regla: `references/shared/skill-provenance-marker.md`.

## Delegates to

| Skill | Suite | Cuándo |
|-------|-------|--------|
| `odoo-owl-component-builder` | developer | Componentes OWL, integración `@html_editor`, popover, plugins del editor |
| `odoo-qweb-report-builder` | developer | QWeb generado, ir.actions.report, paperformat, pitfalls wkhtml |
| `odoo-orm-modeler` | developer | Modelos/campos `indaws.report.designer`, `indaws.report.helper` |
| `common-skills:indaws-core-auditor` | common | Convenciones suite indaws, cadena de dependencias, fixes Odoo 18 |
| `odoo-deep-knowledge` | odoo-knowledge | Gotchas de reporting y wkhtml a bajo nivel |
| `report-docx-template` | consultant | **Generar** una plantilla docxtpl limpia (python-docx) desde mockup — la vía para que la IA cree la estructura DOCX (no la semilla mecánica) |

## Consumed by

| Skill | Suite | For what |
|-------|-------|----------|
| `report-designer-author` | consultant | Skill hermana: el autor usa el editor; si topa con un límite del addon, deriva aquí |

## REFERENCES

Las 8 referencias y cuándo leerlas están en la tabla **"Referencias (leer ANTES de tocar código)"** arriba. QWeb: `architecture.md`, `add-a-feature.md`, `security.md`, `wkhtml-limits.md`, `chip-language.md`. DOCX: `superdoc.md` (editor actual), `docx-architecture.md` (OnlyOffice, histórico). Ambos: `files-map.md`.
