## 17. EMAIL MARKETING (mailing.mailing)

### MAILING_CREATE — Crear Mailing
**Modelo:** mailing.mailing
**Descripción:** Crea una campaña de email marketing (mass mailing).
**Campos mínimos:**
```json
{
  "subject": "Asunto del email",
  "mailing_model_id": "<int>",
  "body_arch": "<html>Contenido del email</html>",
  "contact_list_ids": [[6, 0, [<list_ids>]]]
}
```
**Notas:**
- `subject`: asunto del email (obligatorio)
- `mailing_model_id`: ID del ir.model del modelo destinatario (ej: mailing.list, res.partner, crm.lead)
- `body_arch`: contenido HTML del email
- `contact_list_ids`: listas de contactos (obligatorio si mailing_model es mailing.list)
- `email_from`: remitente (se computa automáticamente)
- `schedule_type`: "now" o "scheduled"
- `schedule_date`: fecha programada si schedule_type == "scheduled"
- state: 'draft' → 'in_queue' → 'sending' → 'done'
**Método/acción:** `create_record.sh mailing.mailing '...'`
**Precondiciones:** Módulo `mass_mailing` instalado, al menos una lista de contactos o modelo con destinatarios
**Resultado:** state = `draft`
**Verificación:** `state == 'draft'`, `subject == <valor>`
**Estrategia de reversión:** `delete_record.sh mailing.mailing '[<id>]'` o archivar: `update_record.sh mailing.mailing '[<id>]' '{"active": false}'`
**Registros cascada generados:** Ninguno

### MAILING_SEND — Enviar Mailing
**Modelo:** mailing.mailing
**Descripción:** Pone en cola y envía el mailing a todos los destinatarios.
**Método:** `execute_method.sh mailing.mailing action_launch '[[<id>]]' '{}'`
**Precondiciones:** state == 'draft', destinatarios configurados (contact_list_ids o dominio)
**Restricciones:**
- `action_launch` establece `schedule_type = 'now'` y llama a `action_put_in_queue`
- UserError si no hay destinatarios seleccionados
- El envío real lo ejecuta el cron `mass_mailing.ir_cron_mass_mailing_queue`
**Resultado:** state → `in_queue` (luego `sending` → `done` via cron)
**Efectos secundarios:**
- Crea registros mail.mail para cada destinatario
- Actualiza estadísticas: sent, delivered, opened, clicked, bounced, etc.
**Verificación:** `state == 'in_queue'` (inmediato), `state == 'done'` (tras cron)
**Estrategia de reversión:**
- Si está en cola: `execute_method.sh mailing.mailing action_cancel '[[<id>]]' '{}'` — vuelve directamente a draft (NO existe `action_draft`), limpia schedule_date
- Si ya se envió (done): no hay reversión posible
**⚠️ PUNTO DE NO RETORNO:** Una vez enviado (done), los emails ya fueron entregados.
**Registros cascada generados:** mail.mail (N, uno por destinatario), mailing.trace (N, tracking de envío)

---

