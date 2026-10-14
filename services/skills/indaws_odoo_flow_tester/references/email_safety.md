# Seguridad de Emails — Odoo Flow Tester (API)

Este documento define los mecanismos para **prevenir el envio de emails reales** durante la ejecucion
de flujos de prueba en entornos de produccion. Es CRITICO aplicar estas medidas antes de cualquier
operacion que pueda disparar notificaciones, correos transaccionales o campanas automatizadas.

---

## Checklist Pre-Ejecucion

Antes de ejecutar cualquier flujo, verificar el estado del sistema de correo:

### 1. Servidores de correo saliente activos

```bash
search_records.sh ir.mail_server '[["active","=",true]]' '["id","name","smtp_host","smtp_port","active","sequence"]' 0
```
**Evaluacion:** Si hay servidores activos, el sistema PUEDE enviar emails. Aplicar alguna estrategia de bloqueo.

### 2. Cola de correos pendientes

```bash
search_records.sh mail.mail '[["state","=","outgoing"]]' '["id","subject","email_to","state","scheduled_date"]' 0
```
**Evaluacion:** Si hay correos en cola, el cron `ir_cron_mail_scheduler_action` puede enviarlos en cualquier momento.
Considerar cancelarlos o pausar el cron antes de iniciar el test.

### 3. Mensajes programados pendientes

```bash
search_records.sh mail.message.schedule '[]' '["id","mail_message_id","scheduled_datetime"]' 0
```
**Evaluacion:** Mensajes con notificaciones diferidas. El cron `ir_cron_send_scheduled_message` los procesa.

### 4. Mensajes planificados (mail.scheduled.message)

```bash
search_records.sh mail.scheduled.message '[]' '["id","subject","scheduled_date","body"]' 0
```
**Evaluacion:** Mensajes planificados por usuarios para envio futuro. El cron `ir_cron_post_scheduled_message` los publica.

### 5. Configuracion de dominio de alias (Odoo 18+)

En Odoo 18, la configuracion de catchall/bounce/default_from se gestiona mediante el modelo `mail.alias.domain`
(ya no mediante `ir.config_parameter`):

```bash
search_records.sh mail.alias.domain '[]' '["id","name","bounce_alias","catchall_alias","default_from","sequence"]' 0
```

**Parametros ICP legados** (verificar por compatibilidad):
```bash
search_records.sh ir.config_parameter '[["key","in",["mail.catchall.domain","mail.default.from","mail.bounce.alias","mail.catchall.domain.allowed","mail.default.from_filter"]]]' '["key","value"]' 0
```

### 6. Crons relacionados con correo

```bash
search_records.sh ir.cron '[["model_id.model","in",["mail.mail","mail.message.schedule","mail.scheduled.message","fetchmail.server","mail.push"]],["active","=",true]]' '["id","name","model_id","active","interval_number","interval_type","nextcall"]' 0
```
**Evaluacion:** Identificar crons activos que procesan cola de correo. Considerar desactivarlos temporalmente.

### 7. Cola de SMS pendientes

```bash
search_records.sh sms.sms '[["state","=","outgoing"]]' '["id","number","state","body"]' 0
```
**Evaluacion:** Si hay SMS en cola, el cron `ir_cron_sms_scheduler_action` puede enviarlos.

### 8. Cola de WhatsApp pendientes

```bash
search_records.sh whatsapp.message '[["state","=","outgoing"]]' '["id","mobile_number","state","body"]' 0
```
**Nota:** Solo si el modulo `whatsapp` esta instalado. Si falla con error de modelo, ignorar.

### 9. Web Push Notifications pendientes

```bash
search_records.sh mail.push '[]' '["id","create_date"]' 0
```
**Evaluacion:** El cron `ir_cron_web_push_notification` envia push notifications a navegadores. Aunque no son
emails, son comunicaciones externas que pueden confundir a usuarios durante el test.

---

## Catalogo Completo de Flags de Contexto

Odoo utiliza flags de contexto para controlar el comportamiento de `mail.thread` y sus modelos relacionados.
A continuacion se documenta CADA flag disponible, basado en el codigo fuente de Odoo 18.0.

### Flags principales de `mail.thread`

| Flag | Efecto | Valor durante test | Operaciones afectadas | Codigo fuente (mail_thread.py) |
|------|--------|-------------------|----------------------|-------------------------------|
| `tracking_disable=True` | Desactiva TODAS las features de mail.thread: auto-suscripcion, tracking, post, notificaciones | `True` | `create()`, `write()` | `create()` y `write()` retornan inmediatamente sin procesar mail features |
| `mail_create_nolog=True` | No genera el mensaje automatico "Documento creado" en el chatter | `True` | `create()` | Solo afecta `create()` — omite `_creation_message()` |
| `mail_notrack=True` | No genera mensajes de tracking de cambios de campos | `True` | `create()`, `write()` | Afecta `create()` y `write()` — omite `_message_track()` |
| `mail_create_nosubscribe=True` | No suscribe automaticamente al usuario creador como follower | `True` | `create()` | Solo en `create()` — omite `_message_subscribe()` del creador |
| `mail_notify_force_send=False` | No envia emails inmediatamente; los encola en `mail.mail` | `False` | `_notify_thread()`, `message_post()` | En `_notify_thread()` — si el nro de emails < `mail.mail.force.send.limit` (default 100), Odoo envia directo; este flag lo previene. **PELIGROSO si True** — fuerza envio inmediato |
| `mail_post_autofollow=False` | No suscribe automaticamente a los partners mencionados | `False` | `message_post()` | En `message_post()` — evita auto-follow de recipients |
| `mail_auto_subscribe_no_notify=True` | Suscribe followers automaticamente pero sin enviar la notificacion de suscripcion | `True` | `_message_auto_subscribe_notify()` | Impide que `_message_auto_subscribe_notify()` envie la notificacion a los nuevos followers auto-suscritos |
| `mail_notify_author=False` | No notifica al autor del mensaje (por defecto Odoo excluye al autor de las notificaciones) | `False` (default) | `_notify_thread()` | Si se pone `True`, el autor TAMBIEN recibe la notificacion — nunca activar en testing |

