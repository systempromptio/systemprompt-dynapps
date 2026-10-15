# FSM — Mobile / Tablet UX

How FSM presents itself on tablets and phones: PWA manifest, kanban / map / gantt JS, timer widget, signature pad, material catalog. Critical for understanding what works offline, what needs a Google API key, and what custom modules should avoid breaking.

**Sources:** `industry_fsm/static/src/`, `industry_fsm/controllers/webmanifest.py`, `industry_fsm/views/`, `worksheet/static/src/`.

---

## 1. PWA / scoped app

`industry_fsm/controllers/webmanifest.py:1-23` registers three scoped app shortcuts:
- `/scoped_app/field-service/new` — quick "Create Task".
- `/scoped_app/field-service` — "My Tasks" landing.
- `/scoped_app/field-service?view_type=calendar` — calendar view.

These are installable as a home-screen icon on iOS/Android via standard PWA install. The manifest declares scope, display mode, icons.

### Offline mode
**Not natively in FSM.** Relies on Odoo's web framework Service Worker (basic asset cache only). The technician can read the kanban/cards but cannot save changes offline. Worksheet drafts are cached in browser `localStorage` only as long as the form view stays open.

---

## 2. Kanban — `FsmMyTaskKanbanView`

**Files:**
- View XML: `industry_fsm/views/fsm_views.xml:225-249`, `project_task_view_kanban_fsm`.
- JS view: `industry_fsm/static/src/views/fsm_my_task_kanban/fsm_my_task_kanban_view.js`.
- Record component: `.../fsm_my_task_kanban_record.js`.
- Renderer: `.../fsm_my_task_kanban_renderer.js`.

Extends `projectTaskKanbanView` (project_enterprise). Stages come from `project.task.type`. Drag-drop across columns triggers `write({'stage_id': new_stage})` which then fires the SMS trigger if `sms_template_id` is set on the new stage.

Mobile behaviour: columns become full-width on small screens, swipeable.

---

## 3. Map view — `fsmMyTaskMapView` (v18) / `fsmTaskMapView` (v19)

**Files (v18):**
- View XML: `industry_fsm/views/fsm_views.xml:141-185`, `project_task_map_view_fsm`.
- JS: `industry_fsm/static/src/views/fsm_my_task_map/fsm_my_task_map_view.js` (+ controller).

**v19:** an additional `fsm_task_map_view.js` is added (new map view), bundled under `web.assets_backend_lazy`. The old `fsm_my_task_map` still ships in v19.

### Geolocation source
- Partner address concatenated (`contact_address_complete` field) → Google Maps geocoding.
- **No explicit `partner_latitude`/`partner_longitude` fields** in FSM. v19 introduces `allow_geolocation` flag on `project.project` and a `timer_geolocation_service` on the JS side (capturing GPS at timer start/stop).

### Google Maps API key
- **Required** for the map view to render and geocode (via `web_map` module from `project_enterprise`).
- Configured under Settings → Integrations → Google Maps. Without it, the map silently fails to load tiles ("API key invalid" in browser console).
- The "Navigate to partner" button on the task uses the `https://www.google.com/maps/dir/?api=1&destination=...` URL — that endpoint **does NOT require an API key** (anyone can build this URL). So navigation works even without a key; only the in-app map breaks.

### Fields displayed on map
- `partner_id`, `partner_phone`, `partner_zip`, `partner_city`.
- Task name, planned dates.

---

## 4. Gantt — `fsmTaskGantt` (v18)

**Files:**
- View XML: `industry_fsm_report/views/project_views.xml:4-21`, `project_task_view_gantt_fsm_worksheet`.
- JS controller / renderer / view: `industry_fsm/static/src/views/fsm_task_gantt/`.

Extends `taskGanttView` (project_enterprise). Group-by `worksheet_template_id` to colour by intervention type. Drag-to-reschedule supported.

**Status:** v18-only. v19 removes the dedicated gantt JS and falls back to the project_enterprise gantt with FSM-specific filters. Custom gantt overrides written for v18 must be ported.

---

## 5. Timer widget

**Files:**
- Component: `industry_fsm/static/src/components/timesheet_timer_header/timesheet_timer_header.js` (v18; **removed in v19**, refactored).
- Grid model: `industry_fsm/static/src/views/industry_fsm_grid/timesheet_grid_model.js`.
- Renderer: `industry_fsm/static/src/views/timesheet_timer_list_renderer/timesheet_timer_list_renderer.js`.

### Buttons
- Start, Stop, Pause, Resume — driven by computed fields `display_timer_start_primary`, `display_timer_stop`, `display_timer_pause`, `display_timer_resume`.
- Elapsed time shown in `h:mm` format while running.

