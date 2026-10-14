# Dynamic Snippets (`s_dynamic_snippet*`)

Shared standards: `references/shared/coding-standards.md`

A dynamic snippet renders a **list of records** fetched from a `website.snippet.filter`
through a sub-template, instead of static editor content. Source:
`$ODOO_SRC/odoo18/addons/website/.../s_dynamic_snippet/` and `s_dynamic_snippet.xml`.

## Template structure

Dynamic snippets call a shared base template. Verified `website/views/snippets/s_dynamic_snippet.xml:3-34`, `website.s_dynamic_snippet_template`:

```xml
<template id="website.s_dynamic_snippet_template">
  <section t-attf-class="#{snippet_name} #{snippet_classes} s_dynamic s_dynamic_empty pt32 pb32"
           t-att-data-snippet="snippet_name"
           t-att-data-custom-template-data="custom_template_data or '{}'">
    <div class="container">
      <div class="row s_nb_column_fixed">
        <section class="s_dynamic_snippet_title oe_unremovable oe_unmovable ...">...</section>
        <section class="s_dynamic_snippet_content oe_unremovable oe_unmovable o_not_editable col">
          <div class="dynamic_snippet_template"><t t-out="0"/></div>
        </section>
      </div>
    </div>
  </section>
</template>

<template id="s_dynamic_snippet" name="Dynamic Snippet">
  <t t-call="website.s_dynamic_snippet_template">
    <t t-set="snippet_name" t-value="'s_dynamic_snippet'"/>
  </t>
</template>
```

Key points:
- **Here `data-snippet` IS set in markup** (`t-att-data-snippet="snippet_name"`,
  `:4`) — the opposite rule of static snippets, because the public widget needs the name
  to look up its config.
- The content area is `o_not_editable` + `oe_unremovable` + `oe_unmovable` (`:7,16`) so
  editor sanitization doesn't strip the fetched records.
- Records are injected into `.dynamic_snippet_template` by the public widget at runtime.

## Options (the dynamic-specific panel)

Verified `website/views/snippets/s_dynamic_snippet.xml:35-46`, `website.s_dynamic_snippet_options_template` — uses `data-attribute-name` to persist config as
`data-*` attributes read by the widget, and always `data-no-preview="true"`:

```xml
<template id="website.s_dynamic_snippet_options_template">
  <div t-attf-data-js="#{snippet_name}" t-attf-data-selector="#{snippet_selector}" data-no-preview="true">
    <we-select string="Filter"   data-name="filter_opt"   data-attribute-name="filterId"/>
    <we-select string="Template" data-name="template_opt" data-attribute-name="templateKey"/>
    <we-select string="Fetched Elements" data-name="number_of_records_opt" data-attribute-name="numberOfRecords">
      <we-button t-foreach="range(1, 17)" t-as="value" t-att-data-select-data-attribute="value" t-esc="value"/>
    </we-select>
    <t t-out="0"/>
  </div>
</template>
```

The JS option class extends `options.Class` and fetches the available filters/templates
(verified `website/static/src/snippets/s_dynamic_snippet/options.js:6-372`, `dynamicSnippetOptions`): it sets `modelNameFilter`,
`contextualFilterDomain`, builds `dynamicFilters`/`dynamicFilterTemplates` and populates
the `we-select`s. Subclasses set `this.modelNameFilter` to scope the model.

Persisted `data-*` attributes: `data-filter-id`, `data-template-key`,
`data-number-of-records`, `data-custom-template-data`.

## Controller (server side)

The browser fetches rendered records via JSON routes. Verified
`website/controllers/main.py`:
- `/website/snippet/filters` (`:416`, `get_dynamic_filter(filter_id, template_key, limit,
  search_domain, with_sample, **custom_template_data)`) — returns the rendered records.
- `/website/snippet/filter_templates` (`:441`) — returns available templates.

Both are `type='json', auth='public', website=True, readonly=True`. For a custom dynamic
snippet that needs its own data source, add a route under `/website/snippet/...` (delegate
to `odoo-website-controller`) and a `website.snippet.filter` record.

## Public widget (render on the published site)

Verified `s_dynamic_snippet/000.js`:
- `import publicWidget from "@web/legacy/js/public/public_widget";` (`:3`)
- `const DynamicSnippet = publicWidget.Widget.extend({ ... })` (`:14`)
- `_fetchData()` calls `/website/snippet/filters` (`:118-122`)
- registered: `publicWidget.registry.dynamic_snippet = DynamicSnippet;` (`:293`)

This widget runs in `web.assets_frontend` (NOT the editor). Building/extending it is
`odoo-website-frontend-js` territory; this skill only wires the snippet + options +
filter to it.

## v19

Dynamic snippets exist in v19 too but the option layer is a `html_builder` Plugin
(`dynamic_snippet_carousel_option.js` in `website/static/src/builder/plugins/options/`)
and the public-side render uses an `Interaction` instead of `publicWidget`. The QWeb
template + filter/controller approach is conceptually the same. Verify exact APIs against
the target v19 build.
