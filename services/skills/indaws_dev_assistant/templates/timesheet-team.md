# Template: Informe de Horas de Equipo

Formato de salida para el informe de horas facturables de uno o varios equipos.

## ⚠️ RESTRICCIÓN CRÍTICA: SOLO DESARROLLO

**TODO el informe es EXCLUSIVO para empleados del departamento de Desarrollo (`department_id = 4`).**

---

## ⛔ REGLAS DE CÁLCULO - NO ESTIMAR NUNCA

**CRÍTICO: Los errores de cálculo son INACEPTABLES. Seguir estas reglas SIN EXCEPCIÓN.**

### Campos de Odoo (account.analytic.line)

| Campo | Qué es | Cómo usar |
|-------|--------|-----------|
| `unit_amount` | Horas totales imputadas | `sum(unit_amount)` = Total |
| `billable_unit_amount` | Horas FACTURABLES | `sum(billable_unit_amount)` = Fact. |
| `unbillable_unit_amount` | Horas NO facturables | `sum(unbillable_unit_amount)` = No Fact. |
| `x_studio_bonus_unit_amount` | Horas en tiempo (para bonus) | `sum(x_studio_bonus_unit_amount)` = En tiempo |

### Reglas OBLIGATORIAS

1. **NUNCA estimar visualmente** - SIEMPRE sumar los campos del JSON
2. **NUNCA redondear arbitrariamente** - Usar los valores exactos
3. **NUNCA inventar datos** - Solo mostrar lo que devuelve Odoo

### Validación OBLIGATORIA

**Verificar SIEMPRE que: `billable + unbillable ≈ total` (tolerancia ±0.5h)**

Si NO cuadra, hay un ERROR en los cálculos. Revisar antes de mostrar el informe.

### Script de cálculo recomendado

```bash
# Usar calculate_hours.py para cálculos precisos
./scripts/get_timesheets.sh 175 2026-01-01 2026-01-31 2>/dev/null | python3 scripts/calculate_hours.py
```

El script devuelve JSON con:
- Totales calculados correctamente
- Validación automática (billable + unbillable = total)
- Proyectos con horas no facturables

### Ejemplo de cálculo correcto

```
Registros JSON: [{"unit_amount": 8.0, "billable_unit_amount": 6.0, "unbillable_unit_amount": 2.0}, ...]

Cálculo:
- Total = sum(unit_amount) = 8.0 + ... = 147.0
- Facturables = sum(billable_unit_amount) = 6.0 + ... = 63.0
- No Facturables = sum(unbillable_unit_amount) = 2.0 + ... = 84.0

Validación: 63.0 + 84.0 = 147.0 ✅ (coincide con total)

Resultado: 147:00 total (63:00 fact. / 84:00 no fact.) = 42.9% facturable
```

---

Cuando el usuario pida "informe del equipo X" o "informe de los equipos X, Y, Z":

### Paso 1: Obtener ID del equipo por nombre

**⚠️ OBLIGATORIO: Si el usuario indica NOMBRE de equipo, buscar primero el ID.**

```bash
./scripts/get_teams.sh Tokyo    # → [{"id": 28, "name": "Tokyo"}]
./scripts/get_teams.sh Monaco   # → [{"id": 33, "name": "Mónaco"}]
```

**Equipos conocidos:**
| ID | Nombre |
|----|--------|
| 28 | Tokyo |
| 29 | Roma |
| 30 | Paris |
| 31 | Programación |
| 32 | Servicios |
| 33 | Mónaco |

### Paso 2: Obtener empleados del equipo

Una vez obtenido el ID, usar `get_team_employees.sh`:

```bash
./scripts/get_team_employees.sh 28           # Un equipo (Tokyo)
./scripts/get_team_employees.sh "[28,33]"    # Varios equipos
./scripts/get_team_employees.sh              # Todo Desarrollo
```

### Paso 3: Filtrar solo Desarrollo

El script `get_team_employees.sh` ya filtra por `department_id = 4` automáticamente.

NUNCA incluir empleados de otros departamentos, aunque pertenezcan a los equipos indicados.

---

