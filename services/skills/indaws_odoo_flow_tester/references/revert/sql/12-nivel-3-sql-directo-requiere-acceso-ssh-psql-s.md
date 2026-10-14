### stock.move.line y stock.quant (limpieza con recalculo)

**ADVERTENCIA CRITICA sobre stock.quant:**
- En el ORM, `stock.move.line` actualiza `stock.quant` automaticamente al crearse/borrarse via
  `_update_available_quantity()`.
- En SQL directo, `stock.quant` **NO se actualiza**. Borrar `stock_move_line` directamente deja
  quants inconsistentes (cantidades fantasma o negativas).
- Despues de borrar move lines via SQL, se DEBE recalcular quants.

```sql
BEGIN;

-- @picking_ids = IDs de stock.picking cuyos move lines se quieren limpiar
-- @move_ids = IDs de stock.move asociados

-- 1. Guardar productos afectados ANTES de borrar (para recalculo posterior)
CREATE TEMP TABLE _affected_products AS
SELECT DISTINCT product_id, location_id, location_dest_id
FROM stock_move_line
WHERE picking_id IN (@picking_ids);

-- 2. Borrar valuation layers primero
DELETE FROM stock_valuation_layer
WHERE stock_move_id IN (@move_ids);

-- 3. Borrar move lines
DELETE FROM stock_move_line WHERE picking_id IN (@picking_ids);

-- 4. Romper auto-referencias de stock.move (M2M move_orig/move_dest)
DELETE FROM stock_move_move_rel
WHERE move_orig_id IN (@move_ids) OR move_dest_id IN (@move_ids);

-- 5. Borrar moves
DELETE FROM stock_move WHERE id IN (@move_ids);

-- 6. RECALCULAR stock.quant para productos afectados
-- OPCION A: Via SQL (recalculo manual)
-- Para cada producto/ubicacion afectada, recalcular quantity basandose en moves done:
-- NOTA: Este recalculo es aproximado. Es preferible usar Opcion B.

-- OPCION B: Via ORM despues del COMMIT (RECOMENDADO)
-- Ejecutar desde odoo-pilot:
-- execute_method.sh product.product _compute_quantities '[[<product_ids>]]' '{}'
-- O forzar un inventory adjustment:
-- Para cada producto afectado:
-- search_records.sh stock.quant '[["product_id","=",<pid>],["location_id.usage","=","internal"]]' '["id","quantity"]' 0

DROP TABLE IF EXISTS _affected_products;

ROLLBACK;
```

**Post-SQL quant fix (ejecutar via ORM despues de COMMIT):**
```bash
# Para cada producto afectado, forzar recalculo de cantidades
# Opcion 1: Recompute via product
execute_method.sh product.product _compute_quantities '[[<product_ids>]]' '{}'

# Opcion 2: Si quants estan desincronizados, usar inventory adjustment
# Ir a Inventario → Operaciones → Ajuste de inventario fisico
# O via API:
search_records.sh stock.quant '[["product_id","in",[<product_ids>]],["location_id.usage","=","internal"]]' '["id","product_id","location_id","quantity"]' 0
# Comparar con cantidades esperadas y ajustar manualmente si es necesario
```