### Flags de modulos especificos

| Flag | Modulo | Efecto | Valor durante test | Operaciones afectadas |
|------|--------|--------|-------------------|----------------------|
| `no_reset_password=True` | `base` / `res.users` | No envia email de reseteo/bienvenida al crear usuarios | `True` | `create()` de `res.users` |
| `no_mail_to_attendees=True` | `calendar` | No envia emails a asistentes de eventos de calendario | `True` | `create()`, `write()` de `calendar.event` |
| `dont_notify=True` | `calendar` | No envia notificaciones de eventos (alarmas, cambios) | `True` | Operaciones de `calendar.event` |
| `loyalty_no_mail=True` | `loyalty` / `sale_loyalty` | No envia emails al crear/modificar tarjetas de fidelidad | `True` | `create()`, `write()` de `loyalty.card` |
| `action_no_send_mail=True` | `loyalty` / `pos_loyalty` | No envia email al crear cupones desde POS | `True` | Creacion de cupones POS |
| `mail_activity_automation_skip=True` | `mail` (mail_activity_mixin) | Salta la automatizacion de actividades (creacion, programacion, eliminacion automatica al cambiar de etapa) | `True` | Cambios de etapa, creacion de actividades |
| `sms_skip_msg_notification=True` | `sms` | Salta las notificaciones de mensaje al procesar SMS | `True` | Procesamiento de SMS |
| `calendar.block_mail` (ICP) | `calendar` | Parametro `ir.config_parameter` que bloquea TODOS los emails de calendario si su valor es truthy | Activar ICP | Todos los emails de calendario |

### Comportamiento detallado de `tracking_disable`

Cuando `tracking_disable=True` esta en el contexto, `mail.thread` omite completamente en `create()` y `write()`:
- Auto-suscripcion del usuario
- Tracking de cambios de campos
- Generacion de mensajes de chatter
- Envio de notificaciones
- Copy (`copy_data`) siempre usa `mail_notrack=True` internamente

Es el flag **mas seguro** porque bloquea el pipeline completo de notificaciones.

### Comportamiento detallado de `mail_notify_force_send`

En `_notify_thread()`, Odoo decide si enviar emails inmediatamente o encolarlos:
1. Lee el contexto `mail_notify_force_send` (default: valor del parametro `force_send` del metodo)
2. Si `force_send=True` Y el numero de emails es menor que `mail.mail.force.send.limit` (ICP, default 100), envia directo
3. Si no, encola en `mail.mail` con estado `outgoing`

Poner `mail_notify_force_send=False` fuerza el encolamiento, dando tiempo para interceptar antes del envio real.

---

## Estrategia A — Flags de Contexto (Preferida, No Invasiva)

La forma menos invasiva de prevenir emails es usar flags de contexto en las llamadas API.
Odoo respeta estos flags en el modelo `mail.thread` (del que heredan casi todos los modelos de negocio).

### Contexto recomendado por defecto

Para la mayoria de flujos de test, usar este contexto combinado:

```json
{
  "tracking_disable": true,
  "mail_create_nolog": true,
  "mail_create_nosubscribe": true,
  "mail_notify_force_send": false,
  "mail_activity_automation_skip": true
}
```

### Contexto extendido para modulos especificos

Si el flujo involucra calendario, loyalty o usuarios:

```json
{
  "tracking_disable": true,
  "mail_create_nolog": true,
  "mail_create_nosubscribe": true,
  "mail_notify_force_send": false,
  "mail_activity_automation_skip": true,
  "no_reset_password": true,
  "no_mail_to_attendees": true,
  "dont_notify": true,
  "loyalty_no_mail": true
}
```

### Aplicacion por modelo/flujo

#### Operaciones de creacion (cualquier modelo mail.thread)
```bash
create_record.sh sale.order '{"partner_id": 1, "order_line": [[0, 0, {"product_id": 5, "product_uom_qty": 1}]]}' \
  '{"context": {"tracking_disable": true, "mail_create_nolog": true, "mail_create_nosubscribe": true, "mail_activity_automation_skip": true}}'
```

#### Operaciones de escritura/actualizacion
```bash
update_record.sh sale.order '[42]' '{"note": "Test"}' \
  '{"context": {"tracking_disable": true, "mail_activity_automation_skip": true}}'
```

#### Confirmaciones y cambios de estado
```bash
execute_method.sh sale.order action_confirm '[[42]]' \
  '{"context": {"tracking_disable": true, "mail_notify_force_send": false, "mail_activity_automation_skip": true}}'
```

#### Creacion de usuarios (evitar email de bienvenida)
```bash
create_record.sh res.users '{"name": "Test User", "login": "test@test.com"}' \
  '{"context": {"no_reset_password": true, "tracking_disable": true, "mail_create_nolog": true}}'
```

#### Publicacion de facturas
```bash
execute_method.sh account.move action_post '[[15]]' \
  '{"context": {"tracking_disable": true, "mail_notify_force_send": false, "mail_activity_automation_skip": true}}'
```

#### Validacion de albaranes
```bash
execute_method.sh stock.picking button_validate '[[78]]' \
  '{"context": {"tracking_disable": true, "mail_notify_force_send": false, "skip_backorder": true, "picking_ids_not_to_backorder": [78]}}'
```

