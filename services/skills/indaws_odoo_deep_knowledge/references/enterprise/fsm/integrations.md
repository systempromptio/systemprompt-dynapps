# FSM — Cross-module integrations

Cómo `industry_fsm` se conecta con el resto del ecosistema Enterprise: planning gantt, helpdesk, `timer.timer`, subscriptions, documents, sign, marketing automation, sale y stock. Este fichero cubre solo los **bridges**; los módulos FSM-internos (`industry_fsm_sale`, `_stock`, `_report`) tienen su propio fichero. Para el flujo extremo a extremo no se duplica aquí: ver [[playbooks/fsm_end_to_end]].

---

## 1. Bridges — catálogo

Todos Enterprise. Auto-install salvo donde se indique.

| Bridge | Depends | Conecta | Auto-install |
|--------|---------|---------|--------------|
| `project_enterprise` | `project` | Gantt view base + overlap math | dependencia directa de FSM |
| `timesheet_grid` | `hr_timesheet` | Timer UI + rounding ICP | dependencia directa |
| `project_forecast` | `planning` + `project` | `planning.slot.project_id` (slot ↔ project) | sí |
| `helpdesk_fsm` | `helpdesk` + `industry_fsm` | ticket → FSM task | sí |
| `industry_fsm_sale` | `sale_timesheet_enterprise` | SO ↔ task, materiales, facturación | sí |
| `industry_fsm_stock` | `sale_stock` | lotes/series, pickings | sí |
| `industry_fsm_report` | — | worksheet + firma + PDF | sí |
| `industry_fsm_sale_subscription` | `project_sale_subscription` | tasks recurrentes por suscripción | sí |
| `industry_fsm_sms` | `project_sms` | SMS por stage | sí |
| `industry_fsm_repair` | `repair` | bridge de solo ACL (sin workflow) | sí |
| `documents_fsm` | `documents` | desactiva Documents en proyectos FSM | sí |
| `timer` | — | modelo `timer.timer` (base del cronómetro) | dependencia |

Marketing automation: **sin bridge nativo** (ver §9).

---

## 2. Diagrama de ecosistema

```
                          industry_fsm (core)
                                  │
   ┌──────────────┬──────────────┼──────────────┬───────────────┐
   │              │              │              │               │
project_enterprise timesheet_grid  hr_timesheet  base_geolocalize  timer
 (gantt base)    (rounding ICP)  (TS backend)   (geocoding opt)  (timer.timer)
   │
   └─ project_forecast ── planning.slot.project_id

   Auto-installed bridges (si dependencias presentes):
   ─ industry_fsm_sale            ← sale_timesheet_enterprise   (SO ↔ task)
   ─ industry_fsm_stock           ← sale_stock                  (lot/serial)
   ─ industry_fsm_report          ←                             (worksheet+PDF)
   ─ industry_fsm_sale_report     ← _sale + _report
   ─ industry_fsm_sale_subscription ← project_sale_subscription
   ─ industry_fsm_sms             ← project_sms
   ─ industry_fsm_repair          ← repair                      (solo ACL)
   ─ helpdesk_fsm                 ← helpdesk + industry_fsm
   ─ documents_fsm                ← documents                   (desactiva docs)

   sign  ──╳── worksheet_signature   (NO integrados; sistemas paralelos)
   marketing_automation ──╳── project.task   (sin bridge; cableado manual)
```

---

## 3. Modelos clave por bridge

Tabla formal de campos que materializa cada bridge. Tipo: M2O/O2M/Char/Bool/Int/Float. "comp" = computed.

