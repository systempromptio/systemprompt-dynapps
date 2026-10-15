<!-- verified: v18.0 @ 2026-06 -->
# account_followup Gotchas — Odoo 18.0 Enterprise

## Configuration

**AFG1. delay can be NEGATIVE (pre-due reminders)** — `followup.line.delay` Integer, no constraint > 0. `delay=-5` = remind 5 days BEFORE due date. Impact: not obvious from UI; admins enter negatives by accident → unexpected early reminders. _ES: el cliente recibe el aviso de reclamación de pago antes de que la factura llegue a vencer._

**AFG2. (company_id, delay) unique constraint** — `_sql_constraints['days_uniq']` blocks two lines with same delay in same company. Impact: cannot have parallel "soft" + "legal" follow-ups at same delay. Workaround: 0 + 1 day apart. _ES: no me deja crear dos niveles de seguimiento con los mismos días de retraso en la misma empresa._

**AFG3. (company_id, name) unique constraint** — `_sql_constraints['uniq_name']`. Same name across companies forbidden in MULTI-COMPANY shared DB. Impact: subsidiary using same template names as parent breaks. `copy_data` automatically prefixes copies with delay. _ES: no puedo repetir el nombre de un nivel de seguimiento en otra empresa de la misma base de datos._

**AFG4. auto_execute and create_activity mutually exclusive** — `_onchange_auto_execute`: setting auto_execute=True clears create_activity. Onchange only — direct ORM write keeps both. Impact: scripts may produce inconsistent state where activity scheduled AND email auto-sent. _ES: al activar el envío automático desaparece la actividad programada del nivel de seguimiento._

**AFG5. delay=0 is valid (= reminder ON due date)** — System treats today's due date as `>=0` overdue. Impact: surprising that 0 doesn't mean "disabled". _ES: poner 0 días no desactiva el nivel, manda el recordatorio el mismo día del vencimiento._

## Cron Behavior

**AFG6. cron processes max 1000 partners per company per run** — `res_partner.py → _cron_execute_followup_company` (~L645): `for partner in in_need_of_action_auto[:1000]`. The pre-filter is `followup_status == 'in_need_of_action'` AND `followup_line_id.auto_execute` AND `followup_reminder_type == 'automatic'` (~L643-644). Impact: large customer bases (>1000 in_need partners) need additional manual runs OR cron tweaks. Silent stall — no log when the cap is hit. _ES: cuando hay muchos morosos el proceso automático deja sin avisar a parte de los clientes sin dar error._

**AFG7. cron name xml_id is misleading** — XML id is `ir_cron_auto_post_draft_entry` (copy-paste artefact). Impact: searching by ID confuses developers. Real name is "Account Report Followup; Execute followup". _ES: no encuentro la acción programada de reclamación buscándola por su identificador técnico._

**AFG8. Cron iterates ALL companies regardless of allowed_company_ids** — `_cron_execute_followup` does `for company in env['res.company'].search([])`. Impact: technician restricting to one company in cron's user manually still triggers all. _ES: la reclamación automática se ejecuta en todas las empresas aunque limite el usuario a una sola._

**AFG9. Followup query cache shared per cursor, NOT per company** — `env.cr.cache['res_partner_all_followup']` populated once. `_cron_execute_followup` MUST `cache.pop(...)` between companies. Impact: missing pop → wrong company's data leaked. The current code does pop, but custom overrides forget. _ES: en multiempresa se mezclan los datos de seguimiento de una empresa con los de otra._

**AFG10. UserError in cron swallowed, logged as internal note** — `res_partner.py → _cron_execute_followup_company` (~L645-652) wraps `partner._execute_followup_partner()` in `try/except UserError`, calls `partner._message_log(body=e)` (internal log, NOT a user-visible chatter message) + `_logger.warning`. Impact: bad config (missing email/phone) silently skips that partner; cron keeps going, no crash, no alert. ONLY `UserError` is caught — other exceptions still abort the whole company run. _ES: un cliente sin email o teléfono se salta la reclamación automática sin ningún aviso ni error visible._

