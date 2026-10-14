## 25. HELPDESK — Tickets [Enterprise]

### TICKET_CREATE — Crear Ticket de Soporte
**Modelo:** helpdesk.ticket
**Descripción:** Crea un ticket de soporte en Helpdesk (Enterprise).
**Campos mínimos:**
```json
{
  "name": "Problema con facturación",
  "team_id": "<int>"
}
```
**Notas:**
- `name`: asunto del ticket (required)
- `team_id`: equipo de helpdesk (helpdesk.team) — `stage_id` se deduce del equipo si no se indica
- `partner_id`: cliente (opcional, se busca por email)
- `partner_email`: email del cliente
- `user_id`: asignado a (se computa según reglas del equipo)
- `priority`: '0' (Normal), '1' (Urgente), '2' (Alta), '3' (Crítica)
- `kanban_state`: 'normal', 'blocked', 'done'
- `stage_id`: se computa desde team_id (primera etapa del equipo)
- `ticket_type_id`: tipo de ticket (helpdesk.ticket.type)
- `tag_ids`: etiquetas
**Método/acción:** `create_record.sh helpdesk.ticket '...'`
**Precondiciones:** Módulo `helpdesk` instalado, al menos un equipo de helpdesk con etapas
**Resultado:** Ticket creado en primera etapa del equipo
**Verificación:** `name == <valor>`, `stage_id != False`, `team_id == <id>`
**Estrategia de reversión:** `delete_record.sh helpdesk.ticket '[<id>]'`
**Registros cascada generados:** Ninguno

### TICKET_ASSIGN — Asignar Ticket
**Modelo:** helpdesk.ticket
**Descripción:** Asigna un agente responsable al ticket.
**Método:** `update_record.sh helpdesk.ticket '[<id>]' '{"user_id": <user_id>}'`
**Notas:**
- `user_id`: usuario agente
- Se puede combinar con cambio de `kanban_state`
**Resultado:** `user_id` asignado
**Verificación:** `user_id == <id>`

### TICKET_CLOSE — Cerrar Ticket
**Modelo:** helpdesk.ticket
**Descripción:** Mueve el ticket a una etapa de cierre (fold=True en la etapa).
**Método:** `update_record.sh helpdesk.ticket '[<id>]' '{"stage_id": <closed_stage_id>}'`
**Notas:**
- La etapa destino debe tener `fold == True` (los tickets en etapas folded se consideran cerrados)
- Al cerrar se establece `close_date` automáticamente
- Para encontrar la etapa cerrada: `search_records.sh helpdesk.stage '[["fold","=",true],["team_ids","in",[<team_id>]]]' '["id","name"]'`
- SLA se evalúa al cerrar (si configurado)
**Precondiciones:** Etapa cerrada configurada para el equipo
**Resultado:** `stage_id` apunta a etapa cerrada, `close_date` establecido
**Verificación:** `close_date != False`, `fold == True` (campo related)
**Estrategia de reversión:** Reabrir moviendo a etapa anterior: `update_record.sh helpdesk.ticket '[<id>]' '{"stage_id": <open_stage_id>, "close_date": false}'`

### TICKET_CREATE_REPAIR — Crear Reparación desde Ticket [helpdesk_repair]
**Modelo:** helpdesk.ticket + repair.order
**Descripción:** Crea una orden de reparación vinculada a un ticket de helpdesk. Requiere módulo `helpdesk_repair` instalado y equipo con feature `use_product_repairs` habilitada.
**Método (vía API — recomendado):**
Crear directamente la repair.order con `ticket_id`:
```
create_record.sh repair.order '{"product_id": <id>, "partner_id": <id>, "ticket_id": <ticket_id>, "picking_type_id": <id>}'
```
**Método alternativo (vía acción del ticket):**
```
execute_method.sh helpdesk.ticket action_repair_order_form '[[<ticket_id>]]' '{}'
```
Esto devuelve un action dict — no es práctico vía API. Mejor crear directamente.
**Campos que se propagan del ticket a la reparación:**
- `product_id` → `default_product_id`
- `lot_id` → `default_lot_id`
- `partner_id` → `default_partner_id`
- `company_id` → `default_company_id`
- `name` → `default_description`
- `description` → `default_internal_notes`
**Notas:**
- El campo `ticket_id` existe en `repair.order` (añadido por `helpdesk_repair`)
- `repair_ids` en ticket: One2many inverso, permite `ticket.repair_ids` para ver reparaciones vinculadas
- `repairs_count` en ticket: campo compute con el número de reparaciones
- Al completar (done) o cancelar la reparación, se publica un mensaje automático en el ticket
- Para ver reparaciones del ticket: `execute_method.sh helpdesk.ticket action_view_repairs '[[<ticket_id>]]' '{}'`
**Precondiciones:** Módulo `helpdesk_repair` instalado, equipo con `use_product_repairs == True`
**Resultado:** repair.order creada en estado `draft` con `ticket_id` vinculado
**Verificación:** `repair.ticket_id == <ticket_id>`, `ticket.repairs_count > 0`

---

