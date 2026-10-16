# Plan — cover Belgium (BE) and Switzerland (CH) in quotation-builder-value-based

> Status: **approved**. Model = **two-layer split (country ⊕ language)**. BE + CH launch **in parallel** after the engine phase. CH bills from its **own legal entity** (own Odoo + MWST number).
>
> This doc is the single source of truth for the engine owner and for the country owners. Contributing a pack follows the contract in `locales/README.md` (§4 stub→live gate, §6 fail-closed).

## 0. Why BE/CH break the current model

Today a pack is **flat**: `locales/<code>/` with 8 files that fuse **one language + one jurisdiction**. BE and CH are **multilingual, single-jurisdiction countries**, so the flat model forces duplication of the fiscal/pricing/instance files (exactly the `fail-closed` ones). Diagnosis:

| Dimension | Belgium | Switzerland | Verdict |
|---|---|---|---|
| Currency | EUR (same axis) | **CHF**, thousands `'`, symbol in front | Pack data. Relax one discovery assert. **Not a blocker.** |
| Languages / jurisdiction | NL + FR (+ DE) | DE + FR + IT | **The underlying problem** → forces the language×country split. |
| Fiscal | EU · BTW/TVA 21% · **Peppol B2B mandate (Jan 2026)** · Belgian chart of accounts | **Non-EU** (no EU OSS/IOSS/Intrastat/reverse-charge) · MWST 8.1% · **QR-bill** · Swiss chart of accounts | Representable as data in `fiscal.json`, authored per jurisdiction under legal sign-off. |
| Legal entity | Dynapps NV (local market) | **Own CH entity** (own Odoo + MWST) | `instance.json`, `[verify from Odoo]`. |

**Currency is not the hard part**: `proposal_data.py`, the JS injector in `generate_proposal.py`, and `locale_loader.money` already read `thousands_sep`/`symbol_position`/`round_step` from the pack. CHF comes in as data. The only tweak needed: relax `scripts/odoo_discovery.py:688-689`, `_smoke` (`assert symbol_position=='after' and decimals==2`).

## 1. Target model — two-layer resolution (country ⊕ language)

`resolve(code)` composes a pack from a **country layer** and a **language layer**. New codes: `nl-BE`, `fr-BE`, `fr-CH`, `en-CH` (CH = FR+EN, see §4). The current codes `es`/`en`/`nl`/`fr` **resolve exactly as they do today** (backward-compatible) → **the `es` byte-parity gate stays green**. **No new language layers are needed**: BE uses `nl`/`fr`, CH uses `fr`/`en` (all already exist).

### 1.1 File-ownership map (Phase 0)

| File | Layer | Notes |
|---|---|---|
| `fiscal.json` | **Country** | VAT, e-invoicing, filings, chart of accounts, reverse-charge, payment terms, fineprint. |
| `pricing.json` | **Country** | Rates, per-app/complement base prices, AI catalog, recurring items, POC/migration/support. |
| `instance.json` | **Country** | Legal entity, Odoo URL/DB, template id (by name), partner manager, offices, `cases_base_url`. |
| `format.json` → `currency` + `spreadsheet` | **Country** | Currency + spreadsheet locale block. |
| `format.json` → `date` + `period_words` | **Language** | Month names, date format, period words. |
| `strings.json` | **Language** | All UI labels. |
| `apps_catalog.json` | **Language** | Per-app prose. |
| `brand_lint.json` | **Language** | Lexicons + `word_charclass` + messages. |
| `matching.json` → `glossary` + `signals` | **Language** | Sector words, fiscal detection signals. |
| `matching.json` → `cases` (`case_translations`, `sector_token_map`) | **Language** (with an optional country override for selecting local cases) | Localized resultado/metric. |

> The two "straddlers" (`format.json`, `matching.json`) are split by section, not by file. That's the only fine-grained design work in Phase 0.

### 1.2 On-disk layout (proposed convention)

- **Country layer:** `locales/_countries/<CC>/` (e.g. `_countries/BE/`, `_countries/CH/`) with `instance.json`, `pricing.json`, `fiscal.json`, `format.currency.json`. The `_`-prefix ⇒ invisible to the current `discover_locales` (like `_schema/`) until Phase 1 wires it in.
- **Language layer:** `locales/<lang>/` as today (`nl`, `fr`, `en`, `es`). No new languages are needed for BE/CH.
- **Composition:** `nl-BE` = language `nl` ⊕ country `BE`. Strings fall back → `en`; pricing/fiscal/instance/product **never** fall back (hard error at validation).

## 2. Phase 1 — Engine (single PR, backward-compatible) [owner: engine]

