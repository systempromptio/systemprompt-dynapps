# Engine-Aware Patterns for Odoo Module Development

Apply these patterns WHILE generating module code. This is not theory — these are concrete rules for the code you write.

Consumer skills: `odoo-module-creator`, `odoo-code-auditor`, `odoo-test-generator`, `odoo-module-upgrader`.

## 1. Performance Patterns

### precompute=True (save an UPDATE on create)
```python
# IF the compute depends ONLY on fields present in create vals → use precompute
total = fields.Float(compute='_compute_total', store=True, precompute=True)

# DO NOT use if compute needs data from other models (will fail on create)
partner_name = fields.Char(compute='_compute_partner_name', store=True)  # NO precompute
```
**Rule:** Add `precompute=True` when ALL depends fields are on the same model.

### Indexes (suggest automatically)
```python
# M2O with sparse data (mostly NULL) → btree_not_null
lot_id = fields.Many2one('stock.lot', index='btree_not_null')

# Name/code fields searched with ilike → trigram
name = fields.Char(required=True, index='trigram')
code = fields.Char(index='trigram')

# Date fields used in filtering → btree
date_order = fields.Date(index=True)

# Regular M2O (always set) → btree (default, auto for M2O)
partner_id = fields.Many2one('res.partner')  # already indexed by default
```
**Rule:** Add `index='trigram'` on name/code fields. Add `index='btree_not_null'` on optional M2O. Add `index=True` on date fields used as filters.

### N+1 Prevention in Compute Methods
```python
# BAD — one query per record
@api.depends('line_ids.price_total')
def _compute_total(self):
    for rec in self:
        rec.total = sum(rec.line_ids.mapped('price_total'))  # N+1 if lines not prefetched

# GOOD — pre-load, then loop
@api.depends('line_ids.price_total')
def _compute_total(self):
    for rec in self:
        rec.total = sum(rec.line_ids.mapped('price_total'))
    # OK here because line_ids is O2M on same model — prefetched by depends

# BAD — traversing M2O in loop
@api.depends('partner_id')
def _compute_partner_phone(self):
    for rec in self:
        rec.partner_phone = rec.partner_id.phone  # 1 query per partner!

# GOOD — pre-load the relation
@api.depends('partner_id')
def _compute_partner_phone(self):
    self.partner_id  # Force prefetch ALL partners in one query
    for rec in self:
        rec.partner_phone = rec.partner_id.phone  # From cache
```
**Rule:** Before a loop that accesses M2O fields, pre-load: `self.relation_id`

### store=True vs store=False Decision
| Use store=True when | Use store=False when |
|---------------------|---------------------|
| Field is searched/filtered frequently | Field is rarely accessed |
| Field appears in list/tree views | Field only shown in form view |
| Field is used in read_group/reports | Depends on many volatile fields |
| Depends on few, stable fields | Recomputation would be too expensive |
| Field needs SQL index | Real-time accuracy is critical |

### Batch Operations (in action methods)
```python
# BAD — create in loop
def action_generate_lines(self):
    for product in products:
        self.env['sale.order.line'].create({'product_id': product.id, ...})

# GOOD — batch create
def action_generate_lines(self):
    vals_list = [{'product_id': p.id, ...} for p in products]
    self.env['sale.order.line'].create(vals_list)  # Single INSERT (batch)
```
**Rule:** Always use `create(vals_list)` for multiple records, never `create()` in a loop.

### read_group for Aggregation
```python
# BAD — load all records and sum in Python
orders = self.env['sale.order'].search(domain)
total = sum(orders.mapped('amount_total'))

# GOOD — SQL aggregation
data = self.env['sale.order'].read_group(
    domain, ['amount_total:sum'], [])
total = data[0]['amount_total']
```
**Rule:** For sums/counts/averages, use `read_group()` instead of loading records.

## 2. Field Declaration Patterns

### compute_sudo
```python
# Use when compute accesses cross-company or restricted data
cost = fields.Float(
    compute='_compute_cost',
    compute_sudo=True,    # Runs as superuser — necessary if field reads cost data
    groups='base.group_system',  # But only visible to admins
)

# DO NOT use if user should see their own scoped data
my_tasks = fields.Integer(compute='_compute_my_tasks')  # No sudo — respects access rules
```
**Rule:** `compute_sudo=True` when the compute needs data the current user can't access. Pair with `groups=` to restrict visibility.

### recursive=True (hierarchical fields)
```python
# REQUIRED when field depends on itself through parent chain
full_path = fields.Char(compute='_compute_full_path', store=True, recursive=True)

@api.depends('name', 'parent_id.full_path')
def _compute_full_path(self):
    for rec in self:
        if rec.parent_id:
            rec.full_path = f"{rec.parent_id.full_path} / {rec.name}"
        else:
            rec.full_path = rec.name
```
**Rule:** If `@api.depends` includes `parent_id.same_field` or any self-referential path → add `recursive=True`.

