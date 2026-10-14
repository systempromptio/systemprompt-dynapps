# Dynamic Phase Generation for Odoo Configuration Migration

## Core Principle

Phases are **NOT hardcoded**. They are **dynamically discovered** based on what is actually installed and configured in the staging database. This document describes the algorithm Claude uses to generate a migration plan for ANY Odoo instance (v14-v19), ANY client.

---

## Phase Discovery Algorithm

```
ALGORITHM: generate_migration_plan(staging_connection)

1. READ all installed modules from staging:
   modules = search_read('ir.module.module', [('state','=','installed')], ['name','shortdesc'])

2. READ all models with data:
   all_models = search_read('ir.model', [('transient','=',False)], ['model','name','state'])
   for each model:
       count = search_count(model, [])
       if count > 0: mark as "has_data"

3. CATEGORIZE each model into a functional area:
   - accounting:  model starts with 'account.' or 'l10n_'
   - stock:       model starts with 'stock.' or 'product.size' or 'warehouse.'
   - delivery:    model starts with 'delivery.' or matches carrier patterns
   - product:     model starts with 'product.' (config only, not transactional)
   - sale:        model starts with 'sale.' or 'sales.' or 'crm.team'
   - purchase:    model starts with 'purchase.'
   - connector:   model in [ftp.*, prestashop.*, smb.*, aws.*, ecommerce.offer, indaws.import.*]
   - pos:         model starts with 'pos.' or 'loyalty.'
   - website:     model starts with 'website.'
   - crm:         model starts with 'crm.'
   - hr:          model starts with 'hr.' or 'resource.'
   - project:     model starts with 'project.'
   - helpdesk:    model starts with 'helpdesk.'
   - mrp:         model starts with 'mrp.'
   - subscription: model starts with 'sale.subscription'
   - payment:     model starts with 'payment.'
   - indaws_core: model in [api.*, ecommerce.*, indaws.import.*, product.pricelist.formula]
   - custom:      model where state='manual' OR module not in (odoo_base + enterprise + OCA + indaws_core)

4. DETECT Studio customizations:
   studio_models = search('ir.model', [('model','like','x_%')])
   studio_fields = search('ir.model.fields', [('name','like','x_studio_%')])
   studio_views  = search('ir.ui.view', [('name','like','%studio%')])

5. DETECT data XML files that need sync:
   noupdate_records = search('ir.model.data', [('noupdate','=',False),('module','not in',base_modules)])

6. FOR EACH category with data or config, CREATE a phase:
   - Always include: PHASE_COMPANY (company+users), PHASE_PARAMS (params+security)
   - Include category phase only if staging has relevant data
   - Include STUDIO phase if any x_* models/fields detected
   - Include INDAWS_CORE phase if any indaws_* modules installed (v18+)
   - Include CUSTOM phase if any non-standard modules with custom models detected

7. GENERATE phase plan with:
   - Phase number (sequential)
   - Phase name
   - Condition that activated it
   - List of models to migrate
   - Record counts from staging
   - Known particularities (from lessons-learned.md)

8. OUTPUT: ordered migration plan table
```

---

## Phase Reference Catalog

These are the POSSIBLE phases. Only phases whose activation condition is met will be included in a given client's migration plan.

### PHASE: Company and Users (ALWAYS ACTIVE)
**Condition**: Always
**Models**:
- `res.company` - fiscal data, branding, special accounts
- `res.users` - users with security groups
- `res.partner` - company partner + company contacts

**Key points**:
- `res.company` ID=1 always exists (safe to `browse(1)`)
- Security groups resolved by `xml_id`, not by name
- User passwords: set temporary, user changes on first login

---

### PHASE: Accounting and Tax
**Condition**: `count(account.journal) > 0` in staging
**Models**:
- `account.journal` - journals (sales, purchases, banks, cash, marketplace, country-specific)
- `account.tax` - taxes (VAT, withholdings, OSS, reverse charge)
- `account.tax.group` - tax groups
- `account.fiscal.position` - fiscal positions + tax/account mappings
- `account.account` - custom chart of accounts entries
- `account.payment.term` - payment terms
- `account.payment.mode` - payment modes (OCA: account-payment)
- `account.payment.method` - payment methods (transfer, check, etc.)
- `account.reconcile.model` - reconciliation models
- `account.analytic.plan` - analytic plans
- `account.analytic.account` - analytic accounts
- `account.incoterms` - incoterms (if custom)
- `l10n_es_aeat.*` - AEAT tax reporting models (Spain, if installed)

