# Dynapps Module Guidelines

**Source:** Odoo 14.0 - 19.0, Dynapps coding standards
**Last Updated:** 2026-04-28

## Overview

This document covers conventions Dynapps follows when writing Odoo modules. Two types of content live here:

1. **Dynapps-specific decisions**: naming (`dynapps_<area>_<feature>`), manifest format (license, version `XX.0.X.X.X`, name `Dynapps: X`), README/CHANGELOG format, directory structure conventions.
2. **Odoo upstream conventions** not yet covered in `odoo-version-api-reference` (e.g., import order, class structure mandatory order, XML IDs naming, translation `_()` rules). These will gradually move out of this file as `odoo-version-api-reference` grows.

For **version-specific API changes** (deprecated APIs, breaking changes between Odoo versions, performance patterns, security gates, gotchas), consult `odoo-version-api-reference` directly via subagent — that is the canonical fuente de verdad. This file no longer duplicates that content.

---

## Brand profile: standard (Dynapps) vs Indaws Core

The brand identity baked into a module depends on **which product line it belongs to**.
Pick the profile from the module's technical-name prefix:

| Profile | Prefix | When |
|---------|--------|------|
| **Standard (Dynapps)** — default | `dynapps_` | Any new module that is **not** part of Indaws Core |
| **Indaws Core (legacy)** | `indaws_` | Modules that belong to the **Indaws Core** product line |

> ⚠️ **Indaws Core will be rebranded to Dynapps too — but NOT yet.** Until that
> happens, new Core modules MUST keep the legacy Indaws identity so they stay
> consistent with the Core modules already in production. Do **not** half-rebrand a
> Core module (Indaws name + Dynapps author, etc.). When Core is rebranded this
> profile is retired and everything collapses to the Dynapps profile.

Every other convention in this document (structure, Python, XML, multi-company,
performance…) is **identical** for both profiles. Only these brand-identity fields differ:

| Field | Standard (Dynapps) — default | Indaws Core (`indaws_*`) — legacy |
|-------|------------------------------|-----------------------------------|
| Tech name prefix | `dynapps_<area>_<feature>` | `indaws_<area>_<feature>` |
| Manifest `name` | `Dynapps: Module Name` | `inDAWS: Module Name` |
| Header comment | _(none)_ | `# Part of Indaws Business Solutions S.L.` |
| Manifest `author` | `dynapps` | `Indaws Business Solutions S.L.` |
| Manifest `website` | `https://www.dynapps.es/` | `https://www.indaws.es` |
| `depends` own-section header | `# Dynapps` | `# inDAWS` |
| Icon (`static/description/icon.png`) | `references/dynapps_icon.png` (Dynapps logo) | `references/indaws_core_icon.png` (Indaws V2 "W") |
| README maintainer block | Dynapps / dynapps.es | Indaws Business Solutions S.L. / indaws.es |

The manifest, `depends` and icon sections below show the **standard (Dynapps)** form.
For a Core module, substitute the legacy values from the table above.

---

## Module Structure Principles

### Isolation and Modularity

- Isolate functionalities in separate modules or methods to maximize reusability
- **ALWAYS** program modules to be inheritable and extensible by others
- Use Mixins for common functionalities or fields used in 3+ models

### Dependencies

- Properly define dependencies between modules in `__manifest__.py`
- When inheriting from a module, it **MUST** be a mandatory dependency
- Avoid circular dependencies

### Naming Convention for Dynapps Modules

- **Mandatory prefix:** `dynapps_` (standard) · `indaws_` for Indaws Core (legacy — see "Brand profile" above)
- **Format:** `dynapps_<area>_<feature>`
- **Examples:**
  - `dynapps_sale` - Main sales module
  - `dynapps_sale_margin` - Sales margin extension
  - `dynapps_base_sale` - Base sales configuration
  - `dynapps_inventory_tracking` - Inventory tracking

---

## Directory Structure

