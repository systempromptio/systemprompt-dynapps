<!-- verified: v18.0 @ 2026-06 -->
# Frontdesk — Gotchas (FDG) — **Enterprise (EE)**

> **EE-only.** `frontdesk` es módulo Enterprise (license OEEL-1, `odoo_enterprise/18/frontdesk/`). No existe en Community. `depends: ['hr','sms']`, sin módulos bridge. Verificado contra fichero:línea de v18 (diffs v19 anotados).

Prefijo **FDG**. Verificado contra `odoo_enterprise/18/frontdesk` (v18 salvo nota explícita).

---

### FDG1 — Las notificaciones se disparan en `write()`, no en los `action_*`

`action_check_in` solo hace `self.state = 'checked_in'` (`frontdesk/models/frontdesk_visitor.py:54-56`, `FrontdeskVisitor.action_check_in`).
El trabajo real (`check_in = now()`, `_notify()`, `_notify_to_people()`) está en
`write()` (`frontdesk/models/frontdesk_visitor.py:37-46`, `FrontdeskVisitor.write`).

**Síntoma**: cualquier escritura que ponga `state='checked_in'` (import, server action, script
XML-RPC, escritura masiva en lista) dispara notificaciones a recepcionistas y hosts y sobreescribe
`check_in` con el `now()` del momento de la escritura, no el real.

**Fix**: para cargas masivas o correcciones, evita escribir `state='checked_in'`; ajusta el estado
con cuidado o crea con `state` final ya puesto evitando el flujo de notificación (en v18 `create`
no notifica; solo `write` lo hace).

---

### FDG2 — Los recepcionistas (`responsible_ids`) siempre reciben Discuss, ignorando los flags `notify_*`

`_notify` (`frontdesk/models/frontdesk_visitor.py:77-83`, `FrontdeskVisitor._notify`) notifica a `responsible_ids` por Discuss
incondicionalmente. Los flags `notify_discuss/email/sms` SOLO controlan la notificación a los
`host_ids`, y SOLO si `station.host_selection` está activo.

**Síntoma**: "desactivé las notificaciones pero los responsables siguen recibiendo mensajes en
Discuss". Es by-design. **Síntoma 2**: "activé notify_email pero el host no recibe nada" → falta
`host_selection=True` (sin él, los hosts nunca se notifican).

---

### FDG3 — Sin `host_selection`, los anfitriones NUNCA se notifican

La rama de notificación a hosts está guardada por
`if visitor.station_id.host_selection and visitor.host_ids:` (`frontdesk/models/frontdesk_visitor.py:84`, `FrontdeskVisitor._notify`).

**Síntoma**: visitas con host asignado pero el empleado no recibe email/SMS/Discuss. Causa:
`host_selection=False` en la estación. Hay que activarlo aunque el host venga de un planned visitor.

---

### FDG4 — `notify_email` requiere `work_email`; `notify_sms` requiere `work_phone`

`_notify_by_email` itera `host_ids` y omite hosts sin `work_email` (`frontdesk/models/frontdesk_visitor.py:115-116`, `FrontdeskVisitor._notify_by_email`).
`_notify_by_sms` omite hosts sin `work_phone` (l.133-134).

**Síntoma**: notificación silenciosamente no enviada a un host concreto. No hay error ni log: el
empleado simplemente no tiene email/teléfono de trabajo configurado en `hr.employee`.

---

### FDG5 — El domain de `mail_template_id`/`sms_template_id` apunta a `frontdesk.frontdesk`, pero las plantillas son de `frontdesk.visitor`

Los campos en la estación tienen `domain="[('model','=','frontdesk.frontdesk')]"`
(`frontdesk/models/frontdesk_frontdesk.py:37`, `Frontdesk.mail_template_id` y `frontdesk/models/frontdesk_frontdesk.py:44`, `Frontdesk.sms_template_id`), pero `frontdesk_mail_template`/`frontdesk_sms_template` están
definidas con `model_id` = `frontdesk.visitor` (`data/mail_template_data.xml`,
`data/sms_template_data.xml`), y el render se hace sobre el visitor (`self.ids`).

**Síntoma**: al crear una plantilla de email/SMS personalizada para frontdesk, si la creas sobre
modelo `frontdesk.frontdesk` (como sugiere el domain) el render fallará o saldrá vacío, porque el
código renderiza sobre `frontdesk.visitor`. Crea la plantilla custom sobre `frontdesk.visitor`.

---

### FDG6 — El QR móvil (self check-in) caduca a la hora exacta

`_verify_token` valida el tmp_code si `time_difference.total_seconds() <= 3600`
(`frontdesk/controllers/main.py:22`, `Frontdesk._verify_token`) y el QR se regenera cada 3600000 ms en el cliente
(`welcome_page/welcome_page.js`).

