# Seguimiento del Proyecto

## Qué es

El conjunto de herramientas de monitorización continua que permiten evaluar la salud del proyecto, generar actualizaciones para el cliente, corregir problemas y planificar el trabajo diario.

## Cuándo aplica

Fase 4.M (Monitorización): Durante toda la fase de ejecución, de forma continua.

## Qué debe saber el consultor

### Cadena de dependencia

```
project-weekly-update (incluye project-health-monitor internamente) → project-health-fixer
                       ↘ consultant-daily-planner
```

Los skills de seguimiento tienen dependencias entre sí:
- `project-weekly-update` ejecuta `project-health-monitor` internamente — no es necesario ejecutarlo por separado antes
- `project-health-fixer` requiere la salida de `project-health-monitor`
- `consultant-daily-planner` usa datos de todos los anteriores

### 1. Health Monitor

**Qué hace**: Calcula el Health Score (0-100) analizando timeline, checklists ponderados y consumo de horas.

**4 estados del proyecto**:

| Score | Estado | Significado |
|-------|--------|-------------|
| 80-100 | SALUDABLE | El proyecto va según lo previsto |
| 60-79 | EN RIESGO | Hay señales de alerta que requieren atención |
| 40-59 | CRÍTICO | Se necesitan acciones correctivas inmediatas |
| 0-39 | EMERGENCIA | El proyecto requiere intervención urgente |

**6 escenarios de diagnóstico**:
- **ON_TRACK**: Todo según lo previsto
- **LOW_PRODUCTIVITY**: Checklists avanzan lento respecto al timeline
- **INEFFICIENCY**: Se consumen muchas horas pero poco avance en checklists
- **BLOCKED**: Tareas vencidas sin acción reciente
- **OVERCOST**: Consumo de horas supera el presupuesto
- **OVERDUE**: El proyecto ha sobrepasado su fecha límite

**Métricas clave**: Burn rate, velocidad puntos/semana, factor de aceleración, EAC (Estimate at Completion).

### 2. Weekly Update

**Qué hace**: Genera actualización semanal HTML para el cliente y crea registro `project.update`.

**REGLA CLAVE**: Las horas son datos INTERNOS. Los timesheets se leen solo para entender qué trabajo se ha hecho y redactar la narrativa. NUNCA se muestran al cliente (ni tabla de horas, ni desglose, ni cifras).

**Contenido del update** (4 secciones):
1. Trabajo Realizado (narrativa funcional)
2. Progreso del Proyecto (cualitativo, sin score numérico)
3. Estado de Tareas (resumen alto nivel)
4. Próximos Pasos (acciones planificadas)

**Frecuencia**: TODOS LOS VIERNES, sin excepción. Práctica OBLIGATORIA. El consultor debe ejecutar `project-weekly-update` cada viernes para cada proyecto activo.

### 3. Health Fixer

**Qué hace**: Ayuda al consultor a corregir problemas detectados por el health monitor.

**3 categorías de issues**:

| Categoría | Tipo de problema | Acción |
|-----------|-----------------|--------|
| **Gobernanza** | Tareas sin fecha, desajuste horas, sin checklists | Corrección directa en Odoo |
| **Progreso** | Tareas/items vencidos, bottlenecks | Requiere decisión del consultor |
| **Estratégico** | Necesidad de desviación (cambio scope/plazo) | Requiere replanteo con PM |

**Score potencial**: Calcula cuánto mejoraría el score si se corrigen los problemas de gobernanza.

### 4. Daily Planner

**Qué hace**: Analiza todos los proyectos del consultor y genera una agenda diaria priorizada.

**3 niveles de prioridad**:
- **URGENTE** (hoy): Requests vencidas >3 días, tareas vencidas con checklists incompletos, tickets >24h, reuniones esta semana sin preparar
- **IMPORTANTE** (si hay tiempo): Health score <60, weekly updates >7 días, requests con deadline ≤10 días, formaciones ≤2 días
- **PLANIFICADO** (esta semana): Validaciones pendientes, checklists de la semana, dev assessments, go-live ≤30 días

**Patrones por día**: Viernes = weekly-update URGENTE (obligatorio para todos los proyectos activos).

## Qué debe hacer el consultor

1. Generar `project-weekly-update` TODOS LOS VIERNES (OBLIGATORIO) — el skill ya incluye el health monitor
2. Si health score < 60, ejecutar `project-health-fixer`
3. Usar `consultant-daily-planner` al inicio del día para priorizar

## Skills relacionados

- `project-weekly-update`: Genera actualización semanal para el cliente (incluye health monitor internamente)
- `project-health-monitor`: Evaluación de salud standalone (para diagnóstico puntual fuera del viernes)
- `project-health-fixer`: Corrige problemas de gobernanza
- `consultant-daily-planner`: Agenda diaria priorizada

## Errores comunes

1. **No enviar el weekly update los viernes**: Es obligatorio para todos los proyectos activos
2. **Mostrar horas al cliente en el weekly**: Las horas son datos INTERNOS, NUNCA se comparten
3. **Corregir sin diagnóstico**: Ejecutar health-fixer requiere health-monitor como prerequisito
4. **Ignorar el score potencial**: Si la gobernanza puede corregirse fácil, hacerlo primero
