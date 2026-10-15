# Odoo Conventions and Best Practices

## Module Structure

### Standard Directory Layout

```
module_name/
├── __init__.py
├── __manifest__.py
├── models/
│   ├── __init__.py
│   └── *.py
├── views/
│   └── *.xml
├── security/
│   ├── ir.model.access.csv
│   └── *.xml (record rules)
├── data/
│   └── *.xml
├── demo/
│   └── *.xml
├── controllers/
│   ├── __init__.py
│   └── *.py
├── static/
│   ├── description/
│   │   ├── icon.png
│   │   └── index.html
│   └── src/
│       ├── js/
│       ├── css/
│       └── xml/
├── wizard/
│   ├── __init__.py
│   ├── *.py
│   └── *.xml
└── report/
    ├── __init__.py
    ├── *.py
    └── *.xml
```

### Detection Rules
- Flag non-standard directory names
- Check for missing `__init__.py` files
- Verify security folder exists for models
- Ensure proper separation of views/data/demo

## Naming Conventions

### Module Names

```python
# GOOD
module_name = 'sale_custom_workflow'
module_name = 'account_payment_extension'

# BAD
module_name = 'SaleCustomWorkflow'  # No camelCase
module_name = 'sale-custom-workflow'  # No hyphens
module_name = 'customsale'  # Not descriptive
```

### Model Names

```python
# GOOD - Dot notation, singular
_name = 'sale.order.custom'
_name = 'res.partner.category'

# BAD
_name = 'SaleOrder'  # No camelCase in _name
_name = 'sale_order_custom'  # Use dots, not underscores
_name = 'sale.orders'  # Use singular
```

### Field Names

```python
# GOOD - Snake_case
partner_id = fields.Many2one('res.partner')
total_amount = fields.Float()

# BAD
partnerId = fields.Many2one('res.partner')  # No camelCase
TotalAmount = fields.Float()  # No PascalCase
```

### Method Names

```python
# GOOD - Snake_case, descriptive
def _compute_total_amount(self):
def action_confirm(self):

# BAD
def computeTotalAmount(self):  # No camelCase
def confirm(self):  # Not descriptive enough for actions
```

### XML ID Conventions

```xml
<!-- GOOD - module_prefix.category_name -->
<record id="view_partner_form_custom" model="ir.ui.view">
<record id="group_sale_manager" model="res.groups">
<menuitem id="menu_custom_reports" name="Custom Reports"/>

<!-- BAD -->
<record id="partner_form" model="ir.ui.view">  <!-- Missing context -->
<record id="view1" model="ir.ui.view">  <!-- Not descriptive -->
```

### Detection Rules
- Flag module names not in snake_case
- Check model names using underscores instead of dots
- Verify field/method names use snake_case
- Ensure XML IDs are descriptive and prefixed

## Model Conventions

### "El criterio del primero": N registros, decision tomada con `records[0]`

Antipatron de recordset: un metodo que opera sobre N registros pero resuelve
**una decision global** con el primero — formato, layout, compania, diario,
plantilla — y la aplica a todos. Pasa desapercibido porque el codigo es correcto
para el caso de un solo registro, que es como se prueba.

```python
# BAD - el formato sale del primero y se aplica a los N
fmt = self.a3_format or moves[0].company_id.a3_default_format
cfg = layout._for_partner(moves[0].partner_id)

# GOOD - o se resuelve por registro, o se EXIGE homogeneidad y falla alto
if len(moves.company_id) > 1:
    raise UserError(...)
```

Que buscar: `records[0]`, `[:1]`, `[0].`, un `mapped()` cuyo resultado se
colapsa, y un `ensure_one()` que **falta** en un metodo que asume uno.
Y la senal de que es un fallo de verdad y no un atajo: **otra parte del mismo
flujo si resuelve ese dato por registro** — ahi el resultado sale incoherente
consigo mismo. *(Medido 2026-09-02 en `indawsodoo/indaws-edi#30`: el fichero de
ancho fijo llevaba el codigo de empresa por asiento y el formato del primero.)*

### Inheritance Patterns

**Extension (_inherit without _name):**
```python
# GOOD - Extending existing model
class ResPartner(models.Model):
    _inherit = 'res.partner'

    custom_field = fields.Char()
```

