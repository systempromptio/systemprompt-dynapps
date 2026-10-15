## Recuperacion de Secuencias (Enhanced)

Logica detallada para recuperar secuencias consumidas por registros de test.

### Paso 1: Identificar secuencias afectadas

```sql
-- Secuencias principales con sus valores actuales
SELECT s.id, s.code, s.name, s.number_next, s.implementation,
       s.prefix, s.suffix, s.company_id
FROM ir_sequence s
WHERE s.code IN ('sale.order', 'purchase.order', 'account.move',
                 'stock.picking', 'mrp.production', 'pos.order')
ORDER BY s.code, s.company_id;

-- Secuencias con rangos de fecha (facturas, nominas)
SELECT s.id AS seq_id, s.code, s.name,
       dr.id AS range_id, dr.date_from, dr.date_to,
       dr.number_next, dr.number_next_actual
FROM ir_sequence s
JOIN ir_sequence_date_range dr ON dr.sequence_id = s.id
WHERE s.code IN ('account.move')
ORDER BY s.code, dr.date_from DESC;
```

### Paso 2: Verificar que los ultimos registros son de test

```sql
-- Para sale.order: verificar si los ultimos N registros son de test
WITH ranked AS (
    SELECT id, name, client_order_ref,
           ROW_NUMBER() OVER (ORDER BY id DESC) AS rn,
           CASE WHEN name LIKE '[TEST-%' OR client_order_ref LIKE '[TEST-%'
                THEN 1 ELSE 0 END AS is_test
    FROM sale_order
)
SELECT id, name, is_test, rn
FROM ranked
WHERE rn <= 20  -- Revisar los ultimos 20 registros
ORDER BY rn;
-- Si los primeros N registros (rn=1..N) son todos is_test=1 y luego hay is_test=0,
-- entonces se puede decrementar la secuencia en N posiciones.

-- Para account.move: misma logica pero con prefijo en name/ref
WITH ranked AS (
    SELECT id, name, ref, sequence_prefix, sequence_number,
           ROW_NUMBER() OVER (PARTITION BY sequence_prefix ORDER BY sequence_number DESC) AS rn,
           CASE WHEN name LIKE '[TEST-%' OR ref LIKE '[TEST-%'
                THEN 1 ELSE 0 END AS is_test
    FROM account_move
)
SELECT sequence_prefix, id, name, sequence_number, is_test, rn
FROM ranked
WHERE rn <= 20
ORDER BY sequence_prefix, rn;
```

### Paso 3: Decrementar secuencia de forma segura

```sql
BEGIN;

-- Solo si se confirmo que los ultimos N registros son de test:
-- @seq_id = ID de la secuencia
-- @count = numero de registros de test que fueron los ultimos
-- @new_next = number_next_actual - @count

-- Para secuencias con implementation = 'standard':
-- Actualizar campo ORM
UPDATE ir_sequence
SET number_next = number_next - @count
WHERE id = @seq_id AND number_next > @count;

-- Actualizar secuencia PostgreSQL subyacente
-- SELECT setval('ir_sequence_' || @seq_id, (number_next - @count) - 1, true);

-- Para secuencias con implementation = 'no_gap':
-- Solo actualizar campo ORM (no hay secuencia PostgreSQL)
UPDATE ir_sequence
SET number_next = number_next - @count
WHERE id = @seq_id AND implementation = 'no_gap' AND number_next > @count;

-- Para secuencias con date_range:
UPDATE ir_sequence_date_range
SET number_next = number_next - @count
WHERE id = @range_id AND number_next > @count;

ROLLBACK; -- Cambiar a COMMIT solo tras verificacion
```

### Paso 4: Validar que no hay gaps

```sql
-- Verificar que la secuencia no genera duplicados
-- Buscar el proximo numero que generaria y comprobar que no existe
SELECT s.id, s.code, s.number_next,
       s.prefix || LPAD(s.number_next::text, s.padding, '0') || COALESCE(s.suffix, '') AS next_name
FROM ir_sequence s
WHERE s.id = @seq_id;

-- Verificar que ese nombre no existe ya en el modelo
-- Ejemplo para sale.order:
SELECT COUNT(*) FROM sale_order WHERE name = @next_name;
-- Debe ser 0

-- ADVERTENCIA: NUNCA recuperar secuencias de diarios con hash fiscal
SELECT aj.id, aj.code, aj.restrict_mode_hash_table
FROM account_journal aj
JOIN ir_sequence iseq ON iseq.code LIKE '%' || aj.code || '%'
WHERE iseq.id = @seq_id AND aj.restrict_mode_hash_table = true;
-- Si devuelve filas → NO RECUPERAR la secuencia
```

---

