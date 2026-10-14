### stock.valuation.layer

**ADVERTENCIA:** Las capas de valoracion determinan el coste del inventario. Borrarlas afecta directamente
al valor contable del stock. Solo borrar si:
- Son de un test (vinculadas a moves de test)
- Se van a borrar tambien los account.move asociados
- Se va a recalcular el coste del producto despues

```sql
BEGIN;

-- @svl_ids = IDs de stock.valuation.layer a eliminar

-- 1. Romper auto-referencias (capas vinculadas entre si)
UPDATE stock_valuation_layer SET stock_valuation_layer_id = NULL
WHERE stock_valuation_layer_id IN (@svl_ids);

-- 2. Romper referencia desde account.move.line (si existe)
UPDATE stock_valuation_layer SET account_move_line_id = NULL
WHERE id IN (@svl_ids) AND account_move_line_id IS NOT NULL;

-- 3. Eliminar capas
DELETE FROM stock_valuation_layer WHERE id IN (@svl_ids);

-- 4. Verificar que no quedan capas vinculadas huerfanas
SELECT id, stock_valuation_layer_id FROM stock_valuation_layer
WHERE stock_valuation_layer_id IS NOT NULL
  AND stock_valuation_layer_id NOT IN (SELECT id FROM stock_valuation_layer);
-- Debe ser 0 filas

ROLLBACK;
```

