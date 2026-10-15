# Template: Informe Individual de Horas

Formato de salida para el informe de partes de horas de un empleado.

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
./scripts/get_timesheets.sh <employee_id> <from> <to> 2>/dev/null | python3 scripts/calculate_hours.py
```

---

## Cabecera

```
📊 **Partes de Horas - [Nombre Empleado]**
*[Mes Año] ([DD/MM/YYYY] - [DD/MM/YYYY]) - Excluye vacaciones/festivos*
*Objetivo horas facturables: [XX]%* (de hr.employee.effective_hours_pct)
```

## Tabla de Proyectos

Ordenar por horas (mayor a menor). Formato nombre: `#ID - Nombre Proyecto`.

### ⚠️ FORMATO OBLIGATORIO - NO MODIFICAR

**La tabla DEBE tener EXACTAMENTE estas 4 columnas. NO omitir ninguna.**

```
| Proyecto | Horas | Facturables | No Facturables |
|----------|-------|-------------|----------------|
| #1 - Interno | 28:35 | 00:00 | **28:35** 🔴 |
| #930 - Gastraval bolsa 250 h | 19:45 | 19:45 | 00:00 ✅ |
| #461 - MR Adhesivos | 16:15 | 16:15 | 00:00 ✅ |
```

### Columnas OBLIGATORIAS

| # | Columna | Formato | Descripción |
|---|---------|---------|-------------|
| 1 | **Proyecto** | #ID - Nombre | Formato obligatorio con ID y nombre |
| 2 | **Horas** | HH:MM | Total horas imputadas en el proyecto |
| 3 | **Facturables** | HH:MM | Horas facturables |
| 4 | **No Facturables** | HH:MM + emoji | Horas no facturables con indicador |

### Indicadores en No Facturables

| Valor | Indicador |
|-------|-----------|
| `00:00` | ✅ |
| Proyecto 100% no facturable (excepto Interno < 25%) | `**HH:MM** 🔴` |
| Proyecto Interno <= 25% | `**HH:MM** 🟡` |

## Totales

```
---

**Totales:**
- Horas imputadas: **100:50**
- Facturables: **62:55** *(58:30 en tiempo)* (62%) 🔴
- No facturables: **37:55** (38%)
- Objetivo: **75%**
```

**Nota sobre "En tiempo":** Las horas "en tiempo" son las horas facturables imputadas a tiempo (campo `x_studio_bonus_unit_amount`). Se muestran en *cursiva* para destacarlas. Estas horas cuentan para el bonus del empleado.

### Indicadores de Totales (vs Objetivo Individual)

El estado se determina comparando el **% facturable real** vs el **objetivo individual** (`effective_hours_pct` del empleado):

| % Facturable vs Objetivo | Indicador |
|--------------------------|-----------|
| >= objetivo | ✅ OK |
| objetivo - 10% a objetivo | 🟡 Aviso |
| < objetivo - 10% | 🔴 Crítico |

**Ejemplo**: Si objetivo = 75%:
- >= 75% = ✅ OK
- 65-74% = 🟡 Aviso
- < 65% = 🔴 Crítico

**Ejemplo**: Si objetivo = 60%:
- >= 60% = ✅ OK
- 50-59% = 🟡 Aviso
- < 50% = 🔴 Crítico

## Tabla de Proyectos con Horas No Facturables

**IMPORTANTE**: Mostrar TODOS los proyectos donde el empleado ha imputado horas NO FACTURABLES en el periodo, sin excepción.

### ⚠️ FORMATO OBLIGATORIO - NO MODIFICAR

**La tabla DEBE tener EXACTAMENTE estas 5 columnas. NO omitir ninguna.**

```
---

### 🔴 PROYECTOS CON DESVIACIÓN NEGATIVA

| Proyecto | No Fact. (periodo) | Asignadas | Exceso histórico | Estado |
|----------|-------------------|-----------|------------------|--------|
| #442 - Consultoria | 02:30 | - | - | 🔴 Sin asignación |
| #997 - Ostras Migración | 03:45 | 65:00 | -178:45 | 🔴 **-275%** |
| #1033 - Autosur Análisis | 01:15 | 03:00 | -05:15 | 🔴 **-175%** |
| #462 - Metafrase | 00:30 | 88:00 | -09:15 | 🟡 **-11%** |
| #899 - Eina Activa | 00:15 | 110:00 | -02:30 | 🟡 **-2%** |
| #1003 - ZINCO AI | 00:30 | 13:00 | 00:00 | ✅ OK |
```

### Columnas OBLIGATORIAS

| # | Columna | Formato | Descripción |
|---|---------|---------|-------------|
| 1 | **Proyecto** | #ID - Nombre | Formato obligatorio con ID y nombre |
| 2 | **No Fact. (periodo)** | HH:MM | Horas no facturables del periodo |
| 3 | **Asignadas** | HH:MM o `-` | Horas asignadas, `-` si no hay |
| 4 | **Exceso histórico** | HH:MM o `-` | Acumulado, `-` si no hay |
| 5 | **Estado** | Ver formato | `🔴 Sin asignación` / `🔴 **-XX%**` / `🟡 **-XX%**` / `✅ OK` |

**Leyenda:**
- **No Fact. (periodo)**: Horas no facturables imputadas en el periodo consultado
- **Asignadas**: Campo `hours` de `project.project.hour.line` (dept. Desarrollo o global)
- **Exceso histórico**: Campo `total_unbillable_hours` (acumulado)
- **%**: Porcentaje de exceso = (total_unbillable_hours / hours) × 100
- **Estado**: 🔴 Sin asignación / 🔴 >30% exceso / 🟡 1-30% exceso / ✅ OK

### Proceso de generación

1. Obtener TODOS los partes de hora del empleado en el periodo
2. Agrupar por proyecto y sumar `unbillable_unit_amount`
3. Filtrar proyectos con horas no facturables > 0
4. Para cada proyecto, buscar su línea de asignación en `project.project.hour.line`:
   - Primero buscar con `department_id = 4` (Desarrollo)
   - Si no existe, usar línea con `department_id = false` (global)
   - Si no existe ninguna → "Sin asignación"
5. Calcular estado según % de exceso histórico
6. Ordenar: primero "Sin asignación", luego por % exceso descendente
7. **Excluir**: Proyecto Interno (id=1)

### Criterios de Estado de Proyecto

| Condición | Estado |
|-----------|--------|
| Sin línea de asignación | 🔴 Sin asignación |
| Exceso histórico > 30% | 🔴 **-XX%** |
| Exceso histórico 1-30% | 🟡 **-XX%** |
| Exceso histórico <= 0% | ✅ OK |

## Alarmas

```
---

### 🚨 ALARMAS

| Nivel | Descripción |
|-------|-------------|
| 🔴 **CRÍTICO** | **Interno** representa 28% del total (>25%) |
| 🔴 **CRÍTICO** | **#997 Ostras Migración**: -275% (exceso sobre horas asignadas) |
| 🔴 **CRÍTICO** | 5 proyectos sin horas de desarrollo asignadas |
| 🔴 **CRÍTICO** | Solo 62% de horas facturables (objetivo: 75%) |
```

## Comparativa (solo si hubo ausencias)

```
---

**Comparativa con/sin vacaciones:**
| Métrica | Sin filtrar | Filtrado |
|---------|-------------|----------|
| Interno | 100:35 (58%) | 28:35 (28%) |
| Total imputado | 172:50 | 100:50 |
| % Facturable | 36% | 62% |
```