1. `locale_loader.load_locale`: compose `lang ⊕ COUNTRY` for `xx-YY` codes; flat codes load as today. Country files are hard-required; language strings fall back to `en`.
2. `resolve_locale`: parse `xx-YY`; **default language per country** for the ambiguous case (BE→`nl-BE`, CH→`fr-CH`) and **log** the resolved code (a country hint alone is ambiguous in BE; in CH the default is FR).
3. `_schema/`: split the schemas by layer; `validate_locale.py`: key-parity **within each layer** + referential integrity on the composed pack; extend `discover_locales` to be aware of `_countries/`.
4. `scripts/odoo_discovery.py:688-689`, `_smoke`: relax the currency assert (allow symbol in front / decimals ≠ 2) → CHF passes.
5. **`cases_base_url` per locale** (see §4).
6. **Gate:** parity suites + smoke tests for `es`/`en`/`nl`/`fr` green before merge. Nothing client-facing changes yet.

## 3. Phase 2 — BE and CH country layers (in parallel, after Phase 1)

Each combo goes `stub → live` through the §4 gate in `locales/README.md`: validator green · smoke green (zero unresolved `{{}}`) · every `verify_from_odoo` confirmed · **named fiscal/legal sign-off** · native-speaker sign-off. **Never** machine-translated fiscal/legal copy.

- **BE** → `_countries/BE/` + deltas in `nl`/`fr` → publish `nl-BE`, `fr-BE`.
- **CH** → `_countries/CH/` (reuses the existing `fr`/`en` layers, with CH deltas) → publish `fr-CH`, then `en-CH`. **de-CH/it-CH do not apply** (there is no Swiss DE/IT site).

## 4. Case-study links — `cases_base_url` per locale

**Original problem (RESOLVED):** `match_cases._localized_cases` used the **shared absolute** `url` from `shared/cases_library.json` (back then a **temporary** November Five URL: `dyna-0001-markets-es.novemberfive.co`) — the same URL for every locale, wrong for the group. Today the per-locale composition below is **landed** (see §7) and the shared `_base_url` points to production `https://www.dynapps.es/en/cases` (`_base_url_TEMP: false`), kept only as fallback. The skill's rule stands: cases link to **the dynapps case-study website** (never to the Odoo backend).

**Design:** the case-study link is **per locale**, `cases_base_url` in `instance.json` (country layer), composed with each case's `slug`: `url = cases_base_url + slug`. `_localized_cases` composes it from the pack; the shared absolute `url` remains as a fallback.

**VERIFIED URLs (HTTP 200, checked 2026-07) — dynapps.eu → 302 → dynapps.be:**

| Locale | `cases_base_url` | Status |
|---|---|---|
| **es** | `https://www.dynapps.es/es/cases` | ✅ 200 (own ES site, Spanish only) |
| **en** | `https://www.dynapps.be/en/cases` | ✅ 200 (dynapps.eu redirects to dynapps.be) |
| **nl-BE** | `https://www.dynapps.be/nl/cases` | ✅ 200 |
| **fr-BE** | `https://www.dynapps.be/fr/cases` | ✅ 200 |
| **fr-CH** | `https://www.dynapps.ch/fr/cases` | ✅ 200 (own CH site, Romandy) |
| **en-CH** | `https://www.dynapps.ch/en/cases` | ✅ 200 |
| (nl-NL) | `https://www.dynapps.nl/nl/cases` | ✅ 200 (future country NL) |
| (fr-FR) | `https://www.dynapps.fr/fr/cases` | ✅ 200 (future country FR) |
| ~~de-* / it-*~~ | — | ❌ 404 on every domain (no DE/IT site exists) |

