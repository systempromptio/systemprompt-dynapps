# l10n_es — v18 → v19 Differences

## 1. Core l10n_es

### Module manifest
| Field | v18 | v19 |
|---|---|---|
| `version` | `5.4` | `5.4` (same string, different semantics) |
| `depends` | `account, base_iban, base_vat` | `+ account_edi_ubl_cii` |
| Description | "tax reports mod 111, 115 and 303" | "tax reports mod 111, 115, **130** and 303" |
| `data` | mod303 only | + `data/mod130.xml`, `data/res_country_group.xml` |

### Absorbed module
- `l10n_es_modelo130` (CE addon in v18) → **merged into `l10n_es`** in v19.
- Enterprise `l10n_es_reports_modelo130` still exists in v18; no separate equivalent in v19 (handled by core `l10n_es`).

### New `controllers/` directory
v19 only: `odoo19/addons/l10n_es/controllers/` — adds Spanish-specific HTTP routes (likely for FACe redirect or tax verification widget).

### UBL/CII auto-generation
The new `account_edi_ubl_cii` dependency means any Spanish invoice can natively generate UBL 2.1 / Factur-X / Peppol BIS without extra modules. Impact: B2B customers in EU receive Peppol-compliant XML automatically.

## 2. l10n_es_edi_sii

### Manifest changes
v19: `+ 'account_edi'` added to depends (back from being an internal helper). Otherwise identical structure.

### Logic
File diff `account_edi_format.py`: structurally same. Tax handling, regime codes, error 1117 retry — all preserved.

## 3. l10n_es_edi_tbai

Manifest changes: minor — license stays LGPL-3, depends list identical.
Models: equivalent code structure. Hash chain logic, signature templates, agency endpoints unchanged.

## 4. l10n_es_edi_facturae

v18 → v19: depends list unchanged. License LGPL-3. The `auto_install: ['l10n_es']` is preserved.

## 5. l10n_es_edi_verifactu

Same in v18 and v19. Production-mandatory in 2026 (autonomous) / 2027 (corp) — both versions support it.

## 6. Enterprise: l10n_es_reports

v19 adds `l10n_es_reports_2025` (was 2024 in v18):
```python
'name': "Spain - Accounting Reports (2025 Update)",
'description': "Backports the tax return functionalities in the Spanish localization",
```

The 2025 update introduces:
- `account_return` integration (data/account_return_data.xml)
- New BOE wizard validation (`wizard/l10n_es_boe_wizard_validation.xml`)
- Updated submission wizards (`l10n_es_return_submission_wizards.xml`)

This aligns Spanish tax returns with the new generic `account.return` framework introduced in v18 → fully adopted v19.

## 7. New v19 Enterprise module: l10n_es_intrastat

- **v18**: no `l10n_es_intrastat` Enterprise module (had OCA `l10n_es_intrastat_report` only)
- **v19**: native Enterprise `l10n_es_intrastat`

Impact: Spanish companies with intra-EU goods movement get Intrastat declaration support out-of-the-box on v19.

## 8. OCA v18 → v19 status (recap)

49 modules in v18 → 13 modules in v19. **~73% NOT PORTED** (close to the 80% noted in memory).

### Ported (13)
- `l10n_es_aeat`, `l10n_es_aeat_mod111/115/123/303/347/349/390`
- `l10n_es_aeat_partner_check`
- `l10n_es_partner`, `l10n_es_toponyms`
- `l10n_es_account_banking_sepa_fsdd`, `l10n_es_account_statement_import_n43`

### NOT PORTED (36) — blockers categorized

