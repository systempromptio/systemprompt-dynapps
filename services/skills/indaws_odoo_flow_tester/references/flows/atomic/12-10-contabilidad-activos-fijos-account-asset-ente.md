## 10. CONTABILIDAD — Activos Fijos (account.asset) [Enterprise]

### ASSET_CREATE — Crear Activo Fijo
**Modelo:** account.asset
**Descripción:** Crea un activo fijo para depreciación. Módulo Enterprise `account_asset`.
**Campos mínimos:**
```json
{
  "name": "Activo test",
  "original_value": "<float>",
  "account_asset_id": "<int>",
  "account_depreciation_id": "<int>",
  "account_depreciation_expense_id": "<int>",
  "journal_id": "<int>",
  "method": "linear",
  "method_number": "<int>",
  "method_period": "1",
  "acquisition_date": "YYYY-MM-DD",
  "prorata_computation_type": "none"
}
```
**Notas:**
- `method`: "linear" o "degressive"
- `method_number`: número de depreciaciones
- `method_period`: "1" (mensual) o "12" (anual)
- `account_asset_id`: cuenta del activo (ej: 2180 Inmovilizado)
- `account_depreciation_id`: cuenta de depreciación acumulada
- `account_depreciation_expense_id`: cuenta de gasto de depreciación
- `prorata_computation_type`: "none", "constant_periods", "daily_computation"
- `salvage_value`: valor residual (opcional, default 0)
**Resultado:** state = `draft`
**Verificación:** `state == 'draft'`, `original_value == <valor>`
**Estrategia de reversión:** `delete_record.sh account.asset '[<id>]'` — solo posible en estado `draft`
**Registros cascada generados:** Ninguno hasta que se valide

### ASSET_DEPRECIATE — Ejecutar Depreciación (Validar Activo)
**Modelo:** account.asset
**Descripción:** Valida el activo y genera el plan de depreciación. Luego, los asientos de depreciación se publican automáticamente cuando llega su fecha.
**Método:** `execute_method.sh account.asset validate '[[<id>]]' '{}'`
**Precondiciones:** state == 'draft', campos obligatorios completos
**Restricciones:**
- Necesita `original_move_line_ids` o valores manuales
- La fecha de adquisición no puede ser posterior a la fecha del último asiento de depreciación
**Resultado:** state → `open` (Running)
**Efectos secundarios:**
- Genera board de depreciación (depreciation_move_ids)
- Los asientos se publican automáticamente al llegar su fecha (vía cron o acción manual)
**Verificación:** `state == 'open'`, `depreciation_entries_count > 0` (tras publicar)
**Estrategia de reversión:**
1. Pausar: **⚠️ `pause()` NO funciona via RPC** (tipo date no se auto-convierte). Workaround: `update_record.sh account.asset '[<id>]' '{"state": "paused"}'`. El método `set_to_paused` NO existe.
2. Cancelar asientos draft asociados
3. En estado 'open', no se puede volver a draft directamente — se debe cerrar (dispose)
**Registros cascada generados:** account.move (N asientos de depreciación, inicialmente en draft)

### ASSET_DISPOSE — Dar de Baja Activo
**Modelo:** account.asset
**Descripción:** Cierra/dispone un activo fijo, generando el asiento de baja con el valor residual.
**Método:** `execute_method.sh account.asset set_to_close '[[<id>]]' '{"invoice_line_ids": [], "date": "YYYY-MM-DD"}'`
**Precondiciones:** state == 'open' o 'paused'
**Restricciones:**
- La fecha de disposición no puede ser anterior a la fecha de bloqueo fiscal
- Si tiene gross increases (children_ids) en estado draft/open, no permite disposal automático con factura
- `invoice_line_ids`: lista vacía para disposición sin venta, o IDs de líneas de factura si se vende el activo
**Resultado:** state → `close`
**Efectos secundarios:**
- Genera asiento de disposición (account.move) con valor residual
- Cancela asientos de depreciación futuros en draft
**Verificación:** `state == 'close'`, `value_residual == 0`
**Estrategia de reversión:** No hay reversión directa. Habría que crear un nuevo activo manualmente.
**⚠️ PUNTO DE NO RETORNO:** Una vez cerrado, no se puede reabrir fácilmente (set_to_running existe pero requiere gestión manual de asientos).
**Registros cascada generados:** account.move (1 asiento de disposición)

### ASSET_CANCEL — Cancelar Activo
**Modelo:** account.asset
**Descripción:** Cancela un activo, revirtiendo todos los asientos de depreciación publicados y eliminando los pendientes en draft.
**Método:** `execute_method.sh account.asset set_to_cancelled '[[<id>]]' '{}'`
**Precondiciones:** state in ('draft', 'open', 'paused')
**Restricciones:**
- Revierte automáticamente todos los asientos de depreciación publicados (crea reversal entries)
- Elimina los asientos de depreciación en draft (`force_delete=True`)
- Genera mensaje en el chatter con detalle de cuentas afectadas y montos
- Resetea `asset_paused_days` a 0
**Resultado:** state → `cancelled`
**Verificación:** `state == 'cancelled'`
**Estrategia de reversión:** `execute_method.sh account.asset set_to_draft '[[<id>]]' '{}'` → state → `draft`

### ASSET_DRAFT — Volver a Borrador
**Modelo:** account.asset
**Descripción:** Devuelve un activo cancelado al estado borrador.
**Método:** `execute_method.sh account.asset set_to_draft '[[<id>]]' '{}'`
**Precondiciones:** state == 'cancelled'
**Resultado:** state → `draft`
**Verificación:** `state == 'draft'`

### ASSET_RUNNING — Reactivar Activo
**Modelo:** account.asset
**Descripción:** Devuelve un activo cerrado o pausado al estado running.
**Método:** `execute_method.sh account.asset set_to_running '[[<id>]]' '{}'`
**Precondiciones:** state in ('close', 'paused')
**Restricciones:**
- Si existen depreciaciones y el último asiento no tiene `asset_remaining_value == 0`, genera una modificación (asset.modify)
- Resetea `net_gain_on_sale` a 0
**Resultado:** state → `open`
**Verificación:** `state == 'open'`

### ASSET_PAUSE — Pausar Activo
**Modelo:** account.asset
**Descripción:** Pone un activo en estado de pausa, generando un asiento de depreciación proporcional al tiempo transcurrido en el período actual.
**Método:** `execute_method.sh account.asset pause '[[<id>]]' '{"pause_date": "YYYY-MM-DD"}'`
**Precondiciones:** state == 'open'
**Resultado:** state → `paused`
**Verificación:** `state == 'paused'`
**Estrategia de reversión:** `execute_method.sh account.asset resume_after_pause '[[<id>]]' '{}'` — reabre y ajusta la depreciación

---

