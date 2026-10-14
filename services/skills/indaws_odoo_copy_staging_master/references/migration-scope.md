# Migration Scope - What Gets Migrated & Data Classification

## What gets migrated

**Configuration:** res.company, res.users, res.partner

**Accounting:** account.journal, account.fiscal.position, account.tax, account.account

**Warehouses:** stock.warehouse, stock.location, stock.picking.type, stock.route, stock.rule

**Sales & Carriers:** delivery.carrier, delivery.price.rule, sales.channel, sale.automatic.workflow

**Products (config only, NOT data):** product.attribute + values, product.category, product.brand, product.pricelist (headers only), color.mapping

**Connectors:** ftp.instance, prestashop.instance, smb.instance, aws.instance

**POS:** pos.config, pos.payment.method, loyalty.program

**Automations (ALL created INACTIVE):** ir.actions.server, base.automation, ir.cron

**Studio:** ir.model (x_*), ir.model.fields (x_studio_*), ir.ui.view, ir.actions.act_window, ir.ui.menu

**Security:** ir.model.access (custom ACLs), ir.rule (record/security rules)

**Website:** website (settings, domain, social links, CDN, analytics), website.page + ir.ui.view (COW key matching), website.menu, website.rewrite, ir.asset with website_id, website.snippet.filter

**Custom Views:** ir.ui.view without xml_id, ir.actions.act_window without xml_id

**Sequences:** ir.sequence (custom numbering), ir.sequence.date_range

**Payment Providers:** payment.provider (created with state='disabled')

**Mail Servers:** ir.mail_server (password=PLACEHOLDER), fetchmail.server (INACTIVE)

**System Config:** ir.config_parameter, ir.default, ir.filters, mail.template, ir.actions.report

**Auxiliary:** automatic.pricing, sales.indices, x_transportistas

## What is NOT migrated (by design)

- **product.template / product.product** -> use `/odoo-importation`
- **product.pricelist.item** -> created when importing products
- **product.color.attribute.discount** -> created by pricing cron
- **sale.index** -> created by pricing cron
- **Transactional data** (sale.order, account.move, stock.picking, etc.)
- **Stock quants** -> created with initial inventory
- **Filestore** -> copy manually (rsync)
- **database.is_neutralized** -> staging flag, NEVER copy

## Data Classification Framework

### Decision Tree

```
Is this model in the NEVER_MIGRATE blacklist?
  YES -> Skip entirely
  NO  -> Does it have > 10,000 records?
    YES -> Probably transactional. Check if linked to SO/AM/SP -> SKIP
           If product data -> IMPORT separately
           If config with many records (stock.location) -> MIGRATE
    NO  -> Has ir.model.data entries with module != '__export__'?
      YES -> Records come from module XML data:
             noupdate=1 -> Verify they exist
             noupdate=0 -> Need SYNC step after migration
      NO  -> Studio (x_*) -> Use REPLAY approach
             Standard config -> MIGRATE using phase scripts
             Custom from client module -> MIGRATE, read fields dynamically
```

### Record Origin Detection

| Origin | How to detect | Action |
|--------|--------------|--------|
| Module XML data | ir.model.data with module != '__export__' and != 'studio_customization' | Verify, SYNC if noupdate=0 |
| Studio | ir.model.data with module = 'studio_customization' | REPLAY |
| User created | No ir.model.data or module = '__export__' | MIGRATE directly |
| System generated | In NEVER_MIGRATE list or computed/transient | SKIP |

### NEVER_MIGRATE Blacklist

```python
NEVER_MIGRATE = {
    # Transactional
    'sale.order', 'sale.order.line', 'purchase.order', 'purchase.order.line',
    'account.move', 'account.move.line', 'account.payment',
    'account.partial.reconcile', 'account.full.reconcile',
    'stock.picking', 'stock.move', 'stock.move.line',
    'stock.quant', 'stock.valuation.layer',
    'mrp.production', 'mrp.workorder',
    'pos.order', 'pos.order.line', 'pos.session',
    'helpdesk.ticket',
    # Generated / Computed
    'product.color.attribute.discount', 'sale.index', 'stock.warehouse.orderpoint',
    # System / Infrastructure
    'mail.message', 'mail.mail', 'mail.notification', 'mail.followers', 'mail.activity',
    'bus.bus', 'bus.presence', 'ir.logging', 'ir.attachment', 'queue.job',
    # Temporary / Session
    'website.visitor', 'website.track',
}
```

### ALWAYS_SYNC Models (XML data modified by users)

```python
ALWAYS_SYNC = {
    'ir.actions.server': ['code'],
    'base.automation': ['filter_domain', 'filter_pre_domain'],
    'ir.cron': ['interval_number', 'interval_type', 'nextcall', 'code'],
    'automatic.pricing': ['python_code_execute', 'products_filter_code', 'products_filter_domain',
                          'days_threshold', 'days_equal', 'discount_percent_1', 'discount_percent_2'],
    'delivery.carrier': ['fixed_price', 'prod_environment', 'correos_product_code',
                         'seur_atlas_service_code', 'seur_atlas_product_code'],
    'stock.package.type': ['height', 'width', 'packaging_length', 'max_weight'],
    'mail.template': ['body_html', 'subject'],
}
```
