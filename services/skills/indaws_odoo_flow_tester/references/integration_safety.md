# Seguridad de Integraciones Externas — Odoo Flow Tester (API)

Referencia para detectar, desactivar y restaurar integraciones externas antes de ejecutar
flujos de prueba en produccion. Las integraciones externas pueden disparar llamadas reales
(cobros, envios, SMS, facturacion electronica) si no se neutralizan previamente.

---

## Principio Fundamental

**Antes de ejecutar cualquier flujo en produccion:**
1. Auditar TODAS las integraciones activas
2. Clasificar cada una segun nivel de riesgo (GREEN / YELLOW / RED)
3. Desactivar las RED obligatoriamente, evaluar las YELLOW
4. Guardar estado previo para restauracion
5. Ejecutar flujo
6. Restaurar estado original

---

## 1. Clasificacion de Riesgo por Tipo de Integracion

### ROJO — Desactivar obligatoriamente
Integraciones que realizan llamadas externas reales con consecuencias irreversibles.

| Tipo | Modelo/Mecanismo | Consecuencia si NO se desactiva | Consulta de deteccion |
|------|-------------------|--------------------------------|----------------------|
| Payment providers (pasarelas de pago) | `payment.provider` | Cobros reales a clientes | `[["state","!=","disabled"]]` |
| Delivery carriers (transportistas) | `delivery.carrier` | Creacion de envios reales, etiquetas, cargos | `[["active","=",true],["prod_environment","=",true]]` |
| EDI / Facturacion electronica | `account.move` + `l10n_*_edi_*` | Envio de facturas a Hacienda — IRREVERSIBLE | Ver seccion 2.4 |
| SMS providers (via IAP) | `iap.account` + `sms.sms` | Envio de SMS reales, consumo de creditos | `iap.account: [["service_name","=","sms"]]` |
| WhatsApp | `whatsapp.message` + `whatsapp.account` | Envio de mensajes WhatsApp reales | `whatsapp.account: []` |
| Webhooks salientes | `ir.actions.server` con `state='webhook'` | Envio de datos POST a URLs externas | `ir.actions.server: [["state","=","webhook"]]` |
| Webhooks entrantes | `base.automation` con `trigger='on_webhook'` | Sistemas externos pueden modificar datos | `base.automation: [["trigger","=","on_webhook"],["active","=",true]]` |
| IAP accounts activas | `iap.account` | Consumo de creditos reales (snailmail, reveal, ocr) | `iap.account: []` |
| Marketing Automation | `marketing.campaign` | Envio automatizado de emails/SMS | `[["state","=","running"]]` |
| Social Marketing | `social.account`, `social.post` | Publicaciones reales en redes sociales | `social.account: [["active","=",true]]` |
| VoIP | `voip.configurator` | Llamadas telefonicas reales via SIP | Verificar modulo instalado |
| IoT devices | `iot.device` | Comandos a dispositivos fisicos | `iot.device: []` |
| Inter-company rules | `res.company` | Creacion automatica de PO/SO/facturas en empresas vinculadas | Ver seccion 2.12 |
| Payroll connectors (SEPA/banco) | `hr.payslip.run` → `account.batch.payment` | Transferencias bancarias reales | Ver seccion 2.14 |
| Digest emails | `digest.digest` | Envio periodico de resumen por email | `[["state","=","activated"]]` |
| Snailmail (correo postal) | `snailmail.letter` | Envio de correo postal fisico via IAP | Verificar cron activo |
| Web Push Notifications | `mail.push` | Notificaciones push a navegadores | Verificar cron activo |

### AMARILLO — Evaluar caso por caso
Integraciones que podrian tener efecto externo dependiendo de la configuracion.

| Tipo | Ejemplo | Riesgo potencial | Consulta de deteccion |
|------|---------|-----------------|----------------------|
| Automatizaciones con email | `base.automation` + `ir.actions.server(state=email)` | Envio de emails reales | `ir.actions.server: [["state","=","email"]]` |
| Automatizaciones con SMS | `base.automation` + `ir.actions.server(state=sms)` | Envio de SMS reales | `ir.actions.server: [["state","=","sms"]]` |
| Automatizaciones con mail_post | `base.automation` + `ir.actions.server(state=mail_post)` | Post en chatter con notificacion | `ir.actions.server: [["state","=","mail_post"]]` |
| Crons de sincronizacion | fetchmail, marketing, mass_mailing | Procesan cola, disparan automatizaciones | Ver seccion 2.15 |
| Cambios de etapa automaticos | `on_stage_set`, `on_state_set` | Podrian disparar cascadas | `base.automation: [["trigger","in",["on_stage_set","on_state_set"]]]` |
| Discuss channels con externos | `discuss.channel` con miembros partners | Notificaciones por email a externos | Verificar miembros del canal |
| Approval workflows | `approvals.approval.request` | Notificaciones a aprobadores | Verificar modulo instalado |
| Planning notifications | `planning.slot` | Notificaciones de turnos | Verificar modulo instalado |
| Calendar sync (Google/Microsoft) | `google.calendar.sync` / `microsoft.calendar.sync` | Sincronizacion bidireccional de eventos | Verificar modulos instalados |

### VERDE — Seguro para testing
Integraciones que solo afectan logica interna de Odoo, sin llamadas externas.

| Tipo | Ejemplo | Razon |
|------|---------|-------|
| Acciones automatizadas internas | Asignar campo, crear registro, cambiar etapa | Solo operan sobre la base de datos local |
| Computed fields | `_compute_*` methods | Solo recalculan valores locales |
| Report generation | `ir.actions.report` | Genera PDF/HTML localmente |
| Sequence generation | `ir.sequence` | Solo incrementa contadores locales |
| Reglas de validacion | Constraints, onchange | Solo verifican datos |
| Crons de limpieza interna | Limpiar archivos temporales, vacuum | No contactan APIs externas |
| Quality checks internos | `quality.check` (sin IoT) | Solo verificacion interna de datos |
| Maintenance requests | `maintenance.request` | Solo registro interno |
| Fleet management | `fleet.vehicle`, `fleet.vehicle.log.*` | Solo registro interno de vehiculos |
| Bus/Presence (longpolling) | `bus.bus`, `bus.presence` | Comunicacion interna browser↔server, sin llamadas externas |

---

## 2. Auditoria Pre-Ejecucion

### 2.1. Acciones Automatizadas (`base.automation`)

**Modelo:** `base.automation`
**Campos clave:** `active`, `trigger`, `model_id`, `action_server_ids`

#### Clasificacion de automatizaciones por tipo de trigger

| Trigger | Descripcion | Nivel de riesgo | Justificacion |
|---------|-------------|-----------------|---------------|
| `on_create` | Se dispara al crear un registro | **ALTO** | Los flujos de test crean registros constantemente |
| `on_write` | Se dispara al modificar un registro | **ALTO** | Los flujos modifican campos en cada paso |
| `on_create_or_write` | Combinacion de creacion y modificacion | **ALTO** | Cubre ambos casos — el mas peligroso |
| `on_unlink` | Se dispara al eliminar un registro | **MEDIO** | Solo afecta durante la fase de revert/limpieza |
| `on_time` | Basado en fecha/hora (evaluado por cron) | **BAJO** (si crons desactivados) | Requiere que el cron `base_automation.ir_cron_data_base_automation_check` este activo |
| `on_stage_set` | Se dispara al cambiar de etapa | **ALTO** | Los flujos cambian etapas (CRM, proyecto, helpdesk) |
| `on_state_set` | Se dispara al cambiar de estado | **ALTO** | Los flujos cambian estados (SO confirm, invoice post) |
| `on_tag_set` | Se dispara al asignar una etiqueta | **ALTO** | Algunos flujos asignan tags |
| `on_webhook` | Webhook entrante que ejecuta acciones | **ALTO** | Sistemas externos pueden disparar acciones |
| `on_message_received` | Se dispara al recibir un mensaje | **MEDIO** | Solo si el flujo genera mensajes entrantes |
| `on_message_sent` | Se dispara al enviar un mensaje | **MEDIO** | Solo si el flujo genera mensajes salientes |

