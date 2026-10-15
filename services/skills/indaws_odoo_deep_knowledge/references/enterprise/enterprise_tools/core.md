# Enterprise Tools — Odoo 18.0 Enterprise (EE)

<!-- verified: v18.0 @ 2026-06 (CE crm_iap_* against odoo18/addons; cross-ref domains: sign, voip, functional/survey, data_tools) -->

> **Scope**: this file is a **GROUP / hub** for standalone "tool" apps. Several of them now own dedicated domains — **do not re-read this file for their internals**, jump to the dedicated domain instead:
>
> | Tool | Lives in domain | This file |
> |---|---|---|
> | Sign (EE digital signatures) | `enterprise/sign/` | summary only (§1) |
> | VoIP (EE SIP softphone) | `enterprise/voip/` | summary only (§2) |
> | Data Cleaning + Data Merge (EE) | `enterprise/data_tools/` | summary only (§3-4) |
> | Survey / Certifications (CE) | `functional/survey/` | summary only (§5) |
> | **Lead Intelligence / IAP (CE)** | **HERE — `enterprise_tools/`** | **§6 + `integrations.md` + `v19_changes.md`** |
>
> **The ONLY thing uniquely documented in this domain is §6 Lead Intelligence (IAP)**: `crm_iap_mine` (Lead Generation), `crm_iap_enrich` (Lead Enrichment), `website_crm_iap_reveal` (visitor → lead). These live in **Community** (`odoo18/addons`), NOT Enterprise. Their full mechanics, IAP-credit model, cross-CRM/website wiring and v18→v19 diffs are in `integrations.md` / `v19_changes.md`. Everything else (§1-5) is a cross-reference stub — see the table above. Source: `odoo18/addons` for IAP; `odoo_enterprise/18` for Sign/VoIP.

## 1. Sign (Digital Signatures)

### Architecture
```
sign.template (PDF template with signature fields)
  ├── attachment_id (M2O ir.attachment — PDF, auto-copied if shared)
  ├── sign_item_ids (O2M sign.item — positioned fields)
  ├── authorized_ids (M2M res.users), group_ids (M2M res.groups)
  ├── tag_ids (M2M sign.template.tag — case-sensitive UNIQUE)
  └── signed_count, in_progress_count (computed stats)

sign.request (Signature request instance)
  ├── template_id (M2O sign.template)
  ├── request_item_ids (O2M sign.request.item — signers)
  ├── state: sent → signed | shared | canceled | expired
  ├── access_token (UUID4 — public access, security-critical)
  ├── completed_document (Binary — final signed PDF)
  ├── reference_doc (Reference — origin: SO, PO, contract, etc.)
  ├── validity (Date — default 6 months, auto-expires via cron)
  └── message, message_cc (Html — custom email content)

sign.request.item (Individual signer task)
  ├── partner_id (M2O res.partner), role_id (M2O sign.item.role)
  ├── state: sent → completed | canceled
  ├── mail_sent_order (Int — sequential signing order)
  ├── access_token (individual), sms_token (SMS auth)
  └── Reassignment: only if role.change_authorized=True AND state='sent'

sign.item (Signature field definition)
  ├── type_id (M2O sign.item.type): signature|initials|text|date|
  │   selection|checkbox|radio|textarea
  ├── responsible_id (M2O sign.item.role — which role fills this)
  ├── page, posX, posY, width, height (Float 0-1 — normalized to page)
  ├── alignment: left|center|right (text types only)
  └── option_ids (M2M sign.item.option — for selection/radio)

sign.item.role (Signer role)
  ├── auth_method: sms | False (extra SMS authentication)
  └── change_authorized (Bool — allow post-send reassignment)
```

### Multi-Signer Sequential Flow
```
Create request → mail_sent_order=1 signers notified immediately
  → ALL order=1 complete → order=2 notified
  → ANY signer refuses → ALL subsequent orders CANCELLED
  → Parallel signing: set all items to same mail_sent_order (default 1)
```

### Crypto & Integrity
- Frame hash = SHA256(signer_email + database_uuid)
- Signature images: Base64 PNG data URLs, processed with PIL transparency fix
- PDF rendering: ReportLab with custom TTF fonts
- Log integrity: `_check_document_integrity()` post-hoc (audit trail, not enforcement)

### Integrations
- `hr_contract_sign`: Auto-fill employee contracts
- `sale_subscription_sign`: Subscription documents
- `documents_sign`: DMS integration
- v19: `sign` depends on `certificate` module (crypto enhancements)

## 2. VoIP (SIP Softphone)

