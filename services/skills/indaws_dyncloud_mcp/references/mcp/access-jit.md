# DynCloud MCP — JIT access

Nothing is granted permanently. Every tool belongs to a **category**; every environment
sits on a **stage**; the stage decides, per category, whether it is open, needs a
just-in-time grant, or is forbidden outright. This is why the same call works on one
environment and is refused on the next.

---

## The categories

| Category | Covers |
| --- | --- |
| `navigation` | The `list_*` discovery chain |
| `data_read` | `orm_search`, `orm_search_read`, `orm_read`, `orm_read_group`, `orm_search_count`, `orm_name_get` |
| `schema` | `orm_fields_get`, `orm_default_get`, `introspect_*` |
| `data_write` | `orm_create`, `orm_write`, `orm_unlink`, `orm_call_method`, `file_upload_request` |
| `module_admin` | `admin_install_module`, `admin_upgrade_module`, `admin_uninstall_module`, `admin_update_module_list` |
| `system_ops` | `system_odoo_version`, `system_health_check`, `system_list_modules` |
| `access_meta` | The access tools themselves |

Typical shape: a **`development`** environment has most categories open, so you just call
the tool. **`production`** gates `data_write`, `module_admin` and `system_ops` behind a
grant. Do not memorise a matrix — the denial tells you.

There is also a separate scope, **`unmask`**, which lifts PII masking so real field values
come back instead of `hash-…` / `+34 ***…`. It is requested the same way and deserves a
better reason than curiosity.

---

## The four answers, and what each one means

**`E_CATEGORY_NEEDS_GRANT`** — request it and retry:

```json
{ "error_code": "E_CATEGORY_NEEDS_GRANT",
  "error": "Category 'module_admin' on stage 'production' requires an active JIT grant…",
  "request_hint": { "scope": "category", "category": "module_admin",
                    "duration_hours": 2, "tool": "request_mcp_access",
                    "environment_id": 4021 } }
```

**Use the `request_hint` verbatim** — it names the exact category the failed call used.
Guessing a different one gets you a grant that does not unlock anything.

**`E_SCOPE_NOT_REQUESTABLE`** — the category is **already open** on this stage. Nothing to
request; just call the tool. Asking for a grant you do not need is noise in the audit log.

**`E_CATEGORY_FORBIDDEN` / `E_SCOPE_FORBIDDEN`** — never grantable here. **Stop.** Retrying
or requesting fails identically. Tell the user this stage does not allow it at all; if the
work genuinely must happen, it is a platform decision, not a permission you can obtain.

**Grant created but `pending`** — see below.

---

## request_mcp_access

```json
{ "environment_id": 4021, "scope": "category", "category": "module_admin",
  "duration_hours": 2,
  "reason": "Install sale_management on acme-erp/main to reproduce the customer's pricing issue." }
```
```json
{ "grant_id": 5, "name": "BKRG/2026/00500", "state": "active",
  "effective_state": "active", "expires_at": "2026-08-13T12:17:22",
  "remaining_minutes": 120, "auto_approved": true, "requires_approval": false,
  "next_step": "Grant is active immediately…" }
```

- **`reason` is mandatory and stored in the audit log.** Write what you are doing and why,
  as a sentence somebody reading it in three months can use. "testing" is a wasted field.
- `duration_hours` is clamped to the stage maximum (minimum 0.5 h). Ask for what the task
  needs, not the maximum.
- An **identical active grant is returned as-is** rather than duplicated (`idempotent`).
- `scope: "write"` is a deprecated alias for `scope: "category", category: "data_write"`.
  Use the explicit form.

### Auto-approved, or pending

If the caller holds the **`mcp_approver`** role on that project, the grant is **active
immediately** (`auto_approved: true`) and audited. Otherwise it is created **`pending`**,
approvers are notified, and nothing is unlocked until a human signs it off.

**When it stays pending, say so plainly and hand it over.** Poll `get_my_mcp_access` to see
when it turns active; do not retry the original call in a loop. What to tell the user
depends on who they are:

- **Technical profile:** the grant is pending approval on project *X*; an `mcp_approver` of
  that project has to approve it.
- **Consulting profile:** "he pedido el permiso para *X* y está esperando aprobación del
  equipo técnico." Then give them the grant reference (`BKRG/...`), the environment and the
  reason used — that is what an approver needs to act without asking questions.

---

## get_my_mcp_access

Your grants on an environment, all states.

```json
{ "environment_id": 4021 }
```

**Read `effective_state`, not `state`.** `state` is what is stored; `effective_state` is
computed against `expires_at` in real time. A grant showing `state: "active"` and
`effective_state: "expired"` is **expired** — the enforcement uses the effective one, and
a call with it is denied exactly as if the grant had never existed.

`remaining_minutes` tells you whether it is worth starting a long operation now or
requesting a fresh window first.

---

## revoke_mcp_access

Hands a grant back before it expires. Good practice after finishing a sensitive task on a
production environment rather than leaving the window open — the audit trail then shows an
access that was closed deliberately.

---

## Reporting a permission problem

Whatever the profile, a useful report names: the **environment and project**, the **tool**,
the **category**, the **literal error code**, and — if a grant exists — its reference and
`effective_state`. With those five, the technical team resolves it without reproducing
anything.
