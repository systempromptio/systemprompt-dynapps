# Recipes — Complete Production Components (OWL backend, v17-v19)

Four end-to-end recipes (JS + XML + registry + manifest). Each mirrors a real
core widget cited as `web:path:line`. Replace `my_module` with the real module
name. Verified against `$ODOO_SRC/odoo19/addons/web/static/src/`.

---

## 1. Editable field widget with `options` + write-back

Reference: `web/static/src/views/fields/boolean/boolean_field.js` (write-back via
`record.update`, l. 30) and `web/static/src/views/fields/char/char_field.js:116`, `charField` (`extractProps`
real signature `({ attrs, options, placeholder })`).

JS (`static/src/fields/percent_field/percent_field.js`):
```js
import { Component } from "@odoo/owl";
import { registry } from "@web/core/registry";
import { _t } from "@web/core/l10n/translation";
import { useRecordObserver } from "@web/model/relational_model/utils";
import { standardFieldProps } from "@web/views/fields/standard_field_props";

export class PercentField extends Component {
    static template = "my_module.PercentField";
    static props = {
        ...standardFieldProps,
        step: { type: Number, optional: true },
    };
    static defaultProps = { step: 1 };

    setup() {
        this.state = useState({ value: 0 });
        // Keeps the widget in sync when the record changes elsewhere.
        useRecordObserver((record) => {
            this.state.value = record.data[this.props.name] || 0;
        });
    }

    onChange(ev) {
        const value = Number(ev.target.value) || 0;
        this.state.value = value;
        // WRITE-BACK: never mutate props.record.data directly.
        this.props.record.update({ [this.props.name]: value });
    }
}

export const percentField = {
    component: PercentField,
    displayName: _t("Percent"),
    supportedTypes: ["integer", "float"],
    // v19 signature: ({ attrs, options, placeholder }, dynamicInfo)
    extractProps: ({ options }) => ({ step: options.step || 1 }),
};
registry.category("fields").add("percent", percentField);
```
Need `useState`/`useRecordObserver` imports too:
`import { Component, useState } from "@odoo/owl";`

XML (`percent_field.xml`):
```xml
<?xml version="1.0" encoding="UTF-8"?>
<templates xml:space="preserve">
    <t t-name="my_module.PercentField">
        <input type="number" class="o_input"
               t-att-value="state.value" t-att-step="props.step"
               t-att-readonly="props.readonly" t-on-change="onChange"/>
    </t>
</templates>
```
Use in view: `<field name="x_margin" widget="percent" options="{'step': 5}"/>`
Manifest: add the `.js` + `.xml` to `web.assets_backend`.

---

## 2. Client action dashboard — `useService("orm")` + loading state

Reference: ORM service `web/static/src/core/orm_service.js` (registered
`category("services").add("orm", ...)`); action trigger pattern
`web/static/src/webclient/actions/action_service.js:1885`, `actionService`.

JS (`static/src/dashboard/sales_dashboard.js`):
```js
import { Component, useState, onWillStart } from "@odoo/owl";
import { registry } from "@web/core/registry";
import { useService } from "@web/core/utils/hooks";

export class SalesDashboard extends Component {
    static template = "my_module.SalesDashboard";

    setup() {
        this.orm = useService("orm");
        this.action = useService("action");
        this.state = useState({ loading: true, total: 0, orders: [] });
        onWillStart(() => this.load());
    }

    async load() {
        this.state.loading = true;
        const orders = await this.orm.searchRead(
            "sale.order", [["state", "=", "sale"]],
            ["name", "amount_total"], { limit: 20 },
        );
        this.state.orders = orders;
        this.state.total = orders.reduce((s, o) => s + o.amount_total, 0);
        this.state.loading = false;
    }

    openOrder(id) {
        this.action.doAction({
            type: "ir.actions.act_window",
            res_model: "sale.order",
            res_id: id,
            views: [[false, "form"]],
        });
    }
}
registry.category("actions").add("my_module.sales_dashboard", SalesDashboard);
```