#### Eventos de calendario
```bash
create_record.sh calendar.event '{"name": "Test Meeting", "start": "2026-03-10 10:00:00", "stop": "2026-03-10 11:00:00"}' \
  '{"context": {"no_mail_to_attendees": true, "dont_notify": true, "tracking_disable": true}}'
```

### Limitaciones de la Estrategia A

- **NO bloquea emails enviados por metodos que ignoran el contexto** (ej: `mail.template.send_mail()` llamado internamente por wizards)
- **NO bloquea emails de crons** que procesan la cola `mail.mail`
- **NO bloquea emails de `base.automation`** que tienen su propia logica de envio
- **NO bloquea SMS** enviados via `sms.sms` (requiere desactivar crons de SMS separadamente)
- **NO bloquea WhatsApp** mensajes (requiere desactivar cron de WhatsApp)
- **NO bloquea Web Push Notifications** (requiere desactivar cron de push)
- Algunos metodos (como `action_send_mail` de wizards) pueden limpiar o ignorar el contexto

**Si el flujo incluye acciones que disparan templates de email, SMS, WhatsApp o automatizaciones, combinar con Estrategia B.**

---

## Estrategia B — Desactivar Servidores de Correo (Agresiva)

Desactivar todos los registros de `ir.mail_server` impide que Odoo pueda conectar a ningun servidor SMTP.
Cualquier intento de envio fallara con excepcion (que Odoo gestiona internamente marcando el mail como `exception`).

### Paso 1: Capturar estado original

```bash
search_records.sh ir.mail_server '[]' '["id","name","smtp_host","active"]' 0
```

Guardar el resultado como referencia para restauracion:
```json
{
  "mail_servers_backup": [
    {"id": 1, "name": "Servidor principal", "smtp_host": "smtp.ejemplo.com", "active": true},
    {"id": 2, "name": "Servidor secundario", "smtp_host": "smtp2.ejemplo.com", "active": true}
  ]
}
```

### Paso 2: Desactivar todos los servidores

```bash
update_record.sh ir.mail_server '[1, 2]' '{"active": false}'
```

### Paso 3: Verificar desactivacion

```bash
search_records.sh ir.mail_server '[["active","=",true]]' '["id","name"]' 0
```
**Resultado esperado:** Lista vacia (ningun servidor activo).

### Paso 4: Ejecutar flujos de test

Con los servidores desactivados, ejecutar normalmente. Los emails generados quedaran en `mail.mail` con estado `exception` o `outgoing` (sin poder enviarse).

### Paso 5: Restaurar servidores

```bash
update_record.sh ir.mail_server '[1]' '{"active": true}'
update_record.sh ir.mail_server '[2]' '{"active": true}'
```

### Paso 6: Verificar restauracion

```bash
search_records.sh ir.mail_server '[["active","=",true]]' '["id","name","smtp_host"]' 0
```

### Ventajas y riesgos

| Ventaja | Riesgo |
|---------|--------|
| Bloqueo total: ningun email puede salir | Si el test falla a mitad y no se restaura, el sistema queda sin correo |
| Funciona incluso con templates y automatizaciones | Otros procesos del sistema que necesiten enviar email tambien fallaran |
| No requiere modificar contextos en cada llamada | Los emails fallidos quedan como `exception` en mail.mail (ruido) |

---

## Estrategia C — Redireccion a Dominio de Test (Parcial)

Cambiar el dominio de alias a un dominio de test para que los emails no lleguen a destinatarios reales.
Esta estrategia es util cuando se necesita verificar que el sistema GENERA emails correctamente pero sin que lleguen a produccion.

### Paso 1: Capturar configuracion original

**Odoo 18+ (mail.alias.domain):**
```bash
search_records.sh mail.alias.domain '[]' '["id","name","bounce_alias","catchall_alias","default_from"]' 0
```

**ICP legados:**
```bash
search_records.sh ir.config_parameter '[["key","in",["mail.catchall.domain","mail.default.from","mail.bounce.alias"]]]' '["id","key","value"]' 0
```

Guardar como backup:
```json
{
  "alias_domain_backup": {
    "id": 1,
    "name": "miempresa.com",
    "bounce_alias": "bounce",
    "catchall_alias": "catchall",
    "default_from": "notifications"
  },
  "icp_backup": [
    {"id": 10, "key": "mail.catchall.domain", "value": "miempresa.com"}
  ]
}
```

### Paso 2: Cambiar a dominio de test

```bash
update_record.sh mail.alias.domain '[1]' '{"name": "test.invalid"}'
execute_method.sh ir.config_parameter set_param '["mail.catchall.domain", "test.invalid"]' '{}'
```

**Nota:** El dominio `.invalid` (RFC 2606) garantiza que ningun servidor de correo aceptara los mensajes.

### Paso 3: Ejecutar flujos

Los emails se generaran con direcciones `@test.invalid` que nunca seran entregadas.

### Paso 4: Restaurar dominio original

```bash
update_record.sh mail.alias.domain '[1]' '{"name": "miempresa.com"}'
execute_method.sh ir.config_parameter set_param '["mail.catchall.domain", "miempresa.com"]' '{}'
```

### Limitaciones

- **NO impide el envio SMTP real** si hay servidores activos: el email se enviara, solo que el dominio del remitente/reply-to sera invalido
- Los emails destinados directamente a direcciones externas (campo `email_to` explicito) SI se enviarian
- Solo protege los emails que usan catchall/bounce/default_from del sistema

**Recomendacion:** Combinar con Estrategia B para bloqueo completo.

---

## Estrategia D — Email Sandbox (Catchall a Direccion de Test)