```
dynapps_<module_name>/
├── __init__.py
├── __manifest__.py
├── README.rst
├── controllers/
│   ├── __init__.py
│   └── <module_name>.py         # Main controller (NOT main.py)
├── data/
│   ├── <model_name>_data.xml    # Initial data
│   └── <model_name>_demo.xml    # Demo data
├── models/
│   ├── __init__.py
│   ├── <main_model>.py          # Main model
│   ├── <secondary_model>.py     # Secondary models
│   └── res_partner.py           # Odoo model inheritances
├── report/
│   ├── __init__.py
│   ├── <model>_report.py        # SQL views models
│   ├── <model>_report_views.xml
│   ├── <model>_reports.xml      # Report actions, paperformat
│   └── <model>_templates.xml    # XML report templates
├── security/
│   ├── ir.model.access.csv      # Access rights (always first)
│   ├── <module>_groups.xml      # User groups
│   └── <model>_security.xml     # Record rules
├── static/
│   ├── description/
│   │   └── icon.png
│   ├── lib/                     # External JS libraries
│   └── src/
│       ├── js/
│       ├── scss/
│       └── xml/
├── views/
│   ├── <module>_menus.xml       # Main menus (optional)
│   ├── <model>_views.xml        # Backend views (form, list, kanban...)
│   └── <model>_templates.xml    # Portal/website templates (QWeb)
└── wizard/
    ├── __init__.py
    ├── <transient_model>.py
    └── <transient_model>_views.xml
```

---

## Python Coding Standards

### Import Order (CRITICAL)

**The order MUST be:**

1. Standard library imports (alphabetical)
2. External library imports (alphabetical)
3. Odoo imports (alphabetical within each group)
4. Odoo addons imports (alphabetical)
5. Local relative imports

```python
# ✅ CORRECT
# 1. Standard library
import base64
import logging
import os
from datetime import datetime
from operator import itemgetter

# 2. External libraries
import psycopg2
from dateutil.relativedelta import relativedelta

# 3. Odoo imports (alphabetical after odoo)
import odoo
# In Odoo 18: include `_` for translations:
#   from odoo import _, api, Command, fields, models
# In Odoo 19+: drop `_` from the import — use `self.env._()` instead
#   (see odoo-version-api-reference/diffs/v19-diff.md "self.env._() for translations").
from odoo import _, api, Command, fields, models
from odoo.exceptions import UserError, ValidationError
from odoo.http import request
from odoo.osv.expression import AND, OR
from odoo.tools import safe_eval

# 4. Odoo addons
from odoo.addons.base.models.res_partner import Partner
from odoo.addons.sale.models.sale_order import SaleOrder

# 5. Local imports
from .utils import helper_function
```

### Odoo Class Structure (Mandatory Order)

```python
class SaleOrder(models.Model):
    # 1. Private attributes
    _name = 'sale.order'
    _description = 'Sale Order'
    _inherit = ['sale.order', 'mail.thread']
    _order = 'date_order desc, id desc'
    # v14-v18: tuple in list
    _sql_constraints = [
        ('name_unique', 'unique(name, company_id)', 'Order name must be unique!')
    ]
    # v19+: class attribute with _ prefix, using models.Constraint descriptor
    # WARNING: in v19 the _sql_constraints tuple list is IGNORED entirely
    # (only a WARNING is logged); constraints are NOT applied to the database
    # (silent data integrity risk). Use models.Constraint instead.
    _name_unique = models.Constraint(
        'unique(name, company_id)',
        'Order name must be unique!',
    )

    # 2. Default methods
    @api.model
    def _default_warehouse(self):
        """Get default warehouse for the current user."""
        return self.env.user._get_default_warehouse_id()

    # 3. Field declarations (grouped by type)
    # -- Basic fields
    name = fields.Char(string='Order Reference', required=True)
    date_order = fields.Datetime(string='Order Date', default=fields.Datetime.now)

    # -- Relational fields (Many2one with _id suffix)
    partner_id = fields.Many2one('res.partner', string='Customer', required=True)
    warehouse_id = fields.Many2one('stock.warehouse', default=_default_warehouse)

    # -- One2many/Many2many fields (_ids suffix)
    order_line_ids = fields.One2many('sale.order.line', 'order_id', string='Order Lines')
    tag_ids = fields.Many2many('sale.order.tag', string='Tags')

    # -- Computed fields
    amount_total = fields.Monetary(compute='_compute_amounts', store=True)

    # 4. Compute, inverse, search methods
    @api.depends('order_line_ids.price_subtotal')
    def _compute_amounts(self):
        """Compute the total amounts of the SO."""
        for order in self:
            order.amount_total = sum(order.order_line_ids.mapped('price_subtotal'))

    # 5. Selection methods
    @api.model
    def _selection_state(self):
        """Return available states for the order."""
        return [('draft', 'Draft'), ('done', 'Done')]

    # 6. Constrains methods
    @api.constrains('date_order')
    def _check_date_order(self):
        """Check that order date is not in the future."""
        for order in self:
            if order.date_order > fields.Datetime.now():
                raise ValidationError(_('Order date cannot be in the future.'))

    # 7. Onchange methods
    @api.onchange('partner_id')
    def _onchange_partner_id(self):
        """Update payment terms when partner changes."""
        if self.partner_id:
            self.payment_term_id = self.partner_id.property_payment_term_id

    # 8. CRUD methods
    @api.model_create_multi
    def create(self, vals_list):
        """Override create to set sequence."""
        for vals in vals_list:
            if not vals.get('name'):
                vals['name'] = self.env['ir.sequence'].next_by_code('sale.order')
        return super().create(vals_list)

    def write(self, vals):
        """Override write to track changes."""
        res = super().write(vals)
        if 'state' in vals:
            self._track_state_change(vals['state'])
        return res

    def unlink(self):
        """Prevent deletion of confirmed orders."""
        if any(order.state == 'sale' for order in self):
            raise UserError(_('Cannot delete confirmed orders.'))
        return super().unlink()

    # 9. Action methods (action_ prefix)
    def action_confirm(self):
        """Confirm the sale order."""
        self.ensure_one()
        self.write({'state': 'sale'})
        return True

    # 10. Business methods
    def _prepare_invoice(self):
        """Prepare invoice values from sale order."""
        self.ensure_one()
        return {
            'partner_id': self.partner_id.id,
            'origin': self.name,
        }
```

