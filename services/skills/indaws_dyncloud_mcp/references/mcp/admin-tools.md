# DynCloud MCP — admin & system tools

Modules and health. The `admin_*` tools are category `module_admin`; the `system_*` ones
are `system_ops`. Both are gated on `production` and usually open on `development`.
Shared conventions: `README.md`.

---

## system_odoo_version

The first call to make on an environment you have not used before, and the one that tells
you whether the MCP will work there at all.

```json
{ "company_id": 3, "project_id": 901, "environment_id": 4022,
  "db": "acmedb", "target_user_login": "admin" }
```
```json
{ "ok": true,
  "data": { "odoo_version": "19.0+e", "odoo_serie": "19.0", "edition": "enterprise",
            "dyncloud_api_version": "19.0.0.3.0",
            "capabilities": ["raw_binary_write"] },
  "elapsed_seconds": 0.09 }
```

Read `capabilities`, **not** the version string. Environments are rebuilt on their own
schedule, so the fleet is never on one version, and a feature is either advertised or it is
not. A build older than the field returns **no `capabilities` key at all** — read that as
"supports none of them".

`raw_binary_write` is the one that matters for imports: without it you cannot load a file
into `base_import.import.file` (`file-tools.md`).

A **404** here is the diagnosis, not an error to retry: `dyncloud_api` is not installed on
that environment (`workflows/troubleshooting.md` §1).

---

## system_health_check

Is it up and answering. Cheap sanity check before a batch of calls, and worth quoting when
reporting a problem.

```json
{ ... location args ... }
```

---

## system_list_modules

What is installed, with an optional `state` filter (`installed` by default; also
`to_install`, `to_upgrade`, `uninstalled`, `all`).

```json
{ ... location args ..., "state": "installed" }
```

**Call it before and after any module operation.** Before, to avoid installing what is
already there; after, because an uninstall removes dependants it never names and this is
the only way to see the real extent of the change.

---

## admin_install_module

```json
{ ... location args ..., "module_names": ["sale_management", "project"] }
```
```json
{ "ok": true, "op": "install",
  "modules": [ { "module": "sale_management", "state": "installed" },
               { "module": "project", "state": "installed" } ] }
```

**Pass every module in one call.** Odoo resolves the combined dependency tree in a single
transaction: it is faster than separate calls (one registry rebuild instead of N) and it
cannot leave the database half-installed when a later module fails.

Notes:

- Names are resolved **before** anything is pressed. An unknown name returns
  `{"ok": false, "error": "e_module_not_found", "missing": [...]}` with **nothing changed** —
  one typo among good names aborts the whole call rather than installing half of it.
- Installing pulls dependencies in, so the database ends with more modules than you named.
  The answer reports the ones you asked for; `system_list_modules` shows the rest.
- **Minutes, not seconds**, for a wide tree. The wait is the server's *Module Admin
  Timeout*. If it times out **the install is not cancelled** — re-read the state before
  retrying (`README.md` → "The six timeouts").

---

## admin_upgrade_module

Same shape, same list semantics.

```json
{ ... location args ..., "module_names": ["sale", "project"] }
```

This runs the modules' **migration scripts against real customer data**. Confirm with
`system_list_modules` that each one is actually `installed` first — upgrading something
that is not installed fails.

---

## admin_uninstall_module

```json
{ ... location args ..., "module_names": ["sale_management"] }
```

**Destructive and irreversible: it drops the modules' tables and data.** Uninstalling
`account` deletes that customer's journal entries, journals and taxes. There is no undo
short of restoring a backup.

And the part that surprises people: **it drags out every module that depends on the ones
you name.** Removing `stock` takes `sale_stock` with it. The answer lists only the modules
you asked for — dependants are removed silently — so re-read with `system_list_modules`
to see what actually happened.

Never call this speculatively or "to clean up". Say out loud what will be lost and get an
explicit yes.

---

## admin_update_module_list

Refreshes the target's module list from its addons path. Run it when a module was just
added to the environment's code and does not yet appear in `system_list_modules` — that is
the normal reason `admin_install_module` answers `e_module_not_found` for something you
know is there.

---

## admin_restart_worker

**Not implemented** on the target as of this writing: it short-circuits with
`E_NOT_IMPLEMENTED` without calling anything. If an environment genuinely needs a restart,
that is a platform operation and belongs to the technical team, not to this MCP.