## Cabecera

**Un equipo:**
```
📊 **Horas Equipo [Nombre Equipo] - [Mes Año]**
*(Solo empleados de Desarrollo)*
```

**Varios equipos:**
```
📊 **Horas Equipos [Nombre 1], [Nombre 2] - [Mes Año]**
*(Solo empleados de Desarrollo)*
```

**Departamento completo:**
```
📊 **Horas Desarrollo - [Mes Año]**
```

---

## Tabla de Empleados

**SOLO empleados de los equipos indicados que pertenezcan al departamento de Desarrollo.**

Ordenar por % facturable (menor a mayor) para destacar problemas.

### ⚠️ FORMATO OBLIGATORIO - NO MODIFICAR

**La tabla DEBE tener EXACTAMENTE estas 9 columnas. NO omitir ninguna. NO añadir columnas extra.**

```
| Empleado | Imputadas | Fact. | En tiempo | No Fact. | Objetivo | % Fact. | Interno | Estado |
|----------|-----------|-------|-----------|----------|----------|---------|---------|--------|
| Empleado A | 100:50 | 62:55 | 60:00 | 37:55 | 75% | 62% | 28:35 (28%) | 🔴 |
| Empleado B | 120:00 | 108:00 | 100:30 | 12:00 | 60% | 90% | 05:00 (4%) | ✅ |
| Empleado C | 85:30 | 72:00 | 68:45 | 13:30 | 60% | 84% | 08:00 (9%) | ✅ |
| Empleado D | 95:00 | 65:00 | 55:00 | 30:00 | 70% | 68% | 15:00 (16%) | 🔴 |
```

### Columnas OBLIGATORIAS

| # | Columna | Formato | Descripción |
|---|---------|---------|-------------|
| 1 | **Empleado** | Texto | Nombre del empleado (SOLO de Desarrollo) |
| 2 | **Imputadas** | HH:MM | Total horas imputadas en el periodo |
| 3 | **Fact.** | HH:MM | Horas facturables |
| 4 | **En tiempo** | HH:MM | Horas facturables imputadas en tiempo (para bonus) |
| 5 | **No Fact.** | HH:MM | Horas no facturables |
| 6 | **Objetivo** | XX% | Objetivo del empleado (`effective_hours_pct` × 100) |
| 7 | **% Fact.** | XX% | Porcentaje de horas facturables sobre total |
| 8 | **Interno** | HH:MM (XX%) | Horas en proyecto Interno y su % sobre el total |
| 9 | **Estado** | Emoji | ✅ OK / 🟡 AVISO / 🔴 CRÍTICO |

**NUNCA mostrar tablas con menos columnas. NUNCA omitir la columna Interno o Objetivo.**

### Criterios de Estado

El estado se determina comparando el **% facturable real** vs el **objetivo individual**:

| % Facturable vs Objetivo | Estado |
|--------------------------|--------|
| >= objetivo | ✅ OK |
| objetivo - 10% a objetivo | 🟡 AVISO |
| < objetivo - 10% | 🔴 CRÍTICO |

**Ejemplo**: Si objetivo = 75%:
- >= 75% = ✅ OK
- 65-74% = 🟡 AVISO
- < 65% = 🔴 CRÍTICO

**Interno > 25%**: Aviso adicional, pero NO cambia el estado del empleado.

---

## Resumen del Equipo/Equipos

```
---

**Resumen** *(solo empleados de Desarrollo)*:
- Total imputado: **401:20**
- Total facturable: **307:55** (77%)
- Total no facturable: **93:25** (23%)
- Media % facturable: **76%**
```

---

## Detalle por Empleado

Después de la tabla resumen, mostrar un desglose detallado de cada empleado ordenado por estado (primero 🔴, luego 🟡, luego ✅).

### ⚠️ FORMATO OBLIGATORIO - NO MODIFICAR

**Cada empleado DEBE tener EXACTAMENTE estos campos en este orden:**

