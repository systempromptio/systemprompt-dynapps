# Input Modes

The skill supports four entry modes (A/B/C/D) plus a transversal FORM mode. Ask the user which one
at the start. Every mode converges on the **same publish step**: build the QWeb arch
(`t-call="website.layout"` + `#wrap.oe_structure` + `s_*` sections) → upload assets → create
`ir.ui.view` + `website.page` **unpublished** → return the page URL.

```
                              ┌─────────────────┐
                              │  USER REQUEST   │
                              └────────┬────────┘
                                       │
          ┌────────────┬───────────────┼───────────────┬────────────┐
          ▼            ▼               ▼               ▼            ▼
      ┌───────┐    ┌───────┐       ┌────────┐      ┌─────────┐  ┌────────┐
      │  (A)  │    │  (B)  │       │  (C)   │      │  (D)    │  │ +FORM  │
      │ BRIEF │    │ HTML  │       │ IMAGE  │      │ DUP.URL │  │(transv)│
      └───┬───┘    └───┬───┘       └───┬────┘      └────┬────┘  └───┬────┘
          │            │               │                │           │
          ▼            │               ▼                │           │
   ┌──────────────┐    │       ┌───────────────┐        │           │
   │ ask user →   │    │       │ analyze image │        │           │
   │ odoo-frontend│    │       │ → section map │        │           │
   │ generates    │    │       │ → odoo-frontend        │           │
   │ HTML         │    │       │   generates   │        │           │
   └──────┬───────┘    │       └───────┬───────┘        │           │
          │            │               │                │  build s_website_form
          ▼            ▼               ▼                │  block, inject into
   ┌────────────────────────────────────────┐          │  new OR existing page
   │ builder-safe transform                  │          │           │
   │  (map → s_* snippets, Bootstrap grid,    │          │           │
   │   strip <script>/on*/fixed, extract CSS) │          │           │
   │ + upload images → ir.attachment          │          │           │
   │ + extract CSS → injection ladder         │          │           │
   └───────────────────┬──────────────────────┘          │           │
                       │                                  │           │
                       ▼                                  ▼           ▼
            ┌──────────────────────────────────────────────────────────┐
            │  PUBLISH: ir.ui.view + website.page (is_published=false)   │
            │  optional website.menu · verify · return edit URL          │
            └──────────────────────────────────────────────────────────┘
```

## Mode A — Brief

The user describes the page but has no HTML. The skill asks, then **delegates HTML
generation to the `odoo-frontend` skill**.

### Questions to ask

```
1. What is the goal of the page? (landing, services, "about us", contact, event…)
2. Which sections do you want and in what order? (hero, features, pricing, testimonials, FAQ, CTA, footer…)
3. What is the main call to action? (button text + destination URL or anchor)
4. Do you want a form that generates leads in the CRM? (yes/no → see FORM mode)
5. Brand? (generic by default; say "Dynapps" to apply the Dynapps brand via dynapps-brand)
6. Do you have local images you want to use? (path to the folder or "no")
7. Desired page URL? (e.g. /servicios) and a menu entry? (yes/no)
8. In what language(s) should the page content be? (by default, the site's primary language; if several → they are translated onto the same page, see `multilanguage-output.md`)
```

### Generation strategy
Delegate to `odoo-frontend` so the markup uses Odoo Editor methodology (Bootstrap 5 + recognised
snippet classes, and Dynapps tokens if requested). Do NOT generate raw web HTML here and convert —
ask `odoo-frontend` for builder-compatible section markup. After generation, flow to the
builder-safe transform (same as Mode B).

## Mode B — Local HTML / pasted design

The user brings a designed HTML file (Claude Design export, hand-coded, etc.) by path or pasted.

### Questions to ask

```
1. Path to the .html file (or paste the HTML here)?
2. Page title? (if the HTML has no <title>)
3. Desired page URL? (e.g. /mi-landing)
4. Does it have local images (assets/ folder or similar)? (yes/no → which folder?)
5. Menu entry? (yes/no)
```

### Validation
Read the file and sanity-check:
- Has at least one block element (`<section>`, `<div>`, `<h1>`…).
- Reasonable size (< ~500 KB; if larger, likely embedded data-URI images to extract).
- Image refs are absolute URLs OR local paths that resolve in the assets folder.

If validation fails, report and ask the user to fix the source.

