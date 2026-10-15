# Standard Odoo Features Often Reimplemented

This document lists commonly reimplemented functionality that already exists in Odoo standard.

## Core Features

### Audit Trail / Change Tracking

**Often reimplemented:**
```python
# Custom audit log implementation
class CustomModel(models.Model):
    _name = 'custom.model'

    change_log_ids = fields.One2many('custom.change.log', 'record_id')

    def write(self, vals):
        # Custom logging logic
        self._create_change_log(vals)
        return super().write(vals)
```

**Odoo standard solution:**
```python
# Just inherit mail.thread
class CustomModel(models.Model):
    _name = 'custom.model'
    _inherit = ['mail.thread', 'mail.activity.mixin']

    # Fields automatically tracked
    name = fields.Char(tracking=True)
    state = fields.Selection(..., tracking=True)
```

**Detection patterns:**
- Custom change log models
- Override of write() for audit purposes
- Custom history tables

### Document Attachments

**Often reimplemented:**
```python
# Custom attachment handling
class CustomModel(models.Model):
    _name = 'custom.model'

    file_data = fields.Binary()
    file_name = fields.Char()
```

**Odoo standard solution:**
```python
# Use standard ir.attachment
class CustomModel(models.Model):
    _name = 'custom.model'
    _inherit = ['mail.thread']

    # Attachments automatically available via mail.thread
    # Or explicitly:
    attachment_ids = fields.Many2many('ir.attachment', compute='_compute_attachments')
```

**Detection patterns:**
- Binary fields for file storage
- Custom file download controllers
- Manual file storage management

### Email Integration

**Often reimplemented:**
```python
# Custom email sending
import smtplib
from email.mime.text import MIMEText

def send_email(self):
    msg = MIMEText(self.content)
    # Manual SMTP configuration
```

**Odoo standard solution:**
```python
# Use Odoo's mail system
class CustomModel(models.Model):
    _name = 'custom.model'
    _inherit = ['mail.thread']

    def send_notification(self):
        self.message_post(
            body="Notification content",
            subject="Subject",
            partner_ids=self.partner_id.ids,
        )

    # Or use mail templates
    template = self.env.ref('module.email_template_id')
    template.send_mail(self.id, force_send=True)
```

**Detection patterns:**
- Direct SMTP usage
- Custom email queue implementations
- Manual email HTML generation

### Activity/Task Management

**Often reimplemented:**
```python
# Custom todo/task system
class CustomTask(models.Model):
    _name = 'custom.task'

    user_id = fields.Many2one('res.users')
    due_date = fields.Date()
    description = fields.Text()
```

**Odoo standard solution:**
```python
# Use mail.activity
class CustomModel(models.Model):
    _name = 'custom.model'
    _inherit = ['mail.activity.mixin']

    # Activities automatically available
    def create_followup_activity(self):
        self.activity_schedule(
            'module.mail_activity_type_id',
            user_id=self.user_id.id,
            date_deadline=fields.Date.today(),
        )
```

**Detection patterns:**
- Custom task models linked to other models
- Manual reminder systems
- Custom due date tracking

### Company/Multi-company Support

**Often reimplemented:**
```python
# Manual company filtering
class CustomModel(models.Model):
    _name = 'custom.model'

    company_id = fields.Many2one('res.company')

    @api.model
    def search(self, args, **kwargs):
        # Manual company filtering
        args += [('company_id', '=', self.env.company.id)]
        return super().search(args, **kwargs)
```

**Odoo standard solution:**
```python
# Use check_company mixin
class CustomModel(models.Model):
    _name = 'custom.model'
    _check_company_auto = True

    company_id = fields.Many2one('res.company', default=lambda self: self.env.company)
    partner_id = fields.Many2one('res.partner', check_company=True)

    # Record rules handle filtering automatically
```

**Detection patterns:**
- Manual company filtering in search
- Custom access control by company
- Missing check_company on relational fields

### Currency Conversion

