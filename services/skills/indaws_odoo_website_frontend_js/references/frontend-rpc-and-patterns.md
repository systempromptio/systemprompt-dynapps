# Frontend RPC, animations, lazy-loading & edit mode

## RPC / ajax from the public frontend

Modern pattern (v17+): import `rpc`, do NOT use `this._rpc`.

```js
import { rpc } from "@web/core/network/rpc";

// publicWidget (v18):
const data = await rpc("/shop/cart/update_json", { line_id: id, product_id: pid });

// Interaction (v19): wrap with waitFor so it is destroy-safe and refreshes content
const data = await this.waitFor(rpc("/my/endpoint", { foo: 1 }));
```

Verified real usage: `$ODOO_SRC/odoo18/addons/website_sale/static/src/js/website_sale.js:12`
(`import { rpc } from "@web/core/network/rpc"`) and `:216` (`rpc("/shop/cart/update_json", {...})`).

The route on the server side is a JSON controller (`type="json"` / `type="jsonrpc"` in v19),
written with `odoo-website-controller`. Access control lives THERE: a `rpc` to an
`auth='user'` route fails for an anonymous visitor — the JS cannot bypass it.

For raw HTTP/file uploads use the browser `fetch`/`XMLHttpRequest` directly; for ORM-style
JSON-RPC always prefer `rpc`.

## Animations

v18: subclass the `Animation` widget (`website/static/src/js/content/snippets.animation.js:341`, `Animation`)
and register `AnimationEffect`s with `requestAnimationFrame` loops bound to start events
(scroll/resize):

```js
import publicWidget from "@web/legacy/js/public/public_widget";
const MyAnim = publicWidget.registry.Animation.extend({
    selector: ".s_my_anim",
    disabledInEditableMode: true,   // do not animate while editing
    effects: [{
        startEvents: "scroll",
        update: "_updateOnScroll",
    }],
    _updateOnScroll(scroll) { /* ... */ },
});
```

The base manages the rAF loop and cancels it on destroy (`website/static/src/js/content/snippets.animation.js:280-287`, `pause`). Do not roll your own loop without cleanup.

v19: use the `Interaction` helpers — `waitForAnimationFrame(fn)` (`web/static/src/public/interaction.js:269`, `Interaction.waitForAnimationFrame`) for a
single destroy-safe rAF, `throttled(fn)` (`web/static/src/public/interaction.js:316`, `Interaction.throttled`) for scroll/resize handlers
throttled for animation. Register scroll/resize via `addListener` so they auto-clean.

```js
start() {
    this.onScroll = this.throttled(() => this.animate());
    this.addListener(window, "scroll", this.onScroll);
}
```

## Lazy-loading images

Odoo websites use native `loading="lazy"` on `<img>` by default. The framework adds a small
JS fix for browsers where lazy loading misbehaves. v19 canonical implementation
(`website/static/src/interactions/image_lazy_loading.js`, verified):

```js
export class ImageLazyLoading extends Interaction {
    static selector = "#wrapwrap img[loading='lazy']";
    setup() {
        this.initialHeight = this.el.style.minHeight;
        this.el.style.minHeight = "1px";   // force 1px so Chrome loads at all zooms
    }
    start() {
        onceAllImagesLoaded(this.el).then(() => {
            if (!this.isDestroyed) { this.el.style.minHeight = this.initialHeight; }
        });
    }
    destroy() { this.el.style.minHeight = this.initialHeight; }
}
registry.category("public.interactions").add("website.image_lazy_loading", ImageLazyLoading);
```

Pattern to copy for any "do X once an image/element is loaded": guard the async callback with
`!this.isDestroyed` and revert the side effect in `destroy()`.

## Edit mode (website builder)

The website builder destroys public widgets/interactions when entering edit mode so they do
not interfere with editing, then re-creates them when leaving it. Implications:

- Any side effect (interval, listener, rAF, injected DOM) MUST be cleaned up, or it leaks into
  the editor. v18: clean in `destroy()`. v19: prefer auto-cleaning helpers
  (`addListener`, `debounced`, `throttled`, `waitForTimeout`) or `registerCleanup(fn)`
  (`web/static/src/public/interaction.js:453`, `Interaction.registerCleanup`).
- v18: control whether the JS runs during editing with `disabledInEditableMode`
  (default `true` for animations, `website/static/src/js/content/snippets.animation.js:59`). `true` = not started in editor.
- v19: split edit-mode behavior into a `*.edit.js` file registering a mixin in
  `public.interactions.edit` (`{Interaction, mixin}`), verified in
  `website/static/src/interactions/search_modal.edit.js:13-16`, `website.search_modal`. Do not branch on edit mode inside the normal interaction.
- v19 OWL roots mounted from interactions are inserted as
  `<owl-root contenteditable="false" data-oe-protected="true">` (`web/static/src/public/interaction_service.js:76`, `InteractionService.prepareRoot`)
  so the editor leaves them alone.

## Quick checklist before finishing

1. `selector` defined (and `static` in v19)?
2. Declared in `web.assets_frontend`?
3. Registry name namespaced with the module?
4. Async data in `willStart`, not `start`?
5. Every side effect cleaned up (destroy / registerCleanup / auto-clean helper)?
6. Edit-mode behavior decided (`disabledInEditableMode` v18 / `.edit.js` v19)?
7. `rpc` via `import { rpc }` (not `this._rpc`)?
