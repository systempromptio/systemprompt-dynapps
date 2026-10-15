# JavaScript Patterns — Odoo 19

Complete reference for migrating JavaScript from v17/v18 to v19.

## 1. `.include()` → `patch()` (CRITICAL)

The most common migration issue. `.include()` and `.extend()` no longer exist on widget classes.

### Detection
```bash
grep -rn "\.include(\|\.extend(" --include="*.js"
grep -rn "publicWidget\.registry\." --include="*.js"
```

### Before (v17/v18)
```javascript
odoo.define('my_module.MyWidget', function (require) {
    "use strict";
    var publicWidget = require('web.public.widget');

    publicWidget.registry.PortalSidebar.include({
        _onClickDownload: function (ev) {
            // custom logic
            this._super.apply(this, arguments);
        },
    });
});
```

### After (v19)
```javascript
/** @odoo-module **/
import { patch } from "@web/core/utils/patch";
import { PortalSidebar } from "@portal/interactions/portal_sidebar";

patch(PortalSidebar.prototype, {
    _onClickDownload(ev) {
        // custom logic
        super._onClickDownload(...arguments);
    },
});
```

### Key rules
- `patch(Class.prototype, {...})` for instance methods
- `patch(Class, {...})` for static methods
- Use `super.methodName(...arguments)` instead of `this._super.apply(this, arguments)`
- `super` only works in methods, NOT in arrow functions
- Import from the correct v19 path (not legacy paths)

---

## 2. `publicWidget` → `Interaction` class (CRITICAL)

In v19, `publicWidget.Widget` is replaced by the `Interaction` class for frontend/portal.

### Detection
```bash
grep -rn "publicWidget\|public_widget\|web\.public\.widget" --include="*.js"
```

### Before (v17/v18) — publicWidget
```javascript
odoo.define('my_module.MyWidget', function (require) {
    "use strict";
    var publicWidget = require('web.public.widget');

    publicWidget.registry.MyWidget = publicWidget.Widget.extend({
        selector: '.my-selector',
        events: {
            'click .btn': '_onClick',
        },
        start: function () {
            this._super.apply(this, arguments);
            // init code
        },
        _onClick: function (ev) {
            // handle click
        },
    });
});
```

### After (v19) — Interaction
```javascript
/** @odoo-module **/
import { Interaction } from "@web/public/interaction";
import { registry } from "@web/core/registry";

export class MyWidget extends Interaction {
    static selector = ".my-selector";

    dynamicContent = {
        '.btn': { 't-on-click': this.onClick },
    };

    setup() {
        // init code (replaces start())
    }

    onClick(ev) {
        // handle click
    }
}

registry.category("public.interactions").add("my_module.MyWidget", MyWidget);
```

### Interaction class API
- `static selector` — CSS selector to match elements
- `static selectorHas` — Additional `:has()` filter
- `dynamicContent` — Event bindings and dynamic attributes (replaces `events:`)
- `setup()` — Initialization (replaces `start()`)
- `willStart()` — Async initialization before render
- `destroy()` — Cleanup
- `this.el` — The matched DOM element
- `this.services` — Access services: `this.services['http']`, `this.services['rpc']`
- Helper methods: `this.debounced()`, `this.locked()`, `this.waitFor()`

---

## 3. `odoo.define()` → ES module (IMPORTANT)

### Detection
```bash
grep -rn "odoo\.define(" --include="*.js"
```

### Before (v17/v18)
```javascript
odoo.define('my_module.MyComponent', function (require) {
    "use strict";
    var Widget = require('web.Widget');
    var MyComponent = Widget.extend({...});
    return MyComponent;
});
```

### After (v19)
```javascript
/** @odoo-module **/
import { Component } from "@odoo/owl";

export class MyComponent extends Component {
    static template = "my_module.MyComponent";
    setup() { ... }
}
```