**Key points**:
- Journals: search by `code` (unique), not by name
- Taxes: many come with localization, only migrate custom ones
- Fiscal positions: verify tax mappings resolve correctly
- Payment terms: search by `name`

---

### PHASE: Warehouses and Logistics
**Condition**: `count(stock.warehouse) > 1` in staging (if only 1, it is the default)
**Models**:
- `stock.warehouse` - warehouses with codes + `resupply_wh_ids`
- `stock.location` - locations (parent_id hierarchy)
- `stock.picking.type` - operation types (standard auto-created, CUSTOM are not)
- `stock.route` - stock routes + `warehouse_ids` + `warehouse_selectable`
- `stock.rule` - stock rules (verify locations exist before creating)
- `stock.package.type` - custom package types (dimensions, weight, barcode)
- `stock.replenishment.origin` - replenishment origins (if custom module)

**Critical inventory points**:
- Warehouses: search by `code`, NOT by name
- `resupply_wh_ids`: configure inter-warehouse replenishment (m2m), verify post-migration
- Locations: respect `parent_id` hierarchy, search by `complete_name`
- Routes auto-created when warehouse steps change; production may have more than staging
- `warehouse_selectable`: sync with staging (often `False` in production but `True` in staging)
- Custom picking types (PICK, PACK-INT, adjustments, precounting, POS): migrate by `sequence_code` + `warehouse_id`
- Orphan stock rules: verify `location_src_id` and `location_dest_id` exist before creating
- `stock.warehouse.orderpoint` (~10K+): product-linked data, NOT config - do not migrate here

**Post-migration inventory sync**:
- `delivery.carrier`: `fixed_price`, `prod_environment`, product codes
- `stock.package.type`: dimensions
- Server actions with stock Python code
- Stock crons (intervals, nextcall)

**Transactional inventory data (DO NOT migrate)**:
- `stock.picking.batch`, `stock.quant.package`, `stock.lot`
- `stock.valuation.layer`, `stock.replenishment`
- Pricing-generated records regenerate when pricing crons run

---

### PHASE: Carriers and Shipping
**Condition**: `count(delivery.carrier) > 0` in staging
**Models**:
- `delivery.carrier` - carriers with configuration
- `delivery.price.rule` - price rules per carrier
- Carrier grouping models (varies: `x_transportistas`, `delivery.carrier.grouping`, etc.)

**Carrier subtypes** (detect which ones the client has):

| Type | `delivery_type` value | Credential fields |
|------|----------------------|-------------------|
| SEUR Atlas | `seur_atlas` | vat, account_code, username, password, secret, client |
| Correos | `correos` | username, password, customer_code |
| Correos Express | `correos_express` | username, password, customer_code |
| Sendcloud | `sendcloud` | public_key, secret_key -> **FORCE `delivery_type='fixed'`** |
| Sending | `sending` | username, password, client_code |
| Amazon | `amazon` | refresh_token, client_secret, access_key, secret_key |
| Miravia | `miravia` | app_key, app_secret, access_token |
| UPS | `ups` | (via Sendcloud or direct) |
| GLS | `gls` | (via integration) |
| DHL | `dhl` | site_id, password, account_number |
| Fixed/Manual | `fixed` | No credentials |

**Key points**:
- ALWAYS need `product_id` (create a "Delivery" service product as fallback)
- Sendcloud: force `delivery_type='fixed'` during migration (key validation fails)
- Filter out Studio fields that do not exist in production

---

### PHASE: Products (configuration only)
**Condition**: `count(product.attribute) > 0` in staging
**Models**:
- `product.attribute` - attributes (COLOR, SIZE, etc.)
- `product.attribute.value` - attribute values (can be 2000+)
- `product.category` - product categories (hierarchy)
- `product.brand` - brands (if brand module installed)
- `product.pricelist` - pricelists (headers only, NOT items)
- Auxiliary models per client (color mapping, special sizes, etc.)

