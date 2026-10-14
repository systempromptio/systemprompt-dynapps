<!-- verified: v18.0 @ 2026-06 (appointment EE base; pago/sync externo/horario-laboral en módulos aparte, marcados; anclas verificadas vs fuente) -->
# Appointment (`appointment`) — Capacidades (¿Odoo puede…?)

> Eje **consultor**, anclado a fuente. **EE**. Método: [`../../_index/HOW-TO-EXTEND.md`](../../_index/HOW-TO-EXTEND.md). Mecánica en [`core.md`](core.md); trampas en [`gotchas.md`](gotchas.md) (APG*); cambios v19 en [`v19_changes.md`](v19_changes.md).

**APPTCAP1. ¿Tipos de cita configurables?** — **NATIVO** · baja · modelo `appointment.type` (`appointment_type.py`). Duración, categoría, asignación, slots, etc. configurables.

**APPTCAP2. ¿Disponibilidad por usuario según calendario y eventos?** — **NATIVO** (horario laboral vía `appointment_hr`) · media · `appointment_type.py::_slot_availability_is_user_available` / `_slot_availability_prepare_users_values_meetings`. Cruza slots con los `calendar.event` (busy); el matching contra el **horario laboral** del empleado lo añade `appointment_hr`.

**APPTCAP3. ¿Citas con recursos (salas/mesas), no solo usuarios?** — **NATIVO** · media · `appointment.resource` (`appointment_resource.py`) + `appointment.type.schedule_based_on='resources'`. Recursos con capacidad y combinables.

**APPTCAP4. ¿Reserva pública desde web sin login?** — **NATIVO** · baja · `controllers/appointment.py::appointment_type_page` (`@route(auth="public", website=True)`). Todo el flujo de booking es público.

**APPTCAP5. ¿Capacidad por slot / reservas múltiples?** — **NATIVO (modo recursos)** · media · `appointment.resource.capacity` + `appointment_type.py::_slots_fill_resources_availability` (`asked_capacity`). Solo con `schedule_based_on='resources'` y recursos `shareable`.

**APPTCAP6. ¿Antelación mínima y ventana máxima de reserva?** — **NATIVO** · baja · `appointment.type.min_schedule_hours` + `max_schedule_days` (+ `min_cancellation_hours`). Campos nativos del tipo.

**APPTCAP7. ¿Preguntas personalizadas al reservar?** — **NATIVO** · baja · `appointment.question` (`appointment_question.py`, tipos char/text/select/radio/checkbox). Obligatorias o no.

**APPTCAP8. ¿Asignación del staff: elegido vs automático?** — **NATIVO** · baja · `appointment.type.assign_method` (`resource_time` / `time_resource` / `time_auto_assign`). Cliente elige usuario, o elige hora y se auto-asigna.

**APPTCAP9. ¿Enlace de videollamada al confirmar?** — **NATIVO (solo Discuss en base)** · baja · `appointment.type.event_videocall_source` (única opción `('discuss',…)`) + `calendar_event.py::_compute_videocall_redirection`. Google Meet requiere `appointment_google_*`.

**APPTCAP10. ¿Recordatorios y confirmación automática?** — **NATIVO** · baja · `appointment.type.reminder_ids` (`calendar.alarm`) + `appointment_manual_confirmation`; `_compute_appointment_status` default `'booked'`. Confirmación automática salvo activar manual.

**APPTCAP11. ¿Pago al reservar la cita?** — **NO (en el módulo base)** · media · sin refs a payment/transaction en `appointment/`. Lo aportan `appointment_account_payment` / `website_appointment_account_payment` (módulos separados).

**APPTCAP12. ¿Sincronización con calendario externo (Google/Microsoft)?** — **NO (en el módulo base)** · media · sin sync en `appointment/`. Lo aportan `appointment_google_calendar` / `appointment_microsoft_calendar` (módulos separados, presentes en EE18).

## Difusión, CRM y avisos (profundización)

**APPTCAP13. ¿Enlaces de invitación compartibles (varios tipos / operador)?** — **NATIVO** (EE `appointment`) · baja · `appointment/models/appointment_invite.py` (`appointment.invite`, `short_code`) · Genera URLs cortas de reserva que agrupan uno o varios tipos de cita y fijan filtros de staff; se comparten por email/redes.

**APPTCAP14. ¿Generar lead/oportunidad CRM desde la cita?** — **EE-hermano** · media · `appointment_crm/models/appointment_invite.py::opportunity_id` + `calendar_event.py::_compute_opportunity_id` · La reserva crea/enlaza un `crm.lead`; requiere `appointment_crm`.

**APPTCAP15. ¿Recordatorios / avisos por SMS?** — **EE-hermano** · baja · módulo `appointment_sms` (además de los recordatorios por email de APPTCAP10). Requiere créditos IAP de SMS.

**APPTCAP16. ¿Reserva desde Google (Reserve with Google)?** — **EE-hermano** · media · `appointment_google_reserve/models/google_reserve_merchant.py` (`google.reserve.merchant`, vía IAP) · Publica la disponibilidad para reservar directamente desde Google; requiere el módulo + configuración de comercio.
