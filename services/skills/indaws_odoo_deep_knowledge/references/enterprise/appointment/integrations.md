# Appointment — Integrations

Bridges del módulo Enterprise `appointment` con payment, CRM, SMS, website, HR, recruitment y calendar sync externo (Google/Microsoft). Total **9 módulos bridge** en `odoo_enterprise/18/appointment*/`.

Para gotchas de la core ver `gotchas.md` (APG1-25). Aquí solo bridges. Continúa la secuencia: **APG26-50**.

Convención IDs: **APG** continúa numeración. Sub-bridges: `appointment_account_payment` → APG con sufijo P; `_crm` → C; `_sms` → S; `website_appointment` → W; `_hr` → H; `_hr_recruitment` → HR; `_google_calendar` → GC; `_google_reserve` → GR; `_microsoft_calendar` → MC.

---

## 1. appointment ↔ payment (`appointment_account_payment`)

### Modelos y campos
- `appointment.type`: `has_payment_step` (Boolean), `product_id` (M2O `product.product`), `product_currency_id` (related), `product_lst_price` (related).
- **NUEVO** `calendar.booking`: holding state pre-pago. `appointment_type_id`, `partner_id`, `start`/`stop`, `asked_capacity`, `booking_line_ids`, `staff_user_id`, `account_move_id`, `product_id`, `booking_token`, `not_available`, `calendar_event_id`.
- **NUEVO** `calendar.booking.line`: pre-reserva de recursos. `appointment_resource_id`, `calendar_booking_id`, `capacity_reserved`, `capacity_used`.
- `appointment.answer.input`: `calendar_booking_id` (M2O, before linked to event), `calendar_event_id` (required=False override).
- `account.move`: `calendar_booking_ids` (O2M).

### Pipeline
```
visitor → /appointment/<id> form submit
  → appointmentAccountPayment.appointment_form_submit() [controllers/appointment.py:62-112]
  → if has_payment_step AND product.lst_price > 0:
    → calendar.booking.create() — pre-evento sin reservar capacidad
    → booking._make_invoice_from_booking() — account.move draft
    → redirect /payment/pay
  → payment.transaction success → account.move._post(soft=False) [account_move.py:12-16]
  → invoice.posted dispara booking._make_event_from_paid_booking() [calendar_booking.py:137-166]
    → _filter_unavailable_bookings() — chequea si availability cambió entre submit y pago
    → si OK: crea calendar.event(appointment_status='booked'), reserva capacidad ahora
  → redirect /calendar/view/<event_access_token>
```

### Configuración
- `appointment.type.has_payment_step=True` requiere `product_id` (SQL constraint, `appointment_account_payment/models/appointment_type.py:27-31`, `class AppointmentType`).
- Product service tipo `sale_ok=True`.
- Payment providers configurados.

### Gotchas

**APG26P.** Booking collision: capacidad puede cambiar entre form submit y pago confirmado. `_filter_unavailable_bookings()` calcula dinámicamente. Booking marcado `not_available=True` → no se convierte a event tras pago. `_log_booking_collisions()` mete msg en invoice; usuario debe re-bookear. `appointment_account_payment/models/calendar_booking.py:60-94`, `CalendarBooking._filter_unavailable_bookings` y `appointment_account_payment/models/calendar_booking.py:142`, `CalendarBooking._make_event_from_paid_booking`.

**APG27P.** SQL constraint: answer input DEBE estar linked a `calendar_booking_id` O `calendar_event_id` (no ambos, no ninguno). Si conversión booking→event falla parcial → constraint viola. Transfer correcto via `Command.set()`. `appointment_account_payment/models/appointment_answer_input.py:10-18`, `class AppointmentAnswerInput`.

**APG28P.** `calendar.booking` NO reserva capacidad real. La reserva ocurre solo al crear `calendar.event` post-pago. **Race condition**: 2 visitors pagando simultáneamente mismo slot → overbooking posible. No DB lock. `appointment_account_payment/tests/test_calendar_booking.py:19-99`, `AppointmentAccountPaymentTest.test_booking_to_event_on_invoice_paid_resource`.

