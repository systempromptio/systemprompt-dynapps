# Module Map — Modelo Odoo → Fichero de referencia

Busca el modelo que necesitas → te dice exactamente qué fichero leer.

## Stock & Logistics

| Modelo | Fichero |
|--------|---------|
| `stock.move` | `functional/stock/models.md` |
| `stock.move.line` | `functional/stock/models.md` |
| `stock.quant` | `functional/stock/models.md` |
| `stock.picking` | `functional/stock/models.md` |
| `stock.picking.type` | `functional/stock/models.md` |
| `stock.location` | `functional/stock/models.md` |
| `stock.route` / `stock.rule` | `functional/stock/rules_deep.md` |
| `stock.valuation.layer` | `functional/stock/account.md` |
| `stock.landed.cost` | `functional/stock/account.md` |
| `stock.warehouse` | `functional/stock/models.md` |
| `stock.warehouse.orderpoint` | `functional/stock/extras.md` |
| `stock.lot` / `stock.package` | `functional/stock/models.md` |
| `procurement.group` | `functional/stock/rules_deep.md` |

## Sales

| Modelo | Fichero |
|--------|---------|
| `sale.order` / `sale.order.line` | `functional/sale/core.md` |
| `sale.order` (subscriptions) | `enterprise/subscription/core.md` |
| `amazon.account` / `amazon.marketplace` / `amazon.offer` (Amazon Connector EE) | `enterprise/sale_amazon/core.md` |
| `loyalty.program` / `loyalty.rule` | `functional/sale/loyalty.md` |
| `product.pricelist` | `functional/sale/core.md` |

## Purchase

| Modelo | Fichero |
|--------|---------|
| `purchase.order` / `purchase.order.line` | `functional/purchase/core.md` |
| `purchase.blanket.order` | `functional/purchase/agreements.md` |

## Accounting

| Modelo | Fichero |
|--------|---------|
| `account.move` / `account.move.line` | `functional/account/core.md` |
| `account.payment` | `functional/account/operations.md` |
| `account.bank.statement` | `functional/account/operations.md` |
| `account.reconcile` | `functional/account/deep.md` |
| `account.tax` | `functional/account/operations.md` |
| `account.journal` | `functional/account/core.md` |
| `account.fiscal.position` | `functional/account/operations.md` |
| SAF-T export/import (`account.general.ledger` handler, `account.saft.import.wizard`; EE, l10n AT/RO/NO/LT/PL — ES no usa) | `enterprise/account_saft/core.md` |

## CRM

| Modelo | Fichero |
|--------|---------|
| `crm.lead` | `functional/crm/core.md` |
| `crm.stage` / `crm.team` | `functional/crm/core.md` |

## Project & Services

| Modelo | Fichero |
|--------|---------|
| `project.project` / `project.task` | `functional/project/core.md` + `task_lifecycle.md` |
| `account.analytic.line` (timesheets) | `enterprise/timesheet/core.md` |
| `planning.slot` | `enterprise/planning/core.md` |

## HR

| Modelo | Fichero |
|--------|---------|
| `hr.employee` / `hr.department` | `functional/hr/core.md` |
| `hr.contract` | `functional/hr/core.md` |
| `hr.leave` / `hr.leave.allocation` | `functional/hr/leaves.md` |
| `hr.attendance` | `functional/hr/attendance.md` |
| `hr.expense` / `hr.expense.sheet` | `functional/hr/expense.md` |
| `hr.appraisal` | `functional/hr/appraisal.md` |
| `hr.payslip` / `hr.salary.rule` | `functional/hr/payroll.md` |
| `hr.applicant` / `hr.candidate` | `functional/hr/recruitment.md` |

## Manufacturing

| Modelo | Fichero |
|--------|---------|
| `mrp.production` | `cross-module/mrp/production.md` |
| `mrp.bom` / `mrp.bom.line` | `cross-module/mrp/production.md` |
| `mrp.workorder` | `enterprise/mrp_workorder/core.md` |
| `quality.point` / `quality.check` | `enterprise/mrp_workorder/quality.md` |

## POS

| Modelo | Fichero |
|--------|---------|
| `pos.session` / `pos.order` | `functional/pos/core.md` |

## Website & eCommerce

