# Errores Comunes del Odoo Upgrade Service

Base de conocimiento de errores frecuentes en migraciones Odoo y sus fixes SQL genericos.
La skill debe consultar este archivo ANTES de pedir al usuario que pegue el error.

---

## Indice de Errores

| # | Categoria | Severidad | Versiones afectadas |
|---|-----------|-----------|---------------------|
| 1 | [FK Constraint Violation](#1-fk-constraint-violation) | BLOCKING | Todas (especial v10→v13) |
| 2 | [Missing Column / Table](#2-missing-column--table) | BLOCKING | v10→v13, v15→v16 |
| 3 | [Data Type Mismatch](#3-data-type-mismatch) | BLOCKING | Todas |
| 4 | [Module Dependency Error](#4-module-dependency-error) | BLOCKING | v10→v11, v15→v16 |
| 5 | [Unique Constraint Violation](#5-unique-constraint-violation) | BLOCKING | Todas (v10→v13+) |
| 6 | [Sequence / Serial Issues](#6-sequence--serial-issues) | BLOCKING | Todas |
| 7 | [Balance Contable Descuadrado](#7-balance-contable-descuadrado) | CRITICAL | Todas |
| 8 | [Custom Constraint Conflicts](#8-custom-constraint-conflicts) | BLOCKING | Todas |
| 9 | [Datos Corruptos (NaN, NULL, fechas)](#9-datos-corruptos) | BLOCKING | Todas |
| 10 | [Workflow / State Migration](#10-workflow--state-migration) | BLOCKING | v10→v11 |
| 11 | [XML/View Corruption](#11-xmlview-corruption) | WARNING-BLOCKING | Todas |
| 12 | [Translation Table Issues](#12-translation-table-issues) | WARNING | v14→v15+ |
| 13 | [Timeout / Memory](#13-timeout--memory) | BLOCKING | BD > 5GB |
| 14 | [Many2many Relation Tables](#14-many2many-relation-tables) | BLOCKING | v16→v17+ |
| 15 | [Filestore References](#15-filestore-references) | WARNING | Todas |

---

## 1. FK Constraint Violation

**Patron regex:**
```
psycopg2\.IntegrityError: insert or update on table "([^"]+)" violates foreign key constraint "([^"]+)"
Key \(([^)]+)\)=\((\d+)\) is not present in table "([^"]+)"
```

**Versiones:** Todas, especialmente v10→v13 (account.invoice → account.move)

**Causa:** Registros con FK apuntando a registros que ya no existen. Comun cuando se han borrado registros padre manualmente (partners, journals, cuentas contables) dejando huerfanos.

**Diagnostico pre-envio:**

```sql
-- Detectar FK huerfanas en las tablas mas problematicas
-- Journal IDs huerfanos en facturas
SELECT 'account_invoice.journal_id' AS campo, count(*) AS huerfanos
FROM account_invoice ai
WHERE NOT EXISTS (SELECT 1 FROM account_journal aj WHERE aj.id = ai.journal_id)
AND ai.journal_id IS NOT NULL
UNION ALL
-- Partner IDs huerfanos en facturas
SELECT 'account_invoice.partner_id', count(*)
FROM account_invoice ai
WHERE NOT EXISTS (SELECT 1 FROM res_partner rp WHERE rp.id = ai.partner_id)
AND ai.partner_id IS NOT NULL
UNION ALL
-- Account IDs huerfanos en apuntes contables
SELECT 'account_move_line.account_id', count(*)
FROM account_move_line aml
WHERE NOT EXISTS (SELECT 1 FROM account_account aa WHERE aa.id = aml.account_id)
AND aml.account_id IS NOT NULL
UNION ALL
-- Currency huerfanas
SELECT 'account_move_line.currency_id', count(*)
FROM account_move_line aml
WHERE NOT EXISTS (SELECT 1 FROM res_currency rc WHERE rc.id = aml.currency_id)
AND aml.currency_id IS NOT NULL
UNION ALL
-- User IDs huerfanos en sale_order
SELECT 'sale_order.user_id', count(*)
FROM sale_order so
WHERE NOT EXISTS (SELECT 1 FROM res_users ru WHERE ru.id = so.user_id)
AND so.user_id IS NOT NULL;
```

**Fix generico:**

```sql
-- =============================================================
-- Fix: FK Constraint Violation - Referencias huerfanas
-- Error: violates foreign key constraint "TABLE_FK_COLUMN_fkey"
-- =============================================================

DO $$
DECLARE
    v_default_id integer;
    v_count integer;
BEGIN
    -- Ejemplo: journal_id huerfano en account_invoice
    IF EXISTS (
        SELECT 1 FROM information_schema.tables WHERE table_name = 'account_invoice'
    ) AND EXISTS (
        SELECT 1 FROM account_invoice ai
        WHERE NOT EXISTS (SELECT 1 FROM account_journal aj WHERE aj.id = ai.journal_id)
        AND ai.journal_id IS NOT NULL
    ) THEN
        -- Obtener journal por defecto (tipo sale)
        SELECT id INTO v_default_id FROM account_journal
        WHERE type = 'sale' AND company_id = 1 LIMIT 1;

        IF v_default_id IS NOT NULL THEN
            UPDATE account_invoice
            SET journal_id = v_default_id
            WHERE journal_id NOT IN (SELECT id FROM account_journal)
            AND journal_id IS NOT NULL;
            GET DIAGNOSTICS v_count = ROW_COUNT;
            RAISE NOTICE 'Fix FK: % facturas con journal_id corregido', v_count;
        END IF;
    ELSE
        RAISE NOTICE 'Fix FK: No hay journal_id huerfanos';
    END IF;
END $$;
```

**Variaciones comunes:**

| Tabla.Campo | Tabla referenciada | Solucion tipica |
|-------------|-------------------|-----------------|
| `account_invoice.journal_id` | `account_journal` | Asignar journal por defecto del mismo tipo |
| `account_invoice.partner_id` | `res_partner` | Crear partner generico o NULL si nullable |
| `account_move_line.account_id` | `account_account` | Asignar cuenta puente (999999) |
| `sale_order.user_id` | `res_users` | Asignar admin (id=2) |
| `purchase_order.partner_id` | `res_partner` | Crear partner generico |
| `stock_move.product_id` | `product_product` | Crear producto generico |
| `account_move_line.tax_line_id` | `account_tax` | SET NULL (es nullable) |

---

## 2. Missing Column / Table

**Patron regex:**
```
psycopg2\.ProgrammingError: column "([^"]+)" of relation "([^"]+)" does not exist
psycopg2\.ProgrammingError: relation "([^"]+)" does not exist
```

**Versiones:** v10→v13 (account.invoice), v15→v16 (stock.inventory), v10→v11 (procurement)

**Causa:** El script de migracion de Odoo intenta acceder a una columna/tabla que un modulo custom o OCA creo y que interfiere con el renombramiento.

**Diagnostico pre-envio:**

```sql
-- Detectar columnas custom en tablas que Odoo va a renombrar/eliminar
-- Estas columnas custom pueden causar conflictos
SELECT c.table_name, c.column_name, c.data_type
FROM information_schema.columns c
WHERE c.table_name IN (
    'account_invoice',          -- Renombrada a account_move (v13)
    'account_invoice_line',     -- Renombrada a account_move_line (v13)
    'account_invoice_tax',      -- Eliminada (v13)
    'procurement_order',        -- Eliminada (v11)
    'stock_pack_operation',     -- Renombrada a stock_move_line (v11)
    'stock_inventory',          -- Eliminada (v16)
    'stock_inventory_line'      -- Eliminada (v16)
)
AND c.column_name LIKE 'x_%'   -- Columnas custom (x_ prefix)
ORDER BY c.table_name, c.column_name;
```

**Fix generico:**

```sql
-- =============================================================
-- Fix: Column/Table does not exist
-- Error: column "X" of relation "Y" does not exist
-- =============================================================

DO $$
BEGIN
    -- Si una tabla custom interfiere con el rename, mover datos a tabla temporal
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'account_invoice' AND column_name = 'x_custom_field'
    ) THEN
        RAISE NOTICE 'Fix: Moviendo campo custom x_custom_field a tabla backup...';

        -- Crear tabla backup si no existe
        CREATE TABLE IF NOT EXISTS _backup_custom_fields AS
        SELECT id, x_custom_field FROM account_invoice WHERE 1=0;

        -- Copiar datos
        INSERT INTO _backup_custom_fields
        SELECT id, x_custom_field FROM account_invoice
        ON CONFLICT DO NOTHING;

        -- Eliminar columna custom que interfiere
        ALTER TABLE account_invoice DROP COLUMN IF EXISTS x_custom_field;

        RAISE NOTICE 'Fix: Campo custom movido a _backup_custom_fields';
    ELSE
        RAISE NOTICE 'Fix: Campo no existe (ya migrado o no necesario)';
    END IF;
END $$;
```

**Tablas criticas que cambian (v10 → v18):**

| Tabla v10 | Destino v18 | Version del cambio |
|-----------|------------|-------------------|
| `account_invoice` | `account_move` | v13 |
| `account_invoice_line` | `account_move_line` | v13 |
| `account_invoice_tax` | Eliminada (lineas tax en `account_move_line`) | v13 |
| `procurement_order` | Eliminada → `stock.rule` + `stock.warehouse.orderpoint` | v11 |
| `stock_pack_operation` | `stock_move_line` | v11 |
| `stock_inventory` | Eliminada → usar `stock.quant` | v16 |
| `stock_inventory_line` | Eliminada → usar `stock.quant` | v16 |
| `wkf_*` (5 tablas) | Eliminadas (workflows eliminados) | v11 |
| `account_account_type` | Eliminada → selection field `account_type` | v15 |

---

## 3. Data Type Mismatch

**Patron regex:**
```
psycopg2\.DataError: invalid input syntax for type (\w+): "([^"]+)"
psycopg2\.DataError: value too long for type character varying\((\d+)\)
column "([^"]+)" is of type (\w+) but expression is of type (\w+)
```

**Versiones:** Todas, especialmente cuando hay datos legacy con formatos incorrectos.

**Causa:** Datos en formato incorrecto para el tipo de campo destino. Comun con campos selection que cambian valores validos, o campos numericos con NaN/Infinity.

**Diagnostico pre-envio:**

```sql
-- NaN en campos contables (MUY COMUN)
SELECT 'account_move_line.debit NaN' AS problema, count(*)
FROM account_move_line WHERE debit = 'NaN'::numeric
UNION ALL
SELECT 'account_move_line.credit NaN', count(*)
FROM account_move_line WHERE credit = 'NaN'::numeric
UNION ALL
SELECT 'account_move_line.amount_currency NaN', count(*)
FROM account_move_line WHERE amount_currency = 'NaN'::numeric;

-- Fechas invalidas
SELECT 'Fechas NULL en facturas' AS problema, count(*)
FROM account_invoice WHERE date_invoice IS NULL AND state != 'draft';

-- Selection fields con valores no validos
-- account_invoice.type debe ser: out_invoice, in_invoice, out_refund, in_refund
SELECT 'account_invoice.type invalido' AS problema, type, count(*)
FROM account_invoice
WHERE type IS NULL OR type NOT IN ('out_invoice','in_invoice','out_refund','in_refund')
GROUP BY type;

-- sale_order.state con valores legacy
SELECT 'sale_order.state invalido' AS problema, state, count(*)
FROM sale_order
WHERE state NOT IN ('draft','sent','sale','done','cancel')
GROUP BY state;
```

**Fix generico:**

```sql
-- =============================================================
-- Fix: Data Type Mismatch - Valores invalidos
-- Error: invalid input syntax for type X: "Y"
-- =============================================================

DO $$
DECLARE
    v_count integer;
BEGIN
    -- Fix NaN en account_move_line
    UPDATE account_move_line SET debit = 0 WHERE debit = 'NaN'::numeric;
    GET DIAGNOSTICS v_count = ROW_COUNT;
    IF v_count > 0 THEN RAISE NOTICE 'Fix: % debit NaN → 0', v_count; END IF;

    UPDATE account_move_line SET credit = 0 WHERE credit = 'NaN'::numeric;
    GET DIAGNOSTICS v_count = ROW_COUNT;
    IF v_count > 0 THEN RAISE NOTICE 'Fix: % credit NaN → 0', v_count; END IF;

    UPDATE account_move_line SET amount_currency = 0 WHERE amount_currency = 'NaN'::numeric;
    GET DIAGNOSTICS v_count = ROW_COUNT;
    IF v_count > 0 THEN RAISE NOTICE 'Fix: % amount_currency NaN → 0', v_count; END IF;

    -- Fix selection fields invalidos en account_invoice
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'account_invoice') THEN
        UPDATE account_invoice SET type = 'out_invoice'
        WHERE type IS NULL OR type NOT IN ('out_invoice','in_invoice','out_refund','in_refund');
        GET DIAGNOSTICS v_count = ROW_COUNT;
        IF v_count > 0 THEN RAISE NOTICE 'Fix: % facturas con type invalido → out_invoice', v_count; END IF;
    END IF;

    -- Fix sale_order.state con valores legacy (shipping_except, invoice_except, manual)
    UPDATE sale_order SET state = 'sale'
    WHERE state IN ('shipping_except', 'invoice_except', 'manual', 'progress');
    GET DIAGNOSTICS v_count = ROW_COUNT;
    IF v_count > 0 THEN RAISE NOTICE 'Fix: % sale_order con state legacy → sale', v_count; END IF;

    -- Fix fechas NULL en facturas confirmadas
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'account_invoice') THEN
        UPDATE account_invoice SET date_invoice = create_date::date
        WHERE date_invoice IS NULL AND state NOT IN ('draft', 'cancel');
        GET DIAGNOSTICS v_count = ROW_COUNT;
        IF v_count > 0 THEN RAISE NOTICE 'Fix: % facturas sin fecha → usando create_date', v_count; END IF;
    END IF;
END $$;
```

**Valores de selection que cambian entre versiones:**

| Campo | Valores v10 | Valores v18 | Accion |
|-------|-------------|-------------|--------|
| `sale.order.state` | draft, sent, sale, done, cancel, **shipping_except**, **invoice_except**, **manual** | draft, sent, sale, done, cancel | Mapear legacy → sale |
| `purchase.order.state` | draft, sent, to approve, purchase, done, cancel, **confirmed**, **approved**, **except_picking**, **except_invoice** | draft, sent, purchase, done, cancel | Mapear legacy → purchase |
| `account.invoice.state` | draft, proforma, proforma2, open, paid, cancel | N/A (usa account.move.state) | proforma→draft |
| `account.move.state` (v13+) | draft, posted, cancel | draft, posted, cancel | OK |
| `stock.picking.state` | draft, waiting, confirmed, partially_available, assigned, done, cancel | draft, waiting, confirmed, assigned, done, cancel | partially_available→confirmed |

---

## 4. Module Dependency Error

**Patron regex:**
```
Module '([^']+)' depends on '([^']+)' which is not available
ModuleNotFound: ([^ ]+)
Dependency.*not satisfied
```

**Versiones:** v10→v11 (procurement), v15→v16 (stock.inventory), v10→v18 (modulos OCA sin version destino)

**Causa:** Modulo instalado depende de otro que fue eliminado o renombrado en la version destino.

**Diagnostico pre-envio:**

```sql
-- Modulos instalados que dependen de modulos eliminados
SELECT m.name AS modulo, d.name AS depende_de,
    CASE d.name
        WHEN 'procurement' THEN 'ELIMINADO v11'
        WHEN 'account_invoice' THEN 'RENOMBRADO v13 → account.move'
        WHEN 'project_issue' THEN 'ELIMINADO v11'
        WHEN 'stock_inventory' THEN 'ELIMINADO v16'
        WHEN 'account_voucher' THEN 'ELIMINADO v14'
        WHEN 'account_bank_statement_import' THEN 'RENOMBRADO v17'
        WHEN 'pad' THEN 'ELIMINADO v17'
        ELSE 'verificar'
    END AS estado_destino
FROM ir_module_module_dependency d
JOIN ir_module_module m ON d.module_id = m.id
WHERE m.state = 'installed'
AND d.name IN (
    'procurement',
    'project_issue',
    'stock_inventory',
    'account_voucher',
    'report',
    'report_xlsx',
    'pad',
    'survey_crm',
    'crm_claim'
)
ORDER BY d.name, m.name;
```

**Fix generico:**

```sql
-- =============================================================
-- Fix: Module Dependency Error - Desinstalar modulos problematicos
-- Error: Module 'X' depends on 'Y' which is not available
-- =============================================================

DO $$
DECLARE
    v_module RECORD;
    v_count integer := 0;
BEGIN
    -- Lista de modulos a desinstalar: dependen de modulos eliminados
    -- o son modulos que no existen en la version destino
    FOR v_module IN
        SELECT m.id, m.name
        FROM ir_module_module m
        WHERE m.state = 'installed'
        AND m.name IN (
            -- Modulos que dependen de procurement (eliminado v11)
            'procurement_analytic',
            'procurement_auto_create_group',
            'stock_procurement_compute_supplier_pnt',
            -- Modulos que dependen de project.issue (eliminado v11)
            'project_issue_task',
            'project_issue_sheet',
            -- Modulos legacy sin v18
            'account_balance_line',
            'account_draft_invoice_print',
            'account_invoice_blocking',
            'account_invoice_force_number',
            'crm_claim',
            'mrp_production_note',
            'partner_contact_gender',
            'partner_contact_lang',
            'sale_proforma_report',
            'version_pnt',
            'stock_picking_show_return'
            -- AÑADIR aqui modulos que fallen
        )
    LOOP
        -- Desinstalar modulo
        UPDATE ir_module_module SET state = 'uninstalled' WHERE id = v_module.id;

        -- Limpiar sus dependencias
        DELETE FROM ir_module_module_dependency WHERE module_id = v_module.id;

        -- Limpiar exclusiones
        DELETE FROM ir_module_module_exclusion WHERE module_id = v_module.id;

        v_count := v_count + 1;
        RAISE NOTICE 'Fix: Desinstalado % (id=%)', v_module.name, v_module.id;
    END LOOP;

    IF v_count > 0 THEN
        RAISE NOTICE 'Fix: % modulos desinstalados', v_count;
    ELSE
        RAISE NOTICE 'Fix: No hay modulos problematicos que desinstalar';
    END IF;
END $$;
```

**Modulos eliminados por version:**

| Version | Modulo eliminado | Sustituto |
|---------|-----------------|-----------|
| v11 | `procurement` | `stock.rule` + `stock.warehouse.orderpoint` |
| v11 | `project_issue` | `project.task` (con `stage_id`) |
| v11 | Workflows (`wkf_*`) | Campo `state` + metodos Python |
| v13 | `account_invoice` (como modulo independiente) | Fusionado en `account` |
| v14 | `account_voucher` | `account.payment` |
| v15 | `account_account_type` (modelo) | Selection field `account_type` |
| v16 | `stock_inventory` | `stock.quant.action_apply_inventory` |
| v17 | `pad` | Eliminado, sin sustituto directo |
| v17 | `account_bank_statement_import` | Renombrado |

---

## 5. Unique Constraint Violation

**Patron regex:**
```
psycopg2\.IntegrityError: duplicate key value violates unique constraint "([^"]+)"
Key \(([^)]+)\)=\(([^)]+)\) already exists
```

**Versiones:** Todas, especialmente v10→v13 (numero factura unico)

**Causa:** Datos duplicados en campos que ganan restriccion UNIQUE en la version destino. Comun con numeros de factura, codigos de producto, XMLIDs.

**Diagnostico pre-envio:**

```sql
-- Numeros de factura duplicados (critico para v13+)
SELECT number, journal_id, count(*)
FROM account_invoice
WHERE number IS NOT NULL AND state != 'cancel'
GROUP BY number, journal_id
HAVING count(*) > 1;

-- External IDs duplicados (ir.model.data)
SELECT module, name, count(*)
FROM ir_model_data
WHERE module != '__export__'
GROUP BY module, name
HAVING count(*) > 1;

-- Barcodes de producto duplicados
SELECT barcode, count(*)
FROM product_product
WHERE barcode IS NOT NULL AND barcode != ''
GROUP BY barcode
HAVING count(*) > 1;

-- Codigos de cuenta contable duplicados por compania
SELECT code, company_id, count(*)
FROM account_account
GROUP BY code, company_id
HAVING count(*) > 1;

-- VAT duplicados en partners
SELECT vat, count(*)
FROM res_partner
WHERE vat IS NOT NULL AND vat != ''
GROUP BY vat
HAVING count(*) > 1;
```

**Fix generico:**

```sql
-- =============================================================
-- Fix: Unique Constraint Violation - Duplicados
-- Error: duplicate key value violates unique constraint "X"
-- =============================================================

DO $$
DECLARE
    v_dup RECORD;
    v_count integer := 0;
BEGIN
    -- Fix 1: Numeros de factura duplicados
    FOR v_dup IN
        SELECT number, journal_id, count(*) as cnt
        FROM account_invoice
        WHERE number IS NOT NULL AND state != 'cancel'
        GROUP BY number, journal_id
        HAVING count(*) > 1
    LOOP
        -- Renumerar duplicados excepto el primero (por id mas bajo)
        WITH numbered AS (
            SELECT id, ROW_NUMBER() OVER (ORDER BY id) as rn
            FROM account_invoice
            WHERE number = v_dup.number AND journal_id = v_dup.journal_id
        )
        UPDATE account_invoice
        SET number = number || '-DUP' || n.rn
        FROM numbered n
        WHERE account_invoice.id = n.id AND n.rn > 1;

        v_count := v_count + 1;
    END LOOP;
    IF v_count > 0 THEN RAISE NOTICE 'Fix: % numeros de factura duplicados resueltos', v_count; END IF;

    -- Fix 2: XMLIDs duplicados
    v_count := 0;
    WITH dups AS (
        SELECT module, name, MIN(id) as keep_id
        FROM ir_model_data
        WHERE module != '__export__'
        GROUP BY module, name
        HAVING count(*) > 1
    )
    DELETE FROM ir_model_data d
    USING dups
    WHERE d.module = dups.module AND d.name = dups.name AND d.id != dups.keep_id;
    GET DIAGNOSTICS v_count = ROW_COUNT;
    IF v_count > 0 THEN RAISE NOTICE 'Fix: % XMLIDs duplicados eliminados (conservando el mas antiguo)', v_count; END IF;

    -- Fix 3: Barcodes duplicados
    v_count := 0;
    WITH dups AS (
        SELECT barcode, MIN(id) as keep_id
        FROM product_product
        WHERE barcode IS NOT NULL AND barcode != ''
        GROUP BY barcode
        HAVING count(*) > 1
    )
    UPDATE product_product pp
    SET barcode = NULL
    FROM dups d
    WHERE pp.barcode = d.barcode AND pp.id != d.keep_id;
    GET DIAGNOSTICS v_count = ROW_COUNT;
    IF v_count > 0 THEN RAISE NOTICE 'Fix: % barcodes duplicados → NULL', v_count; END IF;
END $$;
```

---

## 6. Sequence / Serial Issues

**Patron regex:**
```
sequence "([^"]+)" does not exist
nextval\('([^']+)'\) is not defined
currval.*not yet defined
duplicate key value violates unique constraint.*_pkey
```

**Versiones:** Todas (despues de manipular datos o restaurar dumps parciales)

**Causa:** Las secuencias de PostgreSQL se desincronizaron con los datos reales, causando colisiones de ID.

**Diagnostico pre-envio:**

```sql
-- Detectar secuencias desincronizadas
SELECT
    t.table_name,
    pg_get_serial_sequence(t.table_name, 'id') as sequence_name,
    (SELECT MAX(id) FROM pg_catalog.pg_class c
     JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
     WHERE c.relname = t.table_name) as info
FROM information_schema.tables t
WHERE t.table_schema = 'public'
AND t.table_type = 'BASE TABLE'
AND EXISTS (
    SELECT 1 FROM information_schema.columns
    WHERE table_name = t.table_name AND column_name = 'id'
)
LIMIT 10;

-- Verificar secuencia especifica vs max id
SELECT
    'account_move' as tabla,
    (SELECT MAX(id) FROM account_move) as max_id,
    (SELECT last_value FROM account_move_id_seq) as seq_value,
    CASE WHEN (SELECT MAX(id) FROM account_move) > (SELECT last_value FROM account_move_id_seq)
        THEN 'DESINCRONIZADA' ELSE 'OK' END as estado;
```

**Fix generico:**

```sql
-- =============================================================
-- Fix: Sequence Desynchronization
-- Error: duplicate key value violates unique constraint "X_pkey"
-- =============================================================

DO $$
DECLARE
    v_table RECORD;
    v_max_id bigint;
    v_seq_name text;
BEGIN
    -- Resincronizar TODAS las secuencias de tablas con campo id
    FOR v_table IN
        SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = 'public' AND table_type = 'BASE TABLE'
        AND EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_name = information_schema.tables.table_name
            AND column_name = 'id'
            AND data_type IN ('integer', 'bigint')
        )
    LOOP
        v_seq_name := pg_get_serial_sequence(v_table.table_name, 'id');
        IF v_seq_name IS NOT NULL THEN
            EXECUTE format('SELECT COALESCE(MAX(id), 0) FROM %I', v_table.table_name) INTO v_max_id;
            EXECUTE format('SELECT setval(%L, GREATEST(%s + 1, 1))', v_seq_name, v_max_id);
        END IF;
    END LOOP;
    RAISE NOTICE 'Fix: Todas las secuencias resincronizadas';
END $$;
```

---

## 7. Balance Contable Descuadrado

**Patron regex:**
```
balance.*not zero
descuadre
debit.*!=.*credit
Unbalanced entries
```

**Versiones:** Todas (CRITICO en v10→v13 por merge invoice→move)

**Causa:** El upgrade crea/modifica asientos contables y la suma total debe=haber queda descuadrada. Puede ser por datos corruptos pre-existentes o por errores en la migracion.

**Diagnostico pre-envio:**

```sql
-- CRITICO: Ejecutar ANTES y DESPUES del upgrade
-- 1. Balance global
SELECT
    COALESCE(SUM(debit), 0) AS total_debe,
    COALESCE(SUM(credit), 0) AS total_haber,
    COALESCE(SUM(debit), 0) - COALESCE(SUM(credit), 0) AS descuadre
FROM account_move_line;

-- 2. Asientos individuales descuadrados
SELECT move_id,
    SUM(debit) AS debe,
    SUM(credit) AS haber,
    SUM(debit) - SUM(credit) AS descuadre
FROM account_move_line
GROUP BY move_id
HAVING ABS(SUM(debit) - SUM(credit)) > 0.01
ORDER BY ABS(SUM(debit) - SUM(credit)) DESC
LIMIT 20;

-- 3. Apuntes contables sin asiento padre
SELECT count(*) AS apuntes_huerfanos
FROM account_move_line aml
WHERE NOT EXISTS (SELECT 1 FROM account_move am WHERE am.id = aml.move_id);
```

**Fix generico:**

```sql
-- =============================================================
-- Fix: Balance Contable - Asientos descuadrados
-- Error: Balance not zero / Unbalanced entries
-- PRECAUCION: Solo aplicar si el descuadre es < 1.00 (redondeos)
-- Para descuadres mayores, investigar manualmente
-- =============================================================

DO $$
DECLARE
    v_balance numeric;
    v_orphans integer;
    v_imbalanced RECORD;
BEGIN
    -- 1. Eliminar apuntes huerfanos (sin asiento padre)
    DELETE FROM account_move_line
    WHERE move_id NOT IN (SELECT id FROM account_move);
    GET DIAGNOSTICS v_orphans = ROW_COUNT;
    IF v_orphans > 0 THEN
        RAISE NOTICE 'Fix: % apuntes huerfanos eliminados', v_orphans;
    END IF;

    -- 2. Fix NaN en debit/credit (causa comun de descuadre)
    UPDATE account_move_line SET debit = 0 WHERE debit = 'NaN'::numeric;
    UPDATE account_move_line SET credit = 0 WHERE credit = 'NaN'::numeric;

    -- 3. Verificar balance global
    SELECT COALESCE(SUM(debit - credit), 0) INTO v_balance FROM account_move_line;
    IF ABS(v_balance) > 0.01 THEN
        RAISE WARNING 'Balance descuadrado: %. Investigar asientos individuales.', v_balance;

        -- Listar los 5 asientos mas descuadrados
        FOR v_imbalanced IN
            SELECT move_id, SUM(debit - credit) AS diff
            FROM account_move_line
            GROUP BY move_id
            HAVING ABS(SUM(debit - credit)) > 0.01
            ORDER BY ABS(SUM(debit - credit)) DESC
            LIMIT 5
        LOOP
            RAISE WARNING 'Asiento descuadrado: move_id=%, diff=%', v_imbalanced.move_id, v_imbalanced.diff;
        END LOOP;
    ELSE
        RAISE NOTICE 'Fix: Balance contable OK (descuadre: %)', v_balance;
    END IF;
END $$;
```

**IMPORTANTE:** No crear lineas de balance automaticamente para descuadres grandes (>1.00). En ese caso, reportar al usuario para investigacion manual. Los descuadres por redondeo (<0.01) son aceptables.

---

## 8. Custom Constraint Conflicts

**Patron regex:**
```
constraint "([^"]+)" of relation "([^"]+)" does not exist
cannot drop constraint .* on table .* because other objects depend on it
violates check constraint "([^"]+)"
```

**Versiones:** Todas (modulos OCA/custom crean constraints que interfieren con el upgrade)

**Causa:** Modulos custom/OCA crearon constraints (CHECK, UNIQUE, FK) sobre tablas core de Odoo. Cuando el Upgrade Service intenta modificar esas tablas, las constraints bloquean la operacion.

**Diagnostico pre-envio:**

```sql
-- Constraints custom sobre tablas core de Odoo
SELECT
    tc.table_name,
    tc.constraint_name,
    tc.constraint_type,
    'POSIBLE CONFLICTO' as nota
FROM information_schema.table_constraints tc
WHERE tc.table_schema = 'public'
AND tc.constraint_type IN ('FOREIGN KEY', 'CHECK', 'UNIQUE')
AND tc.table_name IN (
    'account_invoice', 'account_invoice_line', 'account_move', 'account_move_line',
    'sale_order', 'sale_order_line', 'purchase_order', 'purchase_order_line',
    'stock_picking', 'stock_move', 'stock_move_line',
    'res_partner', 'product_template', 'product_product'
)
-- Excluir constraints propias de Odoo (empiezan por el nombre de la tabla)
AND tc.constraint_name NOT LIKE tc.table_name || '_%'
ORDER BY tc.table_name;

-- Triggers custom
SELECT trigger_name, event_object_table, action_timing, event_manipulation
FROM information_schema.triggers
WHERE trigger_schema = 'public'
AND event_object_table IN (
    'account_invoice', 'account_move', 'account_move_line',
    'sale_order', 'stock_picking', 'res_partner'
)
ORDER BY event_object_table;
```

**Fix generico:**

```sql
-- =============================================================
-- Fix: Custom Constraint blocking upgrade
-- Error: cannot drop/violates constraint "X"
-- =============================================================

DO $$
DECLARE
    v_constraint RECORD;
    v_count integer := 0;
BEGIN
    -- Eliminar constraints custom sobre tablas que el upgrade va a modificar
    FOR v_constraint IN
        SELECT tc.table_name, tc.constraint_name, tc.constraint_type
        FROM information_schema.table_constraints tc
        WHERE tc.table_schema = 'public'
        AND tc.constraint_type IN ('FOREIGN KEY', 'CHECK')
        AND tc.table_name IN ('account_invoice', 'account_invoice_line',
                              'procurement_order', 'stock_pack_operation',
                              'stock_inventory', 'stock_inventory_line')
        -- Solo constraints que NO empiezan por el nombre de tabla (son custom)
        AND tc.constraint_name NOT LIKE tc.table_name || '_%_fkey'
        AND tc.constraint_name NOT LIKE tc.table_name || '_pkey'
    LOOP
        EXECUTE format('ALTER TABLE %I DROP CONSTRAINT IF EXISTS %I',
                       v_constraint.table_name, v_constraint.constraint_name);
        v_count := v_count + 1;
        RAISE NOTICE 'Fix: Eliminada constraint % de %',
                     v_constraint.constraint_name, v_constraint.table_name;
    END LOOP;

    -- Eliminar triggers custom sobre las mismas tablas
    FOR v_constraint IN
        SELECT trigger_name, event_object_table
        FROM information_schema.triggers
        WHERE trigger_schema = 'public'
        AND event_object_table IN ('account_invoice', 'account_invoice_line',
                                    'procurement_order', 'stock_inventory')
    LOOP
        EXECUTE format('DROP TRIGGER IF EXISTS %I ON %I',
                       v_constraint.trigger_name, v_constraint.event_object_table);
        RAISE NOTICE 'Fix: Eliminado trigger % de %',
                     v_constraint.trigger_name, v_constraint.event_object_table;
    END LOOP;

    RAISE NOTICE 'Fix: % constraints/triggers eliminados', v_count;
END $$;
```

---

## 9. Datos Corruptos

**Patron regex:**
```
DataError.*NaN
invalid byte sequence
could not convert string to float
invalid input syntax for type (date|timestamp|numeric|integer)
null value in column .* violates not-null constraint
```

**Versiones:** Todas

**Causa:** Datos corruptos preexistentes en la BD que el Upgrade Service no puede procesar.

**Fix generico (limpieza completa):**

```sql
-- =============================================================
-- Fix: Datos Corruptos - Limpieza integral
-- Ejecutar ANTES del primer envio al Upgrade Service
-- =============================================================

DO $$
DECLARE
    v_count integer;
BEGIN
    -- 1. NaN en campos numericos contables
    UPDATE account_move_line SET debit = 0 WHERE debit = 'NaN'::numeric;
    UPDATE account_move_line SET credit = 0 WHERE credit = 'NaN'::numeric;
    UPDATE account_move_line SET amount_currency = 0 WHERE amount_currency = 'NaN'::numeric;
    UPDATE account_move_line SET quantity = 0 WHERE quantity = 'NaN'::numeric;

    -- 2. Infinity en campos numericos
    UPDATE account_move_line SET debit = 0 WHERE debit = 'Infinity'::numeric;
    UPDATE account_move_line SET credit = 0 WHERE credit = 'Infinity'::numeric;

    -- 3. NULL en campos NOT NULL criticos
    -- company_id (obligatorio en casi todas las tablas)
    UPDATE account_move SET company_id = 1 WHERE company_id IS NULL;
    UPDATE account_move_line SET company_id = 1 WHERE company_id IS NULL;
    UPDATE sale_order SET company_id = 1 WHERE company_id IS NULL;
    UPDATE purchase_order SET company_id = 1 WHERE company_id IS NULL;

    -- 4. Campos date con formato invalido (detectar via cast)
    -- No se puede UPDATE generico, pero limpiar los conocidos:
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'account_invoice') THEN
        UPDATE account_invoice SET date_invoice = create_date::date
        WHERE date_invoice IS NULL AND state IN ('open', 'paid');
        GET DIAGNOSTICS v_count = ROW_COUNT;
        IF v_count > 0 THEN RAISE NOTICE 'Fix: % facturas sin fecha → create_date', v_count; END IF;

        UPDATE account_invoice SET date_due = date_invoice
        WHERE date_due IS NULL AND date_invoice IS NOT NULL;
    END IF;

    -- 5. Limpiar caracteres nulos embebidos (causan errores en pg_dump/restore)
    UPDATE res_partner SET name = REPLACE(name, chr(0), '') WHERE name LIKE '%' || chr(0) || '%';
    UPDATE res_partner SET email = REPLACE(email, chr(0), '') WHERE email LIKE '%' || chr(0) || '%';

    -- 6. Campos selection con valores vacios (string vacio en lugar de NULL)
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'account_invoice') THEN
        UPDATE account_invoice SET state = 'draft' WHERE state = '' OR state IS NULL;
        UPDATE account_invoice SET type = 'out_invoice' WHERE type = '' OR type IS NULL;
    END IF;

    RAISE NOTICE 'Fix: Limpieza de datos corruptos completada';
END $$;
```

---

## 10. Workflow / State Migration

**Patron regex:**
```
relation "wkf.*" does not exist
WorkflowError
column .* of relation "wkf_.*" does not exist
```

**Versiones:** v10→v11 (workflows eliminados por completo)

**Causa:** Odoo v10 usa el motor de workflows (tablas `wkf_*`). Desde v11, los workflows fueron eliminados y reemplazados por campo `state` + metodos Python. Modulos custom que referencian workflows causan errores.

**Diagnostico pre-envio:**

```sql
-- Verificar existencia de tablas workflow (deben existir en v10)
SELECT table_name FROM information_schema.tables
WHERE table_name LIKE 'wkf%'
ORDER BY table_name;

-- Verificar modulos que extienden workflows
SELECT name, state FROM ir_module_module
WHERE state = 'installed'
AND name IN (
    SELECT DISTINCT module FROM ir_model_data
    WHERE model = 'workflow' OR model LIKE 'workflow.%'
);
```

**Fix generico:**

```sql
-- =============================================================
-- Fix: Workflow Legacy - Limpiar datos de workflow
-- Error: relation "wkf_*" does not exist
-- Nota: Solo aplicar si la BD es v10 y el upgrade falla por workflows
-- =============================================================

DO $$
BEGIN
    -- Verificar que estamos en v10 (tablas wkf existen)
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'wkf') THEN
        -- Limpiar datos de workflow (el Upgrade Service deberia hacerlo,
        -- pero modulos custom pueden interferir)
        RAISE NOTICE 'Fix: Tablas wkf detectadas (v10). Limpiando datos de workflow...';

        -- Limpiar instancias de workflow huerfanas
        DELETE FROM wkf_workitem wi
        WHERE NOT EXISTS (
            SELECT 1 FROM wkf_instance i WHERE i.id = wi.inst_id
        );

        DELETE FROM wkf_instance
        WHERE res_type NOT IN (
            SELECT model FROM ir_model
        );

        RAISE NOTICE 'Fix: Datos de workflow limpiados';
    ELSE
        RAISE NOTICE 'Fix: No hay tablas wkf (no es v10, no necesario)';
    END IF;
END $$;
```

---

## 11. XML/View Corruption

**Patron regex:**
```
ParseError.*not well-formed
invalid XML
View .* does not exist
ir\.ui\.view .* has no parent
```

**Versiones:** Todas (vistas custom o OCA que referencian elementos eliminados)

**Causa:** Vistas XML almacenadas en `ir_ui_view` que referencian campos, modelos o herencia que ya no existen en la version destino.

**Diagnostico pre-envio:**

```sql
-- Vistas sin arch_db (corruptas)
SELECT id, name, model, type
FROM ir_ui_view
WHERE arch_db IS NULL AND type != 'qweb';

-- Vistas que heredan de vistas que no existen
SELECT v.id, v.name, v.model, v.inherit_id
FROM ir_ui_view v
WHERE v.inherit_id IS NOT NULL
AND NOT EXISTS (SELECT 1 FROM ir_ui_view p WHERE p.id = v.inherit_id);

-- Vistas de modulos custom que podrian interferir
SELECT v.id, v.name, v.model, md.module
FROM ir_ui_view v
JOIN ir_model_data md ON md.res_id = v.id AND md.model = 'ir.ui.view'
WHERE md.module NOT IN (
    SELECT name FROM ir_module_module WHERE author LIKE '%Odoo%'
)
AND v.active = true
ORDER BY md.module;
```

**Fix generico:**

```sql
-- =============================================================
-- Fix: XML/View Corruption
-- Error: ParseError / View does not exist / has no parent
-- =============================================================

DO $$
DECLARE
    v_count integer;
BEGIN
    -- 1. Desactivar vistas sin arch_db
    UPDATE ir_ui_view SET active = false
    WHERE arch_db IS NULL AND type != 'qweb' AND active = true;
    GET DIAGNOSTICS v_count = ROW_COUNT;
    IF v_count > 0 THEN RAISE NOTICE 'Fix: % vistas sin arch_db desactivadas', v_count; END IF;

    -- 2. Desactivar vistas con parent huerfano
    UPDATE ir_ui_view SET active = false
    WHERE inherit_id IS NOT NULL
    AND NOT EXISTS (SELECT 1 FROM ir_ui_view p WHERE p.id = inherit_id)
    AND active = true;
    GET DIAGNOSTICS v_count = ROW_COUNT;
    IF v_count > 0 THEN RAISE NOTICE 'Fix: % vistas con parent huerfano desactivadas', v_count; END IF;

    -- 3. Desactivar vistas de modulos ya desinstalados
    UPDATE ir_ui_view v SET active = false
    FROM ir_model_data md
    JOIN ir_module_module m ON m.name = md.module
    WHERE md.res_id = v.id AND md.model = 'ir.ui.view'
    AND m.state = 'uninstalled' AND v.active = true;
    GET DIAGNOSTICS v_count = ROW_COUNT;
    IF v_count > 0 THEN RAISE NOTICE 'Fix: % vistas de modulos desinstalados desactivadas', v_count; END IF;

    RAISE NOTICE 'Fix: Limpieza de vistas completada';
END $$;
```

---

## 12. Translation Table Issues

**Patron regex:**
```
relation "ir_translation" does not exist
column "ir_translation\..+" does not exist
duplicate key.*ir_translation
```

**Versiones:** v14→v15+ (ir.translation se elimina, traducciones migran a campos `_t`)

**Causa:** En Odoo 15+, las traducciones ya no se almacenan en `ir_translation` sino directamente en los campos de cada modelo (sufijo `_t`). Datos legacy en `ir_translation` pueden causar conflictos.

**Fix generico:**

```sql
-- =============================================================
-- Fix: Translation table duplicates
-- Error: duplicate key in ir_translation
-- Nota: Aplicar SOLO si el error es sobre ir_translation
-- =============================================================

DO $$
DECLARE
    v_count integer;
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'ir_translation') THEN
        -- Eliminar traducciones duplicadas (conservar la mas reciente)
        WITH dups AS (
            SELECT id, ROW_NUMBER() OVER (
                PARTITION BY type, name, lang, res_id, src
                ORDER BY write_date DESC NULLS LAST, id DESC
            ) as rn
            FROM ir_translation
        )
        DELETE FROM ir_translation WHERE id IN (
            SELECT id FROM dups WHERE rn > 1
        );
        GET DIAGNOSTICS v_count = ROW_COUNT;
        IF v_count > 0 THEN
            RAISE NOTICE 'Fix: % traducciones duplicadas eliminadas', v_count;
        ELSE
            RAISE NOTICE 'Fix: No hay traducciones duplicadas';
        END IF;

        -- Eliminar traducciones de modelos que ya no existen
        DELETE FROM ir_translation
        WHERE type = 'model'
        AND split_part(name, ',', 1) NOT IN (SELECT model FROM ir_model);
        GET DIAGNOSTICS v_count = ROW_COUNT;
        IF v_count > 0 THEN
            RAISE NOTICE 'Fix: % traducciones de modelos inexistentes eliminadas', v_count;
        END IF;
    ELSE
        RAISE NOTICE 'Fix: ir_translation no existe (ya migrado a campos _t)';
    END IF;
END $$;
```

---

## 13. Timeout / Memory

**Patron regex:**
```
MemoryError
TimeoutError
state: failed.*timeout
Killed.*signal 9
```

**Versiones:** BD > 5GB, especialmente con muchos attachments o historico grande.

**Causa:** La BD es demasiado grande para el tiempo/memoria asignados por el Upgrade Service.

**Fix: Reducir tamano antes de enviar:**

```sql
-- =============================================================
-- Fix: Reducir tamano de BD (anti-timeout)
-- Ejecutar ANTES del envio si la BD > 5GB
-- PRECAUCION: Solo eliminar datos no criticos
-- =============================================================

DO $$
DECLARE
    v_count integer;
    v_size_before text;
    v_size_after text;
BEGIN
    SELECT pg_size_pretty(pg_database_size(current_database())) INTO v_size_before;
    RAISE NOTICE 'Tamano BD antes: %', v_size_before;

    -- 1. Logs del sistema (pueden pesar GB)
    DELETE FROM ir_logging;
    GET DIAGNOSTICS v_count = ROW_COUNT;
    RAISE NOTICE 'Eliminados % registros de ir_logging', v_count;

    -- 2. Sesiones expiradas
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'ir_sessions') THEN
        DELETE FROM ir_sessions WHERE write_date < NOW() - INTERVAL '7 days';
        GET DIAGNOSTICS v_count = ROW_COUNT;
        RAISE NOTICE 'Eliminadas % sesiones expiradas', v_count;
    END IF;

    -- 3. Auditlog antiguo (> 1 ano)
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'auditlog_log') THEN
        DELETE FROM auditlog_log_line
        WHERE log_id IN (SELECT id FROM auditlog_log WHERE create_date < NOW() - INTERVAL '1 year');
        DELETE FROM auditlog_log WHERE create_date < NOW() - INTERVAL '1 year';
        GET DIAGNOSTICS v_count = ROW_COUNT;
        RAISE NOTICE 'Eliminados % registros de auditlog (> 1 ano)', v_count;
    END IF;

    -- 4. Queue jobs completados (> 6 meses)
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'queue_job') THEN
        DELETE FROM queue_job WHERE state = 'done' AND date_done < NOW() - INTERVAL '6 months';
        GET DIAGNOSTICS v_count = ROW_COUNT;
        RAISE NOTICE 'Eliminados % queue_jobs completados', v_count;
    END IF;

    -- 5. Mail notifications antiguas (> 2 anos, solo tipo notification)
    DELETE FROM mail_message
    WHERE date < NOW() - INTERVAL '2 years'
    AND message_type = 'notification'
    AND subtype_id IS NOT NULL;
    GET DIAGNOSTICS v_count = ROW_COUNT;
    RAISE NOTICE 'Eliminadas % notificaciones mail (> 2 anos)', v_count;

    -- 6. Mail tracking antiguo
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'mail_tracking_value') THEN
        DELETE FROM mail_tracking_value
        WHERE mail_message_id IN (
            SELECT id FROM mail_message WHERE date < NOW() - INTERVAL '2 years'
        );
        GET DIAGNOSTICS v_count = ROW_COUNT;
        RAISE NOTICE 'Eliminados % registros de mail_tracking_value', v_count;
    END IF;

    -- 7. Attachments huerfanos (res_model/res_id apuntan a nada)
    -- PRECAUCION: No eliminar attachments de modelos validos
    DELETE FROM ir_attachment
    WHERE res_model IS NOT NULL
    AND res_model NOT IN (SELECT model FROM ir_model)
    AND store_fname IS NULL;  -- Solo los que NO tienen archivo fisico
    GET DIAGNOSTICS v_count = ROW_COUNT;
    RAISE NOTICE 'Eliminados % attachments huerfanos (sin fichero)', v_count;

    RAISE NOTICE 'Limpieza completada. Ejecutar VACUUM FULL y ANALYZE manualmente.';
END $$;

-- Ejecutar DESPUES del bloque anterior:
-- VACUUM FULL;
-- ANALYZE;
```

**Limites del Upgrade Service:**

| Limite | Valor | Recomendacion |
|--------|-------|---------------|
| Tamano maximo dump | ~10 GB | Mantener < 5 GB |
| Tiempo maximo proceso | ~24 horas | Reducir BD si > 5 GB |
| Memoria servidor | Variable | Limpiar datos historicos |

---

## 14. Many2many Relation Tables

**Patron regex:**
```
relation "([^"]+)_([^"]+)_rel" does not exist
table "([^"]+)" has.*columns.*but.*supplied
```

**Versiones:** v16→v17+ (reestructuracion de algunas relaciones M2M)

**Causa:** Las tablas de relacion Many2many se crean automaticamente por Odoo con nombres como `model1_model2_rel`. Si un modelo se renombra, la tabla de relacion puede quedar inconsistente.

**Diagnostico pre-envio:**

```sql
-- Tablas de relacion M2M que podrian no sobrevivir al upgrade
SELECT table_name
FROM information_schema.tables
WHERE table_schema = 'public'
AND table_name LIKE '%_rel'
AND table_name NOT LIKE 'ir_%'
ORDER BY table_name;

-- Verificar que las tablas M2M tienen la estructura correcta (2-3 columnas)
SELECT table_name, count(*) as num_columns
FROM information_schema.columns
WHERE table_schema = 'public'
AND table_name LIKE '%_rel'
GROUP BY table_name
HAVING count(*) NOT BETWEEN 2 AND 3
ORDER BY table_name;
```

**Fix generico:**

```sql
-- =============================================================
-- Fix: Many2many relation table missing
-- Error: relation "X_Y_rel" does not exist
-- =============================================================

DO $$
BEGIN
    -- Si la tabla de relacion fue eliminada pero los datos existen en otra tabla
    -- Recrear la tabla con la estructura correcta
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.tables
        WHERE table_name = 'expected_rel_table_name'
    ) THEN
        CREATE TABLE expected_rel_table_name (
            column1_id integer NOT NULL,
            column2_id integer NOT NULL,
            UNIQUE (column1_id, column2_id)
        );
        RAISE NOTICE 'Fix: Tabla de relacion recreada: expected_rel_table_name';
    ELSE
        RAISE NOTICE 'Fix: Tabla de relacion ya existe';
    END IF;
END $$;
```

---

## 15. Filestore References

**Patron regex:**
```
FileNotFoundError.*filestore
store_fname.*not found
attachment.*missing file
```

**Versiones:** Todas (el filestore NO se incluye en el dump SQL)

**Causa:** Despues del upgrade, los registros `ir_attachment` con `store_fname` apuntan a ficheros del filestore que deben copiarse manualmente.

**Diagnostico post-upgrade:**

```sql
-- Attachments que requieren ficheros del filestore
SELECT count(*) AS total_con_fichero,
    pg_size_pretty(SUM(file_size)) AS tamano_estimado
FROM ir_attachment
WHERE store_fname IS NOT NULL;

-- Attachments sin fichero ni datos binarios (posiblemente corruptos)
SELECT id, name, res_model, res_id
FROM ir_attachment
WHERE store_fname IS NULL AND db_datas IS NULL
AND res_model IS NOT NULL
LIMIT 20;
```

**Solucion:** No es un fix SQL. El filestore debe copiarse manualmente:

```bash
# Copiar filestore de la instalacion v10 a la instalacion v18
# Origen tipico v10:
# /var/lib/odoo/.local/share/Odoo/filestore/<dbname>/

# Destino tipico v18 (Docker):
# docker cp filestore/ odoo:/var/lib/odoo/filestore/<dbname>/

# Verificar que los permisos son correctos:
# docker exec odoo chown -R odoo:odoo /var/lib/odoo/filestore/
```

---

## Referencia Rapida: Diagnostico por Error

Cuando recibas un error del Upgrade Service, usa esta tabla para encontrar la seccion relevante:

| Si el error contiene... | Ir a seccion | Prioridad |
|------------------------|--------------|-----------|
| `foreign key constraint` | [#1 FK Constraint](#1-fk-constraint-violation) | Alta |
| `does not exist` (column) | [#2 Missing Column](#2-missing-column--table) | Alta |
| `does not exist` (relation) | [#2 Missing Column](#2-missing-column--table) o [#6 Sequence](#6-sequence--serial-issues) | Alta |
| `invalid input syntax` | [#3 Data Type](#3-data-type-mismatch) | Alta |
| `depends on` / `not available` | [#4 Module Dependency](#4-module-dependency-error) | Alta |
| `duplicate key` / `already exists` | [#5 Unique Constraint](#5-unique-constraint-violation) o [#6 Sequence](#6-sequence--serial-issues) | Alta |
| `balance` / `descuadre` | [#7 Balance](#7-balance-contable-descuadrado) | Critica |
| `constraint` (CHECK/custom) | [#8 Custom Constraint](#8-custom-constraint-conflicts) | Alta |
| `NaN` / `null value` / `not-null` | [#9 Datos Corruptos](#9-datos-corruptos) | Alta |
| `wkf` / `workflow` | [#10 Workflow](#10-workflow--state-migration) | Media |
| `ParseError` / `XML` / `view` | [#11 XML/View](#11-xmlview-corruption) | Media |
| `ir_translation` | [#12 Translation](#12-translation-table-issues) | Media |
| `MemoryError` / `timeout` / `Killed` | [#13 Timeout](#13-timeout--memory) | Alta |
| `_rel` (relation table) | [#14 Many2many](#14-many2many-relation-tables) | Media |
| `filestore` / `store_fname` | [#15 Filestore](#15-filestore-references) | Baja (post-upgrade) |

---

## Script de Diagnostico Completo (Pre-Envio)

Ejecutar este script contra la BD ANTES de cada envio al Upgrade Service. Detecta problemas conocidos:

```sql
-- =============================================================
-- DIAGNOSTICO PRE-UPGRADE COMPLETO
-- Ejecutar contra la BD antes de enviar al Upgrade Service
-- =============================================================

DO $$
DECLARE
    v_count integer;
    v_balance numeric;
    v_size text;
BEGIN
    RAISE NOTICE '=== DIAGNOSTICO PRE-UPGRADE ===';
    RAISE NOTICE '';

    -- Tamano BD
    SELECT pg_size_pretty(pg_database_size(current_database())) INTO v_size;
    RAISE NOTICE '[INFO] Tamano BD: %', v_size;

    -- Version
    PERFORM value FROM ir_config_parameter WHERE key = 'database.version';
    RAISE NOTICE '[INFO] Version: %', (SELECT value FROM ir_config_parameter WHERE key = 'database.version');

    -- Modulos instalados
    SELECT count(*) INTO v_count FROM ir_module_module WHERE state = 'installed';
    RAISE NOTICE '[INFO] Modulos instalados: %', v_count;

    RAISE NOTICE '';
    RAISE NOTICE '--- Comprobaciones ---';

    -- 1. Balance contable
    SELECT COALESCE(SUM(debit - credit), 0) INTO v_balance FROM account_move_line;
    IF ABS(v_balance) > 0.01 THEN
        RAISE WARNING '[FAIL] Balance descuadrado: %', v_balance;
    ELSE
        RAISE NOTICE '[OK] Balance contable: %', v_balance;
    END IF;

    -- 2. NaN en campos contables
    SELECT count(*) INTO v_count FROM account_move_line
    WHERE debit = 'NaN'::numeric OR credit = 'NaN'::numeric;
    IF v_count > 0 THEN
        RAISE WARNING '[FAIL] % apuntes con NaN en debit/credit', v_count;
    ELSE
        RAISE NOTICE '[OK] Sin NaN en campos contables';
    END IF;

    -- 3. FK huerfanas en facturas
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'account_invoice') THEN
        SELECT count(*) INTO v_count FROM account_invoice
        WHERE journal_id NOT IN (SELECT id FROM account_journal) AND journal_id IS NOT NULL;
        IF v_count > 0 THEN
            RAISE WARNING '[FAIL] % facturas con journal_id huerfano', v_count;
        ELSE
            RAISE NOTICE '[OK] Sin FK huerfanas en facturas';
        END IF;
    END IF;

    -- 4. Numeros de factura duplicados
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'account_invoice') THEN
        SELECT count(*) INTO v_count FROM (
            SELECT number FROM account_invoice
            WHERE number IS NOT NULL AND state != 'cancel'
            GROUP BY number, journal_id HAVING count(*) > 1
        ) sub;
        IF v_count > 0 THEN
            RAISE WARNING '[FAIL] % numeros de factura duplicados', v_count;
        ELSE
            RAISE NOTICE '[OK] Sin facturas duplicadas';
        END IF;
    END IF;

    -- 5. Modulos con dependencias eliminadas
    SELECT count(*) INTO v_count FROM ir_module_module_dependency d
    JOIN ir_module_module m ON d.module_id = m.id
    WHERE m.state = 'installed'
    AND d.name IN ('procurement', 'project_issue', 'stock_inventory', 'account_voucher', 'pad');
    IF v_count > 0 THEN
        RAISE WARNING '[WARN] % modulos dependen de modulos eliminados', v_count;
    ELSE
        RAISE NOTICE '[OK] Sin dependencias a modulos eliminados';
    END IF;

    -- 6. Vistas huerfanas
    SELECT count(*) INTO v_count FROM ir_ui_view
    WHERE inherit_id IS NOT NULL
    AND NOT EXISTS (SELECT 1 FROM ir_ui_view p WHERE p.id = inherit_id);
    IF v_count > 0 THEN
        RAISE WARNING '[WARN] % vistas con parent huerfano', v_count;
    ELSE
        RAISE NOTICE '[OK] Sin vistas huerfanas';
    END IF;

    -- 7. XMLIDs duplicados
    SELECT count(*) INTO v_count FROM (
        SELECT module, name FROM ir_model_data
        WHERE module != '__export__'
        GROUP BY module, name HAVING count(*) > 1
    ) sub;
    IF v_count > 0 THEN
        RAISE WARNING '[WARN] % XMLIDs duplicados', v_count;
    ELSE
        RAISE NOTICE '[OK] Sin XMLIDs duplicados';
    END IF;

    -- 8. Logs voluminosos
    SELECT count(*) INTO v_count FROM ir_logging;
    IF v_count > 100000 THEN
        RAISE WARNING '[WARN] ir_logging tiene % registros (limpiar)', v_count;
    ELSE
        RAISE NOTICE '[OK] ir_logging: % registros', v_count;
    END IF;

    RAISE NOTICE '';
    RAISE NOTICE '=== FIN DIAGNOSTICO ===';
END $$;
```
