<!-- verified: v18.0 @ 2026-06 -->
# JavaScript — Views, Widgets, Templates & Assets (Odoo 18.0)

## Registry System

```javascript
import { registry } from "@web/core/registry";
```

### Key categories

| Category | Purpose | How to register |
|----------|---------|-----------------|
| `"services"` | Custom services | `registry.category("services").add("my_service", serviceDefinition)` |
| `"views"` | View types (form, list...) | `registry.category("views").add("my_view", viewDefinition)` |
| `"fields"` | Field widgets | `registry.category("fields").add("my_widget", MyFieldComponent)` |
| `"actions"` | Client actions | `registry.category("actions").add("my_action", MyActionComponent)` |
| `"main_components"` | Always-visible components | `registry.category("main_components").add("my_comp", { Component })` |
| `"web_tour.tours"` | Tour definitions | `registry.category("web_tour.tours").add("my_tour", tourDef)` |
| `"public_components"` | Public/frontend components | `registry.category("public_components").add("my_comp", { Component })` |
| `"systray"` | Systray items | `registry.category("systray").add("my_item", { Component })` |
| `"user_menuitems"` | User menu items | `registry.category("user_menuitems").add(...)` |
| `"command_categories"` | Command palette categories | `registry.category("command_categories").add(...)` |
| `"formatters"` | Field value formatters | `registry.category("formatters").add("float", formatFloat)` |
| `"parsers"` | Field value parsers | `registry.category("parsers").add("float", parseFloat)` |

### Registering a client action

```javascript
registry.category("actions").add("my_dashboard", DashboardComponent);

// In Python:
# return { 'type': 'ir.actions.client', 'tag': 'my_dashboard' }
```

---

## View Architecture

Every view follows the Controller / Model / Renderer pattern:

```javascript
export const myView = {
    type: "my_view",
    Controller: MyController,      // orchestration, actions, pager
    Renderer: MyRenderer,          // DOM rendering
    ArchParser: MyArchParser,      // XML arch → JS config
    Model: RelationalModel,        // data loading, CRUD
    Compiler: MyCompiler,          // optional: template compilation
    buttonTemplate: "module.Buttons",
    props(genericProps, view) {
        const archInfo = new MyArchParser().parse(genericProps.arch);
        return { ...genericProps, archInfo, Model: RelationalModel, Renderer: MyRenderer };
    },
};

registry.category("views").add("my_view", myView);
```

### Standard views

| View | Controller | Renderer | Model |
|------|-----------|----------|-------|
| `form` | FormController | FormRenderer | RelationalModel |
| `list` | ListController | ListRenderer | RelationalModel |
| `kanban` | KanbanController | KanbanRenderer | RelationalModel |
| `calendar` | CalendarController | CalendarRenderer | CalendarModel |
| `graph` | GraphController | GraphRenderer | GraphModel |
| `pivot` | PivotController | PivotRenderer | PivotModel |

---

## Field Widgets

88+ registered widgets. Register custom ones:

```javascript
import { registry } from "@web/core/registry";
import { Component } from "@odoo/owl";
import { standardFieldProps } from "@web/views/fields/standard_field_props";

export class MyWidget extends Component {
    static template = "my_module.MyWidget";
    static props = { ...standardFieldProps };

    get formattedValue() {
        return this.props.record.data[this.props.name];
    }

    onChange(ev) {
        this.props.record.update({ [this.props.name]: ev.target.value });
    }
}

registry.category("fields").add("my_widget", {
    component: MyWidget,
    displayName: "My Custom Widget",
    supportedTypes: ["char", "text"],
    extractProps({ attrs }) {
        return { placeholder: attrs.placeholder };
    },
});
```

### Common widgets by type

| Type | Widget names |
|------|-------------|
| **Text** | `char`, `text`, `html`, `url`, `email`, `phone`, `code`/`ace` |
| **Number** | `integer`, `float`, `float_time`, `float_factor`, `monetary`, `percentage`, `percentpie` |
| **Boolean** | `boolean`, `boolean_toggle`, `boolean_favorite`, `boolean_icon` |
| **Selection** | `selection`, `radio`, `selection_badge`, `label_selection`, `filterable_selection`, `statusbar` |
| **Relational** | `many2one`, `many2many_tags`, `many2many_checkboxes`, `one2many`/`many2many` (x2many), `many2one_avatar`, `many2many_tags_avatar` |
| **Date** | `date`, `datetime` |
| **Binary** | `binary`, `image`, `image_url`, `attachment_image`, `pdf_viewer`, `signature` |
| **Display** | `badge`, `priority`, `statinfo`, `gauge`, `color`, `color_picker`, `handle`, `progress_bar`, `remaining_days`, `dashboard_graph` |
| **Utility** | `domain`, `properties`, `CopyClipboardChar`, `CopyClipboardURL`, `jsonb` |

