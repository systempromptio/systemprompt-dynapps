# Enterprise Tools — Integrations

<!-- verified: v18.0 @ 2026-06 (crm_iap_* against odoo18/addons; cross-ref domains verified to exist) -->

This domain is a **hub**. Most "tools" own a dedicated domain — this file documents only the wiring that is **unique to Lead Intelligence / IAP** (mining, enrichment, reveal × CRM/website/IAP) plus a cross-reference table telling you where everything else actually lives.

---

## 1. Cross-reference map — "this lives in domain X"

| Topic / model | Canonical domain | What enterprise_tools keeps |
|---|---|---|
| Sign templates/requests, signers, hash chain, SMS auth | `enterprise/sign/` | `core.md §1` summary + ETG1-10,23-26 |
| VoIP softphone, providers, phone matching, OnSIP | `enterprise/voip/` (+ `providers.md`, `softphone.md`) | `core.md §2` summary + ETG11-12,27-28 |
| Data Cleaning (format rules), Data Merge (dedup), 5 merge bridges | `enterprise/data_tools/` | `core.md §3-4` summary + ETG13-20,29-32 |
| Survey, certifications, NPS, live sessions | `functional/survey/` | `core.md §5` summary |
| **Lead Generation / Mining (`crm_iap_mine`)** | **`enterprise_tools/` (HERE)** | full — `core.md §6`, this file, `v19_changes.md` |
| **Lead Enrichment (`crm_iap_enrich`)** | **`enterprise_tools/` (HERE)** | full |
| **Website visitor Reveal (`website_crm_iap_reveal`)** | **`enterprise_tools/` (HERE)** | full |
| IAP account/credits/`iap.enrich.api` base mechanics | `iap`, `iap_crm`, `iap_mail` (CE base modules) | referenced from §6 |
| `website.visitor` lifecycle | `functional/website` (if present) / CRM | reveal consumes it (§3) |

> Rule of thumb: if you need internals of Sign/VoIP/Data/Survey, **stop reading here and open that domain**. This file only goes deep on the three `crm_iap_*` modules.

---

## 2. Module / dependency graph (Lead Intelligence)

```
                 iap (CE)  ── iap.account, iap.service, iap_tools.iap_jsonrpc,
                  │           InsufficientCreditError, _MAIL_PROVIDERS
        ┌─────────┼─────────┐
        ▼                   ▼
   iap_mail (iap+mail)   iap_crm (crm+iap_mail)   ← both auto_install
        │  enrich_company        │  iap.enrich.api
        └──────────┬─────────────┘
                   │ depends (auto_install all three)
   ┌───────────────┼───────────────────────────┐
   ▼               ▼                            ▼
crm_iap_mine   crm_iap_enrich            website_crm_iap_reveal
(pull wizard)  (email-domain auto)       (IP, depends crm_iap_mine + website_crm)
   │  crm.iap.lead.helpers  ◄───────────────────┘  (reuses lead_vals_from_response,
   │  crm.iap.lead.industry/role/seniority           _notify_no_more_credit)
   ▼
crm.lead  ← all three extend it (+ _merge_get_fields so merges carry provenance)
```

Key fact: the three lead-intel modules depend on `iap_crm` + `iap_mail`, **not** on `crm` directly — `crm` arrives transitively. All are `auto_install=True`, so installing CRM + a website + IAP can pull them in automatically.

---

## 3. How the tools cross into CRM / website / IAP

### 3.1 Three ways a `crm.lead` gets created/enriched from IAP

| Path | Trigger | Model entry point | IAP endpoint | Credits |
|---|---|---|---|---|
| **Mining** (pull) | user runs wizard | `crm.iap.lead.mining.request.action_submit` | `/api/dnb/1/search_by_criteria` | `lead_number + contact_number*lead_number` |
| **Enrichment** (push on create) | `crm.lead` created with email | `crm.lead.create → cron → iap_enrich` | `iap.enrich.api._request_enrich` | 1 per enriched domain |
| **Reveal** (push on page view) | public visitor hits a page | `ir.http._serve_page → crm.reveal.view` → daily cron | `/iap/clearbit/1/reveal` | per company + per extra contact |

All three converge on `crm.iap.lead.helpers.lead_vals_from_response()` to shape the `crm.lead` vals, and all post the `iap_mail.enrich_company` chatter template with the returned company data.

