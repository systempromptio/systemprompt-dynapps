## CADENA 25: Control de Calidad en Fabricacion
**ID:** CHAIN_QUALITY_MRP
**Apps:** Calidad + MRP + Inventario
**Descripcion:** Fabricacion con punto de control de calidad obligatorio: si el QC pasa, la produccion continua; si falla, se genera scrap del producto defectuoso.
**Tiempo estimado:** 3-5 minutos

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| mrp.production.action_confirm | mrp.production | stock.move, mrp.workorder | Buscar por production_id |
| quality.check.do_pass / do_fail | quality.check | — | Cambia quality_state del check |
| stock.scrap.action_validate | stock.scrap | stock.move | Campo move_ids en el scrap |

### Paso de Variables Entre Flujos
- `{chain.MO_id}` — ID de la MO
- `{chain.quality_check_id}` — ID del quality.check
- `{chain.quality_point_id}` — ID del quality.point (regla de QC)
- `{chain.scrap_id}` — ID del stock.scrap (si falla QC)

### Precondiciones
- Modulo `quality_mrp` instalado (Enterprise)
- Punto de control de calidad (quality.point) configurado para el producto o la operacion
- BoM con componentes y producto terminado
- Ubicacion de scrap configurada

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | QC_POINT_CREATE | Crear quality.point con product_ids, picking_type_ids, test_type | quality.point creado |
| 2 | MRP_CREATE | Crear mrp.production con product_id, bom_id, product_qty | state=draft |
| 3 | MRP_CONFIRM | action_confirm | state -> 'confirmed', quality.check generado automaticamente |
| 4 | — detectar QC — | `search_records.sh quality.check '[["production_id","=",<MO_id>]]'` | quality.check en estado 'none' |
| 5 | MRP_PRODUCE | Registrar qty_producing | qty_producing > 0 |
| 6a | QC_PASS | `execute_method.sh quality.check do_pass '[[<qc_id>]]' '{}'` | quality_state -> 'pass' |
| 6b | QC_FAIL (alternativo) | `execute_method.sh quality.check do_fail '[[<qc_id>]]' '{}'` | quality_state -> 'fail' |
| 7a | MRP_DONE (si pass) | button_mark_done | state -> 'done' |
| 7b | SCRAP_CREATE (si fail) | Crear stock.scrap con product_id, scrap_qty, production_id | scrap creado |
| 8b | SCRAP_VALIDATE (si fail) | `execute_method.sh stock.scrap action_validate '[[<scrap_id>]]' '{}'` | scrap.state -> 'done' |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 8b | (scrap done) | **PUNTO DE NO RETORNO** — scrap validado no se puede cancelar | Ajuste de inventario manual |
| 7a | (MO done) | **PUNTO DE NO RETORNO** | Scrap o ajuste inventario |
| 7b | SCRAP_DELETE | Solo si state != 'done' | |
| 6 | (QC state) | No tiene rollback directo | Se puede re-ejecutar do_pass/do_fail |
| 5 | (qty_producing) | Resetear a 0 | |
| 4 | (detectado) | | |
| 3 | MRP_CANCEL | action_cancel | Solo si state != 'done' |
| 2 | MRP_DELETE | delete_record | Solo desde cancel/draft |
| 1 | QC_POINT_DELETE | delete_record quality.point | Solo si fue creado durante el test |

### Puntos de No Retorno
- **Paso 7a (MO done)**: Fabricacion completada, solo scrap
- **Paso 8b (scrap done)**: Producto desguazado, stock ajustado definitivamente

---