**Delegation (_inherits):**
```python
# GOOD - Delegation inheritance
class ProductTemplate(models.Model):
    _name = 'product.template'
    _inherits = {'product.product': 'product_id'}

    product_id = fields.Many2one('product.product', required=True, ondelete='cascade')
```

**Mixin:**
```python
# GOOD - Reusable mixin
class CustomMixin(models.AbstractModel):
    _name = 'custom.mixin'

    is_archived = fields.Boolean()
```

### Model Attributes Order

```python
class CustomModel(models.Model):
    # 1. Private attributes
    _name = 'custom.model'
    _inherit = 'mail.thread'
    _description = 'Custom Model'
    _order = 'date desc, name'
    _rec_name = 'name'

    # 2. Default methods
    def _default_user_id(self):
        return self.env.user

    # 3. Field declarations
    name = fields.Char()
    user_id = fields.Many2one('res.users', default=_default_user_id)

    # 4. Compute methods
    @api.depends('line_ids.amount')
    def _compute_total(self):
        pass

    # 5. Onchange methods
    @api.onchange('partner_id')
    def _onchange_partner_id(self):
        pass

    # 6. CRUD methods
    @api.model_create_multi
    def create(self, vals_list):
        return super().create(vals_list)

    def write(self, vals):
        return super().write(vals)

    # 7. Action methods
    def action_confirm(self):
        pass

    # 8. Business methods
    def _process_data(self):
        pass
```

### Detection Rules
- Flag incorrect attribute ordering
- Check for missing _description
- Verify proper use of _inherit vs _inherits
- Ensure Abstract models use _name not _inherit

## Field Conventions

### `digits` es de `Float`; en `Monetary` es INERTE

`Monetary` **no hereda de `Float`** (`class Monetary(Field[float])`:
`odoo18/odoo/fields.py:1704`, `Monetary`; `odoo19/odoo/orm/fields_numeric.py:184`, `Monetary`) y no
declara `digits` en ningun sitio: su propio docstring dice que *"the decimal
precision and currency symbol are taken from the `currency_field` attribute"*,
o sea que la precision sale de `res.currency.decimal_places`. Un `digits=(16,2)`
sobre un `Monetary` **no da error y no hace nada**.

