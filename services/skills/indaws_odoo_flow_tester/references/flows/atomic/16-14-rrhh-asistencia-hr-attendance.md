## 14. RRHH — Asistencia (hr.attendance)

### ATTENDANCE_CHECK_IN — Fichar Entrada
**Modelo:** hr.attendance
**Descripción:** Registra la entrada (check-in) de un empleado.
**Campos mínimos:**
```json
{
  "employee_id": "<int>",
  "check_in": "YYYY-MM-DD HH:MM:SS"
}
```
**Notas:**
- `check_in`: fecha/hora de entrada (default: ahora)
- `employee_id`: default el empleado del usuario actual
- No se indica `check_out` para simular que el empleado está "fichado"
- Constraint: no puede haber solapamiento con otros registros del mismo empleado
- `in_mode`: "kiosk", "systray", "manual" o "technical" (default: "manual")
**Método/acción:** `create_record.sh hr.attendance '...'`
**Precondiciones:** Empleado sin fichaje abierto (sin check_out pendiente)
**Resultado:** Registro creado con `check_out` = null (fichaje abierto)
**Verificación:** `check_in != False`, `check_out == False`
**Estrategia de reversión:** `delete_record.sh hr.attendance '[<id>]'`
**Registros cascada generados:** Ninguno

### ATTENDANCE_CHECK_OUT — Fichar Salida
**Modelo:** hr.attendance
**Descripción:** Registra la salida (check-out) del fichaje abierto de un empleado.
**Método:** `update_record.sh hr.attendance '[<id>]' '{"check_out": "YYYY-MM-DD HH:MM:SS"}'`
**Precondiciones:** Registro existente con `check_out == False`
**Restricciones:**
- `check_out` debe ser posterior a `check_in` (ValidationError)
- No puede solaparse con otros registros del mismo empleado
**Resultado:** `check_out` asignado, `worked_hours` calculado automáticamente
**Verificación:** `check_out != False`, `worked_hours > 0`
**Estrategia de reversión:** `update_record.sh hr.attendance '[<id>]' '{"check_out": false}'` o `delete_record.sh hr.attendance '[<id>]'`
**Registros cascada generados:** Puede generar/actualizar hr.attendance.overtime (si módulo hr_attendance activo)

---

