---
name: indaws-odoo-qweb-report-builder
version: 0.2.8
effort: medium
model: opus
description: >-
  Construye informes PDF nativos de Odoo con QWeb: ir.actions.report, paperformat, external_layout, cabeceras y pies. Usala cuando pidas un informe PDF. Incluye paperformat, saltos de página y wkhtmltopdf. NOT HTML→PDF Playwright (→ pdf-renderer); NOT DOCX (→ report-docx-template).
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion]
---

# Odoo QWeb Report Builder v0.1

> Convenciones de código y conexión a Odoo: ver `references/shared/coding-standards.md` y `references/shared/odoo-connection.md`.

Crea y corrige informes PDF nativos de Odoo (v17-v19): report action, paperformat, plantilla QWeb, cabeceras/pies y pitfalls de wkhtmltopdf. No genera el scaffolding del módulo (→ `odoo-module-creator`, que ya posee `references/qweb-reports/`, 14 archivos).

## Referencias (leer ANTES de generar código)

| Ref | Cuándo |
|-----|--------|
| `references/report-structure.md` | Siempre — report action, paperformat, declaración en manifest |
| `references/qweb-patterns.md` | Siempre — layouts, loops, t-out vs t-esc, totales, widgets de campo |
| `references/wkhtmltopdf-gotchas.md` | Siempre que haya problemas de renderizado o saltos de página |
| `odoo-version-api-reference` (subagent) | Cambios de API por versión (t-esc→t-out en v17, aggregator en v18, etc.) |

El contenido de bajo nivel (xpath para heredar reportes, migración entre versiones,
ejemplos XML de report-anatomy) vive en `odoo-module-creator/references/qweb-reports/`.
Leer esos archivos directamente cuando se necesite detalle; NO duplicarlos aquí.

## Workflow

### PASO 1 — Recopilar contexto [STOP]

Preguntar con `AskUserQuestion`:
- Versión de Odoo (v17 / v18 / v19)
- Modelo origen del informe (`sale.order`, `account.move`, etc.)
- ¿Módulo ya existe o hay que crearlo?
- Tipo de layout: `web.external_layout` (con cabecera empresa) o `web.basic_layout` (sin cabecera)
- Formato papel necesario (A4 portrait/landscape, personalizado)
- ¿Incluye códigos de barras, imágenes, multipágina?

Si el módulo NO existe → delegar scaffolding a `odoo-module-creator` antes de continuar.

### PASO 2 — Cargar referencia de versión [subagent SONNET]

```
Invocar skill 'odoo-version-api-reference' para versión {version}.
Devolver: cambios en QWeb (t-esc vs t-out), aggregator vs group_operator,
_get_report_values, y gotchas de reports en esta versión.
```

Guardar resultado como `API_RESULT`. No escribir código sin él.

### PASO 3 — Definir report action + paperformat [OPUS]

Usando `references/report-structure.md` + `API_RESULT`:
1. Crear `ir.actions.report` con `report_type="qweb-pdf"`, binding_model_id, binding_type
2. Crear `report.paperformat` si el formato estándar no aplica
3. Añadir ambos a `__manifest__.py` en `data/`
4. Verificar que el `name` del report coincide con el `<template id>` del QWeb

### PASO 4 — Construir plantilla QWeb [OPUS]

Usando `references/qweb-patterns.md`:
1. Estructura: `t-call="web.external_layout"` (o `web.basic_layout`) → `t-set="o" t-value="doc"` → cuerpo
2. Loops con `t-foreach` + `t-as`; condicionales con `t-if`/`t-elif`/`t-else`
3. Salida de valores: `t-out` (existe desde v16). El escapado lo decide el valor (`Markup` vs `str`), no la directiva — ver GOTCHA 1.
4. Campos monetarios: `t-options='{"widget": "monetary", "display_currency": o.currency_id}'`
5. Fechas: `t-options='{"widget": "date"}'`
6. Subtotales: `<t t-set="total" t-value="sum(line.price_subtotal for line in o.order_line)"/>`
7. Dirección personalizada: pasar `t-set="address"` con tu bloque al `web.external_layout` (Odoo lo renderiza vía `web.address_layout`). NO existen slots `address_header`/`address_footer`

### PASO 5 — Manejar wkhtmltopdf [OPUS]

Ver `references/wkhtmltopdf-gotchas.md` para:
- Saltos de página forzados vs automáticos
- CSS soportado vs no soportado
- Imágenes (base64 vs URL), fuentes web
- Cabeceras/pies que desaparecen en página 2+
- `--disable-smart-shrinking` y cuándo activarlo

### PASO 6 — Registrar en manifest y probar

```bash
# Instalar/actualizar el módulo
docker exec -it <container> odoo-bin -u <module> --stop-after-init 2>&1 | tail -30

# Verificar el report action e imprimir un registro de prueba desde la UI
```

