<!-- verified: v18.0 @ 2026-07 (OCA queue 18.0.3.1.3 — queue_job; verificado vs $OCA_SRC/v18/queue/queue_job) -->
# OCA queue_job — Gotchas (QJG)

Prefijo `QJG`. Todo anclado a `queue_job/<fichero>.py → método` (líneas aproximadas, v18.0.3.1.3).

## Runner: por qué los jobs no corren (los 4 grandes)

**QJG1. `queue_job` DEBE estar en `server_wide_modules` (`--load`) o el runner nunca arranca** — el
jobrunner se instala **monkeypatcheando el server** al cargar el módulo *a nivel de servidor*, no de BD
(`queue_job/jobrunner/__init__.py:159-163`). En prefork, si `queue_job` no está en `server_wide_modules`,
`PreforkServer` no tiene atributo `jobrunner` y el spawn se salta (`:123-126,133-136`). Síntoma:
jobs en `pending` para siempre, sin error, sin líneas `starting jobrunner thread` en el log. Fix:
`server_wide_modules = base,web,queue_job` (o `--load=base,web,queue_job`). **Es el gotcha nº1** —
instalar el módulo en la BD NO basta.

**QJG2. `channels = root:0` desactiva el runner entero** — `_is_runner_enabled()` devuelve False si el
canal root arranca con capacidad 0 (`queue_job/jobrunner/__init__.py:89-90`, `_is_runner_enabled` y `queue_job/jobrunner/__init__.py:100-105`, `_start_runner_thread`). El log dice "jobrunner
thread NOT started, because the root channel's capacity is set to 0". Común al copiar config de un
front-web que delega el cron/jobs a otro nodo. Los jobs se acumulan en `pending`.

**QJG3. `--stop-after-init` (updates `-u`/`-i`, tests) NO arranca el runner** (`queue_job/jobrunner/__init__.py:95`, `_start_runner_thread`).
Los jobs encolados durante un `-u modulo` NO se ejecutan en ese proceso: quedan `pending` hasta que
arranca un server normal. No cuentes con que un job dispare durante una migración/actualización.

**QJG4. El runner ejecuta vía HTTP a `localhost:8069` — si no hay worker HTTP alcanzable, nada corre** —
el runner no ejecuta el job en su proceso: hace un GET anónimo a `/queue_job/runjob` contra
`ODOO_QUEUE_JOB_HOST:PORT` (default `localhost` + `http_interface`/`8069`, `queue/queue_job/jobrunner/runner.py:88-93`, `_async_http_get` y `queue/queue_job/jobrunner/runner.py:318-360`, `QueueJobRunner.__init__`;
endpoint `queue_job/controllers/main.py:209-216`, `RunJobController.runjob`). Si Odoo corre `--no-http`, con `workers=0` sin HTTP, detrás de
proxy con puerto/host distinto, o el `http_interface` no es alcanzable desde el runner → jobs `enqueued`
que nunca pasan a `started`. Fix: exponer HTTP y/o fijar `ODOO_QUEUE_JOB_HOST`/`ODOO_QUEUE_JOB_PORT`.

## Modo síncrono / tests

**QJG5. `QUEUE_JOB__NO_DELAY` (env) o `queue_job__no_delay` (ctx) ejecutan el job INLINE, sin encolar** —
`utils.must_run_without_delay` (`queue_job/utils.py:10-21`, `def must_run_without_delay`): con cualquiera de los dos, `with_delay` corre el
método de forma **síncrona** en la misma transacción y NO crea `queue.job`. Imprescindible en dev/tests;
**footgun en prod** (mata toda la asincronía silenciosamente, solo deja un WARNING en el log).

**QJG6. Los jobs NO se ejecutan dentro de un test** — igual que el cron (ver cron/CRONG): el runner no
conduce la BD de test y la transacción no commitea, así que `with_delay` solo deja una fila que nadie
consume. Para testear lógica de jobs: fuerza `queue_job__no_delay` (QJG5) o usa el helper
`queue_job.tests.common.trap_jobs()` (fichero `tests/common.py`) para interceptar y aseverar los jobs
encolados sin ejecutarlos.

