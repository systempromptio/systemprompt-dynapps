# Métricas de Salud del Proyecto

Documento de referencia con los algoritmos de cálculo, umbrales y casos especiales para evaluar el estado de salud de un proyecto Odoo.

---

## 1. Métricas Core

### 1.1 Timeline (T%) - Avance Temporal

Porcentaje del tiempo del proyecto que ha transcurrido.

```
T% = (fecha_referencia - fecha_inicio) / (fecha_fin - fecha_inicio) * 100
```

**Campos Odoo**:
- `fecha_inicio`: `project.project.date_start`
- `fecha_fin`: `project.project.date` (fecha fin planificada)
- `fecha_referencia`: Fecha proporcionada por el usuario (default: hoy)

**Reglas**:
- Si `fecha_inicio` es null → usar fecha de creación del proyecto
- Si `fecha_fin` es null → **ERROR**: no se puede calcular sin fecha fin planificada
- Si `fecha_referencia < fecha_inicio` → T% = 0 (proyecto no iniciado)
- Si `fecha_referencia > fecha_fin` → T% > 100 (proyecto vencido, ver sección 3.2)

---

### 1.2 Checklists Ponderados (C%) - Progreso Real

Porcentaje de avance basado en los pesos de los items del checklist.

```
C% = SUM(peso de items completados) / SUM(peso de todos los items) * 100
```

**Campos Odoo** (modelo `xx.checklist.item`):
- `weight`: Peso del item (1-5, default 1 si null o 0)
- `state`: Selection (`pending` · `in_progress` · `waiting` · `done`). Completado = `state == 'done'`

**Reglas**:
- Items con `weight` null o 0 → usar peso = 1
- Si no hay items de checklist → C% no aplica (ver sección 3.3)
- Los items se agrupan por tarea (`task_id`) para métricas por tarea

**Ejemplo**:
| Item | Peso | Completado |
|------|------|------------|
| Config CRM | 3 | Sí |
| Migración datos | 5 | No |
| Formación | 2 | Sí |
| **Totales** | **10** | **5/10** |

→ C% = (3 + 2) / (3 + 5 + 2) * 100 = **50%**

---

### 1.3 Horas Consumidas (H%) - Consumo de Presupuesto

Porcentaje de horas consumidas respecto al presupuesto.

```
H% = horas_consumidas / horas_presupuestadas * 100
```

**Fuente de datos para horas (por orden de prioridad)**:

1. **`project.project.total_hours`** (PREFERIDO): Campo del proyecto con el total de horas vendidas/presupuestadas. Datos complementarios:
   - `total_hours_available`: Horas disponibles aún
   - `total_unbillable_hours`: Horas no facturables consumidas
   - `total_billable_hours` (en `project.project.hour`): Horas facturables consumidas
   - `horas_consumidas = total_hours - total_hours_available`

2. **`project.project.hour`**: Modelo de asignación de horas del proyecto. Contiene `hours` (total asignado) y está vinculado a un pedido de venta (`x_studio_sale_order_id`).

3. **`project.project.hour.line`**: Desglose por departamento. Campos clave:
   - `department_id`: Departamento (ej. Consultoría, Desarrollo)
   - `hours`: Horas asignadas al departamento
   - `total_billable_hours`: Horas facturables consumidas por departamento
   - `total_hours_available`: Horas disponibles por departamento

4. **Fallback**: Si `total_hours` = 0, usar `SUM(project.task.allocated_hours)` como aproximación.

**Reglas**:
- Si `total_hours` = 0 y suma de tareas = 0 → H% no aplica (ver sección 3.4)
- H% puede superar 100% (sobrecosto)

---

## 2. Health Score (S)

Puntuación global de salud del proyecto en escala 0-100.

### 2.1 Fórmula Principal

```
S = clamp(100 - Pen_rendimiento - Pen_gobernanza, 0, 100)
```

Donde:
```
Pen_rendimiento = Pen_checklist + Pen_horas_exceso + Pen_horas_baja + Pen_tareas_vencidas
Pen_gobernanza  = Pen_sin_checklists + Pen_desajuste_horas + Pen_tareas_sin_fecha
```

