## CADENA 4: CRM a Venta
**ID:** CHAIN_CRM_TO_SALE
**Apps:** CRM + Ventas

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | CRM_CREATE | Crear oportunidad | type=opportunity |
| 2 | — avanzar stage — | Mover por stages del pipeline | stage_id actualizado |
| 3 | CRM_CREATE_QUOTATION | Accion "New Quotation" desde lead | SO creado con tag_ids del lead |
| 4 | SALE_CONFIRM | Confirmar SO | state=sale |
| 5 | CRM_WON | Marcar oportunidad como ganada | probability=100 |

### Plan de Rollback

| # | Accion |
|---|--------|
| 5 | CRM revert: toggle_active (si se archivo) |
| 4 | SALE_CANCEL |
| 3 | SALE_DELETE |
| 2 | (no aplica) |
| 1 | CRM_DELETE |

### Puntos de No Retorno
- **Paso 4 (sale confirmed)**: Si el SO genera pickings y estos se validan (en cadena extendida), el picking done es punto de no retorno
- **Nota**: En esta cadena basica (sin entrega ni facturacion), no hay puntos de no retorno criticos — todo es reversible

---

