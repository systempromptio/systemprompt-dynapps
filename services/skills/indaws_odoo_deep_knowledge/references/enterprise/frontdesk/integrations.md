# Frontdesk — Integrations

Cómo `frontdesk` se conecta con otros módulos. `depends: ['hr', 'sms']` únicamente. NO existen
módulos bridge `frontdesk_*` separados: todas las integraciones viven dentro del propio módulo.

---

## 1. Bridges

### `frontdesk` ↔ `hr.employee` (anfitrión / host)

El anfitrión es SIEMPRE un `hr.employee`, no un `res.users` ni `res.partner`.

- `frontdesk.visitor.host_ids` (Many2many `hr.employee`), domain v18 `[('user_id','!=',False)]`
  (`frontdesk/models/frontdesk_visitor.py:20`, `FrontdeskVisitor.host_ids`). Es decir, en v18 solo empleados con usuario Odoo pueden ser
  hosts.
- En el kiosk, `/frontdesk/<id>/<token>/get_hosts` hace `hr.employee.name_search` filtrando
  `user_id != False` y `company_id == station.company_id` (`frontdesk/controllers/main.py:73-78`, `class Frontdesk`).
- Para notificar, el código usa `host.user_partner_id` (Discuss/email a su partner de usuario),
  `host.work_email` (email) y `host.work_phone` (SMS) (`frontdesk/models/frontdesk_visitor.py:110`, `FrontdeskVisitor._notify_by_discuss` y `frontdesk/models/frontdesk_visitor.py:116`, `FrontdeskVisitor._notify_by_email` y `frontdesk/models/frontdesk_visitor.py:134`, `FrontdeskVisitor._notify_by_sms`).

**Implicación**: si el empleado no tiene `user_id` (v18) no es seleccionable como host. v19 lo
cambia (ver §6).

### `frontdesk` ↔ `mail` / Discuss / SMS (notificación al host)

Tres canales, todos disparados en `_notify()` al check-in (`frontdesk/models/frontdesk_visitor.py:70-105`, `FrontdeskVisitor._notify`):

| Canal | Flag estación | Método | Destinatario | Requisito host |
|-------|---------------|--------|--------------|----------------|
| Discuss recepción | (siempre) | `_notify_by_discuss` | `responsible_ids` (res.users) | — |
| Discuss host | `notify_discuss` | `_notify_by_discuss(is_host=True)` | `host_ids` | tener user |
| Email | `notify_email` | `_notify_by_email` | `host_ids` | `work_email` |
| SMS | `notify_sms` | `_notify_by_sms` | `host_ids` | `work_phone` |
| Discuss bebida | (si drinks) | `_notify_to_people` | `drink_ids.notify_user_ids` | — |

Las dos últimas (host email/SMS) y la Discuss-host requieren además `station.host_selection=True`.

- **Discuss** (`_notify_by_discuss`, l.107): obtiene/crea un `discuss.channel` 1-a-1 con
  `channel_get([partner])` como SUPERUSER, autor `base.partner_root` (OdooBot), subtype
  `mail.mt_comment`. NO usa el chatter del visitor: son DMs directos.
- **Email** (`_notify_by_email`, l.114): renderiza `station.mail_template_id` (`_render_field`
  sobre el visitor) y `host.message_notify(...)` con `email_layout_xmlid='mail.mail_notification_light'`,
  `force_send=True`, `partner_ids=host.user_partner_id`.
- **SMS** (`_notify_by_sms`, l.132): renderiza `station.sms_template_id` y `host._message_sms(...)`
  vía el módulo `sms` (dependencia directa del manifest).

### `frontdesk` ↔ `website` (kiosk público)

`depends` NO incluye `website`, pero los controllers usan `website=True`
(`frontdesk/controllers/main.py:26`, `class Frontdesk` y `frontdesk/controllers/main.py:33`, `class Frontdesk`) para envolver el kiosk en el layout web y servir assets frontend.
El kiosk usa el bundle propio `frontdesk.assets_frontdesk` (manifest) que incluye Bootstrap
frontend + ZXing (QR). El componente raíz es un `public_components` OWL (`frontdesk/static/src/frontdesk.js:275`, `frontdesk`).

### `frontdesk` ↔ `calendar` — NO aplica

No hay integración con `calendar` en v18 ni v19: no se crean `calendar.event`, ni se leen para
matchear visitas con reuniones. Las "planned visitors" son `frontdesk.visitor` con `state=planned`
y un `check_in` futuro, creadas manualmente en backend. Si el cliente quiere ligar visitas a
reuniones del calendario → desarrollo custom.

### `frontdesk` ↔ `planning` — NO aplica

No hay integración con `planning`. La disponibilidad del host no se comprueba en v18. v19 SÍ
comprueba ausencias del host pero vía `resource.calendar` (leaves), no vía `planning` (ver §6).

---

## 2. Diagrama del flujo kiosk

