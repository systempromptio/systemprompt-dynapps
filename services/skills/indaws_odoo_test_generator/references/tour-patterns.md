<!-- verified: v18.0/v19.0 @ 2026-06 -->
# Odoo Tour Test Patterns Reference

Complete reference for creating end-to-end tour tests across Odoo versions 16-19.

> **Source-verified (v18/v19).** Step schema, registration, `start_tour`, `run` command
> vocabulary and the v18→v19 directory restructure are all checked against core. Where a
> property below is marked **LEGACY (≤v16)** it is no longer valid in the modern tour engine.

---

## 1. Architecture Overview

Tour tests have two components:
1. **Python launcher** (`HttpCase`) — sets up data, starts the tour, verifies results.
2. **JavaScript tour definition** — declares UI steps executed in headless Chrome.

```
Python Test (HttpCase)           JavaScript Tour
┌───────────────────┐            ┌───────────────────────────────────┐
│ setUpClass()      │            │ registry.category("web_tour.tours")│
│   create data     │            │   .add("name", {                  │
│                   │            │     url: "/odoo",                 │
│ test_my_tour():   │──starts──> │     steps: () => [                │
│   start_tour()    │            │       {trigger, run, content},    │
│                   │ <──done──  │     ]                             │
│   assert results  │            │   })                              │
└───────────────────┘            └───────────────────────────────────┘
```

---

## 2. Python Tour Launcher

### `start_tour` — verified signature

Source: `odoo/tests/common.py` — **v18:2421 / v19:2555**

```python
def start_tour(self, url_path, tour_name, step_delay=None, **kwargs):
```

Only `url_path`, `tour_name` and `step_delay` are explicit. Everything else flows through
`**kwargs` into `browser_js` (v18:2323 / v19:2450):

```python
def browser_js(self, url_path, code, ready='', login=None, timeout=60, cookies=None,
               error_checker=None, watch=False, success_signal=DEFAULT_SUCCESS_SIGNAL,
               debug=False, cpu_throttling=None, **kw):
```

So the real, supported kwargs for `start_tour` are:
`login`, `timeout` (default 60s), `cookies`, `watch`, `debug`, `step_delay`,
`delay_to_check_undeterminisms`, `error_checker`, `cpu_throttling`.

> **v19 note (source v19:2569):** passing `step_delay` now logs a warning
> *"step_delay is only suitable for local testing"* — never commit a tour launcher with
> `step_delay` set.

```python
import odoo.tests
from odoo.tests import tagged, HttpCase
from odoo import Command


@tagged('post_install', '-at_install')   # tours MUST be post_install
class TestMyTour(HttpCase):

    def test_my_feature_tour(self):
        """Confirm a sale order through the UI and persist state='sale'."""
        # 1. Setup data
        self.env['sale.order'].create({
            'partner_id': self.env.ref('base.res_partner_2').id,
            'order_line': [Command.create({
                'product_id': self.env.ref('product.product_product_4').id,
                'product_uom_qty': 1,
            })],
        })
        # 2. Run tour (name MUST match JS registration)
        self.start_tour('/odoo', 'my_module_feature_tour', login='admin')
        # 3. Assert the *behavior*, not absence of error
        order = self.env['sale.order'].search([('state', '=', 'sale')], limit=1)
        self.assertTrue(order, "Tour must leave the order confirmed")
```

> Why `post_install`: tours require the full registry/assets to be built. `at_install`
> tours fail with missing assets.

### Portal / public tours

```python
def test_portal_tour(self):
    self.start_tour('/my/orders', 'portal_order_tour', login=self.portal_user.login)

def test_public_tour(self):
    self.start_tour('/shop', 'website_sale_shop_tour', login=None)  # None = public
```

---

## 3. JavaScript Tour Definition

### Modern engine (v17–v19) — the only form you should write today

