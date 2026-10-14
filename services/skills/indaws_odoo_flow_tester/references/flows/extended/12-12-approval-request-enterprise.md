## 12. approval.request (Enterprise)

### Estados
```python
request_status = fields.Selection([
    ('new', 'To Submit'),
    ('pending', 'Submitted'),
    ('approved', 'Approved'),
    ('refused', 'Refused'),
    ('cancel', 'Canceled'),
], default="new", compute="_compute_request_status", store=True)
```
**Importante:** `request_status` es COMPUTED desde los estados de los approvers. No se escribe directamente.

### Campos Minimos para create()
| Campo | Tipo | Requerido | Default | Notas |
|-------|------|-----------|---------|-------|
| category_id | Many2one (approval.category) | Si | - | Categoria de aprobacion |

**Nota:** `name` puede ser auto-generado si la categoria tiene `automated_sequence=True`. `request_owner_id` default = env.user.

### Flujos Atomicos

#### APPROVAL_REQUEST_CREATE
- **Modelo:** approval.request
- **Accion:** create
- **Campos minimos:** `{'category_id': <id>}`
- **Resultado:** request_status='new', approver_ids auto-computados desde category
- **Verificacion:** id > 0, request_status == 'new'

#### APPROVAL_REQUEST_CONFIRM
- **Modelo:** approval.request
- **Accion:** action_confirm()
- **Precondicion:** approver_ids >= approval_minimum, documentos si requirer_document='required'
- **Resultado:** approvers pasan a status 'pending' (o 'waiting' si sequence), date_confirmed set
- **Restriccion:** Si manager_approval='required', manager debe estar en approvers
- **Verificacion:** request_status == 'pending'

#### APPROVAL_REQUEST_APPROVE
- **Modelo:** approval.request
- **Accion:** action_approve()
- **Precondicion:** current user es approver con status 'pending' (no 'waiting' si sequence)
- **Resultado:** approver status='approved', si todos aprobados -> request_status='approved'
- **Restriccion:** Con approver_sequence, no se puede aprobar antes del anterior
- **Verificacion:** request_status en ('pending', 'approved')

#### APPROVAL_REQUEST_REFUSE
- **Modelo:** approval.request
- **Accion:** action_refuse()
- **Resultado:** approver status='refused', request_status='refused'
- **Verificacion:** request_status == 'refused'

#### APPROVAL_REQUEST_CANCEL
- **Modelo:** approval.request
- **Accion:** action_cancel()
- **Resultado:** todos approvers status='cancel', request_status='cancel'
- **Verificacion:** request_status == 'cancel'

#### APPROVAL_REQUEST_DRAFT
- **Modelo:** approval.request
- **Accion:** action_draft()
- **Resultado:** todos approvers status='new', request_status='new'
- **Verificacion:** request_status == 'new'

#### APPROVAL_REQUEST_WITHDRAW
- **Modelo:** approval.request
- **Accion:** action_withdraw()
- **Resultado:** approver vuelve a 'pending', siguientes a 'waiting'

#### APPROVAL_REQUEST_DELETE
- **Modelo:** approval.request
- **Accion:** unlink
- **Restricciones:** `@api.ondelete` limpia attachments. `unlink()` borra product_line_ids si has_product. No hay restriccion por estado.

### Registros Cascade
| Modelo hijo | Campo FK | ondelete |
|------------|----------|----------|
| approval.approver | request_id | cascade |
| approval.category.approver | category_id | cascade |
| approval.category.approver | user_id | cascade |

---

