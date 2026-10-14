# Audit-Only Mode

If the user says "audit migration", "verify migration", or explicitly asks to audit/verify a migration already done (by them or someone else), enter **audit-only mode**. In this mode:

- Skip STEPS 0-8 entirely
- Only ask for staging and production credentials (no local .cfg, no deployment mode, no scope)
- Run a comprehensive comparison across ALL migrated model categories
- Generate a detailed report with findings and remediation actions

## Audit-only questions (ask in a single message)

```
STAGING:
  - URL, Database, User, API Key

PRODUCTION:
  - URL, Database, User, API Key

CONTEXT (optional):
  - Who performed the migration? (for report attribution)
  - Migration date? (to filter records created after that date)
```

## Audit comparison table

| Category | Models | Match by | What to compare |
|----------|--------|----------|-----------------|
| **Configuration** | `res.company` | `name` | Fiscal data, branding fields, special accounts |
| **Users** | `res.users` | `login` | Count, groups assigned, active status |
| **Partners** | `res.partner` | `name` + `email` + `is_company` | Count, company vs contact ratio |
| **Accounting** | `account.journal` | `code` | Count, type, currency |
| | `account.fiscal.position` | `name` | Count, tax mapping count |
| | `account.tax` | `name` + `type_tax_use` | Count, amount, tax group |
| | `account.account` | `code` | Count |
| **Warehouses** | `stock.warehouse` | `code` | Count, routes |
| | `stock.location` | `complete_name` | Count |
| | `stock.picking.type` | `sequence_code` | Count |
| | `stock.route`, `stock.rule` | `name` | Count |
| **Sales & Carriers** | `delivery.carrier` | `name` | Count, delivery_type, has product_id |
| | `delivery.price.rule` | — | Count per carrier |
| **Products (config)** | `product.attribute` | `name` | Count, values count |
| | `product.attribute.value` | `name` + `attribute_id.name` | Count |
| | `product.category` | `complete_name` | Count |
| | `product.pricelist` | `name` | Count |
| **Connectors** | `ftp.instance` | `host` | Count, credentials not PLACEHOLDER |
| | `prestashop.instance` | `url` | Count, credentials not PLACEHOLDER |
| | `smb.instance` | `name` | Count |
| | `aws.instance` | `name` | Count |
| **POS** | `pos.config` | `name` | Count, payment methods |
| | `pos.payment.method` | `name` | Count |
| **Automations** | `ir.actions.server` | `name` | Count, code present |
| | `base.automation` | `name` | Count, active status (should be inactive) |
| | `ir.cron` | `name` | Count, active status (should be inactive) |
| **Studio** | `ir.model` (x_*) | `model` | Count |
| | `ir.model.fields` (x_studio_*) | `model` + `name` | Count per model |
| | `ir.ui.view` (Studio) | `key` | Count |
| | `ir.actions.act_window` (Studio) | `name` | Count |
| | `ir.ui.menu` (Studio) | `name` | Count |
| **Custom views** | `ir.ui.view` (no xml_id, no Studio) | `key` or `name`+`model` | Count, arch content match |
| **Security** | `ir.model.access` (custom) | `name` | Count |
| | `ir.rule` (custom) | `name` | Count, domain_force match |
| **Website** | `website` | `domain` or id=1 | Settings: homepage_url, custom_code_head/footer, robots_txt, social links |
| | `website.page` | `url` | Count, is_published, arch content match |
| | `website.menu` | `name` + `url` | Count, hierarchy, mega_menu_content |
| | `website.rewrite` | `url_from` | Count, redirect_type, url_to |
| | `ir.asset` (website_id) | `key` | Count |
| **Sequences** | `ir.sequence` (custom) | `code` | prefix, suffix, padding, number_next_actual |
| | `ir.sequence.date_range` | parent sequence code + date_from | Count |
| **Mail servers** | `ir.mail_server` | `name` | Count, host, port, encryption |
| | `fetchmail.server` | `name` | Count, active status |
| **Payment** | `payment.provider` | `code` | Count, state (should be disabled) |
| **System config** | `ir.config_parameter` | `key` | Values match (skip `database.uuid`, `database.is_neutralized`) |
| | `ir.default` | `field_id` + `company_id` | Count |
| | `ir.filters` | `name` + `model_id` | Count |
| | `mail.template` | `name` | Count, body_html not empty |
| | `ir.actions.report` | `report_name` | Count |

## Additional audit checks

