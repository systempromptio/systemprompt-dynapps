## CADENA 8: Gasto Empleado
**ID:** CHAIN_HR_EXPENSE
**Apps:** RRHH

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | EXPENSE_CREATE | Crear hr.expense | |
| 2 | EXPENSE_SUBMIT | Crear sheet + submit | approval_state=submit |
| 3 | EXPENSE_APPROVE | approve_expense_sheets | approval_state=approve |

### Plan de Rollback

| # | Accion |
|---|--------|
| 3 | action_reset_expense_sheets |
| 2 | (sheet vuelve a draft) |
| 1 | delete_record hr.expense |

### Puntos de No Retorno
- **Paso 3 (sheet approved)**: Si se genera asiento contable (via `action_sheet_move_post` en 18.0) y el diario tiene hash -> el asiento es irreversible
- **Nota**: En esta cadena basica (sin contabilizacion), no hay puntos de no retorno criticos

---

