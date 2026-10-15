# Data layout — scripts, WORKDIR and every CSV

What each script writes and what every column of every CSV means. Read it before touching the
extraction or when a figure in the dashboard needs tracing back to its source.

```
<skill-dir>/scripts/       (read-only plugin code)
  team_report.sh           Wrapper: refresh + generation + open.
  refresh_team_data.sh     Extraction: JSON2 queries to Odoo (curl) → writes data/ and fotos/.
  gen_team_report.sh       Format: reads data/ and produces reports/seguimiento_<team>_<date>.html.
  pm_digest.sh             Analytical digest (text) for the PM analysis.
<WORKDIR>/                 (per-user state)
  data/                    generated CSVs:
      team_projects.csv      projects (stage, dates, hours, checklists, manager)
      team_consultores.csv   id → consultants (x_studio_consultores)
      team_hitos.csv         id → billing milestones pending/total/last + farthest pending milestone
                             ID (project.milestone; feeds the Billing column link)
      team_replan.csv        id → replan count, original date, drift (mail.tracking.value)
      team_vacaciones.csv    team absences, 2 months ahead (hr.leave)
      team_horas_mes.csv     id → hours logged this month
      team_fotos_map.csv     manager name → employee id
      team_capacidad.csv     monthly capacity (TEO) per consultant/dev — AUTO-SYNCED from the
                             TEAM's block in the Dashboard CONTROL sheet (doc 5800)
      team_carga.csv         planned hours of the month per employee — project.project.plan.hour.line
      team_carga_detalle.csv planned per employee/month/project (matrix tooltips and cards)
      team_reparto.csv       real hours per project+consultant (week/month/total) — account.analytic.line
      team_vac6m.csv         absences 6 months ahead per employee ID (hr.leave) — real capacity of
                             the workload matrix
      team_cobros.csv        posted UNPAID customer invoices per project (pid, invoice id/name,
                             residual, due, state, order) — milestones→sale.order→account.move chain
                             (not_paid/partial, posted)
      team_plan_real.csv     team plan vs real logged per CONSULTANT/month (real −2/−1/current,
                             plan current/+1/+2) — "outside the team" context for §5
      team_proj_plan_real.csv team plan vs real logged per PROJECT/month (same ranges) — main table
                             of §5 of the PM analysis
  fotos/                   employee photos (<id>.png)
  snapshots/               snap_<team>_<date>.csv — weekly snapshot PER TEAM (the +pp deltas only
                           compare within the same team)
  reports/                 seguimiento_<team>_<date>.html — output
```

Script paths resolve relative to the script itself, so the WORKDIR is independent from the plugin
cache: skill updates never touch your data/snapshots.

## Upsell CSVs (CRM opportunities)

- `team_upsell.csv` — id, name, client, expected_revenue, probability, stage, salesperson,
  date_deadline, **won_status**. Active opportunities of the team (crm.lead type opportunity);
  WON ones are filtered by the consumers (gen/digest) — the CSV stays raw.
- `team_upsell_closed.csv` — same + date_closed + active flag + **won_status** (col 10).
  Opportunities CLOSED in the current natural month: won/lost read straight off `won_status`
  (the query still needs the active true/false OR, because lost leads are archived and
  `search_read` hides them by default). Detected via `date_closed`, NOT date_deadline (that is
  the EXPECTED close).

**Won/lost comes from `won_status`, never from the stage NAME.** `crm.lead.won_status` is a
stored computed Selection won/lost/pending — `probability == 100 and stage_id.is_won` → won,
`not active and probability == 0` → lost, else pending
(`crm/models/crm_lead.py:613-620`, `CrmLead._compute_won_status`, v19 @d1d161033; the flag it reads is
`crm.stage.is_won`, `crm/models/crm_stage.py:27`, `CrmStage.is_won`, v19 @d1d161033). It replaced two heuristics
that were both wrong at the edges: matching `/won|ganad/` against the stage name (a renamed stage
like "Adjudicado", or another UI language, walked straight through) and treating **every** archived
lead as lost in the closed CSV (a lead archived while still in progress was counted as a lost deal).
Consumers keep a fallback to the old heuristic ONLY when the column is absent, i.e. a `data/` left
over from a refresh older than v2.3.0.

`won_status` is **Odoo 19+** (it does not exist in 18.0). That is fine here because
`refresh_team_data.sh` talks to the corporate portal exclusively over `/json/2/`, which is itself
19-only. If this script ever has to hit an 18.0 instance, the portable equivalent is a second query
for `crm.stage` ids with `is_won = true` and a per-row lookup — `is_won` does exist in 18.0
(`crm/models/crm_stage.py:39`, `Stage.is_won`, v18 @99f5a33c64).

## Parallels trend CSVs

- `team_paralelo.csv` — current month: employee, project id, project name, real hours (raw, all
  projects incl. internal; consumers apply `isint()`).
- `team_paralelo_m1/2/3.csv` — the 3 previous FULL months (one natural month each, same shape),
  fetched in a loop in step [11b]; they feed the trend columns and the closed-months averages of
  the parallels block.