| Bridge | Modelo | Campo | Tipo | Apunta a | Definición (path:line) | Notas |
|--------|--------|-------|------|----------|------------------------|-------|
| project_forecast | `planning.slot` | `project_id` | M2O comp/store | `project.project` | `project_forecast/models/project_forecast.py:15-18`, `PlanningShift.project_id` | `_compute_project_id` desde `template_id.project_id`; editable, `check_company` |
| helpdesk_fsm | `project.task` | `helpdesk_ticket_id` | M2O | `helpdesk.ticket` | `helpdesk_fsm/models/project_task.py:10`, `Task.helpdesk_ticket_id` | readonly, `index='btree_not_null'` |
| helpdesk_fsm | `project.task` | `display_helpdesk_ticket_button` | Bool comp | — | `industry_fsm_sale/models/project_task.py:13`, `Task` | filtra por acceso del portal user |
| helpdesk_fsm | `helpdesk.ticket` | `fsm_task_ids` | O2M | `project.task` | `helpdesk_fsm/models/helpdesk_ticket.py:10`, `HelpdeskTicket.fsm_task_ids` | inverse `helpdesk_ticket_id`, domain `is_fsm=True` |
| helpdesk_fsm | `helpdesk.ticket` | `fsm_task_count` | Int comp | — | `helpdesk_fsm/models/helpdesk_ticket.py:11`, `HelpdeskTicket.fsm_task_count` | — |
| helpdesk_fsm | `helpdesk.ticket` | `use_fsm` | Bool related | — | `helpdesk_fsm/models/helpdesk_ticket.py:9`, `HelpdeskTicket.use_fsm` | related a `team_id.use_fsm` |
| helpdesk_fsm | `helpdesk.team` | `fsm_project_id` | M2O comp/store | `project.project` | `helpdesk_fsm/models/helpdesk_team.py:9-10`, `HelpdeskTeam.fsm_project_id` | domain `is_fsm=True`; fallback `min(id)` |
| sale (CE base) | `project.task` | `sale_order_id` | M2O comp/store | `sale.order` | `sale_project/models/project_task.py:25`, `ProjectTask.sale_order_id` | **definido en CE `sale_project`**, no en `industry_fsm_sale` |
| sale (CE base) | `sale.order.line` | `task_id` | M2O | `project.task` | `sale_project/models/sale_order_line.py:18`, `SaleOrderLine.task_id` | tarea generada/enlazada por SOL |
| industry_fsm_sale | `project.task` | `under_warranty` | Bool | — | `industry_fsm_sale/models/project_task.py:44`, `Task.under_warranty` | invisible salvo `is_fsm AND allow_material` (FSMG24) |
| industry_fsm_sale | `project.task` | `material_line_product_count` | Int comp | — | `industry_fsm_sale/models/project_task.py:35`, `Task.material_line_product_count` | nº líneas material; series inflan (FSMG35) |
| industry_fsm_sale | `project.task` | `material_line_total_price` | Float comp | — | `industry_fsm_sale/models/project_task.py:36`, `Task.material_line_total_price` | diverge del importe factura si bloqueada (FSMG36) |
| industry_fsm_stock | `sale.order.line` | `fsm_lot_id` | M2O | `stock.lot` | `industry_fsm_stock/models/sale_order_line.py:10`, `SaleOrderLine.fsm_lot_id` | domain `product_id`; series → 1 SOL/serial (FSMG35) |
| industry_fsm_stock | `product.product` | `quantity_decreasable_sum` | Int comp | — | `industry_fsm_stock/models/product.py:15`, `ProductProduct.quantity_decreasable_sum` | tope inferior al decrementar qty (FSMG32) |
| industry_fsm_stock | `project.task` | `stock_move_customer_product_count` | Int comp | — | `industry_fsm_stock/models/project_task.py:10`, `Task.stock_move_customer_product_count` | — |
| timer | `timer.timer` | `res_model` / `res_id` / `user_id` | Char/Int/M2O | genérico | `timer/models/timer.py:14-16`, `TimerTimer.res_model` | `UNIQUE(res_model,res_id,user_id)` (FSMG11) |
| timer | `timer.timer` | `timer_start` / `timer_pause` | Datetime | — | `timer/models/timer.py:11-12`, `TimerTimer.timer_start` | base de `_get_minutes_spent` |
| documents_fsm | `project.project` | `use_documents` | Bool comp/store | — | `documents_fsm/models/project_project.py:17`, `ProjectProject.use_documents` | `_compute_use_documents` = `not is_fsm` (FSMG53) |

