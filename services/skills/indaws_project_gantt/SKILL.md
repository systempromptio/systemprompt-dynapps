---
name: indaws-project-gantt
description: >-
  Interactive HTML tracking BOARD of ONE Odoo project — the skill's only deliverable (it also
  answers requests for 'the gantt'). Five views: Overview, Gantt by areas, Areas, Pipeline (technical-task kanban with
  a timesheet-driven execution traffic light, checklist items by state, client requests) and
  Resumen (executive charts). Health = real checklist progress vs expected-by-date, billing
  milestones, Daily, findings, and NO hours anywhere, so it can be shared with the client.
  READ-ONLY: to generate a plan use project-plan. One conversation per project: the first run
  names it, then "update" is enough, and the stable dateless board file is always the one opened.
  Triggers: "tablero de X", "board of X", "gantt de X", "gantt de seguimiento", "cómo va X",
  "actualiza el gantt", "actualiza el tablero", "update the board", "pipeline de X", "resumen de X".
version: 2.3.0
effort: medium
model: sonnet
---

# Project Gantt — predictive management

## Important

- **NEVER write to Odoo** — the skill is 100% read-only (the only writes are local files).
- **NEVER overwrite `snapshots/baseline_<pid>.csv`** during a refresh — it is the first snapshot of
  the plan; without it there is no replan detection. Re-baselining = an explicit user decision
  (delete the file).
- Credentials ONLY via the `ODOO_KEY` environment variable or `config.local.sh` in the WORKDIR —
  never in scripts or in the repo.