### Compute method return convention
```python
# CORRECT — compute methods do NOT return
@api.depends('quantity', 'price_unit')
def _compute_subtotal(self):
    for rec in self:
        rec.subtotal = rec.quantity * rec.price_unit
    # No return statement. The engine ignores any return value from compute.

# WRONG — adding return triggers pylint_odoo missing-return false positive
# AND signals a misunderstanding of how compute works
def _compute_subtotal(self):
    for rec in self:
        rec.subtotal = rec.quantity * rec.price_unit
    return self  # ← unnecessary, ignored by engine
```
**Rule:** `_compute_*` and `_onchange_*` methods mutate `self` directly and never return. Hook overrides (`create`, `copy`, `write`, `unlink`, etc.) DO return what `super()` returns. Pylint_odoo's `missing-return` rule flags both as warning indiscriminately — keep the rule OFF in `.pylintrc-mandatory` (see gotcha #49 for full rationale).

### depends Depth Warning
```python
# OK — shallow (1 level)
@api.depends('partner_id.name')

# OK — moderate (2 levels, specific)
@api.depends('order_id.partner_id.name')

# DANGEROUS — deep + broad (triggers recomputation on ALL child records)
@api.depends('partner_id.child_ids.invoice_ids.amount_total')
# → Changing any invoice of any child of any partner → recomputes THIS field
```
**Rule:** Keep depends depth ≤ 2. If deeper, consider a stored intermediate field or `store=False`.

### company_dependent Fields
```python
# Stored as JSONB: {"company_id": value}
default_warehouse = fields.Many2one(
    'stock.warehouse',
    company_dependent=True,  # Different value per company
)
# Searching: search() automatically filters by current company in context
# Gotcha: search across companies requires sudo() + manual JSONB query
```

### Monetary Fields (write_sequence matters)
```python
# CORRECT ORDER — currency BEFORE monetary
currency_id = fields.Many2one('res.currency', default=lambda self: self.env.company.currency_id)
amount = fields.Monetary(currency_field='currency_id')

# WRONG ORDER — will cause write errors
amount = fields.Monetary(currency_field='currency_id')  # ← declared first
currency_id = fields.Many2one('res.currency')            # ← declared after = problem
```
**Rule:** Always declare `currency_id` BEFORE any `Monetary` field that references it.

## 3. Security Patterns

### Record Rules (generate when needed)
```python
# Common pattern: users see only their own records
# security/ir_rule.xml
<record id="rule_my_model_user" model="ir.rule">
    <field name="name">My Model: User sees own</field>
    <field name="model_id" ref="model_my_model"/>
    <field name="domain_force">[('user_id', '=', user.id)]</field>
    <field name="groups" eval="[(4, ref('base.group_user'))]"/>
</record>

# Global rule (no group = applies to ALL users, AND-ed)
<record id="rule_my_model_company" model="ir.rule">
    <field name="name">My Model: Multi-company</field>
    <field name="model_id" ref="model_my_model"/>
    <field name="domain_force">
        ['|', ('company_id', '=', False), ('company_id', 'in', company_ids)]
    </field>
    <!-- No groups field = global rule -->
</record>
```
**Rule:** Always generate multi-company rule if model has `company_id`. Generate user-scope rule if model has `user_id` or `employee_id`.

### check_company on Many2one — needs `_check_company_auto` on the model
```python
class MyModel(models.Model):
    _name = 'my.model'
    _check_company_auto = True      # WITHOUT this line, nothing below is checked

    # ALWAYS add on M2O that references records potentially from other companies
    partner_id = fields.Many2one('res.partner', check_company=True)
    warehouse_id = fields.Many2one('stock.warehouse', check_company=True)
    journal_id = fields.Many2one('account.journal', check_company=True)

    # NOT needed on: company_id itself, currency_id (shared), country_id (shared)
```
**Rule:** Add `check_company=True` on every M2O except company_id, currency_id, and global reference
data — **and `_check_company_auto = True` on the model**. It defaults to `False` (v18
`odoo/models.py:630`, `BaseModel._check_company_auto`, v18; v19 `odoo/orm/models.py:451`, `BaseModel._check_company_auto`) and is the only condition guarding both calls to
`_check_company()`. **Measured in v18 and v19**: the model without that line silently accepts a
company-A record pointing at a company-B resource. Half the pattern protects nothing.

### Field-level Access (groups)
```python
# Sensitive fields — restrict to specific groups
cost_price = fields.Float(groups='base.group_system')
hourly_cost = fields.Monetary(groups='hr.group_hr_user')
margin_percent = fields.Float(groups='sales_team.group_sale_salesman_all_leads')
```
**Rule:** Fields showing costs, margins, salaries, or internal data → add `groups=` restriction.

