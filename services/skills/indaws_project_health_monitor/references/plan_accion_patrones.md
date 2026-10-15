# Patrones de Plan de Acción

Documento de referencia con los escenarios de diagnóstico, algoritmo de detección de bottlenecks y acciones recomendadas por escenario.

---

## 1. Escenarios de Diagnóstico

Basados en la relación entre las tres métricas core: Timeline (T%), Checklists (C%) y Horas (H%).

### Definiciones de Umbrales para Comparación

```
"aproximado" (≈): diferencia menor a 15 puntos porcentuales
"muy por debajo" (<<): diferencia mayor a 20 puntos porcentuales
"muy por encima" (>>): diferencia mayor a 20 puntos porcentuales
```

---

### 1.1 ON_TRACK - Proyecto en Buen Camino

**Condición**: Health Score >= 80

**Diagnóstico**: El proyecto avanza al ritmo esperado. Los checklists se completan proporcionalmente al tiempo transcurrido y el consumo de horas es coherente.

**Acciones de mantenimiento**:
- Mantener el ritmo actual de trabajo
- Vigilar items de checklist con peso alto (4-5) que aún estén pendientes
- Revisar si hay tareas próximas a vencer que puedan complicarse
- Documentar buenas prácticas del proyecto para replicar en futuros

**Recomendaciones**:
- Reunión de seguimiento estándar (quincenal)
- No se requieren acciones correctivas

---

### 1.2 LOW_PRODUCTIVITY - Baja Productividad

**Condición**: C% << T% (retraso > 20pp) Y H% ≈ T% (horas se consumen al ritmo esperado)

**Diagnóstico**: Se están dedicando horas al proyecto pero no se traducen en avance real de checklists. Posibles causas:
- Tareas mal estimadas (más complejas de lo previsto)
- Asignaciones incorrectas (personas sin el conocimiento necesario)
- Retrabajo frecuente (se hacen cosas que luego hay que rehacer)
- Bloqueos parciales (se trabaja pero en tareas que no avanzan el checklist)

**Acciones inmediatas** (esta semana):
1. Revisar las últimas entradas de timesheet: ¿en qué se están gastando las horas?
2. Identificar tareas con muchas horas pero pocos items completados
3. Reunión con el equipo para entender bloqueos no reportados

**Acciones a corto plazo** (próximas 2 semanas):
1. Reasignar tareas problemáticas a personas con más experiencia
2. Dividir tareas grandes en subtareas más pequeñas y controlables
3. Establecer daily standups breves (15 min) para detectar bloqueos temprano
4. Revisar si las estimaciones originales eran realistas

**Acciones estratégicas**:
1. Considerar refuerzo temporal del equipo
2. Evaluar si el alcance del proyecto necesita recorte
3. Revisar el proceso de asignación de tareas

---

### 1.3 INEFFICIENCY - Ineficiencia / Scope Creep

**Condición**: C% << T% (retraso > 20pp) Y H% >> T% (sobrecosto > 20pp)

**Diagnóstico**: Se consumen más horas de las esperadas y el progreso de checklists es bajo. Es el peor escenario. Posibles causas:
- Scope creep (se añaden tareas no planificadas)
- Problemas técnicos graves que consumen horas sin resolver el fondo
- Falta de dirección clara del proyecto
- Requisitos cambiantes del cliente

**Acciones inmediatas** (esta semana):
1. STOP: Reunión urgente con el Project Manager
2. Auditar las últimas 2 semanas de timesheets: ¿dónde fueron las horas?
3. Identificar trabajo no planificado que se ha realizado
4. Congelar cualquier solicitud nueva del cliente hasta resolver la situación

**Acciones a corto plazo** (próximas 2 semanas):
1. Separar claramente trabajo planificado vs. no planificado
2. Si hay scope creep: documentar las desviaciones y presentar al cliente con impacto en horas/plazos
3. Recalcular el EAC (Estimate at Completion) y presentar proyección realista
4. Priorizar brutalmente: ¿qué items del checklist son imprescindibles para el go-live?