#### Lectura y clasificacion de automatizaciones via API

```python
# Leer todas las automatizaciones activas con sus acciones
automations = execute_kw('base.automation', 'search_read',
    [[['active', '=', True]]],
    {'fields': ['name', 'trigger', 'action_server_ids', 'model_id']})

# Clasificar por nivel de riesgo
HIGH_RISK_TRIGGERS = {'on_create', 'on_write', 'on_create_or_write',
                      'on_stage_set', 'on_state_set', 'on_tag_set', 'on_webhook'}
MEDIUM_RISK_TRIGGERS = {'on_unlink', 'on_message_received', 'on_message_sent'}
LOW_RISK_TRIGGERS = {'on_time'}

high_risk = [a for a in automations if a['trigger'] in HIGH_RISK_TRIGGERS]
medium_risk = [a for a in automations if a['trigger'] in MEDIUM_RISK_TRIGGERS]
low_risk = [a for a in automations if a['trigger'] in LOW_RISK_TRIGGERS]

# Para las de alto riesgo, verificar si sus acciones envian comunicaciones
action_ids = []
for a in high_risk:
    action_ids.extend(a.get('action_server_ids', []))

if action_ids:
    actions = execute_kw('ir.actions.server', 'search_read',
        [[['id', 'in', action_ids]]],
        {'fields': ['name', 'state', 'template_id', 'model_id']})
    comm_actions = [a for a in actions if a['state'] in ('email', 'sms', 'webhook', 'mail_post')]
```

**Triggers peligrosos:**
- `on_create_or_write` — se dispara al crear/modificar registros (flujos crean registros)
- `on_create`, `on_write` — variantes del anterior
- `on_stage_set`, `on_state_set`, `on_tag_set` — se disparan con cambios de estado (flujos cambian estados)
- `on_webhook` — webhooks entrantes que ejecutan acciones
- `on_time` — basados en fecha/hora, podrian coincidir durante el test
- `on_message_received`, `on_message_sent` — si el flujo envia mensajes

**Tipos de accion peligrosos en `ir.actions.server`:**
- `email` — Envia un email usando `mail.template`
- `sms` — Envia un SMS usando `sms.template`
- `mail_post` — Publica un mensaje en el chatter (puede generar notificaciones)
- `webhook` — Envia POST request a una URL externa con datos del registro

**Deteccion:**
```bash
# Todas las automatizaciones activas
search_records.sh base.automation '[["active","=",true]]' '["id","name","trigger","model_id","active"]' 0

# Solo las que afectan modelos que vamos a tocar en el flujo
search_records.sh base.automation '[["active","=",true],["model_id.model","in",["sale.order","account.move","stock.picking"]]]' '["id","name","trigger","model_id"]' 0

# Automatizaciones que envian email/SMS (las mas peligrosas)
search_records.sh base.automation '[["active","=",true],["action_server_ids.state","in",["mail_post","sms","email"]]]' '["id","name","trigger"]' 0

# Automatizaciones con webhooks entrantes
search_records.sh base.automation '[["active","=",true],["trigger","=","on_webhook"]]' '["id","name","url","active"]' 0

# Acciones de servidor tipo webhook saliente (POST a URL externa)
search_records.sh ir.actions.server '[["state","=","webhook"]]' '["id","name","model_id","webhook_url"]' 0
```

**Desactivacion:**
```bash
# Guardar IDs activas antes de desactivar
AUTOMATION_IDS=$(search_records.sh base.automation '[["active","=",true]]' '["id"]' 0)

# Desactivar una automatizacion especifica
update_record.sh base.automation '[<id>]' '{"active": false}'
```

**Restauracion:**
```bash
# Reactivar las que estaban activas
update_record.sh base.automation '[<id>]' '{"active": true}'
```

---

### 2.2. Payment Providers (`payment.provider`)

**Modelo:** `payment.provider`
**Campo clave:** `state` (selection: `disabled`, `enabled`, `test`)

En Odoo 18, el campo `state` controla completamente el comportamiento del proveedor:
- `enabled` — procesa pagos reales en produccion
- `test` — procesa pagos de prueba (aun puede hacer llamadas a la API del proveedor en modo sandbox)
- `disabled` — completamente inactivo

**IMPORTANTE:** Al cambiar `state` a `disabled`:
- Los tokens de pago asociados se **archivan automaticamente** (`payment.token` → `active=False`)
- Los metodos de pago sin otros proveedores activos se desactivan
- Al reactivar, se reactivan los metodos por defecto pero NO los tokens

**Proveedores en Odoo 18 (community + enterprise):**
Stripe, Adyen, PayPal, Authorize.net, Mollie, Worldline, Nuvei, Buckaroo, Razorpay,
Mercado Pago, APS, AsiaPay, Flutterwave, Xendit, SEPA Direct Debit, Custom, Demo.

**Deteccion:**
```bash
# Proveedores activos (enabled o test)
search_records.sh payment.provider '[["state","!=","disabled"]]' '["id","name","code","state","company_id"]' 0
```

**Desactivacion:**
```bash
# Guardar estado previo
PROVIDER_STATES=$(search_records.sh payment.provider '[["state","!=","disabled"]]' '["id","name","state"]' 0)

# Desactivar (poner en disabled)
update_record.sh payment.provider '[<id>]' '{"state": "disabled"}'
```

**Restauracion:**
```bash
# Restaurar al estado original (enabled o test)
update_record.sh payment.provider '[<id>]' '{"state": "<estado_original>"}'

# NOTA: Los tokens archivados NO se restauran automaticamente.
# Si habia tokens activos, restaurarlos manualmente:
search_records.sh payment.token '[["provider_id","=",<id>],["active","=",false]]' '["id"]' 0
update_record.sh payment.token '[<token_ids>]' '{"active": true}'
```

**Riesgos si NO se desactiva:**
- Cobros reales a tarjetas de clientes
- Cargos en cuentas bancarias via SEPA Direct Debit
- Consumo de creditos en pasarelas de pago

---

### 2.3. Delivery Carriers (`delivery.carrier`)

**Modelo:** `delivery.carrier`
**Campos clave:** `prod_environment` (Boolean), `active` (Boolean), `delivery_type` (Selection)

El campo `prod_environment` controla si el carrier usa credenciales de produccion o sandbox:
- `True` — llama a la API real del transportista
- `False` — usa entorno de pruebas/sandbox

Los carriers con `delivery_type` = `fixed` o `base_on_rule` NO hacen llamadas externas
(son calculo local). Solo los carriers con `delivery_type` de proveedor externo contactan APIs.

**Carriers en Odoo 18 (enterprise):**
FedEx, UPS, DHL, USPS, Sendcloud, EasyPost, bpost, Envia, Starshipit, Shiprocket,
Mondial Relay.

**Deteccion:**
```bash
# Carriers activos en entorno de produccion (los peligrosos)
search_records.sh delivery.carrier '[["active","=",true],["prod_environment","=",true]]' '["id","name","delivery_type","prod_environment"]' 0

# Todos los carriers activos (para referencia)
search_records.sh delivery.carrier '[["active","=",true]]' '["id","name","delivery_type","prod_environment"]' 0
```

**Desactivacion (estrategia preferida — cambiar a sandbox):**
```bash
CARRIER_STATES=$(search_records.sh delivery.carrier '[["active","=",true],["prod_environment","=",true]]' '["id","name","prod_environment"]' 0)
update_record.sh delivery.carrier '[<id>]' '{"prod_environment": false}'
```

**Desactivacion (alternativa — desactivar completamente):**
```bash
update_record.sh delivery.carrier '[<id>]' '{"active": false}'
```

**Restauracion:**
```bash
update_record.sh delivery.carrier '[<id>]' '{"prod_environment": true}'
```

**Riesgos si NO se desactiva:**
- Creacion de etiquetas de envio reales (con cargo)
- Solicitudes de recogida a transportistas
- Consumo de creditos en plataformas como Sendcloud/EasyPost

---

