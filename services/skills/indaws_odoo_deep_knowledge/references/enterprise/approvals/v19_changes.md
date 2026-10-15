# Approvals — v18 → v19 changes

Diff completo de los 3 módulos. Origen del scan: `odoo_enterprise/18/approvals*` vs `odoo_enterprise/19/approvals*`.

---

## Resumen ejecutivo

| Área | Cambios |
|---|---|
| Models | `approval.approver` extraído a fichero propio; nuevo `approval_properties` (Properties) |
| Constraints | `@api.constrains('approver_ids')` → `models.Constraint('UNIQUE')` |
| Workflow | Eliminado `_ensure_can_approve()`; nueva server action "Force Approval"; bloqueo de auto-aprobación non-admin |
| Security | `category_id` de grupos → `privilege_id` (modelo `res.groups.privilege`) |
| UoM | `uom_po_id` → `uom_id` en TODO approvals_purchase |
| Domain | `expression.AND(...)` → `Domain(...) &= ` |
| UI | Kanban rediseñada; campo `active` + ribbon "Archived"; widget `many2one_uom`; status label "To Approve" → "Submitted" |
| Bridges | approvals_purchase: `_create_purchase_orders` refactorizado en submétodos con logging chatter; approvals_purchase_stock: domain refactor |

---

## A. approvals — estructural

### Modelos nuevos

- **`approval.approver`** ahora en fichero propio: `models/approval_approver.py` (v18 estaba anidado al final de `approval_request.py`).
- Constraint declarativo (v19 only):
  ```python
  _unique_request_user = models.Constraint(
      'UNIQUE(request_id, user_id)',
      'A user cannot be added twice as an approver on the same request.'
  )
  ```

### Campos nuevos

**approval.request:**
- `active: fields.Boolean(default=True, tracking=True)` — l.20.
- `approval_properties: fields.Properties('Properties', definition='category_id.approval_properties_definition')` — l.64.

**approval.category:**
- `approval_properties_definition: fields.PropertiesDefinition('Approval Properties')` — l.70.

**mail.activity (heredada):**
- `approval_request_id: fields.Many2one(..., compute=..., search=...)` — l.12.
- `approver_id: fields.Many2one('approval.approver', compute=...)` — l.13.

### Campos eliminados

**approval.product.line:**
- `product_uom_category_id` (related) eliminado. Vistas y filters que lo referencien rompen.
- Campo `product_uom_id` perdió `domain="[('category_id', '=', product_uom_category_id)]"`.

### Campos modificados

- `approval.product.line.approval_request_id`: añadido `index=True`.
- `approval.category.approver.category_id`: añadido `index=True`.

---

## B. approvals — funcional

### Métodos eliminados

- `_ensure_can_approve()` (v18 `approvals/models/approval_request.py:175-177`, `ApprovalRequest._ensure_can_approve`). Lógica inline en `action_approve` (v19 l.214-215).
- `@api.constrains('approver_ids') _check_approver_ids()` eliminado del request. Migrado a `models.Constraint` en approver.

### Métodos nuevos

**approval.request:**
- `copy_data(default=None)` — añade " (copy)" al name (l.107-109).
- `_unlink_except_status_is_approved()` con `@api.ondelete(at_uninstall=False)` — bloquea borrado en estados terminales (l.131-135).
- `_action_force_approval()` — marca todos approvers como `approved`, solo officers (l.209-217).

**mail.activity:**
- `_compute_approval_request_id`, `_search_approval_request_id`, `_compute_approver_id` — l.14-42.
- `_to_store_defaults(target)` reemplaza `_to_store(store)` (l.44-45).

### Métodos modificados

**`_compute_request_status` (l.262-303 v19 vs l.273-305 v18):**
- v19 captura `old_status` al inicio.
- Si transición a `approved`/`refused` post-compute → envía notificación inline (lógica antes en `action_approve`/`action_refuse`).
- Centraliza el envío de notificaciones de cambio de estado.