**APG29P.** Conversión booking→event requiere invoice en estado `payment_state='paid'`. Pago pending/timeout → booking huérfano. `_gc_calendar_booking()` autovacuum limpia >6 meses creación o >2 meses stop. `appointment_account_payment/models/calendar_booking.py:184-192`, `CalendarBooking._gc_calendar_booking`.

**APG30P.** `partner_id` en booking sin `required=True` pero usado en `_make_invoice_from_booking()`. Si null → invoice creation falla. Controller siempre pasa customer desde `_get_customer_partner()`. `appointment_account_payment/models/calendar_booking.py:31`, `CalendarBooking.partner_id` y `appointment_account_payment/models/calendar_booking.py:180`, `CalendarBooking._make_invoice_from_booking`, `appointment/controllers/appointment.py:81`, `AppointmentController.appointment_type_index`.

**APG31P.** `has_payment_step=True` sin `product_id` → constraint error en save. `_compute_product_id` busca default ref `appointment_account_payment.default_booking_product`. Si ref no existe → product_id queda False. `appointment_account_payment/models/appointment_type.py:33-44`, `class AppointmentType`.

**APG32P.** Refund de booking pagado NO está implementado. Cancel invoice (`in_refund`) NO elimina event, NO libera capacidad. Manual workaround: admin unlink event + booking. **Capacidad queda reservada indefinidamente.**

**APG33P.** Guest IDs en booking se PIERDEN al convertir a event. `_make_event_from_paid_booking` usa solo `partner_id + staff_user_id.partner_id` para attendees, ignora `guest_ids`. `appointment_account_payment/models/calendar_booking.py:29`, `CalendarBooking.guest_ids` y `appointment_account_payment/models/calendar_booking.py:79`, `CalendarBooking._filter_unavailable_bookings`.

---

## 2. appointment ↔ CRM (`appointment_crm`)

### Modelos y campos
- `appointment.type`: `lead_create` (Boolean), `lead_ids` (M2M crm.lead computed), `lead_count` (Integer computed).
- `appointment.invite`: `opportunity_id` (M2O `crm.lead`) — pre-linked.
- `calendar.event`: `opportunity_id` (M2O computed+stored).

### Pipeline
```
calendar.event.create() con appointment_type_id
  → @api.create_multi triggera _create_lead_from_appointment() si:
    - appointment_type_id.lead_create=True
    - partner_ids - user_id.partner_id NO vacío (external partners exist)
  → _get_lead_values(partner[:1]) — solo primer partner external
    - name = event.name
    - partner_id = primer external partner
    - type = 'opportunity' (hardcoded)
    - user_id = event.user_id
    - description = _get_attendee_description() — emails + names de attendees
  → crm.lead.create() con context allowed_company_ids = user.company_id
  → activity_schedule('mail.mail_activity_data_meeting', deadline=event.start.date)
  → _link_with_lead(lead): res_model_id=crm.lead, res_id=lead.id, opportunity_id=lead.id
```

### Gotchas

**APG34C.** Lead se crea SOLO si `partner_ids - user_id.partner_id` NO vacío. Internal user + otro staff user = no lead. `appointment/models/calendar_event.py:25`, `CalendarEvent.default_get` y `appointment/models/calendar_event.py:43`, `CalendarEvent.default_get`. Test: `test_no_partner()`.

**APG35C.** Lead se crea en company del **event.user_id**, NO del visitor/customer. `with_company(self.user_id.company_id)`. Multi-company: visitor de company B con organizer de A → lead en A, sales team de B no lo ve. `appointment/models/calendar_event.py:47-48`, `CalendarEvent.default_get`. Test: `test_create_opportunity_multi_company()`.

**APG36C.** Lead.name = `event.name` directo, **sin sanitización**. Emojis, unicode, multiline → lead display issues, CRM kanban roto. `appointment/models/calendar_event.py:62`, `class CalendarEvent`.

**APG37C.** GDPR risk: `_get_attendee_description()` mete emails + names de attendees en `lead.description` (full-text searchable). Sin consentimiento explícito. `appointment/models/calendar_event.py:66`, `class CalendarEvent`.