```javascript
import { registry } from "@web/core/registry";

registry.category("web_tour.tours").add("my_module_feature_tour", {
    url: "/odoo",
    steps: () => [
        {
            content: "Open the Sales app",
            trigger: '.o_app[data-menu-xmlid="sale.sale_menu_root"]',
            run: "click",
        },
        {
            content: "Wait for the list view",
            trigger: ".o_list_view .o_data_row:first",
            // no `run` → step just waits for the trigger to exist
        },
    ],
});
```

Verified against real core tours: `account/static/src/js/tours/account.js`
(`registry.category("web_tour.tours").add('account_tour', { url: "/odoo", steps: () => [...] })`).

### LEGACY (≤v16) — `odoo.define` + `tour.register`

```javascript
odoo.define("module.tour", function (require) {
    var tour = require("web_tour.tour");
    tour.register("tour_name", { url: "/web", test: true }, [
        { trigger: ".selector", content: "...", run: "click" },
    ]);
});
```

Do not generate this for v17+. Note also `test: true` is **no longer needed** in the modern
engine — every tour added to the `web_tour.tours` registry is runnable as a test.

---

## 4. Tour Step Schema — VERIFIED (v18/v19)

The schema is enforced by `web_tour/static/src/.../tour_step.js` (`TourStep` class).
Verified accepted keys (v18 `web_tour/static/src/tour_service/tour_step.js:83-89`, `TourStep.stringify`, v19 `js/tour_step.js:83-89`):

| Key | Type | Notes |
|-----|------|-------|
| `trigger` | `string` (HootSelector) | **Required.** Element the action runs on. |
| `content` | `string` | Optional tooltip / description. Use `markup(_t(...))` if HTML. |
| `run` | `string \| function \| false` | Action. Omit/`false` = wait only. |
| `tooltipPosition` | `"top"\|"bottom"\|"left"\|"right"` | Replaces the old `position`. |
| `timeout` | `number` (ms) | Per-step. **Default 10000ms** (not 60000). |
| `isActive` | `string[]` | Conditional activation (see below). |
| `expectUnloadPage` | `boolean` | Step triggers a full page reload. |

> **Cuidado: el `@typedef TourStep` de la fuente contradice esta tabla, y la fuente es la que se
> equivoca.** El jsdoc de cabecera (v18 `tour_service/tour_step.js:6-16`, v19 `js/tour_step.js:6-16`)
> sigue documentando **`position`** en las dos versiones — pero **nadie lo lee en runtime**: el motor
> saca `tooltipPosition` del step (`web_tour/static/src/tour_service/tour_pointer_state.js:113`, `createPointerState`, v18 en `tour_service/`, v19 en
> `js/tour_pointer/`) y lo pasa al puntero como `position` (`:128`, `:146`). Por eso la cita buena de
> esta tabla es el `pick()` de `:83-89`, que es la lista de claves que el runtime **usa**, y no el
> jsdoc, que es la que alguien **escribió una vez**. Verificado 2026-08-17. No lo "corrijas" al
> jsdoc.

> **These keys are NO LONGER valid in v17+** (silently ignored or schema-rejected):
> `position` (→ `tooltipPosition`), `extra_trigger`, `alt_trigger`, `in_modal`, `edition`
> (→ `isActive: ["enterprise"|"community"]`), `skip_trigger`, `pause`, `break`, `debugHelp`.
> `test: true` at tour level is obsolete.

### `isActive` — conditional steps (replaces `edition`/mobile flags)

Source: `web_tour/static/src/tour_service/tour_step.js:33-60`, `TourStep.active`. Recognized keywords:
`"enterprise"`, `"community"`, `"mobile"`, `"desktop"`, `"auto"`, `"manual"`.
Anything else in the array is treated as an extra CSS selector that must match.

```javascript
{ isActive: ["enterprise"], content: "EE-only step", trigger: ".o_enterprise_btn", run: "click" }
{ isActive: ["auto"],       trigger: ".only_in_automatic_run", run: "click" }
```

---

## 5. `run` command vocabulary — VERIFIED