### Module naming
- File at `my_module/static/src/js/my_file.js` → module name `@my_module/js/my_file`
- Use relative imports within same addon: `import { X } from "./other_file"`
- Use full paths cross-addon: `import { X } from "@web/core/utils/patch"`

---

## 4. RPC service → fetch API (for frontend/portal)

### Detection
```bash
grep -rn "useService.*rpc\|this\.rpc\s*=" --include="*.js"
```

### Before (v17/v18)
```javascript
setup() {
    this.rpc = useService("rpc");
}
async fetchData() {
    const result = await this.rpc("/my/route", { param: value });
}
```

### After (v19) — Backend (web client)
```javascript
// RPC service IS available in backend
import { rpc } from "@web/core/network/rpc";

async fetchData() {
    const result = await rpc("/my/route", { param: value });
}
```

### After (v19) — Frontend/Portal (public pages)
```javascript
// RPC service NOT available in portal/public
// Use fetch directly or http service
async fetchData() {
    const response = await fetch("/my/route", {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'X-Csrf-Token': odoo.csrf_token || '',
        },
        body: JSON.stringify({
            jsonrpc: "2.0", method: "call",
            params: { param: value }, id: 1
        })
    });
    const data = await response.json();
    return data.result;
}
```

### In Interaction class (v19 portal)
```javascript
import { rpc } from "@web/core/network/rpc";

export class MyInteraction extends Interaction {
    static selector = ".my-selector";

    async setup() {
        // Use rpc directly (imported, not from service)
        const data = await rpc("/my/route", {});
    }
}
```

---

## 5. Patching OWL Components

### Patch a component's prototype
```javascript
import { patch } from "@web/core/utils/patch";
import { FormController } from "@web/views/form/form_controller";

patch(FormController.prototype, {
    setup() {
        super.setup(...arguments);
        // Add custom hook or state
        this.customState = useState({ myField: false });
    },

    async onClickCustomButton() {
        // new method
    },
});
```

### Patch a component's static properties
```javascript
patch(FormController, {
    template: "my_module.CustomFormController",
});
```

---

## 6. Services in v19

### Creating a service
```javascript
import { registry } from "@web/core/registry";

const myService = {
    dependencies: ["notification", "orm"],
    start(env, { notification, orm }) {
        return {
            async doSomething() {
                const result = await orm.call("res.partner", "some_method", []);
                notification.add("Done!", { type: "success" });
                return result;
            },
        };
    },
};

registry.category("services").add("my_service", myService);
```

### Using a service in a component
```javascript
import { useService } from "@web/core/utils/hooks";

class MyComponent extends Component {
    setup() {
        this.orm = useService("orm");
        this.notification = useService("notification");
        this.action = useService("action");
    }
}
```

### Built-in services (v19)
| Service | Import | Available in |
|---|---|---|
| `orm` | `useService("orm")` | Backend |
| `rpc` | `import { rpc } from "@web/core/network/rpc"` | Backend (import), Portal (import) |
| `http` | `useService("http")` | Both |
| `notification` | `useService("notification")` | Backend |
| `action` | `useService("action")` | Backend |
| `router` | `useService("router")` | Backend |
| `user` | `useService("user")` | Backend |
| `cookie` | `useService("cookie")` | Both |
| `title` | `useService("title")` | Backend |
| `effect` | `useService("effect")` | Backend |

---

## 7. Report Download Handler (specific to indaws_report_docx)

### Before (v17/v18)
```javascript
odoo.define('indaws_report_docx.ReportAction', function (require) {
    var ActionManager = require('web.ActionManager');

    ActionManager.include({
        _executeReportAction: function (action, options) {
            if (action.report_type === 'docx') {
                // custom download logic
                return this._triggerDownload(action, options, 'docx');
            }
            return this._super.apply(this, arguments);
        },
    });
});
```