| Modelo | Fichero |
|--------|---------|
| `website` / `website.page` | `functional/website/core.md` |
| `website.visitor` | `functional/website/core.md` |
| `sale.order` (cart/checkout) | `functional/website/ecommerce.md` |
| `payment.provider` / `payment.transaction` | `functional/website/ecommerce_deep.md` |

## Mail & Messaging

| Modelo | Fichero |
|--------|---------|
| `mail.message` / `mail.mail` | `functional/mail/outgoing.md` |
| `mail.template` | `functional/mail/outgoing.md` |
| `fetchmail.server` / `mail.alias` | `functional/mail/incoming.md` |
| `mailing.mailing` / `mailing.trace` | `functional/mailing/core.md` |
| `ir.mail_server` | `functional/mailing/mail_server.md` |

## Survey

| Modelo | Fichero |
|--------|---------|
| `survey.survey` | `functional/survey/core.md` |
| `survey.question` / `survey.question.answer` | `functional/survey/core.md` |
| `survey.user_input` / `survey.user_input.line` | `functional/survey/core.md` |
| survey scoring / certification / condicionales | `functional/survey/config_guide.md` |
| `survey.user_input` (applicant_id, slide_partner_id, appraisal_id — bridges) | `functional/survey/integrations.md` |
| `hr.applicant`/`hr.job` (survey_id), `slide.channel` (survey), `hr_appraisal_survey` | `functional/survey/integrations.md` |

## Maintenance

| Modelo | Fichero |
|--------|---------|
| `maintenance.equipment` | `functional/maintenance/core.md` |
| `maintenance.request` | `functional/maintenance/core.md` |
| `maintenance.stage` | `functional/maintenance/core.md` |
| `maintenance.team` | `functional/maintenance/core.md` |
| `maintenance.equipment.category` | `functional/maintenance/core.md` |
| `maintenance.mixin` | `functional/maintenance/core.md` |

## Helpdesk & FSM

| Modelo | Fichero |
|--------|---------|
| `helpdesk.ticket` / `helpdesk.team` | `enterprise/helpdesk/core.md` |
| `project.task` (is_fsm=True) | `enterprise/fsm/core.md` |

## Knowledge (EE)

| Modelo | Fichero |
|--------|---------|
| `knowledge.article` | `enterprise/knowledge/core.md` |
| `knowledge.article.member` (permisos/ACL) | `enterprise/knowledge/core.md` |
| `knowledge.article.favorite` | `enterprise/knowledge/core.md` |
| `knowledge.article.thread` (comentarios anclados) | `enterprise/knowledge/integrations.md` |
| `knowledge.article.stage` / `*.template.category` / `knowledge.cover` | `enterprise/knowledge/core.md` |
| article `website.published.mixin` (`website_knowledge`) | `enterprise/knowledge/integrations.md` |

## Rental (EE — sale_renting / sale_stock_renting)

| Modelo | Fichero |
|--------|---------|
| `sale.order` (is_rental_order, rental_status, is_late) | `enterprise/rental/core.md` |
| `sale.order.line` (is_rental, _generate_delay_line, reservation_begin) | `enterprise/rental/core.md` |
| `product.pricing` (PERIOD_RATIO, _compute_price, _get_best_pricing_rule) | `enterprise/rental/core.md` |
| `sale.temporal.recurrence` (overnight v19) | `enterprise/rental/core.md` |
| `rental.order.wizard(.line)` (pickup/return) | `enterprise/rental/core.md` |
| `sale.rental.report` / `sale.rental.schedule` (schedule v17/v18 only) | `enterprise/rental/core.md` |
| stock bridge (rental_loc, 2-picking, serial flow, availability sweep) | `enterprise/rental/integrations.md` |
| account/pricelist/portal/schedule bridges | `enterprise/rental/integrations.md` |
| v18→v19 breaking changes | `enterprise/rental/v19_changes.md` |

## Intercompany

| Modelo | Fichero |
|--------|---------|
| `res.company` (intercompany fields) | `enterprise/intercompany/core.md` |
| `account.move` (auto_generated) | `enterprise/intercompany/core.md` |
| `sale.order` (auto_generated) | `enterprise/intercompany/core.md` |
| `purchase.order` (auto_generated) | `enterprise/intercompany/core.md` |

## Website Sale Subscription

| Modelo | Fichero |
|--------|---------|
| `product.template` (subscription ecommerce) | `enterprise/website_sale_subscription/core.md` |
| `sale.order` (cart subscription) | `enterprise/website_sale_subscription/core.md` |

