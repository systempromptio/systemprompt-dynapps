<!-- verified: v18.0 @ 2026-06 -->
# Approvals — Gotchas (Odoo Enterprise 18)

Convención de IDs: **APPG** = approvals core, **APPP** = approvals_purchase, **APPS** = approvals_purchase_stock.

---

## Aprobadores & cómputo de estado

### APPG1 — `_compute_approver_ids` borra y recrea TODO
`approvals/models/approval_request.py:295-327`, `ApprovalRequest._compute_approver_ids` empieza con `Command.clear()`. Si cambias `category_id` o `request_owner_id`, pierdes approvers añadidos manualmente.
**Workaround:** edita approvers tras el último cambio de category/owner.

### APPG2 — Manager siempre `sequence=9`
Hardcoded en `_compute_approver_ids` l.309-315. En `approver_sequence=True` el manager aprueba SIEMPRE primero, aunque otros approvers tengan `sequence=1`. Para anteceder al manager: setea `sequence < 9` post-recompute (no en plantilla, porque category.approver.sequence default es 10).

### APPG3 — Manager duplicado se pop silenciosamente
Si manager_approval activo Y el manager también está en `category.approver_ids`, l.316-317 hace `pop(manager_user_id)` del dict de plantilla. El manager aparece UNA vez (con sus props de manager_approval, no las de plantilla). No hay warning.

### APPG4 — `required=True` puede bloquear request "para siempre"
`_compute_request_status` l.276: `required_approved = all(a.status=='approved' for a in approver_ids.filtered('required'))`. Si un required tiene status `new/waiting/pending/refused`, el request NUNCA pasa a `approved` aunque `count('approved') >= approval_minimum`. UI no avisa.

### APPG5 — `cancel` tiene prioridad sobre `refused`
`_compute_request_status` l.279-282 evalúa `cancel` antes que `refused`. Si en el set de status hay AMBOS (caso raro tras mass edits), gana `cancel`.

### APPG6 — `minimal_approver` tolerancia silenciosa
`minimal_approver = min(approval_minimum, len(status_lst))` (l.277). Si configuras `approval_minimum=5` pero solo hay 2 approvers, NO bloquea — sigue requiriendo solo 2. Esta tolerancia evita estados imposibles pero también enmascara configuraciones erróneas.

### APPG7 — `status='waiting'` solo existe con `approver_sequence=True`
Sin sequence, todos los approvers van a `pending` simultáneamente. `'waiting'` nunca aparece. Code que filtre por `'waiting'` debe ser conditional.

### APPG8 — `user_status` devuelve `''` (string vacío) si no eres approver
`_compute_user_status` l.268-270: `approver_ids.filtered(user_id==env.user).status`. Sobre un recordset vacío, `.status` da `''`, NO uno de los selection values. Comparaciones `user_status == 'pending'` funcionan, pero `user_status in ('approved', 'refused')` se comporta esperablemente, ojo con `bool(user_status)`.

### APPG9 — `_check_approver_ids` también valida duplicados
`@api.constrains('approver_ids')` l.357-362. Aparte de la validación a nivel category (l.100-108), el request bloquea también si después de recompute se cuela un duplicado por write manual. **v19**: migrado a `models.Constraint('UNIQUE')` en `approval.approver`.

### APPG10 — `action_confirm` con `approver_sequence` y `approval_minimum=0`
`action_confirm` l.153-157: con `approver_sequence=True` filtra approvers `new/pending/waiting` y hace `approvers[0] if approvers and approvers[0].status != 'pending' else env['approval.approver']`. El `if approvers` ya protege contra IndexError (NO peta con lista vacía). El gotcha real es semántico: si `approval_minimum=0` el chequeo `len(approver_ids) < approval_minimum` (l.147) nunca dispara, y un request sin approvers en estado `new` queda en `pending` sin ninguna activity creada → bloqueo silencioso (nadie tiene que aprobar, pero `_compute_request_status` con `status_lst=[]` lo deja en `'new'`). Nota: `_constrains_approver_sequence` (category l.110-113) prohíbe `approver_sequence` con `approval_minimum=0`, así que este caso solo aparece sin sequence.

### APPG11 — `manager_approval='required'` sin parent_id bloquea
`action_confirm` l.139-146: si `manager_approval='required'` y `employee.parent_id.user_id` no existe, **UserError**. Owner sin manager configurado en hr → no puede submitar nunca. No es warning, es bloqueo.

