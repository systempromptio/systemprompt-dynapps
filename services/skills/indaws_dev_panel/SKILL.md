---
name: indaws-dev-panel
description: >-
  Read-only HTML panel of the DEVELOPMENT team from Odoo: kanban of the open tasks tagged
  "Desarrollo" by PLANNING USER (x_studio_plan_user_id) or unplanned, with a timesheet-driven
  EXECUTION TRAFFIC LIGHT per card, a priority x execution matrix, a "Pasar a produccion" deploy
  queue, a per-developer week with an agent-written reading, and project-grouped signals. Use it
  when the team's work in flight is the question, not one project's plan. Triggers: "panel de
  desarrollo", "dev panel", "que estan haciendo los programadores", "carga de los devs",
  "actualiza el panel dev", "what are the devs working on", "dev workload", "update the dev
  panel". NOT one project's Gantt (project-gantt) and NOT replanning the week
  (team-task-assigner): this skill only shows.
version: 1.3.0
effort: medium
model: sonnet
allowed-tools: [Read, Write, Bash, Glob, Grep]
---

# Dev Panel — dev-team tasks and load

## Important

- **NEVER write to Odoo** — 100% read-only (the only writes are local files in the WORKDIR).
- Credentials ONLY via the `ODOO_KEY` environment variable or `config.local.sh` in the WORKDIR.
- **Output language**: chat summaries and the per-dev narratives MUST be in the user's language.
  The generated HTML renders its labels in Spanish (the team's working language) — the known
  limitation tracked in the suite's IDEAS.md.

## Setup (first run per user)

Plugin folder = READ-ONLY cache. All state (data/, reports/, credentials, narrativas.tsv) lives in
the **WORKDIR**: `$HOME/dynapps-informes/dev-panel/` (override with `DEV_PANEL_DIR`). On first run,
if the credential is missing the script stops asking for `<WORKDIR>/config.local.sh` with
`ODOO_KEY="..."` — ask the user for their Odoo API key (portal.dynapps.es → User settings → API
Keys) and create it. `DEV_DEPT` (default 4) and `DEV_TAG` (default "Desarrollo") are configurable
in config.sh/env.

## Workflow

```bash
bash <skill-dir>/scripts/refresh_dev.sh       # 1. Odoo -> WORKDIR/data (devs, tasks, timesheets 14d)
# 2. THE AGENT WRITES <WORKDIR>/data/narrativas.tsv (see below) from the fresh data
bash <skill-dir>/scripts/gen_dev_report.sh    # 3. -> WORKDIR/reports/panel_dev_<date>.html
```

**Step 2 — narratives (MANDATORY after every refresh)**: write `data/narrativas.tsv` (one line per
dev: `uid<TAB>text`, no tabs/newlines inside the text) with **2-3 quality sentences** per
developer: what they are really on beyond the raw timesheet — the week's focus, notable states
(go-live, PRs in review), anomalies in a FACTUAL, actionable tone ("1h coming from 49h — check
whether hours are missing"), never performance judgement. Sources: `partes.csv` descriptions
(rich) + their tasks in `tareas_dev.csv`. Injected UNESCAPED (agent-authored, local). A dev with
no line simply shows no reading.

**Step 3** — open the HTML (`Start-Process` on Windows) and TELL THE USER what stands out:
blocked, stalled, deploy queue, devs with anomalous load. Same-day regenerations overwrite the
file (F5); a new day is a new file.

**Both scripts fail LOUDLY instead of rendering something false.** `refresh_dev.sh` writes to a
private `mktemp -d` and only promotes a download that came back non-empty; anything else aborts
with exit != 0, **keeps the previous snapshot** and does NOT reseal `data/ventana`, which is the
mark of a complete snapshot. `gen_dev_report.sh` refuses to run without it. If `narrativas.tsv`
is older than that mark, the panel still renders but carries a visible warning strip saying the
per-dev readings do not belong to these numbers — a stale reading passed off as current is worse
than no reading. A download that hits its `limit` warns that the panel is under-reporting.

## What it renders

Hero + KPIs · filter chips (team <-> dev, "listo a planificar", "Aceptación") and an
accent-insensitive project search · collapsed pipeline summary · **priority x execution matrix**
(clickable) · **4-column kanban** (A planificar · En proceso · Testeo/review · Bloqueado) with the
traffic light as left border and tooltip ·
grouping views (state / deadline / project) and in-pipeline subgroups (none / project / **plan
user** / deadline / priority) · clickable aggregates opening a modal task list with direct Odoo
links, its own **sticky header** with search and **stackable grouping pills** (activation order =
nesting, level badge per pill) · the **"Pasar a produccion"** deploy queue outside the pipeline ·
per-developer week with the agent reading · project-grouped signals. Block-by-block inventory,
palette, interaction rules and the stage -> column table: `references/rendering.md`.

## Data (WORKDIR/data/)

`devs.csv` (emp_id, uid, name, team — active employees of DEV_DEPT) · `tareas_dev.csv` (11 cols:
id, name, project, stage, state, plan_uid, deadline, priority, write_date, review_priority,
stage_entry_date (`date_last_stage_update` cut to date) —
tasks tagged DEV_TAG whose `x_studio_plan_user_id` is a dev OR EMPTY — the incoming queue —, state ≠ done/canceled) ·
`partes.csv` (date, emp, task_id, task, project, hours, description — account.analytic.line of
DEV_DEPT, 14 days UP TO TODAY) · `narrativas.tsv` (agent-written) · `ventana` (today, from, monday).

## GOTCHAS

1. **Absences are logged into the FUTURE** (vacation lines dated months ahead) -> the timesheet
   window must ALWAYS be `date <= today`, or "latest activity" becomes January vacations.
2. **Absence is a FIELD, not a name**: `holiday_id` / `global_leave_id` on the timesheet line.
   Matching "ausencia" in the TASK name turned every HR-module development task into holiday --
   we build Odoo, those task names exist. Project name only, as a fallback for manual lines.
3. **An empty download is not "no data"**: a 200 with `[]`, an expired key or a timeout used to
   overwrite the CSVs with nothing, and the panel rendered "0 tasks · 0 h" -- indistinguishable
   from a quiet week. `guard()` aborts and keeps the previous snapshot.
4. **`awk` may be MAWK (no multibyte support)**: `substr()` cuts by bytes and splits accents even
   with `LC_ALL=C.UTF-8`. The generator uses `gawk` when present, else `clip()` counts characters;
   route any truncation through them.
5. **Assignee != planning user**: a dev task belongs to `x_studio_plan_user_id`, not `user_ids`.
   Con `user_ids` solo salian 126 tareas; con plan_user, 297 (+173 sin planificar).
6. **Unassigned tasks have no team**: the team filter used to hide a project's incoming queue.
   The generator infers the team of unassigned tasks from the project's dominant team (majority
   of its assigned tasks) and marks the card "sin asignar" — see gotchas 12-13.
7. **In-process without timesheets since a RECENT stage entry = blank light** (not judged yet);
   the recency guard against the 14-day window is load-bearing — see gotcha 12.

Los otros siete -- stage TRANSLATIONS (the API returns "To Merge", the UI shows "Pasar a
produccion"), `x_studio_review_priority` como STRING, el mapeo de `bucket()`, las lineas de parte
sin tarea, la trampa `.fb[data-f]`, la etapa "Hecho" con estado `04_waiting_normal` y **gawk
parseando ramas que no ejecuta** -- con su diagnostico, en `references/gotchas.md`.

## Odoo access

Odoo access follows
`references/shared/odoo-connection.md` (JSON2 + API key, read-only); the transport is not
reimplemented here.

## ALWAYS / NEVER

- ALWAYS cap timesheets at `date <= today` · ALWAYS write the narratives after a refresh · ALWAYS
  keep finished stages out of the kanban (counter in the footer) · NEVER write to Odoo · NEVER
  overwrite config.local.sh.

## Delegates to

| Skill | Suite | When | Via |
|-------|-------|------|-----|
| dynapps-brand | common | Panel HTML identity (palette, typography) | Identity embedded in the generator |
| team-task-assigner | common | The TL wants to ACT on the findings (replan/assign the week) | Hand off — this skill only visualizes |
| standup-facilitator | common | Daily/weekly standup agenda from the same team | Hand off |

## Consumed by

Nobody yet — a leaf skill the TL/PM invokes directly. Record it here **and** in DEPENDENCIES.md
if that changes.

## REFERENCES

- `references/gotchas.md` -- the fourteen, with their diagnosis
- `references/rendering.md` -- block-by-block inventory, palette, interactions
- `references/shared/odoo-connection.md` -- Odoo transport (JSON2, API key)