## Sign (Digital Signatures)

| Modelo | Fichero |
|--------|---------|
| `sign.template` | `enterprise/sign/core.md` |
| `sign.request` / `sign.request.item` / `sign.request.item.value` | `enterprise/sign/core.md` |
| `sign.item` / `sign.item.type` / `sign.item.role` / `sign.item.option` / `sign.item.radio.set` | `enterprise/sign/core.md` |
| `sign.log` | `enterprise/sign/core.md` |
| `sign.document` / `sign.completed.document` (solo v19) | `enterprise/sign/core.md` |

## Studio (Enterprise — web_studio)

| Modelo | Fichero |
|--------|---------|
| `ir.model` / `ir.model.fields` (state='manual', x_* fields) | `enterprise/studio/core.md` |
| `studio.mixin` / `ir.model.data` (studio=True) | `enterprise/studio/core.md` (`engine.md` §1) |
| `studio.approval.rule` / `studio.approval.rule.approver` / `studio.approval.entry` | `enterprise/studio/core.md` §7 |
| `studio.export.model` / `studio.export.wizard` | `enterprise/studio/integrations.md` §3 |
| `base.automation` (Studio-wrapped) / `ir.actions.server` (Studio code) | `enterprise/studio/integrations.md` §1-2 |
| `ir.ui.view` "Odoo Studio: … customization" / report editor diff views | `enterprise/studio/engine.md` §2,§7 |
| `ir.module.module` `studio_customization` | `enterprise/studio/core.md` §8 |

## Partnership / Partner Assign

| Modelo | Fichero |
|--------|---------|
| `res.partner.grade` | `functional/partnership/core.md` |
| `res.partner.activation` | `functional/partnership/core.md` |
| `res.partner` (grade/assign fields) | `functional/partnership/core.md` |
| `crm.lead` (partner_assigned_id) | `functional/partnership/core.md` |
| `crm.lead.forward.to.partner` (wizard) | `functional/partnership/core.md` |
| `crm.partner.report.assign` (SQL view) | `functional/partnership/core.md` |

## Other Enterprise

| Modelo | Fichero |
|--------|---------|
| `marketing.campaign` / `marketing.activity` | `enterprise/marketing_automation/core.md` |
| `social.post` / `social.account` | `enterprise/social/core.md` |
| `whatsapp.message` / `whatsapp.template` | `enterprise/whatsapp/core.md` |
| `sign.request` / `sign.template` | `enterprise/enterprise_tools/core.md` |

## Account Reports (Enterprise)

| Modelo | Fichero |
|--------|---------|
| `account.report` | `enterprise/account_reports/core.md` |
| `account.report.line` | `enterprise/account_reports/core.md` |
| `account.report.expression` | `enterprise/account_reports/core.md` |
| `account.report.column` | `enterprise/account_reports/core.md` |
| `account.report.external.value` | `enterprise/account_reports/core.md` |
| Custom handler models (per report) | `enterprise/account_reports/core.md` |

## Account Budget (Enterprise)

| Modelo | Fichero |
|--------|---------|
| `budget.analytic` | `enterprise/account_budget/core.md` |
| `budget.line` | `enterprise/account_budget/core.md` |
| `budget.report` (SQL view) | `enterprise/account_budget/core.md` |

## Account Accountant (Enterprise)

| Modelo | Fichero |
|--------|---------|
| `bank.rec.widget` / `bank.rec.widget.line` | `enterprise/account_accountant/core.md` |
| `account.reconcile.model` (enhanced) | `enterprise/account_accountant/core.md` |
| `account.fiscal.year` | `enterprise/account_accountant/core.md` |
| `account.deferred.entry.wizard` | `enterprise/account_accountant/core.md` |
| `account.move` (deferred entries) | `enterprise/account_accountant/core.md` |
| `account.bank.statement.line` (auto-reconcile) | `enterprise/account_accountant/core.md` |

## Quality (Enterprise)

| Modelo | Fichero |
|--------|---------|
| `quality.point` | `enterprise/quality/core.md` |
| `quality.check` | `enterprise/quality/core.md` |
| `quality.alert` | `enterprise/quality/core.md` |
| `quality.alert.team` | `enterprise/quality/core.md` |

## Security & Transversal

