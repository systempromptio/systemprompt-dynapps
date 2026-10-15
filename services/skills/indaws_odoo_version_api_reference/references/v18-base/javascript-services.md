<!-- verified: v18.0 @ 2026-06 -->
# JavaScript — Services & ORM (Odoo 18.0)

Access registered services via `useService("name")` in setup().

> **CUIDADO (v18):** `rpc` y `user` NO son servicios. Se importan como funciones/singletons:
> - `import { rpc } from "@web/core/network/rpc";` — `rpc(route, params)`
> - `import { user } from "@web/core/user";` — singleton con `userId`, `name`, `hasGroup(group)`, etc. (NO `useService("user")`)

## Core services

| Service | Methods / Properties | Purpose |
|---------|---------------------|---------|
| `orm` | `read()`, `readGroup()`, `search()`, `searchRead()`, `searchCount()`, `create()`, `write()`, `unlink()`, `call()`, `webSearchRead()` | ORM operations |
| `http` | `get(route, readMethod)`, `post(route, params, readMethod)` | HTTP requests |
| `action` | `doAction(action, options)`, `switchView(viewType)`, `restore()` | Navigate/execute actions |
| `notification` | `add(message, { type, sticky, title, className })` | Toast notifications |
| `dialog` | `add(Component, props, options)` | Open dialogs |
| `company` | `activeCompanyIds`, `currentCompany`, **`allowedCompanies`** (+ `allowedCompaniesWithAncestors`, `disallowedAncestorCompanies`, `getCompany(id)`) | Company context. **Corregido 2026-08-18: decía `allowedCompanyIds`, que NO existe** — 0 ocurrencias en `web/static/src` de v18 y de v19. El objeto real que devuelve el servicio está en `web/static/src/webclient/company_service.js`, y las dos que sí son getters son `activeCompanyIds` (`:85`) y `currentCompany` (`:89`); las otras tres son propiedades planas |
| `currency` | Currency formatting utilities | Money formatting |
| `effect` | `add({ type, message })` | Visual effects (rainbow man) |
| `hotkey` | `add(keys, callback, options)` | Keyboard shortcuts |
| `ui` | `activeElement`, `size`, `isSmall`, `block()`, `unblock()` | UI state, screen size |
| `title` | `current`, `setParts(parts)` | Browser tab title |
| `command` | `add(name, action, options)` | Command palette |
| `menu` | `getMenu(menuId)`, `getApps()`, `selectMenu(menu)` | Menu navigation |
| `view` | `loadViews(params)`, `loadFields(resModel)` | View/field loading |
| `field` | `loadFields(resModel)`, `loadPath(resModel, path)` | Field definitions |
| `name` | `loadDisplayNames(resModel, ids)` | Display name resolution |
| `localization` | Number/date formatting per locale | L10n |
| `file_upload` | `upload(route, files, params)` | File upload with progress |
| `overlay` | `add(Component, props)` | Overlay management |
| `popover` | `add(target, Component, props)` | Popover management |
| `tooltip` | Tooltip management | Tooltips |
| `sortable` | Drag-and-drop | Sortable lists |
| `datetime_picker` | Date/time picker | Date selection |
| `profiling` | Performance profiling | Debug |
| `pwa` | Progressive Web App | PWA support |

## ORM service usage

```javascript
setup() {
    this.orm = useService("orm");
}

// CRUD
const partners = await this.orm.searchRead("res.partner", [["active", "=", true]], ["name", "email"], { limit: 10 });
const ids = await this.orm.create("res.partner", [{ name: "New" }]);
await this.orm.write("res.partner", [1], { name: "Updated" });
await this.orm.unlink("res.partner", [1]);

// Call arbitrary method
const result = await this.orm.call("sale.order", "action_confirm", [[orderId]]);

// Read group
const groups = await this.orm.readGroup("sale.order", [["state", "=", "sale"]], ["partner_id"], ["partner_id"]);
```

## Registering a service

```javascript
registry.category("services").add("myService", {
    dependencies: ["orm", "notification"],
    start(env, { orm, notification }) {
        return {
            async doSomething(id) {
                const result = await orm.call("my.model", "my_method", [id]);
                notification.add("Done!", { type: "success" });
                return result;
            },
        };
    },
});
```
