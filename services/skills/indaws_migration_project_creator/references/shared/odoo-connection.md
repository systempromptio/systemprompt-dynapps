# Odoo Connection Pattern

Single source of truth for connecting to Odoo from any skill. **Do NOT reimplement this in your skill — reference this document.**

## Connection strategy: MCP primary, odoo-pilot fallback

Two ways to reach Odoo. **Pick by what's available, in this order:**

| Path | When to use | How |
|------|-------------|-----|
| **Indaws MCP (primary)** | Interactive sessions against an instance with the MCP server installed — **default for the v19 corporate instance** (`mcp__claude_ai_INDAWS_MCP__*`) and its staging (`mcp__claude_ai_INDAWS_STAGING_MCP__*`). | Native MCP tools in Claude. Schemas load on demand via `ToolSearch` (`select:mcp__claude_ai_INDAWS_MCP__...`). v19 speaks JSON2 + ships the MCP server natively, so this is first-class there. |

**Indaws MCP tool inventory** (generic ORM proxy): `whoami`, `orm_search_read`, `orm_create`, `orm_write`, `orm_unlink`, `orm_execute_kw`, plus helpers `odoo_list_models`, `odoo_describe_model`, `odoo_get_selection`, `odoo_aggregate`, `odoo_resolve`, `odoo_attachment_search`, `odoo_attachment_link`, `odoo_request_upload`. Auth is OAuth — the user must connect the server in their agent once per session (in Claude Code, via `/mcp`); you cannot complete the handshake yourself.

