# Dashboard format (frozen)

The exact output contract of `gen_team_report.sh`: the blocks below, their tooltips and
disclosures, the risk colouring, and how capacity (TEO) is synced. **The format is frozen** —
read this before changing anything the user sees, and see the discarded-UI gotcha in SKILL.md
before reintroducing an idea.

1. **Header KPIs**. The **"Vencidos a producción"** (overdue go-lives) and **"Sin fact. pendiente"**
   (no pending billing) KPIs carry a **hover tooltip** (overdue ones with their date; `cursor:help`,
   no ⓘ icons — tried and removed), and right under the KPI row there is a **collapsed disclosure**
   (`<details class="kdet">`, "Ver proyectos — click to expand") with the **two full lists as
   tables**: overdue (date·project·manager·stage, most overdue first) and no-billing
   (project·manager·stage·available hours, red if ≤0). (A visible list inside the KPI card was
   tried and discarded — took too much space.)
2. **Billing summary box**: projects and hours (week and month) with/without pending billing
   (delivery stages only, excl. To Do and Análisis). MIND the two different windows (each card has
   a clarifying tooltip): the WEEK is a ROLLING last-7-days window (label dd/mm-dd/mm COMPUTED --
   never hardcode it; it once froze at "28/05-04/06") and may cross the month boundary; the MONTH
   is natural (1st -> today) -- early in the month week > month is normal. **Tooltips with the project list** on: bar
   segments, the Con/Sin facturación legends (dotted underline), and the red **projects without
   hours** line (with their logged hours). Lists are collected while counting (`addl()`, separator
   `&#10;` = newline in native title).
3. **Overdue unpaid billing** — "Por cobrar" KPI (TOTAL pending € in red, per-project tooltip;
   the KPI keeps the full total) + **disclosure collapsed by default**, stacked **right below the
   KPI "Ver proyectos" disclosure**, showing ONLY OVERDUE invoices (due < today; the rest are not
   listed), **sorted by oldest due date** and coloured **amber <=30 days overdue · red ⚠ >30 days**
   (exact days in the tooltip): manager, overdue invoice count (tooltip with invoice·amount·due
   detail), overdue amount, oldest due date and order link (`/odoo/sale.order/<oid>`). **Excludes invoices with a "Kit" payment term** (Kit digital / Kit
   Consulting — paid by red.es, not client delinquency; `tolower(term)~/kit/` filter in the
   refresh). Data from the milestones→order→invoices chain (`team_cobros.csv`; +3 light queries).
   Projects with milestone inheritance (alias) show their collection under the source project. The
   KPI total deduplicates invoices shared across projects.
3b. **Upsellings** — collapsed disclosure below the collections one: OPEN CRM opportunities of the
   team (crm.lead type opportunity, active and NOT won — filtered out in gen/digest by
   `won_status != pending`, the CSV stays raw), **sorted by expected close date asc** (dateless
   last), columns
   Opportunity (link /odoo/crm.lead/<id>) · Client · Salesperson · Stage · Expected revenue ·
   Expected close (NO probability column — the user removed it). Summary shows count + total € +
   "N cerrada(s) este mes" when any. **"Cerradas este mes" subsection** inside the same disclosure:
   won (green chip) and lost (red chip) read off **`won_status`** from `team_upsell_closed.csv` —
   if the month has no closes the subsection is not rendered (normal early in the month). GOTCHA:
   closes are detected via `date_closed` (NOT date_deadline, which is the EXPECTED close); lost ones
   are archived -> the domain needs the active true/false OR. **Never classify won/lost by stage
   NAME or by "archived = lost"** — see the `won_status` note in `data-layout.md` for why both
   heuristics were wrong and what the fallback is.
4. **Go-lives — next 2 months** (current month + next): go-live table + **per-person
   mini-calendar** spanning 2 months, now in a **collapsed disclosure** whose summary shows people
   count and how many are on 2+ go-lives (axis with dd/mm dates; counts **manager + consultants**
   of each project, merging name variants; markers with the day and ×N when they collide;
   highlights in red anyone involved in 2+ go-lives). Each **table** row is **colored by type** from a word in
   the title: **Migración** purple · **Implantación** green · **Análisis** blue · **Rescue Pack**
   gray (priority in that reverse order: Implantación beats Análisis when the title has both).