## Retry / errores

**QJG7. `max_retries=0` (o vacío) = reintentos INFINITOS, no cero** — `queue/queue_job/job.py:384`, `Job.__init__` y `queue/queue_job/job.py:496`, `Job.perform` (`elif not
self.max_retries: # infinite retries`). El help del campo lo confirma: "Retries are infinite when empty"
(`queue_job/models/queue_job.py:113-118`, `QueueJob.max_retries`). Poner 0 esperando "no reintentar" hace justo lo contrario. Default
`DEFAULT_MAX_RETRIES = 5` (`queue_job/job.py:38`, `DEFAULT_MAX_RETRIES`).

**QJG8. `identity_key` solo deduplica contra jobs en `wait_dependencies/pending/enqueued`** — 
`Job.job_record_with_same_identity_key` busca existentes SOLO en esos 3 estados (`queue_job/job.py:334-347`, `Job.job_record_with_same_identity_key`,
respaldado por índice parcial `queue_job/models/queue_job.py:132-141`, `QueueJob.init`). Un job con la misma key ya en `started`/`failed`/
`done` **NO** bloquea uno nuevo → puedes acabar con duplicados si el primero ya empezó. `identity_exact`
hashea model+method+recordset.ids+args+kwargs (`queue_job/job.py:83-91`, `def identity_exact_hasher`).

**QJG9. `RetryableJobError` reintenta; `ignore_retry=True` NO incrementa el contador** — 
`queue_job/exception.py:21-34`, `class RetryableJobError` + `queue_job/job.py:492-495`, `Job.err`. `RetryableJobError(msg, seconds=, ignore_retry=)`: reintenta a
`seconds`, o al `retry_pattern`, o a `RETRY_INTERVAL` (600s, `queue_job/job.py:39`, `RETRY_INTERVAL`). `FailedJobError` = terminal.
Un `OperationalError` de PG (deadlock/serialization) lo envuelve el controller como retryable
automáticamente (`queue_job/controllers/main.py:167`, `RunJobController._runjob` — `PG_RETRY` se define en `:25`).

**QJG16. `retry_pattern` es "desde el intento N, pospón M segundos"** — dict `{1: 10, 5: 60, 10: 300}`
en la job.function (`queue_job/models/queue_job_function.py:60-70`, `class QueueJobFunction`; aplicado en `job.py:_get_retry_seconds:835-850`). Se
ordena por count y coge el tramo aplicable. Un valor `(min,max)` (tuple/list) **randomiza** el intervalo.
Se edita en la UI de Job Functions; formato validado por constraint (`:206-228`).

## Contexto / usuario / cursor

**QJG10. El job corre COMO EL USUARIO que lo creó, respetando sus record rules** — `Job.perform`
(`queue_job/job.py:479-483`, `Job.perform`), `user_id` almacenado. NO es sudo. Un job encolado por un usuario portal correrá con
sus permisos; si el método necesita más, usa `sudo()` explícito dentro del método (no lo da el motor).

**QJG11. El contexto se PODA al encolar: solo sobreviven 5 claves por defecto** — 
`base.py:_job_prepare_context_before_enqueue_keys` (`:246-261`) guarda SOLO `tz, lang,
allowed_company_ids, force_company, active_test`. Cualquier ctx-key custom (flags de negocio, company
forzada por otra vía, etc.) **se pierde** cuando el job se ejecuta. Fix: override de ese método en tu
modelo para añadir tus claves. Causa típica de "el job hace algo distinto que la llamada directa".

