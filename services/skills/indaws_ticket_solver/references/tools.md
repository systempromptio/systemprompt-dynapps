# Herramientas — Ticket Solver

Dos instancias, dos juegos de herramientas. No los mezcles: el MCP proxy de Odoo
**solo** llega a la corporativa; los scripts RPC son para el Odoo del cliente.

## Instancia corporativa (helpdesk) — MCP proxy Odoo (nombre de servidor variable)

Esquemas bajo demanda con `ToolSearch` **por palabra clave** (`orm_search_read`,
`orm_post_message`, `whoami`, etc. — NUNCA `select:mcp__odoo-indaws__<tool>`: ese
nombre de servidor es el de Claude Code y puede no existir en otros entornos, ver
`references/credentials-setup.md`). Reutiliza el prefijo `mcp__<servidor>__` que
resulte de esa búsqueda para cada herramienta de la tabla siguiente.

| Herramienta | When |
|-------------|------|
| `whoami` | Una vez por sesión. Fija tu identidad: eres el autor de las notas. |
| `orm_search_read` | Leer `helpdesk.ticket`, `mail.message`, `res.partner`, `res.users`. SIEMPRE `fields=[...]` y `limit`. |
| `orm_post_message` | ÚNICA vía de publicación. `body_is_html=true`, `subtype_xmlid="mail.mt_note"`. |
| `orm_write` | Solo si hay que tocar el propio ticket (stage, asignado). |
| `orm_create` | Fallback de publicación sobre `mail.message` si `orm_post_message` altera el HTML (GOTCHA 2). |
| `odoo_attachment_search` → `odoo_attachment_link` | Adjuntos del ticket: devuelve URL pre-firmada de TTL corto. Descarga con `curl`, luego `process_attachments.py` / `process_video.py`. |
| `orm_execute_kw` | Métodos puntuales sobre la corporativa. Evítalo si `orm_search_read` basta. |

### Lecturas mínimas del ticket

```
orm_search_read helpdesk.ticket
  domain=[["id","=",<id>]]
  fields=["name","description","partner_id","team_id","stage_id","user_id",
          "project_id","create_date","write_date","priority"]

orm_search_read mail.message
  domain=[["model","=","helpdesk.ticket"],["res_id","=",<id>]]
  fields=["date","author_id","body","message_type","subtype_id"]
  order="date desc"  limit=10
```

Nunca pidas `description`/`body` de más de ~10 mensajes de golpe: el chatter viene
en HTML crudo y se come el contexto. Si necesitas más histórico, pagina con `offset`.

> **Coste asumido**: al leer por MCP el bundle entra en contexto sin el resumen
> compacto que hacía `fetch_ticket_bundle.py --summary` (-70% tokens).
> Compénsalo acotando `fields`, bajando `limit` y no releyendo lo que ya tienes
> en el turno.

### Reincidencias (sustituye a `find_prior_resolutions.py`)

```
orm_search_read helpdesk.ticket
  domain=[["partner_id","=",<pid>],["stage_id.fold","=",true],
          ["create_date",">=","<hoy-90d>"]]
  fields=["id","name","create_date"]  limit=10
```
Luego lee el chatter de los candidatos con título parecido y quédate con el
snippet `Resolución:`.

## Instancia del cliente — scripts RPC

`odoo-pilot/scripts/*.sh` (`auth.sh`, `search_records.sh`, `create_record.sh`, `update_record.sh`, `delete_record.sh`, `execute_method.sh`). Autodetecta JSON-RPC v≤18 / JSON2 v≥19. Credenciales `CLIENT_ODOO_*`.

| Script | When |
|--------|------|
| `bootstrap_credentials.sh <slug>` | Entry point del turno cuando hay cliente. Carga `CLIENT_ODOO_*`. Llámalo en cada Bash tool call. |
| `ensure_credentials.sh <item> [vault] [prefix]` | Primitiva debajo de bootstrap. Útil para cargar un item ad-hoc. |
| `vault_index.sh resolve <slug> [vault]` | Cachea índice de bóveda (TTL 24h). Resuelve slugs: exacto → case-insensitive → substring. |
| `odoo_query.py` | `fields_info`, `view_arch`, `resolve_menu`, `resolve_action`. |
| `rollback.py <backup.json>` | Restaurar mutaciones. |
| `recompute_field.py --model M --domain D --trigger-field F [--confirm]` | Fuerza recompute de campo compute store. Dry-run por defecto. |
| `_ts_rpc.py` | Helper RPC interno. Autodetecta JSON2/JSON-RPC. No se invoca directamente. |

## Locales (sin Odoo)

| Script | When |
|--------|------|
| `process_attachments.py` / `process_video.py` | Adjuntos multimedia ya descargados. |
| `known_issues.py query\|append` | KB por cliente. Solo si el consultor lo pide o patrón recurrente. |
| `precompute_module_index.py` | Regenera `module_index.md` (manual, tras instalar módulos). |

## Legacy — solo modo headless / cron

Estos scripts hablan con la **corporativa por RPC** y quedan como fallback para
ejecuciones sin sesión interactiva (cron, Workflow), donde el MCP no está
disponible. Requieren `ODOO_*` de 1Password (`bootstrap_credentials.sh --with-indaws`):
`fetch_ticket_bundle.py`, `post_internal_note.py`, `find_prior_resolutions.py`,
`fetch_consultor_voice.py`, `download_attachments.py`, `ts_auth.py`.

En sesión interactiva **no los uses**: si el MCP no está conectado, para y pide `/mcp`.

## Flags globales

`--dry-run` (sin mutaciones / append / vídeos). `--batch <ids>` (serie de tickets, reutiliza auth).
