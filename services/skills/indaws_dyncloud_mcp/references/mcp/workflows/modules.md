# Workflows — installing, upgrading and uninstalling modules

These change what the customer's Odoo **is**, not just what it contains. They take minutes,
they cannot be half-done safely, and one of them destroys data.

Category `module_admin`, which `production` gates behind a JIT grant, and which gets the
long timeout budget (*Module Admin Timeout*, 900 s by default).

---

## 1. Install one or more modules

```text
system_list_modules { ...five args..., state: "installed" }     # is it already there?

admin_install_module { ...five args...,
                       module_names: ["sale_management", "project"] }
  → { ok: true, op: "install",
      modules: [ { module: "sale_management", state: "installed" },
                 { module: "project",         state: "installed" } ] }

system_list_modules { ...five args..., state: "installed" }     # what else came in
```

**Pass every module in one call.** Odoo resolves the combined dependency tree in a single
transaction: faster (one registry rebuild instead of N) and safer — a loop can leave the
database half-installed when the third module fails.

Two behaviours worth expecting:

- **Names are resolved before anything is pressed.** A typo returns
  `e_module_not_found` with a `missing` list and **nothing changed** — the whole call
  aborts rather than installing the good ones.
- **Dependencies come in too.** The answer reports the modules you named; the second
  `system_list_modules` shows the rest. Installing `stock` also brought `sale_stock` in the
  session that produced this skill.

---

## 2. Upgrade modules

```text
system_list_modules { ...five args..., state: "installed" }     # confirm they ARE installed
admin_upgrade_module { ...five args..., module_names: ["sale", "project"] }
```

This runs **migration scripts against real customer data**. Upgrading something that is not
installed fails, which is why the check comes first.

Same list semantics: one call, one transaction, one registry rebuild.

---

## 3. Uninstall — say what it destroys, first

**Read this before calling, not after.**

`admin_uninstall_module` **drops the modules' tables and their data**. Uninstalling
`account` deletes journal entries, journals and taxes. There is no undo short of restoring
a backup.

And it **removes every module that depends on the ones you name** — `stock` takes
`sale_stock` with it — while the answer lists **only what you asked for**. The damage is
routinely wider than the call.

The sequence:

```text
# 1. See what is installed, and reason about what depends on the target
system_list_modules { ...five args..., state: "installed" }

# 2. SAY IT OUT LOUD and get an explicit yes
#    "Desinstalar account borra los asientos, diarios e impuestos de este cliente,
#     y se lleva por delante sale y sale_management. Es irreversible. ¿Confirmas?"

# 3. Only then
admin_uninstall_module { ...five args..., module_names: ["sale_management"] }

# 4. Read back the REAL extent
system_list_modules { ...five args..., state: "installed" }
```

Never call it speculatively, never "to clean up", and never on a production environment
without a named person confirming.

---

## 4. A module that "does not exist"

`e_module_not_found` for something you can see in the repository usually means the target
has not refreshed its module list:

```text
admin_update_module_list { ...five args... }
system_list_modules      { ...five args..., state: "uninstalled" }
admin_install_module     { ...five args..., module_names: ["..."] }
```

If it is still missing after that, the module is not in that environment's addons path —
its code was never deployed there. That is a deployment question for the technical team,
not something the MCP can fix.

---

## 5. It timed out — now what

**Do not retry.** A timeout does not cancel anything; the install is very likely still
running or already finished.

```text
system_list_modules { ...five args..., state: "installed" }
#   or, for the transient states:
orm_search_read { ...five args..., model: "ir.module.module",
                  domain: [["name","in",["sale_management","project"]]],
                  fields: ["name","state","latest_version"] }
```

Read `state`:

| State | Meaning |
| --- | --- |
| `installed` | It finished. Nothing to do — the timeout was only the client giving up |
| `to install` | **Still in progress.** Wait and read again; do not launch a second install |
| `uninstalled` | It really did not happen. Safe to retry |

In the session that produced this skill, `sale_management` timed out at the client twice
and **installed correctly both times**. Retrying blind would have started the same work
again.

If the wait is genuinely too short for a legitimate operation, the fix is the *Module Admin
Timeout* on the MCP server record — or, more often, the **client-side** timeout on the
user's machine (`../README.md` → the six timeouts). Both are technical-team territory.
