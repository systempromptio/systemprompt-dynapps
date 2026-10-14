---
name: indaws-dev-task-writer
version: 1.6.0
effort: medium
model: opus
allowed-tools: [Read, Write, Bash, Glob, Grep, AskUserQuestion, Agent]
description: >
  Redacta tareas tecnicas Odoo listas para asignar (7 secciones, estimacion con factor IA, criterios
  verificables) a partir de peticiones de consultores, PMs o TLs. NOT análisis ni código → dev-assessment.
  Triggers: "redactar tarea", "crear tarea para dev", "escribir ticket", "tarea para implantacion",
  "tarea de migracion", "task drafting", "crear ticket de desarrollo".
---

# dev-task-writer

Redactor de tareas tecnicas para developers Odoo. El consumidor (TL, consultor o PM) describe una necesidad; este skill produce una tarea lista para asignar — con contexto de negocio, criterios verificables y estimacion de esfuerzo.

> **Filosofia**: una buena tarea es la que el dev completa sin volver a preguntar. El redactor guia, no da codigo.

## Cuando se usa

- **Consultor/PM**: convierte una peticion del cliente en tarea formal para dev
- **TL**: redacta tarea tras decidir Standard/Studio/Custom (post `dev-assessment`)
- **Migracion**: genera tarea de migracion de modulo con 5 gates obligatorios
- **Bug fix**: estructura un ticket reproducible con pasos y comportamiento esperado

## Workflow

### Step 0 — Intake [STOP si falta info]

Validar peticion contra `references/intake-template.md`: problema en lenguaje de negocio, version Odoo, ≥2 escenarios, prioridad. Si falta algo critico → devolver template vacio. NO adivinar. Si viene de `dev-assessment`, exigir el **paquete de traspaso** (origen, adjuntos, acceso al codigo): `references/origin-context.md`.

### Step 0.5 — Fast-path trivial [corta camino]

Si cae claramente en `trivial_if_all` de `delivery-thresholds.json` (un campo/vista/cosmético, sin modelo nuevo, ≤1h) → **saltar assessment, estimación detallada, checklist de impacto, gotchas-preflight y source-trace**: número puntual + `task-redaction.md` mínima (contexto + objetivo + criterio). Ante la duda → NO es trivial, flujo completo.

### Step 1 — Pre-assessment + tipo + niveles

- Si no esta clasificada Standard/Studio/Custom → **delegar a `dev-assessment`**. Si concluye "Standard cubre 100%" → no se redacta tarea, se devuelve respuesta funcional.
- `AskUserQuestion`: tipo (estandar/migracion/bug fix), nivel del asignado (Junior/Mid/Senior), modulo de referencia.
- **Nivel de ENTREGA** (distinto del nivel del dev): derivarlo de las señales de `dev-assessment` (horas, modelos tocados, base/compartido, cross-módulo, migración, seguridad) según `references/shared/delivery-thresholds.json` → trivial/estandar/complejo. Es lo que fija qué exige la tarea (§ estándar de entrega).

### Step 2 — Estimacion + calibración (`references/estimation-calibration.md`)

- **Delegar a `odoo-effort-estimator`**; una sola cifra: el esfuerzo real con factor IA aplicado. Mapear a `x_studio_task_difficulty`: low <8h / medium 8-24h / high 24-60h / very_high >60h.
- **Calibrar con `estimation_history` de `tl-assistant`** si es accesible (sesgo por tipo + gotchas ya vistos en ejecución); si no, anotar "sin calibración histórica".
- **Banda mejor/probable/peor para custom no trivial / integración / migración de datos** (no un punto). Cliente ancla en *probable* (o *peor* si exige cierre). Fase que depende de medición previa → dejarla **abierta**, nunca inventar cierre.

### Step 2.5 — Source-trace del mecanismo central [GATE DURO]

Identificar **el único método/campo/constraint del que depende todo el diseño** y **trazarlo a la fuente de la versión objetivo** (grep en la fuente — resuélvela vía `references/shared/odoo-source.md`, NO asumir `/opt/odoo`; o probe con rollback vía `odoo-flow-tester`), citando `file:line` en la ficha. **Gate:** config-only o specs de stock/UoM/FSM/pagos/contabilidad sin `file:line` (o probe) del mecanismo central → **no se emite** (caso B-04: acertar gotchas periféricos no prueba el mecanismo). Se salta solo en fast-path trivial.