**APG38C.** Multi-partner event → solo **primer partner** [:1] convertido a lead. Group bookings con N attendees externos → N-1 partners ignorados. `appointment/models/calendar_event.py:44`, `CalendarEvent.default_get`.

**APG39C.** `appointment.invite.opportunity_id` pre-linked propaga a `event.opportunity_id`. Si AMBOS activos (pre-linked + auto-create) → lead nuevo SOBRESCRIBE lead pre-linked. `appointment/models/appointment_invite.py:10-12`, `def api`, `appointment/models/calendar_event.py:14-18`, `def Intervals`.

**APG40C.** `_compute_lead_ids` costoso: `_read_group` en calendar.event + joins crm.lead. 1000+ events → compute slow. `allowed_company_ids` context-dependent. `appointment_crm/models/appointment_type.py:17-33`, `class AppointmentType`.

**APG41C.** Booking pagado en pending state NO crea lead (event no existe aún). Si pago timeout → booking huérfano + sin lead. Cross-bridge gotcha con `appointment_account_payment`.

---

## 3. appointment ↔ SMS (`appointment_sms`)

### Modelos y campos
- Sin modelo nuevo. Solo data XML que agrega `calendar.alarm` con `alarm_type='sms'`.
- Hereda flow de `calendar_sms` module (no en appointment_sms).
- View extension: `appointment_templates_registration.xml` añade hint en phone field.

### Pipeline
```
appointment.type.reminder_ids incluye calendar.alarm(alarm_type='sms')
  → calendar.event creada por booking
  → event.alarm_ids = type.reminder_ids
  → cron calendar_sms scheduled
  → at event.start - alarm.duration:
    → SMS template render con partner_id.mobile
    → SMS gateway send (Twilio/AWS SNS/etc.)
```

### Configuración
Data ref: `appointment_sms.calendar_alarm_data_1h_sms` (1h before, alarm_type='sms', default_for_new_appointment_type=True).

### Gotchas

**APG42S.** Visitor sin `phone` field → SMS falla silenciosamente. Form muestra hint "We will use it to remind you" pero phone NO es required. `appointment/views/appointment_templates_registration.xml:4-8`, `appointment_form`.

**APG43S.** SMS via cron (delay 30min típico) vs email reminders inmediato. SMS llega tarde si cron interval grande. Diferencia execution model: `alarm_type='notification'` (immediate) vs `'sms'` (cron). Confusa para usuarios.

**APG44S.** SMS gateway restricciones por país (provider-dependent). Visitor de país no soportado → SMS falla sin notificar usuario. Website appointment NO valida country pre-booking.

**APG45S.** SMS template language hardcoded o por `company.locale`, NO por `visitor.lang`. Visitor ES con company EN → SMS en EN. `appointment_sms` no override template lang.

**APG46S.** Anonymous partner (guest booking sin login) → `event.partner_id.mobile=NULL` → SMS no se puede enviar. Mitigation: form requirir phone si SMS activo.

**APG47S.** Multiple SMS alarms por event (1h+1day before). Cron procesa cada uno independiente. Cron crash mid-job → algunos SMS enviados, otros no. Sin retry.

**APG48S.** GDPR: NO hay `sms_opt_out` ni consentimiento explícito en appointment_sms. SMS enviado sin opt-in. Mitigation manual via `partner.sms_consent` (otro módulo).

---

## 4. appointment ↔ website (`website_appointment`)

### Modelos y campos
- `appointment.type`: mixins `website.seo.metadata`, `website.published.multi.mixin`, `website.searchable.mixin`. Campos `is_published`, `website_id`, `website_url`.
- `appointment.invite`: mixin `website.published.multi.mixin`, `appointment_type_warning_msg` (Char computed).
- Snippets: `s_appointments`, `s_online_appointment` para website builder.

