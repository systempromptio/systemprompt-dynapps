<!-- verified: v18.0 @ 2026-06 -->
# FSM — Gotchas

Hidden behaviours, silent failures and non-obvious traps across the Field Service stack. Numbered for indexing in [[cheatsheet_symptoms]].

---

## Core (`industry_fsm`)

### FSMG1 — Partner phone is bidirectional and writes to the partner record _ES: Editar el teléfono en la tarea cambia el teléfono en la ficha del cliente; dos tareas del mismo cliente se pisan el número y gana la última._
- `project.task.partner_phone` (`industry_fsm/project_task.py:42`, `Task.partner_phone`) computes from `partner_id.mobile or partner_id.phone` (`_compute_partner_phone`, `:128-131`) and **its inverse writes back to the partner** (`_inverse_partner_phone`, `:133-140`): writes `partner.mobile` if mobile is set or phone is empty, else `partner.phone`. Editing the phone on the task silently mutates customer master data — two FSM tasks on the same partner with different phones means the last save wins, both tasks show the same final value, and the customer record is overwritten.
- **Inverse removed in v19** (field persists, reads `partner_id.phone` directly, no write-back). See FSMG58 and [[v19_changes]].

### FSMG2 — Date validation silently clears both `planned_date_begin` and `date_deadline` _ES: Al cambiar solo una fecha de la tarea (inicio o límite) se borran las dos y desaparece la planificación, sin ningún error._
- FSM tasks require **both** dates. Writing only one in `write()` → both nullified, no error. The override at `industry_fsm_sale/models/project_task.py:177-198`, `Task._compute_material_line_totals` enforces this so resource allocation has both ends. Side effect: bulk-edits that touch a single date wipe the schedule.
- `_onchange_planned_dates()` is a no-op for FSM tasks (lines 172-175), so the helpful base validation messages are silenced.

### FSMG3 — `is_fsm` toggle "hides" tasks rather than archives them _ES: Al desactivar y reactivar el modo servicio de campo del proyecto, tareas antiguas desaparecen de los filtros pero siguen existiendo._
- Switching a project's `is_fsm` from True → False → True leaves tasks alive but invisible to FSM-context filters (`fsm_mode=True`). They remain accessible by direct ID (no security rule). On a re-toggle they reappear.

### FSMG4 — `allow_material` cascade auto-enables `allow_billable` _ES: Al activar materiales en el proyecto se activa sola la facturación, y salta un error SQL raro si falta el producto de horas._
- Setting `allow_material=True` on the FSM project silently toggles `allow_billable=True` (bidirectional compute with no warning). Backed by SQL constraint `material_imply_billable`. Forgetting the timesheet product on top → cryptic SQL constraint violation, not a user-friendly error.

### FSMG5 — `timesheet_product_id` is required, but the error is SQL-only _ES: Error crudo de PostgreSQL al configurar facturación de horas porque falta el producto de parte de horas, sin mensaje claro._
- SQL constraint `timesheet_product_required_if_billable_and_time` fires only at DB level. If you tried to enable billable + timesheets via Studio or XML data without setting the product, you get a raw PostgreSQL error mid-install instead of a validation message.

### FSMG6 — Portal user `partner_id` auto-assignment on create (v18 only) _ES: Cuando un usuario del portal crea una tarea de servicio, se pone él mismo como cliente automáticamente (solo v18)._
- A portal user creating an FSM task gets `partner_id = current_user.partner_id` silently injected by `create()` (`industry_fsm/project_task.py:200-206`, `Task.create`, guarded by `has_group('base.group_portal')`). **Removed in v19** — `create()` no longer overrides `partner_id`, so the portal user must now explicitly set the partner.

### FSMG7 — Mark-as-Done buttons filtered while timer runs _ES: No aparece el botón 'Marcar como hecho' mientras el cronómetro está en marcha, sin ninguna explicación._
- `_compute_mark_as_done_buttons` (`industry_fsm/project_task.py:113`, `Task._compute_mark_as_done_buttons`) sets both `display_mark_as_done_primary` and `display_mark_as_done_secondary` to **False** when `task.timer_start` is set (also when `fsm_done` or not `is_fsm`). Tasks with a running timer show no Done button. UI gives no error message — the button just disappears.

