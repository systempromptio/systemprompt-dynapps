# Spain (l10n_es) Ecosystem — Overview

Sources scanned (Odoo 18 unless noted):
- CE: `odoo18/addons/l10n_es*` — 9 modules
- Enterprise: `odoo_enterprise/18/l10n_es*` — 5 modules
- OCA: `OCA/v18/l10n-spain/` — 49 modules; `OCA/v19/l10n-spain/` — 13 modules

## 1. Module map (v18)

### CE (`odoo18/addons/`)
| Module | Purpose | LoC (models) |
|---|---|---|
| `l10n_es` | PGCE 2008 chart, taxes, fiscal positions, mod 111/115/303 tax tags | — |
| `l10n_es_edi_sii` | SII SOAP for AEAT + Bizkaia + Gipuzkoa | 822 |
| `l10n_es_edi_tbai` | TicketBAI for Araba/Bizkaia/Gipuzkoa (Basque Country) | 1718 |
| `l10n_es_edi_facturae` | Facturae 3.2.2 XML signing for B2G | 1226 |
| `l10n_es_edi_verifactu` | Veri*Factu (mainland AEAT, mandatory 2026) | — |
| `l10n_es_edi_tbai_pos` / `l10n_es_edi_verifactu_pos` | POS bridges | — |
| `l10n_es_modelo130` | mod 130 IRPF empresarios (CE only — moved to `l10n_es` in v19) | — |
| `l10n_es_pos` | POS Spanish particulars | — |

### Enterprise (`odoo_enterprise/18/`)
| Module | Purpose |
|---|---|
| `l10n_es_reports` | Reports 303/347/349/111/115/390 + Libro Diario + Libros IVA exports |
| `l10n_es_reports_2024` | mod 303 rectificativa fields update |
| `l10n_es_reports_modelo130` | mod 130 (Enterprise-side wizard) |
| `l10n_es_real_estates` | mod 347 real-estate operations |
| `l10n_es_sale_amazon` | Amazon connector specifics |

### OCA (`OCA/v18/l10n-spain/`) — 49 modules. Highlights:
| Module | Purpose |
|---|---|
| `l10n_es_aeat` | AEAT base: report mixin, tax mapping engine, BOE export, certificate handling |
| `l10n_es_aeat_mod303/mod347/mod349/mod111/mod115/mod130/mod190/mod216/mod296/mod123/mod390` | Each AEAT tax model |
| `l10n_es_aeat_mod303_oss`, `l10n_es_aeat_mod303_vat_prorate`, `l10n_es_aeat_mod369` | OSS / prorrata / OSS general |
| `l10n_es_aeat_mod390_oss`, `l10n_es_aeat_mod390_vat_prorate` | mod 390 variants |
| `l10n_es_aeat_sii_oca` | OCA SII (parallel impl. to Enterprise SII) |
| `l10n_es_aeat_sii_match`, `l10n_es_aeat_sii_invoice_summary`, `l10n_es_aeat_sii_oss` | SII addons |
| `l10n_es_facturae`, `l10n_es_facturae_face`, `l10n_es_facturae_sale_stock` | Facturae + FACe submission |
| `l10n_es_verifactu_oca` | OCA Veri*Factu (alternative to Enterprise) |
| `l10n_es_vat_book`, `l10n_es_vat_book_oss`, `l10n_es_vat_book_pos` | Libros registro IVA |
| `l10n_es_vat_prorate` | Prorrata de IVA |
| `l10n_es_atc`, `l10n_es_atc_mod420` | Canary Islands ATC + mod 420 IGIC |
| `l10n_es_intrastat_report`, `l10n_es_mis_report`, `l10n_es_partner`, `l10n_es_partner_mercantil` | Misc |

**Notable absence in OCA**: TicketBAI is **NOT** in OCA `l10n-spain` (no `l10n_es_ticketbai*` module). Bizkaia/Araba/Gipuzkoa TicketBAI is **Enterprise-only** (`l10n_es_edi_tbai` lives in `odoo18/addons/`, technically CE — see ESG29).