## GOTCHAS

1. **`t-esc` y `t-out` hacen LO MISMO — MEDIDO (v18/v19).** Lo que decide si se escapa no es la directiva sino el **valor**: `Markup` se pinta, `str` se escapa — y quien lo marca es el **tipo de campo** (`Html` vs `Char`). Cambiar el tipo cambia lo impreso **sin tocar la plantilla**. Matriz medida en `references/qweb-patterns.md`. Disponibilidad: `t-out` existe desde **v16** (0 ocurrencias en v14/v15).
2. **`report_name` debe coincidir con `template id`** — MEDIDO (v18 y v19): si no coincide **NO sale un PDF en blanco, revienta**, y la excepcion trae el nombre buscado. Es una buena noticia y cambia donde buscar: se lee el error, no se depura el CSS.
3. **Cabeceras/pies desaparecen en páginas 2+** — wkhtmltopdf requiere que header/footer sean archivos HTML externos o que se usen los slots de `web.external_layout`. Ver `references/wkhtmltopdf-gotchas.md` §3.
4. **Imágenes con URL relativa no cargan** — wkhtmltopdf no resuelve rutas relativas de Odoo. Usar URLs absolutas o base64 inline. Ver `references/wkhtmltopdf-gotchas.md` §4.
5. **`_get_report_values(self, docids, data=None)`** — Es el método real del AbstractModel de report en v17/v18/v19 (`_get_report_data` NO existe; usarlo rompe el report). `API_RESULT` del PASO 2 confirma la firma.
6. **`web.external_layout` añade cabecera de empresa** — Si el cliente no tiene logo/dirección configurada, el header sale vacío. Usar `web.basic_layout` si se quiere control total.
7. **Saltos de página con `page-break-before: always`** — Soportado. `page-break-inside: avoid` NO siempre funciona en wkhtmltopdf <= 0.12.6. Ver `references/wkhtmltopdf-gotchas.md` §2.
8. **Módulo sin `report_type="qweb-pdf"`** — Omitirlo hace que Odoo trate el report como HTML, no genera PDF.
9. **`binding_model_id` vs `model`** — `model` es el modelo que aporta los datos; `binding_model_id` es el modelo al que se "engancha" el botón de impresión (normalmente el mismo).
10. **Moneda antes que campo `monetary`** — El campo `currency_id` debe estar antes en el XML que el campo monetario que lo referencia, o el widget falla silenciosamente.
11. **Render todo-o-nada (`QWebException` en v17/v18, `QWebError` en v19)** — un error runtime en UN solo `t-out`/`t-field`/`t-esc` (p.ej. `1/0`, un método sobre un recordset vacío, un `t-att` que peta) tumba el **PDF ENTERO**, no solo ese nodo. **La clase se renombró en v19 y no hay alias**: `class QWebException(Exception)` (v17 `odoo/addons/base/models/ir_qweb.py:514`, `keep_query`, v18 `:522`) pasa a `class QWebError(Exception)` (v19 `:534`), con **0 ocurrencias de `QWebException` en CE19 y EE19** — un `except QWebException` o su `import` **rompe al migrar a 19** (medido 2026-08-17). Mecanismo, y **la forma del código NO es la misma en las dos** aunque la garantía sí: en **v19** un único `except Exception` de `_render` re-lanza siempre, o como `QWebError(...) from error` o con un `raise` desnudo (`odoo/addons/base/models/ir_qweb.py:835-870`, `IrQWeb._get_template`); en **v18** la garantía la dan dos pares `except QWebException: raise` / `except Exception as e: raise QWebException(...) from e`, uno en el código generado (`:784-792`) y otro al compilar (`:794-798`). **En ninguna de las dos hay rama de degradación por nodo.** (Ojo: la cita `:834-870` es de v19; en v18 esas líneas son otra cosa —el resolutor de `t-call`—, así que no vale para comprobarlo allí.) Caso concreto de `t-field`: el resolutor hace un subíndice desnudo `field = record._fields[field_name]` (`odoo18/odoo/addons/base/models/ir_qweb.py:2451`, `IrQWeb._get_field`; en v19 `:2745`), asi que un nombre de campo que no existe es `KeyError` en render → **sin documento**, no un hueco. La validación estática del arch NO protege del fallo en render. Si emites expresiones dinámicas de usuario, enrútalas por un método envoltorio `try/except → ''` (ver `references/qweb-patterns.md` §expresiones-blindadas); nunca metas expr sin control directamente en el template.
12. **Imágenes en wkhtml: `image_data_uri()`, no `/web/image/`** — `t-field widget="image"` y las URLs `/web/image/…` **no resuelven** de forma fiable en wkhtmltopdf (imagen rota en el PDF). Verificado v18: `image_data_uri` está expuesto en el contexto QWeb (`odoo/addons/base/models/ir_qweb.py:956`, `IrQWeb._prepare_environment`). Usar `<img t-att-src="image_data_uri(record.campo_binary)"/>` con un `t-if` de guarda (si el binario es falsy, `image_data_uri` peta → ver gotcha 11). Helper real: `odoo/tools/image.py:image_data_uri`.
13. **`context_timestamp` es de nivel REPORT, no QWeb genérico** — `context_timestamp(datetime.datetime.now()).strftime('%Y-%m-%d %H:%M')` (fecha de impresión con tz del usuario) funciona en plantillas de **informe** porque `ir.actions.report` lo inyecta en los valores de render (`odoo/addons/base/models/ir_actions_report.py:768`, `IrActionsReport._render_template`, verificado v18 y v19). **NO** está en los globals de `ir.qweb` (una vista QWeb normal no lo tiene). `datetime` sí es global de QWeb (`odoo/addons/base/models/ir_qweb.py:954`, `IrQWeb._prepare_environment`). Patrón real en `web/views/report_templates.xml:842`, `internal_layout`.
14. **`t-out="0"` es el SLOT del `t-call`, no el número 0** — dentro de una plantilla llamada con `t-call`, `<t t-out="0"/>` inserta el **cuerpo** pasado por el llamante. Si parseas/reescribes arch programáticamente, NO trates `"0"` como una expresión numérica ahí.
15. **Mover/borrar un bloque: fallback + tests primero** — antes de reubicar o eliminar un bloque de un report: (1) `grep` los tests de render del módulo (`_render_qweb_html`, `assertIn(...name/vat...)`) — un bloque que un test daba por presente queda en ROJO al borrarlo, y no lo ves hasta correr el test; (2) preserva TODAS las ramas de fallback (p.ej. marca comercial→compañía). Quitar el fallback borra datos legalmente exigibles (razón social/CIF) en la rama "sin dato" **sin ningún error visible** — el PDF sale, solo que incompleto.
16. **El worker vivo cachea el QWeb compilado** — actualizar el arch de una plantilla (por `-u` desde otro proceso, o editando por UI) NO invalida el QWeb ya compilado en el worker que sirve las peticiones: hay que reiniciar el proceso que atiende para ver el cambio. Verifica el render REAL antes de commitear: `/report/html/<report_name>/<id>` para iterar rápido, pero fíate del `/report/pdf/<report_name>/<id>` (wkhtmltopdf) para juzgar layout — el HTML en Chrome difiere del PDF final.

