### Verificacion previa obligatoria (antes de cualquier DELETE SQL de account.move)

```sql
-- Verificar hash fiscal en asientos a borrar
SELECT id, name, inalterable_hash, secure_sequence_number, state, posted_before
FROM account_move
WHERE id IN (/* @move_ids */);

-- Verificar lock dates de la empresa
SELECT fiscalyear_lock_date, tax_lock_date, sale_lock_date, purchase_lock_date, hard_lock_date
FROM res_company
WHERE id = /* @company_id */;

-- Verificar reconciliaciones pendientes
SELECT aml.id, aml.move_id, apr.id as partial_id
FROM account_move_line aml
JOIN account_partial_reconcile apr
  ON apr.debit_move_id = aml.id OR apr.credit_move_id = aml.id
WHERE aml.move_id IN (/* @move_ids */);
```

