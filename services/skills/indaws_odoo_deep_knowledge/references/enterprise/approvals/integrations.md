# Approvals — Integrations

Cómo `approvals` se conecta con otros módulos. Foco en bridges existentes (purchase, stock) y patrones de extensión.

---

## Bridges oficiales

### `approvals_purchase` — RFQ desde approval

**Trigger**: botón manual "Create RFQ's" en `approval.request` form, visible si `approval_type='purchase' AND request_status='approved' AND purchase_order_count==0`.

**Pipeline**:

```
approval.request (state=approved)
  └─ action_create_purchase_orders()  [approvals_purchase/approval_request.py:50]
       └─ por cada approval.product.line:
            ├─ resolver seller via _select_seller(quantity=po_uom_qty)
            ├─ buscar PO existente: domain[company, vendor, currency, state=draft]
            │    ├─ encontrada con line(product, uom) compatible → product_qty += po_uom_qty
            │    └─ encontrada sin line compatible → crea nueva po.line en mismo PO
            └─ no encontrada → crea purchase.order + po.line
       └─ append origin con name del approval (dedupe por ", ")
       └─ asigna line.purchase_order_line_id (link bidirec)
```

**Smart button "Purchase Orders"** (`approvals/views/approval_request_views.xml:9-15`, `approval_request_action`): muestra `purchase_order_count`, abre `action_open_purchase_orders`.

**Domain de búsqueda PO** (`approvals_purchase/models/approval_product_line.py:87-99`, `ApprovalProductLine._get_purchase_orders_domain`):
- `company_id`
- `partner_id` = seller.partner_id
- `currency_id` = seller.currency_id (**vendor currency, no company currency**)
- `state = 'draft'`

**Vals de creación PO** (`approvals_purchase/models/approval_product_line.py:101-117`, `ApprovalProductLine._get_purchase_order_values`):
- `origin = approval.name`
- `partner_id`, `company_id`, `currency_id` (del seller)
- `payment_term_id = vendor.property_supplier_payment_term_id`
- `fiscal_position_id` via `account.fiscal.position._get_fiscal_position(vendor)`

### `approvals_purchase_stock` — warehouse por línea

Añade `warehouse_id` a `approval.product.line` (visible si `stock.group_stock_multi_warehouses`).

Override `_get_purchase_orders_domain` añade `('picking_type_id', '=', warehouse.in_type_id.id)`.
Override `_get_purchase_order_values` añade `picking_type_id` a vals.

**Efecto**: 2 líneas con mismo product+vendor pero distinto `warehouse_id` → 2 POs separadas (cada una con su picking_type).

`hide_location` computed (`approvals/models/approval_request.py:11-19`, `class ApprovalRequest`): si `approval_type='purchase' AND multi-warehouse` → oculta el field `location` del request (porque el warehouse va por línea).

---

## mail (chatter, activities)

### Activity type
`data/mail_activity_data.xml` registra `approvals.mail_activity_data_approval` (icon "fa-check", res_model="approval.request").

### Ciclo de activities

**Al `action_confirm`** (l.155 + l.405-409 de approval.approver):
- Por cada approver con status `pending` (no `waiting`), `_create_activity()` agenda `mail.activity` para `user_id`.

**Al aprobar/rechazar individual** (`action_approve`, `action_refuse`):
- `activity_feedback([type_xmlid], user_id=approver.user_id)` cierra la activity dejando traza en chatter.

**Al cancelar** (`action_cancel`):
- `mail.activity.search([res_model, res_id])._activity_done()` o `.unlink()` (sin feedback) — depende del flow.

**Al llegar a estado terminal** (`approved/refused/cancel`):
- `_compute_request_status` ejecuta `_cancel_activities()` (l.293): borra todas las pendientes.

### chatter subscription
`approvals/models/approval_request.py:113`, `ApprovalRequest.create`: en `create()`, el `request_owner_id` se auto-suscribe al chatter (mail.followers).
`approvals/models/approval_request.py:330-338`, `ApprovalRequest.write`: si cambias `request_owner_id`, desuscribe al viejo y suscribe al nuevo.

### v19 — mail.activity extendida
`odoo_enterprise/19/approvals/models/mail_activity.py` añade:
- `approval_request_id` (computed con `search` callable).
- `approver_id` (computed).
- `_to_store_defaults` (reemplaza `_to_store` de v18). Permite serializar approval_request_id al frontend.

---

## ir.attachment

`approval.request.attachment_number` (computed) cuenta archivos donde `res_model='approval.request' AND res_field IS NULL` (l.16 de `ir_attachment.py`). Filter excluye binarios que están en campos específicos (ej. firmas).

