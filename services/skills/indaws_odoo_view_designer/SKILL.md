---
name: indaws-odoo-view-designer
version: 0.3.18
effort: high
model: opus
description: >-
  Diseña, extiende y depura vistas XML de Odoo (v17-v19): form, list, search/
  searchpanel, kanban, pivot, graph, calendar, activity, gantt, grid, map,
  cohort, hierarchy. Cubre herencia xpath, visibilidad por expresión, catálogo
  de widgets y diffs v17→v18→v19.
  Triggers: "vista form / list / kanban", "tree a list", "searchpanel",
  "heredar vista / xpath", "qué widget uso", "diseñar vista Odoo", "gantt view",
  "migrar vista v17 v18 v19", "campo invisible/readonly por expresión",
  "decoration list", "view inheritance".
allowed-tools: [Read, Write, Edit, Glob, Grep, Bash, AskUserQuestion]
---

# Odoo View Designer

> Convenciones y conexión: ver `references/shared/coding-standards.md`.

Diseña/extiende/depura vistas XML de Odoo (v17-v19). Cada reference trae el set
**completo y verificado contra fuente** de un tipo de vista, con diffs por versión.
No hace el scaffolding del módulo (eso es `odoo-module-creator`): es el
especialista profundo del *arch* de vistas.

## Workflow

1. **Confirmar versión y tipo.** Pregunta/deduce la versión objetivo (v17/v18/v19)
   y el tipo de vista. Los diffs por versión son grandes — ver
   `references/common-and-version-diffs.md`.
2. **Cargar la reference del tipo.** Abre solo la que aplique (tabla REFERENCES) y
   usa su catálogo de atributos/sub-elementos/widgets.
3. **Diseñar o extender.** Para crear: escribe el `<arch>` con los atributos
   correctos. Para extender una vista existente: usa herencia `xpath`/`position`
   (sección en `common-and-version-diffs.md`), NO reescribas la vista entera.
4. **Visibilidad y widgets.** Usa `invisible/readonly/required="expr python"`
   (NO `attrs=`/`states=`, eliminados en v17). Elige widget del catálogo de
   `common-and-version-diffs.md`.
5. **Validar versión.** Comprueba los diffs v18→v19 antes de entregar (ej. `card`
   en kanban v19, `<list>` en vez de `<tree>`). Firmas finas: [delega] en
   `odoo-version-api-reference`.
6. **¿Crear un TIPO DE VISTA nuevo?** (no configurar uno existente) → `references/custom-view-type.md`
   (Python `selection_add` + `_get_view_info`; JS registry; **gotcha caché IndexedDB `rpc`**). El
   componente OWL del renderer se construye con `odoo-owl-component-builder`.
7. **¿Cambiar cómo se COMPORTA una vista existente?** (no su `arch`, sino qué hace al pintar,
   editar o guardar) → eso ya no es esta skill: es la capa JS de vistas, en
   la referencia `view-layer.md` de `odoo-owl-component-builder` — descriptor de cinco piezas, `js_class`,
   API del `Record`. Aquí se decide **qué** se ve; allí, **cómo** se comporta.

## GOTCHAS (verificadas en fuente v18/v19)

1. **`<tree>` ya NO existe** ni en v18 ni en v19 — usa **`<list>`**. El `Selection`
   de `ir.ui.view.type` abre con `('list', 'List')` (`odoo/addons/base/models/ir_ui_view.py:153`, `class View` v18,
   `:149` v19) y en v17 abría con `('tree', 'Tree')` (`:163`). Cero ficheros XML
   con `<tree` en los addons oficiales de v18 y v19 — el corte es duro en los dos
   sentidos: `<list>` tampoco vale en v17.
2. **Kanban v19: `kanban-box` eliminado.** Obligatorio `<t t-name="card">`; si
   falta lanza *"Missing 'card' template"* (`web/static/src/views/kanban/kanban_arch_parser.js:141`, `KanbanArchParser.parse` v19). En
   v18 coexisten: el legacy sigue soportado con `console.warn` (`:153`) y la falta
   de `card` solo revienta si tampoco hay `kanban-box` (`:158`). En v17 la única
   forma válida es `kanban-box` y su error es el simétrico, *"Missing
   'kanban-box' template"* (`:150`). Ver `references/kanban.md`.
3. **`attrs=` y `states=` eliminados en v17.** Usa expresiones directas:
   `invisible="state == 'done'"`, `readonly="..."`, `required="..."`,
   `column_invisible="..."` (list). No es que se ignoren: el arch **no carga**, con
   `ValidationError('Since 17.0, the "attrs" and "states" attributes are no longer
   used…')` (`odoo/addons/base/models/ir_ui_view.py:393`, `View._check_xml` v17, `:387` v18, `:495` v19).
4. **v19: `category_id` → `privilege_id`** en grupos (afecta `groups=` y seguridad).
4b. **Nada de la vista es un permiso — medido en v18 y v19.** `create="0"`, `edit="0"`,
   `delete="0"` y `readonly="1"` los cumple el **navegador**: el servidor los manda en el arch y no
   comprueba ninguno al escribir. `create`/`write`/`unlink` por RPC funcionan igual, y `web_save`
   —el camino real del cliente al guardar, que es un `write` pelado (`web/models/models.py:68-74`, `Base.web_save`)—
   escribe hasta un campo marcado `readonly` en la vista. Igual que `groups=`. La barrera son las
   ACL y las record rules → `odoo-security-designer`. Detalle: `references/form.md`.