### FSMG8 — Recurrence copies `partner_phone` as a frozen Char _ES: Las tareas recurrentes conservan el teléfono del cliente congelado; cambiarlo en la ficha no actualiza las ya creadas._
- `project.task.recurrence` materialises children with a `partner_phone` snapshot of the parent (Char field, not related). Updating the partner phone later does NOT update existing children.

### FSMG9 — `SELF_READABLE_FIELDS` rename _ES: Tras migrar a v19 los usuarios del portal reciben error de acceso al abrir campos extra que antes veían._
- Custom modules extending `SELF_READABLE_FIELDS` to expose extra portal fields must rename it to `TASK_PORTAL_READABLE_FIELDS` in v19. Failure manifests as portal users receiving AccessError on the new fields.

### FSMG10 — Timer rounding silently inflates timesheets _ES: Un cronómetro de 2 minutos genera un parte de 10 minutos por el redondeo o duración mínima, sin avisar al técnico._
- `_stop_all_timers_and_create_timesheets()` applies `timesheet_grid.timesheet_min_duration` and `timesheet_grid.timesheet_rounding` settings during conversion. If admin sets `min_duration=10`, a 2-minute timer creates a 10-min `account.analytic.line` — user sees no feedback that the duration was bumped.

### FSMG11 — Timer collisions across devices fail silently _ES: Iniciar el cronómetro en un segundo dispositivo falla en silencio; el técnico cree que ambos cronometran pero solo uno registra._
- `timer.timer` has `UNIQUE(res_model, res_id, user_id)`. Starting the timer on device B while device A is already running fails the DB constraint with no UI surface. Device A keeps running; technician believes both are timing but only A logs.

### FSMG12 — Resource overlap is warning-only _ES: Dos tareas de campo solapadas para el mismo técnico solo muestran un aviso, no se reprograman automáticamente._
- `_allocated_hours_per_user_for_scale()` flags conflicts when two FSM tasks overlap on the same `user_ids` member, but **no auto-reschedule** happens. Multi-day all-day tasks (≥24h) use work-interval intersection, so the warning math differs from regular hour totals.

### FSMG13 — Project archive does not cascade to FSM tasks _ES: Al archivar el proyecto las tareas siguen activas y apareciendo en 'Mis tareas' del técnico._
- Archiving the FSM project leaves its tasks active. Technician's "My Tasks" still shows them. No cron or post-write hook closes them.

### FSMG14 — Portal access token regenerates on `copy()` _ES: Tras duplicar una tarea, los enlaces de portal enviados al cliente dejan de apuntar a la tarea correcta._
- `task.copy()` generates a fresh `access_token`. Customer URLs from before the copy still work against the original — but if you migrated workflows by copy-then-archive, those URLs go stale silently. Use `task.get_portal_url()` after copy.

### FSMG15 — `action_fsm_validate` does NOT enforce worksheet/signature in core _ES: Se pueden cerrar tareas sin hoja de trabajo ni firma si no está instalado el módulo de informes FSM._
- Only `industry_fsm_report` adds the worksheet + signature checks. Core `industry_fsm` allows `fsm_done=True` without either if your setup doesn't install the report layer. Custom installations that skip `industry_fsm_report` may close tasks with no audit trail.

---

## Worksheet engine (`worksheet`)

### FSMG16 — Form view is the report _ES: Ocultar un campo en el formulario de la hoja de trabajo también lo oculta del PDF que recibe el cliente._
- The QWeb PDF template is regenerated from the worksheet form view every time Studio saves the form. Hiding a field "for internal use" hides it from the customer PDF too. Workaround: custom XML inheritance with `t-if` conditions instead of `invisible=`.

### FSMG17 — Studio field rename breaks PDF until re-save _ES: Tras renombrar un campo en Studio, el informe sale en blanco en ese campo hasta volver a guardar el formulario._
- Renaming `x_foo` to `x_bar` in Studio leaves the old QWeb template with `t-field="worksheet.x_foo"` until the form is saved again (which triggers regeneration). Until then, the report renders blank for that field.

