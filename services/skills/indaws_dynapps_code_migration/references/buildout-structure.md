# Buildout Structure, Stub Modules & Migration Scripts

## Buildout directory layout

```
$ODOO_SRC/buildouts/<customer><version>/
├── parts/
│   ├── custom/                  ← custom modules (one dir per module)
│   │   ├── xx_base/             ← base module: depends on all others
│   │   ├── xx_all/              ← install-all module
│   │   └── <module>/
│   ├── oca_pull/                ← OCA modules (checked out by dyn-tools)
│   └── modules_to_remove/       ← stub modules for forced uninstallation
├── dyn-build.yaml               ← dyn-tools config (repos, addons-path)
├── odoo.cfg                     ← Odoo instance config
└── .pre-commit-config.yaml
```

Key modules:
- `xx_base` — base module of the customer; all others depend on it;
  contains `end-migrate.py` migration script.
- `xx_all` — meta-module that installs all others;
  contains `end-reactivate_views.py` migration script.

## `.gitignore` for parts/

```gitignore
/parts/*
!/parts/custom/
!/parts/oca_pull/
!/parts/modules_to_remove/
```

## `dyn-build.yaml` — adding modules_to_remove

```yaml
repos:
  - repo: parts/modules_to_remove
    source: local
```

## Stub Modules (modules_to_remove)

For every module that must be uninstalled in the new version,
create `parts/modules_to_remove/<module_name>/` with two files:

### `__manifest__.py`

```python
{
    "name": "<Module Name> (REMOVED)",
    "version": "18.0.1.0.0",
    "author": "dynapps",
    "license": "LGPL-3",
    "depends": ["base"],
}
```

Adjust `depends` to list any modules that must still be installed
before this stub can be processed (usually just `["base"]`).

### `__init__.py`

Leave empty (zero bytes).

---

## Migration Scripts

### `end-migrate.py` — in `xx_base`

Path: `parts/custom/xx_base/migrations/<version>/end-migrate.py`

Uninstalls all modules found in `parts/modules_to_remove/` at the end of the upgrade.

```python
import logging, os
from odoo import SUPERUSER_ID, api
from odoo.addons import __path__ as addons_paths

_logger = logging.getLogger(__name__)

def migrate(cr, version):
    env = api.Environment(cr, SUPERUSER_ID, {})
    path = next(p for p in addons_paths if "modules_to_remove" in p)
    for module_name in os.listdir(path):
        _logger.info("Removing module %s", module_name)
        module = env["ir.module.module"].search([("name", "=", module_name)], limit=1)
        if module:
            if module.state != "installed":
                continue
            try:
                with env.cr.savepoint():
                    module.button_upgrade()
                    module.button_uninstall()
            except Exception as e:
                _logger.error(f"Error uninstalling {module_name}: {e}")
        env.cr.commit()
```

---

### `end-reactivate_views.py` — in `xx_all`

Path: `parts/custom/xx_all/migrations/<version>/end-reactivate_views.py`

Reactivates views that were deactivated during upgrade but belong to installed modules.

```python
import logging
from odoo import SUPERUSER_ID, api

_logger = logging.getLogger(__name__)

def migrate(cr, version):
    env = api.Environment(cr, SUPERUSER_ID, {})
    _reactivate_views(env)

def _reactivate_views(env):
    cr = env.cr
    cr.execute("""
        SELECT v.id, v.name, d.module || '.' || d.name AS xml_id
        FROM ir_ui_view v
        JOIN ir_model_data d ON d.model = 'ir.ui.view' AND d.res_id = v.id
        JOIN ir_module_module m ON m.name = d.module
        WHERE v.active = FALSE
          AND m.state = 'installed'
          AND d.module NOT LIKE 'website%'
        ORDER BY v.id
    """)
    rows = cr.fetchall()
    if not rows:
        _logger.info("No inactive module views found to reactivate.")
        return

    reactivated, skipped = [], []
    for view_id, view_name, xml_id in rows:
        view = env["ir.ui.view"].with_context(active_test=False).browse(view_id)
        try:
            with cr.savepoint():
                view.write({"active": True})
            reactivated.append(xml_id)
        except Exception as e:
            skipped.append((xml_id, view_name, e))
            _logger.warning("Cannot reactivate %s (%s): %s", xml_id, view_name, e)

    _logger.info("Done: %d reactivated, %d left inactive.", len(reactivated), len(skipped))
```

---

## Notes

- The `migrations/` folder inside each custom module is **deleted** when migrating
  (old migration scripts are version-specific and do not carry over).
- New migration scripts (`end-migrate.py`, `end-reactivate_views.py`) are created fresh
  using the templates above with the target version number in the path.
- Version in path must match `xx_base` / `xx_all` manifest version exactly
  (e.g. `18.0.1.0.0` → folder `18.0.1.0.0`).
