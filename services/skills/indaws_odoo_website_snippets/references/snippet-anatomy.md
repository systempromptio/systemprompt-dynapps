# Snippet Anatomy & Registration (v17/v18; v19 notes)

Shared standards: `references/shared/coding-standards.md`

A snippet is 3 things: (1) a QWeb **template** `s_*`, (2) its **registration** in the
snippets menu, (3) optional **options panel** + **assets**. This file covers 1, 2, assets.
Source: `$ODOO_SRC/odoo18/addons/website/views/snippets/`.

## 1. The template `s_<name>.xml`

One file per snippet, dropped in `views/snippets/`. Verified shape (`website/views/snippets/s_alert.xml:1-11`):

```xml
<?xml version="1.0" encoding="utf-8"?>
<odoo>
  <template id="s_alert" name="Alert">
    <div class="s_alert s_alert_md alert alert-info w-100 clearfix">
      <i class="fa fa-2x fa-info-circle s_alert_icon"/>
      <div class="s_alert_content">
        <p>Explain the benefits you offer.</p>
      </div>
    </div>
  </template>
  <!-- options template + <asset> go in the SAME file (see below) -->
</odoo>
```

Rules:
- The **root class** must be `s_<name>` matching the template `id`. The options panel
  selector and the public widget target this class.
- The template `id` (`s_alert`) becomes the snippet `name`. **Do NOT add `data-snippet`**
  on a static snippet's markup — the editor sets it on drop from
  `snippetEl.children[0].dataset.snippet` (`web_editor/static/src/js/editor/snippets.editor.js:3352`, `_computeSnippetTemplates`). Dynamic snippets
  are the exception (they set `t-att-data-snippet="snippet_name"`, `website/views/snippets/s_dynamic_snippet.xml:4`, `website.s_dynamic_snippet_template`).
- Use only Bootstrap 5 + Odoo editor-safe markup for the visual content (delegate design
  to `odoo-frontend`).

## 2. Registration in the snippets menu (`snippets.xml`)

The menu is built by inheriting `website.snippets`. Two target containers:

| | v18 | v19 |
|---|---|---|
| `snippet_structure` — full-width structural blocks (`<section>`) | `website/views/snippets/snippets.xml:42`, `snippet_structure` | `:41` |
| `snippet_content` — inner content blocks (button, image, alert…) | `:370` | **`:476`** |

> ⚠️ **`website.snippets` NO hereda de `web_editor.snippets` en v19** — esta línea lo decía hasta
> 2026-08-07. El módulo `web_editor` **no existe en v19**: en v18 la plantilla raíz era
> `web_editor/views/snippets.xml:3`, `snippets` y `website.snippets` la extendía; en v19 `website.snippets` es
> **plantilla raíz** (`website/views/snippets/snippets.xml:5`, `snippets`, sin `inherit_id`). Verificado
> 2026-08-07. Si heredas apuntando a `web_editor.snippets` en un módulo v19, el XML no resuelve la
> referencia y el módulo no instala.

### Two registration styles

**A. With a group (uses the group's auto-thumbnail flow), verified `website/views/snippets/snippets.xml:45-47`, `snippet_structure`:**
```xml
<t t-snippet="website.s_banner" string="Banner" group="intro">
  <keywords>hero, jumbotron, headline, header, branding, intro, home</keywords>
</t>
```

**B. With an explicit thumbnail (inner content), verified `website/views/snippets/snippets.xml:376`, `snippet_content`:**
```xml
<t t-snippet="website.s_alert" string="Alert"
   t-thumbnail="/website/static/src/img/snippets_thumbs/s_alert.svg"/>
```

To add YOUR snippet from your module, inherit and xpath into the right container:
```xml
<template id="my_snippets" inherit_id="website.snippets">
  <xpath expr="//snippets[@id='snippet_content']" position="inside">
    <t t-snippet="my_module.s_my_block" string="My Block"
       t-thumbnail="/my_module/static/src/img/snippets_thumbs/s_my_block.svg">
      <keywords>custom, my, block, special</keywords>
    </t>
  </xpath>
</template>
```

Fields:
- `t-snippet` — `module.template_id` of your `s_*` template. Required.
- `string` — display name in the menu.
- `group` — assigns it to a `snippet_groups` category (`intro`/`columns`/`content`/
  `images`/`people`/`text`/`contact_and_forms`/`social`...). Defined in `website/views/snippets/snippets.xml:11-31`, `default_snippets`.
- `t-thumbnail` — explicit preview image (SVG/PNG under your module's `static`). Read at
  `web_editor/static/src/js/editor/snippets.editor.js:3356`, `_computeSnippetTemplates` (`dataset.oeThumbnail`).
- `<keywords>` — comma-separated search terms for the snippet search box.

`drop-in` / `drop-near` (where the block may be dropped) are configured on the options
side via `data-drop-in`/`data-drop-near` — see `snippet-options.md`.

## 3. Assets

Declared with `<asset>` in the SAME `s_*.xml` file. Verified `website/views/snippets/s_alert.xml:42-45`, `website.s_alert_000_scss`:
```xml
<asset id="website.s_alert_000_scss" name="Alert 000 SCSS">
  <bundle>web.assets_frontend</bundle>
  <path>website/static/src/snippets/s_alert/000.scss</path>
</asset>
```

Bundle rules (verified `website/__manifest__.py`):
- **SCSS / public widget JS** (visible on the published site) → `web.assets_frontend`
  (`website/__manifest__.py:193`).
- **Snippet options JS** (editor-only) → `website.assets_wysiwyg` (`website/__manifest__.py:310`,
  options listed `:321`, `:323`). NEVER ship options JS to `web.assets_frontend`.

Convention: per-snippet folder `static/src/snippets/s_<name>/` with `000.scss`, `000.js`
(public widget), `options.js` (editor options), `000.xml` (OWL/QWeb if needed).

## v19 differences (editor rewritten — `html_builder`)

- Templates `s_*` still live in `views/snippets/` and registration still uses
  `t-snippet` in `snippets.xml`. The template/markup layer is largely unchanged.
- The **options** layer is completely different (OWL components + Plugin). See
  `snippet-options.md`. Per-snippet folders moved to
  `website/static/src/builder/plugins/options/`.
- Verify the exact bundle name for builder assets against `website/__manifest__.py` of
  the target v19 build before declaring `<asset>`.