---

### 2.2 Penalizaciones de Rendimiento

#### a) Penalización Checklist (progreso vs. tiempo)

```
Pen_checklist = max(0, -(C% - T%)) * 1.5
```

- Aplica cuando C% < T% (progreso menor que tiempo transcurrido)
- Factor 1.5x: es la penalización más grave
- Si no hay checklists → Pen_checklist = 0 (pero se aplica Pen_sin_checklists en gobernanza)

#### b) Penalización Horas Exceso (sobrecosto)

```
Pen_horas_exceso = max(0, H% - T%) * 1.0
```

- Aplica cuando H% > T% (quemando presupuesto más rápido que el tiempo)
- Factor 1.0x

#### c) Penalización Horas Baja (posible bloqueo)

```
Pen_horas_baja = max(0, -(H% - T%) - 15) * 0.3
```

- Aplica cuando H% < T% - 15 (margen de 15pp antes de penalizar)
- Factor 0.3x: menor gravedad, puede ser falta de registro

#### d) Penalización Tareas Vencidas

Para cada tarea con `date_deadline` pasado Y etapa NO en ("Done", "CANCELADO"):

```
dias_retraso = fecha_referencia - date_deadline
factor_retraso = min(dias_retraso, 30) / 30
Pen_tarea = factor_retraso * 10

Pen_tareas_vencidas = SUM(Pen_tarea para cada tarea vencida)
```

- Máximo 10 puntos por tarea vencida
- Se usa `stage_id` para verificar que la tarea no está completada ni cancelada
- Etapas de finalización: nombres que contengan "Done", "Hecho", "Completado", "Cerrado"
- Etapas de cancelación: nombres que contengan "CANCELADO", "Cancelado", "Cancel"

**Ejemplo**: 4 tareas vencidas (24, 10, 10, 3 días)
```
Pen = (24/30)*10 + (10/30)*10 + (10/30)*10 + (3/30)*10 = 8.0 + 3.3 + 3.3 + 1.0 = 15.6
```

---

### 2.3 Penalizaciones de Gobernanza

#### a) Sin Checklists

Si el proyecto no tiene registros en `xx.checklist.item`:

```
Pen_sin_checklists = 15 (fija)
```

Sin checklists no se puede medir el avance real del trabajo. Es un problema de gobernanza grave.

→ Indicar en el informe: "⚠️ ALERTA: Sin checklists. No es posible medir el avance real del proyecto. Se recomienda crear los checklists urgentemente."

#### b) Desajuste Horas Tareas vs. Proyecto

Comparar la suma de `allocated_hours` de todas las tareas con `total_hours` del proyecto:

```
suma_tareas = SUM(project.task.allocated_hours)
total_proyecto = project.project.total_hours
diff_pct = abs(suma_tareas - total_proyecto) / total_proyecto * 100
Pen_desajuste_horas = min(diff_pct * 0.3, 10)
```

- Tolerancia: Si diff_pct < 5% → no penalizar (margen razonable)
- Máximo 10 puntos de penalización
- Indica horas sin asignar a tareas o tareas con horas no alineadas al presupuesto

→ Indicar en el informe: "⚠️ ALERTA: Desajuste de XXXh entre horas asignadas en tareas (XXXh) y presupuesto del proyecto (XXXh)."

#### c) Tareas Sin Fecha Límite

Para tareas activas (no Done, no CANCELADO) sin `date_deadline`:

```
tareas_sin_fecha = count(tareas activas sin date_deadline)
Pen_tareas_sin_fecha = tareas_sin_fecha * 2
```

- 2 puntos por cada tarea sin fecha
- Máximo recomendado: cap a 20 puntos → `min(tareas_sin_fecha * 2, 20)`
- Tareas sin fecha dificultan el seguimiento del cronograma

→ Indicar en el informe: "⚠️ ALERTA: X tareas sin fecha límite asignada."

#### d) Tareas Sin Checklists (>= 5h planificadas)

Toda tarea activa con `allocated_hours >= 5` debe tener al menos 1 item de checklist asociado. Esto garantiza que las tareas relevantes tienen seguimiento granular.