### Server-side behaviour
- `action_timer_start()` (`industry_fsm_sale/models/project_task.py:231-242`, `Task._compute_portal_invoice_count`) posts a timestamped message to the task chatter.
- Timer state lives in `timer.timer` (one per `(res_model, res_id, user_id)` — see UNIQUE constraint in [[integrations]]).
- `display_mark_as_done_buttons` is filtered to False while timer is running — you cannot fsm_done a ticking task.

### Persistence across devices
- Timer state is server-side; refreshing the page or switching device shows the resumed elapsed time.
- **However** if browser tab closes mid-tick, frontend animation is lost; the server clock keeps ticking, and elapsed time calculation may be off by the tab-closed duration (the user never saw the running indicator but the server thinks the timer was running).
- The UNIQUE constraint means **only one timer per user per task** — second device start fails silently (DB IntegrityError, no UI surface).

### Rounding
On stop, `_stop_all_timers_and_create_timesheets()` applies `timesheet_grid.timesheet_min_duration` (default 0) and `timesheet_grid.timesheet_rounding` (default 0). User-visible UI shows raw elapsed seconds; the stored timesheet may be inflated by rounding. If admin sets min_duration=10min and the technician runs the timer for 2 min, the timesheet logs 10 min.

---

## 6. Material catalog (tablet kanban)

`industry_fsm_sale` adds a product catalog kanban accessible from the task form via "Products" smart button. Each product card has a quantity control (`+ / -`) and an "Add to task" button.

`industry_fsm_stock` extends the kanban to:
- Hide forecast action for service products.
- Show a `min_quantity` constraint (cannot decrease below `quantity_decreasable_sum`).
- Show a `tracking` flag on the card to indicate lot/serial selection is needed.
- Mark the row `readOnly` if SO is locked (`order._is_readonly()` with `fsm_task_id` in context).

For tracked products, clicking "+" triggers the `fsm.stock.tracking` wizard (see [[extensions_stock]]) for lot/serial selection.

---

## 7. Signature pad

- HTML5 Canvas + `signature_pad.js` (loaded from base portal module).
- Modal-rendered, touch-enabled.
- Submits via `POST /my/tasks/<id>/worksheet/sign/<source>` (see [[extensions_report]]).

### iOS Safari issue
- Versions < 15: touch coordinates offset 2–4 px from the canvas, leading to "drift" on the signature.
- Fixed in Odoo 18.3+ via canvas calibration.

### Crash recovery
- If the browser crashes mid-sign, the canvas data is lost (only persisted on successful POST).
- Odoo's form framework caches drafts in `localStorage`, recovered on page reload — but the canvas state itself is not part of the form draft.

---

## 8. Mobile form view

`industry_fsm.project_task_view_mobile_form` — dedicated form layout for small screens:
- Reduced field density.
- Date pickers replaced with mobile-native datetime selectors.
- Smart buttons collapsed into a header strip.

The web client switches to this view automatically when viewport width is below ~768 px.

---

## 9. Asset bundles

Manifest declares two bundles:
- `web.assets_backend` — kanban, list, form, timer widget, signature pad.
- `web.assets_backend_lazy` — map view, gantt view (loaded only when the user navigates to those views).

**Effect:** first navigation to map/gantt has a 1–2 s lag while the JS chunk loads. Subsequent navigations are cached. If your custom module modifies the map view, declare your assets in the lazy bundle as well, otherwise your overrides will not apply until the lazy bundle is hit.

---

## 10. v19 mobile changes

See [[v19_changes]] for the full diff. Headlines:
- New `timer_geolocation_service.js` — captures lat/lng at timer start when `project.allow_geolocation=True`.
- New `fsm_task_form_view.js` — dedicated form controller (`js_class="fsm_project_task_form"` on the form view).
- New `fsm_task_template_dropdown.js` and `fsm_project_template_dropdown.js` — template selection UI.
- New `fsm_stop_timer_wizard_view.js` — new wizard model `hr.timesheet.stop.timer.confirmation.wizard`.
- Removed JS files:
  - `timesheet_timer_header.js` (refactored).
  - `fsm_date_widget.js` (replaced by OWL component).

---

## 11. Cross-references

- Timer model internals + UNIQUE constraint → [[integrations]].
- Stage SMS template + signature flow → [[extensions_report]].
- Worksheet form/PDF coupling → [[worksheet]].
- Lazy asset bundle + Studio + Safari gotchas → [[gotchas]].
- v19 JS rename / removal → [[v19_changes]].