## NEVER

- Duplicar el contenido de `odoo-module-creator/references/qweb-reports/` — apuntar a esos archivos
- Usar `t-raw` (deprecado, lanza warning; sigue funcionando en v18) — usar `t-out`
- Escribir `<template>` QWeb sin `inherit_id` o `id` único global (`module.template_name`)
- Usar rutas de imagen relativas en el XML del report

## ALWAYS

- Consultar `odoo-version-api-reference` antes de escribir código (v17/v18/v19 difieren)
- Verificar wkhtmltopdf instalado: `wkhtmltopdf --version` (mínimo 0.12.5)
- Declarar el report en `__manifest__.py` sección `data` (NO `demo`); nombrar la plantilla `<module>.<report_tech_name>` (unicidad global)
- **Provenance marker (ALWAYS):** every generated file (except `.csv`/`.json` — no comment syntax) ends with `Made with odoo-qweb-report-builder · v<version del frontmatter de esta skill>`, wrapped in the file-type comment, idempotent. Full rule: `references/shared/skill-provenance-marker.md`.

## Delegates to

| Skill | Suite | Cuándo |
|-------|-------|--------|
| `odoo-module-creator` | developer | Scaffolding del módulo completo; también posee `references/qweb-reports/` (14 archivos con detalle de anatomy, xpath, migrate, layout) |
| `odoo-version-api-reference` | common | Cambios de API QWeb por versión |
| `pdf-renderer` | common | PDF desde HTML externo a Odoo (Playwright), NO para reports nativos |
| `report-docx-template` | consultant | Reports en formato DOCX via indaws_report_docx |
| `odoo-deep-knowledge` | odoo-knowledge | Gotchas de reporting (paperformat, repetición de `<thead>`, límites wkhtmltopdf) |

## Consumed by

| Skill | Suite | For what |
|-------|-------|----------|
| odoo-module-creator | developer | Build non-trivial PDF reports (QWeb/wkhtmltopdf) for a new module |
| report-designer-developer | developer | QWeb generado por el emisor del report designer, pitfalls wkhtml |

## REFERENCES

Las 3 referencias y cuándo leerlas están en la tabla **"Referencias (leer ANTES de generar código)"** arriba: `report-structure.md`, `qweb-patterns.md`, `wkhtmltopdf-gotchas.md` (detalle profundo en `odoo-module-creator/references/qweb-reports/`, ver Delegates to).
