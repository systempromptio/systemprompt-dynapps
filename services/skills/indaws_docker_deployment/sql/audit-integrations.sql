-- audit-integrations.sql
--
-- Inventario de TODO lo que en esta BD puede emitir o recibir datos hacia
-- fuera. No modifica nada: solo mira y cuenta.
--
-- Existe porque no se puede neutralizar lo que no se ha listado: las
-- integraciones a medida no traen neutralize.sql y muchas viven como
-- REGISTROS (codigo Python en plantillas, endpoints en parametros), asi que
-- no aparecen grepeando el codigo de los modulos.
--
-- Salida por RAISE NOTICE para que un guard to_regclass evite que falte una
-- tabla y se corte el informe a la mitad.

DO $nz$
DECLARE
  n     integer;
  r     record;
  txt   text;
BEGIN
  RAISE NOTICE '--- correo ---';

  SELECT count(*) INTO n FROM ir_mail_server
   WHERE active AND COALESCE(smtp_host, '') <> 'invalid';
  RAISE NOTICE 'servidores SMTP activos      : %', n;
  FOR r IN SELECT name, smtp_host FROM ir_mail_server
            WHERE active AND COALESCE(smtp_host, '') <> 'invalid' LOOP
    RAISE NOTICE '  -> % (%)', r.name, r.smtp_host;
  END LOOP;

  IF to_regclass('public.fetchmail_server') IS NOT NULL THEN
    SELECT count(*) INTO n FROM fetchmail_server WHERE active;
    RAISE NOTICE 'servidores entrantes activos : %', n;
  END IF;

  IF to_regclass('public.mail_mail') IS NOT NULL THEN
    SELECT count(*) INTO n FROM mail_mail WHERE state IN ('outgoing', 'exception');
    RAISE NOTICE 'correos en cola              : %', n;
  END IF;

  RAISE NOTICE '--- programado ---';

  SELECT count(*) INTO n FROM ir_cron c
   WHERE c.active
     AND c.id NOT IN (SELECT res_id FROM ir_model_data
                       WHERE model = 'ir.cron' AND module = 'base' AND name = 'autovacuum_job');
  RAISE NOTICE 'crons activos (sin autovacuum): %', n;
  FOR r IN SELECT c.cron_name FROM ir_cron c
            WHERE c.active
              AND c.id NOT IN (SELECT res_id FROM ir_model_data
                                WHERE model = 'ir.cron' AND module = 'base'
                                  AND name = 'autovacuum_job')
            ORDER BY 1 LIMIT 15 LOOP
    RAISE NOTICE '  -> %', r.cron_name;
  END LOOP;

  RAISE NOTICE '--- acciones y automatismos ---';

  IF EXISTS (SELECT 1 FROM information_schema.columns
              WHERE table_name = 'ir_act_server' AND column_name = 'webhook_url') THEN
    SELECT count(*) INTO n FROM ir_act_server
     WHERE webhook_url IS NOT NULL
       AND webhook_url <> 'neutralization - disable webhook';
    RAISE NOTICE 'webhooks salientes           : %', n;
  END IF;

  IF EXISTS (SELECT 1 FROM information_schema.columns
              WHERE table_name = 'ir_act_server' AND column_name = 'code') THEN
    SELECT count(*) INTO n FROM ir_act_server
     WHERE code IS NOT NULL
       AND (code ILIKE '%requests.%' OR code ILIKE '%urlopen%'
         OR code ILIKE '%ftplib%'    OR code ILIKE '%paramiko%'
         OR code ILIKE '%smtplib%'   OR code ILIKE '%xmlrpc%');
    RAISE NOTICE 'acciones con codigo de red   : %', n;
    FOR r IN SELECT id, COALESCE(name::text, '?') AS name FROM ir_act_server
              WHERE code IS NOT NULL
                AND (code ILIKE '%requests.%' OR code ILIKE '%urlopen%'
                  OR code ILIKE '%ftplib%'    OR code ILIKE '%paramiko%'
                  OR code ILIKE '%smtplib%'   OR code ILIKE '%xmlrpc%')
              LIMIT 10 LOOP
      RAISE NOTICE '  -> [%] %', r.id, r.name;
    END LOOP;
  END IF;

  IF to_regclass('public.base_automation') IS NOT NULL THEN
    SELECT count(*) INTO n FROM base_automation WHERE active;
    RAISE NOTICE 'automatizaciones activas     : % (no se desactivan: romperian el flujo a reproducir)', n;
  END IF;

  RAISE NOTICE '--- integraciones a medida ---';

  IF to_regclass('public.indaws_export') IS NOT NULL THEN
    SELECT count(*) INTO n FROM indaws_export WHERE active;
    RAISE NOTICE 'indaws_export activos        : %', n;
  END IF;
  IF to_regclass('public.indaws_import_connection') IS NOT NULL THEN
    SELECT count(*) INTO n FROM indaws_import_connection WHERE active;
    RAISE NOTICE 'indaws conexiones activas    : %', n;
  END IF;
  IF to_regclass('public.queue_job') IS NOT NULL THEN
    SELECT count(*) INTO n FROM queue_job
     WHERE state IN ('pending', 'enqueued', 'started', 'wait_dependencies');
    RAISE NOTICE 'queue_job pendientes         : %', n;
    FOR r IN SELECT method_name, count(*) AS c FROM queue_job
              WHERE state IN ('pending', 'enqueued', 'started', 'wait_dependencies')
              GROUP BY 1 ORDER BY 2 DESC LIMIT 10 LOOP
      RAISE NOTICE '  -> % (%)', r.method_name, r.c;
    END LOOP;
  END IF;

  RAISE NOTICE '--- pagos y servicios de Odoo ---';

  IF to_regclass('public.payment_provider') IS NOT NULL THEN
    SELECT count(*) INTO n FROM payment_provider WHERE state = 'enabled';
    RAISE NOTICE 'proveedores de pago en PROD  : %', n;
  END IF;
  IF to_regclass('public.iap_account') IS NOT NULL THEN
    SELECT count(*) INTO n FROM iap_account
     WHERE account_token IS NOT NULL AND account_token NOT LIKE '%+disabled';
    RAISE NOTICE 'tokens IAP vivos             : %', n;
  END IF;

  RAISE NOTICE '--- parametros con pinta de endpoint o credencial ---';
  -- Solo la CLAVE, nunca el valor: puede ser un secreto del cliente.
  n := 0;
  FOR r IN SELECT key FROM ir_config_parameter
            WHERE value ILIKE 'http%' OR key ILIKE '%url%' OR key ILIKE '%token%'
               OR key ILIKE '%api%'   OR key ILIKE '%secret%'
            ORDER BY 1 LIMIT 25 LOOP
    n := n + 1;
    RAISE NOTICE '  -> %', r.key;
  END LOOP;
  RAISE NOTICE 'parametros sospechosos       : %', n;
END
$nz$;