**Key points**:
- DEACTIVATE automations on `product.attribute.value` BEFORE migrating
- Pricelists: search by `ilike` (names in JSONB in v17+)
- Pricelist items (100K+) are created when importing products
- Do NOT migrate `product.template` or `product.product` here - imported separately

### Custom fields audit for products
- Detect `x_studio_*` and `x_*` fields on `product.template` and `product.product`
- Fields on `product.product` may be delegated from `product.template` (delegation inheritance)
- To verify: check `ir.model.fields` for the field, then check if `product.product` can access it via `product_tmpl_id`
- Common pattern: 10-20 Studio fields appear "missing" on `product.product` but are accessible through delegation

---

### PHASE: Sales (configuration)
**Condition**: Sales teams, channels, or workflow config exists in staging
**Models**:
- `crm.team` - sales teams (when used for sales)
- `sales.channel` - sales channels (Indaws custom, if installed)
- `sales.channel.mapping` - channel mappings
- `sales.channel.carrier` - carrier per channel
- `payment.mode.mapping` - payment mode mappings
- `sale.automatic.workflow` / `sale.workflow.process` - automatic workflows (OCA: sale-workflow)
- `sale.order.type` - order types (OCA: sale-workflow)
- `sale.order.template` - quotation templates
- `utm.source`, `utm.medium`, `utm.campaign` - marketing UTM sources

---

### PHASE: Purchases (configuration)
**Condition**: Purchase module installed with custom configuration
**Models**:
- `purchase.order.type` - purchase order types (if exists)
- `product.supplierinfo` - supplier info (config, not transactional)
- Double validation settings, lead times, etc.

---

### PHASE: External Connectors
**Condition**: Staging has any connector model
**Detection**: Search for these models dynamically in staging:

| Connector | Model(s) | Key fields |
|-----------|----------|------------|
| FTP | `ftp.instance` | `host`, username, password (NO `name` field) |
| Prestashop (v17-) | `prestashop.instance` | `url`, api_key, password (NO `name` field) |
| SMB/Samba | `smb.instance` | server_ip, username, password |
| AWS | `aws.instance`, `aws.configuration` | access_key, secret_key, region |
| Zendesk | params `odoo_zendesk_connector.*` | api_key, domain |
| Shopping Feed | params `ind_shoppingfeed_management.*` | urls, api_keys |
| Make.com | params `make.*` | api_key, scenario_ids |
| Sendcloud | in `delivery.carrier` | public_key, secret_key |
| DynamoDB | `aws.dynamodb.field.mapping` | field_name, field_type |

**Key points**:
- Each connector may or may not exist - detect dynamically
- `ftp.instance`: search by `host`, not `name`
- `prestashop.instance`: search by `url`, not `name`
- Credentials: copy REAL values from staging (not placeholders)
- Paths containing "test"/"staging"/"dev": flag for update in production
- DynamoDB tables with "dev": flag for update

---

### PHASE: Point of Sale
**Condition**: `count(pos.config) > 0` in staging
**Models**:
- `pos.config` - POS configurations (one per shop/register)
- `pos.payment.method` - payment methods (cash, card, voucher)
- `pos.category` - POS categories
- `loyalty.program` - loyalty/coupon programs

**Key points**:
- POS configs need `journal_id` and `warehouse_id` resolved
- Database may have a default "Shop" POS - rename to match the first from staging

---

### PHASE: Ecommerce and Website
**Condition**: `website` module installed and active in staging
**Models**:
- `website` - site configuration
- `website.page` - static pages
- `website.menu` - site menus
- `website.redirect` - 301 redirects (SEO)

**Note**: This phase is uncommon in config migrations. Usually the website is configured manually. Only migrate if the client requests it explicitly.

---

### PHASE: CRM
**Condition**: CRM module installed with custom stages/teams
**Models**:
- `crm.team` - commercial teams
- `crm.stage` - pipeline stages
- `crm.tag` - opportunity tags
- `crm.lost.reason` - lost reasons
- `utm.source`, `utm.medium`, `utm.campaign` - UTM sources
- `crm.team.member` - team members (config, not assignments)

