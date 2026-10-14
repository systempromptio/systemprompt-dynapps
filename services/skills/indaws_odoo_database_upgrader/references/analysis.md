# FASE 1: Analisis Pre-Upgrade (SQL Queries)

Ejecutar contra la BD restaurada en el contenedor Docker:
`docker exec odoo-upgrade-analyzer-pg{PG} psql -U odoo -d <db_name> -c "<query>"`

## 1.1 Informacion basica

```sql
-- Version y edicion
SELECT key, value FROM ir_config_parameter
WHERE key IN ('database.version', 'database.enterprise_code', 'database.uuid')
ORDER BY key;

-- Tamano de la BD
SELECT pg_size_pretty(pg_database_size(current_database()));

-- Modulos por estado
SELECT state, count(*) FROM ir_module_module GROUP BY state ORDER BY count(*) DESC;

-- Tablas mas grandes (top 20)
SELECT tablename AS tabla,
       pg_size_pretty(pg_total_relation_size(tablename::regclass)) AS tamano,
       pg_total_relation_size(tablename::regclass) AS bytes
FROM pg_tables WHERE schemaname = 'public'
ORDER BY bytes DESC LIMIT 20;
```

## 1.2 Clasificacion de modulos

```sql
SELECT
    name, author, state, latest_version,
    CASE
        WHEN author ILIKE '%Odoo%' OR author ILIKE '%OpenERP%' THEN 'CORE'
        WHEN author ILIKE '%OCA%'  OR author ILIKE '%Tecnativa%'
          OR author ILIKE '%Akretion%'                          THEN 'OCA'
        ELSE 'CUSTOM/TERCERO'
    END AS categoria
FROM ir_module_module
WHERE state = 'installed'
ORDER BY categoria, name;
```

**Categorias y acciones:**

| Categoria | Que hace Upgrade Service | Accion |
|-----------|-------------------------|--------|
| CORE | Migra completamente | Ninguna |
| OCA | Conserva tablas, NO migra codigo | Instalar v_destino post-upgrade |
| CUSTOM | Conserva tablas, NO migra codigo | Migrar codigo manualmente |
| OBSOLETO | Puede causar errores de constraint | Desinstalar ANTES del upgrade |

## 1.3 Modulos problematicos (dependencias eliminadas)

```sql
SELECT m.name, d.name AS depends_on
FROM ir_module_module_dependency d
JOIN ir_module_module m ON m.id = d.module_id
WHERE m.state = 'installed'
AND d.name IN (
    'procurement',              -- eliminado v11
    'account_anglo_saxon',      -- eliminado v13
    'stock_account_inventory',  -- eliminado v16
    'pad',                      -- eliminado v17
    'survey_crm',               -- eliminado v17
    'website_rating'            -- eliminado v16
);
```

## 1.4 Datos candidatos a limpiar (reducir tamano del dump)

```sql
-- Logs del sistema
SELECT count(*), pg_size_pretty(pg_total_relation_size('ir_logging')) FROM ir_logging;

-- Sesiones expiradas (si existe la tabla)
SELECT count(*) FROM ir_sessions WHERE write_date < NOW() - INTERVAL '30 days';

-- Mail messages por tipo
SELECT message_type, count(*) FROM mail_message GROUP BY message_type ORDER BY count(*) DESC;

-- Queue jobs completados
SELECT state, count(*) FROM queue_job GROUP BY state;

-- Attachments huerfanos
SELECT count(*) FROM ir_attachment
WHERE res_model IS NOT NULL
  AND NOT EXISTS (
      SELECT 1 FROM information_schema.tables
      WHERE table_name = replace(res_model, '.', '_')
  );

-- Crons con nextcall muy antiguo
SELECT name, nextcall FROM ir_cron WHERE nextcall < '2020-01-01' AND active = true;
```

## 1.5 Recuento de registros principales

```sql
SELECT 'res_partner'      AS modelo, count(*) FROM res_partner
UNION ALL SELECT 'product_template', count(*) FROM product_template
UNION ALL SELECT 'product_product',  count(*) FROM product_product
UNION ALL SELECT 'sale_order',       count(*) FROM sale_order
UNION ALL SELECT 'purchase_order',   count(*) FROM purchase_order
UNION ALL SELECT 'account_move',     count(*) FROM account_move
UNION ALL SELECT 'account_move_line',count(*) FROM account_move_line
UNION ALL SELECT 'stock_move',       count(*) FROM stock_move
UNION ALL SELECT 'stock_picking',    count(*) FROM stock_picking
ORDER BY 1;
```

## 1.6 Verificacion de integridad basica

```sql
-- Balance contable (debe ser 0 en una BD sana)
SELECT round(sum(debit) - sum(credit), 2) AS desbalance FROM account_move_line;

-- FKs rotas en tablas criticas (ejemplo)
SELECT count(*) FROM sale_order_line WHERE order_id NOT IN (SELECT id FROM sale_order);
SELECT count(*) FROM account_move_line WHERE move_id NOT IN (SELECT id FROM account_move);
```
