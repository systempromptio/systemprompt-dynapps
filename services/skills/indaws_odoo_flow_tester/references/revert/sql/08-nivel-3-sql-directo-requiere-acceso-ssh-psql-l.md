### Limpieza de hash fiscal (repair de cadena)

**ADVERTENCIA EXTREMA: Solo usar si se acepta perder la validacion de integridad del diario.**
Esto es necesario tras borrar un move hasheado para que la cadena no quede rota.

```sql
BEGIN;

-- Opcion A: Recalcular hash de toda la cadena posterior al borrado
-- NO ES POSIBLE via SQL (requiere el algoritmo Python de Odoo)
-- Usar: execute_method.sh account.move button_hash '[[<ids_posteriores>]]'

-- Opcion B: Limpiar hash de todos los moves del diario afectado (DRASTICO)
-- Esto desactiva la proteccion de integridad para todo el diario
-- Solo viable si es un diario de test
UPDATE account_move
SET inalterable_hash = NULL, secure_sequence_number = 0
WHERE journal_id = @journal_id
  AND inalterable_hash IS NOT NULL;

-- Opcion C: Desactivar hash en el diario (solo si NO hay moves hasheados restantes)
-- Verificar primero:
SELECT COUNT(*) FROM account_move
WHERE journal_id = @journal_id AND inalterable_hash IS NOT NULL;
-- Si es 0, se puede desactivar via ORM:
-- update_record.sh account.journal '[<journal_id>]' '{"restrict_mode_hash_table": false}'
-- NOTA: En Odoo 18.0, write() de account.journal verifica que no haya moves hasheados
-- antes de permitir desactivar. Si hay moves hasheados, lanza UserError.

ROLLBACK;
```

