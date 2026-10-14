# Itinerarios por perfil

Cada perfil define: señales en los datos de Odoo para proponerlo, el quick win del día 1
(común a todos: el primer parte de horas), 3 casos estrella con sus skills, y qué dejar para
más adelante. La regla: día 1 = quick win + demo de UN caso estrella. Semana 1 = los tres
casos. Lo avanzado, cuando haya rodaje.

Los nombres de skills se refieren al marketplace Dynapps Spain AI. Si un skill no está
instalado en el perfil del consultor, anótalo en la Fase 0 y recomienda el plugin que lo trae.

## 1. Consultor de implantación (BC)

**Señales**: departamento Consultoría, tareas en varios proyectos de implantación, checklists.

**Casos estrella**:
1. **Agenda diaria priorizada** — `consultant-daily-planner`. "Qué tengo hoy y por dónde
   empiezo": tareas, peticiones a cliente, tickets y reuniones en una sola vista.
2. **Actas de reunión que se documentan solas** — `project-meeting-minutes`. Pega sus notas y
   sale el acta en Knowledge, con cambios de alcance detectados.
3. **Pulso del proyecto** — `project-health-monitor`. Horas vs presupuesto, hitos y desvíos
   antes de que se los pregunte el PM.

**Más adelante**: `odoo-configurator` (ejecutar checklists), `project-weekly-update`,
`client-request-manager`, `training-planner`.

## 2. Perfil fiscal / contable

**Señales**: tickets de helpdesk de temática fiscal/contable (impuestos, DUA, conciliación,
libro mayor), puesto tipo Consultor Experto contable, poca actividad en proyectos.

**Casos estrella**:
1. **Resolver tickets con red** — `ticket-solver`. Lee el ticket, propone diagnóstico y
   corrige lo asumible; lo demás lo escala a dev con nota técnica.
2. **Auditoría de cierre contable** — chequeos de cierre trimestral sobre asientos, IVA,
   conciliación y bloqueo de periodos (skill de auditoría contable del catálogo demo/interno).
3. **Sus tickets y su carga** — `ticket-report`. Qué tiene abierto, qué ha cerrado, tiempos.

**Más adelante**: `odoo-pilot` para consultas puntuales a instancias de cliente (solo lectura),
`odoo-importation` para cargas masivas.

## 3. Desarrollador

**Señales**: departamento Desarrollo, actividad en repos/tareas técnicas, tickets escalados.

**Casos estrella**:
1. **Evaluar antes de programar** — `dev-assessment` + `odoo-classification-guide`. La escala
   Estándar → Indaws Core → Studio → Custom con justificación. El custom se defiende, no se asume.
2. **Entorno local en minutos** — `docker-deployment`. Levantar el Odoo del cliente con dump
   restaurado sin pelearse con compose.
3. **API version-aware** — `odoo-version-api-reference`. Deprecaciones y diffs v15-v19 antes
   de dar una solución por buena.

**Más adelante**: `quality-gate` antes de merge, `indaws-core-auditor` antes de PR de módulos
core, `odoo-module-documenter`.

## 4. Comercial / SDR

**Señales**: departamento Ventas, actividad en crm.lead, reuniones de demo.

**Casos estrella**:
1. **Briefing comercial del día** — `daily-briefing`. Pipeline, reuniones y prioridades.
2. **Cualificar sin trabajo manual** — `qualification-form` / `qualify`. Investigación y
   formulario pre-rellenado publicado en el chatter del lead.
3. **Preparar la llamada** — `call-prep`, y después `call-summary` para que la reunión quede
   registrada en el CRM sin picar nada.

**Más adelante**: `quotation-builder`, `demo-builder`, `draft-outreach`, `scout-odoo`.

## 5. PM / responsable de proyecto

**Señales**: rol de project manager, varios proyectos como responsable, project.update.

**Casos estrella**:
1. **Salud de cartera** — `project-health-monitor` proyecto a proyecto, con Health Score.
2. **Update semanal al cliente sin fricción** — `project-weekly-update`.
3. **Planificación desde el análisis** — `project-plan` (horas, Gantt, distribución).

**Más adelante**: `migration-assistant` y `migration-project-creator` para migraciones,
`project-health-fixer`, `timesheet-report` para revisar la imputación de su equipo.

## Nota sobre el ritual por perfil

El briefing matutino de la Fase 4 usa, **si están instaladas**: `consultant-daily-planner`
(perfiles 1, 2 y 5) o `daily-briefing` (perfil 4). Si la suite consultant no está disponible,
ofrece una tarea programada genérica (no bloquees el ritual por ello). Para devs, el cierre del
día con partes suele aportar más que el briefing; ofrécelo en ese orden.
