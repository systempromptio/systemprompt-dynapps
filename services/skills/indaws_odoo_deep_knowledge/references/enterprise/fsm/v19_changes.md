# FSM — v18 → v19 changes

Every breaking change across `industry_fsm`, `industry_fsm_sale`, `industry_fsm_stock`, `industry_fsm_report`, `industry_fsm_sale_report`, `industry_fsm_repair`, `industry_fsm_sms`, `industry_fsm_sale_subscription` and `worksheet`. Read this before porting any custom FSM module.

---

## TL;DR — critical breakages

| Change | Impact | Severity |
|--------|--------|----------|
| `project.task.partner_phone` **se mueve a `project`** (ya no en industry_fsm) | Leer/escribir sigue funcionando — corregido, ver §17 | LOW |
| `_compute_partner_phone()` / `_inverse_partner_phone()` **removed** | Overrides break | HIGH |
| `create()` portal-user auto-assignment **removed** | Portal users no longer auto-set partner_id | MEDIUM |
| `_compute_display_timer_buttons()` → `_compute_display_timesheet_timer()` | Method renamed; overrides fail | HIGH |
| `_stop_all_timers_and_create_timesheets()` → `_stop_all_timers_and_update_timesheets()` | Renamed + signature changed | HIGH |
| `SELF_READABLE_FIELDS` → `TASK_PORTAL_READABLE_FIELDS` | Portal field whitelist renamed | MEDIUM |
| `class Task` → `class ProjectTask` | Class name change in all FSM modules | LOW (only if you reference class) |
| `allow_task_dependencies` and `allow_milestones` **removed** from `project.project` | Compute fields gone | LOW |
| `from odoo.osv import expression` no longer used | Replace with `from odoo.fields import Domain` | HIGH (Python-wide) |
| Security groups: `category_id` → `privilege_id`, `users` → `user_ids` | XML record format changed | MEDIUM |
| New `allow_geolocation` flag on project | Timer captures GPS at start | LOW (additive) |
| Worksheet QWeb generator regex-rewrites field refs in `invisible=` | Portal worksheet templates render differently | LOW |
| `web.qunit_suite_tests` → `web.assets_unit_tests` | Test bundle renamed (worksheet manifest) | LOW |
| `fsm_task_map` JS view added (additional map view alongside `fsm_my_task_map`) | Asset bundles include both | LOW |
| `industry_fsm_report` data files renamed and reorganised | XML IDs change; refs may break | MEDIUM |

---

## 1. `industry_fsm.project_task`

### Class rename
- `class Task(models.Model)` → `class ProjectTask(models.Model)` (v18:12 → v19:16). Module-level `Task` symbol is gone; if your code imports `Task` from `industry_fsm.models.project_task`, switch to `ProjectTask`.

### Fields
**Removed:**
- `partner_phone` (Char, computed + inverse) — se retira de `industry_fsm` pero **se traslada al módulo base `project`** (sigue gestionado ahí, no queda huérfano ni sin compute). Leer/escribir sigue funcionando. Ver §17.

**Added:**
- `allow_geolocation` (Boolean, related to `project_id.allow_geolocation`) at line 63.

### Methods removed
- `_compute_partner_phone()` (v18:129-131).
- `_inverse_partner_phone()` (v18:133-140).
- `_compute_display_timer_buttons()` (v18:160-173) — replaced by `_compute_display_timesheet_timer()` (v19:145-149).
- `create(vals_list)` (v18:197-202) — the portal-user auto-assignment hook. No replacement; portal user submitting an FSM task no longer auto-sets `partner_id`.

### Method renames / signature changes
- `_stop_all_timers_and_create_timesheets(tasks_running_timer_ids, timesheets_running_timer_ids, timesheets)` → `_stop_all_timers_and_update_timesheets(running_timers)`. Single argument, conceptually "any timer record" — the new method walks `parent_res_model` to figure out where to write.
- `action_timer_start()` (v18:201-214) → rewritten (v19:201-243) to capture geolocation context when `allow_geolocation=True`. Posts a chatter message including GPS coordinates.
- `_compute_is_task_phone_update()` simplified (v18:142-148 → v19:130-133): from mobile-first conditional cascade to direct `task.partner_phone != task.partner_id.phone` comparison.
- `_compute_mark_as_done_buttons()`: now uses `task.sudo().timer_start` (v19:118) instead of bare `task.timer_start` (v18:116).
- `_group_expand_user_ids_domain()` — internal `expression.OR/AND` calls replaced with `Domain.OR()` and `Domain(...)` (v19:208-214).
- `web_read(specification: Dict[str, Dict])` → `web_read(specification: dict[str, dict])` — PEP 585 type hints (v19:363).
- New method `_prepare_domains_for_all_deadlines()` (v19:508-521) — FSM mode context filtering.