**Often reimplemented:**
```python
# Custom currency conversion
def convert_amount(self, amount, from_currency, to_currency):
    rate = self._get_exchange_rate(from_currency, to_currency)
    return amount * rate
```

**Odoo standard solution:**
```python
# Use built-in currency compute
def convert_amount(self, amount, from_currency, to_currency, date=None):
    return from_currency._convert(
        amount,
        to_currency,
        self.company_id,
        date or fields.Date.today()
    )
```

**Detection patterns:**
- Custom exchange rate tables
- Manual rate calculations
- Hardcoded conversion logic

### Sequence Number Generation

**Often reimplemented:**
```python
# Custom sequence generation
class CustomModel(models.Model):
    _name = 'custom.model'

    def create(self, vals):
        if not vals.get('name'):
            vals['name'] = self._generate_sequence()
        return super().create(vals)

    def _generate_sequence(self):
        last = self.search([], order='id desc', limit=1)
        return 'SEQ%05d' % (int(last.name[3:]) + 1 if last else 1)
```

**Odoo standard solution:**
```python
# Use ir.sequence
class CustomModel(models.Model):
    _name = 'custom.model'

    name = fields.Char(default='/', readonly=True)

    @api.model_create_multi
    def create(self, vals_list):
        for vals in vals_list:
            if vals.get('name', '/') == '/':
                vals['name'] = self.env['ir.sequence'].next_by_code('custom.model')
        return super().create(vals_list)

# In data file:
# <record id="seq_custom_model" model="ir.sequence">
#     <field name="name">Custom Model</field>
#     <field name="code">custom.model</field>
#     <field name="prefix">SEQ</field>
#     <field name="padding">5</field>
# </record>
```

**Detection patterns:**
- Manual sequence generation in create()
- Custom counter fields
- String manipulation for IDs

### Settings/Configuration

**Often reimplemented:**
```python
# Custom settings model
class CustomSettings(models.Model):
    _name = 'custom.settings'

    setting_name = fields.Char()
    setting_value = fields.Char()
```

**Odoo standard solution:**
```python
# Use res.config.settings
class ResConfigSettings(models.TransientModel):
    _inherit = 'res.config.settings'

    custom_setting = fields.Boolean(
        config_parameter='module.custom_setting'
    )

# Or for system parameters:
value = self.env['ir.config_parameter'].sudo().get_param('module.setting_key')
```

**Detection patterns:**
- Custom configuration tables
- Settings stored in custom models
- Manual parameter storage

### Portal Access

**Often reimplemented:**
```python
# Custom portal logic
class CustomModel(models.Model):
    _name = 'custom.model'

    def _check_portal_access(self, user):
        # Custom access logic
        pass
```

**Odoo standard solution:**
```python
# Inherit portal.mixin
class CustomModel(models.Model):
    _name = 'custom.model'
    _inherit = ['portal.mixin']

    def _compute_access_url(self):
        super()._compute_access_url()
        for record in self:
            record.access_url = f'/my/custom/{record.id}'

# Controller with automatic access check
@http.route('/my/custom/<int:record_id>', auth='public')
def portal_record(self, record_id, access_token=None, **kwargs):
    record = request.env['custom.model']._document_check_access(
        'custom.model', record_id, access_token
    )
```

**Detection patterns:**
- Custom portal access checks
- Manual token generation
- Custom /my/ routes without portal.mixin

## Accounting Features

### Tax Calculation

**Often reimplemented:**
```python
# Manual tax calculation
def compute_taxes(self, amount, tax_rate):
    return amount * tax_rate / 100
```

**Odoo standard solution:**
```python
# Use account.tax compute methods
taxes = line.tax_ids.compute_all(
    line.price_unit,
    currency=line.currency_id,
    quantity=line.quantity,
    product=line.product_id,
    partner=line.partner_id,
)
```

**Detection patterns:**
- Manual percentage calculations for taxes
- Custom tax amount fields
- Hardcoded tax rates

### Invoice/Payment Reconciliation

