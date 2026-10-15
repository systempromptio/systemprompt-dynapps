## 8. helpdesk.ticket (Enterprise)

### Estados
No tiene campo `state` tipo Selection. Usa `stage_id` (Many2one a `helpdesk.stage`, ondelete='restrict'). Stages son por equipo (`team_ids`). Tiene:
```python
kanban_state = fields.Selection([
    ('normal', 'In progress'),
    ('done', 'Ready'),
    ('blocked', 'Blocked')
], default='normal')
```

### Campos Minimos para create()
| Campo | Tipo | Requerido | Default | Notas |
|-------|------|-----------|---------|-------|
| name | Char | Si | - | Asunto del ticket |
| kanban_state | Selection | Si | 'normal' | Estado kanban |

**Nota:** team_id no es required en el modelo, pero tiene default. stage_id y user_id son computed desde team_id.

### Flujos Atomicos

#### HELPDESK_TICKET_CREATE
- **Modelo:** helpdesk.ticket
- **Accion:** create
- **Campos minimos:** `{'name': 'Issue description'}`
- **Resultado:** ticket_ref auto-generado, stage_id y user_id computados desde team_id
- **Verificacion:** id > 0, ticket_ref set

#### HELPDESK_TICKET_MOVE_STAGE
- **Modelo:** helpdesk.ticket
- **Accion:** write({'stage_id': <id>})
- **Resultado:** date_last_stage_update actualizado, SLA status recalculado
- **Verificacion:** stage_id == nuevo stage

#### HELPDESK_TICKET_CLOSE
- **Modelo:** helpdesk.ticket
- **Accion:** write({'stage_id': <closed_stage_id>}) donde stage.fold=True (o is_close=True en algunas versiones)
- **Resultado:** close_date set
- **Verificacion:** close_date != False

#### HELPDESK_TICKET_DELETE
- **Modelo:** helpdesk.ticket
- **Accion:** unlink
- **Restricciones:** Sin restricciones especiales de unlink.

### Registros Cascade
| Modelo hijo | Campo FK | ondelete |
|------------|----------|----------|
| helpdesk.sla.status | ticket_id | cascade |
| helpdesk.sla.status | sla_id | cascade |

---