---

### PHASE: HR
**Condition**: HR module installed with configuration
**Models**:
- `hr.department` - departments
- `hr.job` - job positions
- `hr.work.location` - work locations
- `hr.leave.type` - leave types
- `hr.leave.allocation` - leave allocations (config)
- `hr.salary.rule` - salary rules (if payroll)
- `hr.salary.structure` - salary structures
- `resource.calendar` - work calendars
- `resource.calendar.attendance` - work schedules

---

### PHASE: Project
**Condition**: Project module installed with custom stages/tags
**Models**:
- `project.task.type` - task stages
- `project.tags` - project tags
- `project.project` - projects (templates only, not data)

---

### PHASE: Helpdesk
**Condition**: Helpdesk module installed (Enterprise)
**Models**:
- `helpdesk.team` - helpdesk teams
- `helpdesk.stage` - ticket stages
- `helpdesk.tag` - tags
- `helpdesk.sla` - SLAs
- `helpdesk.ticket.type` - ticket types

---

### PHASE: Manufacturing
**Condition**: MRP module installed with configuration
**Models**:
- `mrp.workcenter` - work centers
- `mrp.workcenter.capacity` - capacities
- `mrp.routing.workcenter` - operations
- BOM configuration (config, not data)

---

### PHASE: Subscriptions
**Condition**: `sale_subscription` module installed
**Models**:
- `sale.subscription.template` - subscription templates
- `sale.subscription.stage` - stages
- Recurring invoicing configuration

---

### PHASE: Payments and Banking
**Condition**: `payment` module with configured providers
**Models**:
- `payment.provider` - payment providers (Stripe, PayPal, Redsys, etc.)
- `payment.method` - payment methods
- `res.partner.bank` - company bank accounts
- `account.payment.method.line` - payment method lines on journals

**Key points**:
- `payment.provider` has credentials (API keys) - copy from staging
- Country-specific providers (e.g. Redsys for Spain): merchant number, secret key

---

### PHASE: Automations
**Condition**: Staging has custom server actions or automated actions
**Models**:
- `ir.actions.server` (state=code/multi/object_write) - server actions with code
- `base.automation` - automated actions with triggers
- `ir.cron` - scheduled actions

**Critical points**:
- **ALL automations created INACTIVE** (`active=False`)
- **ALL custom crons created INACTIVE**
- `ir.actions.server`: read from staging in **batches of 10** (singleton computed field bug)
- `base.automation`: does NOT have `sequence` field in v17; filter it
- Detect and report hardcoded IDs in server action Python code
- Progressive activation post go-live:
  - T+0h: base (auto-vacuum, queue)
  - T+1h: workflows
  - T+4h: stock/pricing
  - T+24h: integrations
  - T+48h: tax reporting

---

### PHASE: Studio
**Condition**: `count(ir.model, [('model','like','x_%')]) > 0` in staging
**Models**:
- `ir.model` - custom models (x_*)
- `ir.model.fields` - Studio fields (x_studio_*)
- `ir.ui.view` - Studio views
- `ir.actions.act_window` - Studio window actions
- `ir.ui.menu` - Studio menus
- `ir.model.access` - Studio ACLs

**Key points**:
- Execute BEFORE migration phases (fields must exist first)
- Order: models -> inverse many2one -> fields -> ACLs -> views
- `product.product` fields: delegated to `product.template`
- `res.users` fields: delegated to `res.partner`
- `one2many` fields need the inverse `many2one` created first

### Custom fields audit
To detect and verify all custom fields per model:
```python
# Find all x_studio_* and x_* fields
custom_fields = search_read('ir.model.fields', [
    '|', ('name', 'like', 'x_studio_%'),
    '&', ('name', 'like', 'x_%'), ('name', 'not like', 'x_studio_%')
], ['model_id', 'name', 'field_description', 'ttype', 'relation'])

# Group by model
for model, fields in group_by(custom_fields, 'model_id'):
    # For product.product fields, check delegation
    if model == 'product.product':
        for f in fields:
            # Check if field actually lives on product.template
            tmpl_field = search('ir.model.fields', [
                ('model_id.model','=','product.template'),
                ('name','=',f['name'])
            ])
            if tmpl_field:
                print(f"  {f['name']}: DELEGATED from product.template")
            else:
                print(f"  {f['name']}: DIRECT on product.product")
```