Usage in XML:
```xml
<field name="my_field" widget="my_widget"/>
```

---

## Template Directives

OWL templates use QWeb-like syntax:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<templates xml:space="preserve">
    <t t-name="my_module.MyComponent">
        <!-- Conditional -->
        <div t-if="state.loading">Loading...</div>
        <div t-elif="state.error">Error!</div>
        <div t-else="">
            <!-- Loop -->
            <t t-foreach="state.items" t-as="item" t-key="item.id">
                <span t-esc="item.name"/>
            </t>
        </div>

        <!-- Dynamic attributes -->
        <div t-att-class="{'active': state.isActive, 'disabled': !state.enabled}"/>
        <div t-attf-class="btn btn-{{state.variant}}"/>

        <!-- Events -->
        <button t-on-click="onSave">Save</button>
        <button t-on-click.stop.prevent="onCancel">Cancel</button>

        <!-- Sub-component -->
        <MyChild record="props.record" onUpdate.bind="onChildUpdate"/>

        <!-- Slot -->
        <t t-slot="default"/>

        <!-- Ref -->
        <input t-ref="myInput"/>

        <!-- Raw HTML (careful: XSS risk) -->
        <div t-out="state.htmlContent"/>
    </t>
</templates>
```

### Event modifiers

| Modifier | Effect |
|----------|--------|
| `.stop` | `stopPropagation()` |
| `.prevent` | `preventDefault()` |
| `.capture` | Capture phase |
| `.once` | Fire once |
| `.self` | Only if target is the element itself |

---

## Asset Bundles

Declare in `__manifest__.py`:

```python
'assets': {
    # Backend (web client)
    'web.assets_backend': [
        'my_module/static/src/**/*.js',
        'my_module/static/src/**/*.xml',
        'my_module/static/src/**/*.scss',
    ],
    # Frontend (portal/website)
    'web.assets_frontend': [
        'my_module/static/src/public/**/*.js',
        'my_module/static/src/public/**/*.xml',
    ],
    # Tests
    'web.assets_unit_tests': [
        'my_module/static/tests/**/*.js',
    ],
}
```

| Bundle | Context |
|--------|---------|
| `web.assets_backend` | Backend web client |
| `web.assets_frontend` | Portal, website public pages |
| `web.assets_frontend_minimal` | Login page, minimal frontend |
| `web.assets_frontend_lazy` | Lazy-loaded frontend components |
| `web.assets_unit_tests` | Hoot test files |
| `web.qunit_suite_tests` | QUnit tests (legacy) |

### Dynamic loading

```javascript
import { loadBundle } from "@web/core/assets";

await loadBundle("my_module.extra_assets");
```

---

## Common Patterns

### Extending a view controller

```javascript
import { patch } from "@web/core/utils/patch";
import { FormController } from "@web/views/form/form_controller";

patch(FormController.prototype, {
    setup() {
        super.setup();
        this.customService = useService("my_service");
    },
    async onRecordSaved(record) {
        await super.onRecordSaved(record);
        this.customService.afterSave(record.resId);
    },
});
```

### Custom systray item

```javascript
export class MySystrayItem extends Component {
    static template = "my_module.SystrayItem";
}
registry.category("systray").add("my_module.MySystrayItem", {
    Component: MySystrayItem,
}, { sequence: 50 });
```

### publicWidget (frontend — v18 way)

```javascript
import publicWidget from "@web/legacy/js/public/public_widget";

publicWidget.registry.MyWidget = publicWidget.Widget.extend({
    selector: ".my-selector",
    events: { "click .btn": "_onClick" },
    start() { return this._super(...arguments); },
    _onClick(ev) { /* handle */ },
});
```

### Tour definition (v18)

```javascript
import { registry } from "@web/core/registry";

registry.category("web_tour.tours").add("my_tour", {
    url: "/web",
    steps: () => [
        {
            trigger: ".o_kanban_view .o_kanban_record:first",
            content: "Click the first record",
            run: "click",
        },
        {
            trigger: ".o_form_button_save",
            content: "Save the record",
            run: "click",
        },
    ],
});
```
