# ir.cron — Integrations (Odoo 18.0)

How the scheduler couples with the modules that depend on it. Every async/deferred/recurring
behaviour in Odoo ultimately rides on `ir.cron` (or on a parallel runner like OCA `queue_job`).
This file maps the real producers/consumers so you can answer "which cron drives X" and "why is X
not happening" without re-grepping. Source paths cited as `path:line`. Cross-refs: `core.md`
(model/runner), `gotchas.md` (CRONG troubleshooting).

> Flow detail for the business processes themselves lives in the domain references; here we only map
> the cron wiring.

---

## 1. The cron-driven map (who schedules what)

```
                         ┌──────────────────────────────┐
                         │           ir.cron             │
                         │  runner threads / WorkerCron  │
                         │  (server.py, LISTEN cron_     │
                         │   trigger on `postgres` DB)   │
                         └───────────────┬───────────────┘
            ┌──────────────┬─────────────┼──────────────┬───────────────┐
            ▼              ▼             ▼              ▼               ▼
   base_automation   mail/discuss    stock           account         OCA queue_job
   (time triggers)   (mail queue,    (procurement    (auto-post,     (autovacuum cron
   _check(True)      fetchmail,      scheduler,      e-invoicing,    ONLY; jobs run on
   self-tunes        scheduled msg,  run_scheduler)  follow-up EE,   a SEPARATE
   interval)         web push, gc)                   bank sync EE)   jobrunner thread)
```

Key insight: `ir.cron` is a SHARED, low-frequency polling substrate (min poll = `SLEEP_INTERVAL`
60 s, default 2 threads). It is NOT a real-time queue. Heavy/real-time async (thousands of jobs,
sub-second dispatch, channels, retries) is what OCA `queue_job` exists for — it bypasses the cron
runner entirely (§5). Studio "Scheduled Actions" and "Automation Rules (time-based)" both compile
down to the crons below.

---

## 2. base_automation — time-based automation rules

The single most important cron integration: ALL time-based Automation Rules (Studio "Automations"
with trigger "Based on date field") share ONE cron.

| Item | Value | Path:line |
|------|-------|-----------|
| Cron | `base_automation.ir_cron_data_base_automation_check` | `base_automation/data/base_automation_data.xml:4`, `ir_cron_data_base_automation_check` |
| Code | `model._check(True)` | xml:9 |
| Ships | `active=False`, `interval 4 hours` (placeholder) | xml:8-11 |
| Driver | `_update_cron()` re-arms + retunes it | `base_automation/models/base_automation.py:576-589`, `BaseAutomation._update_cron` |

How it works:
- `_check(automatic=True)` (`base_automation/models/base_automation.py:998-1020`, `BaseAutomation._check`) loops over all rules with `trigger in TIME_TRIGGERS` (`on_time`-family), evaluates each rule's `filter_domain`, finds records whose `trg_date_id` field falls in the window `[last_run, now)` shifted by `trg_date_range`/`trg_date_range_type`, and runs the action. Per-rule `last_run` is the high-water mark (xml field on the rule), NOT the cron's `lastcall`.
- `_update_cron()` (`base_automation/models/base_automation.py:576-589`, `BaseAutomation._update_cron`) runs on EVERY create/write/unlink of an automation rule (called from `create`/`write`/`unlink`, lines 420/431/437/443). It sets the cron `active = bool(any time-based rules)` and `interval_number = _get_cron_interval()`, switching `interval_type` to `minutes`.
- `_get_cron_interval()` (`base_automation/models/base_automation.py:629-639`, `BaseAutomation._get_cron_interval`): interval = `min(delay)//10` clamped to **[1 minute, 4 hours]**, where each rule's delay = `|trg_date_range| * factor`. So a rule "1 hour before X" tightens the cron to ~6 min; the tightest rule wins for ALL rules.

Integration gotchas → `gotchas.md` CRONG18-CRONG20 (worst-case latency = the shared interval; a
removed cron silently disables ALL time automations; `_check` swallows per-rule exceptions).

---

## 3. mail / discuss — the busiest cron cluster

`mail/data/ir_cron_data.xml` defines the largest cron group in core. The ones that bite:

| Cron id | Name | Interval | Code | active | Line |
|---------|------|----------|------|--------|------|
| `ir_cron_mail_scheduler_action` | Mail: Email Queue Manager | 1 h, prio 6 | `model.process_email_queue(batch_size=1000)` | True | :4 |
| `ir_cron_mail_gateway_action` | Mail: Fetchmail Service | 5 min | `model._fetch_mails()` | **False** | :40 |
| `ir_cron_delete_notification` | GC notifications >6 months | 1 d | `model._gc_notifications(max_age_days=180)` | True | :31 |
| `ir_cron_post_scheduled_message` | Post scheduled messages | 1 d | `model._post_messages_cron()` | True | :51 |
| `ir_cron_send_scheduled_message` | Notify scheduled messages | 1 h | `model._send_notifications_cron()` | True | :60 |
| `ir_cron_web_push_notification` | send web push | 1 d | `model._push_notification_to_endpoint()` | True | :69 |
| `ir_cron_module_update_notification` | Publisher Update Notification | 1 wk, prio 1000 | `model.update_notification(None)` | True | :15 |

