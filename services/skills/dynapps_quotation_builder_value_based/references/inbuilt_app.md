# Dynapps Construcción — Mobile app for the construction sector

**IMPORTANT:** This information is NOT in `common-skills:content-collection` at a commercial-product level.

`content-collection` covers Inbuilt only at:
- **Technical level**: `data/vertical-odoo-inbuilt-modules.json` (15 `inbuilt_*` modules of the Odoo vertical: `inbuilt_base`, `inbuilt_base_project`, `inbuilt_fiebdc3_import`, `inbuilt_project_certification`, `inbuilt_sale_measurement`, etc.).
- **Sitemap / blog level**: pages at `https://www.dynapps.es/inbuilt/...` (project-sale flow, BC3 import) and blog category `blog-inbuilt-4`.

But there is **NO product card** ("Mobile app iOS/Android for construction") in `data/modules.json`. The `[CORE] App móvil` module (id=308) exists with `description: ""` and `features: []` — an empty card.

This local reference fills that gap until `content-collection` is updated.

---

## Dynapps Construcción — Functional description for quote cards

Native mobile app (iOS and Android) to coordinate and monitor construction site projects in real time from the field.

### Visibility
- Coordinate and monitor site projects in real time.
- Full control over each site's status and progress from mobile.
- Information synced with the office and the ERP at all times.

### Efficiency
- Optimize resource allocation and task planning.
- Maximize the profitability of machinery and site staff.
- Reduce dead time and duplicate record-keeping.

### Communication
- Improve collaboration between teams and subcontractors.
- Full sync between the field and project management.
- Two-way office ↔ site communication with no paper or scattered emails.

---

## How it's quoted

- The **Dynapps Construcción** app is sold as an add-on to the construction vertical.
- It's usually accompanied by:
  - Project app + site add-ons (certifications, measurements, BC3).
  - ⚙️ Stock in projects (materials per site).
  - ⚙️ Per-project clock-in (site staff).
- If the client is in the construction sector and the intake data mentions "site", "on-site", "subcontractors", "BC3", "site certifications", "measurements", "field staff", **propose Dynapps Construcción**.
- On the client-facing card, do NOT mention the technical `inbuilt_*` modules. Talk about functional capabilities (Visibility / Efficiency / Communication) and tie them to the concrete pain points from the intake data.

---

## Intake-data signals to detect Dynapps Construcción

| Signal (Spanish, matched verbatim in the intake document) | Indicates |
|-------|--------|
| "obra", "pie de obra", "operarios en campo" (site, on-site, field staff) | Needs Dynapps Construcción |
| "subcontratistas", "industriales" (subcontractors, trades) | Needs mobile communication with third parties |
| "BC3", "presupuesto FIEBDC3", "Presto", "Arquímedes" | Dynapps Construcción vertical (BC3 importer) |
| "certificaciones de obra", "mediciones", "comparativos" (site certifications, measurements, comparisons) | Dynapps Construcción vertical + App |
| "encargado de obra", "jefe de obra reporta" (site foreman, site manager reports) | Mobile app for progress reporting |
| "parte de trabajo en papel", "parte diario" (paper work reports, daily report) | Replace paper with the App |
| "maquinaria en obra", "alquiler maquinaria" (on-site machinery, machinery rental) | Resource tracking via the App |

---

## When NOT to propose Dynapps Construcción

- The client isn't in the construction sector.
- The client has no field team (office/warehouse only).
- The client already has their own integrated mobile app (confirm with the sales rep first).

---

## Cross-references

