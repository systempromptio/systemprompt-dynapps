---
name: indaws-odoo-website-snippets
version: 0.1.14
effort: high
model: opus
description: >-
  Desarrolla snippets del website builder de Odoo (v17-v19): plantillas QWeb s_*,
  registro en snippets.xml, panel de opciones (SnippetOptionWidget/we-*, v19:
  BaseOptionConfig/Plugin), snippets dinámicos y drag-drop. NOT JS público en
  producción (→ odoo-website-frontend-js).
  Triggers: "crear un snippet", "panel de opciones de un snippet", "snippet no
  aparece en el menú", "registrar snippet en snippets.xml", "snippet dinámico",
  "build a website snippet", "SnippetOption", "dynamic snippet".
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion, EnterPlanMode, Agent]
---

# Odoo Website Snippets

Diseña e implementa snippets (bloques arrastrables) del website builder en módulos
Odoo v17-v19. Un snippet completo son tres piezas: (1) la **plantilla QWeb** `s_*`
con el markup del bloque, (2) su **registro** en el menú de snippets (`snippets.xml`)
con grupo, thumbnail y keywords, y (3) opcionalmente un **panel de opciones**
(snippet options) que el usuario ve al seleccionar el bloque. Esta skill cubre las
tres y los assets que las cargan. El contenido visual interno → `odoo-frontend`; la
interactividad en el sitio publicado → `odoo-website-frontend-js`.

Refs compartidas: `references/shared/coding-standards.md`, `references/shared/odoo-connection.md`

## Workflow

1. **Definir el snippet** [STOP — AskUserQuestion]
   - ¿Bloque estructural (`snippet_structure`, una `<section>`) o contenido interno (`snippet_content`)?
   - ¿Grupo del menú (`intro`/`columns`/`content`/`images`/`people`/`text`/...) o `t-thumbnail` directo?
   - ¿Necesita panel de opciones? ¿Qué controla (clases, atributos, estilos)?
   - ¿Es dinámico (lista registros vía filtro/template)?
   - Versión objetivo: **v17/v18 (editor legacy + `options.registry`)** vs **v19 (`html_builder`, Plugin + `BaseOptionComponent`)**.

2. **Analizar el módulo destino** [delega]
   - **[SONNET SUBAGENT]** `Agent(model: sonnet)` → `odoo-module-scanner`: snippets existentes, dependencia `website` en `__manifest__.py`, bundles de assets ya declarados.

3. **Proponer arquitectura** [STOP — confirmación]
   - `views/snippets/s_<nombre>.xml` (plantilla + options + `<asset>`), entrada en herencia de `website.snippets` y `website.snippet_options`, fichero JS de option si aplica.

4. **Plantilla + registro**
   - Leer [`references/snippet-anatomy.md`](references/snippet-anatomy.md): estructura de `s_*.xml`, registro `t-snippet`/`group`/`t-thumbnail`/`keywords`, `data-snippet`, thumbnails SVG, declaración de assets.

5. **Panel de opciones** (si aplica)
   - Leer [`references/snippet-options.md`](references/snippet-options.md): `inherit_id="website.snippet_options"`, `data-selector`/`data-exclude`/`data-target`/`data-js`, widgets `we-*`, métodos `selectClass`/`selectStyle`/`selectDataAttribute`, `onBuilt`/`cleanForSave`, `options.registry` (v17/v18) vs Plugin `html_builder` (v19).

6. **Snippet dinámico** (si aplica)
   - Leer [`references/dynamic-snippets.md`](references/dynamic-snippets.md): `s_dynamic_snippet_template`, `filterId`/`templateKey`/`numberOfRecords`, controller `/website/snippet/...`, public widget de render.

7. **Robustez y revisión**
   - Leer [`references/gotchas.md`](references/gotchas.md) antes de cerrar: option que no aparece, selector mal, sanitización que borra atributos, asset en bundle equivocado, snippet sin `data-snippet`.

8. **Tests** [delega]
   - **[SONNET SUBAGENT]** `Agent(model: sonnet)` → `odoo-test-generator`: tour `HttpCase` que arrastra el snippet y prueba sus opciones.

## GOTCHAS

