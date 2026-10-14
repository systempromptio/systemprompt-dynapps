### account.partial.reconcile y account.full.reconcile (limpieza SQL)

```sql
BEGIN;

-- @move_ids = IDs de account.move cuyas reconciliaciones se quieren limpiar

-- 1. Identificar partials afectados
CREATE TEMP TABLE _partials_to_delete AS
SELECT apr.id, apr.full_reconcile_id, apr.exchange_move_id
FROM account_partial_reconcile apr
JOIN account_move_line aml ON apr.debit_move_id = aml.id OR apr.credit_move_id = aml.id
WHERE aml.move_id IN (@move_ids);

-- 2. Identificar exchange moves que se deben limpiar
CREATE TEMP TABLE _exchange_moves AS
SELECT DISTINCT exchange_move_id AS id FROM _partials_to_delete
WHERE exchange_move_id IS NOT NULL
UNION
SELECT DISTINCT exchange_move_id FROM account_full_reconcile
WHERE id IN (SELECT DISTINCT full_reconcile_id FROM _partials_to_delete WHERE full_reconcile_id IS NOT NULL)
  AND exchange_move_id IS NOT NULL;

-- 3. Borrar partials
DELETE FROM account_partial_reconcile WHERE id IN (SELECT id FROM _partials_to_delete);

-- 4. Borrar fulls huerfanos
DELETE FROM account_full_reconcile
WHERE id NOT IN (
    SELECT DISTINCT full_reconcile_id
    FROM account_partial_reconcile
    WHERE full_reconcile_id IS NOT NULL
);

-- 5. Limpiar exchange moves (si no estan referenciados por otros partials)
-- PRECAUCION: Verificar que no son referenciados por partials NO borrados
DELETE FROM account_move_line WHERE move_id IN (
    SELECT id FROM _exchange_moves
    WHERE id NOT IN (
        SELECT DISTINCT exchange_move_id FROM account_partial_reconcile WHERE exchange_move_id IS NOT NULL
        UNION
        SELECT DISTINCT exchange_move_id FROM account_full_reconcile WHERE exchange_move_id IS NOT NULL
    )
);
DELETE FROM account_move WHERE id IN (
    SELECT id FROM _exchange_moves
    WHERE id NOT IN (
        SELECT DISTINCT exchange_move_id FROM account_partial_reconcile WHERE exchange_move_id IS NOT NULL
        UNION
        SELECT DISTINCT exchange_move_id FROM account_full_reconcile WHERE exchange_move_id IS NOT NULL
    )
);

-- 6. Resetear campos de reconciliacion en move lines
UPDATE account_move_line
SET full_reconcile_id = NULL, matching_number = NULL
WHERE move_id IN (@move_ids);

DROP TABLE IF EXISTS _partials_to_delete, _exchange_moves;

ROLLBACK;
```