| Modelo | Fichero |
|--------|---------|
| `ir.rule` / `ir.model.access` | `cross-module/security/architecture.md` |
| `account.analytic.account` / `account.analytic.plan` | `cross-module/analytic/core.md` |
| `bus.bus` / `discuss.channel` | `cross-module/discuss_bus/core.md` |

## Product / Variants / UoM (cross-module)

| Modelo | Fichero |
|--------|---------|
| `product.template` | `cross-module/product/core.md` |
| `product.product` | `cross-module/product/core.md` |
| `product.category` | `cross-module/product/pricelist_supplier_category.md` |
| `product.tag` | `cross-module/product/core.md` |
| `product.combo` / `product.combo.item` | `cross-module/product/core.md` |
| `product.attribute` | `cross-module/product/variants.md` |
| `product.attribute.value` (PAV) | `cross-module/product/variants.md` |
| `product.template.attribute.line` (PTAL) | `cross-module/product/variants.md` |
| `product.template.attribute.value` (PTAV) | `cross-module/product/variants.md` |
| `product.template.attribute.exclusion` | `cross-module/product/variants.md` |
| `product.attribute.custom.value` | `cross-module/product/variants.md` |
| `uom.uom` | `cross-module/product/uom.md` |
| `uom.category` (v18 only) | `cross-module/product/uom.md` |
| `decimal.precision` | `cross-module/product/uom.md` |
| `product.packaging` (v18 only) | `cross-module/product/uom.md` |
| `product.uom` (v19 new) | `cross-module/product/v19_changes.md` |
| `product.pricelist` | `cross-module/product/pricelist_supplier_category.md` |
| `product.pricelist.item` | `cross-module/product/pricelist_supplier_category.md` |
| `product.supplierinfo` | `cross-module/product/pricelist_supplier_category.md` |
| Service config (`service_tracking`, `service_policy`, `service_type`) | `cross-module/product/core.md` |
| Stock qty fields (`qty_available`, `virtual_available`, `free_qty`, `incoming_qty`, `outgoing_qty`) | `cross-module/product/core.md` |

## Delivery / Carrier

| Modelo | Fichero |
|--------|---------|
| `delivery.carrier` | `functional/delivery/core.md` |
| `delivery.price.rule` | `functional/delivery/core.md` |
| `choose.delivery.carrier` (wizard) | `functional/delivery/core.md` |
| `stock.picking` (carrier fields) | `functional/delivery/core.md` |

## Account Asset (Enterprise)

| Modelo | Fichero |
|--------|---------|
| `account.asset` | `enterprise/account_asset/core.md` |
| `account.asset.group` | `enterprise/account_asset/core.md` |
| `asset.modify` (wizard) | `enterprise/account_asset/core.md` |

## Repair

| Modelo | Fichero |
|--------|---------|
| `repair.order` | `functional/repair/core.md` |
| `stock.move` (repair_line_type) | `functional/repair/core.md` (Parts System section) |
| `stock.move` ↔ `purchase.order` | `functional/repair/integrations.md` |
| `repair.order` ↔ `sale.order` | `functional/repair/integrations.md` |

## Appointment bridges

| Bridge | Fichero |
|--------|---------|
| `appointment.type` ↔ `calendar.booking` + `account.move` (payment) | `enterprise/appointment/integrations.md` |
| `appointment.type` ↔ `crm.lead` (auto-create lead) | `enterprise/appointment/integrations.md` |
| `appointment.type` ↔ `calendar.alarm` SMS (reminders cron) | `enterprise/appointment/integrations.md` |
| `appointment.type` ↔ `website` (publish, snippets, multi-website) | `enterprise/appointment/integrations.md` |
| `appointment.type` ↔ `hr.employee` + `resource.calendar` (working hours) | `enterprise/appointment/integrations.md` |
| `appointment.invite` ↔ `hr.applicant` (recruitment) | `enterprise/appointment/integrations.md` |
| `appointment.type` ↔ Google Calendar (`event_videocall_source='google_meet'`) | `enterprise/appointment/integrations.md` |
| `appointment.type` ↔ Google Reserve (`google.reserve.merchant`) | `enterprise/appointment/integrations.md` |
| `appointment.type` ↔ Microsoft Calendar (Outlook sync) | `enterprise/appointment/integrations.md` |

## Account bridges