### Pipeline
```
/appointment GET
  → controller appointment_type_index() [appointment.py:20-65]
  → _appointments_base_domain(): ['|', ('website_id','=',current_website.id), ('website_id','=',False)]
  → _fetch_and_check_private_appointment_types(): valida invite_token
  → si len(available)==1: redirect /appointment/<id>
  → else: render cards o list

/appointment/<id>
  → check operator/resource selection [appointment.py:105-118]
  → render slot picker JS (per appointment.type config)

POST /appointment/<id>/validate
  → reCAPTCHA validation
  → if has_payment_step: bridge payment
  → else: calendar.event create direct
```

### Configuración
- `is_published=True`.
- `website_id` set (si multi-website).
- Slot times (`appointment.slot` records) por type.

### Gotchas

**APG49W.** Domain `'|', (website_id, '=', current_website), (website_id, '=', False)`. Appointment con `website_id` de otro website → invisible para current website incluso para admin. `appointment/controllers/appointment.py:195-197`, `AppointmentController.appointment_type_page`.

**APG50W.** Operator selection view skip si CUALQUIERA: `not active OR schedule_based_on != 'users' OR avatars_display != 'show' OR len(users_possible) <= 1 OR user_selected`. Admin desactiva avatars por SEO → operador selector desaparece silentemente. `appointment/controllers/appointment.py:105-118`, `AppointmentController._prepare_appointments_list_data`.

**APG51W.** Country detection fallback chain: `geoip > user.partner.country > visitor.country`. VPN → geoip stale → wrong timezone display. `appointment/controllers/appointment.py:184-192`, `AppointmentController.appointment_type_page`.

**APG52W.** `appointment_tz` es `required=True` con `default=lambda self: self.env.user.tz` — es un `Selection` plano vía `_tz_get`, NO un campo calculado (no existe compute de tz) — `appointment/models/appointment_type.py:58-60`, `AppointmentType.appointment_tz`. Si el usuario que crea el tipo no tiene `tz` configurado el default es `False` y al ser required hay que elegirlo a mano; los slots se generan SIEMPRE en `appointment_tz` (APG1), no en la tz del visitante. Confuso para visitors en otra zona.

**APG53W.** `appointment.invite` multi-website warning: si `invite.website_id != type.website_id` → `appointment_type_warning_msg` flag → `disable_save_button=True`. Admin NO puede guardar invite. `appointment/models/appointment_invite.py:20-30`, `AppointmentShare._rec_name`.

**APG54W.** `copy_data()` override fuerza `is_published=False` en duplicado. Admin duplica published → copia unpublished. Rationale: prevenir doble booking. **Sorpresa común**. `appointment/models/appointment_type.py:34-39`, `AppointmentType`.

**APG55W.** SEO search indexing: appointments private (con invite_token) se INDEXAN igual via `_search_get_detail()`. `message_intro` (campo intro) podría contener info confidencial. `appointment/models/appointment_type.py:44-81`, `class AppointmentType`.

**APG56W.** Videocall location depende de `request.website.domain` al construir URL. Si admin cambia website domain post-event → videocall link queda con old domain → broken. `website_appointment/tests/test_appointment.py:137-192`, `WebsiteAppointmentTest.test_apt_videocall_location_multi_company_and_multi_website`.

---

## 5. appointment ↔ HR (`appointment_hr`)

### Modelos y campos
- `appointment.type`: `work_hours_activated` (Boolean) — restringe slots a working hours del employee.
- Override `_slot_availability_is_user_available` para chequear `hr.employee.resource_calendar_id`.
- `calendar.event`: `on_leave_partner_ids` computed — partners en leave durante el slot.

### Pipeline
```
appointment.type.work_hours_activated=True + category='anytime'
  → _get_default_range_slots() override [appointment_type.py:17-25]
    → restringe slot generation a hr working hours

slot availability check
  → _slot_availability_prepare_users_values() batch [appointment_type.py:99-184]
    → busca hr.employee de cada staff user (employee_id field, multi-company aware)
    → resource.calendar._work_intervals_batch(resources=[employees])
  → _slot_availability_is_user_available() [appointment_type.py:27-49]
    → checa si slot está dentro de work intervals
    → checa si NO está en hr.leave (resource.calendar.leaves)

calendar.event display
  → _gantt_unavailability() [calendar_event.py:12-85]
  → _compute_on_leave_partner_ids — marca attendees en leave
```