### Flow
`build_page_arch.py` does the heavy lifting (see `builder-safe-transformations.md`): strip
`<!DOCTYPE>`/`<html>`/`<head>`/`<body>`, wrap in `t-call="website.layout"` + `#wrap.oe_structure`,
map each block to the nearest `s_*` section with `data-snippet`/`data-name`, convert layout to
Bootstrap 5 grid, strip `<script>`/`on*`/loose `<iframe>`/`position:fixed|sticky`, and route CSS
to the injection ladder. Then `upload_assets.py` uploads images to `ir.attachment` and rewrites
`src` to `/web/image/<id>`. Then publish.

## Mode C — Image reference

The user brings a screenshot or mockup.

### Questions to ask

```
1. Path to the reference image?
2. Do you want it to reproduce the design faithfully or adapt it to Odoo's methodology? (faithful/adapt)
3. Brand? (generic by default; "Dynapps" to apply dynapps-brand)
4. Desired page URL? and a menu entry? (yes/no)
5. In what language(s) should the page content be? (by default, the site's primary language; see `multilanguage-output.md`)
```

### Flow
Analyse the image visually and produce a **section map** (hero, features, CTA, testimonials,
pricing, footer…). Show it to the user for confirmation. Then generate the HTML for each section
via `odoo-frontend` (Dynapps branding if requested), and converge on the builder-safe transform +
upload + publish, exactly like Mode B. For heavy creative/visual analysis, escalate to an opus
subagent.

## Mode D — Duplicate existing page

The user gives a URL or id of an existing Odoo page.

### Questions to ask

```
1. URL or ID of the Odoo page to duplicate?
2. New URL for the copy? (e.g. /mi-landing-v2)
3. New name/title?
```

### Extract the id, then copy
`extract_page_id.py "<url>"` parses ids from forms like `/odoo/website/<id>`,
`...id=<id>&model=website.page...`, or a bare integer. Then duplicate:

```bash
"$ODOO_PILOT_PATH"/scripts/execute_method.sh website.page copy '[<id>]' '{}'   # → new page id
```

`copy()` on `website.page` duplicates the linked `ir.ui.view` arch too. The new page is created in
draft/unpublished state. Update its `name`/`url` afterwards. No builder-safe transform is needed —
the source was already a valid Odoo page. Return the new edit URL.

> If `website.page.copy()` does not also clone the view on the target instance, fall back to
> copying `ir.ui.view` explicitly and re-linking via `view_id`. **Verify on the target instance.**

## FORM mode (transversal)

Add an `s_website_form` → `crm.lead` block to a **new** or **existing** page. Combinable with any
of A/B/C/D.

### Questions to ask

```
1. Does the form go on a new page or an existing one? (new → URL? / existing → URL or ID?)
2. Which fields do you want to capture? (name, email, phone, company, message, others…)
3. What happens on submit? (show a confirmation message / redirect to a page, e.g. /contactus-thank-you)
```

### Prerequisites and flow
- Verify `website_crm` is installed (it whitelists `crm.lead` as a web-form model and pre-allows
  `contact_name`, `email_from`, `phone`, `partner_name`, `description`). If missing, offer
  `install_module.sh website_crm`. See `website-forms-to-leads.md`.
- Non-standard fields must be whitelisted (`website_form_access=True` on `ir.model`,
  `website_form_label` on the field). **Verify on the target instance.**
- Build the block from `templates/lead_form_block.html` (`build_lead_form.py`): `<form
  action="/website/form/" ... data-model_name="crm.lead" class="s_website_form …">` with
  `s_website_form_field` inputs and an `s_website_form_send` submit. Insert it into the page arch
  (new page) or append it to the target page's `#wrap.oe_structure` (existing page). The
  `/website/form/` controller materialises the `crm.lead` at submit time.

## Batch handling

When the user wants N pages at once ("create 5 landings from these 5 designs"), ask:

> Should I create them in parallel (faster, N agents) or in series (more controlled)?

Parallel: emit N `Agent()` calls in a single message, each running this skill on one page.
Serial: loop, one page at a time. Every page still lands unpublished and returns its URL.

## Mode-specific edge cases

- **A — no brand given:** stay brand-agnostic (Bootstrap defaults / theme palette). The user can
  restyle later in the Website Builder, or re-run with `dynapps-brand`.
- **B — inline `data:image/...;base64` images:** extract each to `ir.attachment` and rewrite the
  `src` to `/web/image/<id>`; otherwise the arch becomes hundreds of KB.
- **B/C — embedded `<style>`:** editable content strips `<style>`. Route CSS through the injection
  ladder (`css-injection-ladder.md`), never inline in `#wrap`.
- **D — URL or `key` collision:** the new `url` and the view `key` must be unique. If they exist,
  offer a suffix or controlled overwrite (see GOTCHAS).
- **FORM — multi-website:** create/insert the form on the correct `website_id`; a null `website_id`
  makes the page shared across all sites.
