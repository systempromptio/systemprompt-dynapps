## 10. quality.alert (Enterprise)

### Estados
No tiene campo `state` tipo Selection. Usa `stage_id` (Many2one a `quality.alert.stage`, ondelete='restrict'). Stages tienen campo `done` (Boolean).

### Campos Minimos para create()
| Campo | Tipo | Requerido | Default | Notas |
|-------|------|-----------|---------|-------|
| team_id | Many2one (quality.alert.team) | Si | _get_default_team_id | Equipo de calidad |
| company_id | Many2one (res.company) | Si | env.company | Compania |

**Nota:** `name` se auto-genera por secuencia. `stage_id` tiene default.

### Flujos Atomicos

#### QUALITY_ALERT_CREATE
- **Modelo:** quality.alert
- **Accion:** create
- **Campos minimos:** `{'team_id': <id>}` o `{}` si default funciona
- **Resultado:** name auto-secuenciado, stage_id = primer stage del equipo
- **Verificacion:** id > 0, name != 'New'

#### QUALITY_ALERT_CLOSE
- **Modelo:** quality.alert
- **Accion:** write({'stage_id': <done_stage_id>})
- **Resultado:** date_close = now (auto-set en write si stage.done=True)
- **Verificacion:** date_close != False

#### QUALITY_ALERT_DELETE
- **Modelo:** quality.alert
- **Accion:** unlink
- **Restricciones:** Sin restricciones especiales.

### Registros Cascade
| Modelo hijo | Campo FK | ondelete |
|------------|----------|----------|
| (ninguno con cascade) | - | - |

---

