<!-- verified: v18.0 @ 2026-06 -->
# Approvals — Core (Odoo Enterprise 18)

Workflow de solicitudes y aprobaciones genérico (compras, viajes, contratos, préstamos, etc.).
3 módulos: `approvals` (base), `approvals_purchase` (genera RFQ), `approvals_purchase_stock` (warehouse).

Manifest: `odoo_enterprise/18/approvals/__manifest__.py` — depends **`['mail', 'hr', 'product']`** (NO purchase, NO web). La dependencia `hr` es clave: `manager_approval` resuelve el jefe vía `hr.employee.parent_id` — sin `hr` no existiría. `application=True`, `license='OEEL-1'`.

---

## Modelos clave

### `approval.category` (template)
`approvals/models/approval_category.py → ApprovalCategory`

| Campo | Tipo | Notas |
|---|---|---|
| `name`, `description`, `image`, `sequence` | — | Translatable |
| `approval_type` | Selection (vacío base) | **Extendido por bridges**: `'purchase'` lo añade approvals_purchase. Vacío = categoría libre |
| `has_date`, `has_period`, `has_quantity`, `has_amount`, `has_reference`, `has_partner`, `has_payment_method`, `has_location`, `has_product` | Selection `['required','optional','no']` | Controlan qué campos activa cada request |
| `requirer_document` | Selection `required/optional` | Si `required`, no permite submit sin attachment |
| `approval_minimum` | Integer (default=1) | Nº mínimo aprobadores para llegar a `approved` |
| `manager_approval` | Selection `False/approver/required` | Inyecta al `parent_id.user_id` del employee del owner. `'required'` lo hace approver obligatorio |
| `approver_ids` | One2many `approval.category.approver` | Plantilla de aprobadores |
| `approver_sequence` | Boolean | Si True, aprobadores en SECUENCIA (uno tras otro). Constraint: solo si `approval_minimum >= 1` |
| `automated_sequence` + `sequence_code` + `sequence_id` | — | Auto-numeración `APPR/2025/0001` |
| `user_ids` | Many2many computed | Agregación de approver_ids.user_id |

**Constraints:**
- `_constrains_approval_minimum` (l.94-98): `approval_minimum >= count(approvers.required=True)`.
- `_constrains_approver_ids` (l.100-108): no users duplicados en `approver_ids`.
- `_constrains_approver_sequence` (l.110-113): `approver_sequence` requiere `approval_minimum >= 1`.

### `approval.category.approver` (template approver)
`approvals/models/approval_category_approver.py:6-28`, `ApprovalCategoryApprover`

- `category_id`, `user_id`, `sequence` (default=10), `required` (Boolean).
- `required=True` → ese approver DEBE aprobar para llegar a `approved` (override de approval_minimum).

### `approval.request` (instance)
`approvals/models/approval_request.py:9-362`, `ApprovalRequest`

| Campo | Tipo | Notas |
|---|---|---|
| `name` | Char tracking | Auto-secuenciado si `automated_sequence` |
| `category_id` | M2O required | Cambiarlo borra+recrea TODOS approver_ids |
| `request_owner_id` | M2O res.users | Domain `_get_request_owner_id_domain` (l.16-17): `share=False` **AND** `company_ids in env.companies.ids` (companies activas, no todas las allowed) |
| `approver_ids` | O2M `approval.approver`, **computed+store+readonly=False** | Editable manualmente tras recompute |
| `request_status` | Selection computed+stored tracking, group_expand | `new/pending/approved/refused/cancel` |
| `user_status` | Selection computed | Estado **del usuario actual** como approver de ESTE request. `''` si no es approver |
| `has_access_to_request` | Boolean computed | True SOLO si `owner==user AND user in group_approval_user` |
| `change_request_owner` | Boolean computed | True si `user in group_approval_user` |
| `date_confirmed` | Datetime | Se setea en `action_confirm()` con .sudo() |
| `attachment_ids`, `attachment_number` | — | Domain: `res_model='approval.request' AND res_field IS NULL` |
| `product_line_ids` | O2M | Solo si `has_product != 'no'` |
| Campos has_* (related a category) | readonly | `has_date/period/quantity/amount/...` |
| `amount`, `reference`, `partner_id`, `payment_method`, `location`, `quantity`, `date`, `date_start`, `date_end`, `reason` | Mixto | Visibles según `has_*` |

