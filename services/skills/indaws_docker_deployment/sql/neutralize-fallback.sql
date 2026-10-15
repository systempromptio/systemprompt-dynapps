-- neutralize-fallback.sql
--
-- Equivalente minimo de `odoo-bin neutralize` para versiones que NO traen ese
-- comando. Verificado: el CLI existe desde la 16.0; en 14.0 y anteriores no.
-- Replica lo esencial de addons/base/data/neutralize.sql sin depender de que
-- existan tablas o columnas de versiones modernas.
--
-- Idempotente. Todo va guardado por to_regclass / information_schema para que
-- valga igual en una v10 que en una v15.

DO $nz$
BEGIN
  -- Servidores de correo saliente
  IF to_regclass('public.ir_mail_server') IS NOT NULL THEN
    UPDATE ir_mail_server SET active = false
     WHERE COALESCE(smtp_host, '') <> 'invalid';
    -- El falso se deja/vuelve a poner activo: si se desactiva junto al resto,
    -- Odoo se queda sin ningun servidor y cae al SMTP de la config (localhost).
    UPDATE ir_mail_server SET active = true WHERE smtp_host = 'invalid';

    -- Servidor falso para que Odoo no caiga en el SMTP de la linea de
    -- comandos (localhost:25) al no encontrar ninguno activo.
    -- El INSERT se construye segun las columnas que existan: v16+ anadio
    -- `smtp_authentication` y ademas es NOT NULL, asi que un INSERT fijo
    -- con las columnas antiguas revienta (comprobado contra un esquema v18:
    -- 'null value in column "smtp_authentication" violates not-null').
    IF NOT EXISTS (SELECT 1 FROM ir_mail_server WHERE smtp_host = 'invalid') THEN
      IF EXISTS (SELECT 1 FROM information_schema.columns
                  WHERE table_name = 'ir_mail_server'
                    AND column_name = 'smtp_authentication') THEN
        INSERT INTO ir_mail_server
               (name, smtp_host, smtp_port, smtp_encryption, active, smtp_authentication)
        VALUES ('neutralization - disable emails', 'invalid', 1025, 'none', true, 'login');
      ELSE
        INSERT INTO ir_mail_server
               (name, smtp_host, smtp_port, smtp_encryption, active)
        VALUES ('neutralization - disable emails', 'invalid', 1025, 'none', true);
      END IF;
    END IF;
  END IF;

  -- Correo entrante
  IF to_regclass('public.fetchmail_server') IS NOT NULL THEN
    UPDATE fetchmail_server SET active = false;
  END IF;

  -- Plantillas apuntando a un servidor concreto
  IF to_regclass('public.mail_template') IS NOT NULL THEN
    UPDATE mail_template SET mail_server_id = NULL;
  END IF;

  -- Cola de correo pendiente del cliente: no debe salir al arrancar
  IF to_regclass('public.mail_mail') IS NOT NULL THEN
    UPDATE mail_mail SET state = 'cancel' WHERE state IN ('outgoing', 'exception');
  END IF;

  -- Acciones planificadas
  IF to_regclass('public.ir_cron') IS NOT NULL THEN
    UPDATE ir_cron SET active = false;
  END IF;

  -- Marca de neutralizacion (la misma clave que usa el neutralize oficial)
  IF to_regclass('public.ir_config_parameter') IS NOT NULL THEN
    INSERT INTO ir_config_parameter (key, value)
    VALUES ('database.is_neutralized', 'true')
    ON CONFLICT (key) DO UPDATE SET value = 'true';
  END IF;
END
$nz$;
