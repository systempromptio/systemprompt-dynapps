### Limpieza SQL de mrp.production (verificado)

```sql
BEGIN;
-- Variables: @mo_ids = IDs de mrp_production a borrar

-- Paso 1: Borrar productividades de workcenter
DELETE FROM mrp_workcenter_productivity
WHERE workorder_id IN (SELECT id FROM mrp_workorder WHERE production_id IN (@mo_ids));

-- Paso 2: Borrar workorders (production_id es FK restrict — borrar ANTES de production)
DELETE FROM mrp_workorder WHERE production_id IN (@mo_ids);

-- Paso 3: Borrar unbuilds asociados
DELETE FROM mrp_unbuild WHERE mo_id IN (@mo_ids);

-- Paso 4: Obtener moves asociados y limpiar
-- @move_ids = SELECT id FROM stock_move WHERE raw_material_production_id IN (@mo_ids) OR production_id IN (@mo_ids);
DELETE FROM stock_valuation_layer WHERE stock_move_id IN (@move_ids);
DELETE FROM stock_move_line WHERE move_id IN (@move_ids);
DELETE FROM stock_move_move_rel WHERE move_orig_id IN (@move_ids) OR move_dest_id IN (@move_ids);
DELETE FROM stock_move WHERE id IN (@move_ids);

-- Paso 5: Limpiar auxiliares
DELETE FROM mail_message WHERE model = 'mrp.production' AND res_id IN (@mo_ids);
DELETE FROM mail_followers WHERE res_model = 'mrp.production' AND res_id IN (@mo_ids);
DELETE FROM ir_attachment WHERE res_model = 'mrp.production' AND res_id IN (@mo_ids);

-- Paso 6: Borrar produccion
DELETE FROM mrp_production WHERE id IN (@mo_ids);

ROLLBACK;
```