**Often reimplemented:**
```python
# Custom payment matching
class CustomPayment(models.Model):
    def match_with_invoice(self):
        # Manual matching logic
        pass
```

**Odoo standard solution:**
```python
# Use account.payment and reconciliation
payment.action_post()
# Automatic reconciliation via account.move.line
(payment.move_id.line_ids + invoice.line_ids)\
    .filtered(lambda l: l.account_id.reconcile)\
    .reconcile()
```

**Detection patterns:**
- Custom reconciliation tables
- Manual payment status tracking
- Custom paid/unpaid logic

### Chart of Accounts

**Often reimplemented:**
```python
# Custom account structure
class CustomAccount(models.Model):
    _name = 'custom.account'

    code = fields.Char()
    name = fields.Char()
```

**Odoo standard solution:**
```python
# Extend account.account
class AccountAccount(models.Model):
    _inherit = 'account.account'

    custom_field = fields.Char()
```

**Detection patterns:**
- Custom account models
- Reimplemented account hierarchies
- Custom journal entries

## Sales/CRM Features

### Lead/Opportunity Management

**Often reimplemented:**
```python
# Custom lead tracking
class CustomLead(models.Model):
    _name = 'custom.lead'

    stage = fields.Selection([...])
    probability = fields.Float()
```

**Odoo standard solution:**
```python
# Extend crm.lead
class CrmLead(models.Model):
    _inherit = 'crm.lead'

    custom_field = fields.Char()
```

**Detection patterns:**
- Custom opportunity models
- Manual pipeline stages
- Custom probability calculations

### Quotation/Sales Order Workflow

**Often reimplemented:**
```python
# Custom quotation model
class CustomQuotation(models.Model):
    _name = 'custom.quotation'

    line_ids = fields.One2many(...)
    total = fields.Float(compute='_compute_total')
```

**Odoo standard solution:**
```python
# Extend sale.order
class SaleOrder(models.Model):
    _inherit = 'sale.order'

    custom_field = fields.Char()
```

**Detection patterns:**
- Custom quotation models
- Reimplemented order lines
- Manual discount/tax calculations

## Inventory Features

### Stock Movement Tracking

**Often reimplemented:**
```python
# Custom stock tracking
class CustomStockMove(models.Model):
    _name = 'custom.stock.move'

    product_id = fields.Many2one('product.product')
    quantity = fields.Float()
    location_from = fields.Many2one('custom.location')
```

**Odoo standard solution:**
```python
# Use stock.move
move = self.env['stock.move'].create({
    'product_id': product.id,
    'product_uom_qty': quantity,
    'location_id': source_location.id,
    'location_dest_id': dest_location.id,
})
move._action_done()
```

**Detection patterns:**
- Custom inventory movement models
- Manual stock calculations
- Custom location systems

### Lot/Serial Number Tracking

**Often reimplemented:**
```python
# Custom lot tracking
class CustomLot(models.Model):
    _name = 'custom.lot'

    number = fields.Char()
    product_id = fields.Many2one('product.product')
```

**Odoo standard solution:**
```python
# Use stock.lot
class ProductProduct(models.Model):
    _inherit = 'product.product'

    tracking = fields.Selection([
        ('serial', 'By Unique Serial Number'),
        ('lot', 'By Lots'),
        ('none', 'No Tracking'),
    ])
```

**Detection patterns:**
- Custom lot/serial models
- Manual tracking systems
- Custom expiry date logic

## Detection Strategy

For each category above:

1. **Search for custom models** that duplicate standard functionality
2. **Check for manual implementations** of features in compute methods
3. **Identify missing standard mixins** (mail.thread, portal.mixin, etc.)
4. **Flag complex custom logic** that could use standard APIs

## Recommendation Template

When detecting reimplemented features:

```
❌ Issue: Custom [feature name] implementation

Current implementation: [brief description]

Standard Odoo solution:
- Use [standard model/mixin/API]
- Benefits: [maintenance, security, compatibility]
- Migration: [brief steps to adopt standard]

Example: [code snippet]
```
