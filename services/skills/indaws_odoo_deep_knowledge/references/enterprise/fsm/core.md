<!-- verified: v18.0 @ 2026-06 -->
# FSM — Core (`industry_fsm`)

Module that turns a `project.project` into a Field Service operation. Adds technician-oriented kanban/map/gantt, timer, on-site validation flow, customer signature, worksheet templates, partner navigation, recurring interventions and portal sign-off.

**Source:** `odoo_enterprise/18/industry_fsm/` (1854 py lines + JS + views).
**Auto-installed by:** `industry_fsm_sale`, `industry_fsm_stock`, `industry_fsm_report`, `helpdesk_fsm`, `industry_fsm_sms`, `industry_fsm_sale_subscription`, `industry_fsm_sale_report`, `industry_fsm_repair`.

---

## 1. `project.project` extension

Source: `industry_fsm/models/project_project.py`.

### Activation flag
- `is_fsm` (Boolean) — turns the project into a FSM operation.
- **SQL constraint** `company_id_required_for_fsm_project`: `is_fsm=True` requires `company_id NOT NULL`. **No multi-company FSM projects.** Forgetting `company_id` raises a cryptic SQL error, not a friendly ValidationError.

### Cascading toggles (compute fields)
Toggles auto-driven from `is_fsm` and inverse-writable (silent overrides — no warning on user input):
- `allow_task_dependencies` → forced **False** when `is_fsm=True` (v18 only; field removed in v19, see [[v19_changes]]).
- `allow_milestones` → forced **False** when `is_fsm=True` (v18 only; removed in v19).
- `allow_quotations` (added by `industry_fsm_sale`) → gated by group `industry_fsm.group_fsm_quotation_from_task`.
- `allow_billable`, `allow_material`, `allow_worksheets` — see [[extensions_sale]] and [[worksheet]].

### Auto-created stages
On `create()`, FSM projects get 5 default `project.task.type` records: **New → Planned → In Progress → Done (folded) → Cancelled (folded)** (`data/fsm_data.xml`). Companies created post-install also get a default FSM project via `res_company.create` hook.

### SQL constraints worth knowing
**Defined in `industry_fsm_sale/models/project_project.py:21-29`, `Project._sql_constraints`** (NOT in core `industry_fsm`; they only exist once the sale bridge is installed):
- `material_imply_billable` (`:21`) — `allow_material=True` requires `allow_billable=True`.
- `timesheet_product_required_if_billable_and_time` (`:23`) — if `allow_billable AND allow_timesheets AND is_fsm`, `timesheet_product_id NOT NULL`.
- `fsm_imply_task_rate` (`:22`) — FSM project cannot have `sale_line_id` set (pricing must be `task_rate` or `employee_rate`).

The only constraint in core `industry_fsm/models/project_project.py:26-28`, `Project._sql_constraints` is `company_id_required_for_fsm_project`.

---

## 2. `project.task` extension

Source: `industry_fsm/models/project_task.py` (~512 lines v18).

### Core fields
| Field | Type | Notes |
|-------|------|-------|
| `is_fsm` | related, readonly | Mirrors `project_id.is_fsm`. |
| `fsm_done` | Boolean, computed | True iff `state='1_done'`. Computed, **stored**. |
| `planned_date_begin`, `date_deadline` | Datetime | **Bidirectional requirement** — see Gotcha #4. |
| `partner_phone` | Char, computed+inverse | Reads `partner_id.mobile or partner_id.phone` (`:128-131`); **inverse writes back to the partner record** (`:133-140`, mutates customer master data). **Field persists in v19 but the inverse is removed** — reads partner directly, no write-back (FSMG58). |
| `partner_city` | Char, related, readonly | From partner. |
| `worksheet_signature` | Binary, `attachment=True`, `copy=False` | Customer on-site signature. Stored as ir.attachment. |
| `worksheet_signed_by` | Char | Free-text name signed. |
| `fsm_is_sent` | Boolean, readonly | Toggled by `_message_post_after_hook` when report mail sent. |
| `display_enabled_conditions_count` / `display_satisfied_conditions_count` | Integer, computed | Drive the "Mark as Done" button colour (primary green vs secondary yellow). |
| `worksheet_template_id` | Many2one `worksheet.template` | Added by `industry_fsm_report`. Inherited from project or parent task. |
| `worksheet_count` | Integer, computed | Counts rows in the dynamic `x_*` model. |
| `under_warranty` | Boolean | Added by `industry_fsm_sale`. Sets material SOL `price_unit=0` without affecting stock. |

