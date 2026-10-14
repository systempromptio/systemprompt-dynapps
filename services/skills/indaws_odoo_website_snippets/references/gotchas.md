# Snippet GOTCHAS (with path:line)

Shared standards: `references/shared/coding-standards.md`
Paths are `$ODOO_SRC/odoo18/addons/` unless noted (v19 = `$ODOO_SRC/odoo19/addons/`).

1. **Options panel doesn't appear** — The options `<div data-js=... data-selector=...>`
   must be inside a template with `inherit_id="website.snippet_options"`. The
   `optionName` maps to a JS class via `options.registry[optionName] || options.Class`
   (`web_editor/static/src/js/editor/snippets.editor.js:829`, `_initializeOptions`). If neither matches the
   DOM, nothing mounts.

2. **`data-selector` matches nothing → silent no-op** — The selector is applied against
   the snippet `$target`. It must hit the root class `.s_<name>`. A typo gives no error;
   the panel just never shows. Cross-check the template root class in `s_<name>.xml`.

3. **Snippet not in the menu** — Defining the `s_*` template is not enough. You must add
   `<t t-snippet="module.s_x" .../>` inside `website.snippets` (xpath to `snippet_structure`
   `website/views/snippets/snippets.xml:42`, `snippet_structure`, `snippet_structure` or `snippet_content` `:370`). The menu is computed from those
   `<snippets>` nodes (`web_editor/static/src/js/editor/snippets.editor.js:3342-3366`, `_computeSnippetTemplates`).

4. **Don't hardcode `data-snippet` on a static snippet** — The editor sets it on drop from
   `snippetEl.children[0].dataset.snippet` (`web_editor/static/src/js/editor/snippets.editor.js:3352`, `_computeSnippetTemplates`), derived from the
   template `id`. Hardcoding can desync the name. Exception: dynamic snippets set
   `t-att-data-snippet="snippet_name"` deliberately (`website/views/snippets/s_dynamic_snippet.xml:4`, `website.s_dynamic_snippet_template`).

5. **Asset in the wrong bundle** — Options JS belongs to `website.assets_wysiwyg`
   (`website/__manifest__.py:310`, options listed `:321`,`:323`); it must NOT be in
   `web.assets_frontend` (`:193`). SCSS / public widgets visible on the published site go
   to `web.assets_frontend`. `<asset>` declared inline, e.g. `website/views/snippets/s_alert.xml:42-45`, `website.s_alert_000_scss`.

6. **Editor sanitization strips custom attributes/classes** — On save the editor cleans
   editable HTML; non-standard attrs in editable zones can vanish. Protect content with
   `o_not_editable` / `oe_unremovable` / `oe_unmovable` (`website/views/snippets/s_dynamic_snippet.xml:7`, `website.s_dynamic_snippet_template`, y `website/views/snippets/s_dynamic_snippet.xml:16`) and
   only persist state via recognized `data-*` attributes handled by the option.

7. **`onBuilt` vs `start`** — Logic that must run **when the block is dropped** (insert an
   iframe, init markup) goes in `onBuilt()` (`website/static/src/snippets/s_map/options.js:11-21`, `onBuilt`; base
   `web_editor/static/src/js/editor/snippets.options.js:3445`, `onBuilt`), not in `start()`.

8. **`data-select-class` is mutually exclusive in one widget** — Several `we-button` with
   `data-select-class` inside one `we-select`/`we-button-group` toggle each other off
   (`website/views/snippets/s_alert.xml:29-33`, `s_alert_options`). For independent, cumulative classes use separate widgets.

9. **Missing `data-no-preview="true"`** — Hovering an option triggers a live preview;
   for dynamic snippets that re-fetch on change this breaks rendering. Dynamic options set
   it everywhere (`website/views/snippets/s_dynamic_snippet.xml:36`, `website.s_dynamic_snippet_options_template`, y `website/views/snippets/s_dynamic_snippet.xml:46`).

10. **`data-js="Foo"` with no registry class** — Falls back to generic `options.Class`;
    your custom methods (`selectFoo`, `onBuilt`) never fire. Define `options.registry.Foo`
    or remove the `data-js`.

11. **Broken thumbnail** — `t-thumbnail` must point to a servable asset
    (`/module/static/.../x.svg`); the src is read at `web_editor/static/src/js/editor/snippets.editor.js:3356`, `_computeSnippetTemplates`
    (`dataset.oeThumbnail`). Missing file → block shows with no image in the menu.

12. **v19 rewrote the editor (`html_builder`)** — No `options.registry`, no `we-*`, no
    `inherit_id="website.snippet_options"`. Options are OWL `extends BaseOptionComponent`
    with `static selector`/`static template` (`website/static/src/builder/plugins/options/
    process_steps_option.js:10-13`), registered via a `Plugin` whose
    `resources.builder_options` lists them, added to
    `registry.category("builder-plugins")` (`odoo19/addons/website/static/src/builder/plugins/options/process_steps_option_plugin.js:21-26`, `ProcessStepsOptionPlugin`). Don't mix systems.

13. **Wrong drop container** — `drop-in`/`drop-near` (v17/v18 via `data-drop-in`/
    `data-drop-near`, parsed `web_editor/static/src/js/editor/snippets.editor.js:815-816`, `_initializeOptions`; v19 via `dropzone_selector` in
    the Plugin `resources`, `odoo19/addons/website/static/src/builder/plugins/options/process_steps_option_plugin.js:42-45`, `ProcessStepsOptionPlugin`) control where the
    block can be placed. Omitting them can make a structural block droppable in invalid
    spots or not at all.

14. **`type='json'` for snippet data routes** — Dynamic snippet data routes are
    `type='json', auth='public', website=True, readonly=True`
    (`website/controllers/main.py:416`, `Website.get_dynamic_filter`, y `website/controllers/main.py:441`, `Website.get_dynamic_snippet_templates`), not `type='http'`. The public widget fetches
    them via RPC, not page render.