1. **El option no aparece** — El `<div data-js=... data-selector=...>` debe ir dentro de una plantilla con `inherit_id="website.snippet_options"` (en `website`) y casarse con un `optionName` registrado en `options.registry`; si no hay JS, usa `options.Class` por defecto. El mapeo `data-js → registry` está en `web_editor/static/src/js/editor/snippets.editor.js:829`, `_initializeOptions` (`new (options.registry[optionName] || options.Class)(...)`).
2. **`data-selector` mal escrito** — Si el selector no casa con ningún nodo del DOM del snippet, el panel entero no se monta y no hay error visible. El selector se aplica sobre el `$target`; comprueba que la clase raíz del snippet (`.s_<nombre>`) coincide.
3. **Snippet no listado en el menú** — No basta con definir la plantilla `s_*`: hay que añadir `<t t-snippet="modulo.s_x" .../>` dentro de `website.snippets` (xpath a `snippet_structure` o `snippet_content`). El menú se computa leyendo esos `<snippets>` en `web_editor/static/src/js/editor/snippets.editor.js:3254`, `_computeSnippetTemplates`, `_computeSnippetTemplates` (la lectura concreta, `:3342`).
4. **`data-snippet` lo pone el editor, no tú** — Al soltar el bloque, el editor toma `name` de `snippetEl.children[0].dataset.snippet` (`web_editor/static/src/js/editor/snippets.editor.js:3352`, `_computeSnippetTemplates`). El nombre del snippet sale del `id` de la `<template>`; no hardcodees `data-snippet` en el markup salvo en snippets dinámicos donde se setea con `t-att-data-snippet` (`website/views/snippets/s_dynamic_snippet.xml:4`, `website.s_dynamic_snippet_template`).
5. **Asset en el bundle equivocado** — El JS de options va a `website.assets_wysiwyg` (solo en editor), NO a `web.assets_frontend`. El SCSS visible en el sitio publicado va a `web.assets_frontend`. Ver `website/__manifest__.py:193,310` y declaración `<asset>` en `website/views/snippets/s_alert.xml:42-45`, `website.s_alert_000_scss`.
6. **Sanitización borra atributos/clases custom** — El editor sanea el HTML al guardar; atributos no estándar en zonas editables pueden perderse. Marca lo no editable con `o_not_editable`/`oe_unremovable`/`oe_unmovable` (ver `website/views/snippets/s_dynamic_snippet.xml:7`, `website.s_dynamic_snippet_template`, y `website/views/snippets/s_dynamic_snippet.xml:16`) y usa `data-*` reconocidos por el option.
7. **`onBuilt` vs `start`** — La lógica que debe correr **al soltar** el snippet (insertar un iframe, inicializar estado) va en `onBuilt()`, no en `start()`. Ejemplo real `website/static/src/snippets/s_map/options.js:11-21`, `onBuilt`; firma en `web_editor/static/src/js/editor/snippets.options.js:3445`, `onBuilt`.
8. **`selectClass` exclusivo dentro del mismo widget** — Varias `we-button` con `data-select-class` en un mismo `we-select`/`we-button-group` son mutuamente excluyentes (toggle de clase). Para acumular clases independientes usa widgets separados. Patrón en `website/views/snippets/s_alert.xml:29-33`, `s_alert_options`.
9. **`data-no-preview="true"`** — Sin él, mover el ratón sobre una opción aplica un preview en vivo que puede romper snippets dinámicos (re-fetch). Los dynamic options lo ponen siempre (`website/views/snippets/s_dynamic_snippet.xml:36-46`, `website.s_dynamic_snippet_options_template`).
10. **v19 reescribió el editor** — En v19 NO existe `options.registry`/`we-select`. Las opciones son componentes OWL `extends BaseOptionComponent` con `static selector`, registrados vía un `Plugin` de `html_builder` (`resources.builder_options`, `registry.category("builder-plugins")`). Ver `website/static/src/builder/plugins/options/process_steps_option.js:10-12`, `ProcessStepsOption`, v19 y `odoo19/addons/website/static/src/builder/plugins/options/process_steps_option_plugin.js:21-26`, `class ProcessStepsOptionPlugin`. No mezcles ambos sistemas.
11. **`data-js` sin clase JS** — Si pones `data-js="Foo"` pero `options.registry.Foo` no existe, cae al `options.Class` genérico y los métodos custom (`selectFoo`) nunca se llaman. Define la clase o quita el `data-js`.
12. **Thumbnail roto** — `t-thumbnail` debe apuntar a un asset servible (`/modulo/static/.../x.svg`). Si falta, el bloque aparece sin imagen en el menú. El src se lee en `web_editor/static/src/js/editor/snippets.editor.js:3356`, `_computeSnippetTemplates` (`dataset.oeThumbnail`).

