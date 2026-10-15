# Attendance Monitor — Workflow Detail

## Reglas de negocio

| Día | Periodo | Tipo jornada | Registros esperados | Horas objetivo | Margen |
|---|---|---|---|---|---|
| L–J | Sept–Junio | Partida | 2 | 8h | ±1h |
| V | Todo el año | Intensiva | ≥1 | 7h | ±1h |
| L–V | Julio, Agosto | Intensiva | ≥1 | 7h | ±1h |

**Pausa de comida (jornada partida):** entre el `check_out` del primer registro y el `check_in` del segundo debe haber **entre 30 min y 1h30**.

**Excepciones (no se exige fichaje):**
- Día cubierto por `hr.leave` validado del empleado (vacaciones, baja, etc.)
- Día marcado como festivo en `resource.calendar.leaves` del calendario del empleado

## Paso 0: Recopilación de Datos

Antes de hacer cualquier análisis, **SIEMPRE** recopila:

1. **Credenciales Odoo** (si no están como variables de entorno):
   - `ODOO_URL`, `ODOO_DB`, `ODOO_KEY`, `ODOO_USER`

2. **Semana a analizar** (opcional): Si no se indica, usa la **semana pasada completa** (lunes a domingo de la semana ISO anterior). Acepta también:
   - `--week-start YYYY-MM-DD` (lunes de la semana objetivo)
   - `--week-end YYYY-MM-DD` (domingo de la semana objetivo)

**DETENTE** y espera confirmación si la semana por defecto no es lo que quiere analizar.

## Paso 1: Extracción de Datos de Odoo

```bash
export ODOO_URL="<url>"
export ODOO_DB="<db>"
export ODOO_KEY="<api_key>"
export ODOO_USER="<usuario>"

# Semana pasada por defecto
./scripts/fetch_attendance_data.sh > /tmp/att_data.json

# Semana concreta
./scripts/fetch_attendance_data.sh --week-start 2026-04-20 --week-end 2026-04-26 > /tmp/att_data.json
```

**El script obtiene** (5 conjuntos de datos):

1. **Asistencias** (`hr.attendance`) — todos los `check_in` dentro del rango
2. **Empleados** (`hr.employee`) — solo aquellos con alguna asistencia en la semana
3. **Vacaciones/ausencias** (`hr.leave`) con `state='validate'` solapadas con la semana
4. **Festivos del calendario** (`resource.calendar.leaves`) solapados con la semana
5. **Calendarios** (`resource.calendar`) referenciados por los empleados

**Validaciones post-extracción**:
- ✅ Hay asistencias en el rango → al menos un empleado a analizar
- ❌ Sin asistencias → informar "Ningún empleado fichó esta semana" y terminar

## Paso 2: Análisis

```bash
python3 ./scripts/analyze_attendance.py < /tmp/att_data.json
```

**El script aplica las reglas por empleado y por día laboral (L–V):**

1. Determina el tipo de jornada (intensiva si viernes o julio/agosto, partida en otro caso)
2. Comprueba excepciones en orden:
   - ¿Festivo en `resource.calendar.leaves`? → OK
   - ¿Empleado tiene `hr.leave` validada cubriendo ese día? → OK
3. Si no hay excepción, evalúa los registros del día:
   - **0 registros**: 🔴 "Sin fichaje"
   - **Jornada partida con 1 registro**: 🔴 "Falta segundo fichaje"
   - **Jornada partida con 2 registros**:
     - Pausa < 30 min: 🟡 "Pausa comida insuficiente"
     - Pausa > 1h30: 🟡 "Pausa comida excesiva"
   - **Jornada partida con >2 registros**: 🟡 "Más de 2 fichajes (revisar)"
   - **Suma de horas fuera de [target-1h, target+1h]**: 🔴 "Horas fuera de rango"

## Paso 3: Presentación

Mostrar el informe en la conversación. No se guarda fichero — la consulta es semanal y desechable.

Si el usuario pide guardarlo:
```bash
python3 ./scripts/analyze_attendance.py < /tmp/att_data.json > informe_fichajes_2026_W17.md
```

## Formato del informe

- Cabecera con número de semana ISO y rango de fechas
- Resumen (empleados, días, incidencias)
- Empleados con incidencias: tabla Día | Fecha | Tipo | Registros | Horas | Estado | Detalle
- Empleados sin incidencias: lista corta
- Iconos: 🔴 (incidencia grave) / 🟡 (aviso) / 🟢 (OK)
- Español profesional

## Cálculos

- Tolerancia horas: **±1 hora** sobre el objetivo (8h normal / 7h intensiva)
- Pausa comida (jornada partida): **30 min ≤ pausa ≤ 1h30**
- Cálculo de horas: suma de `(check_out - check_in)` de todos los registros del día
- Si `check_out` es `False` (sesión abierta): ignorar ese registro y reportarlo como aviso
- Tipo jornada: intensiva si `weekday == 4` (viernes) **o** `month in (7, 8)`
- Festivos: `resource.calendar.leaves` con `resource_id` vacío que solape con el día
- Vacaciones: `hr.leave` con `state == 'validate'` y `date_from <= día <= date_to`
