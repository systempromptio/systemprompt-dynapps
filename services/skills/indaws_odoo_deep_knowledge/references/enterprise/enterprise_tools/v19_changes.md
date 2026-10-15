# Enterprise Tools — v19 changes

<!-- verified: v18.0→v19.0 @ 2026-06 (diff odoo18/addons vs odoo19/addons for crm_iap_mine, crm_iap_enrich, website_crm_iap_reveal) -->

Scope: only the modules **uniquely owned by this domain** — the three CE Lead-Intelligence modules `crm_iap_mine`, `crm_iap_enrich`, `website_crm_iap_reveal`. For Sign / VoIP / Data Cleaning+Merge / Survey v19 diffs, see their dedicated domains (`enterprise/sign`, `enterprise/voip`, `enterprise/data_tools/v19_changes.md`, `functional/survey`).

The **architecture is unchanged** v18→v19 (same modules, same deps, same IAP `reveal` service, same crons, same lead-vals helper). Changes are framework/API plus one real behavioural rewrite in enrichment.

---

## Cross-cutting (all three modules)

### Python class names → `Model_Submodel` pattern
`CRMLeadMiningRequest`→`CrmIapLeadMiningRequest`, `CRMHelpers`→`CrmIapLeadHelpers`, `CRMRevealRule`→`CrmRevealRule`, `CRMRevealView`→`CrmRevealView`, all the `Lead(crm.lead)` extensions → `CrmLead`.
- Irrelevant to data/XML (uses `_name`). **Relevant** if custom code does `super(CRMRevealRule, self)` or `class X(CRMRevealRule)` by Python name.

### `super(Lead, self)` / `super(CRMRevealRule, self)` → `super()`
All bare-super rewrites. Same caveat as above for class-name-based overrides.

### `self._cr` → `self.env.cr`
e.g. reveal `_process_lead_generation` autocommit (`self._cr.commit()` → `self.env.cr.commit()`). `_cr` still exists as alias but native code dropped it.

### `__manifest__.py`: adds `'author': 'Odoo S.A.'`
Cosmetic, all three.

### `iap.account.account_token` now accessed `.sudo()`
Mining `_perform_request` and reveal `_perform_reveal_service` now read `reveal_account.sudo().account_token`. Reflects tighter ACLs on `iap.account` in v19 — a custom override that read `.account_token` without sudo may now raise AccessError.

---

## crm_iap_mine (Lead Generation)

`crm_iap_lead_mining_request.py`:
- Class rename only + `account_token` via `.sudo()` (above). `params` dict reordered (cosmetic). **No functional change** to the mining flow, credit math, or payload.

`crm_lead.py`, `crm_iap_lead_helpers.py`, industry/role/seniority: **rename-only**.

Net: mining is essentially unchanged in v19 beyond the cross-cutting items.

---

## crm_iap_enrich (Lead Enrichment) — REAL rewrite in v19

`crm_lead.py` was substantially refactored (the only non-cosmetic logic change in this domain):

1. **Cron signature changed**: `_iap_enrich_leads_cron(enrich_hours_delay=1, leads_batch_size=1000)` → `_iap_enrich_leads_cron(enrich_hours_delay=24, batch_size=50)`.
   - Window widened from **1 hour → 24 hours**; batch knob renamed and defaulted to 50.
   - Cron search now also filters `('email_from','!=',False)` and `('active','=',True)` and **drops the `limit`** (processes all matching, paced by `_commit_progress`).
   - Impact: a custom cron or external caller passing `leads_batch_size=` will break (kwarg renamed to `batch_size`).

2. **`iap_enrich(from_cron=False)` → `iap_enrich(*, batch_size=50)`** (keyword-only). `from_cron` is now derived internally from `self.env.context.get('cron_id')`.
   - Impact: any custom call `leads.iap_enrich(from_cron=True)` **breaks** in v19 — the kwarg no longer exists; run it from the cron (which sets `cron_id` in context) or pass nothing.

3. **Cron-aware batching via `ir.cron._commit_progress` + `try_lock_for_update`**: replaces the old manual `SELECT ... FOR UPDATE NOWAIT` + `cr.savepoint`. Uses `OrderedSet`, time-boxed by `_commit_progress`; if all rows are locked it reschedules the cron 5 min later (`_trigger(now+5min)`).
   - Impact: the v18 row-lock gotcha pattern is gone; locked leads now retry instead of being skipped-and-logged.

4. **Test-mode handling via `modules.module.current_test`** (replaces `env.registry.in_test_mode()` checks around commit).

5. **`mobile` no longer auto-filled**: v18 set `values['mobile']` from `phone_numbers[1]`; v19 **removed** that branch (only `phone` from `phone_numbers[0]`). Impact: enriched leads no longer get a secondary number written to `mobile` (aligns with `mobile`/`phone` field consolidation in v18+ CRM).

`res_config_settings.py`: rename-only; the two-switch confusion (ETG35) persists in v19.

---

## website_crm_iap_reveal (Visitor Reveal)

`crm_reveal_rule.py`:
- `_sql_constraints` list → **`models.Constraint`** attribute:
  ```python
  # v18
  _sql_constraints = [('limit_extra_contacts', 'check(extra_contacts >= 1 and extra_contacts <= 5)', 'Maximum 5 contacts are allowed!')]
  # v19
  _limit_extra_contacts = models.Constraint('check(extra_contacts >= 1 and extra_contacts <= 5)', 'Maximum 5 contacts are allowed!')
  ```
- `self._context` → `self.env.context`, `self._cr.commit()` → `self.env.cr.commit()`, `account_token` via `.sudo()` (cross-cutting). Lead-generation/cron/cache logic otherwise **unchanged**.

`crm_reveal_view.py`:
- The manual `init()` that created indexes via raw SQL is **replaced by declarative index attributes**:
  ```python
  # v18: init() with CREATE UNIQUE INDEX ... / CREATE INDEX ... raw SQL
  # v19:
  _ip_rule_id = models.UniqueIndex("(reveal_rule_id,reveal_ip)")
  _state_create_date = models.Index("(reveal_state,create_date)")
  ```
  - Impact: same indexes, declarative now. A custom override of `init()` to add indexes should migrate to `models.Index`/`models.UniqueIndex`.

`ir_http.py`: whitespace-only (one blank line). `_serve_page` reveal gating logic **unchanged** (still public-only, 200-only, GeoIP-required — ETG40 still applies).

`crm_lead.py`: rename + bare-super only.

---

## Migration checklist for custom code over Lead Intelligence

1. Override/subclass by Python class name (`CRMRevealRule`, `CRMLeadMiningRequest`, `Lead`)? → rename to the new `Crm*`/`CrmLead` names or use `super()`.
2. Call `leads.iap_enrich(from_cron=...)`? → **remove the kwarg** (now keyword-only `batch_size`; `from_cron` derived from context). High-risk break.
3. Call `_iap_enrich_leads_cron(leads_batch_size=...)`? → rename to `batch_size=`; note the default window is now 24h not 1h.
4. Rely on enrichment writing `mobile`? → it no longer does in v19.
5. Read `iap.account.account_token` in a custom enrich/reveal override? → add `.sudo()`.
6. Override `crm.reveal.view.init()` to add indexes, or use `_sql_constraints` on reveal rules? → migrate to `models.Index`/`models.UniqueIndex`/`models.Constraint`.
7. Depend on the v18 enrich row-lock skip behaviour? → v19 retries locked leads via `try_lock_for_update` + cron reschedule instead.
