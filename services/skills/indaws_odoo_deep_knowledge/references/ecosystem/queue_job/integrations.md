<!-- verified: v18.0 @ 2026-07 (OCA queue 18.0.3.1.3 + cross-refs indaws/cron/mail) -->
# OCA queue_job — Integraciones cross-módulo

## 1. Motor de datos Indaws (hard-dep) — el consumidor nº1 en vuestro parque

`indaws_data_import` e `indaws_data_export` dependen **duro** de queue_job:

- **Import** (`ecosystem/indaws` IG32): la god-class `indaws.import` despacha vía
  `queue_job.with_delay` (bloqueante o async). Parchea `queue.job` `create`/`write` para **auto-avanzar
  la máquina de estados del import** cuando el job pasa a `done`. → Si los jobs no corren (QJG1-QJG4), el
  import se queda colgado a mitad SIN error: no es bug del import, es que el runner está mudo. **Primer
  diagnóstico de "el import no avanza" = verificar el runner** (QJG1: `server_wide_modules`).
- **Export** (`ecosystem/indaws` IG31): `_register_hook` envuelve `create/write/unlink/_compute_field_value`
  en modelos arbitrarios usando el mismo mecanismo de patch que queue_job (`base._patch_method` /
  `_patch_job_auto_delay`). El auto-delay de export usa `context_key` para no disparar en cualquier
  llamada (patrón recomendado, `queue_job/models/base.py:145-228`, `Base._patch_job_auto_delay`). Cross-ref QJG11 (poda de contexto) y QJG12.

Regla operativa: en un cliente Indaws, **queue_job en `server_wide_modules` es requisito de despliegue**,
no opcional. Un staging sin ese `--load` reproduce "imports/exports que no terminan" que no salen en dev.

## 2. Cron (`cross-module/cron`)

- **Autovacuum** de `queue.job` corre desde un `ir.cron` (QJG20). Si ese cron no corre (CRONG1
  `--max-cron-threads=0`, CRONG5 `active=False`), la tabla crece sin límite.
- **Dead-jobs requeuer**: un cron requeuea jobs atascados en `enqueued`/`started` tras un reinicio del
  server (docstring `queue_job/controllers/main.py:35`, `def _prevent_commit`). Complementa QJG19 (los botones no rescatan un `started`).
- queue_job y cron son motores async **distintos**: cron = programado por tiempo/nextcall; queue_job =
  encolado bajo demanda + NOTIFY. Ambos comparten el mismo footgun de "no corre en tests / stop-after-init".

## 3. Mail / Discuss (`functional/mail`)

- `queue.job` hereda `mail.thread` + `mail.activity.mixin`. Un job **fallido** postea en su chatter y
  notifica al grupo *Queue Job Manager* + al creador (QJG18, subtype `mt_job_failed`). En instancias con
  SMTP esto **manda correos** — punto de contacto con la regla "no email desde Odoo": revisa el subtype.
- `_needaction_domain_get` marca los `failed` como "need action" → aparecen en el contador de actividades.

## 4. Connector framework (OCA)

`connector` (sincronización con sistemas externos) se construye ENCIMA de queue_job: cada operación de
sync (import/export de un registro) es un job. Los mismos gotchas de runner/retry/contexto aplican. Si un
conector "no sincroniza", el 90% de las veces es el runner (QJG1) o un job `failed` sin requeue (QJG19).

## 5. Multi-compañía

El job guarda `company_id` (del contexto o `env.company` al crear, `queue_job/job.py:458-462`, `Job.__init__`) y sobrevive
`allowed_company_ids`/`force_company` en la poda de contexto (QJG11). Pero cualquier OTRA lógica de company
que dependa de un ctx-key custom se pierde al encolar → forzar company vía esas 2 claves soportadas o vía
`sudo().with_company()` dentro del método del job.