## 2. v19 port status (OCA)

Source: `OCA/v19/l10n-spain/` only contains 13 modules vs 49 in v18. Per memory note `[Odoo Deep Knowledge]`, ~80% NOT ported.

| Module | v18 | v19 | Status |
|---|---|---|---|
| `l10n_es_aeat` | ✓ (1.3.8) | ✓ (1.0.1) | Ported |
| `l10n_es_aeat_mod303` | ✓ | ✓ | Ported |
| `l10n_es_aeat_mod347` | ✓ | ✓ | Ported |
| `l10n_es_aeat_mod349` | ✓ | ✓ | Ported |
| `l10n_es_aeat_mod111` | ✓ | ✓ | Ported |
| `l10n_es_aeat_mod115` | ✓ | ✓ | Ported |
| `l10n_es_aeat_mod123` | ✓ | ✓ | Ported |
| `l10n_es_aeat_mod390` | ✓ | ✓ | Ported |
| `l10n_es_aeat_partner_check` | ✓ | ✓ | Ported |
| `l10n_es_partner` | ✓ | ✓ | Ported |
| `l10n_es_toponyms` | ✓ | ✓ | Ported |
| `l10n_es_account_banking_sepa_fsdd` | ✓ | ✓ | Ported |
| `l10n_es_account_statement_import_n43` | ✓ | ✓ | Ported |
| `l10n_es_aeat_mod130` | ✓ | ✗ | **NOT PORTED** (CE absorbed into `l10n_es` in v19) |
| `l10n_es_aeat_mod190` | ✓ | ✗ | **NOT PORTED** |
| `l10n_es_aeat_mod216` | ✓ | ✗ | **NOT PORTED** |
| `l10n_es_aeat_mod296` | ✓ | ✗ | **NOT PORTED** |
| `l10n_es_aeat_mod303_oss` | ✓ | ✗ | **NOT PORTED** (BLOQUEANTE for OSS clients) |
| `l10n_es_aeat_mod303_vat_prorate` | ✓ | ✗ | **NOT PORTED** (BLOQUEANTE for prorrata) |
| `l10n_es_aeat_mod369` | ✓ | ✗ | **NOT PORTED** |
| `l10n_es_aeat_mod390_oss` | ✓ | ✗ | **NOT PORTED** |
| `l10n_es_aeat_mod390_vat_prorate` | ✓ | ✗ | **NOT PORTED** |
| `l10n_es_aeat_sii_oca` | ✓ | ✗ | **NOT PORTED** (BLOQUEANTE if not using Enterprise SII) |
| `l10n_es_aeat_sii_match` / `_invoice_summary` / `_oss` | ✓ | ✗ | NOT PORTED |
| `l10n_es_facturae` / `_face` / `_sale_stock` | ✓ | ✗ | **NOT PORTED** (BLOQUEANTE for FACe) |
| `l10n_es_verifactu_oca` | ✓ | ✗ | **NOT PORTED** |
| `l10n_es_vat_book` (+ OSS, POS, summary) | ✓ | ✗ | **NOT PORTED** |
| `l10n_es_vat_prorate` | ✓ | ✗ | **NOT PORTED** |
| `l10n_es_atc` / `l10n_es_atc_mod420` | ✓ | ✗ | NOT PORTED (Canary IGIC) |
| `l10n_es_mis_report` | ✓ | ✗ | NOT PORTED |
| `l10n_es_intrastat_report` | ✓ | ✗ | NOT PORTED (Enterprise has `l10n_es_intrastat` in v19) |
| `l10n_es_account_asset` | ✓ | ✗ | NOT PORTED |
| `l10n_es_cnae`, `l10n_ca_es_cnae` | ✓ | ✗ | NOT PORTED |
| `l10n_es_facturae_face` | ✓ | ✗ | **NOT PORTED** |
| `l10n_es_pos_oca`, `l10n_es_pos_sii` | ✓ | ✗ | NOT PORTED |
| `payment_redsys`, `delivery_*` | ✓ | ✗ | NOT PORTED |

