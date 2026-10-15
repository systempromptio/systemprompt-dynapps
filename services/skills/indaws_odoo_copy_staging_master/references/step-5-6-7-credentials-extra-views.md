# STEP 5: CREDENTIALS, STEP 6: EXTRA DATA, STEP 7: VIEWS

## STEP 5: COPY CREDENTIALS

```bash
python3 scripts/copy_credentials.py \
  --staging-url $STAGING_URL --staging-db $STAGING_DB --staging-key $STAGING_KEY \
  --prod-url $PROD_URL --prod-db $PROD_DB --prod-key $PROD_KEY
```

Verify output shows **0 PLACEHOLDERs** remaining.

Log: `log_step(5, "CREDENTIALS", "completed", details="N credentials copied, 0 PLACEHOLDERs")`

---

## STEP 6: MIGRATE EXTRA DATA

```bash
python3 scripts/migrate_extra_data.py \
  --staging-url $STAGING_URL --staging-db $STAGING_DB --staging-key $STAGING_KEY \
  --prod-url $PROD_URL --prod-db $PROD_DB --prod-key $PROD_KEY
```

### Models migrated:
- `account.tax`, `account.account` (custom chart)
- `ir.rule` (security rules with group resolution by xml_id)
- `ir.model.access` (custom ACLs)
- `mail.template`, `ir.actions.report` (custom QWeb reports)
- `ir.actions.server` (server actions, read in batches of 10)
- `base.automation` (automated actions, created ALWAYS inactive)
- `ir.cron` (scheduled actions, created ALWAYS inactive)
- `ir.sequence` + `ir.sequence.date_range` (custom sequences)
- `ir.mail_server` (outgoing SMTP servers)
- `fetchmail.server` (incoming IMAP/POP, created ALWAYS inactive)
- `payment.provider` (created ALWAYS with `state='disabled'`)
- `automatic.pricing`, `sales.indices`, partner companies

### Payment provider rules:
- Match by `code` field (e.g. `redsys`, `stripe`, `paypal`)
- Copy configuration fields but set `state='disabled'`
- For Redsys: copy merchant config as PLACEHOLDER values
- Skip `payment.token` records (customer-specific)

### Sequence migration rules:
- Only migrate sequences where `ir.model.data` has NO xml_id OR xml_id belongs to custom module
- Match by `code` field — do NOT create duplicates
- Copy `prefix`, `suffix`, `padding`, `number_increment`, `number_next_actual`
- For `ir.sequence.date_range`: copy all ranges linked to migrated sequences

### Mail server rules:
- `ir.mail_server`: copy config but set `smtp_pass` to `'PLACEHOLDER_SMTP_PASS'`
- `fetchmail.server`: create INACTIVE, user enables after verifying credentials

Log: `log_step(6, "EXTRA_DATA", "completed", details="N records migrated across M models")`

---

## STEP 7: MIGRATE VIEWS (Studio + Custom + Website)

```bash
python3 scripts/migrate_views.py \
  --staging-url $STAGING_URL --staging-db $STAGING_DB --staging-key $STAGING_KEY \
  --prod-url $PROD_URL --prod-db $PROD_DB --prod-key $PROD_KEY
```

### 7a. Studio views
Migrates: `ir.ui.view`, `ir.actions.act_window`, `ir.ui.menu`, `ir.model.access` from Studio.
**Run TWICE**: first pass creates most records, second pass catches dependencies.

### 7b. Custom views (non-Studio, non-module)

Migrates `ir.ui.view` records that:
- Have NO `xml_id` in `ir.model.data` (not owned by any installed module)
- Are NOT Studio views (no `x_studio` in key)
- Are NOT website views (handled in 7c)

**Detection query:**
```python
all_views = stg.search_read('ir.ui.view', [], ['id', 'name', 'key', 'model', 'type', 'inherit_id', 'mode', 'active'])
module_views = stg.search_read('ir.model.data', [('model', '=', 'ir.ui.view'), ('module', '!=', '__export__')], ['res_id', 'module'])
module_view_ids = {r['res_id'] for r in module_views}
custom_views = [v for v in all_views if v['id'] not in module_view_ids and 'x_studio' not in (v.get('key') or '')]
```

Match in production by `key` field. If `key` is NULL, match by `name` + `model` + `type`.

### 7c. Website content

**Migration order:**
1. `website` — match by `domain` or `id=1`. Migrate settings: `homepage_url`, `custom_code_head/footer`, `robots_txt`, social links, CDN config, analytics keys
2. `ir.ui.view` (website) — views with `website_id` set. Match by `key` field (COW system). Include pages, snippet customizations, header/footer overrides, theme toggles
3. `website.page` — after their `ir.ui.view` exists. Match by `url`. Copy `is_published`, `date_publish`, `website_indexed`, header/footer visibility
4. `website.menu` — match by `name` + parent path. Resolve `page_id` by matching `website.page.url`. Copy `sequence`, `url`, `new_window`, `mega_menu_content`
5. `website.rewrite` — URL redirects. Match by `url_from`. Copy `url_to`, `redirect_type`
6. `ir.asset` (website) — website-specific CSS/JS overrides. Match by `key`
7. `website.snippet.filter` — custom dynamic snippet data sources. Match by `name`

**Website migration rules:**
- Skip views where `theme_template_id` is set AND same theme installed in production
- For COW views: if production already has view with same `key` + `website_id`, UPDATE `arch_db`
- `ir.attachment` with `website_id`: migrate only if referenced in page `arch` content
- Mega menu HTML content may contain staging URLs — flag for review

Log: `log_step(7, "VIEWS", "completed", details="N studio views, M custom views, K website pages, J menus migrated")`
