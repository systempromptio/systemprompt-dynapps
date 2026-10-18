# STEP 3.5 — Studio governance check (collision radar)

> **Optional** capability, activated when there is a **live client environment** (the Odoo MCP is
> already connected from STEP 0). It is the counterweight the ecosystem acknowledges is missing:
> there is a quality pipeline for modules, but **no auditor for Studio** (see the
> `ai-impact-studio-vs-custom.md` reference of the **odoo-classification-guide** skill). This step
> covers it.

It runs **between STEP 3 (classification) and the verdict**, in two situations:

- **A new request** leaning towards Studio → confirm it does not step on anything installed.
- **Staging→production promotion**: a change already prototyped in Studio by the consultant and
  validated by the client, heading for production. Here the consultant's NOTE is the main input.

If there is NO environment access (greenfield, description only), skip this step and decide with
the general STEP 3 criterion.

## What it does

1. **Classify installed modules by AUTHOR, not by name.**
   `ir.module.module` (state=installed) with `name, author`. Standard = author `Odoo S.A.`;
   **non-standard** = Indaws (`indaws_*`), **OCA**, client custom, and `studio_customization`
   (the Studio work already done, author = the client). Filtering by `indaws_*` would miss OCA
   and custom.

2. **Map which FIELDS each non-standard module touches** (not just the model): `ir.model.data`
   (`module in <non-standard>`, `model='ir.model.fields'`) → resolve with `ir.model.fields.read`
   → index `model → {field → modules}`. Group fields by **zone** (price/discount, dates, states,
   notes, tax…).

3. **Identify the ZONE the request lands in** (from the description or note: "a discount" → the
   price/discount zone on `sale.order.line` / `account.move.line`).

4. **Detect collisions by ZONE, not by whole model.** There is a collision only if the request
   falls in the same field or area already managed by a non-standard module or
   `studio_customization`. (In a customized client almost the whole model is touched; reasoning
   per model would always answer "module".)

5. **Promotion mode** (changes already built in Studio): to isolate what is NEW from the old
   Studio work, **diff `studio_customization` staging vs PRODUCTION** (what is in staging and not
   in prod = what is new). The skill proposes the short list and **the consultant confirms it**
   (human in the loop).

## How it feeds the verdict (STEP 3)

- **Collision with an existing non-standard module / Studio** → strong signal for a **module**
  (governed): extend the module that already owns that zone (prefer `indaws_*` Core if reusable)
  rather than stacking ungoverned Studio on top.
- **Free zone + trivial change** → Studio is still valid.
- **Genuine doubt afterwards** (unclear whether there is logic) → a developer decides, not the
  skill.

## Technical recipes (Odoo MCP; `odoo-pilot` / XML-RPC as fallback)

```
ir.module.module.search_read([('state','=','installed')], ['name','author'])
ir.model.data.search_read([('module','in',nonstd),('model','=','ir.model.fields')],['module','res_id'])
ir.model.fields.read(res_ids, ['model','name'])     # -> model -> {field -> modules}
# promotion: repeat against production and diff studio_customization
```

- **`indaws_security` blocks `authenticate()`** (returns False): discover the UID by trying the
  API key with `execute_kw` against candidate UIDs. Odoo 19 = JSON2, but `/xmlrpc/2/` still works.
- **Read only**: `search` / `read` / `search_read` / `fields_get`. Never `create` / `write` /
  `unlink`.

## If the verdict is a module

- Create the task via **`dev-task-writer`** (correct project + technical data + which piece to
  extend).
- **New, clean fields** in the module (do not reuse `x_studio_*`).
- **Staging data is a test, it is not migrated.** In production that field never existed → clean
  install, no migration script.
- **A human always performs the production push** (the developer reviews, approves, and cleans up
  the Studio prototype the module replaces). Claude never deploys to production alone.
