### Limpieza SQL de hr.expense + hr.expense.sheet (verificado)

```sql
BEGIN;
-- Variables: @sheet_ids = IDs de hr_expense_sheet a borrar

-- Paso 1: Borrar account.move asociados PRIMERO (ver seccion account.move)

-- Paso 2: Desvincular expenses del sheet (sheet_id es set null, no cascade — L100-107)
UPDATE hr_expense SET sheet_id = NULL WHERE sheet_id IN (@sheet_ids);

-- Paso 3: Limpiar auxiliares
DELETE FROM mail_message WHERE model = 'hr.expense.sheet' AND res_id IN (@sheet_ids);
DELETE FROM mail_followers WHERE res_model = 'hr.expense.sheet' AND res_id IN (@sheet_ids);
DELETE FROM ir_attachment WHERE res_model = 'hr.expense.sheet' AND res_id IN (@sheet_ids);

-- Paso 4: Borrar expense sheets
DELETE FROM hr_expense_sheet WHERE id IN (@sheet_ids);

-- Paso 5: Borrar expenses individuales (si se desea)
-- DELETE FROM hr_expense WHERE id IN (@expense_ids);

ROLLBACK;
```

