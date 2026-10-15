## CADENA 5: Fabricacion Simple
**ID:** CHAIN_MRP_SIMPLE
**Apps:** Fabricacion + Inventario

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | MRP_CREATE | Crear mrp.production con product_id + bom_id | state=draft |
| 2 | MRP_CONFIRM | execute_method action_confirm | state=confirmed |
| 3 | MRP_ASSIGN | execute_method action_assign | state=progress (si hay stock componentes) |
| 4 | MRP_PRODUCE | Registrar qty_producing | qty_producing > 0 |
| 5 | MRP_DONE | execute_method button_mark_done | state=done |

### Plan de Rollback
**Fabricacion done = punto de no retorno parcial**
- Si state != done: action_cancel -> delete
- Si state == done: Requiere ajuste de inventario manual o scrap

### Puntos de No Retorno
- **Paso 5 (`mrp.production.state=done`)**: Fabricacion completada — stock de componentes consumido y producto terminado creado. Alternativa: scrap o ajuste de inventario manual
- **Nota**: Si state != 'done', la MO se puede cancelar y eliminar sin restricciones

---