### Gotchas

**APG57H.** `resource_calendar_id.flexible_hours=True` **desactiva** work hours check completo. work_hours_activated=True pero slots disponibles 24/7. Employee part-time con flexible_hours → infinite availability. `appointment/models/appointment_type.py:156`, `AppointmentType.resource_manual_confirmation_percentage`, `AppointmentType.resource_manual_confirmation_percentage`.

**APG58H.** `on_leave_partner_ids` requiere `invalidate_recordset()` para actualizar. Cambios a `resource.calendar.leaves` NO triggean recompute. Si creas leave mid-request → cached calendar muestra disponible. `appointment/tests/test_appointment.py:38`, `AppointmentTest.test_appointment_availability_after_utc_conversion`.

**APG59H.** `_work_intervals_batch(resources=[])` con resources vacío causa `KeyError`. Si `employee.resource_id=None` (employee sin recurso) → crash en slot generation. `appointment/models/appointment_type.py:169`, `AppointmentType`.

**APG60H.** Multi-company employees: busca employee en **todas** las companies si no encuentra en current. `users_to_employees` mapping puede traer employee de company errónea → wrong calendar usado. `appointment_hr/models/appointment_type.py:138-152`, `AppointmentType._slot_availability_prepare_users_values_workhours`.

**APG61H.** `timezone_datetime()` convierte sin validar tz válida. `start/stop` en UTC pero calendar en tz diferente → intervals off-by-X horas. `appointment/models/calendar_event.py:51`, `CalendarEvent.default_get`.

**APG62H.** `work_hours_activated` field INVISIBLE si `schedule_based_on='resources'` (no users). Admin no puede activar work hours para resource-based appointments — feature gating sin feedback. View attrs en `appointment_type_views.xml`.

---

## 6. appointment ↔ HR Recruitment (`appointment_hr_recruitment`)

### Modelos y campos
- `appointment.invite`: `applicant_id` (M2O `hr.applicant`).
- `calendar.event`: `applicant_id` (related, readonly=False, store=True) via `appointment_invite_id.applicant_id`.

### Pipeline
```
HR manager abre hr.applicant form → action "Meeting"
  → contexto pasa default_applicant_id=applicant.id
  → _get_calendar_view_appointment_type_default_context_fields_whitelist() [appointment_type.py:10-15]
    → whitelist incluye 'applicant_id'
  → appointment.invite created con applicant_id pre-filled
  → calendar.event.applicant_id propagado vía related field
```

### Gotchas

**APG63HR.** `applicant_id` propaga SOLO si viene en **context** (`default_applicant_id`), no en URL params. Aplicante click link sin context → applicant_id=null. `appointment/models/appointment_type.py:13`.

**APG64HR.** `appointment.invite.applicant_id` NO crea `hr.applicant` automáticamente. Solo linkea si ya existe. Recruitment manager puede olvidar crear applicant pre-booking. `appointment/models/appointment_invite.py:10`, `def api`.

**APG65HR.** Answers de appointment NO se copian a `applicant.description` automáticamente. Interview question "salary_expectation" → answer en `appointment.answer.input`, recruiter NO lo ve en applicant card. `appointment/models/calendar_event.py:9`, `def _`.

**APG66HR.** Borrar `appointment.invite` → `calendar.event.applicant_id` se vuelve null (related field). Cascade delete desconecta events de applicants históricos. `appointment/models/calendar_event.py:9`, `def _`.

---

## 7. appointment ↔ Google Calendar (`appointment_google_calendar`)

### Modelos y campos
- `appointment.type`: `event_videocall_source` selection_add `'google_meet'`, `users_wo_google_calendar_msg` (Html computed), `connector_google` (Boolean computed).
- `calendar.event`: override `_google_values()` para inyectar `conferenceData.createRequest` con UUID si google_meet activo. `_get_post_sync_values()` extrae `hangoutLink` post-sync.