Crear una configuracion de catchall que redirija todo el correo entrante/saliente a una direccion
de test controlada. Util cuando se necesita verificar que los emails se generan Y se envian
correctamente, pero sin impactar a destinatarios reales.

### Configuracion

1. **Cambiar el alias domain a un dominio de test controlado:**
```bash
# Capturar config original
search_records.sh mail.alias.domain '[]' '["id","name","bounce_alias","catchall_alias","default_from"]' 0

# Cambiar a dominio sandbox
update_record.sh mail.alias.domain '[1]' '{"name": "sandbox.test.invalid", "catchall_alias": "catchall-test", "default_from": "test-noreply"}'
```

2. **Configurar todos los templates para usar remitente sandbox:**
No es necesario modificar templates. Al cambiar el alias domain, todos los emails generados
usaran automaticamente `@sandbox.test.invalid` como dominio.

3. **Alternativa con direccion real de test:**
Si se dispone de un buzon de test real (ej: `test-inbox@empresa.com`), se puede configurar
como catchall para recibir y verificar todos los emails del test:
```bash
update_record.sh mail.alias.domain '[1]' '{"catchall_alias": "test-inbox"}'
```

### Ventajas
- Permite verificar la generacion completa de emails (contenido, adjuntos, templates)
- Los emails quedan disponibles para inspeccion en el buzon sandbox
- No genera errores de tipo `exception` en `mail.mail`

### Limitaciones
- Requiere un buzon de test real si se quiere verificar la entrega
- Con `.invalid` los emails nunca llegan (similar a Estrategia C)
- No protege emails con `email_to` explicito a direcciones externas

---

## Estrategia E — Mock SMTP (MailHog/MailPit)

Configurar un servidor SMTP temporal en la maquina local que capture todos los emails sin
enviarlos a destinatarios reales. Ideal para entornos de desarrollo y testing donde se
necesita inspeccionar el contenido de los emails.

### Requisitos
- MailHog (`github.com/mailhog/MailHog`) o MailPit (`github.com/axllent/mailpit`)
- Acceso para crear/modificar `ir.mail_server` en Odoo

### Paso 1: Iniciar MailPit/MailHog

```bash
# MailPit (recomendado, mas moderno)
mailpit --smtp 0.0.0.0:1025 --listen 0.0.0.0:8025

# MailHog (alternativa)
mailhog -smtp-bind-addr 0.0.0.0:1025 -ui-bind-addr 0.0.0.0:8025
```

La UI web estara disponible en `http://localhost:8025` para inspeccionar emails capturados.

### Paso 2: Capturar configuracion original de mail servers

```bash
search_records.sh ir.mail_server '[]' '["id","name","smtp_host","smtp_port","active","smtp_encryption","smtp_user","smtp_pass"]' 0
```

### Paso 3: Crear servidor SMTP temporal apuntando a localhost

**Opcion A — Modificar servidor existente (reversible via snapshot):**
```bash
# Guardar config original en snapshot
# Modificar el servidor principal para apuntar a MailPit
update_record.sh ir.mail_server '[1]' '{"smtp_host": "localhost", "smtp_port": 1025, "smtp_encryption": "none", "smtp_user": false, "smtp_pass": false}'
```

**Opcion B — Crear servidor temporal nuevo y desactivar los demas:**
```bash
# Desactivar servidores existentes
update_record.sh ir.mail_server '[1, 2]' '{"active": false}'

# Crear servidor temporal
create_record.sh ir.mail_server '{"name": "[TEST] MailPit Local", "smtp_host": "localhost", "smtp_port": 1025, "smtp_encryption": "none", "sequence": 1}' '{}'
```

### Paso 4: Ejecutar flujos de test

Todos los emails se enviaran al servidor MailPit local. Se pueden inspeccionar en `http://localhost:8025`.

### Paso 5: Restaurar configuracion original

**Si se uso Opcion A:**
```bash
# Restaurar servidor original desde snapshot
update_record.sh ir.mail_server '[1]' '{"smtp_host": "<original_host>", "smtp_port": <original_port>, "smtp_encryption": "<original_enc>", "smtp_user": "<original_user>", "smtp_pass": "<original_pass>"}'
```

**Si se uso Opcion B:**
```bash
# Eliminar servidor temporal
search_records.sh ir.mail_server '[["name","ilike","[TEST] MailPit"]]' '["id"]' 0
delete_record.sh ir.mail_server '[<temp_id>]'

# Reactivar servidores originales
update_record.sh ir.mail_server '[1, 2]' '{"active": true}'
```

### Ventajas y riesgos

| Ventaja | Riesgo |
|---------|--------|
| Los emails se envian completamente (SMTP real) | Requiere que MailPit/MailHog este corriendo en la maquina |
| Se puede inspeccionar contenido, HTML, adjuntos | Si MailPit no esta corriendo, los emails fallan |
| Templates y automatizaciones funcionan normalmente | Requiere acceso de red desde Odoo a localhost:1025 |
| No genera registros `exception` en mail.mail | En produccion, la maquina de Odoo puede no ser accesible |

---

## Tabla de Modelos que Envian Emails Automaticamente

Modelos de Odoo que disparan envio de emails de forma automatica al ejecutar ciertos metodos.
Es CRITICO conocerlos para prevenir envios durante tests.