4b. **8-month go-live bar chart** — collapsed disclosure under the go-lives section, with TWO VIEWS
   and a toggle button (👥 Ver por consultor / 📊 Ver simple): the DEFAULT is the SIMPLE view
   (plain violet bars, current month orange, count on top, tooltip with the month's projects); the
   button switches to the PER-CONSULTANT view where each go-live is a block split into VERTICAL
   STRIPES — the first stripe is the RESPONSIBLE (double width, always leftmost) and the thin ones
   the consultants — with a block tooltip project·date·responsible·team. Blocks stack in an
   **AFFINITY CHAIN**: each block sits next to the one sharing most PEOPLE (whole team, not just
   the responsible); ties -> the candidate with fewest links in the month (so single-link blocks
   don't dangle), then best shared rank, then date; no link -> new group by rank. Colors come from
   a **curated 12-color palette** assigned by involvement order (namehue() collided: Jesus~Rafa,
   Dani~Angel), with name-variant MERGING (same first name + a >=4-letter surname contained in the
   other's full name, e.g. "Angel Angelini" = "Angel Eduardo Angelini") and a legend in involvement
   order with each person's go-live count. Bar height still counts go-lives (no double counting).
5. **Per-stage table** (foldable groups; **all collapsed by default** — click the stage header to
   expand) with: manager (photo + color by name), consultants (small gray chips),
   Calendar/Hours/Checklists bars, week hours, **available hours (`Disp.`)**, billing, go-live date
   with replan indicator, status, and risk highlighting: **yellow** (not finished + no pending
   billing, with hours) or **red** (same but also no available hours).
6. **Executive summary & team** (closing banner): portfolio, production, billing, replanning;
   **team Workload matrix** (6 months: current-month capacity column vs planned hours per month
   from `project.project.plan.hour.line`; load heatmap 🟢<85% 🟠85-100% 🔴>100%; only 👔 Consulting
   (the 💻 Programming team is **excluded**). Each **person is foldable** carga-consultores-style:
   clicking their row (toggle `tgc()`, arrow ▸/▾) expands **their projects per month** in hidden
   rows, **ordered by current-month load** and with the name **highlighted in blue** when they have
   hours this month (`.cprj.cnow`); each person cell has a **tooltip with the theoretical/real
   logic**: `teo · real · disp.teo · disp.real` + the month's projects; and in the **expanded
   project rows**, the **current-month** cell has a tooltip **"plan X h · real logged Y h"** (real
   from `team_paralelo.csv` matched by employee+project name) — where **real = workdays×8 −
   vacations** (`data/team_vac6m.csv`; no public holidays yet) and the group row aggregates the
   team's real ones); **Parallel projects per consultant** (below the workload matrix, **disclosure
   collapsed by default** — the summary carries the average and the count in red): number of
   projects with **≥1 real hour logged this month** (from any team — query [11b] →
   `team_paralelo.csv`; **excludes internal ones**: names starting with "Interno" / "PM " (any
   team) / "Consultoría" as a single word — the CSV stays **raw** (all projects, because it also
   feeds the matrix plan-vs-real hover) and the `isint()` filter is applied by the consumers: gen
   and digest counts), traffic light 🟢≤6 · 🟠7–10 · 🔴≥11, **h/project** (fragmentation), **Plan**
   (projects with planned hours this month, for the real-vs-plan contrast) and top-3 projects with
   full-detail tooltip. The table carries the **3 previous FULL months** (from
   `team_paralelo_m1/2/3.csv`, one natural month each) as dimmed pills with the same traffic light
   and per-month tooltips (count · h/project · full list), then a **"Media 3m" column** — the
   consultant's average over their closed months with data, rendered as a PLAIN BOLD NUMBER with
   no pill/colors (with colors it read as another month), then the **CURRENT month column FRAMED**
   in light violet (bg #FAF5FF + #D9C2F9 border closed on all four sides). The disclosure summary
   shows the **average over the 3 CLOSED months** (the in-progress month would sink it until month
   end) and the red count = people with closed-months average >=11. Criterion: parallelism is measured on **logged** work (real dispersion),
   not on the plan; the real↔plan gap is the signal (real≫plan = unplanned fronts; plan≫real =
   commitments not started). **Vacation gantt redesigned as a real calendar**: shared overlay on
   every row with weekend shading + a thin line on each Monday + a stronger month separator
   (`.gwe/.gwl/.gml`), a TWO-LEVEL axis (month names bold on their own upper line, Mondays dd/mm
   below so they never collide; NO "HOY" chip — the user removed it; the window starts today),
   rows **ordered by days out** (total next to the name), pending-approval absences in amber with
   a dashed outline, tooltips with person·dates·days·absence type·state, and the go-live ▼ row
   INSIDE the table (a separate strip above was tried and reverted: "uglier") plus the red dashed
   go-live lines across rows; and a conflicts block (consultant on vacation around a go-live of
   their project).

**Capacity (TEO):** **auto-synced on every refresh** (step [9b]) from the *Dashboard CONTROL* sheet
(doc 5800, EMPLEADOS sheet, `spreadsheet_snapshot` field base64→JSON): team titles in col A
(TOKYO/MÓNACO/SERVICIOS), CONSULTORES/DESARROLLADORES markers in col B, id in A, name in B, TEO in
col D. `data/team_capacidad.csv` is written with the CURRENT TEAM's block (col A title normalized
without accents). **The sheet is the single source**: to change a TEO, change it in the sheet and
rerun the skill. Failure guard: if the extraction returns <3 rows (or the team has no block in the
sheet), the previous CSV is kept and a warning is printed. (It used to be a manual CSV and diverged
twice — Aitana 65→125, Claudia 125→100.)
