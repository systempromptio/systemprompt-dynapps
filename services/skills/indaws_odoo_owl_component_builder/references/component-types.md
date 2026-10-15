# Component Types — Minimal Skeletons (OWL backend, v17-v19)

Cada arquetipo trae: fichero JS, plantilla XML OWL y registro en `registry`.
Sustituye `mi_modulo` por el nombre real del modulo. Para firmas exactas por
version, consulta `odoo-version-api-reference`.

Layout estandar:
```
mi_modulo/static/src/<feature>/<feature>.js
mi_modulo/static/src/<feature>/<feature>.xml
mi_modulo/static/src/<feature>/<feature>.scss   (opcional)
```

---

## 1. Field widget

JS (`static/src/field_color/field_color.js`):
```js
/** @odoo-module **/
import { Component } from "@odoo/owl";
import { registry } from "@web/core/registry";
import { _t } from "@web/core/l10n/translation";
import { standardFieldProps } from "@web/views/fields/standard_field_props";

export class ColorField extends Component {
    static template = "mi_modulo.ColorField";
    static props = { ...standardFieldProps };
    get value() {
        return this.props.record.data[this.props.name] || "#000000";
    }
    // WRITE-BACK: así se hace editable un field widget
    onChange(ev) {
        this.props.record.update({ [this.props.name]: ev.target.value });
    }
}

export const colorField = {
    component: ColorField,
    displayName: _t("Color"),
    supportedTypes: ["char"],
    // firma genérica: extractProps(fieldInfo, dynamicInfo) — destructura solo lo que uses
    extractProps: ({ attrs, options }) => ({}),
};
registry.category("fields").add("color_picker", colorField);
```

XML (`field_color.xml`):
```xml
<?xml version="1.0" encoding="UTF-8"?>
<templates xml:space="preserve">
    <t t-name="mi_modulo.ColorField">
        <input type="color" t-att-value="value"
               t-on-change="onChange" t-att-disabled="props.readonly"/>
    </t>
</templates>
```

Uso en vista: `<field name="x_color" widget="color_picker"/>`.
Notas v17-v19:
- Registra el **objeto descriptor** (`colorField`), NO la clase.
- Lectura: `props.record.data[props.name]`; escritura:
  `props.record.update({ [props.name]: valor })` (ver `color_field.js` estándar).
- `displayName` con `_t(...)`; `extractProps` tiene firma genérica
  `extractProps(fieldInfo, dynamicInfo)` — destructura solo lo que uses. `dynamicInfo`
  NO es exclusivo de v19 (existe desde v17); las claves del 1er arg varían por campo
  (p.ej. `char_field` v19 usa `({ attrs, options, placeholder })` sin `dynamicInfo`).
  Confirma la firma con `odoo-version-api-reference`.

---

## 2. View component

Componente de vista completa (o sub-componente reutilizable). Para una vista
custom se registra en `category("views")` con su `arch parser`, `model` y
`controller`; para extender una vista estandar, hereda su controller/renderer.

```js
/** @odoo-module **/
import { registry } from "@web/core/registry";
import { listView } from "@web/views/list/list_view";
import { ListController } from "@web/views/list/list_controller";

export class MyListController extends ListController {
    setup() {
        super.setup();
        // logica extra
    }
}
registry.category("views").add("my_list", {
    ...listView,
    Controller: MyListController,
});
```

Uso en `arch`: `<list js_class="my_list">`.

---

## 3. Client action

JS (`static/src/dashboard/dashboard.js`):
```js
/** @odoo-module **/
import { Component, useState } from "@odoo/owl";
import { registry } from "@web/core/registry";
import { useService } from "@web/core/utils/hooks";

export class Dashboard extends Component {
    static template = "mi_modulo.Dashboard";
    setup() {
        this.orm = useService("orm");
        this.state = useState({ count: 0 });
    }
    async load() {
        this.state.count = await this.orm.searchCount("res.partner", []);
    }
}
registry.category("actions").add("mi_modulo.dashboard", Dashboard);
```

XML:
```xml
<t t-name="mi_modulo.Dashboard">
    <div class="o_mi_dashboard">
        <button class="btn btn-primary" t-on-click="load">Load</button>
        <span t-esc="state.count"/>
    </div>
</t>
```

Disparo: `ir.actions.client` con `tag="mi_modulo.dashboard"`, o
`action.doAction({type: "ir.actions.client", tag: "mi_modulo.dashboard"})`.

---

## 4. Systray item

```js
/** @odoo-module **/
import { Component } from "@odoo/owl";
import { registry } from "@web/core/registry";
import { useService } from "@web/core/utils/hooks";

export class MySystray extends Component {
    static template = "mi_modulo.MySystray";
    setup() { this.action = useService("action"); }
    open() { this.action.doAction("mi_modulo.some_action"); }
}
registry.category("systray").add(
    "mi_modulo.MySystray",
    { Component: MySystray },
    { sequence: 50 },
);
```
XML: un `<div class="o_mail_systray_item">` con tu icono/boton.
La categoria es `systray`; el `sequence` ordena los iconos.

---

## 5. Dialog

```js
/** @odoo-module **/
import { Component } from "@odoo/owl";
import { Dialog } from "@web/core/dialog/dialog";

export class ConfirmDialog extends Component {
    static template = "mi_modulo.ConfirmDialog";
    static components = { Dialog };
    static props = { close: Function, title: String, onConfirm: Function };
    confirm() { this.props.onConfirm(); this.props.close(); }
}
```
XML:
```xml
<t t-name="mi_modulo.ConfirmDialog">
    <Dialog title="props.title">
        <p>Are you sure?</p>
        <t t-set-slot="footer">
            <button class="btn btn-primary" t-on-click="confirm">Confirm</button>
            <button class="btn btn-secondary" t-on-click="props.close">Cancel</button>
        </t>
    </Dialog>
</t>
```
Apertura desde otro componente: `this.dialog = useService("dialog");`
`this.dialog.add(ConfirmDialog, { title: "X", onConfirm: () => {...} });`

---

## 6. Service / Hook

Service (singleton inyectable):
```js
/** @odoo-module **/
import { registry } from "@web/core/registry";

export const myService = {
    dependencies: ["orm"],
    start(env, { orm }) {
        return {
            async fetchThing(id) { return orm.read("my.model", [id], ["name"]); },
        };
    },
};
registry.category("services").add("my_service", myService);
```
Consumo: `const my = useService("my_service");`

Hook (logica reutilizable sin estado global):
```js
/** @odoo-module **/
import { useState, onWillStart } from "@odoo/owl";
import { useService } from "@web/core/utils/hooks";

export function usePartnerCount() {
    const orm = useService("orm");
    const state = useState({ count: 0 });
    onWillStart(async () => {
        state.count = await orm.searchCount("res.partner", []);
    });
    return state;
}
```
