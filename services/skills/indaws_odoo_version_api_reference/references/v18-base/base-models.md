<!-- verified: v18.0 @ 2026-06 -->
# Base Models — ir.*, res.* & Mixins (Odoo 18.0)

Source: `$ODOO_SRC/odoo18/odoo/addons/base/models/` (45 files)

This reference covers the base models developers interact with most. Not every field — only the ones you need to know about when inheriting, extending, or debugging.

## Table of Contents

1. [ir.model.access (ACL)](#irmodelaccess)
2. [ir.rule (Record Rules)](#irrele)
3. [ir.config_parameter (System Parameters)](#irconfigparameter)
4. [ir.cron (Scheduled Actions)](#ircron)
5. [ir.sequence (Sequences)](#irsequence)
6. [ir.attachment (File Storage)](#irattachment)
7. [ir.ui.view (Views)](#iruiview)
8. [ir.actions (Actions)](#iractions)
9. [ir.default (Default Values)](#irdefault)
10. [ir.model & ir.model.fields](#irmodel--irmodelfields)
11. [res.partner](#respartner)
12. [res.company](#rescompany)
13. [res.users](#resusers)
14. [res.currency](#rescurrency)
15. [Common Mixins](#common-mixins)

---

## ir.model.access

ACL layer — who can do what on which model.

```csv
id,name,model_id/id,group_id/id,perm_read,perm_write,perm_create,perm_unlink
access_my_model_user,my.model user,model_my_model,base.group_user,1,1,1,0
access_my_model_manager,my.model manager,model_my_model,module.group_manager,1,1,1,1
```

- One line per model + group combination
- No group = public access (dangerous)
- Check at method call level, not per-record

---

## ir.rule

Record rules — row-level security.

Key fields:
- `model_id`: which model
- `domain_force`: domain expression (evaluated with `user`, `company_id`, `company_ids`, `time`)
- `groups`: which groups this rule applies to. Empty = global rule
- `perm_read/write/create/unlink`: which operations are filtered

**Critical behavior:**
- **Global rules** (no groups): AND-ed together. Every global rule must pass
- **Group rules** (with groups): OR-ed together. At least one must pass
- Global AND group rules: `all_global AND any_group`

```xml
<record id="rule_own_records" model="ir.rule">
    <field name="name">Own records only</field>
    <field name="model_id" ref="model_my_model"/>
    <field name="domain_force">[('user_id', '=', user.id)]</field>
    <field name="groups" eval="[(4, ref('base.group_user'))]"/>
</record>
```

Variables available in `domain_force`: `user` (res.users browse), `company_id` (int), `company_ids` (list), `time` (module).

---

## ir.config_parameter

Key-value system parameters. `Settings > Technical > System Parameters`.

```python
# Read
value = self.env['ir.config_parameter'].sudo().get_param('web.base.url')
value = self.env['ir.config_parameter'].sudo().get_param('my.param', default='fallback')

# Write
self.env['ir.config_parameter'].sudo().set_param('my.param', 'value')
```

Always use `sudo()` — parameters are protected. Store secrets here (not in code).

---

## ir.cron

Scheduled actions (cron jobs).

In v18 `ir.cron` has a **delegate** Many2one `ir_actions_server_id` to `ir.actions.server`
(`delegate=True`), so `name`, `state`, `code`, `model_id` are inherited from the server action
and can be set directly on the `ir.cron` record (as the XML below does).

Key fields:
- `model_id` / `code`: model and Python `code` to run (delegated from `ir.actions.server`)
- `interval_number` + `interval_type`: frequency. Selection in v18 is `minutes`, `hours`, `days`, `weeks`, `months` (default `months`)
- `active`: enable/disable
- `nextcall`: next scheduled execution (required, default = now)
- `user_id`: scheduler user (default current user)
- `priority`: 0 = higher, 10 = lower (default 5)
- `failure_count` / `first_failure_date`: consecutive-failure tracking, reset on success

```xml
<record id="ir_cron_my_job" model="ir.cron">
    <field name="name">My Daily Job</field>
    <field name="model_id" ref="model_my_model"/>
    <field name="state">code</field>
    <field name="code">model._cron_my_method()</field>
    <field name="interval_number">1</field>
    <field name="interval_type">days</field>
    <field name="active">True</field>
</record>
```

**Internals (verified against `ir_cron.py`):**
- Concurrency control uses row locks `SELECT ... FOR NO KEY UPDATE SKIP LOCKED` on `ir_cron` / `ir_cron_trigger` (see `_acquire_one_job`), NOT PostgreSQL advisory locks. `_try_lock` uses `FOR NO KEY UPDATE NOWAIT`.
- Failure tracking: `failure_count` field increments on each consecutive failure and resets to 0 on success; the job is deactivated once it reaches `MIN_FAILURE_COUNT_BEFORE_DEACTIVATION` (see `_update_failure_count`). Jobs are ordered `ORDER BY failure_count, priority, id`.
- Each cron runs in its own transaction (commit on success, rollback on failure).
- The cron's target method is a regular model method. `@api.autovacuum` is a separate mechanism: it registers a method to run from the `base.autovacuum_job` cron — it is NOT required for ordinary crons.

---

## ir.sequence

Auto-increment sequences for document numbering.

```python
# Get next value
next_number = self.env['ir.sequence'].next_by_code('sale.order')
# → 'SO0042'
```

Key fields:
- `code`: identifier (e.g., 'sale.order')
- `prefix`/`suffix`: format with `%(year)s`, `%(month)s`, `%(day)s`, `%(y)s`
- `padding`: zero-padding (5 → `00042`)
- `number_next`: next number
- `number_increment`: step
- `use_date_range`: separate sequences per date range

---

## ir.attachment

File storage system.

Key fields:
- `name`: filename
- `datas`: base64 content (read/write)
- `raw`: binary content (read/write, no base64 encoding)
- `res_model` + `res_id`: linked record
- `type`: `'binary'` (filestore) or `'url'` (external URL)
- `mimetype`: auto-detected

**Storage backends:**
- `_storage()` reads the `ir_attachment.location` system parameter; in CE the supported values are `'file'` (filestore in data_dir, the default) and `'db'` (binary in the `db_datas` column). Other backends (S3, etc.) require add-on modules, not core.
- `Binary` fields with `attachment=True` create `ir.attachment` records automatically

```python
attachment = self.env['ir.attachment'].create({
    'name': 'report.pdf',
    'datas': base64_content,
    'res_model': 'sale.order',
    'res_id': order.id,
})
```

---

## ir.ui.view

View definitions and inheritance.

Key fields:
- `name`: view name
- `model`: target model
- `type`: form, list, search, kanban, calendar, pivot, graph, activity, qweb
- `arch`: XML architecture
- `inherit_id`: parent view for inheritance
- `priority`: lower = applied first (default 16)
- `mode`: `'primary'` (standalone) or `'extension'` (inherits from another)

**View resolution:** when loading a view, Odoo finds the primary view with lowest priority, then applies all extensions in priority order.

---

## ir.actions

### ir.actions.act_window (most common)

Opens a view (list, form, kanban...).

```xml
<record id="action_my_model" model="ir.actions.act_window">
    <field name="name">My Records</field>
    <field name="res_model">my.model</field>
    <field name="view_mode">list,form,kanban</field>
    <field name="domain">[('active', '=', True)]</field>
    <field name="context">{'default_type': 'out'}</field>
</record>
```

### ir.actions.server

Runs Python code or calls a method. Used by automations, buttons, scheduled actions.

### ir.actions.report

Generates reports (PDF via wkhtmltopdf, XLSX, etc.).

### ir.actions.client

Opens a client-side action (JavaScript widget).

### ir.actions.act_url

Opens a URL in browser.

---

## ir.default

User-configurable default values for fields.

```python
# v18 signature: set(self, model_name, field_name, value, user_id=False, company_id=False, condition=False)

# Set default for all users
self.env['ir.default'].set('sale.order', 'warehouse_id', warehouse.id)

# Set default for specific user
self.env['ir.default'].set('sale.order', 'warehouse_id', warehouse.id, user_id=user.id)

# Set default for specific company
self.env['ir.default'].set('sale.order', 'warehouse_id', warehouse.id, company_id=company.id)
```

---

## ir.model & ir.model.fields

Metadata about models and fields. Populated automatically by the ORM.

```python
# Get model info
model = self.env['ir.model'].search([('model', '=', 'sale.order')])

# Get fields info
fields = self.env['ir.model.fields'].search([('model_id', '=', model.id)])
```

Useful for dynamic/generic operations (Studio, import/export, audit).

---

## res.partner

The central contact model — used for customers, vendors, employees, addresses.

Key fields:
- `name`, `email`, `phone`, `mobile`, `website`
- `street`, `street2`, `city`, `zip`, `state_id`, `country_id`
- `company_type`: `'person'` or `'company'`
- `is_company`: bool (computed from company_type)
- `parent_id`: parent company (for contacts of a company)
- `child_ids`: child contacts
- `commercial_partner_id`: top-level commercial entity (computed)
- `vat`: tax ID
- `property_account_receivable_id`, `property_account_payable_id`: accounting
- `lang`: preferred language
- `tz`: timezone

**Commercial partner:** `commercial_partner_id` walks up `parent_id` to find the top company. Used for invoicing and credit limits.

---

## res.company

Multi-company support.

Key fields:
- `name`, `logo`, `currency_id`
- `partner_id`: company's partner record (address, VAT, etc.)
- `parent_id` / `child_ids`: company hierarchy
- `fiscalyear_last_day/month`: fiscal year end

Every record with `company_id` belongs to a company. The ORM filters by `env.company` via record rules.

---

## res.users

User accounts.

Key fields:
- `login`: username (email)
- `password`: hashed password (never readable)
- `partner_id`: linked partner (name, email, address come from here)
- `company_id`: current company
- `company_ids`: allowed companies
- `groups_id`: security groups (Many2many to `res.groups`, rel `res_groups_users_rel`). NOTE: in v19 this field is renamed to `group_ids` and `res.groups.category_id` → `privilege_id`. In v18 it is `groups_id`.
- `share`: True = portal/external user, False = internal user

Key methods:
- `_check_credentials(credential, env)`: verify login (v18 signature — `credential` is a dict, not a plain password; raises `AccessDenied` on failure)
- `has_group(group_ext_id)`: check membership of a single group (xmlid)
- `has_groups(group_spec)`: check a comma-separated spec (supports `!` negation and `,` OR)
- `action_reset_password()`: send password reset email

---

## res.currency

Currency definitions and exchange rates.

Key fields:
- `name`: ISO code (EUR, USD)
- `symbol`: display symbol (€, $)
- `rate`: current rate (in `res.currency.rate`)
- `decimal_places`: precision (usually 2)
- `rounding`: rounding factor (0.01 for 2 decimals)

Key methods:
```python
# v18 signature: _convert(self, from_amount, to_currency, company=None, date=None, round=True)
currency._convert(amount, to_currency, company, date)    # company/date optional in v18
currency.round(amount)                                   # round to currency precision
currency.compare_amounts(amount1, amount2)               # -1, 0, 1
currency.is_zero(amount)                                 # True if amount rounds to 0
```

---

## Common Mixins

### mail.thread (Chatter)

Adds the chatter (messages, followers, activities) to a model.

```python
class MyModel(models.Model):
    _name = 'my.model'
    _inherit = ['mail.thread']

    name = fields.Char(tracking=True)  # tracked in chatter
    state = fields.Selection([...], tracking=True)
```

Key methods:
- `message_post(body=..., message_type='comment')`: post a message
- `message_subscribe(partner_ids=[...])`: add followers
- `message_notify(partner_ids=[...], body=...)`: send notification without posting
- `_track_subtype(init_values)`: override to customize tracking subtype

### mail.activity.mixin

Adds scheduled activities (tasks/reminders).

```python
_inherit = ['mail.activity.mixin']
```

Key methods:
- `activity_schedule(act_type_xmlid, date_deadline, summary, user_id)`: create activity
- `activity_feedback(act_type_xmlid, feedback)`: mark as done
- `activity_unlink(act_type_xmlid)`: cancel activity

### image.mixin

Adds standard image fields with auto-resize.

```python
_inherit = ['image.mixin']
# Adds: image_1920, image_1024, image_512, image_256, image_128
```

### portal.mixin

Adds portal access URL.

```python
_inherit = ['portal.mixin']

def _compute_access_url(self):
    for rec in self:
        rec.access_url = f'/my/orders/{rec.id}'
```