### `action_fsm_validate(stop_running_timers=False)` workflow
The on-site close-out method — **PUBLIC name, no leading underscore** (`industry_fsm/project_task.py:280`, `Task.action_fsm_validate`). Order matters:

1. **Collect running timers** — querying both `project.task` timers and `account.analytic.line` (timesheet) timers (`:285-293`).
2. **If timers active and `stop_running_timers=False`** → open the wizard `project.task.stop.timers.wizard` to ask the user (`:297-308`).
3. **Auto-stop branch** (`stop_running_timers=True`):
   - `_stop_all_timers_and_create_timesheets()` (`:315-350`, renamed `_stop_all_timers_and_update_timesheets` in v19) computes elapsed minutes per timer, reads `timesheet_grid.timesheet_min_duration` (default 0) and `timesheet_grid.timesheet_rounding` (default 0) from `ir.config_parameter` (`:323-324`), creates `account.analytic.line` rows, then unlinks the `timer.timer` records.
4. **Write** `fsm_done=True` + `state='1_done'` (`:310`).
5. **Post-hook** in subclasses (overrides of `action_fsm_validate`, all super-chained):
   - `industry_fsm_sale` (`industry_fsm_sale/models/project_task.py:449`, `Task.action_fsm_validate`): `_fsm_ensure_sale_order()` (`:472`), `_fsm_create_sale_order_line()` (`:524`, timesheets), `_prepare_materials_delivery()` (`:702`, material SOLs). Skips (`continue`) when warranty+no-material or no-SO+no-timesheet (`:462`).
   - `industry_fsm_stock` (`industry_fsm_sale/models/project_task.py:175`, `Task._compute_material_line_totals`): `_validate_stock()` (`:36`) validates pickings with `cancel_backorder=True` context (`:115`, no backorder created on partial picks).
   - `industry_fsm_sale_report`: triggers customer signature/send-report buttons depending on satisfied conditions.

**Note:** core itself does **not** require worksheet or signature before fsm_done — that gating is added by `industry_fsm_report`.

### Date validation: the bidirectional trap
`write()` override (`:177-198`): if you write `planned_date_begin` without `date_deadline` (or vice-versa), **both** get nullified silently. Rationale: FSM gantt resource allocation requires both ends. User feedback: none.

### `_onchange_planned_dates()` is a no-op for FSM
Override (`:172-175`) returns early (no super) for FSM tasks, blanking base project validation to avoid clashing with the truncation logic above. Side effect: helpful date errors from base `project` are silenced for FSM tasks.

### Portal user auto-assignment on create
`create()` (`:200-206`) — when a portal user (`has_group('base.group_portal')`) creates an FSM task with no `partner_id`, it is auto-set to `current_user.partner_id` without asking. **Removed in v19** (`create()` no longer overrides `partner_id`).

### Resource overlap detection
`_allocated_hours_per_user_for_scale()` (lines 476-497) and overlap checks (lines 63-100) compute conflicts when:
1. Two FSM tasks on the same `user_ids` member overlap on `[planned_date_begin, date_deadline]`.
2. FSM task + non-FSM task combined allocated hours exceed user's workable calendar hours.

Returns **warnings only** — no auto-reschedule. Multi-day all-day tasks (≥24h duration) use work interval intersection instead of raw hour math.

### Recurrence
`project.task.recurrence` extension (`models/project_task_recurrence.py`): `partner_phone` is **copied** to recurrence children. If you update the parent partner phone later, children keep the old value (Char field, not related). `industry_fsm_sale._get_sale_line_id` returns `False` for recurring FSM tasks so each occurrence creates its own SO.

### Portal readable fields property
`SELF_READABLE_FIELDS` (v18:103) lists which fields portal users can read. **Renamed `TASK_PORTAL_READABLE_FIELDS` in v19** — custom modules that extend this list must rename the assignment.

### Timer extensions
`action_timer_start()` (lines 231-242): wraps base `timer.timer.action_timer_start`, posts a chatter message with timestamp. Override to inject your audit logging here. **Major rewrite in v19** (lines 201-243) to integrate optional geolocation capture (`allow_geolocation` flag).

