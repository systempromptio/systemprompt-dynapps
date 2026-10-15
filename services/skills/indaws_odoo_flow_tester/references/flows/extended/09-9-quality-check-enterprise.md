## 9. quality.check (Enterprise)

### Estados
```python
quality_state = fields.Selection([
    ('none', 'To do'),
    ('pass', 'Passed'),
    ('fail', 'Failed')
], default='none')
```

### Campos Minimos para create()
| Campo | Tipo | Requerido | Default | Notas |
|-------|------|-----------|---------|-------|
| team_id | Many2one (quality.alert.team) | Si | - | Equipo de calidad |
| test_type_id | Many2one (quality.point.test_type) | Si | - | Tipo de test (passfail, measure, etc.) |
| company_id | Many2one (res.company) | Si | env.company | Compania |

**Nota:** `name` se auto-genera por secuencia si no se provee. `point_id` es opcional pero si se da, test_type_id y note se derivan de el.

### Flujos Atomicos

#### QUALITY_CHECK_CREATE
- **Modelo:** quality.check
- **Accion:** create
- **Campos minimos:** `{'team_id': <id>, 'test_type_id': <id>}`
- **Resultado:** name auto-secuenciado, quality_state='none'
- **Verificacion:** id > 0, quality_state == 'none'

#### QUALITY_CHECK_PASS
- **Modelo:** quality.check
- **Accion:** do_pass()
- **Resultado:** quality_state='pass', user_id=current_user, control_date=now
- **Verificacion:** quality_state == 'pass'

#### QUALITY_CHECK_FAIL
- **Modelo:** quality.check
- **Accion:** do_fail()
- **Resultado:** quality_state='fail', user_id=current_user, control_date=now
- **Verificacion:** quality_state == 'fail'

#### QUALITY_CHECK_MEASURE (quality_control)
- **Modelo:** quality.check
- **Accion:** do_measure()
- **Precondicion:** point_id con test_type='measure', tolerance_min/max definidos
- **Resultado:** do_pass() si measure entre tolerancias, do_fail() si no

#### QUALITY_CHECK_DELETE
- **Modelo:** quality.check
- **Accion:** unlink
- **Restricciones:** Borra spreadsheet_id asociado si existe (override en quality_control).

### Registros Cascade
| Modelo hijo | Campo FK | ondelete |
|------------|----------|----------|
| (ninguno con cascade) | - | - |

---