Source: `web_tour/static/src/.../tour_helpers.js` (v18 `tour_service/`, v19 `js/tour_automatic/`).
String form `"<command> [selector] [value]"`; the selector is relative to `trigger` when omitted.

| Command | Example | Effect |
|---------|---------|--------|
| `click` | `run: "click"` | Click trigger (or `"click .o_rows:first"`). |
| `dblclick` | `run: "dblclick"` | Double click. |
| `edit` | `run: "edit Hello World"` | Clear + type into an input. **Replaces the old `text ...`.** |
| `clear` | `run: "clear"` | Empty the input. |
| `check` / `uncheck` | `run: "check"` | Toggle a checkbox. |
| `select` | `run: "select Foden47"` | Select `<option>` by value. |
| `selectByLabel` | `run: "selectByLabel Jeremy Doku"` | Select by visible label. |
| `selectByIndex` | `run: "selectByIndex 2"` | Select Nth option. |
| `hover` | `run: "hover"` | Mouse over. |
| `press` | `run: "press Enter"` | Keyboard key(s). |
| `drag_and_drop` | `run: "drag_and_drop .o_rows:first"` | Drag trigger onto selector. |
| `goToUrl` | `run: "goToUrl /shop"` | Navigate. |
| function | `async run(helpers) { await helpers.click(); }` | Custom; `helpers` exposes all the above. |

> **The legacy `run: "text ..."` is gone in v17+** — use `run: "edit ..."`.

---

## 6. Common Step Patterns (modern engine)

```javascript
// Click a button
{ content: "Confirm", trigger: 'button[name="action_confirm"]', run: "click" },

// Type into a Many2one and pick a result
{ content: "Type partner", trigger: 'div[name="partner_id"] input', run: "edit Test Partner" },
{ content: "Pick partner", trigger: '.o_input_dropdown li:contains("Test Partner")', run: "click" },

// Click inside a modal (no in_modal flag — just target .modal)
{ content: "Confirm dialog", trigger: ".modal-footer .btn-primary", run: "click" },

// Add a one2many line
{ content: "Add line", trigger: ".o_field_x2many_list_row_add a", run: "click" },

// Wait only (no run)
{ content: "Wait for form", trigger: ".o_form_view .o_form_sheet" },

// Step that reloads the page
{ content: "Logout", trigger: ".o_logout", run: "click", expectUnloadPage: true },
```

---

## 7. Tour helpers / `stepUtils` — import path moved in v19

`stepUtils` provides reusable step builders (`goToAppSteps`, `showAppsMenuItem`, …).

| Version | Import |
|---------|--------|
| v18 | `import { stepUtils } from "@web_tour/tour_service/tour_utils";` |
| **v19** | `import { stepUtils } from "@web_tour/tour_utils";` |

Verified 2026-08-17 (CE18 `@99f5a33c64`, CE19 `@d1d161033`): v18
`web_tour/static/src/tour_service/tour_utils.js:117`, `stepUtils` vs v19 `web_tour/static/src/tour_utils.js:3`, `stepUtils` — en los dos, la línea es
`export const stepUtils = {`.

> **No es un wrapper de re-export, y la diferencia es la que rompe.** En v19 ese fichero es la
> implementación entera (232 líneas, **cero** `export *` o `from "@web_tour/..."`) y
> **`static/src/tour_service/` ya no existe** en v19. O sea: el import viejo no cae por una capa de
> compatibilidad, **no resuelve**. Un tour de v18 portado sin tocar el import falla al construir el
> bundle, no en runtime.
>
> **Y no hay nada más que portar**: comparados los helpers de `stepUtils` uno a uno entre v18 y v19,
> **no se eliminó ninguno** (`goToAppSteps`, `showAppsMenuItem`, `saveForm`, `discardForm`,
> `statusbarButtonsSteps`, `toggleHomeMenu`, `goToUrl`, `waitIframeIsReady`, `addDebugHelp`,
> `autoExpandMoreButtons`, `mobileKanbanSearchMany2X`, `_getHelpMessage`) y v19 **añade uno**:
> `editSelectMenuInput`. Cambia la ruta, no la API.

