# Migration Order

The order in which modules are migrated matters because a module cannot be
installed until all its `depends` are already migrated and installable.

## Overall track sequence

```
1. Odoo standard patches  (applied as patches inside xx_base, not separate modules)
2. OCA modules            (update to new-version OCA code from OCA repos)
3. Dynapps vertical modules (indaws_* and dynapps_* shared)
4. Customer custom modules  (parts/custom/, one feature branch each)
```

## Getting topological order for custom modules

```bash
indaws-migrate modules order
```

This command reads all `__manifest__.py` files in `parts/custom/` and outputs
modules sorted so that dependencies come before dependents.

Example output:
```
xx_base
dynapps_base_tools
dynapps_invoicing
dynapps_invoicing_extension
dynapps_jira_connector
xx_all
```

Rules:
- `xx_base` is always first (base of everything).
- `xx_all` is always last (installs everything).
- For modules at the same dependency depth, alphabetical order is used.

## When to migrate OCA modules

OCA modules are **not ported** — they are replaced with the new-version OCA code:

1. Find the OCA module on GitHub/GitLab for the target version branch.
2. Update `dyn-build.yaml` to point to the new OCA repo + branch.
3. Run `dyn-build` to pull the new code into `parts/oca_pull/`.
4. Verify the new OCA module installs cleanly.

If no target-version OCA port exists yet:
- Check the OCA migration board.
- If a PR is open, evaluate using the PR branch.
- If nothing exists, treat as a custom module and port manually.

## When to skip a module (obsolete)

A module should become a **stub** (see `buildout-structure.md`) instead of being migrated if:
- It is no longer needed in the new version.
- Its functionality is covered by a standard Odoo or OCA module.
- The customer explicitly decommissions it.

Mark it in `parts/modules_to_remove/` and add the uninstall logic to `end-migrate.py`.

## Handling circular dependencies

If `indaws-migrate modules order` reports a cycle:
1. Show the cycle to the user (STOP point).
2. Ask which module should be split or which dependency removed.
3. Do not break cycles without developer confirmation.

## Coordination with the DB track

The DB track (`odoo-database-upgrader` + `indaws-migration-kit`) runs upgrade scripts
against the database. The code track must align:

- **Before iterate run**: the module must be installable (`"installable": True`).
- **After iterate run**: fix Python/ORM errors reported in iterate logs.
- **`end-migrate.py` and `end-reactivate_views.py`**: run automatically at end of
  full upgrade — no manual trigger needed.

When the DB team reports iterate errors, check the per-module loop (`per-module-loop.md`)
step 7 for diagnosis guidance.
