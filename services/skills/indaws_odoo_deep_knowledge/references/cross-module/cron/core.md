<!-- verified: v18.0 @ 2026-06 -->
# ir.cron — Scheduled Actions Core (Odoo 18.0)

Source: `odoo/addons/base/models/ir_cron.py` (784 lines), runner in `odoo/service/server.py`.
This is the operational/runtime model. ORM API of the model itself is trivial; the value is in
HOW the runner picks, locks, executes, reschedules and deactivates jobs. See `gotchas.md` (CRONG).

## Model `ir.cron` — key fields (ir_cron.py:54-92)

- `ir_actions_server_id` Many2one ir.actions.server, `delegate=True`, `ondelete='restrict'`, required (line 68). A cron IS-A server action (delegation inheritance); the actual code lives on the server action, default `state='code'` (line 110).
- `cron_name` Char, compute `_compute_cron_name`, stored, mirrors the server action `name` in `en_US` (line 71, 94-97).
- `user_id` Many2one res.users "Scheduler User", required, default = current user (line 72). The job runs AS this user (`_process_job`: `api.Environment(cron_cr, job['user_id'], {})`, line 358; `_run_job` line 414).
- `active` Boolean, default True (line 73). The runner only selects `active = true` rows (line 218).
- `interval_number` Integer, default 1, required, `aggregator=None` (line 74). SQL CHECK `interval_number > 0` (line 86-91) — strictly positive.
- `interval_type` Selection minutes/hours/days/weeks/months, default `months`, required (line 75-79). Mapping to relativedelta in `_intervalTypes` (line 39-45); note `weeks` = `days=7*interval`.
- `nextcall` Datetime, required, default now (line 80). Stored in UTC. The runner readiness test is `nextcall <= now() at time zone 'UTC'` (line 219).
- `lastcall` Datetime (line 81). Set to "now" on every reschedule (line 528-534); injected into the job context under key `lastcall` (line 415) so the action can do incremental processing.
- `priority` Integer, default 5, `aggregator=None` (line 82). 0 = highest. Ordering on selection is `failure_count, priority, id` (line 226).
- `failure_count` Integer, default 0 (line 83). Consecutive failures; reset to 0 on any success.
- `first_failure_date` Datetime (line 84). Timestamp of the first failure in the current failure streak.

### Fields that DO NOT EXIST in Odoo 18 (REMOVED in 18.0)
`numbercall` and `doall` were REMOVED in **18.0** (still present in 17.0/16.0 — verified
`odoo/addons/base/models/ir_cron.py:67-68`, `ir_cron` has both fields; see `v19_changes.md` §v17 for the old semantics).
Old docs/tutorials and any migrated `<field name="numbercall">`/`doall` in module data XML will
ERROR on 18+ (`Invalid field 'numbercall' on model 'ir.cron'`). The "run once and disable"
behaviour (old `numbercall=1`) is gone — auto-deactivation is now driven by repeated FAILURES
(`failure_count` + `first_failure_date`), NOT by a call counter. The "catch up missed runs"
behaviour (old `doall=True`) is gone: a missed schedule fires exactly ONCE and `nextcall` is
fast-forwarded past `now` (see `_reschedule_later`, line 521 `while nextcall <= now`).

## Companion models

- `ir.cron.trigger` (line 753-768): one-shot scheduling. `cron_id` Many2one, `call_at` Datetime (both indexed). A row makes its cron ready when `call_at <= now` even if `nextcall` is in the future (line 220-224). Created by `_trigger()` / `_trigger_list()` (line 644-703). GC'd by `@api.autovacuum _gc_cron_triggers` deleting triggers older than 1 week (line 762-768). Consumed triggers are deleted on successful reschedule (line 536-540: `DELETE ... WHERE call_at < now`).
- `ir.cron.progress` (line 771-784): progress/timeout bookkeeping for the batch protocol. Fields `cron_id`, `remaining`, `done`, `deactivate`, `timed_out_counter`. Written by the action via `_notify_progress(done=, remaining=, deactivate=)` (line 734-750). GC'd after 1 week (line 782-784).

## How the runner processes jobs

Entry point `_process_jobs(db_name)` (classmethod, line 126-168), called per DB by the runner:

1. `_check_version` (line 170-182): DB `base` module `latest_version` must equal code `BASE_VERSION` (line 20), else `BadVersion` → whole DB skipped (line 155-156). This blocks crons when code and DB are out of sync (pre/post-migration window).
2. `_get_all_ready_jobs` (line 212-228): single SELECT of ALL ready jobs — `active = true AND (nextcall <= now_utc OR id IN ready triggers)`, ordered `failure_count, priority, id`. Returns plain dicts (not recordsets).
3. `_check_modules_state` (line 184-210): if any module is `to install/upgrade/remove` (`state LIKE 'to %'`), raises `BadModuleState` → DB skipped, UNLESS the oldest ready job has been waiting > `MAX_FAIL_TIME` (5h, line 21), in which case it force-calls `reset_modules_state` to unstick zombie module states.
4. Per job: `_acquire_one_job` (line 230-316) re-SELECTs that single id with `FOR NO KEY UPDATE SKIP LOCKED` (line 295). If another worker already holds the row lock, SKIP LOCKED returns no row → `job is None` → skipped (line 311, 146-148). A concurrent commit of the new nextcall raises `TransactionRollbackError` (serialization) → caught, `cron_cr.rollback()`, skip (line 142-145, 299-304). `NO KEY UPDATE` is used (not `UPDATE`) so FK references (`ir.cron.trigger.cron_id`) don't deadlock (line 262-269).
5. `_process_job` (line 328-389) runs in the shared `cron_cr` transaction; the actual action runs in a SEPARATE cursor via `_run_job` (line 390-455, `with cls.pool.cursor() as job_cr`). Each successful batch iteration does its own `job_cr.commit()` (line 423, 449). After determining status, `_process_job` updates failure count + reschedules and does ONE `cron_cr.commit()` per job (line 388) — so each job is committed independently; a crash mid-list does not roll back already-finished jobs.

### Batch / progress protocol (`_run_job`, line 390-455)
The action is called up to `MAX_BATCH_PER_CRON_JOB = 10` times (line 22, loop line 421) in ONE worker pass.
- Action does NOT use progress API → treated as `FULLY_DONE` after first call (line 437-440).
- Action calls `_notify_progress(done>0, remaining>0)` → `PARTIALLY_DONE` → loops again (up to 10), then job is left "ready" and rescheduled ASAP (`_reschedule_asap` = no-op leaving nextcall, line 542-548) so another pass/worker continues.
- Action calls `_notify_progress(..., deactivate=True)` and finishes → `job['active'] = False` (line 444-445), cron deactivated.
- Exception with prior committed progress (`progress.done and progress.remaining`) → `PARTIALLY_DONE` (committed work kept); exception with no progress → `FAILED` (line 427-433).

### Failure handling & auto-deactivation (`_update_failure_count`, line 457-509)
On `FAILED`: `failure_count += 1`, set `first_failure_date` if it was empty. The cron is DEACTIVATED
(`active=False`, counters reset) only when BOTH thresholds are met (line 24-26, 478-484):
`failure_count >= MIN_FAILURE_COUNT_BEFORE_DEACTIVATION (5)` AND `first_failure_date + MIN_DELTA_BEFORE_DEACTIVATION (7 days) < now`.
On deactivation, `_notify_admin` (default just logs a warning, line 318-326). On any success → counters reset.
Timeout path: if `timed_out_counter >= CONSECUTIVE_TIMEOUT_FOR_FAILURE (3)` and nothing done, the run is marked FAILED without executing (line 361-375).

### Reschedule (`_reschedule_later`, line 511-540)
Computed in the SCHEDULER USER's timezone (`context_timestamp`) to avoid DST/month-boundary drift
(comment line 516-517). `while nextcall <= now: nextcall += interval` — i.e. ALL missed slots are
collapsed into a single next future slot (no catch-up). `lastcall` set to now. Past triggers deleted.

## Runner topology

