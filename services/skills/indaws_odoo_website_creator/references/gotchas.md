# Odoo Website Creator — GOTCHAS (full list)

Real failure patterns. SKILL.md keeps the top ones inline; this is the complete set.

1. Without `#wrap.oe_structure` inside `website.layout`, the page is **not editable** in the builder.
   ALWAYS wrap it.
2. `<style>`/`<script>` inside editable content gets **stripped** → use the CSS ladder instead.
3. `data-oe-*` is reserved by Odoo; for custom blocks use `data-snippet`/`data-name` + `s_*` classes.
4. `website_id` matters in multi-website setups (`null` = shared).
5. Forms need `website_crm` installed, or the submit fails at runtime.
6. v19 ≠ v18 in the *editor*, not in the HTML; the deltas only bite on palette/JS snippets — and
   they should be verified per-instance (web search / `odoo-version-api-reference`;
   `references/version-notes.md`) rather than treated as always-settled for every new snippet or
   theme mechanic you encounter.
7. Palette snippets and interactive JS **require a module** (v19: `Interaction`/Colibri). Detect this
   and delegate.
8. Publishing is explicit: create with `is_published=false` and return the edit URL.
9. URL / `key` collisions: check both; offer a suffix. Never overwrite an existing page.
10. The builder **rewrites the entire `arch` on Save**: do NOT write the arch via API while the page
    is open in the builder (last-write-wins); coordinate or reconcile instead.

    Mecanismo, verificado 2026-08-07 — y **la ruta cambia de módulo entre v18 y v19**:

    | Versión | Fichero | Guarda en |
    |---|---|---|
    | 18.0 | `addons/web_editor/models/ir_ui_view.py` | `:320` (`self.write({'arch': …})`), guard en `:318`, `_are_archs_equal` en `:213` |
    | 19.0 | `addons/html_editor/models/ir_ui_view.py` | `:357`, guard en `:355`, `_are_archs_equal` en `:224` |

    Trampa: en v18 el módulo `html_editor` **ya existe** pero su `models/` solo trae
    `ir_attachment.py`, así que buscar la lógica ahí en v18 no encuentra nada y parece que no existe.

    Y un matiz a favor: el guardado **compara antes de escribir**
    (`if not self._are_archs_equal(old_arch, new_arch)`), así que un Save que no cambió nada NO pisa.
    Eso protege del ruido, **no de la concurrencia**: si el editor tiene el arch viejo cargado y tú
    escribes por API, su Save siguiente sí lo machaca y en silencio.
11. Brand styles → PREFER customizing the THEME (`references/theme-customization.md`), don't repeat
    the hex per element (not a hard rule). In a DB with an inherited theme, if a token comes out with
    a changed color, hardcode the brand hex inline (see gotcha 15 and `references/layout-recipes.md`).
12. Don't create a "thank you" page: Odoo already ships `/contactus-thank-you` → point
    `data-success-page` there.
13. Hand-built forms must use the MODERN field markup, or Odoo flags them as a legacy version
    (`references/website-forms-to-leads.md`).
14. **`indaws_security` breaks odoo-pilot's *stateless* auth** (`RuntimeError: object is not bound` in
    `check_ip_ban`) → authenticate via a **web session** (cookie, `/web/session/authenticate`) and
    then `execute_kw`. See `references/connection-modes.md`.
15. In a **restored/production** DB do NOT overwrite the theme or `custom_code_footer` blindly: the
    palette (`make_scss_customization` over `o-color-*`) recolors the **whole** site and replacing
    the footer **wipes** the client's code → read the current state, **skip the theme**, **merge** the
    footer between markers, and don't touch `homepage_url`.
16. **Newsletter ≠ website form**: it's `s_newsletter_block` + `js_subscribe` + `data-list-id` (one
    `mailing.list`), not `s_website_form`. It auto-fills the logged-in user's email (not a bug; verify
    while logged out). `data-list-id` is per-instance → remap by NAME on deploy. See
    `references/website-forms-to-leads.md`.
17. **Scroll-pinned swap is ALL-DEVICE** (pin+swap at every width, does NOT degrade to stacked). To
    make it fit on mobile you SHRINK the visual per breakpoint (`max-width` ~150→210→280), **not**
    un-pin it: any `@media` that sets `.iw-sticky-stage`/`.iw-step` to `static` **breaks** the
    responsiveness. The stage column must stretch (no `align-items-start` on the row) or the sticky
    has no travel. It works because `#wrapwrap` uses `overflow-x:clip`. See
    `templates/pattern_scroll_pinned_swap.html`.
18. **Multi-language output**: each target language must be **installed AND enabled on the website**
    (`res.lang` active + present in `website.language_ids`) before its translation appears and the
    `/<lang>/` URL resolves. Write translations onto the **same** records via `with_context(lang=…)`;
    a separate page per language fragments the URL and breaks the language switcher. See
    `references/multilanguage-output.md`.
19. **`display:none` does NOT stop downloads**: images inside hidden `.tab-pane`s / off-screen
    carousel items all download at page load unless they carry `loading="lazy"`. And `/web/image/<id>`
    serves API-uploaded binaries **as-is** (no auto-WebP outside the editor's media dialog) — a heavy
    original stays heavy; resize via `/web/image/<id>/<W>x<H>` or optimize before upload. See
    `references/performance.md` + `references/image-assets.md` §2c.
20. **Mobile overflow is invisible on Odoo**: `#wrapwrap`'s `overflow-x:clip` hides the sideways
    scrollbar, so an overflowing element gets silently **cut off** instead of screaming at you. The
    usual culprits: bare `max-width:NNNpx`/fixed `width` (→ `min(100%,Npx)`), non-breakpointed
    `min-width`/`flex-shrink-0` in a d-flex row (→ `flex-column flex-md-row`), `100vw` (includes the
    scrollbar), and `100vh` on mobile Safari (URL bar → add a `100dvh` fallback line). The build
    script lints all of these; verify at 390px by LOOKING, not scrolling. See `references/responsive.md`.