### View-level Access (groups on XML elements)
When inheriting a view (e.g., `res.partner` form) to add fields from a
restricted model, the element MUST have `groups=` matching the ACL group.
Otherwise Odoo loads the field for ALL users → triggers a read on the
restricted model → AccessError for users without that group.

```xml
<!-- ❌ WRONG — all users see the button/page → AccessError if not in group -->
<button name="action_view_rappels" type="object"
        class="oe_stat_button" icon="fa-percent"
        invisible="rappel_count == 0"/>
<page string="Rappels" name="rappels">
    <field name="rappel_partner_ids"/>
</page>

<!-- ✓ CORRECT — only users in the group see it → no read triggered for others -->
<button name="action_view_rappels" type="object"
        class="oe_stat_button" icon="fa-percent"
        invisible="rappel_count == 0"
        groups="my_module.group_rappel_user"/>
<page string="Rappels" name="rappels"
      groups="my_module.group_rappel_user">
    <field name="rappel_partner_ids"/>
</page>
```

**Rule:** When adding stat buttons, pages, or fields that reference a model
with restricted ACL (only specific groups have read access), add `groups=`
on the XML element to match. Apply to: buttons, pages, fields, and any
container that triggers a read on the restricted model.

### sudo() Safe Pattern
```python
# SAFE — minimal scope, specific operation
def action_approve(self):
    # Need to create a record in a model the user can't access
    self.sudo().env['approval.log'].create({
        'user_id': self.env.uid,  # Track real user
        'record_ref': f'{self._name},{self.id}',
    })

# DANGEROUS — broad scope
def action_do_stuff(self):
    self.sudo().write({'state': 'done'})  # Skips ALL access rules!
    # Better: check why user can't write and fix permissions
```
**Rule:** Use `sudo()` only for creating records in restricted models. Never for write/unlink on the main model (fix permissions instead).

## 4. v19-Specific Generation Patterns

### Domain Class (if target v19)
```python
# v18 (still works in v19)
domain = [('state', '=', 'draft'), ('partner_id', '!=', False)]

# v19 (preferred)
from odoo.fields import Domain
domain = Domain('state', '=', 'draft') & Domain('partner_id', '!=', False)
```

### models.Constraint (if target v19)
```python
# v18 (still works in v19, deprecated)
_sql_constraints = [
    ('check_amount', 'CHECK(amount > 0)', 'Amount must be positive'),
]

# v19 (preferred) — class attribute with `_` prefix, NOT a list
class MyModel(models.Model):
    _name = 'my.model'

    amount = fields.Float()

    _check_amount_positive = models.Constraint(
        'CHECK(amount > 0)',
        'Amount must be positive',
    )
```
**Rule:** Each constraint in v19 is its own class attribute with leading `_`. The Postgres constraint name is derived from the attribute name (without the `_`). Multiple constraints = multiple attributes, never a list.

### models.Index / models.UniqueIndex (if target v19)
```python
# v19 — same convention as Constraint: class attribute, NOT a list
class MyModel(models.Model):
    _name = 'my.model'

    partner_id = fields.Many2one('res.partner')
    date_order = fields.Date()
    code = fields.Char()
    active = fields.Boolean(default=True)

    _partner_date_idx = models.Index('partner_id', 'date_order')
    _code_active_unique = models.UniqueIndex('code', condition='active = TRUE')
```

### Translation: `self.env._()` (if target v19)
```python
# v18 (still works in v19)
from odoo import _
message = _("Hello %(name)s", name=user.name)

# v19 (preferred — lazy + lang-aware)
message = self.env._("Hello %(name)s", name=user.name)

# In controllers (v19)
message = request.env._("Hello %(name)s", name=user.name)
```
**Rule:** In v19, prefer `self.env._()` for translations. The legacy `_()` evaluates eagerly without proper request scoping; `self.env._()` evaluates lazily against `self.env.context['lang']`, which respects the current request's lang correctly in multi-lang environments. Pylint_odoo enforces this via the `prefer-env-translation` rule.

## 5. Checklist Before Generating Each Model

- [ ] M2O fields have `check_company=True` (unless global data)
- [ ] Name/code fields have `index='trigram'`
- [ ] Date filter fields have `index=True`
- [ ] Optional M2O have `index='btree_not_null'`
- [ ] Computed store=True have complete `depends` (no missing fields)
- [ ] Computed with self-reference have `recursive=True`
- [ ] Computed accessing cross-company data have `compute_sudo=True`
- [ ] Monetary fields declared AFTER their currency_id
- [ ] Sensitive fields have `groups=` restriction
- [ ] Inherited view elements (buttons, pages, fields) referencing restricted models have `groups=`
- [ ] Eligible computed fields have `precompute=True`
- [ ] Multi-company model has `company_id` + record rule
- [ ] User-scoped model has user/employee rule
- [ ] Compute loops pre-load M2O relations before iterating
- [ ] Batch create used (not create-in-loop)