### APPG12 — Búsqueda de employee toma primero por company
`_compute_approver_ids` l.305: `hr.employee.search([(user_id, company_id)], limit=1)`. Si usuario tiene múltiples employees en la misma company (caso raro pero posible) → toma uno arbitrario por orden BD.

### APPG13 — `precompute=True` en product_uom_id
`approvals/models/approval_product_line.py:23`, `class ApprovalProductLine` tiene `precompute=True`. Se calcula ANTES del save inicial. Si product cambia tras crear la línea, el compute se redispara y puede sobrescribir `product_uom_id` editado manualmente.

### APPG14 — Activities: cancel vs approve/refuse comportamiento diferente
- `action_cancel()` l.263 → `.unlink()` de activities (NO queda traza en chatter).
- `action_approve/refuse()` → `.action_feedback()` (queda traza).
- Cuando `_compute_request_status` llega a estado terminal, `_cancel_activities()` borra las pendientes restantes (l.293).

### APPG15 — `has_access_to_request` súper restrictivo
`_compute_has_access_to_request` l.86-90: True SOLO si `owner==user AND user in group_approval_user`. Un officer que es approver pero NO owner del request: `has_access_to_request=False`. Afecta visibility de botones "Cancel"/"Back to Draft" del header.

### APPG16 — `copy()` lanza UserError en `approval.approver`
`approvals/models/approval_request.py:431-432`, `ApprovalApprover.copy`. No se duplican approvers ni con `copy_data`. Si quieres clonar un request, el `copy()` de `approval.request` regenera approver_ids vía compute (no copia los del original).

### APPG17 — Resubmit tras withdraw activa "reanudación" silenciosa
`approvals/models/approval_request.py:340-348`, `ApprovalRequest.write` (write override de approver_ids): si request está en `pending`, sequence=True, sin approvers en `pending` ya, busca el primer `waiting` y lo pasa a `pending`+crea activity. Esto permite "reactivar" tras todos los withdrawn.

### APPG18 — `change_request_owner` cambia subscripción
`approvals/models/approval_request.py:330-338`, `ApprovalRequest.write`: cambiar `request_owner_id` desuscribe al viejo del chatter y suscribe al nuevo. Si tenías followers custom añadidos, se mantienen aparte. Pero el owner antiguo deja de recibir notificaciones.

### APPG19 — `kanban_view` con `group_expand=True`
`request_status` field l.47 tiene `group_expand=True`. En kanban se muestran TODAS las columnas (`new/pending/approved/refused`) aunque no haya records. Hacer custom kanban override puede ocultarlas.

### APPG20 — `automated_sequence` cambia ir.sequence al editar
`approvals/models/approval_category.py:118-146`, `ApprovalCategory.create`: si editas `sequence_code` o `company_id` de la category, el `ir.sequence` asociado se actualiza in-place. Próximos requests usarán nueva config. Los existentes mantienen su `name` original.

### APPG21 — `attachments_required` no es campo, es `requirer_document`
El nombre del config en UI es "Documents required" pero el campo es `requirer_document` (Selection `required/optional`). No buscar `attachments_required`.

### APPG22 — Domain de approver `user_id` excluye existing pero por context
`approval.approver.user_id` l.372 domain usa `existing_request_user_ids` (computed). Al añadir desde UI funciona, pero por script en `(0, 0, vals)` el domain no se evalúa → puedes meter duplicados que luego fallan en `_check_approver_ids`.

### APPG23 — Doble botón "Submit" gated por `attachment_number`
`approvals/views/approval_request_views.xml:109-110`, `approval_request_view_form`: hay DOS botones `action_confirm`. El primario (`btn-primary`) es visible solo con `attachment_number >= 1`; el secundario (sin clase) con `attachment_number < 1`. Ambos invisibles si `not approver_ids or request_status != 'new'`. Consecuencia: **sin approvers el botón Submit NO aparece** (no hay error, simplemente no puedes enviar). Igual pasa con Approve (l.111-112, doble botón por `attachment_number` y `request_owner_id == uid`).

### APPG24 — Approve invisible si eres owner sin adjunto
`approvals/views/approval_request_views.xml:111`, `approval_request_view_form`: el botón Approve primario se oculta si `(attachment_number < 1 and request_owner_id == uid)`. Es decir, owner que también es approver de su propio request no ve Approve hasta que haya ≥1 adjunto. Mezcla la regla de auto-aprobación con la de documentos.

