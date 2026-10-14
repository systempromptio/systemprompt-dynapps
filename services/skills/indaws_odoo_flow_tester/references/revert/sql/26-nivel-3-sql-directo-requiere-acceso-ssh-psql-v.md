### Verificacion Post-Cleanup SQL

```sql
-- 1. Verificar saldos contables (balance general debe ser 0)
SELECT SUM(debit) - SUM(credit) AS balance_general
FROM account_move_line
WHERE parent_state = 'posted';

-- 2. Verificar saldos por cuenta
SELECT
    aa.code, aa.name,
    SUM(aml.debit) AS total_debe,
    SUM(aml.credit) AS total_haber,
    SUM(aml.balance) AS saldo
FROM account_move_line aml
JOIN account_account aa ON aml.account_id = aa.id
WHERE aml.parent_state = 'posted'
GROUP BY aa.code, aa.name
HAVING ABS(SUM(aml.balance)) > 0.01
ORDER BY aa.code;

-- 3. Verificar cantidades de stock
SELECT
    pp.id AS product_id, pt.name AS producto,
    sl.complete_name AS ubicacion, sq.quantity, sq.reserved_quantity
FROM stock_quant sq
JOIN product_product pp ON sq.product_id = pp.id
JOIN product_template pt ON pp.product_tmpl_id = pt.id
JOIN stock_location sl ON sq.location_id = sl.id
WHERE sq.quantity != 0 AND sl.usage = 'internal'
ORDER BY pt.name, sl.complete_name;

-- 4. Verificar reconciliaciones huerfanas
SELECT 'partial_sin_move_line' AS tipo, COUNT(*)
FROM account_partial_reconcile apr
WHERE NOT EXISTS (SELECT 1 FROM account_move_line WHERE id = apr.debit_move_id)
   OR NOT EXISTS (SELECT 1 FROM account_move_line WHERE id = apr.credit_move_id)
UNION ALL
SELECT 'full_sin_partial', COUNT(*)
FROM account_full_reconcile afr
WHERE NOT EXISTS (SELECT 1 FROM account_partial_reconcile WHERE full_reconcile_id = afr.id);
-- Ambos deben ser 0

-- 5. Verificar valuation layers huerfanas
SELECT 'svl_sin_stock_move' AS tipo, COUNT(*)
FROM stock_valuation_layer svl
WHERE svl.stock_move_id IS NOT NULL
  AND NOT EXISTS (SELECT 1 FROM stock_move WHERE id = svl.stock_move_id)
UNION ALL
SELECT 'svl_sin_account_move', COUNT(*)
FROM stock_valuation_layer svl
WHERE svl.account_move_id IS NOT NULL
  AND NOT EXISTS (SELECT 1 FROM account_move WHERE id = svl.account_move_id)
UNION ALL
SELECT 'svl_autoref_huerfana', COUNT(*)
FROM stock_valuation_layer svl
WHERE svl.stock_valuation_layer_id IS NOT NULL
  AND NOT EXISTS (SELECT 1 FROM stock_valuation_layer WHERE id = svl.stock_valuation_layer_id);
-- Todos deben ser 0

-- 6. Verificar integridad de hash (solo si se borraron moves hasheados)
-- Listar diarios con hash activo y verificar gaps en secuencia
SELECT aj.code AS journal, am.sequence_prefix, am.sequence_number,
       am.inalterable_hash IS NOT NULL AS has_hash
FROM account_move am
JOIN account_journal aj ON am.journal_id = aj.id
WHERE aj.restrict_mode_hash_table = true
ORDER BY aj.code, am.sequence_prefix, am.sequence_number;
-- Verificar que no hay gaps en sequence_number para moves hasheados
```

---