---

### PHASE: Indaws Core (v18+)
**Condition**: Any `indaws_*` modules installed in staging
**Repositories** (at `/path/to/indaws/`):

| Repository | Modules | Examples |
|------------|---------|---------|
| `indaws-odoo` | ~140 | indaws_base, indaws_api, indaws_account, indaws_sale, indaws_stock, indaws_product, indaws_pos, indaws_data_import, indaws_storage, indaws_report_docx, indaws_l10n_es_edi_verifactu |
| `indaws-odoo-enterprise` | ~13 | indaws_base_enterprise, indaws_base_planning, indaws_sale_subscription, indaws_sign |
| `connector-sale-odoo` | ~6 | indaws_sale_connector, indaws_sale_prestashop, indaws_sale_woocommerce |

**Models added by Indaws Core** (detect dynamically):

| Module | Model | Purpose | Key fields |
|--------|-------|---------|------------|
| `indaws_api` | `api.access_token` | REST API auth tokens | token, user_id, expiry |
| `indaws_api` | `api.endpoint` | REST API endpoints | method, path, model_id |
| `indaws_sale_connector` | `ecommerce.offer` | Ecommerce product offers | product_id, ecommerce_id, price |
| `indaws_sale_connector` | `indaws.import.connection` | Import connections | url, credentials |
| `indaws_data_import` | `indaws.import.connection` | Data import config | host, protocol, credentials |
| `indaws_data_import` | `indaws.import.mapping` | Field mappings | source_field, dest_field, model_id |
| `indaws_data_import` | `indaws.import.group` | Import grouping | name, connection_id |
| `indaws_data_import` | `indaws.import.parser` | Data parsing | format, delimiter |
| `indaws_data_import` | `indaws.import.encoding` | Encoding config | encoding type |
| `indaws_sale` | `product.pricelist.formula` | Pricelist formulas | formula, pricelist_id |
| `indaws_product` | `product.brand` | Product brands | name, logo |

**Key points**:
- Indaws Core modules are standardized across clients for v18+
- Verify repo version matches between staging and production
- Some modules have demo data - do NOT migrate demo records
- `connector-sale-odoo` replaces older custom connectors from v14-v17
- API endpoints and import connections are environment-specific - verify URLs

### Sub-phase: Sale Connectors (v18+)
**Condition**: `indaws_sale_prestashop` or `indaws_sale_woocommerce` installed
- Ecommerce platform connection config
- Product/category/tax mappings between Odoo and store
- Import/export queue configuration
- Store API credentials

### Sub-phase: OCA Modules Config
**Condition**: OCA modules installed that add configuration models

| OCA Repo | Module | Config Model | What to migrate |
|----------|--------|-------------|----------------|
| account-payment | `account_payment_mode` | `account.payment.mode` | Payment modes |
| account-payment | `account_payment_order` | `account.payment.order.mode` | Payment order config |
| bank-payment | `account_banking_mandate` | `account.banking.mandate` | SEPA mandates |
| sale-workflow | `sale_automatic_workflow` | `sale.workflow.process` | Automatic sale workflows |
| sale-workflow | `sale_order_type` | `sale.order.type` | Sale order types |
| delivery-carrier | `delivery_carrier_*` | Extensions of `delivery.carrier` | Extra carrier fields |
| l10n-spain | `l10n_es_aeat` | `l10n.es.aeat.report` | AEAT models (303, 111, 347) |
| commission | `sale_commission` | `sale.commission` | Sales commissions |
| contract | `contract` | `contract.template` | Contract templates |
| queue | `queue_job` | `queue.job.channel` | Queue job channels |

### Sub-phase: Verticals (if applicable)
**Condition**: Vertical repos (e.g. `vertical-odoo-inbuilt`) present
**Example**: Construction vertical with modules for:
- `inbuilt_project_certification` - Project certifications
- `inbuilt_sale_measurement` - Measurements
- `inbuilt_fiebdc3_import` - FIEBDC3 format import
- `inbuilt_sale_margin_project` - Project margin tracking