- Technical vertical: `common-skills:content-collection` → `data/vertical-odoo-inbuilt-modules.json`
- Website: https://www.dynapps.es/inbuilt/flujo-venta-proyecto
- Success case: see `common-skills:content-collection` (`data/cases.json`) for the current URL of the Dynapps Construcción case study on dynapps.es (the blog slug changed after the migration; don't hardcode the old one). TM Technics and Despraz (construction) are already in `shared/cases_library.json`.
- Inbuilt blog: https://www.dynapps.es/blog/blog-inbuilt-4

---

## Render contract in the engine today (the authoritative statement)

The first-class-app migration is **partly shipped**. This is what the code does right now — the
statement "price tile only, NO app card yet" is obsolete; do not carry it forward.

**Scoping.** `deal.vertical = 'inbuilt'` in the `Parámetros` sheet (or `extra_data.vertical`)
activates the vertical. Prices come from the pack: `vertical.inbuilt.pack_oneshot` (6,000 €,
**includes** the mobile App) and `vertical.inbuilt.app_oneshot` (2,500 €, App sellable standalone),
with JS bundle logic (pack ⊃ app, never double-charged). Post-demo selection arrives as
`extra_data.vertical_sel = {pack, app}` (the "Copiar selección" payload carries `vertical`).

**Where it renders** (`generate_proposal.py`):

| Surface | What | Where (grep the symbol, not a line number) |
|---|---|---|
| Web, Applications section | **First-class narrative app card**, *no price* (see "Shipped" below) | `apps_section_web()`, the branch guarded by `app_overrides.get("Dynapps Construcción")`, anchored next to the other app cards |
| Web, Investment section | **`#vert-block` price tile** (between Developments and AI) | `_vertical_block_inner()` (emits `id="vert-block"`) |
| PDF | `vert_block` inside the same value-added group | `_pdf_inversion_blocks()`, local `vert_block` |
| Context | `vertical` dict (`active`, `pack_oneshot`, `app_oneshot`, `pack_sel`, `app_sel`, `num_usuarios_app`) | `build_context()`, under the comment `--- Vertical construcción (Dynapps Construcción) ---` |

**Commercial rules that are code, not prose.** The vertical is a **closed price WITHOUT hours**: the
one-shot tile does **not** feed the 10% go-live support, but it **does** enter the future-migration
base (`mig_base_inicial`) when checked.

**Double-charge suppression** (in `build_context()`, grep the comment `SUPRESIÓN DOBLE COBRO`): if the vertical also arrives as a **first-class sheet
app** — a workbook tab whose base cell is `Dynapps Construcción — Base`, so it lands in
`active` / `c['apps']` — then `vertical["active"] = False` and the one-shot Parámetros route is
switched **off**. The one-shot route survives **only** for older bases without that tab.
`_VERT_APP_NAME = "Dynapps Construcción"` must stay identical to the registry's
`sheet_display_name` (app id `inbuilt`).

**Byte-parity guard:** the vertical is emitted **only when it is active**, which is exactly what
keeps the ES reference render byte-identical (`tests/golden_check.py`). It must never leak into a
non-vertical deal's output.

**Already in place besides the render:** the registry entry `inbuilt`
(`shared/app_registry.json`: `sheet_display_name` / `es_display_name` `Dynapps Construcción`,
`category: vertical_construccion`, `icon: inbuilt.svg`) with `app_to_sheet_group['inbuilt'] =
'Vertical Construcción'`; the `locales/es/apps_catalog.json` `inbuilt` entry (display name, outcome,
intro, value); and `generate_spreadsheet_template.py --vertical inbuilt|vertical_construccion`,
which emits the vertical's sheet opt-in.

**Still pending from the spec below** (do not assume these):

1. The `soporte_exempt` registry flag is **not implemented**. `calc_soporte_salida()` takes the
   marked implementation amounts as-is, so if the vertical arrives as a **sheet app** its € **does**
   currently generate support hours. The "closed price without support hours" rule is only guaranteed
   today on the one-shot tile path.
2. `locales/{en,fr,nl}/apps_catalog.json` have **no `inbuilt` entry** yet (es only).
3. `extra_data.vertical_sel` is **not** deprecated yet; it is still the post-demo selection input.

## Approved spec — full first-class app

Goal: a narrative app card (intro/fit/features, Included/Adapted tiers) + a checkable row in the
single price list, replacing the tile without duplicating the 6,000 €. **Status per item** (the
render contract above is the authority on current behaviour):

| # | Status |
|---|---|
| 1 — 'Vertical' sheet in the template | **SHIPPED** in the generator: `generate_spreadsheet_template.py --vertical inbuilt` (or `vertical_construccion`) emits the `Vertical Construcción` sheet with `A3='Dynapps Construcción — Base'`, `D3=6000`. Bases already deployed in Odoo do NOT have it until re-generated |
| 2 — no structural change in `proposal_data.py` | **SHIPPED / n.a.** (the generic app path handles it) |
| 3 — registry + catalog + icon | **SHIPPED for `es`**: registry `inbuilt` + `app_to_sheet_group`, `assets/odoo-icons/inbuilt.svg`, `locales/es/apps_catalog.json`. **Pending:** `en`/`fr`/`nl` catalog entries |
| 4 — no duplication with the tile | **SHIPPED** (`generate_proposal.py`, `SUPRESIÓN DOBLE COBRO` guard in `build_context()`). `extra_data.vertical_sel` is **not** deprecated yet |
| 5 — `soporte_exempt` flag | **NOT SHIPPED** — see the pending list above; the exemption only holds on the one-shot tile path today |

The spec text below is kept verbatim as the target design (note it predates the rename to
`Dynapps Construcción`: the sheet cell and the registry now use that name, not `Inbuilt
(Construcción)`).

1. **New 'Vertical' sheet** in the spreadsheet template (do NOT touch 'Servicios' — keeps golden
   byte-parity; app discovery is multi-sheet and picks it up automatically). Same pattern as
   Proyecto/Helpdesk: `A3='Inbuilt (Construcción) — Base'`, `D3=6000`; `B4='Funcionalidades'` +
   ✓ rows (BC3 import Presto/Arquímedes · site certifications with measurements · measurements on
   sale lines · indirect costs and real margin per site · projects created from the order · iOS/
   Android mobile app included — copy from this doc's Visibility/Efficiency/Communication);
   `B='Formaciones'` + ✓ '2 sesiones formación — Jefes de obra'; `Extra` block: 'Inbuilt APP móvil
   suelta (solo sin el pack)' qty=0 × 2,500; `Subtotal Extras`; `TOTAL Inbuilt (Construcción)`.
   **Operating rule:** the pack ALREADY includes the app → the 2,500 extra is only set to qty=1
   with base D3 set to 0 (standalone app sale; base=0 + extras>0 keeps the app active). Never
   base=6000 + extra qty=1 (that would be 8,500 €).
2. **`proposal_data.py` — no structural changes** (`_app_matches`, `calc_app_price_split`,
   `extract_app_details` are generic). The `Parámetros` rows 26-27 (`vertical.inbuilt.*`) become
   informative only: the sheet cell D3 becomes the price authority, like every other app.
3. **Registry + catalogs + icon:** `shared/app_registry.json` entry `inbuilt`
   (`sheet_display_name: 'Inbuilt (Construcción)'`, `category: 'vertical'`, `icon: 'inbuilt.svg'`,
   new flag `soporte_exempt: true`), added to `_apps_order` after `proyecto` and to
   `app_to_sheet_group`; `locales/*/apps_catalog.json` entry (outcome: 'Obra certificada y margen
   real, de la oficina al pie de obra'; intro/features from this doc, no `inbuilt_*` module names);
   `assets/odoo-icons/inbuilt.svg`. The web card then renders for free via `apps_section_web`;
   per-deal fit via `extra_data.app_overrides['Inbuilt (Construcción)'].fit` — once this ships,
   the override becomes MANDATORY like every other active app. Move the app-users note
   (`deal.num_usuarios_app`) from the tile to the card.
4. **No duplication with the tile:** if `'Inbuilt (Construcción)'` is in `c['apps']`, force
   `vertical={'active': False}` (web `#vert-block` and PDF block 2.bis disappear; the app enters
   via `apps_marcadas_sum`). `deal.vertical='inbuilt'` stays as the scoping switch (include +
   premark the sheet section). `extra_data.vertical_sel` is deprecated: `{pack:true}` → premark
   the app; `{app:true}` → base 0 + extra qty 1. **Backward compatibility:** old deals WITHOUT the
   'Vertical' sheet keep rendering the current tile (goldens intact).
5. **10% support — keep the commercial rule 'vertical = closed price WITHOUT support hours'**
   (otherwise 6,000 € would add ~8.7 h ≈ +650 € of support). Data-driven via the
   `soporte_exempt` registry flag: exclude from `calc_soporte_salida`/`soporte_inicial` inputs
   in Python, emit `data-no-soporte="1"` on the `<li>` and filter it in the live JS support
   recalculation + copy-selection payload. It DOES enter `mig_base_inicial` (code that gets
   migrated). Tests: one golden with the 'Vertical' sheet (card + row + unchanged support) and one
   legacy golden without it (tile intact).

**Pending business decisions before finishing item 5:** confirm with Iñaki that the mobile app stays
inside the 6,000 € pack (today it does: "Incluye la App móvil"), and ask the client for the
number of site users (`deal.num_usuarios_app`, the intake `[PREGUNTAR]` item).

---

## Shipped (narrative card): first-class app card for the vertical

Ahead of the full migration above, the web proposal already renders a **first-class narrative
card** for the construction vertical in the **Applications** section — same look as the CRM / Ventas
cards (icon, `Incluido` / `Adaptado a tu caso` tiers, per-feature explanations) — while the **price
stays in the Investment tile** (no double billing, exactly like every other app: card sells,
tile prices).

- **When:** rendered only if `c['vertical']['active']` (i.e. `deal.vertical='inbuilt'`) AND
  `extra_data.app_overrides` carries an entry keyed **`'Dynapps Construcción'`** (the canonical key,
  looked up first; also accepts, in order, `'Inbuilt (construcción)'` / `'Inbuilt'` /
  `'Vertical construcción — Inbuilt'`) with at least an `intro` or `features_incluido`. Emitting it
  only under the active vertical keeps the ES golden byte-identical (no re-freeze).
- **Override schema** (in `extra_data.app_overrides['Dynapps Construcción']`):
  - `disp` — card title (default `"Dynapps Construcción"`).
  - `outcome` — one-line outcome under the title.
  - `intro` — HTML `<p>` paragraph.
  - `fit` — one line rendered as the `Para <cliente>:` callout.
  - `features_incluido` — `{feature name: explanation}` → **Incluido** tier (violet).
  - `features_adaptado` — `{feature name: explanation}` → **Adaptado a tu caso** tier (orange).
  - (`features` is accepted as an alias for `features_incluido`.)
- **Defaults:** `disp` → `"Dynapps Construcción"`, `outcome` → `"Obra, certificaciones y coste real,
  en Odoo"`.
- **Icon:** the card reuses `project.svg` (`_app_icon_data_uri("Proyecto")`, letter-avatar fallback);
  the registry entry declares `inbuilt.svg`, used when the vertical arrives as a sheet app.
- **Investment layout:** when the vertical is active the Investment bento switches to
  `.bento-grid.has-vert` and stacks **IA + Vertical** in the right column (`.bento-side`), so the
  vertical no longer sits alone in the left column leaving the right half blank. CSS is emitted
  inline only in that case (ES render unchanged).
