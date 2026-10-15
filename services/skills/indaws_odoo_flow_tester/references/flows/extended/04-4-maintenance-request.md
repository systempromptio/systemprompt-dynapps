## 4. maintenance.request

### Estados
No tiene campo `state` tipo Selection. Usa `stage_id` (Many2one a `maintenance.stage`) con grupo expand. Stages predeterminados del modulo con campo `done` (Boolean) para indicar completado. Tambien tiene:
```python
kanban_state = fields.Selection([
    ('normal', 'In Progress'),
    ('blocked', 'Blocked'),
    ('done', 'Ready for next stage')
], default='normal')
```

### Campos Minimos para create()
| Campo | Tipo | Requerido | Default | Notas |
|-------|------|-----------|---------|-------|
| name | Char | Si | - | Asunto del request |
| company_id | Many2one (res.company) | Si | env.company | Compania |
| maintenance_team_id | Many2one (maintenance.team) | Si | _get_default_team_id | Equipo de mantenimiento |
| kanban_state | Selection | Si | 'normal' | Estado kanban |

### Flujos Atomicos

#### MAINT_REQUEST_CREATE
- **Modelo:** maintenance.request
- **Accion:** create
- **Campos minimos:** `{'name': 'Fix Machine'}`
- **Resultado:** registro con stage_id = primer stage, kanban_state='normal'
- **Verificacion:** id > 0, stage_id set

#### MAINT_REQUEST_CLOSE
- **Modelo:** maintenance.request
- **Accion:** write({'stage_id': <done_stage_id>})
- **Resultado:** close_date se establece automaticamente, si es preventive + recurring se crea copia con nueva fecha
- **Verificacion:** close_date != False, stage_id.done == True

#### MAINT_REQUEST_ARCHIVE
- **Modelo:** maintenance.request
- **Accion:** archive_equipment_request()
- **Resultado:** archive=True, recurring_maintenance=False

#### MAINT_REQUEST_RESET
- **Modelo:** maintenance.request
- **Accion:** reset_equipment_request()
- **Resultado:** archive=False, stage_id = primer stage

#### MAINT_REQUEST_DELETE
- **Modelo:** maintenance.request
- **Accion:** unlink
- **Restricciones:** Sin restricciones especiales. El campo equipment_id tiene `ondelete='restrict'`, pero eso protege al equipment, no al request.

### Registros Cascade
| Modelo hijo | Campo FK | ondelete |
|------------|----------|----------|
| (ninguno) | - | - |

---

