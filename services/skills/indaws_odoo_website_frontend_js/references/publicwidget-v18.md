# publicWidget (v17 / v18)

Source: `$ODOO_SRC/odoo18/addons/web/static/src/legacy/js/public/public_widget.js`

`publicWidget` is the legacy framework for public/interactive JS on website pages.
A widget is instantiated **automatically, once per DOM element** matching its
`selector`, when the public page is initialized. It is jQuery-flavored.

## Minimal skeleton

```js
/** @odoo-module **/
import publicWidget from "@web/legacy/js/public/public_widget";
import { rpc } from "@web/core/network/rpc";

publicWidget.registry.MyWidget = publicWidget.Widget.extend({
    selector: ".s_my_snippet",          // REQUIRED: entry point (CSS selector)
    // disabledInEditableMode defaults to true for Animation; for Widget there is
    // no default flag, but set it explicitly when behavior matters in the editor.
    events: {
        "click .my-btn": "_onMyBtnClick",         // jQuery-delegated, rooted in $el
        "click .async-btn": "async _onAsyncClick", // 'async' -> button-locked handler
    },

    /**
     * Async work the render needs. MUST return a Promise.
     * @override
     */
    willStart() {
        return this._super(...arguments).then(async () => {
            this.data = await rpc("/my/endpoint", { foo: 1 });
        });
    },

    /**
     * Runs AFTER render; this.$el is valid here.
     * @override
     */
    start() {
        this.timer = setInterval(() => this._tick(), 1000);
        return this._super(...arguments);
    },

    /**
     * Clean up EVERYTHING (intervals, listeners, rAF). Called on destroy AND
     * when entering edit mode.
     * @override
     */
    destroy() {
        clearInterval(this.timer);
        this._super(...arguments);
    },

    _onMyBtnClick(ev) {
        const $card = this.$(".card");   // this.$(sel) == this.$el.find(sel)
        $card.toggleClass("active");
    },
});

export default publicWidget.registry.MyWidget;
```

Manifest:

```python
"assets": {
    "web.assets_frontend": [
        "my_module/static/src/js/my_widget.js",
    ],
},
```

## Key API (verified, file:line in public_widget.js)

| Member | Purpose | Line |
|--------|---------|------|
| `selector` | CSS selector (or function). `false` by default → widget never runs | 496 |
| `selectorHas` | Filter matches to those containing this selector (safer than `:has`) | 509 |
| `events` | `{'evt sel': 'method'}` jQuery-delegated; `'async method'` wraps as button/async handler | 532, 756-805 |
| `template` | QWeb template name for `renderElement` (when not attached to existing DOM) | 450 |
| `cssLibs`/`jsLibs`/`assetLibs` | external libs / bundles loaded in `willStart` | 458-476, 553-573 |
| `init(parent, options)` | constructor; sets parent + `this.options` | 539-543 |
| `willStart()` | async pre-render work; returns Promise | 553-574 |
| `start()` | post-render; `this.$el` valid; returns Promise | 587-589 |
| `destroy()` | cleanup; undelegates events; removes `$el` only if NOT selector-based | 596-607 |
| `this.$el` / `this.el` | jQuery root / native root | 722-723 |
| `this.$(sel)` | `this.$el.find(sel)` helper | 746-751 |
| `this._rpc(...)` | legacy rpc via ServicesMixin — prefer `import { rpc }` | (ServicesMixin) |

## Registering (v18 pattern)

Assign onto the `publicWidget.registry` object — NOT `registry.add()`:

```js
publicWidget.registry.searchBar = publicWidget.Widget.extend({ selector: ".o_searchbar_form", ... });
```

Real example: `$ODOO_SRC/odoo18/addons/website/static/src/snippets/s_searchbar/000.js:13`.

## Animation widgets & edit mode

`publicWidget.Widget.include({ disabledInEditableMode: true })` is the global default
for the **Animation** subsystem (`website/static/src/js/content/snippets.animation.js:50,59`).
- `disabledInEditableMode: true` (default) → the widget is destroyed/not started in the
  website builder edit mode.
- Set `disabledInEditableMode: false` when the JS must keep running in the editor (real
  snippets that need it set it explicitly, e.g. `website/static/src/js/content/snippets.animation.js:466`, `registry`).

`Animation` (subclass of `publicWidget.Widget`, `website/static/src/js/content/snippets.animation.js:341`, `Animation`) adds
`AnimationEffect` with `requestAnimationFrame` loops and scroll/resize start events
(`website/static/src/js/content/snippets.animation.js:96-306`). Use it for scroll-driven animations; clean up the rAF
loop on `destroy` (the base `destroy` — `website/static/src/js/content/snippets.animation.js:185`, `destroy` — calls `stop` (`:256`), and the `cancelAnimationFrame` is two hops down in `pause`, `website/static/src/js/content/snippets.animation.js:281`, `pause`).

## Real reference snippet

`s_countdown/000.js` is a complete, production widget: `start()` reads `this.el.dataset.*`,
sets up `setInterval(this._render, 1000)`, and `destroy()` does `clearInterval` +
`removeEventListener` — the canonical cleanup pattern.
