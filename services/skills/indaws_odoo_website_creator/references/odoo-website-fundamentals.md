# Odoo Website Fundamentals

How a website page is stored in Odoo and how to materialize one via API so it is **rendered,
responsive, and editable** in the Website Builder. Applies to v18 and v19 (shared model).

## The two records behind every page

| Record | Role |
|--------|------|
| `ir.ui.view` (`type='qweb'`) | Holds the **content** — the QWeb `arch` (the HTML/snippets). Has a unique `key`. |
| `website.page` | The **page** itself — `url`, publication, SEO. Points to the view via `view_id`. |

`website.page` uses *delegation inheritance* (`_inherits` → `view_id`): many view fields are
reachable through the page record. Despite that, the **robust create order is: view first, then
page** — create the `ir.ui.view`, then the `website.page` linking it. (The convenience method
`website.new_page()` / `_create_page()` exists but its signature varies by version; prefer the
explicit two-record path, and fall back to the method only if needed — verify on the instance.)

## The non-negotiable arch structure

```xml
<t t-name="website.page_mi_landing" name="Mi Landing">
    <t t-call="website.layout">
        <div id="wrap" class="oe_structure oe_empty">
            <!-- snippet sections here -->
        </div>
    </t>
</t>
```

- **`t-call="website.layout"`** injects the active theme's header + footer. Without it you get a
  bare fragment with no site chrome.
- **`<div id="wrap" class="oe_structure">`** is the editable drop zone. The Website Builder only
  lets users add/move/edit blocks **inside an `oe_structure`**. Verificado 2026-08-07: es literalmente
  la plantilla de página vacía que trae Odoo —`<div id="wrap" class="oe_structure oe_empty"/>` en
  `website/views/website_templates.xml:2408-2412`, `default_page` (v18) y `website/views/website_templates.xml:2781` (v19)— y el guardado por-estructura
  tiene su propio hook, `_save_oe_structure_hook` (`website/models/ir_ui_view.py:468`, `View._save_oe_structure_hook`, v18; `website/models/ir_ui_view.py:471`
  v19). Omit it and the page renders but
  is frozen in the editor — the single most common "why can't I edit my page" bug.
- The `t-name` of the root `<t>` must equal the view `key`.

## Snippet sections inside `#wrap`

Each block is a `<section>` with a snippet class, `data-snippet`, `data-name`, and Bootstrap grid:

```xml
<section class="s_text_block pt40 pb40" data-snippet="s_text_block" data-name="Texto">
    <div class="container">
        <div class="row">
            <div class="col-lg-6">…</div>
            <div class="col-lg-6">…</div>
        </div>
    </div>
</section>
```

`data-snippet`/`data-name` make the block individually selectable/movable/deletable in the
builder. See `snippet-catalog.md` for the class map and `builder-safe-transformations.md` for the
rules that keep the arch editor-safe.

## Creating a page via API (canonical path)

Use `scripts/create_website_page.py`, which performs (via odoo-pilot or MCP):

1. Resolve the target `website` id (`search_records.sh website '[]' '["id","name"]'`).
   If the instance has one website, that's it; if several, ask which (multi-website → §below).
2. **Check collisions**: no existing `website.page` with the same `url`, no `ir.ui.view` with the
   same `key`. If found, offer a suffix or a controlled overwrite — never clobber silently.
3. Create the view:
   `create_record.sh ir.ui.view '{"name":"…","type":"qweb","key":"website.page_…","arch":"<t …>"}'`
4. Create the page:
   `create_record.sh website.page '{"name":"…","url":"/…","view_id":<view_id>,"website_id":<id>,
   "is_published":false,"website_meta_title":"…","website_meta_description":"…"}'`
5. Return the **edit URL** (`/odoo/action-website.…` or the backend form) and the **preview URL**
   (`<ODOO_URL>/<url>`). The page is UNPUBLISHED.

> Field name for the arch can be `arch`, `arch_base`, or `arch_db` depending on context; `arch`
> is the safe write target. If a write is rejected, fall back to `arch_db`. Verify on instance.

## Editing an existing page

Write a new `arch` to the page's `ir.ui.view` (`update_record.sh ir.ui.view '[<view_id>]'
'{"arch":"…"}'`). To append a block (e.g. a form), read the current arch, insert the section
before `</div>` of `#wrap`, write it back.

## Multi-website

`website_id` decides which site the page belongs to. `null`/unset = shared across all websites.
In multi-website setups, always set `website_id` explicitly to avoid a page leaking to every site.
See `publish-and-menu.md`.

## Publishing

Pages are created `is_published=false`. Going live is an explicit user action:
`update_record.sh website.page '[<id>]' '{"is_published":true}'`. This skill never publishes
on its own.
