<!-- verified: v18.0 @ 2026-06 -->
# JavaScript — Module System, OWL Components & Reactivity (Odoo 18.0)

Source: `$ODOO_SRC/odoo18/addons/web/static/src/`

Odoo 18 uses OWL 2 as its component framework. All custom JS must use ES modules (`/** @odoo-module **/`).

## Module System

All JS files must start with the module annotation:

```javascript
/** @odoo-module **/
import { Component, useState } from "@odoo/owl";
import { registry } from "@web/core/registry";
import { useService } from "@web/core/utils/hooks";
```

### Import path aliases

| Alias | Maps to |
|-------|---------|
| `@odoo/owl` | OWL framework (Component, hooks, etc.) |
| `@web/core/...` | `addons/web/static/src/core/...` |
| `@web/views/...` | `addons/web/static/src/views/...` |
| `@web/webclient/...` | `addons/web/static/src/webclient/...` |
| `@module_name/...` | `addons/module_name/static/src/...` |

---

## OWL Components

### Component structure

```javascript
/** @odoo-module **/
import { Component, useState, onMounted, onWillUnmount } from "@odoo/owl";

export class MyComponent extends Component {
    static template = "my_module.MyComponent";
    static props = {
        recordId: { type: Number },
        title: { type: String, optional: true },
    };

    setup() {
        this.state = useState({ count: 0, loading: false });
        this.orm = useService("orm");

        onMounted(() => {
            console.log("Component mounted");
        });
        onWillUnmount(() => {
            console.log("Cleaning up");
        });
    }

    async loadData() {
        this.state.loading = true;
        const data = await this.orm.read("res.partner", [this.props.recordId], ["name"]);
        this.state.loading = false;
        return data;
    }
}
```

### Lifecycle hooks (OWL native)

| Hook | When | Use for |
|------|------|---------|
| `setup()` | Component construction | Initialize state, services, subscribe to hooks |
| `onWillStart(async fn)` | Before first render | Async data loading |
| `onMounted(fn)` | After DOM insertion | DOM manipulation, third-party libs |
| `onWillUpdateProps(async fn)` | Before props change | Reload data on prop changes |
| `onWillRender(fn)` | Before every render | Last-minute state adjustments |
| `onRendered(fn)` | After every render | Post-render DOM work |
| `onPatched(fn)` | After re-render (not first) | Update DOM after state change |
| `onWillUnmount(fn)` | Before removal | Cleanup: timers, listeners, subscriptions |
| `onWillDestroy(fn)` | Component destruction | Final cleanup |
| `onError(fn)` | Descendant error | Error boundaries |

### Reactivity

```javascript
import { useState, useRef, useEffect } from "@odoo/owl";

setup() {
    // Reactive state — any change triggers re-render
    this.state = useState({ name: "", items: [] });

    // DOM reference
    this.inputRef = useRef("myInput");

    // Side effect — runs when dependencies change
    useEffect(
        () => {
            console.log("Name changed:", this.state.name);
            return () => { /* cleanup */ };
        },
        () => [this.state.name]  // dependencies
    );
}
```