### FSMG18 — Worksheet manual delete leaves PostgreSQL table orphaned _ES: Borrar una plantilla de hoja de trabajo deja tablas huérfanas en la base de datos tras desinstalar el módulo._
- Manual `unlink()` on `worksheet.template` cascades models/views/ACLs/rules but does **not** drop the dynamic `x_*` PostgreSQL table. Done deliberately to prevent data loss, but if you uninstall the `worksheet` module afterwards, the table is orphaned with no model definition pointing at it.

### FSMG19 — One worksheet record per host record per template _ES: El botón de hoja de trabajo siempre reabre la misma hoja; no se pueden tener varias por tarea con una plantilla._
- SQL `unique(x_<res_model>_id)` constraint means re-opening the worksheet button on a task always returns the existing record. To have multiple worksheets per task, create multiple templates.

### FSMG20 — Studio Many2one chains across worksheet models can crash reports _ES: Enlazar entre sí modelos de hojas de trabajo con Studio puede romper el PDF al imprimir el informe._
- Adding an `x_*` Many2one from one dynamic worksheet model to another (Studio allows it) can produce invalid `t-field` expressions in the regenerated QWeb if the target is not loaded at render time. Safer: keep worksheet models flat.

### FSMG21 — `x_name` orphaning in v18 _ES: Desinstalar el módulo anfitrión sin borrar antes la plantilla deja registros de campo huérfanos en v18._
- v18 does not register `x_name` as `ir.model.data`, so uninstalling the host module without unlinking the template first leaves an orphan field record. v19 fixes this — see [[v19_changes]].

---

## Sale extension (`industry_fsm_sale`)

### FSMG22 — Task created twice if SO confirms before product worksheet template is set _ES: Al confirmar el pedido aparecen dos tareas para la misma línea, una con plantilla de hoja y otra sin ella._
- SO line auto-creates task via `_timesheet_create_task()`. If FSM project + product without `worksheet_template_id`, the task may be created once by `sale_timesheet` and a second time by FSM logic injecting `partner_shipping_id`. Symptom: two tasks linked to the same SOL, one with the template and one without.

### FSMG23 — Task partner inherits SO **shipping** address, not invoice _ES: El cliente de la tarea sale con la dirección de entrega del pedido, no la de factura; cambiar la entrega cambia la tarea._
- `_compute_partner_id()` (`industry_fsm_sale/project_task.py:258`, `Task._compute_partner_id`) sets `task.partner_id = SO.partner_shipping_id` (not `partner_id`) when the user has group `account.group_delivery_invoice_address` (`:259-262`); otherwise it falls through to `super()`. Custom code that assumes `task.partner_id == SO.partner_id` will mismatch. The compute depends on `sale_order_id.partner_shipping_id` (`:257`), so updating SO shipping later silently updates the task's partner.

### FSMG24 — `under_warranty` only visible when material is enabled _ES: No aparece la casilla de garantía en tareas de solo mano de obra que no llevan materiales._
- The field is invisible unless `is_fsm AND allow_material`. Pure-time interventions (no materials) cannot be flagged warranty via the form. Backend write still possible.

### FSMG25 — Locked SO silently rejects material additions _ES: El botón '+' para añadir materiales no hace nada cuando el pedido está bloqueado, sin mensaje de error._
- `product.set_fsm_quantity()` returns early when `sale_order.locked=True` with no error. The "+" button on the catalog gives no feedback. Typically happens after the invoice has been validated.

### FSMG26 — Multi-company SO confirmed in foreign company _ES: Confirmar el pedido desde otra compañía da error de acceso al facturar si el diario no es visible en la compañía activa._
- FSM project pinned to Company A; user in Company B with allowed_company_ids=[A,B] confirms the SO via the task. Company A is preserved on task and SO, but invoicing context may fail later with AccessError if the invoice journal isn't visible to the user's active company.

### FSMG27 — Pricelist missing on SO falls back to product list price _ES: Sin tarifa en pedido ni cliente, los materiales usan el precio de lista del producto, con importes o impuestos inesperados._
- If neither SO nor partner has a pricelist set, `task_rate` SOL uses product list price as-is. Inconsistent multi-currency / tax behaviour if pricelist was meant to apply.