| Bridge | Fichero |
|--------|---------|
| `account.payment` ↔ `account.move` (origin_payment_id) | `functional/account/integrations.md` |
| `account.payment.method.line` ↔ `account.journal` (outstanding) | `functional/account/integrations.md` |
| `account.bank.statement.line` ↔ `account.move` | `functional/account/integrations.md` |
| `account.reconcile.model` (4 tipos) | `functional/account/integrations.md` |
| `bank.rec.widget` (EE transient) | `functional/account/integrations.md` |
| `account.batch.payment` (EE SEPA) | `functional/account/integrations.md` |
| `account.partial.reconcile` (exchange diff + CABA) | `functional/account/integrations.md` |
| `account.move.payment_state` workflow | `functional/account/integrations.md` |
| `account.tax` + `account.tax.repartition.line` | `functional/account/integrations.md` |
| `account.fiscal.position` (tax/account mapping) | `functional/account/integrations.md` |
| `account.edi.document` + `account.edi.format` | `functional/account/integrations.md` |
| `account.peppol` (proxy IAP) | `functional/account/integrations.md` |
| `account.avatax` / `account_external_tax` | `functional/account/integrations.md` |
| `l10n_es_edi_sii` bridge highlights | `functional/account/integrations.md` |
| `account.asset` (EE) | `functional/account/integrations.md` |
| `budget.analytic` / `budget.line` (EE) | `functional/account/integrations.md` |
| `account.lock.exception` + lock dates | `functional/account/integrations.md` |
| `account.transfer.model` (EE auto_transfer) | `functional/account/integrations.md` |
| `consolidation.chart` / `consolidation.period` (EE, v17 last) | `enterprise/account_consolidation/` (core/gotchas/integrations/v19_changes) |
| `account.disallowed.expenses.category` (EE; v19→`account.fiscal.category`) | `enterprise/account_disallowed/` (core/gotchas/integrations/v19_changes) |
| `account.move` `release_to_pay` 3-way match (EE) | `enterprise/account_3way_match/` (core/gotchas/integrations/v19_changes) |

## Purchase bridges

| Bridge | Fichero |
|--------|---------|
| `purchase.order` ↔ `stock.picking`/`stock.move` | `functional/purchase/integrations.md` |
| `purchase.order.line` ↔ `mrp.production`/`mrp.bom` (kit phantom) | `functional/purchase/integrations.md` |
| `purchase.order` ↔ `repair.order` | `functional/purchase/integrations.md` |
| `purchase.order` (dropship) ↔ `res.partner` (dest_address_id) | `functional/purchase/integrations.md` |
| `purchase.order` ↔ `purchase.order` (intercompany) | `functional/purchase/integrations.md` |
| `purchase.order` ↔ `account.move` (vendor bills) | `functional/purchase/integrations.md` |
| `account.move`/`account.move.line` (3-way match release_to_pay) | `functional/purchase/integrations.md` |
| `purchase.order.line` ↔ `account.analytic.line` | `functional/purchase/integrations.md` |
| `purchase.order.line` ↔ `stock.valuation.layer` (PD) | `functional/purchase/integrations.md` |
| `purchase.order` ↔ `intrastat.transaction` (EE) | `functional/purchase/integrations.md`; deep dive `enterprise/intrastat/` (core/gotchas/integrations/v19_changes) |
| `purchase.order` ↔ `account.edi.format` (UBL Peppol) | `functional/purchase/integrations.md` |
| `purchase.requisition`/`purchase.requisition.line` | `functional/purchase/integrations.md` |
| `purchase.order` ↔ `sale.order` (sale_purchase) | `functional/purchase/integrations.md` |
| `purchase.order` ↔ `approval.request` (approvals_purchase) | `functional/purchase/integrations.md` |
| `purchase.order` (product matrix variants) | `functional/purchase/integrations.md` |

## Sale bridges

