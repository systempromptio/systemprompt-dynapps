## 13. approval.category (Enterprise)

### Estados
No tiene campo `state`. Modelo de configuracion.

### Campos Minimos para create()
| Campo | Tipo | Requerido | Default | Notas |
|-------|------|-----------|---------|-------|
| name | Char | Si | - | Nombre de la categoria |
| company_id | Many2one (res.company) | Si | env.company | Compania |
| has_date | Selection | Si | 'no' | required/optional/no |
| has_period | Selection | Si | 'no' | required/optional/no |
| has_quantity | Selection | Si | 'no' | required/optional/no |
| has_amount | Selection | Si | 'no' | required/optional/no |
| has_reference | Selection | Si | 'no' | required/optional/no |
| has_partner | Selection | Si | 'no' | required/optional/no |
| has_payment_method | Selection | Si | 'no' | required/optional/no |
| has_location | Selection | Si | 'no' | required/optional/no |
| has_product | Selection | Si | 'no' | required/optional/no |
| requirer_document | Selection | Si | 'optional' | required/optional |
| approval_minimum | Integer | Si | 1 | Minimo de aprobaciones |

**En practica:** `{'name': 'Test Category'}` es suficiente, todos los Selection tienen defaults.

### Flujos Atomicos

#### APPROVAL_CATEGORY_CREATE
- **Modelo:** approval.category
- **Accion:** create
- **Campos minimos:** `{'name': 'My Category'}`
- **Resultado:** registro creado con defaults
- **Verificacion:** id > 0

#### APPROVAL_CATEGORY_DELETE
- **Modelo:** approval.category
- **Accion:** unlink
- **Restricciones:** Sin restricciones especiales. approver_ids se borran en cascade.

### Constraints
```python
# approval_minimum >= count(required approvers)
# approver_ids unique por user
# approver_sequence requiere approval_minimum >= 1
```

---