### After (v19)
```javascript
/** @odoo-module **/
import { patch } from "@web/core/utils/patch";
import { ReportAction } from "@web/webclient/actions/reports/report_action";

patch(ReportAction.prototype, {
    async _executeReportAction(action, options) {
        if (action.report_type === 'docx') {
            return this._triggerDownload(action, options, 'docx');
        }
        return super._executeReportAction(...arguments);
    },
});
```

---

## 8. Portal Sidebar Extension (specific to indaws_report_docx)

### Before (v17/v18) — `.include()` on publicWidget
```javascript
odoo.define('indaws_report_docx_account.AccountPortalSidebar', function (require) {
    var publicWidget = require('web.public.widget');
    var PortalSidebar = require('portal.PortalSidebar');

    PortalSidebar.include({
        _onClickDownload: function (ev) {
            // custom docx download logic
            this._super.apply(this, arguments);
        },
    });
});
```

### After (v19) — `patch()` on Interaction
```javascript
/** @odoo-module **/
import { patch } from "@web/core/utils/patch";
// Find the correct v19 class to patch:
// Check $ODOO_SRC/odoo19/addons/account/static/src/interactions/
// or $ODOO_SRC/odoo19/addons/portal/static/src/interactions/
import { PortalHomeCounters } from "@portal/interactions/portal_home_counters";

patch(PortalHomeCounters.prototype, {
    getCountersAlwaysDisplayed() {
        return super.getCountersAlwaysDisplayed(...arguments).concat(['docx_count']);
    },
});
```

---

## 9. Asset Bundle Changes

### Before (v17/v18)
```python
# __manifest__.py
'assets': {
    'web.assets_backend': [
        'my_module/static/src/js/**/*.js',
        'my_module/static/src/web/**/*',  # Legacy web bundle
    ],
},
```

### After (v19)
```python
# __manifest__.py
'assets': {
    'web.assets_backend': [
        'my_module/static/src/**/*.js',
        'my_module/static/src/**/*.xml',  # OWL templates
    ],
    'web.assets_frontend': [
        'my_module/static/src/interactions/**/*.js',  # Portal interactions
    ],
},
```

### Key bundles
| Bundle | Usage |
|---|---|
| `web.assets_backend` | Web client (backend) |
| `web.assets_frontend` | Portal/website (frontend) |
| `web.assets_frontend_minimal` | Minimal frontend (login page) |
| `web.assets_frontend_lazy` | Lazy-loaded frontend |

---

## 10. Common Import Path Changes v18 → v19

| v18 Import | v19 Import |
|---|---|
| `require('web.public.widget')` | `import { Interaction } from "@web/public/interaction"` |
| `require('portal.PortalSidebar')` | `import { ... } from "@portal/interactions/..."` |
| `require('web.ActionManager')` | `import { ReportAction } from "@web/webclient/actions/reports/report_action"` |
| `require('web.core')` | `import { _t } from "@web/core/l10n/translation"` |
| `require('web.Widget')` | `import { Component } from "@odoo/owl"` |
| `require('web.rpc')` | `import { rpc } from "@web/core/network/rpc"` |
| `registry.category("services")` | Same |
| `registry.category("public_components")` | `registry.category("public.interactions")` |

---

## 11. Detection Commands (all at once)

```bash
# Find ALL v18 JS patterns that need migration
echo "=== .include() ===" && grep -rn "\.include(" --include="*.js"
echo "=== .extend() ===" && grep -rn "\.extend(" --include="*.js"
echo "=== odoo.define() ===" && grep -rn "odoo\.define(" --include="*.js"
echo "=== publicWidget ===" && grep -rn "publicWidget\|public_widget" --include="*.js"
echo "=== require() ===" && grep -rn "require(" --include="*.js" | grep -v node_modules
echo "=== useService('rpc') ===" && grep -rn "useService.*rpc" --include="*.js"
echo "=== this._super ===" && grep -rn "this\._super" --include="*.js"
echo "=== _serialize_exception ===" && grep -rn "_serialize_exception" --include="*.js"
echo "=== content_disposition ===" && grep -rn "content_disposition" --include="*.js"
```