### Naming Conventions

#### Models

```python
# Main model (singular)
_name = 'sale.order'  # ✅ Correct
_name = 'sale.orders'  # ❌ Incorrect (no plural)

# Transient/Wizard
_name = 'sale.order.make.invoice'  # ✅ <base_model>.<action>

# Report
_name = 'sale.report'  # ✅ <base_model>.report
```

#### Fields

```python
# Many2one → _id suffix
partner_id = fields.Many2one('res.partner')
order_id = fields.Many2one('sale.order')

# One2many/Many2many → _ids suffix
line_ids = fields.One2many('sale.order.line', 'order_id')
tag_ids = fields.Many2many('sale.tag')

# Boolean → is_, has_, can_
is_confirmed = fields.Boolean()
has_lines = fields.Boolean()
can_edit = fields.Boolean()

# Date/Datetime → _date, _datetime
order_date = fields.Date()
confirmation_datetime = fields.Datetime()
```

#### Methods

```python
# Compute → _compute_<field_name>
def _compute_amount_total(self):

# Search → _search_<field_name>
def _search_partner_name(self, operator, value):

# Default → _default_<field_name>
def _default_company_id(self):

# Selection → _selection_<field_name>
def _selection_state(self):

# Onchange → _onchange_<field_name>
def _onchange_partner_id(self):

# Constrains → _check_<constraint_name>
def _check_date_order(self):

# Action → action_<description>
def action_confirm(self):
def action_send_email(self):

# Private/Business → _<description>
def _prepare_invoice_values(self):
def _get_default_warehouse(self):
```

---

## Version-Specific API Changes

For Odoo 19, Odoo 18+, deprecated APIs, breaking changes, and migration guidance, consult `odoo-version-api-reference` (in `ai-claude-common`) via subagent. That skill is the canonical source of all version diffs and is kept up to date with each Odoo release. Do **not** duplicate that content here.

Key categories the api-reference covers:

- v19 breaking changes (`@api.returns` removed, `res.groups.privilege`, `Domain` class, `models.Constraint`, `_cr`/`_context`/`_uid` removed, etc.).
- v19 additions (`@api.private`, `formatted_read_group`, `_commit_progress`, `self.env._()` for translations).
- v18+ changes (`_compute_display_name` over `name_get()`, `aggregator` over `group_operator`, `_read_group` new signature, etc.).
- Performance patterns (precompute, indexes, N+1 prevention, batch operations).
- A comprehensive set of gotchas catalogued (real errors Claude makes when generating code).
- Security gates (12 gates, BLOCKING + WARNING tiers).

