## CADENA 37: Desguace y Ajuste de Inventario
**ID:** CHAIN_SCRAP_AND_ADJUST
**Apps:** Inventario
**Descripcion:** Crear stock de un producto, ejecutar un desguace (scrap), verificar la disminucion de quants, y revertir mediante ajuste de inventario (ya que scrap en done es irreversible via ORM). Incluye alternativa SQL para entornos de test.
**Tiempo estimado:** 1-2 minutos

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| stock.scrap.action_validate | stock.scrap | stock.move | Campo move_ids en el scrap |
| stock.scrap.action_validate | stock.scrap | stock.move.line | `search_records.sh stock.move.line '[["move_id","in",<move_ids>]]'` |
| stock.quant.action_apply_inventory | stock.quant | stock.move | Ajuste de inventario |

### Paso de Variables Entre Flujos
- `{chain.product_id}` — ID del producto (storable)
- `{chain.location_id}` — ID de la ubicacion interna
- `{chain.scrap_location_id}` — ID de la ubicacion de desguace
- `{chain.quant_id}` — ID del quant a ajustar
- `{chain.scrap_id}` — ID del stock.scrap
- `{chain.initial_qty}` — Cantidad inicial en stock antes del scrap
- `{chain.scrap_qty}` — Cantidad desguazada

### Precondiciones
- Producto storable (type == 'consu') con stock disponible
- Ubicacion interna (stock.location usage='internal')
- Ubicacion de desguace (stock.location scrap_location=True)

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | — verificar stock — | Leer quant: `search_records.sh stock.quant '[["product_id","=",<product_id>],["location_id","=",<location_id>]]' '["quantity"]'` | quantity >= scrap_qty |
| 2 | SCRAP_CREATE | `create_record.sh stock.scrap '{"product_id": <id>, "scrap_qty": <qty>, "location_id": <loc_id>}'` | state='draft' |
| 3 | SCRAP_VALIDATE | `execute_method.sh stock.scrap action_validate '[[<scrap_id>]]' '{}'` | state='done', date_done != False |
| 4 | — verificar quant — | Leer quant: verificar que quantity disminuyo en scrap_qty | quantity == initial_qty - scrap_qty |
| 5 | INVENTORY_ADJUST | Revertir via ajuste de inventario: `update_record.sh stock.quant '[<quant_id>]' '{"inventory_quantity": <initial_qty>}'` + `execute_method.sh stock.quant action_apply_inventory '[[<quant_id>]]' '{}'` | quantity == initial_qty |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 3 | **PUNTO DE NO RETORNO** | Scrap en estado done — NO se puede cancelar ni eliminar via ORM | Revertir stock con ajuste de inventario (paso 5) |
| 2 | (scrap ya done) | | |
| 1 | (solo lectura) | | |

**Alternativa SQL para test (NUNCA en produccion):**
```sql
-- 1. Obtener move_id del scrap
SELECT m.id FROM stock_move m WHERE m.scrap_id = <scrap_id>;
-- 2. Eliminar move lines
DELETE FROM stock_move_line WHERE move_id = <move_id>;
-- 3. Eliminar move
DELETE FROM stock_move WHERE id = <move_id>;
-- 4. Ajustar quants (sumar de vuelta)
UPDATE stock_quant SET quantity = quantity + <scrap_qty>
  WHERE product_id = <product_id> AND location_id = <location_id>;
UPDATE stock_quant SET quantity = quantity - <scrap_qty>
  WHERE product_id = <product_id> AND location_id = <scrap_location_id>;
-- 5. Resetear y eliminar scrap
UPDATE stock_scrap SET state = 'draft', date_done = NULL WHERE id = <scrap_id>;
DELETE FROM stock_scrap WHERE id = <scrap_id>;
```

### Puntos de No Retorno
- **Paso 3 (scrap done)**: Irreversible via ORM. Solo reversible mediante ajuste de inventario (ORM) o SQL (solo test)
- **Nota**: El ajuste de inventario (paso 5) restaura las cantidades pero NO deshace el stock.move del scrap — queda en el historial

---