`display_mark_as_done_primary`/`display_mark_as_done_secondary` (computed by `_compute_mark_as_done_buttons`, `:113`) are both filtered to `False` while a timer is running (`timer_start` set) — you cannot mark `fsm_done=True` with the timer ticking.

---

## 3. `res.company` + `res.config.settings`

`res_company.create` hook auto-creates a default FSM project per company on company creation.

`res.config.settings` (`industry_fsm/models/res_config_settings.py`) exposes only:
- `module_industry_fsm_report` (`:11`) — "Worksheet Templates".
- `module_industry_fsm_sale` (`:12`) — computed/forced True once a billable FSM project exists (`_compute_module_industry_fsm_sale`, `:29`), not freely toggleable.
- `group_industry_fsm_quotations` (`:17`) — implied-group toggle for `industry_fsm.group_fsm_quotation_from_task`.

There are **no** `module_industry_fsm_stock`, `module_industry_fsm_sms` or `module_helpdesk_fsm` settings fields — those bridges install via Apps, not a Settings toggle. Toggling the `module_*` fields above triggers install of the corresponding bridge.

---

## 4. Controllers (portal)

`industry_fsm/controllers/portal.py` — single significant route:

- `POST /my/tasks/<int:task_id>/worksheet/sign/<string:source>` (auth=public, JSON)
  - Validates `access_token` via `_document_check_access('project.task', task_id, access_token=...)`.
  - Writes `worksheet_signature` (Binary, base64) + `worksheet_signed_by` (Char).
  - Generates PDF via `report_template_ids` and posts to chatter.
  - **No token rotation on task duplicate** — duplicated task gets a fresh `access_token`, so old portal URLs go stale. Use `task.get_portal_url()` after `copy()`.

---

## 5. Data files

- `data/fsm_data.xml` — five default task stages (New / Planned / In Progress / Done / Cancelled).
- `data/mail_template_data.xml` — `industry_fsm.mail_template_data_task_report`: "Field Service Report - {{name}}", attaches `industry_fsm.task_custom_report` PDF, sends to `partner_id`, `auto_delete=True`.
- Cron: `_cron_recurring_create_task` (from base `project`) materialises recurring task occurrences — not FSM-specific but consumed by FSM.

---

## 6. Security

`security/fsm_security.xml`:

- `group_fsm_user` — base technician.
- `group_fsm_manager` — administrator.
- `group_fsm_quotation_from_task` — hidden group; unlocks "Create Quotation" button on FSM task.

Record rules:
- v18 domain: `('project_id.privacy_visibility', '!=', 'followers')`.
- v19 domain: `('project_id.privacy_visibility', 'in', ['employees', 'portal'])` — semantically equivalent but explicit.

Manager group bypasses all (`'global': True`).

**v19 refactor:** group records now use `privilege_id` (new model `res.groups.privilege`) instead of `category_id`; field `users` renamed to `user_ids`. See [[v19_changes]].

---

## 7. Buttons & UI behaviour

### "Mark as Done" cascade
- Primary (green) when `display_enabled_conditions_count == display_satisfied_conditions_count`.
- Secondary (yellow) when some conditions enabled but unmet (e.g. material needed but none added).
- Hidden entirely when timer running.

### "Sign Report" / "Send Report"
Added by `industry_fsm_sale_report` (without sale) and `industry_fsm_report` (with sale). Visibility:
- `allow_worksheets OR allow_material` must be true.
- No timer running.
- `worksheet_signature` not yet set.
- `display_satisfied_conditions_count > 0`.

### Partner navigation button
`res_partner.action_partner_navigate()` opens `https://www.google.com/maps/dir/?api=1&destination={encoded_address}`. **No Google Maps API key required** for the navigation URL itself, only for the map view (web_map module).

---

## 8. Cross-references

- Dynamic worksheet model engine and PDF generation → [[worksheet]].
- Sale order ↔ task wiring, pricing, materials-as-SOL → [[extensions_sale]].
- Stock moves, lots, picking validation → [[extensions_stock]].
- Customer signature, send-report, portal templates → [[extensions_report]].
- Kanban / map / gantt / timer widget / PWA → [[mobile]].
- Planning, helpdesk, subscription bridges → [[integrations]].
- Hidden behaviour, traps → [[gotchas]].
- v18 → v19 breaking changes → [[v19_changes]].
