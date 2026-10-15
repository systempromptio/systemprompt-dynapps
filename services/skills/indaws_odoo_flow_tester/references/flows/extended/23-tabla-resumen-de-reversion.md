## Tabla Resumen de Reversion

| Modelo | Cancelar | Draft/Reset | Eliminar | Restriccion Principal |
|--------|----------|-------------|----------|----------------------|
| fleet.vehicle | N/A (sin state) | N/A | unlink libre | Ninguna |
| fleet.vehicle.log.services | write state='cancelled' | write state='new' | unlink libre | Ninguna |
| maintenance.equipment | N/A (sin state) | N/A | unlink libre | serial_no unique |
| maintenance.request | archive_equipment_request() | reset_equipment_request() | unlink libre | stage_id ondelete='restrict' |
| repair.order | action_repair_cancel() | action_repair_cancel_draft() | auto-cancela si no draft/cancel | state='done' no cancelable |
| hr.applicant | archive_applicant() (wizard) | reset_applicant() | unlink libre | candidate_id required |
| hr.attendance | N/A (sin state) | N/A | unlink libre (recalcula OT) | No se puede duplicar |
| helpdesk.ticket | N/A (stage-based) | cambiar stage | unlink libre | stage_id ondelete='restrict' |
| quality.check | do_fail() | N/A (no reset) | unlink libre | team_id required |
| quality.alert | N/A (stage-based) | cambiar stage | unlink libre | stage_id ondelete='restrict' |
| planning.slot | N/A | action_unpublish() | unlink libre | end > start, hours >= 0 |
| approval.request | action_cancel() | action_draft() | unlink libre (limpia attachments) | category_id required |
| approval.category | N/A (config) | N/A | unlink libre | approval_minimum constraints |
| account.asset | set_to_cancelled() | N/A (pause via write) | unlink (solo draft) | pause() no funciona via RPC |
| sale.order (suscripcion) | action_cancel() / set_close() | action_draft() (desde cancel) | unlink (draft/cancel) | Usar plan_id (NO recurrence_id) |
| mailing.mailing | action_cancel() → draft | N/A (cancel = draft) | unlink libre | action_draft NO existe |
| sign.request | N/A | N/A | unlink | 1 request_item por rol |
| survey.survey | N/A (sin state) | N/A | unlink libre | Usar action_archive/unarchive |
| purchase.requisition | action_done() | N/A | unlink (draft/cancel) | Metodo es action_confirm (NO action_in_progress) |
| stock.landed.cost | N/A | button_cancel() | unlink (draft) | split_method requerido en lineas |
| hr.work.entry | N/A | N/A | ⚠️ unlink solo draft | Validadas: archivar contrato |
| hr.contract | write state='cancel' | write state='draft' | ⚠️ condicional | state='open' auto-genera work entries |

---