### Step 3 — Generar tarea [opus, inline]

Cargar `references/task-writing-guide.md` (7 secciones), `references/level-scaffolding.md` (Junior/Mid/Senior) y la plantilla correspondiente: `templates/task-redaction.md`, `templates/migration-task.md` o `templates/bug-fix-task.md`. Producir markdown con secciones adaptadas al nivel. **Precargar la sección "Gotchas conocidos que aplican"** (consulta a `odoo-deep-knowledge` + la KB del custom en `.ai-memory/knowledge/` del cliente si existe) y dejar el placeholder de **"Pruebas funcionales realizadas"** + los entregables de flujo/README que exija el nivel de entrega.

Además (salvo fast-path trivial): **pasada del checklist de impacto Odoo** (`references/odoo-impact-checklist.md` — datos existentes/migración, multi-company, multi-currency, seguridad/ACL, upgrade-safety, rendimiento; lo que APLICA entra con instrucción concreta, en complejo cubre `impact_analysis`); **verificar el módulo de referencia** antes de citarlo (existe + versión objetivo + patrón vigente); **criterios de aceptación ejecutables** ("Cuando X con valores, entonces Y") enlazables con `odoo-flow-tester`/`odoo-test-generator`, no prosa.

### Step 4 — Auto-validacion (opcional) → `quality-gate`

Verifica: contexto presente, objetivo describe comportamiento, alcance excluido explicito, criterios "Cuando X, entonces Y", restricciones con justificacion, <3 preguntas abiertas. Si falla → iterar.

### Step 5 — Crear en Odoo (opcional, gate `AskUserQuestion`)