**Síntoma**: "escaneé el QR y sale 'QR Code Expired. Please rescan it.'". Causa típica: foto del
QR de hace más de 1h, o reloj del servidor/dispositivo desincronizado. El tmp_code se basa en
`tools.hmac` con la **fecha** (`_get_tmp_code`, `frontdesk/models/frontdesk_frontdesk.py:176`, `Frontdesk._get_tmp_code`), así que a cambio de
día también invalida.

---

### FDG7 — El kiosk cierra la sesión del usuario (logout) al abrirlo

`_get_additional_info` ejecuta `request.session.logout(keep_db=True)` siempre
(`frontdesk/controllers/main.py:10`, `Frontdesk._get_additional_info`).

**Síntoma**: abro `/kiosk/...` en la misma pestaña donde estaba logueado en backend y me quedo
deslogueado. By-design: el kiosk es anónimo/público. Usa `action_open_kiosk` con target self
sabiendo que perderás la sesión, o ábrelo en otra ventana/dispositivo.

---

### FDG8 — `prepare_visitor_data` con `drink_ids` NO crea ni hace check-in: solo añade bebidas

En el branch `if visitor:` con `kwargs.get('drink_ids')`, el controller hace
`write({'drink_ids': ...})` y devuelve `_notify_to_people()` SIN aplicar `vals` (que contenía
`state=checked_in`) (`frontdesk/controllers/main.py:88-91`, `Frontdesk.prepare_visitor_data`).

**Síntoma**: al pasar drinks a un visitor ya existente, no se actualiza el estado; el valor de
retorno es el de `_notify_to_people` (None), no `{'visitor_id': ...}`. Es intencional (el visitor
ya estaba checked-in), pero confunde si se llama el endpoint manualmente.

---

### FDG9 — `served` se fuerza a True en cada check-out aunque la bebida no se sirviera

`write` con `state='checked_out'` setea `vals['served'] = True` (`frontdesk/models/frontdesk_visitor.py:44-45`, `FrontdeskVisitor.write`).

**Síntoma**: el contador `drink_to_serve` baja al hacer check-out aunque nadie sirviera la bebida.
La analítica de bebidas servidas queda inflada: check-out == servida, por definición del código.

---

### FDG10 — Usuario `frontdesk_group_user` solo ve estaciones donde es `responsible_id`

Record rule `frontdesk_frontdesk_user_rule`: `[('responsible_ids', 'in', user.id)]`
(`security/security.xml`). Idéntico para visitors vía `station_id.responsible_ids`.

**Síntoma**: un recepcionista no ve una estación/visitas pese a estar en el grupo User. Hay que
añadirlo a `responsible_ids` de esa estación, o darle el grupo Administrator (ve todo).

---

### FDG11 — `company` del visitor es texto libre, no un `res.partner`

`company = fields.Char('Visitor Company')` (`frontdesk/models/frontdesk_visitor.py:18`, `class FrontdeskVisitor`). No hay relación con
`res.partner` ni con `res.company` (ese es `company_id`, la compañía Odoo).

**Síntoma**: no puedes filtrar/agrupar visitas por una empresa real ni enlazar a un partner CRM.
Es solo una cadena tecleada en el kiosk.

---

### FDG12 — No existe impresión de etiqueta (badge) en estándar

No hay report QWeb de etiqueta ni acción de impresión en frontdesk v18/v19. El "Reporting" es solo
analítica (pivot/graph) en `views/frontdesk_report_views.xml`.

**Síntoma**: "¿dónde imprimo la etiqueta del visitante?" → no existe; es desarrollo custom.

---

### FDG13 — El `public_user` se añade a la compañía de la estación en un `onchange` (no en write)

`_onchange_company_id` (`frontdesk/models/frontdesk_frontdesk.py:65-71`, `class Frontdesk`) añade `base.public_user` a
`company_ids` de la company seleccionada. Es un `@api.onchange`: solo corre en la UI form.

**Síntoma**: estaciones creadas por XML-RPC/import/script para una compañía nueva → el kiosk
público puede fallar al acceder porque `public_user` no tiene esa company. Crea la estación por la
UI o añade manualmente `public_user.company_ids`.

---

### FDG14 — Quick check-in solo lista visitas en ventana de ±45 minutos

`_get_planned_visitors` filtra `check_in` entre `now()-45min` y `now()+45min` y `state='planned'`
(`frontdesk/models/frontdesk_frontdesk.py:162-167`, `Frontdesk._get_planned_visitors`, `PLANNED_VISITOR_TIME=45`).

