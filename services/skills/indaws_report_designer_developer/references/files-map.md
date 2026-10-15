# Mapa de archivos — `indaws_report_designer`

Raíz: `$PROJECTS_ROOT/indawsodoo/18/indaws_report_designer/`

```
__manifest__.py                              depends: web, indaws_base, indaws_report_docx
models/
  report_designer.py        (~450 loc)       Modelo indaws.report.designer + API del editor
  report_emitter.py         (~1200 loc)      _transform_qweb / _transform_docx, emit, charts, cond, helpers
  report_helper.py                           Modelo indaws.report.helper (funciones con nombre, safe_eval)
static/src/report_designer/
  report_designer.js        (~1400 loc)      Componente OWL: editores @html_editor, chips, inspector, menús, powerbox, preview
  report_designer.xml                        Template OWL (barra, hoja, paneles de modo del inspector)
  report_designer.scss                       Estilos (papel A4, barra, .ird-sheet, landscape)
static/lib/pagedjs/paged.js                  paged.js v0.4.3 vendorizado (window.Paged) para preview paginada
views/report_designer_views.xml             Vistas backend + client action + menús
security/ir.model.access.csv                 ACLs (designer + helper; gestión de helpers = group_system)
tests/test_report_designer.py                tests del emisor (QWeb: chips, herencia/ops, carga fiel, RAW)
```

## Módulo DOCX — `indaws_docx_designer` (OnlyOffice)

Raíz: `$PROJECTS_ROOT/indawsodoo/18/indaws_docx_designer/` · `depends = ["base","web","indaws_report_docx","indaws_report_designer"]` (verificado en el manifest 2026-08-07; `onlyoffice_odoo` **fuera** tras el pivot a SuperDoc). Detalle: `references/docx-architecture.md`.

```
models/indaws_docx_designer.py               Modelo indaws.docx.designer + acciones
  action_open_editor()                       -> client action onlyoffice_editor {attachment_id}
  get_docx_fields(path)                       -> árbol de campos con tags docxtpl (panel F3)
  action_seed_from_qweb() / _docx_make_docxtpl_safe()   semilla QWeb->docx (lossy) + saneo docxtpl 0.20
  action_ai_generate() / _ai_prompt / _apply_html_to_docx   IA (LLM configurable)
  action_generate()                          docx.template (file_ids!) + ir.actions.report + binding
  action_print_test() / unlink()             render de muestra / limpieza
static/src/docx_fields/                      widget OWL ird_docx_fields (copia tag al portapapeles)
views/indaws_docx_designer_views.xml         form (botones editar/sembrar/IA/sincronizar/imprimir) + menú
security/security.xml + ir.model.access.csv  grupo group_indaws_docx_designer
tests/test_docx_designer.py                  tests (modelo, campos, generate/render, semilla docxtpl-safe)
```
**PIVOT — editor actual = SuperDoc** (navegador, sin server): `static/src/superdoc_editor/` (`superdoc_editor.js/.xml/.scss`, `table_builder.js/.xml`, `field_wizard.js/.xml`) + lib vendorizada `static/lib/superdoc/superdoc.min.js` (~5.9 MB, lazy-load, NO en assets_backend). Acción `action_open_superdoc()` → client action `indaws_docx_designer.superdoc`. Detalle y gotchas del bundle: `references/superdoc.md`. El conector **`onlyoffice_odoo`** (rama 18.0) + Document Server queda como histórico en retirada. Render (ambos) = `indaws_report_docx` (docxtpl).

## Dónde vive cada cosa

| Necesito tocar… | Archivo | Símbolo |
|-----------------|---------|---------|
| Traducir un chip a QWeb | `report_emitter.py` | pase en `_transform_qweb` |
| Editor DOCX / campos / IA / render docx | `indaws_docx_designer/` | ver bloque DOCX arriba + `docx-architecture.md` |
| Traducir un chip a DOCX (QWeb→docx, semilla) | `report_emitter.py` | pase en `_transform_docx` |
| Limpiar `data-*` del PDF final | `report_emitter.py` | las DOS listas `strip_attributes` |
| Validar path de campo | `report_emitter.py` | `_resolve_field(path, model)` |
| Validar expresión de usuario | `report_emitter.py` | `_expr_is_safe` |
| Loops / agrupación / condición de fila | `report_emitter.py` | pase de loops, `report_groups`, `_col_cond_expr` |
| Gráficos | `report_emitter.py` | `_chart_png` (QWeb), `report_chart_image` + `_sync_chart_variables` (DOCX) |
| Dims de papel / orientación | `report_emitter.py` / `report_designer.py` | `_paper_dims`, `_report_paperformat` |
| Insertar un elemento / menús / powerbox | `report_designer.js` | `insert*()`, `IrdContextMenu`, `IrdCommandsPlugin` |
| Clic-chip / inspector | `report_designer.js` | handler en `.ird-sheet` (`t-on-click`), `selectChip`, `loadFieldInfo` |
| Editor activo cuerpo/cabecera/pie | `report_designer.js` | `_setActiveFromTarget`, `_ae()`, `_aeSave()` |
| Preview paged.js | `report_designer.js` | `preview()`, `_paginate` |
| Funciones con nombre | `report_helper.py` | `indaws.report.helper`, `report_helper`, `list_helpers`, `test_helper` |
| Campos del modelo designer | `report_designer.py` | `body_html`/`header_html`/`footer_html`, `layout_mode`, `qweb_override`, `history_json`, `paperformat_id`, `orientation`, `is_template` |
| Import QWeb / DOCX | `report_designer.py` / `report_emitter.py` | `import_qweb`, `_qweb_to_html`; `import_docx`, `_docx_to_html` |
| Paneles del inspector (UI) | `report_designer.xml` + `.scss` | — |

Made with report-designer-developer · v0.2.0
