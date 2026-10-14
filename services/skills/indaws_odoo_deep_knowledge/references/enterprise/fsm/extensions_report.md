# FSM — Report, signature & SMS (`industry_fsm_report`, `industry_fsm_sale_report`, `industry_fsm_sms`)

Worksheet template binding to FSM tasks (independent of sale), customer signature flow, PDF report generation, mail send-out, and stage-driven SMS notifications.

**Modules:**
- `industry_fsm_report` (1048 py lines) — worksheet template field on task, "Sign Report" / "Send Report" buttons, PDF.
- `industry_fsm_sale_report` (744 py lines) — same flow but driven by SO product → task auto-binding.
- `industry_fsm_sms` (20 py lines) — two SMS templates for stage transitions.

---

## 1. Worksheet binding without sale (`industry_fsm_report`)

### Fields on `project.task`
| Field | Type | Notes |
|-------|------|-------|
| `worksheet_template_id` | Many2one `worksheet.template`, tracked | Filtered by company. |
| `worksheet_count` | Integer, computed | Counts rows in dynamic `x_*` model. |
| `allow_worksheets` | Boolean, related to `project_id.allow_worksheets` | Toggles UI. |

### Auto-assignment (`_compute_worksheet_template_id`, lines 93-94)
1. From parent task.
2. From project default.
3. From product (only if `industry_fsm_sale_report` installed and product has `worksheet_template_id`).

### Button visibility
`_compute_display_sign_report_buttons()` (lines 44-62):
- `display_sign_report_primary/secondary` true when:
  - `allow_worksheets OR allow_material`.
  - No `timer_start`, no `worksheet_signature` yet.
  - `display_satisfied_conditions_count > 0`.

`_compute_display_send_report_buttons()` (lines 68-86): same logic plus the `fsm_is_sent` check.

---

## 2. Customer signature flow

### Portal route
`industry_fsm_report/controllers/portal.py:1-24`:

`POST /my/tasks/<int:task_id>/worksheet/sign/<string:source>` (auth=public, json)
1. Validate `access_token` via `_document_check_access('project.task', task_id, access_token=...)`.
2. Read POST body: `signature` (base64 PNG), `name`.
3. Write `task.worksheet_signature` (Binary, stored as ir.attachment) + `task.worksheet_signed_by` (Char).
4. Generate PDF via `report_template_ids` and post to chatter.

**Token rotation:** there is none on `copy()` — but the duplicated record gets a **fresh** `access_token` (random UUID via `mail.thread`). Old portal URLs from before the copy still work against the original task only.

### Signature widget (frontend)
- HTML5 Canvas + `signature_pad.js` (loaded via base portal module).
- Mobile-friendly: touch-enabled, modal-rendered.
- Storage: directly into `worksheet_signature` (Binary, `attachment=True`).
- Validation gate: `task._has_to_be_signed()` (inherited from base FSM) — the portal template (`industry_fsm/views/project_portal_templates.xml:5`, `portal_task_sign_button`) hides the signature form when this returns False.

### "Send Report" action
`industry_fsm/models/project_task.py:413-430`, `Task` — `action_send_report()`:
- Filters tasks with `_is_fsm_report_available()` (must have timesheet entries).
- Opens `mail.compose.message` wizard with context `fsm_mark_as_sent=True`.
- `_message_post_after_hook()` (line 442) sets `fsm_is_sent=True` once mail leaves.

Mail template: `industry_fsm.mail_template_data_task_report` (`industry_fsm/data/mail_template_data.xml:35-51`, `mail_template_data_task_report`).
- Subject: `Field Service Report - {{ object.name }}`.
- Attachments: `report_template_ids` → `industry_fsm.task_custom_report` (PDF auto-generated).
- Partner: `{{ object.partner_id.id }}`.
- `auto_delete=True`.
- Respects `partner_id.lang`.

---

## 3. PDF report — `industry_fsm.task_custom_report`

Source: `industry_fsm/report/worksheet_custom_report_templates.xml` (130 lines).

