# Odoo Connection Modes

This skill NEVER asks for Odoo credentials directly. It delegates every authentication and CRUD
call to one of two upstream skills/tools: `odoo-pilot` or an Odoo MCP server.

At the start of any workflow, ask:

> How do you connect to Odoo? `odoo-pilot` or an Odoo MCP server?

If the user has not stated a preference, ask. Don't assume.

## Mode 1 — `odoo-pilot`

[`odoo-pilot`](https://github.com/dynapps-spain-ai/common-skills) is the canonical CRUD-over-HTTP
wrapper for Odoo. It **auto-detects** the protocol: JSON-RPC for Odoo ≤ 18, JSON2 (`/json/2/...`)
for Odoo ≥ 19. The version detection and protocol selection are not this skill's concern — they
are handled by `auth.sh` / `detect_version.sh`.

```bash
# Env vars (set by the user / by odoo-pilot — this skill never prompts for them)
export ODOO_URL="https://your-instance.odoo.com"
export ODOO_DB="your_database"
export ODOO_USER="user@example.com"
export ODOO_KEY="your-api-key"

# Authenticate (auto-detects protocol, exports ODOO_PROTOCOL/ODOO_UID/ODOO_SESSION_ID)
eval $("$ODOO_PILOT_PATH"/scripts/auth.sh)

# Then use the CRUD scripts:
"$ODOO_PILOT_PATH"/scripts/search_records.sh website.page '[]' '["id","url"]' 10
"$ODOO_PILOT_PATH"/scripts/create_record.sh   ir.ui.view '{...}'
"$ODOO_PILOT_PATH"/scripts/update_record.sh   ir.ui.view '[<id>]' '{...}'
"$ODOO_PILOT_PATH"/scripts/execute_method.sh  website.page copy '[<id>]' '{}'
```

### odoo-pilot path resolution

The skill's scripts must locate `odoo-pilot` at runtime — never hardcode an absolute path. Resolve
in this order:

```bash
ODOO_PILOT_PATH="${ODOO_PILOT_PATH:-}"                       # 1. explicit env var wins
[ -z "$ODOO_PILOT_PATH" ] && ODOO_PILOT_PATH=$(ls -d \
      "$HOME/.claude/plugins/cache"/*/*/*/skills/odoo-pilot \
      "$HOME/.config/opencode/skills/odoo-pilot" \
      "$HOME/.dyn-skills"/*/suites/*/skills/odoo-pilot \
      2>/dev/null | head -1)                                 # 2. marketplace cache (cualquier agente)
[ -z "$ODOO_PILOT_PATH" ] && ODOO_PILOT_PATH=$(find "$(dirname "$0")/../.." \
      -type d -name odoo-pilot 2>/dev/null | head -1)        # 3. sibling skills/odoo-pilot
[ -z "$ODOO_PILOT_PATH" ] && { echo "odoo-pilot not found; install it or set ODOO_PILOT_PATH"; exit 1; }
```

### GOTCHA — `indaws_security` breaks `odoo-pilot`'s *stateless* auth

On any instance with the Indaws Core module **`indaws_security`** installed (common across the
Indaws/Dynapps fleet; it auto-installs in its `addons_path`), the **stateless** authentication of
`auth.sh` **fails**: both `common.authenticate` (JSON-RPC/XML-RPC) and the API-key method crash with

```
RuntimeError: object is not bound
  …/indaws_security/models/indaws_security_ip_ban.py in check_ip_ban
      ip_address = ip_address or request.httprequest.remote_addr
```

`check_ip_ban` assumes a *bound* HTTP `request`, which does not exist in a pure RPC call. Symptom in
`auth.sh`: *"Session auth failed, trying API key… Authentication failed with both JSON2 and JSON-RPC"*
and an empty `ODOO_SESSION_ID`. (Also, in Odoo 18 `/web/session/authenticate` no longer returns the
`session_id` in the body, only in the cookie, so `auth.sh`'s parsing doesn't capture it either.)

**Solution — bootstrap via WEB SESSION (cookie)**, like the browser does. The `/web/session/authenticate`
login does have `request` bound → it works. Then `odoo-pilot`'s `execute_kw` scripts use that cookie
(`execute_kw` does **not** trigger `check_ip_ban`, only the login does):

```bash
# 1) web login -> cookie + uid  (NOTE: in zsh do NOT use the variable UID, it is read-only)
curl -s -c /tmp/ck.txt -o /tmp/auth.json -X POST "$ODOO_URL/web/session/authenticate" \
  -H 'Content-Type: application/json' \
  -d "{\"jsonrpc\":\"2.0\",\"params\":{\"db\":\"$ODOO_DB\",\"login\":\"$ODOO_USER\",\"password\":\"$ODOO_KEY\"}}" >/dev/null
export ODOO_UID=$(python3 -c "import json;print((json.load(open('/tmp/auth.json'),strict=False).get('result') or {}).get('uid') or '')")
export ODOO_SESSION_ID=$(awk '/session_id/{print $7}' /tmp/ck.txt | tail -1)
export ODOO_PROTOCOL=jsonrpc          # v18: siempre JSON-RPC
# 2) from here on, odoo-pilot works normally:
"$ODOO_PILOT_PATH"/scripts/search_records.sh website '[]' '["id","name"]' 5
```

If you're going to write your own Python script (e.g. `apply_*_odoo.py`), use the same per-session login
and `POST /web/dataset/call_kw` with the `CookieJar` — avoid depending on `auth.sh` on Indaws instances.

### When `odoo-pilot` is the right choice
- Local Claude Code sessions where the user already has the plugin installed.
- Scripts and batch ops — shell scripts compose more easily than MCP tool calls.
- When `odoo-pilot` is already imported in the current session.

### Operations needed by this skill

| Operation | `odoo-pilot` script + arguments |
|-----------|---------------------------------|
| Detect Odoo version / protocol | `detect_version.sh` |
| Check if `website_crm` is installed | `search_records.sh ir.module.module '[["name","=","website_crm"]]' '["state"]' 1` |
| Install `website_crm` (lead forms) | `install_module.sh website_crm` |
| Find the target `website` id | `search_records.sh website '[]' '["id","name","domain"]' 10` |
| Get `crm.lead` ir.model id (form whitelist check) | `search_records.sh ir.model '[["model","=","crm.lead"]]' '["id","website_form_access"]' 1` |
| Create the QWeb view (page arch) | `create_record.sh ir.ui.view '{"name":"…","type":"qweb","key":"website.<slug>","arch":"<t t-call=…>"}'` |
| Create the page record (unpublished) | `create_record.sh website.page '{"name":"…","url":"/…","view_id":<vid>,"website_id":<wid>,"is_published":false}'` |
| Update an existing view's arch | `update_record.sh ir.ui.view '[<vid>]' '{"arch":"…"}'` |
| Upload image / CSS as attachment | `create_record.sh ir.attachment '{"name":"…","datas":"<b64>","public":true,"mimetype":"image/png","type":"binary"}'` |
| Register site-wide CSS/JS (no module) | `create_record.sh ir.asset '{"name":"…","bundle":"web.assets_frontend","directive":"append","path":"/web/content/<att_id>","active":true}'` |
| Create a menu entry for the page | `create_record.sh website.menu '{"name":"…","url":"/…","website_id":<wid>,"parent_id":<root_menu_id>,"sequence":10}'` |
| Duplicate an existing page | `execute_method.sh website.page copy '[<id>]' '{}'` (and/or `ir.ui.view copy`) |
| Verify the page exists / arch not empty | `search_records.sh website.page '[["url","=","/…"]]' '["id","is_published","view_id"]' 1` |

> Exact field names for `ir.asset` (`path` vs `target`, `directive` values) and the menu root id
> can vary by instance/theme. **Verify on the target instance** before relying on them; do not
> assume. See `css-injection-ladder.md` for the fallback ladder.

## Mode 2 — Odoo MCP server

If the user has an Odoo MCP server configured (e.g. the one shipped with Odoo Enterprise, or a
community MCP for Odoo), call its tools directly:

```python
mcp.call("odoo.create_record", model="ir.ui.view",   values={...})
mcp.call("odoo.create_record", model="website.page",  values={...})
mcp.call("odoo.update_record", model="ir.ui.view",    ids=[<vid>], values={"arch": "…"})
mcp.call("odoo.execute_method", model="website.page",  method="copy", args=[<id>])
```

Tool names vary by MCP server. Discover them via `ListMcpResourcesTool` or the MCP server's
docs, then map every row of the operations table above to the available MCP tool names.

### When MCP is the right choice
- Cloud Claude Code sessions using Anthropic Cloud MCP integrations.
- Production environments with a single managed Odoo connection.
- When the user explicitly says "tengo MCP de Odoo configurado".

## Credentials — what NEVER to do

This skill MUST NOT:
- Prompt the user for `ODOO_URL`, `ODOO_DB`, `ODOO_USER`, `ODOO_KEY`, or any API token.
- Read or write `~/.config/<anything>` to persist credentials.
- Reference specific instance URLs (no `https://www.indaws.es`) — they belong to whoever runs the skill.
- Hardcode example tokens in references or scripts.

If the user has configured neither `odoo-pilot` nor an MCP server, instruct them to set one up
first (point to the respective skill) and stop. Do not try to proxy credentials.

## Determining what's connected

A reasonable runtime check:

1. Is the `odoo-pilot` skill loaded / its path resolvable? If so, default to `odoo-pilot`.
2. Are there MCP servers offering Odoo-shaped tools? If so, offer MCP.
3. Neither? Ask the user to install one and stop.

When ambiguous, ask. Don't assume.
