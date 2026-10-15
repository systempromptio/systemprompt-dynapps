## 4. Restauracion Post-Test

### 4.1 Estrategia A — ORM: `write` sobre `number_next_actual` (Recomendada)

La forma mas segura y estandar. Usa la API ORM de Odoo.

```bash
# Restaurar secuencia estandar
update_record.sh ir.sequence 15 '{"number_next_actual": 42}'

# Restaurar secuencia por rango de fechas
update_record.sh ir.sequence.date_range 8 '{"number_next_actual": 156}'
```

**Como funciona internamente:**

Cuando se escribe `number_next_actual`, Odoo ejecuta:
- Para `implementation='standard'`: `ALTER SEQUENCE ir_sequence_XXX RESTART WITH Y`
- Para `implementation='no_gap'`: `UPDATE ir_sequence SET number_next = Y WHERE id = X`

El campo computado `number_next_actual` se encarga de abstraer la diferencia.

**Ventajas:**
- Respeta el mecanismo interno de Odoo
- Funciona con ambas implementaciones (`standard` y `no_gap`)
- Auditable via log de XML-RPC

**Limitaciones:**
- Requiere permisos de administrador
- No restaura la secuencia interna de `account.move` (ver Seccion 2.2)

### 4.2 Estrategia B — SQL directo (Tier 3, ultimo recurso)

Cuando el ORM falla o no hay acceso API. Requiere acceso SSH + psql.

```sql
-- Para secuencias con implementation='no_gap'
UPDATE ir_sequence
SET number_next = 42
WHERE id = 15;

-- Para secuencias con implementation='standard' (secuencia PostgreSQL)
-- Primero identificar el nombre de la secuencia PostgreSQL
SELECT id, number_next, pg_class.relname as pg_sequence_name
FROM ir_sequence
LEFT JOIN pg_class ON pg_class.relname = 'ir_sequence_' || lpad(ir_sequence.id::text, 3, '0')
WHERE ir_sequence.id = 15;

-- Luego resetear la secuencia PostgreSQL
ALTER SEQUENCE ir_sequence_015 RESTART WITH 42;

-- Tambien actualizar la tabla ir_sequence para consistencia
UPDATE ir_sequence SET number_next = 42 WHERE id = 15;
```

**Para secuencias con rango de fechas:**

```sql
-- Restaurar el valor del rango
UPDATE ir_sequence_date_range
SET number_next = 156
WHERE id = 8;

-- Si usa implementation='standard', tambien hay secuencia PG asociada
-- El nombre sigue el patron: ir_sequence_date_range_XXX_YYY
-- donde XXX = zero-padded sequence_id, YYY = zero-padded date_range_id
```

**Advertencia:** La manipulacion SQL directa:
- No pasa por la capa ORM → no ejecuta computes ni triggers Python
- Puede dejar inconsistencias si no se actualizan ambas tablas
- Requiere `COMMIT` explicito y verificacion posterior

### 4.3 Estrategia C — Manejo de `ir_sequence_date_range`

Para secuencias basadas en fechas (comun en facturacion), cada rango de fechas
tiene su propio contador independiente.

```bash
# 1. Identificar el rango activo para la fecha actual
search_records.sh ir.sequence.date_range \
  '[["sequence_id","=",12],["date_from","<=","2026-03-10"],["date_to",">=","2026-03-10"]]' \
  '["id","number_next_actual","date_from","date_to"]' \
  1

# 2. Restaurar el valor del rango activo
update_record.sh ir.sequence.date_range 8 '{"number_next_actual": 156}'
```

**Consideracion importante:** Si el test se ejecuta cerca de un cambio de periodo
(ej: fin de ano), puede haber creado registros en dos rangos distintos. Verificar
ambos rangos y restaurar los que hayan cambiado.

---