`industry_fsm_sale_subscription` no añade campos: solo override de `action_fsm_view_material` (13 líneas, ver §6).

---

## 4. Planning ↔ FSM

### Sin link directo
**No existe `planning_slot_id`** en `project.task`. Slot y task operan a niveles distintos:
- `planning.slot` = disponibilidad por `resource.resource` (vía `user_id`).
- `project.task.user_ids` = la asignación.

### Bridge vía `project_forecast`
`project_forecast/models/project_forecast.py:15-18`, `PlanningShift.project_id` añade `project_id` a `planning.slot` (computed/store, editable). Permite scope del slot a un proyecto y que el gantt superponga slots y tasks.

### Allocated-hours overlap math
`industry_fsm/models/project_task.py:476-497`, `Task._allocated_hours_per_user_for_scale` — `_allocated_hours_per_user_for_scale()`: las FSM tasks calculan horas asignadas por usuario desde `[planned_date_begin, date_deadline]` contra el calendario del recurso. Tasks multi-día (≥24h) usan intersección de work-interval, no totales ingenuos. Los conflictos solo emiten warning; sin auto-reschedule (FSMG12).

### Date sync
Las FSM tasks **saltan el auto-scheduling estándar del gantt**. `project_enterprise/models/project_task.py:437`, `Task.write` difiere explícitamente a `industry_fsm` cuando `fsm_mode=True`. El requisito de fecha bidireccional bloquea escrituras parciales: FSMG2 (ambas fechas obligatorias, escribir una nulifica las dos) y FSMG56 (bulk-edit borra el schedule).

### Borrado de slot
`planning.slot.recurrency_id` usa `ondelete='set null'` (`planning/models/planning.py:115`, `Planning.recurrency_id`). Borrar un slot no propaga a ninguna FSM task (ningún campo lo referencia). La task queda planificada sin visibilidad de disponibilidad de recurso: FSMG50.

---

## 5. Helpdesk ↔ FSM — `helpdesk_fsm`

Bridge para escalar un `helpdesk.ticket` a intervención FSM. Campos: ver §3.

### Auto-resolución de `fsm_project_id`
`_compute_fsm_project_id()` (`helpdesk_fsm/models/helpdesk_team.py:13-28`, `HelpdeskTeam._compute_fsm_project_id`): busca proyectos FSM en la company del team y coge el de **menor id** (`min(res['ids'])`, l.28). Si no hay → False, dropdown vacío en el wizard sin error: FSMG51.

### Convert ticket → task
`helpdesk.ticket.action_generate_fsm_task()` abre el wizard `helpdesk.create.fsm.task` con `partner_id`, `name`, `project_id` (de `team_id.fsm_project_id`) prefijados. La variante portal auto-crea partner desde `partner_name`/`partner_email` si falta.

### Sin herencia de SLA
Las SLAs son específicas de helpdesk. La FSM task no hereda ningún timer SLA: FSMG55.

### Feedback de cierre de task
`helpdesk_fsm/models/project_task.py:38-65`, `Task.write` — al llegar la FSM task a `fsm_done=True` o `state='1_done'`, postea mensaje en el ticket enlazado con subtype `helpdesk_fsm.mt_ticket_task_done` (o `mt_ticket_task_canceled`).

### Visibilidad portal
`helpdesk_fsm/models/project_task.py:13`, `Task.display_helpdesk_ticket_button` y `helpdesk_fsm/models/project_task.py:21`, `Task._compute_display_helpdesk_ticket_button` — `display_helpdesk_ticket_button` filtra por acceso del portal user; el botón navega al ticket original.

### Report bridge
`helpdesk_fsm_report` (depende de `helpdesk_fsm + industry_fsm_report`) — pre-carga el worksheet template al escalar.

---

## 6. Subscription ↔ FSM — `industry_fsm_sale_subscription`

Bridge mínimo (13 líneas). Construido sobre `project_sale_subscription`.

