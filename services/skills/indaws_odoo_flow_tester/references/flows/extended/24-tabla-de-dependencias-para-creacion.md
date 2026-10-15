## Tabla de Dependencias para Creacion

| Modelo | Dependencias Previas Requeridas |
|--------|-------------------------------|
| fleet.vehicle | fleet.vehicle.model (required) |
| fleet.vehicle.log.services | fleet.vehicle + fleet.service.type |
| maintenance.equipment | (ninguna estricta, name basta) |
| maintenance.request | maintenance.team (default, pero required) |
| repair.order | stock.picking.type con code='repair_operation' (default) |
| hr.applicant | hr.candidate (required, crear primero) |
| hr.attendance | hr.employee (required) |
| helpdesk.ticket | (name basta, team_id tiene default) |
| quality.check | quality.alert.team + quality.point.test_type |
| quality.alert | quality.alert.team (required) |
| planning.slot | (start/end datetime bastan) |
| approval.request | approval.category (required) |
| approval.category | (name basta) |
| account.asset | account.account (3 cuentas), account.journal |
| sale.order (suscripcion) | sale.subscription.plan + producto con recurring_invoice=True |
| mailing.mailing | mailing.list o modelo destinatario |
| sign.request | sign.template con roles definidos |
| survey.survey | (name basta) |
| purchase.requisition | res.partner (vendor) |
| stock.landed.cost | stock.picking (validado), producto servicio, account.account |
| hr.work.entry | hr.employee + hr.contract (open) + hr.work.entry.type |
| hr.contract | hr.employee |
