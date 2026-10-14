# ir.cron — Version diffs (v17 → v18 → v19)

The scheduler was substantially reworked in **18.0** (model semantics) and again refactored in
**19.0** (runner internals). This file is the canonical version map; `core.md`/`gotchas.md` describe
18.0 and reference here. All claims verified against source, cited `path:line`.

Line counts: `odoo17/.../ir_cron.py` ≈ 559 · `odoo18/.../ir_cron.py` 784 · `odoo19/.../ir_cron.py` 933.

---

## §v17 — what 17.0 had that 18.0 REMOVED

17.0 (and 16.0) used the OLD call-counter model. Verified `odoo17/odoo/addons/base/models/ir_cron.py:67-70`, `ir_cron.numbercall`:

```python
numbercall = fields.Integer('Number of Calls', default=1, help='... negative number indicates no limit.')
doall      = fields.Boolean('Repeat Missed', help='execute missed occurrences when the server restarts.')
nextcall   = fields.Datetime(...)   # kept in 18
lastcall   = fields.Datetime(...)   # kept in 18
```

| Concept | v17 / v16 | v18+ |
|---------|-----------|------|
| `numbercall` (calls left; -1 = unlimited) | YES, `odoo/addons/base/models/ir_cron.py:67`, `ir_cron` | **REMOVED** |
| `doall` (catch up missed runs on restart) | YES, `odoo/addons/base/models/ir_cron.py:68`, `ir_cron.ir_actions_server_id` | **REMOVED** (no catch-up; CRONG8) |
| Run-once-and-disable | `numbercall=1` → auto `active=False` | gone — use `active=False` + `_trigger()` |
| Auto-deactivation trigger | call counter hits 0 | repeated FAILURES (`failure_count`+`first_failure_date`) |
| `failure_count` / `first_failure_date` | NO | YES (18) |
| `ir.cron.progress` + `_notify_progress` batch protocol | NO | YES (18) |
| Readiness SQL | `... AND numbercall != 0` (`v17:209`) | `active AND (nextcall<=now OR trigger)` |
| Reschedule | honored `numbercall`/`doall` (`v17:342-356`) | `while nextcall<=now: +=interval`; `lastcall`=now en TODO terminal (FULLY_DONE **y** FAILED, vía `_reschedule_later`), NO solo éxito — ver core.md:156-158 |

v17 already had `ir.cron.trigger` and `FOR NO KEY UPDATE SKIP LOCKED` (`v17:282`), and the
`WorkerCron`/`cron_thread` runner topology is essentially identical to v18. So the **runner** is
stable v17→v18; the **model semantics** are what changed (counter → failure-based + progress).

**Migration impact**: any v17 module data XML with `<field name="numbercall">`/`doall` breaks on 18
(CRONG18). Custom code reading `cron.numbercall`/`cron.doall` breaks. Logic that relied on
`doall=True` to replay missed runs after downtime must be redesigned (no equivalent).

---

## §v18 — baseline (described in core.md/gotchas.md)

Reference point. Key constants `odoo/addons/base/models/ir_cron.py:20-25`, `BASE_VERSION`: `MAX_BATCH_PER_CRON_JOB=10`,
`MAX_FAIL_TIME=5h`, `CONSECUTIVE_TIMEOUT_FOR_FAILURE=3`, `MIN_FAILURE_COUNT_BEFORE_DEACTIVATION=5`,
`MIN_DELTA_BEFORE_DEACTIVATION=7 days`. Acquire: `FOR NO KEY UPDATE SKIP LOCKED` (line 295).

---

## §v19 — refactor (behaviour mostly preserved, internals moved)

**Same field set as 18** — still NO `numbercall`/`doall`; same `active`/`nextcall`/`lastcall`/
`interval_*`/`priority`/`failure_count` (`odoo/addons/base/models/ir_cron.py:121`, `ir_cron`)/`first_failure_date`. Same
deactivation thresholds (5 failures / 7 days). So ALL CRONG gotchas from 18 still hold. What changed:

### Batch protocol: count cap → time/run budget
v18's `MAX_BATCH_PER_CRON_JOB=10` (run the action ≤10 times per pass) is replaced by a dual budget:

```python
MIN_RUNS_PER_JOB = 10        # odoo19/.../ir_cron.py:33
MIN_TIME_PER_JOB = 10        # seconds, ir_cron.py:34
```

`_run_job` now loops until BOTH ≥`MIN_RUNS_PER_JOB` runs AND ≥`MIN_TIME_PER_JOB` seconds elapsed
(`odoo/addons/base/models/ir_cron.py:485`, `ir_cron._update_failure_count` `cron_end_time = start_time + MIN_TIME_PER_JOB`; loop condition `:493-496`). Effect:
fast batches keep going past 10 iterations (until 10 s), slow ones stop after 10 runs. A
progress-API cron drains differently than on 18 — judge by wall-clock, not iteration count.

