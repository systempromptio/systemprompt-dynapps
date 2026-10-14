# NL locale pack — REVIEWED DRAFT (`pack_status: "stub"`)

**Status:** authored **reviewed draft**. Machine `pack_status` is `"stub"` (the loader
blocks it from production; the schema enum is `reference|live|stub` — there is no `"draft"`
value, so `"stub"` is the correct machine status for a not-yet-live pack). The language +
NL jurisdiction structure are authored in Dutch; entity/pricing/product-id/partner-manager
fields remain `<<VERIFY>>` until confirmed and both sign-offs are recorded.
**UI-string fallback:** `locale.json.fallback = "en"`.
**Validation:** `python3 scripts/validate_locale.py nl` → 0 errors (2 expected `ref`
warnings for the NL-specific `tax_filings` ids `btw_aangifte` / `icp_opgaaf`, which are
jurisdiction-specific and not ES-registry complements — by design).

## What is authored (Dutch) in this draft
- `locale.json` — `output_language` (Nederlands, u-vorm).
- `format.json` — NL conventions (€ vóór bedrag met spatie, `.`/`,` separators, `dd-mm-yyyy`, `nl_NL` spreadsheet block, Dutch month names, jaar/maand period words).
- `fiscal.json` — **NL regime, structured and researched** (see Sources below).
- `strings.json` — all UI labels translated to Dutch, key-isomorphic with `en`.
- `apps_catalog.json` — per-app prose + extras translated to Dutch (Core label = "Dynapps Core").
- `matching.json` — Dutch signals/keywords, glossary, ERPs (NL market: Exact, AFAS, Twinfield, King, SnelStart…), `sector_token_map`, and Dutch case narratives; NL fiscal signals present so the matcher fires (**ICP / icp-opgaaf / intracommunautair**, **BTW verlegd**, **Peppol / NLCIUS / SI-UBL / e-facturatie**).
- `brand_lint.json` — Dutch brochure/jargon/slop/fluff lexicons; `word_charclass` extended with NL diacritics (ë ï ö ü é è …).

## NL e-invoicing reality (as authored in `fiscal.json`)
- `einvoicing.mandate_status = "b2g_only"`. **B2G is mandatory** (central government since 2017, decentralised since 2019), implementing EU Directive 2014/55/EU (EN 16931), routed via **Digipoort**, preferred format **Peppol BIS 3.0** (also **NLCIUS / SI-UBL 2.0**).
- **No general B2B mandate** in force — B2B e-invoicing is **voluntary** in NL (buyer consent required).
- Authority: **Logius** (central-gov implementation) + **Nederlandse Peppolautoriteit (NPa)** / Ministry of the Interior (policy).
- **ViDA** (Council Directive (EU) 2025/516): structured e-invoicing + digital reporting mandatory for **intra-Community B2B** from **1 July 2030** (EU-locked). Domestic NL B2B is still only **proposed** (evaluation report 10 Mar 2026; cabinet position expected summer 2026; consultation Q4 2026). Blog claims of enacted 2027/2028 turnover-based B2B mandates are **not** enacted law and were deliberately NOT encoded.
- **Belgium (`nl_be`) DIFFERS and is OUT OF SCOPE:** Belgium has a structured **B2B e-invoicing mandate effective 1 Jan 2026** (Peppol BIS). This pack is **Netherlands ("nl") only**. A separate `nl_be` pack (mandate_status `b2b_full`/`b2b_phased`, TVA/BTW BE, different late-payment/mandatory mentions) is required for Belgium and is not authored here.

### Sources
- European Commission — *eInvoicing in The Netherlands*: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108895/eInvoicing+in+The+Netherlands (B2G mandatory since 1 Nov 2019; NO B2B mandate, voluntary + buyer consent; Logius / Min. of the Interior; Peppol BIS 3.0 / NLCIUS / SI-UBL 2.0; Digipoort).
- Logius — *e-factureren* (Nederlandse overheid), B2G via Peppol: https://www.logius.nl/domeinen/gegevensuitwisseling/e-factureren
- ViDA — Council Directive (EU) 2025/516 (intra-Community B2B mandatory 1 Jul 2030).
- NL domestic B2B status (proposed, not enacted): Secretary of Finance evaluation report to Parliament, 10 Mar 2026.

## Sign-off checklist — required before flipping `pack_status: "stub" → "live"`

### Instance / org (`instance.json`) — ops
- [ ] **[verify]** `odoo_url` + `odoo_db` (NL entity)
- [ ] **[verify]** `sale.order.template` "Implantación v2" id in the NL DB (`impl_template_id` — currently null; resolved by name at runtime, confirm it exists)
- [ ] `team_email_domain`, `contact` (email/web/tel)
- [ ] `core_module_label` + `core_module_set_exists` (does a Dynapps-Core-equivalent set exist in NL?)
- [ ] **[verify]** `odoo_partner_manager` (name/email) — Odoo-SA-assigned per entity
- [ ] **[verify]** `legal_entity` (name / VAT / address); `governing_law` = Nederland ✓
- [ ] `offices` phrasing ✓ (NL); `credibility.claims_text` **reviewed for misleidende-reclame law** (do not reuse unqualified "Nr.1 wereldwijd")

### Pricing (`pricing.json`) — market pricing owner
- [ ] Rates / app base prices / complement prices confirmed for the NL market (currently inherited from ES/EN — verify)
- [ ] **[verify]** Odoo licence €/user (year+month) — currently `-1` sentinel; set by Odoo SA per country+plan (do not invent)
- [ ] **[verify]** all `product_id` (POC, IA catalog) in the NL DB
- [ ] `recurrent_lines` `fallback_price_display` for dyn_cloud / maintenance / odoo_license

### Fiscal / legal (`fiscal.json`) — **named NL finance/legal sign-off REQUIRED**
- [ ] `seller_vat_number` (NL BTW-nummer of the selling entity)
- [ ] E-invoicing regime confirmed as authored (b2g_only; Peppol/NLCIUS/SI-UBL; Logius/NPa) ✓ researched — **confirm with reviewer**
- [ ] `chart_of_accounts` (RGS) copy ✓ — confirm
- [ ] `tax_filings` (BTW-aangifte, ICP-opgaaf) + `odoo_module` technical names in NL DB
- [ ] `payment_terms.late_payment_clause` ✓ authored (art. 6:119a BW; handelsrente ECB+8pp = 10,50% H1-2026; max 60d B2B; groot→mkb 30d) — **confirm with reviewer** (inline <<VERIFY>> flag stays until sign-off)
- [ ] fineprint web/pdf ✓ — confirm; `travel_clause` ✓ — confirm; `carriers` ✓ (PostNL/DHL/DPD/GLS)
- [ ] **Named reviewer + date recorded here:** ________________________

### Content / matchers — **native-speaker + consultant sign-off**
- [ ] `apps_catalog.json` Dutch prose reviewed by native speaker
- [ ] `matching.json` signals/glossary/cases reviewed (NL fiscal terms fire: ICP, BTW verlegd, Peppol) ✓ authored — confirm
- [ ] `strings.json` Dutch UI reviewed by native speaker
- [ ] `brand_lint.json` lexicons reviewed
- [ ] International success-case narratives (`matching.cases.case_translations`) reviewed in Dutch ✓ authored — confirm
- [ ] **Named native-speaker reviewer + date recorded here:** ________________________

**Do not rename keys** — key-parity with `en` is CI-enforced (replace values only).
**Keep archetype join keys unchanged:** `multi-entity / excel-legacy / b2b-webshop / fast-golive / scaling`.
