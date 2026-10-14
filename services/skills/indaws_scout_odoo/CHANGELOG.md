# Changelog — scout-odoo

## 2.1.2 (2026-07-26)

Portabilidad entre agentes. Cambios:

- **SKILL.md**: la instrucción de conexión del MCP ya no asume la UI de Claude Code. Indica la URL y deja la ruta concreta como dependiente del agente (Ajustes → Conectores en Claude Code, bloque de configuración MCP en otros). El [STOP] y la prohibición de psql/psycopg2 se mantienen intactos.

Nota: la referencia a `~/.claude/skills/scout-odoo/.env` del apartado de migración de la 2.1.0 se conserva a propósito — documenta una limpieza histórica para quien venía de la v1.x en Claude Code, no una dependencia activa.

## 2.1.0 (2026-06-04)

Auditoría con skill-designer (22-point checklist). Cambios:

- **Frontmatter**: descripción recortada a <1024 chars (límite duro de Anthropic — la v2.0 se truncaba). Eliminados `allowed-tools: Read Write` (bloqueaba las tools MCP) y el ghost field `model`.
- **Nombres MCP cualificados**: `odoo-scraper-mcp:scraper_run_query` etc., según guía oficial.
- **[STOP] points**: MCP no conectado y resolución ambigua de nombres (nunca elegir candidato por el usuario).
- **Sección NEVER/ALWAYS** con reglas justificadas (no PostgreSQL directo, exclusión Indaws por defecto, seed cutoff, escape de comillas).
- **Progressive disclosure**: esquema completo movido a `references/schema.md`; SKILL.md queda como índice (<150 líneas). GOTCHAS consolidados (9, incluye el de Indaws country=NULL).
- **Dedupe**: "Recordatorios finales" de aliases.md eliminados (duplicaban SKILL.md). TOC añadido a aliases.md.
- **Enriquecimiento de contactos retirado del skill** (no se hace aquí); delegado a `marketing-skills:lead-prospector`. Añadidas tablas Delegates to / Consumed by.


## 2.0.0 (2026-06-04)

Migración completa a MCP. El acceso a la BD del scraper ya NO usa PostgreSQL directo con usuario/contraseña — todo va por el servidor MCP `odoo-scraper-mcp`:

```
https://odoo-scraper-mcp.xv.infra.dyncloud.io/mcp
```

Tools: `scraper_run_query`, `scraper_list_tables`, `scraper_describe_table`, `scraper_sample_table`.

### Cambios

- **SKILL.md** reescrito: acceso exclusivo vía MCP, sin credenciales, sin setup. Esquema y gotchas ampliados (slowly-changing vs snapshots, cap de filas, `odoo_partners.stop_date` no significa cesado).
- **references/aliases.md** reescrito: los aliases del CLI son ahora plantillas SQL listas para `scraper_run_query`, con placeholders, variantes y los mismos defaults (excluir Indaws 36, seed cutoff 2025-07-01).
- **commands/** (7): todos actualizados de `python3 query.py ...` a la receta SQL + MCP correspondiente. Export CSV/Excel ahora se hace con file tools sobre el JSON resultante.
- **ELIMINADOS**: `scripts/query.py`, `scripts/test.sh`, `.env.example`, `.gitignore`. Ya no hay credenciales que gestionar ni dependencia de psycopg2.

### Migración para usuarios

Nada que configurar. Si las tools `scraper_*` no aparecen en sesión, conectar el MCP en Ajustes → Conectores con la URL de arriba. Borrar `~/.claude/skills/scout-odoo/.env` si existe (ya no se usa).
