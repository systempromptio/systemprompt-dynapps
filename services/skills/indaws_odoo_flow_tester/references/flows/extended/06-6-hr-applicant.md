## 6. hr.applicant

### Estados
No tiene campo `state` tipo Selection. Usa `stage_id` (Many2one a `hr.recruitment.stage`). Tiene campo computado:
```python
application_status = fields.Selection([
    ('ongoing', 'Ongoing'),
    ('hired', 'Hired'),
    ('refused', 'Refused'),
    ('archived', 'Archived'),
], compute="_compute_application_status")
```
La logica: si refuse_reason_id -> 'refused'; si not active -> 'archived'; si date_closed -> 'hired'; else 'ongoing'.

### Campos Minimos para create()
| Campo | Tipo | Requerido | Default | Notas |
|-------|------|-----------|---------|-------|
| candidate_id | Many2one (hr.candidate) | Si | - | Candidato asociado (obligatorio) |
| kanban_state | Selection | Si | 'normal' | Estado kanban |

**Nota:** `partner_name` es computed desde candidate_id. Para crear un applicant funcional se necesita un `hr.candidate` previo.

### Flujos Atomicos

#### HR_APPLICANT_CREATE
- **Modelo:** hr.applicant
- **Accion:** create
- **Campos minimos:** `{'candidate_id': <id>, 'partner_name': 'John Doe'}`
- **Resultado:** stage_id auto-computado si job_id esta definido
- **Verificacion:** id > 0

#### HR_APPLICANT_HIRE
- **Modelo:** hr.applicant
- **Accion:** write stage con hired_stage=True -> date_closed auto-set
- **O:** create_employee_from_applicant() para crear empleado
- **Resultado:** application_status = 'hired', date_closed set
- **Verificacion:** date_closed != False

#### HR_APPLICANT_REFUSE
- **Modelo:** hr.applicant
- **Accion:** archive_applicant() -> abre wizard `applicant.get.refuse.reason`
- **Resultado:** refuse_reason_id set, application_status = 'refused'
- **Nota via API:** write({'refuse_reason_id': <id>, 'active': False})

#### HR_APPLICANT_RESET
- **Modelo:** hr.applicant
- **Accion:** reset_applicant()
- **Resultado:** stage_id = primer stage, refuse_reason_id = False
- **Verificacion:** refuse_reason_id == False

#### HR_APPLICANT_DELETE
- **Modelo:** hr.applicant
- **Accion:** unlink
- **Restricciones:** Sin restricciones especiales de unlink.

### Registros Cascade
| Modelo hijo | Campo FK | ondelete |
|------------|----------|----------|
| hr.recruitment.source | job_id | cascade |

---