### 2.4. Facturacion Electronica (EDI)

**Modelos relevantes:**
- Odoo 18 no usa `account.edi.format` como modelo central (deprecado desde v17).
- La facturacion electronica se implementa via modulos de localizacion: `l10n_es_edi_sii`,
  `l10n_es_edi_tbai`, `l10n_es_edi_verifactu`, `l10n_it_edi`, `l10n_fr_facturx`, etc.
- Cada modulo extiende `account.move` con logica de envio propia.

**Riesgo en Espana:**
| Modulo | Sistema | Campo test_env | Consecuencia si NO se neutraliza |
|--------|---------|---------------|--------------------------------|
| `l10n_es_edi_sii` | SII (AEAT) | `res.company.l10n_es_sii_test_env` | Factura comunicada a Hacienda — NO reversible |
| `l10n_es_edi_tbai` | TicketBAI (Pais Vasco/Navarra) | `res.company.l10n_es_tbai_test_env` | Factura registrada en sistema foral — NO reversible |
| `l10n_es_edi_verifactu` | Veri*Factu (AEAT) | `res.company.l10n_es_edi_verifactu_test_environment` | Hash fiscal generado + envio a AEAT — NO reversible |
| `l10n_es_edi_facturae` | FACe / FACeB2B | N/A | Factura electronica enviada — NO reversible |

**Deteccion:**
```bash
# Verificar si hay modulos EDI instalados
search_records.sh ir.module.module '[["name","like","l10n_%_edi%"],["state","=","installed"]]' '["name","shortdesc","state"]' 0

# Verificar si hay modulos EDI espanoles instalados
search_records.sh ir.module.module '[["name","in",["l10n_es_edi_sii","l10n_es_edi_tbai","l10n_es_edi_verifactu","l10n_es_edi_facturae"]],["state","=","installed"]]' '["name","state"]' 0

# Verificar estado de test_env por empresa (SII)
search_records.sh res.company '[]' '["id","name","l10n_es_sii_test_env"]' 0

# Verificar estado de test_env por empresa (TicketBAI)
search_records.sh res.company '[]' '["id","name","l10n_es_tbai_test_env"]' 0

# Verificar estado de test_env por empresa (Veri*Factu)
search_records.sh res.company '[]' '["id","name","l10n_es_edi_verifactu_test_environment"]' 0

# Buscar crons de EDI activos
search_records.sh ir.cron '[["active","=",true],["cron_name","ilike","edi"]]' '["id","cron_name","active","interval_number","interval_type"]' 0

# Buscar cron especifico de Veri*Factu (tiene cron de batch)
search_records.sh ir.cron '[["active","=",true],["cron_name","ilike","verifactu"]]' '["id","cron_name","active"]' 0
```

**Estrategia de neutralizacion:**

**Opcion 1 — Activar modo test en la empresa (PREFERIDA para SII/TicketBAI/Verifactu):**

Las bases de datos neutralizadas ya tienen estos campos en `True` (via `neutralize.sql`).
Para bases NO neutralizadas, activar manualmente:

```bash
# SII → modo test (usa URL de test de AEAT)
update_record.sh res.company '[<company_id>]' '{"l10n_es_sii_test_env": true}'

# TicketBAI → modo test (usa URL de test de cada agencia foral)
update_record.sh res.company '[<company_id>]' '{"l10n_es_tbai_test_env": true}'

# Veri*Factu → modo test (usa URL de test de AEAT)
update_record.sh res.company '[<company_id>]' '{"l10n_es_edi_verifactu_test_environment": true}'
```

**ADVERTENCIA:** Incluso en modo test, se generan documentos fiscales (hashes, cadenas de facturacion).
Si el flujo REQUIERE publicar facturas y hay EDI activo, considerar las siguientes alternativas:

**Opcion 2 — Evitar publicar facturas durante el test:**
Si el flujo crea facturas, dejarlas en `draft` y NO ejecutar `action_post`.
La mayoria de modulos EDI solo envian al publicar.

**Opcion 3 — Desactivar crons de EDI:**
Algunos modulos usan crons para reintentar envios fallidos o procesamiento batch:
```bash
# Veri*Factu tiene un cron de batch activo por defecto
search_records.sh ir.cron '[["active","=",true],["cron_name","ilike","verifactu"]]' '["id","cron_name"]' 0
update_record.sh ir.cron '[<id>]' '{"active": false}'
```

**Opcion 4 — Usar empresa de prueba:**
Si existe una empresa sin configuracion fiscal EDI, ejecutar el flujo en esa empresa.

**Restauracion (si se cambio test_env):**
```bash
# Restaurar SII a produccion
update_record.sh res.company '[<company_id>]' '{"l10n_es_sii_test_env": false}'

# Restaurar TicketBAI a produccion
update_record.sh res.company '[<company_id>]' '{"l10n_es_tbai_test_env": false}'

# Restaurar Veri*Factu a produccion
update_record.sh res.company '[<company_id>]' '{"l10n_es_edi_verifactu_test_environment": false}'
```

**Riesgos si NO se desactiva:**
- Envio irrecuperable de facturas a Hacienda (SII, TicketBAI, Veri*Factu)
- Generacion de hash fiscal en facturas de test (rompe cadena de facturacion)
- Contaminacion de registros fiscales oficiales
- En Veri*Factu: la cadena de hash es secuencial — facturas de test quedan permanentemente en la cadena

---

### 2.5. IAP Accounts (In-App Purchase)

**Modelo:** `iap.account`
**Campos clave:** `account_token`, `service_name` (via `service_id`)

Los servicios IAP consumen creditos reales al ser invocados:
- `sms` — envio de SMS
- `partner_autocomplete` — enriquecimiento de contactos
- `snailmail` — envio de correo postal
- `ocr` — digitalizacion de facturas
- `reveal` — generacion de leads

**Deteccion:**
```bash
# Cuentas IAP activas
search_records.sh iap.account '[]' '["id","service_name","account_token"]' 0
```

**Estrategia de neutralizacion:**
No hay campo `active` en `iap.account`. La neutralizacion se hace desactivando los
consumidores del servicio:

1. **SMS:** Desactivar automatizaciones que envian SMS (ver seccion 2.1) y crons SMS
2. **Partner autocomplete:** No se dispara automaticamente, es manual
3. **Snailmail:** Desactivar crons de envio postal
4. **OCR:** Desactivar crons de digitalizacion
5. **Reveal:** Desactivar crons de reveal

```bash
# Crons relacionados con IAP
search_records.sh ir.cron '[["active","=",true],["cron_name","ilike","iap"]]' '["id","cron_name","active"]' 0
search_records.sh ir.cron '[["active","=",true],["cron_name","ilike","sms"]]' '["id","cron_name","active"]' 0
search_records.sh ir.cron '[["active","=",true],["cron_name","ilike","snailmail"]]' '["id","cron_name","active"]' 0
```

**Alternativa (instancias neutralizadas):**
Las bases de datos neutralizadas ya tienen los tokens IAP deshabilitados (sufijo `+disabled`
en `account_token`). Verificar:
```bash
search_records.sh ir.config_parameter '[["key","=","database.is_neutralized"]]' '["value"]' 1
```

---

### 2.6. SMS (`sms.sms`)

**Modelo:** `sms.sms`
**Campos clave:** `state` (selection: `outgoing`, `process`, `pending`, `sent`, `error`, `canceled`)

Los SMS se envian via IAP (servicio `sms`). El envio real ocurre cuando:
- Se crea un `sms.sms` con `state=outgoing`
- El cron `sms.ir_cron_sms_scheduler_action` procesa la cola
- `_send_with_api()` llama al endpoint IAP que consume creditos

**Deteccion:**
```bash
# Cron de SMS activo
search_records.sh ir.cron '[["active","=",true],["model_id.model","=","sms.sms"]]' '["id","name","active","interval_number","interval_type"]' 0

# Automatizaciones que envian SMS
search_records.sh ir.actions.server '[["state","=","sms"]]' '["id","name","model_id","sms_template_id"]' 0
```