### APPG25 — `action_cancel` botón exige ser owner
`approvals/views/approval_request_views.xml:116`, `approval_request_view_form`: botón Cancel invisible si `request_owner_id != uid` (entre otras). Un officer (`group_approval_user`) que NO es owner **no ve Cancel en el header**, aunque la record rule se lo permitiría. El backend `action_cancel()` sí lo dejaría, pero la UI no expone el botón. Refuerza APPG15.

### APPG26 — `action_withdraw` botón requiere `user_status` no vacío
`approvals/views/approval_request_views.xml:114`, `approval_request_view_form`: Withdraw invisible si `user_status in ['waiting','pending','cancel'] or not user_status`. Solo visible si tu status es `approved` o `refused`. El `not user_status` (string vacío, ver APPG8) lo oculta a no-approvers. Coherente con que solo retractas tu propia decisión ya emitida.

### APPG27 — `_mail_post_access = 'read'`
`approvals/models/approval_request.py:12`, `class ApprovalRequest`: cualquiera con acceso de lectura al request puede postear en el chatter (no requiere write). Combinado con la record rule de approver (read si eres approver), un approver puede comentar sin poder editar el request.

### APPG28 — Record rule "request approver rule" incluye al jefe del owner
`approvals/security/approval_security.xml:35-40`, `approval_request_request_approver_rule`: la rule de `base.group_user` da acceso read/write al request no solo a owner y approvers, sino a `request_owner_id.parent_id.user_id` SI `category.manager_approval in ('approver','required')`. El jefe ve y edita el request **aunque aún no esté en approver_ids** (antes del recompute). Acceso "fantasma" por jerarquía hr.

### APPG29 — Unlink del owner solo en `new`/`cancel`, y vía `function` se restringe más
`approvals/security/approval_security.xml:59-68`, `approval_request_unlink_request_owner_rule` da unlink al owner si `request_status in ['cancel','new']`. Pero el bloque `<function model="ir.rule" name="write">` al final (l.205-208) **reescribe** ese `domain_force` a `[('request_owner_id.id','=',user.id), ('request_status','in',('new','cancel'))]` en instalación. Si una migración custom toca esta rule, ojo: el dominio efectivo lo fija ese `function`, no el `<record>`.

### APPG30 — Approver unlink permitido en `new/pending/cancel` (no en terminal)
`approvals/security/approval_security.xml:137-145`, `approval_approver_manager_unlink` (officer): puede borrar approvers si `request_id.request_status in ['new','pending','cancel']`. NO en `approved`/`refused`. Quitar un approver de un request `pending` puede recalcular `request_status` a `approved` si con eso se cumple `approval_minimum` y `required_approved` (ver APPG4) — borrar un approver pendiente puede "auto-aprobar" el request.

### APPG31 — `group_approval_user` NO es admin: config solo `group_approval_manager`
`approvals/security/approval_security.xml:7-19`: `group_approval_user` ("Officer") aprueba todo pero NO accede a configuración (categorías). Solo `group_approval_manager` ("Administrator", implica user) puede crear/editar `approval.category`. La access csv da CRUD de category solo al manager.

### APPG32 — `request_owner_id` domain excluye portal pero por `env.companies`
`approvals/models/approval_request.py:16-17`, `ApprovalRequest._get_request_owner_id_domain`: `_get_request_owner_id_domain` = `[('share','=',False), ('company_ids','in', env.companies.ids)]`. El owner debe ser interno Y pertenecer a una de las companies ACTIVAS del usuario (no todas las allowed). En multi-company con company switcher cerrado, usuarios de otras companies no son elegibles como owner.

### APPG33 — `attachment_ids` filtra por `res_field IS NULL` implícito vía `attachment_number`
`approvals/models/approval_request.py:60`, `class ApprovalRequest` `attachment_ids` domain es solo `res_model='approval.request'`, SIN filtrar `res_field`. Pero `ir_attachment._unlink_approved_approval_request` (l.16) sí exige `not attachment.res_field`. Las binarias de campos (ej. `image`) podrían contarse en `attachment_ids` aunque no en la protección de unlink. Para adjuntos "reales" la lógica de protección usa `res_field IS NULL`; el contador `attachment_number` (l.92-97) usa `_read_group` sin filtrar res_field tampoco → puede inflar el count con binarios de campo.

