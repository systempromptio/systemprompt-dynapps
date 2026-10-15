# STEP 8: POST-MIGRATION, STEP 9: AUDIT, STEP 9.5: SMOKE TESTS, STEP 10: REPORT

## STEP 8: POST-MIGRATION - Deactivate automations, fix permissions

```bash
# Deactivate ALL base.automation
psql $PROD_DB -c "UPDATE base_automation SET active=false;"

# Deactivate custom crons (keep system crons like auto-vacuum)
psql $PROD_DB -c "UPDATE ir_cron SET active=false WHERE id NOT IN (SELECT res_id FROM ir_model_data WHERE module='base');"
```

**Clear asset bundles** (prevents OWL/WYSIWYG loading errors):
```sql
DELETE FROM ir_attachment WHERE url LIKE '/web/assets/%';
```
Then restart Odoo so assets are recompiled.

**CRITICAL**: Automations must be OFF until go-live review. Crons activated by levels:
1. Base (auto-vacuum, fetchmail)
2. Workflow (sale confirmations, purchase)
3. Stock (scheduler, reordering)
4. Integrations (connectors, sync)
5. Fiscal (VeriFactu, SII)

**Additional post-migration checks** (see references/beyond-technical.md):
```sql
-- URLs hardcoded to staging (MUST change to production)
SELECT key, value FROM ir_config_parameter WHERE value LIKE '%staging%' OR value LIKE '%test%' OR value LIKE '%prueba%';
SELECT id, name FROM ir_act_server WHERE code LIKE '%staging%';
SELECT id, name, webhook_url FROM ir_act_server WHERE webhook_url IS NOT NULL AND webhook_url != '';
```

Log: `log_step(8, "POST_MIGRATION", "completed", details="Automations deactivated")`

---

## STEP 9: AUDIT - Compare everything

```bash
python3 scripts/audit.py \
  --staging-url $STAGING_URL --staging-db $STAGING_DB --staging-key $STAGING_KEY \
  --prod-url $PROD_URL --prod-db $PROD_DB --prod-key $PROD_KEY
```

### Custom fields audit per model

After record count audit, verify all custom fields (x_studio_*, x_*) exist in production:

```python
stg_custom_fields = stg.search_read('ir.model.fields', [
    ('name', 'like', 'x_'), ('state', '=', 'manual')
], ['name', 'model', 'ttype', 'relation'])

by_model = defaultdict(list)
for f in stg_custom_fields:
    by_model[f['model']].append(f)

for model, fields in by_model.items():
    prod_fields = prod.search_read('ir.model.fields', [
        ('model', '=', model), ('name', 'like', 'x_'), ('state', '=', 'manual')
    ], ['name', 'ttype'])
    prod_field_names = {f['name'] for f in prod_fields}
    missing = [f for f in fields if f['name'] not in prod_field_names]
    if missing:
        # Check delegation (product.product -> product.template, res.users -> res.partner)
        DELEGATES = {'product.product': 'product.template', 'res.users': 'res.partner'}
        if model in DELEGATES:
            delegate_fields = prod.search_read('ir.model.fields', [
                ('model', '=', DELEGATES[model]), ('name', 'like', 'x_')
            ], ['name'])
            delegate_names = {f['name'] for f in delegate_fields}
            missing = [f for f in missing if f['name'] not in delegate_names]
        if missing:
            log.warning(f"{model}: {len(missing)} custom fields MISSING")
```

**Decision tree:**
- All OK -> STEP 10
- Some models `-N` -> Re-run migrate_extra_data.py for specific models
- PLACEHOLDERs found -> Re-run copy_credentials.py
- `ERR` models -> Module not installed

Log: `log_step(9, "AUDIT", "completed", details="N models OK, M with differences")`

---

## STEP 9.5: SMOKE TESTS