### FSMG28 — Materials auto-marked delivered without `industry_fsm_stock` _ES: Sin el módulo de stock instalado, se factura el material como entregado aunque nunca salió del almacén._
- Without the stock module installed, `_prepare_materials_delivery()` blindly sets `qty_delivered = product_uom_qty` for material SOLs. Customer is invoiced for "delivered" materials that never moved out of stock.

### FSMG29 — Recurring vs subscription tasks: two mechanisms _ES: Aparecen tareas duplicadas cuando coinciden la recurrencia del proyecto y la renovación de la suscripción._
- `project.task.recurrence` creates copies via cron `_cron_recurring_create_task`.
- `project_sale_subscription` creates new tasks on subscription renewal.
- They coexist; you can get duplicate tasks if both apply. `industry_fsm_sale_subscription` filters `recurring_invoice=True` products out of the FSM material UI to reduce confusion.

### FSMG30 — Employee-rate timesheets unlinked from SOL until fsm_done _ES: Con tarifa por empleado, los partes no se facturan hasta cerrar la tarea; los informes previos muestran cero horas facturables._
- For `pricing_type='employee_rate'`, `_timesheet_determine_sale_line()` returns `False` if the mapping exists but the SOL hasn't been created yet. Timesheets sit unbilled until `action_fsm_validate()` → `_fsm_create_sale_order_line()` creates the SOLs (`industry_fsm_sale/project_task.py:524`, `Task._fsm_create_sale_order_line`, employee-rate branch around `:637-644`). Pre-validation reports show zero billable hours.

---

## Stock extension (`industry_fsm_stock`)

### FSMG31 — `cancel_backorder=True` silently truncates partial picks _ES: Al cerrar la tarea, un pedido con entrega parcial da por entregado solo lo recogido y se pierde el resto sin aviso._
- On close, `action_fsm_validate()` → `_validate_stock()` (`industry_fsm_stock/project_task.py:36`, `Task._validate_stock`) validates pickings with `button_validate()` under `cancel_backorder=True` context (`:115`). A 10-unit order with only 5 picked finishes as 5 delivered — the remaining 5 are gone. No follow-up picking, no warning.

### FSMG32 — Cannot decrease quantity below the delivered amount _ES: Error al bajar la cantidad de material por debajo de lo ya entregado; la devolución hay que hacerla manual en Inventario._
- `_inverse_fsm_quantity()` raises `UserError` if new `fsm_quantity < quantity_decreasable_sum`. To return materials post-pick, you must do a manual return picking in the Inventory app (no FSM-driven return flow).

### FSMG33 — Warehouse is the user's default, no per-task override _ES: Los materiales siempre salen del almacén por defecto del técnico; no se puede fijar otro almacén por tarea._
- Multi-warehouse orgs cannot pin a task to a specific warehouse via FSM. Materials always source from `user.property_warehouse_id`. Workaround: separate user per warehouse.

### FSMG34 — 2-step / 3-step delivery + warehouse mismatch cancels moves _ES: Con entrega en varios pasos y almacén distinto, se cancelan los movimientos intermedios y se pierde la reserva de lote._
- When the technician's default warehouse differs from the SO warehouse and 2-step delivery is configured, the lot-tracking wizard **cancels** the intermediate moves instead of syncing the lot (line 156-158). Lot reservation lost on intermediate stages.

### FSMG35 — Serial tracking = one SOL per serial _ES: Cada número de serie crea su propia línea de pedido con cantidad 1; cinco series salen como cinco líneas._
- For `product.tracking='serial'`, each serial creates its own SOL with qty=1. `material_line_product_count` inflates accordingly. Set expectations: 5 serials = 5 lines, not 1 line with qty 5.

### FSMG36 — Material price update after invoice _ES: Tras facturar, cambiar la cantidad de material no actualiza la factura; el total de la tarea difiere del de la factura._
- SOL `price_unit` locks once invoiced; subsequent quantity edits update `price_total` on the SOL but the invoice line is frozen. `material_line_total_price` on the task may diverge from the invoice total.

