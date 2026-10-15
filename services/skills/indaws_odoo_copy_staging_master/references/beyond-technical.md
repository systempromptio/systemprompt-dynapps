# Beyond the Technical Migration

## What NOBODY thinks about until it explodes in production

### 1. Hardcoded URLs pointing to staging
Search the ENTIRE database for URLs containing "staging", "test", "dev", "preprod":
```sql
-- In ir.config_parameter
SELECT key, value FROM ir_config_parameter WHERE value LIKE '%staging%' OR value LIKE '%test%' OR value LIKE '%preprod%';

-- In server actions (Python code)
SELECT id, name FROM ir_act_server WHERE code LIKE '%staging%' OR code LIKE '%test%';

-- In mail templates
SELECT id, name FROM mail_template WHERE body_html LIKE '%staging%';

-- In automated actions
SELECT id, name FROM base_automation WHERE action_server_ids IN (SELECT id FROM ir_act_server WHERE code LIKE '%staging%');

-- In webhooks
SELECT id, name, webhook_url FROM ir_act_server WHERE webhook_url LIKE '%staging%';
```

### 2. Webhooks and external callbacks
External services that call Odoo (POST/GET):
- Make.com/Zapier scenarios pointing to staging URL
- Ecommerce webhooks (Prestashop, WooCommerce) notifying staging
- Payment provider callbacks (Redsys, Stripe, PayPal) with staging return URL
- Marketplace callbacks (Amazon MWS/SP-API, Miravia)
- Helpdesk/support tools (Zendesk, Freshdesk) calling Odoo

**Action**: List all webhooks and update URLs to production.

### 3. Emails that will fire automatically
When activating automations in production:
- Emails may be sent to REAL customers (templates from staging with test data)
- Stock/replenishment notifications may fire
- Marketing campaigns may activate

**Action**: Before enabling SMTP:
1. Configure `ir.mail_server` with `smtp_debug = True` first
2. Or use a test service (Mailtrap) before going to real production SMTP
3. Verify ALL `mail.template` records that have `auto_delete=True`

### 4. Crons that trigger expensive operations
When activating crons:
- Automatic pricing cron may reprice 100K+ items
- Stock B2B cron may generate picking orders
- Ecommerce sync cron may attempt to synchronize
- Tax reporting cron (e.g. Veri*Factu, SII) may send data to tax authorities

**Action**: Use a progressive activation plan (already built into the skill).

### 5. Demo/test data in the production database
The production dump may contain:
- Test orders
- Test invoices
- Partners named "TEST", "DEMO", "temp"
- Test products
- Test emails in the mail queue

**Action**: Post-migration cleanup script:
```sql
-- Search for test data
SELECT id, name FROM res_partner WHERE name ILIKE '%test%' OR name ILIKE '%demo%';
SELECT id, name FROM sale_order WHERE name ILIKE '%test%';
-- DO NOT delete automatically - only list and let the user decide
```

### 6. Filestore: which files are necessary
Not all of the filestore is needed. Classify:
- **Critical**: company logos, report layouts, PDF templates
- **Important**: product images (but usually imported with products)
- **Unnecessary**: test attachments, old emails, thumbnails

**Action**: Copy only the necessary directories, not the entire filestore (which can be 50GB+).

### 7. Multi-company
If the client has multiple companies:
- Each company has its own accounting, journals, taxes
- Security rules filter by company
- Users have access to one or more companies
- Sequences are per-company

**Action**: The migration must iterate per company, not assume `company_id=1`.

### 8. Custom translations
If the client has:
- Renamed fields ("Internal Reference" -> "SKU")
- Menus in another language
- Translated stages/steps
- Payment terms with custom names

**Action**: Migrate `ir.translation` (v14-v16) or verify JSONB translations (v17+).

### 9. User default values (ir.default)
Each user may have different defaults:
- Default warehouse
- Default journal
- Default company
- Default pricelist

**Action**: Migrate `ir.default` with `user_id` resolution.

### 10. Saved filters and favorites
Users have:
- Saved search filters on list views
- Custom dashboards
- Menu favorites

**Action**: Migrate `ir.filters` (or `ir.ui.view.custom` for dashboards).

### 11. OAuth / External authentication
If the client uses:
- Google OAuth (calendar, contacts, drive)
- Microsoft Azure AD
- LDAP
- 2FA (TOTP)

**Action**: OAuth credentials cannot be copied (tokens expire). Reconfigure from scratch.

### 12. Documents and Knowledge
If the client uses:
- `documents.folder` + `documents.document` (DMS)
- `knowledge.article` (Knowledge base)
- `spreadsheet.dashboard` (Dashboards)

**Action**: Decide whether to migrate content or only structure.

### 13. Backups and disaster recovery
BEFORE touching production:
- Back up the current production database
- Document how to rollback
- Have a Plan B if migration fails

### 14. Performance baseline
BEFORE migration, measure:
- Load time of `/web`
- Time for a `search` on `product.product`
- Time to create a sale order
- Server RAM/CPU usage

AFTER migration, compare. If performance degraded, investigate.