| Bridge | Fichero |
|--------|---------|
| `sale.order` ↔ `stock.move`/`stock.picking` | `functional/sale/integrations.md` |
| `sale.order.line` ↔ `project.project`/`project.task`/`project.milestone` | `functional/sale/integrations.md` |
| `sale.order.line` ↔ `mrp.production`/`mrp.bom` | `functional/sale/integrations.md` |
| `sale.order` ↔ `purchase.order` (service_to_purchase, MTO) | `functional/sale/integrations.md` |
| `sale.order` ↔ `delivery.carrier` | `functional/sale/integrations.md` |
| `sale.order` ↔ `account.move` (invoicing) | `functional/sale/integrations.md` |
| `sale.order.line` ↔ `account.analytic.line` | `functional/sale/integrations.md` |
| `sale.order.line` (margin/cost) | `functional/sale/integrations.md` |
| `sale.order` ↔ `hr.expense` (rebilling) | `functional/sale/integrations.md` |
| `sale.order` ↔ `account.analytic.line` (timesheet billable) | `functional/sale/integrations.md` |
| `sale.order` ↔ `planning.slot` | `functional/sale/integrations.md` |
| `sale.order` ↔ `loyalty.program` | `functional/sale/integrations.md` |
| `sale.order` ↔ `crm.lead` (opportunity_id) | `functional/sale/integrations.md` |
| `sale.order` ↔ `sale.commission` | `functional/sale/integrations.md` |

## Approvals (Enterprise)

| Modelo | Fichero |
|--------|---------|
| `approval.category` | `enterprise/approvals/core.md` |
| `approval.category.approver` | `enterprise/approvals/core.md` |
| `approval.request` | `enterprise/approvals/core.md` |
| `approval.approver` | `enterprise/approvals/core.md` |
| `approval.product.line` | `enterprise/approvals/core.md` |
| `approval.request` ↔ `purchase.order` | `enterprise/approvals/integrations.md` |
| `approval.product.line` (warehouse) | `enterprise/approvals/integrations.md` |
| `mail.activity` (approval_request_id) | `enterprise/approvals/integrations.md` |

## POS — bridges

| Bridge | Fichero |
|--------|---------|
| `pos.order` ↔ `stock.picking`/`stock.move` | `functional/pos/integrations.md` |
| `pos.session` ↔ `account.move`/`account.bank.statement` | `functional/pos/integrations.md` |
| `pos.order` ↔ `sale.order` (`pos_sale`) | `functional/pos/integrations.md` |
| `pos.order` ↔ `restaurant.table`/`restaurant.floor` (`pos_restaurant`) | `functional/pos/integrations.md` |
| `pos.config` ↔ blackbox/fiscal (`pos_blackbox_be`) | `functional/pos/integrations.md` |
| `pos.payment.method` ↔ payment terminals (Adyen/Six/Stripe/Tyro/Razorpay) | `functional/pos/integrations.md` |
| `pos.config` ↔ `pos.self_order` + `pos_self_order_preparation_display` | `functional/pos/integrations.md` |
| `pos.order` ↔ `pos.preparation.display` (KDS) | `functional/pos/integrations.md` |
| `pos.payment.method` ↔ `iot.device` (`pos_iot`) | `functional/pos/integrations.md` |
| `pos.order` ↔ l10n_es VeriFactu/TBAI | `functional/pos/integrations.md` |

## Mail / Discuss — mixin + integrations

| Bridge | Fichero |
|--------|---------|
| `mail.thread` mixin (todos los modelos con tracking) | `functional/mail/integrations.md` |
| `mail.activity.mixin` (modelos con activities) | `functional/mail/integrations.md` |
| `mail.template` ↔ render lang + multi-company | `functional/mail/integrations.md` |
| `mail.activity` / `mail.activity.type` (chaining) | `functional/mail/integrations.md` |
| `mail.alias` + `mail.alias.domain` (v18 multi-company) | `functional/mail/integrations.md` |
| `mail.gateway` (incoming + bounce + In-Reply-To) | `functional/mail/integrations.md` |
| `mail.compose.message` (composer + mass_mail) | `functional/mail/integrations.md` |
| `discuss.channel` + `bus.bus` | `functional/mail/integrations.md` |
| `ir.attachment` ↔ `mail.message` | `functional/mail/integrations.md` |

## Subscription — bridges

| Bridge | Fichero |
|--------|---------|
| `sale.order` (`is_subscription`) ↔ `sale.subscription.plan` | `enterprise/subscription/integrations.md` |
| `sale.order` ↔ `account.move` (recurring invoicing + deferred revenue) | `enterprise/subscription/integrations.md` |
| `sale.order` ↔ `payment.token` (auto-pay) | `enterprise/subscription/integrations.md` |
| `sale.order` ↔ `crm.lead` (renewal pipeline) | `enterprise/subscription/integrations.md` |
| `sale.order` ↔ portal `/my/subscriptions` | `enterprise/subscription/integrations.md` |
| `sale.subscription.report` (MRR/ARR/churn) | `enterprise/subscription/integrations.md` |
| `sale.order` ↔ `stock.picking` (recurring delivery) | `enterprise/subscription/integrations.md` |
| `sale.order` ↔ `pos.order` (`pos_sale_subscription`) | `enterprise/subscription/integrations.md` |
| `sale.order` ↔ `website_sale_subscription` | `enterprise/subscription/integrations.md` |

