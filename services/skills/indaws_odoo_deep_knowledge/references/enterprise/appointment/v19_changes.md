# Appointment — v18 → v19 diff

`diff -u odoo_enterprise/18/appointment/models/appointment_type.py odoo_enterprise/19/appointment/models/appointment_type.py | wc -l` → 751 lines changed (significant refactor).

## Major behaviour changes

### 1. Confirmation logic inverted

| Aspect | v18 | v19 |
|---|---|---|
| Field | `appointment_manual_confirmation` (Boolean, default False) | `auto_confirm` (Boolean, default True) |
| Capacity threshold | `resource_manual_confirmation_percentage` (0..1, default 0) | `manual_confirmation_percentage` (0..1, default **1.0**) |
| Behaviour | Auto-confirm by default; manual flag opts into requests | Auto-confirm by default; only requests once capacity exceeds threshold |
| Semantics | Manual ON ⇒ all bookings start `request` | `auto_confirm=False` mirrors v18 manual ON |
| Helper | `_get_default_appointment_status` | `_get_default_appointment_status` rewritten with `is_always_confirm` flag |

Migration: `auto_confirm = NOT appointment_manual_confirmation`; `manual_confirmation_percentage = resource_manual_confirmation_percentage` (or 1.0 if you want classic auto-confirm-everything).

### 2. Assignment method refactor

v18:
```python
assign_method = Selection([('resource_time', ...), ('time_resource', ...), ('time_auto_assign', ...)])
```

v19 splits into 4 fields:
```python
assignment_method = Selection([('auto', ...), ('manual', ...)], compute='_compute_assignment_method', readonly=False)
is_auto_assign = Boolean('Assign automatically')
is_date_first = Boolean('Select date and time first')
select_first = Selection([('date', ...), ('user_resource', ...)], compute='_compute_select_first', readonly=False)
```

Mapping:
- v18 `resource_time` → v19 `is_auto_assign=False`, `is_date_first=False`
- v18 `time_resource` → v19 `is_auto_assign=False`, `is_date_first=True`
- v18 `time_auto_assign` → v19 `is_auto_assign=True`, `is_date_first=True`

### 3. New capacity model

v19 adds:
- `manage_capacity` (Boolean) — replaces `resource_manage_capacity` (still also exists for resources but renamed semantics).
- `max_bookings` (Integer, default 1) — total slot capacity for non-managed-capacity types.
- `user_capacity` (Integer) — per-user max simultaneous bookings.
- Constraint `CHECK(user_capacity >= 1 AND max_bookings >= 1)`.

v18 only had per-resource `capacity` and the manual-confirmation percentage. v19 unifies users + resources under `manage_capacity` / `max_bookings`.

### 4. Question model: M2O → M2M + reusable

v18:
```python
appointment_type_id = fields.Many2one('appointment.type', ondelete='cascade')
```
v19:
```python
appointment_type_ids = fields.Many2many('appointment.type', relation='appointment_type_appointment_question_rel')
is_default = fields.Boolean('Default question', help="Include by default in new appointment types.")
is_reusable = fields.Boolean('Is Reusable', compute='_compute_is_reusable', default=True, store=True, readonly=False)
extra_comment = fields.Html('Extra Comment', translate=True)
appointment_count = fields.Integer(compute='_compute_appointment_count')
```

New constraint `CHECK(is_default IS DISTINCT FROM TRUE OR is_reusable IS TRUE)`. New question type `'phone'` added (after `'email'`).

New file `data/appointment_question_data.xml` ships pre-built default questions (Email, Phone, Notes).

### 5. New `is_always_confirm` derivation

v19 caches `is_always_confirm = auto_confirm AND manual_confirmation_percentage == 1.0` to short-circuit the status decision. v18 had no such precomputation — it called the function each booking.

## Framework / API migrations

| Area | v18 | v19 |
|---|---|---|
| Domain | `from odoo.osv import expression` | `from odoo.fields import Domain` |
| Command | `from odoo import Command` | `from odoo.fields import Command` |
| URL join | `from werkzeug.urls import url_join` | `from odoo.tools.urls import urljoin as url_join` |
| Default tz | `default=lambda self: self.env.user.tz` | `default=lambda self: self.env.user.tz or 'UTC'` (no None tz) |
| `_sql_constraints` | tuple list | `_xxx = models.Constraint('CHECK(...)', 'msg')` |
| Assets — frontend | `static/src/js/*.js` (legacy publicWidget) | `static/src/interactions/*.js` (new Interaction framework) |
| Assets — wysiwyg | `wysiwyg.js` for editor extensions | dropped; replaced by editor plugin in `appointment_plugin.js` |
| `Partner` class name | `class Partner` | `class ResPartner` |
| Frontend mail utils | imported in manifest (`mail/static/src/utils/common/{html,format}.js`) | dropped (now in `web.assets_frontend` core) |

## New files in v19

- `data/appointment_question_data.xml` — default reusable questions.
- `models/ir_ui_view.py` — view extension hook.
- `views/appointment_answer_views.xml` — answer dedicated views (v18 had only inline).
- `static/src/components/appointment_booking_action_helper/` — empty-state helper.
- `static/src/components/appointment_quick_share_button/` — list-view share shortcut.
- `static/src/components/appointment_template_picker_dialog/` — template picker for new types.
- `static/src/fields/` — custom field widgets.
- `static/src/interactions/` — replaces `static/src/js/`.
- `static/src/img/appointment_{1..4}.jpg` — onboarding visuals.

## Removed/moved

- `views/calendar_views.xml` — merged into other view files.
- `static/src/js/wysiwyg.js` — removed.
- `static/src/js/appointment_select_appointment_type.js` etc. → moved to `interactions/`.

## res.partner

`calendar_verify_availability(date_start, date_end)` gains optional `appointment_type` parameter — if the staff user is checking against their own appointment type, events linked to that same type are skipped (avoid self-block). v18 had no such exclusion.

`_get_busy_calendar_events()` removed in v19 — logic inlined.

## Migration cookbook

1. Map `assign_method` → (`is_auto_assign`, `is_date_first`).
2. `auto_confirm = NOT appointment_manual_confirmation`.
3. `manual_confirmation_percentage`: copy from `resource_manual_confirmation_percentage`; default to 1.0 for non-resource types.
4. For each `appointment.question`: convert M2O `appointment_type_id` to a single-element M2M; set `is_reusable=True` if you want it shareable.
5. Update Python imports: `expression` → `Domain`, `Command` → `from odoo.fields`.
6. If you have custom `static/src/js/appointment_*.js`, rewrite as Interactions in `static/src/interactions/`.
7. Replace `_sql_constraints` with `models.Constraint`.