```javascript
import { registry } from "@web/core/registry";
import { stepUtils } from "@web_tour/tour_utils";  // v19 path

registry.category("web_tour.tours").add("my_full_tour", {
    url: "/odoo",
    steps: () => [
        ...stepUtils.goToAppSteps("sale.sale_menu_root", "Open Sales"),
        { trigger: ".o_list_button_add", run: "click" },
        { trigger: 'div[name="partner_id"] input', run: "edit Test Partner" },
    ],
});
```

For your own reusable builders, return arrays and spread them (`...openApp(...)`).

---

## 8. Manifest assets

```python
'assets': {
    'web.assets_tests': [
        'my_module/static/tests/tours/**/*',   # tour JS, run via HttpCase
    ],
}
```

(For Hoot **component** unit tests use `web.assets_unit_tests` — see `js-test-hoot.md`.)

---

## 9. File organization

```
my_module/
├── static/src/js/tours/        # tour definitions (preferred core location)
│   └── my_feature_tour.js
└── tests/
    ├── __init__.py
    └── test_tour.py            # HttpCase launchers
```

---

## 10. Debugging tours

```python
# Python: bigger timeout, watch the browser, enable debug logging
self.start_tour('/odoo', 'my_tour', login='admin', timeout=600, watch=True, debug=True)
```

```bash
./odoo-bin -d testdb --test-tags /my_module:TestMyTour.test_my_feature_tour \
    -u my_module --stop-after-init --log-level=test
```

> Do not use the old `pause: true` / `break: true` step keys — they are not in the v18/v19
> schema. Use `watch=True` from Python and browser devtools.

---

## 11. v18 → v19 restructure (directory move) — VERIFIED

The whole tour engine moved out of `tour_service/`:

| v18 path | v19 path |
|----------|----------|
| `static/src/tour_service/tour_service.js` | `static/src/js/tour_service.js` |
| `static/src/tour_service/tour_step.js` | `static/src/js/tour_step.js` |
| `static/src/tour_service/tour_state.js` *(existe en v18; `tour_pointer_state.js` es OTRO fichero, tambien en v18)* | `static/src/js/tour_state.js` |
| `static/src/tour_service/tour_helpers.js` | `static/src/js/tour_automatic/tour_helpers.js` |
| `static/src/tour_service/tour_automatic.js` | `static/src/js/tour_automatic/tour_automatic.js` |
| `static/src/tour_service/tour_utils.js` | `static/src/tour_utils.js` (+ `static/src/js/utils/tour_utils.js` — con `js/`) |
| `static/src/widgets/tour_start.js` | `static/src/widgets/tour_start.js` (unchanged) |

**Impact on your code:** only the public import paths matter — `stepUtils` is now
`@web_tour/tour_utils` (§7). The `registry.category("web_tour.tours")` API and the step
schema are unchanged between v18 and v19.

---

## 12. Common CSS selectors

| Element | Selector |
|---------|----------|
| Specific app | `.o_app[data-menu-xmlid="module.menu_id"]` |
| Create button (list) | `.o_list_button_add` |
| Save / discard | `.o_form_button_save` / `.o_form_button_cancel` |
| Breadcrumb | `.o_control_panel .breadcrumb-item` |
| Status bar button | `.o_statusbar_status button` |
| Kanban card / list row | `.o_kanban_record` / `.o_data_row` |
| Modal confirm / cancel | `.modal-footer .btn-primary` / `.btn-secondary` |
| Many2one input | `div[name="field"] input` (or `.o_field_many2one input`) |
| Many2one dropdown item | `.o_input_dropdown li:contains("…")` |
| One2many add line | `.o_field_x2many_list_row_add a` |
| Search input | `.o_searchview_input` |
