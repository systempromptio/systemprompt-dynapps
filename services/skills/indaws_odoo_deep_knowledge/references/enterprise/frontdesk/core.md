<!-- verified: v18.0 @ 2026-06 -->
# Frontdesk — Core — **Enterprise (EE)**

> **EE-only module.** License OEEL-1 — NO existe en Community. `depends: ['hr','sms']`, sin módulos bridge `frontdesk_*`. Gotchas: `gotchas.md` FDG1-22.

Sistema de gestión de visitas (visitor management) de Odoo Enterprise. Un kiosk OWL público
permite a las visitas hacer check-in/check-out y notifica automáticamente al empleado anfitrión
(host) por Discuss, email y/o SMS.

Módulo: `odoo_enterprise/18/frontdesk`. Licencia `OEEL-1`. `depends: ['hr', 'sms']`.
No tiene módulos bridge `frontdesk_*` separados: todo el dominio vive en un único módulo.

---

## Modelos

### `frontdesk.frontdesk` — la estación (station)

`frontdesk/models/frontdesk_frontdesk.py:19`, `Frontdesk`. Una "estación" es un punto de recepción configurable (un
hall, una sede). El `_description` del modelo es solo `'Frontdesk'`; en UI se le llama "station".

Campos clave (`frontdesk/models/frontdesk_frontdesk.py:23-63`, `class Frontdesk`):
- `name` (Char, required), `company_id` (required, default `env.company`).
- `responsible_ids` (Many2many `res.users`, required) — recepcionistas. SIEMPRE reciben
  notificación Discuss en cada check-in.
- `theme` Selection light/dark (default light) — controla el aspecto del kiosk OWL.
- `image` (Image) — fondo del kiosk, servido por `/frontdesk/<id>/background`.
- `host_selection` (Boolean) — si True, la visita elige un anfitrión (`hr.employee`) en el kiosk.
- `authenticate_guest` (Boolean, default True).
- `ask_phone` / `ask_company` / `ask_email` (Selection `required`/`optional`/`none`).
  Defaults: phone=required, company=optional, email=none.
- `notify_email` / `notify_sms` / `notify_discuss` (Boolean) — canales de notificación al host.
  `notify_discuss` default True; los otros default False.
- `mail_template_id` (Many2one `mail.template`) / `sms_template_id` (Many2one `sms.template`).
  Default a `frontdesk.frontdesk_mail_template` / `frontdesk.frontdesk_sms_template`.
- `self_check_in` (Boolean) — muestra un QR en el kiosk para que la visita haga check-in desde
  su móvil.
- `drink_offer` (Boolean) + `drink_ids` (Many2many `frontdesk.drink`) — oferta de bebidas.
- `visitor_properties_definition` (PropertiesDefinition) — propiedades custom por estación,
  heredadas por cada visitor vía `station_id.visitor_properties_definition`.
- `access_token` (Char, default `uuid.uuid4()`, readonly, copy=False) — token de seguridad de la
  URL del kiosk.
- `kiosk_url` (Char, computed) — `<base_url>/kiosk/<id>/<access_token>` (`_compute_kiosk_url` l.108).
- Dashboard computed (`_compute_dashboard_data` l.73): `guest_on_site`, `pending`,
  `drink_to_serve`, `latest_check_in`.

NOTA sobre `groups=`: muchos campos de config (`host_selection`, `authenticate_guest`,
`notify_*`, `self_check_in`, `drink_offer`, `description`, `kiosk_url`) llevan
`groups='frontdesk.frontdesk_group_user'` → invisibles/inaccesibles para usuarios fuera del grupo.

Acciones:
- `action_open_kiosk` (l.113) → abre `kiosk_url` en la misma ventana (`act_url`, target self).
- `install_kiosk` (l.121) → PWA scoped app `/scoped_app?app_id=frontdesk&path=kiosk/...`.
- `action_open_visitors` (l.128) → lista de visitors de la estación.

### `frontdesk.visitor` — la visita

`frontdesk/models/frontdesk_visitor.py:9`, `FrontdeskVisitor`. `_order = 'check_in'`. Campos (`frontdesk/models/frontdesk_visitor.py:14-35`, `FrontdeskVisitor.active`):
- `name` (required), `phone`, `email`, `company` (Char, "Visitor Company" — texto libre, NO
  res.partner), `message` (Html).
- `host_ids` (Many2many `hr.employee`, domain `[('user_id', '!=', False)]` en v18) — anfitriones.
- `drink_ids` (Many2many `frontdesk.drink`), `served` (Boolean).
- `check_in` / `check_out` (Datetime), `duration` (Float computed, store, en horas).
- `state` Selection: `planned` / `checked_in` / `checked_out` / `canceled` (default `planned`).
- `station_id` (Many2one `frontdesk.frontdesk`, required).
- `visitor_properties` (Properties, definition `station_id.visitor_properties_definition`).
- `company_id` (Many2one `res.company`, required).