### 3.2 CRM lead merge integration
Each module appends its FK to `crm.lead._merge_get_fields()` so that merging duplicate leads preserves provenance:
- `crm_iap_mine` → `lead_mining_request_id`
- `website_crm_iap_reveal` → `reveal_ip`, `reveal_iap_credits`, `reveal_rule_id`
- `crm_iap_enrich` → overrides `_merge_get_fields_specific()` so `iap_enrich_done` becomes `any(...)` across merged leads (a merged lead is "done" if ANY source was done).

> This is the integration point with the **`data_tools` / `data_merge_crm`** bridge: `data_merge_crm` redirects FKs and uses `crm.lead._merge_method=merge_opportunity`. Those provenance fields ride along through that merge too. See `enterprise/data_tools/integrations.md` (DTG20: won opportunities can't be merged).

### 3.3 Website integration (reveal only)
- Hooks `ir.http._serve_page` — runs on the **frontend page controller**, gated to public users + HTTP 200.
- Uses `website.visitor._get_visitor_from_request()` to skip visitors that already have a lead (avoids double counting vs `website_crm` form submissions).
- Stores matched rule ids in a browser cookie `rule_ids` (type `optional`, cookie consent aware) to avoid re-revealing the same visitor on every page.
- `crm.reveal.rule.regex_url` + `website_id` scope which pages/sites a rule watches. Rules are cached via `@ormcache` (`_get_active_rules`) and the cache is cleared on rule create/write(country/url/active)/unlink.
- Requires working **GeoIP** (country/state from `request.geoip`) — no GeoIP, no reveal.

### 3.4 IAP credits & accounts (shared backend)
- All three resolve the account via `self.env['iap.account'].get('reveal')` — the **same `reveal` service** funds mining, enrichment AND reveal. One credit pool.
- "Buy credits" links: mining wizard `action_buy_credits()` → `iap.account.get_credits_url(service_name='reveal')`.
- Credit-exhaustion handling differs per module (see gotchas): mining → `state='error'` no exception (ETG via core §6); enrich → notify + abort run (ETG38); reveal → one-shot notify via `reveal.already_notified` (ETG33).
- `iap_tools._MAIL_PROVIDERS` (generic email domains) is shared — enrichment skips them to save credits (ETG37).

---

## 4. Config & system parameters (Lead Intelligence)

| Param / setting | Default | Effect |
|---|---|---|
| `ir.config_parameter crm.iap.lead.enrich.setting` | `auto` | read by `crm.lead.create` to auto-trigger enrich cron (ETG34/35) |
| Settings → CRM `lead_enrich_auto` | follows cron.active | toggles cron `crm_iap_enrich.ir_cron_lead_enrichment` (NOT the param above — ETG35) |
| cron `crm_iap_enrich.ir_cron_lead_enrichment` | hourly | `model._iap_enrich_leads_cron()` (leads <1h old in v18) |
| cron `website_crm_iap_reveal.ir_cron_crm_reveal_lead` | daily | `model._process_lead_generation()` |
| `ir.config_parameter reveal.endpoint` | `https://iap-services.odoo.com` | override IAP base URL (mining + reveal) |
| `ir.config_parameter reveal.lead_month_valid` | `6` | months an IP is "burned" after producing a lead |
| `ir.config_parameter reveal.view_weeks_valid` | `5` | age before `not_found` reveal views are purged |
| `ir.config_parameter reveal.already_notified` | `False` | one-shot credit-exhaustion notify guard (ETG33) |
| constants `MAX_LEAD=200`, `MAX_CONTACT=5` | — | mining wizard caps (onchange-clamped) |
| `crm.reveal.rule` SQL constraint | — | `extra_contacts` must be 1..5 |

---

## 5. Integration gotchas (ETG quick list)

Full text in `gotchas.md`:
- **ETG33** — credit-exhaustion email is one-shot (reveal `already_notified`).
- **ETG34/35** — enrich auto-mode is ON by default and governed by two different switches.
- **ETG36** — enrich never overwrites populated fields.
- **ETG37** — generic email domains skipped (saves credits, looks "processed").
- **ETG38/39** — enrich aborts whole run on credit exhaustion + commits per batch.
- **ETG40** — reveal only for public visitors on 200 pages with GeoIP.
- **ETG41** — reveal dedups per-IP (6 months) and per clearbit_id (forever).
- Cross-domain: provenance FKs survive `data_merge_crm` merges (see `data_tools/integrations.md`).
