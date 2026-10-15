<!-- verified: v18.0 @ 2026-06 (account_followup EE; recargos negativos; anclas verificadas vs fuente) -->
# Follow-up / Recobro (`account_followup`) — Capacidades (¿Odoo puede…?)

> Eje **consultor**, anclado a fuente. **EE**. Método: [`../../_index/HOW-TO-EXTEND.md`](../../_index/HOW-TO-EXTEND.md). Mecánica en [`core.md`](core.md); trampas en [`gotchas.md`](gotchas.md) (AFG*).

**AFCAP1. ¿Niveles de seguimiento configurables?** — **NATIVO** · baja · `account_followup.followup.line` con `delay` ("Due Days") y `_order='delay asc'`, por compañía.

**AFCAP2. ¿Recordatorio escalado por nivel (email/SMS/carta)?** — **NATIVO (parcial)** · media · `followup.line.send_email` + `send_sms`; la "carta" no es un flag `print_letter` sino impresión PDF vía wizard/`_get_followup_report_pdf`.

**AFCAP3. ¿Cron de envío automático?** — **NATIVO** · baja · `res_partner.py::_cron_execute_followup` (cron diario): procesa niveles `auto_execute` y partners `followup_reminder_type='automatic'`.

**AFCAP4. ¿Estado/nivel de seguimiento por cliente?** — **NATIVO** · baja · `res.partner.followup_status` (in_need_of_action/with_overdue/no_action) + `followup_line_id`; manual vs auto por `followup_reminder_type`.

**AFCAP5. ¿Plantilla de email/SMS por nivel?** — **NATIVO** · baja · `followup.line.mail_template_id` + `sms_template_id`. La carta es un QWeb report fijo (sin plantilla separada).

**AFCAP6. ¿Generar carta PDF de recobro?** — **NATIVO** · baja · `res_partner.py::_get_followup_report_pdf` (`report_followup_print_all`), adjuntada al chatter.

**AFCAP7. ¿Recargos/intereses por demora?** — **NO** · — · no hay campo ni lógica de interest/charge/penalty en el módulo.

**AFCAP8. ¿Responsable del recobro?** — **NATIVO** · baja · `res.partner.followup_responsible_id` (company_dependent); resolución por followup/salesperson/account_manager.

**AFCAP9. ¿Excluir/posponer facturas?** — **CONFIG (parcial)** · media · posponer con `res.partner.followup_next_action_date`; **no** hay campo `blocked`/`exclude` persistente por factura (la exclusión en el envío es por selección de adjuntos en el wizard).

**AFCAP10. ¿Informe de partidas vencidas por cliente?** — **NATIVO** · baja · `account.followup.report` + `res.partner.total_overdue`/`total_due`. Los **aging buckets** propiamente dichos viven en `account_reports` (aged receivable).