### Property renames
- `SELF_READABLE_FIELDS` → `TASK_PORTAL_READABLE_FIELDS` (v18:103 → v19:105). Custom modules extending portal-readable fields must update the attribute name.

### Import changes
v18:
```python
from odoo.osv import expression
from odoo.addons.resource.models.utils import Intervals, sum_intervals
```
v19:
```python
from odoo.fields import Command, Domain
from odoo.tools.intervals import Intervals
from odoo.tools.date_utils import sum_intervals
from odoo.addons.timer.utils.timer_utils import round_time_spent
from markupsafe import Markup
from odoo.exceptions import UserError
```

---

## 2. `industry_fsm.project_project`

### Class rename
- `class Project(models.Model)` → `class ProjectProject(models.Model)` (v18:7 → v19:8).

### Fields added
- `allow_geolocation` (Boolean, `compute='_compute_allow_geolocation', store=True, readonly=False`) — `odoo_enterprise/19/industry_fsm/models/project_project.py::allow_geolocation`. Requires `is_fsm=True AND allow_timesheets=True`.

### Fields removed
- `allow_task_dependencies` (v18:13).
- `allow_milestones` (v18:14).

These were always forced `False` on FSM projects, so the v19 removal is consistent — but custom views referencing those fields fail with `ValueError`.

### Methods
**Removed:**
- `_compute_allow_task_dependencies()` (v18:32-35).
- `_compute_allow_milestones()` (v18:37-41).
- `action_view_fsm_projects_rating()` (v18:49-58) — use rating module directly.

**Added:**
- `_compute_allow_geolocation()` (v19:44-47).
- `_get_default_task_type_values()` (v19:63-77) — returns the default 5-stage config.
- `copy_data(default=None)` (v19:79-85) — handles template copying with FSM context.
- `create(vals_list)` (v19:87-101) — auto-creates task stages for new FSM projects.
- `_get_template_default_context_whitelist()` (v19:103-107) — adds `is_fsm`, `allow_material` to the project-template context whitelist.

### Constraints
v18 used a SQL string constraint `company_id_required_for_fsm_project`. v19 uses the new `models.Constraint()` syntax (v19:28-31).

### Import change
- v18 imports `from odoo.osv import expression` — gone in v19, replaced with `from odoo.fields import Command, Domain`.

---

## 3. New models in v19

### `industry_fsm.account_analytic_line`
File: `odoo_enterprise/19/industry_fsm/models/account_analytic_line.py`.

Inherits `account.analytic.line`. Methods:
- `action_timer_start()` — posts a timer-start message to the linked task when timesheet timer starts and parent is an FSM task.
- `action_timer_stop(try_to_match=False)` — posts a timer-stop message similarly.

Rationale: decouples timer chatter logging from `project.task`; the timesheet itself owns the messaging.

### `industry_fsm.rating_rating`
File: `odoo_enterprise/19/industry_fsm/models/rating_rating.py`.

Inherits `rating.rating`. Adds:
- `project_id` (Many2one, `compute='_compute_project_id', search='_search_project_id'`) — derives the project from `parent_res_model='project.project'`.

Allows FSM-specific rating reports (new `rating_rating_views.xml` in v19 manifest, line 30).

---

## 4. `worksheet.worksheet_template`

### Method changes
- `_create_record_model()` now handles `ir.model.data` registration for `x_name` (v19:161-171) when called in `install_module` context. v18 leaves `x_name` unregistered — minor orphaning risk on uninstall.
- New extension hook: `_get_<res_model>_additional_ir_rule_vals_list()` (v19:193-195) — lets subclasses inject custom IR rules per model.

### QWeb portal template generator
Lines 407-428 — significant changes:
1. Adds CSS class `o_worksheet_row` on field containers.
2. Field references inside `invisible="..."` are regex-rewritten to scope properly in portal context: `invisible="x_foo"` becomes `t-if="0 if worksheet.x_foo else 1"`.
3. Removed alternating row styling (`bg-light bg-opacity-25`) and `border-top border-bottom` classes.

### Import changes
- v18: `from odoo.tools import format_list` (used in lines 66, 75).
- v19: import removed; replaced with direct list operations.

### Manifest assets
v18:
```python
'web.report_assets_common': ['worksheet/static/src/scss/worksheet_portal.scss'],
'web.qunit_suite_tests': ['worksheet/static/tests/*.js'],
```
v19:
```python
'web.assets_frontend': ['worksheet/static/src/scss/worksheet_portal.scss'],
'web.report_assets_common': ['worksheet/static/src/scss/worksheet_portal.scss'],
'web.assets_unit_tests': ['worksheet/static/tests/*'],
```