**Acciones estratégicas**:
1. Proponer ampliación de bolsa de horas al cliente (con justificación)
2. Negociar recorte de alcance formal
3. Revisar la gestión del cambio (todo cambio debe ser aprobado y estimado)
4. Considerar cambio de enfoque: entregar un MVP primero, iterar después

---

### 1.4 BLOCKED - Proyecto Bloqueado

**Condición**: C% << T% (retraso > 20pp) Y H% << T% (pocas horas, retraso > 20pp)

**Diagnóstico**: Ni se avanza en checklists ni se consumen horas. El proyecto está efectivamente parado. Posibles causas:
- Bloqueo por parte del cliente (no proporciona información, no valida)
- Equipo reasignado a otros proyectos
- Dependencia externa no resuelta (proveedor, infraestructura)
- Falta de prioridad del proyecto

**Acciones inmediatas** (esta semana):
1. Identificar la causa raíz del bloqueo
2. Escalar al responsable del proyecto / cliente
3. Enviar comunicación formal al cliente indicando el riesgo de retraso
4. Si es bloqueo del cliente: solicitar fecha compromiso de desbloqueo

**Acciones a corto plazo** (próximas 2 semanas):
1. Definir un plan de desbloqueo con fechas concretas
2. Si el bloqueo es interno: reasignar recursos inmediatamente
3. Si el bloqueo es externo: documentar y proponer plan alternativo
4. Recalcular el cronograma asumiendo el tiempo perdido

**Acciones estratégicas**:
1. Proponer nueva fecha de entrega ajustada
2. Evaluar si el proyecto necesita ser pausado formalmente
3. Activar protocolo de escalado si el bloqueo persiste > 2 semanas
4. Documentar el bloqueo para protección contractual

---

### 1.5 OVERCOST - Sobrecosto con Progreso Correcto

**Condición**: C% ≈ T% (progreso ok) Y H% >> T% (sobrecosto > 20pp)

**Diagnóstico**: El proyecto avanza bien en checklists pero consume más horas de las presupuestadas. El trabajo se hace pero cuesta más. Posibles causas:
- Estimaciones originales demasiado optimistas
- Complejidad técnica subestimada
- Equipo junior que necesita más horas de las previstas
- Horas de formación/aprendizaje no presupuestadas

**Acciones inmediatas** (esta semana):
1. Calcular EAC: ¿cuántas horas se necesitarán realmente para terminar?
2. Identificar las tareas que más se han desviado en horas
3. Determinar si el sobrecosto continuará al mismo ritmo

**Acciones a corto plazo** (próximas 2 semanas):
1. Optimizar las tareas restantes: ¿se pueden hacer de forma más eficiente?
2. Considerar uso de herramientas/scripts/automatización para acelerar
3. Si hay tareas repetitivas: crear templates o procedimientos reutilizables
4. Presentar proyección al cliente de forma transparente

**Acciones estratégicas**:
1. Si el EAC supera el presupuesto en > 20%: negociar ampliación
2. Priorizar: ¿se puede entregar el alcance completo con las horas restantes?
3. Documentar las causas del sobrecosto para futuras estimaciones
4. Revisar el factor de contingencia usado en la estimación original

---

### 1.6 OVERDUE - Proyecto Vencido

**Condición**: T% > 100% (fecha fin superada)

**Diagnóstico**: El proyecto ha superado su fecha de entrega planificada. La gravedad depende de cuánto queda por hacer (C%) y cuántas horas quedan (H%).

**Sub-escenarios**:
- C% > 90%: "Casi terminado, pequeño retraso" → Menor gravedad
- C% 60-90%: "Retraso significativo" → Gravedad media
- C% < 60%: "Proyecto con retraso grave" → Máxima gravedad

**Acciones inmediatas** (esta semana):
1. Calcular lo que falta: items pendientes, horas estimadas restantes
2. Comunicación formal al cliente sobre el estado
3. Proponer nueva fecha de entrega realista
4. Priorizar: ¿qué es imprescindible vs. qué puede posponerse?

