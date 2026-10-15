## 7. hr.attendance

### Estados
No tiene campo `state`. Modelo simple de check-in/check-out. Tiene:
```python
overtime_status = fields.Selection([
    ('to_approve', 'To Approve'),
    ('approved', 'Approved'),
    ('refused', 'Refused')
], compute/store/readonly=False)
```

### Campos Minimos para create()
| Campo | Tipo | Requerido | Default | Notas |
|-------|------|-----------|---------|-------|
| employee_id | Many2one (hr.employee) | Si | env.user.employee_id | Empleado, ondelete='cascade' |
| check_in | Datetime | Si | Datetime.now | Hora de entrada |

### Flujos Atomicos

#### HR_ATTENDANCE_CREATE (Check-In)
- **Modelo:** hr.attendance
- **Accion:** create
- **Campos minimos:** `{'employee_id': <id>}` (check_in tiene default)
- **Resultado:** registro con check_out=False
- **Verificacion:** id > 0, check_out == False

#### HR_ATTENDANCE_CHECKOUT
- **Modelo:** hr.attendance
- **Accion:** write({'check_out': <datetime>})
- **Resultado:** worked_hours computado automaticamente
- **Verificacion:** check_out != False, worked_hours > 0

#### HR_ATTENDANCE_APPROVE_OVERTIME
- **Modelo:** hr.attendance
- **Accion:** action_approve_overtime()
- **Resultado:** overtime_status = 'approved'

#### HR_ATTENDANCE_REFUSE_OVERTIME
- **Modelo:** hr.attendance
- **Accion:** action_refuse_overtime()
- **Resultado:** overtime_status = 'refused'

#### HR_ATTENDANCE_DELETE
- **Modelo:** hr.attendance
- **Accion:** unlink
- **Restricciones:** Sin restricciones de unlink. Se recalcula overtime asociado.
- **Nota:** copy() lanza UserError: "You cannot duplicate an attendance."

### Registros Cascade
| Modelo hijo | Campo FK | ondelete |
|------------|----------|----------|
| hr.attendance | employee_id | cascade (si se borra empleado, se borran attendances) |
| hr.attendance.overtime | employee_id | cascade |

### Constraints importantes
```python
_sql_constraints = [
    ('check_in_check_out', ...),  # check_out > check_in
]
# Constraint Python: no solapamiento de attendances del mismo empleado
```

---

