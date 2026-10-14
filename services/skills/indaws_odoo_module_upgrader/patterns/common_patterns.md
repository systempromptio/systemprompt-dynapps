# Common Odoo Upgrade Patterns

## XML/View Patterns

### 1. Tree to List Tag Conversion
**Versions**: **SOLO al migrar a 18.0 o superior.** Decía "Odoo 15+" hasta 2026-08-07 y era falso:
`ir.ui.view.type` es `('tree','Tree')` en v15/v16/**v17** (`odoo17/odoo/addons/base/models/ir_ui_view.py:163`, `View.type`)
y pasa a `('list','List')` en v18 (`odoo18/…:153`). Aplicar esta conversión al migrar 16→17 **rompe
las vistas** (tipo inexistente).
**Detection**: `<tree\s+` — y comprobar la versión DESTINO antes de tocar nada.
**Fix**: Replace `<tree` with `<list` and `</tree>` with `</list>`

### 2. Remove Edit Attribute
**Versions**: Odoo 15+
**Detection**: `edit="1"`
**Fix**: Remove the attribute entirely, keep `editable="top"` or `editable="bottom"`

### 3. Search View Group Tags
**Versions**: Odoo 19
**Detection**: `<search>.*?<group.*?</group>.*?</search>`
**Fix**: Remove `<group>` and `</group>` tags, keep filters at root level

### 4. Active ID in Context
**Versions**: Odoo 19
**Detection**: `context=".*?active_id.*?"`
**Fix**: Replace `active_id` with `id`

### 5. Kanban Template Names
**Versions**: Odoo 19
**Detection**: `t-name="kanban-box"`
**Fix**: Replace with `t-name="card"`

### 6. Kanban JS Class
**Versions**: Odoo 19
**Detection**: `js_class="\w+"`
**Fix**: Remove the entire attribute

## Python Patterns

### 1. Import Changes
```python
# Pattern: from odoo.addons.http_routing.models.ir_http import slug
# Fix: Create compatibility wrapper

def slug(value):
    return request.env['ir.http']._slug(value)
```

### 2. API Decorator Updates
```python
# Pattern: @api.one (deprecated)
# Fix: Replace with @api.multi or remove
```

### 3. Field Changes
```python
# Pattern: fields.Float(digits_compute=)
# Fix: fields.Float(digits=)
```

## JavaScript Patterns

### 1. Module Annotation
**Detection**: Missing `/** @odoo-module **/`
**Fix**: Add at the top of every JS file

### 2. Import Path Changes
```javascript
// Pattern: import {Component} from "@web/core/component";
// Fix: import {Component} from "@odoo/owl";
```

### 3. Service Registration
```javascript
// Pattern: core.serviceRegistry.add()
// Fix: registry.category("services").add()
```

## Manifest Patterns

### 1. Version Format
```python
# Pattern: 'version': '1.0'
# Fix: 'version': '19.0.1.0.0'
```

### 2. Missing License
```python
# Pattern: No 'license' key
# Fix: Add 'license': 'LGPL-3'
```

### 3. External Dependencies
```python
# Pattern: ImportError for external packages
# Fix: Add 'external_dependencies': {'python': ['package_name']}
```

## SCSS/CSS Patterns

### 1. Variable Naming
```scss
// Pattern: $headings-font-weight
// Fix: $o-theme-headings-font-weight
```

### 2. Unit Conversion
```scss
// Pattern: 'btn-padding-y': 16px
// Fix: 'btn-padding-y': 1rem
```

### 3. Font Configuration
```scss
// Pattern: Direct font assignment
// Fix: Use map-merge with $o-theme-font-configs
```

## Python Patterns

### Custom override of a core ORM method copied verbatim from the old version
**Versions**: any cross-major migration | **Severity**: CRITICAL (silent runtime errors)

A custom module overrides an internal core method (`_update_reserved_quantity`, `_gather`, `_compute_*`, `create`, `_action_done`…) with a body **copied verbatim from the source version's core**. After migrating, core changed the method's role, signature, or call-sites — the stale override now conflicts with the new engine and raises spurious errors or corrupts behavior.

Real case: `stock_picking_extension._update_reserved_quantity` kept the v14/v15 core body (with `raise UserError("It is not possible to reserve more products …")`). In v18 that quant method is a thin wrapper without any check; the override re-introduced the raise and blocked legitimate reservations (lots in a parent location).

**Detection heuristic — empty diff against the OLD core**:
```bash
# If the module's override is byte-identical to the source-version core method,
# it's an unmigrated relic. Compare the custom file with the same method in the
# ORIGIN version's core (e.g. $ODOO_SRC/odoo15/addons/<module>/models/<file>.py).
diff <custom_module>/models/stock_quant.py $ODOO_SRC/odoo15/addons/stock/models/stock_quant.py
# empty/near-empty for the overridden method  =>  copied verbatim, not migrated
```

**Fix**: remove the override (let the new core handle it) or rewrite it to the new version's semantics/signature. Keep only the genuinely-custom additions (fields, extra logic).

**General rule**: after a migration, audit custom modules that reimplement core internals. A custom method whose diff with the origin-version core is empty is a red flag — it was never adapted to the new engine.

## Detection Commands

### Find all patterns in project:
```bash
# Find tree views
grep -r "<tree" --include="*.xml"

# Find active_id usage
grep -r "active_id" --include="*.xml"

# Find RPC service usage
grep -r "useService.*rpc" --include="*.js"

# Find missing module annotations
grep -L "@odoo-module" --include="*.js"

# Find old manifest versions
grep -r "'version':" --include="__manifest__.py"
```

## Batch Processing

### Apply all XML fixes:
```python
import re
import glob

def fix_xml_files(directory):
    for file in glob.glob(f"{directory}/**/*.xml", recursive=True):
        with open(file, 'r') as f:
            content = f.read()

        # Apply all patterns
        # SOLO si el destino es >= 18.0 (ver patrón 1): en v17 el tipo de vista sigue
        # siendo 'tree' y esta sustitución deja el módulo con un tipo inexistente.
        content = re.sub(r'<tree(\s+)', r'<list\1', content)
        content = re.sub(r'</tree>', r'</list>', content)
        content = re.sub(r'\sedit="1"', '', content)
        content = re.sub(r'active_id', 'id', content)

        with open(file, 'w') as f:
            f.write(content)
```