| Modelo | Metodo/Trigger | Template (xml_id) | Como prevenir |
|--------|---------------|-------------------|---------------|
| `sale.order` | `action_confirm` | `sale.email_template_edi_sale` | Contexto: `mail_notify_force_send=False`, `tracking_disable=True` |
| `sale.order` | `action_quotation_send` (wizard) | `sale.email_template_edi_sale` | NO ejecutar este metodo durante tests |
| `sale.order` | `message_post` (override) | Usa `mail_post_autofollow` del contexto | Contexto: `mail_post_autofollow=False`, `tracking_disable=True` |
| `account.move` | `action_send_and_print` (wizard) | `account.email_template_edi_invoice` | NO ejecutar send — dejar facturas en draft o usar solo `action_post` |
| `purchase.order` | `button_confirm` | `purchase.email_template_edi_purchase` | Contexto: `mail_notify_force_send=False`, `tracking_disable=True` |
| `purchase.order` | `action_rfq_send` (wizard) | `purchase.email_template_edi_purchase_done` | NO ejecutar este metodo durante tests |
| `hr.leave` | `action_approve` | `hr_holidays.mail_template_*` | Contexto: `tracking_disable=True`, `no_mail_to_attendees=True` |
| `hr.leave` | `action_refuse` | `hr_holidays.mail_template_*` | Contexto: `tracking_disable=True` |
| `helpdesk.ticket` | `create` | `helpdesk.email_template_*` | Contexto: `tracking_disable=True`, `mail_create_nolog=True` |
| `helpdesk.ticket` | `write` (cambio de stage) | Automatizaciones del helpdesk | Desactivar automatizaciones de helpdesk |
| `crm.lead` | `action_set_won` / stage changes | Templates de CRM | Contexto: `tracking_disable=True` |
| `project.task` | `write` (cambio de stage) | Notificaciones a followers | Contexto: `tracking_disable=True` |
| `res.users` | `create` | `base.set_password_email` | Contexto: `no_reset_password=True` |
| `account.payment` | `action_post` | Notificacion por chatter | Contexto: `tracking_disable=True` |
| `stock.picking` | `button_validate` | Notificacion por chatter | Contexto: `tracking_disable=True` |
| `mrp.production` | `button_mark_done` | Notificacion por chatter | Contexto: `tracking_disable=True` |
| `approval.request` | `action_confirm` / `action_approve` | `approvals.approval_template_*` | Contexto: `tracking_disable=True` |
| `planning.slot` | `action_send` | `planning.mail_template_*` | NO ejecutar `action_send` |
| `mailing.mailing` | `action_send_mail` | (contenido propio del mailing) | NO ejecutar — NUNCA enviar mailings en test |
| `social.post` | `action_post` | (publica en redes sociales) | NO ejecutar — desactivar social accounts |
| `digest.digest` | `action_send` (o cron) | `digest.digest_mail_main` | Desactivar digest → `state='deactivated'` |
| `loyalty.card` | `create` / `write(points)` | Template de loyalty | Contexto: `loyalty_no_mail=True`, `tracking_disable=True` |
| `calendar.event` | `create` / `write` | Emails a asistentes | Contexto: `no_mail_to_attendees=True`, `dont_notify=True` |
| `whatsapp.message` | `_send` / `_send_cron` | (mensaje WhatsApp) | Desactivar cron `ir_cron_send_whatsapp_queue` |

---

## Deteccion de Emails Automatizados Peligrosos

Ademas de los emails generados por el flujo de test, existen mecanismos automaticos que pueden
disparar emails de forma independiente. Detectarlos ANTES de ejecutar el test.

### 1. Templates de correo con auto-envio

```bash
search_records.sh mail.template '[]' '["id","name","model_id","auto_delete","email_from","email_to","partner_to","mail_server_id"]' 0
```
**Atencion a:** Templates vinculadas a modelos que se tocaran en el flujo. No se auto-envian por si solas,
pero pueden ser invocadas por `base.automation`, wizards de envio o metodos internos.

### 2. Automatizaciones con acciones de correo

```bash
search_records.sh base.automation '[["active","=",true],["action_server_ids.state","=","email"]]' '["id","name","model_id","trigger","filter_domain"]' 0
```
Si no funciona por la busqueda relacionada, buscar en dos pasos:

```bash
# Paso 1: Buscar acciones de servidor tipo email, SMS o webhook (saliente)
search_records.sh ir.actions.server '[["state","in",["email","sms","mail_post","webhook"]]]' '["id","name","model_id","template_id","state"]' 0

# Paso 2: Buscar automatizaciones que usen esas acciones
search_records.sh base.automation '[["active","=",true]]' '["id","name","model_id","trigger","action_server_ids","filter_domain"]' 0
```
**Cruzar resultados:** Si una automatizacion contiene un `action_server_id` de tipo email/sms/webhook y su modelo/trigger
coincide con el flujo de test, puede disparar comunicaciones automaticas.

### 3. Templates de correo vinculadas a automatizaciones

Las automatizaciones (`base.automation`) pueden ejecutar acciones de servidor (`ir.actions.server`) que a su vez
referencian `mail.template` via el campo `template_id`. Detectar esta cadena:

```bash
# Acciones servidor con template vinculado
search_records.sh ir.actions.server '[["state","in",["email","sms"]],["template_id","!=",false]]' '["id","name","state","template_id","model_id"]' 0
```

### 4. Acciones planificadas (crons) relacionadas con comunicaciones

```bash
search_records.sh ir.cron '[["active","=",true]]' '["id","name","model_id","code","interval_number","interval_type","nextcall"]' 0
```

**Crons criticos a vigilar (Odoo 18.0):**