**Acciones a corto plazo** (próximas 2 semanas):
1. Modo "sprint final": foco absoluto en completar items críticos
2. Recortar alcance si es necesario (diferir a fase 2)
3. Asignar recursos adicionales si hay disponibilidad
4. Daily standups hasta cerrar

**Acciones estratégicas**:
1. Definir formalmente una "Fase 2" con lo que quede fuera del alcance
2. Documentar las causas del retraso (lecciones aprendidas)
3. Ajustar contractualmente si hay penalizaciones
4. Revisar el proceso de planificación para futuros proyectos

---

## 2. Algoritmo de Detección de Bottlenecks

Identificar las tareas que representan mayor riesgo para el proyecto.

### 2.1 Cálculo del Risk Score por Tarea

```
Task_Risk = (peso_pendiente / peso_total) * peso_promedio_items * factor_vencimiento
```

Donde:
- `peso_pendiente` = suma de pesos de items NO completados de esa tarea
- `peso_total` = suma de pesos de TODOS los items de esa tarea
- `peso_promedio_items` = peso_total / count_items (refleja importancia de la tarea)
- `factor_vencimiento`:
  - Sin fecha límite → 1.0
  - Fecha límite > 2 semanas → 1.0
  - Fecha límite 1-2 semanas → 1.5
  - Fecha límite < 1 semana → 2.0
  - Fecha límite pasada → 3.0

### 2.2 Clasificación de Bottlenecks

Ordenar tareas por `Task_Risk` descendente. Las primeras son los bottlenecks:

| Posición | Etiqueta | Acción |
|----------|----------|--------|
| Top 1-2 | **BOTTLENECK CRÍTICO** | Acción inmediata, reasignar si es necesario |
| Top 3-5 | **BOTTLENECK ALTO** | Vigilancia estrecha, plan de contingencia |
| Resto | Normal | Seguimiento estándar |

### 2.3 Información a Incluir por Bottleneck

Para cada bottleneck identificado, reportar:
- Nombre de la tarea
- Items pendientes (cantidad y peso total)
- Item de mayor peso pendiente
- Fecha límite (y días de retraso si aplica)
- Horas consumidas vs. asignadas
- Recomendación específica

---

## 3. Cálculo del Ritmo Requerido

### 3.1 Factor de Aceleración

```
semanas_restantes = (fecha_fin - fecha_referencia) / 7
peso_restante = peso_total - peso_completado
velocidad_actual = peso_completado / semanas_transcurridas
velocidad_requerida = peso_restante / semanas_restantes
factor_aceleracion = velocidad_requerida / velocidad_actual
```

### 3.2 Interpretación y Recomendación

| Factor | Estado | Recomendación |
|--------|--------|---------------|
| <= 1.0 | Ritmo suficiente | Mantener ritmo actual |
| 1.0-1.3 | Ligera aceleración | Optimizar procesos, reducir reuniones no esenciales |
| 1.3-1.5 | Aceleración moderada | Añadir recursos parciales, priorizar items de alto peso |
| 1.5-2.0 | Aceleración significativa | Añadir recursos, recortar alcance de bajo peso |
| > 2.0 | Inviable sin cambios | Negociar extensión de plazo o recorte de alcance formal |

---

## 4. Selección del Escenario

Algoritmo para determinar qué escenario aplica:

```
1. Si T% > 100 → OVERDUE
2. Si S >= 80 → ON_TRACK
3. Calcular deltas:
   delta_C = C% - T%    (positivo = adelantado, negativo = retrasado)
   delta_H = H% - T%    (positivo = sobrecosto, negativo = bajo consumo)

4. Si delta_C < -20 Y abs(delta_H) <= 15 → LOW_PRODUCTIVITY
5. Si delta_C < -20 Y delta_H > 20 → INEFFICIENCY
6. Si delta_C < -20 Y delta_H < -20 → BLOCKED
7. Si abs(delta_C) <= 15 Y delta_H > 20 → OVERCOST
8. Else → Evaluar caso por caso (puede ser combinación)
```

**Nota**: Si faltan checklists, usar solo T% y H%. Si faltan timesheets, usar solo T% y C%. El escenario se adapta a los datos disponibles.