**`action_approve`/`action_refuse`:**
- Eliminada lógica de notificación inline (movida a compute).
- `action_approve` ahora valida con `if request_status == 'approved': raise UserError` inline (no `_ensure_can_approve`).

**`write` override (v19 NUEVO l.342-353):**
```python
def write(self, vals):
    if not self.env.is_admin() and any(
        request.request_owner_id == self.env.user 
        and any(approver.user_id == self.env.user for approver in request.approver_ids)
        for request in self
    ):
        # bloquear si user intenta cambiar status como approver de su propio request
        raise AccessError(...)
```
Impide que el mismo usuario sea owner + approver y se auto-apruebe.

**`unlink_attachments` y `_unlink_except_status_is_approved`:**
- v19 añade `@api.ondelete(at_uninstall=False)` decorators. Antes era unlink override puro.

### Status string cambio

`approval.request.request_status` selection (l.53 v19):
```
('pending', 'To Approve')  # v18
('pending', 'Submitted')   # v19
```

---

## C. approvals — UI

### approval_request_views.xml

**List/Form:**
- `create="false"` REMOVIDO de list y form (l.52, 109).
- Campo `active` invisible (l.126).
- Widget `web_ribbon` "Archived" en form (l.153).
- Campo `approval_properties` añadido (l.63 list, l.195 form).
- Campo `user_id` en approver list ahora con `widget="many2one_avatar_user"` (l.216).
- Botón "Cancel" simplificó invisibility: eliminó `or not user_status and not has_access_to_request` (l.121).

**Search:**
- Filtro "Archived" añadido (l.85).
- Filtros de actividades: `context_today().strftime('%Y-%m-%d')` → `'today'` (l.87-91).
- Filtro "Request Status" en Group By (l.97-98).

**Kanban — rediseño completo (l.252-295):**
- v18: `<t t-name="menu">` con menú contextual (Edit/Delete/Approve/...).
- v19: `<t t-name="card">` con layout 2 columnas:
  - Imagen de categoría con preview.
  - Badge de estado top-right.
  - Botones approve/refuse en footer.
  - Fechas con `luxon.DateTime`.

**Server actions:**
- `model_approval_request_approve_admin` — "Force Approval" para officers (l.303-315).

### approval_category_views.xml

- Action name "Approvals Types" → "Approval Categories" (l.11).
- Label "Approval Type" → "Approval Category" (l.65).
- `<separator>` de approvers → `<label nolabel>` (l.95-97).
- Settings dialog: `"name": "Approval Settings"` añadido.

### approval_product_line_views.xml

- Widget `many2one_uom` en TODOS los `product_uom_id` (l.17, 33, 50).
- Campo `product_uom_category_id` removido de vistas.

### Vistas eliminadas
- `res_users_views.xml` — eliminado en v19.

### Assets reorganizados

Manifest v18:
```python
'web.assets_backend': ['approvals/static/src/**'],
```
Manifest v19:
```python
'web.assets_backend': [
    'approvals/static/src/common/**',
    'approvals/static/src/web/**',
    'approvals/static/src/views/**',
    'approvals/static/src/scss/**',
],
'mail.assets_public': ['approvals/static/src/common/**'],
```

Nuevas vistas QWeb client-side:
- `static/src/web/activity_patch.xml`.
- `static/src/web/activity_list_popover_item_patch.xml`.
- `static/src/web/approval.xml`.

---

## D. approvals_purchase — cambios

### Modelos nuevos
- `purchase_order.py` (l.1-48): hereda `purchase.order` para logging de cambios de estado.
- Importado en `approvals/models/__init__.py:8`, `mail_activity`.

### Manifest
- Añadido `'author': 'Odoo S.A.'` (l.24).

### approval_request.py — refactor masivo

**`action_cancel` (v18 l.25-42 → v19 l.24-62):**
- Lógica reextraída a `_cancel_approval_request()`.
- En POs `state='draft'`: elimina las líneas (no solo notifica).
- En POs en otros estados: log_cancellation_exception en chatter del PO.
- Métodos nuevos:
  - `_log_po_cancellation_to_chatter()` (l.162-176).
  - `_log_cancellation_exception_to_po_chatter()` (l.178-188).

