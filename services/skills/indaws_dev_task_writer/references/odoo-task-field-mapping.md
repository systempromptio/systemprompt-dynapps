# Mapeo de Campos Odoo para `project.task`

## Principio

NO meter todo en `description`. Distribuir la informacion en campos nativos de `project.task` y campos Studio (`x_studio_*`). La descripcion queda solo para contexto narrativo.

## Mapeo: Seccion de tarea → Campo Odoo

| Informacion | Campo | Tipo | Valor / Contenido |
|---|---|---|---|
| Titulo | `name` | char | Tarea estandar: titulo conciso. Migracion: `[MIG-V{Y}] {modulo} (Capa {N} — {grupo})` |
| Proyecto | `project_id` | many2one | ID del proyecto en Odoo |
| Etapa inicial | `stage_id` | many2one | `1008` (A planificar) — **SIEMPRE** |
| Prioridad | `priority` | selection | `"1"` (Medium) — string, no int |
| Dificultad | `x_studio_task_difficulty` | selection | `"low"` / `"medium"` / `"high"` / `"very_high"` |
| Horas estimadas | `allocated_hours` | float | Esfuerzo real, con factor IA aplicado |
| Planificado | `x_studio_is_planned` | boolean | `false` — **SIEMPRE false** al crear |
| Fecha revision | `x_studio_fecha_de_revision` | date | `false` (vacio) — **SIEMPRE vacio** al crear |
| Prioridad revision | `x_studio_review_priority` | selection | `"1"` — **SIEMPRE 1 estrella** |
| TL reviewer | `x_studio_review_user_id` | many2one | `res.users` ID del TL (**NO** employee ID) |
| Horas planificacion TL | `x_studio_plan_hours` | float | Horas que dedico el TL/redactor a planificar |
| Dev asignado | `user_ids` | many2many | **Sin asignar al crear** (TL asigna despues) |
| Etiquetas | `tag_ids` | many2many | Migracion: `[97 (MIGR), 83 (inDAWS Core), 54 (Desarrollo)]` |
| Tarea padre | `parent_id` | many2one | Tarea paraguas del bloque (opcional) |
| Contexto + criterios | `description` | html | Solo contenido narrativo |
| Plan de testing | `x_studio_task_testing` | html | HTML con pasos de validacion |
| Breaking changes / gotchas | `x_studio_explicacin` | text | Texto plano con gotchas y restricciones tecnicas |
| Pasos del alcance | `xx_checklist_ids` | one2many | Items con tipo, peso, secuencia |

## Que va en `description` (HTML)

Solo contenido narrativo que el dev necesita para entender el problema:

1. **Contexto de Negocio** — POR QUE importa (2-3 parrafos)
2. **Objetivo** — QUE comportamiento se espera
3. **Alcance** — Incluido + Excluido explicitos
4. **Guia de Implementacion** — adaptada al nivel del dev
5. **Criterios de Aceptacion** — checklist verificable
6. **Referencias** — modulos/codigo a consultar

## Que NO va en `description`

- Horas estimadas → `allocated_hours`
- Dificultad → `x_studio_task_difficulty`
- Plan de testing → `x_studio_task_testing`
- Breaking changes / restricciones tecnicas → `x_studio_explicacin`
- Pasos del trabajo → `xx_checklist_ids`
- Tags, prioridad, etapa → campos nativos

## Valores selection validos

### `x_studio_task_difficulty`
- `"undefined"` → N/S
- `"low"` → Baja (<8h)
- `"medium"` → Media (8-24h)
- `"high"` → Alta (24-60h)
- `"very_high"` → Muy alta (>60h)

### `priority`
- `"0"` → Low
- `"1"` → Medium
- `"2"` → High
- `"3"` → Urgent

### `x_studio_review_priority`
- `"0"` → Normal
- `"1"` → Low (1 estrella)
- `"2"` → High
- `"3"` → Very High

## Tags de migracion (IDs fijos)

| ID | Tag |
|---|---|
| 97 | MIGR |
| 83 | inDAWS Core |
| 54 | Desarrollo |

Sintaxis ORM para asignar: `[[6, 0, [97, 83, 54]]]`

## Etapas del proyecto (IDs conocidos)

| ID | Etapa | Uso |
|---|---|---|
| 1008 | A planificar | **Etapa inicial de toda tarea nueva** |
| 1909 | Migraciones | Cuando se asigna al sprint de migracion |
| 11 | Desarrollo | Dev trabajando activamente |
| 23 | Testeo | En fase de testing |
| 25 | Pasar a produccion | Listo para deploy |
| 12 | Hecho | Completada |

## GOTCHA: `res.users` ID != employee ID

El campo `x_studio_review_user_id` es Many2one a `res.users`, NO a `hr.employee`.

- `ODOO_UID` en CLAUDE.md = **employee ID** (ej: 1278)
- `res.users` ID = se obtiene buscando por login (ej: 841)

Para obtener el `res.users` ID correcto via `odoo-pilot`:

```bash
./scripts/search_records.sh res.users '[["login","=","toni@indaws.es"]]' '["id","name"]' 1
```

## Sintaxis ORM importante

| Operacion | Sintaxis |
|-----------|----------|
| Many2many: reemplazar lista | `[[6, 0, [id1, id2, id3]]]` |
| Many2many: anadir | `[[4, id]]` |
| One2many: crear nuevo | `[[0, 0, {valores}]]` |
| One2many: actualizar | `[[1, id, {valores}]]` |
| Many2one: asignar | `id` (entero, no array) |

## Checklist `xx_checklist_ids`

Modelo: `xx.checklist.item`. Campos clave:

| Campo | Tipo | Valor |
|-------|------|-------|
| `name` | char | Titulo del item |
| `type` | selection | CERRADA: `analysis` / `migration` / `development` / `report` / `test` / `training` / `studio` / `config` |
| `sequence` | integer | Orden (1, 2, 3...) |
| `weight` | integer | Peso % en el avance (suma 100 entre todos) |
| `state` | selection | `pending` al crear (`pending`/`in_progress`/`waiting`/`done`) |
| `notes` | html | Notas / skills a invocar / referencias |

## Defaults fijos al crear via API

```json
{
  "stage_id": 1008,
  "priority": "1",
  "x_studio_is_planned": false,
  "x_studio_fecha_de_revision": false,
  "x_studio_review_priority": "1"
}
```

Estos cinco valores no se calculan — siempre se ponen asi al crear.
