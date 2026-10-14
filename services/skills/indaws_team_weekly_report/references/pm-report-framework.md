# Executive PM report — framework (step 3, written by the agent)

The 8-section structure, the brand styling and the writing principles for
`reports/analisis_pm_<date>.html`. This is the judgement-heavy part of the skill: read it in
full before writing, and ground every claim in `pm_digest.sh` output.

After generating the dashboard, run `pm_digest.sh` (dumps exact figures and lists) and write an
analysis with **Project Manager judgment** in `reports/analisis_pm_<date>.html`, **in the user's
language**. **Visual style: Dynapps brand** matching the dashboard — *Readex Pro* typography
(Google Fonts link), **midnight `#050543`** hero with a **gradient line**
`linear-gradient(90deg,#951CF3 0%,#FF575D 52%,#FF7759 100%)` at its foot and one headline word in
accent `#FF7E23`, white Dynapps logo (base64 of `assets/dynapps_white.svg`), light body `#F5F5FA`,
violet accents `#951CF3`, text `#050543`; the green/amber/red traffic light is semantic and stays.
Self-contained HTML.

**Fixed structure (8 sections):**
1. **Situation diagnosis** — honest read of the portfolio state (totals + traffic light), and
   identify the 2-3 underlying patterns (don't list data: interpret it). E.g.: project *closing*
   problem, *margin* erosion, date *reliability*.
2. **Urgent (decide this week)** — 2-4 red blocks with a concrete decision + suggested owner.
   Typical candidates: this month's go-lives still in To Do/0%, red projects (no hours + no billing
   but logging), and the blocked-collection case.
3. **Analysis by dimension** — 🚀 Deliveries/deadlines (above all closing near-complete overdue
   >75%), 💶 Profitability/billing (no pending billing + no hours), ⚖️ **Capacity/workload**
   (consulting vs programming matrix: individual overloads >100%, under-use, and warn that future
   months may simply be unplanned → don't read them as slack), 👥 Team (vacation×go-live conflicts,
   go-live concentration).
4. **Per-consultant situation** — ranking table by month load (digest section `FICHAS POR
   CONSULTOR`): per person, load% vs capacity, project count, hours split
   (Billable/No-billing/Internal), stuck projects under their care, and a **signal** (🔴 overloaded
   / scattered / lots of non-billable, 🟢 free…). Group 👔 consulting / 💻 programming; highlight
   who to look at first and who to reassign.
5. **Plan vs reality per project** — digest section `PLAN vs REAL POR PROYECTO`. For each team
   project **in execution** (stages En proceso / Migraciones / Salida a producción; **To Do and
   Análisis are excluded** — they count as "waiting") and **scoped to the team on both sides**:
   **real logged hours** of the 2 previous months (`account.analytic.line`, `project_id in IDS`) vs
   **planned** for the current and next 2 months (`project.project.plan.hour.line`), with the
   **Logged / plan** indicator (average real hours ÷ plan, in %: **100% = ideal**, **<100% = less
   dedication than planned**, **>100% = more**) and the **go-live date** as margin context. **Only
   planning vs logging — no budget, negative hours or billing** (those live in the dashboard and
   other sections; don't mix here). It answers "is the project getting its planned hours and is the
   plan realistic?". Two mismatches to highlight: (a) **plan ≫ real** → planned effort isn't
   happening or the plan is optimistic (🔴 ≥2× → start now or replan the date); (b) **plan ≪ real**
   → more work than planned or no plan reflected → the plan doesn't mirror reality, update it; in
   between, the **aligned** ones (plan ≈ real). Color = **plan↔real misalignment**, not the raw
   ratio. **Background fact:** the team logs most of its hours **outside the team**, which explains
   why heavily-planned projects fall short. Sources: `team_proj_plan_real.csv` (per project, main
   table) and `team_plan_real.csv` (per consultant, for the "outside the team" context).
6. **Work distribution** — digest section `REPARTO DE TRABAJO` (real hours from
   `account.analytic.line`). **Per shared project** view: for each project with 2+ consultants, the
   split of **hours across the WHOLE implementation** (project total, not just the month; with the
   month as reference) and a **concentration** indicator (top contributor %: 🔴 ≥65% bus-factor /
   🟠 50-64% with a lead / 🟢 <50% spread). The total (`team_reparto.csv` 5th col, unfiltered-date
   query) shows who really "owns" each project over its life. Highlight who carries the most real
   work, projects resting on a single person, and whether programming is already helping.
7. **Prioritized action plan** — P1 (today) / P2 (week) / P3 (month), concrete actions with owner.
8. **Watchlist** — latent risks (bottlenecks, margins, unstable dates).

**Principles:** prioritize and recommend, don't describe; every alert with an action and an owner;
ground everything in digest data (overdue, red/yellow, chronic replans, vacation conflicts); direct
project-management tone.
