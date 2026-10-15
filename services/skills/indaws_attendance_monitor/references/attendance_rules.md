# Reglas de fichaje — matriz de incidencias

## Tipo de jornada por día

| Condición | Tipo |
|---|---|
| `weekday == 4` (viernes) | Intensiva |
| `month in (7, 8)` (julio o agosto) | Intensiva |
| Resto de L–J de Sept–Junio | Partida |
| Sábado, domingo | No laborable (no se evalúa) |

---

## Excepciones (días sin obligación de fichar)

Se aplican en este orden. Si cualquiera coincide → 🟢 día OK con motivo.

1. **Festivo del calendario laboral.** Existe un registro `resource.calendar.leaves` que cumple:
   - `calendar_id == empleado.resource_calendar_id`
   - `resource_id` vacío (festivo global, no de un recurso concreto)
   - `date_from <= día 23:59:59` y `date_to >= día 00:00:00`
2. **Vacaciones / ausencia validada.** Existe `hr.leave` con:
   - `employee_id == empleado.id`
   - `state == 'validate'`
   - `date_from <= día 23:59:59` y `date_to >= día 00:00:00`

Si el día queda cubierto por una excepción, **no se exige fichaje** y no se evalúan registros.

---

## Reglas por tipo de jornada

### Jornada partida (L–J fuera de julio/agosto)

| Estado | Condición |
|---|---|
| 🔴 Sin fichaje | 0 registros `hr.attendance` ese día |
| 🔴 Falta segundo fichaje | 1 registro (jornada partida exige 2 sesiones) |
| 🟢 OK | 2 registros, suma horas ∈ [7h, 9h], pausa ∈ [30 min, 1h30] |
| 🟡 Pausa insuficiente | 2 registros, pausa < 30 min |
| 🟡 Pausa excesiva | 2 registros, pausa > 1h30 |
| 🟡 Más de 2 fichajes | 3+ registros (revisar manualmente) |
| 🔴 Pocas horas | Suma horas < 7h |
| 🔴 Demasiadas horas | Suma horas > 9h |
| 🟡 Sesión abierta | Algún registro tiene `check_out` vacío |

**Pausa de comida** = `check_in[1] - check_out[0]` (hora de entrada del segundo registro menos hora de salida del primero).

### Jornada intensiva (viernes y julio/agosto)

| Estado | Condición |
|---|---|
| 🔴 Sin fichaje | 0 registros |
| 🟢 OK | ≥1 registro, suma horas ∈ [6h, 8h] |
| 🔴 Pocas horas | Suma horas < 6h |
| 🔴 Demasiadas horas | Suma horas > 8h |
| 🟡 Más de 1 fichaje | 2+ registros (esperado 1 sesión continua, revisar) |
| 🟡 Sesión abierta | Algún registro tiene `check_out` vacío |

---

## Severidades y orden

Cuando un día acumula varias condiciones, prevalece la más severa: **🔴 > 🟡 > 🟢**.

Ejemplo: jornada partida con 2 registros y pausa correcta pero suma de horas = 6h30 → 🔴 Pocas horas (no 🟢).

Si hay varias condiciones del mismo nivel, se concatenan en la columna `Detalle` separadas por `; `.

---

## Configuración (constantes en `analyze_attendance.py`)

```python
TARGET_NORMAL_HOURS = 8.0
TARGET_INTENSIVE_HOURS = 7.0
TOLERANCE_HOURS = 1.0           # ±1 h sobre el objetivo
LUNCH_MIN_MINUTES = 30
LUNCH_MAX_MINUTES = 90          # 1h30
SUMMER_MONTHS = (7, 8)
INTENSIVE_WEEKDAY = 4           # Viernes (lunes=0)
WORKING_WEEKDAYS = (0, 1, 2, 3, 4)   # L–V
```

Si el negocio ajusta tolerancias (p. ej. ±45 min en lugar de ±1h), modificar estas constantes.