- `--max-cron-threads` (default 2; config.py:321-323). `0` disables the cron runner entirely.
- Threaded mode (`workers=0`): `cron_spawn` (server.py:527-546) spawns `max_cron_threads` daemon threads, each running `cron_thread` (server.py:479-525). Each thread `LISTEN cron_trigger` on the `postgres` DB connection (server.py:499) and polls every `SLEEP_INTERVAL` with per-thread jitter (`time.sleep(number/100)`, server.py:505-506) to spread the thundering herd. On NOTIFY all wake at once.
- Multiprocess mode (`workers>0`): `max_cron_threads` separate `WorkerCron` processes (server.py:937-938). Each `WorkerCron.process_work` (server.py:1255-1277) processes ONE DB per poll, round-robin over `_db_list()` via `db_index` (line 1259-1260). `_db_list` (line 1248-1253) = `--database` (`config['db_name']`, comma-split) if set, else `list_dbs(True)` filtered by `--db-filter`. With many DBs and few cron workers, throughput per DB drops; if `request_max < len(db_names)` it logs an error about un-processable DBs (line 1271-1274).
- `--limit-time-worker-cron` (default 0 = disabled; config.py:324-327): max age before a cron thread/worker is recycled. `--limit-time-real-cron` (default -1 → falls back to `limit_time_real`; config.py:364, server.py:803-805): wall-clock limit applied to a single cron thread (server.py:460-462). A long job exceeding it is killed.
- Trigger wake-up uses PG `LISTEN/NOTIFY` on channel `cron_trigger` (`_notifydb`, line 705-712). LISTEN does NOT work when the PG cluster is in recovery mode (read replica) — logged, triggers effectively disabled (server.py:495-501). `ODOO_NOTIFY_CRON_CHANGES` env var forces a NOTIFY on every cron write / trigger create, not just imminent ones (line 103-105, 611-612, 701-702).

## Defining a cron in a module (data XML)

A cron IS-A server action (delegation). The record is `model="ir.cron"` and carries BOTH the
ir.cron fields AND the inherited `ir.actions.server` fields (`model_id`, `state`, `code`). Canonical
shape (cf. `mail/data/ir_cron_data.xml:4-13`, `ir_cron_mail_scheduler_action`, `base/data/ir_cron_data.xml:3-12`, `autovacuum_job`):

```xml
<record id="ir_cron_my_job" model="ir.cron">
    <field name="name">My Module: do the thing</field>
    <field name="model_id" ref="model_my_model"/>   <!-- required, the action's model -->
    <field name="state">code</field>                 <!-- always 'code' for crons -->
    <field name="code">model._cron_do_the_thing()</field>
    <field name="user_id" ref="base.user_root"/>     <!-- runs AS this user; default = installer -->
    <field name="interval_number">1</field>
    <field name="interval_type">hours</field>
    <field name="priority">5</field>
    <!-- optional: <field name="active" eval="False"/> ships disabled -->
    <!-- optional: <field name="nextcall" eval="(DateTime.now()+timedelta(days=1)).strftime('%Y-%m-%d %H:%M:%S')"/> -->
</record>
```

Conventions seen across core: in the `code`, `model` is the recordset of `model_id` (empty
recordset, env as `user_id`). Common to wrap in `forcecreate="True"` (recreate if user deleted it)
and place under `<data noupdate="1">` so admins' interval edits survive `-u` (cf.
`base_automation/data/base_automation_data.xml:3` — the noupdate wrapper is why the runtime
`_update_cron` interval changes persist). Several core crons ship `active=False` and are armed
programmatically: `mail.ir_cron_mail_gateway_action` (fetchmail, armed on `fetchmail.server`
create/write via `_update_cron`, `mail/models/fetchmail.py:306`, `FetchmailServer._update_cron`), `base_automation` check cron, `account.send_invoice_cron`.

### Server action linkage (`ir.actions.server`)
A cron's `usage='ir_cron'` is forced on create by `ir.cron.create` itself (`vals['usage'] = 'ir_cron'`,
ir_cron.py:101-102), NOT by ir_actions — the value is one of the `usage` Selection options on
`ir.actions.server` (ir_actions.py:538-541). It marks the action as a scheduled action so it does NOT
show in the generic "Server Actions" / contextual-action lists. `state='code'` is the only state used
for crons (`default_get` forces it, ir_cron.py:107-112). The action body runs through
`ir.actions.server.run()` (ir_actions.py:940) → `_run_action_code_multi` (ir_actions.py:826-828);
the `code` field is `safe_eval`'d (`mode="exec"`, `nocopy=True`) with an eval context (ir_actions.py:906-938)
exposing `env`, `model` (the `model_id` model, empty recordset for cron), `record`/`records` (None for cron),
`log`, `UserError`, `_logger`, plus time/datetime/dateutil helpers (ir_actions.py:139-141). A cron is just
a scheduled server action: anything you can do in an "Execute Python Code" server action, a cron does.