### Pipeline
```
appointment.type.event_videocall_source='google_meet'
  → user con google_calendar_token (OAuth)
  → calendar.event.create() en Odoo
  → _google_values() override:
    - if google_meet: inject conferenceData.createRequest{requestId: uuid}
    - else: remove conferenceData
  → google_calendar module sync:
    → POST Google Calendar API → evento creado en Google
    → response incluye hangoutLink
  → _get_post_sync_values():
    → videocall_location = hangoutLink
  → frontend renders videocall button
```

### Gotchas

**APG67GC.** `is_google_calendar_synced()` chequea EXISTENCIA del token, NO refresh status. Token expirado pero presente → method retorna True. Sync falla silenciosamente. `appointment/models/appointment_type.py:22-23`, `AppointmentType`.

**APG68GC.** `videocall_redirection` (Discuss fallback) se genera PRE-sync con `access_token`. `videocall_location` (Google Meet URL) solo POST-sync. Frontend muestra link pero apunta a video conference null si sync no se ha completado. `appointment/models/calendar_event.py:22-24`, `class CalendarEvent`.

**APG69GC.** Google rechaza `conferenceData.createRequest` si user sin permiso → `hangoutLink=None`. Evento creado en Odoo pero sin Google Meet link. No hay error visible al usuario final. `appointment/models/calendar_event.py:53`, `CalendarEvent.default_get`.

**APG70GC.** Cambiar `event_videocall_source` de `google_meet` a otro (o vacío) NO limpia Google Meet links de events viejos. Historic events quedan con Meet URLs aunque no debería. Sin unlink hook.

**APG71GC.** `users_wo_google_calendar_msg` es INFO solo, NO bloquea booking. Staff users sin Google sync → appointments creados sin Meet link, admin no detecta. `appointment_google_calendar/models/appointment_type.py:24-29`, `AppointmentType._compute_users_wo_google_calendar_msg`.

---

## 8. appointment ↔ Google Reserve (`appointment_google_reserve`)

Bridge para "Reserve with Google" (Google Search / Maps booking).

### Modelos y campos
- `appointment.type`: `google_reserve_enable` (Boolean), `google_reserve_pending_sync` (Boolean, copy=False), `google_reserve_merchant_id` (M2O), `google_reserve_access_token` (Char default=uuid4().hex).
- **NUEVO** `google.reserve.merchant`: `name`, `business_category`, `phone`, `website_url`, `location_id` (partner_id con country/city/zip/street).

### Pipeline
```
admin habilita google_reserve_enable=True en appointment.type
  → validación: category ∈ {recurring, anytime} + assign_method='time_auto_assign'
  → register_appointment() via IAP → merchant + appointment registered

CRON ir_cron_iap_google_reserve_availabilities (interval=9999 months — NO automático)
  → _google_reserve_upload_feed() sends slots a Google
  → format: {spots_total, spots_open, duration_sec, start_sec, resources.party_size}

Google user reserva en Search → webhook
  → POST /appointment/<apt_id>/<apt_token>/google_reserve/booking/create
  → extracta idempotency_token, user_information, slot
  → _slots_generate() + _slots_fill_users_availability() + _slots_fill_resources_availability()
  → crea calendar.event con access_token = hmac(apt_id, idempotency_token)[:36]
  → marca is_google_reserve=True

modificaciones desde Google (cancel/reschedule/upsize) → endpoints separados con re-validación de slots
```

### Configuración
- `google_reserve_enable=True`.
- `assign_method='time_auto_assign'` (forzado, no revierte).
- `google.reserve.merchant` con location_id completo.
- IAP endpoint configurado.

### Gotchas

**APG72GR.** `idempotency_token` = `access_token`. Mismo token → Odoo retorna event existente (sin duplicate). Si modificas `access_token` manualmente → Google reintentos crean events duplicados. `appointment_google_reserve/controllers/google_reserve.py:140-145`, `GoogleReserveController.google_reserve_booking_create`.

**APG73GR.** `assign_method='time_auto_assign'` FORZADO al activar google_reserve. Si cambias a resource-based después, `_compute_assign_method` NO revierte → contradictorio. `appointment_google_reserve/models/appointment_type.py:56-61`, `AppointmentType._compute_assign_method`.