**Invocation pattern**: from any module-creator subagent step, use the standard "Invoke skill 'odoo-version-api-reference'. For v{version}, return: ..." pattern. The result (`API_RESULT`) is what informs the code generation step.

---

## Python Best Practices

### Dictionary and List Creation

```python
# ❌ INCORRECT - Don't use .clone()
new_dict = my_dict.clone()
new_list = old_list.clone()

# ✅ CORRECT
new_dict = dict(my_dict)
new_list = list(old_list)

# ✅ CORRECT - Create with values
my_dict = {'foo': 3, 'bar': 4}

# ✅ Update dict with multiple values
my_dict.update(foo=3, bar=4, baz=5)
```

### Translation _() - Critical Rules

```python
_ = self.env._

# ✅ CORRECT - Literal strings with named parameters
error = _('This record is locked!')
error = _('Record %s cannot be modified!', record)
error = _('Record %(name)s cannot be modified', name=record.name)

# ❌ INCORRECT - Formatting before translation (DOESN'T WORK)
error = _('Record %s cannot be modified!' % record)

# ❌ INCORRECT - Dynamic concatenation (DOESN'T WORK)
error = _("'" + question + "' \n")
```

### Extensibility (Think Extendable)

```python
# ❌ INCORRECT - Hardcoded logic that can't be extended
def action(self):
    partners = self.env['res.partner'].search(complex_domain)
    emails = partners.filtered(lambda r: arbitrary_criteria).mapped('email')

# ✅ CORRECT - Small and extensible methods
def action(self):
    partners = self.env['res.partner'].search(self._get_partner_domain())
    emails = partners.filtered(lambda r: r._filter_partners()).mapped('email')

def _get_partner_domain(self):
    """Return domain to filter partners. Override to customize."""
    return [('is_company', '=', True)]

def _filter_partners(self):
    """Return True if partner should be included. Override to customize."""
    self.ensure_one()
    return bool(self.email)
```

### Absolute Prohibitions

```python
# ❌ NEVER do cr.commit() manually
# The framework manages transactions automatically.

# ❌ NEVER use direct SQL if there's an ORM alternative

# ❌ NEVER create custom decorators (use Odoo's ones)
```

---

## XML/View Guidelines

### XML IDs Naming

#### Views

```xml
<record id="<model_name>_view_<view_type>" model="ir.ui.view">
<!-- Examples -->
<record id="sale_order_view_form" model="ir.ui.view">
<record id="sale_order_view_list" model="ir.ui.view">
<record id="sale_order_view_kanban" model="ir.ui.view">
<record id="sale_order_view_search" model="ir.ui.view">
```

#### View Inheritances

```xml
<!-- USE the SAME ID as the original (the module prefix makes it unique) -->
<!-- name must contain .inherit.<module> -->
<record id="model_view_form" model="ir.ui.view">
    <field name="name">model.view.form.inherit.module2</field>
    <field name="inherit_id" ref="module1.model_view_form"/>
    ...
</record>
```

#### Actions

```xml
<!-- Main action -->
<record id="<model_name>_action" model="ir.actions.act_window">
    <field name="name">Model Main Action</field>

<!-- Secondary actions with descriptive suffix -->
<record id="<model_name>_action_<detail>" model="ir.actions.act_window">
```

#### Security

```xml
<!-- Groups: <module>_group_<group_name> -->
<record id="dynapps_sale_group_user" model="res.groups">

<!-- Rules: <model>_rule_<concerned_group> -->
<record id="sale_order_rule_user" model="ir.rule">
```

### Shortcut Tags (Preferred)

```xml
<!-- ✅ PREFERRED - Use <menuitem> instead of <record> -->
<menuitem
    id="model_name_menu_root"
    name="Main Menu"
    sequence="5"
/>

<!-- ✅ PREFERRED - Use <template> for QWeb views -->
<template id="portal_my_home" inherit_id="portal.portal_my_home">
    <xpath expr="..." position="inside">
        ...
    </xpath>
</template>
```

---

## __manifest__.py Format

> **Pick the variant by prefix:** `dynapps_*` → the **standard (Dynapps)** manifest below.
> `indaws_*` → the **Indaws Core (legacy)** manifest right after it. Copy the matching one
> whole — don't mentally diff the standard one.

