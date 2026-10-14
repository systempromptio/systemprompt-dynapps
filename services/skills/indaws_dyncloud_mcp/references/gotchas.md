# Gotchas — the traps this MCP actually sets

Every one of these has cost somebody real time. Most share a shape: **the call succeeds,
the answer looks like success, and something the user believes happened did not.** Read
this before reporting a result back, not after.

## Reaching the environment

- **404 on every ORM call means the MODULE is missing, not the record.** `dyncloud_api` is
  not installed on that environment. Two calls confirm it: `list_databases` (works) +
  `system_odoo_version` (404). Its `auto_install` only carries it in three situations, and
  only the first is automatic:
  - a **fresh** database — `initialize()` walks the auto-install set recursively and both
    modules land on their own;
  - a **restored** database (`pg_restore` from odoo.sh or another host) — `initialize()`
    never runs again, so nothing is auto-installed until something names the modules
    explicitly;
  - an environment that **already had `dyncloud_auth`** — the auth module only ever
    *upgrades*, never re-enters `'to install'`, so it pulls `dyncloud_api` in under no
    circumstances. The code sits in the addons path, listed and uninstalled, forever.

  Which one you are looking at decides who fixes it and how, so say it in the report rather
  than just "the module is missing".
- **`list_databases` and `list_target_users` are real round-trips to the customer's Odoo.**
  They are your cheapest health check — and they need no grant on any stage.
- **A project or environment with `mcp_enabled` off is invisible**, not forbidden. No JIT
  grant makes it appear; someone has to flip the flag in the manager.
- **`list_environments` cascades:** the project *and* the environment must both be enabled.
- **Never assume `main`.** A project has several environments and the names are not
  standardised. Ask which one, and say which exist.

## Permissions

- **`state` is not `effective_state`.** `state` is stored; `effective_state` is computed
  against `expires_at` in real time and is what enforcement uses. A grant reading
  `state: "active"`, `effective_state: "expired"` is expired, and calls with it are denied
  exactly as if it had never existed.
- **`E_SCOPE_NOT_REQUESTABLE` is not a denial.** The category is already open on this
  stage: call the tool. Requesting a grant you do not need is noise in an audit log people
  actually read.
- **`E_CATEGORY_FORBIDDEN` / `E_SCOPE_FORBIDDEN` are final.** Not grantable on this stage.
  Retrying or requesting fails identically — stop and say so.
- **Use the `request_hint` verbatim.** It names the category the failed call used; guessing
  a different one produces a grant that unlocks nothing.
- **A pending grant unlocks nothing and does not resolve itself.** It waits for an
  `mcp_approver` of that project. Tell the user, hand them the reference, and stop — do not
  loop retrying.
- **Masked values are policy, not corruption.** `hash-c4e29035`, `+34 ***1001`, with
  `_masked_fields` naming which. Aggregating over a masked field is refused on purpose.

## Timeouts

- **A timeout cancels nothing.** The environment keeps working. A module install that timed
  out has very likely finished; retrying blind starts the same work twice. Read the state
  first — `to install` means still running, not failed.
- **The shape of the error says which of the six fired.** Bare `The operation timed out.`
  with **no `error_code`** → your client, on the user's machine. `E_TARGET_TIMEOUT` → the
  manager, and the message names the limit and the field that changes it. Paraphrasing the
  error destroys the only evidence you had.
- **Raising the manager's timeout does not help if the client gives up first.** In the
  session that produced this skill, the manager was waiting 900 s and the client cut at far
  less — the fix was in the user's own MCP configuration.

## Writing

- **`orm_call_method` is `data_write` whatever the method does.** So a module operation run
  through it gets the short data-write budget, not the long module-admin one. Use the
  `admin_*` tools for modules.
- **"Borrar" almost never means `orm_unlink`.** Archive (`{"active": false}`) preserves
  history and is reversible. Odoo refusing to delete a referenced record is protection.
- **Count before a mass write.** There is no dry-run for `orm_write`; `orm_search_count`
  plus a single-record test is the substitute.
- **Settings are a wizard, not a table.** `res.config.settings` must be created **and**
  `execute`d; writing the underlying parameter skips the logic that creates records and
  activates groups. And a `module_*` field on that wizard **installs a module** — a module
  operation wearing a checkbox.

## Modules

- **Pass every module in ONE call.** Odoo resolves the combined dependency tree in a single
  transaction: faster, and it cannot leave the database half-installed when a later module
  fails.
- **An unknown name aborts everything.** `e_module_not_found` with a `missing` list and
  **nothing changed** — deliberately, instead of installing the valid half.
- **Uninstall removes modules it never names.** Every dependant comes out too — `stock`
  takes `sale_stock` — and the answer lists only what you asked for. Re-read with
  `system_list_modules` to see the real extent, and say what will be destroyed *before*
  calling.
- **`e_module_not_found` for code you can see** usually means the target's module list is
  stale: `admin_update_module_list` first.
- **`admin_restart_worker` is not implemented** — it answers `E_NOT_IMPLEMENTED` without
  doing anything.

## Files and imports

- **`"raw": true` belongs to exactly one field in practice.** `base_import.import.file`
  stores raw bytes, not base64. Without the flag the field silently receives base64 *text*
  and the import dies later with *"Unable to read file"*; **with** it on an ordinary binary
  field (`ir.attachment.datas`, an image) the content is stored wrong instead. Both
  failures are silent at write time.
- **`raw_binary_write` must be in `capabilities`.** Check `system_odoo_version`; an older
  environment fails with `E_TARGET_LACKS_RAW_BINARY` — loudly, which is the intended
  behaviour.
- **`parse_preview` is unusable through the proxy.** It answers with the model's entire
  field tree (236 KB for `res.partner`) and the proxy truncates it. Go straight to
  `execute_import` with an explicit mapping.
- **The dry-run's ids are not the real run's ids.** Never report them as created.
- **An uploaded file lands on the MANAGER, not on the customer's Odoo.** Nothing reaches
  the target until a value-ref injects it into a write.
- **`consume: true` is one-shot.** Referencing a consumed id fails with the same message as
  an id that never existed — the ambiguity is deliberate, so the error cannot be used to
  probe.
- **An attachment without `res_model` + `res_id` hangs off nothing.** It exists, and to the
  user it looks like it never arrived.
- **Verify a transfer by `checksum`**, not by the call returning ok. It is the SHA-1 of the
  stored content; compare it with the local file.
- **A small binary comes back as inline base64, not a descriptor.** That is the threshold
  working, not a missing feature.

## The client itself

- **The tool catalogue is cached when your MCP client starts.** After the manager is
  deployed, a running session keeps the old tools, old argument names and old bugs.
  **Restart before debugging** a tool or an argument that "does not exist" — in the session
  that produced this skill, `module_name` had become `module_names` and a whole new tool
  was invisible until restart.

- **`odoo-connection.md` is not the fallback here.** The shared contract
  (`references/shared/odoo-connection.md`) covers an Odoo you hold credentials for.
  On this path you hold none: the manager mints a short-lived JWT and impersonates
  `target_user_login` on the customer's instance. So a **404 is not a routing problem** to be
  worked around with odoo-pilot — it means `dyncloud_api` is not installed on the target, and
  going around the manager would skip the JIT permission grant that makes the call legitimate.