## Timesheet — bridges

| Bridge | Fichero |
|--------|---------|
| `account.analytic.line` ↔ `project.task` | `enterprise/timesheet/integrations.md` |
| `account.analytic.line` ↔ `hr.employee` (multi-company) | `enterprise/timesheet/integrations.md` |
| `account.analytic.line` ↔ `account.analytic.account` (multi-plan) | `enterprise/timesheet/integrations.md` |
| `account.analytic.line` ↔ `sale.order.line` (`sale_timesheet`) | `enterprise/timesheet/integrations.md` |
| `account.analytic.line` ↔ `hr.expense` (`sale_expense`) | `enterprise/timesheet/integrations.md` |
| `account.analytic.line` ↔ `timesheet_grid` (validation) | `enterprise/timesheet/integrations.md` |
| `account.analytic.line` ↔ `helpdesk.ticket` (`helpdesk_timesheet`) | `enterprise/timesheet/integrations.md` |
| `account.analytic.line` ↔ `industry_fsm` task | `enterprise/timesheet/integrations.md` (+ `enterprise/fsm/extensions_sale.md`) |
| `account.analytic.line` ↔ `hr.leave` (`project_timesheet_holidays`) | `enterprise/timesheet/integrations.md` |

## Planning — bridges

| Bridge | Fichero |
|--------|---------|
| `planning.slot` ↔ `resource.resource`/`hr.employee` | `enterprise/planning/integrations.md` |
| `planning.slot` ↔ `project.task` (`project_forecast`) | `enterprise/planning/integrations.md` |
| `planning.slot` ↔ `sale.order.line` (`sale_planning`) | `enterprise/planning/integrations.md` |
| `planning.slot` ↔ timesheet (`project_timesheet_forecast`) | `enterprise/planning/integrations.md` |
| `planning.slot` ↔ `hr.contract` (`planning_contract`, eliminado v19) | `enterprise/planning/integrations.md` |
| `planning.slot` ↔ `hr.leave` (`planning_holidays`) | `enterprise/planning/integrations.md` |
| `planning.role` ↔ `hr.skill` (`planning_hr_skills`) | `enterprise/planning/integrations.md` |
| `planning.slot.template` (recurrence + copy_previous_week) | `enterprise/planning/integrations.md` |

## MRP Workorder — bridges

| Bridge | Fichero |
|--------|---------|
| `mrp.workorder` ↔ `mrp.production`/`mrp.workcenter` | `enterprise/mrp_workorder/integrations.md` |
| `mrp.workorder` ↔ `hr.employee` (operators + PIN/badge) | `enterprise/mrp_workorder/integrations.md` |
| `mrp.workorder` ↔ `quality.point`/`quality.check` (stage=operation) | `enterprise/mrp_workorder/integrations.md` |
| `mrp.workorder` ↔ `stock_barcode` (tablet scan) | `enterprise/mrp_workorder/integrations.md` |
| `quality.point` ↔ `iot.device` (`mrp_workorder_iot`) | `enterprise/mrp_workorder/integrations.md` |
| `mrp.workcenter` ↔ `maintenance.request` | `enterprise/mrp_workorder/integrations.md` |
| `mrp.workorder` ↔ `mrp.eco` (`mrp_workorder_plm`) | `enterprise/mrp_workorder/integrations.md` |
| `mrp.workorder` ↔ `account.analytic.line` (`mrp_workorder_hr_account`) | `enterprise/mrp_workorder/integrations.md` |
| `mrp.workorder` ↔ `stock.lot` expiry (`mrp_workorder_expiry`) | `enterprise/mrp_workorder/integrations.md` |

## Quality — bridges