The SCSS is now duplicated across two bundles (frontend + report) — frontend bundle covers portal templates that v18 missed.

---

## 5. Views (`industry_fsm`)

### `project_task_views.xml`
- Form gets `js_class="fsm_project_task_form"` (v19:14-16) — links to new `FsmTaskFormView` JS.
- New invisible field `allow_geolocation` after timer buttons (v19:35).
- Button visibility expressions now factor `has_template_ancestor` and `has_project_template`. Example:
  - v18: `invisible="not display_sign_report_primary"`.
  - v19: `invisible="not display_sign_report_primary or has_template_ancestor or has_project_template"`.

Same pattern for `action_preview_worksheet`, `action_send_report`, `action_fsm_validate`, and partner navigation buttons.

### Date deadline XPath consolidated
v18 had two separate xpaths (label + custom daterange widget). v19 (lines 139-160) merges them and adds repeat_interval controls for recurring FSM tasks (new repeat_interval label xpath at lines 133-135).

### Customer Preview button
- String changed: "Customer Preview" → "Preview" (v19:60).
- Wrapped in `stat_info` div (v19:65-67).

### partner_phone widget
- v18: simple field with `widget="phone" readonly="1"`.
- v19: split into two records by permission:
  - Editable: `widget="phone" groups="base.group_partner_manager"`.
  - Read-only: `widget="phone" readonly="1" groups="!base.group_partner_manager"`.

### Portal templates
`project_portal_templates.xml`:
- Timesheets section condition adds `show_portal_timesheets`: `t-if="timesheets and allow_timesheets and show_portal_timesheets"` (v19:39).

---

## 6. Security (`fsm_security.xml`)

### Group records refactored
v18:
```xml
<record id="group_fsm_user" model="res.groups">
    <field name="category_id" ref="base.module_category_services_field_service"/>
    <field name="users" eval="..."/>
</record>
```
v19:
```xml
<record id="res_groups_privilege_field_service" model="res.groups.privilege">
    ...
</record>
<record id="group_fsm_user" model="res.groups">
    <field name="sequence">10</field>
    <field name="privilege_id" ref="res_groups_privilege_field_service"/>
    <field name="user_ids" eval="..."/>
</record>
```

Changes:
- New model `res.groups.privilege` replaces `module_category` references.
- Field `users` renamed to `user_ids`.
- Removed `<record id="base.default_user">` that added FSM user to admin (v18:33-35).

### Record rule domain
- v18: `('project_id.privacy_visibility', '!=', 'followers')`.
- v19: `('project_id.privacy_visibility', 'in', ['employees', 'portal'])`.

Semantically equivalent, but the explicit list is the new pattern across Odoo 19.

---

## 7. Manifest changes

### `industry_fsm`
- Description: "worker" → "technician" (line 15).
- Data files:
  - Added: `views/rating_rating_views.xml` (line 30).
  - Added: `wizard/hr_timesheet_stop_timer_confirmation_wizard_views.xml` (line 40).
- Assets:
  - Both `fsm_my_task_map/**` (v18 path) AND `fsm_task_map/**` (new in v19) are bundled under `web.assets_backend_lazy`.
- Metadata: adds `'author': 'Odoo S.A.'` (line 74).

### `industry_fsm_report`
- Category: `'Hidden'` → `'Services/Field Service'`.
- Summary: "...for Field service workers" → "...for Field service technicians".
- Massive data file rename:
  - `project_views.xml` → `project_project_views.xml`.
  - `project_portal_templates.xml` → `project_portal_project_task_templates.xml`.
  - `worksheet_custom_report_templates.xml` → `report_industry_fsm_worksheet_custom_templates.xml`.
  - `fsm_report_data.xml` → `worksheet_template_data.xml`.
  - `wizard_load_worksheet_template_views.xml` → `worksheet_template_load_wizard_views.xml`.
  - `fsm_report_demo.xml` → split into `worksheet_template_demo.xml` + `project_task_demo.xml`.
- New files:
  - `views/worksheet_template_views.xml`.
  - `views/industry_fsm_report_menus.xml`.
  - `views/project_task_views.xml`.

Custom modules referencing the old XML IDs / file paths must update their inheritances.

### `industry_fsm_sale_report`
- Data file rename: `industry_fsm_sale_report_data.xml` → `product_product_data.xml`.

---

## 8. JS / static assets

