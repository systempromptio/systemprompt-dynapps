<!-- verified: v18.0 @ 2026-07 (OCA queue 18.0.3.1.3 — queue_job) -->
# OCA queue_job — Guía de configuración

## 1. Arrancar el runner (OBLIGATORIO — sin esto nada corre)

`queue_job` debe estar en **`server_wide_modules`**, no solo instalado en la BD (QJG1):

```ini
# odoo.conf
[options]
server_wide_modules = base,web,queue_job
```
o CLI: `odoo --load=base,web,queue_job`. En el log al arrancar debe verse
`starting jobrunner thread` (threaded) o el spawn de `WorkerJobRunner` (prefork). Si no aparece → revisa
`server_wide_modules`, `channels` (QJG2) y que no sea un arranque `--stop-after-init` (QJG3).

## 2. Canales y capacidad (`channels`)

Se configura en la sección `[queue_job]` de `odoo.conf` (leída vía `config.misc`, `queue_job/jobrunner/__init__.py:20`, `queue_job_config`)
o con la env var `ODOO_QUEUE_JOB_CHANNELS` (`queue_job/jobrunner/runner.py:60-62`, `def _channels`):

```ini
[queue_job]
channels = root:4
```
- `root:N` — N jobs simultáneos en el canal raíz. `root:0` **desactiva el runner entero** (QJG2).
- Subcanales: `root:4,root.import:2` — el canal `root.import` toma capacidad propia dentro del root.
- Un job va al canal de su `queue.job.function` (default `root`); se puede sobreescribir con
  `with_delay(channel="root.import")`.

## 3. Runner detrás de proxy / multi-server (QJG4)

El runner alcanza Odoo por HTTP. Si el runner corre en otro proceso/host que el worker HTTP, o hay proxy:
```ini
[queue_job]
scheme = http
host   = localhost      # o el host real del worker HTTP  (ODOO_QUEUE_JOB_HOST)
port   = 8069           # (ODOO_QUEUE_JOB_PORT); default = http_interface/8069
```
Env vars equivalentes: `ODOO_QUEUE_JOB_SCHEME/HOST/PORT`. Para una conexión de BD dedicada del runner:
`ODOO_QUEUE_JOB_JOBRUNNER_DB_{HOST,PORT,USER,PASSWORD}` (`queue_job/jobrunner/runner.py:77-79`, `def _connection_info_for`).

## 4. Config por job function (UI: *Job Queue → Job Functions*)

Por cada `<model>.method` puedes fijar (modelo `queue.job.function`, cacheado — QJG17):
- **Channel** — a qué canal va.
- **Retry Pattern** — `{1: 10, 5: 60, 10: 300}` (desde intento N, pospón M s; `(min,max)` randomiza). QJG16.
- **Related Action** — dict `{"enable": True, "func_name": "...", "kwargs": {...}}`; botón en el job.
- **Allow Commit** — corre en cursor nuevo (overhead, QJG13). Solo si el job necesita commitear en medio.

Editar por UI limpia la ormcache; editar por SQL NO (QJG17).

## 5. Retención / limpieza

El autovacuum borra `done`/`cancelled` por canal según `removal_interval` (default 30 días, en
`queue.job.channel`) desde un cron (QJG20). Ajusta `removal_interval` por canal si la tabla crece. Los
`failed` NO se borran solos: gestiónalos (requeue QJG19 o cancel).

## 6. Dev / tests — ejecución síncrona

```bash
export QUEUE_JOB__NO_DELAY=1     # todos los with_delay corren inline (QJG5)
```
o en código/test: `self.env['x'].with_context(queue_job__no_delay=True)...`. En tests unitarios usa
`queue_job.tests.common.trap_jobs()` para interceptar y aseverar jobs sin ejecutarlos (QJG6).

## 7. Seguridad

Grupo **Queue Job Manager** (`queue_job.group_queue_job_manager`): ve/gestiona todos los jobs y recibe las
notificaciones de fallo (QJG18). Los jobs corren como su creador (QJG10), respetando sus permisos.