| Cron (xml_id) | Modelo | Funcion | Riesgo |
|---------------|--------|---------|--------|
| `mail.ir_cron_mail_scheduler_action` | `mail.mail` | `process_email_queue(batch_size=1000)` | Envia todos los `mail.mail` en estado `outgoing` |
| `mail.ir_cron_send_scheduled_message` | `mail.message.schedule` | `_send_notifications_cron` | Envia notificaciones programadas |
| `mail.ir_cron_post_scheduled_message` | `mail.scheduled.message` | `_post_messages_cron` | Publica mensajes planificados |
| `mail.ir_cron_mail_gateway_action` | `fetchmail.server` | `_fetch_mails` | Recibe y procesa correos entrantes |
| `mail.ir_cron_web_push_notification` | `mail.push` | `_push_notification_to_endpoint` | Envia push notifications a navegadores |
| `sms.ir_cron_sms_scheduler_action` | `sms.sms` | `_process_queue` | Envia todos los SMS en estado `outgoing` via IAP |
| `whatsapp.ir_cron_send_whatsapp_queue` | `whatsapp.message` | `_send_cron` | Envia mensajes WhatsApp pendientes |
| `digest.ir_cron_digest_scheduler_action` | `digest.digest` | `_cron_send_digest_email` | Envia digest emails periodicos |
| `snailmail.snailmail_print` | `snailmail.letter` | `_snailmail_cron` | Envia correo postal fisico via IAP |

**Estrategia de proteccion para crons:**
```bash
# Buscar todos los crons de comunicaciones activos
search_records.sh ir.cron '[["active","=",true],["model_id.model","in",["mail.mail","mail.message.schedule","mail.scheduled.message","fetchmail.server","mail.push","sms.sms","whatsapp.message","digest.digest","snailmail.letter"]]]' '["id","name","model_id","active"]' 0

# Desactivar cada uno
update_record.sh ir.cron '[<id>]' '{"active": false}'
```

### 5. Actividades planificadas que generan notificaciones

Las actividades (`mail.activity`) pueden generar notificaciones email cuando:
- Se asignan a un usuario (`user_id`)
- Se vencen (deadline passed)
- Se completan (`action_feedback`)

Para prevenir notificaciones de actividades:
```bash
# Usar mail_activity_automation_skip en el contexto
execute_method.sh sale.order action_confirm '[[42]]' \
  '{"context": {"tracking_disable": true, "mail_activity_automation_skip": true}}'
```

### 6. Campanas de marketing (si el modulo esta instalado)

```bash
# Marketing Automation
search_records.sh marketing.campaign '[["state","=","running"]]' '["id","name","model_id"]' 0

# Mass Mailing
search_records.sh mailing.mailing '[["state","=","sending"]]' '["id","name","subject","mailing_model_real"]' 0
```
**Nota:** Estos modelos solo existen si los modulos `marketing_automation` y `mass_mailing` estan instalados.
Si la busqueda falla con error de modelo no encontrado, ignorar.

### 7. Webhooks salientes (automatizaciones que envian datos afuera)

```bash
# Acciones de servidor tipo webhook (envian POST a URL externa)
search_records.sh ir.actions.server '[["state","=","webhook"]]' '["id","name","model_id","webhook_url"]' 0

# Automatizaciones que disparan webhooks salientes
search_records.sh base.automation '[["active","=",true]]' '["id","name","model_id","trigger","action_server_ids"]' 0
```
**Cuidado:** Las acciones de tipo `webhook` en `ir.actions.server` (Odoo 18) envian POST requests
a una URL externa (`webhook_url`) con payload de datos del registro. Si una automatizacion
con trigger `on_create`/`on_write` ejecuta una accion webhook, se enviaran datos a sistemas
externos durante el test.

### 8. Automatizaciones trigger `on_create`/`on_write` en modelos del flujo

```bash
search_records.sh base.automation '[["active","=",true],["trigger","in",["on_create","on_write","on_create_or_write"]]]' '["id","name","model_id","trigger","action_server_ids"]' 0
```
**Cuidado con:** Automatizaciones trigger `on_create` en modelos como `sale.order`, `account.move`, etc.
que pueden ejecutar acciones de email al detectar nuevos registros.

---

## Seguridad SMS

### Como prevenir envio de SMS durante tests

Los SMS se envian via IAP (servicio `sms`) usando el modelo `sms.sms`. El flujo es:
1. Un metodo crea un registro `sms.sms` con `state='outgoing'`
2. El cron `ir_cron_sms_scheduler_action` (o llamada directa a `send()`) procesa la cola
3. `_send_with_api()` llama al servicio IAP de SMS que consume creditos reales

**Estrategia de prevencion (en orden de prioridad):**

1. **Desactivar cron de SMS:**
```bash
search_records.sh ir.cron '[["active","=",true],["model_id.model","=","sms.sms"]]' '["id","name","active"]' 0
update_record.sh ir.cron '[<sms_cron_id>]' '{"active": false}'
```

2. **Desactivar automatizaciones que envian SMS:**
```bash
search_records.sh ir.actions.server '[["state","=","sms"]]' '["id","name","model_id","sms_template_id"]' 0
search_records.sh base.automation '[["active","=",true]]' '["id","name","trigger","action_server_ids"]' 0
```

3. **Verificar SMS templates vinculados a modelos del flujo:**
```bash
search_records.sh sms.template '[]' '["id","name","model_id","body"]' 0
```

4. **Si la base de datos esta neutralizada**, los tokens IAP tienen sufijo `+disabled` y las llamadas
   IAP fallan silenciosamente — no se requiere accion adicional.

---

## Seguridad WhatsApp

### Como prevenir envio de WhatsApp durante tests

WhatsApp en Odoo 18 Enterprise usa el modelo `whatsapp.message` con integracion a la API de WhatsApp Business.

**Deteccion:**
```bash
# Verificar si WhatsApp esta instalado
search_records.sh ir.module.module '[["name","=","whatsapp"],["state","=","installed"]]' '["name","state"]' 0

# Cron de WhatsApp activo
search_records.sh ir.cron '[["active","=",true],["model_id.model","=","whatsapp.message"]]' '["id","name","active"]' 0

# Cuentas de WhatsApp Business configuradas
search_records.sh whatsapp.account '[]' '["id","name","app_uid","active"]' 0
```