**Desactivacion:**
```bash
update_record.sh ir.cron '[<sms_cron_id>]' '{"active": false}'
update_record.sh base.automation '[<automation_ids>]' '{"active": false}'
```

**Restauracion:**
```bash
update_record.sh ir.cron '[<sms_cron_id>]' '{"active": true}'
update_record.sh base.automation '[<automation_ids>]' '{"active": true}'
```

---

### 2.7. WhatsApp (`whatsapp.message`)

**Modulo:** `whatsapp` (Enterprise)
**Modelo:** `whatsapp.message`
**Campos clave:** `state`, `mobile_number`, `body`

WhatsApp en Odoo 18 Enterprise usa la API de WhatsApp Business. El envio real ocurre cuando:
- Se crea un `whatsapp.message` con `state=outgoing`
- El cron `whatsapp.ir_cron_send_whatsapp_queue` llama a `_send_cron()`
- `_send_message()` envia via la API de Meta/WhatsApp Business

**Modulos dependientes (todos Enterprise):**
`whatsapp_sale`, `whatsapp_account`, `whatsapp_delivery`, `whatsapp_event`,
`whatsapp_hr_referral`, `whatsapp_payment`, `whatsapp_pos`, `whatsapp_website_sale`,
`marketing_automation_whatsapp`

**Deteccion:**
```bash
# Verificar si WhatsApp esta instalado
search_records.sh ir.module.module '[["name","=","whatsapp"],["state","=","installed"]]' '["name","state"]' 0

# Cron de WhatsApp activo
search_records.sh ir.cron '[["active","=",true],["model_id.model","=","whatsapp.message"]]' '["id","name","active"]' 0

# Cuentas WhatsApp configuradas
search_records.sh whatsapp.account '[]' '["id","name","app_uid","active"]' 0
```

**Desactivacion:**
```bash
# Desactivar cron de envio
update_record.sh ir.cron '[<whatsapp_cron_id>]' '{"active": false}'
```

**Restauracion:**
```bash
update_record.sh ir.cron '[<whatsapp_cron_id>]' '{"active": true}'
```

---

### 2.8. Webhooks — Entrantes y Salientes

#### 2.8.1. Webhooks Entrantes (`base.automation` con trigger `on_webhook`)

**Modelo:** `base.automation` (con `trigger = 'on_webhook'`)
**Campos clave:** `webhook_uuid`, `url`, `record_getter`, `log_webhook_calls`

En Odoo 18, los webhooks entrantes se implementan como automatizaciones con trigger `on_webhook`.
Cada una expone una URL publica: `/web/hook/<webhook_uuid>`.

Los webhooks son **entrantes** (reciben llamadas de sistemas externos). El riesgo es que
la accion asociada al webhook podria modificar datos durante el test, o que un sistema
externo envie un webhook durante la ejecucion del flujo.

**Deteccion:**
```bash
search_records.sh base.automation '[["active","=",true],["trigger","=","on_webhook"]]' '["id","name","model_id","url","active"]' 0
```

**Desactivacion:**
```bash
WEBHOOK_IDS=$(search_records.sh base.automation '[["active","=",true],["trigger","=","on_webhook"]]' '["id"]' 0)
update_record.sh base.automation '[<id>]' '{"active": false}'
```

**Restauracion:**
```bash
update_record.sh base.automation '[<id>]' '{"active": true}'
# NOTA: La URL del webhook no cambia al reactivar (mismo UUID).
```

#### 2.8.2. Webhooks Salientes (`ir.actions.server` con `state='webhook'`)

**Modelo:** `ir.actions.server`
**Campos clave:** `state='webhook'`, `webhook_url`, `webhook_field_ids`

En Odoo 18, las acciones de servidor pueden enviar POST requests a URLs externas
cuando se disparan. El payload incluye los campos seleccionados en `webhook_field_ids`
mas el nombre de la accion (`_name`).

El metodo `_run_action_webhook()` envia un HTTP POST a `webhook_url` con datos del registro.

**Deteccion:**
```bash
# Acciones de servidor tipo webhook
search_records.sh ir.actions.server '[["state","=","webhook"]]' '["id","name","model_id","webhook_url","webhook_field_ids"]' 0

# Automatizaciones que ejecutan estas acciones
search_records.sh base.automation '[["active","=",true]]' '["id","name","trigger","model_id","action_server_ids"]' 0
```

**Cruzar:** Si una automatizacion activa contiene un `action_server_id` que es de tipo webhook
y su trigger/modelo coincide con el flujo de test, se enviaran datos a la URL externa.

**Desactivacion:** Desactivar la automatizacion padre o cambiar el trigger.
```bash
update_record.sh base.automation '[<id>]' '{"active": false}'
```

---

### 2.9. Marketing Automation (`marketing.campaign`)

**Modelo:** `marketing.campaign`
**Campo clave:** `state` (selection: `draft`, `running`, `stopped`)

Las campanas de marketing envian emails/SMS automaticamente a contactos en determinados
intervalos de tiempo segun reglas configuradas. Si una campana esta `running`, puede
enviar comunicaciones durante la ejecucion del test.

**Deteccion:**
```bash
# Campanas activas
search_records.sh marketing.campaign '[["state","=","running"]]' '["id","name","model_id","state"]' 0

# Mass mailings en proceso de envio
search_records.sh mailing.mailing '[["state","in",["sending","in_queue"]]]' '["id","name","subject","mailing_model_real","state"]' 0
```

**Desactivacion:**
```bash
# Detener campanas (cambiar a stopped)
execute_method.sh marketing.campaign action_stop '[[<id>]]' '{}'
```

**Restauracion:**
```bash
# Reactivar campanas
execute_method.sh marketing.campaign action_start '[[<id>]]' '{}'
```

**Riesgos si NO se desactiva:**
- Envio de emails masivos a contactos reales
- Envio de SMS con consumo de creditos IAP
- Contaminacion de estadisticas de campanas

---

### 2.10. Social Marketing (`social.account`, `social.post`)

**Modelo:** `social.account`
**Campos clave:** `active` (Boolean), `media_type` (Selection)

Las cuentas de social marketing permiten publicar contenido en redes sociales reales
(Facebook, Instagram, LinkedIn, Twitter/X, YouTube).

El modulo `social` tiene su propio cron (`social.ir_cron_data`) que procesa posts programados.

**Deteccion:**
```bash
# Cuentas sociales activas
search_records.sh social.account '[["active","=",true]]' '["id","name","media_type","active"]' 0

# Posts programados pendientes
search_records.sh social.post '[["state","=","scheduled"]]' '["id","message","scheduled_date","state"]' 0

# Cron de social activo
search_records.sh ir.cron '[["active","=",true],["model_id.model","=","social.post"]]' '["id","name","active"]' 0
```

**Desactivacion:**
```bash
update_record.sh social.account '[<id>]' '{"active": false}'

# Tambien desactivar cron de social
update_record.sh ir.cron '[<social_cron_id>]' '{"active": false}'
```

**Restauracion:**
```bash
update_record.sh social.account '[<id>]' '{"active": true}'
update_record.sh ir.cron '[<social_cron_id>]' '{"active": true}'
```

**Riesgos si NO se desactiva:**
- Publicaciones reales en redes sociales del cliente
- Posts programados que se publican durante el test

---

### 2.11. VoIP (`voip.configurator`)

**Modulo:** `voip` (Enterprise)

Si esta instalado, el modulo VoIP puede iniciar llamadas telefonicas reales via protocolo
SIP al interactuar con registros de CRM, Helpdesk, etc.

**Deteccion:**
```bash
# Verificar si el modulo VoIP esta instalado
search_records.sh ir.module.module '[["name","=","voip"],["state","=","installed"]]' '["name","state"]' 0
```

**Estrategia:** El riesgo de VoIP es bajo en tests API (las llamadas se inician desde UI).
No requiere desactivacion proactiva, pero tener en cuenta si se ejecutan tests via Playwright.

---

### 2.12. Inter-Company Rules

**Modulo:** `account_inter_company_rules`, `sale_purchase_inter_company_rules`

