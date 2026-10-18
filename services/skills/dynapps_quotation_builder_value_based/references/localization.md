# Localization — multi-language / multi-country (locale packs)

How the value-based engine produces proposals in **es / en / fr / nl** from one shared engine + per-country `locales/<code>/` packs. Read this when generating a non-`es` proposal, editing a pack, or onboarding a new country. Architecture + pack file ownership + governance live in `locales/README.md`; the runtime intake for a new country is `references/add_country.md`.

## Packs and composition (the detail behind SKILL.md §Locale selection)

- A **language pack is 9 JSON files** under `locales/<code>/`, validated by
  `scripts/validate_locale.py`. `scripts/locale_loader.py` is the **single access point**; the engine
  (`scripts/*.py`, `templates/*.html`, `shared/*`) is shared and **never forked** — a new
  language/country is a new pack, never an engine edit (`tests/add_country_smoke.py` covers the add
  path).
- **Two-layer composition:** an `xx-YY` code composes a **language layer** (`locales/<lang>/`:
  strings / prose / lint / matching) with a **country layer** (`locales/_countries/<CC>/`: instance /
  pricing / fiscal / currency). Plain `es`/`en`/`nl`/`fr` resolve **exactly as before**, so ES parity
  is intact (`tests/composition_smoke.py`).
- **BE/CH already compose** — `nl-BE`/`fr-BE` and `fr-CH`/`en-CH`; **CH = FR + EN** because
  dynapps.ch has no DE/IT site. Their country layers are **stubs**: `load_locale(allow_stub=False)`
  **BLOCKS client-facing output** until `verify_from_odoo` plus the fiscal/legal sign-offs are filled
  in. Plan, intake checklists and per-locale case URLs: `references/add_be_ch_plan.md`.
- **Roles:** `es` = byte-parity reference (regression oracle), `en` = engine base/default,
  `fr`/`nl` = reference packs.
- **Connector-agnostic:** each operator uses their own Dynapps MCP connector. Company entity,
  currency, the "Implantación v2" template id and the product ids are **discovered at runtime**
  (`scripts/odoo_discovery.py`), never hardcoded; **product ids are not variabilized**.
- **Per-deal/country pricing and dates** come from the workbook's `Parameters` sheet and override the
  pack for that deal only — full key list and date cascade in `references/workflow_detail.md`
  §Parameters. No sheet → no override → pack verbatim (ES byte-parity intact).
- **`sheet.*` and `params.*` are `en`-only ON PURPOSE.** The workbook is internal, so its chrome is
  English in every locale and the packs inherit it through the fallback chain;
  `INHERITED_NAMESPACES` in `validate_locale.py` exempts both from key-parity.
- **The one exception is `sheet.formacion_line`**, which renders a base feature the CLIENT reads:
  every pack declares its own. See gotcha 22 and `tests/workbook_chrome_boundary_smoke.py`
  (`CLIENT_FACING_SHEET_KEYS`) — the ES byte-parity gate does NOT cover this boundary.
- **Fallback policy:** UI strings fall back to `en`; **pricing / fiscal / product ids raise a HARD
  ERROR** — never a silent fallback to Spanish. Pack governance (per-file owners, sign-offs):
  `locales/README.md`.

## Mental model — three layers of language

A proposal's language comes from **three independent sources**. Getting a clean non-ES proposal means all three are right:

1. **Static chrome** (section titles, labels, fineprint, money/date format) — from `locales/<code>/` via `scripts/locale_loader.py`. Controlled by the active locale.
2. **Entity display names** (app-card titles, phase labels) — from `locales/<code>/apps_catalog.json` `display_name`, keyed by `app_id`.
3. **Authored prose** (`understanding`, `executive_summary_html`, `app_overrides.*`, `dev_overrides.*`) — this is **content in `extra_data.json`**, NOT localized by the engine. It is whatever language you wrote it in.

> The `--locale` flag only moves layers 1 and 2. **Layer 3 is on the author.** A "Spanish `extra_data` rendered with `--locale en`" gives you English chrome around Spanish prose.

## Selecting the locale

`resolve_locale(explicit, odoo_db, lead_country)` precedence: explicit `--locale` / `extra_data.locale` > connected Odoo DB > lead country hint > default `en`. The resolved code is logged.

- **Set `extra_data.locale`** (top-level `"es"`/`"en"`/…): `build_context` reads it, so a file renders in its own language **even without the `--locale` CLI flag**. For a multi-language deliverable, keep one `extra_<lang>.json` per language, each with its own `locale`.
- Each operator uses **their own Dynapps MCP connector → their own Odoo DB** (connector-agnostic). Don't hardcode a DB/URL. Instance specifics (company legal entity, currency, `sale.order.template` id, product ids) are **discovered at runtime** via `scripts/odoo_discovery.py`, not stored in the pack.

## Delivery-locale ≠ client's operating countries

The **delivery locale** is the language the document is written in. It is independent of which countries the *client* operates in. Davis is an `es` proposal that legitimately discusses **Portugal SAF-T** and ES–PT multi-company: that cross-border content lives in `extra_data` / the `fiscal` pack, not in the delivery locale. Don't switch locale just because the client mentions another country.

## Authored content is per-language (the #1 non-ES pitfall)

For a non-ES proposal you must author `extra_data` **in the target language**:
- `understanding`, `executive_summary_html`, `cost_of_delay`, `app_overrides.<app>.{intro,fit,features}`, `dev_overrides`.
- Keys stay stable across languages; only **values** change (see the key rules below).
- Run `brand_lint.py` in the right language: `--locale es` for Spanish, bare (default `en`) for English/others. Seed proper nouns/acronyms into the glossary `permitir` list so brand_lint doesn't false-flag them (e.g. Orion, Focus, SAF-T, Verifactu, SII, AEAT, Arxi).