```
---

### Detalle por Empleado

**[Nombre Empleado]** [EMOJI_ESTADO]

- **Horas:** HHH:MM (HH:MM fact. *(HH:MM en tiempo)* / HH:MM no fact.)
- **Ausencias:** X días [de vacaciones] [(fechas)]
- **Proyectos principales:** Proyecto1, Proyecto2, Proyecto3
- **Proyectos con horas no facturables:**
  * Proyecto A: HH:MM no facturables
  * Proyecto B: HH:MM no facturables
- **Horas esperadas ajustadas:** HHH:MM (X días laborables)
- **Estado:** [CRÍTICO/AVISO/Cumple objetivo] - XX.X% vs objetivo XX%
- **Observación:** [Texto contextual explicando la situación]

---

**[Nombre Empleado 2]** [EMOJI_ESTADO]

- **Horas:** HHH:MM (HH:MM fact. *(HH:MM en tiempo)* / HH:MM no fact.)
...
```

### Campos OBLIGATORIOS

| # | Campo | Cuándo mostrar | Formato |
|---|-------|----------------|---------|
| 1 | **Nombre + Emoji** | SIEMPRE | `**Nombre** ✅/🟡/🔴` |
| 2 | **Horas** | SIEMPRE | HH:MM (HH:MM fact. *(HH:MM en tiempo)* / HH:MM no fact.) |
| 3 | **Ausencias** | SIEMPRE | X días. Si 0: "0 días". Si hay: incluir fechas |
| 4 | **Proyectos principales** | SIEMPRE | Lista de proyectos (máx 5, por horas desc) |
| 5 | **Proyectos con horas no facturables** | SOLO si hay | Lista con HH:MM no facturables por proyecto |
| 6 | **Horas esperadas ajustadas** | SOLO si hay ausencias | HH:MM (X días laborables) |
| 7 | **Estado** | SIEMPRE | CRÍTICO/AVISO/Cumple objetivo con % real vs objetivo |
| 8 | **Observación** | SOLO si relevante | Contexto explicativo |

**Nota sobre "En tiempo":** Las horas "en tiempo" son las horas facturables imputadas a tiempo (campo `x_studio_bonus_unit_amount`). Se muestran en *cursiva* para destacarlas. Estas horas cuentan para el bonus del empleado.

**⚠️ FORMATO DE HORAS: SIEMPRE HH:MM, NUNCA decimal.**

### Ejemplo completo

```
---

### Detalle por Empleado

**Akash** ✅

- **Horas:** 156:15 (93:50 fact. *(90:00 en tiempo)* / 62:25 no fact.)
- **Ausencias:** 0 días
- **Proyectos principales:** Migración HomeServe v18, Maximice Events, AkuPalma, Ingeniatic
- **Estado:** Cumple objetivo (60.1% >= 60%)

---

**Brenda** 🔴

- **Horas:** 64:00 (19:35 fact. *(15:00 en tiempo)* / 44:25 no fact.)
- **Ausencias:** 6 días de vacaciones (2-5 y 7-13 enero)
- **Proyectos principales:** Autosur, Almacenes Elite, Leleman (no factur.)
- **Proyectos con horas no facturables:**
  * Leleman: 28:00 no facturables
  * Formación interna: 12:00 no facturables
- **Horas esperadas ajustadas:** 120:00 (15 días laborables)
- **Estado:** CRÍTICO - 30.6% vs objetivo 60%
- **Observación:** Alta carga de formación y tareas internas

---

**Laura M. Mantecón** 🔴

- **Horas:** 161:00 (20:52 fact. *(18:30 en tiempo)* / 140:08 no fact.)
- **Ausencias:** 0 días
- **Proyectos principales:** Aloxy (Dynapps), Keycoes, Pharmactive, Distribuciones Batoy
- **Proyectos con horas no facturables:**
  * Aloxy (Dynapps): 70:00 no facturables
  * Keycoes: 02:00 no facturables
  * Pharmactive: 04:00 no facturables
  * Distribuciones Batoy: 02:00 no facturables
- **Estado:** CRÍTICO - 13.0% vs objetivo 60%
- **Observación:** Alta dedicación a proyecto interno Aloxy (Dynapps) - no facturable

---

**Toni Hermosilla** ✅

- **Horas:** 82:30 (51:10 fact. *(48:00 en tiempo)* / 31:20 no fact.)
- **Ausencias:** 6 días de vacaciones (23-30 enero)
- **Proyectos principales:** Gastraval, MR Adhesivos, Ingeniatic, Distribuciones Batoy, BBVA
- **Horas esperadas ajustadas:** 120:00 (15 días laborables)
- **Estado:** Cumple objetivo (62.0% >= 60%)
```