### APPG34 — `ir.attachment` no se puede borrar en estado terminal
`approvals/models/ir_attachment.py:10-22`, `class IrAttachment`: `@api.ondelete` impide borrar adjuntos de un request en `approved/refused/cancel` (UserError). Sube un PDF erróneo y aprueba → no puedes quitarlo salvo volviendo el request a un estado no terminal (que requiere withdraw/draft).

---

## approvals_purchase (bridge)

### APPP1 — `action_create_purchase_orders` es manual, no automático
`approvals_purchase/models/approval_request.py:50-118`, `ApprovalRequest.action_create_purchase_orders` se invoca SOLO desde el botón "Create RFQ's" en el form. Aprobar el request NO crea POs automáticamente. Si quieres auto: extiende `action_approve()` con llamada post-aprobación cuando `request_status == 'approved'`.

### APPP2 — Doble-click en "Create RFQ's" no duplica
`action_create_purchase_orders` l.54-55: si `purchase_order_count > 0`, return silencioso (no error). Pero **modificar línea + reclick NO genera ajuste** — la línea ya tiene `purchase_order_line_id` y el guard salta.

### APPP3 — Edit de línea TRAS approve sigue siendo posible
No hay `readonly` en `product_line_ids` si `request_status='approved'` y `purchase_order_count=0`. Editar quantity/product y luego crear RFQ usa valores nuevos sin re-aprobación. **Riesgo de incoherencia con la decisión aprobada.**

### APPP4 — Agrupación PO por (company, vendor, currency, state=draft)
`_get_purchase_orders_domain` l.87-99: busca PO existente con esos 4 criterios. Currency es del vendor (no de company). Dos vendors con misma partner pero distinta currency → 2 POs.

### APPP5 — Si encuentra PO existente, mergea o crea línea según UoM
`action_create_purchase_orders` l.67-95: si hay PO compatible y dentro hay `purchase.order.line` con mismo `product_id` y mismo `product_uom` → **suma `product_qty`**. Si no, crea nueva PO line en el mismo PO. **Resultado posiblemente sorprendente** al usuario que ve cantidades crecer en RFQs en draft.

### APPP6 — `origin` se appendea con `", "` y dedupe
l.97-103: parsea por `split(', ')`. Si origin contiene `; ` o `|` o similar (algún módulo custom), no dedupe y duplica.

### APPP7 — `search(po_domain)` sin limit; usa `purchase_orders[0]`
`approvals/models/approval_request.py:65`, `class ApprovalRequest`: `purchase_orders = self.env['purchase.order'].search(po_domain)` — **SIN `limit=1`** (el doc previo decía `limit=1`, era incorrecto). Devuelve TODAS las PO draft que casan vendor/currency/company. Para buscar la línea compatible busca en `('order_id', 'in', purchase_orders.ids)` (l.70-74, este sí `limit=1`). Si no hay línea compatible, mete la nueva línea en `purchase_orders[0]` (l.83) → orden por `_order` de purchase.order (`priority desc, id desc` en v18). No determinístico entre varias RFQs draft del mismo vendor; puede no ser la deseada.

### APPP8 — `_select_seller` puede cambiar el vendor según qty
`approvals_purchase/models/approval_product_line.py:47-60`, `ApprovalProductLine._compute_seller_id`: `_compute_seller_id` llama `product._select_seller(quantity=po_uom_qty)`. Si el producto tiene `product.supplierinfo` con diferentes precios según min_qty, cambiar `quantity` cambia el seller. Para forzar un vendor concreto: edita `seller_id` manualmente (es `readonly=False`), pero un onchange posterior lo sobrescribirá.

### APPP9 — `has_no_seller=True` y decoración roja en list
`approval_product_line_views.xml`: la list view tiene `decoration-danger="has_no_seller"`. Filas sin vendor aparecen en rojo. `_check_products_vendor` (l.78-85) bloquea `action_create_purchase_orders` con UserError "select a vendor".

### APPP10 — `action_cancel` NO cancela POs ya creadas
`approvals_purchase/models/approval_request.py:25-42`, `ApprovalRequest.action_cancel`: solo agenda mail.activity tipo `exception_approval_request_canceled` sobre cada PO. Las POs quedan en `draft` y deben cancelarse manualmente. Cancelar y volver a "Create RFQ's" generaría nuevas RFQs adicionales.

