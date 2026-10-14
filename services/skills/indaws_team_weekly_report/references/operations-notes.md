# Operational notes — links, deltas, scheduling

Odoo deep-link patterns, weekly delta behaviour, how to change the format and scheduling.

- **Odoo links**: project names in the tables (per-stage tracking, go-lives and the KPI disclosure)
  link to the backend record — `<ODOO_URL>/odoo/project.project/<id>` (v19; new tab, `a.plink`
  class). The **Billing column** ("Sí · N pend." **and also "Facturado"**, in the main and go-live
  tables) links to the **ALL-milestones view** via the sale order — pattern
  `<ODOO_URL>/odoo/project.milestone/<mid>/sale.order/<oid>/<mid>/project.milestone` (v19
  breadcrumb). `team_hitos.csv` has 6 cols: id, pending, total, last, **mid** (pending milestone
  with the farthest due date, or the last one if none pending) and **oid** (order, resolved from
  `sale.order.line→order_id`, +1 light query). With HITO_ALIAS inheritance it points to the source
  project's milestones. "Sin hitos" doesn't link. Base URL from `ODOO_URL` (default
  portal.dynapps.es). Project rows in the workload matrix do NOT link (they come from
  `carga_detalle` by name, no id).
- **Weekly deltas (`+pp`)**: appear automatically from the 2nd report on (compared against the
  previous week's snapshot). The first report shows "no deltas".
- **To change the format**: edit `gen_team_report.sh` (no need to touch the data).
- **Scheduling**: for an automatic Thursday 17:00 run, schedule `team_report.sh` (Windows task
  or remote routine). Today it is **on demand**.
- The team is resolved dynamically against `crm.team` (Tokyo=28 · Mónaco=33 · Servicios=32); the
  refresh writes `data/equipo_actual` (id, name, slug) and gen/digest read it from there.
