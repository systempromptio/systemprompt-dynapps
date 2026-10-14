## 11. planning.slot (Enterprise)

### Estados
```python
state = fields.Selection([
    ('draft', 'Draft'),
    ('published', 'Published'),
], default='draft')
```

### Campos Minimos para create()
| Campo | Tipo | Requerido | Default | Notas |
|-------|------|-----------|---------|-------|
| start_datetime | Datetime | Si | _default_start_datetime (hoy 00:00) | Inicio del turno |
| end_datetime | Datetime | Si | _default_end_datetime (hoy 23:59) | Fin del turno |
| company_id | Many2one (res.company) | Si | computed from resource o env.company | Compania |
| repeat_unit | Selection | Si | 'week' | Unidad de repeticion |

**Nota:** `resource_id` es opcional (open shift si no se asigna).

### Flujos Atomicos

#### PLANNING_SLOT_CREATE
- **Modelo:** planning.slot
- **Accion:** create
- **Campos minimos:** `{'start_datetime': <dt>, 'end_datetime': <dt>}`
- **Resultado:** state='draft', allocated_hours computado
- **Verificacion:** id > 0, state == 'draft'

#### PLANNING_SLOT_PUBLISH
- **Modelo:** planning.slot
- **Accion:** action_planning_publish_and_send()
- **O:** action_send() (individual)
- **Resultado:** state='published', notificacion enviada al empleado
- **Verificacion:** state == 'published'

#### PLANNING_SLOT_UNPUBLISH
- **Modelo:** planning.slot
- **Accion:** action_unpublish()
- **Precondicion:** usuario con grupo planning.group_planning_manager
- **Resultado:** state='draft', publication_warning=False
- **Verificacion:** state == 'draft'

#### PLANNING_SLOT_SELF_ASSIGN
- **Modelo:** planning.slot
- **Accion:** action_self_assign()
- **Precondicion:** resource_id vacido (open shift), no is_past
- **Resultado:** resource_id = current user's employee resource
- **Restriccion:** No se puede asignar turno pasado ni ya asignado

#### PLANNING_SLOT_SELF_UNASSIGN
- **Modelo:** planning.slot
- **Accion:** action_self_unassign()
- **Precondicion:** allow_self_unassign=True, deadline no pasado
- **Resultado:** resource_id = False

#### PLANNING_SLOT_DELETE
- **Modelo:** planning.slot
- **Accion:** unlink (o action_unlink())
- **Restricciones:** Sin restricciones especiales. action_unlink() simplemente llama unlink() y retorna close action.

### Registros Cascade
| Modelo hijo | Campo FK | ondelete |
|------------|----------|----------|
| (recurrency_id con ondelete="set null") | - | - |

### SQL Constraints
```sql
CHECK(end_datetime > start_datetime)  -- End debe ser posterior a start
CHECK(allocated_hours >= 0)           -- Horas no negativas
```

---