Cargar `references/odoo-task-field-mapping.md`, renderizar `templates/*-api.json`, **delegar a `odoo-pilot`** (`create_record.sh project.task`). Verificar `res.users` ID del TL antes (NO confundir con employee ID). Defaults fijos: `stage_id=1008`, `x_studio_is_planned=false`, `x_studio_review_priority="1"`, `x_studio_fecha_de_revision=false`. **Y adjuntar** (`ir.attachment` con `res_model="project.task"` + `res_id`) el markdown de la tarea y **todo documento citado que no esté en un repo** — es lo que sustituye a la ruta local (NEVER #9); lo que no quepa por API (GOTCHA #5) se dice en la tarea y se sube a mano. Adjuntos DECISIVOS: re-adjuntar al task (los >~90 KB se enlazan, GOTCHA 5); el resto, enlazados — ver `references/origin-context.md`.

### Step 6 — PDF corporativo (opcional) → `dynapps-brand` / `dynapps-brand`

NO generar PDF antes del OK del TL — iterar primero en markdown.

## NEVER

1. Dar codigo de la solucion al dev — solo apuntar al patron/modulo de referencia (eso va de lo que escribe la TAREA; que el dev escriba código no se prohíbe nunca, ver #10)
2. Tarea de Senior con guia paso-a-paso de Junior — micromanagement
3. Tarea de Junior sin referencia a modulo existente — pierde 8h explorando
4. Crear `project.task` sin gate de confirmacion del usuario
5. Meter horas/dificultad/testing/breaking-changes en `description` — usar campos Studio dedicados
6. Asignar `user_ids` al crear — el TL asigna despues
7. Tarea con mas de 3 preguntas abiertas — no esta lista, devolver al intake
8. **Emitir una spec cuyo MECANISMO CENTRAL esté asumido de memoria.** Antes de redactar, identificar el único método/campo del que depende todo el diseño y **citar su `file:line`** contra la fuente de la versión objetivo (o el resultado de un probe con rollback). Acertar los gotchas periféricos NO prueba que el mecanismo central sea correcto (caso B-04: `_fsm_create_sale_order_line` era el hook equivocado, `sale_line_id` a un bien es imposible, y nadie abrió la fuente). Config-only de stock/UoM/FSM/contabilidad = hipótesis hasta ejecutarla.
9. **Escribir una ruta LOCAL en la tarea** (`/opt/…`, `~/…`, `C:\…`, un contenedor o una BD de tu máquina). El dev no tiene tu disco: una ruta local es una referencia rota que además parece válida. Cada recurso va como **repo + rama + ruta relativa al repo**, o como **adjunto de la tarea** si no está en git (specs del cliente, ficheros de ejemplo, planes de prueba). El entorno se describe por lo que hace falta (versión, addons necesarios, skill que lo levanta), NUNCA por la ruta del tuyo. Antes de cerrar: `grep -nE '(/opt/|/home/|~/|[A-Z]:\\)'` sobre el markdown Y sobre los campos que suben a Odoo — si hay coincidencias, no está lista.
10. **Prohibirle al dev tocar o añadir código. NINGUNA tarea lo hace, tampoco una revisión o auditoría.** Para sacar un informe hay que tocar el código, así que la prohibición no evita el trabajo: solo impide que se entregue. El PR lleva **el código**; el informe y las evidencias son el respaldo, no el entregable — y **el código manda sobre las evidencias**. El filtro de calidad es la review del TL, que ocurre DESPUÉS y sobre el diff; no una prohibición previa. Si de verdad hay algo que no se debe cambiar todavía (falta una spec, hay una decisión de diseño abierta), se nombra **caso por caso y con su motivo**, nunca como norma general de la tarea; y si el alcance del arreglo no se puede estimar, la fase se deja **abierta en horas**, jamás prohibida.

## ALWAYS

1. Empezar por el POR QUE (Contexto de Negocio) — sin contexto el dev decide mal
2. Adaptar nivel de detalle al nivel del dev (`level-scaffolding.md`)
3. Criterios verificables tipo "Cuando X (valores concretos), entonces Y"
4. Definir timer de ayuda explicito (Junior 30-45 min / Mid 2-4h / Senior 1 dia)
5. Excluir explicitamente lo que NO entra en alcance (anti scope creep)
6. Para migracion: aplicar los 5 gates de `references/migration-task-requirements.md`
7. Verificar `res.users` ID via `search_records.sh res.users` antes de crear via API
8. Aplicar el **estándar de entrega** (`references/shared/delivery-standard.md`): toda tarea (salvo trivial) lleva sección de gotchas precargada, criterio de prueba de flujo ejecutable y placeholder "Pruebas funcionales realizadas"; las complejas, además README. El nivel de entrega sale de `delivery-thresholds.json`, no a ojo.
9. **Trazar el mecanismo central a fuente ANTES de redactar** (ver NEVER #8): el hook/campo/constraint del que depende el diseño se verifica contra la versión objetivo (o probe con rollback vía `odoo-flow-tester`) y se cita `file:line` en la ficha. Regla dura para specs de stock/UoM/FSM/pagos y para cualquier "config-only".
10. **Precargar "Punto de partida verificado (no re-investigar)"** cuando el redactor YA ha explorado el código: listar lo que está resuelto/cubierto (con evidencia) para que el dev no repita el análisis, y separar lo "a verificar" de lo "ya confirmado". Evita que un junior gaste horas redescubriendo lo que el TL ya comprobó.
11. **Estimación con fase abierta cuando el alcance depende de una medición.** Si parte del trabajo solo se dimensiona tras un paso previo (p.ej. medir coverage antes de saber cuántos tests faltan), dejar esa fase **explícitamente abierta** con las fases fijas cerradas — NUNCA inventar un número cerrado. Decirlo en la ficha: "Fase C escala con lo que aparezca en Fase B".
12. **Pasar el checklist de impacto Odoo** (`references/odoo-impact-checklist.md`) en toda tarea no trivial (datos existentes, multi-company/currency, seguridad/ACL, upgrade-safety, rendimiento); lo que aplica entra con instrucción, y **verificar que el módulo de referencia existe y está a la versión objetivo** antes de citarlo (nunca de memoria).
13. **Banda de riesgo (mejor/probable/peor) para custom no trivial, integración y migración de datos** — no un punto. Calibrar con `estimation_history` de `tl-assistant` si está disponible.

## GOTCHAS

1. **`res.users` ID != employee ID, y el `ODOO_UID` del entorno puede mentir** — con API key (json2/Bearer) el server resuelve el usuario desde la key e **ignora** `ODOO_UID`, así que un valor equivocado no da error: solo deja `x_studio_review_user_id` apuntando a otra persona. Confírmalo con el `create_uid` de un registro creado, no con la variable.
2. **Selection fields son strings** — `priority="1"` no `1`. `x_studio_task_difficulty="low"` no booleano.
3. **Sintaxis ORM** — One2many: `[[0, 0, {...}]]`. Many2many tags: `[[6, 0, [97, 83, 54]]]`.
4. **Stage inicial siempre 1008** (A planificar), nunca "Desarrollo", y **`x_studio_is_planned=false`** al crear: el TL planifica después de revisar.
5. **Los adjuntos grandes NO caben por `odoo-pilot`** — los scripts pasan el JSON como argumento de shell y Linux corta un argumento en 128 KB, así que en base64 solo entran ficheros de ~90 KB (error: *"lista de argumentos demasiado larga"*, no es Odoo). Lo que no quepa: dilo en la tarea y súbelo a mano por el navegador; nunca lo sustituyas por una ruta local (NEVER #9).
6. **`allocated_hours` = el esfuerzo real con factor IA**. Es la unica cifra; no existe una version alternativa sin IA.
7. **`description` solo narrativa** — testing en `x_studio_task_testing`, breaking changes en `x_studio_explicacin`, pasos en `xx_checklist_ids`.
8. **Tarea de migracion en ingles**; tareas estandar en castellano salvo indicacion contraria.
9. **`allow_task_dependencies` y `allow_milestones` pueden estar desactivados** — verificar antes.
10. **No todo dev tiene equipo** — `hr.employee.x_studio_team_id`, Many2one, puede ser null. OJO: es el equipo del EMPLEADO. `project.task` ya NO tiene campo de equipo propio; al equipo de una tarea se llega por `project_id.xx_team_id`, y una tarea sin proyecto no tiene equipo.

## Delegates to

| Skill | Suite | Para que |
|-------|-------|----------|
| `dev-assessment` | developer | Clasificar Standard/Studio/Custom antes de redactar |
| `odoo-effort-estimator` | common | Tablas de horas, riesgo, factor IA |
| `odoo-pilot` | common | Crear `project.task` via API |
| `quality-gate` | common | Validar checklist del redactor (auto-revision) |
| `dynapps-brand` / `dynapps-brand` | common | PDF corporativo del documento |
| `odoo-module-scanner` | common | Inventariar componentes en tareas de migracion |
| `odoo-version-api-reference` | common | Breaking changes en tareas de migracion |
| `odoo-deep-knowledge` | odoo-knowledge | Restricciones técnicas y gotchas para las tareas |

## Consumed by

| Skill | Suite | Para que |
|-------|-------|----------|
| `project-plan` | consultant | Generar tareas individuales tras planificacion |
| `project-odoo-creator` | consultant | Convertir items del plan en project.task |
| `client-request-manager` | consultant | Cuando peticion de cliente requiere desarrollo |
| `migration-assistant` | common | Generar tareas de migracion por modulo |
| `project-analysis` | consultant | Redactar la tarea tecnica de un GAP clasificado como [DEV] |

Tambien lo invoca tl-assistant (common) (WF3 de redaccion tras decision tecnica). Patrones compartidos en `common:references/shared/` (`odoo-connection.md`, `field-mapping-syntax.md`, `batch-limits.md`).

## REFERENCES

- `references/intake-template.md` — validacion de peticion entrante
- `references/task-writing-guide.md` — 7 secciones, anti-patterns, patrones por tipo
- `references/level-scaffolding.md` — tablas Junior/Mid/Senior, timers de escalacion
- `references/odoo-impact-checklist.md` — impacto obligatorio (datos, multi-company, seguridad, upgrade, rendimiento) + verificación del ref; `references/estimation-calibration.md` — feedback loop (`estimation_history`) + banda de riesgo + fase abierta
- `references/odoo-task-field-mapping.md` — contenido → campos `project.task` + Studio
- `references/checklist-model.md` — modelo `xx.checklist.item` (one2many `xx_checklist_ids`)
- `references/migration-task-requirements.md` — 5 gates obligatorios de migracion
- `templates/task-redaction.md` + `task-redaction-api.json` — estandar (md + JSON API)
- `templates/migration-task.md` + `migration-task-api.json` — migracion (md + JSON API)
- `templates/bug-fix-task.md` — bug fix
- `examples/` — Junior, Mid, Senior, migration ejemplos
