# Checklist Workflow

> Maquina de estados del modelo `xx.checklist.item`, reglas de transicion y tipos de items.

---

## Modelo: xx.checklist.item

### Campos principales

| Campo | Tipo | Descripcion |
|-------|------|-------------|
| `name` | Char | Nombre/descripcion del item ("Paso") |
| `task_id` | Many2one → project.task | Tarea a la que pertenece |
| `project_id` | Many2one → project.project | Proyecto del item |
| `type` | Selection | Tipo de item (ver tabla de tipos) |
| `weight` | Integer | Peso/puntos del item para calculo de progreso |
| `state` | Selection | Estado del item (ver maquina de estados) |
| `date_done` | Date | Fecha en que se completo |
| `date_deadline` | Date | Fecha limite prevista |
| `user_id` | Many2one → res.users | Responsable del item |
| `sequence` | Integer | Orden dentro de la tarea |
| `notes` | Html | Instrucciones del item |
| `has_notes` | Boolean | Si el item tiene instrucciones |

El inverso en la tarea es `xx_checklist_ids`.

---

## Maquina de estados (`state`)

Sustituye al booleano `x_studio_completado` del modelo anterior. Ya no hay dos
situaciones (hecho / no hecho) sino cuatro, y el avance de un item es un
recorrido entre ellas, no un interruptor.

| Valor | Etiqueta Odoo | Significado |
|-------|---------------|-------------|
| `pending` | Por hacer | Sin empezar. Estado inicial de todo item |
| `in_progress` | En curso | Alguien lo esta trabajando ahora |
| `waiting` | En espera | Empezado pero bloqueado: falta un dato, una decision o una respuesta del cliente |
| `done` | Hecho | Terminado. Se acompana de `date_done` |

### Transiciones validas

```
pending ──────> in_progress ──────> done
   │                 │  ^             ^
   │                 v  │             │
   └──────────> waiting ┘─────────────┘
```

- `pending → in_progress`: se empieza a trabajar el item
- `pending → waiting`: no se puede ni empezar porque falta algo de fuera
- `in_progress → waiting`: se empezo y aparecio un bloqueo
- `waiting → in_progress`: se desbloqueo y se retoma
- `in_progress → done`: terminado (rellenar `date_done`)
- `waiting → done`: se resolvio el bloqueo y el item quedo cerrado

"Completado" **ya no es un booleano**: es `state == 'done'`. Cualquier filtro
que antes dijera `x_studio_completado = false` ahora es `state != 'done'`, e
incluye los tres estados no terminales (`pending`, `in_progress`, `waiting`).

### Equivalencia con el modelo anterior

| Antes | Ahora |
|-------|-------|
| `x_studio_completado = true` | `state = 'done'` |
| `x_studio_completado = false` | `state != 'done'` (pending + in_progress + waiting) |
| `x_studio_fecha_fin` | `date_done` (realizada) o `date_deadline` (prevista) |
| `x_studio_estado_id` (Many2one) | `state` (Selection) |
| `x_studio_hito` | sin equivalente |

---

## Tipos de Items (`type`)

Los valores son tecnicos (antes eran etiquetas). El filtro de procesables
compara en mayusculas, de modo que `config` y `studio` siguen entrando y el
resto queda fuera.

| Valor | Etiqueta Odoo | Procesable por este skill | Accion |
|-------|---------------|--------------------------|--------|
| `config` | Configuracion | SI | Ejecutar configuracion via odoo-pilot |
| `studio` | Studio / automatizacion | SI | Crear campos/acciones via odoo-studio |
| `analysis` | Analisis | NO | Trabajo de analisis — ignorar |
| `migration` | Migracion | NO | Requiere migracion de datos — ignorar |
| `development` | Desarrollo | NO | Requiere desarrollo — ignorar |
| `report` | Informes y formatos | NO | Plantillas/informes — ignorar |
| `test` | Pruebas | NO | Item de testing — ignorar |
| `training` | Formacion | NO | Formacion — ignorar |
| *(vacio/otro)* | — | NO | Tipo no reconocido — ignorar |

---

## Reglas de Transicion

### Lo que este skill PUEDE hacer:
- Leer todos los items del checklist
- Filtrar por `type` y por `state`
- Corregir `type` si es incorrecto (ej: un item etiquetado como `config` que realmente es `studio`)

### Lo que este skill NUNCA hace:
- Cambiar `state` — ninguna transicion, ni a `done` ni a `in_progress`. Las marca el consultor manualmente
- Cambiar `date_done` — lo rellena el consultor al marcar el item como hecho
- Cambiar `user_id` — la asignacion es decision del consultor
- Eliminar items del checklist
- Crear nuevos items en el checklist

---

## Flujo de Procesamiento

```
Item en checklist
    |
    v
¿state == 'done'? --> SI --> IGNORAR (ya completado)
    |
    NO (pending / in_progress / waiting)
    |
    v
¿state == 'waiting'? --> SI --> INFORMAR del bloqueo, no ejecutar
    |
    NO
    |
    v
¿type es procesable? --> NO --> IGNORAR (development, migration, test, ...)
    |
    SI (config o studio)
    |
    v
Incluir en plan de ejecucion (Paso 2)
    |
    v
¿Usuario aprueba? --> NO --> IGNORAR para esta ejecucion
    |
    SI
    |
    v
Ejecutar configuracion (Paso 3)
    |
    v
Documentar (Paso 4)
    |
    v
(consultor revisa y pone state = 'done' + date_done manualmente)
```

---

## Calculo de Progreso

El progreso de la tarea se calcula con los pesos:
- `progreso = SUM(weight de items con state = 'done') / SUM(weight de todos) * 100`
- Este skill NO recalcula el progreso — Odoo lo hace automaticamente al cambiar `state`
