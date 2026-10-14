# Board spec — what the interactive board contains

The output contract of `gen_board.sh` + `assets/board_template.html`. The generator ONLY emits the
JS data constants (spliced at `/*__DATA__*/`); the **health rule, daily suggestions and overdue
filtering run client-side in the template's JS**, while the **findings row is computed by the
generator** in awk (the template only paints it). Everything Odoo writes goes out through `esc()`,
which escapes the JS string AND `&<>`, because the template renders it with `innerHTML`; the file
opens with `<!DOCTYPE html>` and `<meta charset="utf-8">`. Read this before
changing anything the user sees on the board. Reference design: the Rotger board by Aitana,
integrated on this skill's pipeline (data comes from `refresh_gantt.sh`, never hand-fed).

## Views and chrome

- **Top bar**: brand + project, view switcher (Vista general · Gantt · Áreas · Pipeline · Resumen), zoom −/+, theme
  night/day (☾/☀, stored as `board-tema`), ✦ Daily, 📩 overdue client requests (red badge),
  🗄 archive tray (badge), "Proyecto ↗" (Odoo link). **Zoom compensates width**: `setK` also sets
  `width=100/K%` and `max-width=1660/K` so the time axis ALWAYS fits the window — plain `zoom`
  used to push the right months behind the viewport edge.
- **Bottom bar**: block navigation ‹ › (also ← → keys), current block label, "↑ previous area",
  "⌂ home", one dot per subblock, health legend. `Esc` closes popup first, then the area sheet.
- **Hero**: project name (+`em` = the part after " - "), window, go-live, team chips
  (project manager + task assignees), KPIs: tasks · areas · validated items · milestones reached.

## Chronology (Vista general / Gantt views)

- Areas ordered chronologically, each row: expand +, **health dot**, name (area color from a
  4-stop ramp over the brand gradient), task count, "Detalles" button, range band on the track.
  Expanding shows subblock separators (aggregated progress bar) and one thin bar per task
  (checklist fill; red outline = deviated) with `done/total · CR n` on the right. The EXPANDED
  area is highlighted as one block: an area-color tint (hex+alpha `24` on the header row, `14` on
  child rows), a 3px inset spine in the area color, and a rounded `.zclose` strip before the next
  area — note `.srow.inz,.zsep.inz{border-radius:0}` must keep that specificity or the base row
  radius breaks the spine into arcs.
- **Milestone lane** H1…Hn (filled = reached) and **month axis** live above the rows; the
  **TODAY chip sits on its own strip** (`.todaystrip`) so it never stomps a milestone or a month
  label, and the TODAY line runs CONTINUOUS through strip + months + milestones + every row.
- **Track geometry is unified** — every per-row today segment and bar must share the exact canvas:
  left cluster 436px ("Detalles" width:77 border-box, `+` button box-sizing, `flex:none` on
  .rn/.sn) and right −56px (`.sx` spacer on area rows, `margin-right:56px` on .months/.hitos,
  no right padding on .row). GOTCHA: `.months span{translateX(2px)}` (month labels) also pushed
  the axis today-segment → excluded with `:not(.today)`.

## Health rule (traffic light)

Per task, in template JS (`risk()`): expected = elapsed fraction of the task window at TODAY;
real = validated/total checklist items. gap = real − expected → **En plazo** (gap ≥ −5%),
**Atención** (≥ −20%), **Desviada** (worse). Overdue-not-done = red; `1_done` = Completada;
no dates = "Sin fechas" (amber) and never enters the chronology. **A task with NO checklist items
has no `real` to compare**: it comes out GREY ("Sin checklist"), never red, and carries that as a
blocker — scoring it 0% turned every task without items red as soon as 20% of its window had
elapsed, i.e. the traffic light punished the missing data instead of the delay. They are counted
once as an amber finding ("N tareas sin checklist"). Grey in the legend therefore means "the
traffic light does not judge this one": not started, or nothing to measure. The area inherits the worst
state of its live tasks. Blockers per task (rule-based): no dates · stage "A planificar" /
"Pendiente Cliente" · no assignee.

## Area sheet (click a card / "Detalles")