### `frontdesk.drink` — bebida ofrecida

`frontdesk/models/frontdesk_drink.py:6`, `FrontdeskDrink`. `_order = 'sequence, name'`. Campos:
- `name` (required), `drink_image` (Image), `sequence` (Integer default 1).
- `notify_user_ids` (Many2many `res.users`, **required**) — quién recibe el aviso Discuss cuando
  una visita pide esa bebida. Required: no puedes crear un drink sin responsables de servirlo.

Demo: `frontdesk_drink_1` (Water) y `frontdesk_drink_2` (Cola), ambos notifican a `base.user_admin`
(`data/frontdesk_data.xml`).

---

## Estados del visitor y transiciones

```
planned ──action_check_in──▶ checked_in ──action_check_out──▶ checked_out
   │                                                              ▲
   └──action_canceled──▶ canceled                                 │
                                       write(state=checked_in/out) hace el trabajo real
```

Toda la lógica de transición vive en `write()` (`frontdesk/models/frontdesk_visitor.py:37-46`, `FrontdeskVisitor.write`), NO en los
`action_*`. Los métodos `action_check_in/out/canceled/served` solo asignan `self.state = ...`,
y al disparar el `write` se ejecuta:
- `state == 'checked_in'` → setea `check_in = now()`, llama `_notify()`, y si hay `drink_ids`
  llama `_notify_to_people()`.
- `state == 'checked_out'` → setea `check_out = now()` y `served = True` (al salir se da por
  servida la bebida).

GOTCHA: como la notificación se dispara en `write({'state': 'checked_in'})`, cualquier escritura
masiva que ponga ese estado dispara los avisos. Ver FDG en `gotchas.md`.

---

## Flujo del kiosk (OWL)

El kiosk es un **public component** OWL registrado en `public_components` como `"frontdesk"`
(`frontdesk/static/src/frontdesk.js:275`, `frontdesk`). El controller `launch_frontdesk` (`frontdesk/controllers/main.py:26`, `class Frontdesk`)
renderiza la plantilla QWeb `frontdesk.frontdesk` tras `request.session.logout(keep_db=True)`
(la sesión se cierra siempre: el kiosk es anónimo).

Pantallas (cada una es un componente OWL registrado en `frontdesk_screens`):
`WelcomePage` → `VisitorForm` → `HostPage` (si `host_selection`) → `RegisterPage` →
`DrinkPage` (si `drink_offer`) → `EndPage`. También `QuickCheckIn` para planned visitors.

Flujo desktop:
1. **WelcomePage** (`welcome_page/welcome_page.js`): reloj + (si `self_check_in`) un QR generado
   con ZXing que apunta a `/kiosk/<id>/mobile/<tmp_token>`. El QR se regenera cada hora.
2. **VisitorForm**: la visita teclea nombre/teléfono/email/organización (según `ask_*`).
3. **HostPage** (solo si `host_selection`): la visita busca y elige al empleado anfitrión vía
   `/frontdesk/<id>/<token>/get_hosts` (name_search sobre `hr.employee`).
4. **RegisterPage**: en `onWillStart` llama `createVisitor()` → POST
   `/frontdesk/<id>/<token>/prepare_visitor_data` que CREA el `frontdesk.visitor` con
   `state=checked_in` y dispara `_notify()`. Inactividad de 15s vuelve al inicio (desktop).
5. **DrinkPage** (si `drink_offer`): añade `drink_ids` al visitor → `_notify_to_people()`.
6. **EndPage**: confirmación.

Quick check-in: `QuickCheckIn` lista los planned visitors (ventana ±45 min, `PLANNED_VISITOR_TIME`,
`frontdesk/models/frontdesk_frontdesk.py:17`, `PLANNED_VISITOR_TIME` y `frontdesk/models/frontdesk_frontdesk.py:162`, `Frontdesk._get_planned_visitors`). Al pulsar uno, POST `prepare_visitor_data` con `visitor_id` →
solo cambia el state a `checked_in` (no crea registro).

Endpoints del kiosk (`controllers/main.py`), todos `auth='public'` con verificación de token:
- `/kiosk/<id>/<token>` (http) — lanza el kiosk.
- `/kiosk/<id>/mobile/<token>` (http) — versión móvil vía QR; si token caducó renderiza
  `frontdesk.frontdesk_qr_expired`.
