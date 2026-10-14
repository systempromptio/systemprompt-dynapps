<!-- verified: v18.0 @ 2026-06 -->
# Module System — Registry, Loading & Migration (Odoo 18.0)

Source: `$ODOO_SRC/odoo18/odoo/modules/` (8 files)

## Registry

The Registry is a per-database mapping of model names to model classes. One Registry per database.

```python
from odoo.modules.registry import Registry

registry = Registry(db_name)       # get or create registry for database
registry['res.partner']            # → model class (not recordset)
'res.partner' in registry          # True if model exists
```

In practice, you access models through `env`:
```python
self.env['res.partner']            # returns empty recordset, goes through registry
```

### Registry lifecycle

1. **First request to DB** → registry is built: all installed modules are loaded, models are constructed
2. **Module install/upgrade** → registry is rebuilt: `registry.signal_changes()`
3. **Server restart** → registries are rebuilt from scratch

### Registry methods (rarely used directly)

```python
registry.clear_cache()             # clear all ormcache entries
registry.signal_changes()          # notify workers to rebuild
registry.in_test_mode()            # True during test execution
```

---

## Module Manifest (`__manifest__.py`)

Every module has a `__manifest__.py` defining metadata:

```python
{
    'name': 'Module Display Name',
    'version': '18.0.1.0.0',           # {odoo_version}.{major}.{minor}.{patch}
    'category': 'Sales',
    'summary': 'One-line description',
    'description': '...',               # RST or plain text
    'author': 'dynapps',
    'website': 'https://www.dynapps.es/',
    'license': 'LGPL-3',               # required since v15
    'depends': ['sale', 'stock'],       # module dependencies
    'data': [                           # data files loaded on install/upgrade
        'security/ir.model.access.csv',
        'views/my_views.xml',
        'data/my_data.xml',
    ],
    'demo': ['demo/demo_data.xml'],     # loaded only with demo data
    'assets': {                         # JS/CSS/XML assets
        'web.assets_backend': [
            'module/static/src/**/*.js',
            'module/static/src/**/*.xml',
        ],
    },
    'installable': True,
    'auto_install': False,              # True = install automatically when all depends are installed
    'application': False,               # True = appears in Apps menu
    'external_dependencies': {
        'python': ['requests', 'lxml'],
        'bin': ['wkhtmltopdf'],
    },
    'pre_init_hook': 'pre_init_hook',   # function called before module tables are created
    'post_init_hook': 'post_init_hook', # function called after install
    'uninstall_hook': 'uninstall_hook', # function called on uninstall
    'post_load': 'post_load',          # function called when module code is loaded
}
```

### Module introspection

```python
from odoo.modules.module import get_module_path, get_manifest
from odoo.tools.misc import file_path

get_module_path('sale')                          # → absolute path to the sale module dir
get_manifest('sale')                             # → manifest dict (defaults applied)

# get_resource_path is DEPRECATED since 17.0 (emits DeprecationWarning).
# Use tools.misc.file_path instead:
file_path('sale/views/order.xml')                # → absolute path, validates it exists
```

Notes (verified against `odoo/modules/module.py`):
- `get_resource_path(module, *args)` still exists but warns "Since 17.0: use tools.misc.file_path instead". `get_module_resource` / `check_resource_path` are aliases of it.
- `file_path(file_path, filter_ext=('',), env=None)` lives in `odoo.tools.misc`; it resolves within the addons paths and raises `FileNotFoundError`/`ValueError` if missing or out of bounds.

---

## Module Loading Order

1. `base` module always loads first
2. Dependencies are loaded in topological order (dependency graph)
3. Within a module, data files load in manifest `data` list order
4. `demo` files load after `data` (only if demo mode)

---

## Migration Scripts

When upgrading a module, Odoo runs migration scripts for each version bump.

### Directory structure

```
my_module/
├── migrations/
│   ├── 18.0.1.1.0/
│   │   ├── pre-migrate.py    # stage matched by "pre-" PREFIX, any filename works
│   │   ├── post-fixup.py     # stage matched by "post-" prefix
│   │   └── end-cleanup.py    # stage matched by "end-" prefix
│   ├── 0.0.0/                # special: runs on ANY version change (pre-first, end-last)
│   │   └── post-always.py
│   └── 18.0.1.2.0/
│       └── post-migrate.py
```

**Stage is matched by filename prefix, not by an exact name.** The runner
(`migration.py: _get_migration_files`) selects files where
`os.path.basename(f).startswith("pre-"|"post-"|"end-")`. So `pre-anything.py`,
`post-anything.py`, `end-anything.py` are all valid; multiple files per stage run
sorted by basename. The folder name may be a module version (`1.1.0`) or a full
`server.module` version (`18.0.1.1.0`) — major-less versions are not re-run on an
Odoo-version bump. The `0.0.0` folder always runs (pre stage first, post/end stage last).

### Execution order

1. **pre-migrate**: ORM tables are still in OLD schema. Use for: renaming columns/tables, copying data before schema change
2. **Module update**: ORM applies new model definitions (add/remove columns, constraints)
3. **post-migrate**: ORM tables are in NEW schema. Use for: data transformation, cleanup, recomputation
4. **end-migrate**: ALL modules are updated. Use for: cross-module fixups

### Script template

```python
# migrations/18.0.1.1.0/pre-migrate.py
# Source requires the function be named `migrate`; accepted param names are
# (cr|_cr, version|_version) — see VALID_MIGRATE_PARAMS in migration.py.

def migrate(cr, version):
    """Pre-migration: rename old column before ORM drops it.

    `version` is the previously installed version string ('' on fresh install).
    """
    if not version:
        return
    # Use raw SQL — the new ORM schema is not applied yet in pre-migrate
    cr.execute("ALTER TABLE my_table RENAME COLUMN old_name TO new_name")
```

```python
# migrations/18.0.1.1.0/post-migrate.py

from odoo import api, SUPERUSER_ID

def migrate(cr, version):
    """Post-migration: recompute display names."""
    env = api.Environment(cr, SUPERUSER_ID, {})
    records = env['my.model'].search([])
    records._compute_display_name()
```

### Key rules

- `cr` is a raw cursor; the new ORM schema is not yet applied in pre-migrate
- `env` is available in post/end (construct it yourself: `api.Environment(cr, SUPERUSER_ID, {})`)
- A folder version runs when `installed_version < folder_version <= manifest_version` — it does NOT need to equal the manifest version. Put migration steps in the version that introduces them.
- Scripts run in ascending version order; within a version/stage, by basename
- Idempotency: check before acting (`column_exists`, `table_exists` from `odoo.tools.sql`)
