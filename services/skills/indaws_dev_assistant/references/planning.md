# Planificación Semanal

Documentación técnica para consultas de planificación en `planning.slot`.

## Modelos Odoo

### planning.slot (Slot de planificación)

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `start_datetime` | Datetime | Inicio del slot |
| `end_datetime` | Datetime | Fin del slot |
| `employee_id` | Many2one | Empleado asignado |
| `allocated_hours` | Float | Horas asignadas (decimal) |
| `state` | Selection | 'draft' / 'published' |
| `name` | Text | Nota/descripción del slot |

### Campos Studio (personalizados)

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `x_studio_task_id` | Many2one | Tarea vinculada (project.task) |
| `x_studio_done` | Boolean | Si la tarea está completada |
| `x_studio_task_priority` | Selection | Prioridad de la tarea |

## Cálculos

### Formato de Horas

**OBLIGATORIO**: Siempre mostrar horas en formato **HH:MM**, nunca en decimales.

```
horas = parte_entera
minutos = (parte_decimal * 60) redondeado
Ejemplo: 2.75h → 02:45
```

### Semana Laboral

- **Inicio**: Lunes
- **Fin**: Domingo
- **Días laborables**: Lunes a Viernes (5 días)

```
HOY = fecha actual
DIA_SEMANA = día de la semana (1=lunes, 7=domingo)
LUNES = HOY - (DIA_SEMANA - 1) días
DOMINGO = LUNES + 6 días
```

---

## Consultas

**Script principal**: `scripts/get_planning_slots.sh`
**Template**: `templates/planning.md`

### Tareas de la semana

```bash
./scripts/get_planning_slots.sh <employee_id> <week_start>
```

### Tareas pendientes ("¿Qué hago hoy?")

```bash
./scripts/get_planning_slots.sh <employee_id> <week_start> true
```

### Obtener nombre de tarea vinculada

Si `x_studio_task_id` tiene valor, consultar `project.task`:

```bash
./scripts/get_task_details.sh <task_id>
```

---

## Notas

- `state = 'published'` indica planificación confirmada
- `state = 'draft'` indica planificación en borrador (puede cambiar)
- La prioridad suele ser: 0=Normal, 1=Alta
- Si la tarea viene de `x_studio_task_id`, obtener el nombre real de `project.task`
- La planificación se hace por semanas, no por días