Estas reglas crean automaticamente documentos (PO, SO, facturas) en empresas vinculadas
cuando se confirman documentos en la empresa actual. Esto puede causar efectos cascada
no deseados durante tests.

**Deteccion:**
```bash
# Verificar si inter-company esta instalado
search_records.sh ir.module.module '[["name","in",["account_inter_company_rules","sale_purchase_inter_company_rules"]],["state","=","installed"]]' '["name","state"]' 0

# Verificar configuracion de inter-company por empresa
search_records.sh res.company '[]' '["id","name","intercompany_generate_purchase_orders","intercompany_generate_sales_orders"]' 0
```

**Desactivacion:**
```bash
# Desactivar generacion automatica de PO/SO inter-company
update_record.sh res.company '[<id>]' '{"intercompany_generate_purchase_orders": false, "intercompany_generate_sales_orders": false}'
```

**Restauracion:**
```bash
update_record.sh res.company '[<id>]' '{"intercompany_generate_purchase_orders": true, "intercompany_generate_sales_orders": true}'
```

**Riesgos si NO se desactiva:**
- Creacion automatica de pedidos en otras empresas
- Multiplicacion de registros de test entre empresas
- Complejidad exponencial en la reversion

---

### 2.13. Digest Emails (`digest.digest`)

**Modelo:** `digest.digest`
**Campos clave:** `state` (selection: `activated`, `deactivated`), `is_activated`

Los digest emails envian resumenes periodicos de actividad (KPIs) a usuarios suscritos.
El cron `digest.ir_cron_digest_scheduler_action` procesa los digests activos.

**Deteccion:**
```bash
search_records.sh digest.digest '[["state","=","activated"]]' '["id","name","periodicity","state"]' 0
```

**Desactivacion:**
```bash
update_record.sh digest.digest '[<id>]' '{"state": "deactivated"}'
```

**Restauracion:**
```bash
update_record.sh digest.digest '[<id>]' '{"state": "activated"}'
```

---

### 2.14. Payroll Bank Connectors

**Modelos relevantes:** `hr.payslip.run` → `account.batch.payment`

Modulos como `l10n_us_payment_nacha`, `l10n_ca_payment_cpa005`, `payment_sepa_direct_debit`
pueden generar archivos de transferencia bancaria reales al procesar nominas.

**Deteccion:**
```bash
# Verificar modulos de batch payment instalados
search_records.sh ir.module.module '[["name","in",["l10n_us_payment_nacha","l10n_ca_payment_cpa005","payment_sepa_direct_debit","account_batch_payment"]],["state","=","installed"]]' '["name","state"]' 0
```

**Estrategia:** No ejecutar `action_validate` en `hr.payslip.run` si hay modulos de
batch payment instalados. Mantener payslips en estado `draft` o `verify`.

---

### 2.15. Crons Interferentes (`ir.cron`)

**Modelo:** `ir.cron`
**Campos clave:** `active`, `cron_name`, `nextcall`, `interval_number`, `interval_type`

#### Crons criticos que interfieren con testing

Estos crons especificos deben ser desactivados OBLIGATORIAMENTE antes de ejecutar flujos de test:

| XML ID | Modelo | Funcion | Riesgo para testing |
|--------|--------|---------|-------------------|
| `mail.ir_cron_mail_scheduler_action` | `mail.mail` | `process_email_queue(batch_size=1000)` | **CRITICO** — Procesa y ENVIA todos los `mail.mail` en estado `outgoing`. Si un paso del test genera emails a pesar de los flags de contexto, este cron los enviara |
| `account.ir_cron_auto_post_draft_entry` | `account.move` | `_autopost_draft_entries` | **ALTO** — Auto-publica asientos contables en borrador. Puede publicar facturas de test y disparar EDI |
| `fetchmail.ir_cron_mail_poll` | `fetchmail.server` | `_fetch_mails` | **ALTO** — Recibe emails entrantes y puede disparar automatizaciones `on_message_received` y crear registros |
| `sale.ir_cron_send_invoices` | `sale.order` | `_cron_send_invoices` | **ALTO** — Auto-envia facturas pendientes de pedidos de venta |
| `mail.ir_cron_send_scheduled_message` | `mail.message.schedule` | `_send_notifications_cron` | **MEDIO** — Envia notificaciones programadas que coincidan durante el test |
| `mail.ir_cron_post_scheduled_message` | `mail.scheduled.message` | `_post_messages_cron` | **MEDIO** — Publica mensajes planificados por usuarios |
| `sms.ir_cron_sms_scheduler_action` | `sms.sms` | `_process_queue` | **ALTO** — Envia SMS reales via IAP |
| `whatsapp.ir_cron_send_whatsapp_queue` | `whatsapp.message` | `_send_cron` | **ALTO** — Envia mensajes WhatsApp reales |
| `digest.ir_cron_digest_scheduler_action` | `digest.digest` | `_cron_send_digest_email` | **MEDIO** — Envia digest periodicos |
| `snailmail.snailmail_print` | `snailmail.letter` | `_snailmail_cron` | **ALTO** — Envia correo postal fisico via IAP |
| `mail.ir_cron_web_push_notification` | `mail.push` | `_push_notification_to_endpoint` | **MEDIO** — Push notifications a navegadores |
| `base_automation.ir_cron_data_base_automation_check` | `base.automation` | `_check` | **ALTO** — Evalua y ejecuta automatizaciones `on_time`. Si hay automatizaciones con trigger temporal que coincidan, se ejecutaran |

**Atencion especial:** Cualquier cron con `state = 'code'` cuyo campo `code` referencie modelos
involucrados en el test debe ser revisado manualmente. Leer el campo `code` para determinar
si puede interferir.

#### Clasificacion general por patron de nombre

Crons que pueden interferir con la ejecucion de flujos:

| Patron en nombre | Modulo | Riesgo |
|-------------------|--------|--------|
| `*payment*` | payment | Post-procesado de transacciones de pago |
| `*sms*` | sms | Envio de cola de SMS |
| `*whatsapp*` | whatsapp | Envio de mensajes WhatsApp |
| `*mail*queue*` / `*mail*scheduler*` | mail | Envio de cola de emails |
| `*push*notification*` | mail | Envio de push notifications a navegadores |
| `*edi*` / `*verifactu*` | l10n_*_edi* | Envio/reintento de facturacion electronica |
| `*snailmail*` | snailmail | Envio de correo postal fisico |
| `*iap*` | iap | Operaciones IAP programadas |
| `*automation*` | base_automation | Ejecucion de reglas basadas en tiempo |
| `*delivery*` | delivery | Procesado de envios |
| `*fetchmail*` | fetchmail | Recepcion de emails (podria disparar automatizaciones) |
| `*marketing*` | marketing_automation | Ejecucion de campanas |
| `*mailing*` | mass_mailing | Envio de mailings masivos |
| `*digest*` | digest | Envio de digest emails |
| `*social*` | social | Publicacion de posts programados |
| `*scheduled*message*` | mail | Publicacion de mensajes programados |
| `*auto_post*` | account | Auto-publicacion de asientos contables |
| `*send_invoices*` | sale | Auto-envio de facturas |

**Deteccion completa:**
```bash
# Consulta unificada — todos los crons activos
search_records.sh ir.cron '[["active","=",true]]' '["id","cron_name","nextcall","interval_number","interval_type"]' 0
```

**Desactivacion:**
```bash
CRON_SNAPSHOT=$(search_records.sh ir.cron '[["active","=",true],["id","in",[<list_ids>]]]' '["id","cron_name","active"]' 0)
update_record.sh ir.cron '[<id>]' '{"active": false}'
```

**Restauracion:**
```bash
update_record.sh ir.cron '[<id>]' '{"active": true}'
```

---

### 2.16. Audit Trail (Odoo 17+)

#### Deteccion

El audit trail impide borrar `account.move` con `posted_before=True`. Se activa a nivel de company:

```bash
# Verificar si audit trail esta activo
search_records.sh res.company '[["id","=",1]]' '["check_account_audit_trail"]'
```

