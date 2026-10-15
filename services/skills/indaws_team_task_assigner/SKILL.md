---
name: indaws-team-task-assigner
description: >
  Asistente de asignacion de tareas para Team Leaders. Automatiza la planificacion semanal
  del equipo: revisa pendientes, replanifica tareas no completadas, prioriza nuevas tareas
  y las asigna a desarrolladores segun criterios de afinidad respetando el limite de 40h
  semanales. Usa este skill cuando el usuario pida "asignar tareas", "planificar semana",
  "repartir tareas del equipo", "asignacion semanal".
version: 2.1.1
effort: medium
model: opus
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion]
---

# Team Task Assigner v2.0

Planificacion semanal del equipo en **2 pasos**: snapshot + decisiones.

---

## Principios

1. **SIEMPRE usar scripts** - NUNCA construir curl/API inline
2. **`snapshot.py` da la foto completa** - pendientes + tareas + capacidad + propuesta en 1 ejecucion
3. **El algoritmo de asignacion se ejecuta en Python** (dentro de snapshot), no en el LLM
4. **El LLM presenta, interpreta y ejecuta** - no calcula
5. **1 unico STOP** antes de ejecutar cambios en Odoo

---

## Scripts

Todos importan `scripts/odoo_api.py` (modulo base API). `skill_matrix.json` guarda los datos de skills.

| Script | Entrada | Salida | API calls |
|--------|---------|--------|-----------|
| `snapshot.py` | team_id, week_start [--with-tickets] [--with-releases] | 5 bloques + propuesta | ~12 |
| `execute_replan.py` | action_id, replan_ids, done_ids, [reassign] | resultados | variable |
| `execute_plan.py` | action_id, assignments_json | resultados | 2*N |
| `write_tasks.py` | updates_json | resultados | N |
| `skill_matrix.py` | comando (show/set/remove/sync/match/report) | Matriz o informe | 0-1 |
| `forecast.py` | team_id [--weeks 4] [--week-start] | Tabla forecast | ~8 |

**Env**: `ODOO_KEY` y `ODOO_USER` OBLIGATORIOS. `ODOO_URL` (default `https://portal.dynapps.es`) y `ODOO_DB` (default `master`) opcionales.

---

## Flujo: 2 pasos

### Paso 1: Snapshot

```bash
# desde el directorio de esta skill
python3 scripts/snapshot.py <team_id> <week_start> [--with-tickets] [--with-releases]
```

Ejemplo: `python3 scripts/snapshot.py 28 2026-03-23 --with-tickets`

El script detecta automaticamente los empleados del equipo (dept Desarrollo, excluye CTOs).

Devuelve 5 bloques pipe-separated (CONFIG, PENDIENTES, TAREAS, CAPACIDAD, RESUMEN).
Con ese output, construir **3 tablas markdown** (Pendientes, Propuesta, Capacidad) para el TL.
Formato exacto del output y de las tablas → `references/sample-output.md`.

Si hay tareas con `SIN_HORAS`: alertar y pedir horas al TL antes de continuar.

### Paso 2: Decisiones del TL

El TL responde en **lenguaje natural** (p.ej. *"acepto todo"*, *"la de Akash esta hecha, el resto replanificar"*, *"la 17092 a Brenda en vez de Daniel"*, *"quita la 17115, la 17268 ponle 4h y a Laura"*, *"hecha la 22100, pasa la 22101 a Daniel"*).

#### Interpretar y confirmar

1. Parsear las decisiones del TL
2. Redibujar las tablas con los cambios aplicados
3. Mostrar tabla resumen de acciones:

| Accion | Detalle |
|--------|---------|
| Marcar hecha | slot 22100 |
| Replanificar | slots 22101, 22103 |
| Asignar horas | tarea 17115 -> 4h |
| Planificar | 6 tareas (tabla) |

**STOP**: *"Confirmas? Esto ejecutara los cambios en Odoo."*

#### Ejecutar (tras confirmacion)

**1. Pendientes** (si hay cambios):
```bash
python3 scripts/execute_replan.py <action_id> '<replan_ids>' '<done_ids>' '<reassign_map>'
```

**2. Horas** (si hay tareas con horas nuevas):
```bash
python3 scripts/write_tasks.py '<updates_json>'
```

**3. Planificacion**:
```bash
python3 scripts/execute_plan.py <action_id> '<assignments_json>'
```

Donde `assignments_json` usa los `user_id` del bloque CONFIG:
```json
[{"task_id": 17277, "user_id": 112}, {"task_id": 17092, "user_id": 45}]
```

**4. Resumen final**: tareas OK, errores, capacidad restante por dev.

---

## Reglas

1. **NUNCA ejecutar sin confirmacion** del TL
2. **NUNCA superar 40h** semanales por dev
3. **NUNCA sugerir horas** para tareas sin horas - solo preguntar al TL
4. **NUNCA construir curl/API inline** - SIEMPRE usar scripts
5. **SIEMPRE mostrar justificacion** (columna motivo) de cada asignacion
6. **REDIBUJAR tablas** tras cualquier cambio del TL

---

## REFERENCES

- `references/sample-output.md` — output completo de `snapshot.py` (5 bloques) y plantillas de las 3 tablas de presentacion al TL
- `references/skill-matrix-guide.md` — Skill Matrix: setup, comandos, niveles, taxonomia e integracion en el scoring de `snapshot.py`
- `references/forecast.md` — Forecast multi-semana: uso de `forecast.py`, output y cuando ejecutarlo
- `references/odoo-fields.md` — acciones de servidor (Pasar siguiente semana / Planificar 1ª Semana) y tabla de campos Odoo
- `references/worked-example.md` — ejemplo completo de una sesion de planificacion de principio a fin

## GOTCHAS

1. **`x_studio_review_priority` es Selection almacenado como STRING** — `"3"`=Urgente, `"2"`=Alta, `"1"`=Media, `"0"`=Baja. Filtrar con enteros (`3`) no devuelve nada.
2. **Los IDs de acción de servidor varían entre entornos** — `snapshot.py` los resuelve por nombre automáticamente. NUNCA hardcodear el `action_id`.
3. **"Planificar (1ª Semana)" salta en silencio** si la tarea no tiene `x_studio_plan_user_id`, `x_studio_plan_hours > 0` y `x_studio_is_planned = false`. Verificar los tres antes de ejecutar.
4. **`user_id` ≠ employee id** — `assignments_json` usa el `user_id` (res.users) del bloque CONFIG del snapshot, no el id de empleado. Mezclarlos asigna a la persona equivocada o falla.
5. **Tareas `SIN_HORAS`** — NUNCA inventar horas; alertar y pedirlas al TL. Y NUNCA superar 40h/semana por dev.
6. **Conexión Odoo self-contained** — usa `scripts/odoo_api.py`, no la skill `odoo-pilot`. Requiere `ODOO_KEY`/`ODOO_USER` en entorno.

## Delegates to

Ninguna — skill **self-contained**. El algoritmo corre en Python (`snapshot.py`) y el LLM (**opus**) solo presenta, interpreta y ejecuta con 1 STOP. NO usa `references/shared/odoo-connection.md`: la conexión va por sus scripts propios (`odoo_api.py`), por lo que requiere `ODOO_KEY`/`ODOO_USER` en entorno.

## Consumed by

| Skill | Suite | Para qué |
|-------|-------|----------|
| `tl-assistant` | common | Planificación semanal + forecast 4 semanas del equipo (delegación desde WF-WL) |
| `standup-facilitator` | common | Combinar snapshot de capacidad con la agenda del standup |