Asi que **nunca lo sugieras en una auditoria ni en una review** sobre un campo
`Monetary`: es ruido y ademas transmite un modelo mental falso (que la precision
monetaria se controla en el campo). Sobre `Float` si es la recomendacion buena.
*(Corregido 2026-08-07; el ejemplo de `git-review-pr` decia "for monetary
precision" y de ahi venia la sugerencia mala.)*

### Relational Fields Naming

```python
# GOOD
partner_id = fields.Many2one('res.partner')
line_ids = fields.One2many('sale.order.line', 'order_id')
tag_ids = fields.Many2many('product.tag')

# BAD
partner = fields.Many2one('res.partner')  # Missing _id suffix
lines = fields.One2many('sale.order.line', 'order_id')  # Missing _ids suffix
```

### Selection Fields

```python
# GOOD - Using selection parameter
state = fields.Selection([
    ('draft', 'Draft'),
    ('confirmed', 'Confirmed'),
    ('done', 'Done'),
], default='draft')

# BETTER - Using selection method for reusability
state = fields.Selection(selection='_get_state_selection', default='draft')

@api.model
def _get_state_selection(self):
    return [
        ('draft', 'Draft'),
        ('confirmed', 'Confirmed'),
        ('done', 'Done'),
    ]
```

### Computed Fields

```python
# GOOD - Proper dependencies
@api.depends('line_ids.price_subtotal')
def _compute_amount_total(self):
    for record in self:
        record.amount_total = sum(record.line_ids.mapped('price_subtotal'))

# BAD - Missing dependencies
def _compute_amount_total(self):  # Missing @api.depends
    for record in self:
        record.amount_total = sum(record.line_ids.mapped('price_subtotal'))
```

### Detection Rules
- Flag Many2one fields without _id suffix
- Check One2many/Many2many without _ids suffix
- Verify @api.depends on computed fields
- Ensure selection fields have default values

## View Conventions

### Form View Structure

```xml
<record id="view_model_form" model="ir.ui.view">
    <field name="name">model.form</field>
    <field name="model">custom.model</field>
    <field name="arch" type="xml">
        <form>
            <header>
                <button name="action_confirm" type="object" string="Confirm"/>
                <field name="state" widget="statusbar"/>
            </header>
            <sheet>
                <div class="oe_title">
                    <h1><field name="name"/></h1>
                </div>
                <group>
                    <group>
                        <field name="date"/>
                    </group>
                    <group>
                        <field name="user_id"/>
                    </group>
                </group>
                <notebook>
                    <page string="Details">
                        <field name="line_ids"/>
                    </page>
                </notebook>
            </sheet>
            <!-- v18+: the tag is <chatter/>. The compiler registers selector: "chatter"
                 (odoo18/addons/mail/static/src/chatter/web/form_compiler.js:46, odoo19 :47) and
                 "oe_chatter" appears 0 times in mail/static/src/ in both. So a
                 <div class="oe_chatter"> in v18/v19 matches NO compiler and the chatter simply
                 does not render — silent, no error. In 17.0 it is the opposite: the selector IS
                 "div.oe_chatter" (odoo17/addons/mail/static/src/views/web/form/form_compiler.js:74)
                 and <chatter/> does nothing. Verified 2026-08-07. -->
            <chatter/>
        </form>
    </field>
</record>
```

### List View Conventions

```xml
<!-- v18+ uses <list>. In 17.0 it is still <tree> and <list> does NOT exist:
     ('tree','Tree') at odoo17/odoo/addons/base/models/ir_ui_view.py:163,
     ('list','List') from odoo18/…:153 (odoo19/…:149). Verified 2026-08-07.
     Auditing a 17.0 module: <tree> is CORRECT there, do not flag it. -->
<record id="view_model_list" model="ir.ui.view">
    <field name="name">model.list</field>
    <field name="model">custom.model</field>
    <field name="arch" type="xml">
        <list decoration-info="state=='draft'" decoration-success="state=='done'">
            <field name="name"/>
            <field name="date"/>
            <field name="state" invisible="1"/>
        </list>
    </field>
</record>
```

### Search View Best Practices

```xml
<search>
    <!-- Filters first -->
    <filter name="filter_draft" string="Draft" domain="[('state','=','draft')]"/>

    <!-- Then group by -->
    <group expand="0" string="Group By">
        <filter name="group_user" string="User" context="{'group_by':'user_id'}"/>
    </group>

    <!-- Search fields at the end -->
    <field name="name"/>
    <field name="partner_id"/>
</search>
```

### Detection Rules
- Check view naming: `view_model_form` not `model_form_view`
- Verify chatter is in form views for models inheriting mail.thread
- Ensure statusbar in header for models with state
- Flag missing search views for main models

## Security Conventions

### Access Rights File (ir.model.access.csv)

```csv
id,name,model_id:id,group_id:id,perm_read,perm_write,perm_create,perm_unlink
access_model_user,custom.model user,model_custom_model,base.group_user,1,1,1,0
access_model_manager,custom.model manager,model_custom_model,base.group_system,1,1,1,1
```

### Record Rules

```xml
<!-- GOOD - Proper domain -->
<record id="custom_model_rule" model="ir.rule">
    <field name="name">Custom Model: User Access</field>
    <field name="model_id" ref="model_custom_model"/>
    <field name="domain_force">[('user_id','=',user.id)]</field>
    <field name="groups" eval="[(4, ref('base.group_user'))]"/>
</record>
```

### Detection Rules
- Flag models without access rights
- Check for overly permissive access (all 1s)
- Verify record rules for multi-company/user models
- Ensure proper group references

## Python Code Style

### Odoo-specific Guidelines

**API decorators:**
```python
# Version-specific patterns
# Odoo 14+
@api.model_create_multi
def create(self, vals_list):
    return super().create(vals_list)

# Odoo 13-
@api.model
def create(self, vals):
    return super().create(vals)
```

**NewId handling:**
```python
# GOOD - Check for NewId
@api.depends('line_ids.amount')
def _compute_total(self):
    for record in self:
        if isinstance(record.id, models.NewId):
            continue
        record.total = sum(record.line_ids.mapped('amount'))
```

**Context usage:**
```python
# GOOD - Use context for optional parameters
self.with_context(default_type='out_invoice').action_view_invoices()

# BAD - Modifying global context
self.env.context.update({'default_type': 'out_invoice'})  # Don't do this
```

### Detection Rules
- Flag deprecated decorators (@api.one, @api.returns)
- Check for improper context modification
- Verify @api.model_create_multi usage (Odoo 14+)
- Ensure proper super() calls in overrides

## Version-Specific Patterns

### Odoo 14.0+

```python
# Multi-create is mandatory
@api.model_create_multi
def create(self, vals_list):
    return super().create(vals_list)

# Image fields
image = fields.Image()  # Not Binary
```

### Odoo 15.0+

```python
# Account move changes
move_type = fields.Selection(...)  # Was 'type' before

# Python 3 only
from typing import List  # Type hints encouraged
```

### Odoo 16.0+

```python
# OWL JavaScript framework mandatory
# Legacy widgets deprecated
```

### Odoo 17.0+

```python
# Two-factor authentication changes
# New mail templates structure

# aggregator replaces group_operator (17.2+)
amount = fields.Float(aggregator='sum')  # NOT group_operator='sum'

# _read_group new signature (since 16.3)
data = self.env['model']._read_group(
    domain, groupby=['partner_id'], aggregates=['amount:sum', '__count']
)
```

### Odoo 18.0+

```python
# name_get() removed — use _compute_display_name
@api.depends('code', 'name')
def _compute_display_name(self):
    for rec in self:
        rec.display_name = f'[{rec.code}] {rec.name}'

# _search_display_name for custom name search
def _search_display_name(self, operator, value):
    return ['|', ('name', operator, value), ('code', operator, value)]

# safe_eval import fix
from odoo.tools.safe_eval import safe_eval, test_python_expr  # NOT safe_eval.test_python_expr

# SQL wrapper for safe composition
from odoo.tools import SQL
query = SQL("SELECT * FROM %s WHERE id = %s", SQL.identifier(table), id)

# search_fetch for optimization
records = self.env['model'].search_fetch(domain, ['field_name'])
```

```xml
<!-- v18+: use list for inline sub-views (in 17.0 it is tree) -->
<field name="line_ids">
    <list editable="bottom">
        <field name="name"/>
    </list>
</field>

<!-- view_mode: v18+ list,form — in 17.0 it is tree,form -->
<field name="view_mode">list,form</field>

<!-- widget="code" NOT widget="ace" -->
<field name="python_code" widget="code"/>
```

### Odoo 19.0 (CRITICAL BREAKING CHANGES)

#### Python API — Removed/Renamed

```python
# @api.returns REMOVED — AttributeError at module load
# Detection: grep -rn "@api.returns" models/
@api.returns('self', lambda value: value.id)  # ❌ CRASHES
def copy(self, default=None):                  # ✅ Just remove decorator

# record._cr, _context, _uid DEPRECATED
self._cr.execute(...)      # ❌ Use self.env.cr.execute(...)
self._context.get(...)     # ❌ Use self.env.context.get(...)
self._uid                  # ❌ Use self.env.uid

# base_import.Import class RENAMED
from odoo.addons.base_import.models.base_import import Import  # ❌ ImportError
from odoo.addons.base_import.models.base_import import Base_ImportImport as Import  # ✅

# res.users.groups_id RENAMED to group_ids (res_users.py:257)
user.groups_id                          # ❌ AttributeError
create({'groups_id': …}) / domains      # ❌ ValueError: Invalid field 'groups_id' (measured)
user.group_ids                          # ✅ direct membership
user.all_group_ids                      # ✅ direct + implied — use THIS to authorise

# @api.private decorator (new, 18.2+) — marks methods as non-RPC-exposed
@api.private
def _internal_method(self):
    pass

# read_group deprecated for public API — use formatted_read_group (18.2+)
# Cron batch processing with _commit_progress (new)
self.env['ir.cron']._commit_progress(processed, remaining=remaining)
```

#### XML Security — res.groups Privilege System

```xml
<!-- category_id REMOVED from res.groups — use res.groups.privilege -->
<!-- Detection: grep -rn "category_id" security/*.xml -->

<!-- ❌ WRONG — crashes during installation -->
<record id="group_user" model="res.groups">
    <field name="category_id" ref="module_category_x"/>
    <field name="users" eval="[(4, ref('base.user_admin'))]"/>
</record>

<!-- ✅ CORRECT — new privilege system -->
<record id="privilege_x" model="res.groups.privilege">
    <field name="name">Feature</field>
    <field name="sequence">100</field>
    <field name="category_id" ref="module_category_x"/>
</record>
<record id="group_user" model="res.groups">
    <field name="privilege_id" ref="privilege_x"/>
    <field name="user_ids" eval="[(4, ref('base.user_admin'))]"/>
</record>

<!-- ir.rule domains: groups_id → group_ids -->
<!-- Detection: grep -rn "groups_id" security/*.xml -->
<field name="domain_force">[('x', 'in', user.group_ids.ids)]</field>
```

#### XML Views

```xml
<!-- Search views: <group> wrapper REMOVED — "Vista no disponible" error -->
<!-- Detection: grep -rn 'expand="0"' views/*.xml -->
<search>
    <field name="name"/>
    <separator/>
    <filter name="group_by_x" string="X" context="{'group_by': 'x_id'}"/>
</search>

<!-- Kanban: kanban-box → card -->
<!-- Detection: grep -rn "kanban-box" views/*.xml -->
<t t-name="card">  <!-- NOT t-name="kanban-box" -->

<!-- Form context: active_id → id -->
<!-- Detection: grep -rn "active_id" views/*.xml -->
<button context="{'default_parent_id': id}"/>  <!-- NOT active_id -->

<!-- ir.cron: numbercall REMOVED -->
<!-- Detection: grep -rn "numbercall" data/*.xml -->
<!-- Just remove the line entirely -->
```

#### JavaScript/Frontend

```javascript
// RPC service REMOVED from frontend/public components
// Detection: grep -r "useService.*rpc" --include="*.js"
// Replace with fetch API + CSRF token
async _jsonRpc(endpoint, params = {}) {
    const response = await fetch(endpoint, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'X-Csrf-Token': document.querySelector('meta[name="csrf-token"]')?.content || '',
        },
        body: JSON.stringify({jsonrpc: "2.0", method: "call", params, id: 1})
    });
    const data = await response.json();
    return data.result;
}
```

#### External API

```
# JSON-2 API is now the primary protocol
# POST /json/2/<model>/<method> with Bearer token
# JSON-RPC/XML-RPC deprecated (removal planned Odoo 22/2028)
```

### Detection Rules (all versions)
- Flag version-incompatible code patterns
- Check for deprecated fields/methods
- Verify compatibility with target Odoo version
- Identify outdated JavaScript (pre-OWL in 16+)

### Odoo 19 Specific Detection Commands
```bash
# Run ALL v19 checks at once
echo "=== @api.returns ===" && grep -rn "@api.returns" --include="*.py"
echo "=== self._cr/_context/_uid ===" && grep -rn "self\._cr\b\|self\._context\b\|self\._uid\b" --include="*.py"
echo "=== category_id in groups ===" && grep -rn "category_id" security/*.xml 2>/dev/null
echo "=== users field in groups ===" && grep -rn '"users"' security/*.xml 2>/dev/null
echo "=== groups_id ===" && grep -rn "groups_id" --include="*.xml" --include="*.py"
echo "=== Import class ===" && grep -rn "import Import" --include="*.py"
echo "=== search <group> ===" && grep -rn 'expand="0"' --include="*.xml"
echo "=== numbercall ===" && grep -rn "numbercall" --include="*.xml"
echo "=== kanban-box ===" && grep -rn "kanban-box" --include="*.xml"
echo "=== active_id ===" && grep -rn "active_id" --include="*.xml"
echo "=== name_get ===" && grep -rn "def name_get" --include="*.py"
echo "=== group_operator ===" && grep -rn "group_operator" --include="*.py"
echo "=== RPC service ===" && grep -rn "useService.*rpc" --include="*.js"
echo "=== aggregator=None ===" && grep -rn "aggregator=None" --include="*.py"
```
