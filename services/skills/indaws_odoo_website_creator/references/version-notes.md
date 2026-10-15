# Odoo Version-Specific Notes

This skill's pre-loaded knowledge targets **Odoo v18 and v19**. The website **page model** is the
same in both, and that is what this skill writes. What v18 and v19 **share** (stable, the core of
this skill):

- The `website.page` + `ir.ui.view` (`type='qweb'`, unique `key`) pairing.
- The editable wrapper requirement: `<t t-call="website.layout">` containing
  `<div id="wrap" class="oe_structure">…</div>`. Without it the Website Builder does not treat the
  page as editable.
- Snippet markup: sections carry `s_*` classes plus `data-snippet` / `data-name`.
- Bootstrap 5 grid (`container` → `row` → `col-*`) for responsiveness.
- The `s_website_form` → `crm.lead` structure (action `/website/form/`,
  `data-model_name="crm.lead"`, `s_website_form_field` / `s_website_form_input` markup).

Because the **page** layer is stable, la mayoría de los deltas v18↔v19 afectan a cosas
*periféricas* (theme SCSS, JS interactivo) mayormente fuera del alcance "sin módulo" de esta skill.
**Con una excepción que sí duele**: el módulo `web_editor` desaparece en v19 y con él el nombre del
modelo que usa el panel THEME (ver la tabla de deltas). Eso no es periférico: es una llamada de esta
skill que falla.

## Auto-detect version

Always detect before pushing. Do NOT ask the user the version.

```bash
"$ODOO_PILOT_PATH"/scripts/detect_version.sh      # reports version + protocol (no auth needed)
```

Or read `server_version` directly (no auth):

```bash
curl -sX POST "${ODOO_URL}/jsonrpc" -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"call","params":{"service":"common","method":"version","args":[]},"id":1}' \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["result"]["server_serie"])'
```

Returns `19.0`, `18.0`, `17.0`, … `odoo-pilot` auto-selects JSON-RPC (≤18) vs JSON2 (≥19); the
skill does not handle the transport.

## Decision tree

```
Detected version 18.0 or 19.0?
  → Use the pre-loaded flow. The page model (website.page + ir.ui.view +
    #wrap.oe_structure + s_* snippets + Bootstrap 5 grid + s_website_form) applies as-is.
  → Pull ONLY the relevant v18↔v19 delta from odoo-version-api-reference at runtime
    (do NOT hardcode it here). See "Deltas that matter" below.
  → Proceed.

Detected version 15.0 – 17.0?
  → Consult `odoo-version-api-reference` FIRST — it ships v17/v16/v15 diff files
    (references/diffs/v17-diff.md, v16-diff.md, v15-diff.md; v18 is its canonical base).
    Pull the relevant website/web_editor delta from there.
  → Web search only to fill gaps those diffs don't cover.
  → Verify the four invariants (see "When to refuse"). Proceed only if confirmed.

Detected version >= 20.0 (future)?
  → Web search:
      "odoo 20 website page model website.layout oe_structure changes"
      "odoo 20 s_website_form snippet structure"
  → Same verification. Proceed only if confirmed.
```

## Deltas that matter (consult `odoo-version-api-reference` at runtime)

Invoke `odoo-version-api-reference` (its `references/diffs/v19-diff.md`) as a subagent and keep
ONLY the website/web_editor delta relevant to the current task. Summary of which deltas apply and
when:

| Delta (v18 → v19) | Relevant to this skill? |
|---|---|
| **Grid mode** `o_grid` / `o_grid_item` (free-positioned columns, new in v19) | OPTIONAL. Only if the user wants drag-positioned columns. Plain `s_*` sections + Bootstrap grid work in both versions — prefer them for portability. |
| **Theme SCSS**: `$o-theme-` prefix mandatory; palettes must include `menu`/`footer`/`copyright` keys; units in `rem` not `px`; font configs need a `properties` section | RELEVANT ONLY when injecting **theme** SCSS via `ir.asset`. Plain page-scoped CSS via the injection ladder is unaffected. Most pages need neither. |
| **JS frontend**: RPC service removed from frontend (use `fetch`); `publicWidget` → `Interaction` (Colibri); `registry.category("public_components")` → `"public.interactions")`; `odoo.define()` removed | RELEVANT ONLY for **interactive JS components**, which require a module — **out of scope** (N3). Detect and refer to the dev suite; do not improvise interactive JS by API. |
| **El módulo `web_editor` DESAPARECE en v19** (verificado 2026-08-07: no existe `addons/web_editor`; en v18 coexisten `web_editor` y `html_editor`) | **RELEVANTE Y NO PERIFÉRICO.** Dos consecuencias con las que esta skill se choca de frente: (1) el modelo del panel THEME pasa de `web_editor.assets` (v18) a **`website.assets`** (v19) → la llamada de `theme-customization.md` **falla** si usas el nombre equivocado; (2) la lógica de guardado del `arch` se muda de `web_editor/models/ir_ui_view.py:318-320`, `IrUiView.save` (v18) a `html_editor/models/ir_ui_view.py:355-357`, `IrUiView.save` (v19). En v18 `html_editor` ya existe pero **solo trae `ir_attachment.py`** en `models/`, así que buscar ahí la lógica en v18 no encuentra nada y parece que el mecanismo no existe. |

The skill must NOT copy `v19-diff.md` content into its own references. It loads the relevant slice
at runtime so the single source of truth stays in `odoo-version-api-reference`.

## When to refuse (versions outside v18/v19, or anything unverified)

After a web search, if you CANNOT confirm all four invariants, refuse and report findings — do
NOT push and hope:

1. The `website.page` model exists (with `url`, `view_id`, `website_id`, `is_published`).
2. The editable wrapper `#wrap.oe_structure` inside `t-call="website.layout"` is still the
   convention for editable pages.
3. The `s_website_form` snippet still posts to `/website/form/` and supports `crm.lead` (with
   `website_crm` installed).
4. Snippet sections (`s_*` + `data-snippet`/`data-name`) are still recognised by the builder.

Also refuse interactive-JS components and palette snippets in any version — they need a module
(in v19, the `Interaction`/Colibri framework). Detect the case, explain it, and hand off to the
development suite instead of attempting it via API.