5. **`hierarchy` es Community** (`web_hierarchy`, v17+), NO Enterprise. `dashboard`
   no es un view type con arch (es `board` CE o spreadsheet EE).
6. **Widget que no existe** — el catálogo válido es la carpeta real
   `web/static/src/views/fields/`; no inventes nombres de widget. Para **CREAR** un widget
   nuevo (componente OWL con `{component, supportedTypes, extractProps}`) → `odoo-owl-component-builder`.
   Esta skill USA widgets del catálogo; no construye el componente JS.
7. **`ir.ui.view` valida la herencia al `create`/`write`** — un `xpath` que no ancla lanza excepción
   **al guardar la vista**, no al renderizar. Si insertas vistas por script y quieres que un xpath
   fallido no aborte todo el lote, envuelve **cada** vista en `with self.env.cr.savepoint():` (revierte
   SOLO esa vista, no toda la transacción). NUNCA `cr.rollback()` a pelo. Verificado `ir_ui_view.py`.
8. **Ancla xpath por `hasclass()`/`name=`, no por posición** — `apply_inheritance_specs` soporta la
   función custom `hasclass('page')` (registrada en `odoo/addons/base/models/ir_ui_view.py:136-137`, `xpath_utils`, v18/v19) →
   `//div[hasclass('page')]` es mucho más robusto que `//div[3]` frente a cambios del core. Odoo combina
   todas las extensiones con `get_combined_arch()` (`odoo/addons/base/models/ir_ui_view.py:933`, `View.get_combined_arch`); ver `references/common-and-version-diffs.md`.
9. **View type nuevo que no aparece en el switcher, SIN error** — no es tu código: es la caché
   **IndexedDB `"rpc"`** del webclient (`web/static/src/start.js:36`, `startWebClient`, `core/network/rpc_cache.js`) sirviendo el
   action/arch viejo. Limpia storage+IndexedDB y recarga 2 veces. Detalle y patrón completo de
   registro: `references/custom-view-type.md`.

10. **Un `readonly` condicional PIERDE datos, y lo `invisible` no protege.** Medido con tour en
    v18 el 2026-08-15: con `readonly="state != 'draft'"`, si el usuario rellena el campo y en la
    misma sesión cambia `state` **sin guardar**, el valor no llega al servidor y desaparece sin
    aviso — el cliente evalúa el `readonly` al guardar, no al escribir
    (`web/static/src/model/relational_model/record.js:635`, `Record._getChanges`). Salidas: guardar antes de que cambie la
    condición, o `force_save="1"`. Y al revés: un campo `invisible="1"` **sí** se escribe y **sí**
    se guarda, así que ocultarlo no es protegerlo. Medición completa en
    la referencia `view-layer.md` de la skill `odoo-owl-component-builder`.

## NEVER
- NUNCA uses `<tree>`, `attrs=` ni `states=` en v17+.
- NUNCA reescribas una vista entera para un cambio pequeño → hereda con `xpath`.
- NUNCA inventes atributos o widgets; si dudas, verifica en fuente / api-reference.
- NUNCA asumas que un arch de v18 vale igual en v19 sin mirar los diffs.

## ALWAYS
- SIEMPRE confirma la versión antes de elegir sintaxis.
- SIEMPRE usa `<list>` y expresiones de visibilidad python.
- SIEMPRE elige el widget del catálogo verificado.
- SIEMPRE prefiere herencia (`xpath`/`position`) para modificar vistas existentes.
- **Provenance marker (ALWAYS):** every generated file (except `.csv`/`.json` — no comment syntax) ends with `Made with odoo-view-designer · v<version del frontmatter de esta skill>`, wrapped in the file-type comment, idempotent. Full rule: `references/shared/skill-provenance-marker.md`.

## Delegates to
- **odoo-module-creator** — scaffolding del módulo y registro de la vista/acción.
- **odoo-version-api-reference** — firmas finas y diffs exhaustivos por versión.
- **odoo-module-scanner** — inventariar las vistas existentes antes de extender.

## Consumed by
- **odoo-module-creator** — para diseño profundo de vistas más allá del scaffold base.

## REFERENCES

| Fichero | Tipo de vista | Cargar cuando |
|---------|---------------|---------------|
| [references/form.md](references/form.md) | form + settings (`res.config.settings`) | Vista de detalle / pantallas de ajustes |
| [references/list.md](references/list.md) | list (antes tree) | Listados, editable, decoraciones, totales |
| [references/search.md](references/search.md) | search + searchpanel | Filtros, agrupaciones, panel lateral |
| [references/kanban.md](references/kanban.md) | kanban | Tarjetas, `card` v19, progressbar, drag&drop |
| [references/pivot-graph.md](references/pivot-graph.md) | pivot + graph | Análisis y gráficas |
| [references/calendar-activity.md](references/calendar-activity.md) | calendar + activity | Vistas de tiempo / actividades |
| [references/enterprise-views.md](references/enterprise-views.md) | gantt, grid, map, cohort, hierarchy | Vistas Enterprise (+ hierarchy CE) |
| [references/common-and-version-diffs.md](references/common-and-version-diffs.md) | TRANSVERSAL | Herencia, visibilidad por expr, catálogo de widgets, diffs v17→v18→v19 |
| [references/custom-view-type.md](references/custom-view-type.md) | CREAR un view type nuevo | Registro Python (`selection_add`+`_get_view_info`) + JS registry + gotcha caché IndexedDB `rpc` |