### Removed in v19
- `static/src/components/timesheet_timer_header/timesheet_timer_header.js` — split across new components.
- `static/src/js/fsm_date_widget.js` — refactored into OWL components.
- `static/tests/fsm_date.test.js` — removed with the widget.

### Added in v19
- `static/src/services/timer_geolocation_service.js` — captures GPS at timer start/stop when `allow_geolocation=True`.
- `static/src/views/fsm_project_template_dropdown.js` — project template selector.
- `static/src/views/fsm_task_template_dropdown.js` — task template selector.
- `static/src/views/fsm_task_form/fsm_task_form_view.js` — dedicated form view controller (the `js_class="fsm_project_task_form"` target).
- `static/src/views/fsm_stop_timer_wizard/fsm_stop_timer_wizard_view.js` — UI for the new wizard model `hr.timesheet.stop.timer.confirmation.wizard`.
- `static/src/views/fsm_task_map/fsm_task_map_view.js` + `fsm_task_map_renderer.xml` — new map view (alongside the original `fsm_my_task_map`).
- Test files:
  - `static/tests/views/fsm_project_form_view.test.js`.
  - `static/tests/views/fsm_task_form_view.test.js`.
  - `static/tests/views/fsm_task_map_view.test.js`.

### Asset bundle changes
- `web.assets_backend_lazy` in v19 contains both `fsm_my_task_map/**` and `fsm_task_map/**`. Custom map view overrides should be declared in the lazy bundle.

---

## 9. Context access pattern

`self._context.get(...)` is deprecated. Replace with `self.env.context.get(...)` throughout. v19 core uses the new pattern; custom modules mixing the two still work but lint warnings will appear.

---

## 10. Migration recipe — custom FSM modules

### Step 1 — class names
```python
# Before
class MyCustomTask(models.Model):
    _inherit = "project.task"

# v19 OK — no change to _inherit. But if you import the class directly:
# v18: from odoo.addons.industry_fsm.models.project_task import Task
# v19: from odoo.addons.industry_fsm.models.project_task import ProjectTask
```

### Step 2 — remove `partner_phone` references
```xml
<!-- v18 -->
<field name="partner_phone" widget="phone"/>

<!-- v19 -->
<field name="partner_id" widget="res_partner_many2one"/>
<!-- and read phone via partner_id.phone in computed fields -->
```

### Step 3 — rename overridden methods
```python
# v18
def _compute_display_timer_buttons(self):
    super()._compute_display_timer_buttons()
    # custom logic

# v19
def _compute_display_timesheet_timer(self):
    super()._compute_display_timesheet_timer()
    # custom logic
```

### Step 4 — rewrite `_stop_all_timers_...` signature
```python
# v18
def _stop_all_timers_and_create_timesheets(self, tasks, ts, timesheets):
    super()._stop_all_timers_and_create_timesheets(tasks, ts, timesheets)
    # ...

# v19
def _stop_all_timers_and_update_timesheets(self, running_timers):
    super()._stop_all_timers_and_update_timesheets(running_timers)
    # filter by running_timers.parent_res_model if needed
```

### Step 5 — replace `expression` usage
```python
# v18
from odoo.osv import expression
domain = expression.OR([d1, d2])
domain = expression.AND([d1, d2])

# v19
from odoo.fields import Domain
domain = Domain.OR([d1, d2])
domain = Domain.AND([d1, d2])
# or
domain = Domain(d1) & Domain('is_fsm', '=', True)
```

### Step 6 — context access
```python
# v18
fsm = self._context.get('fsm_mode')

# v19
fsm = self.env.context.get('fsm_mode')
```

### Step 7 — security XML
Rename group records to use `privilege_id` and `user_ids`. If you wrote a `<record id="base.default_user">` to auto-grant FSM rights, you must replicate that in your own res.users data record now.

### Step 8 — adopt geolocation if needed
Optional. If your custom flow wants GPS:
```python
# v19 in action_timer_start override
if task.allow_geolocation:
    geo = self.env.context.get('geolocation', {})
    lat, lng = geo.get('latitude', 0), geo.get('longitude', 0)
    task.message_post(body=f"Timer started at ({lat}, {lng})")
```

### Step 9 — property rename
```python
# v18
class MyTask(models.Model):
    _inherit = "project.task"
    SELF_READABLE_FIELDS = Task.SELF_READABLE_FIELDS + ['x_my_field']

# v19
class MyTask(models.Model):
    _inherit = "project.task"
    TASK_PORTAL_READABLE_FIELDS = ProjectTask.TASK_PORTAL_READABLE_FIELDS + ['x_my_field']
```