`ir.attachment.unlink` override (l.10-22): bloquea borrado de attachments cuyo request padre esté en `approved/refused/cancel`. Mensaje UserError.

**Edge case**: si attachment está en campo binary de un wizard o módulo externo apuntando a `approval.request`, también se cuenta (no hay filter por field).

---

## hr.employee — manager_approval

`approval_request._compute_approver_ids` (l.305-317):
```
hr.employee.search([('user_id','=',owner.id), ('company_id','=',self.company_id.id)], limit=1)
employee.parent_id.user_id  # el manager Odoo del employee
```

Si `manager_approval='required'` y no hay `parent_id` o el parent no tiene `user_id` → `action_confirm` lanza UserError.

**Gotchas**:
- Multi-company: busca employee en `self.company_id`. Si owner tiene employees en varias companies, depende del contexto.
- Manager hierarchy: solo el manager **directo** (`parent_id`), no escala árbol.

---

## product (approval.product.line)

`approval.product.line.product_id` → `product.product` (no template).
Sin lógica adicional en base: las líneas son meramente descriptivas, no afectan stock ni accounting hasta que `approvals_purchase` las convierte en `purchase.order.line`.

`product_uom_id` puede divergir de `product.uom_id` (el campo computed con precompute=True copia el del producto, pero es editable). UoM conversion solo se hace en `approvals_purchase._compute_po_uom_qty`.

---

## ir.sequence — automated_sequence

`approval.category` con `automated_sequence=True` crea un `ir.sequence` (l.118-146 de approval_category.py):
- `prefix = sequence_code` (configurable)
- `padding = 4`
- `company_id = category.company_id`

Si editas `sequence_code` post-creación → el ir.sequence se actualiza in-place (`prefix` cambia). Requests previos mantienen su `name` original.

Si editas `company_id` de la category → ir.sequence también se mueve. Cuidado en multi-company: el sequence puede dejar de coincidir con la company de los próximos requests.

---

## Extender approval_type — patrón de bridge

Para crear tu propio bridge tipo "approvals_X":

1. **Manifest**: `depends: ['approvals', 'X']`, `auto_install: True`.
2. **Extender `approval.category.approval_type`** con un nuevo valor:
   ```python
   class ApprovalCategory(models.Model):
       _inherit = 'approval.category'
       approval_type = fields.Selection(selection_add=[('my_type', 'My Type')], ondelete={'my_type': 'cascade'})
   ```
3. **`@api.onchange('approval_type')`**: forzar `has_*` defaults si tu type lo requiere (ej. `has_product='required'`).
4. **Añadir botón** en `approval_request_views.xml` visible si `approval_type=='my_type' AND request_status=='approved'`.
5. **Action method** que extraiga datos del request y cree el objeto destino (ej. tu wizard, contrato, viaje, etc.).
6. **Smart button** + `_compute_X_count` para count + `action_open_X` para drill-down.
7. **Data**: añadir `approval.category` predefinida con tu `approval_type`.
8. **(Opcional)** sobrecargar `action_cancel` para notificar/cancelar el objeto destino.

---

## Cross-module gotchas

### Sale (no bridge oficial)
No existe `approvals_sale`. Para "aprobar antes de crear SO": custom bridge siguiendo el patrón anterior. OCA tiene módulos similares (no exactos).

### Account (no bridge oficial)
Aprobaciones de facturas en Odoo nativo se gestionan vía `account.move.state` (`draft → posted`) + `mail.activity`, o vía `account_followup`. `approvals` puede usarse como wrapper si quieres workflow multi-approver previo a posting, pero requiere bridge custom.

### Expense
`hr.expense.approve` tiene su propio flow (manager → finance) sin pasar por `approvals`. No mezclar.

### Spreadsheets / Documents
Sin integración. Los archivos adjuntos en approval son ir.attachment normales — pueden enlazarse desde `documents` solo si configuras manualmente la carpeta destino.

---

## v19 — server action "Force Approval"

`odoo_enterprise/19/approvals/views/approval_request_views.xml:303-315`, `model_approval_request_approve_admin` define `model_approval_request_approve_admin`:
- Tipo: `ir.actions.server` con `code = self._action_force_approval()`.
- Grupos: `group_approval_user` (officers).
- Disponible en form y list (bind via `model_id`).
- Marca TODOS los approvers como `approved` directamente — bypassea sequencing y required.

Caso de uso: officer interviene para destrabar requests bloqueadas (e.g., approver indisponible).