### Creación de task recurrente
`project_sale_subscription/models/sale_order_line.py` — al vender un producto de suscripción (`recurring_invoice=True`) con proyecto FSM, `_timesheet_create_task()` crea un `project.task.recurrence` con `repeat_interval` igual al periodo de facturación. **Cron**: el estándar `project.task.recurrence._cron_create_recurring_tasks` (no FSM-specific) crea la siguiente ocurrencia.

### Filtrado FSM-specific
`industry_fsm_sale_subscription/models/project_task.py:11`, `Task.action_fsm_view_material` — `action_fsm_view_material()` añade `('recurring_invoice','=',False)` al domain de productos. El técnico no puede añadir por error productos de suscripción como material puntual. (v18 usa `expression.AND`; v19 usa `Domain.AND` — ver §11.)

### Coexistencia
`project.task.recurrence` y la renovación de suscripción son dos mecanismos separados que coexisten → tasks duplicadas si ambos aplican a la misma SOL: FSMG29 (dos mecanismos) + FSMG52 (cron mid-month genera tasks off-cycle).

---

## 7. Documents ↔ FSM — `documents_fsm`

`documents_fsm/models/project_project.py:13-22`, `ProjectProject.default_get`:
- Los proyectos FSM **desactivan Documents**: `use_documents` se fuerza a False (`_compute_use_documents` = `not is_fsm`, store, l.17,20-22), y `default_get` aplica lo mismo en create (l.13-14).
- Racional: FSM usa PDFs de worksheet y firmas adjuntas, no carpetas de documentos.
- Los links de documento existentes en tasks persisten en BD pero la UI los oculta: FSMG53.

---

## 8. Sign ↔ FSM signature

El módulo `sign` (e-signature formal sobre contratos) **no está integrado** con `worksheet_signature` de FSM (captura on-site en canvas). Son dos sistemas paralelos:
- `sign` usa `sign.request` + `sign.template` con audit trail completo, cadenas de firmantes y sellado por certificado.
- FSM guarda un `Binary` crudo con un "signed by" de texto libre. Sin certificado, sin cadena auditable.

Para firmas legalmente vinculantes hay que integrar `sign` por separado: FSMG54.

---

## 9. Marketing automation ↔ FSM

Sin bridge nativo. Las notificaciones SMS vienen de `industry_fsm_sms` (stage-driven, no automation-driven; ver FSMG42 para el matiz `mobile` vs `phone`). Para flujos complejos (drip, segment-based reminders) hay que cablear `marketing_automation` al modelo `project.task` manualmente.

---

## 10. Timer model — `timer.timer`

Fuente: `odoo_enterprise/18/timer/models/timer.py::TimerTimer`. Modelo base sobre el que FSM monta los timers de timesheet. Campos: ver §3.

### Constraint `UNIQUE(res_model, res_id, user_id)`
`timer/models/timer.py:18-20`, `TimerTimer._sql_constraints` — un solo timer activo por usuario por registro. Un segundo start en otro dispositivo falla la constraint sin superficie UI: FSMG11.

### Métodos
- `action_timer_start()` — `timer_start=now()`.
- `action_timer_stop()` — devuelve minutos transcurridos, limpia `timer_start`/`timer_pause`.
- `_get_minutes_spent()` — descuenta pausas (l.49-62).
- `action_timer_pause()` / `action_timer_resume()`.

### Overlay FSM
`industry_fsm/models/project_task.py:231-242`, `Task.action_timer_start` — override de `action_timer_start()` postea mensaje con timestamp en chatter. v19 añade captura opcional de geolocalización (`allow_geolocation`).

### Timer → timesheet
`_stop_all_timers_and_create_timesheets()` (v18 l.315-350; renombrado `_update_timesheets` en v19):
1. Recoge `timer.timer` ligados a la task y a `account.analytic.line`.
2. Calcula minutos por timer.
3. Aplica ICP `timesheet_grid.timesheet_min_duration` y `timesheet_grid.timesheet_rounding` — infla timesheets en silencio: FSMG10.
4. Crea `account.analytic.line`.
5. Unlink de los `timer.timer`.