### Architecture
```
voip.call (Phone call record)
  ├── phone_number (Char — E.164 recommended)
  ├── direction: incoming | outgoing
  ├── state: aborted|calling|missed|ongoing|rejected|terminated
  ├── start_date, end_date (Datetime)
  ├── partner_id (M2O — auto-matched via phone_mobile_search)
  └── user_id (M2O — handler)

voip.provider (VoIP config per company)
  ├── ws_server (Char — WebSocket URL, default ws://localhost)
  ├── pbx_ip (Char — PBX IP address)
  └── mode: demo | prod

res.users.settings (extended — user VoIP config)
  ├── voip_username, voip_secret, voip_provider_id
  ├── external_device_number (desk phone routing)
  ├── how_to_call_on_mobile: ask | voip | phone
  └── should_call_from_another_device (Bool)
```

### Phone Matching
- Short numbers (<3 digits): exact match on phone/mobile
- International (+, 00): normalized `phone_mobile_search`
- US numbers starting "1": treated as +1
- Partner auto-created/linked after call state change

### Integrations
- `voip_crm`: CRM next activity creation from calls
- `voip_onsip`: OnSIP provider integration
- Click-to-call on any phone field in Odoo

## 3. Data Cleaning

### Architecture
```
data_cleaning.model (Config per target model)
  ├── res_model_id (M2O ir.model)
  ├── cleaning_mode: manual | automatic (auto-validates)
  ├── rule_ids (O2M data_cleaning.rule)
  └── notify_user_ids, notify_frequency

data_cleaning.rule (Cleaning action)
  ├── field_id (M2O ir.model.fields)
  ├── action: trim|remove_accents|format_phone|upper|lower|replace_char
  ├── search_value, replace_value (for replace_char)
  └── sequence (order)

data_cleaning.record (Individual record to clean)
  ├── res_id, res_model_name, field_id, rule_ids
  ├── current_value, suggested_value (computed)
  ├── action_validate() — apply + delete
  └── action_discard() — archive (active=False)
```

### Batch Processing
- Automatic mode: 5,000 records per batch
- Manual mode: 50,000 records per batch
- `batch_commits=True` enables `env.cr.commit()` per batch
- Phone formatting uses country_id from record or company

## 4. Data Merge (Deduplication)

> Engine models physically live in the **`data_cleaning`** module dir: `data_cleaning/models/data_merge_model.py`, `data_merge_group.py`, `data_merge_record.py`, `data_merge_rule.py`. Citations below use those paths.

### Architecture
```
data_merge.model (Dedup config)
  ├── res_model_id, domain (filter eligible records)
  ├── removal_mode: archive | delete (post-merge)
  ├── merge_mode: manual | automatic
  ├── rule_ids (O2M data_merge.rule — match criteria)
  ├── merge_threshold (Int % — default 75, for auto-merge)
  ├── create_threshold (Int % — default 0, for suggestions)
  └── mix_by_company (Bool — cross-company dedup)

data_merge.rule
  ├── field_id (M2O ir.model.fields)
  └── match_mode: accent | standard (unaccent if standard)

data_merge.group (Duplicate group)
  ├── record_ids (O2M data_merge.record)
  ├── similarity (Float 0-1 — matching char fields / total char fields)
  ├── divergent_fields (Char — fields that differ)
  ├── _elect_method() — picks master (default: oldest by create_date)
  └── _merge_method() — redirect FK + archive/delete

data_merge.record
  ├── is_master (Bool), is_discarded (Bool)
  ├── differences, field_values (computed)
  └── _update_foreign_keys() — raw SQL FK redirect
```

### Merge Process
```
find_duplicates() → creates groups+records based on rules
  → Similarity scoring: ONLY char/text fields compared
  → Unaccent: lower(unaccent(field)) comparison
  → _elect_method() → picks master (extensible per model)
  → _merge_method() → redirect all FKs via raw SQL UPDATE → archive/delete
  → Requires PostgreSQL unaccent extension
```

## 5. Survey (Surveys/Certifications)

### Key Features
- **Question types**: simple_choice, multiple_choice, free_text, text_box, numerical_box, date, datetime, matrix, NPS
- **Scoring**: per-answer points, certification pass threshold, badges
- **Conditional display**: `is_conditional` + `triggering_answer_ids`
- **Time limits**: `is_time_limited` + `time_limit` (minutes)
- **Live sessions**: presenter mode with audience sync
- **NPS**: Net Promoter Score calculation (promoters - detractors)
- **Certification**: pass_mark %, badge assignment, max_attempts

### Integrations
- `hr_recruitment_survey`: Interview/assessment surveys
- `event_booth_sale`: Event booth registration
- `survey_crm`: Survey → CRM lead pipeline

## 6. Lead Intelligence (IAP-based) — THE UNIQUE CONTENT OF THIS DOMAIN

> Source verified `odoo18/addons`. All three are **CE, LGPL-3, `auto_install=True`**, depend on `iap_crm` + `iap_mail` (NOT directly on `crm` — `iap_crm` pulls in `crm`). All hit the same Odoo IAP backend `https://iap-services.odoo.com` and consume the **`reveal`** IAP service credits.

