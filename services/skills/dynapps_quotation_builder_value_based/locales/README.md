# Locale packs — country-owner governance & contribution contract

This directory holds the **per-country locale packs** for `sales-skills:quotation-builder-value-based`.
The Python engine (`../scripts/*.py`), the HTML templates (`../templates/*.html`), the shared
data (`../shared/*`) and the JSON Schemas (`_schema/*.schema.json`) are **shared and never
forked**. Adding a country = adding a pack of config/data files here + getting the required
sign-offs. It is **never** an engine-code edit for a country whose structure is a subset of the
shared schema (see the honest scope caveat in the plan §7-A1).

- **`en` is the engine's base / default / reference locale.** The UI-string fallback chain ends
  at `en`; the HTML `lang`/copy default is English.
- **`es` is authored complete** because it is the **byte-for-byte regression oracle** — proof that
  the refactored engine + `es` pack reproduces today's frozen Spanish output exactly. It is
  complete-but-flagged (`pack_status:"stub"`) until its parity gate is recorded green.
- **`fr`, `nl` ship as stubs** (`pack_status:"stub"`, EN-fallback strings, `verify_from_odoo`
  placeholders) and are **blocked from production by the loader** until fully verified and signed off.
- **Connector-agnostic.** No hardcoded instance URL (e.g. `portal.dynapps.es`) / `auth.sh` / fixed DB id anywhere in a pack.
  Each country operator runs their own Dynapps MCP connector against their own Odoo DB. The engine
  discovers instance specifics at runtime where possible (e.g. resolves the `sale.order.template`
  "Implantación v2" id **by name** in the connected DB — never hardcode `15`). `instance.json`
  holds only the non-discoverable copy (legal-entity text, fiscal copy, pricing, partner-manager
  display text).

---

## 1. File layout — one pack per locale

Each locale directory (`es/`, `en/`, `fr/`, `nl/`, …) contains the **same set of files** with the
**same keys**. Authors replace *values*, never *keys*. Files are grouped by **owner competence**
(not by engine-internal category) so review routing and this contract stay clean.

| File | Owner | Contents | Fallback on missing |
|---|---|---|---|
| `locale.json` | **ops** | `code`, `name`, `lang_tag`, `odoo_lang`, `schema_version`, `fallback`, `output_language`, `pack_status` | hard error |
| `format.json` | **ops** | currency format/parse, spreadsheet locale block, date format + month names, period words | hard error |
| `instance.json` | **ops** | Odoo URL/DB, template id (discovered), legal entity, partner manager, offices, credibility, contact | hard error |
| `pricing.json` | **market pricing owner** | all rates, per-app & complement base prices, IA catalog, recurrent lines, POC/migration/support/maintenance | hard error (never falls back) |
| `fiscal.json` | **finance / legal owner** | VAT label, e-invoicing regime (structured), typed tax filings, chart of accounts, reverse-charge, payment terms, fineprint, carriers, fiscal acronyms | hard error |
| `strings.json` | **translator** | ALL UI labels (web / pdf / cli / sheet), phases, recurrent display names+descs, dev fallback | → `en` (explicit, testable) |
| `apps_catalog.json` | **consultant** | per-app prose keyed by `app_id` (outcome, intro, value, features, formaciones, migraciones, core) | hard error |
| `matching.json` | **consultant** | signals + glossary + local success-case translations (`case_translations`, `sector_token_map`) | hard error |
| `brand_lint.json` | **translator** | brochure / jargon / slop / fluff lexicons + mechanical-rule data (`stopwords`, `leak_denylist`, `register`, `passive_patterns`, `pressure` — dynapps-voice draft 2026-07-17) + `word_charclass` + messages | hard error |

**Shared, not per-locale** (do not copy into a pack):
- `../shared/app_registry.json` — canonical `app_id`s / category keys / phase codes / dev ids /
  complement ids (extensible with namespaced ids like `fr.chorus_pro`).
- `../shared/cases_library.json` — international cases: id / slug / url + **language-neutral** sector
  tokens + pain archetype. The localized `resultado`/`metric` for each case live in each pack's
  `matching.cases.case_translations[<id>]`.
- `../shared/slop_en_core.json` — universal AI-slop tells + universal acronyms (shared fallback).
- `_schema/*.schema.json` — the JSON Schema contract each pack file is validated against.

**Fallback policy is split and enforced (do not blur it):**
- **UI display strings** (`strings.json`) may fall back **to `en`** — explicitly, and only for
  cosmetic labels.
- **Pricing, fiscal, product ids, instance/legal values** **never** fall back. A missing value is a
  **hard error at validation time** (in CI, not in front of a client). Fail-closed, never fail-Spanish.

---

## 2. Ownership & required reviewers (CODEOWNERS)

- **Engine maintainer** owns `../scripts/`, `../shared/`, `../templates/`, and `_schema/`. Changes
  there require the engine maintainer's review and must keep the `es` parity gate green.
- **Per-pack files** are owned by the roles in the table above. A PR touching a pack file requires
  the review of that file's owner role. Configure `CODEOWNERS` so each `locales/<code>/` path routes
  to its country owners, and each file class routes to its competence owner.
- No pack file merges without its **owner-role reviewer**. `fiscal.json` and the fineprint
  additionally require a **named local finance/legal sign-off** recorded in the PR.

---

## 3. Intake checklist

Before a pack can move from `stub` to `live`, its country owner must obtain every fact listed in
the **per-country intake checklist — plan §5**. Do not restate or fork it here; §5 is the single
source. It is grouped as: Instance/org · Currency/format · Pricing · Fiscal/legal · Content/matchers.