### FSMG37 — `industry_fsm_repair` has no FSM→repair workflow _ES: El puente con reparaciones no crea órdenes de reparación desde la tarea; hay que crearlas a mano._
- Pure access-control bridge. No automatic `repair.order` creation from FSM tasks, no status sync. Repair operations must be initiated manually.

---

## Report extension (`industry_fsm_report` / `_sale_report` / `_sms`)

### FSMG38 — PDF empty if worksheet template has no `report_view_id` _ES: La sección de hoja de trabajo sale vacía en el PDF si la plantilla nunca se guardó desde su formulario._
- The QWeb section is wrapped in `t-if="doc.worksheet_count and doc.worksheet_template_id.report_view_id"`. Templates created without ever saving the form view have no `report_view_id` — PDF section is silently blank.

### FSMG39 — Signature widget broken on iOS Safari < 15 _ES: La firma se dibuja desplazada unos píxeles en iPhone o iPad con Safari antiguo, saliendo torcida._
- Touch coordinates offset 2–4 px from canvas, causing "drift". Fixed in Odoo 18.3+. Workaround: update iOS or override the signature widget.

### FSMG40 — Map view requires Google Maps API key, navigation button does not _ES: El mapa no carga sin clave de Google Maps, aunque el botón de 'navegar hasta el cliente' sí funciona._
- Map view (uses `web_map`) requires API key in settings. Without it, tiles silently fail to load. The "Navigate to partner" button uses the public Google Maps URL endpoint — works without API key.

### FSMG41 — Map lazy-loaded → first navigation has 1-2 s lag _ES: La primera vez que se abre la pestaña de mapa tarda uno o dos segundos en cargar._
- Map JS is in `web.assets_backend_lazy`. First time the user clicks the map tab, the bundle downloads. Subsequent navigations are cached.

### FSMG42 — SMS uses `partner_id.mobile`, display uses `mobile or phone` _ES: El SMS al cliente no se envía si no tiene móvil, aunque la tarea muestre un teléfono fijo._
- Stage-driven SMS only sends if `partner.mobile` is populated. Tasks can show a phone via `partner_phone` even when SMS fails silently because mobile is empty.

### FSMG43 — `_is_fsm_report_available()` requires at least one timesheet _ES: El botón 'Enviar informe' no aparece si la tarea no tiene ningún parte de horas, aunque tenga materiales y hojas._
- The "Send Report" button hides if no `timesheet_ids` exist on the task — even if you only have worksheets + materials. Workaround: log a dummy timesheet, or override the method.

### FSMG44 — Portal `worksheet_map` not iterated in portal template _ES: Los campos personalizados de la hoja de trabajo se ven en el PDF pero no en la página del cliente en el portal._
- The portal page receives `worksheet_map` in context but the default portal template only renders signature + timesheets. Custom `x_*` fields are visible in the PDF only, not on the customer's portal page. Extend `project_portal_templates.xml` to iterate the worksheet record.

### FSMG45 — Browser crash mid-signature loses the canvas _ES: Si el navegador se cierra antes de guardar se pierde la firma dibujada, aunque el resto del formulario se recupere._
- The signature canvas state is not part of the form draft `localStorage`. A crash before POST loses everything. Form draft framework recovers the rest of the form on reload.

### FSMG46 — Worksheet template delete cascades to all linked dynamic models _ES: Borrar una plantilla de hoja compartida elimina a la vez los datos de hoja de todos los módulos que la usan._
- `worksheet.template.unlink()` removes views, action, ACLs, rules, model. If the template is shared across multiple host modules, all of them lose their worksheet data simultaneously. System blocks delete only if there are existing host records linked.

### FSMG47 — Gantt removed in v19 _ES: Las personalizaciones de la vista Gantt de servicio de campo dejan de funcionar al migrar a v19._
- `fsm_task_gantt` JS view is v18-only. Custom modifications to it fail in v19 upgrades. v19 uses project_enterprise's standard gantt with FSM context filters.

