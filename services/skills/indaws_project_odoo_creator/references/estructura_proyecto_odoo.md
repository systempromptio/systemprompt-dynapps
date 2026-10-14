# Estructura del Proyecto en Odoo

Referencia de campos, mapeos y reglas para la creacion de tareas en `project.project`.

---

## Modelo: project.task

### Campos obligatorios

| Campo | Tipo | Descripcion | Ejemplo |
|-------|------|-------------|---------|
| `name` | Char | Nombre de la tarea | "Ventas" |
| `project_id` | Many2one | ID del proyecto | 245 |
| `allocated_hours` | Float | Horas planificadas | 24.0 |
| `planned_date_begin` | Datetime | Fecha inicio | "2026-04-01 08:00:00" |
| `date_deadline` | Datetime | Fecha limite | "2026-04-15 18:00:00" |
| `sequence` | Integer | Orden de la tarea | 10 |
| `user_ids` | Many2many | Responsables | `[[5]]` para dejar sin asignar (OBLIGATORIO) |

**IMPORTANTE Odoo 19**: El campo `date_start` NO existe en `project.task`. Usar `planned_date_begin`. El campo `date_deadline` es Datetime (no Date), requiere hora.

### Campos opcionales relevantes

| Campo | Tipo | Descripcion | Notas |
|-------|------|-------------|-------|
| `description` | Html | Descripcion de la tarea | Resumen de lo que incluye |
| `tag_ids` | Many2many | Etiquetas | Modulo o categoria |
| `parent_id` | Many2one | Tarea padre | Para subtareas (no usar normalmente) |
| `stage_id` | Many2one | Etapa | Se asigna automaticamente |

### Formato de fechas

La API de Odoo 19 espera Datetime en ambos campos:

```
planned_date_begin (Datetime):  "YYYY-MM-DD HH:MM:SS"  → "2026-04-01 08:00:00"
date_deadline (Datetime):       "YYYY-MM-DD HH:MM:SS"  → "2026-04-15 18:00:00"
```

**Conversion desde el plan** (formato DD/MM/YYYY):
- "01/04/2026" → "2026-04-01"
- Para `planned_date_begin`, anadir " 08:00:00" (inicio jornada)
- Para `date_deadline`, anadir " 18:00:00" (fin jornada)

### Asignacion de responsables

Odoo auto-asigna al usuario creador si no se especifica `user_ids`. Para dejar la tarea sin asignar, pasar SIEMPRE:

```json
"user_ids": [[5]]
```

El comando `[5]` en Many2many significa "vaciar la relacion" (unlink all).

---

## Modelo: xx.checklist.item

Checklist ponderado vinculado a tareas del proyecto. Cada item representa una accion concreta de la implantacion. El inverso en la tarea es `xx_checklist_ids`.

### Campos

| Campo | Tipo | Descripcion | Requerido |
|-------|------|-------------|-----------|
| `name` | Char | Nombre del item ("Paso") | SI |
| `task_id` | Many2one (project.task) | Tarea a la que pertenece | SI |
| `project_id` | Many2one (project.project) | Proyecto del item | SI |
| `weight` | Integer | Peso del item (para Health Score) | SI (default: 1) |
| `state` | Selection | Estado del item | NO (default: `pending`) |
| `type` | Selection | Tipo de accion (lista CERRADA de 8 valores) | NO |
| `sequence` | Integer | Orden dentro de la tarea | NO |
| `user_id` | Many2one (res.users) | Responsable | NO |
| `date_deadline` | Date | Fecha limite prevista | NO |
| `date_done` | Date | Fecha en que se completo | NO |
| `notes` | Html | Instrucciones del item | NO |
| `has_notes` | Boolean | Calculado sobre `notes` | NO (no se escribe) |

### Valores de state

Sustituye al antiguo booleano `x_studio_completado`. Un item recien creado va a `pending`.

| Valor | Etiqueta Odoo | Cuando |
|-------|---------------|--------|
| `pending` | Por hacer | **Default al crear** |
| `in_progress` | En curso | Alguien lo esta trabajando |
| `waiting` | En espera | Bloqueado esperando algo externo |
| `done` | Hecho | Terminado, con `date_done` relleno |

### Valores de type

`type` es una **lista cerrada** de ocho valores tecnicos en minusculas. No admite texto libre: un valor fuera de la lista es rechazado por Odoo.

| Prefijo del plan | Valor `type` | Descripcion |
|-----------------|--------------|-------------|
| [CONFIG] | `config` | Configuracion estandar |
| [DEV] | `development` | Desarrollo a medida |
| [MIGR] | `migration` | Migracion de datos |
| [TEST] | `test` | Pruebas y UAT |
| [TRAIN] | `training` | Formacion |
| [ANLS] | `analysis` | Analisis |
| [STUDIO/AUTO] | `studio` | Automatizacion / Studio |
| [DOC] | **DECISION PENDIENTE** | Ver nota abajo |

El octavo valor de la selection, `report` (Informes y formatos), **no tiene prefijo en el plan**: en produccion agrupa plantillas PDF/DOCX ("[DOCX] Informe Factura", "PDF del pedido de compra"), no documentacion funcional.