### `approval.approver` (instance approver)
`approvals/models/approval_request.py:364-432`, `ApprovalApprover`

| Campo | Tipo | Notas |
|---|---|---|
| `request_id`, `user_id`, `sequence` (default=10) | — | sequence crítico si `approver_sequence` |
| `status` | Selection readonly | `new/pending/waiting/approved/refused/cancel` |
| `required` | Boolean readonly | Copiado de `category.approver_ids.required` |
| `category_approver` | Boolean computed | True si está en plantilla |
| `can_edit`, `can_edit_user_id` | Boolean computed | Quién puede editarlo |
| `existing_request_user_ids` | M2M computed | Excluidos del domain de user_id |

**`copy()` lanza UserError** (l.431-432): no se duplican approvers.

### `approval.product.line`
`approvals/models/approval_product_line.py:8-37`, `class ApprovalProductLine`

- `product_id`, `description`, `quantity`, `product_uom_id`, `product_uom_category_id` (related).
- `description` y `product_uom_id` son **computed con `precompute=True`** → se calculan ANTES de save inicial.

---

## Ciclo de vida (request_status)

```
new ──action_confirm()──▶ pending ──┬─action_approve()─▶ approved
                                    ├─action_refuse()──▶ refused
                                    └─action_cancel()──▶ cancel
                                                            │
                            (acción_draft desde cancel)◀────┘
```

**Sub-estados de `approval.approver.status`:**
- `new` (inicial), `pending` (turno actual), `waiting` (espera turno si sequence), `approved`, `refused`, `cancel`.

### `action_confirm()` (l.136-163)
1. Si `category.manager_approval == 'required'` → verifica `hr.employee(user=owner).parent_id.user_id` existe en `approver_ids`. Si no, **UserError**.
2. `len(approver_ids) >= approval_minimum` o UserError.
3. Si `requirer_document='required'` → `attachment_number >= 1` o UserError.
4. **Secuenciación**:
   - `approver_sequence=True`: `approvers[1:].sudo().write({'status':'waiting'})` + primer approver pasa a `pending`.
   - `approver_sequence=False`: TODOS pasan a `pending`.
5. `_create_activity()` agenda mail.activity tipo `approvals.mail_activity_data_approval` para cada approver `pending`.
6. `self.sudo().write({'date_confirmed': fields.Datetime.now()})`.

### `_compute_request_status()` (l.273-293)
Orden de evaluación (prioridad descendente):
1. `[]` → `'new'`
2. `'cancel' in status_lst` → `'cancel'` (anula todo)
3. `'refused' in status_lst` → `'refused'` (un solo refused hunde el request)
4. `'new' in status_lst` → `'new'`
5. `count('approved') >= minimal_approver AND required_approved` → `'approved'`
6. Resto → `'pending'`

Donde:
- `required_approved = all(a.status=='approved' for a in approver_ids.filtered('required'))` — si UN required no aprueba, NUNCA llega a `approved` aunque approval_minimum se cumpla.
- `minimal_approver = min(approval_minimum, len(status_lst))` — tolerancia silenciosa si configuración inconsistente.

Cuando `request_status` cambia a estado terminal (`approved/refused/cancel`), se ejecuta `_cancel_activities()` automáticamente (borra todas las activities pendientes).

### `action_approve()` (l.165-220, simplificado)
- Si `approver_sequence=True` y aún hay approvers en `'waiting'` previos → **UserError** "no es tu turno".
- Tu `status` → `'approved'`.
- Si `approver_sequence=True`: siguiente approver pasa de `waiting` → `pending` + se crea activity.
- Activity actual se cierra con `action_feedback()` (queda traza en chatter).

### `action_refuse()` (l.222-249)
- Si `approver_sequence=True`: tu status → `refused`, y TODOS los approvers posteriores → `refused` también.
- Si NO sequence: solo TÚ → `refused`.
- Pero como `_compute_request_status` da prioridad a `refused` SOBRE pending → request entero → `refused`.

### `action_withdraw()` (l.251-257)
- Retracta tu aprobación: tu status (`approved/refused`) → `pending`.
- Si `approver_sequence`: approvers posteriores vuelven a `waiting`.
- request vuelve a `pending`.

### `action_cancel()` (l.259-265)
- TODOS approver_ids.status → `'cancel'`.
- Activities → `.unlink()` (NO `action_feedback`, no queda traza).
- Solo `request_owner_id` o `group_approval_user` (vía record rule) puede.

