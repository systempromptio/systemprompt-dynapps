# Patrones de Correccion

Recetas de correccion por tipo de problema detectado en el diagnostico. El LLM consulta este documento para proponer correcciones concretas al consultor.

---

## 1. Gobernanza

### 1.1 Tareas sin Fecha Limite

**Problema**: Tareas activas sin `date_deadline`. Penalizacion: 2 puntos por tarea (max 20).

**Estrategia de asignacion de fechas**:

1. Obtener timeline del proyecto: `date_start`, `date` (fin), semanas restantes
2. Identificar fase logica de cada tarea por su nombre:
   - `[CONFIG]` o "Config" -> Fase temprana (primeras semanas)
   - `[MIGR]` o "Migracion/Importacion" -> Fase media (despues de config)
   - `[DEV]` o desarrollo -> Fase media-tardia
   - `[FORM]` o "Formacion" -> Fase final (antes de go-live)
   - "Reuniones", "Soporte", "Gestion" -> Distribuir a lo largo del proyecto
3. Si hay documento de analisis: usar la secuencia de fases planificada
4. Distribuir proporcionalmente en las semanas disponibles

**Reglas**:
- Nunca asignar fecha anterior a hoy
- Respetar dependencias logicas: Config antes de Datos, Datos antes de Formacion
- Dejar margen de 1 semana antes de la fecha fin del proyecto
- Tareas genericas (reuniones, soporte): fecha fin = fecha fin del proyecto

**Comando Odoo**:
```bash
$PILOT_DIR/update_record.sh "project.task" '[TASK_ID]' '{"date_deadline":"YYYY-MM-DD"}'
```

---

### 1.2 Desajuste de Horas (Tareas vs. Presupuesto)

**Problema**: `SUM(task.allocated_hours)` difiere de `project.total_hours` en mas del 5%. Penalizacion: `min(diff_pct * 0.3, 10)`.

**Estrategias segun tipo de gap**:

#### Gap positivo (presupuesto > suma tareas):
Hay horas sin asignar a ninguna tarea.

1. **Identificar tareas con horas consumidas pero 0 asignadas**: Asignar al menos las horas ya consumidas + margen
2. **Redistribuir proporcionalmente**: Repartir el gap entre tareas existentes segun su peso relativo
3. **Absorber en tarea generica**: Asignar a "Reuniones Seguimiento", "Soporte Post-Go-Live" o "Gestion de Proyecto"

#### Gap negativo (suma tareas > presupuesto):
Las tareas suman mas que el presupuesto.

1. **Identificar tareas sobreestimadas**: Tareas con `effective_hours` << `allocated_hours` (ej. 2h consumidas de 20h asignadas en tarea al 80%)
2. **Reducir tareas no iniciadas**: Si `effective_hours = 0`, la estimacion puede ajustarse
3. **NO reducir tareas con overcost**: Si `effective_hours > allocated_hours`, no tocar

**Reglas**:
- Nunca reducir `allocated_hours` por debajo de `effective_hours`
- Si una tarea tiene `effective_hours > 0` y `allocated_hours = 0`, asignar al menos `effective_hours * 1.2`
- Priorizar absorcion en tareas genericas antes que redistribucion

**Comando Odoo**:
```bash
$PILOT_DIR/update_record.sh "project.task" '[TASK_ID]' '{"allocated_hours":HOURS}'
```

---

### 1.3 Tareas sin Checklists (>= 5h)

**Problema**: Tareas con `allocated_hours >= 5` sin items en `xx.checklist.item`. Penalizacion: 2 puntos por tarea (max 15).

**Estrategia de creacion de checklists**:

#### Si hay documento de analisis:
1. Buscar la tarea en Seccion 6 (Checklist General): extraer items con categoria y peso
2. Buscar en Seccion 7 (Tareas Detalladas): extraer checklist interno ([DEV], [TEST], [DOC])
3. Crear items con pesos del analisis