### 15. Team communication
Generate an email for the client's team:
- What changed in production
- New credentials (if applicable)
- Which modules were activated
- Which functions are temporarily disabled (automations)
- Who to contact if something fails

### 16. Post go-live activation plan (runbook)

```
DAY 0 (go-live):
  [x] Database migrated and verified
  [x] DNS pointing to production
  [x] SSL verified
  [x] Test SMTP (Mailtrap)
  [x] Users logged in OK
  [ ] Automations disabled
  [ ] Crons disabled

DAY 1:
  -> Activate base crons (auto-vacuum, queue)
  -> Verify login works for all users
  -> Import products (if not done before)

DAY 2-3:
  -> Activate workflow crons
  -> Activate sales automations
  -> First real order
  -> Verify invoicing

WEEK 1:
  -> Activate stock/pricing crons
  -> Activate stock automations
  -> Activate integrations (ecommerce, marketplaces)
  -> Switch SMTP to real production

WEEK 2:
  -> Activate tax reporting (Veri*Factu, SII, etc.)
  -> Activate everything else
  -> Close pending issues

MONTH 1:
  -> Performance review
  -> Error log review
  -> Adjust failing automations
  -> Document post go-live changes
```

### 17. Smart detection of custom models
Never assume which models the client has. Detect ALL dynamically:

```python
# Non-standard models in staging
custom_models = stg.search_read('ir.model', [
    ('transient', '=', False),
    ('model', 'not like', 'ir.%'),
    ('model', 'not like', 'base%'),
    ('model', 'not like', 'bus.%'),
    ('model', 'not like', 'mail.%'),
], ['model', 'name'])

# Filter: which ones have data?
for m in custom_models:
    try:
        count = stg.search_count(m['model'], [])
        if count > 0:
            print(f"{m['model']}: {count} records")
    except:
        pass
```

### 18. Computed fields that need recomputation
After migrating, some computed/stored fields may have incorrect values:
- `product.product.qty_available` (stock)
- `sale.order.amount_total` (totals)
- `account.move.payment_state` (payment state)

**Action**: Force recompute after migrating data:
```python
# Via shell or server action
env['product.product'].search([])._compute_quantities()
```

### 19. Pending queue jobs
If staging has pending queue jobs:
- Failed jobs that will retry in production
- Sync jobs that point to staging URLs

**Action**: Clean `queue.job` records with `state != 'done'` before go-live.

### 20. Licenses and subscription
Verify:
- Does the Enterprise license cover the number of users?
- Is the Odoo.sh plan (if applicable) sufficient?
- Do third-party modules have valid licenses?

---

## Indaws Core Post-Migration Checks (v18+)

When the client uses Indaws Core modules, perform these additional checks after migration:

### API Endpoints (`indaws_api`)
- Verify `api.endpoint` records point to production URLs, not staging
- Verify `api.access_token` records are valid or regenerated for production
- Check that endpoint methods (GET/POST/PUT/DELETE) match the production API spec

### Ecommerce Connectors (`connector-sale-odoo`)
- Verify `ecommerce.offer` records reference correct products in production
- Check `indaws.import.connection` settings (URLs, credentials) point to live ecommerce platforms
- Verify import mappings (`indaws.import.mapping`) are compatible with production product catalog
- Test connector sync with a single product before enabling bulk sync

### Data Import Framework (`indaws_data_import`)
- Verify `indaws.import.connection` credentials and hosts
- Check `indaws.import.group` configurations
- Verify `indaws.import.parser` and `indaws.import.encoding` settings match production data formats

### Storage (`indaws_storage`, `indaws_storage_s3`)
- Verify S3 bucket configuration points to production bucket
- Check access credentials are for the production AWS/cloud account

### Pricelist Formulas (`indaws_sale`)
- Verify `product.pricelist.formula` records have correct formulas for production
- These may reference product categories or brands that need ID resolution

### Localization (Spain-specific, `indaws_l10n_es_*`)
- `indaws_l10n_es_edi_sii`: Verify SII certificate and connection settings
- `indaws_l10n_es_edi_verifactu`: Verify Veri*Factu configuration points to production tax authority
- `indaws_l10n_es_account_ipc`: Verify IPC settings

### Report Engine (`indaws_report_docx`)
- Verify DOCX report templates are present in the filestore
- Check that report configurations reference correct template attachments

---

## Example migration scripts reference

| Template | Phase | Lines | What it demonstrates |
|----------|-------|-------|---------------------|
| `example_00_company.py` | 00 | ~480 | Company, users, groups by xml_id |
| `example_03_carriers.py` | 03 | ~850 | Carriers with product_id, sendcloud fix, channels, workflows |
| `example_08_automations.py` | 08 | ~870 | Server actions batched, automations inactive, crons |
| `example_run_migration.py` | Orchestrator | ~300 | Runs phases in order with dependencies |

> **Note**: These example scripts originate from a v17 migration. Adapt model names, field names, and connector types to the target version.

---

## Post-migration verification checklist

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
11. **Indaws Core config** (v18+): Are API endpoints, connectors, and import connections pointing to production?
12. **Ecommerce offers** (v18+): Are product mappings correct for the live store?
