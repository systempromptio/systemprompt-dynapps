### account.move con hash fiscal

**CONTEXTO (verificado en account_move.py L325, L4046-4058):** En Odoo 18.0, un move con `inalterable_hash`
no puede pasar a draft via ORM (L1791 bloquea button_draft). Sin embargo, **unlink() NO verifica inalterable_hash**
— las restricciones de unlink son `_unlink_forbid_parts_of_chain` (L3488-3506) y
`_unlink_account_audit_trail_except_once_post` (L3508-3517), ambas bypasseables con `force_delete=True`.
El hash se calcula como SHA-256 encadenado: `hash(previous_hash + current_record_data)`.
Los campos protegidos son: `name`, `date`, `journal_id`, `company_id` + campos de `_get_integrity_hash_fields()` (L4007-4012).
Borrar un move via SQL rompe la cadena — solo hacerlo si se acepta perder la validacion de integridad para ese diario.

```sql
BEGIN;

-- @move_ids = lista de IDs de account.move a eliminar

-- 1. Eliminar reconciliaciones parciales
-- CRITICO: Debe hacerse ANTES de borrar move lines
DELETE FROM account_partial_reconcile
WHERE debit_move_id IN (SELECT id FROM account_move_line WHERE move_id IN (@move_ids))
   OR credit_move_id IN (SELECT id FROM account_move_line WHERE move_id IN (@move_ids));

-- 2. Eliminar reconciliaciones completas huerfanas
-- (Las que ya no tienen ningun partial asociado)
DELETE FROM account_full_reconcile
WHERE id NOT IN (
    SELECT DISTINCT full_reconcile_id
    FROM account_partial_reconcile
    WHERE full_reconcile_id IS NOT NULL
);

-- 3. Eliminar capas de valoracion de inventario vinculadas
DELETE FROM stock_valuation_layer WHERE account_move_id IN (@move_ids);

-- 4. Eliminar pagos vinculados al move
DELETE FROM account_payment WHERE move_id IN (@move_ids);

-- 5. Limpiar campos que referencian el move desde otros modelos
UPDATE hr_expense_sheet SET account_move_id = NULL WHERE account_move_id IN (@move_ids);
UPDATE hr_payslip SET move_id = NULL WHERE move_id IN (@move_ids);
UPDATE pos_order SET account_move = NULL WHERE account_move IN (@move_ids);
UPDATE account_asset SET disposal_move_id = NULL WHERE disposal_move_id IN (@move_ids);

-- 6. Limpiar tablas auxiliares
DELETE FROM mail_message WHERE model = 'account.move' AND res_id IN (@move_ids);
DELETE FROM mail_followers WHERE res_model = 'account.move' AND res_id IN (@move_ids);
DELETE FROM mail_activity WHERE res_model = 'account.move' AND res_id IN (@move_ids);
DELETE FROM ir_attachment WHERE res_model = 'account.move' AND res_id IN (@move_ids);

-- 7. Eliminar lineas del asiento (DESPUES de reconciliaciones y valuation layers)
DELETE FROM account_move_line WHERE move_id IN (@move_ids);

-- 8. Eliminar el asiento/factura
DELETE FROM account_move WHERE id IN (@move_ids);

-- Verificar antes de COMMIT
SELECT COUNT(*) FROM account_move WHERE id IN (@move_ids);
-- Debe ser 0

ROLLBACK; -- Cambiar a COMMIT solo tras verificacion
```