**Access check before the body runs** (ir_actions.py:962-989): `run()` dispatches as `self.sudo()` but
the access *checks* use the scheduler `user_id`. If the action has `groups_id`, that user must be in one
of those groups (else `AccessError`); otherwise it requires `model.check_access("write")` (and write on any
targeted real records). A low-privilege scheduler user therefore makes the cron fail with AccessError before
its code ever executes — see `gotchas.md` CRONG24. This is why core crons use `user_id ref="base.user_root"`.

## `_trigger()` — on-demand / "run now" scheduling

`record._trigger(at=None)` (ir_cron.py:644-672) and `_trigger_list(at_list)` (674-703) insert
`ir.cron.trigger` rows so the cron becomes ready at `call_at` regardless of `nextcall`. Patterns:

- `self.env.ref('module.ir_cron_x')._trigger()` → run ASAP (next poll, or instantly if NOTIFY lands).
- `cron._trigger(at=fields.Datetime.now() + timedelta(minutes=30))` → schedule a one-shot future run.
- `cron._trigger_list([t1, t2, ...])` → multiple one-shots.

The trigger row + the PG NOTIFY are queued in `postcommit` (line 701-702), so they only materialise
if the surrounding transaction COMMITS. A rolled-back transaction inserts nothing (see `gotchas.md`
CRONG9). On an INACTIVE cron, immediate triggers are dropped — only future `at` survive (CRONG10).
This is the idiomatic Odoo 18 way to do "process this queue now" without `numbercall` or sleeping:
queue work + `_trigger()` and let the cron drain it (used by mail, snailmail, account move send,
queue_job-style patterns).

## Executing a cron manually & in tests

- **UI**: form view button "Run Manually" → `method_direct_trigger` (ir_cron.py:114-124): takes the
  `_try_lock` (NOWAIT), sets `lastcall=now`, calls `_callback` directly in the CURRENT transaction
  (NOT the isolated job cursor, NO batch loop, NO failure_count bookkeeping). So a manual run can
  succeed where the scheduled run fails (different cursor/commit semantics) and vice-versa.
- **Tests**: crons do NOT fire inside `TransactionCase`/`HttpCase` — the runner thread is not driving
  the test DB and the test transaction never commits. To exercise cron logic, call the method
  directly (`self.env.ref('module.ir_cron_x').method_direct_trigger()` or
  `self.env['my.model']._cron_do_the_thing()`). `_trigger()` in a test only writes a trigger row that
  no runner consumes; it does NOT execute. See `gotchas.md` CRONG17.

## Timezone of nextcall / reschedule

`nextcall`/`lastcall` are stored in UTC. Readiness compares `nextcall <= now() at time zone 'UTC'`
(line 219). BUT `_reschedule_later` computes the next slot in the **scheduler user's** timezone via
`context_timestamp` (line 518-519) so daily/weekly/monthly jobs keep their wall-clock time across DST
and month boundaries, then converts back to UTC to store (line 531). Consequence: a "daily at 02:00"
cron stays at 02:00 local even across DST; if you set `nextcall` by hand in the DB, set it in UTC.
The job context gets `lastcall` for incremental processing (line 415). NOTE the field *help* claims
"Previous time the cron ran **successfully**" (line 81), but the code sets `lastcall=now` on EVERY
`_reschedule_later` — which fires for FAILED as well as FULLY_DONE (line 379-380, 525-534). So `lastcall`
is "time of last terminal run (success or failure)", NOT a success marker; the help text is misleading.
The only success-only signal is `failure_count`/`first_failure_date` resetting to 0/NULL (line 493-496).
See `gotchas.md` CRONG7.

## v19 diffs
Detailed in `v19_changes.md`. Same field set (still no numbercall/doall), same SKIP-LOCKED +
deactivation design; runner refactored (staticmethods, `_process_jobs_loop`,
`_get_ready_sql_condition`, `include_not_ready` acquire flag, `write_date` in module-state check),
batch protocol replaced by time/run budget (`MIN_RUNS_PER_JOB`/`MIN_TIME_PER_JOB`), `_notify_progress`
deprecated for `_commit_progress`.