| Bridge | Fichero |
|--------|---------|
| `quality.check` ↔ `mrp.production` (in-process) | `enterprise/quality/integrations.md` |
| `quality.check` ↔ `purchase.order` (incoming) | `enterprise/quality/integrations.md` |
| `quality.check` ↔ `stock.picking` (operation checks) | `enterprise/quality/integrations.md` |
| `quality.check` ↔ `iot.device` (`quality_iot`) | `enterprise/quality/integrations.md` |
| `quality.check` ↔ `stock.picking.batch` (`quality_control_picking_batch`) | `enterprise/quality/integrations.md` |
| `quality.alert` workflow (escalation + root cause) | `enterprise/quality/integrations.md` |
| `quality.point` ↔ `quality_control_worksheet` (PDF) | `enterprise/quality/integrations.md` |

## Account Followup — bridges

| Bridge | Fichero |
|--------|---------|
| `res.partner.followup_status` ↔ `account.move.line` (residual) | `enterprise/account_followup/integrations.md` |
| `followup.line` (workflow + delay + sequence) | `enterprise/account_followup/integrations.md` |
| `res.partner` ↔ `mail.template` (multi-language) | `enterprise/account_followup/integrations.md` |
| `res.partner` ↔ portal `/my/payment` | `enterprise/account_followup/integrations.md` |
| `res.partner` ↔ `mail.activity` (CRM escalation) | `enterprise/account_followup/integrations.md` |
| `res.partner` ↔ multi-company (parent_of vs child_of asymmetry) | `enterprise/account_followup/integrations.md` |
| `_cron_execute_followup` (daily batches) | `enterprise/account_followup/integrations.md` |
| `res.partner` ↔ `sms.template` | `enterprise/account_followup/integrations.md` |

## Account Reports — bridges

| Bridge | Fichero |
|--------|---------|
| `account.report` ↔ `account.move.line` (engines: domain/sum/tax_tags/etc.) | `enterprise/account_reports/integrations.md` |
| `account.report` ↔ analytic distribution (multi-eje) | `enterprise/account_reports/integrations.md` |
| `account.report` ↔ `account.account.tag` (tax_tags) | `enterprise/account_reports/integrations.md` |
| `account.report` ↔ multi-company consolidation (branches roll-up) | `enterprise/account_reports/integrations.md` |
| `account.report.line` audit drill-down → AML | `enterprise/account_reports/integrations.md` |
| `account.report` variants/sections (country-specific) | `enterprise/account_reports/integrations.md` |
| `account.return` / `account.return.type` (303/111/347/etc.) | `enterprise/account_reports/integrations.md` |
| `account.report` ↔ currency_table (FX) | `enterprise/account_reports/integrations.md` |
| `account.report.external.value` (manual figures) | `enterprise/account_reports/integrations.md` |

## Stock Barcode — bridges

| Bridge | Fichero |
|--------|---------|
| `stock.picking` ↔ `stock_barcode.client_action` (OWL view) | `enterprise/stock_barcode/integrations.md` |
| `barcode.nomenclature` (GS1 decoding AI 01/10/17/31xx) | `enterprise/stock_barcode/integrations.md` |
| `stock.move.line` ↔ scan (`qty_done` → `picked`) | `enterprise/stock_barcode/integrations.md` |
| `mrp.production` ↔ stock_barcode (finished_lot scan) | `enterprise/stock_barcode/integrations.md` |
| `quality.check` ↔ barcode_trigger | `enterprise/stock_barcode/integrations.md` |
| `bus.bus` ↔ stock_barcode_refresh (multi-device sync) | `enterprise/stock_barcode/integrations.md` |
| `iot.device` (scanner) | `enterprise/stock_barcode/integrations.md` |
| `stock.quant` ↔ inventory adjustment scan | `enterprise/stock_barcode/integrations.md` |
| `stock.picking.batch` ↔ stock_barcode | `enterprise/stock_barcode/integrations.md` |
| Package/expiry/EPC RFID (v19 SGTIN) | `enterprise/stock_barcode/integrations.md` |

## Cron & Reporting (transversal)

| Modelo | Fichero |
|--------|---------|
| `ir.cron` / `ir.cron.trigger` | `cross-module/cron/core.md` + `gotchas.md` (CRONG) |
| `ir.actions.report` | `cross-module/reporting/core.md` + `gotchas.md` (RPTG) |
| `report.paperformat` | `cross-module/reporting/core.md` |
| QWeb PDF / wkhtmltopdf / `web.external_layout` | `cross-module/reporting/core.md` (RPTG) |
