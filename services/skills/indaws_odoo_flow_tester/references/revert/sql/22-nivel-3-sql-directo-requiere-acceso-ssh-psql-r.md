### Recuperacion de secuencias (sequence recovery)

Despues de revertir registros de test, las secuencias pueden haber avanzado innecesariamente.
Solo resetear si los registros de test fueron los ULTIMOS en usar la secuencia.

```sql
BEGIN;

-- 1. Verificar que el ultimo uso de la secuencia fue del test
-- Para secuencias con date_range (facturas):
SELECT isdr.id, isdr.sequence_id, isdr.number_next, isdr.date_from, isdr.date_to
FROM ir_sequence_date_range isdr
JOIN ir_sequence iseq ON isdr.sequence_id = iseq.id
WHERE iseq.code = 'account.move'  -- o el codigo relevante
ORDER BY isdr.date_from DESC;

-- 2. Verificar que NO hay registros reales posteriores al test
-- Ejemplo para sale.order:
SELECT MAX(id) AS last_real_id, name
FROM sale_order
WHERE name NOT LIKE '[TEST-%'
  AND client_order_ref NOT LIKE '[TEST-%'
ORDER BY id DESC LIMIT 1;

-- 3. Solo si el test fue el ultimo uso, resetear:
-- (ver seccion ir_sequence mas abajo)

ROLLBACK;
```

**ADVERTENCIA:** NUNCA resetear secuencias de diarios con `restrict_mode_hash_table=True`.
El numero de secuencia forma parte del hash encadenado. Retroceder causaria:
- Numero duplicado → fallo en UNIQUE constraint
- Hash no coincidente con el hash anterior → cadena rota

