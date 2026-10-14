# Checklists de Proyecto

## Qué es

El modelo `xx.checklist.item` permite hacer seguimiento granular del progreso de cada tarea. Cada item tiene un tipo, un peso (prioridad) y un estado. El estado ya no es un booleano: tiene cuatro valores.

## Cuándo aplica

- Fase 3 (Pre-proyecto): Creación de checklists desde el análisis
- Fase 4 (Ejecución): Uso diario para marcar progreso
- Fase 4.M (Monitorización): Los checklists alimentan el health score

## Qué debe saber el consultor

### Modelo xx.checklist.item

| Campo | Tipo | Descripción |
|-------|------|-------------|
| name | Char | Nombre descriptivo del item ("Paso") |
| task_id | Many2one → project.task | Tarea a la que pertenece |
| project_id | Many2one → project.project | Proyecto del item |
| type | Selection | Tipo del item (ver tabla de tipos) |
| weight | Integer | Peso/prioridad (1-5) |
| state | Selection | Estado del item (ver tabla de estados) |
| date_done | Date | Fecha en que se completó |
| date_deadline | Date | Fecha límite prevista |
| user_id | Many2one → res.users | Responsable del item |
| sequence | Integer | Orden dentro de la tarea |
| notes | Html | Instrucciones del item |
| has_notes | Boolean | Si el item tiene instrucciones |

El inverso en la tarea es `xx_checklist_ids`.

### Estados (`state`)

Sustituye al antiguo booleano `x_studio_completado`. "Completado" es ahora `state == 'done'`, y "pendiente" es `state != 'done'`, que agrupa tres estados distintos.

| Valor | Etiqueta Odoo | Significado |
|-------|---------------|-------------|
| pending | Por hacer | Sin empezar |
| in_progress | En curso | Alguien lo está trabajando |
| waiting | En espera | Empezado pero bloqueado: falta un dato o una decisión |
| done | Hecho | Terminado, con `date_done` relleno |

### Regla de obligatoriedad

**Todas las tareas DEBEN tener al menos 1 checklist**. Esta regla se verifica en:
- El estándar #4 de la auditoría (tareas con `allocated_hours` ≥ 5)
- El health monitor (penalización de gobernanza)
- El consultant-daily-planner (priorización)

### Pesos

El peso es una mezcla de dificultad y esfuerzo.

| Peso | Significado | Ejemplo |
|------|-------------|---------|
| 1 | Trivial, rápido | Activar un setting |
| 2 | Sencillo | Configurar una secuencia |
| 3 | Normal | Crear campos Studio |
| 4 | Importante | Configurar una app completa |
| 5 | Crítico | Integración o migración |
| null/0 | Sin asignar | Se trata como peso 1 |

### Tipos y quién los ejecuta

Los valores son técnicos (antes eran etiquetas como CONFIG o STUDIO/AUTO).

| Valor | Etiqueta Odoo | Ejecuta | Skill |
|-------|---------------|---------|-------|
| config | Configuración | Consultor via odoo-pilot | odoo-configurator |
| studio | Studio / automatización | Consultor via odoo-studio | odoo-configurator |
| development | Desarrollo | Equipo técnico | dev-assessment → desarrollo manual |
| migration | Migración | Consultor | importation-data-assistant |
| test | Pruebas | Consultor | odoo-flow-tester |
| training | Formación | Consultor | training-planner |
| analysis | Análisis | Consultor | Análisis adicional (manual) |
| report | Informes y formatos | Consultor | report-docx-template / report-designer-author |

Solo `config` y `studio` son procesables por odoo-configurator.

### Workflow de los checklists

1. Se crean durante la fase de pre-proyecto (desde el análisis)
2. El consultor los ejecuta uno a uno durante la fase de ejecución
3. El consultor marca `state = 'done'` y rellena `date_done` manualmente
4. El health monitor calcula el progreso basándose en los pesos completados vs totales

### Cálculo de progreso

```
progreso = SUM(weight de items con state = 'done') / SUM(weight de todos los items) * 100
```

Items con peso null/0 se tratan como peso 1.

## Qué debe hacer el consultor

1. Verificar que todas las tareas tienen checklists
2. Marcar como `done` solo cuando el item está realmente verificado
3. Usar `waiting` cuando un item esté bloqueado esperando al cliente, en vez de dejarlo en `pending`
4. Añadir nuevos items si surgen necesidades durante la ejecución
5. La tarea de reuniones/formaciones debe tener un checklist por cada reunión o formación planificada y realizada. De esta forma llevamos control de todas las sesiones realizadas con el cliente.

## Skills relacionados

- `odoo-configurator`: Ejecuta items `config` y `studio`
- `project-health-monitor`: Usa los pesos para calcular el health score
- `project-health-fixer`: Propone correcciones de gobernanza (crear checklists faltantes)
- `consultant-daily-planner`: Prioriza items pendientes en la agenda

## Errores comunes

1. **No asignar pesos**: Todos los items quedan con peso implícito 1, distorsionando el progreso
2. **Marcar como `done` sin verificar**: El checklist pierde su función de control de calidad
3. **Items `development`/`migration` ejecutados por el consultor**: Deben derivarse al equipo técnico o al skill correspondiente
4. **No actualizar checklists**: Si surgen nuevas necesidades, añadir items nuevos inmediatamente
5. **No usar `waiting`**: Un item bloqueado que se queda en `pending` es invisible como bloqueo

## Nota sobre la migración (sept-2026)

`date_deadline` está vacío en los 30.905 registros migrados: la fecha antigua (`x_studio_fecha_fin`) aterrizó en `date_done`, incluso en items no terminados. Hasta que se pueble `date_deadline`, `date_done` no es fiable como "fecha en que se completó".
