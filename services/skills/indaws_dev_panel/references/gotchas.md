# dev-panel — gotchas

Todos mordieron de verdad durante la construccion del panel con datos reales. Viven aqui y no en
`SKILL.md` porque son diagnostico: se leen cuando algo no cuadra, no en cada activacion.

## Los once

1. **Absences are logged into the FUTURE** (vacation lines dated months ahead) → the timesheet
   window must ALWAYS be `date <= today`, or "latest activity" becomes January vacations.
2. Absence = task or project whose name contains "Ausencia" → split from work hours (own chip in
   the dev card, own aggregate in the KPI).
3. **`LC_ALL=C.UTF-8` before the gen awk** — necessary, and NOT sufficient. It only changes
   anything if the `awk` in the PATH is **gawk**; on Debian/Ubuntu `awk` is **mawk 1.3.4**, which
   has no multibyte support whatsoever, so `substr()` keeps cutting by BYTES and splitting
   accented characters on truncation → invalid UTF-8 in the HTML.
   *Measured 2026-09-02 on this very panel with live data: the file failed to decode at byte
   444275, `...renombrar el m\xc3</a>` — a project name truncated at 46 in the middle of the `ó`.
   The gotcha said the problem was fixed; it was fixed only where gawk happened to be installed,
   and it was not installed on the machine running the review.*
   The generator now resolves `AWK` to gawk when it exists, and otherwise truncates with
   `clip(s,n)`, which walks the UTF-8 lead bytes and never leaves a sequence half-written.
   The same trap hits `inic()`: `substr(name,1,1)` on "Ángel" took HALF a character.
4. A "Hecho"-named stage with state `04_waiting_normal` lands in Bloqueado on purpose: it is
   inconsistent Odoo data and the panel surfaces it (the gray stage chip gives it away).
5. `hr.department` of Development = **4** (DEV_DEPT). A dev's team is the employee's
   `x_studio_team_id` (Tokyo/Mónaco/Servicios/Programación), NOT crm.team.member_ids.
6. "Stalled" is a heuristic (no timesheet ≥5 business days) — a legitimately parked task is a
   false positive; verify before chasing anyone.
7. Timesheet lines may have NO task (task_id=0: coffee breaks, meetings) → grouped by
   project+description in the dev card; they do not feed the kanban hours.
8. **Assignee ≠ planning user**: the owner of a dev task is `x_studio_plan_user_id` ("Usuario
   planificación", the same field team-task-assigner plans with), not `user_ids` (assignees
   usually include consultants/PMs). With user_ids only 126 tasks surfaced; with plan_user, 297
   (+173 unplanned). Card initials and the dev filter use plan_user.
9. **STAGE TRANSLATIONS (bit hard)**: the JSON2 API returns the stage's SOURCE name (e.g.
   "To Merge") while the Spanish web UI shows its TRANSLATION ("Pasar a producción") — the user
   sees a name the API "cannot find" (`stage_id.name ilike "pasar"` returned 0 with 7 tasks on
   screen). Diagnosis: locate ONE concrete task from the user's view (by name or by
   `date_last_stage_update`, which is the "stage + datetime" column in Odoo lists) and read its
   real stage_id; then map BOTH names in bucket(). `x_studio_review_priority` is a Selection
   stored as a STRING ("3"…"0") — filter with strings, never integers.
10. **JS filter-chip family**: every `.fb` button WITHOUT `data-f` (group/subgroup/clear buttons)
    used to crash paint() on `f.startsWith` — paint() must iterate `.fb[data-f]` only.
11. Stage names are heterogeneous per project; `bucket()` normalizes (lowercase, accent-stripped)
    and maps: finished = hecho|done|realizado|completado|cerrado|produccion + cancel (OUT of the
    panel); deploy queue = `pasar a produccion|subir a produccion|to merge` (checked BEFORE the
    finished rule); testeo = review|testeo|test|qa|validacion; blocked = espera|bloquead|standby|
    pendiente cliente or state 04_waiting_normal; a planificar = a planificar|sin asignar|nuevo|
    pendiente|inicio|tareas base|backlog|no stage; anything else = En proceso (the gray stage chip
    on the card exposes mis-mapped names — extend bucket() when new ones appear).

12. **"In process with no timesheet since entering the stage" MUST be guarded by the download
    window.** The rule (requested by the user: leave such tasks blank — the dev has not started
    in this stage yet) compares the last timesheet date with `date_last_stage_update`. Timesheets
    are only downloaded 14 days back, so "no timesheet since stage entry" is UNDECIDABLE for
    tasks that entered the stage earlier than the window: the first, unguarded version blanked
    **87% of the panel and left zero stalled tasks**. Only blank when
    `stage_entry >= window_start`; otherwise the classic traffic light applies (a task long in
    stage with no recent timesheets IS stalled). Card shows a "sin partes en la etapa" chip.
13. **The team filter silently hid every unassigned task** (no plan user -> no team). Real case:
    Rotger's whole incoming queue disappeared under the Tokyo filter. The generator now runs a
    pre-pass computing each project's dominant team (majority team among its ASSIGNED tasks) and
    attributes it to the project's unassigned tasks, with a "sin asignar" chip + tooltip. A
    project with NO assigned tasks stays team-less (visible only under "Todos") — no inference
    is honest there. Also mind bash: `UID` is a readonly shell builtin — use another name when
    scripting checks.
14. **gawk PARSES the whole program even for branches that never run.** clip()'s byte-walk is
    mawk-only (`BYTES=1`), but its original byte-range regexes (`/[\xf0-\xf7]/` etc.) made gawk
    abort at COMPILE time under `LC_ALL=C.UTF-8` — the panel died on every gawk machine
    (Windows git-bash included) while working on mawk. Byte classes for the mawk path must not
    be spelled as regex ranges: clip() now uses a `UTF8W[]` width table built with
    `sprintf("%c",i)` inside `if(BYTES)` — parse-safe everywhere, byte-correct under mawk.

15. **The Odoo 19 migration unified every task-stage catalog into English** (New, Analysis,
    Implementation, Test, Acceptance, Release, Done) — none of the old Spanish names survive on
    migrated projects, and a bucket regex that does not know them drops EVERYTHING into En proceso
    (kanban read 0/357/6/0, deploy queue empty). The mapping lives in the stage -> column table of
    `rendering.md`, and **project-gantt implements the same table in JS (`tbk9`)**: change one,
    change the other — the first version of this fix reached only one of them. Release MERGED the
    old "Pasar a producción" queue AND the old "En Producción" stage: the deploy queue jumps (122
    on migration day) because uploads whose state was never closed now live there — real hygiene
    debt surfaced, not a bug. Stage Done can still carry an open state (165 tasks on migration
    day); it stays off.