**Síntoma**: una visita planificada para más tarde (o muy temprano) no aparece en QuickCheckIn del
kiosk; la persona tiene que registrarse como walk-in. La lista se refresca cada 10 min en cliente
(`frontdesk/static/src/frontdesk.js:64`, `Frontdesk.updatePlannedVisitors`).

---

### FDG15 — Quick check-in filtra por el campo `check_in`, no por una fecha de cita planificada

`_get_planned_visitors` busca visitors con `state='planned'` **y** `check_in` dentro de ±45 min
(`frontdesk/models/frontdesk_frontdesk.py:162-167`, `Frontdesk._get_planned_visitors`). El `check_in` de un planned visitor es por tanto la hora
**prevista** de llegada (se rellena a mano al crear el planned), no la hora real (que se setea en el
write a `checked_in`, FDG1).

**Síntoma**: un planned visitor creado sin `check_in` (o con `check_in` en otro día) NUNCA aparece
en QuickCheckIn aunque sea hoy. Hay que poblar `check_in` con la hora prevista al planificar la
visita; si lo dejas vacío, el kiosk no lo encuentra.

---

### FDG16 — En `write` masivo, `self.drink_ids` se evalúa sobre TODO el recordset, no por registro

`write` hace `if self.drink_ids: self._notify_to_people()` (`frontdesk/models/frontdesk_visitor.py:41-42`, `FrontdeskVisitor.write`). `self`
es el recordset completo y `_notify` itera, pero la condición del drink mira la unión de drinks de
todos los visitors escritos.

**Síntoma**: un `write({'state':'checked_in'})` sobre varios visitors a la vez dispara
`_notify_to_people()` (aviso de bebidas) si **cualquiera** del lote tiene `drink_ids`, notificando
posiblemente bebidas de visitors que no las pidieron. Evita escrituras masivas de estado.

---

### FDG17 — `action_served` solo pone `served=True`; NO dispara el flujo de `write(state=...)`

`action_served` hace `self.served = True` directamente (`frontdesk/models/frontdesk_visitor.py:66-68`, `FrontdeskVisitor.action_served`), sin tocar
`state`. Solo el `write` con `state='checked_in'/'checked_out'` ejecuta `check_in/out`/`_notify`
(`frontdesk/models/frontdesk_visitor.py:37-46`, `FrontdeskVisitor.write`).

**Síntoma**: marcar "served" no cambia estado ni hora; y a la inversa, el check-out fuerza
`served=True` igualmente (FDG9). El botón "served" es puramente cosmético respecto al ciclo de vida.

---

### FDG18 — `duration` es `store=True` con `default=1.0` y solo se recalcula con AMBAS fechas

`duration = fields.Float(..., compute='_compute_duration', store=True, default=1.0)` y el compute
solo asigna si `check_in AND check_out` (`frontdesk/models/frontdesk_visitor.py:24`, `FrontdeskVisitor.duration` y `frontdesk/models/frontdesk_visitor.py:49-52`, `FrontdeskVisitor._compute_duration`).

**Síntoma**: un visitor planned o solo checked-in muestra `duration=1.0` (el default), no 0. La
analítica de duración media incluye ese 1.0 fantasma para visitas sin check-out. Filtra por
`state='checked_out'` antes de promediar.

---

### FDG19 — `host_ids` en el visitor NO está filtrado por la estación; el filtro company solo está en el kiosk

El campo `host_ids` tiene domain `[('user_id','!=',False)]` (`frontdesk/models/frontdesk_visitor.py:20`, `class FrontdeskVisitor`), sin filtro
de company. El filtro `company_id == station.company_id` solo se aplica en el endpoint del kiosk
`get_hosts` (`frontdesk/controllers/main.py:73-78`, `class Frontdesk`).

**Síntoma**: desde el backend puedes asignar como host a un empleado de OTRA company que la de la
estación; el kiosk nunca lo ofrecería. Importaciones/server actions pueden crear visitas con hosts
cross-company que rompen la notificación (work_email/phone de otra company).

---

### FDG20 — El `create` en v18 NO notifica; solo el `write` lo hace (asimetría create vs write)

Toda la lógica de notificación vive en `write` (`frontdesk/models/frontdesk_visitor.py:37-46`, `FrontdeskVisitor.write`). No hay
`create`/`_check_and_notify_visitor` en v18 (eso es v19, ver `v19_changes.md`).

**Síntoma**: crear un visitor con `state='checked_in'` de una sola vez (un solo `create`) NO
dispara `_notify()` ni rellena `check_in`. El kiosk evita esto creando primero y luego escribiendo
el estado. Importaciones que crean directamente en `checked_in` quedan sin notificar y sin
`check_in`. **En v19 cambia**: `create` llama `_check_and_notify_visitor`.

---

