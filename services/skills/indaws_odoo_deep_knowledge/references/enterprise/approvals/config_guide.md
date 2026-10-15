# Approvals — Config Guide

Recetas de configuración para escenarios típicos.

---

## 1. Aprobación simple manager + officer

**Objetivo**: el manager del owner aprueba primero, luego un officer (admin) confirma.

Configuración `approval.category`:
- `manager_approval = 'required'` (manager OBLIGATORIO).
- `approver_ids`: añadir 1 officer con `required=True`.
- `approval_minimum = 2`.
- `approver_sequence = True` (manager primero por su sequence=9 hardcoded).

**Resultado**: action_confirm → manager `pending`, officer `waiting`. Tras aprobación manager → officer `pending`. Tras officer → `approved`.

**Gotcha**: si owner no tiene `hr.employee.parent_id.user_id`, el confirm falla con UserError. Pre-condición: setup hr.

---

## 2. Aprobación con N officers, M requeridos

**Objetivo**: 5 officers pueden aprobar, basta con 3.

Configuración:
- `approver_ids`: añadir 5 officers con `required=False`.
- `approval_minimum = 3`.
- `approver_sequence = False` (en paralelo).

**Resultado**: 5 activities simultáneas. Tras 3 `approved` → request `approved`.

**Gotcha APPG6**: si por error pones `approval_minimum=10`, `minimal_approver = min(10, 5) = 5` → silenciosamente requiere TODOS, no falla. Revisa constraint `_constrains_approval_minimum`.

---

## 3. Aprobación con required override

**Objetivo**: 4 officers, pero el de finanzas SIEMPRE debe aprobar (incluso si los otros 3 ya lo hicieron).

Configuración:
- `approver_ids`: 3 officers `required=False` + 1 finance `required=True`.
- `approval_minimum = 2`.

**Resultado**: `_compute_request_status` solo pasa a `approved` cuando finance.status=`approved` Y count(`approved`) >= 2.

**Gotcha APPG4**: si finance no actúa NUNCA, request bloqueado en `pending` indefinidamente. UI no avisa. Considera añadir `manager_approval='approver'` (no required) + escalado SLA via cron custom.

---

## 4. Auto-numeración con prefix custom

**Objetivo**: requests numerados `CTR/2025/0001` (contracts).

Configuración:
- `automated_sequence = True`.
- `sequence_code = 'CTR'`.
- `sequence_id` se crea automáticamente con prefix.

**Gotcha APPG20**: si luego cambias `sequence_code` a `CTR2`, todos los **próximos** requests usarán `CTR2`. Los existentes mantienen su nombre.

---

## 5. Documents required (attachment obligatorio)

Configuración:
- `requirer_document = 'required'`.
- En el form del request, widget `attach_document` muestra warning si `attachment_number < 1`.

`action_confirm` (l.147-151) bloquea con UserError si no hay attachments.

**Gotcha**: el widget cuenta solo `res_model='approval.request' AND res_field IS NULL`. Si subes un binario en un campo Binary custom, no cuenta.

---

## 6. Approval type purchase (RFQ generation)

Para clientes con flujo "solicitud → aprobación → RFQ":

1. Instalar `approvals_purchase` (auto-install si `purchase` + `approvals`).
2. Crear `approval.category`:
   - `approval_type = 'purchase'`.
   - `has_product = 'required'` (forzado por onchange).
   - `has_quantity = 'required'` (forzado).
   - `manager_approval` según política.
3. Setear `seller_ids` correctamente en cada `product.product` (mínimo 1 por producto).
4. Tras `request_status='approved'`, usuario clica "Create RFQ's" manualmente.

**Customer journey**:
- Compra propone líneas (product + qty).
- Manager aprueba.
- Officer compras revisa RFQs generadas y las envía.

**Gotcha APPP1**: el botón es manual. Para auto-generar al aprobar:
```python
def action_approve(self):
    res = super().action_approve()
    for r in self.filtered(lambda x: x.request_status == 'approved' and x.approval_type == 'purchase'):
        if r.purchase_order_count == 0:
            r.action_create_purchase_orders()
    return res
```

**Gotcha APPP2**: si re-clicas tras edit de líneas, NO regenera. Cancela la PO en draft y vuelve a clicar (re-detecta líneas sin `purchase_order_line_id`).

---

## 7. Multi-warehouse purchase

