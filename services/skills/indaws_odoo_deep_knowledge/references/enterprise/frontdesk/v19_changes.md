# Frontdesk — v19 Changes

Diffs verificados entre `odoo_enterprise/18/frontdesk` y `odoo_enterprise/19/frontdesk`.
`depends` sigue siendo `['hr', 'sms']`. Mismos 3 modelos. Sin nuevos módulos bridge.

---

## Modelo `frontdesk.frontdesk`

- Clase renombrada `Frontdesk` → `FrontdeskFrontdesk`. Ahora `_inherit = ['hr.mixin']` y
  `_order = 'is_favorite desc'`.
- Nuevo `host_ids` (Many2many `hr.employee`) en la ESTACIÓN: lista cerrada de hosts elegibles,
  domain `[('company_id','=',company_id), '|', ('work_email','!=',False), ('work_phone','!=',False)]`.
- `host_selection` y `responsible_ids` ganan `help`.
- **Se ELIMINA `mail_template_id` de la estación**: el email ya no usa una `mail.template`
  configurable; se renderiza directamente la plantilla QWeb `frontdesk.frontdesk_mail_template`
  (sigue existiendo `sms_template_id`).
- Nuevo `notify_warning` (Char, computed `_compute_notify_warning`): lista los hosts que NO tienen
  `user_id` pero sí `work_email`/`work_phone` (dependen del fallback de notificación).
- `_compute_dashboard_data`: el read_group de drinks añade `('state','!=','canceled')`.
- `_compute_kiosk_url` usa `odoo.tools.urls.urljoin` en vez de `werkzeug.urls.url_join`.
- Nuevo `copy_data` que añade " (copy)" al nombre al duplicar.

## Modelo `frontdesk.visitor`

- `_inherit = ['mail.thread', 'hr.mixin']` → **el visitor ahora tiene chatter y tracking**.
- `host_ids` domain cambia: de `[('user_id','!=',False)]` a
  `['|', ('work_email','!=',False), ('work_phone','!=',False)]` con `company_id`. → hosts sin
  usuario Odoo ya son válidos.
- `state` con `tracking=True`. `station_id` con `index=True`.
- Nuevo `action_planned` (vuelve a planned). Nuevo `create` (`@api.model_create_multi`) que llama
  `_check_and_notify_visitor()`.
- `_notify` (host branch): si el host tiene `user_id` → Discuss; si NO y hay `work_email` (y
  `notify_email` está OFF) → email de fallback; idem SMS. Es decir, fallback automático cuando el
  host no tiene cuenta Odoo.
- `_notify_by_discuss`: salta recipients sin `user_id`/partner cuando `is_host`; usa
  `_get_or_create_chat` (antes `channel_get`).
- `_notify_by_email`: ya no usa `station.mail_template_id`; renderiza
  `frontdesk.frontdesk_mail_template` con `ir.qweb._render(lang=...)` y hace `self.message_post(...)`
  (chatter, `message_type='email'`, subtype `mail.mt_comment`), `partner_ids = host.work_contact_id`.
- `_notify_by_sms`: ahora `self._message_sms_with_template(template=station.sms_template_id,
  partner_ids=host_ids.filtered('work_phone').work_contact_id)`.
- **Nuevo `_check_and_notify_visitor` + `_check_resources_leave`**: en `create`, comprueba si algún
  host está de baja/ausente vía `resource.calendar._leave_intervals_batch` y, si lo está, envía un
  mensaje (email al visitante si tiene email / SMS si tiene phone) avisando "Your host isn't
  available", incluyendo el manager del host. Imports nuevos `pytz`, `timedelta`.

## Modelo `frontdesk.drink`

Sin cambios funcionales (solo una línea en blanco).

## Controllers (`controllers/main.py`)

- Todas las rutas `type='json'` → `type='jsonrpc'` (cambio de Odoo 19 en RPC).
- `get_hosts` → renombrado a **`hosts_infos`** con paginación (`limit`, `offset`, `domain`) y
  validación estricta del domain (solo `department_id '='` y `display_name 'ilike'`, si no
  `UserError`). Devuelve `records` (id, display_name, job_id, avatar) + `length`. Usa
  `search_fetch` y `image_data_uri` para el avatar.
- Nuevo endpoint **`get_departments`**: lista `hr.department` con conteo de hosts elegibles por
  departamento (para filtrar hosts por departamento en el kiosk).
- Nuevo endpoint HTTP **`/frontdesk/visitor/check_out/<int:visitor_id>`** (`auth='user'`,
  requiere `frontdesk.frontdesk_group_user`): hace check-out de un visitor y renderiza
  `frontdesk.frontdesk_visitor_check_out`. Permite check-out desde un enlace backend.
- Imports nuevos: `UserError`, `odoo.fields.Domain`, `image_data_uri`.

## Resumen de impacto

1. **Hosts sin cuenta Odoo**: gran cambio funcional. v19 permite empleados sin `user_id` como
   anfitriones y los notifica por email/SMS de fallback automáticamente.
2. **Chatter en el visitor**: trazabilidad de notificaciones y del estado (tracking).
3. **Aviso de ausencia al visitante**: si el host está de leave, se avisa al visitante en el
   check-in (vía resource.calendar leaves, NO planning ni calendar.event).
4. **mail.template eliminada de la estación**: el email usa una plantilla QWeb fija.
5. **Kiosk con filtro por departamento + avatar + paginación** de hosts.
6. **RPC**: `json` → `jsonrpc`; relevante para cualquier override de los endpoints del kiosk.