## Status Computation

**AFG11. amount_residual <= 0 → no_action_needed** — Even partial overpayment (amount_residual = -10€) hides all overdue invoices. Impact: overpaid customers never receive reminders for parallel new overdue invoices. _ES: un cliente con saldo a favor deja de recibir recordatorios de sus facturas vencidas nuevas._

**AFG12. followup_status excludes liability_payable** — `res_partner.py → _get_unreconciled_aml_domain` (~L205) returns both asset_receivable + liability_payable (feeds `total_all_due`/`total_all_overdue`), but `_get_followup_data_query` status SQL filters `account.account_type = 'asset_receivable'` ONLY (~L426, L441, L458). Impact: vendor balances (liability_payable) feed `total_all_*` but never the followup status / `total_due` / `total_overdue` — only customer receivable matters for dunning. _ES: los saldos de proveedor no cuentan para el estado de reclamación, solo las facturas de cliente._

**AFG13. with_overdue_invoices DOES NOT trigger reminder** — Only `in_need_of_action` flows through cron. `with_overdue_invoices` = waiting for next_action_date. Impact: customer can be 60 days overdue but next_action_date in future → silence. _ES: tengo facturas muy vencidas pero el sistema no envía ningún recordatorio._

**AFG14. amount_residual aggregate doesn't filter by maturity** — SQL `SUM(aml.amount_residual)` includes lines without date_maturity (= not overdue but unreconciled). Impact: total may differ from "true" overdue total in UI. _ES: el total vencido que muestra Odoo no cuadra con las facturas que están realmente vencidas._

## Send Logic

**AFG15. send_email defaults True, send_sms False** — On line create, check both before assuming. Impact: copy of "email-only" template into "sms" config silently sends email too. _ES: al copiar una plantilla de solo email a una de SMS sigue enviando también el correo._

**AFG16. Manual reminder wizard ignores auto_execute** — Wizard always sends regardless of `auto_execute=False`. The flag only governs CRON behaviour. Impact: "manual-only" line still fires from wizard. _ES: un nivel marcado como manual igualmente envía el recordatorio desde el asistente._

**AFG17. join_invoices attaches main_attachment only** — `_get_invoices_to_print` filters `inv.message_main_attachment_id`. Impact: invoices not yet PDF-printed (no attachment) are SILENTLY excluded; reminder body says "see invoices" but no invoices attached. _ES: el correo dice que adjunta las facturas pero llega sin ningún PDF adjunto._

**AFG18. Manual followup wizard filters attachment_ids** — `if options.get('manual_followup'): only print invoices with selected attachments`. Impact: deselecting an invoice's attachment in wizard removes it from envelope. _ES: al desmarcar el adjunto de una factura en el asistente esa factura desaparece del correo._

**AFG19. Followup contact = type='followup' children, fallback billing** — `_get_all_followup_contacts`: looks for child partners with `type='followup'`. If none, uses `address_get(['invoice'])`. Impact: partner without explicit followup contact uses billing address, NOT main contact. _ES: el recordatorio se envía a la dirección de facturación en vez de al contacto principal._

**AFG20. v19 type='followup' REMOVED** — `type` Selection no longer has 'followup' option. Migration: existing `followup` children become orphaned. Impact: old reports finding `type='followup'` return zero in v19. _ES: tras migrar a v19 los informes que buscan contactos de seguimiento salen vacíos._

## Partner / Move Line Sync

**AFG21. aml.followup_line_id assigned via _set_followup_line_on_unreconciled_amls** — `res_partner.py → _set_followup_line_on_unreconciled_amls` (~L197-203) is the `inverse` of the computed `followup_line_id`. Writing `partner.followup_line_id` does NOT store that line on the AMLs — it searches the PREVIOUS line (`delay < current.delay`, order desc, limit 1) and writes THAT onto every `unreconciled_aml.followup_line_id`. Impact: counterintuitive — setting a partner to level 3 stamps its AMLs with level 2; the partner's own level is then recomputed from the AMLs. _ES: al subir un cliente al nivel 3 sus apuntes quedan marcados con el nivel 2 anterior._