### FSMG48 — Timer running with browser tab closed _ES: Cerrar la pestaña no para el cronómetro del servidor; al volver el tiempo puede aparecer descuadrado._
- Frontend animation lost, server-side timer keeps ticking. Reopening the task shows resumed time but the elapsed value may be off by the tab-closed duration. Force-stop pattern: `action_fsm_validate(stop_running_timers=True)` from a cron if you want to defensively close abandoned timers.

### FSMG49 — Worksheet signature field excluded from CSV export _ES: Las firmas no salen al exportar a CSV; hay que sacarlas del adjunto por separado._
- Binary fields are excluded by default. Programmatic export of signatures requires a custom export action or reading the `ir.attachment` directly.

---

## Cross-module / integration

### FSMG50 — Planning slot deletion orphans FSM tasks _ES: Borrar un turno de planificación deja la tarea programada sin contexto de disponibilidad del recurso._
- `planning.slot._compute_overlap_slot_count` tracks resource availability, but deleting a planning slot does not nullify any task reference (no field on task to clear). Task remains scheduled with no resource availability context.

### FSMG51 — Helpdesk team `fsm_project_id` silent fallback _ES: El asistente de convertir ticket en tarea abre con el proyecto vacío o con uno inesperado si no hay proyecto FSM configurado._
- `_compute_fsm_project_id()` falls back to `min(id)` of any FSM project in the company if no project is explicitly configured. If no FSM project exists in the team's company, the convert-to-task wizard opens with an empty project dropdown — user confusion, not error.

### FSMG52 — Subscription cron timing creates off-cycle tasks _ES: La renovación de suscripción a mitad de mes crea tareas de campo inesperadas fuera del ciclo previsto._
- `_cron_create_recurring_tasks` is the same cron for project.task.recurrence and project_sale_subscription. Subscription renewal mid-month spawns FSM tasks immediately, not aligned to a fixed cycle. Technicians see unexpected new tasks.

### FSMG53 — `documents_fsm` forces `use_documents=False` on FSM projects _ES: Activar Documentos y luego marcar el proyecto como servicio de campo desactiva la configuración de documentos y oculta los enlaces._
- Enabling Documents on a project and then setting `is_fsm=True` strips the document configuration (`use_documents` auto-computed to False). Existing document links on tasks remain in the DB but the project UI hides them.

### FSMG54 — Sign module separate from `worksheet_signature` _ES: La firma electrónica del módulo Sign y la firma en pantalla de la hoja de trabajo son sistemas separados que no se comunican._
- `sign` (e-signature workflows) is not integrated with FSM `worksheet_signature` (on-site canvas signature). Two independent signing systems coexist.

### FSMG55 — No SLA propagation from helpdesk to FSM task _ES: Al convertir un ticket de soporte en tarea de campo no se heredan los SLA; el seguimiento se queda en el ticket._
- Converting a helpdesk ticket into an FSM task copies partner/team metadata but does not inherit SLAs. SLA tracking stays on the original ticket.

### FSMG56 — FSM date truncation can wipe gantt schedule on bulk edits _ES: Una edición masiva de fecha límite sin fecha de inicio borra ambas fechas en todas las tareas seleccionadas._
- Bulk-edit "set deadline to next Friday" without setting `planned_date_begin` clears BOTH fields on every selected FSM task. The base `project` validation would catch this; FSM's `_onchange_planned_dates` no-op silences the warning.

---

## Validation chain (playbook-derived)