**APG74GR.** CRON `ir_cron_iap_google_reserve_availabilities` con `interval_number=9999 months` — **NUNCA corre automático**. Solo triggered on-demand. Si Google no triggerya CRON → slots invisibles en Google Reserve. `appointment_google_reserve/data/ir_cron_data.xml:15-17`, `ir_cron_iap_google_reserve_availabilities`.

**APG75GR. CORREGIDO — el sumatorio NO está donde decía.** `_google_reserve_get_availabilities` (`appointment_google_reserve/models/appointment_type.py:154-199`, `AppointmentType._google_reserve_get_availabilities`) NO toca `min_schedule_hours`: pasa `reference_date=start_time` a `_slots_generate`, y es ahí, en el core, donde se desplaza el primer slot (`appointment/models/appointment_type.py:630-632`, `AppointmentType._slots_generate`) — y solo si `ref_start <= now + min_schedule_hours`. Efecto neto para Google Reserve: con `min=2h` los slots de la próxima hora no se publican, pero es el comportamiento normal de `min_schedule_hours`, no un off-by-one del feed. Verificar en v19 antes de asumir lo contrario.

**APG76GR.** Party size logic en `_google_reserve_format_slot_availabilities_resources()` IGNORA `linked_resource_ids` (combinables). 2 mesas de 2 spots combinables → sends `spots=2`, no `spots=4`. `appointment/models/appointment_type.py:274-339`, `AppointmentType._compute_location`.

**APG77GR.** `google_reserve_pending_sync=True` se marca pero CRON puede fallar silenciosamente (sin retry automático). Type queda "pending_sync" sin feedback. `appointment/models/appointment_type.py:92-136`, `AppointmentType.avatars_display`.

**APG78GR.** Borrar `calendar.event` Odoo creado vía Google Reserve NO notifica Google. Solo actualiza al recibir webhook `status='CANCELED'`. Odoo unlink → Google mantiene booking activo. `appointment_google_reserve/controllers/google_reserve.py:346-351`, `GoogleReserveController.google_reserve_booking_update`.

---

## 9. appointment ↔ Microsoft Calendar (`appointment_microsoft_calendar`)

### Modelos y campos
- `appointment.type`: `connector_microsoft` (Boolean computed) — disponibilidad módulo + tenant válido.
- Delegated a módulo core `microsoft_calendar` para sync real.

### Pipeline
Similar a Google Calendar pero via Microsoft Graph API (Outlook). Sync bidireccional para event basics (date, time, title). NO sincroniza appointment-specific fields (answers, resources).

### Gotchas

**APG79MC.** `check_synchronization_status()['microsoft_calendar']` puede retornar `'missing_credentials'` aunque token exista. Token expirado no invalida immediate. User ve `connector_microsoft=True` pero sync silenciosamente falla. `appointment/models/appointment_type.py:23`, `AppointmentType._name`.

**APG80MC.** Sync NO es bidireccional para appointment-specific fields. Modificar `appointment.answer.input` en Odoo → no aparece en Outlook event. Solo sync basics. `appointment/__manifest__.py:8`.

**APG81MC.** `microsoft_calendar_sync_paused` global pausa **TODAS** las syncs (no solo manual). Admin pausa → appointments creados pero invisibles en Outlook hasta resume. `appointment/models/appointment_type.py:24`, `AppointmentType._description`.

**APG82MC.** Multi-tenant Microsoft: staff users de tenants diferentes en mismo appointment type → solo tenant donde se crea event lo sincroniza. User en otro tenant no ve appointment en su Outlook. `appointment/models/appointment_type.py:22`, `AppointmentType`.

---

## Matriz de interacción cross-bridge

