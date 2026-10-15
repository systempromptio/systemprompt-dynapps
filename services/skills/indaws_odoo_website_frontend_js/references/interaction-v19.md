# Interaction framework (v19)

Sources:
- `$ODOO_SRC/odoo19/addons/web/static/src/public/interaction.js` (base class)
- `$ODOO_SRC/odoo19/addons/web/static/src/public/interaction_service.js` (lifecycle/dispatch)
- `$ODOO_SRC/odoo19/addons/website/static/src/interactions/_example.js` (canonical OWL example)
- `$ODOO_SRC/odoo19/addons/website/static/src/interactions/image_lazy_loading.js` (real interaction)

`Interaction` is the v19 successor to `publicWidget`: same idea (one instance per element
matching `selector`, created when the public page is initialized) but **no jQuery**, a clean
lifecycle, and a declarative `dynamicContent`. `publicWidget` still exists in v19 for legacy,
but new public JS uses `Interaction`.

## Minimal skeleton

```js
import { Interaction } from "@web/public/interaction";
import { registry } from "@web/core/registry";
import { rpc } from "@web/core/network/rpc";

export class MySnippet extends Interaction {
    static selector = ".s_my_snippet";          // REQUIRED, MUST be static (on class)

    // dynamicContent MUST be on the INSTANCE, not static (service throws otherwise)
    dynamicContent = {
        ".my-btn": { "t-on-click": (ev) => this.onClick(ev) },
        ".counter": { "t-out": () => this.count },
        ".badge": { "t-att-class": () => ({ "text-bg-danger": this.count > 5 }) },
    };

    setup() {                                    // standard constructor; el + services ready
        this.count = 0;
    }

    async willStart() {                          // async pre-start; framework awaits it
        this.data = await this.waitFor(rpc("/my/endpoint", { foo: 1 }));
    }

    start() {                                    // equivalent to "mounted"; handlers attached
        // optional manual listener (auto-cleaned on destroy):
        this.addListener(window, "resize", () => this.onResize());
    }

    destroy() {                                  // cleanup; registered cleanups run automatically
    }

    onClick(ev) {
        this.count++;                            // updateContent runs after handler → t-out/t-att refresh
    }
}

registry.category("public.interactions").add("my_module.my_snippet", MySnippet);
```

Manifest: declare in `web.assets_frontend` (same as publicWidget).

## Lifecycle (interaction.js)

| Method | Purpose | Line |
|--------|---------|------|
| `setup()` | constructor replacement; `this.el`, `this.env`, `this.services` ready | 149 |
| `async willStart()` | async work before dynamicContent is applied | 156 |
| `start()` | "mounted" equivalent; handlers attached | 163 |
| `destroy()` | cleanup; no need to call super unless subclassing a concrete class | 170 |

`this.el` is the native DOM element (no `$el`). `this.services` = `env.services`
(interaction.js:122-125).

## dynamicContent directives (interaction.js:79-110)

```js
dynamicContent = {
    "<selector>": {
        "t-on-<event>": (ev) => this.handler(ev),     // event listener
        "t-att-<attr>": () => value,                  // attribute (false/null/undefined remove it; true → attr name)
        "t-att-class": () => ({ "cls": cond }),       // class toggling
        "t-att-style": () => ({ property: value }),   // inline style
        "t-out": () => value,                         // textContent
        "t-component": () => [Component, { props }],  // mount an OWL component
    },
    _root: { ... },   // special selectors: _root, _body, _document, _window (interaction.js:66-71)
};
```

This is **NOT OWL** — similar syntax, different engine (colibri). Values are functions.
`Interaction.INITIAL_VALUE` resets a `t-att-*`/`t-out` to its pre-start value (interaction.js:57,102).

## Helpers (auto-cleanup, interaction.js)

| Helper | What | Line |
|--------|------|------|
| `addListener(target, event, fn, opts)` | listener that runs `updateContent` and auto-removes on destroy | 365 |
| `waitFor(promise)` | resolve only if not destroyed, then `updateContent` | 192 |
| `waitForTimeout(fn, delay)` / `waitForAnimationFrame(fn)` | destroy-safe setTimeout / rAF | 253, 269 |
| `debounced(fn, delay)` / `throttled(fn)` | debounce/throttle, cancelled on destroy | 284, 316 |
| `locked(fn, useLoadingAnimation)` | prevent re-entry; optional button loading | 345 |
| `registerCleanup(fn)` | run fn on destroy | 453 |
| `insert/renderAt/removeChildren` | DOM ops that auto-revert on destroy | 386, 425, 402 |
| `mountComponent(el, C, props)` | mount an OWL component (root is `<owl-root contenteditable=false>`) | 465 |
| `updateContent()` | re-apply dynamicContent (rarely needed manually) | 183 |

## Registration & edit mode

Normal: `registry.category("public.interactions").add("module.name", MyInteraction)`
(real: `website/static/src/interactions/image_lazy_loading.js:47`, `website.image_lazy_loading`). The service collects all of them
(`web/static/src/public/interaction_service.js:222`, `class InteractionService`) and starts one per `selector` match.

Edit-mode variant (when behavior must differ in the website builder) goes in a separate
`*.edit.js` file, registered in `public.interactions.edit` with `{Interaction, mixin}`:

```js
// search_modal.edit.js:12-16 (verified)
registry.category("public.interactions.edit").add("website.search_modal", {
    Interaction: SearchModal,
    mixin: SearchModalEdit,
});
```

Mount-OWL canonical example: `_example.js` (`Counter`, `static selector = "#wrapwrap h1"`,
`static template = xml\`...\``, uses `useService("notification")`).

## Migration table: publicWidget (v18) -> Interaction (v19)

| publicWidget (v18) | Interaction (v19) | Note |
|--------------------|-------------------|------|
| `publicWidget.Widget.extend({...})` | `class X extends Interaction {}` | ES class, not `.extend` |
| `selector: ".x"` (instance) | `static selector = ".x"` | MUST be static; empty selector throws |
| `selectorHas` | `static selectorHas` / `static selectorNotHas` | also `selectorNotHas` added |
| `events: {"click .b": "_onB"}` | `dynamicContent = {".b": {"t-on-click": (ev)=>this.onB(ev)}}` | declarative, no jQuery delegation |
| `init(parent, options)` | `setup()` | no parent/options args |
| `willStart()` returns Promise | `async willStart()` | same role |
| `start()` | `start()` | "mounted" |
| `destroy()` + manual cleanup | `destroy()` + `registerCleanup` / auto-clean helpers | helpers auto-clean |
| `this.$el` / `this.$(sel)` (jQuery) | `this.el` + `this.el.querySelector(sel)` | native DOM only |
| `this._rpc(...)` | `import { rpc }` + `this.waitFor(rpc(...))` | rpc is an import |
| `publicWidget.registry.X = ...` | `registry.category("public.interactions").add("mod.x", X)` | category registry |
| `disabledInEditableMode` | `*.edit.js` + `public.interactions.edit` | edit behavior split out |
| `setInterval` cleaned in `destroy` | `waitForTimeout` / `debounced` / `registerCleanup` | destroy-safe by construction |
| `this.trigger_up` / OdooEvent bus | use `this.services` / OWL bus | legacy event bus dropped |