XML:
```xml
<t t-name="my_module.SalesDashboard">
    <div class="o_sales_dashboard p-3">
        <t t-if="state.loading">
            <div class="o_loading">Loading…</div>
        </t>
        <t t-else="">
            <h3>Total: <t t-esc="state.total"/></h3>
            <ul>
                <li t-foreach="state.orders" t-as="o" t-key="o.id"
                    t-on-click="() => this.openOrder(o.id)">
                    <t t-out="o.name"/> — <t t-esc="o.amount_total"/>
                </li>
            </ul>
        </t>
    </div>
</t>
```
Wire it to a menu via `ir.actions.client` with `tag="my_module.sales_dashboard"`.

---

## 3. Systray item — `bus_service.subscribe` + notification

Reference: bus service `bus/static/src/services/bus_service.js:158`, `startWorker`
(`subscribe(notificationType, callback)`); notification service
`web/static/src/core/notifications/notification_service.js:43`, `notificationService.start` (`add(message, options)`).

JS (`static/src/systray/alert_systray.js`):
```js
import { Component, useState } from "@odoo/owl";
import { registry } from "@web/core/registry";
import { useService } from "@web/core/utils/hooks";

export class AlertSystray extends Component {
    static template = "my_module.AlertSystray";

    setup() {
        this.bus = useService("bus_service");
        this.notification = useService("notification");
        this.state = useState({ count: 0 });
        // subscribe returns nothing; bus_service auto-cleans on unmount of env.
        this.bus.subscribe("my_module.alert", (payload) => {
            this.state.count++;
            this.notification.add(payload.message, { type: "warning" });
        });
        this.bus.start();
    }
}
registry.category("systray").add(
    "my_module.AlertSystray",
    { Component: AlertSystray },
    { sequence: 25 },
);
```

XML:
```xml
<t t-name="my_module.AlertSystray">
    <div class="o_AlertSystray dropdown">
        <button class="btn"><i class="fa fa-bell"/>
            <span t-if="state.count" class="badge" t-esc="state.count"/>
        </button>
    </div>
</t>
```
Server side pushes with `self.env['bus.bus']._sendone(channel, 'my_module.alert',
{'message': '...'})`. (Channel/push is server scope — not this skill.)

---

## 4. Dialog with a form — `useService("dialog")`

Reference: Dialog component `web/static/src/core/dialog/dialog.js`
(props `title`, `size`, footer slot) and `web/static/src/core/dialog/dialog_service.js:27`, `dialogService`
(`dialogService`, `dialog.add(Component, props)`).

JS (`static/src/dialogs/reason_dialog.js`):
```js
import { Component, useState } from "@odoo/owl";
import { Dialog } from "@web/core/dialog/dialog";
import { _t } from "@web/core/l10n/translation";

export class ReasonDialog extends Component {
    static template = "my_module.ReasonDialog";
    static components = { Dialog };
    static props = {
        close: Function,             // injected by dialog service
        title: { type: String, optional: true },
        onConfirm: Function,
    };
    static defaultProps = { title: _t("Reason") };

    setup() {
        this.state = useState({ reason: "" });
    }

    confirm() {
        this.props.onConfirm(this.state.reason);
        this.props.close();
    }
}
```

XML:
```xml
<t t-name="my_module.ReasonDialog">
    <Dialog title="props.title" size="'md'">
        <div class="mb-2">
            <label>Reason</label>
            <textarea class="form-control" t-model="state.reason"/>
        </div>
        <t t-set-slot="footer">
            <button class="btn btn-primary" t-on-click="confirm">Confirm</button>
            <button class="btn btn-secondary" t-on-click="() => props.close()">
                Cancel
            </button>
        </t>
    </Dialog>
</t>
```
Open from any component:
```js
this.dialog = useService("dialog");
this.dialog.add(ReasonDialog, {
    title: _t("Cancel order"),
    onConfirm: (reason) => this.orm.call("sale.order", "action_cancel",
        [[orderId]], { context: { cancel_reason: reason } }),
});
```
Manifest: all four recipes need their `.js` + `.xml` in `web.assets_backend`.
