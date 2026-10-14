## CADENA 7: Nota de Credito
**ID:** CHAIN_CREDIT_NOTE
**Apps:** Contabilidad

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | INVOICE_CREATE | Crear factura cliente | state=draft |
| 2 | INVOICE_POST | action_post | state=posted |
| 3 | INVOICE_REVERSE | Wizard account.move.reversal | Nota de credito creada |
| 4 | INVOICE_POST (NC) | action_post en nota de credito | state=posted |

### Plan de Rollback

| # | Accion |
|---|--------|
| 4 | NC: button_draft -> button_cancel -> delete |
| 3 | (NC eliminada en paso 4) |
| 2 | Original: button_draft -> button_cancel -> delete |
| 1 | (eliminada en paso 2) |

### Puntos de No Retorno
- **Paso 2 (`account.move.state=posted`)**: Si el diario tiene hash (inalterable_hash) -> no se puede volver a draft. Alternativa: nota de credito
- **Paso 4 (NC posted)**: Misma restriccion de hash para la nota de credito

---

