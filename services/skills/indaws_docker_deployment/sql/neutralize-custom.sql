-- neutralize-custom.sql
--
-- Lo que `odoo-bin neutralize` NO cubre: integraciones que no son de Odoo y
-- que por tanto no traen su propio neutralize.sql.
--
-- Por que hace falta (verificado en indaws_data_export v17):
--   * `_register_hook()` parchea create/write/unlink del modelo destino
--     (codigo adaptado de base_automation) -> dispara al TOCAR un registro,
--     no por cron. Desactivar ir_cron no lo para.
--   * encola con `.with_delay()` en OCA queue_job, que NO trae neutralize.sql
--     y cuyo runner es un server_wide_module con LISTEN/NOTIFY, tampoco un
--     ir_cron. Los jobs pendientes del dump se disparan al arrancar si
--     alguien anade queue_job a server_wide_modules.
--   * el endpoint y el codigo de la llamada viven como REGISTROS en la BD
--     (plantillas con codigo Python), asi que no aparecen ni grepeando el
--     modulo: viajan dentro del dump.
--
-- Idempotente y tolerante: cada bloque comprueba que la tabla/columna exista.

DO $nz$
DECLARE
  n integer;
BEGIN
  -- ── indaws_data_export ──────────────────────────────
  IF to_regclass('public.indaws_export') IS NOT NULL THEN
    UPDATE indaws_export SET active = false WHERE active;
    GET DIAGNOSTICS n = ROW_COUNT;
    RAISE NOTICE 'indaws_export desactivados: %', n;
  END IF;

  IF to_regclass('public.indaws_export_template') IS NOT NULL THEN
    UPDATE indaws_export_template SET active = false WHERE active;
  END IF;

  -- ── indaws_data_import (credenciales SFTP/API + recolectores) ──
  IF to_regclass('public.indaws_import_connection') IS NOT NULL THEN
    UPDATE indaws_import_connection SET active = false WHERE active;
    GET DIAGNOSTICS n = ROW_COUNT;
    RAISE NOTICE 'indaws_import_connection desactivadas: %', n;
  END IF;

  IF to_regclass('public.indaws_import_data_collector') IS NOT NULL THEN
    UPDATE indaws_import_data_collector SET active = false WHERE active;
  END IF;

  IF to_regclass('public.indaws_import') IS NOT NULL
     AND EXISTS (SELECT 1 FROM information_schema.columns
                  WHERE table_name = 'indaws_import' AND column_name = 'active') THEN
    UPDATE indaws_import SET active = false WHERE active;
  END IF;

  -- ── OCA queue_job ───────────────────────────────────
  -- Estados segun queue_job/job.py: wait_dependencies, pending, enqueued,
  -- started, done, cancelled, failed.
  IF to_regclass('public.queue_job') IS NOT NULL THEN
    UPDATE queue_job
       SET state = 'cancelled'
     WHERE state IN ('wait_dependencies', 'pending', 'enqueued', 'started');
    GET DIAGNOSTICS n = ROW_COUNT;
    RAISE NOTICE 'queue_job cancelados: %', n;
  END IF;

  -- ── Webhooks salientes de acciones servidor (v17+) ──
  -- El neutralize oficial ya lo hace en v17+; aqui cubre las versiones
  -- viejas que pasan por el fallback.
  IF to_regclass('public.ir_act_server') IS NOT NULL
     AND EXISTS (SELECT 1 FROM information_schema.columns
                  WHERE table_name = 'ir_act_server' AND column_name = 'webhook_url') THEN
    UPDATE ir_act_server
       SET webhook_url = 'neutralization - disable webhook'
     WHERE webhook_url IS NOT NULL
       AND webhook_url <> 'neutralization - disable webhook';
  END IF;
END
$nz$;
