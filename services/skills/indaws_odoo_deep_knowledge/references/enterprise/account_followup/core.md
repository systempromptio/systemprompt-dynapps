<!-- verified: v18.0 @ 2026-06 -->
# account_followup — Automated Dunning / Payment Chasing

Module path: `odoo_enterprise/18/account_followup/` (EE only).
`auto_install=True`, `OEEL-1`. Depends: `mail`, `sms`, `account_reports`.

## Purpose

Automatic multi-level reminders for unpaid customer invoices. Sends email/SMS, schedules activities, and prints follow-up letters. Operates on partners (not invoices) — aggregates all unreconciled `account.move.line` per partner.

## Models

| Model | Purpose |
|-------|---------|
| `account_followup.followup.line` | A reminder level: `delay` (days after due date), `mail_template_id`, `sms_template_id`, `create_activity`, `auto_execute` |
| `res.partner` (extended) | `followup_status`, `followup_line_id`, `followup_next_action_date`, `followup_responsible_id`, `followup_reminder_type` (auto/manual), `total_due`, `total_overdue`, `unreconciled_aml_ids`, `unpaid_invoice_ids` |
| `account.move.line` (extended) | `followup_line_id` — the level applied to THIS line; `invoice_origin` (related). v19 adds `no_followup` (per-line block flag, AFG36). Adds `followup_overdue` read-group key (`_read_group_groupby`, account_move_line.py:14) |
| `account.followup.report` | AbstractModel for HTML/PDF rendering of follow-up letter |
| `account_followup.manual_reminder` | TransientModel wizard (mail.composer.mixin) for manual send |
| `account_followup.missing.information.wizard` | Wizard listing partners missing email/phone |

## followup.line Fields

`delay` (Integer, can be NEGATIVE = remind BEFORE due date), `name` (description, unique per company), `auto_execute` (boolean — if True, cron sends without manual click), `create_activity` (mutually exclusive with auto_execute via onchange), `activity_default_responsible_type` (`followup`/`salesperson`/`account_manager`).

`_sql_constraints`: `(company_id, delay)` AND `(company_id, name)` unique pairs.

## Lifecycle

```
Invoice posted with date_maturity → counted in partner.unreconciled_aml_ids
  → daily cron calls model._cron_execute_followup() (~02:00, see Cron)
  → loops ALL res.company, pops cache, runs _cron_execute_followup_company per company
  → For each company, query _query_followup_data(all_partners=True) (all partners' status)
  → Filter status='in_need_of_action' AND auto_execute=True AND followup_reminder_type='automatic'
  → Process up to 1000 partners per run
  → For each: _execute_followup_partner()
       - schedule activity (if create_activity)
       - update followup_next_action_date (delta to next level)
       - generate followup_letter PDF (account.followup.report)
       - attach unreconciled invoices (if join_invoices)
       - send email + sms
       - log message + update aml.followup_line_id
```

## Status Computation (`_query_followup_data`)

A single SQL query (`_get_followup_data_query`, res_partner.py:394) with two LATERAL sub-joins (`in_need_of_action_aml`, `exceeded_unreconciled_aml`) produces follow-up status for all partners. The status is a `CASE` evaluated TOP-DOWN (res_partner.py:404-407):