```
                    GET /kiosk/<id>/<access_token>            controllers/main.py:26
                            │  (auth=public, _verify_token, session.logout)
                            ▼
              QWeb 'frontdesk.frontdesk'  →  OWL public_component "frontdesk"
                            │                         frontdesk.js:275
        ┌───────────────────┴───────────────────┐
        ▼ desktop                                ▼ self_check_in QR
   WelcomePage ──(reloj)                    QR → /kiosk/<id>/mobile/<tmp_token>
        │                                        (caduca 1h, FDG6)
        ▼
   VisitorForm ──(ask_phone/email/company)
        │
        ▼  host_selection?
   ┌────┴────┐
   │ sí      │ no
   ▼         │
 HostPage    │   GET /frontdesk/<id>/<token>/get_hosts  (hr.employee.name_search)
   └────┬────┘
        ▼
   RegisterPage ──onWillStart──▶ POST /frontdesk/<id>/<token>/prepare_visitor_data
        │                              │  controllers/main.py:80
        │                              ▼
        │              frontdesk.visitor.create(state=checked_in)
        │                              │  → write() (FDG1)
        │                              ▼
        │                         _notify()  ──▶ Discuss(responsibles)
        │                                     ──▶ Discuss/email/SMS(hosts) si host_selection
        ▼  drink_offer?
   DrinkPage ──▶ POST prepare_visitor_data(drink_ids) ──▶ _notify_to_people() ──▶ Discuss(drink notify_user_ids)
        │
        ▼
    EndPage  (15s inactividad → vuelve a WelcomePage en desktop)

   QuickCheckIn (planned ±45min) ──▶ POST prepare_visitor_data(visitor_id) ──▶ write(state=checked_in)
```

---

## 3. Modelos (resumen para integradores)

- `frontdesk.frontdesk` — estación. Config del kiosk + canales de notificación. `company_id`,
  `responsible_ids`, `access_token`.
- `frontdesk.visitor` — visita. `state` (planned/checked_in/checked_out/canceled), `host_ids`
  (hr.employee), `drink_ids`, `check_in/out`, `duration`. Lógica de notificación en `write()`.
- `frontdesk.drink` — bebida. `notify_user_ids` (res.users, required).

Detalle completo en `core.md`.

---

## 4. Configuración (estación, drinks, theme)

**Estación** (`frontdesk.frontdesk`):
- `responsible_ids`: recepcionistas (reciben Discuss siempre).
- `ask_phone/ask_company/ask_email`: required/optional/none — qué pide el formulario del kiosk.
- `host_selection`: activar para que la visita elija anfitrión Y para que los hosts reciban
  notificaciones (sin él, los hosts nunca se notifican — FDG3).
- `notify_discuss/notify_email/notify_sms`: canales de aviso al host.
- `mail_template_id` / `sms_template_id`: plantillas. CREARLAS sobre modelo `frontdesk.visitor`,
  no `frontdesk.frontdesk` (FDG5).
- `self_check_in`: muestra QR para check-in desde móvil del visitante.
- `theme`: light/dark — estilo visual del kiosk OWL.
- `image`: fondo del kiosk.
- `visitor_properties_definition`: campos custom por estación (Properties).

**Drinks** (`frontdesk.drink`):
- Activar `drink_offer` + asignar `drink_ids` en la estación.
- Cada drink necesita `notify_user_ids` (required) — quién recibe el aviso de "servir".

**Theme/assets**: el kiosk carga `frontdesk.assets_frontdesk` (primary_variables.scss +
bootstrap_overridden.scss + frontend bundles + ZXing). El backend dashboard carga
`web.assets_backend` → `frontdesk/static/src/js/**`.

---

## 5. Gotchas relevantes (ver `gotchas.md`)

- **FDG1**: notificaciones en `write()`, no en `action_*`.
- **FDG2**: recepcionistas reciben Discuss siempre, ignoran flags `notify_*`.
- **FDG3**: sin `host_selection` los hosts NUNCA se notifican.
- **FDG4**: host sin `work_email`/`work_phone` → email/SMS silenciosamente omitido.
- **FDG5**: plantilla mail/SMS debe ser de `frontdesk.visitor` pese al domain.
- **FDG7**: el kiosk hace `session.logout`.
- **FDG13**: `public_user` se añade a la company solo por onchange (no en create por script).

---

## 6. v19 — qué cambia en integraciones

Detalle completo en `v19_changes.md`. Resumen de impacto en integraciones:

- **Hosts sin usuario Odoo**: el domain de `host_ids` pasa a
  `['|', ('work_email','!=',False), ('work_phone','!=',False)]` (ya no exige `user_id`). Empleados
  sin user pueden ser hosts y reciben email/SMS de fallback. `_notify_warning` avisa de hosts sin
  user que dependen del fallback.
- **Chatter en el visitor**: `frontdesk.visitor` hereda `mail.thread` (+ `hr.mixin`). El email al
  host pasa a `message_post` (deja traza en chatter) renderizando `frontdesk.frontdesk_mail_template`
  vía `ir.qweb._render`. SMS pasa a `_message_sms_with_template`. `state` con `tracking=True`.
- **Aviso de ausencia del host** (`_check_and_notify_visitor`, nuevo en `create`): comprueba
  `resource.calendar._leave_intervals_batch` (LEAVES, no planning) y, si el host está de baja/leave,
  envía email/SMS AL VISITANTE avisando. Esto introduce dependencia funcional con `resource`/leaves.
- **Discuss**: `channel_get` → `_get_or_create_chat`; usa `work_contact_id` en vez de
  `user_partner_id` para los partner_ids.
- **Departamentos en el kiosk**: nuevos endpoints `get_departments` y `hosts_infos` (con paginación
  y avatar) → integración más rica con `hr.department`.
- **Controllers**: `type='json'` → `type='jsonrpc'`; nuevo endpoint HTTP `/frontdesk/visitor/check_out/<id>`
  (auth=user) para check-out desde backend.
