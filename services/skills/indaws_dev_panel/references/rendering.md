# dev-panel — que pinta el panel, bloque a bloque

Inventario de lo que genera `gen_dev_report.sh`. Es referencia de lectura: la `SKILL.md` solo
necesita decir que existe cada bloque, no describirlo entero.

## Bloques

- **Hero + KPIs**: devs, tasks on the panel (finished excluded, with counter), the 4 buckets,
  "Pasar a producción" queue, team hours this week (+absences apart, expected = weekdays × 8h).
- **Filters**: team chips (from the employee's `x_studio_team_id`) coordinated with dev chips
  (picking a team trims the dev chips); a **project search box** below (case/accent-insensitive,
  filters the whole pipeline); active-chip toggle; "Todos" resets. Two extra toggle pills that
  stack with the rest: **"✔ Listo a planificar"** (A planificar with state Approved) and
  **"★ Aceptación"** (stage Acceptance). Search, team and dev filters also reach the deploy queue
  and the signal groups, whose counters recount.
- **Pipeline summary** (collapsed): two 100% bars — composition by stage and by execution
  (traffic light) — recounted on every filter.
- **Priority × execution matrix** (collapsed, same compact style): x_studio_review_priority rows ×
  traffic-light columns with totals; hot cells (urgent/high × stalled) highlighted; the summary
  line answers "how many urgent tasks are stalled" without opening. Cells are CLICKABLE.
- **Kanban 4 columns**: A planificar (slate) · En proceso (violet) · Testeo/review (teal) ·
  Bloqueado (carmine) — palette VALIDATED with the dataviz skill validator (chroma ≥0.10, adjacent
  CVD ≥8.7, normal vision ≥16.1); do not revert to gray/blue without revalidating. Cards: task
  link, project, planning-user initials, hours 14d, last timesheet, deadline (red if overdue),
  priority chip (URGENTE/ALTA/MEDIA), state chips ("cambios pedidos", "en espera"), original stage
  in gray, except Approved tasks in A planificar (green ✔, "listo a planificar") and Acceptance
  (teal ★). **Execution traffic light** as left border + tint + tooltip: BLUE = timesheet in the
  last 24h (date granularity: today or yesterday) · GREEN = ≤72 BUSINESS hours (bizdays() skips
  weekends; JDN%7<5 = Mon-Fri) · ORANGE = ≤120h · RED = >120h or none in the 14-day window.
  "A planificar" NEVER carries a light, with or without timesheets: it is a queue, not execution
  (owner's rule, 2026-10-01).
- **Grouping views**: Estado (default) · Fecha fin (6 columns by deadline bucket) · Proyecto
  (stacked bar chart — one bar per project, segments = pipeline states, sorted by visible total,
  click opens its breakdown — plus collapsed per-project sections with cards grouped BY STAGE).
- **In-pipeline subgroups** (Estado view only): Sin subgrupos · Por proyecto · Por fecha fin ·
  Por prioridad — rebuild() regroups the existing cards under .subh headers (no HTML duplication).
- **Clickable aggregates**: column headers and matrix cells open a MODAL list of those exact tasks
  (respecting active filters), each row linking straight to its Odoo task. This replaces Odoo
  deep links: Odoo 17-19 dropped domain-in-URL, and the computed cuts (traffic light) do not exist
  as Odoo filters at all.
- **"Pasar a producción"** (deploy queue, ALWAYS visible after the boards): tasks in stage
  Release (Odoo 19 catalog), "To Merge" or stages literally named "Pasar/Subir a producción" —
  outside the pipeline on purpose (the user refused a 5th column). Collapsed with a
  green "queue empty" summary when zero; project-grouped pills when not. Each pill carries its
  assignees and teams (unassigned ones inherit the project's team, like the cards), so the
  filters hide and recount it like any card.
- **Per-developer section** (collapsed): card per dev ordered by week hours — agent reading
  (violet box), "Partes de la semana · N frentes · Xh" disclosure with ALL task hours, absence
  chip. NOT filtered by DEV_TAG on purpose: it reflects the dev's full week.
- **Signals** (collapsed, counts in the summary): Blocked / Stalled (red light in En proceso) /
  Hours-on-backlog — tinted blocks with project-grouped pill links.

## Popup de listas (cabeceras de columna y celdas de la matriz)

- **Cabecera sticky resaltada**: bloque con degradado lavanda y filo inferior degradado de marca
  (violeta→carmine→naranja); título, contador y toolbar quedan fijos mientras la lista scrollea.
- **Toolbar propia**: lupa (nrm9 sobre tarea+proyecto, contador "N de M") y pills Agrupar
  APILABLES: Sin grupos · Proyecto · Usuario planif. · Prioridad · Fecha fin. La multi-selección
  es ORDENADA: el orden de activación define el anidado y cada pill activa muestra su nivel
  ("Proyecto 1 · Usuario planif. 2"). Render recursivo lgrp()/lparts(); cabeceras .lv1/.lv2
  indentadas con banda violeta decreciente y contador por nivel. Estado local al popup (se
  resetea al abrir); las filas se construyen una vez (LROWS) y solo se recolocan — la lupa no
  pierde el foco.
- **Bandas de grupo**: cabecera de agrupación como banda violeta con lomo de 3px y contador.

## Chips nuevos en tarjeta

- **"sin asignar"**: tarea sin usuario planificación visible en su equipo por inferencia del
  proyecto (gotcha 13).
- **"sin partes en la etapa"**: En proceso con entrada en etapa reciente y sin partes desde
  entonces — semáforo en blanco (gotcha 12).
- Subgrupo del pipeline "Por usuario planif.": alfabético, "Sin asignar" al final; nombres via
  mapa UNM construido desde los title de las pills de dev.

## Etapa → columna (`bucket()`)

Misma tabla que `tbk9` en la skill project-gantt (su `references/board-spec.md`). **Si cambias una,
cambia la otra**: son dos implementaciones de una sola regla (awk aquí, JS allí).

| Stage (Odoo 19 unified catalog) | Legacy / Spanish names also matched | Bucket |
|---|---|---|
| New, Analysis | A planificar, Nuevo, Análisis, Backlog, Sin asignar, Tareas base | A planificar (`pln`) |
| Implementation | anything not matched elsewhere | En proceso (`prc`) |
| Test, Acceptance | Testeo, Review, QA, Validación, Prueba, Aceptación | Testeo·review (`tst`) |
| Release | Pasar / Subir a producción, To Merge | deploy queue (`pro`), outside the pipeline |
| Done | Hecho, Terminado, Realizado, Cerrado, En Producción, Cancelado | out (`off`) |

Blocked (`blk`) wins over all of them: stages "Esperando", "Bloqueado", "Standby", "Pendiente
cliente". The legacy names stay for databases not yet migrated, and the Spanish ones because the
API returns the stage's SOURCE-language name even when the web shows a translation.
