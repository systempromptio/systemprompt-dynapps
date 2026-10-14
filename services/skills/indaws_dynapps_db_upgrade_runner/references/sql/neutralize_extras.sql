-- neutralize_extras.sql
-- Neutralización defensiva de credenciales/conectores que la neutralización
-- nativa de Odoo NO cubre. Idempotente: usa DO-blocks que no fallan si la
-- tabla o columna no existe en este cliente/versión.
--
-- Uso: psql <db> -f neutralize_extras.sql
-- Output: NOTICE por cada acción aplicada o saltada.

\set ON_ERROR_STOP off

-- Helper: ejecuta SQL solo si la tabla existe; reporta lo que hace.
CREATE OR REPLACE FUNCTION _neutralize_if_table(p_table text, p_sql text)
RETURNS void LANGUAGE plpgsql AS $$
BEGIN
  IF EXISTS (SELECT 1 FROM information_schema.tables
             WHERE table_schema='public' AND table_name=p_table) THEN
    EXECUTE p_sql;
    RAISE NOTICE 'neutralized: % (%)', p_table, left(p_sql, 60);
  ELSE
    RAISE NOTICE 'skipped (no table): %', p_table;
  END IF;
END $$;

-- Helper: pone una COLUMNA de flag a false SOLO si la columna existe.
-- Necesario porque el nombre del flag varía entre conectores: unos usan
-- `active`, otros `is_active` (ver ZNU: el EDI Transus usa `is_active`, por eso
-- un `SET active=false` NO lo desactivaba y la copia seguía exportando EDI real).
CREATE OR REPLACE FUNCTION _neutralize_flag_col(p_table text, p_col text)
RETURNS void LANGUAGE plpgsql AS $$
BEGIN
  IF EXISTS (SELECT 1 FROM information_schema.columns
             WHERE table_schema='public' AND table_name=p_table
               AND column_name=p_col) THEN
    EXECUTE format('UPDATE %I SET %I = false WHERE %I IS DISTINCT FROM false',
                   p_table, p_col, p_col);
    RAISE NOTICE 'neutralized flag: %.% = false', p_table, p_col;
  ELSE
    RAISE NOTICE 'skipped (no column): %.%', p_table, p_col;
  END IF;
END $$;

-- 1. Servidores de correo.
--    OJO: el neutralize estándar de Odoo desactiva los servidores reales y a
--    continuación INSERTA uno falso ACTIVO ('neutralization - disable emails',
--    host `invalid`, puerto 1025). No es un descuido: sin NINGÚN ir_mail_server,
--    Odoo cae al `smtp_server` del odoo.cfg / línea de comandos
--    (odoo/addons/base/models/ir_mail_server.py:445-451, y el docstring de
--    send_email en :735-736). El servidor falso es la barrera que impide eso.
--    Un `UPDATE ir_mail_server SET active=false` a secas se lleva la barrera por
--    delante — medido el 2026-08-15 en tools/bench-neutralize.
--    Aquí solo se desactiva lo que NO sea el dummy, y se garantiza que exista.
SELECT _neutralize_if_table('ir_mail_server',
  'UPDATE ir_mail_server SET active=false WHERE smtp_host <> ''invalid''');
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM information_schema.tables
             WHERE table_schema='public' AND table_name='ir_mail_server')
     AND NOT EXISTS (SELECT 1 FROM ir_mail_server WHERE smtp_host='invalid' AND active) THEN
    INSERT INTO ir_mail_server (name, smtp_port, smtp_host, smtp_encryption, active, smtp_authentication)
    VALUES ('neutralization - disable emails', 1025, 'invalid', 'none', true, 'login');
    RAISE NOTICE 'repuesto: servidor de correo falso (barrera anti-fallback)';
  END IF;
END $$;
SELECT _neutralize_if_table('fetchmail_server', 'UPDATE fetchmail_server SET active=false');

-- 2. Crons (refuerzo del estándar). El estándar deja vivo `base.autovacuum_job`
--    a propósito (odoo/addons/base/data/neutralize.sql): es limpieza interna y no
--    habla con nadie de fuera. Se respeta.
SELECT _neutralize_if_table('ir_cron',
  'UPDATE ir_cron SET active=false WHERE id NOT IN (
     SELECT res_id FROM ir_model_data
      WHERE model=''ir.cron'' AND name=''autovacuum_job'' AND module=''base'')');

-- 3. Sincronización bancaria
SELECT _neutralize_if_table('account_online_link', 'UPDATE account_online_link SET provider_data=NULL');
SELECT _neutralize_if_table('account_online_account', 'UPDATE account_online_account SET online_identifier=NULL');

-- 4. Conectores EDI / Alpiek (desactivar backend → corta auto-export de
--    albaranes/facturas). El flag varía por conector: probamos AMBAS columnas
--    (`is_active` y `active`); el helper salta la que no exista.
--    ⚠ ZNU: `edi_backend` (Transus) usa `is_active`; con solo `active=false`
--    el conector seguía VIVO y una copia de prod exportaba EDI de verdad.
SELECT _neutralize_flag_col('edi_backend', 'is_active');
SELECT _neutralize_flag_col('edi_backend', 'active');
SELECT _neutralize_flag_col('connector_edi_backend', 'is_active');
SELECT _neutralize_flag_col('connector_edi_backend', 'active');
SELECT _neutralize_flag_col('edi_exchange_type', 'active');
-- Transportistas / carriers: cortar entorno de producción si el flag existe.
SELECT _neutralize_flag_col('delivery_carrier', 'prod_environment');

-- 5. server_environment → forzar entorno local/dev
--    En Dynapps el entorno activo se guarda en ir_config_parameter.
UPDATE ir_config_parameter
   SET value='dev'
 WHERE key IN ('server_environment', 'running_env')
   AND value IS DISTINCT FROM 'dev';

-- 6. Claves sensibles genéricas en ir_config_parameter (API keys, secrets, tokens
--    de servicios externos: transportistas, IAP custom, payment, webhooks).
--    Se blanquean; NO se borran filas (preserva estructura para re-config en local).
UPDATE ir_config_parameter
   SET value=''
 WHERE (
        key ILIKE '%api_key%' OR
        key ILIKE '%apikey%'  OR
        key ILIKE '%secret%'  OR
        key ILIKE '%_token%'  OR
        key ILIKE '%password%' OR
        key ILIKE '%bpost%'   OR
        key ILIKE '%ups%'     OR
        key ILIKE '%client_id%' OR
        key ILIKE '%client_secret%'
       )
   AND key NOT LIKE 'database.%'        -- no tocar uuid/enterprise_code/expiration
   AND value <> '';

-- 7. Base URL de producción → localhost (evita callbacks/webhooks a prod)
UPDATE ir_config_parameter
   SET value='http://localhost:8069'
 WHERE key='web.base.url'
   AND value NOT LIKE 'http://localhost%';

-- 8. Payment providers (refuerzo: a modo test y desactivados)
SELECT _neutralize_if_table('payment_provider',
  'UPDATE payment_provider SET state=''disabled'' WHERE state=''enabled''');

-- Limpieza de helpers
DROP FUNCTION IF EXISTS _neutralize_if_table(text, text);
DROP FUNCTION IF EXISTS _neutralize_flag_col(text, text);

-- Marca de neutralización (refuerzo del estándar)
INSERT INTO ir_config_parameter (key, value)
SELECT 'database.is_neutralized', 'True'
WHERE NOT EXISTS (SELECT 1 FROM ir_config_parameter WHERE key='database.is_neutralized');
UPDATE ir_config_parameter SET value='True' WHERE key='database.is_neutralized';
