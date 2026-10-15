### Secuencias ir_sequence

```sql
BEGIN;

-- Resetear secuencia principal (campo ORM)
UPDATE ir_sequence
SET number_next = @new_next
WHERE id = @seq_id;

-- Resetear rango de fechas especifico
UPDATE ir_sequence_date_range
SET number_next = @new_next
WHERE id = @range_id;

-- Para secuencias con implementation = 'standard' (PostgreSQL sequence object):
-- Ademas del campo ORM, hay que resetear la secuencia subyacente de PostgreSQL
-- El nombre de la secuencia es 'ir_sequence_NNN' donde NNN es el id
-- IMPORTANTE: setval() toma el ULTIMO valor usado (no el siguiente)
-- SELECT setval('ir_sequence_NNN', @new_next - 1, true);

-- Para secuencias con implementation = 'no_gap':
-- Solo necesitan el UPDATE del campo ORM (no usan PostgreSQL sequences)

-- ADVERTENCIA: NUNCA resetear secuencias de diarios con hash activo
-- Verificar primero:
SELECT aj.id, aj.code, aj.restrict_mode_hash_table
FROM account_journal aj
JOIN ir_sequence iseq ON iseq.code LIKE '%' || aj.code || '%'
WHERE iseq.id = @seq_id AND aj.restrict_mode_hash_table = true;
-- Si devuelve filas → NO RESETEAR

ROLLBACK;
```

---

