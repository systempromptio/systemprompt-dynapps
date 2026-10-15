# Lessons Learned - Odoo Configuration Migration

## Compiled from multiple real migrations (v14-v19), continuously updated

---

## Version Differences Table

| Aspect | v14-v16 | v17 | v18-v19 |
|--------|---------|-----|---------|
| View API | `attrs={}`, `states=` | `invisible=`, `readonly=` inline | Same as v17 |
| Translations | char/text fields, `ir.translation` | JSONB (search with `ilike`) | JSONB |
| `ir.actions.server.active` | Exists | Does NOT exist | Verify per version |
| `base.automation.sequence` | Exists | Does NOT exist | Verify per version |
| API protocol | JSON-RPC only | JSON-RPC only | JSON-RPC + JSON2 (`/json/2/`) |
| Studio fields | `x_studio_*` | `x_studio_*` | `x_studio_*` |
| Indaws repos | Custom per client | Custom per client | Indaws Core (~140 community + ~13 enterprise) |
| Sale connectors | Custom (e.g. `prestashop_connector`) | Custom per client | `connector-sale-odoo` (Prestashop, WooCommerce) |
| OCA repos | Varies | Varies | 62+ repos, ~500 modules |
| Verticals | None standard | None standard | `vertical-odoo-inbuilt` (construction, etc.) |
| `product.product` fields | Direct on model | Delegated from `product.template` | Delegated from `product.template` |
| Localization (Spain) | `l10n_es_aeat` | `l10n_es_aeat` | `l10n_es_edi_sii`, `l10n_es_edi_verifactu` |

---

## Data XML with noupdate (CRITICAL)

Many custom modules load initial data via `data/*.xml` with `noupdate=0` or `noupdate=1`.
Users then modify those records from the staging UI. The migration must use the
**staging values**, NOT the original XML values.

**Commonly affected models:**
- Any model with `code`, `python_code`, `domain`, `filter` fields edited via UI
- Pricing/discount rules with thresholds and Python code
- Sales indices or classification records with renamed labels
- `product.attribute.value` with m2m relationships (e.g. `color_mapping_ids`) configured via UI
- `ir.actions.server` with custom Python `code`
- `base.automation` with adjusted `filter_domain` / `filter_pre_domain`
- `res.config.settings` which writes to `ir.config_parameter` and `res.company` fields

**Problem**: If the module has `noupdate=0`, running `-u all` in production OVERWRITES user values with the XML defaults. If `noupdate=1`, the XML does not overwrite but migration may create duplicates.

**Solution in the skill:**
1. After migrating, perform a **modified data sync**: for each model with data XML, read from staging and compare field by field with production
2. If production has the original XML value and staging has a different value, **update with the staging value**
3. If duplicates exist (XML + migrated), delete the XML one and keep the staging one
4. Detect `noupdate` flags in `ir.model.data` to know which records get overwritten on `-u all`

**M2M fields that get lost:**
- Any m2m field configured via UI (not via XML) will not be copied when creating records in production
- Always check for m2m fields on migrated records and sync them separately

### Recommendation: Set noupdate="1" in custom module data XML

**BEFORE go-live**, review ALL `data/*.xml` files in the client's custom modules and change `noupdate="0"` or bare `<odoo>` (no noupdate) to `noupdate="1"`.

**Why**: If a data file has `noupdate="0"` or does not specify noupdate, running `-u all` or updating the module causes Odoo to OVERWRITE records with the original XML values. This erases all customizations the user made in staging.

**How to detect:**
```bash
# Search in all custom modules
grep -rn 'noupdate="0"\|<odoo>' /path/to/custom/modules/*/data/*.xml | grep -v 'noupdate="1"'
```

**How to fix:**
```bash
# Change <odoo> to <odoo noupdate="1">
sed -i 's|<odoo>|<odoo noupdate="1">|g' file.xml
# Change noupdate="0" to noupdate="1"
sed -i 's|noupdate="0"|noupdate="1"|g' file.xml
```

**Commonly affected file patterns:**
- `*_pricing_data.xml` (discount rules with Python code)
- `ir_cron_data.xml` (crons with custom intervals)
- `mail_template_data.xml` (edited email templates)
- `*_data.xml` in connector modules (FTP, ecommerce, feeds, cloud)
- `delivery_*_data.xml` (carrier configuration)

**The skill must:**
1. In STEP 0 (Discovery), detect all data XML files without `noupdate="1"`
2. List them in the discovery report as WARNING
3. Recommend the user fix them BEFORE go-live
4. If code access is available, fix them automatically

---

## Fatal Mistakes - Never Repeat These