**`action_create_purchase_orders` (v18 l.50-118 → v19 l.70-76):**
- Ahora wrapper: delega a `_create_purchase_orders()` + llama `_log_po_creation_to_chatter()`.
- `_create_purchase_orders` contiene la lógica (l.78-142).

**Métodos helper nuevos:**
- `_get_order_data_from_product_lines()` (l.190-213) — estructura datos por (vendor, line).
- `_generate_po_log_message()` (l.215-232) — mensajes formateados para chatter.
- `_log_po_creation_to_chatter()` (l.159-160).

### UoM change — BREAKING

v18 usaba `product.uom_po_id` (deprecated en v19 base). v19 usa `product.uom_id`:

| Fichero | v18 | v19 |
|---|---|---|
| `approvals/models/approval_request.py:61`, `ApprovalRequest.attachment_number` y `approvals/models/approval_request.py:73`, `ApprovalRequest.requirer_document` | `uom_id=line.product_id.uom_po_id` | `uom_id=line.product_id.uom_id` (l.83) |
| `approvals_purchase/models/approval_product_line.py:42-75`, `ApprovalProductLine._compute_po_uom_qty` (4+ ocurrencias) | `uom_po_id` | `uom_id` |
| Domain PO line search (`approvals/models/approval_request.py:73-95`, `ApprovalRequest.requirer_document`) | `('product_uom', '=', uom_po_id.id)` | `('product_uom_id', '=', seller.product_uom_id.id or product.uom_id.id)` |

### `_prepare_purchase_order_line` signature — BREAKING

v18 (`approvals/models/approval_request.py:84-90`, `class ApprovalRequest`):
```python
self.env['purchase.order.line']._prepare_purchase_order_line(
    line.product_id,
    line.quantity,
    line.product_uom_id,
    line.company_id,
    seller,        # product.supplierinfo
    purchase_order,
)
```

v19 (`approvals/models/approval_request.py:107-114`, `ApprovalRequest.create`):
```python
seller_uom_qty = line.product_uom_id._compute_quantity(line.quantity, seller.product_uom_id)
self.env['purchase.order.line']._prepare_purchase_order_line(
    line.product_id,
    seller_uom_qty,             # qty pre-convertida
    seller.product_uom_id,      # UoM del seller, no del producto
    line.company_id,
    vendor,                     # res.partner, no supplierinfo!
    purchase_order,
)
```

### Domain refactor

v18 `_get_purchase_orders_domain`:
```python
return [
    ('company_id', '=', self.company_id.id),
    ('partner_id', '=', vendor.id),
    ('state', '=', 'draft'),
    ('currency_id', '=', self.seller_id.currency_id.id),
]
```

v19:
```python
from odoo.fields import Domain
return Domain('company_id', '=', self.company_id.id) \
       & Domain('partner_id', '=', vendor.id) \
       & Domain('state', '=', 'draft') \
       & Domain('currency_id', '=', self.seller_id.currency_id.id)
```

Overrides custom que hagan `return super()._get_purchase_orders_domain() + [...]` rompen — usar `&=`.

### Field rename product_line

- `"Purchase UoM Quantity"` → `"Purchase Unit Quantity"`.
- Help: "UoM used by the product" → "Unit used by the product".

### Data

- `groups_id` → `group_ids` en `approval_demo.xml`.
- Nuevo registro `base.default_user_group` con `implied_ids`.

---

## E. approvals_purchase_stock — cambios

### Manifest
- Añadido `'author': 'Odoo S.A.'`.

### approval_product_line.py refactor mínimo

v18:
```python
from odoo.osv import expression
from odoo import api, fields, models

domain = super()._get_purchase_orders_domain(vendor)
picking_type = self._get_picking_type()
return expression.AND([domain, [('picking_type_id', '=', picking_type.id)]])
```

v19:
```python
from odoo.fields import Domain
from odoo import fields, models  # api removido

domain = super()._get_purchase_orders_domain(vendor)
picking_type = self._get_picking_type()
return domain & Domain('picking_type_id', '=', picking_type.id)
```