#### Si NO hay documento de analisis:
Usar items genericos segun tipo de tarea:

| Tipo tarea | Items sugeridos | Pesos |
|------------|-----------------|-------|
| CONFIG | Configurar, Validar con cliente, Documentar | 3, 2, 1 |
| DEV | Desarrollar, Testear, Code Review, Documentar | 4, 3, 2, 1 |
| MIGR | Preparar template, Importar datos, Validar datos | 3, 3, 2 |
| FORM | Preparar material, Impartir formacion, Evaluar | 2, 3, 1 |
| GENERIC | Planificar, Ejecutar, Validar | 2, 3, 2 |

**Campos para crear item**:
```json
{
  "name": "Nombre del item",
  "project_id": PROJECT_ID,
  "task_id": TASK_ID,
  "weight": PESO,
  "state": "pending",
  "date_deadline": "YYYY-MM-DD"
}
```

**Reglas**:
- Peso 1-2 para items simples, 3-5 para items complejos
- `state` arranca siempre en `pending` (sustituye al antiguo booleano a false)
- `date_deadline` es la fecha PREVISTA; `date_done` solo se rellena al completar
- La fecha limite del item debe ser <= fecha fin de la tarea
- Si la tarea no tiene fecha fin, usar fecha fin del proyecto
- Minimo 2 items por tarea, maximo 8

**Comando Odoo**:
```bash
$PILOT_DIR/create_record.sh "xx.checklist.item" '{"name":"...","project_id":ID,"task_id":TID,"weight":P,"state":"pending",...}'
```

---

### 1.4 Items sin Asignar

**Problema**: Items de checklist sin `user_id`.

**Estrategia**:
1. Asignar al responsable de la tarea padre (`user_ids` de la tarea)
2. Si la tarea no tiene responsable, asignar al PM del proyecto (`user_id` del proyecto)

**Comando Odoo**:
```bash
$PILOT_DIR/update_record.sh "xx.checklist.item" '[ITEM_ID]' '{"user_id":USER_ID}'
```

---

## 2. Progreso

### 2.1 Tareas Vencidas

**Arbol de decision** para cada tarea vencida:

```
effective_hours > 80% de allocated_hours?
  SI → Probablemente lista para cerrar
    ¿Checklist items completados al 100%?
      SI → Mover a etapa "Hecho" (cerrar)
      NO → Completar items pendientes y cerrar
  NO → La tarea aun necesita trabajo
    effective_hours = 0?
      SI → Evaluar: ¿es necesaria aun?
        SI → Reprogramar con nueva fecha
        NO → Cancelar (mover a "Cancelado")
      NO (0 < effective < 50%) → Reprogramar con nueva fecha
```

**Para reprogramar**: Proponer nueva fecha basandose en:
- Horas restantes estimadas: `allocated_hours - effective_hours`
- Burn rate actual del proyecto: `hours.burn_rate`
- Semanas necesarias: `horas_restantes / burn_rate`
- Nueva fecha: `hoy + semanas_necesarias * 7`

**Opciones de accion**:
- **Reprogramar**: `update_record.sh "project.task" '[ID]' '{"date_deadline":"YYYY-MM-DD"}'`
- **Cerrar**: `update_record.sh "project.task" '[ID]' '{"stage_id":DONE_STAGE_ID}'`
- **Cancelar**: `update_record.sh "project.task" '[ID]' '{"stage_id":CANCEL_STAGE_ID}'`

---

### 2.2 Items de Checklist Vencidos

Vencido = `date_deadline` pasada Y `state != 'done'`.

**La fecha a comparar es `date_deadline` (prevista), NUNCA `date_done`.** `date_done` es
cuando se completo el item: usarla aqui daria cero vencidos siempre, sin error visible.

`state != 'done'` agrupa tres estados, y NO se tratan igual:

| Estado | Etiqueta | Tratamiento |
|--------|----------|-------------|
| `pending` | Por hacer | Vencido nuestro. Proponer completar o reprogramar |
| `in_progress` | En curso | Vencido nuestro. Proponer completar o reprogramar |
| `waiting` | En espera | **NO proponer cerrarlo.** Ver abajo |

**Items en `waiting`** (DECISION PENDIENTE): un item en espera y vencido esta bloqueado
por algo externo — normalmente el cliente. No es un incumplimiento nuestro y cerrarlo
falsearia el progreso. Hasta que se decida el tratamiento, listarlos aparte como
informativos y NO proponer ninguna escritura sobre ellos.

**Acciones para `pending` / `in_progress`**:
- **Completar**: Si el trabajo ya esta hecho → `{"state":"done","date_done":"YYYY-MM-DD"}`
- **Reprogramar**: Si aun falta trabajo → `{"date_deadline":"YYYY-MM-DD"}`

Completar exige **las dos** claves: `state` a `done` y `date_done` con la fecha real.
Poner solo `state` deja el item sin fecha de cierre y rompe las metricas de velocidad.

---

### 2.3 Bottlenecks

Priorizar los de `risk_score > 5`. Opciones:

| Situacion | Accion recomendada |
|-----------|-------------------|
| Sin responsable | Asignar al recurso mas disponible |
| Bloqueado externamente | Escalar al PM, documentar bloqueo |
| Tarea muy grande | Recomendar dividir en subtareas (accion manual) |
| Sin fecha limite | Asignar deadline (ver 1.1) |
| Progreso 0% con horas | Revisar si las horas son correctas |

---

## 3. Desviacion

### 3.1 Umbrales de Activacion

Se requiere plan de desviacion si:
- Score < 40 Y acceleration_factor > 2.0
- O: T% > 100 Y C% < 80
- O: el consultor lo solicita explicitamente

### 3.2 Formula de Extension

```
semanas_adicionales = (pending_weight / velocity_actual) - weeks_remaining
nueva_fecha_fin = fecha_fin_actual + semanas_adicionales * 7
```

Si `velocity_actual = 0`: usar `burn_rate` y `allocated_hours` pendientes como aproximacion:
```
horas_pendientes = SUM(allocated_hours - effective_hours) para tareas no completadas
semanas_adicionales = horas_pendientes / burn_rate
```

### 3.3 Reduccion de Scope

Criterios para elegir que aplazar a Fase 2 (en orden de prioridad):
1. Tareas **sin iniciar** (`effective_hours = 0`)
2. Tareas con **menor peso** en checklists
3. Tareas **no bloqueantes** para go-live (formaciones avanzadas, integraciones secundarias)
4. Tareas de **desarrollo custom** que no son criticas

**NO aplazar**:
- Tareas de configuracion basica
- Migraciones de datos criticos (clientes, productos)
- Tareas con dependencias downstream

### 3.4 Opciones de Desviacion

Presentar siempre 3 opciones:

| Opcion | Descripcion |
|--------|-------------|
| **A: Extender plazo** | Mantener scope completo, nueva fecha fin |
| **B: Reducir scope** | Mantener fecha, aplazar tareas a Fase 2 |
| **C: Mixto** | Extender parcialmente + aplazar algo de scope |

Para cada opcion calcular:
- Nueva fecha fin estimada
- Tareas afectadas
- Score proyectado (gobernanza se mantiene, rendimiento mejora con mas tiempo)

### 3.5 Comunicacion al Cliente

**Tono**: Transparente, orientado a soluciones, sin blame.

**Estructura de comunicacion**:
1. Situacion actual (datos objetivos)
2. Causas identificadas (sin culpar)
3. Opciones propuestas (con pros/contras)
4. Recomendacion del equipo
5. Proximos pasos concretos

**Evitar**: "el cliente no entrego a tiempo", "la estimacion fue mala"
**Preferir**: "se identificaron requisitos adicionales que impactan el cronograma", "la complejidad tecnica supero la estimacion inicial"