Eyebrow + title + summary (tasks, subblocks, items, range, health, pending requests). **Subblocks
come from the task NAME PREFIX** — `[DEV]`, `[INTEG]`, `[CONFIG]`, `[STUDIO/AUTO]`, `[CORE]`,
`[MIGR]`, `[TEST]`, `[TRAIN]`, `[ANLS]`; no prefix = "Funcional" (the `#id / ` name prefix is
stripped on load). Task cards: type chip + PRIORITARIA (priority ≥ 1) / URGENTE (= 3) + health +
propose-archive ×; dates · assignees · stage; progress bar with a vertical mark at the EXPECTED
position; disclosures for client requests and blockers; **"Checklist · X de Y — ver todos" opens
a POPUP** with the FULL checklist (validated struck with green check, HITO chip (red) or weight ≥ 3
chip (orange), end date, owner initials).

## Client requests

Source: the `x_client_request` model (priority 0/1, state draft/sent/received/cancelled, deadline).
Cancelled are excluded everywhere; requests attached to ARCHIVED tasks are not rendered but counted
as a red finding. **Fallback**: projects still using the legacy char field on checklist items get
synthesized requests (state received when the item state is Hecho, else draft) — it triggers only
when `client_requests.csv` is EMPTY (no model, no rows), never because every request happens to be
cancelled or archived, which would have silently served stale data to a migrated project. The 📩 button
opens the **overdue panel**: deadline < today and not answered, grouped by task, sorted by oldest,
each with ALTA chip, deadline, days late and state; clicking navigates to the task in its sheet.

## Findings (riskrow) and Daily

- **Findings** row on Vista general: reds/ambers computed by the generator — requests never sent,
  requests in archived tasks, overdue tasks, dateless tasks, ≥30% unassigned, no dependencies,
  replans vs baseline, 0 validated items. If `data/hallazgos.json` exists (JS array
  `[{t,lvl:'rojo'|'naranja',d}]`) it REPLACES the automatic list — that is where the agent writes
  a curated PM reading (e.g. "Go-Live without buffer").
- **✦ Daily**: due this week (requests + task starts in [−30d,+7d]), "you can advance now"
  (assigned tasks not on track), and replans vs the baseline (from `MOV`, top 6 by drift).
- **🗄 Tray**: propose-archive marks — LOCAL only (`localStorage` key `board-arch-<pid>`), never
  writes to Odoo; archiving for real happens in Odoo and the mark clears on the next refresh.

## Files

`reports/board_<slug>_<date>.html` — same-day regenerations overwrite (F5); a new day is a new
file. Theme, zoom and archive marks persist per project in the browser. Hours are NOT rendered
anywhere (the `hrs` slot in the task tuple is emitted as 0) — same client-publishable policy as
the classic gantt.

## Pipeline view (3 tabs + toolbar)

- **Toolbar** (persists across re-renders — lives outside `#pipebody`): a live search input
  (`nrm10`-normalized, matches area/module, task, item text, owners; legends and column counters
  recount, "filtro X · N coinciden") and "Agrupar:" pills per tab (dev: none/area/owner/type ·
  checklist: area/owner/task/none · requests: area/task/none) that inject `.pah9` subgroup
  headers inside every column (`pgsort`/`pgfill`; area grouping keeps plan order, the rest sort
  alphabetically).
- **Tareas técnicas**: kanban of technical tasks — bracket prefix `[X]` or tag "Desarrollo"
  (`TDEV`) — in 4 columns by stage bucket (`tbk9`, mapping in **Stage → bucket** below):
  A planificar · En proceso · Testeo·review · Bloqueado. Finished/cancelled
  stay out (legend counter). Cards carry the **execution traffic light** from `TSF` (left border:
  blue <24h · green ≤3 business days · orange ≤5 · red stalled/no timesheets; backlog without
  timesheets gets no light) and sort blues-first, then deadline. Click → area sheet focused on the
  task. **"Pasar a producción" is NOT a column**: it renders as a collapsed teal `details.prosec`
  deployment-queue section under the kanban (summary shows the count; cards reuse `mkptc`).
- **Checklist**: every checklist item of active tasks in 4 columns by ITEM state (`bof9`/`BK9`,
  the same buckets that group the full-checklist popup): Pendiente · En progreso · En espera ·
  Hecho (cancelled out, counter in the legend). Items grouped by area inside each column, HITO
  first, nearest end date next; "estado Hecho + unchecked box" shows "· sin validar". Click →
  the task's full-checklist popup.