#### Localizaciones que lo activan por defecto
| Localizacion | Modulo | Efecto |
|-------------|--------|--------|
| Alemania (l10n_de) | `l10n_de` | `check_account_audit_trail = True` por defecto |
| Dinamarca (l10n_dk) | `l10n_dk` | `check_account_audit_trail = True` por defecto |

#### Impacto en Flow Tester
- Si audit trail activo: `account.move.unlink()` falla para moves con `posted_before=True`
- `button_draft()` falla si `_is_protected_by_audit_trail()` devuelve True
- **Alternativa:** Solo `button_cancel` (cancelar), NO borrar. O usar contexto `force_delete=True`
- **Clasificacion:** AMARILLA — verificar antes de ejecutar flujos contables

#### Deteccion Automatica (verify_data.py)
```bash
# Añadir a verificacion de sistema:
search_records.sh res.company '[["check_account_audit_trail","=",true]]' '["name"]'
# Si devuelve resultados → advertir que moves posted no se podran borrar
```

---

### 2.17. Crons Peligrosos por Modulo

Lista completa de crons que pueden generar side effects durante testing. Deben pausarse con safe_mode.py.

#### Categoria ROJA — Pausar SIEMPRE en produccion

| Modulo | Cron (ir.cron) | XML ID | Riesgo |
|--------|---------------|--------|--------|
| mail | Mail: Email Queue Manager | `mail.ir_cron_mail_scheduler_action` | Envia emails pendientes |
| mail | Mail: Fetchmail Service | `fetchmail.ir_cron_mail_poll` | Procesa emails entrantes |
| sms | SMS: Send Queue | `sms.ir_cron_sms_scheduler_action` | Envia SMS pendientes |
| mass_mailing | Mass Mailing: Process Queue | `mass_mailing.ir_cron_mass_mailing_queue` | Envia campanas email |
| marketing_automation | Marketing Automation | `marketing_automation.ir_cron_campaign_execute_activities` | Ejecuta automatizaciones marketing |
| account_edi | EDI: Process Queue | `account_edi.ir_cron_edi_network` | Envia facturas electronicas |
| sale_subscription | Subscriptions: Generate Recurring Invoices | `sale_subscription.ir_cron_generate_recurring_invoices` | Genera facturas recurrentes |
| digest | Digest: Send Periodic Emails | `digest.ir_cron_digest_scheduler_action` | Envia resumenes periodicos |
| social | Social: Post Queue | `social.ir_cron_post_messages` | Publica en RRSS |
| snailmail | Snailmail: Send Letters | `snailmail.ir_cron_snailmail_parse_send` | Envia correo postal |

#### Categoria AMARILLA — Pausar segun caso

| Modulo | Cron | XML ID | Riesgo |
|--------|------|--------|--------|
| stock | Scheduler: Run Procurement | `stock.ir_cron_scheduler_action` | Puede generar POs/MOs automaticos |
| purchase | Scheduler: Auto Send RFQ | `purchase.ir_cron_purchase_auto_send_rfq` | Envia RFQs a proveedores |
| sale | Sale: Send Invoices | `sale.ir_cron_send_invoice_cron` | Envia facturas automaticamente |
| hr_attendance | Attendance: Auto Checkout | `hr_attendance.ir_cron_auto_check_out` | Cierra fichajes abiertos |
| website | Website: Sitemap Generator | `website.ir_cron_sitemap_generator` | Regenera sitemap |

#### Como Pausar Crons via API

```bash
# Buscar cron por XML ID
search_records.sh ir.cron '[["cron_name","like","mail_scheduler"]]' '["id","active","cron_name"]'

# Desactivar cron
update_record.sh ir.cron '[<cron_id>]' '{"active": false}'

# Reactivar cron (en restauracion)
update_record.sh ir.cron '[<cron_id>]' '{"active": true}'
```

**IMPORTANTE:** Guardar snapshot de crons activos ANTES de desactivar para restaurar correctamente.

---

## 3. Riesgos Especificos por Localizacion

### 3.1. Espana — Localizaciones Fiscales (CRITICO)

Espana tiene CUATRO modulos de facturacion electronica en Odoo 18, cada uno con envio a organismos
gubernamentales. **El envio es IRREVERSIBLE** — una vez comunicada la factura, no se puede eliminar del
registro fiscal oficial.

#### SII — Suministro Inmediato de Informacion (AEAT)

**Modulo:** `l10n_es_edi_sii` (Community)
**Aplica a:** Todas las empresas espanolas obligadas (facturacion > 6M EUR, grandes empresas, REDEME)

- **Trigger:** Al publicar factura (`action_post`) en empresa con SII habilitado
- **API externa:** AEAT (Agencia Tributaria) — envio SOAP
- **Campo test:** `res.company.l10n_es_sii_test_env` (Boolean)
- **Neutralize.sql:** `UPDATE res_company SET l10n_es_sii_test_env = true;`
- **Prevencion:** Activar `l10n_es_sii_test_env = True` redirige a entorno de test de AEAT

```bash
# Verificar y activar modo test
search_records.sh res.company '[]' '["id","name","l10n_es_sii_test_env"]' 0
update_record.sh res.company '[<id>]' '{"l10n_es_sii_test_env": true}'
```

#### TicketBAI (Pais Vasco / Navarra)

**Modulo:** `l10n_es_edi_tbai` (Community)
**Aplica a:** Empresas en Pais Vasco (Araba, Bizkaia, Gipuzkoa) y Navarra

- **Trigger:** Al publicar factura en empresa con TicketBAI configurado
- **API externa:** Haciendas forales — URL especifica por agencia
- **Campo test:** `res.company.l10n_es_tbai_test_env` (Boolean)
- **Neutralize.sql:** `UPDATE res_company SET l10n_es_tbai_test_env = true;`
- **Particularidad:** En modo test, cada agencia tiene sus propias credenciales de prueba

```bash
# Verificar y activar modo test
search_records.sh res.company '[]' '["id","name","l10n_es_tbai_test_env"]' 0
update_record.sh res.company '[<id>]' '{"l10n_es_tbai_test_env": true}'
```

#### Veri*Factu (AEAT — Sistema de Verificacion de Facturas)

**Modulo:** `l10n_es_edi_verifactu` (Community)
**Aplica a:** Todas las empresas espanolas (obligatorio a partir de 2026)

- **Trigger:** Al publicar factura + cron batch `cron_verifactu_batch` (diario)
- **API externa:** AEAT — envio por lotes
- **Campo test:** `res.company.l10n_es_edi_verifactu_test_environment` (Boolean)
- **Neutralize.sql:** `UPDATE res_company SET l10n_es_edi_verifactu_test_environment = TRUE;`
- **Cron activo:** `l10n_es_edi_verifactu.cron_verifactu_batch` — ejecuta `trigger_next_batch()` diariamente
- **PELIGRO ESPECIAL:** Veri*Factu usa cadena de hash secuencial — las facturas de test quedan
  permanentemente enlazadas en la cadena, contaminando el registro fiscal

```bash
# Verificar y activar modo test
search_records.sh res.company '[]' '["id","name","l10n_es_edi_verifactu_test_environment"]' 0
update_record.sh res.company '[<id>]' '{"l10n_es_edi_verifactu_test_environment": true}'

# Desactivar cron de batch (IMPORTANTE — procesa lotes pendientes)
search_records.sh ir.cron '[["cron_name","ilike","verifactu"]]' '["id","cron_name","active"]' 0
update_record.sh ir.cron '[<id>]' '{"active": false}'
```

#### FACe / FACeB2B (Factura Electronica para AAPP)

**Modulo:** `l10n_es_edi_facturae` (Community)
**Aplica a:** Facturas dirigidas a administraciones publicas espanolas

- **Trigger:** Envio explicito de factura via wizard
- **No tiene campo test_env** — no hay modo test
- **Prevencion:** NO ejecutar envio de facturas FACe durante tests

---

### 3.2. Otras Localizaciones con Envio Externo

