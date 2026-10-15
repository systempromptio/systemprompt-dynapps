<!-- verified: v18.0 @ 2026-06 -->
# RPC Services, DB Management & Server (Odoo 18.0)

Source: `$ODOO_SRC/odoo18/odoo/service/` (common.py, db.py, model.py, security.py, server.py)

## RPC Execution Flow

When an external client (web browser, API call) invokes an Odoo method:

```
Client → JSON-RPC/HTTP → http.dispatch_rpc(service, method, params)
       → service.{common|db|object}.dispatch → service/model.py execute_kw()
       → api.call_kw() → Model.method()
```

`dispatch_rpc(service_name, method, params)` lives in `odoo/http.py` (not in
`service/model.py`); it routes `'common'`/`'db'`/`'object'` to the matching
service's `dispatch()`. For the `'object'` service, `dispatch()` runs
`security.check(db, uid, passwd)` then `execute_kw`.

### execute_kw

```python
# service/model.py
execute_kw(db, uid, obj, method, args, kw=None)   # kw is optional (defaults to {})
```

This is what `xmlrpc/2/object` (and the `object` RPC service) calls. It delegates to
`execute(db, uid, obj, method, *args, **kw)` which opens a cursor and runs
`execute_cr`. That:
1. Builds an `Environment(cr, uid, {})` and resolves the model (`env.get(obj)`).
2. Validates the method via `get_public_method`: rejects names starting with `_`,
   methods decorated `@api.private` (`_api_private`), and `_UNSAFE_ATTRIBUTES`
   (raises `AccessError`).
3. Calls `retrying(partial(api.call_kw, ...), env)` to handle serialization errors.
4. Returns the result.

### retrying

```python
# service/model.py
retrying(func, env)
```

Wraps a function call with retry logic. Retries up to
`MAX_TRIES_ON_CONCURRENCY_FAILURE = 5` times on serialization/lock/deadlock
errors (`LockNotAvailable`, `SerializationFailure`, `DeadlockDetected`), with
randomized backoff. On each retry it rolls back, calls `env.reset()` and
`env.registry.reset_changes()`, and rewinds uploaded files. `IntegrityError` is
NOT retried — it is converted to a friendly `ValidationError`. On success it
calls `env.cr.commit()` once at the end. This is why you should NEVER call
`cr.commit()` manually — `retrying` may re-execute your whole call.

---

## Database Management

`from odoo.service.db import *`

These functions manage databases at the server level (create, drop, backup, restore):

```python
exp_create_database(db_name, demo, lang, user_password='admin', login='admin',
                     country_code=None, phone=None)
```
Creates a new database with base module installed.

```python
exp_duplicate_database(db_original_name, db_name, neutralize_database=False)
```
Clones a database. Param is named `neutralize_database` (not `neutralize`).
`neutralize_database=True` removes sensitive data (emails, crons, passwords).

```python
exp_drop(db_name)
```
Deletes database and filestore.

```python
exp_dump(db_name, format)
```
Backup database. Param is named `format` (not `format_`): `'zip'` (DB + filestore)
or `'dump'` (DB only, pg_dump custom format). Returns base64 string.

```python
exp_restore(db_name, data, copy=False)
```
Restore from base64 backup. `copy=True` regenerates the UUID (for cloned
instances). NOTE: in v18 `exp_restore` has NO `neutralize` parameter.

```python
exp_rename(old_name, new_name)
exp_db_exist(db_name) -> bool
exp_list(document=False) -> list
exp_change_admin_password(new_password)
exp_migrate_databases(databases)
exp_server_version()
```

### list_dbs

```python
from odoo.service.db import list_dbs
databases = list_dbs(force=False)  # list available databases (force ignores db_name/dbfilter limits)
```

---

## Authentication

`from odoo.service.security import *`

```python
check(db, uid, passwd)
```
Delegates to `Registry(db)['res.users'].check(db, uid, passwd)`. Raises
`AccessDenied` on failure. Called by the `object`/`common` RPC dispatch.

```python
check_session(session, env, request=None)
```
Recomputes the session token and compares it (constant-time, `consteq`) to the
stored `session.session_token`. Returns `bool`. When `request` is passed it also
updates the device log.

```python
compute_session_token(session, env)
```
Returns `env['res.users'].browse(session.uid)._compute_session_token(session.sid)`.

---

## Server Architecture

### Worker types

| Worker | Purpose | Count |
|--------|---------|-------|
| HTTP Worker | Handles HTTP requests | `--workers N` (default: threads) |
| Cron Worker | Runs scheduled actions | 2 per server |
| Longpolling | Websocket/bus notifications | 1 dedicated (gevent) |

### Server modes

- **Threaded** (default, `--workers 0`): single process, multiple threads. Simple but limited concurrency
- **Prefork** (`--workers N`): master + N worker processes. Production mode. Each worker handles one request at a time

### Request lifecycle

1. Werkzeug receives HTTP request
2. Route is matched against registered controllers
3. Auth is checked (session, token, or none)
4. Dispatcher selected (HTTP or JSON-RPC)
5. Controller method is called with ORM environment
6. Response is returned
7. Transaction is committed (or rolled back on error)

### Key configuration

```
--workers=N          # 0=threaded, N=prefork workers
--limit-memory-soft  # per-worker memory limit (soft kill)
--limit-memory-hard  # per-worker hard memory limit
--limit-time-cpu     # CPU time limit per request
--limit-time-real    # wall time limit per request
--max-cron-threads   # cron worker count (default 2)
--db-filter=regexp   # database name filter per hostname
```