**Key findings from the verification:**
- The **domain is per country** (dynapps.es, dynapps.be, dynapps.ch, dynapps.nl, dynapps.fr), the **language is the path** (`/nl`, `/fr`, `/en`, `/es`). That's why `cases_base_url` lives in the **country** layer (domain) and is composed with the `{lang}` from the language layer: `domain/{lang}/cases + slug`.
- **Switzerland = Romandy, FR + EN only** (`dynapps.ch/fr` and `/en`; `/de` and `/it` = 404). **There is no Swiss DE/IT site.** → CH is served with **fr-CH + en-CH**, reusing the existing `fr`/`en` layers; **de-CH/it-CH do not apply**.
- **No new language layers are needed** for BE or CH: BE = nl + fr (already exist), CH = fr + en (already exist). The `de`/`it` stubs created at the start were **removed** (they were for a market that doesn't exist).

> **Re-baseline note (DONE for `es`):** the `es` switch to `cases_base_url = https://www.dynapps.es/es/cases` already happened and the `es` golden was re-recorded against it (green as of 2026-07). The caveat still applies **going forward**: any change to a locale's `cases_base_url` or to a case's `slug` changes that locale's output → re-record the affected golden in a coordinated way (an action for the engine owner), never a silent edit, and verify each case's `slug` is still valid on each definitive site.

## 5. Intake checklists

Tags: `[verify from Odoo]` = confirm against the entity's Odoo / authority (never invent) · `[author, legal]` = author from facts under local fiscal/legal sign-off.

### Belgium (Dynapps NV)
- **Instance:** Dynapps NV BE entity text · VAT `BE0…` · offices · Odoo URL/DB · `sale.order.template "Implantación v2"` id **by name** · AI product ids · Odoo licence price/plan · Odoo-SA partner manager — all `[verify]`. `cases_base_url` = `dynapps.be/<lang>/cases` `[verify]`.
- **Pricing:** per-app bases, complements, internal analysis rate, POC %, 10% support, maintenance, Dyn Cloud, licence — EUR, `round_step` `[author, market owner BE]`.
- **Fiscal `[author, legal]`:** BTW/TVA **21%**; e-invoicing = **B2B mandate**, authority + Peppol/Hermes network, Peppol BIS formats, **phase date 2026-01-01**, threshold; Belgian chart of accounts (PCMN/MAR); intra-EU reverse-charge; OSS/IOSS; payment terms (Belgian late-payment law); fineprint.
- **Language:** `nl-BE` vs `nl-NL` and `fr-BE` vs `fr-FR` deltas (bpost/DPD carriers, sector words, local Belgian cases).

### Switzerland (dedicated CH entity)
- **Instance:** CH entity text · **UID / MWST number** · CH Odoo URL/DB · template id by name · product ids · licence price **in CHF** · partner manager — all `[verify]`. `cases_base_url` = `dynapps.ch/<lang>/cases` `[verify]`.
- **Currency/format:** CHF, symbol in front, thousands `'`, decimals + `round_step` `[verify]`; sheet block `fr_CH` (+ `en`).
- **Fiscal `[author, legal]`:** MWST **8.1%** (+ reduced rates 2.6% / 3.8% where applicable); **non-EU** → **remove** EU OSS/IOSS/Intrastat/reverse-charge, add an import-VAT note; **QR-bill** as a payment method; Swiss chart of accounts; Swiss payment terms; fineprint.
- **Language:** `fr-CH` (reuses `fr` with CH deltas) and `en-CH` (reuses `en`). **No `de`/`it`**: dynapps.ch is Romandy (FR+EN); no Swiss DE/IT site exists.

## 6. Phase 3 — Flip the warning

Only once the packs are `live`: replace the rule "🚫 do not serve BE/CH" in `SKILL.md`, the webinar deck, and the cheat-sheet with "BE/CH active via nl-BE / fr-BE / fr-CH / en-CH".

## 7. Scaffolding status (this session)

- ✅ This plan (`references/add_be_ch_plan.md`).
- ✅ Case-study URLs **verified against the live sites** (§4): domain per country + language per path; CH = FR+EN (Romandy).
- ❌ (reverted) `de`/`it` stubs: created and **removed** — dynapps.ch doesn't serve DE/IT, so BE/CH need no new languages.
- ✅ Country-layer skeletons `locales/_countries/BE/` and `locales/_countries/CH/` (instance/pricing/fiscal/format-currency with `<<VERIFY>>` + known facts + `cases_base_url`) — inert until Phase 1.
- ✅ **Phase 1 engine (partial) — LANDED:** `locale_loader` composes `<lang>-<CC>` (language ⊕ country); `resolve_locale` accepts composite codes + defaults BE→nl-BE / CH→fr-CH; `match_cases` composes `cases_base_url + slug` per locale (flat packs unchanged). `es` golden **green (0 diffs)**; new composition smoke test (`tests/composition_smoke.py`). The currency assert in `scripts/odoo_discovery.py:688-689`, `_smoke` was only for the fake-EUR smoke test (doesn't block CHF) → no change needed.
- ⏳ Pending Phase 1 (follow-up): **schema split + `validate_locale` for country layers** (validate `_countries/` as a subset). Requires the engine owner.
- ⏳ Pending (deliberate): **re-baseline the `es` golden** when the temporary case-study URL changes → `dynapps.es/es/cases` (changes `es`'s output; NOT in this PR).
- ⏳ Pending (country owners + sign-offs): fill in `verify_from_odoo`, fiscal/legal + native-speaker sign-offs per combo.