> **Access model — verified empirically, NOT as the server announces.** The server's own MCP instructions claim it "runs as superuser with sudo, bypassing record rules" — **this is false.** A leaked traceback shows the real implementation is `env[model].with_user(user).search_read(...)`, i.e. it runs as the **authenticated user with `su=False`, so record rules and access rights ARE enforced** (same effective access as odoo-pilot under that user). **Writes verified too:** an `orm_write` to a real task succeeded and persisted as the authenticated user (read-back confirmed) — reads are *proven* `with_user` (the traceback), writes are *verified to function* under the user's permissions (the write-path traceback couldn't be provoked: probing it requires an unauthorized production write, correctly blocked). Treat the server's self-description as unreliable; trust observed behavior. Binary/image fields return a value-ref placeholder, never base64 — use `odoo_attachment_link` / `odoo_attachment_search` to get a short-TTL download URL.
>
> **This applies to ANY Indaws/Dynapps MCP proxy, not just `INDAWS_MCP`.** The generic `Dynapps_Spain_MCP` proxy carries the same false "sudo/superuser" self-description, yet a created `project.task` came back with `create_uid` = the **authenticated user** (verified 2026-07-17, read-back). **Rule: when authorship/audit matters, do NOT assume the record is owned by a system user — verify `create_uid` by read-back.** If it truly needs to be the user and isn't, only THEN redo via odoo-pilot authenticated as that user. Don't warn the user about "superuser authorship" without having checked it.

> **Attachment upload gotcha.** To attach a file, the flow is `odoo_request_upload` → POST the bytes to the one-time URL → reference the returned attachment. But `odoo_request_upload` with a pre-set `model`/`res_id` **fails on some models** (`Upload target model <X> is not writable on this server`, e.g. `project.task`). Workaround: request the upload **untargeted** (no `model`/`res_id`) to get a bare `attachment_id`, POST the bytes (`curl -F "file=@path;filename=NAME"` — the `filename=` overrides so the local temp name doesn't leak), then link it. Two ways to link:
> - **To a record's chatter (preferred for tasks/messages):** `orm_post_message(model, res_id, body, attachment_ids=[id,…])` — attaches in one step and posts a visible message. **Nuance (verified 2026-07-21):** attachments linked this way keep `res_model=false, res_id=0` — they hang off the `mail.message` m2m, NOT the record. So `odoo_attachment_search([["res_model","=",X],["res_id","=",id]])` returns **nothing**; find them by attachment id or via the message. To remove them cleanly you must `orm_unlink` BOTH the `ir.attachment` **and** the `mail.message` (deleting the message alone orphans the attachment; deleting the attachment alone leaves a broken message).
> - **As a bare record attachment (no chatter post):** `orm_write` on `ir.attachment` setting `res_model` + `res_id` — then `odoo_attachment_search` by res_model/res_id *does* find it.
| **odoo-pilot (fallback)** | (a) **headless / cron / Workflow runs** — interactively-authenticated MCP servers are absent there; (b) **pre-v19 instances** (most client parks) where no MCP module is installed and JSON-RPC is required; (c) the MCP is unreachable / not authenticated. | The bash scripts below (`auth.sh`, `search_records.sh`, …). |

**Decision rule for a skill:** if the target is the v19 corporate instance in an interactive session, prefer the MCP tools. Otherwise (client v18/older, cron, batch, or MCP down) use odoo-pilot. A skill that runs in both contexts (e.g. a cron sync) should try MCP first and fall back to pilot — see `odoo-com-sync` / `crm-health-monitor` for the reference implementation of this dual pattern.

odoo-pilot is **not deprecated** — it stays as the universal fallback. The rest of this document covers it.

### Preflight (MANDATORY before any Odoo access)

Before reaching for odoo-pilot, run this check — **do not silently fall back to pilot when the MCP should have been used:**

1. **Is the session interactive and the target the v19 corporate instance (portal.dynapps.es / staging)?** If no → use odoo-pilot (headless/cron/Workflow, or a pre-v19 client instance). No warning needed; pilot is correct here.
2. **Is the Indaws MCP connected?** Look for the Indaws MCP tools exposed by whichever agent you are running in — under Claude Code they are named `mcp__claude_ai_INDAWS_MCP__*` and discoverable via `ToolSearch`; other agents namespace them differently. If present → **use the MCP** as the primary path.
3. **MCP not connected but it should be (interactive + v19)?** → **[STOP]. Do NOT proceed with pilot.** Tell the user that the Indaws MCP is the primary route to Odoo, that it is not connected, and that they must **connect it in their agent and complete the OAuth handshake themselves** — you cannot do it for them. State the goal, not a menu path: how the server is connected depends on the agent (in Claude Code, the `/mcp` command; in opencode, the `mcp` block of `opencode.json`). Resume via the MCP once they confirm. Only fall back to pilot if the user explicitly declines.

Rationale: the MCP needs an interactive OAuth handshake that the agent cannot complete itself, so the only way to make it the default is to prompt the user instead of quietly using pilot.

## Prerequisites

Configure environment variables via `onboarding` skill or manually:

```bash
export ODOO_URL="https://your-instance.odoo.com"
export ODOO_DB="database_name"
export ODOO_USER="user@email.com"
export ODOO_KEY="api_key_here"        # API key, NOT password
export ODOO_PROTOCOL="jsonrpc"        # or "json2" for v19+
```

## Protocol Selection

| Odoo Version | Protocol | Auth script |
|-------------|----------|-------------|
| ≤ 18.0 | `jsonrpc` | `odoo-pilot/scripts/auth_jsonrpc.sh` |
| ≥ 19.0 | `json2` | `odoo-pilot/scripts/auth_json2.sh` |
| Auto-detect | — | `odoo-pilot/scripts/auth.sh` (recommended) |

**Always use `auth.sh`** — it detects the version and selects the right protocol (tries JSON2, falls back to JSON-RPC).

### JSON-RPC vs JSON2 (what odoo-pilot needs under the hood)

odoo-pilot supports both; you rarely touch this directly (the scripts abstract it), but
when debugging the transport these are the differences:

| Aspect | JSON-RPC (≤18.0) | JSON2 (≥19.0) |
|--------|------------------|---------------|
| Endpoint | `/jsonrpc` | `/api/{model}/{method}` |
| Auth | `call(common, authenticate)` → uid + session (stateful, cookie) | `Authorization: Bearer $ODOO_KEY` per request (stateless) |
| Method call | `call(object, execute_kw, db, uid, pwd, model, method, args, kwargs)` | `POST /api/{model}/{method}` with JSON body |
| Error shape | `{"error":{"data":{"message":"…"}}}` | `{"error":{"message":"…"}}` |
| Batch | not native (loop calls) | not native (loop calls) |

Manual detection (the scripts do this for you): `curl -s "$ODOO_URL/api/res.users/search_read" -H "Authorization: Bearer $ODOO_KEY"` → 404 means JSON2 absent → use JSON-RPC. **Never reimplement protocol logic in a skill** — call `auth.sh`.

## Connection Steps

```bash
# 1. Authenticate (returns UID)
source odoo-pilot/scripts/auth.sh

# 2. Use any operation script
bash odoo-pilot/scripts/search_records.sh "res.partner" '[["is_company","=",true]]' "name,email"
```

## Critical Rules

- **Always include `"context": {"lang": "es_ES"}`** in API calls — otherwise translated fields return English
- **ODOO_KEY is an API key, not a password** — generate in Odoo Settings > Users > API Keys
- **API calls respect record rules** — use `sudo` context only when explicitly needed for cross-company data
- **search_read returns max 80 records by default** — always set explicit `limit` or paginate

## From Your Skill

In your SKILL.md, reference like this:

```markdown
## Prerequisites
Configure Odoo connection via `odoo-pilot` skill. See `references/shared/odoo-connection.md`.
```

Do NOT copy auth scripts, protocol selection, or env var patterns into your skill.