### Progress API renamed
`_notify_progress(done=, remaining=, deactivate=)` is **deprecated** in 19 — `odoo/addons/base/models/ir_cron.py:823`, `IrCron._notify_progress`, v19
`@api.deprecated("Since 19.0, use _commit_progress")`. New name `_commit_progress(...)`
(`odoo/addons/base/models/ir_cron.py:846`, `IrCron._commit_progress`, v19). Custom actions using the progress protocol should migrate the call; the old name
still works (deprecation shim).

### Methods → `@staticmethod`, loop extracted
- `_process_jobs(db_name)` is now `@staticmethod` (was `@classmethod`) — `odoo/addons/base/models/ir_cron.py:187`, `IrCron._process_jobs`, v19.
- New `_process_jobs_loop(cron_cr, *, job_ids=())` `@staticmethod` extracts the per-job loop —
  `odoo/addons/base/models/ir_cron.py:216`, `IrCron._process_jobs_loop`, v19. **Overrides of the old `_process_jobs` body must move here.**
- `_check_version`, `_check_modules_state`, `_get_all_ready_jobs`, `_acquire_one_job` all
  `@staticmethod` now: `odoo/addons/base/models/ir_cron.py:241`, `IrCron._check_version`; `:255`, `IrCron._check_modules_state`; `:297`, `IrCron._get_all_ready_jobs`; `:308`, `IrCron._acquire_one_job` (v19).
- `_process_job(cron_cr, job)` drops the `db` parameter (`odoo/addons/base/models/ir_cron.py:399`, `IrCron._process_job`, v19; era `(db, cron_cr, job)` en 18).

### Readiness SQL extracted + `include_not_ready`
- `_get_ready_sql_condition(cr) -> SQL` returns a composable `SQL` object with bound `now` —
  `odoo/addons/base/models/ir_cron.py:284-294`, `IrCron._get_ready_sql_condition`, v19. Lo compone
  `odoo/addons/base/models/ir_cron.py:297-305`, `IrCron._get_all_ready_jobs`.
- `_acquire_one_job(cr, job_id, *, include_not_ready=False)` gains the flag (`odoo/addons/base/models/ir_cron.py:308`, `IrCron._acquire_one_job`, v19) so
  `method_direct_trigger` ("Run Manually") can acquire a job that is NOT yet due. New
  `_clear_schedule(job)` (`:625`) runs at job start to consume triggers.

### Module-state self-heal uses `write_date`
`_check_modules_state` now keys on `max(job['nextcall'], job['write_date'])` (`:272`) instead of
`min(nextcall)`, avoiding a spurious `reset_modules_state` right after installing a module whose
seed cron carries an old `nextcall`. `reset_modules_state` import moved to `odoo.modules.loading`
(`:280`) — relevant only if you call it directly.

### Runner (server.py) — notify-aware DB queue
- `WorkerCron` replaces the round-robin `db_index` with a `deque` queue (`odoo/service/server.py:1413`, `WorkerCron.__init__`
  `self.db_queue: deque[str]`); `process_work` (`:1425-1463`) populates it from
  `cron_database_list()` + DBs that NOTIFY'd on `cron_trigger`, prioritising notified DBs
  (FIFO `popleft`, `:1447`). New helper `cron_database_list()` (`:110-111`). `cron_thread`
  similarly uses an `OrderedSet` of notified DBs (`:530-606`). Practical effect: with many DBs and
  few cron workers, a DB that just got a trigger is served sooner than under v18's pure round-robin
  (mitigates CRONG2/throughput, not eliminates it).

### Class naming
PEP8 class rename: `ir_cron` → `IrCron`, and `ir_cron_progress` → `IrCronProgress`
(`odoo/addons/base/models/ir_cron.py:918`, `IrCronProgress`, v19). Cosmetic, but matters if you subclass by Python name.

---

## §Override cheat-sheet across versions

| You want to override… | v18 | v19 |
|-----------------------|-----|-----|
| Per-DB processing entry | `_process_jobs` (classmethod) | `_process_jobs` (static) → put loop logic in `_process_jobs_loop` |
| The job loop | inside `_process_jobs` | `_process_jobs_loop(cron_cr, *, job_ids)` |
| Readiness condition | edit `_get_all_ready_jobs` SQL | override `_get_ready_sql_condition` |
| Report progress in action | `self._notify_progress(...)` | `self._commit_progress(...)` (old name deprecated) |
| Batch sizing | tune `MAX_BATCH_PER_CRON_JOB` | tune `MIN_RUNS_PER_JOB` + `MIN_TIME_PER_JOB` |

All other gotchas (SKIP-LOCKED races, version/module-state skips, neutralize guard in `toggle()`,
`_trigger` postcommit, LISTEN/NOTIFY on replica, `--max-cron-threads=0`, deactivation 5/7) are
identical 18↔19.
