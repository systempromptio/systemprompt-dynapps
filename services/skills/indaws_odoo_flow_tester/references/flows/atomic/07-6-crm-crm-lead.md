## 6. CRM (crm.lead)

### CRM_CREATE — Crear Oportunidad
**Modelo:** crm.lead
**Campos mínimos:**
```json
{
  "name": "Oportunidad test",
  "type": "opportunity",
  "partner_id": "<int>"
}
```
**Notas:**
- `type`: "opportunity" o "lead"
- `partner_id`: opcional
**Resultado:** active = True, type según configuración

### CRM_WON — Marcar como Ganada
**Método:** `execute_method.sh crm.lead action_set_won '[[<id>]]' '{}'`
**Efectos:** Desarchiva, mueve a stage is_won=True, probability=100

### CRM_LOST — Marcar como Perdida
**Método:** `execute_method.sh crm.lead action_set_lost '[[<id>]]' '{"lost_reason_id": <int>}'`
**Efectos:** Archiva (active=False), probability=0

### CRM_ARCHIVE — Archivar
**Método:** `execute_method.sh crm.lead toggle_active '[[<id>]]' '{}'`
**Efectos:** active=False, probability=0

### CRM_DELETE — Eliminar
**Método:** `delete_record.sh crm.lead '[<id>]'`
**Restricciones:** Ninguna. Solo limpia referencias de calendar.event

---

