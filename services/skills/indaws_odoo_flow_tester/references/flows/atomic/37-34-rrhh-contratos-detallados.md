## 34. RRHH — Contratos Detallados

### HR_CONTRACT_CREATE — Crear Contrato de Empleado
**(Detalle extendido del CONTRACT_CREATE en sección 13)**
**Modelo:** hr.contract
**Campos mínimos:**
```json
{
  "name": "Contrato test",
  "employee_id": "<int>",
  "wage": "<float>",
  "date_start": "YYYY-MM-DD"
}
```
**Notas:**
- `state`: 'draft' (New) → 'open' (Running) → 'close' (Expired) → 'cancel' (Cancelled)
- `structure_type_id`: tipo de estructura salarial (se computa si existe hr_payroll)
- `resource_calendar_id`: horario laboral (se computa desde el empleado, default: horario de la compañía)
- `contract_type_id`: tipo de contrato (opcional)
- `kanban_state`: 'normal' (Ongoing), 'done' (Ready), 'blocked' (Warning)
  - draft + kanban 'done' = "Incoming" (se abrirá automáticamente al inicio)
  - open + kanban 'blocked' = "Pending" (se cerrará automáticamente al final)
- `date_end`: fecha fin (opcional — indefinido si no se indica)
- `trial_date_end`: fecha fin del período de prueba
**Método/acción:** `create_record.sh hr.contract '...'`
**Resultado:** state = `draft`
**Verificación:** `state == 'draft'`, `wage == <valor>`
**Estrategia de reversión:** `delete_record.sh hr.contract '[<id>]'` — sin restricciones en estado draft. **⚠️ Si se activa (state='open'), auto-genera hr.work.entry.** Cleanup: cancel → eliminar work entries draft → eliminar contrato. Work entries validados: archivar contrato/empleado

### HR_CONTRACT_ACTIVATE — Activar Contrato (Running)
**Modelo:** hr.contract
**Descripción:** Pasa un contrato de borrador a activo (Running). Se puede hacer manualmente o automáticamente vía cron cuando llega la fecha de inicio.
**Método:** `update_record.sh hr.contract '[<id>]' '{"state": "open"}'`
**Precondiciones:** state == 'draft'
**Notas:**
- Normalmente el cron `hr_contract.ir_cron_data_contract_update_state` cambia automáticamente contratos de 'draft' a 'open' cuando `date_start <= today` y `kanban_state == 'done'`
- El empleado solo puede tener un contrato en estado 'open' a la vez
**Resultado:** state → `open`
**Verificación:** `state == 'open'`

### HR_CONTRACT_CLOSE — Cerrar Contrato (Expired)
**Modelo:** hr.contract
**Descripción:** Cierra un contrato activo (por fin de contrato, baja, etc.).
**Método:** `update_record.sh hr.contract '[<id>]' '{"state": "close"}'`
**Precondiciones:** state == 'open'
**Notas:**
- Normalmente el cron cambia automáticamente contratos de 'open' a 'close' cuando `date_end < today` y `kanban_state == 'blocked'`
- También se puede cancelar: `update_record.sh hr.contract '[<id>]' '{"state": "cancel"}'`
**Resultado:** state → `close`
**Verificación:** `state == 'close'`

---