> **NO lleva `description`.** `pylint_odoo` la marca como `C8103 manifest-deprecated-key` y sale
> con **código 16**: en un repo cliente con `pre-commit` obligatorio, esa clave tumba la rama en
> el primer push. La descripción larga va en el `README.md` del módulo, que es lo que Odoo 19
> renderiza en Apps. *(Cicatriz real, 2026-08-13, addons-muebles-romero: esta plantilla la incluía,
> cuatro módulos la copiaron y el gate del repo los rechazó a los cuatro.)*

```python
{
    'name': 'Dynapps: Module Name',
    'summary': 'Short description',
    'category': 'Sales',
    'version': '18.0.1.0.0',  # <odoo_version>.<major>.<minor>.<patch>.<build>
    'depends': [
        # Odoo
        'sale',
        'stock',
        # OCA
        'queue_job',                 # OCA/queue
        'account_payment_order',     # OCA/bank-payment
        # Dynapps
        'dynapps_base',
        'dynapps_base_sale',
    ],
    'external_dependencies': {
        'python': ['numpy', 'requests'],
    },
    'author': 'dynapps',
    'website': 'https://www.dynapps.es/',
    'license': 'LGPL-3',
    'data': [
        # Security (first)
        'security/ir.model.access.csv',
        'security/dynapps_sale_groups.xml',
        # Data
        'data/ir_sequence.xml',
        # Views
        'views/sale_order_views.xml',
        'views/menus.xml',
        # Reports
        'report/sale_report.xml',
    ],
    'demo': [
        'demo/sale_order_demo.xml',
    ],
    'assets': {
        'web.assets_backend': [
            'dynapps_sale/static/src/**/*.js',
            'dynapps_sale/static/src/**/*.xml',
            'dynapps_sale/static/src/**/*.scss',
        ],
    },
    'installable': True,
    'auto_install': False,
    'application': False,
}
```

### Indaws Core variant (legacy profile — `indaws_*`)

**Only for Indaws Core modules.** Identical structure to the standard manifest, with the
Indaws identity restored. Copy this verbatim instead (the elided `# ...` parts are the same
as the standard manifest above — `summary`, `category`, `version`,
`external_dependencies`, `data`, `demo`, `assets`, install flags):

```python
# Part of Indaws Business Solutions S.L.
{
    'name': 'inDAWS: Module Name',
    # ... (summary, category, version — same as standard; tampoco lleva 'description')
    'depends': [
        # Odoo
        'sale',
        # OCA
        'queue_job',
        # inDAWS                       ← own-modules header stays "# inDAWS" for Core
        'indaws_base',
        'indaws_base_sale',
    ],
    # ... (external_dependencies — same as standard)
    'author': 'Indaws Business Solutions S.L.',
    'website': 'https://www.indaws.es',
    'license': 'LGPL-3',
    # ... (data, demo, assets — same as standard; assets paths use indaws_<module>/)
    'installable': True,
    'auto_install': False,
    'application': False,
}
```

> **Icon for Core:** copy `references/indaws_core_icon.png` (the Indaws V2 "W"), **not** `dynapps_icon.png`.

### `depends` section convention

Three explicit comment sections, **always in this order**:

| Section | Modules | Examples |
|---------|---------|----------|
| `# Odoo` | Odoo Community + Enterprise (under `odoo/addons/` or `enterprise/`) | `sale`, `stock`, `account`, `helpdesk` |
| `# OCA` | OCA modules (under `OCA/<repo>/<module>/` in addons_path) | `queue_job`, `account_payment_order`, `delivery_state`, `web_no_bubble` |
| `# Dynapps` | Dynapps first-party modules (`dynapps_*`) | `dynapps_base`, `dynapps_base_sale`, `vt_energy_base` |
| `# inDAWS` | Indaws Core modules (`indaws_*`, legacy — until Core rebrand) | `indaws_base`, `indaws_account` |

**Rules:**

- Only emit a section header if it has at least one entry.
- Within each section, order alphabetically.
- The headers are **case-sensitive**: write `# Dynapps` and `# inDAWS` exactly. Emit `# inDAWS` only for legacy Core deps (`indaws_*`); everything new and first-party goes under `# Dynapps`.
- If a dep is uncertain, check where it lives in the addons_path (`<ODOO_SRC>/<version>/{odoo,enterprise,OCA}/...`) before classifying.