---

### PHASE: Client Custom Modules
**Condition**: Custom modules specific to the client (not Odoo base, not Enterprise, not OCA, not Indaws Core)
**Detection**: Modules in `addons_path` that do not belong to any known repository

For each custom module:
1. Read `__manifest__.py` to understand its purpose
2. Identify models it creates (`models/*.py`)
3. Identify data it needs (`data/*.xml`, `data/*.csv`)
4. Migrate the configuration of those specific models

**Examples from real migrations**:

*Client A (v17) - Retail/ecommerce:*
- `ind_product_pricing_automation` -> `automatic.pricing` (7 rules with Python code)
- `ind_shoppingfeed_management` -> Shopping Feed params
- `indaws_delivery_tracking_manager` -> tracking params
- `indaws_product_electronic_labels` -> electronic label params

*Client B (v18) - Manufacturing:*
- `custom_mrp_planning` -> `mrp.planning.config` (production planning rules)
- `custom_quality_control` -> `quality.check.template` (QC templates)

**Each client will have different custom models** - Claude must analyze them one by one.

---

### PHASE: Parameters and Security (ALWAYS ACTIVE)
**Condition**: Always (last phase)
**Models**:
- `ir.config_parameter` - system parameters
- `ir.default` - user default values
- `ir.filters` - saved filters
- `ir.rule` - security rules (resolve groups by xml_id)
- `ir.model.access` - custom ACLs
- `mail.template` - custom email templates
- `ir.actions.report` - custom QWeb reports

**Key points**:
- `ir.rule`: resolve groups m2m by `xml_id` (NOT by name)
- `ir.config_parameter`: NEVER copy `database.is_neutralized`, `database.uuid`, `mail.catchall.domain`, `publisher_warranty.cloc`, `database.enterprise_code`
- `mail.template`: resolve `model_id` by technical model name

---

## How Claude Decides Which Phases to Activate

After analysis, Claude generates a table like this:

```
MIGRATION PLAN - Client A (v17.0) [real example, anonymized]
=============================================================
[x] PHASE 00 - Company and users           -> 10 users, 180 partners
[x] PHASE 01 - Accounting and tax          -> 42 journals, 215 taxes, 56 FP
[x] PHASE 02 - Warehouses and logistics    -> 4 warehouses, 1300 locations
[x] PHASE 03 - Carriers and shipping       -> 29 carriers (SEUR, Correos, Sendcloud, Amazon)
[x] PHASE 04 - Products (config)           -> 2 attrs, 2673 values, 4 pricelists
[x] PHASE 05 - Sales (config)             -> 12 channels, 58 workflows
[ ] PHASE 06 - Purchases (config)          -> N/A (standard config)
[x] PHASE 07 - External connectors         -> 4 FTP, 2 Prestashop, 10 SMB, 2 AWS, Zendesk
[x] PHASE 08 - Point of Sale              -> 2 POS, 6 payment methods, 3 loyalty
[ ] PHASE 09 - Ecommerce                   -> N/A
[ ] PHASE 10 - CRM                         -> N/A (no custom config)
[ ] PHASE 11 - HR                          -> N/A
[ ] PHASE 12 - Project                     -> N/A
[ ] PHASE 13 - Helpdesk                    -> N/A (via Zendesk)
[ ] PHASE 14 - Manufacturing               -> N/A
[ ] PHASE 15 - Subscriptions               -> N/A
[ ] PHASE 16 - Payments and banking        -> N/A (payments via POS)
[x] PHASE 17 - Automations                -> 208 SA, 42 automations, 12 crons
[x] PHASE 18 - Studio                     -> 19 models, 286 fields, 170 views
[ ] PHASE 19 - Indaws Core                 -> N/A (v17, no Core)
[x] PHASE 20 - Client custom modules      -> 7 pricing rules, 10 sales indices
[x] PHASE 21 - Parameters and security    -> 79 params, 173 rules

Active phases: 12/22
Skipped phases: 10


MIGRATION PLAN - Client B (v18.0) [hypothetical example]
=========================================================
[x] PHASE 00 - Company and users           -> 25 users, 500 partners
[x] PHASE 01 - Accounting and tax          -> 15 journals, 50 taxes
[ ] PHASE 02 - Warehouses                  -> Only 1 warehouse (default)
[ ] PHASE 03 - Carriers                    -> N/A
[x] PHASE 04 - Products (config)           -> 5 attrs, 200 values, 2 pricelists
[x] PHASE 05 - Sales (config)             -> 3 teams, quotation templates
[x] PHASE 06 - Purchases (config)         -> Double validation, lead times
[ ] PHASE 07 - Connectors                  -> N/A
[ ] PHASE 08 - POS                         -> N/A
[x] PHASE 09 - Ecommerce                   -> Active website with custom pages
[x] PHASE 10 - CRM                        -> 5 stages, 3 teams
[x] PHASE 11 - HR                         -> 8 departments, calendar
[x] PHASE 12 - Project                    -> 3 custom stages
[ ] PHASE 13 - Helpdesk                    -> N/A
[x] PHASE 14 - Manufacturing              -> 5 work centers
[ ] PHASE 15 - Subscriptions               -> N/A
[x] PHASE 16 - Payments and banking        -> Redsys, Stripe
[x] PHASE 17 - Automations                -> 50 SA, 20 automations
[x] PHASE 18 - Studio                     -> 10 models, 50 fields
[x] PHASE 19 - Indaws Core                -> indaws_base, indaws_api, indaws_account,
                                              indaws_sale_connector (Prestashop)
[ ] PHASE 20 - Client custom               -> N/A
[x] PHASE 21 - Parameters and security    -> 40 params

Active phases: 15/22
Skipped phases: 7
```