**AFG22. _update_next_followup_action_date assigns level by date** — Iterates AMLs and finds eligible levels (`(today - date_maturity).days >= level.delay`). Picks `max(eligible_levels)`. Impact: backdated AML retroactively jumps to high level on next save. _ES: una factura con fecha atrasada salta de golpe a un nivel de reclamación alto al guardar._

**AFG23. unreconciled_aml_ids One2many with readonly=False** — Editable inline in form. Impact: removing AML from list does NOT reconcile it; just visual filter — confuses users. _ES: quito una factura de la lista de apuntes del cliente pero vuelve a aparecer._

**AFG24. _included_unreconciled_aml_max_followup uses amount_residual for "most_delayed"** — `res_partner.py → _included_unreconciled_aml_max_followup` (~L266): `if most_delayed_aml.amount_residual < aml.amount_residual: most_delayed_aml = aml`. Misnomer: it picks the AML with the HIGHEST debt, NOT the oldest/most-delayed by date. `max_delay` (the real days-overdue max) is tracked separately. `most_delayed_aml` then drives `_get_followup_responsible`'s salesperson resolution (~L221-227). Impact: the salesperson chosen is the one on the biggest-amount invoice, not the oldest. v19 renames it `max_amount_aml` (honest name, AFG34). _ES: el comercial asignado a la reclamación es el de la factura más grande, no el de la más antigua._

## Reports & Localisation

**AFG25. report falls back through 3 candidates** — `account_reports.followup_report` → `customer_statement_report` → `partner_ledger_report`. Impact: missing module/ref produces silent fallback to wrong layout. _ES: el PDF de reclamación sale con un diseño o plantilla distinta a la esperada._

**AFG26. Followup PDF rendered with partner.lang context** — `with_context(lang=self.lang or self.env.user.lang)`. Impact: partner without lang inherits user's lang at cron-run time. Cron user must have correct lang. _ES: el recordatorio llega en otro idioma cuando el cliente no tiene idioma configurado._

**AFG27. Manual reminder default attachment_ids = ALL unreconciled main attachments** — `default_get` sets `attachment_ids = Command.set(partner.unreconciled_aml_ids.move_id.message_main_attachment_id.ids)`. Impact: 50 unpaid invoices → 50 PDFs attached to email by default. User must deselect manually. _ES: el recordatorio manual adjunta por defecto todas las facturas pendientes, decenas de PDFs._

## Activities

**AFG28. Activity scheduled on partner, not invoice** — `activity_schedule` on `res.partner`. Impact: salesperson sees "Follow-up" activity on customer card; clicking "Mark Done" does NOT mark invoice as paid. _ES: marcar como hecha la actividad de seguimiento no marca la factura como pagada._

**AFG29. activity_type_id default to "_default_activity_type"** — If line's `activity_type_id` empty, falls back to default. Impact: re-imports without explicit activity type produce generic activities. _ES: las actividades de seguimiento salen con un tipo genérico tras reimportar los niveles._

**AFG30. additional_follower_ids subscribes USERS to PARTNER** — Adds them as followers on partner record (not invoice/activity). Impact: users get all partner notifications, not just followup ones; over-noisy. _ES: los usuarios añadidos al seguimiento reciben todas las notificaciones del cliente, no solo las de cobro._

## v19 Specific (see v19_changes.md)

**AFG31. v19 _sql_constraints replaced by models.Constraint** — New v19 syntax: `_days_uniq = models.Constraint(...)`. Impact: custom inheritance of v18 followup.line that adds constraints needs syntactic update. _ES: al heredar los niveles de seguimiento en v19 fallan las restricciones definidas al estilo v18._

**AFG32. v19 send_email label changed to "Email"** — String label dropped "Send" prefix. Same for SMS. Impact: translations need re-doing; UI tweaks subtle. _ES: en v19 cambian las etiquetas de Email y SMS y hay que rehacer las traducciones._