### Layout sections
1. **Worker info** (lines 9-17): `user_ids` contact list.
2. **Customer** (lines 19-27): `partner_id` (name, address, phone, email).
3. **Timesheets** (lines 36-84): tabular display of `timesheet_ids` (date, worker, description, time). Hidden if `not doc.allow_timesheets OR not doc.timesheet_ids`.
4. **Worksheet section** — inherited extension in `industry_fsm_report/report/worksheet_custom_report_templates.xml:5-14`, `worksheet_custom_page`:
   - Renders the dynamic model row via `report_view_id` on the worksheet template.
   - Uses `worksheet_map` dict (`{task.id: latest_worksheet_record_by_create_date}`).
   - Only shown when `doc.worksheet_count AND doc.worksheet_template_id.report_view_id` (line 6).
5. **Signature** (lines 86-115): displays `worksheet_signature` + `worksheet_signed_by`.

### Worksheet section gotchas
- Empty if worksheet template has no `report_view_id` (regenerated automatically on form save — see [[worksheet]]).
- Custom `x_*` fields are rendered, **but only those visible in the form view**. Studio "hide from form" hides from PDF too.

---

## 4. SO-driven flow (`industry_fsm_sale_report`)

Adds the same buttons as `industry_fsm_report` but with the SO context. Key wiring:

- **Worksheet template per product** — see [[extensions_sale]] §1.
- **Task auto-receives template** on creation from SOL: context `default_worksheet_template_id` propagated.
- **Button conditions** (`industry_fsm_sale_report/models/project_task.py:9-55`, `Task._compute_display_sign_report_buttons`) duplicate the report module's logic but also factor `has_template_ancestor` and `has_project_template` (template-task / template-project handling new in v19, see [[v19_changes]]).

---

## 5. `industry_fsm_sms`

20 py lines. `auto_install=True` when `industry_fsm + project_sms` are installed.

### Mechanism
Stage transitions on `project.task.type` trigger SMS when the stage has a `sms_template_id` set. Logic comes from base `project_sms`; FSM adds two ready-made templates:

### Templates (`industry_fsm_sms/data/industry_fsm_sms_data.xml:4-16`, `sms_template_data_fsm_intervention`)
1. **`sms_template_data_fsm_intervention`** — "Field Service: Intervention Scheduled"
   - Body: `{{ object.company_id.name }}: Your intervention is scheduled {{ 'from the ' + format_datetime(object.planned_date_begin) + ' to the ' + format_datetime(object.date_deadline) if ... else '' }}`.
2. **`sms_template_data_fsm_onway`** — "Field Service: Our team is on its way to your location."
   - Body: `{{ object.company_id.name }}: We are on our way to your intervention.`

### Demo data
`industry_fsm_sms/data/industry_fsm_sms_demo.xml:6`, `industry_fsm.planning_project_stage_2` — assigns the "on the way" template to stage 2 ("Planned"). Customers therefore receive a "we're on our way" SMS the moment a technician drags the task into the Planned column.

### Delivery
- Via Odoo IAP credits.
- Requires `partner_id.mobile` (not `phone`). If only `phone` is set, the template **silently fails to send**.
- Status tracked in `sms.sms` (sent / failed / pending).

---

## 6. Test scenarios

- `industry_fsm/tests/test_industry_fsm_flow.py` — signature + PDF generation.
- `industry_fsm_report/tests/test_industry_fsm_report.py` — worksheet template auto-assignment from project default.
- `industry_fsm_sale_report/tests/` — product → task worksheet inheritance.

---

## 7. Gotchas specific to this layer

See [[gotchas]] for the comprehensive list. Highlights:

- **PDF empty without `report_view_id` on template** — happens if template was created but its form view never saved (auto-generation never triggered).
- **Signature widget broken on iOS Safari < 15** — touch coordinates offset 2–4 px. Fixed in Odoo 18.3+.
- **SMS uses `partner_id.mobile`, display uses `partner_id.phone or mobile`** — divergence possible.
- **Portal `worksheet_map` not iterated in portal template** — custom `x_*` fields are visible in the PDF but **not** on the portal page itself. To show them on portal, extend `project_portal_templates.xml` manually.
- **Send-report button hidden until at least one timesheet exists** — pure worksheet/material flows trigger the same `_is_fsm_report_available()` check, which only counts timesheet rows.

---

## 8. Cross-references

- Worksheet engine internals → [[worksheet]].
- Product → task template propagation → [[extensions_sale]].
- Mobile signature pad, kanban, gantt, map → [[mobile]].
- Stage-driven SMS / mail sync issues → [[gotchas]].
- v19 view/portal changes → [[v19_changes]].
