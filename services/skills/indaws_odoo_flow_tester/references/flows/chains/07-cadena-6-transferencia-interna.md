## CADENA 6: Transferencia Interna
**ID:** CHAIN_STOCK_INTERNAL
**Apps:** Inventario

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | PICKING_CREATE | Crear picking tipo internal con product + qty | state=draft |
| 2 | PICKING_CONFIRM | action_confirm | state=confirmed/assigned |
| 3 | PICKING_VALIDATE | button_validate | state=done |

### Plan de Rollback

| # | Accion |
|---|--------|
| 3 | PICKING_RETURN -> VALIDATE RETURN |
| 2-1 | (picking original queda intacto, return compensa) |

### Puntos de No Retorno
- **Paso 3 (`stock.picking.state=done`)**: Transferencia validada — no se puede cancelar, solo crear picking inverso (devolucion). Alternativa: wizard stock.return.picking
- **Nota**: La devolucion restaura el stock pero el picking original queda registrado en el historial

---