| Pais | Modulo | Sistema | Riesgo |
|------|--------|---------|--------|
| Italia | `l10n_it_edi` / `l10n_it_edi_sdicoop` | SDI (Agenzia delle Entrate) | Envio de factura electronica |
| Mexico | `l10n_mx_edi` | SAT (PAC) | Timbrado de CFDI |
| Colombia | `l10n_co_edi` | DIAN | Factura electronica |
| Brasil | `l10n_br_edi` | SEFAZ | NF-e |
| Peru | `l10n_pe_edi` | SUNAT | Factura electronica |
| Chile | `l10n_cl_edi` | SII Chile | DTE |
| Arabia Saudi | `l10n_sa_edi` | ZATCA | Factura electronica |

**Estrategia general:** Verificar modulos `l10n_*_edi*` instalados y desactivar crons asociados.

---

### 3.3. Webhooks y Automatizaciones Salientes

#### Deteccion de base.automation con webhooks

```bash
# Buscar automatizaciones activas que usen webhook
search_records.sh base.automation '[["active","=",true],["action_server_ids.state","=","webhook"]]' '["name","model_id","trigger"]'

# Buscar todas las automatizaciones activas
search_records.sh base.automation '[["active","=",true]]' '["name","model_id","trigger","action_server_ids"]'
```

#### Clasificacion
- **ROJA**: Automatizaciones con trigger `on_create` o `on_write` en modelos transaccionales (sale.order, account.move, stock.picking) — PAUSAR obligatorio
- **AMARILLA**: Automatizaciones con trigger `on_time` o `on_unlink` — evaluar caso por caso
- **VERDE**: Automatizaciones en modelos no transaccionales (res.partner, product.product) — generalmente seguras

#### Como Pausar

```bash
# Desactivar automatizacion
update_record.sh base.automation '[<id>]' '{"active": false}'

# Snapshot previo obligatorio
search_records.sh base.automation '[["active","=",true]]' '["id","name","model_id","trigger"]'
```

---

### 3.4. Localizaciones Fiscales Espanolas

#### CRITICO: Integraciones que envian datos a Hacienda

| Modulo | Nombre | Comunicacion | Reversibilidad |
|--------|--------|-------------|----------------|
| `l10n_es_edi_sii` | SII (Suministro Inmediato de Informacion) | AEAT en tiempo real | IRREVERSIBLE — dato comunicado |
| `l10n_es_edi_tbai` | TicketBAI (Pais Vasco) | Hacienda foral | IRREVERSIBLE — dato comunicado |
| `l10n_es_verifactu_oca` | VeriFactu (OCA) | AEAT | IRREVERSIBLE — fuerza hash en diarios |

#### Deteccion

```bash
# Verificar si SII esta instalado y activo
search_records.sh ir.module.module '[["name","in",["l10n_es_edi_sii","l10n_es_edi_tbai","l10n_es_verifactu_oca"]],["state","=","installed"]]' '["name","state"]'

# Verificar si VeriFactu ha forzado hash en diarios
search_records.sh account.journal '[["restrict_mode_hash_table","=",true]]' '["name","type","restrict_mode_hash_table"]'
```

#### Clasificacion: ROJA ABSOLUTA

- **NUNCA** ejecutar flujos contables que publiquen facturas si SII/TicketBAI/VeriFactu esta activo en produccion
- Las facturas publicadas se comunican AUTOMATICAMENTE a Hacienda
- Una vez comunicadas, solo se pueden anular con factura rectificativa (NO borrar)
- El hash fiscal impide `button_draft()` en asientos hasheados

#### Alternativas para Testing

1. **Usar entorno staging** sin modulo fiscal instalado
2. **Desactivar EDI provider** temporalmente (SOLO si el cliente lo autoriza):
   ```bash
   search_records.sh account.edi.format '[["active","=",true]]' '["name","code"]'
   # Desactivar via update si es posible
   ```
3. **Crear diario de test** SIN hash activado:
   ```bash
   create_record.sh account.journal '{"name":"[TEST] Diario Test","type":"sale","code":"TSTS","restrict_mode_hash_table":false}'
   ```
   Usar este diario para todas las facturas de test.

---

## 4. Auditoria Automatica Pre-Ejecucion

Lista completa de checks que se DEBEN ejecutar antes de cualquier test.

### Checklist de deteccion (ejecutar en este orden):

| # | Check | Modelo | Dominio | Accion si encontrado |
|---|-------|--------|---------|---------------------|
| 1 | Payment providers habilitados | `payment.provider` | `[["state","!=","disabled"]]` | Desactivar → `state='disabled'` |
| 2 | Delivery carriers en produccion | `delivery.carrier` | `[["active","=",true],["prod_environment","=",true]]` | Sandbox → `prod_environment=false` |
| 3 | Servidores de correo saliente | `ir.mail_server` | `[["active","=",true]]` | Desactivar → `active=false` |
| 4 | Servidores de correo entrante | `fetchmail.server` | `[["active","=",true]]` | Desactivar → `active=false` |
| 5 | Automatizaciones peligrosas | `base.automation` | `[["active","=",true]]` | Filtrar RED triggers → `active=false` |
| 6 | Webhooks entrantes | `base.automation` | `[["active","=",true],["trigger","=","on_webhook"]]` | Desactivar → `active=false` |
| 7 | Webhooks salientes | `ir.actions.server` | `[["state","=","webhook"]]` | Identificar automatizaciones que los usan |
| 8 | Crons peligrosos | `ir.cron` | `[["active","=",true]]` | Filtrar por nombre → `active=false` |
| 9 | Campanas marketing activas | `marketing.campaign` | `[["state","=","running"]]` | Detener → `action_stop` |
| 10 | Cuentas sociales activas | `social.account` | `[["active","=",true]]` | Desactivar → `active=false` |
| 11 | Cron social activo | `ir.cron` | `[["model_id.model","=","social.post"]]` | Desactivar → `active=false` |
| 12 | Cuentas IAP con SMS | `iap.account` | `[["service_name","=","sms"]]` | Informar (no hay toggle) |
| 13 | Digest emails activos | `digest.digest` | `[["state","=","activated"]]` | Desactivar → `state='deactivated'` |
| 14 | Modulos EDI instalados | `ir.module.module` | `[["name","like","l10n_%_edi%"],["state","=","installed"]]` | Activar test_env o ALERTA |
| 15 | EDI Espana — test_env | `res.company` | Verificar `l10n_es_*_test_env` | Activar → `True` |
| 16 | Cron Veri*Factu | `ir.cron` | `[["cron_name","ilike","verifactu"]]` | Desactivar → `active=false` |
| 17 | Inter-company rules | `res.company` | Verificar campos `intercompany_generate_*` | Desactivar si True |
| 18 | WhatsApp instalado | `ir.module.module` | `[["name","=","whatsapp"],["state","=","installed"]]` | Desactivar cron WA |
| 19 | Mailings en cola | `mailing.mailing` | `[["state","in",["sending","in_queue"]]]` | Informar |
| 20 | Posts sociales programados | `social.post` | `[["state","=","scheduled"]]` | Informar |
| 21 | Base de datos neutralizada | `ir.config_parameter` | `[["key","=","database.is_neutralized"]]` | Si True: IAP ya desactivado |
| 22 | Snailmail cron | `ir.cron` | `[["model_id.model","=","snailmail.letter"]]` | Desactivar → `active=false` |
| 23 | Web Push cron | `ir.cron` | `[["model_id.model","=","mail.push"]]` | Desactivar → `active=false` |

---

## 5. Protocolo de Desactivacion Completo

### Paso 1: Snapshot de estado previo

Capturar el estado de TODAS las integraciones en un JSON:

```json
{
  "timestamp": "2026-03-09T10:00:00Z",
  "integration_snapshot": {
    "automations": [
      {"id": 5, "name": "Enviar email al confirmar venta", "active": true, "trigger": "on_state_set"}
    ],
    "payment_providers": [
      {"id": 1, "name": "Stripe", "state": "enabled"}
    ],
    "delivery_carriers": [
      {"id": 3, "name": "FedEx", "prod_environment": true}
    ],
    "crons_disabled": [
      {"id": 12, "cron_name": "Payment: process transactions", "active": true}
    ],
    "webhooks_incoming": [
      {"id": 8, "name": "Webhook ERP externo", "active": true}
    ],
    "webhooks_outgoing": [
      {"id": 15, "name": "Notificar CRM externo", "state": "webhook", "webhook_url": "https://..."}
    ],
    "marketing_campaigns": [
      {"id": 1, "name": "Welcome series", "state": "running"}
    ],
    "social_accounts": [
      {"id": 1, "name": "Company Facebook", "active": true}
    ],
    "digest_emails": [
      {"id": 1, "name": "Periodic Digest", "state": "activated"}
    ],
    "inter_company_config": [
      {"company_id": 1, "generate_po": true, "generate_so": true}
    ],
    "edi_test_env": [
      {"company_id": 1, "sii_test": false, "tbai_test": false, "verifactu_test": false}
    ]
  }
}
```

### Paso 2: Desactivacion en orden

Orden recomendado de desactivacion (de mayor a menor riesgo):

1. **Payment providers** → `state = 'disabled'`
2. **Delivery carriers** → `prod_environment = false`
3. **EDI test_env** → activar `l10n_es_*_test_env = true` en `res.company`
4. **Mail servers (saliente)** → `active = false`
5. **Fetchmail servers (entrante)** → `active = false`
6. **Crons peligrosos** (email, SMS, WhatsApp, push, digest, snailmail, social, verifactu) → `active = false`
7. **Webhooks entrantes** → `active = false`
8. **Automatizaciones RED/YELLOW** → `active = false`
9. **Marketing campaigns** → `action_stop`
10. **Social accounts** → `active = false`
11. **Digest emails** → `state = 'deactivated'`
12. **Inter-company rules** → desactivar en `res.company`

### Paso 3: Ejecutar flujo

Con todas las integraciones RED neutralizadas y las YELLOW evaluadas.

### Paso 4: Restauracion en orden inverso

Restaurar en orden inverso a la desactivacion:

1. **Inter-company rules** → restaurar config original
2. **Digest emails** → `state = 'activated'`
3. **Social accounts** → `active = true`
4. **Marketing campaigns** → `action_start`
5. **Automatizaciones** → `active = true`
6. **Webhooks entrantes** → `active = true`
7. **Crons** → `active = true`
8. **Fetchmail servers** → `active = true`
9. **Mail servers** → `active = true`
10. **EDI test_env** → restaurar `l10n_es_*_test_env = false`
11. **Delivery carriers** → `prod_environment = true`
12. **Payment providers** → `state = '<estado_original>'`

### Paso 5: Verificacion post-restauracion

```bash
# Verificar que todo esta restaurado
search_records.sh payment.provider '[["state","!=","disabled"]]' '["id","name","state"]' 0
search_records.sh delivery.carrier '[["active","=",true],["prod_environment","=",true]]' '["id","name"]' 0
search_records.sh base.automation '[["active","=",true]]' '["id","name"]' 0
search_records.sh ir.mail_server '[["active","=",true]]' '["id","name"]' 0
search_records.sh ir.cron '[["active","=",true],["model_id.model","in",["mail.mail","sms.sms","whatsapp.message","mail.push","digest.digest","snailmail.letter","social.post"]]]' '["id","name"]' 0

# Verificar EDI restaurado (si aplica)
search_records.sh res.company '[]' '["id","name","l10n_es_sii_test_env","l10n_es_tbai_test_env","l10n_es_edi_verifactu_test_environment"]' 0
```

Comparar con el snapshot inicial. Si hay diferencias, investigar antes de continuar.

---

## 6. Tabla de Referencia Rapida

| Integracion | Modelo | Campo de control | Valor seguro | Restaurar a | Efecto colateral al desactivar |
|-------------|--------|-------------------|-------------|-------------|-------------------------------|
| Automatizaciones | `base.automation` | `active` | `false` | `true` | Ninguno |
| Payment providers | `payment.provider` | `state` | `disabled` | Estado original | Archiva tokens asociados |
| Delivery carriers | `delivery.carrier` | `prod_environment` | `false` | `true` | Usa sandbox en vez de API real |
| Delivery carriers (alt) | `delivery.carrier` | `active` | `false` | `true` | Carrier no disponible en pedidos |
| Webhooks entrantes | `base.automation` | `active` | `false` | `true` | URL webhook deja de responder |
| Webhooks salientes | `base.automation` (padre) | `active` | `false` | `true` | No se envia POST a URL externa |
| Crons | `ir.cron` | `active` | `false` | `true` | Tarea programada no se ejecuta |
| EDI SII | `res.company` | `l10n_es_sii_test_env` | `true` | `false` | Usa endpoint de test de AEAT |
| EDI TicketBAI | `res.company` | `l10n_es_tbai_test_env` | `true` | `false` | Usa endpoint de test foral |
| EDI Veri*Factu | `res.company` | `l10n_es_edi_verifactu_test_environment` | `true` | `false` | Usa endpoint de test de AEAT |
| SMS | `ir.cron` (SMS) | `active` | `false` | `true` | Cola de SMS no se procesa |
| WhatsApp | `ir.cron` (WA) | `active` | `false` | `true` | Cola de WA no se procesa |
| Marketing campaigns | `marketing.campaign` | `state` | `stopped` | `running` | Campana pausada, no envia |
| Social accounts | `social.account` | `active` | `false` | `true` | No se puede publicar |
| Digest emails | `digest.digest` | `state` | `deactivated` | `activated` | No envia digest periodico |
| Inter-company | `res.company` | `intercompany_generate_*` | `false` | `true` | No crea docs automaticos |
| Mail servers | `ir.mail_server` | `active` | `false` | `true` | Ningun email puede salir |
| Fetchmail servers | `fetchmail.server` | `active` | `false` | `true` | No recibe emails |
| Snailmail | `ir.cron` (snailmail) | `active` | `false` | `true` | No envia correo postal |
| Web Push | `ir.cron` (mail.push) | `active` | `false` | `true` | No envia push notifications |
| IAP (general) | N/A | N/A | Verificar neutralized | N/A | Sin toggle directo |

---

## 7. Reglas de Seguridad

1. **NUNCA** ejecutar flujos que publiquen facturas (`action_post`) si hay modulos EDI activos
   en modo produccion (SII, TicketBAI, Veri*Factu, FACe) — el envio a Hacienda es irreversible.
   Si es imprescindible publicar, activar primero `test_env=True` en la empresa.
2. **SIEMPRE** guardar snapshot de integraciones antes de desactivar
3. **SIEMPRE** restaurar integraciones tras el flujo, incluso si el flujo fallo
4. **SIEMPRE** verificar que el snapshot post-restauracion coincide con el pre-desactivacion
5. Si la restauracion de un payment provider falla → no forzar, informar al consultor
   (los tokens podrian haberse perdido)
6. **NUNCA** desactivar crons de contabilidad core (conciliacion, amortizacion) — solo los
   de integraciones externas
7. Si la base de datos esta neutralizada (`database.is_neutralized = True`), las llamadas
   IAP ya estan deshabilitadas — no es necesario desactivar servicios IAP manualmente
8. En caso de duda sobre si una automatizacion es segura → desactivar. Es mejor un falso
   positivo que un envio real no deseado
9. **NUNCA** ejecutar payslip runs (`hr.payslip.run`) si hay modulos de batch payment
   bancario instalados (SEPA, NACHA) — puede generar archivos de pago reales
10. Si hay inter-company rules activas, el test puede crear registros en TODAS las empresas
    vinculadas — desactivar antes de ejecutar
11. Para localizaciones espanolas: **SIEMPRE** verificar el estado de `test_env` de SII,
    TicketBAI y Veri*Factu antes de publicar cualquier factura. Si `test_env=False`, activar
    antes del test y restaurar despues
12. **NUNCA** confiar solo en la desactivacion de servidores de correo para prevenir SMS/WhatsApp —
    estos canales usan IAP/API directamente, no SMTP