```
tareas_sin_check = count(tareas activas con allocated_hours >= 5 Y 0 items en xx.checklist.item)
Pen_tareas_sin_check = min(tareas_sin_check * 2, 15)
```

- 2 puntos por cada tarea sin checklists
- Máximo 15 puntos de penalización
- Se excluyen tareas con < 5h (tareas menores que no requieren desglose)

→ Indicar en el informe: "⚠️ ALERTA: X tareas con >= 5h planificadas no tienen checklists asignados."

---

### 2.4 Tabla Resumen de Penalizaciones

| Categoría | Penalización | Cuándo aplica | Máximo |
|-----------|-------------|---------------|--------|
| **RENDIMIENTO** | | | |
| Pen_checklist | (T% - C%) * 1.5 | C% < T% | ~75 |
| Pen_horas_exceso | (H% - T%) * 1.0 | H% > T% | ~50 |
| Pen_horas_baja | (T% - H% - 15) * 0.3 | H% < T% - 15 | ~15 |
| Pen_tareas_vencidas | sum(factor * 10) | Tareas vencidas | ~50 |
| **GOBERNANZA** | | | |
| Pen_sin_checklists | 15 (fija) | 0 checklists | 15 |
| Pen_desajuste_horas | diff_pct * 0.3 | diff > 5% | 10 |
| Pen_tareas_sin_fecha | count * 2 | Tareas sin fecha | 20 |
| Pen_tareas_sin_check | count * 2 | Tareas >=5h sin items | 15 |

---

### 2.5 Umbrales de Clasificación

| Rango | Estado | Color | Icono | Descripción |
|-------|--------|-------|-------|-------------|
| 80-100 | SALUDABLE | Verde | `🟢` | Proyecto en buen camino |
| 60-79 | EN RIESGO | Amarillo | `🟡` | Desviaciones detectadas, requiere atención |
| 40-59 | CRÍTICO | Naranja | `🟠` | Desviaciones significativas, acción inmediata requerida |
| 0-39 | EMERGENCIA | Rojo | `🔴` | Proyecto en peligro serio, intervención urgente |

---

### 2.6 Barra Visual del Score

Generar una barra visual en texto para el informe:

```
S = 72 → 🟡 EN RIESGO  [████████████████████░░░░░░░░░░] 72/100
S = 85 → 🟢 SALUDABLE  [█████████████████████████████░] 85/100
S = 45 → 🟠 CRÍTICO    [█████████████████░░░░░░░░░░░░░] 45/100
S = 25 → 🔴 EMERGENCIA [████████░░░░░░░░░░░░░░░░░░░░░░] 25/100
```

Usar 30 caracteres para la barra. Caracteres llenos = `round(S / 100 * 30)`.

---

## 3. Casos Especiales

### 3.1 Proyecto Vencido (T% > 100%)

Cuando la fecha fin ha pasado:

```
T% se calcula normalmente (será > 100)
Penalización_vencido = (T% - 100) * 2.0
```

Se suma a las penalizaciones de rendimiento.

→ Indicar en el informe: "⚠️ Proyecto VENCIDO. Fecha fin planificada superada en X días."

---

### 3.2 Sin Checklists

Si el modelo `xx.checklist.item` no existe o no tiene registros:

- `Pen_checklist` = 0 (no se puede evaluar rendimiento de checklists)
- `Pen_sin_checklists` = 15 (penalización de gobernanza)
- Score se calcula con las penalizaciones de horas, tareas vencidas y gobernanza

→ Indicar en el informe: "⚠️ ALERTA: Sin datos de checklists. Score parcial."

---

### 3.3 Sin Horas Presupuestadas

Si `total_hours` = 0 y no hay `allocated_hours` en tareas:

- `Pen_horas_exceso` = 0
- `Pen_horas_baja` = 0
- `Pen_desajuste_horas` = 0

→ Indicar en el informe: "⚠️ Sin datos de horas presupuestadas."

---

### 3.4 Sin Checklists NI Horas

Si faltan ambos y no hay tareas vencidas:

```
S = 50 (sin datos suficientes para evaluar)
Estado = "INDETERMINADO"
```

→ Indicar en el informe: "❌ Datos insuficientes para evaluar la salud del proyecto."

---

## 4. Análisis de Etapas de Tareas

### 4.1 Mapeo de Etapas

Las etapas (`stage_id`) de las tareas reflejan el progreso real. Clasificar:

| Categoría | Etapas típicas | Significado |
|-----------|---------------|-------------|
| **No iniciada** | Nuevo, Backlog | Tarea pendiente de inicio |
| **Planificación** | A planificar, Analysis, Tareas Base | En fase de definición |
| **En progreso** | En desarrollo, In Progress | Trabajo activo |
| **Revisión** | Ready for review, Testeo cliente | Pendiente de validación |
| **Completada** | Done, Hecho, Completado | Finalizada |
| **Cancelada** | CANCELADO, Cancelado | Descartada |

### 4.2 Indicador de Progreso por Etapas

```
tareas_completadas = count(tareas con etapa "Completada")
tareas_activas = count(total) - count("Cancelada")
pct_completadas = tareas_completadas / tareas_activas * 100
```

Comparar con T%:
- Si `pct_completadas` << T% → Alerta: pocas tareas se están cerrando
- Si `pct_completadas` ≈ T% → Ritmo adecuado

---

## 5. Métricas Secundarias

### 5.1 Burn Rate (Tasa de Consumo de Horas)

```
burn_rate_semanal = horas_consumidas / semanas_transcurridas
semanas_transcurridas = (fecha_referencia - fecha_inicio) / 7
```

**Por departamento** (usando `project.project.hour.line`):
```
burn_rate_consultoria = horas_consumidas_consultoria / semanas_transcurridas
burn_rate_desarrollo = horas_consumidas_desarrollo / semanas_transcurridas
```

**Interpretación**:
- `burn_rate_esperado = horas_presupuestadas / semanas_totales`
- Si `burn_rate_real > burn_rate_esperado * 1.2` → "Ritmo de consumo alto"
- Si `burn_rate_real < burn_rate_esperado * 0.5` → "Ritmo de consumo bajo"

---

### 5.2 Velocidad de Checklists (Puntos/Semana)

```
velocidad_puntos = peso_completado / semanas_transcurridas
velocidad_requerida = peso_pendiente / semanas_restantes
factor_aceleracion = velocidad_requerida / velocidad_puntos
```

Si no hay checklists → N/A.

---

### 5.3 Distribución por Tarea

Para cada tarea del proyecto, calcular:

```
tarea_H% = horas_consumidas_tarea / horas_asignadas_tarea * 100
tarea_risk = (1 - tarea_progreso) * factor_vencimiento * factor_peso_horas
```

Donde `factor_vencimiento`:
- Tarea sin fecha límite → 1.0
- Fecha límite > 2 semanas → 1.0
- Fecha límite 1-2 semanas → 1.5
- Fecha límite < 1 semana → 2.0
- Fecha límite pasada → 3.0

---

### 5.4 EAC (Estimate at Completion)

Proyección del total de horas al finalizar:

```
EAC = horas_consumidas / (C% / 100)
```

Si C% = 0 o N/A → usar aproximación por T%: `EAC = horas_consumidas / (T% / 100)`

**Interpretación**:
- `EAC <= horas_presupuestadas` → Dentro de presupuesto
- `EAC > horas_presupuestadas` → Sobrecosto proyectado

---

## 6. Agrupación de Checklists por Tarea

Para las métricas por tarea, agrupar items de `xx.checklist.item` por `task_id`:

```
Para cada tarea:
  items = filtrar checklists donde task_id = tarea.id
  tarea.check_total = count(items)
  tarea.check_completados = count(items donde state = 'done')
  tarea.peso_total = sum(item.weight || 1)
  tarea.peso_completado = sum(item.weight || 1) para completados
  tarea.C% = tarea.peso_completado / tarea.peso_total * 100
```

Items sin `task_id` → agrupar como "General / Sin tarea asignada".