**Blocker assessment for v19 migration**:
- Companies on **OCA SII** (`l10n_es_aeat_sii_oca`) have NO v19 path — must migrate to Enterprise `l10n_es_edi_sii` or wait
- Companies invoicing public admin (Facturae+FACe) → Enterprise `l10n_es_edi_facturae` does NOT do FACe submission, only XML signing → keep on v18 OR build custom FACe
- Prorrata IVA / OSS → Enterprise `l10n_es_reports` doesn't cover these → blocker
- Libro IVA (`l10n_es_vat_book`) → Enterprise has `Libros export` in `l10n_es_reports/models/libros_export.py` (390 lines) — partial coverage

## 3. Decision tree: Enterprise vs OCA

### SII (Suministro Inmediato)
```
Mainland AEAT or Bizkaia/Gipuzkoa SOAP submission?
├── Enterprise customer with annual support → l10n_es_edi_sii (OEEL/CE-LGPL)
│   • account_edi format-based, integrated with account_move state
│   • Tag-based source-of-truth (l10n_es_type + tax tags)
│   • Auto-batches by (move_type, csv) — file:l10n_es_edi_sii/models/account_edi_format.py:584
└── CE customer / fine-grained control / SII-only feature gaps
    └── l10n_es_aeat_sii_oca
        • Uses aeat.mixin chain (filter-based domain over move lines)
        • sii_mixin.py:888 lines vs Enterprise 822 — more configurability
        • Has match (deepdiff vs AEAT consultas), summary, OSS extensions
```

### AEAT model reports (303/347/349/111/115/390/130)
```
                  Enterprise                    OCA
Source-of-truth:  account.report tax tags       l10n.es.aeat.map.tax (filter on tax XMLID)
Engine:           account.tax.report.handler    l10n.es.aeat.report.tax.mapping (mixin)
BOE export:       Inline in handler             aeat.model.export.config (line-by-line CSV-driven)
Regularization:   Manual (no helper)            create_regularization_move() built-in
Wizard data:      Per-model BOE wizards         Generic export_to_boe_wizard
Customization:    Subclass handler              Inherit map + override mixin
```

Choose Enterprise when: client already has Enterprise + standard regimes + simple BOE.
Choose OCA when: complex regimes (REAGYP + recargo + prorrata + OSS), custom maps, regularization automation needed, free CE.

### TicketBAI (País Vasco)
```
Only Enterprise path: l10n_es_edi_tbai.
OCA has NO TicketBAI module. (See ESG29.)
```

### Facturae (B2G public admin)
```
                  Enterprise                    OCA
XML 3.2.2:        l10n_es_edi_facturae          l10n_es_facturae
Signing:          ✓ (XAdES-BES)                 ✓
FACe upload:      ✗ (manual download + upload)  ✓ l10n_es_facturae_face (cron polling)
```

### Veri*Factu (mainland, mandatory 2026 self-employed / 2027 corp)
```
                  Enterprise                    OCA
SOAP submit:      l10n_es_edi_verifactu (CE!)   l10n_es_verifactu_oca
Hash chain:       ✓ (chain_index strict)         ✓
QR code:          ✓                              ✓
v19 status:       ✓ Enterprise                  ✗ OCA NOT PORTED
```

## 4. Reading guide

- `aeat_models.md` — per-model legal scope + computation
- `sii.md` — SII deep dive (Enterprise vs OCA mappings, tax agency endpoints)
- `ticketbai.md` — TBAI hash chain, regional differences, refund flow
- `facturae.md` — XML structure, signing, FACe
- `verifactu.md` — Veri\*Factu CE: documento encadenado, huella SHA-256, lotes SOAP, QR, anulación, POS
- `gotchas.md` — 55 ESG-prefixed gotchas (tax tags, prorrata, REAGYP, retenciones, 347 thresholds, Veri\*Factu ESG41-55)
- `v19_diff.md` — v18→v19 deltas