**[DOC] — decision pendiente**: el plan usa este prefijo para "Generacion de documentacion funcional". El modelo viejo no tenia valor `DOC` y la skill lo mapeaba a `CONFIG`. En la migracion, los 17 items con `[DOC]` en el nombre quedaron como `development`. Hasta que se decida, NO crear items con prefijo [DOC]: pararse y preguntar al consultor.

**NOTA**: Si el item del plan no tiene prefijo de tipo, usar `config` como valor por defecto.

---

## Modelo: project.tags

Etiquetas del proyecto para clasificar tareas por modulo.

### Campos

| Campo | Tipo | Descripcion |
|-------|------|-------------|
| `name` | Char | Nombre de la etiqueta |
| `color` | Integer | Color (0-11) |

### Reglas para etiquetas

- Buscar si la etiqueta ya existe antes de crearla: `search([('name', '=', 'Ventas')])`
- Si no existe, crearla con un color aleatorio (1-11)
- Usar nombres de modulo/categoria como etiquetas: "CRM", "Ventas", "Compras", etc.
- Para tareas genericas: "General", "Soporte", "Reuniones"

---

## Mapeo: Plan de Proyecto → Tareas Odoo

### Tareas agrupadas (por modulo)

Del plan:
```
| # | Nombre de Tarea | Modulo/Categoria | Peso | Horas | Fecha Inicio | Fecha Fin |
| 1 | Configuraciones Generales | General | 12 | 12h | 01/04/2026 | 07/04/2026 |
| 2 | CRM | CRM | 16 | 16h | 08/04/2026 | 18/04/2026 |
```

A Odoo:
```json
{
  "name": "Configuraciones Generales",
  "project_id": 245,
  "allocated_hours": 12.0,
  "planned_date_begin": "2026-04-01 08:00:00",
  "date_deadline": "2026-04-07 18:00:00",
  "sequence": 10,
  "user_ids": [[5]],
  "tag_ids": [[6, 0, [tag_id_general]]]
}
```

### Tareas detalladas

Del plan:
```
| 8 | [DEV] Tarificacion Dinamica | Ventas | 25 | 40h | 15/05/2026 | 30/05/2026 |
```

A Odoo:
```json
{
  "name": "[DEV] Tarificacion Dinamica",
  "project_id": 245,
  "allocated_hours": 40.0,
  "planned_date_begin": "2026-05-15 08:00:00",
  "date_deadline": "2026-05-30 18:00:00",
  "sequence": 80,
  "user_ids": [[5]],
  "tag_ids": [[6, 0, [tag_id_ventas]]]
}
```

### Checklists

Del apartado "Detalle de Tareas" del plan:
```
### Tarea: CRM (16h)
- [CONFIG] Configurar etapas del pipeline
- [CONFIG] Definir equipos de ventas
- [STUDIO/AUTO] Campos personalizados de lead
- [CONFIG] Reglas de asignacion automatica
```

A Odoo (un registro `xx.checklist.item` por item):
```json
{
  "name": "Configurar etapas del pipeline",
  "task_id": task_id,
  "project_id": project_id,
  "weight": 1,
  "sequence": 10,
  "state": "pending",
  "type": "config"
}
```

**NOTA**: El `name` NO incluye el prefijo de tipo. El prefijo se usa para determinar `type`.

---

## Secuencia de tareas

Las tareas se crean con `sequence` incrementando de 10 en 10:

| Tarea # | sequence |
|---------|----------|
| 1 | 10 |
| 2 | 20 |
| 3 | 30 |
| ... | ... |

Esto permite insertar tareas intermedias en el futuro si es necesario.

---

## Modelo: project.task.type (Etapas)

Las etapas de tarea ya existen en Odoo. Solo hay que vincular el proyecto a cada etapa usando el campo Many2many `project_ids`.

### Etapas estandar

| ID | Nombre | Sequence |
|----|--------|----------|
| 10 | Sin asignar | 0 |
| 574 | Asignado | 1 |
| 737 | Tareas Base | 2 |
| 1008 | A planificar | 4 |
| 11 | Desarrollo | 5 |
| 23 | Testeo | 6 |
| 26 | Testeo cliente | 7 |
| 25 | Pasar a produccion | 8 |
| 35 | En produccion | 9 |
| 12 | Hecho | 10 |
| 37 | Cancelado | 11 |

### Vincular etapas al proyecto

Para cada etapa, usar el comando `[4, id]` (link) en Many2many:

```python
write('project.task.type', [stage_id], {'project_ids': [[4, project_id]]})
```

- `[4, id]` = link (anadir sin eliminar los existentes)
- NO usar `[6, 0, ids]` que reemplazaria todos los proyectos vinculados a la etapa

### Asignacion de etapa en tareas

Al crear tareas, asignar siempre `stage_id: 10` (Sin asignar) para que todas las tareas empiecen en la primera etapa.

---

## Verificaciones post-creacion

Despues de crear todas las tareas:

1. **Contar tareas**: `search_count([('project_id', '=', project_id)])` debe coincidir con el total del plan
2. **Sumar horas**: Sumar `allocated_hours` de todas las tareas debe coincidir con el total del plan (tolerancia ±10%)
3. **Verificar checklists**: Cada tarea debe tener al menos 1 item de checklist
4. **Verificar fechas**: Ninguna tarea debe tener `planned_date_begin` posterior a `date_deadline`
5. **Verificar asignacion**: Ninguna tarea debe tener `user_ids` asignados (deben estar vacias)