- `/kiosk/<id>/get_tmp_code/<token>` (json) — genera el código temporal del QR.
- `/frontdesk/<id>/<token>/get_frontdesk_data` (json) — config de la estación + drinks + langs.
- `/frontdesk/<id>/<token>/get_planned_visitors` (json).
- `/frontdesk/<id>/<token>/get_hosts` (json).
- `/frontdesk/<id>/<token>/prepare_visitor_data` (json, POST) — crea o actualiza el visitor.
- `/frontdesk/<id>/background` y `/frontdesk/<drink_id>/get_frontdesk_drinks` (http) — imágenes.

### Token: doble verificación

`_verify_token` (`frontdesk/controllers/main.py:17`, `Frontdesk._verify_token`):
1. `consteq(access_token, token)` → token permanente de la estación (kiosk principal).
2. Si falla: trata el token como `tmp_code(64 chars) + timestamp(19 chars)`. Válido si la
   diferencia con `now()` ≤ 3600s y `consteq(_get_tmp_code(), token[:64])`. Es el QR móvil:
   caduca a la hora. `_get_tmp_code` usa `tools.hmac` con la fecha (`frontdesk/models/frontdesk_frontdesk.py:176`, `Frontdesk._get_tmp_code`).

---

## Notificación al anfitrión

Núcleo en `frontdesk/models/frontdesk_visitor.py:70-142`, `FrontdeskVisitor._notify`. Disparado por `_notify()` al check-in.

1. **Recepcionistas (`responsible_ids`)**: SIEMPRE reciben un mensaje Discuss
   (`_notify_by_discuss`) con el patrón "<station> Check-In: <visitor> to meet <host>". No
   depende de ningún flag `notify_*`. (`frontdesk/models/frontdesk_visitor.py:77-83`, `FrontdeskVisitor._notify`).
2. **Anfitriones (`host_ids`)**: solo si `station.host_selection AND host_ids`. Según los flags:
   - `notify_discuss` → Discuss directo al host (`is_host=True` → usa `user_partner_id`).
   - `notify_email` → `_notify_by_email`: renderiza `mail_template_id` con ctx `host_name`/`lang`
     y hace `host.message_notify(...)` con layout `mail.mail_notification_light`, `force_send=True`.
     Solo a hosts con `work_email`.
   - `notify_sms` → `_notify_by_sms`: renderiza `sms_template_id` y `host._message_sms(...)`.
     Solo a hosts con `work_phone`.
3. **Bebidas**: `_notify_to_people` (l.93) avisa por Discuss a `drink_ids.notify_user_ids` con un
   link al visitor (`/odoo/action-<id>/<visitor_id>`).

`_notify_by_discuss` (l.107): por cada recipient obtiene/crea un `discuss.channel` 1-a-1
(`channel_get`) entre OdooBot (`base.partner_root`) y el destinatario, y hace `message_post`
con autor OdooBot, subtype `mail.mt_comment`. `is_host=True` usa `recipient.user_partner_id`;
si no, `recipient.partner_id`.

---

## Plantillas de notificación

- `data/mail_template_data.xml` → `frontdesk_mail_template` sobre modelo `frontdesk.visitor`.
  Subject "Your Visitor {{ object.name }} Requested To Meet You". `noupdate="1"`.
- `data/sms_template_data.xml` → `frontdesk_sms_template` sobre `frontdesk.visitor`. `noupdate="1"`.

OJO: el campo `mail_template_id`/`sms_template_id` tiene domain `[('model','=','frontdesk.frontdesk')]`
pero las plantillas data están sobre `frontdesk.visitor`. El render se hace siempre sobre el
visitor (`self.ids`), no sobre la estación. Ver FDG en `gotchas.md`.

---

## Seguridad

`security/security.xml` + `ir.model.access.csv`:
- Grupos: `frontdesk_group_user` (User) y `frontdesk_group_administrator` (Administrator, implica
  User). El admin se añade por defecto a `base.default_user`.
- Record rules por compañía: estaciones y visitors filtrados por `company_ids`.
- Record rule de **User**: solo ve estaciones donde `responsible_ids in user.id` y sus visitors
  (`station_id.responsible_ids in user.id`). Admin ve todo (`[(1,'=',1)]`).
- ACL User: drink read-only; visitor create=1 unlink=0; frontdesk create=0 unlink=0.

---

## Impresión de etiqueta

NO existe impresión de etiqueta (badge printing) en frontdesk v18 ni v19. No hay report QWeb de
etiqueta, ni acción de print, ni integración con impresoras. El único "report view" es
`views/frontdesk_report_views.xml` (vistas pivot/graph de analítica de visitas, no una etiqueta
física). Si un cliente lo pide, es desarrollo custom (report QWeb sobre `frontdesk.visitor`).