**QJG12. En cadenas/grafos, `delay()` va en el PADRE top-most** — `queue_job/models/base.py:97-109`, `Base.delayable`. Si haces
`job1 = r1.delayable().work(); job2 = r2.delayable().work(); job1.on_done(job2)` y llamas `job2.delay()`,
job2 se encola y **job1 nunca**. `delay()` sobre job1 recorre el grafo y encola todos. Los hijos esperan
en `wait_dependencies` hasta que el padre pasa a `done` (`job.py:add_depends/enqueue_waiting:468-556`,
SQL sobre el JSON `dependencies`).

**QJG13. `allow_commit` corre el job en un CURSOR NUEVO (conexión extra)** — `queue_job/models/queue_job_function.py:83-88`, `class QueueJobFunction`
+ `job.py:in_temporary_env:486-490,511-519`. Off por defecto. Actívalo solo si el job necesita commitear
en medio (overhead: una conexión de BD extra por job). Con él, los commits internos persisten aunque el
job falle después.

## Modelo / config / mantenimiento

**QJG14. `queue.job` tiene `_log_access = False` — no hay create_date/write_uid/etc.** 
(`queue_job/models/queue_job.py:38`, `class QueueJob`). No los uses en dominios, order ni group by (no existen las columnas). Los
timestamps reales son `date_created`, `date_enqueued`, `date_started`, `date_done`, `date_cancelled`.

**QJG15. No puedes crear/editar `queue.job` por RPC: `create` es `@api.private` y hay campos protegidos** —
`queue_job/models/queue_job.py:45-62,239-240,247-257`, `QueueJob`. Escribir `uuid/model_name/method_name/args/kwargs/records/…`
sin la ctx-key `_job_edit_sentinel` (que solo pone `with_delay`) lanza `AccessError`. Es intencionado
(evita fabricar jobs arbitrarios desde fuera). Para encolar usa siempre `with_delay`/`delayable`, nunca
`env['queue.job'].create`.

**QJG17. La config de la job.function está `ormcache`-ada — editarla fuera de ORM no aplica** — 
`queue_job_function.job_config` es `@tools.ormcache("name")` (`:182`). `create`/`write`/`unlink` llaman
`env.registry.clear_cache()` (`:271,276,281`); un `UPDATE` SQL directo a `queue_job_function` NO surte
efecto hasta reiniciar o limpiar cache. Al instalar un módulo, una job.function en XML se **re-liga** a la
existente creada por el registro automático si `install_mode` (`:251-272`).

**QJG18. Un job FALLIDO postea en su chatter y SUSCRIBE+notifica a todo el grupo "Queue Job Manager" + al
creador** — `models/queue_job.py:_message_post_on_failure:352-398` (subtype `mt_job_failed`). En una
instancia con correo saliente esto **genera emails** a los managers en cada fallo. Relevante para la regla
"no enviar correo desde Odoo": desactiva el subtype/notificación o vacía el grupo si no quieres ese ruido.

**QJG19. `requeue()` funciona desde FAILED/DONE/CANCELLED (¡puedes re-ejecutar un job DONE!); `button_done`/
`button_cancelled` NO tocan un job `started`** — `queue_job/models/queue_job.py:329-350`, `QueueJob`. `requeue` → PENDING desde
esos 3 estados (re-corre un DONE si lo requeueas). Los botones manuales solo actúan desde
`wait_dependencies/pending/enqueued/failed`: un job atascado en `started` no se puede forzar a done/cancel
por UI (necesita el dead-jobs-requeuer o intervención).

**QJG20. El autovacuum borra por CANAL, según `removal_interval` del canal (default 30 días)** — 
`models/queue_job.py:autovacuum:407-432` (corre desde un cron). Recorre cada `queue.job.channel`, borra
`done`/`cancelled` más viejos que `channel.removal_interval` en lotes de 1000 con commit. Si la tabla
`queue_job` crece sin control: revisa (a) que el cron de autovacuum corre, (b) el `removal_interval` de
CADA canal (no es global), (c) que no haya jobs `failed` acumulados (esos NO los borra el autovacuum).
