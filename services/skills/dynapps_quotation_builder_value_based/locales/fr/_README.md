# FR locale pack — REVIEWED DRAFT

**Status:** `pack_status: "stub"` — schema-valid and **content-authored in French**, but still
**blocked from production** by the loader until (a) every `<<VERIFY>>` onboarding item is filled and
(b) both sign-offs (FR finance/legal + FR native speaker) are recorded in the PR.

> Note on status value: the plan calls this a "draft". The `locale.schema.json` enum only allows
> `reference | live | stub`, so a reviewed-but-not-yet-live pack uses **`stub`** (the loader's
> "blocked from production" state). Do NOT set `pack_status: "draft"` — it fails schema validation.

**UI-string fallback:** `locale.json.fallback = "en"` — any UI string not present resolves to EN.
**Validation:** `python3 scripts/validate_locale.py fr` → **0 errors** (3 expected `ref` warnings on
FR-specific tax filings `fec`/`ca3`/`emebi`, which are jurisdiction-specific and not in the shared
registry — intended).

## What is DONE (authored in this draft)

- **`strings.json`** — all UI labels authored in French (web + PDF + CLI + spreadsheet), EN key-parity.
- **`apps_catalog.json`** — per-app prose (intro/value/features/formations/migrations/core/extras) in French.
- **`matching.json`** — signals, metrics regexes, sectors, glossary, soft-skills, lang labels, case
  narratives and `sector_token_map` in French; **local fiscal terms wired into the matcher** so it
  fires on FR pain: `facture électronique`, `chorus pro`, `factur-x`, `ppf`, `pdp` (facturacion);
  `fec`, `ca3`, `pcg`, `tva` (contabilidad); `emebi`, `deb`, `état récapitulatif`, `autoliquidation`
  via reverse-charge clause (cont_intrastat). FR ERPs added (Sage/Cegid/EBP/Divalto/Quadratus/Pennylane).
- **`brand_lint.json`** — French brochure/jargon/slop/fluff lexicons + tips; `word_charclass` covers
  `œàâçèéêëîïôùûü`.
- **`fiscal.json`** — FR regime authored STRUCTURED from current facts (see Sources below):
  - `mandate_status: b2b_phased`; authority DGFiP; network = Plateforme Agréée (ex-PDP) + PPF (annuaire
    + concentrateur e-reporting) + Chorus Pro; formats Factur-X / UBL 2.1 / CII.
  - `phase_dates`: **réception** for all @ 2026-09-01; **émission** for grandes entreprises + ETI @
    2026-09-01 and for PME/TPE @ 2027-09-01.
  - `chart_of_accounts.code = PCG`; `tax_filings` typed = FEC, CA3, EMEBI + état récapitulatif (ex-DEB), SEPA.
  - `reverse_charge_clause` = autoliquidation de la TVA (art. 283 CGI); OSS/IOSS note.
  - **`payment_terms.late_payment_clause`** = pénalités de retard (taux BCE + 10 pts, sans rappel,
    art. L441-10 C. com.) and **`mandatory_mentions`** = indemnité forfaitaire de 40 € (art. D441-5).
    Both are echoed in `fineprint_web`/`fineprint_pdf`.
  - `carriers` = Chronopost/Colissimo/DPD/GLS/Geodis/messagerie palettes.
- **`format.json`** — EUR, symbole après + espace insécable (thousands = espace), séparateur décimal
  virgule, locale block fr_FR, mois FR, suffixes `/ an` `/ mois`.
- **`locale.json`** — code/lang/odoo_lang/fallback + FR `output_language`.

## What still needs LOCAL onboarding fill — `<<VERIFY>>` markers (do NOT invent)

### Instance / org (`instance.json`) — ops
- `odoo_url`, `odoo_db`, `team_email_domain`, `contact` (email/web/tel), `core_module_label`
- **[verify]** `impl_template_id` — "Implantation v2" id in the FR DB (null → discovered at runtime)
- **[verify]** `odoo_partner_manager` (name/email) — Odoo SA-assigned per entity
- **[verify]** `legal_entity` (name / VAT / address); `governing_law` already = France
- `credibility.claims_text` — **must be reviewed for DGCCRF advertising law** before any "N°1 mondial"

### Pricing (`pricing.json`) — market pricing owner
- **[verify]** Odoo licence €/user (year+month) — currently `-1` sentinel (Odoo SA sets per country+plan)
- **[verify]** all `product_id` (POC, IA catalog exclusive + addons) in the FR DB
- **[verify]** `recurrent_lines` fallback price displays (dyn_cloud, maintenance, odoo_license)
- Confirm FR-market rates and app/complement base prices (currently carried from ES/EN reference)

### Fiscal (`fiscal.json`) — **named FR finance/legal sign-off REQUIRED**
- `seller_vat_number` (TVA intracommunautaire de l'entité vendeuse)
- `tax_filings[].odoo_module` technical names in the FR DB (fec/ca3/emebi/sepa) — verify_from_odoo
- Legal review of: e-invoicing phase_dates + copy_render, late-payment clause wording, indemnité
  forfaitaire, reverse-charge citation, fineprint. Nothing here may ship without a named reviewer.

## Sign-off checklist (gate to flip `stub` → `live`)

- [ ] All `<<VERIFY>>` and `-1`/`null` sentinels replaced with confirmed values (validator enforces on `live`)
- [ ] **FR finance/legal reviewer (named in PR)** signs off `fiscal.json` + fineprint (e-invoicing calendar,
      pénalités de retard, indemnité 40 €, autoliquidation, FEC/CA3/EMEBI)
- [ ] **FR native speaker (named in PR)** signs off `strings.json`, `apps_catalog.json`,
      `matching.json` case narratives + glossary
- [ ] `credibility.claims_text` cleared against DGCCRF advertising rules
- [ ] `python3 scripts/validate_locale.py fr` green; ES golden still PASS (`tests/golden_check.py`)
- [ ] Then set `locale.json.pack_status = "live"`

**Keep archetype join keys unchanged:** `multi-entity / excel-legacy / b2b-webshop / fast-golive / scaling`.
**Do not rename keys** — replace values only (key-parity with `en` is CI-enforced).

## Sources (FR e-invoicing, confirmed July 2026)

- economie.gouv.fr — Tout savoir sur la facturation électronique pour les entreprises
- service-public.gouv.fr (entreprendre) — Facturation électronique : calendrier
- EY — France announces simplification for September 2026 e-invoicing mandate (grandes entreprises + ETI émission @ 2026-09-01)
- European Commission — eInvoicing in France (formats UBL 2.1 / CII / Factur-X, EN 16931; PDP/PPF structure)
- Cegid / Pennylane / Sage — Calendrier facture électronique 2026-2027