### FDG21 — `visitor_properties` se hereda de la estación: cambiar la definición rompe propiedades existentes

`visitor_properties = fields.Properties(definition='station_id.visitor_properties_definition')`
(`frontdesk/models/frontdesk_visitor.py:33`, `class FrontdeskVisitor`). La definición vive en la estación
(`frontdesk_frontdesk.py:visitor_properties_definition`).

**Síntoma**: editar/eliminar una property en la estación deja las visitas históricas con valores
huérfanos o invisibles (comportamiento estándar de Properties). No hay migración de datos al
cambiar la definición; las visitas viejas muestran solo las properties que aún existen en la
definición de su estación.

---

### FDG22 — `frontdesk.drink.notify_user_ids` es required: no puedes ofrecer una bebida sin "camarero"

`notify_user_ids = fields.Many2many('res.users', required=True)` (`frontdesk_drink.py`).

**Síntoma**: crear un drink (UI, import o data XML) sin al menos un usuario en `notify_user_ids`
falla. Y a la inversa: ese usuario recibe SIEMPRE un Discuss cuando alguien pide la bebida
(`_notify_to_people`, `frontdesk/models/frontdesk_visitor.py:93`, `FrontdeskVisitor._notify_to_people`), independientemente de los flags `notify_*` de la
estación (esos solo afectan a los hosts, FDG2).

---

### FDG23 — `prepare_visitor_data` (walk-in) NO tolera kwargs ausentes: `host_ids` y `drink_ids` deben venir siempre

En el branch de creación, el controller hace `[(4, host_id) for host_id in kwargs.get('host_ids')]`
(`controllers/main.py`, método `prepare_visitor_data`). `kwargs.get('host_ids')` sin default →
`None` si la clave falta → `TypeError: 'NoneType' is not iterable`. Igual con `drink_ids` en el
branch de update (`for drink_id in kwargs.get('drink_ids')`).

**Síntoma**: llamar el endpoint manualmente (test, integración, script) sin pasar `host_ids`
revienta el create. El kiosk OWL siempre manda la lista (vacía o no) en `createVisitor`
(`frontdesk/static/src/frontdesk.js:77`, `Frontdesk.createVisitor`), por eso no falla en uso normal; solo al invocarlo a mano. Pasa
SIEMPRE `host_ids: []` y `drink_ids: []` aunque estén vacíos.

---

### FDG24 — `get_hosts` exige el kwarg `name` (posicional en la firma del endpoint)

`def get_hosts(self, frontdesk_id, token, name)` (`controllers/main.py`) tiene `name` como
parámetro **obligatorio** y lo pasa posicional a `hr.employee.name_search(name, [...])`. No hay
default.

**Síntoma**: llamar `/frontdesk/<id>/<token>/get_hosts` sin `name` → error de argumento faltante
(no `not_found`). Además el filtro de host se aplica aquí (`[('user_id','!=',False),('company_id','=',
frontdesk.company_id.id)]`), reforzando FDG19: el cribado por company del anfitrión vive solo en
este endpoint del kiosk, no en el campo `host_ids` del modelo.

---

### FDG25 — `frontdesk.visitor` no tiene `_check_company`/`check_company`: `company_id` y `station_id.company_id` pueden divergir

El visitor define `company_id` (required, default `env.company`) y `station_id` (required) sin
ningún `check_company=True` ni `_check_company` (`models/frontdesk_visitor.py`). No hay constraint
que obligue a `company_id == station_id.company_id`.

**Síntoma**: por import/script puedes crear un visitor con `company_id` distinto de
`station_id.company_id`. La record rule de company filtra por `station_id.company_id` (no por el
`company_id` propio del visitor, `security/security.xml`), así que el visitor desaparece de la vista
del usuario aunque su `company_id` sea correcto, o aparece donde no esperas. Mantén ambos alineados
en cargas de datos.

---

### FDG26 — `active=True` por defecto pero no hay archivado en la UI estándar del visitor

`active = fields.Boolean(default=True)` en visitor y en drink y en station
(`frontdesk/models/frontdesk_visitor.py:14`, `class FrontdeskVisitor`, `frontdesk_drink.py`, `frontdesk/models/frontdesk_frontdesk.py:63`, `class Frontdesk`). El visitor se
"cancela" con `action_canceled` (state→`canceled`), que NO toca `active`.

**Síntoma**: una visita cancelada sigue `active=True` y aparece en listados sin filtro de estado;
el conteo del dashboard la excluye porque filtra por `state in (checked_in, planned)`
(`_compute_dashboard_data`, `frontdesk/models/frontdesk_frontdesk.py:76-79`, `Frontdesk._compute_dashboard_data`), pero las vistas/exports sin ese filtro
la incluyen. Cancelado ≠ archivado.
