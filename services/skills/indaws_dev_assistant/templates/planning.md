# Template: Planificación Semanal

Formato de salida para la planificación semanal de un empleado.

## Tareas de la Semana

```
📅 Planificación semana [DD/MM/YYYY] - [DD/MM/YYYY]

| Tarea | Proyecto | Horas | Prioridad | Estado |
|-------|----------|-------|-----------|--------|
| [Nombre tarea] | [Proyecto] | HH:MM | Alta/Media/Baja | ✅/⏳ |

**Total:** HH:MM planificadas | HH:MM completadas | HH:MM pendientes
```

### Columnas

| Columna | Descripción |
|---------|-------------|
| **Tarea** | Nombre de la tarea (de `x_studio_task_id` o `name` del slot) |
| **Proyecto** | Proyecto asociado a la tarea |
| **Horas** | `allocated_hours` en formato HH:MM |
| **Prioridad** | `x_studio_task_priority` (Alta/Media/Baja) |
| **Estado** | ✅ si `x_studio_done = true`, ⏳ si `false` |

## Tareas Pendientes (para "¿Qué hago hoy?")

La planificación se hace por semanas. Cuando pregunten "¿Qué tengo hoy?":
- Mostrar tareas de la semana actual **no completadas**
- Ordenar por prioridad descendente

```
📋 Tareas pendientes (semana actual)

| Tarea | Proyecto | Horas | Prioridad |
|-------|----------|-------|-----------|
| [Nombre] | [Proyecto] | HH:MM | Alta |
| [Nombre] | [Proyecto] | HH:MM | Media |

Tienes HH:MM de trabajo pendiente esta semana.
```

## Sin Tareas Planificadas

```
No tienes tareas planificadas para esta semana.
```

## Notas sobre Estados

| Estado slot | Significado |
|-------------|-------------|
| `state = 'published'` | Planificación confirmada |
| `state = 'draft'` | Planificación en borrador (puede cambiar) |

## Prioridades

Los valores típicos de `x_studio_task_priority`:
- `0` = Normal
- `1` = Alta

(Verificar valores exactos en Studio si difieren)