**Prevencion:**
```bash
# Desactivar cron de envio de WhatsApp
update_record.sh ir.cron '[<whatsapp_cron_id>]' '{"active": false}'

# Desactivar automatizaciones que envian WhatsApp
search_records.sh base.automation '[["active","=",true],["action_server_ids.state","=","whatsapp"]]' '["id","name"]' 0
```

**Modulos dependientes de WhatsApp (todos Enterprise):**
- `whatsapp_sale` — Envio de confirmaciones de venta
- `whatsapp_account` — Envio de facturas
- `whatsapp_delivery` — Notificaciones de envio
- `whatsapp_event` — Notificaciones de eventos
- `whatsapp_pos` — Envio de tickets POS
- `whatsapp_payment` — Solicitudes de pago
- `whatsapp_website_sale` — Notificaciones de ecommerce

---

## Seguridad Web Push Notifications

### Como prevenir push notifications durante tests

Odoo 18 incluye Web Push Notifications (VAPID/JWT) que envian notificaciones a navegadores suscritos.

**Deteccion:**
```bash
# Cron de push notifications
search_records.sh ir.cron '[["active","=",true],["model_id.model","=","mail.push"]]' '["id","name","active"]' 0

# Dispositivos registrados
search_records.sh mail.push.device '[]' '["id","partner_id"]' 0
```

**Prevencion:**
```bash
# Desactivar cron de push
update_record.sh ir.cron '[<push_cron_id>]' '{"active": false}'
```

**Nota:** En bases de datos neutralizadas, las claves VAPID se eliminan automaticamente (ver `mail/data/neutralize.sql`),
lo que impide el envio de push notifications.

---

## Limpieza Post-Test

Despues de ejecutar flujos de test, eliminar los registros de correo generados para no dejar residuos.

### 1. Eliminar mail.mail generados durante el test

```bash
# Buscar mails creados en el rango de tiempo del test
search_records.sh mail.mail '[["create_date",">=","2024-01-15 10:00:00"],["create_date","<=","2024-01-15 11:00:00"]]' '["id","subject","state","email_to"]' 0

# Cancelar los que esten en outgoing (para poder borrarlos)
update_record.sh mail.mail '[<id>]' '{"state": "cancel"}'
delete_record.sh mail.mail '[<id>]'
```

### 2. Eliminar mail.message generados durante el test

```bash
# Buscar mensajes creados durante el test
search_records.sh mail.message '[["create_date",">=","2024-01-15 10:00:00"],["create_date","<=","2024-01-15 11:00:00"],["message_type","in",["notification","email","email_outgoing","auto_comment"]]]' '["id","subject","model","res_id","message_type"]' 0

# Eliminar cada mensaje
delete_record.sh mail.message '[<id>]'
```

**Precaucion:** No eliminar mensajes de tipo `comment` que puedan ser mensajes legitimos de usuarios.

### 3. Eliminar notificaciones

```bash
search_records.sh mail.notification '[["create_date",">=","2024-01-15 10:00:00"],["create_date","<=","2024-01-15 11:00:00"]]' '["id","mail_mail_id","notification_status"]' 0
delete_record.sh mail.notification '[<ids>]'
```

### 4. Eliminar seguidores de test

Si se crearon registros de test que generaron suscripciones automaticas:

```bash
search_records.sh mail.followers '[["res_model","=","sale.order"],["res_id","in",[42,43,44]]]' '["id","partner_id"]' 0
delete_record.sh mail.followers '[<ids>]'
```

### 5. Eliminar SMS generados durante el test

```bash
search_records.sh sms.sms '[["create_date",">=","2024-01-15 10:00:00"],["create_date","<=","2024-01-15 11:00:00"]]' '["id","number","state","body"]' 0
update_record.sh sms.sms '[<ids>]' '{"state": "canceled"}'
delete_record.sh sms.sms '[<ids>]'
```

### 6. Eliminar WhatsApp messages generados durante el test

```bash
search_records.sh whatsapp.message '[["create_date",">=","2024-01-15 10:00:00"],["create_date","<=","2024-01-15 11:00:00"]]' '["id","state","mobile_number"]' 0
delete_record.sh whatsapp.message '[<ids>]'
```

### 7. Eliminar mail.mail con prefijo de test

Si se uso prefijo `[TEST-YYYYMMDD-HHMM]` en los registros, buscar emails relacionados:

```bash
# Buscar mail.mail cuyo subject o body contenga el prefijo de test
search_records.sh mail.mail '[["subject","ilike","[TEST-"]]' '["id","subject","state"]' 0

# Cancelar y eliminar
update_record.sh mail.mail '[<ids>]' '{"state": "cancel"}'
delete_record.sh mail.mail '[<ids>]'
```

### 8. Eliminar mail.message con prefijo de test

```bash
search_records.sh mail.message '[["subject","ilike","[TEST-"]]' '["id","subject","model","res_id"]' 0
delete_record.sh mail.message '[<ids>]'

# Tambien buscar en body (para mensajes sin subject)
search_records.sh mail.message '[["body","ilike","[TEST-"]]' '["id","subject","model","res_id"]' 0
delete_record.sh mail.message '[<ids>]'
```

### 9. Eliminar followers anadidos durante el test

```bash
# Buscar followers de registros creados durante el test (por rango de IDs)
# Para cada modelo y rango de IDs de test:
search_records.sh mail.followers '[["res_model","=","sale.order"],["res_id",">=",<min_test_id>],["res_id","<=",<max_test_id>]]' '["id","partner_id","res_id"]' 0
delete_record.sh mail.followers '[<ids>]'
```

### 10. Restaurar servidores de correo (si se aplico Estrategia B/D/E)