```python
SMOKE_TESTS = [
    {'name': 'Create sale order', 'requires': ['sale'],
     'test': lambda prod: prod.execute('sale.order', 'create', [{
         'partner_id': prod.search('res.partner', [('is_company','=',True)], limit=1)[0],
     }]),
     'cleanup': lambda prod, rec_id: prod.execute('sale.order', 'unlink', [rec_id])},
    {'name': 'Create invoice draft', 'requires': ['account'],
     'test': lambda prod: prod.execute('account.move', 'create', [{
         'move_type': 'out_invoice',
         'partner_id': prod.search('res.partner', [('is_company','=',True)], limit=1)[0],
     }]),
     'cleanup': lambda prod, rec_id: prod.execute('account.move', 'unlink', [rec_id])},
    {'name': 'Create picking', 'requires': ['stock'], ...},
    {'name': 'Open POS session', 'requires': ['point_of_sale'], ...},
    {'name': 'Read delivery carriers', 'requires': ['delivery'], ...},
]

results = []
for test in SMOKE_TESTS:
    installed = prod.search('ir.module.module', [('name','in',test['requires']),('state','=','installed')])
    if not installed:
        results.append((test['name'], 'SKIP', 'Module not installed'))
        continue
    try:
        rec_id = test['test'](prod)
        results.append((test['name'], 'PASS', ''))
        if test.get('cleanup') and rec_id:
            test['cleanup'](prod, rec_id)
    except Exception as e:
        results.append((test['name'], 'FAIL', str(e)[:100]))
```

Log: `log_step(9.5, "SMOKE_TESTS", "completed", details="N passed, M failed, K skipped")`

---

## STEP 10: REPORT - Summary and pending items

Tell the user:
1. **Migration status** - which phases passed/failed
2. **Record counts comparison** - ALL models side by side
3. **PLACEHOLDERs remaining** - any real credentials still missing
4. **Hardcoded IDs** - detected in STEP 0, need manual correction
5. **Still pending manually:**
   - Product import (use `/odoo-importation`)
   - SMTP email configuration
   - Production domain/URL + SSL
   - Filestore copy (logos, reports, product images)
   - VeriFactu / SII verification
   - SMB/FTP paths, DynamoDB/API tables
   - Webhook URLs (Make.com, Prestashop, payment providers)
   - OAuth reconfiguration
   - Re-enable automations one by one after review
   - Activate crons by levels
   - Queue jobs cleanup, test data cleanup
6. **URLs pointing to staging** (detected in STEP 8)
7. **Go-live runbook** (see references/beyond-technical.md):
   - Day 0: DB migrated, DNS, SSL, test SMTP, users login
   - Day 1: Base crons, product import, first real order
   - Day 2-3: Workflow crons, automations, first invoice
   - Week 1: Stock/pricing crons, integrations, production SMTP
   - Week 2: VeriFactu, remaining automations
   - Month 1: Performance review, error log review
8. **Communication to client team** - email with what changed, new credentials, what's disabled
9. **UAT checklist** - for client to validate (login, create SO/PO, POS, Studio fields, reports, carriers, connectors, pricing, email, VeriFactu)
10. **Integration freeze window**:
    - ALL base.automation/ir.cron INACTIVE
    - ir.mail_server debug/disabled, fetchmail.server inactive
    - payment.provider state='disabled'
    - External webhooks/triggers PAUSED
    - Queue jobs cleaned
    - ALL integrations frozen until Day 2-3 of go-live

### Multi-company awareness

If staging has multiple companies, the ENTIRE migration must be company-aware:
```python
COMPANY_MODELS = [
    'account.journal', 'account.tax', 'account.fiscal.position', 'account.account',
    'stock.warehouse', 'stock.location', 'pos.config', 'delivery.carrier',
    'ir.sequence', 'ir.rule', 'ir.default',
]
# For each company: filter by company_id, remap to production company_id
```
If single company: use `company_id=1` everywhere.

### Test migration on a copy (recommended)

Offer to clone production DB and run migration on the copy first (~30 min extra).

### Pending decisions

Throughout migration, items needing human decision are saved in `scripts/logs/pending_decisions.json` and presented in the final report, grouped by severity (BLOCKER/WARNING/INFO).

Log: `log_step(10, "REPORT", "completed", details="Migration report delivered")`
