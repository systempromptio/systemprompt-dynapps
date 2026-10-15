# Modelo de Checklist: `xx.checklist.item`

## Descripcion

Modelo que almacena los items de checklist de una tarea (`project.task`). Cada item representa un paso del trabajo con tipo, peso, secuencia y estado.

Relacion: `project.task.xx_checklist_ids` → `xx.checklist.item` (one2many via `task_id`)

## Campos disponibles

Verificados con `fields_get` sobre el modelo. Estos son TODOS: cualquier otro nombre revienta la creacion de la tarea entera.

| Campo | Tipo | Obligatorio | Descripcion |
|---|---|---|---|
| `name` | char | **si** | Nombre del paso ("Paso") |
| `task_id` | many2one | **si** | Tarea padre (lo rellena Odoo en el one2many) |
| `weight` | integer | **si** | Peso relativo (% del esfuerzo total) |
| `state` | selection | **si** | Estado del item. Default `pending` |
| `type` | selection | no | Tipo de paso (lista CERRADA, ver abajo) |
| `sequence` | integer | no | Orden de ejecucion. Default 10 |
| `notes` | html | no | Notas detalladas (skills a usar, instrucciones) |
| `has_notes` | boolean | no | Calculado sobre `notes`. NO se escribe |
| `user_id` | many2one | no | Responsable del paso |
| `date_deadline` | date | no | Fecha limite prevista |
| `date_done` | date | no | Fecha en que se completo. NO se rellena al crear |
| `project_id` | many2one | no | Proyecto |
| `company_id` | many2one | no | Compania |

**Sin equivalente en el modelo nuevo** (existian en `x_project_indaws_check` y se perdieron):
`x_studio_hito`, `x_studio_equipo`, `x_studio_crea_ticket`, `x_studio_ticket_id`.

## Valores de `state`

Sustituye al antiguo booleano `x_studio_completado`. Un item nuevo nace en `pending`.

| Valor | Label |
|---|---|
| `pending` | Por hacer |
| `in_progress` | En curso |
| `waiting` | En espera |
| `done` | Hecho |

## Valores de `type`

Lista **CERRADA** de ocho valores tecnicos en minusculas. Un valor fuera de la lista es rechazado por Odoo.

| Valor | Label | Uso |
|---|---|---|
| `analysis` | Analisis | Analisis de compatibilidad, impacto |
| `migration` | Migracion | Migration scripts, carga de datos |
| `development` | Desarrollo | Codigo (Python, XML, JS, manifest) |
| `report` | Informes y formatos | Reports, templates |
| `test` | Pruebas | Tests unitarios, quality gate |
| `training` | Formacion | Formacion al equipo/cliente |
| `studio` | Studio / automatizacion | Configuraciones Studio/Automation |
| `config` | Configuracion | Configuracion funcional |

Equivalencia con los valores antiguos: `ANLS`→`analysis`, `MIGR`→`migration`, `DEV`→`development`, `formatos`→`report`, `TEST`→`test`, `TRAIN`→`training`, `STUDIO/AUTO`→`studio`, `CONFIG`→`config`.

## Patron estandar de 5 items para migracion

| Seq | Tipo | Nombre | Peso | Notes |
|-----|------|--------|------|-------|
| 1 | `analysis` | Compatibility analysis v{X}→v{Y} | 15 | Skills: `odoo-module-scanner` + `odoo-version-api-reference` |
| 2 | `migration` | Migration scripts | 20 | Skills: `odoo-module-upgrader` |
| 3 | `development` | Code migration (Python + XML + manifest) | 35 | Skills: `odoo-module-upgrader` + `odoo-module-creator` + `indaws-core-developer` |
| 4 | `test` | Unit tests | 20 | Skills: `odoo-test-generator` |
| 5 | `test` | Quality gate validation | 10 | Skills: `quality-gate` + `odoo-code-auditor` |

Los pesos suman 100 y se ajustan segun complejidad del modulo. Para modulos con JS/OWL, anadir item extra tipo `development` con peso redistribuido.

La columna "Hito" del patron anterior desaparece: `x_studio_hito` no tiene equivalente. Los items 3 y 5, que antes se marcaban como hito, ya no se distinguen del resto.

## Sintaxis ORM para crear items

Desde `project.task.create` o `write`, los checklist items se crean inline con comando `[0, 0, {...}]`. `task_id` NO se pasa: lo rellena Odoo al vincular el one2many.

```json
"xx_checklist_ids": [
  [0, 0, {
    "name": "Compatibility analysis v18→v19",
    "type": "analysis",
    "sequence": 1,
    "weight": 15,
    "state": "pending",
    "notes": "<p><b>Skills</b>: <code>odoo-module-scanner</code></p>"
  }],
  [0, 0, { ... }]
]
```

## Comandos ORM para one2many

| Comando | Sintaxis | Uso |
|---|---|---|
| Crear | `[0, 0, {vals}]` | Crear nuevo item inline |
| Actualizar | `[1, id, {vals}]` | Actualizar item existente |
| Eliminar | `[2, id, 0]` | Eliminar item |
| Desvincular | `[3, id, 0]` | Desvincular sin borrar |
| Reemplazar todo | `[6, 0, [ids]]` | Reemplazar con lista de IDs (para many2many) |