1. **Not clearing `id_mapping.json` between runs** - Scripts skip everything because records are already "mapped" to IDs from another database
2. **OdooRPC with `dry_run=True` by default** - `create`/`write` silently return `None`; it appears to work but writes nothing
3. **Not installing staging custom modules in production** - Fields those modules create do not exist; `safe_create` filters them out and records are created incomplete
4. **Not importing Studio BEFORE migration** - `x_*` models, `x_studio_*` fields, views do not exist; creates fail
5. **Using PLACEHOLDERs for credentials** - In production you must copy the real values from staging
6. **Copying `database.is_neutralized`** - This is a flag that disables critical functions; NEVER copy it from staging
7. **Not syncing XML data modified in staging** - The `-u all` restores XML values in production but staging has user-customized values. ALWAYS do a post-migration sync of: server action code, pricing rules (domains, code, thresholds), carrier config (prices, product codes, `prod_environment`), attribute value m2m relationships
8. **Duplicate records from XML + migration** - When a module creates records via XML and the migration creates more, delete the XML ones (usually identifiable by missing custom fields like `short_code`) and keep the staging ones
9. **Copying infrastructure `ir.config_parameter`** - NEVER sync: `mail.catchall.domain`, `publisher_warranty.cloc`, `database.uuid`, `crm.pls_start_date`, `database.enterprise_code` (these are server-specific)
10. **Not verifying resupply warehouses** - `resupply_wh_ids` (inter-warehouse replenishment origins) do not migrate correctly. Verify and update post-migration

---

## Field Resolution Patterns

| Problem | Cause | Solution |
|---------|-------|----------|
| `Invalid field 'name' on model 'ftp.instance'` | Model has no `name` field | Search by `host` instead |
| `Invalid field 'name' on model 'prestashop.instance'` | Same | Search by `url` instead |
| `Invalid field 'sequence' on model 'base.automation'` | Field does not exist in v17 | Filter with `if field in model._fields` |
| `Invalid field 'active' on model 'ir.actions.server'` | Field does not exist in some versions | Remove from field list before reading |
| `Invalid field 'use_in_electronic_labels'` | Custom module not installed | Install the custom module that creates the field |
| `_compute_webhook_sample_payload` singleton error | Bug in computed field on staging | Read in batches of 10 IDs instead of bulk `search_read` |
| Fields missing on `product.product` | Delegation inheritance from `product.template` | Check if field exists on `product.template` instead; 14+ Studio fields may be delegated |
| Fields missing on `res.users` | Delegation inheritance from `res.partner` | Check if field exists on `res.partner` instead |
| `ecommerce.offer` fields (v18+) | Indaws connector model | Only exists when `indaws_sale_connector` is installed |
| `api.endpoint` fields (v18+) | Indaws API model | Only exists when `indaws_api` is installed |

---

## Inventory-Specific Lessons

### Resupply Warehouses
- `resupply_wh_ids` on `stock.warehouse` defines inter-warehouse replenishment origins (m2m)
- When creating warehouses, configure where they replenish FROM
- Always verify post-migration that the m2m is correctly set

### Picking Types
- Standard picking types (IN, OUT, INTERNAL) are auto-created when a warehouse is created
- **Custom picking types** (PICK, PACK-INT, custom adjustments, precounting, POS operations) are NOT auto-created
- Migrate custom picking types by matching `sequence_code` + `warehouse_id`
- Some clients have 20+ custom picking types per warehouse

### Routes
- Routes are auto-created when warehouse `delivery_steps` / `reception_steps` change
- Production may have MORE routes than staging (not an error, they are auto-generated)
- **`warehouse_selectable`**: Many custom routes end up with `False` in production but `True` in staging. Always sync this flag post-migration
- Custom routes must be migrated explicitly; resolve `warehouse_ids` m2m

### Stock Rules
- Some rules in staging may point to locations that no longer exist (orphaned references)
- NEVER migrate rules with non-existent `location_src_id` or `location_dest_id`
- Validate location existence before creating rules

### Package Types
- `stock.package.type` records for custom packaging (carrier boxes, pallets) are not auto-created
- Migrate by name, including dimensions (`height`, `width`, `packaging_length`) and `max_weight`/`barcode`

### Replenishment Origins
- `stock.replenishment.origin` (if custom module installed) defines origins like B2B, warehouses, shops
- Migrate resolving `warehouse_id` and `route_id` by code/name

### Transactional Inventory Data (DO NOT MIGRATE)
- `stock.picking.batch`, `stock.quant.package`, `stock.lot`
- `stock.warehouse.orderpoint` (can be 10K+ records - linked to products, not config)
- `stock.valuation.layer`, `stock.replenishment`
- Pricing-generated records (e.g. `product.color.attribute.discount`, `sale.index`) regenerate automatically when pricing crons run after product import

---

## Indaws Core Patterns (v18+)

### Repository Structure
Indaws Core for v18+ consists of three main repositories at `/path/to/indaws/`:

| Repository | Module Count | Key Modules |
|------------|-------------|-------------|
| `indaws-odoo` | ~140 | `indaws_base`, `indaws_api`, `indaws_account`, `indaws_sale`, `indaws_stock`, `indaws_product`, `indaws_pos`, `indaws_data_import`, `indaws_storage`, `indaws_report_docx` |
| `indaws-odoo-enterprise` | ~13 | `indaws_base_enterprise`, `indaws_base_planning`, `indaws_sale_subscription`, `indaws_sign` |
| `connector-sale-odoo` | ~6 | `indaws_sale_connector`, `indaws_sale_prestashop`, `indaws_sale_woocommerce`, `indaws_base_sale_connector` |