| Bridge A | Bridge B | Interacción |
|---|---|---|
| **payment** | **CRM** | Booking→Event (post-pago) dispara lead creation. Si pago timeout → no event → no lead (APG41C). |
| **payment** | **SMS** | Event creado post-pago aplica SMS alarms. Si phone no capturado → SMS falla (APG42S). |
| **payment** | **website** | Form submit website → booking payment → event. Si website_id mismatch → booking inaccesible. |
| **CRM** | **website** | `appointment.invite.opportunity_id` pre-linked (website form context) propaga a `event.opportunity_id` y NO crea lead nuevo si ya existe (APG39C). |
| **CRM** | **HR_recruitment** | Mutuamente excluyentes: lead_create vs applicant_id. Si AMBOS activos → orden de propagación indeterminado. |
| **HR** | **website** | `work_hours_activated` invisible si `schedule_based_on='resources'` en website form (APG62H). |
| **google_calendar** | **payment** | event_videocall_source=google_meet con has_payment_step=True → Meet link generated POST-pago (APG68GC). |
| **google_reserve** | **payment** | Google Reserve bookings NO soportan has_payment_step (incompatibilidad design). |
| **microsoft_calendar** | **HR** | Multi-tenant + multi-company → sync confusa entre HR working hours + Microsoft tenants (APG82MC + APG60H). |

---

## v19 — Cambios bridges

Cobertura completa de cambios v19 en `v19_changes.md` (core APG24-25). Bridges v19 specific:

- **appointment_account_payment v19**: `calendar.booking` model refactor menor, mismo flow.
- **appointment_crm v19**: nuevo helper `_get_activity_deadline_from_start(start, allday)` (v18 ya existía).
- **appointment_google_reserve v19**: nuevo CRON `_cron_send_google_reserve_availabilities` puede correr más automático que v18 (`interval_number` reasonable).
- **website_appointment v19**: integrations con nuevos `assignment_method` / `is_date_first` (v19 core refactor APG13).
- **appointment_microsoft_calendar v19**: sync mejorado para event title, sigue NO bidireccional para answers (APG80MC).

---

## Configuración mínima recomendada por caso de uso

### Médico / Profesional individual con cita online + pago + recordatorio

```yaml
appointment.type:
  category: recurring
  schedule_based_on: users
  staff_user_ids: [doctor_user]
  work_hours_activated: True  # → appointment_hr
  has_payment_step: True       # → appointment_account_payment
  product_id: consulta_product
  event_videocall_source: google_meet  # → appointment_google_calendar
  reminder_ids: [calendar_alarm_24h, calendar_alarm_data_1h_sms]  # → appointment_sms
  is_published: True            # → website_appointment

hr.employee:
  resource_calendar_id: standard_8h_calendar
  flexible_hours: False  # NO 24/7 disponible

res.users:
  google_calendar_token: <oauth>
```

### Restaurante con Reserve with Google + multi-mesa

```yaml
appointment.type:
  schedule_based_on: resources
  resource_ids: [mesa_1, mesa_2, mesa_combinada]  # mesa_combinada linkable
  assign_method: time_auto_assign  # FORZADO por google_reserve
  google_reserve_enable: True
  google_reserve_merchant_id: restaurant_merchant
  category: recurring
  has_payment_step: False  # Google Reserve incompatible con pago

google.reserve.merchant:
  business_category: RESTAURANT
  location_id: <partner with full address>
```

### Entrevista de reclutamiento con sync Outlook

```yaml
appointment.type:
  schedule_based_on: users
  staff_user_ids: [recruiter_user]
  category: anytime  # invite-only para applicants
  
appointment.invite:
  applicant_id: <hr.applicant>  # vía context default_applicant_id

# Recruiter user con OAuth Microsoft → appointment_microsoft_calendar sync auto
```

---

## Cross-refs activos

- `enterprise/appointment/core.md` — modelos base, ciclo de vida.
- `enterprise/appointment/gotchas.md` — APG1-25 core gotchas.
- `enterprise/appointment/v19_changes.md` — refactor v19.
- `functional/website/integrations.md` — bridge website_appointment.
- `functional/crm/integrations.md` — appointment_crm side CRM.
- `enterprise/planning/gotchas.md` — solapamientos con planning slots.
- `functional/account/integrations.md` — bridge payment.transaction (cross-ref desde APG26P-33P).