Ver seccion de restauracion en la estrategia correspondiente.

### 11. Restaurar configuracion de dominio (si se aplico Estrategia C/D)

Ver seccion de restauracion en la estrategia correspondiente.

### 12. Restaurar crons de comunicaciones

```bash
# Reactivar todos los crons que se desactivaron
update_record.sh ir.cron '[<ids>]' '{"active": true}'
```

### 13. Verificacion final

```bash
# Confirmar que no quedan mails pendientes del test
search_records.sh mail.mail '[["state","=","outgoing"]]' '["id","subject"]' 0

# Confirmar que no quedan SMS pendientes del test
search_records.sh sms.sms '[["state","=","outgoing"]]' '["id","number"]' 0

# Confirmar servidores restaurados
search_records.sh ir.mail_server '[["active","=",true]]' '["id","name"]' 0

# Confirmar dominio de alias restaurado
search_records.sh mail.alias.domain '[]' '["id","name"]' 0

# Confirmar crons restaurados
search_records.sh ir.cron '[["active","=",true],["model_id.model","in",["mail.mail","sms.sms","whatsapp.message","mail.push","digest.digest","snailmail.letter"]]]' '["id","name","active"]' 0

# Confirmar que no quedan mail.mail con prefijo de test
search_records.sh mail.mail '[["subject","ilike","[TEST-"]]' '["id","subject"]' 0
```

---

## Matriz de Decision — Que Estrategia Usar

| Escenario | Estrategia Recomendada | Justificacion |
|-----------|----------------------|---------------|
| Flujo simple (crear + confirmar SO) | A (contexto) | Bajo riesgo, flags suficientes |
| Flujo con facturacion y pagos | A + B | La facturacion puede disparar templates |
| Flujo con automatizaciones activas | B (desactivar servidores) + desactivar crons SMS/WA | Las automatizaciones ignoran el contexto |
| Flujo con campanas de marketing | B + desactivar crons | Campanas tienen su propio pipeline |
| Flujo con modulos WhatsApp/SMS | A + B + desactivar crons SMS/WA | Canales multiples de comunicacion |
| Test que necesita verificar generacion de emails | E (Mock SMTP) | Captura emails completos sin enviar |
| Test que necesita verificar formato de emails | D (sandbox) + E (mock) | Redirigir + capturar |
| Primer test en instancia desconocida | B + deteccion completa + desactivar TODOS crons comunicaciones | Maximo nivel de precaucion |
| Entorno de staging con MailPit disponible | E (Mock SMTP) | Inspeccion completa de emails |
| Produccion sin acceso a MailPit | A + B | Bloqueo total sin dependencias |
| Base de datos neutralizada | A (contexto basico) | IAP desactivado, tokens disabled, fetchmail off |

---

## Verificacion Post-Paso de Emails

Despues de cada paso del flujo de test, verificar que no se generaron emails inesperados.
Esto actua como una red de seguridad adicional a los flags de contexto y la desactivacion
de servidores/crons.

### Verificacion por timestamp

Registrar el timestamp antes de ejecutar cada paso y luego consultar `mail.mail`:

```bash
# Registrar timestamp ANTES del paso (formato UTC)
TEST_START="2026-03-09 10:30:00"

# Despues del paso, verificar emails generados
search_records.sh mail.mail '[["create_date",">=","'"$TEST_START"'"]]' '["id","subject","email_to","state","create_date"]' 0
```

### Assertion template — verificar 0 emails

```bash
# Verificar que NO se crearon emails desde el inicio del paso
EMAILS=$(search_records.sh mail.mail '[["create_date",">=","'"$TEST_START"'"]]' '["id"]' 0)

# Si la respuesta contiene registros, el paso genero emails inesperados
# Accion: investigar que metodo los genero y ajustar contexto o estrategia
```

### Verificacion por estado

Ademas de buscar emails nuevos, verificar que no hay emails en cola listos para enviar:

```bash
# Emails en outgoing (listos para envio por el cron)
search_records.sh mail.mail '[["state","=","outgoing"],["create_date",">=","'"$TEST_START"'"]]' '["id","subject","email_to"]' 0

# Emails en sending (en proceso de envio)
search_records.sh mail.mail '[["state","=","sending"],["create_date",">=","'"$TEST_START"'"]]' '["id","subject","email_to"]' 0
```

### Accion si se detectan emails inesperados

1. **Cancelar inmediatamente** los emails detectados:
```bash
update_record.sh mail.mail '[<ids>]' '{"state": "cancel"}'
```

2. **Investigar origen**: El campo `model` y `res_id` de `mail.mail` indican que modelo y
   registro disparo el email. Usar esa informacion para ajustar el contexto en los pasos siguientes.

3. **Si el email ya fue enviado** (`state = 'sent'`): ya no se puede prevenir, pero registrar
   el incidente para la seccion de incidencias del informe de test.

---

## Resumen — Orden de Operaciones

```
ANTES DEL TEST:
1. Ejecutar checklist pre-ejecucion (9 verificaciones)
2. Detectar automatizaciones peligrosas (8 categorias)
3. Aplicar estrategia de bloqueo seleccionada (A-E)
4. Desactivar crons de comunicaciones (email, SMS, WhatsApp, push, digest, snailmail)
5. Verificar que el bloqueo es efectivo

DURANTE EL TEST:
6. Usar contexto de seguridad en TODAS las llamadas API
7. Registrar timestamps de inicio/fin para limpieza
7b. Verificar emails post-paso (ver seccion siguiente)

DESPUES DEL TEST:
8. Ejecutar limpieza post-test (13 pasos)
9. Restaurar configuracion original (servidores, crons, dominio)
10. Verificar estado final del sistema
```