Then Claude generates/customizes the migration scripts for each active phase.

---

## Models That Are Always Forgotten

These models are not obvious but contain important configuration:

| Model | Area | Description |
|-------|------|-------------|
| `ir.sequence` | Accounting/Stock | Number sequences (invoices, pickings) - custom config |
| `ir.attachment` | Global | System attachments (logos, reports) - only config ones, not all |
| `res.config.settings` | Each app | Module settings - migrate as `ir.config_parameter` |
| `digest.digest` | Marketing | Email digest configuration |
| `fetchmail.server` | Email | Incoming mail servers |
| `ir.mail_server` | Email | Outgoing SMTP servers |
| `base.document.layout` | Company | Document layout (report header/footer) |
| `res.lang` | Base | Activated languages |
| `res.currency.rate` | Accounting | Exchange rates (if multi-currency) |
| `uom.uom` | Products | Custom units of measure |
| `barcode.nomenclature` | Inventory/POS | Barcode nomenclatures |
| `stock.package.type` | Inventory | Package types |
| `mrp.workcenter.productivity.loss` | Manufacturing | Downtime reasons |
| `maintenance.equipment.category` | Maintenance | Equipment categories |
| `fleet.vehicle.model.brand` | Fleet | Vehicle brands |
| `social.media` | Marketing | Social media accounts |
| `website.redirect` | Website/SEO | 301 redirects |
| `google.calendar.credentials` | Calendar | Google Calendar credentials |
| `documents.folder` | Documents | DMS folders |

---

## Post-Migration Verification Checklist

1. **Sequences**: Do invoice/picking numbers follow the correct sequence?
2. **Logos**: Was the company logo copied? (stored in filestore)
3. **QWeb Reports**: Do PDFs generate correctly with the migrated layout?
4. **Emails**: Do email templates have the correct placeholders?
5. **Translations**: Are Studio field names in the correct language?
6. **Menu permissions**: Are all Studio menus visible to the correct users?
7. **Default views**: Is the default view for each model correct?
8. **Dashboards**: Were custom dashboards migrated?
9. **User favorites**: Do saved filters work?
10. **Timezone**: Do users have the correct timezone set?
11. **Indaws Core** (v18+): Are API endpoints, connectors, and import connections pointing to production?
12. **Ecommerce offers** (v18+): Are product mappings correct for the live store?