## NEVER

- Definir la plantilla `s_*` sin registrarla en `website.snippets` (no aparecerá en el menú)
- Poner el JS de snippet options en `web.assets_frontend` (va a `website.assets_wysiwyg`)
- Hardcodear `data-snippet` en el markup de un snippet estático (lo gestiona el editor)
- Mezclar `options.registry`/`we-*` (v17/v18) con `BaseOptionComponent`/Plugin (v19)
- Meter la interactividad pública del snippet (clicks, sliders en el sitio) en el option JS → va en `publicWidget`/`Interaction`
- Confiar en que el editor preserve atributos custom en zonas editables sin `o_not_editable`
- Usar `start()` para lógica que debe correr al soltar el bloque → usa `onBuilt()`

## ALWAYS

- Leer la reference de cada fase antes de generar código
- Confirmar versión (v17/v18 vs v19) en Step 1: cambia todo el sistema de options
- Declarar dependencia `website` en `__manifest__.py`
- Registrar el bloque con `t-snippet` + grupo o `t-thumbnail`, y dar `keywords` para la búsqueda
- Casar `data-selector` con la clase raíz `.s_<nombre>` del snippet
- Declarar el JS de options en `website.assets_wysiwyg` y el SCSS en `web.assets_frontend`
- Marcar nodos no editables con `o_not_editable`/`oe_unremovable` para sobrevivir la sanitización
- Usar `onBuilt()` para inicialización al soltar y `cleanForSave()` para limpiar antes de guardar
- **Provenance marker (ALWAYS):** every generated file (except `.csv`/`.json` — no comment syntax) ends with `Made with odoo-website-snippets · v<version del frontmatter de esta skill>`, wrapped in the file-type comment, idempotent. Full rule: `references/shared/skill-provenance-marker.md`.

## Delegates to

| Skill | Suite | Cuándo |
|-------|-------|--------|
| odoo-frontend | common | Diseñar el HTML/Bootstrap del contenido visual del snippet |
| odoo-website-frontend-js | developer | Interactividad pública del snippet en el sitio (`publicWidget`/`Interaction`) |
| odoo-website-controller | developer | Controller que sirve datos al snippet dinámico (`/website/snippet/...`) |
| odoo-version-api-reference | common | Diffs editor v17/v18 → `html_builder` v19 |
| odoo-module-scanner | common | Inventariar snippets/assets del módulo antes de ampliar |
| odoo-test-generator | developer | Tours `HttpCase` que arrastran el snippet y prueban opciones |

## Consumed by

- `odoo-module-creator` — cuando un módulo necesita aportar snippets al website builder
- `odoo-ecommerce-developer` (developer) — snippets de website reutilizables no específicos del shop
- `odoo-theme-developer` (developer) — snippets reutilizables drag-and-drop con sus options para el tema

## REFERENCES

| Fichero | Contenido |
|---------|-----------|
| [`references/snippet-anatomy.md`](references/snippet-anatomy.md) | Plantilla `s_*.xml`, registro en `snippets.xml` (`t-snippet`/`group`/`t-thumbnail`/`keywords`), `data-snippet`, thumbnails, declaración de assets/bundles |
| [`references/snippet-options.md`](references/snippet-options.md) | Panel de opciones: `inherit_id="website.snippet_options"`, `data-js`/`data-selector`/`data-target`, widgets `we-*`, métodos `selectClass`/`selectStyle`/`onBuilt`, `options.registry` (v17/v18) vs Plugin v19 |
| [`references/dynamic-snippets.md`](references/dynamic-snippets.md) | `s_dynamic_snippet_template`, `filterId`/`templateKey`/`numberOfRecords`, controller de filtros, public widget de render |
| [`references/gotchas.md`](references/gotchas.md) | 12+ gotchas reales con path:line: option que no aparece, selector mal, sanitización, asset, `data-snippet`, v19 |
