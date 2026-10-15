<!-- verified: v18.0 @ 2026-06 (frontdesk EE; anclas verificadas vs fuente) -->
# Frontdesk (`frontdesk`) — Capacidades (¿Odoo puede…?)

> Eje **consultor**, anclado a fuente. **EE**. Método: [`../../_index/HOW-TO-EXTEND.md`](../../_index/HOW-TO-EXTEND.md). Mecánica en [`core.md`](core.md); trampas en [`gotchas.md`](gotchas.md) (FDG*).

**FDCAP1. ¿Estaciones de recepción configurables?** — **NATIVO** · baja · `frontdesk.frontdesk` (nombre, responsables, compañía, token de kiosko).

**FDCAP2. ¿Registro de visitantes con estados?** — **NATIVO** · baja · `frontdesk.visitor.state` (`planned`/`checked_in`/`checked_out`/`canceled`).

**FDCAP3. ¿Quiosco web público para auto check-in?** — **NATIVO** · media · `controllers/main.py::launch_frontdesk` (ruta `/kiosk/<id>/<token>`, `auth='public'`) + `prepare_visitor_data`.

**FDCAP4. ¿Notificar al anfitrión al llegar?** — **NATIVO** · media · `frontdesk.visitor._notify` (+ `_notify_by_discuss`/`_email`/`_sms`).

**FDCAP5. ¿Campos del visitante configurables?** — **NATIVO** · baja · `frontdesk.frontdesk.ask_phone`/`ask_company`/`ask_email` (`required`/`optional`/`none`).

**FDCAP6. ¿Anfitrión (empleados a visitar)?** — **NATIVO** · baja · `frontdesk.visitor.host_ids` (M2M a `hr.employee`).

**FDCAP7. ¿Ofrecer bebida?** — **NATIVO** · baja · `frontdesk.frontdesk.drink_offer` + `drink_ids` (→ `frontdesk.drink`).

**FDCAP8. ¿Personalización visual del quiosco?** — **NATIVO** · baja · `frontdesk.frontdesk.theme` (light/dark) + `image` + ruta `/frontdesk/<id>/background`.

**FDCAP9. ¿Check-out / registro de salida?** — **NATIVO** · baja · `frontdesk.visitor.action_check_out` (+ `check_out`, estado `checked_out`).

**FDCAP10. ¿Planificar visitas con antelación?** — **NATIVO** · baja · `frontdesk.frontdesk._get_planned_visitors` (`state='planned'`) con quick sign-in en el kiosko.