---

## F. Cross-cutting

### Security migration

v18 `approvals/security/approval_security.xml:2-3`:
```xml
<record model="ir.module.category" id="base.module_category_human_resources_approvals">
    ...
</record>
<record id="group_approval_user">
    <field name="category_id" ref="base.module_category_human_resources_approvals"/>
    ...
</record>
```

v19 (l.3-23):
```xml
<record model="res.groups.privilege" id="res_groups_privilege_approvals">
    <field name="name">Approvals</field>
    <field name="category_id" ref="base.module_category_human_resources"/>
    <field name="sequence">10</field>
</record>
<record id="group_approval_user">
    <field name="privilege_id" ref="res_groups_privilege_approvals"/>
    <field name="sequence">10</field>
    ...
</record>
```

Grupos custom que cuelgan de la category vieja deben migrar a `privilege_id`.

### Admin default eliminado

v18 forzaba `group_approval_manager` en `base.default_user`. v19 no — depende del setup individual.

### Data fields rename

- `groups_id` → `group_ids` en TODOS los XML data files (l.5 de `approval_category_data.xml`, etc.).

---

## G. Migration playbook

### Custom modules — checklist

**Models:**
- [ ] Si haces `_inherit('approval.request')` con override de `unlink`: añadir `@api.ondelete` decorator.
- [ ] Si tienes `@api.constrains('approver_ids')` → considerar migrar a `models.Constraint('UNIQUE')` o mantener el `@api.constrains` como adicional.
- [ ] Si llamas `_ensure_can_approve()` → reemplazar con check inline (`if request_status != 'pending' or user_status != 'pending'`).
- [ ] Si haces `copy()` override → migrar a `copy_data()`.

**approvals_purchase customizations:**
- [ ] Reemplazar `uom_po_id` → `uom_id` en cualquier referencia a `approval.product.line` o `approval.request` lógica de POs.
- [ ] Ajustar llamadas a `_prepare_purchase_order_line`: pasar `vendor` (partner) en vez de `seller`, `seller_uom_qty` pre-convertida.
- [ ] Si override `_get_purchase_orders_domain` → usar `Domain(...) &=` no `+ [...]`.
- [ ] Búsqueda de `purchase.order.line` por UoM: `product_uom` → `product_uom_id`.
- [ ] Si override `action_cancel` para extender lógica de POs → preservar las nuevas llamadas a `_log_po_cancellation_to_chatter()` y `_log_cancellation_exception_to_po_chatter()`.

**approvals_purchase_stock customizations:**
- [ ] `from odoo.osv import expression` → `from odoo.fields import Domain`.
- [ ] `expression.AND([d1, d2])` → `d1 & d2`.

**Security:**
- [ ] Grupos custom que cuelgan de `base.module_category_human_resources_approvals` → migrar a `res_groups_privilege_approvals`.

**Vistas:**
- [ ] Referencias a `product_uom_category_id` en xpath/domain → eliminar (campo ya no existe).
- [ ] Custom `kanban` view extendiendo el base → revisar, estructura cambió radicalmente.
- [ ] Vistas que dependan de `res_users_views.xml` (no existía mucho) → adaptar.

**i18n:**
- [ ] Re-traducir "To Approve" → "Submitted" en cualquier hardcoded.
- [ ] Re-traducir "Approval Type" → "Approval Category" labels.

**Tests:**
- [ ] Tests que esperan `request_status='pending'` con label "To Approve" en UI texts → ajustar.
- [ ] Tests que insertan owner==approver → fallarán por bloqueo de auto-aprobación (l.342-353 v19). Usar admin o separar.

### Workflow customization v19-specific

- Properties (`approval_properties` field): se puede usar para añadir metadatos sin crear módulo. Permite filtros dinámicos en search.
- "Force Approval" server action: si la quieres ocultar para algún officer concreto, restringir el grupo `group_approval_user` o sobrescribir el método `_action_force_approval`.
