## 28. APROBACIONES [Enterprise]

### APPROVAL_CREATE — Crear Solicitud de Aprobación
**Modelo:** approval.request
**Descripción:** Crea una solicitud de aprobación genérica (gastos, compras, viajes, etc.).
**Campos mínimos:**
```json
{
  "category_id": "<int>",
  "name": "Solicitud de aprobación test"
}
```
**Notas:**
- `category_id`: categoría de aprobación (approval.category, required) — define los campos visibles y los aprobadores
- `name`: título de la solicitud (se computa desde categoría si no se indica)
- `request_owner_id`: solicitante (default: usuario actual)
- `date`: fecha de la solicitud
- `partner_id`, `amount`, `quantity`, `reference`: campos opcionales según configuración de la categoría
- `product_line_ids`: líneas de producto (si la categoría tiene has_product != 'no')
- `reason`: descripción/justificación
- `request_status`: 'new' → 'pending' → 'approved' / 'refused' / 'cancel'
**Método/acción:** `create_record.sh approval.request '...'`
**Precondiciones:** Módulo `approvals` instalado, categoría de aprobación configurada con aprobadores
**Resultado:** request_status = `new`
**Verificación:** `request_status == 'new'`, `category_id == <id>`
**Estrategia de reversión:** `delete_record.sh approval.request '[<id>]'`
**Registros cascada generados:** approval.approver (N, según configuración de la categoría)

### APPROVAL_SUBMIT — Enviar Solicitud de Aprobación
**Modelo:** approval.request
**Descripción:** Envía la solicitud para que los aprobadores la revisen.
**Método:** `execute_method.sh approval.request action_confirm '[[<id>]]' '{}'`
**Precondiciones:** request_status == 'new'
**Restricciones:**
- Verifica que los campos obligatorios de la categoría estén completos
- Genera notificaciones a los aprobadores
**Resultado:** request_status → `pending`
**Efectos secundarios:**
- Crea actividades de aprobación para cada aprobador
- Envía notificaciones por email
**Verificación:** `request_status == 'pending'`
**Estrategia de reversión:** Retirar: `execute_method.sh approval.request action_withdraw '[[<id>]]' '{}'` — request_status → `cancel`

### APPROVAL_APPROVE — Aprobar Solicitud
**Modelo:** approval.request
**Descripción:** Un aprobador aprueba la solicitud.
**Método:** `execute_method.sh approval.request action_approve '[[<id>]]' '{}'`
**Precondiciones:** request_status == 'pending', usuario es aprobador designado
**Restricciones:**
- El usuario debe estar en la lista de aprobadores (approval.approver) con status != 'approved'
- Si la categoría requiere aprobación secuencial (approver_sequence=True), debe ser el turno del aprobador
- Si hay múltiples aprobadores, la solicitud puede necesitar el mínimo de aprobaciones definido en la categoría
**Resultado:** request_status → `approved` (si se alcanza el mínimo de aprobaciones)
**Verificación:** `request_status == 'approved'`
**Estrategia de reversión:** No hay reversión directa. Se podría cancelar y recrear.
**Para rechazar:** `execute_method.sh approval.request action_refuse '[[<id>]]' '{}'` — request_status → `refused`

---