### Key Models Introduced by Indaws Core

| Module | Model | Purpose |
|--------|-------|---------|
| `indaws_api` | `api.access_token` | REST API authentication tokens |
| `indaws_api` | `api.endpoint` | REST API endpoint configuration |
| `indaws_sale_connector` | `ecommerce.offer` | Ecommerce product offers (multi-platform) |
| `indaws_sale_connector` | `indaws.import.connection` | Import data connections |
| `indaws_data_import` | `indaws.import.connection` | Data import connection config |
| `indaws_data_import` | `indaws.import.mapping` | Field mapping for imports |
| `indaws_data_import` | `indaws.import.group` | Import grouping |
| `indaws_data_import` | `indaws.import.parser` | Data parsing config |
| `indaws_data_import` | `indaws.import.encoding` | Encoding settings |
| `indaws_sale` | `product.pricelist.formula` | Advanced pricelist formulas |
| `indaws_product` | `product.brand` | Product brands |
| `indaws_storage` / `indaws_storage_s3` | (extends `ir.attachment`) | S3/cloud storage config |

### Migration Patterns for Indaws Core
- Indaws Core modules are **standardized across clients** for v18+, unlike per-client custom modules in v14-v17
- Verify that the Core repo version matches between staging and production
- Some Core modules have demo data - do NOT migrate demo records
- `connector-sale-odoo` replaces older custom connectors (e.g. `prestashop_connector`); when migrating from v17 to v18+, the connector models change completely
- `indaws_api` endpoints and tokens are environment-specific - copy config structure but verify URLs point to production
- `indaws_data_import` connections may reference staging FTP/SFTP hosts - flag for update

### Vertical Modules (Optional)
The `vertical-odoo-inbuilt` repository provides construction-industry modules:
- `inbuilt_project_certification` - Project certifications
- `inbuilt_sale_measurement` - Measurements
- `inbuilt_fiebdc3_import` - FIEBDC3 format import
- `inbuilt_sale_margin_project` - Project margin tracking

These are only present for clients in the construction vertical.

---

## Permission Issues

| Problem | Solution |
|---------|----------|
| `Cannot access records 'X' (x_model_name)` | Create `ir.model.access` with `group_id=NULL` for `x_*` models |
| `Auth returned uid=False` | Reset password: `UPDATE res_users SET password='admin' WHERE id=2;` |
| Database not in selector | Change owner: `ALTER DATABASE mydb OWNER TO odoo;` |
| Database does not load on connect | `curl` to `/web/login` and wait 30s for initialization |

---

## Record Creation Issues

| Problem | Solution |
|---------|----------|
| Carriers without `product_id` | Create a "Delivery" service product as fallback |
| Sendcloud carriers fail validation | Force `delivery_type='fixed'` during migration |
| `x_studio_*` FK violation | Resolve by name in target model, skip if no mapping |
| Pricelists not found by name | Names in JSONB (v17+) - use `ilike` instead of `=` |
| Automations created active | ALWAYS create with `active=False` |
| `product.attribute.value` constraint violation | Deactivate automations on that model BEFORE migrating |

---

## Correct Execution Order

```
1. Restore backup in production
2. ALTER TABLE fixes (if needed)
3. Run -u all
4. Identify + install missing custom modules
5. import_studio.py (models -> inverse m2o -> fields -> ACLs)
6. migrate_studio_views.py (views -> actions -> menus)
7. Reset admin password
8. Deactivate automations on product.attribute.value
9. Clear id_mapping.json
10. Start Odoo, wait for DB to load
11. run_migration.py --live (all active phases)
12. copy_credentials.py (real values from staging)
13. Migrate extra data (taxes, rules, templates, reports, partners)
14. Deactivate ALL automations
15. Run audit (audit.py)
```

---

## Models Requiring Special Migration

| Model | Particularity |
|-------|--------------|
| `product.product` | Studio fields delegated from `product.template` |
| `res.users` | Studio fields delegated from `res.partner` |
| `ftp.instance` | No `name` field - identify by `host` |
| `prestashop.instance` | No `name` field - identify by `url` |
| `ir.actions.server` | Read in batches; `active` field may not exist |
| `base.automation` | `sequence` field may not exist; always create inactive |
| `delivery.carrier` | Needs `product_id`; Sendcloud -> force `fixed`; filter invalid fields |
| `x_*` Studio models | Need ACLs and inverse m2o fields created first |
| `api.endpoint` (v18+) | Indaws Core - verify URLs point to production |
| `ecommerce.offer` (v18+) | Indaws connector - linked to product and ecommerce platform |
| `indaws.import.connection` (v18+) | Data import config - verify connection strings |