### Step 10 — file path renames (industry_fsm_report)
If your inherits reference `industry_fsm_report.project_views`, update to `industry_fsm_report.project_project_views`. Similar for the renamed templates listed in §7.

---

## 11. Database migration considerations

- The `partner_phone` column in `project_task` is **not dropped** by v19. Old data remains queryable via SQL but no longer maintained by the ORM. If you want to drop it: `ALTER TABLE project_task DROP COLUMN partner_phone;` after confirming no view references remain.
- Security group migration is handled by Odoo via `res.groups.privilege` records — verify after install that `fsm_user` / `fsm_manager` privilege records exist in your DB.
- Worksheet templates: existing dynamic `x_*` models are preserved; only the `x_name` `ir.model.data` registration changes. Existing data is safe.

---

## 12. Effort estimate

For a typical custom FSM module with ~500 lines of Python + 10 views:
- **2–4 hours** of refactoring if changes are mechanical (renames + import shifts).
- **4–8 hours** if the module heavily extends timer or worksheet (deeper testing of timer geolocation, signature flow, recurring tasks).
- **8–16 hours** if security XML / data XML IDs were extensively customised (privilege model + renamed file references).

---

## 13. Cross-references

- Class-level overview of core changes → [[core]].
- v19 changes in worksheet engine (`x_name` registration, regex prefix rewriting) → [[worksheet]].
- v19 JS removals / additions → [[mobile]].
- Backwards-compat gotchas in custom FSM modules → [[gotchas]].


<!-- ===== Hallazgos verificados vs fuente v19 (2026-07-14, adversarial) — renumerado 2026-07-14 ===== -->

## 14. Wizards v19: `project.task.create.timesheet` ELIMINADO

`project.task.create.timesheet` (`project_task_create_timesheet.py`) ELIMINADO. Nuevos: (a) `base_document_layout.py` — override de `base.document.layout` para preview del worksheet cuando `active_model=project.task` (template `industry_fsm.worksheet_custom_preview`); (b) `hr_timesheet_stop_timer_confirmation_wizard.py` — hereda `hr.timesheet.stop.timer.confirmation.wizard` (base en `timesheet_grid`); su `action_save_timesheet` emite el chatter "Timer stopped at" + coordenadas GPS si `allow_geolocation`. `task_stop_timer_confirmation_wizard.py`: clases renombradas a `ProjectTaskStopTimersWizard(.Line)`.

## 15. Validación dura v19: `_stop_all_timers_and_update_timesheets`

v19 introduce una validación dura: `_stop_all_timers_and_update_timesheets` lanza `UserError("All running timers should be linked to a timesheet and a task.")` si algún timer en marcha no tiene `res_model=account.analytic.line` (o `parent_res_model != project.task`). Flujos custom que arranquen timers sobre otros modelos y luego validen la tarea FSM revientan. No existía en v18.

## 16. `res_config_settings` v19: `group_industry_fsm_quotations` eliminado [PARTIAL]

Se ELIMINA el campo `group_industry_fsm_quotations` (toggle "Extra Quotations", implied_group `industry_fsm.group_fsm_quotation_from_task`) junto con sus computes `_compute_group_industry_fsm_quotations`/`_compute_module_industry_fsm_sale` y el override `_get_basic_project_domain`. `module_industry_fsm_sale` deja de ser computed (queda store + readonly=False). Custom que dependa del SETTING o del compute rompe. **OJO:** el `res.groups` `group_fsm_quotation_from_task` NO se elimina (sigue en `industry_fsm/security/fsm_security.xml:28`, `group_fsm_quotation_from_task`); las referencias a ESE GRUPO no fallan — lo que desaparece es el toggle de ajustes que lo activaba.

## 17. Corrección de TL;DR / §1 / §11 — `partner_phone` NO se elimina (se mueve a `project`) [LOW]

`partner_phone` ya NO se define en `industry_fsm` (se retira su compute+inverse) pero el campo **SE MUEVE al módulo base `project`** (`project/models/project_task.py:228`, `ProjectTask.partner_phone`, v19, con `_compute_partner_phone` + `_inverse_partner_phone`). **Leer/escribir `task.partner_phone` SIGUE funcionando en v19 → severidad LOW, no HIGH.** El write-back a `partner_id.phone` NO desaparece: el inverse persiste en `project`, solo se simplifica (ya no aplica el cascade mobile-first, escribe solo `phone`). El campo se sigue consumiendo en `industry_fsm._compute_is_task_phone_update` y `TASK_PORTAL_READABLE_FIELDS`. Lo que SÍ rompe: overrides de `_compute_partner_phone`/`_inverse_partner_phone` que estuvieran enganchados en `industry_fsm` (ver §1 "Methods removed").