Setup adicional sobre #6:
1. Instalar `approvals_purchase_stock`.
2. Activar `stock.group_stock_multi_warehouses` en al menos un user.
3. `approval.product.line.warehouse_id` aparece en form (gated by group).
4. Configurar cada warehouse con `in_type_id` válido.

**Resultado**: una línea por warehouse → una PO por warehouse (cada una con su picking_type → su receipt).

**Gotcha APPS3**: mismo product+vendor en dos warehouses → 2 POs, NO una sola con dos lines.

---

## 8. Security: usuario que solo ve sus requests

Setup default: `base.group_user` sin `approvals.group_approval_user`.

Record rule `approval_request_request_approver_rule` (`approvals/security/approval_security.xml:27-40`, `approval_request_request_approver_rule`):
```python
domain = ['|', '|',
    ('request_owner_id', '=', user.id),
    ('approver_ids.user_id', '=', user.id),
    '&',
        ('category_id.manager_approval', 'in', ('approver', 'required')),
        ('request_owner_id.parent_id.user_id', '=', user.id),
]
```

Usuario normal ve:
1. Sus propios requests.
2. Requests donde es approver.
3. Requests donde es manager (vía hr) si la categoría tiene `manager_approval`.

**Gotcha**: NO puede borrar requests except `state in (cancel, new)` AND ser owner. Officers pueden ver TODO (`domain = [(1,'=',1)]`) pero borrar solo en `cancel`.

---

## 9. Workflow secuencial estricto

**Objetivo**: 3 approvers que firman EN ORDEN (1 → 2 → 3).

Configuración:
- `approver_ids`: 3 officers con sequence 10, 20, 30 (orden estricto).
- `approver_sequence = True`.
- `approval_minimum = 3`.

**Resultado**: el primero ve activity y `status='pending'`. Los otros 2 en `waiting`. Tras 1ª aprobación → 2º `pending`.

**Gotcha APPG2**: si tienes `manager_approval='approver'`, el manager se inyecta con sequence=9 → aprueba ANTES que el sequence=10. Para forzar otro orden: edita `sequence` de approver tras recompute.

**Gotcha APPP17 (withdraw)**: si approver 2 hace withdraw tras aprobar, los posteriores vuelven a `waiting`. Approver 3 deja de ser `pending` aunque ya aprobó. El sistema mantiene la cadena correctamente.

---

## 10. Categorización por département

Estrategia: crear `approval.category` por departamento, con sus approvers.

Setup:
- Activar `manager_approval='approver'` (no required) en todas.
- Mantener un officer fijo en `approver_ids` por departamento.
- Asegurar coherencia via `hr.employee.parent_id` = manager por department.

UI: los users ven solo categorías con `company_id` matching.
Pueden filtrar dashboard kanban (`approvals/views/approval_category_views.xml:128`, `approval_category_view_kanban`) por departamento (custom search filter).

---

## 11. Aprobaciones cancelables tras aprobar

**Objetivo**: tras `approved`, owner aún puede cancelar antes de pasar a producción.

Default: botón "Cancel" en form solo invisible si `request_owner_id != uid` (l.121 v18) o estado `new/cancel`.

Test: el owner puede cancelar incluso en `approved`. Pero los officers pueden cancelar siempre vía record rules.

Para customizar: extend con state custom `'closed'` post `approved` y bloquear cancel.

---

## 12. v19 — Properties para metadatos custom

v19 añade `approval_properties` (`fields.Properties`) en `approval.request`, con definition en `category.approval_properties_definition`.

Caso de uso: categorías que necesitan campos custom (e.g., "Tipo de viaje: nacional/internacional", "Centro de coste") SIN crear módulo.

Configuración (v19 only):
- En `approval.category` form → tab Properties → definir campos (Text, Selection, Many2one, etc.).
- En el request, los campos aparecen automáticamente como sub-fields del JSON Properties.

Búsqueda: usar `widget="properties"` en search view para filter dinámico.

---

## 13. v19 — Force Approval (server action)

Caso de uso: officer destraba un request donde un approver está indisponible.

Bind: server action `model_approval_request_approve_admin` en form/list para users con `group_approval_user`.

Efecto: `_action_force_approval` setea TODOS los approvers a `approved`, ignora `approver_sequence` y `required`. request_status → `approved` inmediato.

**No hay undo**. Quedará traza en chatter como mensaje de officer.