**AFG33. v19 type='followup' removed entirely** — `type = fields.Selection(selection_add=...)` line deleted. `_complete_name_displayed_types` override gone. Impact: existing followup contacts in v18 DB become `type='contact'` after migration. _ES: tras migrar a v19 los contactos de seguimiento pasan a ser contacto normal._

**AFG34. v19 _get_followup_responsible signature breaks** — `res_partner.py → _get_followup_responsible(self, multiple_responsible=False)` (v19 ~L233) added the kwarg; `most_delayed_aml` replaced by `max_amount_aml` computed inline via `max(...)` over `amount_residual` (v19 ~L241, still highest amount, renamed). When `multiple_responsible=True` it returns ALL salespeople (`activity_schedule` loops them, v19 ~L535). Override compatibility breaks — must add the kwarg. v18 signature is `_get_followup_responsible(self)` (res_partner.py:214), no param. _ES: tras subir a v19 se rompe la personalización que sobrescribía el responsable de la reclamación._

## Per-line Exclusion / Blocking

**AFG35. v18 has NO per-line block flag** — there is no `no_followup`/`blocked` field on `account.move.line` in v18. The ONLY way to exclude a line is the inline-editable `unreconciled_aml_ids` One2many on the partner form (AFG23), which is purely a visual filter on the current computation, NOT persisted. Impact: you cannot durably exclude one disputed invoice from dunning in v18 standard — it reappears on next `_compute_total_due`. (Localizations sometimes add a flag via `_get_followup_data_query_extra_join_conditions`, res_partner.py:477.) _ES: en v18 no puedo excluir de forma permanente una factura en disputa del proceso de reclamación._

**AFG36. v19 ADDS account.move.line.no_followup block flag** — v19 introduces `no_followup` on AML. `_compute_total_due` groups by it (`groupby=[..., 'no_followup']`, v19 res_partner.py:193) and EXCLUDES blocked lines from receivable totals (`if not no_followup:`, ~L201); the status SQL adds `AND line.no_followup IS NOT TRUE` (v19 ~L427, ~L444). Impact: migrating v18→v19, no existing line is blocked (field defaults False) — behaviour unchanged on upgrade, but new persistent exclusion is now possible (replaces the v18 visual-only workaround). `_compute_total_due` depends on `invoice_ids.line_ids.no_followup` (v19 ~L182), so toggling it recomputes due/overdue. _ES: en v19 puedo marcar una factura para que no entre en la reclamación de cobros._

**AFG37. amount_residual SUM in status query is pre-blocking in v18** — `res_partner.py → _get_followup_data_query` (~L412) computes `SUM(aml.amount_residual)` over ALL posted-unreconciled asset_receivable lines, used by the `amount_residual <= 0 → no_action_needed` gate (~L404). In v18 there's no exclusion, so a manually "removed" line (AFG23) still counts toward this SUM. The in_need/overdue LATERAL subqueries additionally require `line.balance > 0` (~L445, L462), so credit lines (payments/refunds on receivable) never trigger a reminder on their own but DO net down the SUM gate. _ES: una factura que quito de la lista sigue contando en el total que decide si se reclama al cliente._

**AFG38. _included_unreconciled_aml_max_followup skips AMLs without date_maturity** — `res_partner.py → _included_unreconciled_aml_max_followup` (~L255) iterates `self.unreconciled_aml_ids.filtered('date_maturity')`. Lines with no maturity date are silently ignored for level/most-delayed computation (but still count in `total_due`, AFG14). Impact: an unreconciled receivable with empty `date_maturity` never advances the followup level. _ES: una factura sin fecha de vencimiento nunca avanza de nivel de reclamación._

**AFG39. only AMLs in env.company.parent_ids tree drive the level** — same method (~L262): `if self.env.company in aml.company_id.parent_ids`. A receivable booked in a sibling/child company outside the current company's parent chain is excluded from `highest_followup_line`/`max_delay`. Impact: in complex multi-company trees the displayed level depends on `allowed_company_ids` context at compute time. _ES: en multiempresa el nivel de reclamación cambia según la empresa que tenga activa._
