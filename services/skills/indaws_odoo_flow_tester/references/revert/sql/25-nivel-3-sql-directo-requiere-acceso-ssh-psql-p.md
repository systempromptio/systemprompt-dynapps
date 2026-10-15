### Post-SQL: Recalculo de stock.quant

**OBLIGATORIO** despues de cualquier limpieza SQL que haya borrado stock.move.line:

```bash
# Via ORM, recalcular cantidades de los productos afectados
# Opcion 1: Forzar recompute
execute_method.sh product.product _compute_quantities '[[<product_ids>]]' '{}'

# Opcion 2: Verificar quants y corregir manualmente
search_records.sh stock.quant '[["product_id","in",[<product_ids>]],["location_id.usage","=","internal"]]' '["id","product_id","location_id","quantity","reserved_quantity"]' 0
# Comparar con cantidades esperadas del snapshot

# Opcion 3: Inventario fisico via Odoo (mas seguro)
# Ir a Inventario → Operaciones → Ajuste de inventario
# O crear adjustment via API si las diferencias son claras
```

---