- **Email queue**: outgoing mails are `mail.mail` rows in `state='outgoing'`. They are NOT sent
  synchronously; `process_email_queue` (the 1 h cron) flushes them — UNLESS code calls `.send()`
  directly or the message manager sends on post. So "my email is stuck": check `mail.mail` state and
  whether the scheduler cron ran (→ CRONG21). `send_mail(..., force_send=True)` bypasses the cron.
- **Fetchmail** ships DISABLED; `fetchmail.server.create/write` calls `_update_cron()`
  (`mail/models/fetchmail.py:133-143`, `FetchmailServer.create` y `mail/models/fetchmail.py:306`, `FetchmailServer._update_cron`) to flip the cron `active` based on whether any incoming
  server is confirmed. `_fetch_mails` (`mail/models/fetchmail.py:211-213`, `FetchmailServer._fetch_mails`) commits AFTER EACH email
  (`fetch_mail` "WARNING: meant for cron usage only - will commit() after each email!",
  `mail/models/fetchmail.py:215-218`, `FetchmailServer.fetch_mail`) — so a crash mid-batch keeps already-fetched mail.
- **Scheduled messages** (chatter "send later"): two crons — `_post_messages_cron` posts internal
  scheduled `mail.message`, `_send_notifications_cron` pushes the notifications. Two different
  intervals (1 d vs 1 h) → a scheduled chatter message can be posted up to a day late.

---

## 4. account / stock / sale — the business crons

Procurement (the heaviest functional cron):

| Cron id | Module | Interval | Code | Line |
|---------|--------|----------|------|------|
| `ir_cron_scheduler_action` | stock | 1 d | `model.run_scheduler(True)` | `stock/data/stock_sequence_data.xml:56`, `ir_cron_scheduler_action` |

`run_scheduler` is the **reordering-rules / MTO procurement run** (the old "Run Scheduler" / MRP
scheduler). It iterates orderpoints and procurement groups to generate moves/POs/MOs. It is the
batch fallback for real-time procurement — most procurement actually fires synchronously on
SO/PO confirmation, so this daily cron mainly catches reordering rules. If reordering rules "don't
replenish", this cron (or a manual "Run Scheduler") is the trigger.

account (CE):

| Cron id | Interval | Code | Line |
|---------|----------|------|------|
| `ir_cron_auto_post_draft_entry` | 1 d @02:00 | `model._autopost_draft_entries()` | `account/data/service_cron.xml:3`, `ir_cron_auto_post_draft_entry` |
| `ir_cron_account_move_send` | 1 d | `model._cron_account_move_send()` | `account/data/service_cron.xml:13`, `ir_cron_account_move_send` |

- **Auto-post**: draft `account.move` with `auto_post` set and date ≤ today are posted by the daily
  cron. So a recurring/auto-post entry stays DRAFT until the next 02:00 run — not posted at the exact
  configured instant.