```python
# 1. PLACEHOLDER scan — find any field containing "PLACEHOLDER" in production
for model in ['ftp.instance', 'prestashop.instance', 'smb.instance', 'aws.instance',
               'ir.mail_server', 'delivery.carrier', 'payment.provider']:
    records = prod.search_read(model, [], [])
    for rec in records:
        for field, value in rec.items():
            if isinstance(value, str) and 'PLACEHOLDER' in value.upper():
                report.add_finding('PLACEHOLDER', model, rec['id'], field, value)

# 2. Staging URL scan — find URLs pointing to staging in production
staging_domain = urlparse(staging_url).netloc
params = prod.search_read('ir.config_parameter', [], ['key', 'value'])
for p in params:
    if staging_domain in str(p.get('value', '')):
        report.add_finding('STAGING_URL', 'ir.config_parameter', p['id'], p['key'], p['value'])

# 3. Automation safety check — ALL base.automation and ir.cron should be inactive
active_automations = prod.search_read('base.automation', [('active', '=', True)], ['name'])
active_crons = prod.search_read('ir.cron', [('active', '=', True)], ['name', 'cron_name'])
# Filter crons: only flag non-base ones
base_crons = prod.search_read('ir.model.data', [('model', '=', 'ir.cron'), ('module', '=', 'base')], ['res_id'])
base_cron_ids = {r['res_id'] for r in base_crons}
custom_active_crons = [c for c in active_crons if c['id'] not in base_cron_ids]

# 4. Asset bundles — check if they were cleared
stale_assets = prod.search_read('ir.attachment', [('url', 'like', '/web/assets/')], ['url'], limit=5)

# 5. Website content integrity — compare arch of top pages
for page in ['/', '/contactus', '/shop']:
    stg_page = stg.search_read('website.page', [('url', '=', page)], ['arch', 'is_published'])
    prod_page = prod.search_read('website.page', [('url', '=', page)], ['arch', 'is_published'])
    if stg_page and prod_page:
        if stg_page[0]['arch'] != prod_page[0]['arch']:
            report.add_finding('CONTENT_MISMATCH', 'website.page', page)
```

## Audit report format

```
MIGRATION AUDIT REPORT
======================
Client: [name]
Date: [date]
Auditor: Claude (odoo-config-migrator audit-only mode)
Staging: [url] / [db]
Production: [url] / [db]

SUMMARY
-------
Total categories audited: N
Categories OK: N (100% match)
Categories with differences: N
Critical findings: N
Warnings: N

RECORD COUNTS
-------------
| Category | Model | Staging | Production | Diff | Status |
|----------|-------|---------|------------|------|--------|
| Accounting | account.tax | 25 | 25 | 0 | OK |
| Accounting | account.journal | 8 | 7 | -1 | MISSING |
| Website | website.page | 12 | 10 | -2 | MISSING |
| ... | ... | ... | ... | ... | ... |

CRITICAL FINDINGS
-----------------
[!] PLACEHOLDER values found in production:
    - ftp.instance #3, field 'password': 'PLACEHOLDER_FTP_PASS'
    - ir.mail_server #1, field 'smtp_pass': 'PLACEHOLDER_SMTP_PASS'

[!] Staging URLs found in production:
    - ir.config_parameter 'web.base.url': 'https://staging.client.com'
    - ir.actions.server #15 'webhook_url': 'https://staging.client.com/hook'

[!] Active automations (should be inactive post-migration):
    - base.automation #8: 'Auto-confirm SO'
    - ir.cron #22: 'Sync Prestashop Stock'

WARNINGS
--------
[~] Asset bundles not cleared (5 stale entries found) — may cause OWL errors
[~] Website page '/about-us' content differs between staging and production
[~] Sequence 'sale.order' next_number differs: staging=1543, production=1

MISSING RECORDS (require action)
--------------------------------
| Model | Staging record | Match field | Action |
|-------|---------------|-------------|--------|
| account.journal | Bank EUR (BEUR) | code=BEUR | Create manually or re-run STEP 6 |
| website.page | /terms-and-conditions | url | Re-run STEP 7c |

REMEDIATION PLAN
----------------
1. [CRITICAL] Replace PLACEHOLDER values with real credentials
2. [CRITICAL] Update staging URLs to production domain
3. [CRITICAL] Verify all automations are inactive before go-live
4. [HIGH] Clear asset bundles: DELETE FROM ir_attachment WHERE url LIKE '/web/assets/%';
5. [HIGH] Migrate missing 2 website pages
6. [MEDIUM] Migrate missing account.journal BEUR
7. [LOW] Align sequence next_number if needed
```

If audit-only mode is invoked, generate the full report and present it to the user. Do NOT execute any migration steps — only report findings and suggest remediation.