## KEYS are the Spanish machine contract — never translate them

The template JS and the spreadsheet matching branch on **Spanish literals**. These MUST stay the exact Spanish names even in an en/fr/nl proposal:
- `data-app` / `data-app-name` attributes, checkbox `value`s, `req_map`, `COND_VARIANTS`, the copy-to-clipboard payload keys.
- `app_overrides.<app_id>` keys and `app_overrides.<app>.features` **keys** (they map to spreadsheet rows).
- `dev_overrides` keys (they substring-match the spreadsheet dev names).

Only the **human-visible text** is localized:
- **App-card / price-list / mindmap / PDF app titles** → `L.app(app_id).display_name` from `apps_catalog.json`. For `es`, `display_name` == the Spanish name, so ES output stays byte-identical.
- **Dev titles** → `dev_overrides[key]` supports `{ "title": "...", "description": "..." }`; the engine shows `title` as the visible heading while the key stays the Spanish match name. The old `{key: "desc-string"}` form still works (covered by `test_dev_override_render`).

### Feature-bullet titles — localized object form (gap closed in v5)
`app_overrides.<app>.features` values accept `{ "title": "<localized bullet name>", "expl": "<description>" }` alongside the legacy plain string. The engine (`_feature_display`) renders `title` as the visible bullet name (`span.feat-name`) while the key stays the Spanish match/spreadsheet name. The object form is **mandatory for non-es proposals** — with the legacy string form the bullet name still reads the Spanish key (the old pre-v5 behavior, kept for back-compat). brand_lint's IDIOMA check flags leftovers.

## Per-user license, per-locale money (extends Gotcha 12)

**Licence prices per pack.** `es` and `en` (EUR) carry the Spanish list price verified on odoo.com
(2026-09-28): Custom **334.80 €/user·year** (reference) and Light User **4.90 €/user/month**
(`pricing.recurring_monthly.odoo_light_user`). `fr`/`nl` keep `-1` on both (unverified: the
licence card needs the sheet or `costes_recurrentes`, the Light line is omitted with a warning).
BE/CH layers are untouched. **Never verify a market from Spain**: odoo.com sets the pricelist by
geo-IP, so every locale URL returns the ES prices (Gotcha 12c).

Odoo license is **per user**. Set `extra_data.num_usuarios` (drives both the PDF `num_usuarios_inicial` and the web users selector default — otherwise they can disagree) and override the line via `extra_data.costes_recurrentes` with `amount` = the **per-user** figure (so `data-rate-user` stays per-user and the live selector recomputes `n × rate`). Format per locale: `5.376,00 € / año` (es) vs `€5,376 / year` (en).

- **The recurrent block does NOT render `costes_recurrentes[].desc`.** It builds its own subtitle (e.g. "334,80 € / usuario · año"). Any figure that must appear (e.g. a monthly-equivalent total) has to be in a **rendered field**, not buried in `desc`.
- **`_parse_coste` checks month markers before annual.** An annual price whose text contains "/ month" (e.g. an EN monthly-equivalent in the same string) is mis-detected as monthly. Keep the license price in **clean annual form** and put the monthly equivalent elsewhere.

## Team photos through the MCP — reachable via the attachment-id path

Earlier notes said photos were MCP-impossible. **They are reachable** — just not by the
obvious call. Reading `image_512` returns `false` (binary stripped), and
`odoo_attachment_link(model, res_id, field)` errors *"Model … not readable on this
server"* for `hr.employee`/`res.partner`/`res.users` (proxy model allowlist, not an
Odoo ACL — the proxy runs as sudo). But the images are field-backed `ir.attachment`
rows: `orm_search_read('ir.attachment', [res_model='res.partner', res_id=<pid>, res_field='image_512'])`
→ `attachment_id` → `odoo_attachment_link(attachment_id=…)` → presigned URL → curl.
Full recipe + the dev-team improvement request in **`references/mcp_photo_access.md`**.

Alternative: `scripts/fetch_team.py` (odoo-pilot RPC, needs an API key) still works.
Either way, normalize to a centered 512×512 square (Gotcha 13) — `scripts/embed_team_photos.py`
does that and injects the data URI. The team block renders gracefully with `photo: ""`
if no image is available — never invent one.

## Backward-compat gate (dev)

`es` output is **byte-for-byte frozen** and is the regression oracle. After ANY engine edit:
- Run `python3 tests/golden_check.py` — **bare, no flag**. The bare invocation **IS** the ES
  byte-parity gate (4 fixtures × web/pdf/ctx = 12 artifacts) and it is what CI runs, so keep using
  it exactly like that. Positionals are additive, not a locale flag: `golden_check.py es en fr nl`
  re-checks **all four frozen packs** (~48 artifacts) and `golden_check.py en default` narrows to one
  pack + one fixture (a `locales/` code is read as a locale, a `tests/fixtures/` name as a fixture).
  Never re-freeze to make a diff disappear.
- `money_js_config()` carries **separators / symbol / symbol-position / decimals only — NEVER rounding.** Python `eur()` (`int(round())`, banker's) and template JS (`Math.round()`, half-up) already diverge at `.5`; both are frozen. Do not unify them.
- The `es` pack is authored complete but flagged `pack_status:"stub"`/reference; `en` is the engine base; `fr`/`nl` are drafts (`verify_from_odoo` placeholders) blocked from `live` until `/add-country` + fiscal/native sign-off.

## Dev-process note

When multiple agents/processes edit a large engine file (`generate_proposal.py`) concurrently, a golden run can transiently FAIL from a mid-read of a half-written file. Serialize edits to shared files; re-run the gate after writes settle.
