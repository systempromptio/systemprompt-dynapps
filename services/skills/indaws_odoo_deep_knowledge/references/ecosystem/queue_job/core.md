<!-- verified: v18.0 @ 2026-07 (OCA queue 18.0.3.1.3 — queue_job; verificado vs $OCA_SRC/v18/queue/queue_job) -->
# OCA queue_job — Async Jobs Core (v18.0)

`queue_job` (OCA `queue` repo) es el motor de **jobs asíncronos** de Odoo: encola una llamada a un
método de modelo para ejecutarla fuera de la petición, en un runner dedicado. Es la base de `connector`
y **hard-dep del motor de datos Indaws** (`indaws_data_import`/`indaws_data_export`, ver `gotchas.md` QJG-integ
y ecosystem/indaws IG32/IG31). El valor NO está en la API (`with_delay` es trivial) sino en **cómo el
runner arranca, qué lo deja mudo, cómo reintenta y cómo se ejecuta el job**. Ver `gotchas.md` (QJG).

Manifest: `version 18.0.3.1.3`, `depends: ['mail', 'base_sparse_field', 'web']`, python dep `requests`.

## Arquitectura de ejecución (lo que hay que entender)

1. **Encolar** — `record.with_delay(...).metodo(args)` crea una fila `queue.job` en estado `pending`
   (`models/base.py:with_delay` → `delay.DelayableRecordset`). NO ejecuta nada aún.
2. **Runner** — un hilo/worker que **se monkeypatchea sobre el server de Odoo** al cargar el módulo
   (`queue_job/jobrunner/__init__.py:159-163` reemplaza `ThreadedServer.start`/`PreforkServer.process_spawn`).
   Corre en el proceso principal (threaded) o como `WorkerJobRunner` dedicado (prefork).
3. **Disparo** — el runner hace `LISTEN queue_job` en Postgres (`jobrunner/runner.py:183`, `Database._initialize`) y recibe un
   `NOTIFY` en cada job nuevo; cuando toca ejecutar, lanza un **HTTP GET anónimo** a
   `/queue_job/runjob?db=..&job_uuid=..` contra `ODOO_QUEUE_JOB_HOST:PORT` (default `localhost:8069`,
   `queue/queue_job/jobrunner/runner.py:88-93`, `_async_http_get` y `queue/queue_job/jobrunner/runner.py:318-360`, `QueueJobRunner.__init__`).
4. **Ejecutar** — el endpoint `queue_job.runjob` (`queue_job/controllers/main.py:209-216`, `RunJobController.runjob`, `auth="none"`) carga el
   `Job` y llama `Job.perform()` (`queue_job/job.py:479`, `Job.perform`). El job corre **como el usuario que lo creó**, no sudo.

> ⚠ Corolario operativo: si el paso 2 no arranca (módulo no en `server_wide_modules`, `root:0`,
> `stop_after_init`) o el paso 3 no alcanza un worker HTTP, **los jobs se quedan en `pending` para
> siempre sin error**. Es EL problema nº1 → QJG1/QJG2/QJG3/QJG4.

## Modelo `queue.job` — campos clave (`models/queue_job.py`)

- `_log_access = False` (`:38`) → **NO hay create_date/write_date/create_uid/write_uid**. Ordena/filtra
  por `date_created`/`date_done`/`date_started`, no por los de auditoría (no existen). Ver QJG14.
- `_removal_interval = 30` días, `_default_related_action = "related_action_open_record"` (`:42-43`).
- `state` Selection (`queue_job/job.py:19-35`, `WAIT_DEPENDENCIES`): `wait_dependencies → pending → enqueued → started → done`
  (+ `failed`, `cancelled`). Indexado.
- `uuid` (Char, index), `graph_uuid` (grafos de dependencias), `identity_key` (dedup — QJG8),
  `channel` (Char, index), `priority` (Integer, 0 = más alta, default 10; `aggregator=False`).
- `retry` / `max_retries` (Integer; **vacío/0 = reintentos INFINITOS** — QJG7).
- `records` (`JobSerialized`, el recordset), `args` (tuple), `kwargs` (dict), `func_string` (Char, la
  llamada legible), `exec_time` (Float, `aggregator="avg"`), `eta` ("Execute only after").
- `EDIT_SENTINEL` + `_protected_fields` (`:45-62`): campos núcleo NO escribibles por RPC sin la ctx-key
  `_job_edit_sentinel` → `AccessError`. `create` es `@api.private` (`:239-240`). Ver QJG15.

## Config de ejecución — `queue.job.function` (`models/queue_job_function.py`)

Un registro por `<model>.method` (formato `<res.partner>.foo`, `:16,147-149`) define **channel,
retry_pattern, related_action, allow_commit**. `job_config(name)` es **`@tools.ormcache("name")`**
(`:182`) → cambios requieren limpiar cache (create/write/unlink llaman `env.registry.clear_cache()`).
Sin registro → `job_default_config`: channel `root`, sin retry_pattern, related_action ON,
`allow_commit=False` (`:151-160`). Ver QJG13/QJG16.

## API de encolado (`models/base.py`, `delay.py`)

- `with_delay(priority, eta, max_retries, description, channel, identity_key).metodo(args)` — atajo.
- `delayable()...delay()` — para **cadenas/grafos**: `delay()` debe llamarse en el **padre top-most**
  o los hijos se encolan y el padre nunca (`queue_job/models/base.py:97-109`, `Base.delayable`). Ver QJG12.
- `_patch_job_auto_delay(method, context_key=...)` — parchea un método para auto-encolar al llamarlo
  (mecanismo que usa Indaws; `queue_job/models/base.py:145-228`, `Base._patch_job_auto_delay`). Sin `context_key` es peligroso (side effects).
- Ejecución síncrona (dev/tests): env var `QUEUE_JOB__NO_DELAY` o ctx `queue_job__no_delay`
  (`utils.py:must_run_without_delay`). Ver QJG5/QJG6.

## Ficheros fuente relevantes
`job.py` (clase `Job`, ciclo de vida/retry/eta/identity/grafo) · `delay.py` (Delayable/Graph) ·
`models/queue_job.py` (ORM, requeue/autovacuum/related_action/protected write/mail-on-failure) ·
`models/base.py` (`with_delay`, auto-delay, contexto en enqueue) · `models/queue_job_function.py`
(config cacheada) · `jobrunner/__init__.py` + `jobrunner/runner.py` + `jobrunner/channels.py` (runner) ·
`controllers/main.py` (endpoint runjob) · `utils.py` (modo síncrono) · `exception.py` (Retryable/Failed).
