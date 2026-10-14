# Engine / Framework — Delegado

El contenido de engine (ORM, HTTP, OWL/JS, QWeb, performance, services) se ha delegado a:

**`odoo-version-api-reference`** skill → `references/v18-base/`

| Antes (engine/) | Ahora (version-api-ref) |
|-----------------|------------------------|
| orm.md | v18-base/orm-models.md + orm-environment.md |
| owl_js.md | v18-base/javascript-*.md (4 ficheros) |
| qweb.md | v18-base/javascript-views-widgets.md |
| http.md | v18-base/http.md |
| services.md | v18-base/services.md + tools.md |
| performance.md | engine-patterns.md |
| enterprise.md | v18-base/base-models.md (mixins section) |
| v19_changes.md | diffs/v19-diff.md |

Los ficheros originales están en `_archive/engine_original/` por si se necesitan.

Para consultas sobre el framework Odoo, usar directamente `odoo-version-api-reference`.