- **Send & Print / e-invoicing**: `_cron_account_move_send` drains queued invoice sends (the "Send &
  Print" async path, and Peppol/EDI in EE). Stuck "to send" invoices → this cron.

account (Enterprise highlights, paths under `odoo_enterprise/18/`):

| Cron | Module | Interval | Code |
|------|--------|----------|------|
| Execute follow-up | `account_followup` | 1 d | `_cron_execute_followup()` |
| Bank online sync | `account_online_synchronization` | 12 h | `_cron_fetch_online_transactions()` |
| Bank online sync (waiting) | `account_online_synchronization` | 5 min (active=False) | `_cron_fetch_waiting_online_transactions()` |
| Auto-reconcile statement lines | `account_accountant` | 1 d | `_cron_try_auto_reconcile_statement_lines(batch_size=100)` |
| Auto transfers | `account_auto_transfer` | 1 d | `action_cron_auto_transfer()` |
| Send account reports | `account_reports` | 1 d | `_cron_account_report_send(job_count=20)` |
| Peppol get/update | `account_peppol` (CE) | 4 h / 1 d / 1 wk | `_cron_peppol_*` (`account_peppol/data/cron.xml`) |

sale / website:

| Cron | Module | Interval | Code | Note |
|------|--------|----------|------|------|
| `send_invoice_cron` | sale | 1 d (active=False) | `_cron_send_invoice()` | automatic invoicing |
| `ir_cron_send_availability_email` | website_sale | 1 h | `_send_abandoned_cart_email()` | abandoned cart |
| `account_analytic_cron_for_invoice` | sale_subscription (EE) | 1 d | `_cron_recurring_create_invoice()` | recurring invoices |
| `account_analytic_cron` | sale_subscription (EE) | 1 wk | `_cron_subscription_expiration()` | expire subs |

The subscription recurring-invoice cron is the canonical "why didn't my subscription bill" — it's a
DAILY batch, not real-time.

---

## 5. OCA queue_job — the parallel runner (NOT cron)

`queue_job` is the standard answer when `ir.cron` is too coarse (real-time async, thousands of jobs,
per-channel concurrency, automatic retries). It does NOT use the cron runner to execute jobs.

| Aspect | ir.cron | OCA queue_job |
|--------|---------|---------------|
| Runner | cron threads / WorkerCron in the Odoo process | SEPARATE `jobrunner` daemon thread, monkey-patched into the server (`queue_job/jobrunner/__init__.py:159-163`) |
| Trigger | poll `nextcall` + `ir.cron.trigger` NOTIFY on `cron_trigger` | PG NOTIFY on channel `queue_job` from a DB trigger (`queue/queue_job/post_init_hook.py:12-33`, `post_init_hook`), dispatched via HTTP GET `/queue_job/runjob` |
| Unit | a scheduled server action | `queue.job` record, states pending→enqueued→started→done/failed (`queue_job/job.py:19-35`, `WAIT_DEPENDENCIES`) |
| API | XML `<record model="ir.cron">` + `_trigger()` | `record.with_delay(priority=, channel=).method(...)` (`queue_job/models/base.py:21-62`, `Base.with_delay`) |
| Concurrency | rows locked SKIP LOCKED, `--max-cron-threads` | per-channel `capacity` (`ODOO_QUEUE_JOB_CHANNELS="root:4"`) |
| Uses ir.cron for | everything | ONLY a GC cron: "AutoVacuum Job Queue", daily, `model.autovacuum()` (`queue/queue_job/data/queue_data.xml:10-18`, `ir_cron_autovacuum_queue_jobs`) |

How a job flows: `with_delay()` creates a `queue.job` in `PENDING` and the DB trigger fires
`pg_notify('queue_job', uuid)`; the jobrunner (`jobrunner/runner.py` `LISTEN queue_job`, ~:183) picks
it, marks `ENQUEUED`, and does an HTTP GET to `/queue_job/runjob` (`queue_job/controllers/main.py:209-223`, `RunJobController.runjob`)
which marks `STARTED` and runs the method.

queue_job gotchas → `gotchas.md` CRONG22-CRONG23 (jobrunner not started → jobs stuck PENDING
forever; needs `--load=web,queue_job`; `ODOO_QUEUE_JOB_CHANNELS` capacity 0 disables it; base-url
must be reachable for the HTTP callback). These look exactly like "my cron doesn't run" but the fix
is completely different (it's NOT the cron runner).

---

## 6. autovacuum — the GC umbrella (`@api.autovacuum`)

| Item | Value | Path:line |
|------|-------|-----------|
| Cron | `base.autovacuum_job` "Base: Auto-vacuum internal data", 1 d, prio 3 | `base/data/ir_cron_data.xml:3`, `autovacuum_job` |
| Code | `model._run_vacuum_cleaner()` | xml:7 |
| Mechanism | runs EVERY method decorated `@api.autovacuum` across the whole registry | `odoo/addons/base/models/ir_autovacuum.py:25-42`, `AutoVacuum._run_vacuum_cleaner` (`is_autovacuum`, line 15-17) |

A single daily cron drives ALL garbage collection. Modules register GC by decorating a method with
`@api.autovacuum` (no per-module cron needed): e.g. `ir.cron._gc_cron_triggers` /
`_gc_cron_progress` (ir_cron.py:762,782), `ir.attachment._gc_file_store`, `ir.http` sessions,
`res.users` portal cleanup. Consequence: if `autovacuum_job` is disabled or its node has no cron
runner, attachments/triggers/sessions/etc. accumulate silently — and a single autovacuum method
raising is caught & logged per-method (`ir_autovacuum.py` try/except) so one bad GC doesn't block the
others, but it also fails quietly.

---

## 7. Quick "which cron drives X" index

| Symptom / question | Cron / runner | Where |
|--------------------|---------------|-------|
| Time-based Automation Rule / Studio time automation | `base_automation.ir_cron_data_base_automation_check` | §2 |
| Outgoing email stuck | `mail.ir_cron_mail_scheduler_action` (`process_email_queue`) | §3 |
| Incoming email not fetched | `mail.ir_cron_mail_gateway_action` (`_fetch_mails`, ships off) | §3 |
| Chatter "send later" message | `ir_cron_post_scheduled_message` / `_send_scheduled_message` | §3 |
| Reordering rules not replenishing | `stock.ir_cron_scheduler_action` (`run_scheduler`) | §4 |
| Auto-post journal entry not posting | `account.ir_cron_auto_post_draft_entry` | §4 |
| Invoice "to send" not sent | `account.ir_cron_account_move_send` | §4 |
| Bank feed not updating (EE) | `account_online_synchronization` crons | §4 |
| Follow-up reminders (EE) | `account_followup` cron | §4 |
| Subscription not billing (EE) | `sale_subscription` recurring-invoice cron | §4 |
| Abandoned cart email | `website_sale.ir_cron_send_availability_email` | §4 |
| `with_delay()` job stuck PENDING | OCA jobrunner (NOT cron) | §5 |
| Attachments/triggers/sessions accumulating | `base.autovacuum_job` | §6 |
