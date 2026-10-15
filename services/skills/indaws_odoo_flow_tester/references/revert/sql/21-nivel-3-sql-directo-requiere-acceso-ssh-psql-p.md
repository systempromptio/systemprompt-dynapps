### Pre-verificacion de inalterable_hash antes de DELETE SQL

**OBLIGATORIO** antes de intentar cualquier DELETE SQL de `account.move`:

```sql
-- Verificar si los moves tienen hash de integridad
SELECT id, name, journal_id, inalterable_hash, secure_sequence_number,
       posted_before, state
FROM account_move
WHERE id IN (@move_ids)
  AND inalterable_hash IS NOT NULL;

-- Si devuelve filas → DETENERSE y evaluar:
-- 1. Si es diario de test: se puede proceder (aceptando romper cadena)
-- 2. Si es diario de produccion: NUNCA borrar, usar _reverse_moves() via ORM
-- 3. Si se debe proceder: documentar el gap de hash y recalcular despues

-- Verificar tambien el audit trail de la empresa
SELECT id, name, check_account_audit_trail
FROM res_company
WHERE id IN (SELECT company_id FROM account_move WHERE id IN (@move_ids));
-- Si check_account_audit_trail = true → el borrado deja hueco en audit trail
```

