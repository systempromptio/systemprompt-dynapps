## 3. INVENTARIO (stock.picking)

### PICKING_VALIDATE — Validar Albarán
**Método:** `execute_method.sh stock.picking button_validate '[[<id>]]' '{}'`
**Precondición:** state in ('draft', 'waiting', 'confirmed', 'assigned')
**Restricciones:**
- Si state == 'draft', confirma automáticamente primero
- Si cantidades a 0, las rellena con la demanda
- Puede devolver wizard de backorder si entrega parcial
- Para forzar sin backorder usar contexto: `{"skip_backorder": true, "picking_ids_not_to_backorder": [<id>]}`
**Resultado:** state → `done`, establece date_done
**⚠️ PUNTO DE NO RETORNO: Una vez 'done', NO se puede cancelar**

### PICKING_CANCEL — Cancelar Albarán
**Método:** `execute_method.sh stock.picking action_cancel '[[<id>]]' '{}'`
**Restricciones:**
- NO puede cancelar moves en estado 'done' (excepto si scrapped=True) → UserError
- Internamente llama `move_ids._action_cancel()` y luego establece `is_locked = True`
- Los moves sin estado 'done'/'cancel' se desreservan y cancelan
**Estados permitidos:** draft, waiting, confirmed, assigned
**Resultado:** state → `cancel`, is_locked → True

### PICKING_DELETE — Eliminar Albarán
**Método:** `delete_record.sh stock.picking '[<id>]'`
**Restricciones:**
- Internamente cancela moves (`move_ids._action_cancel()`) y luego los elimina
- NO puede eliminar si algún move está en estado `done` (excepto si `scrapped=True`), porque `_action_cancel()` falla con UserError
- **Nota:** La restricción sobre `move_orig_ids`/`move_dest_ids` NO existe en el `unlink()` de stock.move en Odoo 18.0. La única restricción real es que no se pueden cancelar moves en estado `done`.

### PICKING_VALIDATE_SEQUENCE — Secuencia Completa de Validación (1-step)
**Descripción:** Secuencia correcta para validar un picking en almacén de 1 paso.
**Pasos vía API:**
1. **Confirmar:** `execute_method.sh stock.picking action_confirm '[[<id>]]' '{}'` → state: draft → confirmed/assigned
2. **Reservar:** `execute_method.sh stock.picking action_assign '[[<id>]]' '{}'` → reserva stock disponible → state: assigned
3. **Establecer cantidades:**
   - `update_record.sh stock.move.line '[<line_ids>]' '{"quantity": <qty>}'` (campo `quantity` en Odoo 18, NO `qty_done`)
   - `update_record.sh stock.move '[<move_ids>]' '{"picked": true}'` (marca como procesado por el operario)
4. **Validar:** `execute_method.sh stock.picking button_validate '[[<id>]]' '{}'`
   - Contexto para evitar backorder: `{"skip_backorder": true, "picking_ids_not_to_backorder": [<id>]}`
**Resultado:** state → `done`

### PICKING_VALIDATE_2STEP — Validación en 2 Pasos (Pick + Ship)
**Descripción:** Para almacenes con `delivery_steps = 'pick_ship'`. Se generan 2 pickings en secuencia.
**Pasos vía API:**
1. **Validar picking de preparación (PICK):** Seguir PICKING_VALIDATE_SEQUENCE para el picking interno (WH/PICK)
2. **El picking de envío (SHIP)** se genera automáticamente como `move_dest_ids` del primero
3. **Validar picking de envío (SHIP):** Seguir PICKING_VALIDATE_SEQUENCE para el picking de salida (WH/OUT)
**Configuración:** `update_record.sh stock.warehouse '[<id>]' '{"delivery_steps": "pick_ship"}'`
**Nota:** El segundo picking queda en `waiting` hasta que el primero se complete.

### PICKING_VALIDATE_3STEP — Validación en 3 Pasos (Pick + Pack + Ship)
**Descripción:** Para almacenes con `delivery_steps = 'pick_pack_ship'`. Se generan 3 pickings.
**Pasos vía API:**
1. **Validar PICK** (WH/PICK → zona de packing)
2. **Empaquetar (opcional):** `execute_method.sh stock.picking action_put_in_pack '[[<pack_picking_id>]]' '{}'`
   - Agrupa los move_lines en un `stock.quant.package`
   - Se puede llamar múltiples veces para crear múltiples paquetes
3. **Validar PACK** (WH/PACK → zona de envío)
4. **Validar SHIP** (WH/OUT → cliente)
**Configuración:** `update_record.sh stock.warehouse '[<id>]' '{"delivery_steps": "pick_pack_ship"}'`

### STOCK_PRE_POPULATE — Crear Stock Inicial (sin picking)
**Modelo:** stock.quant
**Descripción:** Para tests, se necesita stock disponible antes de crear pickings. Se puede crear directamente vía el método interno de stock.quant.
**Método Python (interno):** `stock.quant._update_available_quantity(product, location, qty)`
**Vía API (ajuste de inventario):**
1. Buscar/crear quant: `search_records.sh stock.quant '[["product_id","=",<id>],["location_id","=",<id>]]' '["id"]'`
2. Si no existe, crear: `create_record.sh stock.quant '{"product_id": <id>, "location_id": <id>, "inventory_quantity": <qty>}'`
3. Si existe, actualizar: `update_record.sh stock.quant '[<quant_id>]' '{"inventory_quantity": <qty>}'`
4. Aplicar: `execute_method.sh stock.quant action_apply_inventory '[[<quant_id>]]' '{}'`
**Nota:** `_update_available_quantity` es un método privado de Python y NO se puede llamar vía XML-RPC. Para API, usar el ajuste de inventario vía `action_apply_inventory`.
**Resultado:** stock.quant con `quantity` = valor indicado

### PICKING_RETURN — Devolver Albarán Validado
**⚠️ Este es el ÚNICO mecanismo para revertir un picking 'done'**
**Pasos via API:**
1. Crear wizard con contexto: `create_record.sh stock.return.picking '{"picking_id": <id>}' --context '{"active_id": <id>, "active_model": "stock.picking"}'`
   - SOLO funciona si picking._can_return() == True (state == 'done')
   - Las líneas `product_return_moves` se computan automáticamente con cantidad = 0
2. Ejecutar devolución completa: `execute_method.sh stock.return.picking action_create_returns_all '[[<wizard_id>]]' '{}'`
   - Calcula automáticamente las cantidades pendientes de devolución (descontando devoluciones previas)
   - O parcial: ajustar `quantity` en product_return_moves antes de ejecutar `action_create_returns`
3. El resultado es un nuevo picking en estado 'confirmed' (con action_confirm + action_assign ejecutados)
4. Validar el picking de devolución: `execute_method.sh stock.picking button_validate '[[<return_picking_id>]]' '{}'`

**Campos del wizard (auto-populated via _compute_moves_locations):**
- picking_id: el picking original
- product_return_moves: líneas con move_id, product_id, quantity (default 0), uom_id
**Nota:** También existe `action_create_exchanges` para crear intercambios (devolver + reenviar)

---