### APPP11 — `_domain_product_id` requiere context
`approvals/models/approval_product_line.py:10-22`, `class ApprovalProductLine`: si no hay `default_category_id` en context O `active_model != 'approval.category'`, devuelve `[]` → ningún producto seleccionable. Bug conocido (TODO en código). Abrir línea desde una action sin contexto → picker vacío.

### APPP12 — `action_approve` valida product_id pero no seller
`approvals_purchase/models/approval_request.py:20-23`, `ApprovalRequest.action_approve`: requiere que todas las líneas tengan `product_id`, pero NO valida seller. Puedes aprobar sin vendedores → al crear RFQs falla.

### APPP13 — UoM conversion `po_uom_qty`
`approvals/models/approval_product_line.py:34-45`, `class ApprovalProductLine`: `po_uom_qty = product_uom_id._compute_quantity(quantity, product.uom_po_id)`. Si el approval UoM y el `uom_po_id` están en categorías distintas (incoherencia), `_compute_quantity` lanza UserError "different category UoMs".

### APPP15 — `action_confirm` (purchase) bloquea request vacío
`approvals_purchase/approval_request.py:44-48`, `ApprovalRequest.action_confirm`: override de `action_confirm` — si `approval_type=='purchase'` y `not product_line_ids` → UserError "You cannot create an empty purchase request." Distinto del flujo base (que solo valida approvers/adjuntos). Para tipo purchase, las líneas de producto son obligatorias para enviar.

### APPP16 — `currency_id` de la PO sale de `seller_id.currency_id`, no del producto
`approvals_purchase/models/approval_product_line.py:97`, `ApprovalProductLine._get_purchase_orders_domain`, y `approvals_purchase/models/approval_product_line.py:115`, `ApprovalProductLine._get_purchase_order_values`: tanto el domain de búsqueda de PO como los vals de creación usan `self.seller_id.currency_id`. Si `seller_id` está vacío (sin vendor seleccionado y `_select_seller` no encontró ninguno), `currency_id = False` → la PO se crea/busca con currency `False`. Normalmente bloqueado antes por `_check_products_vendor` (`approvals_purchase/models/approval_product_line.py:78-85`, `ApprovalProductLine._check_products_vendor`), pero si `seller_id` se setea manualmente a un supplierinfo sin currency, la PO hereda False.

### APPP17 — `_select_seller` se llama con `uom_id=product.uom_po_id`
`approvals/models/approval_request.py:59-62`, `class ApprovalRequest` y `approvals_purchase/models/approval_product_line.py:51-54`, `ApprovalProductLine._compute_seller_id`, y `:67-69`: `_select_seller(quantity=po_uom_qty, uom_id=product.uom_po_id)`. El matching de `product.supplierinfo.min_qty` se hace en la UoM de compra del producto, NO en la UoM de la línea de approval. Si configuras min_qty pensando en la UoM de la línea, el seller elegido puede no ser el esperado.

### APPP18 — `_compute_seller_id` tiene fallback a "cualquier vendor"
`approvals_purchase/models/approval_product_line.py:55-60`, `ApprovalProductLine._compute_seller_id`: si `_select_seller` con la qty real no encuentra vendor, reintenta con `quantity=None` para mostrar *algún* vendor. `_compute_has_no_seller` (`approvals_purchase/models/approval_product_line.py:63-76`, `ApprovalProductLine._compute_has_no_seller`) hace lo mismo. Resultado: `seller_id` puede mostrar un vendor cuyo min_qty NO cubre la cantidad pedida (solo para no dejar la línea sin vendor visible). El precio/condiciones reales al crear la PO pueden diferir.

### APPP14 — Re-cancel: PO en draft NO se borra
La activity en PO sobrevive tras cancelaciones múltiples del approval. En v19 (l.24-62) la lógica fue refactorizada: en POs draft elimina las líneas; en otros estados solo notifica.

---

## approvals_purchase_stock (bridge stock)

### APPS1 — `warehouse_id` solo visible con multi-warehouse
`approvals/views/approval_product_line_views.xml:9-12`, `approval_product_line_view_tree_independent`: `groups="stock.group_stock_multi_warehouses"`. Setup single-warehouse no ve el campo. Si tras configurar single-WH activas multi, las líneas existentes tienen `warehouse_id=False` → fallback al picking_type por default.