### `action_draft()` (l.267-271)
- Solo desde `'cancel'`.
- TODOS approver_ids.status → `'new'`. request_status → `'new'`.

---

## `_compute_approver_ids()` (l.295-327) — recreación

Dispara con `category_id` o `request_owner_id` cambia.

Algoritmo:
1. `Command.clear()` → BORRA TODOS los approver_ids actuales.
2. Si `category.manager_approval != False`:
   - Busca `hr.employee(user_id=owner, company_id=self.company_id, limit=1)`.
   - Si tiene `parent_id.user_id` → añade approver con `sequence=9` (primero en línea siempre), `required=(manager_approval=='required')`.
3. Por cada `user_id` en `category.approver_ids`: añade approver con `sequence`, `required` de la plantilla.
4. **Anti-duplicación silenciosa** (l.316-317): si manager también está en `category.approver_ids`, se pop() del dict para evitar duplicación.

**Impacto:** approvers añadidos manualmente **se pierden** al cambiar category. Edita post-recompute.

---

## Manager sequence=9 — siempre primero

El manager inyectado por `manager_approval` toma `sequence=9` (hardcoded l.309-315), default de los demás es `10`. Si `approver_sequence=True`, el manager SIEMPRE aprueba primero.

Para que un approver lo anteceda hay que setear `sequence < 9` manualmente tras el recompute.

---

## Categorías de demo

`data/approval_category_data.xml` define 8: Business Trip, Borrow Items, General Approval, Contract, Payment, Car Rental, Job Referral Award, Procurement. Todas con `manager_approval='approver'` y combinaciones de `has_*`.

`approvals_purchase` añade una más: "Create RFQ's" con `approval_type='purchase'`, `automated_sequence=True`, `sequence_code='APPR'`, `has_product='required'`, `has_quantity='required'`.

---

## Seguridad

`security/approval_security.xml` + `security/ir.model.access.csv`.

**Grupos** (`module_category_human_resources_approvals`):
- `group_approval_user` ("Officer: Approve all requests", l.7-12) — implica `base.group_user`. Aprueba/rechaza CUALQUIER request (record rule `[(1,'=',1)]`), pero NO accede a configuración de categorías.
- `group_approval_manager` ("Administrator", l.14-19) — implica `group_approval_user`. Único con CRUD de `approval.category`.

**Record rules clave (`base.group_user`, l.27-90):**
- *Request approver rule* (l.27-41): read/write si `owner==user` OR `user in approver_ids` OR (`category.manager_approval in ('approver','required')` AND `owner.parent_id.user_id==user`). El jefe del owner tiene acceso aunque no esté en approver_ids (ver gotcha APPG28).
- *Unlink owner rule* (l.59-68): unlink solo si owner Y `request_status in ('cancel','new')`. **OJO**: el `<function name="write">` final (l.205-208) reescribe este dominio en instalación.
- *Approver read/change own* (l.70-90): el usuario lee requests donde es owner/approver y puede escribir su propia línea approver.

**Officer (`group_approval_user`, l.93-157):** acceso total read/write/create; unlink de request solo en `cancel`; unlink de approver solo en `new/pending/cancel` (NO terminal → quitar approver pendiente puede auto-aprobar, ver APPG30).

**Multi-company (l.183-202):** rules `company_id in company_ids` en request, category, approver, product.line. Todos los modelos tienen `_check_company_auto = True`.

`_mail_post_access = 'read'` (request l.12): postear en chatter requiere solo lectura.

---

## v19 — cambios principales

Ver `v19_changes.md`. Highlights:
- Modelo `approval.approver` extraído a fichero propio (`approval_approver.py`).
- Constraint `_check_approver_ids` migrado a `models.Constraint('UNIQUE')` (nueva API v19).
- `_ensure_can_approve()` eliminado, lógica inline.
- Campo `active` + ribbon "Archived" en form.
- Campo `approval_properties` (Properties) en request, `approval_properties_definition` en category.
- Nuevo `_action_force_approval()` (server action solo officers).
- Validación en `write()`: no admin no puede aprobar/rechazar su propio request.
- Status string `'pending'`: "To Approve" → "Submitted".
- Security: `category_id` de grupos → `privilege_id` (nuevo modelo `res.groups.privilege`).