- **Peticiones cliente**: every x_client_request of active tasks in 3 columns by x_state
  (grid `.c3`): Por enviar (draft) · Enviada·esperando cliente (sent) · Recibida (received);
  cancelled filtered at refresh. ALTA chip when x_priority=1; overdue (deadline past, not
  received) get a carmine left border and bold date. The legend's overdue count must match the
  📩 badge. Click → area sheet.

## Stage → bucket (`tbk9`)

Same table as `bucket()` in the dev-panel skill (its `references/rendering.md`). **Change one, change
the other**: they are two implementations of one rule (awk there, JS here).

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

## Resumen view (executive charts)

- Built ONLY from already-emitted constants (CHK/CR/TSF/HITOS/TDEV) — no extra Odoo calls, and
  the no-hours policy applies end to end.
- **KPI tiles**: real progress % (validated/total items) with a spark bar + expected-by-today
  tick and a signed delta; go-live date with countdown and milestones reached; remaining items
  (with overdue — clickable — and dateless counts); blocked now (blocked tasks + waiting items +
  overdue requests).
- **Charts** (palettes validated with the dataviz skill's `validate_palette.js` against BOTH
  surfaces — dark `#13134D` effective, light `#fff`): per-module 100% stacked bars by item state,
  each row prefixed with an **area-identity dot** (`.mdot`, same ramp color as Gantt/Áreas) and its
  "Pendiente" segment tinted in that SAME area color at `.22` opacity instead of a flat neutral —
  don/prc/esp keep their solid state colors, with an expected marker `▾` (items due by today /
  total) and click-through to the area sheet; remaining items per month (stacked by state,
  "Vencido" bucket first, "sin fecha" last — every column and the tile's overdue count open
  `openLst()`, an item-list popup on the checklist modal that drills into `openCk`); execution
  traffic-light distribution as a **donut** (SVG, `r=86` `stroke-width=24`, total in the center,
  same 5 categories/colors as before — only the chart type changed from a segmented bar); **client
  requests** (`.heatrow`/`.hcell` cards: vencidas · en borrador · enviadas · recibidas, global
  totals from `CR[t].d/.s/.rc`, an empty bucket renders dashed) — before this only the overdue
  count leaked into "Bloqueado ahora"; milestones timeline (diamonds: reached green / overdue
  carmine / pending violet, dashed go-live flag) — **moved to the LAST panel** (full-width, closes
  the view as a timeline after scope → execution → requests → blockers).
- **Panel order** in `.sumgrid` (2-col grid, `.chpanel.full` spans both): Avance por módulo ·
  Ejecución técnica (donut) · Qué queda por mes (full) · Peticiones al cliente · Bloqueado ahora
  (or the ok9 empty state) · Hitos de facturación (full, last).
- **State palette** (shared by charts, pipeline chips, popup groups and the execution donut —
  brand-anchored, ONE color per semantic meaning across the whole board): Hecho `var(--ok)`
  (`#0FBF8F`, same green as the health traffic light and the Pipeline `.sf-g` chips — before this
  Resumen/execution used a second, darker green `#0FA36B`) · En progreso `#951CF3` (brand violet) ·
  En espera / Atención `var(--tangerine)` (`#FF7E23`, same orange as the health traffic light, the
  Pipeline `.sf-o` chips and the "Bloqueado ahora" item chip — before this Resumen/execution/item
  chip each used their own darker orange `#E06A10`) · Pendiente `#7272A8` (--lm, deliberate neutral
  — module bars override this with the area tint, see above). Traffic light: `#2F6FED` / `var(--ok)`
  / `var(--tangerine)` / `#D6336C`. Semantics mirror the board's own footer legend (ok green ·
  attention tangerine · problem carmine · not-started grey) — now literally the same CSS values,
  not just the same intent.

## Full-checklist popup

- Items grouped as a mini-pipeline by item state (same `BK9` buckets/colors as the Pipeline
  Checklist tab) with summary chips on top when more than one group exists; "estado Hecho"
  without the validated check renders in the Hecho group WITHOUT the green check = "done,
  pending client validation". `openLst()` (Resumen drill-down lists) reuses this modal.

## Las cinco vistas, en una frase cada una

*(El gantt clásico y sus dos referencias — las de render y sticky del gantt clásico — se retiraron en v2.0.0; siguen en el historial de git.)* El tablero es el único entregable y su contrato es este fichero.