### APPS2 — `hide_location` oculta el campo en form
`approvals/models/approval_request.py:11-19`, `class ApprovalRequest` (stock bridge): si `approval_type='purchase' AND multi-warehouse` → `hide_location=True`. El campo `location` del approval queda oculto porque el warehouse va por línea, no por request.

### APPS3 — Mismo producto, distintos warehouses → 2 POs
`_get_purchase_orders_domain` override añade `picking_type_id` al domain. Dos líneas con mismo product+vendor pero distinto `warehouse_id` → genera 2 POs distintas (cada una con su picking_type).

### APPS4 — Stock module NO genera pickings al aprobar
Solo añade `picking_type_id` a la PO. Los pickings los crea purchase al confirmar la PO (`button_confirm`). El approval workflow termina en RFQ, no en receipt.

### APPS5 — `warehouse_id` sin `in_type_id` → PO sin picking_type
Si configuras un warehouse sin tipo de operación de entrada (raro), `_get_picking_type()` devuelve `False`. El domain y los vals usan `False`. Resultado: PO sin picking_type → receipts manuales.

---

## v19 — gotchas de migración

### APPG-v19-1 — `approval.approver` extraído a fichero propio
Customizaciones que importaban `from odoo.addons.approvals.models.approval_request import ApprovalApprover` deben cambiar a `from odoo.addons.approvals.models.approval_approver import ApprovalApprover`. Modelo `approval.approver` sigue igual.

### APPG-v19-2 — `_ensure_can_approve()` eliminado
v18: método público (`approvals/models/approval_request.py:175-177`, `ApprovalRequest._ensure_can_approve`). v19: lógica inline en `action_approve`. Overrides → `AttributeError`.

### APPG-v19-3 — `@api.constrains` → `models.Constraint`
v19 (`approvals/models/approval_approver.py:82-85`, `class ApprovalApprover`): `_unique_request_user = models.Constraint('UNIQUE(request_id, user_id)', 'A user cannot be added twice...')`. Migraciones de customizaciones: redefinir como Constraint o seguir usando `@api.constrains` extra (que se sumará).

### APPG-v19-4 — `product_uom_category_id` eliminado del base
v18 `approvals/models/approval_product_line.py:26`, `class ApprovalProductLine` lo tenía como related; v19 no existe. Vistas custom que lo referencien → ParseError.

### APPP-v19-1 — `uom_po_id` → `uom_id` en approvals_purchase
**Breaking masivo**: 7+ ocurrencias cambiadas. v19 ya no usa `uom_po_id`. Si custom code lee este campo en `approval.product.line` flow → falla.

### APPP-v19-2 — `_prepare_purchase_order_line` signature cambió
v18 pasaba `seller` (product.supplierinfo); v19 pasa `vendor` (res.partner) y `seller_uom_qty` pre-convertido. Overrides → TypeError o vals incorrectos.

### APPP-v19-3 — `_get_purchase_orders_domain` retorna `Domain` object
v18: lista de tuplas. v19: `from odoo.fields import Domain` → `Domain(...)`. Overrides que hagan `super().extend([...])` fallan. Hay que usar `&=`.

### APPS-v19-1 — `expression.AND` → `domain &= Domain(...)`
v19 (`approvals_purchase_stock/approval_product_line.py:33`, `ApprovalProductLine._get_purchase_orders_domain`). Imports cambian.

### APPG-v19-5 — Write impide auto-aprobación
v19 `approvals/models/approval_request.py:342-353`, `ApprovalRequest.write`: si `not self.env.is_admin()` y el user es el propio approver de su request → AccessError. Flujos custom que permitían owner=approver1 deben ajustarse (owner separado).

### APPG-v19-6 — Status `'pending'` string "To Approve" → "Submitted"
Reports, dashboards y traducciones hardcoded fallan. Ningún cambio técnico, solo label.

### APPG-v19-7 — Security: `category_id` → `privilege_id`
v19 introduce `res.groups.privilege`. Grupos custom que cuelgan de la category `module_category_human_resources_approvals` deben migrar a `res_groups_privilege_approvals`.

### APPG-v19-8 — `copy()` → `copy_data()`
v19 `approvals/models/approval_request.py:107-109`, `ApprovalRequest.create` override de `copy_data` añade " (copy)" al name. Si tenías override de `copy()`, no se ejecutará en v19 si llamas `copy_data` directamente.