### Modules
- `crm_iap_mine` ("Lead Generation", v1.2): on-demand `crm.iap.lead.mining.request` wizard — generate N leads/opps by country/industry/size/contacts. Also ships the shared catalogs `crm.iap.lead.industry/role/seniority` and the `crm.iap.lead.helpers` model reused by reveal.
- `crm_iap_enrich` ("Lead Enrichment", v1.1): auto-fill an existing `crm.lead` from its email **domain**. Adds `iap_enrich_done`, `show_enrich_button` to `crm.lead`. Has `post_init_hook='_synchronize_cron'` + hourly cron.
- `website_crm_iap_reveal` ("Lead Generation From Website Visits", v1.1): IP-based visitor → lead. Depends additionally on `crm_iap_mine` + `website_crm`. Adds `crm.reveal.rule`, `crm.reveal.view`, daily cron.

### Mining (crm_iap_mine) — pull model
```
crm.iap.lead.mining.request (draft → done | error)
  ├── search_type: companies | people (people = companies + contacts)
  ├── lead_number ≤ MAX_LEAD=200; contact_number ≤ MAX_CONTACT=5
  ├── CREDIT_PER_COMPANY=1, CREDIT_PER_CONTACT=1
  │     total = lead_number + (contact_number * lead_number)   ← shown in blue tooltip
  ├── filters: country_ids/state_ids (states whitelisted per country, _STATES_FILTER_COUNTRIES_WHITELIST),
  │            industry_ids, company_size_min/max, role/seniority
  └── action_submit() → _perform_request() → iap_jsonrpc(.../api/dnb/1/search_by_criteria)
        → credit_error → state='error', error_type='credits'  (NO exception raised)
        → no data    → error_type='no_result'
        → data       → _create_leads_from_response() (batch create + message_post 'iap_mail.enrich_company')
```

### Enrichment (crm_iap_enrich) — email-domain model
```
On crm.lead create → if ir.config_parameter 'crm.iap.lead.enrich.setting'=='auto' (default 'auto')
   → cron 'crm_iap_enrich.ir_cron_lead_enrichment'._trigger()   (hourly, runs as root)
_iap_enrich_leads_cron(): leads where NOT done, NOT reveal_id, prob<100/False, create_date>now-1h
   → iap_enrich(): batches of 50, row-lock FOR UPDATE NOWAIT
      → skip generic mail providers (iap_tools._MAIL_PROVIDERS) → mark done + notfound message
      → iap.enrich.api._request_enrich({lead_id: email_domain})
      → InsufficientCreditError → notify creator, STOP all batches
      → _iap_enrich_from_response(): only fills EMPTY fields (partner_name, reveal_id, street, city, zip,
        phone, mobile, country_id, state_id) + 'iap_mail.enrich_company' note
   → env.cr.commit() per batch (avoid losing credits on rollback)
```

### Reveal (website_crm_iap_reveal) — IP model
```
ir.http._serve_page() [public users only, 200 only]:
   → only if website.visitor has NO lead yet
   → geoip country/state → crm.reveal.view._create_reveal_view() (INSERT ... ON CONFLICT DO NOTHING)
   → matched rule ids stored in 'rule_ids' cookie to avoid re-processing same visitor
crm.reveal.rule (sequence-ordered, cached via @ormcache _get_active_rules):
   filters: regex_url, country/state, website_id, industry, size, role/seniority
daily cron _process_lead_generation():
   → _unlink_unrelevant_reveal_view (skip IPs that already produced a lead in last reveal.lead_month_valid=6 months)
   → batches of DEFAULT_REVEAL_BATCH_LIMIT=25 IPs → iap_jsonrpc(.../iap/clearbit/1/reveal)
   → credit_error → _notify_no_more_credit (one-shot via 'reveal.already_notified' param)
   → _create_lead_from_response(): dedup on reveal_id (clearbit_id); stamps reveal_ip, reveal_iap_credits
crm.reveal.view cleanup: not_found views older than reveal.view_weeks_valid=5 weeks unlinked
```

### Shared plumbing
- `crm.iap.lead.helpers.lead_vals_from_response()` builds the `crm.lead` vals (used by BOTH mining and reveal).
- `crm.iap.lead.helpers._notify_no_more_credit()` emails the record creators **once** (guarded by an `ir.config_parameter` flag) — see ETG33.
- IAP account resolved via `self.env['iap.account'].get('reveal')`; buy-credits URL via `iap.account.get_credits_url(service_name='reveal')`.
- All three add their FK to `crm.lead._merge_get_fields()` so CRM lead merge carries mining/reveal provenance.

See `integrations.md` for the cross-CRM/website wiring and `v19_changes.md` for v18→v19 diffs.