Items tagged **[verify]** in §5 are **Odoo-SA-set or legally-adjacent** and must be **confirmed from
the entity's own Odoo / authority — never authored or invented**. Examples: the `sale.order.template`
id in *that* DB (differs per DB, not `15`), Odoo licence price/plan, IA product ids, the Odoo-SA
partner manager, `legal_entity` VAT/address, `seller_vat_number`. `verify_from_odoo`-tagged fields
that still hold placeholder or inherited-`es` values **fail validation**.

---

## 4. Pack lifecycle: `stub` → `live`

Every `locale.json` carries `pack_status`. The loader treats it as a gate:
`"stub"` = schema-valid but **blocked from production use**; `"live"` = clearable for client output.

A pack may flip `pack_status:"stub"` → `"live"` **only when ALL of the following are recorded in
the promoting PR**:

1. **Validator green** — `python scripts/validate_locale.py <code>` passes: schema conformance +
   cross-pack key-parity (key-isomorphic to the reference set; missing = error, extra = error unless
   namespaced) + referential integrity (every referenced `app_id` / complement / `product_id` /
   shared case id resolves) + **no `verify_from_odoo` field left at a placeholder / inherited-`es`
   value** + **pack-string style lint** (v5: the validator also runs the brand_lint DASH/MARCA
   checks over the pack's own strings — no em-dashes, always "Dynapps" capital D).
2. **Smoke green** — a full proposal generated from a fixture at `--locale <code>` produces **zero
   unresolved `{{...}}`** and **zero empty required renders**.
3. **All `verify_from_odoo` fields confirmed** — every `[verify]` item from the §5 checklist
   confirmed against the entity's Odoo / the relevant authority, not inherited or guessed.
4. **Fiscal / legal sign-off** — a **named** local finance/legal reviewer has approved `fiscal.json`
   (e-invoicing regime, tax filings, reverse-charge, payment terms incl. any mandatory late-payment
   mentions) and the fineprint. **No machine-translated fiscal/legal text ever ships** — it is
   authored from the structured facts.
5. **Native-speaker sign-off** — a native speaker has approved `strings.json`, `apps_catalog.json`,
   and the `case_translations` in `matching.json`.

For jurisdiction-sensitive packs (`fr`, `nl`), acceptance additionally requires **fiscal fixtures**
that assert the *local* tax filings / e-invoicing signals fire (not the ES ones).

`es` is a special case: it is authored complete and its promotion gate is the **byte-for-byte parity
suite** (static golden diff + live-DOM totals + semantic-equality gate + locale-leakage sentinel),
per plan §6.

---

## 5. How to add a new country

1. **Copy the reference pack.** `cp -r en <newcode>` (copy `en`, the base locale — it has the full
   file set with clean, low-fiscal-risk defaults and correct fallbacks). Set `code`, `name`,
   `lang_tag`, `odoo_lang`, `fallback` (usually `"en"`), `output_language`, and
   `pack_status:"stub"` in `locale.json`.
2. **Translate the UI strings** in `strings.json`, `apps_catalog.json`, `matching.json`,
   `brand_lint.json` — keep every key; replace values; extend `word_charclass` with the language's
   accented letters so the tokenizer doesn't drop them.
3. **Fill the market values** in `pricing.json` and `format.json` (rates, base prices, currency
   format, spreadsheet locale block, date format, month names).
4. **Author the fiscal structure** in `fiscal.json` from the jurisdiction's real regime (structured
   e-invoicing mandate status/authority/network/formats/phase dates/threshold; typed `tax_filings`;
   chart of accounts; reverse-charge; payment terms). Author from facts, under local legal review.
5. **Fill the `verify_from_odoo` fields** in `instance.json` and `pricing.json` by **confirming from
   the entity's own Odoo** (template id by name, product ids, licence price, partner manager,
   legal entity). Do not inherit `es`/`en` values.
6. **Re-author the international success cases** — add `case_translations[<id>]` (resultado + metric)
   in `matching.json` and `sector_token_map` (local sector words → the neutral tokens in
   `shared/cases_library.json`). This is real work, not a pointer swap.
7. **Run** `validate_locale.py <newcode>` and the per-locale smoke test until green.
8. **Collect the two sign-offs** (fiscal/legal + native-speaker), record them in the PR, then flip
   `pack_status` to `"live"`.

If the new country needs a **new structural primitive** (a new app / complement / phase / a currency
behavior outside the supported axis — see plan §2.2), that is an **engine change** to `../shared/` +
schema + a re-validation of all packs, **not** a pack-only change. Coordinate with the engine
maintainer first.

---

## 6. Fail-closed rules (non-negotiable)

- **A stub cannot ship.** The loader refuses to produce client output for a `pack_status:"stub"`
  pack. Promotion requires the §4 gate.
- **Missing pricing / fiscal / product / instance value = hard error at validation**, surfaced in CI
  — never a silent fallback, and never a silent fall-back-to-Spanish.
- **Only UI display strings fall back, and only to `en`** — explicitly and testably. Everything
  else is hard-required.
- **`verify_from_odoo` values must be confirmed, never invented.** A placeholder or inherited value
  in one of these fields fails the validator; the checklist word is "confirm from Odoo," not "author."
- **No machine-translated fiscal/legal copy.** Jurisdiction copy is authored from structured facts
  and requires a named local finance/legal reviewer.
- **No hardcoded instance specifics in a pack.** Discover at runtime where possible; put only the
  non-discoverable copy in `instance.json`. Never hardcode instance URL (e.g. `portal.dynapps.es`), `auth.sh`, or a fixed DB id.
- **Keys are a contract.** Packs must stay key-isomorphic to the reference set. Adding a key is an
  engine/schema change; a pack that drops or adds an un-namespaced key fails the validator.
