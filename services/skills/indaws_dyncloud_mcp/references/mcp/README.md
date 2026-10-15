# DynCloud MCP — reference index

The tools, grouped by what you are doing. Read the file for your task; **the conventions
below apply to every tool**, so the individual files do not repeat them.

- **`navigation-tools.md`** — find the environment before you touch it:
  `list_my_companies`, `list_projects`, `list_environments`, `list_databases`,
  `list_target_users`, `set_active_context`, `get_active_context`, `clear_active_context`.
- **`read-tools.md`** — read data and schema on the target: `orm_search`,
  `orm_search_read`, `orm_read`, `orm_search_count`, `orm_read_group`, `orm_fields_get`,
  `orm_default_get`, `orm_name_get`, `introspect_list_models` / `_fields` / `_methods`.
- **`write-tools.md`** — change data: `orm_create`, `orm_write`, `orm_unlink`,
  `orm_call_method`.
- **`admin-tools.md`** — modules and health: `admin_install_module`,
  `admin_upgrade_module`, `admin_uninstall_module`, `admin_update_module_list`,
  `admin_restart_worker`, `system_odoo_version`, `system_health_check`,
  `system_list_modules`.
- **`file-tools.md`** — moving files without putting bytes in the conversation:
  `file_upload_request`, value-refs (`__ref__`), `raw`, `consume`, and the download
  descriptor that comes back instead of a big base64 blob.
- **`access-jit.md`** — the permission system: `request_mcp_access`,
  `get_my_mcp_access`, `revoke_mcp_access`, the categories, how a stage decides, and what
  each denial means.
- **`workflows/`** — how to **chain** them, one file per kind of task, with `README.md` as
  the index: **one line per workflow so you can pick without opening anything.**

## Conventions (apply to every tool)

- **Namespaced call.** The real callable is `mcp__<server-name>__orm_search_read`. These
  files use the short name. The server name is whatever the user configured it as —
  identify it by its tool family (`list_environments`, `orm_*`, `admin_*`), never by a
  name you memorised. If two servers could match, ask before calling.
- **Proxied, not local.** Every `orm_*`, `admin_*`, `introspect_*` and `system_*` call
  leaves the manager and lands on a customer's Odoo. `list_*` up to `list_environments`
  reads the manager's own inventory; `list_databases` and `list_target_users` already hit
  the target.
- **The five location args are required on every proxied call** — `company_id`,
  `project_id`, `environment_id`, `db`, `target_user_login`. No fallback, deliberately.
  Missing one gives `E_MISSING_LOCATION_ARG` naming which.
- **Runs as `target_user_login`.** Their permissions and record rules apply, not yours.
  An empty read can be "that user cannot see it", not "it is not there".
- **Errors are normal answers**, not exceptions: `{"ok": false, "error_code": "...",
  "error": "..."}`. Read the code and adjust. Denials carry a `request_hint` telling you
  exactly what to ask for — use it instead of guessing.
- **Success is `ok: true` plus a payload**, usually nested: the target's own answer arrives
  under `data`, so a search's records are at `data.result.records`. `meta` carries
  `took_ms` and a short `jti` you can quote when reporting a problem.
- **PII may be masked.** Values can come back as `hash-c4e29035` or `+34 ***1001` with a
  `_masked_fields` list naming which. That is the stage's masking policy, not corrupt data.
  Lifting it is a separate JIT scope (`unmask`) and needs a real reason.
- **Large binaries never travel inline.** A binary field over the inline threshold comes
  back as a descriptor with a `download_url`; going the other way you pass a value-ref, not
  base64 (`file-tools.md`).
- **After a manager deployment, restart your MCP client.** The tool list, their schemas and
  their descriptions are negotiated when the session opens. A session started before the
  deploy keeps calling the old tools with the old arguments — including ones that were
  renamed hours ago. If a documented argument "does not exist", restart before debugging.

## The six timeouts

A call crosses six in series. Knowing which fired is the difference between a fix and an
afternoon:

| # | Layer | Typical limit | Configured in |
| --- | --- | --- | --- |
| 1 | **MCP client → manager** | client default | The user's own MCP client config (a per-server `timeout` in ms, or `MCP_TOOL_TIMEOUT`) |
| 2 | Envoy in front of the manager | 1200 s | Platform (chart) |
| 3 | gunicorn on the manager | 1200 s | Platform (chart) |
| 4 | **manager → environment** | 120 s normal, 900 s module admin | The MCP server record in the manager, fields *Environment Timeout (s)* / *Module Admin Timeout (s)* |
| 5 | Envoy of the environment | 1200 s | Platform (chart) |
| 6 | gunicorn of the target | 1200 s | Platform (chart) |

**Telling them apart from the answer alone:**

- Bare `The operation timed out.`, **no `error_code`, no DynCloud envelope** → layer **1**,
  the client. Nothing in DynCloud will fix it; it is raised on the user's machine.
- **`E_TARGET_TIMEOUT`** → layer **4**, the manager. The message names the layer, how long
  it waited, the limit and the field that changes it.

**None of them cancels the work.** The environment carries on. A module install that timed
out at layer 1 has very likely finished — **read the state before retrying**, never retry
blind.

Layers 2, 3, 5 and 6 are platform-wide and rarely the culprit; if you suspect one, that is
a devops question, not an MCP one.

## Scaling this reference

One file per tool group, one file per kind of task under `workflows/`. The rule that
produced both: a reference is only useful if you can reach the part you need **without
reading the rest**. When a page stops passing that test, split it and leave an index
behind.