Timer corriendo con pestaña cerrada: el server-side sigue contando (FSMG48).

---

## 11. Configuración mínima

Para tener un flujo FSM con bridges operativo:

1. **Proyecto FSM**: `project.project.is_fsm=True`. Activa el contexto `fsm_mode`.
2. **Billable + material**: `allow_billable` y `allow_material` (cuidado: `allow_material=True` fuerza `allow_billable=True` — FSMG4) + `timesheet_product_id` obligatorio o salta SQL constraint cruda (FSMG5).
3. **Timer rounding** (opcional): `timesheet_grid.timesheet_min_duration` / `timesheet_rounding` en Settings.
4. **Helpdesk**: en `helpdesk.team` activar `use_fsm` y fijar `fsm_project_id` explícitamente (no fiarse del fallback `min(id)` — FSMG51).
5. **Stock**: instalar `industry_fsm_stock` o los materiales se marcan entregados sin movimiento real (FSMG28). Warehouse = `user.property_warehouse_id`, sin override por task (FSMG33).
6. **Report/worksheet**: instalar `industry_fsm_report` para forzar worksheet + firma en el cierre (el core no los exige — FSMG15) y guardar la form view del template para generar `report_view_id` (FSMG38).
7. **Subscription**: instalar `industry_fsm_sale_subscription` para excluir productos recurrentes del catálogo de materiales (FSMG29).

---

## 12. Cambios v19

- **Gantt FSM eliminado**: `fsm_task_gantt` JS es solo v18; v19 usa el gantt estándar de `project_enterprise` con filtros de contexto FSM (FSMG47). Las task dependencies (forzadas a False en v18) se eliminan por completo.
- **`partner_phone`**: el campo persiste (`industry_fsm/models/project_task.py:109`, `Task` y `industry_fsm/models/project_task.py:130`, `Task._compute_partner_phone`) pero desaparece el **inverse bidireccional**; ya no muta el partner, solo un flag `is_task_phone_update` señala divergencia (FSMG58, FSMG1).
- **Subscription bridge**: v19 usa `from odoo.fields import Domain` + `Domain.AND` (`odoo_enterprise/19/industry_fsm_sale_subscription/models/project_task.py:3,11`) en lugar de `odoo.osv.expression` (FSMG57).
- **Security groups**: `users` → `user_ids`, `category_id` → `privilege_id` en data XML (FSMG59).
- **Timer**: nuevo wizard `hr.timesheet.stop.timer.confirmation.wizard` y nuevo modelo `industry_fsm.account_analytic_line` que decopla el messaging del timer de `project.task`.
- **Portal**: `SELF_READABLE_FIELDS` → `TASK_PORTAL_READABLE_FIELDS` (FSMG9); el auto-assign de `partner_id` en create se elimina (FSMG6).
- Detalle completo → [[v19_changes]].

---

## 13. Test references

- `industry_fsm_sale/tests/test_multicompany.py:12-166`, `TestMultiCompany` — confirmación de SO cross-company con FSM task.
- `industry_fsm_sale/tests/test_industry_fsm_employee_rate.py:24-156`, `TestIndustryFsmEmployeeRate.test_fsm_employee_rate` — routing employee-rate con varios mappings.
- `helpdesk_fsm/tests/` — conversión ticket → task.
- `project_sale_subscription/tests/` — generación de task recurrente.

---

## 14. Cross-references

- Flujo extremo a extremo (no duplicar aquí) → [[playbooks/fsm_end_to_end]].
- Core model behaviour → [[core]].
- Worksheet engine → [[worksheet]].
- Sale chain → [[extensions_sale]].
- Stock chain → [[extensions_stock]].
- Report y SMS → [[extensions_report]].
- Mobile / map / gantt / signature pad → [[mobile]].
- Gotchas cross-module → [[gotchas]] FSMG50-56 (+ FSMG2, FSMG10-12, FSMG24, FSMG28-29, FSMG32-36, FSMG38, FSMG42, FSMG47-48, FSMG58 referenciados arriba).
- v18 → v19 breaking-change index → [[v19_changes]].
