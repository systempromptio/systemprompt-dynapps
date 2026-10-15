<!-- verified: v18.0 @ 2026-06 (industry_fsm EE + industry_fsm_sale/_stock/_report; anclas vs fuente) -->
# Field Service (`fsm`) — Capacidades (¿Odoo puede…?)

> Eje **consultor**, anclado a fuente. **EE** (sobre `project.task`). Método: [`../../_index/HOW-TO-EXTEND.md`](../../_index/HOW-TO-EXTEND.md). Mecánica en [`core.md`](core.md); trampas en [`gotchas.md`](gotchas.md) (FSMG*).

**FSMCAP1. ¿Tareas de servicio en campo (intervenciones)?** — **NATIVO** · baja · `project.task → is_fsm`; `project.project → is_fsm` (context `fsm_mode`).
**FSMCAP2. ¿App móvil para el técnico?** — **NATIVO** · baja · Scoped App web (`/scoped_app/field-service`, kanban/calendario/lista). No hay APK nativa.
**FSMCAP3. ¿Partes de trabajo / worksheets configurables?** — **NATIVO** (módulo `industry_fsm_report`) · media · plantillas reutilizables + PDF; firma del cliente.
**FSMCAP4. ¿Añadir materiales/productos a la intervención?** — **NATIVO** (módulo `industry_fsm_stock` para stock real) · baja · `action_fsm_view_material` (catálogo) → SOL `task_id`.
**FSMCAP5. ¿Facturar la intervención (tiempo + materiales)?** — **NATIVO** (módulo `industry_fsm_sale`) · media · `project.task → action_create_invoice`; timesheets+materiales → SO → factura.
**FSMCAP6. ¿Generar pedido de venta desde la tarea?** — **NATIVO** (`industry_fsm_sale`) · baja · `_fsm_create_sale_order` (auto al asignar partner/materiales).
**FSMCAP7. ¿Planificación / asignación de técnicos?** — **NATIVO** · media · `user_ids` + `planned_date_begin`/`date_deadline` + `_compute_planning_overlap` (Gantt). Ver [[planning]].
**FSMCAP8. ¿Geolocalización / mapa?** — **NATIVO** · media · `base_geolocalize` + mapa JS + `res.partner → action_partner_navigate` (Google Maps). Sin tracking GPS en tiempo real.
**FSMCAP9. ¿Firma del cliente en sitio?** — **NATIVO** · baja · `project.task → worksheet_signature`/`worksheet_signed_by`.
**FSMCAP10. ¿Cronómetro / registro de tiempo in situ?** — **NATIVO** · baja · botones timer (`_compute_display_timer_buttons`) → timesheet.
**FSMCAP11. ¿Intervenciones recurrentes?** — **NATIVO** · baja · `project.task.recurrence` (copia campos FSM).
**FSMCAP12. ¿Intervenciones multi-día?** — **NATIVO** · baja · `planned_date_begin`/`date_deadline` (sin límite de duración).
**FSMCAP13. ¿Generar tarea FSM desde venta?** — **NATIVO** · media · `sale.order.line → _timesheet_create_task_prepare_values` si `project.is_fsm`.
**FSMCAP14. ¿Productos bajo garantía (precio 0)?** — **NATIVO** (`industry_fsm_sale`) · baja · `project.task → under_warranty` (price_unit=0; sin SO si solo garantía).
**FSMCAP15. ¿Informe/parte PDF de la intervención?** — **NATIVO** (`industry_fsm_report`) · baja · report qweb-pdf "Field Service Report" (incluye firma).
**FSMCAP16. ¿Validar y cerrar desde el móvil?** — **NATIVO** · media · `action_fsm_validate` (estado `fsm_done`, convierte timers a timesheet).
**FSMCAP17. ¿Stock/furgoneta del técnico?** — **NATIVO** (`industry_fsm_stock`) · media · `action_fsm_pick_up` + stock moves; entrega al cerrar.
**FSMCAP18. ¿SLA o prioridades en intervenciones?** — **PARCIAL** · — · usa `project.task → priority` (sin motor SLA en FSM). Para SLA real, vía Helpdesk (ver [[helpdesk]]).