#### Critical for daily operations
| Module | Blocker for |
|---|---|
| `l10n_es_aeat_sii_oca` | Companies on OCA SII stack |
| `l10n_es_aeat_mod303_oss`, `l10n_es_aeat_mod390_oss`, `l10n_es_aeat_sii_oss`, `l10n_es_aeat_mod369` | All OSS clients |
| `l10n_es_aeat_mod303_vat_prorate`, `l10n_es_aeat_mod390_vat_prorate`, `l10n_es_vat_prorate` | All prorrata clients (medical, education, mixed) |
| `l10n_es_facturae`, `l10n_es_facturae_face`, `l10n_es_facturae_sale_stock` | B2G public admin auto-FACe submission |
| `l10n_es_aeat_mod190` | Annual IRPF summary (no Enterprise equivalent) |
| `l10n_es_aeat_mod130` | Self-employed pago fraccionado (covered by Enterprise + core in v19) |
| `l10n_es_vat_book` (+ OSS, POS, summary) | Libros registro IVA detail |

#### Operational
| Module | Blocker for |
|---|---|
| `l10n_es_aeat_sii_match` | Reconciliation with AEAT records |
| `l10n_es_aeat_sii_invoice_summary` | Invoice summary in SII |
| `l10n_es_verifactu_oca` | OCA-only Veri*Factu users |

#### Regional
| Module | Blocker for |
|---|---|
| `l10n_es_atc`, `l10n_es_atc_mod420` | Canary Islands IGIC + mod 420 |
| `l10n_es_aeat_mod216`, `l10n_es_aeat_mod296` | Non-resident withholdings |
| `l10n_es_aeat_mod123` | Withholdings on capital income |

#### Asset & misc
| Module | Blocker for |
|---|---|
| `l10n_es_account_asset` | Asset depreciation (covered by Enterprise) |
| `l10n_es_intrastat_report` | Intrastat (covered by new Enterprise `l10n_es_intrastat` in v19) |
| `l10n_es_mis_report` | MIS reporting (no Spanish-specific reports) |
| `l10n_es_cnae`, `l10n_ca_es_cnae`, `l10n_es_partner_mercantil` | CNAE codes + mercantile partner data |
| `l10n_es_pos_oca`, `l10n_es_pos_sii` | POS Spanish features (covered by Enterprise `l10n_es_pos`) |
| `l10n_es_payment_order_confirming_aef`, `l10n_es_payment_order_confirming_sabadell` | Confirming bank export |
| `payment_redsys`, `delivery_dhl_parcel`, `delivery_gls_asm`, `delivery_seur_atlas`, `account_promissory_note_caixabank` | Spanish payment + delivery providers |

## 9. Migration recommendations

### Stay on v18 if
- Heavy OCA SII user with custom map adjustments
- Prorrata IVA clients
- OSS volume > 10% of total turnover
- B2G with auto-FACe submission needed
- Multi-region (Canary IGIC + mainland)

### Migrate to v19 if
- Enterprise SII / TBAI / Verifactu user
- Plain mod 303/347/349/111/115 (no extras)
- B2G with manual FACe upload acceptable
- Mainland-only operation

### Hybrid approach
For complex setups: keep v18 in production, set up v19 sandbox to test Enterprise replacements. Build custom replacement modules for prorrata + OSS during the transition window (12-18 months expected for OCA to catch up).

## 10. Migration script needs

When upgrading v18 → v19, watch for:

1. **mod 130 records** — uninstall `l10n_es_modelo130` (CE) and `l10n_es_reports_modelo130` (EE) BEFORE running upgrade, to prevent duplicate `account.report` records.

2. **OCA `aeat_state` field** — if migrating off `l10n_es_aeat_sii_oca`, the `aeat_state` selection on `account.move` becomes orphan. Either preserve as informational char field or write an `end-migrate` script that maps to `account.edi.document.state`.

3. **Tax tags** — l10n_es tax tag XMLIDs may have changed for the 2024-Q4 transitional rates (165-170). Review module update output for "tag not found" warnings.

4. **Veri*Factu sequences** — `l10n_es_edi_verifactu_chain_sequence_id` is per-company, no_gap. Migration must preserve sequence values to keep chain integrity.

5. **Facturae certificate scope** — v19 may rename certificate scopes; ensure `certificate.certificate.scope='facturae'` is preserved.