### FSMG61 — Close method is public `action_fsm_validate` (no leading underscore) _ES: Módulos custom que sobrescriben una versión 'privada' del cierre de tarea no se ejecutan y el cierre ignora su lógica._
- The whole override chain uses the PUBLIC name without leading underscore: `industry_fsm/project_task.py:280`, `Task.action_fsm_validate`, `Task.action_fsm_validate` → `industry_fsm_sale/project_task.py:449` → `industry_fsm_stock/project_task.py:175` (the stock module's `_validate_stock` lives at `:36`, but the validate override that calls `super()` is `:175`). Custom modules that call `super()` on a private underscore-prefixed name, or patch a private variant, never enter the super-chain and fail silently (close runs without their logic). Always override `action_fsm_validate`. NOTE: a private underscore-prefixed variant does not exist anywhere in the source.

### FSMG62 — Warranty task with no material and no prior SO closes with NO sale order _ES: Una tarea en garantía sin materiales ni pedido previo se cierra sin crear pedido de venta, imposibilitando facturar después._
- In `employee_rate` pricing, `action_fsm_validate` does `continue` (creates no SO) when `under_warranty AND material_line_product_count == 0`, or when `not sale_order_id and not timesheet_count` (`industry_fsm_sale/models/project_task.py:462`, `Task.action_fsm_validate`). A warranty task without materials and without a pre-existing SO closes with no sale order at all — later invoicing is impossible without creating one manually.

### FSMG63 — No offline mode: FSM is PWA-only, native mobile apps discontinued _ES: La app móvil de servicio de campo no funciona sin conexión; sin red no se pueden crear tareas, fichar tiempo ni firmar._
- FSM has **no native offline capability** in v18 or v19. Mobile = **PWA only**; Odoo discontinued the native iOS/Android apps (from v17), so there is no native-app fallback that caches data offline. The PWA Service Worker caches **static assets** (JS/CSS) so the UI loads without network, but every data read/write (create task, change stage, timer, validate worksheet, sign) **requires a live connection**. Worksheet drafts live in browser `localStorage` only while the form tab stays open — closing the app loses them. See [[mobile]] §1. Real offline (queue + IndexedDB sync) needs custom development or a third-party/OCA module; do NOT promise offline FSM out of the box, and do NOT suggest "use the native app" as a workaround — it no longer exists.

---

## v19 migration

### FSMG57 — `from odoo.osv import expression` dead in v19 _ES: Código FSM que importa expression rompe o genera avisos en v19; hay que pasar al nuevo Domain._
- All FSM v19 code uses `from odoo.fields import Domain` (`Domain.OR`, `Domain.AND`, `Domain(domain)`). Mixed imports break or generate warnings.

### FSMG58 — `partner_phone` inverse removed in v19 (field persists) _ES: En v19, editar el teléfono en la tarea ya no actualiza la ficha del cliente como hacía antes._

> ⚠ **Corregido en FSMG69:** `industry_fsm` ya NO define `partner_phone` (se mueve a `project`); leer/escribir sigue funcionando y el inverse persiste en project.
- The field `partner_phone` STILL EXISTS in v19 (`industry_fsm/models/project_task.py:42`, `Task.partner_phone` v18 / `:43` v19). What disappears is the **bidirectional inverse** `_inverse_partner_phone`: present in v18 (`:133-140`, writes back to `partner.mobile`/`partner.phone`), **absent in v19** (no `inverse=` on the field, `_compute_partner_phone` reads partner directly). The computed flag `is_task_phone_update` is NOT new — it already exists in v18 (`:51`, `_compute_is_task_phone_update` at `:142-148`) and v19 (`:52`, `:131-133`); it only signals task/partner divergence and never mutated the partner. Impact: custom code that relied on editing the task phone to update the customer record silently stops propagating in v19. Reading `task.partner_phone` keeps working.

### FSMG59 — Security groups: `users` → `user_ids`, `category_id` → `privilege_id` _ES: Datos XML de grupos que usan el campo users fallan al instalar en v19._
- v18 data XML referencing `<field name="users">` fails install on v19. The migration auto-creates `res.groups.privilege` records.

### FSMG60 — `data` files renamed in `industry_fsm_report` _ES: Herencias XML que apuntan a rutas de fichero antiguas del informe FSM fallan al instalar en v19._
- Module XML inheritances referencing old file paths (e.g. `industry_fsm_report.project_views`) fail in v19. New names: `project_project_views`, `worksheet_template_data`, etc. See [[v19_changes]] §7.

---

## Cross-references

- Core model behaviour → [[core]].
- Worksheet engine internals → [[worksheet]].
- Sale chain wiring → [[extensions_sale]].
- Stock chain wiring → [[extensions_stock]].
- Report and SMS layer → [[extensions_report]].
- Mobile / map / gantt / signature pad → [[mobile]].
- Planning, helpdesk, subscription, timer model → [[integrations]].
- v18 → v19 breaking-change index → [[v19_changes]].


<!-- ===== Hallazgos verificados vs fuente v19 (2026-07-14, adversarial) — renumerado 2026-07-14 ===== -->

### FSMG64 — v19: timers FSM sobre `account.analytic.line`, no `project.task`

En v19 los timers FSM se registran sobre `account.analytic.line` (`res_model="account.analytic.line"`) con puntero al padre `parent_res_model="project.task"`/`parent_res_id=task.id`. Cualquier búsqueda custom de timers por `res_model="project.task"` (patrón v18) devuelve VACÍO en v19; buscar por `('parent_res_model','=','project.task')`. `action_fsm_validate` y `_stop_all_timers_and_update_timesheets` ya usan el nuevo esquema. (Complementa FSMG11: el UNIQUE(res_model,res_id,user_id) NO cambia; FSMG48 force-stop sigue válido.)

### FSMG65 — v19: `project.task.create.timesheet` ELIMINADO (overrides mueren en silencio)

El modelo `project.task.create.timesheet` ya no existe en v19 (ni en industry_fsm ni en base). Overrides custom de su `save_timesheet()` dejan de ejecutarse SILENCIOSAMENTE. El chatter "Timer stopped at" lo emiten ahora: (1) `account.analytic.line.action_timer_stop` (`industry_fsm/models/account_analytic_line.py`) y (2) `hr.timesheet.stop.timer.confirmation.wizard.action_save_timesheet` (`industry_fsm/wizard/`, hereda de `timesheet_grid`). Portar overrides a esos dos puntos. (Ver §3.)

### FSMG66 — v19: ajuste `group_industry_fsm_quotations` eliminado (el grupo NO) [PARTIAL]

v19 elimina el ajuste `group_industry_fsm_quotations` (Extra Quotations) de `res.config.settings` y el override `_get_basic_project_domain`. NO elimina el `res.groups` `group_fsm_quotation_from_task` (sigue en `fsm_security.xml`). Por tanto: referencias al SETTING o al compute rompen; referencias al GRUPO siguen resolviendo.

### FSMG67 — v19: `task_custom_report` (Field Service Report) restringido a `group_fsm_user`

v19 restringe la acción `ir.actions.report` `task_custom_report` al grupo `industry_fsm.group_fsm_user` mediante `group_ids` (`worksheet_custom_reports.xml`). Usuarios/perfiles sin el grupo FSM user no pueden imprimir el Field Service Report; código custom que invoque el informe con otro perfil falla por permisos. No existía en v18.

### FSMG68 — v19: el redondeo de timers FSM migra a `round_time_spent` (amplía FSMG10)

En v19 el redondeo lo hace `round_time_spent` (`odoo.addons.timer.utils.timer_utils`) y se escribe `unit_amount` directamente en el timesheet. `_timer_rounding` y `_add_timesheet_time` YA NO intervienen en el flujo FSM de parada de timers. Overrides de redondeo FSM deben reengancharse a `round_time_spent` o al write del `unit_amount`. Los parámetros `timesheet_grid.timesheet_min_duration`/`timesheet_rounding` se siguen leyendo igual.

### FSMG69 — v19: `partner_phone` se mueve a `project`, NO desaparece (corrige FSMG58)

Es INCORRECTO decir que `partner_phone` sigue definido en `industry_fsm:43` en v19. En v19 `industry_fsm` YA NO define `partner_phone`: la definición (compute `_compute_partner_phone` + inverse `_inverse_partner_phone`) se traslada al módulo base `project` (`project/models/project_task.py:228`, `ProjectTask.partner_phone`, v19, computes en `:663-670`). `industry_fsm` solo lo consume (`_compute_is_task_phone_update`, `TASK_PORTAL_READABLE_FIELDS`). El write-back al partner NO se elimina: el inverse persiste en `project` y escribe `partner_id.phone` (simplificado: desaparece el cascade mobile-first de v18). Leer/escribir `task.partner_phone` sigue funcionando. (FSMG1: el inverse vive ahora en project y solo escribe `phone`; el write-back a master data persiste.)