### Criterios para Observación

Incluir observación cuando:
- Proyecto interno > 25% del total → "Alta dedicación a proyecto interno [nombre]"
- Muchas horas de formación → "Alta carga de formación"
- Proyecto no facturable dominante → "Alta dedicación a [proyecto] - no facturable"
- Migración/importador → "Mucho trabajo en importadores/migraciones no facturables"

### Proceso de generación

1. Obtener empleados con `scripts/get_team_employees.sh [TEAM_IDS]`
2. Para cada empleado, obtener partes con `scripts/get_timesheets.sh`
3. Obtener ausencias con `scripts/get_leaves.sh`
4. Agrupar partes por proyecto para obtener:
   - Proyectos principales (ordenados por horas totales)
   - Proyectos con horas no facturables (solo los que tienen `unbillable_unit_amount > 0`)
5. Calcular horas esperadas ajustadas si hay ausencias
6. Determinar estado comparando % facturable vs objetivo individual
7. Generar observación si aplica algún criterio
8. Ordenar empleados: primero 🔴, luego 🟡, luego ✅

---

## Acciones Recomendadas

```
---

💡 **Acciones recomendadas:**
1. Seguimiento con empleados en estado CRÍTICO
2. Revisar dedicación a proyectos internos no facturables
3. Validar si las horas no facturables son correctas o hay error de imputación
```

---

## Informe de Departamento Completo

Cuando se pide informe de todo el departamento de Desarrollo, agrupar empleados por equipo.

### ⚠️ FORMATO OBLIGATORIO

**Estructura del informe de departamento:**
1. Tabla resumen por equipo (9 columnas obligatorias)
2. Subtotal por equipo
3. Resumen global
4. Detalle por empleado (mismo formato que sección anterior)

```
📊 **Horas Desarrollo - [Mes Año]**

## Equipo [Nombre 1]
| Empleado | Imputadas | Fact. | En tiempo | No Fact. | Objetivo | % Fact. | Interno | Estado |
|----------|-----------|-------|-----------|----------|----------|---------|---------|--------|
| Empleado A | 100:50 | 62:55 | 60:00 | 37:55 | 75% | 62% | 28:35 (28%) | 🔴 |
| Empleado B | 120:00 | 108:00 | 100:30 | 12:00 | 60% | 90% | 05:00 (4%) | ✅ |

**Subtotal Equipo [Nombre 1]**: 220:50 imputadas, 76% facturable

---

## Equipo [Nombre 2]
| Empleado | Imputadas | Fact. | En tiempo | No Fact. | Objetivo | % Fact. | Interno | Estado |
|----------|-----------|-------|-----------|----------|----------|---------|---------|--------|
| Empleado C | 95:30 | 80:00 | 75:00 | 15:30 | 60% | 84% | 10:00 (10%) | ✅ |
| Empleado D | 85:00 | 65:00 | 60:00 | 20:00 | 70% | 76% | 08:00 (9%) | ✅ |

**Subtotal Equipo [Nombre 2]**: 180:30 imputadas, 80% facturable

---

## Resumen Global
- Total imputado: **401:20**
- Total facturable: **315:55** (79%)
- Equipo con mejor % facturable: **Equipo [Nombre 2] (80%)**
- Equipo con peor % facturable: **Equipo [Nombre 1] (76%)**

---

### Detalle por Empleado

[Incluir detalle de TODOS los empleados siguiendo el formato de la sección "Detalle por Empleado"]
[Ordenar por estado: primero 🔴, luego 🟡, luego ✅]
```

**Usar**: `scripts/get_team_employees.sh` (sin argumento) para obtener todos los empleados de Desarrollo.