| Order | Status | Condition (SQL) |
|-------|--------|-----------------|
| 1 | `no_action_needed` | `partner.amount_residual <= 0` (SUM over posted-unreconciled asset_receivable, AFG37) |
| 2 | `in_need_of_action` | `in_need_of_action_aml IS NOT NULL` AND `(followup_next_action_date IS NULL OR <= current_date)` |
| 3 | `with_overdue_invoices` | `exceeded_unreconciled_aml IS NOT NULL` (else branch — has overdue line but #2 failed: either delay threshold not yet reached at the partner's current level, OR `next_action_date` is in the future) |
| 4 | `no_action_needed` | fallback |

`in_need_of_action_aml`: a line whose `date_maturity + COALESCE(ful.delay, min_delay-1) < current_date` AND `balance > 0` AND its level delay `< partner.followup_delay` (res_partner.py:435-452).
`exceeded_unreconciled_aml`: any posted-unreconciled receivable with `balance > 0` and `date_maturity < current_date` (res_partner.py:453-468).

Each partner's `followup_line_id` = the line whose `delay = MAX(COALESCE(next_ful.delay, ful.delay))` across their AMLs (res_partner.py:411, 433). v19 adds `AND line.no_followup IS NOT TRUE` to both LATERAL joins (AFG36); v18 has no per-line exclusion (AFG35).

The query result is CACHED in `env.cr.cache['res_partner_all_followup']` per cursor (NOT per company — must be invalidated when iterating companies in `_cron_execute_followup`).

## Next Date Logic (`followup.line._get_next_date`)

```
next_followup = next level by delay (asc)
if next_followup:
    delay = next_followup.delay - self.delay
elif previous_followup:
    delay = self.delay - previous_followup.delay
else:
    delay = self.delay
return today + timedelta(days=delay)
```

## Responsible Resolution (`_get_followup_responsible`, res_partner.py:214)

Early-return then first-active-in-list. `responsible_type = followup_line_id.activity_default_responsible_type`:
1. If `account_manager` AND `partner.user_id` → return `user_id` (early return, res_partner.py:218)
2. Else `next(u for u in candidates if u and u.active)` over (res_partner.py:223-229):
   - `salesperson` AND `most_delayed_aml.move_id.invoice_user_id` (most_delayed = highest-amount AML, AFG24)
   - `partner.followup_responsible_id`
   - `partner.user_id`
   - `most_delayed_aml.move_id.invoice_user_id`
3. Fallback `super()._get_followup_responsible()` → `account_reports/models/res_partner.py:13`, `ResPartner._get_followup_responsible`.

v19: signature `_get_followup_responsible(self, multiple_responsible=False)`, `most_delayed_aml`→`max_amount_aml`, can return multiple users (AFG34).

## Follow-up Contacts

`_get_all_followup_contacts`:
- v18: child contacts with `type='followup'`, fallback to billing address.
- v19: ONLY billing address (followup type removed).

## Manual Reminder Wizard

`account_followup.manual_reminder`: TransientModel inheriting `mail.composer.mixin`. Pre-populated with current followup_line's templates. User can override email/SMS/print booleans. `attachment_ids` defaults to all unreconciled invoices' main attachments.

## Letter PDF

`action_report_followup` action renders `account_followup.report_followup_print_all` template. Uses partner.lang. PDF attached to chatter automatically. Combined with invoice PDFs into a single multi-doc envelope.

## Cron

`data/cron.xml`:

```xml
<record id="ir_cron_auto_post_draft_entry" model="ir.cron">
    <field name="name">Account Report Followup; Execute followup</field>
    <field name="interval_number">1</field>
    <field name="interval_type">days</field>
    <field name="nextcall" eval="(DateTime.now().replace(hour=2, minute=0) + timedelta(days=1)).strftime('%Y-%m-%d %H:%M:%S')"/>
    <field name="model_id" ref="model_res_partner"/>
    <field name="code">model._cron_execute_followup()</field>
    <field name="state">code</field>
</record>
```

NOTE: `xml_id="ir_cron_auto_post_draft_entry"` is misleading — copy-paste leftover, NOT actually about draft posting (AFG7). Model is `res.partner`, code entrypoint is `_cron_execute_followup` (res_partner.py:654).

## Key Methods

- `res.partner.action_manually_process_automatic_followups()` — bulk-fire on selected partners; opens missing-info wizard if any partner lacks email/sms target.
- `res.partner.execute_followup(options)` — manual single-partner from wizard.
- `_execute_followup_partner(options=None)` — internal; called by cron AND wizard.
- `_get_followup_attachments(options)` — assembles report + invoice attachments per template config.
- `_get_followup_data_query_extra_join_conditions()` — hook for adding filters (e.g. exclude credit notes); override in localization modules.

## Multi-company

`company_id` mandatory on `followup.line`. Lines scoped via `('company_id', 'parent_of', env.company.id)` — child companies inherit parent's plan. `_cron_execute_followup` iterates ALL `res.company` and runs per company with `with_context(allowed_company_ids=company.ids)`.