- **Output language**: everything the skill produces for the user (chat summaries, findings,
  narrative) MUST be in the user's language unless they ask otherwise. The generated dashboard HTML
  currently renders its labels in Spanish (the team's working language) — a known limitation
  tracked in the suite's IDEAS.md.

## Setup (first run per user)

This skill ships as a plugin: its folder is a READ-ONLY cache. All state (data/, snapshots/,
reports/, credentials) lives in the **WORKDIR**: `$HOME/dynapps-informes/project-gantt/`
(override with `GANTT_DIR`). On first run, if the credential is missing the script stops asking for
`<WORKDIR>/config.local.sh` with `ODOO_KEY="..."` — ask the user for their Odoo API key
(portal.dynapps.es → User settings → API Keys) and create it. Per-project baselines survive plugin
updates because they live in the WORKDIR.

Pipeline: `refresh_gantt.sh <project>` (Odoo → data/) → `gen_board.sh`
(→ reports/board_<slug>.html, stable dateless file — the one you open). Dynapps brand. **The project is memorized** in
`data/proyecto_actual`: subsequent invocations ("update it") don't need to repeat it.

## Workflow

**The BOARD is the skill's ONLY deliverable.** If someone asks for the 'classic gantt', its
successor is the board's 'Gantt by areas' view.

```bash
# Step 1 — extract from Odoo (first time: id or text; afterwards the project is memorized)
bash <skill-dir>/scripts/refresh_gantt.sh "Muebles Romero"
# Step 2 — THE deliverable: the board
bash <skill-dir>/scripts/gen_board.sh
# Steps 1+2 on later runs (same project):
bash <skill-dir>/scripts/refresh_gantt.sh && bash <skill-dir>/scripts/gen_board.sh
```

**Step 3** — open it (`Start-Process` on Windows, platform equivalent elsewhere) and TELL THE USER
what changed vs the previous run (KPIs, new replans, deps). If the name search returns several
candidates the script lists them and exits (exit 2) — ask the user which one. ALWAYS open the
**stable dateless copy** `reports/board_<slug>.html`: F5 always shows the latest build.
The dated files (`…_<date>.html`) are history and are NEVER opened for the user — stale dated
tabs are exactly what users report as "the old version keeps appearing".

## What it renders

Five views — **Vista general · Gantt por áreas · Áreas · Pipeline · Resumen** — plus the area
sheet, the checklist popup, the Daily, the archive tray and the findings. The full contract, the
health rule, the palettes and the track-geometry traps are in
[references/board-spec.md](references/board-spec.md): read it BEFORE changing anything the user
sees.

## Data (WORKDIR/data/)

`proyecto.csv` · `etapas.csv` (sequence,id,name) · `tareas.csv` (13 cols:
id,name,stage_id,stage_name,users_ids,start,deadline,state,prio,seq,milestone,depend_ids,tag_ids —
hours were dropped in v1.1.0)
· `usuarios.csv` (id→name) · `etiquetas.csv` (id→name from project.tags) · `checklist_items.csv`
(11 cols: taskid,seq,done,weight,name,end_date,owner,client_request,state,hito,tipo —
search_read of xx.checklist.item ordered by task+sequence; the CLASSIC gantt weights the
aggregates by weight, the BOARD counts items — the weight is only a chip there;
done=`state=='done'`, end_date=`date_deadline` falling back to `date_done`, state=the Odoo
label of the `state` selection (Por hacer / En curso / En espera / Hecho) so `bof9()` keeps
matching, tipo=`type`. client_request and hito are emitted EMPTY: the new model has no
equivalent of x_studio_cliente_request nor x_studio_hito). Parsing GOTCHA: when
extracting an m2o name with `sub(/.*, "/,"",v)` you must
finish with `sub(/"$/,"",v)` or it drags the closing quote (breaks comparisons like `es=="Hecho"`).
· `partes.csv` (date,task_id,hours — account.analytic.line of the project, last 14 days
up to today; the board consumes ONLY the dates for the execution traffic light: blue <24h, green
≤3 business days, orange ≤5, red stalled)
· `hitos.csv` (deadline,name,reached) · `client_requests.csv` (task,priority,deadline,state,
active,date_sent,name — the project's x_client_request model, x_active in the domain so archived
ones come back; the board falls back to the legacy checklist char field only when the FILE is
empty, never because every request is cancelled) · `tareas_arch.csv` (id,name of archived tasks —
requests attached to them become a red finding) · `proyecto_actual` (memorized id+name).

## GOTCHAS

1. **An empty download is not "no data"**: a 200 with `[]`, an expired key or a timeout wiped the
   plan (56 tasks and 172 items -> 0) and the generator still produced an 85 KB board with
   everything at zero, **ready to show the client**. `guard()` aborts and keeps the previous
   snapshot; `gen_board.sh` refuses to run with an empty `tareas.csv`.
2. **Data reaches the browser HTML-escaped** (gotcha 10 of the long list): right for `innerHTML`,
   literal in `textContent` / `createTextNode` / `.title` — that is what `dtxt()` is for. A
   checklist item named `Defaults proveedor > usuario` showed up as `&gt;` on the client's board.
3. **The no-hours policy covers the EMBEDDED DATA too**: `TSF` carries only `[colour, last date]`;
   the hours column of `partes.csv` stays in the local CSV and never reaches the HTML.
4. **Odoo returns the stage name in the SOURCE language** while the web shows the translation
   ("To Merge" vs "Pasar a producción") -> the pipeline buckets map both.
5. **The baseline is OURS** (first snapshot) and is never overwritten on a refresh.
6. **`awk` here is mawk**, with no multibyte support: anything that truncates text must count
   characters (`clip()` with the `UTF8W` table — never `\xNN` range regexes, which gawk refuses
   to compile), and any `printf` of a decimal needs `LC_ALL=C.UTF-8` or `es_ES` emits a comma.

The sixteen originals — board geometry, `tareas.csv` columns, checklist-less tasks,
dataviz-validated palettes, the search-box focus trap, and the tombstones of what v2.0.0 retired —
in [references/gotchas.md](references/gotchas.md).

## ALWAYS

- ALWAYS memorize the project in `data/proyecto_actual` · ALWAYS list dateless tasks (never hide
  them). (The NEVERs are at the top, under **Important**.)

## Delegates to

| Skill | Suite | When | Via |
|-------|-------|------|-----|
| dynapps-brand | common | Gantt HTML identity (logo, typography, colours) | Identity embedded in the generator; `assets/dynapps_white.svg` |
| project-plan | consultant | The user wants to GENERATE or reschedule a plan | Hand off — this skill is read-only and only visualises |

## Consumed by

Nobody yet — a leaf skill the consultant invokes directly per project. Record it here **and** in
DEPENDENCIES.md if that changes.

**Shared refs**: `references/shared/dynapps-identity.md` is the SSOT for the corporate
instance facts (URL/db) this skill defaults to — still DRAFT, so the default stays hardcoded until
it is confirmed. The contract in `references/shared/odoo-connection.md` deliberately does
NOT apply to the transport: `refresh_gantt.sh` speaks `/json/2/` with plain curl instead of
odoo-pilot/MCP so it stays self-contained. Its credential rules DO apply.