This convention exists because OCA modules used to get filed under `# Odoo` by mistake (e.g. `queue_job`, `account_payment_order`) — explicit `# OCA` makes the boundary visible.

---

## Module Icon (`static/description/icon.png`)

Every module **MUST** ship the canonical icon for its brand profile (see "Brand profile" above), copied during `STEP 2`:

- **Standard (Dynapps):** `references/dynapps_icon.png` — Dynapps logo, 140×140 PNG, MD5 `9d388265d5bf4d1a6b443e2e6f249dc5`
- **Indaws Core (`indaws_*`, legacy):** `references/indaws_core_icon.png` — Indaws V2 "W", 151×157 PNG, MD5 `5ee054778ed999adc23f051b20a74af9`

If a module has a legitimate **domain-specific icon** (e.g. `indaws_data_import` uses a wifi/wave icon, `indaws_rappel` uses a bar-chart icon), keep it — those are deliberate product decisions. Document the deviation in the module's README.

---

## User Experience (UX) Guidelines

### Configurability

- Prefer configuration over hardcoding (texts, images, values)
- Use System Parameters for configurable global values
- Allow customization in Settings when it makes sense

### Fields and Help

```python
# ✅ Use help on fields for contextual assistance
field_name = fields.Char(
    string='Field Label',
    help='Explanation of what this field does and how to use it'
)

# ✅ Descriptive and useful error messages
raise UserError(_(
    'Cannot confirm order %(order)s: no lines found.\n'
    'Please add at least one product before confirming.',
    order=self.name
))
```

---

## Code Quality Tooling

Linting and formatting are configured **at the repo level** by the skeleton (`copier`-rendered files: `.pre-commit-config.yaml`, `.ruff.toml`, `.pylintrc`, `.pylintrc-mandatory`, `eslint.config.cjs`, `prettier.config.cjs`). The tools run automatically via pre-commit hooks and the `pre-commit-ci.yml` GitHub Action.

The skill **does not duplicate the rules here**. To know the rules in effect:
- Read the actual config files in the target repo (`.ruff.toml`, etc.).
- For interpretation of pylint_odoo rules and patterns to avoid, consult `odoo-version-api-reference` (`gotchas.md`, `secure-coding-*.md`).

Stack hooks active in the canonical skeleton (April 2026):
- `ruff` + `ruff-format` (Python lint + format, replaces black/isort/flake8)
- `prettier` + plugin-xml (formatting for JS/XML/MD/YAML)
- `eslint` + `eslint-plugin-jsdoc` (JS lint)
- `pylint_odoo` (2 passes: optional + mandatory)
- OCA helpers (`oca-checks-odoo-module`, `oca-checks-po`, manifest fixers)
- pre-commit-hooks (trailing whitespace, end-of-file-fixer, debug-statements, etc.)

---

## Performance Best Practices

For performance patterns (N+1 prevention, batch operations, prefetch, `_read_group` over `len()`, set-based membership tests, store=True/False decisions, indexes, precompute) consult `odoo-version-api-reference/engine-patterns.md` directly via subagent. That file is the canonical source.

---

## References

- **Odoo Official Documentation:** https://www.odoo.com/documentation/19.0/developer.html
- **ORM Changelog:** https://www.odoo.com/documentation/19.0/developer/reference/backend/orm/changelog.html
- **JSON-2 API:** https://www.odoo.com/documentation/19.0/developer/reference/external_api.html
- **Coding Guidelines:** https://www.odoo.com/documentation/19.0/contributing/development/coding_guidelines.html
- **PEP 8:** https://peps.python.org/pep-0008/
- **PEP 257:** https://peps.python.org/pep-0257/

---

## Coding language and locale rules

### Idioma del código: inglés SIEMPRE

Todo el código fuente (identificadores, nombres técnicos de modelos/campos, comentarios, docstrings, external IDs, mensajes de commit) en **inglés**, sea cual sea el cliente o su idioma. Solo los strings de cara al usuario (`string=`, labels, menús, informes, help) se traducen vía i18n. Ver `references/shared/coding-standards.md`.

### Ortografía es_ES obligatoria

Todo contenido en español DEBE usar acentos, eñes y diéresis (á, é, í, ó, ú, ñ, ü). NUNCA sin tildes. (Aplica solo a los strings de cara al usuario; el código va en inglés — ver arriba.)
